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

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
