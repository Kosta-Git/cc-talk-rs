#![allow(dead_code)]

use cc_talk_core::cc_talk::{
    BitMask, CoinAcceptorPollResult, Command, CurrencyToken, Device, ParseResponseError, SorterPath,
};
use cc_talk_host::device::device_commands::*;
use tokio::sync::mpsc;

use crate::transport::tokio_transport::TransportMessage;

use super::base::{CommandError, DeviceCommon, DeviceResult};

pub struct CoinValidator {
    pub device: Device,
    pub sender: mpsc::Sender<TransportMessage>,
    pub event_counter: u8,
}

impl CoinValidator {
    pub fn new(device: Device, sender: mpsc::Sender<TransportMessage>) -> Self {
        CoinValidator {
            device,
            sender,
            event_counter: 0,
        }
    }

    pub async fn set_master_inhibit(&self, inhibit: bool) -> DeviceResult<()> {
        // TODO: This is a bit goofy, the api for this should be simplified.
        let mask_value = !inhibit;
        let mut bitmask = BitMask::<1>::new(1).map_err(|_| CommandError::BufferOverflow)?;
        bitmask
            .set_bit(0, mask_value)
            .map_err(|_| CommandError::BufferOverflow)?;
        let command = ModifyMasterInhibitStatusCommand::<1>::build(bitmask)
            .map_err(|_| CommandError::BufferOverflow)?;
        let response_packet = self.send_command(command).await?;
        let bitmask = BitMask::<1>::new(1).map_err(|_| CommandError::BufferOverflow)?;
        ModifyMasterInhibitStatusCommand::<1>::build(bitmask)
            .map_err(|_| CommandError::BufferOverflow)?
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn enable_master_inhibit(&self) -> DeviceResult<()> {
        self.set_master_inhibit(true).await
    }

    pub async fn disable_master_inhibit(&self) -> DeviceResult<()> {
        self.set_master_inhibit(false).await
    }

    /// Returns the master inhibit status of the coin validator.
    /// True means that the master inhibit is enabled, false means it is disabled.
    pub async fn get_master_inhibit_status(&self) -> DeviceResult<bool> {
        let response_packet = self
            .send_command(RequestMasterInhibitStatusCommand::<1>)
            .await?;
        RequestMasterInhibitStatusCommand::<1>
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|bytes| bytes[0] == 0)
    }

    pub async fn is_master_inhibit_enabled(&self) -> DeviceResult<bool> {
        self.get_master_inhibit_status().await
    }

    pub async fn is_master_inhibit_disabled(&self) -> DeviceResult<bool> {
        let status = self.get_master_inhibit_status().await?;
        Ok(!status)
    }

    pub async fn set_default_sorter_path(&self, new_default_path: u8) -> DeviceResult<()> {
        let command = ModifyDefaultSorterPathCommand::new(new_default_path);
        let response_packet = self.send_command(command).await?;
        ModifyDefaultSorterPathCommand::new(new_default_path)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn get_default_sorter_path(&self) -> DeviceResult<SorterPath> {
        let response_packet = self.send_command(RequestDefaultSorterPathCommand).await?;
        RequestDefaultSorterPathCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    /// Sets the sorter override status for each of the 8 sorter paths.
    /// The `overrides` array should contain 8 boolean values, where each value corresponds
    /// True: sorter override to a different or default path.
    /// False: no action
    pub async fn modify_sorter_override_status(&self, overrides: [bool; 8]) -> DeviceResult<()> {
        let mut bitmask = BitMask::<1>::new(8).map_err(|_| CommandError::BufferOverflow)?;
        for (i, should_override) in overrides.iter().enumerate() {
            bitmask
                // Invert value since 0 is override and 1 is no override
                .set_bit(i, !*should_override)
                .map_err(|_| CommandError::BufferOverflow)?;
        }

        let command = ModifySorterOverrideStatusCommand::build(bitmask)
            .map_err(|_| CommandError::BufferOverflow)?;

        let response_packet = self.send_command(command).await?;
        let bitmask = BitMask::<1>::new(8).map_err(|_| CommandError::BufferOverflow)?;
        ModifySorterOverrideStatusCommand::build(bitmask)
            .map_err(|_| CommandError::BufferOverflow)?
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    /// Requests the sorter override status for each of the 8 sorter paths.
    /// The returned BitMask will have 8 bits, where each bit corresponds to a sorter
    /// 1: sorter override to a different or default path.
    /// 0: no override
    pub async fn request_sorter_override_status(&self) -> DeviceResult<BitMask<1>> {
        let response_packet = self
            .send_command(RequestSorterOverrideStatusCommand)
            .await?;
        RequestSorterOverrideStatusCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|mut mask| {
                mask.flip();
                mask
            })
    }

    pub async fn set_coin_sorter_path(&self, coin_position: u8, path: u8) -> DeviceResult<()> {
        let command = ModifySorterPathCommand::new(coin_position, path);
        let response_packet = self.send_command(command).await?;
        ModifySorterPathCommand::new(coin_position, path)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn get_coin_sorter_path(&self, coin_position: u8) -> DeviceResult<SorterPath> {
        let response_packet = self
            .send_command(RequestSorterPathCommand::new(coin_position))
            .await?;
        RequestSorterPathCommand::new(coin_position)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn poll(&mut self) -> DeviceResult<CoinAcceptorPollResult> {
        let response_packet = self
            .send_command(ReadBufferedCreditOrErrorCodeCommand::default())
            .await?;
        ReadBufferedCreditOrErrorCodeCommand::new(self.event_counter)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .inspect(|result| {
                self.event_counter = result.event_counter;
            })
    }

    pub async fn request_coin_id(&self, coin_position: u8) -> DeviceResult<CurrencyToken> {
        let response_packet = self
            .send_command(RequestCoinIdCommand::new(coin_position))
            .await?;
        RequestCoinIdCommand::new(coin_position)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn request_coin_id_range(
        &self,
        number_of_coins: u8,
    ) -> DeviceResult<Vec<(u8, Option<CurrencyToken>)>> {
        let mut coins = std::vec::Vec::with_capacity(number_of_coins as usize);
        for i in 0..number_of_coins {
            if let Ok(coin) = self.request_coin_id(i).await {
                coins.push((i, Some(coin)));
            } else {
                coins.push((i, None));
            }
        }
        Ok(coins)
    }

    pub async fn request_all_coin_id(&self) -> DeviceResult<Vec<(u8, Option<CurrencyToken>)>> {
        self.request_coin_id_range(16).await
    }

    /// Sets the inhibit status for each of the 16 coin positions.
    /// True: coin is DISABLED
    /// False: coin is ENABLED
    pub async fn set_coin_inhibits(&self, inhibits: [bool; 16]) -> DeviceResult<()> {
        let mut bitmask = BitMask::<2>::new(16).map_err(|_| CommandError::BufferOverflow)?;
        for (i, disable) in inhibits.iter().enumerate() {
            bitmask
                // Invert value since 0 is disabled and 1 is enabled
                .set_bit(i, !*disable)
                .map_err(|_| CommandError::BufferOverflow)?;
        }
        let command = ModifyInhibitStatusCommand::<2>::build(bitmask)
            .map_err(|_| CommandError::BufferOverflow)?;
        let response_packet = self.send_command(command).await?;
        let bitmask = BitMask::<2>::new(16).map_err(|_| CommandError::BufferOverflow)?;
        ModifyInhibitStatusCommand::<2>::build(bitmask)
            .map_err(|_| CommandError::BufferOverflow)?
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn set_all_coin_inhibits(&self, inhibit: bool) -> DeviceResult<()> {
        let inhibits = [inhibit; 16];
        self.set_coin_inhibits(inhibits).await
    }

    /// Requests the inhibit status for each of the 16 coin positions.
    /// True: coin is disabled
    /// False: coin is enabled
    pub async fn get_coin_inhibits(&self) -> DeviceResult<Vec<bool>> {
        let response_packet = self.send_command(RequestInhibitStatusCommand::<2>).await?;
        RequestInhibitStatusCommand::<2>
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|mask| {
                let mut vec = std::vec::Vec::with_capacity(16);
                for byte in mask.iter() {
                    for i in 0..8 {
                        vec.push(byte & (1 << i) == 0);
                    }
                }
                vec
            })
    }

    pub async fn get_polling_priority(&self) -> DeviceResult<PollingPriority> {
        let response_packet = self.send_command(RequestPollingPriorityCommand).await?;
        RequestPollingPriorityCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }
}

impl DeviceCommon for CoinValidator {
    fn get_device(&self) -> &Device {
        &self.device
    }

    fn get_sender(&self) -> &mpsc::Sender<TransportMessage> {
        &self.sender
    }
}
