use std::time::Duration;

use cc_talk_core::cc_talk::{Category, ChecksumType, CurrencyToken, Device};
use cc_talk_tokio_host::{
    device::{base::DeviceCommon, payout::PayoutDevice},
    transport::tokio_transport::TransportMessage,
};
use clap::{Subcommand, ValueEnum};
use tokio::sync::mpsc::Sender;
use tracing::{error, info};

#[derive(Subcommand, Debug)]
pub enum HopperCommands {
    /// Poll the hopper to check if it's online
    Poll {
        #[arg(short, long, default_value_t = 1)]
        repeat: u8,

        #[arg(short, long, default_value_t = false, action = clap::ArgAction::SetTrue)]
        infinite: bool,
    },

    /// Dispense coins from the hopper
    Dispense {
        /// Amount of coins to dispense
        amount: u8,

        /// Repeat dispensing multiple times
        #[arg(short, long, default_value_t = 1)]
        repeat: u8,

        /// Which payout mechanism to use
        #[arg(short = 't', long, default_value = "SerialNumber")]
        payout_type: PayoutType,

        /// Interval between polls in milliseconds
        #[arg(short, long, default_value_t = 1000)]
        poll_interval: u64,
    },

    /// Retrieve hopper information
    Info {},

    /// Adjust the hopper speed
    AdjustSpeed {
        #[arg(short, long, default_value_t = false, action = clap::ArgAction::SetTrue)]
        temporary: bool,

        /// Speed value to be set
        ///
        /// On the WHM 100.C hopper, valid values are 0 (30%) to 7 (100%)
        speed: u8,
    },
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum PayoutType {
    /// Simple payout mechanism not supported by most hoppers
    Simple,
    /// Use hopper serial number for payout
    SerialNumber,
    /// Use the encryption mechanism for payout but with null key
    NoEncryption,
}

pub async fn handler(transport: Sender<TransportMessage>, address: u8, action: &HopperCommands) {
    let hopper = PayoutDevice::new(
        Device::new(address, Category::Payout, ChecksumType::Crc8),
        transport,
    );

    match action {
        HopperCommands::Poll { repeat, infinite } => {
            poll(hopper, *repeat, *infinite).await;
        }
        HopperCommands::Dispense {
            amount,
            repeat,
            payout_type,
            poll_interval,
        } => {
            dispense_coins(hopper, *amount, *repeat, *payout_type, *poll_interval).await;
        }
        HopperCommands::Info {} => info(hopper).await,
        HopperCommands::AdjustSpeed { temporary, speed } => {
            adjust_speed(hopper, *temporary, *speed).await;
        }
    }
}

async fn poll(hopper: PayoutDevice, repeat: u8, infinite: bool) {
    loop {
        for _ in 0..repeat {
            match hopper.simple_poll().await {
                Ok(()) => {
                    info!("simple_poll succeeded");
                }
                Err(e) => {
                    error!("simple_poll failed: {}", e);
                }
            }
        }

        if !infinite {
            break;
        }
    }
}

async fn dispense_coins(
    hopper: PayoutDevice,
    amount: u8,
    repeat: u8,
    payout_type: PayoutType,
    poll_interval: u64,
) {
    if repeat == 0 {
        return;
    }

    for i in 0..repeat {
        if repeat > 1 {
            info!("Dispense iteration {}/{}", i + 1, repeat);
        }

        hopper.enable_hopper().await.unwrap_or_else(|e| {
            error!("Failed to enable hopper: {}", e);
        });

        let result = match payout_type {
            PayoutType::Simple => hopper.payout(amount).await,
            PayoutType::SerialNumber => hopper.payout_serial_number(amount).await,
            PayoutType::NoEncryption => hopper.payout_no_encryption(amount).await,
        };

        match result {
            Ok(Some(r)) => {
                info!("Dispensing {}, response => {}", amount, r);
            }
            Ok(None) => {
                info!("Dispensing {}, no response", amount);
            }
            Err(e) => {
                error!("Failed to dispense coins: {}", e);
                panic!("Aborting dispensing due to error");
            }
        }

        let mut remaining = u8::MAX;
        while remaining > 0 {
            match hopper.get_payout_status().await {
                Ok(status) => {
                    info!("{}", status);
                    remaining = status.coins_remaining;
                }
                Err(e) => {
                    error!("Error getting payout status: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_millis(poll_interval)).await;
        }

        hopper.disable_hopper().await.unwrap_or_else(|e| {
            error!("Failed to disable hopper: {}", e);
        });
    }
}

async fn info(hopper: PayoutDevice) {
    let product_code = hopper
        .get_product_code()
        .await
        .expect("should get product code");
    let manufacturer_id = hopper
        .get_manufacturer_id()
        .await
        .expect("should get manufacturer id");
    let serial_number = hopper
        .get_serial_number()
        .await
        .expect("should get serial number");
    let software_revision = hopper
        .get_software_revision()
        .await
        .expect("should get software revision");
    let coin_type = hopper
        .get_hopper_coin()
        .await
        .unwrap_or(CurrencyToken::Token);
    let supports_speed_adjust = matches!(product_code.as_str(), "WHM 100.C");

    info!("Hopper Information:");
    info!("  Product Code: {}", product_code);
    info!("  Manufacturer ID: {}", manufacturer_id);
    info!("  Serial Number: {}", serial_number);
    info!("  Software Revision: {}", software_revision);
    info!("  Coin Type: {:?}", coin_type);
    info!("  Supports Speed Adjust: {}", supports_speed_adjust);
}

async fn adjust_speed(hopper: PayoutDevice, temporary: bool, speed: u8) {
    match hopper.whm_100_speed_adjust(!temporary, speed).await {
        Ok(()) => {
            info!(
                "Hopper speed adjusted to {} (temporary: {})",
                speed, temporary
            );
        }
        Err(e) => {
            error!("Failed to adjust hopper speed: {}", e);
        }
    }
}
