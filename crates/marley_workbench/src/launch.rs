//! A project's launch configs (#527).
//!
//! `.zed/marley.json` at a project's root names launch configs, each a list of items: a terminal
//! that runs a command, an agent CLI, or a Browser tab on a URL, each a tab or split off the pane
//! of the item before it. The rail's + lists the configs, and one click opens a config's items in
//! order. A config runs only after the user approves its exact text: Zed's key-value store keeps
//! the SHA-256 of what was approved, and a changed text asks again.

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use collections::{HashMap, IndexMap};
use db::kvp::KeyValueStore;
use fs::Fs;
use gpui::{App, AsyncWindowContext, Context, Entity, Global, PromptLevel};
use gpui::{FocusHandle, WeakEntity, Window};
use marley_agent::AgentKind;
use marley_browser::address::LocalUrl;
use project::Project;
use serde::Deserialize;
use sha2::{Digest as _, Sha256};
use terminal_view::TerminalView;
use util::ResultExt as _;
use workspace::item::ItemHandle;
use workspace::notifications::DetachAndPromptErr as _;
use workspace::{Pane, SplitDirection, Workspace};

use crate::{agents, browser};

/// The file, beside Zed's own in a project's `.zed` folder.
pub(crate) const LAUNCH_FILE: &str = ".zed/marley.json";

/// The key-value store's namespace for the approved configs.
const APPROVALS: &str = "marley-launch";

/// How long a Browser item on a loopback URL waits for its port to accept a connection.
const PORT_WAIT: Duration = Duration::from_secs(30);

/// How often it tries.
const PORT_RETRY: Duration = Duration::from_millis(500);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LaunchFile {
    #[serde(default)]
    launch: IndexMap<String, ConfigFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    items: Vec<ItemFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ItemFile {
    terminal: Option<String>,
    agent: Option<String>,
    browser: Option<String>,
    title: Option<String>,
    cwd: Option<String>,
    split: Option<SplitFile>,
    #[serde(default)]
    focus: bool,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SplitFile {
    Right,
    Down,
}

/// A config's item, checked.
#[derive(Clone, Debug)]
struct Item {
    kind: ItemKind,
    title: Option<String>,
    cwd: Option<String>,
    split: Option<SplitDirection>,
    focus: bool,
}

#[derive(Clone, Debug)]
enum ItemKind {
    /// A shell with the command typed once it is ready; an empty command leaves the shell.
    Terminal(String),
    Agent(AgentKind),
    Browser(String),
}

/// A launch config: its name and its items.
#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub(crate) name: String,
    items: Vec<Item>,
}

/// A project's configs as its + menu shows them, or why its file could not be read.
pub(crate) type Configs = Result<Vec<Config>, String>;

/// The configs of each project folder that has the file, read when a workspace opens the folder
/// and again when the file changes.
#[derive(Default)]
struct LaunchConfigs(HashMap<PathBuf, Configs>);

impl Global for LaunchConfigs {}

/// Reads the file of each workspace's folders as it opens and when it changes; [`crate::init`]
/// calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(LaunchConfigs::default());
    cx.observe_new(
        |workspace: &mut Workspace, window, cx: &mut Context<Workspace>| {
            let Some(window) = window else {
                return;
            };
            let project = workspace.project().clone();
            read_folders(&project, cx);
            cx.subscribe_in(
                &project,
                window,
                |_, project, event: &project::Event, _, cx| {
                    if touches_the_file(event) {
                        read_folders(project, cx);
                    }
                },
            )
            .detach();
        },
    )
    .detach();
}

/// Whether `event` may have changed a folder's launch file: a folder added, or the file among a
/// folder's changed entries.
fn touches_the_file(event: &project::Event) -> bool {
    match event {
        project::Event::WorktreeAdded(_) => true,
        project::Event::WorktreeUpdatedEntries(_, entries) => entries
            .iter()
            .any(|(path, _, _)| path.as_unix_str() == LAUNCH_FILE),
        _ => false,
    }
}

/// Reads the launch file of each of `project`'s folders off the main thread and keeps what it
/// finds.
fn read_folders(project: &Entity<Project>, cx: &App) {
    let project = project.read(cx);
    let fs = Arc::clone(project.fs());
    let roots: Vec<PathBuf> = project
        .visible_worktrees(cx)
        .map(|worktree| worktree.read(cx).abs_path().to_path_buf())
        .collect();
    for root in roots {
        let fs = Arc::clone(&fs);
        cx.spawn(async move |cx| {
            let configs = read(&root, fs.as_ref()).await;
            if let Some(Err(error)) = &configs {
                log::warn!("{} in {}: {error}", LAUNCH_FILE, root.display());
            }
            cx.update(|cx| {
                let kept = &mut cx.default_global::<LaunchConfigs>().0;
                match configs {
                    Some(configs) => {
                        kept.insert(root, configs);
                    }
                    None => {
                        kept.remove(&root);
                    }
                }
            });
        })
        .detach();
    }
}

/// The configs in `root`'s launch file, none without one.
async fn read(root: &Path, fs: &dyn Fs) -> Option<Configs> {
    let file = root.join(LAUNCH_FILE);
    match fs.metadata(&file).await {
        Ok(Some(metadata)) if !metadata.is_dir => {}
        _ => return None,
    }
    Some(match fs.load(&file).await {
        Ok(text) => parse(&text),
        Err(error) => Err(format!("{error:#}")),
    })
}

/// The configs of a launch file's text, in the file's order, or its first error.
fn parse(text: &str) -> Configs {
    let file: LaunchFile =
        settings::parse_json_with_comments(text).map_err(|error| format!("{error:#}"))?;
    file.launch
        .into_iter()
        .map(|(name, config)| checked(&name, config).map_err(|why| format!("{name}: {why}")))
        .collect()
}

/// A config as the file gave it, checked: each item one kind, a known agent, an `http` or `https`
/// URL, a folder inside the project, at most one item with the focus.
fn checked(name: &str, config: ConfigFile) -> Result<Config, String> {
    let mut focused = false;
    let mut items = Vec::with_capacity(config.items.len());
    for (index, item) in config.items.into_iter().enumerate() {
        let number = index + 1;
        let kind = match (item.terminal, item.agent, item.browser) {
            (Some(command), None, None) => ItemKind::Terminal(command),
            (None, Some(agent), None) => ItemKind::Agent(
                AgentKind::ALL
                    .into_iter()
                    .find(|kind| kind.program() == agent)
                    .ok_or_else(|| format!("item {number}: no agent CLI named {agent:?}"))?,
            ),
            (None, None, Some(url)) => ItemKind::Browser(
                marley_browser::address::agent_url(&url)
                    .map_err(|why| format!("item {number}: {why}"))?,
            ),
            _ => {
                return Err(format!(
                    "item {number}: needs one of terminal, agent and browser"
                ));
            }
        };
        if let Some(cwd) = &item.cwd {
            let inside = Path::new(cwd)
                .components()
                .all(|part| matches!(part, Component::Normal(_) | Component::CurDir));
            if !inside {
                return Err(format!(
                    "item {number}: cwd {cwd:?} is not a folder inside the project"
                ));
            }
        }
        if item.focus && std::mem::replace(&mut focused, true) {
            return Err(format!("item {number}: a second item with the focus"));
        }
        items.push(Item {
            kind,
            title: item.title,
            cwd: item.cwd,
            split: item.split.map(|split| match split {
                SplitFile::Right => SplitDirection::Right,
                SplitFile::Down => SplitDirection::Down,
            }),
            focus: item.focus,
        });
    }
    if items.is_empty() {
        return Err("no items".to_string());
    }
    Ok(Config {
        name: name.to_string(),
        items,
    })
}

/// The configs kept for the project folder `root`; none when it has no launch file.
pub(crate) fn configs(root: &Path, cx: &App) -> Option<Configs> {
    cx.try_global::<LaunchConfigs>()?.0.get(root).cloned()
}

/// What a config does, one line an item, as the approval shows it and its hash covers.
fn text(config: &Config) -> String {
    config
        .items
        .iter()
        .map(|item| {
            let kind = match &item.kind {
                ItemKind::Terminal(_) => "terminal".to_string(),
                ItemKind::Agent(kind) => format!("agent {}", kind.program()),
                ItemKind::Browser(_) => "browser".to_string(),
            };
            let title = item
                .title
                .as_ref()
                .map(|title| format!(" {title:?}"))
                .unwrap_or_default();
            let cwd = item
                .cwd
                .as_ref()
                .map(|cwd| format!(" in {cwd}"))
                .unwrap_or_default();
            let split = match item.split {
                Some(SplitDirection::Right) => ", split right",
                Some(SplitDirection::Down) => ", split down",
                _ => "",
            };
            let what = match &item.kind {
                ItemKind::Terminal(command) if command.is_empty() => ": a shell".to_string(),
                ItemKind::Terminal(command) => format!(": {command}"),
                ItemKind::Browser(url) => format!(": {url}"),
                ItemKind::Agent(_) => String::new(),
            };
            format!("{kind}{title}{cwd}{split}{what}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Runs the config `name` of the project folder `root` in `workspace`: Zed's prompt with its text
/// unless that exact text was approved before, then its items in order. A refusal opens nothing
/// and records nothing.
pub(crate) fn run(
    workspace: WeakEntity<Workspace>,
    root: PathBuf,
    name: String,
    window: &Window,
    cx: &App,
) {
    let Some(Ok(configs)) = configs(&root, cx) else {
        return;
    };
    let Some(config) = configs.into_iter().find(|config| config.name == name) else {
        return;
    };
    let text = text(&config);
    let hash = format!("{:x}", Sha256::digest(text.as_bytes()));
    let key = format!("{}\n{name}", root.display());
    let store = KeyValueStore::global(cx);
    let approved = store.scoped(APPROVALS).read(&key).log_err().flatten();
    let failure = format!("Could not open {name}");
    window
        .spawn(cx, async move |cx| {
            if approved.as_deref() != Some(hash.as_str()) {
                let question = if approved.is_some() {
                    format!("{name} changed since you approved it. Run it?")
                } else {
                    format!("Run {name}?")
                };
                let answer = cx.update(|window, cx| {
                    window.prompt(
                        PromptLevel::Warning,
                        &question,
                        Some(&text),
                        &["Run", "Cancel"],
                        cx,
                    )
                })?;
                if !matches!(answer.await, Ok(0)) {
                    return Ok(());
                }
                store.scoped(APPROVALS).write(key, hash).await?;
            }
            open_items(&workspace, &root, config.items, cx).await
        })
        .detach_and_prompt_err(&failure, window, cx, |_, _, _| None);
}

/// Opens `items` in order in `workspace`, each once the one before exists: each in the active
/// pane, then, when it splits, moved into a new pane split off the previous item's, since the
/// active pane follows a focus change that may not have landed yet. The item marked for the
/// focus, else the first, has it at the end.
async fn open_items(
    workspace: &WeakEntity<Workspace>,
    root: &Path,
    items: Vec<Item>,
    cx: &mut AsyncWindowContext,
) -> anyhow::Result<()> {
    let mut previous: Option<Entity<Pane>> = None;
    let mut first: Option<FocusHandle> = None;
    let mut focus: Option<FocusHandle> = None;
    for item in items {
        let Some(opened) = open_item(workspace, root, &item, cx).await? else {
            continue;
        };
        let pane = workspace.update_in(cx, |workspace, window, cx| {
            let pane = workspace.pane_for(opened.as_ref())?;
            match (item.split, &previous) {
                (Some(direction), Some(previous)) => {
                    let split = workspace.split_pane(previous.clone(), direction, window, cx);
                    workspace::move_item(&pane, &split, opened.item_id(), 0, true, window, cx);
                    Some(split)
                }
                _ => Some(pane),
            }
        })?;
        previous = pane.or(previous);
        let handle = cx.update(|_, cx| opened.item_focus_handle(cx))?;
        if item.focus {
            focus = Some(handle.clone());
        }
        first.get_or_insert(handle);
    }
    if let Some(handle) = focus.or(first) {
        cx.update(|window, cx| window.focus(&handle, cx))?;
    }
    Ok(())
}

/// Opens one item in the active pane of `workspace` and gives its tab; none when its terminal
/// could not be found once it opened.
async fn open_item(
    workspace: &WeakEntity<Workspace>,
    root: &Path,
    item: &Item,
    cx: &mut AsyncWindowContext,
) -> anyhow::Result<Option<Box<dyn ItemHandle>>> {
    let directory = item
        .cwd
        .as_ref()
        .map_or_else(|| root.to_path_buf(), |cwd| root.join(cwd));
    let input = match &item.kind {
        ItemKind::Browser(url) => {
            if let Some(local) = marley_browser::address::local_url(url) {
                wait_for_port(local, cx).await;
            }
            let view = workspace.update_in(cx, |workspace, window, cx| {
                browser::open_url_tab(workspace, url.clone(), window, cx)
            })?;
            return Ok(Some(Box::new(view)));
        }
        ItemKind::Terminal(command) if command.is_empty() => None,
        ItemKind::Terminal(command) => Some(marley_agent::send_payload(command)),
        ItemKind::Agent(kind) => Some(workspace.read_with(cx, |workspace, cx| {
            agents::launch_input(workspace, *kind, cx)
        })?),
    };
    let opening = workspace.update_in(cx, |workspace, window, cx| {
        agents::start_in_terminal(workspace, Some(directory), input, window, cx)
    })?;
    let terminal = opening.await?;
    let title = item.title.clone();
    workspace.update(cx, |workspace, cx| {
        let view = workspace
            .items_of_type::<TerminalView>(cx)
            .find(|view| view.read(cx).terminal().entity_id() == terminal.entity_id())?;
        if let Some(title) = title {
            view.update(cx, |view, cx| view.set_custom_title(Some(title), cx));
        }
        Some(Box::new(view) as Box<dyn ItemHandle>)
    })
}

/// Waits until the port of `local` accepts a connection, trying every [`PORT_RETRY`] for at most
/// [`PORT_WAIT`]; after that the page loads anyway, so the browser shows why it failed.
async fn wait_for_port(local: LocalUrl, cx: &AsyncWindowContext) {
    let executor = cx.background_executor().clone();
    let deadline = executor.now() + PORT_WAIT;
    loop {
        let accepted = executor
            .spawn({
                let local = local.clone();
                async move { marley_browser::address::accepts(&local).await }
            })
            .await;
        if accepted || executor.now() >= deadline {
            return;
        }
        executor.timer(PORT_RETRY).await;
    }
}
