//! Starting agents: the agents a project can start, the two ways to start one, and the New Agent
//! picker that `marley::NewAgent` opens.
//!
//! A Zed agent starts as a thread in the project's Agent Panel; an agent CLI starts in a new
//! center terminal. The rail's `+` menu and the picker share everything here.

use std::ffi::{OsStr, OsString};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use agent_ui::{Agent, AgentPanel, NewExternalAgentThread};
use anyhow::Context as _;
use askpass::{AskPassDelegate, EncryptedPassword, PasswordProxy};
use collections::HashMap;
use futures::channel::oneshot;
use fuzzy::{StringMatch, StringMatchCandidate};
use git_ui_core::askpass_modal::AskPassModal;
use gpui::{
    Action, AnyWindowHandle, App, AsyncApp, AsyncWindowContext, Context, DismissEvent, Entity,
    EntityId, EventEmitter, FocusHandle, Focusable, Global, PathPromptOptions, Render, Task,
    WeakEntity, Window,
};
use marley_agent::{AgentKind, LaunchMode};
use picker::{Picker, PickerDelegate};
use project::{AgentId, AgentRegistryStore, DirectoryLister, DisableAiSettings, Project};
use schemars::JsonSchema;
use serde::Deserialize;
use settings::{ClaudeCodePermissions, CodexPermissions, MarleySettingsContent, Settings as _};
use terminal::Terminal;
use terminal_view::TerminalView;
use terminal_view::terminal_panel::TerminalPanel;
use ui::{HighlightedLabel, IconName, ListItem, ListItemSpacing, prelude::*};
use util::ResultExt as _;
use workspace::notifications::DetachAndPromptErr as _;
use workspace::{ModalView, MultiWorkspace, SerializedWorkspaceLocation, Workspace, WorkspaceDb};

/// Makes the `Terminal` behind a new center terminal, started in the given directory with the
/// given variables of its own.
///
/// Production uses `Project::create_terminal_shell_with_env` itself, so no line of this crate
/// spawns a shell; tests hand in a display-only terminal.
pub type TerminalFactory = fn(
    &mut Project,
    Option<PathBuf>,
    HashMap<String, String>,
    &mut Context<Project>,
) -> Task<anyhow::Result<Entity<Terminal>>>;

/// How long an agent's launch waits for the shell to say it is ready, as Zed's terminal threads
/// wait, before writing the command anyway.
pub(crate) const STARTUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Where agent CLIs are looked for, and how the terminal one starts in is made. A test sets its
/// own, so that no real agent starts.
#[derive(Clone, Debug)]
pub struct Launcher {
    /// The directories searched for agent CLIs.
    pub search_path: Option<OsString>,
    /// Makes the terminal that New Terminal or an agent CLI starts in.
    pub terminal_factory: TerminalFactory,
    /// Whether an agent's terminal in a local project asks for ssh's passphrases in Marley
    /// (#596), whose socket a test does without.
    pub passphrase_dialog: PassphraseDialog,
}

/// Whether [`Launcher`] opens ssh's passphrase socket for an agent's terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassphraseDialog {
    /// ssh asks in Marley's dialog.
    Shown,
    /// ssh asks on the terminal.
    Skipped,
}

impl Default for Launcher {
    fn default() -> Self {
        Self {
            search_path: std::env::var_os("PATH"),
            terminal_factory: Project::create_terminal_shell_with_env,
            passphrase_dialog: PassphraseDialog::Shown,
        }
    }
}

impl Global for Launcher {}

/// The launcher a test set, or else the process's `PATH` and real shells.
#[must_use]
pub fn launcher(cx: &App) -> Launcher {
    cx.try_global::<Launcher>().cloned().unwrap_or_default()
}

/// An agent's icon: one of Zed's, or an agent server's own SVG.
#[derive(Clone, Debug)]
pub enum AgentIcon {
    /// One of Zed's icons.
    Named(IconName),
    /// The path of an agent server's SVG.
    Svg(SharedString),
}

/// The known agent CLIs `search_path` holds an executable for, in menu order.
#[must_use]
pub fn installed_clis(search_path: Option<&OsStr>) -> Vec<AgentKind> {
    AgentKind::ALL
        .into_iter()
        .filter(|kind| which::which_in(kind.program(), search_path, "/").is_ok())
        .collect()
}

/// The icon an agent CLI's row and menu entry draw.
#[must_use]
pub const fn cli_icon(kind: AgentKind) -> IconName {
    match kind {
        AgentKind::Claude => IconName::AiClaude,
        AgentKind::Codex => IconName::AiOpenAi,
        AgentKind::Gemini => IconName::AiGemini,
        AgentKind::OpenCode => IconName::AiOpenCode,
    }
}

/// The icon a thread of `agent` draws: the Zed Agent's, an agent server's own, the registry's,
/// or a generic one.
pub fn thread_icon(agent: &AgentId, project: &Entity<Project>, cx: &App) -> AgentIcon {
    if Agent::from(agent.clone()) == Agent::NativeAgent {
        return AgentIcon::Named(IconName::ZedAgent);
    }
    project
        .read(cx)
        .agent_server_store()
        .read(cx)
        .agent_icon(agent)
        .or_else(|| {
            AgentRegistryStore::try_global(cx)
                .and_then(|registry| registry.read(cx).agent(agent)?.icon_path().cloned())
        })
        .map_or(AgentIcon::Named(IconName::Sparkle), AgentIcon::Svg)
}

/// The name a thread of `agent` shows: the Zed Agent's, an agent server's display name, the
/// registry's, or the agent's id.
pub fn thread_agent_name(agent: &AgentId, project: &Entity<Project>, cx: &App) -> SharedString {
    if Agent::from(agent.clone()) == Agent::NativeAgent {
        return SharedString::new_static("Zed Agent");
    }
    let servers = project.read(cx).agent_server_store().read(cx);
    let registry = AgentRegistryStore::try_global(cx);
    let registry = registry.as_ref().map(|registry| registry.read(cx));
    servers
        .agent_display_name(agent)
        .or_else(|| Some(registry?.agent(agent)?.name().clone()))
        .unwrap_or_else(|| agent.0.clone())
}

/// The agents a new thread can run: the Zed Agent first, then the project's agent servers sorted
/// by display name without regard to case, each with its name and icon.
pub fn thread_agents(
    project: &Entity<Project>,
    cx: &App,
) -> Vec<(AgentId, SharedString, AgentIcon)> {
    let servers = project.read(cx).agent_server_store().read(cx);
    let mut external: Vec<(AgentId, SharedString)> = servers
        .external_agents()
        .map(|agent| (agent.clone(), thread_agent_name(agent, project, cx)))
        .collect();
    external.sort_by_key(|(_, name)| name.to_lowercase());
    let zed_agent = Agent::NativeAgent.id();
    let zed_name = thread_agent_name(&zed_agent, project, cx);
    std::iter::once((zed_agent, zed_name))
        .chain(external)
        .map(|(agent, name)| {
            let icon = thread_icon(&agent, project, cx);
            (agent, name, icon)
        })
        .collect()
}

/// Starts a thread of `agent` in `workspace`'s Agent Panel, focused. The panel is called
/// directly: a dispatched action would reach whichever workspace the window shows.
///
/// # Errors
///
/// When the workspace has no Agent Panel.
pub fn start_thread(
    workspace: &mut Workspace,
    agent: &AgentId,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> anyhow::Result<()> {
    // The action's one field is private, so it is built the way a keymap builds it.
    let action: NewExternalAgentThread =
        serde_json::from_value(serde_json::json!({ "agent": agent.0 }))?;
    let panel = workspace
        .panel::<AgentPanel>(cx)
        .context("the project has no Agent Panel")?;
    panel.update(cx, |panel, cx| {
        panel.new_external_agent_thread(&action, window, cx);
    });
    workspace.focus_panel::<AgentPanel>(window, cx);
    Ok(())
}

/// What Marley starts Claude Code and Codex with (#532), from the `marley` settings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentPermissions {
    claude_code: ClaudeCodePermissions,
    codex: CodexPermissions,
    /// The per-project entries, their folders with `~/` expanded.
    by_project: Vec<ProjectPermissions>,
}

/// One project's entry of `agent_permissions_by_project`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectPermissions {
    folder: PathBuf,
    claude_code: Option<ClaudeCodePermissions>,
    codex: Option<CodexPermissions>,
}

impl AgentPermissions {
    /// The permissions `content` sets, and the defaults where it sets none.
    #[must_use]
    pub fn from_content(content: Option<&MarleySettingsContent>) -> Self {
        Self {
            claude_code: content
                .and_then(|content| content.claude_code_permissions)
                .unwrap_or_default(),
            codex: content
                .and_then(|content| content.codex_permissions)
                .unwrap_or_default(),
            by_project: content
                .and_then(|content| content.agent_permissions_by_project.as_ref())
                .into_iter()
                .flatten()
                .filter_map(|(folder, permissions)| {
                    Some(ProjectPermissions {
                        folder: crate::system_one::folder_path(folder)?,
                        claude_code: permissions.claude_code,
                        codex: permissions.codex,
                    })
                })
                .collect(),
        }
    }

    /// How `kind` starts in a local project whose main folders are `folders`: as the entry says
    /// whose folder holds one of them, the longest of those that sets `kind`, else as the default
    /// says.
    #[must_use]
    pub fn launch_mode(&self, kind: AgentKind, folders: &[PathBuf]) -> LaunchMode {
        let bypasses = |entry: &ProjectPermissions| match kind {
            AgentKind::Claude => entry
                .claude_code
                .map(|permissions| permissions == ClaudeCodePermissions::Bypass),
            AgentKind::Codex => entry
                .codex
                .map(|permissions| permissions == CodexPermissions::FullAccess),
            AgentKind::Gemini | AgentKind::OpenCode => None,
        };
        let by_project = self
            .by_project
            .iter()
            .filter(|entry| {
                folders
                    .iter()
                    .any(|folder| folder.starts_with(&entry.folder))
            })
            .filter_map(|entry| Some((entry.folder.components().count(), bypasses(entry)?)))
            .max_by_key(|(depth, _)| *depth)
            .map(|(_, bypass)| bypass);
        let bypass = by_project.unwrap_or(match kind {
            AgentKind::Claude => self.claude_code == ClaudeCodePermissions::Bypass,
            AgentKind::Codex => self.codex == CodexPermissions::FullAccess,
            AgentKind::Gemini | AgentKind::OpenCode => false,
        });
        if bypass {
            LaunchMode::Bypass
        } else {
            LaunchMode::Ask
        }
    }
}

/// Starts `kind` in a new center terminal of `workspace`, where New Terminal would start one.
///
/// The command goes in once the shell says it is ready, as Zed's terminal threads start theirs:
/// the agent's program name, with the arguments its permission setting asks for in this project
/// (#532). A remote project takes the defaults, since the per-project entries name local folders.
/// An error reaches a prompt.
pub fn start_cli(workspace: &Workspace, kind: AgentKind, window: &Window, cx: &Context<Workspace>) {
    start_cli_with_prompt(workspace, kind, "", None, window, cx).detach();
}

/// As [`start_cli`], with `prompt` as the agent's first prompt on its command line (#510), and
/// with `setup`, a command typed before the agent's that must succeed first (#585).
///
/// The task gives the agent's terminal once the command is written (#587), and none once its
/// error has reached a prompt.
pub fn start_cli_with_prompt(
    workspace: &Workspace,
    kind: AgentKind,
    prompt: &str,
    setup: Option<&str>,
    window: &Window,
    cx: &Context<Workspace>,
) -> Task<Option<WeakEntity<Terminal>>> {
    let directory = terminal_view::default_working_directory(workspace, cx);
    let (input, joining) = agent_line(
        workspace,
        kind,
        directory.as_deref(),
        setup.unwrap_or_default(),
        prompt,
        cx,
    );
    start_in_terminal(
        workspace,
        directory,
        Some(kind),
        Some(input),
        joining,
        window,
        cx,
    )
    .prompt_err("Could not start the agent", window, cx, |_, _, _| None)
}

/// The line that starts `kind` after `setup` with `prompt` in `directory` of `workspace`, and,
/// for a Codex that joins an App Server of its own, the server and the line without it (#650).
fn agent_line(
    workspace: &Workspace,
    kind: AgentKind,
    directory: Option<&Path>,
    setup: &str,
    prompt: &str,
    cx: &App,
) -> (Vec<u8>, Option<crate::codex_server::Joining>) {
    let mode = launch_mode(workspace, kind, cx);
    let plain = marley_agent::launch_line_after(setup, kind, mode, prompt, None);
    let prepared = (kind == AgentKind::Codex)
        .then(|| crate::codex_server::prepare(workspace.project().read(cx), directory, cx))
        .flatten();
    match prepared {
        Some(prepared) => {
            let line = marley_agent::launch_line_after(
                setup,
                kind,
                mode,
                prompt,
                Some(&prepared.remote()),
            );
            (line, Some(crate::codex_server::Joining { prepared, plain }))
        }
        None => (plain, None),
    }
}

/// The permission mode `kind` starts with in `workspace`'s project (#532); a remote project takes
/// the defaults, since the per-project entries name local folders.
pub(crate) fn launch_mode(workspace: &Workspace, kind: AgentKind, cx: &App) -> LaunchMode {
    let project = workspace.project().read(cx);
    let folders = if project.is_local() {
        project.project_group_key(cx).path_list().paths().to_vec()
    } else {
        Vec::new()
    };
    crate::MarleySettings::get_global(cx)
        .agent_permissions
        .launch_mode(kind, &folders)
}

/// What starts `kind` in a terminal of `workspace` in `directory`, as the rail's Agent CLIs
/// entries start it: its program with the arguments its permission setting asks for (#527), and
/// for a Codex that joins an App Server of its own, the server (#650).
pub(crate) fn launch_input(
    workspace: &Workspace,
    kind: AgentKind,
    directory: &Path,
    cx: &App,
) -> (Vec<u8>, Option<crate::codex_server::Joining>) {
    agent_line(workspace, kind, Some(directory), "", "", cx)
}

/// The variables of a terminal opened for an agent CLI: git's credential prompts off (#537), and,
/// with a passphrase proxy's `script`, ssh's prompts sent to it (#596).
fn agent_env(script: Option<&OsStr>) -> anyhow::Result<HashMap<String, String>> {
    let mut env: HashMap<String, String> = marley_agent::GIT_PROMPTS_OFF
        .iter()
        .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
        .collect();
    if let Some(script) = script {
        let script = script
            .to_str()
            .context("the askpass script's path is not UTF-8")?;
        env.insert("SSH_ASKPASS".to_string(), script.to_string());
        // `force` sends ssh to the script even where it could ask on the terminal, which the
        // agent owns.
        env.insert("SSH_ASKPASS_REQUIRE".to_string(), "force".to_string());
    }
    Ok(env)
}

/// Opens a center terminal of `workspace` in `directory`, and types `input` into it once its
/// shell says it is ready, as Zed's terminal threads start their commands (#527 split it out of
/// [`start_cli_with_prompt`]). A terminal opened for `agent` gets [`agent_env`]'s variables and,
/// in a local project, asks for ssh's passphrases in Marley (#596). With `joining`, the Codex
/// App Server the line names starts first, and the line without it goes in when it does not come
/// up (#650). The task gives the terminal once the input is written.
pub(crate) fn start_in_terminal(
    workspace: &Workspace,
    directory: Option<PathBuf>,
    agent: Option<AgentKind>,
    input: Option<Vec<u8>>,
    joining: Option<crate::codex_server::Joining>,
    window: &Window,
    cx: &Context<Workspace>,
) -> Task<anyhow::Result<WeakEntity<Terminal>>> {
    let Launcher {
        terminal_factory: factory,
        passphrase_dialog,
        ..
    } = launcher(cx);
    let (folders, local) = crate::system_one::project_of(workspace, cx);
    let shown = local && passphrase_dialog == PassphraseDialog::Shown;
    // Marley's editor for the agent's own editor key, in a local project only (#649).
    let editor = agent
        .filter(|_| local)
        .and_then(|_| crate::agent_editor::editor_path(cx));
    let project = workspace.project().clone();
    let dialog_title = agent.filter(|_| shown).map(|kind| {
        let project = crate::system_one::project_name(&folders);
        SharedString::from(marley_agent::ssh_dialog_title(&project, kind))
    });
    cx.spawn_in(window, async move |workspace, cx| {
        // The proxy exists before the terminal, whose shell takes its script's path; the
        // terminal is recorded for the dialog once it exists.
        let asker = Arc::new(OnceLock::new());
        let proxy = match dialog_title {
            Some(title) => passphrase_proxy(title, Arc::clone(&asker), cx).await?,
            None => None,
        };
        let script = proxy
            .as_ref()
            .map(|proxy| proxy.script_path().as_ref().to_os_string());
        let env = match agent {
            Some(_) => {
                let mut env = agent_env(script.as_deref())?;
                if let Some(editor) = &editor {
                    crate::agent_editor::add_env(&mut env, editor);
                }
                env
            }
            None => HashMap::default(),
        };
        // The server's commands run with the variables the agent's terminal gets (#650).
        let server_env: Vec<(String, String)> = joining
            .as_ref()
            .map(|_| {
                env.iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let terminal = workspace
            .update_in(cx, |workspace, window, cx| {
                TerminalPanel::add_center_terminal(workspace, window, cx, move |project, cx| {
                    factory(project, directory, env, cx)
                })
            })?
            .await?;
        if editor.is_some() {
            terminal.update(cx, |_, cx| {
                let id = cx.entity_id();
                crate::agent_editor::give(id, cx);
                cx.on_release(move |_, cx| crate::agent_editor::forget(id, cx))
                    .detach();
            })?;
        }
        if let Some(proxy) = proxy {
            asker.get_or_init(|| terminal.clone());
            // The socket and its folder last exactly as long as the terminal.
            terminal.update(cx, |_, cx| cx.on_release(move |_, _| drop(proxy)).detach())?;
        }
        let input = match joining {
            Some(joining) => {
                let started = crate::codex_server::start(
                    &joining.prepared,
                    &terminal,
                    &project,
                    server_env,
                    cx,
                )
                .await;
                if started { input } else { Some(joining.plain) }
            }
            None => input,
        };
        let Some(input) = input else {
            return Ok(terminal);
        };
        let handshake = |terminal: &mut Terminal, _: &mut Context<Terminal>| {
            terminal.start_init_command_startup_handshake()
        };
        let startup = terminal.update(cx, handshake)?;
        let timeout = cx.background_executor().timer(STARTUP_TIMEOUT);
        // A terminal without a PTY is ready at once; the timeout covers a shell that never
        // echoes the handshake's marker.
        futures::future::select(startup, timeout).await;
        let launch = |terminal: &mut Terminal, cx: &mut Context<Terminal>| {
            terminal.write_init_command_after_startup(input, cx)
        };
        let written = terminal.update(cx, launch)?;
        anyhow::ensure!(
            written,
            "the terminal took other input before its command started"
        );
        anyhow::Ok(terminal)
    })
}

/// Zed's askpass socket and script for one agent terminal (#596). ssh in the terminal runs the
/// script, which runs Marley's executable in its `--askpass` mode; each prompt opens Zed's
/// password dialog, headed `title`, over the terminal `asker` records, and the typed text goes
/// back to ssh alone.
///
/// None when the socket's path would not fit a Unix socket's address: the terminal's ssh then
/// asks on the terminal as before, and the log says why.
async fn passphrase_proxy(
    title: SharedString,
    asker: Arc<OnceLock<WeakEntity<Terminal>>>,
    cx: &mut AsyncWindowContext,
) -> anyhow::Result<Option<PasswordProxy>> {
    let window = cx.window_handle();
    let delegate =
        AskPassDelegate::new_with_cancellation(cx, move |prompt, answer, cancellation, cx| {
            ask_passphrase(
                window,
                &title,
                asker.get(),
                prompt,
                answer,
                cancellation,
                cx,
            );
        });
    let executor = cx.background_executor().clone();
    let ask = {
        let executor = executor.clone();
        move |prompt| {
            let answer = delegate.ask_password(prompt);
            executor.spawn(async move {
                // An unanswered dialog closes the connection without a word, so ssh reads an
                // empty passphrase and fails at once. Zed's own session holds the connection
                // until it kills the command, which Marley cannot do to a terminal's ssh.
                ControlFlow::Continue(
                    answer
                        .await
                        .context("the passphrase dialog closed unanswered"),
                )
            })
        }
    };
    let proxy = PasswordProxy::new(Box::new(ask), executor).await?;
    // `askpass.sock` is the name Zed's proxy binds beside its script, inside its own task, where
    // a failure only reaches the log.
    let socket = Path::new(proxy.script_path().as_ref()).with_file_name("askpass.sock");
    if !marley_browser::service::socket_fits(&socket) {
        log::warn!(
            "ssh in agent terminals asks on the terminal: {} is too long for a socket",
            socket.display()
        );
        return Ok(None);
    }
    Ok(Some(proxy))
}

/// Opens the password dialog for one of ssh's prompts over `asker`'s tab, in the workspace of
/// `window` that holds it. With no such terminal, or a password dialog already open there, the
/// answer is dropped and ssh reads an empty one: Zed's `toggle_modal` would close the open
/// dialog instead of showing a second, cancelling both.
fn ask_passphrase(
    window: AnyWindowHandle,
    title: &SharedString,
    asker: Option<&WeakEntity<Terminal>>,
    prompt: String,
    answer: oneshot::Sender<EncryptedPassword>,
    cancellation: oneshot::Receiver<()>,
    cx: &mut AsyncApp,
) {
    let Some(asker) = asker.map(|asker| asker.entity_id()) else {
        log::warn!("ssh asked for a passphrase before its terminal opened");
        return;
    };
    // Looked up before the window's update: `mcp::terminals` reads every window through its
    // handle, and the window being updated reads as gone.
    let found = cx.update(|cx| {
        crate::mcp::terminals(cx)
            .into_iter()
            .find(|(_, view)| view.read(cx).terminal().entity_id() == asker)
    });
    let Some((workspace, view)) = found else {
        log::warn!("ssh asked for a passphrase in a terminal that has closed");
        return;
    };
    let title = title.clone();
    window
        .update(cx, move |_, window, cx| {
            let shown_here =
                window
                    .root::<MultiWorkspace>()
                    .flatten()
                    .is_some_and(|multi_workspace| {
                        multi_workspace
                            .read(cx)
                            .workspaces()
                            .any(|shown| shown == &workspace)
                    });
            if !shown_here {
                log::warn!("ssh asked for a passphrase in a terminal of another window");
                return;
            }
            if workspace
                .read(cx)
                .active_modal::<AskPassModal>(cx)
                .is_some()
            {
                log::warn!("ssh asked for a passphrase while another dialog asks; refused");
                return;
            }
            crate::browser::reveal_terminal(&view, window, cx);
            workspace.update(cx, |workspace, cx| {
                workspace.toggle_modal(window, cx, |window, cx| {
                    AskPassModal::new(title, prompt.into(), answer, cancellation, window, cx)
                });
            });
        })
        .log_err();
}

/// Registers `marley::NewAgent` on every workspace. [`crate::init`] calls it once.
pub fn init(cx: &App) {
    fix_askpass_program();
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(new_agent);
    })
    .detach();
}

/// Fixes the program every askpass script runs to Marley's executable while the file is still
/// the one running (#596). Zed reads `current_exe` when it makes the first script, and once an
/// install has replaced the file Linux names the running one `… (deleted)`, which no script can
/// run (#583 met the same for the relay). Once per process: the tests call `init` again, and a
/// second set is a debug panic.
fn fix_askpass_program() {
    static FIXED: std::sync::Once = std::sync::Once::new();
    FIXED.call_once(|| match std::env::current_exe() {
        Ok(executable) => askpass::set_askpass_program(executable),
        Err(error) => log::warn!("askpass scripts will find Marley when they are made: {error}"),
    });
}

/// Opens the New Agent picker: Zed's agents and the installed agent CLIs, then where to start the
/// one chosen (#735).
///
/// With `folder`, the agent starts there and the picker asks only which agent, as Open Agent Here
/// on a terminal's row or in the project panel does.
#[derive(Clone, Debug, Default, PartialEq, Eq, JsonSchema, Action)]
#[action(namespace = marley)]
pub struct NewAgent {
    /// The folder the agent starts in; none asks where.
    pub folder: Option<PathBuf>,
}

/// `NewAgent`'s fields as a keymap writes them, read through this struct for the reason
/// `rusty::OpenPage` gives (clippy's `unsafe_derive_deserialize` on a derived `Deserialize`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewAgentFields {
    #[serde(default)]
    folder: Option<PathBuf>,
}

impl<'de> Deserialize<'de> for NewAgent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let NewAgentFields { folder } = NewAgentFields::deserialize(deserializer)?;
        Ok(Self { folder })
    }
}

/// `marley::NewAgent`: the New Agent picker, or nothing while AI is disabled, as Zed shows none
/// of its own agent entry points then.
fn new_agent(
    workspace: &mut Workspace,
    action: &NewAgent,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if DisableAiSettings::get_global(cx).disable_ai {
        return;
    }
    // From a terminal the key is Marley's, not the program's (#563).
    if let Some(view) = crate::blocks::focused_terminal(workspace, window, cx) {
        let workspace_entity = cx.entity();
        crate::shortcut_note::taken(
            action,
            "opened Marley's New Agent picker",
            &view.focus_handle(cx),
            &workspace_entity,
            window,
            cx,
        );
    }
    show_picker(workspace, action.folder.clone(), window, cx);
}

/// Opens the New Agent picker in `workspace`: it asks where the agent starts unless `folder`
/// says (#735).
pub(crate) fn show_picker(
    workspace: &mut Workspace,
    folder: Option<PathBuf>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if DisableAiSettings::get_global(cx).disable_ai {
        return;
    }
    let handle = workspace.weak_handle();
    let project = workspace.project().clone();
    let places = match folder {
        Some(_) => Vec::new(),
        None => places_for(workspace, cx.entity_id(), window, cx),
    };
    let fs = Arc::clone(&workspace.app_state().fs);
    workspace.toggle_modal(window, cx, |window, cx| {
        NewAgentPicker::new(handle, &project, folder, places, fs, window, cx)
    });
}

/// Where the picker offers to start an agent: the guess first, then the window's open projects,
/// then Browse…; recent projects join before Browse… once they load. `workspace` is being updated,
/// so it is read through its reference, never through its entity (`current`).
fn places_for(workspace: &Workspace, current: EntityId, window: &Window, cx: &App) -> Vec<Place> {
    let (guess, note) = guess(workspace, cx);
    let mut places = vec![Place::folder(guess, note)];
    let open = window
        .root::<MultiWorkspace>()
        .flatten()
        .map(|multi_workspace| {
            multi_workspace
                .read(cx)
                .workspaces()
                .filter_map(|member| {
                    let roots = if member.entity_id() == current {
                        workspace.root_paths(cx)
                    } else {
                        member.read(cx).root_paths(cx)
                    };
                    roots.first().map(|root| root.to_path_buf())
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for root in open {
        if !places.iter().any(|place| place.path() == Some(&root)) {
            places.push(Place::folder(root, "open project"));
        }
    }
    places.push(Place::Browse);
    places
}

/// The folder an agent most likely belongs in: the active terminal's folder, else the active
/// file's project, else the workspace's first root, else the home folder.
fn guess(workspace: &Workspace, cx: &App) -> (PathBuf, &'static str) {
    if let Some(item) = workspace.active_item(cx) {
        if let Some(folder) = item
            .downcast::<TerminalView>()
            .and_then(|view| view.read(cx).terminal().read(cx).working_directory())
        {
            return (folder, "this terminal's folder");
        }
        let project = workspace.project().read(cx);
        if let Some(root) = item
            .project_path(cx)
            .and_then(|path| project.worktree_for_id(path.worktree_id, cx))
            .map(|worktree| worktree.read(cx).abs_path().to_path_buf())
        {
            return (root, "this file's project");
        }
    }
    let note = if workspace.root_paths(cx).is_empty() {
        "your home folder"
    } else {
        "this project"
    };
    (crate::thread_tab::default_folder(workspace, cx), note)
}

/// `path` as the picker shows it: under the home folder as `~/…`.
fn shown_path(path: &Path) -> String {
    path.strip_prefix(util::paths::home_dir()).map_or_else(
        |_| path.display().to_string(),
        |rest| {
            if rest.as_os_str().is_empty() {
                "~".to_string()
            } else {
                format!("~/{}", rest.display())
            }
        },
    )
}

/// One place Where lists.
#[derive(Clone)]
enum Place {
    /// A folder, with its name, and what it is: the guess, an open or a recent project.
    Folder {
        name: SharedString,
        path: PathBuf,
        note: &'static str,
    },
    /// Zed's path prompt, for any folder.
    Browse,
}

impl Place {
    fn folder(path: PathBuf, note: &'static str) -> Self {
        let name = path.file_name().map_or_else(
            || shown_path(&path),
            |name| name.to_string_lossy().into_owned(),
        );
        Self::Folder {
            name: name.into(),
            path,
            note,
        }
    }

    const fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Folder { path, .. } => Some(path),
            Self::Browse => None,
        }
    }

    /// The text the query matches: the folder's name and where it is.
    fn text(&self) -> String {
        match self {
            Self::Folder { name, path, .. } => format!("{name} {}", shown_path(path)),
            Self::Browse => "Browse…".to_string(),
        }
    }
}

/// The New Agent picker: Zed's agents, then the agent CLIs on the search path; then, unless the
/// folder is given, where to start the one chosen. It starts in the workspace the picker opened in.
struct NewAgentPicker {
    picker: Entity<Picker<NewAgentDelegate>>,
}

impl NewAgentPicker {
    fn new(
        workspace: WeakEntity<Workspace>,
        project: &Entity<Project>,
        folder: Option<PathBuf>,
        places: Vec<Place>,
        fs: Arc<dyn fs::Fs>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let threads = thread_agents(project, cx)
            .into_iter()
            .map(|(agent, name, icon)| Choice {
                name,
                icon,
                start: Start::Thread(agent),
            });
        let clis = installed_clis(launcher(cx).search_path.as_deref())
            .into_iter()
            .map(|kind| Choice {
                name: kind.display_name().into(),
                icon: AgentIcon::Named(cli_icon(kind)),
                start: Start::Cli(kind),
            });
        let asks = folder.is_none();
        let delegate = NewAgentDelegate {
            modal: cx.entity().downgrade(),
            workspace,
            choices: threads.chain(clis).collect(),
            fixed: folder,
            places,
            fs: Arc::clone(&fs),
            chosen: None,
            matches: Vec::new(),
            selected_index: 0,
        };
        let picker = cx.new(|cx| Picker::uniform_list(delegate, window, cx));
        if asks {
            load_recent(&picker, fs, window, cx);
        }
        Self { picker }
    }
}

/// Adds the recent local projects to Where's places once the workspace database gives them, and
/// shows them if Where is already up.
fn load_recent(
    picker: &Entity<Picker<NewAgentDelegate>>,
    fs: Arc<dyn fs::Fs>,
    window: &Window,
    cx: &Context<NewAgentPicker>,
) {
    let db = WorkspaceDb::global(cx);
    let picker = picker.downgrade();
    cx.spawn_in(window, async move |_, cx| {
        let recent: Vec<PathBuf> = db
            .recent_project_workspaces(fs.as_ref())
            .await
            .log_err()
            .unwrap_or_default()
            .into_iter()
            .filter(|recent| matches!(recent.location, SerializedWorkspaceLocation::Local))
            .filter_map(|recent| recent.paths.paths().first().cloned())
            .take(RECENT_PLACES)
            .collect();
        picker
            .update_in(cx, |picker, window, cx| {
                let places = &mut picker.delegate.places;
                let browse = places.len().saturating_sub(1);
                let fresh: Vec<Place> = recent
                    .into_iter()
                    .filter(|path| !places.iter().any(|place| place.path() == Some(path)))
                    .map(|path| Place::folder(path, "recent project"))
                    .collect();
                places.splice(browse..browse, fresh);
                if picker.delegate.chosen.is_some() {
                    picker.refresh(window, cx);
                }
            })
            .log_err();
    })
    .detach();
}

/// How many recent projects Where lists.
const RECENT_PLACES: usize = 8;

impl ModalView for NewAgentPicker {}

impl EventEmitter<DismissEvent> for NewAgentPicker {}

impl Focusable for NewAgentPicker {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.picker.focus_handle(cx)
    }
}

impl Render for NewAgentPicker {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("NewAgentPicker")
            .w(rems(34.))
            .child(self.picker.clone())
    }
}

/// One entry of the picker: the name and icon it shows, and what it starts.
struct Choice {
    name: SharedString,
    icon: AgentIcon,
    start: Start,
}

#[derive(Clone)]
enum Start {
    /// A thread of this agent in a center tab (#734).
    Thread(AgentId),
    /// This agent CLI in a new center terminal.
    Cli(AgentKind),
}

impl Start {
    /// Where the choice starts, as its entry says: Claude Code can be both a Zed agent and a CLI.
    const fn place(&self) -> &'static str {
        match self {
            Self::Thread(_) => "Thread",
            Self::Cli(_) => "Terminal",
        }
    }

    /// Starts the choice in `folder`, in `workspace`: a thread in a tab, or the CLI in a new
    /// terminal, as a launch config starts one (#527).
    fn run_in(
        self,
        folder: PathBuf,
        workspace: &mut Workspace,
        window: &Window,
        cx: &mut Context<Workspace>,
    ) {
        match self {
            Self::Thread(agent) => {
                crate::thread_tab::start(workspace, Some(agent), Some(folder), window, cx);
            }
            Self::Cli(kind) => {
                let (line, joining) = launch_input(workspace, kind, &folder, cx);
                start_in_terminal(
                    workspace,
                    Some(folder),
                    Some(kind),
                    Some(line),
                    joining,
                    window,
                    cx,
                )
                .detach_and_log_err(cx);
            }
        }
    }
}

struct NewAgentDelegate {
    modal: WeakEntity<NewAgentPicker>,
    workspace: WeakEntity<Workspace>,
    choices: Vec<Choice>,
    /// The folder the agent starts in without asking: Open Agent Here's.
    fixed: Option<PathBuf>,
    /// The places Where lists.
    places: Vec<Place>,
    fs: Arc<dyn fs::Fs>,
    /// The agent chosen and its name, once the picker asks where.
    chosen: Option<(Start, SharedString)>,
    /// The entries the query matches, in order, each naming its entry by index: a choice, or
    /// once an agent is chosen, a place.
    matches: Vec<StringMatch>,
    selected_index: usize,
}

impl NewAgentDelegate {
    /// Asks Zed's path prompt for a folder, then starts `start` there; a file picked means its
    /// folder. Deferred: the prompt is a modal that takes this picker's place, and the modal layer
    /// reads the picker, which the confirm that calls this is updating.
    fn browse(&self, start: Start, window: &Window, cx: &mut Context<Picker<Self>>) {
        let fs = Arc::clone(&self.fs);
        let workspace = self.workspace.clone();
        window.defer(cx, move |window, cx| {
            Self::prompt_and_run(&workspace, start, fs, window, cx);
        });
    }

    fn prompt_and_run(
        workspace: &WeakEntity<Workspace>,
        start: Start,
        fs: Arc<dyn fs::Fs>,
        window: &mut Window,
        cx: &mut App,
    ) {
        let started = workspace.update(cx, |workspace, cx| {
            let lister = DirectoryLister::Local(workspace.project().clone(), Arc::clone(&fs));
            let paths = workspace.prompt_for_open_path(
                PathPromptOptions {
                    files: false,
                    directories: true,
                    multiple: false,
                    prompt: Some(SharedString::new_static("Start Here")),
                },
                lister,
                window,
                cx,
            );
            cx.spawn_in(window, async move |workspace, cx| {
                let Some(path) = paths
                    .await
                    .log_err()
                    .flatten()
                    .and_then(|paths| paths.into_iter().next())
                else {
                    return;
                };
                let folder = if fs.is_dir(&path).await {
                    Some(path)
                } else {
                    path.parent().map(Path::to_path_buf)
                };
                let Some(folder) = folder else {
                    return;
                };
                workspace
                    .update_in(cx, |workspace, window, cx| {
                        start.run_in(folder, workspace, window, cx);
                    })
                    .log_err();
            })
            .detach();
        });
        started.log_err();
    }
}

impl PickerDelegate for NewAgentDelegate {
    type ListItem = ListItem;

    fn name() -> &'static str {
        "marley new agent"
    }

    fn match_count(&self) -> usize {
        self.matches.len()
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(&mut self, index: usize, _: &mut Window, _: &mut Context<Picker<Self>>) {
        self.selected_index = index;
    }

    fn placeholder_text(&self, _: &mut Window, _: &mut App) -> Arc<str> {
        match (&self.chosen, &self.fixed) {
            (Some((_, name)), _) => format!("Where should {name} start?").into(),
            (None, Some(folder)) => format!("Start an agent in {}…", shown_path(folder)).into(),
            (None, None) => "Start an agent…".into(),
        }
    }

    fn update_matches(
        &mut self,
        query: String,
        window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        let texts: Vec<String> = if self.chosen.is_some() {
            self.places.iter().map(Place::text).collect()
        } else {
            self.choices
                .iter()
                .map(|choice| choice.name.to_string())
                .collect()
        };
        let candidates: Vec<StringMatchCandidate> = texts
            .iter()
            .enumerate()
            .map(|(index, text)| StringMatchCandidate::new(index, text))
            .collect();
        let executor = cx.background_executor().clone();
        cx.spawn_in(window, async move |picker, cx| {
            // An empty query keeps every entry in its own order; fuzzy matching would sort them
            // by score.
            let matches = if query.is_empty() {
                candidates
                    .into_iter()
                    .map(|candidate| StringMatch {
                        candidate_id: candidate.id,
                        score: 0.,
                        positions: Vec::new(),
                        string: candidate.string,
                    })
                    .collect()
            } else {
                let cancelled = AtomicBool::new(false);
                fuzzy::match_strings(&candidates, &query, false, true, 100, &cancelled, executor)
                    .await
            };
            let update = |picker: &mut Picker<Self>, cx: &mut Context<Picker<Self>>| {
                picker.delegate.matches = matches;
                picker.delegate.selected_index = 0;
                cx.notify();
            };
            picker.update(cx, update).log_err();
        })
    }

    fn confirm(&mut self, _: bool, window: &mut Window, cx: &mut Context<Picker<Self>>) {
        let Some(found) = self.matches.get(self.selected_index) else {
            return;
        };
        let index = found.candidate_id;
        match self.chosen.clone() {
            None => {
                let Some(choice) = self.choices.get(index) else {
                    return;
                };
                let (start, name) = (choice.start.clone(), choice.name.clone());
                if let Some(folder) = self.fixed.clone() {
                    self.workspace
                        .update(cx, |workspace, cx| {
                            start.run_in(folder, workspace, window, cx);
                        })
                        .log_err();
                    self.dismissed(window, cx);
                    return;
                }
                // Where: the same picker, emptied, lists the places.
                self.chosen = Some((start, name));
                self.selected_index = 0;
                cx.defer_in(window, |picker, window, cx| {
                    picker.set_query("", window, cx);
                    picker.refresh_placeholder(window, cx);
                    picker.refresh(window, cx);
                });
            }
            Some((start, _)) => {
                let Some(place) = self.places.get(index).cloned() else {
                    return;
                };
                self.dismissed(window, cx);
                match place {
                    Place::Folder { path, .. } => {
                        self.workspace
                            .update(cx, |workspace, cx| {
                                start.run_in(path, workspace, window, cx);
                            })
                            .log_err();
                    }
                    Place::Browse => self.browse(start, window, cx),
                }
            }
        }
    }

    fn dismissed(&mut self, _: &mut Window, cx: &mut Context<Picker<Self>>) {
        self.modal
            .update(cx, |_, cx| cx.emit(DismissEvent))
            .log_err();
    }

    fn render_match(
        &self,
        index: usize,
        selected: bool,
        _: &mut Window,
        _: &mut Context<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        let found = self.matches.get(index)?;
        if self.chosen.is_some() {
            let place = self.places.get(found.candidate_id)?;
            let item = ListItem::new(index)
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .toggle_state(selected);
            return Some(match place {
                Place::Folder { name, path, note } => {
                    // The highlight's positions run over the name, then the path after a space.
                    let name_end = name.len();
                    let positions = found
                        .positions
                        .iter()
                        .copied()
                        .filter(|position| *position < name_end)
                        .collect();
                    item.start_slot(Icon::new(IconName::Folder).color(Color::Muted))
                        .child(
                            h_flex()
                                .gap_2()
                                .child(HighlightedLabel::new(name.clone(), positions))
                                .child(
                                    Label::new(shown_path(path))
                                        .size(LabelSize::Small)
                                        .color(Color::Muted)
                                        .truncate(),
                                ),
                        )
                        .end_slot(Label::new(*note).size(LabelSize::Small).color(Color::Muted))
                }
                Place::Browse => item
                    .start_slot(Icon::new(IconName::FolderOpen).color(Color::Muted))
                    .child(Label::new("Browse…")),
            });
        }
        let choice = self.choices.get(found.candidate_id)?;
        let icon = match &choice.icon {
            AgentIcon::Named(name) => Icon::new(*name),
            AgentIcon::Svg(path) => Icon::from_external_svg(path.clone()),
        };
        Some(
            ListItem::new(index)
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .toggle_state(selected)
                .start_slot(icon.color(Color::Muted))
                .child(HighlightedLabel::new(
                    choice.name.clone(),
                    found.positions.clone(),
                ))
                .end_slot(
                    Label::new(choice.start.place())
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
        )
    }
}

#[cfg(test)]
#[path = "agents_tests.rs"]
mod tests;
