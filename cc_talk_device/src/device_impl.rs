use core::future::Future;

use cc_talk_core::{
    cc_talk::{DataStorage, HopperDispenseStatus, HopperStatus, Manufacturer, SerialCode},
    Category, ChecksumType, Device,
};

pub trait DeviceImpl {
    fn manufacturer(&self) -> Manufacturer;
    fn category(&self) -> Category;
    fn checksum_type(&self) -> ChecksumType;
    fn product_code(&self) -> &str;
    fn serial_number(&self) -> SerialCode;
    fn software_revision(&self) -> &str;
    fn build_code(&self) -> &str;
    fn data_storage_availability(&self) -> DataStorage;
    fn comms_revision(&self) -> (u8, u8, u8);
    fn reset(&self) -> impl Future<Output = ()> + '_;

    fn is_for_me(&self, destination_address: u8) -> bool;
    fn address(&self) -> u8;
    fn device(&self) -> Device;
}

pub trait SimplePayoutDevice {
    fn request_sensor_status(&self) -> impl Future<Output = HopperStatus> + '_;
    fn emergency_stop(&self) -> impl Future<Output = ()> + '_;
    fn request_hopper_coin(&self) -> &str; // TODO: Implement a struct to represent the coin
    fn request_hopper_dispense_count(&self) -> impl Future<Output = u32> + '_;
    fn dispense_hopper_coins(&self, count: u8) -> impl Future<Output = ()> + '_;
    fn request_payout_status(&self) -> impl Future<Output = HopperDispenseStatus> + '_;
    fn enable_payout(&self, enable: bool) -> impl Future<Output = ()> + '_;
    fn test(&self) -> impl Future<Output = (u8, u8, u8)> + '_;
}
