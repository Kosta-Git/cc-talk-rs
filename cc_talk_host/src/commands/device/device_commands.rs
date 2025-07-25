#![allow(dead_code)]
use cc_talk_core::{
    Fault, Header,
    cc_talk::{
        BitMask, BitMaskError, CoinAcceptorPollResult, CurrencyToken, FaultCode, HopperStatus,
        PowerOption, RequestOptionFlags, SorterPath, TeachModeStatus,
    },
};

use crate::commands::command::{Command, ParseResponseError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollingUnit {
    Special = 0,
    Ms = 1,
    X10Ms = 2,
    Seconds = 3,
    Minutes = 4,
    Hours = 5,
    Days = 6,
    Weeks = 7,
    Months = 8,
    Years = 9,
}
pub struct PollingPriority {
    pub unit: PollingUnit,
    pub value: u8,
}
pub struct RequestPollingPriorityCommand;
impl Command for RequestPollingPriorityCommand {
    type Response = PollingPriority;

    fn header(&self) -> Header {
        Header::RequestPollingPriority
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            2 => {
                let unit = match response_payload[0] {
                    0 => PollingUnit::Special,
                    1 => PollingUnit::Ms,
                    2 => PollingUnit::X10Ms,
                    3 => PollingUnit::Seconds,
                    4 => PollingUnit::Minutes,
                    5 => PollingUnit::Hours,
                    6 => PollingUnit::Days,
                    7 => PollingUnit::Weeks,
                    8 => PollingUnit::Months,
                    9 => PollingUnit::Years,
                    _ => return Err(ParseResponseError::ParseError("Invalid polling unit")),
                };
                Ok(PollingPriority {
                    unit,
                    value: response_payload[1],
                })
            }
            _ => Err(ParseResponseError::DataLengthMismatch(
                2,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoinAcceptorStatus {
    Ok = 0,
    CoinReturnMechanismActivated = 1,
    CoinOnString = 2,
}
pub struct RequestStatusCommand;
impl Command for RequestStatusCommand {
    type Response = CoinAcceptorStatus;

    fn header(&self) -> Header {
        Header::RequestStatus
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            1 => match response_payload[0] {
                0 => Ok(CoinAcceptorStatus::Ok),
                1 => Ok(CoinAcceptorStatus::CoinReturnMechanismActivated),
                2 => Ok(CoinAcceptorStatus::CoinOnString),
                _ => Err(ParseResponseError::ParseError("Invalid status")),
            },
            _ => Err(ParseResponseError::DataLengthMismatch(
                1,
                response_payload.len(),
            )),
        }
    }
}

pub struct RequestVariableSetCommand;
impl Command for RequestVariableSetCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::RequestVariableSet
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Device specific
    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        Ok(())
    }
}

pub struct RequestDatabaseVersionCommand;
impl Command for RequestDatabaseVersionCommand {
    type Response = u8;

    fn header(&self) -> Header {
        Header::RequestDatabaseVersion
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            1 => Ok(response_payload[0]),
            _ => Err(ParseResponseError::DataLengthMismatch(
                1,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestSolenoidsCommand {
    buffer: u8, // maybe this should be an array of u8?
}
impl TestSolenoidsCommand {
    /// Creates a new TestSolenoidsCommand with the given bitmask.
    pub fn new(bitmask: u8) -> Self {
        TestSolenoidsCommand { buffer: bitmask }
    }
}
impl Command for TestSolenoidsCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::TestSolenoids
    }

    fn data(&self) -> &[u8] {
        core::slice::from_ref(&self.buffer)
    }

    /// Replies with ack
    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperateMotorsCommand {
    buffer: u8,
}
impl OperateMotorsCommand {
    /// Creates a new OperateMotorsCommand with the given bitmask.
    pub fn new(bitmask: u8) -> Self {
        OperateMotorsCommand { buffer: bitmask }
    }
}
impl Command for OperateMotorsCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::OperateMotors
    }

    fn data(&self) -> &[u8] {
        core::slice::from_ref(&self.buffer)
    }

    /// Replies with ack
    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestOutputLinesCommand {
    buffer: u8, // Maybe this should be an array of u8?
}
impl TestOutputLinesCommand {
    /// Creates a new TestOutputLinesCommand with the given bitmask.
    pub fn new(bitmask: u8) -> Self {
        TestOutputLinesCommand { buffer: bitmask }
    }
}
impl Command for TestOutputLinesCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::TestOutputLines
    }

    fn data(&self) -> &[u8] {
        core::slice::from_ref(&self.buffer)
    }

    /// Replies with ack
    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadInputLinesCommand;
impl Command for ReadInputLinesCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ReadInputLines
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// We can't really make assumptions here, its device specific.
    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadOptoStatesCommand;
impl Command for ReadOptoStatesCommand {
    type Response = u8; // Assuming the response is a single byte representing the opto states.

    fn header(&self) -> Header {
        Header::ReadOptoStates
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// We can't really make assumptions here, its device specific.
    fn parse_response(&self, payload: &[u8]) -> Result<Self::Response, ParseResponseError> {
        match payload.len() {
            1 => Ok(payload[0]),
            2..=usize::MAX => {
                // TODO: Add defmt/log optional logging here with a warning about unexpected data
                // length. Might need to make this return a dynamic size, or fixed size like 4u8
                // which should be plenty for opto states.
                Ok(payload[0]) // Assuming the first byte is the opto states.)
            }
            _ => Err(ParseResponseError::DataLengthMismatch(1, payload.len())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatchOutputLinesCommand {
    buffer: u8,
}
impl Command for LatchOutputLinesCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::LatchOutputLines
    }

    fn data(&self) -> &[u8] {
        core::slice::from_ref(&self.buffer)
    }

    /// Replies with ack
    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        Ok(())
    }
}

pub struct PerformSelfCheckCommand;
impl Command for PerformSelfCheckCommand {
    type Response = Fault;

    fn header(&self) -> Header {
        Header::PerformSelfCheck
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Replies with ack
    fn parse_response(&self, payload: &[u8]) -> Result<Self::Response, ParseResponseError> {
        match payload.len() {
            1 => {
                let fault_code = FaultCode::try_from(payload[0])
                    .map_err(|_| ParseResponseError::ParseError("Invalid fault code"))?;

                Ok(Fault::new(fault_code))
            }
            2 => {
                let fault_code = FaultCode::try_from(payload[0])
                    .map_err(|_| ParseResponseError::ParseError("Invalid fault code"))?;
                let fault_info = payload[1];

                Ok(Fault::with_info(fault_code, fault_info))
            }
            _ => Err(ParseResponseError::DataLengthMismatch(0, payload.len())),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModifyInhibitStatusCommand<const N: usize> {
    buffer: [u8; N],
}
impl<const N: usize> ModifyInhibitStatusCommand<N> {
    pub fn build(mask: BitMask<N>) -> Result<Self, BitMaskError> {
        Ok(ModifyInhibitStatusCommand {
            buffer: mask.to_le_bytes::<N>()?,
        })
    }
}
impl<const N: usize> Command for ModifyInhibitStatusCommand<N> {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyInhibitStatus
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    /// Replies with ack
    fn parse_response(&self, payload: &[u8]) -> Result<Self::Response, ParseResponseError> {
        if payload.is_empty() {
            Ok(())
        } else {
            Err(ParseResponseError::DataLengthMismatch(0, payload.len()))
        }
    }
}

pub struct RequestInhibitStatusCommand<const N: usize>;
impl<const N: usize> Command for RequestInhibitStatusCommand<N> {
    type Response = [u8; N];

    fn header(&self) -> Header {
        Header::RequestInhibitStatus
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            len if len == N => Ok(response_payload.try_into().unwrap()),
            len if len > N => {
                crate::log::info!("unexpected response length: expected {}, got {}", N, len);
                Ok(response_payload[0..len].try_into().unwrap())
            }
            _ => Err(ParseResponseError::DataLengthMismatch(
                4,
                response_payload.len(),
            )),
        }
    }
}

pub struct ReadBufferedCreditOrErrorCodeCommand;
impl Command for ReadBufferedCreditOrErrorCodeCommand {
    type Response = CoinAcceptorPollResult;

    fn header(&self) -> Header {
        Header::ReadBufferedCreditOrErrorCodes
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(&self, payload: &[u8]) -> Result<Self::Response, ParseResponseError> {
        if payload.is_empty() {
            return Err(ParseResponseError::DataLengthMismatch(1, payload.len()));
        }

        CoinAcceptorPollResult::try_from(payload)
            .map_err(|_| ParseResponseError::ParseError("Invalid coin acceptor poll result"))
    }
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModifyMasterInhibitStatusCommand<const N: usize> {
    buffer: [u8; N],
}
impl<const N: usize> ModifyMasterInhibitStatusCommand<N> {
    pub fn build(mask: BitMask<N>) -> Result<Self, BitMaskError> {
        Ok(ModifyMasterInhibitStatusCommand {
            buffer: mask.to_le_bytes::<N>()?,
        })
    }
}
impl<const N: usize> Command for ModifyMasterInhibitStatusCommand<N> {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyMasterInhibitStatus
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(&self, payload: &[u8]) -> Result<Self::Response, ParseResponseError> {
        if payload.is_empty() {
            Ok(())
        } else {
            Err(ParseResponseError::DataLengthMismatch(0, payload.len()))
        }
    }
}

pub struct RequestMasterInhibitStatusCommand<const N: usize>;
impl<const N: usize> Command for RequestMasterInhibitStatusCommand<N> {
    type Response = [u8; N];

    fn header(&self) -> Header {
        Header::RequestMasterInhibitStatus
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            len if len == N => Ok(response_payload
                .try_into()
                .map_err(|_| ParseResponseError::ParseError("unable to map to slice"))?),
            len if len > N => {
                crate::log::info!("unexpected response length: expected {}, got {}", N, len);
                Ok(response_payload[0..len]
                    .try_into()
                    .map_err(|_| ParseResponseError::ParseError("unable to map to slice"))?)
            }
            _ => Err(ParseResponseError::DataLengthMismatch(
                4,
                response_payload.len(),
            )),
        }
    }
}

pub struct RequestInsertionCounterCommand;
impl Command for RequestInsertionCounterCommand {
    type Response = u32;

    fn header(&self) -> Header {
        Header::RequestInsertionCounter
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            3 => Ok(u32::from_le_bytes([
                response_payload[0],
                response_payload[1],
                response_payload[2],
                0u8,
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                3,
                response_payload.len(),
            )),
        }
    }
}

pub struct RequestCreditCounterCommand;
impl Command for RequestCreditCounterCommand {
    type Response = u32;

    fn header(&self) -> Header {
        Header::RequestAcceptCounter
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            3 => Ok(u32::from_le_bytes([
                response_payload[0],
                response_payload[1],
                response_payload[2],
                0u8,
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                3,
                response_payload.len(),
            )),
        }
    }
}

// TODO: Implement this once encryption is supported
pub struct ModifyEncryptedInhibitAndOverrideRegistersCommand;

pub struct ModifySorterOverrideStatusCommand {
    buffer: u8,
}
impl ModifySorterOverrideStatusCommand {
    pub fn build(bitmask: BitMask<1>) -> Result<Self, BitMaskError> {
        Ok(ModifySorterOverrideStatusCommand {
            buffer: bitmask.to_le_bytes::<1>()?[0],
        })
    }
}
impl Command for ModifySorterOverrideStatusCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifySorterOverrideStatus
    }

    fn data(&self) -> &[u8] {
        core::slice::from_ref(&self.buffer)
    }

    fn parse_response(&self, payload: &[u8]) -> Result<Self::Response, ParseResponseError> {
        if payload.is_empty() {
            Ok(())
        } else {
            Err(ParseResponseError::DataLengthMismatch(0, payload.len()))
        }
    }
}

pub struct RequestSorterOverrideStatusCommand;
impl Command for RequestSorterOverrideStatusCommand {
    type Response = BitMask<1>;

    fn header(&self) -> Header {
        Header::RequestSorterOverrideStatus
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            1 => BitMask::<1>::from_le_bytes(response_payload, 8).map_err(|_| {
                ParseResponseError::ParseError("Invalid sorter override status bitmask")
            }),
            _ => Err(ParseResponseError::DataLengthMismatch(
                1,
                response_payload.len(),
            )),
        }
    }
}

pub struct EnterNewPinNumberCommand {
    pub pin: [u8; 4],
}
impl Command for EnterNewPinNumberCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::EnterNewPinNumber
    }

    fn data(&self) -> &[u8] {
        &self.pin
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()), // No data expected in response
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

pub struct EnterPinNumberCommand {
    pub pin: [u8; 4],
}
impl Command for EnterPinNumberCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::EnterPinNumber
    }

    fn data(&self) -> &[u8] {
        &self.pin
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()), // No data expected in response
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

pub struct RequestpayoutHighLowStatusCommand;
impl Command for RequestpayoutHighLowStatusCommand {
    type Response = (u8, HopperStatus);

    fn header(&self) -> Header {
        Header::RequestPayoutStatus
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(&self, payload: &[u8]) -> Result<Self::Response, ParseResponseError> {
        match payload.len() {
            1 => Ok((0, HopperStatus::from(payload[0]))),
            2 => Ok((payload[0], HopperStatus::from(payload[1]))),
            _ => Err(ParseResponseError::DataLengthMismatch(1, payload.len())),
        }
    }
}

/// The size `N` should be retrieved from [Header::DataStorageAvailability]
pub struct ReadDataBlockCommand<const N: usize> {
    pub block_number: u8,
}
impl<const N: usize> Command for ReadDataBlockCommand<N> {
    type Response = [u8; N];

    fn header(&self) -> Header {
        Header::ReadDataBlock
    }

    fn data(&self) -> &[u8] {
        core::slice::from_ref(&self.block_number)
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            len if len == N => Ok(response_payload.try_into().unwrap()),
            len if len > N => {
                crate::log::info!("unexpected response length: expected {}, got {}", N, len);
                Ok(response_payload[0..N].try_into().unwrap())
            }
            _ => Err(ParseResponseError::DataLengthMismatch(
                N,
                response_payload.len(),
            )),
        }
    }
}

/// The size `N` should be retrieved from [Header::DataStorageAvailability]
pub struct WriteDataBlockCommand<const N: usize> {
    data: heapless::Vec<u8, 256>,
}
impl<const N: usize> WriteDataBlockCommand<N> {
    pub fn new(block_number: u8, buffer: &[u8]) -> Result<Self, ()> {
        if buffer.len() > N {
            return Err(());
        }

        let mut data = heapless::Vec::new();
        data.push(block_number).map_err(|_| ())?;
        data.extend_from_slice(buffer).map_err(|_| ())?;

        Ok(WriteDataBlockCommand { data })
    }
}
impl<const N: usize> Command for WriteDataBlockCommand<N> {
    type Response = ();

    fn header(&self) -> Header {
        Header::WriteDataBlock
    }

    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.is_empty() {
            Ok(())
        } else {
            Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            ))
        }
    }
}

pub struct RequestOptionFlagsCommand;
impl Command for RequestOptionFlagsCommand {
    type Response = RequestOptionFlags;

    fn header(&self) -> Header {
        Header::RequestOptionFlags
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    // Returns the option flags, you then have to convert them to the specific device type.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            1 => Ok(RequestOptionFlags::new(response_payload[0])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                1,
                response_payload.len(),
            )),
        }
    }
}

pub struct RequestCoinPositionCommand {
    buffer: [u8; 1],
}
impl RequestCoinPositionCommand {
    pub fn new(coin_position: u8) -> Self {
        RequestCoinPositionCommand {
            buffer: [coin_position],
        }
    }
}
impl Command for RequestCoinPositionCommand {
    type Response = (u8, u8);

    fn header(&self) -> Header {
        Header::RequestCoinPosition
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            2 => Ok((response_payload[0], response_payload[1])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                2,
                response_payload.len(),
            )),
        }
    }
}

pub struct PowerManagementControlCommand {
    buffer: [u8; 1],
}
impl PowerManagementControlCommand {
    pub fn new(power_option: PowerOption) -> Self {
        PowerManagementControlCommand {
            buffer: [power_option as u8],
        }
    }
}
impl Command for PowerManagementControlCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::PowerManagementControl
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.is_empty() {
            Ok(())
        } else {
            Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            ))
        }
    }
}

pub struct ModifySorterPathCommand {
    buffer: [u8; 2],
}
impl ModifySorterPathCommand {
    pub fn new(coin_position: u8, sorter: u8) -> Self {
        ModifySorterPathCommand {
            buffer: [coin_position, sorter],
        }
    }
}
impl Command for ModifySorterPathCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifySorterPaths
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.is_empty() {
            Ok(())
        } else {
            Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            ))
        }
    }
}

pub struct RequestSorterPathCommand {
    buffer: [u8; 1],
}
impl RequestSorterPathCommand {
    pub fn new(coin_position: u8) -> Self {
        RequestSorterPathCommand {
            buffer: [coin_position],
        }
    }
}
impl Command for RequestSorterPathCommand {
    type Response = SorterPath;

    fn header(&self) -> Header {
        Header::RequestSorterPaths
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            1 => Ok(SorterPath::from(response_payload[0])),
            2..=usize::MAX => {
                crate::log::info!(
                    "multipath coin are not yet supported, got {} bytes",
                    response_payload.len()
                );
                Ok(SorterPath::from(response_payload[0]))
            }
            _ => Err(ParseResponseError::DataLengthMismatch(
                1,
                response_payload.len(),
            )),
        }
    }
}

pub struct ModifyPayoutAbsoluteCountCommand {
    buffer: [u8; 3],
    has_hopper_number: bool,
}
impl ModifyPayoutAbsoluteCountCommand {
    pub fn new(count: u32) -> Self {
        ModifyPayoutAbsoluteCountCommand {
            buffer: [(count & 0xFF) as u8, ((count >> 8) & 0xFF) as u8, 0u8],
            has_hopper_number: false,
        }
    }

    pub fn new_with_hopper(hopper_number: u8, count: u32) -> Self {
        ModifyPayoutAbsoluteCountCommand {
            buffer: [
                hopper_number,
                (count & 0xFF) as u8,
                ((count >> 8) & 0xFF) as u8,
            ],
            has_hopper_number: true,
        }
    }
}
impl Command for ModifyPayoutAbsoluteCountCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyPayoutAbsoluteCount
    }

    fn data(&self) -> &[u8] {
        if self.has_hopper_number {
            &self.buffer[..]
        } else {
            &self.buffer[..2]
        }
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.is_empty() {
            Ok(())
        } else {
            Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            ))
        }
    }
}

pub struct RequestPayoutAbsoluteCountCommand {
    buffer: [u8; 1],
    has_hopper_number: bool,
}
impl RequestPayoutAbsoluteCountCommand {
    pub fn new() -> Self {
        RequestPayoutAbsoluteCountCommand {
            buffer: [0u8],
            has_hopper_number: false,
        }
    }

    pub fn new_with_hopper(hopper_number: u8) -> Self {
        RequestPayoutAbsoluteCountCommand {
            buffer: [hopper_number],
            has_hopper_number: true,
        }
    }
}
impl Command for RequestPayoutAbsoluteCountCommand {
    type Response = u16;

    fn header(&self) -> Header {
        Header::RequestPayoutAbsoluteCount
    }

    fn data(&self) -> &[u8] {
        if self.has_hopper_number {
            &self.buffer[..]
        } else {
            &[]
        }
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            2 => Ok(u16::from_le_bytes([
                response_payload[0],
                response_payload[1],
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                2,
                response_payload.len(),
            )),
        }
    }
}

// TODO: Implement this
pub struct MeterControlCommand;

// TODO: Implement this
pub struct DisplayControlCommand;

pub struct TeachModeControlCommand {
    buffer: [u8; 2],
    has_orientation: bool,
}
impl TeachModeControlCommand {
    pub fn new(position: u8) -> Self {
        TeachModeControlCommand {
            buffer: [position, 0u8],
            has_orientation: false,
        }
    }

    pub fn new_with_orientation(position: u8, orientation: u8) -> Self {
        TeachModeControlCommand {
            buffer: [position, orientation],
            has_orientation: true,
        }
    }
}
impl Command for TeachModeControlCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::TeachModeControl
    }

    fn data(&self) -> &[u8] {
        if self.has_orientation {
            &self.buffer[..]
        } else {
            &self.buffer[..1]
        }
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.is_empty() {
            Ok(())
        } else {
            Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            ))
        }
    }
}

pub struct RequestTeachModeStatusCommand {
    buffer: [u8; 1],
}
impl RequestTeachModeStatusCommand {
    pub fn new(abort: bool) -> Self {
        RequestTeachModeStatusCommand {
            buffer: [if abort { 1 } else { 0 }],
        }
    }
}
impl Command for RequestTeachModeStatusCommand {
    type Response = (u8, TeachModeStatus);

    fn header(&self) -> Header {
        Header::RequestTeachStatus
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    // Returns (number of coins, TeachModeStatus)
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            2 => Ok((
                response_payload[0],
                TeachModeStatus::from(response_payload[1]),
            )),
            _ => Err(ParseResponseError::DataLengthMismatch(
                2,
                response_payload.len(),
            )),
        }
    }
}

pub struct ConfigurationToEepromCommand;
impl Command for ConfigurationToEepromCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ConfigurationToEEPROM
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Replies with ack
    fn parse_response(&self, payload: &[u8]) -> Result<Self::Response, ParseResponseError> {
        match payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(0, payload.len())),
        }
    }
}

pub struct CountersToEepromCommand;
impl Command for CountersToEepromCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::CountersToEEPROM
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(&self, payload: &[u8]) -> Result<Self::Response, ParseResponseError> {
        match payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(0, payload.len())),
        }
    }
}

pub struct RequestRejectCounterCommand;
impl Command for RequestRejectCounterCommand {
    type Response = u32;

    fn header(&self) -> Header {
        Header::RequestRejectCounter
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            3 => Ok(u32::from_le_bytes([
                response_payload[0],
                response_payload[1],
                response_payload[2],
                0u8,
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                3,
                response_payload.len(),
            )),
        }
    }
}

pub struct RequestFraudCounterCommand;
impl Command for RequestFraudCounterCommand {
    type Response = u32;

    fn header(&self) -> Header {
        Header::RequestFraudCounter
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            3 => Ok(u32::from_le_bytes([
                response_payload[0],
                response_payload[1],
                response_payload[2],
                0u8,
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                3,
                response_payload.len(),
            )),
        }
    }
}

// TODO: Implement this
pub struct KeypadControlCommand;

pub struct ModifyDefaultSorterPathCommand {
    buffer: [u8; 1],
}
impl ModifyDefaultSorterPathCommand {
    pub fn new(sorter: u8) -> Self {
        ModifyDefaultSorterPathCommand { buffer: [sorter] }
    }
}
impl Command for ModifyDefaultSorterPathCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyDefaultSorterPath
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.is_empty() {
            Ok(())
        } else {
            Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            ))
        }
    }
}

pub struct RequestDefaultSorterPathCommand;
impl Command for RequestDefaultSorterPathCommand {
    type Response = SorterPath;

    fn header(&self) -> Header {
        Header::RequestDefaultSorterPath
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            1 => Ok(SorterPath::from(response_payload[0])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                1,
                response_payload.len(),
            )),
        }
    }
}

pub struct ModifyPayoutCapacityCommand {
    buffer: [u8; 3],
    has_hopper_number: bool,
}
impl ModifyPayoutCapacityCommand {
    pub fn new(capacity: u16) -> Self {
        ModifyPayoutCapacityCommand {
            buffer: [(capacity & 0xFF) as u8, ((capacity >> 8) & 0xFF) as u8, 0u8],
            has_hopper_number: false,
        }
    }

    pub fn new_with_hopper(hopper_number: u8, capacity: u16) -> Self {
        ModifyPayoutCapacityCommand {
            buffer: [
                hopper_number,
                (capacity & 0xFF) as u8,
                ((capacity >> 8) & 0xFF) as u8,
            ],
            has_hopper_number: true,
        }
    }
}
impl Command for ModifyPayoutCapacityCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyPayoutCapacity
    }

    fn data(&self) -> &[u8] {
        if self.has_hopper_number {
            &self.buffer[..]
        } else {
            &self.buffer[..2]
        }
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.is_empty() {
            Ok(())
        } else {
            Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            ))
        }
    }
}

pub struct RequestPayoutCapacityCommand {
    buffer: [u8; 1],
    has_hopper_number: bool,
}
impl RequestPayoutCapacityCommand {
    pub fn new() -> Self {
        RequestPayoutCapacityCommand {
            buffer: [0u8],
            has_hopper_number: false,
        }
    }

    pub fn new_with_hopper(hopper_number: u8) -> Self {
        RequestPayoutCapacityCommand {
            buffer: [hopper_number],
            has_hopper_number: true,
        }
    }
}
impl Command for RequestPayoutCapacityCommand {
    type Response = u16;

    fn header(&self) -> Header {
        Header::RequestPayoutCapacity
    }

    fn data(&self) -> &[u8] {
        if self.has_hopper_number {
            &self.buffer[..]
        } else {
            &[]
        }
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            2 => Ok(u16::from_le_bytes([
                response_payload[0],
                response_payload[1],
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                2,
                response_payload.len(),
            )),
        }
    }
}

pub struct ModifyCoinIdCommand {
    buffer: [u8; 7],
}
impl ModifyCoinIdCommand {
    pub fn new(coin_position: u8, coin_id: &[u8; 6]) -> Self {
        ModifyCoinIdCommand {
            buffer: [
                coin_position,
                coin_id[0],
                coin_id[1],
                coin_id[2],
                coin_id[3],
                coin_id[4],
                coin_id[5],
            ],
        }
    }
}
impl Command for ModifyCoinIdCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyCoinId
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.is_empty() {
            Ok(())
        } else {
            Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            ))
        }
    }
}

pub struct RequestCoinIdCommand {
    buffer: [u8; 1],
}
impl RequestCoinIdCommand {
    pub fn new(coin_position: u8) -> Self {
        RequestCoinIdCommand {
            buffer: [coin_position],
        }
    }
}
impl Command for RequestCoinIdCommand {
    type Response = CurrencyToken;

    fn header(&self) -> Header {
        Header::RequestCoinId
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            6 => {
                let payload_str = core::str::from_utf8(&response_payload[0..6])
                    .map_err(|_| ParseResponseError::ParseError("Invalid UTF-8 in coin ID"))?;

                CurrencyToken::build(payload_str)
                    .map_err(|_| ParseResponseError::ParseError("Invalid coin ID format"))
            }
            _ => Err(ParseResponseError::DataLengthMismatch(
                6,
                response_payload.len(),
            )),
        }
    }
}
