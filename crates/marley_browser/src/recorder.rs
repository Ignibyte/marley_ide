//! The flight recorder (#499, pillar C): each page's last minute, kept in memory while a tab
//! draws the page, and saved as a recording when the user asks.
//!
//! The minute holds what reached the page and what came of it: presses, wheels and keys (a key
//! that types counts as a character, the character itself not kept there), the agent's
//! actions, console entries, requests with their URLs' secrets hidden, navigations, the page's
//! accessibility snapshot after each load, and a frame every half second, as the JPEG Chromium
//! sent. A recording is a folder: `timeline.json` and `frames/NNNN.jpg`.
//!
//! Since #506 a listener in the page reports each click, fill and press with the target's
//! locators as they were at the event, which a recording turns into a Playwright test
//! (`crate::playwright`). A fill keeps what an ordinary field holds; a password or other secret
//! field keeps nothing but the fact that it was filled.

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::Engine as _;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Value, json};

use crate::cdp::CdpError;
use crate::observe::redact_url;
use crate::page::Page;
use crate::pick::{css_path, generated_name, interactive, within};

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

/// The isolated world the action listener runs in (#506), apart from the page's own scripts.
pub const WORLD: &str = "marley-record";

/// The binding the action listener reports through, as `Runtime.bindingCalled`.
pub const BINDING: &str = "marleyRecord";

/// The most of a fill's text the listener hands back. It only bounds what crosses the socket: a
/// tool redacts the text whole before it cuts it for an agent.
const FILL_TRIP: usize = 4_096;

/// The most of a locator's value, or a field's name, kept.
const LOCATOR_TRIP: usize = 500;

/// The action listener (#506), which a world runs once however many times it is evaluated there,
/// in the top frame only.
///
/// At the window, in the capture phase, before the page's own handlers change
/// anything, it takes trusted events: a primary-button `pointerdown` (walked up to the
/// interactive ancestor, as a pick is), an `input` (a fill, with the field's text unless the
/// field is a secret one), and Enter, Tab or Escape pressed (on the focused element). It reports
/// each with the target's locators, most durable first, each marked when it finds the element
/// alone: a test id, the role and name, a form field's label, the placeholder, the text, a CSS
/// path. No locator rests on a generated id or a class.
pub const LISTENER: &str = concat!(
    "(() => {\n",
    r"  if (globalThis.__marleyRecordListener || window !== window.top) return;
  globalThis.__marleyRecordListener = true;
",
    interactive!(),
    generated_name!(),
    css_path!(),
    r#"  const cap = (value, limit) => {
    const whole = value.toWellFormed ? value.toWellFormed() : value;
    if (whole.length <= limit) return whole;
    const code = whole.charCodeAt(limit - 1);
    return whole.slice(0, code >= 0xd800 && code <= 0xdbff ? limit - 1 : limit) + ' (truncated)';
  };
  const squeeze = (value) => String(value || '').trim().replace(/\s+/g, ' ');
  const unique = (selector) => { try { return document.querySelectorAll(selector).length === 1; } catch (error) { return false; } };
  const shown = (element) => !element.checkVisibility || element.checkVisibility();
  const typeOf = (element) => (element.getAttribute('type') || 'text').toLowerCase();
  const textTypes = ['text', 'email', 'tel', 'url'];
  const roleOf = (element) => {
    const explicit = (element.getAttribute('role') || '').trim().split(/\s+/)[0];
    if (explicit) return explicit;
    const tag = element.localName;
    if (tag === 'button') return 'button';
    if ((tag === 'a' || tag === 'area') && element.hasAttribute('href')) return 'link';
    if (tag === 'textarea') return 'textbox';
    if (tag === 'select') return element.multiple || element.size > 1 ? 'listbox' : 'combobox';
    if (/^h[1-6]$/.test(tag)) return 'heading';
    if (tag === 'img' && element.getAttribute('alt')) return 'img';
    if (tag !== 'input') return null;
    const type = typeOf(element);
    if (['button', 'submit', 'reset', 'image'].includes(type)) return 'button';
    if (type === 'checkbox') return 'checkbox';
    if (type === 'radio') return 'radio';
    if (type === 'number') return 'spinbutton';
    if (type === 'range') return 'slider';
    if (type === 'search') return element.hasAttribute('list') ? 'combobox' : 'searchbox';
    if (textTypes.includes(type) || !['password', 'hidden', 'file', 'color', 'date',
      'datetime-local', 'month', 'time', 'week'].includes(type)) {
      return element.hasAttribute('list') ? 'combobox' : 'textbox';
    }
    return null;
  };
  const labelOf = (element) => {
    const labels = element.labels ? [...element.labels] : [];
    return squeeze(labels.map((label) => label.innerText).join(' '));
  };
  const nameOf = (element) => {
    const labelledBy = (element.getAttribute('aria-labelledby') || '').trim();
    if (labelledBy) {
      const named = squeeze(labelledBy.split(/\s+/).map((id) => {
        const target = document.getElementById(id);
        return target ? target.innerText || target.textContent : '';
      }).join(' '));
      if (named) return named;
    }
    const label = squeeze(element.getAttribute('aria-label'));
    if (label) return label;
    const tag = element.localName;
    const field = tag === 'input' || tag === 'select' || tag === 'textarea';
    if (field && ['button', 'submit', 'reset'].includes(typeOf(element)) && tag === 'input') {
      return squeeze(element.value);
    }
    if (field && labelOf(element)) return labelOf(element);
    if (tag === 'img') return squeeze(element.getAttribute('alt'));
    if (!field) {
      const text = squeeze(element.innerText);
      if (text) return text;
    }
    return squeeze(element.getAttribute('title')) || squeeze(element.getAttribute('placeholder'));
  };
  const rolesFound = (role, name) => {
    let count = 0;
    for (const element of document.querySelectorAll('*')) {
      if (roleOf(element) === role && shown(element) && nameOf(element) === name) count += 1;
      if (count > 1) break;
    }
    return count;
  };
  const labelsFound = (label) => {
    let count = 0;
    for (const element of document.querySelectorAll('input, select, textarea')) {
      if (labelOf(element) === label && shown(element)) count += 1;
    }
    return count;
  };
  // The innermost elements that read `text`, found from the text nodes that hold its first word.
  const textsFound = (text) => {
    const first = text.split(' ')[0];
    const found = new Set();
    const walker = document.createTreeWalker(document.body || document.documentElement, NodeFilter.SHOW_TEXT);
    for (let node = walker.nextNode(), seen = 0; node && seen < 20000 && found.size < 2; node = walker.nextNode(), seen += 1) {
      if (!node.data.includes(first)) continue;
      for (let element = node.parentElement; element; element = element.parentElement) {
        const own = squeeze(element.innerText);
        if (own === text) { found.add(element); break; }
        if (own.length > text.length) break;
      }
    }
    return found.size;
  };
  const locate = (element) => {
    const locators = [];
    for (const attribute of ['data-testid', 'data-test', 'data-cy', 'data-qa']) {
      const value = element.getAttribute(attribute);
      if (value) {
        locators.push({ kind: 'test id', attribute, value,
          unique: unique('[' + attribute + '=' + JSON.stringify(value) + ']') });
        break;
      }
    }
    const role = roleOf(element);
    const name = role ? nameOf(element) : '';
    if (role && name) locators.push({ kind: 'role', role, value: cap(name, 200), unique: rolesFound(role, name) === 1 });
    const label = labelOf(element);
    if (label) locators.push({ kind: 'label', value: cap(label, 200), unique: labelsFound(label) === 1 });
    const placeholder = squeeze(element.getAttribute('placeholder'));
    if (placeholder) locators.push({ kind: 'placeholder', value: cap(placeholder, 200),
      unique: unique('[placeholder=' + JSON.stringify(element.getAttribute('placeholder')) + ']') });
    const text = element.localName === 'input' || element.localName === 'textarea' ? '' : squeeze(element.innerText);
    if (text && text.length <= 80) locators.push({ kind: 'text', value: text, unique: textsFound(text) === 1 });
    const css = cssPath(element);
    locators.push({ kind: 'css', value: css, unique: unique(css) });
    return locators;
  };
  // A field Playwright fills: a text-like input, a text area, an editable element. A checkbox,
  // a radio or a file input is recorded by its click.
  const fillable = (element) => element.localName === 'textarea' || element.isContentEditable
    || element.localName === 'input' && !['checkbox', 'radio', 'file', 'range', 'color', 'button',
      'submit', 'reset', 'image'].includes(typeOf(element));
  // A field whose value is a secret: a password, a hidden field, or one the page marks as a
  // password, a one-time code or a card's.
  const secretField = (element) => {
    const autocomplete = (element.getAttribute('autocomplete') || '').toLowerCase();
    return element.localName === 'input' && ['password', 'hidden'].includes(typeOf(element))
      || /(^|\s)(current-password|new-password|one-time-code|cc-[a-z-]+)(\s|$)/.test(autocomplete);
  };
  const report = (action, element, extra) => {
    try {
      marleyRecord(JSON.stringify(Object.assign({ action, locators: locate(element), url: location.href,
        field: element.getAttribute('name') || (element.id && !generatedName(element.id) ? element.id : null) }, extra)));
    } catch (error) {}
  };
  addEventListener('pointerdown', (event) => {
    if (!event.isTrusted || event.button !== 0 || !(event.target instanceof Element)) return;
    let element = event.target;
    for (let step = element; step; step = up(step)) {
      if (interactive(step)) { element = step; break; }
    }
    report('click', element, {});
  }, true);
  addEventListener('input', (event) => {
    const field = event.target;
    if (!event.isTrusted || !(field instanceof Element) || !fillable(field)) return;
    const secret = secretField(field);
    const text = secret ? null : cap(String(field.value !== undefined ? field.value : field.textContent || ''), 4096);
    report('fill', field, { text, secret });
  }, true);
  addEventListener('keydown', (event) => {
    if (!event.isTrusted || !['Enter', 'Tab', 'Escape'].includes(event.key)) return;
    const focused = document.activeElement instanceof Element ? document.activeElement : document.body;
    report('press', focused, { key: event.key });
  }, true);
})()"#
);

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
    /// The main frame showed a document, or moved within it.
    Navigation {
        /// Its URL, secret-looking values hidden.
        url: String,
        /// Whether a fragment or the history API moved it within its document (#506).
        within: bool,
    },
    /// What the user did to an element (#506): a click, a fill or a key pressed, with the
    /// element's locators as they were then.
    Action {
        /// `click`, `fill` or `press`.
        action: String,
        /// Ways to find the element again, the most durable first.
        locators: Vec<ActionLocator>,
        /// The page's URL then, secret-looking values hidden.
        url: String,
        /// A fill's text, an ordinary field's; none for a secret one.
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        /// Whether the field is a secret one, whose text is not kept.
        secret: bool,
        /// A press's key: `Enter`, `Tab` or `Escape`.
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        /// The field's `name`, or its id when not generated, which names a secret's variable.
        #[serde(skip_serializing_if = "Option::is_none")]
        field: Option<String>,
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

/// One way to find an action's target again (#506), as the listener found it at the event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionLocator {
    /// `test id`, `role`, `label`, `placeholder`, `text` or `css`.
    pub kind: String,
    /// The test id, the accessible name, the label, the placeholder, the text or the selector.
    pub value: String,
    /// A test id's attribute: `data-testid`, `data-test`, `data-cy` or `data-qa`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribute: Option<String>,
    /// A role locator's role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Whether it found the element alone at the event.
    #[serde(default)]
    pub unique: bool,
}

/// An action as the listener reports it (#506).
#[derive(Debug, Clone, Deserialize)]
pub struct ReportedAction {
    action: String,
    locators: Vec<ActionLocator>,
    url: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    secret: bool,
    #[serde(default)]
    key: Option<String>,
    #[serde(default)]
    field: Option<String>,
}

impl ReportedAction {
    /// Reads a report the listener sent.
    ///
    /// # Errors
    ///
    /// When the report is not an action's.
    pub fn parse(payload: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(payload)
    }

    /// The report as the minute keeps it: the URL's secret-looking values hidden, each text held
    /// to its trip cap, and no text for a secret field whatever the page sent. None for an action
    /// or a key the listener does not report.
    #[must_use]
    pub fn into_entry(self) -> Option<Entry> {
        if !matches!(self.action.as_str(), "click" | "fill" | "press") {
            return None;
        }
        let key = match self.key {
            Some(key) if matches!(key.as_str(), "Enter" | "Tab" | "Escape") => Some(key),
            Some(_) => return None,
            None => None,
        };
        Some(Entry::Action {
            locators: self
                .locators
                .into_iter()
                .map(|locator| ActionLocator {
                    value: within(&locator.value, LOCATOR_TRIP),
                    ..locator
                })
                .collect(),
            url: redact_url(&self.url),
            text: self
                .text
                .filter(|_| !self.secret)
                .map(|text| within(&text, FILL_TRIP)),
            secret: self.secret,
            key,
            field: self.field.map(|field| within(&field, LOCATOR_TRIP)),
            action: self.action,
        })
    }
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
    /// The root of the project whose tab it was recorded in (#506), where a test drafted from it
    /// goes.
    pub project: Option<String>,
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
    /// Keeps `entry`, which came `now`: typing adds to the typing just before it, a wheel turn to
    /// the scroll just before it, and a fill to the fill of the same field when no other action
    /// came between (#506), whose text it takes. The merged fill keeps its first time, so the
    /// minute stays in time order.
    pub fn push(&mut self, entry: Entry, now: Instant) {
        if let Entry::Action {
            action,
            locators,
            text,
            ..
        } = &entry
            && action == "fill"
            && let Some(last) = self
                .entries
                .iter_mut()
                .rev()
                .find(|timed| matches!(timed.value, Entry::Action { .. }))
            && let Entry::Action {
                action: last_action,
                locators: last_locators,
                text: last_text,
                ..
            } = &mut last.value
            && last_action == "fill"
            && last_locators.first() == locators.first()
        {
            last_text.clone_from(text);
            return;
        }
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

impl Page {
    /// Watches what the user does in the page (#506): the binding for the action listener's
    /// world, the listener in that world of each document `session` loads from now on, and in a
    /// new world of the main frame's document already loaded. The listener acts in the top frame
    /// only.
    ///
    /// # Errors
    ///
    /// When a call fails or the browser's answer lacks the main frame or a context.
    pub async fn watch_actions(&self, session: &str) -> Result<(), CdpError> {
        self.call_in(
            session,
            "Runtime.addBinding",
            json!({ "name": BINDING, "executionContextName": WORLD }),
        )
        .await?;
        self.call_in(
            session,
            "Page.addScriptToEvaluateOnNewDocument",
            json!({ "source": LISTENER, "worldName": WORLD }),
        )
        .await?;
        let tree = self
            .call_in(session, "Page.getFrameTree", json!({}))
            .await?;
        let frame = tree
            .pointer("/frameTree/frame/id")
            .and_then(Value::as_str)
            .ok_or_else(|| CdpError::Unexpected("the page has no main frame".to_string()))?;
        let world = self
            .call_in(
                session,
                "Page.createIsolatedWorld",
                json!({ "frameId": frame, "worldName": WORLD }),
            )
            .await?;
        let context = world
            .get("executionContextId")
            .and_then(Value::as_i64)
            .ok_or_else(|| CdpError::Unexpected("the record world has no context".to_string()))?;
        self.call_in(
            session,
            "Runtime.evaluate",
            json!({ "expression": LISTENER, "contextId": context }),
        )
        .await
        .map(drop)
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
