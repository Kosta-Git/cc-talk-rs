use std::time::Duration;

use cc_talk_core::cc_talk::{Category, ChecksumType, Device, Manufacturer};
use cc_talk_tokio_host::{
    device::{base::DeviceCommon, payout::PayoutDevice},
    transport::{retry::RetryConfig, tokio_transport::CcTalkTokioTransport, *},
};
use tokio::sync::mpsc;
use tracing::{Level, error, info};

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .with_target(false)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    info!("💰 ccTalk device info example.");

    let (tx, rx) = mpsc::channel(32);

    // Make sure you have socat running:
    let transport = CcTalkTokioTransport::new(
        rx,
        "/tmp/cctalk.sock".to_string(),
        Duration::from_millis(100),
        Duration::from_millis(100),
        RetryConfig::default(),
    );
    tokio::spawn(async move {
        if let Err(e) = transport.run().await {
            tracing::error!("☠️ Error running transport: {}", e);
        }
    });
    // TODO: find a way to signal the transport is ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    let hopper = PayoutDevice::new(Device::new(3, Category::Payout, ChecksumType::Crc8), tx);

    info!("📡 Trying to reach hopper...");
    match hopper.simple_poll().await {
        Ok(_) => info!("✅ Hopper is online!"),
        Err(error) => {
            error!("☠️ Error reaching hopper: {}", error);
            return;
        }
    }

    let manufacturer = hopper
        .get_manufacturer_id()
        .await
        .unwrap_or(Manufacturer::INOTEK);
    let serial_number = hopper.get_serial_number().await.unwrap();
    let category = hopper.get_category().await.unwrap();
    let product_code = hopper.get_product_code().await.unwrap();
    let software_revision = hopper.get_software_revision().await.unwrap();

    info!("Manufacturer: {}", manufacturer);
    info!("Serial Number: {}", serial_number);
    info!("Category: {:?}", category);
    info!("Product Code: {}", product_code);
    info!("Software Revision: {}", software_revision);
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
