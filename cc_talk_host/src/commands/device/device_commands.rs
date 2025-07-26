use core::time::Duration;

use cc_talk_core::{Fault, Header, cc_talk::FaultCode};

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
pub struct ModifyInhibitStatusCommand {
    buffer: [u8; 4],
}
impl ModifyInhibitStatusCommand {
    /// Creates a new ModifyInhibitStatusCommand with the given buffer.
    ///
    /// Bit 0 at index 0 is the first inhibit flag, bit 1 at index 0 is the second inhibit flag,
    /// etc.
    pub fn new(buffer: [u8; 4]) -> Self {
        ModifyInhibitStatusCommand { buffer }
    }
}
impl Command for ModifyInhibitStatusCommand {
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

pub struct RequestInhibitStatusCommand;
impl Command for RequestInhibitStatusCommand {
    type Response = [u8; 4];

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
            4 => Ok(response_payload.try_into().unwrap()),
            // TODO: Add logging for unexpected lengths
            5..=usize::MAX => Ok(response_payload[0..4].try_into().unwrap()),
            _ => Err(ParseResponseError::DataLengthMismatch(
                4,
                response_payload.len(),
            )),
        }
    }
}
