//! Playwright scripts the user keeps in Marley and runs on a Browser tab (#523).
//!
//! A script is an ES module in Marley's config folder: `playwright/projects/<key>/` for one
//! project, whose key is its browser's (#507), so a project's worktrees share its scripts, and
//! `playwright/global/` for every project. Its default export gets the tab's `page`, its
//! `context` and the `browser`. A run is one command typed into a new terminal beside the tab:
//! Marley's runner, written into its data folder, attaches Marley's own `playwright-core` to the
//! tab's Chromium over CDP, finds the tab's page by its target id and hands it to the script. The
//! command's block shows the run's output and exit code, and a failed run saves the tab's last
//! minute as a recording. Runs are the user's act: no agent tool starts one (AD-claude-492).

use std::borrow::Cow;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context as _;
use gpui::{
    AnyWindowHandle, App, AppContext as _, AsyncApp, Entity, Global, SharedString, WeakEntity,
    Window,
};
use marley_browser::service;
use marley_terminal::BlockState;
use terminal::Terminal;
use terminal_view::TerminalView;
use terminal_view::terminal_panel::TerminalPanel;
use util::shell::ShellKind;
use workspace::notifications::NotificationId;
use workspace::{Pane, SplitDirection, Toast, Workspace};

use crate::PlaywrightScripts;
use crate::agents;
use crate::browser::{BrowserHub, recordings_dir};

/// The runner Marley writes into its data folder before each run.
const RUNNER: &str = include_str!("../playwright/run.mjs");

/// What a new script starts as.
const TEMPLATE: &str = include_str!("../playwright/template.mjs");

/// The runner's package: Marley's own Playwright, pinned, which brings no browser.
const PACKAGE: &str =
    "{\n  \"private\": true,\n  \"dependencies\": { \"playwright-core\": \"1.63.0\" }\n}\n";

/// How long a new terminal's shell gets to say it is ready, as `agents::start_cli` waits.
const STARTUP_WAIT: Duration = Duration::from_secs(5);

/// How often a run's terminal is looked at for its block's end.
const WATCH_POLL: Duration = Duration::from_millis(250);

/// How long a run waits for its command's block to open: a shell without Marley's integration
/// opens none.
const BLOCK_WAIT: Duration = Duration::from_secs(10);

/// Where a script is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// For one project.
    Project,
    /// For every project.
    Global,
}

impl Scope {
    /// How the tray names it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Project => "this project",
            Self::Global => "all projects",
        }
    }
}

/// A saved script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    /// Its name: its file's without `.mjs`.
    pub name: String,
    /// Where it is kept.
    pub scope: Scope,
    /// Its file.
    pub path: PathBuf,
}

/// The folder of the scripts kept in `scope` for the project `key`, in the config folder
/// `config`.
#[must_use]
pub fn library_dir_in(config: &Path, scope: Scope, key: &str) -> PathBuf {
    let library = config.join("playwright");
    match scope {
        Scope::Project => library.join("projects").join(key),
        Scope::Global => library.join("global"),
    }
}

/// The scripts of the project `key` in the config folder `config`: its own, then every
/// project's, each sorted by name.
///
/// # Errors
///
/// When a library folder is there and cannot be read.
pub fn library_in(config: &Path, key: &str) -> std::io::Result<Vec<Script>> {
    let mut scripts = Vec::new();
    for scope in [Scope::Project, Scope::Global] {
        let dir = library_dir_in(config, scope, key);
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        let mut found = Vec::new();
        for entry in entries {
            let path = entry?.path();
            if path.extension().is_some_and(|extension| extension == "mjs")
                && let Some(name) = path.file_stem().and_then(|stem| stem.to_str())
            {
                found.push(Script {
                    name: name.to_string(),
                    scope,
                    path: path.clone(),
                });
            }
        }
        found.sort_by(|left, right| left.name.cmp(&right.name));
        scripts.extend(found);
    }
    Ok(scripts)
}

/// The file stem for a script the user named `name`: its letters, digits, `-` and `_`, each
/// other run of characters one `-`; none when nothing is left.
#[must_use]
pub fn file_stem(name: &str) -> Option<String> {
    let mut stem = String::new();
    for character in name.trim().chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            stem.push(character);
        } else if !stem.is_empty() && !stem.ends_with('-') {
            stem.push('-');
        }
    }
    let stem = stem.trim_end_matches('-').to_string();
    (!stem.is_empty()).then_some(stem)
}

/// Makes the script `name` in `scope` for the project `key` from Marley's template, and answers
/// its file.
///
/// # Errors
///
/// When the name leaves nothing a file can be named, a script of that name exists, or the file
/// cannot be written.
pub fn create_in(config: &Path, scope: Scope, key: &str, name: &str) -> anyhow::Result<PathBuf> {
    let stem = file_stem(name).context("name the script with letters or digits")?;
    let dir = library_dir_in(config, scope, key);
    std::fs::create_dir_all(&dir).with_context(|| format!("making {}", dir.display()))?;
    let path = dir.join(format!("{stem}.mjs"));
    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            anyhow::bail!(
                "a script named {stem} is kept for {} already",
                scope.label()
            )
        }
        Err(error) => return Err(error).context(format!("writing {}", path.display())),
    };
    std::io::Write::write_all(&mut file, TEMPLATE.as_bytes())
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// Writes Marley's runner and its package into `<data>/playwright`, and answers the runner.
///
/// # Errors
///
/// When the folder cannot be made or a file written.
pub fn write_runner_in(data: &Path) -> std::io::Result<PathBuf> {
    let dir = data.join("playwright");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("package.json"), PACKAGE)?;
    let runner = dir.join("run.mjs");
    std::fs::write(&runner, RUNNER)?;
    Ok(runner)
}

/// Whether Marley's Playwright is installed in `<data>/playwright`.
#[must_use]
pub fn installed_in(data: &Path) -> bool {
    data.join("playwright")
        .join("node_modules")
        .join("playwright-core")
        .join("package.json")
        .is_file()
}

/// The command a run types, or none when a path cannot be quoted for the shell.
///
/// It runs the runner with the file that holds the tab's relay's endpoint and token (#583) and
/// the tab's page in its environment, after an install into `install` when Marley's Playwright is
/// not there yet. The command names the file, never the token, which would stay in the block and
/// in the shell's history.
#[must_use]
pub fn command(
    endpoint_file: &Path,
    tab: &str,
    runner: &Path,
    script: &Path,
    install: Option<&Path>,
) -> Option<String> {
    let quote = |text: &str| ShellKind::Posix.try_quote(text).map(Cow::into_owned);
    let run = format!(
        "MARLEY_CDP_FILE={} MARLEY_TAB={} node {} {}",
        quote(endpoint_file.to_str()?)?,
        quote(tab)?,
        quote(runner.to_str()?)?,
        quote(script.to_str()?)?
    );
    Some(match install {
        Some(prefix) => format!(
            "npm install --prefix {} --no-audit --no-fund && {run}",
            quote(prefix.to_str()?)?
        ),
        None => run,
    })
}

/// The tab a script runs on (#523).
pub(crate) struct RunTarget {
    /// The hub, which keeps the page's minute.
    pub hub: Entity<BrowserHub>,
    /// The key of the tab's project, whose Chromium the run attaches to.
    pub project: SharedString,
    /// The tab's page.
    pub tab: String,
    /// The tab's pane, beside which the run's terminal opens.
    pub pane: WeakEntity<Pane>,
    /// The tab's workspace.
    pub workspace: WeakEntity<Workspace>,
    /// The folder a recording names as its project.
    pub root: Option<String>,
}

/// Whether a run installs Marley's Playwright, which a second run waits out.
#[derive(Default)]
struct Installing(bool);

impl Global for Installing {}

/// Runs `script` on the tab `target` names; what went wrong reaches a toast.
pub(crate) fn run(target: RunTarget, script: Script, window: &Window, cx: &App) {
    let window = window.window_handle();
    cx.spawn(async move |cx| {
        if let Err(error) = run_on(&target, &script, window, cx).await {
            toast(
                &target.workspace,
                format!("Could not run {}: {error:#}", script.name),
                cx,
            );
        }
    })
    .detach();
}

async fn run_on(
    target: &RunTarget,
    script: &Script,
    window: AnyWindowHandle,
    cx: &mut AsyncApp,
) -> anyhow::Result<()> {
    let data = paths::data_dir().clone();
    let profile = service::profile_in(&service::project_dir_in(&data, &target.project));
    let (endpoint, runner, installed) = cx
        .background_spawn(futures::future::lazy({
            let data = data.clone();
            move |_| -> anyhow::Result<_> {
                let endpoint = service::relay_endpoint_file_in(&profile);
                anyhow::ensure!(endpoint.is_file(), "the project's browser is not running");
                let runner = write_runner_in(&data).context("writing Marley's runner")?;
                Ok((endpoint, runner, installed_in(&data)))
            }
        }))
        .await?;
    let install = (!installed).then(|| data.join("playwright"));
    if install.is_some() {
        let busy =
            cx.update(|cx| std::mem::replace(&mut cx.default_global::<Installing>().0, true));
        anyhow::ensure!(
            !busy,
            "Marley is installing its Playwright for another run; run this one once that ends"
        );
    }
    let ran = run_command(
        target,
        script,
        &endpoint,
        &runner,
        install.as_deref(),
        window,
        cx,
    )
    .await;
    if install.is_some() {
        cx.update(|cx| cx.default_global::<Installing>().0 = false);
    }
    ran
}

async fn run_command(
    target: &RunTarget,
    script: &Script,
    endpoint_file: &Path,
    runner: &Path,
    install: Option<&Path>,
    window: AnyWindowHandle,
    cx: &mut AsyncApp,
) -> anyhow::Result<()> {
    let line = command(endpoint_file, &target.tab, runner, &script.path, install)
        .context("a path Marley cannot quote for the shell")?;
    let terminal = open_terminal(target, window, cx).await?;
    let startup = terminal.update(cx, |terminal, _| {
        terminal.start_init_command_startup_handshake()
    })?;
    // A shell that never echoes the handshake's marker is written to after the wait.
    let timeout = cx.background_executor().timer(STARTUP_WAIT);
    futures::future::select(startup, timeout).await;
    let written = terminal.update(cx, |terminal, cx| {
        terminal.write_init_command_after_startup(marley_agent::send_payload(&line), cx)
    })?;
    anyhow::ensure!(
        written,
        "the terminal took other input before the run started"
    );
    target.hub.update(cx, |hub, _| {
        hub.record_script(&target.tab, script.name.clone(), None);
    });
    match block_end(&terminal, &line, cx).await {
        Some(Some(0)) => {
            target.hub.update(cx, |hub, _| {
                hub.record_script(&target.tab, script.name.clone(), Some(0));
            });
        }
        Some(Some(code)) => {
            target.hub.update(cx, |hub, _| {
                hub.record_script(&target.tab, script.name.clone(), Some(code));
            });
            let saved = target.hub.update(cx, |hub, cx| {
                hub.record(&target.tab, recordings_dir(), target.root.clone(), cx)
            });
            let message = match saved.await {
                Ok(id) => format!(
                    "{} failed with exit code {code}; the tab's last minute is recording {id}.",
                    script.name
                ),
                Err(error) => format!(
                    "{} failed with exit code {code}, and its minute could not be saved: \
                     {error:#}",
                    script.name
                ),
            };
            toast(&target.workspace, message, cx);
        }
        Some(None) => toast(
            &target.workspace,
            format!(
                "{} ended, and its shell did not say how: Marley saved no recording.",
                script.name
            ),
            cx,
        ),
        None => {}
    }
    Ok(())
}

/// Opens a terminal of the tab's workspace where New Terminal would. When it went into the tab's
/// pane, it moves into the pane right of the tab's, split off for it when there is none, which
/// leaves the tab in front: runs after the first reuse that pane.
async fn open_terminal(
    target: &RunTarget,
    window: AnyWindowHandle,
    cx: &mut AsyncApp,
) -> anyhow::Result<WeakEntity<Terminal>> {
    let workspace = target
        .workspace
        .upgrade()
        .context("the tab's workspace closed")?;
    let opened = window.update(cx, |_, window, cx| {
        let factory = agents::launcher(cx).terminal_factory;
        workspace.update(cx, |workspace, cx| {
            let directory = terminal_view::default_working_directory(workspace, cx);
            TerminalPanel::add_center_terminal(workspace, window, cx, move |project, cx| {
                factory(project, directory, collections::HashMap::default(), cx)
            })
        })
    })?;
    let terminal = opened.await?;
    window.update(cx, |_, window, cx| {
        let Some(pane) = target.pane.upgrade() else {
            return;
        };
        let in_front = pane
            .read(cx)
            .active_item()
            .and_then(|item| item.downcast::<TerminalView>())
            .is_some_and(|view| view.read(cx).terminal().downgrade() == terminal);
        if in_front {
            workspace.update(cx, |workspace, cx| {
                match workspace.find_pane_in_direction(SplitDirection::Right, cx) {
                    Some(right) if right != pane => {
                        workspace::move_active_item(&pane, &right, true, false, window, cx);
                    }
                    _ => workspace.split_and_move(pane, SplitDirection::Right, window, cx),
                }
            });
        }
    })?;
    Ok(terminal)
}

/// How the block of the command `line` in `terminal` ended: its exit code, none inside when the
/// shell did not say; none at all when the terminal closed first. The command's block is found by
/// its command, since a new terminal's startup can open a block of its own before it.
async fn block_end(
    terminal: &WeakEntity<Terminal>,
    line: &str,
    cx: &AsyncApp,
) -> Option<Option<i32>> {
    let mut waited = Duration::ZERO;
    loop {
        let run = terminal
            .read_with(cx, |terminal, _| {
                terminal
                    .blocks()
                    .iter()
                    .rev()
                    .find(|block| block.command.trim() == line.trim())
                    .map(|block| (block.state, block.exit_code.0))
            })
            .ok()?;
        match run {
            Some((BlockState::Finished, code)) => return Some(code),
            Some(_) => {}
            None if waited >= BLOCK_WAIT => return Some(None),
            None => waited += WATCH_POLL,
        }
        cx.background_executor().timer(WATCH_POLL).await;
    }
}

/// Shows `message` as a toast in `workspace`.
fn toast(workspace: &WeakEntity<Workspace>, message: String, cx: &mut AsyncApp) {
    workspace
        .update(cx, |workspace, cx| {
            workspace.show_toast(
                Toast::new(NotificationId::unique::<PlaywrightScripts>(), message),
                cx,
            );
        })
        .ok();
}
