# launch + tag an agent pane — Notes

- **Forge ticket:** #62 `2ee9f90a-67c0-4a1e-889c-f99a78419a5f`
- **AAR:** `8a763e09-847a-471f-af02-bddf152e6e65`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-062-agent-launch.md

## Phase 1 — Plan
- **Request:** forge #62 (M2.B seq-4, auto-approved) — launch + tag an agent pane. First agent-cockpit
  INTERACTION (builds on #61's model).
- **Classification:** work pipeline, `feature`, small PURE (launch_command + keymap) + an app.rs SHIM.
  UI — validate self-test-captures.
- **Fork resolved (D1):** claude, auto-run (launch-and-observe). Configurable command + control later.
- **Reuse:** `split_focused(axis, dir, spawn)` returns the new PaneId (focused); `spawn_session` is the
  existing shell spawn; `focused_state_mut().session.write_bytes` writes to the new pane. `marley_agent`
  (#61) gives AgentRun + agent_kind_of; add `launch_command`.
- **Tag placement (D2):** `RootView.agents: HashMap<PaneId, AgentRun>` — a shim map, NOT in the pure
  workspace (keeps PaneState/the pane tree unchanged). A later ticket reads it to surface + observe.
- **Self-test note:** cmd-shift-a runs real `claude`; it may show an auth/welcome screen — the SPLIT +
  the launch is the proof (even a `command not found` would prove the write). State honestly.
- **AAR id:** `8a763e09-847a-471f-af02-bddf152e6e65`.

## Phase 2 — Design

### PURE-1 — `marley_agent/src/lib.rs`
```rust
/// The CLI program to spawn for an agent kind (the inverse of [`agent_kind_of`]).
pub fn launch_command(kind: AgentKind) -> &'static str {
    match kind {
        AgentKind::Claude => "claude",
        AgentKind::Codex => "codex",
    }
}
```

### PURE-2 — `keymap.rs` default_bindings
`(KeyBinding::chord(true, false, false, true, "a"), "new-agent")` (cmd-shift-a); keymap test asserts
`action_for(cmd-shift-a) == "new-agent"`.

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- Imports: `use marley_agent::{agent status types};` → `AgentKind, AgentRun, launch_command`; `use
  std::collections::HashMap;`; `PaneId` from `crate::workspace`.
- `RootView.agents: HashMap<PaneId, AgentRun>` (+ `agents: HashMap::new()` in `new()`).
- Dispatch arm (mirrors `split-pane`, 724):
```rust
"new-agent" => {
    if let Ok(pane_id) = self.workspace.split_focused(
        PaneAxis::Horizontal, SplitDirection::After,
        || spawn_session(&self.zdotdir, self.term_cols, self.term_rows),
    ) {
        // the split focuses the new pane — run the agent CLI in it, then tag it.
        if let Some(state) = self.workspace.focused_state_mut() {
            let cmd = format!("{}\r\n", launch_command(AgentKind::Claude));
            let _ = state.session.write_bytes(cmd.as_bytes());
        }
        self.agents.insert(pane_id, AgentRun::new(AgentKind::Claude, "claude".to_string()));
    }
}
```
- `marley_app/Cargo.toml` += `marley_agent = { path = "../marley_agent" }`.

### File manifest
- MODIFY `crates/marley_agent/src/lib.rs` — `launch_command` + a test.
- MODIFY `crates/marley_app/src/keymap.rs` — the cmd-shift-a binding + a test.
- MODIFY `crates/marley_app/src/app.rs` — the `agents` field, new() init, the new-agent dispatch, imports.
- MODIFY `crates/marley_app/Cargo.toml` — add marley_agent.

### Mutation Targets
- `launch_command`: the Claude/Codex match arms (each returns a DISTINCT string → a swap is caught).
  keymap: the cmd-shift-a `action_for` arm.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `launch_command_per_kind` — `launch_command(Claude)=="claude"`, `(Codex)=="codex"` | unit (marley_agent) |
| REQ-002 | keymap `cmd_shift_a_new_agent` — `action_for(cmd-shift-a)=="new-agent"` | unit |
| REQ-003 | cmd-shift-a → a pane splits + claude launches | self-test (drive cmd-shift-a → capture) |
| REQ-004 | gate GREEN, cov/MSI 100 launch_command + keymap; app shim excluded | gate |

Uncoverable: the app.rs new-agent dispatch (split+run+tag) — masked + cov-excluded, proven by REQ-003.
Self-test caveat: `claude` is the real CLI (may show an auth/welcome screen); the SPLIT + launch is the
proof; if it can't run, `command not found: claude` in the new pane still proves the write.

### Risks / decisions
- D-2.1 the tag map is in RootView (D2) — no pure-workspace change; a future close-pane should remove the
  pane's agents entry (a small leak otherwise — note for the observe ticket; harmless now). D-2.2
  `split_focused` focuses the new pane, so `focused_state_mut()` is the agent pane — the write targets it.
  D-2.3 auto-run (`\r\n`) is a RUN (unlike #59/#60's cooked-buffer insert) — an agent SHOULD start.

## Phase 3 — Implement
- **Built (PURE):** `marley_agent::launch_command` (Claude→"claude"/Codex→"codex"). `keymap.rs` —
  cmd-shift-a → "new-agent" + the keymap test.
- **Built (SHIM, app.rs — mutants::skip):** `RootView.agents: HashMap<PaneId, AgentRun>` (+ new() init);
  the `new-agent` dispatch (split_focused → run `claude\r\n` in the focused-new pane → tag `agents`).
  `marley_app/Cargo.toml` += marley_agent.
- **Deviation (good):** added `self.agents.remove(&closing)` to the `close-pane` dispatch — (a) gives the
  `agents` map a genuine READ so it isn't dead-code, (b) fixes the D-2.1 tag-leak (a closed agent pane's
  tag is dropped). `closing = self.workspace.focused()` captured before `close_focused()`.
- **Import fix:** `PaneId` lives in `crate::layout` (workspace only re-imports it privately) — imported
  from layout, not workspace.
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK (marley +
  marley_agent); `cargo nextest -p marley -p marley_agent` 127 pass (no regression). Tests are Phase 4.
  (Pre-existing `block v0.1.6` future-compat warning — unrelated.)

## Phase 3.5 — Inspect
- **Critic:** 1 (probe + a `-j1` cargo-mutants + a split_focused/close_focused trace + the #59 render
  distinction). Verdict: correctness/keymap/leak-fix sound; the one real code nit fixed.
- **Findings:**
  | # | Sev | Finding | Verdict / Action |
  |---|---|---|---|
  | F1 | HIGH (as stated) | `launch_command` has no test → 2 survivors (`→""`, `→"xyzzy"`), MSI<100. | **NOT a code defect** — this is the normal "inspect flags the pure fn needs its test → VALIDATE writes it" (my REQ-001). Tests are Phase 4. The critic supplied the exact round-trip test I'll use. |
  | F2 | LOW | The `AgentRun` label hardcoded `"claude"` — two sources of truth with `launch_command`. | **FIXED** — the dispatch now binds `let program = launch_command(Claude)` once and uses it for BOTH the write and the label (single source). clippy OK. |
- **Verified (probe/trace):** launch_command(Claude)="claude"/(Codex)="codex" + the round-trip
  `agent_kind_of(launch_command(k))==Some(k)`; **the write targets the NEW focused pane** (split_focused
  sets `self.focused = new` then `Ok(new)` → focused_state_mut is the agent pane) and **WILL render** —
  the `\r\n`-terminated write RUNS claude in the fresh shell through the same block path as rerun-last,
  fundamentally different from #59's cooked-buffer invisible-insert (that was a NON-newline write to
  Marley's own rendered prompt buffer; here we write to a fresh shell's PTY and want it to run); on spawn
  Err the tag+write are skipped (no orphan tag); **close-pane** captures `closing=focused()` BEFORE
  close_focused (which closes exactly that pane) then `agents.remove(&closing)` (leak fixed + a genuine
  READ → no dead_code; clippy -D warnings clean); cmd-shift-a is the sole `"a"` binding (no collision
  with cmd-shift-r/b/p; cmd-a unbound); no unwrap/expect on the new paths; §20 original.
- **Fix applied:** F2 (label single-source). F1 = the P4 launch_command test.

## Phase 4 — Validate
- **Tests added:** `marley_agent` `launch_command_is_the_inverse_of_agent_kind_of` (REQ-001 — Claude/Codex
  strings + the round-trip `agent_kind_of(launch_command(k))==Some(k)`). `keymap.rs` cmd-shift-a→new-agent
  (added in P3). Harness: added a `cmdshift:` verb to `scripts/selftest/drive.swift` (Cmd+Shift chord).
- **Runs (actual):** `cargo nextest -p marley_agent -p marley -E 'test(launch_command) or test(keymap)'`
  → 4 passed; workspace green in the gate.
- **SELF-TEST (UI — REQ-003, drove the LIVE app):** cmd-shift-a → the pane **split** (two panes) and the
  new focused pane launched **real `claude`** — its actual startup rendered ("2 new MCP servers found…
  forge / playwright" = the .mcp.json config), and the Details dock showed **"claude / running"**
  (`scratchpad/agent_launch.png`). The `\r\n` write ran claude end-to-end + rendered through the block
  path (confirming the inspect #59-distinction — a real run, not the invisible-insert trap).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, MSI 100.0%. app.rs shim
  excluded.
- **Pre-existing:** none (the `block v0.1.6` future-compat note is a dep, unrelated).

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; marley_agent.md launch flow (SHIPPED). Harness gained `cmdshift:`.
- **Knowledge:** aar-submit completed (5). Reused #59-distinction (a \\r\\n write to a FRESH shell RUNS+renders — the cooked-buffer trap only applies to Marley OWN prompt buffer). No new rule.
- **Ticket:** forge #62 → done; archived. **4/6 of M2.B** — the agent cockpit can now launch + track claude.
