#![allow(dead_code)]

use cc_talk_core::cc_talk::{
    BillRouteCode, BillRoutingError, BillValidatorPollResult, BillValidatorPollResultError,
    BitMask, BitMaskError, ChangerDevice, ChangerError, ChangerFlags, ChangerPollResult,
    CoinAcceptorPollResult, CurrencyToken, CurrencyTokenError, EscrowFaultCode, EscrowLevelStatus,
    EscrowOperatingStatus, EscrowServiceStatus, Fault, FaultCode, FirmwareStorageType, Header,
    HopperDispenseStatus, HopperDispenseValueStatus, HopperFlag, HopperStatus, LampControl,
    PowerOption, RequestOptionFlags, SorterPath, StackerCycleError, TeachModeStatus,
    parse_changer_flags_heapless,
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
#[derive(Debug)]
pub struct PollingPriority {
    pub unit: PollingUnit,
    pub value: u8,
}
#[cfg(feature = "std")]
impl PollingPriority {
    pub fn as_duration(&self) -> Option<std::time::Duration> {
        use std::time::Duration;

        let value = self.value as u64;

        let duration = match self.unit {
            PollingUnit::Special => {
                return None;
            }
            PollingUnit::Ms => Duration::from_millis(value),
            PollingUnit::X10Ms => Duration::from_millis(value * 10),
            PollingUnit::Seconds => Duration::from_secs(value),
            PollingUnit::Minutes => Duration::from_secs(value * 60),
            PollingUnit::Hours => Duration::from_secs(value * 3600),
            PollingUnit::Days => Duration::from_secs(value * 86400),
            PollingUnit::Weeks => Duration::from_secs(value * 604800),
            PollingUnit::Months => Duration::from_secs(value * 2629746), // ~30.44 days
            PollingUnit::Years => Duration::from_secs(value * 31556952), // ~365.25 days
        };
        Some(duration)
    }
}
#[derive(Debug)]
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
#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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
                crate::log::warning!(
                    "expected size of 1, but got {} instead. Maybe some information got lost.",
                    payload.len()
                );
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug, Default)]
pub struct ReadBufferedCreditOrErrorCodeCommand {
    last_event_counter: u8,
}
impl ReadBufferedCreditOrErrorCodeCommand {
    pub fn new(last_event_counter: u8) -> Self {
        ReadBufferedCreditOrErrorCodeCommand { last_event_counter }
    }
}
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
        CoinAcceptorPollResult::try_from((payload, self.last_event_counter))
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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
#[derive(Debug)]
pub struct ModifyEncryptedInhibitAndOverrideRegistersCommand;

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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
impl Default for RequestPayoutAbsoluteCountCommand {
    fn default() -> Self {
        Self::new()
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
#[derive(Debug)]
pub struct MeterControlCommand;

// TODO: Implement this
#[derive(Debug)]
pub struct DisplayControlCommand;

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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
#[derive(Debug)]
pub struct KeypadControlCommand;

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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
impl Default for RequestPayoutCapacityCommand {
    fn default() -> Self {
        Self::new()
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct UploadWindowDataCommand {
    buffer: [u8; 3],
    size: u8,
}
impl UploadWindowDataCommand {
    pub fn program_coin(position: u8) -> Self {
        UploadWindowDataCommand {
            buffer: [0u8, position, 0u8],
            size: 2,
        }
    }

    pub fn modify_credit_code(position: u8, credit_code: u8) -> Self {
        UploadWindowDataCommand {
            buffer: [1u8, position, credit_code],
            size: 3,
        }
    }

    pub fn delete_coin(position: u8) -> Self {
        UploadWindowDataCommand {
            buffer: [2u8, position, 0],
            size: 2,
        }
    }

    pub fn program_token(position: u8, data: u8) -> Self {
        UploadWindowDataCommand {
            buffer: [3u8, position, data],
            size: 3,
        }
    }

    pub fn delete_token(position: u8) -> Self {
        UploadWindowDataCommand {
            buffer: [4, position, 0],
            size: 2,
        }
    }
}
impl Command for UploadWindowDataCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::UploadWindowData
    }

    fn data(&self) -> &[u8] {
        &self.buffer[..self.size as usize]
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

/// This command is device specific, no validation/parsing is provided.
#[derive(Debug)]
pub struct DownloadCalibrationDataCommand;
impl Command for DownloadCalibrationDataCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::DownloadCalibrationInfo
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct ModifySecuritySettingCommand {
    buffer: [u8; 2],
}
impl ModifySecuritySettingCommand {
    pub fn new(position: u8, security_setting: u8) -> Self {
        // TODO: use an enum for security_setting
        ModifySecuritySettingCommand {
            buffer: [position, security_setting],
        }
    }
}
impl Command for ModifySecuritySettingCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifySecuritySetting
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestSecuritySettingCommand {
    buffer: [u8; 1],
}
impl RequestSecuritySettingCommand {
    pub fn new(position: u8) -> Self {
        RequestSecuritySettingCommand { buffer: [position] }
    }
}
impl Command for RequestSecuritySettingCommand {
    type Response = u8;

    fn header(&self) -> Header {
        Header::RequestSecuritySetting
    }

    fn data(&self) -> &[u8] {
        &self.buffer
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

#[derive(Debug)]
pub struct ModifyBankSelectCommand {
    buffer: [u8; 1],
}
impl ModifyBankSelectCommand {
    pub fn new(bank: u8) -> Self {
        ModifyBankSelectCommand { buffer: [bank] }
    }
}
impl Command for ModifyBankSelectCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyBankSelect
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestBankSelectCommand;
impl Command for RequestBankSelectCommand {
    type Response = u8;

    fn header(&self) -> Header {
        Header::RequestBankSelect
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

// TODO: Implement this
#[derive(Debug)]
pub struct HandheldFunctionCommand;

#[derive(Debug)]
pub struct RequestAlarmCounterCommand;
impl Command for RequestAlarmCounterCommand {
    type Response = u8;

    fn header(&self) -> Header {
        Header::RequestAlarmCounter
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

#[derive(Debug)]
pub struct ModifyPayoutFloatCommand {
    buffer: [u8; 3],
    has_hopper_number: bool,
}
impl ModifyPayoutFloatCommand {
    pub fn new(number_of_coins: u16) -> Self {
        ModifyPayoutFloatCommand {
            buffer: [
                (number_of_coins & 0xFF) as u8,
                ((number_of_coins >> 8) & 0xFF) as u8,
                0u8,
            ],
            has_hopper_number: false,
        }
    }

    pub fn new_with_hopper(hopper_number: u8, number_of_coins: u16) -> Self {
        ModifyPayoutFloatCommand {
            buffer: [
                hopper_number,
                (number_of_coins & 0xFF) as u8,
                ((number_of_coins >> 8) & 0xFF) as u8,
            ],
            has_hopper_number: true,
        }
    }
}
impl Command for ModifyPayoutFloatCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyPayoutFloat
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
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestPayoutFloatCommand {
    buffer: [u8; 1],
    has_hopper_number: bool,
}
impl RequestPayoutFloatCommand {
    pub fn new() -> Self {
        RequestPayoutFloatCommand {
            buffer: [0u8],
            has_hopper_number: false,
        }
    }

    pub fn new_with_hopper(hopper_number: u8) -> Self {
        RequestPayoutFloatCommand {
            buffer: [hopper_number],
            has_hopper_number: true,
        }
    }
}

impl Default for RequestPayoutFloatCommand {
    fn default() -> Self {
        Self::new()
    }
}
impl Command for RequestPayoutFloatCommand {
    type Response = u16;

    fn header(&self) -> Header {
        Header::RequestPayoutFloat
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

#[derive(Debug)]
pub struct RequestThermistorReadingCommand;
impl Command for RequestThermistorReadingCommand {
    type Response = u8;

    fn header(&self) -> Header {
        Header::RequestThermistorReading
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

#[derive(Debug)]
pub struct EmergencyStopCommand;
impl Command for EmergencyStopCommand {
    type Response = u8;

    fn header(&self) -> Header {
        Header::EmergencyStop
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
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestHopperCoinCommand;
impl Command for RequestHopperCoinCommand {
    type Response = CurrencyToken;

    fn header(&self) -> Header {
        Header::RequestHopperCoin
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        let coin_string = core::str::from_utf8(response_payload)
            .map_err(|_| ParseResponseError::ParseError("Invalid UTF-8 in coin string"))?;

        CurrencyToken::build(coin_string).map_err(|err| match err {
            CurrencyTokenError::InvalidFormat => {
                ParseResponseError::ParseError("invalid coin string format")
            }
            CurrencyTokenError::ValueStringTooSmall => ParseResponseError::BufferTooSmall,
            CurrencyTokenError::CoinNotSupportedByDevice => {
                ParseResponseError::ParseError("not supported by device")
            }
        })
    }
}

#[derive(Debug)]
pub struct RequestHopperDispenseCountCommand;
impl Command for RequestHopperDispenseCountCommand {
    type Response = u32;

    fn header(&self) -> Header {
        Header::RequestHopperDispenseCount
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
                0,
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                3,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct DispenseHopperCoinsCommand {
    buffer: [u8; 32],
    length: u8,
}
impl DispenseHopperCoinsCommand {
    pub fn new(coins: u8) -> Self {
        let buffer = [coins; 32];
        DispenseHopperCoinsCommand { buffer, length: 1 }
    }

    pub fn new_with_data(coins: u8, additional_data: &[u8]) -> Self {
        const MAX_BUFFER_SIZE: usize = 32;

        let mut buffer = [coins; MAX_BUFFER_SIZE];

        let data_to_copy = additional_data.len().min(MAX_BUFFER_SIZE);
        buffer[..data_to_copy].copy_from_slice(&additional_data[..data_to_copy]);

        let command_length = (1 + additional_data.len()).min(MAX_BUFFER_SIZE) as u8;

        DispenseHopperCoinsCommand {
            buffer,
            length: command_length,
        }
    }
}
impl Command for DispenseHopperCoinsCommand {
    type Response = Option<u8>;

    fn header(&self) -> Header {
        Header::DispenseHopperCoins
    }

    fn data(&self) -> &[u8] {
        &self.buffer[..self.length as usize]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.len() == 1 {
            Ok(Some(response_payload[0]))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
pub struct RequestHopperStatusCommand;
impl Command for RequestHopperStatusCommand {
    type Response = HopperDispenseStatus;

    fn header(&self) -> Header {
        Header::RequestHopperStatus
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            4 => Ok(HopperDispenseStatus::from([
                response_payload[0],
                response_payload[1],
                response_payload[2],
                response_payload[3],
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                4,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct ModifyVariableSetCommand<const N: usize> {
    buffer: [u8; N],
}
impl<const N: usize> ModifyVariableSetCommand<N> {
    pub fn new(buffer: [u8; N]) -> Self {
        ModifyVariableSetCommand { buffer }
    }
}
impl<const N: usize> Command for ModifyVariableSetCommand<N> {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyVariableSet
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

#[derive(Debug)]
pub struct EnableHopperCommand {
    buffer: [u8; 1],
}
impl EnableHopperCommand {
    pub fn new(enable: bool) -> Self {
        EnableHopperCommand {
            buffer: [if enable { 0xA5 } else { 0 }],
        }
    }
}
impl Command for EnableHopperCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::EnableHopper
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

#[derive(Debug)]
pub struct TestHopperCommand;
impl Command for TestHopperCommand {
    type Response = heapless::Vec<HopperFlag, 21>;

    fn header(&self) -> Header {
        Header::TestHopper
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0..=3 => Ok(HopperFlag::parse_hopper_flags_heapless(response_payload)),
            _ => Err(ParseResponseError::DataLengthMismatch(
                3,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct PumpRngCommand<const N: usize> {
    buffer: [u8; N],
}
impl<const N: usize> PumpRngCommand<N> {
    pub fn new(buffer: [u8; N]) -> Self {
        PumpRngCommand { buffer }
    }
}
impl<const N: usize> Command for PumpRngCommand<N> {
    type Response = ();

    fn header(&self) -> Header {
        Header::PumpRNG
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestCipherKeyCommand;
impl Command for RequestCipherKeyCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::RequestCipherKey
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Device specific command, no validation/parsing is provided.
    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ReadBufferedBillEventsCommand {
    last_event_counter: u8,
}
impl ReadBufferedBillEventsCommand {
    pub fn new(last_event_counter: u8) -> Self {
        ReadBufferedBillEventsCommand { last_event_counter }
    }
}
impl Command for ReadBufferedBillEventsCommand {
    type Response = BillValidatorPollResult;

    fn header(&self) -> Header {
        Header::ReadBufferedBillEvents
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        BillValidatorPollResult::try_from(response_payload).map_err(|error| match error {
            BillValidatorPollResultError::NotEnoughEvents => {
                ParseResponseError::ParseError("unexpected number of events (too few)")
            }
            BillValidatorPollResultError::TooManyEvents => {
                ParseResponseError::ParseError("unexpected number of events (too many)")
            }
            BillValidatorPollResultError::InvalidPayload => {
                ParseResponseError::DataLengthMismatch(1357911, response_payload.len())
            }
        })
    }
}

#[derive(Debug)]
pub struct ModifyBillIdCommand {
    buffer: [u8; 8],
}
impl ModifyBillIdCommand {
    pub fn new(bill_type: u8, bill_string: &[u8; 7]) -> Self {
        ModifyBillIdCommand {
            buffer: [
                bill_type,
                bill_string[0],
                bill_string[1],
                bill_string[2],
                bill_string[3],
                bill_string[4],
                bill_string[5],
                bill_string[6],
            ],
        }
    }
}
impl Command for ModifyBillIdCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyBillId
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestBillIdCommand {
    buffer: [u8; 1],
}
impl RequestBillIdCommand {
    pub fn new(bill_type: u8) -> Self {
        RequestBillIdCommand {
            buffer: [bill_type],
        }
    }
}
impl Command for RequestBillIdCommand {
    type Response = CurrencyToken;

    fn header(&self) -> Header {
        Header::RequestBillId
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            7 => {
                let payload_str = core::str::from_utf8(&response_payload[0..7])
                    .map_err(|_| ParseResponseError::ParseError("Invalid UTF-8 in bill ID"))?;

                CurrencyToken::build(payload_str)
                    .map_err(|_| ParseResponseError::ParseError("Invalid bill ID format"))
            }
            _ => Err(ParseResponseError::DataLengthMismatch(
                7,
                response_payload.len(),
            )),
        }
    }
}

// TODO: Implement this, however the scaling factor is hardcoded for now
#[derive(Debug)]
pub struct RequestCountryScalingFactorCommand;

#[derive(Debug)]
pub struct RequestBillPositionCommand {
    buffer: [u8; 2],
}
impl RequestBillPositionCommand {
    pub fn new(country_code: &str) -> Self {
        RequestBillPositionCommand {
            buffer: [country_code.as_bytes()[0], country_code.as_bytes()[1]],
        }
    }
}
impl Command for RequestBillPositionCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::RequestBillPosition
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            1..=255 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                1,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RouteBillCommand {
    buffer: [u8; 1],
}
impl RouteBillCommand {
    pub fn new(command: BillRouteCode) -> Self {
        RouteBillCommand {
            buffer: [command as u8],
        }
    }
}
impl Command for RouteBillCommand {
    type Response = Option<BillRoutingError>;

    fn header(&self) -> Header {
        Header::RouteBill
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(None),
            1 => match BillRoutingError::try_from(response_payload[0]) {
                Ok(error) => Ok(Some(error)),
                Err(_) => Ok(None),
            },
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct ModifyBillOperatingModeCommand {
    buffer: [u8; 1],
}
impl ModifyBillOperatingModeCommand {
    pub fn new(use_stacker: bool, use_escrow: bool) -> Self {
        let mut mask = 0u8;
        if use_stacker {
            mask += 1;
        }

        if use_escrow {
            mask += 2;
        }

        ModifyBillOperatingModeCommand { buffer: [mask] }
    }
}
impl Command for ModifyBillOperatingModeCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyBillOperatingMode
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

#[derive(Debug)]
pub struct RequestBillOperatingModeCommand;
impl Command for RequestBillOperatingModeCommand {
    type Response = (bool, bool); // (use_stacker, use_escrow)

    fn header(&self) -> Header {
        Header::RequestBillOperatingMode
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            1 => Ok((
                response_payload[0] & 0x01 != 0,
                response_payload[0] & 0x02 != 0,
            )),
            _ => Err(ParseResponseError::DataLengthMismatch(
                1,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct TestLampsCommand {
    buffer: [u8; 2],
}
impl TestLampsCommand {
    pub fn new(lamp: u8, command: LampControl) -> Self {
        TestLampsCommand {
            buffer: [lamp, command.into()],
        }
    }
}
impl Command for TestLampsCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::TestLamps
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestIndividualAcceptCounterCommand {
    buffer: [u8; 1],
}
impl RequestIndividualAcceptCounterCommand {
    pub fn new(bill_or_coin_type: u8) -> Self {
        RequestIndividualAcceptCounterCommand {
            buffer: [bill_or_coin_type],
        }
    }
}
impl Command for RequestIndividualAcceptCounterCommand {
    type Response = u32;

    fn header(&self) -> Header {
        Header::RequestIndividualAcceptCounter
    }

    fn data(&self) -> &[u8] {
        &self.buffer
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

#[derive(Debug)]
pub struct ReadOptoVoltagesCommand;
impl Command for ReadOptoVoltagesCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ReadOptoVoltages
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    // Device specific, look at your device manual
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            1..=2 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                1,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct PerformStackerCycleCommand;
impl Command for PerformStackerCycleCommand {
    type Response = Option<StackerCycleError>;

    fn header(&self) -> Header {
        Header::PerformStackerCycle
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    // Device specific, no validation/parsing is provided.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            1 => StackerCycleError::try_from(response_payload[0])
                .map(Some)
                .map_err(|_| ParseResponseError::ParseError("Invalid stacker cycle error")),
            _ => Ok(None),
        }
    }
}

#[derive(Debug)]
pub struct OperateBiDirectionalMotorsCommand {
    buffer: [u8; 3],
}
impl OperateBiDirectionalMotorsCommand {
    pub fn new(motors: u8, directions: u8, speed: u8) -> Self {
        OperateBiDirectionalMotorsCommand {
            buffer: [motors, directions, speed],
        }
    }
}
impl Command for OperateBiDirectionalMotorsCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::OperateBiDirectionalMotors
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestCurrencyRevisionCommand {
    buffer: [u8; 2],
    has_country_code: bool,
}
impl RequestCurrencyRevisionCommand {
    pub fn new() -> Self {
        RequestCurrencyRevisionCommand {
            buffer: [0u8, 0u8],
            has_country_code: false,
        }
    }

    pub fn build_with_country(country_code: &str) -> Result<Self, ()> {
        let bytes = country_code.as_bytes();
        if bytes.len() != 2 {
            return Err(());
        }

        Ok(RequestCurrencyRevisionCommand {
            buffer: [bytes[0], bytes[1]],
            has_country_code: true,
        })
    }
}

impl Default for RequestCurrencyRevisionCommand {
    fn default() -> Self {
        Self::new()
    }
}
impl Command for RequestCurrencyRevisionCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::RequestCurrencyRevision
    }

    fn data(&self) -> &[u8] {
        if self.has_country_code {
            &self.buffer[..]
        } else {
            &[]
        }
    }

    // Returns ascii string
    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct UploadBillTablesCommand {
    buffer: [u8; 130], // block + line + 128 data
    data_len: u8,
}
impl UploadBillTablesCommand {
    pub fn new(block: u8, line: u8, data: &[u8]) -> Result<Self, ()> {
        const MAX_PAYLOAD_SIZE: usize = 128;
        const COMMAND_BUFFER_SIZE: usize = 130;

        if data.len() > MAX_PAYLOAD_SIZE {
            return Err(());
        }

        let mut buffer = [0u8; COMMAND_BUFFER_SIZE];
        buffer[0] = block;
        buffer[1] = line;

        let payload_start = 2;
        buffer[payload_start..payload_start + data.len()].copy_from_slice(data);

        Ok(UploadBillTablesCommand {
            buffer,
            data_len: (2 + data.len()) as u8, // header + payload length
        })
    }
}
impl Command for UploadBillTablesCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::UploadBillTables
    }

    fn data(&self) -> &[u8] {
        &self.buffer[..self.data_len as usize]
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

#[derive(Debug)]
pub struct BeginBillTableUpgradeCommand;
impl Command for BeginBillTableUpgradeCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::BeginBillTableUpgrade
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct FinishBillTableUpgradeCommand;
impl Command for FinishBillTableUpgradeCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::FinishBillTableUpgrade
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestFirmwareUpgradeCapability {
    buffer: [u8; 1],
    has_module_identifier: bool,
}
impl RequestFirmwareUpgradeCapability {
    pub fn new() -> Self {
        RequestFirmwareUpgradeCapability {
            buffer: [0],
            has_module_identifier: false,
        }
    }

    pub fn new_with_module_identifier(module_identifier: u8) -> Self {
        RequestFirmwareUpgradeCapability {
            buffer: [module_identifier],
            has_module_identifier: true,
        }
    }
}

impl Default for RequestFirmwareUpgradeCapability {
    fn default() -> Self {
        Self::new()
    }
}
impl Command for RequestFirmwareUpgradeCapability {
    type Response = FirmwareStorageType;

    fn header(&self) -> Header {
        Header::RequestFirmwareUpgradeCapability
    }

    fn data(&self) -> &[u8] {
        if self.has_module_identifier {
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
            1 => FirmwareStorageType::try_from(response_payload[0])
                .map_err(|_| ParseResponseError::ParseError("Invalid firmware storage type")),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct UploadFirmwareCommand {
    buffer: [u8; 130], // block + line + 128 data
    data_len: u8,
}
impl UploadFirmwareCommand {
    pub fn new(block: u8, line: u8, data: &[u8]) -> Result<Self, ()> {
        const MAX_PAYLOAD_SIZE: usize = 128;
        const COMMAND_BUFFER_SIZE: usize = 130;

        if data.len() > MAX_PAYLOAD_SIZE {
            return Err(());
        }

        let mut buffer = [0u8; COMMAND_BUFFER_SIZE];
        buffer[0] = block;
        buffer[1] = line;

        let payload_start = 2;
        buffer[payload_start..payload_start + data.len()].copy_from_slice(data);

        Ok(UploadFirmwareCommand {
            buffer,
            data_len: (2 + data.len()) as u8, // header + payload length
        })
    }
}
impl Command for UploadFirmwareCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::UploadBillTables
    }

    fn data(&self) -> &[u8] {
        &self.buffer[..self.data_len as usize]
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

#[derive(Debug)]
pub struct BeginFirmwareUpgradeCommand {
    buffer: [u8; 1],
    has_module_identifier: bool,
}
impl BeginFirmwareUpgradeCommand {
    pub fn new() -> Self {
        BeginFirmwareUpgradeCommand {
            buffer: [0],
            has_module_identifier: false,
        }
    }

    pub fn new_with_module_identifier(module_identifier: u8) -> Self {
        BeginFirmwareUpgradeCommand {
            buffer: [module_identifier],
            has_module_identifier: true,
        }
    }
}

impl Default for BeginFirmwareUpgradeCommand {
    fn default() -> Self {
        Self::new()
    }
}
impl Command for BeginFirmwareUpgradeCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::BeginFirmwareUpgrade
    }

    fn data(&self) -> &[u8] {
        if self.has_module_identifier {
            &self.buffer[..]
        } else {
            &[]
        }
    }

    fn parse_response(&self, payload: &[u8]) -> Result<Self::Response, ParseResponseError> {
        match payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(0, payload.len())),
        }
    }
}

#[derive(Debug)]
pub struct FinishFirmwareUpgradeCommand;
impl Command for FinishFirmwareUpgradeCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::FinishFirmwareUpgrade
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct SetAcceptLimitCommand {
    buffer: [u8; 1],
}
impl SetAcceptLimitCommand {
    pub fn new(limit: u8) -> Self {
        SetAcceptLimitCommand { buffer: [limit] }
    }
}
impl Command for SetAcceptLimitCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::SetAcceptLimit
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct DispenseHopperValueCommand {
    buffer: [u8; 10],
}
impl DispenseHopperValueCommand {
    pub fn new(coin_value: u16) -> Self {
        DispenseHopperValueCommand {
            buffer: [
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                // Value
                (coin_value & 0xFF) as u8,
                ((coin_value >> 8) & 0xFF) as u8,
            ],
        }
    }

    pub fn new_with_security_code(security_code: [u8; 8], coin_value: u16) -> Self {
        DispenseHopperValueCommand {
            buffer: [
                security_code[0],
                security_code[1],
                security_code[2],
                security_code[3],
                security_code[4],
                security_code[5],
                security_code[6],
                security_code[7],
                // Value
                (coin_value & 0xFF) as u8,
                ((coin_value >> 8) & 0xFF) as u8,
            ],
        }
    }
}
impl Command for DispenseHopperValueCommand {
    type Response = Option<u8>;

    fn header(&self) -> Header {
        Header::DispenseHopperValue
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(None),
            1 => Ok(Some(response_payload[0])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestHopperPollingValueCommand;
impl Command for RequestHopperPollingValueCommand {
    type Response = HopperDispenseValueStatus;

    fn header(&self) -> Header {
        Header::RequestHopperPollingValue
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            7 => Ok(HopperDispenseValueStatus::from([
                response_payload[0],
                response_payload[1],
                response_payload[2],
                response_payload[3],
                response_payload[4],
                response_payload[5],
                response_payload[6],
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                7,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct EmergencyStopValueCommand;
impl Command for EmergencyStopValueCommand {
    type Response = u16;

    fn header(&self) -> Header {
        Header::EmergencyStopValue
    }

    fn data(&self) -> &[u8] {
        &[]
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

#[derive(Debug)]
pub struct RequestHopperCoinValueCommand {
    buffer: [u8; 1],
}
impl RequestHopperCoinValueCommand {
    pub fn new(coin_type: u8) -> Self {
        RequestHopperCoinValueCommand {
            buffer: [coin_type],
        }
    }
}
impl Command for RequestHopperCoinValueCommand {
    type Response = (CurrencyToken, u16); // Currency token, coin value

    fn header(&self) -> Header {
        Header::RequestHopperCoinValue
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            8 => {
                let coin_str = core::str::from_utf8(&response_payload[0..=6])
                    .map_err(|_| ParseResponseError::ParseError("Invalid UTF-8 in coin string"))?;
                let token = CurrencyToken::build(coin_str).map_err(|err| match err {
                    CurrencyTokenError::InvalidFormat => {
                        ParseResponseError::ParseError("invalid coin string format")
                    }
                    CurrencyTokenError::ValueStringTooSmall => ParseResponseError::BufferTooSmall,
                    CurrencyTokenError::CoinNotSupportedByDevice => {
                        ParseResponseError::ParseError("not supported by device")
                    }
                })?;
                let value = u16::from_le_bytes([response_payload[6], response_payload[7]]);
                Ok((token, value))
            }
            _ => Err(ParseResponseError::DataLengthMismatch(
                8,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestIndexedHopperDispenseCountCommand {
    buffer: [u8; 1],
}
impl RequestIndexedHopperDispenseCountCommand {
    pub fn new(coin_type: u8) -> Self {
        RequestIndexedHopperDispenseCountCommand {
            buffer: [coin_type],
        }
    }
}
impl Command for RequestIndexedHopperDispenseCountCommand {
    type Response = u32; // Dispense count

    fn header(&self) -> Header {
        Header::RequestIndexedHopperDispenseCount
    }

    fn data(&self) -> &[u8] {
        &self.buffer
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
                0,
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                3,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct ReadBarcodeDataCommand;
impl Command for ReadBarcodeDataCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ReadBarCodeData
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// ASCII or empty
    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct RequestMoneyInCommand;
impl Command for RequestMoneyInCommand {
    type Response = u32;

    fn header(&self) -> Header {
        Header::RequestMoneyIn
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            4 => Ok(u32::from_le_bytes([
                response_payload[0],
                response_payload[1],
                response_payload[2],
                response_payload[3],
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                4,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestMoneyOutCommand;
impl Command for RequestMoneyOutCommand {
    type Response = u32;

    fn header(&self) -> Header {
        Header::RequestMoneyOut
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            4 => Ok(u32::from_le_bytes([
                response_payload[0],
                response_payload[1],
                response_payload[2],
                response_payload[3],
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                4,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct ClearMoneyCountersCommand;
impl Command for ClearMoneyCountersCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ClearMoneyCounters
    }

    fn data(&self) -> &[u8] {
        &[]
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

#[derive(Debug)]
pub struct PayMoneyOutCommand {
    buffer: [u8; 4],
}
impl PayMoneyOutCommand {
    pub fn new(amount: u32) -> Self {
        PayMoneyOutCommand {
            buffer: amount.to_le_bytes(),
        }
    }
}
impl Command for PayMoneyOutCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::PayMoneyOut
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

#[derive(Debug)]
pub struct VerifyMoneyOutCommand;
impl Command for VerifyMoneyOutCommand {
    type Response = ChangerPollResult;

    fn header(&self) -> Header {
        Header::VerifyMoneyOut
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            9 => ChangerPollResult::try_from(response_payload)
                .map_err(|_| ParseResponseError::ParseError("Invalid ChangerPollResult format")),
            _ => Err(ParseResponseError::DataLengthMismatch(
                9,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestActivityRegisterCommand;
impl Command for RequestActivityRegisterCommand {
    type Response = heapless::Vec<ChangerFlags, 13>;

    fn header(&self) -> Header {
        Header::RequestActivityRegister
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            2 => Ok(parse_changer_flags_heapless(response_payload)),
            _ => Err(ParseResponseError::DataLengthMismatch(
                2,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestErrorStatusCommand;
impl Command for RequestErrorStatusCommand {
    type Response = (ChangerDevice, ChangerError);

    fn header(&self) -> Header {
        Header::RequestErrorStatus
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            2 => Ok((
                ChangerDevice::from(response_payload[0]),
                ChangerError::from(response_payload[1]),
            )),
            _ => Err(ParseResponseError::DataLengthMismatch(
                2,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct PurgeHopperCommand {
    buffer: [u8; 2],
}
impl PurgeHopperCommand {
    // TODO: WH hopper can use the purge hopper command too, from what i experimented they use data
    // [0], but maybe [0,0]/[255,0] works too
    pub fn new(hopper_number: u8, count: u8) -> Self {
        PurgeHopperCommand {
            buffer: [hopper_number, count],
        }
    }
}
impl Command for PurgeHopperCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::PurgeHopper
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct ModifyHopperBalanceCommand {
    buffer: [u8; 3],
}
impl ModifyHopperBalanceCommand {
    pub fn new(hopper_number: u8, balance: u16) -> Self {
        ModifyHopperBalanceCommand {
            buffer: [
                hopper_number,
                (balance & 0xFF) as u8,
                ((balance >> 8) & 0xFF) as u8,
            ],
        }
    }
}
impl Command for ModifyHopperBalanceCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyHopperBalance
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestHopperBalanceCommand {
    buffer: [u8; 1],
}
impl RequestHopperBalanceCommand {
    pub fn new(hopper_number: u8) -> Self {
        RequestHopperBalanceCommand {
            buffer: [hopper_number],
        }
    }
}
impl Command for RequestHopperBalanceCommand {
    type Response = (CurrencyToken, u16); // Currency token, balance

    fn header(&self) -> Header {
        Header::RequestHopperBalance
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            8 => {
                let coin_str = core::str::from_utf8(&response_payload[0..6])
                    .map_err(|_| ParseResponseError::ParseError("Invalid UTF-8 in coin string"))?;
                let token = CurrencyToken::build(coin_str).map_err(|err| match err {
                    CurrencyTokenError::InvalidFormat => {
                        ParseResponseError::ParseError("invalid coin string format")
                    }
                    CurrencyTokenError::ValueStringTooSmall => ParseResponseError::BufferTooSmall,
                    CurrencyTokenError::CoinNotSupportedByDevice => {
                        ParseResponseError::ParseError("not supported by device")
                    }
                })?;
                let count = u16::from_le_bytes([response_payload[6], response_payload[7]]);

                Ok((token, count))
            }
            _ => Err(ParseResponseError::DataLengthMismatch(
                8,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct ModifyCashBoxValueCommand {
    buffer: [u8; 4],
}
impl ModifyCashBoxValueCommand {
    pub fn new(value: u32) -> Self {
        ModifyCashBoxValueCommand {
            buffer: value.to_le_bytes(),
        }
    }
}
impl Command for ModifyCashBoxValueCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyCashBoxValue
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

#[derive(Debug)]
pub struct RequestCashBoxValueCommand;
impl Command for RequestCashBoxValueCommand {
    type Response = u32;

    fn header(&self) -> Header {
        Header::RequestCashBoxValue
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            4 => Ok(u32::from_le_bytes([
                response_payload[0],
                response_payload[1],
                response_payload[2],
                response_payload[3],
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                4,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct ModifyRtcCommand {
    buffer: [u8; 4],
}
impl ModifyRtcCommand {
    pub fn new(unix_epoch_seconds: u32) -> Self {
        ModifyRtcCommand {
            buffer: unix_epoch_seconds.to_le_bytes(),
        }
    }
}
impl Command for ModifyRtcCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ModifyRealTimeClock
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestRtcCommand;
impl Command for RequestRtcCommand {
    type Response = u32; // Unix epoch seconds

    fn header(&self) -> Header {
        Header::RequestRealTimeClock
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            4 => Ok(u32::from_le_bytes([
                response_payload[0],
                response_payload[1],
                response_payload[2],
                response_payload[3],
            ])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                4,
                response_payload.len(),
            )),
        }
    }
}

// TODO: implement when encryption is supported
#[derive(Debug)]
pub struct ReadEncryptedEventsCommand;
#[derive(Debug)]
pub struct RequestEncryptedHopperStatusCommand;
#[derive(Debug)]
pub struct RequestEncryptedMonetaryIdCommand;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DivertMode {
    AcceptCoins = 0,
    ReturnCoins = 1,
}
#[derive(Debug)]
pub struct OperateEscrowCommand {
    buffer: [u8; 1],
}
impl OperateEscrowCommand {
    pub fn new(divert_mode: DivertMode) -> Self {
        OperateEscrowCommand {
            buffer: [divert_mode as u8],
        }
    }
}
impl Command for OperateEscrowCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::OperateEscrow
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        // No response expected, just an empty payload
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestEscrowStatusCommand;
impl Command for RequestEscrowStatusCommand {
    type Response = (EscrowOperatingStatus, EscrowLevelStatus, EscrowFaultCode);

    fn header(&self) -> Header {
        Header::RequestEscrowStatus
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            3 => {
                let operating_status = EscrowOperatingStatus::try_from(response_payload[0])
                    .map_err(|_| ParseResponseError::ParseError("Invalid EscrowOperatingStatus"))?;
                let level_status = EscrowLevelStatus::try_from(response_payload[1])
                    .map_err(|_| ParseResponseError::ParseError("Invalid EscrowLevelStatus"))?;
                let fault_code = EscrowFaultCode::from(response_payload[2]);

                Ok((operating_status, level_status, fault_code))
            }
            _ => Err(ParseResponseError::DataLengthMismatch(
                3,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestServiceStatusCommand {
    buffer: [u8; 1],
}
impl RequestServiceStatusCommand {
    pub fn new_report() -> Self {
        RequestServiceStatusCommand { buffer: [0] }
    }

    pub fn new_clear_report() -> Self {
        RequestServiceStatusCommand { buffer: [1] }
    }
}
impl Command for RequestServiceStatusCommand {
    type Response = Option<EscrowServiceStatus>;

    fn header(&self) -> Header {
        Header::RequestServiceStatus
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(None),
            1 => {
                let status = EscrowServiceStatus::try_from(response_payload[0])
                    .map_err(|_| ParseResponseError::ParseError("Invalid EscrowServiceStatus"))?;

                Ok(Some(status))
            }
            _ => Err(ParseResponseError::DataLengthMismatch(
                1,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct ClearCommsStatusVariablesCommand;
impl Command for ClearCommsStatusVariablesCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ClearCommsStatusVariable
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RequestCommsStatusVariablesCommand;
impl Command for RequestCommsStatusVariablesCommand {
    type Response = (u8, u8, u8);

    fn header(&self) -> Header {
        Header::RequestCommsStatusVariables
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            3 => Ok((
                response_payload[0],
                response_payload[1],
                response_payload[2],
            )),
            _ => Err(ParseResponseError::DataLengthMismatch(
                3,
                response_payload.len(),
            )),
        }
    }
}
