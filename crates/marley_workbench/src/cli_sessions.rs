//! Past Claude Code and Codex sessions in Threads (#754).
//!
//! The sessions Chad runs in Marley's terminals are kept by each CLI in its own files: Claude
//! Code's transcripts under `<config>/projects`, `<config>` being `CLAUDE_CONFIG_DIR` or
//! `~/.claude`, and Codex's rollouts under `<home>/sessions`, named in `<home>/session_index.jsonl`,
//! `<home>` being `CODEX_HOME` or `~/.codex`. A scan reads them, read-only and off the main thread:
//! the newest 500 per CLI, from each its first MiB and last 256 KiB, parsed by
//! `marley_agent::cli_sessions`, and kept by path, size and time so an unchanged file is not read
//! again. The Threads page, the rail's Threads view and Reopen Thread… ask for a scan as they
//! open, at most every 30 seconds, and list what the `CliSessions` global holds.
//! Reopening one brings forward the Marley terminal running it, or resumes it with the CLI's own
//! resume in a new terminal in its folder. `marley.list_cli_sessions` off reads nothing.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read as _, Seek as _, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use chrono::{DateTime, Utc};
use gpui::{App, AppContext as _, Global, TaskExt as _, WeakEntity, Window};
use marley_agent::AgentKind;
use marley_agent::cli_sessions::{self, CliSession, Windows};
use settings::Settings as _;
use workspace::notifications::NotificationId;
use workspace::{MultiWorkspace, Toast, Workspace};

use crate::{ListCliSessions, MarleySettings};

/// How many of each CLI's newest session files a scan reads.
const MAX_FILES: usize = 500;
/// How much of a session file's start a scan reads.
const HEAD_BYTES: u64 = 1024 * 1024;
/// How much of a long session file's end a scan reads.
const TAIL_BYTES: u64 = 256 * 1024;
/// The most of Codex's session index a scan reads.
const MAX_INDEX_BYTES: u64 = 16 * 1024 * 1024;
/// How long a scan's list stands before the next one.
const RESCAN_AFTER: Duration = Duration::from_secs(30);

/// A session as the lists show it: the session and when its file last changed.
#[derive(Clone)]
pub(crate) struct Listed {
    pub(crate) session: Arc<CliSession>,
    pub(crate) modified: DateTime<Utc>,
}

/// A session file a scan read: its size and time when read, and the session it held.
struct Cached {
    size: u64,
    modified: SystemTime,
    session: Option<Arc<CliSession>>,
}

type Cache = HashMap<PathBuf, Cached>;

/// The sessions the last scan found, newest first, and the scan's state.
#[derive(Default)]
pub(crate) struct CliSessions {
    listed: Arc<[Listed]>,
    read_at: Option<Instant>,
    scanning: bool,
    cache: Option<Cache>,
}

impl Global for CliSessions {}

pub(crate) fn init(cx: &mut App) {
    cx.set_global(CliSessions::default());
}

/// The sessions to list, newest first; none while the switch is off.
pub(crate) fn listed(cx: &App) -> Arc<[Listed]> {
    if MarleySettings::get_global(cx).list_cli_sessions == ListCliSessions::Off {
        return Arc::from([]);
    }
    cx.try_global::<CliSessions>()
        .map_or_else(|| Arc::from([]), |sessions| Arc::clone(&sessions.listed))
}

/// Scans the CLIs' session files again, off the main thread, unless a scan runs or the last one
/// is under [`RESCAN_AFTER`] old; with the switch off, reads nothing.
pub(crate) fn refresh(cx: &mut App) {
    if MarleySettings::get_global(cx).list_cli_sessions == ListCliSessions::Off {
        return;
    }
    let Some(sessions) = cx.try_global::<CliSessions>() else {
        return;
    };
    if sessions.scanning
        || sessions
            .read_at
            .is_some_and(|at| at.elapsed() < RESCAN_AFTER)
    {
        return;
    }
    let sessions = cx.global_mut::<CliSessions>();
    sessions.scanning = true;
    let cache = sessions.cache.take().unwrap_or_default();
    let (claude, codex) = (claude_dir(), codex_dir());
    cx.spawn(async move |cx| {
        let (listed, cache) = cx
            .background_spawn(futures::future::lazy(move |_| {
                scan_in(&claude, &codex, cache)
            }))
            .await;
        cx.update(|cx| {
            let sessions = cx.global_mut::<CliSessions>();
            sessions.listed = listed.into();
            sessions.cache = Some(cache);
            sessions.scanning = false;
            sessions.read_at = Some(Instant::now());
        });
    })
    .detach();
}

/// Claude Code's configuration folder: `CLAUDE_CONFIG_DIR`, else `~/.claude`.
fn claude_dir() -> PathBuf {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .filter(|dir| !dir.is_empty())
        .map_or_else(|| util::paths::home_dir().join(".claude"), PathBuf::from)
}

/// Codex's home: `CODEX_HOME`, else `~/.codex`.
fn codex_dir() -> PathBuf {
    std::env::var_os("CODEX_HOME")
        .filter(|dir| !dir.is_empty())
        .map_or_else(|| util::paths::home_dir().join(".codex"), PathBuf::from)
}

/// A session file found: its path, size and time.
struct Found {
    agent: AgentKind,
    path: PathBuf,
    size: u64,
    modified: SystemTime,
}

/// The sessions in `claude`'s and `codex`'s files, newest first, and the cache for the next scan.
/// A file `cache` holds at the same size and time is not read again.
fn scan_in(claude: &Path, codex: &Path, mut cache: Cache) -> (Vec<Listed>, Cache) {
    let names = read_capped(&codex.join("session_index.jsonl"), MAX_INDEX_BYTES)
        .map(|index| cli_sessions::codex_names(&index))
        .unwrap_or_default();
    let mut found = newest(AgentKind::Claude, claude_files(claude));
    found.extend(newest(AgentKind::Codex, codex_files(codex)));
    let mut next = Cache::new();
    let mut listed = Vec::new();
    for file in found {
        let kept = cache
            .remove(&file.path)
            .filter(|cached| cached.size == file.size && cached.modified == file.modified);
        let session = match kept {
            Some(cached) => cached.session,
            None => read_session(&file).map(Arc::new),
        };
        if let Some(session) = &session {
            // Codex names a thread in its index, which changes without its rollout.
            let titled = match names
                .get(&session.id)
                .filter(|name| !name.trim().is_empty())
            {
                Some(name) if session.agent == AgentKind::Codex => Arc::new(CliSession {
                    title: Some(name.split_whitespace().collect::<Vec<_>>().join(" ")),
                    ..(**session).clone()
                }),
                _ => Arc::clone(session),
            };
            listed.push(Listed {
                session: titled,
                modified: file.modified.into(),
            });
        }
        let _replaced = next.insert(
            file.path,
            Cached {
                size: file.size,
                modified: file.modified,
                session,
            },
        );
    }
    listed.sort_by_key(|listed| std::cmp::Reverse(listed.modified));
    (listed, next)
}

/// `files` of `agent`, the newest [`MAX_FILES`] by modification time.
fn newest(agent: AgentKind, files: Vec<PathBuf>) -> Vec<Found> {
    let mut found: Vec<Found> = files
        .into_iter()
        .filter_map(|path| {
            let metadata = std::fs::metadata(&path).ok()?;
            Some(Found {
                agent,
                size: metadata.len(),
                modified: metadata.modified().ok()?,
                path,
            })
        })
        .collect();
    found.sort_by_key(|file| std::cmp::Reverse(file.modified));
    found.truncate(MAX_FILES);
    found
}

/// Claude Code's transcripts: `<config>/projects/<project>/<session>.jsonl`.
fn claude_files(config: &Path) -> Vec<PathBuf> {
    entries(&config.join("projects"))
        .into_iter()
        .flat_map(|project| entries(&project))
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "jsonl")
        })
        .collect()
}

/// Codex's rollouts: `<home>/sessions/YYYY/MM/DD/rollout-*.jsonl`.
fn codex_files(home: &Path) -> Vec<PathBuf> {
    entries(&home.join("sessions"))
        .into_iter()
        .flat_map(|year| entries(&year))
        .flat_map(|month| entries(&month))
        .flat_map(|day| entries(&day))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("rollout-"))
                && path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("jsonl"))
        })
        .collect()
}

/// The paths in `folder`; none when it cannot be read.
fn entries(folder: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(folder).map_or_else(
        |_| Vec::new(),
        |read| {
            read.filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .collect()
        },
    )
}

/// The session `file` holds: its whole bytes, or its head and tail for a long one.
fn read_session(file: &Found) -> Option<CliSession> {
    let mut opened = File::open(&file.path).ok()?;
    let (head, tail) = if file.size <= HEAD_BYTES + TAIL_BYTES {
        (read_up_to(&mut opened, HEAD_BYTES + TAIL_BYTES)?, None)
    } else {
        let head = read_up_to(&mut opened, HEAD_BYTES)?;
        opened
            .seek(SeekFrom::End(-i64::try_from(TAIL_BYTES).ok()?))
            .ok()?;
        (head, Some(read_up_to(&mut opened, TAIL_BYTES)?))
    };
    let windows = Windows {
        head: &head,
        tail: tail.as_deref(),
    };
    if file.agent == AgentKind::Codex {
        cli_sessions::parse_codex(windows)
    } else {
        let stem = file.path.file_stem()?.to_str()?;
        cli_sessions::parse_claude(windows, stem)
    }
}

/// Up to `limit` bytes from `file`'s position.
fn read_up_to(file: &mut File, limit: u64) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let _read = file.take(limit).read_to_end(&mut bytes).ok()?;
    Some(bytes)
}

/// `path`'s bytes, when it is there and no larger than `limit`.
fn read_capped(path: &Path, limit: u64) -> Option<Vec<u8>> {
    if std::fs::metadata(path).ok()?.len() > limit {
        return None;
    }
    std::fs::read(path).ok()
}

/// Reopens `session` (#754): the Marley terminal running it comes forward; otherwise the CLI's own
/// resume runs in a new center terminal of `workspace` in the session's folder, with the project's
/// permission arguments, or a toast says the folder is gone. Called inside the window's update, so
/// the running terminal is looked for through the window's own root.
pub(crate) fn reopen(
    workspace: &WeakEntity<Workspace>,
    session: &Arc<CliSession>,
    window: &mut Window,
    cx: &mut App,
) {
    if let Some(view) = crate::agent_events::view_running(session.agent, &session.id, cx)
        .and_then(|view| running_view(view, window, cx))
    {
        crate::browser::reveal_terminal(&view, window, cx);
        return;
    }
    let folder = PathBuf::from(&session.folder);
    let (workspace, session) = (workspace.clone(), Arc::clone(session));
    window
        .spawn(cx, async move |cx| {
            let there = cx
                .background_spawn({
                    let folder = folder.clone();
                    async move {
                        smol::fs::metadata(&folder)
                            .await
                            .is_ok_and(|metadata| metadata.is_dir())
                    }
                })
                .await;
            let started = cx.update(|window, cx| {
                workspace.update(cx, |workspace, cx| {
                    start(workspace, &session, folder, there, window, cx);
                })
            });
            match started {
                Ok(Ok(())) => {}
                Ok(Err(error)) | Err(error) => {
                    log::info!("cli sessions: the workspace went before the reopen: {error:#}");
                }
            }
        })
        .detach();
}

/// Starts `session`'s resume in a new terminal of `workspace` in `folder`, or says why not.
fn start(
    workspace: &mut Workspace,
    session: &CliSession,
    folder: PathBuf,
    there: bool,
    window: &Window,
    cx: &mut gpui::Context<Workspace>,
) {
    let refused =
        |message: String, workspace: &mut Workspace, cx: &mut gpui::Context<Workspace>| {
            workspace.show_toast(
                Toast::new(NotificationId::unique::<CliSessions>(), message),
                cx,
            );
        };
    if !there {
        refused(
            format!(
                "The session's folder {} is gone, so it was not resumed.",
                folder.display()
            ),
            workspace,
            cx,
        );
        return;
    }
    let Some((line, joining)) =
        crate::agents::resume_input(workspace, session.agent, &session.id, &folder, cx)
    else {
        refused(
            format!(
                "{} cannot resume the session {}.",
                session.agent.display_name(),
                session.id
            ),
            workspace,
            cx,
        );
        return;
    };
    crate::agents::start_in_terminal(
        workspace,
        Some(folder),
        Some(session.agent),
        Some(line),
        joining,
        window,
        cx,
    )
    .detach_and_log_err(cx);
}

/// The terminal view `view` among the window's workspaces.
fn running_view(
    view: gpui::EntityId,
    window: &Window,
    cx: &App,
) -> Option<gpui::Entity<terminal_view::TerminalView>> {
    let members: Vec<gpui::Entity<Workspace>> = window
        .root::<MultiWorkspace>()
        .flatten()?
        .read(cx)
        .workspaces()
        .cloned()
        .collect();
    crate::mcp::terminals_of(members, cx)
        .into_iter()
        .map(|(_, shown)| shown)
        .find(|shown| shown.entity_id() == view)
}
