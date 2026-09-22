---
pipeline_id: 4ca7ed30-bdb8-41fd-9345-2c60bf808f4e
ticket: forge#85 (b17286a1-73ef-4055-9fea-8f4e406478bf) · local docs/planning/tickets/open/TICKET-085-remote-badge.md
aar_id: c8533790-e1cf-48ca-94af-2e676f576786
status: Phase 5 — Complete PASS
title: remote-pane badge (⇄ host)
type: feature
milestone: M3.A
references:
  - crates/marley_remote/src/lib.rs (PURE: remote_badge)
  - crates/marley_app/src/app.rs (SHIM: RootView.remotes tag + render + cleanup)
---

## Title
Mark a remote (ssh) pane with a "⇄ {host}" corner badge so it's visibly remote + which host — mirrors the
agent badge (#66).

## Scope
### In
- PURE (`marley_remote`, cov/MSI 100): `remote_badge(host: &str) -> String` = `"⇄ {host}"`.
- SHIM (app.rs, masked): `RootView.remotes: HashMap<PaneId, String>` (host per remote pane); the #84
  open-remote dispatch tags the new pane (capture the `split_focused` pane id → insert); the pane render
  shows `remote_badge` at the top-left on tagged panes; close-pane + the pump auto-close drop the tag.

### Out
- The connection status (#86) / disconnected glyph. The hosts config (#87). Reconnect. A badge for
  non-ssh panes.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `remotes: HashMap<PaneId, String>` (the host), tagged on open (#84) + dropped on close (#67 pattern)
  — no leak.
- D2 — the badge renders top-LEFT (the agent badge is top-right); a pane is agent XOR remote but distinct
  corners are safe.
- D3 — `remote_badge` is pure (marley_remote) — the glyph ⇄ (U+21C4) + the host, tested at cov/MSI 100.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `remote_badge(host)` is called, it shall be `"⇄ {host}"` (U+21C4 glyph). | unit |
| REQ-002 (visual) | WHEN a remote pane is open, its badge "⇄ {host}" shall render on the pane. | self-test |
| REQ-003 | WHEN a remote pane closes, its `remotes` tag shall be dropped (no leak). | review (mirror #67) |
| REQ-004 | `scripts/gates.sh` GREEN, cov/MSI 100 on remote_badge; the shim masked. | gate |

## Phase Plan
- **P2** — remote_badge; the remotes field + open-remote tag + render + close cleanup; mutation targets; test plan.
- **P3** — implement (marley_remote + app.rs).
- **P3.5** — 1 critic: remote_badge MSI; the remotes lifecycle (tag on open, drop on close — no leak); the render.
- **P4** — remote_badge unit test (cov/MSI 100) + the SELF-TEST (cmd-shift-o localhost → the "⇄ localhost"
  badge) + gate GREEN.
- **P5** — docs, AAR, archive, close #85.
