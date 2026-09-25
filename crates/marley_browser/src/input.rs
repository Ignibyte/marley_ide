//! Keys and the mouse as CDP's Input domain takes them.
//!
//! A key is sent as the page would get it from a keyboard: a key with text as `keyDown` with
//! that text, so the page sees keydown, keypress, input and keyup; a named key or a Ctrl or Alt
//! chord as `rawKeyDown`, which runs Blink's editing commands. gpui carries no physical key
//! code, so `code` and the Windows key code come from the US layout; `key` and the text are
//! always the ones typed. A Super chord never reaches the page.

use std::time::Instant;

use gpui::{Keystroke, Modifiers, MouseButton, NavigationDirection};
use serde_json::{Value, json};

/// The two halves of a key press, as `Input.dispatchKeyEvent` parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPress {
    /// The key going down, with its text when it types one.
    pub down: Value,
    /// The key coming up.
    pub up: Value,
}

/// The keys gpui names, as the page knows them: gpui's name, the DOM key, its code, the
/// Windows key code, and the text the key types when no Ctrl or Alt is held.
const NAMED_KEYS: &[(&str, &str, &str, u32, Option<&str>)] = &[
    // The carriage return is what makes a keypress submit a form.
    ("enter", "Enter", "Enter", 13, Some("\r")),
    ("tab", "Tab", "Tab", 9, None),
    ("backspace", "Backspace", "Backspace", 8, None),
    ("escape", "Escape", "Escape", 27, None),
    ("delete", "Delete", "Delete", 46, None),
    ("insert", "Insert", "Insert", 45, None),
    ("home", "Home", "Home", 36, None),
    ("end", "End", "End", 35, None),
    ("pageup", "PageUp", "PageUp", 33, None),
    ("pagedown", "PageDown", "PageDown", 34, None),
    ("left", "ArrowLeft", "ArrowLeft", 37, None),
    ("up", "ArrowUp", "ArrowUp", 38, None),
    ("right", "ArrowRight", "ArrowRight", 39, None),
    ("down", "ArrowDown", "ArrowDown", 40, None),
    ("space", " ", "Space", 32, Some(" ")),
    ("f1", "F1", "F1", 112, None),
    ("f2", "F2", "F2", 113, None),
    ("f3", "F3", "F3", 114, None),
    ("f4", "F4", "F4", 115, None),
    ("f5", "F5", "F5", 116, None),
    ("f6", "F6", "F6", 117, None),
    ("f7", "F7", "F7", 118, None),
    ("f8", "F8", "F8", 119, None),
    ("f9", "F9", "F9", 120, None),
    ("f10", "F10", "F10", 121, None),
    ("f11", "F11", "F11", 122, None),
    ("f12", "F12", "F12", 123, None),
];

/// The US layout's key codes for letters, by letter.
const LETTER_CODES: [&str; 26] = [
    "KeyA", "KeyB", "KeyC", "KeyD", "KeyE", "KeyF", "KeyG", "KeyH", "KeyI", "KeyJ", "KeyK", "KeyL",
    "KeyM", "KeyN", "KeyO", "KeyP", "KeyQ", "KeyR", "KeyS", "KeyT", "KeyU", "KeyV", "KeyW", "KeyX",
    "KeyY", "KeyZ",
];

/// The US layout's key codes for digits, by digit.
const DIGIT_CODES: [&str; 10] = [
    "Digit0", "Digit1", "Digit2", "Digit3", "Digit4", "Digit5", "Digit6", "Digit7", "Digit8",
    "Digit9",
];

/// The shifted digits of the US layout, from `)` on 0 to `(` on 9.
const SHIFTED_DIGITS: &str = ")!@#$%^&*(";

/// The code and Windows key code of the US key that types `character`, with or without Shift.
fn us_key(character: char) -> Option<(&'static str, u32)> {
    let lower = character.to_ascii_lowercase();
    if lower.is_ascii_lowercase() {
        let index = u32::from(lower) - u32::from('a');
        let code = LETTER_CODES.get(usize::try_from(index).ok()?)?;
        return Some((code, 65 + index));
    }
    let digit = character.to_digit(10).or_else(|| {
        SHIFTED_DIGITS
            .find(character)
            .and_then(|at| u32::try_from(at).ok())
    });
    if let Some(digit) = digit {
        let code = DIGIT_CODES.get(usize::try_from(digit).ok()?)?;
        return Some((code, 48 + digit));
    }
    Some(match character {
        ' ' => ("Space", 32),
        '-' | '_' => ("Minus", 189),
        '=' | '+' => ("Equal", 187),
        '[' | '{' => ("BracketLeft", 219),
        ']' | '}' => ("BracketRight", 221),
        '\\' | '|' => ("Backslash", 220),
        ';' | ':' => ("Semicolon", 186),
        '\'' | '"' => ("Quote", 222),
        ',' | '<' => ("Comma", 188),
        '.' | '>' => ("Period", 190),
        '/' | '?' => ("Slash", 191),
        '`' | '~' => ("Backquote", 192),
        _ => return None,
    })
}

/// The modifiers as CDP counts them: Alt 1, Ctrl 2, Meta 4, Shift 8.
#[must_use]
pub fn modifier_bits(modifiers: Modifiers) -> u32 {
    u32::from(modifiers.alt)
        | u32::from(modifiers.control) << 1
        | u32::from(modifiers.platform) << 2
        | u32::from(modifiers.shift) << 3
}

/// What `keystroke` sends to the page, or nothing: a Super chord, or a key the page has no use
/// for, such as a compose sequence's own keys, which gpui delivers without text.
#[must_use]
pub fn key_press(keystroke: &Keystroke, auto_repeat: bool) -> Option<KeyPress> {
    let modifiers = &keystroke.modifiers;
    if modifiers.platform {
        return None;
    }
    let bits = modifier_bits(*modifiers);
    let chorded = modifiers.control || modifiers.alt;
    if let Some(&(_, key, code, key_code, text)) =
        NAMED_KEYS.iter().find(|(name, ..)| *name == keystroke.key)
    {
        let text = text.filter(|_| !chorded);
        return Some(press(key, code, key_code, text, bits, auto_repeat));
    }
    if !chorded {
        let text = keystroke
            .key_char
            .as_deref()
            .filter(|text| !text.is_empty())?;
        let (code, key_code) = single_char(text).and_then(us_key).unwrap_or(("", 0));
        return Some(press(text, code, key_code, Some(text), bits, auto_repeat));
    }
    let character = single_char(&keystroke.key)?;
    let (code, key_code) = us_key(character)?;
    let key = if modifiers.shift {
        character.to_ascii_uppercase()
    } else {
        character
    }
    .to_string();
    Some(press(&key, code, key_code, None, bits, auto_repeat))
}

fn single_char(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let first = chars.next()?;
    chars.next().is_none().then_some(first)
}

fn press(
    key: &str,
    code: &str,
    key_code: u32,
    text: Option<&str>,
    modifiers: u32,
    auto_repeat: bool,
) -> KeyPress {
    let mut down = json!({
        "type": if text.is_some() { "keyDown" } else { "rawKeyDown" },
        "key": key,
        "code": code,
        "windowsVirtualKeyCode": key_code,
        "modifiers": modifiers,
        "autoRepeat": auto_repeat,
    });
    if let (Some(text), Value::Object(fields)) = (text, &mut down) {
        fields.insert("text".into(), Value::from(text));
        fields.insert("unmodifiedText".into(), Value::from(text));
    }
    let up = json!({
        "type": "keyUp",
        "key": key,
        "code": code,
        "windowsVirtualKeyCode": key_code,
        "modifiers": modifiers,
    });
    KeyPress { down, up }
}

/// A mouse button's CDP name and its bit in `buttons`.
#[must_use]
pub const fn mouse_button(button: MouseButton) -> (&'static str, u32) {
    match button {
        MouseButton::Left => ("left", 1),
        MouseButton::Right => ("right", 2),
        MouseButton::Middle => ("middle", 4),
        MouseButton::Navigate(NavigationDirection::Back) => ("back", 8),
        MouseButton::Navigate(NavigationDirection::Forward) => ("forward", 16),
    }
}

/// An `Input.dispatchMouseEvent` press, release or move at (`x`, `y`) CSS pixels, with
/// `button` the one that changed (`none` for a move) and `buttons` the ones held.
#[must_use]
pub fn mouse_event(
    kind: &str,
    (x, y): (f64, f64),
    button: &str,
    buttons: u32,
    click_count: usize,
    modifiers: u32,
) -> Value {
    json!({
        "type": kind,
        "x": x,
        "y": y,
        "button": button,
        "buttons": buttons,
        "clickCount": click_count,
        "modifiers": modifiers,
        "pointerType": "mouse",
    })
}

/// An `Input.dispatchMouseEvent` wheel turn at (`x`, `y`), by (`delta_x`, `delta_y`) CSS pixels,
/// positive right and down.
#[must_use]
pub fn wheel_event((x, y): (f64, f64), (delta_x, delta_y): (f64, f64), modifiers: u32) -> Value {
    json!({
        "type": "mouseWheel",
        "x": x,
        "y": y,
        "deltaX": delta_x,
        "deltaY": delta_y,
        "modifiers": modifiers,
        "pointerType": "mouse",
    })
}

/// Logs, at debug level, how long an input sent at `sent` took to reach a frame: the spike's
/// latency number, on with `ZED_LOG=marley_browser=debug`.
pub fn log_latency(sent: Instant) {
    log::debug!(
        "input to frame: {:.1} ms",
        sent.elapsed().as_secs_f64() * 1000.0
    );
}
