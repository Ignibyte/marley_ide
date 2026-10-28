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

use gpui::{AnyElement, App, Hsla, SharedString, rems};
use serde_json::Value;
use ui::prelude::*;

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

/// How a property's value is coloured, as the `BrainPage` board draws its pills (#939, D4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tone {
    Plain,
    /// A `status` that is settled: decided or done.
    Settled,
    /// A `follow_up_by` day already past.
    Overdue,
    /// One of the page's `tags`.
    Tag,
}

/// The tone of `value` under `key`. Only the key and the value decide it.
pub(super) fn tone(key: &str, value: &Value) -> Tone {
    let text = value.as_str().unwrap_or_default();
    match key {
        "status" if matches!(text, "decided" | "done") => Tone::Settled,
        "follow_up_by" if is_past(text) => Tone::Overdue,
        "tags" => Tone::Tag,
        _ => Tone::Plain,
    }
}

/// Whether `day`, `YYYY-MM-DD`, is before today here; anything else is not.
fn is_past(day: &str) -> bool {
    chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d")
        .is_ok_and(|day| day < chrono::Local::now().date_naive())
}

/// `text` on a pill in `tone`'s colours; an overdue day says so after it.
pub(super) fn pill(text: SharedString, tone: Tone, cx: &App) -> AnyElement {
    let colors = cx.theme().colors();
    let status = cx.theme().status();
    let (ink, fill, border): (Hsla, Hsla, Hsla) = match tone {
        Tone::Plain => (
            colors.text,
            colors.element_background,
            colors.border_variant,
        ),
        Tone::Settled => (
            status.success,
            status.success_background,
            status.success_border,
        ),
        Tone::Overdue => (status.error, status.error_background, status.error_border),
        Tone::Tag => {
            let teal = colors.terminal_ansi_cyan;
            (teal, teal.opacity(0.14), teal.opacity(0.4))
        }
    };
    let text = if tone == Tone::Overdue {
        SharedString::from(format!("{text} · overdue"))
    } else {
        text
    };
    div()
        .flex_none()
        .px_2()
        .rounded_full()
        .border_1()
        .border_color(border)
        .bg(fill)
        .child(
            Label::new(text)
                .size(LabelSize::Small)
                .color(Color::Custom(ink)),
        )
        .into_any_element()
}

/// A value under `key` as the page shows it: a list as pills, a scalar as a pill, an object as
/// compact JSON on one, each in its tone (#939).
pub(super) fn value(key: &str, value: &Value, cx: &App) -> AnyElement {
    match value {
        Value::Array(items) => h_flex()
            .flex_wrap()
            .gap_1()
            .children(
                items
                    .iter()
                    .map(|item| pill(scalar_text(item), tone(key, item), cx)),
            )
            .into_any_element(),
        Value::Null => Label::new("(empty)")
            .size(LabelSize::Small)
            .color(Color::Muted)
            .into_any_element(),
        other => pill(scalar_text(other), tone(key, other), cx),
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
