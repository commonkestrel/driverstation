#[derive(Debug, Clone)]
pub enum Entry {
    Controller,
    Module,
    Language(Language),
    CANPlugin,
    Accelerometer {
        channel: u8,
    },
    ADXL345(ADXL345),
    AnalogChannel {
        channel: u8,
    },
    AnalogTrigger {
        channel: u8,
    },
    AnalogTriggerOutput {
        index: u8,
        ty: Trigger,
    },
    CANJaguar,
    Compressor {
        pcm_id: u8,
    },
    Counter {
        index: u8,
        mode: CounterMode,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Language {
    LabVIEW = 1,
    Cpp,
    Java,
    Python,
    DotNet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Trigger {
    InWindow,
    State,
    RisingPulse,
    FallingPulse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CounterMode {
    TwoPulse,
    Semiperiod,
    PulseLength,
    ExternalDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Framework {
    Iterative = 1,
    Simple,
    CommandControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SPIPort {
    OnboardCS0,
    OnboardCS1,
    OnboardCS2,
    OnboardCS3,
    MXP,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ADXL345 {
    SPI = 1,
    I2C,
}
