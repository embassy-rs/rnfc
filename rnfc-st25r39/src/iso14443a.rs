use core::fmt::Debug;

use embassy_time::{Timer, with_timeout};
use rnfc_traits::iso14443a_ll as ll;

use crate::fmt::Bytes;
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<T> {
    Interface(T),
    Timeout,

    Framing,
    FramingLastByteMissingParity,

    Crc,
    Collision,
    Parity,
    ResponseTooShort,
    ResponseTooLong,

    FifoOverflow,
    FifoUnderflow,
}

impl<T: Debug> ll::Error for Error<T> {
    fn kind(&self) -> ll::ErrorKind {
        match self {
            Self::Timeout => ll::ErrorKind::Timeout,
            _ => ll::ErrorKind::Other,
        }
    }
}

impl<T> From<crate::Error<T>> for Error<T> {
    fn from(val: crate::Error<T>) -> Self {
        match val {
            crate::Error::Interface(e) => Error::Interface(e),
            crate::Error::Timeout => Error::Timeout,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StartError<T> {
    Interface(T),
    FieldCollision,
    Timeout,
}

impl<T> From<crate::Error<T>> for StartError<T> {
    fn from(val: crate::Error<T>) -> Self {
        match val {
            crate::Error::Interface(e) => StartError::Interface(e),
            crate::Error::Timeout => StartError::Timeout,
        }
    }
}

/// An ST25 chip enabled in Iso14443a mode.
pub struct Iso14443a<'d, I: Interface, IrqPin: InputPin + Wait> {
    inner: &'d mut St25r39<I, IrqPin>,
}

impl<I: Interface, IrqPin: InputPin + Wait> St25r39<I, IrqPin> {
    pub async fn start_iso14443a(&mut self) -> Result<Iso14443a<'_, I, IrqPin>, FieldOnError<I::Error>> {
        self.mode_on().await?;
        match self.field_on().await {
            Ok(()) => {}
            Err(e) => {
                self.mode_off()?;
                return Err(e);
            }
        }

        // Field on guard time
        Timer::after(Duration::from_millis(5)).await;

        Ok(Iso14443a { inner: self })
    }
}

impl<'d, I: Interface, IrqPin: InputPin + Wait> Drop for Iso14443a<'d, I, IrqPin> {
    fn drop(&mut self) {
        if self.inner.mode_off().is_err() {
            warn!("Failed to set field off on Iso14443a drop");
        }
    }
}

impl<'d, I: Interface + 'd, IrqPin: InputPin + Wait + 'd> ll::Reader for Iso14443a<'d, I, IrqPin> {
    type Error = Error<I::Error>;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8], opts: ll::Frame) -> Result<usize, Self::Error> {
        let this = &mut *self.inner;

        Timer::after(Duration::from_millis(1)).await;
        debug!("TX: {:?} {:02x}", opts, Bytes(tx));

        this.cmd(Command::Stop)?;
        this.cmd(Command::ResetRxgain)?;

        let mut fwt_ms = 5;
        let is_anticoll = matches!(opts, ll::Frame::Anticoll { .. });

        let (raw, cmd) = match opts {
            ll::Frame::ReqA => (true, Command::TransmitReqa),
            ll::Frame::WupA => (true, Command::TransmitWupa),
            ll::Frame::Anticoll { bits } => {
                this.regs().num_tx_bytes2().write_value((bits as u8).into())?;
                this.regs().num_tx_bytes1().write_value((bits >> 8) as u8)?;
                this.iface.write_fifo(&tx[..(bits + 7) / 8]).map_err(Error::Interface)?;
                (true, Command::TransmitWithoutCrc)
            }
            ll::Frame::Standard { timeout_1fc, .. } => {
                fwt_ms = timeout_1fc / 13560 + 1;
                let bits = tx.len() * 8;
                this.regs().num_tx_bytes2().write_value((bits as u8).into())?;
                this.regs().num_tx_bytes1().write_value((bits >> 8) as u8)?;
                this.iface.write_fifo(tx).map_err(Error::Interface)?;
                (false, Command::TransmitWithCrc)
            }
        };
        this.regs().corr_conf1().write(|w| {
            w.0 = 0x13;
            w.set_corr_s6(!is_anticoll);
        })?;

        this.regs().iso14443a_nfc().write(|w| {
            w.set_antcl(is_anticoll);
        })?;
        this.regs().aux().write(|w| {
            w.set_no_crc_rx(raw);
        })?;
        this.regs().rx_conf2().write(|w| {
            // Disable Automatic Gain Control (AGC) for better detection of collisions if using Coherent Receiver
            w.set_agc_en(!is_anticoll);
            w.set_agc_m(true); // AGC operates during complete receive period
            w.set_agc6_3(true); // 0: AGC ratio 3
            w.set_sqm_dyn(true); // Automatic squelch activation after end of TX
        })?;

        this.irqs = 0; // stop already clears all irqs
        this.cmd(cmd)?;

        // Wait for tx ended
        this.irq_wait(Interrupt::Txe).await?;

        // Wait for RX started
        this.irq_wait_timeout(Interrupt::Rxs, Duration::from_millis(fwt_ms as _))
            .await?;

        // Wait for rx ended or error
        // The timeout should never hit, it's just for safety.
        let res = with_timeout(Duration::from_millis(500), async {
            loop {
                if this.irq(Interrupt::Err1) {
                    return Err(Error::Framing);
                }
                if this.irq(Interrupt::Par) {
                    return Err(Error::Parity);
                }
                if this.irq(Interrupt::Crc) {
                    return Err(Error::Crc);
                }
                if !is_anticoll && this.irq(Interrupt::Col) {
                    return Err(Error::Collision);
                }

                if this.irq(Interrupt::Rxe) {
                    break;
                }

                yield_now().await;
                this.irq_update()?;
            }
            Ok(())
        })
        .await;

        match res {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(Error::Timeout),
        }

        // If we're here, RX ended without error.

        let stat = this.regs().fifo_status2().read()?;
        if stat.fifo_ovr() {
            return Err(Error::FifoOverflow);
        }
        if stat.fifo_unf() {
            return Err(Error::FifoUnderflow);
        }
        if stat.np_lb() {
            return Err(Error::FramingLastByteMissingParity);
        }

        let mut rx_bytes = this.regs().fifo_status1().read()? as usize;
        rx_bytes |= (stat.fifo_b() as usize) << 8;

        if let ll::Frame::Anticoll { bits } = opts {
            let full_bytes = bits / 8;
            rx[..full_bytes].copy_from_slice(&tx[..full_bytes]);
            this.iface
                .read_fifo(&mut rx[full_bytes..][..rx_bytes])
                .map_err(Error::Interface)?;
            if bits % 8 != 0 {
                let half_byte = tx[full_bytes] & (1 << bits) - 1;
                rx[full_bytes] |= half_byte
            }

            let rx_bits = if this.irq(Interrupt::Col) {
                let coll = this.regs().collision_status().read()?;
                coll.c_byte() as usize * 8 + coll.c_bit() as usize
            } else {
                full_bytes * 8 + rx_bytes * 8
            };
            debug!("RX: {:02x} bits: {}", Bytes(rx), rx_bits);

            Ok(rx_bits)
        } else {
            // Remove received CRC
            if !raw {
                if rx_bytes < 2 {
                    return Err(Error::ResponseTooShort);
                }
                rx_bytes -= 2;
            }

            if rx.len() < rx_bytes {
                return Err(Error::ResponseTooLong);
            }

            this.iface.read_fifo(&mut rx[..rx_bytes]).map_err(Error::Interface)?;
            debug!("RX: {:02x}", Bytes(&rx[..rx_bytes]));
            Ok(rx_bytes * 8)
        }
    }
}
