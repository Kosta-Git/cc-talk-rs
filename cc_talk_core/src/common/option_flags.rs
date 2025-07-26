#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RequestOptionFlags {
    flags: u8,
}
impl RequestOptionFlags {
    pub fn new(flags: u8) -> Self {
        Self { flags }
    }

    pub fn for_coin_acceptor(&self) -> CoinAcceptorOptionFlags {
        CoinAcceptorOptionFlags::new(self.flags)
    }

    pub fn for_bill_validator(&self) -> BillValidatorOptionFlags {
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
    fn new(mask: u8) -> Self {
        let credit_code_format = if mask & 1 == 1 {
            CreditCodeFormat::CoinValueFormat
        } else {
            CreditCodeFormat::CoinPosition
        };
        Self { credit_code_format }
    }

    pub fn credit_code_format(&self) -> CreditCodeFormat {
        self.credit_code_format
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
impl BillValidatorOptionFlags {
    fn new(mask: u8) -> Self {
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

    fn stacker(&self) -> bool {
        self.stacker
    }

    fn escrow(&self) -> bool {
        self.escrow
    }

    fn individual_bill_accept_counter(&self) -> bool {
        self.individual_bill_accept_counter
    }

    fn individual_error_counter(&self) -> bool {
        self.individual_error_counter
    }

    fn non_volatile_counter(&self) -> bool {
        self.non_volatile_counter
    }

    fn bill_teach(&self) -> bool {
        self.bill_teach
    }

    fn bill_security_tuning(&self) -> bool {
        self.bill_security_tuning
    }

    fn remote_bill_programming(&self) -> bool {
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
