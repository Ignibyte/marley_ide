---
pipeline_id: ab85efc9-125f-4eff-a6b5-2a7db70e324d
ticket: forge#89 (c684deaf-5779-4e6c-a53a-023e49778ed3) · local docs/planning/tickets/open/TICKET-089-tab-complete.md
aar_id: 1a8bb125-6624-430b-96eb-7df90d237992
status: Phase 5 — Complete PASS
title: tab completion at the prompt (Marley-local engine)
type: feature
milestone: Terminal Polish
references:
  - crates/marley_app/src/complete.rs (NEW — the pure completion engine)
  - crates/marley_app/src/app.rs (SHIM: the Tab intercept in the cooked-prompt dispatch)
---

## Title
Tab at the Marley prompt completes a path against the cwd (Marley-local engine) instead of inserting a
literal tab — resolves the #51/#33 fork as Option A. #89 = the engine + inline completion; the
multi-candidate popup is a follow-up.

## Scope
### In
- NEW `crates/marley_app/src/complete.rs` (cov/MSI 100): `current_word(line, caret)`, `complete_word(word,
  entries)`, `common_prefix(cands)`.
- SHIM (app.rs): intercept Tab at the COOKED prompt (only when NOT is_command_running) → current_word →
  read the cwd dir → complete_word → single: replace the word (+ trailing space for a non-dir); many:
  extend to common_prefix if longer; none: no-op. NEVER a literal tab.

### Out
- The multi-candidate POPUP overlay (a NEW follow-up ticket — created at Phase 5). Command-name completion
  from $PATH (first word). Real zsh completions (git subcommands/flags — the Option-B alternative). Tab
  while a command runs (still streams to the program, #40).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — Marley-LOCAL engine (Option A of the #51/#33 fork; chad AFK at the ask, Claude recommendation — may
  flip to shell-ZLE later). Fits the cooked-buffer prompt model.
- D2 — #89 SPLITS: the engine + INLINE completion (single→replace, many→extend-to-common-prefix); the
  multi-candidate POPUP is deferred to a follow-up (keeps #89 shippable + testable).
- D3 — Tab is intercepted ONLY at the cooked prompt (`!is_command_running`) — while a command runs, Tab
  still streams to the program (#40). No literal-tab fallthrough at the prompt.
- D4 — the pure engine (complete.rs) is cov/MSI 100; the Tab handler + the dir read are the masked shim.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `current_word(line, caret)` runs, it shall return the whitespace-token ending at the caret (start index + word; empty at a space / caret 0). | unit |
| REQ-002 | WHEN `complete_word(word, entries)` runs, it shall return the entries whose last-'/' segment starts with the word's segment, dir-prefixed + sorted. | unit |
| REQ-003 | WHEN `common_prefix(cands)` runs, it shall return the longest shared prefix (full for one, "" for none/empty). | unit |
| REQ-004 (visual) | WHEN Tab is pressed at the prompt with one match, the word shall be replaced by the completion (e.g. `cd cra`+Tab → `cd crates/`). | self-test (may be env-blocked) |
| REQ-005 | `scripts/gates.sh` GREEN, cov/MSI 100 on complete.rs; the shim masked. | gate |

## Phase Plan
- **P2** — complete.rs signatures + edge cases; the Tab-intercept wiring (cwd resolution, dir read, decide);
  mutation targets; test plan.
- **P3** — implement (complete.rs + the app.rs Tab handler + `mod complete`).
- **P3.5** — 1 critic: the engine MSI (word back-scan, common-prefix reducer, dir-prefix remap, sort); the
  Tab intercept is is_command_running-guarded (no raw-mode break) + no literal-tab fallthrough.
- **P4** — the engine tests (cov/MSI 100) + the self-test (`cd cra`+Tab → crates/; env-blocked → engine
  tests + shim read + capture) + gate GREEN.
- **P5** — docs, AAR, **create the popup follow-up ticket**, archive, close #89.
