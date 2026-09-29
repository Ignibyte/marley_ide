# An agent reads and types into a running program — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-525-agent-drives-a-running-program.md
- **Pipeline spec:** 525-agent-drives-a-running-program.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-25)
- **Request:** Chad approved all seven items of the Warp once-over on 2026-09-25; on item 1, Full
  Terminal Use, "love this idea lets do it". The ticket as asked: an agent reads a running
  program's live screen and types into it, with a take-over key that stops its writes until the
  user hands back and write approval on the first write, every write or never; Marley tools in
  `marley_mcp`, shown in the terminal; rustal-harness's managed input as the model, since the
  harness will be embedded in Marley.
- **Classification:** feature, size M. Marley crates `marley_terminal` (the pure control state),
  `marley_mcp` (two rows and their schemas), `marley_workbench` (the answers, the bar, the card,
  the action). Zed crates, small and additive: `terminal` (a screen read by rows),
  `settings_content` and `settings_ui` (the setting and its dropdown), `assets/settings/default.json`
  (its default).
- **Order:** after #516 (active tonight), whose `Redactor` the screen read uses and whose Agents
  section of the Marley page takes the setting.
- **Recall (§18.3):**
  - AD-claude-492-agents-drive-the-browser-tab-through-the-mcp-server-001: write tools granted at
    start, the client's approval of each call and the tab the user watches as the checks. For
    terminals Chad asked for Marley's own approval as well, Warp's three modes.
  - AD-claude-477-a-footer-hook-in-zeds-terminal-view-and-the-bar-in-marleys-crate-001: anything
    under a terminal goes through `MarleyTerminalFooter`; the footer takes rows from the grid.
  - L-claude-477-a-quiet-foreground-process-is-seen-only-after-output-001: the foreground name
    refreshes on a `Wakeup`, which comes with output. The program check uses `pid()`, a live
    `tcgetpgrp`, and the name only for display.
  - L-claude-493-zed-gives-a-lost-focus-to-the-panes-front-item-001: a tab brought forward in the
    focused pane takes the focus, hence D6.
  - L-claude-491-a-session-per-agent-needs-room-and-a-close-001 and AD-claude-491: every Claude
    Code bridge is one MCP session; the app gets no caller identity (`AppCall` holds the tool,
    the arguments and the answer channel only).
  - F-claude-481 and L-claude-481: keys inside the terminal view must not be swallowed on Linux;
    the card has buttons and no text field, so nothing of that trap applies, and Ctrl-I's
    fall-through copies `marley::RichInput`'s `cx.propagate()`.
  - PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001: this ticket acts on
    no hook frame; the program comes from the PTY's process group, which output cannot forge.
- **Discovery (the seams, checked 2026-09-25):**
  - `crates/marley_mcp/src/registry.rs:78` `REGISTRY`, whose `terminal` rows (`list`, `blocks`,
    `read`) are all `Tier::Read`; `:229` `browser_write`, the pattern for a write row with its
    grant class; `:807` `terminal_schemas`, where the two new verbs get their schemas.
  - `crates/marley_mcp/src/permission.rs:14` `GrantTable`, `:54` `decide`: a write needs its class
    in the table, so `terminal.write` must be granted.
  - `crates/marley_mcp/src/dispatch.rs:133` `tools_call`: the permission check at `:153`, then
    `Outgoing::Deferred` for the `Terminal` and `Browser` families, so the app answers both tools.
  - `crates/marley_mcp/src/marley_mcp.rs:81` `APP_CALL_TIMEOUT_SECONDS` = 30; `:131` `AppCall`;
    `crates/marley_mcp/src/transport.rs:280` `ask_app` waits that long, then answers that Marley
    did not answer. `crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge:31`
    `REQUEST_TIMEOUT = 40`. So an approval must settle inside 30 seconds (D7).
  - `crates/marley_workbench/src/mcp.rs`: `start` grants `browser.write` (line 83 today); `answer`
    (285) routes by tool name; `terminal_with_id` (332); `terminal_read` (459). #516 is editing this
    file for the redaction, so its lines move.
  - `crates/terminal/src/terminal.rs:2305` `input`; `:2553` `try_keystroke` (Zed's `to_esc_str` with
    the program's modes); `:2582` `paste` (bracketed while `Modes::BRACKETED_PASTE` is set, else
    newlines become CRs); `:2619` `get_content` (the whole grid, wrapped lines joined); `:3071`
    `foreground_process_command_name`; `:3266` `pid` (the foreground process group); `:3273`
    `pid_getter`, whose `fallback_pid` is the shell. `crates/terminal/src/pty_info.rs:241`
    `PtyProcessInfo::pid` calls `ProcessIdGetter::pid` (`:33`), a live `tcgetpgrp` on the PTY that
    falls back to the shell's pid.
  - `crates/terminal/src/alacritty.rs:975` `absolute_lines_text`: from a line to the cursor's, so
    not a screen read; the new row reader goes beside it.
  - `crates/terminal_view/src/terminal_view.rs:130` `MarleyFooterContext`, `:141`
    `MarleyTerminalFooter`, called in `render` at `:1397`.
  - `crates/marley_workbench/src/agent_bar.rs:33` `init` sets the footer to `render` (`:149`), which
    returns `None` without an agent CLI; `:129` `agent_in`.
  - `crates/marley_workbench/src/rich_input.rs:49` `None => cx.propagate()`: the key goes on to the
    program.
  - `crates/marley_workbench/src/blocks.rs:63` `focused_terminal`.
  - `crates/marley_workbench/keymap.json:17` the `Terminal` bindings; Zed's
    `assets/keymaps/default-linux.json` binds no `ctrl-i` in `Terminal` (it binds it in the editor,
    the agent panel and the debugger: lines 120, 251, 330, 1160).
  - `crates/workspace/src/workspace.rs:730` `Toast`, `:747` `on_click`.
  - `crates/settings_content/src/marley.rs` (`MarleySettingsContent`, #516 adding its two fields);
    `assets/settings/default.json`'s `marley` block (#516); `crates/settings_ui/src/marley_page.rs`
    (#515, #516's Agents section); `crates/settings_ui/src/settings_ui.rs:559`, one dropdown
    renderer per enum (`MarleyLayout`'s).
  - `crates/marley_fleet/src/verbs.rs` `Receipt`.
  - `/srv/stacks/rustal-harness/docs/MANAGED_INPUT.md`: observer first (line 4), control by claim
    with `takeover: true` (55, 60), "supplies control, not a terminal screen renderer" (48), the
    program's output proves consumption (46), exact retries (88), 1 to 4,096 bytes (130). Its
    `docs/ROADMAP.md:226` (M9): sessions are driven "through receipted commands, never through
    keystrokes or screen reads", the reason agent CLIs are out of this tool.
  - `docs/marley/three-prong-plan.md:180` D9 (`terminal.run` grant-gated), `:187` D10 (a harness
    seat as a display-only terminal promoted by a claim), `:206` C3.
- **Decisions:** D1 to D9 in the spec. The setting's name, `agent_terminal_writes`, and its values
  follow Warp's three modes in Marley's words.

### Changed at promotion (2026-09-29; each item overrides the design below)
- **Checklist** (no task tool): pre-flight ✓ (no other active pipeline, cargo idle); recall ✓;
  the brain ✓ (nothing on this seam); promoted ✓; the seams re-verified by an Explore agent at
  5bbb1c3453 ✓.
- **#520 landed:** `AppCall::caller()` names the calling terminal and client, so the card names the
  agent (`click_pause::Who`), and a first write's approval holds for that program only.
- **#571 is the approval's template:** a 25-second wait raced against a oneshot, a toast with Show,
  a focus-guarded card; `terminal_type` copies its shape.
- **The screen** comes from `Terminal::last_content()` (the viewport's cells, the cursor, the mode),
  with `scrolled` in the answer when the user has scrolled back; no Zed touch in `alacritty.rs`.
- **The generation** follows the foreground process group's leader (`Terminal::pid()`, read at
  each call), since a program restarted under the same name sends no title change.
- **Keys** go through `gpui::Keystroke::parse` and `Terminal::try_keystroke` (`to_esc_str` is
  private to the terminal crate); a name `try_keystroke` does not know is refused before anything
  is typed.
- **Where things live:** the rows and schemas in `marley_mcp`'s registry; the grant in `mcp.rs`; the
  state, the answers, the approval and the take-over in a new `terminal_drive.rs`; the card and the
  bar in `agent_bar::render`'s no-agent branch (the footer hook); the setting beside
  `browser_click_pause_agents`.

### Design
- **Approach.**
  - *The screen.* `Terminal::marley_screen()` returns `MarleyScreen { rows, cursor_row,
    cursor_column, columns, alt_screen }`: each live screen row `Line(0)` to
    `Line(screen_lines - 1)` read alone, whatever the display offset, with trailing blanks
    trimmed, through a helper `screen_rows_text` beside `absolute_lines_text`. The workbench adds
    the program: `pid()` when it differs from `pid_getter().fallback_pid()`, named by
    `foreground_process_command_name`.
  - *The control state* (pure, `marley_terminal::control`, new): `TerminalControl { generation,
    program, taken_over, approved, last_write }`. `observe(program)` advances the generation and
    clears `approved` and `taken_over` when the program differs; `take_over` and `hand_back`
    advance it; `check(generation, mode)` answers `Stale`, `NoProgram`, `TakenOver`, `Ask` or
    `Allow`; `approve` records the program; `wrote(summary)` keeps the last write for the bar.
  - *The tools.* Two rows in `REGISTRY` (`terminal`, `screen`, `Tier::Read`; `terminal`, `type`,
    `Tier::Write`, `terminal.write`), their schemas, and `terminal.write` granted at start in
    `mcp::start`. `mcp::answer` routes both to a new `terminal_control` module in
    `marley_workbench`: `screen` observes, reads, redacts and answers; `type` observes, refuses
    an agent CLI (`agent_in`), checks, then types (`paste(text)`, `try_keystroke` per key,
    `try_keystroke("enter")`), or holds the call as a pending approval.
  - *The approval.* A pending approval per terminal view: the `AppCall`, the text to show, and a
    25-second timer (`cx.background_executor().timer`) that answers the call refused and clears
    the card. Allow approves (in `ask_first_write`), types and answers accepted; Deny answers
    refused. A toast (`Toast::new(..).on_click("Show", ..)`) in the view's workspace, dismissed
    with the card; Show reveals and focuses the terminal as #496's `reveal_terminal` does.
  - *The footer.* `agent_bar::init` sets the footer to a closure that stacks
    `terminal_control::element` (the card while an approval waits, else the driving bar while an
    agent has written to the program) over the agent bar, so a terminal can show both. The card
    and the bar are one row each and take the same row: a footer takes its rows from the grid
    (AD-claude-477), so the program's PTY is resized once when an agent first asks and once when
    the program ends, not at every approval.
  - *The key.* `marley::TakeOverTerminal`, a workspace action like `marley::RichInput`: on the
    focused terminal, take over or hand back while an agent has written to its program, else
    `cx.propagate()`. Taking over while an approval waits denies it.
  - *The setting.* `MarleySettingsContent::agent_terminal_writes: Option<AgentTerminalWrites>`
    (`AskFirstWrite`, the default, `AskEveryWrite`, `NeverAsk`, with strum's `VariantArray` and
    `VariantNames`), its default in `default.json`'s `marley` block, `MarleySettings` resolving it,
    an item in the Agents section of `marley_page.rs`, and its dropdown renderer.
- **The harness mapping, for C3** (a harness seat shown as a display-only terminal, plan D10):

  | Marley (#525) | rustal-harness managed input | On a harness seat |
  |---|---|---|
  | `generation` in `terminal_screen` | the claim generation from `inspect` | Marley's generation also advances on a harness claim change and on a runtime epoch change, so either side's change makes an agent re-read |
  | the screen's rows | none: the managed session "supplies control, not a terminal screen renderer" | the rows of Marley's display-only terminal, fed by the capture stream |
  | `terminal_type` | `input {generation, bytes}`, 1 to 4,096 literal bytes | Marley encodes the text and keys to bytes and sends one `input` on its own controller connection, with the harness generation it holds |
  | accepted, with the bytes | `input_submitted`; `native_input_incomplete` with the prefix count | accepted on `input_submitted`; a short write refused with its count |
  | Take Over and Hand Back | a claim with `takeover: true`; `release` | Marley's connection keeps the claim for the user and agents alike; Take Over stops forwarding agent writes and advances Marley's generation only |
  | a stale generation refused | a stale generation refused | the same refusal |
  | (Out) `request_id` | an exact retry on the connection returns the original response | add `request_id` then and pass it as the harness request id, which also answers MREQ-002's ask for sends |
  | the approval | none: a cooperative same-user contract | stays Marley's |

- **File manifest.**
  - Marley crates: `crates/marley_terminal/src/control.rs` (new), `marley_terminal.rs` (the
    module); `crates/marley_mcp/src/registry.rs`; `crates/marley_workbench/src/terminal_control.rs`
    (new), `mcp.rs`, `agent_bar.rs`, `marley_workbench.rs` (the action, the setting, the init),
    `keymap.json`; `script/e2e/525-agent-drives-a-running-program.sh` (Test).
  - Zed crates: `crates/terminal/src/terminal.rs` (`marley_screen`), `crates/terminal/src/alacritty.rs`
    (`screen_rows_text`), `crates/settings_content/src/marley.rs` (the field and the enum),
    `crates/settings_ui/src/marley_page.rs` (the item), `crates/settings_ui/src/settings_ui.rs` (the
    renderer), `assets/settings/default.json` (the default).
- **Ledger rows** (`docs/marley/zed-touchpoints.md`, written before the hunks): extend the rows of
  `crates/terminal/src/terminal.rs`, `crates/terminal/src/alacritty.rs`,
  `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `crates/settings_ui/src/settings_ui.rs` and `assets/settings/default.json` with the #525 hunks.

### For the quality pass
- No tests (§7, since 2026-09-29): the drafted scenario waits for the quality pass.

### Risks
- **The slice is large.** If Code finds it so, the fallback split keeps REQ-001 to REQ-007, REQ-011
  and REQ-012 (the tools, the first-write approval, take-over) and moves the other two modes, the
  expiry test and the setting's page item to a follow-up; typing must not ship without the
  approval and the take-over.
- **A program that owns the PTY without a process group change** (a shell function, `exec` in the
  shell) reads as no program; the write is refused, which fails safe.
- **ssh is a program.** One approval covers the whole ssh session, remote shell included, as
  Warp's process-scoped approval does; the card names `ssh` and the tool's description says so.
  `ask_every_write` is the answer for a session that matters.
- **25 seconds is short** for a user looking elsewhere; the toast is there for that, and the
  agent can ask again.
- **The row the bar takes** resizes the program's PTY when the bar first shows: readline programs
  redraw their line, a full-screen one its screen. The shots show sqlite3 after it.
- **Ctrl-I inside a program that uses it** (vim's jump forward) is Marley's while an agent drives
  that program (D5).
- **Two agents, one approval.** Without a caller, an approval given for one agent's first write
  lets any agent type into that program. #520 (queued the same night) adds `AppCall::caller`
  from the bridge's `Marley-Terminal` header. If it has landed at promotion, key the approval on
  the program and the caller, name the caller's terminal on the card, and refuse a write whose
  target is the caller's own terminal; if not, the ticket ships as specced and a follow-up takes
  those three lines.

## Folded in from the Orca second pass (2026-09-26)
Smaller item 1 of `docs/planning/design-notes/orca-second-pass-2026-09-25.md`: a terminal read
that leaves out what someone is typing. `terminal_screen` gives the rows as drawn, so with an agent
CLI in the foreground they carry the CLI's input box and whatever Chad has half-typed, and an
agent reading his Claude Code terminal could take the half-typed prompt for output.

- **What Orca does.** `orca terminal read` finds the agent CLI's input box by its prompt glyph at
  the cursor row (`❯` under a frame line of eight or more `─`, `━` or `-`, Claude Code's box; `›`
  or `»` for the others), keeps only the glyph in the returned rows, joins the box's continuation
  rows, and returns the typed text as a separate `draft` field, "UI-only composer text, excluded
  from `tail`" (`src/shared/terminal-composer-draft.ts`, 201 lines;
  `src/main/runtime/orca-runtime-terminal-projection.ts`; `src/shared/runtime-terminal-contracts.ts`;
  MIT, read).
- **For `terminal_screen`.** When the foreground program is an agent CLI (`agent_bar::agent_in`)
  and the cursor row starts with one of the three glyphs (the `❯` form only under a frame line),
  the answer's rows carry the glyph and not the typed text, and a `draft` field carries the typed
  text, redacted as the rows are (D9). No glyph row at the cursor, no `draft`, and the rows are as
  drawn: the heuristic is tied to each CLI's glyphs and breaks when a CLI changes its input box,
  so a miss falls back to the plain read rather than hiding rows. The glyph table is one constant
  in `marley_terminal`, pure, beside the control state. `terminal_read` (blocks) is unchanged.
- **Acceptance to add at promotion.** REQ-015: WHEN `terminal_screen` reads a terminal whose
  foreground program is an agent CLI and whose cursor row starts with that CLI's prompt glyph, the
  rows shall carry the glyph without the typed text and the answer shall carry the typed text as
  `draft`. Verify: the run log, with the stand-in `claude` drawing a frame line and `❯ ` and the
  scenario typing part of a prompt before the client's `screen`. The existing refusal (REQ-011,
  an agent CLI takes no `terminal_type`) stands; this item is about the read only.

## Phase 2 — Code
- **Checklist** (no task tool): the ledger rows (four Zed paths, the setting) ✓; the registry rows
  and schemas ✓; `terminal_drive.rs` ✓; the footer hook ✓; the setting, its dropdown, the page
  item and the default ✓; the action and Ctrl-I ✓; the grant and the dispatch ✓; the review ✓;
  the gate ✓.
- **Built as the changes at promotion say.** Compile and clippy asked for `ToolAnswer` answers,
  `Cell::character()` and `is_wide_char_spacer()` (the cell's field is private), the footer's weak
  view upgraded, the no-agent footer split out of `agent_bar::render` (past 100 lines), and `&App`
  where nothing mutates.
- **The review**, against each criterion:
  - The screen: rows from the viewport's cells; `scrolled` says when the user scrolled back.
  - A write is refused at the shell's prompt (the foreground pid is the shell's), into an agent
    CLI, for another generation, while taken over, and while another write waits; the generation
    is read again after the user's Allow.
  - A key that parses to a plain character is refused before anything is typed, since
    `try_keystroke` sends none for it.
  - Ctrl-I toggles only while an agent has typed into the focused terminal's program or the user
    holds it, and otherwise propagates to the program, as Ctrl-G does for the rich input.
  - The card's answer after the call stopped waiting finds no sender and types nothing.
- **The gate:** `just gate-diff` green: 16 passed, 0 failed, `GATE GREEN [diff]`, the receipt
  written.

---
## Phase 3 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented:** `CHANGELOG.md`; `docs/marley_architecture/marley_workbench.md` ("An agent that
  drives a running program"); `docs/marley_architecture/marley_mcp.md`; `docs/marley/guide.md`
  (the two tools, the approval and the take-over). The four touchpoint rows name the setting.
- **Knowledge:** AD-claude-525-an-agent-types-into-a-running-program-behind-a-generation-and-an-approval-001.
- **Brain:** consultation dfeb4e679408420f9b14e4feafeaceee closed with a decision (follow-up
  2026-10-29).
- **Closed:** TICKET-525 moved to `tickets/closed/`; its BACKLOG row went at promotion.
- **No tests** (§7): the drafted scenario waits for the quality pass.

---
## Phase 3 — Test (the visual check, after the fact)
- **Why after:** #525 shipped on 2026-09-29 while the workflow had no visual check; Chad brought it
  back the same day (7e589cb0a1).
- **The scenario:** `script/e2e/525-agent-types-into-a-terminal.sh`, `compositor sway`: a Python
  REPL in the project's terminal, and a stand-in agent reaching Marley's MCP server through the
  Claude Code plugin's bridge, with the fixture's new `terminal-screen <title>` and
  `terminal-type <title> [--generation n] [--submit] [--keys a,b] <text>`. Each submitted write
  is checked on the screen, not only by the tool's answer.
- **What it found, three bugs, each fixed or filed:**
  - #593: the write's toast covered the card's Deny and Allow at the footer's right end; a click
    on Allow hit the toast and the write timed out. The card now leads with Allow and Deny.
  - #594: the Enter went out in the same burst as the paste, and Python's REPL read it as part of
    the paste; nothing an agent submitted ran, while the tool said it typed. Keys and Enter now
    follow the paste after 200 ms, in the rich input and review notes too.
  - #595 (filed): the drive bar and Ctrl-I's take-over outlive the program an agent typed into.
- **The shots, read (#594's run 2):** `525-01-repl` (the REPL, `terminal_screen` naming
  `python3`, REQ-001); `525-02-ask` (the card with Allow and Deny and the toast, REQ-002);
  `525-03-typed` (`42` under `print(6 * 7)`, the bar with Take Over, REQ-003, REQ-005);
  `525-04-no-ask` (`again`, no card, REQ-004); `525-05-taken-over` ("You have control", Hand
  Back; a write refused, REQ-006); `525-06-handed-back` (the old generation refused, the new one
  typed, REQ-007); `525-07-asks-every-write` and `525-08-denied` (`ask_every_write` asks; Deny
  types nothing, REQ-008); `525-09-timed-out` (refused after 25 seconds, no card, REQ-010);
  `525-10-never-ask` (typed with no card, REQ-009); `525-11-shell-refused` (the shell at its
  prompt refused, REQ-011); `525-12-new-program-asks` (a new program asks again, REQ-002);
  `525-13-rich-input-open` and `525-14-rich-input-ran` (#594's rich-input check).
- **Not reached:** REQ-012 (Ctrl-I reaching a program no agent typed into) waits on #595;
  REQ-013 (the setting on the Marley page) was not opened.
