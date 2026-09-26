//! The flight recorder (#499, pillar C): each page's last minute, kept in memory while a tab
//! draws the page, and saved as a recording when the user asks.
//!
//! The minute holds what reached the page and what came of it: presses, wheels and keys (a key
//! that types counts as a character, and the character itself is never kept), the agent's
//! actions, console entries, requests with their URLs' secrets hidden, navigations, the page's
//! accessibility snapshot after each load, and a frame every half second, as the JPEG Chromium
//! sent. A recording is a folder: `timeline.json` and `frames/NNNN.jpg`.

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::Engine as _;
use serde::{Serialize, Serializer};
use serde_json::{Value, json};

/// How much the recorder keeps.
pub const WINDOW: Duration = Duration::from_secs(60);

/// The least time between two frames it keeps: at most two a second.
pub const FRAME_GAP: Duration = Duration::from_millis(500);

/// The most frame bytes it keeps per page, the oldest going first.
pub const FRAME_BYTES: usize = 16 * 1024 * 1024;

/// How close two wheel turns must be to count as one scroll.
const SCROLL_GAP: Duration = Duration::from_millis(500);

/// A recording's timeline, in its folder.
const TIMELINE: &str = "timeline.json";

/// A recording's frames, in its folder.
const FRAMES: &str = "frames";

/// Something that happened in the page, as the recorder keeps it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Entry {
    /// A press of a mouse button.
    Click {
        /// Where, across the viewport, in CSS pixels.
        x: f64,
        /// Where, down the viewport, in CSS pixels.
        y: f64,
        /// `left`, `middle`, `right`, `back` or `forward`.
        button: String,
        /// One for a click, two for a double click.
        count: usize,
    },
    /// The wheel.
    Scroll {
        /// Right, in CSS pixels; negative is left.
        dx: f64,
        /// Down, in CSS pixels; negative is up.
        dy: f64,
    },
    /// A key that types nothing.
    Key {
        /// Its name, or a shortcut's: `Enter`, `Ctrl+A`.
        key: String,
    },
    /// Characters typed or inserted.
    Typed {
        /// How many; the characters themselves are never kept.
        characters: usize,
    },
    /// The main frame showed a document.
    Navigation {
        /// Its URL, secret-looking values hidden.
        url: String,
    },
    /// A console message, an uncaught error or a browser log entry.
    Console {
        /// `log`, `warning`, `error`, `exception` and the like.
        level: String,
        /// The message.
        text: String,
    },
    /// A request the page made.
    Request {
        /// The browser's id for it, which its response and failure name.
        #[serde(skip)]
        id: String,
        /// Its method.
        method: String,
        /// Its URL, secret-looking values hidden.
        url: String,
        /// The response's status, once one came.
        status: Option<u64>,
        /// Why it failed, when it did.
        failure: Option<String>,
    },
    /// The page's accessibility snapshot after a load.
    Snapshot {
        /// As `snapshot::render` writes it, which writes no field's value.
        text: String,
    },
    /// What an agent did in the page.
    Agent {
        /// As the Agent chip said it.
        did: String,
    },
    /// A frame of the page.
    Frame {
        /// Its number, from 1, and its file's: `frames/0001.jpg`.
        index: usize,
    },
}

/// An entry and when it came, in milliseconds from the recording's start.
#[derive(Debug, Clone, PartialEq)]
pub struct TimedEntry {
    /// When it came.
    pub at_ms: u64,
    /// What it was.
    pub entry: Entry,
}

impl Serialize for TimedEntry {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = serde_json::to_value(&self.entry).map_err(serde::ser::Error::custom)?;
        if let Some(object) = value.as_object_mut() {
            object.insert("at_ms".to_string(), json!(self.at_ms));
        }
        value.serialize(serializer)
    }
}

/// A page's minute as it is saved.
#[derive(Debug, Clone, Serialize)]
pub struct Recording {
    /// Its name, and its folder's.
    pub id: String,
    /// The page it was recorded from.
    pub tab: String,
    /// The page's URL when it was saved, its secrets hidden.
    pub url: String,
    /// The page's title then.
    pub title: String,
    /// When it was saved, in seconds since the Unix epoch.
    pub recorded_at: u64,
    /// How long it spans.
    pub seconds: f64,
    /// How many frames it holds.
    pub frames: usize,
    /// What happened, oldest first.
    pub entries: Vec<TimedEntry>,
    /// The page's accessibility snapshot when it was saved.
    pub snapshot: String,
}

/// What went wrong with a recording's files.
#[derive(Debug, thiserror::Error)]
pub enum RecordingError {
    /// A file could not be read or written.
    #[error("{0}: {1}")]
    Io(PathBuf, std::io::Error),
    /// The name is not one Marley gives a recording.
    #[error("no recording is named {0:?}")]
    Name(String),
    /// A timeline does not parse, or could not be written.
    #[error("a recording's timeline: {0}")]
    Json(#[from] serde_json::Error),
    /// The recording has no such frame.
    #[error("the recording has no frame {0}")]
    NoFrame(usize),
}

/// One kept thing and when it came.
#[derive(Debug, Clone)]
struct Timed<T> {
    at: Instant,
    value: T,
}

/// A page's last minute.
#[derive(Debug, Clone, Default)]
pub struct Recorder {
    entries: VecDeque<Timed<Entry>>,
    /// The frames, as base64 JPEGs.
    frames: VecDeque<Timed<Arc<str>>>,
    frame_bytes: usize,
}

impl Recorder {
    /// Keeps `entry`, which came `now`: typing adds to the typing just before it, and a wheel
    /// turn to the scroll just before it.
    pub fn push(&mut self, entry: Entry, now: Instant) {
        if let Some(last) = self.entries.back_mut() {
            match (&mut last.value, &entry) {
                (Entry::Typed { characters }, Entry::Typed { characters: more }) => {
                    *characters += more;
                    last.at = now;
                    return;
                }
                (
                    Entry::Scroll { dx, dy },
                    Entry::Scroll {
                        dx: more_x,
                        dy: more_y,
                    },
                ) if now.duration_since(last.at) < SCROLL_GAP => {
                    *dx += more_x;
                    *dy += more_y;
                    last.at = now;
                    return;
                }
                _ => {}
            }
        }
        self.entries.push_back(Timed {
            at: now,
            value: entry,
        });
        self.trim(now);
    }

    /// Adds the response's status, or the failure, to the request `id`.
    pub fn request_ended(&mut self, id: &str, ended_with: Result<u64, String>) {
        for timed in self.entries.iter_mut().rev() {
            if let Entry::Request {
                id: known,
                status,
                failure,
                ..
            } = &mut timed.value
                && known == id
            {
                match ended_with {
                    Ok(code) => *status = Some(code),
                    Err(reason) => *failure = Some(reason),
                }
                return;
            }
        }
    }

    /// Whether a frame that came `now` is one to keep: half a second after the last.
    #[must_use]
    pub fn wants_frame(&self, now: Instant) -> bool {
        self.frames
            .back()
            .is_none_or(|last| now.duration_since(last.at) >= FRAME_GAP)
    }

    /// Keeps a frame, a base64 JPEG, when it is one to keep; the oldest go past the byte cap.
    pub fn push_frame(&mut self, jpeg: Arc<str>, now: Instant) {
        if !self.wants_frame(now) {
            return;
        }
        self.frame_bytes += jpeg.len();
        self.frames.push_back(Timed {
            at: now,
            value: jpeg,
        });
        while self.frame_bytes > FRAME_BYTES
            && let Some(oldest) = self.frames.pop_front()
        {
            self.frame_bytes -= oldest.value.len();
        }
        self.trim(now);
    }

    /// Drops what is older than the minute.
    fn trim(&mut self, now: Instant) {
        let Some(start) = now.checked_sub(WINDOW) else {
            return;
        };
        while self.entries.front().is_some_and(|timed| timed.at < start) {
            self.entries.pop_front();
        }
        while let Some(front) = self.frames.front()
            && front.at < start
        {
            self.frame_bytes -= front.value.len();
            self.frames.pop_front();
        }
    }

    /// The minute before `now`: its entries and frames in time order, each frame an entry that
    /// names it, times from the minute's start; the frames' JPEGs, in the order their entries
    /// name them; and how long it spans, in seconds.
    #[must_use]
    pub fn take(&self, now: Instant) -> (Vec<TimedEntry>, Vec<Arc<str>>, f64) {
        let start_bound = now.checked_sub(WINDOW);
        let within = |at: Instant| start_bound.is_none_or(|start| at >= start);
        let entries: Vec<&Timed<Entry>> = self
            .entries
            .iter()
            .filter(|timed| within(timed.at))
            .collect();
        let frames: Vec<&Timed<Arc<str>>> = self
            .frames
            .iter()
            .filter(|timed| within(timed.at))
            .collect();
        let start = entries
            .first()
            .map(|timed| timed.at)
            .into_iter()
            .chain(frames.first().map(|timed| timed.at))
            .min()
            .unwrap_or(now);
        let since =
            |at: Instant| u64::try_from(at.duration_since(start).as_millis()).unwrap_or(u64::MAX);
        let mut timeline = Vec::with_capacity(entries.len() + frames.len());
        let mut jpegs = Vec::with_capacity(frames.len());
        let (mut entries, mut frames) = (
            entries.into_iter().peekable(),
            frames.into_iter().peekable(),
        );
        loop {
            let frame_first = match (entries.peek(), frames.peek()) {
                (Some(entry), Some(frame)) => frame.at < entry.at,
                (None, Some(_)) => true,
                (Some(_), None) => false,
                (None, None) => break,
            };
            if frame_first {
                if let Some(frame) = frames.next() {
                    jpegs.push(Arc::clone(&frame.value));
                    timeline.push(TimedEntry {
                        at_ms: since(frame.at),
                        entry: Entry::Frame { index: jpegs.len() },
                    });
                }
            } else if let Some(entry) = entries.next() {
                timeline.push(TimedEntry {
                    at_ms: since(entry.at),
                    entry: entry.value.clone(),
                });
            }
        }
        (timeline, jpegs, now.duration_since(start).as_secs_f64())
    }
}

/// Writes `recording` and its `frames` into a folder of its own in `dir`, a suffix on its id
/// when another recording has that name; the folder.
///
/// # Errors
///
/// When a folder or file cannot be written, or a frame is not base64.
pub fn save_in(
    dir: &Path,
    recording: &mut Recording,
    frames: &[Arc<str>],
) -> Result<PathBuf, RecordingError> {
    let base = recording.id.clone();
    let mut folder = dir.join(&base);
    let mut suffix = 2;
    while folder.exists() {
        recording.id = format!("{base}-{suffix}");
        folder = dir.join(&recording.id);
        suffix += 1;
    }
    let frames_dir = folder.join(FRAMES);
    fs::create_dir_all(&frames_dir)
        .map_err(|error| RecordingError::Io(frames_dir.clone(), error))?;
    for (index, frame) in frames.iter().enumerate() {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(frame.as_bytes())
            .map_err(|error| {
                RecordingError::Io(
                    frames_dir.clone(),
                    std::io::Error::new(std::io::ErrorKind::InvalidData, error),
                )
            })?;
        let path = frames_dir.join(frame_name(index + 1));
        fs::write(&path, bytes).map_err(|error| RecordingError::Io(path.clone(), error))?;
    }
    let timeline = serde_json::to_vec_pretty(recording)?;
    let path = folder.join(TIMELINE);
    fs::write(&path, timeline).map_err(|error| RecordingError::Io(path.clone(), error))?;
    Ok(folder)
}

/// The recordings in `dir`, oldest first, each without its entries and snapshot.
///
/// Each gives its id, tab, URL, title, time, length, and how many frames and entries it holds.
/// A folder with no timeline Marley can read is left out.
///
/// # Errors
///
/// When `dir` exists and cannot be read.
pub fn list_in(dir: &Path) -> Result<Vec<Value>, RecordingError> {
    let listing = match fs::read_dir(dir) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(RecordingError::Io(dir.to_path_buf(), error)),
    };
    let mut ids: Vec<String> = listing
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|id| is_recording_id(id))
        .collect();
    ids.sort();
    Ok(ids
        .iter()
        .filter_map(|id| read_in(dir, id).ok())
        .map(|timeline| {
            let entries = timeline
                .get("entries")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            json!({
                "id": timeline.get("id"),
                "tab": timeline.get("tab"),
                "url": timeline.get("url"),
                "title": timeline.get("title"),
                "recorded_at": timeline.get("recorded_at"),
                "seconds": timeline.get("seconds"),
                "frames": timeline.get("frames"),
                "entries": entries,
            })
        })
        .collect())
}

/// The recording `id`'s timeline in `dir`.
///
/// # Errors
///
/// When the name is not a recording's, or its timeline cannot be read or parsed.
pub fn read_in(dir: &Path, id: &str) -> Result<Value, RecordingError> {
    if !is_recording_id(id) {
        return Err(RecordingError::Name(id.to_string()));
    }
    let path = dir.join(id).join(TIMELINE);
    let text = fs::read(&path).map_err(|error| RecordingError::Io(path.clone(), error))?;
    Ok(serde_json::from_slice(&text)?)
}

/// The frame `index`, from 1, of the recording `id` in `dir`, as JPEG bytes.
///
/// # Errors
///
/// When the name is not a recording's, or the recording has no such frame.
pub fn frame_in(dir: &Path, id: &str, index: usize) -> Result<Vec<u8>, RecordingError> {
    if !is_recording_id(id) {
        return Err(RecordingError::Name(id.to_string()));
    }
    let path = dir.join(id).join(FRAMES).join(frame_name(index));
    fs::read(&path).map_err(|_| RecordingError::NoFrame(index))
}

/// Whether `id` is a name Marley gives a recording: letters, digits and dashes, which no path
/// climbs out of.
fn is_recording_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

fn frame_name(index: usize) -> String {
    format!("{index:04}.jpg")
}
