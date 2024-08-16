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
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::time::Duration;
use std::io::Write;
use traits::Bytes;

const UDP_PORT: u16 = 1110;
const TCP_PORT: u16 = 1740;
const SIM_IP: [u8; 4] = [127, 0, 0, 1];

// There's probably an IP address that DriverStation connects from
const DS_UDP_IP: [u8; 4] = [127, 0, 0, 1];
const DS_UDP_PORT: u16 = 64651;

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

        Robot {
            team: team_number,
            estopped: false,
            enabled: false,
            alliance: Alliance::Red1,
            mode: Mode::Teleoperated,
        }
    }
}

fn tcp_thread(team_ip: [u8; 4]) -> std::io::Result<()> {
    loop {
        let team_addr = SocketAddr::from((team_ip, TCP_PORT));
        let sim_addr = SocketAddr::from((SIM_IP, TCP_PORT));

        let conn = match TcpStream::connect_timeout(&team_addr, Duration::from_millis(100))
            .or_else(|_| TcpStream::connect_timeout(&sim_addr, Duration::from_millis(100)))
        {
            Ok(conn) => conn,
            Err(err) => {
                println!("{err}");
                continue;
            }
        };

        loop {
            
        }
    }
}

fn udp_thread(team_ip: [u8; 4]) -> std::io::Result<()> {
    loop {}

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
fn ip_from_team(team: u16) -> [u8; 4] {
    [10, (team / 100) as u8, (team % 100) as u8, 1]
}
