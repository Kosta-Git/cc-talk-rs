//! Simple payout pool example with two hoppers.
//!
//! Usage: cargo run --example payout_pool_two_hoppers [value] [socket_path]
//!
//! Arguments:
//!   value        Value to dispense in cents (default: 90)
//!   socket_path  Path to ccTalk socket (default: /tmp/cctalk.sock)
//!
//! Assumes two hoppers:
//!   - Address 3: 0.50 EUR (50 cents)
//!   - Address 4: 0.20 EUR (20 cents)

use std::{env, time::Duration};

use cc_talk_core::cc_talk::{Category, ChecksumType, Device};
use cc_talk_tokio_host::{
    device::{
        base::DeviceCommon,
        payout::PayoutDevice,
        payout_pool::{HopperSelectionStrategy, PayoutPool},
    },
    transport::{retry::RetryConfig, tokio_transport::CcTalkTokioTransport},
};
use tokio::sync::mpsc;
use tracing::{Level, error, info};

fn init_logging() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let value: u32 = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(90);
    let socket_path = env::args()
        .nth(2)
        .unwrap_or_else(|| "/tmp/cctalk.sock".to_string());

    info!("Two-Hopper Payout Pool Example");
    info!("Socket: {}", socket_path);
    info!("Value: {} cents ({:.2} EUR)", value, value as f64 / 100.0);

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

    // Create two hopper devices
    let hopper_50c = PayoutDevice::new(
        Device::new(3, Category::Payout, ChecksumType::Crc8),
        tx.clone(),
    );
    let hopper_20c = PayoutDevice::new(
        Device::new(4, Category::Payout, ChecksumType::Crc8),
        tx,
    );

    // Verify connectivity
    info!("Checking hoppers...");
    hopper_50c.simple_poll().await?;
    info!("  Hopper 50c (addr 3): online");
    hopper_20c.simple_poll().await?;
    info!("  Hopper 20c (addr 4): online");

    // Build and initialize pool
    let mut pool = PayoutPool::builder()
        .add_hopper(hopper_50c, 50)
        .add_hopper(hopper_20c, 20)
        .with_selection_strategy(HopperSelectionStrategy::LargestFirst)
        .build();

    pool.initialize().await?;
    info!("Pool initialized with {} hoppers", pool.hopper_count());

    // Enable hoppers
    pool.enable_all().await?;

    // Execute payout
    info!("Dispensing {} cents...", value);
    let result = pool.payout(value).await?;

    info!("Payout complete!");
    info!("  Dispensed: {} cents", result.dispensed);
    info!("  Coins: {:?}", result.coins_dispensed);

    // Disable hoppers
    pool.disable_all().await?;

    Ok(())
}
