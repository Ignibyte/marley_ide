---
pipeline_id: ffcd2825-3b93-4067-8f28-60b48b890df8
ticket: docs/planning/tickets/open/TICKET-556-terminal-run.md
status: Phase 4 — Complete PASS
title: "terminal_run: an agent runs commands in the user's terminal, as blocks"
type: feature
slice: prong 2 with prong 1 (plan D9's `terminal.run`, grant-gated; the Warp blocks note, recommendation 3)
references: [docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md, docs/planning/pipeline/queued/525-agent-drives-a-running-program.spec.md, docs/planning/pipeline/completed/491-marley-mcp-in-the-app.spec.md, docs/planning/pipeline/completed/474-block-hover-actions.spec.md, docs/planning/pipeline/queued/520-terminal-identity.spec.md]
---

## Title
`terminal_run {terminal, command}` on Marley's MCP server types a command at a terminal's shell
prompt, as the Rerun button does, waits for the block to end, and answers with its exit code,
duration and output. Claude Code's own Bash tool runs in a subprocess nobody sees; this is what
makes an agent's commands blocks in Chad's terminal, with Warp's allow and deny lists in front, a
mark on every block an agent ran, and Ctrl-I to take the terminal back.

## Scope
### In
- **The tool.** `terminal_run {terminal, command, wait_seconds?}`, family `terminal`, verb `run`,
  `Tier::Write`, grant class `terminal.write` (the class #525 names; granted when the server
  starts, beside `browser.write`, whichever of #525 and this ticket lands first). `wait_seconds`
  is 0 to 20, default 20. The answer carries `terminal`, `block` (its index), `command`,
  `running`, `exit_code`, `duration_ms`, `output`, `truncated` and `redacted`, the output under
  `terminal_read`'s caps (2,000 lines, 256 KiB) and #516's redaction; a block still running at the
  deadline answers `running: true` with the output so far, and `terminal_read` follows.
- **The preconditions**, each a refusal with its reason (`isError`): no terminal of that id; the
  shell is not the foreground process (a program runs, an agent CLI included); the blocks show no
  verified prompt (`AnchoredBlocks::at_prompt` and a prompt signed with the terminal's own nonce,
  D2); something is typed at the prompt, or the input line cannot be read (`autosuggest::
  typed_text` is not `Some` of blanks); the user has taken the terminal
  over; the command is empty, over 4,096 bytes, or holds a newline.
- **The lists.** Two settings of regexes with Warp's defaults verbatim (D3):
  `marley.agent_command_allowlist` and `marley.agent_command_denylist`, and
  `marley.agent_commands_outside_lists`: `run` (the default; the agent's own permission prompt is
  the approval) or `ask`. A command the denylist matches always asks; one the allowlist matches
  runs at once; the rest follow the setting. The three join the Marley page's Agents section (the
  two lists as text in `settings.json`, named by the dropdown's description, as #516's patterns
  are).
- **The card.** A command that must ask shows in that terminal's footer: who asks (the calling
  terminal's agent through #520's caller when it has landed, else "An agent"), the command in full,
  Run and Refuse; a toast in the terminal's workspace names the terminal, with Show (#525's
  shape and code, `terminal_drive.rs`). While the card's terminal holds the focus, Enter runs and
  Escape refuses (D5). An answer that comes after the deadline runs nothing.
- **Typing and the block.** Ctrl-U, the command and a return in one `Terminal::input`, the
  Rerun path (`terminal_element.rs:2350`). The next `preexec` frame that carries the nonce and
  the command opens the agent's block; the tool waits for its `precmd` (the block `Finished`)
  or the deadline, then reads it as `terminal_read` does.
- **The mark.** A block an agent ran carries an agent icon before its pill, always shown, with a
  tooltip naming the agent when known; `terminal_blocks` gives each block `"agent": true|false`.
  The mark reaches Zed's element through the chip hook (D7).
- **Take over.** `marley::TakeOverTerminal` on Ctrl-I in `Terminal`, #525's action and state
  (one take-over stops both tools): while an agent has run a command in that terminal, Ctrl-I takes it over or hands
  it back; otherwise the key reaches the program, as Tab's byte. While a run is in flight and
  while the terminal is taken over, the footer shows a bar: "Claude Code ran `cargo test`" with
  Take Over, or "You have control" with Hand Back. A taken-over terminal refuses `terminal_run`
  with the reason.
- The stand-in agent (`script/e2e/browser-fixture.sh`) gains `terminal-run <title> <command>`.

### Out (explicitly deferred)
- Typing into a running program (#525's `terminal_type`) and prompts to agent CLIs (the session
  verbs of plan slice C4).
- Whether the command enters the user's shell history and autosuggestions: #553, on the typing
  path this ticket adds.
- The agent's runs in the rail's row ("Claude Code · running cargo test in terminal 3", the
  blocks note's item 5) and an Agent chip in the terminal: #542's attention order is where the
  row's second line is decided.
- Warp's Auto-approve, "Run until completion", the denylist bypass setting, and per-project
  lists.
- Editing the command on the card before running it (Warp's CLI `E`), and guidance instead of
  yes or no.
- A `request_id` that makes a retried call type once.

## Reference (§20)
- **Warp, command execution under agent profiles**
  (https://docs.warp.dev/agents/capabilities/agent-profiles-permissions/, read 2026-09-26): the
  allowlist "run[s] automatically without confirmation"; "The denylist takes precedence over both
  the allowlist and `Agent decides`"; "If a command matches the denylist, the Agent asks for
  permission even when that action type is set to **Always allow**". The settings reference
  (https://docs.warp.dev/terminal/settings/all-settings/) gives the defaults:
  `agent_mode_command_execution_allowlist` = `cat(\s.*)?`, `echo(\s.*)?`, `find .*`,
  `grep(\s.*)?`, `ls(\s.*)?`, `which .*`; `agent_mode_command_execution_denylist` =
  `bash(\s.*)?`, `fish(\s.*)?`, `pwsh(\s.*)?`, `sh(\s.*)?`, `zsh(\s.*)?`, `curl(\s.*)?`,
  `eval(\s.*)?`, `exec(\s.*)?`, `source(\s.*)?`, `wget(\s.*)?`, `dig(\s.*)?`, `nslookup(\s.*)?`,
  `host(\s.*)?`, `ssh(\s.*)?`, `scp(\s.*)?`, `rsync(\s.*)?`, `telnet(\s.*)?`, `rm(\s.*)?`.
  Warp's "Always ask" is the `ask` value here; "Agent Decides" has no analog, since Marley's
  agents bring their own judgment and their own permission prompt. The approval card, Enter to
  run and the take-over control (`Ctrl+I` on Linux, Full Terminal Use,
  https://docs.warp.dev/agents/capabilities/full-terminal-use/) are kept. No Warp code; the
  behavior map `docs/warp_architecture/subsystems/04-agent-ai-mcp.md` describes the agent's
  command loop in Warp and was read for the shape only.
- **Upstream Zed:** nothing to keep. Zed's agents run commands in terminals they open
  themselves (`TerminalPanel::spawn_task`), never in the user's.
- **Orca:** report 06 §2.6 (`terminal send` answers with receipts), the same seam from the harness
  side; #525 maps the receipts.

### Prior art
- **Behavior maps and reports.** The Warp blocks note, item 4 ("What Marley would do") and
  recommendation 3, with its open question 5 (is Claude Code's prompt enough, or should Marley ask
  outside the allowlist), answered here as a setting that defaults to Claude Code's prompt.
  Orca report 06 items 1 and 2 (the caller behind a call; receipted input). The 525 spec: its
  `terminal.write` grant class, Ctrl-I, the 25-second approval, the card in the footer with a
  toast, and D1, which leaves the shell's prompt to this ticket. #520: `AppCall::caller`, which
  names the asking terminal.
- **Published material.** MCP 2025-06-18 tool results with `isError` for a refusal. Claude Code
  asks before each MCP tool call unless the tool is allowed, which is why `run` is the default
  outside the lists. Bash's `PS0` and zsh's `preexec` report the line as the shell read it, so a
  command typed with a return is exactly what the frame carries.
- **The code we already ship.**
  - `marley_mcp`: `Family` (`registry.rs:12`), `ToolSpec` with `grant_class` (`:46`), the terminal
    rows (`:78`, `terminal_read` at `:113`), `tool_schemas` exhaustive over families (`:285`) and
    `terminal_read_schemas` (`:916`), the shape the answer copies; `GrantTable` (`permission.rs:14`)
    and `decide` (`:54`), deny by default; `tools_call` (`dispatch.rs:133`), its permission check
    (`:153`) and the deferred arm for the app's families (`:165`); `APP_CALL_TIMEOUT_SECONDS`
    = 30 (`marley_mcp.rs:81`) and `AppCall` (`:131`), which bounds every wait here.
  - `marley_workbench::mcp`: `MAX_READ_LINES` and `MAX_READ_BYTES` (`mcp.rs:49`, `:52`), the
    grant of `browser.write` at start (`:89`), `answer` (`:334`, synchronous today; the async
    precedent is `browser_tools::answer`, `browser_tools.rs:53`), `terminal_with_id` (`:381`),
    `terminal_list` (`:397`), `terminal_blocks` (`:434`) and `block_entry` (`:477`),
    `terminal_read` (`:508`) with the redaction it applies.
  - The typing: `terminal_element.rs:2343-2352`, Rerun's nonce check and its
    `terminal.input(format!("\u{15}{command}\r"))`; `Terminal::input` (`terminal.rs:2316`),
    which notes the input's start for `typed_text`. The prompt state:
    `AnchoredBlocks::at_prompt` (`anchored.rs:176`), `apply` with the nonce check on `preexec`
    (`:116`), `PrecmdValue` without a nonce (`dcs.rs:47`); PR-claude-474 says a `precmd` field
    that an action rests on gets the nonce first, which D2 does. What was typed:
    `autosuggest::typed_text` (`autosuggest.rs:76`). The foreground: `Terminal::pid` (`:3277`,
    `tcgetpgrp` through `pty_info`) and `foreground_process_command_name` (`:3082`);
    `agent_bar::agent_in` (`agent_bar.rs:129`) for an agent CLI.
  - The block's end: `apply_shell_hook` (`terminal.rs:1817`) applies the frame and calls
    `cx.notify()` only; no `Event` variant exists for a hook (`Event`, `:729`). The waiter
    observes the terminal entity (`cx.observe`, as `terminal_view.rs:1156` does) and reads
    `blocks().last()`; `BlockTimes` (`anchored.rs:63`) gives the duration.
  - The footer hook `MarleyTerminalFooter` (`terminal_view.rs:141`, AD-claude-477) for the card
    and the bar; `workspace::Toast` with `on_click` for Show; `rich_input`'s container keeps keys
    from the terminal (F-claude-481).
  - Regexes: the `regex` crate (a workspace dependency; #516's `Redactor` compiles user patterns
    the same way). `shlex` 1.3 splits words, not pipelines, so the segment splitter of D3 is
    Marley's, pure, in `marley_terminal`.
  - Settings: `MarleySettingsContent` (`settings_content/src/marley.rs:10`), the Agents section
    (`settings_ui/src/marley_page.rs:42`), the `marley` block of `default.json` (`:1665`), and
    `MarleySettings` in `marley_workbench.rs:167`.
  - Does a crate we build own this seam? `marley_mcp` owns the tool and the grant, the terminal
    owns the typing and the blocks; nothing owns the lists or the pairing of a typed command with
    its block, which are new and pure.

## UI proof
UI-AFFECTING: the card, the bar, the agent mark, the setting.
`script/e2e/556-terminal-run.sh` (`compositor sway`, for the card's buttons). Fixtures: a scratch
repository with `notes.txt`; the scenario's own bash; the stand-in agent from
`browser-fixture.sh`, with `run`. Steps and shots: `run <id> "cat notes.txt"` on the idle terminal,
the block with the agent mark and the log's answer with exit 0 and the file's text
(`556-01-ran-allowlisted`); `run "printf 'x%.0s' {1..3000}"` outside both lists with the default
setting, run at once (`556-02-outside-lists`); `run "rm notes.txt"`, the card and the toast, nothing
typed (`556-03-card`); click Refuse, the refusal in the log (`556-04-refused`); `run` it again and
press Enter in the terminal, the block, the file gone (`556-05-ran-on-enter`); `ech` typed at the
prompt, then `run "ls"` refused for the typing, the prompt untouched (`556-06-typing-refused`);
`sleep 30` typed, `run "ls"` refused while it runs (`556-07-program-refused`); `run "sleep 40"`,
the answer at the deadline with `running: true`, then `terminal-read` after it ends
(`556-08-still-running`); Ctrl-I, the bar's "You have control", `run "ls"` refused
(`556-09-taken-over`); Ctrl-I again, `run "ls"` typed (`556-10-handed-back`); the setting at
`ask`, `run "printf hi"`, the card (`556-11-ask-outside-lists`); a forged `precmd` printed by
`cat` while `sleep 30` runs, `run "ls"` refused (the log); `mcp_agent blocks` showing `agent: true`
on the agent's blocks (the log); the Marley page's Agents section with the new items
(`556-12-settings`). Machine checks: `expect` on each answer's `exit_code`, `running` and
`refused` lines, and `holds` on the terminal's hook log for the typed bytes.

## Locked-In Decisions
- D1: Only a terminal whose shell waits at its prompt takes a command, and the whole line goes as
  Rerun sends it: Ctrl-U, the command, a return. No program, no agent CLI, no half-typed line
  (a refusal, not a Ctrl-U: the typing is Chad's).
- D2: The prompt is trusted only with the nonce. At promotion this had shipped with #526: both
  scripts sign `precmd` with `nonce=`, `decode_hook` wraps it in `DcsHook::Signed`, and
  `AnchoredBlocks::prompt_shell()` is `Some(PromptShell::Local)` only when the prompt's frame
  carried the terminal's own nonce. The tool needs `at_prompt()`, `prompt_shell() ==
  Some(PromptShell::Local)` and the foreground check; no script or `dcs.rs` change.
- D3: The lists are Warp's, as regexes anchored at both ends, tested against the whole command
  and against each segment split at `|`, `||`, `&&`, `;`, `&` and newlines outside quotes. The
  denylist asks when any pattern matches the command or any segment. The allowlist allows only
  when every segment matches a pattern and the command holds no `$(`, backtick, `<(`, `>(` or
  `<<`. Anything else is outside the lists, where `marley.agent_commands_outside_lists` decides:
  `run` by default, because the agent's own permission prompt already asked (Chad's question 5
  in the blocks note, answered by a setting he can flip).
- D4: The approval waits 25 seconds and the whole call answers within 25 (the transport's cap is
  30, `APP_CALL_TIMEOUT_SECONDS`), so `wait_seconds` is at most 20 and a run that outlives the
  deadline answers `running: true` with its index; `terminal_read` reads the rest. A late
  approval runs nothing.
- D5: Enter and Escape answer the card from the terminal it sits under, without taking the focus
  (changed at promotion): `enter` and `escape` in the Marley keymap's `Terminal` context bind
  `marley::RunAgentCommand` and `marley::RefuseAgentCommand`, which answer a run card waiting in
  the focused terminal and otherwise propagate, so the keys reach the program as before (the
  pattern of `AcceptSuggestion` on `right`). Run and Refuse work from anywhere.
- D6: Ctrl-I, as #525 binds it, acts only in a terminal where an agent has run a command; before
  that it is Tab's byte and reaches the program. Taking over stops the agent, never the user,
  and a hand-back needs no agent; the bar's buttons do the same.
- D7: The mark rides the chip hook (`MarleyBlockChip`, #555), which `bookmarks.rs` composes
  (changed at promotion: the hooks had landed with #555, #558 and #559): an agent's block gets an
  icon before the pill, beside a bookmark's, with a tooltip naming who ran it. No Zed hunk.
- D8: The agent's block is the first verified `preexec` after the typing whose command equals the
  one typed (whitespace trimmed); a block nobody typed through the tool is never marked. The
  pairing lives in `marley_workbench`, keyed by the view's entity id like `agent_events`'s seats.
- D9: Everything the tool answers passes #516's redaction, as `terminal_read` does; the card shows
  the command on Chad's own screen, unredacted.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `terminal_run` for a terminal whose shell waits at a verified prompt with nothing typed, and the command matches the allowlist, the system shall type Ctrl-U, the command and a return, and answer with the block's index, exit code, duration and output. | Shot `556-01-ran-allowlisted`; the log's answer |
| REQ-002 | WHERE `marley.agent_commands_outside_lists` is `run`, a command outside both lists shall run at once. | Shot `556-02-outside-lists` |
| REQ-003 | WHEN the command matches the denylist, the system shall show a card in that terminal's footer with the command, Run and Refuse, and a toast with Show, and shall type nothing before Run or Enter. | Shot `556-03-card` |
| REQ-004 | WHEN the user clicks Refuse or presses Escape on the card, the system shall answer the call as refused and type nothing. | Shot `556-04-refused`; the log |
| REQ-005 | WHEN the user clicks Run or presses Enter on the card, the system shall type the command and answer with its block. | Shot `556-05-ran-on-enter` |
| REQ-006 | WHILE something is typed at the prompt, `terminal_run` shall refuse and leave the typed text as it was. | Shot `556-06-typing-refused` |
| REQ-007 | WHILE a program is in the foreground, or the current prompt carries no nonce, `terminal_run` shall refuse with the reason. | Shot `556-07-program-refused`; the log's forged-frame check |
| REQ-008 | WHEN the block is still running at the deadline, the system shall answer `running: true` with the output so far and the block's index. | Shot `556-08-still-running`; the log |
| REQ-009 | WHILE a block an agent ran is on screen, its pill shall carry the agent mark, and `terminal_blocks` shall report `agent: true` for it. | Shots `556-01`, `556-05`; the log's `blocks` |
| REQ-010 | WHEN the user presses Ctrl-I in a terminal where an agent has run a command, the system shall refuse `terminal_run` there until Ctrl-I or Hand Back, and the bar shall say the user has control. | Shots `556-09-taken-over`, `556-10-handed-back` |
| REQ-011 | WHERE `marley.agent_commands_outside_lists` is `ask`, a command outside both lists shall show the card. | Shot `556-11-ask-outside-lists` |
| REQ-012 | WHEN the Marley page shows, its Agents section shall offer the outside-lists setting and name the two lists. | Shot `556-12-settings` |
| REQ-013 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, check whether #525 (Ctrl-I, `terminal.write`, the card),
  #520 (the caller) and #528 (`MarleyBlockExtras`) have landed, and take their pieces instead of
  adding them; ask the brain.
- **P2 Code:** the ledger rows first (`crates/terminal_view/src/terminal_element.rs` for the hook
  if it is new; `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`
  and `assets/settings/default.json` for the settings; `crates/terminal/src/terminal.rs` only if
  `PrecmdValue`'s nonce needs a hunk there); the scripts' `precmd` nonce and `prompt_verified`;
  the lists' pure module; the tool in `marley_mcp` and its answer in `marley_workbench`; the
  card, the bar, the action and the mark; fmt and clippy clean; a review of the diff with a
  security lens: nothing typed without a verified prompt, an empty input line, a foreground
  shell, an approval where one is due, and no take-over.
- **P3 Test:** write and run the scenario and read every shot; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley/three-prong-plan.md` (D9's `terminal.run` shipped);
  `docs/marley_architecture/` for `marley_mcp`, `marley_terminal` and `marley_workbench`; the
  ledger capture; close the ticket, archive, commit.
