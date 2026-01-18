//! Hopper dispense example - dispenses coins from a hopper.
//!
//! Usage: cargo run --example hopper_dispense [coins] [socket_path]
//!
//! Arguments:
//!   coins        Number of coins to dispense (default: 3)
//!   socket_path  Path to ccTalk socket (default: /tmp/cctalk.sock)

use std::{env, time::Duration};

use cc_talk_core::cc_talk::{Category, ChecksumType, Device};
use cc_talk_tokio_host::{
    device::{base::DeviceCommon, payout::PayoutDevice},
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

fn print_usage() {
    eprintln!("Usage: hopper_dispense [coins] [socket_path]");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  coins        Number of coins to dispense (default: 3)");
    eprintln!("  socket_path  Path to ccTalk socket (default: /tmp/cctalk.sock)");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let coins: u8 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let socket_path = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "/tmp/cctalk.sock".to_string());

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return Ok(());
    }

    info!("Hopper Dispense Example");
    info!("Socket: {}", socket_path);
    info!("Coins to dispense: {}", coins);

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

    // Create hopper device (address 3 is common for hoppers)
    let hopper = PayoutDevice::new(Device::new(3, Category::Payout, ChecksumType::Crc8), tx);

    // Check connectivity
    info!("Connecting to hopper...");
    hopper.simple_poll().await?;
    info!("Connected");

    // Display device info
    let manufacturer = hopper.get_manufacturer_id().await?;
    let serial = hopper.get_serial_number().await?;
    info!("Device: {} (S/N: {})", manufacturer, serial);

    // Enable and dispense
    hopper.enable_hopper().await?;
    info!("Hopper enabled");

    hopper.payout_serial_number(coins).await?;
    info!("Dispensing {} coins...", coins);

    // Monitor payout status
    loop {
        match hopper.get_payout_status().await {
            Ok(status) => {
                info!("Status: {} coins remaining", status.coins_remaining);
                if status.coins_remaining == 0 {
                    break;
                }
            }
            Err(e) => {
                error!("Status error: {}", e);
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    hopper.disable_hopper().await?;
    info!("Hopper disabled");
    info!("Dispense complete");

    Ok(())
}
