# The bottom row for a terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-908-bottom-row-for-a-terminal.md
- **Pipeline spec:** 908-bottom-row-for-a-terminal.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this". This
  is the design review's build order, item 2 (the bottom row, for every screen). Build order of
  the batch: 906, 905, 907, 908, 909, 910, 911.
- **Classification / tier:** feature, S–M. Marley crate only (`marley_workbench`); no Zed hunk.
- **Recall (§18.3):**
  - F-claude-833 and PR-claude-833: the focus item shrank below its content and Zed's items were
    drawn over it. The root stays `flex_none`; each new long part (the folder, the failed command)
    gets a `max_w` and truncates.
  - F-claude-893: a status or toolbar item never reads its workspace in `set_active_pane_item`. The
    row already reads the workspace's project only in `render`. The branch, count and rail lookups
    go there too.
  - PR-claude-831: `FocusRow::new` reads nothing of the workspace. That is unchanged.
  - F-claude-630: a long folder pushed the branch off the block header. That led to
    `folder_label`'s short form, which the row reuses. The branch label truncates before the
    failed chip shrinks.
  - F-claude-903 and PR-claude-687: the stand-in `claude` goes first on the terminals' PATH
    through the scenario's `.bashrc`, or Ask an agent with no agent running starts the box's real
    Claude Code.
  - F-claude-880: the harness isolates tmux (`TMUX_TMPDIR`). The scenario starts its sessions
    on that server only, never the box's.
  - F-claude-865: does not apply. The row is not called from a Zed view's render.
  - AD-claude-tmux-grade-detached-sessions-001: tmux sessions are a pillar; the detached count
    is the insight it asks for.
- **Discovery:**
  - `crates/marley_workbench/src/focus_row.rs`: `render_terminal` (`:105-196`), `chip`
    (`:358`), `Render::render` (the rail is already found and observed through `rail()`).
  - `crates/marley_workbench/src/block_headers.rs:234` `folder_label` (private today).
  - `crates/marley_workbench/src/agent_bar.rs:112` `branch_of`, `:150` `branch_for`.
  - `crates/marley_workbench/src/send_block.rs:103` `send`, `Request::Fix`.
  - `crates/marley_workbench/src/rail.rs:5429` `show_sessions` (private; `rail_strip.rs` reaches
    it as a child module, `focus_row.rs` does not); `rail.rs:9655` `command_snapshot`.
  - `crates/marley_workbench/src/marley_workbench.rs:1429` `toggle_rail_strip`;
    `workspace::MultiWorkspace::open_sidebar` (`multi_workspace.rs:511`), `sidebar_open`.
  - `crates/marley_workbench/src/tmux_sessions.rs:48` `TmuxSession::attached`.
  - `crates/terminal/src/terminal.rs:568` `Content::terminal_bounds`, `:898` `num_lines`, `:906`
    `num_columns` (read only).
  - `crates/project/src/git_store.rs:6378` `status_summary` (read only).
- **Decisions:** D1–D7 in the spec. The coordinator asked for #899's `ask_to_fix`. The sweep found
  it works only from an editor's caret line. `send_block::send` with `Request::Fix` is the
  terminal block's own Ask an agent (#826), so the row reuses that (D1). #910's failed shell row
  should do the same.

- **Promoted 2026-10-25:** seams re-read after #905 and #907: `render_terminal` unchanged;
  `block_headers::folder_label` :234 takes a `&str`; `agent_bar::branch_of` :112 and `branch_for`;
  `send_block::send` :103 takes the anchored block's `usize` index; `Rail::show_sessions` :5431;
  `crate::toggle_rail_strip` :1431 is reachable as `crate::`; `Repository::status_summary` gives a
  `GitSummary` with `count`. #905's `item_kind` is in, so the chips take `ItemKind::Shell` and
  `ItemKind::Tmux` tints directly.

### Design
- Approach:
  - `render_terminal` builds two groups. The left group holds the `>_` chip with the title, the
    folder, the branch with its count, and the failed chip. The right group holds "running: …" or
    "no command running", the blocks and bookmarks, the size, and the tmux chip.
  - The folder: `folder_label(&terminal.working_directory())`, capped with `max_w` and truncated.
  - Branch and count: `agent_bar::branch_and_changes(project, folder, cx) -> Option<BranchChanges>`,
    a struct with named fields (`branch: SharedString`, `changed: usize`) beside `branch_of`, which
    picks the same innermost repository and reads its `status_summary().count`. `branch_of` stays
    for its callers.
  - The failed chip: from `marley_anchored().live().last()`. When its state is `Finished` and its
    exit code is non-zero, a `ButtonLike` (id `marley-focus-ask-agent`, selector
    `marley-focus-failed`) holds a dot in `status().error`, on `status().error_background`, with the
    words in the error colour. Its click calls `send_block::send(&view, block.index, Request::Fix,
    window, cx)` with the view kept in `Focus::Terminal`. `command_snapshot` still gives the text,
    so an agent's block and a block from before a restart stay out, as they do today.
  - The size: `format!("{}×{}", bounds.num_columns(), bounds.num_lines())` in the buffer font,
    muted.
  - tmux: count every session and the ones with `!attached`. The chip
    (`marley-focus-tmux`) is on the tmux tint from #905's helper. Its click goes through a weak
    rail handle kept from `rail()`: when the rail is shrunk, `toggle_rail_strip`; when the sidebar
    is closed, `MultiWorkspace::open_sidebar` through `window.root`; then
    `rail.update(|rail, cx| rail.show_sessions(window, cx))`. The click is deferred with
    `window.defer`, so it never updates the multi-workspace from inside the status bar's update.
  - `Focus` stays as it is. The terminal variant already carries the view the click needs.
- File manifest (all Marley crate, `crates/marley_workbench/src/`): `focus_row.rs`,
  `block_headers.rs` (`folder_label` to `pub(crate)`), `agent_bar.rs`
  (`branch_and_changes`, `BranchChanges`), `rail.rs` (`show_sessions` to `pub(crate)`),
  `marley_workbench.rs` (`toggle_rail_strip` to `pub(crate)` if it is not reachable already).
  Files of #905's helper are read, not changed.
- Ledger rows: none. No path outside `crates/marley_*` changes, so `docs/marley/zed-touchpoints.md`
  needs no row.
- Depends on #905 for the per-kind tint helper. If #908 is built first, the chips use the rail's
  `RowKind` tints (`rail.rs:11046`, `Shell` and `Tmux`) through a `pub(crate)` accessor, and #905
  moves them later.

### Visual check plan
- REQ-001, REQ-002, REQ-003, REQ-005, REQ-006, REQ-007: setup builds a git repository on `main`
  with one committed file, then changes it. The project's terminal runs `false`. Shot
  `908-01-root` shows the full row at the root.
- REQ-004: click the red chip. #899's stand-in idles in a second terminal through the plugin hook,
  so it is the one target. Switch to it. Shot `908-02-asked` shows "This command failed. Fix it:"
  in its prompt.
- REQ-005: right-click a block and choose Bookmark (or press its key). Shot `908-03-bookmarked`
  shows "· 1 bookmarked".
- REQ-006: split the pane right, so the terminal narrows. Shot `908-04-resized` shows fewer columns
  than `908-01-root`.
- REQ-007: setup starts two detached sessions with `tmux new-session -d` on the run's server. A
  third terminal tab runs `tmux new-session -s here`, then the project's terminal comes back to the
  front. Shot `908-01-root` shows "tmux: 3 · 2 detached".
- REQ-008: shrink the rail (`marley: toggle rail strip`), then click the tmux chip. Shot
  `908-05-sessions` shows the open rail on its Sessions view.
- Marley Day: switch the theme to Marley Day. Shot `908-06-day` shows the row of `908-01-root`.
- REQ-009: the review of the diff.
- No scenario can reach a remote tmux host's count. It is counted the same way as a local one,
  and #445's fake SSH host is not needed for this row.

### Risks
- The row grows long. In a narrow window, Zed's items after it (the language server message) give
  way first (F-claude-833). The tmux chip and the size must stay visible, so the folder and the
  failed command cap first.
- `status_summary()` is the snapshot's cached status. It can lag a file write by a scan. The shot
  waits for the count before it is taken.
- The stand-in's hook must report the agent as idle for `targets()` to pick it (#899's setup). With
  no target, `send` starts Claude Code, and the `.bashrc` PATH keeps that the stand-in.
- An attached session made from Marley's own terminal tab changes the tab in front. The scenario
  brings the project's terminal back before the shot.

## Phase 2 — Code
- **Built:** `focus_row.rs` (`render_terminal` in two groups, `FailedBlock`, `failed_chip`,
  `tmux_chip`, `show_tmux_sessions`, `blocks_words`, `chip` on a kind's tint);
  `agent_bar.rs` (`BranchChanges`, `branch_and_changes`); `block_headers::folder_label` and
  `Rail::show_sessions` made `pub(crate)`. No Zed path.
- **Deviations:** "no command running" is left out while the failed chip shows, and the caps are
  tighter than planned, so the tmux chip fits a 1600-pixel window with the rail open
  (F-claude-908-the-terminal-row-pushed-its-tmux-chip-under-zeds-items-001). "1 blocks" with a
  bookmark (#833's wording) fixed to "1 block".
- **Review:** the tmux click is deferred, so it updates the multi-workspace outside the status
  bar's update; the failed chip's send defers as the header's does. Google style:
  `BranchChanges` and `FailedBlock` have named fields, no tuple.
- **Gate:** `just gate-diff` 17/17 green after `too_many_lines` (the blocks words moved out),
  `similar_names` and `redundant_closure_for_method_calls`.

## Phase 3 — Test
- **Scenario:** `script/e2e/908-bottom-row-for-a-terminal.sh` (sway): a repository on `main` with
  README.md changed; tmux `one`, `two` detached and `here` attached from a second terminal on a
  private socket; a stand-in `claude` on the terminals' PATH.
- `908-01-root`: `>_ repo — bash`, `…/work.…/repo`, "main · 1 changed" in violet, the red
  "false · exit 1 · Ask an agent", "1 block", "146×42", "tmux: 3 · 2 detached" (REQ-001, REQ-002,
  REQ-003, REQ-005, REQ-006, REQ-007).
- `908-02-bookmarked`: "1 block · 1 bookmarked" (REQ-005).
- `908-06-day`: Marley Day, the same row (REQ-001, REQ-002, REQ-003, REQ-007).
- `908-03-asked`: the chip clicked: with no agent running, the stand-in Claude Code started with
  "This command failed. Fix it:", the block, "exit 1" (REQ-004).
- `908-04-sessions`: the rail shrunk, the tmux chip clicked: the rail open on its Sessions view,
  `one` and `two` detached, `here` attached elsewhere (REQ-008).
- `908-05-resized`: after a split, the narrowed terminal: "71×44" (REQ-006).
- Earlier runs: the tmux chip cut off (fixed, above); a split from the rail's focus did nothing
  and the new pane took the focus (L-claude-908-a-pane-action-from-the-palette-needs-a-pane-focused-001).
- Not reached: #899's idle stand-in as the window's one agent; the send's target choice is
  `send_block`'s, unchanged, and the no-agent path shows the request reaching Claude Code.

## Phase 4 — Complete
- CHANGELOG, `docs/marley_architecture/marley_workbench.md`. No Zed path, so no ledger row.
- Knowledge: F-claude-908-the-terminal-row-pushed-its-tmux-chip-under-zeds-items-001,
  L-claude-908-a-pane-action-from-the-palette-needs-a-pane-focused-001.
