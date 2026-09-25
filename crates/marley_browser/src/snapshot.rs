//! The page's accessibility tree as text an agent reads, with refs its write tools take (#492).
//!
//! Chromium's `Accessibility.getFullAXTree` gives one frame's nodes; the page's frames, and each
//! cross-site iframe's session, give one [`FrameTree`] each. [`render`] writes the interactive
//! nodes of every frame, one a line, each with a ref (`e3`) that names its session and DOM node
//! for [`RefTarget`]; `full` writes every node, indented. No field's value is written: Chromium
//! shows a password field's value as one bullet a character.

use serde::Deserialize;
use serde_json::Value;

use crate::observe::redact_url;

/// The most characters a snapshot holds.
pub const MAX_CHARS: usize = 30_000;

/// The most characters of a name a line keeps.
const MAX_NAME: usize = 100;

/// The roles an agent acts on.
const INTERACTIVE: &[&str] = &[
    "button",
    "checkbox",
    "combobox",
    "link",
    "listbox",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "radio",
    "searchbox",
    "slider",
    "spinbutton",
    "switch",
    "tab",
    "textbox",
    "treeitem",
];

/// Roles that only hold other nodes: without a name, `full` writes their children in their place.
const CONTAINERS: &[&str] = &[
    "generic",
    "none",
    "LabelText",
    "paragraph",
    "group",
    "section",
];

/// One accessibility node, as `Accessibility.getFullAXTree` gives it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AxNode {
    /// The node's id within its tree.
    pub node_id: String,
    /// Whether the tree leaves the node out.
    #[serde(default)]
    pub ignored: bool,
    /// Its role.
    #[serde(default)]
    pub role: Option<AxValue>,
    /// Its accessible name.
    #[serde(default)]
    pub name: Option<AxValue>,
    /// Its properties: states and such.
    #[serde(default)]
    pub properties: Vec<AxProperty>,
    /// Its children's ids.
    #[serde(default)]
    pub child_ids: Vec<String>,
    /// Its parent's id.
    #[serde(default)]
    pub parent_id: Option<String>,
    /// The DOM node behind it.
    #[serde(rename = "backendDOMNodeId", default)]
    pub backend_dom_node_id: Option<i64>,
    /// The frame a document's root node belongs to.
    #[serde(default)]
    pub frame_id: Option<String>,
}

/// An accessibility value: a role, a name, a property's value.
#[derive(Debug, Clone, Deserialize)]
pub struct AxValue {
    /// The value itself.
    #[serde(default)]
    pub value: Option<Value>,
}

/// A named property of a node.
#[derive(Debug, Clone, Deserialize)]
pub struct AxProperty {
    /// Its name: `focusable`, `disabled`, `checked` and so on.
    pub name: String,
    /// Its value.
    pub value: AxValue,
}

/// One frame's nodes, and where they came from.
#[derive(Debug, Clone)]
pub struct FrameTree {
    /// The iframe's session, for a cross-site iframe; none for the page's own frames.
    pub session: Option<String>,
    /// The frame's id: an iframe's, so a ref in it can be placed through its owner element.
    pub frame_id: Option<String>,
    /// What the snapshot calls the frame: none for the main frame, an iframe's URL otherwise.
    pub label: Option<String>,
    /// Its nodes.
    pub nodes: Vec<AxNode>,
}

/// What a ref names: a DOM node in a session, with the role and name the snapshot showed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefTarget {
    /// The ref, `e1` on.
    pub id: String,
    /// The session the node is in; none for the page's own.
    pub session: Option<String>,
    /// The cross-site iframe the node is in, whose owner element places it on the page.
    pub frame_id: Option<String>,
    /// The DOM node.
    pub backend_node_id: i64,
    /// Its role.
    pub role: String,
    /// Its name.
    pub name: String,
}

impl RefTarget {
    /// The node as the Agent chip names it: `button “Sign in”`.
    #[must_use]
    pub fn describe(&self) -> String {
        if self.name.is_empty() {
            self.role.clone()
        } else {
            format!("{} \u{201c}{}\u{201d}", self.role, self.name)
        }
    }
}

/// A snapshot: its text, and what its refs name.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    /// The lines, as the agent reads them.
    pub text: String,
    /// Its refs, in the order they appear.
    pub refs: Vec<RefTarget>,
    /// Whether the snapshot stopped at [`MAX_CHARS`].
    pub cut: bool,
}

/// Writes `frames` as text: the interactive nodes, or with `full` every node, indented.
#[must_use]
pub fn render(frames: &[FrameTree], full: bool) -> Snapshot {
    let mut writer = Writer {
        snapshot: Snapshot::default(),
        full,
    };
    for frame in frames {
        // An iframe's nodes go under a line that names it.
        let depth = frame.label.as_ref().map_or(0, |label| {
            writer.line(0, &format!("- iframe ({}):", redact_url(label)));
            1
        });
        let nodes: std::collections::HashMap<&str, &AxNode> = frame
            .nodes
            .iter()
            .map(|node| (node.node_id.as_str(), node))
            .collect();
        let root = frame.nodes.iter().find(|node| {
            node.parent_id
                .as_deref()
                .is_none_or(|parent| !nodes.contains_key(parent))
        });
        if let Some(root) = root {
            writer.walk(root, &nodes, frame, depth, "");
        }
        if writer.snapshot.cut {
            break;
        }
    }
    if writer.snapshot.cut {
        writer
            .snapshot
            .text
            .push_str("\u{2026} (the snapshot stops here, at 30,000 characters)\n");
    }
    writer.snapshot
}

struct Writer {
    snapshot: Snapshot,
    full: bool,
}

impl Writer {
    fn line(&mut self, depth: usize, text: &str) {
        if self.snapshot.cut {
            return;
        }
        if self.snapshot.text.len() + text.len() + depth * 2 + 1 > MAX_CHARS {
            self.snapshot.cut = true;
            return;
        }
        for _ in 0..depth {
            self.snapshot.text.push_str("  ");
        }
        self.snapshot.text.push_str(text);
        self.snapshot.text.push('\n');
    }

    fn walk(
        &mut self,
        node: &AxNode,
        nodes: &std::collections::HashMap<&str, &AxNode>,
        frame: &FrameTree,
        depth: usize,
        parent_name: &str,
    ) {
        if self.snapshot.cut {
            return;
        }
        let role = text_of(node.role.as_ref());
        let name = clean(&text_of(node.name.as_ref()));
        let mut child_depth = depth;
        let shown = !node.ignored && role != "InlineTextBox";
        if shown {
            let interactive = INTERACTIVE.contains(&role.as_str());
            let line = if interactive && let Some(backend) = node.backend_dom_node_id {
                let id = format!("e{}", self.snapshot.refs.len() + 1);
                let line = self.describe(node, &role, &name, Some(&id));
                self.snapshot.refs.push(RefTarget {
                    id,
                    session: frame.session.clone(),
                    frame_id: frame.frame_id.clone(),
                    backend_node_id: backend,
                    role,
                    name: name.clone(),
                });
                Some(line)
            } else if self.full {
                self.full_line(node, &role, &name, parent_name)
            } else {
                None
            };
            if let Some(line) = line {
                self.line(depth, &line);
                if self.full {
                    child_depth = depth + 1;
                }
            }
        }
        for child in &node.child_ids {
            if let Some(child) = nodes.get(child.as_str()) {
                self.walk(child, nodes, frame, child_depth, &name);
            }
        }
    }

    /// A node that is not interactive, as `full` writes it; none for a node that only holds
    /// others, or text that repeats its parent's name.
    fn full_line(
        &self,
        node: &AxNode,
        role: &str,
        name: &str,
        parent_name: &str,
    ) -> Option<String> {
        match role {
            "StaticText" if name.is_empty() || name == parent_name => None,
            "StaticText" => Some(format!("- text \"{name}\"")),
            _ if name.is_empty() && CONTAINERS.contains(&role) => None,
            "RootWebArea" | "WebArea" => Some(format!("- document \"{name}\"")),
            _ => Some(self.describe(node, role, name, None)),
        }
    }

    /// A node's line: its role, its name, its states, its ref, and a link's URL.
    fn describe(&self, node: &AxNode, role: &str, name: &str, reference: Option<&str>) -> String {
        let mut parts = vec![format!("- {role}")];
        if !name.is_empty() {
            parts.push(format!("\"{name}\""));
        }
        for property in &node.properties {
            let value = property.value.value.as_ref();
            let state = match (property.name.as_str(), value) {
                ("disabled", Some(Value::Bool(true))) => Some("disabled".to_string()),
                ("checked", Some(Value::String(checked))) if checked == "true" => {
                    Some("checked".to_string())
                }
                ("checked", Some(Value::String(checked))) if checked == "mixed" => {
                    Some("mixed".to_string())
                }
                ("checked", Some(Value::Bool(true))) => Some("checked".to_string()),
                ("expanded", Some(Value::Bool(true))) => Some("expanded".to_string()),
                ("expanded", Some(Value::Bool(false))) => Some("collapsed".to_string()),
                ("selected", Some(Value::Bool(true))) => Some("selected".to_string()),
                ("focused", Some(Value::Bool(true))) => Some("focused".to_string()),
                ("level", Some(Value::Number(level))) if self.full => {
                    Some(format!("level={level}"))
                }
                _ => None,
            };
            if let Some(state) = state {
                parts.push(format!("[{state}]"));
            }
        }
        if let Some(reference) = reference {
            parts.push(format!("[ref={reference}]"));
        }
        if role == "link"
            && let Some(url) = node
                .properties
                .iter()
                .find(|property| property.name == "url")
                .and_then(|property| property.value.value.as_ref())
                .and_then(Value::as_str)
        {
            parts.push(format!("({})", redact_url(url)));
        }
        parts.join(" ")
    }
}

/// A value's text: a string as it is, anything else empty.
fn text_of(value: Option<&AxValue>) -> String {
    value
        .and_then(|value| value.value.as_ref())
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// A name on one line, without its quotes' ambiguity, at most [`MAX_NAME`] characters.
fn clean(name: &str) -> String {
    let flat: String = name
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('"', "\u{201d}");
    if flat.chars().count() > MAX_NAME {
        let kept: String = flat.chars().take(MAX_NAME).collect();
        format!("{kept}\u{2026}")
    } else {
        flat
    }
}
