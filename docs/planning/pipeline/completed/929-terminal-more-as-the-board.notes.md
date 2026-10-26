# Table tabs, restored blocks, dropped links and the mic as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-929-terminal-more-as-the-board.md
- **Pipeline spec:** 929-terminal-more-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 11 (TerminalMore).
- **Classification / tier:** feature, S–M. Marley crates only.
- **Recall (§18.3):**
  - AD-claude-764-a-restored-block-is-placed-from-a-mark-in-the-kept-bytes-001 and
    AD-claude-751-a-terminals-output-is-kept-as-filtered-bytes-and-replayed-dim-before-its-shell-001:
    restored blocks sit at the front of the list; the divider's row comes after the replay, so it
    must not shift the blocks' anchored lines (it is written after the last mark).
  - F-claude-751-a-restored-files-two-sessions-ran-together-001: the replay's end is the place a
    new session starts; the divider marks that same place.
  - AD-claude-641-ssh-links-know-they-are-dead-by-keepalive-and-a-checked-reattach-001 and
    F-claude-641-a-down-terminals-count-did-not-redraw-during-a-check-001: the strip redraws from
    its own ticker; Try Now must end the wait without starting a second check loop.
  - AD-claude-543-remote-terminals-in-marleys-tmux-001: every remote terminal runs in Marley's
    tmux on the host, which is what the reassurance line says.
  - AD-claude-640-a-harness-sessions-labels-are-drawn-and-only-declared-states-act-001: an
    undeclared state already gets a muted dot and its source word; this ticket mutes the line too
    and words `detected` as "read off the screen".
  - AD-claude-480-marley-drives-voxtype-and-follows-its-status-001: Voxtype gives its state only,
    so no waveform (Out).
  - L-claude-804-place-each-row-at-its-grid-rows-top-001: the table header sits outside the
    table's grid and must not change its row placement.
  - F-claude-903-a-stand-in-not-on-the-terminals-path-001: stand-ins first on the terminals' PATH.
- **Discovery:**
  - `crates/marley_workbench/src/block_tables.rs`: `open_output` (642), `add_table_tab` (680),
    `show_table` (712), `BlockTable` (784), `render` (799).
  - `crates/marley_terminal/src/tables.rs`: `TableKind` (53).
  - `crates/marley_workbench/src/scrollback.rs`: `restored` (162); `block_headers.rs`: `header`
    (124); `crates/marley_terminal/src/anchored.rs`: `restore` (468), `restored_count` (490).
  - `crates/marley_workbench/src/remote.rs`: the checks (383), `link_overlay` (524), `Phase`.
  - `crates/marley_workbench/src/agent_bar.rs`: the microphone button (380–403),
    `microphone_look` (407); `voice.rs` (`VoiceState`).
  - `crates/marley_workbench/src/rail.rs`: `harness_row` (8140); `harness.rs` `Source::word`
    (1359).
- **Decisions:** D1–D5 in the spec.

### Design
- Table header: `BlockTable` gains `kind: Option<TableKind>`, `rows: Option<usize>` (set when the
  parse lands) and `source: SharedString`. `render` is a `v_flex` of a header `h_flex` (a chip
  with the kind's short word in mono on `element_background`, the name in default weight, the
  count line muted) and the table. `kind_word(TableKind) -> &'static str`: `Delimited(',')`
  "csv", `Delimited('\t')` "tsv", other delimiters "table", `JsonArray` "json", `JsonLines`
  "jsonl", `Markdown` "md".
- Divider: `AnchoredBlocks` gains `restored_at: Option<RestoreMark>`, a struct with named fields
  `line` (the absolute line after the replay) and `time` (`SystemTime`), set by `restore`. The
  replay ends with one empty line so the row exists. The block-header hook draws the divider for
  the first live block's prompt when `restored_count > 0`, over the reserved row above that
  prompt. If the hook cannot reach a row outside a block's prompt rows, the Code phase adds the
  divider as a second element the same hook returns, placed at `restored_at.line`, and records
  the choice in Phase 2.
- Link strip: `link_overlay` builds "<host> stopped answering" (with the reason in the tooltip),
  "checking again in N s" or "checking now", "N keys held back"; a `Button` "Try Now" calls
  `remote::check_now(key, cx)`, which wakes the waiting backoff (a `oneshot` the wait selects on)
  so the loop checks at once; a second muted line under the strip for the tmux sentence.
- Microphone: while `Recording` or `Transcribing`, the `IconButton` becomes a `Button` pill
  with a dot (`Color::Error` recording, `Color::Warning` transcribing) and the words; the tooltip
  and the click stay.
- Screen-read row: in `harness_row`, when `weaker`, the state word and the line's colour are
  `Color::Muted` and the mark for `Source::Detected` is "read off the screen".
- File manifest (all Marley crates): `block_tables.rs`, `scrollback.rs`, `block_headers.rs`,
  `crates/marley_terminal/src/anchored.rs`, `remote.rs`, `agent_bar.rs`, `rail.rs`.
- Ledger: none (no Zed path).

### Visual check plan
- REQ-001: a command printing CSV, then the block's Open as Table; `929-01-table`.
- REQ-002: two commands, then the scenario quits Marley and starts it on the same profile (as
  #764's scenario does); `929-02-restored`.
- REQ-003: a remote terminal on #445's fake SSH host; the host's sshd stopped; two keys typed;
  `929-03-link`. The host started, Try Now clicked: `929-04-try-now` shows the terminal attached.
- REQ-004: the stand-in Claude Code in a terminal, the stand-in Voxtype set to `recording` when
  the microphone is clicked (as #480's scenario does); `929-05-mic`.
- REQ-005: a stand-in harness session with `state.source = detected` (as #640's scenario serves
  one); `929-06-screen-read`.
- Marley Day: `929-07-day`.

### Risks
- The divider's reserved row is one more line in a restored terminal's scrollback; it must not
  be kept again by the recorder, or each restart would add one. The recorder skips it, or the
  replay writes it after the recorder's cut.
- Quitting and restarting Marley inside a scenario leaves the restored session to the run's own
  profile; #764's scenario already does this.

## Phase 2 — Code
- **Built:** `BlockTable`'s `facts`, `source`, `set_table`, `kind_word` and header;
  `AnchoredBlocks::restored_at` and `send_block::restart_divider`; `remote::check_now`,
  `WAKE_STEP` and the strip's words, Try Now and tmux line; `agent_bar::MicrophoneLook` and the
  pill; `rail::row_marks` with "read off the screen".
- **Deviations:** the divider is the footer row of the last restored block while no command has
  run (the footer hook already reserves a row above the live prompt), so no row is left in the
  replay and no Zed path changes; it goes once a command runs. Try Now acts on a waiting check
  only (D3). The line for a stopped link keeps "not sent" for its count.
- **Gate:** `just gate-diff` 17/17 green.

## Phase 3 — Test
- `script/e2e/929-terminal-more-as-the-board.sh` (sway), green:
  - `929-01-table`: the header "csv · printf 'crate,tests\\nrail,…' · 3 rows · from printf …" over
    the table. REQ-001.
  - `929-02-restored`, `929-07-day`: three restored blocks, then "Marley started again · 20:02"
    between rules over the live prompt, in Marley Night and Day. REQ-002.
  - `929-05-mic`: "Listening · Voxtype" in a red-dotted pill on the stand-in Claude Code's bar.
    REQ-004.
- `script/e2e/929-dropped-link.sh` (sway, #641's sshd), green:
  - `929-03-link`, `929-07-day-link`: "e2e stopped answering · checking now · 9 keys held back"
    (Day: "checking again in 1 s"), Try Now, and the tmux line. REQ-003.
  - `929-04-try-now`: reattached, `remote$` back. The ssh log: the failed check ended at
    …858.66 and the next began at …860.66, 2 s later, where the backoff after several failures
    waits 4 s or more: Try Now's check. REQ-003.
- #640's scenario, `640-01-rows`: watcher and asker grey with "read off the screen". REQ-005.
- The UI proof named one scenario; the link has its own, built on #641's sshd, and the screen-read
  row is #640's own fixture.

## Phase 4 — Complete
- Docs: CHANGELOG; `marley_workbench.md` (remote link, harness marks, voice, restart divider);
  `terminal_blocks.md` (the table header); the look plan's TICKET-826 line. No Zed path.
