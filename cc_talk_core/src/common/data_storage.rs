#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DataStorage {
    pub memory_type: MemoryType,
    pub read_blocks: u16,
    pub read_bytes_per_block: u8,
    pub write_blocks: u16,
    pub write_bytes_per_block: u8,
}

impl DataStorage {
    #[must_use]
    pub const fn new(
        memory_type: MemoryType,
        read_blocks: u16,
        read_bytes_per_block: u8,
        write_blocks: u16,
        write_bytes_per_block: u8,
    ) -> Self {
        Self {
            memory_type,
            read_blocks,
            read_bytes_per_block,
            write_blocks,
            write_bytes_per_block,
        }
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn as_bytes(&self) -> [u8; 5] {
        [
            self.memory_type as u8,
            self.read_blocks as u8,
            self.read_bytes_per_block,
            self.write_blocks as u8,
            self.write_bytes_per_block,
        ]
    }

    #[must_use]
    pub const fn read_blocks(&self) -> u16 {
        if self.read_blocks == 0 {
            256
        } else {
            self.read_blocks
        }
    }

    #[must_use]
    pub const fn write_blocks(&self) -> u16 {
        if self.write_blocks == 0 {
            256
        } else {
            self.write_blocks
        }
    }

    #[must_use]
    pub const fn is_read_available(&self) -> bool {
        self.read_bytes_per_block > 0
    }

    #[must_use]
    pub const fn is_write_available(&self) -> bool {
        self.write_bytes_per_block > 0
    }
}

impl From<DataStorage> for [u8; 5] {
    fn from(value: DataStorage) -> Self {
        value.as_bytes()
    }
}

impl From<[u8; 5]> for DataStorage {
    fn from(bytes: [u8; 5]) -> Self {
        Self {
            memory_type: MemoryType::try_from(bytes[0]).expect("Invalid memory type"),
            read_blocks: u16::from(bytes[1]),
            read_bytes_per_block: bytes[2],
            write_blocks: u16::from(bytes[3]),
            write_bytes_per_block: bytes[4],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MemoryType {
    VolatileOnReset = 0,
    VolatileOnPowerDown = 1,
    PermanentLimitedUse = 2,
    PermanentUnlimitedUse = 3,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MemoryTypeError {
    #[error("Invalid memory type")]
    InvalidMemoryType,
}
impl TryFrom<u8> for MemoryType {
    type Error = MemoryTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::VolatileOnReset),
            1 => Ok(Self::VolatileOnPowerDown),
            2 => Ok(Self::PermanentLimitedUse),
            3 => Ok(Self::PermanentUnlimitedUse),
            _ => Err(MemoryTypeError::InvalidMemoryType),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FirmwareStorageType {
    RomOrEprom = 0,
    FlashOrEeprom = 1,
}

impl TryFrom<u8> for FirmwareStorageType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::RomOrEprom),
            1 => Ok(Self::FlashOrEeprom),
            _ => Err(()),
        }
    }
}
