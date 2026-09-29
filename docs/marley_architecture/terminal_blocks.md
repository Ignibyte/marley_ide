# marley_terminal (terminal_blocks) — UI-agnostic PTY session + Block model

**Status:** M1 · TICKET-010 · forge #14 · sprint M1.A seq 3/5 (the product core); **verified current
@ M15** (re-read against the code 2026-07-12 — the block model, the styled/OSC 8 carry, and the pure/shim
seam all hold). Contract: [`SPEC-terminal-blocks.spec.md`](../specs/SPEC-terminal-blocks.spec.md)
(R1–R30). **UI-agnostic** — `visual_acceptance: N/A`; the render is `app_shell` (#16).
**Provenance (§20).** The byte engine is `[permissive: alacritty_terminal + vte, Apache-2.0/MIT — REUSED
directly]`; the **Block model + the DCS wire format are `[Marley-original]`** — a clean-room
reimplementation of the Warp-derived block-terminal DESIGN (AGPL), never its source. The Warp reference is
[`03-terminal-session-core.md`](../warp_architecture/subsystems/03-terminal-session-core.md) (its
*Marley status @ M15* table rates this subsystem at/near Warp parity — Marley's strongest).

## Purpose

The PTY-backed shell **session** + per-command **Block model**. Spawn a shell, write commands, read its
output back, and segment that output into **Blocks** from shell **DCS/OSC hook** metadata (cwd, git
branch, exit code, subshell) — **not** output heuristics (R8) — producing a `BlockList` that any
front-end panel observes. Promotes the PTY/grid patterns proven by the M0 viability spike into a
real, reusable session. The Block model is the unit the **brain** later observes
(`AD-claude-brain-agent-session-supervision-001`).

## Shape — the pure / shim seam

**PURE (100% coverage + mutation MSI 100):**
- `dcs.rs` — the stateless DCS codec: `DcsEncoding{Hex,Plain,AnsiCQuoted}`, `DcsHook`,
  `encoding_for_dcs_terminator`, `decode_hook` (the `UndecodablePayload`-vs-`UnknownHook` split) and
  `decode_frame`. The **ordered `DcsScanner`** (a `Passthrough(bytes)`/`Hook(raw)` event stream in
  byte order, the fix for the coalesced-read ordering race) moved to
  [`marley_dcs`](marley_dcs.md) in #462, so the vendored `alacritty_terminal` can use it too. **R24 (TICKET-022):** an `AnsiCQuoted` payload is split
  into `name;key=value;…` on UNESCAPED separators (`split_unescaped`/`find_unescaped`,
  backslash-consumes-next) BEFORE per-piece `c_unescape` — so the rc's `\;` stays inside its field's
  value (`ls; pwd` round-trips; a `;`-containing `$PWD` can't truncate or inject phantom prompt
  fields). The original unescape-then-split order nullified ANY emit-side escaping (inspect-caught,
  `BF-claude-decoder-unescape-before-split-nullifies-emit-escaping-001`). Hex/Plain keep the legacy
  decode-then-split order; non-UTF-8 payload bytes remain `UndecodablePayload` (a documented M1
  limitation).
- `block.rs` — `BlockId`/`BlockIndex`/`ShellSessionId`/`BlockState`/`ExitCode`/`PromptInfo`/`Block`
  (`output_text` = pure join over the rows) + `BlockList` (the sole id/index allocator).
- `keys.rs` — **M1.D (#33):** the VT INPUT side (gpui-free, symmetric with alacritty owning output
  parsing) — `encode_key(KeyInput) -> Vec<u8>` (printable UTF-8/alt-ESC, `Ctrl+key` → `ctrl_byte`
  = `(c as u8) & 0x1f`, the named-key VT/CSI sequences) + `input_route(alt_screen, ctrl)` (Raw vs
  Cooked). `session.rs` gains `is_alt_screen` (DECSET-1049 via `term.mode()`) + `grid_styled_rows`
  (the live grid for the alt-screen render). What makes vim/top/less + Ctrl-C work (R25/R26).
  **M1.F (#41)** extends `encode_key` for inline interactive programs: `BackTab` (Shift-Tab `ESC[Z`),
  `F(u8)` (F1–4 SS3 / F5–12 CSI), `Insert`, and MODIFIED cursor keys — `KeyInput` gains `shift`, and
  the cursor keys route through `csi_cursor(modifier_param(shift, alt, ctrl), final)` emitting the
  xterm `ESC[1;<param><final>` (param = `1 + Shift + 2·Alt + 4·Ctrl`, a binary bijection) when
  modified, else the byte-identical legacy form. All new arms + the arithmetic are cov/MSI 100.
  **M17 #286** honors DECCKM (application cursor-key mode, DECSET 1): `encode_key` gains an
  `app_cursor` param (the `paste_bytes(text, bracketed)` precedent — terminal state as a param, not a
  `KeyInput` field); an UNMODIFIED cursor key (`param == 1`) switches CSI→SS3 (`ESC O <final>`) via the
  shared `cursor_key_bytes(final, app_cursor)` — a MODIFIED cursor key stays CSI even under DECCKM,
  per xterm. The same helper feeds the #280 alt-scroll wheel fallback (`mouse.rs`), and
  `session.is_app_cursor()` / `MouseModes.app_cursor` snapshot `TermMode::APP_CURSOR`. Wire-only (both
  forms drive vim identically); cov/MSI 100.
- block output trims trailing blanks — **M1.H (#50, BUG FIX):** a block's captured output was the
  full screen-height grid snapshot (`term_to_styled_rows` over all `screen_lines`, incl. the blank
  rows below the command's output), so a one-line command was a full-screen-tall block → history
  didn't stack ("every command clears the previous", chad live-testing). The block-output path now
  applies a pure `trim_trailing_blank_rows` (styled.rs, cov/MSI 100) — drop trailing all-blank rows,
  keep interior blanks — so each block is only as tall as its real output and blocks accumulate as
  scrollback. The alt-screen `grid_styled_rows` keeps the FULL grid. This also makes `output_styled`
  and `output_text` consistent (both trailing-trim). Inspect note: the pre-existing pump tests assert
  `output_text` (which trims independently), so they couldn't distinguish the fix — a direct
  `output_styled().len()` test is the real guard. (R19.)
- re-run a block — **M1.G (#46):** `Block::rerun_command()` (pure, cov/MSI 100) — `Some(command)` iff
  the block is Finished with a non-empty command, else `None` (a running/empty block is not
  re-runnable); `BlockList::last_rerunnable()` finds the most recent such command (drives cmd-R). The
  app-shell adds a ↻ header affordance + a cmd-R keybinding (`"rerun-last"` → `dispatch_action`); both
  resend via `write_command`, GUARDED by #40's `is_command_running` so a re-run never injects into a
  running program (the extract-then-write pattern ends the `blocks()` borrow before the `&mut` write).
  (R30; app-shell R50.)
- block copy actions — **M1.G (#45):** `Block::copy_text(BlockCopy)` (pure, cov/MSI 100) — `Command`
  returns the command line, `Output` returns `output_text()` (so a block-action copy and a drag-copy
  of the same output agree). The app-shell reveals hover-brightened `⧉ cmd` / `⧉ out` affordances on
  the block header; a click looks the block up by (pane, index) at click-time and writes `copy_text`
  to the clipboard (`stop_propagation` so it doesn't also start a selection). Note: cargo-mutants
  emits only whole-fn mutants for `copy_text`, so the `copy_text_arms_distinct` test (command ≠ output)
  is the load-bearing swap guard, not MSI. (R29; app-shell R49.)
- clipboard paste — **M1.F (#42, FINALE):** `paste_bytes(text, bracketed) -> Vec<u8>` (keys.rs,
  gpui-free) — WHEN a program holds `is_bracketed_paste()` (DECSET 2004, tracked like `is_alt_screen`),
  wrap the text in `ESC[200~`…`ESC[201~` so a multi-line paste is literal DATA (not each line run);
  else the raw bytes. SECURITY: every embedded `ESC[201~` is stripped in a LOOP until stable — a single
  `str::replace` reconstitutes a split marker (`ESC[20`+`ESC[201~`+`1~` → `ESC[201~`), a real
  paste-injection the inspect critic exploited ([[PR-claude-sanitizer-must-loop-until-stable-single-replace-reconstitutes]]).
  MSI is blind to the strip (only whole-fn-return mutants), so a hand-written split-marker regression
  golden guards it. The app.rs cmd-V handler routes via #40 (R28/R46).
- input routing by running-command — **M1.F (#40, BUG FIX):** `input_route(alt_screen, ctrl,
  command_running) -> Route` streams `Raw` to the PTY if ANY hold, driven by a new gpui-free
  `TerminalSession::is_command_running()` (`blocks().current().is_some()` — a `Running` block exists).
  #33 only routed on the ALTERNATE screen (vim/top), so INLINE interactive programs on the primary
  screen (Claude Code arrow menus, `read`) never got arrow keys — Marley's local history ate them.
  Now, while ANY foreground command runs, every key reaches it; the local editor + history apply only
  at the bare prompt. The `Running`-block bracket (Preexec→Precmd) is the exact "a command executes"
  signal; `sleep &` finishes at the prompt (no misroute), a REPL stays Running (streams). R26.
- `current_prompt` — **M1.E (#37):** `TerminalSession::current_prompt() -> Option<&PromptInfo>`
  (delegating to `SessionModel::current_prompt` = `staged_prompt.as_ref()`) exposes the context a
  `Precmd` staged for the LIVE prompt (the cwd/git the next command runs in) — `Some` after a precmd,
  `None` before any and once the following `Preexec` consumes it. Feeds the app-shell prompt-row
  cwd/git segments (app-shell R43); stays gpui-free (returns `PromptInfo`, no `Hsla`). R27.
- `styled.rs` — **M1.D (#31):** the styled-output model — `StyledRun`/`StyledLine` carry alacritty's
  own `Color`/`Flags` (this crate stays gpui-FREE; the `Color → Hsla` step is up in marley_app's
  `color.rs`), and `coalesce_row` merges adjacent same-style cells into runs + trailing-trims so
  `Block::output_text()` stays byte-identical. `Block.output` is now `Vec<StyledLine>` with an
  `output_styled()` accessor; `session.rs::term_to_styled_rows` feeds it. `Color`/`NamedColor`/`Rgb`/
  `Flags` are re-exported for the render. **M12.2 (#214, SHIPPED):** `StyledRun` also carries `hyperlink:
  Option<String>` (the OSC 8 explicit-hyperlink URI); `coalesce_row` breaks a run on a hyperlink change too
  (so `Block::output_text()` stays byte-identical — only run boundaries shift), and both `term_to_styled_rows`
  (live grid) + `full_term_to_styled_rows` (command-finish) extract `cell.hyperlink()` from the grid. This
  crate owns only the hyperlink **DATA**; the clickable-link **render** — the text-scan `scan_links` heuristic
  (#196) and the explicit-span `line_links` composer that prefers an OSC 8 URI over it (#214) — lives up in
  `marley_app::links` (painted by `app_shell`), and the browser open goes through `marley_command::open_url`
  (the http/https-guarded seam). So the terminal→link half is SHIPPED end-to-end across the three crates.
- `apply.rs` — `SessionModel` + `apply_hook` (the state machine: InitShell maps the shell id, Preexec
  opens a Running block from the staged prompt, Precmd finishes it + stages the next, Bootstrapped; the
  `MissingSession` guard leaves the BlockList unchanged).
- `anchored.rs` (#464, written in the fork) — `AnchoredBlocks`, the same transitions for Zed's
  terminal, where a block records absolute lines instead of copying its output: `prompt_line`,
  `output_start` and `output_end` from each hook's position. Zed's `Terminal::block_output` reads the
  lines from the main screen's grid while they are held (`main_grid` and `main_bounds_to_string`,
  vendored, so a read while a full-screen program shows reads the blocks and not the alternate
  screen, #546). A resize that changes the width rewraps the grid and
  moves its rows, so `AnchoredBlocks::rewrap` (#544) carries every anchor (the three per block,
  the staged prompt's line, the input's start with its column) across it: each becomes a place
  alacritty's rewrap keeps, logical lines from the cursor's logical line and a character offset
  inside its own, read from a `RowsView` of the main screen's rows (`marley_rows_view`, each row's
  `WRAPLINE`) before and after `resize`, in Zed's Resize arm; an anchor whose line fell off a full
  history reads as evicted, which the vendored `shrink_columns` now counts. `visible_spans` (#470) maps the blocks to a viewport's
  rows: a block starts at its prompt's line, else its output's, and ends before `output_end`, or
  after the cursor's line while it runs. Zed's `TerminalElement` draws them in stage one (the
  plan's D3): `Content::marley_screen_top` (evicted lines plus history) less `display_offset` is
  the viewport's top; `marley_block_spans` gives no spans on the alternate screen; `paint` draws
  a wash (running `info`, failed `error`, faint) after the cells' backgrounds, and a two-pixel
  bar in the one-cell gutter and a pill at the right end of the first row after the text.
  `block_scroll` (#473) gives the scroll offset that puts the start of the last block above the
  viewport's top, or of the first below it, at the top: 0 on the live screen, within the
  history, `None` with no block that way; `marley_workbench::blocks` scrolls to it.
  `marley_block` (#474) gives each block whose first row is on screen one element over its
  rows, with a hover group; the first row holds the pill and, while the pointer is over the
  block, Copy (`Terminal::block_output` to the clipboard) and, while the last block is
  finished and the command is verified, Rerun (Ctrl-U, the command and a carriage return through
  `Terminal::input`). A block's command is verified when its `preexec` frame carried the
  terminal's nonce (`AnchoredBlocks::with_nonce`, `AnchoredBlock::command_verified`): Zed's
  builder gives each local terminal's program one in `MARLEY_SHELL_NONCE` from
  `shell_integration::new_nonce`, and the scripts unset it before the user's files run. Each
  button sits in `marley_keep_from_terminal`, which stops a left press from reaching the
  terminal's listeners; `ButtonLike` stops only its click's release, and an occluding hitbox
  would end the group's hover under the button.
  `bottom_shift` (#476) says how many rows down a viewport is drawn so the live screen's last
  used row sits on the bottom edge: the rows below it (`Content::marley_empty_bottom_rows`, read
  from the grid in `make_content`, the cursor's row counted as used) less the display offset,
  and none on the alternate screen. `TerminalElement::prepaint` moves the grid's origin by it
  after `sync`, plus the padding its snapped rows leave, and stores the moved bounds with a
  second `set_size`, which the mouse maps through and which resizes nothing.
  For autosuggestions (#484), `AnchoredBlocks::at_prompt` says a prompt is staged,
  `note_input` (which Zed's `Terminal::input` calls with the cursor's absolute point, off the
  alternate screen) keeps the first point typed at after it as `input_start`, which `Precmd` and
  `Preexec` clear, and counts every input it notes (`inputs`, #554: a selected block stays
  selected while the count holds), and the `History` hook (`history;file=`, which both scripts send after `init`
  with `$HISTFILE`) keeps the shell's history file. `suggest.rs` has `suggestion`, the rest of
  the first history command, newest first, that starts with the typed text and stays on one
  line, and `parse_history`, bash's lines without their `#<seconds>` lines and zsh's extended
  `: <seconds>:<elapsed>;` lines with backslash continuations joined.
  `filter.rs` (#528) filters a block's output lines: `filter_lines(output, &FilterQuery)` with
  text or a regex (`regex`), case ignored unless asked, invert, and context with a `Gap` between
  groups, grep's meanings; nothing in the grid changes.
  `sticky_block(spans, display_offset)` (#529) is the block whose command a scrolled-back view
  pins over its top row: the first span, when it covers row 0 and started above it; none at the
  live screen.
  `block_lines(block, cursor_line)` (#559) is the absolute lines a block spans, `visible_spans`'s
  rule, and `scrollback_fraction` where a line sits among the lines the terminal can scroll to;
  Zed's view maps the first to grid lines for a search held to one block, and the element draws
  a bookmark's tick at the second.
  `workflow.rs` (#558) turns a command into a workflow's template: `guess` replaces, after the
  first word, a number (`port` after `-p` or `--port`, after a `:`, or from 1024 to 65535 with
  no other flag before it; else `number`), a URL, the branch and an existing path with
  `{{name}}`, the token its default, and a repeat of a kind `port2`, `port3`; quoted tokens stay.
  `params_of` lists a template's parameters and `substitute` fills them; `is_name` is the rule
  for a parameter's name, never Zed's `ZED_` prefix.
  `AnchoredBlock::markdown(output, took)` (#554) writes a block for a note or a message: a fence
  one backtick longer than any run inside, `$ ` and the command, the output or a line saying it
  is gone (no line at all for empty output, #555), then `exit N · took · folder (branch)`.
  `stamp` (#491), which Zed's `Terminal::apply_shell_hook` calls after each hook applies, keeps
  each block's `BlockTimes` beside it: its start, when the `Preexec` that opened it was applied,
  and its end, when the hook that finished it was; a busy main thread moves a stamp by tens of
  milliseconds. `times(index)` reads them for the MCP server's `terminal_blocks`, with
  `Terminal::block_output_kept`, whether a block's first output line is still held.
- Blocks over ssh (#526): every frame the scripts print carries its shell's nonce; `dcs.rs` wraps
  a frame's hook in `DcsHook::Signed { nonce, hook }` (a command's keeps its nonce in
  `PreexecValue`), and `DcsHook::Remote { host, session }` is Marley's `ssh` announcing a
  connection. `AnchoredBlocks` sorts each frame by its nonce (`Local`, the connection's host, or
  unknown): only the local shell's `remote` opens a connection, the host's `init` ends the `ssh`
  block with no exit code, the local shell's next `precmd` ends the connection, each block keeps
  its host (`block_host`), the staged prompt its shell (`prompt_shell`, a `PromptShell`), and
  `rerun_offered(block)` holds only while the block's own shell waits at the prompt; a history
  file is taken from the local shell only. The gpui-era `apply.rs` and `session.rs` read
  `into_unsigned`. `shell_integration.rs` builds `ssh_bootstrap()` (POSIX sh writing both scripts
  into `mktemp -d` and starting the host's bash or zsh with the connection's nonce) and
  `ssh_remote_command()` (one line, `sh -c` decoding the bootstrap from base64), written to
  `ssh-remote-command` and named to the scripts in `MARLEY_SSH_COMMAND`. The scripts' `ssh` walks
  ssh's options, asks `ssh -G` about `RemoteCommand` and `Tag marley-plain`, mints the connection's
  nonce from `/dev/urandom`, prints the `remote` frame and runs `ssh -t … "<command> <nonce>"`; on
  the host, `__MARLEY_CLEANUP` removes the folder at the scripts' top, `__MARLEY_LOGIN` makes bash
  read the login files, and neither defines `ssh`.
- `shell_integration.rs` (#463, written in the fork): the embedded `shell_integration/marley.bash`,
  `install_in(dir)`, which writes it when its content changed, and `for_program`, which gives bash
  `--rcfile` and `MARLEY_SHELL_INTEGRATION=1`. Zed's `TerminalBuilder::new` applies it to a local
  interactive `System` or `Program` shell (`marley_shell_integration` in `crates/terminal`). The
  script sources `~/.bashrc`, then prepends `__marley_precmd` to `PROMPT_COMMAND` (keeping `$?`)
  and appends `__marley_preexec` to `PS0`. `shown_arguments(argv, dir)` (#467) is a process's
  arguments less the run `for_program` adds; Zed's `Terminal::title` lists those, so a shell
  Marley started reads as the same shell started without the integration. zsh (#465) takes no
  argument: `for_program` gives it `ZDOTDIR=<dir>/zsh`, where `install_in` writes
  `shell_integration/marley.zsh` as `.zshenv`, and hands the user's own `ZDOTDIR` on in
  `MARLEY_ZSH_ZDOTDIR`. The script restores it, sources the user's `.zshenv`, and at the first
  prompt puts `__marley_precmd` first in `precmd_functions` and `__marley_preexec` last in
  `preexec_functions`.
  Since #561 it also keeps the opener a new local terminal gives its programs as `BROWSER`
  (`set_browser_opener`, `browser_opener`, a process-wide setting the workbench sets, none under
  `system_browser`).
- `ports.rs` (#590): `PORT_OFFSET_VARIABLE` (`MARLEY_PORT_OFFSET`), `PORT_VARIABLE`, `PORT_BASE`
  3000 and `PORT_STEP` 10; a process-wide `SlotReader` the workbench sets (`set_slot_reader`,
  `slot_reader` to box an async function); `variables(folder)`, awaited by both of Zed's terminal
  builders between the directory's environment and `terminal.env`, which asks the reader for the
  project's first folder and gives the offset and the port, or nothing.
- `links.rs` (#579): `joined_url(rows, row, column, last_column)`, pure. It joins a URL a program
  wrapped itself at the right edge, or drew inside a box frame, from the `LinkRow`s Zed's
  `hyperlinks.rs` builds around a point.
  - The edge rule: a row filled to the last column runs on unless the next starts with a space, a
    new scheme or a `label:` form, or holds a frame.
  - The frame rule: the same frame columns and prefix on each row, each part ending in a
    continuation character or filling most of the width.
  - Both follow Orca's. Zed keeps the joined URL when it is longer than the row's own match, and
    marks every row of an edge-wrapped one and the clicked row of a framed one.
- `session.rs` — `classify_write` + the `PtyChannel`-trait `TerminalSession` (`write_bytes` re-queue,
  `pump`, `resize`) — unit-tested via a `MockPtyChannel` (the logic is reachable headlessly).
  **M1.C (TICKET-023):** `pump` gained an IDLE FAST-PATH — a LEADING `WouldBlock` (nothing read this
  call) returns immediately; the ~1 ms retry windows apply only MID-BURST after bytes were read. The
  unconditional retry sleeps cost ~12 ms per idle pane per call, which the multi-pane pump-all timer
  multiplied into 98 ms/16 ms tick at 8 panes (measured;
  `BF-claude-pump-idle-sleep-floor-times-panes-frame-killer-001`) — post-fix: 19.7 µs. `PtyChannel`
  is now `Send` (supertrait) so a whole `TerminalSession` can move to a reaper thread if wanted.
- **Bounded teardown (TICKET-348).** The child reap was DELEGATED to `alacritty_terminal`'s
  `tty::Pty::Drop`, which `SIGHUP`s then blocks on an **unbounded `child.wait()`** — the source of an
  11-hour wedged gate (`resize_real_pty_succeeds`, SLOW >39,240s at 0% CPU). Now `OsPtyChannel::drop`
  runs an EXPLICIT bounded reap: the pure clock-injected `reap_step` (session.rs, cov/MSI 100) escalates
  `SIGHUP` → 2s → `SIGKILL` → 2s → give-up; the shim pre-empts alacritty's `Drop` by holding the `Pty` in
  an `Option` and `mem::forget`ting it on the give-up path (so the blocking `Drop` never runs — D5
  zombie-over-hang; keeps the crate `unsafe`-free). On success the child is already reaped, so alacritty's
  `child.wait()` returns from cache at once. The `next_child_event` poll is **edge-triggered** (it consumes
  the one `SIGCHLD` self-pipe byte), so `poll_child_exit` latches `child_reaped` — teardown of a child that
  already exited during pumping is instant, not a 4s loop. The systemic backstop is the `.config/
  nextest.toml` terminate ceiling (60s SLOW → 180s SIGKILL, terminated test FAILS) across every nextest lane.
- **Exit-order independence + the post-exit write contract (TICKET-423).** Two Linux realities the
  macOS-shaped pump missed: (1) the slave fds die WITH the child, so the master's `EIO` routinely
  arrives BEFORE the SIGCHLD self-pipe byte — `pump` now treats an `EIO` read as "exit pending"
  for a bounded, NON-BLOCKING `EIO_EXIT_GRACE` (1 s, measured ACROSS pump calls via `eio_since`;
  the pure `eio_grace_expired` owns the boundary — `elapsed == GRACE` counts expired, unit-pinned)
  before surfacing the no-exit `Disconnected`; once `ChildExited` HAS been observed a later `EIO`
  is an instant `Err` again (the alacritty `event_loop` EIO-`continue` precedent, adopted
  Apache-2.0, restructured to Marley's per-tick pump). (2) Linux masters ACCEPT post-close writes
  (flip-buffer, no reader), so errno can never carry the post-exit write contract — the session
  latches `child_exited` when the exit event surfaces and `write_bytes` refuses by STATE first,
  uniformly on every platform. Every new branch is `MockPtyChannel`-unit-covered (raw-errno read
  pushes; MSI 100 on the diff).

**ACCEPTED-UNTESTABLE (the ONLY shim — `pty_os.rs`, `mutants::skip` + a documented `rust_cov` exclude):**
the four raw OS calls — `tty::new` spawn (HOLDS the `Pty` so output isn't discarded), leader-fd
read/write, `rustix::termios::tcsetwinsize` (the winsize ioctl), `Pty::next_child_event` (SIGCHLD reap +
exit code). The crate is **`unsafe`-free**.

**In the fork (#443, 2026-09-22).** The `mutants::skip` masks did not survive the port, and the
end-of-sprint mutation run (`script/mutation.sh`) is unmasked, so `pty_os.rs` is mutated like
every other file. Six added real-PTY integration tests kill its mutants: the program and its
arguments, the working directory, the environment, the window size, the SIGKILL escalation for
a child that ignores SIGHUP, and descriptor cleanup after a reap. The file stays on gate:4's
coverage exclude list for its OS-error arms. The real-PTY tests share a process-wide lock in
place of `#[serial]`.

**Since #447.** The shim is a child module of `session`, the one module that calls it:
`#[path = "pty_os.rs"]` keeps the file where gate:4's exclude and these notes name it, and its
items are `pub(super)`. The reap signal logs any failure except ESRCH, the child having
already exited. The pump logs a shell hook it drops for arriving before `InitShell`.

## Key decisions
- **UI-agnostic** (no gpui) — the render is `app_shell`/a panel (M1.B/M2).
- The DCS wire format is **clean-room INVENTED** (behavior-derived per the spec's `spec_source:
  behavior-only`): `ESC P <h|p|q> name;key=value ESC\`.
- **`rustix::termios::tcsetwinsize`**, not alacritty's `on_resize` (which *aborts the process* on
  failure) → keeps the crate `unsafe`-free AND makes `SessionError::Resize` reachable/killable.
- The `PtyChannel` trait seam → the re-queue/pump/resize LOGIC is mock-unit-tested; only `pty_os.rs`'s
  4 raw fns are coverage-excluded.

## Verification
108 unit tests (27 dcs + 16 block + 13 apply + 36 session + 10 keys + 6 styled, via `MockPtyChannel`) +
5 real-PTY `#[serial]` integration tests with a stub DCS shell (incl. the #214 OSC 8 end-to-end
extraction). The coalesced-read ordering guard proves the ingest fix; cov 100 / MSI 100 (gate-enforced).

## Deferred (M2+)
The out-of-process terminal server + SCM_RIGHTS fd passing; the `local_control` RPC drive surface (the
`session.read` brain-tap, M3); Windows ConPTY; kitty-keyboard. **Shipped since M1:** the gpui render
(`marley_app`), clipboard bracketed-paste (#42), and the terminal→editor/browser link half — clickable
`file:line:col` / URLs (#196, text-scan) + OSC 8 explicit hyperlinks (#214, `StyledRun.hyperlink`).

## See also
- [crate-map.md](crate-map.md) — the Warp lineage (`warp_terminal` → REIMPLEMENT, on `alacritty_terminal`).
- [`warp_architecture/subsystems/03-terminal-session-core.md`](../warp_architecture/subsystems/03-terminal-session-core.md) — the **reference** Marley reimplements clean-room; see its *Marley status @ M15* parity table + *Provenance & licensing* (Block model = `[Warp-derived: AGPL]` design, reimplemented; alacritty byte engine = `[permissive]`; `terminal_blocks` = `[Marley-original]`).
- [app_shell.md](app_shell.md) — the gpui render: the block/prompt UI, the `marley_app::links` clickable-link composer (#196/#214), and the `color.rs` `Color → Hsla` mapping.
- [SPEC-terminal-blocks.spec.md](../specs/SPEC-terminal-blocks.spec.md) · the M0 viability spike whose PTY/grid patterns it promotes (the spike crate was deleted once M1 shipped).
