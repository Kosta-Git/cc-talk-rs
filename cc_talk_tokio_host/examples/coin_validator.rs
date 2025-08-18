use std::time::Duration;

use cc_talk_core::cc_talk::{Address, Category, ChecksumType, CoinEvent, CurrencyToken, Device};
use cc_talk_tokio_host::{
    device::{base::DeviceCommon, coin_validator::CoinValidator},
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

    info!("💰 ccTalk coin validator example.");

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

    // If you don't know the address of your coin validator, you can use the default address for
    // the CoinAcceptor category.
    let coin_validator_address = match Category::CoinAcceptor.default_address() {
        Address::Single(addr) => addr,
        Address::SingleAndRange(addr, _) => addr,
    };
    let mut coin_validator = CoinValidator::new(
        Device::new(
            coin_validator_address,
            Category::CoinAcceptor,
            ChecksumType::Crc8,
        ),
        tx,
    );

    info!("📡 Trying to reach coin validator...");
    match coin_validator.simple_poll().await {
        Ok(_) => info!("✅ Coin validator is online!"),
        Err(error) => {
            error!("☠️ Error reaching coin validator: {}", error);
            return;
        }
    }

    let manufacturer = coin_validator.get_manufacturer_id().await.unwrap();
    let serial_number = coin_validator.get_serial_number().await.unwrap();
    let category = coin_validator.get_category().await.unwrap();
    let product_code = coin_validator.get_product_code().await.unwrap();
    let software_revision = coin_validator.get_software_revision().await.unwrap();

    info!(
        "\n\tManufacturer ID: {}\n\tSerial number: {}\n\tCategory: {:?}\n\tProduct code: {}\n\tSoftware revision: {}",
        manufacturer, serial_number, category, product_code, software_revision
    );

    let master_inhibit_status = coin_validator.get_master_inhibit_status().await.unwrap();
    if master_inhibit_status {
        info!("Master inhibit is ON. Disabling it...");
        coin_validator.disable_master_inhibit().await.unwrap();
    } else {
        info!("Master inhibit is OFF.");
    }

    coin_validator
        .request_all_coin_id()
        .await
        .unwrap()
        .iter()
        .filter(|entry| entry.1.is_some())
        .to_owned()
        .for_each(|entry| {
            let coin = entry.1.clone().expect("");
            match coin {
                CurrencyToken::Token => {
                    info!("coin ID {}: Token", entry.0);
                }
                CurrencyToken::Currency(currency_value) => {
                    info!(
                        "coin ID {}: {}@{}",
                        entry.0,
                        currency_value.country_code(),
                        currency_value.monetary_value()
                    );
                }
            }
        });

    // You could enable/disable some coins based on your needs, by using the coin IDs.
    let coin_inhibits = coin_validator.get_coin_inhibits().await.unwrap();
    if coin_inhibits.iter().any(|inhibit| *inhibit) {
        info!("Some coin inhibits are ON. Disabling them...");
        coin_validator.set_all_coin_inhibits(false).await.unwrap();
    }

    let polling_priority = coin_validator.get_polling_priority().await.unwrap();
    let delay = polling_priority.as_duration().unwrap();
    info!(
        "polling priority: {:?} (delay: {:?})",
        polling_priority, delay
    );

    let mut event_counter: u8 = 0;
    loop {
        match coin_validator.poll().await {
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
                    if poll.lost_events > 0 {
                        error!("lost events: {}", poll.lost_events);
                    }
                    for event in poll.events {
                        match event {
                            CoinEvent::Error(coin_acceptor_error) => {
                                error!(
                                    "error {}: {}",
                                    coin_acceptor_error as u8,
                                    coin_acceptor_error.description()
                                );
                            }
                            CoinEvent::Credit(coin_credit) => {
                                info!(
                                    "coin {} in sorter {:?} ",
                                    coin_credit.credit, coin_credit.sorter_path
                                )
                            }
                            CoinEvent::Reset => {
                                info!("coin validator reset");
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
