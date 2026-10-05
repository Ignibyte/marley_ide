//! `rusty: open page` (#654): a picker over every page of Rusty's brain.
//!
//! Zed's `Picker` in the workspace's modal layer, over one `brain_list_pages` read per open. The
//! query is matched in Marley with Zed's `fuzzy_nucleo`, over titles and over slugs, so nothing
//! typed goes to Rusty. On an empty query the pages Marley opened most recently come first, kept
//! in Zed's key-value store so the order holds after a restart. A query that names no page ends
//! the list with a row that makes it through `brain_new_page`.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use db::kvp::KeyValueStore;
use fuzzy_nucleo::{Case, LengthPenalty, StringMatch, StringMatchCandidate};
use gpui::{
    App, AppContext as _, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable,
    Global, SharedString, Task, WeakEntity, Window,
};
use marley_rusty::page::NewPage;
use marley_rusty::switcher::{self, BRAIN_LIST_PAGES, Matched, PageSummary, RecentPages};
use picker::{Picker, PickerDelegate};
use serde_json::json;
use ui::{HighlightedLabel, ListItem, ListItemSpacing, prelude::*};
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{ModalView, Toast, Workspace};

use super::page::{OpenPage, PageView};

/// The store's scope and key for the recently opened pages.
const RECENT_SCOPE: &str = "marley-rusty-recent-pages";
const RECENT_KEY: &str = "pages";

/// The pages Marley opened most recently, newest first.
#[derive(Default)]
struct Recent(RecentPages);

impl Global for Recent {}

/// Reads the recently opened pages Zed's key-value store kept; `rusty::init` calls it once.
pub(super) fn init(cx: &mut App) {
    let stored = KeyValueStore::global(cx)
        .scoped(RECENT_SCOPE)
        .read(RECENT_KEY)
        .log_err()
        .flatten();
    cx.set_global(Recent(
        stored
            .map(|text| RecentPages::from_json(&text))
            .unwrap_or_default(),
    ));
}

/// Records that page `slug` was opened, by any opener or by a Page tab's link, Back or Forward.
pub(crate) fn opened(slug: &str, cx: &mut App) {
    let recent = &mut cx.default_global::<Recent>().0;
    if recent.slugs().first().is_some_and(|first| first == slug) {
        return;
    }
    recent.visit(slug);
    let stored = recent.to_json();
    let store = KeyValueStore::global(cx);
    cx.background_spawn(async move {
        store
            .scoped(RECENT_SCOPE)
            .write(RECENT_KEY.to_string(), stored)
            .await
            .log_err();
    })
    .detach();
}

/// Opens the picker, or closes it when it is open; the caller checked that Rusty is connected.
pub(super) fn toggle(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    // From a terminal the key is Marley's, not the program's (#563).
    if let Some(view) = crate::blocks::focused_terminal(workspace, window, cx) {
        let workspace_entity = cx.entity();
        crate::shortcut_note::taken(
            &OpenPage {
                slug: None,
                preview: false,
            },
            "opened Rusty's page picker",
            &view.focus_handle(cx),
            &workspace_entity,
            window,
            cx,
        );
    }
    let active = workspace
        .active_item(cx)
        .and_then(|item| item.downcast::<PageView>())
        .map(|view| view.read(cx).slug().to_string());
    let handle = workspace.weak_handle();
    workspace.toggle_modal(window, cx, |window, cx| {
        PagePicker::new(handle, active, window, cx)
    });
}

/// The picker's modal.
struct PagePicker {
    picker: Entity<Picker<PagePickerDelegate>>,
}

impl PagePicker {
    fn new(
        workspace: WeakEntity<Workspace>,
        active: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let delegate = PagePickerDelegate {
            modal: cx.entity().downgrade(),
            workspace,
            active,
            list: List::Reading,
            rows: Vec::new(),
            separator_after: None,
            selected: 0,
            creating: false,
        };
        let picker = cx.new(|cx| Picker::uniform_list(delegate, window, cx));
        let reading = super::call_tool(
            BRAIN_LIST_PAGES,
            json!({ "limit": switcher::LIST_LIMIT }),
            cx,
        );
        let weak_picker = picker.downgrade();
        cx.spawn_in(window, async move |_, cx| {
            let list = match reading.await {
                Ok(text) => {
                    cx.background_spawn(futures::future::lazy(move |_| {
                        switcher::parse_page_list(&text).map_err(|error| {
                            format!("{BRAIN_LIST_PAGES}'s answer did not parse: {error}")
                        })
                    }))
                    .await
                }
                Err(error) => Err(error),
            };
            weak_picker
                .update_in(cx, |picker, window, cx| {
                    picker.delegate.list = match list {
                        Ok(pages) => List::Read(pages.into()),
                        Err(error) => List::Failed(
                            error.lines().next().unwrap_or_default().to_string().into(),
                        ),
                    };
                    picker.refresh(window, cx);
                })
                .log_err();
        })
        .detach();
        Self { picker }
    }
}

impl ModalView for PagePicker {}

impl EventEmitter<DismissEvent> for PagePicker {}

impl Focusable for PagePicker {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.picker.focus_handle(cx)
    }
}

impl Render for PagePicker {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("RustyPagePicker")
            .w(rems(34.))
            .child(self.picker.clone())
    }
}

/// The page list, as far as the read got.
enum List {
    Reading,
    Read(Arc<[PageSummary]>),
    Failed(SharedString),
}

/// One row: a page with the bytes its query lit, or the page the query would make.
enum Row {
    Page {
        page: usize,
        title_positions: Vec<usize>,
        slug_positions: Vec<usize>,
    },
    Create(NewPage),
}

struct PagePickerDelegate {
    modal: WeakEntity<PagePicker>,
    workspace: WeakEntity<Workspace>,
    /// The page the active tab showed when the picker opened.
    active: Option<String>,
    list: List,
    rows: Vec<Row>,
    separator_after: Option<usize>,
    selected: usize,
    /// Whether a create waits on Rusty's answer.
    creating: bool,
}

impl PagePickerDelegate {
    const fn pages(&self) -> Option<&Arc<[PageSummary]>> {
        match &self.list {
            List::Read(pages) => Some(pages),
            List::Reading | List::Failed(_) => None,
        }
    }

    fn toast(&self, id: &'static str, message: String, cx: &mut App) {
        self.workspace
            .update(cx, |workspace, cx| {
                workspace.show_toast(
                    Toast::new(NotificationId::composite::<PagePicker>(id), message),
                    cx,
                );
            })
            .log_err();
    }

    /// Makes the page the create row names; the picker stays open until Rusty answers.
    fn create(&mut self, new_page: NewPage, window: &Window, cx: &mut Context<Picker<Self>>) {
        if self.creating {
            return;
        }
        self.creating = true;
        cx.notify();
        let creating = super::page::create(&new_page, cx);
        cx.spawn_in(window, async move |picker, cx| {
            let made = creating.await;
            picker
                .update_in(cx, |picker, window, cx| {
                    let delegate = &mut picker.delegate;
                    delegate.creating = false;
                    match made {
                        Ok(slug) => {
                            if slug != new_page.path {
                                delegate.toast(
                                    "renamed",
                                    format!("Rusty made the page as {slug}."),
                                    cx,
                                );
                            }
                            delegate.dismissed(window, cx);
                            super::page::open_later(
                                delegate.workspace.clone(),
                                slug,
                                false,
                                true,
                                window,
                                cx,
                            );
                        }
                        Err(message) => {
                            delegate.toast("refused", message, cx);
                            cx.notify();
                        }
                    }
                })
                .log_err();
        })
        .detach();
    }
}

/// `matched` as the pure merge takes it.
fn matched(found: StringMatch) -> Matched {
    Matched {
        page: found.candidate_id,
        score: found.score,
        positions: found.positions,
    }
}

impl PickerDelegate for PagePickerDelegate {
    type ListItem = ListItem;

    fn name() -> &'static str {
        "rusty page picker"
    }

    fn match_count(&self) -> usize {
        self.rows.len()
    }

    fn selected_index(&self) -> usize {
        self.selected
    }

    fn set_selected_index(&mut self, index: usize, _: &mut Window, _: &mut Context<Picker<Self>>) {
        self.selected = index;
    }

    fn separators_after_indices(&self) -> Vec<usize> {
        self.separator_after.into_iter().collect()
    }

    fn placeholder_text(&self, _: &mut Window, _: &mut App) -> Arc<str> {
        "Open a page of Rusty's brain, or name a new one…".into()
    }

    fn no_matches_text(&self, _: &mut Window, _: &mut App) -> Option<SharedString> {
        Some(match &self.list {
            List::Reading => SharedString::new_static("Reading the brain…"),
            List::Failed(error) => error.clone(),
            List::Read(_) => SharedString::new_static("No pages"),
        })
    }

    fn update_matches(
        &mut self,
        query: String,
        window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        let Some(pages) = self.pages().cloned() else {
            self.rows.clear();
            self.separator_after = None;
            self.selected = 0;
            return Task::ready(());
        };
        let recent = cx
            .try_global::<Recent>()
            .map(|recent| recent.0.clone())
            .unwrap_or_default();
        if query.trim().is_empty() {
            let order = switcher::empty_order(&pages, &recent, self.active.as_deref());
            self.rows = order
                .rows
                .into_iter()
                .map(|page| Row::Page {
                    page,
                    title_positions: Vec::new(),
                    slug_positions: Vec::new(),
                })
                .collect();
            self.separator_after = order.separator_after;
            self.selected = order.selected;
            return Task::ready(());
        }
        let titles: Vec<StringMatchCandidate> = pages
            .iter()
            .enumerate()
            .map(|(index, page)| StringMatchCandidate::new(index, page.shown_title().to_string()))
            .collect();
        let slugs: Vec<StringMatchCandidate> = pages
            .iter()
            .enumerate()
            .map(|(index, page)| StringMatchCandidate::new(index, page.slug.clone()))
            .collect();
        let executor = cx.background_executor().clone();
        cx.spawn_in(window, async move |picker, cx| {
            let cancelled = AtomicBool::new(false);
            let case = Case::smart_if_uppercase_in(&query);
            let by_title = fuzzy_nucleo::match_strings_async(
                &titles,
                &query,
                case,
                LengthPenalty::On,
                switcher::MATCH_CAP,
                &cancelled,
                executor.clone(),
            )
            .await;
            let by_slug = fuzzy_nucleo::match_strings_async(
                &slugs,
                &query,
                case,
                LengthPenalty::On,
                switcher::MATCH_CAP,
                &cancelled,
                executor,
            )
            .await;
            let hits = switcher::merge(
                by_title.into_iter().map(matched).collect(),
                by_slug.into_iter().map(matched).collect(),
                &pages,
                &recent,
                switcher::MATCH_CAP,
            );
            let create = switcher::create_target(&query, &pages);
            picker
                .update(cx, |picker, cx| {
                    let delegate = &mut picker.delegate;
                    delegate.rows = hits
                        .into_iter()
                        .map(|hit| Row::Page {
                            page: hit.page,
                            title_positions: hit.title_positions,
                            slug_positions: hit.slug_positions,
                        })
                        .chain(create.map(Row::Create))
                        .collect();
                    delegate.separator_after = None;
                    delegate.selected = 0;
                    cx.notify();
                })
                .log_err();
        })
    }

    fn confirm(&mut self, _: bool, window: &mut Window, cx: &mut Context<Picker<Self>>) {
        match self.rows.get(self.selected) {
            Some(Row::Page { page, .. }) => {
                let Some(slug) = self
                    .pages()
                    .and_then(|pages| pages.get(*page))
                    .map(|page| page.slug.clone())
                else {
                    return;
                };
                self.dismissed(window, cx);
                super::page::open_later(self.workspace.clone(), slug, false, true, window, cx);
            }
            Some(Row::Create(new_page)) => {
                let new_page = new_page.clone();
                self.create(new_page, window, cx);
            }
            None => {}
        }
    }

    fn dismissed(&mut self, _: &mut Window, cx: &mut Context<Picker<Self>>) {
        self.modal
            .update(cx, |_, cx| cx.emit(DismissEvent))
            .log_err();
    }

    fn render_match(
        &self,
        index: usize,
        selected: bool,
        _: &mut Window,
        _: &mut Context<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        let item = ListItem::new(index)
            .inset(true)
            .spacing(ListItemSpacing::Sparse)
            .toggle_state(selected);
        match self.rows.get(index)? {
            Row::Page {
                page,
                title_positions,
                slug_positions,
            } => {
                let page = self.pages()?.get(*page)?;
                Some(
                    item.start_slot(Icon::new(IconName::FileMarkdown).color(Color::Muted))
                        .child(
                            h_flex()
                                .gap_2()
                                .child(HighlightedLabel::new(
                                    page.shown_title().to_string(),
                                    title_positions.clone(),
                                ))
                                .child(
                                    HighlightedLabel::new(
                                        page.slug.clone(),
                                        slug_positions.clone(),
                                    )
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                                ),
                        ),
                )
            }
            Row::Create(new_page) => {
                let words = if self.creating {
                    format!("Creating {}…", new_page.path)
                } else {
                    format!("Create page: {}", new_page.path)
                };
                Some(
                    item.start_slot(Icon::new(IconName::Plus).color(Color::Muted))
                        .child(Label::new(words)),
                )
            }
        }
    }
}
