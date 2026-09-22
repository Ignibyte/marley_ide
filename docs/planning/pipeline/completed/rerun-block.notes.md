# re-run a Block's command (click / cmd-R) — Notes

- **Forge ticket:** #46 `836fc74c-40e2-40f6-b85b-2a3620fdba53`
- **AAR:** `e7f1b647-55f0-41af-80d6-b7c204306916`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-046-rerun-block.md
- **Pipeline spec:** rerun-block.spec.md

## Phase 1 — Plan
- **Request:** forge #46 (M1.G "Block Workflows & Selection" seq-4, auto-approved) — re-run a block's
  command without retyping.
- **Classification / tier:** work pipeline, `feature`, a THIN pure method (`rerun_command` in
  terminal_blocks — cov/MSI 100) + a SHIM (the header ↻ affordance + the #40-guarded write).
- **Discovery (§18):**
  - `Block` has `state: BlockState` (Finished) + `command: String` → `rerun_command` = `Some(command)`
    iff `Finished && !empty`.
  - `TerminalSession::write_command(cmd)` writes `cmd\r\n` (session.rs:226 — used by on_submit) → the
    shell runs it; `is_command_running()` (session.rs:175, #40) is the mid-command guard.
  - The block header's hover affordance row (#45's `⧉ cmd`/`⧉ out`) is where the ↻ re-run glyph joins;
    the click uses the #45 deferred `(pane, index)` lookup.
  - Deps #36/#45 (header) + #40 (guard) — done.
- **Decisions:** D1–D3 in the spec (Finished + non-empty; the session-idle guard is shim; write_command
  reused).
- **Open questions for Design:** whether `rerun_command` is a method (yes — beside `copy_text`) vs a
  free fn; the glyph (↻); whether cmd-R targets the LAST finished block (yes) — the shim finds it.
- **AAR id:** `e7f1b647-55f0-41af-80d6-b7c204306916`.

## Phase 2 — Design

### PURE — `crates/terminal_blocks/src/block.rs` (beside `copy_text`)
```rust
impl Block {
    /// The command to re-run this block (R30): `Some(command)` IFF the block has FINISHED and its
    /// command is non-empty; `None` for a still-running/pending block or an empty command (not
    /// re-runnable). The app resends it (`write_command`) only when the session is idle (#40).
    pub fn rerun_command(&self) -> Option<String> {
        if self.state == BlockState::Finished && !self.command.is_empty() {
            Some(self.command.clone())
        } else {
            None
        }
    }
}
```

### SHIM — `crates/marley_app/src/app.rs` (the #45 header affordance row)
A ↻ re-run glyph joins `⧉ cmd`/`⧉ out`; the click looks the block up by `(pane, index)` and, guarded
by the #40 session state, resends the command. EXTRACT the owned command first so the immutable
`blocks()` borrow ends before the `&mut write_command`:
```rust
.on_mouse_down(MouseButton::Left, cx.listener(move |view, _e: &MouseDownEvent, _w, cx| {
    if let Some(state) = view.workspace.state_mut(pane_id) {
        if !state.session.is_command_running() {
            let cmd = state.session.blocks().iter().nth(block_index).and_then(|b| b.rerun_command());
            if let Some(cmd) = cmd {
                let _ = state.session.write_command(&cmd);
            }
        }
    }
    cx.stop_propagation();
}))
```
cmd-R re-runs the most recent finished block (same helper).

### File manifest
- M `crates/terminal_blocks/src/block.rs` — `Block::rerun_command` + tests.
- M `crates/marley_app/src/app.rs` — the ↻ re-run glyph + the #40-guarded deferred-lookup write.
- M `docs/specs/SPEC-terminal-blocks.spec.md` — R30 + Mutation-Targets. `SPEC-app-shell.spec.md` — R50.
  CHANGELOG; arch.

### Mutation Targets
- `rerun_command` — the `state == BlockState::Finished` check (a Running block → None); the
  `!command.is_empty()` guard (empty → None); the `&&` (a `||` would let a Running non-empty block
  return Some); the whole-fn return. Killed by: Finished+non-empty → Some; Running+non-empty → None
  (kills the state check AND the `&&`→`||`); Finished+empty → None (kills the non-empty guard).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `rerun_command_finished_nonempty` — `open_running("ls")` then `state=Finished`; `rerun_command()` == `Some("ls")` | unit |
| REQ-002 | `rerun_command_running_is_none` — a `Running` block (not finished); `rerun_command()` == `None` | unit |
| REQ-003 | `rerun_command_empty_is_none` — `open_running("")` then `state=Finished`; `rerun_command()` == `None` | unit |
| REQ-004 | click ↻ (idle) → command re-runs as a new block; mid-command → no inject | shim + masked visual — chad-verified |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs ↻ affordance + click + `write_command` + the `is_command_running` guard (shim
exclude; needs a live pointer + PTY).

### Risks / decisions
- D-2.1 `rerun_command` returns `None` for non-`Finished` AND empty — both tested. The `&&` matters:
  the Running (non-empty) fixture kills a `&&`→`||` mutant. D-2.2 The session-idle guard
  (`!is_command_running`) is SHIM (live session state), so `rerun_command` stays a pure block method.
  D-2.3 The shim EXTRACTS the owned command (rerun_command clones) before `write_command(&mut)` — the
  immutable `blocks()` borrow ends first (no borrow conflict). D-2.4 `write_command` (writes `cmd\r\n`)
  reused from on_submit — the shell runs the resend as a new block.

## Phase 3 — Implement
- **Built (per manifest):** `block.rs` — `Block::rerun_command()` (`Some(command.clone())` iff
  `state == Finished && !command.is_empty()`, else None); `app.rs` — a `↻ run` glyph in the header
  affordance row (before the copy glyphs) whose click uses `state_mut(pane_id)`, GUARDS on
  `!is_command_running()`, EXTRACTS the owned command (`blocks().iter().nth(block_index).and_then(|b|
  b.rerun_command())`) so the immutable borrow ends, then `write_command(&cmd)` + `stop_propagation`.
  SPEC-terminal-blocks R30 + SPEC-app-shell R50 + row 50; CHANGELOG.
- **Deviations:** none. (The borrow-extract pattern from the design compiled first try — no borrow
  conflict between `blocks()` and `write_command`.)
- **Verification at this phase:** `cargo check --workspace` 0 err; fmt; clippy `-D warnings` 0 (marley +
  marley_terminal); docs 0. `rerun_command` is USED by the shim ↻ (live). Unit tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (a real scoped cargo-mutants + a full wiring/keymap/security trace). Verdict: pure
  slice SOUND (MSI 100 reachable), the `&&`→`||` worry is a NON-issue, the injection guard `!` is
  correct — but ONE real MED gap.
- **Mutants:** 32 on block.rs → 24 caught (pre-existing, no regression) / 2 unviable / 6 missed = the
  new `rerun_command` mutants (Phase-4-pending; all 6 have killers — T1+T3 cover them, incl.
  `&&`→`||` via T3 [Finished+empty under `||` → Some("") ≠ None]).
- **Findings:**
  | # | Sev | Finding | Verdict | Action |
  |---|---|---|---|---|
  | F1 | **MED** | cmd-R was promised in R50 + REQ-004 + the ticket TITLE + the spec, but ONLY the ↻ CLICK shipped — no keybinding, no dispatch arm, no "last finished" resolver. Not in the deferred list. The render fn is `mutants::skip` + REQ-004 is masked-visual → no gate would catch the omission. | REAL | **FIXED** — implemented cmd-R fully: a pure `BlockList::last_rerunnable()` resolver (reverse-scan for the first `Some` rerun_command), a `cmd-r`→`"rerun-last"` keymap binding (+ its `action_for` test assertion), and a `dispatch_action` arm (same #40-guarded extract-then-write). |
  | F2 | LOW | The plan's REQ-002 Running fixture wasn't pinned non-empty. | REAL (Phase-4) | P4 pins the Running fixture to a NON-empty command (independent doc of the Running-non-empty→None contract, though T3 already kills `&&`→`||`). |
- **Verified CORRECT:** the injection guard `if !is_command_running()` — the `!` is present + the write
  is INSIDE it, so a click/cmd-R while a command runs does NOTHING (no mid-command injection);
  extract-then-write borrow (owned `Option<String>` ends the `blocks()` borrow before `&mut
  write_command`) — `cargo check` 0 err; stale index → `.nth()` None → no write/panic; re-run resends
  the block's OWN recorded command (self.command from Preexec) — a verbatim replay, no NEW external-
  input injection surface; `rerun_command`/`last_rerunnable` have no unwrap/panic; clean-room.
- **Fix expanded the diff:** +`BlockList::last_rerunnable` (block.rs) + the keymap binding/test +
  the dispatch arm. Phase-4 adds the `rerun_command` (×3) AND `last_rerunnable` (×3) unit tests.

## Phase 4 — Validate
- **Tests added (block.rs):** `rerun_command_finished_nonempty` (Finished+"ls -la" → Some);
  `rerun_command_running_is_none` (Running + NON-EMPTY "ls -la" → None — F2, pins the `&&`);
  `rerun_command_empty_is_none` (Finished+"" → None); `last_rerunnable_finds_most_recent` (two
  finished → the LAST); `last_rerunnable_skips_running` (finished then running → the finished one);
  `last_rerunnable_none_when_no_finished` (only running → None). The keymap `action_for` "r"→"rerun-last"
  assertion was added in the inspect-fix.
- **Runs (actual):** `cargo nextest run -p marley_terminal` → 105 passed (all 6 new PASS);
  `--workspace` → 531 passed, 5 skipped.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **9 caught /
  0 missed → MSI 100.0%** (rerun_command + last_rerunnable + the keymap binding). Receipt written. No
  PTY hang.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement, updated for last_rerunnable);
  `terminal_blocks.md` — the re-run bullet. SPEC-terminal-blocks R30 (+ last_rerunnable) + SPEC-app-shell
  R50 at implement.
- **Knowledge captured:** failure `BF-claude-specified-keybinding-cmd-r-unimplemented-only-click-shipped`
  (3f0b067b, validation) + prevention rule
  `PR-claude-verify-every-specified-trigger-is-actually-wired-001` (medium) — when a spec names 2+
  triggers (click AND keybinding), the shim-only one can silently not-ship at MSI 100 (the shim is
  masked); inspect must diff the spec's named triggers against the actual keymap/dispatch/render
  wiring. aar-submit `completed` (score 5). Win: the critic caught a titled-but-absent feature (cmd-R)
  that every gate passed — then it was implemented fully with a tested `last_rerunnable` resolver.
- **Ticket:** forge #46 → done; local doc → closed/; pipeline pair archived. 4 of 6 in M1.G.
