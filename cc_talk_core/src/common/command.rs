use crate::cc_talk::Header;

/// Base command trait that all commands must implement.
pub trait Command {
    type Response;

    /// Command header.
    fn header(&self) -> Header;

    /// Command data payload.
    fn data(&self) -> &[u8];

    /// Parses the payload of the response.
    ///
    /// # Errors
    ///
    /// Returns `ParseResponseError` if the response payload is invalid or cannot be parsed.
    fn parse_response(&self, response_payload: &[u8])
        -> Result<Self::Response, ParseResponseError>;
}
///
/// Errors that can occur during command execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseResponseError {
    /// The response data length does not match the expected length.
    /// .0 is expected length, .1 is actual length.
    DataLengthMismatch(usize, usize),
    ParseError(&'static str),
    /// Buffer is too small to hold the response data.
    BufferTooSmall,
}
