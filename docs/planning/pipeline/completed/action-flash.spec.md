---
pipeline_id: 381bbb6a-bae0-4f00-88d5-f6e1ba4be668
ticket: forge#77 (b0663d2a-b137-4f38-b01b-0484db040b5e) · local docs/planning/tickets/open/TICKET-077-action-flash.md
aar_id: 69a8654a-9d4b-43bd-958a-6125f9dcc678
status: Phase 5 — Complete PASS
title: action confirmation indicator (flash)
type: feature
milestone: M2.D
references:
  - crates/marley_app/src/flash.rs (PURE, NEW: Flash + FLASH_TICKS)
  - crates/marley_app/src/app.rs (SHIM: status_flash field + pump decrement + render + the 5 action sites)
---

## Title
A transient confirmation flash (a "copied #N / sent / claimed / comment posted" toast) so a cockpit
action gives visible feedback — the M2.D finale. Tick-counted (~2s), fades on its own.

## Scope
### In
- PURE (`flash.rs`, NEW, cov/MSI 100): `FLASH_TICKS` const; `Flash { message, remaining }` with `new(msg)`
  (full countdown) + `tick() -> Option<Flash>` (decrement; None when expired).
- SHIM (app.rs, masked): `RootView.status_flash: Option<Flash>`; the pump decrements it (dirty so it
  repaints + clears); a bottom strip renders the message while Some; the 5 action sites set a Flash.

### Out
- Wall-clock timing (Date::now is banned — count PUMP TICKS). A queue of flashes (one at a time; a newer
  action replaces). Per-action colors/icons (a plain text strip). Dismissing early.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — tick-counted, not wall-clock (Date::now banned) — `FLASH_TICKS = 120` ≈ 2s at the pump's 16ms/tick.
- D2 — the pump (the continuous 16ms timer, #67) decrements the flash + marks dirty so the countdown
  repaints + the strip clears at 0.
- D3 — one flash at a time (a new action overwrites `status_flash`); a plain bottom text strip.
- D4 — the 5 existing actions (#70/#72/#73/#75/#76) set a Flash at their site (masked shim) — no new
  behavior, just feedback.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `Flash::new(msg)` is called, `.message` shall be `msg` and `.remaining` shall be `FLASH_TICKS`. | unit |
| REQ-002 | WHEN `Flash::tick()` is called with `remaining > 1`, it shall return `Some` with `remaining` decremented; with `remaining <= 1` it shall return `None`. | unit |
| REQ-003 (visual) | WHEN a cockpit action fires, a confirmation flash shall appear and then fade on its own. | self-test (action → flash → wait → gone) |
| REQ-004 | `scripts/gates.sh` GREEN, cov/MSI 100 on `flash.rs`; the app shim masked. | gate |

## Phase Plan
- **P2** — `flash.rs` (Flash + FLASH_TICKS); the status_flash field + pump decrement + render strip + the
  5 action sites; mutation targets; test plan.
- **P3** — implement the pure module + the shim wiring.
- **P3.5** — 1 critic (correctness): Flash MSI (new/tick arms); the pump decrement + dirty; the 5 action
  sites don't regress copy/send/broadcast/claim/comment.
- **P4** — the Flash unit tests (cov/MSI 100) + the SELF-TEST (action → flash visible → fades) + gate GREEN.
- **P5** — docs, AAR, archive, close #77 — **M2.D COMPLETE (6/6)**.
