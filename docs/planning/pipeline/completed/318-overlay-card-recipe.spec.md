---
pipeline_id: b025d5d8-98e2-4cfd-bc73-e0513359199e
ticket: docs/planning/tickets/open/TICKET-318-overlay-card-recipe.md
status: Phase 5 — Complete PASS
title: Extract the shared overlay-card recipe (chrome + centered-quarter geometry)
type: chore
milestone: M20
references:
  - docs/planning/pipeline/completed/311-lsp-hover.spec.md
  - docs/planning/pipeline/completed/312-lsp-goto-definition.notes.md
  - /Volumes/Offload/Projects/marley-web/artifacts/marley-ide/src/components/overlays/OverlayShell.tsx
  - /Volumes/Offload/Projects/marley-web/docs/MARLEY-PARITY.md
---

## Title

The overlay card chrome is copy-pasted across the app. Extract it once and
convert the hand-rolled geometry onto the shared clamp, with **zero visual and
zero behavioral delta** — this is a pure refactor whose success condition is
that nothing moves.

## Scope

### In

- **One shared chrome helper** for the verbatim 8-call style core
  (`.occlude().flex().flex_col().bg(surface).rounded(corner_radius)
  .overflow_hidden().border_1().border_color(border)`), applied to the
  ticket's five: `hover_card_overlay`, `def_picker_overlay`, palette, finder,
  history.
- **Recipe B geometry** — the verbatim 13-call "centered-quarter card"
  (`left(w*0.25) / top(h/6) / w(w*0.5)` + the core + `text_color`) converted
  onto **one shared, unit-tested pure geometry helper**
  (`overlay_quarter_geometry` — *design amendment: the plan's "onto
  `menu_origin`'s clamp" DIED at design; Recipe B has no `max_h`, so there is
  no box height to clamp, and the fractional geometry is inside the window by
  construction — the helper's tests prove that property instead*). Its members
  are **palette, finder, history, agent_launcher, fleet** (see D2:
  agent_launcher and fleet are token-identical members of the same recipe;
  converting three of five would leave the exact duplication this ticket
  exists to kill). Both are already `OverlayStates` fields, so the #405
  browser hide-shim contract is satisfied unchanged.
- **The #312 clamp-vs-wrap question, answered and recorded** (D4) — decided,
  not implemented: no selection behavior changes in this pipeline.
- A driven pixel-parity capture per converted overlay proving no visual delta.

### Out (explicitly deferred)

- The other **10 sites** carrying the 8-call core (completion popup,
  references, code_action, file_symbols, symbols, search, problems,
  naming_workflow, naming_pane, fleet_dispatch_draft — *count corrected 12→10
  at design: 17 total minus the 7 converted here*). Named follow-up: the
  helper is proven on seven first; a 17-site sweep is its own capture budget.
- **Migrating off `menu_origin` onto gpui's `anchored()`** — a real prior-art
  hit, rejected here for a recorded reason (D5).
- Any change to selection behavior, row windowing, row metrics, or the `.p_2()
  .gap_1()` padding hover adds beyond the shared core.
- Correcting the React POC's selection drift (D4) — a `marley-web` change,
  tracked separately so this pipeline's parity captures compare like for like.

## Reference (§20)

**N/A as an external app reference — Marley-specific.** This is an internal
DRY refactor of Marley's own already-shipped overlay chrome; there is no Warp
or Zed behavior being matched, and the success condition is explicitly "no
observable change." The visual contract being preserved is Marley's own,
recorded in `marley-web/docs/MARLEY-PARITY.md`.

### Prior art

The sweep paid twice — one adoption confirmed, one decision settled:

1. **Our own React POC owns this seam already.**
   `marley-web/.../overlays/OverlayShell.tsx` is literally titled *"The one
   overlay shape"* and already factors exactly what this ticket proposes:
   shared chrome + `OVERLAY_TOP_FRACTION = 1/6` + `OVERLAY_ROW_H` +
   `OverlayRow` + a shared `useOverlaySelection`. It covers palette, finder,
   history, and project search. It is the parity reference and the shape to
   mirror. **It also exposed a live parity drift** — see D4.
2. **The permissive deps DO own the positioning seam.** gpui 0.2.2 ships
   `anchored()` (`gpui-0.2.2/src/elements/anchored.rs:27`) with
   `.snap_to_window()` (`:68`) and `.snap_to_window_with_margin(edges)`
   (`:74`). Reading its snap block (`:190-205`) confirms it is **semantically
   identical to `menu_origin`** — right-overflow pulls left, left-underflow
   pins to the edge, same on the vertical — plus two capabilities we hand-roll
   nothing for (`AnchoredFitMode::SwitchAnchor` corner-flipping, margin edges).
   `menu_origin` is a reimplementation of a shipped gpui element. Recorded as a
   real find; **not adopted here**, for the testability reason in D5.
3. **Behavior maps / published material:** nothing applicable — no Warp/Zed
   behavior is being matched (internal refactor). `docs/zed_architecture/` has
   no overlay-chrome entry.
4. **`marley_ui_components` has NO card/panel/popover helper** — checked
   `crates/ui_components/src/lib.rs:17-23` (button, dialog, icon,
   keyboard_shortcut, switch, tooltip). The nearest thing, `Dialog::render`
   (`render/dialog.rs:12-33`), shares only 3 of the 8 core calls and carries no
   positioning, so it is not a reusable base. `corner_radius` is consumed 36
   times, **all in `app.rs`** — the abstraction genuinely does not exist yet.

## React-first (parity)

**UI-AFFECTING — Zone A (frozen shell).** Five-to-seven overlay surfaces are
touched. The parity twin already exists:
`marley-web/artifacts/marley-ide/src/components/overlays/OverlayShell.tsx`
(+ `CommandPalette.tsx`, `FileFinder.tsx`, `HistorySearch.tsx`,
`AgentLauncher.tsx`), per the port map in `marley-web/docs/MARLEY-PARITY.md`.

**The React leg here is INVERTED from a normal ticket, and that is the point.**
Nothing new is being designed, so implement does NOT build a new look in React
first. Instead: bring the POC up (`pnpm --filter @workspace/marley-ide run dev`
→ localhost:5173), capture each affected overlay as the **before** reference,
and at validate capture the Rust side at the same state and compare per
`MARLEY-PARITY.md` (**sample pixels, don't eyeball**). Any delta on these
surfaces is a refactor bug, not a design choice — because the POC already
carries the settled look and Marley already ships it.

Note the parity doc's own warning, which bounds the geometry work: *"Every
overlay has its own geometry — do not assume … I guessed 'three families' and
was wrong."* The discovery agrees: hover and def_picker are anchored cards with
fixed widths; the centered-quarter five are fractional. Two recipes, not one.

## Locked-In Decisions

- **D1 — The ticket's proposed helper signature is IMPOSSIBLE as written; design
  picks the replacement.** The ticket specifies
  `fn overlay_card(colors: &ThemeColors) -> gpui::Div`, but the chain it quotes
  (`.font_family(TERMINAL_FONT).text_size(px(TERMINAL_FONT_SIZE))`) **no longer
  exists** — `TERMINAL_FONT_SIZE` was deleted and the live calls are
  `.font_family(self.mono_family()).text_size(px(self.font_size))`
  (`app.rs:2891`, `:384`). A free function over `&ThemeColors` cannot reach
  them. Design settles the shape (a `&self` method on `RootView`, or a free fn
  taking resolved font params) — this plan only records that the ticket's
  signature is stale.
- **D2 — Two recipes, seven overlays.** Recipe A (**anchored card**: hover,
  def_picker) already uses `menu_origin` with fixed `W`/`MAX_H`. Recipe B
  (**centered-quarter**: palette, finder, history, agent_launcher, fleet) is
  the verbatim 13-call fractional card. The chrome helper serves both; the
  geometry conversion applies to Recipe B. Scope expands past the ticket's five
  by exactly agent_launcher + fleet because they are byte-identical members of
  Recipe B — and both are `OverlayStates` fields (`browser.rs:63-80`), so the
  #405 "overlays below the #405 OVERLAY REGION comment must be OverlayStates fields" contract
  holds unchanged.
- **D3 — Pure refactor. Any visual or behavioral delta is a BUG, not a
  judgment call.** Verified by driven pixel-parity captures, not by review.
- **D4 — Clamp vs wrap: the split STAYS, and it is now a decision rather than
  drift** (closing the #312 inspect ledger's open question). Palette, finder,
  and history **clamp** (`palette.rs:104-113`, `finder.rs:43-52`); def_picker
  **wraps** (`editor_nav.rs:83-101`). Keep both, because the split is
  *structurally coupled*, not cosmetic: def_picker wraps AND windows its rows
  around the selection (`app.rs:15354`, `MAX_ROWS = 20`), whereas the
  centered-quarter overlays `.take(20)` from the head with no windowing — so
  making them wrap would move the selection to an **unrendered row**. Wrapping
  everywhere therefore requires windowing three overlays first; that is a
  behavior change and belongs to its own ticket, not to a pure refactor.
  **Parity note:** the POC's `useOverlaySelection` (`OverlayShell.tsx:39-65`)
  **wraps**, so React and Rust genuinely disagree today. Per
  `MARLEY-PARITY.md`, the shell is frozen and pre-existing frozen-shell
  behavior is **Marley-authoritative** → the POC is what gets corrected, in its
  own change (Scope-Out), not Marley.
- **D5 — Keep `menu_origin`; do NOT migrate to gpui `anchored()` in this
  pipeline.** The find is real (Prior art §2) and gpui's version is a superset.
  But `menu_origin` is a **pure, unit-tested function**
  (`context_menu.rs:278-284`, test `menu_origin_clamps` at `:403` with four
  branch assertions) while `anchored()` is a render element living inside
  `mutants::skip` shims. Migrating would move a proven clamp into the
  untestable render layer — trading proof for brevity under a 100%-MSI floor,
  inside a refactor whose entire premise is "change nothing observable."
  Recorded as a named future option, with the trade stated so it is re-decided
  on evidence rather than rediscovered.
- **D6 — Every extraction ships with a direct unit test.** Binding, from
  `BF-lsp-hover-extracted-helper-new-mutation-surface-001`: lifting an inline
  expression out of a coverage-excluded render closure into a **named** fn
  creates a NEW standalone cargo-mutants target that no existing test kills,
  so the MSI floor goes red even though behavior is unchanged. Design enumerates
  the real target set with `cargo mutants --list -f <file>` rather than guessing.
- **D7 — Mutation-posture split (design; refines D6).** COMPUTATIONAL
  extractions (`overlay_quarter_geometry`) take direct unit tests with
  exact-value asserts. RENDER-SHAPE extractions (`overlay_card_chrome`) follow
  the shipped `icon_label` precedent (`app.rs:1034`):
  `#[cfg_attr(test, mutants::skip)] // render-shape only — asserted by driven
  captures`. BF-001's substance is "no NEW un-killed mutation surface"; a
  skipped render-shape fn creates none, and its correctness proof is
  REQ-002's pixel captures.
- **D8 — No `max_h` is added to Recipe B.** Content-driven height (bottom
  overflow allowed) is the settled, Marley-authoritative contract — the POC's
  own doc measures it, while its `maxHeight: 83.3333%` CSS contradicts its
  comment (POC drift #2, corrected POC-side in the follow-up alongside D4's
  wrap drift). Adding a cap here would be a small-window visual delta — a D3
  violation.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE an overlay in the converted set renders its card, the app shall obtain the shared chrome calls from ONE helper rather than a local copy. | Review + grep: the 8-call core appears once in the helper; no converted site repeats it. |
| REQ-002 | WHEN any converted overlay is rendered, its pixels shall be identical to the pre-refactor build at the same window size and state. | Driven capture per overlay, before/after, compared by pixel sampling (not eyeballing). |
| REQ-003 | WHERE a Recipe-B overlay computes its origin, the app shall obtain it from the shared pure helper (`overlay_quarter_geometry`) rather than repeating hand-rolled arithmetic. *(Amended at design: `menu_origin` conversion died — no `max_h` exists to clamp with; see notes.)* | Review: all five Recipe-B sites call the helper; + exact-fraction unit tests at two window sizes. |
| REQ-004 | WHEN the window has any size (including degenerate), a Recipe-B overlay's origin shall satisfy containment by construction: `0 ≤ left`, `left + card_w ≤ window_w`, `0 ≤ top ≤ window_h` (strict below the top for h > 0). *(Amended at design: property-proof replaces edge-clamping — the geometry cannot escape horizontally; height stays content-driven per D8.)* | Unit tests over the helper asserting the property at 0×0, 1×1, and large sizes. |
| REQ-005 | Each newly extracted named function shall be killed by at least one direct unit test (no new surviving mutants). | `scripts/gates.sh --diff` gate:5 MSI 100 + the named tests. |
| REQ-006 | Selection behavior in every converted overlay shall be unchanged — palette/finder/history still clamp, def_picker still wraps. | The existing `palette_selection_clamps_at_both_ends`, `move_clamps`, and `def_picker_moves_and_wraps` tests continue to pass, unmodified. |
| REQ-007 | WHEN the browser pane is mounted and a converted overlay opens, the overlay shall still hide the webview (the #405 shim), i.e. it remains an `OverlayStates` member. | The existing `overlay_flip_table_each_of_26_states` (`browser.rs:268`) passes unmodified. |

## Phase Plan

- **P2 Design** — settle D1's helper signature; the two-recipe factoring
  (chrome vs Recipe-B geometry); the exact per-site call inventory including
  hover's extra `.p_2().gap_1()` and def_picker's missing `.text_color`; the
  `cargo mutants --list` target set per D6; the parity-capture matrix.
- **P3 Implement** — React leg is capture-the-reference (no new design), then
  the Rust extraction + conversion.
- **P3.5 Inspect** — critics on the diff, with "did anything move?" as the
  primary lens.
- **P4 Validate** — unit tests per D6/REQ-005, the parity capture pairs, gate green.
- **P5 Complete** — CHANGELOG + architecture docs, ledger capture, archive; file
  the two named follow-ups (the 10 remaining chrome sites; the POC selection
  correction).
