//! PURE, gpui-free — the keystroke → PTY-bytes encoder + the cooked/raw routing (SPEC-terminal-
//! blocks R25/R26).
//!
//! This is the VT INPUT side (symmetric with alacritty owning VT output parsing): a physical key +
//! modifiers become the bytes a terminal program expects. [`encode_key`] is what makes Ctrl-C, the
//! arrows, and full-screen programs (vim/top/less) work once the (shim) app streams its output to
//! the PTY leader. [`input_route`] decides whether a keystroke streams raw or feeds the local
//! (cooked) line editor.

/// A physical key, gpui-free. `Char` already carries the shift-resolved character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    /// A printable character (shift already applied).
    Char(char),
    /// Return / Enter.
    Enter,
    /// Backspace.
    Backspace,
    /// Tab.
    Tab,
    /// Shift-Tab (back-tab) — backward navigation in menus/forms.
    BackTab,
    /// A function key F1–F12 (the number).
    F(u8),
    /// Insert.
    Insert,
    /// Escape.
    Escape,
    /// Arrow up.
    Up,
    /// Arrow down.
    Down,
    /// Arrow left.
    Left,
    /// Arrow right.
    Right,
    /// Home.
    Home,
    /// End.
    End,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Forward delete.
    Delete,
}

/// A key event to encode: the [`KeyCode`] plus the control/alt modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyInput {
    /// The physical key.
    pub code: KeyCode,
    /// The control modifier is held.
    pub ctrl: bool,
    /// The alt/option modifier is held.
    pub alt: bool,
    /// The shift modifier is held (used for the modified-key CSI parameter; a printable `Char`
    /// already carries the shift-resolved character, so this only affects the named keys).
    pub shift: bool,
}

/// Where a keystroke goes: the local (cooked) line editor, or streamed raw to the PTY.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Edit the local prompt buffer (Marley's line editor).
    Cooked,
    /// Stream the encoded bytes straight to the PTY leader.
    Raw,
}

/// The C0 control byte for `Ctrl+key` (R25): `(c as u8) & 0x1f` — e.g. Ctrl-C → 3, Ctrl-D → 4,
/// Ctrl-Z → 26.
///
/// `& 0x1f` already masks the ASCII case bit, so there is deliberately no `to_ascii_uppercase` (it
/// would be a no-op — an unkillable/equivalent mutant).
#[must_use]
pub const fn ctrl_byte(c: char) -> u8 {
    (c as u8) & 0x1f
}

/// Encode a key event to the bytes a terminal program expects (R25): a printable char → its UTF-8
/// (ESC-prefixed when `alt`); `Ctrl+key` → the C0 control byte; named keys → their VT/CSI
/// sequences.
///
/// `app_cursor` (#286 — DECCKM) switches an UNMODIFIED cursor key from the legacy CSI form to SS3.
#[must_use]
pub fn encode_key(input: KeyInput, app_cursor: bool) -> Vec<u8> {
    // The cursor keys carry the held modifiers as an xterm CSI parameter (R25); an unmodified
    // cursor key additionally switches CSI→SS3 under application cursor-key mode (#286).
    let cursor = |final_byte| {
        csi_cursor(
            modifier_param(input.shift, input.alt, input.ctrl),
            final_byte,
            app_cursor,
        )
    };
    match input.code {
        KeyCode::Char(c) if input.ctrl => vec![ctrl_byte(c)],
        KeyCode::Char(c) if input.alt => {
            let mut bytes = vec![0x1b];
            bytes.extend(c.to_string().into_bytes());
            bytes
        }
        KeyCode::Char(c) => c.to_string().into_bytes(),
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::BackTab => vec![0x1b, b'[', b'Z'],
        KeyCode::F(n) => encode_fkey(n),
        KeyCode::Insert => vec![0x1b, b'[', b'2', b'~'],
        KeyCode::Escape => vec![0x1b],
        KeyCode::Up => cursor(b'A'),
        KeyCode::Down => cursor(b'B'),
        KeyCode::Right => cursor(b'C'),
        KeyCode::Left => cursor(b'D'),
        KeyCode::Home => cursor(b'H'),
        KeyCode::End => cursor(b'F'),
        KeyCode::PageUp => vec![0x1b, b'[', b'5', b'~'],
        KeyCode::PageDown => vec![0x1b, b'[', b'6', b'~'],
        KeyCode::Delete => vec![0x1b, b'[', b'3', b'~'],
    }
}

/// The xterm modifier code for a CSI `1;<n>` parameter (R25): 1 = no modifier, then `+Shift(1)
/// +Alt(2) +Ctrl(4)` (2 = Shift, 5 = Ctrl, 8 = all). The powers of two keep the modifiers
/// independent.
fn modifier_param(shift: bool, alt: bool, ctrl: bool) -> u8 {
    1 + u8::from(shift) + 2 * u8::from(alt) + 4 * u8::from(ctrl)
}

/// A cursor key's bytes (R25 / #286): `ESC[1;<param><final>` when a modifier is held (`param > 1`
/// — UNCHANGED even under DECCKM, per xterm), else the unmodified form via [`cursor_key_bytes`]
/// (SS3 `ESC O <final>` under application cursor-key mode, else the legacy `ESC[<final>`).
fn csi_cursor(param: u8, final_byte: u8, app_cursor: bool) -> Vec<u8> {
    if param > 1 {
        let mut v = vec![0x1b, b'[', b'1', b';'];
        v.extend(param.to_string().into_bytes());
        v.push(final_byte);
        v
    } else {
        cursor_key_bytes(final_byte, app_cursor)
    }
}

/// The bytes for one UNMODIFIED cursor key (#286 — DECCKM): the SS3 form `ESC O <final>` when
/// application cursor-key mode is active (DECSET 1 — vim's smkx, less), else the legacy CSI form
/// `ESC[<final>`. Shared by [`encode_key`]'s cursor branch AND the mouse alt-scroll fallback, so
/// the SS3-vs-CSI spec rule lives in exactly one tested place.
pub(crate) fn cursor_key_bytes(final_byte: u8, app_cursor: bool) -> Vec<u8> {
    if app_cursor {
        vec![0x1b, b'O', final_byte]
    } else {
        vec![0x1b, b'[', final_byte]
    }
}

/// The bytes for a function key (R25): F1–4 as SS3 (`ESC O P/Q/R/S`), F5–12 as CSI (`ESC[<code>~`,
/// the standard non-contiguous codes 15/17/18/19/20/21/23/24); an out-of-range `n` encodes to
/// nothing (a safe no-op).
fn encode_fkey(n: u8) -> Vec<u8> {
    match n {
        1 => vec![0x1b, b'O', b'P'],
        2 => vec![0x1b, b'O', b'Q'],
        3 => vec![0x1b, b'O', b'R'],
        4 => vec![0x1b, b'O', b'S'],
        5 => vec![0x1b, b'[', b'1', b'5', b'~'],
        6 => vec![0x1b, b'[', b'1', b'7', b'~'],
        7 => vec![0x1b, b'[', b'1', b'8', b'~'],
        8 => vec![0x1b, b'[', b'1', b'9', b'~'],
        9 => vec![0x1b, b'[', b'2', b'0', b'~'],
        10 => vec![0x1b, b'[', b'2', b'1', b'~'],
        11 => vec![0x1b, b'[', b'2', b'3', b'~'],
        12 => vec![0x1b, b'[', b'2', b'4', b'~'],
        _ => vec![],
    }
}

/// The PTY bytes for a clipboard paste (R25).
///
/// WHEN a program has bracketed paste enabled, wrap the text in the `ESC[200~`…`ESC[201~` markers
/// so a multi-line paste arrives as literal DATA — each newline is not submitted as a command.
///
/// Every embedded `ESC[201~` is stripped first, so a pasted end-marker cannot close the bracket
/// early and run the tail as commands (a paste-injection guard). WHEN bracketed paste is off, the
/// raw UTF-8 bytes.
#[must_use]
pub fn paste_bytes(text: &str, bracketed: bool) -> Vec<u8> {
    if bracketed {
        // Strip every embedded end-marker so a pasted `ESC[201~` cannot close the bracket early. A
        // SINGLE `str::replace` is not enough — removing one marker can splice its neighbours into a
        // fresh one (`ESC[20` + `ESC[201~` + `1~` → `ESC[201~`), so loop until stable. Each pass
        // strictly shrinks the string, so this terminates.
        let mut safe = text.to_string();
        while safe.contains("\x1b[201~") {
            safe = safe.replace("\x1b[201~", "");
        }
        let mut v = Vec::with_capacity(safe.len() + 12);
        v.extend_from_slice(b"\x1b[200~");
        v.extend_from_slice(safe.as_bytes());
        v.extend_from_slice(b"\x1b[201~");
        v
    } else {
        text.as_bytes().to_vec()
    }
}

/// Decide where a keystroke goes (R26).
///
/// Every key streams raw while the alternate screen is active (a full-screen program like vim/top)
/// OR a foreground command is running (an inline interactive program — an arrow-key menu, `read` —
/// owns the terminal) OR a control key is held (so Ctrl-C/D/Z reach the shell); otherwise it feeds
/// the local cooked line editor.
#[must_use]
pub const fn input_route(alt_screen: bool, ctrl: bool, command_running: bool) -> Route {
    if alt_screen || command_running || ctrl {
        Route::Raw
    } else {
        Route::Cooked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyInput {
        KeyInput {
            code,
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    /// Legacy-mode (application-cursor OFF) encode — the default for the pre-#286 goldens.
    fn enc(input: KeyInput) -> Vec<u8> {
        encode_key(input, false)
    }

    // ── R25 — printable chars + the C0 control bytes ────────────────────────────────
    #[test]
    fn encode_char_and_control() {
        // Plain char → its UTF-8; kills the ctrl/alt guards (mutant → [1] / ESC-prefixed).
        assert_eq!(enc(key(KeyCode::Char('a'))), vec![0x61]);
        // alt → ESC prefix.
        assert_eq!(
            enc(KeyInput {
                code: KeyCode::Char('a'),
                ctrl: false,
                alt: true,
                shift: false
            }),
            vec![0x1b, 0x61]
        );
        // Ctrl+letter → the C0 byte. Ctrl-C/D/Z (Ctrl-Z's 26 kills any low-bit mask slip; Ctrl-A=1
        // would be a trap that can't distinguish a `→1` mutant).
        let ctrl = |c| {
            enc(KeyInput {
                code: KeyCode::Char(c),
                ctrl: true,
                alt: false,
                shift: false,
            })
        };
        assert_eq!(ctrl('c'), vec![0x03]);
        assert_eq!(ctrl('d'), vec![0x04]);
        assert_eq!(ctrl('z'), vec![0x1a]);
        assert_eq!(ctrl_byte('c'), 0x03);
        assert_eq!(ctrl_byte('z'), 0x1a);
        // Case is masked by & 0x1f — 'C' and 'c' agree (no to_ascii_uppercase needed).
        assert_eq!(ctrl_byte('C'), ctrl_byte('c'));
        // Multibyte char → its UTF-8 (é = c3 a9).
        assert_eq!(enc(key(KeyCode::Char('\u{e9}'))), vec![0xc3, 0xa9]);
    }

    // ── R25 — named keys: exact VT/CSI bytes (goldens — cargo-mutants doesn't mutate literals) ──
    #[test]
    fn encode_named_keys() {
        assert_eq!(enc(key(KeyCode::Enter)), vec![0x0d]); // CR, not LF
        assert_eq!(enc(key(KeyCode::Backspace)), vec![0x7f]); // DEL, not BS
        assert_eq!(enc(key(KeyCode::Tab)), vec![0x09]);
        assert_eq!(enc(key(KeyCode::Escape)), vec![0x1b]);
        assert_eq!(enc(key(KeyCode::Up)), vec![0x1b, b'[', b'A']);
        assert_eq!(enc(key(KeyCode::Down)), vec![0x1b, b'[', b'B']);
        assert_eq!(enc(key(KeyCode::Right)), vec![0x1b, b'[', b'C']);
        assert_eq!(enc(key(KeyCode::Left)), vec![0x1b, b'[', b'D']);
        assert_eq!(enc(key(KeyCode::Home)), vec![0x1b, b'[', b'H']);
        assert_eq!(enc(key(KeyCode::End)), vec![0x1b, b'[', b'F']);
        assert_eq!(enc(key(KeyCode::PageUp)), vec![0x1b, b'[', b'5', b'~']);
        assert_eq!(enc(key(KeyCode::PageDown)), vec![0x1b, b'[', b'6', b'~']);
        assert_eq!(enc(key(KeyCode::Delete)), vec![0x1b, b'[', b'3', b'~']);
    }

    /// A `KeyInput` with the given modifiers (for the modified-cursor tests).
    fn key_mod(code: KeyCode, ctrl: bool, alt: bool, shift: bool) -> KeyInput {
        KeyInput {
            code,
            ctrl,
            alt,
            shift,
        }
    }

    // ── R25 (#41) — BackTab + Insert goldens ──
    #[test]
    fn encode_backtab_and_insert() {
        assert_eq!(enc(key(KeyCode::BackTab)), vec![0x1b, b'[', b'Z']); // CSI Z
        assert_eq!(enc(key(KeyCode::Insert)), vec![0x1b, b'[', b'2', b'~']);
    }

    // ── R25 (#41) — the function keys: each arm is a golden (F1-4 SS3, F5-12 CSI), out-of-range → empty ──
    #[test]
    fn encode_function_keys() {
        let cases: [(u8, Vec<u8>); 12] = [
            (1, vec![0x1b, b'O', b'P']),
            (2, vec![0x1b, b'O', b'Q']),
            (3, vec![0x1b, b'O', b'R']),
            (4, vec![0x1b, b'O', b'S']),
            (5, vec![0x1b, b'[', b'1', b'5', b'~']),
            (6, vec![0x1b, b'[', b'1', b'7', b'~']),
            (7, vec![0x1b, b'[', b'1', b'8', b'~']),
            (8, vec![0x1b, b'[', b'1', b'9', b'~']),
            (9, vec![0x1b, b'[', b'2', b'0', b'~']),
            (10, vec![0x1b, b'[', b'2', b'1', b'~']),
            (11, vec![0x1b, b'[', b'2', b'3', b'~']),
            (12, vec![0x1b, b'[', b'2', b'4', b'~']),
        ];
        for (n, expected) in cases {
            assert_eq!(enc(key(KeyCode::F(n))), expected, "F{n}");
        }
        // Out-of-range function keys encode to nothing (the `_` arm — covers the fall-through line).
        assert_eq!(enc(key(KeyCode::F(0))), Vec::<u8>::new());
        assert_eq!(enc(key(KeyCode::F(13))), Vec::<u8>::new());
    }

    // ── R25 (#41) — modifier_param: the xterm code, an 8-case truth table (binary-bijection coeffs) ──
    #[test]
    fn modifier_param_cases() {
        //                          shift, alt,  ctrl  → code
        assert_eq!(modifier_param(false, false, false), 1); // none
        assert_eq!(modifier_param(true, false, false), 2); // Shift
        assert_eq!(modifier_param(false, true, false), 3); // Alt
        assert_eq!(modifier_param(true, true, false), 4); // Shift+Alt
        assert_eq!(modifier_param(false, false, true), 5); // Ctrl
        assert_eq!(modifier_param(true, false, true), 6); // Shift+Ctrl
        assert_eq!(modifier_param(false, true, true), 7); // Alt+Ctrl
        assert_eq!(modifier_param(true, true, true), 8); // all
    }

    // ── R25 (#41) — modified cursor keys emit ESC[1;<param><final>; plain (param 1) stays legacy ──
    #[test]
    fn encode_modified_cursor() {
        // Shift-Up → ESC[1;2A (param 2).
        assert_eq!(
            enc(key_mod(KeyCode::Up, false, false, true)),
            vec![0x1b, b'[', b'1', b';', b'2', b'A']
        );
        // Ctrl-Right → ESC[1;5C (param 5).
        assert_eq!(
            enc(key_mod(KeyCode::Right, true, false, false)),
            vec![0x1b, b'[', b'1', b';', b'5', b'C']
        );
        // Alt-Left → ESC[1;3D (param 3).
        assert_eq!(
            enc(key_mod(KeyCode::Left, false, true, false)),
            vec![0x1b, b'[', b'1', b';', b'3', b'D']
        );
        // Shift-Home → ESC[1;2H.
        assert_eq!(
            enc(key_mod(KeyCode::Home, false, false, true)),
            vec![0x1b, b'[', b'1', b';', b'2', b'H']
        );
        // Plain (no modifier) → the legacy form, byte-identical to before #41 (the csi_cursor param 1
        // branch; no #33 regression).
        assert_eq!(enc(key(KeyCode::Up)), vec![0x1b, b'[', b'A']);
    }

    // ── #286 — the shared cursor_key_bytes: SS3 (ESC O X) under app-cursor, else legacy CSI (ESC [ X) ──
    #[test]
    fn cursor_key_bytes_ss3_vs_csi() {
        assert_eq!(cursor_key_bytes(b'A', true), vec![0x1b, b'O', b'A']);
        assert_eq!(cursor_key_bytes(b'B', true), vec![0x1b, b'O', b'B']);
        assert_eq!(cursor_key_bytes(b'A', false), vec![0x1b, b'[', b'A']);
        assert_eq!(cursor_key_bytes(b'C', false), vec![0x1b, b'[', b'C']);
    }

    // ── #286 REQ-001/002 — an UNMODIFIED cursor key under DECCKM → SS3, all six; app-cursor OFF → CSI ──
    #[test]
    fn app_cursor_unmodified_arrows_are_ss3() {
        let cases = [
            (KeyCode::Up, b'A'),
            (KeyCode::Down, b'B'),
            (KeyCode::Right, b'C'),
            (KeyCode::Left, b'D'),
            (KeyCode::Home, b'H'),
            (KeyCode::End, b'F'),
        ];
        for (code, final_byte) in cases {
            assert_eq!(
                encode_key(key(code), true),
                vec![0x1b, b'O', final_byte],
                "{code:?} under app-cursor must be SS3"
            );
            assert_eq!(
                encode_key(key(code), false),
                vec![0x1b, b'[', final_byte],
                "{code:?} without app-cursor stays legacy CSI"
            );
        }
    }

    // ── #286 REQ-003 — a MODIFIED cursor key stays CSI (ESC[1;<param>X) even under DECCKM ──
    #[test]
    fn app_cursor_modified_arrows_stay_csi() {
        // Ctrl-Up under app-cursor → still ESC[1;5A (param 5), NOT SS3.
        assert_eq!(
            encode_key(key_mod(KeyCode::Up, true, false, false), true),
            vec![0x1b, b'[', b'1', b';', b'5', b'A']
        );
        // Shift-End under app-cursor → still ESC[1;2F (param 2).
        assert_eq!(
            encode_key(key_mod(KeyCode::End, false, false, true), true),
            vec![0x1b, b'[', b'1', b';', b'2', b'F']
        );
    }

    // ── R26 — the routing truth table (alt_screen × ctrl × command_running). Each single-true case
    // kills dropping that operand; the all-false case kills any make-it-always-Raw + the `||`→`&&`. ──
    #[test]
    fn input_route_cases() {
        // The ONLY Cooked case: nothing streaming → the local line editor.
        assert_eq!(input_route(false, false, false), Route::Cooked);
        // Each operand alone forces Raw.
        assert_eq!(input_route(true, false, false), Route::Raw); // alt-screen (vim/top)
        assert_eq!(input_route(false, true, false), Route::Raw); // control key (signals)
        assert_eq!(input_route(false, false, true), Route::Raw); // command running (inline interactive)
        // Any combination is still Raw.
        assert_eq!(input_route(true, true, false), Route::Raw);
        assert_eq!(input_route(true, false, true), Route::Raw);
        assert_eq!(input_route(false, true, true), Route::Raw);
        assert_eq!(input_route(true, true, true), Route::Raw);
    }

    // ── R28 (#42) — paste_bytes: raw when not bracketed, wrapped when bracketed ──
    #[test]
    fn paste_bytes_plain_and_wrapped() {
        // Not bracketed → the raw UTF-8 bytes.
        assert_eq!(paste_bytes("hi", false), b"hi".to_vec());
        // Bracketed → ESC[200~ + text + ESC[201~.
        assert_eq!(paste_bytes("hi", true), b"\x1b[200~hi\x1b[201~".to_vec());
        // A multi-line paste keeps its newlines INSIDE the wrapper (literal data, not submitted).
        assert_eq!(
            paste_bytes("a\nb", true),
            b"\x1b[200~a\nb\x1b[201~".to_vec()
        );
    }

    // ── R28 (#42) — a pasted end-marker is stripped so it can't close the bracket early ──
    #[test]
    fn paste_bytes_strips_embedded_end_marker() {
        // The embedded ESC[201~ is removed → only the wrapper's closing marker remains.
        assert_eq!(
            paste_bytes("x\x1b[201~y", true),
            b"\x1b[200~xy\x1b[201~".to_vec()
        );
    }

    // ── R28 (#42) — SECURITY: a SPLIT marker must not RECONSTITUTE (a single str::replace would
    // splice `ESC[20` + `ESC[201~` + `1~` back into `ESC[201~`; the loop-until-stable strip must not).
    // MSI is blind to the strip (only whole-fn-return mutants exist), so this is a hand-written guard. ──
    #[test]
    fn paste_bytes_resists_split_marker_reconstitution() {
        let out = paste_bytes("\x1b[20\x1b[201~1~rm -rf ~", true);
        // The ONLY ESC[201~ in the output is the final wrapper marker — none in the interior body.
        assert_eq!(
            out.windows(6).filter(|w| *w == b"\x1b[201~").count(),
            1,
            "a reconstituted interior end-marker would let the paste escape the bracket"
        );
        // And it is the LAST 6 bytes (the wrapper close), not embedded.
        assert!(out.ends_with(b"\x1b[201~"));
        assert!(!out[..out.len() - 6].windows(6).any(|w| w == b"\x1b[201~"));
    }
}
