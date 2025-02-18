#![no_std]
#![allow(async_fn_in_trait)]
#![deny(unused_must_use)]

// This must go FIRST so that other mods see its macros.
mod fmt;

// no-chip-specified, maybe it'd be useful to have common api
// or support a mock, in case some tests to validate calculations?
#[cfg(any(feature = "st25r3911b", feature = "st25r3916"))]
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
    Off,
    On,
    Wakeup,
}

pub struct St25r39<I: Interface, IrqPin: InputPin + Wait> {
    iface: I,
    irq: IrqPin,
    irqs: u32,
    mode: Mode,
}
