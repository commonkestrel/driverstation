pub mod recv {
    pub mod entry;
    pub mod tcp;
    pub mod udp;
}

pub mod send {
    pub mod tcp;
    pub mod udp;
}

mod sync;
pub mod traits;

use recv::udp::{CodeStatus, UdpResponse};
use send::tcp::{self, MatchInfo, MatchType, TcpEvent};
use send::udp;
use send::udp::UdpEvent;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::task::Poll;
use std::time::{Duration, Instant};
use tokio::net::{TcpStream, UdpSocket};
pub use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;
use tokio::{
    io::{self, AsyncWriteExt},
    select,
};
use traits::Bytes;

const UDP_PORT: u16 = 1110;
const TCP_PORT: u16 = 1740;
const SIM_IP: [u8; 4] = [127, 0, 0, 1];

// There's probably an IP address that DriverStation connects from
const DS_UDP_IP: [u8; 4] = [0, 0, 0, 0];
const DS_SIM_UDP_IP: [u8; 4] = [127, 0, 0, 1];
const DS_UDP_TX_PORT: u16 = 56789;
const DS_UDP_RX_PORT: u16 = 1150;

pub struct Robot {
    state: Arc<RwLock<State>>,
    tcp_tx: UnboundedSender<TcpEvent>,
    udp_tx: UnboundedSender<UdpEvent>,
    rt: sync::Runtime,
}

impl Robot {
    pub fn new(team_number: u16) -> Self {
        let team_ip = ip_from_team(team_number);

        let state = Arc::new(RwLock::new(State::new(team_number)));
        let (conn_tx, conn_rx) = unbounded_channel();

        let rt = sync::Runtime::current().unwrap();

        let (tcp_tx, tcp_rx) = unbounded_channel();
        rt.spawn(tcp_thread(team_ip, state.clone(), tcp_rx, conn_rx));

        let (udp_tx, udp_rx) = unbounded_channel();
        rt.spawn(udp_thread(team_ip, state.clone(), udp_rx, conn_tx));

        tcp_tx.send(TcpEvent::GameData(GameData::empty())).unwrap();
        tcp_tx
            .send(TcpEvent::MatchInfo(MatchInfo::new(None, MatchType::None)))
            .unwrap();

        Robot {
            state,
            tcp_tx,
            udp_tx,
            rt,
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

    pub fn set_team_number(&mut self, team_number: u16) {
        self.queue_udp(UdpEvent::TeamNumber(team_number));
        self.queue_tcp(TcpEvent::TeamNumber);
    }

    pub fn queue_tcp(&mut self, ev: TcpEvent) {
        self.tcp_tx.send(ev).unwrap();
    }

    pub fn queue_udp(&mut self, ev: UdpEvent) {
        self.udp_tx.send(ev).unwrap();
    }
}

#[cfg(feature = "sync")]
impl Robot {
    pub fn connected(&self) -> bool {
        self.rt.block_on(self._connected())
    }

    pub fn enabled(&self) -> bool {
        self.rt.block_on(self._enabled())
    }

    pub fn estopped(&self) -> bool {
        self.rt.block_on(self._estopped())
    }

    pub fn alliance(&self) -> Alliance {
        self.rt.block_on(self._alliance())
    }

    pub fn mode(&self) -> Mode {
        self.rt.block_on(self._mode())
    }

    pub fn code(&self) -> CodeStatus {
        self.rt.block_on(self._code())
    }

    pub fn battery(&self) -> f32 {
        self.rt.block_on(self._battery())
    }

    pub fn game_data(&self) -> GameData {
        self.rt.block_on(self._game_data())
    }

    pub async fn _connected(&self) -> bool {
        self.state.read().await.connected
    }

    pub async fn _enabled(&self) -> bool {
        self.state.read().await.enabled
    }

    pub async fn _estopped(&self) -> bool {
        self.state.read().await.estopped
    }

    pub async fn _alliance(&self) -> Alliance {
        self.state.read().await.alliance
    }

    pub async fn _mode(&self) -> Mode {
        self.state.read().await.mode
    }

    pub async fn _game_data(&self) -> GameData {
        self.state.read().await.game_data
    }

    pub async fn _code(&self) -> CodeStatus {
        self.state.read().await.code
    }

    pub async fn _battery(&self) -> f32 {
        self.state.read().await.battery
    }
}

#[cfg(not(feature = "sync"))]
impl Robot {
    pub async fn connected(&self) -> bool {
        self.state.read().await.connected
    }

    pub async fn enabled(&self) -> bool {
        self.state.read().await.enabled
    }

    pub async fn estopped(&self) -> bool {
        self.state.read().await.estopped
    }

    pub async fn alliance(&self) -> Alliance {
        self.state.read().await.alliance
    }

    pub async fn mode(&self) -> Mode {
        self.state.read().await.mode
    }

    pub async fn game_data(&self) -> GameData {
        self.state.read().await.game_data
    }

    pub async fn code(&self) -> CodeStatus {
        self.state.read().await.code
    }

    pub async fn battery(&self) -> f32 {
        self.state.read().await.battery
    }
}

struct State {
    connected: bool,
    team: u16,
    estopped: bool,
    enabled: bool,
    alliance: Alliance,
    mode: Mode,
    game_data: GameData,
    code: CodeStatus,
    battery: f32,
}

impl State {
    fn new(team_number: u16) -> State {
        State {
            connected: false,
            team: team_number,
            estopped: false,
            enabled: false,
            alliance: Alliance::Red1,
            mode: Mode::Teleoperated,
            game_data: GameData::default(),
            code: CodeStatus::Initializing,
            battery: 0.0,
        }
    }
}

async fn tcp_thread(
    team_ip: [u8; 4],
    state: Arc<RwLock<State>>,
    mut rx: UnboundedReceiver<TcpEvent>,
    mut conn_rx: UnboundedReceiver<Option<SocketAddr>>,
) -> std::io::Result<()> {
    let mut game_data = None;
    let mut match_info = None;
    let mut joysticks = Vec::new();

    loop {
        let mut addr = match conn_rx.recv().await {
            Some(location) => match location {
                Some(addr) => addr,
                None => continue,
            },
            None => return Ok(()),
        };
        addr.set_port(TCP_PORT);

        let mut conn = match TcpStream::connect(addr).await {
            Ok(conn) => conn,
            Err(_) => continue,
        };
        conn.set_nodelay(true)?;

        'conn: loop {
            let start = Instant::now();

            while let Ok(ev) = rx.try_recv() {
                match ev {
                    TcpEvent::Exit => return Ok(()),
                    TcpEvent::GameData(gd) => game_data = Some(gd),
                    TcpEvent::MatchInfo(mi) => match_info = Some(mi),
                    TcpEvent::Joystick(js) => joysticks.push(js),
                    TcpEvent::TeamNumber => continue 'conn,
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

async fn udp_thread(
    team_ip: [u8; 4],
    state: Arc<RwLock<State>>,
    mut rx: UnboundedReceiver<UdpEvent>,
    conn_tx: UnboundedSender<Option<SocketAddr>>,
) -> std::io::Result<()> {
    let mut team_addr = SocketAddr::from((team_ip, UDP_PORT));
    let sim_addr = SocketAddr::from((SIM_IP, UDP_PORT));

    let mut sequence: u16 = 0x0001;

    let mut estopped = false;
    let mut enabled = false;
    let mut fms_connected = false;
    let mut alliance = Alliance::Red1;
    let mut mode = Mode::Teleoperated;
    let mut restarting_code = false;
    let mut tags = Vec::new();

    'conn: loop {
        let udp_tx = UdpSocket::bind(SocketAddr::from((DS_UDP_IP, DS_UDP_TX_PORT))).await?;
        let udp_rx = UdpSocket::bind(SocketAddr::from((DS_UDP_IP, DS_UDP_RX_PORT))).await?;

        udp_tx.connect(sim_addr).await?;
        udp_tx.connect(team_addr).await?;
        let mut last = Instant::now();

        let mut rebooting_roborio = false;

        loop {
            let start = Instant::now();

            while let Ok(ev) = rx.try_recv() {
                match ev {
                    UdpEvent::Enabled(e) => enabled = e,
                    UdpEvent::Estopped(e) => estopped = e,
                    UdpEvent::FmsConnected(fc) => fms_connected = fc,
                    UdpEvent::Alliance(a) => alliance = a,
                    UdpEvent::Mode(m) => mode = m,
                    UdpEvent::Tag(tag) => tags.push(tag),
                    UdpEvent::RestartCode => restarting_code = true,
                    UdpEvent::RebootRoborio => rebooting_roborio = true,
                    UdpEvent::TeamNumber(num) => {
                        team_addr = SocketAddr::from((ip_from_team(num), UDP_PORT));
                        continue 'conn;
                    }
                }
            }

            let mut send_tags = Vec::new();
            send_tags.append(&mut tags);

            let connected = state.read().await.connected;

            let packet = udp::Packet::default()
                .with_sequence(sequence)
                .with_ds_connected(connected)
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
            udp_tx.send(&send).await?;
            sequence = sequence.wrapping_add(1);

            let mut buf = [0u8; 100];
            match udp_rx.try_recv_from(&mut buf) {
                Ok((bytes, addr)) => {
                    if let Ok(packet) = UdpResponse::try_from(&buf[0..bytes]) {
                        conn_tx.send(Some(addr)).unwrap();
                        last = Instant::now();
                        let mut current_state = state.write().await;

                        current_state.connected = true;
                        current_state.enabled = packet.status.enabled();
                        current_state.estopped = packet.status.estopped();
                        current_state.mode = packet.status.mode();
                        current_state.code = packet.status.code_start();
                        current_state.battery = packet.battery.voltage();
                    } else if let Err(err) = UdpResponse::try_from(&buf[0..bytes]) {
                        println!("{err:?}");
                    }
                }
                Err(err) => {
                    if last.elapsed() > Duration::from_millis(500) {
                        state.write().await.connected = false;
                        conn_tx.send(None).unwrap();
                        break;
                    }
                }
            }

            let duration = Instant::now().duration_since(start);
            if duration < Duration::from_millis(20) {
                std::thread::sleep(Duration::from_millis(20) - start.elapsed());
            }
        }
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
        GameData { chars: [None; 3] }
    }

    pub fn single(character: u8) -> Self {
        GameData {
            chars: [Some(character), None, None],
        }
    }

    pub fn double(first: u8, second: u8) -> Self {
        GameData {
            chars: [Some(first), Some(second), None],
        }
    }

    pub fn triple(first: u8, second: u8, third: u8) -> Self {
        GameData {
            chars: [Some(first), Some(second), Some(third)],
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
    [10, (team / 100) as u8, (team % 100) as u8, 2]
}
