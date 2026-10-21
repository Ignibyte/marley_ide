//! The agents' questions as the Dialogs and `MoreForms` boards draw them (#871).
//!
//! A settings or keymap change, a harness seat, a sensitive tool such as `agent_start`, and a
//! pairing each ask the user through a notification in every window. Each draws an `Ask` inside
//! Zed's notification frame: a title, its facts as label and value rows, a muted footnote, and
//! its buttons in a row at the right, weighted as #870's prompts.

use std::rc::Rc;

use gpui::{AnyElement, App, FontWeight, Window};
use ui::prelude::*;

use crate::prompts::{Role, render_button};

/// One of the question's buttons.
type OnClick = Rc<dyn Fn(&mut Window, &mut App)>;

/// A question as the card draws it.
#[derive(Clone)]
pub(crate) struct Ask {
    title: SharedString,
    lines: Vec<SharedString>,
    rows: Vec<(SharedString, SharedString)>,
    footnote: Option<SharedString>,
    buttons: Vec<(SharedString, Role, OnClick)>,
}

impl Ask {
    pub(crate) fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            lines: Vec::new(),
            rows: Vec::new(),
            footnote: None,
            buttons: Vec::new(),
        }
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
            .child(Label::new(self.title.clone()).weight(FontWeight::SEMIBOLD))
            .children(
                self.lines
                    .iter()
                    .map(|line| Label::new(line.clone()).size(LabelSize::Small)),
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
