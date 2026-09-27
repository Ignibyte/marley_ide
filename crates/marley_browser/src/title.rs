//! A page's title as its script sets it (#582).
//!
//! Chromium sends no CDP event for a title a script sets after the load:
//! `Target.targetInfoChanged` carries a page's first title and not a later one, and the Page
//! domain has no title event. So a watcher in an isolated world of the page's main frame reports
//! each new title through a CDP binding.

use serde_json::{Value, json};

use crate::cdp::CdpError;
use crate::page::Page;

/// The isolated world the watcher runs in, apart from the page's own scripts.
pub const WORLD: &str = "marley-title";

/// The binding the watcher reports through: a function in the world whose call Chromium sends as
/// `Runtime.bindingCalled`, the title its payload.
pub const BINDING: &str = "marleyTitle";

/// The watcher, which a world runs once however many times it is evaluated there.
///
/// In the top frame only, it keeps the last title it reported, the page's at its start, and
/// reports `document.title` whenever a change in the document leaves it different. Half of a
/// surrogate pair fails the whole CDP message it is in (#518), so the title is made well formed
/// and cut at 1,000 characters, never inside a pair. It watches the whole document until
/// `DOMContentLoaded` and the head alone after, so a loaded page that changes its body often
/// pays nothing for it.
pub const WATCHER: &str = r"(() => {
  if (globalThis.__marleyTitleWatcher || window !== window.top) return;
  globalThis.__marleyTitleWatcher = true;
  const read = () => {
    const whole = document.title;
    const title = (whole.toWellFormed ? whole.toWellFormed() : whole).slice(0, 1000);
    return /[\uD800-\uDBFF]$/.test(title) ? title.slice(0, -1) : title;
  };
  let last = read();
  const report = () => {
    const title = read();
    if (title !== last) {
      last = title;
      marleyTitle(title);
    }
  };
  const observer = new MutationObserver(report);
  const options = { childList: true, subtree: true, characterData: true };
  const watchHead = () => {
    observer.disconnect();
    observer.observe(document.head || document.documentElement || document, options);
    report();
  };
  if (document.readyState === 'loading') {
    observer.observe(document, options);
    document.addEventListener('DOMContentLoaded', watchHead, { once: true });
  } else {
    watchHead();
  }
})()";

impl Page {
    /// Watches the title of the page `session` shows (#582): the binding for the watcher's world,
    /// the watcher in that world of each document the session loads from now on, and in a new
    /// world of the main frame's document already loaded.
    ///
    /// # Errors
    ///
    /// When a call fails or the browser's answer lacks the main frame or a context.
    pub async fn watch_title(&self, session: &str) -> Result<(), CdpError> {
        self.call_in(
            session,
            "Runtime.addBinding",
            json!({ "name": BINDING, "executionContextName": WORLD }),
        )
        .await?;
        self.call_in(
            session,
            "Page.addScriptToEvaluateOnNewDocument",
            json!({ "source": WATCHER, "worldName": WORLD }),
        )
        .await?;
        let tree = self
            .call_in(session, "Page.getFrameTree", json!({}))
            .await?;
        let frame = tree
            .pointer("/frameTree/frame/id")
            .and_then(Value::as_str)
            .ok_or_else(|| CdpError::Unexpected("the page has no main frame".to_string()))?;
        let world = self
            .call_in(
                session,
                "Page.createIsolatedWorld",
                json!({ "frameId": frame, "worldName": WORLD }),
            )
            .await?;
        let context = world
            .get("executionContextId")
            .and_then(Value::as_i64)
            .ok_or_else(|| CdpError::Unexpected("the title world has no context".to_string()))?;
        self.call_in(
            session,
            "Runtime.evaluate",
            json!({ "expression": WATCHER, "contextId": context }),
        )
        .await
        .map(drop)
    }
}
