---
pipeline_id: db9966fd-90b0-4a64-9bd0-6e738a8b6c5c
ticket: forge#46 (836fc74c-40e2-40f6-b85b-2a3620fdba53) · local docs/planning/tickets/open/TICKET-046-rerun-block.md
aar_id: e7f1b647-55f0-41af-80d6-b7c204306916
status: Phase 5 — Complete PASS
title: re-run a Block's command (click / cmd-R)
type: feature
milestone: M1.G
references:
  - crates/terminal_blocks/src/block.rs (NEW Block::rerun_command)
  - crates/marley_app/src/app.rs (the header re-run affordance — shim)
  - docs/specs/SPEC-terminal-blocks.spec.md (R30)
  - docs/specs/SPEC-app-shell.spec.md (R50)
---

## Title
Another Warp block workflow: re-run a previous command from its Block — a ↻ action (or cmd-R for the
last) — without retyping. It resends the command to the shell.

## Scope
### In
- `crates/terminal_blocks/src/block.rs` (PURE — cov/MSI 100, beside `copy_text`):
  - `impl Block { pub fn rerun_command(&self) -> Option<String> }` — `Some(self.command.clone())` IFF
    `self.state == BlockState::Finished` AND `!self.command.is_empty()`; else `None` (a Running/Pending
    block, or an empty command, is not re-runnable). The pure "is this block re-runnable, and with what".
- `crates/marley_app/src/app.rs` (SHIM): the block header (the #45 hover affordance row) gains a ↻
  re-run glyph; on click it looks the block up by `(pane, index)` at click-time (the #45 pattern) and,
  GUARDED by the #40 session state, resends the command:
  `if !session.is_command_running() { if let Some(cmd) = block.rerun_command() { let _ =
  session.write_command(&cmd) } }` (`write_command` writes `cmd\r\n` → the shell runs it as a new
  block). cmd-R re-runs the most recent finished block.
- SPEC-terminal-blocks R30 (rerun_command + Mutation-Targets); SPEC-app-shell R50 (the re-run shim).
  CHANGELOG + arch doc.

### Out (explicitly deferred)
- EDIT-then-run (re-run with a tweak) — a later cut; this resends verbatim. Re-running into a specific
  pane other than the focused one. A confirmation for a destructive re-run (`rm …`) — later. Re-run
  history/repeat-last shortcuts beyond cmd-R.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `rerun_command` returns `None` for a NON-`Finished` block (Running/Pending) — you don't re-run a
  command that's still going — AND for an empty command. Both are the tested guards.
- D2 — The session-idle guard (`!is_command_running()`, #40) is a SHIM concern (the live session
  state, not the block value) — so `rerun_command` stays pure on the block; the shim adds the guard so
  a re-run never injects mid-command.
- D3 — PURE: `rerun_command` (cov/MSI 100). The ↻ affordance + the click + `write_command` +
  the session guard are SHIM (app.rs, masked). `write_command` (writes `cmd\r\n`) is reused from
  on_submit.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `block.rerun_command()` is called on a `Finished` block with a non-empty command, it shall return `Some(the command)`. | unit (Finished + "ls" → Some("ls")) |
| REQ-002 | WHEN the block is not `Finished` (Running/Pending), `rerun_command` shall return `None`. | unit (Running → None) |
| REQ-003 | WHEN the block's command is empty, `rerun_command` shall return `None` even if `Finished`. | unit (Finished + "" → None) |
| REQ-004 | WHEN the re-run affordance is clicked (or cmd-R) and the session is idle, the app shall resend the block's command to the focused shell; WHEN a command is already running, it shall not inject. | shim + masked visual — chad-verified |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `rerun_command`. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the `rerun_command` shape (Finished + non-empty), the header ↻ shim + the #40 guard,
  the SPEC clauses + mutation targets.
- **P3 Implement** — the method + the app.rs re-run affordance + spec + CHANGELOG.
- **P3.5 Inspect** — critics: the Finished + non-empty guards (a Running or empty block → None), the
  shim's #40 session-idle guard (no mid-command injection), the deferred-lookup click, no regression.
- **P4 Validate** — the rerun_command unit tests + gate GREEN + the masked visual.
- **P5 Complete** — docs, AAR, archive, close #46.
