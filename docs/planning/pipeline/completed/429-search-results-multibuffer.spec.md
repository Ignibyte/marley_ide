---
pipeline_id: 61a14e6c-c309-4dd2-89ee-027f5941c4c3
ticket: docs/planning/tickets/open/TICKET-429-search-results-multibuffer.md
status: Phase 5 — Complete PASS
title: ⌘⇧F phase 2 — editable results + Replace All with capture groups
type: feature
milestone: M32
references:
  - docs/planning/design-notes/display-map-shelf.md
  - docs/planning/pipeline/completed/427-multibuffer-core.spec.md
  - docs/planning/pipeline/completed/428-multibuffer-editing.spec.md
  - docs/zed_architecture/crates/search.md
---

## Title

Project search completes (the B-c chain's step 5). The ⌘⇧F bar grows a REPLACE field
and a regex toggle; **Replace All** applies the replacement to every current match
across all result files — capture groups (`$1`/`${name}`/`$$`, the #347 semantics)
in regex mode — as ONE undoable story, materializing/refreshing the #427/#428
multibuffer to show the applied state. Nothing auto-saves: changed buffers go dirty +
TOUCHED, and the standing #428 ⌘S sweep (touched ∩ dirty, #275-netted) persists them.
Editing inside the refreshed excerpts keeps the #428 contract.

## Scope

### In
- **The regex toggle for project search**: `SearchOpts` grows a regex mode; the
  worker's per-line matcher gains a regex arm reusing the #339 find machinery (one
  compiled `Regex`, per-line matching — multiline patterns explicitly out, recorded);
  the overlay gets the toggle affordance (the editor find bar's chip precedent) and
  an invalid-pattern posture (the #339 bar's own: inert + hinted, never a panic).
- **The replace field**: a second input row in the ⌘⇧F overlay (POC designs it);
  focus/tab order + the Replace All invocation (keyboard-first — the POC picks the
  affordance; a guarded key arm per the F-#553 discipline).
- **The APPLY**: per result file — ensure a live target (the #428 birth path; the
  500-file cap bounds it; unopenable files skip + count), re-run the search on the
  LIVE buffer text (apply-time truth — stale search rows never splice), and apply via
  the find.rs engines: `replace_all_regex` (capture-expanded, the monotone byte→char
  walk — AD-claude-byte-offset…) in regex mode, the plain replacer otherwise; ONE
  undo group per buffer; every changed file marked dirty + touched; the multibuffer
  materialized (or refreshed) showing the applied state; a summary flash
  ("N replaced in M files · K skipped").
- **One-gesture undo**: a single mb ⌘Z reverts the ENTIRE apply across files (the
  journal grows a batch notion — design shapes it); one ⌘⇧Z re-applies.
- **React-first**: ProjectSearch.tsx gains the replace row + toggle + apply flow
  against the doc store FIRST.

### Out (explicitly deferred)
- Per-hit / per-file exclusion before replacing (the reference carries it; recorded
  as the follow-up — v1 replaces every current match).
- Multiline regex patterns; Unicode case-folding (the worker's recorded follow-up).
- Replace-in-selection / single-hit replace inside the multibuffer.
- Making ⌘⏎ the overlay's DEFAULT enter action (the overlay stays the picker; the
  #427 ⌘⏎ materialize + this ticket's apply-materialize are the mb entries).
- The problems form (#430); the DisplayMap unification (the chain's prize).

## Reference (§20)

**Zed (the editor reference — same-gpui-stack).** Behavior matched: project search's
phase 2 — a replace field under the query, regex mode with capture-group replacement,
Replace All applying across every result buffer with the results surface reflecting
the applied state, undo reverting the sweep. Research:
`docs/zed_architecture/crates/search.md` (the phase-1/phase-2 split this chain
follows; "results anchored to buffers, not offsets" — which #428's live targets now
provide) and the #427 spec's cited multibuffer chapters. Adopted as CONTRACT, not
container: the apply is per-buffer through Marley's own find.rs engines; no
BufferStore/ProjectSearch machinery is imported. Clean-room: deconstruction docs
only; no Zed source read.

### Prior art

1. **Behavior maps** — `docs/zed_architecture/crates/search.md` phase 2 (replace
   field + replace-all + exclusion; exclusion recorded as the deferral).
2. **Published** — the `regex` crate's replacement syntax (`$1`, `${name}`, `$$`) —
   already adopted verbatim at #347; no new reading needed.
3. **In-tree (the highest-yield leg):** the ENTIRE apply engine exists —
   `find.rs::replace_all_regex` (capture-expanded, ascending non-overlapping
   `captures_iter`, the ONE monotone byte→char cursor + the debug_assert + the
   REQUIRED multibyte row, per AD-claude-byte-offset-crate-to-char-offset…-001 whose
   consequences section NAMES #347-successors as must-reuse), `replace_all_with` /
   `replace_all` (the plain splice), `find_all_regex` (the #339 matcher + its
   invalid-pattern inert posture). The #428 multibuffer machinery is the apply's
   substrate: `ensure_mb_target` (per-file birth), `touched` (the save consent set),
   the journal (one entry per undo group — the batch notion extends it),
   `mb_save_all` (the persistence story, already drive-proven), `sync_multibuffer_live`
   (the refreshed surface). `Buffer::begin/end_undo_group` (per-buffer atomicity —
   the #282 discipline; `begin` OVERWRITES an open group, the documented hazard to
   respect). The search worker (`marley_project::search_lines` + `SearchOpts`) is
   LITERAL-only today ("regex … is a follow-up" — its own doc); the regex arm
   reuses the find.rs compile + the same char-offset discipline. The overlay's
   finder-input shape (#326) is the replace field's twin.

## React-first (parity)

**UI-AFFECTING — the ⌘⇧F overlay grows a row + the apply flow (zone B; POC is the
design source).** POC files: `overlays/ProjectSearch.tsx` (the replace input row, the
regex toggle chip, the Replace All affordance + keyboard path, the apply against the
doc store + materialize), `views/MultibufferView.tsx` (unchanged surface; receives
the applied state). Implement builds + screenshots the POC FIRST; validate captures
the parity pair; complete re-baselines MARLEY-PARITY's project-search row.

## Locked-In Decisions

- **D1 — The find.rs engines apply; nothing re-derives spans.** Regex mode →
  `replace_all_regex` per buffer (captures, multibyte-safe); literal → the plain
  replacer. The #428 `match_anchors` are LINE-truth only and are never splice inputs
  (the recorded #429 pin honored). The apply re-runs the search on the LIVE buffer
  text at apply time — "replace what matches NOW", per-buffer atomic.
- **D2 — Live targets for every applied file.** The #428 birth path per result file;
  unopenable files skip + count into the summary; the mb materializes/refreshes with
  the applied state (the #427 lifecycle + #428 live index unchanged).
- **D3 — One gesture reverts the sweep.** The journal grows a BATCH notion (design
  shapes it: batch-tagged entries popped together, the liveness guard per entry
  intact) — one ⌘Z undoes the whole apply, one ⌘⇧Z re-applies.
- **D4 — No auto-save.** Changed buffers: dirty + touched; the #428 ⌘S sweep is the
  persistence story (nothing new).
- **D5 — Regex is a per-search MODE with the #339 posture**: a chip/toggle, an
  invalid pattern renders the bar inert with a hint, never panics, never blocks
  literal search.
- **D6 — Per-hit exclusion deferred** (recorded; the reference carries it, v1 does
  not).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a query, mode, and replacement are set and Replace All is invoked, the system shall apply the replacement to every match of the LIVE buffer text across all result files, materialize/refresh the multibuffer showing the applied state, and flash "N replaced in M files (· K skipped)". | Headless drive. |
| REQ-002 | IN regex mode, the replacement shall expand `$1`/`${name}`/`$$` per match (the #347 semantics), multibyte-safe. | Reused find.rs units stay green + a capture-query drive. |
| REQ-003 | ONE ⌘Z gesture in the multibuffer shall revert the ENTIRE apply across every file (and one ⌘⇧Z shall re-apply it), tab-side interleaves draining per the #428 liveness guard. | Headless drive + journal units. |
| REQ-004 | The apply shall be per-buffer atomic (one undo group per buffer); a file that cannot take a live target shall be skipped and counted, its text untouched. | Units + drive. |
| REQ-005 | Replaced buffers shall be dirty + TOUCHED; ⌘S shall persist them through the standing sweep with the #275 hold intact. | Drive (extends #428's consent drive). |
| REQ-006 | The regex toggle shall switch the worker's matching; an invalid pattern shall render search inert with a hint (no panic, no stale results). | Worker units + drive. |
| REQ-007 | Editing inside the refreshed excerpts shall keep the #428 contract (typing/undo/boundaries). | The #428 drives stay green + one post-apply edit drive. |
| REQ-008 | New pure surface at 100% cov / 100% MSI; gate GREEN [diff]. | gate:4/5. |

## Phase Plan

- **P2 Design** — the batch journal shape; the worker's regex arm (compile-once,
  per-line, cap interplay); the overlay's second row + toggle + key affordance (POC);
  the apply orchestration (borrow phases; per-file group brackets); the test table.
- **P3 Implement** — POC FIRST, then Rust to the manifest.
- **P3.5 Inspect** — critics on: apply atomicity + capture correctness (the AD's
  multibyte seam), batch-undo soundness, target lifecycle at 500 files, overlay arm
  discipline, parity.
- **P4 Validate** — REQ tests + drives; live drive + parity pair; gate GREEN.
- **P5 Complete** — docs; MARLEY-PARITY re-baseline; ledger; close + archive.
