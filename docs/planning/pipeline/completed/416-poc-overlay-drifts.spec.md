---
pipeline_id: e73e68db-6aed-457f-959f-30b56560f8e9
ticket: docs/planning/tickets/open/TICKET-416-poc-overlay-drift-corrections.md
status: Phase 5 — Complete PASS
title: marley-web POC overlay drifts — wrap→clamp in the shared hook; maxHeight cap removed
type: chore
milestone: M20
references:
  - docs/planning/pipeline/completed/318-overlay-card-recipe.notes.md
  - /Volumes/Offload/Projects/marley-web/docs/MARLEY-PARITY.md
---

## Title
Correct the two POC drifts #318 recorded (frozen-shell behavior is
Marley-authoritative per MARLEY-PARITY.md): (1) `useOverlaySelection`
(OverlayShell.tsx:49-54) WRAPS at the edges while Marley's palette / finder /
history CLAMP — and #318 D4 decided the clamp-vs-wrap split STAYS, structurally
coupled to row windowing (wrap belongs only to selection-windowed pickers);
(2) `maxHeight: 83.3333%` (OverlayShell.tsx:84) caps the card while Marley's
height is content-driven with deliberate bottom overflow — the component's own
doc (line 12: "allowed to overflow the bottom edge") already contradicts the
CSS. Both fixed in marley-web ONLY; Marley is already correct and unchanged.

## Scope
### In
- `useOverlaySelection`: ArrowUp/ArrowDown clamp at the edges (0 and count−1).
  Consumers verified to be exactly Marley's clamp trio: CommandPalette,
  FileFinder, HistorySearch.
- The `maxHeight: '83.3333%'` style removed; the card's height becomes
  content-driven (the shell keeps its documented top anchor + overflow
  contract). Design decides whether the now-inert `overflow-y-auto
  no-scrollbar` classes on the children container go with it.
- Driven React captures at localhost:5173 verifying both corrections.

### Out (explicitly deferred)
- Any Rust change — Marley is the authority here and already ships the
  correct behavior (clamp asserted by its key-handler tests; content-driven
  height is the #318 D8 settled contract).
- The POC's CodeActions windowed selection (its own hook with
  `slice(start,end)` — the wrap-keeper class per D4's structural rule; not a
  `useOverlaySelection` consumer, untouched by construction).
- Any visual restyle of the shell (row metrics, colors, anchor unchanged).

## Reference (§20)
**N/A — Marley-specific parity correction.** The reference is MARLEY ITSELF:
the POC's Zone A frozen shell must match shipped Marley behavior
(MARLEY-PARITY.md; the overlay rows cite screenshots 04/05/06/19 whose
measurements the component doc carries). No Warp/Zed behavior is consulted —
the drifts are POC-vs-Marley disagreements with recorded Marley-authoritative
verdicts (#318 D4 wrap drift, #318 D8 maxHeight drift). Clean-room untouched:
no fork source involved anywhere.

### Prior art
1. **Marley's own shipped behavior** (the authority being matched): the
   palette/finder/history key handlers clamp selection at the list edges
   (their Rust units pin it), and overlay-card height is content-driven with
   deliberate bottom overflow — `overlay_quarter_geometry`'s doc + #318 D8
   ("no box to clamp; nothing to cap"). The POC component's OWN doc comment
   (OverlayShell.tsx:12-14) records the same contract the CSS violates.
2. **The POC itself**: CodeActions.tsx implements the windowed-selection
   class (a `popup_window` analog with `slice(start,end)`) separately from
   the shared hook — confirming D4's structural split already exists
   React-side; the shared hook just picked the wrong arm.
3. **Permissive deps / behavior maps / published**: none applicable — a
   5-line keyboard hook and one CSS declaration; no crate or React library
   owns the seam. (Checked: the POC uses no list/selection library here.)

## React-first (parity)
UI-AFFECTING — **marley-web IS the deliverable** (Zone A frozen shell:
`overlays/OverlayShell.tsx`, consumed by `CommandPalette.tsx`,
`FileFinder.tsx`, `HistorySearch.tsx` — parity rows 04/05/06/19). This ticket
is the inverse of the usual flow: the React side is being corrected TO match
shipped Marley, so there is NO Rust port half — implement edits marley-web,
visually verifies at localhost:5173 (Vite already running), and validate
captures the React side; the Marley half of the parity pair is its
already-shipped, unit-pinned behavior (live Marley captures remain
environment-blocked this session — the #417 battery covers the eventual
side-by-side; the BEHAVIORAL comparison here is code-vs-code, exact).

## Locked-In Decisions
- D1 — **Clamp semantics mirror Marley exactly:** ArrowUp at 0 stays 0;
  ArrowDown at count−1 stays count−1 (no wrap). Escape/Enter arms unchanged.
- D2 — **The cap is REMOVED, not adjusted** — content-driven height with
  bottom overflow is the contract (D8); any percentage cap is the same drift
  smaller.
- D3 — **No Rust change.** Marley authoritative + already correct.
- D4 — **Verification = typecheck + driven captures** (the POC has no unit
  runner): a many-row overlay demonstrates edge-clamp (top row stays selected
  on extra ArrowUp; bottom on extra ArrowDown) and bottom-edge overflow
  (card extends past the window bottom, no internal scrollbar).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the selection is on row 0 and ArrowUp fires, the shared-hook overlays shall keep row 0 selected (no wrap to the last row). | Driven capture at 5173 (palette open → ArrowUp → screenshot: row 0 still filled); code review of the hook. |
| REQ-002 | WHEN the selection is on the last row and ArrowDown fires, the selection shall stay on the last row (no wrap to 0). | Driven capture (navigate to last row → extra ArrowDown → screenshot); code review. |
| REQ-003 | WHILE an overlay's content exceeds 83.3% of the window height, the card shall extend past the bottom edge (content-driven height, no internal max-height scroll region). | Driven capture with a tall result set (finder in a many-file tree): card clipped by the window bottom, no scrollbar; DOM style assert (no maxHeight). |
| REQ-004 | The POC shall stay TypeScript-clean after the change. | `pnpm --filter @workspace/marley-ide run typecheck` exit 0. |

## Phase Plan
- **P2 Design** — lock the exact hook diff + whether the inert overflow
  classes go; the capture script (Playwright steps).
- **P3 Implement** — edit marley-web; hot-reload verify + screenshots READ.
- **P3.5 Inspect** — critic pass over the React diff (behavior parity,
  no collateral consumers).
- **P4 Validate** — typecheck + the driven capture set; (no Rust gate — no
  `.rs` in the changeset; Marley-side pipeline docs commit rides `--fast`
  static discipline per §15's no-.rs arm).
- **P5 Complete** — MARLEY-PARITY.md note if needed; CHANGELOG (marley-web
  has none — the Marley CHANGELOG records the parity correction);
  ledger capture; archive; close ticket; commit BOTH repos (marley-web
  change + Marley pipeline docs).
