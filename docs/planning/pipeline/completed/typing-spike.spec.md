---
pipeline_id: 119f0cb5-3c45-4989-838d-f0b2d706facf
ticket: forge#172 (67ddc852-e74a-4e7a-ba71-154c4235d54a) · local docs/planning/tickets/open/TICKET-172-typing-spike.md
aar_id: 58a6f6ef-7a03-4ace-80db-311eb19dcd8c
status: Phase 5 — Complete PASS
title: M11 SPIKE — make synthetic typing land (drive.swift type:)
type: chore (spike, scripts-only)
milestone: M11 — Live everywhere + Warp blocks
references:
  - scripts/selftest/drive.swift (typeText)
  - scripts/selftest/README.md (the recipe/blocker note)
---

## Title
Time-boxed spike: chords, bare keys, and clicks all land in gpui; per-char `typeText` does not. Find the
delta and fix `type:`, or document the precise blocker.

## Scope
### In — experiments, in order (stop at first success)
- E1: attach the char via `CGEventKeyboardSetUnicodeString` to the existing per-vk down/up events (gpui
  reads the event's characters; a nil-source CGEvent may carry none).
- E2: a real `CGEventSource(stateID:)` (.hidSystemState / .combinedSessionState) instead of nil.
- E3: `postToPid(marley)` instead of the HID tap.
- E4: delay tuning (the working `key()` uses 15/15ms; typeText 8/30ms).
### Out
- App code (zero `crates/**` changes); IME/dead-key composition beyond plain ASCII.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `focus type:echo spike172 enter` shall render `echo spike172` on the prompt line and execute it (or the blocker shall be documented in README with the failing experiment matrix). | driven capture |
| REQ-002 | The README shall record the working recipe (or the blocker). | doc |
| REQ-003 | gate GREEN (static set — no `.rs` in the diff). | gate |

## Phase Plan
P2 folded (the experiment ladder IS the design). P3 experiments. P3.5 self-review (regression risk to the
WORKING verbs; the keyFor map's reach). P4 the proof capture + gate. P5 docs + close.
