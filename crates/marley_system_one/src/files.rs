//! The layer's files, under the folder the host names.
//!
//! Every call is one JSON line in the day's `calls-<day>.jsonl`, failures and refusals included, so
//! a provider that could not be reached and a model that found nothing never look alike. An
//! outcome a use learns later is a line of its own naming the call; no line is rewritten.
//! `replay.jsonl` holds recorded answers for the `replay` provider.

use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::request::{Answers, ParseError, parse};
use crate::state::State;

/// The file of recorded answers under the layer's folder.
pub const REPLAY_FILE: &str = "replay.jsonl";

/// One call as the log keeps it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallRow {
    /// The call's id, which a later outcome names.
    pub id: String,
    /// When the call was made, in RFC 3339 and local time.
    pub time: String,
    /// The use that asked.
    #[serde(rename = "use")]
    pub use_name: String,
    /// The question set and its version.
    pub set: String,
    /// The model asked.
    pub model: String,
    /// Who answered: `typesafe`, `compatible`, `rules` or `replay`.
    pub provider: String,
    /// The project's name.
    pub project: Option<String>,
    /// The use's mode: `shadow`, `suggest` or `act`.
    pub mode: String,
    /// The use's own verdict, as answers.
    pub verdict: Option<Value>,
    /// The masked state as sent, when one was built.
    pub state: Option<String>,
    /// The state's hash.
    pub state_hash: Option<String>,
    /// The keys of the questions asked.
    pub questions: Vec<String>,
    /// The answers as they came.
    pub answers: Option<Value>,
    /// The reading as one line: `command failed: yes (0.92)`, `Refused: project not listed`.
    pub reading: String,
    /// Whether the call was refused or got no answer.
    pub failed: bool,
    /// The thresholds the answers were read against.
    pub thresholds: Option<Value>,
    /// How long the call took, in milliseconds.
    pub latency_ms: Option<u64>,
    /// The input tokens it spent.
    pub input_tokens: Option<u64>,
    /// What it cost, in billionths of a cent.
    pub cost: Option<u64>,
    /// What went wrong, cut to 300 characters.
    pub error: Option<String>,
}

/// What a use learned of a call later.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeRow {
    /// The call it is about.
    pub call: String,
    /// When it was learned, in RFC 3339 and local time.
    pub time: String,
    /// What happened.
    pub outcome: String,
}

/// A line of the log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "row", rename_all = "snake_case")]
pub enum Row {
    /// A call.
    Call(Box<CallRow>),
    /// A later outcome of a call.
    Outcome(OutcomeRow),
}

/// The thresholds [`crate::reading`] reads answers against, as a log row names them.
#[must_use]
pub fn thresholds() -> Value {
    json!({
        "confidence_floor": crate::reading::CONFIDENCE_FLOOR,
        "noul_no_signal": [0.35, 0.65],
    })
}

/// `day`'s file under `dir`.
fn day_file(dir: &Path, day: &str) -> PathBuf {
    dir.join(format!("calls-{day}.jsonl"))
}

/// Appends `row` to `day`'s file under `dir`, making the folder, and the file readable by its
/// owner alone, when they are missing.
///
/// # Errors
///
/// When the folder or the file cannot be made or written.
pub fn append_in(dir: &Path, day: &str, row: &Row) -> io::Result<()> {
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

/// `day`'s calls under `dir`, oldest first. A missing file is no calls, and a line that does not
/// read is skipped.
///
/// # Errors
///
/// When the file is there and cannot be read.
pub fn read_day_in(dir: &Path, day: &str) -> io::Result<Vec<CallRow>> {
    let text = match fs::read_to_string(day_file(dir, day)) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    Ok(text
        .lines()
        .filter_map(|line| match serde_json::from_str::<Row>(line) {
            Ok(Row::Call(call)) => Some(*call),
            Ok(Row::Outcome(_)) | Err(_) => None,
        })
        .collect())
}

/// A recorded answer for the `replay` provider.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ReplayRow {
    /// The set it answers, such as `check/1`.
    pub set: String,
    /// Text the masked state must hold for this row to answer it.
    #[serde(default, rename = "match")]
    pub holds: Option<String>,
    /// The hash of the one state this row answers.
    #[serde(default)]
    pub state_hash: Option<String>,
    /// The answers, as a provider's `answers` object.
    pub answers: Value,
    /// Whether the row answers again; otherwise it answers once.
    #[serde(default)]
    pub repeat: bool,
}

/// The recorded answers, and which of them were given.
#[derive(Debug, Default)]
pub struct Replay {
    rows: Vec<(ReplayRow, bool)>,
}

impl Replay {
    /// The rows of [`REPLAY_FILE`] under `dir`. A missing file is none, and a line that does not
    /// read is skipped.
    ///
    /// # Errors
    ///
    /// When the file is there and cannot be read.
    pub fn load_in(dir: &Path) -> io::Result<Self> {
        let text = match fs::read_to_string(dir.join(REPLAY_FILE)) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        Ok(Self {
            rows: text
                .lines()
                .filter_map(|line| serde_json::from_str::<ReplayRow>(line).ok())
                .map(|row| (row, false))
                .collect(),
        })
    }

    /// The answers of the first row that fits `set` and `state`, or `None` when no row does. A row
    /// fits when its set is `set`, its `match` is in the state and its `state_hash` is the state's,
    /// each when given; it answers once unless it repeats.
    ///
    /// # Errors
    ///
    /// When the fitting row's answers do not read as answers.
    pub fn answer(&mut self, set: &str, state: &State) -> Result<Option<Answers>, ParseError> {
        let Some((row, given)) = self.rows.iter_mut().find(|(row, given)| {
            !*given
                && row.set == set
                && row
                    .holds
                    .as_deref()
                    .is_none_or(|text| state.text.contains(text))
                && row
                    .state_hash
                    .as_deref()
                    .is_none_or(|hash| hash == state.hash)
        }) else {
            return Ok(None);
        };
        if !row.repeat {
            *given = true;
        }
        parse(&json!({ "model": "replay", "answers": row.answers }).to_string()).map(Some)
    }
}
