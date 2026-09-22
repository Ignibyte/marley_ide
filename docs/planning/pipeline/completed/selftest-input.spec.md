---
pipeline_id: 877bb91a-704c-4b77-ae51-1cf39f631364
ticket: forge#140 (bccf4e3a-626f-47ee-898b-6905a7c58abb) · local docs/planning/tickets/open/TICKET-140-selftest-input.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: live synthetic-input self-test (AX preflight + docs) [M8]
type: chore
milestone: M8 — Warp Chrome & Fidelity
references:
  - scripts/selftest/drive.swift (AXIsProcessTrusted preflight + `check` action)
  - scripts/selftest/README.md (document the working recipes + the one-time grant)
---

## Title
Make driven clicks/keys trustworthy — a permission preflight so a missing Accessibility grant fails loudly,
plus docs, now that synthetic input is proven to work (a driven "+" click spawns terminal 2).

## Scope
### In
- `drive.swift`: a `check` action printing `AX_TRUSTED`/`AX_NOT_TRUSTED`; before any event-posting action,
  a loud stderr message + non-zero exit if not trusted (no more silent no-op).
- `README.md`: document that synthetic input works, the click/key recipes, the one-time grant.

### Out
- Any `.rs` change (scripts + docs only). Re-verifying the shipped interactions (that is #144).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the code + permission already work (`AXIsProcessTrusted()==true`; `clickat:0.127,0.018` on the "+"
  spawned terminal 2). This ticket adds a preflight so a *fresh machine without the grant* fails loudly.
- D2 — gate-is-test (§7): no `.rs`, so `--fast` gate + a driven-click smoke (the "+" → terminal 2).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `drive.swift check` runs with Accessibility granted, it shall print `AX_TRUSTED`. | driven smoke |
| REQ-002 | WHEN an event-posting action runs WITHOUT Accessibility, drive.swift shall print a clear error + exit non-zero (not silently no-op). | code-review (can't revoke AX in-session) |
| REQ-003 | WHEN the "+" is driven (`clickat:0.127,0.018`), a second terminal shall appear (proves the harness). | live capture |
| REQ-004 | `--fast` gate GREEN; the README documents the recipes + the grant. | gate + review |

## Phase Plan
- **P2** — the drive.swift preflight + `check`; the README additions; the smoke recipe.
- **P3** — implement (drive.swift + README).
- **P3.5** — 1 self-review: the preflight fails loud; the recipes are accurate.
- **P4** — driven `check`→AX_TRUSTED + a driven "+" click → terminal 2 capture; `--fast` gate GREEN.
- **P5** — docs, AAR, archive, close #140.
