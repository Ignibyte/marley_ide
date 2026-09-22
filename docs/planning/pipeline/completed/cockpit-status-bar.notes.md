# the cockpit status bar — Notes

- **Forge ticket:** #94 `111a4867-649d-462f-86b5-f84b6b3e04c6` · **AAR:** `58ee5207-3dc8-47d5-a184-8db01dd3f58b`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-094-cockpit-status-bar.md

## Phase 1 — Plan
- **Request:** forge #94 (M2.F 5/6) — a bottom status strip (sprint · agents · focus).
- **Pre-flight:** `SprintView { sprint: SprintMeta{id,name,number}, tickets: Vec<TicketView> }`; AgentRun.
  status (count Working); the root render uses absolute-positioned children (the flash is `.absolute()
  .bottom(px(48))`) — the footer is an absolute bottom(0) full-width strip near the render end.
- **Decisions:** D1 agent_summary pluralize+working-count; D2 cockpit_status order sprint/agents/focus; D3
  focused_label = agent label / remote host / "terminal".
- **AAR id:** `58ee5207-3dc8-47d5-a184-8db01dd3f58b`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **status_bar.rs (NEW PURE):** agent_summary (empty→"no agents"; else "{n} agent[s] · {m} working", pluralize n==1, m=Working count) + sprint_summary (Some→"{name} · {n} tickets"/None→"no sprint") + cockpit_status(sprint,agents,focused) → [sprint_summary, agent_summary, "focus: {focused}"].
- **lib.rs:** `mod status_bar;` (after settings/shell_integration — alphabetical spot).
- **app.rs SHIM:** `const STATUS_BAR_H: f32 = 22.0;`; after `bounds` add `let content_h = (bounds.h - STATUS_BAR_H).max(0.0);`; replace the 3 LAYOUT bounds.h uses (left dock @1654, right dock @1795, center_bounds.h @1804) with content_h so the panes/docks leave the bottom strip; a footer div (absolute, bottom(0), left/right 0, h STATUS_BAR_H, flex row gap_3 px_3, bg surface, border_t_1 border, text muted size 12) renders cockpit_status(self.forge_sprint.as_ref(), &self.agents, &focus_label) segments; focus_label = agents.get(focused).label / remotes.get(focused).host / "terminal". The centered OVERLAYS keep bounds.h (full-height centering).
- **Mutation targets:** the 0/pluralization/working-count (agent_summary), Some/None (sprint_summary), the 3-segment order (cockpit_status).
- **Test plan:** agent_summary_counts_and_pluralizes (0/1-working/2-with-1); sprint_summary_present_and_absent (None + Some[2 tickets]); cockpit_status_segments_in_order. cov/MSI 100. The footer render + focus_label masked (static live).
- **Risks:** reserving content_h shrinks the pane area by 22px (the prompt stays visible above the footer); the footer is the FIRST always-on chrome outside a pane.

## Phase 3 — Implement
- **Built:** status_bar.rs (agent_summary/sprint_summary/cockpit_status); `mod status_bar` (lib.rs); app.rs `STATUS_BAR_H=22` + `content_h = bounds.h - STATUS_BAR_H` replacing the 3 layout heights (left/right docks + center_bounds) + a footer strip (absolute bottom(0), full-width, surface+muted+border_t) rendering cockpit_status(forge_sprint, agents, focus_label); focus_label = agent label / remote host / "terminal".
- **Verification:** fmt; check --all-targets 0 err; clippy OK; the 3 summary tests pass.

## Phase 3.5 — Inspect
- **Method:** self-review (3 small pure summaries + a masked footer render + a bounded layout reserve).
- **Lenses — no findings:** agent_summary (empty→"no agents"; total/working counts; n==1 pluralization — all tested 0/1/2); sprint_summary (Some name+len / None — tested); cockpit_status (fixed [sprint,agents,focus] order — tested); the footer reserves content_h (=max(0, bounds.h-22) — no negative height) so the panes leave the strip; focus_label dispatch (agent/remote/terminal) mirrors #93; the centered overlays keep bounds.h. No panics (no unwrap; .max(0.0) guards the height). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** agent_summary_counts_and_pluralizes (0/1-working/2-with-1) + sprint_summary_present_and_absent (None + Some[2 tickets]) + cockpit_status_segments_in_order. `cargo nextest` → 3 passed.
- **LIVE self-test (PROVEN):** bundled + open + screencapture (WIN 13813) → READ the PNG: the always-on footer strip renders at the bottom showing **"no sprint   no agents   focus: terminal"** — the 3 cockpit_status segments in order, muted, below the panes (which reserve content_h). No synthetic input needed; a genuine live render proof. Capture: scratchpad/footer94.png. (forge_sprint is None in the bundle → the None branch is what live-rendered.)
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(5); forge #94 → done. **M2.F 5/6.** status_bar.rs (agent_summary/sprint_summary/cockpit_status, cov/MSI 100) + the always-on footer. LIVE-PROVEN (footer94.png: "no sprint · no agents · focus: terminal").
