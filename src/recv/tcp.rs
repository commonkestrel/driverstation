use std::ffi::{c_char, CString};

use super::entry::Entry;

pub struct TcpResponse {
    tags: Vec<Tag>,
}

#[derive(Debug, Clone)]
pub enum Tag {
    Radio(String),
    UsageReport {
        team_num: [c_char; 2],
        entries: Vec<Entry>,
    },
    DisableFaults {
        comms: u16,
        twelve_volt: u16,
    },
    RailFaults {
        six_volt: u16,
        five_volt: u16,
        three_three_volt: u16,
    },
    VersionInfo {
        ty: Device,
        id: u8,
        name: CString,
        version: CString,
    },
    ErrorMessage {
        timestamp: f32,
        sequence: u16,
        error_code: i32,
        flags: Flags,
        details: CString, 
        location: CString,
        call_stack: CString,
    },
    StandardOutput {
        timestamp: f32,
        sequence: u16,
        message: CString,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    Software = 0x00,
    CANTalon = 0x02,
    PDP = 0x08,
    PCM = 0x09,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Flags(u8);
