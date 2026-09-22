---
pipeline_id: b618cd02-10d3-4d09-a601-a627a3e31ef2
ticket: docs/planning/tickets/open/TICKET-414-overlay-chrome-remaining-sweep.md
status: Phase 5 — Complete PASS
title: Overlay chrome — convert the remaining 10 sites; add the non-modal (block-mouse-except-scroll) variant
type: chore
milestone: M20
references:
  - docs/planning/pipeline/completed/318-overlay-card-recipe.spec.md
  - docs/planning/pipeline/completed/318-overlay-card-recipe.notes.md
---

## Title
Finish the #318 extraction: the ten overlay sites still hand-writing the 8-call
chrome core convert to the helpers, and the chrome gains its promised non-modal
variant. The Explore map (Phase 1) confirmed every one of the ten carries the
core **token-identical, contiguous, in helper order** — all deviation is prefix
(positioning) or suffix (type/padding) — so conversion is mechanical per site.
Split: **9 true modals** (references, code_action, file_symbols, symbols,
search, problems, naming_workflow, naming_pane, fleet_dispatch_draft) + **1
non-modal-over-scroll** (the #313 editor completion popup), which per
PR-claude-block-mouse-except-scroll-for-nonmodal-scroll-overlays-001 must take
a `block_mouse_except_scroll` variant, never the modal fn.

## Scope
### In
- A non-modal chrome variant (block_mouse_except_scroll in place of occlude;
  exact factoring is design's) + doc updates at the chrome fns.
- Convert the 6 anchored modals (references — preserving its `base` closure
  used twice —, code_action, file_symbols, symbols, search, problems) to
  `overlay_card_chrome` + their existing prefixes/suffixes.
- Convert the completion popup (app.rs:15199-15213) to the non-modal variant +
  its caret-anchored prefix/mono suffix.
- The naming trio (naming_workflow 20994, naming_pane 21043,
  fleet_dispatch_draft 21072 — byte-identical `0.3 / h/4 / 0.4` + `.p_3()`):
  extract as its OWN recipe (geometry = pure fn + exact-value unit per
  L-claude-318-extraction-testing-posture; card = render-shape helper) — the
  ticket's extract-or-leave decided EXTRACT (three byte-identical 15-line
  chains is the textbook case).
- Zero visual delta everywhere except the completion popup's hitbox behavior
  (scroll now reaches the editor beneath — the PR-001-prescribed fix; its
  scrolled-out-of-viewport guard already implements scroll-dismiss).

### Out (explicitly deferred)
- Hover's `.occlude()` — the shipped pre-#318 tension, #318 D3 decided it
  stays; re-deciding it is not this sweep.
- The TERMINAL shell-completion popup (app.rs:21999 — name collision, a
  different overlay with an out-of-order chain and a deliberate
  occlude-blocks-wheel contract).
- The near-miss sites (signature 15310, rename_draft 15362, goto 15399, find
  card 21557, git panel, diff, command bar, context-menu box) — not the 8-call
  core; correctly outside the sweep.
- POC-side drifts — #416 (next in queue).
- Live pixel captures — environment-blocked this session (0×0 off-screen
  windows, no active desk/CRD session; evidence at 415's validate). The
  capture debt is recorded and rides the #417 after-capture battery.

## Reference (§20)
**N/A as an external app reference — Marley-specific.** The continuation of
#318's internal DRY refactor of Marley's own shipped overlay chrome; no Warp or
Zed behavior is being matched, and the success condition is "no observable
change" (single deliberate exception: the completion popup's scroll
pass-through, prescribed by our own recorded rule PR-claude-block-mouse-except-
scroll-for-nonmodal-scroll-overlays-001, which quotes gpui's own docs). The
visual contract preserved is Marley's own (MARLEY-PARITY.md).

### Prior art
1. **Our permissive deps own the variant seam — direct ADOPTION.** gpui 0.2.2
   ships fluent `InteractiveElement::block_mouse_except_scroll`
   (`gpui-0.2.2/src/elements/div.rs:995-1012`, imperative form :572-589) with
   `HitboxBehavior::BlockMouseExceptScroll` (`src/window.rs:596-624`; the
   scroll pass-through arm :783). gpui's doc says it "should be preferred"
   over `occlude` for exactly this case. The variant is a one-call swap on the
   core — nothing to hand-roll.
2. **Our own React POC owns the seam React-side.**
   `marley-web/.../overlays/OverlayShell.tsx` ("the one overlay shape") +
   per-overlay consumers (CodeActions, ProblemsPanel, ProjectSearch,
   ReferencesOverlay, SymbolSearch, NamePaneCard) — the consolidation this
   sweep completes Rust-side already exists there. Its two drifts are #416's
   scope, not this ticket's.
3. **gpui `anchored()`** — the #318 sweep's recorded find (semantically
   `menu_origin`); still NOT adopted, same testability reason (#318 D5).
   Posture unchanged.
4. **Behavior maps / published material** — nothing applicable: internal
   refactor, no external behavior matched.

## React-first (parity)
N/A — no UI delta: a Rust-side DRY refactor; every converted site's rendered
chain is byte-identical by construction (the core is contiguous and
token-identical at all ten sites — Phase 1 Explore verification). The one
behavior delta (completion-popup scroll pass-through) has no POC analog (the
POC's completion popup does not model hitboxes). The POC's own shared
OverlayShell already embodies this consolidation; its drifts are #416.

## Locked-In Decisions
- D1 — **The non-modal variant is a sibling of the modal core, same 7 style
  calls, `block_mouse_except_scroll` in place of `occlude`.** Exact factoring
  (shared private base vs sibling fn) is design's call; the doc comments at
  both fns name the rule and the members.
- D2 — **The naming trio EXTRACTS as its own recipe.** Pure geometry fn
  (`0.3·w / h/4 / 0.4·w`) with exact-value unit tests (computational,
  L-claude-318 posture); an assembled render-shape card helper
  (`mutants::skip`, capture-asserted class) consumed by all three sites.
  Quarter geometry is NOT force-fit (the ticket's recorded obligation).
- D3 — **Verification posture without a live session:** per-site
  chain-identity review at inspect (the #318-proven method — mechanical,
  since the core is contiguous+identical), the existing headless draw smokes
  stay green, a NEGATIVE GREP proves the core chain appears nowhere outside
  the helpers, and the pixel-capture debt is recorded to ride #417. The
  completion popup's hitbox change is asserted by code review + the variant
  fn's doc; live wheel confirmation joins the #417 battery.
- D4 — **The references `base` closure survives conversion** (it builds the
  card twice — searching + results states); the helper call moves inside it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the sweep lands, the 8-call chrome core shall appear in app.rs exactly once — in `overlay_card_chrome` — plus once in the non-modal variant's body if design picks the sibling shape (no verbatim copies at any call site). | Negative grep for the contiguous chain (`occlude`/`block_mouse_except_scroll` head + the 7-call tail) over `crates/marley_app/src/`; review. |
| REQ-002 | Each of the 9 modal sites shall render through the helpers with its site-specific prefix/suffix preserved such that the emitted style chain is call-for-call identical to before. | Per-site chain-identity ledger at inspect (before/after chain listing); full suite green. |
| REQ-003 | The editor completion popup shall use the non-modal variant (`block_mouse_except_scroll`), so wheel events over the popup reach the editor beneath while clicks are still blocked. | Code review vs PR-claude-block-mouse-except-scroll-…-001; the variant fn's rustdoc names the member; live wheel check recorded as #417 ride-along. |
| REQ-004 | The naming-trio geometry shall have a single pure source with exact-value unit tests pinning `left=0.3·w, top=h/4, width=0.4·w`, consumed by all three cards. | `cargo nextest` unit (exact values + containment property); mutation on the pure fn. |
| REQ-005 | WHILE the sweep is a zero-visual-delta refactor, the full workspace suite (incl. the #318/#415 overlay smokes) shall stay green with no test edited to accommodate the change. | `cargo nextest run --workspace` at validate; `git diff` shows no test-expectation edits (test files untouched except where a helper rename would require — none planned). |

## Phase Plan
- **P2 Design** — lock the variant factoring (base vs sibling), the naming-card
  helper signature (what it takes: colors + bounds; what stays caller-side:
  `.p_3()`? — design decides), the conversion order, and the exact negative-
  grep pattern for REQ-001.
- **P3 Implement** — the variant + naming-recipe extraction + 10 conversions,
  compile-checked as they land.
- **P3.5 Inspect** — critics + the per-site chain-identity ledger (REQ-002's
  evidence); provenance check.
- **P4 Validate** — units for REQ-004; negative grep REQ-001; full suite; gate
  `--diff` green.
- **P5 Complete** — CHANGELOG + app_shell.md (the two-recipes AD gains the
  third recipe + the variant); ledger appends; archive; close ticket.
