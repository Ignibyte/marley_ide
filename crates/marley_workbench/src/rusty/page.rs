//! A page of Rusty's brain in a center tab (#645).
//!
//! The tab shows the page's title, its properties and its body, drawn by Zed's own `markdown`
//! crate after `marley_rusty::page::page_markdown` turns each wikilink into a link with Rusty's
//! address. A link opens its page in the same tab, with Back and Forward of the tab's own; Edit
//! shows the page's file in a Zed editor inside the tab, and Read, Back and Forward save it first.
//! `open_later` is the one way to open a page: the rail's Brain view and `rusty::OpenPage` call
//! it, and it follows Zed's preview tabs as the project panel's clicks do.

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use editor::{Editor, EditorEvent};
use gpui::{
    Action, AnyElement, App, Context, Entity, EntityId, EventEmitter, FocusHandle, Focusable,
    Point, ScrollHandle, SharedString, Subscription, Task, TextStyleRefinement, WeakEntity, Window,
    actions,
};
use language::LanguageRegistry;
use markdown::{Markdown, MarkdownElement, MarkdownFont, MarkdownOptions, MarkdownStyle};
use marley_rusty::page::{
    BRAIN_RENDER, PageHistory, PageLink, RenderedPage, Visit, body_of, page_file_in, page_markdown,
};
use marley_rusty::vault;
use project::Project;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use ui::{Headline, HeadlineSize, Tooltip, prelude::*};
use util::ResultExt as _;
use util::markdown::generate_heading_slug;
use workspace::item::{Item, ItemEvent, SaveOptions, TabContentParams};
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use super::properties;

/// How often, and how many times, the tab looks whether the Markdown parse ended.
const PARSE_CHECK: Duration = Duration::from_millis(50);
const PARSE_CHECKS: usize = 40;

/// Opens a page of Rusty's brain in a tab, or brings forward the tab that shows it; with
/// `preview`, in the pane's preview tab, as one click in the project panel opens a file.
#[derive(Clone, Debug, PartialEq, Eq, JsonSchema, Action)]
#[action(namespace = rusty)]
pub struct OpenPage {
    /// The page's slug, such as `projects/marley`.
    pub slug: String,
    /// Whether the page opens in the pane's preview tab.
    pub preview: bool,
}

/// `OpenPage`'s fields as a keymap writes them. The action reads them through this struct: clippy
/// takes the `Action` derive's registration for a method it cannot check, which a derived
/// `Deserialize` beside it trips (`unsafe_derive_deserialize`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OpenPageFields {
    slug: String,
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
    ]
);

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
            open_later(
                cx.weak_entity(),
                action.slug.clone(),
                action.preview,
                !action.preview,
                window,
                cx,
            );
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
    // arrives inside its page's `Markdown` update.
    window.defer(cx, move |window, cx| {
        workspace
            .update(cx, |workspace, cx| {
                open(workspace, slug, preview, focus, window, cx);
            })
            .log_err();
    });
}

fn open(
    workspace: &mut Workspace,
    slug: String,
    preview: bool,
    focus: bool,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let shown = workspace
        .items_of_type::<PageView>(cx)
        .find(|view| view.read(cx).slug() == slug);
    if let Some(shown) = shown {
        workspace.activate_item(&shown, true, focus, window, cx);
        if !preview && let Some(pane) = workspace.pane_for(&shown) {
            pane.update(cx, |pane, _| {
                pane.unpreview_item_if_preview(shown.entity_id());
            });
        }
        return;
    }
    let languages = Arc::clone(workspace.project().read(cx).languages());
    let weak_workspace = cx.weak_entity();
    let view = cx.new(|cx| PageView::new(Visit::page(slug), weak_workspace, languages, window, cx));
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
    _subscriptions: [Subscription; 2],
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
    fn new(
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
            // A change Rusty announces: the page is read again while it shows Read.
            cx.observe_global_in::<super::Announced>(window, |this, window, cx| {
                if matches!(this.mode, Mode::Read) {
                    this.load(window, cx);
                }
            }),
            cx.observe_global_in::<super::Rusty>(window, Self::rusty_changed),
        ];
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
            _subscriptions: subscriptions,
        };
        this.load(window, cx);
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
                self.markdown.update(cx, |view, cx| {
                    view.replace(markdown, cx);
                    if let Some(heading) = heading {
                        view.scroll_to_heading_when_parsed(
                            generate_heading_slug(&heading).into(),
                            cx,
                        );
                    }
                });
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
    fn navigate(&mut self, visit: Visit, window: &mut Window, cx: &mut Context<Self>) {
        self.leave_edit_then(window, cx, move |this, window, cx| {
            this.history.visit(visit);
            this.scroll.set_offset(Point::default());
            this.load(window, cx);
        });
    }

    fn back(&mut self, _: &PageBack, window: &mut Window, cx: &mut Context<Self>) {
        self.leave_edit_then(window, cx, |this, window, cx| {
            if this.history.back() {
                this.scroll.set_offset(Point::default());
                this.load(window, cx);
            }
        });
    }

    fn forward(&mut self, _: &PageForward, window: &mut Window, cx: &mut Context<Self>) {
        self.leave_edit_then(window, cx, |this, window, cx| {
            if this.history.forward() {
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
            PageLink::Missing { target } => {
                self.toast(format!("There is no page {target} yet."), cx);
            }
            PageLink::Heading(heading) => {
                self.markdown.update(cx, |view, cx| {
                    view.scroll_to_heading(&generate_heading_slug(&heading), cx);
                });
            }
            PageLink::External(address) => cx.open_url(&address),
        }
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
                    .child(Label::new(vault::name_of(slug).to_string()).size(LabelSize::Small)),
            )
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
            Shown::Page(page) => body.child(
                v_flex()
                    .w_full()
                    .px_8()
                    .py_6()
                    .gap_4()
                    .child(Headline::new(page.title.clone()).size(HeadlineSize::Large))
                    .children(properties::render(&page.properties, cx))
                    .child(self.markdown_element(window, cx)),
            ),
        }
        .into_any_element()
    }
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
