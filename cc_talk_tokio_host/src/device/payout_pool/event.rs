use crate::device::base::CommandError;

use super::poll_result::DispenseProgress;

/// Events emitted during a payout operation.
///
/// Subscribe to these events by passing an `mpsc::Sender<PayoutEvent>` to
/// [`super::PayoutPool::payout_with_events`].
#[derive(Debug, Clone)]
pub enum PayoutEvent {
    /// Intermediate progress update during an active payout operation.
    Progress(DispenseProgress),

    /// A hopper was detected as empty during a payout operation.
    HopperEmpty {
        /// The ccTalk address of the empty hopper.
        address: u8,
        /// The coin value this hopper dispenses (in smallest currency units).
        coin_value: u32,
    },

    /// The payout plan was rebalanced because a hopper ran empty or failed.
    PlanRebalanced {
        /// The address of the hopper that triggered the rebalance.
        exhausted_hopper: u8,
        /// The remaining value being replanned.
        remaining_value: u32,
        /// The new plan: `(hopper_address, coin_count)` pairs in dispensing order.
        new_plan: Vec<(u8, u8)>,
    },

    /// A hopper encountered a communication error during payout.
    HopperError {
        /// The ccTalk address of the hopper.
        address: u8,
        /// The error that occurred.
        error: CommandError,
    },
}
