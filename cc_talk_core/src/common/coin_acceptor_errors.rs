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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CoinAcceptorError {
    /// No error occurred (code 0)
    ///
    /// This is the default state indicating no error condition is present.
    NullEvent = 0,

    /// Coin rejected - did not match any programmed coin type (code 1)
    ///
    /// A coin was inserted that did not match any of the programmed coin types.
    /// The coin is returned to the customer and no credit is given.
    RejectCoin = 1,

    /// Coin rejected - inhibited by inhibit register (code 2)
    ///
    /// A coin was inserted that matched a programmed window type but was prevented
    /// from acceptance by the inhibit register. The inhibit register can be controlled
    /// serially or may be linked to external DIL switches.
    InhibitedCoin = 2,

    /// Coin rejected - matched multiple enabled window types (code 3)
    ///
    /// A coin was inserted that matched more than one enabled window type.
    /// The coin was rejected because the credit code was indeterminate.
    MultipleWindow = 3,

    /// Wake-up sensor timeout - possible coin jam (code 4)
    ///
    /// A coin acceptor fitted with a wake-up sensor detected a coin entering
    /// the acceptor but it was not subsequently seen in the validation area.
    /// This may indicate a coin jam.
    WakeUpTimeout = 4,

    /// Validation area timeout - possible coin jam (code 5)
    ///
    /// A coin was detected entering the validation area but failed to leave it.
    /// This may indicate a coin jam.
    ValidationTimeout = 5,

    /// Credit sensor timeout - possible coin jam (code 6)
    ///
    /// A coin was validated as genuine but never reached the post-gate credit sensor.
    /// This may indicate a coin jam.
    CreditSensorTimeout = 6,

    /// Sorter optical sensor timeout - possible coin jam (code 7)
    ///
    /// A coin was sent into the sorter/diverter but was not detected coming out.
    /// This may indicate a coin jam.
    SorterOptoTimeout = 7,

    /// Second coin inserted too close to first - rejection (code 8)
    ///
    /// A coin was inserted too close to the preceding coin. One or both coins
    /// will be rejected.
    SecondCloseCoinError = 8,

    /// Accept gate not ready - coins inserted too quickly (code 9)
    ///
    /// A coin was inserted while the accept gate for the preceding coin was
    /// still operating. Coins have been inserted too quickly.
    AcceptGateNotReady = 9,

    /// Credit sensor not ready - coins inserted too quickly (code 10)
    ///
    /// A coin was still over the credit sensor when another coin was ready
    /// to be accepted. Coins have been inserted too quickly.
    CreditSensorNotReady = 10,

    /// Sorter not ready - coins inserted too quickly (code 11)
    ///
    /// A coin was inserted while the sorter flaps for the preceding coin
    /// were still operating. Coins have been inserted too quickly.
    SorterNotReady = 11,

    /// Previous rejected coin not cleared - coins inserted too quickly (code 12)
    ///
    /// A coin was inserted before a previously rejected coin had time to
    /// clear the coin acceptor. Coins have been inserted too quickly.
    RejectCoinNotCleared = 12,

    /// Validation sensor not ready - possible developing fault (code 13)
    ///
    /// The validator inductive sensors were not ready for coin validation.
    /// This may indicate a developing fault.
    ValidationSensorNotReady = 13,

    /// Credit sensor permanently blocked (code 14)
    ///
    /// There is a permanent blockage at the credit sensor. The coin acceptor
    /// will not accept any more coins until cleared.
    CreditSensorBlocked = 14,

    /// Sorter exit sensor permanently blocked (code 15)
    ///
    /// There is a permanent blockage at the sorter exit sensor. The coin
    /// acceptor will not accept any more coins until cleared.
    SorterOptoBlocked = 15,

    /// Credit sequence error - possible fraud attempt (code 16)
    ///
    /// A coin or object was detected going backwards through a directional
    /// credit sensor. This may indicate a fraud attempt.
    CreditSequenceError = 16,

    /// Coin going backwards - possible fraud attempt (code 17)
    ///
    /// A coin was detected going backwards through the coin acceptor.
    /// This may indicate a fraud attempt.
    CoinGoingBackwards = 17,

    /// Coin too fast over credit sensor - possible fraud attempt (code 18)
    ///
    /// A coin was timed going through the credit sensor and was moving too fast.
    /// This may indicate a fraud attempt.
    CoinTooFastCreditSensor = 18,

    /// Coin too slow over credit sensor - possible fraud attempt (code 19)
    ///
    /// A coin was timed going through the credit sensor and was moving too slow.
    /// This may indicate a fraud attempt.
    CoinTooSlowCreditSensor = 19,

    /// Coin-on-string mechanism activated - fraud attempt detected (code 20)
    ///
    /// A specific sensor for detecting a 'coin on string' was activated.
    /// This indicates a possible fraud attempt.
    CoinOnStringMechanism = 20,

    /// Dual Coin Entry optical timeout - possible coin jam (code 21)
    ///
    /// A coin acceptor fitted with a Dual Coin Entry chute detected a coin
    /// or token that was not subsequently seen in the validation area.
    /// This may indicate a coin jam.
    DceOptoTimeout = 21,

    /// Dual Coin Entry optical sensor bypass - possible fraud attempt (code 22)
    ///
    /// A coin acceptor fitted with a Dual Coin Entry chute detected a coin
    /// that was not previously seen by the chute sensor. This may indicate
    /// a fraud attempt.
    DceOptoNotSeen = 22,

    /// Credit sensor reached too early - possible fraud attempt (code 23)
    ///
    /// A coin was timed from the end of the validation area to the post-gate
    /// credit sensor and arrived too early. This may indicate a fraud attempt.
    CreditSensorReachedTooEarly = 23,

    /// Repeated sequential coin rejection - possible fraud attempt (code 24)
    ///
    /// A coin was rejected N times in succession with no intervening genuine
    /// coins. This is statistically unlikely if N ≥ 5 and may indicate a
    /// fraud attempt.
    RejectCoinRepeatedTrip = 24,

    /// Known slug detected and rejected (code 25)
    ///
    /// A coin was rejected but identified as a known slug type. This may be
    /// a pre-programmed fraud coin or known fraud material.
    RejectSlug = 25,

    /// Reject sensor permanently blocked (code 26)
    ///
    /// There is a permanent blockage at the reject sensor. The coin acceptor
    /// will not accept any more coins. Note: Not all coin acceptors have a
    /// reject sensor.
    RejectSensorBlocked = 26,

    /// Games overload - configuration error (code 27)
    ///
    /// Totaliser mode: A game value was set too low (possibly zero).
    /// This is a product configuration error.
    GamesOverload = 27,

    /// Maximum coin meter pulses exceeded - configuration error (code 28)
    ///
    /// Totaliser mode: A meter value was set too low (possibly zero).
    /// This is a product configuration error.
    MaxCoinMeterPulsesExceeded = 28,

    /// Accept gate forced open when should be closed (code 29)
    ///
    /// The accept gate was forced open when it should have been closed.
    /// This may indicate tampering or mechanical failure.
    AcceptGateOpenNotClosed = 29,

    /// Accept gate failed to open when driven (code 30)
    ///
    /// The accept gate did not open when the solenoid was driven.
    /// This indicates a mechanical or electrical failure.
    AcceptGateClosedNotOpen = 30,

    /// Manifold optical timeout - possible coin jam (code 31)
    ///
    /// A coin was sent into the manifold module (coin diverter) but was
    /// not detected coming out. This may indicate a coin jam.
    ManifoldOptoTimeout = 31,

    /// Manifold sensor permanently blocked (code 32)
    ///
    /// There is a permanent blockage at the manifold module sensor
    /// (coin diverter). The coin acceptor will not accept any more coins.
    ManifoldOptoBlocked = 32,

    /// Manifold not ready - coins inserted too quickly (code 33)
    ///
    /// A coin was inserted while the manifold flap for the preceding coin
    /// was still operating. Coins have been inserted too quickly.
    ManifoldNotReady = 33,

    /// Security status changed due to fraud detection (code 34)
    ///
    /// The coin acceptor changed its security status (coin acceptance criteria)
    /// based on the detection of fraudulent activity. Refer to ccTalk header 180
    /// for additional details.
    SecurityStatusChanged = 34,

    /// Motor exception - mechanical problem (code 35)
    ///
    /// For coin acceptors using a motor, this indicates a motor problem
    /// such as a coin jam or mechanical failure.
    MotorException = 35,

    /// Swallowed coin - hardware failure or fraud (code 36)
    ///
    /// A coin was detected at the credit sensor when it should have been rejected.
    /// The coin value is unknown. This may indicate fraudulent manipulation of
    /// the accept gate or a hardware failure (gate stuck open).
    SwallowedCoin = 36,

    /// Coin too fast over validation sensor - possible fraud attempt (code 37)
    ///
    /// A coin was timed going through the validation sensor area and was too quick
    CoinTooFastValidationSensor = 37,

    /// Coin too slow over validation sensor - possible fraud attempt (code 38)
    ///
    /// A coin was timed going through the validation sensor area and was too slow
    CoinTooSlowValidationSensor = 38,

    /// Coin incorrectly sorted - hardware fault notification (code 39)
    ///
    /// A coin was accepted but sent to the wrong sorter path. This may
    /// indicate a hardware fault. This is a notification event without
    /// further details.
    CoinIncorrectlySorted = 39,

    /// External light attack detected (code 40)
    ///
    /// A sensor detected external light. This may be an attempt to
    /// compromise the optical sensors through fraud.
    ExternalLightAttack = 40,

    /// Data block request - attention needed (code 253)
    ///
    /// A mechanism for a coin acceptor to request attention from the host
    /// machine. The device may need data from the host machine or another
    /// peripheral. This is currently an unused mechanism.
    DataBlockRequest = 253,

    /// Coin return mechanism activated - flight deck opened (code 254)
    ///
    /// An attempt to clear a coin jam by opening the flight deck was detected.
    /// The coin acceptor cannot operate until the flight deck is closed.
    CoinReturnMechanism = 254,

    /// Unspecified alarm code (code 255)
    ///
    /// Any alarm code that does not fit into the above categories.
    /// This is a catch-all for undefined error conditions.
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
    pub fn is_coin_rejected(&self) -> bool {
        matches!(
            self,
            CoinAcceptorError::RejectCoin
                | CoinAcceptorError::InhibitedCoin
                | CoinAcceptorError::MultipleWindow
                | CoinAcceptorError::SecondCloseCoinError
                | CoinAcceptorError::AcceptGateNotReady
                | CoinAcceptorError::CreditSensorNotReady
                | CoinAcceptorError::SorterNotReady
                | CoinAcceptorError::RejectCoinNotCleared
                | CoinAcceptorError::ValidationSensorNotReady
                | CoinAcceptorError::CreditSensorBlocked
                | CoinAcceptorError::SorterOptoBlocked
                | CoinAcceptorError::DceOptoNotSeen
                | CoinAcceptorError::RejectCoinRepeatedTrip
                | CoinAcceptorError::RejectSlug
                | CoinAcceptorError::AcceptGateClosedNotOpen
                | CoinAcceptorError::ManifoldOptoBlocked
                | CoinAcceptorError::ManifoldNotReady
                | CoinAcceptorError::CoinTooFastValidationSensor
                | CoinAcceptorError::CoinTooSlowValidationSensor
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
    pub fn is_possible_rejection(&self) -> bool {
        matches!(
            self,
            CoinAcceptorError::WakeUpTimeout
                | CoinAcceptorError::ValidationTimeout
                | CoinAcceptorError::CreditSensorTimeout
                | CoinAcceptorError::DceOptoTimeout
                | CoinAcceptorError::SecurityStatusChanged
                | CoinAcceptorError::MotorException
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
    pub fn is_null_event(&self) -> bool {
        matches!(self, CoinAcceptorError::NullEvent)
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
    pub fn is_fraud_related(&self) -> bool {
        matches!(
            self,
            CoinAcceptorError::CreditSequenceError
                | CoinAcceptorError::CoinGoingBackwards
                | CoinAcceptorError::CoinTooFastCreditSensor
                | CoinAcceptorError::CoinTooSlowCreditSensor
                | CoinAcceptorError::CoinOnStringMechanism
                | CoinAcceptorError::DceOptoNotSeen
                | CoinAcceptorError::CreditSensorReachedTooEarly
                | CoinAcceptorError::RejectCoinRepeatedTrip
                | CoinAcceptorError::RejectSlug
                | CoinAcceptorError::SecurityStatusChanged
                | CoinAcceptorError::SwallowedCoin
                | CoinAcceptorError::CoinTooFastValidationSensor
                | CoinAcceptorError::CoinTooSlowValidationSensor
                | CoinAcceptorError::ExternalLightAttack
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
    pub fn is_hardware_issue(&self) -> bool {
        matches!(
            self,
            CoinAcceptorError::ValidationSensorNotReady
                | CoinAcceptorError::CreditSensorBlocked
                | CoinAcceptorError::SorterOptoBlocked
                | CoinAcceptorError::RejectSensorBlocked
                | CoinAcceptorError::AcceptGateOpenNotClosed
                | CoinAcceptorError::AcceptGateClosedNotOpen
                | CoinAcceptorError::ManifoldOptoBlocked
                | CoinAcceptorError::MotorException
                | CoinAcceptorError::CoinIncorrectlySorted
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
    pub fn is_timing_issue(&self) -> bool {
        matches!(
            self,
            CoinAcceptorError::SecondCloseCoinError
                | CoinAcceptorError::AcceptGateNotReady
                | CoinAcceptorError::CreditSensorNotReady
                | CoinAcceptorError::SorterNotReady
                | CoinAcceptorError::RejectCoinNotCleared
                | CoinAcceptorError::ManifoldNotReady
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
    pub fn description(&self) -> &'static str {
        match self {
            CoinAcceptorError::NullEvent => "No error occurred",
            CoinAcceptorError::RejectCoin => {
                "Coin rejected - did not match any programmed coin type"
            }
            CoinAcceptorError::InhibitedCoin => "Coin rejected - inhibited by inhibit register",
            CoinAcceptorError::MultipleWindow => {
                "Coin rejected - matched multiple enabled window types"
            }
            CoinAcceptorError::WakeUpTimeout => "Wake-up sensor timeout - possible coin jam",
            CoinAcceptorError::ValidationTimeout => "Validation area timeout - possible coin jam",
            CoinAcceptorError::CreditSensorTimeout => "Credit sensor timeout - possible coin jam",
            CoinAcceptorError::SorterOptoTimeout => {
                "Sorter optical sensor timeout - possible coin jam"
            }
            CoinAcceptorError::SecondCloseCoinError => "Second coin inserted too close to first",
            CoinAcceptorError::AcceptGateNotReady => {
                "Accept gate not ready - coins inserted too quickly"
            }
            CoinAcceptorError::CreditSensorNotReady => {
                "Credit sensor not ready - coins inserted too quickly"
            }
            CoinAcceptorError::SorterNotReady => "Sorter not ready - coins inserted too quickly",
            CoinAcceptorError::RejectCoinNotCleared => "Previous rejected coin not cleared",
            CoinAcceptorError::ValidationSensorNotReady => {
                "Validation sensor not ready - possible developing fault"
            }
            CoinAcceptorError::CreditSensorBlocked => "Credit sensor permanently blocked",
            CoinAcceptorError::SorterOptoBlocked => "Sorter exit sensor permanently blocked",
            CoinAcceptorError::CreditSequenceError => {
                "Credit sequence error - possible fraud attempt"
            }
            CoinAcceptorError::CoinGoingBackwards => {
                "Coin going backwards - possible fraud attempt"
            }
            CoinAcceptorError::CoinTooFastCreditSensor => {
                "Coin too fast over credit sensor - possible fraud attempt"
            }
            CoinAcceptorError::CoinTooSlowCreditSensor => {
                "Coin too slow over credit sensor - possible fraud attempt"
            }
            CoinAcceptorError::CoinOnStringMechanism => {
                "Coin-on-string mechanism activated - fraud attempt detected"
            }
            CoinAcceptorError::DceOptoTimeout => {
                "Dual Coin Entry optical timeout - possible coin jam"
            }
            CoinAcceptorError::DceOptoNotSeen => {
                "Dual Coin Entry optical sensor bypass - possible fraud attempt"
            }
            CoinAcceptorError::CreditSensorReachedTooEarly => {
                "Credit sensor reached too early - possible fraud attempt"
            }
            CoinAcceptorError::RejectCoinRepeatedTrip => {
                "Repeated sequential coin rejection - possible fraud attempt"
            }
            CoinAcceptorError::RejectSlug => "Known slug detected and rejected",
            CoinAcceptorError::RejectSensorBlocked => "Reject sensor permanently blocked",
            CoinAcceptorError::GamesOverload => "Games overload - configuration error",
            CoinAcceptorError::MaxCoinMeterPulsesExceeded => {
                "Maximum coin meter pulses exceeded - configuration error"
            }
            CoinAcceptorError::AcceptGateOpenNotClosed => {
                "Accept gate forced open when should be closed"
            }
            CoinAcceptorError::AcceptGateClosedNotOpen => "Accept gate failed to open when driven",
            CoinAcceptorError::ManifoldOptoTimeout => {
                "Manifold optical timeout - possible coin jam"
            }
            CoinAcceptorError::ManifoldOptoBlocked => "Manifold sensor permanently blocked",
            CoinAcceptorError::ManifoldNotReady => {
                "Manifold not ready - coins inserted too quickly"
            }
            CoinAcceptorError::SecurityStatusChanged => {
                "Security status changed due to fraud detection"
            }
            CoinAcceptorError::MotorException => "Motor exception - mechanical problem",
            CoinAcceptorError::SwallowedCoin => "Swallowed coin - hardware failure or fraud",
            CoinAcceptorError::CoinTooFastValidationSensor => {
                "Coin too fast over validation sensor - possible fraud attempt"
            }
            CoinAcceptorError::CoinTooSlowValidationSensor => {
                "Coin too slow over validation sensor - possible fraud attempt"
            }
            CoinAcceptorError::CoinIncorrectlySorted => {
                "Coin incorrectly sorted - hardware fault notification"
            }
            CoinAcceptorError::ExternalLightAttack => "External light attack detected",
            CoinAcceptorError::DataBlockRequest => "Data block request - attention needed",
            CoinAcceptorError::CoinReturnMechanism => {
                "Coin return mechanism activated - flight deck opened"
            }
            CoinAcceptorError::UnspecifiedAlarm => "Unspecified alarm code",
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
            0 => Ok(CoinAcceptorError::NullEvent),
            1 => Ok(CoinAcceptorError::RejectCoin),
            2 => Ok(CoinAcceptorError::InhibitedCoin),
            3 => Ok(CoinAcceptorError::MultipleWindow),
            4 => Ok(CoinAcceptorError::WakeUpTimeout),
            5 => Ok(CoinAcceptorError::ValidationTimeout),
            6 => Ok(CoinAcceptorError::CreditSensorTimeout),
            7 => Ok(CoinAcceptorError::SorterOptoTimeout),
            8 => Ok(CoinAcceptorError::SecondCloseCoinError),
            9 => Ok(CoinAcceptorError::AcceptGateNotReady),
            10 => Ok(CoinAcceptorError::CreditSensorNotReady),
            11 => Ok(CoinAcceptorError::SorterNotReady),
            12 => Ok(CoinAcceptorError::RejectCoinNotCleared),
            13 => Ok(CoinAcceptorError::ValidationSensorNotReady),
            14 => Ok(CoinAcceptorError::CreditSensorBlocked),
            15 => Ok(CoinAcceptorError::SorterOptoBlocked),
            16 => Ok(CoinAcceptorError::CreditSequenceError),
            17 => Ok(CoinAcceptorError::CoinGoingBackwards),
            18 => Ok(CoinAcceptorError::CoinTooFastCreditSensor),
            19 => Ok(CoinAcceptorError::CoinTooSlowCreditSensor),
            20 => Ok(CoinAcceptorError::CoinOnStringMechanism),
            21 => Ok(CoinAcceptorError::DceOptoTimeout),
            22 => Ok(CoinAcceptorError::DceOptoNotSeen),
            23 => Ok(CoinAcceptorError::CreditSensorReachedTooEarly),
            24 => Ok(CoinAcceptorError::RejectCoinRepeatedTrip),
            25 => Ok(CoinAcceptorError::RejectSlug),
            26 => Ok(CoinAcceptorError::RejectSensorBlocked),
            27 => Ok(CoinAcceptorError::GamesOverload),
            28 => Ok(CoinAcceptorError::MaxCoinMeterPulsesExceeded),
            29 => Ok(CoinAcceptorError::AcceptGateOpenNotClosed),
            30 => Ok(CoinAcceptorError::AcceptGateClosedNotOpen),
            31 => Ok(CoinAcceptorError::ManifoldOptoTimeout),
            32 => Ok(CoinAcceptorError::ManifoldOptoBlocked),
            33 => Ok(CoinAcceptorError::ManifoldNotReady),
            34 => Ok(CoinAcceptorError::SecurityStatusChanged),
            35 => Ok(CoinAcceptorError::MotorException),
            36 => Ok(CoinAcceptorError::SwallowedCoin),
            37 => Ok(CoinAcceptorError::CoinTooFastValidationSensor),
            38 => Ok(CoinAcceptorError::CoinTooSlowValidationSensor),
            39 => Ok(CoinAcceptorError::CoinIncorrectlySorted),
            40 => Ok(CoinAcceptorError::ExternalLightAttack),
            128..=159 => Ok(CoinAcceptorError::InhibitedCoin), // Type 1-32 inhibited coins
            253 => Ok(CoinAcceptorError::DataBlockRequest),
            254 => Ok(CoinAcceptorError::CoinReturnMechanism),
            255 => Ok(CoinAcceptorError::UnspecifiedAlarm),
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
        error as u8
    }
}
