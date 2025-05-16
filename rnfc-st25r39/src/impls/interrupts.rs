#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(unused)]
pub enum Interrupt {
    #[cfg(feature = "st25r3911b")]
    /// IRQ due to error and wake-up timer
    Err = 0,
    #[cfg(feature = "st25r3916")]
    /// RFU interrupt
    Rfu = 0,
    #[cfg(feature = "st25r3911b")]
    /// IRQ due to timer or NFC event
    Tim = 1,
    #[cfg(feature = "st25r3916")]
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
    #[cfg(feature = "st25r3916")]
    /// 106kb/s Passive target state interrupt: Active
    WuA = 24,
    #[cfg(feature = "st25r3916")]
    /// 106kb/s Passive target state interrupt: Active*
    WuAX = 25,
    #[cfg(feature = "st25r3916")]
    /// RFU2 interrupt
    Rfu2 = 26,
    #[cfg(feature = "st25r3916")]
    /// 212/424b/s Passive target interrupt: Active
    WuF = 27,
    #[cfg(feature = "st25r3916")]
    /// RXE with an automatic response interrupt
    RxePta = 28,
    #[cfg(feature = "st25r3916")]
    /// Anticollision done and Field On interrupt
    Apon = 29,
    #[cfg(feature = "st25r3916")]
    /// Passive target slot number water level interrupt
    SlWl = 30,
    #[cfg(feature = "st25r3916")]
    /// PPON2 Field on waiting Timer interrupt
    Ppon2 = 31,
}
