use serde::{Deserialize, Serialize};

const SIM_TCP_PORT: u16 = 1740;
const SIM_UDP_PORT: u16 = 1110;

pub struct Robot {
    team: u16,
    estopped: bool,
    enabled: bool,
    mode: Mode,
    station: Station,
}

impl Robot {
    pub fn new_team() -> Robot {
        todo!()
        // Robot::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Mode {
    Teleoperated,
    Autonomous,
    Test,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
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
