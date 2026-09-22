# Git gutter — Notes

- **Forge ticket:** #328 37165c5a-02bb-43f6-856e-530301b163a8
- **AAR:** 34325337-3b6a-4eb3-a1f6-d1cf92625517
- **Local ticket doc:** docs/planning/tickets/open/TICKET-328-git-gutter.md
- **Pipeline spec:** 328-git-gutter.spec.md

## Phase 1 — Plan
- **Request:** per-line added/modified/deleted git markers in the editor gutter, against HEAD. A pure
  hunk→line-set projection over the shipped `git_diff::parse_diff`, a decoupled per-path mark store, a second
  gutter lane, refreshed on save/reload.
- **Classification / tier:** FEATURE, single shippable slice. Unlike #325/#326/#327 there is NO finder picker
  — it's a PURE projection + a per-path store + a gutter render lane + refresh wiring. Builds on #102's
  `git_diff` parser + the argv git precedent. No split.
- **Systems involved:** server/ui (the gutter render + the refresh wiring + a per-path store), a NEW pure
  projection in the git_diff area. No fleet/agents/bridge/settings/lsp.
- **Forge recall (§18.3):** no bulletins. AAR 34325337 opened (0 pre-flagged). Applicable families (by code):
  the render-shim `mutants::skip` (#326/#327), the cache-key/idle-recompute posture (the #274 syntax memo +
  #327 fingerprint gate), the argv-quoting read-only git posture (`git_working_diff_in`).
- **Discovery (seams verified):**
  - `git_diff.rs`: `Hunk{header: String, lines: Vec<DiffLine{kind: DiffKind[Context/Added/Removed], text}>}`
    (:28), `FileDiff{path, hunks}` (:37), `parse_diff` (:57), `ChangeStatus` (:168, HAS `Untracked`),
    `parse_status` (:195). **Route A is feasible:** the `@@ -a,b +c,d @@` NEW-side start `c` is IN
    `Hunk.header` (parse it), and `DiffLine.kind` classifies each body line — so the projection is a
    header-parse + a body-walk over the shipped types. ZERO new deps.
  - The argv precedent: `git_working_diff_in` (app.rs:2812) — `marley_command::blocking::Command::new("git")`,
    argv-quoted, read-only. The per-file `git diff -- <relpath>` reuses this.
  - The gutter: `gutter_width` (code_view.rs:362) + the row assembly (app.rs:3676) — insert a 3px bar child
    before the number cell; `gutter_width` unchanged (the bar is a fixed lane, not part of the number width).
- **Decisions:** D-ROUTE = A (parse git diff, zero deps — the ticket's lean, feasibility confirmed); D-DIRTY =
  saved-state marks (route A's `git diff` sees disk, so a dirty buffer shows the last SAVED state, refreshed
  on save/reload — the honest v1 stance; route B [imara-diff live] is the deferred keystroke-fresh upgrade).
  D1–D5 locked (see spec).
- **§20:** N/A — Marley composition over git's OWN diff output (route A). The git-gutter BEHAVIOR + Zed's
  decoupled-store note are behavior-reference-only (source unread); `imara-diff` (route B) permissive reuse.

## Phase 2 — Design

### D-ROUTE = A + D-DIRTY = saved-state (CONFIRMED)
Route A (project `git diff HEAD` output) reuses the shipped `parse_diff` + the argv `git_working_diff_in`
posture (app.rs:2812, `marley_command::blocking`, read-only) — ZERO new deps. `Hunk.header` carries the `@@
+c` new-side start; `DiffLine.kind` is `DiffKind{Context, Add, Remove}` — so the projection is a header-parse
+ a body-walk over shipped types. `git diff HEAD` sees the file ON DISK, so a DIRTY buffer's marks are the
last SAVED state (D-DIRTY): honest, refreshed on save/reload. Route B (`imara-diff` live-buffer↔HEAD,
keystroke-fresh) is the deferred upgrade.

### Architecture — pure/masked split
1. **PURE `git_diff.rs`** (gpui-free; NOT gate-excluded → cov/MSI 100 via units): `enum GitMark {Added,
   Modified, Deleted}`; `parse_hunk_new_start(header) -> Option<usize>` (the `+c` from `@@ -a,b +c,d @@`, via
   `split_whitespace().find(starts_with '+')` → `[1..].split(',').next().parse()`); `gutter_marks_from_hunks(
   &FileDiff) -> Vec<(usize, GitMark)>` — the SEGMENT model: walk each hunk's body against a 0-based new-side
   `row` (starting `parse_hunk_new_start - 1`), accumulating a SEGMENT (add-rows + a remove count) between
   `Context` lines; at each Context (and hunk-end) FLUSH the segment via `flush_segment`: add-rows present +
   removes>0 → all add-rows **Modified** (a replaced run); add-rows present + removes==0 → **Added**; no adds
   + removes>0 → a **Deleted** boundary marker at the current `row` (between-rows). `Add` pushes `row` then
   `row+=1`; `Remove` only bumps the remove count (a removed line isn't in the new file); `Context` flushes
   then `row+=1`. Malformed header → skip the hunk (no panic — the parser posture).
2. **Masked app shim `app.rs`**: `git_marks: std::collections::HashMap<PathBuf, Vec<(usize, GitMark)>>` (a
   DECOUPLED per-path store — NEVER a field on `OpenFile`/worktree, Zed's warning adopted) + `git_marks_key:
   Option<(PathBuf, marley_editor::BufferVersion, String)>` (the cache key = path + buffer version + HEAD
   oid). `refresh_git_marks` (`#[cfg_attr(test, mutants::skip)]`, cache-gated): compute the active file's key;
   if unchanged return; else `git rev-parse HEAD` (oid) + `git diff HEAD -- <relpath>` (both argv via
   `marley_command::blocking`, read-only) → `parse_diff` → `gutter_marks_from_hunks` → `git_marks[path]`; if
   the diff is EMPTY and `git status --porcelain -- <relpath>` shows `??` (Untracked), mark every buffer row
   Added; store + stamp the key (a reverted file → empty diff → empty marks = cleared). Wired to save
   (didSave), extchange reload, file-open, and a cheap pump check (the key gates the actual git spawn — an
   idle frame recomputes nothing). The gutter RENDER lane (masked, in the row assembly app.rs:3676): a fixed
   ~3px color-bar child BEFORE the number cell, colored by the row's `GitMark` — Added→`colors.success`,
   Modified→`colors.accent` (the theme has NO warning color, like #327; accent = the VS Code "modified" blue
   convention), Deleted→a thin `colors.danger` caret at the row's TOP boundary; COEXISTS with the #310
   diagnostic number tint (two lanes — the one-lane rule is about diagnostic PRODUCERS, not git). `gutter_
   width` unchanged (the bar is a fixed lane, outside the number width).
3. **Test hook**: `push_git_diff_for_test(path, diff_text)` — feed raw `git diff` text → parse+project → store,
   so a headless test drives the render/lookup without a real git spawn. (The end-to-end wire is a git-tempdir
   integration test.)

### §20 confirm
N/A — route A projects git's OWN unified-diff output (git's tool, already shelled out via `git_working_diff_
in`); no reference-app source. The git-gutter behavior + Zed's decoupled-store note are behavior-only (source
unread); `imara-diff` (route B) permissive reuse. Holds.

### File manifest
- `crates/marley_app/src/git_diff.rs` — ADD `GitMark`, `parse_hunk_new_start`, `flush_segment`,
  `gutter_marks_from_hunks`. cov/MSI 100.
- `crates/marley_app/src/app.rs` — the `git_marks` store + `git_marks_key` fields + init; `refresh_git_marks`
  (masked argv shim, cache-gated) + the wire arms (save/extchange/open/pump); the gutter render lane (masked);
  `push_git_diff_for_test`, `git_marks_for_test` hooks.
- Phase 5: `CHANGELOG.md`, `editor.md`, `crate-map.md`.

### Regression Test Plan (≥1 per REQ)
| REQ | Test(s) | Kind |
|---|---|---|
| 001 | `gutter_marks_pure_add`, `gutter_marks_modify_run`, `gutter_marks_pure_delete_boundary`, `gutter_marks_multi_segment_hunk`, `gutter_marks_multi_hunk` | pure unit cov/MSI 100 |
| 001 | `parse_hunk_new_start_cases` (`@@ -1,3 +2,4 @@`, single-line `@@ -1 +1 @@`, malformed) | pure unit |
| 002 | `git_marks_untracked_all_added` (app helper) + integration | unit + integration |
| 003 | render review + LIVE(fallback) — the bar colors coexist with the diag tint | review + LIVE |
| 004 | `git_marks_cache_key` (a version/oid change invalidates) | unit + review |
| 005 | git-tempdir integration: init + commit + edit → `git diff HEAD` → project → marks (dirty=saved) | integration |
| 006 | git-tempdir integration: revert the edit → empty diff → marks cleared | integration + unit |
| 007 | review — `git_marks` is a standalone field, not on `OpenFile` | review |
| 008 | `gutter_marks_empty_and_malformed` (empty FileDiff, a hunk with a bad header) | pure unit |

Integration (git tempdir): `std::process::Command` git in the TEST (allowed — the ticket's precedent) to
init/commit/edit, then the PURE parse+project over the real `git diff HEAD` output. LIVE drive (env-fallback):
a tracked probe file — add + change + delete a line → three distinct colored bars at the right rows
(pixel-sampled, the #313 contrast lesson); ⌘S persists; revert clears.

### Risks / decisions (load-bearing)
- **Route A = saved-state** — a dirty buffer shows the last-saved marks (git sees disk); refresh on save/reload.
  The live-diff (route B) is the named follow-up. Stated, not hidden.
- **The SEGMENT classifier** — mixed segment (removes + adds) → the add-rows are Modified; pure-add → Added;
  pure-delete → one Deleted boundary caret (not per-line). The between-rows Deleted attaches to the new-side
  `row` where content resumes (end-of-file deletion → the past-end row; the render clamps).
- **Modified → `colors.accent`** (the theme lacks a warning color — verified only `danger`+`success` exist;
  accent = the conventional "modified" blue). Implement confirms no `warning` field slipped in.
- **Per-ACTIVE-file refresh, cache-gated** on `(path, version, head-oid)` — the render only shows the active
  file's lane, so refreshing just it on save/open/tab-switch suffices; the key gates the git spawn.
- **head-oid** via `git rev-parse HEAD` — a commit/branch switch (oid change) invalidates every file's marks.

## Phase 3 — Implement
- **Built to the manifest.** `git_diff.rs` (PURE): `GitMark{Added,Modified,Deleted}`, `parse_hunk_new_start`
  (the `@@ +c` new-side start), `flush_segment` (the segment classifier), `gutter_marks_from_hunks` (walk
  hunk bodies against a 0-based new-side counter, flush segments at Context + hunk-end). `app.rs`: the
  `git_marks` store + `git_marks_key` + `git_marks_gen` fields + init; `refresh_git_marks` (masked, cache-
  gated argv `git diff HEAD -- <rel>` → parse → project; untracked → all-added) + `git_path_is_untracked` +
  `git_marks_for_active`; the pump call; the gen bump on save + extchange reload; the gutter render lane (a
  3px bar child before the number cell, `success`/`accent`/`danger`); `push_git_diff_for_test`/
  `git_marks_for_test` hooks.
- **Compile:** `cargo check --workspace --all-targets` clean (only the expected 2 unused test hooks + the
  pre-existing `block v0.1.6`). `cargo fmt` clean. NO new dependency (route A). `colors.accent` confirmed
  (ui_components:48); the theme has NO `warning` field (only `danger`/`success`), so Modified→`accent`.
- **Deviations from design (with reason):**
  1. **Cache key = `(path, git_marks_gen)`, NOT `(path, buffer-version, head-oid)`.** Route A marks are the
     SAVED working-tree state, so keying on the LIVE buffer version would spawn git on EVERY keystroke; a
     `git_marks_gen` counter bumped on save + extchange reload is the correct route-A cadence AND avoids a
     per-pump-tick `git rev-parse HEAD` spawn (the oid would need re-fetching each frame to compare). A
     file-switch still recomputes (the path moves the key). LIMITATION: a commit via an EXTERNAL terminal
     (not the git panel/save) leaves the marks stale until the next save/reload — a named follow-up.
  2. **Dropped `git_head_oid`** — the gen counter subsumes it (a git-panel commit could bump gen too; that
     wiring is a follow-up alongside the external-commit case).
  3. **The bar is a flex child with the row's `gap_2`** (a small gap before the number cell) — a v1 render
     detail; the live drive would confirm the spacing (env-blocked → accept, tune later if needed).
- **Coverage/mutation reminder for Validate:** the pure `git_diff.rs` additions (`gutter_marks_from_hunks`,
  `parse_hunk_new_start`, `flush_segment`) need cov/MSI 100 via units (git_diff.rs is NOT gate-excluded); the
  app.rs shim (refresh/render/untracked) is gate-excluded, behavior-verified by a git-tempdir integration
  test + headless drives via the test hooks. `refresh_git_marks`/`git_marks_for_active`/`git_path_is_untracked`
  + the render already carry `#[cfg_attr(test, mutants::skip)]`.

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Inspect (Phase 3.5)
3 parallel critics (general-purpose) — lenses: pure-projection correctness, state/refresh integrity,
render/reuse/provenance. **Verdict: SOUND** — critic 1 traced all 8 projection scenarios CORRECT (add/modify/
delete/mixed-segment/multi-hunk/header-parse/off-by-one/no-panic); critic 2 confirmed the `(path, gen)` cache
deviation sound (per-keystroke inert, file-switch recomputes, save/reload bump gen, argv read-only + no
injection); critic 3 confirmed the render lane compiles + coexists with the diag tint + the geometry/click
mapping stays correct + clean-room substance. Two fixes applied (one a gate blocker).

### CONFIRMED → FIXED at source
- **[HIGH · C3 gate blocker] A literal "Zed" in an app.rs comment** ("Zed's decoupling warning adopted",
  git_marks field doc) — gate:14 brand-scrub greps `warp|zed` in `crates/**/*.rs` and WOULD have failed the
  commit. The #327 lesson exactly, and it slipped again: a spec phrase that CITES the reference app was copied
  verbatim into a code comment (the spec lives in docs/, which is not scanned). **Fix:** reworded to "git
  state lives in its own store, not on the file entry" — the substance kept, the brand word dropped.
- **[LOW · C2 UX] An in-app git-panel commit left the gutter stale.** `git_commit` didn't bump `git_marks_
  gen`, so committing FROM Marley's own commit panel left the pre-commit Added/Modified bars until the next
  save/reload/switch — MORE surprising than the external-terminal case (the commit originated inside the app).
  **Fix:** bump `git_marks_gen` on a successful `git_commit`, so the next frame recomputes (the committed
  lines are no longer "changed vs HEAD").

### CONFIRMED → VALIDATE follow-through (critic 1 MED + critic 3 MED — load-bearing)
- The 3 pure fns (`gutter_marks_from_hunks`, `parse_hunk_new_start`, `flush_segment`) have ZERO tests + the 2
  hooks are unused — EXPECTED pre-validate. Validate MUST reach cov/MSI 100. **The non-obvious mutant** (C3):
  `removes > 0` → `removes >= 0` (always true, `removes: usize`) makes EVERY zero-add segment — including the
  empty segment flushed at each Context line — push a spurious `(row, Deleted)`. A naive add-only/modify/
  delete-only test set does NOT kill it. Validate MUST include a **hunk with context AROUND a change** and
  assert EXACT `Vec` equality (no spurious Deleted at the context rows), plus `parse_hunk_new_start("@@ -10,3
  +5,4 @@") == Some(5)` (the `-10` must not win), a malformed header → `[]`, and a multi-hunk row-offset case
  (pins `saturating_sub(1)` + `row += 1`).

### CONFIRMED → documented / deferred (LOW, not fixing)
- **[LOW · C1 render] An END-OF-FILE deletion isn't drawn** — the projection emits `(line_count, Deleted)`
  (past the last row), but the render only queries rows `0..line_count`, so the marker is stored but never
  painted. Mid-file deletions (the common case) render fine. Cosmetic edge; a "fold onto the last line's
  bottom edge" render tweak is the follow-up (the live drive would confirm — env-blocked).
- **[LOW · C2/C3] The cache key omits head-oid** — an EXTERNAL HEAD move (commit/checkout/stash in a
  terminal) that doesn't touch the file's disk bytes leaves the marks stale until save/switch. The documented
  deviation-1 trade-off (dropping the per-frame `git rev-parse HEAD` spawn); the in-app commit case is now
  fixed, the external one remains a named v1 limitation.
- **[LOW · C2] Narrow staleness edges** — a same-content external reload skips the gen bump (needs the write
  to equal the live buffer byte-for-byte); a no-op save (conflict-arm) bumps gen → a spurious-but-correct
  recompute; an untracked EMPTY file → one Added mark (`len_lines`=1 for empty). All harmless.
- **[LOW · C3 reuse] `git_marks_for_active` rebuilds a HashMap per frame** — the marks Vec is already
  row-sorted, so a `binary_search` per row (mirroring the sibling `ediag` lane) would drop the per-frame
  alloc. Bounded by the active file's mark count; deferred as a consistency/perf follow-up.

**Post-fix:** `cargo check -p marley` clean; `cargo fmt` clean; the gate:14 brand-scrub dry-run
(`grep -rniwE 'warp|zed'`) over the #328 files is EMPTY.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

**Tests added (17 asserts across 3 layers):**

- **Pure units — `git_diff.rs` (cov/MSI 100, NOT gate-excluded):**
  - `parse_hunk_new_start_cases` — `@@ -10,3 +5,4 @@`→`Some(5)`, single-line `@@ -1 +1 @@`→`Some(1)`, a
    header-comment tail `@@ -0,0 +20,3 @@ fn x()`→`Some(20)`, no-`+`→`None`, non-numeric `+abc`→`None`,
    empty→`None`. Kills the body→`Some(0)`/`Some(1)`/`None` FnValue mutants and the `starts_with('+')`
    closure→`true`/`false` mutants.
  - `flush_segment_classifies` — the private classifier direct, all four states with EXACT Vec equality:
    adds+removes→`Modified` per row, adds+no-removes→`Added`, no-adds+removes→one `Deleted` at the boundary,
    neither→empty. Kills the `removes > 0`→`>= 0`/`< 0`/`== 0` mutants on BOTH the add-side ternary and the
    else-if.
  - `gutter_pure_add_hunk` / `gutter_pure_delete_hunk` / `gutter_replaced_run_is_modified` — the three
    single-segment shapes.
  - `gutter_mixed_segments_exact` — the load-bearing one: context SEPARATES an Added, a Modified, and a
    trailing Deleted segment, pinning the 0-based row counter (`start.saturating_sub(1)`, the per-arm
    `row += 1`) and the `add_rows.clear()` between segments with `[(1,Added),(3,Modified),(5,Deleted)]`.
  - `gutter_context_resets_removes` — a Deleted segment then a pure-add segment; asserts the second is
    `Added` (not `Modified`), killing a dropped `removes = 0` reset.
  - `gutter_multi_hunk_and_skips_malformed` — two valid hunks at distinct `@@ +c` starts (offsetting rows
    1 and 19, exercising `saturating_sub` for a large start) with a MALFORMED hunk between them SKIPPED
    (the `let Some(start) = … else { continue }`).
  - `gutter_empty_file_no_marks` — REQ-008, no hunks → `[]`, no panic.
- **Real-git integration — `headless_drive.rs` `git_marks_real_diff_integration_headless`:** a seeded git
  repo (`marley_command` git init/config/commit — the `std::process::Command` lint is a hard ban even in
  test setup), a committed 3-line file MODIFIED on disk → `refresh_git_marks_for_test` drives the REAL
  `git diff HEAD` → `parse_diff` → `gutter_marks_from_hunks` → store → `[(1, Modified)]` at the edited row;
  REVERT → the diff empties → marks cleared (`[]`); a brand-new UNTRACKED file → every line `Added`
  (`git status --porcelain` `??` → no HEAD blob). Proves the argv pipeline end to end, not just the pushed
  text.
- **Headless drives — `headless_drive.rs`:** `git_marks_push_and_read_headless` (the push hook projects
  add/modify/delete diffs to the right rows in the decoupled per-path store) and
  `git_marks_coexist_with_diagnostic_headless` (REQ-003 mechanism: the SAME path carries a git `Modified`
  mark in `git_marks` AND a diagnostic row in the `DiagnosticStore` at once — two independent producer
  lanes, neither clobbering the other).
- **Test hook added:** `RootView::refresh_git_marks_for_test` (`#[cfg(test)]`, placed AFTER
  `git_marks_for_test` so no `mutants::skip` attribute detaches) — bumps `git_marks_gen` then calls the
  private `refresh_git_marks`, modeling the save/reload cadence so each call recomputes against live disk.

**Test run:** `cargo nextest run -p marley -p marley_lsp` → **732 passed, 0 failed, 2 skipped**. The 8 new
`git_diff::tests` + 3 new `headless_drive` drives all green.

**Two gate reds found + fixed at source (no baselines):**
- **gate:2 clippy** — `run_git` used the DISALLOWED `std::process::Command` (`clippy::disallowed_types`:
  "use marley_command for OS-parity non-PTY spawn"). Fixed → `marley_command::blocking::Command` with
  `.args(args.iter().copied())`.
- **gate:4 coverage (whole-workspace 100%, `headless_drive.rs` is NOT excluded)** — `run_git`'s assert
  message called `String::from_utf8_lossy(&status.stderr)` LAZILY (only on git failure), an uncovered cold
  region since git always succeeds in the seeded repo. Fixed → a literal assert message (`"a git setup
  command failed"`), no lazy formatting.

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** — 15/15 (coverage 100%, mutation MSI 100%,
clippy clean, gate:14 brand-scrub clean, visual/AX green). The `--diff` receipt is commit-valid.

**LIVE drive — env-blocked (documented, per §7 fallback):** the mac screen is LOCKED
(`CGSSessionScreenIsLocked=1`), which fully blocks `screencapture`/CGEvents — the driven three-color pixel
sample (REQ-003) cannot run. Verified instead via **units + mechanism**: the pure projection is cov/MSI 100;
the real-git integration proves the store is populated from live `git diff`; the coexist drive proves the
git and diagnostic lanes are independent; the render lane itself is a byte-for-byte sibling of the shipped
`ediag` capture-before-the-`'static`-closure idiom (`egit = self.git_marks_for_active()` read before the
`uniform_list` closure, a fixed 3px bar child BEFORE the number cell — `Added`→`success`, `Modified`→
`accent`, `Deleted`→a `danger` caret — coexisting with the diagnostic number tint). Re-verify the pixels
when unlocked (30s, no ticket).

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete

- **CHANGELOG.md** — a "Git gutter" entry added above #327 (the pure projection, the decoupled per-path
  store + gen-bump cadence, the second gutter lane, the named v1 cuts).
- **docs/marley_architecture/editor.md** — a "Git gutter (#328, M21) SHIPS" paragraph after the #327
  problems-panel section (route A over `parse_diff`, the argv `git diff HEAD` posture, the decoupled store +
  gen-bump invalidation incl. the in-app-commit case, the saved-state-while-dirty stance + external-HEAD
  limitation, the two-lane render, route B deferred).
- **docs/marley_architecture/crate-map.md** — the `marley_app` row gains an M21 #328 note (the git_diff.rs
  additions `GitMark`/`parse_hunk_new_start`/`flush_segment`/`gutter_marks_from_hunks` + the app `git_marks`
  store + render lane).
- **Knowledge captured (forge):** AAR `34325337` closed `completed` (effectiveness 5, 6 novel findings
  materialized). Failures + prevention rules recorded:
  - `BF-zed-brand-word-in-source-comment-001` → `PR-claude-reference-app-citation-stays-in-docs-not-source-comments-001`
    (a §20 reference citation belongs in docs/, never a crates/ comment — gate:14 brand-scrub; the #327 repeat).
  - `BF-in-app-commit-stale-derived-cache-001` → `PR-claude-invalidation-gen-bumped-by-all-mutation-sources-001`
    (a gen-keyed derived cache must bump on EVERY write to the underlying state, incl. the app's own actions).
  - `BF-lazy-assert-message-uncovered-cold-arm-001` → `PR-claude-test-helper-assert-message-literal-not-lazy-call-001`
    (a runtime CALL in a test-helper assert message is an uncovered cold arm in a 100%-coverage repo — use a
    literal message; and `std::process::Command` is banned even in test setup → `marley_command::blocking`).
  - **NO new AD** — route A is composition over git's own output, no novel architecture (route B `imara-diff`
    is the named deferred upgrade).

status: Phase 5 — Complete PASS
