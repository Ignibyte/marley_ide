//! The agent versions Marley's integrations were tested on (#648).
//!
//! Marley runs the user's own `claude` and `codex`. Most of what it does with them rests on what
//! the agents document; a row of [`INTEGRATIONS`] is an integration that rests on something they
//! do not, checked on a range of versions. Outside its range a row stays off unless the user turns
//! it on (`marley.allow_untested_versions`), and the agent bar says why.
//!
//! A range runs from the oldest version checked up to the next minor release, so a patch release
//! of a checked minor counts as tested: Claude Code updates itself several times a week, and a
//! range closed at the newest patch would turn a row off within days of each Marley build. A
//! prerelease counts as its release. Finding the program and reading its version is
//! `marley_workbench::agent_versions`'s.

use semver::Version;

use crate::AgentKind;

/// The versions an integration was tested on: from `from`, up to but not including `before`, or
/// every later one when `before` is `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    /// The oldest version checked.
    pub from: Version,
    /// The first version outside the range, if any.
    pub before: Option<Version>,
}

impl Range {
    /// Whether `version` is in the range, a prerelease counting as its release: 2.2.0-beta.1 is
    /// 2.2.0 here, which semver alone would order below 2.2.0.
    #[must_use]
    pub fn contains(&self, version: &Version) -> bool {
        let release = Version::new(version.major, version.minor, version.patch);
        release >= self.from && self.before.as_ref().is_none_or(|before| release < *before)
    }

    /// The range in words: "2.1.283 and later 2.1 releases", "2.1.287 and later", "0.155.0 up to
    /// 0.159.0".
    #[must_use]
    pub fn words(&self) -> String {
        match &self.before {
            None => format!("{} and later", self.from),
            Some(before)
                if before.major == self.from.major
                    && before.minor == self.from.minor + 1
                    && before.patch == 0 =>
            {
                format!(
                    "{} and later {}.{} releases",
                    self.from, self.from.major, self.from.minor
                )
            }
            Some(before) => format!("{} up to {before}", self.from),
        }
    }
}

/// An integration that rests on something its agent does not document, and the versions it was
/// tested on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Integration {
    /// Its key under `marley.allow_untested_versions`.
    pub id: &'static str,
    /// The agent it reads.
    pub agent: AgentKind,
    /// What it is, as the agent bar names it.
    pub name: &'static str,
    /// What happens while it is off.
    pub off_means: &'static str,
    /// The Settings window's item that turns it on anyway.
    pub setting: &'static str,
    /// The versions it was tested on.
    pub tested: Range,
}

/// The tags and openings that tell a prompt Claude Code injects from one the user typed.
///
/// A task notification, a system reminder, a slash command's envelope: Claude Code's hooks
/// reference names none of them, and the list is Orca's observed one (#519). Each of the 19 tag
/// names was found in the 2.1.283 and 2.1.288 binaries (#519, #648).
pub const CLAUDE_PROMPT_TAGS: Integration = Integration {
    id: "claude_prompt_tags",
    agent: AgentKind::Claude,
    name: "Prompt tags",
    off_means: "every prompt in Claude Code's terminal reads as yours, task notifications and \
                system reminders included, so the rail and the turns show them as your prompt",
    setting: "Prompt Tags on Untested Claude Code",
    tested: Range {
        from: Version::new(2, 1, 283),
        before: Some(Version::new(2, 2, 0)),
    },
};

/// Every integration with a tested range.
pub static INTEGRATIONS: [Integration; 1] = [CLAUDE_PROMPT_TAGS];

/// `agent`'s integrations with a tested range.
pub fn integrations_of(agent: AgentKind) -> impl Iterator<Item = &'static Integration> {
    INTEGRATIONS
        .iter()
        .filter(move |integration| integration.agent == agent)
}

/// The version a program's `--version` printed.
///
/// The first word of the first line that reads as `N.N.N`, with an optional prerelease and a
/// leading `v` dropped, as `2.1.288 (Claude Code)` and `codex-cli 0.155.1` print it.
///
/// # Errors
///
/// When no line holds a version: what the program printed, its first line cut to 80 characters.
pub fn parse_version(output: &str) -> Result<Version, String> {
    let version = output.lines().find_map(|line| {
        line.split_whitespace().find_map(|word| {
            let word = word.trim_matches(|character: char| matches!(character, '(' | ')' | ','));
            Version::parse(word.strip_prefix('v').unwrap_or(word)).ok()
        })
    });
    version.ok_or_else(|| {
        let first = output.lines().find(|line| !line.trim().is_empty());
        first.map_or_else(
            || "printed nothing".to_string(),
            |line| {
                let cut: String = line.trim().chars().take(80).collect();
                format!("printed \"{cut}\"")
            },
        )
    })
}

/// What Marley found for an agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Found {
    /// The version its `--version` printed.
    Version(Version),
    /// Why no version could be read: what it printed, or why it did not run.
    Unreadable(String),
    /// Where Marley looked and found no program.
    Missing(String),
}

/// Why an integration is off.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Off {
    /// The version found is outside the range tested.
    Untested(Version),
    /// No version could be read.
    Unreadable(String),
    /// No program was found.
    Missing(String),
    /// The first check has not ended.
    NotChecked,
}

/// Whether an integration is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// The version found is in the range tested.
    On,
    /// Outside the range, or unread, and the user turned it on anyway.
    Allowed,
    /// Off, and why.
    Off(Off),
}

impl Verdict {
    /// Whether the integration runs.
    #[must_use]
    pub const fn is_on(&self) -> bool {
        matches!(self, Self::On | Self::Allowed)
    }
}

/// `integration`'s verdict on what was `found` (nothing while the first check runs), `allowed`
/// when the user turned it on anyway. A version Marley cannot read counts as untested.
#[must_use]
pub fn verdict(integration: &Integration, found: Option<&Found>, allowed: bool) -> Verdict {
    let off = match found {
        Some(Found::Version(version)) if integration.tested.contains(version) => {
            return Verdict::On;
        }
        Some(Found::Version(version)) => Off::Untested(version.clone()),
        Some(Found::Unreadable(why)) => Off::Unreadable(why.clone()),
        Some(Found::Missing(looked)) => Off::Missing(looked.clone()),
        None => Off::NotChecked,
    };
    if allowed {
        Verdict::Allowed
    } else {
        Verdict::Off(off)
    }
}

/// The agent bar's words for an agent whose integration is off for `off`.
#[must_use]
pub fn chip_label(agent: AgentKind, off: &Off) -> String {
    let name = agent.display_name();
    match off {
        Off::Untested(version) => format!("Untested {name} {version}"),
        Off::Unreadable(_) => format!("{name} version unknown"),
        Off::Missing(_) => format!("{name} not on Marley's PATH"),
        Off::NotChecked => format!("{name} not checked yet"),
    }
}

/// The chip's tooltip, a line each: for each integration off, what is off and what that changes;
/// the program found and its version, or what went wrong; the range tested; the setting.
#[must_use]
pub fn reasons(agent: AgentKind, path: Option<&str>, offs: &[(&Integration, &Off)]) -> Vec<String> {
    let name = agent.display_name();
    let mut lines = Vec::new();
    for (integration, off) in offs {
        lines.push(format!(
            "{} are off: {}.",
            integration.name, integration.off_means
        ));
        lines.push(match (off, path) {
            (Off::Untested(version), Some(path)) => format!("Found {name} {version} at {path}."),
            (Off::Untested(version), None) => format!("Found {name} {version}."),
            (Off::Unreadable(why), Some(path)) => {
                format!("{path} --version {why}, so Marley cannot tell the version.")
            }
            (Off::Unreadable(why), None) => format!("{name} --version {why}."),
            (Off::Missing(looked), _) => format!("{looked}."),
            (Off::NotChecked, _) => format!("Marley has not read {name}'s version yet."),
        });
        lines.push(format!("Tested on {}.", integration.tested.words()));
        lines.push(format!(
            "Turn on {} in the Marley settings to use it anyway.",
            integration.setting
        ));
    }
    lines
}
