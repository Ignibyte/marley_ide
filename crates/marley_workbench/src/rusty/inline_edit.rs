//! The in-place editor of a brain page's title, name and property values (#656): the value as text,
//! a hover background and a pencil on hover; a click puts a single-line editor in its place with
//! all of the value selected. It is Ely GPUI Components' `InlineEdit` (`src/forms/inline.rs` at
//! `2f8b2f6`), ported onto Zed's `Editor`, theme and `ui` crate; its notice is below. Ely's own
//! input, theme, focus ring and keyed state stay behind: the Page tab holds the state, and Zed's
//! `menu` actions bring Enter and Escape to it.

// The editor's shape is ported from Ely GPUI Components, under this notice:
//
// MIT License
//
// Copyright (c) 2026 Ely GPUI Component contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use editor::{Editor, actions::SelectAll};
use gpui::{AnyElement, App, AppContext as _, ClickEvent, ElementId, Entity, SharedString, Window};
use ui::prelude::*;

/// A single-line editor holding `text`, all of it selected.
pub(super) fn editor_for(text: &str, window: &mut Window, cx: &mut App) -> Entity<Editor> {
    cx.new(|cx| {
        let mut editor = Editor::single_line(window, cx);
        editor.set_text(text, window, cx);
        editor.select_all(&SelectAll, window, cx);
        editor
    })
}

/// The read state: `content` with a hover background and a pencil shown on hover, a click opening
/// the editor; `content` alone when it cannot be edited now.
pub(super) fn shown(
    id: impl Into<ElementId>,
    content: impl IntoElement,
    enabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> AnyElement {
    if !enabled {
        return div().child(content).into_any_element();
    }
    let id = id.into();
    let group = SharedString::from(format!("rusty-inline-{id}"));
    h_flex()
        .id(id)
        .group(group.clone())
        .gap_1()
        .px_1()
        .mx_neg_1()
        .rounded_sm()
        .cursor_text()
        .hover(|style| style.bg(cx.theme().colors().ghost_element_hover))
        .child(content)
        .child(
            div().invisible().group_hover(group, Styled::visible).child(
                Icon::new(IconName::Pencil)
                    .size(IconSize::XSmall)
                    .color(Color::Muted),
            ),
        )
        .on_click(on_click)
        .into_any_element()
}

/// The open state: the editor in the value's place, with what its text needs under it when the
/// last Enter could not be kept.
pub(super) fn field(editor: &Entity<Editor>, error: Option<&SharedString>, cx: &App) -> AnyElement {
    v_flex()
        .w_full()
        .gap_0p5()
        .child(
            div()
                .w_full()
                .px_1()
                .py_0p5()
                .rounded_sm()
                .border_1()
                .border_color(cx.theme().colors().border_focused)
                .bg(cx.theme().colors().editor_background)
                .child(editor.clone()),
        )
        .children(error.map(|error| {
            Label::new(error.clone())
                .size(LabelSize::Small)
                .color(Color::Error)
        }))
        .into_any_element()
}
