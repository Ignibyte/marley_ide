---
pipeline_id: 0640a324-e071-4d1f-bf17-83a1f16bef43
ticket: forge#69 (2baadc57-69db-4cdd-a190-0cdd2dc22b16) · local docs/planning/tickets/open/TICKET-069-forge-refresh.md
aar_id: 654f86a6-ad26-4abb-9194-56589cc5b6f1
status: Phase 5 — Complete PASS
title: Forge pane refresh + honest states
type: feature
milestone: M2.C
references:
  - crates/marley_app/src/forge_view.rs (PURE: forge_status_line)
  - crates/marley_app/src/app.rs (SHIM: store ForgeClient + re-fetch on cmd-shift-f open)
---

## Title
The ⌘⇧F Forge pane re-fetches the sprint each time it's OPENED (was frozen at startup) + shows an honest
header/placeholder — so a forge that starts after Marley, or a sprint that changed, now appears.

## Scope
### In
- PURE (`forge_view.rs`, cov/MSI 100): `forge_status_line(sprint: Option<&SprintView>) -> String` — the
  consistent header ("{name} (#{n}) · {N} tickets") or the "forge unreachable — is the sidecar running?"
  placeholder.
- SHIM (`app.rs`, mutants::skip + cov-excluded): store `forge_client: Option<ForgeClient>` in RootView
  (built in new()); the `toggle-forge` dispatch RE-FETCHES on OPEN (sync, bounded by the #63 socket
  timeout); the overlay header uses `forge_status_line` + renders rows only when Some.

### Out
- A background-thread refresh (a later refinement if the sync fetch feels laggy — noted). Auto-refresh on
  an interval. A "last read at …" timestamp (Date::now is banned in this env). Writes.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — SYNC re-fetch on OPEN, bounded by the #63 5s socket timeout (localhost is fast). Bg-thread deferred.
- D2 — store the `ForgeClient` in RootView (holds the endpoint+bearer, already in memory; the #63
  redacting Debug still guards it; never logged).
- D3 — extract the header/placeholder into the pure `forge_status_line` (cov/MSI 100) so the overlay's
  header is testable + consistent.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `forge_status_line(Some(view))`, it shall be "{name} (#{n}) · {N} tickets"; `None` → "forge unreachable — is the sidecar running?". | unit |
| REQ-002 (visual) | WHEN ⌘⇧F is opened, the sprint shall be re-fetched (fresh) and the header shall reflect it. | self-test (⌘⇧F → the sprint) |
| REQ-003 | `scripts/gates.sh` GREEN, cov/MSI 100 on forge_status_line; app shim excluded. | gate |

## Phase Plan
- **P2** — `forge_status_line` + the store-client/re-fetch/overlay shim, mutation targets, test plan.
- **P3** — forge_status_line + the app.rs refactor (store client, re-fetch on open, overlay header).
- **P3.5** — critic: the header format, the re-fetch-on-open logic, the stored-client security, the overlay seam.
- **P4** — forge_status_line unit tests (cov/MSI 100) + the SELF-TEST (⌘⇧F → the re-fetched sprint) + gate GREEN.
- **P5** — docs, AAR, archive, close #69.
