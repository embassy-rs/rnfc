#![allow(async_fn_in_trait)]
#![deny(unused_must_use)]

use embassy_futures::yield_now;
use embassy_time::{with_timeout, Duration, Instant, Timer};
use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

pub use crate::commands::Command;
pub use crate::interface::Interface;
use crate::regs::Regs;
use crate::{regs, Error, FieldOnError, WakeupConfig, WakeupReference, DEFAULT_TIMEOUT};

// TODO: This is here temporarily, just to provide abstraction similar to st25r3916.
// Thus - bits do not match.
// This chip offers 3 different registers for IRQs.
// Possibly whole IRQ handling would need to be done differently.
pub enum Interrupt {
    /// IRQ due to error and wake-up timer
    Err = 0,
    /// IRQ due to timer or NFC event
    Tim = 1,
    /// bit collision interrupt
    Col = 2,
    /// end of transmission interrupt
    Txe = 3,
    /// end of receive interrupt
    Rxe = 4,
    /// start of receive interrupt
    Rxs = 5,
    /// FIFO water level interrupt
    Fwl = 6,
    /// oscillator stable interrupt
    Osc = 7,
    /// initiator bit rate recognised interrupt
    Nfct = 8,
    /// minimum guard time expired interrupt
    Cat = 9,
    /// collision during RF collision avoidance interrupt
    Cac = 10,
    /// external field off interrupt
    Eof = 11,
    /// external field on interrupt
    Eon = 12,
    /// general purpose timer expired interrupt
    Gpe = 13,
    /// no-response timer expired interrupt
    Nre = 14,
    /// termination of direct command interrupt
    Dct = 15,
    /// wake-up due to capacitance measurement
    Wcap = 16,
    /// wake-up due to phase interrupt
    Wph = 17,
    /// wake-up due to amplitude interrupt
    Wam = 18,
    /// wake-up interrupt
    Wt = 19,
    /// hard framing error interrupt
    Err1 = 20,
    /// soft framing error interrupt
    Err2 = 21,
    /// parity error interrupt
    Par = 22,
    /// CRC error interrupt
    Crc = 23,
    /// RXE with an automatic response interrupt
    RxePta = 28,
    /// Anticollision done and Field On interrupt
    Apon = 29,
}

/// Device starts with default configuration. Initially the oscillator is not enabled.
/// It's possible to verify register contents; then call `configure` to enable RF.
impl<I: Interface, IrqPin: InputPin + Wait> super::St25r39<I, IrqPin> {
    pub async fn new(iface: I, irq: IrqPin) -> Result<Self, Error<I::Error>> {
        let mut this = Self {
            iface,
            irq,
            irqs: 0,
            mode: super::Mode::Off,
        };
        this.init().await?;
        //this.configure();
        // preferably called by user,
        // after checking IC_rev, ect
        // otherwise we might spin forever waiting for OSC
        Ok(this)
    }

    pub fn regs(&mut self) -> Regs<I> {
        Regs::<I>::new(&mut self.iface)
    }

    pub fn cmd(&mut self, cmd: Command) -> Result<(), Error<I::Error>> {
        self.iface.do_command(cmd as u8).map_err(Error::Interface)
    }

    async fn cmd_wait(&mut self, cmd: Command) -> Result<(), Error<I::Error>> {
        self.irq_clear()?;
        self.cmd(cmd)?;
        match self.regs().irq_timer_nfc().read()?.dcd() {
            true => Ok(()),
            false => Err(Error::Timeout),
        }
    }

    async fn is_osc_stable(&mut self) -> Result<(), Error<I::Error>> {
        let failed = Err(Error::Timeout);
        match with_timeout(
            DEFAULT_TIMEOUT / 10, // table 101, Tosc max 10ms, we'll wait 50ms
            async { self.regs().aux_display().read() },
        )
        .await
        {
            Ok(val) => {
                if val?.osc_ok() {
                    Ok(())
                } else {
                    failed
                }
            }
            Err(_) => failed,
        }
    }

    pub async fn enable_osc(&mut self) -> Result<(), Error<I::Error>> {
        trace!("Starting osc...");
        if self.is_osc_stable().await.is_err() {
            self.regs().op_control().write(|w| w.set_en(true))?;
        }
        self.is_osc_stable().await
    }

    async fn init(&mut self) -> Result<(), Error<I::Error>> {
        self.cmd(Command::SetDefault)?;
        self.cmd(Command::Clear)?;

        let id = self.regs().ic_identity().read()?;
        trace!("ic_type = {:02x} ic_rev = {:02x}", id.ic_type().0, id.ic_rev().0);

        Ok(())
    }

    pub async fn mcu_clk_off(&mut self) -> Result<(), Error<I::Error>> {
        // Disable MCU_CLK (default, on power-up is 3.39MHz)
        self.regs().io_conf1().write(|w| {
            w.set_out_cl(regs::IoConf1OutCl::DISABLED);
            w.set_osc(true); // use 27.12Mhz Xtal
        })?;

        match self.regs().io_conf1().read()?.out_cl() == regs::IoConf1OutCl::DISABLED {
            true => Ok(()),
            false => Err(Error::Timeout),
        }
    }

    pub async fn configure(&mut self) -> Result<(), Error<I::Error>> {
        self.irq_clear()?;
        // Enable OSC
        self.enable_osc().await?;
        self.irq_clear()?; // mask osc first

        // Measure vdd
        trace!("measuring vdd...");
        let vdd_mv = self.measure_vdd().await?;
        trace!("measure vdd result = {}mv", vdd_mv);

        let sup3v = vdd_mv < 3600;
        if sup3v {
            self.regs().io_conf2().write(|w| {
                w.set_sup_3v(sup3v);
            })?;
            trace!("using 3v3 supply mode");
        } else {
            trace!("using 5v supply mode");
        }

        // Disable MCU_CLK, default on power-up 3.39MHz
        self.regs().io_conf1().write(|w| {
            w.set_out_cl(regs::IoConf1OutCl::DISABLED);
            w.set_osc(true); // use 27.12Mhz Xtal
        })?;

        // Enable minimum non-overlap
        //self.regs().res_am_mod().write(|w| w.set_fa3_f(true))?;

        // Set ext field detect activ/deactiv thresholds
        //self.regs().field_threshold_actv().write(|w| {
        //    w.set_trg(regs::FieldThresholdActvTrg::_105MV);
        //    w.set_rfe(regs::FieldThresholdActvRfe::_105MV);
        //})?;
        //self.regs().field_threshold_deactv().write(|w| {
        //    w.set_trg(regs::FieldThresholdDeactvTrg::_75MV);
        //    w.set_rfe(regs::FieldThresholdDeactvRfe::_75MV);
        //})?;

        //self.regs().aux_mod().write(|w| {
        //    w.set_lm_ext(false); // Disable external Load Modulation
        //    w.set_lm_dri(true); // Enable internal Load Modulation
        //})?;

        //self.regs().emd_sup_conf().write(|w| {
        //    w.set_rx_start_emv(true);
        //})?;

        // AAT not in use
        //self.regs().ant_tune_a().write_value(0x82)?;
        //self.regs().ant_tune_b().write_value(0x82)?;

        // Adjust regulators

        // Before sending the adjust regulator command it is required to toggle the bit reg_s by setting it first to 1 and then reset it to 0.
        self.regs().regulator_volt_control().write(|w| w.set_reg_s(true))?;
        self.regs().regulator_volt_control().write(|w| w.set_reg_s(false))?;

        self.cmd_wait(Command::AdjustRegulators).await?;

        let res = self.regs().regulator_and_tim_disp().read()?.0;
        trace!("reg result = {}", res);

        Ok(())
    }

    pub async fn mode_on(&mut self) -> Result<(), Error<I::Error>> {
        self.mode = super::Mode::On;
        self.enable_osc().await?;

        Ok(())
    }

    pub fn mode_off(&mut self) -> Result<(), Error<I::Error>> {
        self.mode = super::Mode::Off;
        self.cmd(Command::Stop)?;
        // disable everything
        self.regs().op_control().write(|_| {})?;
        Ok(())
    }

    pub async fn measure_amplitude(&mut self) -> Result<u8, Error<I::Error>> {
        self.cmd_wait(Command::MeasureAmplitude).await?;
        self.regs().ad_conv_result().read()
    }

    pub async fn measure_phase(&mut self) -> Result<u8, Error<I::Error>> {
        self.cmd_wait(Command::MeasurePhase).await?;
        self.regs().ad_conv_result().read()
    }

    pub async fn measure_capacitance(&mut self) -> Result<u8, Error<I::Error>> {
        self.cmd_wait(Command::MeasureCapacitance).await?;
        self.regs().ad_conv_result().read()
    }

    pub async fn calibrate_capacitance(&mut self) -> Result<u8, Error<I::Error>> {
        self.regs().cap_sensor_control().write(|w| {
            // Clear Manual calibration values to enable automatic calibration mode
            w.set_cs_mcal(0);
            w.set_cs_g(0b01); // 6.5v/pF, highest one
        })?;

        // Don't use `cmd_wait`, the irq only fires in Ready mode (op_control.en = 1).
        // Instead, wait for cap_sensor_result.cs_cal_end
        self.cmd(Command::CalibrateCSensor)?;

        let deadline = Instant::now() + DEFAULT_TIMEOUT;

        let res = loop {
            if Instant::now() > deadline {
                return Err(Error::Timeout);
            }

            let res = self.regs().cap_sensor_disp().read()?;
            if res.cs_cal_err() {
                panic!("Capacitive sensor calibration failed!");
            }
            if res.cs_cal_end() {
                break res;
            }

            yield_now().await;
        };
        Ok(res.cs_cal_val())
    }

    /// Change into wakeup mode, return immediately.
    /// The IRQ pin will go high on wakeup.
    pub async fn wait_for_card(&mut self, config: WakeupConfig) -> Result<(), Error<I::Error>> {
        self.mode_on().await?;

        self.mode = super::Mode::Wakeup;
        debug!("Entering wakeup mode");

        self.cmd(Command::Stop)?;
        self.regs().op_control().write(|_| {})?;
        self.regs().mode().write(|w| w.set_om(regs::ModeOm::INI_ISO14443A))?;

        let mut wtc = regs::WupTimerControl(0);
        // let mut irqs = 0;

        wtc.set_wur(config.period as u8 & 0x10 == 0);
        wtc.set_wut(config.period as u8 & 0x0F);

        if let Some(m) = config.inductive_amplitude {
            let mut conf = regs::AmplitudeMeasureConf(0);
            conf.set_am_d(m.delta);
            match m.reference {
                WakeupReference::Manual(val) => {
                    self.regs().amplitude_measure_ref().write_value(val)?;
                }
                WakeupReference::Automatic => {
                    let val = self.measure_amplitude().await?;
                    self.regs().amplitude_measure_ref().write_value(val)?;
                }
                WakeupReference::AutoAverage {
                    include_irq_measurement,
                    weight,
                } => {
                    let val = self.measure_amplitude().await?;
                    self.regs().amplitude_measure_ref().write_value(val)?;
                    conf.set_am_ae(true);
                    conf.set_am_aam(include_irq_measurement);
                    conf.set_am_aew(weight);
                }
            }
            self.regs().amplitude_measure_conf().write_value(conf)?;
            wtc.set_wam(true);
            //irqs |= 1 << Interrupt::Wam as u32;
        }
        if let Some(m) = config.inductive_phase {
            let mut conf = regs::PhaseMeasureConf(0);
            conf.set_pm_d(m.delta);
            match m.reference {
                WakeupReference::Manual(val) => {
                    self.regs().phase_measure_ref().write_value(val)?;
                }
                WakeupReference::Automatic => {
                    let val = self.measure_phase().await?;
                    self.regs().phase_measure_ref().write_value(val)?;
                }
                WakeupReference::AutoAverage {
                    include_irq_measurement,
                    weight,
                } => {
                    let val = self.measure_phase().await?;
                    self.regs().phase_measure_ref().write_value(val)?;
                    conf.set_pm_ae(true);
                    conf.set_pm_aam(include_irq_measurement);
                    conf.set_pm_aew(weight);
                }
            }
            self.regs().phase_measure_conf().write_value(conf)?;
            wtc.set_wph(true);
            // irqs |= 1 << Interrupt::Wph as u32;
        }
        if let Some(m) = config.capacitive {
            debug!("capacitance calibrating...");
            let val = self.calibrate_capacitance().await?;
            info!("capacitance calibrated: {}", val);

            let mut conf = regs::CapacitanceMeasureConf(0);
            conf.set_cm_d(m.delta);
            match m.reference {
                WakeupReference::Manual(val) => {
                    self.regs().capacitance_measure_ref().write_value(val)?;
                }
                WakeupReference::Automatic => {
                    let val = self.measure_capacitance().await?;
                    info!("Measured: {}", val);
                    self.regs().capacitance_measure_ref().write_value(val)?;
                }
                WakeupReference::AutoAverage {
                    include_irq_measurement,
                    weight,
                } => {
                    let val = self.measure_capacitance().await?;
                    info!("Measured: {}", val);
                    self.regs().capacitance_measure_ref().write_value(val)?;
                    conf.set_cm_ae(true);
                    conf.set_cm_aam(include_irq_measurement);
                    conf.set_cm_aew(weight);
                }
            }
            self.regs().capacitance_measure_conf().write_value(conf)?;
            // wtc.set_wcap(true); // wto?
            // irqs |= 1 << Interrupt::Wcap as u32;
        }

        self.regs().wup_timer_control().write_value(wtc)?;
        self.regs().op_control().write(|w| w.set_wu(true))?;
        // self.irq_set_mask(!irqs)?;

        debug!("Entered wakeup mode, waiting for pin IRQ");
        self.irq.wait_for_high().await.unwrap();
        debug!("got pin IRQ!");

        Ok(())
    }

    pub async fn field_on(&mut self) -> Result<(), FieldOnError<I::Error>> {
        self.regs().mode().write(|w| {
            w.set_om(regs::ModeOm::INI_ISO14443A);
        })?;
        // note: set automatically if command AnalogPreset is used
        self.regs().aux().write(|w| {
            w.set_tr_am(false); // use OOK
        })?;

        // self.regs().aux_mod().write(|w| {
        //     w.set_lm_dri(true); // Enable internal Load Modulation
        //     w.set_dis_reg_am(false); // Enable regulator-based AM
        //     w.set_res_am(false);
        // })?;

        // // Default over/under shoot protiection
        // self.regs().overshoot_conf1().write_value(0x40.into())?;
        // self.regs().overshoot_conf2().write_value(0x03.into())?;
        // self.regs().undershoot_conf1().write_value(0x40.into())?;
        // self.regs().undershoot_conf2().write_value(0x03.into())?;

        self.regs().aux().write(|w| {
            // w.set_dis_corr(false); // Enable correlator reception
            w.set_nfc_n(0); // todo this changes
        })?;
        /*
        self.regs().rx_conf1().write_value(0x08.into())?;
        self.regs().rx_conf2().write_value(0x2D.into())?;
        self.regs().rx_conf3().write_value(0x00.into())?;
        self.regs().rx_conf4().write_value(0x00.into())?;
        self.regs().corr_conf1().write_value(0x51.into())?;
        self.regs().corr_conf2().write_value(0x00.into())?;
         */

        self.regs().bit_rate().write(|w| {
            w.set_rxrate(regs::BitRateE::_106);
            w.set_txrate(regs::BitRateE::_106);
        })?;

        // defaults
        self.regs().iso14443a_nfc().write(|_| {})?;

        // Field ON

        // GT is done by software
        // self.regs().field_on_gt().write_value(0)?;

        self.irq_clear()?; // clear
        self.cmd(Command::InitialRfCollision)?;

        // loop {
        //     if self.irq(Interrupt::Cac) {
        //         return Err(FieldOnError::FieldCollision);
        //     }
        //     if self.irq(Interrupt::Apon) {
        //         break;
        //     }

        //     self.irq_update()?;
        // }

        self.regs().op_control().modify(|w| {
            w.set_tx_en(true);
            w.set_rx_en(true);
        })?;

        Ok(())
    }

    async fn measure_vdd(&mut self) -> Result<u32, Error<I::Error>> {
        self.regs().regulator_volt_control().write(|w| w.set_mpsv(0))?;
        self.cmd_wait(Command::MeasureVdd).await?;
        let res = self.regs().ad_conv_result().read()? as u32;

        // result is in units of 23.4mV
        Ok((res * 234 + 5) / 10)
    }

    // =======================
    //     irq stuff

    // This chip offers 3 different registers to check for different IRQs
    // Either an abstraction between chips would need to be build, or
    #[inline]
    pub fn irq(&mut self, irq: Interrupt) -> bool {
        let r = self.regs().irq_main().read().expect("be readable");
        match irq {
            Interrupt::Err => r.err(),
            Interrupt::Tim => r.tim(),
            Interrupt::Col => r.col(),
            Interrupt::Txe => r.txe(),
            Interrupt::Rxe => r.rxe(),
            _ => todo!(), // TODO: ect huge match which doesn't make much sense?
        }
    }

    pub async fn irq_wait_timeout<F>(&self, mut fn_to_call: F, timeout: Duration) -> Result<(), Error<I::Error>>
    where
        F: FnMut() -> bool,
    {
        match with_timeout(
            timeout,
            async {
                while !fn_to_call() {
                    Timer::after_millis(5).await
                }
                embassy_futures::yield_now()
            }
            .await,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Timeout),
        }
    }

    pub async fn irq_wait<F>(&self, fn_to_call: F) -> Result<(), Error<I::Error>>
    where
        F: Fn() -> bool,
    {
        self.irq_wait_timeout(fn_to_call, DEFAULT_TIMEOUT).await
    }

    pub fn irq_update(&mut self) -> Result<(), Error<I::Error>> {
        self.regs().irq_main().read()?;
        Ok(())
    }

    #[inline]
    fn irq_clear(&mut self) -> Result<(), Error<I::Error>> {
        self.regs().irq_main().write(|v| v.0 = 0)?;
        Ok(())
    }

    // pub fn raw(&mut self) -> Raw<'_, I, IrqPin> {
    //     Raw { inner: self }
    // }
}
