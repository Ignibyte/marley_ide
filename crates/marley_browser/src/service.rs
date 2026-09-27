//! Marley's Chromium as transient user units (plan D16), one per project since #507.
//!
//! A project's first Browser tab starts its Chromium with `systemd-run --user`, on a profile in
//! the project's folder under Marley's data directory, so each project keeps its own cookies and
//! logins, and an e2e run's Marley starts its own. The unit runs the browser binary itself, not a
//! distribution's launcher, which would add the user's `chromium-flags.conf`. Chromium runs
//! headless and puts its debugging endpoint on a port it picks, which it writes to
//! `DevToolsActivePort` in the profile, where every client finds it. The unit outlives Marley, so
//! a restored tab finds its page, and stops when its project is removed from Marley, or at logout.

use std::ffi::{OsStr, OsString};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context as _;
use sha2::{Digest as _, Sha256};

/// The environment variable that names the browser binary; when it is set, nothing else is
/// tried.
pub const BINARY_OVERRIDE: &str = "MARLEY_CHROMIUM";

/// The browser binaries that distributions wrap in a `/usr/bin/chromium` launcher: Arch's and
/// Debian's.
const BINARY_PATHS: &[&str] = &["/usr/lib/chromium/chromium"];

/// The names distributions install the browser under, looked up on the PATH.
const BINARY_NAMES: &[&str] = &["chromium", "chromium-browser"];

/// The file Chromium writes its debugging endpoint to, in its profile.
const ENDPOINT_FILE: &str = "DevToolsActivePort";

/// The file in a project's folder that names the project (#507).
const PROJECT_FILE: &str = "project.json";

/// The key of the project whose main worktree paths are `paths` and whose host, for a remote
/// project, is `host` (#507).
///
/// It is the first sixteen hex digits of the SHA-256 of the paths, sorted, each followed by a
/// newline, then of the host and a newline. Nothing in it changes at a restart, so the project's
/// profile stays its own.
#[must_use]
pub fn project_key(paths: &[PathBuf], host: Option<&str>) -> String {
    let mut sorted: Vec<&PathBuf> = paths.iter().collect();
    sorted.sort();
    let mut hasher = Sha256::new();
    for path in sorted {
        hasher.update(path.as_os_str().as_encoded_bytes());
        hasher.update(b"\n");
    }
    if let Some(host) = host {
        hasher.update(host.as_bytes());
        hasher.update(b"\n");
    }
    hex::encode(&hasher.finalize()[..8])
}

/// The folder of the project `key` in the Marley data directory `data`: its profile and its
/// `project.json`.
#[must_use]
pub fn project_dir_in(data: &Path, key: &str) -> PathBuf {
    data.join("browser").join("projects").join(key)
}

/// The Chromium profile in the project folder `project_dir`.
#[must_use]
pub fn profile_in(project_dir: &Path) -> PathBuf {
    project_dir.join("profile")
}

/// The one profile every build before #507 used, in the Marley data directory `data`.
#[must_use]
pub fn legacy_profile_in(data: &Path) -> PathBuf {
    data.join("browser").join("profile")
}

/// Makes the project folder `project_dir` and writes `project.json` in it.
///
/// Only the folder's owner reads it, as Chromium keeps a profile. The file names the project's
/// paths, its host and when the folder was made, for anyone reading the folder. A folder that
/// has the file keeps it.
///
/// # Errors
///
/// When the folder cannot be made or the file written.
pub fn write_project_file_in(
    project_dir: &Path,
    paths: &[PathBuf],
    host: Option<&str>,
) -> anyhow::Result<()> {
    let file = project_dir.join(PROJECT_FILE);
    if file.is_file() {
        return Ok(());
    }
    make_private_dir(project_dir)?;
    let made = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let text = serde_json::to_string_pretty(&serde_json::json!({
        "paths": paths.iter().map(|path| path.to_string_lossy()).collect::<Vec<_>>(),
        "host": host,
        "made": made,
    }))?;
    let written = project_dir.join(format!("{PROJECT_FILE}.new"));
    std::fs::write(&written, text).with_context(|| format!("writing {}", written.display()))?;
    std::fs::rename(&written, &file).with_context(|| format!("writing {}", file.display()))
}

/// What became of the profile of earlier builds (#507).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyMove {
    /// It is the project's profile now, at this path.
    Moved(PathBuf),
    /// There is none.
    NoLegacy,
    /// It stays where it is, since a project has a profile already.
    KeptBecauseAProjectHasOne,
}

/// Moves the profile every build before #507 used to the project folder `project_dir`.
///
/// That is the first project whose Chromium starts, so the profile's cookies and logins become
/// that project's, unless a project has a profile already. The profile is renamed, never copied
/// or deleted, after the caller has stopped the Chromium that used it.
///
/// # Errors
///
/// When the folders cannot be read or made, or the profile cannot be renamed.
pub fn move_legacy_profile_in(data: &Path, project_dir: &Path) -> anyhow::Result<LegacyMove> {
    let legacy = legacy_profile_in(data);
    if !legacy.is_dir() {
        return Ok(LegacyMove::NoLegacy);
    }
    let projects = data.join("browser").join("projects");
    let entries = match std::fs::read_dir(&projects) {
        Ok(entries) => Some(entries),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(error).context(format!("reading {}", projects.display())),
    };
    for entry in entries.into_iter().flatten() {
        let entry = entry.with_context(|| format!("reading {}", projects.display()))?;
        if profile_in(&entry.path()).is_dir() {
            return Ok(LegacyMove::KeptBecauseAProjectHasOne);
        }
    }
    make_private_dir(project_dir)?;
    let profile = profile_in(project_dir);
    std::fs::rename(&legacy, &profile)
        .with_context(|| format!("moving {} to {}", legacy.display(), profile.display()))?;
    Ok(LegacyMove::Moved(profile))
}

/// Makes `dir` and the folders above it that are missing, readable by their owner alone.
fn make_private_dir(dir: &Path) -> anyhow::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder
        .create(dir)
        .with_context(|| format!("making {}", dir.display()))
}

/// Finds the browser binary: `override_path` alone when given, else the first of the known
/// binaries that exists, else the first known name on the PATH.
///
/// # Errors
///
/// When nothing is found, an error that names every place it looked.
pub fn find_binary(override_path: Option<&Path>) -> anyhow::Result<PathBuf> {
    if let Some(path) = override_path {
        anyhow::ensure!(
            path.is_file(),
            "No Chromium at {} (named by {BINARY_OVERRIDE})",
            path.display()
        );
        return Ok(path.to_path_buf());
    }
    if let Some(path) = BINARY_PATHS
        .iter()
        .map(Path::new)
        .find(|path| path.is_file())
    {
        return Ok(path.to_path_buf());
    }
    BINARY_NAMES
        .iter()
        .find_map(|name| which::which(name).ok())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No Chromium found: looked for {} and for {} on the PATH. Install Chromium, or \
                 name its binary with {BINARY_OVERRIDE}.",
                BINARY_PATHS.join(", "),
                BINARY_NAMES.join(" and ")
            )
        })
}

/// The unit's name for the Chromium whose profile is `profile`: `marley-browser-` and the first
/// twelve hex digits of the SHA-256 of the profile's path.
#[must_use]
pub fn unit_name(profile: &Path) -> String {
    let digest = Sha256::digest(profile.as_os_str().as_encoded_bytes());
    format!("marley-browser-{}", hex::encode(&digest[..6]))
}

/// The arguments Chromium runs with.
///
/// It is headless, its profile is `profile`, its debugging endpoint is on a port it picks, it
/// opens no first-run pages, and it keeps cookies without the desktop's keyring: a headless
/// service must never wait on the keyring's unlock prompt. It opens no page of its own either
/// (#494): Marley opens the pages its tabs and agents ask for, and a restored tab its saved URL.
#[must_use]
pub fn chromium_args(profile: &Path) -> Vec<OsString> {
    let mut profile_arg = OsString::from("--user-data-dir=");
    profile_arg.push(profile);
    vec![
        OsString::from("--headless"),
        OsString::from("--remote-debugging-port=0"),
        profile_arg,
        OsString::from("--no-first-run"),
        OsString::from("--no-default-browser-check"),
        OsString::from("--password-store=basic"),
        OsString::from("--no-startup-window"),
    ]
}

/// The `systemd-run` arguments that start `binary` with `profile` as the transient user unit
/// `unit`: removed once it stops, its helpers stopped with it.
#[must_use]
pub fn systemd_run_args(unit: &str, binary: &Path, profile: &Path) -> Vec<OsString> {
    let mut args: Vec<OsString> = [
        "--user",
        "--quiet",
        "--collect",
        &format!("--unit={unit}"),
        "--description=Marley's browser",
        "--service-type=exec",
        "--property=KillMode=mixed",
        "--property=TimeoutStopSec=10",
        "--",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    args.push(binary.as_os_str().to_os_string());
    args.extend(chromium_args(profile));
    args
}

/// How starting the unit went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Started {
    /// systemd started it.
    Started,
    /// A unit of that name was there already, starting or running.
    AlreadyThere,
}

/// Starts Chromium as the transient user unit `unit`.
///
/// # Errors
///
/// When `systemd-run` cannot run, or refuses for another reason than the unit being there.
pub async fn start(unit: &str, binary: &Path, profile: &Path) -> anyhow::Result<Started> {
    let output = util::command::new_command("systemd-run")
        .args(systemd_run_args(unit, binary, profile))
        .output()
        .await
        .context("running systemd-run")?;
    if output.status.success() {
        return Ok(Started::Started);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("already") {
        return Ok(Started::AlreadyThere);
    }
    anyhow::bail!("systemd-run could not start Chromium: {}", stderr.trim())
}

/// Stops the unit `unit` (#507); `systemctl` answers once it has stopped. A unit that is not
/// there is stopped already.
///
/// # Errors
///
/// When `systemctl` cannot run, or refuses for another reason.
pub async fn stop(unit: &str) -> anyhow::Result<()> {
    let output = util::command::new_command("systemctl")
        .args([OsStr::new("--user"), OsStr::new("stop"), OsStr::new(unit)])
        .output()
        .await
        .context("running systemctl")?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("not loaded") {
        return Ok(());
    }
    anyhow::bail!("systemctl could not stop {unit}: {}", stderr.trim())
}

/// What `systemctl --user is-active` says of a unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitState {
    /// Running.
    Active,
    /// Starting.
    Activating,
    /// Not running, or not there at all.
    Inactive,
    /// It ran and failed.
    Failed,
    /// Anything else systemd reports.
    Other(String),
}

impl UnitState {
    /// Whether the unit is running or on its way.
    #[must_use]
    pub const fn is_up(&self) -> bool {
        matches!(self, Self::Active | Self::Activating)
    }
}

/// What systemd says of `unit`.
///
/// # Errors
///
/// When `systemctl` cannot run.
pub async fn unit_state(unit: &str) -> anyhow::Result<UnitState> {
    let output = util::command::new_command("systemctl")
        .args([
            OsStr::new("--user"),
            OsStr::new("is-active"),
            OsStr::new(unit),
        ])
        .output()
        .await
        .context("running systemctl")?;
    // `is-active` answers on stdout and says "inactive" or "failed" with a non-zero status.
    let state = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(match state.as_str() {
        "active" | "reloading" => UnitState::Active,
        "activating" => UnitState::Activating,
        "inactive" | "deactivating" | "" => UnitState::Inactive,
        "failed" => UnitState::Failed,
        _ => UnitState::Other(state),
    })
}

/// Chromium's debugging endpoint: the port on 127.0.0.1 and the browser's WebSocket path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    /// The port Chromium picked.
    pub port: u16,
    /// The browser target's path, `/devtools/browser/<id>`.
    pub path: String,
}

/// The endpoint `DevToolsActivePort` in `profile` names: the port on its first line, the path
/// on its second. `None` while the file is not there.
///
/// # Errors
///
/// When the file cannot be read, or does not name a port and a path.
pub fn endpoint_in(profile: &Path) -> anyhow::Result<Option<Endpoint>> {
    let path = profile.join(ENDPOINT_FILE);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context(format!("reading {}", path.display())),
    };
    let mut lines = text.lines();
    let port = lines
        .next()
        .and_then(|line| line.trim().parse::<u16>().ok())
        .with_context(|| format!("{} names no port", path.display()))?;
    let websocket_path = lines
        .next()
        .map(str::trim)
        .filter(|line| line.starts_with('/'))
        .with_context(|| format!("{} names no WebSocket path", path.display()))?;
    Ok(Some(Endpoint {
        port,
        path: websocket_path.to_string(),
    }))
}

/// Removes the `DevToolsActivePort` a Chromium that is gone left in `profile`.
///
/// # Errors
///
/// When the file is there and cannot be removed.
pub fn remove_endpoint_in(profile: &Path) -> anyhow::Result<()> {
    let path = profile.join(ENDPOINT_FILE);
    match std::fs::remove_file(&path) {
        Err(error) if error.kind() != ErrorKind::NotFound => {
            Err(error).context(format!("removing {}", path.display()))
        }
        _ => Ok(()),
    }
}
