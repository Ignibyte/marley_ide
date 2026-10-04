//! The agent CLIs Marley knows.
//!
//! Marley starts agent CLIs (Claude Code, Codex, …) in its terminals and recognizes them there.
//! This crate is the pure, gpui-free model for that: [`agent_kind_of`] recognizes an agent from
//! the command a terminal runs, [`launch_input`] is what starts one in a shell, with or without
//! its permission prompts, and [`launch_line`] with a first prompt (#510), [`permission_mark`]
//! says whether one runs without them (#532), and
//! [`agent_status`] judges from its terminal whether it is working or waiting on the user.
//! [`claude_events`] reads Claude Code's own hook events, which Marley's plugin sends, and folds
//! them into a fleet seat (#519), [`stop_kind`] says what a stopped turn needs (#566),
//! [`stall`] what flags a looping or quiet working one (#569), [`risk`] what an action waiting in
//! the rail's inbox would do (#568), and [`route`] who should answer it (#570). [`versions`] holds
//! the agent versions Marley's integrations were tested on (#648). [`codex_events`] reads Codex's
//! own App Server and folds its thread into a fleet seat (#650). The launching and the watching
//! live in `marley_workbench`.

// gate:21 runs Zed's dylint lints (`tooling/lints`) with these as errors in the Marley crates;
// Zed's crates keep them at warn (CONSTITUTION §0).
#![cfg_attr(
    dylint_lib = "lints",
    deny(
        async_block_without_await,
        blocking_io_on_foreground,
        entity_update_in_render,
        map_lookup_then_insert,
        notify_in_render,
        owned_string_into_shared,
        shared_string_from_str_literal
    )
)]

use std::time::Duration;

use marley_fleet::State;

pub mod claude_events;
pub mod codex_events;
pub mod risk;
pub mod route;
pub mod stall;
pub mod stop_kind;
pub mod trust;
pub mod versions;

/// An agent CLI Marley knows. [`AgentKind::ALL`], [`AgentKind::program`] and
/// [`AgentKind::display_name`] grow with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentKind {
    /// Anthropic's Claude Code (`claude`).
    Claude,
    /// `OpenAI`'s Codex CLI (`codex`).
    Codex,
    /// Google's Gemini CLI (`gemini`).
    Gemini,
    /// `OpenCode` (`opencode`).
    OpenCode,
}

impl AgentKind {
    /// Every known agent, in the order a menu lists them.
    pub const ALL: [Self; 4] = [Self::Claude, Self::Codex, Self::Gemini, Self::OpenCode];

    /// The program the agent runs as, which is also the command that starts it.
    #[must_use]
    pub const fn program(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::OpenCode => "opencode",
        }
    }

    /// The agent's short name, for a line that has little room, such as a push (#535).
    #[must_use]
    pub const fn short_name(self) -> &'static str {
        match self {
            Self::Claude => "Claude",
            Self::Codex => "Codex",
            Self::Gemini => "Gemini",
            Self::OpenCode => "OpenCode",
        }
    }

    /// The agent's name as people call it.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::Gemini => "Gemini CLI",
            Self::OpenCode => "OpenCode",
        }
    }

    /// The bytes of the key that opens the agent's prompt in `$VISUAL` or `$EDITOR`, at its
    /// default binding (#649).
    ///
    /// Ctrl-G for Claude Code (`chat:externalEditor`), Codex
    /// (`tui.keymap.global.open_external_editor`) and Gemini CLI (`input.openExternalEditor`);
    /// Ctrl-X then E for [`Self::OpenCode`] (`editor_open`, `<leader>e` with the leader
    /// `ctrl+x`), whose own Ctrl-G goes to its first message.
    #[must_use]
    pub const fn editor_key(self) -> &'static [u8] {
        match self {
            Self::Claude | Self::Codex | Self::Gemini => b"\x07",
            Self::OpenCode => b"\x18e",
        }
    }
}

/// Classify a command line as an agent CLI by its leading program.
///
/// A directory path on the program is stripped and arguments are ignored, so `claude --resume`
/// and `/usr/bin/claude` both resolve to [`AgentKind::Claude`]. Returns `None` for any other
/// command, and for an empty or whitespace-only one.
#[must_use]
pub fn agent_kind_of(command: &str) -> Option<AgentKind> {
    // A launch that joins an App Server names the program by its quoted path (#650).
    let token = command
        .split_whitespace()
        .next()?
        .trim_matches(|quote| quote == '\'' || quote == '"');
    let program = token.rsplit('/').next().unwrap_or(token);
    AgentKind::ALL
        .into_iter()
        .find(|kind| kind.program() == program)
}

/// The PTY bytes that send a line to the foreground program: the line, then a carriage return
/// (Enter).
#[must_use]
pub fn send_payload(line: &str) -> Vec<u8> {
    format!("{line}\r").into_bytes()
}

/// Whether Marley starts an agent CLI with its own permission prompts (#532).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LaunchMode {
    /// With them.
    #[default]
    Ask,
    /// Without them: Claude Code's bypass, Codex's full access. The other agents start as they
    /// are.
    Bypass,
}

/// The arguments that start `kind` without its permission prompts: for Codex, the two its Full
/// Access preset stands for, and never `--dangerously-bypass-approvals-and-sandbox`, which its
/// own help keeps for machines sandboxed from outside.
const fn bypass_arguments(kind: AgentKind) -> &'static [&'static str] {
    match kind {
        AgentKind::Claude => &["--dangerously-skip-permissions"],
        AgentKind::Codex => &[
            "--sandbox",
            FULL_ACCESS_SANDBOX,
            "--ask-for-approval",
            "never",
        ],
        AgentKind::Gemini | AgentKind::OpenCode => &[],
    }
}

/// The variables of a terminal Marley opens for an agent CLI (#537).
///
/// With them git, and Git Credential Manager where it is installed, fail with a message where they
/// would wait on a prompt the agent cannot answer. Credential helpers still run before any prompt,
/// so a stored credential keeps working.
///
/// `GIT_ASKPASS` is empty because git falls back to `SSH_ASKPASS` for its own prompts when
/// `GIT_ASKPASS` is unset, and an agent's terminal sets `SSH_ASKPASS` for ssh alone (#596).
pub const GIT_PROMPTS_OFF: [(&str, &str); 3] = [
    ("GIT_TERMINAL_PROMPT", "0"),
    ("GCM_INTERACTIVE", "never"),
    ("GIT_ASKPASS", ""),
];

/// What starts `kind` in a shell: its program name, the arguments `mode` asks for, and Enter.
/// Every word is Marley's own.
#[must_use]
pub fn launch_input(kind: AgentKind, mode: LaunchMode) -> Vec<u8> {
    send_payload(&command(kind, mode, None))
}

/// The Codex App Server a Codex launch joins (#650).
///
/// The program by its full path, so the server and the TUI are one binary, the server's Unix
/// socket, and the folder the thread starts in, which a remote TUI sends only with `--cd`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Remote<'a> {
    /// The `codex` both run.
    pub program: &'a str,
    /// The server's socket.
    pub socket: &'a str,
    /// The terminal's folder.
    pub folder: &'a str,
}

/// What starts `kind` in a shell with `prompt` as its first prompt (#510).
///
/// [`launch_input`]'s words, then the prompt as one quoted argument, the way each agent takes
/// one, and Enter. A prompt that starts with `-` follows a `--` for Claude Code and Codex, and
/// Gemini CLI and `OpenCode` take theirs in an option's `=` form, so no prompt is read as an
/// option. An empty prompt starts the agent with none.
#[must_use]
pub fn launch_line(kind: AgentKind, mode: LaunchMode, prompt: &str) -> Vec<u8> {
    line_with(kind, mode, prompt, None)
}

/// [`launch_line`], joining `remote`'s App Server when given.
fn line_with(kind: AgentKind, mode: LaunchMode, prompt: &str, remote: Option<&Remote>) -> Vec<u8> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return send_payload(&command(kind, mode, remote));
    }
    let mut line = command(kind, mode, remote);
    line.push(' ');
    match kind {
        AgentKind::Claude | AgentKind::Codex => {
            if prompt.starts_with('-') {
                line.push_str("-- ");
            }
        }
        AgentKind::Gemini => line.push_str("--prompt-interactive="),
        AgentKind::OpenCode => line.push_str("--prompt="),
    }
    line.push_str(&quote_argument(prompt));
    send_payload(&line)
}

/// What resumes Claude Code's session `session` in a shell (#540).
///
/// `cd` to `folder`, where the session started and where `claude --resume` looks it up, then
/// Claude Code with the arguments `mode` asks for, `--resume` and the id, and Enter.
///
/// `None` unless `session` is an id in the 8-4-4-4-12 hexadecimal form Claude Code gives, so a
/// damaged or forged value types nothing. The folder goes in as one quoted word; an empty folder
/// leaves the `cd` out.
#[must_use]
pub fn resume_line(mode: LaunchMode, session: &str, folder: &str) -> Option<Vec<u8>> {
    if !is_session_id(session) {
        return None;
    }
    let resume = format!(
        "{} --resume {session}",
        command(AgentKind::Claude, mode, None)
    );
    let line = if folder.is_empty() {
        resume
    } else {
        format!("cd -- {} && {resume}", quote_argument(folder))
    };
    Some(send_payload(&line))
}

/// Whether `id` is a session id in the 8-4-4-4-12 hexadecimal form.
fn is_session_id(id: &str) -> bool {
    let groups: Vec<&str> = id.split('-').collect();
    groups.len() == 5
        && groups.iter().zip([8, 4, 4, 4, 12]).all(|(group, len)| {
            group.len() == len && group.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
}

/// [`launch_line`] after `setup`, a command that must succeed first (#585), joining `remote`'s
/// App Server when given (#650).
///
/// The line is `<setup> && <launch line>`, so the agent starts only once its worktree's install
/// has, and a failed one leaves its error on the screen. An empty `setup` gives the launch line
/// alone.
#[must_use]
pub fn launch_line_after(
    setup: &str,
    kind: AgentKind,
    mode: LaunchMode,
    prompt: &str,
    remote: Option<&Remote>,
) -> Vec<u8> {
    let line = line_with(kind, mode, prompt, remote);
    let setup = setup.trim();
    if setup.is_empty() {
        return line;
    }
    let mut after = format!("{setup} && ").into_bytes();
    after.extend(line);
    after
}

/// The program and the arguments `mode` asks for; with `remote`, the program's full path and the
/// App Server's two arguments first, each one quoted word.
fn command(kind: AgentKind, mode: LaunchMode, remote: Option<&Remote>) -> String {
    let mut line = remote.map_or_else(
        || kind.program().to_string(),
        |remote| {
            format!(
                "{} --remote {} --cd {}",
                quote_argument(remote.program),
                quote_argument(&format!("unix://{}", remote.socket)),
                quote_argument(remote.folder)
            )
        },
    );
    if mode == LaunchMode::Bypass {
        for argument in bypass_arguments(kind) {
            line.push(' ');
            line.push_str(argument);
        }
    }
    line
}

/// `argument` quoted so bash, zsh and fish all read it as one word.
///
/// Its runs of anything but an apostrophe or a backslash go in single quotes, and each of those
/// two in double quotes. Fish reads `\'` and `\\` as escapes even inside single quotes, so sh's
/// `'\''` breaks there; this form, Orca's (`src/shared/tui-agent-startup-shell.ts`, MIT), is read
/// the same by all three.
#[must_use]
pub fn quote_argument(argument: &str) -> String {
    let mut quoted = String::new();
    let mut run = String::new();
    for character in argument.chars() {
        let alone = match character {
            '\'' => "\"'\"",
            '\\' => "\"\\\\\"",
            _ => {
                run.push(character);
                continue;
            }
        };
        if !run.is_empty() {
            quoted.push('\'');
            quoted.push_str(&run);
            quoted.push('\'');
            run.clear();
        }
        quoted.push_str(alone);
    }
    if !run.is_empty() || quoted.is_empty() {
        quoted.push('\'');
        quoted.push_str(&run);
        quoted.push('\'');
    }
    quoted
}

/// A review note as an agent reads it (#522): its file as the agent finds it, its lines from 1,
/// and the reviewer's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewLine {
    /// The file, relative to the agent's folder when it lies under it.
    pub path: String,
    /// The first line the note covers, from 1.
    pub first: u32,
    /// The last line it covers.
    pub last: u32,
    /// The reviewer's words.
    pub comment: String,
}

/// The prompt that hands review notes to an agent (#522).
///
/// It takes the form Orca's report describes for its diff notes
/// (`src/shared/diff-comments-format.ts`): per note `File: <path>`, `Line: N` or `Lines: A-B`, and
/// `User comment: "<words>"`, the notes in file and line order, a blank line between them. In the
/// words a backslash, a quote, a CR and a LF are escaped, and any other control character is
/// dropped, so the prompt holds no escape sequence.
#[must_use]
pub fn review_prompt(mut notes: Vec<ReviewLine>) -> String {
    notes.sort_by(|a, b| (&a.path, a.first, a.last).cmp(&(&b.path, b.first, b.last)));
    notes
        .iter()
        .map(|note| {
            let lines = if note.first == note.last {
                format!("Line: {}", note.first)
            } else {
                format!("Lines: {}-{}", note.first, note.last)
            };
            let path: String = note.path.chars().filter(|c| !c.is_control()).collect();
            format!(
                "File: {path}\n{lines}\nUser comment: \"{}\"",
                escape_comment(&note.comment)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// The reviewer's words inside the prompt's quotes.
fn escape_comment(comment: &str) -> String {
    let mut escaped = String::with_capacity(comment.len());
    for character in comment.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\r' => escaped.push_str("\\r"),
            '\n' => escaped.push_str("\\n"),
            other if other.is_control() => {}
            other => escaped.push(other),
        }
    }
    escaped
}

/// The permission mode Claude Code reports while it asks for no permission.
pub const BYPASS_MODE: &str = "bypassPermissions";

/// Codex's sandbox mode with no sandbox.
const FULL_ACCESS_SANDBOX: &str = "danger-full-access";

/// What an agent that runs without its permission prompts runs without (#532).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkKind {
    /// Claude Code asks for no permission.
    Bypass,
    /// Codex runs with no sandbox and no approvals.
    FullAccess,
}

/// Where Marley read that an agent runs without its permission prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkSource {
    /// Claude Code's hook events report [`BYPASS_MODE`].
    Reported,
    /// Codex's thread reports [`THREAD_FULL_ACCESS`], with this approval policy (#650).
    Thread(&'static str),
    /// The agent's process was started with this argument.
    Argument(&'static str),
}

/// The mark on the rail row of an agent that runs without its permission prompts (#532).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionMark {
    /// What it runs without.
    pub kind: MarkKind,
    /// Where Marley read it.
    pub source: MarkSource,
}

impl PermissionMark {
    /// The mark's words.
    #[must_use]
    pub const fn words(self) -> &'static str {
        match self.kind {
            MarkKind::Bypass => "bypass",
            MarkKind::FullAccess => "full access",
        }
    }

    /// What the mark means, and where Marley read it.
    #[must_use]
    pub fn tooltip(self) -> String {
        let what = match self.kind {
            MarkKind::Bypass => "Claude Code asks for no permission",
            MarkKind::FullAccess => "Codex runs with no sandbox and no approvals",
        };
        match self.source {
            MarkSource::Reported => format!("{what}: its events report {BYPASS_MODE}"),
            MarkSource::Thread(approval) => format!(
                "Codex runs with no sandbox: its thread reports {THREAD_FULL_ACCESS}, with \
                 approvals {approval}"
            ),
            MarkSource::Argument(argument) => format!("{what}: started with {argument}"),
        }
    }
}

/// The mark for `kind` if it runs without its permission prompts (#532).
///
/// `argv` is the process's arguments and `reported` the permission mode its hook events last
/// reported. For Claude Code a reported mode decides: its own settings can start it in bypass
/// with no argument, and Shift+Tab can leave bypass. Otherwise the arguments do.
#[must_use]
pub fn permission_mark(
    kind: AgentKind,
    argv: &[String],
    reported: Option<&str>,
) -> Option<PermissionMark> {
    let (kind, argument) = match kind {
        AgentKind::Claude => {
            if let Some(mode) = reported {
                return (mode == BYPASS_MODE).then_some(PermissionMark {
                    kind: MarkKind::Bypass,
                    source: MarkSource::Reported,
                });
            }
            (MarkKind::Bypass, claude_bypass_argument(options(argv))?)
        }
        AgentKind::Codex => (
            MarkKind::FullAccess,
            codex_full_access_argument(options(argv))?,
        ),
        AgentKind::Gemini | AgentKind::OpenCode => return None,
    };
    Some(PermissionMark {
        kind,
        source: MarkSource::Argument(argument),
    })
}

/// The sandbox policy a Codex thread reports when it runs with no sandbox (#650).
pub const THREAD_FULL_ACCESS: &str = "dangerFullAccess";

/// The mark for a Codex whose App Server thread reports `sandbox` with `approval` (#650).
///
/// Full access when the sandbox is [`THREAD_FULL_ACCESS`], whatever Codex's arguments asked for
/// and whatever the approval policy, as the arguments' rule keys on the sandbox alone.
#[must_use]
pub fn thread_mark(sandbox: &str, approval: &str) -> Option<PermissionMark> {
    let approval = match approval {
        "never" => "never",
        "on-request" => "on request",
        "untrusted" => "untrusted",
        "granular" => "granular",
        _ => "unknown",
    };
    (sandbox == THREAD_FULL_ACCESS).then_some(PermissionMark {
        kind: MarkKind::FullAccess,
        source: MarkSource::Thread(approval),
    })
}

/// The arguments before a `--`, after which they are no options.
fn options(argv: &[String]) -> &[String] {
    argv.split(|argument| argument == "--")
        .next()
        .unwrap_or_default()
}

/// The argument in `arguments` that starts Claude Code asking for no permission.
fn claude_bypass_argument(arguments: &[String]) -> Option<&'static str> {
    let mut arguments = arguments.iter().map(String::as_str);
    while let Some(argument) = arguments.next() {
        if argument == "--dangerously-skip-permissions" {
            return Some("--dangerously-skip-permissions");
        }
        let mode = if argument == "--permission-mode" {
            arguments.next()
        } else {
            argument.strip_prefix("--permission-mode=")
        };
        if mode == Some(BYPASS_MODE) {
            return Some("--permission-mode bypassPermissions");
        }
    }
    None
}

/// The argument in `arguments` that starts Codex with no sandbox.
fn codex_full_access_argument(arguments: &[String]) -> Option<&'static str> {
    let mut arguments = arguments.iter().map(String::as_str);
    while let Some(argument) = arguments.next() {
        if argument == "--dangerously-bypass-approvals-and-sandbox" {
            return Some("--dangerously-bypass-approvals-and-sandbox");
        }
        let sandbox = match argument {
            "--sandbox" | "-s" => arguments.next(),
            _ => argument
                .strip_prefix("--sandbox=")
                .or_else(|| argument.strip_prefix("-s")),
        };
        if sandbox.map(|mode| mode.trim_start_matches('=')) == Some(FULL_ACCESS_SANDBOX) {
            return Some("--sandbox danger-full-access");
        }
        let config = match argument {
            "--config" | "-c" => arguments.next(),
            _ => argument
                .strip_prefix("--config=")
                .or_else(|| argument.strip_prefix("-c")),
        };
        if config.is_some_and(sets_full_access) {
            return Some("--config sandbox_mode=\"danger-full-access\"");
        }
    }
    None
}

/// Whether a Codex `--config` override, `key=value` in TOML, sets the sandbox to full access.
fn sets_full_access(config: &str) -> bool {
    config
        .trim_start_matches('=')
        .split_once('=')
        .is_some_and(|(key, value)| {
            key.trim() == "sandbox_mode"
                && value
                    .trim()
                    .trim_matches(|quote| quote == '"' || quote == '\'')
                    == FULL_ACCESS_SANDBOX
        })
}

/// What an agent CLI is doing, as far as its terminal shows, or as its hook events say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    /// Output is arriving, or its events say a turn is under way: the agent is at work.
    Working,
    /// Output has stopped, the agent rang the bell, or it asked for a permission: it waits on the
    /// user.
    Waiting,
    /// Its events say the turn ended and it waits at its prompt (#519).
    Idle,
    /// Its events say the turn failed, with an error (#519).
    Failed,
}

impl AgentStatus {
    /// The word a row shows for the status.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Waiting => "waiting",
            Self::Idle => "idle",
            Self::Failed => "failed",
        }
    }
}

/// What an agent's turn came to, for a line that names it outside Marley (#535).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnEvent {
    /// It asks for a permission or a choice.
    NeedsInput,
    /// Its turn ended.
    Finished,
    /// Its turn failed.
    Failed,
}

impl TurnEvent {
    /// The event a seat's change of state from `before` to `after` makes, if it makes one: a
    /// wait that starts, a turn that ends, a turn that fails.
    #[must_use]
    pub const fn of_change(before: State, after: State) -> Option<Self> {
        match (before, after) {
            (State::Waiting, State::Waiting) | (State::Error, State::Error) => None,
            (_, State::Waiting) => Some(Self::NeedsInput),
            (State::Working | State::Waiting, State::Idle) => Some(Self::Finished),
            (_, State::Error) => Some(Self::Failed),
            _ => None,
        }
    }

    /// The words a line gives the event.
    #[must_use]
    pub const fn words(self) -> &'static str {
        match self {
            Self::NeedsInput => "needs input",
            Self::Finished => "finished",
            Self::Failed => "failed",
        }
    }
}

/// A line that names an agent's event outside Marley, such as `marley_ide: Claude needs input`
/// (#535). The project's name loses its control characters; nothing the agent wrote goes in.
#[must_use]
pub fn event_line(project: &str, kind: AgentKind, event: TurnEvent) -> String {
    let project: String = project
        .chars()
        .filter(|character| !character.is_control())
        .collect();
    format!("{project}: {} {}", kind.short_name(), event.words())
}

/// The heading of the dialog where ssh in an agent's terminal asks for a passphrase, such as
/// `ssh for Claude Code in marley_ide` (#596). The project's name loses its control characters.
#[must_use]
pub fn ssh_dialog_title(project: &str, kind: AgentKind) -> String {
    let project: String = project
        .chars()
        .filter(|character| !character.is_control())
        .collect();
    format!("ssh for {} in {project}", kind.display_name())
}

/// How long an agent's terminal stays quiet before the agent reads as waiting. Claude Code and
/// Codex redraw a spinner while they work, so a spell this long without output means they have
/// stopped.
pub const WAITING_AFTER: Duration = Duration::from_secs(2);

/// An agent's status from how long its terminal has been quiet and whether it rang the bell.
#[must_use]
pub fn agent_status(quiet_for: Duration, bell: bool) -> AgentStatus {
    if bell || quiet_for >= WAITING_AFTER {
        AgentStatus::Waiting
    } else {
        AgentStatus::Working
    }
}

/// A row's second line for an agent, such as "Claude Code · waiting".
#[must_use]
pub fn status_line(kind: AgentKind, status: AgentStatus) -> String {
    format!("{} · {}", kind.display_name(), status.label())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_agent_has_its_program_and_name() {
        let expected = [
            (AgentKind::Claude, "claude", "Claude Code"),
            (AgentKind::Codex, "codex", "Codex"),
            (AgentKind::Gemini, "gemini", "Gemini CLI"),
            (AgentKind::OpenCode, "opencode", "OpenCode"),
        ];
        assert_eq!(
            AgentKind::ALL,
            expected.map(|(kind, _, _)| kind),
            "menus list them in this order"
        );
        for (kind, program, name) in expected {
            assert_eq!(kind.program(), program);
            assert_eq!(kind.display_name(), name);
        }
    }

    #[test]
    fn agent_kind_of_recognizes_each_program_and_nothing_else() {
        for kind in AgentKind::ALL {
            let program = kind.program();
            assert_eq!(agent_kind_of(program), Some(kind));
            // Arguments are ignored and a directory path is stripped.
            assert_eq!(agent_kind_of(&format!("{program} --resume")), Some(kind));
            assert_eq!(
                agent_kind_of(&format!("/usr/local/bin/{program}")),
                Some(kind)
            );
            // The command the launch writes is recognized back as the same agent.
            let launched =
                String::from_utf8(launch_input(kind, LaunchMode::Ask)).expect("the launch is text");
            assert_eq!(agent_kind_of(&launched), Some(kind));
        }
        assert_eq!(agent_kind_of("ls"), None);
        assert_eq!(agent_kind_of("claude-code"), None, "a different program");
        assert_eq!(
            agent_kind_of("Claude"),
            None,
            "program names are case-sensitive"
        );
        assert_eq!(agent_kind_of(""), None);
        assert_eq!(agent_kind_of("   "), None);
    }

    #[test]
    fn the_launch_is_the_program_and_enter() {
        assert_eq!(
            launch_input(AgentKind::Claude, LaunchMode::Ask),
            b"claude\r"
        );
        assert_eq!(
            launch_input(AgentKind::OpenCode, LaunchMode::Ask),
            b"opencode\r"
        );
        assert_eq!(send_payload("ls"), b"ls\r");
        assert_eq!(send_payload(""), b"\r");
        assert_eq!(send_payload("café"), "café\r".as_bytes());
    }

    #[test]
    fn an_agent_waits_after_a_quiet_spell_or_a_bell() {
        let just_under = WAITING_AFTER.saturating_sub(Duration::from_millis(1));
        assert_eq!(agent_status(Duration::ZERO, false), AgentStatus::Working);
        assert_eq!(agent_status(just_under, false), AgentStatus::Working);
        assert_eq!(agent_status(WAITING_AFTER, false), AgentStatus::Waiting);
        assert_eq!(agent_status(Duration::ZERO, true), AgentStatus::Waiting);
        assert_eq!(agent_status(just_under, true), AgentStatus::Waiting);
    }

    #[test]
    fn the_status_line_names_the_agent_and_its_status() {
        assert_eq!(
            status_line(AgentKind::Claude, AgentStatus::Working),
            "Claude Code · working"
        );
        assert_eq!(
            status_line(AgentKind::Gemini, AgentStatus::Waiting),
            "Gemini CLI · waiting"
        );
    }
}
