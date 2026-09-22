---
pipeline_id: 6093a55b-8190-44d5-834f-f97d1c083533
ticket: forge#86 (ff987d71-bfe9-4117-bb2d-b2ed83862693) · local docs/planning/tickets/open/TICKET-086-remote-status.md
aar_id: 07855e62-39db-494a-a810-02fa6fc696ed
status: Phase 5 — Complete PASS
title: remote connection status (connected / disconnected)
type: feature
milestone: M3.A
references:
  - crates/marley_remote/src/lib.rs (PURE: RemoteStatus, remote_status_from, remote_status_glyph, remote_badge+status)
  - crates/marley_app/src/app.rs (SHIM: Remote{host,status}; the pump disconnect branch)
---

## Title
When a remote pane's ssh exits (network drop, remote logout), mark it DISCONNECTED and keep it visible —
a visibly-dead remote beats a silently-vanishing pane. The badge flips ⇄ → ✗.

## Scope
### In
- PURE (`marley_remote`, cov/MSI 100): `RemoteStatus { Connected, Disconnected }`; `remote_status_from(
  exited: bool)`; `remote_status_glyph(status)` (⇄ / ✗); UPDATE `remote_badge(host, status)` → "{glyph} host".
- SHIM (app.rs, masked): `RootView.remotes` value `String` → `Remote { host, status }`; open-remote inserts
  Connected; the badge render passes the status; the #67 pump auto-close loop BRANCHES remote panes — mark
  Disconnected + flash (once, on the transition) + KEEP the pane (never auto-close a remote).

### Out
- Reconnect / re-open (#87 + later). A "connecting" state (open = Connected optimistically). Distinguishing
  a clean logout from a network drop (both → Disconnected). Non-ssh panes.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — a remote pane is NEVER auto-closed on exit — it flips to Disconnected + stays (mirrors the #67
  last-pane rule: a visibly-dead pane beats a vanishing one). Non-remote panes still auto-close (#67).
- D2 — the flash + repaint fire ONLY on the Connected→Disconnected transition (the dead session keeps
  erroring on pump each frame; without the guard that would be a per-frame flash + repaint busy-loop).
- D3 — `remotes` value becomes `Remote { host: String, status: RemoteStatus }`; `remote_badge` takes the
  status so the glyph reflects it (Connected still renders "⇄ host" — #85 unchanged).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `remote_status_from(exited)` is called, it shall map true→Disconnected, false→Connected. | unit |
| REQ-002 | WHEN `remote_status_glyph(status)` is called, it shall be ⇄ (Connected) / ✗ (Disconnected). | unit |
| REQ-003 | WHEN `remote_badge(host, status)` is called, it shall be "{glyph} {host}". | unit |
| REQ-004 (visual) | WHEN a remote pane's ssh exits, the pane shall STAY, its badge flip to "✗ {host}", + a "disconnected" flash. | self-test |
| REQ-005 | `scripts/gates.sh` GREEN, cov/MSI 100 on the pure surface; the shim masked. | gate |

## Phase Plan
- **P2** — RemoteStatus + from + glyph + remote_badge(+status); the Remote struct + the pump branch;
  mutation targets; test plan.
- **P3** — implement (marley_remote + app.rs).
- **P3.5** — 1 critic: the pure MSI; the pump branch (Disconnected + keep, never close; transition-gated
  flash/dirty — no per-frame loop; the last-pane rule holds; the get_mut + status_flash borrow).
- **P4** — the pure tests (cov/MSI 100) + the SELF-TEST (open ssh localhost → `exit` → the pane stays,
  ✗ badge, disconnected flash) + gate GREEN.
- **P5** — docs, AAR, archive, close #86.
