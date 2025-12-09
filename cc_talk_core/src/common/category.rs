/// ccTalk Standard Category Devices
///
/// This enum represents the standard categories of devices that can be connected via the ccTalk
/// protocol.
///
/// You can find the reference in the specification cctalk-part-3-v4-7.pdf section 11.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Category {
    /// Unknown category, used when the category is not specified or recognized.
    Unknown,
    /// Coin validator, a device that accepts coins and validates them.
    CoinAcceptor,
    /// Hopper, a device that dispenses coins or tokens.
    Payout,
    /// Reel
    Reel,
    /// Bank note acceptor, a device that accepts and validates banknotes.
    BillValidator,
    /// NFC card reader, a device that reads NFC cards. I.E. Nayax
    CardReader,
    /// Money-in, money-out recyclers. Also used for coin sungulators and sorters.
    Changer,
    /// Display, Lcd panels, alphanumeric displays, etc.
    Display,
    /// Remote keyboard
    Keypad,
    /// Security device, interface box, hubs, etc.
    Dongle,
    /// Electro-mechanical counter replacement
    Meter,
    /// Bootloader firmware and diagnostics when no application code is loaded
    Bootloader,
    /// Power switching hub or intelligent power supply
    Power,
    /// Ticket printer for coupons, tickets, or receipts
    Printer,
    /// Random number generator
    RNG,
    /// Hopper with weigh scale
    HopperScale,
    /// Motorized coin feeder or singulator
    CoinFeeder,
    /// Bill or note recycler
    BillRecycler,
    /// Motorized Escrow
    Escrow,
    /// Address range for debugging, used when developing a new device
    Debug,
}

impl Category {
    /// Returns the default [Address] for the category.
    #[must_use]
    pub const fn default_address(&self) -> Address {
        match self {
            Self::Unknown => Address::Single(0),
            Self::CoinAcceptor => Address::SingleAndRange(2, 11..=17),
            Self::Payout => Address::SingleAndRange(3, 4..=10),
            Self::Reel => Address::SingleAndRange(30, 31..=34),
            Self::BillValidator => Address::SingleAndRange(40, 41..=47),
            Self::CardReader => Address::Single(50),
            Self::Changer => Address::Single(55),
            Self::Display => Address::Single(60),
            Self::Keypad => Address::Single(70),
            Self::Dongle => Address::SingleAndRange(80, 85..=89),
            Self::Meter => Address::Single(90),
            Self::Bootloader => Address::Single(99),
            Self::Power => Address::Single(100),
            Self::Printer => Address::Single(110),
            Self::RNG => Address::Single(120),
            Self::HopperScale => Address::Single(130),
            Self::CoinFeeder => Address::Single(140),
            Self::BillRecycler => Address::Single(150),
            Self::Escrow => Address::Single(160),
            Self::Debug => Address::SingleAndRange(240, 241..=255),
        }
    }
}

impl From<&str> for Category {
    /// Converts a string to a [Category].
    ///
    /// # Example
    ///
    /// ```
    /// use cc_talk_core::cc_talk::Category;
    ///
    /// let category = Category::from("CoinAcceptor");
    /// assert_eq!(category, Category::CoinAcceptor);
    /// ```
    fn from(category: &str) -> Self {
        // TODO: Find a way to convert to lowercase without alloc
        let category = category.trim();

        if category.eq_ignore_ascii_case("coin acceptor")
            || category.eq_ignore_ascii_case("coinacceptor")
        {
            return Self::CoinAcceptor;
        }

        if category.eq_ignore_ascii_case("payout") {
            return Self::Payout;
        }

        if category.eq_ignore_ascii_case("reel") {
            return Self::Reel;
        }

        if category.eq_ignore_ascii_case("bill validator")
            || category.eq_ignore_ascii_case("billvalidator")
        {
            return Self::BillValidator;
        }

        if category.eq_ignore_ascii_case("card reader")
            || category.eq_ignore_ascii_case("cardreader")
        {
            return Self::CardReader;
        }

        if category.eq_ignore_ascii_case("changer") {
            return Self::Changer;
        }

        if category.eq_ignore_ascii_case("display") {
            return Self::Display;
        }

        if category.eq_ignore_ascii_case("keypad") {
            return Self::Keypad;
        }

        if category.eq_ignore_ascii_case("dongle") {
            return Self::Dongle;
        }

        if category.eq_ignore_ascii_case("meter") {
            return Self::Meter;
        }

        if category.eq_ignore_ascii_case("bootloader") {
            return Self::Bootloader;
        }

        if category.eq_ignore_ascii_case("power") {
            return Self::Power;
        }

        if category.eq_ignore_ascii_case("printer") {
            return Self::Printer;
        }

        if category.eq_ignore_ascii_case("rng") {
            return Self::RNG;
        }

        if category.eq_ignore_ascii_case("hopper scale")
            || category.eq_ignore_ascii_case("hopperscale")
        {
            return Self::HopperScale;
        }

        if category.eq_ignore_ascii_case("coin feeder")
            || category.eq_ignore_ascii_case("coinfeeder")
        {
            return Self::CoinFeeder;
        }

        if category.eq_ignore_ascii_case("bill recycler")
            || category.eq_ignore_ascii_case("billrecycler")
        {
            return Self::BillRecycler;
        }

        if category.eq_ignore_ascii_case("escrow") {
            return Self::Escrow;
        }

        if category.eq_ignore_ascii_case("debug") {
            return Self::Debug;
        }

        Self::Unknown
    }
}

/// Represents a ccTalk device address.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Address {
    // Represents a single address.
    Single(u8),

    // Represents a single address and a range of addresses.
    // This is mainly used to enumerate all default addresses for a category.
    SingleAndRange(u8, core::ops::RangeInclusive<u8>),
}

impl Address {
    /// Checks if an address is part of the address range for this current address.
    ///
    /// # Example
    ///
    /// ```
    /// use cc_talk_core::cc_talk::Address;
    ///
    /// let hopperAddresses = Address::SingleAndRange(3, 4..=10);
    ///
    /// assert!(hopperAddresses.is_in_range(3));
    /// assert!(hopperAddresses.is_in_range(4));
    /// assert!(hopperAddresses.is_in_range(5));
    /// ```
    #[must_use]
    pub fn is_in_range(&self, address: u8) -> bool {
        match self {
            Self::Single(addr) => *addr == address,
            Self::SingleAndRange(addr, range) => *addr == address || range.contains(&address),
        }
    }

    /// Iterates over the addresses in the range.
    ///
    /// If the address is a single address, it will return an iterator with that single address.
    /// Otherwise, it will return an iterator over the range of addresses. It won't fill gaps
    /// between the single address and the range.
    ///
    /// # Example
    ///
    /// ```
    /// use cc_talk_core::cc_talk::Address;
    ///
    /// let hopper_addresses = Address::SingleAndRange(3, 4..=10);
    /// let hopper_iter = hopper_addresses.iter();
    ///
    /// assert_eq!(hopper_iter.collect::<Vec<_>>(), vec![3, 4, 5, 6, 7, 8, 9, 10]);
    ///
    /// let dongle_addresses = Address::SingleAndRange(80, 85..=89);
    /// let dongle_iter = dongle_addresses.iter();
    ///
    /// assert_eq!(dongle_iter.collect::<Vec<_>>(), vec![80, 85, 86, 87, 88, 89]);
    /// ```
    #[must_use]
    pub fn iter(&self) -> AddressIterator {
        self.into_iter()
    }
}

impl IntoIterator for Address {
    type Item = u8;
    type IntoIter = AddressIterator;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Single(addr) => AddressIterator {
                single_addr: Some(addr),
                range_iter: None,
            },
            Self::SingleAndRange(addr, range) => AddressIterator {
                single_addr: Some(addr),
                range_iter: Some(range),
            },
        }
    }
}

impl IntoIterator for &Address {
    type Item = u8;
    type IntoIter = AddressIterator;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Address::Single(addr) => AddressIterator {
                single_addr: Some(*addr),
                range_iter: None,
            },
            Address::SingleAndRange(addr, range) => AddressIterator {
                single_addr: Some(*addr),
                range_iter: Some(range.clone()),
            },
        }
    }
}

pub struct AddressIterator {
    single_addr: Option<u8>,
    range_iter: Option<core::ops::RangeInclusive<u8>>,
}

impl Iterator for AddressIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(addr) = self.single_addr.take() {
            return Some(addr);
        }

        self.range_iter.as_mut()?.next()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AddressMode {
    Other = 0,
    Flash = 1 << 0,
    ROM = 1 << 1,
    EEPROM = 1 << 2,
    InterfaceConnector = 1 << 3,
    PCBLink = 1 << 4,
    Switch = 1 << 5,
    SerialCommandVolatile = 1 << 6,
    SerialCommandNonVolatile = 1 << 7,
}

impl AddressMode {
    /// Returns the value of the address mode as a u8.
    #[must_use]
    pub const fn value(&self) -> u8 {
        match self {
            Self::Other => 0,
            Self::Flash => 1,
            Self::ROM => 2,
            Self::EEPROM => 4,
            Self::InterfaceConnector => 8,
            Self::PCBLink => 16,
            Self::Switch => 32,
            Self::SerialCommandVolatile => 64,
            Self::SerialCommandNonVolatile => 128,
        }
    }

    pub fn from_value(value: u8) -> Option<Self> {
        match value {
            0 => Some(AddressMode::Other),
            1 => Some(AddressMode::Flash),
            2 => Some(AddressMode::ROM),
            4 => Some(AddressMode::EEPROM),
            8 => Some(AddressMode::InterfaceConnector),
            16 => Some(AddressMode::PCBLink),
            32 => Some(AddressMode::Switch),
            64 => Some(AddressMode::SerialCommandVolatile),
            128 => Some(AddressMode::SerialCommandNonVolatile),
            _ => None,
        }
    }

    pub fn available_address_modes(mask: u8) -> heapless::Vec<AddressMode, 8> {
        let mut modes = heapless::Vec::new();
        for i in 0..=7 {
            if mask & (1 << i) != 0 {
                AddressMode::from_value(1 << i).map(|mode| modes.push(mode).ok());
            }
        }
        modes
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn address_iterator_single() {
        let single_address = Address::Single(5);
        for address in single_address {
            assert_eq!(address, 5);
        }
    }

    #[test]
    fn address_iterator_continuous_range() {
        let continuous_range = Address::SingleAndRange(10, 11..=15);
        let mut iter = continuous_range.into_iter();

        assert_eq!(iter.next(), Some(10));
        assert_eq!(iter.next(), Some(11));
        assert_eq!(iter.next(), Some(12));
        assert_eq!(iter.next(), Some(13));
        assert_eq!(iter.next(), Some(14));
        assert_eq!(iter.next(), Some(15));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn address_iterator_non_continous_range() {
        let non_continuous_range = Address::SingleAndRange(1, 20..=21);
        let mut iter = non_continuous_range.into_iter();

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(20));
        assert_eq!(iter.next(), Some(21));
        assert_eq!(iter.next(), None);
    }
}
