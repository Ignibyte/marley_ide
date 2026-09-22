---
pipeline_id: 374be254-32ef-4a52-b5a9-f07376193059
ticket: forge#200 (841e188f-5de7-47a0-93a1-c36647b9b243) · local docs/planning/tickets/open/TICKET-200-warp-ghost-text.md
aar_id: b01def64-0cb9-46f4-a4ee-f1183baf5fec
status: Phase 5 — Complete PASS (all phases PASS; 305 tests, MSI 100; live captures confirm ghost + accept + no-ghost; 2 MED critic fixes applied)
title: Inline history autosuggestion (ghost text) at the prompt
type: feature
milestone: M12.2
references: []
---

## Title
Add fish/Warp-style inline suggestion: as you type at the prompt, a dimmed "ghost" completion drawn from
command history appears after the caret; → accepts it. The passive inline hint (distinct from #183 Tab's
active popup). Composes with #29 history (`recent()` is most-recent-first + de-duped) + #218's block cursor.

## Scope
### In
- **Pure seam** (cov/MSI 100): `suggest(prefix: &str, history: &[&str]) -> Option<String>` — the FIRST
  (most-recent, since `recent()` is newest-first) entry that STRICTLY starts with `prefix` (entry != prefix),
  returning ONLY the remaining suffix. `None` when: prefix empty; no entry starts with prefix; the match
  equals prefix (exact — nothing to add). A pure `accept_suffix` helper (or reuse `suggest`) for the accept.
- **Shim** (app.rs, mutants::skip): (a) RENDER — when the caret is at END-OF-LINE and the tab-complete popup
  is NOT open, paint `suggest(&buffer.text(), &history.recent())`'s suffix as a `muted` span after the caret;
  (b) ACCEPT — `Key::Right` at EOL with a live ghost inserts the suffix (else the normal move).

### Out (explicitly deferred)
- ⌘→ accept (the ticket lists → OR ⌘→; → alone is the core — ⌘→ is a trivial follow-up if wanted).
- Mid-line ghost (only shown at EOL — a mid-line ghost is confusing).
- Fuzzy/substring suggestion (this is strict-PREFIX, the fish/Warp behavior); Tab (#183) covers the fuzzy cycle.
- Suggesting from anything but history (no filesystem/command-name ghost — Tab covers those).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — the ghost shows ONLY at end-of-line (caret == buffer char count). At EOL `after` is empty, so the
  muted suffix sits flush after the block cursor. A mid-line caret shows no ghost.
- **D2** — the ghost is SUPPRESSED while the #96 tab-complete popup is open (`self.completion.is_some()` for
  the pane) — the two don't fight.
- **D3** — display-only until accepted: the ghost is NEVER in the buffer until → inserts it. `suggest` +
  the accept are pure (id/string math); the buffer edit is the shim.
- **D4** — `recent()` is most-recent-first + de-duped → `suggest` takes the FIRST prefix match (no reversal).
- **D5** — auto-approved (/work 195–222): document with the driven ghost + accept capture.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the caret is at end-of-line AND a history entry strictly starts with the typed prefix, the prompt shall paint the remaining suffix as a dimmed ghost after the caret. | Driven capture (type a known prefix → muted ghost) + unit test |
| REQ-002 | WHEN → (Right) is pressed with a live ghost at end-of-line, the prompt shall accept the suggestion (insert the suffix, caret at the new end). | Driven capture (→ → the line completes) |
| REQ-003 | WHEN the prefix is empty, has no matching history entry, or equals a history entry exactly, the prompt shall show NO ghost. | Unit test (`suggest` = None for each) + capture (no ghost) |
| REQ-004 | WHEN the tab-complete popup is open, the prompt shall show NO ghost. | Review (suppressed on `completion.is_some()`) + capture |
| REQ-005 | The pure `suggest` shall return the correct suffix for all boundaries (most-recent wins, strict prefix, empty, no-match, multi-byte). | Unit tests (exact-value, cov/MSI 100) |

## Phase Plan
- **P2 Design** — the exact `suggest` signature + the accept helper; where the caret-at-EOL + completion-open
  checks live at the render + the Right-dispatch; the muted ghost span placement (after `after`); the
  file manifest + test plan.
- **P3 Implement** — the pure `suggest` (+ accept helper) + the render ghost span + the Right-accept dispatch.
- **P3.5 Inspect** — critics: the ghost is display-only (not in the buffer), EOL-only, suppressed under the
  popup; → accept doesn't break a mid-line Right; multi-byte prefix safe; clean-room.
- **P4 Validate** — the pure unit tests + a driven capture (ghost appears; → accepts); gate.
- **P5 Complete** — CHANGELOG + app_shell doc; AAR; close.
