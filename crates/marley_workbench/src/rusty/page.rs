//! A page of Rusty's brain in a center tab (#645).
//!
//! The tab shows the page's title, its properties and its body, drawn by Zed's own `markdown`
//! crate after `marley_rusty::page::page_markdown` turns each wikilink into a link with Rusty's
//! address. A link opens its page in the same tab, with Back and Forward of the tab's own; Edit
//! shows the page's file in a Zed editor inside the tab, and Read, Back and Forward save it first.
//! `open_later` is the one way to open a page: the rail's Brain view and `rusty::OpenPage` call
//! it, and it follows Zed's preview tabs as the project panel's clicks do.

use std::any::TypeId;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use editor::{Editor, EditorEvent};
use gpui::{
    Action, AnyElement, AnyEntity, App, Context, Entity, EntityId, EventEmitter, FocusHandle,
    Focusable, Global, Point, ScrollHandle, SharedString, Subscription, Task, TextStyleRefinement,
    WeakEntity, Window, actions,
};
use language::LanguageRegistry;
use markdown::{Markdown, MarkdownElement, MarkdownFont, MarkdownOptions, MarkdownStyle};
use marley_rusty::page::{
    BRAIN_REMOVE_PROPERTY, BRAIN_RENDER, Heading, NewPage, PageHistory, PageLink, Property,
    PropertyKind, RenderedPage, Visit, body_of, line_offset, outline_label, page_file_in,
    page_markdown,
};
use marley_rusty::project::BRAIN_SET_PROPERTY;
use marley_rusty::vault::{self, BRAIN_RENAME, RenameReport};
use project::Project;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use ui::{
    Checkbox, Chip, ContextMenu, Headline, HeadlineSize, ListItem, ListItemSpacing, PopoverMenu,
    ToggleState, Tooltip, prelude::*,
};
use util::ResultExt as _;
use util::markdown::generate_heading_slug;
use workspace::item::{Item, ItemEvent, SaveOptions, TabContentParams};
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use super::{inline_edit, properties};

/// How often, and how many times, the tab looks whether the Markdown parse ended.
const PARSE_CHECK: Duration = Duration::from_millis(50);
const PARSE_CHECKS: usize = 40;

/// Opens a page of Rusty's brain in a tab, or brings forward the tab that shows it.
///
/// With `preview`, in the pane's preview tab, as one click in the project panel opens a file.
/// Without a slug, opens the picker over every page (#654).
#[derive(Clone, Debug, PartialEq, Eq, JsonSchema, Action)]
#[action(namespace = rusty)]
pub struct OpenPage {
    /// The page's slug, such as `projects/marley`; none opens the page picker.
    pub slug: Option<String>,
    /// Whether the page opens in the pane's preview tab.
    pub preview: bool,
}

/// `OpenPage`'s fields as a keymap writes them. The action reads them through this struct: clippy
/// takes the `Action` derive's registration for a method it cannot check, which a derived
/// `Deserialize` beside it trips (`unsafe_derive_deserialize`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OpenPageFields {
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    preview: bool,
}

impl<'de> Deserialize<'de> for OpenPage {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let OpenPageFields { slug, preview } = OpenPageFields::deserialize(deserializer)?;
        Ok(Self { slug, preview })
    }
}

actions!(
    rusty,
    [
        /// Shows the page the Page tab showed before.
        #[derive(Eq)]
        PageBack,
        /// Shows the page the Page tab went back from.
        #[derive(Eq)]
        PageForward,
        /// Switches the Page tab between the rendered page and its file in an editor.
        #[derive(Eq)]
        TogglePageEdit,
        /// Shows or hides the Page tab's outline of the page's headings.
        #[derive(Eq)]
        TogglePageOutline,
        /// Adds the page in front as one of Rusty's bookmarks, or removes it when it is one.
        #[derive(Eq)]
        ToggleBookmark,
    ]
);

/// Every Page tab, so a rename made anywhere reaches them all (#656).
#[derive(Default)]
struct PageViews(Vec<WeakEntity<PageView>>);

impl Global for PageViews {}

/// What the open in-place editor edits (#656).
#[derive(Clone, Debug, PartialEq, Eq)]
enum EditTarget {
    Title,
    Name,
    Value { key: String, kind: PropertyKind },
    ListItem { key: String },
    NewKey { kind: PropertyKind },
}

/// The open in-place editor.
struct InlineEditing {
    target: EditTarget,
    editor: Entity<Editor>,
    /// What the text needs, after an Enter that could not be kept.
    error: Option<SharedString>,
    _blur: Subscription,
}

/// One write to the page, with the slug it was made on.
enum Write {
    Set { key: String, value: Value },
    Remove { key: String },
    Rename { to: String },
}

/// The tab's writes, sent one at a time; a change signal heard meanwhile waits for the last.
#[derive(Default)]
struct Writes {
    queue: VecDeque<(String, Write)>,
    busy: bool,
    read_after: bool,
}

/// Registers [`OpenPage`] on every workspace; `rusty::init` calls it once.
pub(super) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, action: &OpenPage, window, cx| {
            if let Some(reason) = super::unavailable(cx) {
                workspace.show_toast(
                    Toast::new(NotificationId::unique::<PageView>(), reason.to_string()),
                    cx,
                );
                return;
            }
            match &action.slug {
                Some(slug) => open_later(
                    cx.weak_entity(),
                    slug.clone(),
                    action.preview,
                    !action.preview,
                    window,
                    cx,
                ),
                None => super::page_picker::toggle(workspace, window, cx),
            }
        });
    })
    .detach();
}

/// Opens page `slug` in `workspace` once the update in progress ends: the tab showing it comes
/// forward, else a new tab opens, in the pane's preview tab with `preview`. `focus` moves the
/// keyboard into the tab.
pub(crate) fn open_later(
    workspace: WeakEntity<Workspace>,
    slug: String,
    preview: bool,
    focus: bool,
    window: &Window,
    cx: &mut App,
) {
    // The opener reads every Page tab, so it may not run inside one (AD-609), and a link's click
    // arrives inside its page's `Markdown` update; `in_rusty_group` defers it.
    super::in_rusty_group(workspace, window, cx, move |workspace, window, cx| {
        open(workspace, Visit::page(slug), preview, focus, window, cx);
    });
}

/// Opens page `slug` with `heading` at the top once the update in progress ends (#662): a
/// heading bookmark's open. The tab showing the page comes forward and scrolls there, else a
/// new kept tab opens there.
pub(crate) fn open_at_heading_later(
    workspace: WeakEntity<Workspace>,
    slug: String,
    heading: String,
    window: &Window,
    cx: &mut App,
) {
    let visit = Visit {
        slug,
        heading: Some(heading),
    };
    super::in_rusty_group(workspace, window, cx, move |workspace, window, cx| {
        open(workspace, visit, false, true, window, cx);
    });
}

fn open(
    workspace: &mut Workspace,
    visit: Visit,
    preview: bool,
    focus: bool,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    super::page_picker::opened(&visit.slug, cx);
    let shown = workspace
        .items_of_type::<PageView>(cx)
        .find(|view| view.read(cx).slug() == visit.slug);
    if let Some(shown) = shown {
        workspace.activate_item(&shown, true, focus, window, cx);
        if !preview && let Some(pane) = workspace.pane_for(&shown) {
            pane.update(cx, |pane, _| {
                pane.unpreview_item_if_preview(shown.entity_id());
            });
        }
        if let Some(heading) = visit.heading {
            shown.update(cx, |view, cx| view.go_to_heading(heading, window, cx));
        }
        return;
    }
    let languages = Arc::clone(workspace.project().read(cx).languages());
    let weak_workspace = cx.weak_entity();
    let view = cx.new(|cx| PageView::new(visit, weak_workspace, languages, window, cx));
    let pane = workspace.active_pane().clone();
    // Zed's own two steps (`Workspace::open_project_item`): the new item takes the preview's
    // place, then is added there.
    let destination = if preview {
        pane.update(cx, |pane, cx| {
            pane.replace_preview_item_id(view.entity_id(), window, cx)
        })
    } else {
        None
    };
    workspace.add_item(pane, Box::new(view), destination, true, focus, window, cx);
}

/// The page `item` shows: a Page tab's, or the page on the Brain tab's right (#678), which the
/// Knowledge panel, the Graph tab and the page picker read as the page in front.
pub(crate) fn page_in(item: &dyn workspace::ItemHandle, cx: &App) -> Option<Entity<PageView>> {
    item.downcast::<PageView>().or_else(|| {
        item.downcast::<super::brain_tab::BrainTab>()
            .and_then(|tab| tab.read(cx).page().cloned())
    })
}

/// Makes `new_page` through `brain_new_page`, and answers the slug Rusty gave it, or Rusty's
/// refusal (#654).
pub(crate) fn create(new_page: &NewPage, cx: &App) -> Task<Result<String, String>> {
    let asking = super::call_tool(vault::BRAIN_NEW_PAGE, new_page.arguments(), cx);
    cx.spawn(async move |_| {
        let answer = asking.await?;
        vault::slug_from_answer(&answer)
            .map_err(|error| format!("{}'s answer did not parse: {error}", vault::BRAIN_NEW_PAGE))
    })
}

/// What the tab has of its page.
enum Shown {
    Loading,
    Page(Box<RenderedPage>),
    /// `brain_render` answered `null`.
    Missing,
    Failed(SharedString),
}

/// Read, or Edit with the page's file in an editor.
enum Mode {
    Read,
    Edit {
        editor: Entity<Editor>,
        _events: Subscription,
    },
}

/// The tab's events, which Zed's pane reads as item events.
pub(crate) enum PageEvent {
    /// The title or the unsaved state changed.
    UpdateTab,
    /// The source was edited, which keeps a preview tab.
    Edited,
}

/// A brain page in a tab.
pub(crate) struct PageView {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    history: PageHistory,
    /// Each load's number: an answer for an older one is dropped.
    generation: u64,
    shown: Shown,
    markdown: Entity<Markdown>,
    scroll: ScrollHandle,
    mode: Mode,
    /// Whether Rusty was connected at its last change.
    connected: bool,
    /// Why the last Edit, save or open of the file failed.
    problem: Option<SharedString>,
    /// The redraw a heading's scroll waits for.
    settling: Option<Task<()>>,
    /// Whether the outline column shows in Read (#656).
    outline_shown: bool,
    /// The open in-place editor (#656).
    inline: Option<InlineEditing>,
    writes: Writes,
    _subscriptions: [Subscription; 3],
}

/// `brain_render`'s answer, and the body for Zed's renderer; run off the main thread.
fn read_page(text: &str) -> Result<Option<(RenderedPage, String)>, String> {
    let page = RenderedPage::from_answer(text)
        .map_err(|error| format!("{BRAIN_RENDER}'s answer did not parse: {error}"))?;
    Ok(page.map(|page| {
        let markdown = page_markdown(body_of(&page.raw), &page.slug, &page.links);
        (page, markdown)
    }))
}

impl PageView {
    pub(crate) fn new(
        first: Visit,
        workspace: WeakEntity<Workspace>,
        languages: Arc<LanguageRegistry>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let markdown = cx.new(|cx| {
            Markdown::new_with_options(
                SharedString::default(),
                Some(languages),
                None,
                MarkdownOptions {
                    parse_html: true,
                    render_mermaid_diagrams: true,
                    parse_heading_slugs: true,
                    ..MarkdownOptions::default()
                },
                cx,
            )
        });
        let subscriptions = [
            // A change Rusty announces: the page is read again while it shows Read, after the
            // tab's own writes have all answered (#656).
            cx.observe_global_in::<super::Announced>(window, |this, window, cx| {
                if this.writes.busy || !this.writes.queue.is_empty() {
                    this.writes.read_after = true;
                } else if matches!(this.mode, Mode::Read) {
                    this.load(window, cx);
                }
            }),
            cx.observe_global_in::<super::Rusty>(window, Self::rusty_changed),
            // The star follows Rusty's bookmarks (#662).
            cx.observe_global::<super::favourites::Bookmarks>(|_, cx| cx.notify()),
        ];
        super::favourites::ensure(cx);
        let mut this = Self {
            workspace,
            focus_handle: cx.focus_handle(),
            history: PageHistory::new(first),
            generation: 0,
            shown: Shown::Loading,
            markdown,
            scroll: ScrollHandle::new(),
            mode: Mode::Read,
            connected: super::is_connected(cx),
            problem: None,
            settling: None,
            outline_shown: true,
            inline: None,
            writes: Writes::default(),
            _subscriptions: subscriptions,
        };
        this.load(window, cx);
        let weak = cx.weak_entity();
        let views = &mut cx.default_global::<PageViews>().0;
        views.retain(|view| view.upgrade().is_some());
        views.push(weak);
        this
    }

    /// The slug of the page the tab shows now, which the opener matches (PR-claude-599).
    pub(crate) fn slug(&self) -> &str {
        &self.history.current().slug
    }

    const fn editor(&self) -> Option<&Entity<Editor>> {
        match &self.mode {
            Mode::Edit { editor, .. } => Some(editor),
            Mode::Read => None,
        }
    }

    /// The not-connected line follows the connection; a page that never loaded loads once it is
    /// back.
    fn rusty_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let connected = super::is_connected(cx);
        if connected == self.connected {
            return;
        }
        self.connected = connected;
        if connected && !matches!(self.shown, Shown::Page(_)) {
            self.load(window, cx);
        }
        cx.notify();
    }

    /// Reads the current page with `brain_render`; an answer that comes back after a newer
    /// navigation is dropped.
    fn load(&mut self, window: &Window, cx: &Context<Self>) {
        self.generation += 1;
        let generation = self.generation;
        let visit = self.history.current().clone();
        let asking = super::call_tool(BRAIN_RENDER, json!({ "slug": visit.slug }), cx);
        cx.spawn_in(window, async move |this, cx| {
            let read = match asking.await {
                Ok(text) => {
                    cx.background_spawn(futures::future::lazy(move |_| read_page(&text)))
                        .await
                }
                Err(error) => Err(error),
            };
            this.update_in(cx, |this, window, cx| {
                if this.generation == generation {
                    this.show(read, visit.heading, window, cx);
                }
            })
            .log_err();
        })
        .detach();
    }

    fn show(
        &mut self,
        read: Result<Option<(RenderedPage, String)>, String>,
        heading: Option<String>,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        match read {
            Ok(Some((page, markdown))) => {
                let scrolls = heading.is_some();
                // A heading the outline holds goes to the top, as the outline's click puts it
                // (#662); another, such as one a link names by its anchor, is scrolled into view.
                let top = heading
                    .as_deref()
                    .and_then(|heading| outline_line(&page, heading));
                self.markdown.update(cx, |view, cx| {
                    view.replace(markdown, cx);
                    if top.is_none()
                        && let Some(heading) = heading
                    {
                        view.scroll_to_heading_when_parsed(
                            generate_heading_slug(&heading).into(),
                            cx,
                        );
                    }
                });
                if let Some(line) = top {
                    self.scroll_to_heading_line(line, cx);
                }
                if scrolls {
                    self.redraw_after_parse(window, cx);
                }
                self.shown = Shown::Page(Box::new(page));
            }
            Ok(None) => self.shown = Shown::Missing,
            Err(error) => self.shown = Shown::Failed(error.into()),
        }
        cx.emit(PageEvent::UpdateTab);
        cx.notify();
    }

    /// Zed's Markdown element scrolls to a heading while it paints, after the frame's layout, so
    /// the new offset shows only on a later frame. Nothing else redraws a page at rest (Zed's
    /// preview has its editor's caret), so the tab asks for one once the parse has ended.
    fn redraw_after_parse(&mut self, window: &Window, cx: &Context<Self>) {
        self.settling = Some(cx.spawn_in(window, async move |this, cx| {
            for _ in 0..PARSE_CHECKS {
                cx.background_executor().timer(PARSE_CHECK).await;
                let parsing = this.read_with(cx, |this, cx| this.markdown.read(cx).is_parsing());
                match parsing {
                    Ok(true) => {}
                    Ok(false) => break,
                    Err(_) => return,
                }
            }
            // The parse's own frame sets the offset; this one draws it.
            cx.background_executor().timer(PARSE_CHECK).await;
            this.update(cx, |_, cx| cx.notify()).log_err();
        }));
    }

    /// Leaves Edit, saving unsaved edits first without formatting them, then runs `then`; a save
    /// that fails stays in Edit and says why.
    fn leave_edit_then(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        then: impl FnOnce(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) {
        let Some(editor) = self.editor().cloned() else {
            then(self, window, cx);
            return;
        };
        let workspace = self.workspace.upgrade();
        let saving = match workspace {
            Some(workspace) if editor.read(cx).is_dirty(cx) => {
                let project = workspace.read(cx).project().clone();
                // Prettier may format Markdown on save; a brain page Marley saves by itself is
                // written as typed.
                let options = SaveOptions {
                    format: false,
                    force_format: false,
                    autosave: true,
                };
                editor.update(cx, |editor, cx| editor.save(options, project, window, cx))
            }
            _ => Task::ready(Ok(())),
        };
        cx.spawn_in(window, async move |this, cx| {
            let saved = saving.await;
            this.update_in(cx, |this, window, cx| {
                match saved {
                    Ok(()) => {
                        this.mode = Mode::Read;
                        this.problem = None;
                        cx.emit(PageEvent::UpdateTab);
                        then(this, window, cx);
                    }
                    Err(error) => this.problem = Some(format!("Could not save: {error}").into()),
                }
                cx.notify();
            })
            .log_err();
        })
        .detach();
    }

    /// Shows `visit`, saving an edit first, and puts it in the tab's history.
    pub(crate) fn navigate(&mut self, visit: Visit, window: &mut Window, cx: &mut Context<Self>) {
        self.inline = None;
        self.leave_edit_then(window, cx, move |this, window, cx| {
            this.history.visit(visit);
            this.moved(cx);
            this.scroll.set_offset(Point::default());
            this.load(window, cx);
        });
    }

    fn back(&mut self, _: &PageBack, window: &mut Window, cx: &mut Context<Self>) {
        self.inline = None;
        self.leave_edit_then(window, cx, |this, window, cx| {
            if this.history.back() {
                this.moved(cx);
                this.scroll.set_offset(Point::default());
                this.load(window, cx);
            }
        });
    }

    fn forward(&mut self, _: &PageForward, window: &mut Window, cx: &mut Context<Self>) {
        self.inline = None;
        self.leave_edit_then(window, cx, |this, window, cx| {
            if this.history.forward() {
                this.moved(cx);
                this.scroll.set_offset(Point::default());
                this.load(window, cx);
            }
        });
    }

    /// A clicked address: a page opens here, a missing one says so, a heading scrolls, anything
    /// else goes to the system.
    fn follow(&mut self, address: &str, window: &mut Window, cx: &mut Context<Self>) {
        match PageLink::parse(address) {
            PageLink::Page { slug, heading }
            | PageLink::Local {
                target: slug,
                heading,
            } => self.navigate(Visit { slug, heading }, window, cx),
            PageLink::Missing { target } => self.create_linked(&target, window, cx),
            PageLink::Heading(heading) => {
                self.markdown.update(cx, |view, cx| {
                    view.scroll_to_heading(&generate_heading_slug(&heading), cx);
                });
            }
            PageLink::External(address) => cx.open_url(&address),
        }
    }

    /// Whether the title, the name and the values open an editor now: connected, in Read, with
    /// the page shown (#656).
    const fn can_edit(&self) -> bool {
        self.connected && matches!(self.mode, Mode::Read) && matches!(self.shown, Shown::Page(_))
    }

    fn page_mut(&mut self) -> Option<&mut RenderedPage> {
        match &mut self.shown {
            Shown::Page(page) => Some(page),
            _ => None,
        }
    }

    fn value_of(&self, key: &str) -> Option<&Value> {
        let Shown::Page(page) = &self.shown else {
            return None;
        };
        page.properties
            .iter()
            .find(|property| property.key == key)
            .map(|property| &property.value)
    }

    /// Shows `value` for `key` at once, the read after the write showing what Rusty stored.
    fn set_local(&mut self, key: &str, value: Value) {
        if let Some(page) = self.page_mut() {
            match page
                .properties
                .iter_mut()
                .find(|property| property.key == key)
            {
                Some(property) => property.value = value,
                None => page.properties.push(Property {
                    key: key.to_string(),
                    value,
                }),
            }
        }
    }

    /// Opens the in-place editor on `target`, holding `text`; one already open keeps its text.
    fn start_inline(
        &mut self,
        target: EditTarget,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_edit() {
            return;
        }
        self.commit_inline(false, window, cx);
        let editor = inline_edit::editor_for(text, window, cx);
        window.focus(&editor.focus_handle(cx), cx);
        // Leaving the editor keeps its text while Marley's window is active, as the project panel's
        // rename does.
        let blur = cx.on_blur(&editor.focus_handle(cx), window, |this, window, cx| {
            if window.is_window_active() {
                this.commit_inline(false, window, cx);
            }
        });
        self.inline = Some(InlineEditing {
            target,
            editor,
            error: None,
            _blur: blur,
        });
        cx.notify();
    }

    fn confirm_inline(&mut self, _: &menu::Confirm, window: &mut Window, cx: &mut Context<Self>) {
        self.commit_inline(true, window, cx);
    }

    fn cancel_inline(&mut self, _: &menu::Cancel, window: &mut Window, cx: &mut Context<Self>) {
        if self.inline.take().is_some() {
            window.focus(&self.focus_handle, cx);
            cx.notify();
        } else {
            cx.propagate();
        }
    }

    /// Keeps what the open editor holds: written when it changes something, the editor left open
    /// with what its text needs when it is not of the value's form.
    fn commit_inline(&mut self, entered: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(editing) = self.inline.take() else {
            return;
        };
        let typed = editing.editor.read(cx).text(cx);
        let kept = match &editing.target {
            EditTarget::Title => {
                self.commit_title(&typed, window, cx);
                Ok(())
            }
            EditTarget::Name => {
                self.commit_name(&typed, window, cx);
                Ok(())
            }
            EditTarget::Value { key, kind } => self.commit_value(key, *kind, &typed, window, cx),
            EditTarget::ListItem { key } => {
                self.commit_item(key, &typed, window, cx);
                Ok(())
            }
            EditTarget::NewKey { kind } => self.commit_key(*kind, &typed, window, cx),
        };
        match kept {
            Err(error) => {
                self.inline = Some(InlineEditing {
                    error: Some(error),
                    ..editing
                });
            }
            Ok(()) => {
                if let (true, EditTarget::ListItem { key }) = (entered, &editing.target) {
                    // The next item, as Rusty's app keeps its add field open.
                    self.start_inline(EditTarget::ListItem { key: key.clone() }, "", window, cx);
                } else if entered {
                    window.focus(&self.focus_handle, cx);
                }
            }
        }
        cx.notify();
    }

    fn commit_title(&mut self, typed: &str, window: &Window, cx: &mut Context<Self>) {
        let title = typed.trim().to_string();
        let Some(page) = self.page_mut() else {
            return;
        };
        if title.is_empty() || title == page.title {
            return;
        }
        page.title.clone_from(&title);
        self.set_local("title", Value::String(title.clone()));
        cx.emit(PageEvent::UpdateTab);
        self.queue(
            Write::Set {
                key: "title".to_string(),
                value: Value::String(title),
            },
            window,
            cx,
        );
    }

    fn commit_name(&mut self, typed: &str, window: &Window, cx: &mut Context<Self>) {
        if !matches!(self.mode, Mode::Read) {
            return;
        }
        if let Some(to) = vault::rename_target(self.slug(), typed) {
            self.queue(Write::Rename { to }, window, cx);
        }
    }

    fn commit_value(
        &mut self,
        key: &str,
        kind: PropertyKind,
        typed: &str,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Result<(), SharedString> {
        let value = kind.parse(typed).map_err(SharedString::new_static)?;
        let unchanged = match self.value_of(key) {
            Some(Value::Null) | None => value == Value::String(String::new()),
            Some(current) => *current == value,
        };
        if !unchanged {
            self.set_local(key, value.clone());
            self.queue(
                Write::Set {
                    key: key.to_string(),
                    value,
                },
                window,
                cx,
            );
        }
        Ok(())
    }

    fn commit_item(&mut self, key: &str, typed: &str, window: &Window, cx: &mut Context<Self>) {
        let item = typed.trim();
        let Some(Value::Array(items)) = self.value_of(key) else {
            return;
        };
        if item.is_empty() || items.iter().any(|kept| kept.as_str() == Some(item)) {
            return;
        }
        let mut items = items.clone();
        items.push(Value::String(item.to_string()));
        self.set_local(key, Value::Array(items.clone()));
        self.queue(
            Write::Set {
                key: key.to_string(),
                value: Value::Array(items),
            },
            window,
            cx,
        );
    }

    fn commit_key(
        &mut self,
        kind: PropertyKind,
        typed: &str,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Result<(), SharedString> {
        let key = typed.trim();
        if key.is_empty() {
            return Ok(());
        }
        if self.value_of(key).is_some() {
            return Err(format!("A property named {key} exists").into());
        }
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let value = kind.empty_value(&today);
        self.set_local(key, value.clone());
        self.queue(
            Write::Set {
                key: key.to_string(),
                value,
            },
            window,
            cx,
        );
        Ok(())
    }

    fn toggle_checkbox(&mut self, key: &str, window: &Window, cx: &mut Context<Self>) {
        let Some(Value::Bool(checked)) = self.value_of(key) else {
            return;
        };
        let value = Value::Bool(!checked);
        self.set_local(key, value.clone());
        self.queue(
            Write::Set {
                key: key.to_string(),
                value,
            },
            window,
            cx,
        );
    }

    fn remove_item(&mut self, key: &str, at: usize, window: &Window, cx: &mut Context<Self>) {
        let Some(Value::Array(items)) = self.value_of(key) else {
            return;
        };
        let mut items = items.clone();
        if at >= items.len() {
            return;
        }
        items.remove(at);
        self.set_local(key, Value::Array(items.clone()));
        self.queue(
            Write::Set {
                key: key.to_string(),
                value: Value::Array(items),
            },
            window,
            cx,
        );
    }

    fn remove_property(&mut self, key: &str, window: &Window, cx: &mut Context<Self>) {
        if let Some(page) = self.page_mut() {
            page.properties.retain(|property| property.key != key);
        }
        self.queue(
            Write::Remove {
                key: key.to_string(),
            },
            window,
            cx,
        );
    }

    /// Queues a write for the page shown now, and sends it when the ones before it have answered.
    fn queue(&mut self, write: Write, window: &Window, cx: &mut Context<Self>) {
        let slug = self.slug().to_string();
        self.writes.queue.push_back((slug, write));
        self.pump(window, cx);
        cx.notify();
    }

    /// Sends the next write; with none left, reads the page once if a write or a change signal
    /// asked for it (D8).
    fn pump(&mut self, window: &Window, cx: &Context<Self>) {
        if self.writes.busy {
            return;
        }
        let Some((slug, write)) = self.writes.queue.pop_front() else {
            if std::mem::take(&mut self.writes.read_after) && matches!(self.mode, Mode::Read) {
                self.load(window, cx);
            }
            return;
        };
        self.writes.busy = true;
        let renaming = matches!(write, Write::Rename { .. });
        let (tool, arguments) = match write {
            Write::Set { key, value } => (
                BRAIN_SET_PROPERTY,
                json!({ "slug": slug, "key": key, "value": value }),
            ),
            Write::Remove { key } => (BRAIN_REMOVE_PROPERTY, json!({ "slug": slug, "key": key })),
            Write::Rename { to } => (BRAIN_RENAME, json!({ "from": slug, "to": to })),
        };
        let asking = super::call_tool(tool, arguments, cx);
        cx.spawn_in(window, async move |this, cx| {
            let answer = asking.await;
            this.update_in(cx, |this, window, cx| {
                this.writes.busy = false;
                this.writes.read_after = true;
                match answer {
                    Ok(text) if renaming => match RenameReport::from_answer(&text) {
                        Ok(report) => {
                            this.history.rename(&report.from, &report.to);
                            cx.emit(PageEvent::UpdateTab);
                            this.toast(renamed_words(&report), cx);
                            follow_rename(&report, window, cx);
                        }
                        Err(error) => this.toast(
                            format!("{BRAIN_RENAME}'s answer did not parse: {error}"),
                            cx,
                        ),
                    },
                    Ok(_) => {}
                    Err(error) => this.toast(error, cx),
                }
                this.pump(window, cx);
                cx.notify();
            })
            .log_err();
        })
        .detach();
    }

    /// Rusty renamed `from` to `to`: the page shown and the history follow, and the page is read
    /// again when it was the one shown (#656).
    fn renamed(&mut self, from: &str, to: &str, window: &Window, cx: &mut Context<Self>) {
        let before = self.slug().to_string();
        self.history.rename(from, to);
        if self.slug() != before {
            cx.emit(PageEvent::UpdateTab);
            self.load(window, cx);
        }
        cx.notify();
    }

    /// Brings `heading` to the top: in the page shown when its outline holds it, else by a visit to
    /// the page at that heading (#662).
    fn go_to_heading(&mut self, heading: String, window: &mut Window, cx: &mut Context<Self>) {
        let line = match &self.shown {
            Shown::Page(page) if page.slug == self.slug() => outline_line(page, &heading),
            _ => None,
        };
        if let Some(line) = line {
            self.scroll_to_heading_line(line, cx);
            cx.notify();
        } else {
            let visit = Visit {
                slug: self.slug().to_string(),
                heading: Some(heading),
            };
            self.navigate(visit, window, cx);
        }
    }

    /// The star and `rusty: toggle bookmark` (#662).
    fn toggle_bookmark(&mut self, _: &ToggleBookmark, _: &mut Window, cx: &mut Context<Self>) {
        super::favourites::toggle_page(self.slug(), self.workspace.clone(), cx);
    }

    fn toggle_outline(&mut self, _: &TogglePageOutline, _: &mut Window, cx: &mut Context<Self>) {
        self.outline_shown = !self.outline_shown;
        cx.notify();
    }

    /// Brings the heading on line `line` of the body to the top of the view (D2, D3): the line's
    /// text after its marks, in the text Zed's renderer parses.
    fn scroll_to_heading_line(&self, line: usize, cx: &mut Context<Self>) {
        let source = self.markdown.read(cx).source().clone();
        let Some(start) = line_offset(&source, line) else {
            return;
        };
        let rest = source.get(start..).unwrap_or_default();
        let marks = rest.len() - rest.trim_start_matches(['#', ' ', '\t']).len();
        self.markdown.update(cx, |markdown, cx| {
            markdown.request_autoscroll_to_top(start + marks, cx);
        });
    }

    /// The tab moved to another page: it counts as opened for the page picker (#654).
    fn moved(&self, cx: &mut App) {
        let slug = self.slug().to_string();
        super::page_picker::opened(&slug, cx);
    }

    /// An unresolved link's page made through `brain_new_page` and shown here; Rusty's refusal in
    /// a toast, the tab as it was (#654).
    fn create_linked(&self, target: &str, window: &Window, cx: &mut Context<Self>) {
        let Some(new_page) = NewPage::from_target(target) else {
            self.toast(format!("{target} names no page Rusty can make."), cx);
            return;
        };
        let creating = create(&new_page, cx);
        cx.spawn_in(window, async move |this, cx| {
            let made = creating.await;
            this.update_in(cx, |this, window, cx| match made {
                Ok(slug) => this.navigate(Visit::page(slug), window, cx),
                Err(message) => this.toast(message, cx),
            })
            .log_err();
        })
        .detach();
    }

    fn toast(&self, message: String, cx: &mut App) {
        self.workspace
            .update(cx, |workspace, cx| {
                workspace.show_toast(Toast::new(NotificationId::unique::<Self>(), message), cx);
            })
            .log_err();
    }

    /// Edit: the page's file in an editor in the tab. Read: the editor saved and dropped, the page
    /// read again.
    fn toggle_edit(&mut self, _: &TogglePageEdit, window: &mut Window, cx: &mut Context<Self>) {
        if self.editor().is_some() {
            self.leave_edit_then(window, cx, |this, window, cx| this.load(window, cx));
            return;
        }
        let Shown::Page(page) = &self.shown else {
            return;
        };
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        let project = workspace.read(cx).project().clone();
        if !project.read(cx).is_local() {
            self.problem = Some(SharedString::new_static(
                "Edit needs a local project: Rusty's vault is on this machine.",
            ));
            cx.notify();
            return;
        }
        // Rusty gives the file since its TICKET-042; before it, the vault's folder and the slug.
        let file = if page.file.is_empty() {
            page_file_in(&super::vault_folder(cx), &page.slug)
        } else {
            Some(PathBuf::from(&page.file))
        };
        let Some(file) = file else {
            self.problem = Some(format!("{} is not a page Marley can open.", page.slug).into());
            cx.notify();
            return;
        };
        let opening = project.update(cx, |project, cx| project.open_local_buffer(file, cx));
        cx.spawn_in(window, async move |this, cx| {
            let buffer = opening.await;
            this.update_in(cx, |this, window, cx| {
                match buffer {
                    Ok(buffer) => {
                        let editor =
                            cx.new(|cx| Editor::for_buffer(buffer, Some(project), window, cx));
                        let events =
                            cx.subscribe(&editor, |_, _, event: &EditorEvent, cx| match event {
                                EditorEvent::BufferEdited => cx.emit(PageEvent::Edited),
                                EditorEvent::DirtyChanged
                                | EditorEvent::Saved
                                | EditorEvent::TitleChanged => cx.emit(PageEvent::UpdateTab),
                                _ => {}
                            });
                        window.focus(&editor.focus_handle(cx), cx);
                        this.mode = Mode::Edit {
                            editor,
                            _events: events,
                        };
                        this.problem = None;
                        cx.emit(PageEvent::UpdateTab);
                    }
                    Err(error) => {
                        this.problem = Some(format!("Could not open the file: {error}").into());
                    }
                }
                cx.notify();
            })
            .log_err();
        })
        .detach();
    }

    fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        let slug = self.slug();
        let folder = vault::folder_of(slug);
        let editing = self.editor().is_some();
        h_flex()
            .w_full()
            .flex_none()
            .gap_1()
            .px_2()
            .py_1()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                IconButton::new("rusty-page-back", IconName::ArrowLeft)
                    .icon_size(IconSize::Small)
                    .disabled(!self.history.can_back())
                    .tooltip(|_, cx| Tooltip::for_action("Back", &PageBack, cx))
                    .on_click(cx.listener(|this, _, window, cx| this.back(&PageBack, window, cx))),
            )
            .child(
                IconButton::new("rusty-page-forward", IconName::ArrowRight)
                    .icon_size(IconSize::Small)
                    .disabled(!self.history.can_forward())
                    .tooltip(|_, cx| Tooltip::for_action("Forward", &PageForward, cx))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.forward(&PageForward, window, cx);
                    })),
            )
            .child(self.render_outline_button(cx))
            .child(
                h_flex()
                    .min_w_0()
                    .flex_1()
                    .gap_1()
                    .pl_2()
                    .when(!folder.is_empty(), |path| {
                        path.child(
                            Label::new(format!("{} /", folder.replace('/', " / ")))
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        )
                    })
                    .child(self.render_name(cx)),
            )
            .child(self.render_star_button(cx))
            .child(
                Button::new(
                    "rusty-page-edit",
                    if editing {
                        SharedString::new_static("Read")
                    } else {
                        SharedString::new_static("Edit")
                    },
                )
                .start_icon(Icon::new(IconName::Pencil).size(IconSize::Small))
                .toggle_state(editing)
                .disabled(!matches!(self.shown, Shown::Page(_)))
                .tooltip(|_, cx| Tooltip::for_action("Edit or Read", &TogglePageEdit, cx))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.toggle_edit(&TogglePageEdit, window, cx);
                })),
            )
    }

    /// The star: filled while the page is one of Rusty's bookmarks (#662).
    fn render_star_button(&self, cx: &Context<Self>) -> impl IntoElement {
        let starred = super::favourites::is_page(self.slug(), cx);
        // An id of its own for each state drops a shown tooltip on a click, which would otherwise
        // keep the words it was built with.
        let (id, icon) = if starred {
            ("rusty-page-starred", IconName::StarFilled)
        } else {
            ("rusty-page-star", IconName::Star)
        };
        IconButton::new(id, icon)
            .icon_size(IconSize::Small)
            .icon_color(if starred { Color::Accent } else { Color::Muted })
            .disabled(!matches!(self.shown, Shown::Page(_)))
            .tooltip(move |_, cx| {
                Tooltip::for_action(
                    if starred {
                        "Remove from Favourites"
                    } else {
                        "Add to Favourites"
                    },
                    &ToggleBookmark,
                    cx,
                )
            })
            .on_click(cx.listener(|this, _, window, cx| {
                this.toggle_bookmark(&ToggleBookmark, window, cx);
            }))
    }

    fn render_outline_button(&self, cx: &Context<Self>) -> impl IntoElement {
        let headings = matches!(&self.shown, Shown::Page(page) if !page.outline.is_empty());
        let reading = matches!(self.mode, Mode::Read);
        IconButton::new("rusty-page-outline", IconName::ListTree)
            .icon_size(IconSize::Small)
            .toggle_state(self.outline_shown && headings && reading)
            .disabled(!headings || !reading)
            .tooltip(move |_, cx| {
                if headings {
                    Tooltip::for_action("Outline", &TogglePageOutline, cx)
                } else {
                    Tooltip::simple("No headings", cx)
                }
            })
            .on_click(cx.listener(|this, _, window, cx| {
                this.toggle_outline(&TogglePageOutline, window, cx);
            }))
    }

    /// The open editor for `target`, in Zed's `menu` context so Enter and Escape reach the tab.
    fn inline_field(&self, target: &EditTarget, cx: &Context<Self>) -> Option<AnyElement> {
        let editing = self
            .inline
            .as_ref()
            .filter(|editing| editing.target == *target)?;
        Some(
            div()
                .key_context("menu")
                .w_full()
                .on_action(cx.listener(Self::confirm_inline))
                .on_action(cx.listener(Self::cancel_inline))
                .child(inline_edit::field(
                    &editing.editor,
                    editing.error.as_ref(),
                    cx,
                ))
                .into_any_element(),
        )
    }

    /// The header's name: renamed in place, in Read (D11).
    fn render_name(&self, cx: &Context<Self>) -> AnyElement {
        if let Some(field) = self.inline_field(&EditTarget::Name, cx) {
            return div().w(rems(16.)).child(field).into_any_element();
        }
        let name = vault::name_of(self.slug()).to_string();
        let opened = name.clone();
        inline_edit::shown(
            "rusty-page-name",
            Label::new(name).size(LabelSize::Small),
            self.can_edit(),
            cx.listener(move |this, _, window, cx| {
                this.start_inline(EditTarget::Name, &opened, window, cx);
            }),
            cx,
        )
    }

    /// The title: the `title` property, edited in place (D5).
    fn render_title(&self, page: &RenderedPage, cx: &Context<Self>) -> AnyElement {
        if let Some(field) = self.inline_field(&EditTarget::Title, cx) {
            return field;
        }
        let opened = page.title.clone();
        inline_edit::shown(
            "rusty-page-title",
            Headline::new(page.title.clone()).size(HeadlineSize::Large),
            self.can_edit(),
            cx.listener(move |this, _, window, cx| {
                this.start_inline(EditTarget::Title, &opened, window, cx);
            }),
            cx,
        )
    }

    /// The properties, each edited by its value's kind (D7), each removable, and Add property.
    fn render_properties(&self, page: &RenderedPage, cx: &Context<Self>) -> AnyElement {
        let enabled = self.can_edit();
        let last = page.properties.len().saturating_sub(1);
        let new_key = self
            .inline
            .as_ref()
            .and_then(|editing| match editing.target {
                EditTarget::NewKey { kind } => Some(kind),
                _ => None,
            });
        v_flex()
            .w_full()
            .children(page.properties.iter().enumerate().map(|(at, property)| {
                let removed = property.key.clone();
                let remove = enabled.then(|| {
                    IconButton::new(("rusty-property-remove", at), IconName::Close)
                        .icon_size(IconSize::XSmall)
                        .icon_color(Color::Muted)
                        .tooltip(Tooltip::text("Remove property"))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.remove_property(&removed, window, cx);
                        }))
                        .into_any_element()
                });
                properties::row(
                    property.key.clone().into(),
                    self.render_value(at, property, enabled, cx),
                    remove,
                    at < last || enabled,
                    cx,
                )
            }))
            .children(new_key.and_then(|kind| {
                let field = self.inline_field(&EditTarget::NewKey { kind }, cx)?;
                Some(
                    h_flex()
                        .gap_2()
                        .py_1p5()
                        .child(
                            Label::new(kind.name())
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        )
                        .child(div().w(rems(16.)).child(field)),
                )
            }))
            .when(enabled, |list| list.child(Self::render_add_property(cx)))
            .into_any_element()
    }

    fn render_value(
        &self,
        at: usize,
        property: &Property,
        enabled: bool,
        cx: &Context<Self>,
    ) -> AnyElement {
        let kind = PropertyKind::of(&property.value);
        let key = property.key.clone();
        if let Some(field) = self.inline_field(
            &EditTarget::Value {
                key: key.clone(),
                kind,
            },
            cx,
        ) {
            return field;
        }
        match (kind, &property.value) {
            (PropertyKind::Checkbox, value) => {
                let state = if value.as_bool() == Some(true) {
                    ToggleState::Selected
                } else {
                    ToggleState::Unselected
                };
                Checkbox::new(("rusty-property-check", at), state)
                    .disabled(!enabled)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.toggle_checkbox(&key, window, cx);
                    }))
                    .into_any_element()
            }
            (PropertyKind::List, Value::Array(items)) => {
                self.render_list(at, &key, items, enabled, cx)
            }
            (PropertyKind::ReadOnly, value) => properties::value(value),
            (_, value) => {
                let text = match value {
                    Value::String(text) => text.clone(),
                    Value::Null => String::new(),
                    other => other.to_string(),
                };
                inline_edit::shown(
                    ("rusty-property-value", at),
                    properties::value(value),
                    enabled,
                    cx.listener(move |this, _, window, cx| {
                        this.start_inline(
                            EditTarget::Value {
                                key: key.clone(),
                                kind,
                            },
                            &text,
                            window,
                            cx,
                        );
                    }),
                    cx,
                )
            }
        }
    }

    /// A list value: a chip per item with its remove button, and an add button or its editor.
    fn render_list(
        &self,
        at: usize,
        key: &str,
        items: &[Value],
        enabled: bool,
        cx: &Context<Self>,
    ) -> AnyElement {
        let adding = self.inline_field(
            &EditTarget::ListItem {
                key: key.to_string(),
            },
            cx,
        );
        let chips = items.iter().enumerate().map(|(index, item)| {
            let removed = key.to_string();
            h_flex()
                .gap_0p5()
                .child(Chip::new(item.as_str().unwrap_or_default().to_string()))
                .when(enabled, |chip| {
                    chip.child(
                        IconButton::new(("rusty-item-remove", at * 1000 + index), IconName::Close)
                            .icon_size(IconSize::XSmall)
                            .icon_color(Color::Muted)
                            .tooltip(Tooltip::text("Remove"))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.remove_item(&removed, index, window, cx);
                            })),
                    )
                })
        });
        let added = key.to_string();
        h_flex()
            .flex_wrap()
            .gap_1()
            .children(chips)
            .map(|list| match adding {
                Some(field) => list.child(div().w(rems(10.)).child(field)),
                None if enabled => list.child(
                    IconButton::new(("rusty-item-add", at), IconName::Plus)
                        .icon_size(IconSize::XSmall)
                        .tooltip(Tooltip::text("Add an item"))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.start_inline(
                                EditTarget::ListItem { key: added.clone() },
                                "",
                                window,
                                cx,
                            );
                        })),
                ),
                None => list,
            })
            .into_any_element()
    }

    /// Add property: a menu of the five kinds, then the key's editor (D10).
    fn render_add_property(cx: &Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        div().pt_1().child(
            PopoverMenu::new("rusty-add-property")
                .trigger(
                    Button::new("rusty-add-property-button", "Add property")
                        .start_icon(Icon::new(IconName::Plus).size(IconSize::Small))
                        .label_size(LabelSize::Small),
                )
                .menu(move |window, cx| {
                    let view = view.clone();
                    Some(ContextMenu::build(window, cx, move |mut menu, _, _| {
                        for kind in PropertyKind::ADDABLE {
                            let view = view.clone();
                            menu = menu.entry(kind.name(), None, move |window, cx| {
                                let view = view.clone();
                                // The menu hands the focus back as it closes; the key's editor
                                // takes it after.
                                window.defer(cx, move |window, cx| {
                                    view.update(cx, |this, cx| {
                                        this.start_inline(
                                            EditTarget::NewKey { kind },
                                            "",
                                            window,
                                            cx,
                                        );
                                    })
                                    .log_err();
                                });
                            });
                        }
                        menu
                    }))
                }),
        )
    }

    /// The outline column: the page's headings, indented by level, a click bringing one to the
    /// top of the body (D4).
    fn render_outline(outline: &[Heading], cx: &Context<Self>) -> AnyElement {
        let shallowest = outline
            .iter()
            .map(|heading| heading.level)
            .min()
            .unwrap_or(1);
        v_flex()
            .id("rusty-page-outline")
            .flex_none()
            .w(rems(14.))
            .h_full()
            .overflow_y_scroll()
            .py_2()
            .border_l_1()
            .border_color(cx.theme().colors().border_variant)
            .children(outline.iter().enumerate().map(|(at, heading)| {
                let line = heading.line;
                let label = outline_label(&heading.text);
                let depth = usize::from(heading.level.saturating_sub(shallowest));
                ListItem::new(("rusty-page-heading", at))
                    .indent_level(depth)
                    .indent_step_size(px(12.))
                    .spacing(ListItemSpacing::Dense)
                    .tooltip(Tooltip::text(label.clone()))
                    .child(
                        Label::new(label)
                            .size(LabelSize::Small)
                            .color(if depth == 0 {
                                Color::Default
                            } else {
                                Color::Muted
                            })
                            .truncate(),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.scroll_to_heading_line(line, cx);
                    }))
            }))
            .into_any_element()
    }

    /// The lines over the body: Rusty not connected, and a failed save or open.
    fn render_lines(&self, cx: &Context<Self>) -> Option<AnyElement> {
        let unavailable = (!self.connected).then(|| super::unavailable(cx)).flatten();
        if unavailable.is_none() && self.problem.is_none() {
            return None;
        }
        Some(
            v_flex()
                .w_full()
                .flex_none()
                .px_4()
                .py_1()
                .gap_0p5()
                .children(unavailable.map(|reason| {
                    Label::new(reason)
                        .size(LabelSize::Small)
                        .color(Color::Warning)
                }))
                .children(self.problem.clone().map(|problem| {
                    Label::new(problem)
                        .size(LabelSize::Small)
                        .color(Color::Error)
                }))
                .into_any_element(),
        )
    }

    fn markdown_element(&self, window: &Window, cx: &Context<Self>) -> MarkdownElement {
        let mut style = MarkdownStyle::themed(MarkdownFont::Preview, window, cx);
        let muted = cx.theme().colors().text_muted;
        // A wikilink to no page yet is drawn muted, as Rusty's own page draws it.
        style.link_callback = Some(Rc::new(move |address: &str, _: &App| {
            address
                .starts_with("rusty:new/")
                .then(|| TextStyleRefinement {
                    color: Some(muted),
                    ..TextStyleRefinement::default()
                })
        }));
        let view = cx.entity().downgrade();
        MarkdownElement::new(self.markdown.clone(), style)
            .scroll_handle(self.scroll.clone())
            .on_url_click(move |address, window, cx| {
                // The click arrives inside the `Markdown` entity's update, which following the
                // link updates again (D9).
                let view = view.clone();
                window.defer(cx, move |window, cx| {
                    view.update(cx, |this, cx| this.follow(&address, window, cx))
                        .log_err();
                });
            })
    }

    fn render_read(&self, window: &Window, cx: &Context<Self>) -> AnyElement {
        if let Shown::Page(page) = &self.shown {
            // A line too long to wrap (a path in code, #663's report) must not widen the body
            // past the tab and push the outline out.
            let body = div()
                .id("rusty-page-body")
                .flex_1()
                .min_w_0()
                .min_h_0()
                .h_full()
                .overflow_x_hidden()
                .overflow_y_scroll()
                .track_scroll(&self.scroll)
                .child(
                    v_flex()
                        .w_full()
                        .px_8()
                        .py_6()
                        .gap_4()
                        .child(self.render_title(page, cx))
                        .child(self.render_properties(page, cx))
                        .child(self.markdown_element(window, cx)),
                );
            let outline = (self.outline_shown && !page.outline.is_empty())
                .then(|| Self::render_outline(&page.outline, cx));
            return h_flex()
                .flex_1()
                .min_h_0()
                .w_full()
                .items_start()
                .child(body)
                .children(outline)
                .into_any_element();
        }
        let body = div()
            .id("rusty-page-body")
            .flex_1()
            .min_h_0()
            .w_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll);
        let slug = self.slug();
        match &self.shown {
            Shown::Loading => body.child(state_line(format!("Reading {slug}…"), Color::Muted)),
            Shown::Missing => body.child(state_line(
                format!("No page {slug} in the brain."),
                Color::Muted,
            )),
            Shown::Failed(error) => body.child(state_line(error.to_string(), Color::Error)),
            Shown::Page(_) => body,
        }
        .into_any_element()
    }
}

/// What a rename did, in words.
fn renamed_words(report: &RenameReport) -> String {
    match report.pages_rewritten {
        0 => format!("Renamed to {}; no page linked to it.", report.to),
        1 => format!("Renamed to {}; links updated in 1 page.", report.to),
        pages => format!("Renamed to {}; links updated in {pages} pages.", report.to),
    }
}

/// Points every Page tab at the page or folder Rusty renamed, in the page each shows and in its
/// history, once the update in progress ends (D9): the Brain view's rename and move call it too.
pub(crate) fn follow_rename(report: &RenameReport, window: &Window, cx: &mut App) {
    let (from, to) = (report.from.clone(), report.to.clone());
    window.defer(cx, move |window, cx| {
        let views: Vec<Entity<PageView>> = cx
            .try_global::<PageViews>()
            .map(|views| views.0.iter().filter_map(WeakEntity::upgrade).collect())
            .unwrap_or_default();
        for view in views {
            view.update(cx, |view, cx| view.renamed(&from, &to, window, cx));
        }
    });
}

/// A line in the body's place: loading, missing, or failed.
fn state_line(text: String, color: Color) -> impl IntoElement {
    div().px_8().py_6().child(Label::new(text).color(color))
}

impl EventEmitter<PageEvent> for PageView {}

impl Focusable for PageView {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor().map_or_else(
            || self.focus_handle.clone(),
            |editor| editor.focus_handle(cx),
        )
    }
}

impl Render for PageView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("RustyPage")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::back))
            .on_action(cx.listener(Self::forward))
            .on_action(cx.listener(Self::toggle_edit))
            .on_action(cx.listener(Self::toggle_outline))
            .on_action(cx.listener(Self::toggle_bookmark))
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(self.render_header(cx))
            .children(self.render_lines(cx))
            .child(self.editor().map_or_else(
                || self.render_read(window, cx),
                |editor| {
                    div()
                        .flex_1()
                        .min_h_0()
                        .w_full()
                        .child(editor.clone())
                        .into_any_element()
                },
            ))
    }
}

impl Item for PageView {
    type Event = PageEvent;

    /// In Edit, the tab hands its editor to Zed, so Zed's outline panel lists the source's headings,
    /// as Zed's Markdown preview hands its source (#656).
    fn act_as_type<'a>(
        &'a self,
        type_id: TypeId,
        self_handle: &'a Entity<Self>,
        _: &'a App,
    ) -> Option<AnyEntity> {
        if type_id == TypeId::of::<Self>() {
            Some(self_handle.clone().into())
        } else if type_id == TypeId::of::<Editor>() {
            self.editor().map(|editor| editor.clone().into())
        } else {
            None
        }
    }

    fn tab_content(&self, params: TabContentParams, _window: &Window, cx: &App) -> AnyElement {
        Label::new(self.tab_content_text(0, cx))
            .color(params.text_color())
            .when(params.preview, LabelCommon::italic)
            .into_any_element()
    }

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        match &self.shown {
            Shown::Page(page) if !page.title.is_empty() => page.title.clone().into(),
            _ => vault::name_of(self.slug()).to_string().into(),
        }
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::FileMarkdown))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(self.slug().to_string().into())
    }

    fn to_item_events(event: &PageEvent, f: &mut dyn FnMut(ItemEvent)) {
        match event {
            PageEvent::UpdateTab => f(ItemEvent::UpdateTab),
            PageEvent::Edited => f(ItemEvent::Edit),
        }
    }

    // While Edit shows, the file's state is the editor's, as Zed's Markdown preview and project
    // diff pass theirs: the unsaved dot, the close prompt and Ctrl-S follow the buffer.
    fn is_dirty(&self, cx: &App) -> bool {
        self.editor()
            .is_some_and(|editor| editor.read(cx).is_dirty(cx))
    }

    fn has_conflict(&self, cx: &App) -> bool {
        self.editor()
            .is_some_and(|editor| editor.read(cx).has_conflict(cx))
    }

    fn can_save(&self, cx: &App) -> bool {
        self.editor()
            .is_some_and(|editor| editor.read(cx).can_save(cx))
    }

    fn save(
        &mut self,
        options: SaveOptions,
        project: Entity<Project>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<anyhow::Result<()>> {
        self.editor().cloned().map_or_else(
            || Task::ready(Ok(())),
            |editor| editor.update(cx, |editor, cx| editor.save(options, project, window, cx)),
        )
    }

    fn reload(
        &mut self,
        project: Entity<Project>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<anyhow::Result<()>> {
        self.editor().cloned().map_or_else(
            || Task::ready(Ok(())),
            |editor| editor.update(cx, |editor, cx| editor.reload(project, window, cx)),
        )
    }

    fn for_each_project_item(
        &self,
        cx: &App,
        f: &mut dyn FnMut(EntityId, &dyn project::ProjectItem),
    ) {
        if let Some(editor) = self.editor() {
            editor.read(cx).for_each_project_item(cx, f);
        }
    }

    fn added_to_workspace(
        &mut self,
        workspace: &mut Workspace,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        // L-613: an item moved to another workspace follows it.
        self.workspace = workspace.weak_handle();
    }
}

/// The line of the heading of `page` whose text, as written or as the outline shows it, is
/// `heading`: how Rusty's app finds a heading bookmark's place.
fn outline_line(page: &RenderedPage, heading: &str) -> Option<usize> {
    let heading = heading.trim();
    page.outline
        .iter()
        .find(|entry| entry.text.trim() == heading || outline_label(&entry.text) == heading)
        .map(|entry| entry.line)
}
