# Shell hooks from alacritty's event loop — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-462-shell-hooks-from-the-event-loop.md
- **Pipeline spec:** 462-shell-hooks-from-the-event-loop.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (Chad: "ok lets go for it").
- **Request:** T0, continued after #461.
- **Split.** #462 as first written also held Zed's side. That side is now #464, queued ahead
  of #463.
- **Recall (§18.3).**
  - `PR-claude-ordered-events-coalesced-stream-hooks-001`: one ordered event stream, processed
    in order, or a coalesced read loses its output.
  - #461's note: a standalone test run writes the copy's `Cargo.lock`, so commit it before the
    gate runs those tests.
  - Brain: consultation `f83171b55e5b42c299db6b4f1aa0a879` closed at #461; nothing further.
- **Discovery** (#461's Explore sweeps, re-read here).
  - The scanner (`dcs.rs:293-412`) is self-contained. Only the decoder needs `block.rs`'s
    types, and `marley_terminal` depends on `alacritty_terminal`, hence the leaf crate.
  - `pty_read` parses `buf[..unprocessed]` under the lock (`event_loop.rs:154`), and `State`
    holds the parser (`:401-405`).
  - Lines leave the grid in `scroll_up` when `region.start == 0` and the history is full: the
    growth is `min(positions, max_scroll_limit - history_size)`, and the rest is evicted. They
    also leave in `clear_history` and `update_history`.
  - `Grid` derives serde for the ref fixtures, so the new field needs `serde(default)`.
  - `Event` is `Clone`, with a hand-written `Debug`. Zed's `From<AlacTermEvent>`
    (`crates/terminal/src/alacritty.rs:302-324`) is exhaustive.

### Design
- **`crates/marley_dcs`** (`src/marley_dcs.rs`): `DcsScanner::feed(&mut self, &[u8]) ->
  Vec<DcsEvent>`, `DcsEvent::{Passthrough(Vec<u8>), Hook(RawDcs)}`, `RawDcs { final_byte,
  payload }`, `HOOK_SELECTORS`, and the payload cap (64 KiB). A new state forwards the rest
  of a frame that is not Marley's until ST.
- **`marley_terminal`**: `dcs.rs` drops its scanner and `session.rs` uses `marley_dcs`'s. The
  scanner's tests move with it.
- **Vendored:**
  - `marley_hooks.rs`: `ShellHook`, `HookPosition::{of, absolute_line}`, and
    `advance_with_hooks<T: EventListener>(parser, scanner, term, bytes)`;
  - `lib.rs`: `pub mod marley_hooks`;
  - `event.rs`: `ShellHook(marley_hooks::ShellHook)` and its `Debug` arm;
  - `event_loop.rs`: `State.hooks`, and the call;
  - `grid/mod.rs`: `evicted_lines`, `evicted_lines()`, and the three increments;
  - `Cargo.toml`: `marley_dcs = { path = "../../crates/marley_dcs" }`.
- **Zed:** `TerminalBackendEvent::ShellHook(ShellHook)`, the `From` arm, and a
  `process_event` arm that ignores it, with rows in `zed-touchpoints.md` first.
- **Gate:** gate:3 also runs `cargo test --locked --manifest-path vendor/<crate>/Cargo.toml`
  for each vendored crate.
- **File manifest.**
  - New: `crates/marley_dcs/**` and `vendor/alacritty_terminal/src/marley_hooks.rs`.
  - Changed: `vendor/alacritty_terminal/{Cargo.toml, Cargo.lock, src/lib.rs, src/event.rs,
    src/event_loop.rs, src/grid/mod.rs, src/grid/tests.rs}`,
    `crates/marley_terminal/{Cargo.toml, src/dcs.rs, src/session.rs}`,
    `crates/terminal/src/{alacritty.rs, terminal.rs}`, the root `Cargo.toml` (member),
    `Cargo.lock`, `script/gates.sh`, `vendor/README.md` and `docs/marley/zed-touchpoints.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | vendored `marley_hooks` tests: a `Term` with a recording listener, fed one coalesced buffer; hook order, positions bracketing the "hi" line, and the grid holding "hi" |
| 002 | vendored: a frame split across two `advance_with_hooks` calls gives one hook, positioned at the second |
| 003 | `marley_dcs`: selector passthrough, CAN and SUB, the cap, split frames; the moved tests; 100% lines |
| 004 | vendored `grid` tests: eviction on full-history scrolls, `clear_history`, `update_history`, and a JSON without the field deserializing to 0 |
| 005 | `cargo nextest run -p terminal -p marley_terminal -p marley_dcs` |
| 006 | `script/gates.sh --diff`, whose gate:3 runs the copy's tests |

Negative checks: hooks emitted after the whole buffer is parsed (REQ-001 fails); no eviction
count on a full-history scroll (REQ-004 fails); a foreign selector taken as a hook (REQ-003
fails).

### Risks
- **A path dependency from the standalone copy into a workspace crate.** `marley_dcs` inherits
  workspace fields, so its manifest must resolve when the copy builds on its own. The first
  standalone build answers it.
- **Synchronized updates** (`CSI ? 2026 h`) buffer bytes inside vte, so a position taken during
  one can lag. Shells do not wrap prompts in them.

## Phase 2 — Code (2026-09-23)
- **Checklist:** [x] `marley_dcs` · [x] the `marley_terminal` switch · [x] the vendored hunks ·
  [x] the Zed mirror, rows first · [x] the gate step · [x] fmt and clippy.
- **Built.**
  - `crates/marley_dcs`: the scanner, public, with no dependencies. Marley frames are collected
    as before. Every other DCS is forwarded in a `Foreign` state until the parser would end it
    (ESC, CAN or SUB). CAN and SUB cancel a Marley frame and pass its bytes through. A payload
    past `MAX_PAYLOAD` (64 KiB) passes through, and the next frame is still found. 12 tests.
  - `marley_terminal`:
    - `dcs.rs` keeps the decoder and gains `decode_frame(&RawDcs)`. With foreign selectors
      filtered by the scanner, `session.rs`'s separate selector check could never fail, and its
      `continue` would have been an uncovered line.
    - `session.rs` takes the scanner from `marley_dcs`.
    - `decode_frame` is exported for #464.
  - Vendored `alacritty_terminal`:
    - `src/marley_hooks.rs` (`ShellHook`, `HookPosition::{of, absolute_line}`,
      `advance_with_hooks`, 5 tests).
    - `lib.rs`, `event.rs` (variant and `Debug`), `event_loop.rs` (`State.hooks`, and the
      call with the loop's own listener).
    - `grid/mod.rs`: `evicted_lines` with `serde(default)`, the accessor, and three
      increments. `grid/tests.rs` has 2 tests.
    - `Cargo.toml` gets the path dependency, and `Cargo.lock` is committed for the standalone
      tests.
  - Zed: the two ledger rows first, then `TerminalBackendEvent::ShellHook`, its `Debug` arm,
    the `From` arm, and a `process_event` arm that ignores it until #464.
  - `script/gates.sh` gate:3 runs `cargo test --locked --manifest-path vendor/<crate>/Cargo.toml`
    for each copy. The CONSTITUTION's gate table row and the `vendor/` known-scope bullet say
    so.
- **Deviations:** `decode_frame`, above. clippy's `byte_char_slices` turned `HOOK_SELECTORS`
  into `*b"hpq"`.
- **The design's risk, answered:** the copy's standalone build resolves its path dependency into
  the workspace crate `marley_dcs`, workspace-inherited fields and all.

## Phase 3 — Test (2026-09-23)
- **REQ-001, REQ-002:** vendored `marley_hooks` tests over a real `Term`:
  - `one_read_with_a_whole_command_reports_both_hooks_around_its_output`: absolute lines 0 and
    1, with "hi" on line 0;
  - `a_frame_split_across_reads_is_reported_once_where_it_completed`;
  - `positions_count_the_lines_the_history_dropped`: absolute line 4 with 2 lines evicted;
  - `another_programs_dcs_reaches_the_parser`;
  - `a_hook_on_the_alternate_screen_says_so`.
- **REQ-003:** 12 `marley_dcs` tests: every Marley selector, stream order, every split point
  of a read, foreign DCS ended by ESC, CAN or SUB, cancel bytes at each place in a frame, an
  escape in place of the selector, and the cap on both sides.
- **REQ-004:** `evicted_lines_count_what_the_history_drops` (`scroll_up`, a region below the
  top, `update_history`, `clear_history`) and
  `a_grid_saved_without_the_counter_loads_with_nothing_evicted`.
- **REQ-005:** `cargo nextest run -p terminal -p marley_terminal -p marley_dcs`: 267 passed. The
  copy standalone: 145 unit tests and 1 doctest.
- **Negative checks** (restored by sha256 checksum):
  - N1, hooks sent after the whole read is parsed: the coalesced-read test fails.
  - N2, no eviction count in `scroll_up`: the grid counter test and the positions test fail.
  - N3, any selector but ESC, CAN and SUB taken as Marley's: two `marley_dcs` tests and the
    copy's passthrough test fail.
- **Live drive:** N/A. Zed ignores the event until #464.
- **Gate:** `script/gates.sh --diff`, scope the Marley crates plus `terminal`, `marley_dcs` and
  `marley_terminal`: GATE GREEN [diff], 20 of 20. 552 tests ran over the scope, and gate:3's new
  step ran the copy's 145 unit tests and 1 doctest. Coverage was 100% of lines (2907) and
  functions (297). The receipt matches.
- **Pre-existing:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist:** [x] document · [x] knowledge · [x] close the ticket · [x] archive · [x] commit.
- **Docs:**
  - `CHANGELOG.md` (Added).
  - `docs/marley_architecture/marley_dcs.md`, new.
  - `docs/marley_architecture/terminal_blocks.md`: where the scanner lives now.
  - `docs/marley/three-prong-plan.md`: D1's scanner and the T0 row.
  - `docs/marley/README.md`: the fork crates' notes.
  - `vendor/README.md`: every hunk.
  - `CONSTITUTION.md`: gate:3's row, and the `vendor/` bullet.
  - The two new ledger rows.
- **Knowledge:** `AD-claude-462-shell-hooks-leave-the-stream-in-the-event-loop-001`. No `F-`
  block: no bug found.
- **Brain:** consultation `ba54a471b6134d319ce545f40f63a828` closed with a decision, follow-up
  by 2026-10-07.
- **Ticket:** #462 closed. #464 is next in the queue, then #463.
