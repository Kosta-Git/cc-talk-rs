//! Coin validator example - polls for coin insertions and displays events.
//!
//! Usage: cargo run --example coin_validator [socket_path]
//!
//! Arguments:
//!   socket_path  Path to ccTalk socket (default: /tmp/cctalk.sock)

use std::{env, time::Duration};

use cc_talk_core::cc_talk::{Address, Category, ChecksumType, CoinEvent, CurrencyToken, Device};
use cc_talk_tokio_host::{
    device::{base::DeviceCommon, coin_validator::CoinValidator},
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let socket_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/cctalk.sock".to_string());

    info!("Coin Validator Example");
    info!("Socket: {}", socket_path);

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

    // Create coin validator
    let address = match Category::CoinAcceptor.default_address() {
        Address::Single(addr) | Address::SingleAndRange(addr, _) => addr,
    };
    let validator = CoinValidator::new(
        Device::new(address, Category::CoinAcceptor, ChecksumType::Crc8),
        tx,
    );

    // Check connectivity
    info!("Connecting to coin validator...");
    validator.simple_poll().await?;
    info!("Connected");

    // Display device info
    let manufacturer = validator.get_manufacturer_id().await?;
    let serial = validator.get_serial_number().await?;
    let product = validator.get_product_code().await?;
    info!("Device: {} {} (S/N: {})", manufacturer, product, serial);

    // Display configured coins
    info!("Configured coins:");
    for (pos, token) in validator
        .request_all_coin_id()
        .await?
        .iter()
        .filter_map(|(p, t)| t.as_ref().map(|t| (*p, t)))
    {
        match token {
            CurrencyToken::Token => info!("  [{}] Token", pos),
            CurrencyToken::Currency(v) => {
                info!("  [{}] {}{:.2}", pos, v.country_code(), v.monetary_value())
            }
        }
    }

    // Enable acceptance
    validator.disable_master_inhibit().await?;
    validator.set_all_coin_inhibits(false).await?;
    info!("Coin acceptance enabled");

    // Get polling interval
    let delay = validator
        .get_polling_priority()
        .await?
        .as_duration()
        .unwrap_or(Duration::from_millis(100));

    info!("Polling (interval: {:?})... Press Ctrl+C to stop", delay);

    // Poll loop
    let mut last_counter = 0u8;
    loop {
        match validator.poll().await {
            Ok(poll) => {
                if poll.event_counter == last_counter {
                    tokio::time::sleep(delay).await;
                    continue;
                }
                last_counter = poll.event_counter;

                if poll.lost_events > 0 {
                    warn!("Lost {} events", poll.lost_events);
                }

                for event in poll.events {
                    match event {
                        CoinEvent::Credit(credit) => {
                            info!(
                                "Coin accepted: position {} -> path {:?}",
                                credit.credit, credit.sorter_path
                            );
                        }
                        CoinEvent::Error(e) => {
                            warn!("Error: {}", e.description());
                        }
                        CoinEvent::Reset => {
                            info!("Device reset detected");
                        }
                    }
                }
            }
            Err(e) => {
                error!("Poll error: {}", e);
            }
        }
        tokio::time::sleep(delay).await;
    }
}
