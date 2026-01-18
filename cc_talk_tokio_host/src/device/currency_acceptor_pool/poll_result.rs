use super::device_id::DeviceId;
use crate::device::base::CommandError;

/// A confirmed currency credit from a coin or bill acceptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrencyCredit {
    /// The value of the accepted currency in smallest units (e.g., cents).
    pub value: u32,
    /// The device that accepted this currency.
    pub source: DeviceId,
    /// The position index (0-15) of the coin/bill type on the device.
    pub position: u8,
}

impl CurrencyCredit {
    /// Creates a new currency credit.
    #[must_use]
    pub const fn new(value: u32, source: DeviceId, position: u8) -> Self {
        Self {
            value,
            source,
            position,
        }
    }
}

/// A bill currently held in escrow awaiting routing decision.
///
/// Only relevant when using `BillRoutingMode::Manual`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingBill {
    /// The value of the pending bill in smallest units.
    pub value: u32,
    /// The bill validator holding this bill.
    pub source: DeviceId,
    /// The bill type position (0-15) on the device.
    pub bill_type: u8,
}

impl PendingBill {
    /// Creates a new pending bill.
    #[must_use]
    pub const fn new(value: u32, source: DeviceId, bill_type: u8) -> Self {
        Self {
            value,
            source,
            bill_type,
        }
    }
}

/// Error that occurred while polling a specific device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolPollError {
    /// The device that produced the error.
    pub source: DeviceId,
    /// The error that occurred.
    pub error: CommandError,
}

impl PoolPollError {
    /// Creates a new pool poll error.
    #[must_use]
    pub const fn new(source: DeviceId, error: CommandError) -> Self {
        Self { source, error }
    }
}

impl std::fmt::Display for PoolPollError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.source, self.error)
    }
}

impl std::error::Error for PoolPollError {}

/// Result of polling all devices in the pool.
#[derive(Debug, Clone, Default)]
pub struct PoolPollResult {
    /// Confirmed credits received during this poll.
    pub credits: Vec<CurrencyCredit>,
    /// Bills currently held in escrow (for manual routing mode).
    pub pending_bills: Vec<PendingBill>,
    /// Errors that occurred while polling individual devices.
    /// Polling continues despite individual device errors.
    pub errors: Vec<PoolPollError>,
    /// Total value received in this poll (sum of credits).
    pub total_received: u32,
}

impl PoolPollResult {
    /// Creates a new empty poll result.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            credits: Vec::new(),
            pending_bills: Vec::new(),
            errors: Vec::new(),
            total_received: 0,
        }
    }

    /// Adds a credit to the result and updates the total.
    pub fn add_credit(&mut self, credit: CurrencyCredit) {
        self.total_received += credit.value;
        self.credits.push(credit);
    }

    /// Adds a pending bill to the result.
    pub fn add_pending_bill(&mut self, bill: PendingBill) {
        self.pending_bills.push(bill);
    }

    /// Adds an error to the result.
    pub fn add_error(&mut self, error: PoolPollError) {
        self.errors.push(error);
    }

    /// Returns `true` if no credits were received and no bills are pending.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.credits.is_empty() && self.pending_bills.is_empty()
    }

    /// Returns `true` if any errors occurred during polling.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_credit_new() {
        let credit = CurrencyCredit::new(100, DeviceId::CoinValidator(0), 5);

        assert_eq!(credit.value, 100);
        assert_eq!(credit.source, DeviceId::CoinValidator(0));
        assert_eq!(credit.position, 5);
    }

    #[test]
    fn pending_bill_new() {
        let bill = PendingBill::new(1000, DeviceId::BillValidator(1), 3);

        assert_eq!(bill.value, 1000);
        assert_eq!(bill.source, DeviceId::BillValidator(1));
        assert_eq!(bill.bill_type, 3);
    }

    #[test]
    fn pool_poll_result_add_credit_updates_total() {
        let mut result = PoolPollResult::new();

        result.add_credit(CurrencyCredit::new(100, DeviceId::CoinValidator(0), 1));
        result.add_credit(CurrencyCredit::new(200, DeviceId::CoinValidator(0), 2));

        assert_eq!(result.total_received, 300);
        assert_eq!(result.credits.len(), 2);
    }

    #[test]
    fn pool_poll_result_is_empty() {
        let mut result = PoolPollResult::new();
        assert!(result.is_empty());

        result.add_credit(CurrencyCredit::new(100, DeviceId::CoinValidator(0), 1));
        assert!(!result.is_empty());
    }

    #[test]
    fn pool_poll_result_has_errors() {
        let mut result = PoolPollResult::new();
        assert!(!result.has_errors());

        result.add_error(PoolPollError::new(
            DeviceId::CoinValidator(0),
            CommandError::Timeout,
        ));
        assert!(result.has_errors());
    }
}
