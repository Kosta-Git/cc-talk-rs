use cc_talk_core::{
    Category,
    cc_talk::{Manufacturer, SerialCode, core_plus_commands::DataStorage},
};

pub trait DeviceImpl {
    fn manufacturer(&self) -> Manufacturer;
    fn category(&self) -> Category;
    fn product_code(&self) -> &str;
    fn serial_number(&self) -> SerialCode;
    fn software_revision(&self) -> &str;
    fn data_storage_availability(&self) -> DataStorage;
    fn comms_revision(&self) -> (u8, u8, u8);
    fn reset(&self);
}

pub trait SimplePayoutDevice {
    fn payout_status(&self) -> u8;
}

