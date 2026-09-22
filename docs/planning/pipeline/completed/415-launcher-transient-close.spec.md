---
pipeline_id: 78b66489-d529-4fba-a4c9-b410f08c38fc
ticket: docs/planning/tickets/open/TICKET-415-launcher-transient-close-gap.md
status: Phase 5 — Complete PASS
title: close_transient_overlays gains the agent launcher; the #318 smoke tightens to one-hot equality
type: bug
milestone: M20
references:
  - docs/planning/pipeline/completed/318-overlay-card-recipe.notes.md
  - docs/planning/pipeline/completed/393-section-plus-menu.notes.md
---

## Title
`close_transient_overlays()` (app.rs:7408) — the one-source-of-truth roster every
mouse-opened modal calls — clears palette/finder/history/find + eleven Some-typed
overlays but NOT `agent_launcher`. Consequence (observed in the #318 draw-smoke):
opening another overlay over a lingering launcher STACKS them — fleet drew over
the launcher — against the PR-claude-mouse-opened-modal-must-close-all-transient-
overlays-001 family. Fix the membership deliberately: launcher IN (it is a
transient centered modal with backdrop + click-away), fleet's toggle-scope
exemption RECORDED in the helper doc, and the #318 smoke tightened to strict
five-flag (one-hot) equality per iteration now that closes are uniform.

## Scope
### In
- `agent_launcher = None` joins the `close_transient_overlays` roster (with the
  #415 provenance comment, matching the roster's per-line idiom).
- The helper's doc comment records the membership rule + fleet's deliberate
  exemption (toggle-scoped dock, not a transient modal).
- `recipe_b_overlays_open_and_draw_headless` (headless_drive.rs:14746) tightens
  from per-index `assert!(flags[i])` to strict one-hot `assert_eq!` per
  iteration, and its "deliberately NOT a five-flag equality" comment is replaced
  by the new invariant's rationale.

### Out (explicitly deferred)
- Fleet joining the close set — deliberately exempt (toggle-scoped by design;
  this spec RECORDS the exemption, it does not revisit it).
- Restructuring the overlay booleans into a single enum (the POC's
  `activeOverlay` model) — a structural refactor beyond this bug's scope;
  recorded as prior-art observation.
- Any chrome/geometry/visual change — none; the #414 chrome sweep is its own
  ticket.

## Reference (§20)
Warp (cockpit UX) — transient overlays are mutually exclusive: opening any
palette-family modal dismisses the others (one-modal-at-a-time), the behavior
class this family has matched since #181/#393 (see
`docs/warp_architecture/subsystems/01-ui-framework-rendering.md` overlay
layering; the launcher itself is Marley-specific cockpit chrome with no Warp
analog, but its dismissal class follows Warp's transient-overlay exclusivity).
Clean-room: behavior-level only; no fork source read.

### Prior art
1. **Behavior maps** — `docs/warp_architecture/subsystems/01-ui-framework-rendering.md`
   (Stack/Overlay/Positioned layering); the one-modal discipline is the
   established family behavior (#181 launcher, #393 section menu). No new
   capture needed — zero visual delta.
2. **Published material** — none applicable (app-local state membership, no
   protocol).
3. **Permissive deps** — checked gpui: it owns paint layering and `.occlude()`
   (mouse blocking), NOT overlay-set membership — that is app-level state with
   no crate owner. No adoption available.
4. **Our own POC** (marley-web, ours to take): `App.tsx:62` holds a single
   `activeOverlay` enum — exclusivity BY CONSTRUCTION (stacking is
   unrepresentable). The Rust boolean/Option roster approximates that enum; the
   enum-wholesale refactor is recorded as out-of-scope prior art, not adopted
   here. Note the POC models its launcher as a fleet EXPANSION
   (`fleetOpen` + `agentLauncherExpanded`, Workspace.tsx:363) — a model
   divergence: Marley's launcher is a standalone transient modal, so membership
   follows Marley's model (launcher in the close set; fleet exempt).

## React-first (parity)
N/A — no UI delta: no chrome, geometry, type, or color changes anywhere. The fix
removes an illegal co-open STATE (a lingering launcher under a newly opened
modal). The POC cannot even represent the bug — its overlays are a single
`activeOverlay` enum and its launcher is a fleet expansion — so there is nothing
to build React-first; parity is unaffected.

## Locked-In Decisions
- D1 — **The launcher JOINS the close set.** It is a transient centered modal
  (backdrop + click-away dismiss at app.rs:21100/21108, Esc arm at 3677) opened
  by mouse from the rail — exactly the class the roster exists for
  (PR-claude-mouse-opened-modal-must-close-all-transient-overlays-001: ONE
  helper, never per-site lists).
- D2 — **Fleet stays EXEMPT, recorded.** `fleet_open` is a toggle-scoped dock
  (persistent surface, `toggle-fleet` closes it), not a transient modal; the
  exemption moves from tribal knowledge into the helper's doc comment. The
  smoke's cleanup keeps its explicit fleet toggle-close.
- D3 — **The smoke tightens to one-hot equality.** With closes uniform, each
  iteration asserts `flags == [i==0, i==1, i==2, i==3, i==4]` — only the fired
  overlay is up — replacing the weaker per-index `assert!(flags[i])` and its
  "deliberately NOT" comment.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `close_transient_overlays()` runs WHILE the agent launcher is open, the system shall clear `agent_launcher` (no launcher rendered on the next frame). | Headless assert (the tightened smoke's cleanup path exercises it every iteration); mutation on the roster is shim-skipped per its existing `mutants::skip`, so the smoke's equality is the kill. |
| REQ-002 | WHEN a Recipe-B open-verb fires after a prior iteration's transient-close, the overlay snapshot shall equal the one-hot expected set (ONLY the fired overlay up: strict five-flag equality). | `recipe_b_overlays_open_and_draw_headless` per-iteration `assert_eq!`; RUN at validate. |
| REQ-003 | WHILE `close_transient_overlays()` runs, the system shall NOT clear `fleet_open` (toggle-scope exemption), and the helper doc shall record the exemption. | The smoke's fleet iteration (fleet stays up through the equality assert until its toggle-close) + review of the doc comment. |

## Phase Plan
- **P2 Design** — lock the roster line's comment idiom + the smoke's exact
  equality shape (expected-array construction, cleanup ordering); confirm no
  other caller depends on the launcher surviving a transient-close.
- **P3 Implement** — the one-line roster add + doc comment + smoke tightening.
- **P3.5 Inspect** — independent critics vs the diff; fix the real findings.
- **P4 Validate** — write + RUN tests; gate green (`--diff`).
- **P5 Complete** — archive, ledger capture (§19), close the ticket (its BACKLOG
  row was removed at promotion).
