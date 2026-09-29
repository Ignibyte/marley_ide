//! The size a launch's first terminals open at: the last session's (#486).
//!
//! Since #485 a terminal opens at the size the last terminal view laid out, so its shell lays
//! its first prompt out for the width it will have. A launch's first terminals open before any
//! view has a size; the size the last session ended with, kept in Zed's key-value store when
//! Marley quits, is theirs until a view lays one out.

use db::kvp::KeyValueStore;
use gpui::App;
use terminal::TerminalBounds;
use util::ResultExt as _;

/// The store's scope and key for the last size.
const SCOPE: &str = "marley-terminal-size";
const KEY: &str = "last";

/// Opens this launch's first terminals at the last session's size, and keeps this one's for the
/// next launch when Marley quits.
pub fn init(cx: &App) {
    let store = KeyValueStore::global(cx);
    let kept = store
        .scoped(SCOPE)
        .read(KEY)
        .log_err()
        .flatten()
        .and_then(|text| serde_json::from_str::<TerminalBounds>(&text).log_err());
    if let Some(bounds) = kept {
        terminal::marley_seed_last_bounds(bounds);
    }
    cx.on_app_quit(move |_| {
        let bounds = terminal::marley_last_bounds();
        let store = store.clone();
        async move {
            let Some(text) = bounds.and_then(|bounds| serde_json::to_string(&bounds).log_err())
            else {
                return;
            };
            store
                .scoped(SCOPE)
                .write(KEY.to_string(), text)
                .await
                .log_err();
        }
    })
    .detach();
}
