//! Replacing a file another program reads (#755), such as Codex's `config.toml`.
//!
//! A symlink is followed so it stays a link, the new text goes to a temp file beside the real one
//! with the original's mode, the old bytes can be kept beside the path as named, and the file is
//! read again just before the rename, so an edit the user made meanwhile is never lost.
//! Synchronous: callers run it off the main thread.

use std::fs::{File, Metadata, OpenOptions};
use std::io::Write as _;
use std::os::unix::fs::{MetadataExt as _, OpenOptionsExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use util::ResultExt as _;

/// The mode of a file the write creates.
const NEW_FILE_MODE: u32 = 0o644;
/// The suffix of the kept old bytes beside the path as named.
const BACKUP_SUFFIX: &str = ".marley-backup";

/// What a replace did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Replaced {
    /// The new text went in.
    Written,
    /// The new text was the old; nothing was written.
    Unchanged,
}

/// A file's bytes and the identity they were read with.
struct Seen {
    bytes: Option<Vec<u8>>,
    identity: Option<(u64, u64, u32, u64, i64, i64)>,
}

impl Seen {
    fn of(path: &Path) -> anyhow::Result<Self> {
        match std::fs::read(path) {
            Ok(bytes) => {
                let metadata = std::fs::metadata(path)
                    .with_context(|| format!("reading {}'s details", path.display()))?;
                Ok(Self {
                    bytes: Some(bytes),
                    identity: Some(identity(&metadata)),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self {
                bytes: None,
                identity: None,
            }),
            Err(error) => Err(error).with_context(|| format!("reading {}", path.display())),
        }
    }

    fn same_as(&self, other: &Self) -> bool {
        self.bytes == other.bytes && self.identity == other.identity
    }
}

/// A file's device, inode, mode, length and modification time.
fn identity(metadata: &Metadata) -> (u64, u64, u32, u64, i64, i64) {
    (
        metadata.dev(),
        metadata.ino(),
        metadata.mode(),
        metadata.len(),
        metadata.mtime(),
        metadata.mtime_nsec(),
    )
}

/// Replaces `path`'s text with what `edit` makes of it (the empty text for a missing file).
/// A symlink is followed; the file keeps its mode, a new one gets 0644; with `backup`, the old
/// bytes go to `<path>.marley-backup` beside the path as named. Refused, writing nothing, when the
/// file changed between the read and the rename.
pub(crate) fn replace_in(
    path: &Path,
    backup: bool,
    edit: impl FnOnce(&str) -> anyhow::Result<String>,
) -> anyhow::Result<Replaced> {
    let real = real_path(path)?;
    let folder = real
        .parent()
        .with_context(|| format!("{} has no folder", real.display()))?
        .to_path_buf();
    std::fs::create_dir_all(&folder).with_context(|| format!("making {}", folder.display()))?;
    let seen = Seen::of(&real)?;
    let old = seen
        .bytes
        .as_deref()
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .unwrap_or_default();
    let new = edit(&old)?;
    if seen.bytes.as_deref() == Some(new.as_bytes()) {
        return Ok(Replaced::Unchanged);
    }
    let mode = seen
        .identity
        .map_or(NEW_FILE_MODE, |(_, _, mode, ..)| mode & 0o7777);
    let name = real
        .file_name()
        .with_context(|| format!("{} names no file", real.display()))?
        .to_string_lossy()
        .into_owned();
    let temp = folder.join(format!(".{name}.marley-{}.tmp", uuid::Uuid::new_v4()));
    let written = write_temp(&temp, new.as_bytes(), mode).and_then(|()| {
        if let (true, Some(bytes)) = (backup, seen.bytes.as_deref()) {
            write_backup(path, bytes, mode)?;
        }
        let now = Seen::of(&real)?;
        if !now.same_as(&seen) {
            bail!(
                "{} changed while Marley wrote it; nothing was written",
                path.display()
            );
        }
        std::fs::rename(&temp, &real)
            .with_context(|| format!("moving the new text into {}", real.display()))?;
        File::open(&folder)
            .and_then(|folder| folder.sync_all())
            .with_context(|| format!("syncing {}", folder.display()))
    });
    if written.is_err() && temp.exists() {
        std::fs::remove_file(&temp)
            .with_context(|| format!("removing {}", temp.display()))
            .log_err();
    }
    written.map(|()| Replaced::Written)
}

/// `path` with its symlinks followed; a path that does not exist yet stands for itself.
fn real_path(path: &Path) -> anyhow::Result<PathBuf> {
    match std::fs::canonicalize(path) {
        Ok(real) => Ok(real),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // A dangling link names a target to create; anything else is created as named.
            match std::fs::read_link(path) {
                Ok(target) if target.is_absolute() => Ok(target),
                Ok(target) => Ok(path
                    .parent()
                    .map_or_else(|| target.clone(), |parent| parent.join(&target))),
                Err(_) => Ok(path.to_path_buf()),
            }
        }
        Err(error) => Err(error).with_context(|| format!("resolving {}", path.display())),
    }
}

/// Writes `bytes` to the new file `temp` with `mode`, synced.
fn write_temp(temp: &Path, bytes: &[u8], mode: u32) -> anyhow::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(temp)
        .with_context(|| format!("making {}", temp.display()))?;
    file.write_all(bytes)
        .with_context(|| format!("writing {}", temp.display()))?;
    // The umask may have taken bits off the mode asked for.
    file.set_permissions(std::fs::Permissions::from_mode(mode))
        .with_context(|| format!("setting {}'s mode", temp.display()))?;
    file.sync_all()
        .with_context(|| format!("syncing {}", temp.display()))
}

/// Keeps the old `bytes` at `<path>.marley-backup` beside the path as named, with `mode`,
/// replacing the backup before it.
fn write_backup(path: &Path, bytes: &[u8], mode: u32) -> anyhow::Result<()> {
    let mut named = path.as_os_str().to_os_string();
    named.push(BACKUP_SUFFIX);
    let backup = PathBuf::from(named);
    std::fs::write(&backup, bytes).with_context(|| format!("writing {}", backup.display()))?;
    std::fs::set_permissions(&backup, std::fs::Permissions::from_mode(mode))
        .with_context(|| format!("setting {}'s mode", backup.display()))
}
