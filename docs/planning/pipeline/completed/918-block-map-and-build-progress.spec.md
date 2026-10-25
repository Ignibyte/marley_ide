---
pipeline_id: 4f0a5415-1036-47cb-b91d-fb3958979ac2
ticket: docs/planning/tickets/open/TICKET-918-block-map-and-build-progress.md
status: Phase 4 — Complete PASS
title: A block map by state, build progress, and the exit-127 hint
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/A-core.md, docs/marley/design-review-2026-10-23/README.md]
---

## Title
Three parts of the TerminalSession board. A mark per block down the terminal's right edge, in
its state's color. A running build's own progress ("1781 of 1813") with a bar filled to it. A
failed command that called another distro's package manager gets the same command in this box's
package manager ("This box runs Arch. Try pacman -Qs libfoo").

## Scope
### In
- **The block map** (Zed crate hunk, `crates/terminal_view/src/terminal_element.rs`): the
  bookmark ticks (`marley_bookmark_ticks`, painted after the search-scope outline) become a mark
  for every block of the main screen. Each mark sits at its block's place in the scrollback and
  is as tall as the block's share of the lines, at least 2 px. Its color follows the block's
  state.
- **The marks' geometry** (Marley crate, `crates/marley_terminal/src/anchored.rs`, beside
  `scrollback_fraction`): `block_map`, a pure function from the blocks and the screen's numbers
  to a list of `MapMark { top, height, state }`, where `state` is an enum (`Running`, `Failed`,
  `Done`, `Unknown`). The element only paints.
- **The cursor's line** (Zed crate hunk, `crates/terminal/src/terminal.rs`):
  `Terminal::marley_cursor_line()`, the text of the cursor's line on the main screen, none on the
  alternate screen. Build tools redraw their bar on that line with a carriage return, and
  `marley_lines_since` stops before it.
- **Progress shapes** (Marley crate, new `crates/marley_terminal/src/progress.rs`): `Progress {
  done, total }` and `parse_progress(line)`. Two shapes: cargo's bar (`Building [==>  ]
  1781/1813: zed`, which `cargo build`, `check`, `test` and `doc` all print) and a leading
  `[N/M]` (ninja, meson).
- **The progress watcher** (Marley crate, new `crates/marley_workbench/src/build_progress.rs`):
  on a plain terminal's `Event::Wakeup`, at most every 500 ms, reads the running block's cursor
  line and keeps the last `Progress` per terminal and block in a global nothing observes
  (L-claude-572). It forgets a block when it finishes.
- **The header** (Marley crate, `crates/marley_workbench/src/block_headers.rs`): `progress_line`
  fills to `done / total` while the block has a `Progress`, and slides as #865 draws it while it
  has none.
- **The count** (Marley crate, `crates/marley_workbench/src/bookmarks.rs` `chip` and
  `build_progress::chip`): "1781 of 1813" in the running color, a part of the one composed
  `MarleyBlockChip` hook (L-claude-559), before the "running" pill.
- **The exit-127 hint** (Marley crate):
  - new `crates/marley_terminal/src/package_hint.rs`: `Family` (`Arch`, `Debian`, `Fedora`,
    `OpenSuse`, `Alpine`, `MacOs`), `Family::from_os_release(content)` from `ID` then
    `ID_LIKE`, and `translate(command, family)`, which reads a package-manager command of
    another family and writes this family's: install, remove, search, and list installed (with
    its `| grep X` filter). Anything else gives none.
  - `crates/marley_workbench/src/send_block.rs` `chip`: for the newest live block at the prompt
    with no agent in front, when it ended with 127 (or its command starts with `sudo` and its
    output says `sudo: <program>: command not found`), its command is verified (PR-claude-474)
    and `translate` gives a command: "This box runs <family>. Try", the command as a button in
    the buffer font, and Ask an agent. The box's family is read once from `/etc/os-release` (or
    `MacOs` on a Mac with `brew` on the PATH).
  - The command button types the command at the prompt with no Return.

### Out (explicitly deferred)
- The rail row's "cargo build · 1781 of 1813" and the bottom row's running chip with its count.
  #910 (rail rows) and #908 (bottom row for a terminal) leave them out because nothing parsed
  progress. A follow-up ticket after #918 shows them from `build_progress`'s global.
- The ticking time beside the count ("· 2 m 04 s"): A-core's Terminal board, "Running block
  time", in the review's build order item 6 (terminal blocks).
- The bar along the block's bottom edge in a rounded frame: it waits for item 6's block frames.
  This slice fills the line #865 put at the header's top.
- The hint as a row under the block's output: item 6 moves a failed block's actions under its
  output, and the hint moves with them.
- Percent-only progress (cmake's `[ 45%]`, pip, npm): a follow-up ticket if wanted.
- A suggestion for a missing program that is not a package manager (which package has it): a
  follow-up ticket; the tools for it (`pacman -F`, `apt-file`) are often not set up.
- A click on a map mark to scroll to its block: not on the board.

## Reference (§20)
Warp for the terminal and its blocks: `docs/warp_architecture/subsystems/03-terminal-session-core.md`
§6, blocks with a state and an exit code set by the shell's hooks, which the map and the hint
read. Warp's map has no spec there, and none is read: the map, the progress shapes and the hint
come from the TerminalSession board and from what cargo and the package managers print.

### Prior art
- `terminal_element.rs:2181-2213`: the search-scope outline and the bookmark ticks;
  `marley_bookmark_ticks` (`:2644`) places each tick with
  `marley_terminal::scrollback_fraction` (`anchored.rs:1012`), none on the alternate screen.
  AD-claude-559: marks are painted by the element at its right edge, not in `ui`'s shared
  scrollbar. The map keeps that.
- `marley_bar_color` (`terminal_element.rs:2796`): the state colors a block's gutter bar uses
  (`info` running, `success` exit 0, `error` another exit, `border` none);
  `MarleyBlockSelection::selected` (`:2158`) and `MarleyBlockMarks::bookmarked`
  (`terminal_view.rs:302`) for the accent marks.
- `block_headers.rs:205` `progress_line`: #865's 30 % bar sliding along a running block's
  header, still and full width under reduced motion.
- `running_errors.rs` (workbench) and `marley_terminal::running_errors`: #572's watcher reads a
  running block's new lines on `Event::Wakeup`, at most every 500 ms, through
  `Terminal::marley_lines_since` (`terminal.rs:2197`), which reads up to the cursor's line and
  not that line itself (`alacritty.rs:1004` `absolute_lines_text`). Its `Watch` and `ErrorMarks`
  split is L-claude-572's.
- `send_block.rs:189` `chip`: the newest failed block's Ask an agent, Explain and Ask Marley;
  for a 127 that reads as English, `english::asks_on_127` (`english.rs:303`). `bookmarks.rs:181`
  composes every chip part into the one hook (L-claude-559).
- `beside.rs:90` `apply_chip`: #865 types a proposed command into the terminal (with a Return;
  this hint types it without one).
- `util::parse_os_release` (`crates/util/src/util.rs:69`) reads `ID` and `VERSION_ID` only; the
  family needs `ID_LIKE` too (this box: `ID=omarchy`, `ID_LIKE=arch`), so `package_hint` reads
  both. `remote::is_remote` (`remote.rs:122`) and the project's `is_remote` tell a terminal whose
  commands run elsewhere.
- grep for "Building \[", "progress" and "os-release" in `crates/marley_*`: nothing parses a
  command's progress or reads the distro.

## UI proof
Scenario `script/e2e/918-block-map-and-build-progress.sh` (sway). A scratch repository and a
HOME whose `.bashrc` puts the stand-ins first on the terminals' PATH (F-claude-903): a stand-in
`cargo` that prints a few `Compiling` lines, then redraws cargo's `Building [..] N/40` bar with a
carriage return, slowly, and waits. In one terminal: `git status` (done), `false` (failed), the
selected block bookmarked with Ctrl Shift B, then `dpkg -l | grep libfoo` (127 on this box,
which runs Arch), then the stand-in `cargo build`.
- `918-01-map`: the map with a mark per block, each in its state's color (REQ-001, REQ-002).
- `918-02-bookmarked`: the bookmarked block's mark in the accent (REQ-002).
- `918-03-progress`: the running stand-in build: "N of 40" before the pill and the header's
  line filled to the share (REQ-003).
- `918-04-hint`: the 127 block: "This box runs Arch. Try", `pacman -Qs libfoo`, Ask an agent
  (REQ-005).
- `918-05-typed`: the command button clicked: `pacman -Qs libfoo` at the prompt, not run
  (REQ-006).
- `918-06-day`: the map, the progress and the hint in Marley Day (REQ-002, REQ-003, REQ-005).

## Locked-In Decisions
- D1 — The map is drawn over the grid's right edge, 8 px wide, as the ticks are. No column is
  reserved, so the terminal's column count does not change.
- D2 — Mark colors: running in `info` (honey in Marley Night), failed in `error`, exit 0 in a
  muted color (the board draws done blocks dim, not green), no exit in `border`; a bookmarked or
  selected block in the accent, over its state.
- D3 — Progress is read from the cursor's line only, which is where cargo and ninja redraw their
  bar. A build's whole output is never read again.
- D4 — The bar fills in the place #865's line has, the header's top. Moving it waits for the
  block frames (Out).
- D5 — The hint names the family that sets the package manager: a box with `ID_LIKE=arch` says
  "This box runs Arch".
- D6 — The hint only types its command: a translated install can be wrong (package names differ
  between distros), so the user reads it and presses Enter.
- D7 — A block with a hint shows the hint, its command and Ask an agent, as the board does.
  Explain and Ask Marley stay on every other failed block.
- D8 — No hint for a terminal whose commands run elsewhere: a remote terminal or a terminal of an
  SSH project. The distro read is this machine's.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a terminal shows its main screen, the terminal shall draw a mark at its right edge for each block, at the block's place in the scrollback and as tall as its share of the lines, at least 2 px, and none on the alternate screen. | `918-01-map`; the review of the diff for the alternate screen |
| REQ-002 | Each map mark shall take its block's state color (running, failed, exit 0 muted), and the accent while its block is bookmarked or selected, in Marley Night and Marley Day. | `918-01-map`, `918-02-bookmarked`, `918-06-day` |
| REQ-003 | WHILE a running block's cursor line shows cargo's progress bar or a leading `[N/M]`, the block shall show "N of M" before its pill and its header's line shall fill to N/M. | `918-03-progress`, `918-06-day` |
| REQ-004 | WHILE a running block has shown no progress, its header's line shall slide as #865 draws it. | the review of the diff |
| REQ-005 | WHEN the newest block of a local terminal ends with 127, or with sudo's "command not found", and its command is another family's package-manager command, the block shall show "This box runs <family>. Try", this box's command and Ask an agent. | `918-04-hint`, `918-06-day` |
| REQ-006 | WHEN the hint's command is clicked, the system shall type it at the prompt without running it. | `918-05-typed` |
| REQ-007 | WHILE a terminal is remote or belongs to an SSH project, the system shall show no package hint. | the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the two Zed hunks with their ledger rows, `progress.rs`, `package_hint.rs`,
  `block_map`, `build_progress.rs`, the header and chip parts; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
