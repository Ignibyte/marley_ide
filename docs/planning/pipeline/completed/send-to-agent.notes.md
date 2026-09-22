# send a line to a running agent — Notes

- **Forge ticket:** #72 `a358706a-f0d8-4a21-b62f-e4a7dd3d5781`
- **AAR:** `62e4730a-7fda-41cd-ba1a-66ad8dc0c8e1`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-072-send-to-agent.md

## Phase 1 — Plan
- **Request:** forge #72 (M2.D seq-1, auto-approved) — send a line to a running agent. First M2.D / control.
- **Classification:** work pipeline, `feature`, PURE (send_payload + keymap) + an app.rs SHIM. UI.
- **Pre-flight facts:** `state.buffer.text()` reads the prompt line; `Buffer::new()` clears it; cmd-shift-s
  FREE (no "s" keymap/hardcoded); the finder/palette use key_char (NOT self-drivable), but the terminal
  cooked-buffer typing uses the key path (drivable — proven by earlier `type:` self-tests).
- **Decisions:** D1 compose at the prompt (self-testable); D2 raw write_bytes is CORRECT for a running
  agent (inverse of #59/#65); D3 target = last_agent; D4 cmd-shift-s free.
- **Self-test flow:** type a line at pane-1's prompt → cmd-shift-a (launch agent in pane-2, last_agent=2,
  focus 2) → click pane-1 (focus) → cmd-shift-s → capture pane-2 (the agent) shows the line; pane-1 prompt clears.
- **AAR id:** `62e4730a-7fda-41cd-ba1a-66ad8dc0c8e1`.

## Phase 2 — Design

### PURE — `marley_agent/src/lib.rs`
```rust
/// The PTY bytes to send a composed line to a running agent — the line plus a trailing `\r` (Enter for
/// the foreground program). A running agent WANTS this raw write (unlike a bare cooked prompt, #59/#65).
pub fn send_payload(line: &str) -> Vec<u8> {
    format!("{line}\r").into_bytes()
}
```

### PURE — `keymap.rs`
`(chord(true, false, false, true, "s"), "send-to-agent")` (cmd-shift-s); keymap test asserts it.

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- `RootView.last_agent: Option<PaneId>` (+ new() `None`); import `send_payload`.
- In the `new-agent` dispatch (#62), after `self.agents.insert(pane_id, …)`: `self.last_agent = Some(pane_id);`.
- Dispatch `"send-to-agent"`:
```rust
"send-to-agent" => {
    let line = self
        .workspace
        .state(self.workspace.focused())
        .map(|state| state.buffer.text());
    if let (Some(line), Some(target)) = (line, self.last_agent) {
        let payload = send_payload(&line);
        if let Some(state) = self.workspace.state_mut(target) {
            let _ = state.session.write_bytes(&payload);
        }
        if let Some(state) = self.workspace.focused_state_mut() {
            state.buffer = Buffer::new();
            state.caret = CharOffset::from(0);
        }
    }
}
```
Borrow-safe: read the focused line (owned) → write to `target` (state_mut) → clear the focused
(focused_state_mut) — each borrow ends before the next. No agent / no focused → no-op.

### File manifest
- MODIFY `crates/marley_agent/src/lib.rs` — `send_payload` + a test.
- MODIFY `crates/marley_app/src/keymap.rs` — cmd-shift-s binding + test.
- MODIFY `crates/marley_app/src/app.rs` — `last_agent` field, new() init, set it in new-agent, the
  send-to-agent dispatch, imports.

### Mutation Targets (pure)
- `send_payload`: the `\r` terminator + the line (a `send_payload("ls")==b"ls\r"` + `""==b"\r"` test pins
  both). keymap: the cmd-shift-s arm.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `send_payload_appends_cr` — `send_payload("ls")==b"ls\r"`; `send_payload("")==b"\r"` | unit (marley_agent) |
| REQ-002 | keymap `cmd_shift_s_send_to_agent` — `action_for(cmd-shift-s)=="send-to-agent"` | unit |
| REQ-003 | compose → cmd-shift-s → the agent receives the line + the prompt clears | self-test |
| REQ-004 | gate GREEN, cov/MSI 100 send_payload + keymap; app shim excluded | gate |

Uncoverable: the app.rs send dispatch — masked + cov-excluded, proven by REQ-003.

### Risks / decisions
- D-2.1 the RAW write to the agent (`write_bytes`) is CORRECT (running claude) — the inverse of #59/#65's
  cooked-buffer rule; the critic must confirm the rationale. D-2.2 `last_agent` points at a pane that may
  have closed — `state_mut(target)` returns None → no-op (safe); the #67 leak-fix removes closed agents
  from the map but not last_agent — a stale last_agent → a harmless no-op write to a missing pane. D-2.3
  clears the FOCUSED prompt (the compose pane), not the agent's.

## Phase 3 — Implement
- **Built (PURE):** `marley_agent::send_payload(&str) -> Vec<u8>` = `"{line}\r"` bytes; keymap cmd-shift-s
  → `send-to-agent` (+ the test assert).
- **Built (SHIM, app.rs — masked):** `RootView.last_agent: Option<PaneId>` (init None; set to the new pane
  in `new-agent`); the `send-to-agent` dispatch — collect the focused `buffer.text()`, write
  `send_payload(line)` to `last_agent`'s session via `state_mut(target)`, clear the focused prompt
  (`Buffer::new()` + `CharOffset::zero()`). Import `send_payload`.
- **Deviations:** used `CharOffset::zero()` (the existing clear idiom at app.rs:346), not `from(0)`.
- **Verification:** `cargo fmt`; `cargo check -p marley -p marley_agent` 0 err; clippy `-D warnings` OK;
  `cargo nextest` 136 pass (no regression). `state`/`state_mut(id)` exist. send_payload test is Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (probe + cargo-mutants + a keymap-shadow grep + a byte-level \r-vs-\r\n trace + borrow check).
  Verdict: **PASS — ship (1 LOW fixed).**
- **Confirmations:** (a) send_payload correct — `send_payload("ls")==b"ls\r"` (6c 73 0d), `""==b"\r"`,
  `"café"` multibyte-clean; MSI 100 reachable (the test pair kills all 3 whole-body mutants; no \r-drop
  mutant exists). (b) **cmd-shift-s NOT shadowed** — "s" bound once; the hardcoded intercepts are cmd-F
  (with the #64 `!shift` guard), cmd-C, cmd-V — none touch "s"; cmd-shift chords fall to the keymap. (c)
  **the raw write is byte-provably CORRECT (inverse of #59/#65)** — `send_payload(line)` == typing the line
  + Enter into the running agent (`\r` matches keys.rs Enter `vec![b'\r']`); claude is foreground
  (is_command_running) so it reaches stdin; the #59 cooked trap is only for a BARE prompt (`\r\n` via
  write_command). (d) borrow-safe (sequential; PaneId is Copy); a stale last_agent → memory-safe no-op.
- **Findings:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | F1 | LOW | The prompt-clear ran even when `state_mut(target)` was None (a closed last_agent) → the composed line was WIPED with nothing sent + no feedback (narrow: close the last agent, don't relaunch, compose, cmd-shift-s). Shim → no gate catches it. | **FIXED** — clear ONLY on a confirmed send: `let sent = match state_mut(target) { Some(s)=>write_bytes(..).is_ok(), None=>false }; if sent { clear }`. A missing target / failed write now preserves the line. |
- **Fix applied (code):** F1 (clear-on-confirmed-delivery). clippy OK, 0 errors.

## Phase 4 — Validate
- **Tests added:** `marley_agent::send_payload_appends_cr` (REQ-001 — "ls"→b"ls\r", ""→b"\r", "café"
  multibyte); keymap `default_keymap_maps_named_chords` extended (REQ-002 — cmd-shift-s→send-to-agent).
- **Runs (actual):** `cargo nextest -E 'test(send_payload) or test(default_keymap_maps_named_chords)'` → 2 passed.
- **SELF-TEST (UI — REQ-003, drove the LIVE app):** typed `sendtest` in pane-1 → ⌘⇧A (claude launched in
  pane-2, `last_agent`=2, splash + the `claude ●` working badge) → clicked pane-1 → ⌘⇧S. Result
  (`send_agent2/3.png`): **pane-1's prompt CLEARED** (`❯ Marley |`) and claude received + auto-submitted
  the line (its input box cycled). **The clear is proof-positive of delivery**: the F1-fixed dispatch
  clears the compose buffer ONLY when `write_bytes(payload).is_ok()` to the agent's PTY — so a clear ⟺ the
  bytes reached claude's stdin. (claude auto-submits on the `\r`, so it can't be freeze-framed in its input
  box — but the clear + the critic's byte-proof [send_payload == typing line+Enter] establish delivery.)
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100%. send_payload + keymap tested; the send dispatch masked.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; marley_agent.md "## Control (M2.D)" (send_payload + last_agent + the \r-vs-\r\n inverse).
- **Knowledge:** aar-submit (5); BF-claude-compose-buffer-cleared-before-confirmed-delivery-drops-input-001 + PR-claude-clear-input-only-after-confirmed-delivery-001 (from inspect F1).
- **Ticket:** forge #72 → done; archived. **1/6 of M2.D.** cmd-shift-s sends a composed line to the last agent.
