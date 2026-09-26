---
pipeline_id: bfc1ee6e-0e68-42e1-a777-19ca3eede102
ticket: docs/planning/tickets/open/TICKET-525-agent-drives-a-running-program.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "An agent reads and types into a running program"
type: feature
slice: prong 2 with prong 1 (plan D9's terminal tools; Warp once-over item 1)
references: [docs/planning/design-notes/warp-once-over-2026-09-25.md, docs/orca_architecture/06-cli-automations-skills.md, docs/planning/pipeline/completed/491-marley-mcp-in-the-app.spec.md, docs/planning/pipeline/completed/492-browser-tools-for-agents.spec.md]
---

## Title
An agent working beside a running program (psql, gdb, a Python REPL, a dev server) reads the
terminal's live screen and types into the program, with Chad's approval and a key that takes the
terminal back. Two tools on Marley's MCP server, `terminal_screen` and `terminal_type`, after
Warp's Full Terminal Use, with rustal-harness's managed input as the model for who may type and
when a write is stale.

## Scope
### In
- **`terminal_screen {terminal}`**, a read tool: the live screen's rows as text (the bottom of the
  grid, wherever the user has scrolled), the cursor's row and column, the size, whether the
  alternate screen shows, the foreground program's command name (none while the shell waits at
  its prompt), and the control state: `generation`, `taken_over`, the approval mode and whether
  this program's writes are approved. The rows pass through #516's `Redactor`, as
  `terminal_read`'s output does, and the answer carries its `redacted` count.
- **`terminal_type {terminal, generation, text?, keys?, submit?}`**, a write tool of grant class
  `terminal.write`: `text` as a paste (bracketed when the program turned bracketed paste on, as
  `Terminal::paste` sends it), then each of `keys` as a keystroke by Zed's names (`escape`,
  `ctrl-c`, `up`, `tab`), then Enter when `submit` is true; at most 4,096 bytes a call, the
  harness's bound. It is refused with a reason when the generation is not the terminal's current
  one, when no program runs in the foreground, when the foreground program is an agent CLI, while
  the user has taken over, and when the approval is denied or not given within 25 seconds. An
  accepted write answers with the bytes written, the program and the generation.
- **The approval**, the setting `marley.agent_terminal_writes`: `ask_first_write` (the default),
  `ask_every_write` or `never_ask`, a dropdown in the Marley page's Agents section (the section
  #516 adds). A pending approval shows as a card in the target terminal's footer, with the text
  to type and Allow and Deny, and as a toast in that terminal's workspace naming the terminal and
  the program, with Show. An answer that comes after the call stopped waiting types nothing.
- **The driving bar**, in the target terminal's footer while an agent has written to its current
  program: the program, the last write, and Take Over; after a take-over it reads "You have
  control" with Hand Back. `marley::TakeOverTerminal`, on Ctrl-I in `Terminal`, takes over or
  hands back while an agent has written to the focused terminal's program, and otherwise lets the
  key reach the program.
- **The generation**, per terminal: it advances when the foreground program changes (another
  process group leads the PTY), when the user takes over and when the user hands back. The
  first write's approval holds while the same program runs.
- `terminal.write` granted when the server starts, beside `browser.write` (#492 D2).

### Out (explicitly deferred)
- Typing at the shell's prompt, which runs commands: plan D9's `terminal.run` is its own ticket.
- Typing prompts to agent CLIs (Claude Code, Codex): Orca's receipted input, with bracketed
  prompts, `turn_started` receipts, `terminal_wait` and blocked reasons (report 06 item 2, plan
  slice C4).
- Naming which agent asks: Marley's bridge forwards no caller today (report 06 item 1), so the card
  says "An agent". #520, drafted the same night, gives `AppCall` its caller; when it has landed
  before this ticket starts, the card names the calling terminal and a first write's approval
  holds for that caller only (the notes' Risks).
- A `request_id` that makes a retried write type once, and harness seats themselves (C3): the
  notes map the tools onto the harness for that ticket.
- Longer approval waits: the transport answers a call within 30 seconds
  (`APP_CALL_TIMEOUT_SECONDS`) and the bridge gives up at 40.
- The agent's writes in the rail, per-program allowlists, and a setting that takes the grant
  away (grants stay in `marley_mcp`'s configuration, #515's Out).

## Reference (§20)
- **Warp, Full Terminal Use** (https://docs.warp.dev/agents/capabilities/full-terminal-use/; the
  Warp once-over, `docs/planning/design-notes/warp-once-over-2026-09-25.md`, item 1). The agent
  attaches to an interactive program (database shells, REPLs, debuggers, dev servers), "can see
  the live terminal buffer", and writes to the PTY to run commands and answer prompts.
  "Takeover control" (`Ctrl+I` on Linux) stops its actions and leaves the program to the user;
  the same control hands it back. Three global settings approve writes: "Ask on first write"
  ("The first write to a shell process requires approval. After that, all subsequent writes for
  that specific process/command will be approved."), "Always ask" and "Always allow". Secret
  redaction still applies. Marley keeps the program-scoped first write, the three modes, the
  take-over key and its toggle, and the redaction. It leaves out Warp's agent conversation (the
  proposal with its reasoning, Enter to allow once, Ctrl+Shift+I to auto-approve similar
  commands): Marley's agents are CLI agents in other terminals, so the approval sits in the
  terminal being typed into. No Warp code.
- **rustal-harness managed input** (`/srv/stacks/rustal-harness/docs/MANAGED_INPUT.md`). A
  connection starts as an observer; a claim gives control and a new generation; input and resize
  must carry the current generation; a takeover transfers control and advances it; a stale
  generation is refused; input is 1 to 4,096 literal bytes; a receipt says the backend took the
  bytes, and only the program's output proves it consumed them. Marley's generation, its bound
  and its refusals follow it, so the tool maps onto a harness seat later.
- **Upstream Zed:** nothing to keep. Zed's agents read a terminal with `Terminal::get_content`
  and run commands in terminals they open themselves.

### Prior art
- **Behavior maps and reports.** The Warp once-over, item 1 ("The Orca survey's receipted input
  (report 06, item 2) is the same seam"). Orca report 06 §2.6: `terminal read --screen` returns
  the rendered frame, "the only faithful read of a TUI", and labels its source `screen`,
  `stream` or `screen-unavailable` (`src/main/runtime/orca-runtime-resolve-terminal-pane.ts`,
  `readTerminal` and `readRenderedScreen`); `terminal send` answers with receipts staged
  `input_accepted` then `turn_started`
  (`src/main/runtime/agent-prompt-submission-verification.ts`); `terminal wait` answers a
  `blockedReason` instead of typing into a dialog (`RuntimeTerminalWaitBlockedReason`,
  `src/shared/runtime-terminal-contracts.ts`). Report 06 item 2 proposes `terminal_screen` and a
  `terminal.write` grant, the names taken here, and asks that each send show in the target
  terminal. The harness's `docs/ROADMAP.md` M9 keeps agent-to-agent control on receipted verbs,
  never keystrokes, which is why agent CLIs are refused here.
- **Published material.** MCP 2025-06-18 tool results, with `isError` for a refusal. Claude Code
  asks before each MCP tool call unless the tool is allowed, so Marley's approval is a second
  check, the one Chad sets for terminals.
- **The code we already ship.**
  - `marley_mcp`: the `terminal` family and the tool table (`registry.rs`), `Tier::Write` and
    `decide` over the `GrantTable` (`permission.rs`), calls the app answers (`dispatch.rs`,
    `AppCall` in `marley_mcp.rs`), `APP_CALL_TIMEOUT_SECONDS` = 30; `marley_fleet::Receipt`
    (`verbs.rs`), accepted or refused with a reason. #516's `Redactor` (`redact.rs`).
  - Zed's `terminal`: `Terminal::paste`, bracketed when the program set mode 2004;
    `try_keystroke`, which encodes a gpui `Keystroke` through `to_esc_str` with the program's
    modes, as the user's own keys go; `pid()` (the PTY's foreground process group, from
    `tcgetpgrp`) against `pid_getter().fallback_pid()` (the shell) says whether a program runs,
    with or without shell integration; `foreground_process_command_name` names it, refreshed on
    output (L-claude-477).
  - `marley_workbench`: the footer hook (AD-claude-477) for the bar and the card; the
    `cx.propagate()` fall-through of `marley::RichInput`, for a key that reaches the program when
    Marley has nothing to do; `agent_bar::agent_in` to recognize agent CLIs; `mcp.rs`'s
    `terminal_with_id` and the grant of `browser.write` at start; `blocks::focused_terminal`.
  - Zed's `workspace::Toast` with `on_click`, for Show.
  - What is missing is a read of the live screen by rows: `Terminal::get_content` joins wrapped
    lines over the whole scrollback, and `absolute_lines_text` stops at the cursor's line, so a
    small hunk beside `absolute_lines_text` reads each screen row.

## UI proof
UI-AFFECTING. `script/e2e/525-agent-drives-a-running-program.sh` (`compositor sway`, for the
clicks on the card and its buttons). Fixtures: the scenario's own bash with a plain prompt, a
scratch SQLite database with a `users` table, a stand-in MCP client run from the harness through
the plugin's bridge as #491's is, and a stand-in `claude` on the PATH for one refusal. Steps and
shots: `ech` and Ctrl-I at the bash prompt, completed by bash (`525-00-ctrl-i-completes`);
`sqlite3 demo.db`, then the client's `terminal_screen`, whose rows the run log prints
(`525-01-screen`); a `terminal_type` of a query, left waiting in the background: the card and the
toast (`525-02-asked`); Allow, the query's result and the bar (`525-03-typed`); a second write,
typed with no card (`525-04-no-second-ask`); Ctrl-I in the terminal, the bar's "You have
control", and a write refused in the log (`525-05-taken-over`); Ctrl-I again, a write with the old
generation refused and one with the new generation typed (`525-06-handed-back`); the setting at
`ask_every_write`, a write, Deny (`525-07-denied`); `never_ask`, a write typed with no card
(`525-08-never`); the Marley page's Agents section with the setting (`525-09-setting`);
`ask_first_write` again, `.quit` and a second `sqlite3`, whose first write asks again
(`525-10-asked-again`) and, left unanswered, expires (`525-11-expired`); `.quit`, a write at the
shell's prompt refused, and one into the stand-in `claude` refused (the log). The settings
change by rewriting the profile copy's `settings.json`, which Marley reloads.

## Locked-In Decisions
- D1 — Only a running program takes an agent's typing. At the shell's prompt a write would run a
  command, which is plan D9's `terminal.run`, a ticket of its own; an agent CLI takes prompts
  through the session verbs later (report 06 item 2). Both are refused.
- D2 — Approval is per program. By default the first write to a foreground process group asks
  (Warp's "Ask on first write"), and the approval ends when that program does. The other two
  modes are Warp's other two.
- D3 — Every write names the generation `terminal_screen` gave, as a harness `input` names its
  claim's generation. A take-over, a hand-back or a new foreground program advances it, so an
  agent reads the screen again before it types after any of them.
- D4 — The user is never locked out. Take Over stops the agent, not the user, and the user's own
  typing does not take over by itself.
- D5 — Ctrl-I, Warp's key, bound in `Terminal`. It acts only while an agent has written to that
  terminal's program; otherwise it reaches the program, where it is Tab's byte. While an agent
  drives vim, Ctrl-I is Marley's.
- D6 — The card sits in the target terminal's footer and a workspace toast with Show points to
  it. Marley switches no tab by itself: a tab brought forward in the focused pane takes the focus
  (L-claude-493), and the user's next keys would land in the program.
- D7 — The approval waits 25 seconds, under the transport's 30; an unanswered or late approval
  types nothing.
- D8 — Text goes as a paste and keys as keystrokes: a multi-line statement reaches a REPL whole
  when it turned bracketed paste on, and control keys go through Zed's encoder with the
  program's modes, as the user's keys do.
- D9 — The screen read gives the live screen, not the scrolled view, and goes through #516's
  redaction.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `terminal_screen` for a terminal, the system shall answer with the live screen's rows as text, the cursor, the size, whether the alternate screen shows, the foreground program and the generation, with #516's redaction applied to the rows. | The run log beside shot `525-01-screen` |
| REQ-002 | WHEN an agent's first write reaches a program in `ask_first_write`, the system shall show the text to type with Allow and Deny in that terminal's footer and a toast with Show in its workspace, and shall type nothing before Allow; a program that starts later asks again. | Shots `525-02-asked`, `525-10-asked-again` |
| REQ-003 | WHEN the user allows a write, the system shall type its text, keys and Enter into the program and answer the call with the bytes written. | Shot `525-03-typed`; the run log |
| REQ-004 | WHILE the program whose first write was allowed still runs in `ask_first_write`, the system shall type later writes without asking. | Shot `525-04-no-second-ask` |
| REQ-005 | WHILE an agent has written to a terminal's current program, the system shall show a bar under that terminal naming the program and the last write, with Take Over. | Shots `525-03-typed`, `525-04-no-second-ask` |
| REQ-006 | WHEN the user presses Ctrl-I in that terminal or clicks Take Over, the system shall refuse the agent's writes until the user hands back, and the bar shall say the user has control. | Shot `525-05-taken-over`; the run log |
| REQ-007 | WHEN the user hands back, the system shall advance the generation, refuse a write that names the old one, and type a write that names the new one. | Shot `525-06-handed-back`; the run log |
| REQ-008 | WHERE `marley.agent_terminal_writes` is `ask_every_write`, the system shall ask on every write, and a denied write shall type nothing. | Shot `525-07-denied`; the run log |
| REQ-009 | WHERE `marley.agent_terminal_writes` is `never_ask`, the system shall type each write without a card. | Shot `525-08-never` |
| REQ-010 | WHEN a write has waited 25 seconds for an answer, the system shall refuse it, remove its card and toast, and type nothing. | Shot `525-11-expired`; the run log |
| REQ-011 | WHEN an agent writes to a terminal whose shell waits at its prompt, or whose foreground program is an agent CLI, the system shall refuse the write and say why. | The run log |
| REQ-012 | WHILE no agent has written to a terminal's current program, Ctrl-I in it shall reach the program as before. | Shot `525-00-ctrl-i-completes` |
| REQ-013 | WHEN the Marley page shows, its Agents section shall offer the agent terminal writes setting with its three values. | Shot `525-09-setting` |
| REQ-014 | The diff gate shall be green. | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec; the design, the harness mapping and the test plan in the notes.
- **P2 Code** — the rows in `docs/marley/zed-touchpoints.md` first; the screen hunk in
  `terminal`; the pure control state in `marley_terminal`; the tools in `marley_mcp` and their
  answers in `marley_workbench`; the bar, the card, the toast and the action; the setting and its
  dropdown; fmt and clippy clean; a review of the diff (security lens: nothing typed without a
  current generation, a running program and an approval).
- **P3 Test** — write and run the scenario and read every shot; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley/three-prong-plan.md` (D9's tools) and
  `docs/marley_architecture/` for the Marley crates (§21), ledger capture (§19), close the
  ticket, archive, commit.
