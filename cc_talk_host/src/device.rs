#![allow(dead_code)]

pub mod payout;

/// Represents the possible errors when talking to a ccTalk device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceError {
    Timeout,
    ChecksumMismatch,
    BufferOverflow,
    Nack,
    MalformedResponse,
}
