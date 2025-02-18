use embassy_futures::yield_now;
use embassy_time::{Duration, Instant};
use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

// use crate::aat::AatConfig;
use crate::commands::Command;
use crate::interface::Interface;
use crate::regs::{self, Regs};
use crate::{Error, Mode, St25r39};
pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(500);
use crate::impls::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(unused)]
pub enum Interrupt {
    /// RFU interrupt
    Rfu = 0,
    /// automatic reception restart interrupt
    RxRest = 1,
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
    /// 106kb/s Passive target state interrupt: Active
    WuA = 24,
    /// 106kb/s Passive target state interrupt: Active*
    WuAX = 25,
    /// RFU2 interrupt
    Rfu2 = 26,
    /// 212/424b/s Passive target interrupt: Active
    WuF = 27,
    /// RXE with an automatic response interrupt
    RxePta = 28,
    /// Anticollision done and Field On interrupt
    Apon = 29,
    /// Passive target slot number water level interrupt
    SlWl = 30,
    /// PPON2 Field on waiting Timer interrupt
    Ppon2 = 31,
}

impl<I: Interface, IrqPin: InputPin + Wait> St25r39<I, IrqPin> {
    pub async fn new(iface: I, irq: IrqPin) -> Result<Self, Error<I::Error>> {
        let mut this = Self {
            iface,
            irq,
            irqs: 0,
            mode: Mode::On,
        };
        this.init().await?;
        Ok(this)
    }

    pub fn regs(&mut self) -> Regs<I> {
        Regs::new(&mut self.iface)
    }

    pub fn cmd(&mut self, cmd: Command) -> Result<(), Error<I::Error>> {
        self.iface.do_command(cmd as u8).map_err(Error::Interface)
    }

    async fn cmd_wait(&mut self, cmd: Command) -> Result<(), Error<I::Error>> {
        self.irq_clear()?;
        self.cmd(cmd)?;
        self.irq_wait(Interrupt::Dct).await
    }

    async fn enable_osc(&mut self) -> Result<(), Error<I::Error>> {
        trace!("Starting osc...");
        self.regs().op_control().write(|w| w.set_en(true))?;
        while !self.regs().aux_display().read()?.osc_ok() {}
        Ok(())
    }

    async fn init(&mut self) -> Result<(), Error<I::Error>> {
        self.cmd(Command::SetDefault)?;

        self.regs().test_unk().write(|w| {
            w.set_dis_overheat_prot(true);
        })?;

        let id = self.regs().ic_identity().read()?;
        trace!("ic_type = {:02x} ic_rev = {:02x}", id.ic_type().0, id.ic_rev().0);

        // Enable OSC
        self.enable_osc().await?;

        // Measure vdd
        trace!("measuring vdd...");
        let vdd_mv = self.measure_vdd().await?;
        trace!("measure vdd result = {}mv", vdd_mv);

        let sup3v = vdd_mv < 3600;
        if sup3v {
            trace!("using 3v3 supply mode");
        } else {
            trace!("using 5v supply mode");
        }

        self.regs().io_conf2().write(|w| {
            w.set_sup_3v(sup3v);
        })?;

        // Disable MCU_CLK
        self.regs().io_conf1().write(|w| {
            w.set_out_cl(regs::IoConf1OutCl::DISABLED);
            w.set_lf_clk_off(true);
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

        self.regs().op_control().modify(|w| {
            w.set_en_fd(regs::OpControlEnFd::AUTO_EFD);
        })?;

        // Adjust regulators

        // Before sending the adjust regulator command it is required to toggle the bit reg_s by setting it first to 1 and then reset it to 0.
        self.regs().regulator_control().write(|w| w.set_reg_s(true))?;
        self.regs().regulator_control().write(|w| w.set_reg_s(false))?;

        self.cmd_wait(Command::AdjustRegulators).await?;

        let res = self.regs().regulator_result().read()?.0;
        trace!("reg result = {}", res);

        Ok(())
    }

    pub(crate) fn mode_off(&mut self) -> Result<(), Error<I::Error>> {
        self.mode = Mode::Off;
        self.cmd(Command::Stop)?;
        // disable everything
        self.regs().op_control().write(|_| {})?;
        Ok(())
    }

    pub async fn measure_amplitude(&mut self) -> Result<u8, Error<I::Error>> {
        self.cmd_wait(Command::MeasureAmplitude).await?;
        self.regs().ad_result().read()
    }

    pub async fn measure_phase(&mut self) -> Result<u8, Error<I::Error>> {
        self.cmd_wait(Command::MeasurePhase).await?;
        self.regs().ad_result().read()
    }

    pub async fn measure_capacitance(&mut self) -> Result<u8, Error<I::Error>> {
        self.cmd_wait(Command::MeasureCapacitance).await?;
        self.regs().ad_result().read()
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

            let res = self.regs().cap_sensor_result().read()?;
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

    pub(crate) async fn mode_on(&mut self) -> Result<(), Error<I::Error>> {
        self.mode = Mode::On;
        self.enable_osc().await?;

        self.regs().op_control().modify(|w| {
            w.set_en_fd(regs::OpControlEnFd::AUTO_EFD);
        })?;
        self.regs().tx_driver().write(|w| {
            w.set_d_res(3);
        })?;
        Ok(())
    }

    /// Change into wakeup mode, return immediately.
    /// The IRQ pin will go high on wakeup.
    pub async fn wait_for_card(&mut self, config: WakeupConfig) -> Result<(), Error<I::Error>> {
        self.mode_on().await?;

        self.mode = Mode::Wakeup;
        debug!("Entering wakeup mode");

        self.cmd(Command::Stop)?;
        self.regs().op_control().write(|_| {})?;
        self.regs().mode().write(|w| w.set_om(regs::ModeOm::INI_ISO14443A))?;

        let mut wtc = regs::WupTimerControl(0);
        let mut irqs = 0;

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
            irqs |= 1 << Interrupt::Wam as u32;
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
            irqs |= 1 << Interrupt::Wph as u32;
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
            wtc.set_wcap(true);
            irqs |= 1 << Interrupt::Wcap as u32;
        }

        self.irq_clear()?;

        self.regs().wup_timer_control().write_value(wtc)?;
        self.regs().op_control().write(|w| w.set_wu(true))?;
        self.irq_set_mask(!irqs)?;

        debug!("Entered wakeup mode, waiting for pin IRQ");
        self.irq.wait_for_high().await.unwrap();
        debug!("got pin IRQ!");

        Ok(())
    }

    pub async fn field_on(&mut self) -> Result<(), FieldOnError<I::Error>> {
        self.regs().mode().write(|w| {
            w.set_om(regs::ModeOm::INI_ISO14443A);
            w.set_tr_am(false); // use OOK
        })?;
        self.regs().tx_driver().write(|w| {
            w.set_am_mod(regs::TxDriverAmMod::_12PERCENT);
        })?;
        self.regs().aux_mod().write(|w| {
            w.set_lm_dri(true); // Enable internal Load Modulation
            w.set_dis_reg_am(false); // Enable regulator-based AM
            w.set_res_am(false);
        })?;

        // Default over/under shoot protiection
        self.regs().overshoot_conf1().write_value(0x40.into())?;
        self.regs().overshoot_conf2().write_value(0x03.into())?;
        self.regs().undershoot_conf1().write_value(0x40.into())?;
        self.regs().undershoot_conf2().write_value(0x03.into())?;

        self.regs().aux().write(|w| {
            w.set_dis_corr(false); // Enable correlator reception
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
        self.regs().field_on_gt().write_value(0)?;

        self.irq_clear()?; // clear
        self.cmd(Command::InitialRfCollision)?;

        loop {
            if self.irq(Interrupt::Cac) {
                return Err(FieldOnError::FieldCollision);
            }
            if self.irq(Interrupt::Apon) {
                break;
            }

            self.irq_update()?;
        }

        self.regs().op_control().modify(|w| {
            w.set_tx_en(true);
            w.set_rx_en(true);
        })?;

        Ok(())
    }

    async fn measure_vdd(&mut self) -> Result<u32, Error<I::Error>> {
        self.regs()
            .regulator_control()
            .write(|w| w.set_mpsv(regs::RegulatorControlMpsv::VDD))?;
        self.cmd_wait(Command::MeasureVdd).await?;
        let res = self.regs().ad_result().read()? as u32;

        // result is in units of 23.4mV
        Ok((res * 234 + 5) / 10)
    }

    // =======================
    //     irq stuff

    pub fn irq(&self, irq: Interrupt) -> bool {
        return (self.irqs & (1 << (irq as u8))) != 0;
    }

    pub async fn irq_wait_timeout(&mut self, irq: Interrupt, timeout: Duration) -> Result<(), Error<I::Error>> {
        let deadline = Instant::now() + timeout;
        self.irq_update()?;
        while !self.irq(irq) {
            if Instant::now() > deadline {
                return Err(Error::Timeout);
            }
            yield_now().await;
            self.irq_update()?;
        }
        Ok(())
    }

    pub async fn irq_wait(&mut self, irq: Interrupt) -> Result<(), Error<I::Error>> {
        self.irq_wait_timeout(irq, DEFAULT_TIMEOUT).await
    }

    pub fn irq_update(&mut self) -> Result<(), Error<I::Error>> {
        for i in 0..4 {
            self.irqs |= (self.regs().irq_main(i).read()? as u32) << (i * 8);
        }
        Ok(())
    }

    fn irq_clear(&mut self) -> Result<(), Error<I::Error>> {
        self.irq_update()?;
        self.irqs = 0;
        Ok(())
    }

    fn irq_set_mask(&mut self, mask: u32) -> Result<(), Error<I::Error>> {
        for i in 0..4 {
            self.regs().irq_mask(i).write_value((mask >> (i * 8)) as u8)?;
        }
        Ok(())
    }

    pub fn raw(&mut self) -> Raw<'_, I, IrqPin> {
        Raw { inner: self }
    }
}

pub struct Raw<'a, I: Interface, IrqPin: InputPin + Wait> {
    inner: &'a mut St25r39<I, IrqPin>,
}

impl<'a, I: Interface, IrqPin: InputPin + Wait> Raw<'a, I, IrqPin> {
    pub async fn field_on(&mut self) -> Result<(), FieldOnError<I::Error>> {
        self.inner.mode_on().await?;
        self.inner.field_on().await?;
        Ok(())
    }
    pub async fn field_off(&mut self) -> Result<(), Error<I::Error>> {
        self.inner.mode_off()?;
        Ok(())
    }

    pub async fn driver_hi_z(&mut self) -> Result<(), Error<I::Error>> {
        self.inner.mode_off()?;
        self.inner.regs().tx_driver().write(|w| {
            w.set_d_res(15); // hi-z
        })?;

        Ok(())
    }
}
