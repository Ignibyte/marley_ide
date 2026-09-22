# Details dock — focused-block inspector — Notes

- **Forge ticket:** #58 `cbbd4790-ca3e-453f-8ea3-adf4e80afce8`
- **AAR:** `5c65c71b-8e99-43fb-82f4-2e5cfa4f9abd`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-058-block-details.md

## Phase 1 — Plan
- **Request:** forge #58 (M2.A seq-6, auto-approved) — the Details dock. **LAST M2.A ticket.**
- **Classification:** work pipeline, `feature`, PURE `block_details` (block_status.rs) + app.rs SHIM. UI
  — validate MUST self-test-capture.
- **Block model:** `Block { command: String, state: BlockState, exit_code: ExitCode(Option<i32>),
  prompt: PromptInfo { pwd: Option<String>, git_branch: Option<String> } }` (block.rs). `exit_status_kind
  (state, exit) -> StatusKind` + `status_indicator(kind, colors) -> (glyph, color)` already in
  block_status.rs (#36/#42) — reuse both.
- **Last-block access (shim):** `BlockList` has `len()`, `get(BlockIndex)`, `current()` (running only) —
  NO `last()`. The most-recent block = `get` of the last index (resolve the BlockIndex type at Design/
  Implement) or iterate; `current()` is the RUNNING block (None after a command finishes), so it's NOT
  the right accessor for "the command you just ran".
- **Hollow-MSI (#53):** block_details is 5 field-projections → few viable mutants; a behavioral test
  asserting all 5 fields (finished-success + running + no-pwd/branch) + coverage is the guard.
  block_status.rs already carries exit_status_kind/status_indicator viable mutants → not 0-viable.
- **AAR id:** `5c65c71b-8e99-43fb-82f4-2e5cfa4f9abd`.

## Phase 2 — Design

### DESIGN DEVIATION (from the ticket's `block_details(&Block)`)
`Block` has a PRIVATE `output` field and NO public cross-crate constructor, so a marley_app test cannot
build a `Block` — `block_details(&Block)` would be untestable. Also `BlockIndex(usize)`'s field is
private. So:
1. `block_details` takes the PUBLIC FIELDS it needs (fully testable, decoupled from Block internals).
2. The shim reads the last block via `blocks().iter().last()` (BlockIndex can't be constructed).

### PURE — ADD to `crates/marley_app/src/block_status.rs`
```rust
use marley_terminal::{Block, BlockState, ExitCode}; // Block only for the doc link / not constructed

/// The inspectable details of a command block, for the Details dock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDetails {
    pub command: String,
    pub status: StatusKind,
    pub exit_code: Option<i32>,
    pub pwd: Option<String>,
    pub git_branch: Option<String>,
}

/// Project a command block's fields into [`BlockDetails`] for the Details dock. Takes the fields
/// (not a `&Block`) because `Block` is not constructable outside `terminal_blocks` — this keeps the
/// projection unit-testable. `status` reuses [`exit_status_kind`] so the classification is single-sourced.
pub fn block_details(
    command: &str,
    state: BlockState,
    exit: ExitCode,
    pwd: Option<&str>,
    git_branch: Option<&str>,
) -> BlockDetails {
    BlockDetails {
        command: command.to_string(),
        status: exit_status_kind(state, exit),
        exit_code: exit.0,
        pwd: pwd.map(str::to_string),
        git_branch: git_branch.map(str::to_string),
    }
}
```
(If `use ...Block` triggers an unused-import warning, drop it — only BlockState/ExitCode are used.)

### SHIM — `app.rs` Right "Details" dock (mutants::skip + cov-excluded)
- Build the Right dock content from the focused session's LAST block:
  `self.workspace.focused_state().and_then(|s| s.session.blocks().iter().last())`. (Confirm an
  immutable `focused_state()` exists; else snapshot via the existing read path.)
- If `Some(block)`: `let d = block_details(&block.command, block.state, block.exit_code,
  block.prompt.pwd.as_deref(), block.prompt.git_branch.as_deref());` then render labeled rows: the
  `status_indicator(d.status, &colors)` glyph + `d.command`; an `exit {n}` / `running` line; `pwd`;
  `git {branch}`. If `None`: a muted "No command selected" placeholder.
- Pass this as the RIGHT dock's `dock_panel` content (was `div().flex_1().p_3()`).

### File manifest
- MODIFY `crates/marley_app/src/block_status.rs` — `BlockDetails` + `block_details` + tests.
- MODIFY `crates/marley_app/src/app.rs` — the Right-dock content builds + renders the last block's details.

### Mutation Targets
- Each field projection: `command.to_string()`, `status = exit_status_kind(state, exit)` (the wiring),
  `exit_code = exit.0`, `pwd.map`, `git_branch.map`. Hollow-MSI (#53): field projections yield few
  viable mutants — the behavioral tests assert ALL five fields with DISTINCT pwd/branch values (so a
  cross-wire would show) + `exit_status_kind`'s own mutants (already tested) live in the same file.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `details_finished_success` — `block_details("pwd", Finished, ExitCode(Some(0)), Some("/w"), Some("main"))` → command "pwd", status Success, exit_code Some(0), pwd Some("/w"), git_branch Some("main") | unit |
| REQ-002 | `details_finished_failure` — `(…, Finished, ExitCode(Some(2)), …)` → status Failure, exit_code Some(2) | unit |
| REQ-003 | `details_running` — `(…, Running, ExitCode(None), …)` → status Running, exit_code None | unit |
| REQ-004 | `details_no_pwd_no_branch` — pwd None + git_branch None → both None (no fabrication) | unit |
| REQ-005 | Right Details dock shows the command + status + cwd | self-test (drive a command → capture) |
| REQ-006 | gate GREEN, cov/MSI 100 block_details; app shim excluded | gate |

Uncoverable: the app.rs Right-dock render — masked + cov-excluded, proven by REQ-005.

### Risks / decisions
- D-2.1 fields-not-`&Block` signature (Block unconstructable) — documented deviation. D-2.2
  `iter().last()` for the most-recent block (BlockIndex unconstructable; `current()` is running-only,
  wrong for a finished command). D-2.3 distinct pwd/branch test values so a projection cross-wire is
  caught (hollow-MSI mitigation). D-2.4 confirm `focused_state()` (immutable) exists for the render read.

## Phase 3 — Implement
- **Built (PURE, block_status.rs):** `BlockDetails` (Debug/Clone/Eq, NO Default) + `block_details(command,
  state, exit, pwd, git_branch)` projecting the five fields (status via `exit_status_kind`) — verbatim
  from the design (fields, not `&Block`).
- **Built (SHIM, app.rs — mutants::skip/cov-excluded):** the Right "Details" dock reads the focused
  session's last block via `self.workspace.state(self.workspace.focused()).and_then(|s|
  s.session.blocks().iter().last())`; renders `block_details` as labeled rows (the `status_indicator`
  glyph + command; `exit N`/`running`; pwd; `git {branch}`); a muted "No command selected" placeholder
  when there are no blocks. Passed as the Right dock's `dock_panel` content. Import gained `block_details`.
- **Deviations:** the immutable accessor is `workspace.state(pane)` (not the planned `focused_state()`,
  which doesn't exist — only `focused_state_mut`); the git branch shows as `git {branch}` (font-safe
  text, no powerline glyph).
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; `cargo clippy -p marley -- -D warnings`
  OK; `cargo nextest -p marley` 119 pass (no regression). block_details tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (11/11 probe + a clean `-j1` cargo-mutants + a rustc type-proof + a shim trace). Verdict:
  **SHIP-able — pure projection correct.** block_details = 5 fields cleanly wired (probe confirms).
- **Findings:**
  | # | Sev | Finding | Verdict | Action |
  |---|---|---|---|---|
  | F1 | MED | HOLLOW-MSI cross-wire: block_details has 0 VIABLE mutants (its only mutant is the unviable `→Default::default()`), so MSI 100 is vacuous — the ONLY guard against a pwd↔git_branch swap / wrong-source field is the Phase-4 test ASSERTIONS. The file isn't 0-viable (exit_status_kind carries 3/3). | REAL (test-adequacy) | P4 tests MUST use DISTINCT values (command≠pwd≠git_branch, e.g. `"pwd"/"/w"/"main"`) + assert each field + a None/None case (design D-2.3). |
  | F2 | MED | REAL shim bug: `exit_status_kind(Finished, None) = Failure` → ✗ glyph, but the shim's `status_line = match d.exit_code { None => "running" }` shows "running" TEXT → a ✗-glyph-vs-"running"-text MISMATCH for a signal-killed / no-code block. Uncaught by gates (app.rs excluded) + the exit-0 self-test. | REAL | **P4 fix (app.rs):** derive status_line from `(d.status, d.exit_code)` — `Some(c)→"exit {c}"`, `(Running,None)→"running"`, `(_,None)→"no exit code"`. |
  | F3 | LOW | `blocks().iter().last()` is O(n)/frame (BlockList has no O(1) last). | Accept | cov-excluded shim, n bounded by scrollback. A `BlockList::last()` is an optional later improvement (terminal_blocks change). |
- **Verified (probe):** all 5 fields correctly wired + distinct; `status = exit_status_kind` (single-
  sourced); `exit_code = exit.0` RAW (independent of status — Running+Some(7)→status Running/code
  Some(7)); NO Default keeps the whole-body mutant unviable; `iter().last()` = most-recent (correct vs
  `current()`); immutable `workspace.state(focused())`; placeholder on empty; no panic; app.rs cov-
  excluded + render mutants::skip. File MSI 3/3 (block_details 0-viable but file not 0-viable).
- **Code fix deferred to P4** (app.rs edit gated to validate): F2 status_line. F1 = P4 distinct-value tests.

## Phase 4 — Validate
- **F2 fix applied (app.rs):** the Details status line now derives from `(d.status, d.exit_code)` —
  `Some(c)→"exit {c}"`, `(Running,None)→"running"`, `(_,None)→"no exit code"` — so a signal-killed
  Finished block no longer reads "running" beside the ✗ glyph.
- **Tests added** (`block_status.rs`, F1 — DISTINCT command/pwd/branch values so a cross-wire fails):
  `details_finished_success` (REQ-001), `details_finished_failure` (REQ-002), `details_running_has_no_code`
  (REQ-003), `details_no_pwd_no_branch_are_none` (REQ-004).
- **Runs (actual):** `cargo nextest -p marley -E 'test(details)'` → 4 passed; workspace 158 passed.
- **SELF-TEST (UI — REQ-005, drove the LIVE app):** rebuilt + launched the bare binary from the project
  dir; drove `echo hi` → the Right "Details" dock rendered **`✓ echo hi` (green Success glyph), `exit 0`,
  and the cwd `/Users/chadpeppers/Projects/ignibyte/Marley`** (`scratchpad/details.png`). The whole M2.A
  workspace is now populated: Files tree ← terminal → Details inspector. The F2 fix is live ("exit 0").
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, MSI 100.0%. app.rs shim
  excluded as designed; block_details' vacuous-MSI mitigated by the distinct-value assertions (F1).
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (M2.A COMPLETE); `marley_search_core.md` M2.A-workspace section.
- **Knowledge:** failure `BF-claude-details-status-line-keyed-off-exit-code-not-status` (the ✗-glyph vs
  "running"-text mismatch — derive both from the same status source). aar-submit `completed` (5).
- **Ticket:** forge #58 → done; local doc → closed/; pair archived.
- **SPRINT M2.A CLOSED** (#9 `fcb8d0fe`) — all 6 tickets (#53 project · #54 search_core · #55 file
  listing · #56 file tree · #57 fuzzy-open · #58 details) delivered at cov/MSI 100 + gate GREEN. The
  moat foundation is in: Files tree ← terminal → Details inspector, with cmd-P quick-open. **6/6.**
