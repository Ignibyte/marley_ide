---
pipeline_id: b7d5cc69-db35-48e1-bfdd-759e0563d689
ticket: forge#40 (659e95ef-bd2d-4d6f-a464-bb2c3193d1cc) · local docs/planning/tickets/open/TICKET-040-running-input-route.md
aar_id: 758a132f-9c4a-4be1-8212-3cc820547bcf
status: Phase 5 — Complete PASS
title: input routing by running-command state
type: bug
milestone: M1.F
references:
  - crates/terminal_blocks/src/keys.rs (input_route — gains command_running)
  - crates/terminal_blocks/src/session.rs (is_command_running — new delegator)
  - crates/terminal_blocks/src/block.rs (BlockList::current — the Running signal, already pub)
  - crates/marley_app/src/app.rs (~573 the routing — shim)
  - docs/specs/SPEC-terminal-blocks.spec.md (R26) + SPEC-app-shell.spec.md
---

## Title
BUG (chad hit it live): interactive prompts on the PRIMARY screen — Claude Code's arrow-key
approval/selection menus, `read`, `npm init`, interactive git — don't receive arrow keys; Marley eats
them for its own local history recall. `input_route(alt_screen, ctrl)` streams to the PTY (`Raw`)
ONLY on the ALTERNATE screen (vim/top, #33) or a ctrl key; a primary-screen interactive program is
NOT in alt-screen, so routing falls to `Cooked` → `history.recall_prev` (app.rs:613). Route by
whether a foreground COMMAND IS RUNNING instead.

## Scope
### In
- `crates/terminal_blocks/src/keys.rs` (PURE, gpui-free — cov/MSI 100): `input_route` gains a third
  arg — `input_route(alt_screen: bool, ctrl: bool, command_running: bool) -> Route` = `Raw` if
  `alt_screen || command_running || ctrl`, else `Cooked`. The `input_route_cases` test extends to the
  8-case truth table.
- `crates/terminal_blocks/src/session.rs` (PURE, gpui-free — cov/MSI 100):
  `TerminalSession::is_command_running(&self) -> bool` = `self.blocks().current().is_some()`
  (`BlockList::current()` already returns the last block IFF `Running`, block.rs:132) — a session
  delegator mirroring `is_alt_screen`, tested via mock+DCS (preexec opens a Running block → true;
  precmd finishes it → false; fresh → false).
- `crates/marley_app/src/app.rs` (SHIM, ~573): read `command_running` from the focused
  `state.session.is_command_running()` and pass it into `input_route`; the existing `Raw` branch
  (`encode_key` → `write_bytes`) already streams every key to the PTY.
- SPEC-terminal-blocks R26 (+ `command_running` + `is_command_running`) + the test-map + mutation-
  targets lines; SPEC-app-shell R40 (the routing note). CHANGELOG + arch docs.

### Out (explicitly deferred)
- Tab-completion / the bare-prompt input model (the deferred #33 `prompt-shell-line-editing-model`
  fork — Warp-local vs shell-ZLE). #40 KEEPS local editing at the bare prompt (#28/#29); it only
  changes routing WHILE A COMMAND RUNS. The extra interactive keys (Shift-Tab/F-keys, #41) + paste
  (#42) are the sibling M1.F tickets. Mouse reporting is later.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — The routing signal is "a foreground command is running" = a `Running` block exists =
  `blocks().current().is_some()` (the shell-integration Preexec opens it, the next Precmd finishes
  it). Between them, the child owns the terminal → stream all input to it.
- D2 — `input_route`'s three conditions are an OR: alt-screen (full-screen TUI) OR command-running
  (inline interactive program) OR ctrl (signals at the prompt). Any → `Raw`.
- D3 — At the BARE PROMPT (no running command, not alt-screen, no ctrl) → `Cooked` → the local line
  editor + history (#28/#29) — UNCHANGED. #40 is orthogonal to the tab-completion fork.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `input_route(alt_screen, ctrl, command_running)` is called, it shall return `Raw` if ANY of the three is true, and `Cooked` only when all three are false. | unit test (the 8-case truth table) |
| REQ-002 | WHEN a foreground command is running (a `Running` block exists), `TerminalSession::is_command_running()` shall return `true`; at the bare prompt (the last block finished, or no blocks) it shall return `false`. | unit test (mock+DCS: preexec→true; precmd→false; fresh→false) |
| REQ-003 | WHILE a command is running, the app shall route every keystroke to the PTY (so an interactive program receives arrows/keys); at the bare prompt it shall feed the local editor. | shim + masked visual (chad drives Claude Code / `read` → arrows navigate) |
| REQ-004 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `input_route` + `is_command_running`. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the exact `input_route` signature + the 8-case test, `is_command_running`'s
  delegate + its mock+DCS test, the app.rs routing change, the SPEC edits + mutation targets.
- **P3 Implement** — keys.rs + session.rs + app.rs + specs + CHANGELOG.
- **P3.5 Inspect** — critics: the 3-way OR killable (each operand), is_command_running true/false
  lifecycle, the shim reads the focused state, no regression to alt-screen/ctrl (#33) or history (#29).
- **P4 Validate** — the input_route 8-case + is_command_running tests + gate GREEN.
- **P5 Complete** — docs, AAR, archive, close #40.
