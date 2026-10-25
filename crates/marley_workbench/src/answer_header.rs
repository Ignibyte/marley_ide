//! The header over each answer of a thread Marley hosts (#914), as the Main board draws it: the
//! agent's tile and name, its model and effort, and on the newest answer the state at the right.

use agent_ui::{AnswerFacts, AnswerState};
use gpui::{AnyElement, App, FontWeight, px};
use project::AgentId;
use ui::prelude::*;

use crate::item_kind::{AgentMark, ItemKind, ItemState, tile};

/// An answer's header, from what the thread view knows of it.
pub(crate) fn answer_header(facts: &AnswerFacts, cx: &App) -> AnyElement {
    let mark = AgentMark::from_agent_id(&AgentId::new(facts.agent_id.clone()));
    let about = match (&facts.model, &facts.effort) {
        (Some(model), Some(effort)) => Some(format!("{model} \u{b7} {effort}")),
        (Some(model), None) => Some(model.to_string()),
        (None, _) => None,
    };
    h_flex()
        .debug_selector(|| "marley-answer-header".into())
        .w_full()
        .px_4()
        .pt_2()
        .pb_1()
        .gap_2()
        .child(tile(ItemKind::Thread(mark), mark.glyph(), px(32.), cx))
        .child(
            Label::new(facts.agent_name.clone())
                .size(LabelSize::Small)
                .weight(FontWeight::SEMIBOLD),
        )
        .children(about.map(|about| Label::new(about).size(LabelSize::Small).color(Color::Muted)))
        .child(div().flex_1())
        .children(facts.state.map(|state| state_chip(state, cx)))
        .into_any_element()
}

/// The newest answer's state: a dot in its colour and its words.
fn state_chip(state: AnswerState, cx: &App) -> AnyElement {
    let (item, words) = match state {
        AnswerState::Working(elapsed) => (
            ItemState::Working,
            format!(
                "Working \u{b7} {}",
                crate::thread_activity::elapsed_words(elapsed)
            ),
        ),
        AnswerState::NeedsYou => (ItemState::NeedsYou, "Needs you".to_string()),
    };
    h_flex()
        .flex_none()
        .gap_1p5()
        .child(div().size_1p5().rounded_full().bg(item.dot(cx)))
        .child(
            Label::new(words)
                .size(LabelSize::Small)
                .color(item.line(cx)),
        )
        .into_any_element()
}
