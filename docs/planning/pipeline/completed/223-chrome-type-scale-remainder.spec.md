---
pipeline_id: 7267808b-aaa7-43ee-8355-03690e37305b
ticket: docs/planning/tickets/open/TICKET-223-chrome-text-type-scale.md
status: Phase 5 — Complete PASS
title: Chrome type scale — route the remaining 28 hardcoded text sizes through type_scale roles
type: chore
milestone: M12.2
references:
  - docs/planning/pipeline/completed/typography-polish.notes.md
  - docs/planning/pipeline/completed/chrome-type-scale.spec.md
---

## Title
Finish what the #230 slice started: the CHROME still hardcodes 28
`.text_size(px(N))` literals in app.rs (Phase 1 Explore re-inventory — the
ticket's 2026-07 anchors were stale; the fallback_cell item is already
superseded by #337). Route all 28 through `typography::type_scale` so chrome
text size has ONE source: 6 sites land on existing roles for free (4×11.0 →
`Caption`, 2×12.0 → `Nav`); the other 22 cluster into five value bands that
become five NEW fixed-size chrome roles. Zero pixel change — every value is
preserved exactly; the win is that the next density tune (or a user chrome-size
setting) edits one table.

## Scope
### In
- Five new `Role` arms (names design's; proposed): `Panel` 13.0 (10 sites —
  Details/git/diff/menu containers, hints, seat titles, empty states), `Chip`
  10.0 (6 — fleet chips/meta), `Badge` 9.0 (3 — smallest badges/phase lines),
  `Headline` 15.0 (2 — empty-state + browser-card headlines), `Display` 22.0
  (1 — launcher title). All FIXED-size (chrome does not zoom — #337 binding).
- Route the 28 sites; call shapes follow the three existing idioms (inline /
  hoisted binding / the `caption_header` FONT_SIZE_DEFAULT pattern for the 12
  fleet-rail sites in `&self`-less free fns).
- Extend `type_scale_by_role` (FULL TextStyle per new arm — the
  struct-return-no-Default trap rule) and
  `t337_type_scale_content_zooms_chrome_does_not` (new roles ignore
  font_size).
- `workspace.rs` `fallback_cell` doc line: "falls back to 13.0" → names
  `font_zoom::FONT_SIZE_DEFAULT` (the code already sources the const — #337;
  the bare number in the doc is the last drift-able copy).

### Out (explicitly deferred)
- Any size VALUE change (a density tune is a different ticket; this is
  routing only).
- Content sites (`px(self.font_size)` ×15 etc.) — already correct (#337).
- The `marley_webview_probe` debug bin's two literals (not cockpit chrome).
- Weight routing beyond what the sites already do (no site gains/loses a
  font_weight call).
- A user-facing chrome-size setting (this ticket makes it POSSIBLE).

## Reference (§20)
**N/A — Marley-specific internal consolidation.** No Warp/Zed behavior is
matched; the values are Marley's own shipped chrome sizes (#195/#230/#337
lineage), preserved exactly. The type-scale MODEL (content zooms, chrome
doesn't) was decided at #337 and is not revisited here. Clean-room untouched.

### Prior art
1. **The seam is owned in-house** — `typography.rs` (#195 foundation, #230
   added `Nav` + routed ~20 sites, #337 the content/chrome split). The #230
   slice's spec (completed archive: `chrome-type-scale.spec.md`) is the direct
   precedent; these 28 are what it left behind. The three routed call shapes
   in app.rs are the idioms to follow (inline / hoisted / caption_header's
   FONT_SIZE_DEFAULT-passing free-fn pattern with its recorded honesty
   argument).
2. **Permissive deps** — checked gpui 0.2.2: no role/type-scale system (its
   TextStyleRefinement is per-element; `text_size` is a raw field write).
   Nothing to adopt.
3. **The POC** — carries the same bands as constants/utilities
   (`OVERLAY_TEXT_PX = 16`, the `marley-nav` 12px class); confirms the band
   model, nothing to port (React side unchanged — no visible delta).
4. **Behavior maps / published** — N/A (internal refactor).

## React-first (parity)
N/A — no UI delta: value-identical routing (every literal keeps its exact
size; the diff moves WHERE the number lives, not what renders). Nothing to
build or verify React-side.

## Locked-In Decisions
- D1 — **Reuse-before-mint:** 11.0 → existing `Caption`, 12.0 → existing
  `Nav` (6 sites free). New roles only for bands with no owner (13/10/9/15/22).
- D2 — **All new roles are FIXED chrome** (ignore `font_size`) — the #337
  content/chrome split is binding; the extended #337 test asserts it for
  every new arm.
- D3 — **Full-value pins per new arm** (size AND weight) in
  `type_scale_by_role` — PR-…-struct-return trap: MSI cannot enforce the
  table's data; exact-value regression asserts are the guard.
- D4 — **Zero-delta:** every site keeps its exact current pixel value; the
  diff is mechanical substitution. Weights: new roles carry
  `TextWeight::Normal` (no routed site sets a weight through the literal
  today; sites with their own font_weight calls keep them caller-side).
- D5 — **The fallback_cell code change is DEAD** (superseded by #337 — the
  guard already sources FONT_SIZE_DEFAULT); only the doc's bare "13.0"
  updates to name the const. Recorded as a prior-art-style win: the substrate
  already did it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the sweep lands, app.rs shall contain ZERO `.text_size(px(<numeric literal>))` calls (every text size reads `type_scale`, `self.font_size`/param, or a binding thereof). | Negative grep (regex for `text_size\(px\([0-9]` over marley_app/src) = 0; run BEFORE (28) and AFTER (0). |
| REQ-002 | `type_scale` shall return the exact preserved values for the five new roles (13.0/10.0/9.0/15.0/22.0, weight Normal) — full-value asserts per arm. | Extended `type_scale_by_role`; mutation + the exact-value pins. |
| REQ-003 | WHILE `font_size` varies (8/20/32/NaN), the five new roles shall return their fixed sizes unchanged (chrome does not zoom). | Extended `t337_…_content_zooms_chrome_does_not`. |
| REQ-004 | The routing shall be pixel-neutral: the full workspace suite passes with no test edits, and the value mapping site→role is reviewed 1:1 at inspect (28-row ledger). | `cargo nextest run --workspace`; inspect ledger. |
| REQ-005 | `fallback_cell`'s doc shall name `FONT_SIZE_DEFAULT` instead of a bare 13.0. | Review; rustdoc gate. |

## Phase Plan
- **P2 Design** — lock role names + the per-site route table (28 rows:
  line/literal/role/call-shape) + where bindings hoist.
- **P3 Implement** — typography.rs arms + the 28 routes + the doc line;
  fmt + check as it lands.
- **P3.5 Inspect** — critics verify the 1:1 value mapping (the zero-delta
  claim) + no missed literal + idiom fit.
- **P4 Validate** — extended tests RUN; negative grep; full suite; gate
  `--diff` green.
- **P5 Complete** — CHANGELOG + architecture doc (typography section);
  ledger; archive; close.
