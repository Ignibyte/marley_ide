//! Rusty's skills and scripts as its tools serve them, and the writes Marley makes (#665).
//!
//! A skill is a `SKILL.md` in Rusty's store: active, so Claude Code loads it, or staged for
//! approval. A script is a `*.sh` beside a skill, the command `rusty <name>`. Rusty checks every
//! name, scans every skill for what should not reach an agent unread, and commits each write.

use serde::Deserialize;
use serde_json::{Value, json};

/// Rusty's tool for the skills.
pub const SKILL_LIST: &str = "skill_list";
/// Makes a skill, active or staged.
pub const SKILL_CREATE: &str = "skill_create";
/// Rewrites a skill's description and body, and answers the scan.
pub const SKILL_UPDATE: &str = "skill_update";
/// Deletes a skill, active or staged.
pub const SKILL_DELETE: &str = "skill_delete";
/// Scans a skill and answers the findings.
pub const SKILL_SCAN: &str = "skill_scan";
/// Makes a staged skill active.
pub const SKILL_APPROVE: &str = "skill_approve";
/// Removes a staged skill.
pub const SKILL_REJECT: &str = "skill_reject";
/// Rusty's tool for the scripts.
pub const SCRIPT_LIST: &str = "script_list";
/// A script and its text.
pub const SCRIPT_VIEW: &str = "script_view";
/// Replaces a script's text.
pub const SCRIPT_UPDATE: &str = "script_update";

/// One skill, as Rusty serves it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Skill {
    /// The directory's name, which is how Claude Code calls it.
    pub name: String,
    /// What it is for: Claude Code reads it to choose the skill.
    #[serde(default)]
    pub description: String,
    /// `user`, or `auto` for one an agent wrote.
    #[serde(default)]
    pub origin: String,
    /// `active`, or `pending` while staged for approval.
    #[serde(default)]
    pub status: String,
    /// Its `SKILL.md`.
    #[serde(default)]
    pub path: String,
    /// The Markdown after the frontmatter.
    #[serde(default)]
    pub body: String,
}

impl Skill {
    /// Whether it waits for approval.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.status == "pending"
    }

    /// Whether an agent wrote it.
    #[must_use]
    pub fn is_auto(&self) -> bool {
        self.origin == "auto"
    }
}

/// One script, as Rusty serves it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Script {
    /// The command's name: the file's name without `.sh`.
    pub name: String,
    /// The skill it sits beside.
    #[serde(default)]
    pub skill: String,
    /// The file.
    pub path: String,
    /// Its skill's status: `active` or `pending`.
    #[serde(default)]
    pub status: String,
}

impl Script {
    /// Whether its skill waits for approval, so it may not run.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.status == "pending"
    }

    /// The name Rusty's script tools take: `skill/name`, which two skills' scripts cannot share.
    #[must_use]
    pub fn qualified(&self) -> String {
        if self.skill.is_empty() {
            self.name.clone()
        } else {
            format!("{}/{}", self.skill, self.name)
        }
    }
}

/// The skills in the order the tab lists them: staged first, then active, each by name, as
/// Rusty's app sorts them.
#[must_use]
pub fn ordered(mut skills: Vec<Skill>) -> Vec<Skill> {
    skills.sort_by(|a, b| {
        b.is_pending()
            .cmp(&a.is_pending())
            .then_with(|| a.name.cmp(&b.name))
    });
    skills
}

/// `skill_list`'s answer.
///
/// # Errors
///
/// When the answer is not a list of skills.
pub fn skills_from_answer(text: &str) -> Result<Vec<Skill>, serde_json::Error> {
    serde_json::from_str(text)
}

/// `script_list`'s answer.
///
/// # Errors
///
/// When the answer is not a list of scripts.
pub fn scripts_from_answer(text: &str) -> Result<Vec<Script>, serde_json::Error> {
    serde_json::from_str(text)
}

/// `skill_scan`'s answer: the findings, none when the scan is clean.
///
/// # Errors
///
/// When the answer is not a list of findings.
pub fn findings_from_answer(text: &str) -> Result<Vec<String>, serde_json::Error> {
    serde_json::from_str(text)
}

/// `skill_update`'s answer: the skill as Rusty reread it, and the scan of its new file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UpdateAnswer {
    /// The skill as written.
    pub skill: Skill,
    /// The scan's findings; none when clean.
    #[serde(default)]
    pub findings: Vec<String>,
}

/// `skill_update`'s answer.
///
/// # Errors
///
/// When the answer is not an update's.
pub fn update_from_answer(text: &str) -> Result<UpdateAnswer, serde_json::Error> {
    serde_json::from_str(text)
}

/// `script_view`'s answer: the script and its text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ScriptText {
    /// The script.
    pub script: Script,
    /// What the file holds.
    #[serde(default)]
    pub text: String,
}

/// `script_view`'s answer.
///
/// # Errors
///
/// When the answer is not a script and its text.
pub fn script_text_from_answer(text: &str) -> Result<ScriptText, serde_json::Error> {
    serde_json::from_str(text)
}

/// Whether Rusty refused an approval because the skill's scan has findings, which Approve Anyway
/// overrides.
#[must_use]
pub fn blocked_by_scan(refusal: &str) -> bool {
    refusal.contains("blocked by safety scan")
}

/// One write to Rusty's store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillWrite {
    /// A new skill, active or staged.
    Create {
        /// Its directory's name; Rusty checks it.
        name: String,
        /// What it is for.
        description: String,
        /// Its Markdown.
        body: String,
        /// Whether it waits for approval.
        staged: bool,
    },
    /// A skill's description and body, sent together.
    Update {
        /// The skill's name.
        name: String,
        /// Its description.
        description: String,
        /// Its Markdown.
        body: String,
    },
    /// A skill deleted, active or staged.
    Delete {
        /// The skill's name.
        name: String,
    },
    /// A staged skill made active; `force` past the scan's findings.
    Approve {
        /// The skill's name.
        name: String,
        /// Whether the scan's findings are overridden.
        force: bool,
    },
    /// A staged skill removed.
    Reject {
        /// The skill's name.
        name: String,
    },
    /// A script's whole text.
    UpdateScript {
        /// The script, as `skill/name`.
        name: String,
        /// Its text.
        body: String,
    },
}

impl SkillWrite {
    /// Rusty's tool for it.
    #[must_use]
    pub const fn tool(&self) -> &'static str {
        match self {
            Self::Create { .. } => SKILL_CREATE,
            Self::Update { .. } => SKILL_UPDATE,
            Self::Delete { .. } => SKILL_DELETE,
            Self::Approve { .. } => SKILL_APPROVE,
            Self::Reject { .. } => SKILL_REJECT,
            Self::UpdateScript { .. } => SCRIPT_UPDATE,
        }
    }

    /// Its arguments, in Rusty's parameter names: a new skill's name and description trimmed, as
    /// Rusty's app sends them.
    #[must_use]
    pub fn arguments(&self) -> Value {
        match self {
            Self::Create {
                name,
                description,
                body,
                staged,
            } => json!({
                "name": name.trim(),
                "description": description.trim(),
                "body": body,
                "pending": staged,
                "force": false,
            }),
            Self::Update {
                name,
                description,
                body,
            } => json!({ "name": name, "description": description.trim(), "body": body }),
            Self::Delete { name } | Self::Reject { name } => json!({ "name": name }),
            Self::Approve { name, force } => json!({ "name": name, "force": force }),
            Self::UpdateScript { name, body } => json!({ "name": name, "body": body }),
        }
    }
}
