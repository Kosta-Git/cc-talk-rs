use crate::{Category, ChecksumType};

/// Represents a ccTalk Device
/// This can be used to remove some boilerplate when sending packets
pub struct Device {
    address: u8,
    category: Category,
    checksum_type: ChecksumType,
    encrypted: bool,
}

impl Device {
    /// Creates a new device, this has no impact on the wire.
    ///
    /// # Note
    ///
    /// Encryption is not implemented yet, so its set to false by default.
    pub fn new(address: u8, category: Category, checksum_type: ChecksumType) -> Self {
        Device {
            address,
            category,
            checksum_type,
            encrypted: false,
        }
    }

    pub fn address(&self) -> u8 {
        self.address
    }

    pub fn category(&self) -> &Category {
        &self.category
    }

    pub fn checksum_type(&self) -> &ChecksumType {
        &self.checksum_type
    }

    pub fn encrypted(&self) -> bool {
        self.encrypted
    }
}
