# 425-display-map-foundation — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-425-display-map-foundation.md
- **Pipeline spec:** 425-display-map-foundation.spec.md

## Phase 1 — Plan

- **Request:** TICKET-425 — the display-map foundation, first of the six-ticket M32 B-c
  sprint (Chad's "/work 425 to 430 auto approved until finished"; sprint provenance in
  `docs/planning/design-notes/display-map-shelf.md`).
- **Classification / tier:** feature, L, single shippable slice (a byte-identical
  refactor + new pure surface). Systems: editor render path (`marley_app`), the
  `marley_syntax` fold projection (consumed, not changed).
- **Recall (§18.3):** the shelf note carries the full sweep; headline pins —
  AD-claude-305 (half-open runs; boundary mutants stay killable), AD-claude-two-boundary-
  maps (#331 — caret map vs code map must stay forked), AD-hscroll-shift-clip-split
  (#336 — two probes in opposite domains; TWO shift sites: app.rs:6911 + :7391),
  PR-reveal-hook (F-#305 — jumps bypass the caret-follow primitive), F-#352 (row-vs-slot
  equality gate froze geom), PR-137 (sizer and `'static` row closure must resolve the
  surface the same way; grep the original pattern after a move — any receiver),
  PR zero-width-boundary probing for byte-identical claims, F-#386 (`cargo check
  --tests` after signature moves), and the MSI-hollowness family (PR-99/757/837/1370 —
  raw-representation asserts; structural asserts; no-mutant steps need discriminating
  fixtures). Completed-archive recall: 305/352 notes carry the rim + crossings design
  and the fold-parse memo history.
- **Discovery:** the 2026-08-14 Explore sweep (recorded in the shelf note) — the rim is
  `app.rs:6735` (`proj.buffer_row(slot)`), count `:6729`; 14 production `FoldProjection`
  crossing sites + test hooks; the memoized builder `fold_projection()`
  `app.rs:13851-13891` with `FoldProjCache` `:820-825`; ~19 interior row==line sites
  (inert under folds, the wrap ticket's problem); `app.rs` is the coverage-excluded shim
  — every new mechanism lands in a pure file. gpui-0.2.2 registry read: `uniform_list`
  rim is `usize`; `LineWrapper::wrap_line` exists (recorded for #426).
- **Decisions:** D1–D5 in the spec (facade-not-framework; typed row spaces; byte-identical
  proven; #331 maps untouched; newtype-owner fork constrained to design).
- **Auto-approval:** the sprint directive is autonomous-through-commit; Phase-1 human
  confirmation is covered by it (recorded here per §15 honesty).

## Phase 2 — Design

### Architecture (§20 confirmed)

The design matches the Zed CONTRACT (one authority for row-space conversion; layers
compose behind it) at Marley's smaller shape — a facade over the shipped
`FoldProjection`, no SumTree, no layer trait until #426 gives it a second layer (D1). No
Zed source was read; the deconstruction docs only. React-first N/A holds — byte-identical.

- **`marley_text_offsets`** (D5 RESOLVED — vocabulary crate): two new `offset_newtype!`
  emissions, **`BufferRow`** (0-based line index in the active buffer's row space) and
  **`DisplayRow`** (0-based rendered-row index — the `uniform_list` slot). Rationale:
  sole-owner precedent (seam-contracts §1), the macro's mutation-aware discipline for
  free (Copy/Ord/Hash, `as_usize`, `From<usize>`, `Add<usize>`, `Add/Sub<Self>`), and the
  crate's EXISTING trybuild suite (`tests/ui/*`) hosts REQ-005 without linking gpui.
  Cross-space impls do not exist, so a BufferRow↔DisplayRow mix cannot compile.
- **NEW `crates/marley_app/src/display_map.rs`** (pure, gpui-free — the code_view/h_scroll
  precedent): `pub(crate) struct DisplayMap { folds: marley_syntax::FoldProjection }` with
  the exact surface the crossing sites consume — `visible_count() -> usize` (a count),
  `buffer_row(DisplayRow) -> BufferRow` (keeps the delegate's EOF clamp),
  `slot_of(BufferRow) -> DisplayRow`, `folded_headers() -> Vec<(BufferRow, usize)>`,
  `viewport_offset(BufferRow, first: DisplayRow, last: DisplayRow) -> Option<usize>`
  (dy stays a count). In-file `#[cfg(test)]` unit + equivalence tables.
- **`app.rs`**: `fold_projection()` is ABSORBED into `fn display_map(&self) ->
  crate::display_map::DisplayMap` — the #352 memo (`FoldProjCache`, (nonce, version,
  anchors) key) moves intact; after this, `FoldProjection` appears in app.rs ONLY inside
  `display_map()` + the cache type (the REQ-001 audit set). `EditorFrameGeom.first/last`
  become `DisplayRow` (the F-#352 class turns into a compile error at the geom seams).
  The rim unwraps ONCE (`let row = map.buffer_row(slot).as_usize()`) so the ~19 interior
  sites keep buffer-row `usize` semantics UNTOUCHED (D-PROJECT-AT-THE-BOUNDARY survives
  verbatim). The sizer + row closure keep sharing the single moved `map` (PR-137
  same-instance by construction, unchanged shape).
- **Preserved micro-contracts** (each keeps its documented semantics verbatim):
  the dwell's below-EOF RAW passthrough (`slot >= visible_count` arm, app.rs:15206-15211);
  the inlay viewport's #401 exclusive-end derivation (`last==0` guard +
  `buffer_row(last-1)+1`); the sticky band's outbound conversion (:7307);
  `scroll_editor_to`'s slot unwrap at the gpui `scroll_to_item` rim (:16426); the five
  `viewport_offset` card anchors + both IME sites. Public helpers
  (`scroll_editor_to(row: usize)` etc.) keep `usize` params this ticket — the typed
  boundary is the facade + geom; widening app-wide is later hygiene, not 425.

### File manifest

1. `crates/marley_text_offsets/src/lib.rs` — MOD: emit `BufferRow` + `DisplayRow`; extend
   the crate-doc "sole owner" sentence; docs per `#![deny(missing_docs)]`.
2. `crates/marley_text_offsets/tests/ui/mixed_rows_fail.rs` (+ `.stderr`) — ADD: the
   REQ-005 compile-fail pair (assignment mix + fn-arg mix), toolchain-pinned rendering
   (#423).
3. `crates/marley_app/src/display_map.rs` — ADD: the facade + its unit/equivalence tests.
4. `crates/marley_app/src/lib.rs` — MOD: register `mod display_map;`.
5. `crates/marley_app/src/app.rs` — MOD: `display_map()` builder (absorbs the memo);
   `EditorFrameGeom` typing + recorder; the 14 crossing sites; the cfg(test) hooks
   (`fold_visible_count_for_test`, `is_row_hidden_for_test`, `folded_headers_for_test`)
   re-routed with `usize` in/out preserved.

### Regression test plan (≥1 row per REQ)

| REQ | Test | Where |
|---|---|---|
| REQ-001 | Grep audit: `FoldProjection` mentions in `marley_app` == {display_map.rs, `display_map()` + cache}; `fold_projection(` call-pattern (any receiver) == zero | inspect ledger (recorded) |
| REQ-002 | `no_folds_is_identity_*`: `visible_count==total`, `buffer_row(s)==s` ∀ s∈0..n+2, `slot_of(r)==r`, `folded_headers().is_empty()`, `viewport_offset` in/out-of-range | display_map.rs unit |
| REQ-002 | No-parse on the no-fold path: the early-return structure preserved (review) + the #352 memo/render tests pass unchanged | REQ-004 run + inspect |
| REQ-003 | `facade_equals_fold_projection_*` over discriminating fixtures {single, nested-merged, adjacent, at-EOF, header-at-0, ≥3 out-of-order} (PR-1370): every method equals the direct `FoldProjection` for all slots/rows incl. `[start,end)` edges; `viewport_offset` over a (first,last) grid incl. degenerate | display_map.rs unit |
| REQ-004 | `cargo nextest run --workspace` green; diff audit — zero edits to existing tests | validate + inspect |
| REQ-005 | trybuild: `let d: DisplayRow = BufferRow::…` + cross-typed fn arg → compile fail with pinned stderr | text_offsets tests/ui |
| REQ-006 | `scripts/gates.sh --diff` green (gate:4 cov 100, gate:5 MSI 100 on touched lines) | validate |
| render guard | #305/#352 fold render + geometry drives keep passing (sizer/closure agreement, PR-137) | existing headless_drive suite |

Genuinely uncoverable: none — the facade is pure; app.rs wiring is the coverage-excluded
shim by standing policy.

### Risks / decisions

- R1 — app.rs churn across 14 sites + geom typing: mechanical-drift risk; mitigated by
  the per-site manifest + `cargo check --tests` after each cluster (F-#386).
- R2 — the newtypes lack `Sub<usize>`: the #401 site (`geom.last - 1`) uses `as_usize()`
  or `Sub<Self>`, PRESERVING the `last == 0` guard + saturating shapes exactly.
- R3 — `From<usize>` is a visible escape hatch (same posture as CharOffset); the trybuild
  pins the cross-space mix, which is the class that bit (F-#352).
- R4 — `folded_headers`'s typed return touches one production consumer + one hook —
  unwrap at the consumer, keep `einlay`/`foldables` keys `usize` (interior untouched).
- R5 — deferred candidate lesson (append at Phase 5 if it proves out): "a vocabulary
  crate with standing trybuild is the cheapest home for new coordinate newtypes".

## Phase 3 — Implement

React-first: N/A (byte-identical refactor, per spec).

Built to the manifest: `BufferRow`/`DisplayRow` emitted in `marley_text_offsets` (crate
doc extended); `crates/marley_app/src/display_map.rs` added (facade + `identity()`
fast-path constructor); `mod display_map;` registered; `fold_projection()` ABSORBED into
`display_map()` (memo intact, three early-return arms now `DisplayMap::identity(…)`);
`EditorFrameGeom.first/last` typed `DisplayRow`; all 14 crossing sites + the 3 fold test
hooks re-routed (zero `fold_projection()` call sites remain; `FoldProjection` mentions in
app.rs = the cache type + the builder — the REQ-001 set, verified by grep mid-phase).

**Deviations from design (both additive, zero behavior change):**
1. The manifest's 14 sites were not the whole surface — the COMPILER found two more the
   sweep missed, exactly the D2 payoff: `HoverCard.first_row` (a slot-domain snapshot
   compared against `geom.first`, its domain previously prose-only — now typed
   `DisplayRow`) and `seed_editor_geom_for_test` (the #352 geom seeder — internal
   conversion added, `usize` hook params preserved).
2. `DisplayMap::identity(total_rows)` added beyond the designed five methods — the
   no-fold/no-editor arms read better than `new(FoldProjection::new(total, &[]))` ×3, and
   it is a pure, mutation-testable path.

`cargo check --workspace --tests` clean (the F-#386 lane); `cargo fmt` applied + check
green. Tests deferred to Phase 4 per §7 (the facade's unit/equivalence tables + trybuild).

## Inspect (Phase 3.5)

Three independent critics over the full diff (correctness · data/type integrity ·
simplification+provenance), each instructed to verify concretely; plus the lead's own
hunk-by-hunk review (every geom.first/last reader re-enumerated — 18 sites — and the
call-site census 14→14 confirmed 1:1).

| # | Finding | Severity | Verdict | Fix |
|---|---|---|---|---|
| 1 | `folded_headers` allocated a SECOND Vec per fold-render (facade re-collect on top of the delegate's) | low | REAL (perf-only; both critics flagged) | Facade returns `impl Iterator<Item=(BufferRow,usize)>`; render walks it directly, the test hook collects |
| 2 | Four stale `fold_projection` prose mentions after the rename (app.rs:299/818/6755, headless_drive.rs:10429) | low | REAL — comments must not mislead (§14) | All four renamed to `display_map` (the memo cache wording kept — it truly stores a raw `FoldProjection`) |
| 3 | Unwrap→sub→rewrap at the inlay exclusive-end (`DisplayRow::from(geom.last.as_usize() - 1)`) | minor | REAL — the newtype's `add_signed(-1)` is the exact API (guard `== DisplayRow::zero()`; debug/release semantics identical under the `==0` guard) | Applied |
| 4 | IME click inverse unwrapped `geom.first` for an add the dwell site already does typed | minor | REAL — idiom mismatch | `(geom.first + n).min(DisplayRow::from(…))` applied |
| 5 | text_offsets' "neither macro instantiation left uncovered" norm broken by the two new emissions (no in-crate smoke; stale "neither") | low | REAL — but test AUTHORING is Phase 4 by charter | Carried to validate: `bufferrow_displayrow_api_smoke` + comment fix |
| 6 | Correctness sweep: dwell below-EOF passthrough, #401 exclusive-end, click clamps, five viewport_offset arg orders, identity/cache arms, HoverCard single writer/reader, sizer/closure same-instance, Add overflow semantics | — | ALL DISSOLVED (verified byte-identical; quoted evidence in critic transcripts) | none needed |
| 7 | Provenance: display_map.rs + newtype docs Marley-original; zero Cargo.toml/lock delta; clippy -D warnings clean (critic-run) | — | CLEAN | — |

No F- append (no behavior bug reached the diff); no new PR- class (stale-comment law is
standing §14). Post-fix `cargo check --workspace --tests` clean + fmt applied.

## Phase 4 — Validate

**Tests added** (per the Phase 2 plan):
- `display_map.rs` unit module (4 tests): `facade_equals_fold_projection_on_every_method`
  + `facade_viewport_offset_equals_fold_projection_over_grid` (REQ-003 — 8 discriminating
  fixtures incl. nested-merged, adjacent, EOF, header-at-0, ≥3 out-of-order, one-row
  no-op; all slots/rows probed 2 past range; viewport grid incl. degenerate windows) and
  `identity_is_identity_in_range_and_headerless` + `identity_equals_empty_fold_projection`
  (REQ-002 — identity + total-zero contracts; the no-PARSE half is structural:
  `identity()` builds from a row count alone).
- `marley_text_offsets`: `bufferrow_displayrow_api_smoke` (the crate's every-instantiation
  norm — the inspect #5 carry) + `tests/ui/mixed_rows_fail.rs` with pinned `.stderr`
  (REQ-005: assignment / argument / arithmetic mixes all fail to compile; 3×E0308
  snapshotted under the pinned 1.96 toolchain).

**Runs (actual):** `cargo nextest run -p marley display_map` → 4/4 PASS.
`cargo nextest run --workspace` → **2174 run: 2174 passed, 7 skipped** (the 7 = standing
env-conditional skips). `cargo test --workspace --doc` → green (no doctests in the
touched crates). `cargo test -p marley_text_offsets --test compile_fail` → 1/1 PASS
against the committed stderr.

**REQ-004 diff audit:** zero behavioral edits to existing tests. The only existing-test
diffs are one stale-comment rename in `headless_drive.rs:10429` (inspect #2) and the
"neither"→"no" comment fix beside `byteoffset_api_smoke` (inspect #5) — prose only,
no assertion or logic touched; everything else is ADDED tests.

**REQ-001 audit (recorded at inspect, re-verified):** `fold_projection()` call sites = 0;
`FoldProjection` in `marley_app` = the `FoldProjCache` type + the builder inside
`display_map()` + the facade module itself.

**Live-app drive (render path touched — the pixels check):**
`bundle-app.sh debug` → `open target/Marley.app` → AX_TRUSTED → drive (click, type,
enter) → `screencapture -l1573` → READ `scratchpad/425-live-smoke.png`. Seen: the full
shell renders — M31 rail (one selected row + presence rows), Files dock tree, and the
EDITOR pane painting a real file through the new facade: sequential gutter 1..43
(identity map), monospace content, tabs, "focus: editor" status. No blank list (the
PR-137 class disproven live). Hygiene: the center-click landed in the Files dock and
opened a (gitignored) mutants-box log tab; typed text raced the focus switch; ⌘Z ×3 +
quit-without-save applied; `git status` confirms zero stray writes. Parity pair: N/A per
spec (no UI delta).

**Gate:** first run 14/15 — gate:2 clippy RED on ONE lint (`useless_conversion`: the test
hook kept `.into_iter()` after `folded_headers` became an iterator). Source-fixed (the
call removed), re-staged, re-run: **GATE GREEN [diff] — 15/15** (rustfmt, clippy -D
warnings, tests, audit, deny, machete, gitleaks, shellcheck, no-suppressions,
source-bans, docs, coverage ≥100% lines, mutation MSI ≥100%, miri, visual/AX). Receipt
written for /commit. Pre-existing exclusions: none touched; the `block v0.1.6`
future-incompat note is a standing transitive-gpui advisory, not ours.

## Phase 5 — Complete

- **§21 docs:** CHANGELOG `[Unreleased] Added` entry (the facade, the typed spaces, the
  proof story, why-now); `docs/marley_architecture/editor.md` — the stale "needs the
  display-map layer" paragraph now records the shipped facade + where wrap inserts;
  `docs/marley_architecture/roadmap.md` — B-c item 1 marked SHIPPED with the sprint
  pointer. Parity sync: N/A (no UI delta).
- **Ledger appends:** `AD-claude-425-display-map-facade-and-typed-row-spaces-001`
  (architecture-decisions.md); `L-claude-425-type-the-seam-and-the-compiler-finishes-
  your-site-sweep-001` (lessons.md). Inspect appended nothing (no behavior bug — recorded
  in the Phase 3.5 ledger).
- **Ticket:** closed → `tickets/closed/`; BACKLOG swept (the 425 row left at promotion;
  426 now tops the queue).
- **Archive:** pair → `pipeline/completed/`.
