//! A local terminal's output kept across a restart, and drawn again, dimmed, when Zed restores the
//! terminal (#751).
//!
//! [`init`] sets `terminal`'s `MarleyTerminalHistory`. Each local interactive terminal's builder
//! asks it for a recorder, which runs on the terminal's IO thread, keeps what draws
//! (`marley_dcs::history::Allowlist`) and, at each shell hook that opens or closes a block, a mark
//! (`marley_terminal::history::Mark`, #764), and sends them to one writer task. A restored
//! terminal's builder asks it for what draws the kept output, cut at the marks
//! (`marley_terminal::history::restore`), and places the blocks itself. The writer appends a
//! terminal's output at most a second after it came, compacts a file grown past the caps, and
//! writes what is pending at the quit. The files are
//! `<data dir>/terminal_history/<MARLEY_TERMINAL_ID>.log` (`marley_terminal::history`).
//!
//! A terminal's file goes when the user closes the terminal (its view released while its window
//! is still open and Marley is not quitting), when no saved terminal holds its id at the start,
//! and every file when `marley.restore_scrollback` turns off, which also stops every recorder.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use futures::StreamExt as _;
use futures::channel::{mpsc, oneshot};
use futures::future::{self, Either};
use gpui::{App, AppContext as _, BackgroundExecutor, Context, Global, Task, Window};
use marley_dcs::history::Allowlist;
use marley_dcs::{DcsEvent, DcsScanner};
use marley_terminal::anchored::ZED_STARTUP_MARKER;
use marley_terminal::history::{self, MAX_BYTES, MAX_LINES, Mark, Piece};
use marley_terminal::{DcsHook, RawDcs};
use settings::{Settings as _, SettingsStore};
use terminal::MarleyTerminalHistory;
use terminal_view::TerminalView;
use util::ResultExt as _;

use crate::{MarleySettings, RestoreScrollback};

/// How long a terminal's output waits for more before it is written.
const WRITE_AFTER: Duration = Duration::from_secs(1);

/// What a terminal's file holds before it is compacted to the caps.
const COMPACT_PAST_BYTES: u64 = 2 * MAX_BYTES as u64;

/// A terminal's recorder, fed on its IO thread.
type Recorder = Box<dyn FnMut(&[u8]) + Send>;

/// What the writer is told.
enum Message {
    /// What a terminal printed that draws.
    Output { id: Arc<str>, bytes: Vec<u8> },
    /// Removes every file but the terminals' named, or every one with `None`.
    Retain(Option<HashSet<String>>),
    /// Removes a closed terminal's file; its later output is not kept.
    Remove(Arc<str>),
    /// Writes what is pending, then answers.
    Flush(oneshot::Sender<()>),
}

struct Scrollback {
    writer: mpsc::UnboundedSender<Message>,
    enabled: Arc<AtomicBool>,
    quitting: bool,
}

impl Global for Scrollback {}

/// Sets the hook, starts the writer and removes the files no saved terminal holds;
/// [`crate::init`] calls it once, after [`crate::terminal_ids::init`] and before any window
/// restores its terminals.
pub fn init(cx: &mut App) {
    init_in(paths::data_dir().join("terminal_history"), cx);
}

fn init_in(dir: PathBuf, cx: &mut App) {
    let on = MarleySettings::get_global(cx).restore_scrollback == RestoreScrollback::Kept;
    let enabled = Arc::new(AtomicBool::new(on));
    let (writer, messages) = mpsc::unbounded();
    let keep = on.then(|| crate::terminal_ids::known_ids(cx));
    send(&writer, Message::Retain(keep));
    let executor = cx.background_executor().clone();
    cx.background_spawn(write(dir.clone(), messages, executor))
        .detach();
    cx.set_global(MarleyTerminalHistory {
        restored: Arc::new({
            let writer = writer.clone();
            let enabled = Arc::clone(&enabled);
            move |id: &str| restored(&dir, id, &writer, &enabled)
        }),
        recorder: Arc::new({
            let writer = writer.clone();
            let enabled = Arc::clone(&enabled);
            move |id: &str, nonce: Option<&str>| {
                recorder(id, nonce, writer.clone(), Arc::clone(&enabled))
            }
        }),
    });
    cx.set_global(Scrollback {
        writer,
        enabled,
        quitting: false,
    });
    cx.observe_global::<SettingsStore>(|cx| {
        let on = MarleySettings::get_global(cx).restore_scrollback == RestoreScrollback::Kept;
        let scrollback = cx.global::<Scrollback>();
        if scrollback.enabled.swap(on, Ordering::Relaxed) && !on {
            send(&scrollback.writer, Message::Retain(None));
        }
    })
    .detach();
    cx.on_app_quit(|cx| {
        let scrollback = cx.global_mut::<Scrollback>();
        scrollback.quitting = true;
        let (written, done) = oneshot::channel();
        send(&scrollback.writer, Message::Flush(written));
        async move {
            if done.await.is_err() {
                log::warn!("terminal history: the writer ended before the quit's write");
            }
        }
    })
    .detach();
    cx.observe_new(
        |view: &mut TerminalView, window: Option<&mut Window>, cx: &mut Context<TerminalView>| {
            let Some(window) = window.map(|window| window.window_handle()) else {
                return;
            };
            let Some(id) = view
                .terminal()
                .read(cx)
                .marley_terminal_id()
                .map(Arc::<str>::from)
            else {
                return;
            };
            cx.on_release(move |_, cx| {
                // gpui takes a closing window out of the list before its views go, and Zed
                // restores a closed window's terminals: only a terminal closed in a window still
                // open is gone.
                let scrollback = cx.global::<Scrollback>();
                if !scrollback.quitting && cx.windows().contains(&window) {
                    send(&scrollback.writer, Message::Remove(id));
                }
            })
            .detach();
        },
    )
    .detach();
}

fn send(writer: &mpsc::UnboundedSender<Message>, message: Message) {
    if writer.unbounded_send(message).is_err() {
        log::warn!("terminal history: the writer has ended");
    }
}

/// What draws the terminal `id`'s kept output, dimmed and cut at its marks, when the setting is
/// on. It runs in a terminal's builder, off the main thread, before the terminal's recorder
/// exists; the file gains the end the replay draws, so the new session's output starts on a line
/// of its own there too.
fn restored(
    dir: &Path,
    id: &str,
    writer: &mpsc::UnboundedSender<Message>,
    enabled: &AtomicBool,
) -> Option<Vec<Piece>> {
    if !enabled.load(Ordering::Relaxed) {
        return None;
    }
    let kept = history::read_in(dir, id).log_err().flatten()?;
    send(
        writer,
        Message::Output {
            id: Arc::from(id),
            bytes: marley_dcs::history::SESSION_END.to_vec(),
        },
    );
    Some(history::restore(&kept))
}

/// The terminal `id`'s recorder, run on its IO thread: what draws of each read, and a mark for
/// each hook that opens or closes a block, go to the writer while the setting is on. The event
/// loop gives it each hook frame as it came, in stream order (#764); a command's mark says whether
/// its frame carried `nonce`, the terminal's, and a hook on the alternate screen, which opens no
/// block, leaves none. The allowlist reads every byte either way, so it knows whether the
/// alternate screen shows when the setting turns on again.
fn recorder(
    id: &str,
    nonce: Option<&str>,
    writer: mpsc::UnboundedSender<Message>,
    enabled: Arc<AtomicBool>,
) -> Recorder {
    let id = Arc::<str>::from(id);
    let nonce = nonce.map(str::to_string);
    let mut scanner = DcsScanner::default();
    let mut allowlist = Allowlist::default();
    let mut writer = Some(writer);
    Box::new(move |bytes: &[u8]| {
        let mut kept = Vec::new();
        for event in scanner.feed(bytes) {
            match event {
                DcsEvent::Passthrough(bytes) => allowlist.feed(&bytes, &mut kept),
                DcsEvent::Hook(frame) => {
                    if !allowlist.on_alternate_screen()
                        && let Some(mark) = mark_of(&frame, nonce.as_deref())
                    {
                        kept.extend_from_slice(&mark.frame());
                    }
                }
            }
        }
        if kept.is_empty() || !enabled.load(Ordering::Relaxed) {
            return;
        }
        let message = Message::Output {
            id: Arc::clone(&id),
            bytes: kept,
        };
        // The writer ends only after the quit's write; what comes after it is not kept.
        if writer
            .as_ref()
            .is_some_and(|sender| sender.unbounded_send(message).is_err())
        {
            writer = None;
        }
    })
}

/// The mark a hook frame leaves: a command's, with whether it carried `nonce`, or a prompt's;
/// none for any other hook, for Zed's startup check (#639) or for a frame that does not decode.
fn mark_of(frame: &RawDcs, nonce: Option<&str>) -> Option<Mark> {
    let hook = match marley_terminal::decode_frame(frame).ok()? {
        DcsHook::Signed { hook, .. } => *hook,
        hook => hook,
    };
    let at_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .and_then(|since| u64::try_from(since.as_millis()).ok())?;
    match hook {
        DcsHook::Preexec(value) if !value.command.contains(ZED_STARTUP_MARKER) => {
            Some(Mark::Command {
                at_ms,
                verified: nonce.is_some() && value.nonce.as_deref() == nonce,
                command: value.command,
            })
        }
        DcsHook::Precmd(value) => Some(Mark::Prompt {
            at_ms,
            exit: value.exit_code.0,
            pwd: value.prompt.pwd,
        }),
        _ => None,
    }
}

/// The writer: appends each terminal's pending output once it has waited [`WRITE_AFTER`], or at
/// once past the caps, and acts on the other messages in the order they came.
async fn write(
    dir: PathBuf,
    mut messages: mpsc::UnboundedReceiver<Message>,
    executor: BackgroundExecutor,
) {
    let mut pending = Pending::default();
    let mut due: Option<Task<()>> = None;
    loop {
        let message = match due.take() {
            None => messages.next().await,
            Some(timer) => match future::select(messages.next(), timer).await {
                Either::Left((message, timer)) => {
                    due = Some(timer);
                    message
                }
                Either::Right(((), _)) => {
                    pending.write(&dir);
                    continue;
                }
            },
        };
        let Some(message) = message else {
            pending.write(&dir);
            return;
        };
        match message {
            Message::Output { id, bytes } => {
                if pending.add(id, &bytes) {
                    pending.write(&dir);
                    due = None;
                } else if due.is_none() {
                    due = Some(executor.timer(WRITE_AFTER));
                }
            }
            Message::Retain(keep) => {
                if keep.is_none() {
                    pending.output.clear();
                }
                history::retain_in(&dir, keep.as_ref()).log_err();
            }
            Message::Remove(id) => {
                let _dropped = pending.output.remove(&id);
                history::remove_in(&dir, &id).log_err();
                let _new = pending.removed.insert(id);
            }
            Message::Flush(written) => {
                pending.write(&dir);
                due = None;
                if written.send(()).is_err() {
                    log::debug!("terminal history: the quit stopped waiting for its write");
                }
            }
        }
    }
}

/// The writer's state: each terminal's output not yet written, the lines each file gained since
/// it was compacted, and the closed terminals, whose output is not kept.
#[derive(Default)]
struct Pending {
    output: HashMap<Arc<str>, Vec<u8>>,
    lines: HashMap<Arc<str>, usize>,
    removed: HashSet<Arc<str>>,
}

impl Pending {
    /// Adds a terminal's output; true when what is pending passed the caps and is written now.
    fn add(&mut self, id: Arc<str>, bytes: &[u8]) -> bool {
        if self.removed.contains(&id) {
            return false;
        }
        let output = self.output.entry(id).or_default();
        output.extend_from_slice(bytes);
        output.len() > MAX_BYTES
    }

    /// Appends each terminal's pending output, compacting a file grown past twice the caps.
    fn write(&mut self, dir: &Path) {
        for (id, bytes) in self.output.drain() {
            let Some(length) = history::append_in(dir, &id, &bytes).log_err() else {
                continue;
            };
            let lines = self.lines.entry(Arc::clone(&id)).or_default();
            *lines += bytes.split(|byte| *byte == b'\n').count() - 1;
            if length > COMPACT_PAST_BYTES || *lines > MAX_LINES {
                *lines = 0;
                history::compact_in(dir, &id).log_err();
            }
        }
    }
}
