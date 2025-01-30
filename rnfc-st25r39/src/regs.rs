#![allow(unused)]

use core::marker::PhantomData;

use super::Interface;
use crate::Error;

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

// ==========================================================
// ==========================================================
// ==========================================================

pub struct Regs<'a, I: Interface> {
    iface: &'a mut I,
}

impl<'a, I: Interface> Regs<'a, I> {
    pub fn new(iface: &'a mut I) -> Self {
        Self { iface }
    }

    pub fn io_conf1(&mut self) -> Reg<'_, I, IoConf1> {
        Reg::new(self.iface, 0)
    }
    pub fn io_conf2(&mut self) -> Reg<'_, I, IoConf2> {
        Reg::new(self.iface, 1)
    }
    pub fn op_control(&mut self) -> Reg<'_, I, OpControl> {
        Reg::new(self.iface, 2)
    }
    pub fn mode(&mut self) -> Reg<'_, I, Mode> {
        Reg::new(self.iface, 3)
    }
    pub fn bit_rate(&mut self) -> Reg<'_, I, BitRate> {
        Reg::new(self.iface, 4)
    }
    pub fn iso14443a_nfc(&mut self) -> Reg<'_, I, Iso14443aNfc> {
        Reg::new(self.iface, 5)
    }
    pub fn iso14443b_1(&mut self) -> Reg<'_, I, Iso14443b1> {
        Reg::new(self.iface, 6)
    }
    pub fn iso14443b_2(&mut self) -> Reg<'_, I, Iso14443b2> {
        Reg::new(self.iface, 7)
    }
    pub fn passive_target(&mut self) -> Reg<'_, I, PassiveTarget> {
        Reg::new(self.iface, 8)
    }
    pub fn stream_mode(&mut self) -> Reg<'_, I, StreamMode> {
        Reg::new(self.iface, 9)
    }
    pub fn aux(&mut self) -> Reg<'_, I, Aux> {
        Reg::new(self.iface, 10)
    }
    pub fn rx_conf1(&mut self) -> Reg<'_, I, RxConf1> {
        Reg::new(self.iface, 11)
    }
    pub fn rx_conf2(&mut self) -> Reg<'_, I, RxConf2> {
        Reg::new(self.iface, 12)
    }
    pub fn rx_conf3(&mut self) -> Reg<'_, I, RxConf3> {
        Reg::new(self.iface, 13)
    }
    pub fn rx_conf4(&mut self) -> Reg<'_, I, RxConf4> {
        Reg::new(self.iface, 14)
    }
    pub fn mask_rx_timer(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 15)
    }
    pub fn no_response_timer1(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 16)
    }
    pub fn no_response_timer2(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 17)
    }
    pub fn timer_emv_control(&mut self) -> Reg<'_, I, TimerEmvControl> {
        Reg::new(self.iface, 18)
    }
    pub fn gpt1(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 19)
    }
    pub fn gpt2(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 20)
    }
    pub fn ppon2(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 21)
    }
    pub fn irq_mask(&mut self, n: u8) -> Reg<'_, I, u8> {
        assert!(n < 4);
        Reg::new(self.iface, 22 + n)
    }
    pub fn irq_main(&mut self, n: u8) -> Reg<'_, I, u8> {
        assert!(n < 4);
        Reg::new(self.iface, 26 + n)
    }
    pub fn fifo_status1(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 30)
    }
    pub fn fifo_status2(&mut self) -> Reg<'_, I, FifoStatus2> {
        Reg::new(self.iface, 31)
    }
    pub fn collision_status(&mut self) -> Reg<'_, I, CollisionStatus> {
        Reg::new(self.iface, 32)
    }
    pub fn passive_target_status(&mut self) -> Reg<'_, I, PassiveTargetStatus> {
        Reg::new(self.iface, 33)
    }
    pub fn num_tx_bytes1(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 34)
    }
    pub fn num_tx_bytes2(&mut self) -> Reg<'_, I, NumTxBytes2> {
        Reg::new(self.iface, 35)
    }
    pub fn nfcip1_bit_rate(&mut self) -> Reg<'_, I, Nfcip1BitRate> {
        Reg::new(self.iface, 36)
    }
    pub fn ad_result(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 37)
    }
    pub fn ant_tune_a(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 38)
    }
    pub fn ant_tune_b(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 39)
    }
    pub fn tx_driver(&mut self) -> Reg<'_, I, TxDriver> {
        Reg::new(self.iface, 40)
    }
    pub fn pt_mod(&mut self) -> Reg<'_, I, PtMod> {
        Reg::new(self.iface, 41)
    }
    pub fn field_threshold_actv(&mut self) -> Reg<'_, I, FieldThresholdActv> {
        Reg::new(self.iface, 42)
    }
    pub fn field_threshold_deactv(&mut self) -> Reg<'_, I, FieldThresholdDeactv> {
        Reg::new(self.iface, 43)
    }
    pub fn regulator_control(&mut self) -> Reg<'_, I, RegulatorControl> {
        Reg::new(self.iface, 44)
    }
    pub fn rssi_result(&mut self) -> Reg<'_, I, RssiResult> {
        Reg::new(self.iface, 45)
    }
    pub fn gain_red_state(&mut self) -> Reg<'_, I, GainRedState> {
        Reg::new(self.iface, 46)
    }
    pub fn cap_sensor_control(&mut self) -> Reg<'_, I, CapSensorControl> {
        Reg::new(self.iface, 47)
    }
    pub fn cap_sensor_result(&mut self) -> Reg<'_, I, CapSensorResult> {
        Reg::new(self.iface, 48)
    }
    pub fn aux_display(&mut self) -> Reg<'_, I, AuxDisplay> {
        Reg::new(self.iface, 49)
    }
    pub fn wup_timer_control(&mut self) -> Reg<'_, I, WupTimerControl> {
        Reg::new(self.iface, 50)
    }
    pub fn amplitude_measure_conf(&mut self) -> Reg<'_, I, AmplitudeMeasureConf> {
        Reg::new(self.iface, 51)
    }
    pub fn amplitude_measure_ref(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 52)
    }
    pub fn amplitude_measure_aa_result(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 53)
    }
    pub fn amplitude_measure_result(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 54)
    }
    pub fn phase_measure_conf(&mut self) -> Reg<'_, I, PhaseMeasureConf> {
        Reg::new(self.iface, 55)
    }
    pub fn phase_measure_ref(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 56)
    }
    pub fn phase_measure_aa_result(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 57)
    }
    pub fn phase_measure_result(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 58)
    }
    pub fn capacitance_measure_conf(&mut self) -> Reg<'_, I, CapacitanceMeasureConf> {
        Reg::new(self.iface, 59)
    }
    pub fn capacitance_measure_ref(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 60)
    }
    pub fn capacitance_measure_aa_result(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 61)
    }
    pub fn capacitance_measure_result(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 62)
    }
    pub fn ic_identity(&mut self) -> Reg<'_, I, IcIdentity> {
        Reg::new(self.iface, 63)
    }
    pub fn emd_sup_conf(&mut self) -> Reg<'_, I, EmdSupConf> {
        Reg::new(self.iface, 69)
    }
    pub fn subc_start_time(&mut self) -> Reg<'_, I, SubcStartTime> {
        Reg::new(self.iface, 70)
    }
    pub fn p2p_rx_conf(&mut self) -> Reg<'_, I, P2pRxConf> {
        Reg::new(self.iface, 75)
    }
    pub fn corr_conf1(&mut self) -> Reg<'_, I, CorrConf1> {
        Reg::new(self.iface, 76)
    }
    pub fn corr_conf2(&mut self) -> Reg<'_, I, CorrConf2> {
        Reg::new(self.iface, 77)
    }
    pub fn squelch_timer(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 79)
    }
    pub fn field_on_gt(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 85)
    }
    pub fn aux_mod(&mut self) -> Reg<'_, I, AuxMod> {
        Reg::new(self.iface, 104)
    }
    pub fn tx_driver_timing(&mut self) -> Reg<'_, I, TxDriverTiming> {
        Reg::new(self.iface, 105)
    }
    pub fn res_am_mod(&mut self) -> Reg<'_, I, ResAmMod> {
        Reg::new(self.iface, 106)
    }
    pub fn tx_driver_status(&mut self) -> Reg<'_, I, TxDriverStatus> {
        Reg::new(self.iface, 107)
    }
    pub fn regulator_result(&mut self) -> Reg<'_, I, RegulatorResult> {
        Reg::new(self.iface, 108)
    }
    pub fn overshoot_conf1(&mut self) -> Reg<'_, I, OvershootConf1> {
        Reg::new(self.iface, 112)
    }
    pub fn overshoot_conf2(&mut self) -> Reg<'_, I, OvershootConf2> {
        Reg::new(self.iface, 113)
    }
    pub fn undershoot_conf1(&mut self) -> Reg<'_, I, UndershootConf1> {
        Reg::new(self.iface, 114)
    }
    pub fn undershoot_conf2(&mut self) -> Reg<'_, I, UndershootConf2> {
        Reg::new(self.iface, 115)
    }
    pub fn test_unk(&mut self) -> Reg<'_, I, TestUnk> {
        Reg::new(self.iface, 132)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TxDriver(pub u8);
impl TxDriver {
    pub const fn d_res(&self) -> TxDriverDRes {
        let val = (self.0 >> 0usize) & 0x0f;
        TxDriverDRes(val as u8)
    }
    pub fn set_d_res(&mut self, val: TxDriverDRes) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val.0 as u8) & 0x0f) << 0usize);
    }
    pub const fn am_mod(&self) -> TxDriverAmMod {
        let val = (self.0 >> 4usize) & 0x0f;
        TxDriverAmMod(val as u8)
    }
    pub fn set_am_mod(&mut self, val: TxDriverAmMod) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val.0 as u8) & 0x0f) << 4usize);
    }
}
impl Default for TxDriver {
    fn default() -> TxDriver {
        TxDriver(0)
    }
}
impl From<u8> for TxDriver {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<TxDriver> for u8 {
    fn from(val: TxDriver) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AuxMod(pub u8);
impl AuxMod {
    pub const fn rfu0(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_rfu0(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn rfu1(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_rfu1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn rfu2(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_rfu2(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn res_am(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_res_am(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn lm_dri(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_lm_dri(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn lm_ext(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_lm_ext(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn lm_ext_pol(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_lm_ext_pol(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn dis_reg_am(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_dis_reg_am(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for AuxMod {
    fn default() -> AuxMod {
        AuxMod(0)
    }
}
impl From<u8> for AuxMod {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<AuxMod> for u8 {
    fn from(val: AuxMod) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RxConf2(pub u8);
impl RxConf2 {
    pub const fn agc6_3(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_agc6_3(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn agc_alg(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_agc_alg(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn agc_m(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_agc_m(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn agc_en(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_agc_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn pulz_61(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_pulz_61(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn sqm_dyn(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_sqm_dyn(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn amd_sel(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_amd_sel(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn amd_sel_mixer(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_amd_sel_mixer(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn demod_mode(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_demod_mode(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for RxConf2 {
    fn default() -> RxConf2 {
        RxConf2(0)
    }
}
impl From<u8> for RxConf2 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<RxConf2> for u8 {
    fn from(val: RxConf2) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SubcStartTime(pub u8);
impl SubcStartTime {
    pub const fn sst(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x1f;
        val as u8
    }
    pub fn set_sst(&mut self, val: u8) {
        self.0 = (self.0 & !(0x1f << 0usize)) | (((val as u8) & 0x1f) << 0usize);
    }
    pub const fn rfu0(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_rfu0(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn rfu1(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_rfu1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn rfu2(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_rfu2(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for SubcStartTime {
    fn default() -> SubcStartTime {
        SubcStartTime(0)
    }
}
impl From<u8> for SubcStartTime {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<SubcStartTime> for u8 {
    fn from(val: SubcStartTime) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RegulatorControl(pub u8);
impl RegulatorControl {
    pub const fn mpsv(&self) -> RegulatorControlMpsv {
        let val = (self.0 >> 0usize) & 0x07;
        RegulatorControlMpsv(val as u8)
    }
    pub fn set_mpsv(&mut self, val: RegulatorControlMpsv) {
        self.0 = (self.0 & !(0x07 << 0usize)) | (((val.0 as u8) & 0x07) << 0usize);
    }
    pub const fn rege(&self) -> u8 {
        let val = (self.0 >> 3usize) & 0x0f;
        val as u8
    }
    pub fn set_rege(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 3usize)) | (((val as u8) & 0x0f) << 3usize);
    }
    pub const fn reg_s(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_reg_s(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for RegulatorControl {
    fn default() -> RegulatorControl {
        RegulatorControl(0)
    }
}
impl From<u8> for RegulatorControl {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<RegulatorControl> for u8 {
    fn from(val: RegulatorControl) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct EmdSupConf(pub u8);
impl EmdSupConf {
    pub const fn emd_thld(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x0f;
        val as u8
    }
    pub fn set_emd_thld(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val as u8) & 0x0f) << 0usize);
    }
    pub const fn rfu0(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_rfu0(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn rfu1(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_rfu1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn rx_start_emv(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_rx_start_emv(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn emd_emv(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_emd_emv(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for EmdSupConf {
    fn default() -> EmdSupConf {
        EmdSupConf(0)
    }
}
impl From<u8> for EmdSupConf {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<EmdSupConf> for u8 {
    fn from(val: EmdSupConf) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RssiResult(pub u8);
impl RssiResult {
    pub const fn rssi_pm(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x0f;
        val as u8
    }
    pub fn set_rssi_pm(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val as u8) & 0x0f) << 0usize);
    }
    pub const fn rssi_am(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    pub fn set_rssi_am(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for RssiResult {
    fn default() -> RssiResult {
        RssiResult(0)
    }
}
impl From<u8> for RssiResult {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<RssiResult> for u8 {
    fn from(val: RssiResult) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CollisionStatus(pub u8);
impl CollisionStatus {
    pub const fn c_pb(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_c_pb(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn c_bit(&self) -> u8 {
        let val = (self.0 >> 1usize) & 0x07;
        val as u8
    }
    pub fn set_c_bit(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 1usize)) | (((val as u8) & 0x07) << 1usize);
    }
    pub const fn c_byte(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    pub fn set_c_byte(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for CollisionStatus {
    fn default() -> CollisionStatus {
        CollisionStatus(0)
    }
}
impl From<u8> for CollisionStatus {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<CollisionStatus> for u8 {
    fn from(val: CollisionStatus) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct StreamMode(pub u8);
impl StreamMode {
    pub const fn stx(&self) -> StreamModeStx {
        let val = (self.0 >> 0usize) & 0x07;
        StreamModeStx(val as u8)
    }
    pub fn set_stx(&mut self, val: StreamModeStx) {
        self.0 = (self.0 & !(0x07 << 0usize)) | (((val.0 as u8) & 0x07) << 0usize);
    }
    pub const fn scp(&self) -> StreamModeScp {
        let val = (self.0 >> 3usize) & 0x03;
        StreamModeScp(val as u8)
    }
    pub fn set_scp(&mut self, val: StreamModeScp) {
        self.0 = (self.0 & !(0x03 << 3usize)) | (((val.0 as u8) & 0x03) << 3usize);
    }
    pub const fn scf(&self) -> StreamModeScf {
        let val = (self.0 >> 5usize) & 0x03;
        StreamModeScf(val as u8)
    }
    pub fn set_scf(&mut self, val: StreamModeScf) {
        self.0 = (self.0 & !(0x03 << 5usize)) | (((val.0 as u8) & 0x03) << 5usize);
    }
    pub const fn rfu(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_rfu(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for StreamMode {
    fn default() -> StreamMode {
        StreamMode(0)
    }
}
impl From<u8> for StreamMode {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<StreamMode> for u8 {
    fn from(val: StreamMode) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct BitRate(pub u8);
impl BitRate {
    pub const fn rxrate(&self) -> BitRateE {
        let val = (self.0 >> 0usize) & 0x03;
        BitRateE(val as u8)
    }
    pub fn set_rxrate(&mut self, val: BitRateE) {
        self.0 = (self.0 & !(0x03 << 0usize)) | (((val.0 as u8) & 0x03) << 0usize);
    }
    pub const fn txrate(&self) -> BitRateE {
        let val = (self.0 >> 4usize) & 0x03;
        BitRateE(val as u8)
    }
    pub fn set_txrate(&mut self, val: BitRateE) {
        self.0 = (self.0 & !(0x03 << 4usize)) | (((val.0 as u8) & 0x03) << 4usize);
    }
}
impl Default for BitRate {
    fn default() -> BitRate {
        BitRate(0)
    }
}
impl From<u8> for BitRate {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<BitRate> for u8 {
    fn from(val: BitRate) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RxConf3(pub u8);
impl RxConf3 {
    pub const fn lf_op(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_lf_op(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn lf_en(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_lf_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn rg1_pm(&self) -> u8 {
        let val = (self.0 >> 2usize) & 0x07;
        val as u8
    }
    pub fn set_rg1_pm(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 2usize)) | (((val as u8) & 0x07) << 2usize);
    }
    pub const fn rg1_am(&self) -> u8 {
        let val = (self.0 >> 5usize) & 0x07;
        val as u8
    }
    pub fn set_rg1_am(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 5usize)) | (((val as u8) & 0x07) << 5usize);
    }
}
impl Default for RxConf3 {
    fn default() -> RxConf3 {
        RxConf3(0)
    }
}
impl From<u8> for RxConf3 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<RxConf3> for u8 {
    fn from(val: RxConf3) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct IcIdentity(pub u8);
impl IcIdentity {
    pub const fn ic_rev(&self) -> IcIdentityIcRev {
        let val = (self.0 >> 0usize) & 0x07;
        IcIdentityIcRev(val as u8)
    }
    pub fn set_ic_rev(&mut self, val: IcIdentityIcRev) {
        self.0 = (self.0 & !(0x07 << 0usize)) | (((val.0 as u8) & 0x07) << 0usize);
    }
    pub const fn ic_type(&self) -> IcIdentityIcType {
        let val = (self.0 >> 3usize) & 0x1f;
        IcIdentityIcType(val as u8)
    }
    pub fn set_ic_type(&mut self, val: IcIdentityIcType) {
        self.0 = (self.0 & !(0x1f << 3usize)) | (((val.0 as u8) & 0x1f) << 3usize);
    }
}
impl Default for IcIdentity {
    fn default() -> IcIdentity {
        IcIdentity(0)
    }
}
impl From<u8> for IcIdentity {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<IcIdentity> for u8 {
    fn from(val: IcIdentity) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PtMod(pub u8);
impl PtMod {
    pub const fn pt_res(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x0f;
        val as u8
    }
    pub fn set_pt_res(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val as u8) & 0x0f) << 0usize);
    }
    pub const fn ptm_res(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    pub fn set_ptm_res(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for PtMod {
    fn default() -> PtMod {
        PtMod(0)
    }
}
impl From<u8> for PtMod {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<PtMod> for u8 {
    fn from(val: PtMod) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct WupTimerControl(pub u8);
impl WupTimerControl {
    pub const fn wcap(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_wcap(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn wph(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_wph(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn wam(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_wam(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn wto(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_wto(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn wut(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x07;
        val as u8
    }
    pub fn set_wut(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 4usize)) | (((val as u8) & 0x07) << 4usize);
    }
    pub const fn wur(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_wur(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for WupTimerControl {
    fn default() -> WupTimerControl {
        WupTimerControl(0)
    }
}
impl From<u8> for WupTimerControl {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<WupTimerControl> for u8 {
    fn from(val: WupTimerControl) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FieldThresholdDeactv(pub u8);
impl FieldThresholdDeactv {
    pub const fn rfe(&self) -> FieldThresholdDeactvRfe {
        let val = (self.0 >> 0usize) & 0x0f;
        FieldThresholdDeactvRfe(val as u8)
    }
    pub fn set_rfe(&mut self, val: FieldThresholdDeactvRfe) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val.0 as u8) & 0x0f) << 0usize);
    }
    pub const fn trg(&self) -> FieldThresholdDeactvTrg {
        let val = (self.0 >> 4usize) & 0x07;
        FieldThresholdDeactvTrg(val as u8)
    }
    pub fn set_trg(&mut self, val: FieldThresholdDeactvTrg) {
        self.0 = (self.0 & !(0x07 << 4usize)) | (((val.0 as u8) & 0x07) << 4usize);
    }
}
impl Default for FieldThresholdDeactv {
    fn default() -> FieldThresholdDeactv {
        FieldThresholdDeactv(0)
    }
}
impl From<u8> for FieldThresholdDeactv {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<FieldThresholdDeactv> for u8 {
    fn from(val: FieldThresholdDeactv) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct IoConf1(pub u8);
impl IoConf1 {
    pub const fn lf_clk_off(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_lf_clk_off(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn out_cl(&self) -> IoConf1OutCl {
        let val = (self.0 >> 1usize) & 0x03;
        IoConf1OutCl(val as u8)
    }
    pub fn set_out_cl(&mut self, val: IoConf1OutCl) {
        self.0 = (self.0 & !(0x03 << 1usize)) | (((val.0 as u8) & 0x03) << 1usize);
    }
    pub const fn rfu(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_rfu(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn i2c_thd(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x03;
        val as u8
    }
    pub fn set_i2c_thd(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 4usize)) | (((val as u8) & 0x03) << 4usize);
    }
    pub const fn rfo2(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_rfo2(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn single(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_single(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for IoConf1 {
    fn default() -> IoConf1 {
        IoConf1(0)
    }
}
impl From<u8> for IoConf1 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<IoConf1> for u8 {
    fn from(val: IoConf1) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RegulatorResult(pub u8);
impl RegulatorResult {
    pub const fn i_lim(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_i_lim(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn reg(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    pub fn set_reg(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for RegulatorResult {
    fn default() -> RegulatorResult {
        RegulatorResult(0)
    }
}
impl From<u8> for RegulatorResult {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<RegulatorResult> for u8 {
    fn from(val: RegulatorResult) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Iso14443b2(pub u8);
impl Iso14443b2 {
    pub const fn f_p(&self) -> Iso14443b2FP {
        let val = (self.0 >> 0usize) & 0x03;
        Iso14443b2FP(val as u8)
    }
    pub fn set_f_p(&mut self, val: Iso14443b2FP) {
        self.0 = (self.0 & !(0x03 << 0usize)) | (((val.0 as u8) & 0x03) << 0usize);
    }
    pub const fn no_eof(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_no_eof(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn no_sof(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_no_sof(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn tr1(&self) -> Iso14443b2Tr1 {
        let val = (self.0 >> 6usize) & 0x03;
        Iso14443b2Tr1(val as u8)
    }
    pub fn set_tr1(&mut self, val: Iso14443b2Tr1) {
        self.0 = (self.0 & !(0x03 << 6usize)) | (((val.0 as u8) & 0x03) << 6usize);
    }
}
impl Default for Iso14443b2 {
    fn default() -> Iso14443b2 {
        Iso14443b2(0)
    }
}
impl From<u8> for Iso14443b2 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Iso14443b2> for u8 {
    fn from(val: Iso14443b2) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FifoStatus2(pub u8);
impl FifoStatus2 {
    pub const fn np_lb(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_np_lb(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn fifo_lb(&self) -> u8 {
        let val = (self.0 >> 1usize) & 0x07;
        val as u8
    }
    pub fn set_fifo_lb(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 1usize)) | (((val as u8) & 0x07) << 1usize);
    }
    pub const fn fifo_ovr(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_fifo_ovr(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn fifo_unf(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_fifo_unf(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn fifo_b(&self) -> u8 {
        let val = (self.0 >> 6usize) & 0x03;
        val as u8
    }
    pub fn set_fifo_b(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 6usize)) | (((val as u8) & 0x03) << 6usize);
    }
}
impl Default for FifoStatus2 {
    fn default() -> FifoStatus2 {
        FifoStatus2(0)
    }
}
impl From<u8> for FifoStatus2 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<FifoStatus2> for u8 {
    fn from(val: FifoStatus2) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PassiveTargetStatus(pub u8);
impl PassiveTargetStatus {
    pub const fn pta_state(&self) -> PassiveTargetStatusPtaState {
        let val = (self.0 >> 0usize) & 0x0f;
        PassiveTargetStatusPtaState(val as u8)
    }
    pub fn set_pta_state(&mut self, val: PassiveTargetStatusPtaState) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val.0 as u8) & 0x0f) << 0usize);
    }
    pub const fn rfu3(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_rfu3(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn rfu2(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_rfu2(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn rfu1(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_rfu1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn rfu(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_rfu(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for PassiveTargetStatus {
    fn default() -> PassiveTargetStatus {
        PassiveTargetStatus(0)
    }
}
impl From<u8> for PassiveTargetStatus {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<PassiveTargetStatus> for u8 {
    fn from(val: PassiveTargetStatus) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CorrConf1(pub u8);
impl CorrConf1 {
    pub const fn corr_s0(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_corr_s0(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn corr_s1(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_corr_s1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn corr_s2(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_corr_s2(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn corr_s3(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_corr_s3(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn corr_s4(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_corr_s4(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn corr_s5(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_corr_s5(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn corr_s6(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_corr_s6(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn corr_s7(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_corr_s7(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for CorrConf1 {
    fn default() -> CorrConf1 {
        CorrConf1(0)
    }
}
impl From<u8> for CorrConf1 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<CorrConf1> for u8 {
    fn from(val: CorrConf1) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TxDriverStatus(pub u8);
impl TxDriverStatus {
    pub const fn d_tim(&self) -> TxDriverStatusDTim {
        let val = (self.0 >> 0usize) & 0x07;
        TxDriverStatusDTim(val as u8)
    }
    pub fn set_d_tim(&mut self, val: TxDriverStatusDTim) {
        self.0 = (self.0 & !(0x07 << 0usize)) | (((val.0 as u8) & 0x07) << 0usize);
    }
    pub const fn rfu(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_rfu(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn d_rat(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    pub fn set_d_rat(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for TxDriverStatus {
    fn default() -> TxDriverStatus {
        TxDriverStatus(0)
    }
}
impl From<u8> for TxDriverStatus {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<TxDriverStatus> for u8 {
    fn from(val: TxDriverStatus) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PhaseMeasureConf(pub u8);
impl PhaseMeasureConf {
    pub const fn pm_ae(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_pm_ae(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn pm_aew(&self) -> u8 {
        let val = (self.0 >> 1usize) & 0x03;
        val as u8
    }
    pub fn set_pm_aew(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 1usize)) | (((val as u8) & 0x03) << 1usize);
    }
    pub const fn pm_aam(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_pm_aam(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn pm_d(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    pub fn set_pm_d(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for PhaseMeasureConf {
    fn default() -> PhaseMeasureConf {
        PhaseMeasureConf(0)
    }
}
impl From<u8> for PhaseMeasureConf {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<PhaseMeasureConf> for u8 {
    fn from(val: PhaseMeasureConf) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct OvershootConf2(pub u8);
impl OvershootConf2 {
    pub const fn ov_pattern(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0xff;
        val as u8
    }
    pub fn set_ov_pattern(&mut self, val: u8) {
        self.0 = (self.0 & !(0xff << 0usize)) | (((val as u8) & 0xff) << 0usize);
    }
}
impl Default for OvershootConf2 {
    fn default() -> OvershootConf2 {
        OvershootConf2(0)
    }
}
impl From<u8> for OvershootConf2 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<OvershootConf2> for u8 {
    fn from(val: OvershootConf2) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PassiveTarget(pub u8);
impl PassiveTarget {
    pub const fn d_106_ac_a(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_d_106_ac_a(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn rfu(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_rfu(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn d_212_424_1r(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_d_212_424_1r(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn d_ac_ap2p(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_d_ac_ap2p(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn fdel(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    pub fn set_fdel(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for PassiveTarget {
    fn default() -> PassiveTarget {
        PassiveTarget(0)
    }
}
impl From<u8> for PassiveTarget {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<PassiveTarget> for u8 {
    fn from(val: PassiveTarget) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct OpControl(pub u8);
impl OpControl {
    pub const fn en_fd(&self) -> OpControlEnFd {
        let val = (self.0 >> 0usize) & 0x03;
        OpControlEnFd(val as u8)
    }
    pub fn set_en_fd(&mut self, val: OpControlEnFd) {
        self.0 = (self.0 & !(0x03 << 0usize)) | (((val.0 as u8) & 0x03) << 0usize);
    }
    pub const fn wu(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_wu(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn tx_en(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_tx_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn rx_man(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_rx_man(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn rx_chn(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_rx_chn(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn rx_en(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_rx_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn en(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for OpControl {
    fn default() -> OpControl {
        OpControl(0)
    }
}
impl From<u8> for OpControl {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<OpControl> for u8 {
    fn from(val: OpControl) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Mode(pub u8);
impl Mode {
    pub const fn nfc_ar(&self) -> ModeNfcAr {
        let val = (self.0 >> 0usize) & 0x03;
        ModeNfcAr(val as u8)
    }
    pub fn set_nfc_ar(&mut self, val: ModeNfcAr) {
        self.0 = (self.0 & !(0x03 << 0usize)) | (((val.0 as u8) & 0x03) << 0usize);
    }
    pub const fn tr_am(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_tr_am(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn om(&self) -> ModeOm {
        let val = (self.0 >> 3usize) & 0x0f;
        ModeOm(val as u8)
    }
    pub fn set_om(&mut self, val: ModeOm) {
        self.0 = (self.0 & !(0x0f << 3usize)) | (((val.0 as u8) & 0x0f) << 3usize);
    }
    pub const fn targ(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_targ(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
    pub const fn targ_targ(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_targ_targ(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Mode {
    fn default() -> Mode {
        Mode(0)
    }
}
impl From<u8> for Mode {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Mode> for u8 {
    fn from(val: Mode) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FieldThresholdActv(pub u8);
impl FieldThresholdActv {
    pub const fn rfe(&self) -> FieldThresholdActvRfe {
        let val = (self.0 >> 0usize) & 0x0f;
        FieldThresholdActvRfe(val as u8)
    }
    pub fn set_rfe(&mut self, val: FieldThresholdActvRfe) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val.0 as u8) & 0x0f) << 0usize);
    }
    pub const fn trg(&self) -> FieldThresholdActvTrg {
        let val = (self.0 >> 4usize) & 0x07;
        FieldThresholdActvTrg(val as u8)
    }
    pub fn set_trg(&mut self, val: FieldThresholdActvTrg) {
        self.0 = (self.0 & !(0x07 << 4usize)) | (((val.0 as u8) & 0x07) << 4usize);
    }
}
impl Default for FieldThresholdActv {
    fn default() -> FieldThresholdActv {
        FieldThresholdActv(0)
    }
}
impl From<u8> for FieldThresholdActv {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<FieldThresholdActv> for u8 {
    fn from(val: FieldThresholdActv) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AmplitudeMeasureConf(pub u8);
impl AmplitudeMeasureConf {
    pub const fn am_ae(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_am_ae(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn am_aew(&self) -> u8 {
        let val = (self.0 >> 1usize) & 0x03;
        val as u8
    }
    pub fn set_am_aew(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 1usize)) | (((val as u8) & 0x03) << 1usize);
    }
    pub const fn am_aam(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_am_aam(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn am_d(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    pub fn set_am_d(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for AmplitudeMeasureConf {
    fn default() -> AmplitudeMeasureConf {
        AmplitudeMeasureConf(0)
    }
}
impl From<u8> for AmplitudeMeasureConf {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<AmplitudeMeasureConf> for u8 {
    fn from(val: AmplitudeMeasureConf) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RxConf1(pub u8);
impl RxConf1 {
    pub const fn hz(&self) -> RxConf1Hz {
        let val = (self.0 >> 0usize) & 0x0f;
        RxConf1Hz(val as u8)
    }
    pub fn set_hz(&mut self, val: RxConf1Hz) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val.0 as u8) & 0x0f) << 0usize);
    }
    pub const fn z12k(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_z12k(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn h80(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_h80(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn h200(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_h200(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn z600k(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_z600k(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn lp(&self) -> RxConf1Lp {
        let val = (self.0 >> 4usize) & 0x07;
        RxConf1Lp(val as u8)
    }
    pub fn set_lp(&mut self, val: RxConf1Lp) {
        self.0 = (self.0 & !(0x07 << 4usize)) | (((val.0 as u8) & 0x07) << 4usize);
    }
    pub const fn ch_sel(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_ch_sel(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for RxConf1 {
    fn default() -> RxConf1 {
        RxConf1(0)
    }
}
impl From<u8> for RxConf1 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<RxConf1> for u8 {
    fn from(val: RxConf1) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CapSensorResult(pub u8);
impl CapSensorResult {
    pub const fn cs_cal_err(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_cs_cal_err(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn cs_cal_end(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_cs_cal_end(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn cs_cal_val(&self) -> u8 {
        let val = (self.0 >> 3usize) & 0x1f;
        val as u8
    }
    pub fn set_cs_cal_val(&mut self, val: u8) {
        self.0 = (self.0 & !(0x1f << 3usize)) | (((val as u8) & 0x1f) << 3usize);
    }
}
impl Default for CapSensorResult {
    fn default() -> CapSensorResult {
        CapSensorResult(0)
    }
}
impl From<u8> for CapSensorResult {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<CapSensorResult> for u8 {
    fn from(val: CapSensorResult) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CorrConf2(pub u8);
impl CorrConf2 {
    pub const fn corr_s8(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_corr_s8(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn corr_s9(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_corr_s9(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn rfu0(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_rfu0(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn rfu1(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_rfu1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn rfu2(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_rfu2(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn rfu3(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_rfu3(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn rfu4(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_rfu4(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn rfu5(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_rfu5(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for CorrConf2 {
    fn default() -> CorrConf2 {
        CorrConf2(0)
    }
}
impl From<u8> for CorrConf2 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<CorrConf2> for u8 {
    fn from(val: CorrConf2) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct UndershootConf1(pub u8);
impl UndershootConf1 {
    pub const fn un_pattern(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x3f;
        val as u8
    }
    pub fn set_un_pattern(&mut self, val: u8) {
        self.0 = (self.0 & !(0x3f << 0usize)) | (((val as u8) & 0x3f) << 0usize);
    }
    pub const fn un_tx_mode0(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_un_tx_mode0(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn un_tx_mode1(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_un_tx_mode1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for UndershootConf1 {
    fn default() -> UndershootConf1 {
        UndershootConf1(0)
    }
}
impl From<u8> for UndershootConf1 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<UndershootConf1> for u8 {
    fn from(val: UndershootConf1) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Iso14443b1(pub u8);
impl Iso14443b1 {
    pub const fn rx_st_om(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_rx_st_om(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn half(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_half(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn eof(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_eof(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn eof_11etu(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_eof_11etu(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn sof_1(&self) -> Iso14443b1Sof1 {
        let val = (self.0 >> 3usize) & 0x01;
        Iso14443b1Sof1(val as u8)
    }
    pub fn set_sof_1(&mut self, val: Iso14443b1Sof1) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val.0 as u8) & 0x01) << 3usize);
    }
    pub const fn sof_0_11etu(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_sof_0_11etu(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn sof_0_mak(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_sof_0_mak(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn egt(&self) -> u8 {
        let val = (self.0 >> 5usize) & 0x07;
        val as u8
    }
    pub fn set_egt(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 5usize)) | (((val as u8) & 0x07) << 5usize);
    }
}
impl Default for Iso14443b1 {
    fn default() -> Iso14443b1 {
        Iso14443b1(0)
    }
}
impl From<u8> for Iso14443b1 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Iso14443b1> for u8 {
    fn from(val: Iso14443b1) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct IoConf2(pub u8);
impl IoConf2 {
    pub const fn slow_up(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_slow_up(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn am_ref_rf(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_am_ref_rf(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn io_drv_lvl(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_io_drv_lvl(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn miso_pd1(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_miso_pd1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn miso_pd2(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_miso_pd2(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn aat_en(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_aat_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn sup_3v(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_sup_3v(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
    pub const fn vspd_off(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_vspd_off(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
}
impl Default for IoConf2 {
    fn default() -> IoConf2 {
        IoConf2(0)
    }
}
impl From<u8> for IoConf2 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<IoConf2> for u8 {
    fn from(val: IoConf2) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CapacitanceMeasureConf(pub u8);
impl CapacitanceMeasureConf {
    pub const fn cm_ae(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_cm_ae(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn cm_aew(&self) -> u8 {
        let val = (self.0 >> 1usize) & 0x03;
        val as u8
    }
    pub fn set_cm_aew(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 1usize)) | (((val as u8) & 0x03) << 1usize);
    }
    pub const fn cm_aam(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_cm_aam(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn cm_d(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    pub fn set_cm_d(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for CapacitanceMeasureConf {
    fn default() -> CapacitanceMeasureConf {
        CapacitanceMeasureConf(0)
    }
}
impl From<u8> for CapacitanceMeasureConf {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<CapacitanceMeasureConf> for u8 {
    fn from(val: CapacitanceMeasureConf) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RxConf4(pub u8);
impl RxConf4 {
    pub const fn rg2_pm(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x0f;
        val as u8
    }
    pub fn set_rg2_pm(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val as u8) & 0x0f) << 0usize);
    }
    pub const fn rg2_am(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    pub fn set_rg2_am(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for RxConf4 {
    fn default() -> RxConf4 {
        RxConf4(0)
    }
}
impl From<u8> for RxConf4 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<RxConf4> for u8 {
    fn from(val: RxConf4) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct OvershootConf1(pub u8);
impl OvershootConf1 {
    pub const fn ov_pattern(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x3f;
        val as u8
    }
    pub fn set_ov_pattern(&mut self, val: u8) {
        self.0 = (self.0 & !(0x3f << 0usize)) | (((val as u8) & 0x3f) << 0usize);
    }
    pub const fn ov_tx_mode0(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_ov_tx_mode0(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn ov_tx_mode1(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_ov_tx_mode1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for OvershootConf1 {
    fn default() -> OvershootConf1 {
        OvershootConf1(0)
    }
}
impl From<u8> for OvershootConf1 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<OvershootConf1> for u8 {
    fn from(val: OvershootConf1) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TxDriverTiming(pub u8);
impl TxDriverTiming {
    pub const fn d_tim_m(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x07;
        val as u8
    }
    pub fn set_d_tim_m(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 0usize)) | (((val as u8) & 0x07) << 0usize);
    }
    pub const fn rfu(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_rfu(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn d_rat(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    pub fn set_d_rat(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for TxDriverTiming {
    fn default() -> TxDriverTiming {
        TxDriverTiming(0)
    }
}
impl From<u8> for TxDriverTiming {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<TxDriverTiming> for u8 {
    fn from(val: TxDriverTiming) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct P2pRxConf(pub u8);
impl P2pRxConf {
    pub const fn ask_thd(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_ask_thd(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn ask_rc0(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_ask_rc0(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn ask_rc1(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_ask_rc1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn ook_thd0(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_ook_thd0(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn ook_thd1(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_ook_thd1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn ook_rc0(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_ook_rc0(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn ook_rc1(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_ook_rc1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn ook_fd(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_ook_fd(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for P2pRxConf {
    fn default() -> P2pRxConf {
        P2pRxConf(0)
    }
}
impl From<u8> for P2pRxConf {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<P2pRxConf> for u8 {
    fn from(val: P2pRxConf) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Iso14443aNfc(pub u8);
impl Iso14443aNfc {
    pub const fn antcl(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_antcl(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn p_len(&self) -> u8 {
        let val = (self.0 >> 1usize) & 0x0f;
        val as u8
    }
    pub fn set_p_len(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 1usize)) | (((val as u8) & 0x0f) << 1usize);
    }
    pub const fn nfc_f0(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_nfc_f0(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn no_rx_par(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_no_rx_par(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn no_tx_par(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_no_tx_par(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Iso14443aNfc {
    fn default() -> Iso14443aNfc {
        Iso14443aNfc(0)
    }
}
impl From<u8> for Iso14443aNfc {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Iso14443aNfc> for u8 {
    fn from(val: Iso14443aNfc) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct UndershootConf2(pub u8);
impl UndershootConf2 {
    pub const fn un_pattern(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0xff;
        val as u8
    }
    pub fn set_un_pattern(&mut self, val: u8) {
        self.0 = (self.0 & !(0xff << 0usize)) | (((val as u8) & 0xff) << 0usize);
    }
}
impl Default for UndershootConf2 {
    fn default() -> UndershootConf2 {
        UndershootConf2(0)
    }
}
impl From<u8> for UndershootConf2 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<UndershootConf2> for u8 {
    fn from(val: UndershootConf2) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CapSensorControl(pub u8);
impl CapSensorControl {
    pub const fn cs_g(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x07;
        val as u8
    }
    pub fn set_cs_g(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 0usize)) | (((val as u8) & 0x07) << 0usize);
    }
    pub const fn cs_mcal(&self) -> u8 {
        let val = (self.0 >> 3usize) & 0x1f;
        val as u8
    }
    pub fn set_cs_mcal(&mut self, val: u8) {
        self.0 = (self.0 & !(0x1f << 3usize)) | (((val as u8) & 0x1f) << 3usize);
    }
}
impl Default for CapSensorControl {
    fn default() -> CapSensorControl {
        CapSensorControl(0)
    }
}
impl From<u8> for CapSensorControl {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<CapSensorControl> for u8 {
    fn from(val: CapSensorControl) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TestUnk(pub u8);
impl TestUnk {
    pub const fn dis_overheat_prot(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_dis_overheat_prot(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
}
impl Default for TestUnk {
    fn default() -> TestUnk {
        TestUnk(0)
    }
}
impl From<u8> for TestUnk {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<TestUnk> for u8 {
    fn from(val: TestUnk) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NumTxBytes2(pub u8);
impl NumTxBytes2 {
    pub const fn nbtx(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x07;
        val as u8
    }
    pub fn set_nbtx(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 0usize)) | (((val as u8) & 0x07) << 0usize);
    }
    pub const fn ntx(&self) -> u8 {
        let val = (self.0 >> 3usize) & 0x1f;
        val as u8
    }
    pub fn set_ntx(&mut self, val: u8) {
        self.0 = (self.0 & !(0x1f << 3usize)) | (((val as u8) & 0x1f) << 3usize);
    }
}
impl Default for NumTxBytes2 {
    fn default() -> NumTxBytes2 {
        NumTxBytes2(0)
    }
}
impl From<u8> for NumTxBytes2 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<NumTxBytes2> for u8 {
    fn from(val: NumTxBytes2) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TimerEmvControl(pub u8);
impl TimerEmvControl {
    pub const fn nrt_step(&self) -> TimerEmvControlNrtStep {
        let val = (self.0 >> 0usize) & 0x01;
        TimerEmvControlNrtStep(val as u8)
    }
    pub fn set_nrt_step(&mut self, val: TimerEmvControlNrtStep) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val.0 as u8) & 0x01) << 0usize);
    }
    pub const fn nrt_emv(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_nrt_emv(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn nrt_nfc(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_nrt_nfc(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn mrt_step(&self) -> TimerEmvControlMrtStep {
        let val = (self.0 >> 3usize) & 0x01;
        TimerEmvControlMrtStep(val as u8)
    }
    pub fn set_mrt_step(&mut self, val: TimerEmvControlMrtStep) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val.0 as u8) & 0x01) << 3usize);
    }
    pub const fn rfu(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_rfu(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn gptc(&self) -> TimerEmvControlGptc {
        let val = (self.0 >> 5usize) & 0x07;
        TimerEmvControlGptc(val as u8)
    }
    pub fn set_gptc(&mut self, val: TimerEmvControlGptc) {
        self.0 = (self.0 & !(0x07 << 5usize)) | (((val.0 as u8) & 0x07) << 5usize);
    }
}
impl Default for TimerEmvControl {
    fn default() -> TimerEmvControl {
        TimerEmvControl(0)
    }
}
impl From<u8> for TimerEmvControl {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<TimerEmvControl> for u8 {
    fn from(val: TimerEmvControl) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct GainRedState(pub u8);
impl GainRedState {
    pub const fn gs_pm(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x0f;
        val as u8
    }
    pub fn set_gs_pm(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val as u8) & 0x0f) << 0usize);
    }
    pub const fn gs_am(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    pub fn set_gs_am(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for GainRedState {
    fn default() -> GainRedState {
        GainRedState(0)
    }
}
impl From<u8> for GainRedState {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<GainRedState> for u8 {
    fn from(val: GainRedState) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AuxDisplay(pub u8);
impl AuxDisplay {
    pub const fn en_ac(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_en_ac(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn en_peer(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_en_peer(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn rx_act(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_rx_act(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn rx_on(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_rx_on(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn osc_ok(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    pub fn set_osc_ok(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    pub const fn tx_on(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    pub fn set_tx_on(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    pub const fn efd_o(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_efd_o(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn a_cha(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_a_cha(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for AuxDisplay {
    fn default() -> AuxDisplay {
        AuxDisplay(0)
    }
}
impl From<u8> for AuxDisplay {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<AuxDisplay> for u8 {
    fn from(val: AuxDisplay) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ResAmMod(pub u8);
impl ResAmMod {
    pub const fn md_res(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x7f;
        val as u8
    }
    pub fn set_md_res(&mut self, val: u8) {
        self.0 = (self.0 & !(0x7f << 0usize)) | (((val as u8) & 0x7f) << 0usize);
    }
    pub const fn fa3_f(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_fa3_f(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for ResAmMod {
    fn default() -> ResAmMod {
        ResAmMod(0)
    }
}
impl From<u8> for ResAmMod {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<ResAmMod> for u8 {
    fn from(val: ResAmMod) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Aux(pub u8);
impl Aux {
    pub const fn nfc_n(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x03;
        val as u8
    }
    pub fn set_nfc_n(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 0usize)) | (((val as u8) & 0x03) << 0usize);
    }
    pub const fn dis_corr(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_dis_corr(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn mfaz_cl90(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_mfaz_cl90(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn nfc_id(&self) -> AuxNfcId {
        let val = (self.0 >> 4usize) & 0x03;
        AuxNfcId(val as u8)
    }
    pub fn set_nfc_id(&mut self, val: AuxNfcId) {
        self.0 = (self.0 & !(0x03 << 4usize)) | (((val.0 as u8) & 0x03) << 4usize);
    }
    pub const fn rfu(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_rfu(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn no_crc_rx(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_no_crc_rx(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Aux {
    fn default() -> Aux {
        Aux(0)
    }
}
impl From<u8> for Aux {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Aux> for u8 {
    fn from(val: Aux) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Nfcip1BitRate(pub u8);
impl Nfcip1BitRate {
    pub const fn mrt_on(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    pub fn set_mrt_on(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    pub const fn nrt_on(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    pub fn set_nrt_on(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    pub const fn gpt_on(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    pub fn set_gpt_on(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    pub const fn ppt2_on(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    pub fn set_ppt2_on(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    pub const fn nfc_rate(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x03;
        val as u8
    }
    pub fn set_nfc_rate(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 4usize)) | (((val as u8) & 0x03) << 4usize);
    }
    pub const fn nfc_rfu0(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    pub fn set_nfc_rfu0(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    pub const fn nfc_rfu1(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    pub fn set_nfc_rfu1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Nfcip1BitRate {
    fn default() -> Nfcip1BitRate {
        Nfcip1BitRate(0)
    }
}
impl From<u8> for Nfcip1BitRate {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Nfcip1BitRate> for u8 {
    fn from(val: Nfcip1BitRate) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Iso14443b2FP(pub u8);
impl Iso14443b2FP {
    pub const _48: Self = Self(0);
    pub const _64: Self = Self(0x01);
    pub const _80: Self = Self(0x02);
    pub const _96: Self = Self(0x03);
}
impl From<u8> for Iso14443b2FP {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Iso14443b2FP> for u8 {
    fn from(val: Iso14443b2FP) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TimerEmvControlNrtStep(pub u8);
impl TimerEmvControlNrtStep {
    pub const _64FC: Self = Self(0);
    pub const _4096_FC: Self = Self(0x01);
}
impl From<u8> for TimerEmvControlNrtStep {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<TimerEmvControlNrtStep> for u8 {
    fn from(val: TimerEmvControlNrtStep) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct StreamModeStx(pub u8);
impl StreamModeStx {
    pub const _106: Self = Self(0);
    pub const _212: Self = Self(0x01);
    pub const _424: Self = Self(0x02);
    pub const _848: Self = Self(0x03);
}
impl From<u8> for StreamModeStx {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<StreamModeStx> for u8 {
    fn from(val: StreamModeStx) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct FieldThresholdDeactvRfe(pub u8);
impl FieldThresholdDeactvRfe {
    pub const _75MV: Self = Self(0);
    pub const _105MV: Self = Self(0x01);
    pub const _150MV: Self = Self(0x02);
    pub const _205MV: Self = Self(0x03);
    pub const _290MV: Self = Self(0x04);
    pub const _400MV: Self = Self(0x05);
    pub const _560MV: Self = Self(0x06);
    pub const _800MV: Self = Self(0x07);
    pub const _25MV: Self = Self(0x08);
    pub const _33MV: Self = Self(0x09);
    pub const _47MV: Self = Self(0x0a);
    pub const _64MV: Self = Self(0x0b);
    pub const _90MV: Self = Self(0x0c);
    pub const _125MV: Self = Self(0x0d);
    pub const _175MV: Self = Self(0x0e);
    pub const _250MV: Self = Self(0x0f);
}
impl From<u8> for FieldThresholdDeactvRfe {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<FieldThresholdDeactvRfe> for u8 {
    fn from(val: FieldThresholdDeactvRfe) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IcIdentityIcType(pub u8);
impl IcIdentityIcType {
    pub const ST25R3916: Self = Self(0x05);
}
impl From<u8> for IcIdentityIcType {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<IcIdentityIcType> for u8 {
    fn from(val: IcIdentityIcType) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PassiveTargetStatusPtaState(pub u8);
impl PassiveTargetStatusPtaState {
    pub const POWER_OFF: Self = Self(0);
    pub const IDLE: Self = Self(0x01);
    pub const READY_L1: Self = Self(0x02);
    pub const READY_L2: Self = Self(0x03);
    pub const RFU4: Self = Self(0x04);
    pub const ACTIVE: Self = Self(0x05);
    pub const RFU6: Self = Self(0x06);
    pub const RFU7: Self = Self(0x07);
    pub const RFU8: Self = Self(0x08);
    pub const HALT: Self = Self(0x09);
    pub const READY_L1_X: Self = Self(0x0a);
    pub const READY_L2_X: Self = Self(0x0b);
    pub const RFU12: Self = Self(0x0c);
    pub const ACTIVE_X: Self = Self(0x0d);
}
impl From<u8> for PassiveTargetStatusPtaState {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<PassiveTargetStatusPtaState> for u8 {
    fn from(val: PassiveTargetStatusPtaState) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct FieldThresholdActvRfe(pub u8);
impl FieldThresholdActvRfe {
    pub const _75MV: Self = Self(0);
    pub const _105MV: Self = Self(0x01);
    pub const _150MV: Self = Self(0x02);
    pub const _205MV: Self = Self(0x03);
    pub const _290MV: Self = Self(0x04);
    pub const _400MV: Self = Self(0x05);
    pub const _560MV: Self = Self(0x06);
    pub const _800MV: Self = Self(0x07);
    pub const _25MV: Self = Self(0x08);
    pub const _33MV: Self = Self(0x09);
    pub const _47MV: Self = Self(0x0a);
    pub const _64MV: Self = Self(0x0b);
    pub const _90MV: Self = Self(0x0c);
    pub const _125MV: Self = Self(0x0d);
    pub const _175MV: Self = Self(0x0e);
    pub const _250MV: Self = Self(0x0f);
}
impl From<u8> for FieldThresholdActvRfe {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<FieldThresholdActvRfe> for u8 {
    fn from(val: FieldThresholdActvRfe) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IcIdentityIcRev(pub u8);
impl IcIdentityIcRev {
    pub const V0: Self = Self(0);
}
impl From<u8> for IcIdentityIcRev {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<IcIdentityIcRev> for u8 {
    fn from(val: IcIdentityIcRev) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OpControlEnFd(pub u8);
impl OpControlEnFd {
    pub const EFD_OFF: Self = Self(0);
    pub const MANUAL_EFD_CA: Self = Self(0x01);
    pub const MANUAL_EFD_PDT: Self = Self(0x02);
    pub const AUTO_EFD: Self = Self(0x03);
}
impl From<u8> for OpControlEnFd {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<OpControlEnFd> for u8 {
    fn from(val: OpControlEnFd) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IoConf1OutCl(pub u8);
impl IoConf1OutCl {
    pub const _3_39_MHZ: Self = Self(0);
    pub const _6_78_MHZ: Self = Self(0x01);
    pub const _13_86_MHZ: Self = Self(0x02);
    pub const DISABLED: Self = Self(0x03);
}
impl From<u8> for IoConf1OutCl {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<IoConf1OutCl> for u8 {
    fn from(val: IoConf1OutCl) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Iso14443b1Sof1(pub u8);
impl Iso14443b1Sof1 {
    pub const _2ETU: Self = Self(0);
    pub const _3ETU: Self = Self(0x01);
}
impl From<u8> for Iso14443b1Sof1 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Iso14443b1Sof1> for u8 {
    fn from(val: Iso14443b1Sof1) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RxConf1Hz(pub u8);
impl RxConf1Hz {
    pub const _60_400KHZ: Self = Self(0);
    pub const _12_200KHZ: Self = Self(0x01);
    pub const _40_80KHZ: Self = Self(0x02);
    pub const _12_80KHZ: Self = Self(0x03);
    pub const _60_200KHZ: Self = Self(0x04);
    pub const _12_200KHZ_ALT: Self = Self(0x05);
    pub const _600_400KHZ: Self = Self(0x08);
    pub const _600_200KHZ: Self = Self(0x0c);
}
impl From<u8> for RxConf1Hz {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<RxConf1Hz> for u8 {
    fn from(val: RxConf1Hz) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct StreamModeScp(pub u8);
impl StreamModeScp {
    pub const _1PULSE: Self = Self(0);
    pub const _2PULSES: Self = Self(0x01);
    pub const _4PULSES: Self = Self(0x02);
    pub const _8PULSES: Self = Self(0x03);
}
impl From<u8> for StreamModeScp {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<StreamModeScp> for u8 {
    fn from(val: StreamModeScp) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ModeNfcAr(pub u8);
impl ModeNfcAr {
    pub const OFF: Self = Self(0);
    pub const AUTO_RX: Self = Self(0x01);
    pub const EOF: Self = Self(0x02);
    pub const RFU: Self = Self(0x03);
}
impl From<u8> for ModeNfcAr {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<ModeNfcAr> for u8 {
    fn from(val: ModeNfcAr) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TimerEmvControlGptc(pub u8);
impl TimerEmvControlGptc {
    pub const NO_TRIGGER: Self = Self(0);
    pub const ERX: Self = Self(0x01);
    pub const SRX: Self = Self(0x02);
    pub const ETX_NFC: Self = Self(0x03);
}
impl From<u8> for TimerEmvControlGptc {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<TimerEmvControlGptc> for u8 {
    fn from(val: TimerEmvControlGptc) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BitRateE(pub u8);
impl BitRateE {
    pub const _106: Self = Self(0);
    pub const _212: Self = Self(0x01);
    pub const _424: Self = Self(0x02);
    pub const _848: Self = Self(0x03);
}
impl From<u8> for BitRateE {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<BitRateE> for u8 {
    fn from(val: BitRateE) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RegulatorControlMpsv(pub u8);
impl RegulatorControlMpsv {
    pub const VDD: Self = Self(0);
    pub const VDD_A: Self = Self(0x01);
    pub const VDD_D: Self = Self(0x02);
    pub const VDD_RF: Self = Self(0x03);
    pub const VDD_AM: Self = Self(0x04);
    pub const RFU: Self = Self(0x05);
    pub const RFU1: Self = Self(0x06);
    pub const RFU2: Self = Self(0x07);
}
impl From<u8> for RegulatorControlMpsv {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<RegulatorControlMpsv> for u8 {
    fn from(val: RegulatorControlMpsv) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RxConf1Lp(pub u8);
impl RxConf1Lp {
    pub const _1200KHZ: Self = Self(0);
    pub const _600KHZ: Self = Self(0x01);
    pub const _300KHZ: Self = Self(0x02);
    pub const _2000KHZ: Self = Self(0x04);
    pub const _7000KHZ: Self = Self(0x05);
}
impl From<u8> for RxConf1Lp {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<RxConf1Lp> for u8 {
    fn from(val: RxConf1Lp) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct TxDriverDRes(pub u8);
impl TxDriverDRes {
    pub const _1_00: Self = Self(0);
    pub const _1_19: Self = Self(0x01);
    pub const _1_40: Self = Self(0x02);
    pub const _1_61: Self = Self(0x03);
    pub const _1_79: Self = Self(0x04);
    pub const _2_02: Self = Self(0x05);
    pub const _2_49: Self = Self(0x06);
    pub const _2_94: Self = Self(0x07);
    pub const _3_41: Self = Self(0x08);
    pub const _4_06: Self = Self(0x09);
    pub const _5_95: Self = Self(0x0a);
    pub const _8_26: Self = Self(0x0b);
    pub const _17_10: Self = Self(0x0c);
    pub const _36_60: Self = Self(0x0d);
    pub const _51_20: Self = Self(0x0e);
    pub const _HIGH_Z: Self = Self(0x0f);
}
impl From<u8> for TxDriverDRes {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<TxDriverDRes> for u8 {
    fn from(val: TxDriverDRes) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TxDriverAmMod(pub u8);
impl TxDriverAmMod {
    pub const _5PERCENT: Self = Self(0);
    pub const _6PERCENT: Self = Self(0x01);
    pub const _7PERCENT: Self = Self(0x02);
    pub const _8PERCENT: Self = Self(0x03);
    pub const _9PERCENT: Self = Self(0x04);
    pub const _10PERCENT: Self = Self(0x05);
    pub const _11PERCENT: Self = Self(0x06);
    pub const _12PERCENT: Self = Self(0x07);
    pub const _13PERCENT: Self = Self(0x08);
    pub const _14PERCENT: Self = Self(0x09);
    pub const _15PERCENT: Self = Self(0x0a);
    pub const _17PERCENT: Self = Self(0x0b);
    pub const _19PERCENT: Self = Self(0x0c);
    pub const _22PERCENT: Self = Self(0x0d);
    pub const _26PERCENT: Self = Self(0x0e);
    pub const _40PERCENT: Self = Self(0x0f);
}
impl From<u8> for TxDriverAmMod {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<TxDriverAmMod> for u8 {
    fn from(val: TxDriverAmMod) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AuxNfcId(pub u8);
impl AuxNfcId {
    pub const _4BYTES: Self = Self(0);
    pub const _7BYTES: Self = Self(0x01);
}
impl From<u8> for AuxNfcId {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<AuxNfcId> for u8 {
    fn from(val: AuxNfcId) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct FieldThresholdDeactvTrg(pub u8);
impl FieldThresholdDeactvTrg {
    pub const _75MV: Self = Self(0);
    pub const _105MV: Self = Self(0x01);
    pub const _150MV: Self = Self(0x02);
    pub const _205MV: Self = Self(0x03);
    pub const _290MV: Self = Self(0x04);
    pub const _400MV: Self = Self(0x05);
    pub const _560MV: Self = Self(0x06);
    pub const _800MV: Self = Self(0x07);
}
impl From<u8> for FieldThresholdDeactvTrg {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<FieldThresholdDeactvTrg> for u8 {
    fn from(val: FieldThresholdDeactvTrg) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct StreamModeScf(pub u8);
impl StreamModeScf {
    pub const BPSK848: Self = Self(0);
    pub const SC212: Self = Self(0);
    pub const BPSK1695: Self = Self(0x01);
    pub const SC424: Self = Self(0x01);
    pub const BPSK3390: Self = Self(0x02);
    pub const SC848: Self = Self(0x02);
    pub const BPSK106: Self = Self(0x03);
    pub const SC1695: Self = Self(0x03);
}
impl From<u8> for StreamModeScf {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<StreamModeScf> for u8 {
    fn from(val: StreamModeScf) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ModeOm(pub u8);
impl ModeOm {
    pub const INI_NFC: Self = Self(0);
    pub const INI_ISO14443A: Self = Self(0x01);
    pub const TARG_NFCA: Self = Self(0x01);
    pub const INI_ISO14443B: Self = Self(0x02);
    pub const TARG_NFCB: Self = Self(0x02);
    pub const INI_FELICA: Self = Self(0x03);
    pub const INI_TOPAZ: Self = Self(0x04);
    pub const TARG_NFCF: Self = Self(0x04);
    pub const TARG_NFCIP: Self = Self(0x07);
    pub const INI_SUBCARRIER_STREAM: Self = Self(0x0e);
    pub const INI_BPSK_STREAM: Self = Self(0x0f);
}
impl From<u8> for ModeOm {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<ModeOm> for u8 {
    fn from(val: ModeOm) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct FieldThresholdActvTrg(pub u8);
impl FieldThresholdActvTrg {
    pub const _75MV: Self = Self(0);
    pub const _105MV: Self = Self(0x01);
    pub const _150MV: Self = Self(0x02);
    pub const _205MV: Self = Self(0x03);
    pub const _290MV: Self = Self(0x04);
    pub const _400MV: Self = Self(0x05);
    pub const _560MV: Self = Self(0x06);
    pub const _800MV: Self = Self(0x07);
}
impl From<u8> for FieldThresholdActvTrg {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<FieldThresholdActvTrg> for u8 {
    fn from(val: FieldThresholdActvTrg) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TimerEmvControlMrtStep(pub u8);
impl TimerEmvControlMrtStep {
    pub const _64: Self = Self(0);
    pub const _512: Self = Self(0x01);
}
impl From<u8> for TimerEmvControlMrtStep {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<TimerEmvControlMrtStep> for u8 {
    fn from(val: TimerEmvControlMrtStep) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Iso14443b2Tr1(pub u8);
impl Iso14443b2Tr1 {
    pub const _80FS80FS: Self = Self(0);
    pub const _64FS32FS: Self = Self(0x01);
}
impl From<u8> for Iso14443b2Tr1 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Iso14443b2Tr1> for u8 {
    fn from(val: Iso14443b2Tr1) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TxDriverStatusDTim(pub u8);
impl TxDriverStatusDTim {
    pub const SLOW: Self = Self(0);
    pub const MEDIUM_SLOW: Self = Self(0x01);
    pub const NOMINAL: Self = Self(0x02);
    pub const MEDIUM_FAST: Self = Self(0x03);
    pub const FAST: Self = Self(0x04);
}
impl From<u8> for TxDriverStatusDTim {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<TxDriverStatusDTim> for u8 {
    fn from(val: TxDriverStatusDTim) -> u8 {
        val.0
    }
}
