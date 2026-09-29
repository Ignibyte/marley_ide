//! A running command's printed error (#572): a dev server that prints a failure and keeps running
//! tells it once, and once more when it recovers.
//!
//! While `marley.system_one.uses.running_error` is not `off`, each plain terminal's running block
//! is read after its output, at most every 500 ms, from the line the last read ended on
//! (`Terminal::marley_lines_since`). Each line goes through [`scan`]: a failure or a recovery
//! moves the block's [`Episode`]; a line the shapes leave open is asked of the System One layer,
//! whose reading acts as a shape in `act`, marks with a `?` in `suggest`, and is only logged in
//! `shadow`. A failed episode marks the terminal's rail row and, when the terminal is not in
//! front, posts `<project>: <command> printed an error`; a recovery clears the mark and posts
//! `recovered`. Agent CLIs' and SSH clients' terminals are left out, as #551 leaves agents out.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use gpui::{App, Context, EntityId, Focusable as _, Global, Task, Window};
use marley_rail::RunningError;
use marley_system_one::RUNNING_ERROR;
use marley_system_one::reading::{Reading, Signal};
use marley_system_one::request::Answers;
use marley_terminal::BlockState;
use marley_terminal::running_errors::{Change, Episode, Shape, scan};
use settings::{SettingsStore, SystemOneMode};
use terminal::Event;
use terminal_view::TerminalView;
use util::ResultExt as _;

use crate::agent_bar::agent_in;
use crate::links::ssh_in;
use crate::notifications::{looking_at, mark_unread, notify};
use crate::system_one::{self, Asking};

const USE: &str = RUNNING_ERROR.name;

/// How long after output a terminal's new lines are read, at most once in that time.
const SCAN_DELAY: Duration = Duration::from_millis(500);

/// How many of a block's last lines the facts count shapes in.
const RECENT_LINES: usize = 40;

/// The most characters of a line a banner shows, as #538's banners.
const BANNER_BODY_MAX: usize = 180;

/// What each terminal view's running block has come to, by the view's entity id. Nothing observes
/// it: it changes at every read.
#[derive(Default)]
struct Watch(HashMap<EntityId, Watched>);

impl Global for Watch {}

#[derive(Default)]
struct Watched {
    /// The block being read.
    block: Option<usize>,
    /// The absolute line its next read starts at.
    next_line: u64,
    /// The shapes of its last lines, for the facts an open line is asked with.
    recent: VecDeque<Option<Shape>>,
    /// The last line read, the one before an open line that starts the next read.
    last_line: String,
    episode: Episode,
    /// The row of the call that opened the episode, or of its flag's rules row, for its outcome.
    call: Option<String>,
    scan: Option<Task<()>>,
    grace: Option<Task<()>>,
}

/// The rail's marks: each terminal view's failed episode. A global of its own, which the rail
/// observes, so that it changes only with a mark and not at every read.
#[derive(Default)]
pub(crate) struct ErrorMarks(HashMap<EntityId, RunningError>);

impl Global for ErrorMarks {}

/// The failed episode of the terminal view `view`, for its rail row.
pub(crate) fn mark(view: EntityId, cx: &App) -> Option<RunningError> {
    cx.try_global::<ErrorMarks>()
        .and_then(|marks| marks.0.get(&view).cloned())
}

/// A running block the watch reads, with what its banners and questions name.
#[derive(Clone)]
struct Running {
    index: usize,
    output_start: u64,
    command: String,
    started: Option<SystemTime>,
    project: String,
    folders: Vec<PathBuf>,
}

/// A line the shapes left open, with the lines around it.
struct Open {
    before: String,
    line: String,
    after: String,
}

/// Watches every terminal view's running block; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.observe_new(
        |view: &mut TerminalView, window, cx: &mut Context<TerminalView>| {
            let Some(window) = window else { return };
            let terminal = view.terminal().clone();
            cx.subscribe_in(&terminal, window, |_, _, event, window, cx| {
                if matches!(event, Event::Wakeup) {
                    schedule(window, cx);
                }
            })
            .detach();
            // A block's end stamps its times and notifies the terminal, with no event (#551).
            cx.observe_in(&terminal, window, |view, _, _, cx| check_end(view, cx))
                .detach();
            let focus_handle = view.focus_handle(cx);
            cx.on_focus_in(&focus_handle, window, |_, _, cx| seen(cx))
                .detach();
            let id = cx.entity_id();
            cx.on_release(move |_, cx| {
                if cx.has_global::<Watch>() {
                    let _forgotten = cx.global_mut::<Watch>().0.remove(&id);
                }
                set_mark(id, None, cx);
            })
            .detach();
        },
    )
    .detach();
    cx.observe_global::<SettingsStore>(|cx| {
        if system_one::use_mode(USE, cx) == SystemOneMode::Off {
            forget_all(cx);
        }
    })
    .detach();
}

/// Reads the view's new lines once [`SCAN_DELAY`] has passed, unless a read is already coming.
fn schedule(window: &Window, cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    // Read without `default_global` first: it is written at every wakeup otherwise.
    if system_one::use_mode(USE, cx) == SystemOneMode::Off
        || cx
            .try_global::<Watch>()
            .and_then(|watch| watch.0.get(&id))
            .is_some_and(|watched| watched.scan.is_some())
    {
        return;
    }
    let scan = cx.spawn_in(window, async move |this, cx| {
        cx.background_executor().timer(SCAN_DELAY).await;
        this.update_in(cx, |view, window, cx| read(view, window, cx))
            .log_err();
    });
    cx.default_global::<Watch>().0.entry(id).or_default().scan = Some(scan);
}

/// The view's running block, unless it is not one the watch reads: an agent CLI's, whether it
/// still runs or its block names it (L-claude-551), an SSH client's, or a remote project's.
fn running_block(view: &TerminalView, cx: &App) -> Option<Running> {
    let terminal = view.terminal().read(cx);
    let block = terminal
        .blocks()
        .last()
        .filter(|block| block.state == BlockState::Running)?;
    if agent_in(terminal).is_some()
        || marley_agent::agent_kind_of(&block.command).is_some()
        || ssh_in(terminal)
    {
        return None;
    }
    let workspace = view.marley_workspace().upgrade()?;
    let (folders, local) = system_one::project_of(workspace.read(cx), cx);
    if !local {
        return None;
    }
    Some(Running {
        index: block.index,
        output_start: block.output_start,
        command: block
            .command
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
        started: terminal
            .marley_anchored()
            .times(block.index)
            .map(|times| times.started),
        project: system_one::project_name(&folders),
        folders,
    })
}

/// Reads the view's new lines, moves its episode, and asks about the open ones.
fn read(view: &TerminalView, window: &Window, cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    if let Some(running) = watched_mut(id, cx).scan.take() {
        // The task running this read: it ends by itself.
        running.detach();
    }
    let Some(running) = running_block(view, cx) else {
        end(cx);
        return;
    };
    if watched_mut(id, cx).block != Some(running.index) {
        end(cx);
        let watched = watched_mut(id, cx);
        watched.block = Some(running.index);
        watched.next_line = running.output_start;
    }
    let from = watched_mut(id, cx).next_line;
    let Some((text, cursor)) = view.terminal().read(cx).marley_lines_since(from) else {
        return;
    };
    let now = cx.background_executor().now();
    let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    let watched = watched_mut(id, cx);
    watched.next_line = cursor;
    let mut changes = Vec::new();
    let mut opens = Vec::new();
    for (position, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let shape = scan(line);
        if watched.recent.len() == RECENT_LINES {
            let _oldest = watched.recent.pop_front();
        }
        watched.recent.push_back(shape);
        match shape {
            Some(Shape::Failure) => opened_by(watched, line.trim(), false, None, now),
            Some(Shape::Recovery) => changes.extend(watched.episode.recovery(line.trim())),
            Some(Shape::Open) => opens.push(Open {
                before: position
                    .checked_sub(1)
                    .and_then(|before| lines.get(before))
                    .map_or_else(|| watched.last_line.clone(), ToString::to_string),
                line: line.trim().to_string(),
                after: lines
                    .get(position + 1)
                    .map(ToString::to_string)
                    .unwrap_or_default(),
            }),
            None => {}
        }
    }
    if let Some(last) = lines.iter().rev().find(|line| !line.trim().is_empty()) {
        watched.last_line = last.to_string();
    }
    changes.extend(watched.episode.tick(now));
    for change in changes {
        apply(view, &running, change, false, window, cx);
    }
    for open in opens {
        ask(&running, open, window, cx);
    }
    arm_grace(window, cx);
}

fn watched_mut(id: EntityId, cx: &mut App) -> &mut Watched {
    cx.default_global::<Watch>().0.entry(id).or_default()
}

/// A failure `line` for `watched`'s episode; `call` is the reading's row when a reading found it.
fn opened_by(
    watched: &mut Watched,
    line: &str,
    questioned: bool,
    call: Option<String>,
    now: Instant,
) {
    if watched.episode == Episode::Quiet {
        watched.episode.failure(line, questioned, now);
        watched.call = call;
    }
}

/// What the layer is told about `running`: the facts, and `texts` when the project may send them.
fn asking(
    running: &Running,
    id: EntityId,
    texts: Vec<(&'static str, String)>,
    verdict: Option<Answers>,
    cx: &App,
) -> Asking {
    let (failures, recoveries) = cx
        .try_global::<Watch>()
        .and_then(|watch| watch.0.get(&id))
        .map_or((0, 0), |watched| {
            watched
                .recent
                .iter()
                .fold((0, 0), |(failures, recoveries), shape| match shape {
                    Some(Shape::Failure) => (failures + 1, recoveries),
                    Some(Shape::Recovery) => (failures, recoveries + 1),
                    Some(Shape::Open) | None => (failures, recoveries),
                })
        });
    let running_for = running
        .started
        .and_then(|started| SystemTime::now().duration_since(started).ok())
        .map(marley_terminal::duration_label)
        .unwrap_or_default();
    let program = running
        .command
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    Asking {
        subject: id.to_string(),
        project: running.project.clone(),
        folders: running.folders.clone(),
        local: true,
        facts: vec![
            ("project", running.project.clone()),
            ("program", program),
            ("running for", running_for),
            ("failure lines in the last 40", failures.to_string()),
            ("recovery lines in the last 40", recoveries.to_string()),
        ],
        texts,
        verdict,
    }
}

/// Acts on `change` of the view's episode. A change the shapes made, not `by_reading`, leaves a
/// `rules` row of its own (#565's D6); a reading's call is its row.
fn apply(
    view: &TerminalView,
    running: &Running,
    change: Change,
    by_reading: bool,
    window: &Window,
    cx: &mut Context<TerminalView>,
) {
    let id = cx.entity_id();
    match change {
        Change::Flag { line, questioned } => {
            if watched_mut(id, cx).call.is_none() {
                let asking = asking(
                    running,
                    id,
                    vec![("line", line.clone())],
                    Some(system_one::noul_verdict("new_failure", true)),
                    cx,
                );
                let asked = system_one::record(RUNNING_ERROR, &asking, cx);
                watched_mut(id, cx).call = asked.row.map(|row| row.id);
            }
            set_mark(
                id,
                Some(RunningError {
                    line: line.clone(),
                    questioned,
                }),
                cx,
            );
            if !questioned && !looking_at(view, window, cx) {
                mark_unread(cx);
                let title = format!("{}: {} printed an error", running.project, running.command);
                let body = util::truncate_and_trailoff(&line, BANNER_BODY_MAX);
                notify(view, Some(&title), &body, window, cx);
            }
        }
        Change::Recovered { line, told } => {
            if !by_reading {
                let asking = asking(
                    running,
                    id,
                    vec![("line", line.clone())],
                    Some(system_one::noul_verdict("recovered", true)),
                    cx,
                );
                let _recorded = system_one::record(RUNNING_ERROR, &asking, cx);
            }
            if let Some(call) = watched_mut(id, cx).call.take() {
                system_one::outcome(&call, "recovered".to_string(), cx);
            }
            set_mark(id, None, cx);
            if told {
                let title = format!("{}: {} recovered", running.project, running.command);
                let body = util::truncate_and_trailoff(&line, BANNER_BODY_MAX);
                notify(view, Some(&title), &body, window, cx);
            }
        }
        Change::Cleared => {
            if let Some(call) = watched_mut(id, cx).call.take() {
                system_one::outcome(&call, "ended".to_string(), cx);
            }
            set_mark(id, None, cx);
        }
    }
}

/// Asks the layer about `open`, and acts on the reading by the use's mode.
fn ask(running: &Running, open: Open, window: &Window, cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    let Open {
        before,
        line,
        after,
    } = open;
    // A line at the start or the end of what was read has no line on that side to send.
    let texts = [("before", before), ("line", line.clone()), ("after", after)]
        .into_iter()
        .filter(|(_, text)| !text.trim().is_empty())
        .collect();
    let asking = asking(running, id, texts, None, cx);
    let asked = system_one::ask(RUNNING_ERROR, &asking, cx);
    let running = running.clone();
    cx.spawn_in(window, async move |this, cx| {
        let asked = asked.await;
        this.update_in(cx, |view, window, cx| {
            answered(view, &running, &line, &asked, window, cx);
        })
        .log_err();
    })
    .detach();
}

/// What a reading about the open `line` says: a new failure opens the episode in `act`, and in
/// `suggest` as a question; a recovery closes it in `act`, and in `suggest` only a questioned one.
fn answered(
    view: &TerminalView,
    running: &Running,
    line: &str,
    asked: &system_one::Asked,
    window: &Window,
    cx: &mut Context<TerminalView>,
) {
    let id = cx.entity_id();
    let row = asked
        .row
        .as_ref()
        .filter(|_| !asked.reading.failed())
        .map(|row| row.id.clone());
    if watched_mut(id, cx).block != Some(running.index) {
        if let Some(call) = row {
            let outcome = "the block ended before the reading came".to_string();
            system_one::outcome(&call, outcome, cx);
        }
        return;
    }
    let holds = |key: &str| match &asked.reading {
        Reading::Model(reads) => reads
            .iter()
            .any(|read| read.key == key && matches!(read.signal, Signal::Noul { holds: true, .. })),
        Reading::Off | Reading::Rules(_) | Reading::Refused(_) | Reading::Unavailable(_) => false,
    };
    let (failure, recovered) = (holds("new_failure"), holds("recovered"));
    let now = cx.background_executor().now();
    let watched = watched_mut(id, cx);
    let questioned_episode = watched
        .episode
        .mark()
        .is_some_and(|(_, questioned)| questioned);
    let change = match asked.mode {
        SystemOneMode::Act | SystemOneMode::Suggest if failure => {
            let questioned = asked.mode == SystemOneMode::Suggest;
            opened_by(watched, line, questioned, row, now);
            None
        }
        SystemOneMode::Act if recovered => watched.episode.recovery(line),
        SystemOneMode::Suggest if recovered && questioned_episode => watched.episode.recovery(line),
        SystemOneMode::Off
        | SystemOneMode::Shadow
        | SystemOneMode::Act
        | SystemOneMode::Suggest => None,
    };
    if let Some(change) = change {
        apply(view, running, change, true, window, cx);
    }
    arm_grace(window, cx);
}

/// Starts the timer that fails a suspect episode when its grace ends, unless one runs.
fn arm_grace(window: &Window, cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    let now = cx.background_executor().now();
    let watched = watched_mut(id, cx);
    let Some(due) = watched.episode.due().filter(|_| watched.grace.is_none()) else {
        return;
    };
    let wait = due.saturating_duration_since(now);
    let grace = cx.spawn_in(window, async move |this, cx| {
        cx.background_executor().timer(wait).await;
        this.update_in(cx, |view, window, cx| grace_ended(view, window, cx))
            .log_err();
    });
    watched_mut(id, cx).grace = Some(grace);
}

/// A suspect's grace ended: it fails if its block still runs.
fn grace_ended(view: &TerminalView, window: &Window, cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    if let Some(grace) = watched_mut(id, cx).grace.take() {
        // The task running this check: it ends by itself.
        grace.detach();
    }
    let Some(running) = running_block(view, cx) else {
        end(cx);
        return;
    };
    let now = cx.background_executor().now();
    let change = watched_mut(id, cx).episode.tick(now);
    if let Some(change) = change {
        apply(view, &running, change, false, window, cx);
    }
    arm_grace(window, cx);
}

/// Ends the watch of the view's block once it is not the running last block any more.
fn check_end(view: &TerminalView, cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    let Some(block) = cx
        .try_global::<Watch>()
        .and_then(|watch| watch.0.get(&id))
        .and_then(|watched| watched.block)
    else {
        return;
    };
    let still_running = view
        .terminal()
        .read(cx)
        .blocks()
        .last()
        .is_some_and(|last| last.index == block && last.state == BlockState::Running);
    if !still_running {
        end(cx);
    }
}

/// Forgets the view's block: a failed episode's mark goes, and its call's outcome is the end.
fn end(cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    if !cx.has_global::<Watch>() {
        return;
    }
    let Some(watched) = cx.global_mut::<Watch>().0.get_mut(&id) else {
        return;
    };
    let change = watched.episode.ended();
    let call = watched.call.take();
    watched.block = None;
    watched.recent.clear();
    watched.last_line.clear();
    watched.grace = None;
    if let Some((Change::Cleared, call)) = change.zip(call) {
        system_one::outcome(&call, "ended".to_string(), cx);
    }
    set_mark(id, None, cx);
}

/// The user looked at the terminal while it was flagged: its call's outcome.
fn seen(cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    if mark(id, cx).is_none() {
        return;
    }
    if let Some(call) = watched_mut(id, cx).call.take() {
        system_one::outcome(&call, "seen".to_string(), cx);
    }
}

/// Sets or clears `id`'s mark, touching the observed global only when the mark changes.
fn set_mark(id: EntityId, mark: Option<RunningError>, cx: &mut App) {
    let current = cx
        .try_global::<ErrorMarks>()
        .and_then(|marks| marks.0.get(&id));
    if current == mark.as_ref() {
        return;
    }
    let marks = &mut cx.default_global::<ErrorMarks>().0;
    match mark {
        Some(mark) => {
            let _earlier = marks.insert(id, mark);
        }
        None => {
            let _cleared = marks.remove(&id);
        }
    }
}

/// The use went off: every watch and mark goes.
fn forget_all(cx: &mut App) {
    if cx
        .try_global::<Watch>()
        .is_some_and(|watch| !watch.0.is_empty())
    {
        cx.global_mut::<Watch>().0.clear();
    }
    if cx
        .try_global::<ErrorMarks>()
        .is_some_and(|marks| !marks.0.is_empty())
    {
        cx.global_mut::<ErrorMarks>().0.clear();
    }
}
