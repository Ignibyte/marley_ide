//! A terminal's kept output, in a file of its own, for its restore after a restart (#751).
//!
//! Each local interactive terminal's output, through `marley_dcs::history::Allowlist`, is
//! appended to `<dir>/<terminal id>.log`, the folder made for the user alone (0700) and the file
//! created readable by the user alone (0600). Only the newest [`MAX_LINES`] lines and
//! [`MAX_BYTES`] bytes are read back, cut at a line's start, so neither a long history nor one
//! long line passes the caps and no character is split; [`compact_in`] rewrites a file to that tail
//! once it has grown past them. Every function takes the folder, so a scenario's files stay its
//! own.

use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::hash::BuildHasher;
use std::io::{self, Read as _, Seek as _, SeekFrom, Write as _};
use std::path::{Path, PathBuf};

use crate::identity::is_terminal_id;

/// The most lines of a terminal's output kept.
pub const MAX_LINES: usize = 5_000;

/// The most bytes of a terminal's output kept.
pub const MAX_BYTES: usize = 8 * 1024 * 1024;

/// The extension of a kept output's file.
const EXTENSION: &str = "log";

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
