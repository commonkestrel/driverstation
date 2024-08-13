use std::ffi::{c_char, CString};

use super::entry::Entry;

#[derive(Debug, Clone)]
pub enum Event {
    Radio(String),
    UsageReport {
        team_num: [c_char; 2],
        entries: Vec<Entry>,
    },
}
