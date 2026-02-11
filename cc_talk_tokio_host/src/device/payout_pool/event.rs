use std::collections::HashMap;

use crate::device::base::CommandError;

/// Events emitted by the [`super::PayoutPool`] during operations.
///
/// Subscribe to these events by receiving from the event channel
/// returned by the pool builder.
#[derive(Debug, Clone)]
pub enum PayoutPoolEvent {
    /// A hopper was detected as empty during a payout operation.
    HopperEmpty {
        /// The ccTalk address of the empty hopper.
        address: u8,
        /// The coin value this hopper dispenses (in smallest currency units).
        coin_value: u32,
    },

    /// Progress update during an active payout operation.
    PayoutProgress {
        /// Total value requested for this payout.
        requested: u32,
        /// Total value dispensed so far.
        dispensed: u32,
        /// Value remaining to dispense.
        remaining: u32,
        /// The hopper currently dispensing (by address), if any.
        active_hopper: Option<u8>,
        /// Number of coins dispensed so far.
        coins_dispensed: u32,
    },

    /// The payout plan was rebalanced because a hopper ran empty or failed.
    PayoutPlanRebalanced {
        /// The address of the hopper that triggered the rebalance.
        exhausted_hopper: u8,
        /// The remaining value being replanned.
        remaining_value: u32,
        /// The new plan: hopper address -> coin count.
        new_plan: HashMap<u8, u8>,
    },

    /// A payout operation completed.
    PayoutCompleted {
        /// Value that was requested.
        requested: u32,
        /// Value actually dispensed.
        dispensed: u32,
        /// Number of coins dispensed.
        coins_count: u32,
        /// Whether the full requested amount was dispensed.
        fully_dispensed: bool,
    },

    /// A hopper encountered a communication error.
    HopperError {
        /// The ccTalk address of the hopper.
        address: u8,
        /// The error that occurred.
        error: CommandError,
    },

    /// A hopper was enabled in the pool.
    HopperEnabled {
        /// The ccTalk address of the hopper.
        address: u8,
    },

    /// A hopper was disabled in the pool.
    HopperDisabled {
        /// The ccTalk address of the hopper.
        address: u8,
    },
}
