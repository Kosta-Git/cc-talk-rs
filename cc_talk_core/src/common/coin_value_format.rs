#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CoinType {
    Token,
    Coin(u16),
    None,
}

impl From<u8> for CoinType {
    fn from(value: u8) -> Self {
        match value {
            0 | 128 => CoinType::None,
            255 => CoinType::Token,
            value if (value & 0b10000000) != 0 => {
                let data_value = (value & 0b01111111) as u16;
                let coin_value = data_value * 10;
                CoinType::Coin(coin_value)
            }
            value => CoinType::Coin(value as u16),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_u8_none_values() {
        assert_eq!(CoinType::from(0), CoinType::None);
        assert_eq!(CoinType::from(128), CoinType::None);
    }

    #[test]
    fn test_from_u8_token() {
        assert_eq!(CoinType::from(255), CoinType::Token);
    }

    #[test]
    fn test_from_u8_direct_values() {
        // Values without multiplier bit (1-127)
        assert_eq!(CoinType::from(1), CoinType::Coin(1));
        assert_eq!(CoinType::from(2), CoinType::Coin(2));
        assert_eq!(CoinType::from(5), CoinType::Coin(5));
        assert_eq!(CoinType::from(10), CoinType::Coin(10));
        assert_eq!(CoinType::from(20), CoinType::Coin(20));
        assert_eq!(CoinType::from(25), CoinType::Coin(25));
        assert_eq!(CoinType::from(50), CoinType::Coin(50));
        assert_eq!(CoinType::from(100), CoinType::Coin(100));
        assert_eq!(CoinType::from(127), CoinType::Coin(127));
    }

    #[test]
    fn test_from_u8_multiplied_values() {
        // Values with multiplier bit set (bit 7 = 1)
        // 148 = 0b10010100 = 128 + 20, data value = 20, result = 20 * 10 = 200
        assert_eq!(CoinType::from(148), CoinType::Coin(200));

        // 153 = 0b10011001 = 128 + 25, data value = 25, result = 25 * 10 = 250
        assert_eq!(CoinType::from(153), CoinType::Coin(250));

        // 178 = 0b10110010 = 128 + 50, data value = 50, result = 50 * 10 = 500
        assert_eq!(CoinType::from(178), CoinType::Coin(500));

        // 228 = 0b11100100 = 128 + 100, data value = 100, result = 100 * 10 = 1000
        assert_eq!(CoinType::from(228), CoinType::Coin(1000));

        // 254 = 0b11111110 = 128 + 126, data value = 126, result = 126 * 10 = 1260
        assert_eq!(CoinType::from(254), CoinType::Coin(1260));
    }

    #[test]
    fn test_common_cvf_values() {
        // Test the table from the specification
        let test_cases = [
            (1, 1),
            (2, 2),
            (5, 5),
            (10, 10),
            (20, 20),
            (25, 25),
            (50, 50),
            (100, 100),
            (148, 200),
            (153, 250),
            (178, 500),
            (228, 1000),
        ];

        for (cvf_code, expected_value) in test_cases {
            assert_eq!(CoinType::from(cvf_code), CoinType::Coin(expected_value));
        }

        assert_eq!(CoinType::from(255), CoinType::Token);
    }

    #[test]
    fn test_edge_cases() {
        // Test boundary values
        assert_eq!(CoinType::from(129), CoinType::Coin(10)); // 129 = 128 + 1, data=1, result=1*10=10
        assert_eq!(CoinType::from(130), CoinType::Coin(20)); // 130 = 128 + 2, data=2, result=2*10=20

        // Test maximum data value with multiplier
        assert_eq!(CoinType::from(254), CoinType::Coin(1260)); // 254 = 128 + 126, data=126, result=126*10=1260
    }

    #[test]
    fn test_multiplier_bit_detection() {
        // Values without multiplier bit (0-127)
        for i in 0..=127 {
            if i == 0 {
                assert_eq!(CoinType::from(i), CoinType::None);
            } else {
                assert_eq!(CoinType::from(i), CoinType::Coin(i as u16));
            }
        }

        // Value 128 is special case (None)
        assert_eq!(CoinType::from(128), CoinType::None);

        // Values with multiplier bit (129-254)
        for i in 129..=254 {
            let data_value = (i & 0b01111111) as u16;
            let expected_value = data_value * 10;
            assert_eq!(CoinType::from(i), CoinType::Coin(expected_value));
        }

        // Value 255 is special case (Token)
        assert_eq!(CoinType::from(255), CoinType::Token);
    }
}
