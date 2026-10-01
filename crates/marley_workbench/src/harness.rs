//! The harness's sessions (#534): Marley follows rustal-harness's fleet over `rh mcp`, read side
//! only, and shows each session in the rail's Harness section and in a read-only tab.
//!
//! `marley.harness` names the MCP server's command, as a context server's is named; without it
//! Marley starts nothing. With it, Marley starts the command through Zed's MCP client, takes the
//! fleet from `fleet_snapshot` (the harness checks that its snapshot is Marley's own fold of its
//! events), and asks `fleet_events` from the snapshot's cursor every second, folding each event
//! with `marley_fleet::apply`; an expired cursor re-reads the snapshot. Zed's client sees no
//! server exit, so a call that fails, or takes more than five seconds, is the sign: the section
//! says the harness is not running and why, keeps its rows, marked stale, and starts the command
//! again after 1 s, doubling to at most 60 s. Marley calls no write verb.

use std::pin::pin;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use context_server::types::requests::CallTool;
use context_server::types::{CallToolParams, CallToolResponse};
use context_server::{ContextServer, ContextServerCommand, ContextServerId};
use futures::future::{Either, select};
use gpui::{
    App, AsyncApp, Context, EventEmitter, FocusHandle, Focusable, Global, SharedString,
    Subscription, Task,
};
use marley_fleet::{FleetSnapshot, SessionEvent, State, apply};
use serde::Deserialize;
use serde_json::{Value, json};
use settings::{Settings as _, SettingsStore};
use ui::prelude::*;
use util::ResultExt as _;
use workspace::Workspace;
use workspace::item::Item;

use crate::MarleySettings;

/// How often the fleet is asked for new events.
const POLL: Duration = Duration::from_secs(1);

/// How long one call may take before the harness counts as not answering.
const CALL_TIMEOUT: Duration = Duration::from_secs(5);

/// The longest wait between two starts of the command, in seconds.
const BACKOFF_MAX_S: u64 = 60;

/// How many of a session's last lines its tab shows.
const VIEW_LINES: u32 = 500;

/// How often an open tab reads its session's lines again while the session runs: an actor's
/// output does not change its envelope.
const VIEW_REFRESH: Duration = Duration::from_secs(2);

/// How many characters of a reason the section's header shows.
const REASON_CHARS: usize = 120;

/// The harness's connection, as the section's header says it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Connection {
    Connecting,
    Connected,
    /// Not running, with the reason.
    Down(SharedString),
}

/// What Marley follows of the harness: the setting's command, the connection, and the fleet.
#[derive(Default)]
pub(crate) struct Harness {
    command: Option<ContextServerCommand>,
    connection: Option<Connection>,
    seats: FleetSnapshot,
    /// Bumped each minute while a session works, so its `no update in N m` is drawn again.
    minute: u64,
    /// Whether the rail's Harness section folds its rows under its header.
    folded: bool,
    server: Option<Arc<ContextServer>>,
    run: Option<Task<()>>,
}

impl Global for Harness {}

impl Harness {
    /// The connection, while `marley.harness` names a command.
    pub(crate) fn connection(cx: &App) -> Option<Connection> {
        cx.try_global::<Self>()?.connection.clone()
    }

    /// Whether the rail's Harness section folds its rows.
    pub(crate) fn folded(cx: &App) -> bool {
        cx.try_global::<Self>()
            .is_some_and(|harness| harness.folded)
    }

    /// Folds the rail's Harness section, or unfolds it.
    pub(crate) fn toggle_folded(cx: &mut App) {
        let harness = cx.global_mut::<Self>();
        harness.folded = !harness.folded;
    }

    /// The harness's sessions, in the order the harness first published them.
    pub(crate) fn seats(cx: &App) -> &FleetSnapshot {
        static NONE: std::sync::LazyLock<FleetSnapshot> =
            std::sync::LazyLock::new(FleetSnapshot::default);
        cx.try_global::<Self>()
            .map_or(&NONE, |harness| &harness.seats)
    }
}

/// Follows `marley.harness`; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(Harness::default());
    follow_setting(cx);
    cx.observe_global::<SettingsStore>(follow_setting).detach();
}

/// Starts following the command `marley.harness` names, or stops, when the setting changed.
fn follow_setting(cx: &mut App) {
    let command = MarleySettings::get_global(cx).harness.clone();
    if cx.global::<Harness>().command == command {
        return;
    }
    let run = command.clone().map(|command| {
        cx.spawn(async move |cx| {
            follow(command, cx).await;
        })
    });
    let harness = cx.global_mut::<Harness>();
    harness.connection = command.as_ref().map(|_| Connection::Connecting);
    harness.command = command;
    harness.seats = FleetSnapshot::default();
    harness.server = None;
    harness.run = run;
}

/// Keeps a connection to the harness's MCP server: connects, follows the fleet until a call
/// fails, and connects again after a growing wait.
async fn follow(command: ContextServerCommand, cx: &AsyncApp) {
    let mut failures = 0_u32;
    loop {
        let error = match connected(&command, &mut failures, cx).await {
            Ok(never) => match never {},
            Err(error) => error,
        };
        // The root cause says the most in the few words a header has.
        let reason = error.root_cause().to_string();
        let reason: String = reason
            .lines()
            .next()
            .unwrap_or_default()
            .chars()
            .take(REASON_CHARS)
            .collect();
        log::info!("harness: not running: {error:#}");
        cx.update(|cx| {
            let harness = cx.global_mut::<Harness>();
            harness.server = None;
            harness.connection = Some(Connection::Down(reason.into()));
        });
        let wait_s = 1_u64
            .checked_shl(failures)
            .unwrap_or(BACKOFF_MAX_S)
            .min(BACKOFF_MAX_S);
        failures = failures.saturating_add(1);
        cx.background_executor()
            .timer(Duration::from_secs(wait_s))
            .await;
        cx.update(|cx| {
            cx.global_mut::<Harness>().connection = Some(Connection::Connecting);
        });
    }
}

/// One connection: starts the server, takes the fleet, and folds its events each second. Returns
/// only the failure that ends it.
async fn connected(
    command: &ContextServerCommand,
    failures: &mut u32,
    cx: &AsyncApp,
) -> anyhow::Result<std::convert::Infallible> {
    let server = Arc::new(ContextServer::stdio(
        ContextServerId(Arc::from("marley-harness")),
        command.clone(),
        None,
    ));
    let executor = cx.background_executor().clone();
    match select(pin!(server.start(cx)), pin!(executor.timer(CALL_TIMEOUT))).await {
        Either::Left((started, _)) => started.context("the harness's MCP server did not start")?,
        Either::Right(_) => anyhow::bail!("the harness's MCP server did not answer"),
    }
    let mut cursor = seed(&server, cx).await?;
    *failures = 0;
    cx.update(|cx| {
        let harness = cx.global_mut::<Harness>();
        harness.server = Some(Arc::clone(&server));
        harness.connection = Some(Connection::Connected);
    });
    let mut minute = now_minute();
    loop {
        executor.timer(POLL).await;
        loop {
            let page = match call(&server, "fleet_events", json!({ "after": cursor }), cx).await {
                Ok(page) => serde_json::from_value::<EventPage>(page)
                    .context("fleet_events's answer did not parse")?,
                Err(Failure::Resync) => {
                    cursor = seed(&server, cx).await?;
                    break;
                }
                Err(Failure::Failed(error)) => return Err(error),
            };
            cursor = page.next_cursor;
            if page.events.is_empty() {
                break;
            }
            cx.update(|cx| {
                let harness = cx.global_mut::<Harness>();
                for entry in &page.events {
                    apply(&mut harness.seats, &entry.event);
                }
            });
        }
        let now = now_minute();
        if now != minute {
            minute = now;
            cx.update(|cx| {
                let working = Harness::seats(cx)
                    .seats()
                    .iter()
                    .any(|seat| seat.state == State::Working);
                if working {
                    cx.global_mut::<Harness>().minute = now;
                }
            });
        }
    }
}

/// Takes the fleet from `fleet_snapshot` and gives the cursor it holds at.
async fn seed(server: &ContextServer, cx: &AsyncApp) -> anyhow::Result<u64> {
    let value = match call(server, "fleet_snapshot", json!({}), cx).await {
        Ok(value) => value,
        Err(Failure::Resync) => anyhow::bail!("fleet_snapshot asked for a resync"),
        Err(Failure::Failed(error)) => return Err(error),
    };
    let cursor = value
        .get("cursor")
        .and_then(Value::as_u64)
        .context("fleet_snapshot gave no cursor")?;
    let seats: FleetSnapshot =
        serde_json::from_value(value).context("fleet_snapshot's seats did not parse")?;
    cx.update(|cx| cx.global_mut::<Harness>().seats = seats);
    Ok(cursor)
}

/// A page of `fleet_events`.
#[derive(Deserialize)]
struct EventPage {
    events: Vec<EventEntry>,
    next_cursor: u64,
}

#[derive(Deserialize)]
struct EventEntry {
    event: SessionEvent,
}

/// Why a call came back without an answer.
enum Failure {
    /// The cursor fell behind what the harness keeps.
    Resync,
    Failed(anyhow::Error),
}

/// Calls one of the harness's read tools and gives its structured content, within
/// [`CALL_TIMEOUT`].
async fn call(
    server: &ContextServer,
    tool: &str,
    arguments: Value,
    cx: &AsyncApp,
) -> Result<Value, Failure> {
    let executor = cx.background_executor().clone();
    let asking = async {
        let protocol = server
            .client()
            .context("the harness's MCP server is not running")?;
        protocol
            .request::<CallTool>(CallToolParams {
                name: tool.to_string(),
                arguments: Some(arguments),
                meta: None,
            })
            .await
            .with_context(|| format!("{tool} failed"))
    };
    let response = match select(pin!(asking), pin!(executor.timer(CALL_TIMEOUT))).await {
        Either::Left((response, _)) => response.map_err(Failure::Failed)?,
        Either::Right(_) => {
            return Err(Failure::Failed(anyhow::anyhow!(
                "the harness did not answer {tool} within {} s",
                CALL_TIMEOUT.as_secs()
            )));
        }
    };
    answer(tool, response)
}

/// A tool's structured content, or why there is none.
fn answer(tool: &str, response: CallToolResponse) -> Result<Value, Failure> {
    let text = response.text_contents();
    if response.is_error == Some(true) {
        if text.starts_with("resync_required") {
            return Err(Failure::Resync);
        }
        return Err(Failure::Failed(anyhow::anyhow!(
            "{tool} failed: {}",
            text.lines().next().unwrap_or_default()
        )));
    }
    response.structured_content.map_or_else(
        || {
            serde_json::from_str(&text)
                .with_context(|| format!("{tool} gave no JSON"))
                .map_err(Failure::Failed)
        },
        Ok,
    )
}

/// A session's question as the rail shows it: without the `[wait <key>, generation <id>]` the
/// harness adds to an actor's prompt to route its answer, which leaves no room for the options.
pub(crate) fn shown_prompt(prompt: &str) -> &str {
    match prompt.rfind(" [wait ") {
        Some(start) if prompt.ends_with(']') => &prompt[..start],
        _ => prompt,
    }
}

/// The minutes since the epoch.
fn now_minute() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs() / 60)
}

/// The milliseconds since the epoch.
pub(crate) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| {
            u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
        })
}

/// Shows the tab of the session `id` in `workspace`, or brings forward the one open there.
pub(crate) fn open(
    workspace: &mut Workspace,
    id: &str,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let open = workspace
        .items_of_type::<HarnessView>(cx)
        .find(|view| view.read(cx).id == id);
    if let Some(open) = open {
        workspace.activate_item(&open, true, true, window, cx);
        return;
    }
    let id = id.to_string();
    let view = cx.new(|cx| HarnessView::new(id, window, cx));
    workspace.add_item_to_center(Box::new(view), window, cx);
}

/// A harness session's output, read-only: its last lines as the harness renders them.
pub(crate) struct HarnessView {
    id: String,
    title: SharedString,
    lines: Vec<SharedString>,
    error: Option<SharedString>,
    /// The envelope's time and state when the lines were last asked for.
    seen: Option<(u64, State)>,
    focus_handle: FocusHandle,
    reading: Option<Task<()>>,
    _harness: Subscription,
    _refresh: Task<()>,
}

impl std::fmt::Debug for HarnessView {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HarnessView")
            .field("id", &self.id)
            .field("lines", &self.lines.len())
            .finish_non_exhaustive()
    }
}

impl HarnessView {
    fn new(id: String, window: &Window, cx: &mut Context<Self>) -> Self {
        let title = Harness::seats(cx).get(&id).map_or_else(
            || SharedString::from(id.clone()),
            |seat| seat.title.clone().into(),
        );
        let harness = cx.observe_global_in::<Harness>(window, |view, _, cx| view.seat_changed(cx));
        let refresh = cx.spawn(async move |view, cx| {
            loop {
                cx.background_executor().timer(VIEW_REFRESH).await;
                let Ok(()) = view.update(cx, |view, cx| {
                    let running = Harness::seats(cx)
                        .get(&view.id)
                        .is_some_and(|seat| seat.state != State::Done);
                    if running {
                        view.ask_lines(cx);
                    }
                }) else {
                    break;
                };
            }
        });
        let mut view = Self {
            id,
            title,
            lines: Vec::new(),
            error: None,
            seen: None,
            focus_handle: cx.focus_handle(),
            reading: None,
            _harness: harness,
            _refresh: refresh,
        };
        view.seat_changed(cx);
        view
    }

    /// Reads the lines again when the session's envelope changed.
    fn seat_changed(&mut self, cx: &mut Context<Self>) {
        let Some(seat) = Harness::seats(cx).get(&self.id) else {
            return;
        };
        let now = (seat.last_event_ms, seat.state);
        let title = SharedString::from(seat.title.clone());
        if self.title != title {
            self.title = title;
            cx.notify();
        }
        if self.seen != Some(now) {
            self.seen = Some(now);
            self.ask_lines(cx);
        }
    }

    /// Asks the harness for the session's last lines, unless a read is under way.
    fn ask_lines(&mut self, cx: &Context<Self>) {
        if self.reading.is_some() {
            return;
        }
        let Some(server) = cx
            .try_global::<Harness>()
            .and_then(|harness| harness.server.clone())
        else {
            return;
        };
        let arguments = json!({
            "id": self.id,
            "range": { "mode": "tail", "lines": VIEW_LINES },
        });
        self.reading = Some(cx.spawn(async move |view, cx| {
            let read = call(&server, "session_read", arguments, cx).await;
            view.update(cx, |view, cx| {
                view.reading = None;
                match read.map_err(|failure| match failure {
                    Failure::Resync => anyhow::anyhow!("session_read asked for a resync"),
                    Failure::Failed(error) => error,
                }) {
                    Ok(value) => match read_lines(&value) {
                        Ok(lines) => {
                            view.lines = lines;
                            view.error = None;
                        }
                        Err(error) => view.error = Some(error.to_string().into()),
                    },
                    Err(error) => view.error = Some(format!("{error:#}").into()),
                }
                cx.notify();
            })
            .log_err();
        }));
    }
}

/// The lines of a `session_read` receipt, or what it refused.
fn read_lines(value: &Value) -> anyhow::Result<Vec<SharedString>> {
    let result = value
        .get("result")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if result != "accepted" {
        let reason = value
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or(result);
        anyhow::bail!("session_read was refused: {reason}");
    }
    let lines = value
        .pointer("/value/lines")
        .and_then(Value::as_array)
        .context("session_read gave no lines")?;
    Ok(lines
        .iter()
        .map(|line| SharedString::from(line.as_str().unwrap_or_default().to_string()))
        .collect())
}

impl Render for HarnessView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        v_flex()
            .id("marley-harness-view")
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .p_2()
            .bg(colors.editor_background)
            .when_some(self.error.clone(), |this, error| {
                this.child(Label::new(error).color(Color::Error))
            })
            .children(
                self.lines
                    .iter()
                    .map(|line| Label::new(line.clone()).buffer_font(cx).single_line()),
            )
    }
}

impl Focusable for HarnessView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<()> for HarnessView {}

impl Item for HarnessView {
    type Event = ();

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        self.title.clone()
    }
}
