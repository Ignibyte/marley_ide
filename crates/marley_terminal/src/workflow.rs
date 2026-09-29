//! PURE — a command saved as a workflow, with `{{name}}` parameters (#558).
//!
//! [`guess`] turns a block's command into a template once, when the user saves it: numbers, URLs,
//! the block's git branch and paths that exist become placeholders with the token as the default.
//! [`params_of`] lists a template's parameters and [`substitute`] fills them when the workflow
//! runs. `substitute` and `params_of` come from the gpui-era app's workflows (#204), Marley's own.

use std::collections::BTreeMap;
use std::fmt;

/// A template's parameter with no value when it was filled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingParameter(pub String);

impl fmt::Display for MissingParameter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "no value for {{{{{}}}}}", self.0)
    }
}

impl std::error::Error for MissingParameter {}

/// Fills `template`'s `{{name}}` tokens from `values`. A single brace, or a `{{` with no `}}` after
/// it, is left as it is.
///
/// # Errors
///
/// [`MissingParameter`] for the first token `values` has no value for.
pub fn substitute(
    template: &str,
    values: &BTreeMap<String, String>,
) -> Result<String, MissingParameter> {
    let mut filled = String::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        filled.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        if let Some(end) = after.find("}}") {
            let name = after[..end].trim();
            let value = values
                .get(name)
                .ok_or_else(|| MissingParameter(name.to_string()))?;
            filled.push_str(value);
            rest = &after[end + 2..];
        } else {
            filled.push_str("{{");
            rest = after;
        }
    }
    filled.push_str(rest);
    Ok(filled)
}

/// The distinct `{{name}}` parameters of `template`, first seen first; a blank one is left out.
#[must_use]
pub fn params_of(template: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            break;
        };
        let name = after[..end].trim();
        if !name.is_empty() && !names.iter().any(|known| known == name) {
            names.push(name.to_string());
        }
        rest = &after[end + 2..];
    }
    names
}

/// Whether `name` can name a parameter, as Warp's arguments are named: letters, digits, `-` and
/// `_`, not starting with a digit, and never Zed's `ZED_` prefix, whose variables Zed checks.
#[must_use]
pub fn is_name(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_' || first == '-')
        && characters.all(|character| character.is_ascii_alphanumeric() || "-_".contains(character))
        && !name.starts_with("ZED_")
}

/// A parameter [`guess`] made: its name and the token it replaced, the default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    /// The parameter's name.
    pub name: String,
    /// The token it replaced.
    pub default: String,
}

/// A command with its guessed parameters in place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guessed {
    /// The command, each guessed token replaced by `{{name}}`.
    pub command: String,
    /// The parameters, in the command's order.
    pub parameters: Vec<Parameter>,
}

/// Guesses the parameters of `command`; its first word is never one.
///
/// A number is `port` after `-p` or `--port`, after a `:` in the token, or from 1024 to 65535 with
/// no other flag before it, else `number`; a URL is `url`, the block's git branch `branch`, and a
/// token `exists` says names something `path`. A second guess of the same kind is `port2`, then
/// `port3`. Quoted tokens are left alone.
#[must_use]
pub fn guess(command: &str, branch: Option<&str>, exists: impl Fn(&str) -> bool) -> Guessed {
    let tokens = spans(command);
    let mut replacements: Vec<(std::ops::Range<usize>, &'static str)> = Vec::new();
    for (place, span) in tokens.iter().enumerate().skip(1) {
        let token = &command[span.clone()];
        if token.starts_with(['"', '\'']) {
            continue;
        }
        let previous = place
            .checked_sub(1)
            .and_then(|before| tokens.get(before))
            .map(|before| &command[before.clone()]);
        if let Some(colon) = token.rfind(':')
            && colon + 1 < token.len()
            && token[colon + 1..].bytes().all(|byte| byte.is_ascii_digit())
            && !token.starts_with("http")
        {
            replacements.push((span.start + colon + 1..span.end, "port"));
        } else if token.bytes().all(|byte| byte.is_ascii_digit()) {
            let port_flag = matches!(previous, Some("-p" | "-P" | "--port"));
            let other_flag = previous.is_some_and(|word| word.starts_with('-')) && !port_flag;
            let port_range = token
                .parse::<u32>()
                .is_ok_and(|number| (1024..=65535).contains(&number));
            let name = if port_flag || (port_range && !other_flag) {
                "port"
            } else {
                "number"
            };
            replacements.push((span.clone(), name));
        } else if token.starts_with("http://") || token.starts_with("https://") {
            replacements.push((span.clone(), "url"));
        } else if branch == Some(token) {
            replacements.push((span.clone(), "branch"));
        } else if !token.starts_with('-') && exists(token) {
            replacements.push((span.clone(), "path"));
        }
    }
    let mut guessed = String::new();
    let mut parameters: Vec<Parameter> = Vec::new();
    let mut end = 0;
    for (range, kind) in replacements {
        guessed.push_str(&command[end..range.start]);
        let taken = parameters
            .iter()
            .filter(|parameter| parameter.name.trim_end_matches(char::is_numeric) == kind)
            .count();
        let name = if taken == 0 {
            kind.to_string()
        } else {
            format!("{kind}{}", taken + 1)
        };
        guessed.push_str("{{");
        guessed.push_str(&name);
        guessed.push_str("}}");
        parameters.push(Parameter {
            name,
            default: command[range.clone()].to_string(),
        });
        end = range.end;
    }
    guessed.push_str(&command[end..]);
    Guessed {
        command: guessed,
        parameters,
    }
}

/// The byte ranges of `command`'s words, split on whitespace.
fn spans(command: &str) -> Vec<std::ops::Range<usize>> {
    let mut spans = Vec::new();
    let mut start = None;
    for (at, character) in command.char_indices() {
        match (character.is_whitespace(), start) {
            (true, Some(from)) => {
                spans.push(from..at);
                start = None;
            }
            (false, None) => start = Some(at),
            _ => {}
        }
    }
    if let Some(from) = start {
        spans.push(from..command.len());
    }
    spans
}
