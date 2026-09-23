//! The discovery-file IO (#375, cluster B) — `mcp-endpoint.json` (`{url, bearer}`) that a manager's
//! `.mcp.json` reads to reach this server.
//!
//! Deliberately NOT masked (unlike the rest of the transport/host glue): file-PERMISSION
//! correctness IS this ticket's payload, so the helpers take a dir param (the house `*_in(dir)`
//! testable-IO idiom) and are proven by tempdir metadata-mode asserts. The bearer is a secret, so
//! the file is owner-only (0600) and is removed on clean shutdown so a stale bearer never lingers.

use std::fs::{self, OpenOptions};
use std::io;
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

/// The discovery file's fixed name within the config dir.
const DISCOVERY_FILE: &str = "mcp-endpoint.json";

/// Write `json` to `<dir>/mcp-endpoint.json` owner-only (0600).
///
/// A fresh file is created 0600; a pre-existing looser file is corrected via an fchmod on the OPEN
/// HANDLE (`OpenOptions::mode` only applies on create) BEFORE the new bearer bytes are written — so
/// the fresh bearer never exists in a world-readable file (REQ-005). fchmod-ing the handle (not
/// re-resolving the path) is symlink/TOCTOU-immune.
///
/// # Errors
///
/// Any IO error opening, re-moding or writing the file.
pub fn write_discovery_file_in(dir: &Path, json: &str) -> io::Result<()> {
    let path = dir.join(DISCOVERY_FILE);
    let mut options = OpenOptions::new();
    let options = options.write(true).create(true).truncate(true);
    // Applies on create (the fresh-file path). Only Unix has file modes.
    #[cfg(unix)]
    let options = options.mode(0o600);
    let mut file = options.open(&path)?;
    // fchmod the opened fd — corrects a pre-existing looser mode, and does so on the fd we hold (immune to
    // a symlink/racing-swap of the path). Happens BEFORE the write, so the bearer only ever lands at 0600.
    #[cfg(unix)]
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    io::Write::write_all(&mut file, json.as_bytes())
}

/// Remove `<dir>/mcp-endpoint.json` (clean shutdown, REQ-006) so a stale bearer doesn't linger. Idempotent:
/// an absent file is `Ok(())`, not an error.
///
/// # Errors
///
/// Any IO error removing the file other than its absence.
pub fn remove_discovery_file_in(dir: &Path) -> io::Result<()> {
    match fs::remove_file(dir.join(DISCOVERY_FILE)) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn mode_of(path: &Path) -> u32 {
        fs::metadata(path).expect("metadata").permissions().mode() & 0o777
    }

    // REQ-004 — a fresh write creates the file 0600 with the given content.
    #[cfg(unix)]
    #[test]
    fn write_creates_owner_only_with_content() {
        let dir = tempfile::tempdir().unwrap();
        write_discovery_file_in(dir.path(), r#"{"url":"http://x","bearer":"secret"}"#).unwrap();
        let path = dir.path().join(DISCOVERY_FILE);
        assert_eq!(
            mode_of(&path),
            0o600,
            "a fresh discovery file must be owner-only"
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            r#"{"url":"http://x","bearer":"secret"}"#
        );
    }

    // REQ-005 — a pre-existing world-readable file is corrected to 0600 AND gets the new content; the new
    // bearer never lands in a >0600 file.
    #[cfg(unix)]
    #[test]
    fn write_corrects_a_preexisting_loose_mode() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(DISCOVERY_FILE);
        // Pre-create it 0644 with a STALE bearer.
        fs::write(&path, r#"{"bearer":"stale"}"#).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(mode_of(&path), 0o644);

        write_discovery_file_in(dir.path(), r#"{"bearer":"fresh"}"#).unwrap();
        assert_eq!(
            mode_of(&path),
            0o600,
            "the mode must be corrected to owner-only"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), r#"{"bearer":"fresh"}"#);
    }

    // REQ-006 — remove deletes the file and is idempotent on an absent one.
    #[test]
    fn remove_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(DISCOVERY_FILE);
        write_discovery_file_in(dir.path(), "{}").unwrap();
        assert!(path.exists());
        remove_discovery_file_in(dir.path()).unwrap();
        assert!(!path.exists(), "the file must be removed");
        // a second remove on the now-absent file is Ok, not an error.
        remove_discovery_file_in(dir.path()).unwrap();
    }

    // A non-NotFound remove error must PROPAGATE (never be swallowed as Ok) — a real failure to delete a
    // stale-bearer file must surface. Covers the `Err(err) => Err(err)` arm; a directory at the file's name
    // yields IsADirectory/PermissionDenied (both ≠ NotFound).
    #[test]
    fn remove_propagates_non_notfound_error() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(DISCOVERY_FILE)).unwrap(); // a DIRECTORY where the file name is
        let err =
            remove_discovery_file_in(dir.path()).expect_err("removing a directory must error");
        assert_ne!(
            err.kind(),
            io::ErrorKind::NotFound,
            "the error must propagate, not be swallowed"
        );
    }
}
