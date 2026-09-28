//! The stall watch (#569), which flags a working Claude Code that loops or has gone quiet.
//!
//! A seat that repeats one tool line is flagged `looping?` at once, and one that has gone quiet is
//! sampled and judged at its checks, the System One layer asked about what the facts leave open.
//!
//! `marley_agent::stall` decides. This module keeps the samples, the quiet episodes and the calls
//! waiting for their outcome in a global no view observes, so its ticks redraw nothing; only
//! landing a flag writes the seat, which the rail and `fleet_snapshot` read, as #566 lands a stop's
//! kind. In `act` a flag posts one banner. Nothing here stops, interrupts or types into the agent:
//! the flag only marks the row.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use gpui::{App, AppContext as _, AsyncApp, Entity, Global};
use marley_agent::claude_events::{
    self, PROMPT_ID_LABEL, PROMPT_LABEL, SESSION_LABEL, TOOL_LABEL, TurnFacts,
};
use marley_agent::stall::{self, Facts, Flag, FlagShown, Stalled, Verdict};
use marley_fleet::{Session, State};
use marley_system_one::STALL_KIND;
use marley_system_one::reading::{Reading, Signal};
use settings::{Settings as _, SettingsStore, SystemOneMode};
use terminal_view::TerminalView;
use util::ResultExt as _;
use workspace::Workspace;

use crate::MarleySettings;
use crate::agent_events::{self, AgentEvents};
use crate::system_one::{self, Asked, Asking};

/// The use's name, which its mode is set under.
const USE: &str = "stall_kind";

/// The least and the most the watch waits between two ticks.
const MIN_TICK: Duration = Duration::from_secs(2);
const MAX_TICK: Duration = Duration::from_secs(10);

/// How many of the terminal's last lines the state carries.
const SCREEN_LINES: usize = 5;

/// The state's labels for the terminal's last lines.
const LINE_LABELS: [&str; SCREEN_LINES] = ["line 1", "line 2", "line 3", "line 4", "line 5"];

/// A sample of a seat's turn's CPU.
#[derive(Debug, Clone, Copy)]
struct Sample {
    ticks: u64,
    at_ms: u64,
}

/// A working seat's quiet since its last event.
#[derive(Debug, Default)]
struct Episode {
    /// The seat's last event, which the episode is the quiet after.
    since_ms: u64,
    /// How many of the checks were asked.
    asked: usize,
    /// Whether an ask is on its way.
    asking: bool,
    /// Whether the banner went.
    bannered: bool,
}

/// A seat's call waiting for its outcome, in any mode but `off`: its row, when it was read, and
/// for a loop its line, which stays the loop's until another line ends.
#[derive(Debug)]
struct Pending {
    call: String,
    at_ms: u64,
    looped: Option<String>,
}

/// The watch: each seat's last two samples, its quiet episode and its call waiting for an outcome,
/// and whether the timer runs. No view observes it.
#[derive(Debug, Default)]
pub(crate) struct StallWatch {
    samples: HashMap<String, (Option<Sample>, Sample)>,
    episodes: HashMap<String, Episode>,
    pending: HashMap<String, Pending>,
    boot_secs: Option<u64>,
    running: bool,
}

impl Global for StallWatch {}

/// Arms the watch at a settings change, for a use turned on while a seat is already quiet.
pub(crate) fn init(cx: &mut App) {
    cx.observe_global::<SettingsStore>(watch).detach();
}

/// Whether the stall kind's flags show on the rows: in `suggest` and `act`.
pub(crate) fn flag_shown(cx: &App) -> FlagShown {
    match system_one::use_mode(USE, cx) {
        SystemOneMode::Suggest | SystemOneMode::Act => FlagShown::Shown,
        SystemOneMode::Off | SystemOneMode::Shadow => FlagShown::Hidden,
    }
}

/// The quiet checks the settings set, in milliseconds after a seat's last event.
fn checks(cx: &App) -> Vec<u64> {
    stall::checks(MarleySettings::get_global(cx).stall_check_after_seconds)
}

/// Starts the watch's timer while the use is on and has checks to make; the timer ends itself
/// when no seat works.
pub(crate) fn watch(cx: &mut App) {
    let first = checks(cx).first().copied();
    let Some(first) = first.filter(|_| system_one::use_mode(USE, cx) != SystemOneMode::Off) else {
        return;
    };
    if cx
        .try_global::<StallWatch>()
        .is_some_and(|watch| watch.running)
    {
        return;
    }
    cx.default_global::<StallWatch>().running = true;
    let tick = Duration::from_millis(first / 4).clamp(MIN_TICK, MAX_TICK);
    cx.spawn(async move |cx| {
        loop {
            cx.background_executor().timer(tick).await;
            if !run_tick(cx).await {
                break;
            }
        }
        cx.update(|cx| cx.default_global::<StallWatch>().running = false);
    })
    .detach();
}

/// A working seat the tick samples: its terminal's foreground pid and its turn's start.
struct ToSample {
    seat: String,
    pid: i32,
    since_ms: u64,
}

/// One tick: the samples of the working seats quiet half a check or more, read off the main
/// thread, then the judgement of each. False when the watch should stop: the use off, no checks,
/// or no seat working.
async fn run_tick(cx: &AsyncApp) -> bool {
    let Some((to_sample, boot_secs)) = cx.update(|cx| plan(cx)) else {
        return false;
    };
    let read = cx
        .background_spawn(futures::future::lazy(move |_| {
            let root = Path::new("/proc");
            let boot_secs = boot_secs.or_else(|| stall::boot_time_in(root));
            let samples: Vec<(String, Option<u64>)> = to_sample
                .into_iter()
                .map(|sample| {
                    // `btime` is whole seconds, so the turn's start may read up to a second late:
                    // a second's slack keeps a tool started at once in the turn.
                    let since = boot_secs.map_or(0, |boot| {
                        stall::ticks_since_boot(sample.since_ms, boot)
                            .saturating_sub(stall::TICKS_PER_SECOND)
                    });
                    (sample.seat, stall::tree_cpu_in(root, sample.pid, since))
                })
                .collect();
            (samples, boot_secs)
        }))
        .await;
    cx.update(|cx| decide(read.0, read.1, cx));
    true
}

/// What the tick samples, and the boot time known: `None` when the watch should stop.
fn plan(cx: &App) -> Option<(Vec<ToSample>, Option<u64>)> {
    let first = checks(cx).first().copied()?;
    if system_one::use_mode(USE, cx) == SystemOneMode::Off {
        return None;
    }
    let seats: Vec<Session> = working_seats(cx);
    if seats.is_empty() {
        return None;
    }
    let now = agent_events::now_ms();
    let views = crate::mcp::terminals(cx);
    let to_sample = seats
        .iter()
        .filter(|seat| now.saturating_sub(seat.last_event_ms) >= first / 2)
        .filter_map(|seat| {
            let (_, view) = views
                .iter()
                .find(|(_, view)| view.entity_id().as_u64().to_string() == seat.id)?;
            let pid = view.read(cx).terminal().read(cx).pid()?;
            Some(ToSample {
                seat: seat.id.clone(),
                pid: i32::try_from(pid.as_u32()).ok()?,
                since_ms: TurnFacts::of(&seat.labels).started_ms,
            })
        })
        .collect();
    let boot_secs = cx
        .try_global::<StallWatch>()
        .and_then(|watch| watch.boot_secs);
    Some((to_sample, boot_secs))
}

/// The seats that work now.
fn working_seats(cx: &App) -> Vec<Session> {
    cx.try_global::<AgentEvents>()
        .map(|events| {
            events
                .snapshot()
                .seats()
                .iter()
                .filter(|seat| seat.state == State::Working)
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

/// Keeps the tick's samples, then judges each working seat at its next check and asks about an
/// open one.
fn decide(samples: Vec<(String, Option<u64>)>, boot_secs: Option<u64>, cx: &mut App) {
    let now = agent_events::now_ms();
    let seats = working_seats(cx);
    let checks = checks(cx);
    let watch = cx.default_global::<StallWatch>();
    watch.boot_secs = boot_secs;
    for (seat, ticks) in samples {
        match ticks {
            Some(ticks) => {
                let sample = Sample { ticks, at_ms: now };
                let _kept = watch
                    .samples
                    .entry(seat)
                    .and_modify(|(previous, latest)| {
                        *previous = Some(*latest);
                        *latest = sample;
                    })
                    .or_insert((None, sample));
            }
            None => {
                let _gone = watch.samples.remove(&seat);
            }
        }
    }
    let working: Vec<&str> = seats.iter().map(|seat| seat.id.as_str()).collect();
    watch
        .samples
        .retain(|seat, _| working.contains(&seat.as_str()));
    watch
        .episodes
        .retain(|seat, _| working.contains(&seat.as_str()));
    watch
        .pending
        .retain(|seat, _| working.contains(&seat.as_str()));
    let mut open = Vec::new();
    for seat in &seats {
        let episode = watch.episodes.entry(seat.id.clone()).or_default();
        if episode.since_ms != seat.last_event_ms {
            *episode = Episode {
                since_ms: seat.last_event_ms,
                ..Episode::default()
            };
        }
        if episode.asking || seat.labels.contains_key(stall::FLAG_LABEL) {
            continue;
        }
        let cpu_active = watch.samples.get(&seat.id).and_then(|(previous, latest)| {
            // Only two samples inside this quiet say what it burns.
            let previous = (*previous).filter(|previous| previous.at_ms >= seat.last_event_ms)?;
            Some(stall::active(
                latest.ticks.saturating_sub(previous.ticks),
                latest.at_ms.saturating_sub(previous.at_ms),
            ))
        });
        let facts = Facts {
            quiet_ms: now.saturating_sub(seat.last_event_ms),
            tool_in_flight: seat.labels.get(TOOL_LABEL).cloned(),
            cpu_active,
        };
        if stall::judge(&facts, checks.get(episode.asked).copied()) == Verdict::Open {
            // A watch that starts late asks once for the checks already passed.
            episode.asked = checks
                .iter()
                .filter(|check| **check <= facts.quiet_ms)
                .count();
            episode.asking = true;
            open.push((seat.clone(), facts));
        }
    }
    for (seat, facts) in open {
        ask(&seat, &facts, cx);
    }
}

/// The seat's terminal view and its workspace.
fn view_of(seat: &str, cx: &App) -> Option<(Entity<Workspace>, Entity<TerminalView>)> {
    crate::mcp::terminals(cx)
        .into_iter()
        .find(|(_, view)| view.entity_id().as_u64().to_string() == seat)
}

/// The project a terminal's workspace holds: its name, folders, and whether it is on this machine.
fn project_of(workspace: &Entity<Workspace>, cx: &App) -> (String, Vec<std::path::PathBuf>, bool) {
    let (folders, local) = system_one::project_of(workspace.read(cx), cx);
    (system_one::project_name(&folders), folders, local)
}

/// What a quiet seat's flag rests on, in words.
fn quiet_reason(facts: &Facts) -> String {
    let tool = facts.tool_in_flight.as_ref().map_or_else(
        || "no tool in flight".to_string(),
        |tool| format!("{tool} in flight"),
    );
    let cpu = if facts.cpu_active == Some(true) {
        "its processes use the CPU"
    } else {
        "no CPU in the turn's tools"
    };
    format!(
        "quiet {}, {tool}, {cpu}",
        stall::quiet_words(facts.quiet_ms)
    )
}

/// Asks the layer what the open `seat` is doing, and lands a stalled flag its answer earns.
fn ask(seat: &Session, facts: &Facts, cx: &mut App) {
    let Some(asking) = asking_for(seat, facts, cx) else {
        forget_asking(&seat.id, cx);
        return;
    };
    let landing = Landing::of(seat, quiet_reason(facts));
    let asked = system_one::ask(STALL_KIND, &asking, cx);
    cx.spawn(async move |cx| {
        let asked = asked.await;
        cx.update(|cx| answered(&landing, &asked, cx));
    })
    .detach();
}

/// What the stall kind asks about the open `seat`: the facts code knows, and as text the prompt,
/// the tool line and the terminal's last lines. `None` when its terminal is gone.
fn asking_for(seat: &Session, facts: &Facts, cx: &App) -> Option<Asking> {
    let (workspace, view) = view_of(&seat.id, cx)?;
    let (project, folders, local) = project_of(&workspace, cx);
    let label = |key: &str| seat.labels.get(key).cloned().unwrap_or_default();
    let cpu = if facts.cpu_active == Some(true) {
        "yes"
    } else {
        "no"
    };
    let state_facts = vec![
        ("project", project.clone()),
        ("agent", "Claude Code".to_string()),
        ("quiet", stall::quiet_words(facts.quiet_ms)),
        // The tool's name is a fact; what it acts on is text, which a metadata-only project
        // keeps back.
        (
            "tool in flight",
            facts
                .tool_in_flight
                .as_deref()
                .map_or("none", stall::tool_name)
                .to_string(),
        ),
        ("tools use the CPU", cpu.to_string()),
        (
            "subagents",
            claude_events::subagents(&seat.labels).to_string(),
        ),
        ("permission mode", label("permission_mode")),
    ];
    let lines = view
        .read(cx)
        .terminal()
        .read(cx)
        .last_n_non_empty_lines(SCREEN_LINES);
    let texts = [("prompt", label(PROMPT_LABEL)), ("tool", label(TOOL_LABEL))]
        .into_iter()
        .chain(LINE_LABELS.into_iter().zip(lines))
        .filter(|(_, text)| !text.is_empty())
        .collect();
    Some(Asking {
        subject: seat.id.clone(),
        project,
        folders,
        local,
        facts: state_facts,
        texts,
        verdict: None,
    })
}

/// The answer to the ask `landing` came from: the call waits for its outcome, and a stalled kind
/// in `suggest` or `act` lands its flag, with one banner for the episode in `act`.
fn answered(landing: &Landing, asked: &Asked, cx: &mut App) {
    forget_asking(&landing.seat, cx);
    // A refused or failed call read nothing, so what the agent does next says nothing of it.
    let row = asked
        .row
        .as_ref()
        .filter(|_| !asked.reading.failed())
        .map(|row| row.id.clone());
    if !landing.still(cx) {
        if let Some(call) = row {
            let outcome = "the agent moved on before the reading came back".to_string();
            system_one::outcome(&call, outcome, cx);
        }
        return;
    }
    if let Some(call) = row {
        let pending = Pending {
            call,
            at_ms: agent_events::now_ms(),
            looped: None,
        };
        hold(&landing.seat, pending, cx);
    }
    let kind = match &asked.reading {
        Reading::Model(reads) => reads.iter().find_map(|read| match &read.signal {
            Signal::Choice { option, confidence } if read.key == "kind" => {
                Stalled::from_option(option).map(|kind| (kind, *confidence))
            }
            _ => None,
        }),
        Reading::Off | Reading::Rules(_) | Reading::Refused(_) | Reading::Unavailable(_) => None,
    };
    let Some((kind, confidence)) = kind else {
        return;
    };
    if !matches!(asked.mode, SystemOneMode::Suggest | SystemOneMode::Act) {
        return;
    }
    let labels = stall::labels(Flag::Stalled(kind), "model", confidence, &landing.reason);
    if !landing.land(labels, cx) {
        return;
    }
    let body = format!("{} ({confidence:.2}): {}", kind.words(), landing.reason);
    let episode = cx
        .default_global::<StallWatch>()
        .episodes
        .entry(landing.seat.clone())
        .or_default();
    let banner = asked.mode == SystemOneMode::Act && !episode.bannered;
    episode.bannered |= banner;
    if banner {
        post_banner(&landing.seat, &body, cx);
    }
}

/// The episode of `seat` asks no more for now.
fn forget_asking(seat: &str, cx: &mut App) {
    if let Some(episode) = cx.default_global::<StallWatch>().episodes.get_mut(seat) {
        episode.asking = false;
    }
}

/// Keeps `pending` as `seat`'s call waiting for its outcome. A call it replaces is settled: a
/// quiet reading saw no event before the next check, and a loop went on until the agent went
/// quiet.
fn hold(seat: &str, pending: Pending, cx: &mut App) {
    let replaced = cx
        .default_global::<StallWatch>()
        .pending
        .insert(seat.to_string(), pending);
    if let Some(replaced) = replaced {
        let after = stall::quiet_words(agent_events::now_ms().saturating_sub(replaced.at_ms));
        let outcome = if replaced.looped.is_some() {
            format!("the loop went on, then the agent went quiet, {after} later")
        } else {
            format!("still quiet {after} later")
        };
        system_one::outcome(&replaced.call, outcome, cx);
    }
}

/// Posts the banner that `seat`'s agent may be stuck, in its terminal's window.
fn post_banner(seat: &str, body: &str, cx: &mut App) {
    let Some((workspace, view)) = view_of(seat, cx) else {
        return;
    };
    let Some(window) = crate::browser::window_of(&workspace, cx) else {
        return;
    };
    let (project, _, _) = project_of(&workspace, cx);
    let title = format!("{project}: Claude Code may be stuck");
    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                crate::notifications::notify_stall(view, &title, body, window, cx);
            });
        })
        .log_err();
}

/// Where a flag lands, and the guard it lands behind: the seat as it was when the flag was
/// decided, which must still hold when it lands (#566's rule).
struct Landing {
    seat: String,
    since_ms: u64,
    session: Option<String>,
    prompt_id: Option<String>,
    stops: u32,
    reason: String,
}

impl Landing {
    fn of(seat: &Session, reason: String) -> Self {
        Self {
            seat: seat.id.clone(),
            since_ms: seat.last_event_ms,
            session: seat.labels.get(SESSION_LABEL).cloned(),
            prompt_id: seat.labels.get(PROMPT_ID_LABEL).cloned(),
            stops: TurnFacts::of(&seat.labels).stops,
            reason,
        }
    }

    /// Whether `seat` is still as the flag found it: working on the same prompt, with no event
    /// since.
    fn holds(&self, seat: &Session) -> bool {
        seat.state == State::Working
            && seat.last_event_ms == self.since_ms
            && seat.labels.get(SESSION_LABEL) == self.session.as_ref()
            && seat.labels.get(PROMPT_ID_LABEL) == self.prompt_id.as_ref()
            && TurnFacts::of(&seat.labels).stops == self.stops
    }

    /// Whether the seat is still as the flag found it.
    fn still(&self, cx: &App) -> bool {
        cx.try_global::<AgentEvents>()
            .and_then(|events| events.snapshot().get(&self.seat))
            .is_some_and(|seat| self.holds(seat))
    }

    /// Puts `labels` on the seat while it is still as the flag found it, at the seat's own time,
    /// so the quiet clock stands. Whether they landed.
    fn land(&self, labels: Vec<(&'static str, String)>, cx: &mut App) -> bool {
        agent_events::land_labels(&self.seat, |seat| self.holds(seat), labels, cx)
    }
}

/// After a lead tool ended in `seat`, flags it `looping?` when its turn repeats one tool line:
/// a `rules` row whatever the provider, and in `suggest` or `act` the flag, from the rule alone.
pub(crate) fn note_tool_end(view: &TerminalView, seat: &Session, cx: &mut App) {
    let mode = system_one::use_mode(USE, cx);
    if mode == SystemOneMode::Off || seat.labels.contains_key(stall::FLAG_LABEL) {
        return;
    }
    let Some(repeat) = stall::repeats(&TurnFacts::of(&seat.labels).tool_lines) else {
        return;
    };
    // A loop is logged once: in `shadow` no label lands to say it was.
    let logged = cx
        .try_global::<StallWatch>()
        .and_then(|watch| watch.pending.get(&seat.id))
        .is_some_and(|pending| pending.looped.as_deref() == Some(repeat.line.as_str()));
    if logged {
        return;
    }
    let (project, folders, local) = view.marley_workspace().upgrade().map_or_else(
        || ("a project".to_string(), Vec::new(), true),
        |workspace| project_of(&workspace, cx),
    );
    let ended = if repeat.failed { "failed" } else { "ended" };
    let facts = vec![
        ("project", project.clone()),
        ("tool", stall::tool_name(&repeat.line).to_string()),
        (
            "repeats",
            format!("{ended} {} times in a row", repeat.times),
        ),
    ];
    let asking = Asking {
        subject: seat.id.clone(),
        project,
        folders,
        local,
        facts,
        texts: vec![("tool line", repeat.line.clone())],
        verdict: Some(system_one::noul_verdict("repeating", true)),
    };
    let asked = system_one::record(STALL_KIND, &asking, cx);
    if let Some(row) = &asked.row {
        let pending = Pending {
            call: row.id.clone(),
            at_ms: agent_events::now_ms(),
            looped: Some(repeat.line.clone()),
        };
        hold(&seat.id, pending, cx);
    }
    if !matches!(asked.mode, SystemOneMode::Suggest | SystemOneMode::Act) {
        return;
    }
    let reason = format!("{} {ended} {} times in a row", repeat.line, repeat.times);
    let landing = Landing::of(seat, reason);
    let labels = stall::labels(Flag::Looping, "rules", 1.0, &landing.reason);
    let _landed = landing.land(labels, cx);
}

/// `seat` moved on with `event`: the outcome of its call waiting for one, unless that is a loop
/// that goes on. The fold took the flag off at the same event, or keeps a loop's (#569).
pub(crate) fn moved(seat: &Session, event: &str, cx: &mut App) {
    let Some(pending) = cx
        .try_global::<StallWatch>()
        .and_then(|watch| watch.pending.get(&seat.id))
    else {
        return;
    };
    let looping = pending.looped.as_ref().is_some_and(|line| {
        seat.state == State::Working
            && stall::repeats(&TurnFacts::of(&seat.labels).tool_lines)
                .is_some_and(|repeat| &repeat.line == line)
    });
    if looping {
        return;
    }
    let Some(pending) = cx.default_global::<StallWatch>().pending.remove(&seat.id) else {
        return;
    };
    let after = stall::quiet_words(agent_events::now_ms().saturating_sub(pending.at_ms));
    let outcome = if pending.looped.is_some() {
        format!("the loop ended {after} later: {event}")
    } else {
        format!("the next event {after} later: {event}")
    };
    system_one::outcome(&pending.call, outcome, cx);
}
