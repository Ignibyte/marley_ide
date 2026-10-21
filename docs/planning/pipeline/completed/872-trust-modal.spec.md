---
pipeline_id: 3f2a0ba6-a29e-4e01-a098-c1f47694a547
ticket: docs/planning/tickets/open/TICKET-872-the-trust-modal.md
status: Phase 4 — Complete PASS
title: Zed's trust modal as the Dialogs board draws it
type: feature
slice: The Marley look, W3.4 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
In the Marley layout Zed's trust modal ("Unrecognized Project") opens as the Dialogs board's trust
card: "Trust <folder>?" over a line on what trust allows, and Not Now (quiet) and Trust (primary)
at the right, with Zed's restricted-mode list and its "Trust all projects in" choice kept between.

## Scope
### In
- A Marley hook in `SecurityModal::render` for the modal's header and footer; Zed's body (the
  restricted-mode list, the scope checkbox and its field), keys and dismissal stay.
- Zed's layout keeps Zed's modal.

### Out (explicitly deferred)
- The first-run screen on the same board (onboarding, its own ticket when planned).

## Reference (§20)
Upstream Zed: `workspace::security_modal` (`SecurityModal`, an `AlertModal` with a header of
the folders, a body of what Restricted Mode prevents and the scope checkbox, and a footer of Stay
in Restricted Mode and Trust and Continue); `ui::AlertModal`, whose `header` and `footer` a
later call replaces.

### Prior art
- The code we ship: `SecurityModal` (its private `trust_and_dismiss` and `trusted`),
  `AlertModal`; #870's `prompts::render_button` and `Role`.
- Ledger: the hook pattern of #834–#871.
- The brain (consultation a9e65840): nothing on this seam.

## UI proof
Scenario `script/e2e/872-trust-modal.sh` (sway): a scratch repository opened in a fresh profile.
- `872-01-night`: the trust card at start: "Trust repo?", the line, Zed's list and checkbox, Not
  Now quiet and Trust primary at the right (REQ-001).
- `872-02-trusted`: Enter: the card gone, the project trusted (its terminal open) (REQ-002).
- `872-03-day`: Marley Day, a second untrusted folder handed over: its card (REQ-001).

## Locked-In Decisions
- D1 — Only the header and footer change; the body stays Zed's, so the scope choice and its field
  keep working unchanged.
- D2 — The title names the folder (`~`-shortened); several folders: "Trust these N projects?" with
  the folders listed under it.
- D3 — Enter still trusts and `ToggleWorktreeSecurity` still stays restricted; Not Now is Zed's
  Stay in Restricted Mode.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley opens an untrusted folder in the Marley layout, it shall show the trust card: "Trust <folder>?", a line on what trust allows, and Not Now and Trust at the right, in Marley Night and Marley Day. | `872-01-night`, `872-03-day` |
| REQ-002 | WHEN the user chooses Trust or presses Enter, Marley shall trust the folder as Zed's modal does. | `872-02-trusted` |

## Phase Plan
- **P1 Plan**, **P2 Code** (the hook, `trust_card.rs`), **P3 Test**, **P4 Complete**.
