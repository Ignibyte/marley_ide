//! A terminal's kept output, in a file of its own, for its restore after a restart (#751).
//!
//! Each local interactive terminal's output, through `marley_dcs::history::Allowlist`, is
//! appended to `<dir>/<terminal id>.log`, the folder made for the user alone (0700) and the file
//! created readable by the user alone (0600). Only the newest [`MAX_LINES`] lines and
//! [`MAX_BYTES`] bytes are read back, cut at a line's start, so neither a long history nor one
//! long line passes the caps and no character is split; [`compact_in`] rewrites a file to that tail
//! once it has grown past them. Every function takes the folder, so a scenario's files stay its
//! own.
//!
//! Each hook that opens or closes a block adds a [`Mark`] to the kept bytes, where it fell (#764):
//! a frame of Marley's own that no replay draws. [`restore`] cuts the kept bytes at the marks, so
//! a restored terminal draws each piece and places each mark's block on the line it reached.

use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::hash::BuildHasher;
use std::io::{self, Read as _, Seek as _, SeekFrom, Write as _};
use std::path::{Path, PathBuf};

use marley_dcs::history::{Allowlist, DIM, SESSION_END};
use marley_dcs::{DcsEvent, DcsScanner, RawDcs};

use crate::identity::is_terminal_id;

/// The most lines of a terminal's output kept.
pub const MAX_LINES: usize = 5_000;

/// The most bytes of a terminal's output kept.
pub const MAX_BYTES: usize = 8 * 1024 * 1024;

/// The extension of a kept output's file.
const EXTENSION: &str = "log";

/// The selector of a mark's frame: Marley's plain hook encoding, which [`DcsScanner`] finds.
const MARK_SELECTOR: u8 = b'p';

/// What a mark's payload starts with, which no shell hook does.
const MARK_PREFIX: &str = "mark;";

/// Where a block began or ended, kept in a terminal's file at the place its hook fell (#764).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mark {
    /// A command started (`preexec`).
    Command {
        /// When, in milliseconds since the Unix epoch.
        at_ms: u64,
        /// Whether the frame carried the terminal's nonce, so the shell itself reported it.
        verified: bool,
        /// The command line.
        command: String,
    },
    /// The shell came back to its prompt (`precmd`), ending the command before it.
    Prompt {
        /// When, in milliseconds since the Unix epoch.
        at_ms: u64,
        /// The exit code of the command before it, when the shell gave one.
        exit: Option<i32>,
        /// The folder the prompt waits in.
        pwd: Option<String>,
    },
}

impl Mark {
    /// The mark's frame, `ESC P p mark;… ESC \`, its text hex-encoded so that it holds no `;`,
    /// `=` or newline.
    #[must_use]
    pub fn frame(&self) -> Vec<u8> {
        let fields = match self {
            Self::Command {
                at_ms,
                verified,
                command,
            } => vec![
                "kind=command".to_string(),
                format!("at={at_ms}"),
                format!("verified={}", u8::from(*verified)),
                format!("command={}", hex(command)),
            ],
            Self::Prompt { at_ms, exit, pwd } => ["kind=prompt".to_string(), format!("at={at_ms}")]
                .into_iter()
                .chain(exit.map(|exit| format!("exit={exit}")))
                .chain(pwd.as_deref().map(|pwd| format!("pwd={}", hex(pwd))))
                .collect(),
        };
        RawDcs {
            final_byte: MARK_SELECTOR,
            payload: format!("{MARK_PREFIX}{}", fields.join(";")).into_bytes(),
        }
        .to_bytes()
    }

    /// The mark `frame` holds, or `None` for any other frame.
    #[must_use]
    pub fn read(frame: &RawDcs) -> Option<Self> {
        if frame.final_byte != MARK_SELECTOR {
            return None;
        }
        let payload = std::str::from_utf8(&frame.payload).ok()?;
        let fields: HashMap<&str, &str> = payload
            .strip_prefix(MARK_PREFIX)?
            .split(';')
            .filter_map(|field| field.split_once('='))
            .collect();
        let at_ms = fields.get("at")?.parse().ok()?;
        match *fields.get("kind")? {
            "command" => Some(Self::Command {
                at_ms,
                verified: fields.get("verified") == Some(&"1"),
                command: unhex(fields.get("command")?)?,
            }),
            "prompt" => Some(Self::Prompt {
                at_ms,
                exit: fields.get("exit").and_then(|exit| exit.parse().ok()),
                pwd: fields.get("pwd").and_then(|pwd| unhex(pwd)),
            }),
            _ => None,
        }
    }
}

/// A piece of what a restored terminal draws: bytes for its parser, or a mark at the line they
/// reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    /// Bytes that only draw, every cell dim.
    Draw(Vec<u8>),
    /// A block's edge, at the line the bytes before it reached.
    Mark(Mark),
}

/// What draws `kept`, a terminal's earlier output, in a restored terminal, cut at its marks.
///
/// The bytes go through the allowlist again with every cell dim, since a file can come from an
/// older build or a hand; the first piece starts dim and the last ends with the scroll region and
/// the attributes reset and a new line, where the new shell's prompt goes.
#[must_use]
pub fn restore(kept: &[u8]) -> Vec<Piece> {
    let mut pieces = Vec::new();
    let mut allowlist = Allowlist::dimming();
    let mut draw = DIM.to_vec();
    for event in DcsScanner::default().feed(kept) {
        match event {
            DcsEvent::Passthrough(bytes) => allowlist.feed(&bytes, &mut draw),
            DcsEvent::Hook(frame) => {
                if let Some(mark) = Mark::read(&frame) {
                    pieces.push(Piece::Draw(std::mem::take(&mut draw)));
                    pieces.push(Piece::Mark(mark));
                }
            }
        }
    }
    draw.extend_from_slice(SESSION_END);
    pieces.push(Piece::Draw(draw));
    pieces
}

fn hex(text: &str) -> String {
    text.bytes()
        .flat_map(|byte| [byte >> 4, byte & 0x0f])
        .filter_map(|nibble| char::from_digit(u32::from(nibble), 16))
        .collect()
}

fn unhex(encoded: &str) -> Option<String> {
    let bytes = encoded
        .as_bytes()
        .chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect::<Option<Vec<u8>>>()?;
    String::from_utf8(bytes).ok()
}

/// The file holding the terminal `id`'s output under `dir`.
///
/// # Errors
///
/// When `id` is not a terminal id, which keeps any other name out of the folder's paths.
pub fn file_in(dir: &Path, id: &str) -> io::Result<PathBuf> {
    if !is_terminal_id(id) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{id:?} is not a terminal id"),
        ));
    }
    Ok(dir.join(format!("{id}.{EXTENSION}")))
}

/// Appends `bytes` to the terminal `id`'s file under `dir`, making the folder and the file for the
/// user alone when they are missing, and gives the file's length after the write.
///
/// # Errors
///
/// When the folder or the file cannot be made or written.
pub fn append_in(dir: &Path, id: &str, bytes: &[u8]) -> io::Result<u64> {
    let path = file_in(dir, id)?;
    private_folder(dir)?;
    let mut file = private_file(OpenOptions::new().create(true).append(true)).open(&path)?;
    file.write_all(bytes)?;
    Ok(file.metadata()?.len())
}

/// The terminal `id`'s kept output under `dir`, the newest [`MAX_LINES`] lines within
/// [`MAX_BYTES`] bytes, or `None` when it has none.
///
/// # Errors
///
/// When the file exists and cannot be read.
pub fn read_in(dir: &Path, id: &str) -> io::Result<Option<Vec<u8>>> {
    let path = file_in(dir, id)?;
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let length = file.metadata()?.len();
    let start = file.seek(SeekFrom::Start(length.saturating_sub(MAX_BYTES as u64)))?;
    let mut bytes = Vec::new();
    let read = file.read_to_end(&mut bytes)?;
    let kept = tail(bytes.get(..read).unwrap_or_default(), start > 0);
    Ok((!kept.is_empty()).then(|| kept.to_vec()))
}

/// Rewrites the terminal `id`'s file under `dir` to what [`read_in`] gives, through a file beside
/// it, so a crash leaves the old file or the new one.
///
/// # Errors
///
/// When the file cannot be read, written or replaced.
pub fn compact_in(dir: &Path, id: &str) -> io::Result<()> {
    let path = file_in(dir, id)?;
    let Some(kept) = read_in(dir, id)? else {
        return Ok(());
    };
    let partial = path.with_extension(format!("{EXTENSION}.partial"));
    let mut file =
        private_file(OpenOptions::new().create(true).write(true).truncate(true)).open(&partial)?;
    file.write_all(&kept)?;
    file.sync_all()?;
    fs::rename(&partial, &path)
}

/// Removes the terminal `id`'s file under `dir`; a missing one is already gone.
///
/// # Errors
///
/// When the file exists and cannot be removed.
pub fn remove_in(dir: &Path, id: &str) -> io::Result<()> {
    ignore_missing(fs::remove_file(file_in(dir, id)?))
}

/// Removes every kept output under `dir` but the terminals' in `keep`, or every one with `None`.
///
/// # Errors
///
/// When the folder exists and cannot be read, or a file cannot be removed.
pub fn retain_in<S: BuildHasher>(dir: &Path, keep: Option<&HashSet<String, S>>) -> io::Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    for entry in entries {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(id) = name.split('.').next().filter(|id| is_terminal_id(id)) else {
            continue;
        };
        if !keep.is_some_and(|keep| keep.contains(id)) {
            ignore_missing(fs::remove_file(&path))?;
        }
    }
    Ok(())
}

/// The newest [`MAX_LINES`] lines of `bytes` within [`MAX_BYTES`], cut at a line's start. With
/// `cut_head`, `bytes` began inside a line, which is dropped.
fn tail(bytes: &[u8], cut_head: bool) -> &[u8] {
    let mut start = bytes.len().saturating_sub(MAX_BYTES);
    if cut_head || start > 0 {
        start = bytes
            .get(start..)
            .and_then(|rest| rest.iter().position(|byte| *byte == b'\n'))
            .map_or(bytes.len(), |newline| start + newline + 1);
    }
    let rest = bytes.get(start..).unwrap_or_default();
    // The last line may be unfinished, such as a prompt; a final newline ends a line, not one.
    let body = rest.strip_suffix(b"\n").unwrap_or(rest);
    let cut = body
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, byte)| **byte == b'\n')
        .nth(MAX_LINES - 1)
        .map_or(0, |(index, _)| index + 1);
    rest.get(cut..).unwrap_or_default()
}

/// Makes `folder` and leaves it the user's alone.
fn private_folder(folder: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt as _, PermissionsExt as _};
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(folder)?;
        fs::set_permissions(folder, fs::Permissions::from_mode(0o700))
    }
    #[cfg(not(unix))]
    fs::create_dir_all(folder)
}

/// `options` with a new file readable by the user alone.
fn private_file(options: &mut OpenOptions) -> &mut OpenOptions {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600)
    }
    #[cfg(not(unix))]
    options
}

fn ignore_missing(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}
