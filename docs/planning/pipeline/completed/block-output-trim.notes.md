# block output trims trailing blank rows — Notes

- **Forge ticket:** #50 `9cdc30e8-2f65-4a07-bbf1-fdc55b8ae936`
- **AAR:** `d60e56c1-ea66-4078-8a9c-0e6104a2e596`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-050-block-output-trim.md
- **Pipeline spec:** block-output-trim.spec.md

## Phase 1 — Plan
- **Request:** forge #50 (M1.H "Real Terminal Feel" seq-2) — the CRITICAL "every command clears the
  previous" bug. Blocks capture the full screen-height grid → each is a full-screen block → history
  doesn't stack.
- **Classification / tier:** work pipeline, `bug` (critical), a PURE fn (styled.rs — cov/MSI 100) + a
  one-line session wiring. terminal_blocks.
- **Root cause (confirmed, §18):** `session.rs:337-338` `set_current_output(term_to_styled_rows(
  &self.term))`; `term_to_styled_rows` (session.rs:396) iterates `0..grid.screen_lines()` pushing a row
  for EACH (coalesce_row trims trailing spaces per row, NOT blank rows) → a block's output = the full
  grid height incl. trailing blanks. `grid_styled_rows()` (session.rs:188) shares `term_to_styled_rows`
  but is the ALT-SCREEN path (must stay full). Capture (marley_history.png): `ls` output + prompt, no
  header, no `pwd` — one full-screen block dominating the ~48-row viewport.
- **Fix:** trim trailing blank rows on the block-output path only (a pure `trim_trailing_blank_rows`
  in styled.rs). `StyledLine = Vec<StyledRun{text}>` (styled.rs:27); a blank row = all runs trim-empty.
- **Decisions:** D1–D4 in the spec (trailing-only; block-path-only; blank predicate; pure surface).
- **Open for Design:** the blank predicate (`text.trim().is_empty()` per run — covers empty runs +
  whitespace); whether an empty `Vec<StyledLine>` block output renders as header-only (yes — correct);
  the session test (feed a Preexec + one output line + no trailing content → block output = [that line]).
- **AAR id:** `d60e56c1-ea66-4078-8a9c-0e6104a2e596`.

## Phase 2 — Design

### PURE — `crates/terminal_blocks/src/styled.rs`
```rust
/// Drop the TRAILING all-blank rows from a block's captured output (R19). The grid snapshot is
/// screen-height, so a short command leaves many empty rows below its output; trimming them keeps a
/// block only as tall as its real output, so command blocks STACK as scrollback (not one full-screen
/// block per command). Interior blank rows are KEPT; an all-blank/empty input → `[]`. NOT applied to
/// the alt-screen grid (`grid_styled_rows`) — a full-screen program owns its whole screen.
pub fn trim_trailing_blank_rows(mut rows: Vec<StyledLine>) -> Vec<StyledLine> {
    while rows
        .last()
        .is_some_and(|row| row.iter().all(|run| run.text.trim().is_empty()))
    {
        rows.pop();
    }
    rows
}
```
(`.all(..)` on an empty row [no runs] is vacuously true → blank; `is_some_and` on `[]` is false → stop
→ returns `[]`. Interior blanks survive because we only pop from the END.)

### WIRING — `crates/terminal_blocks/src/session.rs` (`ingest`, the Passthrough arm)
```rust
self.model.set_current_output(trim_trailing_blank_rows(term_to_styled_rows(&self.term)));
```
`grid_styled_rows()` (alt-screen) stays `term_to_styled_rows(&self.term)` — UNTRIMMED.

### File manifest
- M `crates/terminal_blocks/src/styled.rs` — `trim_trailing_blank_rows` + tests.
- M `crates/terminal_blocks/src/session.rs` — the `ingest` Passthrough wrap; UPDATE any existing block
  test that asserted the full-grid `output_styled()` length (e.g. `pump_renders_multi_row_output`,
  `pump_two_commands_reset_render_grid_between_blocks`) — they should now expect trimmed output
  (`output_text()` is unchanged since it already `.trim_end()`s, so text-based asserts still pass).
- M `docs/specs/SPEC-terminal-blocks.spec.md` — R19 note. CHANGELOG; arch.

### Mutation Targets
- `trim_trailing_blank_rows` — the blank predicate `run.text.trim().is_empty()` (a mutant → wrong
  blank detection); the `.all(..)` (any→all); the `while … pop()` trailing-only loop (stops at the
  first non-blank from the end); the `is_some_and` empty guard. Killed by: trailing-trimmed + interior-
  kept + all-blank→[] + no-trailing-unchanged + whitespace-row fixtures.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `trim_trailing_blank_rows_trailing_and_interior` — `[a, "", b, "", ""]` → `[a, "", b]` (trailing dropped, interior kept); a whitespace-only row counts as blank | unit (styled.rs) |
| REQ-002 | `trim_trailing_blank_rows_edges` — all-blank `["", ""]` → `[]`; empty `[]` → `[]`; no-trailing `[a, b]` → `[a, b]` | unit |
| REQ-003 | a session pump test: feed a `Preexec` + one output line (+ the grid's trailing blanks) → the block's `output_styled()` has NO trailing blank rows (its len == the real output lines) | session unit |
| REQ-004 | alt-screen `grid_styled_rows()` returns the full grid — reasoning (separate path, untrimmed) + the existing alt-screen test stays green | build + existing test |
| REQ-005 | `scripts/gates.sh --diff` GREEN (cov/MSI 100 on the pure fn) + rebuild + window capture: two commands STACK as separate blocks | gate + capture |

Uncoverable by unit tests: the final "two commands stack visually" (needs a live pane + chad typing) —
captured via `screencapture` after chad types (the AX-verify step).

### Risks / decisions
- D-2.1 Trailing-only (interior blanks kept) — a command with a mid-output blank line keeps it. D-2.2
  Block-path only; `grid_styled_rows` (alt-screen) untouched. D-2.3 May need to update existing
  `output_styled()`-length asserts in session.rs tests (text asserts unaffected — `output_text` already
  trims). D-2.4 Long output (> grid) still only captures the last screen — a separate scrollback-capture
  concern, out of scope; this fixes the common short-command stacking.

## Phase 3 — Implement
- **Built:** `styled.rs` — `trim_trailing_blank_rows` (while-pop trailing rows whose runs are all
  `text.trim().is_empty()`; a blank grid line coalesces to an EMPTY StyledLine, so the predicate is
  vacuously true on it). `session.rs` — the `ingest` Passthrough arm wraps the block-output set:
  `set_current_output(trim_trailing_blank_rows(term_to_styled_rows(&self.term)))`; import added.
  `grid_styled_rows()` UNCHANGED (alt-screen keeps the full grid). SPEC-terminal-blocks R19 amended;
  CHANGELOG `### Fixed`.
- **Deviations:** none. **No existing test broke** — all 105 terminal_blocks tests pass (the pump/block
  tests assert via `output_text()` which already trims, or their small-grid output had no trailing
  blanks), so no expectation updates were needed.
- **Verification at this phase:** `cargo check -p marley_terminal` 0 err; fmt; 105 tests pass. Unit
  tests for `trim_trailing_blank_rows` + the session no-trailing-blanks test are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (a 15-case verbatim probe + a real scoped cargo-mutants + a wiring/regression trace).
  Verdict: **code CORRECT, wiring SAFE, no regression** — one MED test-adequacy finding.
- **Findings:**
  | # | Sev | Finding | Verdict | Action |
  |---|---|---|---|---|
  | F1 | MED (test-adequacy) | VACUOUS MSI: cargo-mutants emits only 2 whole-body mutants for `trim_trailing_blank_rows` (both "return empty"), killed by the pre-existing pump tests that assert `output_text()` — which independently `.trim_end()`s, so it CANNOT distinguish trimmed from untrimmed rows. So MSI 100 holds even for an IDENTITY fn; no test exercises the trailing/interior/blank semantics OR the real effect (`output_styled().len()` collapsing). Reverting the trim would keep all tests green. | REAL (the tests are the guard) | P4 MUST write direct `trim_trailing_blank_rows` unit tests (trailing-dropped, interior-KEPT, all-blank→[], empty→[], no-trailing unchanged) + a session/block test asserting `output_styled()` has no trailing blanks (the real effect). Same family as the prior MSI-blind lessons. |
- **Verified CORRECT (probe + trace):** interior blanks KEPT (`[a,[],b,[],[]]`→`[a,[],b]`), trailing-only,
  per-run `.all()` (any non-blank run → row kept); empty-row vacuous-true is intended (coalesce makes
  blank lines empty StyledLines); terminates (pop shrinks), no panic; the alt-screen `grid_styled_rows`
  is UNTRIMMED (session.rs:189, gated by `is_alt_screen`) — a full-screen program keeps its full grid;
  `set_current_output` only sets output on `current_mut()`, never drops the block → `is_command_running`
  (reads block STATE) UNAFFECTED; `output_styled` + `output_text` are now CONSISTENT (both trailing-trim
  — a fix, not a loss). MSI 100 on styled.rs (11/11).
- **No code fix** — F1 is the Phase-4 direct tests.

## Phase 4 — Validate
- **Tests added:** `styled.rs` — `trim_trailing_blank_rows_drops_trailing_keeps_interior` (the F1
  guard: `[a,"",b,"",""]`→`[a,"",b]` — interior kept, trailing dropped; whitespace rows blank) +
  `trim_trailing_blank_rows_edges` (all-blank→[], empty→[], no-trailing unchanged). `session.rs` —
  `pump_block_output_trims_the_grids_trailing_blank_rows` (a 2-line command in the 24-row grid →
  `output_styled().len() == 2`, NOT 24 — kills the identity/no-trim regression that `output_text`
  can't catch).
- **Runs (actual):** `cargo nextest run -p marley_terminal` → 108 passed (all 3 new PASS);
  `--workspace` → 542 passed, 5 skipped.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **3 caught /
  0 missed → MSI 100.0%**. Receipt written. No PTY hang.
- **Rebuilt + relaunched** the fixed build (`open`, PID survives) for chad's live re-test.
- **Chad-verify pending:** the live "two commands STACK as separate blocks" (the mechanism is proven
  by the unit + session tests; the visual stacking needs chad to type 2 commands — I can't drive input).
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Fixed` (at implement); `terminal_blocks.md` — the block-output-trim
  bullet. SPEC-terminal-blocks R19 amended.
- **Knowledge captured:** prevention rule
  `PR-claude-assert-the-raw-representation-not-a-normalizing-projection-001` (medium) — when a fix
  changes a raw repr (output_styled) but a projection normalizes it away (output_text `.trim_end()`),
  tests over the projection can't distinguish the fix → vacuous MSI; assert the raw repr directly.
  aar-submit `completed` (score 5). Win: root-caused a CORE "not a terminal" bug (each block = a
  full-screen grid snapshot) via a window capture + code read, fixed it with a tiny pure fn, and the
  inspect caught that the gate was vacuously green (the direct output_styled().len() test is the real
  guard).
- **Ticket:** forge #50 → done; local doc → closed/; pipeline pair archived. Sprint #8 M1.H stays OPEN.
