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
    AnyWindowHandle, App, AsyncApp, AsyncWindowContext, Context, DismissEvent, Entity,
    EventEmitter, FocusHandle, Focusable, Global, Render, Task, WeakEntity, Window,
};
use marley_agent::{AgentKind, LaunchMode};
use picker::{Picker, PickerDelegate};
use project::{AgentId, AgentRegistryStore, DisableAiSettings, Project};
use settings::{ClaudeCodePermissions, CodexPermissions, MarleySettingsContent, Settings as _};
use terminal::Terminal;
use terminal_view::terminal_panel::TerminalPanel;
use ui::{HighlightedLabel, IconName, ListItem, ListItemSpacing, prelude::*};
use util::ResultExt as _;
use workspace::notifications::DetachAndPromptErr as _;
use workspace::{ModalView, MultiWorkspace, Workspace};

use crate::NewAgent;

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
    let mode = launch_mode(workspace, kind, cx);
    let input = marley_agent::launch_line_after(setup.unwrap_or_default(), kind, mode, prompt);
    let directory = terminal_view::default_working_directory(workspace, cx);
    start_in_terminal(workspace, directory, Some(kind), Some(input), window, cx).prompt_err(
        "Could not start the agent",
        window,
        cx,
        |_, _, _| None,
    )
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

/// What starts `kind` in a terminal of `workspace`, as the rail's Agent CLIs entries start it:
/// its program with the arguments its permission setting asks for (#527).
pub(crate) fn launch_input(workspace: &Workspace, kind: AgentKind, cx: &App) -> Vec<u8> {
    marley_agent::launch_line_after("", kind, launch_mode(workspace, kind, cx), "")
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
/// in a local project, asks for ssh's passphrases in Marley (#596). The task gives the terminal
/// once the input is written.
pub(crate) fn start_in_terminal(
    workspace: &Workspace,
    directory: Option<PathBuf>,
    agent: Option<AgentKind>,
    input: Option<Vec<u8>>,
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
            Some(_) => agent_env(script.as_deref())?,
            None => HashMap::default(),
        };
        let terminal = workspace
            .update_in(cx, |workspace, window, cx| {
                TerminalPanel::add_center_terminal(workspace, window, cx, move |project, cx| {
                    factory(project, directory, env, cx)
                })
            })?
            .await?;
        if let Some(proxy) = proxy {
            asker.get_or_init(|| terminal.clone());
            // The socket and its folder last exactly as long as the terminal.
            terminal.update(cx, |_, cx| cx.on_release(move |_, _| drop(proxy)).detach())?;
        }
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

/// `marley::NewAgent`: the New Agent picker, or nothing while AI is disabled, as Zed shows none
/// of its own agent entry points then.
fn new_agent(
    workspace: &mut Workspace,
    _: &NewAgent,
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
            &NewAgent,
            "opened Marley's New Agent picker",
            &view.focus_handle(cx),
            &workspace_entity,
            window,
            cx,
        );
    }
    let handle = workspace.weak_handle();
    let project = workspace.project().clone();
    workspace.toggle_modal(window, cx, |window, cx| {
        NewAgentPicker::new(handle, &project, window, cx)
    });
}

/// The New Agent picker: Zed's agents, then the agent CLIs on the search path. The choice starts
/// in the workspace the picker opened in.
struct NewAgentPicker {
    picker: Entity<Picker<NewAgentDelegate>>,
}

impl NewAgentPicker {
    fn new(
        workspace: WeakEntity<Workspace>,
        project: &Entity<Project>,
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
        let delegate = NewAgentDelegate {
            modal: cx.entity().downgrade(),
            workspace,
            choices: threads.chain(clis).collect(),
            matches: Vec::new(),
            selected_index: 0,
        };
        Self {
            picker: cx.new(|cx| Picker::uniform_list(delegate, window, cx)),
        }
    }
}

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
    /// A thread of this agent in the Agent Panel.
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

    fn run(
        self,
        workspace: &mut Workspace,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Thread(agent) => start_thread(workspace, &agent, window, cx),
            Self::Cli(kind) => {
                start_cli(workspace, kind, window, cx);
                Ok(())
            }
        }
    }
}

struct NewAgentDelegate {
    modal: WeakEntity<NewAgentPicker>,
    workspace: WeakEntity<Workspace>,
    choices: Vec<Choice>,
    /// The choices the query matches, in order, each naming its choice by index.
    matches: Vec<StringMatch>,
    selected_index: usize,
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
        "Start an agent in this project…".into()
    }

    fn update_matches(
        &mut self,
        query: String,
        window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        let candidates: Vec<StringMatchCandidate> = self
            .choices
            .iter()
            .enumerate()
            .map(|(index, choice)| StringMatchCandidate::new(index, &choice.name))
            .collect();
        let executor = cx.background_executor().clone();
        cx.spawn_in(window, async move |picker, cx| {
            // An empty query keeps every choice in its own order; fuzzy matching would sort
            // them by score.
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
        let start = self
            .matches
            .get(self.selected_index)
            .and_then(|found| self.choices.get(found.candidate_id))
            .map(|choice| choice.start.clone());
        if let Some(start) = start {
            let started = self
                .workspace
                .update(cx, |workspace, cx| start.run(workspace, window, cx));
            Task::ready(started.and_then(|started| started)).detach_and_prompt_err(
                "Could not start the agent",
                window,
                cx,
                |_, _, _| None,
            );
        }
        self.dismissed(window, cx);
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
