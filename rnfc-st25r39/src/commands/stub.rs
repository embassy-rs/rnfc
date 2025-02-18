/// Direct commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(unused)]
pub enum Command {
    Stop,
    SetDefault = 0xCA,
    SetOther = 0xFE,
    // required by iso14443a
    ResetRxgain,
    TransmitReqa,
    TransmitWupa,
    TransmitWithoutCrc,
    TransmitWithCrc,
}
