//! Home's own page (#701): the first tab of the window's Home group (#676, #700), a home page for
//! Zed and Marley drawn in place of Zed's Welcome page.
//!
//! Its cards start what belongs to no project (a terminal or an agent CLI in the home folder, a
//! folder or a clone), list Zed's recent projects and the window's agents at work from the rail, and
//! open Zed's and Marley's settings and guide. While `marley.rusty.enabled` is on it holds Rusty's
//! cards too (#782), which Rusty's own page held until then.

use std::path::PathBuf;
use std::sync::Arc;

use fs::Fs;
use gpui::{
    App, AppContext as _, Context, Div, ElementId, Entity, EventEmitter, FocusHandle, Focusable,
    FontWeight, Render, SharedString, Stateful, Subscription, Task, WeakEntity, Window,
};
use marley_agent::AgentKind;
use project::Project;
use settings::SettingsStore;
use terminal_view::terminal_panel::TerminalPanel;
use ui::{ButtonStyle, ContextMenu, PopoverMenu, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::{
    MultiWorkspace, MultiWorkspaceEvent, OpenMode, SerializedWorkspaceLocation, Workspace,
    WorkspaceDb, WorkspaceId, notifications::DetachAndPromptErr as _,
};

use crate::OpenGuide;
use crate::agent_activity::AgentActivity;
use crate::agents;
use crate::rail::Rail;
use crate::rusty::home_tab::RustyCards;

/// How many recent projects the page lists.
const RECENT_SHOWN: usize = 8;

/// How many rows of agent activity the page lists (#703).
const ACTIVITY_SHOWN: usize = 5;

/// Keeps Default from showing no tab (#701): Home's page goes in when Default is shown with none,
/// and when its last tab closes. [`crate::init`] calls it once.
pub(crate) fn init(cx: &App) {
    cx.observe_new(|_: &mut Workspace, window, cx: &mut Context<Workspace>| {
        let Some(window) = window else {
            return;
        };
        // Its last tab closed.
        cx.subscribe_in(
            &cx.entity(),
            window,
            |workspace, _, event: &workspace::Event, window, cx| {
                if let workspace::Event::ItemRemoved { .. } = event {
                    fill_emptied(workspace, window, cx);
                }
            },
        )
        .detach();
    })
    .detach();
    // Shown with no tab: from the rail or a switch, or after a restart, once the rail adopts the
    // restored group.
    cx.observe_new(
        |_: &mut MultiWorkspace, window, cx: &mut Context<MultiWorkspace>| {
            let Some(window) = window else {
                return;
            };
            cx.subscribe_in(
                &cx.entity(),
                window,
                |multi_workspace, _, event: &MultiWorkspaceEvent, window, cx| {
                    if let MultiWorkspaceEvent::ActiveWorkspaceChanged { .. } = event {
                        fill_shown(multi_workspace, window, cx);
                    }
                },
            )
            .detach();
            cx.observe_global_in::<crate::groups::Groups>(window, |multi_workspace, window, cx| {
                fill_shown(multi_workspace, window, cx);
            })
            .detach();
        },
    )
    .detach();
}

/// Home's page in the workspace the window shows when it is the Default group (#701).
fn fill_shown(multi_workspace: &MultiWorkspace, window: &mut Window, cx: &mut App) {
    let shown = multi_workspace.workspace().clone();
    if crate::groups::is_home_workspace(shown.entity_id(), cx) {
        shown.update(cx, |workspace, cx| {
            ensure(workspace, window, cx);
        });
    }
}

/// Adds Home's page first in `workspace`'s active pane when the workspace has none, taking the
/// active tab only from an empty pane.
pub(crate) fn ensure(workspace: &Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    if workspace.items_of_type::<MarleyHome>(cx).next().is_some() {
        return;
    }
    let home = new_page(workspace, window, cx);
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

/// Brings `workspace`'s Home page forward, or opens one in its center (#739): the status bar's Home
/// button opens it in whatever group the window shows.
pub(crate) fn open_here(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let open_page = workspace.items_of_type::<MarleyHome>(cx).next();
    if let Some(home) = open_page {
        workspace.activate_item(&home, true, true, window, cx);
        return;
    }
    let home = new_page(workspace, window, cx);
    workspace.add_item_to_center(Box::new(home), window, cx);
}

/// A new Home page of `workspace`, not yet in a pane.
pub(crate) fn new_page(
    workspace: &Workspace,
    window: &Window,
    cx: &mut Context<Workspace>,
) -> Entity<MarleyHome> {
    let fs = Arc::clone(&workspace.app_state().fs);
    let weak_workspace = cx.weak_entity();
    cx.new(|cx| MarleyHome::new(weak_workspace, fs, window, cx))
}

/// Home's page in the Home group once its last tab closed, so it never shows Zed's Welcome page.
fn fill_emptied(workspace: &Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
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

/// A project the prompt's chip offers, by its name and folders (#867).
type ProjectChoice = (SharedString, Vec<PathBuf>);

/// Home's page.
pub(crate) struct MarleyHome {
    workspace: WeakEntity<Workspace>,
    /// The file system the recent projects are read through, kept for a split's new page (#785):
    /// a split runs while the workspace is being updated, so it cannot be read there.
    fs: Arc<dyn Fs>,
    focus_handle: FocusHandle,
    /// Zed's recent local projects; `None` while they are read.
    recent: Option<Vec<Recent>>,
    /// The prompt box: what the picked agent starts with (#835), made at the first draw, where a
    /// window can be written.
    prompt: Option<Entity<editor::Editor>>,
    /// The agent the prompt starts; the first installed CLI until one is picked.
    picked: Option<AgentKind>,
    /// The project the prompt starts in, by its name and folders; none for Home's own (#867, D1).
    target: Option<ProjectChoice>,
    /// The rail the page lists the agents at work from, held weakly and observed, so a rail a
    /// layout swap replaced goes (#713).
    rail: Option<(WeakEntity<Rail>, Subscription)>,
    /// The agent activity and the kill switch, for the AGENT ACTIVITY card (#703), and the project
    /// marks the recent projects show (#831).
    _activity: [Subscription; 4],
    /// Rusty's cards, held while `marley.rusty.enabled` is on (#782), and observed, since Home
    /// draws them in its own grid (#903).
    rusty: Option<(Entity<RustyCards>, Subscription)>,
    /// Rusty's switch, which makes and drops the cards.
    _rusty_switch: Subscription,
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
        let recent_fs = Arc::clone(&fs);
        cx.spawn_in(window, async move |this, cx| {
            let recent: Vec<Recent> = db
                .recent_project_workspaces(recent_fs.as_ref())
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
                // Each recent project's mark (#831), most of them not open.
                for recent in &recent {
                    if let Some(first) = recent.paths.first() {
                        crate::project_icons::want(first, cx);
                    }
                }
                this.recent = Some(recent);
                cx.notify();
            })
            .log_err();
        })
        .detach();
        let mut this = Self {
            workspace,
            fs,
            focus_handle: cx.focus_handle(),
            recent: None,
            prompt: None,
            picked: None,
            target: None,
            rail: None,
            _activity: [
                cx.observe_global_in::<AgentActivity>(window, |_, _, cx| cx.notify()),
                cx.observe_global_in::<SettingsStore>(window, |_, _, cx| cx.notify()),
                cx.observe_global_in::<crate::project_icons::ProjectMarks>(window, |_, _, cx| {
                    cx.notify();
                }),
                cx.observe_global_in::<crate::forge_watch::ForgeWatch>(window, |_, _, cx| {
                    cx.notify();
                }),
            ],
            rusty: None,
            _rusty_switch: cx.observe_global_in::<crate::rusty::Rusty>(
                window,
                |this, window, cx| {
                    this.follow_rusty(window, cx);
                },
            ),
        };
        this.follow_rusty(window, cx);
        this
    }

    /// Makes Rusty's cards when Rusty's switch is on and drops them when it is off, so Home holds
    /// nothing of Rusty while it is off.
    fn follow_rusty(&mut self, window: &Window, cx: &mut Context<Self>) {
        let on = crate::rusty::is_on(cx);
        if on == self.rusty.is_some() {
            return;
        }
        self.rusty = on.then(|| {
            let (workspace, focus_handle) = (self.workspace.clone(), self.focus_handle.clone());
            let cards = cx.new(|cx| RustyCards::new(workspace, focus_handle, window, cx));
            let observed = cx.observe(&cards, |_, _, cx| cx.notify());
            (cards, observed)
        });
        cx.notify();
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

    /// The window's rail, observed while it is the window's, so the agents at work follow it; a
    /// layout swap's new rail is found and observed in its place (#713).
    fn rail(&mut self, window: &Window, cx: &mut Context<Self>) -> Option<Entity<Rail>> {
        let multi_workspace = window.root::<MultiWorkspace>().flatten()?;
        let rail = multi_workspace
            .read(cx)
            .sidebar()?
            .to_any()
            .downcast::<Rail>()
            .ok()?;
        let followed = self
            .rail
            .as_ref()
            .is_some_and(|(followed, _)| followed.entity_id() == rail.entity_id());
        if !followed {
            let subscription = cx.observe(&rail, |_, _, cx| cx.notify());
            self.rail = Some((rail.downgrade(), subscription));
        }
        Some(rail)
    }

    // The page's handlers are plain closures over weak handles, never `cx.listener`: a workspace
    // updated from them reads its items, this page among them, which a listener would hold leased.

    /// The start actions under the prompt, as pills (#903): a terminal, a folder, a clone, the
    /// threads to reopen, and Today's note while Rusty is on.
    fn render_start(&self, cx: &App) -> impl IntoElement {
        let mut entries: Vec<(&'static str, &'static str)> = vec![
            ("marley-home-new-terminal", "New Terminal"),
            ("marley-home-open-folder", "Open Folder\u{2026}"),
            ("marley-home-clone", "Clone Repository\u{2026}"),
            ("marley-home-threads", "Reopen a Thread"),
        ];
        if self.rusty.is_some() {
            entries.push(("marley-home-today", "Today\u{2019}s Note"));
        }
        h_flex()
            .debug_selector(|| "marley-home-start".into())
            .flex_wrap()
            .justify_center()
            .gap_2()
            .children(entries.into_iter().map(|(id, label)| {
                let workspace = self.workspace.clone();
                let focus = self.focus_handle.clone();
                pill(id, false, cx)
                    .px_3p5()
                    .child(Label::new(label).size(LabelSize::Small))
                    .on_click(move |_, window, cx| match id {
                        "marley-home-new-terminal" => new_terminal(&workspace, window, cx),
                        "marley-home-open-folder" => {
                            focus.dispatch_action(&workspace::Open::DEFAULT, window, cx);
                        }
                        "marley-home-clone" => focus.dispatch_action(&git::Clone, window, cx),
                        "marley-home-threads" => {
                            workspace
                                .update(cx, |workspace, cx| {
                                    crate::threads_page::open(workspace, window, cx);
                                })
                                .log_err();
                        }
                        _ => {
                            RustyCards::open_screen(crate::rusty::brain::Screen::Today, window, cx);
                        }
                    })
            }))
    }

    /// The prompt (#835), as the Home board draws it (#903): a box two lines tall, then the
    /// project chip (#867), the agent CLIs as pills, and a round Start.
    fn render_prompt(&self, cx: &Context<Self>) -> AnyElement {
        let kinds = agents::installed_clis(agents::launcher(cx).search_path.as_deref());
        let picked = self.picked.or_else(|| kinds.first().copied());
        let colors = cx.theme().colors();
        let accent = colors.text_accent;
        let chips = kinds.into_iter().enumerate().map(|(index, kind)| {
            pill(("marley-home-agent", index), picked == Some(kind), cx)
                .child(glyph_tile(cli_glyph(kind), cx))
                .child(Label::new(kind.display_name()).size(LabelSize::Small))
                .on_click(cx.listener(move |home, _, _, cx| {
                    home.picked = Some(kind);
                    cx.notify();
                }))
        });
        let start = div()
            .id("marley-home-start-agent")
            .debug_selector(|| "marley-home-start-agent".into())
            .flex_none()
            .size(px(34.))
            .rounded(px(10.))
            .bg(accent)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .child(
                Icon::new(IconName::ArrowRight)
                    .size(IconSize::Small)
                    .color(Color::Custom(colors.editor_background)),
            )
            .tooltip(Tooltip::text("Start"))
            .on_click(cx.listener(|home, _, window, cx| home.start_prompt(window, cx)));
        v_flex()
            .debug_selector(|| "marley-home-prompt".into())
            .key_context("MarleyHomePrompt")
            .on_action(cx.listener(|home, _: &menu::Confirm, window, cx| {
                home.start_prompt(window, cx);
            }))
            // Enter starts the agent: the box is two lines tall only to show a longer task.
            .capture_action(
                cx.listener(|home, _: &editor::actions::Newline, window, cx| {
                    cx.stop_propagation();
                    home.start_prompt(window, cx);
                }),
            )
            .w_full()
            .gap_2p5()
            .pl(px(18.))
            .pr_3p5()
            .pt_3p5()
            .pb_2p5()
            .rounded(px(18.))
            .border_1()
            .border_color(colors.border)
            .bg(colors.editor_background)
            .children(self.prompt.clone())
            .child(
                h_flex()
                    .w_full()
                    .gap_1p5()
                    .flex_wrap()
                    .child(self.render_project_chip(cx))
                    .children(chips)
                    .child(div().flex_1())
                    .child(start),
            )
            .into_any_element()
    }

    /// The project chip (#867): where the prompt's agent starts, and a menu of Home, the window's
    /// open projects and the recent ones.
    fn render_project_chip(&self, cx: &Context<Self>) -> impl IntoElement {
        let name = self.target.as_ref().map_or_else(
            || {
                self.workspace.upgrade().map_or_else(
                    || "Home".to_string(),
                    |workspace| {
                        // The folder's name, as the board's chip reads (#903); the home folder is
                        // Home.
                        terminal_view::default_working_directory(workspace.read(cx), cx)
                            .filter(|folder| folder != paths::home_dir())
                            .and_then(|folder| {
                                folder
                                    .file_name()
                                    .map(|name| name.to_string_lossy().into_owned())
                            })
                            .unwrap_or_else(|| "Home".to_string())
                    },
                )
            },
            |(name, _)| name.to_string(),
        );
        let home = cx.entity().downgrade();
        let projects = self.project_choices(cx);
        PopoverMenu::new("marley-home-project")
            .trigger(
                Button::new("marley-home-project-chip", name)
                    .start_icon(Icon::new(IconName::Folder).size(IconSize::Small))
                    .end_icon(Icon::new(IconName::ChevronDown).size(IconSize::XSmall))
                    .style(ButtonStyle::Filled)
                    .label_size(LabelSize::Small),
            )
            .menu(move |window, cx| {
                let home = home.clone();
                let projects = projects.clone();
                Some(ContextMenu::build(window, cx, move |menu, _, _| {
                    let home_entry = home.clone();
                    let menu = menu.entry("Home", None, move |_, cx| {
                        home_entry
                            .update(cx, |page, cx| {
                                page.target = None;
                                cx.notify();
                            })
                            .log_err();
                    });
                    projects.into_iter().fold(menu, |menu, (section, entries)| {
                        let menu = if entries.is_empty() {
                            menu
                        } else {
                            menu.header(section)
                        };
                        entries.into_iter().fold(menu, |menu, (name, paths)| {
                            let home = home.clone();
                            let label = name.clone();
                            menu.entry(label, None, move |_, cx| {
                                let picked = (name.clone(), paths.clone());
                                home.update(cx, |page, cx| {
                                    page.target = Some(picked);
                                    cx.notify();
                                })
                                .log_err();
                            })
                        })
                    })
                }))
            })
    }

    /// The projects the chip offers (#867): the window's open ones but Home's own, then the recent
    /// ones not open, each by its name and folders.
    fn project_choices(&self, cx: &App) -> Vec<(&'static str, Vec<ProjectChoice>)> {
        let mut open: Vec<ProjectChoice> = Vec::new();
        // Home's own project is the menu's Home entry, so Recent leaves it out too.
        let mut own_paths: Vec<PathBuf> = Vec::new();
        if let Some(workspace) = self.workspace.upgrade() {
            let own = workspace.entity_id();
            own_paths = worktree_paths(workspace.read(cx).project().read(cx), cx);
            let others = workspace
                .read(cx)
                .multi_workspace()
                .and_then(WeakEntity::upgrade)
                .map(|multi| {
                    multi
                        .read(cx)
                        .workspaces()
                        .filter(|other| other.entity_id() != own)
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            for other in others {
                let paths = worktree_paths(other.read(cx).project().read(cx), cx);
                let Some(first) = paths.first() else {
                    continue;
                };
                let name = first
                    .file_name()
                    .map_or_else(|| first.to_string_lossy(), |name| name.to_string_lossy());
                open.push((SharedString::from(name.into_owned()), paths));
            }
        }
        let recent = self
            .recent
            .iter()
            .flatten()
            .filter(|recent| {
                recent.paths != own_paths && !open.iter().any(|(_, paths)| *paths == recent.paths)
            })
            .map(|recent| {
                (
                    SharedString::from(recent.name.clone()),
                    recent.paths.clone(),
                )
            })
            .collect();
        vec![("Open in this window", open), ("Recent", recent)]
    }

    /// New Agent…, the picker that asks which agent and where (#735), and Threads (#745), as links.
    /// Starts the picked agent with the prompt's task in Home's workspace, and empties the box.
    fn start_prompt(&self, window: &mut Window, cx: &mut Context<Self>) {
        let kinds = agents::installed_clis(agents::launcher(cx).search_path.as_deref());
        let Some(kind) = self.picked.or_else(|| kinds.first().copied()) else {
            return;
        };
        let task = self
            .prompt
            .as_ref()
            .map(|prompt| prompt.read(cx).text(cx).trim().to_string())
            .unwrap_or_default();
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        match self.target.clone() {
            None => workspace.update(cx, |workspace, cx| {
                start_in(workspace, kind, &task, window, cx);
            }),
            // The picked project, opened or brought forward first (#867). The open runs once
            // this update ends: it asks every item, this page too, whether it is dirty.
            Some((_, paths)) => {
                cx.spawn_in(window, async move |_, cx| {
                    let opening = workspace.update_in(cx, |workspace, window, cx| {
                        workspace.open_workspace_for_paths(OpenMode::Activate, paths, window, cx)
                    })?;
                    let opened = opening.await?;
                    opened.update_in(cx, |workspace, window, cx| {
                        start_in(workspace, kind, &task, window, cx);
                    })
                })
                .detach_and_log_err(cx);
            }
        }
        if let Some(prompt) = &self.prompt {
            prompt.update(cx, |editor, cx| editor.set_text("", window, cx));
        }
    }

    fn render_recent(&self, cx: &App) -> AnyElement {
        let Some(recent) = &self.recent else {
            return muted("Reading your recent projects…");
        };
        if recent.is_empty() {
            // The States board's card (#857).
            return crate::states::EmptyState::new(
                "..",
                "No recent projects",
                "Open a folder, clone one, or start an agent in your home folder.",
            )
            .action(
                "Open a Folder",
                crate::prompts::Role::Primary,
                |window, cx| {
                    window.dispatch_action(Box::new(workspace::Open::DEFAULT), cx);
                },
            )
            .render(0, cx);
        }
        v_flex()
            .children(recent.iter().enumerate().map(|(index, recent)| {
                let paths = recent.paths.clone();
                let workspace = self.workspace.clone();
                row(("marley-home-recent", index), cx)
                    .child(paths.first().map_or_else(
                        || {
                            Icon::new(IconName::Folder)
                                .size(IconSize::Small)
                                .color(Color::Muted)
                                .into_any_element()
                        },
                        |first| {
                            crate::project_icons::project_mark(first, &recent.name, px(16.), cx)
                        },
                    ))
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
                let status = cx.theme().status();
                let dot = match agent.state {
                    "failed" => status.error,
                    "waiting" => status.warning,
                    _ => status.success,
                };
                // A tile with the agent's state at its corner, over its status, as the board
                // draws a row (#903).
                row(("marley-home-at-work", index), cx)
                    .gap_2p5()
                    .child(state_tile(dot, cx))
                    .child(
                        v_flex()
                            .min_w_0()
                            .flex_1()
                            .child(Label::new(agent.title).size(LabelSize::Small).truncate())
                            .child(
                                Label::new(format!("{} \u{b7} {}", agent.status, agent.project))
                                    .size(LabelSize::XSmall)
                                    .color(color)
                                    .truncate(),
                            ),
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

    /// Home's cards in the board's order (#903): agents at work, Rusty's while it is on, recent
    /// projects, Forge's, agent activity; each at least 300 px wide, so the grid wraps.
    fn cards(&mut self, window: &Window, cx: &mut Context<Self>) -> Vec<Div> {
        let at_work = self.render_at_work(window, cx);
        let rusty_cards = self
            .rusty
            .as_ref()
            .map(|(rusty, _)| rusty.clone())
            .map(|rusty| rusty.update(cx, |cards, cx| cards.home_cards(cx)))
            .unwrap_or_default();
        let threads = self.workspace.clone();
        let activity = self.workspace.clone();
        std::iter::once(card_with(
            "AGENTS AT WORK",
            "marley-home-at-work",
            Some(card_link(
                "marley-home-at-work-threads",
                "Threads",
                move |window, cx| {
                    threads
                        .update(cx, |workspace, cx| {
                            crate::threads_page::open(workspace, window, cx);
                        })
                        .log_err();
                },
            )),
            at_work,
            cx,
        ))
        .chain(rusty_cards)
        .chain(std::iter::once(card_with(
            "RECENT PROJECTS",
            "marley-home-recent",
            Some(card_link(
                "marley-home-recent-open-folder",
                "Open Folder\u{2026}",
                |window, cx| window.dispatch_action(Box::new(workspace::Open::DEFAULT), cx),
            )),
            self.render_recent(cx),
            cx,
        )))
        .chain(
            crate::forge_watch::render_home_card(cx)
                .map(|forge| card("FORGE", "marley-home-forge", forge, cx)),
        )
        .chain(std::iter::once(card_with(
            "AGENT ACTIVITY",
            "marley-home-activity",
            Some(card_link(
                "marley-home-activity-open",
                "Open",
                move |window, cx| {
                    crate::agent_activity::open_later(activity.clone(), window, cx);
                },
            )),
            self.render_activity(cx),
            cx,
        )))
        // Each card takes at least 300 px of the row and grows, so the grid wraps, as the board's
        // `auto-fit` does.
        .map(|card| card.flex_grow(1.).flex_basis(px(300.)).min_w_0())
        .collect()
    }

    fn render_configure(&self) -> impl IntoElement {
        let entries: [(&'static str, &'static str); 7] = [
            ("marley-home-settings", "Settings"),
            ("marley-home-marley-settings", "Marley Settings"),
            ("marley-home-keymap", "Keymap"),
            ("marley-home-extensions", "Extensions"),
            ("marley-home-mcp-servers", "MCP Servers"),
            ("marley-home-components", "Components"),
            ("marley-home-guide", "Marley Guide"),
        ];
        h_flex()
            .debug_selector(|| "marley-home-configure".into())
            .flex_wrap()
            .justify_center()
            .gap_2()
            .children(entries.into_iter().map(|(id, label)| {
                let focus = self.focus_handle.clone();
                // The board's row of links under the cards (#835, #903).
                Button::new(id, label)
                    .style(ButtonStyle::Transparent)
                    .color(Color::Accent)
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
                        "marley-home-mcp-servers" => {
                            focus.dispatch_action(&crate::mcp_servers::OpenMcpServers, window, cx);
                        }
                        "marley-home-components" => focus.dispatch_action(
                            &crate::component_library::OpenComponentLibrary,
                            window,
                            cx,
                        ),
                        _ => focus.dispatch_action(&OpenGuide, window, cx),
                    })
            }))
    }
}

/// A card: a muted title over its body.
pub(crate) fn card(
    title: &'static str,
    selector: &'static str,
    body: impl IntoElement,
    cx: &App,
) -> Div {
    card_with(title, selector, None, body, cx)
}

/// A card as the Home board draws one (#903): rounded, its title in capitals with a link at its
/// right when it has one.
pub(crate) fn card_with(
    title: &'static str,
    selector: &'static str,
    link: Option<AnyElement>,
    body: impl IntoElement,
    cx: &App,
) -> Div {
    let colors = cx.theme().colors();
    v_flex()
        .debug_selector(move || selector.into())
        .gap_2p5()
        .px_4()
        .py_3p5()
        .rounded(px(14.))
        .border_1()
        .border_color(colors.border)
        .bg(colors.panel_background)
        .child(
            h_flex()
                .justify_between()
                .child(
                    Label::new(title)
                        .size(LabelSize::XSmall)
                        .weight(FontWeight::SEMIBOLD)
                        .color(Color::Muted),
                )
                .children(link),
        )
        .child(body)
}

/// A card's link at the right of its title (#903).
pub(crate) fn card_link(
    id: &'static str,
    label: &'static str,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    Button::new(id, label)
        .style(ButtonStyle::Transparent)
        .label_size(LabelSize::Small)
        .color(Color::Accent)
        .on_click(move |_, window, cx| on_click(window, cx))
        .into_any_element()
}

/// A round-ended button, lit while `pressed` (#903): the agent chips and the start actions.
fn pill(id: impl Into<ElementId>, pressed: bool, cx: &App) -> Stateful<Div> {
    let colors = cx.theme().colors();
    let accent = colors.text_accent;
    let hover = colors.ghost_element_hover;
    h_flex()
        .id(id)
        .flex_none()
        .h(px(30.))
        .px_2p5()
        .gap_1p5()
        .rounded_full()
        .border_1()
        .map(|pill| {
            if pressed {
                pill.border_color(accent.opacity(0.6))
                    .bg(accent.opacity(0.15))
            } else {
                pill.border_color(colors.border)
            }
        })
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
}

/// An agent CLI's two letters, on its chip's tile (#903).
const fn cli_glyph(kind: AgentKind) -> &'static str {
    match kind {
        AgentKind::Claude => "CC",
        AgentKind::Codex => "CX",
        AgentKind::Gemini => "G",
        AgentKind::OpenCode => "OC",
    }
}

/// A chip's small tile with its letters (#903).
fn glyph_tile(glyph: &'static str, cx: &App) -> Div {
    div()
        .flex_none()
        .size(px(18.))
        .rounded(px(5.))
        .bg(cx.theme().colors().element_background)
        .flex()
        .items_center()
        .justify_center()
        .child(
            Label::new(glyph)
                .size(LabelSize::XSmall)
                .buffer_font(cx)
                .color(Color::Accent),
        )
}

/// An agent at work's tile, its state a dot at the corner (#903).
fn state_tile(dot: gpui::Hsla, cx: &App) -> Div {
    let colors = cx.theme().colors();
    div()
        .relative()
        .flex_none()
        .size(px(30.))
        .rounded(px(8.))
        .bg(colors.element_background)
        .flex()
        .items_center()
        .justify_center()
        .child(
            Icon::new(IconName::Sparkle)
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            div()
                .absolute()
                .right(px(-2.))
                .bottom(px(-2.))
                .size(px(10.))
                .rounded_full()
                .border_2()
                .border_color(colors.panel_background)
                .bg(dot),
        )
}

/// A clickable row of a card.
pub(crate) fn row(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    let hover = cx.theme().colors().ghost_element_hover;
    h_flex()
        .id(id)
        .w_full()
        .gap_2()
        .px_2()
        .py_1()
        .rounded_sm()
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
}

pub(crate) fn muted(text: impl Into<SharedString>) -> AnyElement {
    div()
        .px_2()
        .child(
            Label::new(text.into())
                .size(LabelSize::Small)
                .color(Color::Muted),
        )
        .into_any_element()
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
        if self.prompt.is_none() {
            self.prompt = Some(cx.new(|cx| {
                let mut editor = editor::Editor::auto_height(2, 6, window, cx);
                editor.set_placeholder_text(
                    "Ask Marley, or describe a task and pick who does it",
                    window,
                    cx,
                );
                editor
            }));
        }
        let colors = cx.theme().colors();
        let background = colors.editor_background;
        let cards = self.cards(window, cx);
        v_flex()
            .id("marley-home")
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .items_center()
            .px_6()
            .pt(px(56.))
            .pb_8()
            .gap_7()
            .bg(background)
            .child(
                v_flex()
                    .w_full()
                    .max_w(px(860.))
                    .gap(px(18.))
                    .child(
                        div().flex().justify_center().child(
                            Label::new("What are we working on?")
                                .size(LabelSize::Custom(rems(1.875)))
                                .weight(FontWeight::SEMIBOLD),
                        ),
                    )
                    .child(self.render_prompt(cx))
                    .child(self.render_start(cx)),
            )
            .child(
                h_flex()
                    .debug_selector(|| "marley-home-cards".into())
                    .w_full()
                    .max_w(px(1080.))
                    .flex_wrap()
                    .items_start()
                    .gap(px(14.))
                    .children(cards),
            )
            .child(self.render_configure())
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
        Some(Icon::new(IconName::MarleyHome))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static(
            "Home: a terminal, an agent, a project",
        ))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    // Home holds nothing to copy, so a split shows a new one, and Zed's split button stays live
    // on a pane whose only tab is Home (#785).
    fn can_split(&self) -> bool {
        true
    }

    fn clone_on_split(
        &self,
        _workspace_id: Option<WorkspaceId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Option<Entity<Self>>> {
        let workspace = self.workspace.clone();
        let fs = Arc::clone(&self.fs);
        Task::ready(Some(cx.new(|cx| Self::new(workspace, fs, window, cx))))
    }
}

fn worktree_paths(project: &Project, cx: &App) -> Vec<PathBuf> {
    project
        .visible_worktrees(cx)
        .map(|worktree| worktree.read(cx).abs_path().to_path_buf())
        .collect()
}

/// Starts `kind` in `workspace`, with `task` as its first prompt when there is one.
fn start_in(
    workspace: &Workspace,
    kind: AgentKind,
    task: &str,
    window: &Window,
    cx: &Context<Workspace>,
) {
    if task.is_empty() {
        agents::start_cli(workspace, kind, window, cx);
        return;
    }
    let started = agents::start_cli_with_prompt(workspace, kind, task, None, window, cx);
    cx.spawn(async move |_, _| {
        if started.await.is_none() {
            log::warn!("home: {} did not start for the prompt", kind.display_name());
        }
    })
    .detach();
}
