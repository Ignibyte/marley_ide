//! Who should answer what an agent waits on (#570): the user, a manager agent, or nobody, since
//! the agent could go on by itself.
//!
//! An inbox entry's action, with #568's chips for it, is read by rules first: what could destroy,
//! leak, rewrite, send, spend or reach outside the project is the owner's, and so is what names
//! money or a message to people; a read inside the project, or a command that only reads, is one
//! the agent could proceed on; the rest is open, for the System One layer to read. A route only
//! marks and ranks an entry: nothing here, or in its users, answers a prompt. Pure.

use std::path::PathBuf;

use crate::risk::{self, Action, Chip, ChipKind, ToolClass};

/// Who should answer an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// The user, who owns the project.
    Owner,
    /// A manager agent, from the project's plan and conventions.
    Manager,
    /// Nobody: the agent could go on by itself.
    CouldProceed,
    /// Nothing settles it, so a person answers.
    Unclear,
}

impl Route {
    /// The mark's words.
    #[must_use]
    pub const fn words(self) -> &'static str {
        match self {
            Self::Owner => "for you",
            Self::Manager => "for the manager",
            Self::CouldProceed => "could proceed",
            Self::Unclear => "unclear",
        }
    }

    /// Where the route sorts within a level: the owner's and the unclear first, since a person is
    /// the floor, then the manager's, then what could proceed.
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Owner | Self::Unclear => 0,
            Self::Manager => 1,
            Self::CouldProceed => 2,
        }
    }
}

/// Where a route came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteSource {
    /// A rule of Marley's, by its name.
    Rule(&'static str),
    /// A model's reading; its confidence stays with the reading.
    Reading,
}

/// An inbox entry's route, and where it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteMark {
    /// Who should answer.
    pub route: Route,
    /// Where that came from.
    pub source: RouteSource,
}

/// What Marley's rules decide of an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// The owner's, by the rule named.
    Owner(&'static str),
    /// One the agent could proceed on, by the rule named.
    CouldProceed(&'static str),
    /// Neither: the System One layer may read it.
    Open,
}

impl Class {
    /// The mark a rule's class gives, none for an open entry.
    #[must_use]
    pub const fn mark(self) -> Option<RouteMark> {
        match self {
            Self::Owner(rule) => Some(RouteMark {
                route: Route::Owner,
                source: RouteSource::Rule(rule),
            }),
            Self::CouldProceed(rule) => Some(RouteMark {
                route: Route::CouldProceed,
                source: RouteSource::Rule(rule),
            }),
            Self::Open => None,
        }
    }
}

/// What the rules read of an entry: its action, its tool's name, #568's chips for it, and a
/// question's options.
#[derive(Debug, Clone, Copy)]
pub struct Facts<'a> {
    /// The action, as #568 reads it.
    pub action: &'a Action<'a>,
    /// The tool's name, such as `Read` or `WebFetch`.
    pub tool_name: &'a str,
    /// #568's chips for the action.
    pub chips: &'a [Chip],
    /// A question's options, none for a permission.
    pub options: &'a [String],
}

/// What the rules decide of `facts`, the owner's rules first.
#[must_use]
pub fn classify(facts: &Facts) -> Class {
    if let Some(chip) = facts
        .chips
        .iter()
        .find(|chip| OWNER_CHIPS.contains(&chip.kind))
    {
        return Class::Owner(chip.kind.words());
    }
    let text = std::iter::once(facts.action.line)
        .chain(facts.options.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");
    let spoken = risk::words(&text);
    let command = matches!(facts.action.tool, ToolClass::Execute);
    if risk::holds_any(&spoken, MONEY) || (!command && names_an_amount(&text)) {
        return Class::Owner("names money");
    }
    if risk::holds_any(&spoken, PEOPLE) {
        return Class::Owner("messages people");
    }
    let claims = facts
        .chips
        .iter()
        .any(|chip| chip.kind == ChipKind::ClaimsApproval);
    if claims {
        return Class::Open;
    }
    if matches!(facts.action.tool, ToolClass::Read) && reads_inside(facts) {
        return Class::CouldProceed("a read inside the project");
    }
    if command && reads_only(facts.action.line) {
        return Class::CouldProceed("a command that only reads");
    }
    Class::Open
}

/// The route of the System One choice `option`: `owner`, `manager` or `agent_proceeds`, and
/// `unclear` for `cannot_tell` or anything else.
#[must_use]
pub fn route_of_choice(option: &str) -> Route {
    match option {
        "owner" => Route::Owner,
        "manager" => Route::Manager,
        "agent_proceeds" => Route::CouldProceed,
        _ => Route::Unclear,
    }
}

/// The chips whose action is the owner's to allow.
const OWNER_CHIPS: [ChipKind; 7] = [
    ChipKind::Destroys,
    ChipKind::Credentials,
    ChipKind::RewritesHistory,
    ChipKind::SendsOut,
    ChipKind::OutsideProject,
    ChipKind::Pays,
    ChipKind::ChangesAccount,
];

/// Words that name money.
const MONEY: &[&str] = &[
    "invoice",
    "invoices",
    "billing",
    "purchase",
    "purchases",
    "subscription",
    "subscriptions",
    "refund",
    "refunds",
    "payment",
    "payments",
    "checkout",
    "charge",
    "charged",
    "pay",
    "paid",
    "wallet",
    "stripe",
    "paypal",
];

/// Words that name a message to people.
const PEOPLE: &[&str] = &[
    "mail",
    "email",
    "sendmail",
    "slack",
    "discord",
    "tweet",
    "sms",
    "whatsapp",
    "telegram",
    "reply",
    "notify send",
    "gh pr create",
    "gh issue comment",
    "gh pr comment",
];

/// Whether `text` names an amount of money: a `$`, `€` or `£` next to a figure. A command's `$`
/// is a variable, so the caller asks only of other text.
fn names_an_amount(text: &str) -> bool {
    let characters: Vec<char> = text.chars().collect();
    characters.iter().enumerate().any(|(index, character)| {
        matches!(character, '$' | '€' | '£') && {
            let after = characters
                .get(index + 1..)
                .and_then(|rest| rest.iter().find(|next| !next.is_whitespace()));
            let before = characters
                .get(..index)
                .and_then(|rest| rest.iter().rev().find(|next| !next.is_whitespace()));
            after.is_some_and(char::is_ascii_digit) || before.is_some_and(char::is_ascii_digit)
        }
    })
}

/// Whether a read tool reads only inside the project: a local tool, not a web one, whose every
/// path lies under a folder of the project.
fn reads_inside(facts: &Facts) -> bool {
    let name = facts.tool_name.to_lowercase();
    if name.contains("web") || name.contains("fetch") || facts.action.folders.is_empty() {
        return false;
    }
    let mut paths: Vec<PathBuf> = facts.action.paths.to_vec();
    if paths.is_empty() && !facts.action.line.is_empty() {
        if facts.action.line.contains("://") {
            return false;
        }
        paths.push(PathBuf::from(facts.action.line));
    }
    !paths.is_empty()
        && paths.iter().all(|path| {
            risk::resolve(path, facts.action.cwd, facts.action.home).is_some_and(|path| {
                facts
                    .action
                    .folders
                    .iter()
                    .any(|folder| path.starts_with(folder))
            })
        })
}

/// Programs that only read.
const READ_ONLY: &[&str] = &[
    "ls", "cat", "head", "tail", "wc", "rg", "grep", "pwd", "which", "echo", "tree", "stat",
    "file", "du", "df",
];

/// `git` subcommands that only read.
const GIT_READS: &[&str] = &["status", "log", "diff", "show", "blame"];

/// Whether every simple command of `line` only reads, with no redirection.
fn reads_only(line: &str) -> bool {
    let commands = risk::commands(line);
    !commands.is_empty()
        && commands.iter().all(|command| {
            let redirects = command
                .words
                .iter()
                .any(|word| word.contains('>') || word.contains('<'));
            let arguments = risk::arguments(command);
            let reads = match risk::program(command) {
                Some("find") => !arguments.iter().any(|argument| {
                    ["-delete", "-exec", "-execdir", "-ok", "-okdir", "-fprint"]
                        .contains(&argument.as_str())
                }),
                Some("git") => arguments
                    .iter()
                    .find(|argument| !argument.starts_with('-'))
                    .is_some_and(|subcommand| GIT_READS.contains(&subcommand.as_str())),
                Some(program) => READ_ONLY.contains(&program),
                None => false,
            };
            reads && !redirects
        })
}
