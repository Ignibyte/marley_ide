//! Marley: shell hooks found in the PTY stream.
//!
//! Marley's shell integration reports each prompt and command as a DCS frame in the terminal's
//! output. The event loop runs every read through [`advance_with_hooks`], which takes Marley's
//! frames out before the parser sees them (vte drops every DCS unread anyway) and reports each
//! one as an [`Event::ShellHook`] with the grid position where it fell. The bytes before a frame
//! are parsed before its position is taken, so a command's output and the hooks around it keep
//! their order even when one read carries them all. The bytes the parser sees also go through a
//! [`NotificationScanner`], which reports each desktop-notification escape as an
//! [`Event::Notification`] (#478).
//!
//! This file is Marley's (MIT OR Apache-2.0), carried in the vendored crate; `vendor/README.md`
//! lists every Marley hunk.

use marley_dcs::{DcsEvent, DcsScanner, NotificationScanner};

use crate::event::{Event, EventListener};
use crate::grid::Dimensions;
use crate::term::{Term, TermMode};
use crate::vte::ansi;

/// One Marley shell hook frame and where it fell in the terminal's output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellHook {
    /// The frame's selector, which names its payload's encoding.
    pub final_byte: u8,
    /// The frame's raw, still-encoded payload.
    pub payload: Vec<u8>,
    /// Where the cursor was when the frame arrived, after everything before it was parsed.
    pub position: HookPosition,
}

/// The grid position a hook fell at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HookPosition {
    /// Lines dropped off the top of the history before the hook.
    pub evicted_lines: u64,
    /// The history's size at the hook.
    pub history_size: usize,
    /// The cursor's line on the screen, counted from its top.
    pub cursor_line: i32,
    /// The cursor's column.
    pub cursor_column: usize,
    /// Whether the alternate screen, which keeps no history, was showing.
    pub alt_screen: bool,
    /// How many clears the grid had counted before the hook (#639).
    pub clears: u64,
}

impl HookPosition {
    /// The position of `term`'s cursor now.
    pub fn of<T>(term: &Term<T>) -> Self {
        let grid = term.grid();
        Self {
            evicted_lines: grid.evicted_lines(),
            history_size: grid.history_size(),
            cursor_line: grid.cursor.point.line.0,
            cursor_column: grid.cursor.point.column.0,
            alt_screen: term.mode().contains(TermMode::ALT_SCREEN),
            clears: grid.marley_clears(),
        }
    }

    /// The cursor's line counted from the first line the grid ever held. It stays the same as
    /// the history scrolls and drops lines.
    pub fn absolute_line(&self) -> u64 {
        self.evicted_lines + self.history_size as u64 + self.cursor_line.max(0) as u64
    }
}

/// Parses `bytes` into `term`, taking Marley's hook frames out and reporting each one to
/// `listener` as an [`Event::ShellHook`] at the position where it fell, and each notification
/// escape as an [`Event::Notification`].
pub(crate) fn advance_with_hooks<T: EventListener, L: EventListener>(
    parser: &mut ansi::Processor,
    scanner: &mut DcsScanner,
    notifications: &mut NotificationScanner,
    term: &mut Term<T>,
    bytes: &[u8],
    listener: &L,
) {
    for event in scanner.feed(bytes) {
        match event {
            DcsEvent::Passthrough(bytes) => {
                parser.advance(term, &bytes);
                for notification in notifications.feed(&bytes) {
                    listener.send_event(Event::Notification(notification));
                }
            },
            DcsEvent::Hook(frame) => listener.send_event(Event::ShellHook(ShellHook {
                final_byte: frame.final_byte,
                payload: frame.payload,
                position: HookPosition::of(term),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::event::VoidListener;
    use crate::index::{Column, Line, Point};
    use crate::term::Config;
    use crate::term::test::TermSize;

    /// Keeps every shell hook and notification it is sent.
    #[derive(Default)]
    struct Recorder {
        hooks: RefCell<Vec<ShellHook>>,
        notifications: RefCell<Vec<marley_dcs::Notification>>,
    }

    impl EventListener for Recorder {
        fn send_event(&self, event: Event) {
            match event {
                Event::ShellHook(hook) => self.hooks.borrow_mut().push(hook),
                Event::Notification(notification) => {
                    self.notifications.borrow_mut().push(notification)
                },
                _ => {},
            }
        }
    }

    fn frame(payload: &[u8]) -> Vec<u8> {
        let mut bytes = b"\x1bPp".to_vec();
        bytes.extend_from_slice(payload);
        bytes.extend_from_slice(b"\x1b\\");
        bytes
    }

    struct Session {
        parser: ansi::Processor,
        scanner: DcsScanner,
        notifications: NotificationScanner,
        term: Term<VoidListener>,
        hooks: Recorder,
    }

    impl Session {
        fn new(columns: usize, lines: usize, history: usize) -> Self {
            let config = Config { scrolling_history: history, ..Config::default() };
            Self {
                parser: ansi::Processor::new(),
                scanner: DcsScanner::default(),
                notifications: NotificationScanner::default(),
                term: Term::new(config, &TermSize::new(columns, lines), VoidListener),
                hooks: Recorder::default(),
            }
        }

        fn read(&mut self, bytes: &[u8]) {
            advance_with_hooks(
                &mut self.parser,
                &mut self.scanner,
                &mut self.notifications,
                &mut self.term,
                bytes,
                &self.hooks,
            );
        }

        fn hooks(&self) -> Vec<ShellHook> {
            self.hooks.hooks.borrow().clone()
        }

        fn notifications(&self) -> Vec<marley_dcs::Notification> {
            self.hooks.notifications.borrow().clone()
        }

        fn line_text(&self, line: i32) -> String {
            let columns = self.term.columns();
            (0..columns)
                .map(|column| self.term.grid()[Point::new(Line(line), Column(column))].c)
                .collect::<String>()
                .trim_end()
                .to_string()
        }
    }

    #[test]
    fn one_read_with_a_whole_command_reports_both_hooks_around_its_output() {
        let mut session = Session::new(20, 5, 100);
        let mut read = frame(b"preexec;command=echo hi");
        read.extend_from_slice(b"hi\r\n");
        read.extend(frame(b"precmd;exit=0"));
        session.read(&read);

        let hooks = session.hooks();
        assert_eq!(hooks.len(), 2);
        assert_eq!(hooks[0].payload, b"preexec;command=echo hi");
        assert_eq!(hooks[1].payload, b"precmd;exit=0");
        assert_eq!(hooks[0].position.absolute_line(), 0);
        assert_eq!(hooks[1].position.absolute_line(), 1);
        assert_eq!(hooks[1].position.cursor_column, 0);
        assert_eq!(session.line_text(0), "hi");
    }

    #[test]
    fn a_frame_split_across_reads_is_reported_once_where_it_completed() {
        let mut session = Session::new(20, 5, 100);
        let read = frame(b"precmd;exit=0");
        let (head, tail) = read.split_at(6);
        session.read(&[b"x\r\n".as_slice(), head].concat());
        session.read(b"");
        assert!(session.hooks().is_empty());
        session.read(tail);
        let hooks = session.hooks();
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].payload, b"precmd;exit=0");
        assert_eq!(hooks[0].position.absolute_line(), 1);
        assert_eq!(session.line_text(0), "x");
    }

    #[test]
    fn a_notification_escape_is_reported_and_its_text_around_it_parsed() {
        let mut session = Session::new(20, 5, 100);
        session.read(b"a\x1b]777;notify;Build;done\x07b");
        assert_eq!(session.notifications(), [marley_dcs::Notification {
            title: Some("Build".to_string()),
            body: "done".to_string(),
        }]);
        assert_eq!(session.line_text(0), "ab");
    }

    #[test]
    fn another_programs_dcs_reaches_the_parser() {
        let mut session = Session::new(20, 5, 100);
        session.read(b"\x1bP+q544e\x1b\\ok");
        assert!(session.hooks().is_empty());
        assert_eq!(session.line_text(0), "ok");
    }

    #[test]
    fn positions_count_the_lines_the_history_dropped() {
        let mut session = Session::new(10, 2, 1);
        session.read(b"1\r\n2\r\n3\r\n4\r\n");
        session.read(&frame(b"precmd;exit=0"));
        let position = session.hooks()[0].position;
        assert_eq!(position.history_size, 1);
        assert!(position.evicted_lines > 0);
        assert_eq!(position.absolute_line(), 4);
        assert!(!position.alt_screen);
    }

    #[test]
    fn a_hook_on_the_alternate_screen_says_so() {
        let mut session = Session::new(10, 3, 10);
        session.read(b"\x1b[?1049h");
        session.read(&frame(b"precmd;exit=0"));
        assert!(session.hooks()[0].position.alt_screen);
    }
}
