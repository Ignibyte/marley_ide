---
pipeline_id: c012d0aa-dcde-4d76-a21c-290db9c63967
ticket: forge#58 (cbbd4790-ca3e-453f-8ea3-adf4e80afce8) · local docs/planning/tickets/open/TICKET-058-block-details.md
aar_id: 5c65c71b-8e99-43fb-82f4-2e5cfa4f9abd
status: Phase 5 — Complete PASS
title: Details dock — focused-block inspector
type: feature
milestone: M2.A
references:
  - crates/marley_app/src/block_status.rs (PURE: BlockDetails + block_details)
  - crates/marley_app/src/app.rs (SHIM: Right dock renders the last block's details)
---

## Title
The Details dock (Right, empty since M1.B) inspects the current command BLOCK — its command, status,
exit code, working directory, and git branch. The Right-dock counterpart to #56's file tree; the final
M2.A ticket.

## Scope
### In
- PURE (`block_status.rs`, cov/MSI 100 — a marley_app module):
  `#[derive(Debug, Clone, PartialEq, Eq)] pub struct BlockDetails { pub command: String, pub status:
  StatusKind, pub exit_code: Option<i32>, pub pwd: Option<String>, pub git_branch: Option<String> }`
  (NO `Default`) + `pub fn block_details(block: &Block) -> BlockDetails` projecting the five fields
  (status via the existing `exit_status_kind`).
- SHIM (`app.rs`, mutants::skip + cov-excluded): the Right "Details" dock renders `block_details` of
  the active session's MOST-RECENT block — the `status_indicator` glyph + the command, an `exit N`/
  `running` line, the pwd, the git branch; a placeholder when the session has no blocks. Passed as the
  Right dock's `dock_panel` content (was the empty div).

### Out
- Focus-on-click (click a block → inspect THAT one) — the first cut shows the most-recent block. Live
  duration/timing. Rich metadata beyond these five fields. Editing from the dock.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `block_details` in `block_status.rs` (reuses `StatusKind` + `exit_status_kind` #36) — the
  status classification stays single-sourced.
- D2 — `BlockDetails` holds `status: StatusKind` (not a pre-formatted string) — the shim maps it to the
  glyph via `status_indicator`; the model stays render-free + the label lives in one place.
- D3 — NO `Default` on `BlockDetails` (keeps the whole-body `Default::default()` mutant unviable, per
  #54/#55). block_status.rs already has viable mutants (exit_status_kind), so it's not the 0-viable trap.
- D4 — The dock shows the MOST-RECENT block of the active session (the command you just ran); when the
  session has no blocks, a placeholder. Focus-tracking is deferred.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `block_details(block)` runs on a FINISHED exit-0 block, it shall project command, `status=Success`, `exit_code=Some(0)`, pwd, git_branch from the block. | unit |
| REQ-002 | WHEN the block is FINISHED with a non-zero code, `status` shall be `Failure` and `exit_code` that code. | unit |
| REQ-003 | WHEN the block is RUNNING, `status` shall be `Running` (regardless of any code). | unit |
| REQ-004 | WHEN the block's `pwd`/`git_branch` are `None`, `block_details` shall carry `None` for them (no fabrication). | unit |
| REQ-005 (visual) | WHEN a command has run, the Right "Details" dock shall show that command, its status glyph, and its cwd. | self-test (drive a command → capture) |
| REQ-006 | `scripts/gates.sh` GREEN, cov/MSI 100 on block_details; app shim excluded. | gate |

## Phase Plan
- **P2** — `BlockDetails`/`block_details` shapes, the Right-dock render + last-block access + placeholder,
  mutation targets, unit + self-test plan.
- **P3** — block_details in block_status.rs + the app.rs Right-dock shim.
- **P3.5** — critic: each field projection, the status reuse, the no-pwd/branch case, the last-block
  access + placeholder, the hollow-MSI note, mutants.
- **P4** — block_details unit tests (cov/MSI 100) + `-p marley` green + the SELF-TEST capture (Details
  shows the command + ✓ + cwd) + gate GREEN.
- **P5** — docs, AAR, archive, close #58 → **M2.A COMPLETE**.
