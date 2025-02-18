use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

use super::WakeupConfig;
use crate::commands::Command;
use crate::impls::FieldOnError;
use crate::regs::Regs;
use crate::{Error, Interface, Mode, St25r39};

impl<I: Interface, IrqPin: InputPin + Wait> St25r39<I, IrqPin> {
    pub async fn new(iface: I, irq: IrqPin) -> Result<Self, Error<I::Error>> {
        Ok(Self {
            iface,
            irq,
            irqs: 0,
            mode: Mode::Off,
        })
    }
    pub async fn mode_on(&mut self) -> Result<(), FieldOnError<I::Error>> {
        self.mode = Mode::On;
        let _ = Mode::Wakeup;
        Ok(())
    }
    pub fn mode_off(&mut self) -> Result<(), FieldOnError<I::Error>> {
        self.mode = Mode::Off;
        let _ = self.irq;
        Ok(())
    }
    pub async fn field_on(&self) -> Result<(), FieldOnError<I::Error>> {
        Ok(())
    }
    pub async fn field_off(&self) -> Result<(), Error<I::Error>> {
        Ok(())
    }
    pub fn cmd(&mut self, _cmd: Command) -> Result<(), Error<I::Error>> {
        Ok(())
    }
    pub fn regs(&mut self) -> Regs<I> {
        crate::regs::Regs::new(&mut self.iface)
    }
    pub fn irq_update(&mut self) -> Result<(), Error<I::Error>> {
        Ok(())
    }
    pub fn irq(&mut self, _irq: Interrupt) -> bool {
        true
    }
    pub async fn wait_for_card(&self, _: WakeupConfig) -> Result<(), Error<I::Error>> {
        Ok(())
    }
}

pub enum Interrupt {
    // required by iso14443a
    Err1,
    Par,
    Crc,
    Col,
    Rxe,
}
