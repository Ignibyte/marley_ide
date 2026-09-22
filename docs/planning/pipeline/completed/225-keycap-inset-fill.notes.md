# 225 — keycap inset fill — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-225-keycap-chip-inset-fill.md
- **Pipeline spec:** 225-keycap-inset-fill.spec.md

## Phase 1 — Plan
- **Request:** batch position 5: the #222-deferred one-token chip polish.
- **Classification / tier:** chore, single slice, React-first.
- **Recall (§18.3):** #222 (ui-components-widgets archive) shipped surface
  chips capture-confirmed legible; its inspect critic computed the background
  inset (~6.6:1, no selected-row downside); deferral reason was purely the
  phase-gate. Render at `ui_components/src/render/keyboard_shortcut.rs:18`
  (`mutants::skip` render-shape; the pure `KeyboardShortcut` seam untouched).
- **Plan-time finding:** the POC chip is `bg-muted` (dark-theme L 26%,
  LIGHTER than its 11% card) — a pre-existing THIRD look neither side ever
  reconciled (Marley: invisible surface fill). This ticket converges both on
  the inset (background token: POC 5% / Marley dark ~5% under the ~11%
  surface). Recorded in the spec's prior art.
- **Decisions:** D1 inset both sides; D2 nothing else moves.

## Phase 2 — Design
- Trivial (locked in plan): POC `CommandPalette.tsx` `bg-muted`→`bg-background`
  on the kbd chip span; Rust `bg(colors.surface)`→`bg(colors.background)` +
  doc word. **Confirmed:** grep shows exactly ONE `bg-muted` keycap consumer
  (CommandPalette.tsx:252) — the find-bar `.*`/`Aa` chips are a different
  component, out of scope. Test plan = the spec's REQ table (no new units —
  the pure seam is untouched; the render is a skip-shape; the oracle is the
  POC capture + the one-line review).

## Phase 3 — Implement
- **React FIRST:** CommandPalette.tsx chip class `bg-muted`→`bg-background`;
  verified at 5173: DOM computed check — 12 chips at rgb(11,12,15) == the
  root background token, on the rgb(26,27,31) card (darker = inset);
  screenshot READ (scratchpad 225-react-inset-chips.png): dark inset pills,
  dim text legible, and on the selected cyan row the dark chips gain contrast
  exactly as the #222 critic computed. Look settled before Rust.
- **Rust port (1:1 token map):** keyboard_shortcut.rs `bg(colors.surface)` →
  `bg(colors.background)` + the doc gained the #225 line (inset rationale +
  React-approved note).
- `cargo check` 0 errors; fmt clean; POC typecheck clean.
- Deviations: none.

## Phase 3.5 — Inspect

One critic (sized to the two-file diff), full consumer + both-theme sweep.

| # | Sev | Finding | Verdict | Action |
|---|---|---|---|---|
| F1 | MINOR | The widgets-gallery shortcut fixture now renders background-on-background (fill invisible at the gallery root) — the exact class #225 fixes, relocated | REAL | Fixed — the fixture wraps the chip in a `surface` card (mirrors the real palette-row context, which is what the gallery is for). |
| F2 | MINOR | The new doc stated dark-theme facts unconditionally (light theme flips to lighter/raised, ~4.8:1) | REAL | Fixed — doc qualifies per theme + records the POC flipping identically + the surface-parent consumer rule. |

Clean lenses: consumer sweep (exactly 2 render sites — palette rows on
`surface` ✓, gallery fixed), light-theme parity (tokens numerically aligned
both sides: 97/93 light, 5/11 dark; relative direction identical per theme —
**and the change FIXES a real AA failure: pre-change POC light chips were
3.13:1 FAIL, now 4.58:1; Marley light 4.4→4.8** — the critic computed all
eight combinations), the pure #222 seam untouched, no other POC kbd renderer
(the shadcn `ui/kbd.tsx` stock is imported nowhere — dead), selected-row
contrast rises everywhere. Info: chip-vs-card fill delta is subtle by design
(the same bg-vs-surface delta as the pane/dock split #194/#231; the border
carries the boundary).

## Phase 4 — Validate
- REQ-001: the React capture + DOM computed check landed at implement
  (in-transcript: 12 chips at the root background token on the card;
  screenshot READ). REQ-002: the one-line diff + doc reviewed at inspect.
- `cargo nextest run --workspace` → **2148 passed, 5 skipped, 0 failed**;
  POC typecheck clean. Gate: **GATE GREEN [diff], 15 passed 0 failed**,
  receipt written.
- Live-Marley pixel half: standing environment block (0×0 windows, this
  session); rides #417. The React-approved capture + the 1:1 token map is the
  shipped evidence (the React-first contract's intended shape).

## Phase 5 — Complete
- CHANGELOG entry (incl. the AA finding: pre-change POC light chips 3.13:1
  FAIL → 4.58:1; Marley light 4.4→4.8). MARLEY-PARITY palette bullet notes
  the chip fill token. No arch-doc delta beyond the component doc itself
  (ui_components has no separate arch page section for the chip). No ledger
  appends (the decision + evidence live in the ticket/spec; nothing durable
  beyond them — the both-themes flip note is in the component doc).
- Ticket → closed/; backlog clean; archived.
