use heapless::Vec;

/// A flexible bitmask that can be any size, using `heapless::Vec` for storage
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BitMask<const N: usize> {
    data: Vec<u8, N>,
    bit_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BitMaskError {
    #[error("index out of bounds")]
    OutOfBounds,
    #[error("not enough capacity")]
    NotEnoughCapacity,
    #[error("size mismatch")]
    SizeMismatch,
}

impl<const N: usize> BitMask<N> {
    /// Create a new bitmask with the specified number of bits
    /// All bits are initially set to 0
    ///
    /// # Errors
    ///
    /// Causes an error if the required storage exceeds the capacity N
    pub fn new(bit_count: usize) -> Result<Self, BitMaskError> {
        let byte_count = bit_count.div_ceil(8);

        if byte_count > N {
            return Err(BitMaskError::NotEnoughCapacity); // Not enough capacity
        }

        let mut data = Vec::new();
        for _ in 0..byte_count {
            data.push(0).map_err(|_| BitMaskError::OutOfBounds)?;
        }

        Ok(Self { data, bit_count })
    }

    /// Create a new bitmask filled with all 1s
    ///
    /// # Errors
    ///
    /// As with `new`, causes an error if the required storage exceeds the capacity N
    pub fn new_filled(bit_count: usize) -> Result<Self, BitMaskError> {
        let mut mask = Self::new(bit_count)?;
        mask.set_all(true);
        Ok(mask)
    }

    /// Create a bitmask from little-endian bytes
    /// The `bit_count` parameter specifies how many bits are actually used
    /// Extra bits in the last byte are ignored
    ///
    /// # Errors
    ///
    /// Errors if the provided bytes are not enough to cover `bit_count`
    pub fn from_le_bytes(bytes: &[u8], bit_count: usize) -> Result<Self, BitMaskError> {
        let required_bytes = bit_count.div_ceil(8);

        if bytes.len() < required_bytes || required_bytes > N {
            return Err(BitMaskError::NotEnoughCapacity);
        }

        let mut data = Vec::new();
        for &byte in bytes.iter().take(required_bytes) {
            data.push(byte).map_err(|_| BitMaskError::OutOfBounds)?;
        }

        let mut mask = Self { data, bit_count };

        // Clear any unused bits in the last byte
        if !bit_count.is_multiple_of(8) {
            let last_byte_index = mask.data.len() - 1;
            let used_bits_in_last_byte = bit_count % 8;
            let mask_value = (1u8 << used_bits_in_last_byte) - 1;
            mask.data[last_byte_index] &= mask_value;
        }

        Ok(mask)
    }

    /// Create a bitmask from big-endian bytes
    /// The bytes are reversed before processing as little-endian
    ///
    /// # Errors
    ///
    /// Errors if the provided bytes are not enough to cover `bit_count`
    pub fn from_be_bytes(bytes: &[u8], bit_count: usize) -> Result<Self, BitMaskError> {
        let required_bytes = bit_count.div_ceil(8);

        if bytes.len() < required_bytes || required_bytes > N {
            return Err(BitMaskError::NotEnoughCapacity);
        }

        // Take the last required_bytes and reverse them
        let start_index = bytes.len() - required_bytes;
        let le_bytes: Vec<u8, N> = bytes[start_index..].iter().rev().copied().collect();

        Self::from_le_bytes(&le_bytes, bit_count)
    }

    /// Create a bitmask from a fixed-size little-endian byte array
    ///
    /// # Errors
    ///
    /// Errors if the provided bytes are not enough to cover `bit_count`
    pub fn from_le_array<const M: usize>(
        bytes: [u8; M],
        bit_count: usize,
    ) -> Result<Self, BitMaskError> {
        Self::from_le_bytes(&bytes, bit_count)
    }

    /// Create a bitmask from a fixed-size big-endian byte array
    ///
    /// # Errors
    ///
    /// Errors if the provided bytes are not enough to cover `bit_count`
    pub fn from_be_array<const M: usize>(
        bytes: [u8; M],
        bit_count: usize,
    ) -> Result<Self, BitMaskError> {
        Self::from_be_bytes(&bytes, bit_count)
    }

    /// Get the total number of bits in this mask
    #[must_use]
    pub const fn len(&self) -> usize {
        self.bit_count
    }

    /// Check if the mask is empty (0 bits)
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bit_count == 0
    }

    /// Set a specific bit to 0 or 1
    ///
    /// # Errors
    ///
    /// Errors if the bit index is out of bounds
    pub fn set_bit(&mut self, bit_index: usize, value: bool) -> Result<(), BitMaskError> {
        if bit_index >= self.bit_count {
            return Err(BitMaskError::OutOfBounds);
        }

        let byte_index = bit_index / 8;
        let bit_offset = bit_index % 8;

        if value {
            self.data[byte_index] |= 1 << bit_offset;
        } else {
            self.data[byte_index] &= !(1 << bit_offset);
        }

        Ok(())
    }

    /// Get the value of a specific bit
    ///
    /// # Errors
    ///
    /// Errors if the bit index is out of bounds
    pub fn get_bit(&self, bit_index: usize) -> Result<bool, BitMaskError> {
        if bit_index >= self.bit_count {
            return Err(BitMaskError::OutOfBounds);
        }

        let byte_index = bit_index / 8;
        let bit_offset = bit_index % 8;

        Ok((self.data[byte_index] & (1 << bit_offset)) != 0)
    }

    /// Set a range of bits to 0 or 1 (inclusive range)
    ///
    /// # Errors
    ///
    /// Errors if the range is out of bounds
    pub fn set_range(&mut self, start: usize, end: usize, value: bool) -> Result<(), BitMaskError> {
        if start > end || end >= self.bit_count {
            return Err(BitMaskError::OutOfBounds);
        }

        for bit_index in start..=end {
            self.set_bit(bit_index, value)?;
        }

        Ok(())
    }

    /// Set all bits to 0 or 1
    pub fn set_all(&mut self, value: bool) {
        let fill_value = if value { 0xFF } else { 0x00 };

        for byte in &mut self.data {
            *byte = fill_value;
        }

        // Clear any unused bits in the last byte
        if value && !self.bit_count.is_multiple_of(8) {
            let last_byte_index = self.data.len() - 1;
            let used_bits_in_last_byte = self.bit_count % 8;
            let mask = (1u8 << used_bits_in_last_byte) - 1;
            self.data[last_byte_index] &= mask;
        }
    }

    /// Clear all bits (set to 0)
    pub fn clear(&mut self) {
        self.set_all(false);
    }

    /// Count the number of set bits (1s)
    #[must_use]
    pub fn count_ones(&self) -> usize {
        let mut count = 0;
        for &byte in &self.data {
            count += byte.count_ones() as usize;
        }

        // Adjust for any unused bits in the last byte
        if !self.bit_count.is_multiple_of(8) {
            let last_byte_index = self.data.len() - 1;
            let unused_ones =
                (self.data[last_byte_index] >> (self.bit_count % 8)).count_ones() as usize;
            count -= unused_ones;
        }

        count
    }

    /// Count the number of clear bits (0s)
    #[must_use]
    pub fn count_zeros(&self) -> usize {
        self.bit_count - self.count_ones()
    }

    /// Check if all bits are set to 1
    #[must_use]
    pub fn all(&self) -> bool {
        self.count_ones() == self.bit_count
    }

    /// Check if any bit is set to 1
    #[must_use]
    pub fn any(&self) -> bool {
        self.count_ones() > 0
    }

    /// Get the underlying byte data
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the bytes in little-endian format (least significant byte first)
    /// This returns the bytes in the same order as stored internally
    #[must_use]
    pub fn as_bytes_le(&self) -> &[u8] {
        &self.data
    }

    /// Get the bytes in big-endian format (most significant byte first)
    /// This reverses the byte order
    #[must_use]
    pub fn to_bytes_be(&self) -> Vec<u8, N> {
        let mut result = Vec::new();
        for &byte in self.data.iter().rev() {
            result.push(byte).ok(); // Safe because we know capacity is sufficient
        }
        result
    }

    /// Convert to a little-endian byte array of specified size
    /// Pads with zeros if the array is larger than needed
    /// Returns error if the array is too small
    ///
    /// # Errors
    ///
    /// Causes an error if the provided array size is smaller than needed
    pub fn to_le_bytes<const M: usize>(&self) -> Result<[u8; M], BitMaskError> {
        if M < self.data.len() {
            return Err(BitMaskError::NotEnoughCapacity);
        }

        let mut result = [0u8; M];
        for (i, &byte) in self.data.iter().enumerate() {
            result[i] = byte;
        }
        Ok(result)
    }

    /// Convert to a big-endian byte array of specified size
    /// Pads with zeros if the array is larger than needed
    /// Returns error if the array is too small
    ///
    /// # Errors
    ///
    /// Causes an error if the provided array size is smaller than needed
    pub fn to_be_bytes<const M: usize>(&self) -> Result<[u8; M], BitMaskError> {
        if M < self.data.len() {
            return Err(BitMaskError::NotEnoughCapacity);
        }

        let mut result = [0u8; M];
        let start_offset = M - self.data.len();
        for (i, &byte) in self.data.iter().rev().enumerate() {
            result[start_offset + i] = byte;
        }
        Ok(result)
    }

    /// Flip all bits
    pub fn flip(&mut self) {
        for byte in &mut self.data {
            *byte = !*byte;
        }

        // Clear any unused bits in the last byte
        if !self.bit_count.is_multiple_of(8) {
            let last_byte_index = self.data.len() - 1;
            let used_bits_in_last_byte = self.bit_count % 8;
            let mask = (1u8 << used_bits_in_last_byte) - 1;
            self.data[last_byte_index] &= mask;
        }
    }

    /// Flip a specific bit
    ///
    /// # Errors
    ///
    /// Causes an error if the bit index is out of bounds
    pub fn flip_bit(&mut self, bit_index: usize) -> Result<(), BitMaskError> {
        let current = self.get_bit(bit_index)?;
        self.set_bit(bit_index, !current)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BitMaskBinaryOpError {
    #[error("size mismatch")]
    SizeMismatch,
}

// Implement bitwise operations
impl<const N: usize> BitMask<N> {
    /// Bitwise AND with another bitmask
    ///
    /// # Errors
    ///
    /// Errors if the sizes of the two bitmasks do not match
    pub fn and(&self, other: &Self) -> Result<Self, BitMaskBinaryOpError> {
        if self.bit_count != other.bit_count {
            return Err(BitMaskBinaryOpError::SizeMismatch);
        }

        let mut result = self.clone();
        for (i, &other_byte) in other.data.iter().enumerate() {
            result.data[i] &= other_byte;
        }

        Ok(result)
    }

    /// Bitwise OR with another bitmask
    ///
    /// # Errors
    ///
    /// Errors if the sizes of the two bitmasks do not match
    pub fn or(&self, other: &Self) -> Result<Self, BitMaskBinaryOpError> {
        if self.bit_count != other.bit_count {
            return Err(BitMaskBinaryOpError::SizeMismatch);
        }

        let mut result = self.clone();
        for (i, &other_byte) in other.data.iter().enumerate() {
            result.data[i] |= other_byte;
        }

        Ok(result)
    }

    /// Bitwise XOR with another bitmask
    ///
    /// # Errors
    ///
    /// Errors if the sizes of the two bitmasks do not match
    pub fn xor(&self, other: &Self) -> Result<Self, BitMaskBinaryOpError> {
        if self.bit_count != other.bit_count {
            return Err(BitMaskBinaryOpError::SizeMismatch);
        }

        let mut result = self.clone();
        for (i, &other_byte) in other.data.iter().enumerate() {
            result.data[i] ^= other_byte;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut mask: BitMask<4> = BitMask::new(15).expect("test");

        // Test setting individual bits
        mask.set_bit(0, true).expect("test");
        mask.set_bit(7, true).expect("test");
        mask.set_bit(14, true).expect("test");

        assert!(mask.get_bit(0).expect("test"));
        assert!(mask.get_bit(7).expect("test"));
        assert!(mask.get_bit(14).expect("test"));
        assert!(!mask.get_bit(1).expect("test"));

        // Test range setting
        mask.set_range(3, 5, true).expect("test");
        for i in 3..=5 {
            assert!(mask.get_bit(i).expect("test"));
        }
    }

    #[test]
    fn test_your_example() {
        // Create a bitmask with 16 bits (2 bytes)
        let mut mask: BitMask<2> = BitMask::new(16).expect("test");

        // Set bit 15 (last bit of second byte)
        mask.set_bit(15, true).expect("test");

        // Verify the bytes
        let bytes = mask.as_bytes();
        assert_eq!(bytes.len(), 2);
        assert_eq!(bytes[0], 0); // First byte should be 0
        assert_eq!(bytes[1], 0b1000_0000); // Second byte should have MSB set

        assert!(mask.get_bit(15).expect("test"));
    }

    #[test]
    fn test_endianness() {
        let mut mask: BitMask<4> = BitMask::new(24).expect("test"); // 3 bytes

        // Set some bits: bit 0, bit 8, bit 16
        mask.set_bit(0, true).expect("test"); // LSB of byte 0
        mask.set_bit(8, true).expect("test"); // LSB of byte 1
        mask.set_bit(16, true).expect("test"); // LSB of byte 2

        // Little-endian (natural order): [0x01, 0x01, 0x01]
        let le_bytes = mask.as_bytes_le();
        assert_eq!(le_bytes, &[0x01, 0x01, 0x01]);

        // Big-endian (reversed): [0x01, 0x01, 0x01] (same in this case)
        let be_bytes = mask.to_bytes_be();
        assert_eq!(&be_bytes[..], &[0x01, 0x01, 0x01]);

        // Test with a more complex pattern
        let mut mask2: BitMask<4> = BitMask::new(16).expect("test");
        mask2.set_bit(0, true).expect("test"); // 0x01 in byte 0
        mask2.set_bit(15, true).expect("test"); // 0x80 in byte 1

        let le_bytes = mask2.as_bytes_le();
        assert_eq!(le_bytes, &[0x01, 0x80]);

        let be_bytes = mask2.to_bytes_be();
        assert_eq!(&be_bytes[..], &[0x80, 0x01]); // Reversed
    }

    #[test]
    fn test_from_bytes() {
        // Test from_le_bytes
        let bytes = [0x01, 0x80, 0x03]; // 3 bytes
        let mask: BitMask<4> = BitMask::from_le_bytes(&bytes, 20).expect("test");

        assert!(mask.get_bit(0).expect("test")); // LSB of first byte
        assert!(mask.get_bit(15).expect("test")); // MSB of second byte
        assert!(mask.get_bit(16).expect("test")); // LSB of third byte
        assert!(mask.get_bit(17).expect("test")); // Second bit of third byte
        assert!(!mask.get_bit(18).expect("test")); // Should be 0
        assert!(!mask.get_bit(19).expect("test")); // Should be 0

        // Test from_be_bytes
        let be_bytes = [0x03, 0x80, 0x01]; // Same data but big-endian
        let mask_be: BitMask<4> = BitMask::from_be_bytes(&be_bytes, 20).expect("test");

        // Should produce the same result as LE version
        assert_eq!(mask.as_bytes(), mask_be.as_bytes());
    }

    #[test]
    fn test_from_arrays() {
        // Test from_le_array
        let le_array = [0x0F, 0xF0]; // 0000_1111, 1111_0000
        let mask: BitMask<4> = BitMask::from_le_array(le_array, 16).expect("test");

        // Check first byte (0x0F = 0000_1111)
        for i in 0..4 {
            assert!(mask.get_bit(i).expect("test"));
        }
        for i in 4..8 {
            assert!(!mask.get_bit(i).expect("test"));
        }

        // Check second byte (0xF0 = 1111_0000)
        for i in 8..12 {
            assert!(!mask.get_bit(i).expect("test"));
        }
        for i in 12..16 {
            assert!(mask.get_bit(i).expect("test"));
        }

        // Test from_be_array
        let be_array = [0xF0, 0x0F]; // Big-endian version
        let mask_be: BitMask<4> = BitMask::from_be_array(be_array, 16).expect("test");

        // Should be the same as LE version
        assert_eq!(mask.as_bytes(), mask_be.as_bytes());
    }

    #[test]
    fn test_unused_bits_cleared() {
        // Test that unused bits in the last byte are properly cleared
        let bytes = [0xFF, 0xFF]; // All bits set
        let mask: BitMask<4> = BitMask::from_le_bytes(&bytes, 12).expect("test"); // Only use 12 bits

        // First byte should be unchanged
        assert_eq!(mask.as_bytes()[0], 0xFF);

        // Second byte should have only lower 4 bits set (12 - 8 = 4 bits used)
        assert_eq!(mask.as_bytes()[1], 0x0F);

        // Verify individual bits
        for i in 0..12 {
            assert!(mask.get_bit(i).expect("test"));
        }

        // Bits 12-15 should not be accessible (beyond bit_count)
        assert!(mask.get_bit(12).is_err());
    }

    #[test]
    fn test_roundtrip() {
        // Create a mask, convert to bytes, then back to mask
        let mut original: BitMask<4> = BitMask::new(20).expect("test");
        original.set_bit(0, true).expect("test");
        original.set_bit(7, true).expect("test");
        original.set_bit(15, true).expect("test");
        original.set_bit(19, true).expect("test");

        // Convert to LE bytes and back
        let le_bytes = original.as_bytes_le();
        let reconstructed: BitMask<4> = BitMask::from_le_bytes(le_bytes, 20).expect("test");

        assert_eq!(original.as_bytes(), reconstructed.as_bytes());
        assert_eq!(original.len(), reconstructed.len());

        // Test individual bits
        for i in 0..20 {
            assert_eq!(
                original.get_bit(i).expect("test"),
                reconstructed.get_bit(i).expect("test")
            );
        }
    }

    #[test]
    fn test_bitwise_operations() {
        let mut mask1: BitMask<2> = BitMask::new(10).expect("test");
        let mut mask2: BitMask<2> = BitMask::new(10).expect("test");

        mask1.set_range(0, 4, true).expect("test");
        mask2.set_range(2, 6, true).expect("test");

        let and_result = mask1.and(&mask2).expect("test");
        assert!(and_result.get_bit(2).expect("test"));
        assert!(and_result.get_bit(3).expect("test"));
        assert!(and_result.get_bit(4).expect("test"));
        assert!(!and_result.get_bit(0).expect("test"));
        assert!(!and_result.get_bit(6).expect("test"));
    }
}
