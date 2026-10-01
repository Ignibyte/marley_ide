---
pipeline_id: 650c42a0-8e8d-4571-9d22-ef09ee59aac8
ticket: docs/planning/tickets/open/TICKET-623-a-failed-blocks-errors-as-diagnostics.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "A failed block's errors as project diagnostics"
type: feature
slice: prong 1 T4
references: [docs/marley/three-prong-plan.md, docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md]
---

## Title
A failed block's located errors become project diagnostics under a Marley source, cleared when
the same command next succeeds (plan T4, D6). After #620 (the locator) and #621 (task blocks).

## Scope
### In
- `crates/marley_terminal`: #620's locator also returns every failure in a block, in order, with
  its message and severity (error or warning).
- `crates/marley_workbench`: when a local block finishes non-zero, its failures inside the
  project's worktrees are published as diagnostics through `LspStore::merge_lsp_diagnostics`
  under a reserved language server id named for Marley's blocks, with `DiagnosticSourceKind::Other`;
  when a block of the same command in the same folder next finishes with exit 0, they are cleared.
- Task blocks (#621) and shell blocks alike.

### Out (explicitly deferred)
- Files outside the worktrees (the store drops them).
- Remote projects.
- Merging with a language server's own diagnostics for the same line.

## Reference (§20)
- **Upstream Zed:** `project::lsp_store::LspStore::merge_lsp_diagnostics` and the Diagnostics view
  (`crates/diagnostics`); diagnostics are keyed by a language server's id.
- **Warp (behavior):** N/A, Warp has no editor
  (`docs/warp_architecture/subsystems/03-terminal-session-core.md`, the fusion paragraph); the
  behavior is the fusion note's (`docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md` §8).

### Prior art
- **Behavior maps:** plan D6 ("Failed Blocks feed the diagnostics panel"); the fusion note.
- **Published material:** LSP's `PublishDiagnosticsParams`, which `merge_lsp_diagnostics` takes.
- **Code we already ship:** `merge_lsp_diagnostics` (public, local projects); `merge_diagnostic_entries`
  (public, but its document type has private fields); `update_diagnostics` (test support only).
  No Marley crate publishes diagnostics yet. Whether a language server id with no running server
  shows its name in the Diagnostics view is to be checked at promotion.

## UI proof
`script/e2e/623-a-failed-blocks-errors-as-diagnostics.sh` (sway): `repo/src/main.rs`; a task that
prints two rustc-shaped errors and a warning in `src/main.rs`, exiting 101, then a second task
with the same command made to pass by a flag file.
- `status.png`: the status bar's diagnostics count, 2 errors and 1 warning;
- `diagnostics.png`: the Diagnostics view listing them under `src/main.rs`;
- `cleared.png`: after the passing run, the count back to none.

## Locked-In Decisions
- D1 — One reserved source for every block's diagnostics; a command's are replaced by its next
  run's, so they never pile up.
- D2 — "The same command" is its text and its folder.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a local block fails with located errors in the project, Marley shall list them in the project's diagnostics. | Shots `status.png`, `diagnostics.png` |
| REQ-002 | WHEN a block of the same command in the same folder next succeeds, Marley shall clear them. | Shot `cleared.png` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams (the reserved id, the view's name for it), the design.
- **P2 Code** — the locator's full list, the publishing and the clearing; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
