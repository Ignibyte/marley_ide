---
pipeline_id: 47153da3-f3fa-4f03-9f13-2d40fd87760e
ticket: forge#14 (4701d6d6-24a3-470c-ab77-56190f3bc301) · local docs/planning/tickets/open/TICKET-010-terminal-blocks.md
aar_id: b0e4c12f-407b-4d1d-9c45-62224314c22a
sprint: M1.A — The Usable Terminal (aa46e22f) seq 3/5
status: Phase 5 — Complete PASS
title: marley_terminal — UI-agnostic PTY session + per-command Block model (DCS-hook segmented)
type: feature
milestone: M1
references:
  - ../../../specs/SPEC-terminal-blocks.spec.md
  - ../../../marley_architecture/crate-map.md
---

## Title

TICKET-010 — `marley_terminal` (crate `crates/terminal_blocks`), the M1.A **product core**: the
UI-agnostic PTY-backed shell **session** + per-command **Block model**. Spawn a shell, write commands,
read output back, and segment it into Blocks **from shell DCS/OSC hooks** (cwd/git/exit), producing a
`BlockList` any front-end observes. The render is the front-end's job (`app_shell` #16). Adopts
[`SPEC-terminal-blocks.spec.md`](../../../specs/SPEC-terminal-blocks.spec.md) **R1–R23 verbatim**.
Promotes the proven marley_spike PTY/grid/Block patterns into a real, reusable session.

## Scope

### In (this ticket, M1.A — the full spec minus its listed deferrals)
- **Value types + Block model:** `BlockId`/`BlockIndex`, `ShellSessionId` (correlation-only),
  `BlockState{Pending,Running,Finished}`, `ExitCode`, `PromptInfo`, `Block` (+ `output_text`),
  `BlockList` (`len`/`get`/`current`/`iter`). R1, R2, R8, R19.
- **Stateless DCS codec (PURE):** `DcsEncoding{Hex,Plain,AnsiCQuoted}`, `DcsHook{InitShell,Precmd,
  Preexec,Bootstrapped}`, `encoding_for_dcs_terminator(u8) -> Option<DcsEncoding>` (R20),
  `decode_hook(encoding, payload) -> Result<DcsHook, DecodeError{UndecodablePayload,UnknownHook}>`
  (R9/R10/R23). No registry/BlockList access; codec errors only.
- **Stateful hook application (PURE, no IO):** `apply_hook(DcsHook) -> Result<(), ApplyHookError{
  MissingSession}>` — drives the BlockList: Preexec opens a Running block from the staged prompt (R3),
  output appends to the current block (R4), Precmd finishes it with the exit code + stages the next
  prompt (R5/R6), InitShell records the `ShellSessionId->SessionId` mapping + stamps blocks (R7),
  MissingSession guard leaves the BlockList unchanged (R11), staged-prompt replace/discard/consume
  lifecycle (R21/R22).
- **Session (the PTY seam):** `TerminalSession::spawn(SessionOptions) -> Result<Self, SessionError>`
  (allocates `marley_core::SessionId::next()` + `alacritty_terminal::tty` PTY, R12), `write_bytes`
  (partial-write re-queue, R13), `write_command` (`\r\n` append, R14), `resize` (winsize + grid, R17),
  `pump` (read→decode→apply→`SessionEvent::Wakeup`, R16), child-exit→`ChildExited`(code) + close block
  (R18), `Disconnected` on write-after-disconnect (R15), `session_id`/`blocks`/`shutdown`.
- §21 — CHANGELOG + `docs/marley_architecture/terminal_blocks.md`.

### Out / deferred (per the spec's own Out-of-scope)
- gpui rendering/painting of blocks (the front-end's job — `app_shell` #16 / an M2 UI panel).
- The out-of-process terminal server + `SCM_RIGHTS` fd passing (M1 = in-process direct spawn).
- `local_control`-style external RPC drive surface + byte-stream tap (the brain-observation seam rides
  on `blocks()` + the BlockList for now; the RPC tap is a later spec — AD-claude-brain-agent-session-
  supervision-001's `session.read` lands when `local_control` is built, M3).
- Windows ConPTY (M1 Unix PTY only); bracketed-paste/kitty-keyboard/sync-output nuances (later).
- R14b: no Block boundary from output heuristics (R8 — DCS-only segmentation is enforced, not deferred).

## Acceptance Criteria (EARS — adopt SPEC-terminal-blocks R1–R23)
The authoritative clauses are R1–R23; this pipeline adopts them all. The load-bearing subset:
- **AC-codec (R9/R10/R20/R23)** — `encoding_for_dcs_terminator` maps each known terminator → its
  encoding, unknown → `None`; `decode_hook` decodes Hex/Plain/AnsiCQuoted; undecodable →
  `UndecodablePayload`, well-formed-but-unknown → `UnknownHook`; stateless (no model mutation).
- **AC-applyhook (R3–R7/R11/R21/R22)** — the hook state machine drives the BlockList exactly: Preexec
  opens Running w/ staged prompt; Precmd finishes + stages next + exit code; InitShell maps+stamps;
  MissingSession leaves BlockList unchanged; staged buffer replace/discard.
- **AC-blocklist (R1/R2/R8/R19)** — unique BlockId per session; BlockIndex execution order; no Block
  without a hook; `output_text` without a PTY re-read.
- **AC-session (R12–R18, headless integration)** — spawn /bin/sh, `write_command("echo hi")`, `pump`
  until a Finished block whose `output_text` contains "hi"; `exit 3` → `ChildExited(ExitCode(Some(3)))`;
  resize updates winsize; write-after-disconnect → `Disconnected`; partial-write re-queues.
- **AC-gate** — cov 100 / MSI 100 on the PURE surface (codec + apply_hook + BlockList + write re-queue
  logic); the raw `tty::new`/read/write/`ioctl` shim ACCEPTED-UNTESTABLE (mutants::skip + rust_cov
  exclude); FULL `scripts/gates.sh` → `GATE GREEN [diff]`. **gate-15 N/A** (visual_acceptance N/A).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
1. **UI-agnostic** — no gpui dep, no Render, no visual_acceptance. The crate produces a `BlockList`;
   `app_shell` (#16) renders it from `&ThemeColors`.
2. **Reuse marley_spike's proven patterns** — spawn_pty (HOLD the Pty, BF-pty-drop), drive_reader (~1ms
   WouldBlock backoff, BF-nonblocking-busyloop), term_to_rows, build_block/exit_code→status (assert the
   code, the 004 trap). Promote them into the real `TerminalSession`/`pump`/Block.
3. **The PURE/shim seam** — the codec, apply_hook, BlockList, and the `write_bytes` re-queue LOGIC are
   PURE (testable with a MockReader/MockWriter + synthesized hooks, NO PTY) → cov 100/MSI 100. Only the
   raw `tty::new` spawn + the leader-fd read/write + the `TIOCSWINSZ` ioctl are ACCEPTED-UNTESTABLE
   (the spec's own list) — exercised by the headless real-PTY integration test.
4. **SessionId from marley_core** (R12, seam-contracts §2) — import `SessionId::next()`; do NOT redefine.
   `ShellSessionId` is correlation-only (no allocation).
5. Deps: `alacritty_terminal="0.26"`, `marley_core`(path); dev `serial_test`+`tempfile`+`mutants`. NO
   gpui, NO marley_command (PTY owned here, seam-contracts §4.2). alacritty/vte allowlisted.
6. **The shell DCS bootstrap** (the script that makes a REAL shell emit precmd/preexec DCS) — design
   decides: the integration test uses a STUB shell emitting scripted DCS (per the spec's test plan);
   whether a real bash/zsh bootstrap script ships is a design call (likely a tiny shipped script or
   deferred to app_shell/M2). The Rust surface (decode/apply) is the same regardless.

## Phase Plan
- **P2 Design** — read the spike sources (term_io/drive/grid/block) + SPEC-terminal-blocks; the module
  manifest (codec / model / session split); the PURE/shim seam precisely (write_bytes re-queue via a
  MockWriter; pump's read via the drive_reader pattern; apply_hook's state machine pure); the stub-shell
  + bootstrap decision; the regression-test plan (the spec's 16 unit + 4 integration) + the mutation map
  (apply_hook state machine + the codec branches + the staged-prompt lifecycle — non-origin/multi-row
  fixtures). DELEGATE the spike-read + manifest to a subagent if it conserves context.
- **P3 Implement** — codec + model + session; the shim mutants::skip + rust_cov exclude.
- **P3.5 Inspect** — critics: the apply_hook state machine (every transition + the MissingSession/staged
  lifecycle), the codec error split, the write re-queue loop, clean-room (Marley-original names).
- **P4 Validate** — the unit suite (R1–R11, R14, R19–R23) + the headless real-PTY integration (R12/R13/
  R15/R16/R17/R18) with a stub DCS shell; FULL gate (gate-15 N/A).
- **P5 Complete** — §21; close #14; archive; → #15 editor.
