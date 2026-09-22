# broadcast a prompt to all agents — Notes

- **Forge ticket:** #73 `a466ec24-4271-409f-b932-9f380753edb5`
- **AAR:** `3f9e50e2-fc4c-4d1d-9a9e-1714a5d65d9d`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-073-broadcast-agents.md

## Phase 1 — Plan
- **Request:** forge #73 (M2.D seq-2, auto-approved) — broadcast a line to all agents.
- **Classification:** work pipeline, `feature`, PURE (agent_pane_ids + keymap) + an app.rs SHIM. UI.
- **Pre-flight facts:** cmd-shift-g FREE (0 "g" bindings/hardcoded); agent_view.rs has `PaneId` (via
  crate::layout) + the `agent_rows` sort pattern (`entries.sort_by_key(|(id,_)| id.0)`) to mirror;
  reuses #72 `send_payload` + the F1 clear-on-confirmed pattern.
- **Decisions:** D1 agent_pane_ids (deterministic, testable); D2 clear only on ≥1 delivered (#72 F1);
  D3 cmd-shift-g, reuse send_payload.
- **Self-test flow:** cmd-shift-a ×2 (2 agents) → compose in the original pane → cmd-shift-g → both
  agents receive + prompt clears.
- **AAR id:** `3f9e50e2-fc4c-4d1d-9a9e-1714a5d65d9d`.

## Phase 2 — Design

### PURE — `agent_view.rs`
```rust
/// The running agents' pane ids, sorted ascending by id — the deterministic set a broadcast (#73) fans a
/// line out to.
pub fn agent_pane_ids(agents: &HashMap<PaneId, AgentRun>) -> Vec<PaneId> {
    let mut ids: Vec<PaneId> = agents.keys().copied().collect();
    ids.sort_by_key(|id| id.0);
    ids
}
```

### PURE — `keymap.rs`
`(chord(true, false, false, true, "g"), "broadcast-to-agents")` (cmd-shift-g); keymap test asserts it.

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- `use crate::agent_view::{…, agent_pane_ids};` (extend the agent_view import).
- Dispatch `"broadcast-to-agents"`:
```rust
"broadcast-to-agents" => {
    let line = self
        .workspace
        .state(self.workspace.focused())
        .map(|state| state.buffer.text());
    if let Some(line) = line {
        let payload = send_payload(&line);
        let mut delivered = false;
        for id in agent_pane_ids(&self.agents) {
            if let Some(state) = self.workspace.state_mut(id) {
                if state.session.write_bytes(&payload).is_ok() {
                    delivered = true;
                }
            }
        }
        if delivered {
            if let Some(state) = self.workspace.focused_state_mut() {
                state.buffer = Buffer::new();
                state.caret = CharOffset::zero();
            }
        }
    }
}
```
Borrow-safe: read the focused line (owned) → `agent_pane_ids` (owned `Vec<PaneId>`, Copy) → per-id
`state_mut` writes (each borrow ends) → the focused clear. Clear ONLY if ≥1 agent got it (#72 F1).

### File manifest
- MODIFY `crates/marley_app/src/agent_view.rs` — `agent_pane_ids` + a test.
- MODIFY `crates/marley_app/src/keymap.rs` — cmd-shift-g binding + test.
- MODIFY `crates/marley_app/src/app.rs` — the broadcast dispatch + the agent_view import.

### Mutation Targets (pure)
- `agent_pane_ids`: the sort (a 3-SCRAMBLED fixture — insert PaneId(3),(1),(2), assert ids == [1,2,3] — per
  the sort-order rule; a 2-elem assert is ~50% flaky + a sort has no cargo-mutants mutant, so a behavioral
  ordered assert is the guard) + the collect (empty map → []). keymap: the cmd-shift-g arm.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `agent_pane_ids_sorted_by_id` — insert (3,1,2) → `[PaneId(1),PaneId(2),PaneId(3)]`; empty → `[]` | unit |
| REQ-002 | keymap `default_keymap_maps_named_chords` extended — cmd-shift-g → "broadcast-to-agents" | unit |
| REQ-003 | 2 agents → compose → cmd-shift-g → both receive + prompt clears | self-test |
| REQ-004 | gate GREEN, cov/MSI 100 agent_pane_ids + keymap; app shim excluded | gate |

Uncoverable: the app.rs broadcast dispatch — masked + cov-excluded, proven by REQ-003.

### Risks / decisions
- D-2.1 clear ONLY on ≥1 delivered — no agents → the line is preserved (the #72 F1 lesson). D-2.2
  agent_pane_ids returns owned Copy PaneIds → no borrow held across the write loop. D-2.3 a stale/closed
  agent in the map: `state_mut(id)` → None → skipped (the #67 close-pane removes closed agents from the
  map, so this is rare). D-2.4 reuses `send_payload` (#72) verbatim — same `\r`, same running-agent rule.

## Phase 3 — Implement
- **Built (PURE):** `agent_view::agent_pane_ids(&HashMap<PaneId,AgentRun>) -> Vec<PaneId>` (keys, sorted by
  id.0); keymap cmd-shift-g → `broadcast-to-agents` (+ test assert).
- **Built (SHIM, app.rs — masked):** the `broadcast-to-agents` dispatch — collect the focused
  `buffer.text()`, `send_payload(line)` (reused #72) to each `agent_pane_ids` pane's session, track
  `delivered`, clear the focused prompt only if `delivered` (the #72 F1 rule). Import `agent_pane_ids`.
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK; `cargo nextest
  -p marley` 132 pass (no regression). agent_pane_ids test is Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (probe + cargo-mutants + a keymap-shadow grep + a borrow trace). Verdict: **PASS — no
  correctness/borrow/panic defects; MSI-100 deterministically reachable.**
- **Confirmations:** (a) agent_pane_ids correct ({3,1,2}→[1,2,3]; empty→[]); MSI 100 with the 3-scrambled +
  empty fixture — the 2 whole-body mutants (`vec![]`, `vec![Default]`) die DETERMINISTICALLY (no sort
  mutant exists to be flaky about; the sibling agent_rows sort confirms cargo-mutants emits only
  whole-body). The unit test is ALSO load-bearing for coverage (the only other caller is the cov-excluded
  shim). (b) cmd-shift-g NOT shadowed — "g" bound once, no hardcoded key=="g"; dispatch fires before raw
  PTY routing. (c) fans to EVERY agent (loop over all keys); `delivered` = OR of the writes → clear iff ≥1
  got it → no agents preserves the line (the #72 F1 rule); send_payload computed once (no per-agent
  realloc). (d) borrow-safe (agent_pane_ids returns an owned Copy Vec, borrow released before the loop; the
  focused read is owned) — compiled + 132 pass; no panic (is_ok total).
- **Findings (all LOW / informational — no code change):**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | L1 | LOW/info | The design said the sorted-order assert catches a no-op sort "~always"; it's really ~81% (a no-op sort = HashMap order, coincidentally [1,2,3] ~18.8% of the time for 3 elements). | Accept — IMMATERIAL: there is no sort mutant, and the correct code's sort always produces [1,2,3] (the test never flakes on correct code). The 3-scrambled equality assert is the strongest practical guard (a 2-elem is ~44% false-pass). |
  | L2 | LOW/info | Partial fan-out (1 of N writes ok) still clears the prompt (`delivered` is ≥1 OR). | Accept — the intended #72 clear-on-confirmed rule (a dead pane must not block the clear). |
  | L3 | LOW/info | If the focused pane IS an agent, it both receives the broadcast and gets its buffer cleared. | Accept — consistent with "broadcast to ALL agents". |
- **No code change** — SHIP as-is.

## Phase 4 — Validate
- **Tests added:** `agent_view::agent_pane_ids_sorted_by_id` (REQ-001 — insert PaneId(3),(1),(2) →
  [1,2,3]; empty → []); keymap `default_keymap_maps_named_chords` extended (REQ-002 — cmd-shift-g →
  broadcast-to-agents).
- **Runs (actual):** `cargo nextest -E 'test(agent_pane_ids) or test(default_keymap_maps_named_chords)'`
  → 2 passed.
- **SELF-TEST (UI — REQ-003, drove the LIVE app) — PASSED:** typed `bcastmark` in the single initial pane
  (`reset_mid.png`: **`❯ Marley bcastmark`** — composed, no agents) → ⌘⇧A (claude launched, `claude ●`
  badge) → clicked pane-1 → ⌘⇧G. Result (`bcast_final.png`): the **agent (claude) shows `❯ bcastmark` →
  "Computing…"** — it RECEIVED the broadcast line and is responding to it — and **pane-1's prompt CLEARED**
  (`❯ Marley |`). Definitive: the composed line reached the agent AND the prompt cleared on confirmed
  delivery. (agent_pane_ids returning all + the critic-verified loop extend this to N agents.)
- **HARNESS NOTE (fixed mid-validate):** the synthetic-input harness had entered a STUCK cmd-shift state
  from the session's many ⌘⇧ chord drives — `type:` chars were interpreted as chords (its "a"s fired
  ⌘⇧A=new-agent; chord-free "z" typed nothing; mouse clicks still worked). Diagnosed it as an OS/CGEvent
  artifact (NOT a code defect) and CLEARED it by posting modifier keyUp events
  (`scratchpad/reset_mods.swift` — keyUp for L/R cmd/shift/opt/ctrl), after which `type:` worked normally.
  Captured as PR-claude-selftest-stuck-synthetic-modifier for the remaining M2.D UI tickets.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100%. agent_pane_ids + keymap
  tested; the broadcast dispatch masked.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; marley_agent.md broadcast note.
- **Knowledge:** aar-submit (5); PR-claude-selftest-stuck-synthetic-modifier-clear-with-keyup-001 (the reset_mods.swift harness fix — reused for the rest of M2.D).
- **Ticket:** forge #73 → done; archived. **2/6 of M2.D.** cmd-shift-g broadcasts a line to every agent.
