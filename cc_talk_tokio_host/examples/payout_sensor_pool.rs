//! Payout Sensor Pool example - monitors hopper inventory levels.
//!
//! This example demonstrates how to use the PayoutSensorPool to continuously
//! monitor hopper sensor levels and react to inventory changes.
//!
//! Usage: cargo run --example payout_sensor_pool [socket_path]
//!
//! Arguments:
//!   socket_path  Path to ccTalk socket (default: /tmp/cctalk.sock)
//!
//! The example assumes three hoppers at addresses 3, 4, and 5.

use std::{env, time::Duration};

use cc_talk_core::cc_talk::{Category, ChecksumType, Device};
use cc_talk_tokio_host::{
    device::{
        payout::PayoutDevice,
        payout_sensor_pool::{PayoutSensorPool, PollingStatus, SensorEvent},
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
    eprintln!("Usage: payout_sensor_pool [socket_path]");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  socket_path  Path to ccTalk socket (default: /tmp/cctalk.sock)");
    eprintln!();
    eprintln!("Assumes hoppers at addresses 3, 4, 5");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return Ok(());
    }

    let socket_path = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "/tmp/cctalk.sock".to_string());

    info!("Payout Sensor Pool Example");
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

    // Build the sensor pool
    info!("Building sensor pool...");
    let sensor_pool = PayoutSensorPool::builder()
        .add_hopper(hopper1)
        .add_hopper(hopper2)
        .add_hopper(hopper3)
        .polling_interval(Duration::from_millis(2000))
        .build();

    info!(
        "Sensor pool created with {} hoppers: {:?}",
        sensor_pool.hopper_count(),
        sensor_pool.hopper_addresses()
    );

    // Start background sensor polling
    info!("Starting background sensor polling...");
    let (_polling_tx, polling_rx) = tokio::sync::watch::channel(PollingStatus::Running);
    let mut guard = sensor_pool
        .try_start_polling(polling_rx)
        .expect("should start polling");

    let _ = sensor_pool.mark_empty(3);
    let _ = sensor_pool.mark_empty(4);
    let _ = sensor_pool.mark_empty(5);

    // Monitor sensor events
    info!("Monitoring sensor events (press Ctrl+C to stop)...");
    while let Some(event) = guard.recv().await {
        match event {
            SensorEvent::InventoryUpdate {
                inventories,
                errors,
            } => {
                for reading in &inventories {
                    info!(
                        "  Hopper {} sensor: {} (status: {:?})",
                        reading.address, reading.level, reading.status
                    );
                }
                for err in &errors {
                    warn!("  Hopper {} error: {}", err.address, err.error);
                }
            }
            SensorEvent::LevelChanged {
                address,
                previous,
                current,
            } => {
                info!(
                    "Level changed: hopper {} {} -> {}",
                    address, previous, current
                );
            }
            SensorEvent::MarkedEmpty { address } => {
                warn!("Hopper {} marked empty", address);
            }
            SensorEvent::MarkedNonEmpty { address, reason } => {
                info!("Hopper {} marked non-empty (reason: {:?})", address, reason);
            }
        }
    }

    // Guard is dropped here, stopping the background polling task.
    info!("Done");
    Ok(())
}
