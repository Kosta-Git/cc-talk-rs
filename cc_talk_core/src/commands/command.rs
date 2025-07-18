use crate::{Category, Header};

/// Defines which command are available to a category.
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
    /// Command header.
    fn header(&self) -> Header;

    /// Command data payload.
    fn data(&self) -> &[u8];
}
///
/// Errors that can occur during command execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    IncompatibleCommandSet {
        command_set: &'static str,
        device_category: Category,
    },
    CommunicationError(CommunicationError),
    DeviceError(u8),
    ParseError(&'static str),
    OperationFailed(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommunicationError {
    Timeout,
    ChecksumMismatch,
    InvalidResponse,
    TransportError,
}
