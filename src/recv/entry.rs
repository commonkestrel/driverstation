use std::{
    ffi::CString,
    mem::{self, size_of},
};

use macros::ParseEntries;

#[derive(Debug, Clone, ParseEntries)]
pub enum Entry {
    #[entry(b"A")]
    Controller,
    #[entry(b"B")]
    Module,
    #[entry(b"C")]
    Language(Language),
    #[entry(b"D")]
    CANPlugin,
    #[entry(b"E")]
    Accelerometer {
        channel: u8,
    },
    #[entry(b"F")]
    ADXL345(ADXL345),
    #[entry(b"G")]
    AnalogChannel {
        channel: u8,
    },
    #[entry(b"H")]
    AnalogTrigger {
        channel: u8,
    },
    #[entry(b"I")]
    AnalogTriggerOutput {
        index: u8,
        ty: Trigger,
    },
    #[entry(b"J")]
    CANJaguar,
    #[entry(b"K")]
    Compressor {
        pcm_id: u8,
    },
    #[entry(b"L")]
    Counter {
        index: u8,
        mode: CounterMode,
    },
    #[entry(b"M")]
    Dashboard,
    #[entry(b"N")]
    DigitalInput {
        channel: u8,
    },
    #[entry(b"O")]
    DigitalOutput {
        channel: u8,
    },
    #[entry(b"P")]
    DriverStationCIO,
    #[entry(b"Q")]
    DriverStationEIO,
    #[entry(b"R")]
    DriverStationLCD,
    #[entry(b"S")]
    Encoder {
        fpga_index: u8,
        encoding: Encoding,
    },
    #[entry(b"T")]
    GearTooth {
        channel: u8,
    },
    #[entry(b"U")]
    Gyro {
        channel: u8,
    },
    #[entry(b"V")]
    I2C {
        address: u8,
    },
    #[entry(b"W")]
    Framework(Framework),
    #[entry(b"X")]
    Jaguar {
        channel: u8,
    },
    #[entry(b"Y")]
    Joystick {
        port: u8,
    },
    #[entry(b"Z")]
    Kinect,
    #[entry(b"a")]
    KinectStick,
    #[entry(b"b")]
    PIDController {
        /// The instance number.
        /// Starts at `1`.
        instance: u8,
    },
    #[entry(b"c")]
    Preferences,
    #[entry(b"d")]
    PWM {
        channel: u8,
    },
    #[entry(b"e")]
    Relay {
        channel: u8,
        reversable: bool,
    },
    #[entry(b"f")]
    RobotDrive {
        motors: u8,
        ty: DriveType,
    },
    #[entry(b"g")]
    SerialPort,
    #[entry(b"h")]
    Servo {
        channel: u8,
    },
    #[entry(b"i")]
    Solenoid {
        channel: u8,
    },
    #[entry(b"j")]
    SPI {
        /// The instance number.
        /// Starts at `1`.
        instance: u8,
    },
    #[entry(b"k")]
    Task,
    #[entry(b"l")]
    Ultrasonic {
        channel: u8,
    },
    #[entry(b"m")]
    Victor {
        channel: u8,
    },
    #[entry(b"n")]
    Button,
    #[entry(b"o")]
    Command,
    #[entry(b"p")]
    AxisCamera {
        handle: u8,
    },
    #[entry(b"q")]
    PCVideoServer {
        handle: u8,
    },
    #[entry(b"r")]
    SmartDashboard,
    #[entry(b"s")]
    Talon {
        channel: u8,
    },
    #[entry(b"t")]
    HiTechnicColorSensor,
    #[entry(b"u")]
    HiTechnicAccel,
    #[entry(b"v")]
    HiTechnicCompass,
    #[entry(b"w")]
    SRF08 {
        channel: u8,
    },
    #[entry(b"x")]
    AnalogOutput,
    #[entry(b"y")]
    VictorSP {
        channel: u8,
    },
    #[entry(b"z")]
    PWMTalonSRC {
        channel: u8,
    },
    #[entry(b">A")]
    CANTalonSRX {
        channel: u8,
    },
    #[entry(b">B")]
    ADXL362 {
        port: SPIPort,
    },
    #[entry(b">C")]
    ADXRS450 {
        port: SPIPort,
    },
    #[entry(b">D")]
    RevSPARK {
        channel: u8,
    },
    #[entry(b">E")]
    MindsensorsSD540 {
        channel: u8,
    },
    #[entry(b">F")]
    DigitalFilter {
        channel: u8,
    },
    #[entry(b">G")]
    ADIS16448,
    #[entry(b">H")]
    PDP,
    #[entry(b">I")]
    PCM,
    #[entry(b">J")]
    PigeonIMU {
        id: u8,
    },
    #[entry(b">K")]
    NidecBrushless {
        channel: u8,
    },
    #[entry(b">L")]
    CANifier {
        id: u8,
    },
    #[entry(b">M")]
    CTRE_future0 {
        id: u8,
    },
    #[entry(b">N")]
    CTRE_future1 {
        id: u8,
    },
    #[entry(b">O")]
    CTRE_future2 {
        id: u8,
    },
    #[entry(b">P")]
    CTRE_future3 {
        id: u8,
    },
    #[entry(b">Q")]
    CTRE_future4,
    #[entry(b">R")]
    CTRE_future5,
    #[entry(b">S")]
    CTRE_future6,
    
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

impl From<u8> for Language {
    fn from(value: u8) -> Self {
        match value {
            1 => Language::LabVIEW,
            2 => Language::Cpp,
            3 => Language::Java,
            4 => Language::Python,
            _ => Language::DotNet,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Trigger {
    InWindow,
    State,
    RisingPulse,
    FallingPulse,
}

impl From<u8> for Trigger {
    fn from(value: u8) -> Self {
        match value {
            0 => Trigger::InWindow,
            1 => Trigger::State,
            2 => Trigger::RisingPulse,
            _ => Trigger::FallingPulse,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CounterMode {
    TwoPulse,
    Semiperiod,
    PulseLength,
    ExternalDirection,
}

impl From<u8> for CounterMode {
    fn from(value: u8) -> Self {
        match value {
            0 => CounterMode::TwoPulse,
            1 => CounterMode::Semiperiod,
            2 => CounterMode::PulseLength,
            _ => CounterMode::ExternalDirection,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Framework {
    Iterative = 1,
    Simple,
    CommandControl,
}

impl From<u8> for Framework {
    fn from(value: u8) -> Self {
        match value {
            1 => Framework::Iterative,
            2 => Framework::Simple,
            _ => Framework::CommandControl,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DriveType {
    ArcadeStandard,
    ArcadeButtonSpin,
    ArcadeRatioCurve,
    Tank,
    MecanumPolar,
    MecanumCartesian,
}

impl From<u8> for DriveType {
    fn from(value: u8) -> Self {
        match value {
            0 => DriveType::ArcadeStandard,
            1 => DriveType::ArcadeButtonSpin,
            2 => DriveType::ArcadeRatioCurve,
            3 => DriveType::Tank,
            4 => DriveType::MecanumPolar,
            _ => DriveType::MecanumCartesian,
        }
    }
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

impl From<u8> for SPIPort {
    fn from(value: u8) -> Self {
        match value {
            0 => SPIPort::OnboardCS0,
            1 => SPIPort::OnboardCS1,
            2 => SPIPort::OnboardCS2,
            3 => SPIPort::OnboardCS3,
            _ => SPIPort::MXP,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ADXL345 {
    SPI = 1,
    I2C,
}

impl From<u8> for ADXL345 {
    fn from(value: u8) -> Self {
        match value {
            1 => ADXL345::SPI,
            _ => ADXL345::I2C,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Encoding {
    X1,
    X2,
    X4,
}

impl From<u8> for Encoding {
    fn from(value: u8) -> Self {
        match value {
            0 => Encoding::X1,
            1 => Encoding::X2,
            _ => Encoding::X4,
        }
    }
}
