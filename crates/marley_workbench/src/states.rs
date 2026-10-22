//! Empty and error states as the States board draws them (#857).
//!
//! When a place has nothing yet, or something it needs broke, it shows one card: a glyph tile, a
//! title, a line saying why or what to do, and its one action. A broken state's tile and border
//! take the error's tint.

use std::rc::Rc;

use gpui::{App, Window};
use ui::prelude::*;

use crate::prompts::{Role, render_button};

/// What the action does when clicked.
type OnClick = Rc<dyn Fn(&mut Window, &mut App)>;

/// One empty or error state.
pub(crate) struct EmptyState {
    glyph: SharedString,
    title: SharedString,
    body: SharedString,
    broken: bool,
    action: Option<(SharedString, Role, OnClick)>,
}

impl EmptyState {
    /// A state with nothing yet: its glyph, title and line.
    pub(crate) fn new(
        glyph: impl Into<SharedString>,
        title: impl Into<SharedString>,
        body: impl Into<SharedString>,
    ) -> Self {
        Self {
            glyph: glyph.into(),
            title: title.into(),
            body: body.into(),
            broken: false,
            action: None,
        }
    }

    /// Marks it as something broken: the error's tint on its tile and border.
    pub(crate) const fn broken(mut self) -> Self {
        self.broken = true;
        self
    }

    /// Its one action, drawn as #870's button in `role`.
    pub(crate) fn action(
        mut self,
        label: impl Into<SharedString>,
        role: Role,
        on_click: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), role, Rc::new(on_click)));
        self
    }

    /// The card, `id` keeping its button apart from another state's in the same view.
    pub(crate) fn render(&self, id: usize, cx: &App) -> AnyElement {
        let colors = cx.theme().colors();
        let status = cx.theme().status();
        let (tile, ink, border) = if self.broken {
            (
                status.error.opacity(0.14),
                Color::Error,
                status.error_border,
            )
        } else {
            (colors.element_background, Color::Muted, colors.border)
        };
        v_flex()
            .items_center()
            .gap_2()
            .max_w(px(360.))
            .p_4()
            .rounded_lg()
            .border_1()
            .border_color(border)
            .bg(colors.elevated_surface_background)
            .child(
                div()
                    .size(px(38.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_lg()
                    .bg(tile)
                    .child(
                        Label::new(self.glyph.clone())
                            .size(LabelSize::Small)
                            .buffer_font(cx)
                            .weight(gpui::FontWeight::BOLD)
                            .color(ink),
                    ),
            )
            .child(Label::new(self.title.clone()).weight(gpui::FontWeight::SEMIBOLD))
            .child(
                div().text_center().child(
                    Label::new(self.body.clone())
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
            )
            .children(self.action.clone().map(|(label, role, on_click)| {
                render_button(id, &label, role, false, cx)
                    .cursor_pointer()
                    .on_click(move |_, window, cx| on_click(window, cx))
            }))
            .into_any_element()
    }
}
