//! PURE — the stateless shell-hook codec. The incremental scanner that finds the frames is
//! `marley_dcs`'s, shared with the terminal emulator Marley carries.
//!
//! ## Wire format (clean-room, behavior-derived)
//!
//! A hook is a 7-bit DCS control string: `ESC P <selector> <encoded-payload> ESC \`. The
//! `<selector>` (the DCS "final byte") names the payload encoding — `h` = [`DcsEncoding::Hex`],
//! `p` = [`DcsEncoding::Plain`], `q` = [`DcsEncoding::AnsiCQuoted`] (see
//! [`encoding_for_dcs_terminator`]). The decoded payload is `name;key=value;…` where `name` is one
//! of `init` / `preexec` / `precmd` / `bootstrapped`.
//!
//! For `AnsiCQuoted` payloads the `name;key=value;…` split happens on UNESCAPED separators
//! BEFORE un-escaping (R24), so an escaped `\;` (or a `\xHH` decoding to `;`/`=`) stays inside
//! its field's value — the emit side (`marley_app`'s zsh rc) escapes exactly those bytes, letting
//! a `;`-containing command or `$PWD` round-trip with no truncation and no phantom fields. `Hex`
//! and `Plain` payloads retain the legacy decode-then-split order and its naive-split limitation.
//! Payload values must be UTF-8 once decoded (a raw-byte paste of invalid UTF-8 is rejected as
//! `UndecodablePayload` — a known M1 limitation).

use marley_dcs::RawDcs;

use crate::block::{ExitCode, PromptInfo, ShellSessionId};

/// The encoding of a DCS hook payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DcsEncoding {
    /// Pairwise lowercase/uppercase hex (`6c73` → `ls`).
    Hex,
    /// The bytes verbatim.
    Plain,
    /// C-style escapes: `\n` `\t` `\r` `\\` `\;` and `\xHH`.
    AnsiCQuoted,
}

/// The fields of a `Preexec` hook: the command starting to execute, and the frame's nonce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreexecValue {
    /// The command text.
    pub command: String,
    /// The nonce the frame carried, if any; Marley's scripts send the terminal's own.
    pub nonce: Option<String>,
}

/// The fields of a `Precmd` hook — the exit code of the just-finished command and the prompt
/// metadata for the *next* block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrecmdValue {
    /// The exit code of the command that just finished.
    pub exit_code: ExitCode,
    /// The prompt metadata to stage for the next block.
    pub prompt: PromptInfo,
}

/// A decoded shell bootstrap hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DcsHook {
    /// The shell announces its self-reported session id (correlation only).
    InitShell {
        /// The shell-reported id, mapped to the session's `SessionId` by `apply_hook`.
        shell_session_id: ShellSessionId,
    },
    /// A command finished: its exit code plus the next block's prompt metadata.
    Precmd(PrecmdValue),
    /// A command is starting to execute.
    Preexec(PreexecValue),
    /// The bootstrap completed, reporting whether this shell is a subshell.
    Bootstrapped {
        /// Whether the bootstrapped shell is a subshell of an already-known session.
        is_subshell: bool,
    },
    /// The shell names the file it keeps its history in, for autosuggestions (#484).
    History {
        /// The file's path, as the shell has it.
        file: String,
    },
}

/// Why [`decode_hook`] rejected a payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// The payload could not be decoded under its encoding, was not UTF-8, or a required field was
    /// missing/malformed. The codec is stateless and mutates no model (R10).
    UndecodablePayload,
    /// The payload decoded cleanly but named no known hook (R23).
    UnknownHook,
}

/// Map a DCS terminator (selector) byte to the [`DcsEncoding`] it names, or `None` for an unknown
/// terminator (R20).
#[must_use]
pub const fn encoding_for_dcs_terminator(terminator: u8) -> Option<DcsEncoding> {
    match terminator {
        b'h' => Some(DcsEncoding::Hex),
        b'p' => Some(DcsEncoding::Plain),
        b'q' => Some(DcsEncoding::AnsiCQuoted),
        _ => None,
    }
}

/// Decode `payload` under `encoding` into a [`DcsHook`] (R9/R10/R23/R24).
///
/// `AnsiCQuoted` payloads are split into `name;key=value;…` on UNESCAPED separators FIRST and
/// each piece is un-escaped separately (R24) — so `\;` (or a `\xHH` decoding to a separator)
/// stays inside its field's value. `Hex`/`Plain` payloads keep the legacy order: decode the whole
/// payload, then split the decoded text. Stateless — it mutates no model.
///
/// # Errors
///
/// A codec/UTF-8 failure or a missing/malformed required field yields
/// [`DecodeError::UndecodablePayload`]; a well-formed payload naming no known hook yields
/// [`DecodeError::UnknownHook`].
pub fn decode_hook(encoding: DcsEncoding, payload: &[u8]) -> Result<DcsHook, DecodeError> {
    let (name, fields) = match encoding {
        DcsEncoding::AnsiCQuoted => {
            let mut segments = split_unescaped(payload, b';').into_iter();
            let name_bytes = c_unescape(segments.next().unwrap_or(&[]))?;
            let name =
                String::from_utf8(name_bytes).map_err(|_| DecodeError::UndecodablePayload)?;
            let mut fields = Vec::new();
            for segment in segments {
                let Some(eq) = find_unescaped(segment, b'=') else {
                    continue; // a separator-less segment names no field (legacy-consistent)
                };
                let key = String::from_utf8(c_unescape(&segment[..eq])?)
                    .map_err(|_| DecodeError::UndecodablePayload)?;
                let value = String::from_utf8(c_unescape(&segment[eq + 1..])?)
                    .map_err(|_| DecodeError::UndecodablePayload)?;
                fields.push((key, value));
            }
            (name, fields)
        }
        DcsEncoding::Hex | DcsEncoding::Plain => {
            let decoded = match encoding {
                DcsEncoding::Hex => hex_decode(payload)?,
                _ => payload.to_vec(),
            };
            let text =
                std::str::from_utf8(&decoded).map_err(|_| DecodeError::UndecodablePayload)?;
            let (name, fields_str) = match text.split_once(';') {
                Some((name, rest)) => (name, rest),
                None => (text, ""),
            };
            let fields = fields_str
                .split(';')
                .filter_map(|part| {
                    let (k, v) = part.split_once('=')?;
                    Some((k.to_string(), v.to_string()))
                })
                .collect();
            (name.to_string(), fields)
        }
    };
    let field = |key: &str| -> Option<&str> {
        fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    };

    match name.as_str() {
        "init" => {
            let id = field("id").ok_or(DecodeError::UndecodablePayload)?;
            let id = id
                .parse::<u64>()
                .map_err(|_| DecodeError::UndecodablePayload)?;
            Ok(DcsHook::InitShell {
                shell_session_id: ShellSessionId(id),
            })
        }
        "preexec" => {
            let command = field("command").ok_or(DecodeError::UndecodablePayload)?;
            Ok(DcsHook::Preexec(PreexecValue {
                command: command.to_string(),
                nonce: field("nonce").map(String::from),
            }))
        }
        "precmd" => {
            let exit_code = match field("exit") {
                Some(s) => ExitCode(Some(
                    s.parse::<i32>()
                        .map_err(|_| DecodeError::UndecodablePayload)?,
                )),
                None => ExitCode(None),
            };
            let prompt = PromptInfo {
                pwd: field("pwd").map(String::from),
                git_branch: field("git").map(String::from),
                virtual_env: field("venv").map(String::from),
                node_version: field("node").map(String::from),
            };
            Ok(DcsHook::Precmd(PrecmdValue { exit_code, prompt }))
        }
        "bootstrapped" => {
            let subshell = field("subshell").ok_or(DecodeError::UndecodablePayload)?;
            let is_subshell = match subshell {
                "1" => true,
                "0" => false,
                _ => return Err(DecodeError::UndecodablePayload),
            };
            Ok(DcsHook::Bootstrapped { is_subshell })
        }
        "history" => {
            let file = field("file").ok_or(DecodeError::UndecodablePayload)?;
            Ok(DcsHook::History {
                file: file.to_string(),
            })
        }
        _ => Err(DecodeError::UnknownHook),
    }
}

/// Decode a single hex nibble, or `None` if `b` is not a hex digit.
const fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Pairwise hex decode; an odd length or a non-hex digit yields [`DecodeError::UndecodablePayload`].
fn hex_decode(bytes: &[u8]) -> Result<Vec<u8>, DecodeError> {
    if !bytes.len().is_multiple_of(2) {
        return Err(DecodeError::UndecodablePayload);
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.as_chunks::<2>().0 {
        let hi = hex_val(pair[0]).ok_or(DecodeError::UndecodablePayload)?;
        let lo = hex_val(pair[1]).ok_or(DecodeError::UndecodablePayload)?;
        out.push((hi << 4) + lo);
    }
    Ok(out)
}

/// C-unescape `\n` `\t` `\r` `\\` `\;` and `\xHH`; an unknown escape, a trailing `\`, or a bad
/// `\xHH` yields [`DecodeError::UndecodablePayload`].
fn c_unescape(bytes: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'\\' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        let esc = *bytes.get(i + 1).ok_or(DecodeError::UndecodablePayload)?;
        match esc {
            b'n' => out.push(b'\n'),
            b't' => out.push(b'\t'),
            b'r' => out.push(b'\r'),
            b'\\' => out.push(b'\\'),
            b';' => out.push(b';'),
            b'x' => {
                let hi = hex_val(*bytes.get(i + 2).ok_or(DecodeError::UndecodablePayload)?)
                    .ok_or(DecodeError::UndecodablePayload)?;
                let lo = hex_val(*bytes.get(i + 3).ok_or(DecodeError::UndecodablePayload)?)
                    .ok_or(DecodeError::UndecodablePayload)?;
                out.push((hi << 4) + lo);
                i += 2;
            }
            _ => return Err(DecodeError::UndecodablePayload),
        }
        i += 2;
    }
    Ok(out)
}

/// Split still-escaped bytes at each UNESCAPED `sep` — a `\` consumes the following byte (one
/// escape unit), so `\;` and every `\xHH` stay inside their segment (R24). Always returns at
/// least one (possibly empty) segment; segments remain escaped.
fn split_unescaped(bytes: &[u8], sep: u8) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2; // skip the escape unit; a trailing `\` simply ends the scan
        } else if bytes[i] == sep {
            out.push(&bytes[start..i]);
            start = i + 1;
            i += 1;
        } else {
            i += 1;
        }
    }
    // `start <= bytes.len()` always: it only ever becomes `i + 1` right after matching `sep` at
    // `i < len`, so the final (possibly empty) segment slice is in-bounds by contract.
    out.push(&bytes[start..]);
    out
}

/// The index of the first UNESCAPED `sep` in still-escaped bytes (same escape-unit rule as
/// [`split_unescaped`]), or `None` if every occurrence is escaped or absent (R24).
const fn find_unescaped(bytes: &[u8], sep: u8) -> Option<usize> {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
        } else if bytes[i] == sep {
            return Some(i);
        } else {
            i += 1;
        }
    }
    None
}

/// Decode a hook frame the scanner found into a [`DcsHook`]: its selector names the encoding
/// ([`encoding_for_dcs_terminator`]) and [`decode_hook`] reads the payload.
///
/// # Errors
///
/// A selector that names no encoding yields [`DecodeError::UndecodablePayload`]; otherwise the
/// errors of [`decode_hook`].
pub fn decode_frame(frame: &RawDcs) -> Result<DcsHook, DecodeError> {
    let encoding =
        encoding_for_dcs_terminator(frame.final_byte).ok_or(DecodeError::UndecodablePayload)?;
    decode_hook(encoding, &frame.payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── encoding_for_dcs_terminator (R20) ───────────────────────────────────
    #[test]
    fn encoding_for_dcs_terminator_table() {
        // The exact terminator→encoding table; an unknown selector (`Z`) is None. Asserting each
        // arm kills `delete match arm` (would fall to None) and the whole-fn `-> None` mutant.
        assert_eq!(encoding_for_dcs_terminator(b'h'), Some(DcsEncoding::Hex));
        assert_eq!(encoding_for_dcs_terminator(b'p'), Some(DcsEncoding::Plain));
        assert_eq!(
            encoding_for_dcs_terminator(b'q'),
            Some(DcsEncoding::AnsiCQuoted)
        );
        assert_eq!(encoding_for_dcs_terminator(b'Z'), None);
    }

    // ── decode_hook via Plain: the name-dispatch happy paths (R9/R23) ────────
    #[test]
    fn decode_plain_init() {
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"init;id=7"),
            Ok(DcsHook::InitShell {
                shell_session_id: ShellSessionId(7),
            })
        );
    }

    #[test]
    fn decode_plain_preexec() {
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"preexec;command=ls"),
            Ok(DcsHook::Preexec(PreexecValue {
                command: "ls".into(),
                nonce: None,
            }))
        );
    }

    #[test]
    fn decode_plain_precmd_all_fields_exact_exit() {
        // EXACT ExitCode(Some(3)) (the 004 trap) plus all four prompt fields. The multi-field
        // payload also kills the `k == key`→`k != key` mutant: with `!=`, field("exit") would
        // return the FIRST part whose key is not "exit" → wrong value → Err on parse.
        assert_eq!(
            decode_hook(
                DcsEncoding::Plain,
                b"precmd;exit=3;pwd=/h;git=main;venv=v;node=20"
            ),
            Ok(DcsHook::Precmd(PrecmdValue {
                exit_code: ExitCode(Some(3)),
                prompt: PromptInfo {
                    pwd: Some("/h".into()),
                    git_branch: Some("main".into()),
                    virtual_env: Some("v".into()),
                    node_version: Some("20".into()),
                },
            }))
        );
    }

    #[test]
    fn decode_plain_precmd_without_exit_is_none() {
        // The `None => ExitCode(None)` arm: a precmd that reports no exit field.
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"precmd;pwd=/h"),
            Ok(DcsHook::Precmd(PrecmdValue {
                exit_code: ExitCode(None),
                prompt: PromptInfo {
                    pwd: Some("/h".into()),
                    ..Default::default()
                },
            }))
        );
    }

    #[test]
    fn decode_plain_bootstrapped_true_and_false() {
        // subshell=1→true, =0→false (kills `delete match arm "1"`/`"0"` — each would fall to Err).
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"bootstrapped;subshell=1"),
            Ok(DcsHook::Bootstrapped { is_subshell: true })
        );
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"bootstrapped;subshell=0"),
            Ok(DcsHook::Bootstrapped { is_subshell: false })
        );
    }

    // ── the Undecodable-vs-Unknown split (R10/R23) ──────────────────────────
    #[test]
    fn unknown_hook_name_is_unknown_not_undecodable() {
        // A clean payload naming no known hook → UnknownHook (R23).
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"nope"),
            Err(DecodeError::UnknownHook)
        );
    }

    #[test]
    fn malformed_payloads_are_undecodable() {
        // Non-UTF-8, missing required field, and malformed numeric fields all → UndecodablePayload
        // (NOT UnknownHook). This is the split the critics flagged.
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"\xff"),
            Err(DecodeError::UndecodablePayload)
        );
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"init"),
            Err(DecodeError::UndecodablePayload)
        );
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"init;id=x"),
            Err(DecodeError::UndecodablePayload)
        );
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"preexec;nope=1"),
            Err(DecodeError::UndecodablePayload)
        );
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"precmd;exit=x"),
            Err(DecodeError::UndecodablePayload)
        );
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"bootstrapped;subshell=2"),
            Err(DecodeError::UndecodablePayload)
        );
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"bootstrapped"),
            Err(DecodeError::UndecodablePayload)
        );
    }

    // ── hex_val: every range arm + the exact arithmetic (R9) ─────────────────
    #[test]
    fn hex_val_each_range_and_arithmetic() {
        // Digit arm `b - b'0'`: b'0'→0 kills `Some(1)` + `-→/` (48/48=1); b'9'→9 kills `Some(0)`
        // and `-→+` (0x39+0x30=105).
        assert_eq!(hex_val(b'0'), Some(0));
        assert_eq!(hex_val(b'9'), Some(9));
        // Lowercase arm `b - b'a' + 10`: a→10, f→15. `+→-` underflows (panic), `+→*`=0, `-→+`
        // and `-→/` give 204/11 — all ≠ the asserted value.
        assert_eq!(hex_val(b'a'), Some(10));
        assert_eq!(hex_val(b'f'), Some(15));
        // Uppercase arm `b - b'A' + 10`: A→10, F→15 (kills the UPPERCASE `A..=F` arm + its arith).
        assert_eq!(hex_val(b'A'), Some(10));
        assert_eq!(hex_val(b'F'), Some(15));
        // Non-hex → None (kills the whole-fn `Some(0)`/`Some(1)` mutants on a non-digit, and the
        // arm-deletions can't reach here). `/` is one below `0`; `g` is just past `f`.
        assert_eq!(hex_val(b'/'), None);
        assert_eq!(hex_val(b'g'), None);
        assert_eq!(hex_val(b'G'), None);
    }

    // ── hex_decode: even/odd/non-hex + the (hi<<4)+lo assembly (R9) ──────────
    #[test]
    fn hex_decode_pairs_and_errors() {
        // Even-length valid hex decodes to the exact bytes — kills `delete !` (would Err on even),
        // the whole-fn vec![]/vec![0]/vec![1] defaults, and proves the byte assembly.
        assert_eq!(hex_decode(b"6c73"), Ok(vec![0x6c, 0x73]));
        // A single nonzero-high-nibble pair: (6<<4)+12 = 0x6c. `<<→>>` gives 12, `+→-` gives 84,
        // `+→*` overflows (panic) — none equal 0x6c.
        assert_eq!(hex_decode(b"6c"), Ok(vec![0x6c]));
        // Odd length and a non-hex digit both → UndecodablePayload.
        assert_eq!(hex_decode(b"abc"), Err(DecodeError::UndecodablePayload));
        assert_eq!(hex_decode(b"zz"), Err(DecodeError::UndecodablePayload));
    }

    #[test]
    fn decode_hook_hex_uppercase_real_hook() {
        // decode_hook(Hex, …) over the UPPERCASE hex of "init;id=7" (nonzero high nibbles
        // throughout: 6/7/3). Exercises the uppercase `A..=F` arm (E,B,D) end to end.
        assert_eq!(
            decode_hook(DcsEncoding::Hex, b"696E69743B69643D37"),
            Ok(DcsHook::InitShell {
                shell_session_id: ShellSessionId(7),
            })
        );
        // Lowercase hex of "init;id=5" — exercises the lowercase `a..=f` arm (e,b,d) end to end.
        assert_eq!(
            decode_hook(DcsEncoding::Hex, b"696e69743b69643d35"),
            Ok(DcsHook::InitShell {
                shell_session_id: ShellSessionId(5),
            })
        );
        // Odd-length / non-hex hex payloads surface as UndecodablePayload through decode_hook.
        assert_eq!(
            decode_hook(DcsEncoding::Hex, b"abc"),
            Err(DecodeError::UndecodablePayload)
        );
        assert_eq!(
            decode_hook(DcsEncoding::Hex, b"zz"),
            Err(DecodeError::UndecodablePayload)
        );
    }

    // ── c_unescape: each escape + the \xHH path + the error edges (R9) ───────
    #[test]
    fn c_unescape_each_escape_exact() {
        // Each escape yields its exact byte (kills every `delete match arm` — a deleted arm would
        // hit `_ => Err`).
        assert_eq!(c_unescape(br"\n"), Ok(vec![b'\n']));
        assert_eq!(c_unescape(br"\t"), Ok(vec![b'\t']));
        assert_eq!(c_unescape(br"\r"), Ok(vec![b'\r']));
        assert_eq!(c_unescape(br"\\"), Ok(vec![b'\\']));
        assert_eq!(c_unescape(br"\;"), Ok(vec![b';']));
        // \xHH with a nonzero high nibble at byte-index 0 (i+2=2 ≠ i*2=0, so the `+→*` mutant is
        // killable here): (4<<4)+1 = 0x41. `<<→>>`=1, `+→-`=63, `+→*`=64 — none equal 0x41.
        assert_eq!(c_unescape(br"\x41"), Ok(vec![0x41]));
    }

    #[test]
    fn c_unescape_plain_runs_and_loop_bounds() {
        // A multi-byte non-escape run: kills the loop-bound mutants (`<`→`>` returns empty,
        // `<`→`<=` reads out of bounds → panic, `<`→`==` returns empty), the `!=`→`==` escape
        // detector, the `i += 1` advance, and the whole-fn vec![]/vec![0]/vec![1] defaults.
        assert_eq!(c_unescape(b"abc"), Ok(vec![b'a', b'b', b'c']));
    }

    #[test]
    fn c_unescape_error_edges() {
        // An unknown escape, a trailing backslash, a truncated `\x`, and a bad `\x` digit all
        // surface UndecodablePayload (the i+2 / i+3 bounds + the `_ => Err` arm).
        assert_eq!(c_unescape(br"\q"), Err(DecodeError::UndecodablePayload));
        assert_eq!(c_unescape(b"\\"), Err(DecodeError::UndecodablePayload));
        assert_eq!(c_unescape(br"\x4"), Err(DecodeError::UndecodablePayload));
        assert_eq!(c_unescape(br"\xzz"), Err(DecodeError::UndecodablePayload));
    }

    #[test]
    fn decode_hook_cquoted_command_with_escapes() {
        // Escapes inside a command VALUE decode after the R24 unescaped-separator split:
        // A\nB\tC\rD\\E → A, LF, B, TAB, C, CR, D, backslash, E.
        assert_eq!(
            decode_hook(DcsEncoding::AnsiCQuoted, br"preexec;command=A\nB\tC\rD\\E"),
            Ok(DcsHook::Preexec(PreexecValue {
                command: "A\nB\tC\rD\\E".into(),
                nonce: None,
            }))
        );
        // \x41 placed at byte-index 16 (after "preexec;command=") — NOT index 2, so the i+2/i*2
        // and i+3/i*3 mutants are non-equivalent and killed here too.
        assert_eq!(
            decode_hook(DcsEncoding::AnsiCQuoted, br"preexec;command=\x41"),
            Ok(DcsHook::Preexec(PreexecValue {
                command: "A".into(),
                nonce: None,
            }))
        );
    }

    #[test]
    fn decode_hook_preexec_carries_its_nonce() {
        // The nonce is a field of its own after the command, in either encoding.
        assert_eq!(
            decode_hook(
                DcsEncoding::AnsiCQuoted,
                br"preexec;command=ls\; pwd;nonce=00ff"
            ),
            Ok(DcsHook::Preexec(PreexecValue {
                command: "ls; pwd".into(),
                nonce: Some("00ff".into()),
            }))
        );
        assert_eq!(
            decode_hook(DcsEncoding::Plain, b"preexec;command=ls;nonce=00ff"),
            Ok(DcsHook::Preexec(PreexecValue {
                command: "ls".into(),
                nonce: Some("00ff".into()),
            }))
        );
    }

    // ── R24 — the unescaped-separator split happens BEFORE un-escaping ──────
    #[test]
    fn decode_ansic_escaped_separators_stay_in_value() {
        // `\;` inside a command value survives the field split (the inspect-critic payload that
        // was truncated to "ls" pre-fix).
        assert_eq!(
            decode_hook(DcsEncoding::AnsiCQuoted, br"preexec;command=ls\; pwd"),
            Ok(DcsHook::Preexec(PreexecValue {
                command: "ls; pwd".into(),
                nonce: None,
            }))
        );
        // `\x3b` (an escaped `;` by hex) also stays in-value.
        assert_eq!(
            decode_hook(DcsEncoding::AnsiCQuoted, br"preexec;command=a\x3bb"),
            Ok(DcsHook::Preexec(PreexecValue {
                command: "a;b".into(),
                nonce: None,
            }))
        );
        // A `;`-containing pwd can no longer truncate or inject a phantom field: the whole
        // string stays the pwd value and git_branch remains None.
        assert_eq!(
            decode_hook(
                DcsEncoding::AnsiCQuoted,
                br"precmd;exit=0;pwd=/tmp/x\;git=evil"
            ),
            Ok(DcsHook::Precmd(PrecmdValue {
                exit_code: ExitCode(Some(0)),
                prompt: PromptInfo {
                    pwd: Some("/tmp/x;git=evil".into()),
                    ..Default::default()
                },
            }))
        );
        // Escaped-backslash-then-REAL-separator boundary: the value is `a\`; `x=1` is a field.
        assert_eq!(
            decode_hook(DcsEncoding::AnsiCQuoted, br"preexec;command=a\\;x=1"),
            Ok(DcsHook::Preexec(PreexecValue {
                command: "a\\".into(),
                nonce: None,
            }))
        );
        // `\x3d` decodes to `=` inside a value without becoming a key/value split.
        assert_eq!(
            decode_hook(DcsEncoding::AnsiCQuoted, br"precmd;pwd=a\x3db"),
            Ok(DcsHook::Precmd(PrecmdValue {
                exit_code: ExitCode(None),
                prompt: PromptInfo {
                    pwd: Some("a=b".into()),
                    ..Default::default()
                },
            }))
        );
    }

    #[test]
    fn decode_ansic_field_grammar_edges() {
        // A separator-less segment names no field and is skipped (legacy-consistent); the
        // following real field still lands.
        assert_eq!(
            decode_hook(DcsEncoding::AnsiCQuoted, br"precmd;junk;pwd=a"),
            Ok(DcsHook::Precmd(PrecmdValue {
                exit_code: ExitCode(None),
                prompt: PromptInfo {
                    pwd: Some("a".into()),
                    ..Default::default()
                },
            }))
        );
        // Duplicate keys: the FIRST wins (pins the field() find order).
        assert_eq!(
            decode_hook(DcsEncoding::AnsiCQuoted, br"precmd;pwd=first;pwd=second"),
            Ok(DcsHook::Precmd(PrecmdValue {
                exit_code: ExitCode(None),
                prompt: PromptInfo {
                    pwd: Some("first".into()),
                    ..Default::default()
                },
            }))
        );
        // An empty payload decodes to an empty name → UnknownHook (same as legacy).
        assert_eq!(
            decode_hook(DcsEncoding::AnsiCQuoted, b""),
            Err(DecodeError::UnknownHook)
        );
        // A bad escape INSIDE a key still surfaces UndecodablePayload (keys un-escape too).
        assert_eq!(
            decode_hook(DcsEncoding::AnsiCQuoted, br"precmd;ex\qit=0"),
            Err(DecodeError::UndecodablePayload)
        );
        // Raw non-UTF-8 bytes (0xff) in the NAME, a KEY, and a VALUE each surface
        // UndecodablePayload through the AnsiCQuoted path (its three from_utf8 arms).
        assert_eq!(
            decode_hook(DcsEncoding::AnsiCQuoted, b"\xff"),
            Err(DecodeError::UndecodablePayload)
        );
        assert_eq!(
            decode_hook(DcsEncoding::AnsiCQuoted, b"precmd;p\xffwd=a"),
            Err(DecodeError::UndecodablePayload)
        );
        assert_eq!(
            decode_hook(DcsEncoding::AnsiCQuoted, b"precmd;pwd=\xffa"),
            Err(DecodeError::UndecodablePayload)
        );
        // Plain keeps the LEGACY decode-then-split order: a literal `\;` in a Plain payload IS
        // split at its `;` — the command truncates to `ls\` (R24 changes AnsiCQuoted ONLY).
        assert_eq!(
            decode_hook(DcsEncoding::Plain, br"preexec;command=ls\; pwd"),
            Ok(DcsHook::Preexec(PreexecValue {
                command: "ls\\".into(),
                nonce: None,
            }))
        );
    }

    // ── the R24 raw-field scanners: split_unescaped / find_unescaped ────────
    #[test]
    fn split_unescaped_boundaries() {
        // A plain split, an escaped separator kept in-segment, and the escaped-backslash-then-
        // real-separator boundary. Kills the backslash-detector `==`→`!=`, the escape-unit
        // `i += 2` skip, the separator match, and the `start = i + 1` advance.
        assert_eq!(split_unescaped(b"a;b;c", b';'), vec![&b"a"[..], b"b", b"c"]);
        assert_eq!(split_unescaped(br"a\;b;c", b';'), vec![&br"a\;b"[..], b"c"]);
        assert_eq!(split_unescaped(br"a\\;b", b';'), vec![&br"a\\"[..], b"b"]);
        // No separator at all → one segment; the end-of-scan flush is load-bearing.
        assert_eq!(split_unescaped(b"abc", b';'), vec![&b"abc"[..]]);
        // Leading/trailing separators produce empty segments (kills off-by-one start math).
        assert_eq!(split_unescaped(b";x;", b';'), vec![&b""[..], b"x", b""]);
        // Empty input → exactly one empty segment.
        assert_eq!(split_unescaped(b"", b';'), vec![&b""[..]]);
        // A trailing backslash ends the scan without panicking; the segment keeps it.
        assert_eq!(split_unescaped(br"a\", b';'), vec![&br"a\"[..]]);
    }

    #[test]
    fn find_unescaped_positions() {
        // Skips the escaped `=` and reports the REAL one's exact index (kills Some(i)→Some(0)
        // and the escape-unit skip); None when every occurrence is escaped or absent.
        assert_eq!(find_unescaped(br"a\=b=c", b'='), Some(4));
        assert_eq!(find_unescaped(b"=x", b'='), Some(0));
        assert_eq!(find_unescaped(br"\=", b'='), None);
        assert_eq!(find_unescaped(b"abc", b'='), None);
        assert_eq!(find_unescaped(b"", b'='), None);
    }

    // ── decode_frame: a scanned frame's selector picks its encoding ─────────
    #[test]
    fn decode_frame_reads_the_selector_then_the_payload() {
        let frame = RawDcs {
            final_byte: b'p',
            payload: b"preexec;command=ls".to_vec(),
        };
        assert_eq!(
            decode_frame(&frame),
            Ok(DcsHook::Preexec(PreexecValue {
                command: "ls".into(),
                nonce: None,
            }))
        );
        let unknown = RawDcs {
            final_byte: b'z',
            payload: b"preexec;command=ls".to_vec(),
        };
        assert_eq!(decode_frame(&unknown), Err(DecodeError::UndecodablePayload));
    }
}
