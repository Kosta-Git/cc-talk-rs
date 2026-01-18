/// Identifies the source device for a currency event.
///
/// Used to track which device in the pool produced a credit or error event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceId {
    /// A coin validator device.
    /// The index corresponds to the order in which it was added to the pool.
    CoinValidator(usize),
    /// A bill validator device.
    /// The index corresponds to the order in which it was added to the pool.
    BillValidator(usize),
}

impl DeviceId {
    /// Returns `true` if this is a coin validator.
    #[must_use]
    pub const fn is_coin_validator(&self) -> bool {
        matches!(self, Self::CoinValidator(_))
    }

    /// Returns `true` if this is a bill validator.
    #[must_use]
    pub const fn is_bill_validator(&self) -> bool {
        matches!(self, Self::BillValidator(_))
    }

    /// Returns the device index within its category.
    #[must_use]
    pub const fn index(&self) -> usize {
        match self {
            Self::CoinValidator(idx) | Self::BillValidator(idx) => *idx,
        }
    }
}

impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CoinValidator(idx) => write!(f, "CoinValidator[{}]", idx),
            Self::BillValidator(idx) => write!(f, "BillValidator[{}]", idx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_id_coin_validator() {
        let id = DeviceId::CoinValidator(2);

        assert!(id.is_coin_validator());
        assert!(!id.is_bill_validator());
        assert_eq!(id.index(), 2);
        assert_eq!(id.to_string(), "CoinValidator[2]");
    }

    #[test]
    fn device_id_bill_validator() {
        let id = DeviceId::BillValidator(0);

        assert!(!id.is_coin_validator());
        assert!(id.is_bill_validator());
        assert_eq!(id.index(), 0);
        assert_eq!(id.to_string(), "BillValidator[0]");
    }
}
