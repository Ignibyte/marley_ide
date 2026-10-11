# Restored scrollback — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-751-restored-scrollback.md
- **Pipeline spec:** 751-restored-scrollback.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11, on the survey list (`docs/planning/design-notes/finish-line-2026-10-11.md`),
  item 1, "Restarts that lose nothing": "1.) this is nice". The survey's text: "then saved
  scrollback replayed dimmed above the new prompt, with blocks redrawn from saved records rather
  than from replayed bytes. … M to L for scrollback" (`docs/t3code_architecture/README.md`, item
  1; report 05, item 1). T3's open question 7 default: "on, mode 0600, with a setting to turn it
  off". The parent asked for an honest size and welcomed a split.
- **Classification:** feature. The whole item is L; this spec is the first slice (M), the output;
  the blocks half is a second slice (M), sketched below, waiting for a number.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **Recall (§18.3):**
  - AD-claude-575 and PR-claude-a-row-a-restore-reads-is-read-before-any-cleanup-001: a restored
    terminal keeps its `MARLEY_TERMINAL_ID`; Zed runs more than one cleanup, so what a restore needs
    is decided before any cleanup.
  - AD-claude-474 and PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001: a
    frame is output; a restored frame would carry an old nonce, so restored bytes must not make
    blocks.
  - PR-claude-639-a-change-to-the-grid-on-the-main-thread-carries-the-hooks-in-flight-001: anything
    anchored to grid lines must follow clears and rewraps; the dim rides the cells instead (D4).
  - PR-claude-restore-is-a-second-constructor-001: the recorder is made in the builder, the one path
    both a new and a restored terminal take.
  - PR-claude-blocking-io-off-the-gpui-main-thread-selftest-proves-it-001: the read happens in the
    builder's background future, the write in a background task.
  - F-claude-587 and PR-claude-what-a-terminal-shows-now-is-read-from-its-visible-rows-001: readers
    that skip blank rows reach into the scrollback. Restored rows will be in the scrollback, so the
    trust watch, #508's question reads and #547's checks must keep reading the visible rows; a
    restored Claude Code question must not read as asked again.
  - F-claude-588: a scenario names what it opens; the runner's `--user-data-dir` keeps the logs in
    the run's profile.
  - #540's spec Out: "The scrollback: Zed restores an empty terminal, and the resumed Claude Code
    draws its own conversation." #559: bookmarks have no scrollback to sit on after a restore.
  - Brain: no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **Discovery (2026-10-11; re-verify at promotion):**
  - `crates/terminal/src/terminal.rs`: the builder's future (1282, run by `cx.background_spawn` at
    1687); the restored id (1335–1343); `open_pty` (1522); `spawn_event_loop` (1539); the
    `blocks` field (1839, built 1632–1645); `process_event` answers: `ClipboardLoad` 1957, `PtyWrite`
    1966, `TextAreaSizeRequest` 1967, `ColorRequest` ~2000; `write_raw_output` 2480 (a plain
    `Processor<StdSyncHandler>`).
  - `crates/terminal/src/alacritty.rs`: `spawn_event_loop` 214–235 (`EventLoop::new(term, ZedListener,
    pty, drain_on_exit, false)`).
  - `vendor/alacritty_terminal/src/marley_hooks.rs`: `ShellHook` 24, `HookPosition` 35,
    `advance_with_hooks` 74; `event_loop.rs:155` in `pty_read` (104); the `State` holds the
    notification scanner (#478). `vendor/README.md` lists each hunk.
  - `crates/marley_dcs/src/marley_dcs.rs`: `DcsScanner`, no dependencies; `notification.rs` (OSC 9,
    OSC 777).
  - `crates/marley_terminal/src/anchored.rs`: `AnchoredBlock` 44–65, `BlockTimes` 141,
    `AnchoredBlocks` 188–214, `prompt_rows` 770 (verified only); no `Serialize` on any of them.
  - `crates/terminal_view/src/terminal_element.rs:929-932`: `cell.is_dim()` → `fg.a *= 0.7`.
  - `crates/terminal_view/src/persistence.rs`: `terminals(workspace_id, item_id,
    working_directory, working_directory_path, custom_title)` 410–456.
  - `crates/marley_workbench/src/terminal_ids.rs`: `known_ids` 53, `saved` 60, `cleanup` 99.
  - 0600 and 0700: `agent_activity.rs:271-283` (`append_in`), `marley_mcp/src/discovery.rs:39-50`,
    `agent_reports.rs:143-148`.
  - Settings: `settings_content/src/marley.rs` (`block_headers` 70 as a terminal bool's model),
    `default.json` 1702 on, `marley_page.rs` Terminal section 592.
  - Nothing in the tree records terminal output or persists blocks today.

### Design
- **`marley_dcs::history` (Marley crate, new module):** `Allowlist`, a scanner read by read with a
  carry (as `DcsScanner`): passes text, CR, LF, BS, HT, `CSI … m`, cursor motion (`A`–`H`, `f`,
  `d`, `e`, `a`, `` ` ``), erase, insert and delete (`J`, `K`, `X`, `P`, `@`, `L`, `M`), scroll
  (`S`, `T`) and the scroll region (`r`) without private markers or intermediates; drops every
  other CSI, every OSC, DCS, APC, PM, SOS, BEL and lone ESC sequences; tracks `?1049`, `?1047` and
  `?47` and drops what comes while the alternate screen is on. `dim(bytes)`: the replay form,
  starting `ESC[2m`, each SGR given a trailing `;2`, ending `ESC[0m` and CR LF. Pure, no
  dependencies, usable from the vendored crate.
- **`marley_terminal::history` (Marley crate, new module):** `History`, the ring with both caps
  (whole lines, never a split character); `append_in(dir, id, bytes)`, `compact_in(dir, id)`,
  `read_in(dir, id) -> Option<Vec<u8>>` (the tail within the caps), `remove_in(dir, id)`,
  `retain_in(dir, ids)`; the folder 0700, the file opened with mode 0600 at creation.
- **`vendor/alacritty_terminal` (Marley-owned for the ledger; `vendor/README.md` gains the
  hunks):** an optional history sink in the event loop's `State`, beside the notification scanner;
  `advance_with_hooks` feeds it the bytes the parser sees after Marley's frames are taken out; the
  sink is the `Allowlist` and a channel to the writer.
- **Zed's `terminal` crate (rows exist for both files, each widened):**
  - `terminal.rs`: a `MarleyTerminalHistory` hook global (function pointers, as
    `MarleyTerminalIdentity` is): `restored(id) -> Option<Vec<u8>>` and `recorder(id)`; in the
    builder, for a local interactive terminal with an id: before `spawn_event_loop`, the restored
    bytes through `history::dim` into `term`; the recorder handed to the loop; a
    `// Marley:` comment on each hunk.
  - `alacritty.rs`: `spawn_event_loop` takes the optional sink.
- **`marley_workbench` (Marley crate):** a `scrollback.rs` that sets the hook from `init` (after
  `terminal_ids::init`): the folder under `paths::data_dir()`, the setting, the writer task (append
  within a second of quiet, compaction past twice the caps, a flush awaited in `on_app_quit`), the
  deletions (`retain_in` the known ids at `init`; a terminal closed while Marley runs, through
  `terminal_ids`' cleanup or the view's release, never while quitting; every file when the setting
  turns off).
- **Settings (Zed paths, rows exist):** `crates/settings_content/src/marley.rs`
  (`restore_scrollback`), `assets/settings/default.json`, `crates/settings_ui/src/marley_page.rs`
  (Terminal section); `MarleySettings::restore_scrollback`.
- **File manifest:** `crates/marley_dcs/src/{marley_dcs.rs, history.rs}`,
  `crates/marley_terminal/src/{marley_terminal.rs, history.rs}`,
  `crates/marley_workbench/src/{marley_workbench.rs, scrollback.rs, terminal_ids.rs}` (Marley crates);
  `vendor/alacritty_terminal/src/{marley_hooks.rs, event_loop.rs}` and `vendor/README.md`;
  `crates/terminal/src/{terminal.rs, alacritty.rs}` (Zed crate); the three settings files;
  `docs/marley/guide.md`; `script/e2e/751-restored-scrollback.sh`.
- **The second slice, sketched for its ticket:** at each `precmd` and `preexec` Marley applies, the
  writer notes the log offset; a finished block's record (command, verified at the time, exit code,
  folder, host, start and end, the four offsets) is appended to `<id>.blocks.jsonl` beside the log,
  dropped with the log's head at compaction. At the restore, the replay is fed in pieces cut at those
  offsets, the cursor's absolute line read at each cut, and `AnchoredBlocks` gains `restore(record,
  prompt_line, output_start, output_end)`, a block marked restored: drawn with its pill and header,
  never offered Rerun, never read by an agent as current. Size M.

### Visual check plan
| REQ | The scenario does | The shot or check |
|---|---|---|
| — | Colours, `seq 1 20000`, `echo before`, `bin/ask` (queries, answers read away) | 751-01-before |
| 001, 002, 006 | `quit_marley`, `launch_marley`, settle | 751-02-restored: dim lines above a bright prompt, no pills on them |
| 003, 006 | Types `echo clean` | 751-03-clean: `clean` printed, a block for it, nothing else at the prompt |
| 004 | Scrolls the restored terminal to its top | 751-04-cap: the first number past 15000 |
| 005 | A second terminal ran `less README.md` at the quit | 751-05-fullscreen: the lines before `less`, no `less` screen |
| 007 | `stat` of the log and its folder in the run's data folder | `expect`: 600 and 700 |
| 008 | Closes the second terminal | `expect`: its log gone |
| 009 | Quit; `profile_setting marley.restore_scrollback false`; launch | 751-06-off: an empty terminal; `expect`: no logs |
| 010 | review | the builder's condition (an id, local, interactive) |
Not reachable by a scenario: a crash's loss window (at most a second of output); the scenario
quits cleanly.

### Risks
- **Agent TUIs in the log.** Claude Code draws inline with cursor motion; its redraws fill the caps
  fast, and replayed at another width they can garble. The result is dim and above the resumed
  agent, which redraws its own conversation (#540, #750).
- **Readers of the scrollback.** Restored rows hold old prompts and questions; any reader that walks
  into the scrollback (F-claude-587) could read them as current. Check the trust watch, the
  question reads and the stall checks against a restored terminal at promotion.
- **The IO thread's cost.** The allowlist runs on every read; it must stay a byte scanner with a
  carry, no allocation per byte.
- **A split or a new terminal** gets a new id and no history; only a restored id replays.
- **Secrets.** The log keeps what was printed (a token echoed once stays until it scrolls out of
  the caps). Mode 0600, the setting, and deletion on close are the guard; the guide says so.

### Promotion (2026-10-11)
- Moved from `queued/` to `active/`; the BACKLOG row removed; the ticket in progress.
- **Seams checked again** at `c02d9e6b44` (an Explore pass over all eleven). They hold:
  - **`terminal.rs`:** `TerminalBuilder::new` 1236, the future 1282 run at 1687; the restored id
    1335-1343 (`marley_restored_id`, the only sign of a restore); `new_term` 1459-1465; `open_pty`
    1522; `spawn_event_loop` 1538-1539. A local interactive terminal is `task.is_none() &&
    !is_remote_terminal` (1341), with `!no_pty` for shell integration (1375); `new_display_only`
    1117 never reaches `new`. The answers: `ClipboardLoad` 1956, `PtyWrite` 1966,
    `TextAreaSizeRequest` 1967, `ColorRequest` 1990, and `ClipboardStore` 1953, `Title` 1933,
    `Bell` 1975.
  - **The rest:** `alacritty.rs` `spawn_event_loop` 214-237; `marley_hooks.rs`
    `advance_with_hooks` 74-97; `event_loop.rs` 155-162 and `State` 408-417 (made in the reader
    thread at 215); `marley_dcs` with no dependencies, used by the vendored crate (its
    `Cargo.toml:30`); nothing records output; `terminal_element.rs:929-932`;
    `marley_workbench::init` calls `terminal_ids::init` at 900 and `resume::init` at 901.
- **What differs from the draft, and where the design takes it:**
  - **No function-pointer global in `terminal.rs`, and the builder's future has no `cx`**
    (`MarleyTerminalIdentity` is in `terminal_view.rs:340`, not `Send`). `MarleyTerminalHistory` is
    read in `TerminalBuilder::new` before the future, as `MarleyAgentHistory` is (1256), and its two
    closures are `Send + Sync`: `restored(id) -> Option<Vec<u8>>`, the replay bytes ready to feed
    (read, filtered again and dimmed, off the main thread in the future), and `recorder(id) ->
    Box<dyn FnMut(&[u8]) + Send>`, the sink the event loop feeds.
  - **The replay goes after `new_term` and before `open_pty`,** before the shell exists. What the
    term raises while it reads the replay waits in `events_tx` and is handled after `subscribe`,
    so it would answer the live shell: only the allowlist can stop that (D1, D3), and it must pass
    nothing that makes `Term` raise `PtyWrite`, `ColorRequest`, `TextAreaSizeRequest`,
    `ClipboardLoad`, `ClipboardStore`, `Title` or `Bell`.
  - **The recorder is a sink, not the allowlist in the vendored crate:** `EventLoop` takes an
    optional `Box<dyn FnMut(&[u8]) + Send>`, moved into the reader thread with `State`, and
    `advance_with_hooks` calls it with each passthrough piece. The allowlist runs inside the sink,
    which `marley_workbench` builds, so the vendored hunks stay generic and small.
  - **No hook tells a close from a quit:** Zed removes a closed terminal's rows at the next load's
    cleanup. A `Terminal` entity released while Marley is not quitting is a closed terminal, as
    `resume.rs` decides (`on_app_quit` sets `quitting`); its file goes then. Closing a project
    releases its terminals too, so they come back empty if it is reopened.
  - **The writer** is a background task in `marley_workbench::scrollback`: the sinks send
    `(id, bytes)` on an unbounded channel; it appends a terminal's pending bytes after a second of
    quiet, compacts a file past twice the caps, and flushes at `on_app_quit`. The setting is an
    `Arc<AtomicBool>` the sinks and `restored` read, so turning it off stops every recorder at once.
  - **Readers that reach restored rows** (`last_n_non_empty_lines` walks into history on a sparse
    screen): the links watch (`links.rs:326`, 200 lines) can offer an old URL whose port listens
    again, harmless; the stall watch (`stall.rs:381`) reads five lines of a stalled agent, which a
    resumed agent redraws; Zed's inline assistant reads restored rows as earlier output, which they
    are; the trust watch runs only in a worktree agent's fresh terminal, never a restored one.
    #508's inbox and #547's checks read seat state, not the grid.
  - **Settings model:** `sticky_command_header`, a plain bool (`marley.rs:64`,
    `marley_workbench.rs:399`, 671-673); the Terminal section's array grows from 6 to 7 items
    (`marley_page.rs:590`).
  - **The folder's 0700** follows `agent_reports::private_folder` (`DirBuilder` with `mode(0o700)`,
    then `set_permissions`); the file's 0600 follows `agent_activity::append_in`.
- **Brain:** consultation ffa0db78226843e8a85e670e5bc514dd (nothing on this seam); a duplicate ask
  closed with `no-decision`.

## Phase 2 — Code
- **Built:**
  - `marley_dcs::history` (new): `Allowlist` (text, CR, LF, BS, HT; SGR; cursor motion `A`-`H`,
    `f`, `d`, `e`, `a`, `` ` ``; `J`, `K`, `X`, `P`, `@`, `L`, `M`; `S`, `T`, `r`, `s`, `u`; `ESC 7`,
    `ESC 8`, `ESC D`, `ESC E`, `ESC M`; everything else dropped, the alternate screen's bytes too,
    `?47`, `?1047` and `?1049` tracked); `Allowlist::dimming`, SGR 2 after every SGR; `replay`;
    `SESSION_END`.
  - `marley_terminal::history` (new): `MAX_LINES`, `MAX_BYTES`, `file_in` (the id checked),
    `append_in` (folder 0700, file 0600), `read_in` (the tail within the caps, cut at a line's
    start, the file read from `MAX_BYTES` before its end), `compact_in` (through a `.partial` file
    and a rename), `remove_in`, `retain_in`.
  - `vendor/alacritty_terminal`: `EventLoop::set_history` and its field; `advance_with_hooks`
    feeds the sink each passthrough piece; `vendor/README.md` lists the hunks.
  - Zed's `terminal`: `MarleyTerminalHistory` (two `Send + Sync` closures), read in
    `TerminalBuilder::new` before the future; `marley_restored`; the replay into the term after
    `new_term` and before the PTY, for a local interactive terminal with an id and a PTY; the
    recorder handed to `spawn_event_loop`, which sets it on the event loop.
  - `marley_workbench::scrollback` (new): the hook, the writer task (`Message::{Output, Retain,
    Remove, Flush}`, `Pending`), the start's `Retain` of the known ids, the setting's observer,
    the quit's `Flush`, and the deletion when a `TerminalView` goes while its window is open.
  - Settings: `restore_scrollback` in `MarleySettingsContent`, `default.json` and the Marley page's
    Terminal section (seven items now); `MarleySettings::restore_scrollback: RestoreScrollback`.
  - Guides: `docs/marley/guide.md` (Earlier output after a restart) and the in-app guide (an
    article, its contents entry, the settings table's row).
- **Deviations from the plan:**
  - `RestoreScrollback` is an enum, as `ResumeAgents` is: a fourth bool in `MarleySettings` fails
    clippy's `struct_excessive_bools`. `from_settings` was at its 100-line limit, so
    `EnglishHint::from_content` replaced its eight-line `if` (same values).
  - `crates/marley_terminal/Cargo.toml` asks for `rand`'s `thread_rng` and `uuid`'s `v4`, which it
    uses: `just clippy marley_terminal` alone failed in untouched code (`rand::random`,
    `Uuid::new_v4`), built only where another crate turned the features on. Pre-existing, the
    class of F-claude-771.
  - `marley_dcs` became a dependency of `marley_workbench` (the recorder's allowlist).
  - The sink's trait object is `dyn FnMut(&[u8]) + Send + 'static` in `advance_with_hooks`: with
    the lifetime elided, the borrow of the event loop's field had to live for `'static`.
  - `ResumeAgents`' doc now names any agent's session (#750 left it saying Claude Code's).
- **Review of the diff:**
  - **Found and fixed: a restored file's sessions ran together.** The new session's output was
    appended right after the old session's last line, an unfinished prompt with its attributes,
    so the next replay would draw two prompts on one line. `restored` now sends `SESSION_END` to
    the writer before the recorder exists, so the file holds the same break the replay draws.
  - **Found and fixed: a window's close deleted every file.** On Linux the last window's close
    quits only after the window is gone, and its terminals are released first, while `quitting`
    is still false, so "released while not quitting" read as closed. gpui takes a closing window
    out of `cx.windows()` before its views go (`window.rs` `trail`), so the deletion now also
    needs the view's window still open, and moved from a `Terminal` observer to a
    `TerminalView` one.
  - What the replay can raise: only drawing passes the allowlist, so `Term` raises no `PtyWrite`,
    `ColorRequest`, `TextAreaSizeRequest`, `ClipboardLoad`, `ClipboardStore`, `Title` or `Bell`
    from it; what it raises waits in `events_tx` and would reach the live shell (D3).
  - A closed terminal's late output: the writer keeps the removed ids and drops their output, so
    the event loop's last read cannot make the file again.
  - The recorder reads every byte with the setting off too, so its alternate-screen state stays
    right when the setting turns on again; only the sending stops.
  - REQ-010: the builder's condition is an id (`task.is_none() && !is_remote_terminal`) and a PTY
    (`!no_pty`); `new_display_only` never reaches it.
  - Provenance: the allowlist is written from ECMA-48 and xterm's sequence list, the caps and the
    double filter from T3's report (MIT, research); nothing from Warp.
- **Gate:** `just gate-diff` GREEN, 17 passed, after the guides' edits (the in-app guide is in the
  receipt).

## Phase 3 — Test
- **Scenario:** `script/e2e/751-restored-scrollback.sh` (`compositor sway`); one run, every check
  passing. It ran on the build with both review fixes, before the gate; the gate's formatting and
  lints moved no behavior.
- **Shots:**
  - `751-01-before`: the first terminal's blocks: `seq 1 20000` ending at 20000, the red and the
    green line, `before`, `ask` with `asked` and a clean prompt.
  - `751-02-restored` (REQ-001, REQ-002, REQ-006): after the relaunch the same lines down to
    `asked`, then the old `$ `, then the new prompt. The restored rows have no block bar or header
    (the old prompts read `$ printf …`, `$ echo before`, as the shell printed them), and they are
    dim: the brightest restored pixel is 170 against 226 for the same text before the quit, the
    red and green dimmed too; the new prompt's cursor at full colour.
  - `751-03-clean` (REQ-003, REQ-006): `echo clean` became a block with its header (`…/repo ·
    main`, the green bar) and printed `clean`; nothing reached the prompt from the old queries.
  - `751-04-cap` (REQ-004): scrolled to the top, the first line is 15009: the newest 5,000 lines
    (4,992 numbers and the eight lines after them).
  - `751-05-fullscreen` (REQ-005): the second terminal: `echo second`, `second`, the id line, `$ less
    README.md`, a blank line and the new prompt; none of `less`'s screen.
  - `751-06-off` (REQ-009): with the setting off, the terminal came back with an empty screen and
    only the new prompt.
- **Checks:** `folder 700, files 600; 2 kept` (REQ-007); after the second terminal's close one file
  is left, the first terminal's, and the second's is gone (REQ-008); with the setting off `no file
  is kept` (REQ-009).
- REQ-010 by the review (the builder's condition).
- **Not reached by a scenario:** a crash's loss (at most a second of output; the scenario quits
  cleanly), and a window closed by the window manager (the scenario quits with `zed: quit`); the
  second rests on gpui's order of removal, read in `window.rs`.
- **Seen, not in scope:** after a restore the old prompts show as the shell printed them (`$ `),
  not as Marley's headers; the blocks' records are #764.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Added: earlier output after a restart); `docs/marley/guide.md` (Earlier
  output after a restart) and the in-app guide (its article, contents entry and settings row);
  `docs/marley_architecture/marley_workbench.md` (Terminal output across a restart),
  `marley_dcs.md` (Kept output, its consumer) and `terminal_blocks.md` (Kept output files);
  `docs/marley/zed-touchpoints.md`, the rows of `terminal.rs`, `alacritty.rs`,
  `settings_content/src/marley.rs`, `settings_ui/src/marley_page.rs` and `default.json` widened;
  `vendor/README.md` lists the vendored hunks.
- **Ledgers:** F-claude-751-a-restored-files-two-sessions-ran-together-001,
  F-claude-751-a-windows-close-read-as-every-terminal-closed-001,
  F-claude-751-marley-terminal-built-only-beside-crates-that-turned-on-rand-and-uuid-features-001,
  PR-claude-a-release-is-a-close-only-while-its-window-is-open-001,
  L-claude-751-a-cap-scenario-prints-its-markers-after-the-bulk-001,
  AD-claude-751-a-terminals-output-is-kept-as-filtered-bytes-and-replayed-dim-before-its-shell-001.
- **Brain:** consultation ffa0db78226843e8a85e670e5bc514dd, `brain decide`
  (`decisions/marley-a-terminals-output-is-kept-as-filtered-bytes-and-drawn-again-dimmed-before-its-restored-shell-starts-751`).
- **Next:** the blocks half is TICKET-764, already in the queue.
- Ticket closed, pipeline archived, committed.

