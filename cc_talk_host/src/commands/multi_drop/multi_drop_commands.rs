use cc_talk_core::{Address, Header};

use super::{
    super::command::{BelongsTo, Command, ParseResponseError},
    MdbCommandSet,
};

/// Address poll is a MDCES command.
///
/// Your transport should be able to handle response with ~3ms space between packets.
/// And will receive as many response as there are devices connected to the bus up to 255 devices.
pub struct AddressPollCommand;
impl Command for AddressPollCommand {
    type Response = u8;

    fn header(&self) -> Header {
        Header::AddressPoll
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Returns the address of the device that responded to the poll.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.len() != 1 {
            return Err(ParseResponseError::DataLengthMismatch(
                1,
                response_payload.len(),
            ));
        }
        Ok(response_payload[0])
    }
}
impl BelongsTo<MdbCommandSet> for AddressPollCommand {}

/// Address clash is a MDCES command.
pub struct AddressClashCommand;
impl Command for AddressClashCommand {
    type Response = u8;

    fn header(&self) -> Header {
        Header::AddressClash
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Returns the address of the device that responded to the clash.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.len() != 1 {
            return Err(ParseResponseError::DataLengthMismatch(
                1,
                response_payload.len(),
            ));
        }
        Ok(response_payload[0])
    }
}
impl BelongsTo<MdbCommandSet> for AddressClashCommand {}

/// Address change is a MDCES command.
pub struct AddressChangeCommand {
    buffer: [u8; 1],
}
impl AddressChangeCommand {
    /// Creates a new address change command.
    /// If a [Address::SingleAndRange] is provided, the single address will be used.
    pub fn new(new_address: Address) -> Self {
        AddressChangeCommand {
            buffer: [match new_address {
                Address::SingleAndRange(addr, _) => addr,
                Address::Single(addr) => addr,
            }; 1],
        }
    }
}
impl Command for AddressChangeCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::AddressChange
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    /// Returns an ack
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.is_empty() {
            true => Ok(()),
            false => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}
impl BelongsTo<MdbCommandSet> for AddressChangeCommand {}

/// Address random is a MDCES command.
pub struct AddressRandomCommand;
impl Command for AddressRandomCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::AddressRandom
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Returns an ack
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.is_empty() {
            true => Ok(()),
            false => Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            )),
        }
    }
}
impl BelongsTo<MdbCommandSet> for AddressRandomCommand {}
