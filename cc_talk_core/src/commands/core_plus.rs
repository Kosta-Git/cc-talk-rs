use crate::Category;

use super::command::CommandSet;

pub mod core_plus_commands;

/// ccTalk code plus command set.
pub struct CorePlusCommandSet;

impl CommandSet for CorePlusCommandSet {
    const NAME: &'static str = "CorePlus";

    /// The core plus command set is compatible with all categories.
    fn is_compatible_with(_: Category) -> bool {
        true
    }
}
