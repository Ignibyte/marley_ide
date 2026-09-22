# capture full command output beyond the grid (block scrollback) — Notes

- **Forge ticket:** #52 `bd410f09-cc72-4bb4-860f-4a23cea5fcb3` · **AAR:** `5b95f7a9-f442-4085-974a-ff0f49c7a764`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-052-block-scrollback.md

## Phase 1 — Plan
- **Request:** forge #52 (Terminal Polish 4/15) — the #50-found limitation: output beyond the grid is lost.
- **Classification:** work pipeline, `bug`, terminal_blocks/session.rs. Integration-tested.
- **Root cause:** `term_to_styled_rows` reads only `0..screen_lines` (visible); the ingest snapshots it as
  the block output. But `Config::default().scrolling_history = 10000` → alacritty KEEPS the scrolled-off
  lines in history; we just never read them.
- **Decisions:** D1 full-capture ONCE at Precmd (bounded, no quadratic); D2 read `-(history)..screen`; D3
  capture BEFORE apply_hook(Precmd) closes the block + trim (#50).
- **Risk:** confirm `grid.history_size()` is public + negative `Line` indexing reads history (verify at
  implement via cargo check). alacritty_terminal 0.26.
- **AAR id:** `5b95f7a9-f442-4085-974a-ff0f49c7a764`.

## Phase 2 — Design

### `session.rs`
```rust
/// The FULL command output — the grid's HISTORY region (scrolled-off lines) + the visible screen (#52).
/// alacritty keeps the scrolled-off lines (Config `scrolling_history: 10000`); we read them via the grid's
/// negative `Line` indices. `history = total_lines - screen_lines` (Dimensions).
fn full_term_to_styled_rows(term: &Term<VoidListener>) -> Vec<StyledLine> {
    let grid = term.grid();
    let cols = grid.columns();
    let screen = grid.screen_lines() as i32;
    let history = (grid.total_lines() - grid.screen_lines()) as i32;
    let mut rows = Vec::with_capacity((history + screen) as usize);
    for line in -history..screen {
        let row = &grid[Line(line)];
        rows.push(coalesce_row(
            (0..cols).map(|col| {
                let cell = &row[Column(col)];
                (cell.c, cell.fg, cell.bg, cell.flags)
            }),
        ));
    }
    rows
}
```
- `ingest` Hook arm — add a Precmd full-capture BEFORE `apply_hook` closes the block:
  ```rust
  if matches!(hook, DcsHook::Preexec(_)) {
      self.reset_term();
  }
  if matches!(hook, DcsHook::Precmd(_)) {
      // #52: the FINISHED block gets the complete scrollback (one bounded full read at finish).
      self.model
          .set_current_output(trim_trailing_blank_rows(full_term_to_styled_rows(&self.term)));
  }
  let _ = self.model.apply_hook(hook);
  ```
- `total_lines`/`screen_lines` are the `Dimensions` trait (public); `grid[Line(neg)]` reads history. (If
  `total_lines` isn't in scope, `use alacritty_terminal::grid::Dimensions` — already imported at :14.)

### File manifest
- MODIFY `crates/terminal_blocks/src/session.rs` — add `full_term_to_styled_rows`; the Precmd capture in ingest.

### Mutation Targets
- `full_term_to_styled_rows`: the `-history..screen` range (both bounds), the `total_lines - screen_lines`.
- ingest: the `matches!(hook, DcsHook::Precmd(_))` branch.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `ingest_keeps_output_beyond_the_grid` — a MockPtyChannel scripts `[Preexec]` + N (>screen_lines) distinct rows + `[Precmd]`; pump; the finished block's `output_styled().len()` == N (all rows, incl. scrolled-off), trailing blanks trimmed, first + last row present | integration (session) |
| REQ-002 | `ingest_fitting_output_unchanged` — output < screen_lines → the block has exactly those rows (no regression) | integration |
| REQ-003 | gate GREEN, cov/MSI 100 | gate |

Uncoverable: none new — the grid-read + ingest are integration-covered via the MockPtyChannel seam (the
existing test pattern). Build the DCS Preexec/Precmd frames as the existing session tests do.

### Risks / decisions
- D-2.1 the full read is at Precmd ONLY (bounded) — a running block keeps the per-Passthrough visible
  snapshot (unchanged). D-2.2 `history = total_lines - screen_lines` (robust vs a maybe-private
  `history_size()`); `-history..screen` covers the scrollback + screen. D-2.3 verify negative-Line
  indexing compiles + reads history (cargo check + the integration test at implement). D-2.4 the Precmd
  capture writes the RUNNING block, so it MUST precede `apply_hook(Precmd)` (which closes it).

## Phase 3 — Implement
- **Built (session.rs):** `full_term_to_styled_rows` (reads `-history..screen`, history = total_lines -
  screen_lines) + the ingest Precmd branch capturing the trimmed full output BEFORE `apply_hook` closes
  the block.
- **Verification:** `cargo fmt`; `cargo check -p marley_terminal` 0 err (total_lines() + negative Line
  indexing compile — the alacritty grid API works as designed); clippy `-D warnings` OK. The integration
  test (Phase 4) is the correctness proof (real Term feeding >screen output).

## Phase 3.5 — Inspect
- **Critic:** 1 (correctness; wrote + ran a real-Term probe + cargo-mutants). Verdict: **PASS — the fix is
  correct, no HIGH/MED defect.**
- **Confirmations:** (a) the `-history..screen` range captures ALL scrolled output — probe: 5-line screen +
  20 fed lines → `full_term_to_styled_rows` returns exactly 20 (L01..L20, oldest first, no dup/skip at the
  history/screen boundary) vs `term_to_styled_rows` returning only L16..L20; it's identical to alacritty's
  own `topmost_line()..=bottommost_line()`. (b) alacritty GENUINELY keeps the history (total_lines 20 >
  screen 5; the fix is real, not a no-op); the history==0 case → `-0..screen` == `0..screen` (no
  regression). (c) the Precmd capture lands on the still-RUNNING block once at finish (before apply_hook
  flips it Finished), no quadratic. (d) no panic (total≥screen invariant holds; debug-asserts didn't fire).
  (e) **the planned >screen test is REQUIRED for MSI 100** — the `delete -` mutant SURVIVES the original
  suite (≤screen tests can't distinguish it); the new test kills it. 6/6 new-code mutants caught with it.
- **Findings + actions:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | L1 | LOW | The child-exit finish path (`poll_child_exit → finish_current_if_running`) skipped the full capture → a block finished by the shell dying mid-command lost scrollback (inconsistent with Precmd). | **FIXED** — full-capture before `finish_current_if_running` too. |
  | L2 | LOW/nit | `total_lines - screen_lines` relied on an implicit invariant. | **FIXED** — use `grid.history_size()` (saturating, self-documenting). |
  | I1 | INFO | "Full" = up to 10000 (scrolling_history) + screen; older overflow is ring-evicted (by design). | Documented. |
  | I2 | INFO | A running block is screen-bounded until finish (the full output materializes at Precmd). | Documented design tradeoff. |
- **Fix applied:** L1 + L2.

## Phase 4 — Validate
- (pending)

## Phase 5 — Complete
- (pending)

## Phase 4 — Validate
- **Tests added (session.rs):** `ingest_keeps_output_beyond_the_grid` (REQ-001 — feed 40 rows to a 24-row screen + Precmd → the finished block keeps all 40, L01 first [scrolled off], L40 last; kills the `delete -` mutant the original suite missed) + `ingest_fitting_output_unchanged` (REQ-002 — ≤screen output unchanged).
- **Runs (actual):** `cargo nextest -p marley_terminal -E ...` → 2 passed.
- **Self-test (drive `seq 200` + scroll):** ENV-BLOCKED (typing) → the integration test (real Term, real scroll into history) IS the correctness proof (feeds >screen output, asserts the scrolled-off rows survive).
- **Gate:** (running).
- **Pre-existing:** none.

## Phase 5 — Complete
- CHANGELOG ### Fixed; aar-submit(5); PR-claude-history-range-read-needs-over-screen-test-001 (MSI 100 does not prove a scrollback read — needs a >screen test; the delete- mutant survives a ≤screen suite). forge #52 → done; **#51 → done (superseded by #89)**. **Terminal Polish 4/4 — POLISH COMPLETE.** Also closes the last M1.H open tickets. Critic PASS (probe: 20/20 rows, alacritty history real); 2 LOW fixed (child-exit full-capture + history_size).
