#![no_std]

#[cfg(any(feature = "std", test))]
extern crate std;

mod common;
mod log;
mod serde;

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
    pub use crate::common::bit_mask::*;
    pub use crate::common::category::*;
    pub use crate::common::checksum::*;
    pub use crate::common::coin_acceptor_errors::*;
    pub use crate::common::coin_event::*;
    pub use crate::common::coin_value_format::*;
    pub use crate::common::data_storage::*;
    pub use crate::common::date::*;
    pub use crate::common::device::*;
    pub use crate::common::fault_code::*;
    pub use crate::common::hopper_flags::*;
    pub use crate::common::hopper_status::*;
    pub use crate::common::manufacturers::*;
    pub use crate::common::option_flags::*;
    pub use crate::common::packet::*;

    pub use crate::serde::*;
}

#[cfg(test)]
#[cfg_attr(feature = "defmt", defmt::panic_handler)]
fn panic() -> ! {
    core::panic!("panic via `defmt::panic!`")
}
