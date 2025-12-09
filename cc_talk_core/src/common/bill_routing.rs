#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BillRouteCode {
    Return = 0,
    Stack = 1,
    ExtendEscrow = 255,
}

impl TryFrom<u8> for BillRouteCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Return),
            1 => Ok(Self::Stack),
            255 => Ok(Self::ExtendEscrow),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BillRoutingError {
    EscrowEmpty = 254,
    FailedToRoute = 255,
}

impl TryFrom<u8> for BillRoutingError {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            254 => Ok(Self::EscrowEmpty),
            255 => Ok(Self::FailedToRoute),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StackerCycleError {
    StackerFault = 254,
    StackerNotFitted = 255,
}

impl TryFrom<u8> for StackerCycleError {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            254 => Ok(Self::StackerFault),
            255 => Ok(Self::StackerNotFitted),
            _ => Err(()),
        }
    }
}
