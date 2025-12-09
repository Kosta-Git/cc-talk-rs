const REGISTER_1_MASK: u16 = 0;
const REGISTER_2_MASK: u16 = 256;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChangerFlags {
    SingulatorRunning = REGISTER_1_MASK + (1 << 0),
    EscalatorRunning = REGISTER_1_MASK + (1 << 1),
    PrcessingMoneyIn = REGISTER_1_MASK + (1 << 2),
    ProcessingMoneyOut = REGISTER_1_MASK + (1 << 3),
    FaultDetected = REGISTER_1_MASK + (1 << 4),
    AvalancheDetected = REGISTER_1_MASK + (1 << 5),
    ChangerInitialising = REGISTER_1_MASK + (1 << 6),
    EntryFlapOpen = REGISTER_1_MASK + (1 << 7),
    ContinuousRejects = REGISTER_2_MASK + (1 << 0),
    HopperConfigChange = REGISTER_2_MASK + (1 << 1),
    RejectDivertActive = REGISTER_2_MASK + (1 << 2),
    ExitCupFull = REGISTER_2_MASK + (1 << 3),
    NonFatalFaultDetected = REGISTER_2_MASK + (1 << 4),
}

impl ChangerFlags {
    /// Verifies if the flag is set in the given register.
    ///
    /// # Panics
    ///
    /// Panics if an invalid `register_id` is provided.
    #[must_use]
    pub fn has_flag(&self, register: u8, register_id: u8) -> bool {
        let register_mask = match register_id {
            1 => REGISTER_1_MASK,
            2 => REGISTER_2_MASK,
            _ => panic!("Invalid register: {}", register),
        };

        let flag_raw_value = *self as u16;
        if flag_raw_value < register_mask || flag_raw_value >= register_mask + 256 {
            return false;
        }

        #[allow(clippy::cast_possible_truncation)]
        let flag_value = (flag_raw_value ^ register_mask) as u8;
        (flag_value & register) == flag_value
    }

    const fn all_flags() -> [Self; 13] {
        [
            Self::SingulatorRunning,
            Self::EscalatorRunning,
            Self::PrcessingMoneyIn,
            Self::ProcessingMoneyOut,
            Self::FaultDetected,
            Self::AvalancheDetected,
            Self::ChangerInitialising,
            Self::EntryFlapOpen,
            Self::ContinuousRejects,
            Self::HopperConfigChange,
            Self::RejectDivertActive,
            Self::ExitCupFull,
            Self::NonFatalFaultDetected,
        ]
    }
}

/// Parses changer flags from the provided registers into a vec.
///
/// # Panics
///
/// Panics if the length of `registers` is not 0, 1, or 2.
#[must_use]
pub fn parse_changer_flags_heapless(registers: &[u8]) -> heapless::Vec<ChangerFlags, 13> {
    assert!(
        (0..=2).contains(&registers.len()),
        "registers must be 0, 1 or 2 bytes long"
    );

    let mut flags = heapless::Vec::new();
    for flag in ChangerFlags::all_flags() {
        if flag.has_flag(registers[0], 1) || flag.has_flag(registers[1], 2) {
            flags.push(flag).ok();
        }
    }
    flags
}
