use std::time::Duration;

use cc_talk_core::cc_talk::{
    Address, BillEvent, BillRouteCode, Category, ChecksumType, CurrencyToken, Device,
};
use cc_talk_tokio_host::{
    device::{base::DeviceCommon, bill_validator::BillValidator},
    transport::{retry::RetryConfig, tokio_transport::CcTalkTokioTransport},
};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

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

    info!("💰 ccTalk bill validator example.");

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

    // If you don't know the address of your bill validator, you can use the default address for
    // the BillValidator category.
    let bill_validator_address = match Category::BillValidator.default_address() {
        Address::Single(addr) => addr,
        Address::SingleAndRange(addr, _) => addr,
    };
    let bill_validator = BillValidator::new(
        Device::new(
            bill_validator_address,
            Category::BillValidator,
            ChecksumType::Crc16,
        ),
        tx,
    );

    info!("📡 Trying to reach bill validator...");
    match bill_validator.simple_poll().await {
        Ok(_) => info!("✅ Coin validator is online!"),
        Err(error) => {
            error!("☠️ Error reaching coin validator: {}", error);
            return;
        }
    }

    let manufacturer = bill_validator.get_manufacturer_id().await.unwrap();
    let serial_number = bill_validator.get_serial_number().await.unwrap();
    let category = bill_validator.get_category().await.unwrap();
    let product_code = bill_validator.get_product_code().await.unwrap();
    let software_revision = bill_validator.get_software_revision().await.unwrap();

    info!(
        "\n\tManufacturer ID: {}\n\tSerial number: {}\n\tCategory: {:?}\n\tProduct code: {}\n\tSoftware revision: {}",
        manufacturer, serial_number, category, product_code, software_revision
    );

    let master_inhibit_status = bill_validator.get_master_inhibit_status().await.unwrap();
    if master_inhibit_status {
        info!("Master inhibit is ON. Disabling it...");
        bill_validator.disable_master_inhibit().await.unwrap();
    } else {
        info!("Master inhibit is OFF.");
    }

    bill_validator
        .request_all_bill_id()
        .await
        .unwrap()
        .iter()
        .filter(|entry| entry.1.is_some())
        .to_owned()
        .for_each(|entry| {
            let bill = entry.1.clone().expect("");
            match bill {
                CurrencyToken::Token => {
                    info!("bill ID {}: Token", entry.0);
                }
                CurrencyToken::Currency(currency_value) => {
                    info!(
                        "bill ID {}: {}@{}",
                        entry.0,
                        currency_value.country_code(),
                        currency_value.monetary_value()
                    );
                }
            }
        });

    // You could enable/disable some coins based on your needs, by using the coin IDs.
    let inhibits = bill_validator.get_bill_inhibits().await.unwrap();
    if inhibits.iter().any(|inhibit| *inhibit) {
        info!("Some bill inhibits are ON. Disabling them...");
        bill_validator.set_all_bill_inhibits(false).await.unwrap();
    }

    let polling_priority = bill_validator.get_polling_priority().await.unwrap();
    let delay = polling_priority.as_duration().unwrap();
    info!(
        "polling priority: {:?} (delay: {:?})",
        polling_priority, delay
    );

    let mut event_counter: u8 = 0;
    loop {
        match bill_validator.poll().await {
            Ok(poll) => {
                if event_counter != poll.event_counter {
                    event_counter = poll.event_counter;
                    info!("event counter: {}", event_counter);
                } else {
                    continue;
                }

                if !poll.events.is_empty() {
                    info!(
                        "===================== [counter {}] =====================",
                        poll.event_counter
                    );
                    for event in poll.events {
                        match event {
                            BillEvent::Credit(credit) => {
                                info!("bill in stacker: {}", credit);
                            }
                            BillEvent::PendingCredit(credit) => {
                                info!("bill in escrow: {}", credit);
                                info!("sending route to stacker command");
                                bill_validator
                                    .route_bill(BillRouteCode::Stack)
                                    .await
                                    .unwrap();
                            }
                            BillEvent::Reject(bill_event_reason) => {
                                warn!("bill rejected: {}", bill_event_reason);
                            }
                            BillEvent::FraudAttempt(bill_event_reason) => {
                                warn!("fraud attempt detected: {}", bill_event_reason);
                            }
                            BillEvent::FatalError(bill_event_reason) => {
                                warn!("fatal error: {}", bill_event_reason);
                            }
                            BillEvent::Status(bill_event_reason) => {
                                warn!("status: {}", bill_event_reason);
                            }
                        }
                    }
                }
            }
            Err(error) => {
                error!("command error: {}", error);
            }
        }
        tokio::time::sleep(delay).await;
    }
}
