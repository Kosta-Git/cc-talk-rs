//! Bill validator example - polls for bill insertions and auto-stacks them.
//!
//! Usage: cargo run --example bill_validator [socket_path]
//!
//! Arguments:
//!   socket_path  Path to ccTalk socket (default: /tmp/cctalk.sock)

use std::{env, time::Duration};

use cc_talk_core::cc_talk::{
    Address, BillEvent, BillRouteCode, Category, ChecksumType, CurrencyToken, Device,
};
use cc_talk_tokio_host::{
    device::{base::DeviceCommon, bill_validator::BillValidator},
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

    info!("Bill Validator Example");
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

    // Create bill validator
    let address = match Category::BillValidator.default_address() {
        Address::Single(addr) | Address::SingleAndRange(addr, _) => addr,
    };
    let validator = BillValidator::new(
        Device::new(address, Category::BillValidator, ChecksumType::Crc16),
        tx,
    );

    // Check connectivity
    info!("Connecting to bill validator...");
    validator.simple_poll().await?;
    info!("Connected");

    // Display device info
    let manufacturer = validator.get_manufacturer_id().await?;
    let serial = validator.get_serial_number().await?;
    let product = validator.get_product_code().await?;
    info!("Device: {} {} (S/N: {})", manufacturer, product, serial);

    // Build value lookup table and display configured bills
    info!("Configured bills:");
    let bill_values: Vec<(u8, u32)> = validator
        .request_all_bill_id()
        .await?
        .iter()
        .filter_map(|(pos, token)| {
            token.as_ref().map(|t| match t {
                CurrencyToken::Token => {
                    info!("  [{}] Token", pos);
                    (*pos, 0)
                }
                CurrencyToken::Currency(v) => {
                    info!("  [{}] {}{:.2}", pos, v.country_code(), v.monetary_value());
                    (*pos, v.smallest_unit_value())
                }
            })
        })
        .collect();

    // Enable acceptance
    validator.disable_master_inhibit().await?;
    validator.set_all_bill_inhibits(false).await?;
    info!("Bill acceptance enabled");

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

                for event in poll.events {
                    match event {
                        BillEvent::Credit(bill_type) => {
                            let value = bill_values
                                .iter()
                                .find(|(p, _)| *p == bill_type)
                                .map_or(0, |(_, v)| *v);
                            info!(
                                "Bill stacked: position {} (value: {} cents)",
                                bill_type, value
                            );
                        }
                        BillEvent::PendingCredit(bill_type) => {
                            let value = bill_values
                                .iter()
                                .find(|(p, _)| *p == bill_type)
                                .map_or(0, |(_, v)| *v);
                            info!(
                                "Bill in escrow: position {} (value: {} cents) -> stacking",
                                bill_type, value
                            );
                            if let Err(e) = validator.route_bill(BillRouteCode::Stack).await {
                                error!("Failed to route bill: {}", e);
                            }
                        }
                        BillEvent::Reject(reason) => {
                            warn!("Bill rejected: {}", reason);
                        }
                        BillEvent::FraudAttempt(reason) => {
                            warn!("Fraud attempt: {}", reason);
                        }
                        BillEvent::FatalError(reason) => {
                            error!("Fatal error: {}", reason);
                        }
                        BillEvent::Status(reason) => {
                            info!("Status: {}", reason);
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
