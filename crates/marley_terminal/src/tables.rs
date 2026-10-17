//! PURE, gpui-free — tables in a block's output (#803).
//!
//! A finished block's text reads as a table when it is one JSON array of objects or one object
//! per line, a markdown table, or delimited text: a comma, a tab, a semicolon or a pipe between
//! fields, with RFC 4180's quoting. [`detect`] reads a capped sample, for the block's chip;
//! [`parse`] reads the whole text into cells when the user opens the table. [`tabular_file`]
//! names the file a plain read of one tabular file prints, which opens whole in place of the
//! block's text.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// The most of a block's text [`detect`] reads.
const SAMPLE_BYTES: usize = 1 << 20;

/// The most lines it reads.
pub const SAMPLE_LINES: usize = 2_000;

/// The delimiters delimited text may use; of two that fit as well, the first wins.
const DELIMITERS: [char; 4] = [',', '\t', ';', '|'];

/// The share of records, in percent, that must have the commonest field count.
const CONSISTENT_PERCENT: usize = 95;

/// The programs whose plain read of one file prints the file.
const READERS: [&str; 5] = ["cat", "head", "tail", "bat", "batcat"];

/// The options of `head`, `tail` and `bat` whose value is the next word.
const VALUE_OPTIONS: [&str; 14] = [
    "-n",
    "-c",
    "--lines",
    "--bytes",
    "-l",
    "--language",
    "-r",
    "--line-range",
    "-H",
    "--highlight-line",
    "--style",
    "--theme",
    "-m",
    "--map-syntax",
];

/// The extensions of the files Zed's table preview reads.
const TABULAR: [&str; 5] = ["csv", "tsv", "psv", "ssv", "jsonl"];

/// What a table was read from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableKind {
    /// Delimited text, with its delimiter.
    Delimited(char),
    /// One JSON array of objects.
    JsonArray,
    /// One JSON object on each line.
    JsonLines,
    /// A markdown table.
    Markdown,
}

/// What [`detect`] found in a sample: what a block's chip says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableShape {
    /// What the table was read from.
    pub kind: TableKind,
    /// Its columns.
    pub columns: usize,
    /// Its rows under the header, in the sample.
    pub rows: usize,
    /// Whether the sample stopped before the text ended, so more rows follow.
    pub more: bool,
}

/// A table's cells: the header and the rows, every row as wide as the header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    /// What the table was read from.
    pub kind: TableKind,
    /// The column names.
    pub headers: Vec<String>,
    /// The rows, each padded with empty cells to the header's width.
    pub rows: Vec<Vec<String>>,
}

/// The table in the first 1 MB and [`SAMPLE_LINES`] lines of `text`, or none.
#[must_use]
pub fn detect(text: &str) -> Option<TableShape> {
    let (sample, more) = sample_of(text);
    // A JSON array is read whole while it fits the byte cap: its sample would not parse.
    let table = if more && text.len() <= SAMPLE_BYTES && text.trim_start().starts_with('[') {
        read(text)
    } else {
        read(sample)
    }?;
    Some(TableShape {
        kind: table.kind,
        columns: table.headers.len(),
        rows: table.rows.len(),
        more: more && table.kind != TableKind::JsonArray,
    })
}

/// The whole of `text` as a table, by [`detect`]'s rules, or none.
#[must_use]
pub fn parse(text: &str) -> Option<Table> {
    read(text)
}

/// The tabular file that `command`, run in `folder`, prints.
///
/// The file is a `.csv`, `.tsv`, `.psv`, `.ssv` or `.jsonl`, and the command a `cat`, `head`,
/// `tail`, `bat` or `batcat` of that one file, with no pipe, redirect, list or substitution.
/// Whether the file exists is the caller's to check.
#[must_use]
pub fn tabular_file(command: &str, folder: &Path) -> Option<PathBuf> {
    if command.contains(['|', '>', '<', ';', '&', '`', '$', '(', ')', '*', '?']) {
        return None;
    }
    let mut words = command.split_whitespace();
    let program = Path::new(words.next()?).file_name()?.to_str()?;
    if !READERS.contains(&program) {
        return None;
    }
    let mut files = Vec::new();
    let mut takes_value = false;
    for word in words {
        if takes_value {
            takes_value = false;
        } else if word.len() > 1 && word.starts_with('-') {
            takes_value = VALUE_OPTIONS.contains(&word);
        } else {
            files.push(unquoted(word));
        }
    }
    let [file] = files.as_slice() else {
        return None;
    };
    let path = folder.join(file);
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    TABULAR.contains(&extension.as_str()).then_some(path)
}

/// `word` without the quotes around it.
fn unquoted(word: &str) -> &str {
    ['\'', '"']
        .iter()
        .find_map(|quote| word.strip_prefix(*quote)?.strip_suffix(*quote))
        .unwrap_or(word)
}

/// The first 1 MB and [`SAMPLE_LINES`] lines of `text`, cut after a whole line, and whether that
/// left any of it out.
fn sample_of(text: &str) -> (&str, bool) {
    let by_lines = text
        .match_indices('\n')
        .nth(SAMPLE_LINES - 1)
        .map_or(text.len(), |(index, _)| index);
    if by_lines <= SAMPLE_BYTES {
        return (text.get(..by_lines).unwrap_or(text), by_lines < text.len());
    }
    // The byte cap cuts through a line: the sample ends at the last line break before it.
    let end = text
        .as_bytes()
        .get(..SAMPLE_BYTES)
        .and_then(|bytes| bytes.iter().rposition(|byte| *byte == b'\n'))
        .unwrap_or(0);
    (text.get(..end).unwrap_or_default(), true)
}

/// `text` as a table: JSON first, then a markdown table, then delimited text.
fn read(text: &str) -> Option<Table> {
    let text = text.trim_start_matches(['\n', '\r']).trim_end();
    if text.is_empty() {
        return None;
    }
    json_table(text)
        .or_else(|| markdown_table(text))
        .or_else(|| delimited_table(text))
}

/// An array of at least two JSON objects, or at least two lines that are each an object: the
/// keys, in the order they first appear, are the columns.
fn json_table(text: &str) -> Option<Table> {
    let (items, kind) = if text.starts_with('[') {
        let Value::Array(items) = serde_json::from_str::<Value>(text).ok()? else {
            return None;
        };
        (items, TableKind::JsonArray)
    } else if text.starts_with('{') {
        let items = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str::<Value>(line).ok())
            .collect::<Option<Vec<_>>>()?;
        (items, TableKind::JsonLines)
    } else {
        return None;
    };
    if items.len() < 2 {
        return None;
    }
    let objects = items
        .into_iter()
        .map(|item| match item {
            Value::Object(object) => Some(object),
            _ => None,
        })
        .collect::<Option<Vec<Map<String, Value>>>>()?;
    let mut seen = HashSet::new();
    let mut headers = Vec::new();
    for key in objects.iter().flat_map(Map::keys) {
        if seen.insert(key.as_str()) {
            headers.push(key.clone());
        }
    }
    if headers.is_empty() {
        return None;
    }
    let rows = objects
        .iter()
        .map(|object| {
            headers
                .iter()
                .map(|key| object.get(key).map_or_else(String::new, json_cell))
                .collect()
        })
        .collect();
    Some(Table {
        kind,
        headers,
        rows,
    })
}

/// A JSON value as a cell's text: a string as it is, null as nothing, anything else as JSON.
fn json_cell(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// The first markdown table in `text`: a header row, a delimiter row with as many cells, and the
/// rows with a pipe that follow, at least one.
fn markdown_table(text: &str) -> Option<Table> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.windows(2).position(|pair| {
        let [header, delimiter] = pair else {
            return false;
        };
        header.contains('|')
            && is_delimiter_row(delimiter)
            && markdown_cells(header).len() == markdown_cells(delimiter).len()
    })?;
    let headers = markdown_cells(lines.get(start)?);
    let rows: Vec<Vec<String>> = lines
        .iter()
        .skip(start + 2)
        .take_while(|line| line.contains('|'))
        .map(|line| markdown_cells(line))
        .collect();
    if rows.is_empty() {
        return None;
    }
    Some(padded(TableKind::Markdown, headers, rows))
}

/// A markdown row's cells, trimmed, without its outer pipes; `\|` is a pipe inside a cell.
fn markdown_cells(line: &str) -> Vec<String> {
    let line = line.trim();
    let line = line.strip_prefix('|').unwrap_or(line);
    let line = line.strip_suffix('|').unwrap_or(line);
    let mut cells = vec![String::new()];
    let mut characters = line.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\\' && characters.next_if_eq(&'|').is_some() {
            push_char(&mut cells, '|');
        } else if character == '|' {
            cells.push(String::new());
        } else {
            push_char(&mut cells, character);
        }
    }
    cells.iter().map(|cell| cell.trim().to_string()).collect()
}

/// Adds `character` to the last cell.
fn push_char(cells: &mut [String], character: char) {
    if let Some(cell) = cells.last_mut() {
        cell.push(character);
    }
}

/// Whether `line` is a markdown table's delimiter row: pipes between cells of dashes, each with
/// an optional colon at either end.
fn is_delimiter_row(line: &str) -> bool {
    line.contains('|')
        && markdown_cells(line).iter().all(|cell| {
            let inner = cell.strip_prefix(':').unwrap_or(cell);
            let inner = inner.strip_suffix(':').unwrap_or(inner);
            !inner.is_empty() && inner.chars().all(|character| character == '-')
        })
}

/// `text` as delimited text: of the delimiters whose commonest field count, at least 2, holds for
/// at least 95 percent of at least 3 records, the one with the most columns.
fn delimited_table(text: &str) -> Option<Table> {
    let mut best: Option<(usize, char, Vec<Vec<String>>)> = None;
    for delimiter in DELIMITERS {
        let records = records(text, delimiter);
        let Some(width) = consistent_width(&records) else {
            continue;
        };
        if best.as_ref().is_none_or(|(widest, _, _)| width > *widest) {
            best = Some((width, delimiter, records));
        }
    }
    let (_, delimiter, records) = best?;
    let mut records = records.into_iter();
    let headers = records.next()?;
    if headers.iter().all(String::is_empty) {
        return None;
    }
    Some(padded(
        TableKind::Delimited(delimiter),
        headers,
        records.collect(),
    ))
}

/// The commonest field count of `records` when it is at least 2 and at least 95 percent of at
/// least 3 records have it.
fn consistent_width(records: &[Vec<String>]) -> Option<usize> {
    if records.len() < 3 {
        return None;
    }
    let mut counts: HashMap<usize, usize> = HashMap::new();
    for record in records {
        *counts.entry(record.len()).or_default() += 1;
    }
    let (width, count) = counts
        .into_iter()
        .max_by_key(|(width, count)| (*count, *width))?;
    (width >= 2 && count * 100 >= records.len() * CONSISTENT_PERCENT).then_some(width)
}

/// `text`'s records split at `delimiter`, by RFC 4180: a field that starts with a quote runs to
/// the closing quote, across delimiters and line breaks, and `""` in it is one quote. Blank lines
/// are no records, and each field is trimmed.
fn records(text: &str, delimiter: char) -> Vec<Vec<String>> {
    let mut records = Vec::new();
    let mut record = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut field_started = false;
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        if quoted {
            if character != '"' {
                field.push(character);
            } else if characters.next_if_eq(&'"').is_some() {
                field.push('"');
            } else {
                quoted = false;
            }
            continue;
        }
        match character {
            '"' if !field_started => {
                quoted = true;
                field_started = true;
            }
            '\r' if characters.peek() == Some(&'\n') => {}
            '\n' => {
                record.push(field.trim().to_string());
                field.clear();
                records.push(std::mem::take(&mut record));
                field_started = false;
            }
            character if character == delimiter => {
                record.push(field.trim().to_string());
                field.clear();
                field_started = false;
            }
            ' ' if !field_started => {}
            character => {
                field.push(character);
                field_started = true;
            }
        }
    }
    if field_started || !record.is_empty() {
        record.push(field.trim().to_string());
        records.push(record);
    }
    records.retain(|record| !matches!(record.as_slice(), [only] if only.is_empty()));
    records
}

/// A table of `headers` and `rows`, every row and the header padded with empty cells to the
/// widest.
fn padded(kind: TableKind, mut headers: Vec<String>, mut rows: Vec<Vec<String>>) -> Table {
    let width = rows
        .iter()
        .map(Vec::len)
        .chain([headers.len()])
        .max()
        .unwrap_or(0);
    headers.resize(width, String::new());
    for row in &mut rows {
        row.resize(width, String::new());
    }
    Table {
        kind,
        headers,
        rows,
    }
}
