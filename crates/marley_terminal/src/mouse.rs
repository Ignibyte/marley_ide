//! PURE — xterm mouse-tracking encoding (M17 #280, the `keys.rs`/`encode_key` symmetry): when
//! a TUI has requested tracking (DECSET 1000/1002/1003, negotiated up to SGR 1006), the app's
//! grid events encode here and stream to the PTY. The reference is the public xterm ctlseqs
//! specification; the DECSET flags themselves are parsed by `alacritty_terminal` into
//! `TermMode` — the session snapshots them into the gpui-free [`MouseModes`].

/// A gpui-free snapshot of the terminal's mouse-related mode flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MouseModes {
    /// DECSET 1000 — report press/release.
    pub click: bool,
    /// DECSET 1002 — additionally report drags (motion with a button held).
    pub drag: bool,
    /// DECSET 1003 — additionally report all motion (Marley emits drags only).
    pub motion: bool,
    /// DECSET 1006 — SGR encoding negotiated (else the legacy X10 bytes).
    pub sgr: bool,
    /// DECSET 1007 — the alternate-scroll fallback (wheel → arrow keys in alt-screen).
    pub alt_scroll: bool,
    /// The alternate screen is active (the fallback's gate).
    pub alt_screen: bool,
    /// DECSET 1 (DECCKM) — application cursor-key mode: the arrows and the alt-scroll wheel
    /// fallback emit the SS3 form (`ESC O A`) instead of the legacy CSI (`ESC [ A`) (#286).
    pub app_cursor: bool,
}

impl MouseModes {
    /// Whether ANY tracking mode is on (the report gate).
    pub fn tracking(self) -> bool {
        self.click || self.drag || self.motion
    }
}

/// A grid mouse event in the encoder's terms. Button codes: 0 left, 1 middle, 2 right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEvent {
    /// A button press.
    Press(u8),
    /// A button release.
    Release(u8),
    /// Motion with `button` held (one report per CELL — the shim throttles).
    Drag(u8),
    /// Wheel up one step (button 64).
    WheelUp,
    /// Wheel down one step (button 65).
    WheelDown,
}

/// The modifier bits xterm adds to the button code (shift 4, alt/meta 8, ctrl 16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MouseMods {
    /// Shift held. (Marley's shim BYPASSES reporting on shift — the local-selection hatch —
    /// so this bit only appears if a future caller forwards shifted events deliberately.)
    pub shift: bool,
    /// Option/alt held.
    pub alt: bool,
    /// Control held.
    pub ctrl: bool,
}

fn mod_bits(mods: MouseMods) -> u8 {
    (if mods.shift { 4 } else { 0 })
        + (if mods.alt { 8 } else { 0 })
        + (if mods.ctrl { 16 } else { 0 })
}

/// Encode one mouse event for the PTY, or `None` when the program has not asked for it
/// (M17 #280 — the full decision):
/// - no tracking mode → `None`, EXCEPT the alternate-scroll fallback: wheel in alt-screen
///   with DECSET 1007 → the arrow bytes (SS3 `ESC O A/B` under application cursor-key mode
///   (#286), else the legacy `CSI A/B` — the less/vim-without-mouse case);
/// - click-only mode (1000) reports press/release/wheel but NOT drags; drag (1002) and
///   motion (1003) add drags;
/// - SGR 1006: `ESC [ < b ; x ; y M` (press/drag/wheel) or `… m` (release), coords 1-based
///   unclamped u16;
/// - legacy X10 otherwise: `ESC [ M` + three bytes offset by 32, coords clamped at 223
///   (the byte-encoding ceiling); a release sends button code 3 per the spec;
/// - buttons: base 0/1/2, +32 for drags, 64/65 for the wheel; modifier bits +4/+8/+16.
pub fn mouse_report(
    modes: MouseModes,
    event: MouseEvent,
    col: u16,
    row: u16,
    mods: MouseMods,
) -> Option<Vec<u8>> {
    if !modes.tracking() {
        // The alternate-scroll fallback: the wheel becomes arrow keys for full-screen
        // programs that never asked for real mouse tracking.
        if modes.alt_screen && modes.alt_scroll {
            return match event {
                MouseEvent::WheelUp => Some(crate::keys::cursor_key_bytes(b'A', modes.app_cursor)),
                MouseEvent::WheelDown => {
                    Some(crate::keys::cursor_key_bytes(b'B', modes.app_cursor))
                }
                _ => None,
            };
        }
        return None;
    }
    // Click-only mode drops drags; drag/motion modes accept them.
    if matches!(event, MouseEvent::Drag(_)) && !modes.drag && !modes.motion {
        return None;
    }
    let (code, release) = match event {
        MouseEvent::Press(b) => (b + mod_bits(mods), false),
        MouseEvent::Release(b) => (b + mod_bits(mods), true),
        MouseEvent::Drag(b) => (b + 32 + mod_bits(mods), false),
        MouseEvent::WheelUp => (64 + mod_bits(mods), false),
        MouseEvent::WheelDown => (65 + mod_bits(mods), false),
    };
    if modes.sgr {
        let suffix = if release { 'm' } else { 'M' };
        return Some(format!("\x1b[<{code};{col};{row}{suffix}").into_bytes());
    }
    // Legacy X10: the release byte is code 3 (button identity is lost by design); wheel
    // sends press-only like SGR. Coordinates clamp at 223 (255 - 32).
    let x10_code = if release { 3 + mod_bits(mods) } else { code };
    let x = col.min(223) as u8;
    let y = row.min(223) as u8;
    Some(vec![0x1b, b'[', b'M', 32 + x10_code, 32 + x, 32 + y])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(sgr: bool) -> MouseModes {
        MouseModes {
            click: true,
            drag: true,
            motion: false,
            sgr,
            alt_scroll: false,
            alt_screen: false,
            app_cursor: false,
        }
    }

    // REQ-001 — the SGR matrix, byte-exact against ctlseqs (the inspect probe's vectors).
    #[test]
    fn sgr_matrix_exact_bytes() {
        let m = all(true);
        let none = MouseMods::default();
        assert_eq!(
            mouse_report(m, MouseEvent::Press(0), 5, 10, none),
            Some(b"\x1b[<0;5;10M".to_vec())
        );
        assert_eq!(
            mouse_report(m, MouseEvent::Release(0), 5, 10, none),
            Some(b"\x1b[<0;5;10m".to_vec())
        );
        assert_eq!(
            mouse_report(m, MouseEvent::Drag(0), 5, 10, none),
            Some(b"\x1b[<32;5;10M".to_vec())
        );
        assert_eq!(
            mouse_report(m, MouseEvent::WheelUp, 1, 1, none),
            Some(b"\x1b[<64;1;1M".to_vec())
        );
        assert_eq!(
            mouse_report(m, MouseEvent::WheelDown, 200, 300, none),
            Some(b"\x1b[<65;200;300M".to_vec()),
            "SGR coords are unclamped u16"
        );
        // Modifier bits: shift 4, alt 8, ctrl 16 — combined on a drag = 32+8+16.
        let ctrl_alt = MouseMods {
            shift: false,
            alt: true,
            ctrl: true,
        };
        assert_eq!(
            mouse_report(m, MouseEvent::Drag(0), 2, 3, ctrl_alt),
            Some(b"\x1b[<56;2;3M".to_vec())
        );
        let shifted = MouseMods {
            shift: true,
            alt: false,
            ctrl: false,
        };
        assert_eq!(
            mouse_report(m, MouseEvent::Press(2), 2, 3, shifted),
            Some(b"\x1b[<6;2;3M".to_vec()),
            "right button 2 + shift 4"
        );
    }

    // REQ-001 — the legacy X10 bytes: 32-offsets, the release code 3 (identity lost by
    // design), wheel 0x60, and the 223 clamp boundary.
    #[test]
    fn x10_matrix_exact_bytes() {
        let m = all(false);
        let none = MouseMods::default();
        assert_eq!(
            mouse_report(m, MouseEvent::Press(0), 5, 10, none),
            Some(vec![0x1b, b'[', b'M', 32, 37, 42])
        );
        assert_eq!(
            mouse_report(m, MouseEvent::Release(1), 5, 10, none),
            Some(vec![0x1b, b'[', b'M', 35, 37, 42]),
            "X10 release = code 3 regardless of button"
        );
        assert_eq!(
            mouse_report(m, MouseEvent::WheelUp, 1, 1, none),
            Some(vec![0x1b, b'[', b'M', 96, 33, 33])
        );
        // The clamp boundary: 223 encodes to 255; 224 clamps to the same.
        assert_eq!(
            mouse_report(m, MouseEvent::Press(0), 223, 224, none),
            Some(vec![0x1b, b'[', b'M', 32, 255, 255])
        );
    }

    // 407 enumeration — modifier bits ride the arms the matrices above left bare: RELEASE and
    // the wheel add `mod_bits` too (the `+` is load-bearing — `-`/`*` mutants change the byte),
    // in both encodings.
    #[test]
    fn modifier_bits_ride_release_and_wheel() {
        let shift = MouseMods {
            shift: true,
            alt: false,
            ctrl: false,
        };
        let ctrl = MouseMods {
            shift: false,
            alt: false,
            ctrl: true,
        };
        let alt = MouseMods {
            shift: false,
            alt: true,
            ctrl: false,
        };
        // SGR release: button 0 + shift(4) = 4, suffix 'm'.
        assert_eq!(
            mouse_report(all(true), MouseEvent::Release(0), 5, 10, shift),
            Some(b"\x1b[<4;5;10m".to_vec())
        );
        // SGR wheel: 64 + ctrl(16) = 80; 65 + alt(8) = 73.
        assert_eq!(
            mouse_report(all(true), MouseEvent::WheelUp, 2, 3, ctrl),
            Some(b"\x1b[<80;2;3M".to_vec())
        );
        assert_eq!(
            mouse_report(all(true), MouseEvent::WheelDown, 2, 3, alt),
            Some(b"\x1b[<73;2;3M".to_vec())
        );
        // X10 release: code 3 + shift(4) = 7 → byte 32 + 7 = 39.
        assert_eq!(
            mouse_report(all(false), MouseEvent::Release(1), 5, 10, shift),
            Some(vec![0x1b, b'[', b'M', 39, 37, 42])
        );
    }

    // REQ-001 — the decision edges: untracked → None; click-only drops drags; drag-mode
    // accepts; the alternate-scroll fallback needs BOTH alt_screen and alt_scroll and only
    // ever fires for the wheel.
    #[test]
    fn decision_edges_and_alt_scroll_fallback() {
        let off = MouseModes::default();
        assert_eq!(
            mouse_report(off, MouseEvent::Press(0), 1, 1, MouseMods::default()),
            None
        );
        assert_eq!(
            mouse_report(off, MouseEvent::WheelUp, 1, 1, MouseMods::default()),
            None
        );

        let click_only = MouseModes {
            click: true,
            ..Default::default()
        };
        assert_eq!(
            mouse_report(click_only, MouseEvent::Drag(0), 1, 1, MouseMods::default()),
            None,
            "1000 reports no drags"
        );
        assert!(
            mouse_report(click_only, MouseEvent::Press(0), 1, 1, MouseMods::default()).is_some()
        );

        let fallback = MouseModes {
            alt_scroll: true,
            alt_screen: true,
            ..Default::default()
        };
        assert_eq!(
            mouse_report(fallback, MouseEvent::WheelUp, 9, 9, MouseMods::default()),
            Some(b"\x1b[A".to_vec())
        );
        assert_eq!(
            mouse_report(fallback, MouseEvent::WheelDown, 9, 9, MouseMods::default()),
            Some(b"\x1b[B".to_vec())
        );
        assert_eq!(
            mouse_report(fallback, MouseEvent::Press(0), 9, 9, MouseMods::default()),
            None,
            "the fallback is wheel-only"
        );
        // Each half of the AND-gate alone is not enough.
        let only_scroll = MouseModes {
            alt_scroll: true,
            ..Default::default()
        };
        assert_eq!(
            mouse_report(only_scroll, MouseEvent::WheelUp, 1, 1, MouseMods::default()),
            None
        );
        let only_alt = MouseModes {
            alt_screen: true,
            ..Default::default()
        };
        assert_eq!(
            mouse_report(only_alt, MouseEvent::WheelUp, 1, 1, MouseMods::default()),
            None
        );
        // tracking() itself: each mode flag alone turns it on.
        assert!(
            MouseModes {
                click: true,
                ..Default::default()
            }
            .tracking()
        );
        assert!(
            MouseModes {
                drag: true,
                ..Default::default()
            }
            .tracking()
        );
        assert!(
            MouseModes {
                motion: true,
                ..Default::default()
            }
            .tracking()
        );
        assert!(!MouseModes::default().tracking());
    }

    // #286 — the alt-scroll fallback emits SS3 (ESC O A/B) under application-cursor mode, else CSI.
    #[test]
    fn alt_scroll_fallback_honors_app_cursor() {
        let base = MouseModes {
            alt_scroll: true,
            alt_screen: true,
            ..Default::default()
        };
        // app-cursor OFF → legacy CSI A/B (unchanged).
        assert_eq!(
            mouse_report(base, MouseEvent::WheelUp, 1, 1, MouseMods::default()),
            Some(b"\x1b[A".to_vec())
        );
        // app-cursor ON → SS3 O A / O B.
        let app = MouseModes {
            app_cursor: true,
            ..base
        };
        assert_eq!(
            mouse_report(app, MouseEvent::WheelUp, 1, 1, MouseMods::default()),
            Some(b"\x1bOA".to_vec())
        );
        assert_eq!(
            mouse_report(app, MouseEvent::WheelDown, 1, 1, MouseMods::default()),
            Some(b"\x1bOB".to_vec())
        );
    }
}
