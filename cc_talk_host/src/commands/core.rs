use cc_talk_core::Category;

use super::command::CommandSet;

pub mod core_commands;

/// ccTalk core command set.
pub struct CoreCommandSet;

impl CommandSet for CoreCommandSet {
    const NAME: &'static str = "Core";

    /// The core command set is compatible with all categories.
    fn is_compatible_with(_: Category) -> bool {
        true
    }
}
