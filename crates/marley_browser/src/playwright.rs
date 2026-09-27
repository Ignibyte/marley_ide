//! A Playwright test drafted from a recording (#506).
//!
//! The draft replays a recording's clicks, fills and key presses with the locators the recorder
//! took at each event, the first that found its element alone: `getByTestId`, `getByRole` with
//! the exact name, `getByLabel`, `getByPlaceholder`, `getByText`, then a CSS path. It sets
//! `baseURL` from the first action's origin, goes to that action's page by its path, and after
//! each navigation an action caused expects the page's URL. A secret field's fill reads an
//! environment variable through a helper that fails the test by the variable's name when it is
//! unset. What it cannot write becomes a comment saying why. It reads a saved timeline and
//! writes nothing.

use serde_json::Value;
use url::Url;

/// What marks a text the recorder or a tool cut.
const CUT: &str = " (truncated)";

/// The helper a test that fills a secret field starts with: the variable's value, or a failure
/// that names the variable.
const SECRET_HELPER: &str = "\n// A secret field's text comes from the environment, never from the recording.\n\
function secret(name) {\n  const value = process.env[name];\n  if (!value) {\n    \
throw new Error('set ' + name + ' to run this test');\n  }\n  return value;\n}\n";

/// A drafted test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    /// The test file's text.
    pub text: String,
    /// The environment variables it reads, one per secret field.
    pub env: Vec<String>,
    /// What it left out, and why, one line each.
    pub skipped: Vec<String>,
    /// The page it starts on.
    pub start: String,
}

/// Why no test could be drafted.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DraftError {
    /// The recording holds no action to replay.
    #[error(
        "the recording holds no click, fill or key press to replay: none came in its minute, or it \
         was saved before Marley recorded them"
    )]
    NoActions,
    /// The first action's page has no URL a test can go to.
    #[error("the recording's first action was on {0:?}, which a test cannot go to")]
    Start(String),
}

/// Drafts a Playwright test from `timeline`, a recording as `recorder::read_in` gives it.
///
/// # Errors
///
/// When the recording holds no action, or its first action's page is not an http or https one.
pub fn draft(timeline: &Value) -> Result<Draft, DraftError> {
    let entries = timeline
        .get("entries")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let first = entries
        .iter()
        .find(|entry| string(entry, "kind") == "action")
        .ok_or(DraftError::NoActions)?;
    let start_url = string(first, "url");
    let start = Url::parse(start_url)
        .ok()
        .filter(|url| matches!(url.scheme(), "http" | "https"))
        .ok_or_else(|| DraftError::Start(start_url.to_string()))?;
    let origin = start.origin().ascii_serialization();
    let mut lines = vec![format!(
        "await page.goto({});",
        js_string(&path_in(&start, &origin))
    )];
    let mut env = Vec::new();
    let mut skipped = Vec::new();
    let mut acted = false;
    // The last navigation since the last action: the one the action caused.
    let mut navigated: Option<String> = None;
    for entry in entries {
        match string(entry, "kind") {
            "action" => {
                if let Some(url) = navigated.take() {
                    lines.push(expect_url(&url, &origin));
                }
                acted = true;
                match action_line(entry, &mut env) {
                    Ok(line) => lines.push(line),
                    Err(reason) => {
                        lines.push(format!("// Skipped {reason}."));
                        skipped.push(reason);
                    }
                }
            }
            "navigation" if acted => navigated = Some(string(entry, "url").to_string()),
            _ => {}
        }
    }
    if let Some(url) = navigated {
        lines.push(expect_url(&url, &origin));
    }
    let id = string(timeline, "id");
    let title = string(timeline, "title");
    let name = if title.is_empty() {
        format!("recording {id}")
    } else {
        format!("{title} (recording {id})")
    };
    let helper = if env.is_empty() { "" } else { SECRET_HELPER };
    let steps: String = lines
        .iter()
        .flat_map(|line| ["  ", line.as_str(), "\n"])
        .collect();
    let text = format!(
        "import {{ test, expect }} from '@playwright/test';\n\n\
         // Drafted by Marley from recording {id}. It replays what was done in the page and\n\
         // checks where each step led; run it while {origin} serves the app.\n{helper}\n\
         test.use({{ baseURL: {base} }});\n\n\
         test({name}, async ({{ page }}) => {{\n{steps}}});\n",
        base = js_string(&origin),
        name = js_string(&name),
    );
    Ok(Draft {
        text,
        env,
        skipped,
        start: start.to_string(),
    })
}

/// The line that replays `entry`, an action, or why none can.
fn action_line(entry: &Value, env: &mut Vec<String>) -> Result<String, String> {
    let action = string(entry, "action");
    let locators = entry
        .get("locators")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let target = describe(locators);
    let locator = locators
        .iter()
        .filter(|locator| locator.get("unique").and_then(Value::as_bool) == Some(true))
        .find_map(locator_code);
    match action {
        "press" => {
            let key = js_string(string(entry, "key"));
            Ok(locator.map_or_else(
                || format!("await page.keyboard.press({key});"),
                |locator| format!("await page.{locator}.press({key});"),
            ))
        }
        "click" => locator
            .map(|locator| format!("await page.{locator}.click();"))
            .ok_or_else(|| format!("a click on {target}: no locator found it alone")),
        "fill" => {
            let locator =
                locator.ok_or_else(|| format!("a fill of {target}: no locator found it alone"))?;
            if entry.get("secret").and_then(Value::as_bool) == Some(true) {
                let name = variable_name(entry, locators);
                if !env.contains(&name) {
                    env.push(name.clone());
                }
                return Ok(format!(
                    "await page.{locator}.fill(secret({}));",
                    js_string(&name)
                ));
            }
            let text = entry
                .get("text")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("a fill of {target}: its text was not kept"))?;
            if text.ends_with(CUT) {
                return Err(format!(
                    "a fill of {target}: its text was longer than Marley keeps"
                ));
            }
            Ok(format!("await page.{locator}.fill({});", js_string(text)))
        }
        other => Err(format!("an action Marley does not replay, {other:?}")),
    }
}

/// The Playwright locator `locator` names, from `page.`; none for a kind Playwright has no
/// locator for, or a value the recorder cut.
fn locator_code(locator: &Value) -> Option<String> {
    let value = locator.get("value").and_then(Value::as_str)?;
    if value.is_empty() || value.ends_with(CUT) {
        return None;
    }
    Some(match locator.get("kind").and_then(Value::as_str)? {
        "test id" => match locator.get("attribute").and_then(Value::as_str) {
            Some("data-testid") | None => format!("getByTestId({})", js_string(value)),
            Some(attribute) => {
                let quoted = serde_json::to_string(value).ok()?;
                format!("locator({})", js_string(&format!("[{attribute}={quoted}]")))
            }
        },
        "role" => format!(
            "getByRole({}, {{ name: {}, exact: true }})",
            js_string(locator.get("role").and_then(Value::as_str)?),
            js_string(value)
        ),
        "label" => format!("getByLabel({}, {{ exact: true }})", js_string(value)),
        "placeholder" => format!("getByPlaceholder({}, {{ exact: true }})", js_string(value)),
        "text" => format!("getByText({}, {{ exact: true }})", js_string(value)),
        "css" => format!("locator({})", js_string(value)),
        _ => return None,
    })
}

/// The target of an action as a skipped line names it: its first locator.
fn describe(locators: &[Value]) -> String {
    locators.first().map_or_else(
        || "an element with no locator".to_string(),
        |locator| {
            format!(
                "the element whose {} is {:?}",
                string(locator, "kind"),
                string(locator, "value")
            )
        },
    )
}

/// The environment variable a secret field's fill reads: its name, its id or its label in upper
/// snake case, `SECRET` when it has none.
fn variable_name(entry: &Value, locators: &[Value]) -> String {
    let source = entry
        .get("field")
        .and_then(Value::as_str)
        .filter(|field| !field.is_empty())
        .or_else(|| {
            locators
                .iter()
                .find(|locator| string(locator, "kind") == "label")
                .map(|locator| string(locator, "value"))
        })
        .unwrap_or("secret");
    let mut name = String::new();
    for character in source.chars() {
        if character.is_ascii_alphanumeric() {
            name.push(character.to_ascii_uppercase());
        } else if !name.is_empty() && !name.ends_with('_') {
            name.push('_');
        }
    }
    let name = name.trim_end_matches('_').to_string();
    match name.chars().next() {
        None => "SECRET".to_string(),
        Some(first) if first.is_ascii_digit() => format!("SECRET_{name}"),
        Some(_) => name,
    }
}

/// The step that expects the page at `url`, as a path from `origin` when it is there. The
/// recorder hides secret-looking values as `…`, which no URL holds, so such a URL is expected by
/// its path alone.
fn expect_url(url: &str, origin: &str) -> String {
    let Ok(parsed) = Url::parse(url) else {
        return format!("// Skipped expecting the page's URL, {url:?}, which does not parse.");
    };
    if url.contains('\u{2026}') {
        let prefix = format!("{}{}", parsed.origin().ascii_serialization(), parsed.path());
        return format!(
            "await expect(page).toHaveURL(new RegExp({}));",
            js_string(&format!("^{}", regex_escape(&prefix)))
        );
    }
    format!(
        "await expect(page).toHaveURL({});",
        js_string(&path_in(&parsed, origin))
    )
}

/// `url` as a test addresses it with `origin` its base: the path, query and fragment when it is
/// there, else the whole URL.
fn path_in(url: &Url, origin: &str) -> String {
    if url.origin().ascii_serialization() == origin {
        url[url::Position::BeforePath..].to_string()
    } else {
        url.to_string()
    }
}

/// `text` as a JavaScript string literal in single quotes.
fn js_string(text: &str) -> String {
    let mut literal = String::with_capacity(text.len() + 2);
    literal.push('\'');
    for character in text.chars() {
        match character {
            '\\' => literal.push_str("\\\\"),
            '\'' => literal.push_str("\\'"),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\t' => literal.push_str("\\t"),
            '\u{2028}' => literal.push_str("\\u2028"),
            '\u{2029}' => literal.push_str("\\u2029"),
            control if control.is_control() => {
                literal.push_str("\\u");
                let code = u32::from(control);
                for shift in [12, 8, 4, 0] {
                    literal.push(char::from_digit((code >> shift) & 0xf, 16).unwrap_or('0'));
                }
            }
            other => literal.push(other),
        }
    }
    literal.push('\'');
    literal
}

/// `text` with a regular expression's special characters escaped.
fn regex_escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        if "\\^$.|?*+()[]{}/".contains(character) {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

/// The string at `key` of `value`, empty when there is none.
fn string<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or_default()
}
