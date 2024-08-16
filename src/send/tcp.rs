use std::ffi::CString;

use crate::{traits::Bytes, GameData};

pub struct Packet {
    game_data: Option<GameData>,
    match_info: Option<MatchInfo>,
    joysticks: Vec<Joystick>,
}

impl Packet {
    pub fn with_game_data(mut self, game_data: Option<GameData>) -> Self {
        self.game_data = game_data;
        self
    }

    pub fn with_match_info(mut self, match_info: Option<MatchInfo>) -> Self {
        self.match_info = match_info;
        self
    }

    pub fn with_joystick(mut self, joystick: Joystick) -> Self {
        self.joysticks.push(joystick);
        self
    }

    pub fn with_joysticks(mut self, joysticks: Vec<Joystick>) -> Self {
        self.joysticks = joysticks;
        self
    }

    pub fn extend_joysticks(mut self, mut joysticks: Vec<Joystick>) -> Self {
        self.joysticks.append(&mut joysticks);
        self
    }
}

impl Bytes for Packet {
    fn write_bytes(&self, out: &mut Vec<u8>) {
        if self.game_data.is_none() && self.match_info.is_none() && self.joysticks.is_empty() {
            out.extend_from_slice(&[0x00, 0x00]);
            return;
        }

        if let Some(game_data) = self.game_data {
            out.push(game_data.len() + 1);
            out.push(0x0e);
            game_data.write_bytes(out);
        }

        if let Some(ref match_info) = self.match_info {
            out.push(match_info.len());
            out.push(0x07);
            match_info.write_bytes(out);
        }
    }
}

impl Default for Packet {
    fn default() -> Self {
        Packet {
            game_data: None,
            match_info: None,
            joysticks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TcpEvent {
    GameData(GameData),
    MatchInfo(MatchInfo),
    Joystick(Joystick),
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchInfo {
    competition: Option<CString>,
    ty: MatchType,
}

impl MatchInfo {
    pub fn new(competition: Option<CString>, ty: MatchType) -> Self {
        MatchInfo {
            competition,
            ty,
        }
    }

    fn len(&self) -> u8 {
        let competition_len = match self.competition {
            Some(ref competition) => competition.as_bytes().len(),
            None => 5,
        };

        // The length of the internal string, plus two bytes for the match type and string length
        (competition_len as u8) + 2
    }
}

impl Bytes for MatchInfo {
    fn write_bytes(&self, out: &mut Vec<u8>) {
        let competition = match self.competition {
            Some(ref competition) => competition.as_bytes(),
            None => [0x00, 0x00, 0x00, 0x00, 0x00].as_slice(),
        };
        out.push(competition.len() as u8);
        out.extend_from_slice(competition);

        out.push(self.ty as u8);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MatchType {
    None,
    Practice,
    Qualifications,
    Eliminations,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Joystick {
    index: u8,
    is_xbox: bool,
    ty: JoystickType,
    name: CString,
    axis_types: Vec<AxisType>,
    button_count: u8,
    pov_count: u8,
}

impl Bytes for Joystick {
    fn write_bytes(&self, out: &mut Vec<u8>) {
        out.push(self.index);
        out.push(self.is_xbox as u8);

        out.push(unsafe { std::mem::transmute(self.ty as i8) });
        // SAFETY: will be reinterpreted as an `i8` when recieved
        out.extend_from_slice(self.name.as_bytes());

        out.push(self.axis_types.len() as u8);
        // SAFETY: `AxisType` is stored as `repr(u8)`, and will be interpreted as such
        out.extend_from_slice(unsafe { std::mem::transmute(self.axis_types.as_slice()) });

        out.push(self.button_count);
        out.push(self.pov_count);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum JoystickType {
    Unknown = -1,
    XInputUnknown,
    XInputGamepad,
    XInputWheel,
    XInputArcade,
    XInputFlightStick,
    XInputDancePad,
    XInputGuitar,
    XInputGuitar2,
    XInputDrumKit,
    XInputGuitar3 = 11,
    XInputArcadePad = 19,
    HIDJoystick,
    HIDGamepad,
    HIDDriving,
    HIDFlight,
    HIDFirstPerson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AxisType {
    X,
    Y,
    Z,
    Twist,
    Throttle,
}
