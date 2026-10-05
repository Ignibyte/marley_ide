//! The Graph tab's settings (#657): the record of Rusty's graph panel.
//!
//! Rusty keeps one such record for both of its graph tabs (`GraphView.qml:101-106`), and Obsidian
//! one `graph.json` per vault: the four switches, the depth, the colour groups, Display (arrows,
//! the label fade, node size, link thickness) and the four forces. Marley keeps it once for every
//! window. [`GraphSettings::from_stored`] reads it field by field, so a field that is missing,
//! of the wrong type or out of its range falls back to Rusty's default alone, and says why.
//! [`group_colors`] gives each shown node the first group whose query matches it.

use std::collections::{BTreeSet, HashMap};

use serde_json::{Map, Value, json};

use crate::graph::{Graph, Node, Query, Shown};

/// A slider's range, its step and its default (Rusty's, `GraphView.qml:538-559`, with each
/// default on the step's grid).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderRange {
    /// The least value.
    pub min: f32,
    /// The greatest value.
    pub max: f32,
    /// The grid the value keeps to, from `min`.
    pub step: f32,
    /// Rusty's default.
    pub default: f32,
}

impl SliderRange {
    /// `value` held to the range and put on the step's grid; anything not finite is the default.
    #[must_use]
    pub fn clamp(self, value: f32) -> f32 {
        if !value.is_finite() {
            return self.default;
        }
        let held = value.clamp(self.min, self.max);
        if self.step <= 0.0 {
            return held;
        }
        let snapped = ((held - self.min) / self.step)
            .round()
            .mul_add(self.step, self.min);
        snapped.clamp(self.min, self.max)
    }

    /// Whether `value` is finite and inside the range, so it is kept as stored.
    #[must_use]
    pub fn holds(self, value: f32) -> bool {
        value.is_finite() && value >= self.min && value <= self.max
    }

    /// How many decimals the value is shown with: two for a step under 0.1, one under 1, none
    /// from 1 up.
    #[must_use]
    pub fn decimals(self) -> usize {
        if self.step >= 1.0 {
            0
        } else if self.step >= 0.1 - f32::EPSILON {
            1
        } else {
            2
        }
    }
}

/// Text fade threshold: where labels start showing as the view zooms in.
pub const TEXT_FADE: SliderRange = SliderRange {
    min: 0.0,
    max: 1.0,
    step: 0.05,
    default: 0.5,
};

/// Node size: each node's radius times this.
pub const NODE_SIZE: SliderRange = SliderRange {
    min: 0.3,
    max: 3.0,
    step: 0.1,
    default: 1.0,
};

/// Link thickness: each edge's width times this.
pub const LINK_THICKNESS: SliderRange = SliderRange {
    min: 0.2,
    max: 4.0,
    step: 0.1,
    default: 1.0,
};

/// Center force: the pull to the origin.
pub const CENTER_FORCE: SliderRange = SliderRange {
    min: 0.0,
    max: 1.0,
    step: 0.05,
    default: 0.5,
};

/// Repel force: the push between every pair of nodes.
pub const REPEL_FORCE: SliderRange = SliderRange {
    min: 0.0,
    max: 20.0,
    step: 0.5,
    default: 10.0,
};

/// Link force: the pull along every edge.
pub const LINK_FORCE: SliderRange = SliderRange {
    min: 0.0,
    max: 1.0,
    step: 0.05,
    default: 1.0,
};

/// Link distance: the length edges settle towards, in Rusty's units.
pub const LINK_DISTANCE: SliderRange = SliderRange {
    min: 30.0,
    max: 500.0,
    step: 10.0,
    default: 250.0,
};

/// The depths a local graph reaches.
pub const DEPTHS: [usize; 4] = [1, 2, 3, 4];

/// The four force sliders, each a multiplier on the layout's step that is 1 at Rusty's default,
/// so a graph with no record is laid out as #647 lays it out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Forces {
    /// The pull to the origin.
    pub center: f32,
    /// The push between every pair.
    pub repel: f32,
    /// The pull along every edge.
    pub link: f32,
    /// The length edges settle towards, in Rusty's units.
    pub distance: f32,
}

impl Default for Forces {
    fn default() -> Self {
        Self {
            center: CENTER_FORCE.default,
            repel: REPEL_FORCE.default,
            link: LINK_FORCE.default,
            distance: LINK_DISTANCE.default,
        }
    }
}

impl Forces {
    /// The ideal edge length in world units: `link` at the default distance.
    #[must_use]
    pub fn length(self, link: f32) -> f32 {
        link * self.distance / LINK_DISTANCE.default
    }

    /// The repulsion's multiplier.
    #[must_use]
    pub fn repel_scale(self) -> f32 {
        self.repel / REPEL_FORCE.default
    }

    /// The pull to the origin's multiplier.
    #[must_use]
    pub fn center_scale(self) -> f32 {
        self.center / CENTER_FORCE.default
    }

    /// The forces as the run's log line names them.
    #[must_use]
    pub fn describe(self) -> String {
        format!(
            "center {:.2} repel {:.1} link {:.2} distance {:.0}",
            self.center, self.repel, self.link, self.distance
        )
    }
}

/// Display: arrows, where labels show, node size and link thickness.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Display {
    /// A head at the target end of each edge that has a direction.
    pub arrows: bool,
    /// Text fade threshold, 0 to 1.
    pub text_fade: f32,
    /// Node size, a multiplier.
    pub node_size: f32,
    /// Link thickness, a multiplier.
    pub link_thickness: f32,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            arrows: false,
            text_fade: TEXT_FADE.default,
            node_size: NODE_SIZE.default,
            link_thickness: LINK_THICKNESS.default,
        }
    }
}

/// The zoom labels start at under the default threshold, and how much more zoom brings them in
/// fully (#647's).
const LABEL_ZOOM: f32 = 0.9;
const LABEL_FADE: f32 = 0.4;

/// How far the start moves per unit of the threshold (Rusty's, `GraphView.qml:348`).
const FADE_SLOPE: f32 = 1.6;

impl Display {
    /// How opaque the ranked labels are at `zoom`: 0 below the threshold's start, 1 from 0.4 of
    /// zoom past it.
    #[must_use]
    pub fn label_alpha(self, zoom: f32) -> f32 {
        let start = (TEXT_FADE.default - self.text_fade).mul_add(FADE_SLOPE, LABEL_ZOOM);
        ((zoom - start) / LABEL_FADE).clamp(0.0, 1.0)
    }
}

/// A colour group's hue: the theme's terminal colours in Rusty's palette order, kept by name so
/// a theme change recolours them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupColor {
    /// The terminal's red.
    Red,
    /// The terminal's green.
    Green,
    /// The terminal's yellow.
    Yellow,
    /// The terminal's magenta.
    Magenta,
    /// The terminal's cyan.
    Cyan,
    /// The terminal's blue.
    Blue,
}

impl GroupColor {
    /// Every hue, in the order new groups and the swatch take them.
    pub const ALL: [Self; 6] = [
        Self::Red,
        Self::Green,
        Self::Yellow,
        Self::Magenta,
        Self::Cyan,
        Self::Blue,
    ];

    /// The hue a group at `place` in the list starts with.
    #[must_use]
    pub fn for_place(place: usize) -> Self {
        Self::ALL
            .get(place % Self::ALL.len())
            .copied()
            .unwrap_or(Self::Red)
    }

    /// The hue after this one, back to the first after the last.
    #[must_use]
    pub fn next(self) -> Self {
        let at = Self::ALL.iter().position(|hue| *hue == self).unwrap_or(0);
        Self::for_place(at + 1)
    }

    /// The hue's stored name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Magenta => "magenta",
            Self::Cyan => "cyan",
            Self::Blue => "blue",
        }
    }

    fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|hue| hue.name() == name)
    }
}

/// A colour group: the nodes its query matches take its hue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// A query in the filter's grammar; an empty one matches nothing.
    pub query: String,
    /// Its hue.
    pub color: GroupColor,
}

/// One of the panel's four switches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Switch {
    /// Tags as nodes.
    Tags,
    /// Links to pages that do not exist yet.
    Unresolved,
    /// A decision's typed edges.
    DecisionEdges,
    /// Pages with no shown edge.
    Orphans,
}

impl Switch {
    /// Every switch, in the panel's order.
    pub const ALL: [Self; 4] = [
        Self::Tags,
        Self::Unresolved,
        Self::DecisionEdges,
        Self::Orphans,
    ];

    /// The switch's key in the stored record.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Tags => "tags",
            Self::Unresolved => "unresolved",
            Self::DecisionEdges => "decision_edges",
            Self::Orphans => "orphans",
        }
    }

    /// Whether it is on with no record (Rusty's, as #647 starts).
    #[must_use]
    pub const fn on_by_default(self) -> bool {
        matches!(self, Self::DecisionEdges | Self::Orphans)
    }
}

/// The Graph tab's settings, one record for every window.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphSettings {
    /// The switches that are on.
    pub switches: BTreeSet<Switch>,
    /// How far a local graph reaches, one of [`DEPTHS`].
    pub depth: usize,
    /// The colour groups, the first match winning.
    pub groups: Vec<Group>,
    /// Display.
    pub display: Display,
    /// Forces.
    pub forces: Forces,
}

impl Default for GraphSettings {
    fn default() -> Self {
        Self {
            switches: Switch::ALL
                .into_iter()
                .filter(|switch| switch.on_by_default())
                .collect(),
            depth: 1,
            groups: Vec::new(),
            display: Display::default(),
            forces: Forces::default(),
        }
    }
}

/// One record's fields as they are read, with a line for each that fell back.
struct Reader<'a> {
    fields: &'a Map<String, Value>,
    fell_back: Vec<String>,
}

impl Reader<'_> {
    fn flag(&mut self, key: &str, default: bool) -> bool {
        match self.fields.get(key) {
            None => default,
            Some(Value::Bool(flag)) => *flag,
            Some(other) => {
                self.fell_back
                    .push(format!("{key}: {other} is not true or false"));
                default
            }
        }
    }

    fn number(&mut self, key: &str, range: SliderRange) -> f32 {
        let Some(value) = self.fields.get(key) else {
            return range.default;
        };
        // Read through the number's text, so no float is cast; one past f32's range reads as
        // infinite, which `holds` refuses.
        let read = value
            .as_number()
            .and_then(|number| number.to_string().parse::<f32>().ok());
        match read {
            Some(number) if range.holds(number) => range.clamp(number),
            _ => {
                self.fell_back.push(format!(
                    "{key}: {value} is not a number from {} to {}",
                    range.min, range.max
                ));
                range.default
            }
        }
    }

    fn depth(&mut self, default: usize) -> usize {
        let Some(value) = self.fields.get("depth") else {
            return default;
        };
        if let Some(depth) = value
            .as_u64()
            .and_then(|depth| usize::try_from(depth).ok())
            .filter(|depth| DEPTHS.contains(depth))
        {
            return depth;
        }
        self.fell_back
            .push(format!("depth: {value} is not one of 1 to 4"));
        default
    }

    fn groups(&mut self) -> Vec<Group> {
        let Some(value) = self.fields.get("groups") else {
            return Vec::new();
        };
        let Some(list) = value.as_array() else {
            self.fell_back
                .push(format!("groups: {value} is not a list"));
            return Vec::new();
        };
        let mut groups = Vec::new();
        for (place, entry) in list.iter().enumerate() {
            let Some(query) = entry.get("query").and_then(Value::as_str) else {
                self.fell_back
                    .push(format!("groups: {entry} has no query; left out"));
                continue;
            };
            let named = entry.get("color").and_then(Value::as_str);
            let color = named.and_then(GroupColor::named).unwrap_or_else(|| {
                self.fell_back.push(format!(
                    "groups: {query:?} has no known colour; it takes the one for its place"
                ));
                GroupColor::for_place(place)
            });
            groups.push(Group {
                query: query.to_string(),
                color,
            });
        }
        groups
    }
}

impl GraphSettings {
    /// Whether `switch` is on.
    #[must_use]
    pub fn is_on(&self, switch: Switch) -> bool {
        self.switches.contains(&switch)
    }

    /// Turns `switch` the other way.
    pub fn toggle(&mut self, switch: Switch) {
        if !self.switches.remove(&switch) {
            let _added = self.switches.insert(switch);
        }
    }

    /// Reads a stored record: the settings, and a line for each field that fell back to Rusty's
    /// default, for the log.
    #[must_use]
    pub fn from_stored(text: &str) -> (Self, Vec<String>) {
        let defaults = Self::default();
        let parsed: Value = match serde_json::from_str(text) {
            Ok(parsed) => parsed,
            Err(error) => return (defaults, vec![format!("the record did not parse: {error}")]),
        };
        let Some(fields) = parsed.as_object() else {
            return (
                defaults,
                vec!["the record is not a JSON object".to_string()],
            );
        };
        let mut reader = Reader {
            fields,
            fell_back: Vec::new(),
        };
        let settings = Self {
            switches: Switch::ALL
                .into_iter()
                .filter(|switch| reader.flag(switch.key(), switch.on_by_default()))
                .collect(),
            depth: reader.depth(defaults.depth),
            groups: reader.groups(),
            display: Display {
                arrows: reader.flag("arrows", defaults.display.arrows),
                text_fade: reader.number("text_fade", TEXT_FADE),
                node_size: reader.number("node_size", NODE_SIZE),
                link_thickness: reader.number("link_thickness", LINK_THICKNESS),
            },
            forces: Forces {
                center: reader.number("center_force", CENTER_FORCE),
                repel: reader.number("repel_force", REPEL_FORCE),
                link: reader.number("link_force", LINK_FORCE),
                distance: reader.number("link_distance", LINK_DISTANCE),
            },
        };
        (settings, reader.fell_back)
    }

    /// The record as it is stored.
    #[must_use]
    pub fn to_stored(&self) -> String {
        let groups: Vec<Value> = self
            .groups
            .iter()
            .map(|group| json!({ "query": group.query, "color": group.color.name() }))
            .collect();
        let mut stored = json!({
            "depth": self.depth,
            "groups": groups,
            "arrows": self.display.arrows,
            "text_fade": self.display.text_fade,
            "node_size": self.display.node_size,
            "link_thickness": self.display.link_thickness,
            "center_force": self.forces.center,
            "repel_force": self.forces.repel,
            "link_force": self.forces.link,
            "link_distance": self.forces.distance,
        });
        for switch in Switch::ALL {
            stored[switch.key()] = json!(self.is_on(switch));
        }
        stored.to_string()
    }
}

/// Which group colours each shown node, and how many nodes each colours.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GroupColoring {
    /// For each shown node, by its place, the group whose hue it takes.
    pub per_node: Vec<Option<usize>>,
    /// For each group, by its place, the shown nodes it colours.
    pub counts: Vec<usize>,
}

/// Gives each node of `shown` the first group of `groups` whose query matches it in `graph`. A
/// group an earlier one shadows colours none, and so counts 0.
#[must_use]
pub fn group_colors(graph: &Graph, shown: &Shown, groups: &[Group]) -> GroupColoring {
    let queries: Vec<Option<Query>> = groups
        .iter()
        .map(|group| Some(Query::parse(&group.query)).filter(|query| !query.is_empty()))
        .collect();
    let mut counts = vec![0; groups.len()];
    if queries.iter().all(Option::is_none) {
        return GroupColoring {
            per_node: vec![None; shown.nodes.len()],
            counts,
        };
    }
    let by_id: HashMap<&str, &Node> = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();
    let per_node = shown
        .nodes
        .iter()
        .map(|shown_node| {
            let node = by_id.get(shown_node.id.as_str())?;
            let at = queries
                .iter()
                .position(|query| query.as_ref().is_some_and(|query| query.matches(node)))?;
            if let Some(count) = counts.get_mut(at) {
                *count += 1;
            }
            Some(at)
        })
        .collect();
    GroupColoring { per_node, counts }
}
