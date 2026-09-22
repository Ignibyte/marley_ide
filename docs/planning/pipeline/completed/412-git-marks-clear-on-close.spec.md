---
pipeline_id: ad95c323-1049-4846-bd32-f05c27cc3545
ticket: docs/planning/tickets/open/TICKET-412-git-marks-not-cleared-on-close.md
status: Phase 5 — Complete PASS
title: git_marks + git_marks_key cleared on last-instance close (the #353 sibling)
type: bug
milestone: M19
references:
  - docs/planning/pipeline/completed/353-editor-folds-clear-on-close.spec.md
  - docs/planning/pipeline/completed/353-editor-folds-clear-on-close.notes.md
  - docs/planning/pipeline/completed/328-git-gutter.notes.md
---

## Title
Scrub `git_marks[path]` — and a `git_marks_key` naming that path — when the last
same-path editor instance drops. The exact leak class #353 fixed for
`editor_folds`, one field down (found by the #353 inspect correctness critic:
F-claude-git-marks-close-leak-sibling-of-editor-folds-001). `git_marks`
(app.rs:323) is populated per viewed file whenever the active file carries diff
marks and removed only when that same file is ACTIVE again with an empty diff —
no close path scrubs it, so a file closed while carrying marks leaks its entry
for the app's life.

**Sharpened at plan discovery:** the key half is load-bearing, not hygiene. No
production path bumps `git_marks_gen` on activation (the three bumps are save
:9261, external reload :8987, in-app commit :5053); the reopen "self-heal" works
only because the cache key's PATH half moves on a file switch. So a naive
map-only scrub would MINT a wrong-render the leak never had: close file A while
it is the active editor (key stays `(A, g)`), reopen A with no save/commit/reload
in between → `refresh_git_marks` early-returns on the matching key (app.rs:4097)
against a now-empty map → a dirty file renders an unmarked gutter. The key must
clear with the entry.

## Scope
### In
- Extend the existing #353 census-guarded scrub arm
  (`RootView::release_editor_views`, app.rs:5907) to also remove the dropped
  path's `git_marks` entry and clear `git_marks_key` when it names that path.
  No new choke, no call-site changes — all production close paths already route
  through the wrapper since #353.
- Headless drives proving: scrub-on-last-close, twin-view retention, alias-root
  survivor retention (census), key-never-dangles + the close→reopen
  recompute-not-early-return behavior. The scrub lines must clear the 100%-MSI
  bar via the #328 fixtures (`push_git_diff_for_test`, `git_marks_for_test`,
  the real-git seeded-repo drive pattern).
- The sibling per-path-map re-sweep
  (PR-claude-lifecycle-scrub-sweeps-sibling-per-path-maps-001) recorded at
  inspect on the final diff.

### Out (explicitly deferred)
- Any change to `refresh_git_marks` compute/caching semantics (route A: marks
  are the SAVED working-tree-vs-HEAD state; keystroke-inert — #328 locked).
- The `git_marks_gen` bump set (save / external reload / in-app commit) stays
  as-is; no activation bump is added.
- File-rename key migration — no rename surface exists (#353 stance; LSP
  resource ops rejected at #322 D2).
- `editor_folds` behavior — shipped at #353; this ticket only rides beside it.

## Reference (§20)
**Zed (the editor reference), behavior level — the same stance #353 recorded.**
Per `docs/zed_architecture/` deconstruction 06 §2 (cited at #328): git state is
fully DECOUPLED from the file/worktree entry and buffer-scoped — Marley adopted
that as the standalone per-path `git_marks` store. Lifecycle: Zed's per-buffer
diff state dies with the buffer; nothing persists gutter marks across a close.
Marley matches that BEHAVIOR: the marks entry dies with the last same-path
editor instance; reopening recomputes fresh from the working tree (`git diff
HEAD`, git's own output — the #328 "Marley-specific composition over git's OWN
output" stance carries over). Clean-room: behavior maps + our own substrate
only; no copyleft source read.

### Prior art
1. **Behavior maps** — `docs/zed_architecture/` deconstruction 06 §2 (git status
   decoupled from worktree entries; adopted at #328 as the standalone store);
   #353's Zed FoldMap citation for the lifecycle stance (per-view/per-buffer
   state dies with its owner, no cross-close persistence). Warp maps: no editor
   git-gutter analog (terminal/cockpit product).
2. **Published** — git's own `git diff HEAD -- <pathspec>` / `status --porcelain`
   semantics (already the #328 compute route): a recompute-on-demand model with
   no client-side lifecycle to adopt. Nothing further.
3. **Deps / in-repo substrate (the paying leg)** — no external crate owns this
   seam (argv git by #328 decision — git2 is not a dep; gpui renders, holds no
   app state). The substrate we already ship owns ALL of it:
   `RootView::release_editor_views` (app.rs:5907, #353) is the census-guarded
   scrub arm; `content::any_open_editor_with_path` (content.rs:287) is the
   census; the `#[must_use]` free fn returns the dropped paths; #328's test
   fixtures (`push_git_diff_for_test` app.rs:4203, `git_marks_for_test` :4208,
   `refresh_git_marks_for_test` :4219, the seeded-real-git headless drive)
   are the mutation-bar fixtures this ticket was deferred to get — they exist.
   **ADOPT the #353 arm + the #328 fixtures; invent nothing.**

## React-first (parity)
N/A — no UI delta: close-path state hygiene on maps the React POC does not
model. The only user-observable consequences are the bug's own edges
disappearing: a reopened-while-closed-modified file no longer renders leaked
stale marks, and the key-dangle wrong-render (unmarked gutter on a dirty file)
is prevented rather than introduced. No chrome, layout, type, color, or
affordance changes.

## Locked-In Decisions
- **D1 — SCRUB POINT = the existing #353 arm, unchanged shape.** The
  `git_marks.remove(&path)` rides inside the same
  `!any_open_editor_with_path(...)` census arm that scrubs `editor_folds`
  (app.rs:5909–5911). No new choke, no call-site migration, no new census code.
- **D2 — CENSUS SEMANTICS IDENTICAL to #353** (ticket acceptance pins this):
  a surviving distinct same-path instance (alias-root workspaces, #319 D3)
  retains the shared `git_marks` entry; exact `==` on stored spellings via the
  existing helper; census after the whole release batch.
- **D3 — KEY CLEARS WITH THE ENTRY, inside the census arm.** When the scrub
  removes path P and `git_marks_key` names P, set it `None` — the discovery
  finding above makes this load-bearing (prevents the early-return wrong-render
  on close→reopen with no intervening gen bump). While a same-path instance
  survives, entry and key both stay — a consistent live pair.
- **D4 (→ design) — key-state verification shape:** a `#[cfg(test)]`
  `git_marks_key_for_test` accessor vs asserting behavior-only (reopen
  recomputes marks). Either way the accessor lands at VALIDATE atomically with
  its drives (#353's dead-code-deferral lesson: an accessor added at implement
  is `-D warnings` red until the drives read it).
- **D5 (→ design) — drive seeding mix:** `push_git_diff_for_test` seeding for
  the per-close-path scrub/twin/alias rows (no git spawn) vs the #328
  seeded-real-git repo for the end-to-end close→reopen recompute row. Design
  settles the exact row set.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the last view of an open file's editor instance is released (tab close, file-strip close, pane close, or project close), the system shall remove that file's `git_marks` entry. | Headless drives per close path: seed marks → close → `git_marks_for_test(path)` empty; kills the remove-deletion mutant (gate:5). |
| REQ-002 | WHILE another view of the same editor instance remains open (a twin), WHEN one view is released, the system shall retain the file's `git_marks` entry unchanged. | Twin-view drive: entry present, mark rows unchanged. |
| REQ-003 | WHILE a distinct same-path editor instance survives (alias-root workspaces, #319 D3), WHEN another instance of that path fully drops, the system shall retain the shared `git_marks` entry. | Alias-root two-project drive (the #353 REQ-004 pattern, /var↔/private/var); kills the guard→always-clear mutant. |
| REQ-004 | WHEN the scrub removes a path's `git_marks` entry, the system shall clear a `git_marks_key` that names that path, such that reopening the file recomputes marks from the live working tree instead of early-returning on a stale key. | Key-state assert per D4 + end-to-end drive: dirty file → open (marks) → close last view → reopen with no gen bump → marks present (recomputed); kills the key-clear-deletion mutant. |

## Phase Plan
- **P2 Design** — settle D4 (accessor vs behavior-only) + D5 (drive row set);
  file manifest (app.rs scrub arm + decl comments; headless_drive.rs drives);
  regression test plan mapping every REQ + mutation notes; confirm the §20
  match line; re-run the sibling sweep grep on the current tree.
- **P3 Implement** — the two scrub lines + decl-comment updates (`git_marks`
  :320 and `git_marks_key` :325 gain their #412 lifecycle sentences; the
  wrapper doc :5898 notes the git-marks scrub).
- **P3.5 Inspect** — independent critics vs the diff (key/entry consistency
  edges, census parity with #353, leak-fixed-not-moved, provenance §20, the
  sibling re-sweep).
- **P4 Validate** — write + RUN the planned drives; `scripts/gates.sh --diff`
  green.
- **P5 Complete** — CHANGELOG + arch docs (§21), ledger appends (§19: close the
  open F-claude-git-marks-close-leak… status, lessons), archive the pair,
  close the ticket.
