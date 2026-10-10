//! Home's own page (#701): the first tab of the window's Home group (#676, #700), a home page for
//! Zed and Marley drawn in place of Zed's Welcome page.
//!
//! Its cards start what belongs to no project (a terminal or an agent CLI in the home folder, a
//! folder or a clone), list Zed's recent projects and the window's agents at work from the rail, and
//! open Zed's and Marley's settings and guide.

use std::path::PathBuf;
use std::sync::Arc;

use fs::Fs;
use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable, Render,
    SharedString, Subscription, WeakEntity, Window,
};
use settings::SettingsStore;
use terminal_view::terminal_panel::TerminalPanel;
use ui::{ButtonStyle, prelude::*};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::{
    MultiWorkspace, OpenMode, SerializedWorkspaceLocation, Workspace, WorkspaceDb,
    notifications::DetachAndPromptErr as _,
};

use crate::OpenGuide;
use crate::agent_activity::AgentActivity;
use crate::agents;
use crate::rail::Rail;
use crate::rusty::home_tab::{card, muted, row};

/// How many recent projects the page lists.
const RECENT_SHOWN: usize = 8;

/// How many rows of agent activity the page lists (#703).
const ACTIVITY_SHOWN: usize = 5;

/// Adds Home's page first in `workspace`'s active pane when the workspace has none, taking the
/// active tab only from an empty pane.
pub(crate) fn ensure(workspace: &Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    if workspace.items_of_type::<MarleyHome>(cx).next().is_some() {
        return;
    }
    let fs = Arc::clone(&workspace.app_state().fs);
    let weak_workspace = cx.weak_entity();
    let home = cx.new(|cx| MarleyHome::new(weak_workspace, fs, window, cx));
    let pane = workspace.active_pane().clone();
    pane.update(cx, |pane, cx| {
        let empty = pane.items_len() == 0;
        let active = pane.active_item();
        pane.add_item_inner(Box::new(home), false, false, empty, Some(0), window, cx);
        // Inserted before the active tab without activating, the page takes the active index, so
        // a tab just opened into Home would hide behind it (#703).
        if let Some(active) = active
            && let Some(index) = pane.index_for_item(active.as_ref())
        {
            pane.activate_item(index, false, false, window, cx);
        }
    });
}

/// Home's page in the Home group once its last tab closed, so it never shows Zed's Welcome page.
pub(crate) fn fill_emptied(
    workspace: &Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if crate::groups::is_home_workspace(cx.entity_id(), cx) && workspace.items(cx).next().is_none()
    {
        ensure(workspace, window, cx);
    }
}

/// A recent project, as the page lists it.
struct Recent {
    name: String,
    folder: String,
    paths: Vec<PathBuf>,
}

/// Home's page.
pub(crate) struct MarleyHome {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    /// Zed's recent local projects; `None` while they are read.
    recent: Option<Vec<Recent>>,
    /// The rail the page lists the agents at work from, observed once found.
    rail: Option<(Entity<Rail>, Subscription)>,
    /// The agent activity and the kill switch, for the AGENT ACTIVITY card (#703).
    _activity: [Subscription; 2],
}

impl std::fmt::Debug for MarleyHome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarleyHome").finish_non_exhaustive()
    }
}

impl MarleyHome {
    fn new(
        workspace: WeakEntity<Workspace>,
        fs: Arc<dyn Fs>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let db = WorkspaceDb::global(cx);
        cx.spawn_in(window, async move |this, cx| {
            let recent = db
                .recent_project_workspaces(fs.as_ref())
                .await
                .log_err()
                .unwrap_or_default()
                .into_iter()
                .filter(|recent| matches!(recent.location, SerializedWorkspaceLocation::Local))
                .take(RECENT_SHOWN)
                .map(|recent| {
                    let paths = recent.paths.paths().to_vec();
                    let first = paths.first();
                    Recent {
                        name: first
                            .and_then(|path| path.file_name())
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_default(),
                        folder: first
                            .and_then(|path| path.parent())
                            .map(|parent| parent.to_string_lossy().into_owned())
                            .unwrap_or_default(),
                        paths,
                    }
                })
                .collect();
            this.update(cx, |this, cx| {
                this.recent = Some(recent);
                cx.notify();
            })
            .log_err();
        })
        .detach();
        Self {
            workspace,
            focus_handle: cx.focus_handle(),
            recent: None,
            rail: None,
            _activity: [
                cx.observe_global_in::<AgentActivity>(window, |_, _, cx| cx.notify()),
                cx.observe_global_in::<SettingsStore>(window, |_, _, cx| cx.notify()),
            ],
        }
    }

    /// The AGENT ACTIVITY card's body (#703): the kill switch, the newest rows, and the tab.
    fn render_activity(&self, cx: &App) -> impl IntoElement {
        let workspace = self.workspace.clone();
        v_flex()
            .gap_2()
            .child(crate::agent_activity::render_state(
                "marley-home-agent-control",
                cx,
            ))
            .child(crate::agent_activity::render_rows(ACTIVITY_SHOWN, cx))
            .child(
                h_flex().child(
                    Button::new("marley-home-open-activity", "Open Agent Activity")
                        .start_icon(Icon::new(IconName::ListTodo).size(IconSize::Small))
                        .style(ButtonStyle::Subtle)
                        .on_click(move |_, window, cx| {
                            crate::agent_activity::open_later(workspace.clone(), window, cx);
                        }),
                ),
            )
    }

    /// The window's rail, observed the first time it is found, so the agents at work follow it.
    fn rail(&mut self, window: &Window, cx: &mut Context<Self>) -> Option<Entity<Rail>> {
        if let Some((rail, _)) = &self.rail {
            return Some(rail.clone());
        }
        let multi_workspace = window.root::<MultiWorkspace>().flatten()?;
        let rail = multi_workspace
            .read(cx)
            .sidebar()?
            .to_any()
            .downcast::<Rail>()
            .ok()?;
        let subscription = cx.observe(&rail, |_, _, cx| cx.notify());
        self.rail = Some((rail.clone(), subscription));
        Some(rail)
    }

    // The page's handlers are plain closures over weak handles, never `cx.listener`: a workspace
    // updated from them reads its items, this page among them, which a listener would hold leased.

    fn render_start(&self) -> impl IntoElement {
        let entries: [(&'static str, &'static str, IconName); 4] = [
            (
                "marley-home-new-terminal",
                "New Terminal",
                IconName::Terminal,
            ),
            (
                "marley-home-open-folder",
                "Open Folder…",
                IconName::FolderOpen,
            ),
            (
                "marley-home-clone",
                "Clone Repository…",
                IconName::GitBranch,
            ),
            ("marley-home-palette", "Command Palette", IconName::Command),
        ];
        h_flex()
            .flex_wrap()
            .gap_2()
            .children(entries.into_iter().map(|(id, label, icon)| {
                let workspace = self.workspace.clone();
                let focus = self.focus_handle.clone();
                Button::new(id, label)
                    .start_icon(Icon::new(icon).size(IconSize::Small))
                    .style(ButtonStyle::Filled)
                    .on_click(move |_, window, cx| match id {
                        "marley-home-new-terminal" => new_terminal(&workspace, window, cx),
                        "marley-home-open-folder" => {
                            focus.dispatch_action(&workspace::Open::DEFAULT, window, cx);
                        }
                        "marley-home-clone" => focus.dispatch_action(&git::Clone, window, cx),
                        _ => {
                            focus.dispatch_action(
                                &zed_actions::command_palette::Toggle,
                                window,
                                cx,
                            );
                        }
                    })
            }))
    }

    /// New Agent…, the picker that asks which agent and where (#735), then a button per agent CLI
    /// on the search path, each started in a terminal in Home's home folder.
    fn render_agents(&self, cx: &App) -> AnyElement {
        let kinds = agents::installed_clis(agents::launcher(cx).search_path.as_deref());
        let picker_workspace = self.workspace.clone();
        let picker = Button::new("marley-home-new-agent", "New Agent…")
            .start_icon(Icon::new(IconName::Plus).size(IconSize::Small))
            .style(ButtonStyle::Filled)
            .on_click(move |_, window, cx| {
                picker_workspace
                    .update(cx, |workspace, cx| {
                        agents::show_picker(workspace, None, window, cx);
                    })
                    .log_err();
            });
        h_flex()
            .flex_wrap()
            .gap_2()
            .child(picker)
            .children(kinds.into_iter().enumerate().map(|(index, kind)| {
                let workspace = self.workspace.clone();
                Button::new(("marley-home-agent", index), kind.display_name())
                    .start_icon(Icon::new(agents::cli_icon(kind)).size(IconSize::Small))
                    .style(ButtonStyle::Filled)
                    .on_click(move |_, window, cx| {
                        workspace
                            .update(cx, |workspace, cx| {
                                agents::start_cli(workspace, kind, window, cx);
                            })
                            .log_err();
                    })
            }))
            .into_any_element()
    }

    fn render_recent(&self, cx: &App) -> AnyElement {
        let Some(recent) = &self.recent else {
            return muted("Reading your recent projects…");
        };
        if recent.is_empty() {
            return muted("No project opened yet.");
        }
        v_flex()
            .children(recent.iter().enumerate().map(|(index, recent)| {
                let paths = recent.paths.clone();
                let workspace = self.workspace.clone();
                row(("marley-home-recent", index), cx)
                    .child(
                        Icon::new(IconName::Folder)
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    // The name keeps its width; a long folder is what gets cut.
                    .child(
                        div()
                            .flex_none()
                            .child(Label::new(recent.name.clone()).size(LabelSize::Small)),
                    )
                    .child(
                        div().min_w_0().flex_1().child(
                            Label::new(recent.folder.clone())
                                .size(LabelSize::XSmall)
                                .color(Color::Muted)
                                .truncate(),
                        ),
                    )
                    .on_click(move |_, window, cx| {
                        let paths = paths.clone();
                        workspace
                            .update(cx, |workspace, cx| {
                                workspace
                                    .open_workspace_for_paths(OpenMode::Activate, paths, window, cx)
                                    .detach_and_log_err(cx);
                            })
                            .log_err();
                    })
            }))
            .into_any_element()
    }

    fn render_at_work(&mut self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(rail) = self.rail(window, cx) else {
            return muted("The Marley layout's rail lists the agents at work.");
        };
        let at_work = rail.read(cx).agents_at_work();
        if at_work.is_empty() {
            return muted("No agent at work.");
        }
        v_flex()
            .children(at_work.into_iter().enumerate().map(|(index, agent)| {
                let rail = rail.downgrade();
                let color = match agent.state {
                    "failed" => Color::Error,
                    "waiting" => Color::Warning,
                    _ => Color::Muted,
                };
                let opened = agent.row.clone();
                row(("marley-home-at-work", index), cx)
                    .child(
                        Icon::new(IconName::Sparkle)
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .child(Label::new(agent.title).size(LabelSize::Small).truncate()),
                    )
                    .child(
                        Label::new(agent.project)
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .truncate(),
                    )
                    .child(
                        Label::new(agent.status)
                            .size(LabelSize::XSmall)
                            .color(color),
                    )
                    .on_click(move |_, window, cx| {
                        rail.update(cx, |rail, cx| {
                            rail.open_selection(opened.clone(), window, cx);
                        })
                        .log_err();
                    })
            }))
            .into_any_element()
    }

    fn render_configure(&self) -> impl IntoElement {
        let entries: [(&'static str, &'static str, IconName); 5] = [
            ("marley-home-settings", "Settings", IconName::Settings),
            (
                "marley-home-marley-settings",
                "Marley Settings",
                IconName::Settings,
            ),
            ("marley-home-keymap", "Keymap", IconName::Keyboard),
            ("marley-home-extensions", "Extensions", IconName::Blocks),
            ("marley-home-guide", "Marley Guide", IconName::Book),
        ];
        h_flex()
            .flex_wrap()
            .gap_2()
            .children(entries.into_iter().map(|(id, label, icon)| {
                let focus = self.focus_handle.clone();
                Button::new(id, label)
                    .start_icon(Icon::new(icon).size(IconSize::Small))
                    .style(ButtonStyle::Filled)
                    .on_click(move |_, window, cx| match id {
                        "marley-home-settings" => {
                            focus.dispatch_action(&zed_actions::OpenSettings, window, cx);
                        }
                        "marley-home-marley-settings" => focus.dispatch_action(
                            &zed_actions::OpenSettingsAt {
                                path: "marley.layout".to_string(),
                                target: None,
                            },
                            window,
                            cx,
                        ),
                        "marley-home-keymap" => {
                            focus.dispatch_action(&zed_actions::OpenKeymap, window, cx);
                        }
                        "marley-home-extensions" => {
                            focus.dispatch_action(&zed_actions::Extensions::default(), window, cx);
                        }
                        _ => focus.dispatch_action(&OpenGuide, window, cx),
                    })
            }))
    }
}

/// Opens a terminal in `workspace`'s center, in its default folder (the home folder for Home), as
/// the rail's New Terminal does.
fn new_terminal(workspace: &WeakEntity<Workspace>, window: &mut Window, cx: &mut App) {
    let factory = agents::launcher(cx).terminal_factory;
    workspace
        .update(cx, |workspace, cx| {
            let directory = terminal_view::default_working_directory(workspace, cx);
            TerminalPanel::add_center_terminal(workspace, window, cx, move |project, cx| {
                factory(project, directory, collections::HashMap::default(), cx)
            })
            .detach_and_prompt_err(
                "Could not open a terminal",
                window,
                cx,
                |_, _, _| None,
            );
        })
        .log_err();
}

impl Render for MarleyHome {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let background = colors.editor_background;
        let at_work = self.render_at_work(window, cx);
        v_flex()
            .id("marley-home")
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_4()
            .bg(background)
            .child(card("START", "marley-home-start", self.render_start(), cx))
            .child(card(
                "NEW AGENT",
                "marley-home-agents",
                self.render_agents(cx),
                cx,
            ))
            .child(
                h_flex()
                    .w_full()
                    .gap_4()
                    .items_start()
                    .child(
                        card(
                            "RECENT PROJECTS",
                            "marley-home-recent",
                            self.render_recent(cx),
                            cx,
                        )
                        .flex_1()
                        .min_w_0(),
                    )
                    .child(
                        card("AGENTS AT WORK", "marley-home-at-work", at_work, cx)
                            .flex_1()
                            .min_w_0(),
                    ),
            )
            .child(card(
                "CONFIGURE",
                "marley-home-configure",
                self.render_configure(),
                cx,
            ))
            .child(card(
                "AGENT ACTIVITY",
                "marley-home-activity",
                self.render_activity(cx),
                cx,
            ))
    }
}

impl Focusable for MarleyHome {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for MarleyHome {}

impl Item for MarleyHome {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Home")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::ListTree))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static(
            "Home: a terminal, an agent, a project",
        ))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }
}
