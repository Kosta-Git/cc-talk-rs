#![allow(dead_code)]

use super::packet::{
    DATA_LENGTH_OFFSET, DATA_OFFSET, DESTINATION_OFFSET, HEADER_OFFSET, SOURCE_OFFSET,
};

/// ccTalk checksum types.
///
/// The two main checksum types are simple checksum ([ChecksumType::Crc8]) and the
/// [ChecksumType::Crc16].
///
/// A 5 bit checksum is also used for USB full speed, however I never saw it used in practice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChecksumType {
    Crc8,
    Crc16,
}

// Calculates the crc16 checksum for a ccTalk block.
///
/// This function assumes a valid ccTalk block, which would be at least 4 bytes long, maximum 256
/// bytes. As well as the data block being the size specified at [DATA_LENGTH_OFFSET].
///
/// # Implementation
///
/// The implementation method is selected at compile time based on feature flags:
///
/// - **Default** (no features): Uses bit-by-bit computation method ([`crc16_compute`])
///   - Slower execution but uses no lookup table memory
///   - Ideal for memory-constrained embedded systems
///
/// - **With `crc-lookup` feature**: Uses pre-calculated lookup table method ([`crc16_lookup`])
///   - Faster execution but requires 512 bytes of ROM/Flash for the lookup table
///   - Ideal when performance is more critical than memory usage
///
/// # Definition
///
/// The CRC 16 checksum definition can be found in the ccTalk specification, section 7.11
/// cctalk-part-1-v4-7.pdf.
///
/// # Examples
///
/// ```rust
/// use cc_talk_core::cc_talk::crc16;
///
/// // Example ccTalk packet: [dest][len][src][header][data...]
/// let packet = [40, 0, 0x3F, 1, 0x46];
/// let checksum = crc16(&packet);
/// assert_eq!(checksum, 0x3F46);
/// ```
///
/// # Panics
///
/// This function assumes that the block is a valid ccTalk block and has a length of at least 4 bytes.
pub fn crc16(block: &[u8]) -> u16 {
    #[cfg(not(feature = "crc-lookup"))]
    return crc16_compute(block);

    #[cfg(feature = "crc-lookup")]
    return crc16_lookup(block);
}

fn crc16_compute(block: &[u8]) -> u16 {
    let data_end_offset = DATA_OFFSET + block[DATA_LENGTH_OFFSET] as usize;
    [
        block[DESTINATION_OFFSET],
        block[DATA_LENGTH_OFFSET],
        block[HEADER_OFFSET],
    ]
    .iter()
    .chain(block[DATA_OFFSET..data_end_offset].iter())
    .fold(0u16, |crc, &byte| crc16_compute_pass(crc, byte))
}

fn crc16_lookup(block: &[u8]) -> u16 {
    let data_end_offset = DATA_OFFSET + block[DATA_LENGTH_OFFSET] as usize;
    [
        block[DESTINATION_OFFSET],
        block[DATA_LENGTH_OFFSET],
        block[HEADER_OFFSET],
    ]
    .iter()
    .chain(block[DATA_OFFSET..data_end_offset].iter())
    .fold(0u16, |crc, &byte| crc16_lookup_pass(crc, byte))
}

fn crc16_compute_pass(mut crc: u16, byte: u8) -> u16 {
    crc ^= (byte as u16) << 8;
    for _ in 0..8 {
        if crc & 0x8000 != 0 {
            crc = (crc << 1) ^ 0x1021;
        } else {
            crc <<= 1;
        }
    }
    crc
}

fn crc16_lookup_pass(crc: u16, byte: u8) -> u16 {
    let table_index = (crc >> 8) ^ (byte as u16);
    (crc << 8) ^ CRC_CCITT_LOOKUP[table_index as usize]
}

const CRC_CCITT_LOOKUP: [u16; 256] = [
    0x0000, 0x1021, 0x2042, 0x3063, 0x4084, 0x50A5, 0x60C6, 0x70E7, 0x8108, 0x9129, 0xA14A, 0xB16B,
    0xC18C, 0xD1AD, 0xE1CE, 0xF1EF, 0x1231, 0x0210, 0x3273, 0x2252, 0x52B5, 0x4294, 0x72F7, 0x62D6,
    0x9339, 0x8318, 0xB37B, 0xA35A, 0xD3BD, 0xC39C, 0xF3FF, 0xE3DE, 0x2462, 0x3443, 0x0420, 0x1401,
    0x64E6, 0x74C7, 0x44A4, 0x5485, 0xA56A, 0xB54B, 0x8528, 0x9509, 0xE5EE, 0xF5CF, 0xC5AC, 0xD58D,
    0x3653, 0x2672, 0x1611, 0x0630, 0x76D7, 0x66F6, 0x5695, 0x46B4, 0xB75B, 0xA77A, 0x9719, 0x8738,
    0xF7DF, 0xE7FE, 0xD79D, 0xC7BC, 0x48C4, 0x58E5, 0x6886, 0x78A7, 0x0840, 0x1861, 0x2802, 0x3823,
    0xC9CC, 0xD9ED, 0xE98E, 0xF9AF, 0x8948, 0x9969, 0xA90A, 0xB92B, 0x5AF5, 0x4AD4, 0x7AB7, 0x6A96,
    0x1A71, 0x0A50, 0x3A33, 0x2A12, 0xDBFD, 0xCBDC, 0xFBBF, 0xEB9E, 0x9B79, 0x8B58, 0xBB3B, 0xAB1A,
    0x6CA6, 0x7C87, 0x4CE4, 0x5CC5, 0x2C22, 0x3C03, 0x0C60, 0x1C41, 0xEDAE, 0xFD8F, 0xCDEC, 0xDDCD,
    0xAD2A, 0xBD0B, 0x8D68, 0x9D49, 0x7E97, 0x6EB6, 0x5ED5, 0x4EF4, 0x3E13, 0x2E32, 0x1E51, 0x0E70,
    0xFF9F, 0xEFBE, 0xDFDD, 0xCFFC, 0xBF1B, 0xAF3A, 0x9F59, 0x8F78, 0x9188, 0x81A9, 0xB1CA, 0xA1EB,
    0xD10C, 0xC12D, 0xF14E, 0xE16F, 0x1080, 0x00A1, 0x30C2, 0x20E3, 0x5004, 0x4025, 0x7046, 0x6067,
    0x83B9, 0x9398, 0xA3FB, 0xB3DA, 0xC33D, 0xD31C, 0xE37F, 0xF35E, 0x02B1, 0x1290, 0x22F3, 0x32D2,
    0x4235, 0x5214, 0x6277, 0x7256, 0xB5EA, 0xA5CB, 0x95A8, 0x8589, 0xF56E, 0xE54F, 0xD52C, 0xC50D,
    0x34E2, 0x24C3, 0x14A0, 0x0481, 0x7466, 0x6447, 0x5424, 0x4405, 0xA7DB, 0xB7FA, 0x8799, 0x97B8,
    0xE75F, 0xF77E, 0xC71D, 0xD73C, 0x26D3, 0x36F2, 0x0691, 0x16B0, 0x6657, 0x7676, 0x4615, 0x5634,
    0xD94C, 0xC96D, 0xF90E, 0xE92F, 0x99C8, 0x89E9, 0xB98A, 0xA9AB, 0x5844, 0x4865, 0x7806, 0x6827,
    0x18C0, 0x08E1, 0x3882, 0x28A3, 0xCB7D, 0xDB5C, 0xEB3F, 0xFB1E, 0x8BF9, 0x9BD8, 0xABBB, 0xBB9A,
    0x4A75, 0x5A54, 0x6A37, 0x7A16, 0x0AF1, 0x1AD0, 0x2AB3, 0x3A92, 0xFD2E, 0xED0F, 0xDD6C, 0xCD4D,
    0xBDAA, 0xAD8B, 0x9DE8, 0x8DC9, 0x7C26, 0x6C07, 0x5C64, 0x4C45, 0x3CA2, 0x2C83, 0x1CE0, 0x0CC1,
    0xEF1F, 0xFF3E, 0xCF5D, 0xDF7C, 0xAF9B, 0xBFBA, 0x8FD9, 0x9FF8, 0x6E17, 0x7E36, 0x4E55, 0x5E74,
    0x2E93, 0x3EB2, 0x0ED1, 0x1EF0,
];

/// Calculates the simple checksum for a ccTalk block.
///
/// This function assumes a valid ccTalk block, which would be at least 4 bytes long, maximum 256
/// bytes. As well as the data block being the size specified at [DATA_LENGTH_OFFSET].
///
/// # Definition
///
/// The simple checksum definition can be found in the ccTalk specification, section 7.10
/// cctalk-part-1-v4-7.pdf.
///
/// # Panics
///
/// This function assumes that the block is a valid ccTalk block and has a length of at least 4 bytes.
pub fn crc8(block: &[u8]) -> u8 {
    let data_end_offset = DATA_OFFSET + block[DATA_LENGTH_OFFSET] as usize;
    let crc = [
        block[DESTINATION_OFFSET],
        block[DATA_LENGTH_OFFSET],
        block[SOURCE_OFFSET],
        block[HEADER_OFFSET],
    ]
    .iter()
    .chain(block[DATA_OFFSET..data_end_offset].iter())
    .fold(0u8, |acc, &byte| acc.wrapping_add(byte));

    (256u16 - crc as u16) as u8
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        crc16(&[0, 0, 0, 0]);
    }

    #[test]
    fn example_simple_checksum() {
        assert_eq!(crc8(&[2, 0, 1, 242]), 11);

        // appending the expected checksum should not matter
        assert_eq!(crc8(&[2, 0, 1, 246, 7]), 7);

        assert_eq!(crc8(&[1, 3, 2, 0, 78, 97, 188, 143]), 143);
    }

    #[test]
    fn example_crc16_compute_checksum() {
        assert_eq!(crc16_compute(&[40, 0, 0x3F, 1, 0x46]), 0x3F46);
        assert_eq!(crc16_compute(&[1, 0, 0x37, 0, 0x30]), 0x3730);
    }

    #[test]
    fn example_crc16_lookup_checksum() {
        assert_eq!(crc16_lookup(&[40, 0, 0x3F, 1, 0x46]), 0x3F46);
        assert_eq!(crc16_lookup(&[1, 0, 0x37, 0, 0x30]), 0x3730);
    }
}
