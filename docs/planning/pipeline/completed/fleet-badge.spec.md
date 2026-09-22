---
pipeline_id: e99f3f31-61ac-429d-b09d-7ccfac33f371
ticket: forge#188 (369dae42-9dee-44e8-b34b-8b1600780139) · local docs/planning/tickets/open/TICKET-188-fleet-badge.md
aar_id: e56d2026-2e72-4983-a1d5-c7e680f1a1d9
status: Phase 5 — Complete PASS
title: M12 — a live fleet badge: top-bar 🤖 running-count + click → Agents tab
type: feature
milestone: M12 — The Agent Cockpit
references:
  - crates/marley_app/src/agent_view.rs (PURE: fleet_badge(agents) -> Option<String>)
  - crates/marley_app/src/app.rs (SHIM: badge on the Agents cockpit-tab icon; footer agents segment click → Agents)
---

## Title
Glance + one click to the observe surface: the top-bar Agents cockpit icon shows a small running-count badge
(hidden at zero), and the footer "n agents · n working" segment — like the icon already does — clicks to open
the Agents cockpit tab.

## Scope
### In
- PURE `agent_view::fleet_badge(agents) -> Option<String>` — `None` when zero agents are RUNNING (Working or
  Waiting, the #167 aggregate's live states; Idle + Exited don't count), else the running count as a string.
- SHIM `app.rs`: overlay `fleet_badge(&self.agents)` on the Agents cockpit-tab icon (top-right tab strip) when
  Some; make the footer's agents segment (index 1 of `cockpit_status`) a clickable div that opens the Agents
  cockpit (right_section = Agents + open_or_switch_cockpit + persist), matching the existing tab-click handler.

### Out
- A badge on the other cockpit tabs; animating the count; a badge on the ⌘⇧E overlay (retired-ish) or the
  🧠 launch icon. Changing `cockpit_status`'s pure shape (still Vec<String>; the shim decides which segment clicks).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `fleet_badge` shall return `None` at zero running and `Some(count)` of Working+Waiting agents otherwise (Idle/Exited excluded). | unit + mutation |
| REQ-002 (visual) | WHEN an agent is running, the top-bar Agents icon shall show its count badge; clicking the icon OR the footer agents segment shall open the Agents cockpit tab. | driven capture |
| REQ-003 | gate GREEN; the pure fn cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 the pure fn + the two shim wirings. P3.5 1 critic (the running-state set vs #167; the footer
enumerate index; the badge overlay layout / no click-steal; reuse of the tab-click handler). P4 unit + driven +
gate. P5 docs.
