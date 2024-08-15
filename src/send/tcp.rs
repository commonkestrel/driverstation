use std::ffi::CString;

pub struct Packet {
    game_data: Option<GameData>,
    match_info: Option<MatchInfo>,
    joysticks: Vec<Joystick>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GameData {
    first_char: Option<u8>,
    second_char: Option<u8>,
    third_char: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MatchInfo {
    competition: CString,
    ty: MatchType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MatchType {
    None,
    Practice,
    Qualifications,
    Eliminations,
}

pub struct Joystick {
    
}
