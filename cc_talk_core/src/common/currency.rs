use core::str::FromStr;
use heapless::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Factor {
    Micro, // m = 0.001
    None,  // No factor = 1.0
    Dot,   // . = 1.0 (decimal point)
    Kilo,  // K = 1000
    Mega,  // M = 1,000,000
    Giga,  // G = 1,000,000,000
}

impl Factor {
    #[must_use]
    pub const fn multiplier(&self) -> f64 {
        match self {
            Self::Micro => 0.001,
            Self::None | Self::Dot => 1.0,
            Self::Kilo => 1000.0,
            Self::Mega => 1_000_000.0,
            Self::Giga => 1_000_000_000.0,
        }
    }
}

impl From<char> for Factor {
    fn from(value: char) -> Self {
        match value {
            'm' => Self::Micro,
            '.' => Self::Dot,
            'K' => Self::Kilo,
            'M' => Self::Mega,
            'G' => Self::Giga,
            _ => Self::None,
        }
    }
}

// We could do a full structure with cctalk, mbd, jcm and dialing code, but it seems unnecessary
fn country_code_to_decimals(country_code: &str) -> u8 {
    match country_code {
        "JP" | "JPY" | "XP" | "XPF" => 0,
        "BH" | "BHD" | "OM" | "OMR" | "TN" | "TND" => 3,
        _ => 2, // Default to 2 decimal places for other countries
    }
}

/// Represents a Token, which can either be a coin, bill or token.
///
/// For tokens no more information is needed.
///
/// For coins and bills, the `CurrencyValue` struct is used to represent the value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrencyToken {
    Token,
    Currency(CurrencyValue),
}

impl CurrencyToken {
    /// Creates a currency or token from a value string.
    ///
    /// # Errors
    ///
    /// Errors if the value string is too small or if the coin is not supported by the device.
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    pub fn build(value_string: &str) -> Result<Self, CurrencyTokenError> {
        if value_string.len() < 6 {
            return Err(CurrencyTokenError::ValueStringTooSmall);
        }

        let country_code = &value_string[0..2];
        let decimals = country_code_to_decimals(country_code);

        if country_code == ".." {
            return Err(CurrencyTokenError::CoinNotSupportedByDevice);
        }

        if country_code == "TK" {
            return Ok(Self::Token);
        }

        let chars: Vec<char, 16> = value_string.chars().collect();
        let to_skip = 2;
        let to_take = value_string.len() - to_skip;

        // Extract all digits from the value part
        let digits: Vec<u8, 8> = chars
            .iter()
            .skip(to_skip)
            .take(to_take)
            .filter(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).unwrap_or(0) as u8)
            .collect();

        // Calculate numeric value from digits
        let mut numeric_value = 0u32;
        for &digit in &digits {
            numeric_value = numeric_value * 10 + u32::from(digit);
        }

        // Find factor (last non-digit character in the value part)
        let factor = chars
            .iter()
            .skip(to_skip)
            .take(to_take)
            .filter(|c| Factor::from(**c) != Factor::None)
            .map(|c| Factor::from(*c))
            .next_back() // Changed from next_back() to last()
            .unwrap_or(Factor::None);

        let final_value = match factor {
            // TODO: Find a solution for micro factors that works without std
            #[cfg(feature = "std")]
            Factor::Micro => {
                let float_result = f64::from(numeric_value) * factor.multiplier();

                if value_string.len() == 7 {
                    (float_result * 10_f64.powi(i32::from(decimals))) as u32
                } else {
                    float_result as u32
                }
            }
            _ => {
                // For integer factors (None, Dot, Kilo, Mega, Giga)
                let factor_multiplier = factor.multiplier() as u32;
                let factored_value = numeric_value * factor_multiplier;

                if value_string.len() == 7 {
                    // Bill: multiply by 10^decimals to get smallest units
                    factored_value * 10u32.pow(u32::from(decimals))
                } else {
                    // Coin: value is already in appropriate units
                    factored_value
                }
            }
        };

        Ok(Self::Currency(CurrencyValue {
            country_code: heapless::String::from_str(country_code)
                .map_err(|_| CurrencyTokenError::InvalidFormat)?,
            factor,
            decimals,
            value: final_value,
        }))
    }
}

/// Represents a monetary value in a specific currency, including the country code, factor,
/// decimals, and value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrencyValue {
    country_code: heapless::String<2>,
    factor: Factor,
    decimals: u8,
    value: u32, // Value in smallest currency units (cents, pence, etc.)
}

impl CurrencyValue {
    /// Get the monetary value as a float
    #[cfg(feature = "std")]
    #[must_use]
    pub fn monetary_value(&self) -> f64 {
        // TODO: Find a solution for no_std
        f64::from(self.value) / 10_f64.powi(i32::from(self.decimals))
    }

    /// Get the value in smallest currency units
    #[must_use]
    pub const fn smallest_unit_value(&self) -> u32 {
        self.value
    }

    #[must_use]
    pub fn country_code(&self) -> &str {
        &self.country_code
    }

    #[must_use]
    pub const fn factor(&self) -> Factor {
        self.factor
    }

    #[must_use]
    pub const fn decimals(&self) -> u8 {
        self.decimals
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CurrencyTokenError {
    #[error("invalid currency token format")]
    InvalidFormat,
    #[error("value string too small")]
    ValueStringTooSmall,
    #[error("coin not supported by device")]
    CoinNotSupportedByDevice,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "std")] // Temporary until we find a no_std solution
    fn euro_coin() {
        let coins = [
            "EU001A", "EU002A", "EU005A", "EU010A", "EU020A", "EU050A", "EU100A", "EU200A",
            // With decimal point
            "EU.01A", "EU.02A", "EU.05A", "EU.10A", "EU.20A", "EU.50A", "EU100A", "EU200A",
        ];

        let answers = [1, 2, 5, 10, 20, 50, 100, 200];

        for (index, coin) in coins.iter().enumerate() {
            let token = CurrencyToken::build(coin).expect("should build currency token");
            match token {
                CurrencyToken::Currency(currency) => {
                    assert_eq!(currency.country_code(), "EU");
                    assert_eq!(currency.decimals(), 2);

                    let expected_factor = if (8..=13).contains(&index) {
                        Factor::Dot
                    } else {
                        Factor::None
                    };
                    assert_eq!(currency.factor(), expected_factor);

                    let expected_value = answers[index % 8];
                    assert_eq!(currency.smallest_unit_value(), expected_value);

                    // Check monetary value
                    let expected_monetary = f64::from(expected_value) / 100.0;
                    assert!((currency.monetary_value() - expected_monetary).abs() < 0.01);
                }
                CurrencyToken::Token => panic!("Expected currency, got {:?}", token),
            }
        }
    }

    #[test]
    #[cfg(feature = "std")] // Temporary until we find a no_std solution
    fn euro_bills() {
        let bills = [
            "EU0005B", "EU0010B", "EU0020B", "EU0050B", "EU0100B", "EU0200B", "EU0500B",
        ];

        let answers = [500, 1000, 2000, 5000, 10000, 20000, 50000];

        for (index, bill) in bills.iter().enumerate() {
            let token = CurrencyToken::build(bill).expect("should build currency token");
            match token {
                CurrencyToken::Currency(currency) => {
                    assert_eq!(currency.country_code(), "EU");
                    assert_eq!(currency.factor(), Factor::None);
                    assert_eq!(currency.decimals(), 2);
                    assert_eq!(currency.smallest_unit_value(), answers[index]);

                    // Check monetary value
                    let expected_monetary = f64::from(answers[index]) / 100.0;
                    assert!((currency.monetary_value() - expected_monetary).abs() < 0.01);
                }
                CurrencyToken::Token => panic!("Expected currency, got {:?}", token),
            }
        }
    }

    #[test]
    #[cfg(feature = "std")] // Temporary until we find a no_std solution
    fn test_factors() {
        // Test Kilo factor
        let result = CurrencyToken::build("US001K").expect("should build currency token");
        match result {
            CurrencyToken::Currency(currency) => {
                assert_eq!(currency.factor(), Factor::Kilo);
                assert_eq!(currency.smallest_unit_value(), 1000);
                assert!((currency.monetary_value() - 10.0).abs() < 0.01);
            }
            CurrencyToken::Token => panic!("Expected currency"),
        }

        // Test Micro factor
        let result = CurrencyToken::build("EU500m").expect("should build currency token");
        match result {
            CurrencyToken::Currency(currency) => {
                assert_eq!(currency.factor(), Factor::Micro);
                assert_eq!(currency.smallest_unit_value(), 0); // less than 1cent
                assert!((currency.monetary_value() - 0.0).abs() < 0.01);
            }
            CurrencyToken::Token => panic!("Expected currency"),
        }
    }

    #[test]
    #[cfg(feature = "std")] // Temporary until we find a no_std solution
    fn test_japanese_yen() {
        // Test 0 decimal currency
        let result = CurrencyToken::build("JP100A").expect("should build currency token");
        match result {
            CurrencyToken::Currency(currency) => {
                assert_eq!(currency.country_code(), "JP");
                assert_eq!(currency.decimals(), 0);
                assert_eq!(currency.smallest_unit_value(), 100);
                assert!((currency.monetary_value() - 100.0).abs() < 0.01);
            }
            CurrencyToken::Token => panic!("Expected currency"),
        }
    }

    #[test]
    fn test_token() {
        let result = CurrencyToken::build("TK001A").expect("should build currency token");
        assert_eq!(result, CurrencyToken::Token);
    }

    #[test]
    fn test_error_cases() {
        assert!(matches!(
            CurrencyToken::build("US12"),
            Err(CurrencyTokenError::ValueStringTooSmall)
        ));

        assert!(matches!(
            CurrencyToken::build("..123A"),
            Err(CurrencyTokenError::CoinNotSupportedByDevice)
        ));
    }

    #[test]
    #[cfg(feature = "std")] // Temporary until we find a no_std solution
    fn test_decimal_point_parsing() {
        let result = CurrencyToken::build("EU.50A").expect("should build currency token");
        match result {
            CurrencyToken::Currency(currency) => {
                assert_eq!(currency.factor(), Factor::Dot);
                assert_eq!(currency.smallest_unit_value(), 50);
                assert!((currency.monetary_value() - 0.50).abs() < 0.01);
            }
            CurrencyToken::Token => panic!("Expected currency"),
        }
    }

    #[test]
    #[cfg(feature = "std")] // Temporary until we find a no_std solution
    fn test_bill_vs_coin_detection() {
        // 6-character string should be treated as coin
        let coin = CurrencyToken::build("US100A").expect("should build currency token");
        match coin {
            CurrencyToken::Currency(currency) => {
                assert_eq!(currency.smallest_unit_value(), 100);
                assert!((currency.monetary_value() - 1.0).abs() < 0.01);
            }
            CurrencyToken::Token => panic!("Expected currency"),
        }

        // 7-character string should be treated as bill
        let bill = CurrencyToken::build("US0100A").expect("should build currency token");
        match bill {
            CurrencyToken::Currency(currency) => {
                assert_eq!(currency.smallest_unit_value(), 10000);
                assert!((currency.monetary_value() - 100.0).abs() < 0.01);
            }
            CurrencyToken::Token => panic!("Expected currency"),
        }
    }

    #[test]
    #[cfg(feature = "std")] // Temporary until we find a no_std solution
    fn test_additional_factor_cases() {
        let result = CurrencyToken::build("US001M").expect("should build currency token");
        match result {
            CurrencyToken::Currency(currency) => {
                assert_eq!(currency.factor(), Factor::Mega);
                assert_eq!(currency.smallest_unit_value(), 1_000_000);
                assert!((currency.monetary_value() - 10_000.0).abs() < 0.01);
            }
            CurrencyToken::Token => panic!("Expected currency"),
        }

        let result = CurrencyToken::build("US001G").expect("should build currency token");
        match result {
            CurrencyToken::Currency(currency) => {
                assert_eq!(currency.factor(), Factor::Giga);
                assert_eq!(currency.smallest_unit_value(), 1_000_000_000);
                assert!((currency.monetary_value() - 10_000_000.0).abs() < 0.01);
            }
            CurrencyToken::Token => panic!("Expected currency"),
        }
    }

    #[test]
    #[cfg(feature = "std")] // Temporary until we find a no_std solution
    fn test_edge_cases() {
        let result = CurrencyToken::build("US000A").expect("should build currency token");
        match result {
            CurrencyToken::Currency(currency) => {
                assert_eq!(currency.smallest_unit_value(), 0);
                assert!((currency.monetary_value() - 0.0).abs() < 0.01);
            }
            CurrencyToken::Token => panic!("Expected currency"),
        }

        let result = CurrencyToken::build("US999A").expect("should build currency token");
        match result {
            CurrencyToken::Currency(currency) => {
                assert_eq!(currency.smallest_unit_value(), 999);
                assert!((currency.monetary_value() - 9.99).abs() < 0.01);
            }
            CurrencyToken::Token => panic!("Expected currency"),
        }
    }
}
