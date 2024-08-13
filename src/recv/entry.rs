use std::{
    ffi::CString,
    mem::{self, size_of},
};

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
    Dashboard,
    DigitalInput {
        channel: u8,
    },
    DigitalOutput {
        channel: u8,
    },
    DriverStationCIO,
    DriverStationEIO,
    DriverStationLCD,
    Encoder {
        fpga_index: u8,
        encoding: Encoding,
    },
    GearTooth {
        channel: u8,
    },
    Gyro {
        channel: u8,
    },
    I2C {
        address: u8,
    },
    Framework(Framework),
    Jaguar {
        channel: u8,
    },
    Joystick {
        port: u8,
    },
    Kinect,
    KinectStick,
    PIDController {
        /// The instance number.
        /// Starts at `1`.
        instance: u8,
    },
    Preferences,
    PWM {
        channel: u8,
    },
    Relay {
        channel: u8,
        reversable: bool,
    },
    RobotDrive {
        motors: u8,
        ty: DriveType,
    },
    SerialPort,
    Servo {
        channel: u8,
    },
    Solenoid {
        channel: u8,
    },
    SPI {
        /// The instance number.
        /// Starts at `1`.
        instance: u8,
    },
    Task,
    Ultrasonic {
        channel: u8,
    },
    Victor {
        channel: u8,
    },
    Button,
    Command,
    AxisCamera {
        handle: u8,
    },
    PCVideoServer {
        handle: u8,
    },
    SmartDashboard,
    Talon {
        channel: u8,
    },
    HiTechnicColorSensor,
    HiTechnicAccel,
    HiTechnicCompass,
    SRF08 {
        channel: u8,
    },
    AnalogOutput,
    VictorSP {
        channel: u8,
    },
    PWMTalonSRC {
        channel: u8,
    },
    CANTalonSRX {
        channel: u8,
    },
    ADXL362 {
        port: SPIPort,
    },
    ADXRS450 {
        port: SPIPort,
    },
    RevSPARK {
        channel: u8,
    },
    MindsensorsSD540 {
        channel: u8,
    },
    DigitalFilter {
        channel: u8,
    },
    ADIS16448,
    PDP,
    PCM,
    PigeonIMU {
        id: u8,
    },
    NidecBrushless {
        channel: u8,
    },
    CANifier {
        id: u8,
    },
    CTRE_future0 {
        id: u8,
    },
    CTRE_future1 {
        id: u8,
    },
    CTRE_future2 {
        id: u8,
    },
    CTRE_future3 {
        id: u8,
    },
    CTRE_future4,
    CTRE_future5,
    CTRE_future6,
}

impl Entry {
    pub fn entries_from_string(source: CString) -> Vec<Entry> {
        let bytes = source.into_bytes();
        let mut entries = Vec::new();

        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'A' => entries.push(Entry::Controller),
                b'B' => entries.push(Entry::Module),
                b'C' => entries.push(Entry::Language(Entry::parse_instance(&mut i, &bytes))),
                b'D' => entries.push(Entry::CANPlugin),
                b'E' => entries.push(Entry::Accelerometer {
                    channel: Entry::parse_instance(&mut i, &bytes),
                }),
                b'F' => entries.push(Entry::ADXL345(Entry::parse_instance(&mut i, &bytes))),
                b'G' => entries.push(Entry::AnalogChannel {
                    channel: Entry::parse_instance(&mut i, &bytes),
                }),
                b'H' => entries.push(Entry::AnalogTrigger {
                    channel: Entry::parse_instance(&mut i, &bytes),
                }),
                b'I' => {
                    let index = Entry::parse_instance(&mut i, &bytes);

                    let ty = if bytes[i+1] == b':' {
                        i += 2;
                        bytes[i].into()
                    } else {
                        continue;
                    };

                    entries.push(Entry::AnalogTriggerOutput {index, ty})
                },
                _ => {}
            }

            i += 1;
        }

        entries
    }

    fn parse_instance<Dst: From<u8>>(i: &mut usize, bytes: &[u8]) -> Dst {
        assert_eq!(size_of::<Dst>(), size_of::<u8>());

        *i += 1;
        let instance = bytes[*i].into();
        instance
    }
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
