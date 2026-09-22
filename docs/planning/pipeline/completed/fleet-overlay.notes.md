# Fleet overlay (cmd-shift-e) — Notes

- **Forge ticket:** #68 `0690368e-6fbe-419d-aa20-13f51977a0f4`
- **AAR:** `a577e1a8-9793-4265-8678-6c2f1aa0b05a`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-068-fleet-overlay.md

## Phase 1 — Plan
- **Request:** forge #68 (M2.C seq-3, auto-approved) — the Fleet overlay, agent counterpart of the forge pane.
- **Classification:** work pipeline, `feature`, PURE (agent_view additions + keymap) + an app.rs overlay
  SHIM. UI — validate self-test-captures.
- **Pre-flight facts:** cmd-shift-e FREE (no keymap "e", no hardcoded cmd-E → no #64-style shadow);
  `PaneId(pub u64)` → sort by `id.0`.
- **Reuse:** the ⌘⇧F forge overlay pattern (#64); `agent_status_glyph` (#66); the `RootView.agents` map (#62).
- **Decisions:** D1 cmd-shift-e; D2 sort by id.0; D3 agent_view imports PaneId; D4 add agent_status_label.
- **AAR id:** `a577e1a8-9793-4265-8678-6c2f1aa0b05a`.

## Phase 2 — Design

### PURE — `agent_view.rs` (extend; add `use crate::layout::PaneId; use std::collections::HashMap;`)
```rust
/// The status text label for an agent run (for the Fleet overlay).
pub fn agent_status_label(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Working => "working",
        AgentStatus::Idle => "idle",
        AgentStatus::Exited => "exited",
    }
}

/// One fleet row: an agent's label + its status glyph + status text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRow {
    pub label: String,
    pub glyph: &'static str,
    pub status: &'static str,
}

/// The fleet rows for the launched agents, sorted by pane id (deterministic — HashMap order is random).
pub fn agent_rows(agents: &HashMap<PaneId, AgentRun>) -> Vec<AgentRow> {
    let mut entries: Vec<(&PaneId, &AgentRun)> = agents.iter().collect();
    entries.sort_by_key(|(id, _)| id.0);
    entries
        .into_iter()
        .map(|(_, run)| AgentRow {
            label: run.label.clone(),
            glyph: agent_status_glyph(run.status),
            status: agent_status_label(run.status),
        })
        .collect()
}
```

### PURE — `keymap.rs`
`(chord(true, false, false, true, "e"), "toggle-fleet")` (cmd-shift-e); keymap test asserts it.

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- `RootView.fleet_open: bool` (+ new() init `false`); `use crate::agent_view::{agent_badge, agent_rows};`.
- Dispatch `"toggle-fleet"` → `self.fleet_open = !self.fleet_open`.
- Overlay (mirror the ⌘⇧F forge overlay): header `"\u{1f6f0} Agents ({N})"` (🛰) + `agent_rows(&self.agents)`
  rows `"{glyph} {label} ({status})"`, or `"\u{1f6f0} no agents running"` when empty.

### File manifest
- MODIFY `crates/marley_app/src/agent_view.rs` — agent_status_label + AgentRow + agent_rows + tests.
- MODIFY `crates/marley_app/src/keymap.rs` — cmd-shift-e binding + test.
- MODIFY `crates/marley_app/src/app.rs` — fleet_open field, new() init, toggle-fleet dispatch, the overlay, imports.

### Mutation Targets (pure)
- `agent_status_label`: the 3 arms (distinct strings). `agent_rows`: the `sort_by_key(id.0)` (a 2-agent
  out-of-insertion-order test pins it) + the field mapping (label/glyph/status). keymap: the cmd-shift-e arm.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `agent_status_label_per_status` — Working→"working", Idle→"idle", Exited→"exited" | unit |
| REQ-002 | `agent_rows_sorted_by_pane_id` — insert PaneId(2) then PaneId(1) (distinct labels/statuses) → rows in id order [1,2] with full fields; empty map → `[]` | unit |
| REQ-003 | keymap `cmd_shift_e_toggle_fleet` — `action_for(cmd-shift-e)=="toggle-fleet"` | unit |
| REQ-004 | ⌘⇧E lists the running agents | self-test (launch agent, ⌘⇧E → capture) |
| REQ-005 | gate GREEN, cov/MSI 100 agent_view + keymap; app shim excluded | gate |

Uncoverable: the app.rs fleet overlay — masked + cov-excluded, proven by REQ-004.

### Risks / decisions
- D-2.1 sort by `id.0` (not label) — deterministic + handles duplicate "claude" labels. D-2.2 agent_view
  imports crate::layout::PaneId (a pure value type) — no gpui pulled. D-2.3 the overlay reads the LIVE
  `self.agents` each render, so #67's live statuses show in the fleet automatically. D-2.4 cmd-shift-e is
  free (no shadow) — checked per PR-claude-new-chord-shadowed-by-hardcoded-key-001.

## Phase 3 — Implement
- **Built (PURE):** `agent_view.rs` — `agent_status_label` (working/idle/exited) + `AgentRow` + `agent_rows`
  (sort by id.0, map). `keymap.rs` — cmd-shift-e → toggle-fleet + the keymap test.
- **Built (SHIM, app.rs — masked):** `RootView.fleet_open` (+ new() init); dispatch `toggle-fleet`; the
  🛰 Fleet overlay (mirrors the forge overlay — header "Agents (N)" + rows "{glyph} {label} ({status})",
  or "no agents running"). Import `agent_rows`.
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK; `cargo nextest
  -p marley` 128 pass (keymap test added, no regression). Tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (probe + a `-j2` cargo-mutants + a HashMap-order-flakiness measurement + a shadow grep).
  Verdict: **PASS — correct, no blockers; MSI 100 reachable.**
- **Mutants:** 8 (7 viable / 1 unviable). 4 caught (#66's glyph/badge), 3 MISSED = the new pure fns
  (agent_status_label ×2, agent_rows `vec![]` ×1) — killed by the planned REQ-001 + REQ-002 tests.
- **Findings (no code change — P4 test refinements):**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | L1 | LOW | The `sort_by_key(id.0)` gets NO cargo-mutants mutant (whole-body only), so it's guarded ONLY by the ordered behavioral assert — and a **2-element** assert is a FLAKY regression detector (HashMap 2-elem order is ~50/50 process-random, measured). | **P4 uses ≥3 agents inserted OUT of id-order** → asserts id-sorted, so an accidental-correct order without sorting is 1/n! — a robust guard. |
  | I1 | INFO | The REQ-002 rows test's NON-EMPTY case is load-bearing (kills the `vec![]` mutant #7); an empty-only test would leave it surviving. | P4 REQ-002 has both the non-empty (sorted) + empty cases. |
  | L2 | LOW | The Fleet list is uncapped (no take(N), unlike the finder's take(20)). | Accept — few agents expected; note only. |
- **Verified:** agent_status_label distinct (working/idle/exited); agent_rows sorts by id.0 + maps fields;
  **NO cmd-E shadow** (no hardcoded `key=="e"`; the cmd-F check is `!shift`-guarded per #64; toggle-fleet
  dispatched at app.rs:781, reachable); PaneId import doesn't pull gpui (layout.rs gpui-free, `PaneId(pub
  u64)` pure); the overlay reads the LIVE self.agents (shows #67's ●/○); no panic; render mutants::skip +
  cov-excluded.
- **No code change** — L1/I1 = the P4 3-agent-scrambled rows test; L2 accepted.

## Phase 4 — Validate
- **Tests added** (agent_view.rs): `agent_status_label_per_status` (REQ-001); `agent_rows_sorted_by_pane_id`
  (REQ-002 — **3 agents inserted 3,1,2 → asserted sorted by id [a,b,c] with full glyph/status fields**
  per L1's robust-sort guard; empty→empty). keymap cmd-shift-e→toggle-fleet (added in P3).
- **Runs (actual):** `cargo nextest -p marley -E 'test(agent_status_label) or test(agent_rows) or
  test(keymap)'` → 5 passed.
- **SELF-TEST (UI — REQ-004, drove the LIVE app):** cmd-shift-a → an agent; cmd-shift-e → the 🛰 Fleet
  overlay showed **"Agents (1)"** + **"● claude (working)"** (`scratchpad/fleet.png`) — the LIVE ● status
  (#67) in the fleet list, alongside the pane's "claude ●" badge (#66). All three M2.C tickets compose.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, MSI 100.0%. app shim excluded.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; marley_agent.md fleet note.
- **Knowledge:** aar-submit (5); PR-claude-sort-order-with-no-mutant-needs-3plus-scrambled-fixture-001 (MED — a HashMap→sorted-Vec sort has no mutant; a 2-elem ordered assert is ~50% flaky, use 3+ scrambled).
- **Ticket:** forge #68 → done; archived. **3/6 of M2.C** (halfway). The cockpit: agent badge (live) + Fleet overlay + Forge pane.
