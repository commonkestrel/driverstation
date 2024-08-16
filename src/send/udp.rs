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

impl Packet {
    pub fn with_sequence(mut self, sequence: u16) -> Self {
        self.sequence = sequence;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.ctrl.set_enabled(enabled);
        self
    }

    pub fn with_estopped(mut self, enabled: bool) -> Self {
        self.ctrl.set_estopped(enabled);
        self
    }

    pub fn with_fms_connected(mut self, fms_connected: bool) -> Self {
        self.ctrl.set_fms_connected(fms_connected);
        self
    }

    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.ctrl.set_mode(mode);
        self
    }

    pub fn with_reboot_roborio(mut self, reboot_roborio: bool) -> Self {
        self.req.set_reboot_roborio(reboot_roborio);
        self
    }

    pub fn with_restart_code(mut self, restart_code: bool) -> Self {
        self.req.set_restart_code(restart_code);
        self
    }

    pub fn with_alliance(mut self, alliance: Alliance) -> Self {
        self.alliance = alliance;
        self
    }

    pub fn with_tag(mut self, tag: Tag) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn with_tags(mut self, tags: Vec<Tag>) -> Self {
        self.tags = tags;
        self
    }

    pub fn extend_tags(mut self, mut tags: Vec<Tag>) -> Self {
        self.tags.append(&mut tags);
        self
    }
}

impl Default for Packet {
    fn default() -> Self {
        Packet {
            sequence: 0,
            version: 0x01,
            ctrl: Control::default(),
            req: Request::default(),
            alliance: Alliance::Red1,
            tags: Vec::new(),
        }
    }
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

pub enum UdpEvent {
    Enabled(bool),
    Estopped(bool),
    FmsConnected(bool),
    Mode(Mode),
    RebootRoborio,
    RestartCode,
    Alliance(Alliance),
    Tag(Tag),
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

impl Default for Control {
    fn default() -> Self {
        Control(0)
    }
}

struct Request(u8);

impl Request {
    const REBOOT_ROBORIO_MASK: u8 = 0x08;
    const RESTART_CODE_MASK: u8 = 0x04;
    const DS_MASK: u8 = 0x10;

    pub fn reboot_roborio(&self) -> bool {
        self.0 & Self::REBOOT_ROBORIO_MASK > 0
    }

    pub fn set_reboot_roborio(&mut self, reboot_roborio: bool) {
        if reboot_roborio {
            self.0 |= Self::REBOOT_ROBORIO_MASK;
        } else {
            self.0 &= !Self::REBOOT_ROBORIO_MASK;
        }
    }

    pub fn with_reboot_roborio(mut self, reboot_roborio: bool) -> Self {
        self.set_reboot_roborio(reboot_roborio);
        self
    }

    pub fn restart_code(&self) -> bool {
        self.0 & Self::RESTART_CODE_MASK > 0
    }

    pub fn set_restart_code(&mut self, restart_code: bool) {
        if restart_code {
            self.0 |= Self::RESTART_CODE_MASK;
        } else {
            self.0 &= !Self::RESTART_CODE_MASK;
        }
    }

    pub fn with_restart_code(mut self, restart_code: bool) -> Self {
        self.set_restart_code(restart_code);
        self
    }

    pub fn ds_connected(&self) -> bool {
        self.0 & Self::DS_MASK > 0
    }

    pub fn set_ds_connected(&mut self, ds_connected: bool) {
        if ds_connected {
            self.0 |= Self::DS_MASK;
        } else {
            self.0 &= !Self::DS_MASK;
        }
    }

    pub fn with_ds_connected(mut self, ds_connected: bool) -> Self {
        self.set_ds_connected(ds_connected);
        self
    }
}

impl Bytes for Request {
    fn write_bytes(&self, out: &mut Vec<u8>) {
        out.push(self.0);
    }
}

impl Default for Request {
    fn default() -> Self {
        Request(0).with_ds_connected(true)
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
            Tag::Joystick {
                axes,
                buttons,
                povs,
            } => {
                out.push(axes.len() as u8);
                let unsigned_axes = axes
                    .iter()
                    .map(|signed| unsafe { std::mem::transmute::<i8, u8>(*signed) });

                out.extend(unsigned_axes);

                out.push(buttons.len());
                buttons.write_bytes(out);

                out.push(povs.len() as u8);
                let pov_bytes = povs.iter().map(|pov| pov.to_be_bytes()).flatten();

                out.extend(pov_bytes);
            }
            Tag::Date {
                microseconds,
                second,
                minute,
                hour,
                day,
                month,
                year,
            } => {
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
        Buttons { count, inner: 0 }
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
