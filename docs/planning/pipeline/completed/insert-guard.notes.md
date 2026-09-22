# guard finder/file-click prompt-inserts on is_command_running — Notes

- **Forge ticket:** #71 `1743b2b3-91e7-4fb8-9652-42f0821a08f3` · **AAR:** `fda397ee-2a23-4a1c-bbe6-645b34f68c7c`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-071-insert-guard.md

## Phase 1 — Plan
- **Request:** forge #71 (Terminal Polish 3/15) — #65's inspect follow-up (LOW, was out of scope).
- **Classification:** work pipeline, `bug`, SHIM-only (app.rs). No pure surface.
- **Insert sites:** finder Enter app.rs:830-838 (`handle_finder_key` "enter"); file-tree click app.rs:1540-
  1551 (the FILE-row `on_mouse_down`). Both `buffer.edit(caret..caret, path)` unconditionally.
- **Paste guard to mirror:** #42 (app.rs:1189 / the cmd-V path) — `if running { write_bytes } else { edit }`.
- **Decisions:** D1 mirror #42; D2 no caret advance on write; D3 SHIM-only, no tests.
- **AAR id:** `fda397ee-2a23-4a1c-bbe6-645b34f68c7c`.

## Phase 2 — Design
- **Both sites** wrap the existing insert in the #42 guard:
  ```rust
  if let Some(state) = <focused state> {
      if state.session.is_command_running() {
          let _ = state.session.write_bytes(text.as_bytes()); // → the running program
      } else {
          state.buffer.edit(state.caret..state.caret, &text, EditOrigin::Human);
          state.caret = CharOffset::from(state.caret.as_usize() + text.chars().count());
      }
  }
  ```
  - Site A: `handle_finder_key` "enter" (app.rs ~830) — `self.workspace.focused_state_mut()`.
  - Site B: the file-tree FILE-row `on_mouse_down` (app.rs ~1540) — `view.workspace.focused_state_mut()`.
- **File manifest:** MODIFY `crates/marley_app/src/app.rs` (the two insert blocks).
- **Regression Test Plan:** NO unit tests — SHIM-only (cov-excluded + masked), no pure surface. REQ-001/002
  = self-test/structural; REQ-003 = the gate (masked → green). Uncoverable: both handlers — gpui/PTY,
  masked; the guard is the PROVEN #42 paste split applied verbatim.
- **Risks:** D-2.1 on the write path NO caret advance (Marley's cooked caret is untouched; the program owns
  its cursor) — matches #42. D-2.2 a plain `write_bytes(path)` (not bracketed) — a picked path is like a
  typed path, not a multi-line paste. D-2.3 the finder/tree still CLOSE after the pick (unchanged).

## Phase 3 — Implement
- **Built:** both insert sites now guard on `state.session.is_command_running()` — running →
  `write_bytes(text.as_bytes())` (to the program, no caret advance), else the existing `buffer.edit` +
  caret advance. Site A: handle_finder_key "enter". Site B: the file-tree FILE-row on_mouse_down.
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK. SHIM-only, masked.

## Phase 3.5 — Inspect
- **Method:** self-review (scaled to a 2-site shim change that mirrors the PROVEN #42 paste guard verbatim
  — no subagent).
- **Lenses covered — no findings:** correctness (running→`write_bytes` with NO caret advance [the cooked
  caret is correctly untouched; the program owns its cursor], prompt→the unchanged `buffer.edit`+advance;
  identical at both sites); security/data (plain `write_bytes(path)` — a path is a word like a typed path,
  no bracketing; no panic — write_bytes result `let _`, edit unchanged); simplification (6-line dup across
  two distinct contexts [a method vs a listener closure] — acceptable). The guard is the #42 pattern that
  already ships + is self-test-proven.
- **Fix applied:** none (clean).

## Phase 4 — Validate
- (pending)

## Phase 5 — Complete
- (pending)

## Phase 4 — Validate
- No unit tests (masked shim). `cargo nextest -p marley` → 158 pass (no regression).
- Self-test (both prompt + running-command cases) ENV-BLOCKED (needs typing + a running command) → verified structurally: the guard is the #42 paste split applied verbatim at both sites.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**.

## Phase 5 — Complete
- CHANGELOG ### Fixed; aar-submit(5); forge #71 → done. **Terminal Polish 3/4.** The finder-Enter + file-tree-click inserts now mirror the #42 paste guard. Masked shim; gate GREEN; self-test env-blocked (structural).
