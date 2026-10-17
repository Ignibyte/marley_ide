//! A block's output as a table (#803).
//!
//! Each block is read once, as it finishes, off the main thread: a command that reads one
//! tabular file (`cat data.csv`, `head -n 50 big.csv`) offers that file, and otherwise output
//! that `marley_terminal::tables` reads as a table offers itself, unless its cells carry colors
//! or styles. What was found is kept by terminal and block. The block's Table chip and its menu's
//! Open Output as Table open it in a tab in the terminal's pane: a file in Zed's table preview,
//! whole; output in a `BlockTable`, Zed's table view filled with the parsed cells. A second open
//! brings the tab forward. The block's text stays as the program printed it.

use std::path::{Path, PathBuf};

use collections::HashMap;
use editor::Editor;
use fs::Fs;
use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, EntityId, EventEmitter, FocusHandle,
    Focusable, Global, IntoElement as _, ParentElement as _, Render, Styled as _, Subscription,
    TaskExt as _, WeakEntity, Window, div,
};
use marley_terminal::tables::{self, SAMPLE_LINES, Table, TableShape};
use marley_terminal::{AnchoredBlock, BlockState};
use tabular_data_preview::TableView;
use tabular_data_preview::TabularDataPreviewPane;
use tabular_data_preview::types::{LineNumber, TableCell, TableLikeContent};
use terminal::Terminal;
use terminal_view::{MarleyFooterContext, TerminalView};
use ui::table_row::TableRow;
use ui::{
    Button, ButtonCommon as _, ButtonStyle, Clickable as _, ContextMenu, Icon, IconName, LabelSize,
    SharedString, TintColor, Tooltip,
};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

/// The longest command a table's tab title shows.
const TITLE_LIMIT: usize = 40;

/// What each terminal's finished blocks offer as tables.
#[derive(Default)]
struct BlockTables {
    by_block: HashMap<(EntityId, usize), Found>,
    /// The index of the next block of each terminal to read.
    next: HashMap<EntityId, usize>,
    /// The terminal each view shows and the watch on it: Zed's task rerun gives a view a new
    /// terminal.
    watched: HashMap<EntityId, (EntityId, Subscription)>,
}

impl Global for BlockTables {}

impl BlockTables {
    /// Lets go of what a terminal no view shows any more had.
    fn forget(&mut self, terminal: EntityId) {
        self.next.remove(&terminal);
        self.by_block.retain(|(id, _), _| *id != terminal);
    }
}

/// What a block offers, and the tab that shows it once opened.
#[derive(Clone)]
struct Found {
    source: Source,
    opened: Option<Opened>,
}

/// Where a block's table comes from.
#[derive(Clone)]
enum Source {
    /// The one tabular file its command read.
    File(PathBuf),
    /// Its own output, with the shape its sample had.
    Output(TableShape),
}

/// The tab a block's table opened in.
#[derive(Clone)]
enum Opened {
    Preview(WeakEntity<TabularDataPreviewPane>),
    Table(WeakEntity<BlockTable>),
}

/// A finished block to read: its index, command, folder, and its output's first lines when they
/// are plain text still held.
struct Unread {
    index: usize,
    command: String,
    folder: Option<String>,
    output: Option<(String, bool)>,
}

/// The toasts of a table that would not open.
struct TableToast;

/// Reads each terminal's blocks as they finish. [`crate::init`] calls it once, before any window
/// opens.
pub fn init(cx: &mut App) {
    cx.set_global(BlockTables::default());
    cx.observe_new(
        |view: &mut TerminalView, _, cx: &mut Context<TerminalView>| {
            watch(view, cx);
            cx.observe_self(|view, cx| {
                let watched = cx
                    .try_global::<BlockTables>()
                    .and_then(|tables| tables.watched.get(&cx.entity_id()))
                    .map(|(terminal, _)| *terminal);
                if watched != Some(view.terminal().entity_id()) {
                    watch(view, cx);
                }
            })
            .detach();
            let view_id = cx.entity_id();
            cx.on_release(move |_, cx| {
                let tables = cx.default_global::<BlockTables>();
                if let Some((terminal, _)) = tables.watched.remove(&view_id) {
                    tables.forget(terminal);
                }
            })
            .detach();
        },
    )
    .detach();
}

/// Watches the terminal `view` shows, its blocks read as each finishes, in place of the one it
/// showed before.
fn watch(view: &TerminalView, cx: &mut Context<TerminalView>) {
    let terminal = view.terminal().clone();
    let subscription = cx.observe(&terminal, |_, terminal, cx| {
        read_finished(&terminal, cx);
    });
    let view_id = cx.entity_id();
    let tables = cx.default_global::<BlockTables>();
    if let Some((earlier, _)) = tables
        .watched
        .insert(view_id, (terminal.entity_id(), subscription))
    {
        tables.forget(earlier);
    }
    read_finished(&terminal, cx);
}

/// Reads the blocks of `terminal` that finished since the last read, off the main thread, and
/// keeps what each offers. A terminal notifies at every output, so this returns at once when no
/// block finished.
fn read_finished(terminal: &Entity<Terminal>, cx: &mut Context<TerminalView>) {
    let id = terminal.entity_id();
    // A block from before a restart is history (#764).
    let next = cx
        .try_global::<BlockTables>()
        .and_then(|tables| tables.next.get(&id).copied())
        .unwrap_or(0)
        .max(terminal.read(cx).marley_anchored().restored_count());
    let (unread, read_to) = {
        let terminal = terminal.read(cx);
        let mut read_to = next;
        let mut unread = Vec::new();
        let newer = terminal
            .blocks()
            .iter()
            .rev()
            .take_while(|block| block.index >= next)
            .collect::<Vec<_>>();
        for block in newer.into_iter().rev() {
            if block.state != BlockState::Finished {
                break;
            }
            read_to = block.index + 1;
            unread.push(Unread {
                index: block.index,
                command: block.command.clone(),
                folder: block.prompt.pwd.clone(),
                output: sample(terminal, block),
            });
        }
        (unread, read_to)
    };
    if read_to == next {
        return;
    }
    cx.default_global::<BlockTables>().next.insert(id, read_to);
    let fs = <dyn Fs>::global(cx);
    cx.spawn(async move |view, cx| {
        let mut found = Vec::new();
        for block in unread {
            let file = block
                .folder
                .as_deref()
                .and_then(|folder| tables::tabular_file(&block.command, Path::new(folder)));
            if let Some(file) = file
                && fs
                    .metadata(&file)
                    .await
                    .ok()
                    .flatten()
                    .is_some_and(|metadata| !metadata.is_dir && !metadata.is_fifo)
            {
                found.push((block.index, Source::File(file)));
                continue;
            }
            let Some((text, cut)) = block.output else {
                continue;
            };
            let shape = smol::unblock(move || tables::detect(&text)).await;
            if let Some(mut shape) = shape {
                shape.more |= cut;
                found.push((block.index, Source::Output(shape)));
            }
        }
        if found.is_empty() {
            return;
        }
        cx.update(|cx| {
            let tables = cx.default_global::<BlockTables>();
            for (index, source) in found {
                tables.by_block.insert(
                    (id, index),
                    Found {
                        source,
                        opened: None,
                    },
                );
            }
        });
        view.update(cx, |_, cx| cx.notify()).log_err();
    })
    .detach();
}

/// The first [`SAMPLE_LINES`] lines of `block`'s output, and whether it has more, when its cells
/// are plain and still held.
fn sample(terminal: &Terminal, block: &AnchoredBlock) -> Option<(String, bool)> {
    let end = block.output_end?;
    if !terminal.block_output_kept(block) || terminal.block_output_styled(block) {
        return None;
    }
    let lines = u64::try_from(SAMPLE_LINES).unwrap_or(u64::MAX);
    let cut = end.saturating_sub(block.output_start) > lines;
    let mut first = block.clone();
    first.output_end = Some(end.min(block.output_start.saturating_add(lines)));
    Some((terminal.block_output(&first)?, cut))
}

/// `menu` with Open Output as Table when the block at `index` of the menu's terminal offers a
/// table.
pub(crate) fn with_menu_item(
    menu: ContextMenu,
    context: &MarleyFooterContext,
    index: usize,
    cx: &App,
) -> ContextMenu {
    if found(context.terminal, index, cx).is_none() {
        return menu;
    }
    let view = context.view.clone();
    menu.entry("Open Output as Table", None, move |window, cx| {
        if let Some(view) = view.upgrade() {
            open(&view, index, window, cx);
        }
    })
}

/// What `terminal`'s block at `index` offers, while its output is still held when that is the
/// source.
fn found(terminal: &Entity<Terminal>, index: usize, cx: &App) -> Option<Found> {
    let found = cx
        .try_global::<BlockTables>()?
        .by_block
        .get(&(terminal.entity_id(), index))?
        .clone();
    if matches!(found.source, Source::Output(_)) {
        let terminal = terminal.read(cx);
        let block = terminal
            .blocks()
            .iter()
            .find(|block| block.index == index)?;
        if !terminal.block_output_kept(block) {
            return None;
        }
    }
    Some(found)
}

/// The Table chip of each block starting on screen that offers a table, part of the element's
/// `MarleyBlockChip` hook (`bookmarks`).
pub(crate) fn chip(
    view: &Entity<TerminalView>,
    terminal: &Entity<Terminal>,
    index: usize,
    cx: &App,
) -> Option<AnyElement> {
    let found = found(terminal, index, cx)?;
    let tooltip = match &found.source {
        Source::File(path) => format!(
            "Open {} as a table",
            path.file_name()
                .map_or_else(|| path.to_string_lossy(), |name| name.to_string_lossy())
        ),
        Source::Output(shape) => format!(
            "Open the output as a table: {} columns, {}{} rows",
            shape.columns,
            if shape.more { "over " } else { "" },
            shape.rows
        ),
    };
    let view = view.clone();
    Some(
        Button::new(("marley-block-table", index), "Table")
            .style(ButtonStyle::Tinted(TintColor::Accent))
            .label_size(LabelSize::XSmall)
            .tooltip(Tooltip::text(tooltip))
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                open(&view, index, window, cx);
            })
            .into_any_element(),
    )
}

/// Opens the table `view`'s block at `index` offers, after the event that asked: adding a tab
/// reads every item of the pane, the asking one among them (F-claude-733).
pub(crate) fn open(view: &Entity<TerminalView>, index: usize, window: &Window, cx: &mut App) {
    let view = view.clone();
    window.defer(cx, move |window, cx| open_now(&view, index, window, cx));
}

fn open_now(view: &Entity<TerminalView>, index: usize, window: &mut Window, cx: &mut App) {
    let Some(workspace) = view.read(cx).marley_workspace().upgrade() else {
        return;
    };
    let terminal = view.read(cx).terminal().clone();
    let Some(found) = found(&terminal, index, cx) else {
        return;
    };
    let shown = workspace.update(cx, |workspace, cx| match &found.opened {
        Some(Opened::Preview(tab)) => tab
            .upgrade()
            .is_some_and(|tab| workspace.activate_item(&tab, true, true, window, cx)),
        Some(Opened::Table(tab)) => tab
            .upgrade()
            .is_some_and(|tab| workspace.activate_item(&tab, true, true, window, cx)),
        None => false,
    });
    if shown {
        return;
    }
    let key = (terminal.entity_id(), index);
    match found.source {
        Source::File(path) => open_file(&workspace, view, key, path, window, cx),
        Source::Output(_) => open_output(&workspace, view, &terminal, key, window, cx),
    }
}

/// Opens `path` in Zed's table preview in the terminal's pane: the preview reads an editor, so a
/// buffer of the file gets one that no pane shows.
fn open_file(
    workspace: &Entity<Workspace>,
    view: &Entity<TerminalView>,
    key: (EntityId, usize),
    path: PathBuf,
    window: &Window,
    cx: &mut App,
) {
    let project = workspace.read(cx).project().clone();
    let buffer = project.update(cx, |project, cx| project.open_local_buffer(&path, cx));
    let workspace = workspace.downgrade();
    let view = view.downgrade();
    window
        .spawn(cx, async move |cx| {
            let buffer = match buffer.await {
                Ok(buffer) => buffer,
                Err(error) => {
                    let message = format!("Could not open {}: {error}", path.display());
                    workspace.update(cx, |workspace, cx| show_toast(workspace, message, cx))?;
                    return anyhow::Ok(());
                }
            };
            workspace.update_in(cx, |workspace, window, cx| {
                let editor = cx.new(|cx| Editor::for_buffer(buffer, Some(project), window, cx));
                let pane = view
                    .upgrade()
                    .and_then(|view| workspace.pane_for(&view))
                    .unwrap_or_else(|| workspace.active_pane().clone());
                TabularDataPreviewPane::open_preview_in_pane(editor, pane.clone(), window, cx);
                let preview = pane
                    .read(cx)
                    .active_item()
                    .and_then(|item| item.downcast::<TabularDataPreviewPane>());
                if let Some(preview) = preview {
                    remember(key, Opened::Preview(preview.downgrade()), cx);
                }
            })
        })
        .detach_and_log_err(cx);
}

/// Opens the block's output, parsed whole off the main thread, in a `BlockTable` in the
/// terminal's pane.
fn open_output(
    workspace: &Entity<Workspace>,
    view: &Entity<TerminalView>,
    terminal: &Entity<Terminal>,
    key: (EntityId, usize),
    window: &mut Window,
    cx: &mut App,
) {
    let (command, text) = {
        let terminal = terminal.read(cx);
        let Some(block) = terminal.blocks().iter().find(|block| block.index == key.1) else {
            return;
        };
        (block.command.clone(), terminal.block_output(block))
    };
    let Some(text) = text else {
        workspace.update(cx, |workspace, cx| {
            show_toast(
                workspace,
                "The block's output has left the scrollback".to_string(),
                cx,
            );
        });
        return;
    };
    let table_view = cx.new(|cx| {
        let mut table = TableView::new(window, cx);
        table.set_loading(true, cx);
        table
    });
    let tab = cx.new(|cx| BlockTable {
        title: title_of(&command),
        command: SharedString::from(command),
        _table_changed: cx.observe(&table_view, |_, _, cx| cx.notify()),
        table: table_view.clone(),
    });
    workspace.update(cx, |workspace, cx| match workspace.pane_for(view) {
        Some(pane) => pane.update(cx, |pane, cx| {
            pane.add_item(Box::new(tab.clone()), true, true, None, window, cx);
        }),
        None => {
            workspace.add_item_to_center(Box::new(tab.clone()), window, cx);
        }
    });
    remember(key, Opened::Table(tab.downgrade()), cx);
    cx.spawn(async move |cx| {
        let table = smol::unblock(move || tables::parse(&text)).await;
        table_view.update(cx, |table_view, cx| match table {
            Some(table) => table_view.set_contents(content_of(table), cx),
            None => table_view.set_loading(false, cx),
        });
    })
    .detach();
}

/// Keeps the tab `key`'s table opened in, so the next open brings it forward.
fn remember(key: (EntityId, usize), opened: Opened, cx: &mut App) {
    if let Some(found) = cx.default_global::<BlockTables>().by_block.get_mut(&key) {
        found.opened = Some(opened);
    }
}

fn show_toast(workspace: &mut Workspace, message: String, cx: &mut Context<Workspace>) {
    workspace.show_toast(
        Toast::new(NotificationId::unique::<TableToast>(), message),
        cx,
    );
}

/// `command` cut to [`TITLE_LIMIT`] characters, for a tab's title.
fn title_of(command: &str) -> SharedString {
    let command = command.trim();
    if command.chars().count() <= TITLE_LIMIT {
        return SharedString::from(command.to_string());
    }
    let cut: String = command.chars().take(TITLE_LIMIT - 1).collect();
    SharedString::from(format!("{cut}…"))
}

/// `table`'s cells as Zed's table view takes them; every row is as wide as the header.
fn content_of(table: Table) -> TableLikeContent {
    let width = table.headers.len();
    let row_of = |cells: Vec<String>| {
        let cells = cells
            .into_iter()
            .map(|text| TableCell::Generated(SharedString::from(text)))
            .collect();
        TableRow::try_from_vec(cells, width).log_err()
    };
    let rows: Vec<TableRow<TableCell>> = table.rows.into_iter().filter_map(row_of).collect();
    TableLikeContent {
        number_of_cols: width,
        headers: row_of(table.headers)
            .unwrap_or_else(|| TableRow::from_vec(vec![TableCell::Virtual; width], width)),
        line_numbers: (1..=rows.len()).map(LineNumber::Line).collect(),
        rows,
    }
}

/// A block's output as a table: Zed's table view over the parsed cells.
pub(crate) struct BlockTable {
    title: SharedString,
    command: SharedString,
    table: Entity<TableView>,
    _table_changed: Subscription,
}

impl Focusable for BlockTable {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.table.read(cx).focus_handle(cx)
    }
}

impl EventEmitter<ItemEvent> for BlockTable {}

impl Render for BlockTable {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div().size_full().child(self.table.clone())
    }
}

impl Item for BlockTable {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        self.title.clone()
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Table))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::from(format!(
            "The output of {} as a table",
            self.command
        )))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}
