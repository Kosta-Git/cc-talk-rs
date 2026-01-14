#![allow(dead_code)]

use cc_talk_core::cc_talk::Header;

/// Base command trait that all commands must implement.
pub trait Command {
    type Response;

    /// Command header.
    fn header(&self) -> Header;

    /// Command data payload.
    fn data(&self) -> &[u8];

    /// Parses the payload of the response.
    fn parse_response(&self, response_payload: &[u8])
    -> Result<Self::Response, ParseResponseError>;
}

/// Errors that can occur during command execution
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseResponseError {
    /// The response data length does not match the expected length.
    /// .0 is expected length, .1 is actual length.
    #[error("data length mismatch: expected {0} bytes, got {1} bytes")]
    DataLengthMismatch(usize, usize),
    #[error("parse error: {0}")]
    ParseError(&'static str),
    /// Buffer is too small to hold the response data.
    #[error("buffer too small to hold response data")]
    BufferTooSmall,
}
