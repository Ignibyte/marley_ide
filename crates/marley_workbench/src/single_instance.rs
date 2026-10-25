//! One Marley per data directory (#513).
//!
//! Every Marley build is on Zed's `dev` channel, and they share one data directory, where a
//! second app hangs. Zed's single-instance check binds [`socket_path`],
//! `<data dir>/marley-<channel>.sock` since #760, and reads
//! each datagram on it as a URL: `file://` opens a path, and `zed://open` brings a window
//! forward. When that check finds a Marley running, `zed`'s `main` calls [`hand_off`] before
//! the launch exits, so the paths go to the running Marley, as Zed's CLI would hand them, instead
//! of to a second app, and prints what it answers. The check runs only where
//! [`socket_fits`]: a socket path longer than a Unix socket's address fails the bind as a running
//! Marley would. A `marley <verb>` hands itself over earlier, before Zed's `main` starts its log,
//! through `hand_verb` (#775).

use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use anyhow::{Context as _, Result, bail};
use release_channel::RELEASE_CHANNEL_NAME;

/// The buffer Zed's listener reads a datagram into: a longer URL would arrive cut.
const MAX_URL_BYTES: usize = 1024;

/// The schemes the running app takes as they are, as its own command line does; `marley://` is
/// Marley's twin of `zed://` (#762).
const URL_SCHEMES: [&str; 5] = ["file://", "zed://", "marley://", "zed-cli://", "ssh://"];

/// A Unix socket's address holds a path shorter than this (`sun_path`, with its NUL).
const SOCKET_PATH_BYTES: usize = if cfg!(target_os = "freebsd") {
    104
} else {
    108
};

/// The datagram that carries a second launch's activation token, ahead of its paths (#545).
const ACTIVATION_TOKEN_URL: &str = "zed://marley-activation-token/";

/// The launch's `XDG_ACTIVATION_TOKEN`, read before the platform takes it out of the environment.
static ACTIVATION_TOKEN: OnceLock<Option<String>> = OnceLock::new();

/// The folders a second launch handed over and the running Marley has not yet opened (#913, D5):
/// the socket's listener is a thread of its own.
static HANDED: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

/// Reads the launch's `XDG_ACTIVATION_TOKEN`, the token its launcher made for its click (#545).
///
/// gpui's Wayland client takes the variable out of the environment when it starts, so `zed`'s
/// `main` calls this first, and a hand-off can carry the token to the running Marley.
pub fn keep_activation_token() {
    let token = std::env::var("XDG_ACTIVATION_TOKEN")
        .ok()
        .filter(|token| !token.is_empty());
    ACTIVATION_TOKEN.get_or_init(|| token);
}

/// The activation token `url` carries, when it is a hand-off's token datagram.
#[must_use]
pub fn activation_token_in(url: &str) -> Option<String> {
    url.strip_prefix(ACTIVATION_TOKEN_URL)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
}

/// `url` with Marley's scheme read as Zed's: `marley://<rest>` is `zed://<rest>` (#762), so every
/// link Zed's open listener reads has a `marley://` twin.
#[must_use]
pub fn zed_url(url: String) -> String {
    if let Some(rest) = url.strip_prefix("marley://") {
        return format!("zed://{rest}");
    }
    url
}

/// The socket Zed's check binds in the data directory, and Marley's hand-off sends to.
#[must_use]
pub fn socket_path() -> PathBuf {
    socket_in(paths::data_dir())
}

/// The socket Zed's check binds in `data_dir`: Marley's name, with Zed's channel (#760).
fn socket_in(data_dir: &Path) -> PathBuf {
    data_dir.join(format!("marley-{}.sock", *RELEASE_CHANNEL_NAME))
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
    Ok(handed_line(data_dir, paths_or_urls))
}

/// Hands `url`, a `marley <verb>`'s, to the Marley running on `data_dir`, before Zed's `main`
/// starts its log, so the launch prints the hand-off's line and nothing else (#775). Answers the
/// line once the URL went, or `None` when no Marley listens there (no socket, a refused one, or a
/// path too long for one), for the launch to start Marley instead.
///
/// # Errors
///
/// The URL is longer than the listener reads, or the socket is there and a send fails.
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub(crate) fn hand_verb(url: &str, data_dir: &Path) -> Result<Option<String>> {
    let socket = socket_in(data_dir);
    if socket.as_os_str().len() >= SOCKET_PATH_BYTES {
        return Ok(None);
    }
    let sender = UnixDatagram::unbound().context("making a socket")?;
    match sender.connect(&socket) {
        Ok(()) => {}
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
            ) =>
        {
            return Ok(None);
        }
        Err(error) => {
            return Err(anyhow::Error::new(error).context(format!("reaching {}", socket.display())));
        }
    }
    let urls = [url.to_string()];
    deliver(&sender, &with_token(urls.to_vec()))?;
    Ok(Some(handed_line(data_dir, &urls)))
}

/// What a hand-off of `paths_or_urls` to the Marley on `data_dir` says it did.
fn handed_line(data_dir: &Path, paths_or_urls: &[String]) -> String {
    let verbs: Vec<&str> = paths_or_urls
        .iter()
        .filter_map(|url| crate::commands::verb_in(url))
        .collect();
    if paths_or_urls.is_empty() {
        format!(
            "Marley is already running on {}; it was asked to come forward.",
            data_dir.display()
        )
    } else if let [verb] = verbs.as_slice() {
        format!(
            "Marley is already running on {}; it was asked to run `marley {verb}`.",
            data_dir.display()
        )
    } else {
        format!(
            "Marley is already running on {}; it was handed {} of this launch's paths.",
            data_dir.display(),
            paths_or_urls.len()
        )
    }
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
    let urls = with_token(urls);
    let socket = socket_path();
    let sender = UnixDatagram::unbound().context("making a socket")?;
    sender
        .connect(&socket)
        .with_context(|| format!("reaching {}", socket.display()))?;
    deliver(&sender, &urls)
}

/// `urls` behind the launch's activation token, so the running Marley holds the token when the
/// paths bring a window forward (#545).
fn with_token(urls: Vec<String>) -> Vec<String> {
    let token = ACTIVATION_TOKEN
        .get()
        .and_then(Option::as_ref)
        .map(|token| format!("{ACTIVATION_TOKEN_URL}{token}"));
    token.into_iter().chain(urls).collect()
}

/// Sends each of `urls` as a datagram on `sender`, all checked against the listener's buffer
/// first.
fn deliver(sender: &UnixDatagram, urls: &[String]) -> Result<()> {
    if let Some(url) = urls.iter().find(|url| url.len() > MAX_URL_BYTES) {
        bail!("{url} is longer than the {MAX_URL_BYTES} bytes the running Marley reads");
    }
    for url in urls {
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

/// Notes a path a second launch handed over through the socket (#913, D5), so the window it opens
/// says where it came from. A launch's own arguments never pass here.
pub fn note_handed(url: &str) {
    let Some(path) = url.strip_prefix("file://") else {
        return;
    };
    if let Ok(mut handed) = HANDED.lock() {
        handed.push(PathBuf::from(path));
    }
}

/// Shows "Opened `<name>` from another launch" in `window` for each of `paths` a second launch
/// handed over (#913), and forgets them.
pub fn announce_handed(
    window: gpui::WindowHandle<workspace::MultiWorkspace>,
    paths: &[PathBuf],
    cx: &mut gpui::AsyncApp,
) {
    let mut announced: Vec<String> = Vec::new();
    if let Ok(mut handed) = HANDED.lock() {
        handed.retain(|path| {
            let opened = paths.iter().any(|opened| opened == path);
            if opened {
                announced.push(path.file_name().map_or_else(
                    || path.display().to_string(),
                    |name| name.to_string_lossy().into_owned(),
                ));
            }
            !opened
        });
    }
    let Some(name) = announced.first().cloned() else {
        return;
    };
    let message = format!(
        "Opened {name} from another launch. Only one Marley runs on a data folder. The second \
         launch handed you its folder and quit."
    );
    window
        .update(cx, |multi_workspace, _, cx| {
            multi_workspace.workspace().update(cx, |workspace, cx| {
                workspace.show_toast(
                    workspace::Toast::new(
                        workspace::notifications::NotificationId::unique::<HandedNotice>(),
                        message,
                    )
                    .on_click("Dismiss", |_, _| {}),
                    cx,
                );
            });
        })
        .ok();
}

/// The second-launch notice's toast (#913).
struct HandedNotice;
