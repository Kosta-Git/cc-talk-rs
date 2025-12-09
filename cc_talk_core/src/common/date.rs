/// Struct for the special Money Controls format Relative To Base Year Date (`RTBYDate`).
/// Originally chosen to avoid the Y2K problem
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RTBYDate {
    date: u16,
}

impl RTBYDate {
    /// Creates a new date from a u16 value.
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self { date: value }
    }

    /// This returns the year, it can be maximul 31 years after the relative base year
    #[must_use]
    pub const fn year(&self, relative: u16) -> u16 {
        let offset = (self.date >> 9) & 0b1_1111;
        relative + offset
    }

    /// 1 to 12
    #[must_use]
    pub const fn month(&self) -> u8 {
        ((self.date >> 5) & 0b1111) as u8
    }

    /// 1 to 31
    #[must_use]
    pub const fn day(&self) -> u8 {
        (self.date & 0b11111) as u8
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn relative_year() {
        let base_year = 2000;
        for i in 0..32 {
            let date = super::RTBYDate::new((i << 9) | (1 << 5) | 1);
            assert_eq!(date.year(base_year), base_year + i);
        }
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn month() {
        for i in 1..=12 {
            let date = super::RTBYDate::new((1 << 9) | (i << 5) | 1);
            assert_eq!(date.month(), i as u8);
        }
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn day() {
        for i in 1..=31 {
            let date = super::RTBYDate::new((1 << 9) | (1 << 5) | i);
            assert_eq!(date.day(), i as u8);
        }
    }
}
