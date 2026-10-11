//! What of a terminal's output is kept, to draw again after a restart (#751).
//!
//! [`Allowlist`] reads a terminal's bytes read by read and keeps only what draws: text, CR, LF, BS
//! and HT; SGR; cursor motion; erase, insert and delete; scrolling and the scroll region; saving
//! and restoring the cursor. Everything else is dropped, among it every query a terminal answers
//! (device status and cursor reports, device attributes, mode and setting reports, the colour
//! queries), every mode change, every OSC (titles, notifications, the clipboard, links), DCS, SOS,
//! PM, APC and BEL. Bytes it kept can be fed to a new terminal with nothing answering into its
//! shell and nothing raised but drawing. While a program shows the alternate screen nothing is
//! kept, so a full-screen program's screen never comes back.
//!
//! [`replay`] gives kept bytes the form a restored terminal draws: filtered again, since a file
//! can come from an older build or a hand, every cell dim, and the attributes and the scroll
//! region reset at the end.

use crate::{CAN, ESC, ST_TAIL, SUB};

/// The longest run of a control sequence's parameters the allowlist keeps; a longer sequence is
/// dropped.
pub const MAX_PARAMETERS: usize = 64;

/// BEL, which ends an OSC.
const BEL: u8 = 0x07;
/// BS.
const BACKSPACE: u8 = 0x08;
/// SGR 2, dim, which [`Allowlist::dimming`] puts after every SGR.
const DIM: &[u8] = b"\x1b[2m";
/// The end of a session's output.
///
/// The cursor saved, the scroll region reset (which homes the cursor), the cursor restored, the
/// attributes reset, and a new line. A replay ends with it, for the new shell's prompt, and it goes
/// between two sessions' output in a kept file, so the next replay draws them as the terminal did.
pub const SESSION_END: &[u8] = b"\x1b7\x1b[r\x1b8\x1b[0m\r\n";
/// The DEC private modes that switch to the alternate screen.
const ALTERNATE_SCREEN_MODES: [&[u8]; 3] = [b"47", b"1047", b"1049"];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum State {
    #[default]
    Ground,
    Escape,
    /// An escape with intermediate bytes (`ESC ( B`, `ESC # 8`), dropped through its final byte.
    EscapeIntermediate,
    Csi,
    /// An OSC, dropped through BEL or ST.
    Osc,
    OscEscape,
    /// A DCS, SOS, PM or APC, dropped through ST.
    ControlString,
    ControlStringEscape,
}

/// Keeps what draws of a terminal's output, read by read.
#[derive(Debug, Default)]
pub struct Allowlist {
    state: State,
    parameters: Vec<u8>,
    overflowed: bool,
    alternate_screen: bool,
    dim: bool,
}

impl Allowlist {
    /// An allowlist that dims every cell it keeps: each SGR it keeps is followed by SGR 2.
    #[must_use]
    pub fn dimming() -> Self {
        Self {
            dim: true,
            ..Self::default()
        }
    }

    /// Reads `bytes` and appends to `kept` what of them draws. A sequence split across reads is
    /// kept whole once its end arrives.
    pub fn feed(&mut self, bytes: &[u8], kept: &mut Vec<u8>) {
        for &byte in bytes {
            self.state = self.step(self.state, byte, kept);
        }
    }

    fn step(&mut self, state: State, byte: u8, kept: &mut Vec<u8>) -> State {
        match state {
            State::Ground => self.ground(byte, kept),
            State::Escape => self.escape(byte, kept),
            State::EscapeIntermediate => match byte {
                CAN | SUB | 0x30..=0x7e => State::Ground,
                ESC => State::Escape,
                _ => State::EscapeIntermediate,
            },
            State::Csi => match byte {
                CAN | SUB | 0x7f..=0xff => State::Ground,
                ESC => State::Escape,
                0x20..=0x3f => {
                    if self.parameters.len() < MAX_PARAMETERS {
                        self.parameters.push(byte);
                    } else {
                        self.overflowed = true;
                    }
                    State::Csi
                }
                0x40..=0x7e => {
                    self.finish_csi(byte, kept);
                    State::Ground
                }
                // A control inside a sequence acts at once, as a terminal's parser runs it.
                0x00..=0x1f => {
                    if !is_dropped_control(byte) {
                        self.keep(&[byte], kept);
                    }
                    State::Csi
                }
            },
            State::Osc => match byte {
                BEL | CAN | SUB => State::Ground,
                ESC => State::OscEscape,
                _ => State::Osc,
            },
            State::ControlString => match byte {
                CAN | SUB => State::Ground,
                ESC => State::ControlStringEscape,
                _ => State::ControlString,
            },
            // An escape that is not ST ends the string unfinished and starts a sequence of its own.
            State::OscEscape | State::ControlStringEscape => match byte {
                ST_TAIL => State::Ground,
                _ => self.escape(byte, kept),
            },
        }
    }

    fn ground(&self, byte: u8, kept: &mut Vec<u8>) -> State {
        if byte == ESC {
            return State::Escape;
        }
        if !is_dropped_control(byte) {
            self.keep(&[byte], kept);
        }
        State::Ground
    }

    fn escape(&mut self, byte: u8, kept: &mut Vec<u8>) -> State {
        match byte {
            b'[' => {
                self.parameters.clear();
                self.overflowed = false;
                State::Csi
            }
            b']' => State::Osc,
            b'P' | b'X' | b'^' | b'_' => State::ControlString,
            ESC => State::Escape,
            0x20..=0x2f => State::EscapeIntermediate,
            // Saving and restoring the cursor, index, next line and reverse index only move it.
            b'7' | b'8' | b'D' | b'E' | b'M' => {
                self.keep(&[ESC, byte], kept);
                State::Ground
            }
            _ => State::Ground,
        }
    }

    fn finish_csi(&mut self, final_byte: u8, kept: &mut Vec<u8>) {
        if self.overflowed {
            return;
        }
        let parameters = std::mem::take(&mut self.parameters);
        let private = parameters
            .first()
            .is_some_and(|byte| (b'<'..=b'?').contains(byte));
        let intermediate = parameters.iter().any(|byte| (0x20..=0x2f).contains(byte));
        if private {
            if parameters.first() == Some(&b'?')
                && !intermediate
                && matches!(final_byte, b'h' | b'l')
                && parameters
                    .get(1..)
                    .unwrap_or_default()
                    .split(|byte| *byte == b';')
                    .any(|mode| ALTERNATE_SCREEN_MODES.contains(&mode))
            {
                self.alternate_screen = final_byte == b'h';
            }
        } else if !intermediate && draws(final_byte) {
            let mut sequence = Vec::with_capacity(parameters.len() + 3);
            sequence.extend_from_slice(&[ESC, b'[']);
            sequence.extend_from_slice(&parameters);
            sequence.push(final_byte);
            self.keep(&sequence, kept);
            if self.dim && final_byte == b'm' {
                self.keep(DIM, kept);
            }
        }
        self.parameters = parameters;
        self.parameters.clear();
    }

    fn keep(&self, bytes: &[u8], kept: &mut Vec<u8>) {
        if !self.alternate_screen {
            kept.extend_from_slice(bytes);
        }
    }
}

/// Whether `byte` is a control the allowlist drops: every C0 control but CR, LF, BS and HT, and
/// DEL.
const fn is_dropped_control(byte: u8) -> bool {
    matches!(byte, 0x00..=0x1f | 0x7f) && !matches!(byte, b'\r' | b'\n' | BACKSPACE | b'\t')
}

/// Whether a control sequence ending in `final_byte`, with no private marker and no intermediate
/// bytes, only draws: SGR, cursor motion, erase, insert and delete, scrolling, the scroll region,
/// and saving and restoring the cursor.
const fn draws(final_byte: u8) -> bool {
    matches!(
        final_byte,
        b'm' | b'A'
            ..=b'H'
                | b'f'
                | b'd'
                | b'e'
                | b'a'
                | b'`'
                | b'J'
                | b'K'
                | b'X'
                | b'P'
                | b'@'
                | b'L'
                | b'M'
                | b'S'
                | b'T'
                | b'r'
                | b's'
                | b'u'
    )
}

/// The bytes that draw `kept`, a terminal's earlier output, in a restored terminal.
///
/// They go through the allowlist again with every cell dim, and end with the scroll region and
/// the attributes reset and a new line, where the new shell's prompt goes.
#[must_use]
pub fn replay(kept: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(kept.len() + DIM.len() + SESSION_END.len());
    bytes.extend_from_slice(DIM);
    Allowlist::dimming().feed(kept, &mut bytes);
    bytes.extend_from_slice(SESSION_END);
    bytes
}
