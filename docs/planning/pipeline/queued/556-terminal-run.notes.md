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
- **`marley_terminal::agent_commands`** (pure): `segments(command) -> Vec<String>` (split at
  the operators of D3 outside single and double quotes; a backslash escapes the next character);
  `has_substitution(command)`; `enum Verdict { Allowed, Ask, Outside }` and
  `verdict(command, allow: &[Regex], deny: &[Regex]) -> Verdict` by D3's rules; `WARP_ALLOWLIST`
  and `WARP_DENYLIST` as the defaults. Patterns anchor with `^(?:…)$` when compiled; a pattern
  that fails to compile is logged and skipped, never allows.
- **The prompt's nonce.** `marley.bash`'s `__marley_precmd` and `marley.zsh`'s print
  `;nonce=%s` after `pwd`; `PrecmdValue { nonce: Option<String> }`; `AnchoredBlocks::apply`'s
  `Precmd` arm stages the prompt with `verified = self.nonce.is_some() && value.nonce ==
  self.nonce`; `prompt_verified() -> bool`. The `bootstrapped`, `init` and `history` frames stay
  as they are.
- **The tool.** `ToolSpec { family: Terminal, verb: "run", tier: Write, grant_class:
  "terminal.write" }`, served; input schema `terminal` (integer), `command` (string, 1 to 4,096
  bytes), `wait_seconds` (0 to 20); output as the spec lists. `mcp.rs` grants `terminal.write`
  beside `browser.write`. `answer` routes `terminal_run` to a new `terminal_run::answer(call,
  cx)` in `marley_workbench` that spawns on the foreground executor, as the browser tools do.
- **`marley_workbench::terminal_run`** (new module):
  - `Runs` global: per view id, `Option<InFlight { command, agent: Option<AgentKind>, started,
    block: Option<usize>, approval: Option<Approval> }>`, `taken_over: bool`, and the marks:
    `BTreeMap<usize, AgentRun { agent, command }>`; forgotten on the view's release.
  - `check(view, command, cx) -> Result<(), String>`: the id; `foreground is the shell` from
    `Terminal::pid()` against the shell's pid (`pty_info`'s fallback), and no agent CLI;
    `marley_anchored().at_prompt() && prompt_verified()`; `typed_text` empty; not taken over; one
    run at a time per terminal.
  - The verdict from the settings; `Ask` and `Outside` under `ask` show the card (the footer
    renderer reads `Runs`, draws the card with Run and Refuse, and a `MarleyCommandCard` key
    context whose `enter` and `escape` bind `marley::RunAgentCommand` and
    `marley::RefuseAgentCommand` in the Marley keymap; the card's focus handle takes the focus
    when the terminal had it, and gives it back), a toast with Show; a `oneshot` answered by
    either button, the keys, or the 25-second timer.
  - The typing: `terminal.update(cx, |t, _| t.input(format!("\u{15}{command}\r")))`, then the
    wait: `cx.observe(&terminal)` looking for a verified block whose command trims to the typed
    one and whose index is past the count at typing time; it becomes the run's block and the
    mark; then for `Finished`, or the deadline. The answer copies `terminal_read`'s route
    (`block_output`, redaction, `tail`) plus `exit_code`, `duration_ms` from `BlockTimes`, and
    `running`.
  - `marley::TakeOverTerminal`: on the focused terminal; if `Runs` has marks or a run there,
    toggle `taken_over` (a run in flight is answered refused) and `cx.notify()`; else
    `cx.propagate()` so Tab reaches the program. The bar in the footer: "<agent> ran `<command>`"
    with Take Over while a run is in flight, "You have control" with Hand Back while taken over.
- **The mark.** `terminal_view::MarleyBlockExtras { marks: Arc<dyn Fn(&Entity<Terminal>, &App)
  -> BlockMarks>, buttons: Arc<dyn Fn(&Entity<Terminal>, &AnchoredBlock, &App) ->
  Vec<AnyElement>> }` (a Global, set by `marley_workbench`); the element calls `marks` once per
  frame and draws an `IconName::Ai` (or the agent's icon from `agents::cli_icon`) before the pill
  of each marked block, with a tooltip; `buttons` is empty here. `block_entry` gains `agent`.
- **Settings.** `agent_command_allowlist: Option<Vec<String>>`, `agent_command_denylist:
  Option<Vec<String>>`, `agent_commands_outside_lists: Option<AgentCommandsOutsideLists>`
  (`Run`, `Ask`, with `strum` for the page's dropdown as `MarleyLayout` has); defaults in
  `default.json`; the page's Agents section gains the dropdown, whose description names the two
  lists.
- **File manifest.** Marley crates: `crates/marley_terminal/src/{agent_commands.rs (new),
  anchored.rs, dcs.rs}`, `crates/marley_terminal/shell_integration/{marley.bash, marley.zsh}`;
  `crates/marley_mcp/src/registry.rs`; `crates/marley_workbench/src/{terminal_run.rs (new),
  mcp.rs, agent_bar.rs, marley_workbench.rs}`, `crates/marley_workbench/keymap.json`. Zed paths:
  `crates/terminal_view/src/terminal_element.rs` (the hook and the mark),
  `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `assets/settings/default.json`. Scripts: `script/e2e/browser-fixture.sh` (`run`),
  `script/e2e/556-terminal-run.sh`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: the `terminal_element.rs` row gains the
  hook and the mark (or the hook's row from #528 gains the mark); the three settings rows gain
  the new keys.

### E2E plan
The stand-in's `run` calls `terminal_run` and prints the answer's fields; `terminals` gives the
id. The scenario's bash has `HISTCONTROL` unset and a plain prompt.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | `run <id> "cat notes.txt"` at the idle prompt | `556-01-ran-allowlisted`: the block, its mark; the log: exit 0, the text |
| REQ-002 | `run <id> "printf 'x%.0s' {1..3000}"`, default setting | `556-02-outside-lists`: ran at once |
| REQ-003 | `run <id> "rm notes.txt"` in the background | `556-03-card`: the card and the toast; the prompt empty |
| REQ-004 | click Refuse | `556-04-refused`; the log's refusal |
| REQ-005 | `run` again; press Return in the terminal | `556-05-ran-on-enter`: the block; `notes.txt` gone (log) |
| REQ-006 | type `ech`; `run <id> "ls"` | `556-06-typing-refused`: `ech` still at the prompt; the log |
| REQ-007 | type `sleep 30` Return; `run <id> "ls"`; then a `cat forged.txt` holding a `precmd` frame during a second `sleep 30`, `run` again | `556-07-program-refused`; the log's two reasons |
| REQ-008 | `run <id> "sleep 40"` after the sleeps end | `556-08-still-running`: `running: true` in the log; `terminal-read sleep` after 45 s |
| REQ-009 | `mcp_agent blocks` | the log: `agent: true` on blocks 1, 2 and 5 only |
| REQ-010 | Ctrl-I; `run <id> "ls"`; Ctrl-I; `run <id> "ls"` | `556-09-taken-over`, `556-10-handed-back` |
| REQ-011 | the setting set to `ask` in the profile's settings (516's `set_marley`); `run <id> "printf hi"` | `556-11-ask-outside-lists`: the card |
| REQ-012 | `marley::OpenSettings` | `556-12-settings` |

Not reachable by a scenario: a real Claude Code calling the tool (its own permission prompt
sits in front; L-claude-482's pty method could show one call, and Test may take it as far as a
`claude -p` with the plugin allows); the caller's name on the card needs #520.

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
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
