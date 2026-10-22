//! The Ctrl Tab switcher's rows as the Splits board draws them (#883), and the center tabs and the
//! split menu as the Chrome board draws them (#866).
//!
//! Each switcher row leads with a tile naming its tab's kind and says under its title where the
//! tab is. In the Marley layout each tab leads with the same tile and carries its state as a dot,
//! and the split menu draws each split as a small picture.

use gpui::{App, Hsla, Window};
use marley_terminal::BlockState;
use tab_switcher::{MarleySwitcherLook, MarleySwitcherRow};
use terminal_view::TerminalView;
use ui::prelude::*;
use workspace::SplitDirection;
use workspace::item::ItemHandle;
use workspace::pane::{MarleySplitPictures, MarleyTabLook, MarleyTabMark};

use crate::browser::BrowserView;

/// Sets the switcher's look, the tabs' and the split menu's.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(MarleySwitcherLook(switcher_row));
    cx.set_global(MarleyTabLook(tab_mark));
    cx.set_global(MarleySplitPictures(split_picture));
}

/// `item`'s tile and state in the Marley layout (#866); none in Zed's, which keeps Zed's tabs.
fn tab_mark(item: &dyn ItemHandle, _window: &mut Window, cx: &mut App) -> Option<MarleyTabMark> {
    if !crate::marley_layout(cx) {
        return None;
    }
    Some(MarleyTabMark {
        tile: tile(glyph_of(item, cx), px(24.), cx),
        dot: state_dot(item, cx),
    })
}

/// A terminal tab's state (#866, D1): its last command failed, runs, or the terminal needs you.
fn state_dot(item: &dyn ItemHandle, cx: &App) -> Option<Hsla> {
    let view = item.downcast::<TerminalView>()?;
    let status = cx.theme().status();
    let terminal = view.read(cx).terminal().read(cx);
    let last = terminal.marley_anchored().live().last();
    match last.map(|block| (block.state, block.exit_code.0)) {
        Some((BlockState::Running, _)) => Some(cx.theme().colors().text_accent),
        Some((BlockState::Finished, Some(code))) if code != 0 => Some(status.error),
        _ => crate::notifications::unread(view.entity_id(), cx).then_some(status.warning),
    }
}

/// A split's picture (#866): a small window frame with the new pane's side filled.
fn split_picture(direction: SplitDirection, cx: &App) -> AnyElement {
    let colors = cx.theme().colors();
    // Each pane fills its share of the frame both ways: a row centres its children, which would
    // leave them no height.
    let new_pane = || {
        div()
            .flex_1()
            .size_full()
            .rounded_xs()
            .bg(colors.text_accent.opacity(0.7))
    };
    let old_pane = || div().flex_1().size_full().rounded_xs().bg(colors.border);
    let frame = match direction {
        SplitDirection::Right => h_flex().child(old_pane()).child(new_pane()),
        SplitDirection::Left => h_flex().child(new_pane()).child(old_pane()),
        SplitDirection::Up => v_flex().child(new_pane()).child(old_pane()),
        SplitDirection::Down => v_flex().child(old_pane()).child(new_pane()),
    };
    frame
        .flex_none()
        .w(px(22.))
        .h(px(16.))
        .p_0p5()
        .gap_0p5()
        .rounded_sm()
        .border_1()
        .border_color(colors.border)
        .into_any_element()
}

/// The glyph naming `item`'s kind (D1): ">_" for a terminal, "www" for a Browser tab, a file's
/// extension, else the title's first letter.
fn glyph_of(item: &dyn ItemHandle, cx: &App) -> SharedString {
    if item.downcast::<TerminalView>().is_some() {
        SharedString::new_static(">_")
    } else if item.downcast::<BrowserView>().is_some() {
        SharedString::new_static("www")
    } else if let Some(extension) = item
        .project_path(cx)
        .and_then(|path| path.path.extension().map(str::to_string))
    {
        extension.chars().take(3).collect::<String>().into()
    } else {
        crate::project_icons::initial(&item.tab_content_text(0, cx)).into()
    }
}

/// `glyph` on a tile `width` wide.
fn tile(glyph: SharedString, width: Pixels, cx: &App) -> AnyElement {
    div()
        .flex_none()
        .w(width)
        .py_0p5()
        .flex()
        .justify_center()
        .rounded_sm()
        .bg(cx.theme().colors().element_background)
        .child(
            Label::new(glyph)
                .size(LabelSize::XSmall)
                .buffer_font(cx)
                .color(Color::Muted),
        )
        .into_any_element()
}

/// `item`'s tile and line (D1, D2): ">_" for a terminal, "www" for a Browser tab, a file's
/// extension, else the title's first letter; a file's path in its project, else the tab's tooltip,
/// when it says more than its title.
fn switcher_row(item: &dyn ItemHandle, _window: &mut Window, cx: &mut App) -> MarleySwitcherRow {
    let title = item.tab_content_text(0, cx);
    let glyph = glyph_of(item, cx);
    let line = item
        .project_path(cx)
        .map(|path| SharedString::from(path.path.as_unix_str().to_string()))
        .or_else(|| item.tab_tooltip_text(cx))
        .filter(|line| *line != title);
    MarleySwitcherRow {
        tile: tile(glyph, px(30.), cx),
        line,
    }
}
