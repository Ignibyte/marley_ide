---
pipeline_id: 0f59403a-b9cc-4086-ace8-e2fb65c5a244
ticket: forge#328 (37165c5a-02bb-43f6-856e-530301b163a8) · local docs/planning/tickets/open/TICKET-328-git-gutter.md
aar_id: 34325337-3b6a-4eb3-a1f6-d1cf92625517
status: Phase 5 — Complete PASS
title: Git gutter — added/modified/deleted markers against HEAD in the editor gutter
type: feature
milestone: M21
references: [forge#328, forge#102, forge#310]
---

## Title
The editor learns what changed: a colored bar in the gutter per edited line — added (success), modified
(warning-tone), a deleted-run caret (danger) — against HEAD. Today git surfaces only as the ⌘⇧D diff overlay
+ the changes/commit panel; the editor gutter knows NOTHING per-line (verified). This wires git's OWN diff
output into the gutter, per row.

## Scope
### In
- **The hunk→line-set projection (PURE, the heart)** — `gutter_marks_from_hunks(&FileDiff) -> Vec<(row,
  GitMark)>`: reuse the shipped `git_diff::parse_diff` (`FileDiff{path, hunks: Vec<Hunk{header, lines:
  Vec<DiffLine{kind, text}>}>}`), parse each hunk header's NEW-side start line (`@@ -a,b +c,d @@` → `c`), then
  walk the body's `Added`/`Removed`/`Context` lines against a running new-side row counter: a pure `+` run →
  **Added**, a `-`-then-`+` replaced run → **Modified** (the new rows), a pure `-` run → **Deleted** (a
  ZERO-HEIGHT marker at the boundary BETWEEN rows — the LSP zero-width convention adapted). Pure over the
  existing parser, ZERO new deps. Tolerates malformed/empty diff (no marks, never a panic — the parser's
  posture).
- **`GitMark`** — `enum { Added, Modified, Deleted }` (gpui-free; the render maps to `success`/a warning tone/
  `danger`).
- **HEAD source (argv, the shipped posture)** — per changed file, `git diff -- <relpath>` via
  `marley_command::blocking` (the `git_working_diff_in` precedent, app.rs:2812 — argv-quoted, read-only) →
  `parse_diff` → project. An UNTRACKED file (classified by `parse_status`) → EVERY line **Added** (no HEAD
  blob to diff). The marks reflect the SAVED working-tree-vs-HEAD state.
- **The per-path mark store (decoupled — Zed's warning adopted)** — a `HashMap<PathBuf, Vec<(row, GitMark)>>`
  (or equivalent) in its OWN store, NEVER a field on `OpenFile`/worktree entries. Cached per `(path,
  buffer-version, head-oid)` so an idle frame recomputes nothing.
- **The second gutter lane (render)** — insert a fixed ~3px color-bar child BEFORE the number cell in the row
  assembly (app.rs:3676; `gutter_width` unchanged, code_view.rs:362). The git bar and the #310 diagnostic
  NUMBER tint COEXIST — two lanes, no collision (the #310 one-lane rule is about diagnostic PRODUCERS, not
  git). A `Deleted` marker renders as a thin danger caret at the row's top boundary.
- **Refresh cadence** — recompute a file's marks on SAVE (the didSave moment, the file hits disk → `git diff`
  updates), on external-change reload (extchange), and on OPEN. While a buffer is DIRTY the marks stay at the
  last SAVED state (git sees disk, not the live buffer — the honest route-A stance; a live in-process diff is
  the route-B follow-up).

### Out (explicitly deferred)
- **Route B — `imara-diff` live-buffer↔HEAD in-process** (keystroke-fresh marks on a dirty buffer). The named
  upgrade if route A's save-cadence feels laggy; permissive (Apache-2.0) reuse when it comes.
- **Hunk revert / stage-from-gutter**, **inline blame**, the **deleted-hunk BLOCK render** (Zed's BlockMap
  widget — multibuffer-era), an **index-vs-HEAD toggle**.

## Reference (§20)
**N/A — Marley-specific composition over git's OWN output.** Route A projects the unified `git diff` text
(git's own tool, which Marley already shells out for via `git_working_diff_in`) into gutter rows — no
reference-app source involved. The BEHAVIOR (a git gutter: added/modified/deleted bars per line) is a
universal editor affordance; Zed's `git.rs` + its "git status is fully DECOUPLED — no `git_status` field on
the worktree entry" note (deconstruction 06 §2) are a BEHAVIOR reference only (adopted: the marks live in
their own per-path store), source unread. `imara-diff` (route B, deferred) is permissive REUSE. Clean-room:
no copyleft source read/translated.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D-ROUTE → A (parse `git diff`)** — reuse the shipped `parse_diff` + a header-parse + a body-walk; ZERO new
  deps; the argv-only "no git2/gix" posture (Zed shells out too). Route B (imara-diff) is the deferred
  keystroke-fresh upgrade. (Feasibility confirmed at plan: `Hunk.header` carries the `@@ +c` new-side start;
  `DiffLine.kind` is Added/Removed/Context.)
- **D-DIRTY → saved-state marks** — route A's `git diff` reflects the file ON DISK, so a dirty buffer shows
  the last SAVED state's marks; a save/reload refreshes. Honest (design decides whether to surface a
  staleness hint); the live-buffer diff is route B.
- **D1 — `GitMark{Added,Modified,Deleted}`** pure; `Deleted` = a between-rows boundary marker.
- **D2 — a DECOUPLED per-path mark store** (never on `OpenFile`), cached per `(path, version, head-oid)`.
- **D3 — a SECOND gutter lane** (a 3px bar before the number), coexisting with the diagnostic tint.
- **D4 — argv `git diff -- <relpath>`** via `marley_command::blocking` (read-only, the shipped precedent);
  untracked → all-added.
- **D5 — v1 cuts**: no revert/stage/blame/block-render/index-toggle; route B deferred.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `gutter_marks_from_hunks(&FileDiff)` shall project the hunks to `(row, GitMark)` — a pure `+` run → Added, a `-`-then-`+` run → Modified on the new rows, a pure `-` run → a Deleted boundary marker — parsing the new-side start from each `@@` header. | pure unit cov/MSI 100 |
| REQ-002 | An UNTRACKED file shall project to every line Added. | pure unit + review |
| REQ-003 | The editor gutter shall render a git bar (Added=success, Modified=warning-tone, Deleted=danger) per marked row, COEXISTING with the diagnostic number tint. | headless/render review + LIVE(fallback) |
| REQ-004 | The git marks shall live in a decoupled per-path store, recomputed on save + external reload + open, cached per `(path, buffer-version, head-oid)` (an idle frame recomputes nothing). | unit (cache key) + review |
| REQ-005 | WHILE a buffer is dirty, the marks shall reflect the last SAVED working-tree-vs-HEAD state; a save shall refresh them. | review + integration (git tempdir) |
| REQ-006 | WHEN a file's changes are reverted (its `git diff` becomes empty), its marks shall clear. | integration (git tempdir) + unit |
| REQ-007 | The git marks store shall NOT be a field on `OpenFile`/worktree entries (decoupled). | review |
| REQ-008 | The projection shall tolerate malformed/empty diff output (no marks, no panic). | pure unit |

## Phase Plan
- **P2 Design** — CONFIRM D-ROUTE/D-DIRTY. The pure layer (`gutter_marks_from_hunks`, the `@@`-header
  new-start parse, the run classifier, `GitMark`, the cache key) + the per-path store + the argv refresh shim
  + the gutter render lane. The mutation surface. §20.
- **P3 Implement** — to the manifest; every new pure fn gets a direct unit.
- **P3.5 Inspect** — critics vs the diff; the run classifier (modified vs added vs deleted), the header parse,
  the cache invalidation, the argv quoting, decoupling get the hardest look.
- **P4 Validate** — tests + gate green; a git-tempdir integration test (init + commit + edit → marks) + the
  LIVE drive (add/change/delete a line in a tracked probe file → three distinct colored bars at the right
  rows, pixel-sampled), falling back to units+mechanism if locked.
- **P5 Complete** — CHANGELOG + editor.md + crate-map.md; AAR; archive; close #328.
