#![no_std]
#![allow(async_fn_in_trait)]

// Must go FIRST so that other mods see its macros.
mod fmt;

mod interface;
pub mod iso14443a;
mod regs;

// Exactly one chip must be selected at compile time.
#[cfg(all(feature = "fm175xx", feature = "ws1850s"))]
compile_error!("features `fm175xx` and `ws1850s` are mutually exclusive: enable exactly one.");
#[cfg(not(any(feature = "fm175xx", feature = "ws1850s")))]
compile_error!("no chip selected: enable exactly one of the `fm175xx` or `ws1850s` features.");

use core::convert::Infallible;

use embassy_time::{Duration, Instant, Timer};
#[cfg(feature = "fm175xx")]
use embassy_time::{TimeoutError, with_timeout};
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

/// LPCD wakeup configuration for the FM17550 (and FM17xx family).
///
/// The FM17550 drives the *extended* LPCD register page (accessed via reg
/// `0x0F`) and needs software ADC auto-ranging + periodic recalibration.
#[cfg(feature = "fm175xx")]
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

/// LPCD wakeup configuration for the WS1850S (Wisesun).
///
/// The WS1850S drives its LPCD detector through the `VersionReg`-unlocked
/// Page4/Page6 banks (main-page addresses `0x31..0x3E`). It calibrates its
/// wake reference in hardware on each LPCD *entry* (`CalibEn`), searching
/// CWGsP near `cwgsp_lpcd` for the ADC closest to `adc_ref`; see
/// `equilibrium_settle` for when that reference is sampled.
#[cfg(feature = "ws1850s")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WakeupConfig {
    /// Sleep period between probes (`WUPeriodReg`, `0x3D`).
    /// `T_inactivity = wu_period * 256 * Tclk_32k`.
    /// e.g. `0x0D` ⇒ ~101 ms, `0x20` ⇒ 250 ms, `0x40` ⇒ 500 ms. Reset `0x0F`.
    pub wu_period: u8,

    /// Trigger offset from the DAC reference (`LPCDReg` bits[3:0], `0x3C`). 0..=15.
    ///
    /// A field change beyond ±delta wakes the chip. Larger delta → shorter
    /// detect distance but more noise immunity. AN602 recommends delta ≥ 4.
    /// This is the dominant false-wakeup vs sensitivity knob.
    pub delta: u8,

    /// Probe duration (`SwingsCntReg` bits[3:0], `0x3E`). 0..=15. Reset 4.
    /// `T_detect = swings_cnt * 16 * 2 * Tclk_27M12` (e.g. 5 ⇒ ~5.9 µs).
    pub swings_cnt: u8,

    /// Fire `TagDetIrq` only after detecting a card `skip + 1` times
    /// (`SwingsCntReg` bits[6:4], `0x3E`) — a debounce/false-wakeup filter. 0..=7.
    pub skip: u8,

    /// N-driver conductance during LPCD (`CWGsN_lpcd`, `P5_Reg38` high nibble,
    /// `0x38`). 0..=15. Together with `cwgsp_lpcd` sets the LPCD carrier field.
    pub cwgsn_lpcd: u8,

    /// P-driver conductance during LPCD (`CWGsP_lpcd`, `P5_Reg39` bits[5:0],
    /// `0x39`). 0..=63. Starting point of the entry calibration search, which
    /// walks ±8 around it (CalibMode=1, CalibStep=0) and parks where the ADC
    /// lands closest to `adc_ref` (AN602 §10.3.2).
    pub cwgsp_lpcd: u8,

    /// ADC target of the calibration search (`LPCDADCRef`, `P5_Reg36`, `0x36`).
    /// Reset 0x80 (mid-scale ≈ AN602 §9's "field around half of max"). Bias it
    /// toward the ADC value of the intended parking point when neighboring
    /// points are equidistant from mid-scale, so parking is deterministic.
    /// Note: with a search range confined to the saturated plateau (high
    /// drive), the target cannot influence the result.
    pub adc_ref: u8,

    /// Extra low bits ORed into `P4_Reg33` (`0x33`, bits\[4:0\]):
    /// `LPCDADCManEn`\[4\], `LPCDEnRCcal`\[3\], `RC32KCalMan`\[2\],
    /// `RC27MCalMan`\[1\], `LPCDUseRC`\[0\] (AN602 §10.3.2, semantics largely
    /// undocumented). 0 = vendor typical. `LPCDEnRCcal` (0x08) measurably
    /// reduced the constant probe-vs-reference offset in field trials
    /// (2026-07-14), extending usable sensitivity.
    pub calib_flags: u8,

    /// If set, calibrate the wake reference from soft-power-down equilibrium
    /// ("double entry"): first enter LPCD with delta forced to max so the
    /// detector stays asleep while the chip settles for this duration, then
    /// knock it awake *without* an NPD/soft-reset cycle and immediately
    /// re-enter — the entry calibration then samples the settled state.
    ///
    /// Bench-measured (2026-07-14, cylinder-07): a single-entry reference is
    /// taken warm (right after NPD cycle + soft reset + I2C + carrier) and
    /// reads 10..15 ADC counts below the settled probe level — a one-sided
    /// systematic that eats nearly the whole delta range and storms the
    /// detector at any delta < 15. `None`: single-entry calibration
    /// (previous behavior).
    pub equilibrium_settle: Option<Duration>,
}

const FIFO_SIZE: usize = 64;

#[cfg(feature = "fm175xx")]
const ADC_REFERENCE_MIN: u8 = 0;
#[cfg(feature = "fm175xx")]
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

    fn regs(&mut self) -> Regs<'_, I> {
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
        Timer::after(Duration::from_millis(10)).await;

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

    #[cfg(feature = "fm175xx")]
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

    /// Put a WS1850S into LPCD (Low Power Card Detection) and block until a card
    /// is detected.
    ///
    /// Unlike the FM17550 path, the WS1850S configures LPCD through the
    /// `VersionReg`-unlocked Page4/Page6 banks (AN602 §7.1/§11) and calibrates
    /// its wake reference in hardware on entry (`CalibEn`), auto-ranging CWGsP
    /// near `config.cwgsp_lpcd` to hit `config.adc_ref`.
    /// A real card returns `Ok(())` and the caller re-enters on its next poll
    /// (which re-calibrates); a false wake self-corrects on re-enter the same
    /// way. We block on the IRQ line indefinitely.
    #[cfg(feature = "ws1850s")]
    pub async fn wait_for_card(&mut self, config: WakeupConfig) -> Result<(), Infallible> {
        assert!(config.delta <= 0x0F);
        assert!(config.swings_cnt <= 0x0F);
        assert!(config.skip <= 0x07);
        assert!(config.cwgsn_lpcd <= 0x0F);
        assert!(config.cwgsp_lpcd <= 0x3F);
        assert!(config.calib_flags <= 0x1F);

        // SoftReset + release NPD. Leaves the chip powered (NPD high); LPCD
        // runs in soft power-down, not hard power-down.
        self.on().await;

        // Carrier on (TxControlReg, 0x14). AN602 §7.1: 0x14 = 0x83.
        self.regs().txcontrol().write(|w| {
            w.set_tx1rfen(true);
            w.set_tx2rfen(true);
            w.set_invtx2on(true);
        });

        // With equilibrium calibration, the first entry is only there to
        // let the chip settle in soft power-down: force delta to max so
        // the (warm, biased-low) initial reference can't storm meanwhile.
        let first_delta = if config.equilibrium_settle.is_some() {
            0x0F
        } else {
            config.delta
        };

        // --- Page4 (unlock VersionReg = 0x5E) ---
        self.reg_write_raw(0x37, 0x5E);
        // LPCDReg (0x3C): CLK32K_En[5] | CalibEn[4] | Delta[3:0].
        self.reg_write_raw(0x3C, 0x20 | 0x10 | first_delta);
        // WUPeriodReg (0x3D): sleep period.
        self.reg_write_raw(0x3D, config.wu_period);
        // SwingsCntReg (0x3E): LPCD_en[7] | Skip[6:4] | SwingsCnt[3:0].
        self.reg_write_raw(0x3E, 0x80 | ((config.skip & 0x07) << 4) | (config.swings_cnt & 0x0F));
        // Re-lock.
        self.reg_write_raw(0x37, 0x00);

        // --- Page6 (unlock VersionReg = 0x5A) ---
        self.reg_write_raw(0x37, 0x5A);
        // P5_Reg38 (0x38): CWGsN_lpcd in the high nibble.
        self.reg_write_raw(0x38, (config.cwgsn_lpcd & 0x0F) << 4);
        // P5_Reg39 (0x39): CWGsP_lpcd in bits[5:0].
        self.reg_write_raw(0x39, config.cwgsp_lpcd & 0x3F);
        // P5_Reg31 (0x31): AN602 §10.3.1 documents it as the read-only LPCD
        // reference, but the vendor reference init (§7.1) writes 0xA1 to it —
        // follow the vendor code.
        self.reg_write_raw(0x31, 0xA1);
        // P5_Reg36 (0x36): ADC target of the entry calibration search. (Earlier
        // "0x36 is a no-op" bench findings were taken with the search range
        // confined to the saturated plateau, where no target is reachable.)
        self.reg_write_raw(0x36, config.adc_ref);
        // P4_Reg33 (0x33): CalibMode[5]=1 with CalibStep[7:6]=0 — search CWGsP
        // ±8 in steps of 1 around cwgsp_lpcd for the ADC closest to adc_ref.
        // Coarser steps quantize too hard on a steep antenna (bench-measured
        // ~20 ADC counts per CWGsP step), and direct-sample mode (CalibMode=0)
        // is only used by the `lpcd_sample_adc` diagnostic.
        self.reg_write_raw(0x33, 0x20 | (config.calib_flags & 0x1F));
        // Re-lock.
        self.reg_write_raw(0x37, 0x00);

        // IRQ: active-low, push-pull — matches the FM17xx path, the board's
        // `Pull::None` IRQ wiring and `wait_for_low()`. Do NOT use AN602 §7.1's
        // active-high/open-drain example: with `Pull::None` the line would float.
        self.regs().commien().write(|w| w.set_irqinv(true)); // ComIEnReg bit7
        self.regs().divien().write(|w| w.set_irqpushpull(true)); // DivIEnReg bit7
        // DivIEnReg (0x03) bit5 = TagDetIEn (WS1850S-specific, no typed field).
        let divien = self.reg_read_raw(0x03) | 0x20;
        self.reg_write_raw(0x03, divien);

        // The chip samples the ambient field and stores its LPCD reference in
        // hardware on entry (CalibEn). It can be read back for diagnostics at
        // P5_Reg31 (Page6, read-only) — see `read_lpcd_reference()`.

        // Enter LPCD: PCD soft power-down (CommandReg 0x01 = 0x10).
        self.regs().command().write(|w| w.set_powerdown(true));

        if let Some(settle) = config.equilibrium_settle {
            // Let the chip reach soft-power-down thermal equilibrium
            // (probing blind at delta 15), then wake it — one I2C
            // transaction, no NPD/soft-reset, so the settled state is
            // barely disturbed — and re-enter with the real delta. The
            // re-entry calibration samples the settled field.
            //
            // The chip NACKs its I2C address in soft power-down and the
            // NACKed transaction itself wakes it; the I2C interface retries
            // writes, absorbing that first NACK. Clearing PowerDown
            // (CommandReg = idle) finishes the exit once the write lands.
            Timer::after(settle).await;
            self.reg_write_raw(0x01, 0x00);

            // A detection during settle (delta 15 = a *large* field
            // change, i.e. a card slammed on) auto-woke the chip and
            // latched TagDetIrq: report it as a wake instead of silently
            // calibrating the card into the reference.
            let divirq = self.reg_read_raw(0x05);
            if divirq & 0x20 != 0 {
                debug!("ws1850s: LPCD wake during settle! divirq={:02x}", divirq);
                return Ok(());
            }

            self.reg_write_raw(0x37, 0x5E);
            self.reg_write_raw(0x3C, 0x20 | 0x10 | (config.delta & 0x0F));
            self.reg_write_raw(0x37, 0x00);
            // Clear any pending DivIrq bits (bit7=0 ⇒ clear marked bits).
            self.reg_write_raw(0x05, 0x7F);
            self.regs().command().write(|w| w.set_powerdown(true));
        }

        // Wake arrives as DivIrqReg (0x05) TagDetIrq (bit5 / & 0x20).
        // These per-entry logs are debug-level on purpose: at a sensitive
        // operating point a false-wake storm re-enters every ~2 s, which
        // would flood the device's in-RAM log buffer at info level.
        debug!("ws1850s: entering LPCD, waiting for irq...");
        loop {
            match self.irq.wait_for_low().await {
                Ok(()) => {
                    // LPCD diagnostics. TagDetIrq means the chip has auto-woken
                    // to Ready (AN602 §3.2), so I2C access is safe again.
                    // lpcd_ref is the hardware-calibrated reference; cwgsp is
                    // where the entry calibration parked the P-driver.
                    let divirq = self.reg_read_raw(0x05);
                    self.reg_write_raw(0x37, 0x5A);
                    let lpcd_ref = self.reg_read_raw(0x31);
                    let cwgsp = self.reg_read_raw(0x39);
                    self.reg_write_raw(0x37, 0x00);
                    debug!(
                        "ws1850s: got LPCD irq! divirq={:02x} lpcd_ref={:02x} cwgsp={:02x}",
                        divirq, lpcd_ref, cwgsp
                    );
                    return Ok(());
                }
                Err(_) => warn!("irq.wait_for_low() error"),
            }
        }
    }

    /// Raw main-page register write (addr < 0x40), used by the WS1850S LPCD path
    /// for the `VersionReg`-unlocked Page4/Page6 banks that have no typed
    /// accessors.
    #[cfg(feature = "ws1850s")]
    fn reg_write_raw(&mut self, addr: usize, val: u8) {
        self.iface.write_reg(addr, val);
    }

    /// Raw main-page register read (addr < 0x40). See [`Self::reg_write_raw`].
    #[cfg(feature = "ws1850s")]
    fn reg_read_raw(&mut self, addr: usize) -> u8 {
        self.iface.read_reg(addr)
    }

    /// Read the LPCD reference (`P5_Reg31`, Page6, read-only) that the WS1850S
    /// calibrated to on its last LPCD entry. Diagnostic only — call it after a
    /// wake, while the chip is still powered.
    #[cfg(feature = "ws1850s")]
    pub fn read_lpcd_reference(&mut self) -> u8 {
        self.reg_write_raw(0x37, 0x5A); // unlock Page6
        let r = self.reg_read_raw(0x31);
        self.reg_write_raw(0x37, 0x00); // re-lock
        r
    }

    /// Read `VersionReg` (0x37) to identify the chip.
    ///
    /// FM17xx returns e.g. `0x88`; WS1850S returns `0x12` or `0x15`. Used by the
    /// factory hwtest to assert the right silicon is on the board. Powers the
    /// chip on first (`new()` never touches it).
    pub async fn read_version(&mut self) -> u8 {
        self.on().await;
        self.regs().version().read()
    }

    #[cfg(feature = "fm175xx")]
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

    #[cfg(feature = "fm175xx")]
    fn lpcd_set_adc_config(&mut self, reference: u8, bias_current: u8) {
        self.regs().lpcd_adc_referece().write_value(reference & 0x3F);
        self.regs().lpcd_bias_current().write(|w| {
            w.set_adc_referece_h((reference >> 6) != 0);
            w.set_bias_current(bias_current);
        });
    }

    #[cfg(feature = "fm175xx")]
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

    #[cfg(feature = "fm175xx")]
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
#[cfg(feature = "fm175xx")]
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
