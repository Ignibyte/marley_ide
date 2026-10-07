//! The Knowledge panel (#646), in the right dock while Rusty is on.
//!
//! The active Page tab's tags with their page counts, the pages that link to it with the line each
//! link sits on, its own links in order (one to no page yet offered as a new page), and brain
//! search, sent on Enter.
//!
//! It is always added and hides itself while Rusty is off, as Zed's Agent Panel does while AI is
//! off: no dock button, a toggle that says why, and a dock that closes when it was showing the
//! panel. It follows the workspace's active item, as Zed's outline panel follows the editor.
//!
//! With no Page tab in front it shows the project view (#655): the brain project page the
//! workspace's folders resolve to, its summary, its follow-ups due and its task group's open tasks,
//! or the pages to link when none resolves.

use std::mem;

use editor::Editor;
use gpui::{
    Action, AnyElement, App, Context, Entity, EventEmitter, FocusHandle, Focusable, Pixels,
    SharedString, Subscription, WeakEntity, Window, actions, px,
};
use marley_rusty::decisions::{BRAIN_DUE, DecisionSummary, due_from_answer};
use marley_rusty::knowledge::{
    BRAIN_GET_LINKS, BRAIN_GRAPH, BRAIN_TAGS, Graph, Outgoing, PageKnowledge, PageLinks,
    SEARCH_LIMIT, TagCount, page_knowledge, snippet,
};
use marley_rusty::project::{
    BRAIN_READ_PAGE, GroupJoin, Matched, ProjectPage, Resolution, due_for, group_join,
    read_from_answer, summary,
};
use marley_rusty::tasks::{
    LIST_TASK_GROUPS, LIST_TASKS, TaskGroup, UserTask, groups_from_answer, tasks_from_answer,
};
use marley_rusty::vault::{self, BRAIN_NEW_PAGE, BRAIN_SEARCH, SearchHit, hits_from_answer};
use serde_json::json;
use ui::{
    Callout, Chip, HighlightedLabel, IconButtonShape, ListItem, ListItemSpacing, ListSubHeader,
    Tooltip, prelude::*,
};
use util::ResultExt as _;
use workspace::dock::{DockPosition, Panel, PanelEvent};
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use super::page::{PageEvent, PageView};
use super::project::{self, Join, LinkProjectPage, LinkTaskGroup};

actions!(
    rusty,
    [
        /// Shows or hides the Knowledge panel: the open brain page's tags, backlinks and links,
        /// and brain search.
        #[derive(Eq)]
        ToggleKnowledgePanel,
    ]
);

/// After the Fleet panel's 20, before Zed's own.
const ACTIVATION_PRIORITY: u32 = 21;

const DEFAULT_WIDTH: Pixels = px(320.);

/// Registers the panel's toggle on every workspace, and gives each its Knowledge panel; `rusty::init`
/// calls it once.
pub(super) fn init(cx: &App) {
    cx.observe_new(
        |workspace: &mut Workspace, window: Option<&mut Window>, cx: &mut Context<Workspace>| {
            workspace.register_action(|workspace, _: &ToggleKnowledgePanel, window, cx| {
                if !super::is_on(cx) {
                    let reason = super::unavailable(cx).unwrap_or_default();
                    workspace.show_toast(
                        Toast::new(
                            NotificationId::unique::<KnowledgePanel>(),
                            reason.to_string(),
                        ),
                        cx,
                    );
                    return;
                }
                workspace.toggle_panel_focus::<KnowledgePanel>(window, cx);
            });
            let Some(window) = window else {
                return;
            };
            let workspace_entity = cx.entity();
            let panel = cx.new(|cx| KnowledgePanel::new(&workspace_entity, window, cx));
            workspace.add_panel(panel, window, cx);
        },
    )
    .detach();
}

/// The page view's reads, coalesced: one under way, one more queued.
#[derive(Default)]
struct Reads {
    reading: bool,
    again: bool,
}

/// Whether the panel shows in its dock.
#[derive(Clone, Copy, PartialEq, Eq)]
enum InDock {
    Hidden,
    Shown,
}

/// What the project view shows of its page (#655).
struct ProjectData {
    title: String,
    summary: String,
    follow_ups: Vec<DecisionSummary>,
    groups: GroupJoin,
    /// The open tasks of the groups found.
    tasks: Vec<UserTask>,
}

/// Brain search's last query and what came back: `None` while Rusty answers.
struct Results {
    query: String,
    case_sensitive: bool,
    regex: bool,
    hits: Option<Result<Vec<SearchHit>, SharedString>>,
    selected: Option<usize>,
}

/// The Knowledge panel.
pub(crate) struct KnowledgePanel {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    search: Entity<Editor>,
    case_sensitive: bool,
    regex: bool,
    /// The Page tab the panel follows, and its events.
    page_tab: Option<(WeakEntity<PageView>, Subscription)>,
    /// The page shown, and what Rusty gave of it.
    slug: Option<String>,
    knowledge: Option<Result<PageKnowledge, SharedString>>,
    page_reads: Reads,
    results: Option<Results>,
    /// Whether a close of the dock is already asked for, while Rusty is off.
    closing: bool,
    /// Whether the panel shows in its dock, so the project pages are worth reading.
    shown_in_dock: InDock,
    /// The project's join, for the project view (#655).
    join: Join,
    /// The resolved page's slug, and what Rusty gave of it.
    project_view: Option<(String, Option<Result<ProjectData, SharedString>>)>,
    project_reads: Reads,
    /// The workspace's project's folder changes.
    project_events: Option<Subscription>,
    _subscriptions: [Subscription; 5],
}

impl KnowledgePanel {
    fn new(workspace: &Entity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Search the brain…", window, cx);
            editor
        });
        let subscriptions = [
            cx.subscribe_in(
                workspace,
                window,
                |this, workspace, event: &workspace::Event, window, cx| {
                    if matches!(event, workspace::Event::ActiveItemChanged) {
                        this.follow_active_item(workspace, window, cx);
                    }
                },
            ),
            // A change Rusty announces: the page, the project's page and the query are read again.
            cx.observe_global_in::<super::Announced>(window, |this, _, cx| {
                this.load_page(cx);
                this.load_project(cx);
                this.search_again(cx);
            }),
            cx.observe_global_in::<project::ProjectPages>(window, |this, _, cx| {
                this.refresh_join(cx);
            }),
            // The connection's state line, and a panel that opened while Rusty was down.
            cx.observe_global_in::<super::Rusty>(window, |this, _, cx| {
                if super::is_connected(cx) && this.knowledge.as_ref().is_some_and(Result::is_err) {
                    this.load_page(cx);
                }
                cx.notify();
            }),
            // The switch: a panel showing while Rusty turns off closes its dock.
            cx.observe_global_in::<settings::SettingsStore>(window, |this, _, cx| {
                if super::is_on(cx) {
                    this.closing = false;
                }
                cx.notify();
            }),
        ];
        // The workspace is being updated while its panels are made, so the first look at its
        // active item waits until it is not.
        let weak_workspace = workspace.downgrade();
        cx.defer_in(window, {
            let weak_workspace = weak_workspace.clone();
            move |this, window, cx| {
                if let Some(workspace) = weak_workspace.upgrade() {
                    this.watch_project(&workspace, window, cx);
                    this.follow_active_item(&workspace, window, cx);
                }
            }
        });
        Self {
            workspace: weak_workspace,
            focus_handle: cx.focus_handle(),
            search,
            case_sensitive: false,
            regex: false,
            page_tab: None,
            slug: None,
            knowledge: None,
            page_reads: Reads::default(),
            results: None,
            closing: false,
            shown_in_dock: InDock::Hidden,
            join: Join::NoProject,
            project_view: None,
            project_reads: Reads::default(),
            project_events: None,
            _subscriptions: subscriptions,
        }
    }

    /// Follows the workspace's project's folders, which the join reads.
    fn watch_project(
        &mut self,
        workspace: &Entity<Workspace>,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let project = workspace.read(cx).project().clone();
        self.project_events = Some(cx.subscribe_in(
            &project,
            window,
            |_, _, event: &::project::Event, window, cx| {
                if matches!(
                    event,
                    ::project::Event::WorktreePathsChanged { .. }
                        | ::project::Event::WorktreeAdded(_)
                        | ::project::Event::WorktreeRemoved(_)
                ) {
                    // The project emits these in its own update; the join reads the workspace.
                    cx.defer_in(window, |this, _, cx| this.refresh_join(cx));
                }
            },
        ));
    }

    /// The project's join again; a newly resolved page is read.
    fn refresh_join(&mut self, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        if self.shown_in_dock == InDock::Shown && self.slug.is_none() {
            project::ensure(cx);
        }
        let join = project::join(workspace.read(cx), cx);
        if join == self.join {
            return;
        }
        let slug = match &join {
            Join::Resolved(Resolution::Page { slug, .. }) => Some(slug.clone()),
            _ => None,
        };
        self.join = join;
        if slug.as_deref() != self.project_view.as_ref().map(|(shown, _)| shown.as_str()) {
            self.project_view = slug.map(|slug| (slug, None));
            self.load_project(cx);
        }
        cx.notify();
    }

    /// Reads the project page, its links, the follow-ups due and the task groups, then the open
    /// tasks of the groups it joins; an answer for a page no longer shown is dropped.
    fn load_project(&mut self, cx: &Context<Self>) {
        let Some(slug) = self.project_view.as_ref().map(|(slug, _)| slug.clone()) else {
            return;
        };
        if self.project_reads.reading {
            self.project_reads.again = true;
            return;
        }
        self.project_reads.reading = true;
        let page = super::call_tool(BRAIN_READ_PAGE, json!({ "slug": slug }), cx);
        let links = super::call_tool(BRAIN_GET_LINKS, json!({ "slug": slug }), cx);
        let due = super::call_tool(BRAIN_DUE, json!({ "days": 0 }), cx);
        let groups = super::call_tool(LIST_TASK_GROUPS, json!({}), cx);
        cx.spawn(async move |this, cx| {
            let (page, links, due, groups) = futures::join!(page, links, due, groups);
            let data = match project_reads(page, links, due, groups) {
                Err(error) => Err(error),
                Ok((data, ids)) => {
                    let asking = this.update(cx, |_, cx| {
                        ids.iter()
                            .map(|id| {
                                super::call_tool(
                                    LIST_TASKS,
                                    json!({ "group_id": id, "include_archived": false }),
                                    cx,
                                )
                            })
                            .collect::<Vec<_>>()
                    });
                    match asking {
                        Ok(asking) => {
                            let answers = futures::future::join_all(asking).await;
                            open_tasks(answers).map(|tasks| ProjectData { tasks, ..data })
                        }
                        Err(error) => Err(error.to_string()),
                    }
                }
            };
            this.update(cx, |this, cx| {
                this.project_reads.reading = false;
                if let Some((shown, read)) = &mut this.project_view
                    && *shown == slug
                {
                    *read = Some(data.map_err(SharedString::from));
                    cx.notify();
                }
                if mem::take(&mut this.project_reads.again) {
                    this.load_project(cx);
                }
            })
            .log_err();
        })
        .detach();
    }

    /// The active item's page, when it is a Page tab, followed as it navigates.
    fn follow_active_item(
        &mut self,
        workspace: &Entity<Workspace>,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let page = workspace
            .read(cx)
            .active_item(cx)
            .and_then(|item| super::page::page_in(item.as_ref(), cx));
        let Some(page) = page else {
            self.page_tab = None;
            self.show(None, cx);
            self.refresh_join(cx);
            return;
        };
        let followed = self
            .page_tab
            .as_ref()
            .is_some_and(|(tab, _)| tab.entity_id() == page.entity_id());
        if !followed {
            let events = cx.subscribe_in(&page, window, |this, page, event: &PageEvent, _, cx| {
                if matches!(event, PageEvent::UpdateTab) {
                    let slug = page.read(cx).slug().to_string();
                    this.show(Some(slug), cx);
                }
            });
            self.page_tab = Some((page.downgrade(), events));
        }
        let slug = page.read(cx).slug().to_string();
        self.show(Some(slug), cx);
    }

    /// Shows `slug`'s view, read from Rusty, or the no-page state.
    fn show(&mut self, slug: Option<String>, cx: &mut Context<Self>) {
        if self.slug == slug {
            return;
        }
        self.slug = slug;
        self.knowledge = None;
        self.load_page(cx);
        cx.notify();
    }

    /// Reads the shown page's links, its neighbourhood (titles and tags) and the tag counts; an
    /// answer for a page no longer shown is dropped.
    fn load_page(&mut self, cx: &Context<Self>) {
        let Some(slug) = self.slug.clone() else {
            return;
        };
        if self.page_reads.reading {
            self.page_reads.again = true;
            return;
        }
        self.page_reads.reading = true;
        let links = super::call_tool(BRAIN_GET_LINKS, json!({ "slug": slug }), cx);
        let graph = super::call_tool(BRAIN_GRAPH, json!({ "around": slug, "depth": 1 }), cx);
        let tags = super::call_tool(BRAIN_TAGS, json!({}), cx);
        cx.spawn(async move |this, cx| {
            let (links, graph, tags) = futures::join!(links, graph, tags);
            let read = build(&slug, links, graph, tags);
            this.update(cx, |this, cx| {
                this.page_reads.reading = false;
                if this.slug.as_deref() == Some(slug.as_str()) {
                    this.knowledge = Some(read.map_err(SharedString::from));
                    cx.notify();
                }
                if mem::take(&mut this.page_reads.again) {
                    this.load_page(cx);
                }
            })
            .log_err();
        })
        .detach();
    }

    fn query(&self, cx: &App) -> String {
        self.search.read(cx).text(cx).trim().to_string()
    }

    /// Sends the field's query to brain search, as typed: Rusty parses its operators. Sent on
    /// Enter alone: with an embedding provider set, Rusty embeds every query it is sent.
    fn run_search(&mut self, cx: &mut Context<Self>) {
        let query = self.query(cx);
        if query.is_empty() {
            self.results = None;
            cx.notify();
            return;
        }
        let (case_sensitive, regex) = (self.case_sensitive, self.regex);
        self.results = Some(Results {
            query: query.clone(),
            case_sensitive,
            regex,
            hits: None,
            selected: None,
        });
        cx.notify();
        let asking = super::call_tool(
            BRAIN_SEARCH,
            json!({
                "query": query,
                "limit": SEARCH_LIMIT,
                "case_sensitive": case_sensitive,
                "regex": regex,
            }),
            cx,
        );
        cx.spawn(async move |this, cx| {
            let hits = asking.await.and_then(|text| {
                hits_from_answer(&text)
                    .map_err(|error| format!("{BRAIN_SEARCH}'s answer did not parse: {error}"))
            });
            this.update(cx, |this, cx| {
                if let Some(results) = this.results.as_mut().filter(|results| {
                    results.query == query
                        && results.case_sensitive == case_sensitive
                        && results.regex == regex
                }) {
                    results.hits = Some(hits.map_err(SharedString::from));
                    cx.notify();
                }
            })
            .log_err();
        })
        .detach();
    }

    /// The query shown, asked again after a change.
    fn search_again(&mut self, cx: &mut Context<Self>) {
        if self.results.is_some() {
            self.run_search(cx);
        }
    }

    fn hits(&self) -> Option<&[SearchHit]> {
        self.results
            .as_ref()?
            .hits
            .as_ref()?
            .as_ref()
            .ok()
            .map(Vec::as_slice)
    }

    /// Opens `slug` in the workspace's Page tab, as one click in the tree does.
    fn open(&self, slug: &str, window: &Window, cx: &mut App) {
        super::page::open_later(
            self.workspace.clone(),
            slug.to_string(),
            true,
            false,
            window,
            cx,
        );
    }

    fn toast(&self, message: String, cx: &mut App) {
        self.workspace
            .update(cx, |workspace, cx| {
                workspace.show_toast(Toast::new(NotificationId::unique::<Self>(), message), cx);
            })
            .log_err();
    }

    /// Makes the page an unresolved link names and opens it. Rusty's main takes `path`, the
    /// exact target (its TICKET-041); older binaries take `folder` and `name`, which a target split
    /// at its last `/` keeps from flattening into one name.
    fn create(target: &str, window: &Window, cx: &Context<Self>) {
        let asking = super::call_tool(
            BRAIN_NEW_PAGE,
            json!({
                "path": target,
                "folder": vault::folder_of(target),
                "name": vault::name_of(target),
            }),
            cx,
        );
        let target = target.to_string();
        cx.spawn_in(window, async move |this, cx| {
            let made = asking.await.and_then(|text| {
                vault::slug_from_answer(&text)
                    .map_err(|error| format!("{BRAIN_NEW_PAGE}'s answer did not parse: {error}"))
            });
            this.update_in(cx, |this, window, cx| match made {
                Ok(slug) => this.open(&slug, window, cx),
                Err(error) => this.toast(format!("Could not make {target}: {error}"), cx),
            })
            .log_err();
        })
        .detach();
    }

    /// A tag's search: `tag:<name>` in the field, sent.
    fn search_tag(&mut self, tag: &str, window: &mut Window, cx: &mut Context<Self>) {
        let query = format!("tag:{tag}");
        self.search
            .update(cx, |editor, cx| editor.set_text(query, window, cx));
        self.run_search(cx);
    }

    fn select(&mut self, forward: bool, cx: &mut Context<Self>) {
        let count = self.hits().map_or(0, <[_]>::len);
        let Some(results) = self.results.as_mut() else {
            return;
        };
        let Some(last) = count.checked_sub(1) else {
            return;
        };
        results.selected = Some(match (results.selected, forward) {
            (None, true) => 0,
            (None, false) => last,
            (Some(at), true) => (at + 1).min(last),
            (Some(at), false) => at.saturating_sub(1),
        });
        cx.notify();
    }

    /// Enter: a new query (or new options) is sent; the same one opens the selected hit, or the
    /// first.
    fn confirm(&mut self, _: &menu::Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let query = self.query(cx);
        let unchanged = self.results.as_ref().is_some_and(|results| {
            results.query == query
                && results.case_sensitive == self.case_sensitive
                && results.regex == self.regex
        });
        if !unchanged {
            self.run_search(cx);
            return;
        }
        let selected = self
            .results
            .as_ref()
            .and_then(|results| results.selected)
            .unwrap_or(0);
        if let Some(slug) = self
            .hits()
            .and_then(|hits| hits.get(selected))
            .map(|hit| hit.slug.clone())
        {
            self.open(&slug, window, cx);
        }
    }

    /// Escape: the query cleared and the page view back; with nothing to clear it passes on.
    fn cancel(&mut self, _: &menu::Cancel, window: &mut Window, cx: &mut Context<Self>) {
        if self.results.is_none() && self.query(cx).is_empty() {
            cx.propagate();
            return;
        }
        self.search
            .update(cx, |editor, cx| editor.set_text("", window, cx));
        self.results = None;
        cx.notify();
    }

    /// While Rusty is off the panel shows nothing and closes its dock, when the dock is showing
    /// it: once, after this update, since closing reads the panel (PR-claude-defer-in…).
    fn close_while_off(&mut self, window: &Window, cx: &mut Context<Self>) {
        if mem::replace(&mut self.closing, true) {
            return;
        }
        let workspace = self.workspace.clone();
        window.defer(cx, move |window, cx| {
            workspace
                .update(cx, |workspace, cx| {
                    let shows_this = workspace
                        .right_dock()
                        .read(cx)
                        .visible_panel()
                        .is_some_and(|panel| panel.to_any().downcast::<Self>().is_ok());
                    if shows_this {
                        workspace.close_panel::<Self>(window, cx);
                    }
                })
                .log_err();
        });
    }

    fn render_field(&self, cx: &Context<Self>) -> impl IntoElement {
        let toggle = |id: &'static str, icon: IconName, on: bool, tip: &'static str| {
            IconButton::new(id, icon)
                .shape(IconButtonShape::Square)
                .icon_size(IconSize::Small)
                .toggle_state(on)
                .tooltip(Tooltip::text(tip))
        };
        h_flex()
            .w_full()
            .flex_none()
            .gap_1()
            .px_2()
            .py_1p5()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                Icon::new(IconName::MagnifyingGlass)
                    .size(IconSize::Small)
                    .color(Color::Muted),
            )
            .child(div().min_w_0().flex_1().child(self.search.clone()))
            .child(
                toggle(
                    "rusty-knowledge-case",
                    IconName::CaseSensitive,
                    self.case_sensitive,
                    "Match Case",
                )
                .on_click(cx.listener(|this, _, window, cx| {
                    this.case_sensitive = !this.case_sensitive;
                    this.after_toggle(window, cx);
                })),
            )
            .child(
                toggle(
                    "rusty-knowledge-regex",
                    IconName::Regex,
                    self.regex,
                    "Regular Expression",
                )
                .on_click(cx.listener(|this, _, window, cx| {
                    this.regex = !this.regex;
                    this.after_toggle(window, cx);
                })),
            )
    }

    /// A toggle's click gives the field its focus back, so Up, Down and Enter still reach the list,
    /// and sends the query again when one is shown.
    fn after_toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.search.focus_handle(cx), cx);
        self.search_again(cx);
        cx.notify();
    }

    fn render_results(results: &Results, cx: &Context<Self>) -> AnyElement {
        let hits = match &results.hits {
            None => return muted_line("Searching the brain…".to_string()),
            Some(Err(error)) => return error_line(error.to_string()),
            Some(Ok(hits)) => hits,
        };
        let count = match hits.len() {
            0 => "No pages match.".to_string(),
            1 => "1 page".to_string(),
            many => format!("{many} pages"),
        };
        v_flex()
            .w_full()
            .child(muted_line(count))
            .children(hits.iter().enumerate().map(|(at, hit)| {
                let slug = hit.slug.clone();
                let (text, ranges) = snippet(&hit.snippet);
                ListItem::new(("rusty-knowledge-hit", at))
                    .inset(true)
                    .spacing(ListItemSpacing::Sparse)
                    .toggle_state(results.selected == Some(at))
                    .child(
                        v_flex()
                            .min_w_0()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(Label::new(hit.title.clone()).truncate())
                                    .child(
                                        Label::new(hit.slug.clone())
                                            .size(LabelSize::XSmall)
                                            .color(Color::Muted)
                                            .truncate(),
                                    ),
                            )
                            .when(!text.is_empty(), |row| {
                                row.child(
                                    HighlightedLabel::from_ranges(text, ranges)
                                        .size(LabelSize::Small)
                                        .color(Color::Muted),
                                )
                            }),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open(&slug, window, cx);
                    }))
                    .into_any_element()
            }))
            .into_any_element()
    }

    fn render_page(knowledge: &PageKnowledge, cx: &Context<Self>) -> AnyElement {
        v_flex()
            .w_full()
            .gap_1()
            .child(
                v_flex()
                    .px_2p5()
                    .pt_2()
                    .child(Label::new(knowledge.title.clone()).size(LabelSize::Large))
                    .child(
                        Label::new(knowledge.slug.clone())
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .when(!knowledge.tags.is_empty(), |page| {
                page.child(h_flex().flex_wrap().gap_1().px_2p5().py_1().children(
                    knowledge.tags.iter().enumerate().map(|(at, (tag, count))| {
                        let searched = tag.clone();
                        // `ui::Chip` takes no click of its own.
                        div()
                            .id(("rusty-knowledge-tag", at))
                            .cursor_pointer()
                            .tooltip(Tooltip::text(format!("Search tag:{tag}")))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.search_tag(&searched, window, cx);
                            }))
                            .child(Chip::new(format!("#{tag} {count}")))
                    }),
                ))
            })
            .child(section("Backlinks", knowledge.backlinks.len()))
            .children(knowledge.backlinks.iter().enumerate().map(|(at, link)| {
                let slug = link.slug.clone();
                let ranges = link.mention.clone().into_iter().collect();
                ListItem::new(("rusty-knowledge-backlink", at))
                    .inset(true)
                    .spacing(ListItemSpacing::Sparse)
                    .child(
                        v_flex()
                            .min_w_0()
                            .child(Label::new(link.title.clone()).truncate())
                            .child(
                                HighlightedLabel::from_ranges(link.context.clone(), ranges)
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                            ),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open(&slug, window, cx);
                    }))
                    .into_any_element()
            }))
            .child(section("Links", knowledge.outgoing.len()))
            .children(
                knowledge
                    .outgoing
                    .iter()
                    .enumerate()
                    .map(|(at, link)| outgoing_row(at, link, cx)),
            )
            .into_any_element()
    }

    /// The project view, in the no-page place (#655).
    fn render_project(&self, cx: &Context<Self>) -> AnyElement {
        match &self.join {
            Join::NoProject => {
                muted_line("Open a brain page to see its tags, backlinks and links.".to_string())
            }
            Join::Reading => muted_line("Reading the brain's project pages…".to_string()),
            Join::Failed(error) => error_line(error.to_string()),
            Join::Resolved(Resolution::Page { slug, by, also }) => {
                self.render_project_page(slug, *by, also, cx)
            }
            Join::Resolved(Resolution::Candidates { name, slugs }) => {
                render_candidates(name, slugs, cx)
            }
            Join::Resolved(Resolution::Unmatched { folders, names }) => {
                render_unmatched(folders, names)
            }
        }
    }

    fn render_project_page(
        &self,
        slug: &str,
        by: Matched,
        also: &[String],
        cx: &Context<Self>,
    ) -> AnyElement {
        let read = self
            .project_view
            .as_ref()
            .filter(|(shown, _)| shown == slug)
            .and_then(|(_, read)| read.as_ref());
        let title = match read {
            Some(Ok(data)) => data.title.clone(),
            _ => title_of(slug, cx),
        };
        let matched = match by {
            Matched::Path => "matched by path",
            Matched::Name => "matched by name",
        };
        let opened = slug.to_string();
        let header = v_flex()
            .px_2p5()
            .pt_2()
            .gap_0p5()
            .child(
                h_flex()
                    .justify_between()
                    .gap_2()
                    .child(Label::new(title).size(LabelSize::Large).truncate())
                    .child(
                        Button::new("rusty-project-open", "Open Page")
                            .end_icon(Icon::new(IconName::ArrowUpRight).size(IconSize::Small))
                            .label_size(LabelSize::Small)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.open_kept(&opened, window, cx);
                            })),
                    ),
            )
            .child(
                Label::new(format!("{slug} · {matched}"))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            );
        let also_lines = also.iter().enumerate().map(|(at, other)| {
            let opened = other.clone();
            ListItem::new(("rusty-project-also", at))
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .child(
                    Label::new(format!("{} also lists this folder.", title_of(other, cx)))
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.open(&opened, window, cx);
                }))
                .into_any_element()
        });
        let body = match read {
            None => muted_line(format!("Reading {slug}…")),
            Some(Err(error)) => error_line(error.to_string()),
            Some(Ok(data)) => Self::render_project_data(data, cx),
        };
        v_flex()
            .w_full()
            .gap_1()
            .child(header)
            .children(also_lines)
            .child(body)
            .into_any_element()
    }

    /// The project view's Tasks heading, with Open in Tasks when a group is joined (#658).
    fn tasks_header(
        label: SharedString,
        count: usize,
        list: Option<i64>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .w_full()
            .child(div().flex_1().min_w_0().child(section(label, count)))
            .children(list.map(|list| {
                div().pr_1().child(
                    IconButton::new("rusty-project-open-tasks", IconName::ListTodo)
                        .icon_size(IconSize::Small)
                        .tooltip(Tooltip::text("Open in Tasks"))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            super::tasks_tab::open_later(
                                this.workspace.clone(),
                                Some(list),
                                window,
                                cx,
                            );
                        })),
                )
            }))
    }

    fn render_project_data(data: &ProjectData, cx: &Context<Self>) -> AnyElement {
        let follow_ups = data
            .follow_ups
            .iter()
            .enumerate()
            .map(|(at, decision)| follow_up_row(at, decision, cx));
        let found: Vec<&TaskGroup> = match &data.groups {
            GroupJoin::Named { found, .. } => found.iter().collect(),
            GroupJoin::Like(group) => vec![group],
            GroupJoin::Nothing => Vec::new(),
        };
        let tasks_label = if found.is_empty() {
            SharedString::new_static("Tasks")
        } else {
            let names: Vec<&str> = found.iter().map(|group| group.name.as_str()).collect();
            format!("Tasks · {}", names.join(", ")).into()
        };
        let missing = match &data.groups {
            GroupJoin::Named { missing, .. } => missing.clone(),
            GroupJoin::Like(_) | GroupJoin::Nothing => Vec::new(),
        };
        let no_group = match (&data.groups, missing.first()) {
            (GroupJoin::Nothing, _) => Some("No task group for this project.".to_string()),
            (GroupJoin::Named { .. }, Some(name)) if found.is_empty() => {
                Some(format!("No task group named {name}."))
            }
            _ => None,
        };
        let tasks = data.tasks.iter().enumerate().map(|(at, task)| {
            ListItem::new(("rusty-project-task", at))
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .start_slot(
                    Icon::new(IconName::Circle)
                        .size(IconSize::Small)
                        .color(Color::Muted),
                )
                .child(Label::new(task.title.clone()).truncate())
                .into_any_element()
        });
        v_flex()
            .w_full()
            .gap_1()
            .when(!data.summary.is_empty(), |view| {
                view.child(
                    div().px_2p5().child(
                        Label::new(data.summary.clone())
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
                )
            })
            .child(section("Follow-ups due", data.follow_ups.len()))
            .when(data.follow_ups.is_empty(), |view| {
                view.child(muted_line("None due.".to_string()))
            })
            .children(follow_ups)
            .child(Self::tasks_header(
                tasks_label,
                data.tasks.len(),
                found.first().map(|group| group.id),
                cx,
            ))
            .map(|view| match no_group {
                Some(words) => view.child(
                    v_flex()
                        .px_2p5()
                        .py_1()
                        .gap_1()
                        .items_start()
                        .child(Label::new(words).size(LabelSize::Small).color(Color::Muted))
                        .child(
                            Button::new("rusty-project-link-group", "Link a Task Group")
                                .start_icon(Icon::new(IconName::Link).size(IconSize::Small))
                                .label_size(LabelSize::Small)
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(LinkTaskGroup.boxed_clone(), cx);
                                }),
                        ),
                ),
                None if data.tasks.is_empty() => {
                    view.child(muted_line("No open tasks.".to_string()))
                }
                None => view.children(tasks),
            })
            .into_any_element()
    }

    /// Opens `slug` in a kept tab with the focus, as Open Page does.
    fn open_kept(&self, slug: &str, window: &Window, cx: &mut App) {
        super::page::open_later(
            self.workspace.clone(),
            slug.to_string(),
            false,
            true,
            window,
            cx,
        );
    }

    fn link(&self, slug: &str, cx: &mut App) {
        if let Some(workspace) = self.workspace.upgrade() {
            project::link_page(&workspace, slug, cx);
        }
    }

    fn render_body(&self, window: &Window, cx: &Context<Self>) -> AnyElement {
        if !super::is_connected(cx) {
            let reason = super::unavailable(cx).unwrap_or_default();
            return muted_line(reason.to_string());
        }
        if let Some(results) = &self.results {
            return Self::render_results(results, cx);
        }
        let hint = (self.query(cx).is_empty() && self.search.focus_handle(cx).is_focused(window))
            .then(|| {
                muted_line(
                    "Narrow with tag: path: file: type:, quotes keep spaces, a leading - \
                     excludes."
                        .to_string(),
                )
            });
        let view = match (&self.slug, &self.knowledge) {
            (None, _) => self.render_project(cx),
            (Some(slug), None) => muted_line(format!("Reading {slug}…")),
            (Some(_), Some(Err(error))) => error_line(error.to_string()),
            (Some(_), Some(Ok(knowledge))) => Self::render_page(knowledge, cx),
        };
        v_flex()
            .w_full()
            .children(hint)
            .child(view)
            .into_any_element()
    }
}

/// A row of the page's own links: a page by its title, a click opening it; a missing one as
/// written, with Create.
fn outgoing_row(at: usize, link: &Outgoing, cx: &Context<KnowledgePanel>) -> AnyElement {
    match link {
        Outgoing::Page { slug, title } => {
            let slug = slug.clone();
            ListItem::new(("rusty-knowledge-link", at))
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .start_slot(
                    Icon::new(IconName::Link)
                        .size(IconSize::Small)
                        .color(Color::Muted),
                )
                .child(Label::new(title.clone()).truncate())
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.open(&slug, window, cx);
                }))
                .into_any_element()
        }
        Outgoing::Missing { target } => {
            let created = target.clone();
            ListItem::new(("rusty-knowledge-link", at))
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .start_slot(
                    Icon::new(IconName::Plus)
                        .size(IconSize::Small)
                        .color(Color::Muted),
                )
                .child(Label::new(target.clone()).color(Color::Muted).truncate())
                .end_slot(
                    Button::new(("rusty-knowledge-create", at), "Create")
                        .label_size(LabelSize::Small)
                        .on_click(cx.listener(move |_, _, window, cx| {
                            cx.stop_propagation();
                            KnowledgePanel::create(&created, window, cx);
                        })),
                )
                .into_any_element()
        }
    }
}

/// A follow-up due: the decision's title and its day, overdue in the warning colour; a click
/// opens the decision.
fn follow_up_row(
    at: usize,
    decision: &DecisionSummary,
    cx: &Context<KnowledgePanel>,
) -> AnyElement {
    let opened = decision.slug.clone();
    let by = decision.follow_up_by.clone().unwrap_or_default();
    ListItem::new(("rusty-project-follow-up", at))
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .start_slot(
            Icon::new(IconName::Clock)
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            v_flex()
                .min_w_0()
                .child(Label::new(decision.title.clone()).truncate())
                .child(
                    Label::new(format!("follow up by {by}"))
                        .size(LabelSize::Small)
                        .color(if decision.overdue {
                            Color::Warning
                        } else {
                            Color::Muted
                        }),
                ),
        )
        .on_click(cx.listener(move |this, _, window, cx| {
            this.open(&opened, window, cx);
        }))
        .into_any_element()
}

/// A project page's title from the cache, else its slug.
fn title_of(slug: &str, cx: &App) -> String {
    project::cached(slug, cx).map_or_else(|| slug.to_string(), |page| page.title)
}

/// The pages named like the project, each with Link; none is taken as the project's.
fn render_candidates(name: &str, slugs: &[String], cx: &Context<KnowledgePanel>) -> AnyElement {
    let rows = slugs.iter().enumerate().map(|(at, slug)| {
        let linked = slug.clone();
        let opened = slug.clone();
        ListItem::new(("rusty-project-candidate", at))
            .inset(true)
            .spacing(ListItemSpacing::Sparse)
            .child(
                v_flex()
                    .min_w_0()
                    .child(Label::new(title_of(slug, cx)).truncate())
                    .child(
                        Label::new(slug.clone())
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .end_slot(
                Button::new(("rusty-project-link", at), "Link")
                    .start_icon(Icon::new(IconName::Link).size(IconSize::Small))
                    .label_size(LabelSize::Small)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.link(&linked, cx);
                    })),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.open(&opened, window, cx);
            }))
            .into_any_element()
    });
    v_flex()
        .w_full()
        .gap_1()
        .child(muted_line(format!(
            "{} project pages are named {name}. Link one to this folder.",
            slugs.len()
        )))
        .children(rows)
        .into_any_element()
}

/// No page: the folders and the name looked for, and Link a Page.
fn render_unmatched(folders: &[std::path::PathBuf], names: &[String]) -> AnyElement {
    let folders: Vec<String> = folders
        .iter()
        .map(|folder| folder.to_string_lossy().into_owned())
        .collect();
    div()
        .px_2()
        .py_2()
        .child(
            Callout::new()
                .icon(IconName::Info)
                .title("No brain page for this project")
                .description(format!(
                    "No project page lists {} in its path, and none is named {}.",
                    folders.join(", "),
                    names.join(", ")
                ))
                .actions_slot(
                    Button::new("rusty-project-link-page", "Link a Page")
                        .start_icon(Icon::new(IconName::Link).size(IconSize::Small))
                        .label_size(LabelSize::Small)
                        .on_click(|_, window, cx| {
                            window.dispatch_action(LinkProjectPage.boxed_clone(), cx);
                        }),
                ),
        )
        .into_any_element()
}

/// The project page and its follow-ups from the first four answers, with the ids of the task
/// groups it joins; or the first failure.
fn project_reads(
    page: Result<String, String>,
    links: Result<String, String>,
    due: Result<String, String>,
    groups: Result<String, String>,
) -> Result<(ProjectData, Vec<i64>), String> {
    let read = read_from_answer(&page?)
        .map_err(|error| format!("{BRAIN_READ_PAGE}'s answer did not parse: {error}"))?
        .ok_or_else(|| "The project page is gone from the brain.".to_string())?;
    let links: PageLinks = serde_json::from_str(&links?)
        .map_err(|error| format!("{BRAIN_GET_LINKS}'s answer did not parse: {error}"))?;
    let due = due_from_answer(&due?)
        .map_err(|error| format!("{BRAIN_DUE}'s answer did not parse: {error}"))?;
    let groups = groups_from_answer(&groups?)
        .map_err(|error| format!("{LIST_TASK_GROUPS}'s answer did not parse: {error}"))?;
    let page = ProjectPage::from_read(&read);
    let groups = group_join(&page, &groups);
    let ids = match &groups {
        GroupJoin::Named { found, .. } => found.iter().map(|group| group.id).collect(),
        GroupJoin::Like(group) => vec![group.id],
        GroupJoin::Nothing => Vec::new(),
    };
    Ok((
        ProjectData {
            title: read.title.clone(),
            summary: summary(page.summary.as_deref(), &read.compiled_truth),
            follow_ups: due_for(&due, &links),
            groups,
            tasks: Vec::new(),
        },
        ids,
    ))
}

/// The open tasks from `list_tasks`' answers, in their groups' order; or the first failure.
fn open_tasks(answers: Vec<Result<String, String>>) -> Result<Vec<UserTask>, String> {
    let mut open = Vec::new();
    for answer in answers {
        let tasks = tasks_from_answer(&answer?)
            .map_err(|error| format!("{LIST_TASKS}'s answer did not parse: {error}"))?;
        open.extend(tasks.into_iter().filter(|task| !task.completed));
    }
    Ok(open)
}

/// A section's header with its count, as the `ui` crate's own example draws one.
fn section(label: impl Into<SharedString>, count: usize) -> impl IntoElement {
    ListSubHeader::new(label).inset(true).end_slot(
        Label::new(count.to_string())
            .size(LabelSize::Small)
            .color(Color::Muted)
            .into_any_element(),
    )
}

fn muted_line(text: String) -> AnyElement {
    div()
        .px_2p5()
        .py_2()
        .child(Label::new(text).size(LabelSize::Small).color(Color::Muted))
        .into_any_element()
}

fn error_line(text: String) -> AnyElement {
    div()
        .px_2p5()
        .py_2()
        .child(Label::new(text).size(LabelSize::Small).color(Color::Error))
        .into_any_element()
}

/// The page's knowledge from the three answers, or the first failure.
fn build(
    slug: &str,
    links: Result<String, String>,
    graph: Result<String, String>,
    tags: Result<String, String>,
) -> Result<PageKnowledge, String> {
    let links: PageLinks = serde_json::from_str(&links?)
        .map_err(|error| format!("{BRAIN_GET_LINKS}'s answer did not parse: {error}"))?;
    let graph: Graph = serde_json::from_str(&graph?)
        .map_err(|error| format!("{BRAIN_GRAPH}'s answer did not parse: {error}"))?;
    let tags: Vec<TagCount> = serde_json::from_str(&tags?)
        .map_err(|error| format!("{BRAIN_TAGS}'s answer did not parse: {error}"))?;
    Ok(page_knowledge(slug, links, &graph, &tags))
}

impl EventEmitter<PanelEvent> for KnowledgePanel {}

impl Focusable for KnowledgePanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for KnowledgePanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !super::is_on(cx) {
            self.close_while_off(window, cx);
            return div().into_any_element();
        }
        let body = self.render_body(window, cx);
        v_flex()
            .id("rusty-knowledge-panel")
            .key_context("KnowledgePanel menu")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &menu::SelectNext, _, cx| this.select(true, cx)))
            .on_action(cx.listener(|this, _: &menu::SelectPrevious, _, cx| {
                this.select(false, cx);
            }))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(
                h_flex().px_2().py_1p5().child(
                    Label::new("KNOWLEDGE")
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                ),
            )
            .child(self.render_field(cx))
            .child(
                div()
                    .id("rusty-knowledge-body")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(body),
            )
            .into_any_element()
    }
}

impl Panel for KnowledgePanel {
    fn persistent_name() -> &'static str {
        "MarleyKnowledgePanel"
    }

    fn set_active(&mut self, active: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.shown_in_dock = if active {
            InDock::Shown
        } else {
            InDock::Hidden
        };
        if active {
            // The dock opens inside the workspace's update; the join reads the workspace.
            cx.defer_in(window, |this, _, cx| this.refresh_join(cx));
        }
    }

    fn panel_key() -> &'static str {
        "MarleyKnowledgePanel"
    }

    fn activation_focus_handle(&self, cx: &App) -> FocusHandle {
        self.search.focus_handle(cx)
    }

    fn position(&self, _window: &Window, _cx: &App) -> DockPosition {
        DockPosition::Right
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        matches!(position, DockPosition::Right)
    }

    fn set_position(
        &mut self,
        _position: DockPosition,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn default_size(&self, _window: &Window, _cx: &App) -> Pixels {
        DEFAULT_WIDTH
    }

    // No button while Rusty is off, as the Agent Panel shows none while AI is off; the dock's
    // buttons redraw on every settings change.
    fn icon(&self, _window: &Window, cx: &App) -> Option<IconName> {
        super::is_on(cx).then_some(IconName::Book)
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("Knowledge")
    }

    fn toggle_action(&self) -> Box<dyn Action> {
        Box::new(ToggleKnowledgePanel)
    }

    fn activation_priority(&self) -> u32 {
        ACTIVATION_PRIORITY
    }

    fn enabled(&self, cx: &App) -> bool {
        super::is_on(cx)
    }
}
