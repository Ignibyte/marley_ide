//! Whether an agent's command at a shell's prompt runs at once or asks first (#556), by an
//! allowlist and a denylist of regular expressions, as Warp's agent profiles decide theirs.
//!
//! Each pattern is anchored at both ends and matched against the whole command and against each
//! part of it split at `|`, `||`, `&&`, `;`, `&` and newlines outside quotes. The denylist asks
//! when it matches the command or any part. The allowlist allows only when it matches every part
//! and the command holds no substitution, since `echo $(rm -rf ~)` is an `echo`. Anything else is
//! outside both lists, for the user's setting to decide.

use regex::Regex;

/// Warp's default allowlist (`agent_mode_command_execution_allowlist`).
pub const WARP_ALLOWLIST: &[&str] = &[
    r"cat(\s.*)?",
    r"echo(\s.*)?",
    r"find .*",
    r"grep(\s.*)?",
    r"ls(\s.*)?",
    r"which .*",
];

/// Warp's default denylist (`agent_mode_command_execution_denylist`).
pub const WARP_DENYLIST: &[&str] = &[
    r"bash(\s.*)?",
    r"fish(\s.*)?",
    r"pwsh(\s.*)?",
    r"sh(\s.*)?",
    r"zsh(\s.*)?",
    r"curl(\s.*)?",
    r"eval(\s.*)?",
    r"exec(\s.*)?",
    r"source(\s.*)?",
    r"wget(\s.*)?",
    r"dig(\s.*)?",
    r"nslookup(\s.*)?",
    r"host(\s.*)?",
    r"ssh(\s.*)?",
    r"scp(\s.*)?",
    r"rsync(\s.*)?",
    r"telnet(\s.*)?",
    r"rm(\s.*)?",
];

/// What the lists say of a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The allowlist matches every part: it runs at once.
    Allowed,
    /// The denylist matches the command or a part: it asks.
    Ask,
    /// Neither list settles it.
    Outside,
}

/// What `allow` and `deny` say of `command`: the denylist first, then the allowlist.
#[must_use]
pub fn verdict<S: AsRef<str>>(command: &str, allow: &[S], deny: &[S]) -> Verdict {
    let command = command.trim();
    let parts = segments(command);
    let deny = compiled(deny);
    let matches = |patterns: &[Regex], text: &str| {
        patterns.iter().any(|pattern| pattern.is_match(text.trim()))
    };
    if matches(&deny, command) || parts.iter().any(|part| matches(&deny, part)) {
        return Verdict::Ask;
    }
    let allow = compiled(allow);
    if !parts.is_empty()
        && !has_substitution(command)
        && parts.iter().all(|part| matches(&allow, part))
    {
        Verdict::Allowed
    } else {
        Verdict::Outside
    }
}

/// Each pattern anchored at both ends; one that does not compile is logged and left out, so it
/// never allows and never asks.
fn compiled<S: AsRef<str>>(patterns: &[S]) -> Vec<Regex> {
    patterns
        .iter()
        .filter_map(|pattern| {
            let pattern = pattern.as_ref();
            Regex::new(&format!("^(?:{pattern})$"))
                .inspect_err(|error| {
                    log::warn!("agent commands: the pattern {pattern:?} does not compile: {error}");
                })
                .ok()
        })
        .collect()
}

/// `command` split at `|`, `||`, `&&`, `;`, `&` and newlines outside single and double quotes, a
/// backslash escaping the next character; each part trimmed, empty parts left out.
#[must_use]
pub fn segments(command: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut characters = command.chars().peekable();
    while let Some(character) = characters.next() {
        match (quote, character) {
            (_, '\\') if quote != Some('\'') => {
                current.push(character);
                if let Some(escaped) = characters.next() {
                    current.push(escaped);
                }
            }
            (Some(open), _) if character == open => {
                quote = None;
                current.push(character);
            }
            (None, '\'' | '"') => {
                quote = Some(character);
                current.push(character);
            }
            (None, '|' | '&' | ';' | '\n') => {
                // `||` and `&&` are one operator.
                if matches!(character, '|' | '&') && characters.peek() == Some(&character) {
                    let _second = characters.next();
                }
                parts.push(std::mem::take(&mut current));
            }
            // Inside quotes everything but the closing quote is the part's own.
            _ => current.push(character),
        }
    }
    parts.push(current);
    parts
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

/// Whether `command` holds a command substitution, a process substitution or a here-document,
/// which run a command the lists never see.
#[must_use]
pub fn has_substitution(command: &str) -> bool {
    ["$(", "`", "<(", ">(", "<<"]
        .iter()
        .any(|marker| command.contains(marker))
}
