//! A block's output filtered in a panel over its terminal (#528), Warp's block filter.
//!
//! Alt+Shift+F filters the selected block (#554), else the newest block in view; the Filter button
//! on a hovered block filters that block. The panel lists the lines that match text or a regex,
//! with case, invert and context lines (`marley_terminal::filter`), in a read-only editor with the
//! matches highlighted. The terminal's own rows are never touched, so nothing is deleted. Each
//! terminal keeps its panel, query and toggles for the session, and a running block's list follows
//! its output, filtered again at most four times a second off the main thread.

use std::sync::Arc;
use std::time::Duration;

use collections::HashMap;
use editor::{Editor, EditorEvent, HighlightKey};
use gpui::{
    App, AppContext as _, Context, Entity, EntityId, FocusHandle, Focusable, Global, KeyDownEvent,
    Subscription, Task, WeakEntity, Window,
};
use language::Point;
use marley_terminal::filter::{FilterQuery, Filtered, FilteredRow, filter_lines};
use terminal::Terminal;
use terminal_view::{
    MarleyBlockFilter, MarleyBlockSelection, MarleyFooterContext, MarleyTerminalOverlay,
    TerminalView,
};
use ui::{Button, IconButton, IconName, Label, LabelSize, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::Workspace;

use crate::{CloseBlockFilter, FilterBlock};

/// The least time between two filters of a running block's output.
const FOLLOW_EVERY: Duration = Duration::from_millis(250);

/// Each terminal view's filter panel, made the first time it opens and dropped with the view.
#[derive(Default)]
struct Filters(HashMap<EntityId, Entity<FilterPanel>>);

impl Global for Filters {}

/// Installs the key, the hooks Zed's terminal view asks, and the panels' cleanup. [`crate::init`]
/// calls it once, before any window opens.
pub fn init(cx: &mut App) {
    cx.set_global(Filters::default());
    cx.set_global(MarleyTerminalOverlay(Arc::new(overlay)));
    cx.set_global(MarleyBlockFilter(Arc::new(open)));
    cx.observe_new(|_: &mut TerminalView, _, cx: &mut Context<TerminalView>| {
        let view = cx.entity_id();
        cx.on_release(move |_, cx| {
            cx.default_global::<Filters>().0.remove(&view);
        })
        .detach();
    })
    .detach();
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action_renderer(|div, _, _, cx| {
            div.on_action(cx.listener(|workspace, _: &FilterBlock, window, cx| {
                let Some(view) = crate::blocks::focused_terminal(workspace, window, cx) else {
                    cx.propagate();
                    return;
                };
                let Some(block) = block_to_filter(&view, cx) else {
                    cx.propagate();
                    return;
                };
                open(&view, block, window, cx);
            }))
        });
    })
    .detach();
}

/// The block the key filters: the selected one, else the newest with a row in view; none on the
/// alternate screen or with no block in view.
pub(crate) fn block_to_filter(view: &Entity<TerminalView>, cx: &App) -> Option<usize> {
    let terminal = view.read(cx).terminal();
    if let Some(selected) = MarleyBlockSelection::selected(terminal, cx) {
        return Some(selected);
    }
    let terminal = terminal.read(cx);
    let content = terminal.last_content();
    if content.mode.contains(terminal::Modes::ALT_SCREEN) {
        return None;
    }
    let display_offset = u64::try_from(content.display_offset).unwrap_or(u64::MAX);
    let top = content.marley_screen_top.saturating_sub(display_offset);
    let cursor_line =
        content.marley_screen_top + u64::try_from(content.cursor.point.line).unwrap_or_default();
    marley_terminal::visible_spans(terminal.blocks(), top, content.screen_lines, cursor_line)
        .last()
        .map(|span| span.index)
}

/// Opens the filter of `view` on its block at `block`, with the last query and toggles, and gives
/// the query field the focus.
fn open(view: &Entity<TerminalView>, block: usize, window: &mut Window, cx: &mut App) {
    let key = view.entity_id();
    let existing = cx
        .try_global::<Filters>()
        .and_then(|filters| filters.0.get(&key))
        .cloned();
    let panel = existing.unwrap_or_else(|| {
        let terminal = view.read(cx).terminal().clone();
        let panel = cx.new(|cx| FilterPanel::new(view.downgrade(), &terminal, window, cx));
        cx.default_global::<Filters>().0.insert(key, panel.clone());
        panel
    });
    panel.update(cx, |panel, cx| {
        panel.block = block;
        panel.open = true;
        panel.refilter(cx);
    });
    let query = panel.read(cx).query.focus_handle(cx);
    window.focus(&query, cx);
    view.update(cx, |_, cx| cx.notify());
}

/// The hook Zed's terminal view asks in its render: a remote terminal's down line first (#641),
/// else the open panel of the view in `context`. It reads the view's id only, since the view is
/// being rendered.
fn overlay(context: &MarleyFooterContext, _: &mut Window, cx: &mut App) -> Option<AnyElement> {
    if let Some(down) = crate::remote::link_overlay(context, cx) {
        return Some(down);
    }
    let key = context.view.entity_id();
    let panel = cx.try_global::<Filters>()?.0.get(&key)?.clone();
    panel.read(cx).open.then(|| panel.into_any_element())
}

/// The filter's three toggles.
#[derive(Clone, Copy, Default)]
struct Toggles {
    regex: bool,
    case_sensitive: bool,
    invert: bool,
}

/// A terminal's filter: the block it filters, the fields, the toggles and the last list.
struct FilterPanel {
    view: WeakEntity<TerminalView>,
    terminal: WeakEntity<Terminal>,
    block: usize,
    open: bool,
    query: Entity<Editor>,
    context: Entity<Editor>,
    list: Entity<Editor>,
    toggles: Toggles,
    command: String,
    picked: usize,
    total: usize,
    error: Option<String>,
    filtering: Task<()>,
    following: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl FilterPanel {
    fn new(
        view: WeakEntity<TerminalView>,
        terminal: &Entity<Terminal>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let query = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Filter the block's lines", window, cx);
            editor
        });
        let context = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("0", window, cx);
            editor
        });
        let list = cx.new(|cx| {
            let mut editor = Editor::multi_line(window, cx);
            editor.set_read_only(true);
            editor.set_show_gutter(false, cx);
            editor
        });
        let edited =
            |this: &mut Self, _: Entity<Editor>, event: &EditorEvent, cx: &mut Context<Self>| {
                if matches!(event, EditorEvent::BufferEdited) {
                    this.refilter(cx);
                }
            };
        let subscriptions = vec![
            cx.subscribe(&query, edited),
            cx.subscribe(&context, edited),
            cx.subscribe(terminal, |this, _, event: &terminal::Event, cx| {
                if matches!(event, terminal::Event::Wakeup) && this.open {
                    this.follow(cx);
                }
            }),
        ];
        Self {
            view,
            terminal: terminal.downgrade(),
            block: 0,
            open: false,
            query,
            context,
            list,
            toggles: Toggles::default(),
            command: String::new(),
            picked: 0,
            total: 0,
            error: None,
            filtering: Task::ready(()),
            following: None,
            _subscriptions: subscriptions,
        }
    }

    /// The query the fields and toggles make; a context that is not a number counts as none.
    fn filter_query(&self, cx: &App) -> FilterQuery {
        FilterQuery {
            text: self.query.read(cx).text(cx),
            regex: self.toggles.regex,
            case_sensitive: self.toggles.case_sensitive,
            invert: self.toggles.invert,
            context: self.context.read(cx).text(cx).trim().parse().unwrap_or(0),
        }
    }

    /// Filters the block again, off the main thread, and shows what it keeps.
    fn refilter(&mut self, cx: &Context<Self>) {
        let Some(terminal) = self.terminal.upgrade() else {
            return;
        };
        let (command, output) = {
            let terminal = terminal.read(cx);
            terminal
                .blocks()
                .get(self.block)
                .map_or((String::new(), None), |block| {
                    (block.command.clone(), terminal.block_output(block))
                })
        };
        self.command = command;
        let query = self.filter_query(cx);
        let filtering = cx.background_spawn(futures::future::lazy(move |_| {
            output.map(|output| filter_lines(&output, &query))
        }));
        self.filtering = cx.spawn(async move |this, cx| {
            let result = filtering.await;
            this.update(cx, |this, cx| this.show(result, cx)).log_err();
        });
    }

    /// Filters a running block again once [`FOLLOW_EVERY`] has passed since the last time, so its
    /// list takes in the new output without a filter for every frame.
    fn follow(&mut self, cx: &Context<Self>) {
        if self.following.is_some() {
            return;
        }
        let running = self.terminal.upgrade().is_some_and(|terminal| {
            terminal
                .read(cx)
                .blocks()
                .get(self.block)
                .is_some_and(|block| block.state != marley_terminal::BlockState::Finished)
        });
        if !running {
            return;
        }
        self.following = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(FOLLOW_EVERY).await;
            this.update(cx, |this, cx| {
                this.following = None;
                this.refilter(cx);
            })
            .log_err();
        }));
    }

    /// Shows a filter's outcome: the kept lines with their matches highlighted, or why there are
    /// none; an invalid query keeps the last list.
    fn show(
        &mut self,
        result: Option<Result<Filtered, marley_terminal::filter::FilterError>>,
        cx: &mut Context<Self>,
    ) {
        match result {
            None => {
                self.error = Some("The block's output is no longer in the scrollback.".to_string());
            }
            Some(Err(error)) => {
                self.error = Some(format!("Not a valid regular expression: {error}"));
            }
            Some(Ok(filtered)) => {
                self.error = None;
                self.picked = filtered.picked;
                self.total = filtered.total;
                self.fill_list(&filtered, cx);
            }
        }
        cx.notify();
    }

    /// Puts `filtered`'s rows in the list, `--` for a gap, and highlights the matches.
    fn fill_list(&self, filtered: &Filtered, cx: &mut Context<Self>) {
        let mut text = String::new();
        let mut matches: Vec<(u32, std::ops::Range<usize>)> = Vec::new();
        for (row, line) in filtered.rows.iter().enumerate() {
            if row > 0 {
                text.push('\n');
            }
            match line {
                FilteredRow::Gap => text.push_str("--"),
                FilteredRow::Line {
                    text: line,
                    matches: found,
                    ..
                } => {
                    text.push_str(line);
                    let row = u32::try_from(row).unwrap_or(u32::MAX);
                    matches.extend(found.iter().map(|range| (row, range.clone())));
                }
            }
        }
        let Some(buffer) = self.list.read(cx).buffer().read(cx).as_singleton() else {
            return;
        };
        buffer.update(cx, |buffer, cx| buffer.set_text(text, cx));
        self.list.update(cx, |editor, cx| {
            let snapshot = editor.buffer().read(cx).snapshot(cx);
            let point = |row: u32, column: usize| {
                Point::new(row, u32::try_from(column).unwrap_or(u32::MAX))
            };
            let ranges: Vec<_> = matches
                .iter()
                .map(|(row, range)| {
                    snapshot.anchor_before(point(*row, range.start))
                        ..snapshot.anchor_after(point(*row, range.end))
                })
                .collect();
            editor.highlight_background(
                HighlightKey::BufferSearchHighlights,
                &ranges,
                |_, theme| theme.colors().search_match_background,
                cx,
            );
        });
    }

    /// Closes the panel and gives the terminal the focus back.
    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = false;
        if let Some(view) = self.view.upgrade() {
            window.focus(&view.focus_handle(cx), cx);
            view.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
    }

    fn toggle(&mut self, flip: fn(&mut Self), cx: &Context<Self>) {
        flip(self);
        self.refilter(cx);
    }
}

impl Focusable for FilterPanel {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.query.focus_handle(cx)
    }
}

impl FilterPanel {
    /// The panel's first row: the block's command, the count, and Close.
    fn header(&self, cx: &Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .child(
                Label::new(format!("Filter · {}", self.command))
                    .size(LabelSize::Small)
                    .truncate(),
            )
            .child(div().flex_1())
            .child(
                Label::new(format!("{} of {} lines", self.picked, self.total))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                IconButton::new("marley-filter-close", IconName::Close)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text("Close the Filter"))
                    .on_click(cx.listener(|this, _, window, cx| this.close(window, cx))),
            )
    }

    /// The query field, the case, regex and invert toggles, and the context field.
    fn controls(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let field = |editor: Entity<Editor>| {
            div()
                .px_1p5()
                .py_1()
                .border_1()
                .border_color(colors.border)
                .rounded_sm()
                .bg(colors.editor_background)
                .child(editor)
        };
        let toggle = |id: &'static str, icon: IconName, on: bool, tip: &'static str| {
            IconButton::new(id, icon)
                .icon_size(IconSize::Small)
                .toggle_state(on)
                .tooltip(Tooltip::text(tip))
        };
        h_flex()
            .gap_1()
            .child(field(self.query.clone()).flex_1())
            .child(
                toggle(
                    "marley-filter-case",
                    IconName::CaseSensitive,
                    self.toggles.case_sensitive,
                    "Match Case",
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.toggle(
                        |this| this.toggles.case_sensitive = !this.toggles.case_sensitive,
                        cx,
                    );
                })),
            )
            .child(
                toggle(
                    "marley-filter-regex",
                    IconName::Regex,
                    self.toggles.regex,
                    "Regular Expression",
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.toggle(|this| this.toggles.regex = !this.toggles.regex, cx);
                })),
            )
            .child(
                Button::new("marley-filter-invert", "Invert")
                    .label_size(LabelSize::Small)
                    .toggle_state(self.toggles.invert)
                    .tooltip(Tooltip::text("Keep the Lines That Do Not Match"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.toggle(|this| this.toggles.invert = !this.toggles.invert, cx);
                    })),
            )
            .child(
                Label::new("Context")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(field(self.context.clone()).w(px(48.)))
    }
}

impl Render for FilterPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        v_flex()
            .debug_selector(|| "marley-block-filter".into())
            .key_context("MarleyBlockFilter")
            .on_action(cx.listener(|this, _: &CloseBlockFilter, window, cx| this.close(window, cx)))
            .on_action(cx.listener(|this, _: &FilterBlock, window, cx| this.close(window, cx)))
            // The terminal view sends its program the keys it maps, as for the rich input: stop
            // those, and let a key that types text go on to the focused field.
            .on_key_down(|event: &KeyDownEvent, _, cx| {
                let modifiers = &event.keystroke.modifiers;
                let types_text = event.keystroke.key_char.is_some()
                    && !(modifiers.control || modifiers.alt || modifiers.platform);
                if !types_text {
                    cx.stop_propagation();
                }
            })
            .occlude()
            .absolute()
            .inset_0()
            .p_2()
            .gap_2()
            .bg(colors.panel_background)
            .border_1()
            .border_color(colors.border_variant)
            .child(self.header(cx))
            .child(self.controls(cx))
            .children(
                self.error
                    .clone()
                    .map(|error| Label::new(error).size(LabelSize::Small).color(Color::Error)),
            )
            .child(div().flex_1().min_h_0().child(self.list.clone()))
    }
}
