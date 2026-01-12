#![allow(dead_code)]

pub mod payout;

/// Represents the possible errors when talking to a ccTalk device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DeviceError {
    #[error("timeout waiting for device response")]
    Timeout,
    #[error("checksum mismatch in response")]
    ChecksumMismatch,
    #[error("buffer overflow")]
    BufferOverflow,
    #[error("device sent NACK response")]
    Nack,
    #[error("malformed response from device")]
    MalformedResponse,
}
