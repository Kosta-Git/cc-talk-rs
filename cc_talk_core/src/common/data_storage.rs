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
    pub fn new(
        memory_type: MemoryType,
        read_blocks: u16,
        read_bytes_per_block: u8,
        write_blocks: u16,
        write_bytes_per_block: u8,
    ) -> Self {
        DataStorage {
            memory_type,
            read_blocks,
            read_bytes_per_block,
            write_blocks,
            write_bytes_per_block,
        }
    }

    pub fn as_bytes(&self) -> [u8; 5] {
        [
            self.memory_type as u8,
            self.read_blocks as u8,
            self.read_bytes_per_block,
            self.write_blocks as u8,
            self.write_bytes_per_block,
        ]
    }

    pub fn read_blocks(&self) -> u16 {
        if self.read_blocks == 0 {
            256
        } else {
            self.read_blocks
        }
    }

    pub fn write_blocks(&self) -> u16 {
        if self.write_blocks == 0 {
            256
        } else {
            self.write_blocks
        }
    }

    pub fn is_read_available(&self) -> bool {
        self.read_bytes_per_block > 0
    }

    pub fn is_write_available(&self) -> bool {
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
        DataStorage {
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MemoryTypeError {
    InvalidMemoryType,
}
impl TryFrom<u8> for MemoryType {
    type Error = MemoryTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MemoryType::VolatileOnReset),
            1 => Ok(MemoryType::VolatileOnPowerDown),
            2 => Ok(MemoryType::PermanentLimitedUse),
            3 => Ok(MemoryType::PermanentUnlimitedUse),
            _ => Err(MemoryTypeError::InvalidMemoryType),
        }
    }
}
