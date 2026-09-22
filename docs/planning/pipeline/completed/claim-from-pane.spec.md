---
pipeline_id: b7e14d24-626f-4fea-9803-cf3046ef1679
ticket: forge#75 (bc3d5e08-5079-4eaa-a8a1-e66f5c37ca21) · local docs/planning/tickets/open/TICKET-075-claim-from-pane.md
aar_id: 2f1aa154-357d-4a21-8331-8fd1560258fd
status: Phase 5 — Complete PASS
title: claim a ticket from the Forge pane
type: feature
milestone: M2.D
security_sensitive: true
references:
  - crates/marley_forge_client/src/lib.rs (PURE: TicketView.id)
  - crates/marley_app/src/forge_view.rs (PURE: TicketRow.id + sprint_rows)
  - crates/marley_app/src/app.rs (SHIM: MARLEY_OWNER + claim_forge_ticket + the cmd-click branch)
  - scripts/selftest/drive.swift (HARNESS: cmdclick verb)
---

## Title
cmd+left-click a ticket row in the ⌘⇧F Forge overlay to CLAIM it — the first write-from-the-UI. A
deliberate, unambiguous affordance (distinct from #70's plain-click copy); the claim runs off the UI
thread and the pane re-fetches to show the new owner.

## Scope
### In
- PURE (cov/MSI 100): `TicketView.id: String` (forge_client, deserializes the response "id");
  `TicketRow.id: String` (forge_view) + `sprint_rows` threads `ticket.id`.
- SHIM (app.rs, masked): `MARLEY_OWNER` const; `claim_forge_ticket(id)` — a BG-thread claim + re-fetch via
  the #69 `forge_pending` channel; the forge-row on_mouse_down branches on `event.modifiers.platform`
  (cmd) → claim, else → the #70 copy.
- HARNESS: a `cmdclick:fx,fy` verb in drive.swift (cmd-held left-click) so the affordance is self-testable.

### Out
- A visible "claim" button (chose cmd-click — clean row, unambiguous). Un-claim / release from the UI.
  Claiming a subset. Editing the owner (fixed MARLEY_OWNER). #76's comment (separate ticket).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — cmd+left-click (not a plain click — that's #70 copy; not a button — keeps the row clean); the cmd
  makes it deliberate + unambiguous (the security ask).
- D2 — the claim runs on a BG THREAD then re-fetches (the #72 blocking-IO-off-the-UI PR + the #69
  forge_pending channel + loading state) — a claim's socket round-trip is ~seconds.
- D3 — a fixed `MARLEY_OWNER` UUID (the app's claim identity); only `claim_ticket` (the #74 closed write
  set), localhost + bearer-guarded.
- D4 — the ticket UUID threads TicketView.id → TicketRow.id → the click handler.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `parse_tickets` parses a ticket-list response, each `TicketView.id` shall carry the response "id". | unit |
| REQ-002 | WHEN `sprint_rows` builds rows, each `TicketRow.id` shall equal its `TicketView.id`. | unit |
| REQ-003 (visual/security) | WHEN a ticket row is cmd+left-clicked, that ticket shall be claimed for MARLEY_OWNER (and a plain click still copies). | self-test (cmdclick → raw forge read) |
| REQ-004 (security) | The claim shall only fire on a deliberate cmd-click, hit only the loopback endpoint, and never log the bearer. | review + self-test |
| REQ-005 | `scripts/gates.sh` GREEN, cov/MSI 100 on the pure surface; the app shim masked. | gate |

## Phase Plan
- **P2** — TicketView.id + TicketRow.id/sprint_rows + MARLEY_OWNER/claim_forge_ticket/the cmd-click branch
  + the cmdclick verb, mutation targets, test plan (incl. the existing-fixture id updates).
- **P3** — implement the pure + the shim + the harness verb.
- **P3.5** — 2 critics (correctness + SECURITY): the id threads correctly; the cmd-click can't fire on a
  plain click (no accidental mutation); bg-thread + no bearer leak; MSI 100.
- **P4** — the id parse + sprint_rows tests (cov/MSI 100) + the SELF-TEST (reset mods → ⌘⇧F → cmdclick →
  forge read shows the owner → release) + gate GREEN.
- **P5** — docs, AAR, archive, close #75.
