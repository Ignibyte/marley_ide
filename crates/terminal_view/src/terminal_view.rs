mod persistence;
pub mod terminal_element;
pub mod terminal_panel;
mod terminal_path_like_target;
pub mod terminal_scrollbar;

use editor::{
    Editor, EditorSettings, actions::SelectAll, blink_manager::BlinkManager,
    ui_scrollbar_settings_from_raw,
};
use gpui::{
    Action, AnyElement, App, ClipboardEntry, DismissEvent, Entity, EventEmitter, ExternalPaths,
    FocusHandle, Focusable, Font, KeyContext, KeyDownEvent, Keystroke, MouseButton, MouseDownEvent,
    Pixels, Point as GpuiPoint, Render, ScrollWheelEvent, Styled, Subscription, Task, TaskExt,
    WeakEntity, actions, anchored, deferred, div,
};
use menu;
use persistence::TerminalDb;
use project::{Project, ProjectEntryId, search::SearchQuery};
use schemars::JsonSchema;
use serde::Deserialize;
use settings::{
    SeedQuerySetting, Settings, SettingsStore, TerminalBell, TerminalBlink, WorkingDirectory,
};
use std::{
    any::Any,
    cmp,
    ops::Range as StdRange,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::Duration,
};
use task::TaskId;
use terminal::{
    Clear, Copy, Event, HoveredWord, MaybeNavigationTarget, Modes, Paste, PasteText, Point, Range,
    ScrollLineDown, ScrollLineUp, ScrollPageDown, ScrollPageUp, ScrollToBottom, ScrollToTop,
    Search, ShowCharacterPalette, TaskState, TaskStatus, Terminal, TerminalBounds, ToggleViMode,
    terminal_settings::{CursorShape, TerminalSettings},
};
use terminal_element::TerminalElement;
use terminal_panel::TerminalPanel;
use terminal_path_like_target::{hover_path_like_target, open_path_like_target};
use terminal_scrollbar::TerminalScrollHandle;
use ui::{
    ContextMenu, Divider, ScrollAxes, Scrollbars, Tooltip, WithScrollbar,
    prelude::*,
    scrollbars::{self, ScrollbarVisibility},
};
use util::ResultExt;
use workspace::{
    CloseActiveItem, DraggedSelection, DraggedTab, NewCenterTerminal, NewTerminal, Pane,
    ToolbarItemLocation, Workspace, WorkspaceId, delete_unloaded_items,
    item::{
        HighlightedText, Item, ItemEvent, SerializableItem, TabContentParams, TabTooltipContent,
    },
    register_serializable_item,
    searchable::{
        Direction, SearchEvent, SearchOptions, SearchToken, SearchableItem, SearchableItemHandle,
    },
};
use zed_actions::{agent::AddSelectionToThread, assistant::InlineAssist};

struct ImeState {
    marked_text: String,
}

fn viewport_line_for_point(point: Point, display_offset: usize) -> Option<usize> {
    let display_offset = i32::try_from(display_offset).unwrap_or(i32::MAX);
    let line = point.line.saturating_add(display_offset);
    if line < 0 {
        None
    } else {
        usize::try_from(line).ok()
    }
}

const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

/// Event to transmit the scroll from the element to the view
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollTerminal(pub i32);

/// Sends the specified text directly to the terminal.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Action)]
#[action(namespace = terminal)]
pub struct SendText(String);

/// Sends a keystroke sequence to the terminal.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Action)]
#[action(namespace = terminal)]
pub struct SendKeystroke(String);

actions!(
    terminal,
    [
        /// Reruns the last executed task in the terminal.
        RerunTask,
    ]
);

/// Renames the terminal tab.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Action)]
#[action(namespace = terminal)]
pub struct RenameTerminal;

pub fn init(cx: &mut App) {
    terminal_panel::init(cx);

    register_serializable_item::<TerminalView>(cx);

    cx.observe_new(|workspace: &mut Workspace, _window, _cx| {
        workspace.register_action(TerminalView::deploy);
    })
    .detach();
}

pub struct BlockProperties {
    pub height: u8,
    pub render: Box<dyn Send + Fn(&mut BlockContext) -> AnyElement>,
}

pub struct BlockContext<'a, 'b> {
    pub window: &'a mut Window,
    pub context: &'b mut App,
    pub dimensions: TerminalBounds,
}

// Marley: what a footer below a terminal is drawn from (#477).
pub struct MarleyFooterContext<'a> {
    pub view: WeakEntity<TerminalView>,
    pub terminal: &'a Entity<Terminal>,
    pub project: &'a WeakEntity<Project>,
    pub workspace: &'a WeakEntity<Workspace>,
    pub focus_handle: &'a FocusHandle,
}

// Marley: draws an element below each terminal, or none; Marley's workbench sets it, so the
// footer's code stays in Marley's crates (#477).
#[derive(Clone)]
pub struct MarleyTerminalFooter(
    pub Arc<dyn Fn(&MarleyFooterContext, &mut Window, &mut App) -> Option<AnyElement>>,
);

impl gpui::Global for MarleyTerminalFooter {}

// Marley: where a URL clicked in a terminal opens; Marley's workbench sets it, and `true` means it
// took the URL (#503).
#[derive(Clone)]
pub struct MarleyTerminalUrl(
    pub Arc<dyn Fn(&MarleyFooterContext, &str, &mut Window, &mut App) -> bool>,
);

impl gpui::Global for MarleyTerminalUrl {}

// Marley: the entries a menu on a clicked link starts with; Marley's workbench sets it (#579).
#[derive(Clone)]
pub struct MarleyTerminalLinkMenu(
    pub  Arc<
        dyn Fn(
            &MarleyFooterContext,
            &terminal::MarleyLink,
            ContextMenu,
            &mut Window,
            &mut App,
        ) -> ContextMenu,
    >,
);

impl gpui::Global for MarleyTerminalLinkMenu {}

// Marley: the block each terminal has selected, by the terminal's entity id, with the count of
// inputs its blocks had noted then, so any input since ends the selection. Marley's workbench
// writes it; the view's key context and the element's outline read it (#554).
#[derive(Default)]
pub struct MarleyBlockSelection(pub collections::HashMap<gpui::EntityId, (usize, u64)>);

impl gpui::Global for MarleyBlockSelection {}

impl MarleyBlockSelection {
    /// The block `terminal` has selected, while no input has reached it since.
    pub fn selected(terminal: &Entity<Terminal>, cx: &App) -> Option<usize> {
        let (index, inputs) = *cx.try_global::<Self>()?.0.get(&terminal.entity_id())?;
        (terminal.read(cx).marley_anchored().inputs() == inputs).then_some(index)
    }
}

// Marley: the Block section a right-click on a block adds after Zed's items, given the block's
// index; Marley's workbench sets it (#554).
#[derive(Clone)]
pub struct MarleyTerminalBlockMenu(
    pub Arc<dyn Fn(&MarleyFooterContext, usize, ContextMenu, &mut Window, &mut App) -> ContextMenu>,
);

impl gpui::Global for MarleyTerminalBlockMenu {}

// Marley: an element on a block's first row before its pill, such as Ask the agent under a failed
// command, or none; Marley's workbench sets it (#555).
#[derive(Clone)]
pub struct MarleyBlockChip(
    pub Arc<dyn Fn(&Entity<TerminalView>, &Entity<Terminal>, usize, &App) -> Option<AnyElement>>,
);

impl gpui::Global for MarleyBlockChip {}

// Marley: the header pinned over the top row of a view scrolled back into a block whose first row
// is above it, given the block's index, or none; Marley's workbench sets it (#529).
#[derive(Clone)]
pub struct MarleyStickyHeader(
    pub Arc<dyn Fn(&Entity<TerminalView>, &Entity<Terminal>, usize, &App) -> Option<AnyElement>>,
);

impl gpui::Global for MarleyStickyHeader {}

// Marley: the header drawn over a block's prompt rows in place of the shell's prompt, given the
// block's index, the height of a row and how many rows it covers (#630), or none, which leaves the
// rows as the shell drew them; Marley's workbench sets it (#628).
#[derive(Clone)]
pub struct MarleyBlockHeader(
    pub  Arc<
        dyn Fn(
            &Entity<TerminalView>,
            &Entity<Terminal>,
            usize,
            Pixels,
            usize,
            &App,
        ) -> Option<AnyElement>,
    >,
);

impl gpui::Global for MarleyBlockHeader {}

// Marley: how far blocks stand apart and whether a header over a one-row prompt takes a second
// row, from `marley.block_density`; Marley's workbench sets it, and without it every row is drawn
// where the grid puts it (#631).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MarleyBlockSpacing {
    /// The space between blocks, in rows.
    pub gap_rows: f32,
    /// Whether a header over a one-row prompt takes a second row.
    pub tall_headers: bool,
}

impl gpui::Global for MarleyBlockSpacing {}

// Marley: an element drawn over the terminal's grid, such as a block's filter, or none; Marley's
// workbench sets it (#528).
#[derive(Clone)]
pub struct MarleyTerminalOverlay(
    pub Arc<dyn Fn(&MarleyFooterContext, &mut Window, &mut App) -> Option<AnyElement>>,
);

impl gpui::Global for MarleyTerminalOverlay {}

// Marley: what a block's Filter button does, given the view and the block's index; while it is
// set the element shows the button (#528).
#[derive(Clone)]
pub struct MarleyBlockFilter(pub Arc<dyn Fn(&Entity<TerminalView>, usize, &mut Window, &mut App)>);

impl gpui::Global for MarleyBlockFilter {}

// Marley: the extra buttons a block's hover actions start with, such as Save as Workflow; Marley's
// workbench sets it (#558).
#[derive(Clone)]
pub struct MarleyBlockExtras(
    pub Arc<dyn Fn(&Entity<TerminalView>, &Entity<Terminal>, usize, &App) -> Vec<AnyElement>>,
);

impl gpui::Global for MarleyBlockExtras {}

// Marley: each terminal's bookmarked blocks and the block its search is held to, by the
// terminal's entity id. Marley's workbench writes it; the element ticks the bookmarks and
// outlines the scoped block, `find_matches` keeps the scoped block's matches, and a closed search
// bar ends the scope (#559).
#[derive(Default)]
pub struct MarleyBlockMarks {
    pub bookmarks: collections::HashMap<gpui::EntityId, collections::BTreeSet<usize>>,
    pub search_scopes: collections::HashMap<gpui::EntityId, usize>,
}

impl gpui::Global for MarleyBlockMarks {}

impl MarleyBlockMarks {
    /// The blocks `terminal` has bookmarked, oldest first.
    pub fn bookmarked(terminal: &Entity<Terminal>, cx: &App) -> Vec<usize> {
        cx.try_global::<Self>()
            .and_then(|marks| marks.bookmarks.get(&terminal.entity_id()))
            .map(|marked| marked.iter().copied().collect())
            .unwrap_or_default()
    }

    /// The block `terminal`'s search is held to.
    pub fn search_scope(terminal: &Entity<Terminal>, cx: &App) -> Option<usize> {
        cx.try_global::<Self>()?
            .search_scopes
            .get(&terminal.entity_id())
            .copied()
    }
}

// Marley: the autosuggestion a terminal shows after its cursor, or none; Marley's workbench
// sets it (#484).
#[derive(Clone)]
pub struct MarleyTerminalSuggestion(
    pub Arc<dyn Fn(&Entity<Terminal>, &mut App) -> Option<SharedString>>,
);

impl gpui::Global for MarleyTerminalSuggestion {}

// Marley: the command typed at the shell's prompt, in the theme's syntax colours (#626).
/// The colours of the cells on one line of the grid: each run of columns and its colour.
#[derive(Clone, Debug, Default)]
pub struct MarleyPromptColors {
    /// The grid line, as the content's cells number it.
    pub line: i32,
    /// Each run of columns and its colour.
    pub runs: Vec<(std::ops::Range<usize>, gpui::Hsla)>,
}

impl MarleyPromptColors {
    /// The colour of the cell at `line` and `column`, when a run holds it.
    pub fn color_at(&self, line: i32, column: usize) -> Option<gpui::Hsla> {
        (line == self.line)
            .then(|| self.runs.iter().find(|(run, _)| run.contains(&column)))
            .flatten()
            .map(|(_, color)| *color)
    }
}

// Marley: the colours of what was typed at a terminal's prompt; Marley's workbench sets it (#626).
#[derive(Clone)]
pub struct MarleyPromptColoring(
    pub Arc<dyn Fn(&Entity<Terminal>, &App) -> Option<MarleyPromptColors>>,
);

impl gpui::Global for MarleyPromptColoring {}

// Marley: keeps each terminal's MARLEY_TERMINAL_ID across a restore, in a table of Marley's own
// whose rows follow the `terminals` rows; Marley's workbench sets it (#575).
#[derive(Clone)]
pub struct MarleyTerminalIdentity {
    /// The id saved for an item, if any.
    pub saved: Arc<dyn Fn(WorkspaceId, workspace::ItemId, &App) -> Option<String>>,
    /// Saves an item's id.
    pub save:
        Arc<dyn Fn(WorkspaceId, workspace::ItemId, String, &mut App) -> Task<anyhow::Result<()>>>,
    /// Moves an item's row from the old workspace id to the new one.
    pub moved: Arc<
        dyn Fn(WorkspaceId, WorkspaceId, workspace::ItemId, &mut App) -> Task<anyhow::Result<()>>,
    >,
    /// Deletes the rows of the workspace's items that were not loaded.
    pub cleanup:
        Arc<dyn Fn(WorkspaceId, Vec<workspace::ItemId>, &mut App) -> Task<anyhow::Result<()>>>,
}

impl gpui::Global for MarleyTerminalIdentity {}

///A terminal view, maintains the PTY's file handles and communicates with the terminal
pub struct TerminalView {
    terminal: Entity<Terminal>,
    workspace: WeakEntity<Workspace>,
    project: WeakEntity<Project>,
    focus_handle: FocusHandle,
    //Currently using iTerm bell, show bell emoji in tab until input is received
    has_bell: bool,
    context_menu: Option<(Entity<ContextMenu>, GpuiPoint<Pixels>, Subscription)>,
    cursor_shape: CursorShape,
    blink_manager: Entity<BlinkManager>,
    mode: TerminalMode,
    // Explicit override for whether workspace-specific context menu actions are shown.
    // When `None`, visibility is derived from `mode` (hidden for embedded terminals).
    show_workspace_actions: Option<bool>,
    blinking_terminal_enabled: bool,
    needs_serialize: bool,
    custom_title: Option<String>,
    hover: Option<HoverTarget>,
    hover_tooltip_update: Task<()>,
    workspace_id: Option<WorkspaceId>,
    show_breadcrumbs: bool,
    block_below_cursor: Option<Rc<BlockProperties>>,
    scroll_top: Pixels,
    scroll_handle: TerminalScrollHandle,
    ime_state: Option<ImeState>,
    self_handle: WeakEntity<Self>,
    rename_editor: Option<Entity<Editor>>,
    rename_editor_subscription: Option<Subscription>,
    _subscriptions: Vec<Subscription>,
    _terminal_subscriptions: Vec<Subscription>,
}

#[derive(Default, Clone)]
pub enum TerminalMode {
    #[default]
    Standalone,
    Embedded {
        max_lines_when_unfocused: Option<usize>,
    },
}

#[derive(Clone)]
pub enum ContentMode {
    Scrollable,
    Inline {
        displayed_lines: usize,
        total_lines: usize,
    },
}

impl ContentMode {
    pub fn is_limited(&self) -> bool {
        match self {
            ContentMode::Scrollable => false,
            ContentMode::Inline {
                displayed_lines,
                total_lines,
            } => displayed_lines < total_lines,
        }
    }

    pub fn is_scrollable(&self) -> bool {
        matches!(self, ContentMode::Scrollable)
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(Clone, Eq, PartialEq))]
struct HoverTarget {
    tooltip: String,
    hovered_word: HoveredWord,
}

impl EventEmitter<Event> for TerminalView {}
impl EventEmitter<ItemEvent> for TerminalView {}
impl EventEmitter<SearchEvent> for TerminalView {}

impl Focusable for TerminalView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl TerminalView {
    ///Create a new Terminal in the current working directory or the user's home directory
    pub fn deploy(
        workspace: &mut Workspace,
        action: &NewCenterTerminal,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        let local = action.local;
        let working_directory = default_working_directory(workspace, cx);
        TerminalPanel::add_center_terminal(workspace, window, cx, move |project, cx| {
            if local {
                project.create_local_terminal(cx)
            } else {
                project.create_terminal_shell(working_directory, cx)
            }
        })
        .detach_and_log_err(cx);
    }

    pub fn new(
        terminal: Entity<Terminal>,
        workspace: WeakEntity<Workspace>,
        workspace_id: Option<WorkspaceId>,
        project: WeakEntity<Project>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let workspace_handle = workspace.clone();
        let terminal_subscriptions =
            subscribe_for_terminal_events(&terminal, workspace, window, cx);

        let focus_handle = cx.focus_handle();
        let focus_in = cx.on_focus_in(&focus_handle, window, |terminal_view, window, cx| {
            terminal_view.focus_in(window, cx);
        });
        let focus_out = cx.on_focus_out(
            &focus_handle,
            window,
            |terminal_view, _event, window, cx| {
                terminal_view.focus_out(window, cx);
            },
        );
        let cursor_shape = TerminalSettings::get_global(cx).cursor_shape;

        let scroll_handle = TerminalScrollHandle::new(terminal.read(cx));

        let blink_manager = cx.new(|cx| {
            BlinkManager::new(
                CURSOR_BLINK_INTERVAL,
                |cx| {
                    !matches!(
                        TerminalSettings::get_global(cx).blinking,
                        TerminalBlink::Off
                    )
                },
                cx,
            )
        });

        let subscriptions = vec![
            focus_in,
            focus_out,
            cx.observe(&blink_manager, |_, _, cx| cx.notify()),
            cx.observe_global::<SettingsStore>(Self::settings_changed),
        ];

        Self {
            terminal,
            workspace: workspace_handle,
            project,
            has_bell: false,
            focus_handle,
            context_menu: None,
            cursor_shape,
            blink_manager,
            blinking_terminal_enabled: false,
            hover: None,
            hover_tooltip_update: Task::ready(()),
            mode: TerminalMode::Standalone,
            show_workspace_actions: None,
            workspace_id,
            show_breadcrumbs: TerminalSettings::get_global(cx).toolbar.breadcrumbs,
            block_below_cursor: None,
            scroll_top: Pixels::ZERO,
            scroll_handle,
            needs_serialize: false,
            custom_title: None,
            ime_state: None,
            self_handle: cx.entity().downgrade(),
            rename_editor: None,
            rename_editor_subscription: None,
            _subscriptions: subscriptions,
            _terminal_subscriptions: terminal_subscriptions,
        }
    }

    /// Enable 'embedded' mode where the terminal displays the full content with an optional limit of lines.
    pub fn set_embedded_mode(
        &mut self,
        max_lines_when_unfocused: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        self.mode = TerminalMode::Embedded {
            max_lines_when_unfocused,
        };
        cx.notify();
    }

    /// Explicitly override whether workspace-specific context menu actions (e.g. creating or
    /// closing terminal tabs, inline assist) are shown.
    ///
    /// This lets hosts that aren't workspace panes (such as the agent panel) hide these
    /// actions without `terminal_view` needing to know about those hosts. When never called,
    /// visibility is derived from the terminal's `mode`.
    pub fn set_show_workspace_actions(&mut self, show: bool, cx: &mut Context<Self>) {
        self.show_workspace_actions = Some(show);
        cx.notify();
    }

    fn shows_workspace_actions(&self) -> bool {
        self.show_workspace_actions
            .unwrap_or_else(|| !matches!(self.mode, TerminalMode::Embedded { .. }))
    }

    const MAX_EMBEDDED_LINES: usize = 1_000;

    /// Returns the current `ContentMode` depending on the set `TerminalMode` and the current number of lines
    ///
    /// Note: Even in embedded mode, the terminal will fallback to scrollable when its content exceeds `MAX_EMBEDDED_LINES`
    pub fn content_mode(&self, window: &Window, cx: &App) -> ContentMode {
        match &self.mode {
            TerminalMode::Standalone => ContentMode::Scrollable,
            TerminalMode::Embedded {
                max_lines_when_unfocused,
            } => {
                let terminal = self.terminal.read(cx);
                let total_lines = terminal.total_lines();

                if total_lines > Self::MAX_EMBEDDED_LINES {
                    ContentMode::Scrollable
                } else {
                    let mut displayed_lines = terminal.used_lines().min(total_lines);

                    if !self.focus_handle.is_focused(window)
                        && let Some(max_lines) = max_lines_when_unfocused
                    {
                        displayed_lines = displayed_lines.min(*max_lines)
                    }

                    ContentMode::Inline {
                        displayed_lines,
                        total_lines,
                    }
                }
            }
        }
    }

    /// Sets the marked (pre-edit) text from the IME.
    pub(crate) fn set_marked_text(&mut self, text: String, cx: &mut Context<Self>) {
        if text.is_empty() {
            return self.clear_marked_text(cx);
        }
        self.ime_state = Some(ImeState { marked_text: text });
        cx.notify();
    }

    /// Gets the current marked range (UTF-16).
    pub(crate) fn marked_text_range(&self) -> Option<StdRange<usize>> {
        self.ime_state
            .as_ref()
            .map(|state| 0..state.marked_text.encode_utf16().count())
    }

    /// Clears the marked (pre-edit) text state.
    pub(crate) fn clear_marked_text(&mut self, cx: &mut Context<Self>) {
        if self.ime_state.is_some() {
            self.ime_state = None;
            cx.notify();
        }
    }

    /// Commits (sends) the given text to the PTY. Called by InputHandler::replace_text_in_range.
    pub(crate) fn commit_text(&mut self, text: &str, cx: &mut Context<Self>) {
        if !text.is_empty() {
            self.terminal.update(cx, |term, _| {
                term.input(text.to_string().into_bytes());
            });
        }
    }

    pub(crate) fn terminal_bounds(&self, cx: &App) -> TerminalBounds {
        self.terminal.read(cx).last_content().terminal_bounds
    }

    pub fn entity(&self) -> &Entity<Terminal> {
        &self.terminal
    }

    pub fn has_bell(&self) -> bool {
        self.has_bell
    }

    pub fn custom_title(&self) -> Option<&str> {
        self.custom_title.as_deref()
    }

    pub fn set_custom_title(&mut self, label: Option<String>, cx: &mut Context<Self>) {
        let label = label.filter(|l| !l.trim().is_empty());
        if self.custom_title != label {
            self.custom_title = label;
            self.needs_serialize = true;
            cx.emit(ItemEvent::UpdateTab);
            cx.notify();
        }
    }

    pub(crate) fn mark_needs_serialize(&mut self, cx: &mut Context<Self>) {
        self.needs_serialize = true;
        cx.emit(ItemEvent::UpdateTab);
    }

    pub fn is_renaming(&self) -> bool {
        self.rename_editor.is_some()
    }

    pub fn rename_editor_is_focused(&self, window: &Window, cx: &App) -> bool {
        self.rename_editor
            .as_ref()
            .is_some_and(|editor| editor.focus_handle(cx).is_focused(window))
    }

    fn finish_renaming(&mut self, save: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(editor) = self.rename_editor.take() else {
            return;
        };
        self.rename_editor_subscription = None;
        if save {
            let new_label = editor.read(cx).text(cx).trim().to_string();
            let label = if new_label.is_empty() {
                None
            } else {
                // Only set custom_title if the text differs from the terminal's dynamic title.
                // This prevents subtle layout changes when clicking away without making changes.
                let terminal_title = self.terminal.read(cx).title(true);
                if new_label == terminal_title {
                    None
                } else {
                    Some(new_label)
                }
            };
            self.set_custom_title(label, cx);
        }
        cx.notify();
        self.focus_handle.focus(window, cx);
    }

    pub fn rename_terminal(
        &mut self,
        _: &RenameTerminal,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.terminal.read(cx).task().is_some() {
            return;
        }

        let current_label = self
            .custom_title
            .clone()
            .unwrap_or_else(|| self.terminal.read(cx).title(true));

        let rename_editor = cx.new(|cx| Editor::single_line(window, cx));
        let rename_editor_subscription = cx.subscribe_in(&rename_editor, window, {
            let rename_editor = rename_editor.clone();
            move |_this, _, event, window, cx| {
                if let editor::EditorEvent::Blurred = event {
                    // Defer to let focus settle (avoids canceling during double-click).
                    let rename_editor = rename_editor.clone();
                    cx.defer_in(window, move |this, window, cx| {
                        let still_current = this
                            .rename_editor
                            .as_ref()
                            .is_some_and(|current| current == &rename_editor);
                        if still_current && !rename_editor.focus_handle(cx).is_focused(window) {
                            this.finish_renaming(false, window, cx);
                        }
                    });
                }
            }
        });

        self.rename_editor = Some(rename_editor.clone());
        self.rename_editor_subscription = Some(rename_editor_subscription);

        rename_editor.update(cx, |editor, cx| {
            editor.set_text(current_label, window, cx);
            editor.select_all(&SelectAll, window, cx);
            editor.focus_handle(cx).focus(window, cx);
        });
        cx.notify();
    }

    pub fn clear_bell(&mut self, cx: &mut Context<TerminalView>) {
        self.has_bell = false;
        cx.emit(Event::Wakeup);
    }

    pub fn deploy_context_menu(
        &mut self,
        position: GpuiPoint<Pixels>,
        has_selection: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let assistant_enabled = self
            .workspace
            .upgrade()
            .and_then(|workspace| workspace.read(cx).panel::<TerminalPanel>(cx))
            .is_some_and(|terminal_panel| terminal_panel.read(cx).assistant_enabled());
        // Marley: a click on a link starts the menu with the link's entries (#579).
        let marley_link = cx
            .try_global::<MarleyTerminalLinkMenu>()
            .cloned()
            .and_then(|hook| {
                let link = self
                    .terminal
                    .update(cx, |terminal, _| terminal.marley_link_at(position))?;
                Some((hook, link))
            });
        let marley_view = cx.entity().downgrade();
        // Marley: the block under the click, for the menu's Block section (#554).
        let marley_block = cx
            .try_global::<MarleyTerminalBlockMenu>()
            .cloned()
            .and_then(|hook| {
                let block = terminal_element::marley_block_at(self.terminal.read(cx), position)?;
                Some((hook, block))
            });
        let context_menu = ContextMenu::build(window, cx, |menu, window, cx| {
            let menu = match marley_link {
                Some((hook, link)) => {
                    let context = MarleyFooterContext {
                        view: marley_view.clone(),
                        terminal: &self.terminal,
                        project: &self.project,
                        workspace: &self.workspace,
                        focus_handle: &self.focus_handle,
                    };
                    (hook.0)(&context, &link, menu, window, cx).separator()
                }
                None => menu,
            };
            menu.context(self.focus_handle.clone())
                .when(self.shows_workspace_actions(), |menu| {
                    menu.action("New Terminal", Box::new(NewTerminal::default()))
                        .action(
                            "New Center Terminal",
                            Box::new(NewCenterTerminal::default()),
                        )
                        .separator()
                })
                .action("Copy", Box::new(Copy))
                .when(
                    !matches!(self.mode, TerminalMode::Embedded { .. }),
                    |menu| {
                        menu.action("Paste", Box::new(Paste))
                            .action("Paste Text", Box::new(PasteText))
                    },
                )
                .action("Select All", Box::new(SelectAll))
                .when(
                    !matches!(self.mode, TerminalMode::Embedded { .. }),
                    |menu| menu.action("Clear", Box::new(Clear)),
                )
                .when(
                    assistant_enabled && !matches!(self.mode, TerminalMode::Embedded { .. }),
                    |menu| {
                        menu.separator()
                            .action("Inline Assist", Box::new(InlineAssist::default()))
                            .when(has_selection && self.shows_workspace_actions(), |menu| {
                                menu.action("Add to Agent Thread", Box::new(AddSelectionToThread))
                            })
                    },
                )
                .when(self.shows_workspace_actions(), |menu| {
                    menu.separator().action(
                        "Close Terminal Tab",
                        Box::new(CloseActiveItem {
                            save_intent: None,
                            close_pinned: true,
                        }),
                    )
                })
                .map(|menu| match marley_block {
                    Some((hook, block)) => {
                        let context = MarleyFooterContext {
                            view: marley_view,
                            terminal: &self.terminal,
                            project: &self.project,
                            workspace: &self.workspace,
                            focus_handle: &self.focus_handle,
                        };
                        (hook.0)(&context, block, menu, window, cx)
                    }
                    None => menu,
                })
        });

        window.focus(&context_menu.focus_handle(cx), cx);
        let subscription = cx.subscribe_in(
            &context_menu,
            window,
            |this, _, _: &DismissEvent, window, cx| {
                if this.context_menu.as_ref().is_some_and(|context_menu| {
                    context_menu.0.focus_handle(cx).contains_focused(window, cx)
                }) {
                    cx.focus_self(window);
                }
                this.context_menu.take();
                cx.notify();
            },
        );

        self.context_menu = Some((context_menu, position, subscription));
    }

    // Marley: a menu of a clicked link's entries alone, for a plain click on it; an OSC 8 link's
    // plain click opens it instead (#579).
    /// Shows the link menu at `position` when a plain click there lands on a link in the text.
    pub fn marley_deploy_link_menu(
        &mut self,
        position: GpuiPoint<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(hook) = cx.try_global::<MarleyTerminalLinkMenu>().cloned() else {
            return;
        };
        let Some(link) = self
            .terminal
            .update(cx, |terminal, _| terminal.marley_link_at(position))
            .filter(|link| !link.osc8)
        else {
            return;
        };
        let view = cx.entity().downgrade();
        let context_menu = ContextMenu::build(window, cx, |menu, window, cx| {
            let context = MarleyFooterContext {
                view,
                terminal: &self.terminal,
                project: &self.project,
                workspace: &self.workspace,
                focus_handle: &self.focus_handle,
            };
            (hook.0)(
                &context,
                &link,
                menu.context(self.focus_handle.clone()),
                window,
                cx,
            )
        });
        window.focus(&context_menu.focus_handle(cx), cx);
        let subscription = cx.subscribe_in(
            &context_menu,
            window,
            |this, _, _: &DismissEvent, window, cx| {
                if this.context_menu.as_ref().is_some_and(|context_menu| {
                    context_menu.0.focus_handle(cx).contains_focused(window, cx)
                }) {
                    cx.focus_self(window);
                }
                this.context_menu.take();
                cx.notify();
            },
        );
        self.context_menu = Some((context_menu, position, subscription));
        cx.notify();
    }

    fn settings_changed(&mut self, cx: &mut Context<Self>) {
        let settings = TerminalSettings::get_global(cx);
        let breadcrumb_visibility_changed = self.show_breadcrumbs != settings.toolbar.breadcrumbs;
        self.show_breadcrumbs = settings.toolbar.breadcrumbs;

        let should_blink = match settings.blinking {
            TerminalBlink::Off => false,
            TerminalBlink::On => true,
            TerminalBlink::TerminalControlled => self.blinking_terminal_enabled,
        };
        let new_cursor_shape = settings.cursor_shape;
        let old_cursor_shape = self.cursor_shape;
        if old_cursor_shape != new_cursor_shape {
            self.cursor_shape = new_cursor_shape;
            self.terminal.update(cx, |term, _| {
                term.set_cursor_shape(self.cursor_shape);
            });
        }

        self.blink_manager.update(
            cx,
            if should_blink {
                BlinkManager::enable
            } else {
                BlinkManager::disable
            },
        );

        if breadcrumb_visibility_changed {
            cx.emit(ItemEvent::UpdateBreadcrumbs);
        }
        cx.notify();
    }

    fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self
            .terminal
            .read(cx)
            .last_content
            .mode
            .contains(Modes::ALT_SCREEN)
        {
            self.terminal.update(cx, |term, cx| {
                term.try_keystroke(
                    &Keystroke::parse("ctrl-cmd-space").unwrap(),
                    TerminalSettings::get_global(cx).option_as_meta,
                )
            });
        } else {
            window.show_character_palette();
        }
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.terminal.update(cx, |term, _| term.select_all());
        cx.notify();
    }

    fn rerun_task(&mut self, _: &RerunTask, window: &mut Window, cx: &mut Context<Self>) {
        let task = self
            .terminal
            .read(cx)
            .task()
            .map(|task| terminal_rerun_override(&task.spawned_task.id))
            .unwrap_or_default();
        window.dispatch_action(Box::new(task), cx);
    }

    fn clear(&mut self, _: &Clear, _: &mut Window, cx: &mut Context<Self>) {
        self.scroll_top = px(0.);
        self.terminal.update(cx, |term, _| term.clear());
        cx.notify();
    }

    fn max_scroll_top(&self, cx: &App) -> Pixels {
        let terminal = self.terminal.read(cx);

        let Some(block) = self.block_below_cursor.as_ref() else {
            return Pixels::ZERO;
        };

        let line_height = terminal.last_content().terminal_bounds.line_height;
        let viewport_lines = terminal.viewport_lines();
        let cursor_line = viewport_line_for_point(
            terminal.last_content.cursor.point,
            terminal.last_content.display_offset,
        )
        .unwrap_or_default();
        let max_scroll_top_in_lines =
            (block.height as usize).saturating_sub(viewport_lines.saturating_sub(cursor_line + 1));

        max_scroll_top_in_lines as f32 * line_height
    }

    fn scroll_wheel(&mut self, event: &ScrollWheelEvent, cx: &mut Context<Self>) {
        let terminal_content = self.terminal.read(cx).last_content();

        if self.block_below_cursor.is_some() && terminal_content.display_offset == 0 {
            let line_height = terminal_content.terminal_bounds.line_height;
            let y_delta = event.delta.pixel_delta(line_height).y;
            if y_delta < Pixels::ZERO || self.scroll_top > Pixels::ZERO {
                self.scroll_top = cmp::max(
                    Pixels::ZERO,
                    cmp::min(self.scroll_top - y_delta, self.max_scroll_top(cx)),
                );
                cx.notify();
                return;
            }
        }
        self.terminal.update(cx, |term, cx| {
            term.scroll_wheel(
                event,
                TerminalSettings::get_global(cx).scroll_multiplier.max(0.01),
            )
        });
    }

    fn is_alt_screen(&self, cx: &App) -> bool {
        self.terminal
            .read(cx)
            .last_content
            .mode
            .contains(Modes::ALT_SCREEN)
    }

    fn scroll_line_up(&mut self, _: &ScrollLineUp, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_alt_screen(cx) {
            cx.propagate();
            return;
        }

        let terminal_content = self.terminal.read(cx).last_content();
        if self.block_below_cursor.is_some()
            && terminal_content.display_offset == 0
            && self.scroll_top > Pixels::ZERO
        {
            let line_height = terminal_content.terminal_bounds.line_height;
            self.scroll_top = cmp::max(self.scroll_top - line_height, Pixels::ZERO);
            return;
        }

        self.terminal.update(cx, |term, _| term.scroll_line_up());
        cx.notify();
    }

    fn scroll_line_down(&mut self, _: &ScrollLineDown, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_alt_screen(cx) {
            cx.propagate();
            return;
        }

        let terminal_content = self.terminal.read(cx).last_content();
        if self.block_below_cursor.is_some() && terminal_content.display_offset == 0 {
            let max_scroll_top = self.max_scroll_top(cx);
            if self.scroll_top < max_scroll_top {
                let line_height = terminal_content.terminal_bounds.line_height;
                self.scroll_top = cmp::min(self.scroll_top + line_height, max_scroll_top);
            }
            return;
        }

        self.terminal.update(cx, |term, _| term.scroll_line_down());
        cx.notify();
    }

    fn scroll_page_up(&mut self, _: &ScrollPageUp, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_alt_screen(cx) {
            cx.propagate();
            return;
        }

        if self.scroll_top == Pixels::ZERO {
            self.terminal.update(cx, |term, _| term.scroll_page_up());
        } else {
            let line_height = self
                .terminal
                .read(cx)
                .last_content
                .terminal_bounds
                .line_height();
            let visible_block_lines = (self.scroll_top / line_height) as usize;
            let viewport_lines = self.terminal.read(cx).viewport_lines();
            let visible_content_lines = viewport_lines - visible_block_lines;

            if visible_block_lines >= viewport_lines {
                self.scroll_top = ((visible_block_lines - viewport_lines) as f32) * line_height;
            } else {
                self.scroll_top = px(0.);
                self.terminal
                    .update(cx, |term, _| term.scroll_up_by(visible_content_lines));
            }
        }
        cx.notify();
    }

    fn scroll_page_down(&mut self, _: &ScrollPageDown, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_alt_screen(cx) {
            cx.propagate();
            return;
        }

        self.terminal.update(cx, |term, _| term.scroll_page_down());
        let terminal = self.terminal.read(cx);
        if terminal.last_content().display_offset < terminal.viewport_lines() {
            self.scroll_top = self.max_scroll_top(cx);
        }
        cx.notify();
    }

    fn scroll_to_top(&mut self, _: &ScrollToTop, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_alt_screen(cx) {
            cx.propagate();
            return;
        }

        self.terminal.update(cx, |term, _| term.scroll_to_top());
        cx.notify();
    }

    fn scroll_to_bottom(&mut self, _: &ScrollToBottom, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_alt_screen(cx) {
            cx.propagate();
            return;
        }

        self.terminal.update(cx, |term, _| term.scroll_to_bottom());
        if self.block_below_cursor.is_some() {
            self.scroll_top = self.max_scroll_top(cx);
        }
        cx.notify();
    }

    fn toggle_vi_mode(&mut self, _: &ToggleViMode, _: &mut Window, cx: &mut Context<Self>) {
        self.terminal.update(cx, |term, _| term.toggle_vi_mode());
        cx.notify();
    }

    pub fn should_show_cursor(&self, focused: bool, cx: &mut Context<Self>) -> bool {
        // Hide cursor when in embedded mode and not focused (read-only output like Agent panel)
        if let TerminalMode::Embedded { .. } = &self.mode {
            if !focused {
                return false;
            }
        }

        // For Standalone mode: always show cursor when not focused or in special modes
        if !focused
            || self
                .terminal
                .read(cx)
                .last_content
                .mode
                .contains(Modes::ALT_SCREEN)
        {
            return true;
        }

        // When focused, check blinking settings and blink manager state
        match TerminalSettings::get_global(cx).blinking {
            TerminalBlink::Off => true,
            TerminalBlink::TerminalControlled => {
                !self.blinking_terminal_enabled || self.blink_manager.read(cx).visible()
            }
            TerminalBlink::On => self.blink_manager.read(cx).visible(),
        }
    }

    pub fn pause_cursor_blinking(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.blink_manager.update(cx, BlinkManager::pause_blinking);
    }

    pub fn terminal(&self) -> &Entity<Terminal> {
        &self.terminal
    }

    // Marley: the workspace the view belongs to, which a notification's click shows it in
    // (#478).
    pub fn marley_workspace(&self) -> &WeakEntity<Workspace> {
        &self.workspace
    }

    pub fn set_block_below_cursor(
        &mut self,
        block: BlockProperties,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.block_below_cursor = Some(Rc::new(block));
        self.scroll_to_bottom(&ScrollToBottom, window, cx);
        cx.notify();
    }

    pub fn clear_block_below_cursor(&mut self, cx: &mut Context<Self>) {
        self.block_below_cursor = None;
        self.scroll_top = Pixels::ZERO;
        cx.notify();
    }

    ///Attempt to paste the clipboard into the terminal
    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        self.terminal.update(cx, |term, _| term.copy(None));
        cx.notify();
    }

    /// Specific handler for the [`editor::actions::Copy`] action in order for
    /// the `Edit > Copy` menu item to not be disabled, as the app expects a
    /// handler for this action in order to enable/disable the menu item.
    fn editor_copy(
        &mut self,
        _: &editor::actions::Copy,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.copy(&Copy, window, cx);
    }

    ///Attempt to paste the clipboard into the terminal
    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        let Some(clipboard) = cx.read_from_clipboard() else {
            return;
        };

        match clipboard.entries().first() {
            Some(ClipboardEntry::Image(image)) if !image.bytes.is_empty() => {
                self.forward_ctrl_v(cx);
            }
            Some(ClipboardEntry::ExternalPaths(paths)) => {
                self.add_paths_to_terminal(paths.paths(), window, cx);
            }
            _ => {
                if let Some(text) = clipboard.text() {
                    self.terminal
                        .update(cx, |terminal, _cx| terminal.paste(&text));
                }
            }
        }
    }

    /// Specific handler for the [`editor::actions::Paste`] action in order for
    /// the `Edit > Paste` menu item to not be disabled, as the app expects a
    /// handler for this action in order to enable/disable the menu item.
    fn editor_paste(
        &mut self,
        _: &editor::actions::Paste,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.paste(&Paste, window, cx);
    }

    ///Attempt to paste the clipboard text into the terminal
    fn paste_text(&mut self, _: &PasteText, _: &mut Window, cx: &mut Context<Self>) {
        let Some(clipboard) = cx.read_from_clipboard() else {
            return;
        };

        if let Some(text) = clipboard.text() {
            self.terminal
                .update(cx, |terminal, _cx| terminal.paste(&text));
        }
    }

    /// Emits a raw Ctrl+V so TUI agents can read the OS clipboard directly
    /// and attach images using their native workflows.
    fn forward_ctrl_v(&self, cx: &mut Context<Self>) {
        self.terminal.update(cx, |term, _| {
            term.input(vec![0x16]);
        });
    }

    pub fn add_paths_to_terminal(&self, paths: &[PathBuf], window: &mut Window, cx: &mut App) {
        // Marley: while an agent CLI runs, an image's path goes in raw inside a bracketed paste
        // of its own, the form in which Claude Code and Codex attach it; any other path is
        // quoted with a space after it, and a space parts an image from a quoted path (#536).
        if self.terminal.read(cx).marley_agent_in_foreground() {
            window.focus(&self.focus_handle(cx), cx);
            self.terminal.update(cx, |terminal, _| {
                let mut after_image = false;
                for path in paths {
                    let Some(text) = path.to_str() else {
                        continue;
                    };
                    if marley_terminal::paste::is_raw_image_path(path) {
                        terminal.marley_paste_bracketed(text);
                        after_image = true;
                    } else if let Ok(quoted) = shlex::try_quote(text) {
                        let separator = if after_image { " " } else { "" };
                        terminal.paste(&format!("{separator}{quoted} "));
                        after_image = false;
                    }
                }
            });
            return;
        }
        let mut text = paths
            .iter()
            .filter_map(|path| Some(format!(" {}", shlex::try_quote(path.to_str()?).ok()?)))
            .collect::<String>();
        text.push(' ');
        window.focus(&self.focus_handle(cx), cx);
        self.terminal.update(cx, |terminal, _| {
            terminal.paste(&text);
        });
    }

    fn send_text(&mut self, text: &SendText, _: &mut Window, cx: &mut Context<Self>) {
        self.clear_bell(cx);
        self.blink_manager.update(cx, BlinkManager::pause_blinking);
        self.terminal.update(cx, |term, _| {
            term.input(text.0.to_string().into_bytes());
        });
    }

    fn send_keystroke(&mut self, text: &SendKeystroke, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(keystroke) = Keystroke::parse(&text.0).log_err() {
            self.clear_bell(cx);
            self.blink_manager.update(cx, BlinkManager::pause_blinking);
            self.process_keystroke(&keystroke, cx);
        }
    }

    fn dispatch_context(&self, cx: &App) -> KeyContext {
        let mut dispatch_context = KeyContext::new_with_defaults();
        dispatch_context.add("Terminal");
        // Marley: the keys that move and act on a selected block apply while one is (#554).
        if MarleyBlockSelection::selected(&self.terminal, cx).is_some() {
            dispatch_context.add("MarleyBlockSelected");
        }

        if self.terminal.read(cx).vi_mode_enabled() {
            dispatch_context.add("vi_mode");
        }

        let mode = self.terminal.read(cx).last_content.mode;
        dispatch_context.set(
            "screen",
            if mode.contains(Modes::ALT_SCREEN) {
                "alt"
            } else {
                "normal"
            },
        );

        if mode.contains(Modes::APP_CURSOR) {
            dispatch_context.add("DECCKM");
        }
        if mode.contains(Modes::APP_KEYPAD) {
            dispatch_context.add("DECPAM");
        } else {
            dispatch_context.add("DECPNM");
        }
        if mode.contains(Modes::SHOW_CURSOR) {
            dispatch_context.add("DECTCEM");
        }
        if mode.contains(Modes::LINE_WRAP) {
            dispatch_context.add("DECAWM");
        }
        if mode.contains(Modes::ORIGIN) {
            dispatch_context.add("DECOM");
        }
        if mode.contains(Modes::INSERT) {
            dispatch_context.add("IRM");
        }
        //LNM is apparently the name for this. https://vt100.net/docs/vt510-rm/LNM.html
        if mode.contains(Modes::LINE_FEED_NEW_LINE) {
            dispatch_context.add("LNM");
        }
        if mode.contains(Modes::FOCUS_IN_OUT) {
            dispatch_context.add("report_focus");
        }
        if mode.contains(Modes::ALTERNATE_SCROLL) {
            dispatch_context.add("alternate_scroll");
        }
        if mode.contains(Modes::BRACKETED_PASTE) {
            dispatch_context.add("bracketed_paste");
        }
        if mode.intersects(Modes::MOUSE_MODE) {
            dispatch_context.add("any_mouse_reporting");
        }
        {
            let mouse_reporting = if mode.contains(Modes::MOUSE_REPORT_CLICK) {
                "click"
            } else if mode.contains(Modes::MOUSE_DRAG) {
                "drag"
            } else if mode.contains(Modes::MOUSE_MOTION) {
                "motion"
            } else {
                "off"
            };
            dispatch_context.set("mouse_reporting", mouse_reporting);
        }
        {
            let format = if mode.contains(Modes::SGR_MOUSE) {
                "sgr"
            } else if mode.contains(Modes::UTF8_MOUSE) {
                "utf8"
            } else {
                "normal"
            };
            dispatch_context.set("mouse_format", format);
        };

        if self.terminal.read(cx).last_content.selection.is_some() {
            dispatch_context.add("selection");
        }

        dispatch_context
    }

    fn set_terminal(
        &mut self,
        terminal: Entity<Terminal>,
        window: &mut Window,
        cx: &mut Context<TerminalView>,
    ) {
        self._terminal_subscriptions =
            subscribe_for_terminal_events(&terminal, self.workspace.clone(), window, cx);
        self.terminal = terminal;
    }

    fn rerun_button(task: &TaskState) -> Option<IconButton> {
        if !task.spawned_task.show_rerun {
            return None;
        }

        let task_id = task.spawned_task.id.clone();
        Some(
            IconButton::new("rerun-icon", IconName::Rerun)
                .icon_size(IconSize::Small)
                .size(ButtonSize::Compact)
                .icon_color(Color::Default)
                .shape(ui::IconButtonShape::Square)
                .tooltip(move |_window, cx| Tooltip::for_action("Rerun task", &RerunTask, cx))
                .on_click(move |_, window, cx| {
                    window.dispatch_action(Box::new(terminal_rerun_override(&task_id)), cx);
                }),
        )
    }
}

fn terminal_rerun_override(task: &TaskId) -> zed_actions::Rerun {
    zed_actions::Rerun {
        task_id: Some(task.0.clone()),
        allow_concurrent_runs: Some(true),
        use_new_terminal: Some(false),
        reevaluate_context: false,
    }
}

fn subscribe_for_terminal_events(
    terminal: &Entity<Terminal>,
    workspace: WeakEntity<Workspace>,
    window: &mut Window,
    cx: &mut Context<TerminalView>,
) -> Vec<Subscription> {
    let terminal_subscription = cx.observe(terminal, |_, _, cx| cx.notify());
    let mut previous_cwd = None;
    let terminal_events_subscription = cx.subscribe_in(
        terminal,
        window,
        move |terminal_view, terminal, event, window, cx| {
            let current_cwd = terminal.read(cx).working_directory();
            if current_cwd != previous_cwd {
                previous_cwd = current_cwd;
                terminal_view.needs_serialize = true;
            }

            match event {
                Event::Wakeup => {
                    cx.notify();
                    window.invalidate_character_coordinates();
                    cx.emit(Event::Wakeup);
                    cx.emit(ItemEvent::UpdateTab);
                    cx.emit(SearchEvent::MatchesInvalidated);
                }

                Event::Bell => {
                    terminal_view.has_bell = true;
                    if let TerminalBell::System = TerminalSettings::get_global(cx).bell {
                        window.play_system_bell();
                    }
                    cx.emit(Event::Wakeup);
                }

                // Marley: a notification marks the terminal as a bell does, without its sound;
                // Marley's workbench shows the notification (#478). One titled
                // `marley-event` carries Claude Code's hook events for the rail, and marks
                // nothing (#519).
                Event::MarleyNotification { title, .. } => {
                    if title.as_deref() != Some(marley_terminal::AGENT_EVENT_TITLE) {
                        terminal_view.has_bell = true;
                    }
                    cx.emit(Event::Wakeup);
                }

                Event::BlinkChanged(blinking) => {
                    terminal_view.blinking_terminal_enabled = *blinking;

                    // If in terminal-controlled mode and focused, update blink manager
                    if matches!(
                        TerminalSettings::get_global(cx).blinking,
                        TerminalBlink::TerminalControlled
                    ) && terminal_view.focus_handle.is_focused(window)
                    {
                        terminal_view.blink_manager.update(cx, |manager, cx| {
                            if *blinking {
                                manager.enable(cx);
                            } else {
                                manager.disable(cx);
                            }
                        });
                    }
                }

                Event::TitleChanged => {
                    cx.emit(ItemEvent::UpdateTab);
                }

                Event::NewNavigationTarget(maybe_navigation_target) => {
                    match maybe_navigation_target
                        .as_ref()
                        .zip(terminal.read(cx).last_content.last_hovered_word.as_ref())
                    {
                        Some((MaybeNavigationTarget::Url(url), hovered_word)) => {
                            if Some(hovered_word)
                                != terminal_view
                                    .hover
                                    .as_ref()
                                    .map(|hover| &hover.hovered_word)
                            {
                                terminal_view.hover = Some(HoverTarget {
                                    tooltip: url.clone(),
                                    hovered_word: hovered_word.clone(),
                                });
                                terminal_view.hover_tooltip_update = Task::ready(());
                                cx.notify();
                            }
                        }
                        Some((MaybeNavigationTarget::PathLike(path_like_target), hovered_word)) => {
                            if Some(hovered_word)
                                != terminal_view
                                    .hover
                                    .as_ref()
                                    .map(|hover| &hover.hovered_word)
                            {
                                terminal_view.hover = None;
                                terminal_view.hover_tooltip_update = hover_path_like_target(
                                    &workspace,
                                    hovered_word.clone(),
                                    path_like_target,
                                    cx,
                                );
                                cx.notify();
                            }
                        }
                        None => {
                            terminal_view.hover = None;
                            terminal_view.hover_tooltip_update = Task::ready(());
                            cx.notify();
                        }
                    }
                }

                Event::Open(maybe_navigation_target) => match maybe_navigation_target {
                    // Marley: Marley decides where the URL opens, and Zed's open runs when it
                    // does not (#503).
                    MaybeNavigationTarget::Url(url) => {
                        let taken =
                            cx.try_global::<MarleyTerminalUrl>()
                                .cloned()
                                .is_some_and(|hook| {
                                    let context = MarleyFooterContext {
                                        view: cx.entity().downgrade(),
                                        terminal,
                                        project: &terminal_view.project,
                                        workspace: &terminal_view.workspace,
                                        focus_handle: &terminal_view.focus_handle,
                                    };
                                    (hook.0)(&context, url, window, cx)
                                });
                        if !taken {
                            cx.open_url(url);
                        }
                    }
                    MaybeNavigationTarget::PathLike(path_like_target) => open_path_like_target(
                        &workspace,
                        terminal_view,
                        path_like_target,
                        window,
                        cx,
                    ),
                },
                Event::BreadcrumbsChanged => cx.emit(ItemEvent::UpdateBreadcrumbs),
                Event::CloseTerminal => cx.emit(ItemEvent::CloseItem),
                Event::SelectionsChanged => {
                    window.invalidate_character_coordinates();
                    cx.emit(SearchEvent::ActiveMatchChanged)
                }
            }
        },
    );
    vec![terminal_subscription, terminal_events_subscription]
}

// Marley: the grid lines of the block `terminal`'s search is held to, through the last frame's
// screen top, or none when no scope is set (#559).
fn marley_search_lines(terminal: &Entity<Terminal>, cx: &App) -> Option<std::ops::Range<i32>> {
    let index = MarleyBlockMarks::search_scope(terminal, cx)?;
    let terminal = terminal.read(cx);
    let block = terminal.blocks().get(index)?;
    let content = terminal.last_content();
    let screen_top = i64::try_from(content.marley_screen_top).ok()?;
    let cursor_line = content
        .marley_screen_top
        .saturating_add_signed(i64::from(content.cursor.point.line));
    let lines = marley_terminal::block_lines(block, cursor_line);
    let grid_line =
        |line: u64| i32::try_from(i64::try_from(line).ok()?.saturating_sub(screen_top)).ok();
    Some(grid_line(lines.start)?..grid_line(lines.end)?)
}

fn regex_search_for_query(query: &SearchQuery) -> Option<Search> {
    let str = query.as_str();
    if query.is_regex() {
        if str == "." {
            return None;
        }
        Search::new(str)
    } else {
        Search::new(&regex::escape(str))
    }
}

#[derive(Default)]
struct TerminalScrollbarSettingsWrapper;

impl ScrollbarVisibility for TerminalScrollbarSettingsWrapper {
    fn visibility(&self, cx: &App) -> scrollbars::ShowScrollbar {
        TerminalSettings::get_global(cx)
            .scrollbar
            .show
            .map(ui_scrollbar_settings_from_raw)
            .unwrap_or_else(|| EditorSettings::get_global(cx).scrollbar.show)
    }
}

impl TerminalView {
    /// Attempts to process a keystroke in the terminal. Returns true if handled.
    ///
    /// In vi mode, explicitly triggers a re-render because vi navigation (like j/k)
    /// updates the cursor locally without sending data to the shell, so there's no
    /// shell output to automatically trigger a re-render.
    fn process_keystroke(&mut self, keystroke: &Keystroke, cx: &mut Context<Self>) -> bool {
        let (handled, vi_mode_enabled) = self.terminal.update(cx, |term, cx| {
            (
                term.try_keystroke(keystroke, TerminalSettings::get_global(cx).option_as_meta),
                term.vi_mode_enabled(),
            )
        });

        if handled && vi_mode_enabled {
            cx.notify();
        }

        handled
    }

    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.clear_bell(cx);
        self.pause_cursor_blinking(window, cx);

        if event.prefer_character_input
            && event.keystroke.key_char.is_some()
            && !self.terminal.read(cx).vi_mode_enabled()
        {
            return;
        }

        if self.process_keystroke(&event.keystroke, cx) {
            cx.stop_propagation();
        }
    }

    fn focus_in(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.terminal.update(cx, |terminal, _| {
            terminal.set_cursor_shape(self.cursor_shape);
            terminal.focus_in();
        });

        let should_blink = match TerminalSettings::get_global(cx).blinking {
            TerminalBlink::Off => false,
            TerminalBlink::On => true,
            TerminalBlink::TerminalControlled => self.blinking_terminal_enabled,
        };

        if should_blink {
            self.blink_manager.update(cx, BlinkManager::enable);
        }

        window.invalidate_character_coordinates();
        cx.notify();
    }

    fn focus_out(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.blink_manager.update(cx, BlinkManager::disable);
        self.terminal.update(cx, |terminal, _| {
            terminal.focus_out();
            terminal.set_cursor_shape(CursorShape::Hollow);
        });
        cx.notify();
    }
}

impl Render for TerminalView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // TODO: this should be moved out of render
        self.scroll_handle.update(self.terminal.read(cx));

        if let Some(new_display_offset) = self.scroll_handle.future_display_offset.take() {
            self.terminal.update(cx, |term, _| {
                let delta = new_display_offset as i32 - term.last_content.display_offset as i32;
                match delta.cmp(&0) {
                    cmp::Ordering::Greater => term.scroll_up_by(delta as usize),
                    cmp::Ordering::Less => term.scroll_down_by(-delta as usize),
                    cmp::Ordering::Equal => {}
                }
            });
        }

        let terminal_handle = self.terminal.clone();
        let terminal_view_handle = cx.entity();

        let focused = self.focus_handle.is_focused(window);

        // Marley: the footer below the terminal, such as the agent bar (#477).
        let marley_footer = cx
            .try_global::<MarleyTerminalFooter>()
            .cloned()
            .and_then(|footer| {
                let context = MarleyFooterContext {
                    view: terminal_view_handle.downgrade(),
                    terminal: &self.terminal,
                    project: &self.project,
                    workspace: &self.workspace,
                    focus_handle: &self.focus_handle,
                };
                (footer.0)(&context, window, cx)
            });
        // Marley: an overlay over the grid, such as a block's filter (#528).
        let marley_overlay =
            cx.try_global::<MarleyTerminalOverlay>()
                .cloned()
                .and_then(|overlay| {
                    let context = MarleyFooterContext {
                        view: terminal_view_handle.downgrade(),
                        terminal: &self.terminal,
                        project: &self.project,
                        workspace: &self.workspace,
                        focus_handle: &self.focus_handle,
                    };
                    (overlay.0)(&context, window, cx)
                });

        div()
            .id("terminal-view")
            .size_full()
            .relative()
            // Marley: a column, so the footer takes its rows from the grid (#477).
            .flex()
            .flex_col()
            .track_focus(&self.focus_handle(cx))
            .key_context(self.dispatch_context(cx))
            .on_action(cx.listener(TerminalView::send_text))
            .on_action(cx.listener(TerminalView::send_keystroke))
            .on_action(cx.listener(TerminalView::copy))
            .on_action(cx.listener(TerminalView::editor_copy))
            .on_action(cx.listener(TerminalView::paste))
            .on_action(cx.listener(TerminalView::editor_paste))
            .on_action(cx.listener(TerminalView::paste_text))
            .on_action(cx.listener(TerminalView::clear))
            .on_action(cx.listener(TerminalView::scroll_line_up))
            .on_action(cx.listener(TerminalView::scroll_line_down))
            .on_action(cx.listener(TerminalView::scroll_page_up))
            .on_action(cx.listener(TerminalView::scroll_page_down))
            .on_action(cx.listener(TerminalView::scroll_to_top))
            .on_action(cx.listener(TerminalView::scroll_to_bottom))
            .on_action(cx.listener(TerminalView::toggle_vi_mode))
            .on_action(cx.listener(TerminalView::show_character_palette))
            .on_action(cx.listener(TerminalView::select_all))
            .on_action(cx.listener(TerminalView::rerun_task))
            .on_action(cx.listener(TerminalView::rename_terminal))
            .on_key_down(cx.listener(Self::key_down))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    if !this.terminal.read(cx).mouse_mode(event.modifiers.shift) {
                        let had_selection = this.terminal.read(cx).last_content.selection.is_some();
                        if !had_selection {
                            this.terminal.update(cx, |terminal, _| {
                                terminal.select_word_at_event_position(event);
                            });
                        }
                        let has_selection = !had_selection
                            || this
                                .terminal
                                .read(cx)
                                .last_content
                                .selection_text
                                .as_ref()
                                .is_some_and(|text| !text.is_empty());
                        this.deploy_context_menu(event.position, has_selection, window, cx);
                        cx.notify();
                    }
                }),
            )
            .child(
                // TODO: Oddly this wrapper div is needed for TerminalElement to not steal events from the context menu
                div()
                    .id("terminal-view-container")
                    // Marley: where the pane's rows end, for the driven tests (#476).
                    .debug_selector(|| "marley-terminal-view".into())
                    .size_full()
                    .bg(cx.theme().colors().editor_background)
                    .child(TerminalElement::new(
                        terminal_handle,
                        terminal_view_handle,
                        self.workspace.clone(),
                        self.focus_handle.clone(),
                        focused,
                        self.should_show_cursor(focused, cx),
                        self.block_below_cursor.clone(),
                        self.mode.clone(),
                    ))
                    .when(self.content_mode(window, cx).is_scrollable(), |div| {
                        let colors = cx.theme().colors();
                        div.custom_scrollbars(
                            Scrollbars::for_settings::<TerminalScrollbarSettingsWrapper>()
                                .show_along(ScrollAxes::Vertical)
                                .with_stable_track_along(
                                    ScrollAxes::Vertical,
                                    colors.editor_background,
                                )
                                .tracked_scroll_handle(&self.scroll_handle),
                            window,
                            cx,
                        )
                    })
                    // Marley: #528.
                    .when_some(marley_overlay, |div, overlay| div.relative().child(overlay)),
            )
            // Marley: #477.
            .children(marley_footer)
            .children(self.context_menu.as_ref().map(|(menu, position, _)| {
                deferred(
                    anchored()
                        .position(*position)
                        .anchor(gpui::Anchor::TopLeft)
                        .child(menu.clone()),
                )
                .with_priority(1)
            }))
    }
}

impl Item for TerminalView {
    type Event = ItemEvent;

    fn tab_tooltip_content(&self, cx: &App) -> Option<TabTooltipContent> {
        Some(TabTooltipContent::Custom(Box::new(Tooltip::element({
            let terminal = self.terminal().read(cx);
            let title = terminal.title(false);
            let pid = terminal.pid_getter()?.fallback_pid();

            move |_, _| {
                v_flex()
                    .gap_1()
                    .child(Label::new(title.clone()))
                    .child(h_flex().flex_grow_1().child(Divider::horizontal()))
                    .child(
                        Label::new(format!("Process ID (PID): {}", pid))
                            .color(Color::Muted)
                            .size(LabelSize::Small),
                    )
                    .into_any_element()
            }
        }))))
    }

    fn tab_content(&self, params: TabContentParams, _window: &Window, cx: &App) -> AnyElement {
        let terminal = self.terminal().read(cx);
        let title = self
            .custom_title
            .as_ref()
            .filter(|title| !title.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| terminal.title(true));

        let (icon, icon_color, rerun_button) = match terminal.task() {
            Some(terminal_task) => match &terminal_task.status {
                TaskStatus::Running => (
                    IconName::PlayFilled,
                    Color::Disabled,
                    TerminalView::rerun_button(terminal_task),
                ),
                TaskStatus::Unknown => (
                    IconName::Warning,
                    Color::Warning,
                    TerminalView::rerun_button(terminal_task),
                ),
                TaskStatus::Completed { success } => {
                    let rerun_button = TerminalView::rerun_button(terminal_task);

                    if *success {
                        (IconName::Check, Color::Success, rerun_button)
                    } else {
                        (IconName::XCircle, Color::Error, rerun_button)
                    }
                }
            },
            None => (IconName::Terminal, Color::Muted, None),
        };

        let self_handle = self.self_handle.clone();
        h_flex()
            .gap_1()
            .group("term-tab-icon")
            .when(!params.selected, |this| {
                this.track_focus(&self.focus_handle)
            })
            .on_action(move |action: &RenameTerminal, window, cx| {
                self_handle
                    .update(cx, |this, cx| this.rename_terminal(action, window, cx))
                    .ok();
            })
            .child(
                h_flex()
                    .group("term-tab-icon")
                    .child(
                        div()
                            .when(rerun_button.is_some(), |this| {
                                this.hover(|style| style.invisible().w_0())
                            })
                            .child(Icon::new(icon).color(icon_color)),
                    )
                    .when_some(rerun_button, |this, rerun_button| {
                        this.child(
                            div()
                                .absolute()
                                .visible_on_hover("term-tab-icon")
                                .child(rerun_button),
                        )
                    }),
            )
            .child(
                div()
                    .relative()
                    .child(
                        Label::new(title)
                            .single_line()
                            .color(params.text_color())
                            .when(self.is_renaming(), |this| this.alpha(0.)),
                    )
                    .when_some(self.rename_editor.clone(), |this, editor| {
                        let self_handle = self.self_handle.clone();
                        let self_handle_cancel = self.self_handle.clone();
                        this.child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .size_full()
                                .child(editor)
                                .on_action(move |_: &menu::Confirm, window, cx| {
                                    self_handle
                                        .update(cx, |this, cx| {
                                            this.finish_renaming(true, window, cx)
                                        })
                                        .ok();
                                })
                                .on_action(move |_: &menu::Cancel, window, cx| {
                                    self_handle_cancel
                                        .update(cx, |this, cx| {
                                            this.finish_renaming(false, window, cx)
                                        })
                                        .ok();
                                }),
                        )
                    }),
            )
            .into_any()
    }

    fn tab_content_text(&self, detail: usize, cx: &App) -> SharedString {
        if let Some(custom_title) = self.custom_title.as_ref().filter(|l| !l.trim().is_empty()) {
            return custom_title.clone().into();
        }
        let terminal = self.terminal().read(cx);
        terminal.title(detail == 0).into()
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        None
    }

    fn handle_drop(
        &self,
        active_pane: &Pane,
        dropped: &dyn Any,
        window: &mut Window,
        cx: &mut App,
    ) -> bool {
        let Some(project) = self.project.upgrade() else {
            return false;
        };

        if let Some(paths) = dropped.downcast_ref::<ExternalPaths>() {
            let is_local = project.read(cx).is_local();
            if is_local {
                self.add_paths_to_terminal(paths.paths(), window, cx);
                return true;
            }

            return false;
        } else if let Some(tab) = dropped.downcast_ref::<DraggedTab>() {
            let Some(self_handle) = self.self_handle.upgrade() else {
                return false;
            };

            let Some(workspace) = self.workspace.upgrade() else {
                return false;
            };

            let Some(this_pane) = workspace.read(cx).pane_for(&self_handle) else {
                return false;
            };

            let item = if tab.pane == this_pane {
                active_pane.item_for_index(tab.ix)
            } else {
                tab.pane.read(cx).item_for_index(tab.ix)
            };

            let Some(item) = item else {
                return false;
            };

            if item.downcast::<TerminalView>().is_some() {
                let Some(split_direction) = active_pane.drag_split_direction() else {
                    return false;
                };

                let Some(terminal_panel) = workspace.read(cx).panel::<TerminalPanel>(cx) else {
                    return false;
                };

                if !terminal_panel.read(cx).center.panes().contains(&&this_pane) {
                    return false;
                }

                let source = tab.pane.clone();
                let item_id_to_move = item.item_id();
                let is_zoomed = {
                    let terminal_panel = terminal_panel.read(cx);
                    if terminal_panel.active_pane == this_pane {
                        active_pane.is_zoomed()
                    } else {
                        terminal_panel.active_pane.read(cx).is_zoomed()
                    }
                };

                let workspace = workspace.downgrade();
                let terminal_panel = terminal_panel.downgrade();
                // Defer the split operation to avoid re-entrancy panic.
                // The pane may be the one currently being updated, so we cannot
                // call mark_positions (via split) synchronously.
                window
                    .spawn(cx, async move |cx| {
                        cx.update(|window, cx| {
                            let Ok(new_pane) = terminal_panel.update(cx, |terminal_panel, cx| {
                                let new_pane = terminal_panel::new_terminal_pane(
                                    workspace, project, is_zoomed, window, cx,
                                );
                                terminal_panel.apply_tab_bar_buttons(&new_pane, cx);
                                terminal_panel.center.split(
                                    &this_pane,
                                    &new_pane,
                                    split_direction,
                                    cx,
                                );
                                anyhow::Ok(new_pane)
                            }) else {
                                return;
                            };

                            let Some(new_pane) = new_pane.log_err() else {
                                return;
                            };

                            workspace::move_item(
                                &source,
                                &new_pane,
                                item_id_to_move,
                                new_pane.read(cx).active_item_index(),
                                true,
                                window,
                                cx,
                            );
                        })
                        .ok();
                    })
                    .detach();

                return true;
            } else {
                if let Some(project_path) = item.project_path(cx)
                    && let Some(path) = project.read(cx).absolute_path(&project_path, cx)
                {
                    self.add_paths_to_terminal(&[path], window, cx);
                    return true;
                }
            }

            return false;
        } else if let Some(selection) = dropped.downcast_ref::<DraggedSelection>() {
            let project = project.read(cx);
            let paths = selection
                .items()
                .map(|selected_entry| selected_entry.entry_id)
                .filter_map(|entry_id| project.path_for_entry(entry_id, cx))
                .filter_map(|project_path| project.absolute_path(&project_path, cx))
                .collect::<Vec<_>>();

            if !paths.is_empty() {
                self.add_paths_to_terminal(&paths, window, cx);
            }

            return true;
        } else if let Some(&entry_id) = dropped.downcast_ref::<ProjectEntryId>() {
            let project = project.read(cx);
            if let Some(path) = project
                .path_for_entry(entry_id, cx)
                .and_then(|project_path| project.absolute_path(&project_path, cx))
            {
                self.add_paths_to_terminal(&[path], window, cx);
            }

            return true;
        }

        false
    }

    fn tab_extra_context_menu_actions(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<(SharedString, Box<dyn gpui::Action>)> {
        let terminal = self.terminal.read(cx);
        if terminal.task().is_none() {
            vec![("Rename".into(), Box::new(RenameTerminal))]
        } else {
            Vec::new()
        }
    }

    fn buffer_kind(&self, _: &App) -> workspace::item::ItemBufferKind {
        workspace::item::ItemBufferKind::Singleton
    }

    fn can_split(&self) -> bool {
        true
    }

    fn clone_on_split(
        &self,
        workspace_id: Option<WorkspaceId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Option<Entity<Self>>> {
        let Ok(terminal) = self.project.update(cx, |project, cx| {
            let cwd = project
                .active_project_directory(cx)
                .map(|it| it.to_path_buf());
            project.clone_terminal(self.terminal(), cx, cwd)
        }) else {
            return Task::ready(None);
        };
        cx.spawn_in(window, async move |this, cx| {
            let terminal = terminal.await.log_err()?;
            this.update_in(cx, |this, window, cx| {
                cx.new(|cx| {
                    TerminalView::new(
                        terminal,
                        this.workspace.clone(),
                        workspace_id,
                        this.project.clone(),
                        window,
                        cx,
                    )
                })
            })
            .ok()
        })
    }

    fn is_dirty(&self, cx: &App) -> bool {
        match self.terminal.read(cx).task() {
            Some(task) => task.status == TaskStatus::Running,
            None => self.has_bell(),
        }
    }

    fn has_conflict(&self, _cx: &App) -> bool {
        false
    }

    fn can_save_as(&self, _cx: &App) -> bool {
        false
    }

    fn as_searchable(
        &self,
        handle: &Entity<Self>,
        _: &App,
    ) -> Option<Box<dyn SearchableItemHandle>> {
        Some(Box::new(handle.clone()))
    }

    fn breadcrumb_location(&self, cx: &App) -> ToolbarItemLocation {
        if self.show_breadcrumbs && !self.terminal().read(cx).breadcrumb_text.trim().is_empty() {
            ToolbarItemLocation::PrimaryLeft
        } else {
            ToolbarItemLocation::Hidden
        }
    }

    fn breadcrumbs(&self, cx: &App) -> Option<(Vec<HighlightedText>, Option<Font>)> {
        Some((
            vec![HighlightedText {
                text: self.terminal().read(cx).breadcrumb_text.clone().into(),
                highlights: vec![],
            }],
            None,
        ))
    }

    fn added_to_workspace(
        &mut self,
        workspace: &mut Workspace,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Marley: the rail moves a terminal to another project's workspace (#613); the view then
        // acts in that project, its links, menu and events included.
        let joined = workspace.weak_handle();
        if joined != self.workspace {
            self.project = workspace.project().downgrade();
            self._terminal_subscriptions =
                subscribe_for_terminal_events(&self.terminal, joined.clone(), window, cx);
            self.workspace = joined;
        }
        if self.terminal().read(cx).task().is_none() {
            if let Some((new_id, old_id)) = workspace.database_id().zip(self.workspace_id) {
                log::debug!(
                    "Updating workspace id for the terminal, old: {old_id:?}, new: {new_id:?}",
                );
                let db = TerminalDb::global(cx);
                let entity_id = cx.entity_id().as_u64();
                cx.background_spawn(async move {
                    db.update_workspace_id(new_id, old_id, entity_id).await
                })
                .detach();
                // Marley: the terminal's id follows it to the new workspace id (#575).
                if let Some(identity) = cx.try_global::<MarleyTerminalIdentity>().cloned() {
                    (identity.moved)(new_id, old_id, entity_id, cx).detach_and_log_err(cx);
                }
            }
            self.workspace_id = workspace.database_id();
        }
    }

    fn to_item_events(event: &Self::Event, f: &mut dyn FnMut(ItemEvent)) {
        f(*event)
    }
}

impl SerializableItem for TerminalView {
    fn serialized_item_kind() -> &'static str {
        "Terminal"
    }

    fn cleanup(
        workspace_id: WorkspaceId,
        alive_items: Vec<workspace::ItemId>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Task<anyhow::Result<()>> {
        // Marley: the ids of the items not loaded go with their rows (#575).
        if let Some(identity) = cx.try_global::<MarleyTerminalIdentity>().cloned() {
            (identity.cleanup)(workspace_id, alive_items.clone(), cx).detach_and_log_err(cx);
        }
        let db = TerminalDb::global(cx);
        delete_unloaded_items(alive_items, workspace_id, "terminals", &db, cx)
    }

    fn serialize(
        &mut self,
        _workspace: &mut Workspace,
        item_id: workspace::ItemId,
        _closing: bool,
        cx: &mut Context<Self>,
    ) -> Option<Task<anyhow::Result<()>>> {
        let terminal = self.terminal().read(cx);
        if terminal.task().is_some() {
            return None;
        }

        if !self.needs_serialize {
            return None;
        }

        let workspace_id = self.workspace_id?;
        let cwd = terminal.working_directory();
        let custom_title = self.custom_title.clone();
        self.needs_serialize = false;
        // Marley: the terminal's id is saved with the rest, and awaited with it, so a quit keeps
        // it (#575).
        let marley_id = terminal.marley_terminal_id().map(str::to_string);
        let marley_save = marley_id
            .zip(cx.try_global::<MarleyTerminalIdentity>().cloned())
            .map(|(id, identity)| (identity.save)(workspace_id, item_id, id, cx));

        let db = TerminalDb::global(cx);
        Some(cx.background_spawn(async move {
            if let Some(cwd) = cwd {
                db.save_working_directory(item_id, workspace_id, cwd)
                    .await?;
            }
            db.save_custom_title(item_id, workspace_id, custom_title)
                .await?;
            if let Some(marley_save) = marley_save {
                marley_save.await?;
            }
            Ok(())
        }))
    }

    fn should_serialize(&self, _: &Self::Event) -> bool {
        self.needs_serialize
    }

    fn deserialize(
        project: Entity<Project>,
        workspace: WeakEntity<Workspace>,
        workspace_id: WorkspaceId,
        item_id: workspace::ItemId,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<anyhow::Result<Entity<Self>>> {
        window.spawn(cx, async move |cx| {
            let (cwd, custom_title) = cx
                .update(|_window, cx| {
                    let db = TerminalDb::global(cx);
                    let from_db = db
                        .get_working_directory(item_id, workspace_id)
                        .log_err()
                        .flatten();
                    let cwd = if from_db
                        .as_ref()
                        .is_some_and(|from_db| !from_db.as_os_str().is_empty())
                    {
                        from_db
                    } else {
                        workspace
                            .upgrade()
                            .and_then(|workspace| default_working_directory(workspace.read(cx), cx))
                    };
                    let custom_title = db
                        .get_custom_title(item_id, workspace_id)
                        .log_err()
                        .flatten()
                        .filter(|title| !title.trim().is_empty());
                    (cwd, custom_title)
                })
                .ok()
                .unwrap_or((None, None));
            // Marley: the id the terminal had, which the restored terminal keeps (#575).
            let marley_terminal_id = cx
                .update(|_window, cx| {
                    cx.try_global::<MarleyTerminalIdentity>()
                        .and_then(|identity| (identity.saved)(workspace_id, item_id, cx))
                })
                .ok()
                .flatten();

            let terminal = project
                .update(cx, |project, cx| {
                    project.create_terminal_shell_restoring(cwd, marley_terminal_id, cx)
                })
                .await?;
            cx.update(|window, cx| {
                cx.new(|cx| {
                    let mut view = TerminalView::new(
                        terminal,
                        workspace,
                        Some(workspace_id),
                        project.downgrade(),
                        window,
                        cx,
                    );
                    if custom_title.is_some() {
                        view.custom_title = custom_title;
                    }
                    view
                })
            })
        })
    }
}

impl SearchableItem for TerminalView {
    type Match = Range;

    fn supported_options(&self) -> SearchOptions {
        SearchOptions {
            case: false,
            word: false,
            regex: true,
            replacement: false,
            selection: false,
            select_all: false,
            find_in_results: false,
        }
    }

    /// Clear stored matches
    fn clear_matches(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.terminal().update(cx, |term, _| term.matches.clear())
    }

    /// Store matches returned from find_matches somewhere for rendering
    fn update_matches(
        &mut self,
        matches: &[Self::Match],
        _active_match_index: Option<usize>,
        _token: SearchToken,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.terminal()
            .update(cx, |term, _| term.matches = matches.to_vec())
    }

    /// Returns the selection content to pre-load into this search
    fn query_suggestion(
        &mut self,
        _seed_query_override: Option<SeedQuerySetting>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> String {
        self.terminal()
            .read(cx)
            .last_content
            .selection_text
            .clone()
            .unwrap_or_default()
    }

    /// Focus match at given index into the Vec of matches
    fn activate_match(
        &mut self,
        index: usize,
        _: &[Self::Match],
        _token: SearchToken,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.terminal()
            .update(cx, |term, _| term.activate_match(index));
        cx.notify();
    }

    /// Add selections for all matches given.
    fn select_matches(
        &mut self,
        matches: &[Self::Match],
        _token: SearchToken,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.terminal()
            .update(cx, |term, _| term.select_matches(matches));
        cx.notify();
    }

    /// Get all of the matches for this query, should be done on the background
    fn find_matches(
        &mut self,
        query: Arc<SearchQuery>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Vec<Self::Match>> {
        if let Some(s) = regex_search_for_query(&query) {
            // Marley: a search held to one block keeps the matches starting in its lines (#559).
            if let Some(lines) = marley_search_lines(self.terminal(), cx) {
                let matches = self
                    .terminal()
                    .update(cx, |term, cx| term.find_matches(s, cx));
                return cx.background_spawn(async move {
                    let mut matches = matches.await;
                    matches.retain(|found| lines.contains(&found.start().line));
                    matches
                });
            }
            self.terminal()
                .update(cx, |term, cx| term.find_matches(s, cx))
        } else {
            Task::ready(vec![])
        }
    }

    // Marley: a closed search bar ends the search held to one block (#559).
    fn search_bar_visibility_changed(
        &mut self,
        visible: bool,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let terminal = self.terminal().entity_id();
        if !visible && cx.has_global::<MarleyBlockMarks>() {
            cx.global_mut::<MarleyBlockMarks>()
                .search_scopes
                .remove(&terminal);
            cx.notify();
        }
    }

    /// Reports back to the search toolbar what the active match should be (the selection)
    fn active_match_index(
        &mut self,
        direction: Direction,
        matches: &[Self::Match],
        _token: SearchToken,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<usize> {
        // Selection head might have a value if there's a selection that isn't
        // associated with a match. Therefore, if there are no matches, we should
        // report None, no matter the state of the terminal

        if !matches.is_empty() {
            if let Some(selection_head) = self.terminal().read(cx).selection_head {
                // If selection head is contained in a match. Return that match
                match direction {
                    Direction::Prev => {
                        // If no selection before selection head, return the first match
                        Some(
                            matches
                                .iter()
                                .enumerate()
                                .rev()
                                .find(|(_, search_match)| {
                                    search_match.contains(selection_head)
                                        || search_match.start() < selection_head
                                })
                                .map(|(ix, _)| ix)
                                .unwrap_or(0),
                        )
                    }
                    Direction::Next => {
                        // If no selection after selection head, return the last match
                        Some(
                            matches
                                .iter()
                                .enumerate()
                                .find(|(_, search_match)| {
                                    search_match.contains(selection_head)
                                        || search_match.start() > selection_head
                                })
                                .map(|(ix, _)| ix)
                                .unwrap_or(matches.len().saturating_sub(1)),
                        )
                    }
                }
            } else {
                // Matches found but no active selection, return the first last one (closest to cursor)
                Some(matches.len().saturating_sub(1))
            }
        } else {
            None
        }
    }
    fn replace(
        &mut self,
        _: &Self::Match,
        _: &SearchQuery,
        _token: SearchToken,
        _window: &mut Window,
        _: &mut Context<Self>,
    ) {
        // Replacement is not supported in terminal view, so this is a no-op.
    }
}

/// Gets the working directory for the given workspace, respecting the user's settings.
/// Falls back to home directory when no project directory is available.
///
/// For remote projects, local-only resolution (home dir fallback, shell expansion,
/// local `is_dir` checks) is skipped -- returning `None` lets the remote shell
/// open in the remote user's home directory by default.
pub fn default_working_directory(workspace: &Workspace, cx: &App) -> Option<PathBuf> {
    let is_remote = workspace.project().read(cx).is_remote();
    let directory = match &TerminalSettings::get_global(cx).working_directory {
        WorkingDirectory::CurrentFileDirectory => workspace
            .project()
            .read(cx)
            .active_entry_directory(cx)
            .or_else(|| current_project_directory(workspace, cx)),
        WorkingDirectory::CurrentProjectDirectory => current_project_directory(workspace, cx),
        WorkingDirectory::FirstProjectDirectory => first_project_directory(workspace, cx),
        WorkingDirectory::AlwaysHome => None,
        WorkingDirectory::Always { directory } if !is_remote => shellexpand::full(directory)
            .ok()
            .map(|dir| Path::new(&dir.to_string()).to_path_buf())
            .filter(|dir| dir.is_dir()),
        WorkingDirectory::Always { .. } => None,
    };

    if is_remote {
        directory
    } else {
        directory.or_else(dirs::home_dir)
    }
}

fn current_project_directory(workspace: &Workspace, cx: &App) -> Option<PathBuf> {
    workspace
        .project()
        .read(cx)
        .active_project_directory(cx)
        .as_deref()
        .map(Path::to_path_buf)
        .or_else(|| first_project_directory(workspace, cx))
}

///Gets the first project's home directory, or the home directory
fn first_project_directory(workspace: &Workspace, cx: &App) -> Option<PathBuf> {
    let worktree = workspace.worktrees(cx).next()?.read(cx);
    let worktree_path = worktree.abs_path();
    if worktree.root_entry()?.is_dir() {
        Some(worktree_path.to_path_buf())
    } else {
        // If worktree is a file, return its parent directory
        worktree_path.parent().map(|p| p.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{TestAppContext, UpdateGlobal, VisualTestContext};
    use project::{Entry, Project, ProjectPath, Worktree};
    use remote::RemoteClient;
    use std::path::{Path, PathBuf};
    use util::paths::PathStyle;
    use util::rel_path::RelPath;
    use workspace::item::test::{TestItem, TestProjectItem};
    use workspace::{AppState, MultiWorkspace, SelectedEntry};

    // Marley: the title and panel tests start the system shell, whose Marley scripts install in a
    // scratch data directory (#475).
    // SAFETY: before `main` it reads the binary's path and sets `paths`' `OnceLock`, nothing else.
    #[ctor::ctor(unsafe)]
    fn marley_test_data_dir() {
        terminal::marley_use_test_data_dir();
    }

    fn expected_drop_text(paths: &[PathBuf]) -> String {
        let mut text = String::new();
        for path in paths {
            text.push(' ');
            text.push_str(&shlex::try_quote(path.to_str().unwrap()).unwrap());
        }
        text.push(' ');
        text
    }

    fn assert_drop_writes_to_terminal(
        pane: &Entity<Pane>,
        terminal_view_index: usize,
        terminal: &Entity<Terminal>,
        dropped: &dyn Any,
        expected_text: &str,
        window: &mut Window,
        cx: &mut Context<MultiWorkspace>,
    ) {
        let _ = terminal.update(cx, |terminal, _| terminal.take_input_log());

        let handled = pane.update(cx, |pane, cx| {
            pane.item_for_index(terminal_view_index)
                .unwrap()
                .handle_drop(pane, dropped, window, cx)
        });
        assert!(handled, "handle_drop should return true for {:?}", dropped);

        let mut input_log = terminal.update(cx, |terminal, _| terminal.take_input_log());
        assert_eq!(input_log.len(), 1, "expected exactly one write to terminal");
        let written =
            String::from_utf8(input_log.remove(0)).expect("terminal write should be valid UTF-8");
        assert_eq!(written, expected_text);
    }

    // DEC private mode 1049: a program writes this to enter the alternate screen buffer.
    const ENTER_ALT_SCREEN: &[u8] = b"\x1b[?1049h";

    // CSI `1;2A` = cursor-up with the xterm Shift modifier (`1 + 1` for Shift).
    const SHIFT_UP_ESCAPE: &[u8] = b"\x1b[1;2A";

    #[gpui::test]
    async fn edit_menu_copy_and_paste_are_available_when_terminal_is_focused(
        cx: &mut TestAppContext,
    ) {
        let (project, _workspace, window_handle) = init_test_with_window(cx).await;
        let (_pane, terminal, _terminal_view) =
            add_display_only_terminal(&project, window_handle, true, cx);

        let mut cx = VisualTestContext::from_window(window_handle.into(), cx);
        cx.update(|window, cx| {
            let _ = window.draw(cx);
            assert!(window.is_action_available(&editor::actions::Copy, cx));
            assert!(window.is_action_available(&editor::actions::Paste, cx));

            cx.write_to_clipboard(gpui::ClipboardItem::new_string("foo".to_string()));
            terminal.update(cx, |terminal, _| terminal.take_input_log());
            window.dispatch_action(Box::new(editor::actions::Paste), cx);
        });
        cx.run_until_parked();

        cx.update(|_, cx| {
            let input_log = terminal.update(cx, |terminal, _| terminal.take_input_log());
            assert_eq!(input_log, vec![b"foo".to_vec()]);
        });
    }

    #[gpui::test]
    async fn shift_up_scrolls_history_in_normal_screen(cx: &mut TestAppContext) {
        let (project, _workspace, window_handle) = init_test_with_window(cx).await;
        cx.update(load_default_keymap);
        let (_pane, terminal, _terminal_view) =
            add_display_only_terminal(&project, window_handle, true, cx);

        let mut cx = VisualTestContext::from_window(window_handle.into(), cx);
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        cx.run_until_parked();

        let output = (0..200)
            .map(|line| format!("line {line}\n"))
            .collect::<String>();
        cx.update(|window, cx| {
            terminal.update(cx, |terminal, cx| {
                terminal.write_output(output.as_bytes(), cx);
                terminal.sync(window, cx);
            });
        });
        terminal.read_with(&cx, |terminal, _| {
            assert!(!terminal.last_content.mode.contains(Modes::ALT_SCREEN));
            assert_eq!(terminal.last_content.display_offset, 0);
        });

        cx.simulate_keystrokes("shift-up");
        cx.update(|window, cx| {
            terminal.update(cx, |terminal, cx| terminal.sync(window, cx));
        });

        assert_eq!(
            terminal.read_with(&cx, |terminal, _| terminal.last_content.display_offset),
            1,
            "shift-up should scroll terminal history in the normal screen",
        );
        assert!(
            terminal
                .update(&mut cx, |terminal, _| terminal.take_input_log())
                .is_empty(),
            "shift-up in the normal screen should not be forwarded to the shell",
        );
    }

    #[gpui::test]
    async fn shift_up_is_forwarded_to_program_in_alt_screen(cx: &mut TestAppContext) {
        let (project, _workspace, window_handle) = init_test_with_window(cx).await;
        cx.update(load_default_keymap);
        let (_pane, terminal, _terminal_view) =
            add_display_only_terminal(&project, window_handle, true, cx);

        let mut cx = VisualTestContext::from_window(window_handle.into(), cx);
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        cx.run_until_parked();

        cx.update(|window, cx| {
            terminal.update(cx, |terminal, cx| {
                terminal.write_output(ENTER_ALT_SCREEN, cx);
                terminal.sync(window, cx);
            });
        });
        terminal.read_with(&cx, |terminal, _| {
            assert!(terminal.last_content.mode.contains(Modes::ALT_SCREEN));
        });

        cx.simulate_keystrokes("shift-up");
        assert_eq!(
            terminal.update(&mut cx, |terminal, _| terminal.take_input_log()),
            vec![SHIFT_UP_ESCAPE.to_vec()],
            "shift-up should be forwarded to the program in the alternate screen",
        );
    }

    #[cfg(target_os = "linux")]
    #[gpui::test]
    async fn ctrl_q_is_forwarded_to_terminal_not_quit(cx: &mut TestAppContext) {
        let (project, _workspace, window_handle) = init_test_with_window(cx).await;
        cx.update(load_default_keymap);
        let (_pane, terminal, _terminal_view) =
            add_display_only_terminal(&project, window_handle, true, cx);

        let mut cx = VisualTestContext::from_window(window_handle.into(), cx);
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        cx.run_until_parked();

        cx.simulate_keystrokes("ctrl-q");
        assert_eq!(
            terminal.update(&mut cx, |terminal, _| terminal.take_input_log()),
            vec![vec![0x11]],
            "ctrl-q in a focused terminal should send 0x11 to the PTY, not trigger zed::Quit",
        );
    }

    #[gpui::test]
    async fn altgr_character_input_is_not_swallowed_as_meta_sequence(cx: &mut TestAppContext) {
        let (project, _workspace, window_handle) = init_test_with_window(cx).await;
        cx.update(|cx| {
            SettingsStore::update_global(cx, |store, cx| {
                store.update_user_settings(cx, |settings| {
                    settings.terminal.get_or_insert_default().option_as_meta = Some(true);
                });
            });
        });
        let (_pane, terminal, _terminal_view) =
            add_display_only_terminal(&project, window_handle, true, cx);

        let mut cx = VisualTestContext::from_window(window_handle.into(), cx);
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        cx.run_until_parked();

        let mut altgr_a = Keystroke::parse("ctrl-alt-a").unwrap();
        altgr_a.key_char = Some("ą".to_string());

        let propagate = cx.update(|window, cx| {
            window
                .dispatch_event(
                    gpui::PlatformInput::KeyDown(KeyDownEvent {
                        keystroke: altgr_a.clone(),
                        is_held: false,
                        prefer_character_input: true,
                    }),
                    cx,
                )
                .propagate
        });
        assert!(
            propagate,
            "a keystroke that produced a character must propagate so the platform can commit the text",
        );
        assert_eq!(
            terminal.update(&mut cx, |terminal, _| terminal.take_input_log()),
            Vec::<Vec<u8>>::new(),
            "AltGr-produced characters must not be sent as meta escape sequences",
        );

        let propagate = cx.update(|window, cx| {
            window
                .dispatch_event(
                    gpui::PlatformInput::KeyDown(KeyDownEvent {
                        keystroke: altgr_a,
                        is_held: false,
                        prefer_character_input: false,
                    }),
                    cx,
                )
                .propagate
        });
        assert!(!propagate);
        assert_eq!(
            terminal.update(&mut cx, |terminal, _| terminal.take_input_log()),
            vec![b"\x1b\x01".to_vec()],
            "ctrl-alt-a without character input preference is still sent as meta ctrl-a",
        );
    }

    // Working directory calculation tests

    // No Worktrees in project -> home_dir()
    #[gpui::test]
    async fn no_worktree(cx: &mut TestAppContext) {
        let (project, workspace) = init_test(cx).await;
        cx.read(|cx| {
            let workspace = workspace.read(cx);
            let active_entry = project.read(cx).active_entry();

            //Make sure environment is as expected
            assert!(active_entry.is_none());
            assert!(workspace.worktrees(cx).next().is_none());

            let res = default_working_directory(workspace, cx);
            assert_eq!(res, dirs::home_dir());
            let res = first_project_directory(workspace, cx);
            assert_eq!(res, None);
        });
    }

    #[gpui::test]
    async fn remote_no_worktree_uses_remote_shell_default_cwd(
        cx: &mut TestAppContext,
        server_cx: &mut TestAppContext,
    ) {
        let (_project, workspace) = init_remote_test(cx, server_cx).await;

        cx.read(|cx| {
            let workspace = workspace.read(cx);

            assert!(workspace.project().read(cx).is_remote());
            assert!(workspace.worktrees(cx).next().is_none());
            assert_eq!(default_working_directory(workspace, cx), None);
        });
    }

    // No active entry, but a worktree, worktree is a file -> parent directory
    #[gpui::test]
    async fn no_active_entry_worktree_is_file(cx: &mut TestAppContext) {
        let (project, workspace) = init_test(cx).await;

        create_file_wt(project.clone(), "/root.txt", cx).await;
        cx.read(|cx| {
            let workspace = workspace.read(cx);
            let active_entry = project.read(cx).active_entry();

            //Make sure environment is as expected
            assert!(active_entry.is_none());
            assert!(workspace.worktrees(cx).next().is_some());

            let res = default_working_directory(workspace, cx);
            assert_eq!(res, Some(Path::new("/").to_path_buf()));
            let res = first_project_directory(workspace, cx);
            assert_eq!(res, Some(Path::new("/").to_path_buf()));
        });
    }

    // No active entry, but a worktree, worktree is a folder -> worktree_folder
    #[gpui::test]
    async fn no_active_entry_worktree_is_dir(cx: &mut TestAppContext) {
        let (project, workspace) = init_test(cx).await;

        let (_wt, _entry) = create_folder_wt(project.clone(), "/root/", cx).await;
        cx.update(|cx| {
            let workspace = workspace.read(cx);
            let active_entry = project.read(cx).active_entry();

            assert!(active_entry.is_none());
            assert!(workspace.worktrees(cx).next().is_some());

            let res = default_working_directory(workspace, cx);
            assert_eq!(res, Some(Path::new("/root/").to_path_buf()));
            let res = first_project_directory(workspace, cx);
            assert_eq!(res, Some(Path::new("/root/").to_path_buf()));
        });
    }

    // Active entry with a work tree, worktree is a file -> worktree_folder()
    #[gpui::test]
    async fn active_entry_worktree_is_file(cx: &mut TestAppContext) {
        let (project, workspace) = init_test(cx).await;

        let (_wt, _entry) = create_folder_wt(project.clone(), "/root1/", cx).await;
        let (wt2, entry2) = create_file_wt(project.clone(), "/root2.txt", cx).await;
        insert_active_entry_for(wt2, entry2, project.clone(), cx);

        cx.update(|cx| {
            let workspace = workspace.read(cx);
            let active_entry = project.read(cx).active_entry();

            assert!(active_entry.is_some());

            let res = default_working_directory(workspace, cx);
            assert_eq!(res, Some(Path::new("/root1/").to_path_buf()));
            let res = first_project_directory(workspace, cx);
            assert_eq!(res, Some(Path::new("/root1/").to_path_buf()));
        });
    }

    // Active entry, with a worktree, worktree is a folder -> worktree_folder
    #[gpui::test]
    async fn active_entry_worktree_is_dir(cx: &mut TestAppContext) {
        let (project, workspace) = init_test(cx).await;

        let (_wt, _entry) = create_folder_wt(project.clone(), "/root1/", cx).await;
        let (wt2, entry2) = create_folder_wt(project.clone(), "/root2/", cx).await;
        insert_active_entry_for(wt2, entry2, project.clone(), cx);

        cx.update(|cx| {
            let workspace = workspace.read(cx);
            let active_entry = project.read(cx).active_entry();

            assert!(active_entry.is_some());

            let res = default_working_directory(workspace, cx);
            assert_eq!(res, Some(Path::new("/root2/").to_path_buf()));
            let res = first_project_directory(workspace, cx);
            assert_eq!(res, Some(Path::new("/root1/").to_path_buf()));
        });
    }

    // active_entry_directory: No active entry -> returns None (used by CurrentFileDirectory)
    #[gpui::test]
    async fn active_entry_directory_no_active_entry(cx: &mut TestAppContext) {
        let (project, _workspace) = init_test(cx).await;

        let (_wt, _entry) = create_folder_wt(project.clone(), "/root/", cx).await;

        cx.update(|cx| {
            assert!(project.read(cx).active_entry().is_none());

            let res = project.read(cx).active_entry_directory(cx);
            assert_eq!(res, None);
        });
    }

    // active_entry_directory: Active entry is file -> returns parent directory (used by CurrentFileDirectory)
    #[gpui::test]
    async fn active_entry_directory_active_file(cx: &mut TestAppContext) {
        let (project, _workspace) = init_test(cx).await;

        let (wt, _entry) = create_folder_wt(project.clone(), "/root/", cx).await;
        let entry = create_file_in_worktree(wt.clone(), "src/main.rs", cx).await;
        insert_active_entry_for(wt, entry, project.clone(), cx);

        cx.update(|cx| {
            let res = project.read(cx).active_entry_directory(cx);
            assert_eq!(res, Some(Path::new("/root/src").to_path_buf()));
        });
    }

    // active_entry_directory: Active entry is directory -> returns that directory (used by CurrentFileDirectory)
    #[gpui::test]
    async fn active_entry_directory_active_dir(cx: &mut TestAppContext) {
        let (project, _workspace) = init_test(cx).await;

        let (wt, entry) = create_folder_wt(project.clone(), "/root/", cx).await;
        insert_active_entry_for(wt, entry, project.clone(), cx);

        cx.update(|cx| {
            let res = project.read(cx).active_entry_directory(cx);
            assert_eq!(res, Some(Path::new("/root/").to_path_buf()));
        });
    }

    /// Creates a worktree with 1 file: /root.txt
    pub async fn init_test(cx: &mut TestAppContext) -> (Entity<Project>, Entity<Workspace>) {
        let (project, workspace, _) = init_test_with_window(cx).await;
        (project, workspace)
    }

    fn load_default_keymap(cx: &mut App) {
        cx.bind_keys(
            settings::KeymapFile::load_asset_allow_partial_failure(
                settings::DEFAULT_KEYMAP_PATH,
                cx,
            )
            .unwrap(),
        );
    }

    fn add_display_only_terminal(
        project: &Entity<Project>,
        window_handle: gpui::WindowHandle<MultiWorkspace>,
        focus: bool,
        cx: &mut TestAppContext,
    ) -> (Entity<Pane>, Entity<Terminal>, Entity<TerminalView>) {
        let project = project.clone();
        window_handle
            .update(cx, |multi_workspace, window, cx| {
                let workspace = multi_workspace.workspace().clone();
                let active_pane = workspace.read(cx).active_pane().clone();

                let terminal = cx.new(|cx| {
                    terminal::TerminalBuilder::new_display_only(
                        CursorShape::default(),
                        terminal::terminal_settings::AlternateScroll::On,
                        None,
                        0,
                        cx.background_executor(),
                        PathStyle::local(),
                    )
                    .subscribe(cx)
                });
                let terminal_view = cx.new(|cx| {
                    TerminalView::new(
                        terminal.clone(),
                        workspace.downgrade(),
                        None,
                        project.downgrade(),
                        window,
                        cx,
                    )
                });

                active_pane.update(cx, |pane, cx| {
                    pane.add_item(
                        Box::new(terminal_view.clone()),
                        true,
                        false,
                        None,
                        window,
                        cx,
                    );
                });

                if focus {
                    let focus_handle = terminal_view.read(cx).focus_handle.clone();
                    focus_handle.focus(window, cx);
                }

                (active_pane, terminal, terminal_view)
            })
            .unwrap()
    }

    /// Creates a worktree with 1 file /root.txt and returns the project, workspace, and window handle.
    async fn init_test_with_window(
        cx: &mut TestAppContext,
    ) -> (
        Entity<Project>,
        Entity<Workspace>,
        gpui::WindowHandle<MultiWorkspace>,
    ) {
        let params = cx.update(AppState::test);
        cx.update(|cx| {
            theme_settings::init(theme::LoadThemes::JustBase, cx);
        });

        let project = Project::test(params.fs.clone(), [], cx).await;
        let window_handle =
            cx.add_window(|window, cx| MultiWorkspace::test_new(project.clone(), window, cx));
        let workspace = window_handle
            .read_with(cx, |mw, _| mw.workspace().clone())
            .unwrap();

        (project, workspace, window_handle)
    }

    async fn init_remote_test(
        cx: &mut TestAppContext,
        server_cx: &mut TestAppContext,
    ) -> (Entity<Project>, Entity<Workspace>) {
        cx.update(|cx| {
            release_channel::init(semver::Version::new(0, 0, 0), cx);
        });
        server_cx.update(|cx| {
            release_channel::init(semver::Version::new(0, 0, 0), cx);
        });

        let params = cx.update(AppState::test);
        let (opts, server_session, connect_guard) = RemoteClient::fake_server(cx, server_cx);
        let ping_handler = server_cx.new(|_| ());
        server_session.add_request_handler::<rpc::proto::Ping, _, _, _>(
            ping_handler.downgrade(),
            |_entity, _envelope, _cx| async { Ok(rpc::proto::Ack {}) },
        );
        drop(connect_guard);

        let remote_client = RemoteClient::connect_mock(opts, cx).await;
        let project = cx.update(|cx| {
            Project::remote(
                remote_client,
                params.client.clone(),
                params.node_runtime.clone(),
                params.user_store.clone(),
                params.languages.clone(),
                params.fs.clone(),
                false,
                cx,
            )
        });

        let window_handle = cx.add_window({
            let params = params.clone();
            let project_for_workspace = project.clone();
            move |window, cx| {
                window.activate_window();
                let workspace = cx.new(|cx| {
                    Workspace::new(
                        None,
                        project_for_workspace.clone(),
                        params.clone(),
                        window,
                        cx,
                    )
                });
                MultiWorkspace::new(workspace, window, cx)
            }
        });
        let workspace = window_handle
            .read_with(cx, |mw, _| mw.workspace().clone())
            .unwrap();

        (project, workspace)
    }

    /// Creates a file in the given worktree and returns its entry.
    async fn create_file_in_worktree(
        worktree: Entity<Worktree>,
        relative_path: impl AsRef<Path>,
        cx: &mut TestAppContext,
    ) -> Entry {
        cx.update(|cx| {
            worktree.update(cx, |worktree, cx| {
                worktree.create_entry(
                    RelPath::new(relative_path.as_ref(), PathStyle::local())
                        .unwrap()
                        .as_ref()
                        .into(),
                    false,
                    None,
                    cx,
                )
            })
        })
        .await
        .unwrap()
        .into_included()
        .unwrap()
    }

    /// Creates a worktree with 1 folder: /root{suffix}/
    async fn create_folder_wt(
        project: Entity<Project>,
        path: impl AsRef<Path>,
        cx: &mut TestAppContext,
    ) -> (Entity<Worktree>, Entry) {
        create_wt(project, true, path, cx).await
    }

    /// Creates a worktree with 1 file: /root{suffix}.txt
    async fn create_file_wt(
        project: Entity<Project>,
        path: impl AsRef<Path>,
        cx: &mut TestAppContext,
    ) -> (Entity<Worktree>, Entry) {
        create_wt(project, false, path, cx).await
    }

    async fn create_wt(
        project: Entity<Project>,
        is_dir: bool,
        path: impl AsRef<Path>,
        cx: &mut TestAppContext,
    ) -> (Entity<Worktree>, Entry) {
        let (wt, _) = project
            .update(cx, |project, cx| {
                project.find_or_create_worktree(path, true, cx)
            })
            .await
            .unwrap();

        let entry = cx
            .update(|cx| {
                wt.update(cx, |wt, cx| {
                    wt.create_entry(RelPath::empty_arc(), is_dir, None, cx)
                })
            })
            .await
            .unwrap()
            .into_included()
            .unwrap();

        (wt, entry)
    }

    pub fn insert_active_entry_for(
        wt: Entity<Worktree>,
        entry: Entry,
        project: Entity<Project>,
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| {
            let p = ProjectPath {
                worktree_id: wt.read(cx).id(),
                path: entry.path,
            };
            project.update(cx, |project, cx| project.set_active_path(Some(p), cx));
        });
    }

    // Terminal drag/drop test

    #[gpui::test]
    async fn test_handle_drop_writes_paths_for_all_drop_types(cx: &mut TestAppContext) {
        let (project, _workspace, window_handle) = init_test_with_window(cx).await;

        let (worktree, _) = create_folder_wt(project.clone(), "/root/", cx).await;
        let first_entry = create_file_in_worktree(worktree.clone(), "first.txt", cx).await;
        let second_entry = create_file_in_worktree(worktree.clone(), "second.txt", cx).await;

        let worktree_id = worktree.read_with(cx, |worktree, _| worktree.id());
        let first_path = project
            .read_with(cx, |project, cx| {
                project.absolute_path(
                    &ProjectPath {
                        worktree_id,
                        path: first_entry.path.clone(),
                    },
                    cx,
                )
            })
            .unwrap();
        let second_path = project
            .read_with(cx, |project, cx| {
                project.absolute_path(
                    &ProjectPath {
                        worktree_id,
                        path: second_entry.path.clone(),
                    },
                    cx,
                )
            })
            .unwrap();

        let (active_pane, terminal, terminal_view) =
            add_display_only_terminal(&project, window_handle, false, cx);

        let tab_item = window_handle
            .update(cx, |_, window, cx| {
                let tab_project_item = cx.new(|_| TestProjectItem {
                    entry_id: Some(second_entry.id),
                    project_path: Some(ProjectPath {
                        worktree_id,
                        path: second_entry.path.clone(),
                    }),
                    is_dirty: false,
                });
                let tab_item =
                    cx.new(|cx| TestItem::new(cx).with_project_items(&[tab_project_item]));
                active_pane.update(cx, |pane, cx| {
                    pane.add_item(Box::new(tab_item.clone()), true, false, None, window, cx);
                });
                tab_item
            })
            .unwrap();

        cx.run_until_parked();

        window_handle
            .update(cx, |multi_workspace, window, cx| {
                let workspace = multi_workspace.workspace().clone();
                let terminal_view_index =
                    active_pane.read(cx).index_for_item(&terminal_view).unwrap();
                let dragged_tab_index = active_pane.read(cx).index_for_item(&tab_item).unwrap();

                assert!(
                    workspace.read(cx).pane_for(&terminal_view).is_some(),
                    "terminal view not registered with workspace after run_until_parked"
                );

                // Dragging an external file should write its path to the terminal
                let external_paths = ExternalPaths(vec![first_path.clone()].into());
                assert_drop_writes_to_terminal(
                    &active_pane,
                    terminal_view_index,
                    &terminal,
                    &external_paths,
                    &expected_drop_text(std::slice::from_ref(&first_path)),
                    window,
                    cx,
                );

                // Dragging a tab should write the path of the tab's item to the terminal
                let dragged_tab = DraggedTab {
                    pane: active_pane.clone(),
                    item: Box::new(tab_item.clone()),
                    ix: dragged_tab_index,
                    detail: 0,
                    is_active: false,
                };
                assert_drop_writes_to_terminal(
                    &active_pane,
                    terminal_view_index,
                    &terminal,
                    &dragged_tab,
                    &expected_drop_text(std::slice::from_ref(&second_path)),
                    window,
                    cx,
                );

                // Dragging multiple selections should write both paths to the terminal
                let dragged_selection = DraggedSelection {
                    active_selection: SelectedEntry {
                        worktree_id,
                        entry_id: first_entry.id,
                    },
                    marked_selections: Arc::from([
                        SelectedEntry {
                            worktree_id,
                            entry_id: first_entry.id,
                        },
                        SelectedEntry {
                            worktree_id,
                            entry_id: second_entry.id,
                        },
                    ]),
                };
                assert_drop_writes_to_terminal(
                    &active_pane,
                    terminal_view_index,
                    &terminal,
                    &dragged_selection,
                    &expected_drop_text(&[first_path.clone(), second_path.clone()]),
                    window,
                    cx,
                );

                // Dropping a project entry should write the entry's path to the terminal
                let dropped_entry_id = first_entry.id;
                assert_drop_writes_to_terminal(
                    &active_pane,
                    terminal_view_index,
                    &terminal,
                    &dropped_entry_id,
                    &expected_drop_text(&[first_path]),
                    window,
                    cx,
                );
            })
            .unwrap();
    }

    // Terminal rename tests

    #[gpui::test]
    async fn test_custom_title_initially_none(cx: &mut TestAppContext) {
        cx.executor().allow_parking();

        let (project, workspace) = init_test(cx).await;

        let terminal = project
            .update(cx, |project, cx| project.create_terminal_shell(None, cx))
            .await
            .unwrap();

        let terminal_view = cx
            .add_window(|window, cx| {
                TerminalView::new(
                    terminal,
                    workspace.downgrade(),
                    None,
                    project.downgrade(),
                    window,
                    cx,
                )
            })
            .root(cx)
            .unwrap();

        terminal_view.update(cx, |view, _cx| {
            assert!(view.custom_title().is_none());
        });
    }

    #[gpui::test]
    async fn test_set_custom_title(cx: &mut TestAppContext) {
        cx.executor().allow_parking();

        let (project, workspace) = init_test(cx).await;

        let terminal = project
            .update(cx, |project, cx| project.create_terminal_shell(None, cx))
            .await
            .unwrap();

        let terminal_view = cx
            .add_window(|window, cx| {
                TerminalView::new(
                    terminal,
                    workspace.downgrade(),
                    None,
                    project.downgrade(),
                    window,
                    cx,
                )
            })
            .root(cx)
            .unwrap();

        terminal_view.update(cx, |view, cx| {
            view.set_custom_title(Some("frontend".to_string()), cx);
            assert_eq!(view.custom_title(), Some("frontend"));
        });
    }

    #[gpui::test]
    async fn test_set_custom_title_empty_becomes_none(cx: &mut TestAppContext) {
        cx.executor().allow_parking();

        let (project, workspace) = init_test(cx).await;

        let terminal = project
            .update(cx, |project, cx| project.create_terminal_shell(None, cx))
            .await
            .unwrap();

        let terminal_view = cx
            .add_window(|window, cx| {
                TerminalView::new(
                    terminal,
                    workspace.downgrade(),
                    None,
                    project.downgrade(),
                    window,
                    cx,
                )
            })
            .root(cx)
            .unwrap();

        terminal_view.update(cx, |view, cx| {
            view.set_custom_title(Some("test".to_string()), cx);
            assert_eq!(view.custom_title(), Some("test"));

            view.set_custom_title(Some("".to_string()), cx);
            assert!(view.custom_title().is_none());

            view.set_custom_title(Some("  ".to_string()), cx);
            assert!(view.custom_title().is_none());
        });
    }

    #[gpui::test]
    async fn test_custom_title_marks_needs_serialize(cx: &mut TestAppContext) {
        cx.executor().allow_parking();

        let (project, workspace) = init_test(cx).await;

        let terminal = project
            .update(cx, |project, cx| project.create_terminal_shell(None, cx))
            .await
            .unwrap();

        let terminal_view = cx
            .add_window(|window, cx| {
                TerminalView::new(
                    terminal,
                    workspace.downgrade(),
                    None,
                    project.downgrade(),
                    window,
                    cx,
                )
            })
            .root(cx)
            .unwrap();

        terminal_view.update(cx, |view, cx| {
            view.needs_serialize = false;
            view.set_custom_title(Some("new_label".to_string()), cx);
            assert!(view.needs_serialize);
        });
    }

    #[gpui::test]
    async fn test_tab_content_uses_custom_title(cx: &mut TestAppContext) {
        cx.executor().allow_parking();

        let (project, workspace) = init_test(cx).await;

        let terminal = project
            .update(cx, |project, cx| project.create_terminal_shell(None, cx))
            .await
            .unwrap();

        let terminal_view = cx
            .add_window(|window, cx| {
                TerminalView::new(
                    terminal,
                    workspace.downgrade(),
                    None,
                    project.downgrade(),
                    window,
                    cx,
                )
            })
            .root(cx)
            .unwrap();

        terminal_view.update(cx, |view, cx| {
            view.set_custom_title(Some("my-server".to_string()), cx);
            let text = view.tab_content_text(0, cx);
            assert_eq!(text.as_ref(), "my-server");
        });

        terminal_view.update(cx, |view, cx| {
            view.set_custom_title(None, cx);
            let text = view.tab_content_text(0, cx);
            assert_ne!(text.as_ref(), "my-server");
        });
    }

    async fn draw_standalone_terminal(
        output: &[u8],
        cx: &mut TestAppContext,
    ) -> (gpui::Bounds<Pixels>, gpui::Size<Pixels>) {
        let (project, workspace) = init_test(cx).await;
        let terminal = cx.new(|cx| {
            terminal::TerminalBuilder::new_display_only(
                CursorShape::default(),
                terminal::terminal_settings::AlternateScroll::On,
                None,
                0,
                cx.background_executor(),
                PathStyle::local(),
            )
            .subscribe(cx)
        });
        terminal.update(cx, |terminal, cx| {
            terminal.write_output(output, cx);
        });

        let (terminal_view, cx) = cx.add_window_view(|window, cx| {
            TerminalView::new(
                terminal.clone(),
                workspace.downgrade(),
                None,
                project.downgrade(),
                window,
                cx,
            )
        });

        let draw_size = gpui::size(px(400.), px(201.));
        cx.simulate_resize(draw_size);
        cx.draw(gpui::Point::default(), draw_size, |_, _| {
            terminal_view.clone().into_any_element()
        });
        cx.run_until_parked();
        cx.draw(gpui::Point::default(), draw_size, |_, _| {
            terminal_view.clone().into_any_element()
        });

        let bounds = terminal.read_with(cx, |terminal, _| {
            terminal.last_content().terminal_bounds.bounds
        });
        (bounds, draw_size)
    }

    // Marley: a short standalone terminal is drawn down onto the bottom edge, its one row in use
    // in view and the empty rows below it past the edge; Zed's own test asserted the top
    // (#476).
    #[gpui::test]
    async fn test_short_standalone_terminal_sits_on_the_bottom_edge_on_resize(
        cx: &mut TestAppContext,
    ) {
        let (bounds, draw_size) = draw_standalone_terminal(b"$ ", cx).await;
        assert!(bounds.origin.y > px(0.) && bounds.origin.y < draw_size.height);
        assert!(bounds.bottom() > draw_size.height);
    }

    #[gpui::test]
    async fn test_full_standalone_terminal_stays_bottom_anchored_on_resize(
        cx: &mut TestAppContext,
    ) {
        let (bounds, draw_size) = draw_standalone_terminal(
            b"one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\nten\n",
            cx,
        )
        .await;
        assert!(bounds.origin.y > px(0.));
        assert_eq!(bounds.bottom(), draw_size.height);
    }

    #[gpui::test]
    async fn test_short_alt_screen_stays_bottom_anchored_on_resize(cx: &mut TestAppContext) {
        let (bounds, draw_size) = draw_standalone_terminal(b"\x1b[?1049h$ ", cx).await;
        assert!(bounds.origin.y > px(0.));
        assert_eq!(bounds.bottom(), draw_size.height);
    }

    #[gpui::test]
    async fn test_inline_terminal_displays_all_of_its_lines(cx: &mut TestAppContext) {
        let (project, workspace) = init_test(cx).await;
        let terminal = cx.new(|cx| {
            terminal::TerminalBuilder::new_display_only(
                CursorShape::default(),
                terminal::terminal_settings::AlternateScroll::On,
                None,
                0,
                cx.background_executor(),
                PathStyle::local(),
            )
            .subscribe(cx)
        });
        let (terminal_view, cx) = cx.add_window_view(|window, cx| {
            let mut terminal_view = TerminalView::new(
                terminal.clone(),
                workspace.downgrade(),
                None,
                project.downgrade(),
                window,
                cx,
            );
            terminal_view.set_embedded_mode(None, cx);
            terminal_view
        });

        for _ in 1..=20 {
            terminal.update(cx, |terminal, cx| {
                terminal.write_output(b"line\n", cx);
            });
            cx.draw(
                gpui::Point::default(),
                gpui::size(px(400.), px(100.)),
                |_, _| terminal_view.clone().into_any_element(),
            );
            terminal.read_with(cx, |terminal, _| {
                assert_eq!(terminal.viewport_lines(), terminal.total_lines());
            })
        }
    }

    #[gpui::test]
    async fn test_inline_terminal_shrinks_after_clear(cx: &mut TestAppContext) {
        let (project, workspace) = init_test(cx).await;
        let terminal = cx.new(|cx| {
            terminal::TerminalBuilder::new_display_only(
                CursorShape::default(),
                terminal::terminal_settings::AlternateScroll::On,
                None,
                0,
                cx.background_executor(),
                PathStyle::local(),
            )
            .subscribe(cx)
        });
        let (terminal_view, cx) = cx.add_window_view(|window, cx| {
            let mut terminal_view = TerminalView::new(
                terminal.clone(),
                workspace.downgrade(),
                None,
                project.downgrade(),
                window,
                cx,
            );
            terminal_view.set_embedded_mode(None, cx);
            terminal_view
        });

        for _ in 1..=20 {
            terminal.update(cx, |terminal, cx| {
                terminal.write_output(b"line\n", cx);
            });
            cx.draw(
                gpui::Point::default(),
                gpui::size(px(400.), px(100.)),
                |_, _| terminal_view.clone().into_any_element(),
            );
        }
        terminal.read_with(cx, |terminal, _| {
            assert_eq!(terminal.total_lines(), 21);
        });

        terminal.update(cx, |terminal, _| terminal.clear());
        for _ in 1..=2 {
            cx.draw(
                gpui::Point::default(),
                gpui::size(px(400.), px(100.)),
                |_, _| terminal_view.clone().into_any_element(),
            );
        }
        terminal.read_with(cx, |terminal, _| {
            assert_eq!(terminal.total_lines(), 1);
            assert_eq!(terminal.viewport_lines(), 1);
        });
    }

    #[gpui::test]
    async fn test_tab_content_shows_terminal_title_when_custom_title_directly_set_empty(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();

        let (project, workspace) = init_test(cx).await;

        let terminal = project
            .update(cx, |project, cx| project.create_terminal_shell(None, cx))
            .await
            .unwrap();

        let terminal_view = cx
            .add_window(|window, cx| {
                TerminalView::new(
                    terminal,
                    workspace.downgrade(),
                    None,
                    project.downgrade(),
                    window,
                    cx,
                )
            })
            .root(cx)
            .unwrap();

        terminal_view.update(cx, |view, cx| {
            view.custom_title = Some("".to_string());
            let text = view.tab_content_text(0, cx);
            assert!(
                !text.is_empty(),
                "Tab should show terminal title, not empty string; got: '{}'",
                text
            );
        });

        terminal_view.update(cx, |view, cx| {
            view.custom_title = Some("   ".to_string());
            let text = view.tab_content_text(0, cx);
            assert!(
                !text.is_empty() && text.as_ref() != "   ",
                "Tab should show terminal title, not whitespace; got: '{}'",
                text
            );
        });
    }

    // Marley: a terminal view over a real PTY whose script prints Marley's shell-hook frames,
    // shown in a test window (#470). The script then sleeps: a child that exits at once can
    // leave its last bytes unread under load, as the event loop drains only once at exit.
    #[cfg(unix)]
    async fn marley_hook_terminal(
        script: &str,
        cx: &mut TestAppContext,
    ) -> (Entity<Terminal>, &'static mut VisualTestContext) {
        let (project, _, window_handle) = init_test_with_window(cx).await;
        let program = "/bin/sh".to_string();
        let args = vec!["-c".to_string(), format!("{script}; sleep 60")];
        let builder = cx
            .update(|cx| {
                terminal::TerminalBuilder::new(
                    None,
                    terminal::TerminalMode::task(task::SpawnInTerminal {
                        command: Some(program.clone()),
                        args: args.clone(),
                        ..Default::default()
                    }),
                    task::Shell::WithArguments {
                        program,
                        args,
                        title_override: None,
                    },
                    Default::default(),
                    Default::default(),
                    terminal::terminal_settings::AlternateScroll::On,
                    None,
                    vec![],
                    std::time::Duration::ZERO,
                    false,
                    0,
                    cx,
                    vec![],
                    PathStyle::local(),
                )
            })
            .await
            .unwrap();
        let terminal = cx.new(|cx| builder.subscribe(cx));
        window_handle
            .update(cx, |multi_workspace, window, cx| {
                let workspace = multi_workspace.workspace().clone();
                let pane = workspace.read(cx).active_pane().clone();
                let view = cx.new(|cx| {
                    TerminalView::new(
                        terminal.clone(),
                        workspace.downgrade(),
                        None,
                        project.downgrade(),
                        window,
                        cx,
                    )
                });
                pane.update(cx, |pane, cx| {
                    pane.add_item(Box::new(view), true, true, None, window, cx);
                });
            })
            .unwrap();
        let cx = VisualTestContext::from_window(window_handle.into(), cx).into_mut();
        (terminal, cx)
    }

    // Marley: the frames of a command that succeeds and of one that fails, one row apart (#470).
    // Each command's and each prompt's frame carries `$nonce`, which a script sets to the
    // terminal's own to have the commands verified (#474) and the prompt known as the local
    // shell's, where a Rerun is offered (#526). The script runs as a task, whose own block comes
    // first and ends at the first prompt (#621), so the two commands are blocks 1 and 2.
    #[cfg(unix)]
    const MARLEY_TWO_BLOCKS: &str = r#"printf '\033Ppinit;id=1\033\\\033Ppprecmd;exit=0;nonce=%s\033\\$ true\r\n\033Pppreexec;command=true;nonce=%s\033\\\033Ppprecmd;exit=0;nonce=%s\033\\$ false\r\n\033Pppreexec;command=false;nonce=%s\033\\oops\r\n\033Ppprecmd;exit=1;nonce=%s\033\\$ ' "$nonce" "$nonce" "$nonce" "$nonce" "$nonce""#;

    // Marley: draws frames until the terminal holds `count` finished blocks (#470).
    #[cfg(unix)]
    async fn marley_draw_until_finished(
        terminal: &Entity<Terminal>,
        count: usize,
        cx: &mut VisualTestContext,
    ) {
        for _ in 0..300 {
            cx.update(|window, _| window.refresh());
            cx.run_until_parked();
            let finished = terminal.read_with(cx, |terminal, _| {
                terminal
                    .blocks()
                    .iter()
                    .filter(|block| block.state == marley_terminal::BlockState::Finished)
                    .count()
            });
            if finished >= count {
                return;
            }
            cx.background_executor
                .timer(std::time::Duration::from_millis(10))
                .await;
        }
        panic!("the terminal kept fewer than {count} finished blocks");
    }

    // Marley: each block whose first row is on screen draws its status pill at the right end
    // of that row (#470).
    #[cfg(unix)]
    #[gpui::test]
    async fn marley_blocks_draw_their_pills_on_their_first_rows(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        // Output first, so the blocks sit below lines that have left the screen for the
        // scrollback, and their rows depend on it.
        let script = format!("seq 1 200; {MARLEY_TWO_BLOCKS}");
        let (terminal, cx) = marley_hook_terminal(&script, cx).await;
        marley_draw_until_finished(&terminal, 3, cx).await;
        cx.update(|window, _| window.refresh());
        let history = terminal.read_with(cx, |terminal, _| {
            let content = terminal.last_content();
            content.total_lines - content.screen_lines
        });
        assert!(history > 0, "the output scrolled");
        let first = cx
            .debug_bounds("marley-block-pill-1")
            .expect("the passing block's pill");
        let second = cx
            .debug_bounds("marley-block-pill-2")
            .expect("the failing block's pill");
        let grid = terminal.read_with(cx, |terminal, _| terminal.last_content().terminal_bounds);
        // Each is centered in its block's first row, and the blocks start one row apart; the
        // layout snaps each pill to a whole pixel.
        let apart = second.center().y - first.center().y;
        assert!(
            (apart - grid.line_height()).abs() < px(1.),
            "{apart:?} apart, rows of {:?}",
            grid.line_height()
        );
        // Both end at the grid's right end, less the row's padding.
        assert_eq!(first.right(), second.right());
        let right = grid.bounds.origin.x + grid.width();
        assert!(first.right() <= right && first.right() > right - px(8.));
    }

    // Marley: on the alternate screen no block is drawn (#470).
    #[cfg(unix)]
    #[gpui::test]
    async fn marley_the_alternate_screen_draws_no_block(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let script = format!(r"{MARLEY_TWO_BLOCKS}; printf '\033[?1049h'");
        let (terminal, cx) = marley_hook_terminal(&script, cx).await;
        marley_draw_until_finished(&terminal, 3, cx).await;
        for _ in 0..300 {
            let alternate = terminal.read_with(cx, |terminal, _| {
                terminal
                    .last_content()
                    .mode
                    .contains(terminal::Modes::ALT_SCREEN)
            });
            if alternate {
                break;
            }
            cx.background_executor
                .timer(std::time::Duration::from_millis(10))
                .await;
            cx.update(|window, _| window.refresh());
            cx.run_until_parked();
        }
        cx.update(|window, _| window.refresh());
        assert!(cx.debug_bounds("marley-block-pill-1").is_none());
        assert!(cx.debug_bounds("marley-block-pill-2").is_none());
    }

    // Marley: points at `selector`'s middle, draws a frame, and returns its bounds then (#474).
    #[cfg(unix)]
    fn marley_point_at(
        selector: &'static str,
        cx: &mut VisualTestContext,
    ) -> gpui::Bounds<gpui::Pixels> {
        cx.update(|window, _| window.refresh());
        let bounds = cx.debug_bounds(selector).expect(selector);
        cx.simulate_mouse_move(bounds.center(), None, gpui::Modifiers::none());
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        bounds
    }

    // Marley: the buttons of the block under the pointer copy its output and rerun its command.
    // Mouse reporting is on, so whatever the terminal takes reaches the program as a report: a
    // press on a button stays off it, and a press on the block's output and a release over a
    // button reach it (#474).
    #[cfg(unix)]
    #[gpui::test]
    async fn marley_a_hovered_blocks_buttons_copy_and_rerun_it(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let script =
            format!(r"nonce=$MARLEY_SHELL_NONCE; {MARLEY_TWO_BLOCKS}; printf '\033[?1000h'");
        let (terminal, cx) = marley_hook_terminal(&script, cx).await;
        marley_draw_until_finished(&terminal, 3, cx).await;
        let mut reporting = false;
        for _ in 0..300 {
            reporting = terminal.read_with(cx, |terminal, _| {
                terminal
                    .last_content()
                    .mode
                    .contains(terminal::Modes::MOUSE_REPORT_CLICK)
            });
            if reporting {
                break;
            }
            cx.background_executor
                .timer(std::time::Duration::from_millis(10))
                .await;
            cx.update(|window, _| window.refresh());
            cx.run_until_parked();
        }
        assert!(reporting, "the program turned mouse reporting on");

        // Over the passing block, its buttons show and the failed block's do not.
        marley_point_at("marley-block-pill-1", cx);
        assert!(cx.debug_bounds("marley-block-copy-1").is_some());
        assert!(cx.debug_bounds("marley-block-copy-2").is_none());

        // A press on the failed block's output, released over Copy: the program gets both, and
        // nothing is copied.
        let pill = marley_point_at("marley-block-pill-2", cx);
        let copy = cx
            .debug_bounds("marley-block-copy-2")
            .expect("Copy on the failed block");
        let grid = terminal.read_with(cx, |terminal, _| terminal.last_content().terminal_bounds);
        let output = gpui::point(
            grid.bounds.origin.x + grid.cell_width() * 0.5,
            pill.center().y + grid.line_height(),
        );
        terminal.update(cx, |terminal, _| terminal.take_pty_write_log());
        cx.simulate_event(gpui::MouseDownEvent {
            button: gpui::MouseButton::Left,
            position: output,
            modifiers: gpui::Modifiers::none(),
            click_count: 1,
            first_mouse: false,
        });
        cx.simulate_event(gpui::MouseUpEvent {
            button: gpui::MouseButton::Left,
            position: copy.center(),
            modifiers: gpui::Modifiers::none(),
            click_count: 1,
        });
        let reports = terminal.update(cx, |terminal, _| terminal.take_pty_write_log());
        // A report is `ESC [ M`, then 32 plus the button (0 pressed, 3 released), then the cell.
        let buttons: Vec<Option<u8>> = reports
            .iter()
            .map(|report| report.strip_prefix(b"\x1b[M")?.first().copied())
            .collect();
        assert_eq!(buttons, [Some(b' '), Some(b'#')], "{reports:?}");
        assert!(cx.read_from_clipboard().is_none(), "nothing copied");

        let copy = marley_point_at("marley-block-copy-2", cx);
        cx.simulate_click(copy.center(), gpui::Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            cx.read_from_clipboard().and_then(|item| item.text()),
            Some("oops".to_string())
        );
        assert_eq!(
            terminal.update(cx, |terminal, _| terminal.take_pty_write_log()),
            Vec::<Vec<u8>>::new(),
            "no mouse report"
        );

        let rerun = marley_point_at("marley-block-rerun-2", cx);
        cx.simulate_click(rerun.center(), gpui::Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            terminal.update(cx, |terminal, _| terminal.take_pty_write_log()),
            [b"\x15false\r".to_vec()],
            "the rerun alone, no mouse report"
        );
    }

    // Marley: while a block runs there is no prompt to rerun at (#474).
    #[cfg(unix)]
    #[gpui::test]
    async fn marley_no_rerun_while_a_block_runs(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let script = format!(
            r"nonce=$MARLEY_SHELL_NONCE; {MARLEY_TWO_BLOCKS}; printf '\033Pppreexec;command=sleep 60\033\\'"
        );
        let (terminal, cx) = marley_hook_terminal(&script, cx).await;
        marley_draw_until_finished(&terminal, 3, cx).await;
        for _ in 0..300 {
            if terminal.read_with(cx, |terminal, _| terminal.blocks().len()) == 4 {
                break;
            }
            cx.background_executor
                .timer(std::time::Duration::from_millis(10))
                .await;
        }
        marley_point_at("marley-block-pill-2", cx);
        assert!(cx.debug_bounds("marley-block-copy-2").is_some());
        assert!(cx.debug_bounds("marley-block-rerun-2").is_none());
    }

    // Marley: waits until the terminal's modes hold `mode`, drawing frames (#476).
    #[cfg(unix)]
    async fn marley_draw_until_mode(
        terminal: &Entity<Terminal>,
        mode: terminal::Modes,
        cx: &mut VisualTestContext,
    ) {
        for _ in 0..300 {
            cx.update(|window, _| window.refresh());
            cx.run_until_parked();
            if terminal.read_with(cx, |terminal, _| {
                terminal.last_content().mode.contains(mode)
            }) {
                return;
            }
            cx.background_executor
                .timer(std::time::Duration::from_millis(10))
                .await;
        }
        panic!("the terminal never held {mode:?}");
    }

    // Marley: the rows between the middle of `bounds` and the bottom edge of the view (#476).
    #[cfg(unix)]
    fn marley_rows_above_the_bottom(
        bounds: gpui::Bounds<gpui::Pixels>,
        terminal: &Entity<Terminal>,
        cx: &mut VisualTestContext,
    ) -> f32 {
        let view = cx
            .debug_bounds("marley-terminal-view")
            .expect("the view's container");
        let line_height = terminal.read_with(cx, |terminal, _| {
            terminal.last_content().terminal_bounds.line_height()
        });
        (view.bottom() - bounds.center().y) / line_height
    }

    // Marley: with the screen mostly empty, the content sits on the bottom edge: the prompt on
    // the grid's last row, the failed block's command two rows above it, its pill with it
    // (#476).
    #[cfg(unix)]
    #[gpui::test]
    async fn marley_short_content_sits_on_the_bottom_edge(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let (terminal, cx) = marley_hook_terminal(MARLEY_TWO_BLOCKS, cx).await;
        marley_draw_until_finished(&terminal, 3, cx).await;
        cx.update(|window, _| window.refresh());
        let pill = cx
            .debug_bounds("marley-block-pill-2")
            .expect("the failed block's pill");
        // The prompt is on the grid's last row, which Zed's own layout can leave less than a row
        // above the edge: it snaps the rows' height to whole device pixels.
        let rows = marley_rows_above_the_bottom(pill, &terminal, cx);
        assert!((2.5..3.5).contains(&rows), "the pill is {rows} rows up");
    }

    // Marley: the cursor's row counts as content, so a line ending in a newline keeps the
    // empty row the next text goes on in view (#476).
    #[cfg(unix)]
    #[gpui::test]
    async fn marley_the_cursors_row_counts_as_content(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let (terminal, cx) = marley_hook_terminal(r"printf 'x\r\n'", cx).await;
        for _ in 0..300 {
            cx.update(|window, _| window.refresh());
            cx.run_until_parked();
            if terminal
                .update(cx, |terminal, _| terminal.get_content())
                .starts_with('x')
            {
                break;
            }
            cx.background_executor
                .timer(std::time::Duration::from_millis(10))
                .await;
        }
        cx.update(|window, _| window.refresh());
        let (empty, lines) = terminal.read_with(cx, |terminal, _| {
            let content = terminal.last_content();
            (content.marley_empty_bottom_rows, content.screen_lines)
        });
        // `x` on the first row, the cursor on the second, and every row below them empty.
        assert_eq!(empty, lines - 2);
    }

    // Marley: scrolled back, the content stays where it was and the history shows above it
    // (#476).
    #[cfg(unix)]
    #[gpui::test]
    async fn marley_scrolled_back_the_history_shows_above_the_content(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        // A cleared screen over 200 lines of history, with the two blocks at its top.
        let script = format!(r"seq 1 200; printf '\033[H\033[2J'; {MARLEY_TWO_BLOCKS}");
        let (terminal, cx) = marley_hook_terminal(&script, cx).await;
        marley_draw_until_finished(&terminal, 3, cx).await;
        cx.update(|window, _| window.refresh());
        let pill = cx
            .debug_bounds("marley-block-pill-2")
            .expect("the failed block's pill");
        let before = marley_rows_above_the_bottom(pill, &terminal, cx);
        terminal.update(cx, |terminal, _| terminal.scroll_up_by(1));
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        assert_eq!(
            terminal.read_with(cx, |terminal, _| terminal.last_content().display_offset),
            1
        );
        let pill = cx
            .debug_bounds("marley-block-pill-2")
            .expect("the pill, still in view");
        let after = marley_rows_above_the_bottom(pill, &terminal, cx);
        assert!(
            (after - before).abs() < 0.1,
            "from {before} to {after} rows up"
        );
    }

    // Marley: the alternate screen is drawn from the top, as before (#476).
    #[cfg(unix)]
    #[gpui::test]
    async fn marley_the_alternate_screen_is_drawn_from_the_top(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let (terminal, cx) = marley_hook_terminal(r"printf '\033[?1049hx'", cx).await;
        marley_draw_until_mode(&terminal, terminal::Modes::ALT_SCREEN, cx).await;
        let view = cx
            .debug_bounds("marley-terminal-view")
            .expect("the view's container");
        let grid = terminal.read_with(cx, |terminal, _| terminal.last_content().terminal_bounds);
        // Only the padding the snapped rows leave, less than a row, lies above the grid.
        let above = grid.bounds.origin.y - view.top();
        assert!(
            above >= px(0.) && above < grid.line_height(),
            "{above:?} above the grid"
        );
    }

    // Marley: a click on content drawn down onto the bottom edge acts on the row it was drawn
    // on: with mouse reporting on, it is reported at that row's grid line (#476).
    #[cfg(unix)]
    #[gpui::test]
    async fn marley_a_click_on_shifted_content_reports_its_own_row(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let script = format!(r"{MARLEY_TWO_BLOCKS}; printf '\033[?1000h'");
        let (terminal, cx) = marley_hook_terminal(&script, cx).await;
        marley_draw_until_finished(&terminal, 3, cx).await;
        marley_draw_until_mode(&terminal, terminal::Modes::MOUSE_REPORT_CLICK, cx).await;
        let pill = cx
            .debug_bounds("marley-block-pill-2")
            .expect("the failed block's pill");
        let grid = terminal.read_with(cx, |terminal, _| terminal.last_content().terminal_bounds);
        // `oops`, on grid line 2, is drawn on the row below the failed block's pill.
        let oops = gpui::point(
            grid.bounds.origin.x + grid.cell_width() * 0.5,
            pill.center().y + grid.line_height(),
        );
        terminal.update(cx, |terminal, _| terminal.take_pty_write_log());
        cx.simulate_click(oops, gpui::Modifiers::none());
        cx.run_until_parked();
        let reports = terminal.update(cx, |terminal, _| terminal.take_pty_write_log());
        // `ESC [ M`, the button, then the column and the line, each 33 more than its number.
        assert_eq!(
            reports.first().map(|press| press.as_slice()),
            Some(b"\x1b[M !#".as_slice()),
            "{reports:?}"
        );
    }

    // Marley: a command whose frame did not carry the terminal's nonce, as a frame that output
    // prints cannot, is offered no Rerun (#474).
    #[cfg(unix)]
    #[gpui::test]
    async fn marley_no_rerun_for_a_command_that_output_printed(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let script = format!("nonce=forged; {MARLEY_TWO_BLOCKS}");
        let (terminal, cx) = marley_hook_terminal(&script, cx).await;
        marley_draw_until_finished(&terminal, 3, cx).await;
        marley_point_at("marley-block-pill-2", cx);
        assert!(cx.debug_bounds("marley-block-copy-2").is_some());
        assert!(cx.debug_bounds("marley-block-rerun-2").is_none());
    }
}
