/// ccTalk fault codes as defined in the ccTalk Generic Specification
/// Table 3 - ccTalk Fault Code Table
///
/// These are status codes returned in response to a 'Perform self-check' command.
/// All non-zero fault codes are 'fatal' errors that prevent normal operation and
/// require service intervention. The device automatically inhibits operation when
/// a fault is detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FaultCode {
    /// No fault detected - normal operating condition
    Ok = 0,

    /// EEPROM checksum corrupted
    ///
    /// The coin acceptor found a mismatch between the checksum calculated from
    /// a coin data area and a stored checksum. Indicates possible EEPROM corruption.
    /// This checksum is not intended for use with program code/firmware.
    EepromChecksumCorrupted = 1,

    /// Fault on inductive coils
    ///
    /// A fault was detected with the coils for inductive coin validation.
    /// Optional extra info: Coil number
    InductiveCoilsFault = 2,

    /// Fault on credit sensor
    ///
    /// A fault was detected with the post-gate credit sensor. A serial credit
    /// can only be generated if the coin passes this sensor.
    CreditSensorFault = 3,

    /// Fault on piezo sensor
    ///
    /// A fault was detected on the piezo sensor used for slug rejection.
    PiezoSensorFault = 4,

    /// Fault on reflective sensor
    ///
    /// A fault was detected on an opto-reflective sensor for coin validation.
    ReflectiveSensorFault = 5,

    /// Fault on diameter sensor
    ///
    /// A fault was detected on a validation sensor specifically reserved for
    /// diameter resolution.
    DiameterSensorFault = 6,

    /// Fault on wake-up sensor
    ///
    /// A fault was detected on the sensor used to wake-up a coin acceptor from
    /// a sleeping or power-down state.
    WakeUpSensorFault = 7,

    /// Fault on sorter exit sensors
    ///
    /// A fault was detected on the sorter exit sensors. These sensors confirm
    /// that a coin has cleared the sorter flaps and perhaps to verify the path taken.
    /// Optional extra info: Sensor number
    SorterExitSensorsFault = 8,

    /// NVRAM checksum corrupted
    ///
    /// If battery-backed RAM is used then a corrupted checksum was discovered.
    NvramChecksumCorrupted = 9,

    /// Coin dispensing error (Obsolete)
    ///
    /// A fault was found during a hopper coin dispense operation.
    /// Note: This fault code is obsolete and was incorporated into the 'Test hopper' command.
    CoinDispensingError = 10,

    /// Low level sensor error (Obsolete)
    ///
    /// A fault was found on a hopper low level sensor.
    /// Note: This fault code is obsolete and was incorporated into the 'Test hopper' command.
    /// Optional extra info: Hopper or tube number
    LowLevelSensorError = 11,

    /// High level sensor error (Obsolete)
    ///
    /// A fault was found on a hopper high level sensor.
    /// Note: This fault code is obsolete and was incorporated into the 'Test hopper' command.
    /// Optional extra info: Hopper or tube number
    HighLevelSensorError = 12,

    /// Coin counting error (Obsolete)
    ///
    /// A fault was detected in the hopper 'dead reckoning' system. It probably ran out of coins.
    /// Note: This fault code is obsolete and was incorporated into the 'Test hopper' command.
    CoinCountingError = 13,

    /// Keypad error
    ///
    /// A fault was found on a keypad.
    /// Optional extra info: Key number
    KeypadError = 14,

    /// Button error
    ///
    /// A fault was found on a button.
    ButtonError = 15,

    /// Display error
    ///
    /// A fault was found on a display device.
    DisplayError = 16,

    /// Coin auditing error
    ///
    /// A fault was detected in the memory block used to record the number of
    /// inserted and accepted coins on a coin acceptor.
    CoinAuditingError = 17,

    /// Fault on reject sensor
    ///
    /// A fault was detected with the reject sensor. This is the sensor used to
    /// confirm a coin has left the reject path and has been returned to the customer.
    RejectSensorFault = 18,

    /// Fault on coin return mechanism
    ///
    /// A fault was detected in the flight deck mechanism used by the customer
    /// to clear coin jams in the entry or validation area.
    CoinReturnMechanismFault = 19,

    /// Fault on C.O.S. mechanism
    ///
    /// A fault was found on the 'Coin on String' sensor.
    CosMechanismFault = 20,

    /// Fault on rim sensor
    ///
    /// A fault was found on a coin rim validation sensor.
    RimSensorFault = 21,

    /// Fault on thermistor
    ///
    /// A fault was found on a thermistor used to measure ambient temperature.
    ThermistorFault = 22,

    /// Payout motor fault
    ///
    /// A fault was found on a hopper motor (used on changers).
    /// Optional extra info: Hopper number
    PayoutMotorFault = 23,

    /// Payout timeout (Obsolete)
    ///
    /// A coin was dispensed from a hopper but was not seen on the payout verification sensor.
    /// Note: This fault code is obsolete and was incorporated into the 'Test hopper' command.
    /// Optional extra info: Hopper or tube number
    PayoutTimeout = 24,

    /// Payout jammed (Obsolete)
    ///
    /// A jam was detected in a hopper.
    /// Note: This fault code is obsolete and was incorporated into the 'Test hopper' command.
    /// Optional extra info: Hopper or tube number
    PayoutJammed = 25,

    /// Payout sensor fault
    ///
    /// A fault was found on a hopper payout verification sensor (used on changers).
    /// Optional extra info: Hopper or tube number
    PayoutSensorFault = 26,

    /// Level sensor error
    ///
    /// A fault was found on a hopper level sensor.
    /// Optional extra info: Hopper or tube number
    LevelSensorError = 27,

    /// Personality module not fitted
    ///
    /// A personality or configuration module needed with some ccTalk peripherals was not fitted.
    PersonalityModuleNotFitted = 28,

    /// Personality checksum corrupted
    ///
    /// A data checksum on a personality or configuration module was corrupted.
    PersonalityChecksumCorrupted = 29,

    /// ROM checksum mismatch
    ///
    /// The device has found a mismatch between the checksum calculated from a program
    /// code area and a stored checksum. Possible flash memory/ROM corruption.
    /// Now used for any kind of application firmware checksum error.
    RomChecksumMismatch = 30,

    /// Missing slave device (Obsolete)
    ///
    /// A ccTalk peripheral did not find an attached slave device.
    /// Only of use in multi-master systems.
    /// Optional extra info: Slave address
    MissingSlaveDevice = 31,

    /// Internal comms bad
    ///
    /// A ccTalk peripheral could not access an internal serial device.
    /// Optional extra info: Slave address
    InternalCommsBad = 32,

    /// Supply voltage outside operating limits
    ///
    /// The ccTalk device is operating outside supply voltage limits defined
    /// in the product specification.
    SupplyVoltageOutsideLimits = 33,

    /// Temperature outside operating limits
    ///
    /// The ccTalk device is operating outside temperature limits defined
    /// in the product specification.
    TemperatureOutsideLimits = 34,

    /// D.C.E. fault
    ///
    /// A fault was found on the Dual Coin Entry chute.
    /// Optional extra info: 1 = coin, 2 = token
    DceFault = 35,

    /// Fault on bill validation sensor
    ///
    /// A fault was found on one of the bill validator validation sensors.
    /// Optional extra info: Sensor number
    BillValidationSensorFault = 36,

    /// Fault on bill transport motor
    ///
    /// A fault was found on the motor used to drive a bill through the bill validator.
    BillTransportMotorFault = 37,

    /// Fault on stacker
    ///
    /// A fault was found on the stacker attached to a bill validator.
    StackerFault = 38,

    /// Bill jammed
    ///
    /// A bill is stuck in the bill validator.
    BillJammed = 39,

    /// RAM test fail
    ///
    /// A read/write test cycle of the bill validator RAM has indicated a fault.
    RamTestFail = 40,

    /// Fault on string sensor
    ///
    /// A fault was found on a sensor used for detecting bills on a string.
    StringSensorFault = 41,

    /// Accept gate failed open
    ///
    /// The coin accept gate is stuck open due to a jam or fraud attempt.
    AcceptGateFailedOpen = 42,

    /// Accept gate failed closed
    ///
    /// The coin accept gate is stuck closed. Possible open-circuit fault on the solenoid driver.
    AcceptGateFailedClosed = 43,

    /// Stacker missing
    ///
    /// The stacker is not fitted and needs to be for notes to accept.
    StackerMissing = 44,

    /// Stacker full
    ///
    /// The stacker is full and needs emptying.
    StackerFull = 45,

    /// Flash memory erase fail
    ///
    /// The last flash memory erase cycle did not complete successfully.
    FlashMemoryEraseFail = 46,

    /// Flash memory write fail
    ///
    /// The last flash memory write cycle did not complete successfully.
    FlashMemoryWriteFail = 47,

    /// Slave device not responding
    ///
    /// If an attached device acts as a host to other slave devices then this fault
    /// code indicates a failure to communicate with those other devices.
    /// Optional extra info: Device number
    SlaveDeviceNotResponding = 48,

    /// Fault on opto sensor
    ///
    /// A fault was detected on an opto-electronic sensor.
    /// Optional extra info: Opto number
    OptoSensorFault = 49,

    /// Battery fault
    ///
    /// A system battery is missing or low on charge and requires replacing/recharging.
    BatteryFault = 50,

    /// Door open
    ///
    /// A door on the system was left in the open position. It must be shut to continue.
    DoorOpen = 51,

    /// Microswitch fault
    ///
    /// A fault was detected on a microswitch.
    MicroswitchFault = 52,

    /// RTC fault
    ///
    /// A fault was detected on the Real Time Clock.
    RtcFault = 53,

    /// Firmware error
    ///
    /// A non-checksum type error was detected in the firmware or the firmware of
    /// an attached peripheral. Self-calculated checksum errors should be reported
    /// in fault code 30.
    FirmwareError = 54,

    /// Initialisation error
    ///
    /// An initialisation error was detected in the peripheral on power-up.
    /// The optional extra info field can break this down further if required.
    InitialisationError = 55,

    /// Supply current outside operating limits
    ///
    /// The ccTalk device is operating outside supply current limits defined in
    /// the product specification. There may be a short or a faulty component.
    SupplyCurrentOutsideLimits = 56,

    /// Forced bootloader mode
    ///
    /// An external input was used to force the device into a bootloader mode.
    /// From here firmware can be re-programmed and the device reset. This is a
    /// special mode which stops the main application firmware from running.
    ForcedBootloaderMode = 57,

    /// Unspecified fault code
    ///
    /// Any fault code which does not fall into the above categories. Some manufacturers
    /// may wish to use the optional byte to specify a manufacturer-specific fault code.
    /// Optional extra info: Further information
    UnspecifiedFault = 255,
}

impl FaultCode {
    /// Returns true if this fault code can have optional extra information
    ///
    /// Some fault codes support an additional byte of information to provide
    /// more specific details about the fault (e.g., which sensor failed).
    pub fn has_optional_info(&self) -> bool {
        matches!(
            self,
            FaultCode::InductiveCoilsFault
                | FaultCode::SorterExitSensorsFault
                | FaultCode::LowLevelSensorError
                | FaultCode::HighLevelSensorError
                | FaultCode::KeypadError
                | FaultCode::PayoutMotorFault
                | FaultCode::PayoutTimeout
                | FaultCode::PayoutJammed
                | FaultCode::PayoutSensorFault
                | FaultCode::LevelSensorError
                | FaultCode::MissingSlaveDevice
                | FaultCode::InternalCommsBad
                | FaultCode::DceFault
                | FaultCode::BillValidationSensorFault
                | FaultCode::SlaveDeviceNotResponding
                | FaultCode::OptoSensorFault
                | FaultCode::UnspecifiedFault
        )
    }

    /// Returns true if this fault code is marked as obsolete in the specification
    ///
    /// Obsolete fault codes were incorporated into the 'Test hopper' command
    /// in a past revision of the protocol.
    pub fn is_obsolete(&self) -> bool {
        matches!(
            self,
            FaultCode::CoinDispensingError
                | FaultCode::LowLevelSensorError
                | FaultCode::HighLevelSensorError
                | FaultCode::CoinCountingError
                | FaultCode::PayoutTimeout
                | FaultCode::PayoutJammed
                | FaultCode::MissingSlaveDevice
        )
    }

    /// Returns true if this fault code indicates normal operation (no fault)
    pub fn is_ok(&self) -> bool {
        matches!(self, FaultCode::Ok)
    }

    /// Returns true if this fault code indicates a fatal error requiring service
    ///
    /// All non-zero fault codes are considered fatal and prevent normal operation.
    pub fn is_fatal(&self) -> bool {
        !self.is_ok()
    }
}

impl TryFrom<u8> for FaultCode {
    type Error = InvalidFaultCode;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FaultCode::Ok),
            1 => Ok(FaultCode::EepromChecksumCorrupted),
            2 => Ok(FaultCode::InductiveCoilsFault),
            3 => Ok(FaultCode::CreditSensorFault),
            4 => Ok(FaultCode::PiezoSensorFault),
            5 => Ok(FaultCode::ReflectiveSensorFault),
            6 => Ok(FaultCode::DiameterSensorFault),
            7 => Ok(FaultCode::WakeUpSensorFault),
            8 => Ok(FaultCode::SorterExitSensorsFault),
            9 => Ok(FaultCode::NvramChecksumCorrupted),
            10 => Ok(FaultCode::CoinDispensingError),
            11 => Ok(FaultCode::LowLevelSensorError),
            12 => Ok(FaultCode::HighLevelSensorError),
            13 => Ok(FaultCode::CoinCountingError),
            14 => Ok(FaultCode::KeypadError),
            15 => Ok(FaultCode::ButtonError),
            16 => Ok(FaultCode::DisplayError),
            17 => Ok(FaultCode::CoinAuditingError),
            18 => Ok(FaultCode::RejectSensorFault),
            19 => Ok(FaultCode::CoinReturnMechanismFault),
            20 => Ok(FaultCode::CosMechanismFault),
            21 => Ok(FaultCode::RimSensorFault),
            22 => Ok(FaultCode::ThermistorFault),
            23 => Ok(FaultCode::PayoutMotorFault),
            24 => Ok(FaultCode::PayoutTimeout),
            25 => Ok(FaultCode::PayoutJammed),
            26 => Ok(FaultCode::PayoutSensorFault),
            27 => Ok(FaultCode::LevelSensorError),
            28 => Ok(FaultCode::PersonalityModuleNotFitted),
            29 => Ok(FaultCode::PersonalityChecksumCorrupted),
            30 => Ok(FaultCode::RomChecksumMismatch),
            31 => Ok(FaultCode::MissingSlaveDevice),
            32 => Ok(FaultCode::InternalCommsBad),
            33 => Ok(FaultCode::SupplyVoltageOutsideLimits),
            34 => Ok(FaultCode::TemperatureOutsideLimits),
            35 => Ok(FaultCode::DceFault),
            36 => Ok(FaultCode::BillValidationSensorFault),
            37 => Ok(FaultCode::BillTransportMotorFault),
            38 => Ok(FaultCode::StackerFault),
            39 => Ok(FaultCode::BillJammed),
            40 => Ok(FaultCode::RamTestFail),
            41 => Ok(FaultCode::StringSensorFault),
            42 => Ok(FaultCode::AcceptGateFailedOpen),
            43 => Ok(FaultCode::AcceptGateFailedClosed),
            44 => Ok(FaultCode::StackerMissing),
            45 => Ok(FaultCode::StackerFull),
            46 => Ok(FaultCode::FlashMemoryEraseFail),
            47 => Ok(FaultCode::FlashMemoryWriteFail),
            48 => Ok(FaultCode::SlaveDeviceNotResponding),
            49 => Ok(FaultCode::OptoSensorFault),
            50 => Ok(FaultCode::BatteryFault),
            51 => Ok(FaultCode::DoorOpen),
            52 => Ok(FaultCode::MicroswitchFault),
            53 => Ok(FaultCode::RtcFault),
            54 => Ok(FaultCode::FirmwareError),
            55 => Ok(FaultCode::InitialisationError),
            56 => Ok(FaultCode::SupplyCurrentOutsideLimits),
            57 => Ok(FaultCode::ForcedBootloaderMode),
            255 => Ok(FaultCode::UnspecifiedFault),
            _ => Err(InvalidFaultCode(value)),
        }
    }
}

impl From<FaultCode> for u8 {
    fn from(fault: FaultCode) -> Self {
        fault as u8
    }
}

/// Error type for invalid fault codes
///
/// Returned when attempting to convert a u8 value that doesn't correspond
/// to a valid ccTalk fault code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidFaultCode(pub u8);

impl core::fmt::Display for InvalidFaultCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Invalid ccTalk fault code: {}", self.0)
    }
}

/// Represents a ccTalk fault with optional extra information
///
/// Some fault codes can include an additional byte of information to provide
/// more specific details about the fault condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fault {
    pub code: FaultCode,
    pub extra_info: Option<u8>,
}

impl Fault {
    /// Creates a new fault with no extra information
    pub fn new(code: FaultCode) -> Self {
        Self {
            code,
            extra_info: None,
        }
    }

    /// Creates a new fault with extra information
    ///
    /// # Panics
    ///
    /// Panics if the fault code doesn't support optional extra information.
    /// Use `try_with_info` for a non-panicking version.
    pub fn with_info(code: FaultCode, info: u8) -> Self {
        assert!(
            code.has_optional_info(),
            "Fault code {:?} does not support optional extra information",
            code
        );
        Self {
            code,
            extra_info: Some(info),
        }
    }

    /// Attempts to create a new fault with extra information
    ///
    /// Returns `Err` if the fault code doesn't support optional extra information.
    pub fn try_with_info(code: FaultCode, info: u8) -> Result<Self, &'static str> {
        if code.has_optional_info() {
            Ok(Self {
                code,
                extra_info: Some(info),
            })
        } else {
            Err("Fault code does not support optional extra information")
        }
    }

    /// Returns true if this fault indicates normal operation
    pub fn is_ok(&self) -> bool {
        self.code.is_ok()
    }

    /// Returns true if this fault indicates a fatal error requiring service
    pub fn is_fatal(&self) -> bool {
        self.code.is_fatal()
    }
}
