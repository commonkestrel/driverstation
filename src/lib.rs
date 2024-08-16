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

use async_std::io;
use async_std::prelude::*;
use async_std::net::{SocketAddr, TcpStream, UdpSocket};
use send::tcp::Joystick;
use send::tcp::{self, MatchInfo, MatchType, TcpEvent};
use send::udp;
use send::udp::UdpEvent;
use serde::{Deserialize, Serialize};
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

        let (conn_tx, conn_rx) = channel();

        let (tcp_tx, tcp_rx) = channel();
        async_std::task::spawn(tcp_thread(team_ip, tcp_rx, conn_tx));
        
        let (udp_tx, udp_rx) = channel();
        async_std::task::spawn(udp_thread(team_ip, udp_rx, conn_rx));

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

    pub fn set_enabled(&mut self, enabled: bool) {
        self.queue_udp(UdpEvent::Enabled(enabled));
    }

    pub fn set_estopped(&mut self, estopped: bool) {
        self.queue_udp(UdpEvent::Estopped(estopped));
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.queue_udp(UdpEvent::Mode(mode));
    }

    pub fn set_alliance(&mut self, alliance: Alliance) {
        self.queue_udp(UdpEvent::Alliance(alliance));
    }

    pub fn queue_tcp(&mut self, ev: TcpEvent) {
        self.tcp_tx.send(ev).unwrap();
    }

    pub fn queue_udp(&mut self, ev: UdpEvent) {
        self.udp_tx.send(ev).unwrap();
    }
}

async fn tcp_thread(team_ip: [u8; 4], rx: Receiver<TcpEvent>, conn_tx: Sender<Location>) -> std::io::Result<()> {
    let mut team_addr = SocketAddr::from((team_ip, TCP_PORT));
    let sim_addr = SocketAddr::from((SIM_IP, TCP_PORT));

    let mut game_data = None;
    let mut match_info = None;
    let mut joysticks = Vec::new();

    loop {
        conn_tx.send(Location::None).unwrap();

        let team_conn = tcp_connect(team_addr, Location::Team);
        let sim_conn = tcp_connect(sim_addr, Location::Sim);
        let (location, mut conn) = match team_conn.try_race(sim_conn).await {
            Ok(conn) => conn,
            Err(_) => continue,
        };
        conn_tx.send(location).unwrap();
        conn.set_nodelay(true)?;

        loop {
            let start = Instant::now();

            for ev in rx.try_iter() {
                match ev {
                    TcpEvent::Exit => return Ok(()),
                    TcpEvent::GameData(gd) => game_data = Some(gd),
                    TcpEvent::MatchInfo(mi) => match_info = Some(mi),
                    TcpEvent::Joystick(js) => joysticks.push(js),
                }
            }

            let mut copied = Vec::with_capacity(joysticks.len());
            copied.append(&mut joysticks);

            let packet = tcp::Packet::default()
                .with_game_data(game_data)
                .with_match_info(match_info)
                .with_joysticks(copied);

            game_data = None;
            match_info = None;

            let mut send = Vec::new();
            packet.write_bytes(&mut send);
            if conn.write_all(&send).await.is_err() {
                break;
            }

            // Sends a TCP packet every second
            std::thread::sleep(Duration::from_secs(1) - Instant::now().duration_since(start));
        }
    }
}

async fn udp_thread(team_ip: [u8; 4], rx: Receiver<UdpEvent>, conn_rx: Receiver<Location>) -> std::io::Result<()> {
    let mut team_addr = SocketAddr::from((team_ip, UDP_PORT));
    let sim_addr = SocketAddr::from((SIM_IP, UDP_PORT));

    let socket = UdpSocket::bind(SocketAddr::from((DS_UDP_IP, DS_UDP_PORT))).await?;
    let mut sequence: u16 = 0x0000;

    let mut estopped = false;
    let mut enabled = false;
    let mut fms_connected = false;
    let mut alliance = Alliance::Red1;
    let mut mode = Mode::Teleoperated;
    let mut restarting_code = false;
    let mut tags = Vec::new();
    
    for connection in conn_rx {
        if connection != Location::None {
            let addr = match connection {
                Location::Team => team_addr,
                Location::Sim => sim_addr,
                Location::None => unreachable!(),
            };
            socket.connect(addr).await?;
            let mut rebooting_roborio = false;

            loop {
                let start = Instant::now();

                for ev in rx.try_iter() {
                    match ev {
                        UdpEvent::Enabled(e) => enabled = e,
                        UdpEvent::Estopped(e) => estopped = e,
                        UdpEvent::FmsConnected(fc) => fms_connected = fc,
                        UdpEvent::Alliance(a) => alliance = a,
                        UdpEvent::Mode(m) => mode = m,
                        UdpEvent::Tag(tag) => tags.push(tag),
                        UdpEvent::RestartCode => restarting_code = true,
                        UdpEvent::RebootRoborio => rebooting_roborio = true,
                    }
                    
                }

                let mut send_tags = Vec::new();
                send_tags.append(&mut tags);
            
                let packet = udp::Packet::default()
                    .with_sequence(sequence)
                    .with_enabled(enabled)
                    .with_estopped(estopped)
                    .with_alliance(alliance)
                    .with_fms_connected(fms_connected)
                    .with_mode(mode)
                    .with_reboot_roborio(rebooting_roborio)
                    .with_restart_code(restarting_code)
                    .with_tags(send_tags);
                
                let mut send = Vec::new();
                packet.write_bytes(&mut send);
                socket.send(&send).await?;
                sequence = sequence.wrapping_add(1);

                let duration = Instant::now().duration_since(start);
                if duration < Duration::from_millis(20) {
                    std::thread::sleep(Duration::from_millis(20) - Instant::now().duration_since(start));
                }
            }
        }
    }

    Ok(())
}

async fn tcp_connect(ip: SocketAddr, location: Location) -> io::Result<(Location, TcpStream)> {
    TcpStream::connect(ip).await.map(|stream| (location, stream))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Location {
    Team,
    Sim,
    None,
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
