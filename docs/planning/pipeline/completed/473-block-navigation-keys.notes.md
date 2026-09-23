# Block navigation keys — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-473-block-navigation-keys.md
- **Pipeline spec:** 473-block-navigation-keys.spec.md

## Phase 1 — Plan (2026-09-23)
- **Request:** the plan's T1, "block navigation keys", after T1a (#470); Chad's goal "lets
  continue on those".
- **Recall.** AD-claude-449 (catch actions at the workspace's root; the Marley keymap takes only
  keys with no Zed action behind them, and `secondary-up`/`secondary-down` have none in
  `Terminal`); #470's absolute lines.
- **Design.** `block_scroll` in `anchored.rs`; `crates/marley_workbench/src/blocks.rs` with the
  two handlers, installed by `init` through `register_action_renderer`; the focused view found
  among the center's terminal views and the Terminal Panel's; the keymap's `Terminal` entry.
- **File manifest.** Marley only: `crates/marley_terminal/src/anchored.rs`,
  `crates/marley_workbench/src/{marley_workbench.rs,blocks.rs}`, `keymap.json`, tests.

## Phase 2 — Code (2026-09-23)
- **Built.**
  - `marley_terminal::block_scroll`, re-exported, with two unit tests.
  - `marley_workbench`: the actions `PreviousBlock` and `NextBlock`; `blocks.rs`, whose `init`
    catches them at the workspace's root and scrolls the focused terminal view (a center pane's
    or a Terminal Panel pane's active item); `marley_terminal` as a dependency; the keymap's
    `Terminal` entry, `secondary-up` and `secondary-down`.
- **Deviation: the handler syncs before it reads.** `last_content` is the last frame's, so a
  second key before the next frame would have read the offset the first had not yet applied.
  Scrolling to the bottom and then up by the offset avoided an overshoot but lost the second
  key; `Terminal::sync`, public, applies the queued scroll first, so each key moves one block.
- **Review.** The actions reach the handler only from inside the workspace's element tree, as
  `routing`'s do; with no terminal focused it returns. The scroll stays within the history
  (`block_scroll` clamps). No Zed path changed.

## Phase 3 — Test (2026-09-23)
- **Tests.**
  - `marley_terminal`: `the_block_keys_scroll_to_each_blocks_start_from_the_viewports_top` and
    `with_no_block_that_way_the_block_keys_do_nothing` (REQ-001, REQ-002, REQ-003).
  - `marley_workbench::blocks`: `the_block_keys_walk_the_focused_terminals_blocks`, a real PTY's
    three finished blocks, 200 lines and a running block: `secondary-up` pressed twice before a
    frame lands on the second-to-last block, then the first, then stays; `NextBlock` walks
    forward, and `secondary-down` from the last finished block reaches the live screen
    (REQ-001 to REQ-004); `the_block_keys_leave_a_terminal_without_focus_alone`, with the focus
    in an empty pane split beside it (REQ-003). The crate's suite: 144 passed.
- **Negative checks,** each restored by checksum: without the sync the walk test fails (the
  double press stops a block short); without the keymap's keys it fails; with the focus check
  gone the other test fails.
- **Live drive:** not run. The keys need input, and Chad is at the desk (no input into his
  session); the driven test presses the same keys through the Marley keymap in a real
  window's dispatch.
- **Gate.** Two reds before the green:
  - gate:2: `clippy::double_ended_iterator_last` on `block_scroll` (`.rev().find(..)` now),
    and in the new files a long module doc and two `Default::default()` calls
    (`collections::HashMap` as a dev-dependency for the test's). Clippy was not run on
    `marley_terminal` before the gate, against L-claude-465-zeds-clippy-bans-std-process-in-tests-too-001's
    last line.
  - gate:4: `focused_terminal`'s Terminal Panel branch never ran. The spec promised the keys in
    the Terminal Panel, and no test put a terminal there;
    `the_block_keys_work_in_the_terminal_panel` does, and fails with the panel's panes left out
    of the search.
  - Then `GATE GREEN [diff]`, 20 passed, the receipt matching the tree.

## Phase 4 — Complete (2026-09-23)
- **Docs (§21).** `CHANGELOG.md` (Added); `docs/marley_architecture/marley_workbench.md` (the
  block keys) and `terminal_blocks.md` (`block_scroll`); `docs/marley/three-prong-plan.md`
  (T1c shipped). No Zed path changed.
- **Ledger (§19).** `AD-claude-473-the-block-keys-scroll-and-select-nothing-001`.
- **Brain.** Consultation `adaff31de9ed401b9ac309fab7eddd09` closed by
  `decisions/the-block-keys-scroll-the-focused-terminal-and-select-nothing`, follow-up 2026-10-07.
- **Ticket** closed; the pipeline archived; one commit.
