//! The Ctrl Tab switcher's rows as the Splits board draws them (#883).
//!
//! Each row leads with a tile naming its tab's kind and says under its title where the tab is.

use gpui::{App, Window};
use tab_switcher::{MarleySwitcherLook, MarleySwitcherRow};
use terminal_view::TerminalView;
use ui::prelude::*;
use workspace::item::ItemHandle;

use crate::browser::BrowserView;

/// Sets the switcher's look.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(MarleySwitcherLook(switcher_row));
}

/// `item`'s tile and line (D1, D2): ">_" for a terminal, "www" for a Browser tab, a file's
/// extension, else the title's first letter; a file's path in its project, else the tab's tooltip,
/// when it says more than its title.
fn switcher_row(item: &dyn ItemHandle, _window: &mut Window, cx: &mut App) -> MarleySwitcherRow {
    let title = item.tab_content_text(0, cx);
    let glyph: SharedString = if item.downcast::<TerminalView>().is_some() {
        SharedString::new_static(">_")
    } else if item.downcast::<BrowserView>().is_some() {
        SharedString::new_static("www")
    } else if let Some(extension) = item
        .project_path(cx)
        .and_then(|path| path.path.extension().map(str::to_string))
    {
        extension.chars().take(3).collect::<String>().into()
    } else {
        crate::project_icons::initial(&title).into()
    };
    let line = item
        .project_path(cx)
        .map(|path| SharedString::from(path.path.as_unix_str().to_string()))
        .or_else(|| item.tab_tooltip_text(cx))
        .filter(|line| *line != title);
    let colors = cx.theme().colors();
    let mark = div()
        .flex_none()
        .w(px(30.))
        .py_0p5()
        .flex()
        .justify_center()
        .rounded_sm()
        .bg(colors.element_background)
        .child(
            Label::new(glyph)
                .size(LabelSize::XSmall)
                .buffer_font(cx)
                .color(Color::Muted),
        )
        .into_any_element();
    MarleySwitcherRow { tile: mark, line }
}
