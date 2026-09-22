---
pipeline_id: d382244a-4fae-400e-8b51-5800effe71fe
ticket: forge#178 (16f5b5df-c67a-46bd-9072-2a0b159d224e) · local docs/planning/tickets/open/TICKET-178-completion-filter.md
aar_id: 4c31d15e-e8be-40a1-96a6-c92e7e5d111d
status: Phase 5 — Complete PASS
title: M11 — the completion popup live-filters as you type
type: feature
milestone: M11 — Live everywhere + Warp blocks
references:
  - crates/marley_app/src/complete.rs (PURE: CompletionState.entries + refilter)
  - crates/marley_app/src/app.rs (SHIM: the popup `_` arm narrows instead of dismissing)
---

## Title
Typing over the open popup NARROWS it in place (zsh-menu feel) instead of dismissing — the char still types
through; it closes only when fewer than 2 candidates remain.

## Scope
### In
- PURE `complete.rs`: `CompletionState.entries: Vec<String>` (the dir-listing SNAPSHOT from open time) +
  `refilter(&mut self, word) -> bool` — recompute `candidates = complete_word(word, &entries)`, clamp
  `selected` into the new list, return `candidates.len() >= 2` (keep-open).
- SHIM: the modal `_` arm — for a PLAIN printable, append to the buffer (type-through stays), recompute
  `word = buffer[start..caret]`, `refilter`; a backspace over the open popup deletes + re-widens; either
  closes the popup when refilter returns false (the #89 inline path finishes a lone match on the next Tab).

### Out
- Re-reading the directory as you type (the snapshot is fixed at open — new files mid-type don't appear);
  fuzzy matching (prefix is #89's contract).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `refilter` shall narrow to `complete_word(word, entries)`, clamp `selected` in range, and return keep-open (≥2) — the clamp + threshold mutation-killed. | unit |
| REQ-002 (visual) | WHEN the popup is open, typing a matching char shall shorten the list (the char lands on the prompt); backspace shall re-widen. | driven (typed — #172) |
| REQ-003 | gate GREEN; the pure surface cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 implement. P3.5 self-review + gate mutation (small pure diff; the deep critic budget went to
#173/#175/#177). P4 tests + driven + gate. P5 docs.
