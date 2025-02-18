/// Direct commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(unused)]
pub enum Command {
    /// Puts the chip in default state (same as after power-up)
    SetDefault = 0xC1,
    /// Stops all activities and clears FIFO
    Stop = 0xC2,
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
    /// Passive target logic to Sense/Idle state
    GotoSense = 0xCD,
    /// Passive target logic to Sleep/Halt state
    GotoSleep = 0xCE,
    /// Mask receive data
    MaskReceiveData = 0xD0,
    /// Unmask receive data
    UnmaskReceiveData = 0xD1,
    /// AM Modulation state change
    AmModStateChange = 0xD2,
    /// Measure singal amplitude on RFI inputs
    MeasureAmplitude = 0xD3,
    /// Reset RX Gain
    ResetRxgain = 0xD5,
    /// Adjust regulators
    AdjustRegulators = 0xD6,
    /// Starts the sequence to adjust the driver timing
    CalibrateDriverTiming = 0xD8,
    /// Measure phase between RFO and RFI signal
    MeasurePhase = 0xD9,
    /// Clear RSSI bits and restart the measurement
    ClearRssi = 0xDA,
    /// Clears FIFO, Collision and IRQ status
    ClearFifo = 0xDB,
    /// Transparent mode
    TransparentMode = 0xDC,
    /// Calibrate the capacitive sensor
    CalibrateCSensor = 0xDD,
    /// Measure capacitance
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
    /// Start PPon2 timer
    StartPpon2Timer = 0xE4,
    /// Stop No Response Timer
    StopNrt = 0xE8,
    /// Enable R/W access to the test registers
    SpaceBAccess = 0xFB,
    /// Enable R/W access to the test registers
    TestAccess = 0xFC,
}
