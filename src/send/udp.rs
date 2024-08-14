use std::ffi::CString;

use crate::{traits::Bytes, Alliance, Mode};

pub struct Packet {
    sequence: u16,
    version: u8,
    ctrl: Control,
    req: Request,
    alliance: Alliance,
    tags: Vec<Tag>,
}

impl Bytes for Packet {
    fn write_bytes(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.sequence.to_be_bytes());
        out.push(self.version);
        self.ctrl.write_bytes(out);
        self.req.write_bytes(out);
        self.alliance.write_bytes(out);
        for tag in self.tags.iter() {
            tag.write_bytes(out);
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Control(u8);

impl Bytes for Control {
    fn write_bytes(&self, out: &mut Vec<u8>) {
        out.push(self.0);
    }
}

impl Control {
    const MODE_MASK: u8 = 0x03;
    const ENABLE_MASK: u8 = 0x04;
    const ESTOP_MASK: u8 = 0x80;
    const FMS_MASK: u8 = 0x08;

    pub fn mode(&self) -> Mode {
        Mode::from_bits(self.0 & Self::MODE_MASK)
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.0 &= !Self::MODE_MASK;
        self.0 |= mode.into_bits();
    }

    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.set_mode(mode);
        self
    }

    pub fn enabled(&self) -> bool {
        self.0 & Self::ENABLE_MASK > 0
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.0 |= Self::ENABLE_MASK;
        } else {
            self.0 &= !Self::ENABLE_MASK;
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.set_enabled(enabled);
        self
    }

    pub fn fms_connected(&self) -> bool {
        self.0 & Self::FMS_MASK > 0
    }

    pub fn set_fms_connected(&mut self, fms_connected: bool) {
        if fms_connected {
            self.0 |= Self::FMS_MASK;
        } else {
            self.0 &= !Self::FMS_MASK;
        }
    }

    pub fn with_fms_connected(mut self, fms_connected: bool) -> Self {
        self.set_fms_connected(fms_connected);
        self
    }

    pub fn estopped(&self) -> bool {
        self.0 & Self::ESTOP_MASK > 0
    }

    pub fn set_estopped(&mut self, estopped: bool) {
        if estopped {
            self.0 |= Self::ESTOP_MASK;
        } else {
            self.0 &= !Self::ESTOP_MASK;
        }
    }

    pub fn with_estopped(mut self, estopped: bool) -> Self {
        self.set_estopped(estopped);
        self
    }
}

pub struct Request(u8);

impl Bytes for Request {
    fn write_bytes(&self, out: &mut Vec<u8>) {
        out.push(self.0);
    }
}

#[derive(Debug, Clone)]
pub enum Tag {
    Countdown(f32),
    Joystick {
        axes: Vec<i8>,
        buttons: Buttons,
        povs: Vec<i16>,
    },
    Date {
        microseconds: u32,
        second: u8,
        minute: u8,
        hour: u8,
        day: u8,
        /// The month with `0` representing January
        month: u8,
        /// The year with `0` representing 1900
        year: u8,
    },
    Timezone(CString),
}

impl Bytes for Tag {
    fn write_bytes(&self, out: &mut Vec<u8>) {
        match self {
            Tag::Countdown(count) => out.extend(count.to_be_bytes()),
            Tag::Joystick { axes, buttons, povs } => {
                out.push(axes.len() as u8);
                let unsigned_axes = axes
                    .iter()
                    .map(|signed| unsafe { std::mem::transmute::<i8, u8>(*signed) });

                out.extend(unsigned_axes);

                out.push(buttons.len());
                buttons.write_bytes(out);

                out.push(povs.len() as u8);
                let pov_bytes = povs
                    .iter()
                    .map(|pov| pov.to_be_bytes())
                    .flatten();

                out.extend(pov_bytes);
            },
            Tag::Date { microseconds, second, minute, hour, day, month, year } => {
                out.extend_from_slice(&microseconds.to_be_bytes());
                out.push(*second);
                out.push(*minute);
                out.push(*hour);
                out.push(*day);
                out.push(*month);
                out.push(*year);
            }
            Tag::Timezone(timezone) => out.extend_from_slice(timezone.as_bytes()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Buttons {
    count: u8,
    inner: u64,
}

impl Buttons {
    pub fn new(count: u8) -> Buttons {
        Buttons {
            count,
            inner: 0,
        }
    }

    pub fn len(&self) -> u8 {
        self.count
    }

    /// Sets the `n`th button to the given `state`.
    /// 
    /// # Panics
    /// 
    /// This function will panic if `n` is greater than the `count` given when calling [`Buttons::new`].
    pub fn set_button(&mut self, n: u8, state: bool) {
        if state {
            self.inner |= 1 << n;
        } else {
            self.inner &= !(1 << n);
        }
    }
}

impl Bytes for Buttons {
    fn write_bytes(&self, out: &mut Vec<u8>) {
        let bytes = (self.count / 8).max(1);

        out.extend_from_slice(&self.inner.to_be_bytes()[0..(bytes as usize)])
    }
}
