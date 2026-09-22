# 429-search-results-multibuffer — pipeline notes

## Phase 1 — Plan (2026-08-14)

**Recall.** AD-claude-byte-offset-crate-to-char-offset (the #339/#347 byte→char seam:
successors MUST reuse `replace_all_regex`'s monotone walk — its consequences section
names this ticket's class explicitly; the multibyte row is REQUIRED); the #428 ledger
(touched consent, journal liveness, PR-claude-428-a/b); the #427 pin that
`match_anchors` are line-truth, never splice inputs. Completed archive: 427/428 (the
substrate), 272 (find bar), 282 (grouped undo), 354 (saves).

**Load-bearing facts.** `find.rs` already owns the whole apply engine
(`replace_all_regex` captures + `replace_all_with`/`replace_all`); the project-search
worker is LITERAL-only by its own doc ("regex … is a follow-up") — the regex mode is
this ticket's one genuinely new seam (per-line, compile-once, the #339 inert-invalid
posture). The #428 machinery (birth, touched, journal, save sweep, live index) is the
apply's substrate unchanged. `begin_undo_group` overwrites an open group (documented
hazard) — the per-buffer bracket must respect it.

**Classification**: work pipeline, M. One slice: regex toggle + replace field +
Replace All (capture-aware) + batch undo + the standing save story. OUT: per-hit
exclusion (reference carries it — recorded deferral), multiline patterns, replace-in-
selection, default-form promotion.

Ticket promoted (BACKLOG row removed). Spec authored fresh against the post-#428 tree
per the shelf's no-pre-authoring method.

## Phase 2 — Design

### Architecture (§20 confirmed: Zed's phase-2 project-search CONTRACT — replace
field, regex captures, apply-across-buffers, one-undo — via Marley's own engines;
no container imported; clean-room holds)

**A. The regex mode lives in the WORKER, not marley_project.** `run_search`'s
per-file body gains a mode fork: literal → `search_lines` (unchanged); regex →
`marley_editor::find::find_all_regex`-family over the file text (the editor crate
already owns `regex`; marley_project stays dep-free). ONE compile per request at
the spawn site (`build_regex(pattern, !case_sensitive)`); a compile error never
spawns — the overlay renders inert with an "invalid pattern" hint (the #339
posture; `SearchMsg` unchanged, results simply absent). Flat char-offset matches
map to per-line `(row, col, len)` via ONE new pure helper in `editor_search.rs`:
`rows_cols_of(text, &[(CharOffset, CharOffset)])` — a single monotone walk
(AD-claude-byte-offset discipline; multibyte row REQUIRED). Per-file/total caps
apply identically (the admit fns are already shared).

**B. Overlay state mirrors the efind bar's own shape** (`efind_replace`/
`efind_focus_replace`/`efind_regex` — RootView fields at app.rs:584-594):
`OpenSearch` gains `replace: String`, `focus_replace: bool`, `regex: bool`. Keys
in `handle_search_key`: Tab toggles field focus; the regex chip toggles with the
SAME binding the efind bar uses (read at implement); typed chars route to the
focused field; the RE-SEARCH debounce is the standing finder push. **Replace All
= ⌘⌥⏎** (a guarded arm BEFORE ⌘⏎, `stop_propagation` — the F-#553 discipline;
the POC confirms the affordance visually with a footer hint "⌘⌥⏎ replace all").

**C. The APPLY (`mb_replace_all_apply`)**: ⌘⌥⏎ → (1) `open_multibuffer_from_search`
(the standing materialize — closes the overlay, pins open files); (2) per result
file: `ensure_mb_target` (the #428 birth; a `None` skips + counts); (3) per
target buffer: live text → regex mode: `replace_all_regex(text, pattern, ci,
template)` → `replace_all_with(buffer, &edits)`; literal: `find_all(buffer,
needle, fold)` → `replace_all(buffer, &matches, &replace)` — BOTH appliers
bracket their own per-buffer undo group (find.rs's own #347 machinery, atomicity
free); (4) journal push per changed file + `touched.insert` + the BATCH record;
(5) flash "N replaced in M files (· K skipped)". The #428 anchors rebase the
excerpt windows over the splices automatically — the refreshed surface is the
live index doing its job.

**D. Batch undo (D3 shaped).** The model gains `batches: Vec<(usize /*start
index into journal*/, usize /*len*/)>` + pure ops in multibuffer.rs:
`batch_note(batches, journal_len_before, n_entries)`, `batch_covering(batches,
journal_len) -> Option<len>` (the top entry sits at the END of a batch → pop the
whole run), invalidation (`batches.retain(start + len <= journal.len())` after
any pop/clear; `journal_note_edit`'s redone-clear also clears batches whose
range broke). `mb_undo`: if the top entry closes a batch → loop `len` pops (each
with the #428 liveness guard; stale entries drain, the loop continues); `mb_redo`
mirrors (redone entries popped as a run — the redone stack records the batch len
symmetrically via `redone_batches`). All pure ops unit-tested.

**E. Reference match**: replace-under-query field + regex captures + whole-sweep
apply + one-gesture revert = the observed phase-2 contract; exclusion deferred
(recorded).

### File manifest

**marley-web (FIRST):** `overlays/ProjectSearch.tsx` — the replace input row
(rendered under the query, the efind bar's twin), the regex chip, Tab focus
toggle, ⌘⌥⏎ apply against the doc store (literal + a JS-regex capture arm for
the demo) + materialize with the applied state + summary line; footer hint.
`views/MultibufferView.tsx` — none (receives applied state).

**crates:**
- `crates/marley_app/src/editor_search.rs` — `rows_cols_of` (pure, monotone,
  multibyte-tested).
- `crates/marley_app/src/multibuffer.rs` — `batches`/`redone_batches` fields +
  the pure batch ops (+ unit table).
- `crates/marley_app/src/app.rs` — OpenSearch fields (replace/focus_replace/
  regex); `handle_search_key`: Tab/toggle/typed-routing/⌘⌥⏎ arm; the worker
  spawn's compile-once + mode fork in `run_search`; `mb_replace_all_apply`;
  `mb_undo`/`mb_redo` batch loops; the overlay render's second row + chip +
  hint; flash.
- `crates/editor/src/find.rs` — expected ZERO changes (the appliers are
  complete); any need is a recorded deviation.

### Regression Test Plan

| REQ | Test | Where |
|---|---|---|
| REQ-001 | Drive: two files seeded → query+replace → ⌘⌥⏎ → both buffers spliced (live truth), mb open with applied rows, flash counts | headless_drive |
| REQ-002 | find.rs capture units stay green; drive with a regex `(\w+)` query + `[$1]` template incl. a multibyte line | headless_drive + existing units |
| REQ-003 | Drive: apply across 2 files → ONE ⌘Z reverts BOTH; ONE ⌘⇧Z re-applies; a tab-side undo between drains per the liveness guard; batch pure-op unit table (note/covering/invalidate) | headless_drive + multibuffer.rs |
| REQ-004 | Unit: per-buffer atomicity via the appliers' own groups (undo_depth +1 per applied file); drive: an unopenable file skips + counts, text untouched | units + drive |
| REQ-005 | Drive: post-apply ⌘S saves touched∩dirty; the #275 hold arm re-verified (extends the 428 consent drive) | headless_drive |
| REQ-006 | Worker units: regex mode rows/cols (multibyte REQUIRED row); invalid pattern → inert + hint (drive) | editor_search.rs units + drive |
| REQ-007 | The #428 drives stay green; one post-apply typing drive | existing + headless_drive |
| REQ-008 | gates.sh --diff GREEN | gate |
| parity | POC ↔ live at the SAME state: replace row + chip + applied mb + summary — pixels sampled | validate |

Uncoverable: none new.

### Risks / decisions
- **Apply-time re-search** means the flash's N can differ from the overlay's
  count (edits since) — honest by design; recorded.
- **⌘⌥⏎** assumes the router delivers alt+platform chords to the overlay
  (the #427 critic verified the overlay receives every key) — verify at
  implement; fall back to a different guarded chord if the platform eats it.
- **Batch invalidation** is conservative: any journal shrink below a batch's
  range drops the batch (single-entry undo resumes) — never a wrong-range pop.
- The POC's apply uses JS `String.replace` semantics for the capture demo —
  recorded as a stand-in (the Rust truth is `replace_all_regex`).

## Phase 3 — Implement

**React-first (built + captured FIRST).** ProjectSearch.tsx: the replace row under
the query (efind twin), the clickable `.*` chip + ⌘⌥R, Tab field-hopping, regex-mode
search (compile-once, invalid → "invalid pattern" inert hint), ⌘⌥⏎ Replace All
applying to the doc store (JS capture semantics as the recorded stand-in) +
materializing the APPLIED multibuffer; the hint row grew "⌘⌥⏎ replace all". A STALE
CLOSURE bug (the keydown effect's deps missed replace/regexMode — the first apply
spliced with an EMPTY replacement) was caught by the drive-through and fixed (deps
carry the closure's reads; resubscribe churn recorded). tsc green. Capture READ:
scratchpad/429-poc-applied.png ("Serialize"→"Encode" applied, dirty dot, summary).

**Rust (to the manifest).**
- `crates/editor/src/find.rs`: `CompiledFind` (opaque) + `compile_find` +
  `find_all_compiled` — `find_all_regex`'s walk extracted into a shared inner
  (per-call compile keeps suiting the find bar; the 500-file walk compiles ONCE).
  (A doc-splice mishap en route was caught immediately by the docs lint —
  PR-claude-427-b honored on the second try.) lib.rs re-exports grew.
- `crates/marley_app/src/editor_search.rs`: `rows_cols_of` (one monotone walk,
  char domain).
- `crates/marley_app/src/multibuffer.rs`: `batches`/`redone_batches` +
  `batch_note`/`batch_covering`/`batches_invalidate` (pure).
- `crates/marley_app/src/app.rs`: OpenSearch grew replace/focus_replace/regex/
  invalid_pattern (all 3 construction sites); handle_search_key: Tab hop, ⌘⌥R
  toggle (`toggle_search_regex` re-parks), ⌘⌥⏎ arm BEFORE ⌘⏎, backspace/typing
  routed by focus; SearchReq carries the ONE pre-compiled pattern (invalid →
  inert flag, no spawn); run_search's regex arm (compiled walk → `rows_cols_of` →
  LineMatch with the literal engine's own cap + preview clamp); 
  `mb_replace_all_apply` (materialize → per-file ensure target → find.rs appliers
  with their OWN per-buffer undo groups → journal + touched + ONE batch → summary
  flash); `mb_undo`/`mb_redo` refactored to `_one` + batch loops (a covering
  batch pops whole; the #428 liveness guard per entry intact; conservative
  invalidation); mb_after_edit clears batches on ordinary edits; the overlay
  render: replace row + passive `.*` chip + extended hint + the invalid-pattern
  title.

**Deviations (recorded):** the Rust chip is a passive indicator (`search_overlay`
is `&self` — the overlay is keyboard-only like its rows; ⌘⌥R is the toggle; the
POC keeps its clickable chip + gains the key); the overlay searches
case-insensitively in BOTH modes (the spawn's standing opts); POC capture
semantics are JS `String.replace` (the Rust truth is `replace_all_regex`).

`cargo check --workspace --tests` clean; clippy zero; multibuffer suite 29/29
(the #428 drives stay green); fmt applied.

## Inspect (Phase 3.5)

Two critics (apply/undo correctness — probe-backed against the real engines;
reuse/provenance/parity across both repos). Every confirmed finding fixed.

**Confirmed → fixed:**
1. **HIGH (both critics) — the stale-multibuffer fallthrough.** ⌘⌥⏎ with an
   unmaterializable set (streaming/empty/unsourceable) left the overlay open,
   and `active_mb_id()` then pointed at whatever OLD multibuffer tab sat behind
   it — the NEW query applied over the OLD file list, journal and batch written
   into the wrong model, text the user never previewed rewritten. Fix: bail
   HARD when `open_search` is still Some after the materialize (its None is the
   success signal) + up-front guards for empty/invalid; the POC always had the
   empty-set guard (a parity regression). Also closes the batch-under-covers
   LOW (a fresh materialize always starts at journal 0).
2. **HIGH — the empty-match walk/apply divergence (probe: `a*` showed 1 match,
   spliced 3).** The walk hides empty matches (`a < b`); the apply consumed
   `replace_all_regex`'s edits unfiltered. Fix: `edits.retain(|(a, b, _)| a < b)`
   — what-you-see-is-what-replaces.
3. **MED — a streaming walk under-applied silently.** `SearchMsg::Done` was
   discarded; ⌘⌥⏎ mid-stream covered a partial set. Fix: `OpenSearch.complete`
   (set on the live gen's Done; cleared per consume; empty-query and invalid
   arms finalize; the test seeds mark complete) + the apply refuses with
   "Search still running — try again".
4. **MED — invalid-at-apply swallowed per-file.** Fix: the apply guards on
   `invalid_pattern` up front (the per-file `Err → 0` stays as the belt).
5. **MED reuse — the regex assembly lived inline in the worker** (uncoverable by
   the planned units) and collected every match pre-cap. Fix:
   `editor_search::to_file_matches` (pure, unit-coverable; the cap/preview/
   truncated mirror in ONE place) + an early `take(cap + 1)` on the walk's
   iterator. (`find_all_compiled`'s internal full collect stays — bounded by
   the 2MB file cap; recorded.)
6. **LOW — `pos.min(start)` dead arithmetic** in `rows_cols_of` (the retired-
   pivot equivalent-mutant class) → `pos.saturating_sub(line_start)` + the
   AD-required ascending `debug_assert!`.
7. **LOW — a focused EMPTY replace row rendered blank.** Fix: the efind bar's
   own ▏ caret mark rides the focused field.
8. **POC MED — the literal apply was case-SENSITIVE** while search (both sides)
   and the Rust apply fold — a "fn" query left "FN" lines marked-but-unchanged.
   Fix: escape + `RegExp('gi')` in both modes.
9. **POC MED — Tab moved focus query→replace but never back** (autoFocus fires
   at mount only; the replace input had the remount key, the query didn't) —
   typing went dead until a click. Fix: the same key trick on the query input.
10. **POC LOW — hint lacked "· ⌘⌥R regex"** (added) **+ the dead `replacedFiles`
    local** (removed; the summary flash is recorded Rust-only).

**Recorded, no action:** whole-text regex matching means a `\n`-bearing pattern
matches across lines and `^`/`$` anchor text edges (walk and apply AGREE; bands
clamp; no panic) — a deviation from the design's "per-line" wording, recorded ·
⌘⌥R is a MINTED chord (efind's toggle is palette-only — nothing to reuse) ·
per-file recompile at apply time (one-shot gesture; commented) · the three
OpenSearch initializer sites (two are test hooks) · POC micro-deltas (border
alpha, chip rounding) + first-match-per-line POC search (inherited #326
simplification) + the POC's two mb-builder arms sharing ~25 lines (stand-in).

**Verified clean:** `rows_cols_of` multibyte/CRLF/boundary behavior (probed);
cap/preview/case parity between engines (at-cap → truncated=false BOTH);
batch undo/redo cycles (duplicate (start,len) impossible; stale in-batch
entries pop-but-skip with correct redo pairing); the apply's read-before-
materialize order; overlay routing (replace edits never re-park; Esc drops
all; arm order); the find.rs split is byte-identical (30/30 find tests);
provenance clean (brand scrub zero).

**Verify:** workspace check clean · multibuffer 29/29 · find 30/30 · clippy
zero · POC tsc clean · fmt applied.

Ledger appends: F-claude-429-a, PR-claude-429-a.

## Phase 4 — Validate

**Units (Phase 3 + this phase):** multibuffer.rs 28/28 (incl.
`batch_ops_note_cover_invalidate` — note/cover/invalidate + the len-0 skip);
`editor_search::rows_cols_of_maps_char_ranges_to_rows` (multibyte "αβγ δ\nεζ
foo" → (1,3,3) — the saturating_sub fix proven) +
`to_file_matches_mirrors_cap_and_preview` (cap CUT → truncated, preview clamp);
editor crate: find 30/30 (split byte-identical), `undo_depth_counts_groups_and
_the_open_transaction`.

**Headless drives (2 new, both green):**
- `replace_all_applies_and_batch_undoes_headless` — seeded 2-file project;
  ⌘⇧F → "delta" → Tab → "X" → ⌘⌥⏎: both buffers spliced, flash "3 replaced in
  2 files", ONE ⌘Z reverts BOTH files (batch loop), ONE ⌘⇧Z re-applies, ⌘S
  persists both to disk (touched ∩ dirty).
- `replace_all_regex_capture_and_guards_headless` — REAL worker walk
  (poll_until); streaming guard flashes "Search still running — try again";
  ⌘⌥R + `(t)hree` + `[$1]` → capture template applied; tab-switch + invalid
  `(` → overlay inert, nothing applied (invalid-at-apply guard).

**Full suite:** `cargo nextest run --workspace` **2231/2231 PASS** + doctests
green (real run in transcript).

**Live drive (scratch HOME, 428-live fixtures, WIN 1898):** ⌘⇧F → "delta" →
Tab → "X" → ⌘⌥⏎ (System Events key code 36 {command, option down}).
- `429-live-applied.png` READ: alpha "gamma X" + beta "b0 X"/"b6 X" applied,
  dirty dots ●1/●2 on both headers, match washes, footer "3 matches in
  2 files", status "focus: search results".
- `429-live-reverted.png` READ: ONE live ⌘Z restored ALL THREE rows across
  BOTH files ("gamma delta", "b0 delta", "b6 delta") — the one-gesture batch
  revert proven on the real app; caret parked at the undo site.

**Parity pair (React-first):** `429-parity-poc.png` (POC applied state:
Serialize→Encode with replace row exercised) ↔ `429-live-applied.png`.
Pixel-sampled (PIL): POC match wash (17,30,34) ↔ Marley (18,28,31); header
band (26,27,31) ↔ (25,26,28); body (14,15,17) family — the SAME Δ≤4 token
families pinned at #427/#428, verdict 1:1. The replace row + ".*" chip are
proven on both sides by the live apply gestures themselves (the row is
transient chrome consumed on apply); POC row pixels sampled at build time
(Phase 3 capture READ). Paths in scratchpad; verdict: PASS.

**Gate:** `git add -A && scripts/gates.sh --diff` → **GATE GREEN [diff]**
(receipt written; output in transcript). No pre-existing failures touched.

**Gate round (honest record):** first `--diff` run was RED — cov 1 uncovered
line (compile_find's empty-pattern `Ok(None)` arm; the app guards empty before
compiling, so only a unit reaches it) + MSI 87.2% (6 missed, all in new code:
find.rs compile_find→Ok(None) / find_all_compiled→vec![]/vec![default] —
the editor crate had no direct consumer test; rows_cols_of `<`→`<=` — no range
starting AT a newline char; batch_covering `>`→`>=` — no zero-len entry probe;
batches_invalidate `+`→`*` — no case where sum and product straddle the
boundary). Fixes at source: `t429_compile_find_rules_and_compiled_walk_
equivalence` (editor — empty/invalid/valid rules + exact-offset equivalence
with find_all_regex), a newline-start range row in `rows_cols_of_maps_char_
ranges_to_rows`, and zero-len-cover + (2,3)/(5,1)@5 edge rows in
`batch_ops_note_cover_invalidate`. Re-run: **GATE GREEN [diff]**, mutation
47 caught / 0 missed → MSI 100.0%, cov 100%.

## Phase 5 — Complete

Docs: CHANGELOG entry (429, top of Added); `docs/marley_architecture/editor.md`
— new "#429 — replace all" paragraph in the multibuffer section, the #326
⌘⇧F paragraph's "deferred phase 2" corrected to shipped-across-#427/428/429,
deferred list trimmed to #430+; `roadmap.md` M32 row now names #429 shipped.
Parity sync: `marley-web/docs/MARLEY-PARITY.md` ProjectSearch row grew the
#429 ✅ block (what's mirrored, the three POC fixes made during the ticket,
the recorded Rust-only gaps: summary flash, batch undo, every-match walk).
POC already matches shipped surface — no back-port needed beyond the fixes
already landed during implement/validate.

Knowledge appended: L-claude-429-a-library-fn-needs-its-own-crate-tests-for-
the-mutation-gate-001 (lessons), AD-claude-429-replace-all-is-a-batch-over-
the-journal-not-a-new-history-001 (architecture-decisions). F-claude-429-a +
PR-claude-429-a were appended at inspect.

Ticket closed → tickets/closed/; pipeline pair archived → completed/.
