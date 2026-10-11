//! `marley <command>`: verbs on Marley's command line that open things in the running Marley
//! (#749).
//!
//! Chad, 2026-10-11: "we should think about commands we can run like `marley <command>` where it can
//! open things inside marley. for example i could cd /whatever/folder and do marley agent-thread
//! --provider=claude and it would pop open a new agent thread". `marley` is the app itself. The
//! launch reads a verb before Zed's own argument parser, which would refuse its flags, answers
//! `help` and argument errors itself, and otherwise launches with one `zed://marley/…` URL: a
//! second launch hands it to the running Marley over #513's socket, and a first one starts Marley,
//! restores the session and then runs it. A verb works in a folder, the one named or else the
//! shell's, and lands in the group whose project holds that folder, else in Default.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use agent_ui::{Agent, AgentPanel};
use anyhow::Context as _;
use db::kvp::KeyValueStore;
use gpui::{App, AppContext as _, Context, Entity, TaskExt as _, Window, WindowHandle};
use marley_agent::AgentKind;
use project::{AgentId, AgentRegistryStore, DisableAiSettings, Project};
use settings::Settings as _;
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{AppState, MultiWorkspace, Toast, Workspace};

use crate::groups::{self, GroupKind};
use crate::rusty::brain::Screen;

/// The prefix of a verb's URL. The verbs' grammar never names the scheme, so #445 can move them
/// to `marley://` here alone.
const URL_PREFIX: &str = "zed://marley/";

/// The longest datagram the running Marley's listener reads (#513).
const MAX_URL_BYTES: usize = 1024;

/// What `marley help` prints.
const HELP: &str = "\
marley <command>: open things in the running Marley, working in the folder you run it from.

  marley agent-thread [--provider=NAME] [--folder=PATH]  an agent thread in a tab
  marley agent-cli [--provider=NAME]                     an agent CLI in a new terminal
  marley terminal [PATH]                                 a terminal in PATH, else here
  marley talk marley|rusty                               a conversation with Marley or Rusty
  marley threads                                         the Threads page
  marley home                                            the Home page
  marley rusty [SCREEN]                                  Rusty's home page, or one of its screens:
                                                         brain, today, graph, tasks, decisions,
                                                         memory, skills, secrets
  marley panel NAME                                      project, git, outline, collab, fleet,
                                                         knowledge, containers or debug
  marley browser URL                                     a Browser tab on URL, or on a file here
  marley help                                            this list

A command opens in the group whose project holds the folder, else in Default. For a thread,
--provider takes claude, codex, zed, marley, rusty or an agent server's id; for a CLI, claude,
codex, gemini or opencode. Without it, the agent you started last that way. With no Marley
running, Marley starts, restores your session, then runs the command. In a terminal of a Marley on
a data directory of its own, the command reaches that Marley.";

/// The verbs, as typed.
const VERBS: [&str; 10] = [
    "agent-thread",
    "agent-cli",
    "terminal",
    "talk",
    "threads",
    "home",
    "rusty",
    "panel",
    "browser",
    "help",
];

/// Who `marley talk` talks to.
const TALKERS: [&str; 2] = ["marley", "rusty"];

/// Rusty's screens by the names `marley rusty` takes, in the header's order.
const RUSTY_SCREENS: [(&str, Screen); 8] = [
    ("brain", Screen::Brain),
    ("today", Screen::Today),
    ("graph", Screen::Graph),
    ("tasks", Screen::Tasks),
    ("decisions", Screen::Decisions),
    ("memory", Screen::Memory),
    ("skills", Screen::Skills),
    ("secrets", Screen::Secrets),
];

/// The panels `marley panel` opens, by the names it takes and their persistent names.
const PANELS: [(&str, &str); 8] = [
    ("project", "Project Panel"),
    ("git", "GitPanel"),
    ("outline", "Outline Panel"),
    ("collab", "CollabPanel"),
    ("fleet", "MarleyFleetPanel"),
    ("knowledge", "MarleyKnowledgePanel"),
    ("containers", "MarleyContainersPanel"),
    ("debug", "DebugPanel"),
];

/// The key-value scope where the agent each kind of verb started last is kept.
const SCOPE: &str = "marley-commands";
const THREAD_KEY: &str = "agent-thread";
const CLI_KEY: &str = "agent-cli";

/// How often, and how many times, a verb looks for what it needs in a group just shown: the Agent
/// Panel, or the panel it opens, which Zed adds while it restores a window (L-claude-736).
const READY_WAITS: usize = 100;
const READY_WAIT: Duration = Duration::from_millis(100);

/// What a launch does with its arguments.
#[derive(Debug, PartialEq, Eq)]
pub enum Invoked {
    /// Not a verb: Zed reads the arguments as before.
    NotACommand,
    /// Answered here: the launch prints the message and exits.
    Exit {
        /// The exit code: 0 for `help`, 2 for wrong arguments.
        code: i32,
        /// What to print: to standard output for code 0, else to standard error.
        message: String,
    },
    /// Launch with these arguments: the binary, the data directory when one is known, and the
    /// verb's URL.
    Launch(Vec<String>),
}

/// Reads a `marley <verb>` launch. The app's `main` calls it before Zed's own argument parser.
#[must_use]
pub fn args_if_invoked() -> Invoked {
    let Ok(arguments) = std::env::args_os()
        .map(std::ffi::OsString::into_string)
        .collect::<Result<Vec<String>, _>>()
    else {
        return Invoked::NotACommand;
    };
    let inherited = std::env::var(marley_terminal::identity::DATA_DIR_VARIABLE)
        .ok()
        .filter(|directory| !directory.is_empty());
    match read_launch(&arguments, std::env::current_dir(), inherited) {
        Launch::NotACommand => Invoked::NotACommand,
        Launch::Help => Invoked::Exit {
            code: 0,
            message: HELP.to_string(),
        },
        Launch::Wrong(error) => Invoked::Exit {
            code: 2,
            message: format!("marley: {error}\nRun `marley help` for the commands."),
        },
        Launch::Run { data_dir, url } => {
            // A Marley that runs gets the verb now, before Zed's `main` starts its log (#775).
            #[cfg(any(target_os = "linux", target_os = "freebsd"))]
            match hand_to_running(data_dir.as_deref(), &url) {
                Ok(Some(line)) => {
                    return Invoked::Exit {
                        code: 0,
                        message: line,
                    };
                }
                Ok(None) => {}
                Err(error) => {
                    return Invoked::Exit {
                        code: 1,
                        message: format!("marley: {error:#}"),
                    };
                }
            }
            let mut launch: Vec<String> = arguments.into_iter().take(1).collect();
            if let Some(data_dir) = data_dir {
                launch.extend(["--user-data-dir".to_string(), data_dir]);
            }
            launch.push(url);
            Invoked::Launch(launch)
        }
    }
}

/// Hands the verb's `url` to the Marley running on the launch's data directory: `data_dir` when
/// the launch names one, read as `paths::set_custom_data_dir` makes it but without making it,
/// since that function panics once the data directory was read; else the default one, which
/// `main` reads the same way after (#775). `None` when no Marley runs there.
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn hand_to_running(data_dir: Option<&str>, url: &str) -> anyhow::Result<Option<String>> {
    let data_dir = match data_dir {
        Some(directory) => match std::fs::canonicalize(directory) {
            Ok(directory) => directory,
            // A directory that is not there has no Marley.
            Err(_) => return Ok(None),
        },
        None => paths::data_dir().clone(),
    };
    crate::single_instance::hand_verb(url, &data_dir)
}

/// Notes the data directory this Marley runs on when its launch named one, for its terminals to
/// pass on (#749's D9). Never the default directory: `--user-data-dir` also moves the config.
pub(crate) fn note_data_dir() {
    let named = std::env::args()
        .any(|argument| argument == "--user-data-dir" || argument.starts_with("--user-data-dir="));
    marley_terminal::identity::set_data_dir(named.then(|| paths::data_dir().clone()));
}

/// A launch's arguments, read.
#[derive(Debug, PartialEq, Eq)]
enum Launch {
    NotACommand,
    Help,
    Wrong(String),
    Run {
        data_dir: Option<String>,
        url: String,
    },
}

fn read_launch(
    arguments: &[String],
    working_dir: std::io::Result<PathBuf>,
    inherited_data_dir: Option<String>,
) -> Launch {
    let mut rest = arguments.get(1..).unwrap_or_default();
    let mut data_dir = None;
    if let Some(first) = rest.first() {
        if first == "--user-data-dir" {
            let Some(directory) = rest.get(1) else {
                return Launch::NotACommand;
            };
            data_dir = Some(directory.clone());
            rest = rest.get(2..).unwrap_or_default();
        } else if let Some(directory) = first.strip_prefix("--user-data-dir=") {
            data_dir = Some(directory.to_string());
            rest = rest.get(1..).unwrap_or_default();
        }
    }
    let Some((verb, verb_arguments)) = rest.split_first() else {
        return Launch::NotACommand;
    };
    if !VERBS.contains(&verb.as_str()) {
        return Launch::NotACommand;
    }
    if verb == "help"
        || verb_arguments
            .iter()
            .any(|argument| argument == "--help" || argument == "-h")
    {
        return Launch::Help;
    }
    let working_dir = match working_dir {
        Ok(working_dir) => working_dir,
        Err(error) => return Launch::Wrong(format!("the working directory: {error}")),
    };
    let command = match Command::read(verb, verb_arguments, &working_dir) {
        Ok(command) => command,
        Err(error) => return Launch::Wrong(error),
    };
    let url = match command.to_url() {
        Ok(url) => url,
        Err(error) => return Launch::Wrong(error),
    };
    if url.len() > MAX_URL_BYTES {
        return Launch::Wrong(format!(
            "the command is {} bytes as a URL, over the {MAX_URL_BYTES} Marley reads",
            url.len()
        ));
    }
    Launch::Run {
        data_dir: data_dir.or(inherited_data_dir),
        url,
    }
}

/// A verb and the folder it works in.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Command {
    verb: Verb,
    folder: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Verb {
    AgentThread { provider: Option<String> },
    AgentCli { provider: Option<AgentKind> },
    Terminal,
    Talk(&'static str),
    Threads,
    Home,
    Rusty(Option<&'static str>),
    Panel(&'static str),
    Browser(String),
}

impl Verb {
    const fn name(&self) -> &'static str {
        match self {
            Self::AgentThread { .. } => "agent-thread",
            Self::AgentCli { .. } => "agent-cli",
            Self::Terminal => "terminal",
            Self::Talk(_) => "talk",
            Self::Threads => "threads",
            Self::Home => "home",
            Self::Rusty(_) => "rusty",
            Self::Panel(_) => "panel",
            Self::Browser(_) => "browser",
        }
    }
}

/// A verb's flags and its other arguments, as typed.
#[derive(Default)]
struct Typed {
    provider: Option<String>,
    folder: Option<String>,
    positional: Vec<String>,
}

impl Typed {
    /// Reads `arguments`, taking only the flags in `flags`: `--name=value` or `--name value`.
    fn read(verb: &str, arguments: &[String], flags: &[&str]) -> Result<Self, String> {
        let mut typed = Self::default();
        let mut arguments = arguments.iter();
        while let Some(argument) = arguments.next() {
            let Some(flag) = argument.strip_prefix("--") else {
                typed.positional.push(argument.clone());
                continue;
            };
            let (name, value) = match flag.split_once('=') {
                Some((name, value)) => (name, value.to_string()),
                None => (
                    flag,
                    arguments
                        .next()
                        .cloned()
                        .ok_or_else(|| format!("--{flag} needs a value"))?,
                ),
            };
            if !flags.contains(&name) {
                return Err(format!("{verb} takes no --{name}"));
            }
            if value.is_empty() {
                return Err(format!("--{name} needs a value"));
            }
            match name {
                "provider" => typed.provider = Some(value),
                _ => typed.folder = Some(value),
            }
        }
        Ok(typed)
    }

    /// The one argument `verb` takes, if it takes one.
    fn at_most_one(&self, verb: &str) -> Result<Option<&str>, String> {
        match self.positional.as_slice() {
            [] => Ok(None),
            [one] => Ok(Some(one)),
            [_, extra, ..] => Err(format!("{verb} does not take {extra:?}")),
        }
    }

    /// The one argument `verb` needs.
    fn exactly_one(&self, verb: &str, what: &str) -> Result<&str, String> {
        self.at_most_one(verb)?
            .ok_or_else(|| format!("{verb} needs {what}"))
    }
}

/// The name in `names` that `typed` is, in any case.
fn pick(names: &[&'static str], typed: &str, what: &str) -> Result<&'static str, String> {
    names
        .iter()
        .find(|name| name.eq_ignore_ascii_case(typed))
        .copied()
        .ok_or_else(|| format!("{typed:?} is not {what}: {}", names.join(", ")))
}

/// `path` against `working_dir`, or `working_dir` itself, as an absolute folder that exists.
fn folder_at(path: Option<&str>, working_dir: &Path) -> Result<PathBuf, String> {
    let path = path.map_or_else(|| working_dir.to_path_buf(), |path| working_dir.join(path));
    let folder =
        std::fs::canonicalize(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    if !folder.is_dir() {
        return Err(format!("{} is not a folder", folder.display()));
    }
    Ok(folder)
}

impl Command {
    /// Reads `verb` with its arguments, the launch's working directory standing for the folder.
    fn read(verb: &str, arguments: &[String], working_dir: &Path) -> Result<Self, String> {
        let flags: &[&str] = match verb {
            "agent-thread" => &["provider", "folder"],
            "agent-cli" => &["provider"],
            _ => &[],
        };
        let typed = Typed::read(verb, arguments, flags)?;
        let mut folder_path = typed.folder.as_deref();
        let verb = match verb {
            "agent-thread" => {
                if let Some(extra) = typed.positional.first() {
                    return Err(format!("agent-thread does not take {extra:?}"));
                }
                Verb::AgentThread {
                    provider: typed.provider.clone(),
                }
            }
            "agent-cli" => {
                if let Some(extra) = typed.at_most_one(verb)? {
                    return Err(format!("agent-cli does not take {extra:?}"));
                }
                let provider = match typed.provider.as_deref() {
                    Some(name) => {
                        let programs = AgentKind::ALL.map(AgentKind::program);
                        let program = pick(&programs, name, "an agent CLI")?;
                        AgentKind::ALL
                            .into_iter()
                            .find(|kind| kind.program() == program)
                    }
                    None => None,
                };
                Verb::AgentCli { provider }
            }
            "terminal" => {
                folder_path = typed.at_most_one(verb)?;
                Verb::Terminal
            }
            "talk" => Verb::Talk(pick(
                &TALKERS,
                typed.exactly_one(verb, "marley or rusty")?,
                "someone to talk to",
            )?),
            "rusty" => {
                let names = RUSTY_SCREENS.map(|(name, _)| name);
                Verb::Rusty(
                    typed
                        .at_most_one(verb)?
                        .map(|screen| pick(&names, screen, "a Rusty screen"))
                        .transpose()?,
                )
            }
            "panel" => {
                let names = PANELS.map(|(name, _)| name);
                Verb::Panel(pick(
                    &names,
                    typed.exactly_one(verb, "a panel's name")?,
                    "a panel",
                )?)
            }
            "browser" => {
                let text = typed.exactly_one(verb, "a URL")?;
                let folder = folder_at(None, working_dir)?;
                let url = match marley_browser::address::local_page(text, &folder) {
                    Some(page) => url::Url::from_file_path(&page)
                        .map_err(|()| format!("{} is not a page Marley can open", page.display()))?
                        .to_string(),
                    None => marley_browser::address::url_for(text)
                        .ok_or_else(|| format!("{text:?} is not an address"))?,
                };
                Verb::Browser(url)
            }
            _ => {
                if let Some(extra) = typed.at_most_one(verb)? {
                    return Err(format!("{verb} does not take {extra:?}"));
                }
                match verb {
                    "threads" => Verb::Threads,
                    _ => Verb::Home,
                }
            }
        };
        Ok(Self {
            verb,
            folder: folder_at(folder_path, working_dir)?,
        })
    }

    /// The command as the URL a launch hands over: the verb, then its folder and arguments.
    fn to_url(&self) -> Result<String, String> {
        let folder = self
            .folder
            .to_str()
            .ok_or_else(|| format!("{} is not a UTF-8 path", self.folder.display()))?;
        let mut query = url::form_urlencoded::Serializer::new(String::new());
        query.append_pair("folder", folder);
        match &self.verb {
            Verb::AgentThread {
                provider: Some(provider),
            } => {
                query.append_pair("provider", provider);
            }
            Verb::AgentCli {
                provider: Some(kind),
            } => {
                query.append_pair("provider", kind.program());
            }
            Verb::Talk(name) | Verb::Rusty(Some(name)) | Verb::Panel(name) => {
                query.append_pair("name", name);
            }
            Verb::Browser(url) => {
                query.append_pair("url", url);
            }
            _ => {}
        }
        Ok(format!(
            "{URL_PREFIX}{}?{}",
            self.verb.name(),
            query.finish()
        ))
    }

    /// The command a URL's path and query name, as [`Command::to_url`] wrote them.
    fn from_path_and_query(text: &str) -> Result<Self, String> {
        let (verb, query) = text.split_once('?').unwrap_or((text, ""));
        let mut folder = None;
        let mut provider = None;
        let mut name = None;
        let mut url = None;
        for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
            let value = value.into_owned();
            match key.as_ref() {
                "folder" => folder = Some(PathBuf::from(value)),
                "provider" => provider = Some(value),
                "name" => name = Some(value),
                "url" => url = Some(value),
                _ => {}
            }
        }
        let folder = folder.ok_or("no folder")?;
        let named = |names: &[&'static str]| {
            let name = name.as_deref().ok_or("no name")?;
            pick(names, name, "known")
        };
        let verb = match verb {
            "agent-thread" => Verb::AgentThread { provider },
            "agent-cli" => Verb::AgentCli {
                provider: provider.and_then(|program| {
                    AgentKind::ALL
                        .into_iter()
                        .find(|kind| kind.program() == program)
                }),
            },
            "terminal" => Verb::Terminal,
            "talk" => Verb::Talk(named(&TALKERS)?),
            "threads" => Verb::Threads,
            "home" => Verb::Home,
            "rusty" => Verb::Rusty(match name {
                Some(_) => Some(named(&RUSTY_SCREENS.map(|(name, _)| name))?),
                None => None,
            }),
            "panel" => Verb::Panel(named(&PANELS.map(|(name, _)| name))?),
            "browser" => Verb::Browser(url.ok_or("no url")?),
            other => return Err(format!("no verb {other:?}")),
        };
        Ok(Self { verb, folder })
    }
}

/// The verb part of a URL a launch handed over, for `OpenRequest::parse`.
#[must_use]
pub fn command_in(url: &str) -> Option<&str> {
    url.strip_prefix(URL_PREFIX)
}

/// The verb a handed URL names, for the hand-off's line (#513).
pub(crate) fn verb_in(url: &str) -> Option<&str> {
    let command = command_in(url)?;
    Some(command.split_once('?').map_or(command, |(verb, _)| verb))
}

/// Runs a command the launch handed over: finds its group, shows it, waits for what the verb needs
/// and runs it. What the app cannot do shows as a toast in that group (D3).
pub fn run(command: &str, app_state: Arc<AppState>, cx: &App) {
    let command = match Command::from_path_and_query(command) {
        Ok(command) => command,
        Err(error) => {
            log::error!("marley: a command Marley cannot read, {command:?}: {error}");
            return;
        }
    };
    cx.spawn(async move |cx| -> anyhow::Result<()> {
        let (window, workspace) = target(&command.folder, app_state, cx).await?;
        let multi_workspace = window.update(cx, |_, _, cx| cx.entity())?;
        cx.update_window(window.into(), |_, window, cx| {
            window.activate_window();
            multi_workspace.update(cx, |multi_workspace, cx| {
                multi_workspace.activate(workspace.clone(), None, window, cx);
            });
        })?;
        for _ in 0..READY_WAITS {
            if cx.update(|cx| ready(&command.verb, &workspace, cx)) {
                break;
            }
            cx.background_executor().timer(READY_WAIT).await;
        }
        cx.update_window(window.into(), |_, window, cx| {
            run_in(command, &multi_workspace, &workspace, window, cx);
        })
    })
    .detach_and_log_err(cx);
}

/// The window and the group a command in `folder` runs in (D5).
async fn target(
    folder: &Path,
    app_state: Arc<AppState>,
    cx: &mut gpui::AsyncApp,
) -> anyhow::Result<(WindowHandle<MultiWorkspace>, Entity<Workspace>)> {
    if let Some(found) = cx.update(|cx| group_holding(folder, cx)) {
        return Ok(found);
    }
    let window = workspace::get_any_active_multi_workspace(app_state, cx.clone())
        .await
        .context("no Marley window to run the command in")?;
    let workspace = default_group(window, cx).await?;
    Ok((window, workspace))
}

/// The window and the group whose project has a visible root holding `folder`, the deepest root
/// first, the active window's first among equals.
fn group_holding(
    folder: &Path,
    cx: &App,
) -> Option<(WindowHandle<MultiWorkspace>, Entity<Workspace>)> {
    let active = cx.active_window();
    let mut windows: Vec<WindowHandle<MultiWorkspace>> = cx
        .windows()
        .into_iter()
        .filter_map(|window| window.downcast::<MultiWorkspace>())
        .collect();
    windows
        .sort_by_key(|window| Some(window.window_id()) != active.map(|active| active.window_id()));
    let mut best: Option<(usize, WindowHandle<MultiWorkspace>, Entity<Workspace>)> = None;
    for window in windows {
        let Ok(multi_workspace) = window.read(cx) else {
            continue;
        };
        for workspace in multi_workspace.workspaces() {
            let depth = workspace
                .read(cx)
                .visible_worktrees(cx)
                .map(|worktree| worktree.read(cx).abs_path())
                .filter(|root| folder.starts_with(root.as_ref()))
                .map(|root| root.components().count())
                .max();
            if let Some(depth) = depth
                && best.as_ref().is_none_or(|(deepest, _, _)| depth > *deepest)
            {
                best = Some((depth, window, workspace.clone()));
            }
        }
    }
    best.map(|(_, window, workspace)| (window, workspace))
}

/// The group a command no project holds runs in: Default in the Marley layout, made when the
/// window has none; in Zed's layout, the workspace the window shows.
async fn default_group(
    window: WindowHandle<MultiWorkspace>,
    cx: &mut gpui::AsyncApp,
) -> anyhow::Result<Entity<Workspace>> {
    let multi_workspace = window.update(cx, |_, _, cx| cx.entity())?;
    if !cx.update(|cx| crate::marley_layout(cx)) {
        return Ok(cx.update(|cx| multi_workspace.read(cx).workspace().clone()));
    }
    let (sender, receiver) = futures::channel::oneshot::channel();
    cx.update_window(window.into(), |_, window, cx| {
        groups::with_group(
            GroupKind::Home,
            &multi_workspace,
            move |workspace, _, _| {
                sender.send(workspace).ok();
            },
            window,
            cx,
        );
    })?;
    receiver
        .await
        .context("the window's Default group was not made")
}

/// Whether the group has what the verb needs: the Agent Panel for a thread, the panel for
/// `panel`. With AI off a thread never gets its panel, and the verb says so at once.
fn ready(verb: &Verb, workspace: &Entity<Workspace>, cx: &App) -> bool {
    match verb {
        Verb::AgentThread { .. } | Verb::Talk(_) => {
            DisableAiSettings::get_global(cx).disable_ai
                || workspace.read(cx).panel::<AgentPanel>(cx).is_some()
        }
        Verb::Panel(name) => panel_named(name).is_none_or(|persistent| {
            workspace.read(cx).all_docks().iter().any(|dock| {
                dock.read(cx)
                    .panel_for_persistent_name(persistent)
                    .is_some()
            })
        }),
        _ => true,
    }
}

/// Runs the verb in `workspace`, the group just shown.
fn run_in(
    command: Command,
    multi_workspace: &Entity<MultiWorkspace>,
    workspace: &Entity<Workspace>,
    window: &mut Window,
    cx: &mut App,
) {
    // A Rusty screen opens through Rusty's own dispatcher, which reads the window's shown group.
    if let Verb::Rusty(Some(name)) = command.verb {
        let screen = RUSTY_SCREENS
            .iter()
            .find(|(screen, _)| *screen == name)
            .map(|(_, screen)| *screen);
        match (rusty_unavailable(cx), screen) {
            (Some(why), _) => workspace.update(cx, |workspace, cx| tell(workspace, &why, cx)),
            (None, Some(screen)) => {
                crate::rusty::brain::open_screen(screen, &multi_workspace.downgrade(), window, cx);
            }
            (None, None) => {}
        }
        return;
    }
    workspace.update(cx, |workspace, cx| {
        if let Err(why) = run_verb(command, workspace, window, cx) {
            tell(workspace, &why, cx);
        }
    });
}

fn run_verb(
    command: Command,
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Result<(), String> {
    let Command { verb, folder } = command;
    match verb {
        Verb::AgentThread { provider } => {
            ai_on(cx)?;
            let typed = provider.is_some();
            let Some(name) = provider.or_else(|| last(THREAD_KEY, cx)) else {
                crate::thread_tab::start(workspace, None, Some(folder), window, cx);
                return Ok(());
            };
            match thread_agent(&name, workspace.project(), cx) {
                Some(ThreadAgent::Entry(entry)) => {
                    if !crate::assistant::talk_by_name(entry, Some(folder), workspace, window, cx) {
                        return Err(format!("Marley has no {entry} agent turned on"));
                    }
                    remember_thread_agent(entry, cx);
                }
                Some(ThreadAgent::Agent(agent)) => {
                    crate::thread_tab::start(workspace, Some(agent), Some(folder), window, cx);
                }
                None if typed => {
                    return Err(format!(
                        "No agent is named {name:?}: try claude, codex, zed, marley, rusty or an \
                         agent server's id"
                    ));
                }
                None => {
                    return Err(format!(
                        "The agent started last, {name:?}, is gone; name one with --provider"
                    ));
                }
            }
        }
        Verb::AgentCli { provider } => {
            let installed =
                crate::agents::installed_clis(crate::agents::launcher(cx).search_path.as_deref());
            let kept = last(CLI_KEY, cx).and_then(|program| {
                AgentKind::ALL
                    .into_iter()
                    .find(|kind| kind.program() == program && installed.contains(kind))
            });
            let kind = match provider {
                Some(kind) if !installed.contains(&kind) => {
                    return Err(format!("{} is not installed", kind.display_name()));
                }
                Some(kind) => kind,
                None => kept
                    .or_else(|| installed.first().copied())
                    .ok_or("No agent CLI is installed")?,
            };
            crate::agents::start_cli_in(kind, folder, workspace, window, cx);
        }
        Verb::Terminal => {
            crate::routing::open_center_terminal(workspace, false, Some(folder), window, cx);
        }
        Verb::Talk(name) => {
            ai_on(cx)?;
            if !crate::assistant::talk_by_name(name, None, workspace, window, cx) {
                let entry = if name == "rusty" { "Rusty" } else { "Marley" };
                return Err(format!("Marley has no {entry} agent turned on"));
            }
        }
        Verb::Threads => crate::threads_page::open(workspace, window, cx),
        Verb::Home => crate::home_page::open_here(workspace, window, cx),
        Verb::Rusty(_) => {
            if let Some(why) = rusty_unavailable(cx) {
                return Err(why);
            }
            crate::rusty::open_home_here(workspace, window, cx);
        }
        Verb::Panel(name) => open_panel(name, workspace, window, cx)?,
        Verb::Browser(url) => {
            crate::browser::open_url_tab(workspace, url, window, cx);
        }
    }
    Ok(())
}

fn ai_on(cx: &App) -> Result<(), String> {
    if DisableAiSettings::get_global(cx).disable_ai {
        Err("AI is turned off in the settings".to_string())
    } else {
        Ok(())
    }
}

/// Why Rusty's pages cannot open, if they cannot.
fn rusty_unavailable(cx: &App) -> Option<String> {
    if !crate::rusty::is_on(cx) {
        Some("Rusty is off: turn on marley.rusty.enabled".to_string())
    } else if !crate::rusty::is_connected(cx) {
        Some("Rusty is not connected yet".to_string())
    } else {
        None
    }
}

/// The persistent name of the panel `marley panel` calls `name`.
fn panel_named(name: &str) -> Option<&'static str> {
    PANELS
        .iter()
        .find(|(short, _)| *short == name)
        .map(|(_, persistent)| *persistent)
}

/// Opens the panel `name` as Zed's `focus_panel` opens one: its dock activates it, opens and
/// focuses it. In the Marley layout #747's dock observer turns that into the panel's tab; Debug
/// stays in its dock (#774).
fn open_panel(
    name: &str,
    workspace: &Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Result<(), String> {
    let persistent = panel_named(name).ok_or_else(|| format!("No panel is named {name}"))?;
    let docks: Vec<_> = workspace.all_docks().into_iter().cloned().collect();
    for dock in docks {
        let Some(index) = dock
            .read(cx)
            .panel_index_for_persistent_name(persistent, cx)
        else {
            continue;
        };
        let Some(panel) = dock.read(cx).panel_for_persistent_name(persistent).cloned() else {
            continue;
        };
        if !panel.enabled(cx) {
            return Err(format!("The {name} panel is turned off"));
        }
        dock.update(cx, |dock, cx| {
            dock.activate_panel(index, window, cx);
            dock.set_open(true, window, cx);
        });
        panel.activation_focus_handle(cx).focus(window, cx);
        return Ok(());
    }
    Err(format!("This window has no {name} panel"))
}

/// The agent a thread's provider names.
enum ThreadAgent {
    /// One of the entries Marley adds, by its name.
    Entry(&'static str),
    Agent(AgentId),
}

/// The agent `name` names, in any case: a short name (`claude`, `codex`, `zed`, `marley`,
/// `rusty`), an agent's id, or its display name, among the agents a new thread can run and the
/// registry's.
fn thread_agent(name: &str, project: &Entity<Project>, cx: &App) -> Option<ThreadAgent> {
    match name.to_ascii_lowercase().as_str() {
        "marley" => return Some(ThreadAgent::Entry("Marley")),
        "rusty" => return Some(ThreadAgent::Entry("Rusty")),
        "zed" => return Some(ThreadAgent::Agent(Agent::NativeAgent.id())),
        "claude" => return Some(ThreadAgent::Agent(AgentId::new("claude-acp"))),
        "codex" => return Some(ThreadAgent::Agent(AgentId::new("codex-acp"))),
        _ => {}
    }
    let listed = crate::agents::thread_agents(project, cx)
        .into_iter()
        .find(|(agent, display, _)| {
            agent.0.eq_ignore_ascii_case(name) || display.eq_ignore_ascii_case(name)
        })
        .map(|(agent, _, _)| agent);
    let agent = listed.or_else(|| {
        let agent = AgentId::new(name.to_string());
        AgentRegistryStore::try_global(cx)?
            .read(cx)
            .agent(&agent)
            .is_some()
            .then_some(agent)
    })?;
    if crate::assistant::is_assistant(&agent) {
        return ["Marley", "Rusty"]
            .into_iter()
            .find(|entry| agent.0.as_ref() == *entry)
            .map(ThreadAgent::Entry);
    }
    Some(ThreadAgent::Agent(agent))
}

/// Shows why a command did not run, in the group it ran in.
fn tell(workspace: &mut Workspace, message: &str, cx: &mut Context<Workspace>) {
    struct CommandToast;
    workspace.show_toast(
        Toast::new(
            NotificationId::unique::<CommandToast>(),
            format!("marley: {message}"),
        ),
        cx,
    );
}

/// Keeps `agent` as the one `marley agent-thread` starts without `--provider` (D6).
pub(crate) fn remember_thread_agent(agent: &str, cx: &App) {
    remember(THREAD_KEY, agent.to_string(), cx);
}

/// Keeps `kind` as the CLI `marley agent-cli` starts without `--provider` (D6).
pub(crate) fn remember_cli(kind: AgentKind, cx: &App) {
    remember(CLI_KEY, kind.program().to_string(), cx);
}

fn remember(key: &'static str, value: String, cx: &App) {
    let store = KeyValueStore::global(cx);
    cx.background_spawn(async move {
        store
            .scoped(SCOPE)
            .write(key.to_string(), value)
            .await
            .log_err();
    })
    .detach();
}

fn last(key: &str, cx: &App) -> Option<String> {
    KeyValueStore::global(cx)
        .scoped(SCOPE)
        .read(key)
        .log_err()
        .flatten()
}
