use crate::Category;

use super::command::CommandSet;

/// ccTalk core command set.
pub struct CoreCommandSet;

impl CommandSet for CoreCommandSet {
    const NAME: &'static str = "Core";

    /// The core command set is compatible with all categories.
    fn is_compatible_with(category: Category) -> bool {
        true
    }
}
