//! Following a script's source map (#497): from a place in the script the page runs to the file
//! and line it was built from.
//!
//! A map is Source Map v3 (ECMA-426): its `sources`, resolved against its `sourceRoot` and its
//! own URL, and its `mappings`, base64 VLQ segments in one run per generated line. Only the
//! mappings are needed, so they stay text and are scanned up to the place asked for. An index
//! map's first level of `sections` is read, each section with its offset.

use std::borrow::Cow;

use base64::Engine as _;
use serde::Deserialize;
use serde_json::{Value, json};
use url::Url;
use util::ResultExt as _;

use crate::cdp::CdpError;
use crate::page::Page;

/// The largest map read, in bytes.
const MAP_CAP: usize = 32 * 1024 * 1024;

/// How much of a map one `IO.read` asks for.
const READ_CHUNK: u64 = 1024 * 1024;

/// Where a script's source map is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapLocation {
    /// A map the script carries in a `data:` URL, decoded.
    Inline(String),
    /// A map to load, at this URL.
    Remote(String),
}

/// What went wrong reading a map.
#[derive(Debug, thiserror::Error)]
pub enum SourceMapError {
    /// The map is not JSON of the shape a map has.
    #[error("the source map does not parse: {0}")]
    Json(#[from] serde_json::Error),
    /// The map is of a version other than 3.
    #[error("the source map is version {0:?}, not 3")]
    Version(Option<u64>),
}

/// A place in a script's original source, as its map gives it. Lines and columns count from 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OriginalPosition {
    /// The source, resolved against the map's `sourceRoot` and URL.
    pub source: String,
    /// Its line.
    pub line: u32,
    /// Its column.
    pub column: u32,
}

/// The path a map's source names, as the workspace looks for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePath {
    /// Its components, `.` dropped and `..` applied.
    pub components: Vec<String>,
    /// Whether it names a path on the machine: a `file:` URL, or Vite's `/@fs/`.
    pub absolute: bool,
}

/// A parsed map: one section for a plain map, or an index map's sections.
#[derive(Debug, Clone)]
pub struct SourceMap {
    sections: Vec<Section>,
}

/// A run of mappings and the sources they name, from a place in the generated script.
#[derive(Debug, Clone)]
struct Section {
    line: u32,
    column: u32,
    sources: Vec<Option<String>>,
    mappings: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawMap {
    version: Option<u64>,
    #[serde(default)]
    sources: Vec<Option<String>>,
    source_root: Option<String>,
    #[serde(default)]
    mappings: String,
    sections: Option<Vec<RawSection>>,
}

#[derive(Deserialize)]
struct RawSection {
    offset: RawOffset,
    /// A section's own map; a section that names a `url` instead is not read.
    map: Option<RawMap>,
}

#[derive(Deserialize)]
struct RawOffset {
    line: u32,
    column: u32,
}

/// A mapping that names a source: which source, and the place in it.
#[derive(Debug, Clone, Copy)]
struct Mapped {
    source: usize,
    line: u32,
    column: u32,
}

/// Where a script's source map is, from the script's URL and the `sourceMapURL` it names: a
/// `data:` URL's map is decoded here, and any other URL is joined to the script's.
#[must_use]
pub fn map_location(script_url: &str, source_map_url: &str) -> Option<MapLocation> {
    if let Some(data) = source_map_url.strip_prefix("data:") {
        let (header, payload) = data.split_once(',')?;
        let text = if header.split(';').any(|part| part == "base64") {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(payload.trim())
                .log_err()?;
            String::from_utf8(bytes).log_err()?
        } else {
            percent_encoding::percent_decode_str(payload)
                .decode_utf8()
                .log_err()?
                .into_owned()
        };
        return Some(MapLocation::Inline(text));
    }
    let base = Url::parse(script_url).ok()?;
    base.join(source_map_url)
        .ok()
        .map(|url| MapLocation::Remote(url.into()))
}

/// The path a map's source names: its scheme and host taken off (`webpack://app/`,
/// `http://localhost:5173/`), each component decoded, `.` dropped and `..` applied.
#[must_use]
pub fn source_path(source: &str) -> SourcePath {
    let (path, absolute) = match Url::parse(source) {
        Ok(url) if url.scheme() == "file" => (url.path().to_string(), true),
        Ok(url) => url.path().strip_prefix("/@fs/").map_or_else(
            || (url.path().to_string(), false),
            |rest| (format!("/{rest}"), true),
        ),
        Err(_) => (source.to_string(), source.starts_with('/')),
    };
    let mut components = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            part => components.push(
                percent_encoding::percent_decode_str(part)
                    .decode_utf8()
                    .map_or_else(|_| part.to_string(), Cow::into_owned),
            ),
        }
    }
    SourcePath {
        components,
        absolute,
    }
}

impl SourceMap {
    /// Parses a map's `text`; its sources resolve against `base`, the map's URL, or the script's
    /// for a map the script carries.
    ///
    /// # Errors
    ///
    /// When the text is not a version 3 map.
    pub fn parse(text: &str, base: &str) -> Result<Self, SourceMapError> {
        let raw: RawMap = serde_json::from_str(text)?;
        if raw.version != Some(3) {
            return Err(SourceMapError::Version(raw.version));
        }
        let base = Url::parse(base).ok();
        let sections = match raw.sections {
            Some(sections) => sections
                .into_iter()
                .filter_map(|section| {
                    let map = section.map?;
                    Some(Section::new(
                        section.offset.line,
                        section.offset.column,
                        map,
                        base.as_ref(),
                    ))
                })
                .collect(),
            None => vec![Section::new(0, 0, raw, base.as_ref())],
        };
        Ok(Self { sections })
    }

    /// Where the place at `line` and `column` of the generated script comes from: the last
    /// mapping at or before it that names a source, as Chromium's own tools find it.
    #[must_use]
    pub fn original(&self, line: u32, column: u32) -> Option<OriginalPosition> {
        let section = self
            .sections
            .iter()
            .rev()
            .find(|section| (section.line, section.column) <= (line, column))?;
        let line_in_section = line - section.line;
        let column_in_section = if line_in_section == 0 {
            column - section.column
        } else {
            column
        };
        let mapped = scan(&section.mappings, line_in_section, column_in_section)?;
        let source = section.sources.get(mapped.source)?.clone()?;
        Some(OriginalPosition {
            source,
            line: mapped.line,
            column: mapped.column,
        })
    }
}

impl Section {
    fn new(line: u32, column: u32, map: RawMap, base: Option<&Url>) -> Self {
        let root = map.source_root.as_deref().filter(|root| !root.is_empty());
        let sources = map
            .sources
            .into_iter()
            .map(|source| source.map(|source| resolve_source(&source, root, base)))
            .collect();
        Self {
            line,
            column,
            sources,
            mappings: map.mappings,
        }
    }
}

/// A source's URL: `root` before it unless it is a URL of its own, then resolved against `base`.
fn resolve_source(source: &str, root: Option<&str>, base: Option<&Url>) -> String {
    let joined = match root {
        Some(root) if Url::parse(source).is_err() => {
            if root.ends_with('/') {
                format!("{root}{source}")
            } else {
                format!("{root}/{source}")
            }
        }
        _ => source.to_string(),
    };
    base.and_then(|base| base.join(&joined).ok())
        .map_or(joined, String::from)
}

/// The last mapping at or before `line` and `column` that names a source, the mappings decoded
/// as they are scanned; `None` when there is none, or when they stop parsing before it.
fn scan(mappings: &str, line: u32, column: u32) -> Option<Mapped> {
    let mut source: i64 = 0;
    let mut original_line: i64 = 0;
    let mut original_column: i64 = 0;
    let mut found = None;
    for (generated_line, segments) in (0u32..).zip(mappings.split(';')) {
        if generated_line > line {
            break;
        }
        let mut generated_column: i64 = 0;
        for segment in segments.split(',').filter(|segment| !segment.is_empty()) {
            let (values, count) = vlq(segment)?;
            let [
                column_delta,
                source_delta,
                line_delta,
                original_column_delta,
                _,
            ] = values;
            generated_column += column_delta;
            if generated_line == line && generated_column > i64::from(column) {
                return found;
            }
            // A segment of one value maps its columns to no source.
            if count == 1 {
                found = None;
                continue;
            }
            source += source_delta;
            original_line += line_delta;
            original_column += original_column_delta;
            found = Some(Mapped {
                source: usize::try_from(source).ok()?,
                line: u32::try_from(original_line).ok()?,
                column: u32::try_from(original_column).ok()?,
            });
        }
    }
    found
}

/// A segment's base64 VLQ values, at most five, and how many it held; `None` when it is not
/// base64 VLQ.
fn vlq(segment: &str) -> Option<([i64; 5], usize)> {
    let mut values = [0i64; 5];
    let mut count = 0;
    let mut value: i64 = 0;
    let mut shift = 0u32;
    for byte in segment.bytes() {
        let digit = base64_digit(byte)?;
        value |= i64::from(digit & 31) << shift;
        if digit & 32 == 0 {
            let magnitude = value >> 1;
            let decoded = if value & 1 == 1 {
                -magnitude
            } else {
                magnitude
            };
            if let Some(slot) = values.get_mut(count) {
                *slot = decoded;
            }
            count += 1;
            value = 0;
            shift = 0;
        } else {
            shift += 5;
            // More than 64 bits: not a map any tool writes.
            if shift > 55 {
                return None;
            }
        }
    }
    (shift == 0 && matches!(count, 1 | 4 | 5)).then_some((values, count))
}

const fn base64_digit(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

impl Page {
    /// Loads `url` in the page's main frame the way Chromium's own tools load a source map
    /// (`Network.loadNetworkResource`), and reads it to its end.
    ///
    /// # Errors
    ///
    /// When a call fails, the load does not succeed, or the resource is larger than 32 MiB or
    /// not UTF-8.
    pub async fn load_resource(&self, url: &str) -> Result<String, CdpError> {
        let answer = self
            .call(
                "Network.loadNetworkResource",
                json!({
                    "frameId": self.target_id(),
                    "url": url,
                    "options": { "disableCache": false, "includeCredentials": false },
                }),
            )
            .await?;
        let resource = answer.get("resource").cloned().unwrap_or_default();
        if resource.get("success").and_then(Value::as_bool) != Some(true) {
            let reason = resource
                .get("httpStatusCode")
                .and_then(Value::as_i64)
                .map_or_else(
                    || {
                        let error = resource
                            .get("netErrorName")
                            .and_then(Value::as_str)
                            .unwrap_or("no answer");
                        format!("{url} did not load: {error}")
                    },
                    |status| format!("{url} answered {status}"),
                );
            return Err(CdpError::Unexpected(reason));
        }
        let stream = resource
            .get("stream")
            .and_then(Value::as_str)
            .ok_or_else(|| CdpError::Unexpected(format!("{url} loaded with no stream")))?;
        let read = self.read_stream(stream).await;
        self.call("IO.close", json!({ "handle": stream }))
            .await
            .log_err();
        read
    }

    async fn read_stream(&self, handle: &str) -> Result<String, CdpError> {
        let mut bytes = Vec::new();
        loop {
            let chunk = self
                .call("IO.read", json!({ "handle": handle, "size": READ_CHUNK }))
                .await?;
            let data = chunk
                .get("data")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if chunk.get("base64Encoded").and_then(Value::as_bool) == Some(true) {
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(data)
                    .map_err(|error| CdpError::Unexpected(format!("a map's bytes: {error}")))?;
                bytes.extend(decoded);
            } else {
                bytes.extend_from_slice(data.as_bytes());
            }
            if bytes.len() > MAP_CAP {
                return Err(CdpError::Unexpected(
                    "the source map is larger than 32 MiB".to_string(),
                ));
            }
            if data.is_empty() || chunk.get("eof").and_then(Value::as_bool) != Some(false) {
                break;
            }
        }
        String::from_utf8(bytes)
            .map_err(|_| CdpError::Unexpected("the source map is not UTF-8".to_string()))
    }
}
