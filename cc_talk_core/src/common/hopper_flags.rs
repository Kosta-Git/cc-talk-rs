#![allow(dead_code)]

const REGISTER_1_MASK: u16 = 0;
const REGISTER_2_MASK: u16 = 256;
const REGISTER_3_MASK: u16 = 512;

/// Represents all the possible flags when testing a hopper.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HopperFlag {
    // Register 1 starts at 0b00_00000001 == 1
    /// Payout stopped because a maximum threshold current was exceeded. This is the
    /// software equivalent of a protection fuse and may indicate an irrecoverable jam or a
    /// motor bridge fault. The flag must be cleared with a software reset prior to next payout
    /// but if the fault is permanent then it may not be possible to clear this condition.
    AbsoluteMaximumCurrentExceeded = REGISTER_1_MASK + (1 << 0),
    /// Indicates the payout operation terminated after a specified timeout period with no
    /// coins visible on the exit sensor. This could be because the hopper was empty.
    /// Warning flag only.
    PayoutTimeoutOccurred = REGISTER_1_MASK + (1 << 1),
    /// A motor jam was detected and the hopper attempted a reverse to clear it. Warning flag
    /// only
    MotorReversedToClearJam = REGISTER_1_MASK + (1 << 2),
    /// The hopper exit opto saw a blockage outside a payout operation. This could be due to
    /// a coin stuck in the exit path or a deliberate fraud attempt. The flag must be cleared
    /// with a software reset prior to next payout.
    OptoFraudPathBlockedDuringIdle = REGISTER_1_MASK + (1 << 3),
    /// The hopper exit opto saw a short-circuit outside a payout operation. This is most
    /// likely due to shining light directly into the exit opto. The flag must be cleared with a
    /// software reset prior to next payout.
    OptoFraudShortCircuitDuringIdle = REGISTER_1_MASK + (1 << 4),
    /// The hopper exit opto saw a blockage for too long a time while paying out coins. This
    /// could be due to a coin stuck in the exit path or a deliberate fraud attempt. The flag
    /// must be cleared with a software reset prior to next payout.
    OptoBlockedPermanentlyDuringPayout = REGISTER_1_MASK + (1 << 5),
    /// This flag indicates that the hopper has had power applied to it. It is cleared by a
    /// software reset and so can be used to distinguish between a software reset and a
    /// hardware reset / power supply removal. Warning flag only.
    PowerUpDetected = REGISTER_1_MASK + (1 << 6),
    /// The hopper is always disabled after a power-up or reset. The flag must be cleared with
    /// an ‘Enable hopper’ command prior to next payout
    PayoutDisabled = REGISTER_1_MASK + (1 << 7),
    // Register 2 starts at 0b01_00000001 == 257
    /// The hopper exit opto saw a short-circuit while paying out coins. This is most likely
    /// due to shining light directly into the exit opto. The flag must be cleared with a
    /// software reset prior to next payout.
    OptoFraudPathBlockedDuringPayout = REGISTER_2_MASK + (1 << 0),
    /// This flag indicates the hopper is working in single coin mode and will only respond to
    /// commands to dispense one coin at a time. This method is slower but achieves
    /// maximum security as there is full handshaking on each coin paid out. Status flag only.
    SingleCoinPayoutMode = REGISTER_2_MASK + (1 << 1),
    /// Accumulator mode : This flag indicates the hopper cannot dispense any more coins
    /// and a secondary hopper must be used to complete the dispense operation. Status flag
    /// only.
    UseOtherHopper = REGISTER_2_MASK + (1 << 2),
    /// Some hoppers are fitted with sensors on the fingers or sliders used to eject coins
    /// cleanly. If a coin is seen leaving the exit optos but no signal is seen on the finger or
    /// slider then this error flag is set as it could be a possible fraud attempt. The flag must
    /// be cleared with a software reset prior to next payout.
    OptoFraudAttemptFinger = REGISTER_2_MASK + (1 << 3),
    /// If the number of sequential motor reverses exceeds a preset limit then the hopper
    /// dispense is aborted and this flag set. It is likely that there is a permanent coin jam in
    /// the hopper. The flag must be cleared with a software reset prior to next payout.
    MotorReverseLimitReached = REGISTER_2_MASK + (1 << 4),
    /// Accumulator mode : This flag indicates a fault has been detected with the coil used to
    /// distinguish between different coin types. The flag must be cleared with a software
    /// reset prior to next payout but if the fault is permanent then it may not be possible to
    /// clear this condition.
    InductiveCoilFault = REGISTER_2_MASK + (1 << 5),
    /// An error has occurred with the NV memory which means the values of the coin paid
    /// and unpaid counters may not be correct. Refer to the product manual for appropriate
    /// action.
    NVMemoryChecksumError = REGISTER_2_MASK + (1 << 6),
    /// This flag indicates that the PIN number mechanism is enabled and that a PIN has to
    /// be entered using command header 218 before any coins can be paid out. Status flag
    /// only.
    PinNumberMechanism = REGISTER_2_MASK + (1 << 7),
    // Register3 starts at 0b10_00000001 == 513
    /// If power is lost during a payout operation then this flag will be set. The values of the
    /// paid and unpaid counters will indicate what happened just prior to power being lost.
    /// The flag must be cleared with a software reset prior to next payout.
    PowerDownDuringPayout = REGISTER_3_MASK + (1 << 0),
    /// Accumulator mode : This flag indicates that the coin just paid was not a recognised
    /// type. The hopper stops immediately. Either an illegal coin type has been placed in the
    /// hopper or there is a problem with the coin sensing. The flag must be cleared with a
    /// software reset prior to next payout.
    UnknownCoinTypePaid = REGISTER_3_MASK + (1 << 1),
    /// The hopper dispense operation failed because the PIN number protection mechanism
    /// has been enabled ( using the ‘Enter new PIN number’ command ) - see PIN number
    /// mechanism flag - but the ‘Enter PIN number’ command has not yet been sent or the
    /// number sent was incorrect. The flag must be cleared with a software reset prior to
    /// next payout.
    PinNumberIncorrect = REGISTER_3_MASK + (1 << 2),
    /// If the hopper uses encryption and an incorrect cipher key is sent then the hopper
    /// dispense operation will fail and this flag set. The flag must be cleared with a software
    /// reset prior to next payout
    IncorrectCipherKey = REGISTER_3_MASK + (1 << 3),
    /// If the hopper requires a cipher key to be calculated when dispensing coins then this
    /// flag is set. Status flag only
    EncryptionEnabled = REGISTER_3_MASK + (1 << 4),
}

impl HopperFlag {
    /// Checks if a register contains a specific flag.
    /// `register_id` represents the hopper register number (1, 2, or 3).
    ///
    /// # Panics
    ///
    /// It will panic if `register_id` is not 1, 2, or 3.
    pub fn has_flag(&self, register: u8, register_id: u8) -> bool {
        let register_mask = match register_id {
            1 => REGISTER_1_MASK,
            2 => REGISTER_2_MASK,
            3 => REGISTER_3_MASK,
            _ => panic!("register_id must be 1, 2, or 3"),
        };

        let flag_raw_value = *self as u16;
        if flag_raw_value < register_mask || flag_raw_value > register_mask + (1 << 7) {
            return false;
        }

        let flag_value = (*self as u16 ^ register_mask) as u8;
        (flag_value & register) == flag_value
    }

    const fn all_flags() -> [HopperFlag; 21] {
        [
            HopperFlag::AbsoluteMaximumCurrentExceeded,
            HopperFlag::PayoutTimeoutOccurred,
            HopperFlag::MotorReversedToClearJam,
            HopperFlag::OptoFraudPathBlockedDuringIdle,
            HopperFlag::OptoFraudShortCircuitDuringIdle,
            HopperFlag::OptoBlockedPermanentlyDuringPayout,
            HopperFlag::PowerUpDetected,
            HopperFlag::PayoutDisabled,
            HopperFlag::OptoFraudPathBlockedDuringPayout,
            HopperFlag::SingleCoinPayoutMode,
            HopperFlag::UseOtherHopper,
            HopperFlag::OptoFraudAttemptFinger,
            HopperFlag::MotorReverseLimitReached,
            HopperFlag::InductiveCoilFault,
            HopperFlag::NVMemoryChecksumError,
            HopperFlag::PinNumberMechanism,
            HopperFlag::PowerDownDuringPayout,
            HopperFlag::UnknownCoinTypePaid,
            HopperFlag::PinNumberIncorrect,
            HopperFlag::IncorrectCipherKey,
            HopperFlag::EncryptionEnabled,
        ]
    }

    /// Parses flags from a byte array representing hopper registers.
    /// Only works if the `heapless` feature is enabled.
    ///
    /// # Usage
    ///
    /// ```
    /// use cc_talk_core::cc_talk::*;
    ///
    /// let register_1 = 0b00000001; // AbsoluteMaximumCurrentExceeded
    /// let register_2 = 0b00000010; // SingleCoinPayoutMode
    /// let register_3 = 0b00000001; // PowerDownDuringPayout
    ///
    /// let registers = &[register_1, register_2, register_3];
    ///
    /// let flags = HopperFlag::parse_hopper_flags_heapless(registers);
    /// assert_eq!(flags.len(), 3);
    /// assert_eq!(flags[0], HopperFlag::AbsoluteMaximumCurrentExceeded);
    /// assert_eq!(flags[1], HopperFlag::SingleCoinPayoutMode);
    /// assert_eq!(flags[2], HopperFlag::PowerDownDuringPayout);
    /// ```
    ///
    /// # Panics
    ///
    /// It will panic if the length of `registers` is not 0, 1, 2, or 3.
    pub fn parse_hopper_flags_heapless(registers: &[u8]) -> heapless::Vec<HopperFlag, 21> {
        assert!(
            (0..=3).contains(&registers.len()),
            "registers must be of length 0, 1, 2, or 3"
        );

        let mut flags = heapless::Vec::new();

        for (register_id, &register_value) in registers.iter().enumerate() {
            let register_num = (register_id + 1) as u8;
            for bit_pos in 0..8 {
                if register_num == 3 && bit_pos > 4 {
                    // Register 3 only has flags up to bit position 4
                    break;
                }

                if (register_value & (1 << bit_pos)) != 0 {
                    let flag_value = match register_num {
                        1 => REGISTER_1_MASK + (1 << bit_pos),
                        2 => REGISTER_2_MASK + (1 << bit_pos),
                        3 => REGISTER_3_MASK + (1 << bit_pos),
                        _ => unreachable!(),
                    };

                    if let Some(flag) = HopperFlag::u16_to_hopper_flag(flag_value) {
                        let _ = flags.push(flag);
                    }
                }
            }
        }

        flags
    }

    /// Parses flags from a byte array representing hopper registers.
    /// Only works if the `heapless` feature is enabled.
    ///
    /// # Usage
    ///
    /// ```
    /// use cc_talk_core::cc_talk::*;
    ///
    /// let register_1 = 0b00000001; // AbsoluteMaximumCurrentExceeded
    /// let register_2 = 0b00000010; // SingleCoinPayoutMode
    /// let register_3 = 0b00000001; // PowerDownDuringPayout
    ///
    /// let registers = &[register_1, register_2, register_3];
    ///
    /// let flags = HopperFlag::parse_hopper_flags_std(registers);
    /// assert_eq!(flags.len(), 3);
    /// assert_eq!(flags[0], HopperFlag::AbsoluteMaximumCurrentExceeded);
    /// assert_eq!(flags[1], HopperFlag::SingleCoinPayoutMode);
    /// assert_eq!(flags[2], HopperFlag::PowerDownDuringPayout);
    /// ```
    ///
    /// # Panics
    ///
    /// It will panic if the length of `registers` is not 0, 1, 2, or 3.
    #[cfg(feature = "std")]
    pub fn parse_hopper_flags_std(registers: &[u8]) -> std::vec::Vec<HopperFlag> {
        assert!(
            (0..=3).contains(&registers.len()),
            "registers must be of length 0, 1, 2, or 3"
        );

        let mut flags = std::vec::Vec::new();

        for (register_id, &register_value) in registers.iter().enumerate() {
            let register_num = (register_id + 1) as u8;
            for bit_pos in 0..8 {
                if register_num == 3 && bit_pos > 4 {
                    // Register 3 only has flags up to bit position 4
                    break;
                }

                if (register_value & (1 << bit_pos)) != 0 {
                    let flag_value = match register_num {
                        1 => REGISTER_1_MASK + (1 << bit_pos),
                        2 => REGISTER_2_MASK + (1 << bit_pos),
                        3 => REGISTER_3_MASK + (1 << bit_pos),
                        _ => unreachable!(),
                    };

                    if let Some(flag) = HopperFlag::u16_to_hopper_flag(flag_value) {
                        flags.push(flag);
                    }
                }
            }
        }

        flags
    }

    /// Parses flags from a byte array representing hopper registers.
    ///
    /// # Usage
    ///
    /// ```
    /// use cc_talk_core::cc_talk::*;
    ///
    /// let register_1 = 0b00000001; // AbsoluteMaximumCurrentExceeded
    /// let register_2 = 0b00000010; // SingleCoinPayoutMode
    /// let register_3 = 0b00000001; // PowerDownDuringPayout
    ///
    /// let hopper_status_registers = &[register_1, register_2, register_3];
    ///
    /// let (flags, count) = HopperFlag::parse_hopper_flags_array(hopper_status_registers);
    /// assert_eq!(count, 3);
    /// assert_eq!(flags[0], Some(HopperFlag::AbsoluteMaximumCurrentExceeded));
    /// assert_eq!(flags[1], Some(HopperFlag::SingleCoinPayoutMode));
    /// assert_eq!(flags[2], Some(HopperFlag::PowerDownDuringPayout));
    /// ```
    ///
    /// # Panics
    ///
    /// It will panic if the length of `registers` is not 0, 1, 2, or 3.
    pub fn parse_hopper_flags_array(registers: &[u8]) -> ([Option<HopperFlag>; 21], usize) {
        assert!(
            (0..=3).contains(&registers.len()),
            "registers must be of length 0, 1, 2, or 3"
        );

        let mut flags = [None; 21];
        let mut count = 0;
        for (register_id, &register_value) in registers.iter().enumerate() {
            let register_num = (register_id + 1) as u8;

            for bit_pos in 0..8 {
                if register_num == 3 && bit_pos > 4 {
                    // Register 3 only has flags up to bit position 4
                    break;
                }

                if (register_value & (1 << bit_pos)) != 0 && count < 21 {
                    let flag_value = match register_num {
                        1 => REGISTER_1_MASK + (1 << bit_pos),
                        2 => REGISTER_2_MASK + (1 << bit_pos),
                        3 => REGISTER_3_MASK + (1 << bit_pos),
                        _ => continue,
                    };

                    if let Some(flag) = HopperFlag::u16_to_hopper_flag(flag_value) {
                        flags[count] = Some(flag);
                        count += 1;
                    }
                }
            }
        }

        (flags, count)
    }

    fn u16_to_hopper_flag(value: u16) -> Option<HopperFlag> {
        match value {
            1 => Some(HopperFlag::AbsoluteMaximumCurrentExceeded),
            2 => Some(HopperFlag::PayoutTimeoutOccurred),
            4 => Some(HopperFlag::MotorReversedToClearJam),
            8 => Some(HopperFlag::OptoFraudPathBlockedDuringIdle),
            16 => Some(HopperFlag::OptoFraudShortCircuitDuringIdle),
            32 => Some(HopperFlag::OptoBlockedPermanentlyDuringPayout),
            64 => Some(HopperFlag::PowerUpDetected),
            128 => Some(HopperFlag::PayoutDisabled),
            257 => Some(HopperFlag::OptoFraudPathBlockedDuringPayout),
            258 => Some(HopperFlag::SingleCoinPayoutMode),
            260 => Some(HopperFlag::UseOtherHopper),
            264 => Some(HopperFlag::OptoFraudAttemptFinger),
            272 => Some(HopperFlag::MotorReverseLimitReached),
            288 => Some(HopperFlag::InductiveCoilFault),
            320 => Some(HopperFlag::NVMemoryChecksumError),
            384 => Some(HopperFlag::PinNumberMechanism),
            513 => Some(HopperFlag::PowerDownDuringPayout),
            514 => Some(HopperFlag::UnknownCoinTypePaid),
            516 => Some(HopperFlag::PinNumberIncorrect),
            520 => Some(HopperFlag::IncorrectCipherKey),
            528 => Some(HopperFlag::EncryptionEnabled),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_flag_register_1() {
        let flag = HopperFlag::AbsoluteMaximumCurrentExceeded;
        assert!(flag.has_flag(0b00000001, 1));
        assert!(!flag.has_flag(0b00000010, 1));
        assert!(!flag.has_flag(0b00000001, 2));
    }

    #[test]
    fn test_has_flag_register_2() {
        let flag = HopperFlag::SingleCoinPayoutMode;
        assert!(flag.has_flag(0b00000010, 2)); // Bit 1 set in register 2
        assert!(!flag.has_flag(0b00000001, 2)); // Bit 0 set, but flag is bit 1
        assert!(!flag.has_flag(0b00000010, 1)); // Wrong register
    }

    #[test]
    fn test_has_flag_register_3() {
        let flag = HopperFlag::EncryptionEnabled;
        assert!(flag.has_flag(0b00010000, 3)); // Bit 4 set in register 3
        assert!(!flag.has_flag(0b00001000, 3)); // Bit 3 set, but flag is bit 4
        assert!(!flag.has_flag(0b00010000, 1)); // Wrong register
    }

    #[test]
    #[should_panic(expected = "register_id must be 1, 2, or 3")]
    fn test_has_flag_invalid_register() {
        let flag = HopperFlag::AbsoluteMaximumCurrentExceeded;
        flag.has_flag(0b00000001, 4); // Invalid register
    }

    #[test]
    fn test_parse_single_register() {
        let registers = &[0b00000001]; // Only first flag set in register 1
        let (flags, count) = HopperFlag::parse_hopper_flags_array(registers);
        assert_eq!(count, 1);
        assert_eq!(flags[0], Some(HopperFlag::AbsoluteMaximumCurrentExceeded));
    }

    #[test]
    fn test_parse_multiple_flags_single_register() {
        let registers = &[0b10000001]; // First and last flags in register 1
        let (flags, count) = HopperFlag::parse_hopper_flags_array(registers);
        assert_eq!(count, 2);
        assert_eq!(flags[0], Some(HopperFlag::AbsoluteMaximumCurrentExceeded));
        assert_eq!(flags[1], Some(HopperFlag::PayoutDisabled));
    }

    #[test]
    fn test_parse_multiple_registers() {
        let registers = &[0b00000001, 0b00000010, 0b00000100]; // One flag in each register
        let (flags, count) = HopperFlag::parse_hopper_flags_array(registers);
        assert_eq!(count, 3);

        let mut found_flags: heapless::Vec<HopperFlag, 21> = heapless::Vec::new();
        for flag in flags.into_iter().flatten() {
            found_flags.push(flag).unwrap();
        }

        assert!(found_flags.contains(&HopperFlag::AbsoluteMaximumCurrentExceeded));
        assert!(found_flags.contains(&HopperFlag::SingleCoinPayoutMode));
        assert!(found_flags.contains(&HopperFlag::PinNumberIncorrect));
    }

    #[test]
    fn test_parse_empty_registers() {
        let registers = &[];
        let (flags, count) = HopperFlag::parse_hopper_flags_array(registers);
        assert_eq!(count, 0);
        assert_eq!(flags[0], None);
    }

    #[test]
    fn test_parse_no_flags_set() {
        let registers = &[0b00000000, 0b00000000, 0b00000000];
        let (_, count) = HopperFlag::parse_hopper_flags_array(registers);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_parse_all_register_1_flags() {
        let registers = &[0b11111111]; // All flags set in register 1
        let (_, count) = HopperFlag::parse_hopper_flags_array(registers);
        assert_eq!(count, 8); // All 8 flags in register 1
    }

    #[test]
    fn test_parse_all_registers_flags() {
        let registers = &[255, 255, 255]; // All flags
        let (_, count) = HopperFlag::parse_hopper_flags_array(registers);
        assert_eq!(count, 21); // All flags
    }

    #[test]
    fn test_parse_register_3_boundary() {
        let registers = &[0b00000000, 0b00000000, 0b00011111]; // All valid flags in register 3
        let (_, count) = HopperFlag::parse_hopper_flags_array(registers);
        assert_eq!(count, 5); // Only 5 flags in register 3 (bits 0-4)
    }

    #[test]
    fn test_parse_register_3_invalid_bits() {
        let registers = &[0b00000000, 0b00000000, 0b11100000]; // Invalid bits 5-7 in register 3
        let (_, count) = HopperFlag::parse_hopper_flags_array(registers);
        assert_eq!(count, 0); // Should ignore invalid bits
    }

    #[test]
    #[should_panic(expected = "registers must be of length 0, 1, 2, or 3")]
    fn test_parse_invalid_length() {
        let registers = &[0, 1, 2, 3, 4]; // Invalid length > 3
        HopperFlag::parse_hopper_flags_array(registers);
    }

    #[test]
    fn test_u16_to_hopper_flag_valid() {
        assert_eq!(
            HopperFlag::u16_to_hopper_flag(1),
            Some(HopperFlag::AbsoluteMaximumCurrentExceeded)
        );
        assert_eq!(
            HopperFlag::u16_to_hopper_flag(258),
            Some(HopperFlag::SingleCoinPayoutMode)
        );
        assert_eq!(
            HopperFlag::u16_to_hopper_flag(528),
            Some(HopperFlag::EncryptionEnabled)
        );
    }

    #[test]
    fn test_u16_to_hopper_flag_invalid() {
        assert_eq!(HopperFlag::u16_to_hopper_flag(999), None);
        assert_eq!(HopperFlag::u16_to_hopper_flag(0), None);
    }

    #[test]
    fn test_parse_with_heapless() {
        let registers = &[0b00000011]; // First two flags in register 1
        let flags = HopperFlag::parse_hopper_flags_heapless(registers);
        assert_eq!(flags.len(), 2);
        assert_eq!(flags[0], HopperFlag::AbsoluteMaximumCurrentExceeded);
        assert_eq!(flags[1], HopperFlag::PayoutTimeoutOccurred);
    }

    #[test]
    fn test_parse_multi_with_heapless() {
        let registers = &[255, 255, 255]; // All flags
        let flags = HopperFlag::parse_hopper_flags_heapless(registers);
        assert_eq!(flags.len(), 21);
    }

    #[test]
    fn test_flag_values_consistency() {
        // Test that enum values match expected bit patterns
        assert_eq!(HopperFlag::AbsoluteMaximumCurrentExceeded as u16, 1);
        assert_eq!(HopperFlag::PayoutTimeoutOccurred as u16, 2);
        assert_eq!(HopperFlag::SingleCoinPayoutMode as u16, 258);
        assert_eq!(HopperFlag::EncryptionEnabled as u16, 528);
    }
}
