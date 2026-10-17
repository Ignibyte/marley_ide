//! The components an agent asks Marley to draw (#799).
//!
//! A component is a JSON object whose `type` names one entry of the [`catalog`]. A `row`, a
//! `column` and a `card` hold more components in `children`. [`parse`] checks a tree against the
//! catalog and the limits ([`MAX_DEPTH`], [`MAX_NODES`], [`MAX_TABLE_CELLS`], [`MAX_SERIES`]) and
//! names the JSON path of the first component or field it refuses. [`render`] draws a parsed tree
//! with Zed's `ui` pieces. Every color comes from the active theme through a [`Tone`]: no component
//! takes a color value, so a tree looks right in every theme.
//!
//! The design and the published approaches it was chosen over are in
//! `docs/planning/design-notes/agent-components-2026-10-17.md`.

// gate:21 runs Zed's dylint lints (`tooling/lints`) with these as errors in the Marley crates;
// Zed's crates keep them at warn (CONSTITUTION §0).
#![cfg_attr(
    dylint_lib = "lints",
    deny(
        async_block_without_await,
        blocking_io_on_foreground,
        entity_update_in_render,
        map_lookup_then_insert,
        notify_in_render,
        owned_string_into_shared,
        shared_string_from_str_literal
    )
)]

use std::fmt;

use gpui::{FontWeight, Hsla, PathBuilder, canvas, point};
use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use serde_json::{Map, Number, Value, json};
use ui::{Callout, Chip, Divider, ProgressBar, Table, prelude::*};

/// The deepest a tree may nest, the root at depth 1.
pub const MAX_DEPTH: usize = 12;

/// The most components one tree may hold.
pub const MAX_NODES: usize = 2_000;

/// The most cells one table may hold, all rows together.
pub const MAX_TABLE_CELLS: usize = 100_000;

/// The most values a bar chart or a sparkline may hold.
pub const MAX_SERIES: usize = 2_000;

/// A tree the catalog accepts, ready to draw.
#[derive(Clone, Debug)]
pub struct Tree {
    root: Node,
}

/// One component and the components it holds.
#[derive(Clone, Debug)]
struct Node {
    kind: Kind,
    children: Vec<Self>,
}

/// Why a tree was refused: the JSON path of the component or field, and the reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// The JSON path, from `$` for the root: `$.children[2].rows[3][1]`.
    pub path: String,
    /// The reason, in a sentence an agent can act on.
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for ParseError {}

fn refuse(path: &str, message: impl Into<String>) -> ParseError {
    ParseError {
        path: path.to_owned(),
        message: message.into(),
    }
}

/// The color a component takes from the theme. No component takes a color value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    /// The theme's text color.
    #[default]
    Default,
    /// The theme's muted text color.
    Muted,
    /// The theme's accent color.
    Accent,
    /// The theme's success color.
    Success,
    /// The theme's warning color.
    Warning,
    /// The theme's error color.
    Error,
}

/// How a `text` component looks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum TextStyle {
    #[default]
    Body,
    Muted,
    Strong,
    Code,
}

/// The size of a `text` component.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum TextScale {
    Small,
    #[default]
    Default,
    Large,
}

/// The space between the components a `row` or a `column` holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Gap {
    None,
    Small,
    #[default]
    Medium,
    Large,
}

/// Where a `row` or a `column` puts its components on the cross axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Align {
    #[default]
    Start,
    Center,
    End,
}

/// Where a table column puts its text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum ColumnAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// The fields of a `text` component.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct TextFields {
    /// The text. A line break starts a new line.
    text: String,
    /// `body`, `muted`, `strong` or `code`. The default is `body`.
    #[serde(default)]
    style: TextStyle,
    /// `small`, `default` or `large`. The default is `default`.
    #[serde(default)]
    size: TextScale,
}

/// The fields of a `heading` component.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct HeadingFields {
    /// The heading's text.
    text: String,
    /// 1, 2 or 3. Level 1 is the largest. The default is 2.
    #[serde(default = "default_level")]
    level: u8,
}

const fn default_level() -> u8 {
    2
}

/// The fields of a `row` or a `column`, besides `children`.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct StackFields {
    /// `none`, `small`, `medium` or `large`. The default is `medium`.
    #[serde(default)]
    gap: Gap,
    /// `start`, `center` or `end` on the cross axis. The default is `start`.
    #[serde(default)]
    align: Align,
    /// For a `row`: true puts components that do not fit on a next line. The default is false.
    #[serde(default)]
    wrap: bool,
}

/// The fields of a `card`, besides `children`.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CardFields {
    /// The text at the top of the card.
    #[serde(default)]
    title: Option<String>,
    /// The color of the card's border. The default is `default`.
    #[serde(default)]
    tone: Tone,
}

/// The fields of a `divider`: none.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DividerFields {}

/// A table column: its label as a string, or an object with the label and the alignment.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(untagged)]
enum Column {
    /// The column's label.
    Label(String),
    /// The column's label and alignment.
    Full {
        /// The column's label.
        label: String,
        /// `left`, `center` or `right`. The default is `left`.
        #[serde(default)]
        align: ColumnAlign,
    },
}

impl Column {
    fn label(&self) -> &str {
        match self {
            Self::Label(label) | Self::Full { label, .. } => label,
        }
    }

    const fn align(&self) -> ColumnAlign {
        match self {
            Self::Label(_) => ColumnAlign::Left,
            Self::Full { align, .. } => *align,
        }
    }
}

/// The tag of a badge in a table cell.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum BadgeTag {
    Badge,
}

/// A table cell: a string, a number, true or false, null for an empty cell, or a badge.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(untagged)]
enum Cell {
    /// Text.
    Text(String),
    /// A number, shown as written.
    Number(Number),
    /// True or false.
    Flag(bool),
    /// A badge: `{"type": "badge", "text": "ok", "tone": "success"}`.
    Badge {
        /// Always `badge`.
        #[serde(rename = "type")]
        _tag: BadgeTag,
        /// The badge's text.
        text: String,
        /// The badge's color. The default is `default`.
        #[serde(default)]
        tone: Tone,
    },
    /// An empty cell.
    Empty(()),
}

/// The fields of a `table`.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct TableFields {
    /// The columns, left to right. Each is a label, or `{"label": ..., "align": ...}`.
    columns: Vec<Column>,
    /// The rows, top to bottom. Each row is a list of cells, one cell for each column.
    rows: Vec<Vec<Cell>>,
    /// A line of text under the table.
    #[serde(default)]
    caption: Option<String>,
    /// True shades every other row. The default is false.
    #[serde(default)]
    striped: bool,
}

/// A value of a `key_value` item or a `stat`: a string or a number.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(untagged)]
enum Scalar {
    /// Text.
    Text(String),
    /// A number, shown as written.
    Number(Number),
}

impl Scalar {
    fn text(&self) -> String {
        match self {
            Self::Text(text) => text.clone(),
            Self::Number(number) => number.to_string(),
        }
    }
}

/// One `key_value` item.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct KeyValueItem {
    /// The label on the left.
    label: String,
    /// The value on the right: a string or a number.
    value: Scalar,
    /// The value's color. The default is `default`.
    #[serde(default)]
    tone: Tone,
}

/// The fields of a `key_value` list.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct KeyValueFields {
    /// The items, top to bottom.
    items: Vec<KeyValueItem>,
}

/// The fields of a `stat`.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct StatFields {
    /// The small text over the value.
    label: String,
    /// The large value: a string or a number.
    value: Scalar,
    /// A short change under the value, such as `+12% since Monday`.
    #[serde(default)]
    change: Option<String>,
    /// The change's color. The default is `default`.
    #[serde(default)]
    tone: Tone,
}

/// The fields of a `badge`.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct BadgeFields {
    /// The badge's text.
    text: String,
    /// The badge's color. The default is `default`.
    #[serde(default)]
    tone: Tone,
}

/// The fields of a `progress` bar.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProgressFields {
    /// How far the work is, from 0 to `max`.
    value: f32,
    /// The value of complete work. The default is 100.
    #[serde(default = "default_max")]
    max: f32,
    /// The text over the bar.
    #[serde(default)]
    label: Option<String>,
    /// The bar's color. The default is `accent`.
    #[serde(default = "accent")]
    tone: Tone,
}

const fn default_max() -> f32 {
    100.0
}

const fn accent() -> Tone {
    Tone::Accent
}

/// One bar of a `bar_chart`.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Bar {
    /// The text on the left of the bar.
    label: String,
    /// The bar's value, 0 or more.
    value: f32,
    /// The bar's color. The default is `accent`.
    #[serde(default = "accent")]
    tone: Tone,
}

/// The fields of a `bar_chart`.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct BarChartFields {
    /// The bars, top to bottom. The longest bar shows the largest value.
    bars: Vec<Bar>,
    /// A unit after each value, such as `ms` or `%`.
    #[serde(default)]
    unit: Option<String>,
    /// The text over the chart.
    #[serde(default)]
    title: Option<String>,
}

/// The fields of a `sparkline`.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SparklineFields {
    /// The values, oldest first. A sparkline needs 2 values or more.
    values: Vec<f32>,
    /// The text on the left of the line.
    #[serde(default)]
    label: Option<String>,
    /// The line's color. The default is `accent`.
    #[serde(default = "accent")]
    tone: Tone,
}

/// The fields of a `code` block.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CodeFields {
    /// The code. A line break starts a new line.
    text: String,
    /// The language's name, shown over the code, such as `rust`.
    #[serde(default)]
    language: Option<String>,
}

/// The fields of a `callout`.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CalloutFields {
    /// The callout's text.
    text: String,
    /// The text in bold over the callout's text.
    #[serde(default)]
    title: Option<String>,
    /// `success`, `warning` or `error`. Any other tone shows as information. The default is
    /// `default`.
    #[serde(default)]
    tone: Tone,
}

/// A component's own fields, by its type.
#[derive(Clone, Debug)]
enum Kind {
    Text(TextFields),
    Heading(HeadingFields),
    Row(StackFields),
    Column(StackFields),
    Card(CardFields),
    Divider,
    Table(TableFields),
    KeyValue(KeyValueFields),
    Stat(StatFields),
    Badge(BadgeFields),
    Progress(ProgressFields),
    BarChart(BarChartFields),
    Sparkline(SparklineFields),
    Code(CodeFields),
    Callout(CalloutFields),
}

/// The types that hold `children`.
const CONTAINERS: [&str; 3] = ["row", "column", "card"];

/// Every type the catalog has, in the order `component_list` gives them.
pub const TYPES: [&str; 15] = [
    "text",
    "heading",
    "row",
    "column",
    "card",
    "divider",
    "table",
    "key_value",
    "stat",
    "badge",
    "progress",
    "bar_chart",
    "sparkline",
    "code",
    "callout",
];

/// Checks `value` against the catalog and the limits.
///
/// # Errors
///
/// A [`ParseError`] names the first component or field the catalog refuses, by its JSON path.
pub fn parse(value: &Value) -> Result<Tree, ParseError> {
    let mut nodes = 0;
    let root = parse_node(value, "$", 1, &mut nodes)?;
    Ok(Tree { root })
}

/// Reads `text` as JSON and checks it as [`parse`] does.
///
/// # Errors
///
/// A [`ParseError`] at `$` when the text is not JSON, or [`parse`]'s error.
pub fn parse_str(text: &str) -> Result<Tree, ParseError> {
    let value: Value = serde_json::from_str(text)
        .map_err(|error| refuse("$", format!("The text is not JSON: {error}.")))?;
    parse(&value)
}

fn parse_node(
    value: &Value,
    path: &str,
    depth: usize,
    nodes: &mut usize,
) -> Result<Node, ParseError> {
    if depth > MAX_DEPTH {
        return Err(refuse(
            path,
            format!("The tree nests deeper than {MAX_DEPTH} levels. Use fewer levels."),
        ));
    }
    *nodes += 1;
    if *nodes > MAX_NODES {
        return Err(refuse(
            path,
            format!("The tree holds more than {MAX_NODES} components. Show less data."),
        ));
    }
    let Value::Object(object) = value else {
        return Err(refuse(
            path,
            "A component is a JSON object with a `type` field.",
        ));
    };
    let Some(type_name) = object.get("type").and_then(Value::as_str) else {
        return Err(refuse(
            path,
            "The component has no `type` string. Set `type` to a catalog type.",
        ));
    };
    if !TYPES.contains(&type_name) {
        return Err(refuse(
            path,
            format!(
                "The type `{type_name}` is not in the catalog. The types are: {}.",
                TYPES.join(", ")
            ),
        ));
    }
    let mut fields: Map<String, Value> = object.clone();
    fields.remove("type");
    let children = fields.remove("children");
    let children = if CONTAINERS.contains(&type_name) {
        match children {
            None => Vec::new(),
            Some(Value::Array(children)) => children
                .iter()
                .enumerate()
                .map(|(index, child)| {
                    parse_node(
                        child,
                        &format!("{path}.children[{index}]"),
                        depth + 1,
                        nodes,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
            Some(_) => {
                return Err(refuse(
                    &format!("{path}.children"),
                    "`children` is a list of components.",
                ));
            }
        }
    } else {
        if children.is_some() {
            return Err(refuse(
                &format!("{path}.children"),
                format!(
                    "The type `{type_name}` holds no `children`. Only {} hold children.",
                    CONTAINERS.join(", ")
                ),
            ));
        }
        Vec::new()
    };
    let fields = Value::Object(fields);
    let kind = match type_name {
        "text" => Kind::Text(fields_of(fields, path)?),
        "heading" => {
            let heading: HeadingFields = fields_of(fields, path)?;
            if !(1..=3).contains(&heading.level) {
                return Err(refuse(&format!("{path}.level"), "The level is 1, 2 or 3."));
            }
            Kind::Heading(heading)
        }
        "row" => Kind::Row(fields_of(fields, path)?),
        "column" => Kind::Column(fields_of(fields, path)?),
        "card" => Kind::Card(fields_of(fields, path)?),
        "divider" => {
            let DividerFields {} = fields_of(fields, path)?;
            Kind::Divider
        }
        "table" => Kind::Table(table(fields_of(fields, path)?, path)?),
        "key_value" => Kind::KeyValue(fields_of(fields, path)?),
        "stat" => Kind::Stat(fields_of(fields, path)?),
        "badge" => Kind::Badge(fields_of(fields, path)?),
        "progress" => Kind::Progress(progress(fields_of(fields, path)?, path)?),
        "bar_chart" => Kind::BarChart(bar_chart(fields_of(fields, path)?, path)?),
        "sparkline" => Kind::Sparkline(sparkline(fields_of(fields, path)?, path)?),
        "code" => Kind::Code(fields_of(fields, path)?),
        _ => Kind::Callout(fields_of(fields, path)?),
    };
    Ok(Node { kind, children })
}

/// Reads a component's own fields, the error at the field's path under the component's.
fn fields_of<T: serde::de::DeserializeOwned>(fields: Value, path: &str) -> Result<T, ParseError> {
    serde_path_to_error::deserialize(fields).map_err(|error| {
        let inner = error.path().to_string();
        let at = if inner == "." {
            path.to_owned()
        } else {
            format!("{path}.{inner}")
        };
        let message = error.into_inner().to_string();
        refuse(&at, sentence(&message))
    })
}

/// serde's message as a sentence: a capital first letter and a final period.
fn sentence(message: &str) -> String {
    let mut characters = message.chars();
    let mut text = characters
        .next()
        .map(|first| first.to_uppercase().chain(characters).collect::<String>())
        .unwrap_or_default();
    if !text.ends_with('.') {
        text.push('.');
    }
    text
}

fn table(table: TableFields, path: &str) -> Result<TableFields, ParseError> {
    if table.columns.is_empty() {
        return Err(refuse(
            &format!("{path}.columns"),
            "A table needs one column or more.",
        ));
    }
    let mut cells = 0;
    for (index, row) in table.rows.iter().enumerate() {
        if row.len() > table.columns.len() {
            return Err(refuse(
                &format!("{path}.rows[{index}]"),
                format!(
                    "The row has {} cells for {} columns. Give each row one cell for each column.",
                    row.len(),
                    table.columns.len()
                ),
            ));
        }
        cells += row.len();
        if cells > MAX_TABLE_CELLS {
            return Err(refuse(
                &format!("{path}.rows[{index}]"),
                format!("The table holds more than {MAX_TABLE_CELLS} cells. Show less data."),
            ));
        }
    }
    Ok(table)
}

fn progress(progress: ProgressFields, path: &str) -> Result<ProgressFields, ParseError> {
    if !progress.value.is_finite() {
        return Err(refuse(
            &format!("{path}.value"),
            "The value is a finite number.",
        ));
    }
    if !(progress.max.is_finite() && progress.max > 0.0) {
        return Err(refuse(
            &format!("{path}.max"),
            "The maximum is a number over 0.",
        ));
    }
    Ok(progress)
}

fn bar_chart(chart: BarChartFields, path: &str) -> Result<BarChartFields, ParseError> {
    if chart.bars.len() > MAX_SERIES {
        return Err(refuse(
            &format!("{path}.bars"),
            format!("A bar chart holds {MAX_SERIES} bars or fewer."),
        ));
    }
    for (index, bar) in chart.bars.iter().enumerate() {
        if !(bar.value.is_finite() && bar.value >= 0.0) {
            return Err(refuse(
                &format!("{path}.bars[{index}].value"),
                "A bar's value is a finite number, 0 or more.",
            ));
        }
    }
    Ok(chart)
}

fn sparkline(line: SparklineFields, path: &str) -> Result<SparklineFields, ParseError> {
    if line.values.len() < 2 || line.values.len() > MAX_SERIES {
        return Err(refuse(
            &format!("{path}.values"),
            format!("A sparkline holds 2 to {MAX_SERIES} values."),
        ));
    }
    if let Some(index) = line.values.iter().position(|value| !value.is_finite()) {
        return Err(refuse(
            &format!("{path}.values[{index}]"),
            "Each value is a finite number.",
        ));
    }
    Ok(line)
}

/// The most rows a table draws. A line under the table counts the rest.
pub const MAX_DRAWN_ROWS: usize = 500;

/// Draws `tree`. `id` keeps the element ids of two trees in one view apart.
#[must_use]
pub fn render(tree: &Tree, id: impl Into<SharedString>, cx: &App) -> AnyElement {
    let id: SharedString = id.into();
    node(&tree.root, &id, cx)
}

/// Draws a refused tree's error in place of the component.
#[must_use]
pub fn render_error(error: &ParseError) -> AnyElement {
    Callout::new()
        .severity(Severity::Error)
        .icon(IconName::Warning)
        .title(SharedString::new_static(
            "Marley could not draw this component",
        ))
        .description(SharedString::from(error.to_string()))
        .into_any_element()
}

impl Tone {
    /// The label color of the tone.
    #[must_use]
    pub const fn color(self) -> Color {
        match self {
            Self::Default => Color::Default,
            Self::Muted => Color::Muted,
            Self::Accent => Color::Accent,
            Self::Success => Color::Success,
            Self::Warning => Color::Warning,
            Self::Error => Color::Error,
        }
    }

    /// The tone's color in the active theme, for what is not text.
    #[must_use]
    pub fn hsla(self, cx: &App) -> Hsla {
        let theme = cx.theme();
        match self {
            Self::Default => theme.colors().text,
            Self::Muted => theme.colors().text_muted,
            Self::Accent => theme.colors().text_accent,
            Self::Success => theme.status().success,
            Self::Warning => theme.status().warning,
            Self::Error => theme.status().error,
        }
    }

    const fn icon(self) -> IconName {
        match self {
            Self::Success => IconName::Check,
            Self::Warning => IconName::Warning,
            Self::Error => IconName::XCircle,
            Self::Default | Self::Muted | Self::Accent => IconName::Info,
        }
    }

    const fn severity(self) -> Severity {
        match self {
            Self::Success => Severity::Success,
            Self::Warning => Severity::Warning,
            Self::Error => Severity::Error,
            Self::Default | Self::Muted | Self::Accent => Severity::Info,
        }
    }
}

fn node(node: &Node, id: &str, cx: &App) -> AnyElement {
    let children = || {
        node.children
            .iter()
            .enumerate()
            .map(|(index, child)| self::node(child, &format!("{id}-{index}"), cx))
            .collect::<Vec<_>>()
    };
    match &node.kind {
        Kind::Text(text) => text_element(text, cx),
        Kind::Heading(heading) => {
            let size = match heading.level {
                1 => HeadlineSize::Large,
                2 => HeadlineSize::Medium,
                _ => HeadlineSize::Small,
            };
            Headline::new(SharedString::from(heading.text.clone()))
                .size(size)
                .into_any_element()
        }
        Kind::Row(stack) => stacked(h_flex(), stack.gap, stack.align)
            .when(stack.wrap, Styled::flex_wrap)
            .children(children())
            .into_any_element(),
        Kind::Column(stack) => stacked(v_flex(), stack.gap, stack.align)
            .children(children())
            .into_any_element(),
        Kind::Card(card) => card_element(card, children(), cx),
        Kind::Divider => Divider::horizontal().into_any_element(),
        Kind::Table(table) => table_element(table),
        Kind::KeyValue(list) => key_value_element(list),
        Kind::Stat(stat) => stat_element(stat),
        Kind::Badge(badge) => badge_chip(&badge.text, badge.tone),
        Kind::Progress(progress) => progress_element(progress, id, cx),
        Kind::BarChart(chart) => bar_chart_element(chart, cx),
        Kind::Sparkline(line) => sparkline_element(line, cx),
        Kind::Code(code) => code_element(code, cx),
        Kind::Callout(callout) => Callout::new()
            .severity(callout.tone.severity())
            .icon(callout.tone.icon())
            .when_some(callout.title.clone(), |drawn, title| {
                drawn.title(SharedString::from(title))
            })
            .description(SharedString::from(callout.text.clone()))
            .into_any_element(),
    }
}

fn text_element(text: &TextFields, cx: &App) -> AnyElement {
    let size = match text.size {
        TextScale::Small => LabelSize::Small,
        TextScale::Default => LabelSize::Default,
        TextScale::Large => LabelSize::Large,
    };
    v_flex()
        .children(text.text.lines().map(|line| {
            let label = Label::new(SharedString::from(line.to_owned())).size(size);
            match text.style {
                TextStyle::Body => label,
                TextStyle::Muted => label.color(Color::Muted),
                TextStyle::Strong => label.weight(FontWeight::BOLD),
                TextStyle::Code => label.buffer_font(cx),
            }
        }))
        .into_any_element()
}

fn card_element(card: &CardFields, children: Vec<AnyElement>, cx: &App) -> AnyElement {
    let colors = cx.theme().colors();
    let border = if card.tone == Tone::Default {
        colors.border
    } else {
        card.tone.hsla(cx)
    };
    v_flex()
        .gap_2()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(border)
        .bg(colors.elevated_surface_background)
        .when_some(card.title.clone(), |card, title| {
            card.child(Label::new(SharedString::from(title)).weight(FontWeight::BOLD))
        })
        .children(children)
        .into_any_element()
}

fn table_element(table: &TableFields) -> AnyElement {
    let columns = table.columns.len();
    let header = table
        .columns
        .iter()
        .map(|column| {
            aligned(
                Label::new(SharedString::from(column.label().to_owned()))
                    .size(LabelSize::Small)
                    .color(Color::Muted)
                    .into_any_element(),
                column.align(),
            )
        })
        .collect::<Vec<_>>();
    let mut drawn = Table::new(columns).header(header);
    if table.striped {
        drawn = drawn.striped();
    }
    for row in table.rows.iter().take(MAX_DRAWN_ROWS) {
        let cells = table
            .columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                aligned(
                    row.get(index)
                        .map_or_else(|| div().into_any_element(), cell),
                    column.align(),
                )
            })
            .collect::<Vec<_>>();
        drawn = drawn.row(cells);
    }
    let more = table.rows.len().saturating_sub(MAX_DRAWN_ROWS);
    v_flex()
        .gap_1()
        .w_full()
        .child(drawn)
        .when(more > 0, |table| {
            table.child(
                Label::new(SharedString::from(format!("{more} more rows")))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
        })
        .when_some(table.caption.clone(), |table, caption| {
            table.child(
                Label::new(SharedString::from(caption))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
        })
        .into_any_element()
}

fn key_value_element(list: &KeyValueFields) -> AnyElement {
    v_flex()
        .gap_1()
        .children(list.items.iter().map(|item| {
            h_flex()
                .gap_3()
                .items_start()
                .child(
                    div().w(rems(10.)).flex_none().child(
                        Label::new(SharedString::from(item.label.clone())).color(Color::Muted),
                    ),
                )
                .child(Label::new(SharedString::from(item.value.text())).color(item.tone.color()))
        }))
        .into_any_element()
}

fn stat_element(stat: &StatFields) -> AnyElement {
    v_flex()
        .gap_0p5()
        .child(
            Label::new(SharedString::from(stat.label.clone()))
                .size(LabelSize::Small)
                .color(Color::Muted),
        )
        .child(Headline::new(SharedString::from(stat.value.text())).size(HeadlineSize::Large))
        .when_some(stat.change.clone(), |stat_view, change| {
            stat_view.child(
                Label::new(SharedString::from(change))
                    .size(LabelSize::Small)
                    .color(stat.tone.color()),
            )
        })
        .into_any_element()
}

fn progress_element(progress: &ProgressFields, id: &str, cx: &App) -> AnyElement {
    let share = (progress.value / progress.max).clamp(0.0, 1.0) * 100.0;
    v_flex()
        .w_full()
        .gap_1()
        .child(
            h_flex()
                .justify_between()
                .gap_2()
                .child(
                    Label::new(SharedString::from(
                        progress.label.clone().unwrap_or_default(),
                    ))
                    .size(LabelSize::Small),
                )
                .child(
                    Label::new(SharedString::from(format!("{share:.0}%")))
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
        )
        .child(
            ProgressBar::new(
                SharedString::from(format!("{id}-progress")),
                progress.value,
                progress.max,
                cx,
            )
            .fg_color(progress.tone.hsla(cx)),
        )
        .into_any_element()
}

fn bar_chart_element(chart: &BarChartFields, cx: &App) -> AnyElement {
    let largest = chart.bars.iter().map(|bar| bar.value).fold(0.0, f32::max);
    let unit = chart.unit.clone().unwrap_or_default();
    let track = cx.theme().colors().element_background;
    v_flex()
        .w_full()
        .gap_1()
        .when_some(chart.title.clone(), |chart_view, title| {
            chart_view.child(Label::new(SharedString::from(title)).weight(FontWeight::BOLD))
        })
        .children(chart.bars.iter().map(|bar| {
            let share = if largest > 0.0 {
                bar.value / largest
            } else {
                0.0
            };
            h_flex()
                .gap_2()
                .child(
                    div().w(rems(8.)).flex_none().child(
                        Label::new(SharedString::from(bar.label.clone()))
                            .size(LabelSize::Small)
                            .truncate(),
                    ),
                )
                .child(
                    div().flex_1().h(rems(0.75)).rounded_sm().bg(track).child(
                        div()
                            .h_full()
                            .w(relative(share))
                            .rounded_sm()
                            .bg(bar.tone.hsla(cx)),
                    ),
                )
                .child(
                    div().w(rems(5.)).flex_none().child(
                        Label::new(SharedString::from(format!("{}{unit}", bar.value)))
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
                )
        }))
        .into_any_element()
}

fn sparkline_element(line: &SparklineFields, cx: &App) -> AnyElement {
    let values = line.values.clone();
    let color = line.tone.hsla(cx);
    h_flex()
        .w_full()
        .gap_2()
        .when_some(line.label.clone(), |row, label| {
            row.child(
                Label::new(SharedString::from(label))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
        })
        .child(
            div().flex_1().h(px(32.)).child(
                canvas(
                    |_, _, _| {},
                    move |bounds, (), window, _| {
                        let lowest = values.iter().copied().fold(f32::INFINITY, f32::min);
                        let highest = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                        let span = highest - lowest;
                        // A sparkline holds at most `MAX_SERIES` values, which fit a u16.
                        let steps = u16::try_from(values.len().saturating_sub(1))
                            .unwrap_or(u16::MAX)
                            .max(1);
                        let step = bounds.size.width / f32::from(steps);
                        let mut path = PathBuilder::stroke(px(1.5));
                        let mut x = bounds.origin.x;
                        for (index, value) in values.iter().enumerate() {
                            let rise = if span > 0.0 {
                                (value - lowest) / span
                            } else {
                                0.5
                            };
                            let at = point(x, bounds.origin.y + bounds.size.height * (1.0 - rise));
                            if index == 0 {
                                path.move_to(at);
                            } else {
                                path.line_to(at);
                            }
                            x += step;
                        }
                        if let Ok(path) = path.build() {
                            window.paint_path(path, color);
                        }
                    },
                )
                .size_full(),
            ),
        )
        .into_any_element()
}

fn code_element(code: &CodeFields, cx: &App) -> AnyElement {
    let colors = cx.theme().colors();
    v_flex()
        .p_2()
        .gap_1()
        .rounded_md()
        .border_1()
        .border_color(colors.border_variant)
        .bg(colors.editor_background)
        .when_some(code.language.clone(), |block, language| {
            block.child(
                Label::new(SharedString::from(language))
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
        })
        .children(code.text.lines().map(|line| {
            Label::new(SharedString::from(line.to_owned()))
                .size(LabelSize::Small)
                .buffer_font(cx)
        }))
        .into_any_element()
}

fn stacked(stack: Div, gap: Gap, align: Align) -> Div {
    let stack = match gap {
        Gap::None => stack,
        Gap::Small => stack.gap_1(),
        Gap::Medium => stack.gap_2(),
        Gap::Large => stack.gap_4(),
    };
    match align {
        Align::Start => stack.items_start(),
        Align::Center => stack.items_center(),
        Align::End => stack.items_end(),
    }
}

fn aligned(element: AnyElement, align: ColumnAlign) -> AnyElement {
    let cell = h_flex().w_full();
    match align {
        ColumnAlign::Left => cell.justify_start(),
        ColumnAlign::Center => cell.justify_center(),
        ColumnAlign::Right => cell.justify_end(),
    }
    .child(element)
    .into_any_element()
}

fn cell(cell: &Cell) -> AnyElement {
    match cell {
        Cell::Text(text) => Label::new(SharedString::from(text.clone())).into_any_element(),
        Cell::Number(number) => {
            Label::new(SharedString::from(number.to_string())).into_any_element()
        }
        Cell::Flag(flag) => Label::new(if *flag {
            SharedString::new_static("true")
        } else {
            SharedString::new_static("false")
        })
        .into_any_element(),
        Cell::Badge { text, tone, .. } => badge_chip(text, *tone),
        Cell::Empty(()) => div().into_any_element(),
    }
}

fn badge_chip(text: &str, tone: Tone) -> AnyElement {
    Chip::new(SharedString::from(text.to_owned()))
        .label_color(tone.color())
        .into_any_element()
}

/// The library's groups, in the order it shows them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    /// Components that hold and arrange other components.
    Layout,
    /// Text, headings and code.
    Text,
    /// Tables, lists and single values.
    Data,
    /// Charts.
    Charts,
    /// Badges, progress and callouts.
    Status,
}

impl Group {
    /// Every group, in the library's order.
    pub const ALL: [Self; 5] = [
        Self::Layout,
        Self::Text,
        Self::Data,
        Self::Charts,
        Self::Status,
    ];

    /// The group's heading in the library.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Layout => "Layout",
            Self::Text => "Text",
            Self::Data => "Data",
            Self::Charts => "Charts",
            Self::Status => "Status",
        }
    }
}

/// One catalog type, as the library shows it and `component_list` gives it.
#[derive(Clone, Debug)]
pub struct Entry {
    /// The value of `type`.
    pub name: &'static str,
    /// The library group.
    pub group: Group,
    /// What the type shows, for an agent and for the user.
    pub description: &'static str,
    /// The JSON Schema of the type's fields, `type` and `children` included.
    pub schema: Value,
    /// A tree that uses the type.
    pub example: Value,
}

/// Every catalog type, by group, in the library's order.
#[must_use]
pub fn catalog() -> Vec<Entry> {
    [layout(), text(), data(), charts(), status()].concat()
}

fn layout() -> Vec<Entry> {
    vec![
        entry::<StackFields>(
            "row",
            Group::Layout,
            "Puts its children side by side, left to right.",
            json!({"type": "row", "gap": "large", "children": [
                {"type": "stat", "label": "Tests", "value": 1284},
                {"type": "stat", "label": "Failures", "value": 0, "change": "-3 since Monday", "tone": "success"}
            ]}),
        ),
        entry::<StackFields>(
            "column",
            Group::Layout,
            "Puts its children one under the other, top to bottom.",
            json!({"type": "column", "gap": "small", "children": [
                {"type": "text", "text": "Deploy started", "style": "strong"},
                {"type": "text", "text": "Waiting for the health check.", "style": "muted"}
            ]}),
        ),
        entry::<CardFields>(
            "card",
            Group::Layout,
            "Puts its children in a box with a border and an optional title.",
            json!({"type": "card", "title": "staging", "tone": "success", "children": [
                {"type": "key_value", "items": [
                    {"label": "Version", "value": "1.4.0"},
                    {"label": "Uptime", "value": "3 days"}
                ]}
            ]}),
        ),
        entry::<DividerFields>(
            "divider",
            Group::Layout,
            "Shows a horizontal line between components.",
            json!({"type": "column", "children": [
                {"type": "text", "text": "Before"},
                {"type": "divider"},
                {"type": "text", "text": "After"}
            ]}),
        ),
    ]
}

fn text() -> Vec<Entry> {
    vec![
        entry::<TextFields>(
            "text",
            Group::Text,
            "Shows text. A line break starts a new line. Set `style` to muted, strong or code.",
            json!({"type": "text", "text": "The build passed in 4 minutes.", "style": "body"}),
        ),
        entry::<HeadingFields>(
            "heading",
            Group::Text,
            "Shows a heading. Level 1 is the largest.",
            json!({"type": "heading", "text": "Release 1.4", "level": 2}),
        ),
        entry::<CodeFields>(
            "code",
            Group::Text,
            "Shows code in the editor font, with the language's name over it.",
            json!({"type": "code", "language": "rust", "text": "fn main() {\n    println!(\"hello\");\n}"}),
        ),
    ]
}

fn data() -> Vec<Entry> {
    vec![
        entry::<TableFields>(
            "table",
            Group::Data,
            "Shows rows of cells under column labels. A cell is text, a number, true, false, \
             null or a badge. The table shows 500 rows or fewer.",
            json!({"type": "table",
                "columns": ["Service", {"label": "Latency", "align": "right"}, "State"],
                "rows": [
                    ["api", "42 ms", {"type": "badge", "text": "up", "tone": "success"}],
                    ["worker", "310 ms", {"type": "badge", "text": "slow", "tone": "warning"}],
                    ["search", null, {"type": "badge", "text": "down", "tone": "error"}]
                ],
                "caption": "Measured at 09:30 UTC.",
                "striped": true}),
        ),
        entry::<KeyValueFields>(
            "key_value",
            Group::Data,
            "Shows labels on the left and their values on the right.",
            json!({"type": "key_value", "items": [
                {"label": "Branch", "value": "main"},
                {"label": "Commits ahead", "value": 3},
                {"label": "Checks", "value": "failing", "tone": "error"}
            ]}),
        ),
        entry::<StatFields>(
            "stat",
            Group::Data,
            "Shows one large value with a label and an optional change.",
            json!({"type": "stat", "label": "Open tickets", "value": 12, "change": "+2 today", "tone": "warning"}),
        ),
    ]
}

fn charts() -> Vec<Entry> {
    vec![
        entry::<BarChartFields>(
            "bar_chart",
            Group::Charts,
            "Shows horizontal bars. The longest bar shows the largest value. Values are 0 or more.",
            json!({"type": "bar_chart", "title": "Build time by crate", "unit": " s", "bars": [
                {"label": "editor", "value": 48},
                {"label": "terminal", "value": 31},
                {"label": "gpui", "value": 77, "tone": "warning"}
            ]}),
        ),
        entry::<SparklineFields>(
            "sparkline",
            Group::Charts,
            "Shows a line through a series of values, oldest first, with no axes.",
            json!({"type": "sparkline", "label": "Requests", "values": [3, 5, 4, 8, 13, 9, 15, 21]}),
        ),
    ]
}

fn status() -> Vec<Entry> {
    vec![
        entry::<BadgeFields>(
            "badge",
            Group::Status,
            "Shows a short word in a small chip.",
            json!({"type": "row", "children": [
                {"type": "badge", "text": "passed", "tone": "success"},
                {"type": "badge", "text": "flaky", "tone": "warning"},
                {"type": "badge", "text": "failed", "tone": "error"}
            ]}),
        ),
        entry::<ProgressFields>(
            "progress",
            Group::Status,
            "Shows a bar that fills from 0 to `max`, with a label and the percent.",
            json!({"type": "progress", "label": "Indexing", "value": 64, "max": 100}),
        ),
        entry::<CalloutFields>(
            "callout",
            Group::Status,
            "Shows a message in a colored box. The tone sets the color and the icon.",
            json!({"type": "callout", "tone": "warning", "title": "Migration pending", "text": "Run the migration before the next deploy."}),
        ),
    ]
}

/// One entry, its schema the fields' with `type` (and `children` for a container) added.
fn entry<T: JsonSchema>(
    name: &'static str,
    group: Group,
    description: &'static str,
    example: Value,
) -> Entry {
    let mut schema = schema_for!(T).to_value();
    if let Value::Object(object) = &mut schema {
        object.remove("$schema");
        object.insert("title".into(), Value::String(name.to_owned()));
        object.insert("description".into(), Value::String(description.to_owned()));
        let properties = object
            .entry("properties")
            .or_insert_with(|| Value::Object(Map::new()));
        if let Value::Object(properties) = properties {
            properties.insert("type".into(), json!({"const": name}));
            if CONTAINERS.contains(&name) {
                properties.insert(
                    "children".into(),
                    json!({
                        "type": "array",
                        "description": "The components this component holds.",
                        "items": {"type": "object", "description": "A component."}
                    }),
                );
            }
        }
        let required = object
            .entry("required")
            .or_insert_with(|| Value::Array(Vec::new()));
        if let Value::Array(required) = required {
            required.insert(0, Value::String("type".into()));
        }
    }
    Entry {
        name,
        group,
        description,
        schema,
        example,
    }
}
