//! Secrets hidden from what agents read (#516, #562).
//!
//! Marley's tools hand agents terminal commands, their output and the browser's console. Before
//! any of it leaves, [`Redactor::redact`] replaces what looks like a secret with
//! `[redacted: <kind>]`: known key and token shapes, private key blocks, bearer tokens and other
//! authorization credentials, cookie headers, passwords in URLs, values assigned to secret-named
//! variables, and the user's own patterns. It names the kind so an agent knows something was
//! there, and counts the replacements for the tool's answer. The terminal's own buffer is never
//! changed: only what goes to a model is.

use std::sync::LazyLock;

use regex::{Captures, Regex};

/// What stands in for a hidden value of `kind`, such as `[redacted: secret]`.
#[must_use]
pub fn marker(kind: &str) -> String {
    format!("[redacted: {kind}]")
}

/// One rule: what it finds, and the capture group it leaves in place (the variable's name, the
/// `Bearer ` prefix, the URL up to its password), if any.
#[derive(Debug)]
struct Rule {
    kind: &'static str,
    regex: Regex,
    keep: Option<usize>,
    /// For rules with a trailing part to keep after the secret (the `@` of a URL's password).
    keep_after: Option<usize>,
}

/// The built-in rules, in the order they run: the ones that keep a name run first, so a token
/// assigned to `GITHUB_TOKEN=` is redacted once, as a secret assignment.
const BUILT_IN: &[(&str, &str, Option<usize>, Option<usize>)] = &[
    // A block with no END line yet (a running block still printing it) is hidden to the end.
    (
        "private key",
        r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY(?: BLOCK)?-----(?:[\s\S]*?-----END [A-Z0-9 ]*PRIVATE KEY(?: BLOCK)?-----|[\s\S]*)",
        None,
        None,
    ),
    // A cookie header is a list of credentials, so all of it goes: the rest of the line, or up
    // to a quote before a space or the line's end, which closes the argument the header sits in
    // (`curl -H "Cookie: …" <url>`). It runs before `secret`, which would otherwise take a cookie
    // named `token=` first and leave the others.
    (
        "cookie",
        r#"(?i)(\b(?:set-)?cookie["']?[ \t]*[=:][ \t]*)("[^"\n]*"|'[^'\n]*'|(?:[^\n"']|["'][^\s"'])+)"#,
        Some(1),
        None,
    ),
    (
        "secret",
        r#"(?i)(\b(?:export[ \t]+)?[A-Z0-9_.-]*(?:SECRET|TOKEN|PASSWORD|PASSWD|PASSPHRASE|API[_-]?KEY|PRIVATE[_-]?KEY|ACCESS[_-]?KEY|CREDENTIALS?|BEARER|PRIVKEY)[A-Z0-9_.-]*["']?[ \t]*[=:][ \t]*)("[^"\n]*"|'[^'\n]*'|[^\s"',;]+)"#,
        Some(1),
        None,
    ),
    (
        "bearer token",
        r"(?i)(\bbearer[ \t]+)[A-Za-z0-9._~+/-]{16,}=*",
        Some(1),
        None,
    ),
    // `authorization` is not a `secret` name: that rule's value stops at the space after the
    // scheme, and on `Authorization: Basic <credential>` it would hide `Basic` and leave the
    // credential. This rule keeps the scheme and hides what follows, a Digest header's parameter
    // list whole. A list has two parameters at least, so a Basic credential's `=` padding is not
    // read as one and the quote after it kept. It runs after `bearer token`, whose marker it
    // leaves as it is.
    (
        "authorization",
        r#"(?i)(\b(?:proxy-)?authorization["']?[ \t]*[=:][ \t]*(?:["']?(?:basic|bearer|token|digest|negotiate|ntlm|apikey)[ \t]+)?)("[^"\n]*"|'[^'\n]*'|[\w.-]+=(?:"[^"\n]*"|[^\s",]+)(?:[ \t]*,[ \t]*[\w.-]+=(?:"[^"\n]*"|[^\s",]+))+|[^\s"',;]+)"#,
        Some(1),
        None,
    ),
    (
        "url password",
        r"(\b[a-zA-Z][a-zA-Z0-9+.-]*://[^/\s:@]+:)[^/\s@]+(@)",
        Some(1),
        Some(2),
    ),
    // A token standing alone before the host, as git prints a clone URL that carries one.
    (
        "url password",
        r"(\bhttps?://)[A-Za-z0-9_-]{20,}(@)",
        Some(1),
        Some(2),
    ),
    ("aws key id", r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b", None, None),
    (
        "github token",
        r"\b(?:gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{22,})",
        None,
        None,
    ),
    (
        "slack token",
        r"\bxox[abeoprs]-[A-Za-z0-9-]{10,}",
        None,
        None,
    ),
    (
        "stripe key",
        r"\b(?:sk|rk)_(?:live|test)_[A-Za-z0-9]{16,}",
        None,
        None,
    ),
    ("google api key", r"\bAIza[0-9A-Za-z_-]{35}", None, None),
    (
        "api key",
        r"\bsk-(?:ant-|proj-)?[A-Za-z0-9_-]{20,}",
        None,
        None,
    ),
    (
        "jwt",
        r"\beyJ[A-Za-z0-9_-]{8,}\.eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}",
        None,
        None,
    ),
];

// A built-in rule that did not compile would be dropped; each kind's redaction is proven by the
// e2e scenario, which would show the gap.
static BUILT_IN_RULES: LazyLock<Vec<Rule>> = LazyLock::new(|| {
    BUILT_IN
        .iter()
        .filter_map(|&(kind, pattern, keep, keep_after)| {
            Regex::new(pattern).ok().map(|regex| Rule {
                kind,
                regex,
                keep,
                keep_after,
            })
        })
        .collect()
});

/// Text with its secrets hidden, and how many were.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redacted {
    /// The text, each secret replaced by `[redacted: <kind>]`.
    pub text: String,
    /// How many replacements were made.
    pub count: usize,
}

/// The built-in rules and the user's own patterns.
#[derive(Debug)]
pub struct Redactor {
    user: Vec<Regex>,
}

impl Redactor {
    /// A redactor with the built-in rules and `patterns`, each a regular expression whose
    /// matches are hidden as `[redacted: pattern]`, and the errors of the patterns that did not
    /// compile, which are left out.
    #[must_use]
    pub fn new(patterns: &[String]) -> (Self, Vec<String>) {
        let mut errors = Vec::new();
        let user = patterns
            .iter()
            .filter_map(|pattern| match Regex::new(pattern) {
                Ok(regex) => Some(regex),
                Err(error) => {
                    errors.push(format!("{pattern}: {error}"));
                    None
                }
            })
            .collect();
        (Self { user }, errors)
    }

    /// `text` with every secret the rules find replaced, and the count.
    #[must_use]
    pub fn redact(&self, text: &str) -> Redacted {
        let mut count = 0;
        let mut text = text.to_string();
        for rule in BUILT_IN_RULES.iter() {
            text = apply(rule, &text, &mut count);
        }
        for regex in &self.user {
            let replaced = regex.replace_all(&text, |_: &Captures<'_>| {
                count += 1;
                marker("pattern")
            });
            text = replaced.into_owned();
        }
        Redacted { text, count }
    }
}

/// One rule over `text`: each match becomes its kept part and the marker, unless the value is a
/// marker already (a rule earlier in the order took it).
fn apply(rule: &Rule, text: &str, count: &mut usize) -> String {
    rule.regex
        .replace_all(text, |captures: &Captures<'_>| {
            let whole = captures.get(0).map_or("", |matched| matched.as_str());
            let kept = rule
                .keep
                .and_then(|group| captures.get(group))
                .map_or("", |kept| kept.as_str());
            let secret = whole.get(kept.len()..).unwrap_or_default();
            if secret
                .trim_start_matches(['"', '\''])
                .starts_with("[redacted")
            {
                return whole.to_string();
            }
            *count += 1;
            let after = rule
                .keep_after
                .and_then(|group| captures.get(group))
                .map_or("", |after| after.as_str());
            format!("{kept}{}{after}", marker(rule.kind))
        })
        .into_owned()
}
