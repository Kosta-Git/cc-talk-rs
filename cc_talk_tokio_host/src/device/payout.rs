#![allow(dead_code)]

use cc_talk_core::cc_talk::{Device, HopperDispenseStatus, HopperFlag, HopperStatus};
use cc_talk_host::{command::Command, device::device_commands::*};
use tokio::sync::mpsc;

use crate::transport::tokio_transport::TransportMessage;

use super::base::{CommandError, DeviceCommon, DeviceResult};

pub struct PayoutDevice {
    pub device: Device,
    pub sender: mpsc::Sender<TransportMessage>,
}

impl PayoutDevice {
    pub fn new(device: Device, sender: mpsc::Sender<TransportMessage>) -> Self {
        PayoutDevice { device, sender }
    }

    pub async fn get_payout_status(&self) -> DeviceResult<HopperDispenseStatus> {
        let response_packet = self.send_command(RequestHopperStatusCommand).await?;
        RequestHopperStatusCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn self_test(&self) -> DeviceResult<Vec<HopperFlag>> {
        let response_packet = self.send_command(TestHopperCommand).await?;
        TestHopperCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|flags| flags.to_vec())
    }

    pub async fn get_sensor_status(&self) -> DeviceResult<(u8, HopperStatus)> {
        let response_packet = self.send_command(RequestpayoutHighLowStatusCommand).await?;
        RequestpayoutHighLowStatusCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn change_hopper_status(&self, enabled: bool) -> DeviceResult<()> {
        let command = EnableHopperCommand::new(enabled);
        let response_packet = self.send_command(command).await?;
        EnableHopperCommand::new(enabled)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|_| ())
    }

    pub async fn enable_hopper(&self) -> DeviceResult<()> {
        self.change_hopper_status(true).await
    }

    pub async fn disable_hopper(&self) -> DeviceResult<()> {
        self.change_hopper_status(false).await
    }

    pub async fn payout(&self, coins: u8) -> DeviceResult<Option<u8>> {
        let command = DispenseHopperCoinsCommand::new(coins);
        let response_packet = self.send_command(command).await?;
        DispenseHopperCoinsCommand::new(coins)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn payout_serial_number(&self, coins: u8) -> DeviceResult<Option<u8>> {
        let serial_number = self.get_serial_number().await?;
        let command = DispenseHopperCoinsCommand::new_with_data(
            coins,
            &[
                serial_number.fix(),
                serial_number.minor(),
                serial_number.major(),
            ],
        );
        let response_packet = self.send_command(command).await?;
        DispenseHopperCoinsCommand::new(coins)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn payout_no_encryption(&self, coins: u8) -> DeviceResult<Option<u8>> {
        self.send_command(PumpRngCommand::new([0, 0, 0, 0, 0, 0, 0, 0]))
            .await?;
        self.send_command(RequestCipherKeyCommand).await?;
        let command =
            DispenseHopperCoinsCommand::new_with_data(coins, &[0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let response_packet = self.send_command(command).await?;
        DispenseHopperCoinsCommand::new(coins)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn purge(&self, hopper_number: u8, count: u8) -> DeviceResult<()> {
        let command = PurgeHopperCommand::new(hopper_number, count);
        let response_packet = self.send_command(command).await?;
        PurgeHopperCommand::new(hopper_number, count)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn get_dispense_count(&self) -> DeviceResult<u32> {
        let response_packet = self.send_command(RequestHopperDispenseCountCommand).await?;
        RequestHopperDispenseCountCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn whm_100_speed_adjust(&self, permanent: bool, speed: u8) -> DeviceResult<()> {
        let permanent_flag: u8 = if permanent { 2 } else { 1 };
        let command = OperateBiDirectionalMotorsCommand::new(permanent_flag, speed, 0);
        let response_packet = self.send_command(command).await?;
        OperateBiDirectionalMotorsCommand::new(0, 0, 0)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
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
