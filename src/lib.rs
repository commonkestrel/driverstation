mod recv {
    pub mod entry;
    pub mod tcp;
    pub mod udp;
}

use serde::{Deserialize, Serialize};

const UDP_PORT: u16 = 1110;
const TCP_PORT: u16 = 1740;

const SIM_TCP_ADDR: &str = "127.0.0.1:1740";
const SIM_UDP_ADDR: &str = "127.0.0.1:1110";
// There's probably an IP address that DriverStation connects from
const DS_UDP_ADDR: &str = "127.0.0.1:64651";

pub struct Robot {
    team: u16,
    ctrl: Control,
    station: Station,
}

impl Robot {
    pub fn new(team_number: u16) -> Robot {
        let team_ip = ip_from_team(team_number);
        let cloned_team_ip = team_ip.clone();

        std::thread::spawn(move || {
            if let Err(err) = tcp_thread(cloned_team_ip) {
                todo!()
            }
        });

        std::thread::spawn(move || {
            if let Err(err) = udp_thread(team_ip) {
                todo!()
            }
        });

        todo!()
    }
}

fn tcp_thread(team_ip: String) -> std::io::Result<()> {
    let team_addr = format!("{team_ip}:{TCP_PORT}");

    Ok(())
}

fn udp_thread(team_ip: String) -> std::io::Result<()> {
    let team_addr = format!("{team_ip}:{UDP_PORT}");

    Ok(())
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Control(u8);

impl Control {
    const MODE_MASK: u8 = 0x03;
    const ENABLE_MASK: u8 = 0x04;
    const ESTOP_MASK: u8 = 0x80;

    pub fn mode(&self) -> Mode {
        Mode::from_bits(self.0 & Self::MODE_MASK)
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.0 &= !Self::MODE_MASK;
        self.0 |= mode.into_bits();
    }

    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.set_mode(mode);
        self
    }

    pub fn enabled(&self) -> bool {
        self.0 & Self::ENABLE_MASK > 0
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.0 |= Self::ENABLE_MASK;
        } else {
            self.0 &= !Self::ENABLE_MASK;
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.set_enabled(enabled);
        self
    }

    pub fn estopped(&self) -> bool {
        self.0 & Self::ESTOP_MASK > 0
    }

    pub fn set_estopped(&mut self) {
        self.0 |= Self::ESTOP_MASK;
    }

    pub fn with_estopped(mut self) -> Self {
        self.set_estopped();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Teleoperated,
    Autonomous,
    Test,
}

impl Mode {
    const fn into_bits(self) -> u8 {
        match self {
            Mode::Teleoperated => 0x00,
            Mode::Autonomous => 0x02,
            Mode::Test => 0x01,
        }
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => Mode::Teleoperated,
            0x02 => Mode::Autonomous,
            _ => Mode::Test,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Station {
    Red1,
    Red2,
    Red3,
    Blue1,
    Blue2,
    Blue3,
}

/// Constructs the RoboRIO IP address from the given team number.
fn ip_from_team(team: u16) -> String {
    format!("10.{}.{}.1", team / 100, team % 100)
}
