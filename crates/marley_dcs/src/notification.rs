//! Desktop-notification escapes in a terminal's byte stream (T7b).
//!
//! Two escapes ask a terminal for a desktop notification: OSC 9, iTerm2's (`ESC ] 9 ; body`),
//! and OSC 777, rxvt's and Ghostty's (`ESC ] 777 ; notify ; title ; body`), each ended by BEL or
//! by `ESC \`. [`NotificationScanner`] finds them read by read and only reports them: the bytes
//! go on to the parser, which ignores both.

use crate::{CAN, ESC, ST_TAIL, SUB};

/// The longest escape the scanner reads as a notification; a longer one is not reported.
pub const MAX_NOTIFICATION: usize = 4 * 1024;

/// The OSC introducer byte (`]`) that follows `ESC`.
const OSC_INTRODUCER: u8 = b']';
/// BEL, which ends an OSC as `ESC \` does.
const BEL: u8 = 0x07;

/// A desktop notification a program asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    /// The title an OSC 777 gives; an OSC 9 gives none.
    pub title: Option<String>,
    /// The text.
    pub body: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum State {
    #[default]
    Ground,
    Escape,
    Osc,
    OscEscape,
}

/// Finds OSC 9 and OSC 777 notify escapes in a byte stream, across reads.
#[derive(Debug, Default)]
pub struct NotificationScanner {
    state: State,
    osc: Vec<u8>,
    overflowed: bool,
}

impl NotificationScanner {
    /// Reads `bytes` and returns the notifications they complete, in order.
    #[must_use]
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Notification> {
        let mut found = Vec::new();
        for &byte in bytes {
            self.state = match (self.state, byte) {
                (_, CAN | SUB) => State::Ground,
                // An escape outside an OSC, or one that ends an OSC unfinished, starts over.
                (State::Ground | State::Escape | State::OscEscape, ESC) => State::Escape,
                (State::Escape | State::OscEscape, OSC_INTRODUCER) => {
                    self.osc.clear();
                    self.overflowed = false;
                    State::Osc
                }
                (State::OscEscape, ST_TAIL) | (State::Osc, BEL) => {
                    found.extend(self.finish());
                    State::Ground
                }
                (State::Osc, ESC) => State::OscEscape,
                (State::Osc, _) => {
                    self.push(byte);
                    State::Osc
                }
                (State::Ground | State::Escape | State::OscEscape, _) => State::Ground,
            };
        }
        found
    }

    fn push(&mut self, byte: u8) {
        if self.osc.len() < MAX_NOTIFICATION {
            self.osc.push(byte);
        } else {
            self.overflowed = true;
        }
    }

    fn finish(&self) -> Option<Notification> {
        if self.overflowed {
            None
        } else {
            parse(&self.osc)
        }
    }
}

/// The notification an OSC's content asks for, if it asks for one.
fn parse(osc: &[u8]) -> Option<Notification> {
    let text = String::from_utf8_lossy(osc);
    let (number, rest) = text.split_once(';')?;
    match number {
        "9" => {
            // ConEmu's OSC 9 commands, such as `9;4;…`, the progress, start with a number and
            // a `;`.
            let command = rest.split_once(';').is_some_and(|(head, _)| {
                !head.is_empty() && head.bytes().all(|byte| byte.is_ascii_digit())
            });
            (!command && !rest.is_empty()).then(|| Notification {
                title: None,
                body: clean(rest),
            })
        }
        "777" => {
            let rest = rest.strip_prefix("notify;")?;
            let (title, body) = rest.split_once(';').unwrap_or((rest, ""));
            Some(Notification {
                title: Some(clean(title)),
                body: clean(body),
            })
        }
        _ => None,
    }
}

/// `text` without its control characters, which a notification has no use for.
fn clean(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_control())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(bytes: &[u8]) -> Vec<Notification> {
        NotificationScanner::default().feed(bytes)
    }

    fn titled(title: &str, body: &str) -> Notification {
        Notification {
            title: Some(title.to_string()),
            body: body.to_string(),
        }
    }

    fn untitled(body: &str) -> Notification {
        Notification {
            title: None,
            body: body.to_string(),
        }
    }

    #[test]
    fn each_escape_is_a_notification_whichever_way_it_ends() {
        assert_eq!(
            scan(b"out\x1b]777;notify;Build;done; all green\x07more"),
            [titled("Build", "done; all green")]
        );
        assert_eq!(
            scan(b"\x1b]777;notify;Claude Code\x1b\\"),
            [titled("Claude Code", "")]
        );
        assert_eq!(scan(b"\x1b]9;Tests passed\x07"), [untitled("Tests passed")]);
        assert_eq!(scan(b"\x1b]9;42 tests\x1b\\"), [untitled("42 tests")]);
    }

    #[test]
    fn an_escape_split_across_reads_is_found_once_it_ends() {
        let mut scanner = NotificationScanner::default();
        let bytes = b"\x1b]777;notify;T;b\x1b\\";
        let mut found = Vec::new();
        for byte in bytes {
            found.extend(scanner.feed(std::slice::from_ref(byte)));
        }
        assert_eq!(found, [titled("T", "b")]);
    }

    #[test]
    fn other_escapes_are_not_notifications() {
        // ConEmu's commands and progress, an empty OSC 9, another OSC 777, a title, and an OSC
        // with no number.
        for bytes in [
            &b"\x1b]9;4;1;50\x07"[..],
            b"\x1b]9;1;100\x07",
            b"\x1b]9;\x07",
            b"\x1b]777;preexec\x07",
            b"\x1b]0;a title\x07",
            b"\x1b]notify\x07",
            b"\x1b[31m\x1bPp\x1b\\",
        ] {
            assert_eq!(scan(bytes), [], "{bytes:?}");
        }
    }

    #[test]
    fn a_cancelled_or_restarted_escape_reports_nothing_but_the_next() {
        assert_eq!(scan(b"\x1b]9;lost\x18\x1b]9;kept\x07"), [untitled("kept")]);
        assert_eq!(scan(b"\x1b]9;lost\x1a\x1b]9;kept\x07"), [untitled("kept")]);
        // An escape inside ends the OSC unfinished; a new OSC starts over, an ESC before it
        // included.
        assert_eq!(scan(b"\x1b]9;lost\x1b]9;kept\x07"), [untitled("kept")]);
        assert_eq!(
            scan(b"\x1b]9;lost\x1bx\x1b\x1b]9;kept\x07"),
            [untitled("kept")]
        );
    }

    #[test]
    fn an_escape_past_the_cap_is_dropped_and_the_next_is_read() {
        let mut bytes = b"\x1b]9;".to_vec();
        bytes.extend(std::iter::repeat_n(b'x', MAX_NOTIFICATION));
        bytes.extend_from_slice(b"\x07\x1b]9;next\x07");
        assert_eq!(scan(&bytes), [untitled("next")]);
    }

    #[test]
    fn control_characters_are_left_out_of_the_text() {
        assert_eq!(
            scan(b"\x1b]777;notify;T\x01itle;bo\x02dy\x07"),
            [titled("Title", "body")]
        );
    }
}
