# drive AgentStatus live — Notes

- **Forge ticket:** #67 `a12d6977-0e9a-4402-84fb-8f0509caa635`
- **AAR:** `277e763d-5448-4f71-be43-0cebdbe9a70d`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-067-agent-status-live.md

## Phase 1 — Plan
- **Request:** forge #67 (M2.C seq-2, auto-approved) — make the #66 badge live. The OBSERVE step.
- **Classification:** work pipeline, `feature`, SHIM-ONLY (no new pure surface — reuses #61's
  `agent_status_from`). UI — validate self-test-captures the live ● badge.
- **Signals (grep'd):** `session.is_command_running()` (session.rs:175) → Working; the pump's
  `SessionEvent::ChildExited` (app.rs:243) marks a pane dead → auto-closed (app.rs:241). So while claude
  runs, is_command_running=true → ●; back at the shell prompt → ○; a dead pane is removed (not shown ✓).
- **Decisions:** D1 reuse agent_status_from (no new tests); D2 active=is_command_running, exited=false
  for live panes; D3 all logic in mutants::skip fns (gate has nothing to mutate; app.rs cov-excluded);
  D4 leak fix (remove agents entry on pump-close, completing #62's lifecycle).
- **AAR id:** `277e763d-5448-4f71-be43-0cebdbe9a70d`.

## Phase 2 — Design

### SHIM — `app.rs` (all mutants::skip / masked; NO new pure surface)
- Import: extend `use marley_agent::{...}` with `agent_status_from`.
- New method:
```rust
    /// Refresh each launched agent's status from its pane's live session state (#67): ● Working while a
    /// command runs, ○ Idle at the prompt. Called each pump tick.
    #[cfg_attr(test, mutants::skip)]
    fn refresh_agent_statuses(&mut self) {
        // Collect (read the workspace) BEFORE mutating self.agents — avoids borrowing both at once.
        let updates: Vec<(PaneId, bool)> = self
            .agents
            .keys()
            .filter_map(|&id| {
                self.workspace
                    .state(id)
                    .map(|state| (id, state.session.is_command_running()))
            })
            .collect();
        for (id, running) in updates {
            if let Some(run) = self.agents.get_mut(&id) {
                run.status = agent_status_from(false, running);
            }
        }
    }
```
- Pump closure (after the `for id in dead` loop, before `if dirty`): `view.refresh_agent_statuses();`.
  (Status changes coincide with Preexec/Precmd pump events → `dirty` is already true → the badge repaints
  the same tick.)
- LEAK FIX — in the dead-close block: `view.agents.remove(&id);` inside `if let Ok(state) =
  view.workspace.close(id)` (PaneId is Copy; close(id) + remove(&id) both fine). Completes #62's lifecycle
  (the pump auto-close path was the one place that didn't drop the tag).

### File manifest
- MODIFY `crates/marley_app/src/app.rs` — the import, `refresh_agent_statuses`, the pump-tick call, the
  agents-remove leak fix. (No other files; no new pure surface.)

### Mutation / coverage
- ZERO new pure lines. `refresh_agent_statuses` is `mutants::skip` + app.rs is cov-excluded → the gate has
  nothing new to mutate/cover. The status DECISION is `agent_status_from` (already cov/MSI 100, #61).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | badge ● while an agent runs a command, ○ at its prompt | self-test (cmd-shift-a → ● live) |
| REQ-002 | the leak fix — a pump-auto-closed agent pane drops its `agents` entry | code review (masked pump path) + no-regression suite |
| REQ-003 | `cargo nextest --workspace` green (no regression; #61's agent_status_from tests) + gate GREEN | gate |

Uncoverable: the pump closure + refresh (masked, live loop) — proven by the self-test (REQ-001). No NEW
unit tests (D1 — the decision is #61's tested fn); run the full suite to confirm no regression + satisfy
the tests-ran gate.

### Risks / decisions
- D-2.1 the borrow-safe collect-then-write avoids a workspace+agents double borrow. D-2.2 no forced dirty
  — status flips ride the Preexec/Precmd pump events (already dirty). D-2.3 Exited (✓) is intentionally
  not shown — a dead agent pane is removed (leak fix), matching every dead pane; the live signal is ●↔○.

## Phase 3 — Implement
- **Built (SHIM, app.rs — all masked):** imported `agent_status_from`; `refresh_agent_statuses(&mut self)`
  (`mutants::skip`, collect-then-write); the pump-closure call after the dead loop; the leak fix
  (`view.agents.remove(&id)` in the dead-close block).
- **Deviations:** none — the borrow-safe collect-then-write compiled clean (no workspace+agents double borrow).
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK; `cargo nextest
  -p marley` 128 pass (no regression). No new tests (D1 — reuses #61's agent_status_from). Zero new pure
  lines → the gate has nothing new to cover/mutate; validate = no-regression suite + the live self-test.

## Phase 3.5 — Inspect
- **Critic:** 1 (diff read + a `cargo mutants --list` masking check + a borrow/session trace). Verdict:
  **SHIP — no HIGH/MED.**
- **Confirmations:** (a) borrow-safe — the collect-then-write reads two disjoint immutable field borrows
  (agents.keys + workspace.state), materializes an owned Vec, then agents.get_mut; compiles, 128 pass.
  (b) Working/Idle mapping correct — `is_command_running() = blocks().current().is_some()`, true while
  claude runs → ●, false at the prompt → ○. (c) leak fix sound — the only two close sites (pump auto-close
  app.rs:253 + dispatch close_focused :807) both now `agents.remove`; remove is idempotent. **(d) KEY:
  `cargo mutants --file app.rs --list` → 0 mutants** (new() carries mutants::skip at app.rs:173, so the
  whole pump closure incl. the leak-fix + refresh call is masked; app.rs also cov-excluded) → NO gate:5
  risk. No missed repaint (status flips ride the same DCS-byte pump events that set dirty). No panic.
- **Finding:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | L1 | LOW | Exited (✓) is live-unreachable: a dead non-last agent pane is removed (vanishes); a dead LAST pane isn't closed + shows ○ (is_command_running false after finish_current_if_running), not ✓. | Accepted — matches D2/D3 ("dead pane vanishes; ●↔○ is the live signal"). A future ticket could drive `exited` from a per-pane dead flag. |
- **No code change** — SHIP as-is.

## Phase 4 — Validate
- **Tests:** NONE new (D1 — the decision is #61's `agent_status_from`, already cov/MSI 100).
- **No-regression (actual):** `cargo nextest -p marley -p marley_agent -p marley_forge_client` → 141 passed.
  (Skipped the marley_terminal real-PTY integration test for the quick check — unchanged crate.)
- **SELF-TEST (UI — REQ-001, drove the LIVE app):** cmd-shift-a → the agent pane's badge now shows
  **"claude ●"** (filled = Working, because claude is a running command) — vs #66's static "claude ○"
  (`scratchpad/agent_live.png`). The ○→● change PROVES the live drive works.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, MSI 100.0%. app.rs masked
  (0 mutants — new() is mutants::skip).
- **Env note:** the gate first hung on the `workspace_two_real_sessions_are_independent` real-PTY test —
  **3 stale instances from earlier runs were competing** (the #27 flake). Killed them, re-ran GREEN. NOT
  a code issue.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Changed` (live badge); marley_agent.md live-drive note.
- **Knowledge:** aar-submit (5). No new rule. Env: killed 3 stale real-PTY test procs that hung the gate (the #27 flake — kill `integration-*real_sessions` before the gate).
- **Ticket:** forge #67 → done; archived. **2/6 of M2.C.** The agent badge is now LIVE (● working / ○ idle).
