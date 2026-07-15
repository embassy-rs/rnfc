use cortex_m::asm::delay;
use embedded_hal::i2c::I2c;

use super::Interface;
use crate::FIFO_SIZE;
use crate::fmt::Bytes;

pub struct I2cInterface<T: I2c> {
    i2c: T,
    address: u8,
}

impl<T: I2c> I2cInterface<T> {
    pub fn new(i2c: T, address: u8) -> Self {
        Self { i2c, address }
    }

    fn read_reg_raw(&mut self, reg: u8) -> u8 {
        let mut buf = [0; 1];
        self.i2c.write_read(self.address, &[reg], &mut buf).unwrap();
        buf[0]
    }

    fn write_reg_raw(&mut self, reg: u8, val: u8) {
        // Retry NACKs a few times before giving up: the WS1850S NACKs its I2C
        // address while in soft power-down, and the NACKed transaction itself
        // wakes the chip (the wake takes ~1 ms) — so the first write after
        // soft power-down legitimately fails once and then succeeds. Real bus
        // faults still panic after the retries are exhausted.
        for _ in 0..5 {
            if self.i2c.write(self.address, &[reg, val]).is_ok() {
                return;
            }
            delay(64_000); // ~1 ms at 64 MHz
        }
        self.i2c.write(self.address, &[reg, val]).unwrap();
    }
}

impl<T: I2c> Interface for I2cInterface<T> {
    fn read_reg(&mut self, reg: usize) -> u8 {
        let reg = reg as u8;
        let res = if reg < 0x40 {
            // Main register
            self.read_reg_raw(reg)
        } else {
            // Extended register
            let reg = reg - 0x40;
            self.write_reg_raw(0x0f, reg | 0x80);
            self.read_reg_raw(0x0f) & 0x3F
        };
        trace!("     read {:02x} = {:02x}", reg, res);
        res
    }

    fn write_reg(&mut self, reg: usize, val: u8) {
        let reg = reg as u8;
        trace!("     write {:02x} = {:02x}", reg, val);

        if reg < 0x40 {
            // Main register
            self.write_reg_raw(reg, val)
        } else {
            // Extended register
            let reg = reg - 0x40;
            self.write_reg_raw(0x0F, reg | 0x40);
            self.write_reg_raw(0x0F, (val & 0x3F) | 0xC0);
        }
    }

    fn read_fifo(&mut self, data: &mut [u8]) {
        if data.len() == 0 {
            return;
        }

        self.i2c.write_read(self.address, &[0x09], data).unwrap();
        trace!("     read_fifo {:02x}", Bytes(data));
    }

    fn write_fifo(&mut self, data: &[u8]) {
        if data.len() == 0 {
            return;
        }

        assert!(data.len() <= FIFO_SIZE);

        let mut buf = [0; FIFO_SIZE + 1];
        buf[0] = 0x09;
        buf[1..1 + data.len()].copy_from_slice(data);
        self.i2c.write(self.address, &buf[..1 + data.len()]).unwrap();
        trace!("     write_fifo {:02x}", Bytes(data));
    }
}
