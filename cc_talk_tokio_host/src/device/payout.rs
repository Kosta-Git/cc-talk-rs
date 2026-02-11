#![allow(dead_code)]

use cc_talk_core::cc_talk::{
    CurrencyToken, Device, HopperDispenseStatus, HopperFlag, HopperStatus,
};
use cc_talk_host::{command::Command, device::device_commands::*};
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::transport::tokio_transport::TransportMessage;

use super::base::{CommandError, DeviceCommon, DeviceResult};

pub struct PayoutDevice {
    pub device: Device,
    pub sender: mpsc::Sender<TransportMessage>,
}

impl std::fmt::Debug for PayoutDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PayoutDevice")
            .field("device", &self.device)
            .finish_non_exhaustive()
    }
}

impl PayoutDevice {
    pub fn new(device: Device, sender: mpsc::Sender<TransportMessage>) -> Self {
        debug!(
            address = device.address(),
            category = ?device.category(),
            "creating payout device"
        );
        PayoutDevice { device, sender }
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn get_payout_status(&self) -> DeviceResult<HopperDispenseStatus> {
        trace!("requesting hopper dispense status");
        let response_packet = self.send_command(RequestHopperStatusCommand).await?;
        let status = RequestHopperStatusCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        debug!(status = ?status, "hopper dispense status received");
        Ok(status)
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn self_test(&self) -> DeviceResult<Vec<HopperFlag>> {
        info!("running hopper self-test");
        let response_packet = self.send_command(TestHopperCommand).await?;
        let flags = TestHopperCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|flags| flags.to_vec())?;
        if flags.is_empty() {
            info!("hopper self-test passed with no flags");
        } else {
            warn!(flags = ?flags, "hopper self-test returned flags");
        }
        Ok(flags)
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn get_sensor_status(&self) -> DeviceResult<(u8, HopperStatus)> {
        trace!("requesting sensor status");
        let response_packet = self.send_command(RequestpayoutHighLowStatusCommand).await?;
        let result = RequestpayoutHighLowStatusCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        debug!(level = result.0, status = ?result.1, "sensor status received");
        Ok(result)
    }

    #[instrument(skip(self), fields(enabled), level = "debug")]
    pub async fn change_hopper_status(&self, enabled: bool) -> DeviceResult<()> {
        info!(enabled, "changing hopper status");
        let command = EnableHopperCommand::new(enabled);
        let response_packet = self.send_command(command).await?;
        EnableHopperCommand::new(enabled)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|_| ())?;
        info!(enabled, "hopper status changed");
        Ok(())
    }

    pub async fn enable_hopper(&self) -> DeviceResult<()> {
        debug!("enabling hopper");
        self.change_hopper_status(true).await
    }

    pub async fn disable_hopper(&self) -> DeviceResult<()> {
        debug!("disabling hopper");
        self.change_hopper_status(false).await
    }

    #[instrument(skip(self), fields(coins), level = "info")]
    pub async fn payout(&self, coins: u8) -> DeviceResult<Option<u8>> {
        info!(coins, "initiating payout");
        let command = DispenseHopperCoinsCommand::new(coins);
        let response_packet = self.send_command(command).await?;
        let result = DispenseHopperCoinsCommand::new(coins)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        match result {
            Some(remaining) => info!(coins, remaining, "payout initiated with remaining coins"),
            None => info!(coins, "payout initiated"),
        }
        Ok(result)
    }

    #[instrument(skip(self), fields(coins), level = "info")]
    pub async fn payout_serial_number(&self, coins: u8) -> DeviceResult<Option<u8>> {
        debug!(coins, "initiating payout with serial number authentication");
        let serial_number = self.get_serial_number().await?;
        trace!(
            serial_fix = serial_number.fix(),
            serial_minor = serial_number.minor(),
            serial_major = serial_number.major(),
            "using serial number for authentication"
        );
        let command = DispenseHopperCoinsCommand::new_with_data(
            coins,
            &[
                serial_number.fix(),
                serial_number.minor(),
                serial_number.major(),
            ],
        );
        let response_packet = self.send_command(command).await?;
        let result = DispenseHopperCoinsCommand::new(coins)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        info!(coins, result = ?result, "payout with serial number completed");
        Ok(result)
    }

    #[instrument(skip(self), fields(coins), level = "info")]
    pub async fn payout_no_encryption(&self, coins: u8) -> DeviceResult<Option<u8>> {
        debug!(coins, "initiating payout without encryption");
        trace!("pumping RNG");
        self.send_command(PumpRngCommand::new([0, 0, 0, 0, 0, 0, 0, 0]))
            .await?;
        trace!("requesting cipher key");
        self.send_command(RequestCipherKeyCommand).await?;
        let command =
            DispenseHopperCoinsCommand::new_with_data(coins, &[0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let response_packet = self.send_command(command).await?;
        let result = DispenseHopperCoinsCommand::new(coins)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        info!(coins, result = ?result, "payout without encryption completed");
        Ok(result)
    }

    #[instrument(skip(self), fields(hopper_number, count), level = "info")]
    pub async fn purge(&self, hopper_number: u8, count: u8) -> DeviceResult<()> {
        warn!(hopper_number, count, "purging hopper");
        let command = PurgeHopperCommand::new(hopper_number, count);
        let response_packet = self.send_command(command).await?;
        PurgeHopperCommand::new(hopper_number, count)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        info!(hopper_number, count, "hopper purge completed");
        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn get_dispense_count(&self) -> DeviceResult<u32> {
        trace!("requesting dispense count");
        let response_packet = self.send_command(RequestHopperDispenseCountCommand).await?;
        let count = RequestHopperDispenseCountCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        debug!(count, "dispense count received");
        Ok(count)
    }

    #[instrument(skip(self), level = "warn")]
    pub async fn emergency_stop(&self) -> DeviceResult<u8> {
        error!("emergency stop triggered");
        let response_packet = self.send_command(EmergencyStopCommand).await?;
        let result = EmergencyStopCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        warn!(result, "emergency stop completed");
        Ok(result)
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn get_hopper_coin(&self) -> DeviceResult<CurrencyToken> {
        trace!("requesting hopper coin info");
        let response_packet = self.send_command(RequestHopperCoinCommand).await?;
        let token = RequestHopperCoinCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        debug!(token = ?token, "hopper coin info received");
        Ok(token)
    }

    #[instrument(skip(self), fields(coin_type), level = "debug")]
    pub async fn get_hopper_coin_value(&self, coin_type: u8) -> DeviceResult<(CurrencyToken, u16)> {
        trace!(coin_type, "requesting hopper coin value");
        let response_packet = self
            .send_command(RequestHopperCoinValueCommand::new(coin_type))
            .await?;
        let result = RequestHopperCoinValueCommand::new(coin_type)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        debug!(coin_type, token = ?result.0, value = result.1, "hopper coin value received");
        Ok(result)
    }

    #[instrument(skip(self), fields(permanent, speed), level = "debug")]
    pub async fn whm_100_speed_adjust(&self, permanent: bool, speed: u8) -> DeviceResult<()> {
        info!(permanent, speed, "adjusting WHM-100 motor speed");
        let permanent_flag: u8 = if permanent { 2 } else { 1 };
        let command = OperateBiDirectionalMotorsCommand::new(permanent_flag, speed, 0);
        let response_packet = self.send_command(command).await?;
        OperateBiDirectionalMotorsCommand::new(0, 0, 0)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)?;
        info!(permanent, speed, "motor speed adjusted");
        Ok(())
    }
}

impl Clone for PayoutDevice {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            sender: self.sender.clone(),
        }
    }
}

impl DeviceCommon for PayoutDevice {
    fn get_device(&self) -> &Device {
        &self.device
    }

    fn get_sender(&self) -> &mpsc::Sender<TransportMessage> {
        &self.sender
    }
}
