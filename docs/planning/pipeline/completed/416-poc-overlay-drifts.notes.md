# 416 — POC overlay drifts — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-416-poc-overlay-drift-corrections.md
- **Pipeline spec:** 416-poc-overlay-drifts.spec.md

## Phase 1 — Plan
- **Request:** batch position 3: fix the two #318-recorded POC drifts
  (wrap→clamp; maxHeight cap). Auto-approved batch run.
- **Classification / tier:** chore, marley-web only; single slice.
- **Recall (§18.3):** #318 D4 (clamp-vs-wrap split stays, structurally coupled
  to row windowing; POC gets corrected) + D8 (content-driven height is the
  settled contract; the cap is drift #2, "corrected POC-side in the named
  follow-up, never here"); MARLEY-PARITY.md Zone A rows 04/05/06/19 hold
  Marley authoritative; the component's own doc (OverlayShell.tsx:12) already
  states the overflow contract the CSS violates.
- **Discovery:** hook at OverlayShell.tsx:39-65 (wrap arms :49-54); cap at :84
  (`maxHeight: '83.3333%'`); children container has `overflow-y-auto
  no-scrollbar` (:102) which goes inert once the cap is gone (design call).
  Consumers of the hook = exactly the clamp trio (CommandPalette, FileFinder,
  HistorySearch — grep-verified); CodeActions has its OWN windowed selection
  (`slice(start,end)`, a popup_window analog) — the wrap-keeper class, not a
  hook consumer, untouched by construction. POC checks: `pnpm typecheck` only
  (no unit runner). Vite dev server ALREADY RUNNING at localhost:5173 (pgrep
  evidence earlier this session).
- **Decisions:** D1 exact clamp semantics; D2 cap removed not adjusted; D3 no
  Rust change; D4 typecheck + driven captures. See spec.

## Phase 2 — Design

**Approach.** marley-web only; §20 N/A confirmed (parity correction toward
shipped Marley; no external reference, no fork source). One file.

**Manifest (marley-web half only — there is no Rust half):**
- `artifacts/marley-ide/src/components/overlays/OverlayShell.tsx`:
  1. Hook arms (:49-54): `ArrowUp → setSelected(s => Math.max(0, s - 1))`;
     `ArrowDown → setSelected(s => Math.min(count - 1, s + 1))`. The hook's
     one-line doc (":38 wraps both ways") rewrites to the clamp contract +
     the D4 pointer (wrap belongs only to selection-windowed pickers —
     CodeActions' own hook keeps it).
  2. Style (:84): drop `maxHeight: '83.3333%'` → `style={{ fontSize:
     OVERLAY_TEXT_PX }}`.
  3. Children container (:102): drop the now-inert `overflow-y-auto
     no-scrollbar` (dead once the cap is gone; leaving a scroll class
     invites the next re-cap — the contract is NO internal scroll region),
     keep `flex flex-col`.
  4. No doc-block change needed at :12 — it already states the contract the
     CSS violated.
- Guards untouched: `if (!count) return` (empty-list), the `[count]`
  re-anchor effect.

**Test plan.**

| REQ | Check (RUN at validate) | Assert |
|---|---|---|
| REQ-001 | Playwright at 5173: enter workspace, ⌘⇧P palette → screenshot; ArrowUp → screenshot | row 0 cyan-filled in BOTH (no wrap to last); READ both PNGs |
| REQ-002 | ArrowDown to the last row, extra ArrowDown → screenshot | last row stays filled; READ PNG |
| REQ-003 | `browser_resize` to a SHORT window (e.g. 900×400 → old cap ≈333px ≈12 rows), open the finder/palette with the fixture set | JS evaluate: card `getBoundingClientRect().bottom > innerHeight` AND computed `maxHeight === 'none'`; screenshot shows the card clipped by the window bottom with NO internal scrollbar; READ PNG |
| REQ-004 | `pnpm --filter @workspace/marley-ide run typecheck` | exit 0 |
| parity pair | React captures above ↔ Marley side | live Marley remains environment-blocked (0×0 windows, re-evidenced at #414 validate); the Marley half of the comparison is its unit-pinned shipped behavior (clamp in the palette/finder/history key handlers; content-driven height per #318 D8) — code-vs-code, exact; the pixel side-by-side rides #417 |

**Risks.** None load-bearing. Clamp at count=0 unreachable (guard). The
overflow-class removal could reveal a fixture set short enough that nothing
overflows at full size — the short-window resize in REQ-003 makes overflow
reachable regardless of fixture size.

## Phase 3 — Implement
- **React-first: this ticket IS the React work** (the inverse flow — POC
  corrected toward shipped Marley; no Rust half).
- Built to the manifest, all in `OverlayShell.tsx`: clamp arms
  (`Math.max(0, s-1)` / `Math.min(count-1, s+1)`) + the hook doc rewritten to
  the clamp contract with the D4 pointer; `maxHeight` dropped from the card
  style; the children container's inert `overflow-y-auto no-scrollbar`
  removed with a contract comment in place.
- **Driven verification at 5173 (BEFORE + AFTER, evidence in-transcript):**
  - BEFORE: palette open, mouse parked off-card → ArrowUp at row 0 wrapped to
    index 24/25 (the last row) — the drift demonstrated (DOM query).
  - AFTER (HMR): ArrowUp at row 0 → **stays index 0**; 30 ArrowDowns →
    **pins at 24/25** (wrap would land at 5); computed card
    `maxHeight === 'none'`. Screenshot READ: last row cyan-filled, chrome
    otherwise identical (750px card, top anchor, keycap chips).
  - Capture-protocol gotcha found: with the pointer over the card, Chromium's
    synthesized hover after repaint fires `onMouseEnter` → the row under the
    stationary mouse steals selection right after a keystroke (first BEFORE
    attempt read row 3, confusingly). Park the mouse off-card before
    key-driving overlays — recorded for the ledger at complete.
- `pnpm --filter @workspace/marley-ide run typecheck` → clean.
- Deviations: none. PNGs in the session scratchpad (416-before-*/416-after-*).

## Phase 3.5 — Inspect

Two parallel critics (correctness; simplification+provenance), both verifying
two-sided against the Rust.

**Against the diff: ZERO findings.** The clamp is token-equivalent to Marley's
`palette.rs:105-113` (`saturating_sub` / `min(len-1)`, doc'd "clamping") and
`finder.rs:44-52` (history rides the same `FinderState`); the hook doc's
wrap-belongs-to-windowed claim verified BOTH sides (Marley
`editor_code_action.rs:44-61` wraps + windows; POC CodeActions.tsx:47-52 same,
own hook); nothing ever auto-scrolled the removed region (zero
scrollIntoView/scrollTop in overlays/ — the old cap only enabled manual wheel,
which Marley never had); `no-scrollbar` still used by 5 other components (NOT
dead — left alone, correctly); clamp strictly shrinks the stale-selected race
window and React bails on edge-press re-renders (hover interference no worse,
marginally better); §318 D4/D8 references resolve to decisions saying exactly
what the diff says.

| # | Sev | Finding | Verdict | Action |
|---|---|---|---|---|
| F1 | MINOR pre-existing | Render-cap divergence: Marley finder/history `.take(20)`, POC renders all rows (visible ≥ ~746px height; palette matches exactly on both sides) | REAL, NOT this diff's (the old cap papered over it only at short windows) | Captured in `docs/planning/intake/poc-overlay-parity-nits.md` with the Marley-warts-and-all fix shape |
| F2 | MINOR pre-existing | Enter on empty result set: Marley closes, POC swallows | REAL, pre-existing context line | Same intake doc |
| F3 | MINOR pre-existing | Re-anchor keyed on `[count]`, Marley resets per query EDIT (concrete repro: all-22-match query keeps row 5) | REAL, pre-existing | Same intake doc |
| F4 | LOW (process) | Chromium synthesized-hover steals selection under a parked pointer — poisoned the first BEFORE reading | REAL (protocol, not product) | PR-claude-park-the-pointer-before-key-driving-hover-selecting-overlays-001 appended |

Ledger appends at this phase:
PR-claude-park-the-pointer-before-key-driving-hover-selecting-overlays-001;
intake doc `poc-overlay-parity-nits.md` (three pre-existing divergences).

## Phase 4 — Validate
- **REQ-001/002 (clamp):** driven at implement with in-transcript DOM asserts —
  ArrowUp at 0 stays 0; 30 ArrowDowns pin at 24/25 (wrap would land at 5);
  screenshots READ (scratchpad 416-before-*/416-after-*). BEFORE wrap-to-24
  demonstrated for contrast.
- **REQ-003 (content-driven height):** window resized to 900×400 → palette
  card top 67 (the ⅙ anchor), height 704 (25 rows content-driven), bottom 771
  — **overflows the window bottom by 371px**; computed `maxHeight: none`,
  children `overflowY: visible`, `scrollHeight == clientHeight` (no internal
  scroll region). Screenshot READ (scratchpad
  416-req003-overflow-short-window.png): card runs off the bottom edge, no
  scrollbar. Exactly the Marley contract.
- **REQ-004:** `pnpm --filter @workspace/marley-ide run typecheck` → clean
  (run twice: implement + validate).
- **Rust side sanity:** `cargo nextest run -p marley context_menu` → 12/12
  PASS (no Rust change in this ticket; the run also satisfies the
  tests-ran discipline).
- **Gate:** no `.rs` in this changeset → `scripts/gates.sh --fast` →
  **GATE GREEN [fast], 11 passed 0 failed** (heavy gates correctly skipped;
  the §15 no-.rs arm applies — no receipt needed for the docs-only Marley
  commit; the marley-web repo has its own no-gate convention).
- **Parity pair:** the Marley half remains environment-blocked live (0×0
  windows; unchanged posture) — the behavioral halves were compared
  code-vs-code at inspect (token-equivalent clamp; two-sided windowing
  verification), which for a BEHAVIOR correction is exact rather than
  sampled. The eventual pixel side-by-side rides #417.
- Pre-existing: the three POC divergences in the intake doc (F1-F3) — not in
  scope.

## Phase 5 — Complete
- **CHANGELOG:** Marley CHANGELOG entry (the parity correction record; no `.rs`).
- **Architecture docs:** app_shell.md's drift sentence updated to landed (+ the
  intake pointer for the three new nits).
- **Parity sync:** MARLEY-PARITY.md overlay bullets corrected — the doc itself
  carried the drifted "↑/↓ wrap" description; now states the clamp contract +
  bottom-overflow (the ContextMenu section's wrap line LEFT ALONE — Marley's
  context menu genuinely wraps, `selection_wraps_and_maps`).
- **Ledger appends (at inspect):**
  PR-claude-park-the-pointer-before-key-driving-hover-selecting-overlays-001;
  intake `poc-overlay-parity-nits.md`. No L-/AD- (D4/D8 already own the
  decisions; nothing durable beyond the PR-rule).
- **Ticket:** TICKET-416 → tickets/closed/; backlog clean.
- Archived to docs/planning/pipeline/completed/.
