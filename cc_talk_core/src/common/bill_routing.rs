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
            0 => Ok(BillRouteCode::Return),
            1 => Ok(BillRouteCode::Stack),
            255 => Ok(BillRouteCode::ExtendEscrow),
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
            254 => Ok(BillRoutingError::EscrowEmpty),
            255 => Ok(BillRoutingError::FailedToRoute),
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
            254 => Ok(StackerCycleError::StackerFault),
            255 => Ok(StackerCycleError::StackerNotFitted),
            _ => Err(()),
        }
    }
}
