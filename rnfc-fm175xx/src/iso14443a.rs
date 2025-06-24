use embassy_futures::yield_now;
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::digital::Wait;
use rnfc_traits::iso14443a_ll as ll;

use crate::fmt::Bytes;
use crate::{FIFO_SIZE, Fm175xx, Interface, regs};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Other,
    Timeout,
    Crc,
    Protocol,
    Collision,
}

impl ll::Error for Error {
    fn kind(&self) -> ll::ErrorKind {
        match self {
            Error::Other => ll::ErrorKind::Other,
            Error::Timeout => ll::ErrorKind::Timeout,
            Error::Protocol => ll::ErrorKind::Corruption,
            Error::Crc => ll::ErrorKind::Corruption,
            Error::Collision => ll::ErrorKind::Corruption,
        }
    }
}

pub struct Iso14443a<'d, I: Interface, NpdPin, IrqPin>
where
    I: Interface + 'd,
    NpdPin: OutputPin + 'd,
    IrqPin: InputPin + Wait + 'd,
{
    inner: &'d mut Fm175xx<I, NpdPin, IrqPin>,
}

impl<I: Interface, NpdPin, IrqPin> Fm175xx<I, NpdPin, IrqPin>
where
    NpdPin: OutputPin,
    IrqPin: InputPin + Wait,
{
    pub async fn start_iso14443a(&mut self) -> Result<Iso14443a<I, NpdPin, IrqPin>, Error> {
        self.on().await;

        self.regs().txmode().write(|w| {
            w.set_framing(regs::Framing::ISO14443A);
            w.set_speed(regs::Speed::_106KBPS);
        });
        self.regs().rxmode().write(|w| {
            w.set_framing(regs::Framing::ISO14443A);
            w.set_speed(regs::Speed::_106KBPS);
        });
        self.regs().modwidth().write_value(0x27);
        self.regs().control().write(|w| {
            w.set_initiator(true);
        });
        let config = self.config.clone();
        self.regs().rfcfg().write(|w| {
            w.set_rxgain(config.rx_gain);
        });
        self.regs().rxtreshold().write(|w| {
            w.set_collevel(config.colllevel);
            w.set_minlevel(config.minlevel);
        });
        self.regs().txauto().write(|w| {
            w.set_force100ask(true);
        });

        self.rf_on();

        // Field on guard time
        Timer::after(Duration::from_millis(5)).await;

        Ok(Iso14443a { inner: self })
    }
}

impl<'d, I, NpdPin, IrqPin> Drop for Iso14443a<'d, I, NpdPin, IrqPin>
where
    I: Interface + 'd,
    NpdPin: OutputPin + 'd,
    IrqPin: InputPin + Wait + 'd,
{
    fn drop(&mut self) {
        self.inner.off();
    }
}

impl<'d, I, NpdPin, IrqPin> ll::Reader for Iso14443a<'d, I, NpdPin, IrqPin>
where
    I: Interface + 'd,
    NpdPin: OutputPin + 'd,
    IrqPin: InputPin + Wait + 'd,
{
    type Error = Error;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8], opts: ll::Frame) -> Result<usize, Self::Error> {
        debug!("TX: {:?} {:02x}", opts, Bytes(tx));

        let r = &mut *self.inner;

        let (tx, crc, timeout_1fc, lastbits, rxalign) = match opts {
            ll::Frame::Anticoll { bits } => (&tx[..(bits + 7) / 8], false, 65536, (bits % 8) as u8, (bits % 8) as u8),
            ll::Frame::ReqA => (&[0x26][..], false, 16384, 7, 0),
            ll::Frame::WupA => (&[0x52][..], false, 16384, 7, 0),
            ll::Frame::Standard { timeout_1fc } => (tx, true, timeout_1fc, 0, 0),
        };

        // Set CRC
        r.regs().txmode().modify(|w| w.set_crcen(crc));
        r.regs().rxmode().modify(|w| w.set_crcen(crc));

        // Set timeout
        r.set_timer(timeout_1fc);

        // Halt whatever currently running command.
        r.regs().command().write(|w| {
            w.set_command(regs::CommandVal::IDLE);
        });

        // Clear all IRQs
        r.regs().divirq().write_value(0x7f.into());
        r.regs().commirq().write_value(0x7f.into());

        r.clear_fifo();

        r.regs().coll().write(|w| {
            w.set_valuesaftercoll(!matches!(opts, ll::Frame::Anticoll { .. }));
        });

        let mut collision = false;

        let mut tx_pos = 0;
        let mut write_fifo = |r: &mut Fm175xx<I, NpdPin, IrqPin>| {
            if tx_pos >= tx.len() {
                return Ok::<(), Error>(());
            }

            let used = r.regs().fifolevel().read().level() as usize;
            let free = FIFO_SIZE - used;
            let n = free.min(tx.len() - tx_pos);
            r.iface.write_fifo(&tx[tx_pos..][..n]);
            tx_pos += n;
            Ok(())
        };

        let mut rx_pos = 0;
        let mut read_fifo = |r: &mut Fm175xx<I, NpdPin, IrqPin>| {
            let bytes = r.regs().fifolevel().read().level() as usize;
            if rx_pos + bytes > rx.len() {
                warn!("rx overflow! received {} but buffer is only {}", rx_pos + bytes, rx.len());
                return Err(Error::Other);
            }
            r.iface.read_fifo(&mut rx[rx_pos..][..bytes]);
            rx_pos += bytes;
            Ok(())
        };

        // Fill FIFO as much as we can, to begin with.
        write_fifo(r)?;

        // Start trx
        r.regs().command().write(|w| {
            w.set_command(regs::CommandVal::TRANSCEIVE);
        });

        r.regs().bitframing().write(|w| {
            w.set_startsend(true);
            w.set_rxalign(rxalign);
            w.set_txlastbits(lastbits);
        });

        let mut tx_done = false;
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            // make sure to not loop forever if timeri never fires for whatever reason.
            if Instant::now() > deadline {
                warn!("emergency timeout");
                return Err(Error::Other);
            }

            let mut irqs = r.regs().commirq().read();

            if irqs.timeri() {
                trace!("irq: timeri");
                return Err(Error::Timeout);
            }

            if irqs.erri() {
                trace!("irq: ERR");
                let errs = r.regs().error().read();
                if errs.collerr() {
                    debug!("err: collision");
                    collision = true;
                    //break;
                }
                if errs.bufferovfl() {
                    warn!("err: buffer overflow");
                    return Err(Error::Other);
                }
                if errs.crcerr() {
                    warn!("err: bad CRC");
                    return Err(Error::Crc);
                }
                if errs.parityerr() && !collision {
                    warn!("err: parity");
                    return Err(Error::Crc);
                }
                if errs.proterr() {
                    warn!("err: protocol");
                    return Err(Error::Protocol);
                }
                if errs.rferr() {
                    warn!("err: rf");
                    return Err(Error::Protocol);
                }
                if errs.temperr() {
                    warn!("err: temperature");
                    return Err(Error::Other);
                }
                if errs.wrerr() {
                    warn!("err: write access error??");
                    return Err(Error::Other);
                }
            }

            if irqs.txi() {
                trace!("irq: tx done");
                tx_done = true;
            }
            if irqs.rxi() {
                trace!("irq: rx done");
                break;
            }

            irqs.set_set(false);
            r.regs().commirq().write_value(irqs);

            if tx_done {
                read_fifo(r)?;
            } else {
                write_fifo(r)?;
            }

            yield_now().await;
        }

        if tx_pos != tx.len() {
            warn!("TX fifo underflow (tx done fired before we wrote the bytes)");
            return Err(Error::Other);
        }

        read_fifo(r)?;

        if let ll::Frame::Anticoll { bits } = opts {
            let shift = bits / 8;
            rx[rx_pos + shift..].fill(0);
            for i in (0..rx_pos).rev() {
                rx[i + shift] = rx[i];
            }
            rx[..shift].copy_from_slice(&tx[..shift]);
            if bits % 8 != 0 {
                let byte_part = tx[bits / 8];
                let mask = 1u8 << (bits % 8) - 1;
                rx[bits / 8] |= byte_part & mask;
            }
            for i in 0..(bits + 7) / 8 {
                rx[i] |= tx[i];
            }

            // Collision at bit `i` means that bit is not valid, only `0..i-1` are.
            // substract 1 because collpos is 1-based, not 0-based (why??)
            let total_bits = if collision {
                let coll = r.regs().coll().read();
                if coll.collposnotvalid() {
                    warn!("collision position out of range");
                    return Err(Error::Protocol);
                }

                let mut collpos = coll.collpos() as usize;
                if collpos == 0 {
                    collpos = 32;
                }
                debug!("collision at: collpos={}", collpos);
                bits + collpos - 1
            } else {
                bits / 8 * 8 + rx_pos * 8
            };

            debug!("RX: {:02x} bits: {}", Bytes(rx), total_bits);

            Ok(total_bits)
        } else {
            if collision {
                return Err(Error::Collision);
            }
            debug!("RX: {:02x}", Bytes(&rx[..rx_pos]));
            Ok(rx_pos * 8)
        }
    }
}
