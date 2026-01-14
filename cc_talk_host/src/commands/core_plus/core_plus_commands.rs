use cc_talk_core::cc_talk::{DataStorage, Header, RTBYDate, SerialCode};

use super::{
    super::command::{BelongsTo, Command, ParseResponseError},
    CorePlusCommandSet,
};

#[derive(Debug)]
pub struct RequestSerialNumberCommand;
impl Command for RequestSerialNumberCommand {
    type Response = SerialCode;

    fn header(&self) -> Header {
        Header::RequestSerialNumber
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Parses the response payload as a serial code.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.len() != 3 {
            return Err(ParseResponseError::DataLengthMismatch(
                3,
                response_payload.len(),
            ));
        }
        Ok(SerialCode::new(
            response_payload[2],
            response_payload[1],
            response_payload[0], // Byte 0 is LSB
        ))
    }
}
impl BelongsTo<CorePlusCommandSet> for RequestSerialNumberCommand {}

#[derive(Debug)]
pub struct RequestSoftwareRevisionCommand;
impl Command for RequestSoftwareRevisionCommand {
    type Response = heapless::String<64>;

    fn header(&self) -> Header {
        Header::RequestSoftwareRevision
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// The answer to this command is a string, currently the `parse_response` will only check if
    /// the response is valid UTF-8.
    ///
    /// The cast to a valid data type depending on the enviornment (std, heapless, etc.) is left to
    /// the user.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if !response_payload.iter().all(|&b| b.is_ascii()) {
            return Err(ParseResponseError::ParseError("Invalid ASCII response"));
        }
        Ok(heapless::String::from_iter(
            response_payload.iter().map(|b| *b as char),
        ))
    }
}
impl BelongsTo<CorePlusCommandSet> for RequestSoftwareRevisionCommand {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadDHPublicKeyMode {
    RequestStatus,
    RequestPublicKey,
}
#[deprecated(note = "encryption is not supported yet, so this command is not implemented")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadDHPublicKeyCommand {
    buffer: [u8; 1],
}
impl ReadDHPublicKeyCommand {
    /// Creates a new command to read the Diffie-Hellman public key.
    ///
    /// `mode` specifies whether to request the status or the public key.
    pub fn new(mode: ReadDHPublicKeyMode) -> Self {
        Self {
            buffer: [mode as u8],
        }
    }
}
impl Command for ReadDHPublicKeyCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ReadDHPubKey
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    /// Parses the response payload as a 32-byte public key.
    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        todo!("encryption is not supported yet, so this command is not implemented")
    }
}
impl BelongsTo<CorePlusCommandSet> for ReadDHPublicKeyCommand {}

#[derive(Debug)]
pub struct SendDHPublicKeyCommand<'a> {
    key: &'a [u8],
}
impl<'a> SendDHPublicKeyCommand<'a> {
    /// Creates a new command to send the Diffie-Hellman public key.
    ///
    /// `key` specifies the public key to send.
    fn new(key: &'a [u8]) -> Self {
        Self { key }
    }
}
impl Command for SendDHPublicKeyCommand<'_> {
    type Response = ();

    fn header(&self) -> Header {
        Header::SendDHPubKey
    }

    fn data(&self) -> &[u8] {
        self.key
    }

    /// Parses the response payload, which is expected to be empty.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if !response_payload.is_empty() {
            return Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            ));
        }
        Ok(())
    }
}
impl BelongsTo<CorePlusCommandSet> for SendDHPublicKeyCommand<'_> {}

#[deprecated(note = "encryption is not supported yet, so this command is not implemented")]
#[derive(Debug)]
pub struct RequestEncryptedProductIdCommand;
impl BelongsTo<CorePlusCommandSet> for RequestEncryptedProductIdCommand {}

#[deprecated(note = "encryption is not supported yet, so this command is not implemented")]
#[derive(Debug)]
pub struct RequestACMIEncryptedDataCommand;
impl BelongsTo<CorePlusCommandSet> for RequestACMIEncryptedDataCommand {}

#[derive(Debug)]
pub struct RequestDataStorageAvailabilityCommand;
impl Command for RequestDataStorageAvailabilityCommand {
    type Response = DataStorage;

    fn header(&self) -> Header {
        Header::RequestDataStorageAvailability
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Parses the data storage availability response.
    ///
    /// If read or write is not available, the corresponding blocks and bytes per block will be set
    /// to 0.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.len() != 5 {
            return Err(ParseResponseError::DataLengthMismatch(
                5,
                response_payload.len(),
            ));
        }
        Ok(DataStorage::from([
            response_payload[0],
            response_payload[1],
            response_payload[2],
            response_payload[3],
            response_payload[4],
        ]))
    }
}
impl BelongsTo<CorePlusCommandSet> for RequestDataStorageAvailabilityCommand {}

#[deprecated(note = "encryption is not supported yet, so this command is not implemented")]
#[derive(Debug)]
pub struct ACMIUnencryptedProductIdCommand;
impl BelongsTo<CorePlusCommandSet> for ACMIUnencryptedProductIdCommand {}

#[derive(Debug)]
pub struct CalculateRomChecksumCommand;
impl Command for CalculateRomChecksumCommand {
    type Response = u32;

    fn header(&self) -> Header {
        Header::CalculateROMChecksum
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Parses the response payload as a 4-byte checksum, byte 0 is LSB.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.len() != 4 {
            return Err(ParseResponseError::DataLengthMismatch(
                4,
                response_payload.len(),
            ));
        }
        Ok(u32::from_le_bytes([
            response_payload[0],
            response_payload[1],
            response_payload[2],
            response_payload[3],
        ]))
    }
}
impl BelongsTo<CorePlusCommandSet> for CalculateRomChecksumCommand {}

#[derive(Debug)]
pub struct RequestCreationDateCommand;
impl Command for RequestCreationDateCommand {
    type Response = RTBYDate;

    fn header(&self) -> Header {
        Header::RequestCreationDate
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Parses the response payload, which is expected to be empty.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        parse_rtby_from_payload(response_payload)
    }
}
impl BelongsTo<CorePlusCommandSet> for RequestCreationDateCommand {}

#[derive(Debug)]
pub struct RequestLastModificationDateCommand;
impl Command for RequestLastModificationDateCommand {
    type Response = RTBYDate;

    fn header(&self) -> Header {
        Header::RequestLastModificationDate
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Parses the response payload, which is expected to be empty.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        parse_rtby_from_payload(response_payload)
    }
}
impl BelongsTo<CorePlusCommandSet> for RequestLastModificationDateCommand {}

fn parse_rtby_from_payload(response_payload: &[u8]) -> Result<RTBYDate, ParseResponseError> {
    if response_payload.len() != 2 {
        return Err(ParseResponseError::DataLengthMismatch(
            2,
            response_payload.len(),
        ));
    }
    let date_code = u16::from_le_bytes([response_payload[0], response_payload[1]]);
    Ok(RTBYDate::new(date_code))
}

#[derive(Debug)]
pub struct RequestBaseYearCommand;
impl Command for RequestBaseYearCommand {
    type Response = u16;

    fn header(&self) -> Header {
        Header::RequestBaseYear
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Parses the response payload as a u16 value, which represents the base year.
    ///
    /// The original response is in ASCII.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.len() != 4 {
            return Err(ParseResponseError::DataLengthMismatch(
                4,
                response_payload.len(),
            ));
        }

        if response_payload.iter().any(|&byte| !byte.is_ascii_digit()) {
            return Err(ParseResponseError::ParseError("Invalid base year response"));
        }

        response_payload
            .iter()
            .enumerate()
            .try_fold(0u16, |acc, (i, &byte)| {
                let numeric_value = (byte - b'0') as u16;
                Ok(acc + (numeric_value * 10u16.pow(3 - i as u32)))
            })
    }
}
impl BelongsTo<CorePlusCommandSet> for RequestBaseYearCommand {}

#[derive(Debug)]
pub struct RequestAddressModeCommand;
impl Command for RequestAddressModeCommand {
    type Response = u8;

    fn header(&self) -> Header {
        Header::RequestAddressMode
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Parses the response payload as a single byte representing the address mode.
    /// Refer to the header documentation for details on the address modes.
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
impl BelongsTo<CorePlusCommandSet> for RequestAddressModeCommand {}

#[deprecated(note = "encryption is not supported yet, so this command is not implemented")]
#[derive(Debug)]
pub struct SwitchEncryptionCodeCommand;
impl BelongsTo<CorePlusCommandSet> for SwitchEncryptionCodeCommand {}

#[deprecated(note = "encryption is not supported yet, so this command is not implemented")]
#[derive(Debug)]
pub struct StoreEncryptionCodeCommand;
impl BelongsTo<CorePlusCommandSet> for StoreEncryptionCodeCommand {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UsbInfo {
    pub vendor_id: u16,
    pub product_id: u16,
}
#[derive(Debug)]
pub struct RequestUsbIdCommand;
impl Command for RequestUsbIdCommand {
    type Response = UsbInfo;

    fn header(&self) -> Header {
        Header::RequestUsbId
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Parses the response payload as a USB vendor and product ID.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.len() != 4 {
            return Err(ParseResponseError::DataLengthMismatch(
                4,
                response_payload.len(),
            ));
        }
        Ok(UsbInfo {
            vendor_id: u16::from_le_bytes([response_payload[0], response_payload[1]]),
            product_id: u16::from_le_bytes([response_payload[2], response_payload[3]]),
        })
    }
}
impl BelongsTo<CorePlusCommandSet> for RequestUsbIdCommand {}

/// Represents the status of a baud rate switch command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaudRateSwitchStatus {
    ShouldBeAckOrNack,
    BaudRateCode(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaudRateOperation {
    RequestBaudRateInUse = 0,
    SwitchBaudRateToNewValue = 1,
    RequestMaximumBaudRateSupported = 2,
    RequestSupportForNewBaudRate = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaudRateCode {
    Rate4800 = 0,
    Rate9600 = 1,
    Rate19200 = 2,
    Rate38400 = 3,
    Rate57600 = 4,
    Rate115200 = 5,
    Rate230400 = 6,
    Rate460800 = 7,
    Rate921600 = 8,
    Rate1000000 = 10,
    Rate1843200 = 18,
    Rate2000000 = 20,
    Rate3000000 = 30,
}

/// This command returns different status depending on the flow
/// Please read the documentation of the command for more details.
/// As switching baud rate is a quite involved process with pitfalls.
#[derive(Debug)]
pub struct SwitchBaudRateCommand {
    buffer: [u8; 2],
}
impl SwitchBaudRateCommand {
    /// Creates a new command to switch the baud rate.
    ///
    /// `operation` specifies the operation to perform, such as requesting the current baud rate,
    /// switching to a new baud rate, or checking support for a new baud rate.
    ///
    /// `code` specifies the baud rate code to switch to or check support for.
    pub fn new(operation: BaudRateOperation, code: BaudRateCode) -> Self {
        Self {
            buffer: [operation as u8, code as u8],
        }
    }
}
impl Command for SwitchBaudRateCommand {
    type Response = BaudRateSwitchStatus;

    fn header(&self) -> Header {
        Header::SwitchBaudRate
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    /// Parses the response payload, which is expected to be empty.
    /// If the response is [BaudRateSwitchStatus::ShouldBeAckOrNack] please verify that the command
    /// header is NACK or ACK.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        match response_payload.len() {
            0 => Ok(BaudRateSwitchStatus::ShouldBeAckOrNack),
            1 => Ok(BaudRateSwitchStatus::BaudRateCode(response_payload[0])),
            _ => Err(ParseResponseError::DataLengthMismatch(
                0, // Technically could be 1 as well
                response_payload.len(),
            )),
        }
    }
}
impl BelongsTo<CorePlusCommandSet> for SwitchBaudRateCommand {}

#[deprecated(note = "encryption is not supported yet, so this command is not implemented")]
#[derive(Debug)]
pub struct SwitchEncryptionKeyCommand;
impl BelongsTo<CorePlusCommandSet> for SwitchEncryptionKeyCommand {}

#[derive(Debug, Eq, PartialEq)]
pub struct DataStreamCommand<'a> {
    buffer: &'a [u8],
}
impl<'a> DataStreamCommand<'a> {
    /// Creates a new data stream command with the specified data.
    pub fn new(data: &'a [u8]) -> Self {
        Self { buffer: data }
    }
}
impl Command for DataStreamCommand<'_> {
    type Response = ();

    fn header(&self) -> Header {
        Header::DataStream
    }

    fn data(&self) -> &[u8] {
        self.buffer
    }

    /// Does nothing as this is used for custom data streams.
    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        Ok(())
    }
}
impl BelongsTo<CorePlusCommandSet> for DataStreamCommand<'_> {}

#[derive(Debug)]
pub struct BusyCommand;
impl Command for BusyCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::Busy
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Busy is a response, so this should really never be called.
    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        Ok(())
    }
}
impl BelongsTo<CorePlusCommandSet> for BusyCommand {}

#[derive(Debug)]
pub struct NackCommand;
impl Command for NackCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::NACK
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// NACK is a response, so this should really never be called.
    fn parse_response(&self, _: &[u8]) -> Result<Self::Response, ParseResponseError> {
        Ok(())
    }
}
impl BelongsTo<CorePlusCommandSet> for NackCommand {}

#[derive(Debug)]
pub struct RequestCommsRevisionCommand;
impl Command for RequestCommsRevisionCommand {
    type Response = (u8, u8, u8);

    fn header(&self) -> Header {
        Header::RequestCommsRevision
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Parses the response payload as a tuple of three bytes representing the communication
    /// revision.
    ///
    /// (release, major, minor)
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if response_payload.len() != 3 {
            return Err(ParseResponseError::DataLengthMismatch(
                3,
                response_payload.len(),
            ));
        }
        Ok((
            response_payload[0],
            response_payload[1],
            response_payload[2],
        ))
    }
}
impl BelongsTo<CorePlusCommandSet> for RequestCommsRevisionCommand {}

#[derive(Debug)]
pub struct ResetDeviceCommand;
impl Command for ResetDeviceCommand {
    type Response = ();

    fn header(&self) -> Header {
        Header::ResetDevice
    }

    fn data(&self) -> &[u8] {
        &[]
    }

    /// Parses the response payload, which is expected to be empty.
    fn parse_response(
        &self,
        response_payload: &[u8],
    ) -> Result<Self::Response, ParseResponseError> {
        if !response_payload.is_empty() {
            return Err(ParseResponseError::DataLengthMismatch(
                0,
                response_payload.len(),
            ));
        }
        Ok(())
    }
}
impl BelongsTo<CorePlusCommandSet> for ResetDeviceCommand {}

#[cfg(test)]
mod test {
    use cc_talk_core::cc_talk::MemoryType;
    use heapless::format;

    use super::*;

    #[test]
    fn request_valid_serial_number() {
        use super::RequestSerialNumberCommand;
        use cc_talk_core::cc_talk::SerialCode;

        let command = RequestSerialNumberCommand;
        let response = command.parse_response(&[0, 0, 1]).unwrap();
        assert_eq!(response, SerialCode::new(1, 0, 0));
        assert_eq!(response.as_number(), 65536);
    }

    #[test]
    fn request_software_revision() {
        let command = RequestSoftwareRevisionCommand;
        let response = command.parse_response(b"v1.0.0");
        assert!(response.is_ok());
    }

    #[test]
    fn data_storage_availability() {
        let command = RequestDataStorageAvailabilityCommand;
        let response = command.parse_response(&[0, 1, 2, 3, 4]).unwrap();
        assert_eq!(
            response,
            DataStorage {
                memory_type: MemoryType::VolatileOnReset,
                read_blocks: 1,
                read_bytes_per_block: 2,
                write_blocks: 3,
                write_bytes_per_block: 4
            }
        );
    }

    #[test]
    fn data_storage_not_available() {
        let command = RequestDataStorageAvailabilityCommand;
        let response = command.parse_response(&[3, 255, 0, 50, 0]).unwrap();
        assert_eq!(
            response,
            DataStorage {
                memory_type: MemoryType::PermanentUnlimitedUse,
                read_blocks: 255,
                read_bytes_per_block: 0,
                write_blocks: 50,
                write_bytes_per_block: 0
            }
        );
    }

    #[test]
    fn calculate_rom_checksum() {
        let command = CalculateRomChecksumCommand;
        let response = command.parse_response(&[0, 1, 2, 3]).unwrap();
        assert_eq!(response, 0x03020100);
    }

    #[test]
    fn request_creation_date() {
        let command = RequestCreationDateCommand;
        assert_eq!(command.header(), Header::RequestCreationDate);
        assert!(command.parse_response(&[0, 1]).is_ok());
    }

    #[test]
    fn request_last_modification_date() {
        let command = RequestLastModificationDateCommand;
        assert_eq!(command.header(), Header::RequestLastModificationDate);
        assert!(command.parse_response(&[0, 1]).is_ok());
    }

    #[test]
    fn request_base_year() {
        let command = RequestBaseYearCommand;
        assert_eq!(command.header(), Header::RequestBaseYear);

        let base_year = "4269";
        let parse_result = command.parse_response(base_year.as_bytes());
        assert!(parse_result.is_ok());
    }

    #[test]
    fn request_base_year_works_with_all_base_year() {
        for i in 0..=9999 {
            let command = RequestBaseYearCommand;
            let base_year: heapless::String<4> = format!("{:04}", i).expect("should work");
            let parse_result = command.parse_response(base_year.as_bytes());
            assert!(parse_result.is_ok(), "Failed for base year: {}", i);
            assert_eq!(
                parse_result.unwrap(),
                i as u16,
                "Failed for base year: {}",
                i
            );
        }
    }

    #[test]
    fn character_invalidates_base_year() {
        let command = RequestBaseYearCommand;
        let invalid_base_year = "42a9";
        let parse_result = command.parse_response(invalid_base_year.as_bytes());
        assert!(
            parse_result.is_err(),
            "Expected error for invalid base year"
        );
    }

    #[test]
    fn switch_baud_rate_command() {
        let command = SwitchBaudRateCommand::new(
            BaudRateOperation::SwitchBaudRateToNewValue,
            BaudRateCode::Rate115200,
        );
        assert_eq!(command.header(), Header::SwitchBaudRate);
        assert_eq!(command.data(), &[1, 5]);
        let response = command.parse_response(&[1]);
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), BaudRateSwitchStatus::BaudRateCode(1));
    }

    #[test]
    fn test_data_stream() {
        let command = DataStreamCommand::new(&[1, 2, 3, 4]);
        assert_eq!(command.header(), Header::DataStream);
        assert_eq!(command.data(), &[1, 2, 3, 4]);

        let response = command.parse_response(&[]);
        assert!(response.is_ok());
    }
}
