#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PowerOption {
    Normal = 0,
    LowPower = 1,
    FullPower = 2,
    Shutdown = 3,
}
