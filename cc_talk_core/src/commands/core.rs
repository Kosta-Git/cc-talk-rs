use crate::Category;

use super::command::{Command, CommandSet, ParseResponseError};

/// ccTalk core command set.
pub struct CoreCommandSet;

impl CommandSet for CoreCommandSet {
    const NAME: &'static str = "Core";

    /// The core command set is compatible with all categories.
    fn is_compatible_with(_: Category) -> bool {
        true
    }
}

pub struct SimplePollCommand;
impl Command for SimplePollCommand {
    type Response = ();

    fn header(&self) -> crate::Header {
        crate::Header::SimplePoll
    }

    fn data(&self) -> &[u8] {
        &[]
    }

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
