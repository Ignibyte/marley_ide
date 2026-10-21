//! Toasts as the Inbox board draws them (#869).
//!
//! In the Marley layout a toast reads as a coloured dot, a title and the rest of its message as a
//! muted line under it. Marley raises most of its notices with one string, so the title is the
//! message's first sentence or clause: up to its first ". " or ": ". The dot is red for a failure
//! and the accent otherwise. A hook `Workspace::show_toast` asks.

use gpui::{App, SharedString};
use ui::Color;
use workspace::notifications::{MarleyToast, MarleyToastLook};

/// The words that mark a toast as a failure.
const FAILURES: [&str; 3] = ["Could not", "failed", "Failed"];

/// Sets the hook. [`crate::init`] calls it once.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(MarleyToastLook(look));
}

/// A toast's look in the Marley layout; none in Zed's, which keeps Zed's toasts.
fn look(message: &str, cx: &App) -> Option<MarleyToast> {
    if !crate::marley_layout(cx) {
        return None;
    }
    let (title, line) = split(message);
    let dot = if FAILURES.iter().any(|word| message.contains(word)) {
        Color::Error
    } else {
        Color::Accent
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
