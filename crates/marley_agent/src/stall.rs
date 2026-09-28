//! Stalled or looping agents (#569): what code decides before a model is asked.
//!
//! It reads a working Claude Code seat that repeats itself or has gone quiet, and says how the
//! flag it earns rides on the seat.
//!
//! A loop is the same tool line ended three times in a row, or the same line failing twice in a
//! row: an edit and a test run in turn are an ordinary cycle, not a loop, and neither is a run of
//! edits or reads of one file, whose lines name only the file. A quiet seat is judged
//! at its checks: a tool whose processes burn CPU is a long task, and anything else quiet past a
//! check is open, which the System One layer may read. The flag only marks the row: nothing here
//! or in its users stops, interrupts or types into the agent. Pure, but for the `/proc` readers,
//! which take their root, so a scenario's may stand in.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use procfs_core::FromRead as _;
use procfs_core::process::Stat;

use crate::claude_events::ToolLine;

/// The label for a seat's flag: `looping`, or `stalled:<kind>`.
pub const FLAG_LABEL: &str = "flag";
/// The label for who set the flag: `rules` or `model`.
pub const FLAG_SOURCE_LABEL: &str = "flag_source";
/// The label for how sure the flag is, such as `0.91`.
pub const FLAG_CONFIDENCE_LABEL: &str = "flag_confidence";
/// The label for what the flag rests on, in words, for the row's tooltip.
pub const FLAG_REASON_LABEL: &str = "flag_reason";
/// The flag's labels.
pub(crate) const FLAG_LABELS: [&str; 4] = [
    FLAG_LABEL,
    FLAG_SOURCE_LABEL,
    FLAG_CONFIDENCE_LABEL,
    FLAG_REASON_LABEL,
];

/// How many times in a row one tool line ends before it is a loop.
const REPEATED: usize = 3;

/// How many failures in a row of one tool line make a loop.
const FAILED: usize = 2;

/// The tools whose line names only the file they act on, so that several ends in a row on one
/// file are ordinary work, such as a run of edits; their failures in a row still count.
const FILE_TOOLS: [&str; 5] = ["Read", "Write", "Edit", "MultiEdit", "NotebookEdit"];

/// The clock ticks a second of `/proc`'s times: Linux's `USER_HZ`, which is 100.
pub const TICKS_PER_SECOND: u64 = 100;

/// The share of one core, in percent, that the turn's processes must burn between two samples to
/// count as working.
const ACTIVE_PERCENT: u64 = 2;

/// A tool line a turn keeps ending the same way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repeat {
    /// The tool line, such as `Bash: cargo test`.
    pub line: String,
    /// How many times in a row.
    pub times: usize,
    /// Whether each of those ends was a failure.
    pub failed: bool,
}

/// The loop `lines`, the turn's tool lines oldest first, end in: the newest line failing twice in
/// a row, or ended three times in a row unless its tool names only a file.
#[must_use]
pub fn repeats(lines: &[ToolLine]) -> Option<Repeat> {
    let newest = lines.last()?;
    let same: Vec<&ToolLine> = lines
        .iter()
        .rev()
        .take_while(|line| line.line == newest.line)
        .collect();
    let failed = same.iter().take_while(|line| line.failed).count();
    if failed >= FAILED {
        return Some(Repeat {
            line: newest.line.clone(),
            times: failed,
            failed: true,
        });
    }
    let tool = tool_name(&newest.line);
    (same.len() >= REPEATED && !FILE_TOOLS.contains(&tool)).then(|| Repeat {
        line: newest.line.clone(),
        times: same.len(),
        failed: false,
    })
}

/// The tool a tool line names, such as `Bash` for `Bash: cargo test`: a fact, where the line after
/// it is text the agent wrote.
#[must_use]
pub fn tool_name(line: &str) -> &str {
    line.split_once(": ").map_or(line, |(tool, _)| tool)
}

/// What code knows of a working seat at a check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    /// How long since the seat's last event.
    pub quiet_ms: u64,
    /// The lead's tool in flight, as its line reads.
    pub tool_in_flight: Option<String>,
    /// Whether the turn's processes burned CPU between the last two samples; `None` before two.
    pub cpu_active: Option<bool>,
}

/// What code decides about a quiet working seat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// A tool runs and its processes burn CPU: a long task.
    LongTask,
    /// Quiet past the check with nothing burning CPU: the layer may read it.
    Open,
    /// Nothing to decide yet.
    Calm,
}

/// Decides about a working seat's `facts` at the check `check_ms` of quiet.
///
/// A tool whose processes burn CPU is a long task, and anything else quiet past the check, once
/// two samples say nothing burns CPU, is open.
#[must_use]
pub fn judge(facts: &Facts, check_ms: Option<u64>) -> Verdict {
    if facts.tool_in_flight.is_some() && facts.cpu_active == Some(true) {
        return Verdict::LongTask;
    }
    match check_ms {
        Some(check) if facts.quiet_ms >= check && facts.cpu_active == Some(false) => Verdict::Open,
        _ => Verdict::Calm,
    }
}

/// The quiet checks from the first, `first_seconds` after the last event: it and twice, four and
/// eight times it, in milliseconds. A first of 0 is no checks.
#[must_use]
pub fn checks(first_seconds: u64) -> Vec<u64> {
    if first_seconds == 0 {
        return Vec::new();
    }
    [1, 2, 4, 8]
        .into_iter()
        .map(|times| first_seconds.saturating_mul(times).saturating_mul(1000))
        .collect()
}

/// How long a seat has been quiet, in words: `12 seconds`, `3 minutes`.
#[must_use]
pub fn quiet_words(milliseconds: u64) -> String {
    let seconds = milliseconds / 1000;
    match seconds {
        0..=1 => "a second".to_string(),
        2..=119 => format!("{seconds} seconds"),
        _ => format!("{} minutes", seconds / 60),
    }
}

/// What a stalled agent seems to be doing, as the layer's choice names it; the two readings that
/// flag nothing, a long task and cannot tell, are not here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stalled {
    /// It waits for input no hook reported.
    WaitingForInput,
    /// It is stuck on a lock or the network.
    Stuck,
    /// It does nothing at all.
    Frozen,
}

impl Stalled {
    /// The kind a reading's option names; `long_task`, `cannot_tell` and anything else name none.
    #[must_use]
    pub fn from_option(option: &str) -> Option<Self> {
        match option {
            "waiting_for_input" => Some(Self::WaitingForInput),
            "stuck" => Some(Self::Stuck),
            "frozen" => Some(Self::Frozen),
            _ => None,
        }
    }

    /// The kind as the question's option and the label name it.
    #[must_use]
    pub const fn option(self) -> &'static str {
        match self {
            Self::WaitingForInput => "waiting_for_input",
            Self::Stuck => "stuck",
            Self::Frozen => "frozen",
        }
    }

    /// The kind in words, for the tooltip and the banner.
    #[must_use]
    pub const fn words(self) -> &'static str {
        match self {
            Self::WaitingForInput => "waiting for input",
            Self::Stuck => "stuck",
            Self::Frozen => "frozen",
        }
    }
}

/// A seat's flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    /// The turn repeats one tool line.
    Looping,
    /// The seat has gone quiet in one of the ways the layer names.
    Stalled(Stalled),
}

impl Flag {
    /// The flag as its label names it.
    #[must_use]
    pub fn value(self) -> String {
        match self {
            Self::Looping => "looping".to_string(),
            Self::Stalled(kind) => format!("stalled:{}", kind.option()),
        }
    }

    /// The flag a label names.
    #[must_use]
    pub fn from_value(value: &str) -> Option<Self> {
        if value == "looping" {
            return Some(Self::Looping);
        }
        value
            .strip_prefix("stalled:")
            .and_then(Stalled::from_option)
            .map(Self::Stalled)
    }
}

/// The labels a flag lands on its seat: the flag, who set it, how sure, and what it rests on.
#[must_use]
pub fn labels(
    flag: Flag,
    source: &str,
    confidence: f64,
    reason: &str,
) -> Vec<(&'static str, String)> {
    vec![
        (FLAG_LABEL, flag.value()),
        (FLAG_SOURCE_LABEL, source.to_string()),
        (FLAG_CONFIDENCE_LABEL, format!("{confidence:.2}")),
        (FLAG_REASON_LABEL, reason.to_string()),
    ]
}

/// Whether a seat's flag shows on its row, from the use's mode: in `suggest` and `act`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FlagShown {
    /// The row shows no flag (`off`, and `shadow`, which only logs).
    #[default]
    Hidden,
    /// The row shows the flag.
    Shown,
}

/// The state word of a working seat's row whose `labels` hold a flag, as `shown`: `looping?` or
/// `stalled?`, a suspicion rather than a verdict.
#[must_use]
pub fn row_word(labels: &BTreeMap<String, String>, shown: FlagShown) -> Option<String> {
    if shown == FlagShown::Hidden {
        return None;
    }
    let flag = labels
        .get(FLAG_LABEL)
        .and_then(|value| Flag::from_value(value))?;
    Some(match flag {
        Flag::Looping => "looping?".to_string(),
        Flag::Stalled(_) => "stalled?".to_string(),
    })
}

/// The row's warning mark's tooltip for a flagged seat's `labels`, as `shown`: what the flag rests
/// on, and a stalled flag's kind and confidence.
#[must_use]
pub fn tooltip(labels: &BTreeMap<String, String>, shown: FlagShown) -> Option<String> {
    if shown == FlagShown::Hidden {
        return None;
    }
    let flag = labels
        .get(FLAG_LABEL)
        .and_then(|value| Flag::from_value(value))?;
    let reason = labels.get(FLAG_REASON_LABEL).map_or("", String::as_str);
    Some(match flag {
        Flag::Looping => format!("Looping? {reason}"),
        Flag::Stalled(kind) => {
            let confidence = labels
                .get(FLAG_CONFIDENCE_LABEL)
                .map_or_else(String::new, |confidence| format!(" ({confidence})"));
            format!("Stalled? {}{confidence}: {reason}", kind.words())
        }
    })
}

/// Whether `ticks` of CPU over `elapsed_ms` is more than 2% of one core.
#[must_use]
pub const fn active(ticks: u64, elapsed_ms: u64) -> bool {
    elapsed_ms > 0
        && ticks.saturating_mul(100_000)
            > ACTIVE_PERCENT
                .saturating_mul(TICKS_PER_SECOND)
                .saturating_mul(elapsed_ms)
}

/// `epoch_ms`, a moment in the fleet's epoch milliseconds, in the clock ticks since boot
/// `/proc`'s `starttime` counts, with the machine booted at `boot_secs` since the epoch.
#[must_use]
pub const fn ticks_since_boot(epoch_ms: u64, boot_secs: u64) -> u64 {
    epoch_ms.saturating_sub(boot_secs.saturating_mul(1000)) / (1000 / TICKS_PER_SECOND)
}

/// When the machine booted, in seconds since the epoch: `stat`'s `btime` under `proc_root`.
#[must_use]
pub fn boot_time_in(proc_root: &Path) -> Option<u64> {
    let stat = std::fs::read_to_string(proc_root.join("stat")).ok()?;
    stat.lines()
        .find_map(|line| line.strip_prefix("btime "))
        .and_then(|seconds| seconds.trim().parse().ok())
}

/// The CPU, in clock ticks, that the turn's tools under `pid` have burned.
///
/// The turn's tools are `pid`'s descendants started at or after `since_ticks`. `pid` itself,
/// Claude Code with its spinner, and the children it had before, its MCP and language servers,
/// are left out. `None` when `pid` is gone; a descendant that goes while it is read is skipped.
#[must_use]
pub fn tree_cpu_in(proc_root: &Path, pid: i32, since_ticks: u64) -> Option<u64> {
    // The root must be there; its descendants may go while they are read.
    let _root = Stat::from_file(proc_root.join(pid.to_string()).join("stat")).ok()?;
    let mut seen = HashSet::from([pid]);
    let mut waiting = children_in(proc_root, pid);
    let mut ticks = 0_u64;
    while let Some(child) = waiting.pop() {
        if !seen.insert(child) {
            continue;
        }
        waiting.extend(children_in(proc_root, child));
        if let Ok(stat) = Stat::from_file(proc_root.join(child.to_string()).join("stat"))
            && stat.starttime >= since_ticks
        {
            ticks = ticks.saturating_add(stat.utime.saturating_add(stat.stime));
        }
    }
    Some(ticks)
}

/// The children of `pid` under `proc_root`: each of its threads' `children`.
fn children_in(proc_root: &Path, pid: i32) -> Vec<i32> {
    let Ok(threads) = std::fs::read_dir(proc_root.join(pid.to_string()).join("task")) else {
        return Vec::new();
    };
    threads
        .filter_map(Result::ok)
        .filter_map(|thread| std::fs::read_to_string(thread.path().join("children")).ok())
        .flat_map(|children| {
            children
                .split_whitespace()
                .filter_map(|child| child.parse().ok())
                .collect::<Vec<i32>>()
        })
        .collect()
}
