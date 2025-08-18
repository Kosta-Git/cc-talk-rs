use std::time::Duration;

use cc_talk_core::cc_talk::{Category, ChecksumType, Device};
use cc_talk_tokio_host::{
    device::{base::DeviceCommon, payout::PayoutDevice},
    transport::{retry::RetryConfig, tokio_transport::CcTalkTokioTransport},
};
use tokio::sync::mpsc;
use tracing::{error, info};

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

    info!("💰 ccTalk payout example.");

    let (tx, rx) = mpsc::channel(32);

    // Make sure you have socat running:
    let transport = CcTalkTokioTransport::new(
        rx,
        "/tmp/cctalk.sock".to_string(),
        Duration::from_millis(100),
        Duration::from_millis(100),
        RetryConfig::default(),
        true,
    );
    tokio::spawn(async move {
        if let Err(e) = transport.run().await {
            tracing::error!("☠️ Error running transport: {}", e);
        }
    });
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

    let product_code = hopper.get_product_code().await.unwrap();

    info!("Product Code: {}", product_code);

    if product_code == "WHM 100.C" {
        // At 100% speed the WHM 100.C is like a gun shot :)
        let _ = hopper.whm_100_speed_adjust(true, 0).await; // 0 is 30%, 7 is 100%
    }

    let _ = hopper.enable_hopper().await;
    let _ = hopper.payout_serial_number(3).await;

    let _ = tokio::spawn(async move {
        let mut remaining = u8::MAX;

        while remaining > 0 {
            match hopper.get_payout_status().await {
                Ok(status) => {
                    info!("Hopper Status: {}", status);
                    remaining = status.coins_remaining;
                }
                Err(e) => {
                    error!("Error getting payout status: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
        let _ = hopper.disable_hopper().await;
    })
    .await;

    tokio::time::sleep(Duration::from_secs(2)).await;
}
