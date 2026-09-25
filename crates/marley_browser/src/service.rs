//! Marley's Chromium as a transient user unit (plan D16).
//!
//! The first Browser tab starts it with `systemd-run --user`, one unit per Marley data
//! directory, so an e2e run's Marley starts its own. The unit runs the browser binary itself,
//! not a distribution's launcher, which would add the user's `chromium-flags.conf`. Chromium
//! runs headless and puts its debugging endpoint on a port it picks, which it writes to
//! `DevToolsActivePort` in the profile, where every client finds it. The unit outlives Marley
//! and ends at logout.

use std::ffi::{OsStr, OsString};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

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
