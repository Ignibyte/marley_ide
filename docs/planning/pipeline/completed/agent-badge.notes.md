# agent status badge on its pane — Notes

- **Forge ticket:** #66 `b48ace07-d215-4994-8cdb-da70c75da4c1`
- **AAR:** `d567948c-1da4-4503-9362-86498d9aa0e2`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-066-agent-badge.md

## Phase 1 — Plan
- **Request:** forge #66 (M2.C seq-1, auto-approved) — surface the agent tag as a pane badge. First M2.C.
- **Classification:** work pipeline, `feature`, a small PURE `agent_view` + an app.rs render SHIM. UI.
- **Decisions:** D1 distinct agent glyphs (●/○/✓); D2 reuse launch_command for the label (no dup
  kind_label); D3 absolute corner badge in the pane render.
- **Reuse:** the pane render loop `for (pane_id, r) in &rect_list` (app.rs ~1275) has pane_id in scope;
  `self.agents.get(&pane_id)` reads the #62 map (the states_mut borrow at ~1260 has ended). marley_agent
  gives AgentKind/AgentStatus/AgentRun + launch_command.
- **AAR id:** `d567948c-1da4-4503-9362-86498d9aa0e2`.

## Phase 2 — Design

### PURE — `crates/marley_app/src/agent_view.rs` (NEW)
```rust
use marley_agent::{AgentRun, AgentStatus};

/// The status glyph for an agent run — DISTINCT from the forge status glyphs.
pub fn agent_status_glyph(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Working => "●",
        AgentStatus::Idle => "○",
        AgentStatus::Exited => "✓",
    }
}

/// The pane badge for an agent run: its label + its status glyph (e.g. "claude ○").
pub fn agent_badge(run: &AgentRun) -> String {
    format!("{} {}", run.label, agent_status_glyph(run.status))
}
```
(Uses the `AgentRun.label` field — set to "claude" at launch, #62 — so no `launch_command`/`kind_label`.)

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- `mod agent_view;` in lib.rs; `use crate::agent_view::agent_badge;`.
- In the pane render loop (`for (pane_id, r) in &rect_list`, ~1275), after the pane's content children,
  if `self.agents.get(&pane_id)` is `Some(run)`, add an absolutely-positioned top-right corner child:
  `div().absolute().top(px(4.)).right(px(8.)).px_1().bg(colors.surface).text_color(colors.muted)
  .child(agent_badge(run))`. (`self.agents` read is a clean immutable borrow — the states_mut at ~1260 ended.)

### File manifest
- NEW `crates/marley_app/src/agent_view.rs` — the 2 fns + tests.
- MODIFY `crates/marley_app/src/lib.rs` — `mod agent_view;`.
- MODIFY `crates/marley_app/src/app.rs` — the import + the corner badge in the pane loop.

### Mutation Targets (pure)
- `agent_status_glyph`: the 3 arms (each a DISTINCT glyph → a per-status test kills arm swaps + the
  no-catch-all-here 3-variant enum is exhaustive). `agent_badge`: the format (label BEFORE glyph, the space).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `agent_status_glyph_per_status` — Working→●, Idle→○, Exited→✓ | unit |
| REQ-002 | `agent_badge_is_label_then_glyph` — `AgentRun { Claude, "claude", Idle }` → "claude ○"; a Working one → "claude ●" | unit |
| REQ-003 | cmd-shift-a → the agent pane shows a corner "claude …" badge | self-test |
| REQ-004 | gate GREEN, cov/MSI 100 agent_view; app shim excluded | gate |

Uncoverable: the app.rs corner-badge render — masked + cov-excluded, proven by REQ-003.

### Risks / decisions
- D-2.1 the badge reads `run.label` (the model's display field) not `launch_command` — decouples display
  from the spawn CLI (they coincide today). D-2.2 an absolute corner child doesn't disturb the
  bottom-anchored terminal flow. D-2.3 #67 will flip `run.status` live → the ● / ○ / ✓ glyph updates for free.

## Phase 3 — Implement
- **Built (PURE):** `agent_view.rs` — `agent_status_glyph` (●/○/✓) + `agent_badge` (`{run.label} {glyph}`).
  `mod agent_view;` in lib.rs.
- **Built (SHIM, app.rs — mutants::skip):** import `agent_badge`; in the pane render loop, a top-right
  corner pill (`bg accent / on_accent text`) showing `agent_badge(run)` when `self.agents.get(&pane_id)`.
- **Deviations:** none (used `run.label` per the design; the pill uses accent/on_accent like the finder's
  selected row).
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK; `cargo nextest
  -p marley` 126 pass (no regression). Tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (probe + cargo-mutants + a borrow/overlap trace). Verdict: **PASS — no HIGH/MED defect;
  code correct, MSI 100 reachable, render borrow clean.**
- **Findings (all LOW, doc-accuracy):**
  | # | Finding | Action |
  |---|---|---|
  | L1 | The module doc overstated "DISTINCT from forge glyphs" — only ● (Working) differs from forge's ◐; ✓/○ are shared universal conventions. | **FIXED** — softened the doc to the accurate claim (● is the distinct active glyph; ✓/○ reuse done/idle conventions, never co-render with the forge overlay). |
  | L2 | Notes "Deviations: none" but the pill uses accent/on_accent vs the design's surface/muted. | Reconciled here — the accent/on_accent pill was a deliberate visibility choice (mirrors the finder's selected row); shim is masked, self-test-verified. |
  | L3 | REQ-002 EARS wording says `launch_command(kind)`; impl uses `run.label` (they coincide; D2/D-2.1 chose run.label). | P4 REQ-002 asserts the exact literal "claude ○" (kills the mutant under either reading). |
  | L4/L5 | ● / ○ in Menlo (self-test catches a tofu box); empty-label → " ○" (harmless). | Accepted — self-test / note only. |
- **Verified:** agent_status_glyph 3 distinct glyphs (●○✓); agent_badge = label-space-glyph (total, no
  panic); `self.agents.get(&pane_id)` is a clean immutable borrow (states_mut at ~1260 ended; closures
  capture pane_id not self); the absolute badge doesn't disturb the bottom-anchored flow; #67-ready (all
  3 arms map, reachable once status is driven). cargo-mutants: 4 whole-body mutants, all killed by the
  planned REQ-001 (all 3 statuses → cov + kill) + REQ-002 (exact string).
- **Fix applied (code):** L1 doc softening. No behavioral change.

## Phase 4 — Validate
- **Tests added** (agent_view.rs): `agent_status_glyph_per_status` (REQ-001 — ●/○/✓); `agent_badge_is_label_then_glyph`
  (REQ-002 — "claude ○" + "claude ●", exact strings).
- **Runs (actual):** `cargo nextest -p marley -E 'test(agent_status_glyph) or test(agent_badge)'` → 2 passed.
- **SELF-TEST (UI — REQ-003, drove the LIVE app):** cmd-shift-a → the new agent pane launched claude AND
  its top-right corner showed a cyan **"claude ○"** pill (`scratchpad/agent_badge.png`) — the badge
  renders (○ Idle, since #67 hasn't driven the status live yet).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, MSI 100.0%. app.rs shim excluded.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG; marley_agent.md badge-view note.
- **Knowledge:** aar-submit (5). No new rule (small pure ticket).
- **Ticket:** forge #66 → done; archived. **1/6 of M2.C.** The agent badge is visible (○ Idle until #67 drives it live).
