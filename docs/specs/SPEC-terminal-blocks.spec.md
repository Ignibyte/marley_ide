---
spec_id: terminal-blocks
component: marley_terminal
bucket: REIMPLEMENT
milestone: M1
status: draft
title: Block model — per-command grid + shell-hook DCS metadata; PTY write + output read-back
goal: Spawn a shell session, write bytes to it, and stream its output back into per-command Blocks whose boundaries and metadata come from shell DCS hooks, not heuristics.
reuses: [alacritty_terminal, vte]
spec_source: behavior-only — observable I/O of a PTY-backed shell session that segments output into per-command blocks from shell-emitted DCS/OSC hook metadata (cwd, git branch, exit code, originating subshell) and supports byte/command write + output read-back. No fork file paths, no private module/type/static names. Seam ownership per standards/seam-contracts.md §2 (SessionId) and §4 (hook error surface + PTY spawn).
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
---

## Purpose

`marley_terminal` is the UI-agnostic terminal session and block model. It spawns a PTY-backed
shell, accepts byte/command writes, reads the shell's output back, and segments that output into
**Blocks** — one per command — using the shell bootstrap's DCS/OSC hook metadata (cwd, git branch,
exit code, originating subshell) rather than output heuristics. It owns no rendering and no GUI; it
produces a `BlockList` that any front-end panel can observe. The PTY byte engine and ANSI cell grid
are supplied by the reused `alacritty_terminal` (with `vte` as its parser); this component adds the
block-segmentation, DCS-hook decoding, and write/read-back coordination on top.

The session's process-unique identity is **`marley_core::SessionId`** (the one workspace session
counter, owned by `marley_core` per seam-contracts §2). `marley_terminal` allocates one such id when
a session spawns and stamps it onto every block; it does **not** define its own session counter. The
id a shell *self-reports* in its bootstrap `InitShell` hook is a different thing — a correlation-only
`ShellSessionId` with no allocation API — and is mapped to the session's `SessionId` for subshell
detection.

PTY spawn is owned end-to-end by this component via `alacritty_terminal::tty` (its own
openpty + fork + exec). Per seam-contracts §4.2 there is **no dependency on `marley_command`** for the
PTY path; `marley_command` is the workspace's separate non-PTY child-process spawn seam.

## Public surface (the contract)

```rust
// `marley_core::SessionId` is the process-unique session counter (seam-contracts §2).
// This crate imports it; it does NOT redefine a `SessionId` newtype.
use marley_core::SessionId;

// Identity / ordering
pub struct BlockId(u64);
pub struct BlockIndex(usize);

// The id a shell SELF-REPORTS in its `InitShell` bootstrap hook. Correlation only —
// NO `next()`/allocation API. Mapped to the session's `marley_core::SessionId` by `apply_hook`.
pub struct ShellSessionId(u64);            // Copy + Clone + Eq + Hash + Debug

// Per-command record
pub enum BlockState { Pending, Running, Finished }
pub struct ExitCode(Option<i32>);
pub struct PromptInfo {
    pub pwd: Option<String>,
    pub git_branch: Option<String>,
    pub virtual_env: Option<String>,
    pub node_version: Option<String>,
}
pub struct Block {
    pub id: BlockId,
    pub index: BlockIndex,
    pub session_id: Option<SessionId>,     // the session's `marley_core::SessionId`, never the shell id
    pub command: String,
    pub state: BlockState,
    pub exit_code: ExitCode,
    pub prompt: PromptInfo,
    // output cells held in an alacritty_terminal grid
}
impl Block { pub fn output_text(&self) -> String; }

// Ordered collection
pub struct BlockList { /* … */ }
impl BlockList {
    pub fn len(&self) -> usize;
    pub fn get(&self, index: BlockIndex) -> Option<&Block>;
    pub fn current(&self) -> Option<&Block>;
    pub fn iter(&self) -> impl Iterator<Item = &Block>;
}

// Shell DCS hook schema + STATELESS codec (seam-contracts §4.1)
pub enum DcsEncoding { Hex, Plain, AnsiCQuoted }
pub enum DcsHook {
    InitShell { shell_session_id: ShellSessionId },  // shell-reported id, correlation only
    Precmd(PrecmdValue),       // pwd/git/venv/node + exit code of the just-finished command
    Preexec(PreexecValue),     // command text starting to execute
    Bootstrapped { is_subshell: bool },
}

// Encoding SELECTION is observable and tested on its own (seam-contracts §4.1; review-r1 R9 HIGH):
pub fn encoding_for_dcs_terminator(terminator: u8) -> Option<DcsEncoding>;

// STATELESS free fn — codec only, NO registry / BlockList access. Returns ONLY codec errors.
pub fn decode_hook(encoding: DcsEncoding, payload: &[u8]) -> Result<DcsHook, DecodeError>;
pub enum DecodeError { UndecodablePayload, UnknownHook }

// Session: spawn / write / read-back / stateful hook application
pub struct SessionOptions { pub shell: PathBuf, pub cwd: PathBuf, pub env: Vec<(String, String)>, pub cols: u16, pub rows: u16 }
pub enum SessionError { Spawn, Write, Disconnected, Resize }
pub enum SessionEvent { Wakeup, ChildExited(ExitCode) }

// STATEFUL — owns the session registry (ShellSessionId -> SessionId), the staged-prompt
// buffer, and the BlockList. This is where `MissingSession` and the
// "leave the BlockList unchanged on error" guarantee live (seam-contracts §4.1).
pub enum ApplyHookError { MissingSession }

pub struct TerminalSession { /* … */ }
impl TerminalSession {
    pub fn spawn(options: SessionOptions) -> Result<Self, SessionError>; // allocates SessionId::next()
    pub fn session_id(&self) -> SessionId;                               // the session's core id
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), SessionError>;
    pub fn write_command(&mut self, command: &str) -> Result<(), SessionError>; // appends CR,LF
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), SessionError>;
    pub fn pump(&mut self) -> Result<Vec<SessionEvent>, SessionError>;   // read+parse; decode_hook then apply_hook
    pub fn apply_hook(&mut self, hook: DcsHook) -> Result<(), ApplyHookError>; // stateful: drives the BlockList
    pub fn blocks(&self) -> &BlockList;
    pub fn shutdown(self);
}
```

## EARS Requirements

- **R1.** The system shall assign each `Block` a `BlockId` that is unique within its `TerminalSession`.
- **R2.** The system shall order `BlockList` entries by strictly increasing `BlockIndex` matching the order in which their commands began executing.
- **R3.** WHEN a `Preexec` DCS hook is applied via `apply_hook`, the system shall open a new `Block` in `BlockState::Running`, set its `command` to the hook's command text, and initialize its `PromptInfo` from the staged-prompt buffer (consuming and clearing that buffer).
- **R4.** WHEN output bytes arrive after a `Preexec` hook and before the next `Precmd` hook, the system shall append those bytes' rendered cells to the current `Running` block's output grid.
- **R5.** WHEN a `Precmd` DCS hook is applied while a block is `Running`, the system shall set that block's `state` to `BlockState::Finished` and set its `ExitCode` to the hook's exit-code value.
- **R6.** WHEN a `Precmd` DCS hook is applied, the system shall populate the staged-prompt buffer — holding the `PromptInfo` (`pwd`, `git_branch`, `virtual_env`, `node_version`) for the *next*, not-yet-opened block — from the hook's fields, so that the next `Preexec` (R3) initializes the block it opens from this buffer.
- **R7.** WHEN an `InitShell` DCS hook is applied, the system shall record the hook's `ShellSessionId -> SessionId` correlation in the session registry (the session's `SessionId` being the one allocated at `spawn`) and stamp `session_id` on every subsequently opened block with that `SessionId`.
- **R8.** The system shall segment output into blocks only from decoded DCS/OSC hooks and shall not create a block boundary from any output-content heuristic.
- **R9.** WHEN `decode_hook(encoding, payload)` is called, the system shall decode `payload` under the supplied `DcsEncoding` (`Hex`, `Plain`, or `AnsiCQuoted`) and return the corresponding `DcsHook`.
- **R10.** IF a DCS hook payload cannot be decoded under its supplied `DcsEncoding`, THEN `decode_hook` shall return `DecodeError::UndecodablePayload` (it is stateless and mutates no model).
- **R11.** IF `apply_hook` is given a `Preexec` or `Precmd` hook but no `InitShell` hook has yet registered a `ShellSessionId` for the session, THEN `apply_hook` shall return `ApplyHookError::MissingSession` and leave the `BlockList` unchanged.
- **R12.** WHEN `spawn` is called, the system shall allocate the session's `marley_core::SessionId` via `SessionId::next()`, allocate a PTY leader/follower pair via `alacritty_terminal::tty`, start the shell from `SessionOptions`, and return a `TerminalSession` handle.
- **R13.** WHEN `write_bytes` is called, the system shall write all supplied bytes to the PTY leader, re-queueing any unwritten remainder of a partial write until the full buffer is written.
- **R14.** WHEN `write_command` is called, the system shall write the command text followed by the carriage-return then line-feed bytes (`\r\n`) to the PTY leader.
- **R15.** IF a write to the PTY leader fails because the child has disconnected, THEN the system shall return `SessionError::Disconnected`.
- **R16.** WHILE a session is alive, WHEN `pump` is called and the PTY leader has readable bytes, the system shall read up to the read-buffer size, feed the bytes to the ANSI parser (decoding any DCS/OSC hooks via `decode_hook` and applying them via `apply_hook`) to update the `BlockList`, and include a `SessionEvent::Wakeup` in its return value.
- **R17.** WHEN `resize` is called, the system shall set the PTY window size to the new `cols`/`rows` and update the active block grid dimensions to match.
- **R18.** WHEN the child shell process exits, the system shall set the current `Running` block (if any) to `BlockState::Finished` and include a `SessionEvent::ChildExited` carrying the process exit code in the next `pump` result.
- **R19.** The system shall expose finished and in-progress block output via `Block::output_text` without re-reading the PTY. A block's captured output shall have its TRAILING blank rows trimmed (`trim_trailing_blank_rows` on the block-output path) — the grid snapshot is screen-height, so a short command would otherwise be a full-screen block; trimming keeps a block only as tall as its real output so command blocks STACK as scrollback (M1.H #50). The alt-screen `grid_styled_rows` keeps the FULL grid (a full-screen program owns its whole screen).
- **R20b.** The system shall hold a block's rendered output as styled lines — each a sequence of runs of adjacent grid cells sharing the same foreground color, background color, and cell flags (carrying alacritty's own `Color`/`Flags`, so the model stays gpui-free) — exposed via `Block::output_styled`; `coalesce_row` shall merge each maximal same-style run and trailing-trim the line so `Block::output_text` (the runs flattened) remains byte-identical to the pre-color plain-string model.
- **R20.** WHEN a DCS hook terminator byte is presented to `encoding_for_dcs_terminator`, the system shall return the `DcsEncoding` that terminator indicates (`Hex`, `Plain`, or `AnsiCQuoted`), or `None` if the terminator names no known encoding.
- **R21.** WHEN a second `Precmd` hook is applied while the staged-prompt buffer is still populated (not yet consumed by a `Preexec`), the system shall replace the staged buffer's contents with the newer hook's fields.
- **R22.** WHEN an `InitShell` hook is applied or the session is shut down, the system shall discard the staged-prompt buffer.
- **R23.** IF a DCS hook payload decodes successfully but names no known hook, THEN `decode_hook` shall return `DecodeError::UnknownHook`.
- **R24.** WHEN an `AnsiCQuoted` payload contains an escaped separator (`\;`, or a `\xHH` sequence decoding to `;` or `=`), `decode_hook` shall split the payload into its `name;key=value;…` pieces on UNESCAPED separators before un-escaping each piece, so the escaped byte remains part of its field's decoded value — a `;`-containing command or working directory round-trips exactly, and an embedded `…\;key=value` cannot create a phantom field. `Hex` and `Plain` payloads shall retain the legacy decode-then-split order.
- **R25.** WHEN `encode_key` is given a `KeyInput`, the system shall emit the PTY bytes for that key — a printable char as its UTF-8 (prefixed by `ESC` when `alt`); a `Ctrl+key` as the C0 control byte `(c as u8) & 0x1f`; Enter as `\r`, Backspace as `0x7f`, Tab as `0x09`, Escape as `0x1b`, BackTab (Shift-Tab) as `ESC[Z`, Insert as `ESC[2~`; a function key `F(n)` as its SS3 (F1–4 → `ESC O P/Q/R/S`) or CSI (F5–12 → `ESC[{15,17,18,19,20,21,23,24}~`) sequence (an out-of-range `n` → empty); Delete/PageUp/PageDown as their CSI sequences; and the cursor keys (arrows/Home/End) as `ESC[1;<param><final>` when a modifier is held — `param = modifier_param(shift, alt, ctrl) = 1 + Shift(1) + Alt(2) + Ctrl(4)` — else the plain `ESC[<final>`.
- **R26.** The system shall expose `is_alt_screen` (true while a program holds the alternate screen — DECSET 1049), `is_command_running` (true while a foreground command is running — a `Running` block exists, opened by Preexec and finished by the next Precmd), and `grid_styled_rows` (the live grid as styled rows); and `input_route(alt_screen, ctrl, command_running)` shall return `Raw` while the alternate screen is active OR a foreground command is running OR a control key is held, else `Cooked`. (Routing on `command_running` — not only `alt_screen` — is what lets an INLINE interactive program on the primary screen, e.g. an arrow-key menu or `read`, receive keystrokes instead of Marley's local editor.)
- **R28.** The system shall expose `is_bracketed_paste()` (true while a program holds DECSET 2004) and a pure `paste_bytes(text, bracketed) -> Vec<u8>`: WHEN `bracketed`, the text wrapped in `ESC[200~`…`ESC[201~` with every embedded `ESC[201~` end-marker stripped first (so a pasted marker cannot close the bracket early and inject the tail as commands); WHEN not, the raw UTF-8 bytes. The app pastes the clipboard as `paste_bytes(text, is_bracketed_paste())` — to the running program (R26 routing) or the local prompt buffer (app-shell R42).
- **R30.** The system shall expose `Block::rerun_command(&self) -> Option<String>` — `Some(command)`
  IFF the block's `state == BlockState::Finished` AND its command is non-empty; `None` for a
  still-running/pending block or an empty command (not re-runnable). Mutation targets: the `Finished`
  check (a Running block → None), the `!command.is_empty()` guard (empty → None), the `&&` (a Running
  non-empty block must NOT be re-runnable), the whole-fn return. The system shall also expose
  `BlockList::last_rerunnable(&self) -> Option<String>` — the LAST block (execution order) whose
  `rerun_command()` is `Some` (drives cmd-R); `None` when no finished command exists. (The app resends
  it via `write_command` only when the session is idle — app-shell R50.)
- **R29.** The system shall expose `Block::copy_text(what: BlockCopy) -> String` where `BlockCopy {
  Command, Output }` — `Command` returns the block's command line, `Output` returns `output_text()`
  (the plain `\n`-joined, trailing-trimmed output, so a block-action copy and a drag-copy of the same
  output agree). Mutation targets: the `Command` vs `Output` arm (each returns the right field, not
  swapped or collapsed) + the whole-fn return. (The app reveals copy-command/copy-output affordances
  on the block header — app-shell R49.)
- **R27.** The system shall expose `current_prompt() -> Option<&PromptInfo>` returning the context a `Precmd` staged for the LIVE prompt (the cwd/git the next command will run in) — `Some` after a precmd reports it, `None` before any precmd and again once the following `Preexec` consumes it — so the app-shell can render the prompt's cwd/git segments (app-shell R43).

## Acceptance Criteria

| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | Two blocks in one session never share a `BlockId` (R1) | planned |
| 2 | Blocks iterate in command-execution order by `BlockIndex` (R2) | planned |
| 3 | A `Preexec` hook opens a `Running` block with the command text and the staged `PromptInfo` (R3) | planned |
| 4 | Output between `Preexec` and next `Precmd` lands in the current block (R4) | planned |
| 5 | A `Precmd` hook closes the running block as `Finished` with its exit code (R5) | planned |
| 6 | A `Precmd` hook fills the staged-prompt buffer for the next block (R6) | planned |
| 7 | `InitShell` records the `ShellSessionId -> SessionId` mapping; later blocks carry the session's `SessionId` (R7) | planned |
| 8 | Identical output bytes with no hook produce zero new blocks (R8) | planned |
| 9 | Hex, Plain, and AnsiCQuoted payloads each decode under the supplied encoding (R9) | planned |
| 10 | An undecodable payload yields `DecodeError::UndecodablePayload` and no model mutation (R10) | planned |
| 11 | A `Preexec`/`Precmd` applied with no registered session yields `ApplyHookError::MissingSession`, `BlockList` unchanged (R11) | planned |
| 12 | `spawn` allocates a `SessionId`, returns a handle, and the shell process is running (R12) | planned |
| 13 | A partial PTY write re-queues and eventually writes the whole buffer (R13) | planned |
| 14 | `write_command("ls")` writes `ls\r\n` to the leader (R14) | planned |
| 15 | A write after child disconnect returns `Disconnected` (R15) | planned |
| 16 | `pump` reads available bytes, decodes+applies hooks, updates blocks, and returns `Wakeup` (R16) | planned |
| 17 | `resize` issues the winsize ioctl and resizes the active grid (R17) | planned |
| 18 | Child exit closes the running block and surfaces `ChildExited` with the code (R18) | planned |
| 19 | `output_text` returns the rendered output without a PTY read (R19) | planned |
| 20 | `encoding_for_dcs_terminator` maps each known terminator to its encoding and unknowns to `None` (R20) | planned |
| 21 | A second `Precmd` replaces the still-staged prompt buffer (R21) | planned |
| 22 | `InitShell`/shutdown discards the staged prompt buffer (R22) | planned |
| 23 | A well-formed payload naming no known hook yields `DecodeError::UnknownHook` (R23) | planned |
| 24 | An AnsiCQuoted payload splits on unescaped separators BEFORE un-escaping: `\;` stays in its value, no phantom fields; Hex/Plain semantics unchanged (R24) | planned |

## Visual / Behavioral Acceptance

N/A — this component owns no rendering surface (`browser_testable: no`). Behavior is asserted entirely
through the `TerminalSession` / `BlockList` / `decode_hook` / `apply_hook` API in unit and integration
tests. The downstream UI panel that paints `BlockList` (a later M2 spec) carries the visual/AX harness
acceptance.

## Test Plan

- **Unit:**
  - `coalesce_row_merges_runs_and_trailing_trims` → R20b (a multi-run fixture: adjacent same-style cells merge; a style change splits; trailing all-blank runs drop + the last run trims so the joined text equals `raw.trim_end()`; an all-default-blank line → `[]`)
  - `output_text_stays_plain_after_styling` → R20b/R19 (a styled Block flattens to the same plain string; the existing session/apply/integration suites stay byte-identical)
  - `encode_char_and_control` + `encode_named_keys` → R25 (every key's bytes: char/alt-char/Ctrl-C/D/Z via `&0x1f`; Enter/Backspace/Tab/Escape + the arrow/Home/End/Page/Delete CSI finals)
  - `input_route_cases` → R26 (the 8-case alt_screen×ctrl×command_running truth table); `is_alt_screen_tracks_decset` + `grid_styled_rows_reflects_grid` → R26 (mock feeds `\x1b[?1049h`/`l` + content); `is_command_running_tracks_the_foreground_block` → R26 (mock+DCS: `[init, preexec]`→true, `[init, preexec, precmd]`→false, fresh→false); `ctrl_c_writes_the_interrupt_byte` → R25 (a mock session; `write_bytes(encode_key(Ctrl-C))` records `[0x03]`)
  - `block_ids_unique_within_session` → R1
  - `block_index_orders_by_execution` → R2
  - `preexec_opens_running_block_with_command_and_staged_prompt` → R3
  - `output_appends_to_current_block` → R4
  - `precmd_finishes_block_with_exit_code` → R5
  - `precmd_stages_next_prompt_info` → R6
  - `init_shell_records_shell_to_session_mapping_and_stamps_blocks` → R7
  - `plain_output_creates_no_block_boundary` → R8
  - `decode_hook_decodes_under_supplied_encoding` → R9 (one case per `DcsEncoding`)
  - `undecodable_payload_returns_decode_error_no_mutation` → R10 (`DecodeError::UndecodablePayload`)
  - `hook_without_session_returns_missing_session` → R11 (asserts `apply_hook` returns `ApplyHookError::MissingSession` and `blocks()` is unchanged)
  - `write_command_appends_cr_lf` → R14
  - `output_text_does_not_touch_pty` → R19 (fake-PTY read counter stays at its pre-call value)
  - `encoding_for_dcs_terminator_selects_encoding` → R20 (each known terminator → its encoding; unknown → `None`)
  - `second_precmd_replaces_staged_prompt` → R21
  - `init_shell_and_shutdown_discard_staged_prompt` → R22
  - `unknown_hook_returns_unknown_hook` → R23 (`DecodeError::UnknownHook`)
  - `decode_ansic_escaped_separators_stay_in_value` → R24 (`\;` inside a command and a pwd value; `\x3b`; the `a\\;` escaped-backslash-then-real-separator boundary; `…\;git=evil` yields NO phantom `git` field; a Plain payload with a literal `\;` keeps the legacy split)
  - `pump_leading_wouldblock_is_idle_fast_path_not_fatal` + `pump_midburst_wouldblock_retries_within_budget` → R16 (an idle pump returns immediately on a LEADING `WouldBlock` — no retry sleeps; the ~1 ms retry windows apply only MID-BURST after bytes were read this call. The idle sleeps cost ~12 ms per pane per pump — the TICKET-023 multi-pane frame killer, measured)
  - 100% coverage on this component's touched lines.
- **Integration** (real PTY via `alacritty_terminal::tty`, a stub shell that emits scripted DCS hooks):
  - `spawn_writes_reads_back_block` → R12, R13, R16 (spawn → `write_command("echo hi")` → `pump` until a `Finished` block whose `output_text` contains `hi`).
  - `resize_updates_winsize_and_grid` → R17.
  - `child_exit_closes_block_and_emits_event` → R18 (shell `exit 3` → `ChildExited(ExitCode(Some(3)))`).
  - `write_after_disconnect_errors` → R15.
- **Visual:** none (`browser_testable: no`).
- **Regression:** the full `marley_terminal` suite stays green; `decode_hook` round-trips for all three encodings remain stable; `encoding_for_dcs_terminator` selection stays stable; block-segmentation tests keep passing after any `alacritty_terminal` bump.

## Mutation Targets

`cargo-mutants` must kill every viable mutant on: `encoding_for_dcs_terminator` (terminator→encoding
selection table, the `None` fallthrough), `decode_hook` (per-encoding codec branches, the
`UndecodablePayload` vs `UnknownHook` error split), the R24 raw-field scanners `split_unescaped` /
`find_unescaped` (the backslash-consumes-next escape-unit rule, the separator match, the segment
boundary/final push, the found-index return, the loop advances), `apply_hook` and its hook-dispatch state machine
(`Preexec`/`Precmd`/`InitShell`/`Bootstrapped` transitions, the `Running`→`Finished` edge, exit-code
assignment, the `ShellSessionId -> SessionId` registry insert, the `MissingSession` guard, the
staged-prompt populate/replace/discard/consume lifecycle), `BlockId`/`BlockIndex` allocation, the
partial-write re-queue loop in `write_bytes`, the `\r\n` append in `write_command`, the `pump`
read/decode/apply/event-emit path, and `coalesce_row` (the same-`(fg,bg,flags)` run-merge decision,
the new-run vs append branch, the trailing-trim loop — pop-empty-run vs truncate-last vs break, so
`output_text` stays byte-identical), `encode_key` (every emitted byte per key — the `ctrl_byte`
`& 0x1f`, the ctrl-vs-alt-vs-plain Char branch order, the CSI intro `0x1b 0x5b` + each key's final),
`ctrl_byte` (the `& 0x1f`), `input_route` (the two `||` + the alt_screen/ctrl/command_running operands — the 8-case table kills each), `is_command_running` (the `.is_some()` on `current()` — killed by the running/finished/fresh fixtures), `paste_bytes` (the `bracketed` branch, the `ESC[200~`/`ESC[201~` marker goldens, the `.replace(ESC[201~)` injection-strip — killed by the wrap/raw goldens + an embedded-marker fixture) + `is_bracketed_paste` (the DECSET-2004 mode read — killed by the `ESC[?2004h`/fresh fixtures), `modifier_param` (the `1 +`/`Shift`/`2·Alt`/`4·Ctrl` coefficients — the 8-case table pins every output, powers of two keep modifiers independent), `csi_cursor` (the `param > 1` branch — plain vs parameterized), `encode_fkey` (each F1–12 arm + the `_` empty arm — the table-driven per-F goldens), and
`is_alt_screen` (the `contains(ALT_SCREEN)`). MSI 100% on this surface.

ACCEPTED-UNTESTABLE: the raw `openpty`/fork+exec syscall lines inside `spawn` (reused from
`alacritty_terminal::tty`) and the OS-level `ioctl(TIOCSWINSZ)` call in `resize` have no deterministic
unit harness; they are exercised by the real-PTY integration tests and excluded from mutation via
`// mutants: skip` with this justification.

## Dependencies

- REUSE (permissive): `alacritty_terminal` (Apache-2.0 — PTY spawn via `tty`, cell `Grid`, ANSI
  `Processor`), `vte` (Apache-2.0 / MIT — the escape-sequence parser underneath the processor).
- Marley components: **depends on `marley_core`** for `SessionId` (the process-unique session counter;
  seam-contracts §2). `marley_core` lists `marley_terminal` as a downstream consumer. **No dependency
  on `marley_command`** — PTY spawn is owned here via `alacritty_terminal::tty` (seam-contracts §4.2);
  `marley_command` is the separate non-PTY child-process spawn seam and is not in the PTY path at M1.
  Downstream, a future `marley_panel`/UI spec observes the `BlockList`; the out-of-process control seam
  is a separate later spec.

## Out of scope / deferred

- Rendering/painting blocks in gpui (a UI spec, M2+).
- The out-of-process terminal server and `SCM_RIGHTS` leader-fd passing (deferred; M1 is in-process
  direct spawn only).
- External automation / `local_control`-style RPC drive surface and byte-stream tap (later spec).
- Windows ConPTY backend (M1 is Unix PTY only; Windows deferred).
- Bracketed-paste, kitty-keyboard, and synchronized-output batching nuances (later terminal-model spec).

## Clean-room provenance

Behavior-derived from a fork-reference doc (observable I/O of a PTY-backed, DCS-hook-segmented shell
session) — no AGPL/fork source read, no private module/type/static names reproduced, IP-counsel
sign-off pending (open item in `clean-build-plan.md`). Seam ownership conforms to
`standards/seam-contracts.md` §2 (`SessionId`) and §4 (hook error split + single PTY spawn seam).
Public-surface identifiers are Marley-original (clean-room Posture A): the round-1 terminal-local
`SessionId(u64)` is removed in favor of imported `marley_core::SessionId` + the correlation-only
`ShellSessionId`, and the single `HookError` enum is split into `DecodeError` (stateless) and
`ApplyHookError` (stateful). REUSE crates (`alacritty_terminal`, `vte`) are Apache-2.0 / MIT.
