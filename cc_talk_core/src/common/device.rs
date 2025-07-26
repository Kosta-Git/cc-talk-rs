use crate::{Category, ChecksumType};

/// Represents a ccTalk Device
/// This can be used to remove some boilerplate when sending packets
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

/// Represents the device serial number.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SerialCode(u8, u8, u8);
impl SerialCode {
    /// Creates a new serial code.
    pub fn new(a: u8, b: u8, c: u8) -> Self {
        SerialCode(a, b, c)
    }

    /// Returns the first byte of the serial code.
    pub fn major(&self) -> u8 {
        self.0
    }

    /// Returns the second byte of the serial code.
    pub fn minor(&self) -> u8 {
        self.1
    }

    /// Returns the third byte of the serial code.
    pub fn fix(&self) -> u8 {
        self.2
    }

    // Verifies if the device version is at least the specified version.
    pub fn is_at_least(&self, major: u8, minor: u8, fix: u8) -> bool {
        (self.0 > major)
            || (self.0 == major && self.1 > minor)
            || (self.0 == major && self.1 == minor && self.2 >= fix)
    }

    /// Returns the serial number in decimal as specified by the ccTalk protocol.
    pub fn as_number(&self) -> u32 {
        self.fix() as u32 + (256 * (self.minor() as u32)) + (65536 * (self.major() as u32))
    }
}

impl core::fmt::Display for SerialCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

impl core::fmt::Debug for SerialCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serial_code_display() {
        let code = SerialCode::new(1, 2, 3);
        assert_eq!(std::format!("{code}"), "1.2.3");
    }

    #[test]
    fn as_decimal() {
        let code = SerialCode::new(255, 255, 255);
        // Should be 255 + 256 * 255 + 65536 * 255 which is 24 bits set to 1
        assert_eq!(code.as_number(), 0xFFFFFF);
    }
}
