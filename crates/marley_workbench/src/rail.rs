//! The rail: the Marley layout's sidebar. Each project group, and under it the terminals in that
//! group's center panes, with the agent CLIs running in them, and the group's agent threads. It
//! implements Zed's `workspace::Sidebar`, so the `MultiWorkspace` keeps the resize handle, open
//! state, persistence and the toggle actions; the rows and the one selected row come from
//! `marley_rail`.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use acp_thread::{AcpThread, AcpThreadEvent};
use agent_ui::thread_metadata_store::{ThreadId, ThreadMetadata, ThreadMetadataStore};
use agent_ui::{Agent, AgentPanel, AgentPanelEvent, AgentThreadSource, NewExternalAgentThread};
use anyhow::Context as _;
use gpui::{
    Anchor, AnyElement, App, Context, Entity, EntityId, EventEmitter, FocusHandle, Focusable,
    Pixels, Render, Subscription, Task, WeakEntity, Window, px,
};
use marley_agent::{AgentKind, WAITING_AFTER};
use marley_rail::{
    Focus, ProjectRow, ProjectSnapshot, RailSnapshot, Row, TerminalAgent, TerminalRow,
    TerminalSnapshot, ThreadRow, ThreadSnapshot, ThreadStatus,
};
use project::{
    AgentId, AgentRegistryStore, AgentServerStore, AgentServersUpdated, Project, ProjectGroupKey,
};
use recent_projects::sidebar_recent_projects::SidebarRecentProjects;
use terminal::Terminal;
use terminal_view::{TerminalView, terminal_panel::TerminalPanel};
use ui::{
    AgentThreadStatus, ContextMenu, ContextMenuEntry, Disclosure, Icon, IconButton, IconName,
    IconSize, Indicator, Label, LabelSize, ListItem, PopoverMenu, PopoverMenuHandle, ThreadItem,
    Tooltip, prelude::*, utils::platform_title_bar_height,
};
use util::ResultExt as _;
use util::path_list::PathList;
use workspace::{
    MultiWorkspace, MultiWorkspaceEvent, ProjectGroup, Sidebar, SidebarEvent, SidebarSide,
    Workspace,
    item::{Item as _, ItemEvent},
    notifications::DetachAndPromptErr as _,
};

const DEFAULT_WIDTH: Pixels = px(260.);
const MIN_WIDTH: Pixels = px(180.);
const MAX_WIDTH: Pixels = px(600.);

/// Makes the `Terminal` behind a new center terminal, started in the given directory. Production
/// uses `Project::create_terminal_shell` itself, so no line of this crate spawns a shell; tests
/// hand in a display-only terminal.
type TerminalFactory = fn(
    &mut Project,
    Option<PathBuf>,
    &mut Context<Project>,
) -> Task<anyhow::Result<Entity<Terminal>>>;

/// Reads the command a terminal's foreground process runs. Production asks the PTY; a test's
/// display-only terminal has no process, so tests hand in their own.
type ForegroundCommand = fn(&Entity<Terminal>, &App) -> Option<String>;

/// How long an agent's launch waits for the shell to say it is ready, as Zed's terminal threads
/// wait, before writing the command anyway.
const AGENT_STARTUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Zed's own sidebar and whether it was open, kept by the rail that replaced it.
pub type KeptSidebar = (Entity<sidebar::Sidebar>, bool);

/// The Marley layout's sidebar: each project group, with the terminals in its center panes and
/// its agent threads under it.
pub struct Rail {
    multi_workspace: WeakEntity<MultiWorkspace>,
    focus_handle: FocusHandle,
    width: Pixels,
    terminal_factory: TerminalFactory,
    foreground_command: ForegroundCommand,
    /// Where the `+` menu looks for agent CLIs: the process's `PATH`, which tests replace.
    agent_search_path: Option<OsString>,
    /// When each terminal last wrote output, on the executor's clock.
    terminal_output: HashMap<EntityId, Instant>,
    /// Per agent terminal: a refresh due once its output has been quiet for `WAITING_AFTER`,
    /// replaced on each output.
    quiet_timers: HashMap<EntityId, Task<()>>,
    /// The window as the rail last read it. `render` draws from this alone, so another entity's
    /// notify does not redraw the window, and an event that changes nothing shown does not either:
    /// workspaces and terminal views report every chunk of terminal output.
    snapshot: Snapshot,
    /// Zed's sidebar, kept while the rail stands in for it, so the switch back loses neither its
    /// state nor the work it has running.
    zed_sidebar: Option<KeptSidebar>,
    /// Zed's sidebar state restored into a window that opened in the Marley layout, kept unread
    /// and written back unchanged.
    zed_sidebar_state: Option<String>,
    add_project_menu: PopoverMenuHandle<SidebarRecentProjects>,
    workspace_subscriptions: HashMap<EntityId, Subscription>,
    terminal_subscriptions: HashMap<EntityId, [Subscription; 2]>,
    /// Per Agent Panel: its events, and focus entering and leaving it.
    panel_subscriptions: HashMap<EntityId, [Subscription; 3]>,
    /// Per live thread: the events that can change its row.
    thread_subscriptions: HashMap<EntityId, Subscription>,
    /// Per project: its agent servers, which name the New Agent Thread entries.
    agent_server_subscriptions: HashMap<EntityId, Subscription>,
    /// The thread metadata store, once it exists: it notifies and emits nothing else.
    thread_store_subscription: Option<Subscription>,
    /// Each listed thread's status at the last rebuild, by key, so a run's end is seen.
    thread_statuses: HashMap<String, ThreadStatus>,
    /// The threads whose attention dot is lit.
    noted_threads: HashSet<String>,
    _multi_workspace_subscriptions: [Subscription; 2],
}

impl std::fmt::Debug for Rail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rail")
            .field("width", &self.width)
            .field("rows", &self.snapshot.rail)
            .finish_non_exhaustive()
    }
}

/// The entities behind a project row, held weakly: a closed tab must not outlive its terminal.
struct GroupEntry {
    key: ProjectGroupKey,
    workspace: WeakEntity<Workspace>,
}

/// The entities behind a terminal row, held weakly.
struct TerminalEntry {
    workspace: WeakEntity<Workspace>,
    view: WeakEntity<TerminalView>,
}

/// What a thread row opens, and the icon it draws.
struct ThreadEntry {
    workspace: WeakEntity<Workspace>,
    thread_id: ThreadId,
    agent: Agent,
    work_dirs: PathList,
    title: Option<SharedString>,
    icon: AgentIcon,
}

/// An agent's icon: one of Zed's, or an agent server's own SVG.
#[derive(Clone)]
enum AgentIcon {
    Named(IconName),
    Svg(SharedString),
}

/// The window, read once: the pure snapshot, plus the entities the handlers act on.
#[derive(Default)]
struct Snapshot {
    rail: RailSnapshot,
    groups: Vec<GroupEntry>,
    terminals: HashMap<u64, TerminalEntry>,
    /// The terminal views an agent CLI is running in.
    agent_terminals: HashSet<EntityId>,
    threads: HashMap<String, ThreadEntry>,
    /// The thread the displayed workspace's visible Agent Panel shows, which is seen by now.
    shown_thread: Option<String>,
}

impl Rail {
    /// A rail for `multi_workspace`, keeping Zed's sidebar when it replaces one.
    pub fn new(
        multi_workspace: &Entity<MultiWorkspace>,
        zed_sidebar: Option<KeptSidebar>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscriptions = [
            cx.subscribe_in(
                multi_workspace,
                window,
                |rail, _, _: &MultiWorkspaceEvent, window, cx| rail.refresh(window, cx),
            ),
            // Re-keying a project group notifies without an event.
            cx.observe_in(multi_workspace, window, |rail, _, window, cx| {
                rail.refresh(window, cx);
            }),
        ];
        // The `MultiWorkspace` may be mid-update while its sidebar is built, so the first read
        // waits for the end of this effect cycle.
        cx.defer_in(window, Self::refresh);
        Self {
            multi_workspace: multi_workspace.downgrade(),
            focus_handle: cx.focus_handle(),
            width: DEFAULT_WIDTH,
            terminal_factory: Project::create_terminal_shell,
            foreground_command: |terminal, cx| terminal.read(cx).foreground_process_command_name(),
            agent_search_path: std::env::var_os("PATH"),
            terminal_output: HashMap::default(),
            quiet_timers: HashMap::default(),
            snapshot: Snapshot::default(),
            zed_sidebar,
            zed_sidebar_state: None,
            add_project_menu: PopoverMenuHandle::default(),
            workspace_subscriptions: HashMap::default(),
            terminal_subscriptions: HashMap::default(),
            panel_subscriptions: HashMap::default(),
            thread_subscriptions: HashMap::default(),
            agent_server_subscriptions: HashMap::default(),
            thread_store_subscription: None,
            thread_statuses: HashMap::default(),
            noted_threads: HashSet::default(),
            _multi_workspace_subscriptions: subscriptions,
        }
    }

    /// Hands Zed's sidebar back for the switch to the Zed layout: the one the rail kept, and the
    /// state to restore into a fresh one when it kept none.
    pub(crate) const fn take_zed_sidebar(&mut self) -> (Option<KeptSidebar>, Option<String>) {
        (self.zed_sidebar.take(), self.zed_sidebar_state.take())
    }

    /// Follows every workspace, terminal, Agent Panel and live thread, rereads the window, and
    /// redraws only when what the rail shows has changed.
    fn refresh(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.sync_subscriptions(window, cx);
        let mut snapshot = self
            .multi_workspace
            .upgrade()
            .map(|multi_workspace| {
                build_snapshot(
                    &multi_workspace,
                    self.foreground_command,
                    &self.terminal_output,
                    window,
                    cx,
                )
            })
            .unwrap_or_default();
        self.note_ended_runs(&mut snapshot);
        if snapshot.rail != self.snapshot.rail {
            cx.notify();
        }
        self.snapshot = snapshot;
    }

    /// Lights the dot of each thread whose run ended since the last rebuild while it was not
    /// shown, and keeps it lit until the thread is shown.
    fn note_ended_runs(&mut self, snapshot: &mut Snapshot) {
        let shown = snapshot.shown_thread.as_deref();
        let mut statuses = HashMap::default();
        let mut noted = HashSet::default();
        for thread in snapshot
            .rail
            .projects
            .iter_mut()
            .flat_map(|project| project.threads.iter_mut())
        {
            thread.attention = marley_rail::thread_attention(
                self.thread_statuses.get(&thread.key).copied(),
                thread.status,
                shown == Some(thread.key.as_str()),
                self.noted_threads.contains(&thread.key),
            );
            if thread.attention {
                noted.insert(thread.key.clone());
            }
            statuses.insert(thread.key.clone(), thread.status);
        }
        self.thread_statuses = statuses;
        self.noted_threads = noted;
    }

    /// Notes a terminal's output, and for an agent terminal re-arms the refresh that reads it as
    /// waiting once the output stops.
    fn note_output(&mut self, view: EntityId, window: &mut Window, cx: &mut Context<Self>) {
        self.terminal_output
            .insert(view, cx.background_executor().now());
        self.refresh(window, cx);
        if self.snapshot.agent_terminals.contains(&view) {
            let quiet = cx.background_executor().timer(WAITING_AFTER);
            let timer = cx.spawn_in(window, async move |rail, cx| {
                quiet.await;
                rail.update_in(cx, Self::refresh).log_err();
            });
            self.quiet_timers.insert(view, timer);
        }
    }

    fn sync_subscriptions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Watched {
            workspaces,
            views,
            panels,
            threads,
            agent_servers,
        } = self
            .multi_workspace
            .upgrade()
            .map(|multi_workspace| Watched::in_window(&multi_workspace, cx))
            .unwrap_or_default();
        let open: HashSet<EntityId> = views.iter().map(Entity::entity_id).collect();
        self.terminal_output.retain(|view, _| open.contains(view));
        self.quiet_timers.retain(|view, _| open.contains(view));
        // Rebuilt from what exists now: a restore replaces panes without reporting removals, and
        // whatever is left in the old maps is dropped, ending those subscriptions.
        self.workspace_subscriptions = resubscribe(
            &mut self.workspace_subscriptions,
            &workspaces,
            |workspace| {
                cx.subscribe_in(
                    workspace,
                    window,
                    |rail, _, _: &workspace::Event, window, cx| rail.refresh(window, cx),
                )
            },
        );
        self.terminal_subscriptions =
            resubscribe(&mut self.terminal_subscriptions, &views, |view| {
                [
                    cx.subscribe_in(
                        view,
                        window,
                        |rail, view, event: &terminal::Event, window, cx| {
                            if *event == terminal::Event::Wakeup {
                                rail.note_output(view.entity_id(), window, cx);
                            } else {
                                rail.refresh(window, cx);
                            }
                        },
                    ),
                    cx.subscribe_in(view, window, |rail, _, _: &ItemEvent, window, cx| {
                        rail.refresh(window, cx);
                    }),
                ]
            });
        self.panel_subscriptions = resubscribe(&mut self.panel_subscriptions, &panels, |panel| {
            // The panel's own focus handle wraps whatever view it shows, so focus inside a
            // thread counts as focus in the panel.
            let focus_handle = panel.focus_handle(cx);
            [
                cx.subscribe_in(panel, window, |rail, _, _: &AgentPanelEvent, window, cx| {
                    rail.refresh(window, cx);
                }),
                cx.on_focus_in(&focus_handle, window, Self::refresh),
                cx.on_focus_out(&focus_handle, window, |rail, _, window, cx| {
                    rail.refresh(window, cx);
                }),
            ]
        });
        self.thread_subscriptions =
            resubscribe(&mut self.thread_subscriptions, &threads, |thread| {
                cx.subscribe_in(
                    thread,
                    window,
                    |rail, _, event: &AcpThreadEvent, window, cx| {
                        if changes_the_row(event) {
                            rail.refresh(window, cx);
                        }
                    },
                )
            });
        self.agent_server_subscriptions = resubscribe(
            &mut self.agent_server_subscriptions,
            &agent_servers,
            |store| {
                cx.subscribe_in(
                    store,
                    window,
                    |rail, _, _: &AgentServersUpdated, window, cx| {
                        rail.refresh(window, cx);
                    },
                )
            },
        );
        if self.thread_store_subscription.is_none()
            && let Some(store) = ThreadMetadataStore::try_global(cx)
        {
            self.thread_store_subscription =
                Some(cx.observe_in(&store, window, |rail, _, window, cx| {
                    rail.refresh(window, cx);
                }));
        }
    }

    /// Shows `workspace` in the window. The rows hold their entities weakly, so the project may
    /// have closed since the rail last drew.
    fn activate_workspace(
        &self,
        workspace: &WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<Entity<Workspace>> {
        let workspace = workspace.upgrade().context("the project was closed")?;
        self.multi_workspace
            .update(cx, |multi_workspace, cx| {
                multi_workspace.activate(workspace.clone(), None, window, cx);
            })
            .map(|()| workspace)
    }

    fn activate_terminal(
        &self,
        workspace: &WeakEntity<Workspace>,
        view: &WeakEntity<TerminalView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let view = view.upgrade().context("the terminal was closed")?;
        let workspace = self.activate_workspace(workspace, window, cx)?;
        workspace.update(cx, |workspace, cx| {
            workspace.activate_item(&view, true, true, window, cx)
        });
        view.update(cx, TerminalView::clear_bell);
        Ok(())
    }

    /// Opens a terminal in `workspace`'s center, where Zed's own New Terminal would start one: the
    /// workspace's own project directory (a linked worktree's, not its main repository's).
    fn new_terminal(
        &self,
        workspace: &WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let workspace = self.activate_workspace(workspace, window, cx)?;
        let factory = self.terminal_factory;
        workspace.update(cx, |workspace, cx| {
            let directory = terminal_view::default_working_directory(workspace, cx);
            TerminalPanel::add_center_terminal(workspace, window, cx, move |project, cx| {
                factory(project, directory, cx)
            })
            .detach_and_prompt_err(
                "Could not open a terminal",
                window,
                cx,
                |_, _, _| None,
            );
        });
        Ok(())
    }

    /// Starts `kind` in a new center terminal of `workspace`, where New Terminal would start
    /// one. The command goes in once the shell says it is ready, as Zed's terminal threads start
    /// theirs, and it is only the agent's program name.
    fn new_agent(
        &self,
        workspace: &WeakEntity<Workspace>,
        kind: AgentKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let workspace = self.activate_workspace(workspace, window, cx)?;
        let factory = self.terminal_factory;
        let terminal = workspace.update(cx, |workspace, cx| {
            let directory = terminal_view::default_working_directory(workspace, cx);
            TerminalPanel::add_center_terminal(workspace, window, cx, move |project, cx| {
                factory(project, directory, cx)
            })
        });
        workspace.update(cx, |_, cx| {
            cx.spawn_in(window, async move |_, cx| {
                let terminal = terminal.await?;
                let handshake = |terminal: &mut Terminal, _: &mut Context<Terminal>| {
                    terminal.start_init_command_startup_handshake()
                };
                let startup = terminal.update(cx, handshake)?;
                let timeout = cx.background_executor().timer(AGENT_STARTUP_TIMEOUT);
                // A terminal without a PTY is ready at once; the timeout covers a shell that
                // never echoes the handshake's marker.
                futures::future::select(startup, timeout).await;
                let input = marley_agent::launch_input(kind);
                let launch = |terminal: &mut Terminal, cx: &mut Context<Terminal>| {
                    terminal.write_init_command_after_startup(input, cx)
                };
                let written = terminal.update(cx, launch)?;
                anyhow::ensure!(
                    written,
                    "the terminal took other input before the agent started"
                );
                anyhow::Ok(())
            })
            .detach_and_prompt_err(
                "Could not start the agent",
                window,
                cx,
                |_, _, _| None,
            );
        });
        Ok(())
    }

    /// Shows a thread: displays its workspace and opens the thread, focused, in the workspace's
    /// Agent Panel, which sits on the right in the Marley layout.
    fn open_thread(
        &self,
        key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let thread = self
            .snapshot
            .threads
            .get(key)
            .context("the thread is no longer listed")?;
        let (thread_id, agent, work_dirs, title) = (
            thread.thread_id,
            thread.agent.clone(),
            thread.work_dirs.clone(),
            thread.title.clone(),
        );
        let workspace = self.activate_workspace(&thread.workspace, window, cx)?;
        workspace.update(cx, |workspace, cx| {
            let panel = workspace
                .panel::<AgentPanel>(cx)
                .context("the project has no Agent Panel")?;
            panel.update(cx, |panel, cx| {
                panel.load_agent_thread(
                    agent,
                    thread_id,
                    Some(work_dirs),
                    title,
                    true,
                    AgentThreadSource::Sidebar,
                    window,
                    cx,
                );
            });
            workspace.focus_panel::<AgentPanel>(window, cx);
            anyhow::Ok(())
        })
    }

    /// Starts a thread for `agent` in `workspace`'s Agent Panel. The panel is called directly:
    /// a dispatched action would reach whichever workspace the window shows.
    fn new_agent_thread(
        &self,
        workspace: &WeakEntity<Workspace>,
        agent: &AgentId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        // The action's one field is private, so it is built the way a keymap builds it.
        let action: NewExternalAgentThread =
            serde_json::from_value(serde_json::json!({ "agent": agent.0 }))?;
        let workspace = self.activate_workspace(workspace, window, cx)?;
        workspace.update(cx, |workspace, cx| {
            let panel = workspace
                .panel::<AgentPanel>(cx)
                .context("the project has no Agent Panel")?;
            panel.update(cx, |panel, cx| {
                panel.new_external_agent_thread(&action, window, cx);
            });
            workspace.focus_panel::<AgentPanel>(window, cx);
            anyhow::Ok(())
        })
    }

    fn toggle_expanded(
        &mut self,
        key: &ProjectGroupKey,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.multi_workspace
            .update(cx, |multi_workspace, cx| {
                if let Some(group) = multi_workspace.group_state_by_key_mut(key) {
                    group.expanded = !group.expanded;
                }
                multi_workspace.serialize(cx);
            })
            .log_err();
        // Collapsing emits no event the rail hears.
        self.refresh(window, cx);
    }

    fn render_header(&self, window: &Window, cx: &Context<Self>) -> impl IntoElement {
        let header = h_flex()
            .h(platform_title_bar_height(window))
            .w_full()
            .flex_none()
            .gap_1()
            .px_1()
            .border_b_1()
            .border_color(cx.theme().colors().border);
        // While a sidebar is open on the left, the title bar leaves its window controls to it.
        #[cfg(target_os = "macos")]
        let header = if window.is_fullscreen() {
            header
        } else {
            header.pl(px(ui::utils::TRAFFIC_LIGHT_PADDING))
        };
        #[cfg(not(target_os = "macos"))]
        let header = if window.is_fullscreen() {
            header
        } else {
            header.children(platform_title_bar::render_left_window_controls(
                cx.button_layout(),
                Box::new(workspace::CloseWindow),
                window,
            ))
        };
        header
            .child(
                Label::new("PROJECTS")
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
            .child(div().flex_1())
            .child(self.render_add_project())
    }

    fn render_add_project(&self) -> impl IntoElement {
        let multi_workspace = self.multi_workspace.clone();
        let focus_handle = self.focus_handle.clone();
        div()
            .debug_selector(|| "marley-rail-add-project".into())
            .child(
                PopoverMenu::new("marley-rail-add-project")
                    .trigger(
                        IconButton::new("marley-rail-add-project-button", IconName::FolderAdd)
                            .icon_size(IconSize::Small)
                            .tooltip(Tooltip::text("Add Project")),
                    )
                    .menu(move |window, cx| {
                        let multi_workspace = multi_workspace.upgrade()?.read(cx);
                        let workspace = multi_workspace.workspace().downgrade();
                        let groups = multi_workspace.project_group_keys();
                        Some(SidebarRecentProjects::popover(
                            workspace,
                            groups,
                            focus_handle.clone(),
                            window,
                            cx,
                        ))
                    })
                    .with_handle(self.add_project_menu.clone())
                    .anchor(Anchor::TopRight),
            )
    }

    fn render_project_row(
        row: ProjectRow,
        group: &GroupEntry,
        agent_search_path: Option<OsString>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        // Element ids follow the workspace, not the row's position, so an open menu stays with
        // its project when another group is inserted above it.
        let id = group.workspace.entity_id();
        let index = row.index;
        let workspace = group.workspace.clone();
        let key = group.key.clone();
        div()
            .debug_selector(move || format!("marley-rail-project-{index}"))
            .child(
                ListItem::new(("marley-rail-project", id))
                    .toggle_state(row.selected)
                    .start_slot(
                        div()
                            .debug_selector(move || format!("marley-rail-disclosure-{index}"))
                            .child(
                                Disclosure::new(("marley-rail-disclosure", id), row.expanded)
                                    .on_click(cx.listener(move |rail, _, window, cx| {
                                        rail.toggle_expanded(&key, window, cx);
                                    })),
                            ),
                    )
                    .child(Label::new(row.name).size(LabelSize::Small))
                    .end_slot(
                        h_flex()
                            .gap_1()
                            .when(row.attention, |slot| {
                                slot.child(
                                    div()
                                        .debug_selector(move || {
                                            format!("marley-rail-attention-{index}")
                                        })
                                        .child(Indicator::dot().color(Color::Accent)),
                                )
                            })
                            .child(Self::render_project_menu(
                                index,
                                id,
                                group,
                                agent_search_path,
                                cx,
                            )),
                    )
                    .on_click(cx.listener(move |rail, _, window, cx| {
                        rail.activate_workspace(&workspace, window, cx).log_err();
                    })),
            )
    }

    fn render_project_menu(
        index: usize,
        id: EntityId,
        group: &GroupEntry,
        agent_search_path: Option<OsString>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let rail = cx.entity().downgrade();
        let workspace = group.workspace.clone();
        div()
            .debug_selector(move || format!("marley-rail-project-menu-{index}"))
            .child(
                PopoverMenu::new(("marley-rail-project-menu", id))
                    .trigger(
                        IconButton::new(("marley-rail-project-plus", id), IconName::Plus)
                            .icon_size(IconSize::Small)
                            .tooltip(Tooltip::text("New in this project")),
                    )
                    .menu(move |window, cx| {
                        let rail = rail.clone();
                        let workspace = workspace.clone();
                        let agent_search_path = agent_search_path.clone();
                        Some(ContextMenu::build(window, cx, move |menu, _, _| {
                            let terminal_rail = rail.clone();
                            let terminal_workspace = workspace.clone();
                            let cli_rail = rail.clone();
                            let cli_workspace = workspace.clone();
                            let menu = menu
                                .entry("New Terminal", None, move |window, cx| {
                                    terminal_rail
                                        .update(cx, |rail, cx| {
                                            rail.new_terminal(&terminal_workspace, window, cx)
                                        })
                                        .flatten()
                                        .log_err();
                                })
                                .submenu("New Agent Thread", move |menu, _, cx| {
                                    Self::agent_menu(menu, &rail, &workspace, cx)
                                });
                            Self::agent_cli_entries(
                                menu,
                                &cli_rail,
                                &cli_workspace,
                                agent_search_path.as_deref(),
                            )
                        }))
                    })
                    .anchor(Anchor::TopRight),
            )
    }

    /// The New Agent Thread entries: the Zed Agent, then every agent the project's agent servers
    /// list, as the Agent Panel's own menu names and orders them.
    fn agent_menu(
        menu: ContextMenu,
        rail: &WeakEntity<Self>,
        workspace: &WeakEntity<Workspace>,
        cx: &App,
    ) -> ContextMenu {
        let choices = workspace
            .upgrade()
            .map(|workspace| agent_choices(workspace.read(cx).project(), cx))
            .unwrap_or_default();
        choices.into_iter().fold(menu, |menu, (agent, name, icon)| {
            let rail = rail.clone();
            let workspace = workspace.clone();
            let entry = ContextMenuEntry::new(name);
            let entry = match icon {
                AgentIcon::Named(icon) => entry.icon(icon),
                AgentIcon::Svg(path) => entry.custom_icon_svg(path),
            };
            menu.item(entry.icon_color(Color::Muted).handler(move |window, cx| {
                rail.update(cx, |rail, cx| {
                    rail.new_agent_thread(&workspace, &agent, window, cx)
                })
                .flatten()
                .log_err();
            }))
        })
    }

    /// The agent CLIs the search path holds, one entry each under a header, after New Agent
    /// Thread; nothing when none is installed.
    fn agent_cli_entries(
        menu: ContextMenu,
        rail: &WeakEntity<Self>,
        workspace: &WeakEntity<Workspace>,
        search_path: Option<&OsStr>,
    ) -> ContextMenu {
        let agents = agents_on_path(search_path);
        if agents.is_empty() {
            return menu;
        }
        agents
            .into_iter()
            .fold(menu.separator().header("Agent CLIs"), |menu, kind| {
                let rail = rail.clone();
                let workspace = workspace.clone();
                menu.item(
                    ContextMenuEntry::new(kind.display_name())
                        .icon(agent_icon_name(kind))
                        .icon_color(Color::Muted)
                        .handler(move |window, cx| {
                            rail.update(cx, |rail, cx| {
                                rail.new_agent(&workspace, kind, window, cx)
                            })
                            .flatten()
                            .log_err();
                        }),
                )
            })
    }

    fn render_thread_row(
        row: ThreadRow,
        thread: &ThreadEntry,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let key = row.key.clone();
        let status = ui_status(row.status);
        let item = ThreadItem::new(
            SharedString::from(format!("marley-rail-thread-{key}")),
            row.title,
        )
        .status(status)
        .notified(row.attention)
        .selected(row.selected)
        .rounded(true)
        .base_bg(cx.theme().colors().panel_background);
        let item = match &thread.icon {
            AgentIcon::Named(icon) => item.icon(*icon),
            AgentIcon::Svg(path) => item.custom_icon_from_external_svg(path.clone()),
        };
        div()
            .debug_selector({
                let key = key.clone();
                move || format!("marley-rail-thread-{key}")
            })
            // The terminal rows' indent, so a thread's icon lines up under its project.
            .pl(px(12.))
            .child(item.on_click(cx.listener(move |rail, _, window, cx| {
                rail.open_thread(&key, window, cx).log_err();
            })))
    }

    fn render_terminal_row(
        row: TerminalRow,
        terminal: &TerminalEntry,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let workspace = terminal.workspace.clone();
        let view = terminal.view.clone();
        let id = row.id;
        div()
            .debug_selector(move || format!("marley-rail-terminal-{id}"))
            .child(
                ListItem::new(("marley-rail-terminal", id))
                    .toggle_state(row.selected)
                    .indent_level(1)
                    .start_slot(
                        div()
                            .when(row.agent.is_some(), |slot| {
                                slot.debug_selector(move || format!("marley-rail-agent-{id}"))
                            })
                            .child(
                                Icon::new(row.agent.map_or(IconName::Terminal, |agent| {
                                    agent_icon_name(agent.kind)
                                }))
                                .size(IconSize::Small)
                                .color(Color::Muted),
                            ),
                    )
                    .child(
                        v_flex()
                            .child(Label::new(row.title).size(LabelSize::Small))
                            // Always drawn, empty when unknown, so every terminal row has one
                            // height.
                            .child(
                                Label::new(row.subtitle.unwrap_or_default())
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            ),
                    )
                    .when(row.bell, |item| {
                        item.end_slot(
                            div()
                                .debug_selector(move || format!("marley-rail-bell-{id}"))
                                .child(Indicator::dot().color(Color::Accent)),
                        )
                    })
                    .on_click(cx.listener(move |rail, _, window, cx| {
                        rail.activate_terminal(&workspace, &view, window, cx)
                            .log_err();
                    })),
            )
    }
}

/// Everything in a window the rail follows.
#[derive(Default)]
struct Watched {
    workspaces: Vec<Entity<Workspace>>,
    views: Vec<Entity<TerminalView>>,
    panels: Vec<Entity<AgentPanel>>,
    threads: Vec<Entity<AcpThread>>,
    agent_servers: Vec<Entity<AgentServerStore>>,
}

impl Watched {
    fn in_window(multi_workspace: &Entity<MultiWorkspace>, cx: &App) -> Self {
        let workspaces: Vec<Entity<Workspace>> =
            multi_workspace.read(cx).workspaces().cloned().collect();
        let views = workspaces
            .iter()
            .flat_map(|workspace| workspace.read(cx).items_of_type::<TerminalView>(cx))
            .collect();
        let panels: Vec<Entity<AgentPanel>> = workspaces
            .iter()
            .filter_map(|workspace| workspace.read(cx).panel::<AgentPanel>(cx))
            .collect();
        let threads = panels
            .iter()
            .flat_map(|panel| live_threads(panel, cx))
            .collect();
        let agent_servers = workspaces
            .iter()
            .map(|workspace| {
                workspace
                    .read(cx)
                    .project()
                    .read(cx)
                    .agent_server_store()
                    .clone()
            })
            .collect();
        Self {
            workspaces,
            views,
            panels,
            threads,
            agent_servers,
        }
    }
}

/// Keeps each entity's subscriptions, makes them for each new entity, and returns the map for
/// what exists now; whatever the old map still holds is dropped, ending those subscriptions.
fn resubscribe<E: 'static, S>(
    old: &mut HashMap<EntityId, S>,
    entities: &[Entity<E>],
    mut subscribe: impl FnMut(&Entity<E>) -> S,
) -> HashMap<EntityId, S> {
    entities
        .iter()
        .map(|entity| {
            let subscriptions = old
                .remove(&entity.entity_id())
                .unwrap_or_else(|| subscribe(entity));
            (entity.entity_id(), subscriptions)
        })
        .collect()
}

/// The known agent CLIs `search_path` holds an executable for, in menu order.
fn agents_on_path(search_path: Option<&OsStr>) -> Vec<AgentKind> {
    AgentKind::ALL
        .into_iter()
        .filter(|kind| which::which_in(kind.program(), search_path, "/").is_ok())
        .collect()
}

/// The icon an agent CLI's row and menu entry draw.
const fn agent_icon_name(kind: AgentKind) -> IconName {
    match kind {
        AgentKind::Claude => IconName::AiClaude,
        AgentKind::Codex => IconName::AiOpenAi,
        AgentKind::Gemini => IconName::AiGemini,
        AgentKind::OpenCode => IconName::AiOpenCode,
    }
}

/// The status Zed's thread row draws for a rail status.
const fn ui_status(status: ThreadStatus) -> AgentThreadStatus {
    match status {
        ThreadStatus::Done => AgentThreadStatus::Completed,
        ThreadStatus::Running => AgentThreadStatus::Running,
        ThreadStatus::Waiting => AgentThreadStatus::WaitingForConfirmation,
        ThreadStatus::Error => AgentThreadStatus::Error,
    }
}

/// The events after which a thread's row may read differently: its status, its title, or a
/// confirmation asked or answered. Streamed output changes none of these.
const fn changes_the_row(event: &AcpThreadEvent) -> bool {
    matches!(
        event,
        AcpThreadEvent::StatusChanged
            | AcpThreadEvent::TitleUpdated
            | AcpThreadEvent::ToolAuthorizationRequested(_)
            | AcpThreadEvent::ToolAuthorizationReceived(_)
            | AcpThreadEvent::Stopped(_)
            | AcpThreadEvent::Error
            | AcpThreadEvent::LoadError(_)
            | AcpThreadEvent::Refusal
    )
}

/// The root thread of each conversation the panel holds, shown or kept in the background.
fn live_threads(panel: &Entity<AgentPanel>, cx: &App) -> Vec<Entity<AcpThread>> {
    panel
        .read(cx)
        .conversation_views()
        .into_iter()
        .filter_map(|conversation| {
            let view = conversation.read(cx).root_thread_view()?;
            Some(view.read(cx).thread.clone())
        })
        .collect()
}

/// The status of every thread the workspaces' panels hold, by thread id, and the thread each
/// panel shows.
fn live_statuses(
    workspaces: &[Entity<Workspace>],
    cx: &App,
) -> (HashMap<ThreadId, ThreadStatus>, HashSet<ThreadId>) {
    let mut statuses = HashMap::default();
    let mut active = HashSet::default();
    for panel in workspaces
        .iter()
        .filter_map(|workspace| workspace.read(cx).panel::<AgentPanel>(cx))
    {
        let panel = panel.read(cx);
        active.extend(panel.active_thread_id(cx));
        // A conversation still connecting has no thread yet, and nothing to report.
        let loaded = panel
            .conversation_views()
            .into_iter()
            .filter_map(|conversation| {
                let view = conversation.read(cx).root_thread_view()?;
                Some((conversation, view))
            });
        for (conversation, view) in loaded {
            let conversation = conversation.read(cx);
            let thread = view.read(cx).thread.read(cx);
            statuses.insert(
                conversation.parent_id(),
                marley_rail::thread_status(
                    conversation.root_thread_has_pending_tool_call(cx),
                    thread.had_error(),
                    thread.status() == acp_thread::ThreadStatus::Generating,
                ),
            );
        }
    }
    (statuses, active)
}

/// A group's threads, newest first. The metadata store files a thread under the main worktree
/// paths of the project it ran in; rows written before those were kept are found by their folder
/// paths, and a row whose paths disagree with its group by its workspace's own roots. Drafts are
/// listed only while their panel shows them, so a new thread appears at once.
fn group_threads(
    group: &ProjectGroup,
    listed: &Entity<Workspace>,
    cx: &App,
) -> Vec<(ThreadSnapshot, ThreadEntry)> {
    let Some(store) = ThreadMetadataStore::try_global(cx) else {
        return Vec::new();
    };
    let store = store.read(cx);
    let host = group.key.host();
    let members: Vec<(PathList, &Entity<Workspace>)> = group
        .workspaces
        .iter()
        .map(|workspace| (PathList::new(&workspace.read(cx).root_paths(cx)), workspace))
        .collect();
    let (statuses, active) = live_statuses(&group.workspaces, cx);
    let mut seen: HashSet<ThreadId> = HashSet::default();
    let mut rows: Vec<&ThreadMetadata> = store
        .entries_for_main_worktree_path(group.key.path_list(), host.as_ref())
        .chain(store.entries_for_path(group.key.path_list(), host.as_ref()))
        .chain(
            members
                .iter()
                .filter(|(paths, _)| !paths.paths().is_empty())
                .flat_map(|(paths, _)| store.entries_for_path(paths, host.as_ref())),
        )
        .filter(|row| seen.insert(row.thread_id))
        .filter(|row| !row.is_draft() || active.contains(&row.thread_id))
        .collect();
    rows.sort_by_key(|row| std::cmp::Reverse(row.interacted_at.unwrap_or(row.updated_at)));
    rows.into_iter()
        .map(|row| {
            let workspace = members
                .iter()
                .find(|(paths, _)| paths == row.folder_paths())
                .map_or(listed, |(_, workspace)| workspace);
            let key = row.thread_id.to_key_string();
            (
                ThreadSnapshot {
                    key,
                    title: row.display_title().to_string(),
                    status: statuses.get(&row.thread_id).copied().unwrap_or_default(),
                    attention: false,
                },
                ThreadEntry {
                    workspace: workspace.downgrade(),
                    thread_id: row.thread_id,
                    agent: Agent::from(row.agent_id.clone()),
                    work_dirs: row.folder_paths().clone(),
                    title: row.title(),
                    icon: agent_icon(&row.agent_id, workspace.read(cx).project(), cx),
                },
            )
        })
        .collect()
}

/// An agent's icon, looked up as the Agent Panel's menu looks it up.
fn agent_icon(agent: &AgentId, project: &Entity<Project>, cx: &App) -> AgentIcon {
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

/// The agents a new thread can run: the Zed Agent first, then the project's agent servers sorted
/// by display name without regard to case, each with its name and icon.
fn agent_choices(project: &Entity<Project>, cx: &App) -> Vec<(AgentId, SharedString, AgentIcon)> {
    let servers = project.read(cx).agent_server_store().read(cx);
    let registry = AgentRegistryStore::try_global(cx);
    let registry = registry.as_ref().map(|registry| registry.read(cx));
    let mut external: Vec<(AgentId, SharedString)> = servers
        .external_agents()
        .map(|agent| {
            let name = servers
                .agent_display_name(agent)
                .or_else(|| Some(registry?.agent(agent)?.name().clone()))
                .unwrap_or_else(|| agent.0.clone());
            (agent.clone(), name)
        })
        .collect();
    external.sort_by_key(|(_, name)| name.to_lowercase());
    let zed_agent = Agent::NativeAgent.id();
    std::iter::once((zed_agent, SharedString::from("Zed Agent")))
        .chain(external)
        .map(|(agent, name)| {
            let icon = agent_icon(&agent, project, cx);
            (agent, name, icon)
        })
        .collect()
}

/// One terminal's row: an agent row when a known agent CLI runs in its foreground, labelled with
/// the CLI's own title and its status, else the terminal's title and working directory.
fn terminal_snapshot(
    view: &Entity<TerminalView>,
    root: Option<&std::path::Path>,
    home: &std::path::Path,
    foreground_command: ForegroundCommand,
    last_output: Option<Instant>,
    now: Instant,
    cx: &App,
) -> TerminalSnapshot {
    let terminal_view = view.read(cx);
    let terminal = terminal_view.terminal();
    let bell = terminal_view.has_bell();
    // No output seen yet counts as quiet: the agent is at its prompt, as far as the rail knows.
    let agent = foreground_command(terminal, cx)
        .as_deref()
        .and_then(marley_agent::agent_kind_of)
        .map(|kind| {
            let quiet_for =
                last_output.map_or(Duration::MAX, |at| now.saturating_duration_since(at));
            TerminalAgent {
                kind,
                status: marley_agent::agent_status(quiet_for, bell),
            }
        });
    let (title, subtitle) = agent.map_or_else(
        || {
            (
                terminal_view.tab_content_text(0, cx).to_string(),
                marley_rail::working_directory_label(
                    terminal.read(cx).working_directory().as_deref(),
                    root,
                    Some(home),
                ),
            )
        },
        |agent| {
            (
                agent_title(&terminal.read(cx).breadcrumb_text, agent.kind),
                Some(marley_agent::status_line(agent.kind, agent.status)),
            )
        },
    );
    TerminalSnapshot {
        id: view.entity_id().as_u64(),
        title,
        subtitle,
        bell,
        agent,
    }
}

/// An agent row's title: the title the CLI set over OSC, else the agent's name.
fn agent_title(breadcrumb: &str, kind: AgentKind) -> String {
    if breadcrumb.trim().is_empty() {
        kind.display_name().to_string()
    } else {
        breadcrumb.to_string()
    }
}

/// The window, read once. Only groups with an open workspace are listed; a group Zed keeps after
/// its last workspace closed has nothing for the rail to switch to.
fn build_snapshot(
    multi_workspace: &Entity<MultiWorkspace>,
    foreground_command: ForegroundCommand,
    terminal_output: &HashMap<EntityId, Instant>,
    window: &Window,
    cx: &App,
) -> Snapshot {
    let multi_workspace = multi_workspace.read(cx);
    let groups: Vec<ProjectGroup> = multi_workspace
        .project_groups(cx)
        .into_iter()
        .filter(|group| !group.workspaces.is_empty())
        .collect();
    let names = group_names(&groups);
    let displayed = multi_workspace.workspace();
    let home = util::paths::home_dir().as_path();
    let now = cx.background_executor().now();
    let mut snapshot = Snapshot::default();
    let listed = groups.iter().zip(names).filter_map(|(group, name)| {
        let workspace = multi_workspace
            .last_active_workspace_for_group(&group.key, cx)
            .or_else(|| group.workspaces.first().cloned())?;
        Some((group, name, workspace))
    });
    for (group, name, workspace) in listed {
        let mut terminals = Vec::new();
        for member in &group.workspaces {
            // Subtitles read against the member's own first root: a linked worktree's, not the
            // main repository's the group is keyed by.
            let root = member
                .read(cx)
                .project()
                .read(cx)
                .visible_worktrees(cx)
                .find_map(|worktree| worktree.read(cx).root_dir());
            for view in member.read(cx).items_of_type::<TerminalView>(cx) {
                let id = view.entity_id().as_u64();
                let terminal = terminal_snapshot(
                    &view,
                    root.as_deref(),
                    home,
                    foreground_command,
                    terminal_output.get(&view.entity_id()).copied(),
                    now,
                    cx,
                );
                if terminal.agent.is_some() {
                    snapshot.agent_terminals.insert(view.entity_id());
                }
                terminals.push(terminal);
                snapshot.terminals.insert(
                    id,
                    TerminalEntry {
                        workspace: member.downgrade(),
                        view: view.downgrade(),
                    },
                );
            }
        }
        let mut threads = Vec::new();
        for (thread, entry) in group_threads(group, &workspace, cx) {
            snapshot.threads.insert(thread.key.clone(), entry);
            threads.push(thread);
        }
        snapshot.rail.projects.push(ProjectSnapshot {
            name,
            expanded: group.expanded,
            terminals,
            threads,
        });
        snapshot.groups.push(GroupEntry {
            key: group.key.clone(),
            workspace: workspace.downgrade(),
        });
    }
    let panel = displayed.read(cx).panel::<AgentPanel>(cx);
    let panel_thread = panel
        .as_ref()
        .and_then(|panel| panel.read(cx).active_thread_id(cx))
        .map(|thread_id| thread_id.to_key_string());
    snapshot.shown_thread = panel_thread
        .clone()
        .filter(|_| AgentPanel::is_visible(displayed, cx));
    snapshot.rail.focus = Focus {
        project: groups
            .iter()
            .position(|group| group.workspaces.contains(displayed)),
        terminal: displayed
            .read(cx)
            .active_item(cx)
            .and_then(|item| item.downcast::<TerminalView>())
            .map(|view| view.entity_id().as_u64()),
        thread: panel_thread.filter(|_| {
            panel
                .as_ref()
                .is_some_and(|panel| panel.focus_handle(cx).contains_focused(window, cx))
        }),
    };
    snapshot
}

/// The label for each group: its roots' last components, with parent components added where two
/// groups' labels would otherwise read the same, through the public naming functions Zed's own
/// project groups use (`compute_disambiguation_details`, `path_suffix`, `display_name`).
fn group_names(groups: &[ProjectGroup]) -> Vec<String> {
    let roots: BTreeSet<PathBuf> = groups
        .iter()
        .flat_map(|group| group.key.path_list().paths().to_vec())
        .collect();
    let roots: Vec<PathBuf> = roots.into_iter().collect();
    let depths = util::disambiguate::compute_disambiguation_details(&roots, |root, depth| {
        project::path_suffix(root, depth)
    });
    let depth_by_root: HashMap<PathBuf, usize> = roots.into_iter().zip(depths).collect();
    groups
        .iter()
        .map(|group| group.key.display_name(&depth_by_root).to_string())
        .collect()
}

impl EventEmitter<SidebarEvent> for Rail {}

impl Focusable for Rail {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// The `MultiWorkspace` calls these while it is being updated, so none of them reads it.
impl Sidebar for Rail {
    fn width(&self, _cx: &App) -> Pixels {
        self.width
    }

    fn set_width(&mut self, width: Option<Pixels>, cx: &mut Context<Self>) {
        // The resize handle passes the raw pointer position.
        self.width = width.unwrap_or(DEFAULT_WIDTH).clamp(MIN_WIDTH, MAX_WIDTH);
        cx.notify();
    }

    fn has_notifications(&self, _cx: &App) -> bool {
        marley_rail::has_attention(&self.snapshot.rail)
    }

    // `SidebarSide`'s default is the left, the rail's side.
    fn side(&self, _cx: &App) -> SidebarSide {
        SidebarSide::default()
    }

    // `true` would silence every thread's OS notification in the window while the rail is
    // open, the Agent Panel's terminal threads included, which the rail does not list.
    fn is_threads_list_view_active(&self) -> bool {
        false
    }

    // The window's saved sidebar state stays Zed's sidebar's while the rail stands in for it.
    fn serialized_state(&self, cx: &App) -> Option<String> {
        self.zed_sidebar
            .as_ref()
            .and_then(|(sidebar, _)| sidebar.read(cx).serialized_state(cx))
            .or_else(|| self.zed_sidebar_state.clone())
    }

    fn restore_serialized_state(
        &mut self,
        state: &str,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.zed_sidebar_state = Some(state.to_string());
    }
}

impl Render for Rail {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows: Vec<AnyElement> = marley_rail::rail_rows(&self.snapshot.rail)
            .into_iter()
            .filter_map(|row| match row {
                Row::Project(row) => self.snapshot.groups.get(row.index).map(|group| {
                    Self::render_project_row(row, group, self.agent_search_path.clone(), cx)
                        .into_any_element()
                }),
                Row::Terminal(row) => self.snapshot.terminals.get(&row.id).map(|terminal| {
                    Self::render_terminal_row(row, terminal, cx).into_any_element()
                }),
                Row::Thread(row) => self
                    .snapshot
                    .threads
                    .get(&row.key)
                    .map(|thread| Self::render_thread_row(row, thread, cx).into_any_element()),
            })
            .collect();
        v_flex()
            .id("marley-rail")
            .key_context("MarleyRail")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(self.render_header(window, cx))
            .child(
                v_flex()
                    .id("marley-rail-rows")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_1()
                    .gap_px()
                    .children(rows),
            )
    }
}

#[cfg(test)]
#[path = "rail_tests.rs"]
mod tests;
