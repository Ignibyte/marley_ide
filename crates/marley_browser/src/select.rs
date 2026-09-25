//! Marley's own `<select>` lists (#495).
//!
//! Headless Chromium opens a select's popup where no frame shows it, so a listener in an
//! isolated world of each frame takes the press that would open one: it keeps the popup shut,
//! gives the select the focus, and reports the select through a CDP binding. Marley draws the
//! options, and sets the choice in the same world, which then dispatches the `input` and
//! `change` a choice in Chromium's own popup would.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::cdp::CdpError;
use crate::page::Page;

/// The isolated world the listener runs in, apart from the page's own scripts and from the world
/// the agent tools read in.
pub const WORLD: &str = "marley-select";

/// The binding the listener reports through: a function in the world whose call Chromium sends
/// as `Runtime.bindingCalled`.
pub const BINDING: &str = "marleySelect";

/// The listener, which a world runs once however many times it is evaluated there.
///
/// It takes a primary-button `mousedown` on a select, and Alt+Down, Alt+Up, F4 or Space on a
/// focused one, for a select that is not `multiple`, sized as a list or disabled. The page's own
/// listeners still get the event.
pub const LISTENER: &str = r"(() => {
  if (globalThis.__marleySelectListener) return;
  globalThis.__marleySelectListener = true;
  let open = null;
  const opens = (select) => select instanceof HTMLSelectElement && !select.multiple
    && select.size <= 1 && !select.disabled;
  const report = (select, event) => {
    open = select;
    const box = select.getBoundingClientRect();
    const press = event instanceof MouseEvent;
    marleySelect(JSON.stringify({
      left: box.left, top: box.top, width: box.width, height: box.height,
      pressX: press ? event.clientX : null, pressY: press ? event.clientY : null,
      selected: select.selectedIndex >= 0 ? select.selectedIndex : null,
      options: Array.from(select.options, (option) => {
        const group = option.parentElement instanceof HTMLOptGroupElement ? option.parentElement : null;
        return { text: option.label, disabled: option.disabled || Boolean(group && group.disabled),
          group: group ? group.label : null };
      }),
    }));
  };
  addEventListener('mousedown', (event) => {
    const select = event.target instanceof Element ? event.target.closest('select') : null;
    if (event.button !== 0 || !opens(select)) return;
    event.preventDefault();
    select.focus();
    report(select, event);
  }, true);
  addEventListener('keydown', (event) => {
    const key = event.key;
    const opening = (event.altKey && (key === 'ArrowDown' || key === 'ArrowUp')) || key === 'F4' || key === ' ';
    if (!opening || !opens(event.target)) return;
    event.preventDefault();
    report(event.target, event);
  }, true);
  globalThis.__marleyChoose = (index) => {
    const select = open;
    const option = select && select.isConnected ? select.options[index] : null;
    if (!option || option.disabled) return false;
    if (select.selectedIndex !== index) {
      select.selectedIndex = index;
      select.dispatchEvent(new Event('input', { bubbles: true }));
      select.dispatchEvent(new Event('change', { bubbles: true }));
    }
    return true;
  };
})()";

/// A select the listener caught opening, as its report gives it: the select's box and the press
/// in its frame's viewport, in CSS pixels, and its options.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectRequest {
    /// The select's left edge.
    pub left: f64,
    /// Its top edge.
    pub top: f64,
    /// Its width.
    pub width: f64,
    /// Its height.
    pub height: f64,
    /// Where the press that opened it landed; none for a key.
    pub press_x: Option<f64>,
    /// The press's height.
    pub press_y: Option<f64>,
    /// The option selected now.
    pub selected: Option<usize>,
    /// The options, in the page's order.
    pub options: Vec<SelectOption>,
}

/// One option of a select.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SelectOption {
    /// What the option shows.
    pub text: String,
    /// Whether it cannot be chosen, itself or through its group.
    pub disabled: bool,
    /// The label of the group it is in.
    pub group: Option<String>,
}

impl SelectRequest {
    /// Reads a report the listener sent.
    ///
    /// # Errors
    ///
    /// When the report is not a select's.
    pub fn parse(payload: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(payload)
    }
}

impl Page {
    /// Watches the selects of `session`, the page's or a cross-site iframe's: the binding for the
    /// listener's world, the listener in that world of each document the session loads from now
    /// on, and in a new world of each frame it shows already.
    ///
    /// # Errors
    ///
    /// When a call fails or the browser's answer lacks a frame or a context.
    pub async fn watch_selects(&self, session: &str) -> Result<(), CdpError> {
        self.call_in(
            session,
            "Runtime.addBinding",
            json!({ "name": BINDING, "executionContextName": WORLD }),
        )
        .await?;
        self.call_in(
            session,
            "Page.addScriptToEvaluateOnNewDocument",
            json!({ "source": LISTENER, "worldName": WORLD }),
        )
        .await?;
        let tree = self
            .call_in(session, "Page.getFrameTree", json!({}))
            .await?;
        let mut frames = Vec::new();
        frame_ids(tree.get("frameTree"), &mut frames);
        for frame in frames {
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
                .ok_or_else(|| {
                    CdpError::Unexpected("the select world has no context".to_string())
                })?;
            self.call_in(
                session,
                "Runtime.evaluate",
                json!({ "expression": LISTENER, "contextId": context }),
            )
            .await?;
        }
        Ok(())
    }

    /// Chooses option `index` of the select the listener in `context` of `session` reported
    /// last, and says whether it could: the select may have gone, or the option been disabled
    /// since.
    ///
    /// # Errors
    ///
    /// When the call fails.
    pub async fn choose_option(
        &self,
        session: &str,
        context: i64,
        index: usize,
    ) -> Result<bool, CdpError> {
        let answer = self
            .call_in(
                session,
                "Runtime.evaluate",
                json!({
                    "expression": format!("globalThis.__marleyChoose({index})"),
                    "contextId": context,
                    "returnByValue": true,
                }),
            )
            .await?;
        Ok(answer
            .pointer("/result/value")
            .and_then(Value::as_bool)
            .unwrap_or(false))
    }
}

/// The ids of the frames of `tree`, a `Page.getFrameTree` answer's `frameTree`, the main one first.
fn frame_ids(tree: Option<&Value>, ids: &mut Vec<String>) {
    let Some(tree) = tree else {
        return;
    };
    if let Some(id) = tree.pointer("/frame/id").and_then(Value::as_str) {
        ids.push(id.to_string());
    }
    if let Some(children) = tree.get("childFrames").and_then(Value::as_array) {
        for child in children {
            frame_ids(Some(child), ids);
        }
    }
}
