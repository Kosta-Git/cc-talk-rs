use crate::{
    cc_talk::{CorePlusCommandSet, MdbCommandSet},
    Category,
};

use super::{
    command::{BelongsTo, CommandSet},
    core::{core_commands::SimplePollCommand, CoreCommandSet},
};

pub mod device_commands;

pub struct CoinAcceptorCommandSet;
impl CommandSet for CoinAcceptorCommandSet {
    const NAME: &'static str = "Coin Acceptor";

    /// The coin acceptor command set is compatible with the Coin Acceptor category.
    fn is_compatible_with(category: Category) -> bool {
        category == Category::CoinAcceptor
    }
}
/// Makes the core command set compatible with the coin acceptor command set.
impl BelongsTo<CoinAcceptorCommandSet> for CoreCommandSet {}
/// Makes the core plus command set compatible with the coin acceptor command set.
impl BelongsTo<CoinAcceptorCommandSet> for CorePlusCommandSet {}
/// Makes the MDB command set compatible with the coin acceptor command set.
impl BelongsTo<CoinAcceptorCommandSet> for MdbCommandSet {}

pub struct PayoutCommandSet;
impl CommandSet for PayoutCommandSet {
    const NAME: &'static str = "Payout";

    /// The payout command set is compatible with the Payout category.
    fn is_compatible_with(category: Category) -> bool {
        category == Category::Payout
    }
}
/// Makes the core command set compatible with the coin acceptor command set.
impl BelongsTo<PayoutCommandSet> for CoreCommandSet {}
/// Makes the core plus command set compatible with the coin acceptor command set.
impl BelongsTo<PayoutCommandSet> for CorePlusCommandSet {}
/// Makes the MDB command set compatible with the coin acceptor command set.
impl BelongsTo<PayoutCommandSet> for MdbCommandSet {}

pub struct BillValidatorCommandSet;
impl CommandSet for BillValidatorCommandSet {
    const NAME: &'static str = "Bill Validator";

    /// The bill validator command set is compatible with the Bill Validator category.
    fn is_compatible_with(category: Category) -> bool {
        category == Category::BillValidator
    }
}

pub struct ChangerEscrowCommandSet;
impl CommandSet for ChangerEscrowCommandSet {
    const NAME: &'static str = "Changer/Escrow";

    /// The changer/escrow command set is compatible with the Changer/Escrow category.
    fn is_compatible_with(category: Category) -> bool {
        matches!(category, Category::Changer | Category::Escrow,)
    }
}
