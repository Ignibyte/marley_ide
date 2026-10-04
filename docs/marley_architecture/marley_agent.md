# `marley_agent`

The agent CLIs Marley knows, for the rail's agent rows (W4, #440). Pure and gpui-free, MIT OR
Apache-2.0. Since #519 it depends on `marley_fleet`, `serde`, `serde_json` and `base64`, for
Claude Code's hook events, and since #569 on `procfs-core`, for the stall rule's reads of `/proc`. Refreshed for the fork on 2026-09-22; the gpui-era cockpit it once
served is history (below).

## Provenance

Marley-original, built from observed behavior only. It never borrows from Warp's agent source
(`crates/ai`, `app/src/ai/**`): those are AGPL, and the agent layer is where that licence would
matter most (`docs/warp_architecture/subsystems/04-agent-ai-mcp.md`).

## Surface

```rust
pub enum AgentKind { Claude, Codex, Gemini, OpenCode }
impl AgentKind {
    pub const ALL: [AgentKind; 4];                 // the order menus list them
    pub const fn program(self) -> &'static str;    // "claude", "codex", "gemini", "opencode"
    pub const fn display_name(self) -> &'static str;
}
pub fn agent_kind_of(command: &str) -> Option<AgentKind>;
pub fn send_payload(line: &str) -> Vec<u8>;        // the line and a carriage return
pub enum LaunchMode { Ask, Bypass }                // #532
pub fn launch_input(kind: AgentKind, mode: LaunchMode) -> Vec<u8>;
                                                     // the program, the mode's arguments, Enter
pub fn launch_line(kind: AgentKind, mode: LaunchMode, prompt: &str) -> Vec<u8>;   // #510
pub struct Remote<'a> { pub program: &'a str, pub socket: &'a str, pub folder: &'a str }  // #650
pub fn launch_line_after(setup: &str, kind: AgentKind, mode: LaunchMode, prompt: &str,
                         remote: Option<&Remote>) -> Vec<u8>;   // #585, #650
pub fn thread_mark(sandbox: &str, approval: &str) -> Option<PermissionMark>;   // #650
pub fn quote_argument(argument: &str) -> String;     // one word for bash, zsh and fish
pub const BYPASS_MODE: &str;                         // "bypassPermissions"
pub enum MarkKind { Bypass, FullAccess }
pub enum MarkSource { Reported, Thread(&'static str), Argument(&'static str) }
pub struct PermissionMark { pub kind: MarkKind, pub source: MarkSource }  // words(), tooltip()
pub fn permission_mark(kind: AgentKind, argv: &[String], reported: Option<&str>)
    -> Option<PermissionMark>;

pub enum AgentStatus { Working, Waiting, Idle, Failed }  // label(): "working", "waiting", …
pub const WAITING_AFTER: Duration;                  // 2 s
pub fn agent_status(quiet_for: Duration, bell: bool) -> AgentStatus;
pub fn status_line(kind: AgentKind, status: AgentStatus) -> String;   // "Claude Code · waiting"
pub enum TurnEvent { NeedsInput, Finished, Failed }    // of_change(before, after), words()
pub fn event_line(project: &str, kind: AgentKind, event: TurnEvent) -> String;
                                                     // "marley_ide: Claude needs input" (#535)
pub const GIT_PROMPTS_OFF: [(&str, &str); 3];       // an agent terminal's git variables (#537)
pub fn ssh_dialog_title(project: &str, kind: AgentKind) -> String;
                                                     // "ssh for Claude Code in marley_ide" (#596)
```

- **`GIT_PROMPTS_OFF`** is `GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never` and, since #596, an
  empty `GIT_ASKPASS`: git falls back to `SSH_ASKPASS` for its own prompts when `GIT_ASKPASS` is
  unset, and an agent's terminal sets `SSH_ASKPASS` for ssh alone.

- **`agent_kind_of`** reads a command line's leading program, with a directory path stripped,
  quotes around it dropped (a launch that joins an App Server names Codex by its quoted path,
  #650) and arguments ignored, so `/home/me/.local/bin/claude --resume` is Claude Code. Program names
  are case-sensitive. The rail feeds it a terminal's foreground argv, never a process name:
  Claude Code's binary on the dev box is named after its version.
- **`launch_input`** is everything the rail writes to start an agent: a program name from the
  fixed list, for `LaunchMode::Bypass` the agent's own bypass arguments (Claude Code's
  `--dangerously-skip-permissions`; Codex's `--sandbox danger-full-access --ask-for-approval
  never`, never `--dangerously-bypass-approvals-and-sandbox`), and Enter. No text from a user
  or a file reaches the shell this way (#532 keeps it: the arguments are constants).
- **`launch_line`** (#510) adds a first prompt as one quoted argument: positional for Claude Code
  and Codex, after a `--` when it starts with `-`; `--prompt-interactive=` for Gemini CLI and
  `--prompt=` for OpenCode. `quote_argument` puts runs of anything but an apostrophe or a
  backslash in single quotes and each of those two in double quotes, Orca's form (MIT), which
  bash, zsh and fish read alike; fish reads `\'` and `\\` as escapes inside single quotes, so sh's
  `'\''` breaks there.
- **`launch_line_after`** (#585) puts a setup command and `&&` before `launch_line`'s bytes, so the
  agent starts only once its worktree's install succeeded; an empty setup gives the launch line
  alone.
- **`review_prompt`** (#522) turns `ReviewLine`s into the prompt review notes are sent as, in file
  and line order: `File:`, `Line: N` or `Lines: A-B`, `User comment: "…"`, a blank line between
  notes; the words' backslashes, quotes, CR and LF escaped and other control characters dropped.
- **`permission_mark`** (#532) says whether an agent runs without its prompts. For Claude Code a
  reported mode decides (`BYPASS_MODE` marks, any other clears), else its arguments
  (`--dangerously-skip-permissions`, `--permission-mode bypassPermissions` and its `=` form);
  for Codex its arguments (`--sandbox danger-full-access`, `-s` and the `=` and joined forms, a
  `-c`/`--config` of `sandbox_mode` in TOML, `--dangerously-bypass-approvals-and-sandbox`).
  Arguments after a `--` are not options. The mark's `tooltip` names the source.
- **`agent_status`** reads waiting on a bell, or once the terminal has been quiet for
  `WAITING_AFTER`; otherwise working. Claude Code and Codex redraw a spinner while they work,
  so output keeps flowing until they stop. An agent that thinks without printing reads as
  waiting until it prints again.

## Claude Code's hook events (`src/claude_events.rs`, #519)

```rust
pub struct HookEvent { v, event, session_id, agent_id, prompt_id, permission_mode, cwd,
                       transcript_path, prompt, tool, preview, tool_use_id, message, error,
                       source, trigger, is_interrupt }
pub fn decode(body: &str) -> Result<HookEvent, DecodeError>;   // base64, ≤ 3,000 bytes, v 1
pub fn fold(seat: &str, previous: Option<&Session>, event: &HookEvent, now_ms: u64)
    -> Vec<SessionEvent>;
pub const fn seat_status(state: State) -> AgentStatus;
pub fn seat_line(seat: &Session, now_ms: u64, no_update_after_ms: u64, shown: StopKindShown,
                 flag: FlagShown) -> String;         // "working · 1 subagent · Add a README"
pub fn seat_activity(seat: &Session, shown: StopKindShown) -> Option<String>;
                                                     // "Bash: ls -la", the question, …
pub struct TurnFacts { started_ms, tools, subagent_tools, failures, interrupted,
                       permissions_asked, pending, edited, checked_after_edit, stops,
                       tool_lines }
pub struct ToolLine { pub line: String, pub failed: bool }          // #569
impl TurnFacts { pub fn of(labels: &BTreeMap<String, String>) -> Self; }   // the `turn` label
pub fn is_harness_injected(prompt: &str) -> bool;
pub fn is_compact_continuation(prompt: &str) -> bool;
pub enum PromptOrigin { User, SlashCommand(String), Injected(Option<&'static str>), Continuation }
pub fn prompt_origin(prompt: &str) -> PromptOrigin;                 // #509
```

- The plugin's `hooks/event.py` sends each hook event as an OSC 777 notify titled
  `marley-event` whose body is the base64 of a JSON summary; `decode` reads the body.
- `fold` moves a terminal's seat from the seat as it was, and returns the fleet events for
  `marley_fleet::apply`: an `Upsert` with the state and labels, then a `QuestionRaised` while
  the seat waits, or an `Ended` for `SessionEnd`. The rules are the spec's D6: prompts and tools
  make it working, a permission request (or `AskUserQuestion`) waiting until the tool it asked
  for finishes or the turn ends, `Stop` idle, `StopFailure` failed, a subagent's events move
  only a count, and a new session id starts the seat over.
- The labels carry the prompt, the lead's tool line, the last message, the error, the
  subagent count and the session's id, transcript, directory and permission mode (its key is
  `PERMISSION_MODE_LABEL` since #571, which the stall watch and the click pause read), and, for the
  fold itself, the tools in flight by call id (`lead_tool:`, `subagent_tool:`) and the call a
  wait ends with (`waiting_on`), so `fold` needs nothing but the previous seat.
- `seat_line` gives a working seat whose last event is at least `no_update_after_ms` old
  `no update in N m` in place of `working` (#547, through `marley_fleet::is_stale`); 0 never.
- `TurnFacts` (#566) is what the user's request has done so far: when its prompt came, the lead's
  tools by name and count, the subagents' tool count, the failures, whether the user interrupted
  the turn, the permissions asked and the calls still waiting on one, whether a file was edited,
  whether a Bash finished after the last edit (or with no edit, at all), and how many times the
  request has stopped. `fold` starts from the seat at each event, so the facts ride on the seat as
  the `turn` label, in JSON. A prompt the user types starts them over; a prompt a harness injects
  keeps them and starts only `interrupted` and the waiting calls over. The first interrupt of the
  lead's tool ends the turn and counts as a stop: tools in parallel each report it. Every turn's
  start and end clears the stop's four labels, so a kind never outlives its stop.
- An idle seat's `seat_line` reads its stop kind as `shown` says, and `seat_activity` puts the
  prompt's parts the message does not cover before the message: `not covered: "Add a license." ·
  Added the README.`
- `TurnFacts.tool_lines` (#569) keeps the lead's last twelve tool ends, oldest first, each line
  (`Bash: cargo test`) cut to 120 characters with whether it failed; an interrupt is not one. The
  fold takes the flag's four labels off at every event it takes, but a `looping` flag while the
  seat works and its newest line still repeats. A working seat's `seat_line` gives the flag's word
  as `flag` says (`looping?`, `stalled?`), before `no update in N m`.
- A prompt with one of the tags or openings Orca observed harnesses inject
  (`src/shared/harness-injected-user-turns.ts` in stablyai/orca, MIT) keeps the user's prompt on
  the seat; the continuation after a compaction changes nothing.
- `prompt_origin` (#509) says where a prompt came from, for a turn's title: the continuation
  first, then the tag at the head, where a slash command's envelope (`command-name`,
  `command-message`, `command-args`) is the user's and gives the name inside `<command-name>`,
  then the openings with no tag, else the user's.
- `PromptReading` (#648): `Recognized` reads prompts with the tags and openings, `AllTyped` reads
  every prompt as the user's; `is_injected`, `is_continuation` and `origin` answer as the reading
  says. `fold_with` takes a reading; `fold`, `is_harness_injected`, `is_compact_continuation` and
  `prompt_origin` are `Recognized`'s and keep their signatures.

## Codex's App Server (`src/codex_events.rs`, `src/marley_agent.rs`, #650)

- `launch_line_after` with a `Remote` names the program by its quoted full path, then
  `--remote 'unix://<socket>' --cd '<folder>'`, then the mode's arguments and the prompt, so the
  server and the TUI are one binary and the thread starts in the terminal's folder.
- `thread_mark(sandbox, approval)` is `full access` when the thread reports `dangerFullAccess`,
  whatever the approval policy; its source, `MarkSource::Thread`, carries the policy in words for
  the tooltip.
- `codex_events` reads the App Server's messages as Codex 0.155.1 and 0.158.0 generate them:
  `Thread` (`is_lead`: not ephemeral, no parent, started by the user or not saying),
  `ThreadStatus` (an unknown type is `Unknown`, which moves nothing), `Turn`, `Policies`.
  `Notification::decode(method, params)` reads `thread/started`, `thread/status/changed`,
  `turn/started`, `turn/completed`, `thread/tokenUsage/updated`, `thread/settings/updated` and
  `thread/closed`, and gives `None` for any other method; `read_answer` and `resume_answer` read
  `thread/read` and `thread/resume`.
- `fold(seat, previous, input, now_ms)` is `claude_events::fold`'s counterpart. The status decides
  the state (`active` working or waiting with the `wait` label `approval` or `input` and a
  question, "Waits on an approval" or "Waits on an answer"; `idle`; `systemError` failed;
  `notLoaded` ends the seat); a `failed` turn makes the seat failed with the turn's message under
  `error` and keeps it so through the `idle` after it until `turn/started`; token use rides as
  `tokens` and `context_window`, the policies as `sandbox` and `approval`, and the seat's `agent`
  label is `codex` with the lead under `thread`. `Input::ServerStopped` fails the seat with
  "Codex's App Server stopped" and the server's last line. `seat_words` gives the row's words,
  #547's `no update in N m` included; `seat_activity` the waiting question or the error.

- Since #651 `codex_events` also reads the four requests a server asks its clients to approve:
  `Request::decode` (a command, a file change, permissions, an MCP server's elicitation, with the
  JSON-RPC id kept as sent and the params for a later comparison); `decisions(files)` gives only
  those the request offers, and an allow only beside what it allows (a command with its line, a
  file change whose files its `item/started` named, permissions Marley reads, no session allow for
  a `grantRoot`); `response(decision)` builds the body Codex parses (`{decision}`, the requested
  permissions back with scope `turn` or `session` and an empty grant for a denial, `{action}`);
  `ask(files)` the inbox's line. `resolved` and `file_change_started` read `serverRequest/resolved`
  and a file change's `item/started`.

## The versions integrations were tested on (`src/versions.rs`, #648)

- A row of `INTEGRATIONS` is an integration that rests on something its agent does not document:
  `Integration { id, agent, name, off_means, setting, tested }`, its id the key under
  `marley.allow_untested_versions`. `CLAUDE_PROMPT_TAGS` (`claude_prompt_tags`): the tags and
  openings `claude_events` reads, tested from Claude Code 2.1.283 before 2.2.0. Since #650
  `CODEX_APP_SERVER` (`codex_app_server`): the App Server's messages, from Codex 0.155.1 before
  0.158.1, closed at the two ends checked.
  `terminalSequence`, which the hook channel rides on, is documented in Claude Code's hooks
  reference and gets no row.
- `Range { from, before }`: `contains` judges a prerelease by its release numbers; `words` gives
  "2.1.283 and later 2.1 releases", "X and later" or "X up to Y".
- `parse_version` takes the first word of the first line that reads as `N.N.N` (a leading `v`
  dropped), or says what the program printed. `Found { Version, Unreadable, Missing }`;
  `verdict(integration, found, allowed)` is `On`, `Allowed` (out of range or unread, and the
  user's setting on, which also covers a check not yet ended) or `Off(Untested | Unreadable |
  Missing | NotChecked)`; an unreadable version counts as untested. `chip_label` and `reasons`
  give the agent bar's words.

## What a stopped turn needs (`src/stop_kind.rs`, #566)

```rust
pub enum Verdict { Interrupted, Blocked, AsksYou, Open }
pub fn rules(message: Option<&str>, facts: &TurnFacts) -> Verdict;
pub fn parts(prompt: &str) -> Vec<String>;            // "Add a README.", "Add a license."
pub enum Kind { DoneChecked, DoneClaimed, AsksYou, Blocked, StillGoing, Interrupted }
pub const fn apply_evidence(kind: Kind, facts: &TurnFacts) -> Kind;
pub fn labels(kind: Option<(Kind, Source, f64)>, missing_parts: &[String])
    -> Vec<(&'static str, String)>;
pub enum StopKindShown { Hidden, Suggest, Act }
pub fn row_word(labels: &BTreeMap<String, String>, shown: StopKindShown) -> Option<String>;
pub fn not_covered(labels: &BTreeMap<String, String>, shown: StopKindShown) -> Option<String>;
pub fn state_facts(facts: &TurnFacts, now_ms: u64) -> Vec<(&'static str, String)>;
```

- `rules` settles what code can see before anything is asked ("local first"): an interrupt, a
  permission whose call never finished (refused, or never answered), or a last message whose last
  sentence ends in a question mark or opens with an asking phrase (`should I`, `do you want`,
  `would you like`, `which`, `shall I`, `let me know`, `can you confirm`). Anything else is `Open`,
  for the System One layer. A sentence ends at `.`, `?`, `!` or `;` and a space, not after an
  abbreviation or a list's number.
- `parts` splits the prompt into what the layer is asked about: its sentences and numbered items,
  and the items of a bullet the prompt opens with (the plugin sends a prompt on one line, so a
  dash elsewhere is a dash), each of two words or more, without an introduction ending in `:` or a
  last part the plugin cut short, at most six.
- `Kind`'s `value` is its label and its option in the question (`done_checked`), and `words` how a
  row says it (`done · checked`). `apply_evidence` holds the model to what code saw: a
  `done_checked` with no Bash after the last edit is `done_claimed`. The model can take a check
  away, never add one.
- The four labels are `stop_kind`, `stop_kind_source` (`rules` or `model`),
  `stop_kind_confidence` (`0.91`) and `stop_parts_missing`, a JSON array of the parts not covered.
- `row_word` gives the kind in place of `idle` (`Act`), after it with a question mark (`Suggest`,
  `idle · still going?`), or nothing (`Hidden`), and `not_covered` likewise.
- `state_facts` gives the facts a stop's state carries: the agent, the tools (`Bash 1, Edit 1`),
  the failed tools, the permissions pending, `checked after edit` and the turn's length in words
  (`under a minute`, `4 minutes`), since a phrase reads better to a model than a count.

## Stalled or looping agents (`src/stall.rs`, #569)

```rust
pub fn repeats(lines: &[ToolLine]) -> Option<Repeat>;   // Repeat { line, times, failed }
pub fn tool_name(line: &str) -> &str;                   // "Bash" for "Bash: cargo test"
pub struct Facts { quiet_ms, tool_in_flight: Option<String>, cpu_active: Option<bool> }
pub enum Verdict { LongTask, Open, Calm }
pub fn judge(facts: &Facts, check_ms: Option<u64>) -> Verdict;
pub fn checks(first_seconds: u64) -> Vec<u64>;          // 60: 1, 2, 4 and 8 minutes, in ms
pub fn quiet_words(milliseconds: u64) -> String;        // "12 seconds", "3 minutes"
pub enum Stalled { WaitingForInput, Stuck, Frozen }
pub enum Flag { Looping, Stalled(Stalled) }             // "looping", "stalled:frozen"
pub fn labels(flag: Flag, source: &str, confidence: f64, reason: &str)
    -> Vec<(&'static str, String)>;
pub enum FlagShown { Hidden, Shown }
pub fn row_word(labels: &BTreeMap<String, String>, shown: FlagShown) -> Option<String>;
pub fn tooltip(labels: &BTreeMap<String, String>, shown: FlagShown) -> Option<String>;
pub const fn active(ticks: u64, elapsed_ms: u64) -> bool;          // over 2% of a core
pub const fn ticks_since_boot(epoch_ms: u64, boot_secs: u64) -> u64;
pub fn boot_time_in(proc_root: &Path) -> Option<u64>;
pub fn tree_cpu_in(proc_root: &Path, pid: i32, since_ticks: u64) -> Option<u64>;
```

- `repeats` reads a loop in the turn's tool lines: the newest line failing twice in a row, or
  ended three times in a row. The file tools (`Read`, `Write`, `Edit`, `MultiEdit`,
  `NotebookEdit`) count toward the failures only, since the plugin's line names only their file
  and several edits of one file in a row are ordinary work; an edit and a test in turn are a
  cycle, not a loop. `tool_name` gives a line's tool, a fact, where the rest of the line is text
  the agent wrote.
- `judge` decides a quiet working seat at its next check: a tool in flight whose processes burn
  CPU is a long task, however long the quiet; quiet past the check with two samples saying
  nothing burns CPU is open, for the layer; anything else is calm. `checks` doubles one number,
  and 0 is none.
- The CPU is the turn's tools': `tree_cpu_in` sums `utime` and `stime` over `pid`'s descendants
  (`task/<tid>/children`) started at or after `since_ticks`, read with `procfs-core`'s `Stat`.
  `pid` is Claude Code, whose spinner burns CPU while it waits, and the children it had before the
  turn are its MCP and language servers, so both are left out. The turn's start comes from the
  fleet's epoch milliseconds through `/proc/stat`'s `btime` (`ticks_since_boot`, at `USER_HZ`
  100), and `btime` is whole seconds, so the caller takes a second off it. The readers take their
  root, so a scenario's could stand in.
- The four labels are `flag` (`looping` or `stalled:<kind>`), `flag_source` (`rules` or
  `model`), `flag_confidence` (`0.91`) and `flag_reason`, what the flag rests on in words, which
  the tooltip reads. `row_word` and `tooltip` give nothing while `Hidden` (`off` and `shadow`).

## What an inbox entry's action would do (`src/risk.rs`, #568)

```rust
pub enum ToolClass { Read, Write, Execute, Delete, Question, Other }  // of_claude_tool(name)
pub enum ChipKind { Destroys, Credentials, RewritesHistory, Pays, SendsOut, Installs,
                    ChangesAccount, OutsideProject, ClaimsApproval }  // words, level, noul
pub enum ChipSource { Rules, Model }
pub struct Chip { kind: ChipKind, source: ChipSource }
pub struct Action<'a> { tool, line, paths, cwd, folders, home, secret: bool }
pub fn classify(action: &Action) -> Vec<Chip>;          // in the order of their kinds
pub fn level(tool: ToolClass, chips: &[Chip]) -> u8;    // 1 to 5
```

- `classify` reads a line as simple commands, split at `;`, `&&`, `||`, `|`, `&`, `(`, `)` and
  new lines outside quotes, each without the prefixes that only run it (`sudo`, `doas`, `env`,
  `nohup`, `time`, `command`, `exec`) and the variables set before it, and matches whole words:
  no regex and no shell parse, so a line the plugin cut at 200 characters still reads, and a
  quoted word that matches adds a chip, which errs toward caution. The tables: `destroys` (`rm`
  with a recursive or force flag or a glob, `git reset --hard`, `git clean -f`, `git checkout
  .`, `git branch -D`, `shred`, `dd of=`, `find -delete`, `mkfs`, SQL that drops or truncates, a
  call of kind `delete`); `credentials` (key files and folders by name, `.env`, `secret-tool`,
  `gpg`, `pass`, or `secret`, the redactor's finding); `rewrites history`; `sends out` (a `curl`
  or `wget` that sends data or a method other than GET, `scp`, `rsync` to a host, `ssh`, `git
  push`, `gh` creating, publishing tools); `installs` (package managers' install verbs, and a
  download piped into a shell); `outside project` (a write tool's path, or an absolute, `~` or
  `cd` path in a command, under no folder of the project, `/dev` and the temporary folders
  aside, resolved lexically against `cwd` and `home`); and `claims approval` (phrases that claim
  an approval, in the line or a question). A question gets only `claims approval`, and only a
  command gets the command rules.
- `level` is the highest chip's (5 for `destroys`, `credentials`, `rewrites history` and `pays`;
  4 for `sends out`, `installs` and `changes account`; 3 for `outside project`), else 1 for a
  read tool and 2 for any other, one higher when a chip claims an approval, at most 5.
- `ChipKind::noul` names the model's question about each of the seven a tool's action can carry,
  and `from_noul` reads it back; `pays` and `changes account` come from #571's classes alone.

## Who should answer an inbox entry (`src/route.rs`, #570)

```rust
pub enum Route { Owner, Manager, CouldProceed, Unclear }  // words, rank: 0, 1, 2, and 0
pub enum RouteSource { Rule(&'static str), Reading }
pub struct RouteMark { route: Route, source: RouteSource }
pub struct Facts<'a> { action: &Action<'a>, tool_name: &str, chips: &[Chip], options: &[String] }
pub enum Class { Owner(&'static str), CouldProceed(&'static str), Open }  // mark()
pub fn classify(facts: &Facts) -> Class;
pub fn route_of_choice(option: &str) -> Route;
```

- `classify` decides, first match winning: the owner's for a #568 chip of `destroys`,
  `credentials`, `rewrites history`, `sends out`, `outside project`, `pays` or `changes account`,
  for words that name money (and, outside a command, whose `$` is a variable, a currency sign
  next to a figure) or a message to people; open for a claim of approval, which no rule lets
  through; could proceed for a local read tool whose every path lies inside the project, or a
  command whose every simple command is a read-only program (`ls`, `cat`, `grep`, `git status`,
  `git log`, `find` without a delete or an exec …) with no redirection; open for the rest. The
  rule's name, such as `rewrites history`, is what the mark's tooltip gives. It shares
  `risk.rs`'s tokenizer, `program`, `words` and `resolve`.
- `route_of_choice` reads the System One choice: `owner`, `manager`, `agent_proceeds`, and
  `unclear` for `cannot_tell` or anything else. `rank` puts the owner's and the unclear first,
  since a person is the floor.
- `claude_events::HookEvent.options` (#570) carries an AskUserQuestion's first question's option
  labels, which `wait` puts in the seat's `Question.options`.

## Resuming a session (`src/marley_agent.rs`, #540)

- `resume_line(mode, session, folder)` is what resumes Claude Code's session in a shell:
  `cd -- <folder> && claude [mode's arguments] --resume <session>` and Enter, the folder one
  `quote_argument` word and the `cd` left out for an empty folder; `None` unless the session is
  an 8-4-4-4-12 hexadecimal id. `HookEvent::reason` carries `SessionEnd`'s reason.

## An agent's report (`src/report.rs`, #652)

- rustal-harness's report contract (its TICKET-099, `docs/AGENT_SEATS.md`), as Marley takes it
  from `marley-agent`. `Report` is the JSON object `rh report` builds (`source`, `seq`, `state`,
  and the optional `question`, `progress`, `session_id`, `resume_argv`, `usage`, `quota`), read
  with `deny_unknown_fields` so an unknown field is `agent_report_shape`.
- `Report::validate` checks the harness's field rules in its order and returns the first broken
  as a `Refusal` and a reason; `Refusal::name` gives the harness's names (`agent_unknown`,
  `agent_report_stale`, `agent_report_authority`, `agent_report_shape`, `_source`, `_state`,
  `_question`, `_progress`, `_session`, `_argv`, `_quota`, `agent_release_authority`,
  `agent_release_none`). A question is allowed only with `waiting`; `validate_resume_argv` is
  herdr's check (1 to 64 arguments, at most 8 KiB, no control character or apostrophe, and a
  plain command name first).
- `fleet_state` maps the six states onto `marley_fleet::State`; `labels` gives what the seat
  carries: `source`, `state.source` (`reported`), `report.session_id` (not `session_id`, which
  `claude_events::fold` reads as a new session), `progress.percent`, `progress.activity`, the
  three `usage.*` counts and `quota.KIND.percent_used` and `.resets_at_ms`.
- `Request` is the socket's line, tagged by `verb`: `Report { terminal, report }` or
  `Release { terminal, source }`; `answer_ok` and `answer_refused` are the answers.
- `stat_fields(stat)` reads a `/proc/PID/stat` line's parent and start time from the fields after
  its last `)`, since a process's name may hold spaces and parentheses. `MAX_ANCESTRY` (32) bounds
  the walk to a terminal.

## Claude Code's trust question (`src/trust.rs`, #587)

- `read(lines)` finds the question in a terminal's last lines, oldest first: the footer (`Enter
  to confirm`) among the last three, the first question line within 24 lines above it (`Do you
  trust the files in this folder?`, `Is this a project you created or one you trust?` or
  `Accessing workspace:`), and between them a trust option (`Yes, I trust this folder` or `Yes,
  proceed`) and an exit option (`No, exit`), each read past `❯`, spaces and a `1.` numbering.
  `TrustQuestion` keeps the options in order, the one marked `❯` as the focus, the first path line
  after the question line as its folder, and the lines that start with `⚠` as its warnings.
- `footer_on_screen(rows)` says whether the footer is on a row the screen shows.
- `answer_keys()` gives Up or Down (`\x1b[A`, `\x1b[B`) from the focus to the trust option, then
  `\r`; nothing without a focus.

## Consumers

- `marley_rail`: a terminal row carries `TerminalAgent { kind, status }` when an agent runs in
  its foreground, and `activity` from `seat_activity` when its events gave one.
- `marley_workbench::agent_events`: `decode` and `fold` for each `marley-event` frame, and
  since #566 `TurnFacts`, `rules`, `parts`, `state_facts`, `apply_evidence` and `labels` for the
  stop kind; `marley_workbench::rail`: `seat_status`, `seat_line` and `seat_activity` for a
  Claude Code row with a seat, with the `StopKindShown` and, since #569, the `FlagShown` the uses'
  modes give, and `stall::tooltip` for the row's mark.
- `marley_workbench::stall` (#569): `repeats`, `tool_name`, `judge`, `checks`, `quiet_words`,
  `labels`, `active`, `ticks_since_boot`, `boot_time_in` and `tree_cpu_in` for the watch.
- `marley_workbench::agent_trust` (#587): `trust::read`, `footer_on_screen` and `answer_keys`
  for the watch of a worktree agent's Claude Code.
- `marley_workbench::rail` (#568): `classify`, `level`, `ToolClass::of_claude_tool` and the
  chips' words for the inbox, whose entries in `marley_rail` carry the `Chip`s; since #570
  `route::classify`, `route_of_choice` and the `RouteMark` an entry carries.
- `marley_workbench::rail`:
  - recognition through `agent_kind_of`;
  - the `+` menu's Agent CLIs section (`AgentKind::ALL`, filtered to what the search path
    holds);
  - the launch (`launch_input`, written after the shell's startup handshake);
  - the quiet timers (`WAITING_AFTER` on gpui's executor clock).

## Tests

`src/marley_agent.rs`, five unit tests: names and order, recognition of all four and rejection
of others, the launch bytes, status around `WAITING_AFTER` and with the bell, and status lines.

## History

In the gpui-era app (M2, #66 to #80, #187) this crate modelled a pump-driven cockpit: an
`AgentRun` with tick counters, `agent_status_from` over `WAITING_TICKS`, and a `marley_app`
shim that launched and watched agents. The fork has no 16 ms pump and nothing used that model,
so #440 removed it. The agent brain that note once planned now belongs to the three-prong
plan's control plane (`docs/marley/three-prong-plan.md`).
