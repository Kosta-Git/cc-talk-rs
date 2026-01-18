//! Currency acceptor pool example - accepts payment using coin and bill validators.
//!
//! Usage: cargo run --example currency_acceptor_pool <amount_cents> [socket_path]
//!
//! Arguments:
//!   amount_cents  Payment amount in euro cents (required)
//!   socket_path   Path to ccTalk socket (default: /tmp/cctalk.sock)
//!
//! Examples:
//!   cargo run --example currency_acceptor_pool 500          # Accept 5.00 EUR
//!   cargo run --example currency_acceptor_pool 1250         # Accept 12.50 EUR
//!   cargo run --example currency_acceptor_pool 100 /dev/ttyUSB0

use std::{env, process, time::Duration};

use cc_talk_core::cc_talk::{Address, Category, ChecksumType, Device};
use cc_talk_tokio_host::{
    device::{
        base::DeviceCommon,
        bill_validator::BillValidator,
        coin_validator::CoinValidator,
        currency_acceptor_pool::{
            BillRoutingMode, CurrencyAcceptorPool, PaymentProgress, PoolError,
        },
    },
    transport::{retry::RetryConfig, tokio_transport::CcTalkTokioTransport},
};
use tokio::sync::{mpsc, oneshot};
use tracing::{Level, error, info, warn};

fn init_logging() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();
}

fn print_usage() {
    eprintln!("Usage: currency_acceptor_pool <amount_cents> [socket_path]");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  amount_cents  Payment amount in euro cents (required)");
    eprintln!("  socket_path   Path to ccTalk socket (default: /tmp/cctalk.sock)");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  currency_acceptor_pool 500          # Accept 5.00 EUR");
    eprintln!("  currency_acceptor_pool 1250         # Accept 12.50 EUR");
    eprintln!("  currency_acceptor_pool 100 /dev/ttyUSB0");
}

fn format_cents(cents: u32) -> String {
    format!("{:.2} EUR", cents as f64 / 100.0)
}

/// Handles progress updates from the payment process
async fn handle_progress(mut progress_rx: mpsc::Receiver<PaymentProgress>) {
    while let Some(progress) = progress_rx.recv().await {
        info!(
            "  + {} from {} -> Total: {} / {} (remaining: {})",
            format_cents(progress.credit.value),
            progress.credit.source,
            format_cents(progress.total_received),
            format_cents(progress.target_value),
            format_cents(progress.remaining),
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let args: Vec<String> = env::args().collect();

    // Check for help flag
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return Ok(());
    }

    // Parse amount (required)
    let amount_cents: u32 = match args.get(1).and_then(|s| s.parse().ok()) {
        Some(amount) if amount > 0 => amount,
        _ => {
            print_usage();
            process::exit(1);
        }
    };

    let socket_path = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "/tmp/cctalk.sock".to_string());

    info!("Currency Acceptor Pool Example");
    info!("Socket: {}", socket_path);
    info!(
        "Target amount: {} ({} cents)",
        format_cents(amount_cents),
        amount_cents
    );

    // Setup transport
    let (tx, rx) = mpsc::channel(32);
    let transport = CcTalkTokioTransport::new(
        rx,
        socket_path,
        Duration::from_millis(100),
        Duration::from_millis(100),
        RetryConfig::default(),
        true,
    );

    tokio::spawn(async move {
        if let Err(e) = transport.run().await {
            error!("Transport error: {}", e);
        }
    });
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Create coin validator
    let coin_address = match Category::CoinAcceptor.default_address() {
        Address::Single(addr) | Address::SingleAndRange(addr, _) => addr,
    };
    let coin_validator = CoinValidator::new(
        Device::new(coin_address, Category::CoinAcceptor, ChecksumType::Crc8),
        tx.clone(),
    );

    // Create bill validator
    let bill_address = match Category::BillValidator.default_address() {
        Address::Single(addr) | Address::SingleAndRange(addr, _) => addr,
    };
    let bill_validator = BillValidator::new(
        Device::new(bill_address, Category::BillValidator, ChecksumType::Crc16),
        tx,
    );

    // Check device connectivity
    info!("Checking devices...");
    match coin_validator.simple_poll().await {
        Ok(_) => {
            info!("  Coin validator: online");
            coin_validator.reset_device().await?;
        }
        Err(e) => warn!("  Coin validator: {} (continuing)", e),
    }
    match bill_validator.simple_poll().await {
        Ok(_) => {
            info!("  Bill validator: online");
            bill_validator.reset_device().await?;
        }
        Err(e) => warn!("  Bill validator: {} (continuing)", e),
    }

    // Build and initialize pool
    info!("Initializing pool...");
    let pool = CurrencyAcceptorPool::builder()
        .add_coin_validator(coin_validator)
        .add_bill_validator(bill_validator)
        .with_denomination_range(1, 50000) // 0.01 to 500.00 EUR
        .with_bill_routing_mode(BillRoutingMode::AutoStack)
        .with_polling_interval(Duration::from_millis(100))
        .build_and_initialize()
        .await?;

    info!(
        "Pool ready: {} coin validator(s), {} bill validator(s)",
        pool.coin_validator_count(),
        pool.bill_validator_count()
    );

    // Setup progress and cancellation channels
    let (_cancel_tx, cancel_rx) = oneshot::channel();
    let (progress_tx, progress_rx) = mpsc::channel(16);

    // Spawn progress handler
    let progress_handle = tokio::spawn(handle_progress(progress_rx));

    // Accept payment
    info!("Waiting for payment... (60 minutes timeout)");
    info!("Insert coins or bills now!");
    info!("---");

    let result = pool
        .accept_payment_with_progress(
            amount_cents,
            Duration::from_mins(60),
            cancel_rx,
            Some(progress_tx),
        )
        .await;

    // Wait for progress handler to finish
    let _ = progress_handle.await;

    info!("---");

    match result {
        Ok(result) => {
            info!("Payment successful!");
            info!(
                "  Received: {} ({} cents)",
                format_cents(result.total_received),
                result.total_received
            );
            info!("  Credits: {} ({:?})", result.credits.len(), result.credits);

            if result.total_received > amount_cents {
                let overpaid = result.total_received - amount_cents;
                info!("  Overpayment: {}", format_cents(overpaid));
            }
        }
        Err(PoolError::PaymentTimeout {
            target,
            received,
            credits,
        }) => {
            error!("Payment timed out");
            error!(
                "  Target: {}, Received: {}",
                format_cents(target),
                format_cents(received)
            );
            if !credits.is_empty() {
                error!("  Partial credits: {}", credits.len());
            }
            process::exit(1);
        }
        Err(PoolError::PaymentCancelled {
            target,
            received,
            credits,
        }) => {
            warn!("Payment cancelled");
            warn!(
                "  Target: {}, Received: {}",
                format_cents(target),
                format_cents(received)
            );
            if !credits.is_empty() {
                warn!("  Credits received: {}", credits.len());
            }
        }
        Err(e) => {
            error!("Payment failed: {}", e);
            process::exit(1);
        }
    }

    Ok(())
}
