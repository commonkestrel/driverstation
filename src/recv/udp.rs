use std::mem::size_of;

use serde::{Deserialize, Serialize};

use crate::Mode;

// sequence + comm_version + status + trace + battery + first_conn + tag length
const MIN_RESPONSE_SIZE: usize = size_of::<u16>()
    + size_of::<u8>()
    + size_of::<Status>()
    + size_of::<Trace>()
    + size_of::<Battery>()
    + size_of::<bool>();

pub struct UdpResponse {
    pub sequence: u16,
    pub comm_version: u8,
    pub status: Status,
    pub trace: Trace,
    pub battery: Battery,
    pub first_conn: bool,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdpParseError {
    InvalidLength,
    InvalidTag,
}

impl TryFrom<&[u8]> for UdpResponse {
    type Error = UdpParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < MIN_RESPONSE_SIZE {
            return Err(UdpParseError::InvalidTag);
        }

        let sequence = u16::from_be_bytes([value[0], value[1]]);
        let comm_version = value[2];
        let status = Status::from_bits(value[3]);
        let trace = Trace::from_bits(value[4]);
        let battery = Battery::from_bits([value[5], value[6]]);
        let first_conn = value[7] > 0;
        let tags = if value.len() > MIN_RESPONSE_SIZE {
            Tag::parse_tags(&value[8..])?
        } else {
            Vec::new()
        };

        Ok(UdpResponse {
            sequence,
            comm_version,
            status,
            trace,
            battery,
            first_conn,
            tags,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Status(u8);

impl Status {
    const ESTOP_MASK: u8 = 0x80;
    const BROWNOUT_MASK: u8 = 0x10;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Trace(u8);

impl Trace {
    const ROBOT_CODE_MASK: u8 = 0x20;
    const IS_RIO_MASK: u8 = 0x10;
    const TEST_MASK: u8 = 0x08;
    const AUTO_MASK: u8 = 0x04;
    const TELEOP_MASK: u8 = 0x02;
    const DISABLED_MASK: u8 = 0x01;

    pub fn from_bits(bits: u8) -> Self {
        Trace(bits)
    }

    pub fn robot_code(&self) -> CodeStatus {
        match self.0 & Self::ROBOT_CODE_MASK {
            0 => CodeStatus::Initializing,
            _ => CodeStatus::Running,
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Battery(u16);

impl Battery {
    pub fn from_bits(bits: [u8; 2]) -> Battery {
        Battery(u16::from_be_bytes(bits))
    }

    pub fn voltage(&self) -> f32 {
        let xx = (self.0 >> 8) as f32;
        let yy = (self.0 & 0xFF) as f32;

        xx + yy / 256.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tag {
    JoystickOutput {
        /// 1 bit per output, stored LSB 0
        outputs: u32,
        left_rumble: u16,
        right_rumble: u16,
    },
    DiskInfo {
        /// Number of bytes available
        free_space: u32,
    },
    CPUInfo {
        num_cpus: u8,
        critical: f32,
        above_normal: f32,
        normal: f32,
        low: f32,
    },
    RAMInfo {
        block: u32,
        /// Number of bytes available
        free_space: u32,
    },
    PDPLog {
        stats: [u16; 16],
    },
    CANMetrics {
        utilization: f32,
        bus_off: u32,
        tx_full: u32,
        rx_errors: u8,
        tx_errors: u8,
    },
}

impl Tag {
    const JOYSTICK_OUTPUT_LENGTH: u8 = 8;
    const DISK_INFO_LENGTH: u8 = 4 + 4;
    const CPU_INFO_LENGTH: u8 = 1 + 8 + 8 + 4 * 4;
    const RAM_INFO_LENGTH: u8 = 2 * 4;
    const PDP_LOG_LENGTH: u8 = 1 + 21 + 3;
    const CAN_METRICS_LENGTH: u8 = 4 + 4 + 4 + 1 + 1;

    pub fn parse_tags(buf: &[u8]) -> Result<Vec<Tag>, UdpParseError> {
        let mut tags = Vec::new();

        let mut i = 0;
        while i < buf.len() {
            let length = buf[i];

            if length as usize >= buf.len() - i {
                return Err(UdpParseError::InvalidTag);
            }

            if length > 0 {
                i += 1;
                match buf[i] {
                    0x01 => {
                        // Joystick Output
                        if length - 1 != Self::JOYSTICK_OUTPUT_LENGTH {
                            return Err(UdpParseError::InvalidTag);
                        }

                        let outputs =
                            u32::from_be_bytes([buf[i + 1], buf[i + 2], buf[i + 3], buf[i + 4]]);
                        let left_rumble = u16::from_be_bytes([buf[i + 5], buf[i + 6]]);
                        let right_rumble = u16::from_be_bytes([buf[i + 7], buf[i + 8]]);
                        i += 8;

                        tags.push(Tag::JoystickOutput {
                            outputs,
                            left_rumble,
                            right_rumble,
                        });
                    }
                    0x04 => {
                        // Disk Info
                        if length - 1 != Self::DISK_INFO_LENGTH {
                            return Err(UdpParseError::InvalidTag);
                        }

                        // Unknown 4 byte value
                        i += 4;

                        let free_space =
                            u32::from_be_bytes([buf[i + 1], buf[i + 2], buf[i + 3], buf[i + 4]]);
                        i += 4;

                        tags.push(Tag::DiskInfo { free_space })
                    }
                    0x05 => {
                        // CPU Info
                        if length - 1 != Self::CPU_INFO_LENGTH {
                            return Err(UdpParseError::InvalidTag);
                        }

                        let num_cpus = buf[i + 1];
                        let critical =
                            f32::from_be_bytes([buf[i + 2], buf[i + 3], buf[i + 4], buf[i + 5]]);
                        let above_normal = f32::from_be_bytes([
                            buf[i + 14],
                            buf[i + 15],
                            buf[i + 16],
                            buf[i + 17],
                        ]);
                        let normal = f32::from_be_bytes([
                            buf[i + 18],
                            buf[i + 19],
                            buf[i + 20],
                            buf[i + 21],
                        ]);
                        let low = f32::from_be_bytes([
                            buf[i + 30],
                            buf[i + 31],
                            buf[i + 32],
                            buf[i + 33],
                        ]);
                        i += 33;

                        tags.push(Tag::CPUInfo {
                            num_cpus,
                            critical,
                            above_normal,
                            normal,
                            low,
                        });
                    }
                    0x06 => {
                        // RAM Info
                        if length - 1 != Self::RAM_INFO_LENGTH {
                            return Err(UdpParseError::InvalidTag);
                        }

                        let block =
                            u32::from_be_bytes([buf[i + 1], buf[i + 2], buf[i + 3], buf[i + 4]]);
                        let free_space =
                            u32::from_be_bytes([buf[i + 5], buf[i + 6], buf[i + 7], buf[i + 8]]);
                        i += 8;

                        tags.push(Tag::RAMInfo { block, free_space });
                    }
                    0x08 => {
                        // PDP Log
                        if length - 1 != Self::PDP_LOG_LENGTH {
                            return Err(UdpParseError::InvalidTag);
                        }

                        i += 1;
                        let stats = [
                            (buf[i] as u16) + ((buf[i + 1] as u16) << 8) * 0x03FF,
                            (((buf[i + 1] >> 2) as u16) + ((buf[i + 2] as u16) << 6)) & 0x03FF,
                            (((buf[i + 2] >> 4) as u16) + ((buf[i + 3] as u16) << 4)) & 0x03FF,
                            (((buf[i + 3] >> 6) as u16) + ((buf[i + 4] as u16) << 2)) & 0x03FF,
                            (buf[i + 5] as u16) + ((buf[i + 6] as u16) << 8) & 0x03FF,
                            (((buf[i + 6] >> 2) as u16) + ((buf[i + 7] as u16) << 6)) & 0x03FF,
                            (buf[i + 8] as u16) + ((buf[i + 9] as u16) << 8) * 0x03FF,
                            (((buf[i + 9] >> 2) as u16) + ((buf[i + 10] as u16) << 6)) & 0x03FF,
                            (((buf[i + 10] >> 4) as u16) + ((buf[i + 11] as u16) << 4)) & 0x03FF,
                            (((buf[i + 11] >> 6) as u16) + ((buf[i + 12] as u16) << 2)) & 0x03FF,
                            (buf[i + 13] as u16) + ((buf[i + 14] as u16) << 8) & 0x03FF,
                            (((buf[i + 14] >> 2) as u16) + ((buf[i + 15] as u16) << 6)) & 0x03FF,
                            (buf[i + 16] as u16) + ((buf[i + 17] as u16) << 8) * 0x03FF,
                            (((buf[i + 17] >> 2) as u16) + ((buf[i + 18] as u16) << 6)) & 0x03FF,
                            (((buf[i + 18] >> 4) as u16) + ((buf[i + 19] as u16) << 4)) & 0x03FF,
                            (((buf[i + 19] >> 6) as u16) + ((buf[i + 20] as u16) << 2)) & 0x03FF,
                        ];
                        i += 24;

                        tags.push(Tag::PDPLog { stats })
                    }
                    0x09 => {
                        // Unknown value of length 9
                        i += 9;
                    }
                    0x0e => {
                        // CAN Metrics
                        if length - 1 != Self::CAN_METRICS_LENGTH {
                            return Err(UdpParseError::InvalidLength);
                        }

                        let utilization =
                            f32::from_be_bytes([buf[i + 1], buf[i + 2], buf[i + 3], buf[i + 4]]);
                        let bus_off =
                            u32::from_be_bytes([buf[i + 5], buf[i + 6], buf[i + 7], buf[i + 8]]);
                        let tx_full =
                            u32::from_be_bytes([buf[i + 9], buf[i + 10], buf[i + 11], buf[i + 12]]);
                        let rx_errors = buf[i + 13];
                        let tx_errors = buf[i + 14];
                        i += 14;

                        tags.push(Tag::CANMetrics {
                            utilization,
                            bus_off,
                            tx_full,
                            rx_errors,
                            tx_errors,
                        });
                    }
                    _ => {}
                }

                i += 1;
            }
        }

        Ok(tags)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CodeStatus {
    Running,
    Initializing,
}
