#![allow(dead_code)]

use cc_talk_core::cc_talk::{Category, Header};

/// Defines which command are available to a category.
/// This trait is an utility design to help users know which commands are available for their
/// device category.
pub trait CommandSet {
    /// The name of the command set
    const NAME: &'static str;

    /// Check if the command set is compatible with the given category.
    fn is_compatible_with(category: Category) -> bool;
}

/// Marker trait for commands that belong to a specific command set.
pub trait BelongsTo<CS: CommandSet> {}

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
