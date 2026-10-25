//! Toasts as the Inbox board draws them (#869).
//!
//! In the Marley layout a toast reads as a coloured dot, a title and the rest of its message as a
//! muted line under it. Marley raises most of its notices with one string, so the title is the
//! message's first sentence or clause: up to its first ". " or ": ". The dot says the toast's kind
//! (#925): yellow for a warning, green for a thing done, red for a failure, the accent for a
//! question, grey for the rest. A toast with no kind is read from its words. A hook
//! `Workspace::show_toast` asks.

use gpui::{App, SharedString};
use ui::Color;
use workspace::notifications::{MarleyToast, MarleyToastKind, MarleyToastLook};

/// The words that mark a toast as a failure.
const FAILURES: [&str; 3] = ["Could not", "failed", "Failed"];

/// Sets the hook. [`crate::init`] calls it once.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(MarleyToastLook(look));
}

/// A toast's look in the Marley layout; none in Zed's, which keeps Zed's toasts.
fn look(message: &str, kind: Option<MarleyToastKind>, cx: &App) -> Option<MarleyToast> {
    if !crate::marley_layout(cx) {
        return None;
    }
    let (title, line) = split(message);
    // The raiser's kind, else the words': a failure, a question, else neutral (#925, D1, D2).
    let kind = kind.unwrap_or_else(|| {
        if FAILURES.iter().any(|word| message.contains(word)) {
            MarleyToastKind::Error
        } else if title.trim_end().ends_with('?') {
            MarleyToastKind::Question
        } else {
            MarleyToastKind::Neutral
        }
    });
    let dot = match kind {
        MarleyToastKind::Warning => Color::Warning,
        MarleyToastKind::Success => Color::Success,
        MarleyToastKind::Error => Color::Error,
        MarleyToastKind::Question => Color::Accent,
        MarleyToastKind::Neutral => Color::Muted,
    };
    Some(MarleyToast {
        title: title.into(),
        line,
        dot,
    })
}

/// The message's title and the rest. The title ends at the first ". " or ": " after a title of
/// more than one word, so a prefix such as `marley: ` stays in it; the line starts with a capital.
fn split(message: &str) -> (String, Option<SharedString>) {
    let message = message.trim();
    let mut search_from = 0;
    while let Some(offset) = message[search_from..].find([':', '.']) {
        let end = search_from + offset;
        let rest = &message[end + 1..];
        if rest.starts_with(' ') && message[..end].contains(' ') {
            let rest = rest.trim_start();
            let line = rest.chars().next().map(|first| {
                let mut line = first.to_uppercase().collect::<String>();
                line.push_str(&rest[first.len_utf8()..]);
                SharedString::from(line)
            });
            return (message[..end].to_string(), line);
        }
        search_from = end + 1;
    }
    (message.trim_end_matches('.').to_string(), None)
}
