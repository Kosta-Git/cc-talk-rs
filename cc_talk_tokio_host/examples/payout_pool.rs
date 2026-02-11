//! Payout Pool example - dispenses coins using multiple hoppers.
//!
//! This example demonstrates how to use the PayoutPool to manage multiple
//! hopper devices and dispense a specific value using the optimal combination
//! of coins.
//!
//! Usage: cargo run --example payout_pool [value] [socket_path]
//!
//! Arguments:
//!   value        Value to dispense in cents (default: 170)
//!   socket_path  Path to ccTalk socket (default: /tmp/cctalk.sock)
//!
//! The example assumes three hoppers:
//!   - Address 3: 1.00 EUR (100 cents)
//!   - Address 4: 0.50 EUR (50 cents)
//!   - Address 5: 0.20 EUR (20 cents)

use std::{env, time::Duration};

use cc_talk_core::cc_talk::{Category, ChecksumType, Device};
use cc_talk_tokio_host::{
    device::{
        payout::PayoutDevice,
        payout_pool::{HopperSelectionStrategy, PayoutEvent, PayoutPool},
    },
    transport::{retry::RetryConfig, tokio_transport::CcTalkTokioTransport},
};
use tokio::sync::mpsc;
use tracing::{Level, error, info, warn};

fn init_logging() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();
}

fn print_usage() {
    eprintln!("Usage: payout_pool [value] [socket_path]");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  value        Value to dispense in cents (default: 170)");
    eprintln!("  socket_path  Path to ccTalk socket (default: /tmp/cctalk.sock)");
    eprintln!();
    eprintln!("Assumes hoppers at addresses 3 (100c), 4 (50c), 5 (20c)");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let value: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(170);
    let socket_path = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "/tmp/cctalk.sock".to_string());

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return Ok(());
    }

    info!("Payout Pool Example");
    info!("Socket: {}", socket_path);
    info!(
        "Value to dispense: {} cents ({:.2} EUR)",
        value,
        value as f64 / 100.0
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
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create hopper devices
    let hopper1 = PayoutDevice::new(
        Device::new(3, Category::Payout, ChecksumType::Crc8),
        tx.clone(),
    );
    let hopper2 = PayoutDevice::new(
        Device::new(4, Category::Payout, ChecksumType::Crc8),
        tx.clone(),
    );
    let hopper3 = PayoutDevice::new(Device::new(5, Category::Payout, ChecksumType::Crc8), tx);

    // Build the payout pool
    info!("Building payout pool...");
    let pool = PayoutPool::builder()
        .add_hopper(hopper1, 100) // 1.00 EUR
        .add_hopper(hopper2, 50) // 0.50 EUR
        .add_hopper(hopper3, 10) // 0.20 EUR
        .selection_strategy(HopperSelectionStrategy::LargestFirst)
        .polling_interval(Duration::from_millis(250))
        .build();

    // Initialize the pool
    info!("Initializing pool...");
    match pool.initialize().await {
        Ok(()) => info!("Pool initialized with {} hoppers", pool.hopper_count()),
        Err(e) => {
            error!("Failed to initialize pool: {}", e);
            return Err(e.into());
        }
    }

    // Check if we can dispense the requested amount
    if !pool.can_payout(value) {
        warn!(
            "Cannot dispense exact amount {} - some remainder may be left",
            value
        );
    }

    // Poll hopper inventories
    info!("Checking hopper inventories...");
    let inventory_result = pool.poll_inventories().await;
    for inv in &inventory_result.inventories {
        info!(
            "  Hopper {} ({} cents): {}",
            inv.address, inv.value, inv.level
        );
    }
    for err in &inventory_result.errors {
        warn!("  Hopper {} error: {}", err.address, err.error);
    }

    // Create event channel for this payout
    let (event_tx, mut event_rx) = mpsc::channel::<PayoutEvent>(16);

    // Spawn event monitor
    let event_handle = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                PayoutEvent::Progress(progress) => {
                    if !progress.done {
                        info!(
                            "Progress: dispensed {} of {} cents ({} coins)",
                            progress.dispensed,
                            progress.requested,
                            progress.coins_count()
                        );
                    }
                }
                PayoutEvent::HopperEmpty {
                    address,
                    coin_value,
                } => {
                    warn!("Hopper {} ({} cents) is empty!", address, coin_value);
                }
                PayoutEvent::PlanRebalanced {
                    exhausted_hopper,
                    remaining_value,
                    ..
                } => {
                    info!(
                        "Rebalanced after hopper {} emptied, {} cents remaining",
                        exhausted_hopper, remaining_value
                    );
                }
                PayoutEvent::HopperError { address, error } => {
                    error!("Hopper {} error: {}", address, error);
                }
            }
        }
    });

    // Execute payout with events
    info!("Dispensing {} cents...", value);
    let result = pool.payout_with_events(value, event_tx).await;

    // Wait for event monitor to finish
    let _ = event_handle.await;

    match result {
        Ok(progress) => {
            info!("Payout complete!");
            info!("  Requested: {} cents", progress.requested);
            info!("  Dispensed: {} cents", progress.dispensed);
            info!("  Remaining: {} cents", progress.remaining);
            info!(
                "  Coins: {} ({:?})",
                progress.coins_count(),
                progress.coins_dispensed
            );
            if !progress.empty_hoppers.is_empty() {
                warn!("  Empty hoppers: {:?}", progress.empty_hoppers);
            }
        }
        Err(e) => {
            error!("Payout failed: {}", e);
        }
    }

    info!("Done");
    Ok(())
}
