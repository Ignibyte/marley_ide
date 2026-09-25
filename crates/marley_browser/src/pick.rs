//! Picking an element in the page (#496, pillar A).
//!
//! Chromium's inspect mode draws its highlight into the frames and reports the element the user
//! clicks. The pick is then read in one pass into a [`PickBundle`] that outlives the page's
//! changes: the element walked up to its nearest interactive ancestor, its locators, its role
//! and name, its listeners with their script locations, what blocks a click on it, its box, and
//! a crop of the page around it.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::cdp::CdpError;
use crate::page::Page;

/// The most listeners a bundle lists, the element's first.
const LISTENER_CAP: usize = 24;

/// The space kept around the element in its crop, in CSS pixels.
const CROP_MARGIN: f64 = 16.0;

/// The function, run on the picked node, that gives its nearest interactive ancestor, the node's
/// own element when none is, through open shadow roots.
const INTERACTIVE_ANCESTOR: &str = r#"function () {
  const roles = new Set(['button', 'link', 'checkbox', 'radio', 'tab', 'menuitem', 'option', 'switch',
    'textbox', 'combobox', 'slider', 'treeitem', 'gridcell', 'searchbox', 'spinbutton']);
  const interactive = (element) => element.matches('a[href], button, input, select, textarea, summary, label, '
    + '[tabindex]:not([tabindex="-1"]), [contenteditable=""], [contenteditable="true"], [onclick]')
    || roles.has(element.getAttribute('role'));
  const up = (node) => node.parentElement || (node.parentNode instanceof ShadowRoot ? node.parentNode.host : null);
  const start = this.nodeType === Node.ELEMENT_NODE ? this : this.parentElement;
  for (let element = start; element; element = up(element)) {
    if (interactive(element)) return element;
  }
  return start;
}"#;

/// The function, run on the element, that reads what the bundle needs of it: its tag and text,
/// its locators and whether each finds it alone, its box in the page through same-origin frames,
/// and what would block a click on it. A field's value is what the user typed, a password's
/// included, so the text is a button's label only.
const DESCRIBE: &str = r"function () {
  const element = this;
  const document = element.ownerDocument;
  const labelled = element.localName === 'input' && ['button', 'submit', 'reset'].includes(element.type);
  const name = (node) => node.localName + (node.id ? '#' + node.id : '')
    + [...node.classList].slice(0, 2).map((token) => '.' + token).join('');
  const unique = (selector) => { try { return document.querySelectorAll(selector).length === 1; } catch (error) { return false; } };
  const locators = [];
  for (const attribute of ['data-testid', 'data-test', 'data-cy', 'data-qa']) {
    const value = element.getAttribute(attribute);
    if (value) {
      const selector = '[' + attribute + '=' + JSON.stringify(value) + ']';
      locators.push({ kind: 'test id', value: selector, unique: unique(selector) });
      break;
    }
  }
  if (element.id) {
    const selector = '#' + CSS.escape(element.id);
    locators.push({ kind: 'id', value: selector, unique: unique(selector) });
  }
  const text = (element.innerText || (labelled ? element.value : '') || '').trim().replace(/\s+/g, ' ');
  if (text && text.length <= 80) locators.push({ kind: 'text', value: text, unique: null });
  const path = [];
  for (let node = element; node && node.nodeType === Node.ELEMENT_NODE && path.length < 8; node = node.parentElement) {
    if (node.id) { path.unshift('#' + CSS.escape(node.id)); break; }
    const siblings = node.parentElement ? [...node.parentElement.children].filter((child) => child.localName === node.localName) : [];
    path.unshift(siblings.length > 1 ? node.localName + ':nth-of-type(' + (siblings.indexOf(node) + 1) + ')' : node.localName);
  }
  const css = path.join(' > ');
  locators.push({ kind: 'css', value: css, unique: unique(css) });
  const box = element.getBoundingClientRect();
  let left = box.left;
  let top = box.top;
  let view = document.defaultView;
  while (view && view !== view.top && view.frameElement) {
    const frame = view.frameElement;
    const frameBox = frame.getBoundingClientRect();
    left += frameBox.left + frame.clientLeft;
    top += frameBox.top + frame.clientTop;
    view = view.parent;
  }
  let scrollX = 0;
  let scrollY = 0;
  try { scrollX = view.scrollX; scrollY = view.scrollY; } catch (error) {}
  const blockers = [];
  const style = getComputedStyle(element);
  for (let node = element; node; node = node.parentElement) {
    if (getComputedStyle(node).pointerEvents === 'none') { blockers.push('pointer-events: none on ' + name(node)); break; }
  }
  if (style.visibility !== 'visible') blockers.push('visibility: ' + style.visibility);
  if (style.display === 'none') blockers.push('display: none');
  if (Number(style.opacity) === 0) blockers.push('opacity: 0');
  if (element.disabled || element.getAttribute('aria-disabled') === 'true') blockers.push('disabled');
  const onTop = document.elementFromPoint(box.left + box.width / 2, box.top + box.height / 2);
  if (onTop && onTop !== element && !element.contains(onTop)) blockers.push('covered by ' + name(onTop));
  return { tag: element.localName, text: text.slice(0, 200), locators,
    box: { x: left + scrollX, y: top + scrollY, width: box.width, height: box.height }, blockers };
}";

/// The function, run on the element, that gives it, its ancestors through open shadow roots, the
/// document and the window: where its listeners can sit.
const LISTENER_HOSTS: &str = r"function () {
  const hosts = [];
  for (let node = this; node; node = node.parentElement || (node.parentNode instanceof ShadowRoot ? node.parentNode.host : null)) {
    hosts.push(node);
  }
  hosts.push(this.ownerDocument, this.ownerDocument.defaultView);
  return hosts;
}";

/// A way to find the picked element again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locator {
    /// What it goes by: `test id`, `id`, `text` or `css`.
    pub kind: String,
    /// The selector or the text.
    pub value: String,
    /// Whether it finds this element alone in its document; unknown for text.
    pub unique: Option<bool>,
}

/// A listener on the element or one of its ancestors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Listener {
    /// The event it listens for.
    pub event: String,
    /// What it sits on: the element, an ancestor, the document or the window.
    pub on: String,
    /// The URL of the script that added it.
    pub script: Option<String>,
    /// Its line in the script, from 0.
    pub line: i64,
    /// Its column in the line, from 0.
    pub column: i64,
    /// The script's source map, as the script names it (#497).
    pub source_map: Option<String>,
}

/// The element's box in the page, in CSS pixels: the document's, not the viewport's.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PageBox {
    /// Its left edge.
    pub x: f64,
    /// Its top edge.
    pub y: f64,
    /// Its width.
    pub width: f64,
    /// Its height.
    pub height: f64,
}

/// What a pick captured of its element, at the moment of the pick.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PickBundle {
    /// The element's tag.
    pub tag: String,
    /// Its accessible role.
    pub role: Option<String>,
    /// Its accessible name.
    pub name: Option<String>,
    /// Its text, at most 200 characters.
    pub text: String,
    /// Ways to find it again, the most durable first.
    pub locators: Vec<Locator>,
    /// The listeners on it and its ancestors.
    pub listeners: Vec<Listener>,
    /// What would block a click on it.
    pub blockers: Vec<String>,
    /// Its box in the page.
    pub page_box: PageBox,
}

/// A script the page loaded, as `Debugger.scriptParsed` names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptInfo {
    /// Its URL; empty for a script with none.
    pub url: String,
    /// Its source map's URL, as the script names it.
    pub source_map: Option<String>,
}

/// What [`DESCRIBE`] answers.
#[derive(Deserialize)]
struct Described {
    tag: String,
    text: String,
    locators: Vec<Locator>,
    #[serde(rename = "box")]
    page_box: PageBox,
    blockers: Vec<String>,
}

impl PickBundle {
    /// What the tray and the agent's line call the pick: its role and name, else its tag and text.
    #[must_use]
    pub fn summary(&self) -> String {
        let what = self.role.as_deref().unwrap_or(&self.tag);
        let label = self
            .name
            .as_deref()
            .filter(|name| !name.is_empty())
            .unwrap_or(&self.text);
        let label: String = label.chars().take(60).collect();
        if label.is_empty() {
            what.to_string()
        } else {
            format!("{what} \u{201c}{label}\u{201d}")
        }
    }
}

impl Page {
    /// Turns Chromium's inspect mode on in the page, its highlight and tooltip following the
    /// pointer, or off.
    ///
    /// # Errors
    ///
    /// When a call fails.
    pub async fn set_inspect(&self, on: bool) -> Result<(), CdpError> {
        if on {
            self.call("DOM.enable", json!({})).await?;
            self.call("Overlay.enable", json!({})).await?;
            self.call(
                "Overlay.setInspectMode",
                json!({
                    "mode": "searchForNode",
                    "highlightConfig": {
                        "showInfo": true,
                        "showAccessibilityInfo": true,
                        "contentColor": { "r": 111, "g": 168, "b": 220, "a": 0.55 },
                        "paddingColor": { "r": 147, "g": 196, "b": 125, "a": 0.45 },
                        "borderColor": { "r": 255, "g": 229, "b": 153, "a": 0.66 },
                        "marginColor": { "r": 246, "g": 178, "b": 107, "a": 0.55 },
                    },
                }),
            )
            .await
            .map(drop)
        } else {
            self.call(
                "Overlay.setInspectMode",
                json!({ "mode": "none", "highlightConfig": {} }),
            )
            .await
            .map(drop)
        }
    }

    /// Turns on the page's Debugger domain, which names each script the page loads, with every
    /// pause skipped, so a `debugger` statement in the page never stops it.
    ///
    /// # Errors
    ///
    /// When a call fails.
    pub async fn watch_scripts(&self) -> Result<(), CdpError> {
        self.call("Debugger.enable", json!({})).await?;
        self.call("Debugger.setSkipAllPauses", json!({ "skip": true }))
            .await
            .map(drop)
    }

    /// Reads the element the user picked, `backend_node_id`, into a bundle, with `scripts` naming
    /// the page's scripts.
    ///
    /// # Errors
    ///
    /// When a call fails or an answer is not what the reads expect, as when the element went.
    pub async fn capture_pick(
        &self,
        backend_node_id: i64,
        scripts: &HashMap<String, ScriptInfo>,
    ) -> Result<PickBundle, CdpError> {
        let node = self
            .call(
                "DOM.resolveNode",
                json!({ "backendNodeId": backend_node_id }),
            )
            .await?;
        let node = object_id(&node, "/object/objectId")?;
        let element = self.call_on(&node, INTERACTIVE_ANCESTOR, false).await?;
        let element = object_id(&element, "/result/objectId")?;
        let described = self.call_on(&element, DESCRIBE, true).await?;
        let described: Described = serde_json::from_value(
            described
                .pointer("/result/value")
                .cloned()
                .unwrap_or_default(),
        )
        .map_err(|error| CdpError::Unexpected(format!("the picked element's reading: {error}")))?;
        let tree = self
            .call(
                "Accessibility.getPartialAXTree",
                json!({ "objectId": element, "fetchRelatives": false }),
            )
            .await?;
        let role = tree
            .pointer("/nodes/0/role/value")
            .and_then(Value::as_str)
            .map(str::to_string);
        let name = tree
            .pointer("/nodes/0/name/value")
            .and_then(Value::as_str)
            .map(|name| name.trim().to_string());
        let listeners = self.listeners(&element, scripts).await?;
        Ok(PickBundle {
            tag: described.tag,
            role,
            name,
            text: described.text,
            locators: described.locators,
            listeners,
            blockers: described.blockers,
            page_box: described.page_box,
        })
    }

    /// The page around `page_box`, as a base64 JPEG.
    ///
    /// # Errors
    ///
    /// When the call fails or its answer holds no image.
    pub async fn crop(&self, page_box: PageBox) -> Result<String, CdpError> {
        let answer = self
            .call(
                "Page.captureScreenshot",
                json!({
                    "format": "jpeg",
                    "quality": 80,
                    "clip": {
                        "x": (page_box.x - CROP_MARGIN).max(0.0),
                        "y": (page_box.y - CROP_MARGIN).max(0.0),
                        "width": 2.0f64.mul_add(CROP_MARGIN, page_box.width),
                        "height": 2.0f64.mul_add(CROP_MARGIN, page_box.height),
                        "scale": 1,
                    },
                }),
            )
            .await?;
        answer
            .get("data")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| CdpError::Unexpected("the crop holds no image".to_string()))
    }

    /// Calls `function` on the object `object`, answering by value or with an object.
    async fn call_on(
        &self,
        object: &str,
        function: &str,
        by_value: bool,
    ) -> Result<Value, CdpError> {
        let answer = self
            .call(
                "Runtime.callFunctionOn",
                json!({
                    "objectId": object,
                    "functionDeclaration": function,
                    "returnByValue": by_value,
                }),
            )
            .await?;
        if let Some(exception) = answer.get("exceptionDetails") {
            return Err(CdpError::Unexpected(format!(
                "reading the picked element threw: {exception}"
            )));
        }
        Ok(answer)
    }

    /// The listeners on `element` and where they can sit above it, at most [`LISTENER_CAP`].
    async fn listeners(
        &self,
        element: &str,
        scripts: &HashMap<String, ScriptInfo>,
    ) -> Result<Vec<Listener>, CdpError> {
        let hosts = self.call_on(element, LISTENER_HOSTS, false).await?;
        let hosts = object_id(&hosts, "/result/objectId")?;
        let properties = self
            .call(
                "Runtime.getProperties",
                json!({ "objectId": hosts, "ownProperties": true }),
            )
            .await?;
        let mut listeners = Vec::new();
        for property in properties
            .get("result")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let is_index = property
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name.bytes().all(|byte| byte.is_ascii_digit()));
            let (true, Some(host), Some(on)) = (
                is_index,
                property.pointer("/value/objectId").and_then(Value::as_str),
                property
                    .pointer("/value/description")
                    .and_then(Value::as_str),
            ) else {
                continue;
            };
            let found = self
                .call("DOMDebugger.getEventListeners", json!({ "objectId": host }))
                .await?;
            for listener in found
                .get("listeners")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if listeners.len() == LISTENER_CAP {
                    return Ok(listeners);
                }
                let script = listener
                    .get("scriptId")
                    .and_then(Value::as_str)
                    .and_then(|id| scripts.get(id));
                listeners.push(Listener {
                    event: listener
                        .get("type")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    on: on.to_string(),
                    script: script
                        .map(|script| script.url.clone())
                        .filter(|url| !url.is_empty()),
                    line: listener
                        .get("lineNumber")
                        .and_then(Value::as_i64)
                        .unwrap_or(0),
                    column: listener
                        .get("columnNumber")
                        .and_then(Value::as_i64)
                        .unwrap_or(0),
                    source_map: script.and_then(|script| script.source_map.clone()),
                });
            }
        }
        Ok(listeners)
    }
}

/// The object id at `pointer` in `answer`.
fn object_id(answer: &Value, pointer: &str) -> Result<String, CdpError> {
    answer
        .pointer(pointer)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| CdpError::Unexpected("the picked element is gone".to_string()))
}
