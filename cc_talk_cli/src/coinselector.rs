use std::time::Duration;

use cc_talk_core::cc_talk::{Category, ChecksumType, CoinEvent, CurrencyToken, Device};
use cc_talk_tokio_host::{
    device::{base::DeviceCommon, coin_validator::CoinValidator},
    transport::tokio_transport::TransportMessage,
};
use clap::Subcommand;
use tokio::sync::mpsc::Sender;
use tracing::{error, info};

#[derive(Subcommand, Debug)]
pub enum CoinSelectorCommands {
    /// Get the currently selected coin types
    Info {},
    Accept {
        /// Number of coins to accept before stopping. 0 means infinite.
        #[arg(short, long, default_value_t = 0)]
        count: u32,
    },
}

pub async fn handler(
    transport: Sender<TransportMessage>,
    address: u8,
    action: &CoinSelectorCommands,
) {
    let selector = CoinValidator::new(
        Device::new(address, Category::CoinAcceptor, ChecksumType::Crc8),
        transport,
    );

    match action {
        CoinSelectorCommands::Info {} => {
            info_selector(selector).await;
        }
        CoinSelectorCommands::Accept { count } => {
            accept_coins(selector, *count, *count == 0).await;
        }
    }
}

async fn accept_coins(mut selector: CoinValidator, mut count: u32, infinite: bool) {
    selector
        .disable_master_inhibit()
        .await
        .expect("should disable master inhibit");

    selector
        .set_all_coin_inhibits(false)
        .await
        .expect("should enable all coins");

    let polling_priority = selector
        .get_polling_priority()
        .await
        .expect("should get polling priority")
        .as_duration()
        .unwrap_or(Duration::from_millis(200));

    let mut event_counter: u8 = 0;
    while count > 0 || infinite {
        match selector.poll().await {
            Ok(poll) => {
                if event_counter == poll.event_counter {
                    continue;
                }

                event_counter = poll.event_counter;
                info!("event counter: {}", event_counter);

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
                                );
                            }
                            CoinEvent::Reset => {
                                info!("coin validator reset");
                            }
                        }
                    }
                }

                count = count.saturating_sub(1);
            }
            Err(e) => {
                info!("Error polling for event: {}", e);
            }
        }
        tokio::time::sleep(polling_priority).await;
    }
}

#[allow(clippy::explicit_iter_loop)]
async fn info_selector(selector: CoinValidator) {
    let product_code = selector
        .get_product_code()
        .await
        .expect("should get product code");
    let manufacturer_id = selector
        .get_manufacturer_id()
        .await
        .expect("should get manufacturer id");
    let serial_number = selector
        .get_serial_number()
        .await
        .expect("should get serial number");
    let software_revision = selector
        .get_software_revision()
        .await
        .expect("should get software revision");
    let coin_ids = selector
        .request_all_coin_id()
        .await
        .expect("should request all coin IDs")
        .into_iter()
        .filter(|(_, opt)| opt.is_some())
        .collect::<Vec<_>>();
    let mut coin_sorter_paths = vec![];
    // TODO: Improve this mess
    for coin in coin_ids.iter() {
        let csp = selector
            .get_coin_sorter_path(coin.0)
            .await
            .expect("should get coin sorter path");
        coin_sorter_paths.push((coin.0, csp));
    }
    let coin_ids = coin_ids
        .into_iter()
        .map(|(id, opt)| {
            opt.map_or_else(
                || format!("{id}: Unknown Coin ID"),
                |coin_id| match coin_id {
                    CurrencyToken::Token => format!("{id}: Token"),
                    CurrencyToken::Currency(value) => {
                        format!("{id}: {} {}", value.monetary_value(), value.country_code())
                    }
                },
            )
        })
        .collect::<Vec<_>>();
    let polling_priority = selector
        .get_polling_priority()
        .await
        .expect("should get polling priority");

    info!("Coin Selector Information:");
    info!("  Product Code: {}", product_code);
    info!("  Manufacturer ID: {}", manufacturer_id);
    info!("  Serial Number: {}", serial_number);
    info!("  Software Revision: {}", software_revision);
    info!("  Coin Ids: {:#?}", coin_ids);
    info!("  Coin Sorter Paths: {:#?}", coin_sorter_paths);
    info!(
        "  Polling Priority: {:?}",
        polling_priority.as_duration().unwrap_or(Duration::ZERO)
    );
}
