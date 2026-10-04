//! Per-turn diffs for Claude Code in a terminal (#509).
//!
//! Each turn of a terminal's Claude Code, from #519's events, opens and closes with a checkpoint
//! of the repository it works in: Zed's own, which takes the tree with the shell's edits as well
//! as the agent's, through a temporary index that leaves the user's alone. A turn that changed
//! the tree becomes a commit of the close's tree whose parent is the open's checkpoint, pinned
//! under `refs/marley/turns/<session>/<n>`, so Zed's commit view, which diffs against the first
//! parent, shows that turn and nothing else. The rail lists the turns under the terminal's row.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context as _;
use futures::FutureExt as _;
use futures::future::Shared;
use git::repository::GitRepositoryCheckpoint;
use git::status::DiffTreeType;
use gpui::{App, AppContext as _, AsyncApp, Entity, Global, SharedString, Task, WeakEntity};
use marley_agent::claude_events::{
    CWD_LABEL, HookEvent, PROMPT_LABEL, PromptOrigin, PromptReading, SESSION_LABEL,
};
use marley_fleet::Session;
use project::git_store::Repository;
use terminal_view::TerminalView;
use util::ResultExt as _;

use crate::turn_git::{self, TURN_REFS};

/// How old a turn ref may grow before a session's first turn prunes it.
const KEPT: Duration = Duration::from_hours(30 * 24);

/// A turn a terminal's Claude Code finished that changed the tree, and the commit that holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    /// The prompt on one line, or what injected it.
    pub title: String,
    /// How many files it changed.
    pub files: usize,
    /// Whether it ended in an error (`StopFailure`).
    pub failed: bool,
    /// Whether a harness's prompt started it.
    pub injected: bool,
    /// Its commit, in full.
    pub sha: String,
    /// The repository it was taken in, which holds its commit.
    pub repository: WeakEntity<Repository>,
}

/// A checkpoint being taken, shared so a turn's close waits for its open.
type Checkpoint = Shared<Task<Option<GitRepositoryCheckpoint>>>;

/// A turn being recorded, shared so the terminal's next turn waits for it.
type Recording = Shared<Task<()>>;

/// A turn not closed yet.
#[derive(Debug)]
struct Open {
    title: String,
    injected: bool,
    checkpoint: Checkpoint,
}

/// One terminal's turns.
#[derive(Debug)]
struct Seat {
    session: String,
    repository: WeakEntity<Repository>,
    work_dir: PathBuf,
    open: Option<Open>,
    turns: Vec<Turn>,
    /// The last turn being recorded: the next waits for it, so each takes the next number and
    /// lists in order.
    recording: Option<Recording>,
}

/// Every terminal's turns, by the terminal view's id. The rail observes it.
#[derive(Debug, Default)]
pub struct Turns {
    seats: HashMap<u64, Seat>,
    /// The sessions that have pinned a turn: a session's first prunes the old refs.
    pinned: HashSet<String>,
}

impl Global for Turns {}

impl Turns {
    /// The finished turns of the terminal `view`, oldest first.
    #[must_use]
    pub fn of(view: u64, cx: &App) -> &[Turn] {
        cx.try_global::<Self>()
            .and_then(|turns| turns.seats.get(&view))
            .map_or(&[], |seat| seat.turns.as_slice())
    }

    /// The repository the turn `sha` of the terminal `view` was taken in.
    #[must_use]
    pub fn repository_of(view: u64, sha: &str, cx: &App) -> Option<WeakEntity<Repository>> {
        Self::of(view, cx)
            .iter()
            .find(|turn| turn.sha == sha)
            .map(|turn| turn.repository.clone())
    }
}

/// Follows `event`, which moved the terminal `view`'s seat to `seat`: a lead prompt opens a turn,
/// closing the one open, and the turn's ends close it. `id` is the view's entity id.
pub(crate) fn on_event(
    view: &TerminalView,
    id: u64,
    event: &HookEvent,
    seat: &Session,
    reading: PromptReading,
    cx: &mut App,
) {
    let session = event
        .session_id
        .clone()
        .or_else(|| seat.labels.get(SESSION_LABEL).cloned());
    // A new session in the terminal ends the last session's turn.
    let other_session = session.as_deref().is_some_and(|session| {
        cx.try_global::<Turns>()
            .and_then(|turns| turns.seats.get(&id))
            .is_some_and(|known| known.open.is_some() && known.session != session)
    });
    if other_session {
        drop(close(id, false, cx));
    }
    match event.event.as_str() {
        "UserPromptSubmit" => {
            let prompt = event
                .prompt
                .clone()
                .or_else(|| seat.labels.get(PROMPT_LABEL).cloned())
                .unwrap_or_default();
            let origin = reading.origin(&prompt);
            if origin == PromptOrigin::Continuation {
                return;
            }
            let reused = close(id, false, cx);
            let (title, injected) = title_of(&prompt, &origin);
            let folder = event
                .cwd
                .as_ref()
                .or_else(|| seat.labels.get(CWD_LABEL))
                .map(PathBuf::from)
                .or_else(|| view.terminal().read(cx).working_directory());
            // The session names the turn's refs.
            let session = session.filter(|session| ref_safe(session));
            if let (Some(session), Some(folder)) = (session, folder) {
                open(view, id, &session, &folder, (title, injected), reused, cx);
            }
        }
        "Stop" | "SessionEnd" | "SessionStart" => drop(close(id, false, cx)),
        "StopFailure" => drop(close(id, true, cx)),
        "PostCompact" if event.trigger.as_deref() == Some("manual") => drop(close(id, false, cx)),
        "PostToolUseFailure" if event.is_interrupt == Some(true) => drop(close(id, false, cx)),
        _ => {}
    }
}

/// Closes the open turn of each terminal named, whose Claude Code has left it (#547).
pub(crate) fn on_end(ids: &[u64], cx: &mut App) {
    for id in ids {
        drop(close(*id, false, cx));
    }
}

/// Closes the open turn of the terminal `id`, which is closing, and forgets its turns: its rows
/// go with it, and its refs keep the commits.
pub(crate) fn forget(id: u64, cx: &mut App) {
    drop(close(id, false, cx));
    if cx
        .try_global::<Turns>()
        .is_some_and(|turns| turns.seats.contains_key(&id))
    {
        drop(cx.default_global::<Turns>().seats.remove(&id));
    }
}

/// Whether `session` can name a ref's part: Claude Code's session ids are UUIDs.
fn ref_safe(session: &str) -> bool {
    !session.is_empty()
        && session.len() <= 128
        && session
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

/// A turn's title and whether a harness started it: the prompt on one line, the slash command, or
/// what injected it.
fn title_of(prompt: &str, origin: &PromptOrigin) -> (String, bool) {
    match origin {
        PromptOrigin::User => {
            let line = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
            (
                if line.is_empty() {
                    "a prompt".to_string()
                } else {
                    line
                },
                false,
            )
        }
        PromptOrigin::SlashCommand(name) if !name.is_empty() => (name.clone(), false),
        PromptOrigin::SlashCommand(_) => ("a slash command".to_string(), false),
        PromptOrigin::Injected(Some(tag)) => (tag.replace('-', " "), true),
        PromptOrigin::Injected(None) | PromptOrigin::Continuation => {
            ("injected prompt".to_string(), true)
        }
    }
}

/// Opens a turn for the terminal `id` in the repository that holds `folder`, with the checkpoint
/// a closing turn in that repository took, or a new one.
fn open(
    view: &TerminalView,
    id: u64,
    session: &str,
    folder: &Path,
    (title, injected): (String, bool),
    reused: Option<(WeakEntity<Repository>, Checkpoint)>,
    cx: &mut App,
) {
    let Some(workspace) = view.marley_workspace().upgrade() else {
        return;
    };
    let project = workspace.read(cx).project().clone();
    if !project.read(cx).is_local() {
        return;
    }
    let git_store = project.read(cx).git_store().clone();
    let Some(repository) = repository_for(folder, git_store.read(cx).repositories().values(), cx)
    else {
        return;
    };
    let checkpoint = match reused {
        Some((reused_repository, checkpoint)) if reused_repository == repository.downgrade() => {
            checkpoint
        }
        _ => take_checkpoint(&repository, cx),
    };
    let work_dir = repository.read(cx).work_directory_abs_path.to_path_buf();
    let turns = cx.default_global::<Turns>();
    let seat = turns.seats.entry(id).or_insert_with(|| Seat {
        session: session.to_string(),
        repository: repository.downgrade(),
        work_dir: work_dir.clone(),
        open: None,
        turns: Vec::new(),
        recording: None,
    });
    session.clone_into(&mut seat.session);
    seat.repository = repository.downgrade();
    seat.work_dir = work_dir;
    seat.open = Some(Open {
        title,
        injected,
        checkpoint,
    });
}

/// The innermost of `repositories` whose work directory holds `folder`, as the agent bar finds a
/// terminal's branch.
fn repository_for<'a>(
    folder: &Path,
    repositories: impl IntoIterator<Item = &'a Entity<Repository>>,
    cx: &App,
) -> Option<Entity<Repository>> {
    repositories
        .into_iter()
        .filter(|repository| folder.starts_with(&repository.read(cx).work_directory_abs_path))
        .max_by_key(|repository| {
            repository
                .read(cx)
                .work_directory_abs_path
                .components()
                .count()
        })
        .cloned()
}

/// A checkpoint of `repository`, taken in its job queue after what is queued there already.
fn take_checkpoint(repository: &Entity<Repository>, cx: &mut App) -> Checkpoint {
    let receiver = repository.update(cx, |repository, _| repository.checkpoint());
    cx.background_spawn(async move { receiver.await.ok()?.log_err() })
        .shared()
}

/// Closes the open turn of the terminal `id`, `failed` when it ended in an error, and records it
/// once its checkpoints are taken, after the terminal's turn before it; the close's checkpoint,
/// which a prompt's new turn opens with.
fn close(id: u64, failed: bool, cx: &mut App) -> Option<(WeakEntity<Repository>, Checkpoint)> {
    let seat = cx.try_global::<Turns>()?.seats.get(&id)?;
    seat.open.as_ref()?;
    let repository = seat.repository.upgrade()?;
    let work_dir = seat.work_dir.clone();
    let session = seat.session.clone();
    let (open, previous) = {
        let seat = cx.default_global::<Turns>().seats.get_mut(&id)?;
        (seat.open.take()?, seat.recording.take())
    };
    let closing = take_checkpoint(&repository, cx);
    let turn = Closing {
        id,
        session,
        work_dir,
        failed,
        open,
        end: closing.clone(),
    };
    let weak = repository.downgrade();
    let recording = weak.clone();
    let recorded = cx
        .spawn(async move |cx| {
            if let Some(previous) = previous {
                previous.await;
            }
            record(turn, recording, cx).await.log_err();
        })
        .shared();
    if let Some(seat) = cx.default_global::<Turns>().seats.get_mut(&id) {
        seat.recording = Some(recorded.clone());
    }
    // The record runs to its end though the terminal closes first.
    cx.spawn(async move |_| recorded.await).detach();
    Some((weak, closing))
}

/// A turn being closed.
struct Closing {
    id: u64,
    session: String,
    work_dir: PathBuf,
    failed: bool,
    open: Open,
    /// The checkpoint of the turn's end.
    end: Checkpoint,
}

/// Records the turn `turn` in `repository` once its checkpoints are taken: nothing when its tree
/// did not change, and otherwise its commit, pinned, and its row.
async fn record(
    turn: Closing,
    repository: WeakEntity<Repository>,
    cx: &mut AsyncApp,
) -> anyhow::Result<()> {
    let start = turn
        .open
        .checkpoint
        .await
        .context("the turn's start has no checkpoint")?;
    let end = turn.end.await.context("the turn's end has no checkpoint")?;
    let start_sha = start.commit_sha.to_string();
    let end_sha = end.commit_sha.to_string();
    let unchanged = repository
        .update(cx, move |repository, _| {
            repository.compare_checkpoints(start, end)
        })?
        .await??;
    if unchanged {
        return Ok(());
    }
    let refs = turn_git::turn_refs_in(&turn.work_dir).await?;
    let prefix = format!("{TURN_REFS}{}/", turn.session);
    let number = refs
        .iter()
        .filter_map(|known| known.name.strip_prefix(&prefix)?.parse::<u64>().ok())
        .max()
        .unwrap_or_default()
        + 1;
    let message = [
        turn.open.title.clone(),
        format!("Marley-Session: {}\nMarley-Turn: {number}", turn.session),
    ];
    let sha = turn_git::commit_tree_in(
        &turn.work_dir,
        &format!("{end_sha}^{{tree}}"),
        &start_sha,
        &message,
    )
    .await?;
    repository
        .update(cx, |repository, _| {
            repository.update_ref(format!("{prefix}{number}"), sha.clone())
        })?
        .await??;
    let diff_type = DiffTreeType::Since {
        base: SharedString::from(start_sha),
        head: SharedString::from(sha.clone()),
    };
    let files = repository
        .update(cx, |repository, cx| repository.diff_tree(diff_type, cx))?
        .await??
        .entries
        .len();
    let first = cx.update(|cx| {
        cx.default_global::<Turns>()
            .pinned
            .insert(turn.session.clone())
    });
    if first {
        prune(&refs, &repository, cx).await;
    }
    let listed = Turn {
        title: turn.open.title,
        files,
        failed: turn.failed,
        injected: turn.open.injected,
        sha,
        repository: repository.clone(),
    };
    let id = turn.id;
    cx.update(|cx| {
        if let Some(seat) = cx.default_global::<Turns>().seats.get_mut(&id) {
            seat.turns.push(listed);
        }
    });
    Ok(())
}

/// Deletes the turn refs of `refs` whose commits are older than [`KEPT`].
async fn prune(refs: &[turn_git::TurnRef], repository: &WeakEntity<Repository>, cx: &mut AsyncApp) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let cutoff = now.saturating_sub(KEPT.as_secs());
    for old in refs.iter().filter(|known| known.committed < cutoff) {
        let deleting = repository
            .update(cx, |repository, _| repository.delete_ref(old.name.clone()))
            .log_err();
        if let Some(deleting) = deleting {
            deleting
                .await
                .map_err(anyhow::Error::from)
                .and_then(|deleted| deleted)
                .log_err();
        }
    }
}
