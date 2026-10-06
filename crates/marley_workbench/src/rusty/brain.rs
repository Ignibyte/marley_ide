//! The rail's Brain view (#644): Rusty's vault behind the Brain button in the rail's header.
//!
//! Under the header, a fixed row with Today and Graph (#647), a brain search field that asks Rusty on Enter, and the
//! vault tree over `brain_tree`. Its menus, its name editor and its drags make, rename, move and
//! delete pages and folders through Rusty's tools, never the disk; every open goes through
//! `open_page`, which opens a Page tab.

use std::collections::HashSet;
use std::ops::Range;
use std::sync::Arc;

use editor::Editor;
use editor::actions::SelectAll;
use gpui::{
    Anchor, AnyElement, App, ClickEvent, Context, DismissEvent, ElementId, Entity, FocusHandle,
    Focusable, MouseButton, MouseDownEvent, Pixels, Point, PromptLevel, Render, ScrollStrategy,
    Subscription, UniformListScrollHandle, WeakEntity, Window, anchored, deferred, px,
    uniform_list,
};
use marley_rusty::bookmarks::{Bookmark, BookmarkKind, BookmarkWrite};
use marley_rusty::vault::{
    self, BRAIN_DAILY_NOTE, BRAIN_DELETE_FOLDER, BRAIN_DELETE_PAGE, BRAIN_NEW_FOLDER,
    BRAIN_NEW_PAGE, BRAIN_RENAME, BRAIN_SEARCH, Key, Move, NodeKind, RenameReport, SEARCH_LIMIT,
    SearchHit, VaultNode, VaultRow,
};
use menu::{
    Cancel, Confirm, SelectChild, SelectFirst, SelectLast, SelectNext, SelectParent, SelectPrevious,
};
use serde_json::{Value, json};
use settings::Settings as _;
use smallvec::SmallVec;
use ui::{
    ContextMenu, IndentGuideColors, ListItem, ListItemSpacing, ListSubHeader, Tooltip,
    indent_guides, prelude::*,
};
use util::ResultExt as _;
use workspace::item::PreviewTabsSettings;
use workspace::notifications::NotificationId;
use workspace::{MultiWorkspace, Toast, Workspace};

use super::favourites;
use crate::ToggleBrainView;
use crate::rail::Rail;

/// The tree's step per level, the project panel's default.
const INDENT: Pixels = px(20.);

/// Registers [`ToggleBrainView`] on every workspace; [`super::init`] calls it once.
pub(super) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|_, _: &ToggleBrainView, window, cx| {
            // Opening the rail updates the window's `MultiWorkspace`, which this may run inside.
            window.defer(cx, toggle_in_window);
        });
    })
    .detach();
}

/// Flips the window's rail between its views, or says in a toast why the Brain view cannot show.
fn toggle_in_window(window: &mut Window, cx: &mut App) {
    let Some(multi_workspace) = window.root::<MultiWorkspace>().flatten() else {
        return;
    };
    let rail = multi_workspace
        .read(cx)
        .sidebar()
        .map(workspace::SidebarHandle::to_any)
        .and_then(|view| view.downcast::<Rail>().ok());
    let refused = match rail {
        None => Some(SharedString::new_static(
            "The Brain view is in the Marley layout's rail; this window uses Zed's layout.",
        )),
        Some(_) if !multi_workspace.read(cx).multi_workspace_enabled(cx) => {
            Some(SharedString::new_static(
                "The rail, and the Brain view in it, is hidden while Zed's AI features are off.",
            ))
        }
        Some(rail) => rail.read(cx).brain_refusal(cx).or_else(|| {
            // Opening reads the rail, so it comes before the rail's update.
            if !multi_workspace.read(cx).sidebar_open() {
                multi_workspace.update(cx, MultiWorkspace::open_sidebar);
            }
            rail.update(cx, |rail, cx| rail.toggle_brain_view(window, cx));
            None
        }),
    };
    if let Some(message) = refused {
        let workspace = multi_workspace.read(cx).workspace().clone();
        workspace.update(cx, |workspace, cx| {
            show_toast(workspace, message.to_string(), cx);
        });
    }
}

fn show_toast(workspace: &mut Workspace, message: String, cx: &mut Context<Workspace>) {
    workspace.show_toast(
        Toast::new(NotificationId::unique::<BrainView>(), message),
        cx,
    );
}

/// Opens the page `slug` in `workspace`'s Page tab (#645): in the pane's preview tab when
/// `preview`, the keyboard moved into it when `focus`. Every open in the Brain view comes here.
pub(crate) fn open_page(
    workspace: &Entity<Workspace>,
    slug: &str,
    preview: bool,
    focus: bool,
    window: &Window,
    cx: &mut App,
) {
    super::page::open_later(
        workspace.downgrade(),
        slug.to_string(),
        preview,
        focus,
        window,
        cx,
    );
}

/// Whether one click opens a page in a preview tab: where Zed's project panel would open a file
/// in one.
fn preview_on_click(cx: &App) -> bool {
    let settings = PreviewTabsSettings::get_global(cx);
    settings.enabled && settings.enable_preview_from_project_panel
}

/// The query last sent to brain search, and its hits: `None` while Rusty answers.
struct Found {
    query: String,
    hits: Option<Result<Vec<SearchHit>, SharedString>>,
}

/// One line of the list: a row of the tree, the name editor of a page or folder being made, or a
/// search hit.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Line {
    Row(usize),
    /// At this depth.
    Draft(usize),
    Hit(usize),
}

/// A name being typed in the tree.
#[derive(Clone, PartialEq, Eq)]
enum Edit {
    NewPage {
        folder: String,
    },
    NewFolder {
        parent: String,
    },
    Rename {
        path: String,
    },
    /// A favourite's title, by the bookmark's key (#662).
    BookmarkTitle {
        key: String,
    },
}

struct Editing {
    edit: Edit,
    editor: Entity<Editor>,
    _blur: Subscription,
}

/// A page or folder dragged in the tree.
#[derive(Clone)]
struct DraggedVaultEntry {
    path: String,
    name: SharedString,
}

impl Render for DraggedVaultEntry {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        div()
            .max_w_64()
            .px_2()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(colors.border)
            .bg(colors.elevated_surface_background)
            .shadow_md()
            .child(
                Label::new(self.name.clone())
                    .size(LabelSize::Small)
                    .truncate(),
            )
    }
}

/// The rail's Brain view.
pub(crate) struct BrainView {
    multi_workspace: WeakEntity<MultiWorkspace>,
    focus_handle: FocusHandle,
    search: Entity<Editor>,
    /// A search's hits, shown in the tree's place until the field is cleared.
    found: Option<Found>,
    /// The vault as last read, or why it could not be.
    tree: Option<Result<Arc<VaultNode>, SharedString>>,
    /// The open folders, by path.
    open: HashSet<String>,
    rows: Vec<VaultRow>,
    lines: Vec<Line>,
    /// The selected row's path, or hit's slug: the keyboard's, or the last one clicked, opened or
    /// made.
    selected: Option<String>,
    /// Whether the selected row should scroll into view once the vault lists it.
    reveal_pending: bool,
    editing: Option<Editing>,
    scroll: UniformListScrollHandle,
    /// The menu a right-click opened, where it opened.
    menu: Option<(Entity<ContextMenu>, Point<Pixels>, Subscription)>,
    _vault: Subscription,
    _bookmarks: Subscription,
}

impl BrainView {
    pub(crate) fn new(
        multi_workspace: WeakEntity<MultiWorkspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Search the brain…", window, cx);
            editor
        });
        let vault = cx.observe_global::<super::Vault>(Self::vault_changed);
        super::want_vault(cx);
        let bookmarks = cx.observe_global::<favourites::Bookmarks>(|_, cx| cx.notify());
        favourites::ensure(cx);
        let mut this = Self {
            multi_workspace,
            focus_handle: cx.focus_handle(),
            search,
            found: None,
            tree: cx.global::<super::Vault>().tree.clone(),
            open: HashSet::default(),
            rows: Vec::new(),
            lines: Vec::new(),
            selected: None,
            reveal_pending: false,
            editing: None,
            scroll: UniformListScrollHandle::new(),
            menu: None,
            _vault: vault,
            _bookmarks: bookmarks,
        };
        this.rebuild();
        this
    }

    /// The search field's focus, which `secondary-f` takes while the Brain view shows.
    pub(crate) fn search_focus(&self, cx: &App) -> FocusHandle {
        self.search.focus_handle(cx)
    }

    /// The header's New Page: in the selected row's folder, or at the top.
    pub(crate) fn new_page_here(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let folder = match self.selected_row() {
            Some(row) if row.kind == NodeKind::Folder && self.found.is_none() => row.path.clone(),
            Some(row) if self.found.is_none() => vault::folder_of(&row.path).to_string(),
            _ => String::new(),
        };
        self.start_edit(Edit::NewPage { folder }, window, cx);
    }

    fn vault_changed(&mut self, cx: &mut Context<Self>) {
        self.tree.clone_from(&cx.global::<super::Vault>().tree);
        self.rebuild();
        if self.reveal_pending && self.selected_row().is_some() {
            self.reveal_pending = false;
            self.scroll_to_selected();
        }
        cx.notify();
    }

    fn hits(&self) -> Option<&[SearchHit]> {
        match &self.found {
            Some(Found {
                hits: Some(Ok(hits)),
                ..
            }) => Some(hits),
            _ => None,
        }
    }

    fn selected_row(&self) -> Option<&VaultRow> {
        let selected = self.selected.as_deref()?;
        self.rows.iter().find(|row| row.path == selected)
    }

    /// The rows and the lines from the tree, the open folders, a search and an edit.
    fn rebuild(&mut self) {
        self.rows = match &self.tree {
            Some(Ok(tree)) => vault::rows(tree, &self.open),
            _ => Vec::new(),
        };
        if self.found.is_some() {
            self.lines = (0..self.hits().map_or(0, <[_]>::len))
                .map(Line::Hit)
                .collect();
            return;
        }
        let mut lines: Vec<Line> = (0..self.rows.len()).map(Line::Row).collect();
        let draft_in = match self.editing.as_ref().map(|editing| &editing.edit) {
            Some(Edit::NewPage { folder } | Edit::NewFolder { parent: folder }) => Some(folder),
            _ => None,
        };
        if let Some(folder) = draft_in {
            if folder.is_empty() {
                lines.insert(0, Line::Draft(0));
            } else if let Some(at) = self.rows.iter().position(|row| row.path == *folder) {
                lines.insert(at + 1, Line::Draft(self.rows[at].depth + 1));
            }
        }
        self.lines = lines;
    }

    fn line_key(&self, line: Line) -> Option<&str> {
        match line {
            Line::Row(at) => self.rows.get(at).map(|row| row.path.as_str()),
            Line::Hit(at) => self.hits()?.get(at).map(|hit| hit.slug.as_str()),
            Line::Draft(_) => None,
        }
    }

    fn line_depth(&self, line: Line) -> usize {
        match line {
            Line::Row(at) => self.rows.get(at).map_or(0, |row| row.depth),
            Line::Draft(depth) => depth,
            Line::Hit(_) => 0,
        }
    }

    fn selected_line(&self) -> Option<usize> {
        let selected = self.selected.as_deref()?;
        self.lines
            .iter()
            .position(|line| self.line_key(*line) == Some(selected))
    }

    fn scroll_to_selected(&self) {
        if let Some(line) = self.selected_line() {
            self.scroll.scroll_to_item(line, ScrollStrategy::Nearest);
        }
    }

    /// Selects `path` and opens the folders above it, scrolling to it once the vault lists it.
    fn reveal(&mut self, path: &str) {
        self.open.extend(vault::folders_above(path));
        self.selected = Some(path.to_string());
        self.rebuild();
        if self.selected_row().is_some() {
            self.scroll_to_selected();
        } else {
            self.reveal_pending = true;
        }
    }

    fn toggle_folder(&mut self, path: &str) {
        if !self.open.remove(path) {
            self.open.insert(path.to_string());
        }
        self.rebuild();
    }

    fn open(&self, slug: &str, preview: bool, focus: bool, window: &Window, cx: &mut App) {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return;
        };
        let workspace = multi_workspace.read(cx).workspace().clone();
        open_page(&workspace, slug, preview, focus, window, cx);
    }

    /// The Graph entry: the workspace's Graph tab, opened or brought forward (#647).
    fn open_graph(&self, window: &Window, cx: &mut App) {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return;
        };
        let workspace = multi_workspace.read(cx).workspace().downgrade();
        super::graph_tab::open_later(workspace, window, cx);
    }

    /// The Tasks entry: the workspace's Tasks tab, opened or brought forward (#658).
    fn open_tasks(&self, window: &Window, cx: &mut App) {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return;
        };
        let workspace = multi_workspace.read(cx).workspace().downgrade();
        super::tasks_tab::open_later(workspace, None, window, cx);
    }

    /// The Decisions entry: the workspace's Decisions tab, opened or brought forward (#659).
    fn open_decisions(&self, window: &Window, cx: &mut App) {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return;
        };
        let workspace = multi_workspace.read(cx).workspace().downgrade();
        super::decisions_tab::open_later(workspace, window, cx);
    }

    /// The Memory entry: the workspace's Memory tab, opened or brought forward (#664).
    fn open_memory(&self, window: &Window, cx: &mut App) {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return;
        };
        let workspace = multi_workspace.read(cx).workspace().downgrade();
        super::memory_tab::open_later(workspace, window, cx);
    }

    fn toast(&self, message: String, cx: &mut App) {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return;
        };
        let workspace = multi_workspace.read(cx).workspace().clone();
        workspace.update(cx, |workspace, cx| show_toast(workspace, message, cx));
    }

    /// A click on a row or a hit: a folder folds or unfolds; a page opens, in a preview tab on
    /// one click and kept on two, as the project panel opens files.
    fn clicked(&mut self, path: &str, click_count: usize, window: &Window, cx: &mut Context<Self>) {
        let folder = self.found.is_none()
            && self
                .rows
                .iter()
                .any(|row| row.path == path && row.kind == NodeKind::Folder);
        self.selected = Some(path.to_string());
        if folder {
            self.toggle_folder(path);
        } else {
            let preview = click_count == 1 && preview_on_click(cx);
            self.open(path, preview, click_count > 1, window, cx);
        }
        cx.notify();
    }

    /// Enter on the selected row or hit does what one click on it does.
    fn open_selected(&mut self, window: &Window, cx: &mut Context<Self>) {
        if let Some(selected) = self.selected.clone()
            && (self.found.is_some() || self.selected_row().is_some())
        {
            self.clicked(&selected, 1, window, cx);
        }
    }

    fn step(&mut self, key: Key, cx: &mut Context<Self>) {
        if self.found.is_some() {
            let Some(last) = self.lines.len().checked_sub(1) else {
                return;
            };
            let at = self.selected_line();
            let to = match key {
                Key::Next => at.map_or(0, |at| (at + 1).min(last)),
                Key::Previous => at.map_or(last, |at| at.saturating_sub(1)),
                Key::First => 0,
                Key::Last => last,
                Key::Child | Key::Parent => return,
            };
            self.selected = self
                .lines
                .get(to)
                .and_then(|line| self.line_key(*line))
                .map(str::to_string);
        } else {
            let at = self
                .selected
                .as_deref()
                .and_then(|selected| self.rows.iter().position(|row| row.path == selected));
            match vault::step(&self.rows, at, key) {
                Some(Move::To(at)) => self.selected = self.rows.get(at).map(|row| row.path.clone()),
                Some(Move::Open(path)) => {
                    self.open.insert(path);
                    self.rebuild();
                }
                Some(Move::Close(path)) => {
                    self.open.remove(&path);
                    self.rebuild();
                }
                None => return,
            }
        }
        self.scroll_to_selected();
        cx.notify();
    }

    fn select_next(&mut self, _: &SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        self.step(Key::Next, cx);
    }

    fn select_previous(&mut self, _: &SelectPrevious, _: &mut Window, cx: &mut Context<Self>) {
        self.step(Key::Previous, cx);
    }

    fn select_first(&mut self, _: &SelectFirst, _: &mut Window, cx: &mut Context<Self>) {
        self.step(Key::First, cx);
    }

    fn select_last(&mut self, _: &SelectLast, _: &mut Window, cx: &mut Context<Self>) {
        self.step(Key::Last, cx);
    }

    fn select_child(&mut self, _: &SelectChild, _: &mut Window, cx: &mut Context<Self>) {
        self.step(Key::Child, cx);
    }

    fn select_parent(&mut self, _: &SelectParent, _: &mut Window, cx: &mut Context<Self>) {
        self.step(Key::Parent, cx);
    }

    /// Enter: a name being typed is made; in the search field, a new query is sent to Rusty and
    /// an unchanged one opens the selected hit; elsewhere the selected row opens.
    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        if self
            .editing
            .as_ref()
            .is_some_and(|editing| editing.editor.focus_handle(cx).is_focused(window))
        {
            self.commit_edit(true, window, cx);
        } else if self.search.focus_handle(cx).is_focused(window) {
            let query = self.search.read(cx).text(cx).trim().to_string();
            if query.is_empty() {
                return;
            }
            if self
                .found
                .as_ref()
                .is_some_and(|found| found.query == query)
            {
                self.open_selected(window, cx);
            } else {
                self.search_for(query, cx);
            }
        } else {
            self.open_selected(window, cx);
        }
    }

    /// Escape, as the rail's filter takes it: a name being typed is dropped; the search field is
    /// cleared, then left for the tree; then the key passes on.
    fn cancel(&mut self, _: &Cancel, window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.take().is_some() {
            window.focus(&self.focus_handle, cx);
            self.rebuild();
            cx.notify();
        } else if self.found.is_some() || !self.search.read(cx).text(cx).is_empty() {
            self.clear_search(window, cx);
        } else if self.search.focus_handle(cx).is_focused(window) {
            window.focus(&self.focus_handle, cx);
        } else {
            cx.propagate();
        }
    }

    fn clear_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.search
            .update(cx, |editor, cx| editor.set_text("", window, cx));
        self.found = None;
        self.rebuild();
        self.scroll_to_selected();
        cx.notify();
    }

    /// Asks brain search for `query`, its hits listed in the tree's place. Sent on Enter alone:
    /// with an embedding provider set, Rusty embeds every query it is sent.
    fn search_for(&mut self, query: String, cx: &mut Context<Self>) {
        self.found = Some(Found {
            query: query.clone(),
            hits: None,
        });
        self.selected = None;
        self.rebuild();
        cx.notify();
        let asking = super::call_tool(
            BRAIN_SEARCH,
            json!({ "query": query, "limit": SEARCH_LIMIT }),
            cx,
        );
        cx.spawn(async move |this, cx| {
            let hits = asking.await.and_then(|text| {
                vault::hits_from_answer(&text)
                    .map_err(|error| format!("{BRAIN_SEARCH}'s answer did not parse: {error}"))
            });
            this.update(cx, |this, cx| {
                if this.found.as_ref().is_none_or(|found| found.query != query) {
                    return;
                }
                this.selected = hits
                    .as_ref()
                    .ok()
                    .and_then(|hits| hits.first())
                    .map(|hit| hit.slug.clone());
                this.found = Some(Found {
                    query,
                    hits: Some(hits.map_err(SharedString::from)),
                });
                this.rebuild();
                cx.notify();
            })
            .log_err();
        })
        .detach();
    }

    /// Today: Rusty's note for today, made when missing, opened kept.
    fn today(window: &Window, cx: &Context<Self>) {
        let asking = super::call_tool(BRAIN_DAILY_NOTE, json!({}), cx);
        cx.spawn_in(window, async move |this, cx| {
            let slug = asking.await.and_then(|text| {
                vault::page_slug_from_answer(&text)
                    .map_err(|error| format!("{BRAIN_DAILY_NOTE}'s answer did not parse: {error}"))
            });
            this.update_in(cx, |this, window, cx| match slug {
                Ok(slug) => {
                    super::reread_vault(cx);
                    if this.found.is_none() {
                        this.reveal(&slug);
                    }
                    this.open(&slug, false, true, window, cx);
                    cx.notify();
                }
                Err(error) => this.toast(format!("Could not open today's note: {error}"), cx),
            })
            .log_err();
        })
        .detach();
    }

    /// Calls a tool that writes the vault: a success reads the vault again and runs `done` with
    /// Rusty's answer; a refusal shows Rusty's message after `failure` and changes nothing shown.
    fn write(
        tool: &'static str,
        arguments: Value,
        failure: String,
        done: impl FnOnce(&mut Self, String, &mut Window, &mut Context<Self>) + 'static,
        window: &Window,
        cx: &Context<Self>,
    ) {
        let asking = super::call_tool(tool, arguments, cx);
        cx.spawn_in(window, async move |this, cx| {
            let answer = asking.await;
            this.update_in(cx, |this, window, cx| match answer {
                Ok(text) => {
                    super::reread_vault(cx);
                    // Rusty's renames and deletes carry and drop bookmarks, unannounced on the
                    // service connection (#662).
                    favourites::read(cx);
                    done(this, text, window, cx);
                }
                Err(error) => this.toast(format!("{failure}: {error}"), cx),
            })
            .log_err();
        })
        .detach();
    }

    /// Puts a name editor in the tree: a new row under its folder, or in place of a row's name,
    /// the name selected.
    fn start_edit(&mut self, edit: Edit, window: &mut Window, cx: &mut Context<Self>) {
        // A favourite's title is typed in the Favourites group, so a search stays (#662).
        let in_tree = !matches!(edit, Edit::BookmarkTitle { .. });
        if in_tree && self.found.is_some() {
            self.search
                .update(cx, |editor, cx| editor.set_text("", window, cx));
            self.found = None;
        }
        let name = match &edit {
            Edit::NewPage { folder } | Edit::NewFolder { parent: folder } => {
                if !folder.is_empty() {
                    self.open.extend(vault::folders_above(folder));
                    self.open.insert(folder.clone());
                }
                String::new()
            }
            Edit::Rename { path } => {
                self.selected = Some(path.clone());
                vault::name_of(path).to_string()
            }
            Edit::BookmarkTitle { key } => favourites::list(cx)
                .iter()
                .find(|bookmark| bookmark.key() == *key)
                .map(|bookmark| bookmark.shown_title().to_string())
                .unwrap_or_default(),
        };
        let editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_text(name, window, cx);
            editor.select_all(&SelectAll, window, cx);
            editor
        });
        window.focus(&editor.focus_handle(cx), cx);
        // Losing focus makes the name, as the project panel's editor does.
        let blur = cx.on_blur(&editor.focus_handle(cx), window, |this, window, cx| {
            this.commit_edit(false, window, cx);
        });
        self.editing = Some(Editing {
            edit,
            editor,
            _blur: blur,
        });
        if in_tree {
            self.rebuild();
            if let Some(line) = self
                .lines
                .iter()
                .position(|line| matches!(line, Line::Draft(_)))
                .or_else(|| self.selected_line())
            {
                self.scroll.scroll_to_item(line, ScrollStrategy::Nearest);
            }
        }
        cx.notify();
    }

    /// Makes what the name editor holds through Rusty's tools. On Enter an empty page name lets
    /// Rusty call it Untitled; on a lost focus, an empty name makes nothing.
    fn commit_edit(&mut self, entered: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(editing) = self.editing.take() else {
            return;
        };
        let typed = editing.editor.read(cx).text(cx);
        if entered {
            window.focus(&self.focus_handle, cx);
        }
        self.rebuild();
        cx.notify();
        match editing.edit {
            Edit::NewPage { folder } => {
                if !entered && typed.trim().is_empty() {
                    return;
                }
                Self::write(
                    BRAIN_NEW_PAGE,
                    json!({ "folder": folder, "name": typed.trim() }),
                    "Could not make the page".to_string(),
                    |this, answer, window, cx| match vault::slug_from_answer(&answer) {
                        Ok(slug) => {
                            this.reveal(&slug);
                            this.open(&slug, false, true, window, cx);
                            cx.notify();
                        }
                        Err(error) => this.toast(
                            format!("{BRAIN_NEW_PAGE}'s answer did not parse: {error}"),
                            cx,
                        ),
                    },
                    window,
                    cx,
                );
            }
            Edit::NewFolder { parent } => {
                let Some(path) = vault::child_path(&parent, &typed) else {
                    return;
                };
                Self::write(
                    BRAIN_NEW_FOLDER,
                    json!({ "path": path }),
                    "Could not make the folder".to_string(),
                    move |this, _, _, cx| {
                        this.reveal(&path);
                        cx.notify();
                    },
                    window,
                    cx,
                );
            }
            Edit::Rename { path } => {
                let Some(to) = vault::rename_target(&path, &typed) else {
                    return;
                };
                Self::write(
                    BRAIN_RENAME,
                    json!({ "from": path, "to": to }),
                    format!("Could not rename {}", vault::name_of(&path)),
                    |this, answer, window, cx| this.renamed(&answer, window, cx),
                    window,
                    cx,
                );
            }
            Edit::BookmarkTitle { key } => {
                if !typed.trim().is_empty()
                    && let Some(workspace) = self.workspace(cx)
                {
                    favourites::retitle(&key, &typed, workspace, cx);
                }
            }
        }
    }

    /// After a rename or a move: the open folders follow a folder, and the row is selected.
    fn renamed(&mut self, answer: &str, window: &Window, cx: &mut Context<Self>) {
        match RenameReport::from_answer(answer) {
            Ok(report) => {
                self.open = vault::reopen(&self.open, &report.from, &report.to);
                self.reveal(&report.to);
                // Open Page tabs follow the page or folder (#656).
                super::page::follow_rename(&report, window, cx);
                cx.notify();
            }
            Err(error) => self.toast(
                format!("{BRAIN_RENAME}'s answer did not parse: {error}"),
                cx,
            ),
        }
    }

    /// A row dropped on a folder, a page (its folder) or the tree's empty space (the top).
    fn dropped(
        &self,
        dragged: &DraggedVaultEntry,
        onto: Option<&str>,
        window: &Window,
        cx: &Context<Self>,
    ) {
        let Some(Ok(tree)) = &self.tree else {
            return;
        };
        let Some(to) = vault::move_target(tree, &dragged.path, onto) else {
            return;
        };
        Self::write(
            BRAIN_RENAME,
            json!({ "from": dragged.path, "to": to }),
            format!("Could not move {}", dragged.name),
            |this, answer, window, cx| this.renamed(&answer, window, cx),
            window,
            cx,
        );
    }

    /// Delete asks first, naming the page, or the folder with its pages, and what Rusty does.
    fn delete(&self, path: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(Ok(tree)) = &self.tree else {
            return;
        };
        let Some(node) = tree.find(&path) else {
            return;
        };
        let (message, detail, tool, arguments) = if node.kind == NodeKind::Folder {
            let message = match node.pages {
                0 => format!("Delete the folder {path}?"),
                1 => format!("Delete the folder {path} and its page?"),
                pages => format!("Delete the folder {path} and its {pages} pages?"),
            };
            (
                message,
                "Rusty moves it into archive/ with everything in it, and leaves the links to \
                 its pages.",
                BRAIN_DELETE_FOLDER,
                json!({ "path": path }),
            )
        } else {
            (
                format!("Delete the page {path}?"),
                "Rusty moves it into archive/ and leaves the links to it.",
                BRAIN_DELETE_PAGE,
                json!({ "slug": path }),
            )
        };
        let answer = window.prompt(
            PromptLevel::Warning,
            &message,
            Some(detail),
            &["Delete", "Cancel"],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            if !matches!(answer.await, Ok(0)) {
                return;
            }
            this.update_in(cx, |_, window, cx| {
                Self::write(
                    tool,
                    arguments,
                    format!("Could not delete {path}"),
                    move |this, _, _, cx| {
                        if this.selected.as_deref() == Some(path.as_str()) {
                            this.selected = None;
                        }
                        cx.notify();
                    },
                    window,
                    cx,
                );
            })
            .log_err();
        })
        .detach();
    }

    /// The menu of a right-click: a folder's, a page's, or the tree's empty space's.
    fn deploy_menu(
        &mut self,
        target: Option<String>,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let kind = target.as_deref().and_then(|path| {
            self.rows
                .iter()
                .find(|row| row.path == path)
                .map(|row| row.kind)
        });
        if target.is_some() {
            self.selected.clone_from(&target);
        }
        let view = cx.entity().downgrade();
        let menu = ContextMenu::build(window, cx, move |menu, _, _| match (target, kind) {
            (Some(path), Some(NodeKind::Folder)) => menu
                .entry(
                    "New Page",
                    None,
                    on(&view, {
                        let folder = path.clone();
                        move |this, window, cx| {
                            let edit = Edit::NewPage {
                                folder: folder.clone(),
                            };
                            this.start_edit(edit, window, cx);
                        }
                    }),
                )
                .entry(
                    "New Folder",
                    None,
                    on(&view, {
                        let parent = path.clone();
                        move |this, window, cx| {
                            let edit = Edit::NewFolder {
                                parent: parent.clone(),
                            };
                            this.start_edit(edit, window, cx);
                        }
                    }),
                )
                .separator()
                .entry("Rename", None, rename_entry(&view, &path))
                .entry("Delete", None, delete_entry(&view, &path)),
            (Some(path), Some(_)) => menu
                .entry(
                    "Open",
                    None,
                    on(&view, {
                        let slug = path.clone();
                        move |this, window, cx| this.open(&slug, false, true, window, cx)
                    }),
                )
                .separator()
                .entry("Rename", None, rename_entry(&view, &path))
                .entry("Delete", None, delete_entry(&view, &path)),
            _ => menu
                .entry(
                    "New Page",
                    None,
                    on(&view, |this, window, cx| {
                        let edit = Edit::NewPage {
                            folder: String::new(),
                        };
                        this.start_edit(edit, window, cx);
                    }),
                )
                .entry(
                    "New Folder",
                    None,
                    on(&view, |this, window, cx| {
                        let edit = Edit::NewFolder {
                            parent: String::new(),
                        };
                        this.start_edit(edit, window, cx);
                    }),
                )
                .separator()
                .entry("Refresh", None, |_, cx| super::reread_vault(cx)),
        });
        let subscription = cx.subscribe_in(&menu, window, |this, _, _: &DismissEvent, _, cx| {
            this.menu = None;
            cx.notify();
        });
        window.focus(&menu.focus_handle(cx), cx);
        self.menu = Some((menu, position, subscription));
        cx.notify();
    }

    fn render_fixed_row(cx: &Context<Self>) -> impl IntoElement {
        h_flex()
            .w_full()
            .flex_none()
            .gap_1()
            .px_2()
            .py_1()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                IconButton::new("marley-brain-today", IconName::Notepad)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text("Today's Note"))
                    .on_click(cx.listener(|_, _, window, cx| Self::today(window, cx))),
            )
            .child(
                IconButton::new("marley-brain-graph", IconName::GitGraph)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text("Graph"))
                    .on_click(cx.listener(|this, _, window, cx| this.open_graph(window, cx))),
            )
            .child(
                IconButton::new("marley-brain-tasks", IconName::ListTodo)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text("Tasks"))
                    .on_click(cx.listener(|this, _, window, cx| this.open_tasks(window, cx))),
            )
            .child(
                IconButton::new("marley-brain-decisions", IconName::CheckDouble)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text("Decisions"))
                    .on_click(cx.listener(|this, _, window, cx| this.open_decisions(window, cx))),
            )
            .child(
                IconButton::new("marley-brain-memory", IconName::Book)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text("Memory"))
                    .on_click(cx.listener(|this, _, window, cx| this.open_memory(window, cx))),
            )
    }

    fn render_search(&self, cx: &Context<Self>) -> impl IntoElement {
        h_flex()
            .w_full()
            .flex_none()
            .gap_2()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                Icon::new(IconName::MagnifyingGlass)
                    .size(IconSize::Small)
                    .color(Color::Muted),
            )
            .child(div().min_w_0().flex_1().child(self.search.clone()))
            .when(self.found.is_some(), |field| {
                field.child(
                    IconButton::new("marley-brain-search-clear", IconName::Close)
                        .icon_size(IconSize::Small)
                        .tooltip(Tooltip::text("Clear Search"))
                        .on_click(cx.listener(|this, _, window, cx| this.clear_search(window, cx))),
                )
            })
    }

    /// What shows in the list's place: a search under way, failed or empty, the vault being
    /// read, or a failed read.
    fn message(&self) -> Option<(SharedString, Color)> {
        match (&self.found, &self.tree) {
            (Some(Found { hits: None, .. }), _) => Some((
                SharedString::new_static("Searching the brain…"),
                Color::Muted,
            )),
            (
                Some(Found {
                    hits: Some(Err(error)),
                    ..
                }),
                _,
            )
            | (None, Some(Err(error))) => Some((error.clone(), Color::Error)),
            (
                Some(Found {
                    query,
                    hits: Some(Ok(hits)),
                }),
                _,
            ) if hits.is_empty() => {
                Some((format!("No page matches {query}.").into(), Color::Muted))
            }
            (None, None) => Some((SharedString::new_static("Reading the vault…"), Color::Muted)),
            _ => None,
        }
    }

    fn render_body(&self, cx: &Context<Self>) -> impl IntoElement {
        let body = div()
            .id("marley-brain-body")
            .flex_1()
            .min_h_0()
            .w_full()
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    // The rail's focus on a mouse-down would take the keyboard from the menu
                    // (L-600).
                    cx.stop_propagation();
                    window.prevent_default();
                    this.deploy_menu(None, event.position, window, cx);
                }),
            )
            .on_drop(
                cx.listener(|this, dragged: &DraggedVaultEntry, window, cx| {
                    this.dropped(dragged, None, window, cx);
                }),
            );
        if let Some((message, color)) = self.message() {
            return body.child(
                div()
                    .px_3()
                    .py_2()
                    .child(Label::new(message).size(LabelSize::Small).color(color)),
            );
        }
        let list = uniform_list(
            "marley-brain-lines",
            self.lines.len(),
            cx.processor(|this, range: Range<usize>, _window, cx| this.render_lines(range, cx)),
        )
        .track_scroll(&self.scroll)
        .size_full()
        .when(self.found.is_none(), |list| {
            list.with_decoration(
                indent_guides(INDENT, IndentGuideColors::panel(cx)).with_compute_indents_fn(
                    cx.entity(),
                    |this, range, _, _| {
                        this.lines
                            .get(range)
                            .unwrap_or_default()
                            .iter()
                            .map(|line| this.line_depth(*line))
                            .collect::<SmallVec<[usize; 64]>>()
                    },
                ),
            )
        });
        body.child(list)
    }

    fn render_lines(&self, range: Range<usize>, cx: &Context<Self>) -> Vec<AnyElement> {
        range
            .filter_map(|at| self.lines.get(at).copied())
            .map(|line| match line {
                Line::Row(at) => self.render_row(at, cx),
                Line::Draft(depth) => self.render_draft(depth),
                Line::Hit(at) => self.render_hit(at, cx),
            })
            .collect()
    }

    fn render_row(&self, at: usize, cx: &Context<Self>) -> AnyElement {
        let Some(row) = self.rows.get(at) else {
            return div().into_any_element();
        };
        let path = row.path.clone();
        let folder = row.kind == NodeKind::Folder;
        let renaming = match &self.editing {
            Some(Editing {
                edit: Edit::Rename { path: renamed },
                editor,
                ..
            }) if *renamed == row.path => Some(editor.clone()),
            _ => None,
        };
        let icon = match (folder, row.open) {
            (true, true) => IconName::FolderOpen,
            (true, false) => IconName::Folder,
            (false, _) => IconName::FileTextOutlined,
        };
        let item = ListItem::new("marley-brain-row")
            .indent_level(row.depth)
            .indent_step_size(INDENT)
            .spacing(ListItemSpacing::Dense)
            .toggle_state(self.selected.as_deref() == Some(row.path.as_str()))
            .start_slot(start_slot(folder.then_some(row.open), icon))
            .when(folder, |item| {
                item.end_slot(
                    Label::new(row.pages.to_string())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                )
            })
            .child(renaming.map_or_else(
                || Label::new(row.name.clone()).truncate().into_any_element(),
                |editor| div().flex_1().child(editor).into_any_element(),
            ))
            .on_click(cx.listener({
                let path = path.clone();
                move |this, event: &ClickEvent, window, cx| {
                    this.clicked(&path, event.click_count(), window, cx);
                }
            }))
            .on_secondary_mouse_down(cx.listener({
                let path = path.clone();
                move |this, event: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    window.prevent_default();
                    this.deploy_menu(Some(path.clone()), event.position, window, cx);
                }
            }));
        let dragged = DraggedVaultEntry {
            path: path.clone(),
            name: row.name.clone().into(),
        };
        div()
            .id(ElementId::Name(format!("marley-brain-{path}").into()))
            .w_full()
            .on_drag(dragged, |dragged, _, _, cx| cx.new(|_| dragged.clone()))
            .drag_over::<DraggedVaultEntry>(|style, _, _, cx| {
                style.bg(cx.theme().colors().drop_target_background)
            })
            .on_drop(
                cx.listener(move |this, dragged: &DraggedVaultEntry, window, cx| {
                    this.dropped(dragged, Some(&path), window, cx);
                }),
            )
            .child(item)
            .into_any_element()
    }

    fn render_draft(&self, depth: usize) -> AnyElement {
        let Some(editing) = &self.editing else {
            return div().into_any_element();
        };
        let icon = match editing.edit {
            Edit::NewFolder { .. } => IconName::Folder,
            Edit::NewPage { .. } | Edit::Rename { .. } | Edit::BookmarkTitle { .. } => {
                IconName::FileTextOutlined
            }
        };
        ListItem::new("marley-brain-draft")
            .indent_level(depth)
            .indent_step_size(INDENT)
            .spacing(ListItemSpacing::Dense)
            .toggle_state(true)
            .start_slot(start_slot(None, icon))
            .child(div().flex_1().child(editing.editor.clone()))
            .into_any_element()
    }

    fn render_hit(&self, at: usize, cx: &Context<Self>) -> AnyElement {
        let Some(hit) = self.hits().and_then(|hits| hits.get(at)) else {
            return div().into_any_element();
        };
        let slug = hit.slug.clone();
        let title = if hit.title.is_empty() {
            vault::name_of(&slug).to_string()
        } else {
            hit.title.clone()
        };
        ListItem::new(ElementId::Name(format!("marley-brain-hit-{slug}").into()))
            .spacing(ListItemSpacing::Dense)
            .toggle_state(self.selected.as_deref() == Some(slug.as_str()))
            .start_slot(
                Icon::new(IconName::FileTextOutlined)
                    .size(IconSize::Small)
                    .color(Color::Muted),
            )
            .child(
                h_flex()
                    .min_w_0()
                    .gap_2()
                    .child(Label::new(title).truncate())
                    .child(
                        Label::new(slug.clone())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .truncate(),
                    ),
            )
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                this.clicked(&slug, event.click_count(), window, cx);
            }))
            .into_any_element()
    }
}

/// A row's start: a folder's chevron, or the same width blank for a page, then its icon.
fn start_slot(open: Option<bool>, icon: IconName) -> impl IntoElement {
    h_flex()
        .gap_1()
        .child(open.map_or_else(
            || div().w(IconSize::XSmall.rems()).into_any_element(),
            |open| {
                Icon::new(if open {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                })
                .size(IconSize::XSmall)
                .color(Color::Muted)
                .into_any_element()
            },
        ))
        .child(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
}

/// Rusty's bookmarks above the tree (#662).
impl BrainView {
    /// The Favourites group: Rusty's bookmarks in its order, drawn only while it holds one.
    fn render_favourites(&self, cx: &Context<Self>) -> Option<AnyElement> {
        let list = favourites::list(cx);
        if list.is_empty() {
            return None;
        }
        let rows: Vec<AnyElement> = list
            .iter()
            .enumerate()
            .map(|(index, bookmark)| self.render_favourite(index, bookmark, cx))
            .collect();
        Some(
            v_flex()
                .flex_none()
                .w_full()
                .pb_1()
                .border_b_1()
                .border_color(cx.theme().colors().border_variant)
                // `ListSubHeader` grows in a column (F-659).
                .child(
                    div()
                        .flex_none()
                        .child(ListSubHeader::new("Favourites").inset(true)),
                )
                .children(rows)
                .into_any_element(),
        )
    }

    fn render_favourite(
        &self,
        index: usize,
        bookmark: &Bookmark,
        cx: &Context<Self>,
    ) -> AnyElement {
        let key = bookmark.key();
        let renaming = match &self.editing {
            Some(Editing {
                edit: Edit::BookmarkTitle { key: edited },
                editor,
                ..
            }) if *edited == key => Some(editor.clone()),
            _ => None,
        };
        let (icon, detail) = match bookmark.kind() {
            BookmarkKind::Folder => (IconName::Folder, bookmark.path.clone()),
            BookmarkKind::Search => (IconName::MagnifyingGlass, bookmark.query.clone()),
            BookmarkKind::Heading => (
                IconName::Hash,
                format!("{} › {}", bookmark.path, bookmark.heading),
            ),
            BookmarkKind::File | BookmarkKind::Other(_) => {
                (IconName::FileTextOutlined, bookmark.path.clone())
            }
        };
        let title = SharedString::from(bookmark.shown_title().to_string());
        ListItem::new(("marley-brain-favourite", index))
            .spacing(ListItemSpacing::Dense)
            .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
            .tooltip(Tooltip::text(detail))
            .child(renaming.map_or_else(
                || Label::new(title).truncate().into_any_element(),
                |editor| div().flex_1().child(editor).into_any_element(),
            ))
            .on_click(cx.listener({
                let bookmark = bookmark.clone();
                move |this, event: &ClickEvent, window, cx| {
                    this.favourite_clicked(&bookmark, event.click_count(), window, cx);
                }
            }))
            .on_secondary_mouse_down(cx.listener({
                let bookmark = bookmark.clone();
                move |this, event: &MouseDownEvent, window, cx| {
                    // The rail's focus on a mouse-down would take the keyboard from the menu
                    // (L-600).
                    cx.stop_propagation();
                    window.prevent_default();
                    this.deploy_favourite_menu(bookmark.clone(), event.position, window, cx);
                }
            }))
            .into_any_element()
    }

    /// A click on a favourite: a page opens as a tree row's does, a folder opens in the tree, a
    /// search runs its query, and a heading opens its page at that heading.
    fn favourite_clicked(
        &mut self,
        bookmark: &Bookmark,
        click_count: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match bookmark.kind() {
            BookmarkKind::Folder => {
                if self.found.is_some() {
                    self.clear_search(window, cx);
                }
                self.open.insert(bookmark.path.clone());
                self.reveal(&bookmark.path);
                cx.notify();
            }
            BookmarkKind::Search => {
                let query = bookmark.query.clone();
                self.search
                    .update(cx, |editor, cx| editor.set_text(query.clone(), window, cx));
                self.search_for(query, cx);
            }
            BookmarkKind::Heading => {
                if let Some(workspace) = self.workspace(cx) {
                    super::page::open_at_heading_later(
                        workspace,
                        bookmark.path.clone(),
                        bookmark.heading.clone(),
                        window,
                        cx,
                    );
                }
            }
            BookmarkKind::File | BookmarkKind::Other(_) => {
                self.selected = Some(bookmark.path.clone());
                let preview = click_count == 1 && preview_on_click(cx);
                self.open(&bookmark.path, preview, click_count > 1, window, cx);
                cx.notify();
            }
        }
    }

    /// A favourite's right-click menu: Rename… edits its title in place; Remove asks Rusty to
    /// drop it.
    fn deploy_favourite_menu(
        &mut self,
        bookmark: Bookmark,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let view = cx.entity().downgrade();
        let key = bookmark.key();
        let menu = ContextMenu::build(window, cx, move |menu, _, _| {
            menu.entry(
                "Rename…",
                None,
                on(&view, move |this, window, cx| {
                    let edit = Edit::BookmarkTitle { key: key.clone() };
                    this.start_edit(edit, window, cx);
                }),
            )
            .entry(
                "Remove",
                None,
                on(&view, move |this, _, cx| {
                    if let Some(workspace) = this.workspace(cx) {
                        favourites::write(&BookmarkWrite::Remove(bookmark.clone()), workspace, cx);
                    }
                }),
            )
        });
        let subscription = cx.subscribe_in(&menu, window, |this, _, _: &DismissEvent, _, cx| {
            this.menu = None;
            cx.notify();
        });
        window.focus(&menu.focus_handle(cx), cx);
        self.menu = Some((menu, position, subscription));
        cx.notify();
    }

    /// The workspace the rail shows, for the openers and the toasts.
    fn workspace(&self, cx: &App) -> Option<WeakEntity<Workspace>> {
        Some(
            self.multi_workspace
                .upgrade()?
                .read(cx)
                .workspace()
                .downgrade(),
        )
    }
}

/// A menu entry's handler that runs `action` on the Brain view.
fn on(
    view: &WeakEntity<BrainView>,
    action: impl Fn(&mut BrainView, &mut Window, &mut Context<BrainView>) + 'static,
) -> impl Fn(&mut Window, &mut App) + 'static {
    let view = view.clone();
    move |window, cx| {
        view.update(cx, |this, cx| action(this, window, cx))
            .log_err();
    }
}

fn rename_entry(
    view: &WeakEntity<BrainView>,
    path: &str,
) -> impl Fn(&mut Window, &mut App) + 'static {
    let path = path.to_string();
    on(view, move |this, window, cx| {
        let edit = Edit::Rename { path: path.clone() };
        this.start_edit(edit, window, cx);
    })
}

fn delete_entry(
    view: &WeakEntity<BrainView>,
    path: &str,
) -> impl Fn(&mut Window, &mut App) + 'static {
    let path = path.to_string();
    on(view, move |this, window, cx| {
        this.delete(path.clone(), window, cx);
    })
}

impl Focusable for BrainView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BrainView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("marley-brain")
            // Zed binds left and right for lists only in the `menu` context (AD-453).
            .key_context("MarleyBrain menu")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_first))
            .on_action(cx.listener(Self::select_last))
            .on_action(cx.listener(Self::select_child))
            .on_action(cx.listener(Self::select_parent))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .flex_1()
            .min_h_0()
            .w_full()
            .child(Self::render_fixed_row(cx))
            .child(self.render_search(cx))
            .children(self.render_favourites(cx))
            .child(self.render_body(cx))
            .children(self.menu.as_ref().map(|(menu, position, _)| {
                deferred(
                    anchored()
                        .position(*position)
                        .anchor(Anchor::TopLeft)
                        .child(menu.clone()),
                )
                .with_priority(1)
            }))
    }
}
