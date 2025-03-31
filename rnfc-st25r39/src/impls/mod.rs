#[cfg(feature = "st25r3916")]
pub mod lib_st25r3916;
#[cfg(feature = "st25r3916")]
pub use lib_st25r3916::*;

#[cfg(feature = "st25r3911b")]
pub mod lib_st25r3911b;
#[cfg(feature = "st25r3911b")]
pub use lib_st25r3911b::*;

pub mod lib;

// TODO: check wup and other wake-up registers in st25r3911b
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WakeupPeriod {
    /// 10ms
    Ms10 = 0x00,
    /// 20ms
    Ms20 = 0x01,
    /// 30ms
    Ms30 = 0x02,
    /// 40ms
    Ms40 = 0x03,
    /// 50ms
    Ms50 = 0x04,
    /// 60ms
    Ms60 = 0x05,
    /// 70ms
    Ms70 = 0x06,
    /// 80ms
    Ms80 = 0x07,
    /// 100ms
    Ms100 = 0x10,
    /// 200ms
    Ms200 = 0x11,
    /// 300ms
    Ms300 = 0x12,
    /// 400ms
    Ms400 = 0x13,
    /// 500ms
    Ms500 = 0x14,
    /// 600ms
    Ms600 = 0x15,
    /// 700ms
    Ms700 = 0x16,
    /// 800ms
    Ms800 = 0x17,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WakeupConfig {
    pub period: WakeupPeriod,
    pub inductive_amplitude: Option<WakeupMethodConfig>,
    pub inductive_phase: Option<WakeupMethodConfig>,
    pub capacitive: Option<WakeupMethodConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WakeupMethodConfig {
    pub delta: u8,
    pub reference: WakeupReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WakeupReference {
    Manual(u8),
    Automatic,
    AutoAverage { include_irq_measurement: bool, weight: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FieldOnError<T> {
    /// There's some other device emitting its own field, so we shouldn't
    /// turn ours on.
    FieldCollision,
    Interface(T),
    Timeout,
}

impl<T> From<crate::Error<T>> for FieldOnError<T> {
    fn from(val: crate::Error<T>) -> Self {
        match val {
            crate::Error::Interface(e) => FieldOnError::Interface(e),
            crate::Error::Timeout => FieldOnError::Timeout,
        }
    }
}
