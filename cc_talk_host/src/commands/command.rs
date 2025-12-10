#![allow(dead_code)]

use cc_talk_core::cc_talk::Category;

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
