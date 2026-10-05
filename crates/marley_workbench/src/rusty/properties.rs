//! A page's properties on its Page tab (#645): each frontmatter key in a quiet column and its value
//! beside it, a value short of room dropping under its key, a hairline between rows. The Page tab
//! puts its in-place editors and remove buttons into these rows (#656). The layout is
//! Ely GPUI Components' `DescriptionList` (`src/data_display/records.rs` at `2f8b2f6`), ported
//! onto Zed's theme and `ui` crate; its notice is below.

// The layout is ported from Ely GPUI Components, under this notice:
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

use gpui::{AnyElement, App, SharedString, rems};
use serde_json::Value;
use ui::{Chip, prelude::*};

/// One property's row: its key in a quiet column, its value beside it (dropping under the key
/// when short of room), what ends the row, and a hairline under it unless it is the last.
pub(super) fn row(
    key: SharedString,
    value: AnyElement,
    end: Option<AnyElement>,
    hairline: bool,
    cx: &App,
) -> AnyElement {
    h_flex()
        .flex_wrap()
        .items_start()
        .gap_x_4()
        .gap_y_0p5()
        .py_1p5()
        .when(hairline, |row| {
            row.border_b_1()
                .border_color(cx.theme().colors().border_variant)
        })
        .child(
            div()
                .flex_none()
                .w(rems(10.))
                .child(Label::new(key).size(LabelSize::Small).color(Color::Muted)),
        )
        .child(div().flex_1().min_w(rems(5.)).child(value))
        .children(end)
        .into_any_element()
}

/// A value as the page shows it: a list as chips, a scalar as its text, an object as compact JSON.
pub(super) fn value(value: &Value) -> AnyElement {
    match value {
        Value::Array(items) => h_flex()
            .flex_wrap()
            .gap_1()
            .children(items.iter().map(|item| Chip::new(scalar_text(item))))
            .into_any_element(),
        Value::Null => Label::new("(empty)")
            .size(LabelSize::Small)
            .color(Color::Muted)
            .into_any_element(),
        other => Label::new(scalar_text(other))
            .size(LabelSize::Small)
            .into_any_element(),
    }
}

/// A scalar's text, and anything else as compact JSON.
fn scalar_text(value: &Value) -> SharedString {
    match value {
        Value::String(text) => text.clone().into(),
        Value::Null => SharedString::new_static("(empty)"),
        other => other.to_string().into(),
    }
}
