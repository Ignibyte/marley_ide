//! Agent activity and the kill switch (#703), the first part of the agent-control layer.
//!
//! Every call an agent makes to a write-tier tool of Marley's MCP server (typing into and running
//! in terminals, the browser's clicks and typing, settings and keymap changes, harness seats) is a
//! row in a day log under Marley's data directory and in the Agent Activity tab: who called, the
//! tool, what it acted on, and how it ended: its result's line or the refusal's reason (#933). While `marley.agent_control.stopped` is on, those
//! tools refuse every agent; the tools that only read keep answering.

use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use ::fs::Fs;
use futures::StreamExt as _;
use futures::channel::mpsc;
use gpui::{
    App, AppContext as _, Context, EventEmitter, FocusHandle, Focusable, Global, Render,
    SharedString, Subscription, WeakEntity, Window, actions,
};
use marley_mcp::redact::Redactor;
use marley_mcp::{AppCall, Refusal, ToolAnswer};
use serde::{Deserialize, Serialize};
use settings::{Settings as _, SettingsStore};
use ui::{ButtonStyle, Headline, HeadlineSize, prelude::*};
use util::ResultExt as _;
use workspace::Workspace;
use workspace::item::{Item, ItemEvent};

use crate::groups::GroupKind;
use crate::home_page::muted;
use crate::item_kind::{self, AgentMark, ItemKind};
use crate::{AgentControl, MarleySettings};

actions!(
    marley,
    [
        /// Stops every agent's use of Marley's tools that act: typing into and running in
        /// terminals, the browser's clicks and typing, settings and keymap changes, harness seats.
        /// The tools that only read keep working.
        #[derive(Eq)]
        StopAgentControl,
        /// Lets agents use Marley's tools that act again.
        #[derive(Eq)]
        ResumeAgentControl,
        /// Opens Agent Activity: every call agents made to Marley's tools that act, newest first.
        #[derive(Eq)]
        OpenAgentActivity,
    ]
);

/// The refusal's code while agents' write tools are stopped.
pub(crate) const STOPPED: &str = "agent_control_stopped";

/// How many rows the app keeps in memory; the day files keep them all.
const ROWS_KEPT: usize = 500;

/// How long a row's summary may be, in characters.
const SUMMARY_CHARS: usize = 160;

/// How long a call's result or reason may be, in characters (#933, D5).
const RESULT_CHARS: usize = 120;

/// The arguments a row's summary is taken from, the first one present.
const SUMMARY_FIELDS: [&str; 7] = [
    "command",
    "text",
    "url",
    "key",
    "keystrokes",
    "name",
    "path",
];

/// One call of a write-tier tool.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ActivityRow {
    /// When it was answered, local time, `YYYY-MM-DD HH:MM:SS`.
    pub(crate) time: String,
    /// The caller in words, as `click_pause::Who` names it.
    pub(crate) who: String,
    /// The caller's tile (#933); none in rows written before it, and for a caller Marley cannot
    /// name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) agent: Option<AgentMark>,
    pub(crate) tool: String,
    /// What it acted on: a command, a URL, a key, cut to a line and redacted.
    pub(crate) summary: String,
    pub(crate) outcome: Outcome,
}

/// How a call ended. The lines default, so rows written before #933 still read (D7).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub(crate) enum Outcome {
    Done {
        /// The answer's line, from [`result_line`].
        #[serde(default, skip_serializing_if = "Option::is_none")]
        result: Option<String>,
    },
    Refused {
        code: String,
        /// The refusal's first sentence.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
}

/// How [`render_rows`] lays a row out: the tab's card, or Home's compact line (#933, D6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RowLayout {
    Card,
    Line,
}

/// The rows of this run and today's earlier ones, newest last; the tab and Home's card observe it.
pub(crate) struct AgentActivity {
    rows: VecDeque<ActivityRow>,
    log: mpsc::UnboundedSender<ActivityRow>,
}

impl Global for AgentActivity {}

/// Sets up the log, reads today's rows back, and registers the actions on every workspace;
/// [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    let (log, mut logged) = mpsc::unbounded::<ActivityRow>();
    let (shown, mut to_show) = mpsc::unbounded::<ActivityRow>();
    cx.set_global(AgentActivity {
        rows: VecDeque::new(),
        log,
    });
    // Off the main thread: today's earlier rows first, then each new one, written in order, then
    // handed to the main thread to show.
    let dir = folder();
    cx.background_spawn(async move {
        for row in read_day_in(&dir, &today()).log_err().unwrap_or_default() {
            shown.unbounded_send(row).log_err();
        }
        while let Some(row) = logged.next().await {
            append_in(&dir, &today(), &row).log_err();
            shown.unbounded_send(row).log_err();
        }
    })
    .detach();
    cx.spawn(async move |cx| {
        while let Some(row) = to_show.next().await {
            cx.update(|cx| {
                let activity = cx.global_mut::<AgentActivity>();
                activity.rows.push_back(row);
                trim(&mut activity.rows);
            });
        }
    })
    .detach();
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|_, _: &StopAgentControl, _, cx| set_stopped(true, cx));
        workspace.register_action(|_, _: &ResumeAgentControl, _, cx| set_stopped(false, cx));
        workspace.register_action(|_, _: &OpenAgentActivity, window, cx| {
            open_later(cx.weak_entity(), window, cx);
        });
    })
    .detach();
}

/// Logs `call`'s outcome once it is answered, and refuses it while agents' write tools are
/// stopped. `mcp::answer` runs it for every write-tier tool, before the tool's own code.
///
/// # Errors
///
/// [`STOPPED`] while `marley.agent_control.stopped` is on.
pub(crate) fn gate(call: &mut AppCall, cx: &App) -> Result<(), Refusal> {
    log(call, cx);
    if stopped(cx) {
        return Err(Refusal::new(
            STOPPED,
            "The user stopped agents' use of Marley's tools that act. The tools that only read \
             still answer.",
        )
        .next(
            "Ask the user to resume it with `marley: resume agent control`, or the Resume button \
             on Home's AGENT ACTIVITY card.",
        ));
    }
    Ok(())
}

/// Logs `call`'s outcome once it is answered, with no refusal: a read that reveals a file's text
/// (#704) is listed while the kill switch leaves it working.
pub(crate) fn log(call: &mut AppCall, cx: &App) {
    let caller = crate::click_pause::Who::of(call, cx);
    let summary = summary(&call.arguments, cx);
    let tool = call.tool.clone();
    let redactor = crate::mcp::model_redactor(cx);
    if let Some(activity) = cx.try_global::<AgentActivity>() {
        let log = activity.log.clone();
        call.on_answer(move |result| {
            let outcome = match result {
                Ok(answer) => Outcome::Done {
                    result: result_line(answer).map(|line| shortened(&line, &redactor)),
                },
                Err(refusal) => Outcome::Refused {
                    code: refusal.code.to_string(),
                    reason: first_sentence(&refusal.reason)
                        .map(|sentence| shortened(sentence, &redactor)),
                },
            };
            log.unbounded_send(ActivityRow {
                time: now(),
                who: caller.words,
                agent: caller.mark,
                tool,
                summary,
                outcome,
            })
            .log_err();
        });
    }
}

/// The line a row shows for `answer` (#933, D5): a receipt's choice (#367), an exit code, a run
/// still going, a save, else the answer's first line of text.
fn result_line(answer: &ToolAnswer) -> Option<String> {
    let structured = &answer.structured;
    match structured.get("result").and_then(serde_json::Value::as_str) {
        Some("accepted") => return Some("you accepted".to_string()),
        Some("refused") => return Some("you refused".to_string()),
        _ => {}
    }
    if structured
        .get("running")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        return Some("running".to_string());
    }
    if let Some(code) = structured
        .get("exit_code")
        .and_then(serde_json::Value::as_i64)
    {
        return Some(format!("exit {code}"));
    }
    if structured.get("saved").and_then(serde_json::Value::as_bool) == Some(true) {
        return Some("saved".to_string());
    }
    answer
        .text
        .as_deref()?
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

/// `reason`'s first sentence, none when it is empty.
fn first_sentence(reason: &str) -> Option<&str> {
    let reason = reason.trim();
    if reason.is_empty() {
        return None;
    }
    Some(reason.find(". ").map_or(reason, |end| &reason[..=end]))
}

/// `line` redacted and cut to [`RESULT_CHARS`].
fn shortened(line: &str, redactor: &Redactor) -> String {
    cut(redactor.redact(line).text, RESULT_CHARS)
}

/// `text` cut to `chars` characters, an ellipsis marking the cut.
fn cut(text: String, chars: usize) -> String {
    if text.chars().count() > chars {
        let mut cut: String = text.chars().take(chars).collect();
        cut.push('…');
        cut
    } else {
        text
    }
}

/// Whether agents' write tools are stopped.
pub(crate) fn stopped(cx: &App) -> bool {
    MarleySettings::get_global(cx).agent_control == AgentControl::Stopped
}

/// Writes `marley.agent_control.stopped` to the user's settings, when it changes.
pub(crate) fn set_stopped(stopped: bool, cx: &App) {
    if self::stopped(cx) == stopped {
        return;
    }
    settings::update_settings_file(<dyn Fs>::global(cx), cx, move |content, _| {
        content
            .marley
            .get_or_insert_default()
            .agent_control
            .get_or_insert_default()
            .stopped = Some(stopped);
    });
}

/// The `limit` newest rows, newest first.
fn newest(limit: usize, cx: &App) -> Vec<ActivityRow> {
    cx.try_global::<AgentActivity>()
        .map(|activity| activity.rows.iter().rev().take(limit).cloned().collect())
        .unwrap_or_default()
}

fn trim(rows: &mut VecDeque<ActivityRow>) {
    while rows.len() > ROWS_KEPT {
        rows.pop_front();
    }
}

/// What a call acts on, in a line: the first of [`SUMMARY_FIELDS`] it carries, else its
/// arguments, redacted whatever agents' redaction says, and cut to [`SUMMARY_CHARS`].
fn summary(arguments: &serde_json::Value, cx: &App) -> String {
    let picked = SUMMARY_FIELDS
        .iter()
        .find_map(|field| arguments.get(*field))
        .map_or_else(
            || arguments.to_string(),
            |value| match value {
                serde_json::Value::String(text) => text.clone(),
                other => other.to_string(),
            },
        );
    let line = picked.lines().next().unwrap_or_default();
    cut(
        crate::mcp::model_redactor(cx).redact(line).text,
        SUMMARY_CHARS,
    )
}

/// The log's folder under Marley's data directory.
fn folder() -> PathBuf {
    paths::data_dir().join("agent_control")
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// `day`'s file under `dir`.
fn day_file(dir: &Path, day: &str) -> PathBuf {
    dir.join(format!("activity-{day}.jsonl"))
}

/// Appends `row` to `day`'s file under `dir`, making the folder, and the file readable by its
/// owner alone, when they are missing.
///
/// # Errors
///
/// When the folder or the file cannot be made or written.
fn append_in(dir: &Path, day: &str, row: &ActivityRow) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let mut line = serde_json::to_string(row).map_err(io::Error::other)?;
    line.push('\n');
    let mut options = OpenOptions::new();
    let options = options.create(true).append(true);
    #[cfg(unix)]
    let options = {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600)
    };
    options.open(day_file(dir, day))?.write_all(line.as_bytes())
}

/// `day`'s rows under `dir`, oldest first. A missing file is no rows, and a line that does not
/// read is skipped.
///
/// # Errors
///
/// When the file is there and cannot be read.
fn read_day_in(dir: &Path, day: &str) -> io::Result<Vec<ActivityRow>> {
    let text = match fs::read_to_string(day_file(dir, day)) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    Ok(text
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect())
}

/// Opens the Agent Activity tab in the window's Home group, or brings it forward.
pub(crate) fn open_later(workspace: WeakEntity<Workspace>, window: &Window, cx: &mut App) {
    crate::groups::in_group(
        GroupKind::Home,
        workspace,
        window,
        cx,
        |workspace, window, cx| {
            let open = workspace.items_of_type::<AgentActivityView>(cx).next();
            if let Some(view) = open {
                workspace.activate_item(&view, true, true, window, cx);
                return;
            }
            let view = cx.new(|cx| AgentActivityView::new(window, cx));
            workspace.add_item_to_active_pane(Box::new(view), None, true, window, cx);
        },
    );
}

/// The switch's state in words, and the button that flips it, for the tab and Home's card.
pub(crate) fn render_state(id: &'static str, cx: &App) -> impl IntoElement {
    let stopped = stopped(cx);
    let (words, color, label) = if stopped {
        (
            "Stopped: Marley's tools that act refuse every agent. The tools that only read still answer.",
            Color::Error,
            "Resume",
        )
    } else {
        (
            "Agents can use Marley's tools that act; each call is listed here.",
            Color::Muted,
            "Stop",
        )
    };
    h_flex()
        .w_full()
        .gap_2()
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(Label::new(words).size(LabelSize::Small).color(color)),
        )
        .child(
            Button::new(id, label)
                .start_icon(
                    Icon::new(if stopped {
                        IconName::PlayFilled
                    } else {
                        IconName::Stop
                    })
                    .size(IconSize::Small),
                )
                .style(ButtonStyle::Filled)
                .on_click(move |_, _, cx| set_stopped(!stopped, cx)),
        )
}

/// The `limit` newest rows, newest first, as `layout` draws them.
pub(crate) fn render_rows(limit: usize, layout: RowLayout, cx: &App) -> AnyElement {
    let rows = newest(limit, cx);
    if rows.is_empty() {
        return muted("No agent has used a tool that acts yet.");
    }
    v_flex()
        .w_full()
        .when(layout == RowLayout::Card, Styled::gap_2)
        .children(rows.into_iter().map(|row| match layout {
            RowLayout::Card => render_card(row, cx),
            RowLayout::Line => render_line(row, cx),
        }))
        .into_any_element()
}

/// A row's `HH:MM`, or the whole stamp when it has no time.
fn minute(row: &ActivityRow) -> String {
    let clock = row.time.split(' ').nth(1).unwrap_or(&row.time);
    clock.get(..5).unwrap_or(clock).to_string()
}

/// Home's line for a row, as the Home board draws it: time, tool, what (#933, D6).
fn render_line(row: ActivityRow, cx: &App) -> AnyElement {
    let minute = minute(&row);
    let refused = matches!(row.outcome, Outcome::Refused { .. });
    h_flex()
        .w_full()
        .gap_2()
        .px_2()
        .py_0p5()
        .child(
            div().w_10().flex_none().child(
                Label::new(minute)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            ),
        )
        .child(
            div().w_32().flex_none().child(
                Label::new(row.tool)
                    .size(LabelSize::Small)
                    .buffer_font(cx)
                    .color(if refused { Color::Error } else { Color::Accent })
                    .truncate(),
            ),
        )
        .child(
            div().flex_1().min_w_0().child(
                Label::new(row.summary)
                    .size(LabelSize::Small)
                    .color(Color::Muted)
                    .truncate(),
            ),
        )
        .into_any_element()
}

/// The tab's card for a row, as the Tools board draws it: the caller's tile, the tool and the
/// time, what it acted on, and how it ended (#933).
fn render_card(row: ActivityRow, cx: &App) -> AnyElement {
    let colors = cx.theme().colors();
    let (ended, color) = match &row.outcome {
        Outcome::Done { result } => (
            result.clone().unwrap_or_else(|| "done".to_string()),
            Color::Muted,
        ),
        Outcome::Refused { code, reason } => (
            reason.clone().unwrap_or_else(|| format!("refused: {code}")),
            Color::Error,
        ),
    };
    let tile = row.agent.map_or_else(
        || {
            let first = row
                .who
                .chars()
                .next()
                .map_or('?', |first| first.to_ascii_uppercase());
            item_kind::square_tile(ItemKind::Page, first.to_string().into(), px(28.), cx)
        },
        |mark| item_kind::square_tile(ItemKind::Thread(mark), mark.glyph(), px(28.), cx),
    );
    let minute = minute(&row);
    h_flex()
        .w_full()
        .items_start()
        .gap_3()
        .px_3()
        .py_2()
        .rounded_lg()
        .border_1()
        .border_color(colors.border)
        .bg(colors.panel_background)
        .child(tile)
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_0p5()
                .child(
                    h_flex()
                        .w_full()
                        .gap_2()
                        .child(
                            Label::new(row.tool)
                                .size(LabelSize::Small)
                                .buffer_font(cx)
                                .color(Color::Accent),
                        )
                        .child(
                            Label::new(row.who)
                                .size(LabelSize::XSmall)
                                .color(Color::Muted)
                                .truncate(),
                        )
                        .child(div().flex_1())
                        .child(
                            Label::new(minute)
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        ),
                )
                .child(Label::new(row.summary).size(LabelSize::Small).truncate())
                .child(
                    Label::new(ended)
                        .size(LabelSize::XSmall)
                        .color(color)
                        .truncate(),
                ),
        )
        .into_any_element()
}

/// The Agent Activity tab.
pub(crate) struct AgentActivityView {
    focus_handle: FocusHandle,
    _subscriptions: [Subscription; 2],
}

impl std::fmt::Debug for AgentActivityView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentActivityView").finish_non_exhaustive()
    }
}

impl AgentActivityView {
    fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            _subscriptions: [
                cx.observe_global_in::<AgentActivity>(window, |_, _, cx| cx.notify()),
                cx.observe_global_in::<SettingsStore>(window, |_, _, cx| cx.notify()),
            ],
        }
    }
}

impl Render for AgentActivityView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        v_flex()
            .id("marley-agent-activity")
            .debug_selector(|| "marley-agent-activity".into())
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .bg(colors.editor_background)
            .child(Headline::new("Agent Activity").size(HeadlineSize::Small))
            .child(render_state("marley-agent-activity-switch", cx))
            .child(render_rows(ROWS_KEPT, RowLayout::Card, cx))
    }
}

impl Focusable for AgentActivityView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for AgentActivityView {}

impl Item for AgentActivityView {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Agent Activity")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::ListTodo))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static(
            "Every call agents made to Marley's tools that act",
        ))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }
}
