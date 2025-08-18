#![allow(dead_code)]

use cc_talk_core::cc_talk::{
    BillRouteCode, BillRoutingError, BillValidatorPollResult, BitMask, CurrencyToken, Device,
};
use cc_talk_host::{command::Command, device::device_commands::*};
use tokio::sync::mpsc;

use crate::transport::tokio_transport::TransportMessage;

use super::base::{CommandError, DeviceCommon, DeviceResult};

pub struct BillValidator {
    pub device: Device,
    pub sender: mpsc::Sender<TransportMessage>,
    pub event_counter: u8,
}

impl BillValidator {
    pub fn new(device: Device, sender: mpsc::Sender<TransportMessage>) -> Self {
        Self {
            device,
            sender,
            event_counter: 0,
        }
    }

    pub async fn set_master_inhibit(&self, inhibit: bool) -> DeviceResult<()> {
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

    /// Requests the bill operating mode of the bill validator.
    /// Returns a tuple:
    /// first element is for the stacker availability
    /// second element is for the escrow availability
    pub async fn request_operating_mode(&self) -> DeviceResult<(bool, bool)> {
        let response_packet = self.send_command(RequestBillOperatingModeCommand).await?;
        RequestBillOperatingModeCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn set_operating_mode(
        &self,
        use_stacker: bool,
        use_escrow: bool,
    ) -> DeviceResult<()> {
        let command = ModifyBillOperatingModeCommand::new(use_stacker, use_escrow);
        let response_packet = self.send_command(command).await?;
        ModifyBillOperatingModeCommand::new(use_stacker, use_escrow)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn request_bill_id(&self, id: u8) -> DeviceResult<CurrencyToken> {
        let response_packet = self.send_command(RequestBillIdCommand::new(id)).await?;
        RequestBillIdCommand::new(id)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn request_all_bill_id(&self) -> DeviceResult<Vec<(u8, Option<CurrencyToken>)>> {
        let mut bills = std::vec::Vec::with_capacity(16);
        for i in 0..16 {
            if let Ok(bill) = self.request_bill_id(i).await {
                bills.push((i, Some(bill)));
            } else {
                bills.push((i, None));
            }
        }
        Ok(bills)
    }

    /// Sets the inhibit status for each of the 16 bill positions.
    /// True: bill is DISABLED
    /// False: bill is ENABLED
    pub async fn set_bill_inhibits(&self, inhibits: [bool; 16]) -> DeviceResult<()> {
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

    pub async fn set_all_bill_inhibits(&self, inhibit: bool) -> DeviceResult<()> {
        let inhibits = [inhibit; 16];
        self.set_bill_inhibits(inhibits).await
    }

    /// Requests the inhibit status for each of the 16 bill positions.
    /// True: bill is disabled
    /// False: bill is enabled
    pub async fn get_bill_inhibits(&self) -> DeviceResult<Vec<bool>> {
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

    pub async fn route_bill(
        &self,
        route_code: BillRouteCode,
    ) -> DeviceResult<Option<BillRoutingError>> {
        let command = RouteBillCommand::new(route_code);
        let response_packet = self.send_command(command).await?;
        RouteBillCommand::new(route_code)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    pub async fn poll(&mut self) -> DeviceResult<BillValidatorPollResult> {
        let response_packet = self
            .send_command(ReadBufferedBillEventsCommand::default())
            .await?;
        ReadBufferedBillEventsCommand::new(self.event_counter)
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .inspect(|result| {
                self.event_counter = result.event_counter;
            })
    }

    pub async fn get_polling_priority(&self) -> DeviceResult<PollingPriority> {
        let response_packet = self.send_command(RequestPollingPriorityCommand).await?;
        RequestPollingPriorityCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }
}

impl DeviceCommon for BillValidator {
    fn get_device(&self) -> &Device {
        &self.device
    }

    fn get_sender(&self) -> &mpsc::Sender<TransportMessage> {
        &self.sender
    }
}
