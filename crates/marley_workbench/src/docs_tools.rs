//! The docs tools of Marley's MCP server (#681): `docs_search` and `docs_read`.
//!
//! They read Zed's docs and Marley's guide as this build ships them, so an agent explains the
//! Marley that runs and no other version.
//!
//! A release build embeds `docs/src` and `docs/marley/guide.md` (`util::fs_embed!`); a dev build
//! reads them from the checkout. The pages split into sections at their headings, once, off the
//! main thread. Search is by words: a section ranks by how many of the query's words it holds, then
//! by how often, a heading's hit counting more. Zed's docs carry `{#kb action}` and
//! `{#action action}` placeholders, which Zed's docs preprocessor fills when it builds the site;
//! they are filled here from this Marley's keymap and its palette names.

use std::sync::OnceLock;

use collections::HashMap;
use gpui::App;
use marley_mcp::{AppCall, Refusal, ToolAnswer};
use serde_json::{Value, json};

use crate::settings_tools::{BoundKey, palette_name, query_words};

util::fs_embed! {
    struct DocsBundle,
    crate_relative = "../../docs",
    root_relative = "docs",
    include = ["src/**/*.md", "marley/guide.md"],
}

/// The most sections `docs_search` answers.
const MAX_RESULTS: usize = 10;

/// How much a hit in a section's heading counts against one in its text.
const HEADING_WEIGHT: usize = 5;

/// The most hits of one word a section's text counts.
const MAX_HITS_PER_WORD: usize = 20;

/// About how many characters a snippet holds.
const SNIPPET_CHARACTERS: usize = 200;

/// The most headings a read lists, or a refusal's next steps name.
const MAX_HEADINGS: usize = 60;

/// One page of the bundle.
struct DocPage {
    /// What `docs_read` takes: `zed/<path under docs/src>` or `marley/guide.md`.
    name: String,
    lines: Vec<String>,
    /// Its sections, in order.
    sections: Vec<Section>,
}

/// A page's text from one heading to the next.
struct Section {
    heading: String,
    /// The section's lines in its page, its heading's line first.
    start: usize,
    end: usize,
    heading_lower: String,
    text_lower: String,
}

/// Every page, split once.
struct DocsIndex {
    pages: Vec<DocPage>,
}

/// The index, built on first use.
fn index() -> &'static DocsIndex {
    static INDEX: OnceLock<DocsIndex> = OnceLock::new();
    INDEX.get_or_init(build_index)
}

fn build_index() -> DocsIndex {
    let mut pages: Vec<DocPage> = DocsBundle::iter()
        .filter_map(|path| {
            let file = DocsBundle::get(&path)?;
            let text = String::from_utf8_lossy(&file.data).into_owned();
            let name = path
                .strip_prefix("src/")
                .map_or_else(|| path.to_string(), |rest| format!("zed/{rest}"));
            Some(split_page(name, &text))
        })
        .collect();
    pages.sort_by(|left, right| left.name.cmp(&right.name));
    DocsIndex { pages }
}

/// A page's lines and its sections: one starts at each heading of levels 1 to 4 outside a code
/// fence, and the lines before the first heading are a section of their own.
fn split_page(name: String, text: &str) -> DocPage {
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let mut starts: Vec<(usize, String)> = Vec::new();
    let mut fenced = false;
    for (number, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if let Some(heading) = heading_of(line) {
            starts.push((number, heading));
        }
    }
    if starts.first().is_none_or(|(first, _)| *first > 0) {
        starts.insert(0, (0, "(top)".to_string()));
    }
    let sections = starts
        .iter()
        .enumerate()
        .map(|(position, (start, heading))| {
            let end = starts
                .get(position + 1)
                .map_or(lines.len(), |(next, _)| *next);
            let text_lower = lines
                .get(*start..end)
                .unwrap_or_default()
                .join("\n")
                .to_lowercase();
            Section {
                heading: heading.clone(),
                start: *start,
                end,
                heading_lower: heading.to_lowercase(),
                text_lower,
            }
        })
        .collect();
    DocPage {
        name,
        lines,
        sections,
    }
}

/// A Markdown heading's text, for `#` to `####`, its `{#anchor}` dropped.
fn heading_of(line: &str) -> Option<String> {
    let level = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=4).contains(&level) {
        return None;
    }
    let rest = line.get(level..)?.strip_prefix(' ')?;
    let rest = rest
        .find(" {#")
        .map_or(rest, |anchor| rest.get(..anchor).unwrap_or(rest));
    let heading = rest.trim().to_string();
    (!heading.is_empty()).then_some(heading)
}

/// A heading as a URL fragment: lower case, words joined by `-`.
fn slug(heading: &str) -> String {
    heading
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Answers `docs_search` or `docs_read` off the main thread, the keymap read here for the
/// placeholders.
pub(crate) fn answer(call: AppCall, cx: &App) {
    let bound = crate::settings_tools::bound_keys(cx);
    cx.background_executor()
        .spawn(futures::future::lazy(move |_| {
            let result = match call.tool.as_str() {
                "docs_search" => search(&call.arguments),
                "docs_read" => read(&call.arguments, &bound),
                other => Err(Refusal::from(format!(
                    "Marley answers no tool named {other}"
                ))),
            };
            call.answer(result);
        }))
        .detach();
}

/// `docs_search`: the sections that hold the most of the query's words, then the most hits.
fn search(arguments: &Value) -> Result<ToolAnswer, Refusal> {
    let query = arguments
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let words = query_words(query);
    if words.is_empty() {
        return Err(
            Refusal::new("bad_argument", "`query` holds no word to look for")
                .next("give `query`, words such as \"terminal font size\""),
        );
    }
    let mut scored: Vec<(usize, usize, &DocPage, &Section)> = index()
        .pages
        .iter()
        .flat_map(|page| page.sections.iter().map(move |section| (page, section)))
        .filter_map(|(page, section)| {
            let mut held = 0;
            let mut hits = 0;
            for word in &words {
                let in_heading = section.heading_lower.matches(word.as_str()).count();
                let in_text = section
                    .text_lower
                    .matches(word.as_str())
                    .count()
                    .min(MAX_HITS_PER_WORD);
                if in_heading + in_text > 0 {
                    held += 1;
                    hits += in_heading * HEADING_WEIGHT + in_text;
                }
            }
            (held > 0).then_some((held, hits, page, section))
        })
        .collect();
    scored.sort_by(|left, right| right.0.cmp(&left.0).then(right.1.cmp(&left.1)));
    let matched = scored.len();
    let results: Vec<Value> = scored
        .into_iter()
        .take(MAX_RESULTS)
        .map(|(held, _, page, section)| {
            json!({
                "page": page.name,
                "heading": section.heading,
                "snippet": snippet(page, section, &words),
                "words": held,
            })
        })
        .collect();
    Ok(ToolAnswer {
        structured: json!({ "query": query, "matched": matched, "results": results }),
        text: None,
        image: None,
    })
}

/// About [`SNIPPET_CHARACTERS`] of a section's text around the first word of the query it holds,
/// on one line.
fn snippet(page: &DocPage, section: &Section, words: &[String]) -> String {
    let text = page
        .lines
        .get(section.start..section.end)
        .unwrap_or_default()
        .join(" ");
    let lower = text.to_lowercase();
    let at = words
        .iter()
        .find_map(|word| lower.find(word.as_str()))
        .unwrap_or(0);
    let characters: Vec<(usize, char)> = text.char_indices().collect();
    let center = characters
        .iter()
        .position(|(offset, _)| *offset >= at)
        .unwrap_or(0);
    let from = center.saturating_sub(SNIPPET_CHARACTERS / 3);
    let to = (from + SNIPPET_CHARACTERS).min(characters.len());
    let piece: String = characters
        .get(from..to)
        .unwrap_or_default()
        .iter()
        .map(|(_, character)| *character)
        .collect();
    let piece = piece.split_whitespace().collect::<Vec<_>>().join(" ");
    let before = if from > 0 { "…" } else { "" };
    let after = if to < characters.len() { "…" } else { "" };
    format!("{before}{piece}{after}")
}

/// `docs_read`: a page, or one section of it, its placeholders filled, a page at a time from the
/// top.
fn read(
    arguments: &Value,
    bound: &HashMap<&'static str, Vec<BoundKey>>,
) -> Result<ToolAnswer, Refusal> {
    let asked = arguments
        .get("page")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim();
    let pages = &index().pages;
    let Some(page) = pages.iter().find(|page| page.name == asked).or_else(|| {
        pages
            .iter()
            .find(|page| page.name == format!("zed/{asked}"))
    }) else {
        let stem = asked
            .rsplit('/')
            .next()
            .unwrap_or(asked)
            .trim_end_matches(".md")
            .to_lowercase();
        let close: Vec<&str> = pages
            .iter()
            .filter(|page| !stem.is_empty() && page.name.to_lowercase().contains(&stem))
            .take(5)
            .map(|page| page.name.as_str())
            .collect();
        let refusal = Refusal::new("no_doc", format!("no page is named `{asked}`"))
            .next("docs_search finds the pages that hold your words, with their names");
        return Err(if close.is_empty() {
            refusal
        } else {
            refusal.next(format!("pages with a name like it: {}", close.join(", ")))
        });
    };
    let heading = arguments
        .get("heading")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|heading| !heading.is_empty());
    let (start, end) = match heading {
        None => (0, page.lines.len()),
        Some(heading) => {
            let wanted = heading.to_lowercase();
            let section = page.sections.iter().find(|section| {
                section.heading_lower == wanted || slug(&section.heading) == slug(heading)
            });
            match section {
                Some(section) => (section.start, section.end),
                None => {
                    return Err(Refusal::new(
                        "no_section",
                        format!("`{}` has no heading `{heading}`", page.name),
                    )
                    .next(format!("its headings: {}", headings(page).join(" | "))));
                }
            }
        }
    };
    let text = page
        .lines
        .get(start..end)
        .unwrap_or_default()
        .iter()
        .map(|line| fill_placeholders(line, bound))
        .collect::<Vec<_>>()
        .join("\n");
    let after = match arguments.get("after") {
        None | Some(Value::Null) => None,
        Some(after) => Some(
            after
                .as_u64()
                .and_then(|after| usize::try_from(after).ok())
                .ok_or_else(|| {
                    Refusal::new("bad_argument", "`after` is not a line number")
                        .next("pass a page's `next` as `after`, or leave it out")
                })?,
        ),
    };
    let shown = crate::mcp::page_from(&text, after)?;
    let mut structured = json!({
        "page": page.name,
        "heading": heading,
        "headings": if heading.is_none() { headings(page) } else { Vec::new() },
    });
    shown.fill_forward(&mut structured);
    Ok(ToolAnswer {
        structured,
        text: Some(shown.text_block_forward()),
        image: None,
    })
}

/// A page's headings, in order, the top's left out.
fn headings(page: &DocPage) -> Vec<String> {
    page.sections
        .iter()
        .filter(|section| section.heading != "(top)")
        .take(MAX_HEADINGS)
        .map(|section| section.heading.clone())
        .collect()
}

/// `line` with Zed's docs placeholders filled: `{#kb action}` becomes the keys bound to it in this
/// Marley ("no key" when none is), `{#action action}` its palette name.
fn fill_placeholders(line: &str, bound: &HashMap<&'static str, Vec<BoundKey>>) -> String {
    let mut filled = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(open) = rest.find("{#") {
        let Some(close) = rest.get(open..).and_then(|tail| tail.find('}')) else {
            break;
        };
        let inner = rest.get(open + 2..open + close).unwrap_or_default();
        let (kind, name) = inner.split_once(' ').unwrap_or((inner, ""));
        let name = name.trim();
        let replacement = match kind {
            "kb" => Some(
                bound
                    .get(name)
                    .and_then(|keys| {
                        keys.iter()
                            .find(|key| key.context.is_none())
                            .or_else(|| keys.first())
                    })
                    .map_or_else(|| "no key".to_string(), |key| key.keystrokes.clone()),
            ),
            "action" => Some(palette_name(name)),
            _ => None,
        };
        filled.push_str(rest.get(..open).unwrap_or_default());
        match replacement {
            Some(replacement) => filled.push_str(&replacement),
            None => filled.push_str(rest.get(open..=open + close).unwrap_or_default()),
        }
        rest = rest.get(open + close + 1..).unwrap_or_default();
    }
    filled.push_str(rest);
    filled
}
