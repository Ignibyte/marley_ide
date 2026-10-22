//! MCP Servers (#720): every MCP server each agent sees, and what Marley needs of each agent CLI,
//! in a center tab.
//!
//! Claude Code's servers come from its `.claude.json` (the user scope, and the local scope of each
//! open project's folder) and each open project's `.mcp.json`; Codex's from its `config.toml`;
//! Zed's from the open projects' context server stores. The files are read off the main thread
//! when the tab opens, when its action brings it forward, and on Refresh.
//!
//! No env or header value reaches a row: the CLIs' maps are read for their names alone. A command
//! line shows with keys and tokens masked by the built-in redaction rules, and a URL with its
//! secret-looking query values hidden, since an argument can carry a token as an env value can.
//! Claude Code's and Codex's servers are added, changed and removed from the tab through
//! `mcp_server_form` (#768); Zed's own MCP Servers page edits Zed's servers, and Configure in
//! Settings opens it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable, SharedString,
    Subscription, Task, WeakEntity, Window, actions,
};
use marley_agent::AgentKind;
use marley_agent::versions::{self, Found, integrations_of};
use marley_mcp::redact::Redactor;
use project::Project;
use project::context_server_store::{ContextServerStatus, ServerStatusChangedEvent};
use project::project_settings::ContextServerSettings;
use serde::Deserialize;
use serde::de::IgnoredAny;
use serde_json::value::RawValue;
use ui::{ButtonStyle, Headline, HeadlineSize, prelude::*};
use util::ResultExt as _;
use workspace::Workspace;
use workspace::item::{Item, ItemEvent};

use crate::home_page::muted;
use crate::mcp_server_form::{self, Kind, McpServerForm, Place, Stored, entry_hash};

actions!(
    marley,
    [
        /// Opens MCP Servers: every MCP server Claude Code, Codex and Zed's agents see, and what
        /// Marley needs of each agent CLI, in a tab of the shown group.
        #[derive(Eq)]
        OpenMcpServers,
    ]
);

pub(crate) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &OpenMcpServers, window, cx| {
            open(workspace, window, cx);
        });
    })
    .detach();
}

/// Brings `workspace`'s MCP Servers forward and reads the files again, or opens one in its center.
pub(crate) fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let open_page = workspace.items_of_type::<McpServers>(cx).next();
    if let Some(page) = open_page {
        workspace.activate_item(&page, true, true, window, cx);
        page.update(cx, |page, cx| page.refresh(window, cx));
        return;
    }
    let handle = workspace.weak_handle();
    let page = cx.new(|cx| McpServers::new(handle, window, cx));
    workspace.add_item_to_center(Box::new(page), window, cx);
}

/// Where a server runs.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {
    /// A program the agent starts: its command line, masked.
    Local(String),
    /// A server at a URL, its secret-looking query values hidden.
    Remote(String),
    /// Neither, in words: an extension's server, or an entry that could not be read.
    Other(String),
}

/// Whether a server runs for its agent.
#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    On,
    Off,
    /// A project's `.mcp.json` server the user approved in that folder.
    Approved,
    /// One the user turned down there.
    Denied,
    /// One Claude Code has not asked about yet.
    NotAsked,
    /// Zed's statuses, per project.
    Status(String),
}

impl State {
    fn words(&self) -> SharedString {
        match self {
            Self::On => SharedString::new_static("on"),
            Self::Off => SharedString::new_static("off"),
            Self::Approved => SharedString::new_static("approved"),
            Self::Denied => SharedString::new_static("denied"),
            Self::NotAsked => SharedString::new_static("not yet asked"),
            Self::Status(status) => status.clone().into(),
        }
    }

    const fn color(&self) -> Color {
        match self {
            Self::On | Self::Approved => Color::Success,
            Self::Denied => Color::Warning,
            Self::Off | Self::NotAsked | Self::Status(_) => Color::Muted,
        }
    }
}

/// One server as a file or a store names it: never an env or header value.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    name: String,
    scope: String,
    target: Target,
    env_names: Vec<String>,
    header_names: Vec<String>,
    state: State,
    /// Marley's own `marley` or `rusty`.
    marleys: bool,
    /// What an edit starts from, for a server the tab can change (#768).
    stored: Option<Stored>,
}

/// One CLI's rows, or why its file could not be read, and the file read.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Read {
    path: PathBuf,
    rows: Result<Vec<Row>, String>,
}

/// What the last reading of the CLIs' files found.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Files {
    claude: Read,
    codex: Read,
}

/// The tab: the CLIs' files as last read, the reading under way, the open projects' context
/// server stores it redraws with, and their folders, where an added Claude Code server can go.
pub struct McpServers {
    workspace: WeakEntity<Workspace>,
    folders: Vec<PathBuf>,
    focus_handle: FocusHandle,
    files: Option<Files>,
    reading: Task<()>,
    stores: Vec<Subscription>,
}

impl std::fmt::Debug for McpServers {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("McpServers")
            .field("files", &self.files)
            .field("stores", &self.stores.len())
            .finish_non_exhaustive()
    }
}

impl McpServers {
    fn new(workspace: WeakEntity<Workspace>, window: &Window, cx: &Context<Self>) -> Self {
        let mut page = Self {
            workspace,
            folders: Vec::new(),
            focus_handle: cx.focus_handle(),
            files: None,
            reading: Task::ready(()),
            stores: Vec::new(),
        };
        page.refresh(window, cx);
        page
    }

    /// Reads the CLIs' files again off the main thread, for the folders of the open projects, and
    /// follows those projects' context server stores. The projects are looked up once the caller's
    /// update has ended: the action that opens the tab runs while its workspace is being updated,
    /// and reading it then would panic.
    pub(crate) fn refresh(&mut self, window: &Window, cx: &Context<Self>) {
        let claude_json = claude_json();
        let codex_config = crate::agent_notify::codex_config();
        self.reading = cx.spawn_in(window, async move |page, cx| {
            let Some(folders) = page
                .update_in(cx, |page, window, cx| {
                    let projects = open_projects(window, cx);
                    page.stores = projects
                        .iter()
                        .map(|project| {
                            let store = project.read(cx).context_server_store();
                            cx.subscribe(&store, |_, _, _: &ServerStatusChangedEvent, cx| {
                                cx.notify();
                            })
                        })
                        .collect();
                    page.folders = local_folders(&projects, cx);
                    page.folders.clone()
                })
                .log_err()
            else {
                return;
            };
            let files = cx
                .background_spawn(futures::future::lazy(move |_| {
                    let (redactor, _) = Redactor::new(&[]);
                    Files {
                        claude: Read {
                            rows: claude_rows_in(&claude_json, &folders, &redactor),
                            path: claude_json,
                        },
                        codex: Read {
                            rows: codex_rows_in(&codex_config, &redactor),
                            path: codex_config,
                        },
                    }
                }))
                .await;
            page.update(cx, |page, cx| {
                page.files = Some(files);
                cx.notify();
            })
            .log_err();
        });
    }

    /// One CLI's section: its heading with Add Server for `places`, then its rows or why there are
    /// none.
    fn render_read(
        key: &'static str,
        heading_text: &'static str,
        read: Option<&Read>,
        none: &str,
        places: Vec<Place>,
        cx: &Context<Self>,
    ) -> AnyElement {
        let body = match read {
            None => muted("Reading…"),
            Some(Read {
                path,
                rows: Err(why),
            }) => v_flex()
                .px_2()
                .child(
                    Label::new(format!("{} could not be read: {why}", path.display()))
                        .size(LabelSize::Small)
                        .color(Color::Warning),
                )
                .into_any_element(),
            Some(Read {
                path,
                rows: Ok(rows),
            }) if rows.is_empty() => muted(format!("{none} ({}).", path.display())),
            Some(Read { rows: Ok(rows), .. }) => v_flex()
                .w_full()
                .gap_1()
                .children(rows.iter().map(|row| Self::render_row(row, cx)))
                .into_any_element(),
        };
        v_flex()
            .w_full()
            .gap_1()
            .child(
                h_flex().gap_2().child(heading(heading_text)).child(
                    Button::new(
                        SharedString::from(format!("marley-mcp-add-{key}")),
                        "Add Server",
                    )
                    .start_icon(Icon::new(IconName::Plus).size(IconSize::XSmall))
                    .style(ButtonStyle::Subtle)
                    .label_size(LabelSize::Small)
                    .on_click(cx.listener(move |page, _, window, cx| {
                        page.open_form(None, places.clone(), window, cx);
                    })),
                ),
            )
            .child(body)
            .into_any_element()
    }

    /// The form over the tab: a server added to one of `places`, or `editing` changed.
    fn open_form(
        &self,
        editing: Option<(String, Stored)>,
        places: Vec<Place>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let page = cx.entity().downgrade();
        let workspace = self.workspace.clone();
        self.workspace
            .update(cx, |workspace_view, cx| {
                workspace_view.toggle_modal(window, cx, move |window, cx| {
                    McpServerForm::new(page, workspace, editing, places, window, cx)
                });
            })
            .log_err();
    }

    /// Where an added Claude Code server can go: the user scope, then each open folder's local
    /// scope and `.mcp.json`.
    fn claude_places(&self) -> Vec<Place> {
        std::iter::once(Place::ClaudeUser)
            .chain(self.folders.iter().flat_map(|folder| {
                [
                    Place::ClaudeLocal(folder.clone()),
                    Place::ClaudeProject(folder.clone()),
                ]
            }))
            .collect()
    }

    fn render_row(row: &Row, cx: &Context<Self>) -> AnyElement {
        let buttons = row.stored.clone().map(|stored| {
            let id = |verb: &str| {
                SharedString::from(format!("marley-mcp-{verb}-{}-{}", row.scope, row.name))
            };
            let name = row.name.clone();
            let edit = (name.clone(), stored.clone());
            h_flex()
                .gap_1()
                .child(
                    Button::new(id("edit"), "Edit")
                        .style(ButtonStyle::Subtle)
                        .label_size(LabelSize::Small)
                        .on_click(cx.listener(move |page, _, window, cx| {
                            let places = vec![edit.1.place.clone()];
                            page.open_form(Some(edit.clone()), places, window, cx);
                        })),
                )
                .child(
                    Button::new(id("remove"), "Remove")
                        .style(ButtonStyle::Subtle)
                        .label_size(LabelSize::Small)
                        .on_click(cx.listener(move |page, _, window, cx| {
                            mcp_server_form::remove(
                                cx.entity().downgrade(),
                                page.workspace.clone(),
                                name.clone(),
                                stored.clone(),
                                window,
                                cx,
                            );
                        })),
                )
        });
        render_row(row, buttons)
    }
}

impl Render for McpServers {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let zed = zed_rows(&open_projects(window, cx), cx);
        let claude = self.files.as_ref().map(|files| &files.claude);
        let codex = self.files.as_ref().map(|files| &files.codex);
        v_flex()
            .id("marley-mcp-servers")
            .debug_selector(|| "marley-mcp-servers".into())
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_4()
            .bg(colors.editor_background)
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .child(Headline::new("MCP Servers").size(HeadlineSize::Small))
                    .child(div().flex_1())
                    .child(
                        Button::new("marley-mcp-servers-refresh", "Refresh")
                            .start_icon(Icon::new(IconName::RotateCw).size(IconSize::Small))
                            .style(ButtonStyle::Filled)
                            .on_click(cx.listener(|page, _, window, cx| page.refresh(window, cx))),
                    )
                    .child(
                        Button::new("marley-mcp-servers-configure", "Configure in Settings")
                            .start_icon(Icon::new(IconName::Settings).size(IconSize::Small))
                            .style(ButtonStyle::Filled)
                            .on_click(|_, window, cx| {
                                window.dispatch_action(
                                    Box::new(zed_actions::OpenSettingsAt {
                                        path: "context_servers".to_string(),
                                        target: None,
                                    }),
                                    cx,
                                );
                            }),
                    ),
            )
            // A card per agent, what Marley knows of it over its servers, as the Tools board
            // draws them (#846).
            .child(
                card(cx)
                    .child(render_agent(AgentKind::Claude, cx))
                    .child(Self::render_read(
                        "claude",
                        "SERVERS",
                        claude,
                        "No MCP servers for the open projects",
                        self.claude_places(),
                        cx,
                    ))
                    .child(muted(
                        "Claude Code's plugins add servers of their own, which claude mcp list \
                         shows.",
                    )),
            )
            .child(
                card(cx)
                    .child(render_agent(AgentKind::Codex, cx))
                    .child(Self::render_read(
                        "codex",
                        "SERVERS",
                        codex,
                        "No MCP servers",
                        vec![Place::Codex],
                        cx,
                    )),
            )
            .child(
                card(cx)
                    .child(heading("ZED"))
                    .when(zed.is_empty(), |section| {
                        section.child(muted("No context servers."))
                    })
                    .children(zed.iter().map(|row| render_row(row, None))),
            )
    }
}

impl Focusable for McpServers {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for McpServers {}

impl Item for McpServers {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("MCP Servers")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::ToolHammer))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static(
            "Every MCP server Claude Code, Codex and Zed's agents see",
        ))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }
}

fn heading(text: &'static str) -> impl IntoElement {
    Label::new(text).size(LabelSize::XSmall).color(Color::Muted)
}

/// One agent CLI: the program and version found, and each part of Marley that rests on it.
/// One agent's card (#846, D1).
fn card(cx: &App) -> Div {
    let colors = cx.theme().colors();
    v_flex()
        .w_full()
        .gap_2()
        .p_3()
        .rounded_lg()
        .border_1()
        .border_color(colors.border)
        .bg(colors.panel_background)
}

fn render_agent(kind: AgentKind, cx: &App) -> AnyElement {
    let program = crate::agent_versions::program(kind, cx)
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let found = match crate::agent_versions::found(kind, cx) {
        None => "not checked yet".to_string(),
        Some(Found::Version(version)) => format!("{version} at {program}"),
        Some(Found::Unreadable(why)) => format!("{program}: version unknown, --version {why}"),
        Some(Found::Missing(_)) => "not on Marley's PATH".to_string(),
    };
    let outdated = crate::agent_versions::outdated(kind, cx);
    let update = (!outdated.is_empty())
        .then(|| versions::update_command(kind))
        .flatten();
    v_flex()
        .w_full()
        .px_2()
        .gap_0p5()
        .child(
            h_flex()
                .gap_2()
                .child(Label::new(kind.display_name()))
                .child(Label::new(found).size(LabelSize::Small).color(Color::Muted)),
        )
        .children(integrations_of(kind).map(|integration| {
            let wanted = crate::agent_versions::wanted(integration, cx);
            let older = outdated
                .iter()
                .any(|outdated| outdated.id == integration.id);
            let (state, color) = if older {
                (
                    format!("needs {} or later", integration.tested.from),
                    Color::Warning,
                )
            } else if !wanted {
                ("not turned on".to_string(), Color::Muted)
            } else if crate::agent_versions::is_on(integration, cx) {
                ("on".to_string(), Color::Success)
            } else {
                ("off on this version".to_string(), Color::Muted)
            };
            h_flex()
                .pl_4()
                .gap_2()
                .when(older, |line| {
                    line.child(
                        Icon::new(IconName::Warning)
                            .size(IconSize::XSmall)
                            .color(Color::Warning),
                    )
                })
                .child(
                    Label::new(crate::agent_versions::capitalized(integration.name))
                        .size(LabelSize::Small),
                )
                .child(Label::new(state).size(LabelSize::Small).color(color))
                .child(
                    Label::new(format!("tested on {}", integration.tested.words()))
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
        }))
        .when_some(update, |agent, command| {
            agent.child(
                div().pl_4().child(
                    Label::new(format!("Run {command} to update it."))
                        .size(LabelSize::Small)
                        .color(Color::Warning),
                ),
            )
        })
        .into_any_element()
}

fn render_row(row: &Row, buttons: Option<Div>) -> AnyElement {
    let target = match &row.target {
        Target::Local(line) => line.clone(),
        Target::Remote(url) => url.clone(),
        Target::Other(words) => words.clone(),
    };
    let names = |label: &str, names: &[String]| {
        (!names.is_empty()).then(|| format!("{label}: {}", names.join(", ")))
    };
    let details: Vec<String> = [
        names("env", &row.env_names),
        names("headers", &row.header_names),
    ]
    .into_iter()
    .flatten()
    .collect();
    v_flex()
        .w_full()
        .px_2()
        .py_1()
        .gap_0p5()
        .child(
            h_flex()
                .gap_2()
                .child(Label::new(row.name.clone()))
                .child(
                    Label::new(row.scope.clone())
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
                .child(
                    Label::new(row.state.words())
                        .size(LabelSize::Small)
                        .color(row.state.color()),
                )
                .when(row.marleys, |line| {
                    line.child(
                        Label::new("Marley's")
                            .size(LabelSize::Small)
                            .color(Color::Accent),
                    )
                })
                .when_some(buttons, ParentElement::child),
        )
        .child(
            Label::new(target)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .when(!details.is_empty(), |row| {
            row.child(
                Label::new(details.join(" · "))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
        })
        .into_any_element()
}

/// The folders of the local projects among `projects`.
fn local_folders(projects: &[Entity<Project>], cx: &App) -> Vec<PathBuf> {
    projects
        .iter()
        .filter(|project| project.read(cx).is_local())
        .flat_map(|project| {
            project
                .read(cx)
                .visible_worktrees(cx)
                .map(|worktree| worktree.read(cx).abs_path().to_path_buf())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The projects of Marley's windows, each once.
fn open_projects(window: &Window, cx: &App) -> Vec<Entity<Project>> {
    let mut projects: Vec<Entity<Project>> = Vec::new();
    for workspace in crate::mcp::window_workspaces(window, cx) {
        let project = workspace.read(cx).project().clone();
        if !projects
            .iter()
            .any(|known| known.entity_id() == project.entity_id())
        {
            projects.push(project);
        }
    }
    projects
}

/// Zed's context servers in `projects`' stores, a row per name with each project's status.
fn zed_rows(projects: &[Entity<Project>], cx: &App) -> Vec<Row> {
    let mut rows: BTreeMap<String, (Row, Vec<String>)> = BTreeMap::new();
    for project in projects {
        let label = project_label(project, cx);
        let store = project.read(cx).context_server_store();
        let store = store.read(cx);
        for id in store.server_ids() {
            let status = if store.is_server_enabled(id, cx) {
                status_words(store.status_for_server(id))
            } else {
                "off".to_string()
            };
            let entry = rows.entry(id.0.to_string()).or_insert_with(|| {
                let (target, env_names, header_names) = match store.settings_for_server(id) {
                    Some(ContextServerSettings::Stdio { command, .. }) => (
                        Target::Local(masked_command(
                            &command.path.to_string_lossy(),
                            &command.args,
                            &Redactor::new(&[]).0,
                        )),
                        command
                            .env
                            .as_ref()
                            .map(|env| sorted(env.keys().cloned()))
                            .unwrap_or_default(),
                        Vec::new(),
                    ),
                    Some(ContextServerSettings::Http { url, headers, .. }) => (
                        Target::Remote(marley_browser::observe::redact_url(url)),
                        Vec::new(),
                        sorted(headers.keys().cloned()),
                    ),
                    Some(ContextServerSettings::Extension { .. }) | None => (
                        Target::Other("an extension's server".to_string()),
                        Vec::new(),
                        Vec::new(),
                    ),
                };
                let name = id.0.to_string();
                let marleys =
                    name == crate::mcp::CONTEXT_SERVER || name == crate::rusty::CONTEXT_SERVER;
                (
                    Row {
                        name,
                        scope: "Zed".to_string(),
                        target,
                        env_names,
                        header_names,
                        state: State::Off,
                        marleys,
                        stored: None,
                    },
                    Vec::new(),
                )
            });
            entry.1.push(if projects.len() > 1 {
                format!("{status} in {label}")
            } else {
                status
            });
        }
    }
    rows.into_values()
        .map(|(row, statuses)| Row {
            state: State::Status(statuses.join(" · ")),
            ..row
        })
        .collect()
}

/// A project's name in a status: its folders' names, or "no folder".
fn project_label(project: &Entity<Project>, cx: &App) -> String {
    let names: Vec<String> = project
        .read(cx)
        .visible_worktrees(cx)
        .map(|worktree| worktree.read(cx).root_name_str().to_string())
        .collect();
    if names.is_empty() {
        "no folder".to_string()
    } else {
        names.join(", ")
    }
}

fn status_words(status: Option<ContextServerStatus>) -> String {
    match status {
        None | Some(ContextServerStatus::Stopped) => "stopped".to_string(),
        Some(ContextServerStatus::Starting) => "starting".to_string(),
        Some(ContextServerStatus::Running) => "running".to_string(),
        Some(ContextServerStatus::Error(error)) => format!("error: {error}"),
        Some(ContextServerStatus::AuthRequired) => "needs sign-in".to_string(),
        Some(ContextServerStatus::ClientSecretRequired { .. }) => {
            "needs a client secret".to_string()
        }
        Some(ContextServerStatus::Authenticating) => "signing in".to_string(),
    }
}

/// Claude Code's global config: `.claude.json` in `CLAUDE_CONFIG_DIR` when that is set and not
/// empty, else in the home folder (not in `~/.claude`), as Claude Code 2.1.296 places it.
pub(crate) fn claude_json() -> PathBuf {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .filter(|folder| !folder.is_empty())
        .map_or_else(|| util::paths::home_dir().clone(), PathBuf::from)
        .join(".claude.json")
}

/// `.claude.json`'s parts the tab reads; every other key is skipped unread.
#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct ClaudeJson<'a> {
    #[serde(borrow)]
    mcp_servers: BTreeMap<String, &'a RawValue>,
    #[serde(borrow)]
    projects: BTreeMap<String, &'a RawValue>,
}

/// One folder's entry in `.claude.json`'s `projects`.
#[derive(Deserialize, Default)]
#[serde(default)]
struct ClaudeProject<'a> {
    #[serde(borrow, rename = "mcpServers")]
    servers: BTreeMap<String, &'a RawValue>,
    /// The `.mcp.json` servers the user approved in this folder.
    #[serde(rename = "enabledMcpjsonServers")]
    approved: Vec<String>,
    /// The ones turned down.
    #[serde(rename = "disabledMcpjsonServers")]
    denied: Vec<String>,
}

/// A project's `.mcp.json`.
#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct McpJson<'a> {
    #[serde(borrow)]
    mcp_servers: BTreeMap<String, &'a RawValue>,
}

/// A Claude Code server entry: its env and header maps are read for their names only.
#[derive(Deserialize, Default)]
#[serde(default)]
struct ClaudeServer {
    #[serde(rename = "type")]
    transport: Option<String>,
    command: Option<String>,
    args: Vec<String>,
    url: Option<String>,
    env: Names,
    headers: Names,
}

/// A map's keys, its values skipped unread: an env or header map, whose values never reach a row.
#[derive(Default)]
struct Names(Vec<String>);

impl<'de> Deserialize<'de> for Names {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Keys;

        impl<'de> serde::de::Visitor<'de> for Keys {
            type Value = Names;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Names, A::Error> {
                let mut names = Vec::new();
                while let Some((name, IgnoredAny)) = map.next_entry::<String, IgnoredAny>()? {
                    names.push(name);
                }
                Ok(Names(names))
            }
        }

        deserializer.deserialize_map(Keys)
    }
}

/// Claude Code's servers: the user scope of `claude_json`, then for each of `folders` its local
/// scope there and its `.mcp.json`, each marked approved, denied or not yet asked in that folder.
/// A missing file holds no servers.
fn claude_rows_in(
    claude_json: &Path,
    folders: &[PathBuf],
    redactor: &Redactor,
) -> Result<Vec<Row>, String> {
    let text = read_if_there(claude_json)?.unwrap_or_default();
    let config: ClaudeJson<'_> = if text.trim().is_empty() {
        ClaudeJson::default()
    } else {
        serde_json::from_str(&text).map_err(|error| error.to_string())?
    };
    let mut rows: Vec<Row> = config
        .mcp_servers
        .iter()
        .map(|(name, entry)| claude_row(name, &Place::ClaudeUser, entry, &State::On, redactor))
        .collect();
    for folder in folders {
        let key = folder.to_string_lossy();
        let project: ClaudeProject<'_> = config
            .projects
            .get(key.as_ref())
            .map(|entry| serde_json::from_str(entry.get()))
            .transpose()
            .map_err(|error| format!("{key}'s entry: {error}"))?
            .unwrap_or_default();
        rows.extend(project.servers.iter().map(|(server, entry)| {
            claude_row(
                server,
                &Place::ClaudeLocal(folder.clone()),
                entry,
                &State::On,
                redactor,
            )
        }));
        let mcp_json = folder.join(".mcp.json");
        let Some(text) = read_if_there(&mcp_json)? else {
            continue;
        };
        let shared: McpJson<'_> = serde_json::from_str(&text)
            .map_err(|error| format!("{}: {error}", mcp_json.display()))?;
        rows.extend(shared.mcp_servers.iter().map(|(server, entry)| {
            let state = if project.denied.contains(server) {
                State::Denied
            } else if project.approved.contains(server) {
                State::Approved
            } else {
                State::NotAsked
            };
            claude_row(
                server,
                &Place::ClaudeProject(folder.clone()),
                entry,
                &state,
                redactor,
            )
        }));
    }
    Ok(rows)
}

/// A Claude Code entry as a row, with what an edit starts from. The entry's hash is taken over
/// its JSON as parsed, which the form compares again before it writes.
fn claude_row(
    name: &str,
    place: &Place,
    entry: &RawValue,
    state: &State,
    redactor: &Redactor,
) -> Row {
    let row = |target, env_names, header_names, stored| Row {
        name: name.to_string(),
        scope: place.words(),
        target,
        env_names,
        header_names,
        state: state.clone(),
        marleys: false,
        stored,
    };
    let server: ClaudeServer = match serde_json::from_str(entry.get()) {
        Ok(server) => server,
        Err(error) => {
            return row(
                Target::Other(format!("an entry Marley cannot read: {error}")),
                Vec::new(),
                Vec::new(),
                None,
            );
        }
    };
    let seen = serde_json::from_str::<serde_json::Value>(entry.get())
        .map(|value| entry_hash(&value.to_string()))
        .ok();
    let env_names = sorted(server.env.0);
    let header_names = sorted(server.headers.0);
    let (target, kind) = match (server.transport.as_deref(), server.command, server.url) {
        (Some("http" | "sse"), _, Some(url)) | (None, None, Some(url)) => (
            Target::Remote(marley_browser::observe::redact_url(&url)),
            Some(Kind::Url { url }),
        ),
        (_, Some(command), _) => (
            Target::Local(masked_command(&command, &server.args, redactor)),
            Some(Kind::Command {
                command,
                args: server.args,
            }),
        ),
        _ => (
            Target::Other("neither a command nor a URL".to_string()),
            None,
        ),
    };
    let stored = kind.zip(seen).map(|(kind, seen)| Stored {
        pairs: match kind {
            Kind::Command { .. } => env_names.clone(),
            Kind::Url { .. } => header_names.clone(),
        },
        place: place.clone(),
        kind,
        seen,
    });
    row(target, env_names, header_names, stored)
}

/// Codex's servers: `[mcp_servers.<name>]` in `config`, off where `enabled = false`. A missing
/// file holds no servers.
fn codex_rows_in(config: &Path, redactor: &Redactor) -> Result<Vec<Row>, String> {
    let Some(text) = read_if_there(config)? else {
        return Ok(Vec::new());
    };
    let document: toml_edit::DocumentMut = text.parse().map_err(|error| format!("{error}"))?;
    let Some(servers) = document
        .get("mcp_servers")
        .and_then(toml_edit::Item::as_table_like)
    else {
        return Ok(Vec::new());
    };
    Ok(servers
        .iter()
        .map(|(name, server)| codex_row(name, server, redactor))
        .collect())
}

/// One `[mcp_servers.<name>]` table as a row, with what an edit starts from.
fn codex_row(name: &str, server: &toml_edit::Item, redactor: &Redactor) -> Row {
    let row = |target, env_names, header_names, state, stored| Row {
        name: name.to_string(),
        scope: Place::Codex.words(),
        target,
        env_names,
        header_names,
        state,
        marleys: false,
        stored,
    };
    let seen = entry_hash(&server.to_string());
    let Some(server) = server.as_table_like() else {
        return row(
            Target::Other("not a table".to_string()),
            Vec::new(),
            Vec::new(),
            State::Off,
            None,
        );
    };
    let text = |key: &str| {
        server
            .get(key)
            .and_then(toml_edit::Item::as_str)
            .map(str::to_string)
    };
    let keys = |key: &str| {
        server
            .get(key)
            .and_then(toml_edit::Item::as_table_like)
            .map(|table| {
                table
                    .iter()
                    .map(|(key, _)| key.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default()
    };
    let strings = |key: &str| {
        server
            .get(key)
            .and_then(toml_edit::Item::as_array)
            .map(|array| {
                array
                    .iter()
                    .filter_map(|value| value.as_str().map(str::to_string))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default()
    };
    // The form edits `env` and `http_headers`; the names the other keys list are shown.
    let env_pairs = keys("env");
    let header_pairs = keys("http_headers");
    let mut env_names = env_pairs.clone();
    env_names.extend(strings("env_vars"));
    let mut header_names = header_pairs.clone();
    header_names.extend(keys("env_http_headers"));
    if server.contains_key("bearer_token_env_var") {
        header_names.push("Authorization".to_string());
    }
    let (target, stored) = match (text("command"), text("url")) {
        (Some(command), _) => {
            let args = strings("args");
            (
                Target::Local(masked_command(&command, &args, redactor)),
                Some((Kind::Command { command, args }, sorted(env_pairs))),
            )
        }
        (None, Some(url)) => (
            Target::Remote(marley_browser::observe::redact_url(&url)),
            Some((Kind::Url { url }, sorted(header_pairs))),
        ),
        (None, None) => (
            Target::Other("neither a command nor a URL".to_string()),
            None,
        ),
    };
    let stored = stored.map(|(kind, pairs)| Stored {
        place: Place::Codex,
        kind,
        pairs,
        seen,
    });
    let enabled = server
        .get("enabled")
        .and_then(toml_edit::Item::as_bool)
        .unwrap_or(true);
    row(
        target,
        sorted(env_names),
        sorted(header_names),
        if enabled { State::On } else { State::Off },
        stored,
    )
}

/// `path`'s text, `None` when there is no file.
fn read_if_there(path: &Path) -> Result<Option<String>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("{}: {error}", path.display())),
    }
}

/// A command and its arguments as one line, keys and tokens masked.
fn masked_command(command: &str, args: &[String], redactor: &Redactor) -> String {
    let line = std::iter::once(command)
        .chain(args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");
    redactor.redact(&line).text
}

fn sorted(names: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut names: Vec<String> = names.into_iter().collect();
    names.sort();
    names.dedup();
    names
}
