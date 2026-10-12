//! Scheduled runs from the harness (#719).
//!
//! rustal-harness keeps the scheduled tasks, runs each in a session it opens and stops, and
//! records the outcome (MREQ-011); Marley builds no timer. It reads each followed harness's tasks
//! every 15 seconds for the rail, reads a task's runs for its history tab, and raises a run that
//! ended with `finding`: a Needs you entry until the user opens it, which sets the run's session
//! label `finding.seen`, and one desktop notice when the run ended after Marley started.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use gpui::{
    App, AppContext as _, AsyncApp, Context, EventEmitter, FocusHandle, Focusable, Global, Render,
    SharedString, Subscription, SystemNotification, Task, WeakEntity, Window,
};
use marley_fleet::Session;
use serde::Deserialize;
use serde_json::{Value, json};
use ui::{Headline, HeadlineSize, prelude::*};
use util::ResultExt as _;
use workspace::Workspace;
use workspace::item::{Item, ItemEvent};

use crate::groups::GroupKind;
use crate::harness::Slot;

/// How often a followed harness's tasks are read.
const TASK_POLL: Duration = Duration::from_secs(15);
/// The harness's labels on a run's session (rustal-harness `docs/SCHEDULE.md`).
const SCHEDULE_TASK: &str = "schedule.task";
const SCHEDULE_RUN: &str = "schedule.run";
const SCHEDULE_OUTCOME: &str = "schedule.outcome";
/// The label Marley sets on a finding's session once the user opened it. The harness reserves
/// `schedule.`, so it starts with `finding`, as #716's `delegation.told` starts with its own word.
const SEEN: &str = "finding.seen";
/// How many runs the history tab lists.
const HISTORY_RUNS: u64 = 32;

/// The followed harnesses' scheduled tasks, and what Marley did about their findings.
pub(crate) struct Schedules {
    /// When Marley started following: a finding that ended before it raises no notice.
    started_ms: u64,
    slots: HashMap<Slot, SlotSchedules>,
}

impl Default for Schedules {
    fn default() -> Self {
        Self {
            started_ms: crate::harness::now_ms(),
            slots: HashMap::default(),
        }
    }
}

impl Global for Schedules {}

/// One harness's tasks and findings.
#[derive(Default)]
struct SlotSchedules {
    tasks: Vec<ScheduledTask>,
    /// When `schedule_tasks` was last asked.
    read_at: Option<Instant>,
    /// Each finding's first line, by run, once read; empty when it could not be read.
    lines: HashMap<String, String>,
    /// The findings whose first line is being read.
    reading: HashSet<String>,
    /// The findings given their notice, or waiting for their first line to get it.
    notified: HashSet<String>,
    /// The findings whose notice waits for their first line.
    awaiting: HashSet<String>,
    /// The findings the user opened, until the harness publishes their `finding.seen`.
    seen: HashSet<String>,
}

/// A task as `schedule_tasks` lists it.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct ScheduledTask {
    pub(crate) name: String,
    schedule: Schedule,
    paused: bool,
    next_run_ms: Option<u64>,
    last_run: Option<RunLine>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Schedule {
    Every(u64),
    Daily(String),
    Weekdays(String),
}

/// A run as `schedule_runs` lists it; `outcome` and `first_line` are unset while it continues.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct RunLine {
    run: String,
    at_ms: u64,
    outcome: Option<String>,
    first_line: Option<String>,
    session: Option<String>,
}

#[derive(Deserialize)]
struct TaskList {
    tasks: Vec<ScheduledTask>,
}

#[derive(Deserialize)]
struct RunList {
    runs: Vec<RunLine>,
}

/// A run that found something and that the user has not opened yet.
pub(crate) struct Finding {
    pub(crate) session: String,
    pub(crate) run: String,
    pub(crate) task: String,
    /// Its first line, once read.
    pub(crate) line: Option<String>,
}

fn slot_of<'a>(slot: &Slot, cx: &'a App) -> Option<&'a SlotSchedules> {
    cx.try_global::<Schedules>()?.slots.get(slot)
}

fn slot_mut<'a>(slot: &Slot, cx: &'a mut App) -> &'a mut SlotSchedules {
    cx.default_global::<Schedules>()
        .slots
        .entry(slot.clone())
        .or_default()
}

/// `slot`'s tasks, as last read.
pub(crate) fn tasks_of<'a>(slot: &Slot, cx: &'a App) -> &'a [ScheduledTask] {
    slot_of(slot, cx).map_or(&[], |schedules| &schedules.tasks)
}

/// Whether the session is a scheduled run's, which the rail lists under its task.
pub(crate) fn is_run(seat: &Session) -> bool {
    seat.labels.contains_key(SCHEDULE_RUN)
}

/// The run id of a session whose run found something, unless the user opened it.
fn unseen_finding(seat: &Session) -> Option<&str> {
    let outcome = seat.labels.get(SCHEDULE_OUTCOME)?;
    if outcome != "finding" || seat.labels.contains_key(SEEN) {
        return None;
    }
    seat.labels.get(SCHEDULE_RUN).map(String::as_str)
}

/// `slot`'s runs that found something and that the user has not opened.
pub(crate) fn findings(slot: &Slot, cx: &App) -> Vec<Finding> {
    let schedules = slot_of(slot, cx);
    crate::harness::seats_of(slot, cx)
        .seats()
        .iter()
        .filter_map(|seat| {
            let run = unseen_finding(seat)?;
            if schedules.is_some_and(|schedules| schedules.seen.contains(run)) {
                return None;
            }
            Some(Finding {
                session: seat.id.clone(),
                run: run.to_string(),
                task: seat.labels.get(SCHEDULE_TASK).cloned().unwrap_or_default(),
                line: schedules.and_then(|schedules| schedules.lines.get(run).cloned()),
            })
        })
        .collect()
}

/// Reads `slot`'s tasks once [`TASK_POLL`] has passed since the last read. A harness that serves no
/// `schedule_tasks` has none.
pub(crate) async fn poll(slot: &Slot, cx: &AsyncApp) {
    let due = cx.update(|cx| {
        slot_of(slot, cx)
            .and_then(|schedules| schedules.read_at)
            .is_none_or(|at| at.elapsed() >= TASK_POLL)
    });
    if !due {
        return;
    }
    let tasks = match crate::harness::read_tool(slot, "schedule_tasks", json!({}), cx).await {
        Ok(value) => serde_json::from_value::<TaskList>(value).map_or_else(
            |error| {
                log::warn!("harness {slot:?}: schedule_tasks's answer did not parse: {error}");
                Vec::new()
            },
            |list| list.tasks,
        ),
        Err(reason) => {
            log::debug!("harness {slot:?}: no scheduled tasks: {reason}");
            Vec::new()
        }
    };
    cx.update(|cx| {
        let schedules = slot_mut(slot, cx);
        schedules.read_at = Some(Instant::now());
        if schedules.tasks != tasks {
            schedules.tasks = tasks;
        }
    });
}

/// Follows `slot`'s findings after its sessions changed: reads each new one's first line, and gives
/// one notice to each that ended after Marley started.
pub(crate) fn follow(slot: &Slot, cx: &mut App) {
    if !cx.has_global::<Schedules>() {
        cx.set_global(Schedules::default());
    }
    let started_ms = cx.global::<Schedules>().started_ms;
    let schedules = slot_of(slot, cx);
    // Each unseen finding: its run, task and when its session last changed.
    let found: Vec<(String, String, u64)> = crate::harness::seats_of(slot, cx)
        .seats()
        .iter()
        .filter_map(|seat| {
            let run = unseen_finding(seat)?;
            let task = seat.labels.get(SCHEDULE_TASK).cloned().unwrap_or_default();
            Some((run.to_string(), task, seat.last_event_ms))
        })
        .collect();
    let to_read: Vec<(String, String)> = found
        .iter()
        .filter(|(run, _, _)| {
            schedules.is_none_or(|schedules| {
                !schedules.lines.contains_key(run) && !schedules.reading.contains(run)
            })
        })
        .map(|(run, task, _)| (run.clone(), task.clone()))
        .collect();
    let to_notify: Vec<(String, String)> = found
        .iter()
        .filter(|(run, _, ended_ms)| {
            *ended_ms >= started_ms
                && schedules.is_none_or(|schedules| !schedules.notified.contains(run))
        })
        .map(|(run, task, _)| (run.clone(), task.clone()))
        .collect();
    if to_read.is_empty() && to_notify.is_empty() {
        return;
    }
    let schedules = slot_mut(slot, cx);
    let mut ready = Vec::new();
    for (run, task) in to_notify {
        schedules.notified.insert(run.clone());
        match schedules.lines.get(&run) {
            Some(line) => ready.push((run, task, line.clone())),
            None => {
                schedules.awaiting.insert(run);
            }
        }
    }
    for (run, _) in &to_read {
        schedules.reading.insert(run.clone());
    }
    for (run, task, line) in ready {
        notify(slot, &run, &task, &line, cx);
    }
    for (run, task) in to_read {
        read_line(slot.clone(), run, task, cx);
    }
}

/// Reads a finding's first line with `schedule_run`, and gives its notice if it waits for it. A line
/// that cannot be read is kept empty, so it is not asked for again.
fn read_line(slot: Slot, run: String, task: String, cx: &App) {
    cx.spawn(async move |cx| {
        let read =
            crate::harness::read_tool(&slot, "schedule_run", json!({ "run": run }), cx).await;
        let line = match read {
            Ok(value) => value
                .get("first_line")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            Err(reason) => {
                log::warn!("scheduled run {run}: its first line was not read: {reason}");
                String::new()
            }
        };
        cx.update(|cx| {
            let schedules = slot_mut(&slot, cx);
            schedules.reading.remove(&run);
            schedules.lines.insert(run.clone(), line.clone());
            if schedules.awaiting.remove(&run) {
                notify(&slot, &run, &task, &line, cx);
            }
        });
    })
    .detach();
}

/// The desktop notice for a finding, once per run.
fn notify(slot: &Slot, run: &str, task: &str, line: &str, cx: &App) {
    log::info!("harness {slot:?}: scheduled task {task} found something: {line}");
    let title = slot.name().map_or_else(
        || format!("{task} found something"),
        |name| format!("{task} on {name} found something"),
    );
    cx.show_system_notification(SystemNotification {
        tag: SharedString::from(format!("marley-finding-{run}")),
        title: SharedString::from(title),
        body: SharedString::from(line.to_string()),
        actions: Vec::new(),
    });
}

/// Marks a finding opened: at once here, and on its session as `finding.seen`, so the entry does
/// not come back after a restart. Without the harness's write grant only the first holds.
pub(crate) fn mark_seen(slot: &Slot, session: &str, run: &str, cx: &mut App) {
    if !slot_mut(slot, cx).seen.insert(run.to_string()) {
        return;
    }
    let (slot, session, run) = (slot.clone(), session.to_string(), run.to_string());
    cx.spawn(async move |cx| {
        let arguments = json!({
            "id": session,
            "key": SEEN,
            "value": "1",
            "request": uuid::Uuid::new_v4().to_string(),
        });
        if let Err(reason) = crate::harness::write_tool(&slot, "session_label", arguments, cx).await
        {
            log::warn!("scheduled run {run}: {SEEN} not set: {reason}");
        }
    })
    .detach();
}

/// A task's schedule and its next run in words: "every 60 min · next 14:05", "daily 09:00 ·
/// paused".
pub(crate) fn schedule_words(task: &ScheduledTask) -> String {
    let schedule = match &task.schedule {
        Schedule::Every(minutes) => format!("every {minutes} min"),
        Schedule::Daily(at) => format!("daily {at}"),
        Schedule::Weekdays(at) => format!("weekdays {at}"),
    };
    match (task.paused, task.next_run_ms.and_then(local)) {
        (true, _) => format!("{schedule} · paused"),
        (false, Some(next)) => {
            let format = if next.date_naive() == chrono::Local::now().date_naive() {
                "%H:%M"
            } else {
                "%a %H:%M"
            };
            format!("{schedule} · next {}", next.format(format))
        }
        (false, None) => schedule,
    }
}

/// A task's last run in words and its color: "running", or the outcome and its first line.
pub(crate) fn last_run_words(task: &ScheduledTask) -> (String, Color) {
    task.last_run.as_ref().map_or_else(
        || ("no run yet".to_string(), Color::Muted),
        |run| {
            let (outcome, color) = outcome_of(run);
            run.first_line
                .as_deref()
                .filter(|line| !line.is_empty())
                .map_or_else(
                    || (outcome.to_string(), color),
                    |line| (format!("{outcome}: {line}"), color),
                )
        },
    )
}

/// A run's outcome as a word and its color: a finding stands out, a failure reads as one.
fn outcome_of(run: &RunLine) -> (&str, Color) {
    match run.outcome.as_deref() {
        None => ("running", Color::Accent),
        Some(outcome @ "finding") => (outcome, Color::Warning),
        Some(outcome @ "failed") => (outcome, Color::Error),
        Some(outcome) => (outcome, Color::Muted),
    }
}

/// Milliseconds since the epoch in the local clock.
fn local(ms: u64) -> Option<chrono::DateTime<chrono::Local>> {
    chrono::DateTime::from_timestamp_millis(i64::try_from(ms).ok()?)
        .map(|at| at.with_timezone(&chrono::Local))
}

/// Shows the history of `slot`'s task `task` in the window's Home group, or brings it forward.
pub(crate) fn open_history(
    asked_from: WeakEntity<Workspace>,
    slot: Slot,
    task: String,
    window: &Window,
    cx: &mut App,
) {
    crate::groups::in_group(
        GroupKind::Home,
        asked_from,
        window,
        cx,
        move |workspace, window, cx| {
            let open = workspace
                .items_of_type::<ScheduleView>(cx)
                .find(|view| view.read(cx).slot == slot && view.read(cx).task == task);
            if let Some(open) = open {
                workspace.activate_item(&open, true, true, window, cx);
                return;
            }
            let handle = workspace.weak_handle();
            let view = cx.new(|cx| ScheduleView::new(slot, task, handle, window, cx));
            workspace.add_item_to_center(Box::new(view), window, cx);
        },
    );
}

/// A scheduled task's runs, newest first: each run's time, outcome and first line; a click opens
/// the run's session.
pub(crate) struct ScheduleView {
    slot: Slot,
    task: String,
    /// The workspace the tab opened in, where a run's session opens.
    workspace: WeakEntity<Workspace>,
    runs: Vec<RunLine>,
    /// The task's last run when the runs were read: a change reads them again.
    last: Option<RunLine>,
    error: Option<SharedString>,
    focus_handle: FocusHandle,
    reading: Option<Task<()>>,
    _schedules: Subscription,
}

impl std::fmt::Debug for ScheduleView {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScheduleView")
            .field("task", &self.task)
            .field("runs", &self.runs.len())
            .finish_non_exhaustive()
    }
}

impl ScheduleView {
    fn new(
        slot: Slot,
        task: String,
        workspace: WeakEntity<Workspace>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut view = Self {
            slot,
            task,
            workspace,
            runs: Vec::new(),
            last: None,
            error: None,
            focus_handle: cx.focus_handle(),
            reading: None,
            _schedules: cx
                .observe_global_in::<Schedules>(window, |view, _, cx| view.tasks_changed(cx)),
        };
        view.read(cx);
        view
    }

    fn task_of<'a>(&self, cx: &'a App) -> Option<&'a ScheduledTask> {
        tasks_of(&self.slot, cx)
            .iter()
            .find(|task| task.name == self.task)
    }

    fn tasks_changed(&mut self, cx: &mut Context<Self>) {
        let last = self.task_of(cx).and_then(|task| task.last_run.clone());
        if last != self.last {
            self.read(cx);
        }
        cx.notify();
    }

    fn read(&mut self, cx: &Context<Self>) {
        self.last = self.task_of(cx).and_then(|task| task.last_run.clone());
        let (slot, task) = (self.slot.clone(), self.task.clone());
        self.reading = Some(cx.spawn(async move |view, cx| {
            let arguments = json!({ "task": task, "limit": HISTORY_RUNS });
            let runs = crate::harness::read_tool(&slot, "schedule_runs", arguments, cx)
                .await
                .and_then(|value| {
                    serde_json::from_value::<RunList>(value)
                        .map(|list| list.runs)
                        .map_err(|error| {
                            SharedString::from(format!(
                                "schedule_runs's answer did not parse: {error}"
                            ))
                        })
                });
            view.update(cx, |view, cx| {
                match runs {
                    Ok(runs) => {
                        view.runs = runs;
                        view.error = None;
                    }
                    Err(reason) => view.error = Some(reason),
                }
                cx.notify();
            })
            .log_err();
        }));
    }

    fn render_run(run: &RunLine, cx: &Context<Self>) -> AnyElement {
        let time =
            local(run.at_ms).map_or_else(String::new, |at| at.format("%b %-d, %H:%M").to_string());
        let (outcome, color) = outcome_of(run);
        let line = run.first_line.clone().unwrap_or_default();
        let row = h_flex()
            .id(SharedString::from(format!(
                "marley-schedule-run-{}",
                run.run
            )))
            .w_full()
            .gap_2()
            .px_2()
            .py_1()
            .rounded_sm()
            .child(
                div()
                    .w_32()
                    .child(Label::new(time).size(LabelSize::Small).color(Color::Muted)),
            )
            .child(
                div().w_16().child(
                    Label::new(outcome.to_string())
                        .size(LabelSize::Small)
                        .color(color),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(Label::new(line).size(LabelSize::Small).truncate()),
            );
        let Some(session) = run.session.clone() else {
            return row.into_any_element();
        };
        let hover = cx.theme().colors().element_hover;
        row.cursor_pointer()
            .hover(move |style| style.bg(hover))
            .tooltip(ui::Tooltip::text("Open the run's session"))
            .on_click(cx.listener(move |view, _, window, cx| {
                crate::harness::open_in_home(
                    view.workspace.clone(),
                    view.slot.clone(),
                    session.clone(),
                    window,
                    cx,
                );
            }))
            .into_any_element()
    }
}

impl Render for ScheduleView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let summary = self.task_of(cx).map_or_else(
            || "The harness no longer lists this task.".to_string(),
            schedule_words,
        );
        let summary = match self.slot.name() {
            Some(name) => format!("{summary} · on {name}"),
            None => summary,
        };
        let runs: AnyElement = match (&self.error, self.runs.is_empty()) {
            (Some(error), _) => Label::new(error.clone())
                .size(LabelSize::Small)
                .color(Color::Error)
                .into_any_element(),
            (None, true) => Label::new("No runs yet.")
                .size(LabelSize::Small)
                .color(Color::Muted)
                .into_any_element(),
            (None, false) => v_flex()
                .w_full()
                .children(self.runs.iter().map(|run| Self::render_run(run, cx)))
                .into_any_element(),
        };
        v_flex()
            .id("marley-schedule")
            .debug_selector(|| "marley-schedule".into())
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .bg(colors.editor_background)
            .child(Headline::new(self.task.clone()).size(HeadlineSize::Small))
            .child(
                Label::new(summary)
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(runs)
    }
}

impl Focusable for ScheduleView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for ScheduleView {}

impl Item for ScheduleView {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::from(self.task.clone())
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Clock))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::from(format!(
            "The runs of the scheduled task {}",
            self.task
        )))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }
}
