/// ccTalk Standard Manufacturer Strings as defined in the ccTalk Generic Specification
/// Table 6 - ccTalk Standard Manufacturer Strings
///
/// These manufacturer identifiers are returned by the 'Request manufacturer id' command
/// and can be used to help identify a specific product.
///
/// BNVs (Bill Note Validators) are expected to reply with abbreviated names.
/// Other peripherals may return a full name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Manufacturer {
    /// Aardvark Embedded Solutions Ltd (AES)
    AardvarkEmbeddedSolutions,

    /// Alberici (ALB)
    Alberici,

    /// `AlfaNet` informatika d.o.o (ANI)
    AlfaNetInformatika,

    /// `AstroSystems` Ltd (AST)
    AstroSystems,

    /// Azkoyen (AZK)
    Azkoyen,

    /// Comestero Group (CMG)
    ComesteroGroup,

    /// Crane `CashCode` Company (CCC)
    CraneCashCodeCompany,

    /// Crane Payment Solutions (CPS)
    CranePaymentSolutions,

    /// Encopim SL (ECP)
    Encopim,

    /// Gaming Technology Distribution (GTD)
    GamingTechnologyDistribution,

    /// Himecs (HIM)
    Himecs,

    /// Industrias Lorenzo (IL)
    IndustriasLorenzo,

    /// Innovative Technology Ltd (ITL)
    InnovativeTechnology,

    /// Intergrated Technology Ltd (INT)
    /// Note: "Intergrated" appears to be a typo in the original specification
    IntegratedTechnology,

    /// International Currency Technologies (ICT)
    InternationalCurrencyTechnologies,

    /// Japan Cash Machine (JCM)
    JapanCashMachine,

    /// Jofemar (JOF)
    Jofemar,

    /// Kuky (KUK)
    Kuky,

    /// Mars Electronics International (MEI)
    MarsElectronicsInternational,

    /// Microsystem Controls Pty. Ltd. (Microcoin) (MSC)
    MicrosystemControls,

    /// Money Controls (International) (MCI)
    MoneyControlsInternational,

    /// National Rejectors Inc (NRI)
    NationalRejectorsInc,

    /// Phoenix Mecano Digital (PMD)
    PhoenixMecanoDigital,

    /// Starpoint Electrics Ltd (SEL)
    StarpointElectrics,

    /// Telequip / Crane (TQP)
    TelequipCrane,

    /// Weavefuture Inc (WFT)
    WeavefutureInc,

    /// WH Münzprüfer (WHM)
    WHMunzprufer,

    /// iNOTEK (INK)
    /// This manufacturer is not in the original specification but is included because I created
    /// the library :)
    INOTEK,
}

impl Manufacturer {
    /// Returns the full company name
    #[must_use]
    pub const fn full_name(&self) -> &'static str {
        match self {
            Self::AardvarkEmbeddedSolutions => "Aardvark Embedded Solutions Ltd",
            Self::Alberici => "Alberici",
            Self::AlfaNetInformatika => "AlfaNet informatika d.o.o",
            Self::AstroSystems => "AstroSystems Ltd",
            Self::Azkoyen => "Azkoyen",
            Self::ComesteroGroup => "Comestero Group",
            Self::CraneCashCodeCompany => "Crane CashCode Company",
            Self::CranePaymentSolutions => "Crane Payment Solutions",
            Self::Encopim => "Encopim SL",
            Self::GamingTechnologyDistribution => "Gaming Technology Distribution",
            Self::Himecs => "Himecs",
            Self::IndustriasLorenzo => "Industrias Lorenzo",
            Self::InnovativeTechnology => "Innovative Technology Ltd",
            Self::IntegratedTechnology => "Intergrated Technology Ltd",
            Self::InternationalCurrencyTechnologies => "International Currency Technologies",
            Self::JapanCashMachine => "Japan Cash Machine",
            Self::Jofemar => "Jofemar",
            Self::Kuky => "Kuky",
            Self::MarsElectronicsInternational => "Mars Electronics International",
            Self::MicrosystemControls => "Microsystem Controls Pty. Ltd.",
            Self::MoneyControlsInternational => "Money Controls (International)",
            Self::NationalRejectorsInc => "National Rejectors Inc",
            Self::PhoenixMecanoDigital => "Phoenix Mecano Digital",
            Self::StarpointElectrics => "Starpoint Electrics Ltd",
            Self::TelequipCrane => "Telequip / Crane",
            Self::WeavefutureInc => "Weavefuture Inc",
            Self::WHMunzprufer => "WH Münzprüfer",
            Self::INOTEK => "iNOTEK",
        }
    }

    /// Returns the abbreviated name (typically used by BNVs)
    #[must_use]
    pub const fn abbreviated_name(&self) -> &'static str {
        match self {
            Self::AardvarkEmbeddedSolutions => "AES",
            Self::Alberici => "ALB",
            Self::AlfaNetInformatika => "ANI",
            Self::AstroSystems => "AST",
            Self::Azkoyen => "AZK",
            Self::ComesteroGroup => "CMG",
            Self::CraneCashCodeCompany => "CCC",
            Self::CranePaymentSolutions => "CPS",
            Self::Encopim => "ECP",
            Self::GamingTechnologyDistribution => "GTD",
            Self::Himecs => "HIM",
            Self::IndustriasLorenzo => "IL",
            Self::InnovativeTechnology => "ITL",
            Self::IntegratedTechnology => "INT",
            Self::InternationalCurrencyTechnologies => "ICT",
            Self::JapanCashMachine => "JCM",
            Self::Jofemar => "JOF",
            Self::Kuky => "KUK",
            Self::MarsElectronicsInternational => "MEI",
            Self::MicrosystemControls => "MSC",
            Self::MoneyControlsInternational => "MCI",
            Self::NationalRejectorsInc => "NRI",
            Self::PhoenixMecanoDigital => "PMD",
            Self::StarpointElectrics => "SEL",
            Self::TelequipCrane => "TQP",
            Self::WeavefutureInc => "WFT",
            Self::WHMunzprufer => "WHM",
            Self::INOTEK => "INK",
        }
    }

    /// Returns all known manufacturers as a slice
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::AardvarkEmbeddedSolutions,
            Self::Alberici,
            Self::AlfaNetInformatika,
            Self::AstroSystems,
            Self::Azkoyen,
            Self::ComesteroGroup,
            Self::CraneCashCodeCompany,
            Self::CranePaymentSolutions,
            Self::Encopim,
            Self::GamingTechnologyDistribution,
            Self::Himecs,
            Self::IndustriasLorenzo,
            Self::InnovativeTechnology,
            Self::IntegratedTechnology,
            Self::InternationalCurrencyTechnologies,
            Self::JapanCashMachine,
            Self::Jofemar,
            Self::Kuky,
            Self::MarsElectronicsInternational,
            Self::MicrosystemControls,
            Self::MoneyControlsInternational,
            Self::NationalRejectorsInc,
            Self::PhoenixMecanoDigital,
            Self::StarpointElectrics,
            Self::TelequipCrane,
            Self::WeavefutureInc,
            Self::WHMunzprufer,
            Self::INOTEK,
        ]
    }

    /// Attempts to parse a manufacturer from a full name string
    #[must_use]
    pub fn from_full_name(name: &str) -> Option<Self> {
        Self::all()
            .iter()
            .find(|manufacturer| manufacturer.full_name() == name)
            .copied()
    }

    /// Attempts to parse a manufacturer from an abbreviated name string
    #[must_use]
    pub fn from_abbreviated_name(name: &str) -> Option<Self> {
        Self::all()
            .iter()
            .find(|manufacturer| manufacturer.abbreviated_name() == name)
            .copied()
    }

    /// Attempts to parse a manufacturer from either full or abbreviated name
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::from_full_name(name).or_else(|| Self::from_abbreviated_name(name))
    }
}

impl core::fmt::Display for Manufacturer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

/// Represents a manufacturer identifier that could be either a known manufacturer
/// or an unrecognized string from a new/unlisted manufacturer
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManufacturerIdentifier {
    /// A known, registered manufacturer
    Known(Manufacturer),

    /// An unrecognized manufacturer string
    ///
    /// This allows for new manufacturers that haven't been registered yet
    /// or custom manufacturer strings.
    #[cfg(not(feature = "std"))]
    Unknown(heapless::String<64>), // Using heapless for no_std compatibility

    #[cfg(feature = "std")]
    Unknown(std::string::String), // Using std::String when available
}

impl ManufacturerIdentifier {
    /// Creates a new manufacturer identifier from a string
    ///
    /// First attempts to match against known manufacturers (both full and abbreviated names),
    /// falling back to storing as an unknown manufacturer if no match is found.
    #[must_use]
    #[allow(clippy::option_if_let_else)] // For clarity in this context
    pub fn new(name: &str) -> Self {
        match Manufacturer::from_name(name) {
            Some(manufacturer) => Self::Known(manufacturer),
            None => {
                #[cfg(not(feature = "std"))]
                {
                    match heapless::String::try_from(name) {
                        Ok(unknown) => Self::Unknown(unknown),
                        Err(_) => {
                            // If the string is too long, truncate it
                            let truncated = &name[..name.len().min(64)];
                            let truncated_string = heapless::String::try_from(truncated)
                                .unwrap_or_else(|_| heapless::String::new());
                            Self::Unknown(truncated_string)
                        }
                    }
                }
                #[cfg(feature = "std")]
                {
                    use std::string::ToString;
                    Self::Unknown(name.to_string())
                }
            }
        }
    }

    /// Returns the manufacturer name as a string
    #[must_use]
    pub const fn name(&self) -> &str {
        match self {
            Self::Known(manufacturer) => manufacturer.full_name(),
            Self::Unknown(name) => name.as_str(),
        }
    }

    /// Returns true if this is a known/registered manufacturer
    #[must_use]
    pub const fn is_known(&self) -> bool {
        matches!(self, Self::Known(_))
    }

    /// Returns the known manufacturer if available
    #[must_use]
    pub const fn known_manufacturer(&self) -> Option<Manufacturer> {
        match self {
            Self::Known(manufacturer) => Some(*manufacturer),
            Self::Unknown(_) => None,
        }
    }
}

impl core::fmt::Display for ManufacturerIdentifier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<Manufacturer> for ManufacturerIdentifier {
    fn from(manufacturer: Manufacturer) -> Self {
        Self::Known(manufacturer)
    }
}

#[cfg(test)]
mod tests {
    use std::string::ToString;

    use super::*;

    #[test]
    fn test_full_names() {
        assert_eq!(
            Manufacturer::CranePaymentSolutions.full_name(),
            "Crane Payment Solutions"
        );
        assert_eq!(
            Manufacturer::InnovativeTechnology.full_name(),
            "Innovative Technology Ltd"
        );
    }

    #[test]
    fn test_abbreviated_names() {
        assert_eq!(
            Manufacturer::CranePaymentSolutions.abbreviated_name(),
            "CPS"
        );
        assert_eq!(Manufacturer::InnovativeTechnology.abbreviated_name(), "ITL");
    }

    #[test]
    fn test_from_full_name() {
        assert_eq!(
            Manufacturer::from_full_name("Crane Payment Solutions"),
            Some(Manufacturer::CranePaymentSolutions)
        );
        assert_eq!(Manufacturer::from_full_name("Unknown Company"), None);
    }

    #[test]
    fn test_from_abbreviated_name() {
        assert_eq!(
            Manufacturer::from_abbreviated_name("CPS"),
            Some(Manufacturer::CranePaymentSolutions)
        );
        assert_eq!(Manufacturer::from_abbreviated_name("UNK"), None);
    }

    #[test]
    fn test_from_name() {
        assert_eq!(
            Manufacturer::from_name("CPS"),
            Some(Manufacturer::CranePaymentSolutions)
        );
        assert_eq!(
            Manufacturer::from_name("Crane Payment Solutions"),
            Some(Manufacturer::CranePaymentSolutions)
        );
        assert_eq!(Manufacturer::from_name("Unknown"), None);
    }

    #[test]
    fn test_manufacturer_identifier_known() {
        let id = ManufacturerIdentifier::new("CPS");
        assert!(id.is_known());
        assert_eq!(
            id.known_manufacturer(),
            Some(Manufacturer::CranePaymentSolutions)
        );
        assert_eq!(id.name(), "Crane Payment Solutions");
    }

    #[test]
    fn test_manufacturer_identifier_unknown() {
        let id = ManufacturerIdentifier::new("Custom Manufacturer");
        assert!(!id.is_known());
        assert_eq!(id.known_manufacturer(), None);
        assert_eq!(id.name(), "Custom Manufacturer");
    }

    #[test]
    fn test_display() {
        assert_eq!(
            Manufacturer::CranePaymentSolutions.to_string(),
            "Crane Payment Solutions"
        );

        let id = ManufacturerIdentifier::new("ITL");
        assert_eq!(id.to_string(), "Innovative Technology Ltd");
    }

    #[test]
    fn test_all_manufacturers_count() {
        assert_eq!(Manufacturer::all().len(), 28);
    }
}
