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
    Render, SharedString, Stateful, Subscription, Task, WeakEntity, Window,
};
use marley_agent::AgentKind;
use settings::SettingsStore;
use terminal_view::terminal_panel::TerminalPanel;
use ui::{ButtonStyle, prelude::*};
use util::ResultExt as _;
use util::paths::PathExt as _;
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
    /// The rail the page lists the agents at work from, held weakly and observed, so a rail a
    /// layout swap replaced goes (#713).
    rail: Option<(WeakEntity<Rail>, Subscription)>,
    /// The agent activity and the kill switch, for the AGENT ACTIVITY card (#703), and the project
    /// marks the recent projects show (#831).
    _activity: [Subscription; 3],
    /// Rusty's cards, held while `marley.rusty.enabled` is on (#782).
    rusty: Option<Entity<RustyCards>>,
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
            rail: None,
            _activity: [
                cx.observe_global_in::<AgentActivity>(window, |_, _, cx| cx.notify()),
                cx.observe_global_in::<SettingsStore>(window, |_, _, cx| cx.notify()),
                cx.observe_global_in::<crate::project_icons::ProjectMarks>(window, |_, _, cx| {
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
            cx.new(|cx| RustyCards::new(workspace, focus_handle, window, cx))
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
                    .start_icon(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
                    .style(ButtonStyle::Subtle)
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

    /// The PROMPT card (#835): a box for the task, the agent CLIs to start it with, where it
    /// starts, and Start; under it the links START and NEW AGENT held.
    fn render_prompt(&self, cx: &Context<Self>) -> AnyElement {
        let kinds = agents::installed_clis(agents::launcher(cx).search_path.as_deref());
        let picked = self.picked.or_else(|| kinds.first().copied());
        let folder = self.workspace.upgrade().map(|workspace| {
            terminal_view::default_working_directory(workspace.read(cx), cx).map_or_else(
                || "your home folder".to_string(),
                |folder| folder.compact().to_string_lossy().into_owned(),
            )
        });
        let colors = cx.theme().colors();
        let field = div()
            .key_context("MarleyHomePrompt")
            .on_action(cx.listener(|home, _: &menu::Confirm, window, cx| {
                home.start_prompt(window, cx);
            }))
            .w_full()
            .px_3()
            .py_2()
            .rounded_lg()
            .border_1()
            .border_color(colors.border)
            .bg(colors.editor_background)
            .children(self.prompt.clone());
        let chips = kinds.into_iter().enumerate().map(|(index, kind)| {
            Button::new(("marley-home-agent", index), kind.display_name())
                .start_icon(Icon::new(agents::cli_icon(kind)).size(IconSize::Small))
                .style(ButtonStyle::Filled)
                .toggle_state(picked == Some(kind))
                .selected_style(ButtonStyle::Tinted(ui::TintColor::Accent))
                .on_click(cx.listener(move |home, _, _, cx| {
                    home.picked = Some(kind);
                    cx.notify();
                }))
        });
        let start = Button::new("marley-home-start-agent", "Start")
            .style(ButtonStyle::Tinted(ui::TintColor::Accent))
            .on_click(cx.listener(|home, _, window, cx| home.start_prompt(window, cx)));
        v_flex()
            .gap_2()
            .child(field)
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .flex_wrap()
                    .children(chips)
                    .children(folder.map(|folder| {
                        Label::new(format!("in {folder}"))
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                    }))
                    .child(div().flex_1())
                    .child(start),
            )
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_1()
                    .child(self.render_start())
                    .child(self.render_agent_links()),
            )
            .into_any_element()
    }

    /// New Agent…, the picker that asks which agent and where (#735), and Threads (#745), as links.
    fn render_agent_links(&self) -> impl IntoElement {
        let picker_workspace = self.workspace.clone();
        let threads_workspace = self.workspace.clone();
        h_flex()
            .gap_1()
            .child(
                Button::new("marley-home-new-agent", "New Agent…")
                    .start_icon(
                        Icon::new(IconName::Plus)
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .style(ButtonStyle::Subtle)
                    .on_click(move |_, window, cx| {
                        picker_workspace
                            .update(cx, |workspace, cx| {
                                agents::show_picker(workspace, None, window, cx);
                            })
                            .log_err();
                    }),
            )
            .child(
                Button::new("marley-home-threads", "Threads")
                    .start_icon(
                        Icon::new(IconName::Thread)
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .style(ButtonStyle::Subtle)
                    .on_click(move |_, window, cx| {
                        threads_workspace
                            .update(cx, |workspace, cx| {
                                crate::threads_page::open(workspace, window, cx);
                            })
                            .log_err();
                    }),
            )
    }

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
        workspace.update(cx, |workspace, cx| {
            if task.is_empty() {
                agents::start_cli(workspace, kind, window, cx);
                return;
            }
            let started = agents::start_cli_with_prompt(workspace, kind, &task, None, window, cx);
            cx.spawn(async move |_, _| {
                if started.await.is_none() {
                    log::warn!("home: {} did not start for the prompt", kind.display_name());
                }
            })
            .detach();
        });
        if let Some(prompt) = &self.prompt {
            prompt.update(cx, |editor, cx| editor.set_text("", window, cx));
        }
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
        let entries: [(&'static str, &'static str, IconName); 7] = [
            ("marley-home-settings", "Settings", IconName::Settings),
            (
                "marley-home-marley-settings",
                "Marley Settings",
                IconName::Settings,
            ),
            ("marley-home-keymap", "Keymap", IconName::Keyboard),
            ("marley-home-extensions", "Extensions", IconName::Blocks),
            (
                "marley-home-mcp-servers",
                "MCP Servers",
                IconName::ToolHammer,
            ),
            ("marley-home-components", "Components", IconName::Blocks),
            ("marley-home-guide", "Marley Guide", IconName::Book),
        ];
        h_flex()
            .flex_wrap()
            .gap_2()
            .children(entries.into_iter().map(|(id, label, icon)| {
                let focus = self.focus_handle.clone();
                // Configure's entries read as links (#835).
                Button::new(id, label)
                    .start_icon(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
                    .style(ButtonStyle::Subtle)
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
    let colors = cx.theme().colors();
    v_flex()
        .debug_selector(move || selector.into())
        .gap_2()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(colors.border)
        .bg(colors.panel_background)
        .child(Label::new(title).size(LabelSize::Small).color(Color::Muted))
        .child(body)
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
                let mut editor = editor::Editor::single_line(window, cx);
                editor.set_placeholder_text("What should an agent do?", window, cx);
                editor
            }));
        }
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
            // One prompt first (#835); START's and NEW AGENT's actions are its links.
            .child(card(
                "PROMPT",
                "marley-home-prompt",
                self.render_prompt(cx),
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
            .children(self.rusty.clone())
            .child(card(
                "AGENT ACTIVITY",
                "marley-home-activity",
                self.render_activity(cx),
                cx,
            ))
            // The card used least goes last (#782).
            .child(card(
                "CONFIGURE",
                "marley-home-configure",
                self.render_configure(),
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
