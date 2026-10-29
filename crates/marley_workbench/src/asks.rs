//! The agents' questions as the Dialogs and `MoreForms` boards draw them (#871).
//!
//! A settings or keymap change, a harness seat, a sensitive tool such as `agent_start`, and a
//! pairing each ask the user through a notification in every window. Each draws an `Ask` inside
//! Zed's notification frame: a title, its facts as label and value rows, a muted footnote, and
//! its buttons in a row at the right, weighted as #870's prompts.
//!
//! An edit's question (#938) also leads with the agent's tile, says its task under the title, and
//! shows the change as removed and added lines. A pairing's (#947) shows its code in digit boxes
//! and the grants as a radio list, whose pick its Pair button reads.

use std::cell::Cell;
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

/// One choice in a question's radio list (#947).
#[derive(Clone, Debug)]
pub(crate) struct Choice {
    pub(crate) label: SharedString,
    /// What the choice allows, under its label.
    pub(crate) line: SharedString,
}

/// A radio list and the index of its pick, which a button reads when it is clicked.
#[derive(Clone, Debug)]
pub(crate) struct Choices {
    pub(crate) items: Vec<Choice>,
    pub(crate) chosen: Rc<Cell<usize>>,
}

/// A question as the card draws it.
#[derive(Clone)]
pub(crate) struct Ask {
    title: SharedString,
    tile: Option<AgentMark>,
    subtitle: Option<SharedString>,
    change: Option<Change>,
    lines: Vec<SharedString>,
    code: Option<SharedString>,
    choices: Option<Choices>,
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
            code: None,
            choices: None,
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

    /// A code to compare, each digit in a box of its own, drawn under the lines.
    pub(crate) fn code(mut self, code: impl Into<SharedString>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// A radio list under the code; a click on a choice picks it.
    pub(crate) fn choices(mut self, choices: Choices) -> Self {
        self.choices = Some(choices);
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
            .children(self.code.as_ref().map(|code| render_code(code, cx)))
            .children(
                self.choices
                    .as_ref()
                    .map(|choices| render_choices(choices, cx)),
            )
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

/// A code as the `AgentAccess` board draws it: each digit in a bordered box in the buffer font, a
/// space between groups kept as a gap.
fn render_code(code: &str, cx: &App) -> AnyElement {
    let colors = cx.theme().colors();
    let mut boxes: Vec<AnyElement> = Vec::new();
    for character in code.chars() {
        if character.is_whitespace() {
            boxes.push(div().w_2().into_any_element());
            continue;
        }
        boxes.push(
            div()
                .w(px(28.))
                .h(px(34.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_md()
                .border_1()
                .border_color(colors.border)
                .bg(colors.editor_background)
                .child(
                    Label::new(character.to_string())
                        .size(LabelSize::Large)
                        .buffer_font(cx),
                )
                .into_any_element(),
        );
    }
    h_flex()
        .debug_selector(|| "marley-ask-code".into())
        .gap_1()
        .py_1()
        .children(boxes)
        .into_any_element()
}

/// A radio list: a row per choice, its dot filled in the accent when picked, its line muted
/// under the label. A click picks the row and redraws the window, since the card is drawn by a
/// notification that holds no state of its own.
fn render_choices(choices: &Choices, cx: &App) -> AnyElement {
    let colors = cx.theme().colors();
    let picked = choices.chosen.get();
    let rows = choices
        .items
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let chosen = Rc::clone(&choices.chosen);
            let is_picked = index == picked;
            let dot = div()
                .mt(px(3.))
                .size(px(12.))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .border_1()
                .border_color(if is_picked {
                    colors.text_accent
                } else {
                    colors.border
                })
                .when(is_picked, |dot| {
                    dot.child(div().size(px(6.)).rounded_full().bg(colors.text_accent))
                });
            h_flex()
                .id(("marley-ask-choice", index))
                .items_start()
                .gap_2()
                .px_1p5()
                .py_1()
                .rounded_sm()
                .cursor_pointer()
                .when(is_picked, |row| row.bg(colors.element_selected))
                .hover(|row| row.bg(colors.element_hover))
                .child(dot)
                .child(
                    v_flex()
                        .child(Label::new(choice.label.clone()).size(LabelSize::Small))
                        .child(
                            Label::new(choice.line.clone())
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        ),
                )
                .on_click(move |_, window, _| {
                    chosen.set(index);
                    window.refresh();
                })
        })
        .collect::<Vec<_>>();
    v_flex()
        .debug_selector(|| "marley-ask-choices".into())
        .gap_0p5()
        .children(rows)
        .into_any_element()
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
