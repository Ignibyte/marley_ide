//! The `claude` and `codex` Marley finds, their versions, and the integrations it keeps off on
//! a version they were not tested on (#648).
//!
//! The program is the one `MARLEY_CLAUDE` or `MARLEY_CODEX` names, else the first on the search
//! path Marley's agents use. Its version is read with `--version` at start, and again when an
//! agent bar for it draws ten seconds or more after the last check and the file changed (its
//! canonical path, size or modification time), as `agent_notify` re-reads its files; nothing
//! polls. `marley_agent::versions` judges each integration; the agent bar's chip says why one is
//! off, and its click opens the setting that turns it on anyway.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use gpui::{AnyElement, App, Global, Subscription};
use marley_agent::AgentKind;
use marley_agent::claude_events::PromptReading;
use marley_agent::versions::{
    self, CLAUDE_PROMPT_TAGS, CODEX_APP_SERVER, Found, Integration, Off, Verdict, integrations_of,
};
use settings::{Settings as _, SettingsStore};
use terminal::Terminal;
use terminal_view::MarleyFooterContext;
use ui::{Button, Icon, IconName, IconSize, LabelSize, Tooltip, prelude::*};

use crate::MarleySettings;

/// How long a check stands before a drawing bar asks for another.
const FRESH: Duration = Duration::from_secs(10);

/// How long `--version` may take.
const VERSION_TIMEOUT: Duration = Duration::from_secs(5);

/// How much of what `--version` prints is read.
const VERSION_OUTPUT: usize = 4_096;

/// What identifies the file found, so an unchanged one is not run again.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Identity {
    path: PathBuf,
    size: u64,
    modified: Option<SystemTime>,
}

/// One agent's check.
#[derive(Default)]
struct Check {
    /// The program found, as found (not resolved).
    path: Option<PathBuf>,
    identity: Option<Identity>,
    found: Option<Found>,
    checked_at: Option<Instant>,
    running: bool,
}

/// The two agents' checks, and the allowed rows last seen, so a settings change that moves a
/// verdict redraws the bars.
#[derive(Default)]
struct AgentVersions {
    claude: Check,
    codex: Check,
    allowed: Vec<String>,
}

impl Global for AgentVersions {}

impl AgentVersions {
    const fn check(&self, kind: AgentKind) -> Option<&Check> {
        match kind {
            AgentKind::Claude => Some(&self.claude),
            AgentKind::Codex => Some(&self.codex),
            AgentKind::Gemini | AgentKind::OpenCode => None,
        }
    }

    const fn check_mut(&mut self, kind: AgentKind) -> Option<&mut Check> {
        match kind {
            AgentKind::Claude => Some(&mut self.claude),
            AgentKind::Codex => Some(&mut self.codex),
            AgentKind::Gemini | AgentKind::OpenCode => None,
        }
    }
}

/// Starts both checks, and redraws the bars when the allowed rows change.
pub fn init(cx: &mut App) {
    cx.set_global(AgentVersions {
        allowed: allowed_ids(cx),
        ..AgentVersions::default()
    });
    check(AgentKind::Claude, cx);
    check(AgentKind::Codex, cx);
    cx.observe_global::<SettingsStore>(|cx| {
        let allowed = allowed_ids(cx);
        if cx.global::<AgentVersions>().allowed != allowed {
            cx.global_mut::<AgentVersions>().allowed = allowed;
            cx.refresh_windows();
        }
    })
    .detach();
}

/// The integrations `marley.allow_untested_versions` turns on, by id.
pub(crate) fn allowed_in(marley: Option<&settings::MarleySettingsContent>) -> BTreeSet<String> {
    marley
        .and_then(|marley| marley.allow_untested_versions.as_ref())
        .map(|allowed| {
            allowed
                .iter()
                .filter(|(_, on)| **on)
                .map(|(id, _)| id.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn allowed_ids(cx: &App) -> Vec<String> {
    MarleySettings::get_global(cx)
        .allow_untested_versions
        .iter()
        .cloned()
        .collect()
}

/// The variable that names `kind`'s program, ahead of the search path.
const fn variable(kind: AgentKind) -> &'static str {
    match kind {
        AgentKind::Codex => "MARLEY_CODEX",
        _ => "MARLEY_CLAUDE",
    }
}

/// Finds `kind`'s program and, when its file changed since `previous`, reads its version, off the
/// main thread; the answer is stored, logged, and redraws the bars when a verdict moved.
fn check(kind: AgentKind, cx: &mut App) {
    let Some(state) = cx
        .global_mut::<AgentVersions>()
        .check_mut(kind)
        .filter(|state| !state.running)
    else {
        return;
    };
    state.running = true;
    let previous = state.identity.clone();
    let named = std::env::var_os(variable(kind)).map(PathBuf::from);
    let search_path = crate::agents::launcher(cx).search_path;
    let executor = cx.background_executor().clone();
    cx.spawn(async move |cx| {
        let located = cx
            .background_spawn(futures::future::lazy({
                let search_path = search_path.clone();
                move |_| locate(kind, named, search_path.as_ref())
            }))
            .await;
        let read = match located {
            Err(missing) => Some((None, None, Found::Missing(missing))),
            // The same file: its last reading stands.
            Ok((_, identity)) if previous.as_ref() == Some(&identity) => None,
            Ok((path, identity)) => {
                let found = read_version(&path, search_path, &executor).await;
                Some((Some(path), Some(identity), found))
            }
        };
        cx.update(|cx| store(kind, read, cx));
    })
    .detach();
}

/// `kind`'s program: the variable's, else the search path's first, and its identity.
fn locate(
    kind: AgentKind,
    named: Option<PathBuf>,
    search_path: Option<&OsString>,
) -> Result<(PathBuf, Identity), String> {
    let program = kind.program();
    let path = match named {
        Some(named) if named.is_file() => named,
        Some(named) => {
            return Err(format!(
                "{} names {}, which is not there",
                variable(kind),
                named.display()
            ));
        }
        None => which::which_in(program, search_path, "/").map_err(|_| {
            format!(
                "No {program} on Marley's PATH, and {} is unset",
                variable(kind)
            )
        })?,
    };
    let canonical = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
    let metadata = std::fs::metadata(&canonical)
        .map_err(|error| format!("{} cannot be read: {error}", path.display()))?;
    let identity = Identity {
        path: canonical,
        size: metadata.len(),
        modified: metadata.modified().ok(),
    };
    Ok((path, identity))
}

/// What `path --version` prints, read within the time allowed, with the program's own folder first
/// on `PATH` so a launcher written for an interpreter finds it.
async fn read_version(
    path: &Path,
    search_path: Option<OsString>,
    executor: &gpui::BackgroundExecutor,
) -> Found {
    let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let mut paths = vec![folder];
    paths.extend(search_path.iter().flat_map(std::env::split_paths));
    let joined = std::env::join_paths(paths).unwrap_or_default();
    let joined = joined.to_string_lossy().into_owned();
    let env = [("PATH", joined.as_str())];
    let run = crate::process::output(path, ["--version"], None, &env);
    let timer = executor.timer(VERSION_TIMEOUT);
    let output = match futures::future::select(Box::pin(run), timer).await {
        futures::future::Either::Left((output, _)) => output,
        futures::future::Either::Right(_) => {
            return Found::Unreadable(format!(
                "did not answer within {} s",
                VERSION_TIMEOUT.as_secs()
            ));
        }
    };
    let output = match output {
        Ok(output) => output,
        Err(error) => return Found::Unreadable(format!("did not run: {error}")),
    };
    let printed = |bytes: &[u8]| {
        let cut = bytes.get(..VERSION_OUTPUT).unwrap_or(bytes);
        String::from_utf8_lossy(cut).into_owned()
    };
    let (stdout, stderr) = (printed(&output.stdout), printed(&output.stderr));
    // Most print the version on stdout; a program that prints nothing there may on stderr.
    match versions::parse_version(&stdout) {
        Ok(version) => Found::Version(version),
        Err(why) => match versions::parse_version(&stderr) {
            Ok(version) => Found::Version(version),
            Err(other) if stdout.trim().is_empty() => Found::Unreadable(other),
            Err(_) => Found::Unreadable(why),
        },
    }
}

/// Stores a check's outcome (`None`: the file had not changed), logs it, and redraws the bars when
/// an integration's verdict moved.
fn store(kind: AgentKind, read: Option<(Option<PathBuf>, Option<Identity>, Found)>, cx: &mut App) {
    let before = verdicts(kind, cx);
    let Some(state) = cx.global_mut::<AgentVersions>().check_mut(kind) else {
        return;
    };
    state.running = false;
    state.checked_at = Some(Instant::now());
    let Some((path, identity, found)) = read else {
        return;
    };
    // Logged when it changes, so a program missing for a while is one line, not one a check.
    let shown = path.as_ref().map(|path| path.display().to_string());
    let changed = state.found.as_ref() != Some(&found) || state.path != path;
    match (&found, &shown) {
        _ if !changed => {}
        (Found::Version(version), Some(path)) => {
            log::info!(
                "agent versions: {} {version} at {path}",
                kind.display_name()
            );
        }
        (Found::Unreadable(why), Some(path)) => {
            log::info!("agent versions: {} at {path} {why}", kind.display_name());
        }
        (Found::Missing(looked), _) => {
            log::info!("agent versions: {}: {looked}", kind.display_name());
        }
        _ => {}
    }
    state.path = path;
    state.identity = identity;
    state.found = Some(found);
    if verdicts(kind, cx) != before {
        cx.refresh_windows();
    }
}

/// A check for `kind` when none runs and the last ended [`FRESH`] ago or more.
fn check_if_stale(kind: AgentKind, cx: &mut App) {
    let stale = cx
        .try_global::<AgentVersions>()
        .and_then(|versions| versions.check(kind))
        .is_some_and(|state| {
            !state.running && state.checked_at.is_none_or(|at| at.elapsed() >= FRESH)
        });
    if stale {
        check(kind, cx);
    }
}

fn verdicts(kind: AgentKind, cx: &App) -> Vec<Verdict> {
    integrations_of(kind)
        .map(|integration| verdict(integration, cx))
        .collect()
}

/// `integration`'s verdict from the check and the user's setting.
fn verdict(integration: &Integration, cx: &App) -> Verdict {
    let found = cx
        .try_global::<AgentVersions>()
        .and_then(|versions| versions.check(integration.agent))
        .and_then(|state| state.found.as_ref());
    let allowed = MarleySettings::get_global(cx)
        .allow_untested_versions
        .contains(integration.id);
    versions::verdict(integration, found, allowed)
}

/// Whether `integration` runs now: its agent's version tested, or the user allowed it (#650).
pub(crate) fn is_on(integration: &Integration, cx: &App) -> bool {
    verdict(integration, cx).is_on()
}

/// What the last check found for `kind`, if one has ended (#653).
pub(crate) fn found(kind: AgentKind, cx: &App) -> Option<Found> {
    cx.try_global::<AgentVersions>()?.check(kind)?.found.clone()
}

/// The program the last check found for `kind`: `MARLEY_CLAUDE` or `MARLEY_CODEX`, else the
/// search path's, as found (#650).
pub(crate) fn program(kind: AgentKind, cx: &App) -> Option<PathBuf> {
    cx.try_global::<AgentVersions>()?.check(kind)?.path.clone()
}

/// Calls `changed` after each check and each change of the allowed rows, so a feature that
/// follows a verdict sees it move (#653).
pub(crate) fn observe(cx: &mut App, changed: impl FnMut(&mut App) + 'static) -> Subscription {
    cx.observe_global::<AgentVersions>(changed)
}

/// Whether the user asked for `integration`, so its being off is worth a chip: Codex's App Server
/// only while `marley.codex_app_server` is on, Claude Code's IDE link only while
/// `marley.claude_code_ide` is (#653).
fn wanted(integration: &Integration, cx: &App) -> bool {
    let settings = MarleySettings::get_global(cx);
    if integration.id == CODEX_APP_SERVER.id {
        settings.codex_app_server == crate::codex_server::CodexAppServer::On
    } else if crate::claude_ide::ROWS
        .iter()
        .any(|row| row.id == integration.id)
    {
        settings.claude_code_ide == crate::claude_ide::ClaudeCodeIde::On
    } else {
        true
    }
}

/// How a terminal's Claude Code prompts are read: with the tags while they are on, or allowed,
/// else every prompt as the user's. A remote terminal's are read with the tags: the local
/// `claude` says nothing of the host's (D10).
pub(crate) fn prompt_reading(terminal: &Terminal, cx: &App) -> PromptReading {
    if crate::remote::is_remote(terminal) || verdict(&CLAUDE_PROMPT_TAGS, cx).is_on() {
        PromptReading::Recognized
    } else {
        PromptReading::AllTyped
    }
}

/// The chip under a local terminal whose foreground agent has an integration off, saying why;
/// none while the first check runs. A click opens the setting of the first one off.
pub(crate) fn chip(
    kind: AgentKind,
    context: &MarleyFooterContext,
    cx: &mut App,
) -> Option<AnyElement> {
    if !matches!(kind, AgentKind::Claude | AgentKind::Codex) {
        return None;
    }
    check_if_stale(kind, cx);
    if crate::remote::is_remote(context.terminal.read(cx)) {
        return None;
    }
    let verdicts: Vec<(&'static Integration, Off)> = integrations_of(kind)
        .filter(|integration| wanted(integration, cx))
        .filter_map(|integration| match verdict(integration, cx) {
            Verdict::Off(Off::NotChecked) | Verdict::On | Verdict::Allowed => None,
            Verdict::Off(off) => Some((integration, off)),
        })
        .collect();
    let (first, first_off) = verdicts.first()?;
    let path = cx
        .try_global::<AgentVersions>()
        .and_then(|versions| versions.check(kind))
        .and_then(|state| state.path.as_ref())
        .map(|path| path.display().to_string());
    let offs: Vec<(&Integration, &Off)> = verdicts
        .iter()
        .map(|(integration, off)| (*integration, off))
        .collect();
    let tooltip = versions::reasons(kind, path.as_deref(), &offs).join("\n");
    let setting = format!("marley.allow_untested_versions.{}", first.id);
    Some(
        div()
            .debug_selector(|| "marley-agent-version-chip".into())
            .child(
                Button::new(
                    "marley-agent-version-chip",
                    versions::chip_label(kind, first_off),
                )
                .start_icon(
                    Icon::new(IconName::Warning)
                        .size(IconSize::XSmall)
                        .color(Color::Warning),
                )
                .label_size(LabelSize::Small)
                .tooltip(Tooltip::text(tooltip))
                .on_click(move |_, window, cx| {
                    window.dispatch_action(
                        Box::new(zed_actions::OpenSettingsAt {
                            path: setting.clone(),
                            target: None,
                        }),
                        cx,
                    );
                }),
            )
            .into_any_element(),
    )
}
