/// ccTalk Coin Acceptor Error Codes
///
/// Represents the standardized error codes for ccTalk coin acceptors as defined in the
/// ccTalk Generic Specification (Part 3 v4.7) by Crane Payment Solutions.
///
/// These error codes are specific to coin acceptors and can be reported at any time in a
/// reply to a host credit poll. Not all coin acceptor manufacturers implement every error
/// code - the subset depends on the technology used and the device's self-diagnostic
/// capabilities.
///
/// # Error Categories
///
/// - **Coin Rejection**: Errors that definitely result in coin rejection
/// - **Possible Rejection**: Errors that may result in coin rejection
/// - **Hardware Issues**: Sensor blockages, timeouts, and mechanical failures
/// - **Fraud Detection**: Security-related errors indicating potential fraud attempts
/// - **Configuration**: Product configuration errors
/// - **Special Codes**: Data requests and unspecified alarms
///
/// # Usage
///
/// ```rust
/// use std::convert::TryFrom;
/// use cc_talk_core::cc_talk::CoinAcceptorError;
///
/// // Convert from raw error code
/// let error = CoinAcceptorError::try_from(1).unwrap();
/// assert_eq!(error, CoinAcceptorError::RejectCoin);
/// assert!(error.is_coin_rejected());
///
/// // Convert to raw error code
/// let code: u8 = CoinAcceptorError::ValidationTimeout.into();
/// assert_eq!(code, 5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CoinAcceptorError {
    /// No error occurred (code 0)
    ///
    /// This is the default state indicating no error condition is present.
    #[error("no error occurred")]
    NullEvent = 0,

    /// Coin rejected - did not match any programmed coin type (code 1)
    ///
    /// A coin was inserted that did not match any of the programmed coin types.
    /// The coin is returned to the customer and no credit is given.
    #[error("coin rejected - did not match any programmed coin type")]
    RejectCoin = 1,

    /// Coin rejected - inhibited by inhibit register (code 2)
    ///
    /// A coin was inserted that matched a programmed window type but was prevented
    /// from acceptance by the inhibit register. The inhibit register can be controlled
    /// serially or may be linked to external DIL switches.
    #[error("coin rejected - inserted coin is inhibited")]
    InhibitedCoin = 2,

    /// Coin rejected - matched multiple enabled window types (code 3)
    ///
    /// A coin was inserted that matched more than one enabled window type.
    /// The coin was rejected because the credit code was indeterminate.
    #[error("coin rejected - matched multiple enabled window types")]
    MultipleWindow = 3,

    /// Wake-up sensor timeout - possible coin jam (code 4)
    ///
    /// A coin acceptor fitted with a wake-up sensor detected a coin entering
    /// the acceptor but it was not subsequently seen in the validation area.
    /// This may indicate a coin jam.
    #[error("wake-up sensor timeout - possible coin jam")]
    WakeUpTimeout = 4,

    /// Validation area timeout - possible coin jam (code 5)
    ///
    /// A coin was detected entering the validation area but failed to leave it.
    /// This may indicate a coin jam.
    #[error("validation area timeout - possible coin jam")]
    ValidationTimeout = 5,

    /// Credit sensor timeout - possible coin jam (code 6)
    ///
    /// A coin was validated as genuine but never reached the post-gate credit sensor.
    /// This may indicate a coin jam.
    #[error("credit sensor timeout - possible coin jam")]
    CreditSensorTimeout = 6,

    /// Sorter optical sensor timeout - possible coin jam (code 7)
    ///
    /// A coin was sent into the sorter/diverter but was not detected coming out.
    /// This may indicate a coin jam.
    #[error("sorter optical sensor timeout - possible coin jam")]
    SorterOptoTimeout = 7,

    /// Second coin inserted too close to first - rejection (code 8)
    ///
    /// A coin was inserted too close to the preceding coin. One or both coins
    /// will be rejected.
    #[error("second coin inserted too close to first")]
    SecondCloseCoinError = 8,

    /// Accept gate not ready - coins inserted too quickly (code 9)
    ///
    /// A coin was inserted while the accept gate for the preceding coin was
    /// still operating. Coins have been inserted too quickly.
    #[error("accept gate not ready - coins inserted too quickly")]
    AcceptGateNotReady = 9,

    /// Credit sensor not ready - coins inserted too quickly (code 10)
    ///
    /// A coin was still over the credit sensor when another coin was ready
    /// to be accepted. Coins have been inserted too quickly.
    #[error("credit sensor not ready - coins inserted too quickly")]
    CreditSensorNotReady = 10,

    /// Sorter not ready - coins inserted too quickly (code 11)
    ///
    /// A coin was inserted while the sorter flaps for the preceding coin
    /// were still operating. Coins have been inserted too quickly.
    #[error("sorter not ready - coins inserted too quickly")]
    SorterNotReady = 11,

    /// Previous rejected coin not cleared - coins inserted too quickly (code 12)
    ///
    /// A coin was inserted before a previously rejected coin had time to
    /// clear the coin acceptor. Coins have been inserted too quickly.
    #[error("previous rejected coin not cleared")]
    RejectCoinNotCleared = 12,

    /// Validation sensor not ready - possible developing fault (code 13)
    ///
    /// The validator inductive sensors were not ready for coin validation.
    /// This may indicate a developing fault.
    #[error("validation sensor not ready - possible developing fault")]
    ValidationSensorNotReady = 13,

    /// Credit sensor permanently blocked (code 14)
    ///
    /// There is a permanent blockage at the credit sensor. The coin acceptor
    /// will not accept any more coins until cleared.
    #[error("credit sensor permanently blocked")]
    CreditSensorBlocked = 14,

    /// Sorter exit sensor permanently blocked (code 15)
    ///
    /// There is a permanent blockage at the sorter exit sensor. The coin
    /// acceptor will not accept any more coins until cleared.
    #[error("sorter exit sensor permanently blocked")]
    SorterOptoBlocked = 15,

    /// Credit sequence error - possible fraud attempt (code 16)
    ///
    /// A coin or object was detected going backwards through a directional
    /// credit sensor. This may indicate a fraud attempt.
    #[error("credit sequence error - possible fraud attempt")]
    CreditSequenceError = 16,

    /// Coin going backwards - possible fraud attempt (code 17)
    ///
    /// A coin was detected going backwards through the coin acceptor.
    /// This may indicate a fraud attempt.
    #[error("coin going backwards - possible fraud attempt")]
    CoinGoingBackwards = 17,

    /// Coin too fast over credit sensor - possible fraud attempt (code 18)
    ///
    /// A coin was timed going through the credit sensor and was moving too fast.
    /// This may indicate a fraud attempt.
    #[error("coin too fast over credit sensor - possible fraud attempt")]
    CoinTooFastCreditSensor = 18,

    /// Coin too slow over credit sensor - possible fraud attempt (code 19)
    ///
    /// A coin was timed going through the credit sensor and was moving too slow.
    /// This may indicate a fraud attempt.
    #[error("coin too slow over credit sensor - possible fraud attempt")]
    CoinTooSlowCreditSensor = 19,

    /// Coin-on-string mechanism activated - fraud attempt detected (code 20)
    ///
    /// A specific sensor for detecting a 'coin on string' was activated.
    /// This indicates a possible fraud attempt.
    #[error("coin-on-string mechanism activated - fraud attempt detected")]
    CoinOnStringMechanism = 20,

    /// Dual Coin Entry optical timeout - possible coin jam (code 21)
    ///
    /// A coin acceptor fitted with a Dual Coin Entry chute detected a coin
    /// or token that was not subsequently seen in the validation area.
    /// This may indicate a coin jam.
    #[error("dual coin entry optical timeout - possible coin jam")]
    DceOptoTimeout = 21,

    /// Dual Coin Entry optical sensor bypass - possible fraud attempt (code 22)
    ///
    /// A coin acceptor fitted with a Dual Coin Entry chute detected a coin
    /// that was not previously seen by the chute sensor. This may indicate
    /// a fraud attempt.
    #[error("dual coin entry optical sensor bypass - possible fraud attempt")]
    DceOptoNotSeen = 22,

    /// Credit sensor reached too early - possible fraud attempt (code 23)
    ///
    /// A coin was timed from the end of the validation area to the post-gate
    /// credit sensor and arrived too early. This may indicate a fraud attempt.
    #[error("credit sensor reached too early - possible fraud attempt")]
    CreditSensorReachedTooEarly = 23,

    /// Repeated sequential coin rejection - possible fraud attempt (code 24)
    ///
    /// A coin was rejected N times in succession with no intervening genuine
    /// coins. This is statistically unlikely if N ≥ 5 and may indicate a
    /// fraud attempt.
    #[error("repeated sequential coin rejection - possible fraud attempt")]
    RejectCoinRepeatedTrip = 24,

    /// Known slug detected and rejected (code 25)
    ///
    /// A coin was rejected but identified as a known slug type. This may be
    /// a pre-programmed fraud coin or known fraud material.
    #[error("known slug detected and rejected")]
    RejectSlug = 25,

    /// Reject sensor permanently blocked (code 26)
    ///
    /// There is a permanent blockage at the reject sensor. The coin acceptor
    /// will not accept any more coins. Note: Not all coin acceptors have a
    /// reject sensor.
    #[error("reject sensor permanently blocked")]
    RejectSensorBlocked = 26,

    /// Games overload - configuration error (code 27)
    ///
    /// Totaliser mode: A game value was set too low (possibly zero).
    /// This is a product configuration error.
    #[error("games overload - configuration error")]
    GamesOverload = 27,

    /// Maximum coin meter pulses exceeded - configuration error (code 28)
    ///
    /// Totaliser mode: A meter value was set too low (possibly zero).
    /// This is a product configuration error.
    #[error("maximum coin meter pulses exceeded - configuration error")]
    MaxCoinMeterPulsesExceeded = 28,

    /// Accept gate forced open when should be closed (code 29)
    ///
    /// The accept gate was forced open when it should have been closed.
    /// This may indicate tampering or mechanical failure.
    #[error("accept gate forced open when should be closed")]
    AcceptGateOpenNotClosed = 29,

    /// Accept gate failed to open when driven (code 30)
    ///
    /// The accept gate did not open when the solenoid was driven.
    /// This indicates a mechanical or electrical failure.
    #[error("accept gate failed to open when driven")]
    AcceptGateClosedNotOpen = 30,

    /// Manifold optical timeout - possible coin jam (code 31)
    ///
    /// A coin was sent into the manifold module (coin diverter) but was
    /// not detected coming out. This may indicate a coin jam.
    #[error("manifold optical timeout - possible coin jam")]
    ManifoldOptoTimeout = 31,

    /// Manifold sensor permanently blocked (code 32)
    ///
    /// There is a permanent blockage at the manifold module sensor
    /// (coin diverter). The coin acceptor will not accept any more coins.
    #[error("manifold sensor permanently blocked")]
    ManifoldOptoBlocked = 32,

    /// Manifold not ready - coins inserted too quickly (code 33)
    ///
    /// A coin was inserted while the manifold flap for the preceding coin
    /// was still operating. Coins have been inserted too quickly.
    #[error("manifold not ready - coins inserted too quickly")]
    ManifoldNotReady = 33,

    /// Security status changed due to fraud detection (code 34)
    ///
    /// The coin acceptor changed its security status (coin acceptance criteria)
    /// based on the detection of fraudulent activity. Refer to ccTalk header 180
    /// for additional details.
    #[error("security status changed due to fraud detection")]
    SecurityStatusChanged = 34,

    /// Motor exception - mechanical problem (code 35)
    ///
    /// For coin acceptors using a motor, this indicates a motor problem
    /// such as a coin jam or mechanical failure.
    #[error("motor exception - mechanical problem")]
    MotorException = 35,

    /// Swallowed coin - hardware failure or fraud (code 36)
    ///
    /// A coin was detected at the credit sensor when it should have been rejected.
    /// The coin value is unknown. This may indicate fraudulent manipulation of
    /// the accept gate or a hardware failure (gate stuck open).
    #[error("swallowed coin - hardware failure or fraud")]
    SwallowedCoin = 36,

    /// Coin too fast over validation sensor - possible fraud attempt (code 37)
    ///
    /// A coin was timed going through the validation sensor area and was too quick
    #[error("coin too fast over validation sensor - possible fraud attempt")]
    CoinTooFastValidationSensor = 37,

    /// Coin too slow over validation sensor - possible fraud attempt (code 38)
    ///
    /// A coin was timed going through the validation sensor area and was too slow
    #[error("coin too slow over validation sensor - possible fraud attempt")]
    CoinTooSlowValidationSensor = 38,

    /// Coin incorrectly sorted - hardware fault notification (code 39)
    ///
    /// A coin was accepted but sent to the wrong sorter path. This may
    /// indicate a hardware fault. This is a notification event without
    /// further details.
    #[error("coin incorrectly sorted - hardware fault")]
    CoinIncorrectlySorted = 39,

    /// External light attack detected (code 40)
    ///
    /// A sensor detected external light. This may be an attempt to
    /// compromise the optical sensors through fraud.
    #[error("external light attack detected")]
    ExternalLightAttack = 40,

    /// Data block request - attention needed (code 253)
    ///
    /// A mechanism for a coin acceptor to request attention from the host
    /// machine. The device may need data from the host machine or another
    /// peripheral. This is currently an unused mechanism.
    #[error("data block request - attention needed")]
    DataBlockRequest = 253,

    /// Coin return mechanism activated - flight deck opened (code 254)
    ///
    /// An attempt to clear a coin jam by opening the flight deck was detected.
    /// The coin acceptor cannot operate until the flight deck is closed.
    #[error("coin return mechanism activated - flight deck opened")]
    CoinReturnMechanism = 254,

    /// Unspecified alarm code (code 255)
    ///
    /// Any alarm code that does not fit into the above categories.
    /// This is a catch-all for undefined error conditions.
    #[error("unspecified alarm")]
    UnspecifiedAlarm = 255,
}

impl CoinAcceptorError {
    /// Returns `true` if this error code indicates definitive coin rejection
    ///
    /// These errors always result in the coin being rejected and returned
    /// to the customer with no credit given.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cc_talk_core::cc_talk::CoinAcceptorError;
    /// assert!(CoinAcceptorError::RejectCoin.is_coin_rejected());
    /// assert!(CoinAcceptorError::InhibitedCoin.is_coin_rejected());
    /// assert!(!CoinAcceptorError::ValidationTimeout.is_coin_rejected());
    /// ```
    #[must_use]
    pub const fn is_coin_rejected(&self) -> bool {
        matches!(
            self,
            Self::RejectCoin
                | Self::InhibitedCoin
                | Self::MultipleWindow
                | Self::SecondCloseCoinError
                | Self::AcceptGateNotReady
                | Self::CreditSensorNotReady
                | Self::SorterNotReady
                | Self::RejectCoinNotCleared
                | Self::ValidationSensorNotReady
                | Self::CreditSensorBlocked
                | Self::SorterOptoBlocked
                | Self::DceOptoNotSeen
                | Self::RejectCoinRepeatedTrip
                | Self::RejectSlug
                | Self::AcceptGateClosedNotOpen
                | Self::ManifoldOptoBlocked
                | Self::ManifoldNotReady
                | Self::CoinTooFastValidationSensor
                | Self::CoinTooSlowValidationSensor
        )
    }

    /// Returns `true` if this error code might result in coin rejection
    ///
    /// These errors may or may not result in coin rejection, depending on
    /// the specific circumstances and coin acceptor implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cc_talk_core::cc_talk::CoinAcceptorError;
    /// assert!(CoinAcceptorError::ValidationTimeout.is_possible_rejection());
    /// assert!(CoinAcceptorError::WakeUpTimeout.is_possible_rejection());
    /// assert!(!CoinAcceptorError::RejectCoin.is_possible_rejection());
    /// ```
    #[must_use]
    pub const fn is_possible_rejection(&self) -> bool {
        matches!(
            self,
            Self::WakeUpTimeout
                | Self::ValidationTimeout
                | Self::CreditSensorTimeout
                | Self::DceOptoTimeout
                | Self::SecurityStatusChanged
                | Self::MotorException
        )
    }

    /// Returns `true` if this represents no error condition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cc_talk_core::cc_talk::CoinAcceptorError;
    /// assert!(CoinAcceptorError::NullEvent.is_null_event());
    /// assert!(!CoinAcceptorError::RejectCoin.is_null_event());
    /// ```
    #[must_use]
    pub const fn is_null_event(&self) -> bool {
        matches!(self, Self::NullEvent)
    }

    /// Returns `true` if this error indicates a potential fraud attempt
    ///
    /// These errors suggest that someone may be trying to defraud the
    /// coin acceptor through various means.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cc_talk_core::cc_talk::CoinAcceptorError;
    /// assert!(CoinAcceptorError::CoinOnStringMechanism.is_fraud_related());
    /// assert!(CoinAcceptorError::CoinGoingBackwards.is_fraud_related());
    /// assert!(!CoinAcceptorError::ValidationTimeout.is_fraud_related());
    /// ```
    #[must_use]
    pub const fn is_fraud_related(&self) -> bool {
        matches!(
            self,
            Self::CreditSequenceError
                | Self::CoinGoingBackwards
                | Self::CoinTooFastCreditSensor
                | Self::CoinTooSlowCreditSensor
                | Self::CoinOnStringMechanism
                | Self::DceOptoNotSeen
                | Self::CreditSensorReachedTooEarly
                | Self::RejectCoinRepeatedTrip
                | Self::RejectSlug
                | Self::SecurityStatusChanged
                | Self::SwallowedCoin
                | Self::CoinTooFastValidationSensor
                | Self::CoinTooSlowValidationSensor
                | Self::ExternalLightAttack
        )
    }

    /// Returns `true` if this error indicates a hardware or mechanical issue
    ///
    /// These errors suggest problems with sensors, gates, motors, or other
    /// mechanical components of the coin acceptor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cc_talk_core::cc_talk::CoinAcceptorError;
    /// assert!(CoinAcceptorError::CreditSensorBlocked.is_hardware_issue());
    /// assert!(CoinAcceptorError::MotorException.is_hardware_issue());
    /// assert!(!CoinAcceptorError::RejectCoin.is_hardware_issue());
    /// ```
    #[must_use]
    pub const fn is_hardware_issue(&self) -> bool {
        matches!(
            self,
            Self::ValidationSensorNotReady
                | Self::CreditSensorBlocked
                | Self::SorterOptoBlocked
                | Self::RejectSensorBlocked
                | Self::AcceptGateOpenNotClosed
                | Self::AcceptGateClosedNotOpen
                | Self::ManifoldOptoBlocked
                | Self::MotorException
                | Self::CoinIncorrectlySorted
        )
    }

    /// Returns `true` if this error indicates coins are being inserted too quickly
    ///
    /// These errors occur when coins are inserted faster than the acceptor
    /// can process them properly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cc_talk_core::cc_talk::CoinAcceptorError;
    /// assert!(CoinAcceptorError::AcceptGateNotReady.is_timing_issue());
    /// assert!(CoinAcceptorError::SorterNotReady.is_timing_issue());
    /// assert!(!CoinAcceptorError::RejectCoin.is_timing_issue());
    /// ```
    #[must_use]
    pub const fn is_timing_issue(&self) -> bool {
        matches!(
            self,
            Self::SecondCloseCoinError
                | Self::AcceptGateNotReady
                | Self::CreditSensorNotReady
                | Self::SorterNotReady
                | Self::RejectCoinNotCleared
                | Self::ManifoldNotReady
        )
    }

    /// Returns a human-readable description of the error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cc_talk_core::cc_talk::CoinAcceptorError;
    /// let error = CoinAcceptorError::RejectCoin;
    /// assert_eq!(error.description(), "Coin rejected - did not match any programmed coin type");
    /// ```
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::NullEvent => "No error occurred",
            Self::RejectCoin => "Coin rejected - did not match any programmed coin type",
            Self::InhibitedCoin => "Coin rejected - inhibited by inhibit register",
            Self::MultipleWindow => "Coin rejected - matched multiple enabled window types",
            Self::WakeUpTimeout => "Wake-up sensor timeout - possible coin jam",
            Self::ValidationTimeout => "Validation area timeout - possible coin jam",
            Self::CreditSensorTimeout => "Credit sensor timeout - possible coin jam",
            Self::SorterOptoTimeout => "Sorter optical sensor timeout - possible coin jam",
            Self::SecondCloseCoinError => "Second coin inserted too close to first",
            Self::AcceptGateNotReady => "Accept gate not ready - coins inserted too quickly",
            Self::CreditSensorNotReady => "Credit sensor not ready - coins inserted too quickly",
            Self::SorterNotReady => "Sorter not ready - coins inserted too quickly",
            Self::RejectCoinNotCleared => "Previous rejected coin not cleared",
            Self::ValidationSensorNotReady => {
                "Validation sensor not ready - possible developing fault"
            }
            Self::CreditSensorBlocked => "Credit sensor permanently blocked",
            Self::SorterOptoBlocked => "Sorter exit sensor permanently blocked",
            Self::CreditSequenceError => "Credit sequence error - possible fraud attempt",
            Self::CoinGoingBackwards => "Coin going backwards - possible fraud attempt",
            Self::CoinTooFastCreditSensor => {
                "Coin too fast over credit sensor - possible fraud attempt"
            }
            Self::CoinTooSlowCreditSensor => {
                "Coin too slow over credit sensor - possible fraud attempt"
            }
            Self::CoinOnStringMechanism => {
                "Coin-on-string mechanism activated - fraud attempt detected"
            }
            Self::DceOptoTimeout => "Dual Coin Entry optical timeout - possible coin jam",
            Self::DceOptoNotSeen => {
                "Dual Coin Entry optical sensor bypass - possible fraud attempt"
            }
            Self::CreditSensorReachedTooEarly => {
                "Credit sensor reached too early - possible fraud attempt"
            }
            Self::RejectCoinRepeatedTrip => {
                "Repeated sequential coin rejection - possible fraud attempt"
            }
            Self::RejectSlug => "Known slug detected and rejected",
            Self::RejectSensorBlocked => "Reject sensor permanently blocked",
            Self::GamesOverload => "Games overload - configuration error",
            Self::MaxCoinMeterPulsesExceeded => {
                "Maximum coin meter pulses exceeded - configuration error"
            }
            Self::AcceptGateOpenNotClosed => "Accept gate forced open when should be closed",
            Self::AcceptGateClosedNotOpen => "Accept gate failed to open when driven",
            Self::ManifoldOptoTimeout => "Manifold optical timeout - possible coin jam",
            Self::ManifoldOptoBlocked => "Manifold sensor permanently blocked",
            Self::ManifoldNotReady => "Manifold not ready - coins inserted too quickly",
            Self::SecurityStatusChanged => "Security status changed due to fraud detection",
            Self::MotorException => "Motor exception - mechanical problem",
            Self::SwallowedCoin => "Swallowed coin - hardware failure or fraud",
            Self::CoinTooFastValidationSensor => {
                "Coin too fast over validation sensor - possible fraud attempt"
            }
            Self::CoinTooSlowValidationSensor => {
                "Coin too slow over validation sensor - possible fraud attempt"
            }
            Self::CoinIncorrectlySorted => "Coin incorrectly sorted - hardware fault notification",
            Self::ExternalLightAttack => "External light attack detected",
            Self::DataBlockRequest => "Data block request - attention needed",
            Self::CoinReturnMechanism => "Coin return mechanism activated - flight deck opened",
            Self::UnspecifiedAlarm => "Unspecified alarm code",
        }
    }
}

impl TryFrom<u8> for CoinAcceptorError {
    type Error = ();

    /// Converts a raw error code to a `CoinAcceptorError`
    ///
    /// Returns `Err(())` if the code is not a valid ccTalk error code.
    /// Codes 128-159 are all mapped to `InhibitedCoin` as they represent
    /// inhibited coins of types 1-32.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::convert::TryFrom;
    /// use cc_talk_core::cc_talk::CoinAcceptorError;
    ///
    /// assert_eq!(CoinAcceptorError::try_from(1), Ok(CoinAcceptorError::RejectCoin));
    /// assert_eq!(CoinAcceptorError::try_from(128), Ok(CoinAcceptorError::InhibitedCoin));
    /// assert_eq!(CoinAcceptorError::try_from(100), Err(()));
    /// ```
    fn try_from(code: u8) -> Result<Self, ()> {
        match code {
            0 => Ok(Self::NullEvent),
            1 => Ok(Self::RejectCoin),
            2 | 128..=159 => Ok(Self::InhibitedCoin), // Type 1-32 inhibited coins
            3 => Ok(Self::MultipleWindow),
            4 => Ok(Self::WakeUpTimeout),
            5 => Ok(Self::ValidationTimeout),
            6 => Ok(Self::CreditSensorTimeout),
            7 => Ok(Self::SorterOptoTimeout),
            8 => Ok(Self::SecondCloseCoinError),
            9 => Ok(Self::AcceptGateNotReady),
            10 => Ok(Self::CreditSensorNotReady),
            11 => Ok(Self::SorterNotReady),
            12 => Ok(Self::RejectCoinNotCleared),
            13 => Ok(Self::ValidationSensorNotReady),
            14 => Ok(Self::CreditSensorBlocked),
            15 => Ok(Self::SorterOptoBlocked),
            16 => Ok(Self::CreditSequenceError),
            17 => Ok(Self::CoinGoingBackwards),
            18 => Ok(Self::CoinTooFastCreditSensor),
            19 => Ok(Self::CoinTooSlowCreditSensor),
            20 => Ok(Self::CoinOnStringMechanism),
            21 => Ok(Self::DceOptoTimeout),
            22 => Ok(Self::DceOptoNotSeen),
            23 => Ok(Self::CreditSensorReachedTooEarly),
            24 => Ok(Self::RejectCoinRepeatedTrip),
            25 => Ok(Self::RejectSlug),
            26 => Ok(Self::RejectSensorBlocked),
            27 => Ok(Self::GamesOverload),
            28 => Ok(Self::MaxCoinMeterPulsesExceeded),
            29 => Ok(Self::AcceptGateOpenNotClosed),
            30 => Ok(Self::AcceptGateClosedNotOpen),
            31 => Ok(Self::ManifoldOptoTimeout),
            32 => Ok(Self::ManifoldOptoBlocked),
            33 => Ok(Self::ManifoldNotReady),
            34 => Ok(Self::SecurityStatusChanged),
            35 => Ok(Self::MotorException),
            36 => Ok(Self::SwallowedCoin),
            37 => Ok(Self::CoinTooFastValidationSensor),
            38 => Ok(Self::CoinTooSlowValidationSensor),
            39 => Ok(Self::CoinIncorrectlySorted),
            40 => Ok(Self::ExternalLightAttack),
            253 => Ok(Self::DataBlockRequest),
            254 => Ok(Self::CoinReturnMechanism),
            255 => Ok(Self::UnspecifiedAlarm),
            _ => Err(()),
        }
    }
}

impl From<CoinAcceptorError> for u8 {
    /// Converts a `CoinAcceptorError` to its raw error code
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cc_talk_core::cc_talk::CoinAcceptorError;
    /// let code: u8 = CoinAcceptorError::RejectCoin.into();
    /// assert_eq!(code, 1);
    /// ```
    fn from(error: CoinAcceptorError) -> Self {
        error as Self
    }
}
