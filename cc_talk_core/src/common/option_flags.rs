#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RequestOptionFlags {
    flags: u8,
}
impl RequestOptionFlags {
    #[must_use]
    pub const fn new(flags: u8) -> Self {
        Self { flags }
    }

    #[must_use]
    pub const fn for_coin_acceptor(&self) -> CoinAcceptorOptionFlags {
        CoinAcceptorOptionFlags::new(self.flags)
    }

    #[must_use]
    pub const fn for_bill_validator(&self) -> BillValidatorOptionFlags {
        BillValidatorOptionFlags::new(self.flags)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CreditCodeFormat {
    CoinPosition,
    CoinValueFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CoinAcceptorOptionFlags {
    credit_code_format: CreditCodeFormat,
}
impl CoinAcceptorOptionFlags {
    #[must_use]
    const fn new(mask: u8) -> Self {
        let credit_code_format = if mask & 1 == 1 {
            CreditCodeFormat::CoinValueFormat
        } else {
            CreditCodeFormat::CoinPosition
        };
        Self { credit_code_format }
    }

    #[must_use]
    pub const fn credit_code_format(&self) -> CreditCodeFormat {
        self.credit_code_format
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(clippy::struct_excessive_bools)] // TODO: refactor this
pub struct BillValidatorOptionFlags {
    stacker: bool,
    escrow: bool,
    individual_bill_accept_counter: bool,
    individual_error_counter: bool,
    non_volatile_counter: bool,
    bill_teach: bool,
    bill_security_tuning: bool,
    remote_bill_programming: bool,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl BillValidatorOptionFlags {
    const fn new(mask: u8) -> Self {
        let stacker = mask & 0b0000_0001 != 0;
        let escrow = mask & 0b0000_0010 != 0;
        let individual_bill_accept_counter = mask & 0b0000_0100 != 0;
        let individual_error_counter = mask & 0b0000_1000 != 0;
        let non_volatile_counter = mask & 0b0001_0000 != 0;
        let bill_teach = mask & 0b0010_0000 != 0;
        let bill_security_tuning = mask & 0b0100_0000 != 0;
        let remote_bill_programming = mask & 0b1000_0000 != 0;
        Self {
            stacker,
            escrow,
            individual_bill_accept_counter,
            individual_error_counter,
            non_volatile_counter,
            bill_teach,
            bill_security_tuning,
            remote_bill_programming,
        }
    }

    const fn stacker(&self) -> bool {
        self.stacker
    }

    const fn escrow(&self) -> bool {
        self.escrow
    }

    const fn individual_bill_accept_counter(&self) -> bool {
        self.individual_bill_accept_counter
    }

    const fn individual_error_counter(&self) -> bool {
        self.individual_error_counter
    }

    const fn non_volatile_counter(&self) -> bool {
        self.non_volatile_counter
    }

    const fn bill_teach(&self) -> bool {
        self.bill_teach
    }

    const fn bill_security_tuning(&self) -> bool {
        self.bill_security_tuning
    }

    const fn remote_bill_programming(&self) -> bool {
        self.remote_bill_programming
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn coin_acceptor_option_flags() {
        let options = RequestOptionFlags::new(0b0000_0001);
        let acceptor_flags = options.for_coin_acceptor();

        assert_eq!(
            acceptor_flags.credit_code_format(),
            CreditCodeFormat::CoinValueFormat
        );
    }

    #[test]
    fn bill_validator_option_flags() {
        let options = RequestOptionFlags::new(0b1111_1111);
        let validator_flags = options.for_bill_validator();

        assert!(validator_flags.stacker());
        assert!(validator_flags.escrow());
        assert!(validator_flags.individual_bill_accept_counter());
        assert!(validator_flags.individual_error_counter());
        assert!(validator_flags.non_volatile_counter());
        assert!(validator_flags.bill_teach());
        assert!(validator_flags.bill_security_tuning());
        assert!(validator_flags.remote_bill_programming());
    }
}
