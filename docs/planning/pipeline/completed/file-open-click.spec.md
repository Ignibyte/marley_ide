---
pipeline_id: d8f17a04-2bbf-418a-8f8a-68a48fdccebc
ticket: forge#59 (f35cfb9c-c034-4510-8178-500f7991e975) · local docs/planning/tickets/open/TICKET-059-file-open-click.md
aar_id: 2bffe609-9c53-416a-afbe-d4b1de755aa1
status: Phase 5 — Complete PASS
title: click a file-tree row to open it
type: feature
milestone: M2.B
references:
  - crates/marley_project/src/lib.rs (PURE: FileTree::path_at)
  - crates/marley_app/src/app.rs (SHIM: file-row click → write path to PTY)
---

## Title
Completes #56: clicking a FILE row in the Left "Files" dock inserts that file's path at the shell
prompt (dirs already toggle collapse) — the same "open" cmd-P does on Enter.

## Scope
### In
- PURE (`marley_project`, cov/MSI 100): `pub fn FileTree::path_at(&self, index: usize) -> Option<PathBuf>`
  — the FULL relative path of the node at visible-row `index` (joining each ancestor's name down from the
  root), walking the SAME pre-order as `visit`/`toggle_at`; `None` out of range. Private `path_at_in`
  recursion tracking a `prefix` path.
- SHIM (`app.rs`, mutants::skip + cov-excluded): the Left-dock row click attaches to ALL rows — a DIR
  row still `toggle`s; a FILE row → `path_at(index)` → write the path bytes to the active session's PTY
  (reuse the cmd-P write path).

### Out
- Opening in an editor pane (first cut inserts the path at the prompt). Double-click semantics. Drag.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `path_at` returns the row's path for BOTH dirs and files; the SHIM decides (dir→toggle,
  file→open) — keeps the pure fn simple + reusable.
- D2 — `path_at_in` mirrors `toggle_at`/`visit` traversal EXACTLY (node, then visible children) so the
  click's row ordinal maps to the right node — same invariant proven for `toggle`.
- D3 — "Open" = write the path to the PTY (prompt insert), consistent with cmd-P's Enter (#57 D3).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `path_at(index)` targets a nested node, it shall return its FULL relative path (all ancestor components joined), not just the leaf name. | unit (`a/b/x` → `path_at(2)=="a/b/x"`) |
| REQ-002 | WHEN `path_at(index)` targets a root-level node, it shall return that name; WHEN `index` is out of range, it shall return `None`. | unit |
| REQ-003 (visual) | WHEN a FILE row in the Files dock is clicked, its path shall appear at the shell prompt. | self-test (click a file row → capture) |
| REQ-004 | `scripts/gates.sh` GREEN, cov/MSI 100 on `path_at`; app shim excluded. | gate |

## Phase Plan
- **P2** — the `path_at`/`path_at_in` shapes, the click-shim change, mutation targets, unit + self-test plan.
- **P3** — `path_at` in marley_project + the app.rs file-row click.
- **P3.5** — critic: the ancestor-join, index↔traversal parity, collapsed-subtree, out-of-range, mutants.
- **P4** — path_at unit tests (cov/MSI 100) + `-p marley` green + the SELF-TEST (click a file → path at
  prompt) + gate GREEN.
- **P5** — docs, AAR, archive, close #59.
