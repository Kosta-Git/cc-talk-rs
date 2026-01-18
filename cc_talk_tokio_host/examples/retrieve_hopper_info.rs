//! Hopper info example - retrieves and displays hopper device information.
//!
//! Usage: cargo run --example retrieve_hopper_info [socket_path]
//!
//! Arguments:
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let socket_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/cctalk.sock".to_string());

    info!("Hopper Info Example");
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

    // Create hopper device (address 3 is common for hoppers)
    let hopper = PayoutDevice::new(Device::new(3, Category::Payout, ChecksumType::Crc8), tx);

    // Check connectivity
    info!("Connecting to hopper...");
    hopper.simple_poll().await?;
    info!("Connected");

    // Display device info
    info!("Device Information:");
    info!(
        "  Manufacturer:      {}",
        hopper.get_manufacturer_id().await?
    );
    info!("  Serial Number:     {}", hopper.get_serial_number().await?);
    info!("  Category:          {:?}", hopper.get_category().await?);
    info!("  Product Code:      {}", hopper.get_product_code().await?);
    info!(
        "  Software Revision: {}",
        hopper.get_software_revision().await?
    );

    Ok(())
}
