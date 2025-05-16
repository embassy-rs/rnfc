#![no_std]
#![allow(async_fn_in_trait)]
#![deny(unused_must_use)]

// This must go FIRST so that other mods see its macros.
mod fmt;

#[cfg(all(not(feature = "st25r3911b"), not(feature = "st25r3916")))]
compile_error!("A chip/feature has to be selected in Cargo.toml");

mod aat;
pub mod commands;
pub mod impls;
mod interface;
pub mod iso14443a;
pub mod regs;

use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;
pub use interface::{I2cInterface, Interface, SpiInterface};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<T> {
    Interface(T),
    Timeout,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Mode {
    /// SPI running, AFE static power consumpton mnimized
    Off,
    /// Ready mode
    On,
    /// Low power mode, card presence detection
    Wakeup,
}

pub struct St25r39<I: Interface, IrqPin: InputPin + Wait> {
    iface: I,
    irq: IrqPin,
    irqs: u32,
    mode: Mode,
}
