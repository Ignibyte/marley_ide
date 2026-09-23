//! The rail: the Marley layout's sidebar. Each project group, and under it the terminals in that
//! group's center panes. It implements Zed's `workspace::Sidebar`, so the `MultiWorkspace` keeps
//! the resize handle, open state, persistence and the toggle actions; the rows and the one selected
//! row come from `marley_rail`.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use anyhow::Context as _;
use gpui::{
    Anchor, AnyElement, App, Context, Entity, EntityId, EventEmitter, FocusHandle, Focusable,
    Pixels, Render, Subscription, Task, WeakEntity, Window, px,
};
use marley_rail::{
    Focus, ProjectRow, ProjectSnapshot, RailSnapshot, Row, TerminalRow, TerminalSnapshot,
};
use project::{Project, ProjectGroupKey};
use recent_projects::sidebar_recent_projects::SidebarRecentProjects;
use terminal::Terminal;
use terminal_view::{TerminalView, terminal_panel::TerminalPanel};
use ui::{
    ContextMenu, Disclosure, Icon, IconButton, IconName, IconSize, Indicator, Label, LabelSize,
    ListItem, PopoverMenu, PopoverMenuHandle, Tooltip, prelude::*,
    utils::platform_title_bar_height,
};
use util::ResultExt as _;
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

/// Zed's own sidebar and whether it was open, kept by the rail that replaced it.
pub type KeptSidebar = (Entity<sidebar::Sidebar>, bool);

/// The Marley layout's sidebar: each project group, with the terminals in its center panes under
/// it.
pub struct Rail {
    multi_workspace: WeakEntity<MultiWorkspace>,
    focus_handle: FocusHandle,
    width: Pixels,
    terminal_factory: TerminalFactory,
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

/// The window, read once: the pure snapshot, plus the entities the handlers act on.
#[derive(Default)]
struct Snapshot {
    rail: RailSnapshot,
    groups: Vec<GroupEntry>,
    terminals: HashMap<u64, TerminalEntry>,
}

impl Rail {
    /// A rail for `multi_workspace`, keeping Zed's sidebar when it replaces one.
    pub fn new(
        multi_workspace: &Entity<MultiWorkspace>,
        zed_sidebar: Option<KeptSidebar>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscriptions = [
            cx.subscribe_in(
                multi_workspace,
                window,
                |rail, _, _: &MultiWorkspaceEvent, _, cx| rail.refresh(cx),
            ),
            // Re-keying a project group notifies without an event.
            cx.observe(multi_workspace, |rail, _, cx| rail.refresh(cx)),
        ];
        // The `MultiWorkspace` may be mid-update while its sidebar is built, so the first read
        // waits for the end of this effect cycle.
        cx.defer_in(window, |rail, _, cx| rail.refresh(cx));
        Self {
            multi_workspace: multi_workspace.downgrade(),
            focus_handle: cx.focus_handle(),
            width: DEFAULT_WIDTH,
            terminal_factory: Project::create_terminal_shell,
            snapshot: Snapshot::default(),
            zed_sidebar,
            zed_sidebar_state: None,
            add_project_menu: PopoverMenuHandle::default(),
            workspace_subscriptions: HashMap::default(),
            terminal_subscriptions: HashMap::default(),
            _multi_workspace_subscriptions: subscriptions,
        }
    }

    /// Hands Zed's sidebar back for the switch to the Zed layout: the one the rail kept, and the
    /// state to restore into a fresh one when it kept none.
    pub(crate) const fn take_zed_sidebar(&mut self) -> (Option<KeptSidebar>, Option<String>) {
        (self.zed_sidebar.take(), self.zed_sidebar_state.take())
    }

    /// Follows every workspace and every listed terminal, rereads the window, and redraws only
    /// when what the rail shows has changed.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        self.sync_subscriptions(cx);
        let snapshot = self
            .multi_workspace
            .upgrade()
            .map(|multi_workspace| build_snapshot(&multi_workspace, cx))
            .unwrap_or_default();
        if snapshot.rail != self.snapshot.rail {
            cx.notify();
        }
        self.snapshot = snapshot;
    }

    fn sync_subscriptions(&mut self, cx: &mut Context<Self>) {
        let (workspaces, views) = self
            .multi_workspace
            .upgrade()
            .map(|multi_workspace| {
                let workspaces: Vec<Entity<Workspace>> =
                    multi_workspace.read(cx).workspaces().cloned().collect();
                let views: Vec<Entity<TerminalView>> = workspaces
                    .iter()
                    .flat_map(|workspace| workspace.read(cx).items_of_type::<TerminalView>(cx))
                    .collect();
                (workspaces, views)
            })
            .unwrap_or_default();
        // Rebuilt from what exists now: a restore replaces panes without reporting removals, and
        // whatever is left in the old maps is dropped, ending those subscriptions.
        let mut workspace_subscriptions = HashMap::default();
        for workspace in &workspaces {
            let subscription = self
                .workspace_subscriptions
                .remove(&workspace.entity_id())
                .unwrap_or_else(|| {
                    cx.subscribe(workspace, |rail, _, _: &workspace::Event, cx| {
                        rail.refresh(cx);
                    })
                });
            workspace_subscriptions.insert(workspace.entity_id(), subscription);
        }
        self.workspace_subscriptions = workspace_subscriptions;
        let mut terminal_subscriptions = HashMap::default();
        for view in &views {
            let subscriptions = self
                .terminal_subscriptions
                .remove(&view.entity_id())
                .unwrap_or_else(|| {
                    [
                        cx.subscribe(view, |rail, _, _: &terminal::Event, cx| rail.refresh(cx)),
                        cx.subscribe(view, |rail, _, _: &ItemEvent, cx| rail.refresh(cx)),
                    ]
                });
            terminal_subscriptions.insert(view.entity_id(), subscriptions);
        }
        self.terminal_subscriptions = terminal_subscriptions;
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

    fn toggle_expanded(&mut self, key: &ProjectGroupKey, cx: &mut Context<Self>) {
        self.multi_workspace
            .update(cx, |multi_workspace, cx| {
                if let Some(group) = multi_workspace.group_state_by_key_mut(key) {
                    group.expanded = !group.expanded;
                }
                multi_workspace.serialize(cx);
            })
            .log_err();
        // Collapsing emits no event the rail hears.
        self.refresh(cx);
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
                                    .on_click(cx.listener(move |rail, _, _, cx| {
                                        rail.toggle_expanded(&key, cx);
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
                            .child(Self::render_project_menu(index, id, group, cx)),
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
                        Some(ContextMenu::build(window, cx, move |menu, _, _| {
                            menu.entry("New Terminal", None, move |window, cx| {
                                rail.update(cx, |rail, cx| {
                                    rail.new_terminal(&workspace, window, cx)
                                })
                                .flatten()
                                .log_err();
                            })
                        }))
                    })
                    .anchor(Anchor::TopRight),
            )
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
                        Icon::new(IconName::Terminal)
                            .size(IconSize::Small)
                            .color(Color::Muted),
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

/// The window, read once. Only groups with an open workspace are listed; a group Zed keeps after
/// its last workspace closed has nothing for the rail to switch to.
fn build_snapshot(multi_workspace: &Entity<MultiWorkspace>, cx: &App) -> Snapshot {
    let multi_workspace = multi_workspace.read(cx);
    let groups: Vec<ProjectGroup> = multi_workspace
        .project_groups(cx)
        .into_iter()
        .filter(|group| !group.workspaces.is_empty())
        .collect();
    let names = group_names(&groups);
    let displayed = multi_workspace.workspace();
    let home = util::paths::home_dir().as_path();
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
                let terminal_view = view.read(cx);
                let working_directory = terminal_view.terminal().read(cx).working_directory();
                terminals.push(TerminalSnapshot {
                    id,
                    title: terminal_view.tab_content_text(0, cx).to_string(),
                    subtitle: marley_rail::working_directory_label(
                        working_directory.as_deref(),
                        root.as_deref(),
                        Some(home),
                    ),
                    bell: terminal_view.has_bell(),
                });
                snapshot.terminals.insert(
                    id,
                    TerminalEntry {
                        workspace: member.downgrade(),
                        view: view.downgrade(),
                    },
                );
            }
        }
        snapshot.rail.projects.push(ProjectSnapshot {
            name,
            expanded: group.expanded,
            terminals,
        });
        snapshot.groups.push(GroupEntry {
            key: group.key.clone(),
            workspace: workspace.downgrade(),
        });
    }
    snapshot.rail.focus = Focus {
        project: groups
            .iter()
            .position(|group| group.workspaces.contains(displayed)),
        terminal: displayed
            .read(cx)
            .active_item(cx)
            .and_then(|item| item.downcast::<TerminalView>())
            .map(|view| view.entity_id().as_u64()),
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

    // The rail lists no threads yet, so Zed must not treat thread notifications as seen.
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
                Row::Project(row) => self
                    .snapshot
                    .groups
                    .get(row.index)
                    .map(|group| Self::render_project_row(row, group, cx).into_any_element()),
                Row::Terminal(row) => self.snapshot.terminals.get(&row.id).map(|terminal| {
                    Self::render_terminal_row(row, terminal, cx).into_any_element()
                }),
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
