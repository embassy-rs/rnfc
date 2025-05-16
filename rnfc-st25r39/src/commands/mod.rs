/// Direct commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(unused)]
pub enum Command {
    // #[cfg(feature = "st25r3916")]
    // Puts the chip in default state, st25r39{16,20} (same as after power-up)
    // SetDefault1 = 0xC0,
    /// Puts the chip in default state (same as after power-up)
    SetDefault = 0xC1,
    /// Stops all activities and clears FIFO same as C3
    Stop = 0xC2,
    /// Stops all activities and clears FIFO same as C2
    Clear = 0xC3,
    /// Transmit with CRC
    TransmitWithCrc = 0xC4,
    /// Transmit without CRC
    TransmitWithoutCrc = 0xC5,
    /// Transmit REQA
    TransmitReqa = 0xC6,
    /// Transmit WUPA
    TransmitWupa = 0xC7,
    /// NFC transmit with Initial RF Collision Avoidance
    InitialRfCollision = 0xC8,
    /// NFC transmit with Response RF Collision Avoidance
    ResponseRfCollisionN = 0xC9,
    #[cfg(feature = "st25r3911b")]
    /// NFC transmit with Response RF Collision Avoidance with n=0
    ResponseRfCollisionNzero = 0xCA,
    #[cfg(feature = "st25r3911b")]
    /// Accepted in NFCIP-1 active communication bitrate detection mode
    GotoNormalNFCMode = 0xCB,
    #[cfg(feature = "st25r3911b")]
    /// Presets Rx and Tx configuration based on state of Mode definition register and Bit rate definition register
    PresetAnalog = 0xCC,
    #[cfg(feature = "st25r3916")]
    /// Passive target logic to Sense/Idle state
    GotoSense = 0xCD,
    #[cfg(feature = "st25r3916")]
    /// Passive target logic to Sleep/Halt state
    GotoSleep = 0xCE,
    /// Mask receive data
    MaskReceiveData = 0xD0,
    /// Unmask receive data
    UnmaskReceiveData = 0xD1,
    #[cfg(feature = "st25r3916")]
    /// AM Modulation state change
    AmModStateChange = 0xD2,
    /// Measure singal amplitude on RFI inputs
    MeasureAmplitude = 0xD3,
    #[cfg(feature = "st25r3911b")]
    /// Performs gain reduction based on the current noise level
    Squelch = 0xD4,
    /// Reset RX Gain
    ResetRxgain = 0xD5,
    /// Adjust regulators
    AdjustRegulators = 0xD6,
    #[cfg(feature = "st25r3911b")]
    /// Starts sequence that activates the Tx, measures the modulation depth, and adapts it to comply with the specified modulation depth
    CalibrateModDepth = 0xD7,
    #[cfg(feature = "st25r3911b")]
    /// Calibrates antenna
    CalibrateAntenna = 0xD8,
    #[cfg(feature = "st25r3916")]
    /// Starts the sequence to adjust the driver timing.
    CalibrateDriverTiming = 0xD8,
    /// Measure phase between RFO and RFI signal
    MeasurePhase = 0xD9,
    /// Clear RSSI bits and restart the measurement
    ClearRssi = 0xDA,
    #[cfg(feature = "st25r3916")]
    /// Clears FIFO, Collision and IRQ status
    ClearFifo = 0xDB,
    /// Transparent mode  - amplitude of signal present on RFI inputs is measured, result is stored in A/D converter output register
    TransparentMode = 0xDC,
    /// Calibrate the capacitive sensor
    CalibrateCSensor = 0xDD,
    /// Measure capacitance sensor
    MeasureCapacitance = 0xDE,
    /// Measure power supply voltage
    MeasureVdd = 0xDF,
    /// Start the general purpose timer
    StartGpTimer = 0xE0,
    /// Start the wake-up timer
    StartWupTimer = 0xE1,
    /// Start the mask-receive timer
    StartMaskReceiveTimer = 0xE2,
    /// Start the no-response timer
    StartNoResponseTimer = 0xE3,
    #[cfg(feature = "st25r3916")]
    /// Start PPon2 timer
    StartPpon2Timer = 0xE4,
    #[cfg(feature = "st25r3916")]
    /// Stop No Response Timer
    StopNrt = 0xE8,
    #[cfg(feature = "st25r3916")]
    /// Enable R/W access to the test registers
    SpaceBAccess = 0xFB,
    /// Enable R/W access to the test registers
    TestAccess = 0xFC,
}
