//! A running command's printed error (#572): the shapes build tools and servers print a failure
//! and a recovery in, and the episode a running block's lines move through.
//!
//! A failure line opens a suspect; if the block still runs [`GRACE`] later the episode fails,
//! which the watcher tells once. A recovery line closes it. A line with an error's word that no
//! shape settles is open, for the System One layer to read.

use std::sync::LazyLock;
use std::time::{Duration, Instant};

use regex::RegexSet;

/// How long a failure line waits for its block to end before the episode fails: a command that
/// ends inside it is #551's to tell, by its exit.
pub const GRACE: Duration = Duration::from_secs(5);

/// How compilers, test runners, dev servers and runtimes print a failure.
const FAILURES: &[&str] = &[
    r"^\s*(error|ERROR|Error)(\[\w+\])?\s*:",
    r"error TS\d+:",
    r"^Traceback \(most recent call last\)",
    r"panicked at",
    r"\b(Unhandled|Uncaught)\b",
    r"\b(EADDRINUSE|EACCES|ECONNREFUSED|ENOENT|EPERM)\b",
    r"Failed to compile|Build failed|Compilation failed|could not compile|failed with exit code",
    r"^\s*(✖|✘|×|⨯|FAIL)\s",
    r"\[vite\] Internal server error",
    r"Segmentation fault",
    r"^fatal:",
];

/// How the same tools print that they build or serve again.
const RECOVERIES: &[&str] = &[
    r"[Cc]ompiled successfully",
    r"✓ (built|Compiled)",
    r"ready in \d+",
    r"^\s*Finished \S",
    r"webpack compiled",
    r"No issues found|Found 0 errors",
    r"Listening on|Server running|Local:\s+https?://",
    r"(hmr|HMR) update|page reload",
];

/// An error's words, which leave a line no shape matched open.
const OPEN: &[&str] = &[r"(?i)\b(error|fail|failed|exception|fatal)\b"];

static FAILURE_SET: LazyLock<Option<RegexSet>> = LazyLock::new(|| compiled(FAILURES));
static RECOVERY_SET: LazyLock<Option<RegexSet>> = LazyLock::new(|| compiled(RECOVERIES));
static OPEN_SET: LazyLock<Option<RegexSet>> = LazyLock::new(|| compiled(OPEN));

fn compiled(patterns: &[&str]) -> Option<RegexSet> {
    RegexSet::new(patterns)
        .inspect_err(|error| log::error!("running errors: a shape does not compile: {error}"))
        .ok()
}

fn holds(set: Option<&RegexSet>, line: &str) -> bool {
    set.is_some_and(|set| set.is_match(line))
}

/// What a line of a running block's output says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// A failure as a tool prints one.
    Failure,
    /// A build or a server working again.
    Recovery,
    /// An error's word that no shape settles.
    Open,
}

/// What `line` says: a failure first, then a recovery, then an open line; `None` for an ordinary
/// line.
#[must_use]
pub fn scan(line: &str) -> Option<Shape> {
    if holds(FAILURE_SET.as_ref(), line) {
        Some(Shape::Failure)
    } else if holds(RECOVERY_SET.as_ref(), line) {
        Some(Shape::Recovery)
    } else if holds(OPEN_SET.as_ref(), line) {
        Some(Shape::Open)
    } else {
        None
    }
}

/// Where a running block stands on failure.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Episode {
    /// No failure since the block started or last recovered.
    #[default]
    Quiet,
    /// A failure line, waiting out [`GRACE`].
    Suspect {
        /// When the line came.
        since: Instant,
        /// The first failure line.
        line: String,
        /// Whether a reading in `suggest`, not a shape, found it.
        questioned: bool,
    },
    /// The block kept running past the grace.
    Failed {
        /// The first failure line.
        line: String,
        /// Whether a reading in `suggest`, not a shape, found it.
        questioned: bool,
    },
}

/// What a step of an episode tells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// The block kept running past the grace: `line` is told, unless `questioned`.
    Flag {
        /// The first failure line.
        line: String,
        /// Whether a reading in `suggest` found it.
        questioned: bool,
    },
    /// A recovery line closed a failed episode; `told` when its flag was told.
    Recovered {
        /// The recovery line.
        line: String,
        /// Whether the flag was told.
        told: bool,
    },
    /// The block ended while failed: the mark goes, with nothing to tell.
    Cleared,
}

impl Episode {
    /// A failure line at `now`. Only a quiet episode takes it: a new episode needs a recovery or
    /// a new block between.
    pub fn failure(&mut self, line: &str, questioned: bool, now: Instant) {
        if *self == Self::Quiet {
            *self = Self::Suspect {
                since: now,
                line: line.to_string(),
                questioned,
            };
        }
    }

    /// A recovery line: a suspect closes with nothing to tell, a failed episode as recovered.
    #[must_use]
    pub fn recovery(&mut self, line: &str) -> Option<Change> {
        match std::mem::take(self) {
            Self::Failed { questioned, .. } => Some(Change::Recovered {
                line: line.to_string(),
                told: !questioned,
            }),
            Self::Quiet | Self::Suspect { .. } => None,
        }
    }

    /// When a suspect's grace ends, for the watcher's timer.
    #[must_use]
    pub fn due(&self) -> Option<Instant> {
        match self {
            Self::Suspect { since, .. } => since.checked_add(GRACE),
            Self::Quiet | Self::Failed { .. } => None,
        }
    }

    /// The episode at `now`: a suspect past its grace fails.
    #[must_use]
    pub fn tick(&mut self, now: Instant) -> Option<Change> {
        let Self::Suspect {
            since,
            line,
            questioned,
        } = self
        else {
            return None;
        };
        if now.saturating_duration_since(*since) < GRACE {
            return None;
        }
        let (line, questioned) = (std::mem::take(line), *questioned);
        *self = Self::Failed {
            line: line.clone(),
            questioned,
        };
        Some(Change::Flag { line, questioned })
    }

    /// The block ended: a suspect goes with nothing to tell (the exit tells it), a failed episode
    /// clears its mark.
    #[must_use]
    pub fn ended(&mut self) -> Option<Change> {
        matches!(std::mem::take(self), Self::Failed { .. }).then_some(Change::Cleared)
    }

    /// A failed episode's line, and whether a reading in `suggest` found it.
    #[must_use]
    pub fn mark(&self) -> Option<(&str, bool)> {
        match self {
            Self::Failed { line, questioned } => Some((line, *questioned)),
            Self::Quiet | Self::Suspect { .. } => None,
        }
    }
}
