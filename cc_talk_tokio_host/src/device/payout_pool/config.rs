/// Strategy for selecting hoppers during payout operations.
///
/// Determines the order in which hoppers are used when dispensing coins.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum HopperSelectionStrategy {
    /// Use hoppers with highest coin value first (greedy algorithm).
    /// This minimizes the number of coins dispensed.
    #[default]
    LargestFirst,
    /// Use hoppers with lowest coin value first.
    /// This maximizes the number of coins dispensed.
    SmallestFirst,
    /// Prefer hoppers with highest inventory levels.
    /// This helps balance inventory across hoppers.
    BalanceInventory,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_strategy_is_largest_first() {
        assert_eq!(
            HopperSelectionStrategy::default(),
            HopperSelectionStrategy::LargestFirst
        );
    }
}
