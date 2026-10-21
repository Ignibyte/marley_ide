//! Prompts as the Dialogs board draws them (#870).
//!
//! In the Marley layout a prompt is a rounded card: its message as the title, its detail as the
//! body, and its buttons in a row at the right. Zed lists the main button first; the board puts it
//! at the right, so the row runs backwards. Zed's prompt renderer keeps the scrim, the focus, the
//! keys and the answer, and asks the hook for the card.

use std::rc::Rc;

use gpui::{AnyElement, App, Div, FontWeight, Stateful, Window, px};
use ui::prelude::*;
use ui_prompt::{MarleyPrompt, MarleyPromptLook};

/// The labels that cancel, drawn quiet whether or not the prompt marks them as its cancel.
const QUIET: [&str; 3] = ["Cancel", "Dismiss", "Not Now"];

/// The verbs whose main button is drawn as a danger.
const DANGER: [&str; 8] = [
    "Close",
    "Quit",
    "Delete",
    "Remove",
    "Discard",
    "End",
    "Clear",
    "Overwrite",
];

/// A button's weight on the board.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Quiet,
    Plain,
    Primary,
    Danger,
}

/// Sets the hook. [`crate::init`] calls it once.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(MarleyPromptLook(card));
}

/// The prompt's card in the Marley layout; none in Zed's, which keeps Zed's prompts.
fn card(prompt: &MarleyPrompt<'_>, _window: &mut Window, cx: &mut App) -> Option<AnyElement> {
    if !crate::marley_layout(cx) {
        return None;
    }
    let roles = prompt
        .actions
        .iter()
        .enumerate()
        .map(|(index, label)| role(prompt, index, label))
        .collect::<Vec<_>>();
    let colors = cx.theme().colors();
    let border = if roles.first() == Some(&Role::Danger) {
        cx.theme().status().warning_border
    } else {
        colors.border
    };
    let buttons = prompt
        .actions
        .iter()
        .zip(&roles)
        .enumerate()
        .rev()
        .map(|(index, (label, role))| {
            let answer = Rc::clone(&prompt.answer);
            let chosen = index == prompt.active && index != 0;
            render_button(index, label, *role, chosen, cx)
                .on_click(move |_, window, cx| answer(index, window, cx))
        })
        .collect::<Vec<_>>();
    let card = v_flex()
        .debug_selector(|| "marley-prompt".into())
        .occlude()
        .w(px(420.))
        .p_4()
        .gap_2p5()
        .rounded(px(16.))
        .border_1()
        .border_color(border)
        .bg(colors.elevated_surface_background)
        .shadow_lg()
        .child(Label::new(plain(prompt.message)).weight(FontWeight::SEMIBOLD))
        .when_some(prompt.detail, |card, detail| {
            card.child(
                Label::new(plain(detail))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
        })
        .child(
            h_flex()
                .pt_1()
                .gap_1p5()
                .flex_wrap()
                .justify_end()
                .children(buttons),
        );
    Some(card.into_any_element())
}

/// The weight of action `index`: a cancel is quiet; the first is danger for a destructive verb
/// and primary otherwise; the rest are plain.
fn role(prompt: &MarleyPrompt<'_>, index: usize, label: &str) -> Role {
    if prompt.cancel == Some(index) || QUIET.contains(&label) {
        Role::Quiet
    } else if index > 0 {
        Role::Plain
    } else if DANGER
        .iter()
        .any(|verb| label.split_whitespace().next() == Some(verb))
    {
        Role::Danger
    } else {
        Role::Primary
    }
}

/// Zed's prompts write their message as Markdown; the card draws it as text, without the code
/// marks.
fn plain(text: &str) -> String {
    text.replace('`', "")
}

/// A button as the board draws its role: quiet on nothing, plain filled with a border, primary and
/// danger solid in the accent or the error colour with dark ink. A chosen button (the arrow keys')
/// has a focus border.
fn render_button(index: usize, label: &str, role: Role, chosen: bool, cx: &App) -> Stateful<Div> {
    let colors = cx.theme().colors();
    let (fill, border, ink) = match role {
        Role::Quiet => (None, None, Color::Muted),
        Role::Plain => (
            Some(colors.element_background),
            Some(colors.border),
            Color::Default,
        ),
        Role::Primary => (
            Some(colors.text_accent),
            Some(colors.text_accent),
            Color::Custom(colors.background),
        ),
        Role::Danger => {
            let error = cx.theme().status().error;
            (Some(error), Some(error), Color::Custom(colors.background))
        }
    };
    let border = if chosen {
        Some(colors.border_focused)
    } else {
        border
    };
    h_flex()
        .id(("marley-prompt", index))
        .h(px(30.))
        .px_3()
        .rounded(px(8.))
        .border_1()
        .border_color(border.unwrap_or_else(gpui::transparent_black))
        .when_some(fill, Styled::bg)
        .when(fill.is_none(), |button| {
            button.hover(|style| style.bg(colors.ghost_element_hover))
        })
        .when(fill.is_some(), |button| {
            button.hover(|style| style.opacity(0.9))
        })
        .cursor_pointer()
        .child(
            Label::new(label.to_string())
                .size(LabelSize::Small)
                .weight(if matches!(role, Role::Primary | Role::Danger) {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .color(ink),
        )
}
