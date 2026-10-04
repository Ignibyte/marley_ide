//! The Knowledge panel (#646), in the right dock while Rusty is on.
//!
//! The active Page tab's tags with their page counts, the pages that link to it with the line each
//! link sits on, its own links in order (one to no page yet offered as a new page), and brain
//! search, sent on Enter.
//!
//! It is always added and hides itself while Rusty is off, as Zed's Agent Panel does while AI is
//! off: no dock button, a toggle that says why, and a dock that closes when it was showing the
//! panel. It follows the workspace's active item, as Zed's outline panel follows the editor.

use std::mem;

use editor::Editor;
use gpui::{
    Action, AnyElement, App, Context, Entity, EventEmitter, FocusHandle, Focusable, Pixels,
    SharedString, Subscription, WeakEntity, Window, actions, px,
};
use marley_rusty::knowledge::{
    BRAIN_GET_LINKS, BRAIN_GRAPH, BRAIN_TAGS, Graph, Outgoing, PageKnowledge, PageLinks,
    SEARCH_LIMIT, TagCount, page_knowledge, snippet,
};
use marley_rusty::vault::{self, BRAIN_NEW_PAGE, BRAIN_SEARCH, SearchHit, hits_from_answer};
use serde_json::json;
use ui::{
    Chip, HighlightedLabel, IconButtonShape, ListItem, ListItemSpacing, ListSubHeader, Tooltip,
    prelude::*,
};
use util::ResultExt as _;
use workspace::dock::{DockPosition, Panel, PanelEvent};
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use super::page::{PageEvent, PageView};

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
    _subscriptions: [Subscription; 4],
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
            // A change Rusty announces: the page and the query are read again.
            cx.observe_global_in::<super::Announced>(window, |this, _, cx| {
                this.load_page(cx);
                this.search_again(cx);
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
            _subscriptions: subscriptions,
        }
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
            .and_then(|item| item.downcast::<PageView>());
        let Some(page) = page else {
            self.page_tab = None;
            self.show(None, cx);
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
            (None, _) => {
                muted_line("Open a brain page to see its tags, backlinks and links.".to_string())
            }
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

/// A section's header with its count, as the `ui` crate's own example draws one.
fn section(label: &'static str, count: usize) -> impl IntoElement {
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
