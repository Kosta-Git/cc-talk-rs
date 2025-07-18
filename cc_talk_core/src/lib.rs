#![no_std]

#[cfg(any(feature = "std", test))]
extern crate std;

mod commands;
mod common;

pub use common::bill_event_types::BillEvent;

pub use common::category::Address;
pub use common::category::Category;

pub use common::checksum::ChecksumType;

pub use common::coin_acceptor_errors::CoinAcceptorError;

pub use common::device::Device;

pub use common::fault_code::Fault;

pub use common::hopper_flags::HopperFlag;

pub use common::packet::Header;

pub mod cc_talk {
    pub use crate::common::bill_event_types::*;
    pub use crate::common::category::*;
    pub use crate::common::checksum::*;
    pub use crate::common::coin_acceptor_errors::*;
    pub use crate::common::device::*;
    pub use crate::common::fault_code::*;
    pub use crate::common::hopper_flags::*;
    pub use crate::common::manufacturers::*;
    pub use crate::common::packet::*;

    pub use crate::commands::bill_validator::*;
    pub use crate::commands::changer_escrow::*;
    pub use crate::commands::coin_acceptor::*;
    pub use crate::commands::core::*;
    pub use crate::commands::core_plus::*;
    pub use crate::commands::multi_drop::*;
    pub use crate::commands::payout::*;
}
