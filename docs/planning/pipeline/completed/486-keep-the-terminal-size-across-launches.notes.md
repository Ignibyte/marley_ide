# The first terminals of a launch open at the last session's size — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-486-keep-the-terminal-size-across-launches.md
- **Pipeline spec:** 486-keep-the-terminal-size-across-launches.spec.md

## Phase 1 — Plan
- **Request:** TICKET-486, #485's known limit.
- **Classification:** bug, prong 1 T0; a Marley module and two functions in Zed's `terminal` crate.
- **Recall:** #485's notes (the slot, the long-prompt scenario, readline's multi-line redraw).
  Brain: consultation 6719680f80254a0190b79eee8cedb58d, nothing on this seam.
- **Design.** `terminal_size::init` reads the kept `TerminalBounds` (JSON) from the key-value
  store and seeds the slot; an `on_app_quit` future writes `marley_last_bounds()` back.
- **Manifest.** `crates/terminal/src/terminal.rs` (its row grown first),
  `crates/marley_workbench/src/terminal_size.rs`, `marley_workbench.rs`; the scenario.

## Phase 2 — Code
- **Built.** `terminal::marley_last_bounds` and `terminal::marley_seed_last_bounds` (its row grown
  first); `crates/marley_workbench/src/terminal_size.rs` (`init`: the synchronous read and the seed;
  the quit's write), declared and called in `marley_workbench::init`. `TerminalBounds` already
  derives serde's traits, so the store holds its JSON.
- **Gate.** GATE GREEN [diff].

## Phase 3 — Test
- **Scenario.** `script/e2e/486-keep-the-terminal-size-across-launches.sh`, #485's prompt; a quit
  through the palette, then a launch on the same data directory.
- **Found on the way.** The first run, under sway, drew both launches right: in the headless sway
  the first view lays out before bash starts readline, so the bug does not show there. The
  scenario runs on Hyprland's hidden workspace, as #485's does (focus report: the user's window and
  workspace as they were).
- **Shots (Hyprland, 3416 × 1390, read with crops of the prompt line).**
  - `486-01-first-launch`: `probe ❯ e`, the cursor among the dashes: no size kept yet, #485's limit
    as before. REQ-002.
  - `486-02-relaunched`: `probe ❯ echo again`, the cursor after it: the first terminal of the
    second launch opened at the size the first launch quit with. REQ-001.

## Phase 4 — Complete
- Ledger: L-claude-486-the-first-terminal-bug-shows-on-hyprland-not-sway-001.
- Brain: decision recorded on consultation 6719680f80254a0190b79eee8cedb58d.
