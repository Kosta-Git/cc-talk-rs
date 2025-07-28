#![no_std]

#[cfg(any(feature = "std", test))]
extern crate std;

mod common;
mod log;
mod serde;

pub mod cc_talk {
    pub use crate::common::bill_event_types::*;
    pub use crate::common::bill_routing::*;
    pub use crate::common::bit_mask::*;
    pub use crate::common::category::*;
    pub use crate::common::changer_device::*;
    pub use crate::common::changer_error::*;
    pub use crate::common::changer_flags::*;
    pub use crate::common::changer_status::*;
    pub use crate::common::checksum::*;
    pub use crate::common::coin_acceptor_errors::*;
    pub use crate::common::coin_calibration_codes::*;
    pub use crate::common::coin_event::*;
    pub use crate::common::coin_value_format::*;
    pub use crate::common::currency::*;
    pub use crate::common::data_storage::*;
    pub use crate::common::date::*;
    pub use crate::common::device::*;
    pub use crate::common::escrow_status::*;
    pub use crate::common::fault_code::*;
    pub use crate::common::hopper_flags::*;
    pub use crate::common::hopper_status::*;
    pub use crate::common::lamp_control::*;
    pub use crate::common::manufacturers::*;
    pub use crate::common::option_flags::*;
    pub use crate::common::packet::*;
    pub use crate::common::power_option::*;
    pub use crate::common::teach_mode_status::*;

    pub use crate::serde::*;
}

#[cfg(test)]
#[cfg_attr(feature = "defmt", defmt::panic_handler)]
fn panic() -> ! {
    core::panic!("panic via `defmt::panic!`")
}
