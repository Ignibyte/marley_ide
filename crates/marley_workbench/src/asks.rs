//! The agents' questions as the Dialogs and `MoreForms` boards draw them (#871).
//!
//! A settings or keymap change, a harness seat, a sensitive tool such as `agent_start`, and a
//! pairing each ask the user through a notification in every window. Each draws an `Ask` inside
//! Zed's notification frame: a title, its facts as label and value rows, a muted footnote, and
//! its buttons in a row at the right, weighted as #870's prompts.
//!
//! An edit's question (#938) also leads with the agent's tile, says its task under the title, and
//! shows the change as removed and added lines.

use std::rc::Rc;

use gpui::{AnyElement, App, FontWeight, Hsla, Window};
use ui::prelude::*;

use crate::item_kind::{self, AgentMark, ItemKind};
use crate::prompts::{Role, render_button};

/// Lines a change shows before it says how many more there are.
const CHANGE_LINES: usize = 12;

/// Which matches of the removed text an edit replaces.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Occurrences {
    /// The one match the text has.
    #[default]
    One,
    /// Every match in the file.
    Every,
}

/// The text an edit takes out and puts in, line by line, as the agent asked for it (#938, D2).
#[derive(Clone, Debug, Default)]
pub(crate) struct Change {
    pub(crate) removed: Vec<String>,
    pub(crate) added: Vec<String>,
    pub(crate) occurrences: Occurrences,
}

/// One of the question's buttons.
type OnClick = Rc<dyn Fn(&mut Window, &mut App)>;

/// A question as the card draws it.
#[derive(Clone)]
pub(crate) struct Ask {
    title: SharedString,
    tile: Option<AgentMark>,
    subtitle: Option<SharedString>,
    change: Option<Change>,
    lines: Vec<SharedString>,
    rows: Vec<(SharedString, SharedString)>,
    footnote: Option<SharedString>,
    buttons: Vec<(SharedString, Role, OnClick)>,
}

impl Ask {
    pub(crate) fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            tile: None,
            subtitle: None,
            change: None,
            lines: Vec::new(),
            rows: Vec::new(),
            footnote: None,
            buttons: Vec::new(),
        }
    }

    /// The asking agent's tile, before the title.
    pub(crate) const fn tile(mut self, mark: AgentMark) -> Self {
        self.tile = Some(mark);
        self
    }

    /// A muted line under the title, such as the task the agent works on.
    pub(crate) fn subtitle(mut self, text: impl Into<SharedString>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    /// The change an edit would make, drawn under the lines.
    pub(crate) fn change(mut self, change: Change) -> Self {
        self.change = Some(change);
        self
    }

    /// A line of text under the title.
    pub(crate) fn line(mut self, text: impl Into<SharedString>) -> Self {
        self.lines.push(text.into());
        self
    }

    /// A fact, its label in the first column.
    pub(crate) fn row(mut self, label: &'static str, value: impl Into<SharedString>) -> Self {
        self.rows.push((label.into(), value.into()));
        self
    }

    pub(crate) fn footnote(mut self, text: impl Into<SharedString>) -> Self {
        self.footnote = Some(text.into());
        self
    }

    /// A button, drawn left to right in the order given.
    pub(crate) fn button(
        mut self,
        label: &'static str,
        role: Role,
        on_click: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.buttons.push((label.into(), role, Rc::new(on_click)));
        self
    }

    pub(crate) fn render(&self, cx: &App) -> AnyElement {
        let buttons = self
            .buttons
            .iter()
            .enumerate()
            .map(|(index, (label, role, on_click))| {
                let on_click = Rc::clone(on_click);
                render_button(index, label, *role, false, cx)
                    .on_click(move |_, window, cx| on_click(window, cx))
            })
            .collect::<Vec<_>>();
        v_flex()
            .debug_selector(|| "marley-ask".into())
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .children(self.tile.map(|mark| {
                        item_kind::square_tile(ItemKind::Thread(mark), mark.glyph(), px(22.), cx)
                    }))
                    .child(Label::new(self.title.clone()).weight(FontWeight::SEMIBOLD)),
            )
            .when_some(self.subtitle.clone(), |card, subtitle| {
                card.child(
                    Label::new(subtitle)
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
            })
            .children(
                self.lines
                    .iter()
                    .map(|line| Label::new(line.clone()).size(LabelSize::Small)),
            )
            .children(self.change.as_ref().map(|change| render_change(change, cx)))
            .when(!self.rows.is_empty(), |card| {
                card.child(
                    v_flex()
                        .gap_1()
                        .children(self.rows.iter().map(|(label, value)| {
                            h_flex()
                                .gap_3()
                                .items_start()
                                .child(
                                    div().w_16().flex_none().child(
                                        Label::new(label.clone())
                                            .size(LabelSize::Small)
                                            .color(Color::Muted),
                                    ),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .child(Label::new(value.clone()).size(LabelSize::Small)),
                                )
                        })),
                )
            })
            .when_some(self.footnote.clone(), |card, footnote| {
                card.child(
                    Label::new(footnote)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                )
            })
            .when(!buttons.is_empty(), |card| {
                card.child(
                    h_flex()
                        .pt_1()
                        .gap_1p5()
                        .flex_wrap()
                        .justify_end()
                        .children(buttons),
                )
            })
            .into_any_element()
    }
}

/// A change as removed lines on the deleted background and added lines on the created one, the
/// colours Zed's own diffs use; at most [`CHANGE_LINES`], then how many more.
fn render_change(change: &Change, cx: &App) -> AnyElement {
    let status = cx.theme().status();
    let total = change.removed.len() + change.added.len();
    let removed = change
        .removed
        .iter()
        .map(|text| change_line("−", text, Color::Deleted, status.deleted_background, cx));
    let added = change
        .added
        .iter()
        .map(|text| change_line("+", text, Color::Created, status.created_background, cx));
    let shown: Vec<Div> = removed.chain(added).take(CHANGE_LINES).collect();
    let more = total.saturating_sub(CHANGE_LINES);
    v_flex()
        .debug_selector(|| "marley-ask-change".into())
        .rounded_sm()
        .overflow_hidden()
        .children(shown)
        .when(more > 0, |lines| {
            lines.child(
                Label::new(format!("… {more} more lines"))
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
        })
        .when(change.occurrences == Occurrences::Every, |lines| {
            lines.child(
                Label::new("Every match in the file.")
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
        })
        .into_any_element()
}

/// One line of a change: its sign in the side's colour, then the text in the buffer font.
fn change_line(
    sign: &'static str,
    text: &str,
    sign_color: Color,
    background: Hsla,
    cx: &App,
) -> Div {
    h_flex()
        .px_1p5()
        .gap_2()
        .bg(background)
        .child(
            Label::new(sign)
                .size(LabelSize::Small)
                .buffer_font(cx)
                .color(sign_color),
        )
        .child(
            Label::new(text.to_owned())
                .size(LabelSize::Small)
                .buffer_font(cx),
        )
}
