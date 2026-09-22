---
pipeline_id: 55776812-e7fb-48f6-94fe-45d07dfe0d48
ticket: docs/planning/tickets/open/TICKET-225-keycap-chip-inset-fill.md
status: Phase 5 — Complete PASS
title: Keycap chips take the inset fill (surface→background) — both sides converge
type: chore
milestone: M12.2
references:
  - docs/planning/pipeline/completed/ui-components-widgets.notes.md
---

## Title
The #222 keycap chips use `bg(colors.surface)` — the same value as the
enclosing palette card, so only the 1px border outlines the chip. The #222
inspect critic computed that `bg(colors.background)` (darker than the card)
reads as a proper inset keycap and lifts chip-text contrast to ~6.6:1, with no
downside on the selected accent row; the change was deferred only because the
phase-gate blocked a post-validate edit. Plan-time finding: the POC chips are
`bg-muted` (L 26% — LIGHTER than its 11% card), a pre-existing third look —
so this ticket converges BOTH sides on the inset fill, React-first.

## Scope
### In
- marley-web: `CommandPalette.tsx` chip class `bg-muted` → `bg-background`
  (the POC's 5% token — darker than the 11% card = the inset), visually
  verified at 5173 BEFORE the Rust edit; screenshot READ.
- Rust: `ui_components/src/render/keyboard_shortcut.rs` — `bg(colors.surface)`
  → `bg(colors.background)` + the doc comment's "a `surface` box" updates.
  (The widgets-gallery bin renders the same component — no separate edit.)

### Out (explicitly deferred)
- Any other chip surface (find-bar `.*`/`Aa` chips, fleet chips) — different
  components, not the #222 keycap.
- Text color/border changes — the muted text + 1px border stay.

## Reference (§20)
Warp (command palette UX) — rows carrying shortcuts render one GREY FILLED
chip per key (observed capture: marley-web reference screenshot `04`, the
measurement the POC's CommandPalette doc records: "one grey chip per key …
keep that grey on the selected (cyan) row"). Marley matches the behavior class
(filled grey keycap chips, kept on the selected row); the exact TONE (inset,
darker-than-card) is Marley's own contrast-driven refinement, computed at the
#222 inspect (~6.6:1) — behavior-level observation only, no fork source.

### Prior art
1. **Behavior maps / observed** — reference shot `04` (grey filled chips; the
   POC transcribed the measurement). The chip-fill TONE was never pixel-locked
   on either side: Marley shipped surface (invisible fill), the POC shipped
   muted (lighter) — a recorded pre-existing divergence this ticket ends.
2. **In-house** — the #222 inspect computation (background inset, ~6.6:1,
   no selected-row downside) is the decision's evidence; the ticket carries it.
3. **Permissive deps / published** — none applicable (a one-token color
   choice; checked gpui: no keycap primitive).

## React-first (parity)
UI-AFFECTING — Zone A (frozen shell, palette rows; parity row `04` →
`overlays/CommandPalette.tsx`). Implement edits the POC chip class FIRST
(`bg-muted` → `bg-background`), hot-reload verifies at localhost:5173 with a
screenshot READ (the look settles in React), THEN ports to Rust 1:1
(`surface` → `background` in keyboard_shortcut.rs — the token map's
background↔background). Validate captures the React side; the live-Marley
half of the pixel pair remains environment-blocked (standing 0×0 evidence
today) and rides #417; the Rust-side change is the same one-token map.

## Locked-In Decisions
- D1 — **Inset direction, both sides:** chip fill = the app background token
  (POC `bg-background`, Marley `colors.background`) — darker than the
  enclosing card on both sides' dark themes; the #222-computed contrast is
  the evidence.
- D2 — **Nothing else moves:** border, radius, padding, muted text, gap stay.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The POC palette chips shall render with the background-token fill (darker than the card), verified visually before any Rust edit. | 5173 screenshot READ + DOM computed-style check (chip bg == root --background). |
| REQ-002 | Marley's `KeyboardShortcut::render` shall fill chips with `colors.background` (doc updated), all other styling unchanged. | Code review; the one-line diff. |
| REQ-003 | The full workspace suite shall stay green (the pure `KeyboardShortcut` seam is untouched; the render is a `mutants::skip` shape). | `cargo nextest run --workspace`; gate `--diff`. |

## Phase Plan
- **P2 Design** — trivial; lock the POC class edit + confirm no other
  `bg-muted` keycap consumers.
- **P3 Implement** — React first (edit, verify, READ), then the Rust token.
- **P3.5 Inspect** — single critic pass (collateral `bg-muted` uses; token-map
  fidelity).
- **P4 Validate** — POC capture + typecheck; suite; gate `--diff`.
- **P5 Complete** — CHANGELOG; MARLEY-PARITY chip line update; archive; close.
