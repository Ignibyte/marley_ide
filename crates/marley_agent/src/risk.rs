//! Risk chips for the rail's inbox (#568): what code says a waiting action would do.
//!
//! An inbox entry names a tool and what it acts on: a command, a path, a question. Code reads
//! that first, from words and paths, and marks what could destroy data, touch credentials,
//! rewrite history, send data out, install software, reach outside the project or claim an
//! approval; each mark is a chip with a level, and the levels order the inbox. The words are
//! matched whole over each simple command of the line, with no shell parse: a line the plugin cut
//! at 200 characters still reads, and a quoted word that matches adds a chip, which errs toward
//! caution. A model's reading may add chips, never remove one, and nothing here answers a prompt.
//! Pure.

use std::path::{Component, Path, PathBuf};

/// The highest level, what could destroy data or cannot be undone.
pub const TOP_LEVEL: u8 = 5;

/// What kind of tool an entry waits to run; it sets the level when no chip marks the entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolClass {
    /// A tool that only reads: `Read`, `Grep`, `Glob`, `LS`, `WebFetch` or `WebSearch`, or an Agent
    /// Panel call of kind `read`, `search`, `fetch` or `think`.
    Read,
    /// A tool that writes the files it names: `Write`, `Edit`, `MultiEdit` or `NotebookEdit`, or a
    /// call of kind `edit` or `move`.
    Write,
    /// A command: `Bash`, or a call of kind `execute`.
    Execute,
    /// An Agent Panel call of kind `delete`.
    Delete,
    /// A question the agent asks the user.
    Question,
    /// Any other tool.
    Other,
}

impl ToolClass {
    /// The class of a Claude Code tool, by its name.
    #[must_use]
    pub fn of_claude_tool(name: &str) -> Self {
        match name {
            "Read" | "Grep" | "Glob" | "LS" | "WebFetch" | "WebSearch" => Self::Read,
            "Write" | "Edit" | "MultiEdit" | "NotebookEdit" => Self::Write,
            "Bash" => Self::Execute,
            "AskUserQuestion" => Self::Question,
            _ => Self::Other,
        }
    }

    /// The level of an entry of this class that no chip marks.
    const fn base_level(self) -> u8 {
        match self {
            Self::Read => 1,
            Self::Write | Self::Execute | Self::Delete | Self::Question | Self::Other => 2,
        }
    }
}

/// What a chip says the action would do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChipKind {
    /// It deletes files or data that cannot easily be brought back.
    Destroys,
    /// It reads or writes credentials, keys or tokens.
    Credentials,
    /// It rewrites git history that may be shared.
    RewritesHistory,
    /// It spends money (a click #571 holds).
    Pays,
    /// It sends data or code out of the machine, or in the user's name.
    SendsOut,
    /// It installs software or dependencies.
    Installs,
    /// It changes the user's account (a click #571 holds).
    ChangesAccount,
    /// It reaches outside the project's folders.
    OutsideProject,
    /// Its text claims an approval, which raises the entry one level.
    ClaimsApproval,
}

impl ChipKind {
    /// The kinds a tool's action can carry, which the model's set asks about one by one.
    pub const TOOL_KINDS: [Self; 7] = [
        Self::Destroys,
        Self::Credentials,
        Self::RewritesHistory,
        Self::SendsOut,
        Self::Installs,
        Self::OutsideProject,
        Self::ClaimsApproval,
    ];

    /// The chip's words.
    #[must_use]
    pub const fn words(self) -> &'static str {
        match self {
            Self::Destroys => "destroys",
            Self::Credentials => "credentials",
            Self::RewritesHistory => "rewrites history",
            Self::Pays => "pays",
            Self::SendsOut => "sends out",
            Self::Installs => "installs",
            Self::ChangesAccount => "changes account",
            Self::OutsideProject => "outside project",
            Self::ClaimsApproval => "claims approval",
        }
    }

    /// The level the chip puts its entry at; a claim of approval has none of its own and raises
    /// the entry's level by one instead.
    #[must_use]
    pub const fn level(self) -> Option<u8> {
        match self {
            Self::Destroys | Self::Credentials | Self::RewritesHistory | Self::Pays => {
                Some(TOP_LEVEL)
            }
            Self::SendsOut | Self::Installs | Self::ChangesAccount => Some(4),
            Self::OutsideProject => Some(3),
            Self::ClaimsApproval => None,
        }
    }

    /// The key of the model's noul about this kind, for the kinds a tool's action can carry.
    #[must_use]
    pub const fn noul(self) -> Option<&'static str> {
        match self {
            Self::Destroys => Some("destroys"),
            Self::Credentials => Some("credentials"),
            Self::RewritesHistory => Some("rewrites_history"),
            Self::SendsOut => Some("sends_out"),
            Self::Installs => Some("installs"),
            Self::OutsideProject => Some("outside_project"),
            Self::ClaimsApproval => Some("claims_approval"),
            Self::Pays | Self::ChangesAccount => None,
        }
    }

    /// The kind whose noul is `key`.
    #[must_use]
    pub fn from_noul(key: &str) -> Option<Self> {
        Self::TOOL_KINDS
            .into_iter()
            .find(|kind| kind.noul() == Some(key))
    }
}

/// Who put a chip on an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipSource {
    /// Marley's own rules.
    Rules,
    /// A model's reading; its probability stays with the reading.
    Model,
}

/// A mark on an inbox entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chip {
    /// What it says the action would do.
    pub kind: ChipKind,
    /// Who put it there.
    pub source: ChipSource,
}

impl Chip {
    /// A chip of Marley's rules.
    #[must_use]
    pub const fn rules(kind: ChipKind) -> Self {
        Self {
            kind,
            source: ChipSource::Rules,
        }
    }
}

/// An action an inbox entry waits on, as the rules read it.
#[derive(Debug, Clone, Copy)]
pub struct Action<'a> {
    /// The kind of tool.
    pub tool: ToolClass,
    /// What the tool acts on, on one line: a command, a path, a URL or a question.
    pub line: &'a str,
    /// The paths the tool names beside its line, such as an edit's file.
    pub paths: &'a [PathBuf],
    /// Where a command runs, against which a relative path is read.
    pub cwd: Option<&'a Path>,
    /// The project's folders.
    pub folders: &'a [PathBuf],
    /// The user's home folder, which `~` names.
    pub home: Option<&'a Path>,
    /// Whether the redactor found a secret in the line.
    pub secret: bool,
}

/// The chips Marley's rules give `action`, in the order of their kinds.
#[must_use]
pub fn classify(action: &Action) -> Vec<Chip> {
    let mut kinds = Vec::new();
    let commands = commands(action.line);
    if matches!(action.tool, ToolClass::Execute) {
        for command in &commands {
            kinds.extend(command_kinds(command));
        }
        if piped_into_a_shell(&commands) {
            kinds.push(ChipKind::Installs);
        }
        if holds_any(&words(action.line), SQL_DESTROYS) {
            kinds.push(ChipKind::Destroys);
        }
    }
    if matches!(action.tool, ToolClass::Delete) {
        kinds.push(ChipKind::Destroys);
    }
    if !matches!(action.tool, ToolClass::Question) && names_credentials(action, &commands) {
        kinds.push(ChipKind::Credentials);
    }
    if matches!(
        action.tool,
        ToolClass::Execute | ToolClass::Write | ToolClass::Delete
    ) && reaches_outside(action, &commands)
    {
        kinds.push(ChipKind::OutsideProject);
    }
    if holds_any(&words(action.line), APPROVAL_CLAIMS) {
        kinds.push(ChipKind::ClaimsApproval);
    }
    kinds.sort();
    kinds.dedup();
    kinds.into_iter().map(Chip::rules).collect()
}

/// The level of an entry of `tool` marked with `chips`: its highest chip's, else its tool's, one
/// higher when a chip claims an approval, and at most [`TOP_LEVEL`].
#[must_use]
pub fn level(tool: ToolClass, chips: &[Chip]) -> u8 {
    let marked = chips.iter().filter_map(|chip| chip.kind.level()).max();
    let raised = chips
        .iter()
        .any(|chip| chip.kind == ChipKind::ClaimsApproval);
    (marked.unwrap_or_else(|| tool.base_level()) + u8::from(raised)).min(TOP_LEVEL)
}

/// One simple command of a line: its words without their quotes, and whether the line piped the
/// command before it into this one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Command {
    pub(crate) words: Vec<String>,
    pub(crate) piped: bool,
}

/// The simple commands of `line`, split at `;`, `&&`, `||`, `|`, `&`, `(`, `)` and new lines
/// outside quotes.
pub(crate) fn commands(line: &str) -> Vec<Command> {
    let mut commands = Vec::new();
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut piped = false;
    let mut characters = line.chars().peekable();
    while let Some(character) = characters.next() {
        if let Some(open) = quote {
            if character == open {
                quote = None;
            } else {
                word.push(character);
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '\\' => word.extend(characters.next()),
            ';' | '&' | '|' | '(' | ')' | '\n' => {
                // `&&`, `||` and `;;` are one separator each, and only a lone `|` pipes.
                let doubled = characters.next_if_eq(&character).is_some();
                let pipe = character == '|' && !doubled;
                end_word(&mut word, &mut words);
                end_command(&mut words, &mut commands, piped);
                piped = pipe;
            }
            character if character.is_whitespace() => end_word(&mut word, &mut words),
            character => word.push(character),
        }
    }
    end_word(&mut word, &mut words);
    end_command(&mut words, &mut commands, piped);
    commands
}

fn end_word(word: &mut String, words: &mut Vec<String>) {
    if !word.is_empty() {
        words.push(std::mem::take(word));
    }
}

/// Ends the command of `words`, without the prefixes that only run it (`sudo`, `env`, `nohup`,
/// `time`, `command`, `exec`) and the variables set before it.
fn end_command(words: &mut Vec<String>, commands: &mut Vec<Command>, piped: bool) {
    let taken = std::mem::take(words);
    let mut rest = taken.as_slice();
    loop {
        match rest {
            [first, tail @ ..] if is_assignment(first) => rest = tail,
            [first, tail @ ..] if RUNNERS.contains(&first.as_str()) => {
                rest = tail;
                // `sudo -u <user>` and `-g <group>` take a word each.
                while let [flag, tail @ ..] = rest
                    && flag.starts_with('-')
                {
                    rest = if flag == "-u" || flag == "-g" {
                        tail.get(1..).unwrap_or_default()
                    } else {
                        tail
                    };
                }
            }
            _ => break,
        }
    }
    if !rest.is_empty() {
        commands.push(Command {
            words: rest.to_vec(),
            piped,
        });
    }
}

/// The commands that only run the command after them.
const RUNNERS: [&str; 7] = ["sudo", "doas", "env", "nohup", "time", "command", "exec"];

/// Whether `word` sets a variable, as `FOO=1` does before a command.
fn is_assignment(word: &str) -> bool {
    word.split_once('=').is_some_and(|(name, _)| {
        !name.is_empty()
            && name
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
            && !name.starts_with(|character: char| character.is_ascii_digit())
    })
}

/// The program's name: the last part of its path.
pub(crate) fn program(command: &Command) -> Option<&str> {
    let first = command.words.first()?;
    Some(first.rsplit('/').next().unwrap_or(first))
}

/// The words after the program.
pub(crate) fn arguments(command: &Command) -> &[String] {
    command.words.get(1..).unwrap_or_default()
}

/// Whether one of `arguments` is a short flag holding one of `letters`, or one of `long`.
fn has_flag(arguments: &[String], letters: &[char], long: &[&str]) -> bool {
    arguments.iter().any(|argument| {
        argument.strip_prefix("--").map_or_else(
            || {
                argument
                    .strip_prefix('-')
                    .is_some_and(|short| short.chars().any(|letter| letters.contains(&letter)))
            },
            |name| long.contains(&name.split('=').next().unwrap_or(name)),
        )
    })
}

/// The kinds one command's program and arguments mark.
fn command_kinds(command: &Command) -> Vec<ChipKind> {
    let Some(program) = program(command) else {
        return Vec::new();
    };
    let arguments = arguments(command);
    let has = |word: &str| arguments.iter().any(|argument| argument == word);
    let mut kinds = Vec::new();
    match program {
        "rm" if has_flag(arguments, &['r', 'R', 'f'], &["recursive", "force"])
            || arguments.iter().any(|argument| argument.contains('*')) =>
        {
            kinds.push(ChipKind::Destroys);
        }
        "git" => kinds.extend(git_kinds(arguments)),
        "shred" | "wipefs" | "truncate" => kinds.push(ChipKind::Destroys),
        "dd" if arguments.iter().any(|argument| argument.starts_with("of=")) => {
            kinds.push(ChipKind::Destroys);
        }
        "find" if has("-delete") => kinds.push(ChipKind::Destroys),
        name if name == "mkfs" || name.starts_with("mkfs.") => kinds.push(ChipKind::Destroys),
        "curl" | "wget" if sends_data(program, arguments) => kinds.push(ChipKind::SendsOut),
        "scp" | "sftp" | "ssh" | "ftp" | "nc" | "ncat" | "netcat" | "mail" | "mailx"
        | "sendmail" | "mutt" => kinds.push(ChipKind::SendsOut),
        "rsync" if arguments.iter().any(|argument| is_remote(argument)) => {
            kinds.push(ChipKind::SendsOut);
        }
        "gh" if gh_sends(arguments) => kinds.push(ChipKind::SendsOut),
        "npm" | "cargo" | "gem" | "twine" | "docker" | "podman"
            if first_word(arguments).is_some_and(|word| PUBLISH.contains(&word)) =>
        {
            kinds.push(ChipKind::SendsOut);
        }
        "aws"
            if has("s3")
                && (has("cp") || has("sync") || has("mv"))
                && arguments
                    .iter()
                    .any(|argument| argument.starts_with("s3://")) =>
        {
            kinds.push(ChipKind::SendsOut);
        }
        _ => {}
    }
    if installs(program, arguments) {
        kinds.push(ChipKind::Installs);
    }
    kinds
}

/// The subcommands that publish what a package or image tool made.
const PUBLISH: [&str; 3] = ["publish", "push", "upload"];

/// The first word of `arguments` that is not a flag.
fn first_word(arguments: &[String]) -> Option<&str> {
    arguments
        .iter()
        .map(String::as_str)
        .find(|argument| !argument.starts_with('-'))
}

/// What a `git` command's subcommand and flags mark.
fn git_kinds(arguments: &[String]) -> Vec<ChipKind> {
    let mut rest = arguments;
    // `-C <path>` and `-c <name=value>` come before the subcommand and take a word each.
    while let [flag, tail @ ..] = rest
        && flag.starts_with('-')
    {
        rest = if flag == "-C" || flag == "-c" {
            tail.get(1..).unwrap_or_default()
        } else {
            tail
        };
    }
    let Some((subcommand, flags)) = rest.split_first() else {
        return Vec::new();
    };
    let has = |word: &str| flags.iter().any(|flag| flag == word);
    match subcommand.as_str() {
        "reset" if has("--hard") => vec![ChipKind::Destroys],
        "clean" if has_flag(flags, &['f'], &["force"]) => vec![ChipKind::Destroys],
        "checkout" | "restore" if has(".") || (subcommand == "checkout" && has("--")) => {
            vec![ChipKind::Destroys]
        }
        "branch" if has("-D") => vec![ChipKind::Destroys],
        "stash" if has("drop") || has("clear") => vec![ChipKind::Destroys],
        "rebase" | "filter-branch" | "filter-repo" => vec![ChipKind::RewritesHistory],
        "commit" if has("--amend") => vec![ChipKind::RewritesHistory],
        "reflog" if has("expire") => vec![ChipKind::RewritesHistory],
        "push" => {
            let forced = has_flag(
                flags,
                &['f'],
                &["force", "force-with-lease", "force-if-includes"],
            ) || flags.iter().any(|flag| flag.starts_with('+'));
            if forced {
                vec![ChipKind::RewritesHistory, ChipKind::SendsOut]
            } else {
                vec![ChipKind::SendsOut]
            }
        }
        _ => Vec::new(),
    }
}

/// Whether a `curl` or `wget` sends data rather than only fetching.
fn sends_data(program: &str, arguments: &[String]) -> bool {
    let method = arguments.iter().enumerate().find_map(|(index, argument)| {
        if argument == "-X" || argument == "--request" || argument == "--method" {
            arguments.get(index + 1).map(String::as_str)
        } else {
            argument
                .strip_prefix("-X")
                .or_else(|| argument.strip_prefix("--request="))
                .or_else(|| argument.strip_prefix("--method="))
                .filter(|method| !method.is_empty())
        }
    });
    if method.is_some_and(|method| {
        !method.eq_ignore_ascii_case("get") && !method.eq_ignore_ascii_case("head")
    }) {
        return true;
    }
    if program == "wget" {
        return has_flag(
            arguments,
            &[],
            &["post-data", "post-file", "body-data", "body-file"],
        );
    }
    has_flag(
        arguments,
        &['d', 'F', 'T'],
        &[
            "data",
            "data-raw",
            "data-binary",
            "data-urlencode",
            "form",
            "form-string",
            "upload-file",
            "json",
        ],
    )
}

/// Whether an `rsync` operand names another machine, as `host:path` does.
fn is_remote(argument: &str) -> bool {
    !argument.starts_with('-')
        && argument
            .split_once(':')
            .is_some_and(|(host, _)| !host.is_empty() && !host.contains('/'))
}

/// Whether a `gh` command publishes: a pull request, an issue, a release, a gist or a repository.
fn gh_sends(arguments: &[String]) -> bool {
    let mut parts = arguments
        .iter()
        .map(String::as_str)
        .filter(|part| !part.starts_with('-'));
    matches!(
        (parts.next(), parts.next()),
        (Some("release"), _)
            | (
                Some("pr" | "issue" | "gist" | "repo"),
                Some("create" | "comment" | "edit")
            )
    )
}

/// Whether a command installs software or dependencies.
fn installs(program: &str, arguments: &[String]) -> bool {
    let first = first_word(arguments);
    let second = arguments
        .iter()
        .map(String::as_str)
        .filter(|word| !word.starts_with('-'))
        .nth(1);
    match program {
        "npm" | "pnpm" | "bun" => {
            first.is_some_and(|word| ["i", "install", "add", "ci"].contains(&word))
        }
        "yarn" => first.is_some_and(|word| ["add", "install"].contains(&word)),
        "pip" | "pip3" | "pipx" | "gem" | "brew" | "snap" | "flatpak" | "apt" | "apt-get"
        | "dnf" | "yum" | "zypper" => first == Some("install"),
        "cargo" => first.is_some_and(|word| ["install", "add"].contains(&word)),
        "go" => first.is_some_and(|word| ["install", "get"].contains(&word)),
        "uv" => first == Some("add") || (first == Some("pip") && second == Some("install")),
        "apk" => first == Some("add"),
        "pacman" | "yay" | "paru" => arguments
            .iter()
            .any(|argument| argument.starts_with("-S") || argument.starts_with("-U")),
        name if name.starts_with("python") => {
            has_flag(arguments, &['m'], &[]) && first == Some("pip") && second == Some("install")
        }
        _ => false,
    }
}

/// Whether a download is piped into a shell, as `curl … | sh` installs what it fetched.
fn piped_into_a_shell(commands: &[Command]) -> bool {
    commands.windows(2).any(|pair| {
        let [fetch, shell] = pair else {
            return false;
        };
        shell.piped
            && matches!(program(fetch), Some("curl" | "wget"))
            && matches!(program(shell), Some("sh" | "bash" | "zsh" | "dash"))
    })
}

/// Words that name credentials, keys or tokens where they appear in a path or a command.
const CREDENTIAL_PARTS: [&str; 9] = [
    ".ssh/",
    ".aws/",
    ".gnupg",
    ".config/gh",
    ".netrc",
    "id_rsa",
    "id_ed25519",
    "id_ecdsa",
    "keychain",
];

/// File endings of keys and certificates.
const KEY_ENDINGS: [&str; 4] = [".pem", ".key", ".p12", ".pfx"];

/// Programs that read or keep secrets.
const SECRET_PROGRAMS: [&str; 4] = ["secret-tool", "gpg", "pass", "security"];

/// Whether the action names credentials: a secret in its line, a key's file, or a program that
/// keeps secrets.
fn names_credentials(action: &Action, commands: &[Command]) -> bool {
    if action.secret {
        return true;
    }
    let paths = action.paths.iter().filter_map(|path| path.to_str());
    let words = commands
        .iter()
        .flat_map(|command| command.words.iter().map(String::as_str));
    if paths.chain(words).any(names_a_secret) || action.line.split_whitespace().any(names_a_secret)
    {
        return true;
    }
    commands
        .iter()
        .filter_map(program)
        .any(|program| SECRET_PROGRAMS.contains(&program))
}

/// Whether one path or word names a secret's file.
fn names_a_secret(word: &str) -> bool {
    let lower = word.to_lowercase();
    // A file's name ends a path, and follows the `=` or `@` of a flag such as `--data=@.env`.
    let name = lower.rsplit(['/', '=', '@']).next().unwrap_or(&lower);
    CREDENTIAL_PARTS.iter().any(|part| lower.contains(part))
        || lower.ends_with("/.ssh")
        || name == ".env"
        || name.starts_with(".env.")
        || KEY_ENDINGS
            .iter()
            .any(|ending| name.ends_with(ending) && name.len() > ending.len())
}

/// Whether the action writes or runs somewhere under no folder of the project: a write tool's
/// path, or an absolute, `~` or `cd` path in a command.
fn reaches_outside(action: &Action, commands: &[Command]) -> bool {
    if action.folders.is_empty() {
        return false;
    }
    let mut named: Vec<PathBuf> = action.paths.to_vec();
    if matches!(action.tool, ToolClass::Execute) {
        for command in commands {
            let is_cd = program(command) == Some("cd");
            for argument in arguments(command) {
                let value = argument.rsplit('=').next().unwrap_or(argument);
                if value.contains("://") || is_remote(value) {
                    continue;
                }
                if value.starts_with('/')
                    || value.starts_with('~')
                    || (is_cd && !value.starts_with('-'))
                {
                    named.push(PathBuf::from(value));
                }
            }
        }
    } else if action.paths.is_empty() && !action.line.is_empty() {
        named.push(PathBuf::from(action.line));
    }
    named
        .iter()
        .filter_map(|path| resolve(path, action.cwd, action.home))
        .any(|path| !is_allowed(&path, action.folders))
}

/// `path` as an absolute path: `~` read as the home folder and a relative path against `cwd`,
/// with `.` and `..` taken out; none when it cannot be told.
pub(crate) fn resolve(path: &Path, cwd: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    let text = path.to_str()?;
    let joined = if text == "~" {
        home?.to_path_buf()
    } else if let Some(rest) = text.strip_prefix("~/") {
        home?.join(rest)
    } else if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd?.join(path)
    };
    let mut clean = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::ParentDir => {
                clean = clean
                    .parent()
                    .map_or_else(|| clean.clone(), Path::to_path_buf);
            }
            Component::CurDir => {}
            other => clean.push(other),
        }
    }
    Some(clean)
}

/// Whether a path is inside the project, or where commands write as a matter of course: the
/// devices and the temporary folders.
fn is_allowed(path: &Path, folders: &[PathBuf]) -> bool {
    folders.iter().any(|folder| path.starts_with(folder))
        || ["/dev", "/tmp", "/var/tmp"]
            .iter()
            .any(|allowed| path.starts_with(allowed))
}

/// SQL that drops or empties a table.
const SQL_DESTROYS: &[&str] = &[
    "drop table",
    "drop database",
    "drop schema",
    "truncate table",
];

/// Words that claim the action was already approved, which the note's safety rule 6 counts as
/// hostile text.
const APPROVAL_CLAIMS: &[&str] = &[
    "owner approved",
    "already approved",
    "pre approved",
    "preapproved",
    "you allowed this",
    "you already allowed",
    "permission granted",
    "user approved",
    "approved by the user",
    "approved by the owner",
];

/// `text` lowercased, as words separated by single spaces with a space before and after, so a
/// phrase matches whole words only.
pub(crate) fn words(text: &str) -> String {
    let joined = text
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    format!(" {joined} ")
}

/// Whether the `words` of a text hold one of `phrases` as whole words.
pub(crate) fn holds_any(words: &str, phrases: &[&str]) -> bool {
    phrases
        .iter()
        .any(|phrase| words.contains(&format!(" {phrase} ")))
}
