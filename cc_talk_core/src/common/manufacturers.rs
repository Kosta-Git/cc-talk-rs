/// ccTalk Standard Manufacturer Strings as defined in the ccTalk Generic Specification
/// Table 6 - ccTalk Standard Manufacturer Strings
///
/// These manufacturer identifiers are returned by the 'Request manufacturer id' command
/// and can be used to help identify a specific product.
///
/// BNVs (Bill Note Validators) are expected to reply with abbreviated names.
/// Other peripherals may return a full name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Manufacturer {
    /// Aardvark Embedded Solutions Ltd (AES)
    AardvarkEmbeddedSolutions,

    /// Alberici (ALB)
    Alberici,

    /// AlfaNet informatika d.o.o (ANI)
    AlfaNetInformatika,

    /// AstroSystems Ltd (AST)
    AstroSystems,

    /// Azkoyen (AZK)
    Azkoyen,

    /// Comestero Group (CMG)
    ComesteroGroup,

    /// Crane CashCode Company (CCC)
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
    pub const fn full_name(&self) -> &'static str {
        match self {
            Manufacturer::AardvarkEmbeddedSolutions => "Aardvark Embedded Solutions Ltd",
            Manufacturer::Alberici => "Alberici",
            Manufacturer::AlfaNetInformatika => "AlfaNet informatika d.o.o",
            Manufacturer::AstroSystems => "AstroSystems Ltd",
            Manufacturer::Azkoyen => "Azkoyen",
            Manufacturer::ComesteroGroup => "Comestero Group",
            Manufacturer::CraneCashCodeCompany => "Crane CashCode Company",
            Manufacturer::CranePaymentSolutions => "Crane Payment Solutions",
            Manufacturer::Encopim => "Encopim SL",
            Manufacturer::GamingTechnologyDistribution => "Gaming Technology Distribution",
            Manufacturer::Himecs => "Himecs",
            Manufacturer::IndustriasLorenzo => "Industrias Lorenzo",
            Manufacturer::InnovativeTechnology => "Innovative Technology Ltd",
            Manufacturer::IntegratedTechnology => "Intergrated Technology Ltd",
            Manufacturer::InternationalCurrencyTechnologies => {
                "International Currency Technologies"
            }
            Manufacturer::JapanCashMachine => "Japan Cash Machine",
            Manufacturer::Jofemar => "Jofemar",
            Manufacturer::Kuky => "Kuky",
            Manufacturer::MarsElectronicsInternational => "Mars Electronics International",
            Manufacturer::MicrosystemControls => "Microsystem Controls Pty. Ltd.",
            Manufacturer::MoneyControlsInternational => "Money Controls (International)",
            Manufacturer::NationalRejectorsInc => "National Rejectors Inc",
            Manufacturer::PhoenixMecanoDigital => "Phoenix Mecano Digital",
            Manufacturer::StarpointElectrics => "Starpoint Electrics Ltd",
            Manufacturer::TelequipCrane => "Telequip / Crane",
            Manufacturer::WeavefutureInc => "Weavefuture Inc",
            Manufacturer::WHMunzprufer => "WH Münzprüfer",
            Manufacturer::INOTEK => "iNOTEK",
        }
    }

    /// Returns the abbreviated name (typically used by BNVs)
    pub const fn abbreviated_name(&self) -> &'static str {
        match self {
            Manufacturer::AardvarkEmbeddedSolutions => "AES",
            Manufacturer::Alberici => "ALB",
            Manufacturer::AlfaNetInformatika => "ANI",
            Manufacturer::AstroSystems => "AST",
            Manufacturer::Azkoyen => "AZK",
            Manufacturer::ComesteroGroup => "CMG",
            Manufacturer::CraneCashCodeCompany => "CCC",
            Manufacturer::CranePaymentSolutions => "CPS",
            Manufacturer::Encopim => "ECP",
            Manufacturer::GamingTechnologyDistribution => "GTD",
            Manufacturer::Himecs => "HIM",
            Manufacturer::IndustriasLorenzo => "IL",
            Manufacturer::InnovativeTechnology => "ITL",
            Manufacturer::IntegratedTechnology => "INT",
            Manufacturer::InternationalCurrencyTechnologies => "ICT",
            Manufacturer::JapanCashMachine => "JCM",
            Manufacturer::Jofemar => "JOF",
            Manufacturer::Kuky => "KUK",
            Manufacturer::MarsElectronicsInternational => "MEI",
            Manufacturer::MicrosystemControls => "MSC",
            Manufacturer::MoneyControlsInternational => "MCI",
            Manufacturer::NationalRejectorsInc => "NRI",
            Manufacturer::PhoenixMecanoDigital => "PMD",
            Manufacturer::StarpointElectrics => "SEL",
            Manufacturer::TelequipCrane => "TQP",
            Manufacturer::WeavefutureInc => "WFT",
            Manufacturer::WHMunzprufer => "WHM",
            Manufacturer::INOTEK => "INK",
        }
    }

    /// Returns all known manufacturers as a slice
    pub const fn all() -> &'static [Manufacturer] {
        &[
            Manufacturer::AardvarkEmbeddedSolutions,
            Manufacturer::Alberici,
            Manufacturer::AlfaNetInformatika,
            Manufacturer::AstroSystems,
            Manufacturer::Azkoyen,
            Manufacturer::ComesteroGroup,
            Manufacturer::CraneCashCodeCompany,
            Manufacturer::CranePaymentSolutions,
            Manufacturer::Encopim,
            Manufacturer::GamingTechnologyDistribution,
            Manufacturer::Himecs,
            Manufacturer::IndustriasLorenzo,
            Manufacturer::InnovativeTechnology,
            Manufacturer::IntegratedTechnology,
            Manufacturer::InternationalCurrencyTechnologies,
            Manufacturer::JapanCashMachine,
            Manufacturer::Jofemar,
            Manufacturer::Kuky,
            Manufacturer::MarsElectronicsInternational,
            Manufacturer::MicrosystemControls,
            Manufacturer::MoneyControlsInternational,
            Manufacturer::NationalRejectorsInc,
            Manufacturer::PhoenixMecanoDigital,
            Manufacturer::StarpointElectrics,
            Manufacturer::TelequipCrane,
            Manufacturer::WeavefutureInc,
            Manufacturer::WHMunzprufer,
            Manufacturer::INOTEK,
        ]
    }

    /// Attempts to parse a manufacturer from a full name string
    pub fn from_full_name(name: &str) -> Option<Self> {
        Self::all()
            .iter()
            .find(|manufacturer| manufacturer.full_name() == name)
            .copied()
    }

    /// Attempts to parse a manufacturer from an abbreviated name string
    pub fn from_abbreviated_name(name: &str) -> Option<Self> {
        Self::all()
            .iter()
            .find(|manufacturer| manufacturer.abbreviated_name() == name)
            .copied()
    }

    /// Attempts to parse a manufacturer from either full or abbreviated name
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
    #[cfg(feature = "heapless")]
    Unknown(heapless::String<64>), // Using heapless for no_std compatibility

    #[cfg(all(feature = "std", not(feature = "heapless")))]
    Unknown(std::string::String), // Using std::String when available

    #[cfg(not(any(feature = "heapless", feature = "std")))]
    Unknown(&'static str), // Fallback to static string when no dynamic allocation is available
}

impl ManufacturerIdentifier {
    /// Creates a new manufacturer identifier from a string
    ///
    /// First attempts to match against known manufacturers (both full and abbreviated names),
    /// falling back to storing as an unknown manufacturer if no match is found.
    pub fn new(name: &str) -> Self {
        match Manufacturer::from_name(name) {
            Some(manufacturer) => ManufacturerIdentifier::Known(manufacturer),
            None => {
                #[cfg(feature = "heapless")]
                {
                    match heapless::String::try_from(name) {
                        Ok(unknown) => ManufacturerIdentifier::Unknown(unknown),
                        Err(_) => {
                            // If the string is too long, truncate it
                            let truncated = &name[..name.len().min(64)];
                            let truncated_string = heapless::String::try_from(truncated)
                                .unwrap_or_else(|_| heapless::String::new());
                            ManufacturerIdentifier::Unknown(truncated_string)
                        }
                    }
                }
                #[cfg(all(feature = "std", not(feature = "heapless")))]
                {
                    ManufacturerIdentifier::Unknown(name.to_string())
                }
                #[cfg(not(any(feature = "heapless", feature = "std")))]
                {
                    // In this case, we can only store static strings
                    // This is a limitation when no dynamic allocation is available
                    ManufacturerIdentifier::Unknown("")
                }
            }
        }
    }

    /// Creates a new manufacturer identifier from a static string
    ///
    /// This method is useful when you have a static string and want to avoid
    /// potential allocation or truncation issues.
    #[cfg(not(any(feature = "heapless", feature = "std")))]
    pub fn new_static(name: &'static str) -> Self {
        match Manufacturer::from_name(name) {
            Some(manufacturer) => ManufacturerIdentifier::Known(manufacturer),
            None => ManufacturerIdentifier::Unknown(name),
        }
    }

    /// Returns the manufacturer name as a string
    pub fn name(&self) -> &str {
        match self {
            ManufacturerIdentifier::Known(manufacturer) => manufacturer.full_name(),
            ManufacturerIdentifier::Unknown(name) => {
                #[cfg(any(feature = "std", feature = "heapless"))]
                {
                    name.as_str()
                }
                #[cfg(not(any(feature = "heapless", feature = "std")))]
                {
                    name
                }
            }
        }
    }

    /// Returns true if this is a known/registered manufacturer
    pub fn is_known(&self) -> bool {
        matches!(self, ManufacturerIdentifier::Known(_))
    }

    /// Returns the known manufacturer if available
    pub fn known_manufacturer(&self) -> Option<Manufacturer> {
        match self {
            ManufacturerIdentifier::Known(manufacturer) => Some(*manufacturer),
            ManufacturerIdentifier::Unknown(_) => None,
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
        ManufacturerIdentifier::Known(manufacturer)
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
    #[cfg(any(feature = "heapless", feature = "std"))]
    fn test_manufacturer_identifier_unknown() {
        let id = ManufacturerIdentifier::new("Custom Manufacturer");
        assert!(!id.is_known());
        assert_eq!(id.known_manufacturer(), None);
        assert_eq!(id.name(), "Custom Manufacturer");
    }

    #[test]
    #[cfg(not(any(feature = "heapless", feature = "std")))]
    fn test_manufacturer_identifier_static() {
        let id = ManufacturerIdentifier::new_static("Custom Manufacturer");
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
