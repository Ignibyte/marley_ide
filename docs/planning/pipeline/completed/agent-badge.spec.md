---
pipeline_id: c2b716d2-f3d2-4bd5-b284-adfa2ac198ba
ticket: forge#66 (b48ace07-d215-4994-8cdb-da70c75da4c1) · local docs/planning/tickets/open/TICKET-066-agent-badge.md
aar_id: d567948c-1da4-4503-9362-86498d9aa0e2
status: Phase 5 — Complete PASS
title: agent status badge on its pane
type: feature
milestone: M2.C
references:
  - crates/marley_app/src/agent_view.rs (NEW PURE — agent_status_glyph + agent_badge)
  - crates/marley_app/src/app.rs (SHIM: the corner badge on tagged panes)
---

## Title
Surface the invisible `RootView.agents` tag (#62): each agent pane shows a small corner badge with its
kind + a live-ready status glyph. The first M2.C step toward a LIVE cockpit.

## Scope
### In
- PURE (`agent_view.rs`, cov/MSI 100): `agent_status_glyph(AgentStatus) -> &'static str` (Working→●,
  Idle→○, Exited→✓ — DISTINCT from the forge glyphs); `agent_badge(&AgentRun) -> String` =
  `"{launch_command(kind)} {glyph}"` (e.g. `"claude ○"`).
- SHIM (`app.rs`, mutants::skip + cov-excluded): `mod agent_view;`; in the per-pane render loop, a
  top-right corner badge on any pane with a `self.agents` entry, showing `agent_badge(run)`.

### Out
- DRIVING the status live (it's always Idle until #67). The Fleet overlay (#68). Any pane-title change.
  A separate `kind_label` — reuse `launch_command` (the display label = the CLI, per the #62 DRY fix).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 (glyph) — agent glyphs are DISTINCT from forge's: Working→● (filled = active), Idle→○ (hollow),
  Exited→✓. (Forge uses ◐ for in-progress; ● vs ◐ disambiguates the two cockpit surfaces.)
- D2 — reuse `marley_agent::launch_command(kind)` as the badge label (it already yields "claude"/"codex")
  — no duplicate `kind_label` (consistent with #62's label-single-source fix).
- D3 — the badge is an absolutely-positioned corner child in the pane render (the pane is bottom-anchored
  `justify_end`; an absolute badge sits in the top-right regardless).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `agent_status_glyph(s)` is called, it shall return ● for Working, ○ for Idle, ✓ for Exited. | unit |
| REQ-002 | WHEN `agent_badge(run)` is called, it shall be `"{launch_command(kind)} {status_glyph(status)}"` (label before glyph). | unit |
| REQ-003 (visual) | WHEN cmd-shift-a launches an agent, its pane shall show a corner badge with the kind + status glyph. | self-test (cmd-shift-a → capture) |
| REQ-004 | `scripts/gates.sh` GREEN, cov/MSI 100 on agent_view; app shim excluded. | gate |

## Phase Plan
- **P2** — `agent_status_glyph`/`agent_badge` + the corner-badge shim, mutation targets, test plan.
- **P3** — agent_view.rs + the lib.rs mod + the app.rs badge.
- **P3.5** — critic: the glyph arms, the badge format, the render seam + the `self.agents` borrow, mutants.
- **P4** — agent_view unit tests (cov/MSI 100) + the SELF-TEST (cmd-shift-a → the badge) + gate GREEN.
- **P5** — docs, AAR, archive, close #66.
