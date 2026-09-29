//! Claude Code's events pushed to the phone through ntfy (#535).
//!
//! When Claude Code in a terminal the user is not looking at needs input, finishes or fails,
//! `on_change` posts one line to the ntfy server `marley.push` names, such as
//! `marley_ide: Claude needs input`. The line carries nothing the agent wrote. The server must be
//! on this machine (ntfy on the dev box, which `tailscale serve` publishes to the phone), a token
//! comes from a file only its owner can read, a burst of events from one project makes one push,
//! and a server that does not answer shows one toast until it answers again.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use gpui::{App, AppContext as _, Context, Global, WeakEntity, Window};
use http_client::{AsyncBody, Host, HttpClient, Method, Request, Url};
use marley_agent::claude_events::CWD_LABEL;
use marley_agent::{AgentKind, TurnEvent, event_line};
use marley_fleet::{Session, State};
use settings::Settings as _;
use terminal_view::TerminalView;
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use crate::{MarleySettings, PushSettings, notifications};

/// A burst of events from one project within this long makes one push, as Orca's cooldown does.
const COOLDOWN: Duration = Duration::from_secs(5);

/// When each project last pushed, and whether the server has failed since it last answered.
#[derive(Default)]
struct Pushes {
    last: HashMap<String, Instant>,
    failing: bool,
}

impl Global for Pushes {}

/// What became of a push.
enum Outcome {
    Posted,
    /// The settings forbid it; logged, since a toast for each event would only repeat it.
    Refused(String),
    /// The server did not take it.
    Failed(anyhow::Error),
}

/// Pushes the event, if any, that `view`'s seat made by moving from `before` to its state in
/// `seat`, unless no server is set or the user is looking at the terminal.
pub(crate) fn on_change(
    view: &TerminalView,
    before: State,
    seat: &Session,
    window: &Window,
    cx: &mut Context<TerminalView>,
) {
    let Some(event) = TurnEvent::of_change(before, seat.state) else {
        return;
    };
    let Some(settings) = MarleySettings::get_global(cx).push.clone() else {
        return;
    };
    if notifications::looking_at(view, window, cx) {
        return;
    }
    let target = match target_url(&settings) {
        Ok(target) => target,
        Err(error) => {
            log::warn!("push: {error}");
            return;
        }
    };
    let project = project_name(seat);
    let now = cx.background_executor().now();
    match cx.default_global::<Pushes>().last.entry(project.clone()) {
        Entry::Occupied(mut last) => {
            if now.saturating_duration_since(*last.get()) < COOLDOWN {
                return;
            }
            *last.get_mut() = now;
        }
        Entry::Vacant(slot) => {
            let _stored = slot.insert(now);
        }
    }
    let line = event_line(&project, AgentKind::Claude, event);
    let post = cx.background_spawn(post(
        cx.http_client(),
        target,
        settings.token_file.clone(),
        line,
        event,
    ));
    let workspace = view.marley_workspace().clone();
    cx.spawn(async move |_, cx| {
        let outcome = post.await;
        cx.update(|cx| report(outcome, &settings.url, &workspace, cx));
    })
    .detach();
}

/// The topic's URL on the server, when the server is on this machine and the topic is one ntfy
/// takes. A push to anywhere else would carry the line across a network before ntfy's.
fn target_url(settings: &PushSettings) -> anyhow::Result<Url> {
    let server = Url::parse(&settings.url)
        .with_context(|| format!("`{}` is not a URL, so nothing is pushed", settings.url))?;
    let on_this_machine = match server.host() {
        Some(Host::Domain(domain)) => domain == "localhost",
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        None => false,
    };
    if !on_this_machine || !matches!(server.scheme(), "http" | "https") {
        bail!(
            "{} is not a server on this machine, so nothing is pushed to it",
            settings.url
        );
    }
    let topic = &settings.topic;
    if topic.len() > 64
        || !topic
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        bail!("`{topic}` is not an ntfy topic (letters, digits, `-` and `_`, at most 64)");
    }
    let target = format!("{}/{topic}", settings.url.trim_end_matches('/'));
    Url::parse(&target).with_context(|| format!("`{target}` is not a URL"))
}

/// The project's name for the line: the last folder of where Claude Code started, as the
/// plugin's desktop notification names it.
pub(crate) fn project_name(seat: &Session) -> String {
    seat.labels
        .get(CWD_LABEL)
        .and_then(|cwd| Path::new(cwd).file_name())
        .map_or_else(
            || "A terminal".to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
}

/// Posts `line` for `event` to `target`, off the main thread, with the token from `token_file`.
async fn post(
    client: Arc<dyn HttpClient>,
    target: Url,
    token_file: Option<String>,
    line: String,
    event: TurnEvent,
) -> Outcome {
    let token = match token_file.as_deref().map(read_token).transpose() {
        Ok(token) => token,
        Err(error) => return Outcome::Refused(format!("{error:#}")),
    };
    let (priority, tag) = match event {
        TurnEvent::NeedsInput => ("4", "question"),
        TurnEvent::Finished => ("3", "white_check_mark"),
        TurnEvent::Failed => ("4", "x"),
    };
    let mut request = Request::builder()
        .method(Method::POST)
        .uri(target.as_str())
        .header("Title", "Marley")
        .header("Priority", priority)
        .header("Tags", tag)
        .header("Content-Type", "text/plain; charset=utf-8");
    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {token}"));
    }
    let sent = async {
        let response = client.send(request.body(AsyncBody::from(line))?).await?;
        if !response.status().is_success() {
            bail!("it answered {}", response.status());
        }
        anyhow::Ok(())
    };
    match sent.await {
        Ok(()) => Outcome::Posted,
        Err(error) => Outcome::Failed(error),
    }
}

/// The access token in `file`, which only its owner may read; `~/` is the home directory.
fn read_token(file: &str) -> anyhow::Result<String> {
    let path = file.strip_prefix("~/").map_or_else(
        || PathBuf::from(file),
        |rest| util::paths::home_dir().join(rest),
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(&path)
            .with_context(|| format!("reading {}", path.display()))?
            .permissions()
            .mode();
        if mode & 0o077 != 0 {
            bail!(
                "{} can be read by others (mode {:o}), so its token is not sent and nothing is \
                 pushed",
                path.display(),
                mode & 0o777
            );
        }
    }
    let token =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let token = token.trim();
    if token.is_empty() {
        bail!("{} holds no token", path.display());
    }
    Ok(token.to_string())
}

/// Logs what became of a push, and shows one toast when the server stops answering, until it
/// answers again. The desktop notification never waits on any of it.
fn report(outcome: Outcome, server: &str, workspace: &WeakEntity<Workspace>, cx: &mut App) {
    match outcome {
        Outcome::Posted => cx.default_global::<Pushes>().failing = false,
        Outcome::Refused(why) => log::warn!("push: {why}"),
        Outcome::Failed(error) => {
            log::warn!("push: {server}: {error:#}");
            let pushes = cx.default_global::<Pushes>();
            if !pushes.failing {
                pushes.failing = true;
                workspace
                    .update(cx, |workspace, cx| {
                        workspace.show_toast(
                            Toast::new(
                                NotificationId::unique::<Pushes>(),
                                format!(
                                    "Marley could not push to your phone: the server at {server} \
                                     did not take it ({error})."
                                ),
                            ),
                            cx,
                        );
                    })
                    .log_err();
            }
        }
    }
}
