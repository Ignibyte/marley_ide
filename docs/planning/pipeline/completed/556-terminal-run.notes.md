# terminal_run: an agent runs commands in the user's terminal, as blocks — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-556-terminal-run.md
- **Pipeline spec:** 556-terminal-run.spec.md

## Phase 1 — Plan
- **Request:** the Warp blocks note (2026-09-25), item 4 and recommendation 3: "`terminal_run` with
  Warp's list defaults, the agent mark and a takeover key. M. It is the 'agent takes natural
  language and executes' piece, for every agent Marley hosts." Chad, 2026-09-26: every remaining
  Warp finding gets built. Plan D9 reserves `terminal.run` behind a grant; #525's D1 leaves the
  shell's prompt to this ticket.
- **Classification / tier:** feature, size M. One tool in `marley_mcp`, its answer and the pairing
  in `marley_workbench`, a pure lists module and the `precmd` nonce in `marley_terminal`, three
  settings, a card, a bar, an action, a mark. Zed paths: the settings trio (rows exist), the
  element's hook (a new row unless #528 added it), and `terminal.rs` only if `PrecmdValue`'s new
  field needs a hunk in `apply_shell_hook` (the decoder is Marley's, so likely none).
- **Recall (§18.3):**
  - F-claude-474-rerun-would-have-run-a-command-that-output-printed-001 and
    PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001: a frame is output
    until its nonce proves it. Rerun checks `command_verified`; an action resting on a `precmd`
    field adds the nonce to `precmd` first. D2 does exactly that for the prompt state.
  - AD-claude-491-marleys-mcp-server-runs-in-the-app-behind-a-stdio-bridge-001: tools whose
    answers are the app's come back deferred and are answered on the main thread; a wait longer
    than `APP_CALL_TIMEOUT_SECONDS` (30) is refused by the transport, and the bridge gives up at
    40. D4's 25-second budget comes from here.
  - AD-claude-477-a-footer-hook-in-zeds-terminal-view-and-the-bar-in-marleys-crate-001: anything
    under a terminal goes through `MarleyTerminalFooter`; the card and the bar do.
  - F-claude-481-the-rich-input-dropped-every-typed-character-on-linux-001: a footer element
    that takes keys stops the terminal's `key_down` on its own container and never relies on
    `prefer_character_input`. The card's Enter and Escape are actions in the card's context.
  - L-claude-477-a-quiet-foreground-process-is-seen-only-after-output-001: the foreground
    process is refreshed on output; a scenario's stand-in program must print before the check.
  - L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001 for the regex compile
    off the main thread, if it is ever slow.
  - The 525 spec (queued): the same card, toast, Ctrl-I and grant class; the 520 spec: the caller.
  - Brain: no page on agent-run commands or allowlists (searched 2026-09-26).
- **Recall at promotion (2026-09-29):**
  - #525 shipped `terminal_drive.rs`: the `Drives` global (generation, program, approval,
    `taken_over`, the last write, a `Pending` write with its oneshot), the card and the bar in the
    footer (`footer`, composed by `agent_bar::footer_without_agent`), the toast with Show, the
    25-second wait, `click_pause::Who` for the caller's words, and `TakeOverTerminal` on Ctrl-I.
    `mcp.rs` grants `terminal.write` at start and hands `terminal_type` to `type_into`.
    AD-claude-525: the shell's prompt was left to this ticket.
  - PR-claude-474: a frame is output until its nonce says otherwise. #526 already signs `precmd`
    (both scripts send `nonce=`; `decode_hook` wraps it in `Signed`), and `prompt_shell()` is
    `Local` only for the terminal's own nonce: D2 needs no new frame field.
  - The chip hook (`MarleyBlockChip`) is composed in `bookmarks.rs` (#559's one hook per slot):
    the mark joins it. The Marley keymap's `right` → `AcceptSuggestion` propagates when it has
    nothing to do; Enter and Escape follow it (D5).
  - Brain (consultation 6b516716): nothing on this seam.
- **Seams re-verified** (at `878d059cd4`): `PrecmdValue` (`dcs.rs:44`), `decode_hook`'s `Signed`
  wrap (`dcs.rs:253`), `AnchoredBlocks::apply`'s Precmd arm (`anchored.rs:266`),
  `prompt_shell()` (`:319`), `at_prompt()` (`:371`), `rerun_offered` (`:327`); Rerun's typing
  (`terminal_element.rs:2511-2523`, `\u{15}{command}\r`) and `blocks::reinput`
  (`blocks.rs:217`); `autosuggest::typed_text` (`autosuggest.rs:79`, private; `None` off the
  prompt, in vi mode, scrolled, on the alternate screen); `MarleyBlockChip`
  (`terminal_view.rs:197`) and `bookmarks::chip` (`bookmarks.rs:181`); `mcp::answer`
  (`mcp.rs:470`), `terminal_of` (`:766`), `block_entry` (`:867`), `terminal_read` (`:900`) and its
  redact-then-tail (`:938`), `MAX_READ_LINES`/`MAX_READ_BYTES` (`:51`, `:54`);
  `registry.rs`'s `terminal_type` row (`:157`) and `terminal_schemas` (`:1230`, a new verb needs
  its arm); the fixture's `terminal-type` (`browser-fixture.sh:830-860`); the keymap's
  `Terminal` block (`keymap.json:22`). The registry's count tests (`registry.rs:1579`,
  `dispatch.rs:302`) are stale already and not run (§7); they must only compile.
- **Discovery:**
  - `crates/marley_mcp/src/registry.rs`: `Family` (12), `ToolSpec` (46), `REGISTRY` (78; the
    terminal rows 93 to 121), `tool_schemas` (285), `terminal_schemas` (811), the read schema
    (916). `permission.rs`: `GrantTable` (14), `Tier` (31), `decide` (54). `dispatch.rs`:
    `tools_call` (133), the check (153), the deferred arm (165). `marley_mcp.rs`: the 30-second
    cap (81), `AppCall` (131). The registry's own test counts five tools (977 to 998) and
    `dispatch.rs:269` counts two; both look stale against the browser rows and are not run
    (§7), but a new row changes what they assert.
  - `crates/marley_workbench/src/mcp.rs`: the caps (49, 52), the grants (89), `answer` (334),
    `terminal_with_id` (381), `terminal_list` (397), `terminal_blocks` (434), `block_entry`
    (477), `terminal_read` (508), `tail` after it. `browser_tools.rs:53` answers a call from a
    spawned task, the shape a waiting tool needs.
  - `crates/terminal_view/src/terminal_element.rs`: the at-prompt check for Rerun (1632, the
    last block `Finished`), `marley_block` (2320), the nonce check (2343), the typing (2350),
    `marley_pill` (2262), the hover row (2366). `crates/terminal/src/terminal.rs`: `Event` (729),
    `process_event` (1726), `apply_shell_hook` (1817, `cx.notify()` only), `blocks` (1845),
    `marley_anchored` (1852), `block_output` (1859), `input` (2316), `pid` (3277),
    `foreground_process_command_name` (3082). `crates/marley_terminal/src/anchored.rs`:
    `AnchoredBlock` (38), `BlockTimes` (63), `apply` (101, the `preexec` nonce at 116, `precmd`
    at 129), `at_prompt` (176), `note_input` (182), `input_start` (190). `dcs.rs`:
    `PreexecValue` (37), `PrecmdValue` (47), `DcsHook` (56), the `precmd` decoder arm (177).
    The scripts: `marley.bash:36-41` and `marley.zsh:49-54` print `precmd;exit=…;pwd=…` with no
    nonce.
  - `crates/marley_workbench/src/autosuggest.rs`: `typed_text` (76). `agent_bar.rs`: `agent_in`
    (129), the footer renderer (149). `agent_events.rs`: the seat per view (30), `seat_id` (68).
    `marley_agent.rs`: `agent_kind_of` (76).
  - Settings: `marley.rs:10-29`, `marley_page.rs:42-91`, `default.json:1665-1674`,
    `marley_workbench.rs:167-198`.
  - Keymap: `crates/marley_workbench/keymap.json:17-23` (the `Terminal` block); Zed's `Terminal`
    context binds nothing to `ctrl-i` (`default-linux.json:1295-1341`).
  - e2e: `browser-fixture.sh:76-81` (`mcp_agent`), `write_mcp_agent` (205 to 479, the
    `terminals`, `blocks` and `terminal-read` commands to copy for `run`); `491-marley-mcp.sh`
    for a scenario that reads blocks over the bridge.
- **Decisions:** D1 to D9 in the spec.

### Design
- **`marley_terminal::agent_commands`** (new, pure): `segments(command) -> Vec<String>` (split at
  `|`, `||`, `&&`, `;`, `&` and newlines outside single and double quotes; a backslash escapes the
  next character); `has_substitution(command)` (`$(`, a backtick, `<(`, `>(`, `<<`);
  `enum Verdict { Allowed, Ask, Outside }` and `verdict(command, allow: &[String], deny:
  &[String]) -> Verdict` by D3 (each pattern anchored `^(?:…)$`, compiled on the call; one that
  does not compile is logged and matches nothing, so it never allows); `WARP_ALLOWLIST` and
  `WARP_DENYLIST`, Warp's defaults verbatim.
- **The tool.** `registry.rs`: `ToolSpec { family: Terminal, verb: "run", tier: Write,
  grant_class: "terminal.write" }` and `terminal_run_schemas()` (input `terminal`, `command`
  1 to 4,096 bytes, `wait_seconds` 0 to 20; output `terminal`, `block`, `command`, `running`,
  `exit_code`, `duration_ms`, `output`, `truncated`, `redacted`), its arm in `terminal_schemas`.
  `mcp.rs`: `terminal_run` goes to `terminal_drive::run_at_prompt(call, cx)`, as `terminal_type`
  goes to `type_into`; `block_entry` gains `agent`; `tail` becomes crate-visible for the answer.
- **`terminal_drive.rs` grows the run** (one module for both tools, so one take-over and one card
  stop both):
  - `Drive` gains `runs: BTreeMap<usize, Ran>` (the blocks an agent ran: who and the command) and
    `running: Option<Ran>` (the run in flight, for the bar); `Pending` gains its words and its two
    labels, so the card reads `<who> wants to run <command>` with Run and Refuse, or #525's words
    with Allow and Deny.
  - `run_at_prompt`: `check_run` refuses no terminal, an empty or long command or one with a
    newline, a program in the foreground (`program_of`) or an agent CLI (`agent_in`), no prompt
    or one not signed by the terminal's own shell (`at_prompt`, `prompt_shell`), typing at the
    prompt or an input line it cannot read (`autosuggest::typed_text`, made crate-visible), a
    take-over, a card already waiting or another run in flight. Then `agent_commands::verdict`
    with the settings: `Allowed`, or `Outside` under `run`, types at once; `Ask`, or `Outside`
    under `ask`, shows the card and the toast and waits as #525's write does.
  - The typing: `\u{15}{command}\r` through `Terminal::input`, as Rerun sends it, after
    re-checking the prompt. The wait polls the terminal every 50 ms until the call's deadline
    (25 s from the call, and at most `wait_seconds` after the typing): the first block past the
    count at typing whose command trims to the typed one and is `command_verified` becomes the run
    (D8) and its mark; its `Finished` ends the wait. The answer reads the block as
    `terminal_read` does (`block_output`, `for_agents`, `tail`), with `exit_code` and
    `duration_ms` from `BlockTimes`, or `running: true`.
  - `RunAgentCommand` and `RefuseAgentCommand` (Enter and Escape in `Terminal`) answer a run card
    waiting in the focused terminal and propagate otherwise. `toggle_control` also acts once an
    agent has run a command there. The bar reads `<who> ran <command>` with Take Over while a run
    is in flight.
  - `agent_mark(terminal, index, cx)`: an `IconName::Ai` in `Color::Muted` with the tooltip
    `Run by <who>`, which `bookmarks::chip` puts before the bookmark and the Ask chip.
- **Settings.** `agent_command_allowlist` and `agent_command_denylist: Option<Vec<String>>`,
  `agent_commands_outside_lists: Option<MarleyAgentCommandsOutsideLists>` (`Run`, `Ask`; strum
  for the dropdown, as `MarleyAgentTerminalWrites`); `MarleySettings` fields; `default.json` with
  Warp's lists; the Agents section's dropdown, whose description names the two lists; the renderer
  in `settings_ui.rs`.
- **File manifest.** Marley: `crates/marley_terminal/src/agent_commands.rs` (new),
  `marley_terminal.rs`; `crates/marley_mcp/src/registry.rs`; `crates/marley_workbench/src/
  {terminal_drive.rs, mcp.rs, autosuggest.rs, bookmarks.rs, marley_workbench.rs}`,
  `crates/marley_workbench/keymap.json`. Zed: `crates/settings_content/src/marley.rs`,
  `crates/settings_ui/src/marley_page.rs`, `crates/settings_ui/src/settings_ui.rs`,
  `assets/settings/default.json`. Scripts: `script/e2e/browser-fixture.sh`
  (`terminal-run`), `script/e2e/556-terminal-run.sh`.
- **Ledger rows.** The three settings rows and the `settings_ui.rs` row gain the new keys and the
  renderer.

### E2E plan
`compositor sway`. The scratch repository holds `notes.txt`; the scenario's bash has a plain
prompt. `mcp_agent terminal-run <title> <command> [--wait N]` calls the tool and prints the
answer's fields, or the refusal. Settings changes go through the profile's settings file.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, 009 | `terminal-run repo "cat notes.txt"` | `556-01-ran-allowlisted`: the block and its mark; the log: exit 0, the file's text |
| REQ-002 | `terminal-run repo "printf 'x%.0s' {1..30}"` | `556-02-outside-lists`: ran at once |
| REQ-003 | `terminal-run repo "rm notes.txt"` in the background | `556-03-card`: the card and the toast; nothing typed |
| REQ-004 | click Refuse | `556-04-refused`; the log's refusal; `notes.txt` still there |
| REQ-005 | the same run again; Return in the terminal | `556-05-ran-on-enter`: the block; `notes.txt` gone |
| REQ-006 | `ech` typed; `terminal-run repo ls` | `556-06-typing-refused`: `ech` still at the prompt; the log |
| REQ-007 | `sleep 8` running; `terminal-run repo ls` | `556-07-program-refused`; the log |
| REQ-008 | `terminal-run repo "sleep 12" --wait 2` | `556-08-still-running`: `running: true`; later `terminal-read sleep` |
| REQ-009 | `mcp_agent blocks` | the log: `agent` on the agent's blocks only |
| REQ-010 | Ctrl-I; `terminal-run repo ls`; Ctrl-I; `terminal-run repo ls` | `556-09-taken-over`, `556-10-handed-back` |
| REQ-011 | `agent_commands_outside_lists: ask`; `terminal-run repo "printf hi"`; Escape | `556-11-ask-outside-lists`: the card; the refusal |
| REQ-012 | `marley: open settings`, search `Agent Commands` | `556-12-settings` |

Not reachable by a scenario: a real Claude Code calling the tool behind its own permission
prompt; a forged `precmd` at an idle prompt (a frame needs a program to print it, and while one
runs the foreground check refuses first), so D2's check is review.

### Risks
- A `precmd` frame with the nonce is a change to the shell scripts every terminal loads; a
  terminal started by an older Marley build keeps printing frames without it after an update,
  and `prompt_verified` stays false there: the tool refuses until that terminal restarts, and
  says so.
- bash's `PS0` runs `fc -ln -0` from history: with the user's `HISTCONTROL=ignorespace`, a
  command typed with a leading space reports the previous command in its frame, so D8's pairing
  by command text fails and the block gets no mark. #553 types a leading space on purpose and
  settles it; here Marley types none.
- The card's focus grab: a user typing in that terminal at the moment the card appears could
  press Enter into the card. The tool refuses while anything is typed, which narrows it to an
  Enter pressed on an empty line.
- The 25-second budget makes long commands two calls (`run`, then `terminal_read`); the answer
  says so in words, so an agent that reads it does the right thing.
- The registry's stale tests (`registry.rs:977`, `dispatch.rs:269`) must still compile with the
  new row (gate:2 builds tests); their assertions are not run (§7) but are updated for honesty.

## Phase 2 — Code
- **Built to the manifest.** `marley_terminal::agent_commands` (`segments`, `has_substitution`,
  `Verdict`, `verdict`, `WARP_ALLOWLIST`, `WARP_DENYLIST`; patterns anchored and compiled per
  call, one that does not compile logged and left out). `registry.rs`: the `terminal_run` row,
  `terminal_run_schemas` and its arm. `mcp.rs`: `terminal_run` to
  `terminal_drive::run_at_prompt`, `block_entry`'s `agent`, `tail` crate-visible.
  `terminal_drive.rs`: `run_at_prompt`, `check_run`, `prompt_refusal`, `run_then_wait` (a 50 ms
  poll for the typed block and its end, within 25 s of the call and the run's wait of the
  typing), `type_run`, `ask_to_run`, `run_answer`, `answer_focused_run`, `running_run`,
  `run_by_agent`, `agent_mark`; `Drive` gains `runs` and `run_in_flight`; `Pending` carries its
  words and buttons, so one card serves both tools; `show_toast` and `dismiss_toast` take the
  workspace and view; `toggle_control` acts once an agent ran a command. `autosuggest::typed_text`
  crate-visible; `bookmarks::chip` puts the mark first. `RunAgentCommand` and `RefuseAgentCommand`
  with Enter and Escape in the keymap's `Terminal` block. The three settings, their fallbacks to
  Warp's lists, `default.json`, the Agents section's dropdown and its renderer. The fixture's
  `terminal-run` and `agent` in `blocks`.
- **Deviations.** At promotion (D2, D5, D7): no new frame field, since #526 signs `precmd`;
  Enter and Escape by propagating bindings, with no focus grab; the mark through the chip hook.
  `run_in_flight` guards the typing-to-answer window, since a second call inside it would pair
  with the first's block. Nothing typed since the prompt counts as an empty line: `input_start()`
  is unset until a key or a paste (`Terminal::input` notes both), and `typed_text` reads the line
  after that. A run whose block never starts answers an error naming `terminal_blocks`, since the
  schema's `block` is required. The mark is `IconName::Sparkle` (no generic agent icon exists).
- **Review** (the security lens of the phase plan): nothing is typed unless the foreground is the
  shell (`program_of`, `agent_in`), the prompt is signed by the terminal's own nonce
  (`prompt_shell() == Some(Local)`), the line is empty, the terminal is not taken over, and an
  approval was given where one is due; `type_run` checks the prompt again after a late answer.
  The denylist wins over the allowlist; the allowlist never allows a substitution; a broken
  pattern neither allows nor asks. The answer passes #516's redaction before its tail is cut.
  Re-entrancy: every update runs from the call's task through `cx.update`; the footer reads the
  terminal it is given (#595). Clippy's one red, `match_same_arms` in `segments`, fixed by a
  single catch-all arm.
- **Gate.** Run 1: `GATE GREEN [diff]`, 16 passed, 0 failed, the receipt written.

## Phase 3 — Test
- **Scenario** `script/e2e/556-terminal-run.sh` under `compositor sway`; the stand-in agent
  reaches Marley through the plugin's bridge. Run 1 passed every check.
- **Every shot read:**
  - `556-01-ran-allowlisted`: `$ cat notes.txt` and `hello from notes`, the block's pill with the
    sparkle mark before it; the answer: block 0, exit 0, the file's line. REQ-001, REQ-009.
  - `556-02-outside-lists`: `printf 'x%.0s' {1..30}; echo` ran with no card (outside both
    lists, the default `run`), marked. REQ-002.
  - `556-03-card`: the footer's card, Run, Refuse and `Stand-in agent wants to run rm
    notes.txt`, and the toast with Show; the prompt empty; `notes.txt` still there (it covers
    the printf block's pill, which shot 02 shows). REQ-003.
  - `556-04-refused`: after Escape, no card and no `rm` block; the answer `the user refused to
    run`. REQ-004.
  - `556-05-ran-on-enter`: after Enter on the card, the `rm notes.txt` block, marked; the file
    gone. REQ-005.
  - `556-06-typing-refused`: `$ ech` still at the prompt; the refusal names the typing. REQ-006.
  - `556-07-program-refused`: `sleep 8` running with no mark; the refusal names `sleep`. REQ-007.
  - `556-08-still-running`: `sleep 12` marked and running, and the bar `Stand-in agent ran sleep
    12` with Take Over; the answer `running True`; `terminal_read` read it after. REQ-008.
  - `556-09-taken-over`: the bar `You have control` with Hand Back; a run refused as taken over.
  - `556-10-handed-back`: after Ctrl-I again, `ls -a` ran and is marked. REQ-010.
  - `556-11-ask-outside-lists`: with `ask`, `printf hi` on the card and the toast; the click on
    Refuse refused it. REQ-011.
  - `556-12-settings`: Agent Commands Outside Lists under Marley › Agents, at Ask with its reset
    arrow, its description naming both lists. REQ-012.
  - The log: `terminal_blocks` says `agent True` for the agent's `cat` and `agent False` for the
    user's `sleep 8`. REQ-009.
- **Enter and Escape with no card** reached the shell throughout (every command the scenario
  typed ran), so the propagating bindings leave the terminal's keys as they were.
- **Not reachable:** a real Claude Code behind its own permission prompt; a forged `precmd` at an
  idle prompt (review). In a narrow terminal the pill column covers a long command's end, as it
  did before this ticket.

## Phase 4 — Complete
- **Docs.** CHANGELOG Added; the plan's D9 (`terminal.run` shipped as `terminal_run`);
  `marley_workbench.md` (the run in `terminal_drive.rs`), `marley_mcp.md` (the tool and its
  schemas), `terminal_blocks.md` (`agent_commands.rs`); the touchpoint rows for the settings
  files, `settings_ui.rs` and `default.json` describe what shipped.
- **Knowledge.** AD-claude-556-an-agent-runs-at-the-prompt-behind-two-lists-and-a-signed-prompt-001,
  L-claude-556-nothing-typed-since-the-prompt-is-an-unset-input-start-001. No F: the review and
  the visual check found no product bug. Brain: `brain_decide` on consultation 6b516716.
- **Closed** the ticket, archived the pair, committed.
