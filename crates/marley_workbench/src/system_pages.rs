//! The System tab (#855, #885): pages about Marley itself.
//!
//! Each page is a card as the `SystemPages` board draws one: a title, what it covers, a lead line,
//! rows with a mark, and buttons. Folder sizes and Troubleshooting's checks are read off the main
//! thread when the tab opens and again on their buttons; nothing on a page writes or starts
//! anything.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gpui::{
    App, ClipboardItem, Context, EventEmitter, FocusHandle, Focusable, SharedString, Stateful,
    Task, Window, actions,
};
use marley_agent::AgentKind;
use marley_agent::versions::Found;
use release_channel::{AppCommitSha, AppVersion, ReleaseChannel};
use settings::Settings as _;
use ui::prelude::*;
use util::ResultExt as _;
use workspace::Workspace;
use workspace::item::{Item, ItemEvent};

use crate::fleet::{Fleet, SourceState};
use crate::prompts::{Role, render_button};

actions!(
    marley,
    [
        /// Opens the System tab: which build runs, where Marley keeps its data, what it checks,
        /// and the servers and browsers it runs.
        #[derive(Eq)]
        OpenSystemPages
    ]
);

/// A folder past this many bytes gets the warning colour (#855, D1).
const LARGE_FOLDER_BYTES: u64 = 500 * 1024 * 1024;

/// Registers the open action on every workspace.
pub(crate) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &OpenSystemPages, window, cx| {
            open(workspace, window, cx);
        });
    })
    .detach();
}

/// Brings the workspace's System tab forward, or opens one in the center.
fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let open = workspace.items_of_type::<SystemPages>(cx).next();
    if let Some(tab) = open {
        workspace.activate_item(&tab, true, true, window, cx);
        return;
    }
    let tab = cx.new(SystemPages::new);
    workspace.add_item_to_center(Box::new(tab), window, cx);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Build,
    Data,
    Install,
    Trouble,
    Shell,
    Mcp,
    Chromium,
    Fleet,
    Developers,
}

impl Page {
    const ALL: [Self; 9] = [
        Self::Build,
        Self::Data,
        Self::Install,
        Self::Trouble,
        Self::Shell,
        Self::Mcp,
        Self::Chromium,
        Self::Fleet,
        Self::Developers,
    ];

    const fn title(self) -> &'static str {
        match self {
            Self::Build => "This build",
            Self::Data => "Where data lives",
            Self::Install => "Install and update",
            Self::Trouble => "Troubleshooting",
            Self::Shell => "Shell integration",
            Self::Mcp => "Marley's MCP endpoint",
            Self::Chromium => "Marley's Chromium",
            Self::Fleet => "Fleet providers",
            Self::Developers => "For developers",
        }
    }

    const fn covers(self) -> &'static str {
        match self {
            Self::Build => "Which build · Its data folder",
            Self::Data => "Where data lives",
            Self::Install => "Install · The running build",
            Self::Trouble => "What Marley needs, checked",
            Self::Shell => "The shells · How an agent is told apart",
            Self::Mcp => "Where it listens · Who reaches it",
            Self::Chromium => "One browser per project",
            Self::Fleet => "Real stores",
            Self::Developers => "Where things are · What is planned",
        }
    }

    const fn lead(self) -> &'static str {
        match self {
            Self::Build => "Which build runs, on which data folder.",
            Self::Data => {
                "Marley's folders, their size, and what each holds. Nothing here leaves the machine."
            }
            Self::Install => {
                "An install builds Marley and puts it in place. The Marley running keeps its build \
                 until you restart it."
            }
            Self::Trouble => "Marley checks what it needs and says what it found.",
            Self::Shell => {
                "What each shell gets from Marley: blocks, their exit codes and their folders."
            }
            Self::Mcp => "The server agents talk to: where it listens and who may reach it.",
            Self::Chromium => {
                "The browser behind the Browser tab: one profile per project, started on first use."
            }
            Self::Fleet => "The stores the Fleet panel reads, and how each answered.",
            Self::Developers => "Where Marley's code and plans are, for working on it.",
        }
    }
}

/// A row's mark (#885, D1): found and working, working with a gap, missing or broken, or off by
/// choice.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    Ok,
    Warn,
    Bad,
    Idle,
}

/// One row of a page: its mark, its name over its detail, and its value at the right.
struct Row {
    mark: Mark,
    name: SharedString,
    detail: SharedString,
    value: SharedString,
}

impl Row {
    fn new(
        mark: Mark,
        name: impl Into<SharedString>,
        detail: impl Into<SharedString>,
        value: impl Into<SharedString>,
    ) -> Self {
        Self {
            mark,
            name: name.into(),
            detail: detail.into(),
            value: value.into(),
        }
    }

    /// The row as a line of a copied report.
    fn line(&self) -> String {
        format!("{}: {} {}", self.name, self.detail, self.value)
            .trim_end()
            .to_string()
    }
}

/// One of Marley's folders on Where data lives, and its size once counted.
struct Folder {
    name: &'static str,
    path: PathBuf,
    /// None while counting; zero bytes and `present` false for a folder not made yet.
    bytes: Option<u64>,
    present: bool,
}

/// What Troubleshooting reads off the main thread: whether git is found, and the agent CLIs on
/// Marley's search path.
struct Checks {
    git: bool,
    clis: Vec<AgentKind>,
}

/// The folders Where data lives lists, in the order it lists them.
fn folders() -> Vec<(&'static str, PathBuf)> {
    let data = paths::data_dir();
    vec![
        ("Settings", paths::config_dir().clone()),
        ("Database", paths::database_dir().clone()),
        ("Logs", paths::logs_dir().clone()),
        ("Terminal history", data.join("terminal_history")),
        ("Browser profiles", data.join("browser")),
        ("Handoffs", data.join("handoffs")),
        ("Extensions", paths::extensions_dir().clone()),
        ("Language servers", paths::languages_dir().clone()),
    ]
}

/// The bytes of the files under `path`, symlinks not followed; zero for a missing folder.
/// Blocking: it runs on the background executor.
fn folder_size(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| match entry.file_type() {
            Ok(kind) if kind.is_dir() => folder_size(&entry.path()),
            Ok(kind) if kind.is_file() => entry.metadata().map_or(0, |metadata| metadata.len()),
            _ => 0,
        })
        .sum()
}

/// `bytes` as the board writes a size: "84 KB", "42 MB", "1.2 GB".
fn size_text(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    match bytes {
        0 => "empty".to_string(),
        bytes if bytes < MB => format!("{} KB", bytes.div_ceil(KB)),
        bytes if bytes < GB => format!("{} MB", bytes / MB),
        bytes => format!("{}.{} GB", bytes / GB, (bytes % GB) * 10 / GB),
    }
}

/// This build's rows.
fn build_rows(cx: &App) -> Vec<Row> {
    let commit = AppCommitSha::try_global(cx).map_or_else(
        || SharedString::new_static("not recorded"),
        |sha| sha.short().into(),
    );
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let platform = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    vec![
        Row::new(
            Mark::Idle,
            "Channel",
            ReleaseChannel::global(cx).display_name(),
            AppVersion::global(cx).to_string(),
        ),
        Row::new(Mark::Idle, "Commit", commit, ""),
        Row::new(
            Mark::Idle,
            "Data folder",
            paths::data_dir().display().to_string(),
            "",
        ),
        Row::new(Mark::Idle, "Platform", platform, desktop),
    ]
}

/// Install and update's rows: the running build and its binary, and how an install goes.
/// What `script/install-marley` wrote at the end of its last full install (#945).
#[derive(Debug, Clone, serde::Deserialize)]
struct InstallRecord {
    commit: String,
    #[serde(default)]
    dirty: bool,
    #[serde(default)]
    ended: String,
    binary: PathBuf,
    #[serde(default)]
    steps: Vec<InstallStep>,
}

/// One step of an install, with what it gave and how it went.
#[derive(Debug, Clone, serde::Deserialize)]
struct InstallStep {
    name: String,
    value: String,
    result: String,
}

/// The install record, and whether this window runs the binary it installed.
struct InstallState {
    record: InstallRecord,
    running_is_installed: bool,
}

/// The install record in `data_dir`, read off the main thread; none when there is none or it
/// does not read.
fn read_install_in(data_dir: &Path) -> Option<InstallState> {
    let text = std::fs::read_to_string(data_dir.join("install.json")).ok()?;
    let record: InstallRecord = serde_json::from_str(&text).log_err()?;
    let running = std::env::current_exe()
        .ok()
        .and_then(|path| path.canonicalize().ok());
    let installed = record.binary.canonicalize().ok();
    let running_is_installed = running.is_some() && running == installed;
    Some(InstallState {
        record,
        running_is_installed,
    })
}

/// Install and update's rows: each step of the last install with its result, and a warning
/// while this window runs an older build than the one installed (#945); without a record, what
/// this build is and how it is installed.
fn install_rows(state: Option<&InstallState>, cx: &App) -> Vec<Row> {
    if let Some(state) = state {
        let record = &state.record;
        let short = |commit: &str| commit.chars().take(7).collect::<String>();
        let mut rows: Vec<Row> = record
            .steps
            .iter()
            .map(|step| {
                let mark = if step.result == "ok" {
                    Mark::Ok
                } else {
                    Mark::Bad
                };
                Row::new(
                    mark,
                    step.name.clone(),
                    step.value.clone(),
                    step.result.clone(),
                )
            })
            .collect();
        let changes = if record.dirty {
            ", with changes not committed"
        } else {
            ""
        };
        rows.insert(
            0,
            Row::new(
                Mark::Idle,
                "Installed",
                format!(
                    "commit {}{changes} · {}",
                    short(&record.commit),
                    record.ended
                ),
                "",
            ),
        );
        let running_commit = AppCommitSha::try_global(cx).map(|sha| sha.full());
        if state.running_is_installed
            && running_commit
                .as_deref()
                .is_some_and(|commit| commit != record.commit)
        {
            let still_on = running_commit.as_deref().map(short).unwrap_or_default();
            rows.push(Row::new(
                Mark::Warn,
                "Running Marley",
                format!("still on {still_on}"),
                "restart",
            ));
        }
        return rows;
    }
    // With no install record, what this build is (#885).
    let commit =
        AppCommitSha::try_global(cx).map_or_else(|| "not recorded".to_string(), |sha| sha.short());
    let binary = std::env::current_exe().map_or_else(
        |error| format!("unknown: {error}"),
        |path| path.display().to_string(),
    );
    vec![
        Row::new(
            Mark::Ok,
            "Running",
            format!("commit {commit}"),
            "this build",
        ),
        Row::new(Mark::Idle, "Its binary", binary, ""),
        Row::new(
            Mark::Idle,
            "Install",
            "script/install-marley in Marley's repository builds the release and puts it in place",
            "",
        ),
        Row::new(
            Mark::Idle,
            "After an install",
            "this window keeps its build; Marley starts the new one when you restart it",
            "restart",
        ),
    ]
}

/// A CLI's check from the version Marley read (#648): its version, why it could not be read, or
/// that it is missing; none while the read has not answered.
fn version_row(name: &'static str, found: Option<Found>) -> Row {
    match found {
        Some(Found::Version(version)) => Row::new(Mark::Ok, name, version.to_string(), "found"),
        Some(Found::Unreadable(why)) => Row::new(Mark::Warn, name, why, "unreadable"),
        Some(Found::Missing(where_looked)) => Row::new(Mark::Bad, name, where_looked, "missing"),
        None => Row::new(Mark::Idle, name, "not read yet", "checking"),
    }
}

/// Troubleshooting's rows (D1): the shells, Rusty, the agent CLIs, the plugin and git.
fn trouble_rows(checks: Option<&Checks>, cx: &App) -> Vec<Row> {
    let mut rows = vec![Row::new(
        Mark::Ok,
        "Shell integration",
        "bash · zsh · fish",
        "on",
    )];
    rows.push(
        if !cx.has_global::<crate::rusty::Rusty>() || !crate::rusty::is_on(cx) {
            Row::new(Mark::Idle, "Rusty", "marley.rusty is off", "off")
        } else if crate::rusty::is_connected(cx) {
            Row::new(Mark::Ok, "Rusty", "its MCP server answers", "connected")
        } else {
            let why = crate::rusty::unavailable(cx)
                .unwrap_or_else(|| SharedString::new_static("connecting"));
            Row::new(Mark::Warn, "Rusty", why, "not connected")
        },
    );
    rows.push(version_row(
        "Claude Code",
        crate::agent_versions::found(AgentKind::Claude, cx),
    ));
    rows.push(version_row(
        "Codex",
        crate::agent_versions::found(AgentKind::Codex, cx),
    ));
    rows.push(
        match cx.try_global::<crate::claude_plugin::ClaudePlugin>() {
            Some(plugin) if plugin.installed == Some(true) && plugin.needs_update() => Row::new(
                Mark::Warn,
                "Claude Code plugin",
                plugin.installed_version.clone().unwrap_or_default(),
                "update ready",
            ),
            Some(plugin) if plugin.installed == Some(true) => Row::new(
                Mark::Ok,
                "Claude Code plugin",
                plugin.installed_version.clone().unwrap_or_default(),
                "installed",
            ),
            Some(plugin) if plugin.installed == Some(false) => Row::new(
                Mark::Warn,
                "Claude Code plugin",
                "Marley's plugin is not installed in Claude Code",
                "not installed",
            ),
            _ => Row::new(Mark::Idle, "Claude Code plugin", "not read yet", "checking"),
        },
    );
    match checks {
        None => rows.push(Row::new(
            Mark::Idle,
            "Other agent CLIs and git",
            "",
            "checking",
        )),
        Some(checks) => {
            for kind in [AgentKind::Gemini, AgentKind::OpenCode] {
                rows.push(if checks.clis.contains(&kind) {
                    Row::new(Mark::Ok, kind.display_name(), kind.program(), "found")
                } else {
                    Row::new(
                        Mark::Idle,
                        kind.display_name(),
                        format!("no {} on Marley's search path", kind.program()),
                        "not installed",
                    )
                });
            }
            rows.push(if checks.git {
                Row::new(Mark::Ok, "git", "on Marley's search path", "found")
            } else {
                Row::new(
                    Mark::Bad,
                    "git",
                    "no git on Marley's search path",
                    "missing",
                )
            });
        }
    }
    rows
}

/// Shell integration's rows: the shells Marley integrates and the rest.
fn shell_rows() -> Vec<Row> {
    vec![
        Row::new(
            Mark::Ok,
            "bash",
            "Marley's --rcfile, which reads your ~/.bashrc first",
            "on",
        ),
        Row::new(
            Mark::Ok,
            "zsh",
            "Marley's ZDOTDIR, which puts yours back, so zsh reads your own files",
            "on",
        ),
        Row::new(
            Mark::Ok,
            "fish",
            "Marley's vendor snippet, which puts your XDG_DATA_DIRS back before your config.fish",
            "on",
        ),
        Row::new(
            Mark::Idle,
            "Other shells",
            "no blocks; the terminal works as Zed's",
            "off",
        ),
        Row::new(
            Mark::Idle,
            "Agents",
            "Claude Code, Codex, Gemini CLI and OpenCode, told apart by the program a terminal runs",
            "",
        ),
    ]
}

/// The MCP endpoint's rows: whether it runs, where, its address file, and the paired clients.
fn mcp_rows(cx: &App) -> Vec<Row> {
    let Some(url) = crate::clients::server_url(cx) else {
        return vec![Row::new(
            Mark::Bad,
            "Server",
            "Marley's MCP server is not running",
            "stopped",
        )];
    };
    let clients = crate::clients::paired_clients(cx);
    let paired = if clients.is_empty() {
        Row::new(Mark::Idle, "Paired programs", "none yet", "0")
    } else {
        Row::new(
            Mark::Ok,
            "Paired programs",
            clients.join(", "),
            clients.len().to_string(),
        )
    };
    // The endpoint as the System board lists it (#945).
    let bridge = paths::data_dir().join("mcp").join("marley-mcp-bridge");
    let refusal = crate::agent_activity::last_refusal_today(cx).map_or_else(
        || Row::new(Mark::Idle, "Last refusal", "none today", ""),
        |row| {
            let code = match &row.outcome {
                crate::agent_activity::Outcome::Refused { code, .. } => code.clone(),
                _ => String::new(),
            };
            let when = row.time.get(11..16).unwrap_or_default().to_string();
            Row::new(
                Mark::Warn,
                "Last refusal",
                format!("{} · {code} · {}", row.tool, row.summary),
                when,
            )
        },
    );
    let limit = marley_mcp::transport::MAX_BODY_BYTES / (1 << 20);
    vec![
        Row::new(Mark::Ok, "Endpoint", url, "loopback only · up"),
        Row::new(Mark::Idle, "Bridge", bridge.display().to_string(), "stdio"),
        Row::new(
            Mark::Idle,
            "Tools",
            format!(
                "{} tools · {} write classes granted",
                marley_mcp::served_tool_count(),
                crate::mcp::GRANTED_WRITE_CLASSES.len()
            ),
            "",
        ),
        refusal,
        Row::new(
            Mark::Idle,
            "Limits",
            format!("request bodies ≤ {limit} MiB · loopback only"),
            "",
        ),
        paired,
    ]
}

/// Marley's Chromium's rows: each project's browser with its state and pages.
fn chromium_rows(cookie_sites: &HashMap<String, usize>, cx: &App) -> Vec<Row> {
    // The guide's "Limits of the Browser tab", in a line (#945).
    let limits = Row::new(
        Mark::Idle,
        "Limits",
        "no context menus or cursor shapes · soft pages above 1× · downloads to Downloads only",
        "",
    );
    let Some(hub) = crate::browser::BrowserHub::try_global(cx) else {
        return vec![Row::new(Mark::Idle, "Chromium", "", "not started"), limits];
    };
    let hub = hub.read(cx);
    let product = hub
        .projects()
        .iter()
        .find_map(|(project, _)| hub.product_of(&project.key));
    let chromium = product.map_or_else(
        || Row::new(Mark::Idle, "Chromium", "", "not started"),
        |product| Row::new(Mark::Ok, "Chromium", product, "running"),
    );
    let tabs = hub.tabs();
    let rows: Vec<Row> = hub
        .projects()
        .into_iter()
        .map(|(project, state)| {
            let pages = tabs
                .iter()
                .filter(|tab| {
                    hub.project_of(&tab.id)
                        .is_some_and(|owner| owner.key == project.key)
                })
                .count();
            let profile = marley_browser::service::profile_in(
                &marley_browser::service::project_dir_in(paths::data_dir(), &project.key),
            );
            let cookies = cookie_sites
                .get(project.key.as_ref())
                .map(|sites| match sites {
                    1 => " · cookies from 1 site".to_string(),
                    sites => format!(" · cookies from {sites} sites"),
                })
                .unwrap_or_default();
            let pages = match pages {
                1 => "1 page".to_string(),
                pages => format!("{pages} pages"),
            };
            let (mark, value) = match state {
                crate::browser::HubState::Showing => (Mark::Ok, format!("{pages}{cookies}")),
                crate::browser::HubState::Starting | crate::browser::HubState::Connecting => {
                    (Mark::Idle, "starting".to_string())
                }
                crate::browser::HubState::Failed(why) => (Mark::Warn, why.to_string()),
            };
            Row::new(mark, project.name, profile.display().to_string(), value)
        })
        .collect();
    let mut all = vec![chromium];
    if rows.is_empty() {
        all.push(Row::new(
            Mark::Idle,
            "Browsers",
            "no project has started its browser yet",
            "",
        ));
    } else {
        all.extend(rows);
    }
    all.push(limits);
    all
}

/// Fleet providers' rows: each configured store, and how it last answered when the Fleet panel
/// read it.
fn fleet_rows(cx: &App) -> Vec<Row> {
    let providers = &crate::MarleySettings::get_global(cx).fleet_providers;
    if providers.is_empty() {
        return vec![Row::new(
            Mark::Idle,
            "No providers",
            "marley.fleet.providers is empty",
            "",
        )];
    }
    let sources = cx.try_global::<Fleet>().map(|fleet| &fleet.sources);
    providers
        .iter()
        .map(|entry| {
            let name = crate::fleet_providers::name_of(entry);
            let source =
                sources.and_then(|sources| sources.iter().find(|source| source.name == name));
            let (mark, value) = match source.map(|source| &source.state) {
                Some(SourceState::Ready) => (Mark::Ok, "answering".to_string()),
                Some(SourceState::Connecting) => (Mark::Idle, "connecting".to_string()),
                Some(SourceState::Stale) => (Mark::Warn, "stale".to_string()),
                Some(
                    SourceState::Failed(why)
                    | SourceState::Unreachable(why)
                    | SourceState::Incompatible(why),
                ) => (Mark::Bad, why.clone()),
                None => (Mark::Idle, "read when the Fleet panel shows".to_string()),
            };
            Row::new(mark, name, provider_kind(entry), value)
        })
        .collect()
}

/// What kind of store a provider is, as Fleet providers names it.
const fn provider_kind(entry: &settings::FleetProviderContent) -> &'static str {
    match entry {
        settings::FleetProviderContent::Pseudo => "fixtures",
        settings::FleetProviderContent::Mcp { .. } => "an MCP server",
        settings::FleetProviderContent::Http { .. } => "an HTTP endpoint",
        settings::FleetProviderContent::Forge { .. } => "Forge",
    }
}

/// For developers' rows: where Marley's code, rules and plans are.
fn developer_rows() -> Vec<Row> {
    vec![
        Row::new(
            Mark::Idle,
            "The repository",
            "github.com/Ignibyte/marley_ide, a fork of Zed",
            "",
        ),
        Row::new(Mark::Idle, "The docs map", "docs/marley/README.md", ""),
        Row::new(
            Mark::Idle,
            "How work moves",
            "CONSTITUTION.md: Plan, Code, Test, Complete",
            "",
        ),
        Row::new(
            Mark::Idle,
            "Changes to Zed's crates",
            "docs/marley/zed-touchpoints.md",
            "",
        ),
        Row::new(
            Mark::Idle,
            "What is planned",
            "docs/marley/three-prong-plan.md · docs/marley/look-plan.md",
            "",
        ),
    ]
}

/// The System tab.
pub(crate) struct SystemPages {
    page: Page,
    folders: Vec<Folder>,
    checks: Option<Checks>,
    focus_handle: FocusHandle,
    /// The count running or last run; a count asked for while one runs waits for it.
    counting: Option<Task<()>>,
    /// The checks running or last run.
    checking: Option<Task<()>>,
    /// The last install's record, read off the main thread when the tab opens (#945).
    install: Option<InstallState>,
    install_reading: Option<Task<()>>,
    /// How many sites each running project browser keeps cookies for, by project key (#945).
    cookie_sites: HashMap<String, usize>,
    cookie_reads: Vec<Task<()>>,
}

impl SystemPages {
    fn new(cx: &mut Context<Self>) -> Self {
        let mut pages = Self {
            page: Page::Build,
            folders: Vec::new(),
            checks: None,
            focus_handle: cx.focus_handle(),
            counting: None,
            checking: None,
            install: None,
            install_reading: None,
            cookie_sites: HashMap::new(),
            cookie_reads: Vec::new(),
        };
        pages.count(cx);
        pages.check(cx);
        pages.install_reading = Some(cx.spawn(async move |pages, cx| {
            let state = cx
                .background_spawn(futures::future::lazy(
                    |_| read_install_in(paths::data_dir()),
                ))
                .await;
            pages
                .update(cx, |pages, cx| {
                    pages.install = state;
                    cx.notify();
                })
                .log_err();
        }));
        pages
    }

    /// Counts every folder's size again (#855, D2).
    fn count(&mut self, cx: &mut Context<Self>) {
        let running = self.folders.iter().any(|folder| folder.bytes.is_none());
        if self.counting.is_some() && running {
            return;
        }
        self.folders = folders()
            .into_iter()
            .map(|(name, path)| Folder {
                name,
                path,
                bytes: None,
                present: true,
            })
            .collect();
        let paths: Vec<PathBuf> = self
            .folders
            .iter()
            .map(|folder| folder.path.clone())
            .collect();
        self.counting = Some(cx.spawn(async move |pages, cx| {
            let sizes = cx
                .background_spawn(futures::future::lazy(move |_| {
                    paths
                        .iter()
                        .map(|path| (path.exists(), folder_size(path)))
                        .collect::<Vec<_>>()
                }))
                .await;
            pages
                .update(cx, |pages, cx| {
                    for (folder, (present, bytes)) in pages.folders.iter_mut().zip(sizes) {
                        folder.present = present;
                        folder.bytes = Some(bytes);
                    }
                    cx.notify();
                })
                .log_err();
        }));
        cx.notify();
    }

    /// Reads Troubleshooting's checks again (D2): git and the agent CLIs on Marley's search path.
    fn check(&mut self, cx: &mut Context<Self>) {
        if self.checking.is_some() && self.checks.is_none() {
            return;
        }
        self.checks = None;
        let search_path = crate::agents::launcher(cx).search_path;
        self.checking = Some(cx.spawn(async move |pages, cx| {
            let checks = cx
                .background_spawn(futures::future::lazy(move |_| Checks {
                    git: which::which_in("git", search_path.as_deref(), "/").is_ok(),
                    clis: crate::agents::installed_clis(search_path.as_deref()),
                }))
                .await;
            pages
                .update(cx, |pages, cx| {
                    pages.checks = Some(checks);
                    cx.notify();
                })
                .log_err();
        }));
        cx.notify();
    }

    /// Reads how many sites each running project browser keeps cookies for (#945).
    fn read_cookie_sites(&mut self, cx: &Context<Self>) {
        let Some(hub) = crate::browser::BrowserHub::try_global(cx) else {
            return;
        };
        let reads: Vec<(String, Task<anyhow::Result<usize>>)> = {
            let hub = hub.read(cx);
            hub.projects()
                .into_iter()
                .filter_map(|(project, _)| {
                    let key = project.key.to_string();
                    hub.cookie_sites_of(&key, cx).map(|read| (key, read))
                })
                .collect()
        };
        self.cookie_reads = reads
            .into_iter()
            .map(|(key, read)| {
                cx.spawn(async move |pages, cx| {
                    if let Some(sites) = read.await.log_err() {
                        pages
                            .update(cx, |pages, cx| {
                                pages.cookie_sites.insert(key, sites);
                                cx.notify();
                            })
                            .log_err();
                    }
                })
            })
            .collect();
    }

    fn data_rows(&self) -> Vec<Row> {
        self.folders
            .iter()
            .map(|folder| {
                let large = folder.bytes.is_some_and(|bytes| bytes > LARGE_FOLDER_BYTES);
                let value: SharedString = match folder.bytes {
                    None => SharedString::new_static("counting…"),
                    Some(_) if !folder.present => SharedString::new_static("none yet"),
                    Some(bytes) => size_text(bytes).into(),
                };
                Row::new(
                    if large { Mark::Warn } else { Mark::Idle },
                    folder.name,
                    folder.path.display().to_string(),
                    value,
                )
            })
            .collect()
    }

    fn rows(&self, cx: &App) -> Vec<Row> {
        match self.page {
            Page::Build => build_rows(cx),
            Page::Data => self.data_rows(),
            Page::Install => install_rows(self.install.as_ref(), cx),
            Page::Trouble => trouble_rows(self.checks.as_ref(), cx),
            Page::Shell => shell_rows(),
            Page::Mcp => mcp_rows(cx),
            Page::Chromium => chromium_rows(&self.cookie_sites, cx),
            Page::Fleet => fleet_rows(cx),
            Page::Developers => developer_rows(),
        }
    }

    fn render_list(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        v_flex()
            .flex_none()
            .w(px(220.))
            .h_full()
            .p_2()
            .gap_0p5()
            .border_r_1()
            .border_color(colors.border_variant)
            .bg(colors.panel_background)
            .child(
                div().px_2().pb_1().child(
                    Label::new("System")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
            )
            .children(Page::ALL.into_iter().map(|page| {
                let chosen = page == self.page;
                h_flex()
                    .id(SharedString::from(format!(
                        "marley-system-page-{}",
                        page.title()
                    )))
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .cursor_pointer()
                    .when(chosen, |row| row.bg(colors.element_selected))
                    .hover(|style| style.bg(colors.ghost_element_hover))
                    .on_click(cx.listener(move |pages, _, _, cx| {
                        pages.page = page;
                        if page == Page::Chromium {
                            pages.read_cookie_sites(cx);
                        }
                        cx.notify();
                    }))
                    .child(
                        Label::new(page.title())
                            .size(LabelSize::Small)
                            .color(if chosen { Color::Default } else { Color::Muted }),
                    )
            }))
    }

    /// The page's buttons.
    fn render_buttons(&self, cx: &Context<Self>) -> Vec<Stateful<Div>> {
        let button = |index: usize, label: &'static str, role: Role| {
            render_button(index, label, role, false, cx).cursor_pointer()
        };
        match self.page {
            Page::Build => vec![
                button(0, "Copy build info", Role::Plain)
                    .on_click(cx.listener(|pages, _, _, cx| pages.copy_report(cx))),
            ],
            Page::Data => vec![
                button(0, "Open the folder", Role::Plain).on_click(cx.listener(|_, _, _, cx| {
                    cx.reveal_path(paths::data_dir());
                })),
                button(1, "Count again", Role::Quiet)
                    .on_click(cx.listener(|pages, _, _, cx| pages.count(cx))),
            ],
            Page::Install => vec![button(0, "Open the log", Role::Plain).on_click(cx.listener(
                |_, _, window, cx| {
                    window.dispatch_action(Box::new(workspace::OpenLog), cx);
                },
            ))],
            Page::Trouble => vec![
                button(0, "Run the checks again", Role::Plain)
                    .on_click(cx.listener(|pages, _, _, cx| pages.check(cx))),
                button(1, "Copy a report", Role::Quiet)
                    .on_click(cx.listener(|pages, _, _, cx| pages.copy_report(cx))),
            ],
            // The System board's buttons for the other pages (#945).
            Page::Shell => vec![
                button(0, "Open the guide", Role::Plain)
                    .on_click(cx.listener(|_, _, window, cx| open_guide(window, cx))),
            ],
            Page::Mcp => vec![
                button(0, "Open Agent Activity", Role::Plain).on_click(cx.listener(
                    |_, _, window, cx| {
                        window.dispatch_action(
                            Box::new(crate::agent_activity::OpenAgentActivity),
                            cx,
                        );
                    },
                )),
                button(1, "Copy the endpoint", Role::Quiet).on_click(cx.listener(|_, _, _, cx| {
                    if let Some(url) = crate::clients::server_url(cx) {
                        cx.write_to_clipboard(ClipboardItem::new_string(url));
                    }
                })),
            ],
            Page::Chromium => Vec::new(),
            Page::Fleet => vec![
                button(0, "Add a provider", Role::Plain).on_click(cx.listener(
                    |_, _, window, cx| {
                        window.dispatch_action(Box::new(zed_actions::OpenSettingsFile), cx);
                    },
                )),
                button(1, "Open Settings", Role::Quiet).on_click(cx.listener(
                    |_, _, window, cx| {
                        window.dispatch_action(
                            Box::new(zed_actions::OpenSettingsAt {
                                path: "marley".to_string(),
                                target: None,
                            }),
                            cx,
                        );
                    },
                )),
            ],
            Page::Developers => vec![
                button(0, "Open the plan", Role::Plain).on_click(cx.listener(|_, _, _, cx| {
                    cx.open_url(PLAN_URL);
                })),
                button(1, "Open the guide", Role::Quiet)
                    .on_click(cx.listener(|_, _, window, cx| open_guide(window, cx))),
            ],
        }
    }

    /// Puts the page's rows on the clipboard, a line each.
    fn copy_report(&self, cx: &App) {
        let text = self
            .rows(cx)
            .iter()
            .map(Row::line)
            .collect::<Vec<_>>()
            .join("\n");
        cx.write_to_clipboard(ClipboardItem::new_string(text));
    }

    fn render_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let page = self.page;
        let rows: Vec<AnyElement> = self
            .rows(cx)
            .into_iter()
            .map(|row| render_row(row, cx))
            .collect();
        let buttons = self.render_buttons(cx);
        v_flex()
            .id("marley-system-page")
            .flex_1()
            .min_w_0()
            .h_full()
            .overflow_y_scroll()
            .p_6()
            .child(
                v_flex()
                    .max_w(px(760.))
                    .gap_3()
                    .p_4()
                    .rounded_lg()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.elevated_surface_background)
                    .child(
                        h_flex()
                            .gap_3()
                            .items_baseline()
                            .child(Headline::new(page.title()).size(HeadlineSize::Small))
                            .child(
                                Label::new(page.covers())
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            ),
                    )
                    .child(
                        Label::new(page.lead())
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .child(v_flex().gap_0p5().children(rows))
                    .when(!buttons.is_empty(), |card| {
                        card.child(h_flex().gap_2().children(buttons))
                    }),
            )
    }
}

/// A page's row as the board draws it: its mark's dot, the name over its detail, and the value at
/// the right, coloured when the mark warns or says something is wrong.
fn render_row(row: Row, cx: &App) -> AnyElement {
    let colors = cx.theme().colors();
    let status = cx.theme().status();
    let (dot, ink) = match row.mark {
        Mark::Ok => (status.success, Color::Muted),
        Mark::Warn => (status.warning, Color::Warning),
        Mark::Bad => (status.error, Color::Error),
        Mark::Idle => (colors.text_muted, Color::Muted),
    };
    h_flex()
        .gap_3()
        .px_2()
        .py_1p5()
        .rounded_md()
        .bg(colors.surface_background)
        .child(div().flex_none().size(px(7.)).rounded_full().bg(dot))
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .child(Label::new(row.name).size(LabelSize::Small))
                .child(
                    Label::new(row.detail)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .buffer_font(cx)
                        .truncate(),
                ),
        )
        .child(
            div().flex_none().max_w(rems(16.)).child(
                Label::new(row.value)
                    .size(LabelSize::Small)
                    .color(ink)
                    .truncate(),
            ),
        )
        .into_any_element()
}

impl Focusable for SystemPages {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for SystemPages {}

impl Render for SystemPages {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .key_context("MarleySystemPages")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(self.render_list(cx))
            .child(self.render_page(cx))
    }
}

impl Item for SystemPages {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("System")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Info))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static(
            "Which build runs, where Marley keeps its data, and what it checks",
        ))
    }

    fn to_item_events(event: &Self::Event, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}

/// The public plan Marley follows, for For developers (#945).
const PLAN_URL: &str = "https://github.com/Ignibyte/marley_ide/blob/marley/workbench-shell/docs/marley/three-prong-plan.md";

/// Opens Marley's guide in a tab.
fn open_guide(window: &mut Window, cx: &mut App) {
    window.dispatch_action(Box::new(crate::OpenGuide), cx);
}
