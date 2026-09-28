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
pub fn launch_input(kind: AgentKind) -> Vec<u8>;   // the program's name and a carriage return

pub enum AgentStatus { Working, Waiting, Idle, Failed }  // label(): "working", "waiting", …
pub const WAITING_AFTER: Duration;                  // 2 s
pub fn agent_status(quiet_for: Duration, bell: bool) -> AgentStatus;
pub fn status_line(kind: AgentKind, status: AgentStatus) -> String;   // "Claude Code · waiting"
pub enum TurnEvent { NeedsInput, Finished, Failed }    // of_change(before, after), words()
pub fn event_line(project: &str, kind: AgentKind, event: TurnEvent) -> String;
                                                     // "marley_ide: Claude needs input" (#535)
```

- **`agent_kind_of`** reads a command line's leading program, with a directory path stripped
  and arguments ignored, so `/home/me/.local/bin/claude --resume` is Claude Code. Program names
  are case-sensitive. The rail feeds it a terminal's foreground argv, never a process name:
  Claude Code's binary on the dev box is named after its version.
- **`launch_input`** is everything the rail writes to start an agent: a program name from the
  fixed list and Enter. No text from a user or a file reaches the shell this way.
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
- `marley_workbench::rail` (#568): `classify`, `level`, `ToolClass::of_claude_tool` and the
  chips' words for the inbox, whose entries in `marley_rail` carry the `Chip`s.
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
