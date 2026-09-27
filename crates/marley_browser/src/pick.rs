//! Picking an element in the page (#496, pillar A).
//!
//! Chromium's inspect mode draws its highlight into the frames and reports the element the user
//! clicks. The pick is then read in one pass into a [`PickBundle`] that outlives the page's
//! changes: the element walked up to its nearest interactive ancestor, its locators, its role
//! and name, its listeners with their script locations, what blocks a click on it, its box, and
//! a crop of the page around it. Since #518 the pass also reads the element's HTML, its computed
//! styles, its siblings' texts, the page's selection and, on a React dev build, the components
//! around it and where it was written.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::cdp::CdpError;
use crate::observe::SECRET_NAMES;
use crate::page::Page;

/// The most listeners a bundle lists, the element's first.
const LISTENER_CAP: usize = 24;

/// The space kept around the element in its crop, in CSS pixels.
const CROP_MARGIN: f64 = 16.0;

/// What marks a text [`within`] or a trip cap cut.
const CUT: &str = " (truncated)";

/// The most of the HTML the page's read hands back (#518). The trip caps only bound what crosses
/// the socket: a text keeps them until the tool that gives it to an agent has redacted it whole
/// and cut it to its budget.
const HTML_TRIP: usize = 65_536;
/// The most of each other text the read hands back: the element's, a sibling's, the selection.
const TEXT_TRIP: usize = 4_096;
/// The most of a computed style's value kept.
const STYLE_TRIP: usize = 500;
/// The most of a component's name kept.
const NAME_TRIP: usize = 200;
/// The most of a debug source's file name kept.
const SOURCE_TRIP: usize = 500;
/// The most of a debug stack kept.
const STACK_TRIP: usize = 4_000;

/// How many computed styles a bundle keeps.
const STYLE_COUNT: usize = 16;
/// How many sibling texts a bundle keeps.
const NEARBY_COUNT: usize = 10;
/// How many component names a bundle keeps.
const CHAIN_COUNT: usize = 6;

/// The characters of a pick's HTML an agent gets (#518, Orca's budget).
pub const HTML_BUDGET: usize = 4_096;
/// The characters of the element's text, and of each sibling's, an agent gets.
pub const TEXT_BUDGET: usize = 200;
/// The characters of the page's selection an agent gets.
pub const SELECTION_BUDGET: usize = 500;

/// The functions of React's own that its debug stacks start with.
const REACT_FRAMES: &[&str] = &[
    "jsxDEV",
    "jsx",
    "jsxs",
    "createElement",
    "react_stack_bottom_frame",
    "react-stack-bottom-frame",
];

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

/// The function, run on the element in the page's main world, that reads what the bundle needs of
/// it: its tag and text, its locators and whether each finds it alone, its box in the page through
/// same-origin frames, and what would block a click on it. A field's value is what the user
/// typed, a password's included, so the text is a button's label only.
///
/// Since #518 it also reads the element's HTML, from a clone that loses its scripts, its fields'
/// values, each attribute whose name holds one of `secretNames` or whose value holds a secret
/// pattern, and its URLs' queries and fragments; sixteen computed styles; its siblings' texts; the
/// page's selection, unless a field has the focus; and React's fiber, which only the main world
/// sees. Each text comes cut to its trip cap, never to its budget, which [`within`] applies after
/// redaction.
const DESCRIBE: &str = r#"function (secretNames) {
  const element = this;
  const document = element.ownerDocument;
  const cap = (value, limit) => {
    const whole = value.toWellFormed ? value.toWellFormed() : value;
    if (whole.length <= limit) return whole;
    const code = whole.charCodeAt(limit - 1);
    return whole.slice(0, code >= 0xd800 && code <= 0xdbff ? limit - 1 : limit) + ' (truncated)';
  };
  const read = (fallback, reader) => { try { return reader(); } catch (error) { return fallback; } };
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
  const html = read('', () => {
    const secretValues = ['access_token', 'auth_token', 'api_key', 'apikey', 'client_secret', 'oauth_state',
      'x-amz-', 'session_id', 'sessionid', 'csrf', 'secret', 'password', 'passwd'];
    // These name things rather than hold data: `type="password"` is no secret.
    const keywords = new Set(['type', 'autocomplete', 'inputmode', 'role', 'id', 'name', 'for', 'class']);
    const addresses = new Set(['href', 'src', 'action', 'formaction', 'poster', 'cite', 'data', 'ping', 'xlink:href']);
    const buttons = new Set(['button', 'submit', 'reset', 'image']);
    const bare = (value) => {
      const trimmed = value.trim();
      let url;
      try { url = new URL(trimmed, document.baseURI); } catch (error) { return '[redacted]'; }
      if (url.protocol === 'data:') return 'data:…';
      if (!['http:', 'https:', 'file:', 'about:', 'mailto:', 'tel:'].includes(url.protocol)) return '[redacted]';
      if (url.username || url.password) {
        url.username = '';
        url.password = '';
        url.search = '';
        url.hash = '';
        return url.href;
      }
      const end = trimmed.search(/[?#]/);
      return end === -1 ? trimmed : trimmed.slice(0, end);
    };
    const clone = element.cloneNode(true);
    for (const script of clone.querySelectorAll('script')) script.remove();
    // Each element takes four characters at least (`<br>`), so none past these can start within
    // the trip cap.
    const walker = document.createTreeWalker(clone, NodeFilter.SHOW_ELEMENT);
    for (let node = clone, seen = 0; node && seen < 20000; node = walker.nextNode(), seen += 1) {
      if (node.localName === 'input' && !buttons.has((node.getAttribute('type') || '').toLowerCase())) {
        node.removeAttribute('value');
      }
      if (node.localName === 'textarea') node.textContent = '';
      for (const attribute of [...node.attributes]) {
        const attributeName = attribute.name.toLowerCase();
        const value = attribute.value.toLowerCase();
        if (secretNames.some((secret) => attributeName.includes(secret))
          || (!keywords.has(attributeName) && secretValues.some((secret) => value.includes(secret)))) {
          node.setAttribute(attribute.name, '[redacted]');
        } else if (addresses.has(attributeName)) {
          node.setAttribute(attribute.name, bare(attribute.value));
        } else if (attributeName === 'srcset' || attributeName === 'imagesrcset') {
          node.setAttribute(attribute.name, attribute.value.split(',').map((candidate) => {
            const [address, ...descriptor] = candidate.trim().split(/\s+/);
            return [bare(address || ''), ...descriptor].join(' ');
          }).join(', '));
        }
      }
    }
    return cap(clone.outerHTML || '', 65536);
  });
  const styles = read({}, () => Object.fromEntries(['display', 'position', 'width', 'height', 'margin', 'padding',
    'color', 'background-color', 'border', 'border-radius', 'font-family', 'font-size', 'font-weight',
    'line-height', 'text-align', 'z-index'].map((property) => [property, cap(String(style.getPropertyValue(property) || ''), 500)])));
  const textOf = (root) => {
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, { acceptNode: (node) =>
      node.parentElement && node.parentElement.closest('script, style, template, noscript, textarea')
        ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT });
    let collected = '';
    for (let node = walker.nextNode(), seen = 0; node && collected.length <= 4096 && seen < 400; node = walker.nextNode(), seen += 1) {
      collected += ' ' + node.nodeValue.slice(0, 4097 - collected.length);
    }
    return cap(collected.trim().replace(/\s+/g, ' '), 4096);
  };
  const nearby = read([], () => {
    const texts = [];
    let previous = element.previousElementSibling;
    let next = element.nextElementSibling;
    for (let seen = 0; texts.length < 10 && seen < 80 && (previous || next);) {
      for (const sibling of [previous, next]) {
        if (!sibling || texts.length >= 10) continue;
        seen += 1;
        const siblingText = textOf(sibling);
        if (siblingText) texts.push(siblingText);
      }
      previous = previous && previous.previousElementSibling;
      next = next && next.nextElementSibling;
    }
    return texts;
  });
  // A selection inside a field is what the user typed there.
  const selected = read(null, () => {
    const active = document.activeElement;
    if (active && (active.localName === 'input' || active.localName === 'textarea')) return null;
    const selection = document.getSelection();
    const selectedText = selection ? selection.toString().trim() : '';
    return selectedText ? cap(selectedText, 4096) : null;
  });
  const react = read(null, () => {
    const key = Object.keys(element).find((property) => property.startsWith('__reactFiber$')
      || property.startsWith('__reactInternalInstance$'));
    const host = key ? element[key] : null;
    if (!host || typeof host !== 'object') return null;
    const skipped = (component) => component.length <= 2
      || /^(Fragment|Root|Routes|Route|Outlet|Provider|Consumer|Profiler|Suspense)$/.test(component)
      || /(Boundary|BoundaryHandler|Router|Provider|Consumer|Context|Wrapper)$/.test(component)
      || /^(Inner|Outer|Client|Server|RSC|Dev|React|Hot)/.test(component);
    const named = (type) => {
      if (!type || typeof type === 'string') return null;
      for (const candidate of [type, type.render, type.type]) {
        const component = candidate && (candidate.displayName || candidate.name);
        if (typeof component === 'string' && component) return component;
      }
      return null;
    };
    const chain = [];
    let source = null;
    let stack = null;
    let fiber = host;
    for (let depth = 0; fiber && depth < 35; depth += 1, fiber = fiber.return) {
      const component = named(fiber.type || fiber.elementType);
      if (component && !skipped(component) && !chain.includes(component) && chain.length < 6) {
        chain.push(cap(component, 200));
      }
      const debug = fiber._debugSource || (fiber._debugOwner && fiber._debugOwner._debugSource);
      if (!source && debug && typeof debug.fileName === 'string' && Number.isFinite(debug.lineNumber)) {
        source = { file: cap(debug.fileName, 500), line: debug.lineNumber,
          column: Number.isFinite(debug.columnNumber) ? debug.columnNumber : null };
      }
      if (!stack && fiber._debugStack && typeof fiber._debugStack.stack === 'string') {
        stack = cap(fiber._debugStack.stack, 4000);
      }
    }
    return { chain: chain.reverse(), source, stack: source ? null : stack };
  });
  return { tag: element.localName, text: cap(text, 4096), locators,
    box: { x: left + scrollX, y: top + scrollY, width: box.width, height: box.height }, blockers,
    html, styles, nearby_text: nearby, selected_text: selected, react };
}"#;

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
    /// Its line in the script, from 1.
    pub line: u32,
    /// Its column in the line, from 1.
    pub column: u32,
    /// The script's source map, as the script names it.
    pub source_map: Option<String>,
    /// Where its source map says it was written (#497), once the map is read.
    pub original: Option<SourcePosition>,
}

/// A listener's place in its original source (#497).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourcePosition {
    /// The source, as its map names it, resolved to a URL.
    pub source: String,
    /// The file in the user's project that holds it, relative to its worktree, when one does.
    pub file: Option<String>,
    /// Its line, from 1.
    pub line: u32,
    /// Its column, from 1.
    pub column: u32,
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
    /// Its text, a button's label only; agents get [`TEXT_BUDGET`] characters of it.
    pub text: String,
    /// Ways to find it again, the most durable first.
    pub locators: Vec<Locator>,
    /// The listeners on it and its ancestors.
    pub listeners: Vec<Listener>,
    /// What would block a click on it.
    pub blockers: Vec<String>,
    /// Its box in the page.
    pub page_box: PageBox,
    /// Its HTML, without scripts, field values, secret-looking attribute values or URL queries
    /// (#518); agents get [`HTML_BUDGET`] characters of it.
    pub html: String,
    /// Sixteen of its computed styles, by their CSS names.
    pub styles: BTreeMap<String, String>,
    /// Its siblings' texts, the nearest first, before and after in turn.
    pub nearby_text: Vec<String>,
    /// The page's selection at the pick, unless it lay in a field.
    pub selected_text: Option<String>,
    /// The React component around it, on a React dev build.
    pub component: Option<Component>,
}

/// The React components around a picked element, and where it was written (#518).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Component {
    /// The components around it, the outermost first, at most six.
    pub chain: Vec<String>,
    /// Where it was written, once known: React's debug source at once, React 19's debug stack
    /// once the frames' source maps are read.
    pub source: Option<ComponentSource>,
    /// React 19's debug stack, for the workbench to map through the scripts' source maps.
    #[serde(skip)]
    pub frames: Vec<StackFrame>,
}

/// Where React says a picked element was written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComponentSource {
    /// Which of React's fields said it.
    pub from: SourceKind,
    /// The source as React or the source map names it.
    pub source: String,
    /// The file in the user's project that holds it, relative to its worktree, when one does.
    pub file: Option<String>,
    /// Its line, from 1.
    pub line: u32,
    /// Its column: from 1 for a debug stack, as the JSX transform wrote it for a debug source.
    pub column: Option<u32>,
}

/// Which of React's fields names an element's source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SourceKind {
    /// `_debugSource`, which React 18 and older copy from the JSX transform's `__source`.
    #[serde(rename = "debug source")]
    DebugSource,
    /// React 19's `_debugStack`, mapped through its scripts' source maps.
    #[serde(rename = "debug stack")]
    DebugStack,
}

/// One frame of a V8 stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackFrame {
    /// The function, when V8 names one.
    pub function: Option<String>,
    /// The script's URL.
    pub url: String,
    /// Its line, from 1.
    pub line: u32,
    /// Its column, from 1.
    pub column: u32,
}

impl StackFrame {
    /// Whether the frame is one of React's own functions, which its debug stacks start with.
    #[must_use]
    pub fn is_reacts(&self) -> bool {
        self.function.as_deref().is_some_and(|function| {
            let last = function.rsplit('.').next().unwrap_or(function);
            REACT_FRAMES.contains(&last)
        })
    }
}

/// A script the page loaded, as `Debugger.scriptParsed` names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptInfo {
    /// Its URL; empty for a script with none.
    pub url: String,
    /// Its source map's URL, as the script names it.
    pub source_map: Option<String>,
}

/// What [`DESCRIBE`] answers. A read that fails in the page gives its field's empty value, so the
/// pick still comes; the React part is read from its JSON by hand, since the page's own objects
/// fill it.
#[derive(Deserialize)]
struct Described {
    tag: String,
    text: String,
    locators: Vec<Locator>,
    #[serde(rename = "box")]
    page_box: PageBox,
    blockers: Vec<String>,
    #[serde(default)]
    html: String,
    #[serde(default)]
    styles: BTreeMap<String, String>,
    #[serde(default)]
    nearby_text: Vec<String>,
    #[serde(default)]
    selected_text: Option<String>,
    #[serde(default)]
    react: Option<Value>,
}

/// `text` cut to `budget` characters, with the cut marked.
#[must_use]
pub fn within(text: &str, budget: usize) -> String {
    match text.char_indices().nth(budget) {
        Some((end, _)) => format!("{}{CUT}", text.get(..end).unwrap_or(text)),
        None => text.to_string(),
    }
}

/// `text` held to its trip cap on arrival. The page's read cuts at the cap and marks the cut, so
/// only a page that rewrote the read sends more.
fn trip(text: &str, cap: usize) -> String {
    if text.chars().count() > cap + CUT.chars().count() {
        within(text, cap)
    } else {
        text.to_string()
    }
}

/// The frames of a V8 stack, `at <function> (<url>:<line>:<column>)` or `at <url>:<line>:<column>`
/// a line; its first line, the error's message, and any other line are passed over.
#[must_use]
pub fn stack_frames(stack: &str) -> Vec<StackFrame> {
    stack
        .lines()
        .filter_map(|line| {
            let frame = line.trim().strip_prefix("at ")?;
            let (function, place) = match frame.strip_suffix(')') {
                Some(called) => {
                    let (function, place) = called.rsplit_once(" (")?;
                    (Some(function.to_string()), place)
                }
                None => (None, frame),
            };
            let (rest, column) = place.rsplit_once(':')?;
            let (url, line) = rest.rsplit_once(':')?;
            Some(StackFrame {
                function,
                url: url.to_string(),
                line: line.parse().ok()?,
                column: column.parse().ok()?,
            })
        })
        .collect()
}

/// The React part of [`DESCRIBE`]'s answer as a [`Component`], when the element has a fiber.
fn component_of(react: &Value) -> Option<Component> {
    let chain: Vec<String> = react
        .get("chain")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .take(CHAIN_COUNT)
        .map(|component| trip(component, NAME_TRIP))
        .collect();
    let source = react.get("source").and_then(|source| {
        Some(ComponentSource {
            from: SourceKind::DebugSource,
            source: trip(source.get("file")?.as_str()?, SOURCE_TRIP),
            file: None,
            line: u32::try_from(source.get("line")?.as_u64()?).ok()?,
            column: source
                .get("column")
                .and_then(Value::as_u64)
                .and_then(|column| u32::try_from(column).ok()),
        })
    });
    let frames = react
        .get("stack")
        .and_then(Value::as_str)
        .map(|stack| stack_frames(&trip(stack, STACK_TRIP)))
        .unwrap_or_default();
    (!chain.is_empty() || source.is_some() || !frames.is_empty()).then_some(Component {
        chain,
        source,
        frames,
    })
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
        let described = self
            .call_on_with(&element, DESCRIBE, true, &[json!(SECRET_NAMES)])
            .await?;
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
            text: trip(&described.text, TEXT_TRIP),
            locators: described.locators,
            listeners,
            blockers: described.blockers,
            page_box: described.page_box,
            html: trip(&described.html, HTML_TRIP),
            styles: described
                .styles
                .into_iter()
                .take(STYLE_COUNT)
                .map(|(property, value)| (property, trip(&value, STYLE_TRIP)))
                .collect(),
            nearby_text: described
                .nearby_text
                .iter()
                .take(NEARBY_COUNT)
                .map(|text| trip(text, TEXT_TRIP))
                .collect(),
            selected_text: described
                .selected_text
                .as_deref()
                .map(|text| trip(text, TEXT_TRIP)),
            component: described.react.as_ref().and_then(component_of),
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
        self.call_on_with(object, function, by_value, &[]).await
    }

    /// Calls `function` on the object `object` with `arguments`, each passed by value.
    async fn call_on_with(
        &self,
        object: &str,
        function: &str,
        by_value: bool,
        arguments: &[Value],
    ) -> Result<Value, CdpError> {
        let arguments: Vec<Value> = arguments
            .iter()
            .map(|value| json!({ "value": value }))
            .collect();
        let answer = self
            .call(
                "Runtime.callFunctionOn",
                json!({
                    "objectId": object,
                    "functionDeclaration": function,
                    "arguments": arguments,
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
                    line: from_one(listener.get("lineNumber")),
                    column: from_one(listener.get("columnNumber")),
                    source_map: script.and_then(|script| script.source_map.clone()),
                    original: None,
                });
            }
        }
        Ok(listeners)
    }
}

/// CDP's count from 0 as a count from 1, as an editor shows it.
fn from_one(value: Option<&Value>) -> u32 {
    value
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .map_or(1, |value| value.saturating_add(1))
}

/// The object id at `pointer` in `answer`.
fn object_id(answer: &Value, pointer: &str) -> Result<String, CdpError> {
    answer
        .pointer(pointer)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| CdpError::Unexpected("the picked element is gone".to_string()))
}
