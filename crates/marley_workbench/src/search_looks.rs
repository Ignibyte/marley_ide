//! The boards' looks for Zed's own views, each a global the view asks for.
//!
//! The `SearchMore` board's for three views (#891), the `EditorPopups` board's code actions
//! heading (#856), its Ask an agent to fix this, the rename note (#899) and inline assist's offer to
//! send the selection to an agent (#900), and the Splits board's Collab card (#890).
//!
//! Project search counts its matches and the files that hold them; the branch picker gives each
//! branch a dot and a note; each extension's card leads with a tile and says Installed in green.
//! Each is a global the Zed view asks for; without it the view is Zed's.

use collab_ui::collab_panel::MarleyCollabLook;
use extensions_ui::MarleyExtensionLook;
use git_ui::branch_picker::{MarleyBranchLook, MarleyBranchMark, MarleyBranchRow};
use gpui::{AnyElement, App, SharedString};
use search::project_search::MarleySearchLook;
use ui::prelude::*;

/// Sets the three looks; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(MarleySearchLook);
    cx.set_global(MarleyBranchLook(branch_mark));
    cx.set_global(MarleyExtensionLook(extension_tile));
    cx.set_global(editor::code_context_menus::MarleyCodeActionsLook {
        footer: code_actions_footer,
    });
    cx.set_global(editor::MarleyRenameLook);
    cx.set_global(MarleyCollabLook);
    cx.set_global(agent_ui::MarleyInlineAssistLook {
        line: crate::send_selection::inline_assist_line,
    });
    cx.set_global(agent_ui::MarleyThreadLook {
        extra_chip: crate::beside::apply_chip,
    });
}

/// A branch's dot and note (#891, D2): the coat's orange and "current" for the head, the lilac and
/// "worktree" for a branch a linked worktree has checked out, "remote" for a remote branch, and how
/// far a local branch is behind or ahead of its upstream.
fn branch_mark(row: &MarleyBranchRow, cx: &App) -> MarleyBranchMark {
    let colors = cx.theme().colors();
    let lilac = cx.theme().players().color_for_participant(2).cursor;
    let (color, note) = if row.is_head {
        (
            colors.text_accent,
            Some(SharedString::new_static("current")),
        )
    } else if row.in_worktree {
        (lilac, Some(SharedString::new_static("worktree")))
    } else if row.is_remote {
        (colors.text_muted, Some(SharedString::new_static("remote")))
    } else if row.behind > 0 {
        (
            colors.text_muted,
            Some(format!("{} behind", row.behind).into()),
        )
    } else if row.ahead > 0 {
        (
            colors.text_muted,
            Some(format!("{} ahead", row.ahead).into()),
        )
    } else {
        (colors.text_muted, None)
    };
    MarleyBranchMark {
        dot: div()
            .flex_none()
            .size_2()
            .rounded_full()
            .bg(color)
            .into_any_element(),
        note,
    }
}

/// An extension's tile: the first two letters of its name in the theme's raised surface.
fn extension_tile(name: &str, cx: &App) -> AnyElement {
    let colors = cx.theme().colors();
    let mut letters = name.chars().filter(|letter| letter.is_alphanumeric());
    let initials: String = letters
        .next()
        .map(|first| {
            let mut initials = first.to_uppercase().collect::<String>();
            initials.extend(letters.next().map(|second| second.to_ascii_lowercase()));
            initials
        })
        .unwrap_or_default();
    div()
        .flex_none()
        .size_7()
        .rounded_md()
        .flex()
        .items_center()
        .justify_center()
        .bg(colors.element_background)
        .border_1()
        .border_color(colors.border)
        .child(
            Label::new(initials)
                .size(LabelSize::Small)
                .color(Color::Accent),
        )
        .into_any_element()
}

/// The code actions menu's last row (#899): Ask an agent to fix this…, which closes the menu and
/// runs `marley::AskAgentToFix` from the editor.
fn code_actions_footer(cx: &App) -> AnyElement {
    let hover = cx.theme().colors().element_hover;
    h_flex()
        .id("marley-ask-agent-to-fix")
        // The menu's list takes the mouse from the editor under it; this row is outside the list.
        .occlude()
        .mx_1()
        .px_1p5()
        .py_0p5()
        .rounded_sm()
        .cursor_pointer()
        .hover(move |row| row.bg(hover))
        .child(Label::new("Ask an agent to fix this…").color(Color::Accent))
        .on_click(|_, window, cx| {
            window.dispatch_action(Box::new(editor::actions::Cancel), cx);
            window.dispatch_action(Box::new(crate::AskAgentToFix), cx);
        })
        .into_any_element()
}
