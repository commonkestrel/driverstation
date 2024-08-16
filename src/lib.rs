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

use send::tcp::{MatchInfo, MatchType, TcpEvent};
use send::udp::UdpEvent;
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::sync::mpsc::{channel, Receiver, SendError, Sender};
use std::time::{Duration, Instant};
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
    game_data: GameData,
    tcp_tx: Sender<TcpEvent>,
    udp_tx: Sender<UdpEvent>,
}

impl Robot {
    pub fn new(team_number: u16) -> Self {
        let team_ip = ip_from_team(team_number);
        let cloned_team_ip = team_ip.clone();

        let (tcp_tx, tcp_rx) = channel();
        std::thread::spawn(move || {
            if let Err(err) = tcp_thread(cloned_team_ip, tcp_rx) {
                todo!()
            }
        });

        let (udp_tx, udp_rx) = channel();
        std::thread::spawn(move || {
            if let Err(err) = udp_thread(team_ip, udp_rx) {
                todo!()
            }
        });

        tcp_tx.send(TcpEvent::GameData(GameData::empty())).unwrap();
        tcp_tx.send(TcpEvent::MatchInfo(MatchInfo::new(None, MatchType::None))).unwrap();

        Robot {
            team: team_number,
            estopped: false,
            enabled: false,
            alliance: Alliance::Red1,
            mode: Mode::Teleoperated,
            game_data: GameData::default(),
            tcp_tx,
            udp_tx,
        }
    }

    pub fn queue_tcp(&mut self, ev: TcpEvent) {
        self.tcp_tx.send(ev).unwrap();
    }

    pub fn queue_udp(&mut self, ev: UdpEvent) {
        self.udp_tx.send(ev).unwrap();
    }
}

fn tcp_thread(team_ip: [u8; 4], rx: Receiver<TcpEvent>) -> std::io::Result<()> {
    let mut team_addr = SocketAddr::from((team_ip, TCP_PORT));
    let sim_addr = SocketAddr::from((SIM_IP, TCP_PORT));



    'search: loop {
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
            let start = Instant::now();

            std::thread::sleep(Duration::from_secs(1))
        }
    }
}

fn udp_thread(team_ip: [u8; 4], rx: Receiver<UdpEvent>) -> std::io::Result<()> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameData {
    chars: [Option<u8>; 3],
}

impl GameData {
    pub fn empty() -> Self {
        GameData {
            chars: [None; 3],
        }
    }

    pub fn single(character: u8) -> Self {
        GameData {
            chars: [Some(character), None, None]
        }
    }

    pub fn double(first: u8, second: u8) -> Self {
        GameData {
            chars: [Some(first), Some(second), None]
        }
    }

    pub fn triple(first: u8, second: u8, third: u8) -> Self {
        GameData {
            chars: [Some(first), Some(second), Some(third)]
        }
    }

    fn len(&self) -> u8 {
        match self.chars {
            [Some(_), Some(_), Some(_)] => 3,
            [Some(_), Some(_), _] => 2,
            [Some(_), _, _] => 1,
            _ => 0,
        }
    }
}

impl Bytes for GameData {
    fn write_bytes(&self, out: &mut Vec<u8>) {
        match self.chars {
            [Some(first), Some(second), Some(third)] => {
                out.extend_from_slice(&[first, second, third])
            }
            [Some(first), Some(second), _] => out.extend_from_slice(&[first, second]),
            [Some(first), _, _] => out.push(first),
            _ => {}
        }
    }
}

impl Default for GameData {
    fn default() -> Self {
        GameData {
            chars: [None, None, None],
        }
    }
}

/// Constructs the RoboRIO IP address from the given team number.
fn ip_from_team(team: u16) -> [u8; 4] {
    [10, (team / 100) as u8, (team % 100) as u8, 1]
}
