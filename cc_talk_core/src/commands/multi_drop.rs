use crate::Category;

use super::command::CommandSet;

pub mod multi_drop_commands;

/// MDB command set.
pub struct MdbCommandSet;

impl CommandSet for MdbCommandSet {
    const NAME: &'static str = "MDB";

    /// The MDB command set is compatible with the MDB category.
    fn is_compatible_with(_: Category) -> bool {
        true
    }
}
