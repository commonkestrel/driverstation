pub mod recv {
    pub mod entry;
    pub mod tcp;
    pub mod udp;
}

pub mod send {
    pub mod tcp;
    pub mod udp;
}

pub mod traits;

use serde::{Deserialize, Serialize};
use traits::Bytes;

const UDP_PORT: u16 = 1110;
const TCP_PORT: u16 = 1740;

const SIM_TCP_ADDR: &str = "127.0.0.1:1740";
const SIM_UDP_ADDR: &str = "127.0.0.1:1110";
// There's probably an IP address that DriverStation connects from
const DS_UDP_ADDR: &str = "127.0.0.1:64651";

pub struct Robot {
    team: u16,
    estopped: bool,
    enabled: bool,
    alliance: Alliance,
    mode: Mode,
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
pub enum Alliance {
    Red1,
    Red2,
    Red3,
    Blue1,
    Blue2,
    Blue3,
}

impl Bytes for Alliance {
    fn write_bytes(&self, out: &mut Vec<u8>) {
        let byte = match self {
            Alliance::Red1 => 0,
            Alliance::Red2 => 1,
            Alliance::Red3 => 2,
            Alliance::Blue1 => 3,
            Alliance::Blue2 => 4,
            Alliance::Blue3 => 5,
        };

        out.push(byte);
    }
}

/// Constructs the RoboRIO IP address from the given team number.
fn ip_from_team(team: u16) -> String {
    format!("10.{}.{}.1", team / 100, team % 100)
}
