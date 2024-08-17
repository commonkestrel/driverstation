use crate::Mode;

pub struct UdpResponse {
    sequence: u16,
    comm_version: u8,
    status: Status,
    trace: Trace,
    battery: Battery,
    first_conn: bool,
    tags: Vec<Tag>,
}

pub struct Status(u8);

impl Status {
    const ESTOP_MASK: u8 = 0x80;
    const BROWNOUT_MASK: u8 = 0x10;
    const CODE_START_MASK: u8 = 0x08;
    const ENABLED_MASK: u8 = 0x04;
    const MODE_MASK: u8 = 0x03;

    pub fn from_bits(bits: u8) -> Status {
        Status(bits)
    }

    pub fn estopped(&self) -> bool {
        (self.0 & Self::ESTOP_MASK) > 0
    }

    pub fn brownout(&self) -> bool {
        (self.0 & Self::BROWNOUT_MASK) > 0
    }

    pub fn code_start(&self) -> CodeStatus {
        match self.0 & Self::CODE_START_MASK {
            0 => CodeStatus::Running,
            _ => CodeStatus::Initializing,
        }
    }

    pub fn enabled(&self) -> bool {
        (self.0 & Self::ENABLED_MASK) > 0
    }

    pub fn mode(&self) -> Mode {
        match self.0 & Self::MODE_MASK {
            0 => Mode::Teleoperated,
            2 => Mode::Autonomous,
            _ => Mode::Test,
        }
    }
}

pub struct Trace(u8);

impl Trace {
    const ROBOT_CODE_MASK: u8 = 0x20;
    const IS_RIO_MASK: u8 = 0x10;
    const TEST_MASK: u8 = 0x08;
    const AUTO_MASK: u8 = 0x04;
    const TELEOP_MASK: u8 = 0x02;
    const DISABLED_MASK: u8 = 0x01;

    pub fn robot_code(&self) -> bool {
        (self.0 & Self::ROBOT_CODE_MASK) > 0
    }

    pub fn is_roborio(&self) -> bool {
        (self.0 & Self::IS_RIO_MASK) > 0
    }

    pub fn test_mode(&self) -> bool {
        (self.0 & Self::TEST_MASK) > 0
    }

    pub fn autonomous_mode(&self) -> bool {
        (self.0 & Self::AUTO_MASK) > 0
    }

    pub fn teleop_mode(&self) -> bool {
        (self.0 & Self::TELEOP_MASK) > 0
    }

    pub fn enabled(&self) -> bool {
        !((self.0 & Self::DISABLED_MASK) > 0)
    }
}

pub struct Battery(u16);

impl Battery {
    pub fn voltage(&self) -> f32 {
        let xx = (self.0 >> 8) as f32;
        let yy = (self.0 & 0xFF) as f32;

        xx + yy/256.0
    }
}

pub enum Tag {
    
}

pub enum CodeStatus {
    Running,
    Initializing,
}
