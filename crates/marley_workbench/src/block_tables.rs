//! A block's output as a table (#803).
//!
//! Each block is read once, as it finishes, off the main thread: a command that reads one
//! tabular file (`cat data.csv`, `head -n 50 big.csv`) offers that file, and otherwise output
//! that `marley_terminal::tables` reads as a table offers itself, unless its cells carry colors
//! or styles. What was found is kept by terminal and block. The block's Table chip and its menu's
//! Open Output as Table open it in a tab in the terminal's pane: a file in Zed's table preview,
//! whole; output in a `BlockTable`, Zed's table view filled with the parsed cells. A second open
//! brings the tab forward. The block's text stays as the program printed it.
//!
//! Since #804 a block of delimited text whose every output row is one record (the header and the
//! rows fill its rows, none wrapped) can also show its table in the terminal, over those rows: the
//! block's Inline toggle, or `marley.block_tables: in_place` as the block finishes. The element
//! draws one table row per terminal row through `MarleyBlockBody`, and the terminal leaves those
//! rows' cells out, so copy, find and links still read the text. A row holding a search match is
//! marked, and a selection being dragged shows the text.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use collections::{HashMap, HashSet};
use editor::Editor;
use fs::Fs;
use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, EntityId, EventEmitter, FocusHandle,
    Focusable, FontWeight, Global, IntoElement as _, ParentElement as _, Pixels, Render,
    Styled as _, Subscription, TaskExt as _, WeakEntity, Window, div,
};
use marley_terminal::tables::{self, SAMPLE_LINES, Table, TableKind, TableShape};
use marley_terminal::{AnchoredBlock, BlockState, OutputRows};
use settings::{MarleyBlockTables, Settings as _};
use tabular_data_preview::TableView;
use tabular_data_preview::TabularDataPreviewPane;
use tabular_data_preview::types::{LineNumber, TableCell, TableLikeContent};
use terminal::Terminal;
use terminal::terminal_settings::TerminalSettings;
use terminal_view::{MarleyBlockBody, MarleyFooterContext, TerminalView};
use theme::ActiveTheme as _;
use theme_settings::ThemeSettings;
use ui::table_row::TableRow;
use ui::{
    Button, ButtonCommon as _, ButtonStyle, Clickable as _, Color, ContextMenu, FluentBuilder as _,
    Icon, IconName, Label, LabelCommon as _, LabelSize, SharedString, TintColor, Tooltip, h_flex,
    v_flex,
};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use crate::MarleySettings;

/// The longest command a table's tab title shows.
const TITLE_LIMIT: usize = 40;

/// The narrowest and the widest a column drawn in place starts, in cells.
const COLUMN_CELLS: (usize, usize) = (3, 40);

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

/// What a block offers, the tab that shows it once opened, and its table while drawn in place.
#[derive(Clone, Default)]
struct Found {
    /// The one tabular file its command read, which a tab opens in place of the output.
    file: Option<PathBuf>,
    /// The shape its output's sample had, when that reads as a table.
    shape: Option<TableShape>,
    /// The output's row count, when the table can be drawn over it: one record on each row.
    fits: Option<u64>,
    /// The table drawn over the block's rows, while it is (#804).
    inline: Option<Arc<Table>>,
    opened: Option<Opened>,
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
    /// The output's row count.
    rows: u64,
}

/// The toasts of a table that would not open.
struct TableToast;

/// Reads each terminal's blocks as they finish. [`crate::init`] calls it once, before any window
/// opens.
pub fn init(cx: &mut App) {
    cx.set_global(BlockTables::default());
    cx.set_global(MarleyBlockBody(Arc::new(body)));
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
                rows: block
                    .output_end
                    .unwrap_or(block.output_start)
                    .saturating_sub(block.output_start),
            });
        }
        (unread, read_to)
    };
    if read_to == next {
        return;
    }
    cx.default_global::<BlockTables>().next.insert(id, read_to);
    let fs = <dyn Fs>::global(cx);
    let in_place = MarleySettings::get_global(cx).block_tables == MarleyBlockTables::InPlace;
    cx.spawn(async move |view, cx| {
        let mut found = Vec::new();
        for block in unread {
            let mut offer = Found::default();
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
                offer.file = Some(file);
            }
            if let Some((text, cut)) = block.output {
                let rows = block.rows;
                let (shape, fits, inline) = smol::unblock(move || {
                    let Some(mut shape) = tables::detect(&text) else {
                        return (None, None, None);
                    };
                    shape.more |= cut;
                    let lines = u64::try_from(text.lines().count()).unwrap_or(u64::MAX);
                    let fits = (matches!(shape.kind, TableKind::Delimited(_))
                        && !shape.more
                        && lines == rows
                        && u64::try_from(shape.rows + 1).is_ok_and(|records| records == lines))
                    .then_some(rows);
                    let inline = fits
                        .filter(|_| in_place)
                        .and_then(|_| tables::parse(&text))
                        .map(Arc::new);
                    (Some(shape), fits, inline)
                })
                .await;
                offer.shape = shape;
                offer.fits = fits;
                offer.inline = inline;
            }
            if offer.file.is_some() || offer.shape.is_some() {
                found.push((block.index, offer));
            }
        }
        if found.is_empty() {
            return;
        }
        cx.update(|cx| {
            let tables = cx.default_global::<BlockTables>();
            for (index, offer) in found {
                tables.by_block.insert((id, index), offer);
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

/// What `terminal`'s block at `index` offers, while its output is still held when no file is.
fn found(terminal: &Entity<Terminal>, index: usize, cx: &App) -> Option<Found> {
    let found = cx
        .try_global::<BlockTables>()?
        .by_block
        .get(&(terminal.entity_id(), index))?
        .clone();
    if found.file.is_none() {
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
    let tooltip = match (&found.file, &found.shape) {
        (Some(path), _) => format!(
            "Open {} as a table",
            path.file_name()
                .map_or_else(|| path.to_string_lossy(), |name| name.to_string_lossy())
        ),
        (None, Some(shape)) => format!(
            "Open the output as a table: {} columns, {}{} rows",
            shape.columns,
            if shape.more { "over " } else { "" },
            shape.rows
        ),
        (None, None) => return None,
    };
    let open_view = view.clone();
    let table = Button::new(("marley-block-table", index), "Table")
        .style(ButtonStyle::Tinted(TintColor::Accent))
        .label_size(LabelSize::XSmall)
        .tooltip(Tooltip::text(tooltip))
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            open(&open_view, index, window, cx);
        });
    // A block whose rows each hold one record can show its table over them (#804).
    let toggle = found.fits.map(|_| {
        let shown = found.inline.is_some();
        let view = view.clone();
        Button::new(
            ("marley-block-table-inline", index),
            if shown { "Text" } else { "Inline" },
        )
        .style(ButtonStyle::Tinted(TintColor::Accent))
        .label_size(LabelSize::XSmall)
        .tooltip(Tooltip::text(if shown {
            "Show the block's text"
        } else {
            "Draw the table over the block's output"
        }))
        .on_click(move |_, _, cx| {
            cx.stop_propagation();
            toggle_inline(&view, index, cx);
        })
    });
    Some(
        h_flex()
            .gap_1()
            .child(table)
            .children(toggle)
            .into_any_element(),
    )
}

/// Draws the table of `view`'s block at `index` over its rows, parsed off the main thread, or
/// shows its text again.
fn toggle_inline(view: &Entity<TerminalView>, index: usize, cx: &mut App) {
    let terminal = view.read(cx).terminal().clone();
    let key = (terminal.entity_id(), index);
    let shown = cx
        .try_global::<BlockTables>()
        .and_then(|tables| tables.by_block.get(&key))
        .is_some_and(|found| found.inline.is_some());
    if shown {
        if let Some(found) = cx.default_global::<BlockTables>().by_block.get_mut(&key) {
            found.inline = None;
        }
        view.update(cx, |_, cx| cx.notify());
        return;
    }
    let text = {
        let terminal = terminal.read(cx);
        terminal
            .blocks()
            .iter()
            .find(|block| block.index == index)
            .and_then(|block| terminal.block_output(block))
    };
    let Some(text) = text else {
        return;
    };
    let view = view.downgrade();
    cx.spawn(async move |cx| {
        let Some(table) = smol::unblock(move || tables::parse(&text)).await else {
            return;
        };
        cx.update(|cx| {
            if let Some(found) = cx.default_global::<BlockTables>().by_block.get_mut(&key) {
                found.inline = Some(Arc::new(table));
            }
        });
        view.update(cx, |_, cx| cx.notify()).log_err();
    })
    .detach();
}

/// The element's `MarleyBlockBody` hook (#804): the table of a block drawn in place, for its
/// output rows on screen, one table row on each terminal row; none while the block's rows no
/// longer hold one record each (a width change rewrapped them) or a selection is being dragged.
fn body(
    _: &Entity<TerminalView>,
    terminal: &Entity<Terminal>,
    output: &OutputRows,
    line_height: Pixels,
    cx: &App,
) -> Option<AnyElement> {
    let found = cx
        .try_global::<BlockTables>()?
        .by_block
        .get(&(terminal.entity_id(), output.index))?;
    let table = found.inline.clone()?;
    let fits = found.fits?;
    let terminal = terminal.read(cx);
    if terminal.selection_started() {
        return None;
    }
    let block = terminal
        .blocks()
        .iter()
        .find(|block| block.index == output.index)?;
    if block.output_end?.saturating_sub(block.output_start) != fits {
        return None;
    }
    let content = terminal.last_content();
    let screen_top = i64::try_from(content.marley_screen_top).ok()?;
    let output_start = i64::try_from(block.output_start).ok()?;
    let marked: HashSet<usize> = terminal
        .matches
        .iter()
        .filter_map(|found| {
            usize::try_from(screen_top + i64::from(found.start().line) - output_start).ok()
        })
        .collect();
    let widths = column_cells(&table, content.terminal_bounds.num_columns());
    let cell_width = content.terminal_bounds.cell_width;
    let colors = cx.theme().colors();
    let (font_family, font_size) = terminal_font(cx);
    // Each row is placed at its own grid row's top: rows of a rounded height would drift from the
    // text rows under them, a fraction of a pixel each.
    let rows = (output.above..output.above + output.rows.len()).map(|line| {
        let cells = if line == 0 {
            Some(&table.headers)
        } else {
            table.rows.get(line - 1)
        };
        let row = f32::from(u16::try_from(line - output.above).unwrap_or(u16::MAX));
        h_flex()
            .absolute()
            .left_0()
            .right_0()
            .top(line_height * row)
            .h(line_height)
            .when(line == 0, |row| {
                row.bg(colors.surface_background)
                    .font_weight(FontWeight::BOLD)
            })
            .when(line > 0 && line % 2 == 0, |row| {
                row.bg(colors.ghost_element_hover)
            })
            .when(marked.contains(&line), |row| {
                row.bg(colors.search_match_background)
            })
            .children(
                cells
                    .into_iter()
                    .flatten()
                    .zip(&widths)
                    .map(|(text, width)| {
                        let cells = f32::from(u16::try_from(*width).unwrap_or(u16::MAX));
                        div()
                            .w(cell_width * (cells + 1.0))
                            .flex_none()
                            .pl(cell_width)
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(SharedString::from(text.clone()))
                    }),
            )
    });
    Some(
        div()
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(colors.terminal_background)
            .font_family(font_family)
            .text_size(font_size)
            .text_color(colors.text)
            .children(rows)
            .into_any_element(),
    )
}

/// Each column's width in cells: its widest cell between 3 and 40, shrunk in proportion when the
/// table is wider than the terminal's `columns`, a cell of padding before each.
fn column_cells(table: &Table, columns: usize) -> Vec<usize> {
    let (narrowest, widest) = COLUMN_CELLS;
    let mut widths: Vec<usize> = (0..table.headers.len())
        .map(|column| {
            std::iter::once(&table.headers)
                .chain(&table.rows)
                .filter_map(|row| row.get(column))
                .map(|cell| cell.chars().count())
                .max()
                .unwrap_or(0)
                .clamp(narrowest, widest)
        })
        .collect();
    let total: usize = widths.iter().map(|width| width + 1).sum();
    if total > columns && total > 0 {
        for width in &mut widths {
            *width = ((*width + 1) * columns / total)
                .saturating_sub(1)
                .max(narrowest);
        }
    }
    widths
}

/// The terminal's font family and size, as its element picks them.
fn terminal_font(cx: &App) -> (SharedString, Pixels) {
    let terminal = TerminalSettings::get_global(cx);
    let theme = ThemeSettings::get_global(cx);
    let family = terminal.font_family.as_ref().map_or_else(
        || theme.buffer_font.family.clone(),
        |family| SharedString::from(Arc::clone(&family.0)),
    );
    let size = terminal.font_size.map_or_else(
        || theme.buffer_font_size(cx),
        |size| theme_settings::adjusted_font_size(size, cx),
    );
    (family, size)
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
    match found.file {
        Some(path) => open_file(&workspace, view, Some(key), path, window, cx),
        None => open_output(&workspace, view, &terminal, key, window, cx),
    }
}

/// Opens `path` in Zed's table preview in the terminal's pane: the preview reads an editor, so a
/// buffer of the file gets one that no pane shows. A block's open, `key`, remembers the tab.
pub(crate) fn open_file(
    workspace: &Entity<Workspace>,
    view: &Entity<TerminalView>,
    key: Option<(EntityId, usize)>,
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
                if let (Some(key), Some(preview)) = (key, preview) {
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
    let tooltip = SharedString::from(format!("The output of {command} as a table"));
    let source = title_of(&command);
    let (tab, table_view) = add_table_tab(
        workspace,
        view,
        title_of(&command),
        tooltip,
        source,
        window,
        cx,
    );
    remember(key, Opened::Table(tab.downgrade()), cx);
    let tab = tab.downgrade();
    cx.spawn(async move |cx| {
        let table = smol::unblock(move || tables::parse(&text)).await;
        match table {
            Some(table) => {
                tab.update(cx, |tab, cx| tab.set_table(table, cx)).log_err();
            }
            None => table_view.update(cx, |table_view, cx| table_view.set_loading(false, cx)),
        }
    })
    .detach();
}

/// A new `BlockTable` titled `title`, its table loading, in the terminal's pane.
fn add_table_tab(
    workspace: &Entity<Workspace>,
    view: &Entity<TerminalView>,
    title: SharedString,
    tooltip: SharedString,
    source: SharedString,
    window: &mut Window,
    cx: &mut App,
) -> (Entity<BlockTable>, Entity<TableView>) {
    let table_view = cx.new(|cx| {
        let mut table = TableView::new(window, cx);
        table.set_loading(true, cx);
        table
    });
    let tab = cx.new(|cx| BlockTable {
        title,
        tooltip,
        facts: None,
        source,
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
    (tab, table_view)
}

/// Shows `table`, which `show` read (#805), in the table tab titled `title`, reused, else a new one
/// in the terminal's pane.
pub(crate) fn show_table(
    workspace: &Entity<Workspace>,
    view: &Entity<TerminalView>,
    title: SharedString,
    table: Table,
    window: &mut Window,
    cx: &mut App,
) {
    let open = workspace
        .read(cx)
        .items_of_type::<BlockTable>(cx)
        .find(|tab| tab.read(cx).title == title);
    let tab = if let Some(tab) = open {
        workspace.update(cx, |workspace, cx| {
            workspace.activate_item(&tab, true, true, window, cx);
        });
        tab
    } else {
        let tooltip = SharedString::from(format!("{title}, shown from a terminal"));
        let source = SharedString::from(format!("show {title}"));
        add_table_tab(workspace, view, title, tooltip, source, window, cx).0
    };
    tab.update(cx, |tab, cx| tab.set_table(table, cx));
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
    tooltip: SharedString,
    table: Entity<TableView>,
    /// What the header says once the table is read (#929).
    facts: Option<TableFacts>,
    /// Where the table came from: the block's command, or `show <file>`.
    source: SharedString,
    _table_changed: Subscription,
}

/// A table's kind and its row count, for its tab's header (#929, D1).
#[derive(Clone, Copy)]
struct TableFacts {
    kind: TableKind,
    rows: usize,
}

impl BlockTable {
    /// Shows `table` and keeps its kind and row count for the header.
    fn set_table(&mut self, table: Table, cx: &mut Context<Self>) {
        self.facts = Some(TableFacts {
            kind: table.kind,
            rows: table.rows.len(),
        });
        self.table.update(cx, |table_view, cx| {
            table_view.set_contents(content_of(table), cx);
        });
        cx.notify();
    }
}

/// A table kind's short word, as its chip shows it.
const fn kind_word(kind: TableKind) -> &'static str {
    match kind {
        TableKind::Delimited(',') => "csv",
        TableKind::Delimited('\t') => "tsv",
        TableKind::Delimited(_) => "table",
        TableKind::JsonArray => "json",
        TableKind::JsonLines => "jsonl",
        TableKind::Markdown => "md",
    }
}

impl Focusable for BlockTable {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.table.read(cx).focus_handle(cx)
    }
}

impl EventEmitter<ItemEvent> for BlockTable {}

impl Render for BlockTable {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let colors = cx.theme().colors();
        // The kind, the name and where the rows came from over the table (#929).
        let header = h_flex()
            .flex_none()
            .w_full()
            .gap_2()
            .px_3()
            .py_1p5()
            .border_b_1()
            .border_color(colors.border_variant)
            .children(self.facts.map(|facts| {
                div()
                    .px_1p5()
                    .rounded_sm()
                    .bg(colors.element_background)
                    .child(
                        Label::new(kind_word(facts.kind))
                            .size(LabelSize::XSmall)
                            .buffer_font(cx),
                    )
            }))
            .child(Label::new(self.title.clone()).size(LabelSize::Small))
            .child(
                Label::new(match self.facts {
                    Some(facts) => format!("{} rows \u{b7} from {}", facts.rows, self.source),
                    None => format!("from {}", self.source),
                })
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
            );
        v_flex()
            .size_full()
            .child(header)
            .child(div().flex_1().min_h_0().child(self.table.clone()))
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
        Some(self.tooltip.clone())
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}
