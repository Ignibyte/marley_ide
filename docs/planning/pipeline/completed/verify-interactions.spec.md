---
pipeline_id: c77bdeb7-0ae5-4040-aa62-47dc6aba074a
ticket: forge#144 (cf5fab0c-3662-40d7-8e3c-44770034a375) · local docs/planning/tickets/open/TICKET-144-verify-interactions.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: live-verify the shipped interactions (+ fix the ⌘⌥-arrow chord) [M8]
type: chore
milestone: M8 — Warp Chrome & Fidelity
references:
  - crates/marley_app/src/keymap.rs (FIX: the 4 focus-nav bindings ⌘⇧→⌘⌥ + their tests)
  - scripts/selftest/drive.swift (drag: + cmdopt: harness additions — done)
---

## Title
Verify the shipped interactions on the live app (now input works) and fix the one that's broken: pane
focus-nav responds to ⌘⇧-arrow instead of the documented ⌘⌥-arrow.

## Scope
### In
- FIX: `keymap.rs` focus-left/right/up/down bindings from `chord(true,false,false,true,…)` (⌘⇧) to
  `chord(true,false,true,false,…)` (⌘⌥); update the asserting tests.
- Harness: `drive.swift` `drag:` + `cmdopt:` (done).
- Verify (drive+capture): close-×, 📁 folder, 🧠 agent, cockpit tabs, #130 drag-resize; fix any also-broken.

### Out
- New features. Re-testing what already passed (+, ⌘D, search) beyond a note.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the intent is ⌘⌥-arrow (comment + ticket #131 both say so; ⌘-arrow is block-jump, ⌥-arrow is word
  motion, so ⌘⌥-arrow is the distinct pane-nav chord). The shipped ⌘⇧-arrow is the bug.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the keymap is queried for ⌘⌥-Right, it shall return `focus-right` (and ⌘⇧-Right shall NOT). | unit |
| REQ-002 (visual) | WHEN ⌘⌥→ is driven with the left pane focused, focus shall move to the right pane. | driven capture |
| REQ-003 | WHEN each remaining interaction (close-×, 📁, 🧠, cockpit tabs, drag) is driven, it shall do its job. | driven captures |
| REQ-004 | gate GREEN, cov/MSI 100 (keymap tests updated). | gate |

## Phase Plan
- **P2** — the keymap binding + test change; the verification checklist.
- **P3** — implement (keymap.rs).
- **P3.5** — 1 self-review: the corrected chord; no other binding collides with ⌘⌥-arrow.
- **P4** — keymap tests + driven captures (⌘⌥→ moves focus; the rest) + gate GREEN.
- **P5** — docs (a verification log), AAR, archive, close #144 → close M8 sprint.
