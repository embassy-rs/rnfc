#[cfg(feature = "st25r3911b")]
mod regs_st25r3911b;

#[cfg(feature = "st25r3911b")]
pub use regs_st25r3911b::*;

#[cfg(feature = "st25r3916")]
mod regs_st25r3916;
#[cfg(feature = "st25r3916")]
pub use regs_st25r3916::*;

#[cfg(all(not(feature = "st25r3911b"), not(feature = "st25r3916")))]
mod stub;
use core::marker::PhantomData;

#[cfg(all(not(feature = "st25r3911b"), not(feature = "st25r3916")))]
pub use stub::*;

use crate::interface::Interface;
use crate::Error;

// TODO: if this api is set, then maybe one somehow could remove some bolierplate generation for regs
pub struct Reg<'a, I: Interface, T: Copy> {
    addr: u8,
    iface: &'a mut I,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, I: Interface, T: Copy + Into<u8> + From<u8>> Reg<'a, I, T> {
    pub fn new(iface: &'a mut I, addr: u8) -> Self {
        Self {
            iface,
            addr,
            phantom: PhantomData,
        }
    }

    pub fn read(&mut self) -> Result<T, Error<I::Error>> {
        Ok(self.iface.read_reg(self.addr).map_err(Error::Interface)?.into())
    }

    pub fn write_value(&mut self, val: T) -> Result<(), Error<I::Error>> {
        self.iface.write_reg(self.addr, val.into()).map_err(Error::Interface)
    }

    pub fn modify<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> Result<R, Error<I::Error>> {
        let mut val = self.read()?;
        let res = f(&mut val);
        self.write_value(val)?;
        Ok(res)
    }
}

impl<'a, I: Interface, T: Default + Copy + Into<u8> + From<u8>> Reg<'a, I, T> {
    pub fn write<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> Result<R, Error<I::Error>> {
        let mut val = Default::default();
        let res = f(&mut val);
        self.write_value(val)?;
        Ok(res)
    }
}

// just for stub
#[cfg(not(any(feature = "st25r3911b", feature = "st25r3916")))]
pub struct Regs<'a, I: Interface> {
    iface: &'a mut I,
}
