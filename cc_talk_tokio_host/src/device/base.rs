#![allow(dead_code, async_fn_in_trait)]

use cc_talk_core::cc_talk::{Category, Device, Manufacturer, Packet, PacketError, SerialCode};
use cc_talk_host::{
    command::{Command, ParseResponseError},
    core::core_commands::{
        RequestEquipementCategoryIdCommand, RequestManufacturerIdCommand,
        RequestProductCodeCommand, SimplePollCommand,
    },
    core_plus::core_plus_commands::{
        RequestSerialNumberCommand, RequestSoftwareRevisionCommand, ResetDeviceCommand,
    },
};
use thiserror::Error;
use tokio::sync::{mpsc::Sender, oneshot};
use tracing::instrument;

use crate::transport::tokio_transport::{TransportError, TransportMessage};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CommandError {
    #[error("Timeout")]
    Timeout,
    #[error("NACK")]
    Nack,
    #[error("Buffer overflow")]
    BufferOverflow,
    #[error("Packet creation error")]
    PacketCreationError,
    #[error("Socket write error")]
    SocketWriteError,
    #[error("Socket read error")]
    SocketReadError,
    #[error("Checksum error")]
    ChecksumError,
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
    #[error("Send error")]
    SendError,
    #[error("Receive error")]
    ReceiveError,
    #[error("Response data length mismatch (expected: {0}, actual: {1})")]
    DataLengthMismatch(u8, u8),
    #[error("invalid header: {0}")]
    InvalidHeader(u8),
    #[error("invalid packet")]
    InvalidPacket,
    #[error("Unable to parse response: {0}")]
    ParseError(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PollingError {
    #[error("background polling is already locked by another task")]
    AlreadyLeased,
}

impl From<TransportError> for CommandError {
    fn from(error: TransportError) -> Self {
        match error {
            TransportError::Timeout => CommandError::Timeout,
            TransportError::Nack => CommandError::Nack,
            TransportError::BufferOverflow => CommandError::BufferOverflow,
            TransportError::PacketCreationError => CommandError::PacketCreationError,
            TransportError::SocketWriteError => CommandError::SocketWriteError,
            TransportError::SocketReadError => CommandError::SocketReadError,
            TransportError::ChecksumError => CommandError::ChecksumError,
            TransportError::MaxRetriesExceeded => CommandError::MaxRetriesExceeded,
        }
    }
}

impl From<PacketError> for CommandError {
    fn from(error: PacketError) -> Self {
        match error {
            PacketError::DataLengthMismatch => CommandError::DataLengthMismatch(0, 0),
            PacketError::InvalidHeader(header) => CommandError::InvalidHeader(header),
            PacketError::InvalidPacket => CommandError::InvalidPacket,
            PacketError::OutOfBounds => CommandError::BufferOverflow,
        }
    }
}

impl From<ParseResponseError> for CommandError {
    fn from(error: ParseResponseError) -> Self {
        match error {
            ParseResponseError::DataLengthMismatch(expected, actual) => {
                CommandError::DataLengthMismatch(expected as u8, actual as u8)
            }
            ParseResponseError::ParseError(error) => CommandError::ParseError(error),
            ParseResponseError::BufferTooSmall => CommandError::BufferOverflow,
        }
    }
}

pub type DeviceResult<T> = Result<T, CommandError>;

pub trait DeviceCommon {
    fn get_device(&self) -> &Device;
    fn get_sender(&self) -> &Sender<TransportMessage>;

    #[instrument(name = "device_send_command", skip(self), level = "debug")]
    async fn send_command<C>(&self, command: C) -> Result<Packet<Vec<u8>>, CommandError>
    where
        C: Command + core::fmt::Debug,
    {
        let (tx, rx) = oneshot::channel();
        let message = TransportMessage::new(self.get_device(), command, tx);
        self.get_sender()
            .send(message)
            .await
            .map_err(|_| CommandError::SendError)?;

        let result = rx.await.map_err(|_| CommandError::ReceiveError)??;
        Ok(Packet::new(result))
    }

    async fn simple_poll(&self) -> Result<(), CommandError> {
        let response_packet = self.send_command(SimplePollCommand).await?;
        SimplePollCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|_| ())
    }

    async fn get_manufacturer_id(&self) -> Result<Manufacturer, CommandError> {
        let response_packet = self.send_command(RequestManufacturerIdCommand).await?;
        RequestManufacturerIdCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    async fn get_category(&self) -> Result<Category, CommandError> {
        let response_packet = self
            .send_command(RequestEquipementCategoryIdCommand)
            .await?;

        RequestEquipementCategoryIdCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    async fn get_product_code(&self) -> Result<String, CommandError> {
        let response_packet = self.send_command(RequestProductCodeCommand).await?;
        RequestProductCodeCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|s| s.to_string())
    }

    async fn get_serial_number(&self) -> Result<SerialCode, CommandError> {
        let response_packet = self.send_command(RequestSerialNumberCommand).await?;
        RequestSerialNumberCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
    }

    async fn get_software_revision(&self) -> Result<String, CommandError> {
        let response_packet = self.send_command(RequestSoftwareRevisionCommand).await?;
        RequestSoftwareRevisionCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|s| s.to_string())
    }

    async fn reset_device(&self) -> Result<(), CommandError> {
        let response_packet = self.send_command(ResetDeviceCommand).await?;
        ResetDeviceCommand
            .parse_response(response_packet.get_data()?)
            .map_err(CommandError::from)
            .map(|_| ())
    }
}
