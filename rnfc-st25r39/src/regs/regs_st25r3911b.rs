#![allow(unused)]

use core::marker::PhantomData;

use crate::{Error, Interface};

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
pub struct Regs<'a, I: Interface> {
    iface: &'a mut I,
}

impl<'a, I: Interface> Regs<'a, I> {
    pub fn new(iface: &'a mut I) -> Self {
        Self { iface }
    }

    /// IO configuration register 1
    pub fn io_conf1(&mut self) -> Reg<'_, I, IoConf1> {
        Reg::new(self.iface, 0)
    }
    /// IO configuration register 2
    pub fn io_conf2(&mut self) -> Reg<'_, I, IoConf2> {
        Reg::new(self.iface, 1)
    }
    /// Operation control register
    pub fn op_control(&mut self) -> Reg<'_, I, OpControl> {
        Reg::new(self.iface, 2)
    }
    /// Mode definition register
    pub fn mode(&mut self) -> Reg<'_, I, Mode> {
        Reg::new(self.iface, 3)
    }
    /// Bit rate definition register
    pub fn bit_rate(&mut self) -> Reg<'_, I, BitRate> {
        Reg::new(self.iface, 4)
    }
    /// ISO14443A and NFC 106kb/s settings register
    pub fn iso14443a_nfc(&mut self) -> Reg<'_, I, Iso14443ANfc> {
        Reg::new(self.iface, 5)
    }
    /// ISO14443B settings register 1
    pub fn iso14443b_1(&mut self) -> Reg<'_, I, Iso14443B1> {
        Reg::new(self.iface, 6)
    }
    /// ISO14443B and FeliCa settings register
    pub fn iso14443b_2(&mut self) -> Reg<'_, I, Iso14443B2> {
        Reg::new(self.iface, 7)
    }
    /// Stream mode definition register
    pub fn stream_mode(&mut self) -> Reg<'_, I, StreamMode> {
        Reg::new(self.iface, 8)
    }
    /// Auxiliary definition register
    pub fn aux(&mut self) -> Reg<'_, I, Aux> {
        Reg::new(self.iface, 9)
    }
    /// Receiver configuration register 1
    pub fn rx_conf1(&mut self) -> Reg<'_, I, RxConf1> {
        Reg::new(self.iface, 10)
    }
    /// Receiver configuration register 2
    pub fn rx_conf2(&mut self) -> Reg<'_, I, RxConf2> {
        Reg::new(self.iface, 11)
    }
    /// Receiver configuration register 3
    pub fn rx_conf3(&mut self) -> Reg<'_, I, RxConf3> {
        Reg::new(self.iface, 12)
    }
    /// Receiver configuration register 4
    pub fn rx_conf4(&mut self) -> Reg<'_, I, RxConf4> {
        Reg::new(self.iface, 13)
    }
    /// Mask receive timer register
    pub fn mask_rx_timer(&mut self) -> Reg<'_, I, MaskRxTimer> {
        Reg::new(self.iface, 14)
    }
    /// No-response timer register 1
    pub fn no_response_timer1(&mut self) -> Reg<'_, I, NoResponseTimer1> {
        Reg::new(self.iface, 15)
    }
    /// No-response timer register 2
    pub fn no_response_timer2(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 16)
    }
    /// General purpose and no-response timer control register
    pub fn gpt_nrt_ctrl(&mut self) -> Reg<'_, I, GptNrtCtrl> {
        Reg::new(self.iface, 17)
    }
    /// General purpose timer register 1
    pub fn gpt1(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 18)
    }
    /// General purpose timer register 2
    pub fn gpt2(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 19)
    }
    /// Main interrupt register mask, combined wth offset to other irq mask regs
    pub fn irq_mask(&mut self, n: u8) -> Reg<'_, I, u8> {
        assert!(n < 3);
        Reg::new(self.iface, 20 + n)
    }
    /// Main interrupt register mask
    pub fn irq_mask_main(&mut self) -> Reg<'_, I, IrqMaskMain> {
        Reg::new(self.iface, 20)
    }
    /// Mask timer and NFC interrupt register
    pub fn irq_mask_timer_nfc(&mut self) -> Reg<'_, I, IrqMaskTimerNfc> {
        Reg::new(self.iface, 21)
    }
    /// Mask error and wake-up interrupt register
    pub fn irq_mask_error_wup(&mut self) -> Reg<'_, I, IrqMaskErrorWup> {
        Reg::new(self.iface, 22)
    }
    /// Main interrupt register
    pub fn irq_main_direct(&mut self) -> Reg<'_, I, IrqMain> {
        Reg::new(self.iface, 23)
    }
    /// Main interrupt register, combined with offset to other regs
    pub fn irq_main(&mut self, n: u8) -> Reg<'_, I, u8> {
        assert!(n < 3);
        Reg::new(self.iface, 23 + n)
    }
    /// Mask timer and NFC interrupt register
    pub fn irq_timer_nfc(&mut self) -> Reg<'_, I, IrqTimerNfc> {
        Reg::new(self.iface, 24)
    }
    /// Error and wake-up interrupt register
    pub fn irq_error_wup(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 25)
    }
    /// FIFO status register 1
    pub fn fifo_status1(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 26)
    }
    /// FIFO status register 2
    pub fn fifo_status2(&mut self) -> Reg<'_, I, FifoStatus2> {
        Reg::new(self.iface, 27)
    }
    /// Collision display register
    pub fn collision_status(&mut self) -> Reg<'_, I, CollisionStatus> {
        Reg::new(self.iface, 28)
    }
    /// Number of transmitted bytes register 1
    pub fn num_tx_bytes1(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 29)
    }
    /// Number of transmitted bytes register 2
    pub fn num_tx_bytes2(&mut self) -> Reg<'_, I, NumTxBytes2> {
        Reg::new(self.iface, 30)
    }
    /// NFCIP bit rate detection display register
    pub fn nfcip1_bit_rate_disp(&mut self) -> Reg<'_, I, Nfcip1BitRateDisp> {
        Reg::new(self.iface, 31)
    }
    /// A/D converter output register
    pub fn ad_result(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 32)
    }
    /// Antenna calibration control register
    pub fn ant_tune_ctrl(&mut self) -> Reg<'_, I, AntTuneCtrl> {
        Reg::new(self.iface, 33)
    }
    /// Antenna calibration target register
    pub fn ant_tune_target(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 34)
    }
    /// Antenna calibration display register
    pub fn ant_tune_disp(&mut self) -> Reg<'_, I, AntTuneDisp> {
        Reg::new(self.iface, 35)
    }
    /// AM modulation depth control register
    pub fn am_mod_depth_ctrl(&mut self) -> Reg<'_, I, AmModDepthCtrl> {
        Reg::new(self.iface, 36)
    }
    /// AM modulation depth display register
    pub fn am_mod_depth_disp(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 37)
    }
    /// RFO AM modulated level definition register
    pub fn rfo_am_mod_level_def(&mut self) -> Reg<'_, I, RfoAmLevelDef> {
        Reg::new(self.iface, 38)
    }
    /// RFO normal level definition register
    pub fn rfo_normal_level_def(&mut self) -> Reg<'_, I, RfoAmLevelDef> {
        Reg::new(self.iface, 39)
    }
    /// External field detector threshold register
    pub fn ext_field_det_thr(&mut self) -> Reg<'_, I, ExtFieldDetThr> {
        Reg::new(self.iface, 41)
    }
    /// Regulator voltage control register
    pub fn regulator_volt_control(&mut self) -> Reg<'_, I, RegulatorVoltControl> {
        Reg::new(self.iface, 42)
    }
    /// Regulator and timer display register
    pub fn regulator_and_tim_disp(&mut self) -> Reg<'_, I, RegulatorAndTimDisp> {
        Reg::new(self.iface, 43)
    }
    /// RSSI display register
    pub fn rssi_result(&mut self) -> Reg<'_, I, RssiResult> {
        Reg::new(self.iface, 44)
    }
    /// Gain reduction state register
    pub fn gain_redu_state(&mut self) -> Reg<'_, I, GainReduState> {
        Reg::new(self.iface, 45)
    }
    /// Capacitive sensor control register
    pub fn cap_sensor_control(&mut self) -> Reg<'_, I, CapSensorControl> {
        Reg::new(self.iface, 46)
    }
    /// Capacitive sensor display register
    pub fn cap_sensor_result(&mut self) -> Reg<'_, I, CapSensorDisp> {
        Reg::new(self.iface, 47)
    }
    /// Auxiliary display register
    pub fn aux_display(&mut self) -> Reg<'_, I, AuxDisplay> {
        Reg::new(self.iface, 48)
    }
    /// Wake-up timer control register
    pub fn wup_timer_control(&mut self) -> Reg<'_, I, WupTimerControl> {
        Reg::new(self.iface, 49)
    }
    /// Amplitude measurement configuration register
    pub fn amplitude_measure_conf(&mut self) -> Reg<'_, I, AmplitudeMeasureConf> {
        Reg::new(self.iface, 50)
    }
    /// Amplitude measurement reference register
    pub fn amplitude_measure_ref(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 51)
    }
    /// Amplitude measurement auto-averaging display
    pub fn amplitude_measure_auto_avg_disp(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 52)
    }
    /// Amplitude measurement display
    pub fn amplitude_measure_disp(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 53)
    }
    /// Phase measurement configuration
    pub fn phase_measure_conf(&mut self) -> Reg<'_, I, PhaseMeasureConf> {
        Reg::new(self.iface, 54)
    }
    /// Phase measurement reference
    pub fn phase_measure_ref(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 55)
    }
    /// Phase measurement auto-averaging display
    pub fn phase_measure_auto_avg_disp(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 56)
    }
    /// Phase measurement display
    pub fn phase_measure_disp(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 57)
    }
    /// Capacitance measurement configuration
    pub fn capacitance_measure_conf(&mut self) -> Reg<'_, I, CapacitanceMeasureConf> {
        Reg::new(self.iface, 58)
    }
    /// Capacitance measurement reference
    pub fn capacitance_measure_ref(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 59)
    }
    /// Capacitance measurement auto-averaging display
    pub fn capacitance_measure_auto_avg_disp(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 60)
    }
    /// Capacitance measurement display
    pub fn capacitance_measure_disp(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 61)
    }
    /// IC identity register
    pub fn ic_identity(&mut self) -> Reg<'_, I, IcIdentity> {
        Reg::new(self.iface, 63)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AmplitudeMeasureConf(pub u8);
impl AmplitudeMeasureConf {
    pub const fn am_ae(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_am_ae(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn am_aew(&self) -> u8 {
        let val = (self.0 >> 1_usize) & 2;
        val as u8
    }
    pub fn set_am_aew(&mut self, val: u8) {
        self.0 = (self.0 & !(2 << 1_usize)) | ((val as u8) & 2) << 1_usize;
    }
    pub const fn am_aam(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_am_aam(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn am_d(&self) -> u8 {
        let val = (self.0 >> 4_usize) & 4;
        val as u8
    }
    pub fn set_am_d(&mut self, val: u8) {
        self.0 = (self.0 & !(4 << 4_usize)) | ((val as u8) & 4) << 4_usize;
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
pub struct AmModDepthCtrl(pub u8);
impl AmModDepthCtrl {
    pub const fn modd(&self) -> u8 {
        let val = (self.0 >> 1_usize) & 6;
        val as u8
    }
    pub fn set_modd(&mut self, val: u8) {
        self.0 = (self.0 & !(6 << 1_usize)) | ((val as u8) & 6) << 1_usize;
    }
    pub const fn am_s(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_am_s(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
    }
}
impl Default for AmModDepthCtrl {
    fn default() -> AmModDepthCtrl {
        AmModDepthCtrl(0)
    }
}
impl From<u8> for AmModDepthCtrl {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<AmModDepthCtrl> for u8 {
    fn from(val: AmModDepthCtrl) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AntTuneCtrl(pub u8);
impl AntTuneCtrl {
    pub const fn tre(&self) -> u8 {
        let val = (self.0 >> 3_usize) & 4;
        val as u8
    }
    pub fn set_tre(&mut self, val: u8) {
        self.0 = (self.0 & !(4 << 3_usize)) | ((val as u8) & 4) << 3_usize;
    }
    pub const fn trim_s(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_trim_s(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
    }
}
impl Default for AntTuneCtrl {
    fn default() -> AntTuneCtrl {
        AntTuneCtrl(0)
    }
}
impl From<u8> for AntTuneCtrl {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<AntTuneCtrl> for u8 {
    fn from(val: AntTuneCtrl) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AntTuneDisp(pub u8);
impl AntTuneDisp {
    pub const fn tri_err(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub const fn tri(&self) -> u8 {
        let val = (self.0 >> 4_usize) & 4;
        val as u8
    }
}
impl Default for AntTuneDisp {
    fn default() -> AntTuneDisp {
        AntTuneDisp(0)
    }
}
impl From<u8> for AntTuneDisp {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<AntTuneDisp> for u8 {
    fn from(val: AntTuneDisp) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Aux(pub u8);
impl Aux {
    pub const fn nfc_n(&self) -> u8 {
        let val = (self.0 >> 0_usize) & 2;
        val as u8
    }
    pub fn set_nfc_n(&mut self, val: u8) {
        self.0 = (self.0 & !(2 << 0_usize)) | ((val as u8) & 2) << 0_usize;
    }
    pub const fn rx_tol(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_rx_tol(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn ook_hr(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_ook_hr(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn en_fd(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub fn set_en_fd(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 4_usize)) | ((val as u8) & 1) << 4_usize;
    }
    pub const fn tr_am(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub fn set_tr_am(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 5_usize)) | ((val as u8) & 1) << 5_usize;
    }
    pub const fn crc_2_fifo(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub fn set_crc_2_fifo(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 6_usize)) | ((val as u8) & 1) << 6_usize;
    }
    pub const fn no_crc_rx(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_no_crc_rx(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
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
pub struct AuxDisplay(pub u8);
impl AuxDisplay {
    pub const fn en_ac(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub const fn nfc_t(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub const fn rx_act(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub const fn rx_on(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub const fn osc_ok(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub const fn tx_on(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub const fn efd_o(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub const fn a_cha(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
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
pub struct BitRate(pub u8);
impl BitRate {
    pub const fn rxrate(&self) -> BitRateE {
        let val = (self.0 >> 0_usize) & 4;
        BitRateE(val as u8)
    }
    pub fn set_rxrate(&mut self, val: BitRateE) {
        self.0 = (self.0 & !(4 << 0_usize)) | ((val.0 as u8) & 4) << 0_usize;
    }
    pub const fn txrate(&self) -> BitRateE {
        let val = (self.0 >> 4_usize) & 4;
        BitRateE(val as u8)
    }
    pub fn set_txrate(&mut self, val: BitRateE) {
        self.0 = (self.0 & !(4 << 4_usize)) | ((val.0 as u8) & 4) << 4_usize;
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
pub struct CapacitanceMeasureConf(pub u8);
impl CapacitanceMeasureConf {
    pub const fn cm_ae(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_cm_ae(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn cm_aew(&self) -> u8 {
        let val = (self.0 >> 1_usize) & 2;
        val as u8
    }
    pub fn set_cm_aew(&mut self, val: u8) {
        self.0 = (self.0 & !(2 << 1_usize)) | ((val as u8) & 2) << 1_usize;
    }
    pub const fn cm_aam(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_cm_aam(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn cm_d(&self) -> u8 {
        let val = (self.0 >> 4_usize) & 4;
        val as u8
    }
    pub fn set_cm_d(&mut self, val: u8) {
        self.0 = (self.0 & !(4 << 4_usize)) | ((val as u8) & 4) << 4_usize;
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
pub struct CapSensorControl(pub u8);
impl CapSensorControl {
    pub const fn cs_g(&self) -> u8 {
        let val = (self.0 >> 0_usize) & 3;
        val as u8
    }
    pub fn set_cs_g(&mut self, val: u8) {
        self.0 = (self.0 & !(3 << 0_usize)) | ((val as u8) & 3) << 0_usize;
    }
    pub const fn cs_mcal(&self) -> u8 {
        let val = (self.0 >> 3_usize) & 5;
        val as u8
    }
    pub fn set_cs_mcal(&mut self, val: u8) {
        self.0 = (self.0 & !(5 << 3_usize)) | ((val as u8) & 5) << 3_usize;
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
pub struct CapSensorDisp(pub u8);
impl CapSensorDisp {
    pub const fn cs_cal_err(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub const fn cs_cal_end(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub const fn cs_cal_val(&self) -> u8 {
        let val = (self.0 >> 3_usize) & 5;
        val as u8
    }
}
impl Default for CapSensorDisp {
    fn default() -> CapSensorDisp {
        CapSensorDisp(0)
    }
}
impl From<u8> for CapSensorDisp {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<CapSensorDisp> for u8 {
    fn from(val: CapSensorDisp) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CollisionStatus(pub u8);
impl CollisionStatus {
    pub const fn c_pb(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub const fn c_bit(&self) -> u8 {
        let val = (self.0 >> 1_usize) & 3;
        val as u8
    }
    pub const fn c_byte(&self) -> u8 {
        let val = (self.0 >> 4_usize) & 4;
        val as u8
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
pub struct ExtFieldDetThr(pub u8);
impl ExtFieldDetThr {
    pub const fn rfe_t(&self) -> ThresholdDef2 {
        let val = (self.0 >> 0_usize) & 4;
        ThresholdDef2(val as u8)
    }
    pub fn set_rfe_t(&mut self, val: ThresholdDef2) {
        self.0 = (self.0 & !(4 << 0_usize)) | ((val.0 as u8) & 4) << 0_usize;
    }
    pub const fn trg_l(&self) -> ThresholdDef1 {
        let val = (self.0 >> 4_usize) & 3;
        ThresholdDef1(val as u8)
    }
    pub fn set_trg_l(&mut self, val: ThresholdDef1) {
        self.0 = (self.0 & !(3 << 4_usize)) | ((val.0 as u8) & 3) << 4_usize;
    }
}
impl Default for ExtFieldDetThr {
    fn default() -> ExtFieldDetThr {
        ExtFieldDetThr(0)
    }
}
impl From<u8> for ExtFieldDetThr {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<ExtFieldDetThr> for u8 {
    fn from(val: ExtFieldDetThr) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FifoStatus2(pub u8);
impl FifoStatus2 {
    pub const fn np_lb(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub const fn fifo_lb(&self) -> u8 {
        let val = (self.0 >> 1_usize) & 3;
        val as u8
    }
    pub const fn fifo_ovr(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub const fn fifo_unf(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
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
pub struct GainReduState(pub u8);
impl GainReduState {
    pub const fn gs_pm(&self) -> u8 {
        let val = (self.0 >> 0_usize) & 4;
        val as u8
    }
    pub const fn gs_am(&self) -> u8 {
        let val = (self.0 >> 4_usize) & 4;
        val as u8
    }
}
impl Default for GainReduState {
    fn default() -> GainReduState {
        GainReduState(0)
    }
}
impl From<u8> for GainReduState {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<GainReduState> for u8 {
    fn from(val: GainReduState) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct GptNrtCtrl(pub u8);
impl GptNrtCtrl {
    pub const fn nrt_step(&self) -> TimerEmvControlNrtStep {
        let val = (self.0 >> 0_usize) & 1;
        TimerEmvControlNrtStep(val as u8)
    }
    pub fn set_nrt_step(&mut self, val: TimerEmvControlNrtStep) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val.0 as u8) & 1) << 0_usize;
    }
    pub const fn nrt_emv(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub fn set_nrt_emv(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 1_usize)) | ((val as u8) & 1) << 1_usize;
    }
    pub const fn gptc(&self) -> TimerEmvControlGptc {
        let val = (self.0 >> 5_usize) & 3;
        TimerEmvControlGptc(val as u8)
    }
    pub fn set_gptc(&mut self, val: TimerEmvControlGptc) {
        self.0 = (self.0 & !(3 << 5_usize)) | ((val.0 as u8) & 3) << 5_usize;
    }
}
impl Default for GptNrtCtrl {
    fn default() -> GptNrtCtrl {
        GptNrtCtrl(0)
    }
}
impl From<u8> for GptNrtCtrl {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<GptNrtCtrl> for u8 {
    fn from(val: GptNrtCtrl) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct IcIdentity(pub u8);
impl IcIdentity {
    pub const fn ic_rev(&self) -> IcIdentityIcRev {
        let val = (self.0 >> 0_usize) & 3;
        IcIdentityIcRev(val as u8)
    }
    pub const fn ic_type(&self) -> IcIdentityIcType {
        let val = (self.0 >> 3_usize) & 5;
        IcIdentityIcType(val as u8)
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
pub struct IoConf1(pub u8);
impl IoConf1 {
    pub const fn lf_clk_off(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_lf_clk_off(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn out_cl(&self) -> IoConf1OutCl {
        let val = (self.0 >> 1_usize) & 2;
        IoConf1OutCl(val as u8)
    }
    pub fn set_out_cl(&mut self, val: IoConf1OutCl) {
        self.0 = (self.0 & !(2 << 1_usize)) | ((val.0 as u8) & 2) << 1_usize;
    }
    pub const fn osc(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_osc(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn fifo_lt(&self) -> u8 {
        let val = (self.0 >> 4_usize) & 2;
        val as u8
    }
    pub fn set_fifo_lt(&mut self, val: u8) {
        self.0 = (self.0 & !(2 << 4_usize)) | ((val as u8) & 2) << 4_usize;
    }
    pub const fn rfo2(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub fn set_rfo2(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 6_usize)) | ((val as u8) & 1) << 6_usize;
    }
    pub const fn single(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_single(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
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
pub struct IoConf2(pub u8);
impl IoConf2 {
    pub const fn slow_up(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_slow_up(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn io_18(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_io_18(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn miso_pd1(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_miso_pd1(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn miso_pd2(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub fn set_miso_pd2(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 4_usize)) | ((val as u8) & 1) << 4_usize;
    }
    pub const fn vspd_off(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub fn set_vspd_off(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 6_usize)) | ((val as u8) & 1) << 6_usize;
    }
    pub const fn sup_3v(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_sup_3v(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
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
pub struct IrqMain(pub u8);
impl IrqMain {
    pub const fn err(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub const fn tim(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub const fn col(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub const fn txe(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub const fn rxe(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub const fn rxs(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub const fn wl(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub const fn osc(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
}
impl Default for IrqMain {
    fn default() -> IrqMain {
        IrqMain(0)
    }
}
impl From<u8> for IrqMain {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<IrqMain> for u8 {
    fn from(val: IrqMain) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct IrqMaskErrorWup(pub u8);
impl IrqMaskErrorWup {
    pub const fn m_ncap(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_m_ncap(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn m_wph(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub fn set_m_wph(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 1_usize)) | ((val as u8) & 1) << 1_usize;
    }
    pub const fn m_wam(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_m_wam(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn m_wt(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_m_wt(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn m_err1(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub fn set_m_err1(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 4_usize)) | ((val as u8) & 1) << 4_usize;
    }
    pub const fn m_err2(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub fn set_m_err2(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 5_usize)) | ((val as u8) & 1) << 5_usize;
    }
    pub const fn m_par(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub fn set_m_par(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 6_usize)) | ((val as u8) & 1) << 6_usize;
    }
    pub const fn m_crc(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_m_crc(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
    }
}
impl Default for IrqMaskErrorWup {
    fn default() -> IrqMaskErrorWup {
        IrqMaskErrorWup(0)
    }
}
impl From<u8> for IrqMaskErrorWup {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<IrqMaskErrorWup> for u8 {
    fn from(val: IrqMaskErrorWup) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct IrqMaskMain(pub u8);
impl IrqMaskMain {
    pub const fn m_col(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_m_col(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn m_txe(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_m_txe(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn m_rxe(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub fn set_m_rxe(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 4_usize)) | ((val as u8) & 1) << 4_usize;
    }
    pub const fn m_rxs(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub fn set_m_rxs(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 5_usize)) | ((val as u8) & 1) << 5_usize;
    }
    pub const fn m_wl(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub fn set_m_wl(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 6_usize)) | ((val as u8) & 1) << 6_usize;
    }
    pub const fn m_osc(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_m_osc(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
    }
}
impl Default for IrqMaskMain {
    fn default() -> IrqMaskMain {
        IrqMaskMain(0)
    }
}
impl From<u8> for IrqMaskMain {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<IrqMaskMain> for u8 {
    fn from(val: IrqMaskMain) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct IrqMaskTimerNfc(pub u8);
impl IrqMaskTimerNfc {
    pub const fn m_nfct(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_m_nfct(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn m_cat(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub fn set_m_cat(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 1_usize)) | ((val as u8) & 1) << 1_usize;
    }
    pub const fn m_cac(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_m_cac(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn m_eof(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_m_eof(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn m_eon(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub fn set_m_eon(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 4_usize)) | ((val as u8) & 1) << 4_usize;
    }
    pub const fn m_gpe(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub fn set_m_gpe(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 5_usize)) | ((val as u8) & 1) << 5_usize;
    }
    pub const fn m_nre(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub fn set_m_nre(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 6_usize)) | ((val as u8) & 1) << 6_usize;
    }
    pub const fn m_dcd(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_m_dcd(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
    }
}
impl Default for IrqMaskTimerNfc {
    fn default() -> IrqMaskTimerNfc {
        IrqMaskTimerNfc(0)
    }
}
impl From<u8> for IrqMaskTimerNfc {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<IrqMaskTimerNfc> for u8 {
    fn from(val: IrqMaskTimerNfc) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct IrqTimerNfc(pub u8);
impl IrqTimerNfc {
    pub const fn nfct(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub const fn cat(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub const fn cac(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub const fn eof(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub const fn eon(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub const fn gpe(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub const fn nre(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub const fn dcd(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
}
impl Default for IrqTimerNfc {
    fn default() -> IrqTimerNfc {
        IrqTimerNfc(0)
    }
}
impl From<u8> for IrqTimerNfc {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<IrqTimerNfc> for u8 {
    fn from(val: IrqTimerNfc) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Iso14443ANfc(pub u8);
impl Iso14443ANfc {
    pub const fn antcl(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_antcl(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn p_len(&self) -> u8 {
        let val = (self.0 >> 1_usize) & 4;
        val as u8
    }
    pub fn set_p_len(&mut self, val: u8) {
        self.0 = (self.0 & !(4 << 1_usize)) | ((val as u8) & 4) << 1_usize;
    }
    pub const fn nfc_f0(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub fn set_nfc_f0(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 5_usize)) | ((val as u8) & 1) << 5_usize;
    }
    pub const fn no_rx_par(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub fn set_no_rx_par(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 6_usize)) | ((val as u8) & 1) << 6_usize;
    }
    pub const fn no_tx_par(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_no_tx_par(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
    }
}
impl Default for Iso14443ANfc {
    fn default() -> Iso14443ANfc {
        Iso14443ANfc(0)
    }
}
impl From<u8> for Iso14443ANfc {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Iso14443ANfc> for u8 {
    fn from(val: Iso14443ANfc) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Iso14443B1(pub u8);
impl Iso14443B1 {
    pub const fn rx_st_om(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_rx_st_om(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn half(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub fn set_half(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 1_usize)) | ((val as u8) & 1) << 1_usize;
    }
    pub const fn eof(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_eof(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn sof_1(&self) -> Iso14443B1Sof1 {
        let val = (self.0 >> 3_usize) & 1;
        Iso14443B1Sof1(val as u8)
    }
    pub fn set_sof_1(&mut self, val: Iso14443B1Sof1) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val.0 as u8) & 1) << 3_usize;
    }
    pub const fn sof_0(&self) -> Iso14443B1Sof0 {
        let val = (self.0 >> 4_usize) & 1;
        Iso14443B1Sof0(val as u8)
    }
    pub fn set_sof_0(&mut self, val: Iso14443B1Sof0) {
        self.0 = (self.0 & !(1 << 4_usize)) | ((val.0 as u8) & 1) << 4_usize;
    }
    pub const fn egt(&self) -> u8 {
        let val = (self.0 >> 5_usize) & 3;
        val as u8
    }
    pub fn set_egt(&mut self, val: u8) {
        self.0 = (self.0 & !(3 << 5_usize)) | ((val as u8) & 3) << 5_usize;
    }
}
impl Default for Iso14443B1 {
    fn default() -> Iso14443B1 {
        Iso14443B1(0)
    }
}
impl From<u8> for Iso14443B1 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Iso14443B1> for u8 {
    fn from(val: Iso14443B1) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Iso14443B2(pub u8);
impl Iso14443B2 {
    pub const fn f_p(&self) -> Iso14443B2FP {
        let val = (self.0 >> 0_usize) & 2;
        Iso14443B2FP(val as u8)
    }
    pub fn set_f_p(&mut self, val: Iso14443B2FP) {
        self.0 = (self.0 & !(2 << 0_usize)) | ((val.0 as u8) & 2) << 0_usize;
    }
    pub const fn phc_th(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_phc_th(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn eof_12(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_eof_12(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn no_eof(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub fn set_no_eof(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 4_usize)) | ((val as u8) & 1) << 4_usize;
    }
    pub const fn no_sof(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub fn set_no_sof(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 5_usize)) | ((val as u8) & 1) << 5_usize;
    }
    pub const fn tr1(&self) -> Iso14443B2Tr1 {
        let val = (self.0 >> 6_usize) & 2;
        Iso14443B2Tr1(val as u8)
    }
    pub fn set_tr1(&mut self, val: Iso14443B2Tr1) {
        self.0 = (self.0 & !(2 << 6_usize)) | ((val.0 as u8) & 2) << 6_usize;
    }
}
impl Default for Iso14443B2 {
    fn default() -> Iso14443B2 {
        Iso14443B2(0)
    }
}
impl From<u8> for Iso14443B2 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Iso14443B2> for u8 {
    fn from(val: Iso14443B2) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MaskRxTimer(pub u8);
impl MaskRxTimer {
    pub const fn mrt0(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_mrt0(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn mrt1(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub fn set_mrt1(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 1_usize)) | ((val as u8) & 1) << 1_usize;
    }
    pub const fn mrt2(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_mrt2(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn mrt3(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_mrt3(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn mrt4(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub fn set_mrt4(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 4_usize)) | ((val as u8) & 1) << 4_usize;
    }
    pub const fn mrt5(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub fn set_mrt5(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 5_usize)) | ((val as u8) & 1) << 5_usize;
    }
    pub const fn mrt6(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub fn set_mrt6(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 6_usize)) | ((val as u8) & 1) << 6_usize;
    }
    pub const fn mrt7(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_mrt7(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
    }
}
impl Default for MaskRxTimer {
    fn default() -> MaskRxTimer {
        MaskRxTimer(0)
    }
}
impl From<u8> for MaskRxTimer {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<MaskRxTimer> for u8 {
    fn from(val: MaskRxTimer) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Mode(pub u8);
impl Mode {
    pub const fn nfc_ar(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_nfc_ar(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn om(&self) -> ModeOm {
        let val = (self.0 >> 3_usize) & 4;
        ModeOm(val as u8)
    }
    pub fn set_om(&mut self, val: ModeOm) {
        self.0 = (self.0 & !(4 << 3_usize)) | ((val.0 as u8) & 4) << 3_usize;
    }
    pub const fn targ(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_targ(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
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
pub struct Nfcip1BitRateDisp(pub u8);
impl Nfcip1BitRateDisp {
    pub const fn nfc_rate(&self) -> u8 {
        let val = (self.0 >> 4_usize) & 4;
        val as u8
    }
}
impl Default for Nfcip1BitRateDisp {
    fn default() -> Nfcip1BitRateDisp {
        Nfcip1BitRateDisp(0)
    }
}
impl From<u8> for Nfcip1BitRateDisp {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Nfcip1BitRateDisp> for u8 {
    fn from(val: Nfcip1BitRateDisp) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NoResponseTimer1(pub u8);
impl NoResponseTimer1 {
    pub const fn nrt8(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_nrt8(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn nrt9(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub fn set_nrt9(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 1_usize)) | ((val as u8) & 1) << 1_usize;
    }
    pub const fn nrt10(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_nrt10(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn nrt11(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_nrt11(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn nrt12(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub fn set_nrt12(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 4_usize)) | ((val as u8) & 1) << 4_usize;
    }
    pub const fn nrt13(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub fn set_nrt13(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 5_usize)) | ((val as u8) & 1) << 5_usize;
    }
    pub const fn nrt14(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub fn set_nrt14(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 6_usize)) | ((val as u8) & 1) << 6_usize;
    }
    pub const fn nrt15(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_nrt15(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
    }
}
impl Default for NoResponseTimer1 {
    fn default() -> NoResponseTimer1 {
        NoResponseTimer1(0)
    }
}
impl From<u8> for NoResponseTimer1 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<NoResponseTimer1> for u8 {
    fn from(val: NoResponseTimer1) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NumTxBytes2(pub u8);
impl NumTxBytes2 {
    pub const fn nbtx(&self) -> u8 {
        let val = (self.0 >> 0_usize) & 3;
        val as u8
    }
    pub fn set_nbtx(&mut self, val: u8) {
        self.0 = (self.0 & !(3 << 0_usize)) | ((val as u8) & 3) << 0_usize;
    }
    pub const fn ntx(&self) -> u8 {
        let val = (self.0 >> 3_usize) & 5;
        val as u8
    }
    pub fn set_ntx(&mut self, val: u8) {
        self.0 = (self.0 & !(5 << 3_usize)) | ((val as u8) & 5) << 3_usize;
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
pub struct OpControl(pub u8);
impl OpControl {
    pub const fn wu(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_wu(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn tx_en(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_tx_en(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn rx_man(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub fn set_rx_man(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 4_usize)) | ((val as u8) & 1) << 4_usize;
    }
    pub const fn rx_chn(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub fn set_rx_chn(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 5_usize)) | ((val as u8) & 1) << 5_usize;
    }
    pub const fn rx_en(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub fn set_rx_en(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 6_usize)) | ((val as u8) & 1) << 6_usize;
    }
    pub const fn en(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_en(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
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
pub struct PhaseMeasureConf(pub u8);
impl PhaseMeasureConf {
    pub const fn pm_ae(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_pm_ae(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn pm_aew(&self) -> u8 {
        let val = (self.0 >> 1_usize) & 2;
        val as u8
    }
    pub fn set_pm_aew(&mut self, val: u8) {
        self.0 = (self.0 & !(2 << 1_usize)) | ((val as u8) & 2) << 1_usize;
    }
    pub const fn pm_aam(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_pm_aam(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn pm_d(&self) -> u8 {
        let val = (self.0 >> 4_usize) & 4;
        val as u8
    }
    pub fn set_pm_d(&mut self, val: u8) {
        self.0 = (self.0 & !(4 << 4_usize)) | ((val as u8) & 4) << 4_usize;
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
pub struct RegulatorAndTimDisp(pub u8);
impl RegulatorAndTimDisp {
    pub const fn mrt_on(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub const fn nrt_on(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub const fn gpt_on(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub const fn reg(&self) -> u8 {
        let val = (self.0 >> 4_usize) & 4;
        val as u8
    }
}
impl Default for RegulatorAndTimDisp {
    fn default() -> RegulatorAndTimDisp {
        RegulatorAndTimDisp(0)
    }
}
impl From<u8> for RegulatorAndTimDisp {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<RegulatorAndTimDisp> for u8 {
    fn from(val: RegulatorAndTimDisp) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RegulatorVoltControl(pub u8);
impl RegulatorVoltControl {
    pub const fn mpsv(&self) -> u8 {
        let val = (self.0 >> 1_usize) & 2;
        val as u8
    }
    pub fn set_mpsv(&mut self, val: u8) {
        self.0 = (self.0 & !(2 << 1_usize)) | ((val as u8) & 2) << 1_usize;
    }
    pub const fn rege(&self) -> u8 {
        let val = (self.0 >> 3_usize) & 4;
        val as u8
    }
    pub fn set_rege(&mut self, val: u8) {
        self.0 = (self.0 & !(4 << 3_usize)) | ((val as u8) & 4) << 3_usize;
    }
    pub const fn reg_s(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_reg_s(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
    }
}
impl Default for RegulatorVoltControl {
    fn default() -> RegulatorVoltControl {
        RegulatorVoltControl(0)
    }
}
impl From<u8> for RegulatorVoltControl {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<RegulatorVoltControl> for u8 {
    fn from(val: RegulatorVoltControl) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RfoAmLevelDef(pub u8);
impl RfoAmLevelDef {
    pub const fn d0(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_d0(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn d1(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub fn set_d1(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 1_usize)) | ((val as u8) & 1) << 1_usize;
    }
    pub const fn d2(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_d2(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn d3(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_d3(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn d4(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub fn set_d4(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 4_usize)) | ((val as u8) & 1) << 4_usize;
    }
    pub const fn d5(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub fn set_d5(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 5_usize)) | ((val as u8) & 1) << 5_usize;
    }
    pub const fn d6(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub fn set_d6(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 6_usize)) | ((val as u8) & 1) << 6_usize;
    }
    pub const fn d7(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_d7(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
    }
}
impl Default for RfoAmLevelDef {
    fn default() -> RfoAmLevelDef {
        RfoAmLevelDef(0)
    }
}
impl From<u8> for RfoAmLevelDef {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<RfoAmLevelDef> for u8 {
    fn from(val: RfoAmLevelDef) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RssiResult(pub u8);
impl RssiResult {
    pub const fn rssi_pm(&self) -> u8 {
        let val = (self.0 >> 0_usize) & 4;
        val as u8
    }
    pub const fn rssi_am(&self) -> u8 {
        let val = (self.0 >> 4_usize) & 4;
        val as u8
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
pub struct RxConf1(pub u8);
impl RxConf1 {
    pub const fn z12k(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_z12k(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn h80(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub fn set_h80(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 1_usize)) | ((val as u8) & 1) << 1_usize;
    }
    pub const fn h200(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_h200(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn lp(&self) -> RxConf1Lp {
        let val = (self.0 >> 3_usize) & 3;
        RxConf1Lp(val as u8)
    }
    pub fn set_lp(&mut self, val: RxConf1Lp) {
        self.0 = (self.0 & !(3 << 3_usize)) | ((val.0 as u8) & 3) << 3_usize;
    }
    pub const fn amd_sel(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub fn set_amd_sel(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 6_usize)) | ((val as u8) & 1) << 6_usize;
    }
    pub const fn ch_sel(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_ch_sel(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
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
pub struct RxConf2(pub u8);
impl RxConf2 {
    pub const fn pmix_cl(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_pmix_cl(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn sqm_dyn(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub fn set_sqm_dyn(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 1_usize)) | ((val as u8) & 1) << 1_usize;
    }
    pub const fn agc_alg(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_agc_alg(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn agc_m(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_agc_m(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn agc_en(&self) -> bool {
        let val = (self.0 >> 4_usize) & 1;
        val != 0
    }
    pub fn set_agc_en(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 4_usize)) | ((val as u8) & 1) << 4_usize;
    }
    pub const fn lf_en(&self) -> bool {
        let val = (self.0 >> 5_usize) & 1;
        val != 0
    }
    pub fn set_lf_en(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 5_usize)) | ((val as u8) & 1) << 5_usize;
    }
    pub const fn lf_op(&self) -> bool {
        let val = (self.0 >> 6_usize) & 1;
        val != 0
    }
    pub fn set_lf_op(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 6_usize)) | ((val as u8) & 1) << 6_usize;
    }
    pub const fn rx_lp(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_rx_lp(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
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
pub struct RxConf3(pub u8);
impl RxConf3 {
    pub const fn rg_nfc(&self) -> bool {
        let val = (self.0 >> 0_usize) & 1;
        val != 0
    }
    pub fn set_rg_nfc(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 0_usize)) | ((val as u8) & 1) << 0_usize;
    }
    pub const fn lim(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub fn set_lim(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 1_usize)) | ((val as u8) & 1) << 1_usize;
    }
    pub const fn rg1_pm(&self) -> u8 {
        let val = (self.0 >> 2_usize) & 3;
        val as u8
    }
    pub fn set_rg1_pm(&mut self, val: u8) {
        self.0 = (self.0 & !(3 << 2_usize)) | ((val as u8) & 3) << 2_usize;
    }
    pub const fn rg1_am(&self) -> u8 {
        let val = (self.0 >> 5_usize) & 3;
        val as u8
    }
    pub fn set_rg1_am(&mut self, val: u8) {
        self.0 = (self.0 & !(3 << 5_usize)) | ((val as u8) & 3) << 5_usize;
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
pub struct RxConf4(pub u8);
impl RxConf4 {
    pub const fn rg2_pm(&self) -> u8 {
        let val = (self.0 >> 0_usize) & 4;
        val as u8
    }
    pub fn set_rg2_pm(&mut self, val: u8) {
        self.0 = (self.0 & !(4 << 0_usize)) | ((val as u8) & 4) << 0_usize;
    }
    pub const fn rg2_am(&self) -> u8 {
        let val = (self.0 >> 4_usize) & 4;
        val as u8
    }
    pub fn set_rg2_am(&mut self, val: u8) {
        self.0 = (self.0 & !(4 << 4_usize)) | ((val as u8) & 4) << 4_usize;
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
pub struct StreamMode(pub u8);
impl StreamMode {
    pub const fn stx(&self) -> StreamModeStx {
        let val = (self.0 >> 0_usize) & 3;
        StreamModeStx(val as u8)
    }
    pub fn set_stx(&mut self, val: StreamModeStx) {
        self.0 = (self.0 & !(3 << 0_usize)) | ((val.0 as u8) & 3) << 0_usize;
    }
    pub const fn scp(&self) -> StreamModeScp {
        let val = (self.0 >> 3_usize) & 2;
        StreamModeScp(val as u8)
    }
    pub fn set_scp(&mut self, val: StreamModeScp) {
        self.0 = (self.0 & !(2 << 3_usize)) | ((val.0 as u8) & 2) << 3_usize;
    }
    pub const fn scf(&self) -> StreamModeScf {
        let val = (self.0 >> 5_usize) & 2;
        StreamModeScf(val as u8)
    }
    pub fn set_scf(&mut self, val: StreamModeScf) {
        self.0 = (self.0 & !(2 << 5_usize)) | ((val.0 as u8) & 2) << 5_usize;
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
pub struct WupTimerControl(pub u8);
impl WupTimerControl {
    pub const fn wph(&self) -> bool {
        let val = (self.0 >> 1_usize) & 1;
        val != 0
    }
    pub fn set_wph(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 1_usize)) | ((val as u8) & 1) << 1_usize;
    }
    pub const fn wam(&self) -> bool {
        let val = (self.0 >> 2_usize) & 1;
        val != 0
    }
    pub fn set_wam(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 2_usize)) | ((val as u8) & 1) << 2_usize;
    }
    pub const fn wto(&self) -> bool {
        let val = (self.0 >> 3_usize) & 1;
        val != 0
    }
    pub fn set_wto(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 3_usize)) | ((val as u8) & 1) << 3_usize;
    }
    pub const fn wut(&self) -> u8 {
        let val = (self.0 >> 4_usize) & 3;
        val as u8
    }
    pub fn set_wut(&mut self, val: u8) {
        self.0 = (self.0 & !(3 << 4_usize)) | ((val as u8) & 3) << 4_usize;
    }
    pub const fn wur(&self) -> bool {
        let val = (self.0 >> 7_usize) & 1;
        val != 0
    }
    pub fn set_wur(&mut self, val: bool) {
        self.0 = (self.0 & !(1 << 7_usize)) | ((val as u8) & 1) << 7_usize;
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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct BitRateE(pub u8);
impl BitRateE {
    pub const _106: Self = Self(0x00);
    pub const _212: Self = Self(0x01);
    pub const _424: Self = Self(0x02);
    pub const _848: Self = Self(0x03);
    pub const _1695: Self = Self(0x04);
    pub const _3390: Self = Self(0x05);
    pub const _6780: Self = Self(0x06);
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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct IcIdentityIcRev(pub u8);
impl IcIdentityIcRev {
    pub const V0: Self = Self(0x00);
    pub const V3_1: Self = Self(0x02);
    pub const V3_3: Self = Self(0x03);
    pub const V4_0: Self = Self(0x04);
    pub const V4_1: Self = Self(0x05);
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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct IoConf1OutCl(pub u8);
impl IoConf1OutCl {
    pub const _3_39_MHZ: Self = Self(0x00);
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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Iso14443B1Sof0(pub u8);
impl Iso14443B1Sof0 {
    pub const _10ETU: Self = Self(0x00);
    pub const _11ETU: Self = Self(0x01);
}
impl From<u8> for Iso14443B1Sof0 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Iso14443B1Sof0> for u8 {
    fn from(val: Iso14443B1Sof0) -> u8 {
        val.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Iso14443B1Sof1(pub u8);
impl Iso14443B1Sof1 {
    pub const _2ETU: Self = Self(0x00);
    pub const _3ETU: Self = Self(0x01);
}
impl From<u8> for Iso14443B1Sof1 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Iso14443B1Sof1> for u8 {
    fn from(val: Iso14443B1Sof1) -> u8 {
        val.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Iso14443B2FP(pub u8);
impl Iso14443B2FP {
    pub const _48: Self = Self(0x00);
    pub const _64: Self = Self(0x01);
    pub const _80: Self = Self(0x02);
    pub const _96: Self = Self(0x03);
}
impl From<u8> for Iso14443B2FP {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Iso14443B2FP> for u8 {
    fn from(val: Iso14443B2FP) -> u8 {
        val.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Iso14443B2Tr1(pub u8);
impl Iso14443B2Tr1 {
    pub const _80FS80FS: Self = Self(0x00);
    pub const _64FS32FS: Self = Self(0x01);
}
impl From<u8> for Iso14443B2Tr1 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<Iso14443B2Tr1> for u8 {
    fn from(val: Iso14443B2Tr1) -> u8 {
        val.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ModeOm(pub u8);
impl ModeOm {
    pub const TARG_NFCIP1_ACTIVE_COMM_BR_DET: Self = Self(0x00);
    pub const TARG_NFCIP1_ACTIVE_COMM_NORMAL: Self = Self(0x00);
    pub const INI_ISO14443A: Self = Self(0x01);
    pub const TARG_NFCA: Self = Self(0x01);
    pub const INI_ISO14443B: Self = Self(0x02);
    pub const TARG_NFCB: Self = Self(0x02);
    pub const INI_FELICA: Self = Self(0x03);
    pub const INI_TOPAZ: Self = Self(0x04);
    pub const TARG_NFCF: Self = Self(0x04);
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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct RxConf1Lp(pub u8);
impl RxConf1Lp {
    pub const _1200KHZ: Self = Self(0x00);
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
pub struct StreamModeScf(pub u8);
impl StreamModeScf {
    pub const BPSK848: Self = Self(0x00);
    pub const SC212: Self = Self(0x00);
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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct StreamModeScp(pub u8);
impl StreamModeScp {
    pub const _1PULSE: Self = Self(0x00);
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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct StreamModeStx(pub u8);
impl StreamModeStx {
    pub const _106: Self = Self(0x00);
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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ThresholdDef1(pub u8);
impl ThresholdDef1 {
    pub const _75MV: Self = Self(0x00);
    pub const _105MV: Self = Self(0x01);
    pub const _150MV: Self = Self(0x02);
    pub const _205MV: Self = Self(0x03);
    pub const _290MV: Self = Self(0x04);
    pub const _400MV: Self = Self(0x05);
    pub const _560MV: Self = Self(0x06);
    pub const _800MV: Self = Self(0x07);
}
impl From<u8> for ThresholdDef1 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<ThresholdDef1> for u8 {
    fn from(val: ThresholdDef1) -> u8 {
        val.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ThresholdDef2(pub u8);
impl ThresholdDef2 {
    pub const _75MV: Self = Self(0x00);
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
impl From<u8> for ThresholdDef2 {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<ThresholdDef2> for u8 {
    fn from(val: ThresholdDef2) -> u8 {
        val.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct TimerEmvControlGptc(pub u8);
impl TimerEmvControlGptc {
    pub const NO_TRIGGER: Self = Self(0x00);
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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct TimerEmvControlNrtStep(pub u8);
impl TimerEmvControlNrtStep {
    pub const _64_FC: Self = Self(0x00);
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

/// Typical wake-up time, values for wur=1; multiply by 10 for wur=0
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct WakeupTimesDef(pub u8);
impl WakeupTimesDef {
    pub const _10: Self = Self(0x00);
    pub const _20: Self = Self(0x01);
    pub const _30: Self = Self(0x02);
    pub const _40: Self = Self(0x03);
    pub const _50: Self = Self(0x04);
    pub const _60: Self = Self(0x05);
    pub const _70: Self = Self(0x06);
    pub const _80: Self = Self(0x07);
}
impl From<u8> for WakeupTimesDef {
    fn from(val: u8) -> Self {
        Self(val)
    }
}
impl From<WakeupTimesDef> for u8 {
    fn from(val: WakeupTimesDef) -> u8 {
        val.0
    }
}
