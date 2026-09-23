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
use agent_ui::threads_archive_view::fuzzy_match_positions;
use agent_ui::{Agent, AgentPanel, AgentPanelEvent, AgentThreadSource};
use anyhow::Context as _;
use editor::{Editor, EditorEvent};
use gpui::{
    Anchor, AnyElement, AnyView, App, ClickEvent, Context, Entity, EntityId, EventEmitter,
    FocusHandle, Focusable, Pixels, Render, Subscription, Task, WeakEntity, Window, px,
};
use marley_agent::{AgentKind, WAITING_AFTER};
use marley_rail::{
    Focus, ProjectRow, ProjectSnapshot, RailSnapshot, Row, Selection, SwitcherRow, TerminalAgent,
    TerminalRow, TerminalSnapshot, ThreadRow, ThreadSnapshot, ThreadStatus,
};
use menu::{
    Cancel, Confirm, SelectChild, SelectFirst, SelectLast, SelectNext, SelectParent, SelectPrevious,
};
use project::{AgentId, AgentServerStore, AgentServersUpdated, ProjectGroupKey};
use recent_projects::sidebar_recent_projects::SidebarRecentProjects;
use terminal::Terminal;
use terminal_view::{RenameTerminal, TerminalView, terminal_panel::TerminalPanel};
use ui::{
    AgentThreadStatus, ContextMenu, ContextMenuEntry, Disclosure, HighlightedLabel, Icon,
    IconButton, IconName, IconSize, Indicator, KeyBinding, Label, LabelSize, ListItem, PopoverMenu,
    PopoverMenuHandle, ThreadItem, Tooltip, prelude::*, right_click_menu,
    utils::platform_title_bar_height,
};
use util::ResultExt as _;
use util::path_list::PathList;
use workspace::{
    MultiWorkspace, MultiWorkspaceEvent, ProjectGroup, SaveIntent, Sidebar, SidebarEvent,
    SidebarSide, Workspace,
    item::{Item as _, ItemEvent},
    notifications::DetachAndPromptErr as _,
};
use zed_actions::agents_sidebar::FocusSidebarFilter;

use crate::agents::{self, AgentIcon};

#[path = "rail_switcher.rs"]
mod switcher;

use switcher::{RailSwitcher, SwitcherEntry, SwitcherEvent};

const DEFAULT_WIDTH: Pixels = px(260.);
const MIN_WIDTH: Pixels = px(180.);
const MAX_WIDTH: Pixels = px(600.);

/// Reads the command a terminal's foreground process runs. Production asks the PTY; a test's
/// display-only terminal has no process, so tests hand in their own.
type ForegroundCommand = fn(&Entity<Terminal>, &App) -> Option<String>;

/// Zed's own sidebar and whether it was open, kept by the rail that replaced it.
pub type KeptSidebar = (Entity<sidebar::Sidebar>, bool);

/// The Marley layout's sidebar: each project group, with the terminals in its center panes and
/// its agent threads under it.
pub struct Rail {
    multi_workspace: WeakEntity<MultiWorkspace>,
    focus_handle: FocusHandle,
    width: Pixels,
    /// Whether `width` came from the user, which is when it is saved.
    width_set_by_user: bool,
    /// Whether the user closed the rail, which the window's saved state keeps.
    closed: bool,
    /// The row the keyboard is on while the rail holds focus.
    cursor: Option<Selection>,
    /// The filter's field, under the header.
    filter_editor: Entity<Editor>,
    /// When each terminal and thread last took the window's focus, as a count of the changes of
    /// the window's row: the switcher lists the highest first.
    shown_at: HashMap<Selection, u64>,
    shown_count: u64,
    /// The terminal or thread row that held the window's focus at the last rebuild.
    window_row: Option<Selection>,
    /// The switcher, while it is open.
    switcher: Option<OpenSwitcher>,
    foreground_command: ForegroundCommand,
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
    /// Zed's sidebar state restored into a window that opened in the Marley layout. The rail
    /// reads its own fields from it and writes it back with every other field kept.
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
    _focus_out: Subscription,
    _filter_edits: Subscription,
}

impl std::fmt::Debug for Rail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rail")
            .field("width", &self.width)
            .field("rows", &self.snapshot.rail)
            .finish_non_exhaustive()
    }
}

/// The open switcher, what had focus before it, and its events.
struct OpenSwitcher {
    view: Entity<RailSwitcher>,
    return_focus: Option<FocusHandle>,
    _events: Subscription,
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
            // Re-keying a project group notifies without an event, and so do opening and
            // closing the sidebar.
            cx.observe_in(
                multi_workspace,
                window,
                |rail, multi_workspace, window, cx| {
                    rail.note_open(&multi_workspace, cx);
                    rail.refresh(window, cx);
                },
            ),
        ];
        // Built over Zed's sidebar, the rail starts at its width: one width for both layouts.
        let saved = zed_sidebar
            .as_ref()
            .and_then(|(sidebar, _)| sidebar.read(cx).serialized_state(cx))
            .map(|blob| read_rail_state(&blob))
            .unwrap_or_default();
        // The `MultiWorkspace` may be mid-update while its sidebar is built, so the first read
        // waits for the end of this effect cycle.
        cx.defer_in(window, Self::refresh);
        let focus_handle = cx.focus_handle();
        // The keyboard's row is the selection only while the rail holds focus.
        let focus_out = cx.on_focus_out(&focus_handle, window, |rail, _, window, cx| {
            rail.cursor = None;
            rail.refresh(window, cx);
        });
        let filter_editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Filter…", window, cx);
            editor
        });
        let filter_edits = cx.subscribe_in(
            &filter_editor,
            window,
            |rail, _, event: &EditorEvent, window, cx| {
                if matches!(event, EditorEvent::BufferEdited) {
                    rail.filter_edited(window, cx);
                }
            },
        );
        Self {
            multi_workspace: multi_workspace.downgrade(),
            focus_handle,
            width: saved
                .width
                .map_or(DEFAULT_WIDTH, |width| px(width).clamp(MIN_WIDTH, MAX_WIDTH)),
            width_set_by_user: saved.width.is_some(),
            closed: false,
            cursor: None,
            filter_editor,
            shown_at: HashMap::default(),
            shown_count: 0,
            window_row: None,
            switcher: None,
            foreground_command: |terminal, cx| terminal.read(cx).foreground_process_command_name(),
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
            _focus_out: focus_out,
            _filter_edits: filter_edits,
        }
    }

    /// Hands Zed's sidebar back for the switch to the Zed layout: the one the rail kept, and the
    /// state to restore into a fresh one when it kept none, with the rail's width in it.
    pub(crate) fn take_zed_sidebar(&mut self) -> (Option<KeptSidebar>, Option<String>) {
        let state = write_rail_state(self.zed_sidebar_state.take().as_deref(), self.rail_state());
        (self.zed_sidebar.take(), Some(state))
    }

    /// Notes whether the user has closed the rail. Nothing is noted while the sidebar cannot be
    /// shown at all.
    fn note_open(&mut self, multi_workspace: &Entity<MultiWorkspace>, cx: &App) {
        let multi_workspace = multi_workspace.read(cx);
        if multi_workspace.multi_workspace_enabled(cx) {
            self.closed = !multi_workspace.sidebar_open();
        }
    }

    fn rail_state(&self) -> RailState {
        RailState {
            width: self.width_set_by_user.then(|| f32::from(self.width)),
            closed: self.closed,
        }
    }

    /// Follows every workspace, terminal, Agent Panel and live thread, rereads the window, and
    /// redraws only when what the rail shows has changed.
    fn refresh(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.sync_subscriptions(window, cx);
        let filter = self.filter_editor.read(cx).text(cx);
        let mut snapshot = self
            .multi_workspace
            .upgrade()
            .map(|multi_workspace| {
                build_snapshot(
                    &multi_workspace,
                    self.foreground_command,
                    &self.terminal_output,
                    &filter,
                    window,
                    cx,
                )
            })
            .unwrap_or_default();
        self.note_ended_runs(&mut snapshot);
        self.note_window_row(&snapshot.rail);
        if self.focus_handle.contains_focused(window, cx) {
            snapshot.rail.focus.cursor.clone_from(&self.cursor);
        }
        if snapshot.rail != self.snapshot.rail {
            cx.notify();
        }
        self.snapshot = snapshot;
    }

    /// Notes a change of the terminal or thread row that holds the window's focus, for the
    /// switcher's order, and forgets the rows that are gone. The switcher's own focus is no row,
    /// so opening it notes nothing.
    fn note_window_row(&mut self, rail: &RailSnapshot) {
        let row = marley_rail::window_row(rail);
        if row != self.window_row {
            if let Some(row) = &row {
                self.shown_count += 1;
                self.shown_at.insert(row.clone(), self.shown_count);
            }
            self.window_row = row;
        }
        let listed: HashSet<Selection> = marley_rail::switcher_rows(rail, |_| None)
            .iter()
            .map(SwitcherRow::selection)
            .collect();
        self.shown_at.retain(|row, _| listed.contains(row));
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

    /// Shows a terminal and starts Zed's rename on its tab, as the tab's own Rename does: the name
    /// is edited in the tab and kept on Enter.
    fn rename_terminal(
        &self,
        workspace: &WeakEntity<Workspace>,
        view: &WeakEntity<TerminalView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        self.activate_terminal(workspace, view, window, cx)?;
        let view = view.upgrade().context("the terminal was closed")?;
        view.update(cx, |view, cx| {
            view.rename_terminal(&RenameTerminal, window, cx);
        });
        Ok(())
    }

    /// Closes a terminal through its pane, as its tab's close does: Zed asks first while a task
    /// runs in it.
    fn close_terminal(
        workspace: &WeakEntity<Workspace>,
        view: &WeakEntity<TerminalView>,
        window: &mut Window,
        cx: &mut App,
    ) -> anyhow::Result<()> {
        let view = view.upgrade().context("the terminal was closed")?;
        let workspace = workspace.upgrade().context("the project was closed")?;
        let pane = workspace
            .read(cx)
            .pane_for(&view)
            .context("the terminal is in no pane")?;
        let closing = pane.update(cx, |pane, cx| {
            pane.close_item_by_id(view.entity_id(), SaveIntent::Close, window, cx)
        });
        closing.detach_and_prompt_err("Could not close the terminal", window, cx, |_, _, _| None);
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
        let factory = agents::launcher(cx).terminal_factory;
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

    /// Shows `workspace` and starts `kind` in a new center terminal there.
    fn new_agent(
        &self,
        workspace: &WeakEntity<Workspace>,
        kind: AgentKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let workspace = self.activate_workspace(workspace, window, cx)?;
        workspace.update(cx, |workspace, cx| {
            agents::start_cli(workspace, kind, window, cx);
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

    /// Shows `workspace` and starts a thread of `agent` in its Agent Panel.
    fn new_agent_thread(
        &self,
        workspace: &WeakEntity<Workspace>,
        agent: &AgentId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let workspace = self.activate_workspace(workspace, window, cx)?;
        workspace.update(cx, |workspace, cx| {
            agents::start_thread(workspace, agent, window, cx)
        })
    }

    /// Puts the keyboard's row where `to` says, from the rail as it stands.
    fn move_cursor(
        &mut self,
        to: impl FnOnce(&RailSnapshot) -> Selection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cursor = Some(to(&self.snapshot.rail));
        self.refresh(window, cx);
    }

    fn select_next(&mut self, _: &SelectNext, window: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(|rail| marley_rail::step(rail, true), window, cx);
    }

    fn select_previous(&mut self, _: &SelectPrevious, window: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(|rail| marley_rail::step(rail, false), window, cx);
    }

    fn select_first(&mut self, _: &SelectFirst, window: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(marley_rail::first_row, window, cx);
    }

    fn select_last(&mut self, _: &SelectLast, window: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(marley_rail::last_row, window, cx);
    }

    /// Left: folds an open project, or climbs from a row to its project's header.
    fn select_parent(&mut self, _: &SelectParent, window: &mut Window, cx: &mut Context<Self>) {
        let selected = marley_rail::selection(&self.snapshot.rail);
        match selected {
            Selection::Project(index) => self.fold(index, true, window, cx),
            row => self.move_cursor(|rail| marley_rail::parent(rail, &row), window, cx),
        }
    }

    /// Right: unfolds a folded project.
    fn select_child(&mut self, _: &SelectChild, window: &mut Window, cx: &mut Context<Self>) {
        if let Selection::Project(index) = marley_rail::selection(&self.snapshot.rail) {
            self.fold(index, false, window, cx);
        }
    }

    /// Folds or unfolds the project at `index`, when it is not that way already and no filter
    /// decides what shows.
    fn fold(&mut self, index: usize, fold: bool, window: &mut Window, cx: &mut Context<Self>) {
        let expanded = self
            .snapshot
            .rail
            .projects
            .get(index)
            .is_some_and(|project| project.expanded);
        if let Some(group) = self.snapshot.groups.get(index)
            && expanded == fold
            && !self.snapshot.rail.filtering
        {
            let key = group.key.clone();
            self.toggle_expanded(&key, window, cx);
        }
    }

    /// `ctrl-f`: the filter takes focus, as Zed's action gives its own sidebar's.
    fn focus_filter(
        &mut self,
        _: &FocusSidebarFilter,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.filter_editor.focus_handle(cx), cx);
    }

    /// Escape, as Zed's Threads Sidebar has it: it clears the filter, and from an empty filter
    /// goes back to the rows. With neither to do it passes on.
    fn cancel(&mut self, _: &Cancel, window: &mut Window, cx: &mut Context<Self>) {
        if self.snapshot.rail.filtering {
            self.clear_filter(window, cx);
        } else if self.filter_editor.focus_handle(cx).is_focused(window) {
            window.focus(&self.focus_handle, cx);
        } else {
            cx.propagate();
        }
    }

    fn clear_filter(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.filter_editor
            .update(cx, |editor, cx| editor.set_text("", window, cx));
    }

    /// Each edit rereads the rail and puts the keyboard on the first row that matched, as Zed's
    /// Threads Sidebar selects its first match.
    fn filter_edited(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.refresh(window, cx);
        if self.snapshot.rail.filtering {
            self.move_cursor(marley_rail::first_match, window, cx);
        }
    }

    /// Enter: what a click on the keyboard's row does.
    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let selection = marley_rail::selection(&self.snapshot.rail);
        self.open_row(selection, window, cx).log_err();
    }

    /// Opens a row as a click on it does.
    fn open_row(
        &self,
        selection: Selection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        match selection {
            Selection::None => Ok(()),
            Selection::Project(index) => {
                let group = self.snapshot.groups.get(index);
                let workspace = group.map(|group| group.workspace.clone());
                workspace.map_or(Ok(()), |workspace| {
                    self.activate_workspace(&workspace, window, cx).map(drop)
                })
            }
            Selection::Terminal(id) => {
                let terminal = self.snapshot.terminals.get(&id);
                let terminal =
                    terminal.map(|terminal| (terminal.workspace.clone(), terminal.view.clone()));
                terminal.map_or(Ok(()), |(workspace, view)| {
                    self.activate_terminal(&workspace, &view, window, cx)
                })
            }
            Selection::Thread(key) => self.open_thread(&key, window, cx),
        }
    }

    /// The switcher's rows, from the window as the rail last read it.
    fn switcher_entries(&self) -> Vec<SwitcherEntry> {
        let rail = &self.snapshot.rail;
        let project_name = |index: usize| {
            rail.projects
                .get(index)
                .map(|project| SharedString::from(project.name.clone()))
        };
        marley_rail::switcher_rows(rail, |row| self.shown_at.get(row).copied())
            .into_iter()
            .filter_map(|row| match row {
                SwitcherRow::Terminal(row) => {
                    let project = project_name(row.project)?;
                    Some(SwitcherEntry::Terminal { row, project })
                }
                SwitcherRow::Thread(row) => {
                    let project = project_name(row.project)?;
                    let icon = self.snapshot.threads.get(&row.key)?.icon.clone();
                    Some(SwitcherEntry::Thread { row, project, icon })
                }
            })
            .collect()
    }

    /// Closes the switcher and opens what it chose, or after Escape gives focus back.
    fn switcher_ended(
        &mut self,
        event: &SwitcherEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let return_focus = self.switcher.take().and_then(|open| open.return_focus);
        self.multi_workspace
            .update(cx, |multi_workspace, cx| {
                multi_workspace.set_sidebar_overlay(None, cx);
            })
            .log_err();
        match event {
            SwitcherEvent::Confirmed(selection) => {
                self.open_row(selection.clone(), window, cx).log_err();
            }
            SwitcherEvent::Cancelled { restore_focus } => {
                if let Some(focus) = return_focus.filter(|_| *restore_focus) {
                    window.focus(&focus, cx);
                }
            }
        }
    }

    /// Moves a project one place up or down in the window, as Zed's own reorder does.
    fn move_project(
        &mut self,
        key: &ProjectGroupKey,
        up: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let moved = self.multi_workspace.update(cx, |multi_workspace, cx| {
            if up {
                multi_workspace.move_project_group_up(key, cx)
            } else {
                multi_workspace.move_project_group_down(key, cx)
            }
        });
        moved.log_err();
        self.refresh(window, cx);
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

    fn render_filter(&self, cx: &Context<Self>) -> impl IntoElement {
        let filtering = self.snapshot.rail.filtering;
        h_flex()
            .w_full()
            .flex_none()
            .gap_1()
            .px_2()
            .py_1()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                Icon::new(IconName::MagnifyingGlass)
                    .size(IconSize::Small)
                    .color(Color::Muted),
            )
            .child(
                div()
                    .debug_selector(|| "marley-rail-filter".into())
                    .min_w_0()
                    .flex_1()
                    .child(self.filter_editor.clone()),
            )
            .map(|field| {
                if filtering {
                    field.child(
                        div()
                            .debug_selector(|| "marley-rail-filter-clear".into())
                            .child(
                                IconButton::new("marley-rail-filter-clear", IconName::Close)
                                    .icon_size(IconSize::Small)
                                    .tooltip(Tooltip::text("Clear Filter"))
                                    .on_click(cx.listener(|rail, _, window, cx| {
                                        rail.clear_filter(window, cx);
                                    })),
                            ),
                    )
                } else {
                    field.child(KeyBinding::for_action_in(
                        &FocusSidebarFilter,
                        &self.focus_handle,
                        cx,
                    ))
                }
            })
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
        last: bool,
        filtering: bool,
        agent_search_path: Option<OsString>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        // Element ids follow the workspace, not the row's position, so an open menu stays with
        // its project when another group is inserted above it.
        let id = group.workspace.entity_id();
        let index = row.index;
        let workspace = group.workspace.clone();
        let key = group.key.clone();
        let rail = cx.entity().downgrade();
        let menu_key = group.key.clone();
        let header = ListItem::new(("marley-rail-project", id))
            .toggle_state(row.selected)
            // While the filter decides which rows show, the header does not fold.
            .when(!filtering, |header| {
                header.start_slot(
                    div()
                        .debug_selector(move || format!("marley-rail-disclosure-{index}"))
                        .child(
                            Disclosure::new(("marley-rail-disclosure", id), row.expanded).on_click(
                                cx.listener(move |rail, _, window, cx| {
                                    rail.toggle_expanded(&key, window, cx);
                                }),
                            ),
                        ),
                )
            })
            .child(row_label(row.name, row.highlight))
            .end_slot(
                h_flex()
                    .gap_1()
                    .when(row.attention, |slot| {
                        slot.child(
                            div()
                                .debug_selector(move || format!("marley-rail-attention-{index}"))
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
            }));
        right_click_menu(("marley-rail-project-context", id))
            .trigger(move |_, _, _| {
                div()
                    .debug_selector(move || format!("marley-rail-project-{index}"))
                    .child(header)
            })
            .menu(move |window, cx| {
                let (rail, key) = (rail.clone(), menu_key.clone());
                ContextMenu::build(window, cx, move |menu, _, _| {
                    [
                        ("Move Project Up", true, index == 0),
                        ("Move Project Down", false, last),
                    ]
                    .into_iter()
                    .fold(menu, |menu, (label, up, at_the_end)| {
                        let (rail, key) = (rail.clone(), key.clone());
                        menu.item(ContextMenuEntry::new(label).disabled(at_the_end).handler(
                            move |window, cx| {
                                rail.update(cx, |rail, cx| {
                                    rail.move_project(&key, up, window, cx);
                                })
                                .log_err();
                            },
                        ))
                    })
                })
            })
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
            .map(|workspace| agents::thread_agents(workspace.read(cx).project(), cx))
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
        let agents = agents::installed_clis(search_path);
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
                        .icon(agents::cli_icon(kind))
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
        let item = thread_item(
            SharedString::from(format!("marley-rail-thread-{key}")),
            row,
            &thread.icon,
        )
        .base_bg(cx.theme().colors().panel_background);
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
        let id = row.id;
        let (workspace, view) = (terminal.workspace.clone(), terminal.view.clone());
        let (close_workspace, close_view) = (terminal.workspace.clone(), terminal.view.clone());
        let close = div()
            .debug_selector(move || format!("marley-rail-terminal-close-{id}"))
            .child(
                IconButton::new(("marley-rail-terminal-close", id), IconName::Close)
                    .icon_size(IconSize::Small)
                    .icon_color(Color::Muted)
                    .tooltip(Tooltip::text("Close Terminal"))
                    .on_click(cx.listener(move |_, _, window, cx| {
                        // The row under the button would show the terminal it closes.
                        cx.stop_propagation();
                        Self::close_terminal(&close_workspace, &close_view, window, cx).log_err();
                    })),
            );
        let item = ListItem::new(("marley-rail-terminal", id))
            .toggle_state(row.selected)
            .indent_level(1)
            .start_slot(
                div()
                    .when(row.agent.is_some(), |slot| {
                        slot.debug_selector(move || format!("marley-rail-agent-{id}"))
                    })
                    .child(
                        Icon::new(terminal_icon(&row))
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .child(
                v_flex()
                    .child(row_label(row.title, row.highlight))
                    // Always drawn, empty when unknown, so every terminal row has one height.
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
            .end_slot_on_hover(close)
            .on_click(cx.listener(move |rail, event: &ClickEvent, window, cx| {
                // The second click of a double-click renames, as a tab's does.
                if event.click_count() == 2 {
                    rail.rename_terminal(&workspace, &view, window, cx)
                        .log_err();
                } else {
                    rail.activate_terminal(&workspace, &view, window, cx)
                        .log_err();
                }
            }));
        let rail = cx.entity().downgrade();
        let (menu_workspace, menu_view) = (terminal.workspace.clone(), terminal.view.clone());
        right_click_menu(("marley-rail-terminal-menu", id))
            .trigger(move |_, _, _| {
                div()
                    .debug_selector(move || format!("marley-rail-terminal-{id}"))
                    .child(item)
            })
            .menu(move |window, cx| {
                let (rename_rail, rename_workspace, rename_view) =
                    (rail.clone(), menu_workspace.clone(), menu_view.clone());
                let (close_workspace, close_view) = (menu_workspace.clone(), menu_view.clone());
                ContextMenu::build(window, cx, move |menu, _, _| {
                    menu.entry("Rename", None, move |window, cx| {
                        rename_rail
                            .update(cx, |rail, cx| {
                                rail.rename_terminal(&rename_workspace, &rename_view, window, cx)
                            })
                            .flatten()
                            .log_err();
                    })
                    .entry("Close", None, move |window, cx| {
                        Self::close_terminal(&close_workspace, &close_view, window, cx).log_err();
                    })
                })
            })
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
/// What the rail keeps in the window's saved sidebar state, beside Zed's sidebar's fields.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct RailState {
    /// The width the user set, if any.
    width: Option<f32>,
    /// Whether the user closed the rail.
    closed: bool,
}

/// The fields of a saved sidebar blob the rail reads. `width` and `width_set_by_user` are the
/// names Zed's sidebar writes and reads; `marley_rail_closed` is the rail's own, which Zed's
/// sidebar ignores.
#[derive(Default, serde::Deserialize)]
struct SavedRail {
    #[serde(default)]
    width: Option<f32>,
    #[serde(default)]
    width_set_by_user: bool,
    #[serde(default)]
    marley_rail_closed: bool,
}

/// The rail's fields in a saved sidebar blob. A blob that cannot be read holds nothing for it.
fn read_rail_state(blob: &str) -> RailState {
    let saved: SavedRail = serde_json::from_str(blob).unwrap_or_default();
    RailState {
        width: saved.width.filter(|_| saved.width_set_by_user),
        closed: saved.marley_rail_closed,
    }
}

/// `zed_state`, the Zed sidebar's saved blob, with the rail's fields written into it and every
/// other field kept, so Zed's sidebar still restores its own state from it.
fn write_rail_state(zed_state: Option<&str>, state: RailState) -> String {
    let mut blob: serde_json::Map<String, serde_json::Value> = zed_state
        .and_then(|zed_state| serde_json::from_str(zed_state).ok())
        .unwrap_or_default();
    blob.insert("width".into(), serde_json::json!(state.width));
    blob.insert("width_set_by_user".into(), state.width.is_some().into());
    blob.insert("marley_rail_closed".into(), state.closed.into());
    serde_json::Value::Object(blob).to_string()
}

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

/// A terminal row's icon: its agent CLI's, or the terminal's.
fn terminal_icon(row: &TerminalRow) -> IconName {
    row.agent
        .map_or(IconName::Terminal, |agent| agents::cli_icon(agent.kind))
}

/// A thread's row as Zed's thread list draws it, with the agent's icon: the rail's rows and the
/// switcher's.
fn thread_item(id: SharedString, row: ThreadRow, icon: &AgentIcon) -> ThreadItem {
    let item = ThreadItem::new(id, row.title)
        .highlight_positions(row.highlight)
        .status(ui_status(row.status))
        .notified(row.attention)
        .selected(row.selected)
        .rounded(true);
    match icon {
        AgentIcon::Named(icon) => item.icon(*icon),
        AgentIcon::Svg(path) => item.custom_icon_from_external_svg(path.clone()),
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
                    matched: None,
                },
                ThreadEntry {
                    workspace: workspace.downgrade(),
                    thread_id: row.thread_id,
                    agent: Agent::from(row.agent_id.clone()),
                    work_dirs: row.folder_paths().clone(),
                    title: row.title(),
                    icon: agents::thread_icon(&row.agent_id, workspace.read(cx).project(), cx),
                },
            )
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
            // A name the user gave the terminal wins over the CLI's own title.
            let title = terminal_view.custom_title().map_or_else(
                || agent_title(&terminal.read(cx).breadcrumb_text, agent.kind),
                ToString::to_string,
            );
            (
                title,
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
        matched: None,
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
    filter: &str,
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
                let mut terminal = terminal_snapshot(
                    &view,
                    root.as_deref(),
                    home,
                    foreground_command,
                    terminal_output.get(&view.entity_id()).copied(),
                    now,
                    cx,
                );
                terminal.matched = filter_match(filter, &terminal.title);
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
        for (mut thread, entry) in group_threads(group, &workspace, cx) {
            thread.matched = filter_match(filter, &thread.title);
            snapshot.threads.insert(thread.key.clone(), entry);
            threads.push(thread);
        }
        let matched = filter_match(filter, &name);
        snapshot.rail.projects.push(ProjectSnapshot {
            name,
            expanded: group.expanded,
            terminals,
            threads,
            matched,
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
    snapshot.rail.filtering = !filter.is_empty();
    let active_terminal = displayed
        .read(cx)
        .active_item(cx)
        .and_then(|item| item.downcast::<TerminalView>());
    snapshot.rail.focus = Focus {
        cursor: None,
        project: groups
            .iter()
            .position(|group| group.workspaces.contains(displayed)),
        terminal: active_terminal
            .as_ref()
            .map(|view| view.entity_id().as_u64()),
        terminal_focused: active_terminal
            .is_some_and(|view| view.focus_handle(cx).contains_focused(window, cx)),
        thread: panel_thread.filter(|_| {
            panel
                .as_ref()
                .is_some_and(|panel| panel.focus_handle(cx).contains_focused(window, cx))
        }),
    };
    snapshot
}

/// Where the rail's filter matches `text`, by the matcher Zed's Threads Sidebar uses; `None`
/// without a filter.
fn filter_match(filter: &str, text: &str) -> Option<Vec<usize>> {
    (!filter.is_empty())
        .then(|| fuzzy_match_positions(filter, text))
        .flatten()
}

/// A row's name or title, with the characters the filter matched highlighted.
fn row_label(text: String, highlight: Vec<usize>) -> AnyElement {
    if highlight.is_empty() {
        Label::new(text).size(LabelSize::Small).into_any_element()
    } else {
        HighlightedLabel::new(text, highlight)
            .size(LabelSize::Small)
            .into_any_element()
    }
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
        // The resize handle passes the raw pointer position, and `None` to reset.
        self.width_set_by_user = width.is_some();
        self.width = width.unwrap_or(DEFAULT_WIDTH).clamp(MIN_WIDTH, MAX_WIDTH);
        // One width for both layouts: the Zed sidebar the rail keeps takes the rail's clamped
        // width too, or the reset.
        if let Some((sidebar, _)) = &self.zed_sidebar {
            let width = width.map(|_| self.width);
            sidebar.update(cx, |sidebar, cx| sidebar.set_width(width, cx));
        }
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

    // Runs deferred, outside the `MultiWorkspace`'s update, so the overlay can be set here.
    fn toggle_thread_switcher(
        &mut self,
        select_last: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(open) = &self.switcher {
            open.view
                .update(cx, |switcher, cx| switcher.step(!select_last, cx));
            return;
        }
        let entries = self.switcher_entries();
        if entries.len() < 2 {
            return;
        }
        let return_focus = window.focused(cx);
        let view = cx.new(|cx| RailSwitcher::new(entries, select_last, window, cx));
        let events = cx.subscribe_in(
            &view,
            window,
            |rail, _, event: &SwitcherEvent, window, cx| rail.switcher_ended(event, window, cx),
        );
        let overlay = AnyView::from(view.clone());
        self.multi_workspace
            .update(cx, |multi_workspace, cx| {
                multi_workspace.set_sidebar_overlay(Some(overlay), cx);
            })
            .log_err();
        // The overlay neither focuses nor dismisses what it shows, and the release of `ctrl`
        // reaches only the focused view.
        window.focus(&view.focus_handle(cx), cx);
        self.switcher = Some(OpenSwitcher {
            view,
            return_focus,
            _events: events,
        });
    }

    // The window's saved sidebar state stays Zed's sidebar's while the rail stands in for it,
    // with the rail's own fields added. This runs inside the `MultiWorkspace`'s update, so it
    // reads the rail's fields and never the `MultiWorkspace`.
    fn serialized_state(&self, cx: &App) -> Option<String> {
        let zed_state = self
            .zed_sidebar
            .as_ref()
            .and_then(|(sidebar, _)| sidebar.read(cx).serialized_state(cx))
            .or_else(|| self.zed_sidebar_state.clone());
        Some(write_rail_state(zed_state.as_deref(), self.rail_state()))
    }

    fn restore_serialized_state(
        &mut self,
        state: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.zed_sidebar_state = Some(state.to_string());
        let saved = read_rail_state(state);
        if let Some(width) = saved.width {
            self.width = px(width).clamp(MIN_WIDTH, MAX_WIDTH);
            self.width_set_by_user = true;
        }
        if saved.closed {
            self.closed = true;
            // The restore runs inside the `MultiWorkspace`'s update, and closing reads the rail,
            // so the close waits until neither is being updated: a `defer_in` on the rail would
            // still run inside the rail's own update.
            let multi_workspace = self.multi_workspace.clone();
            window.defer(cx, move |window, cx| {
                let close = |multi_workspace: &mut MultiWorkspace,
                             cx: &mut Context<MultiWorkspace>| {
                    multi_workspace.close_sidebar(window, cx);
                };
                multi_workspace.update(cx, close).log_err();
            });
        }
    }
}

impl Render for Rail {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let search_path = agents::launcher(cx).search_path;
        let rows: Vec<AnyElement> = marley_rail::rail_rows(&self.snapshot.rail)
            .into_iter()
            .filter_map(|row| match row {
                Row::Project(row) => self.snapshot.groups.get(row.index).map(|group| {
                    let last = row.index + 1 == self.snapshot.groups.len();
                    let filtering = self.snapshot.rail.filtering;
                    Self::render_project_row(row, group, last, filtering, search_path.clone(), cx)
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
            // Zed binds left and right for lists only in the `menu` context.
            .key_context("MarleyRail menu")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_first))
            .on_action(cx.listener(Self::select_last))
            .on_action(cx.listener(Self::select_parent))
            .on_action(cx.listener(Self::select_child))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::focus_filter))
            .on_action(cx.listener(Self::cancel))
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(self.render_header(window, cx))
            .child(self.render_filter(cx))
            .child(
                v_flex()
                    .id("marley-rail-rows")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_1()
                    .gap_px()
                    .when(rows.is_empty() && self.snapshot.rail.filtering, |list| {
                        list.child(
                            div()
                                .debug_selector(|| "marley-rail-no-matches".into())
                                .px_2()
                                .py_1()
                                .child(
                                    Label::new("No matches")
                                        .size(LabelSize::Small)
                                        .color(Color::Muted),
                                ),
                        )
                    })
                    .children(rows),
            )
    }
}

#[cfg(test)]
#[path = "rail_tests.rs"]
mod tests;
