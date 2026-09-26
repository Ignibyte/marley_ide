//! One Marley per data directory (#513).
//!
//! Every Marley build is on Zed's `dev` channel, and they share one data directory, where a
//! second app hangs. Zed's single-instance check binds `<data dir>/zed-<channel>.sock` and reads
//! each datagram on it as a URL: `file://` opens a path, and `zed://open` brings a window
//! forward. When that check finds a Marley running, `zed`'s `main` calls [`hand_off`] before
//! the launch exits, so the paths go to the running Marley, as Zed's CLI would hand them, instead
//! of to a second app, and prints what it answers. The check runs only where
//! [`socket_fits`]: a socket path longer than a Unix socket's address fails the bind as a running
//! Marley would.

use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use release_channel::RELEASE_CHANNEL_NAME;

/// The buffer Zed's listener reads a datagram into: a longer URL would arrive cut.
const MAX_URL_BYTES: usize = 1024;

/// The schemes the running app takes as they are, as its own command line does.
const URL_SCHEMES: [&str; 4] = ["file://", "zed://", "zed-cli://", "ssh://"];

/// A Unix socket's address holds a path shorter than this (`sun_path`, with its NUL).
const SOCKET_PATH_BYTES: usize = if cfg!(target_os = "freebsd") {
    104
} else {
    108
};

/// The socket Zed's check binds in the data directory.
fn socket_path() -> PathBuf {
    paths::data_dir().join(format!("zed-{}.sock", *RELEASE_CHANNEL_NAME))
}

/// Whether Zed's check can bind the data directory's socket.
///
/// On a longer path the bind fails as it does when a Marley runs, and Marley would refuse to
/// start, so such a data directory (a deep `--user-data-dir`) starts without the check, as the
/// dev channel always did, and says so in the log.
#[must_use]
pub fn socket_fits() -> bool {
    let socket = socket_path();
    let fits = socket.as_os_str().len() < SOCKET_PATH_BYTES;
    if !fits {
        log::warn!(
            "{} is too long for a Unix socket, so nothing stops a second Marley on this data directory",
            socket.display()
        );
    }
    fits
}

/// Sends each of `paths_or_urls` to the Marley that owns the data directory's socket as the URL
/// it opens, or `zed://open` when there is none, and answers the line that says what went.
///
/// # Errors
///
/// The working directory cannot be read, a URL is longer than the listener reads, or the socket
/// refuses the connection or a send (its Marley has gone).
pub fn hand_off(paths_or_urls: &[String]) -> Result<String> {
    let data_dir = paths::data_dir();
    send(paths_or_urls).map_err(|error| {
        error.context(format!(
            "Marley is already running on {}, and this launch's paths did not reach it",
            data_dir.display()
        ))
    })?;
    Ok(if paths_or_urls.is_empty() {
        format!(
            "Marley is already running on {}; it was asked to come forward.",
            data_dir.display()
        )
    } else {
        format!(
            "Marley is already running on {}; it was handed {} of this launch's paths.",
            data_dir.display(),
            paths_or_urls.len()
        )
    })
}

/// The datagrams themselves: one URL each, all checked against the listener's buffer first.
fn send(paths_or_urls: &[String]) -> Result<()> {
    let urls = if paths_or_urls.is_empty() {
        vec!["zed://open".to_string()]
    } else {
        let working_dir = std::env::current_dir().context("reading the working directory")?;
        paths_or_urls
            .iter()
            .map(|arg| url_for(arg, &working_dir))
            .collect()
    };
    if let Some(url) = urls.iter().find(|url| url.len() > MAX_URL_BYTES) {
        bail!("{url} is longer than the {MAX_URL_BYTES} bytes the running Marley reads");
    }
    let socket = socket_path();
    let sender = UnixDatagram::unbound().context("making a socket")?;
    sender
        .connect(&socket)
        .with_context(|| format!("reaching {}", socket.display()))?;
    for url in &urls {
        sender
            .send(url.as_bytes())
            .with_context(|| format!("sending {url}"))?;
    }
    Ok(())
}

/// `arg` as the running app opens it. A path that exists goes canonicalized; a URL of a scheme
/// the app takes goes as it is; anything else, such as `src/main.rs:10:5`, is made absolute
/// against this launch's working directory, since the running app's is another.
fn url_for(arg: &str, working_dir: &Path) -> String {
    if let Ok(path) = std::fs::canonicalize(arg) {
        return format!("file://{}", path.display());
    }
    if URL_SCHEMES.iter().any(|scheme| arg.starts_with(scheme)) {
        return arg.to_string();
    }
    format!("file://{}", working_dir.join(arg).display())
}
