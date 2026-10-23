# A block map by state, build progress, and the exit-127 hint — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-918-block-map-and-build-progress.md
- **Pipeline spec:** 918-block-map-and-build-progress.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order. This ticket takes three TerminalSession gaps from A-core.md.
- **Classification / tier:** feature, M. Mostly Marley crates; two small Zed hunks
  (`terminal_element.rs`, `terminal.rs`).
- **Recall (§18.3):**
  - AD-claude-559: block marks are data in `MarleyBlockMarks`; the element paints them at its
    right edge, not in the shared scrollbar. The map stays in the element.
  - L-claude-559: `MarleyBlockChip` has one slot; the count is one more part of
    `bookmarks::chip`, never a second `set_global`.
  - L-claude-572: a watcher written at every wakeup keeps its state in a global nothing
    observes. The progress global is read at paint, not observed.
  - PR-claude-474: an action that turns a block's command into input checks
    `command_verified` first. The hint reads the command and types a new one, so it checks.
  - F-claude-865: a hook a Zed view calls during its render never reads that view. The header
    and chip hooks read the progress global and the terminal, never the `TerminalView` being
    drawn.
  - F-claude-903: the stand-in `cargo` goes first on the terminals' PATH through the scenario's
    `.bashrc`, or the box's real cargo runs.
  - #557 (AD-claude-557): a 127 that reads as English asks the agent with its words. The package
    hint is checked first; a package-manager command does not read as English.
- **Discovery:**
  - `crates/terminal_view/src/terminal_element.rs`: the tick loop at `:2204`,
    `marley_bookmark_ticks` at `:2644`, `marley_bar_color` at `:2796`.
  - `crates/terminal/src/terminal.rs:2197` `marley_lines_since`, beside which
    `marley_cursor_line` goes; `crates/terminal/src/alacritty.rs:1004` `absolute_lines_text`.
  - `crates/marley_terminal/src/anchored.rs:1012` `scrollback_fraction`.
  - `crates/marley_workbench/src/block_headers.rs:205` `progress_line`, `:187` its caller.
  - `crates/marley_workbench/src/bookmarks.rs:181` `chip`; `send_block.rs:189` `chip`;
    `english.rs:303` `asks_on_127`; `running_errors.rs` (the watcher pattern);
    `remote.rs:122` `is_remote`; `links.rs:277` `ssh_in`; `agent_bar::agent_in`.
- **Decisions:** D1–D8 in the spec.

### Design
- `marley_terminal::anchored::block_map(blocks, screen_top, total_lines, screen_lines) ->
  Vec<MapMark>`: for each block, the share of its first line (`prompt_line` or `output_start`)
  and of its end (`output_end`, or the cursor's line while it runs), as `top` and `height`, with
  a 2 px floor applied by the element. `MarkState` from the block's state and exit code. Pure,
  so the element keeps no logic.
- The element paints each mark 8 px wide at `bounds.right() - 8`, color by `MarkState` (D2),
  then the bookmarked and selected marks in the accent over them.
- `marley_terminal::progress::parse_progress(line) -> Option<Progress>`: a `regex` for cargo's
  `Building \[[=> ]*\] (\d+)/(\d+)` and one for a leading `^\[(\d+)/(\d+)\]`; `done <= total`
  and `total > 0`, or none.
- `build_progress.rs`: `init` subscribes each new `TerminalView` (as `running_errors` does) and
  leaves out agent CLIs' and SSH clients' terminals. On `Wakeup` it schedules one read per 500 ms:
  the newest live block, if running, and `marley_cursor_line()` through `parse_progress`. A
  `Progresses` global (`HashMap<(EntityId, usize), Progress>`) keeps the last value; a finished
  block's entry goes. When the value changes, the view is notified. `progress(terminal, index,
  cx)` and `chip(..)` read it.
- `block_headers.rs`: `progress_line(index, progress, cx)`: with a `Progress`, a full-width track
  in the accent at low opacity and a fill of `relative(done / total)`; without, as now.
- `package_hint.rs`: `Family::from_os_release` (`ID`, then each word of `ID_LIKE`: arch,
  debian/ubuntu, fedora/rhel, suse/opensuse, alpine). `translate` splits the command on `|` and
  whitespace, drops `sudo` and `-y`, and maps verbs:

  | Verb | apt / apt-get / dpkg | dnf / yum / rpm | pacman | zypper | apk | brew |
  |---|---|---|---|---|---|---|
  | install X | `install X` | `install X` | `-S X` | `install X` | `add X` | `install X` |
  | remove X | `remove X`, `purge X` | `remove X` | `-R X` | `remove X` | `del X` | `uninstall X` |
  | search X | `search X` | `search X` | `-Ss X` | `search X` | `search X` | `search X` |
  | list installed [X] | `dpkg -l [X]`, `apt list --installed`, `… \| grep X` | `rpm -qa`, `dnf list installed` | `-Q`, `-Qs X` | `search -i` | `info` | `list` |

  It writes install and remove with `sudo` on Linux families, never on `MacOs`. A command whose
  program belongs to this box's own family gives none.
- `send_block.rs`: `package_hint(terminal, index, cx) -> Option<String>` with the conditions in
  the spec; `chip` returns the hint's row first when it answers. The family is read once into a
  `LazyLock<Option<Family>>`. The button writes the command's bytes with
  `Terminal::input`, no `\r`, and focuses the terminal.
- File manifest:
  - Zed crate: `crates/terminal_view/src/terminal_element.rs` (the ticks become the map),
    `crates/terminal/src/terminal.rs` (`marley_cursor_line`).
  - Marley crate: `crates/marley_terminal/src/anchored.rs`, `progress.rs` (new),
    `package_hint.rs` (new), `marley_terminal.rs` (the two modules);
    `crates/marley_workbench/src/build_progress.rs` (new), `block_headers.rs`, `bookmarks.rs`,
    `send_block.rs`, `marley_workbench.rs` (`build_progress::init`).
  - Scenario: `script/e2e/918-block-map-and-build-progress.sh`.
- Ledger rows in `docs/marley/zed-touchpoints.md`:
  - the `crates/terminal_view/src/terminal_element.rs` row: "a tick at the right edge for each
    bookmarked block" becomes "a mark at the right edge for each block, from
    `marley_terminal::block_map`, in its state's color and the accent while bookmarked or
    selected (#918)";
  - the `crates/terminal/src/terminal.rs` row: "`marley_cursor_line()`, beside
    `marley_lines_since`, the cursor's line on the main screen, where build tools redraw their
    progress (#918)".

### Visual check plan
- REQ-001: four blocks (done, failed, bookmarked, 127) and the running build; `918-01-map` shows
  one mark per block down the right edge, spaced by place and sized by length.
- REQ-002: `918-01-map` shows the colors; `918-02-bookmarked` shows the bookmarked mark in the
  accent after Ctrl Up selects `git status` and Ctrl Shift B marks it; `918-06-day` repeats in
  Marley Day.
- REQ-003: the stand-in cargo's bar at about 30 of 40; `918-03-progress` shows "30 of 40" before
  the "running" pill and the header's line filled to about three quarters.
- REQ-004: the stand-in prints no bar for its first 3 s; the review of the diff confirms the
  slide is kept for a block with no `Progress` (a shot of a moving bar proves little).
- REQ-005: `dpkg -l | grep libfoo` at the prompt; `918-04-hint` shows the hint on its block.
- REQ-006: a click on `pacman -Qs libfoo`; `918-05-typed` shows it typed at the prompt and no new
  block.
- REQ-007: no scenario: a remote host needs #445's fake SSH host and this ticket shows nothing
  there; the review of the diff checks `is_remote` and the project check.
- Not reachable here: a Debian-family or Mac box (this box runs Arch). The table's other columns
  are covered by the review of the diff; the Mac case is noted for the Mac tickets.

### Risks
- A terminal with thousands of blocks: thousands of 2 px marks overlap. Marks that land on the
  same pixel row can be merged in `block_map`; check the paint cost with a long scrollback.
- Cargo prints its bar only when stderr is a terminal and the width allows; `CARGO_TERM_PROGRESS_WHEN`
  can turn it off. The stand-in prints cargo's exact shape so the scenario does not depend on it.
- The cursor's line belongs to the newest block only; a block's progress is read while it is the
  running one.
- `sudo apt …` on a box without apt exits 1, not 127; the hint keys on sudo's own message for that
  case. Other wrappers (`doas`) are not read.
- A translated package name can be wrong for this distro (D6): the hint only types.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
