#![no_std]
#![allow(async_fn_in_trait)]

// Must go FIRST so that other mods see its macros.
mod fmt;

mod interface;
pub mod iso14443a;
mod regs;

use core::convert::Infallible;

use embassy_time::{Duration, Instant, TimeoutError, Timer, with_timeout};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::digital::Wait;
pub use interface::*;
use regs::Regs;
pub use regs::Rxgain as RxGain;

#[derive(Clone, Copy)]
pub struct RfConfig {
    /// NMOS carrier wave drive strength. 0..=15
    pub n_drive_cw: u8,
    /// NMOS carrier wave drive strength when modulating. 0..=15
    pub n_drive_mod: u8,
    /// PMOS carrier wave drive strength. 0..=63
    pub p_drive_cw: u8,
    /// PMOS carrier wave drive strength when modulating. 0..=63
    pub p_drive_mod: u8,
    /// RX gain.
    pub rx_gain: RxGain,
    /// Min rx level. 0..=15
    pub minlevel: u8,
    /// Collision rx level. 0..=15
    pub colllevel: u8,
}

impl Default for RfConfig {
    fn default() -> Self {
        Self {
            n_drive_cw: 8,
            n_drive_mod: 8,
            p_drive_cw: 32,
            p_drive_mod: 32,
            rx_gain: RxGain::_33DB,
            minlevel: 8,
            colllevel: 4,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WakeupConfig {
    /// sleep time.
    /// Valid values: 1-15.
    /// Tsleep = (val + 2) * 100ms
    pub sleep_time: u8,

    /// mesaure prepare time?
    /// Valid values: 2-31
    /// Tprepare = (val + 2) * 100us
    pub prepare_time: u8,

    /// measure time.
    /// Valid values: 2-31
    /// Tmeasure = (val - 1) * 4.7us
    pub measure_time: u8,

    /// Wakeup threshold for ADC readings.
    ///
    /// If the ADC reading differs by more than `threshold` from the reading at calibration, wakeup is triggered.
    pub threshold: u8,

    // NMOS carrier wave drive strength. 0..=1
    pub n_drive: u8,
    // PMOS carrier wave drive strength. 0..=7
    pub p_drive: u8,

    pub recalibrate_interval: Option<Duration>,
}

const FIFO_SIZE: usize = 64;

const ADC_REFERENCE_MIN: u8 = 0;
const ADC_REFERENCE_MAX: u8 = 0x7F;

pub struct Fm175xx<I, NpdPin, IrqPin> {
    iface: I,
    npd: NpdPin,
    irq: IrqPin,
    config: RfConfig,
}

impl<I, NpdPin, IrqPin> Fm175xx<I, NpdPin, IrqPin>
where
    I: Interface,
    NpdPin: OutputPin,
    IrqPin: InputPin + Wait,
{
    pub async fn new(iface: I, mut npd: NpdPin, irq: IrqPin) -> Self {
        npd.set_low().unwrap();

        Self {
            iface,
            npd,
            irq,
            config: Default::default(),
        }
    }

    fn regs(&mut self) -> Regs<I> {
        Regs {
            iface: &mut self.iface,
            addr: 0,
        }
    }

    fn off(&mut self) {
        self.npd.set_low().unwrap();
    }

    async fn on(&mut self) {
        self.npd.set_low().unwrap();

        // datasheet says reset takes 5ms.
        Timer::after(Duration::from_millis(5)).await;

        self.npd.set_high().unwrap();

        // datasheet doesn't say anything about time after reset, wait a bit just in case.
        Timer::after(Duration::from_millis(1)).await;

        debug!("softreset");
        self.regs().command().write(|w| w.set_command(regs::CommandVal::SOFTRESET));

        let deadline = Instant::now() + Duration::from_secs(1);
        while self.regs().command().read().command() != regs::CommandVal::IDLE {
            if Instant::now() > deadline {
                warn!("timeout waiting for softreset.");
                break;
            }
        }

        // again, just in case
        Timer::after(Duration::from_millis(1)).await;

        //let ver = self.regs().version().read();
        //debug!("IC version: {:02x}", ver);
    }

    fn rf_on(&mut self) {
        let config = self.config;

        self.regs().gsn().write(|w| {
            w.set_cwgsn(config.n_drive_cw); // reset value: 8
            w.set_modgsn(config.n_drive_mod); // reset value: 8
        });
        self.regs().cwgsp().write(|w| {
            w.set_cwgsp(config.p_drive_cw); // reset value: 32
        });
        self.regs().modgsp().write(|w| {
            w.set_modgsp(config.p_drive_mod); // reset value: 32
        });

        self.regs().command().write(|w| {
            w.set_powerdown(false);
            w.set_rcvoff(false);
        });

        self.regs().txcontrol().write(|w| {
            w.set_tx1rfen(true);
            w.set_tx2rfen(true);
            w.set_invtx2on(true);
        });
    }

    pub async fn sleep(&mut self) {
        self.on().await;

        // lpcd reset
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(false); // clear bits written with 1
            w.set_rstn(true); // nRST=0
            w.set_en(true); // EN=0
            w.set_ie(true); // IE=0
            w.set_calibra_en(true); // CALIBRA_EN=0
        });
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(true); // set bits written with 1
            w.set_rstn(true); // nRST=1
        });

        // lpcd disable
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(false); // clear bits written with 1
            w.set_rstn(true); // nRST=0
            w.set_en(true); // EN=0
            w.set_ie(true); // IE=0
            w.set_calibra_en(true); // CALIBRA_EN=0
        });

        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(false); // clear bits written with 1
            w.set_en(true); // EN=0
        });

        self.regs().lpcd_ctrl3().write(|w| {
            w.set_hpden(false);
        });

        Timer::after(Duration::from_millis(1)).await; // give it some time

        self.off();
    }

    pub async fn wait_for_card(&mut self, config: WakeupConfig) -> Result<(), Infallible> {
        assert!((1..=15).contains(&config.sleep_time));
        assert!((2..=31).contains(&config.prepare_time));
        assert!((2..=31).contains(&config.measure_time));
        assert!((0..=1).contains(&config.n_drive));
        assert!((0..=7).contains(&config.p_drive));

        loop {
            self.on().await;

            //self.regs().command().write(|_| {});
            self.regs().commien().write(|w| w.set_irqinv(true));
            self.regs().divien().write(|w| w.set_irqpushpull(true));

            // lpcd reset + enable
            self.regs().lpcd_ctrl1().write(|w| {
                w.set_bit_ctrl_set(false); // clear bits written with 1
                w.set_rstn(true); // nRST=0
                w.set_calibra_en(true); // CALIBRA_EN=0
            });
            self.regs().lpcd_ctrl1().write(|w| {
                w.set_bit_ctrl_set(true); // set bits written with 1
                w.set_rstn(true); // nRST=1
                w.set_en(true); // EN=1
                w.set_ie(true); // IE=1
                w.set_sense_1(true); // SENSE1 = 1
            });

            self.regs().lpcd_ctrl2().write(|w| {
                w.set_tx2en(true);
                w.set_cwn(config.n_drive == 1);
                w.set_cwp(config.p_drive);
            });

            self.regs().lpcd_ctrl3().write(|w| w.set_hpden(false));

            let (t3clkdiv, adc_shift) = match config.measure_time {
                16.. => (regs::LpcdT3clkdivk::DIV16, 3),
                8.. => (regs::LpcdT3clkdivk::DIV8, 4),
                0.. => (regs::LpcdT3clkdivk::DIV4, 5),
            };

            let adc_range = (config.measure_time - 1) << adc_shift;
            let adc_center = adc_range / 2;

            debug!("adc: range={} center={}", adc_range, adc_center);

            self.regs().lpcd_t1cfg().write(|w| {
                w.set_t1cfg(config.sleep_time);
                w.set_t3clkdivk(t3clkdiv);
            });
            self.regs().lpcd_t2cfg().write(|w| w.set_t2cfg(config.prepare_time));
            self.regs().lpcd_t3cfg().write(|w| w.set_t3cfg(config.measure_time));
            self.regs().lpcd_vmid_bd_cfg().write(|w| w.set_vmid_bd_cfg(8));
            self.regs().lpcd_auto_wup_cfg().write(|w| w.set_en(false));

            self.regs().lpcd_misc().write(|w| w.set_calib_vmid_en(true));

            // Calibrate! Note that:
            // - Higher gain -> lower ADC reading
            // - Higher reference voltage -> lower ADC reading

            // First, find lowest gain (multiplier/divider) that satisfies "reading < center".
            self.lpcd_set_adc_config(ADC_REFERENCE_MAX, 0);
            let levels: [u8; 32] = [
                0, 4, 2, 8, 6, 1, 10, 5, 12, 16, 9, 3, 14, 20, 18, 7, 24, 22, 13, 11, 17, 26, 28, 21, 15, 30, 25, 19, 23, 29,
                27, 31,
            ];

            /*
            for level in 0..levels.len() {
                let mut vals = [0; ADC_REFERENCE_MAX as usize + 1];
                for reference in ADC_REFERENCE_MIN..=ADC_REFERENCE_MAX {
                    self.regs().lpcd_ctrl4().write_value(levels[level].into());
                    self.lpcd_set_adc_config(reference as _, 0);

                    vals[reference as usize] = self.lpcd_read_adc();
                }
                info!("level={} {}", level, vals);
                embassy_futures::yield_now().await;
            }

            return Ok(());
            */

            let mut failed = false;

            let level = binary_search(0, levels.len() as _, |val| {
                self.regs().lpcd_ctrl4().write_value(levels[val as usize].into());
                let meas = self.lpcd_read_adc();
                let res = meas < adc_center;
                debug!("adc search level: {} => {} {}", val, meas, res);
                res
            });
            let level = match level {
                Some(x) => x as usize,
                None => {
                    warn!("Gain calibration failed.");
                    failed = true;
                    levels.len() - 1
                }
            };
            debug!("adc level {}", level);
            self.regs().lpcd_ctrl4().write_value(levels[level].into());

            // Second, find lowest reference voltage that satisfies "reading < center".
            let reference = binary_search(ADC_REFERENCE_MIN as _, ADC_REFERENCE_MAX as _, |val| {
                self.lpcd_set_adc_config(val as _, 0);
                let meas = self.lpcd_read_adc();
                let res = meas < adc_center;
                debug!("adc search refer: {} => {} {}", val, meas, res);
                res
            });
            let reference = match reference {
                Some(x) => x as u8,
                None => {
                    warn!("Reference voltage calibration failed.");
                    failed = true;
                    ADC_REFERENCE_MAX
                }
            };
            debug!("adc refer {}", reference);
            self.lpcd_set_adc_config(reference, 0);

            // Configure threshold based on current reading.
            let curr = self.lpcd_read_adc();
            let threshold_offs = ((adc_range as u32) * (config.threshold as u32) / 256) as u8;
            let threshold_min = curr.saturating_sub(threshold_offs);
            let threshold_max = curr.saturating_add(threshold_offs);
            debug!(
                "adc: curr={} threshold_offs={} threshold_min={} threshold_max={}",
                curr, threshold_offs, threshold_min, threshold_max
            );
            self.regs().lpcd_threshold_min_l().write_value(threshold_min & 0x3F);
            self.regs().lpcd_threshold_min_h().write_value(threshold_min >> 6);
            self.regs().lpcd_threshold_max_l().write_value(threshold_max & 0x3F);
            self.regs().lpcd_threshold_max_h().write_value(threshold_max >> 6);

            /*
            loop {
                let r = self.lpcd_read_adc();
                if r < threshold_min || r > threshold_max {
                    info!(" res: {=u8} ====== CARD DETECTED", r);
                } else {
                    info!(" res: {=u8}", r);
                }
                Timer::after(Duration::from_millis(30)).await;
            }
            */

            self.regs().lpcd_misc().write(|w| w.set_calib_vmid_en(false));
            self.regs().lpcd_auto_wup_cfg().write(|w| {
                w.set_en(false);
                w.set_time(regs::LpcdAutoWupTime::_1HOUR);
            });

            self.regs().lpcd_ctrl1().write(|w| {
                w.set_bit_ctrl_set(false);
                w.set_rstn(true); // nRST = 0
            });
            self.regs().lpcd_ctrl1().write(|w| {
                w.set_bit_ctrl_set(true);
                w.set_rstn(true); // nRST = 1
            });

            //self.dump();

            self.npd.set_low().unwrap();

            let dur = if failed {
                // if calibration failed, force recalibrate very soon.
                Duration::from_secs(10)
            } else {
                config.recalibrate_interval.unwrap_or(Duration::from_secs(3 * 60 * 60))
            };

            info!("Waiting for irq...");
            match with_timeout(dur, self.irq.wait_for_low()).await {
                Ok(Ok(())) => {
                    info!("Got irq!");

                    //self.npd.set_high().unwrap();
                    //Timer::after(Duration::from_millis(1)).await;
                    //self.dump();
                    //self.regs().lpcd_misc().write(|w| w.set_calib_vmid_en(true));
                    //info!(" NOW READ: {=u8}", self.lpcd_read_adc());

                    return Ok(());
                }
                Ok(Err(_)) => warn!("irq.wait_for_low() error"),
                Err(TimeoutError) => info!("timed out, recalibrating..."),
            }
        }
    }

    fn _dump(&mut self) {
        info!("==============");
        info!(
            "comirq {:02x} divirq {:02x} lpcdirq {:02x}",
            self.regs().commirq().read().0,
            self.regs().divirq().read().0,
            self.regs().lpcd_irq().read().0
        );
        info!(
            "ctrl1={:02x} ctrl2={:02x} ctrl3={:02x} ctrl4={:02x} misc={:02x}",
            self.regs().lpcd_ctrl1().read().0,
            self.regs().lpcd_ctrl2().read().0,
            self.regs().lpcd_ctrl3().read().0,
            self.regs().lpcd_ctrl4().read().0,
            self.regs().lpcd_misc().read().0,
        );
        info!(
            "t1cfg={:02x} t2cfg={:02x} t3cfg={:02x} adcref={:02x} adcbcu={:02x}",
            self.regs().lpcd_t1cfg().read().0,
            self.regs().lpcd_t2cfg().read().0,
            self.regs().lpcd_t3cfg().read().0,
            self.regs().lpcd_adc_referece().read(),
            self.regs().lpcd_bias_current().read().0,
        );

        info!("adc val {}", self.lpcd_get_adc_value());
    }

    fn lpcd_set_adc_config(&mut self, reference: u8, bias_current: u8) {
        self.regs().lpcd_adc_referece().write_value(reference & 0x3F);
        self.regs().lpcd_bias_current().write(|w| {
            w.set_adc_referece_h((reference >> 6) != 0);
            w.set_bias_current(bias_current);
        });
    }

    fn lpcd_read_adc(&mut self) -> u8 {
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(false);
            w.set_rstn(true); // nRST = 0
            w.set_calibra_en(true); // calibra_en = 0
        });
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(true);
            w.set_rstn(true); // nRST = 1
        });
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(true);
            w.set_calibra_en(true); // calibra_en = 1
        });

        //cortex_m::asm::delay(640_000); // 100ms

        //info!("calib: waiting for irq..");
        let deadline = Instant::now() + Duration::from_secs(1);
        while !self.regs().lpcd_irq().read().calib_irq() {
            if Instant::now() > deadline {
                warn!("timeout waiting for adc calibration.");
                break;
            }
        }

        // calibra_en = 0
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(false);
            w.set_calibra_en(true);
        });

        self.lpcd_get_adc_value()
    }

    fn lpcd_get_adc_value(&mut self) -> u8 {
        let h = self.regs().lpcd_adc_result_h().read();
        let l = self.regs().lpcd_adc_result_l().read();
        ((h & 0x3) << 6) | (l & 0x3f)
    }

    fn clear_fifo(&mut self) {
        self.regs().fifolevel().write(|w| w.set_flushfifo(true));
    }

    fn set_timer(&mut self, onefc: u32) {
        let mut prescaler: u32 = 0;
        let mut timereload: u32 = 0;
        while prescaler < 0xfff {
            timereload = (onefc - 1).div_ceil(prescaler * 2 + 1);

            if timereload < 0xffff {
                break;
            }
            prescaler += 1;
        }
        timereload = timereload & 0xFFFF;
        self.regs().tmode().write(|w| {
            w.set_tauto(true);
            w.set_tprescaler_hi((prescaler >> 8) as u8);
        });
        self.regs().tprescaler().write_value(prescaler as u8);
        self.regs().treloadhi().write_value((timereload >> 8) as u8);
        self.regs().treloadlo().write_value(timereload as u8);
    }

    /*
    fn transceive(&mut self, tx: &[u8], rx: &mut [u8], timeout_1fc: u32) -> Result<usize, Error> {
        let (len, bits) = self.transceive_raw(tx, rx, timeout_1fc, true, 0)?;
        if bits != 0 {
            warn!("incomplete last byte (got {=u8} bits)", bits);
            return Err(Error::Other);
        }
        Ok(len)
    }
     */

    pub fn raw(&mut self) -> Raw<'_, I, NpdPin, IrqPin> {
        Raw { inner: self }
    }

    pub fn set_config(&mut self, config: RfConfig) {
        self.config = config;
    }
}

/// Find lowest value in min..max (min included, max excluded)
/// satisfying `f(val) = true`.
///
/// If `f` returns `false` for all values, returns `None`.
///
/// `f` is assumed to be monotonically increasing.
fn binary_search(mut min: i32, mut max: i32, mut f: impl FnMut(i32) -> bool) -> Option<i32> {
    let orig_max = max;
    min -= 1;
    while min + 1 < max {
        let m = (min + max) / 2;
        if f(m) { max = m } else { min = m }
    }
    if max == orig_max { None } else { Some(max) }
}

pub struct Raw<'a, I, NpdPin, IrqPin>
where
    I: Interface,
    NpdPin: OutputPin,
    IrqPin: InputPin + Wait,
{
    inner: &'a mut Fm175xx<I, NpdPin, IrqPin>,
}

impl<'a, I, NpdPin, IrqPin> Raw<'a, I, NpdPin, IrqPin>
where
    I: Interface,
    NpdPin: OutputPin,
    IrqPin: InputPin + Wait,
{
    pub async fn field_on(&mut self) -> Result<(), Infallible> {
        self.inner.on().await;
        self.inner.rf_on();

        Ok(())
    }
    pub async fn field_off(&mut self) -> Result<(), Infallible> {
        self.inner.off();
        Ok(())
    }
    pub async fn driver_hi_z(&mut self) -> Result<(), Infallible> {
        self.inner.off();
        Ok(())
    }
}
