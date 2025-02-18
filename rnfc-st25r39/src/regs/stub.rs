use super::{Reg, Regs};
use crate::Interface;

impl<'a, I: Interface> Regs<'a, I> {
    pub fn new(iface: &'a mut I) -> Self {
        Self { iface }
    }

    /// IO configuration register 1
    pub fn io_conf1(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 0)
    }
    pub fn aux(&mut self) -> Reg<'_, I, Aux> {
        Reg::new(self.iface, 1)
    }
    pub fn rx_conf2(&mut self) -> Reg<'_, I, RxConf2> {
        Reg::new(self.iface, 0)
    }
    pub fn num_tx_bytes2(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 0)
    }
    pub fn num_tx_bytes1(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 0)
    }
    pub fn fifo_status1(&mut self) -> Reg<'_, I, u8> {
        Reg::new(self.iface, 0)
    }
    pub fn fifo_status2(&mut self) -> Reg<'_, I, FifoStatus2> {
        Reg::new(self.iface, 0)
    }
    pub fn iso14443a_nfc(&mut self) -> Reg<'_, I, Iso14443ANfc> {
        Reg::new(self.iface, 0)
    }
    pub fn collision_status(&mut self) -> Reg<'_, I, CollisionStatus> {
        Reg::new(self.iface, 0)
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
