//! Marley's shell-hook frames, found in a terminal's byte stream.
//!
//! Marley's shell integration reports each prompt and command as a DCS control string,
//! `ESC P <selector> <payload> ESC \`, where the selector names the payload's encoding (`h`, `p`
//! or `q`, [`HOOK_SELECTORS`]). [`DcsScanner`] finds those frames in the bytes a PTY yields, read
//! by read, and returns them in stream order with the bytes around them, so a terminal can parse
//! the output and apply each hook exactly where it fell.
//!
//! Every byte that is not part of a Marley frame passes through unchanged, in the order it
//! came: another program's DCS, a frame cancelled by CAN or SUB, and a payload too long to be a
//! hook ([`MAX_PAYLOAD`]). The parser behind the scanner then sees them as it would have without
//! it. Decoding a frame's payload is `marley_terminal`'s.
//!
//! The crate has no dependencies, so the terminal emulator Marley carries,
//! `vendor/alacritty_terminal`, can use it without a dependency cycle.

// gate:21 runs Zed's dylint lints (`tooling/lints`) with these as errors in the Marley crates;
// Zed's crates keep them at warn (CONSTITUTION §0).
#![cfg_attr(
    dylint_lib = "lints",
    deny(
        async_block_without_await,
        blocking_io_on_foreground,
        entity_update_in_render,
        map_lookup_then_insert,
        notify_in_render,
        owned_string_into_shared,
        shared_string_from_str_literal
    )
)]

/// The selectors of Marley's hook frames: `h` (hex), `p` (plain) and `q` (C-style escapes).
///
/// A DCS with any other selector is another program's, and passes through.
pub const HOOK_SELECTORS: [u8; 3] = *b"hpq";

/// The longest payload a hook frame carries.
///
/// A frame whose payload grows past it is not Marley's, and the scanner passes it through, so a
/// stray `ESC P` cannot hide the output that follows it.
pub const MAX_PAYLOAD: usize = 64 * 1024;

const ESC: u8 = 0x1b;
/// CAN, which cancels a control string.
const CAN: u8 = 0x18;
/// SUB, which cancels a control string.
const SUB: u8 = 0x1a;
/// The DCS introducer byte (`P`) that follows `ESC`.
const DCS_INTRODUCER: u8 = b'P';
/// The `\` that completes the `ESC \` string terminator.
const ST_TAIL: u8 = b'\\';

/// One Marley hook frame: its selector and its raw, still-encoded payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDcs {
    /// The selector, the byte after `ESC P`, which names the payload's encoding.
    pub final_byte: u8,
    /// The payload bytes between the selector and the `ESC \` terminator.
    pub payload: Vec<u8>,
}

/// One item of [`DcsScanner::feed`]'s result, in stream order.
///
/// Emitting passthrough and hooks as one ordered stream keeps a coalesced `[hook]output[hook]`
/// read in its true order, so the output lands in the block the preceding hook opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DcsEvent {
    /// Bytes that are not part of a Marley hook frame, for the terminal's parser.
    Passthrough(Vec<u8>),
    /// A complete Marley hook frame.
    Hook(RawDcs),
}

/// Where the scanner is within `ESC P <selector> <payload> ESC \`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum ScanState {
    /// Outside any control string: bytes pass through.
    #[default]
    Ground,
    /// Saw `ESC`, which may introduce a DCS.
    EscSeen,
    /// Saw `ESC P`; the next byte is the selector.
    Selector,
    /// Collecting a Marley frame's payload.
    Data,
    /// Saw `ESC` inside a Marley frame's payload; `\` would end the frame.
    EscInData,
    /// Inside a DCS that is not a Marley frame: its bytes pass through until it ends.
    Foreign,
}

/// The incremental hook-frame extractor.
///
/// [`DcsScanner::feed`] consumes one read and returns the ordered [`DcsEvent`] stream it
/// produces. State persists across calls, so a frame split over several reads is reassembled,
/// and the bytes of an unfinished frame wait for the next read.
#[derive(Debug, Default)]
pub struct DcsScanner {
    state: ScanState,
    final_byte: u8,
    payload: Vec<u8>,
}

impl DcsScanner {
    /// Feeds one read, returning its [`DcsEvent`]s in stream order.
    ///
    /// Each run of bytes that are not part of a Marley frame is a [`DcsEvent::Passthrough`], and
    /// each frame completed in this read is a [`DcsEvent::Hook`]. A pending passthrough run is
    /// flushed right before each hook and at the end of the read.
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<DcsEvent> {
        let mut events = Vec::new();
        let mut pending = Vec::new();
        for &byte in bytes {
            match self.state {
                ScanState::Ground => {
                    if byte == ESC {
                        self.state = ScanState::EscSeen;
                    } else {
                        pending.push(byte);
                    }
                }
                ScanState::EscSeen => match byte {
                    DCS_INTRODUCER => self.state = ScanState::Selector,
                    ESC => pending.push(ESC),
                    _ => {
                        pending.extend_from_slice(&[ESC, byte]);
                        self.state = ScanState::Ground;
                    }
                },
                ScanState::Selector => {
                    if HOOK_SELECTORS.contains(&byte) {
                        self.final_byte = byte;
                        self.payload.clear();
                        self.state = ScanState::Data;
                    } else {
                        pending.extend_from_slice(&[ESC, DCS_INTRODUCER]);
                        self.state = foreign(byte, &mut pending);
                    }
                }
                ScanState::Data => match byte {
                    ESC => self.state = ScanState::EscInData,
                    CAN | SUB => {
                        self.abandon(&mut pending);
                        pending.push(byte);
                        self.state = ScanState::Ground;
                    }
                    _ if self.payload.len() >= MAX_PAYLOAD => {
                        self.abandon(&mut pending);
                        self.state = foreign(byte, &mut pending);
                    }
                    _ => self.payload.push(byte),
                },
                ScanState::EscInData => match byte {
                    ST_TAIL => {
                        if !pending.is_empty() {
                            events.push(DcsEvent::Passthrough(std::mem::take(&mut pending)));
                        }
                        events.push(DcsEvent::Hook(RawDcs {
                            final_byte: self.final_byte,
                            payload: std::mem::take(&mut self.payload),
                        }));
                        self.state = ScanState::Ground;
                    }
                    ESC => self.payload.push(ESC),
                    CAN | SUB => {
                        self.abandon(&mut pending);
                        pending.extend_from_slice(&[ESC, byte]);
                        self.state = ScanState::Ground;
                    }
                    _ => {
                        self.payload.extend_from_slice(&[ESC, byte]);
                        self.state = ScanState::Data;
                    }
                },
                ScanState::Foreign => self.state = foreign(byte, &mut pending),
            }
        }
        if !pending.is_empty() {
            events.push(DcsEvent::Passthrough(pending));
        }
        events
    }

    /// Gives up on the frame being collected: its bytes so far pass through as they came.
    fn abandon(&mut self, pending: &mut Vec<u8>) {
        pending.extend_from_slice(&[ESC, DCS_INTRODUCER, self.final_byte]);
        pending.append(&mut self.payload);
    }
}

/// One byte of a control string that is not a Marley frame: it passes through. The string ends
/// where the parser ends it: at `ESC`, which is held to see whether it starts a new DCS, or at
/// CAN or SUB.
fn foreign(byte: u8, pending: &mut Vec<u8>) -> ScanState {
    match byte {
        ESC => ScanState::EscSeen,
        CAN | SUB => {
            pending.push(byte);
            ScanState::Ground
        }
        _ => {
            pending.push(byte);
            ScanState::Foreign
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(selector: u8, payload: &[u8]) -> Vec<u8> {
        let mut bytes = vec![ESC, DCS_INTRODUCER, selector];
        bytes.extend_from_slice(payload);
        bytes.extend_from_slice(&[ESC, ST_TAIL]);
        bytes
    }

    fn hook(selector: u8, payload: &[u8]) -> DcsEvent {
        DcsEvent::Hook(RawDcs {
            final_byte: selector,
            payload: payload.to_vec(),
        })
    }

    fn passthrough(bytes: &[u8]) -> DcsEvent {
        DcsEvent::Passthrough(bytes.to_vec())
    }

    /// The events with each run of adjacent passthrough joined, so reads split at different
    /// points compare equal.
    fn joined(events: Vec<DcsEvent>) -> Vec<DcsEvent> {
        let mut out: Vec<DcsEvent> = Vec::new();
        for event in events {
            if let (Some(DcsEvent::Passthrough(last)), DcsEvent::Passthrough(bytes)) =
                (out.last_mut(), &event)
            {
                last.extend_from_slice(bytes);
            } else {
                out.push(event);
            }
        }
        out
    }

    #[test]
    fn plain_bytes_pass_through_and_an_empty_read_yields_nothing() {
        let mut scanner = DcsScanner::default();
        assert_eq!(scanner.feed(b"hi"), vec![passthrough(b"hi")]);
        assert_eq!(scanner.feed(b""), Vec::new());
    }

    #[test]
    fn each_marley_selector_makes_a_hook() {
        for selector in HOOK_SELECTORS {
            let mut scanner = DcsScanner::default();
            assert_eq!(
                scanner.feed(&frame(selector, b"init;id=7")),
                vec![hook(selector, b"init;id=7")]
            );
        }
    }

    #[test]
    fn hooks_and_output_keep_their_order() {
        let mut read = frame(b'p', b"preexec;command=echo hi");
        read.extend_from_slice(b"hi\r\n");
        read.extend(frame(b'p', b"precmd;exit=0"));
        let mut scanner = DcsScanner::default();
        assert_eq!(
            scanner.feed(&read),
            vec![
                hook(b'p', b"preexec;command=echo hi"),
                passthrough(b"hi\r\n"),
                hook(b'p', b"precmd;exit=0"),
            ]
        );
    }

    #[test]
    fn a_read_split_anywhere_gives_the_same_events() {
        let mut stream = b"ab".to_vec();
        stream.extend(frame(b'q', b"precmd;exit=1;pwd=/tmp"));
        stream.extend_from_slice(b"cd");
        let whole = DcsScanner::default().feed(&stream);
        for at in 0..=stream.len() {
            let (head, tail) = stream.split_at(at);
            let mut scanner = DcsScanner::default();
            let mut events = scanner.feed(head);
            events.extend(scanner.feed(tail));
            assert_eq!(joined(events), whole, "split at {at}");
        }
    }

    #[test]
    fn an_escape_that_starts_no_dcs_passes_through() {
        let mut scanner = DcsScanner::default();
        assert_eq!(
            scanner.feed(b"\x1bAB"),
            vec![passthrough(&[ESC, b'A', b'B'])]
        );
        // A doubled escape passes one on and still finds the frame after it.
        assert_eq!(
            scanner.feed(b"\x1b\x1bPpX\x1b\\"),
            vec![passthrough(&[ESC]), hook(b'p', b"X")]
        );
    }

    #[test]
    fn an_escape_inside_a_marley_payload_stays_in_it() {
        let mut scanner = DcsScanner::default();
        assert_eq!(
            scanner.feed(b"\x1bPpA\x1bX\x1b\\"),
            vec![hook(b'p', &[b'A', ESC, b'X'])]
        );
        assert_eq!(
            scanner.feed(b"\x1bPpA\x1b\x1b\\"),
            vec![hook(b'p', &[b'A', ESC])]
        );
    }

    #[test]
    fn another_programs_dcs_passes_through_whole() {
        let mut read = b"\x1bP+q544e\x1b\\".to_vec();
        read.extend(frame(b'p', b"init;id=1"));
        let mut scanner = DcsScanner::default();
        assert_eq!(
            joined(scanner.feed(&read)),
            vec![passthrough(b"\x1bP+q544e\x1b\\"), hook(b'p', b"init;id=1")]
        );
    }

    #[test]
    fn a_foreign_dcs_ends_where_the_parser_ends_it() {
        // An escape ends it, and may start a Marley frame at once.
        let mut read = b"\x1bP0;1q#0".to_vec();
        read.extend(frame(b'p', b"x"));
        let mut scanner = DcsScanner::default();
        assert_eq!(
            joined(scanner.feed(&read)),
            vec![passthrough(b"\x1bP0;1q#0"), hook(b'p', b"x")]
        );
        // CAN or SUB ends it too.
        for cancel in [CAN, SUB] {
            let mut scanner = DcsScanner::default();
            let read = [ESC, DCS_INTRODUCER, b'z', b'a', cancel, b'b'];
            assert_eq!(joined(scanner.feed(&read)), vec![passthrough(&read)]);
        }
    }

    #[test]
    fn cancel_bytes_undo_a_marley_frame_wherever_they_fall() {
        for cancel in [CAN, SUB] {
            // In the payload.
            let mut scanner = DcsScanner::default();
            let read = [ESC, DCS_INTRODUCER, b'p', b'a', cancel, b'b'];
            assert_eq!(joined(scanner.feed(&read)), vec![passthrough(&read)]);
            // Right after an escape in the payload.
            let mut scanner = DcsScanner::default();
            let read = [ESC, DCS_INTRODUCER, b'p', b'a', ESC, cancel, b'b'];
            assert_eq!(joined(scanner.feed(&read)), vec![passthrough(&read)]);
            // In place of the selector.
            let mut scanner = DcsScanner::default();
            let read = [ESC, DCS_INTRODUCER, cancel, b'b'];
            assert_eq!(joined(scanner.feed(&read)), vec![passthrough(&read)]);
        }
    }

    #[test]
    fn an_escape_in_place_of_the_selector_starts_over() {
        let mut read = vec![ESC, DCS_INTRODUCER];
        read.extend(frame(b'p', b"x"));
        let mut scanner = DcsScanner::default();
        assert_eq!(
            scanner.feed(&read),
            vec![passthrough(&[ESC, DCS_INTRODUCER]), hook(b'p', b"x")]
        );
    }

    #[test]
    fn a_payload_past_the_cap_passes_through_and_the_next_frame_is_found() {
        let mut read = vec![ESC, DCS_INTRODUCER, b'p'];
        read.resize(read.len() + MAX_PAYLOAD + 10, b'x');
        read.extend_from_slice(&[ESC, ST_TAIL]);
        let overflow = read.clone();
        read.extend(frame(b'p', b"init;id=2"));
        let mut scanner = DcsScanner::default();
        assert_eq!(
            joined(scanner.feed(&read)),
            vec![passthrough(&overflow), hook(b'p', b"init;id=2")]
        );
    }

    #[test]
    fn a_payload_at_the_cap_is_still_a_hook() {
        let payload = vec![b'x'; MAX_PAYLOAD];
        let mut scanner = DcsScanner::default();
        assert_eq!(
            scanner.feed(&frame(b'p', &payload)),
            vec![hook(b'p', &payload)]
        );
    }
}
