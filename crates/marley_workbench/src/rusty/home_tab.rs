//! The Rusty home page (#679): a dashboard tab, always first in the window's Rusty group (#675).
//!
//! The rail's one Rusty button opens it. A card holds every Rusty screen's button; the cards
//! under it list the vault's recent pages, the decisions whose follow-up is due and the open tasks
//! of every list, each row opening what it names: pages and decisions in the Brain tab (#678),
//! tasks in Tasks on their list.

use std::sync::Arc;

use gpui::{
    Action, App, AppContext as _, ClickEvent, Context, Entity, EventEmitter, FocusHandle,
    Focusable, Render, SharedString, Stateful, Subscription, WeakEntity, Window, actions,
};
use marley_rusty::decisions::{BRAIN_DUE, DecisionSummary, due_from_answer};
use marley_rusty::tasks::{
    LIST_TASK_GROUPS, LIST_TASKS, TaskGroup, groups_from_answer, tasks_from_answer,
};
use marley_rusty::vault;
use serde_json::json;
use ui::{ButtonStyle, ContextMenu, ContextMenuEntry, prelude::*};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::{MarleyNewItemMenu, MultiWorkspace, MultiWorkspaceEvent, Workspace};

use super::brain::{Screen, open_screen};
use super::capture::{CaptureToInbox, CaptureToToday, CaptureUrl};
use super::page::OpenPage;

actions!(
    rusty,
    [
        /// Opens Rusty's home page: every screen's button, the recent pages, the follow-ups due
        /// and the open tasks.
        #[derive(Eq)]
        OpenHome,
    ]
);

/// How many rows the recent pages and the follow-ups list.
const ROWS_SHOWN: usize = 8;

/// How many open tasks the table lists.
const TASKS_SHOWN: usize = 12;

/// Registers `rusty: open home` on every workspace, the Rusty group's + menu, and keeps the Rusty
/// and Home groups from showing no tab (#699, #701); `rusty::init` calls it once.
pub(super) fn init(cx: &mut App) {
    cx.observe_new(
        |workspace: &mut Workspace, window, cx: &mut Context<Workspace>| {
            workspace.register_action(|_, _: &OpenHome, window, cx| {
                open_later(cx.weak_entity(), window, cx);
            });
            let Some(window) = window else {
                return;
            };
            // Its last tab closed.
            cx.subscribe_in(
                &cx.entity(),
                window,
                |workspace, _, event: &workspace::Event, window, cx| {
                    if let workspace::Event::ItemRemoved { .. } = event {
                        fill(workspace, window, cx);
                        crate::home_page::fill_emptied(workspace, window, cx);
                    }
                },
            )
            .detach();
        },
    )
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
    cx.set_global(MarleyNewItemMenu(Arc::new(rusty_links)));
}

/// Opens the home page in `workspace` when it is the Rusty group's and holds no tab, while Rusty is
/// on, so the group never shows Zed's Welcome page (#699).
fn fill(workspace: &Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    if super::is_on(cx)
        && crate::groups::is_rusty_workspace(cx.entity_id(), cx)
        && workspace.items(cx).next().is_none()
    {
        ensure(workspace, window, cx);
    }
}

/// [`fill`] on the workspace the window shows, and Home's page when it is the Home group (#701).
fn fill_shown(multi_workspace: &MultiWorkspace, window: &mut Window, cx: &mut App) {
    let shown = multi_workspace.workspace().clone();
    if crate::groups::is_rusty_workspace(shown.entity_id(), cx) {
        shown.update(cx, |workspace, cx| fill(workspace, window, cx));
    } else if crate::groups::is_home_workspace(shown.entity_id(), cx) {
        shown.update(cx, |workspace, cx| {
            crate::home_page::ensure(workspace, window, cx);
        });
    }
}

/// The Rusty group's + menu: Rusty's quick links ahead of Zed's entries (#699). Any other
/// workspace's menu comes back unchanged.
fn rusty_links(
    menu: ContextMenu,
    workspace: &WeakEntity<Workspace>,
    _window: &mut Window,
    cx: &mut App,
) -> ContextMenu {
    if !super::is_on(cx) || !crate::groups::is_rusty_workspace(workspace.entity_id(), cx) {
        return menu;
    }
    let home = workspace.clone();
    let menu = menu.item(
        ContextMenuEntry::new("Home")
            .icon(super::RUSTY_ICON)
            .action(OpenHome.boxed_clone())
            .handler(move |window, cx| open_later(home.clone(), window, cx)),
    );
    let menu = Screen::ALL.iter().fold(menu, |menu, &screen| {
        menu.item(
            ContextMenuEntry::new(screen.label())
                .icon(screen.icon())
                .handler(move |window, cx| RustyHome::open_screen(screen, window, cx)),
        )
    });
    let forms: [(&'static str, IconName, Box<dyn Action>); 4] = [
        (
            "Open Page…",
            IconName::FileMarkdown,
            Box::new(OpenPage {
                slug: None,
                preview: false,
            }),
        ),
        (
            "Capture to Today…",
            IconName::Plus,
            Box::new(CaptureToToday),
        ),
        (
            "Capture to Inbox…",
            IconName::Plus,
            Box::new(CaptureToInbox),
        ),
        ("Capture a URL…", IconName::Link, Box::new(CaptureUrl)),
    ];
    forms
        .into_iter()
        .fold(menu.separator(), |menu, (label, icon, action)| {
            menu.item(
                ContextMenuEntry::new(label)
                    .icon(icon)
                    .action(action.boxed_clone())
                    // The + menu sets no action context, so this is Zed's own `action` entry's
                    // dispatch.
                    .handler(move |window, cx| window.dispatch_action(action.boxed_clone(), cx)),
            )
        })
        .separator()
}

/// Opens the window's Rusty home page in the Rusty group, or brings it forward.
pub(crate) fn open_later(workspace: WeakEntity<Workspace>, window: &Window, cx: &mut App) {
    // `in_rusty_group` makes the page first in the group before this runs.
    super::in_rusty_group(workspace, window, cx, |workspace, window, cx| {
        if !super::capture::ready(workspace, cx) {
            return;
        }
        if let Some(home) = home_of(workspace, cx) {
            workspace.activate_item(&home, true, true, window, cx);
        }
    });
}

fn home_of(workspace: &Workspace, cx: &App) -> Option<Entity<RustyHome>> {
    workspace.items_of_type::<RustyHome>(cx).next()
}

/// Makes the home page the first tab of `workspace`'s active pane when the workspace has none:
/// `in_rusty_group` calls it before every open in the Rusty group, so the page is always there and
/// always first (#679).
pub(super) fn ensure(workspace: &Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    if home_of(workspace, cx).is_some() {
        return;
    }
    let weak_workspace = cx.weak_entity();
    let home = cx.new(|cx| RustyHome::new(weak_workspace, window, cx));
    let pane = workspace.active_pane().clone();
    pane.update(cx, |pane, cx| {
        pane.add_item(Box::new(home), false, false, Some(0), window, cx);
    });
}

/// An open task, as the table lists it.
struct TaskRow {
    title: String,
    list: String,
    list_id: i64,
}

/// What the cards show of Rusty's data.
struct HomeData {
    due: Vec<DecisionSummary>,
    tasks: Vec<TaskRow>,
}

enum Read {
    Loading,
    Done(HomeData),
    Failed(SharedString),
}

/// Rusty's home page.
pub(crate) struct RustyHome {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    read: Read,
    /// Whether a read is in flight, and whether another was asked for meanwhile.
    reading: bool,
    again: bool,
    _subscriptions: [Subscription; 2],
}

impl std::fmt::Debug for RustyHome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustyHome")
            .field("reading", &self.reading)
            .finish_non_exhaustive()
    }
}

impl RustyHome {
    fn new(workspace: WeakEntity<Workspace>, window: &Window, cx: &mut Context<Self>) -> Self {
        let subscriptions = [
            // A change Rusty announces: the cards read again.
            cx.observe_global_in::<super::Announced>(window, |this, _, cx| this.load(cx)),
            // Connected again after a failed read.
            cx.observe_global_in::<super::Rusty>(window, |this, _, cx| {
                if super::is_connected(cx) && matches!(this.read, Read::Failed(_)) {
                    this.load(cx);
                }
                cx.notify();
            }),
        ];
        let mut this = Self {
            workspace,
            focus_handle: cx.focus_handle(),
            read: Read::Loading,
            reading: false,
            again: false,
            _subscriptions: subscriptions,
        };
        this.load(cx);
        this
    }

    /// Reads the follow-ups due and every list's open tasks, as the Knowledge panel's project view
    /// does (#655); one read at a time, and one more when asked for during it.
    fn load(&mut self, cx: &Context<Self>) {
        if self.reading {
            self.again = true;
            return;
        }
        self.reading = true;
        let due = super::call_tool(BRAIN_DUE, json!({ "days": 0 }), cx);
        let groups = super::call_tool(LIST_TASK_GROUPS, json!({}), cx);
        cx.spawn(async move |this, cx| {
            let (due, groups) = futures::join!(due, groups);
            let read = match (due, groups) {
                (Ok(due), Ok(groups)) => match (due_from_answer(&due), groups_from_answer(&groups))
                {
                    (Ok(due), Ok(groups)) => {
                        let asking = this.update(cx, |_, cx| {
                            groups
                                .iter()
                                .map(|group| {
                                    super::call_tool(
                                        LIST_TASKS,
                                        json!({ "group_id": group.id, "include_archived": false }),
                                        cx,
                                    )
                                })
                                .collect::<Vec<_>>()
                        });
                        match asking {
                            Ok(asking) => {
                                let answers = futures::future::join_all(asking).await;
                                task_rows(&groups, answers).map(|tasks| HomeData { due, tasks })
                            }
                            Err(error) => Err(error.to_string()),
                        }
                    }
                    (Err(error), _) => Err(format!("{BRAIN_DUE}'s answer did not parse: {error}")),
                    (_, Err(error)) => Err(format!(
                        "{LIST_TASK_GROUPS}'s answer did not parse: {error}"
                    )),
                },
                (Err(error), _) | (_, Err(error)) => Err(error),
            };
            this.update(cx, |this, cx| {
                this.reading = false;
                this.read = match read {
                    Ok(data) => Read::Done(data),
                    Err(error) => Read::Failed(error.into()),
                };
                cx.notify();
                if std::mem::take(&mut this.again) {
                    this.load(cx);
                }
            })
            .log_err();
        })
        .detach();
    }

    fn open_screen(screen: Screen, window: &Window, cx: &mut App) {
        if let Some(multi_workspace) = window.root::<MultiWorkspace>().flatten() {
            open_screen(screen, &multi_workspace.downgrade(), window, cx);
        }
    }

    fn render_screens(cx: &Context<Self>) -> impl IntoElement {
        h_flex()
            .flex_wrap()
            .gap_2()
            .children(Screen::ALL.iter().map(|&screen| {
                Button::new(screen.id(), screen.label())
                    .start_icon(Icon::new(screen.icon()).size(IconSize::Small))
                    .style(ButtonStyle::Filled)
                    .on_click(cx.listener(move |_, _: &ClickEvent, window, cx| {
                        Self::open_screen(screen, window, cx);
                    }))
            }))
    }

    fn render_recent(&self, cx: &Context<Self>) -> AnyElement {
        let recent = super::page_picker::recent_slugs(cx);
        if recent.is_empty() {
            return muted("No page opened yet.");
        }
        v_flex()
            .children(
                recent
                    .into_iter()
                    .take(ROWS_SHOWN)
                    .enumerate()
                    .map(|(index, slug)| {
                        let folder = slug.rsplit_once('/').map(|(folder, _)| folder.to_string());
                        let name = vault::name_of(&slug).to_string();
                        let workspace = self.workspace.clone();
                        row(("marley-rusty-home-recent", index), cx)
                            .child(
                                Icon::new(IconName::File)
                                    .size(IconSize::Small)
                                    .color(Color::Muted),
                            )
                            .child(Label::new(name).size(LabelSize::Small).truncate())
                            .children(folder.map(|folder| {
                                Label::new(folder)
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted)
                                    .truncate()
                            }))
                            .on_click(move |_, window, cx| {
                                super::brain_tab::open_page_later(
                                    workspace.clone(),
                                    slug.clone(),
                                    window,
                                    cx,
                                );
                            })
                    }),
            )
            .into_any_element()
    }

    fn render_due(&self, cx: &Context<Self>) -> AnyElement {
        let due = match &self.read {
            Read::Loading => return muted("Reading Rusty…"),
            Read::Failed(error) => return muted(error.clone()),
            Read::Done(data) => &data.due,
        };
        if due.is_empty() {
            return muted("No follow-up due.");
        }
        v_flex()
            .children(
                due.iter()
                    .take(ROWS_SHOWN)
                    .enumerate()
                    .map(|(index, decision)| {
                        let title = if decision.title.is_empty() {
                            vault::name_of(&decision.slug).to_string()
                        } else {
                            decision.title.clone()
                        };
                        let date = decision.follow_up_by.clone().unwrap_or_default();
                        let color = if decision.overdue {
                            Color::Error
                        } else {
                            Color::Muted
                        };
                        let (workspace, slug) = (self.workspace.clone(), decision.slug.clone());
                        row(("marley-rusty-home-due", index), cx)
                            .child(
                                Icon::new(IconName::CheckDouble)
                                    .size(IconSize::Small)
                                    .color(Color::Muted),
                            )
                            .child(
                                div()
                                    .min_w_0()
                                    .flex_1()
                                    .child(Label::new(title).size(LabelSize::Small).truncate()),
                            )
                            .child(Label::new(date).size(LabelSize::XSmall).color(color))
                            .on_click(move |_, window, cx| {
                                super::brain_tab::open_page_later(
                                    workspace.clone(),
                                    slug.clone(),
                                    window,
                                    cx,
                                );
                            })
                    }),
            )
            .into_any_element()
    }

    fn render_tasks(&self, cx: &Context<Self>) -> AnyElement {
        let tasks = match &self.read {
            Read::Loading => return muted("Reading Rusty…"),
            Read::Failed(error) => return muted(error.clone()),
            Read::Done(data) => &data.tasks,
        };
        if tasks.is_empty() {
            return muted("No open task.");
        }
        let header = h_flex()
            .px_2()
            .gap_2()
            .child(
                div().flex_1().child(
                    Label::new("Task")
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                ),
            )
            .child(
                div().w_40().child(
                    Label::new("List")
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                ),
            );
        let more = tasks.len().saturating_sub(TASKS_SHOWN);
        v_flex()
            .child(header)
            .children(
                tasks
                    .iter()
                    .take(TASKS_SHOWN)
                    .enumerate()
                    .map(|(index, task)| {
                        let (workspace, list_id) = (self.workspace.clone(), task.list_id);
                        row(("marley-rusty-home-task", index), cx)
                            .child(
                                Icon::new(IconName::Circle)
                                    .size(IconSize::XSmall)
                                    .color(Color::Muted),
                            )
                            .child(
                                div().min_w_0().flex_1().child(
                                    Label::new(task.title.clone())
                                        .size(LabelSize::Small)
                                        .truncate(),
                                ),
                            )
                            .child(
                                div().w_40().child(
                                    Label::new(task.list.clone())
                                        .size(LabelSize::Small)
                                        .color(Color::Muted)
                                        .truncate(),
                                ),
                            )
                            .on_click(move |_, window, cx| {
                                super::tasks_tab::open_later(
                                    workspace.clone(),
                                    Some(list_id),
                                    window,
                                    cx,
                                );
                            })
                    }),
            )
            .when(more > 0, |table| {
                table.child(
                    div()
                        .px_2()
                        .pt_1()
                        .child(muted(format!("and {more} more in Tasks"))),
                )
            })
            .into_any_element()
    }
}

/// Every list's open tasks, in the lists' order, each with its list.
fn task_rows(
    groups: &[TaskGroup],
    answers: Vec<Result<String, String>>,
) -> Result<Vec<TaskRow>, String> {
    let mut rows = Vec::new();
    for (group, answer) in groups.iter().zip(answers) {
        let tasks = tasks_from_answer(&answer?)
            .map_err(|error| format!("{LIST_TASKS}'s answer did not parse: {error}"))?;
        rows.extend(
            tasks
                .into_iter()
                .filter(|task| !task.completed)
                .map(|task| TaskRow {
                    title: task.title,
                    list: group.name.clone(),
                    list_id: group.id,
                }),
        );
    }
    Ok(rows)
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

impl Render for RustyHome {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        v_flex()
            .id("marley-rusty-home")
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_4()
            .bg(colors.editor_background)
            .child(card(
                "RUSTY",
                "marley-rusty-home-screens",
                Self::render_screens(cx),
                cx,
            ))
            .child(
                h_flex()
                    .w_full()
                    .gap_4()
                    .items_start()
                    .child(
                        card(
                            "RECENT PAGES",
                            "marley-rusty-home-recent",
                            self.render_recent(cx),
                            cx,
                        )
                        .flex_1()
                        .min_w_0(),
                    )
                    .child(
                        card(
                            "FOLLOW-UPS DUE",
                            "marley-rusty-home-due",
                            self.render_due(cx),
                            cx,
                        )
                        .flex_1()
                        .min_w_0(),
                    ),
            )
            .child(card(
                "TASKS",
                "marley-rusty-home-tasks",
                self.render_tasks(cx),
                cx,
            ))
    }
}

impl Focusable for RustyHome {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for RustyHome {}

impl Item for RustyHome {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Rusty")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(super::RUSTY_ICON))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static("Rusty's home page"))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }
}
