/// ccTalk Coin Calibration Reply Codes as defined in the ccTalk Generic Specification
/// Table 5 - ccTalk Coin Calibration Reply Codes
///
/// These codes are returned in response to coin calibration operations.
/// See obsolete header 200, 'Upload coin data' in part 4 of the specification.
///
/// Note: Code 0 (success) is not explicitly listed in the specification table
/// but is implied as the successful response when no error occurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CoinCalibrationReplyCode {
    /// Calibration completed successfully (implied)
    ///
    /// This code is not explicitly listed in the specification but represents
    /// a successful calibration when no error code is returned.
    Success = 0,

    /// Calibration denied
    ///
    /// The calibration operation was denied by the device.
    CalibrationDenied = 1,

    /// Calibration recharge required
    ///
    /// The device requires a recharge operation before calibration can proceed.
    CalibrationRechargeRequired = 2,

    /// Calibration failed (product name mismatch)
    ///
    /// The calibration failed because the product name in the calibration data
    /// does not match the device's expected product name.
    CalibrationFailedProductNameMismatch = 3,

    /// Calibration failed (database number mismatch)
    ///
    /// The calibration failed because the database number in the calibration data
    /// does not match the device's expected database number.
    CalibrationFailedDatabaseNumberMismatch = 4,

    /// Calibration error (key not supported)
    ///
    /// The calibration operation failed because the specified key is not supported
    /// by this device.
    CalibrationErrorKeyNotSupported = 250,

    /// Calibration error (internal bin failure)
    ///
    /// The calibration operation failed due to an internal bin failure.
    CalibrationErrorInternalBinFailure = 251,

    /// Calibration error (op-code not supported)
    ///
    /// The calibration operation failed because the specified operation code
    /// is not supported by this device.
    CalibrationErrorOpCodeNotSupported = 252,

    /// Calibration error (illegal parameter)
    ///
    /// The calibration operation failed due to an illegal parameter in the
    /// calibration data.
    CalibrationErrorIllegalParameter = 253,

    /// Calibration error (database corrupt)
    ///
    /// The calibration operation failed because the calibration database
    /// is corrupt.
    CalibrationErrorDatabaseCorrupt = 254,

    /// Calibration error (unspecified)
    ///
    /// An unspecified calibration error occurred that doesn't fall into
    /// any of the other error categories.
    CalibrationErrorUnspecified = 255,
}

impl CoinCalibrationReplyCode {
    /// Returns true if the calibration was successful
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Returns true if this represents a calibration error
    #[must_use]
    pub const fn is_error(&self) -> bool {
        !self.is_success()
    }

    /// Returns true if this is a calibration failure (codes 1-4)
    ///
    /// These are operational failures as opposed to internal errors.
    #[must_use]
    pub const fn is_calibration_failure(&self) -> bool {
        matches!(
            self,
            Self::CalibrationDenied
                | Self::CalibrationRechargeRequired
                | Self::CalibrationFailedProductNameMismatch
                | Self::CalibrationFailedDatabaseNumberMismatch
        )
    }

    /// Returns true if this is an internal calibration error (codes 250-255)
    ///
    /// These represent system-level errors during calibration.
    #[must_use]
    pub const fn is_internal_error(&self) -> bool {
        matches!(
            self,
            Self::CalibrationErrorKeyNotSupported
                | Self::CalibrationErrorInternalBinFailure
                | Self::CalibrationErrorOpCodeNotSupported
                | Self::CalibrationErrorIllegalParameter
                | Self::CalibrationErrorDatabaseCorrupt
                | Self::CalibrationErrorUnspecified
        )
    }

    /// Returns a human-readable description of the error
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Success => "calibration completed successfully",
            Self::CalibrationDenied => "calibration denied",
            Self::CalibrationRechargeRequired => "calibration recharge required",
            Self::CalibrationFailedProductNameMismatch => {
                "calibration failed (product name mismatch)"
            }
            Self::CalibrationFailedDatabaseNumberMismatch => {
                "calibration failed (database number mismatch)"
            }
            Self::CalibrationErrorKeyNotSupported => "calibration error (key not supported)",
            Self::CalibrationErrorInternalBinFailure => "calibration error (internal bin failure)",
            Self::CalibrationErrorOpCodeNotSupported => "calibration error (op-code not supported)",
            Self::CalibrationErrorIllegalParameter => "calibration error (illegal parameter)",
            Self::CalibrationErrorDatabaseCorrupt => "calibration error (database corrupt)",
            Self::CalibrationErrorUnspecified => "calibration error (unspecified)",
        }
    }
}

impl TryFrom<u8> for CoinCalibrationReplyCode {
    type Error = InvalidCalibrationReplyCode;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            1 => Ok(Self::CalibrationDenied),
            2 => Ok(Self::CalibrationRechargeRequired),
            3 => Ok(Self::CalibrationFailedProductNameMismatch),
            4 => Ok(Self::CalibrationFailedDatabaseNumberMismatch),
            250 => Ok(Self::CalibrationErrorKeyNotSupported),
            251 => Ok(Self::CalibrationErrorInternalBinFailure),
            252 => Ok(Self::CalibrationErrorOpCodeNotSupported),
            253 => Ok(Self::CalibrationErrorIllegalParameter),
            254 => Ok(Self::CalibrationErrorDatabaseCorrupt),
            255 => Ok(Self::CalibrationErrorUnspecified),
            _ => Err(InvalidCalibrationReplyCode(value)),
        }
    }
}

impl From<CoinCalibrationReplyCode> for u8 {
    fn from(code: CoinCalibrationReplyCode) -> Self {
        code as Self
    }
}

impl core::fmt::Display for CoinCalibrationReplyCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Error type for invalid calibration reply codes
///
/// Returned when attempting to convert a u8 value that doesn't correspond
/// to a valid ccTalk coin calibration reply code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidCalibrationReplyCode(pub u8);

impl core::fmt::Display for InvalidCalibrationReplyCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Invalid ccTalk coin calibration reply code: {}", self.0)
    }
}
