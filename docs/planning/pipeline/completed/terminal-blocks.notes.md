---
pipeline_id: 47153da3-f3fa-4f03-9f13-2d40fd87760e
aar_id: b0e4c12f-407b-4d1d-9c45-62224314c22a
---

# marley_terminal (terminal_blocks) — pipeline notes

## Phase 1 — Plan (2026-06-29)

**Intent:** the M1.A product core — the UI-agnostic PTY session + per-command Block model
(DCS-hook-segmented), producing a `BlockList` app_shell (#16) renders. Sprint M1.A seq 3/5. Adopts
SPEC-terminal-blocks R1–R23 verbatim (the full spec; its own out-of-scope list is the deferral).

**The cut call:** the spec is cohesive — R8 mandates DCS-hook-only segmentation (no heuristics), so the
codec + apply_hook state machine is the core, not cuttable. The spec is already M1-scoped (its
out-of-scope defers the gpui render, the OOP server, Windows, local_control RPC). So #14 = the full
spec. Big (23 reqs) but mostly PURE + heavily testable.

**Why this is testable (not as heavy as it looks):** the value types + BlockList + the STATELESS codec
(decode_hook/encoding_for_dcs_terminator) + the STATEFUL apply_hook state machine + the write_bytes
re-queue loop are ALL pure (no IO) → unit-testable without a PTY. Only the raw `tty::new` spawn + the
leader-fd read/write + the `TIOCSWINSZ` ioctl are the shim (the spec's own ACCEPTED-UNTESTABLE list),
proven by ONE headless real-PTY integration test with a stub DCS shell. SessionId::next() confirmed in
marley_core.

## Carry to Design (Phase 2)
1. **READ** crates/marley_spike/src/{term_io,drive,grid,block}.rs (the proven spawn_pty[HOLD Pty]/
   drive_reader[~1ms backoff]/term_to_rows/build_block/exit_code_to_status) + SPEC-terminal-blocks
   (the full surface). DELEGATE the spike-read + module-manifest proposal to a subagent if it conserves
   context.
2. **Module manifest** (the codec/model/session split):
   - `dcs.rs` (PURE) — DcsEncoding/DcsHook/DecodeError + `encoding_for_dcs_terminator` (R20) +
     `decode_hook` (R9/R10/R23, the 3 encodings: Hex/Plain/AnsiCQuoted). Stateless.
   - `block.rs` (PURE) — BlockId/BlockIndex/ShellSessionId/BlockState/ExitCode/PromptInfo/Block
     (+ output_text via the held grid) + BlockList (len/get/current/iter, R1/R2/R19).
   - `apply.rs` (PURE) — the staged-prompt buffer + the ShellSessionId→SessionId registry + apply_hook
     (R3–R7/R11/R21/R22 — the state machine; takes a DcsHook, mutates in-memory model, NO IO).
   - `session.rs` (SHIM + the testable LOGIC) — TerminalSession: spawn (tty::new, mutants::skip), the
     leader-fd read/write (mutants::skip raw OS calls), resize ioctl (mutants::skip), BUT the write_bytes
     re-queue LOOP + pump's read→decode→apply orchestration are PURE-ish (test write_bytes with a
     MockWriter; test pump's decode/apply via apply.rs; the raw fd read/write is the thin skip'd seam).
   - `lib.rs` — re-exports + crate docs.
3. **The seam decision** — keep the raw OS calls (tty::new, the fd read/write, ioctl) in tiny skip'd
   fns; the LOGIC (re-queue, decode+apply, child-exit→event) is pure + tested. rust_cov exclude only the
   raw-OS-call fns/file regions (like marley_spike's term_io/app exclude).
4. **The stub DCS shell** for the integration test — a small `/bin/sh -c` script (or a fixture) that
   emits the DCS hook sequences (Precmd/Preexec) so a REAL PTY round-trip produces Blocks. Decide its
   shape (the spec's "stub shell that emits scripted DCS hooks"). Whether a real bash/zsh bootstrap
   ships is a design call (likely defer to app_shell/M2; the integration test uses the stub).
5. **Mutation map** — the BIG surface: apply_hook's every transition (Preexec→open, Precmd→finish+stage,
   InitShell→map+stamp, MissingSession guard, the Running→Finished edge, exit-code assign, staged
   replace/discard/consume); decode_hook's per-encoding branches + the UndecodablePayload vs UnknownHook
   split; encoding_for_dcs_terminator's table + None; BlockId/BlockIndex allocation; the write re-queue
   loop. Use NON-ORIGIN/multi-row fixtures (PR-claude-nonorigin-interior-fixture) + assert carried codes
   (the 004 trap). This is where the rigor concentrates.
6. **Scale via subagents** — this is the biggest crate; design + implement + the ~20 tests will likely
   span a compaction. Lean on subagents for the codec, apply_hook, and the test suite; the pipeline docs
   + forge carry state across any compaction.

**Phase 1 status:** PASS (autonomous-through-commit per chad's /goal). → Phase 2 Design.

## Phase 2 — Design (2026-06-29)

### Key API facts (alacritty 0.26, confirmed)
- `tty::new(&Options, WindowSize, u64) -> Result<Pty>`; `Pty::file()->&File` (leader, O_NONBLOCK);
  `Pty::next_child_event()->Option<ChildEvent>`, `ChildEvent::Exited(Option<ExitStatus>)`→`.code()` —
  the race-free SIGCHLD child-exit + **exit code** for R18 (the only public reap path; `child()` is
  immutable). **`Pty::on_resize` ABORTS the process on failure** → use `rustix::termios::tcsetwinsize`
  (safe `io::Result`, allowlisted) so `SessionError::Resize` is reachable + the crate stays **unsafe-free**
  (miri/SAST clean). `Term`/`Processor` headless-synthesizable (spike-proven).

### Deps (UI-agnostic — NO gpui, NO direct vte)
`alacritty_terminal="0.26"` + `rustix={version="0.38",features=["termios"]}` + `marley_core`(path,
SessionId). dev: `serial_test`+`tempfile`+`mutants`. `#![deny(missing_docs)]`.

### Module manifest (crates/terminal_blocks/src/)
- **dcs.rs (PURE 100/100)** — `DcsEncoding{Hex,Plain,AnsiCQuoted}`, `DcsHook{InitShell,Precmd(PrecmdValue),
  Preexec(PreexecValue),Bootstrapped}`, `DecodeError{UndecodablePayload,UnknownHook}`,
  `encoding_for_dcs_terminator(u8)->Option<DcsEncoding>` (R20, the terminator→enc table), `decode_hook`
  (R9/R10/R23 — codec[hex/plain/cquoted]→utf8→parse→dispatch-by-name; **split: codec/utf8 fail→
  UndecodablePayload, unknown-name→UnknownHook, missing-required-field→UndecodablePayload**), +
  `pub(crate) DcsScanner` (the incremental DCS byte extractor for pump, pure).
- **block.rs (PURE 100/100)** — BlockId/BlockIndex/ShellSessionId + BlockState/ExitCode/PromptInfo +
  Block (**output = owned `Vec<String>`, NOT a live grid** → `output_text`=pure join, R19) + BlockList
  (len/get/current[=last IFF Running]/iter + `pub(crate) open_running`[the SOLE id/index alloc, R1/R2] +
  current_mut).
- **apply.rs (PURE 100/100 — the rigor center)** — `SessionModel{ session_id, registry:HashMap<
  ShellSessionId,SessionId>, staged_prompt:Option<PromptInfo>, blocks, bootstrapped:Option<bool> }` +
  `apply_hook(DcsHook)->Result<(),ApplyHookError>` + set_current_output(R4) + finish_current_if_running
  (R18). **Transition table:** InitShell→registry.insert + staged=None(R7,R22); Preexec→`registry.is_empty()
  →Err(MissingSession)`(R11) else staged.take().unwrap_or_default()→open_running(R3); Precmd→`is_empty→
  MissingSession`(R11) else finish current + set exact exit_code(R5,004 trap) + staged=Some(prompt)(R6/R21);
  Bootstrapped→bootstrapped=Some(is_subshell). R22-shutdown half = Drop (NO redundant discard_staged —
  would be an equivalent mutant).
- **pty_os.rs (THE ONLY SHIM — per-fn `mutants::skip` + the SOLE rust_cov exclude)** — `OsPtyChannel{pty,
  io}` + `spawn(&SessionOptions)->io::Result` (tty::new, HOLDS the Pty per BF-pty-drop) + impl PtyChannel:
  read/write (raw leader-fd) / set_winsize (rustix tcsetwinsize) / poll_child_exit (next_child_event→ExitCode).
- **session.rs (TESTABLE LOGIC over the trait)** — `pub(crate) trait PtyChannel{read,write,set_winsize,
  poll_child_exit}`; `classify_write(&io::Error)->WriteOutcome{Retry,Disconnected,Fatal}` (R15);
  `TerminalSession<P:PtyChannel=OsPtyChannel>{channel,model,scanner,term,processor,cols,rows}` — spawn
  (R12 glue, integration-covered), write_bytes (R13 re-queue loop over channel.write + 1ms backoff),
  write_command (R14 `\r\n`), pump (R16/R18 — bounded read[drive_reader pattern]→scanner.feed→render
  passthrough via term_to_rows→set_current_output; per hook: reset render engine on Preexec, decode_hook
  +apply_hook; Wakeup; then poll_child_exit→finish+ChildExited), resize (R17 set_winsize+term.resize),
  apply_hook delegate, blocks, shutdown(Drop). **`#[cfg(test)] MockPtyChannel`** scripts read/write/winsize/
  child-exit + RECORDS writes (R13 concat) + COUNTS reads (R19).

### THE SEAM WIN
Because write_bytes/pump/resize are **generic over PtyChannel** + render uses a headless Term, **R13
(partial-write re-queue), R15 (disconnect classify), R16 (pump decode/apply/render), R18 (child-exit)
are UNIT-tested with mocks — not only integration.** The ONLY ACCEPTED-UNTESTABLE = pty_os.rs's 4 raw
fns. rust_cov exclude = append `|terminal_blocks/src/pty_os\.rs` to scripts/gates.sh rust_cov (one-line).
`#[cfg_attr(test,mutants::skip)]` also on the `Dims::total_lines` method (spike-confirmed equivalent).

### DCS wire format (clean-room INVENTED, behavior-derived — risk F1, in-bounds per spec_source: behavior-only)
`ESC P <selector> <encoded-payload> ESC\` (7-bit DCS+ST). selector `h`/`p`/`q` = Hex/Plain/AnsiCQuoted
(R20). Decoded payload = `name;key=value;…` (name = init/preexec/precmd/bootstrapped). Naive `;`/`=`
split (no escaping) — M1 limitation (AnsiCQuoted carries `;`/control when a real M2 bootstrap ships).

### Test plan (17 unit + 4 integration[#serial] + 6 mutation-completion unit) + stub shell
The spec's 17 unit (R1-R11,R14,R19-R23) + 4 real-PTY integration (R12/R13/R15/R16/R17/R18 via a STUB
shell: `printf '\033Pp…\033\\'` emitting the DCS sequences) + 6 MOCK unit tests the Mutation Targets
demand (partial_write_requeues[NON-ORIGIN multi-write], classify_write, pump_decodes_applies_renders,
pump_child_exit[004 trap], resize_maps_error, dcs_scanner_split) — because integration can't
deterministically force partial-writes/child-exit. Mutation map: apply_hook every transition, decode_hook
encoding branches + the Undecodable/Unknown split, the scanner, open_running id/index (3-block non-origin
fixture), write re-queue (recorded-concat == full input), exit_code (exact code), write_command (`b"ls\r\n"`).

### Risks
F1 wire-format invented (correct, behavior-only). F2 next_child_event sans Poller (integration-validated;
fallback: Ok(0)/EIO→reap). F3 equivalent mutants (Bootstrapped→white-box `model.bootstrapped` test;
shutdown-discard via Drop, omit redundant call). F4 alacritty surface (confirmed, spike-matched). F5 the
6 extra mutation-completion tests are floor-required, not scope creep.

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement (2026-06-29)

Built `crates/terminal_blocks` (package `marley_terminal`): Cargo.toml + src/{lib,dcs,block,apply,
session,pty_os}.rs. check + clippy(-D warnings) + fmt + rustdoc(-D warnings) all clean; **no unsafe**;
`mutants::skip` ONLY on pty_os.rs fns + `Dims::total_lines` (session.rs, the spike-confirmed equivalent)
+ lib.rs doc prose. scripts/gates.sh rust_cov exclude extended with `terminal_blocks/src/pty_os\.rs`.

**Deviations (all forced/justified — subagent-confirmed against installed sources):**
1. **`TerminalSession` is `Box<dyn PtyChannel>` (non-generic), NOT `<P: PtyChannel = OsPtyChannel>`** —
   the generic-with-default forces PtyChannel + OsPtyChannel public (`private_bounds`/`private_interfaces`
   under -D warnings), conflicting with the `pub(crate) trait` + private `mod pty_os`. `Box<dyn>` keeps
   both seam types private, matches the spec's non-generic `pub struct TerminalSession`, and preserves
   the mock seam (inject via `pub(crate) with_channel(Box<dyn PtyChannel>, SessionId, cols, rows)`).
   Object-safe (all methods `&mut self`).
2. `pty_os::spawn` returns `Result<OsPtyChannel, SessionError>` (maps the failure INSIDE the excluded
   file) so `session::spawn` carries no untestable error region.
3. `SessionModel` is `pub` with `pub fn bootstrapped(&self) -> Option<bool>` — the white-box read for the
   F3 Bootstrapped mutant (+ kills the write-only-field dead-code).
4. `ApplyHookError` defined in apply.rs, re-exported `pub use crate::apply::ApplyHookError` from session.
5. `pump` uses a fixed `PUMP_RETRY_BUDGET=8` (1ms backoff) since it's polled repeatedly; a fatal read
   checks `poll_child_exit` first (macOS surfaces child-exit as Ok(0) EOF — race-tolerant).

**API facts (new-vs-spike):** `rustix::termios::tcsetwinsize<Fd:AsFd>(fd, Winsize)` by-value, `Winsize{
ws_row,ws_col,ws_xpixel,ws_ypixel:u16}`, `?` maps Errno→io::Error; `Pty::next_child_event(&mut self)`
on the `EventedPty` trait → `ChildEvent::Exited(Option<ExitStatus>)`→`.code()`; `Term::resize<S:
Dimensions>(by value)`. Spike alacritty calls matched exactly.

**Heads-up for VALIDATE:** dev-deps `serial_test`+`tempfile` are unused until the tests land → **machete
(gate:9) flags them until validate writes the tests** (the #[serial] integration + a tempfile cwd use
them). Mock seam: `TerminalSession::with_channel(Box::new(MockPtyChannel), SessionId::from(n), cols,
rows)`; the `Disconnected` path needs a mock scripting a fatal read + `poll_child_exit → None`.

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5) — 2 critics (codec+state-machine; session+seam)

**Verdict:** correctness mostly CLEAN (decode_hook total/no-panic on malformed bytes — verified; the
apply_hook transitions, BlockList alloc, classify_write, the seam all PASS). 3 fixes applied at source +
re-verified (check/clippy/fmt/rustdoc/no-unsafe clean):

| # | Sev | Finding | Fix |
|---|---|---|---|
| 1 | **HIGH** | **ingest ordering / blank-Block race** — pump rendered the read's passthrough BEFORE applying its hooks; a coalesced `[Preexec]out[Precmd]` read drops `out` (rendered while no block open) → blank Finished block. Breaks R4 + R16-flaky. | `DcsScanner::feed` now emits an ORDERED `Vec<DcsEvent{Passthrough(bytes)|Hook(raw)}>`; `ingest` processes in order (Preexec resets+opens BEFORE the following passthrough renders into it). Forge `BF-…-coalesced-pty-read-…-001` + `PR-claude-ordered-events-coalesced-stream-hooks-001`. |
| 2 | **HIGH** | **2 equivalent mutants** — `(hi<<4) | lo` (hi,lo ∈0..=15, disjoint nibbles) → `|`==`^`==`+`, so `|→^` is unkillable → MSI<100 (hex_decode + c_unescape). | rewrote `|` → `+` (identical for disjoint nibbles; `+→-`/`+→*` mutants ARE killable). Forge `BF-…-disjoint-nibble-…-001` + `PR-claude-use-plus-not-or-for-disjoint-bitfield-assembly-001`. |
| 3 | LOW | `write_bytes` no `Ok(0)`/budget → a degenerate mock can infinite-loop (prod-unreachable, test foot-gun). | `Ok(0)→Disconnected` + a retry budget (pump symmetry); successful partial write refreshes the budget. |

**Verified GREEN:** the SEAM works (`pub(crate) with_channel(Box<dyn PtyChannel>, SessionId, cols, rows)`
→ unit-test write_bytes/pump/classify_write/resize with a MockPtyChannel; rust_cov excludes pty_os.rs
ONLY → session.rs in the 100% denominator; spawn glue integration-covered, no uncovered error line).
clean-room clean (grep warp empty; names Marley-original; DCS wire format invented). deny ok (rustix
licenses allowlisted, 0 advisories). machete green NOW (it doesn't flag unused dev-deps — the Phase-3
heads-up was wrong; serial_test/tempfile harmless). §14: no unwrap/expect/panic on reachable paths.

**Deferred/noted (LOW):** pump drops events on fatal-read+no-child-exit (edge — the front-end could miss
the last pre-disconnect frame; defer); `SessionModel`/`bootstrapped()` pub beyond the spec contract (the
F3 white-box affordance — kept, justified).

### CARRY TO VALIDATE — must-dos + MUTATION KILL MAP (the critics' fixtures)
- **gate:4 trap:** `Dims::total_lines` (session.rs) is `mutants::skip` but session.rs is NOT
  coverage-excluded → its line is coverage-dead unless exercised. ADD a unit test asserting
  `dims.total_lines()` (mirror the spike's `assert_eq!(dims.total_lines(), 24)`) OR confirm Term::resize
  calls it — else gate:4 fails closed.
- **Mock guards:** unit tests live IN-CRATE (`#[cfg(test)]` in src/ — `tests/` can't see `pub(crate)`
  with_channel; real-PTY tests go in tests/). The mock must NEVER script write `Ok(0)` (now→Disconnected)
  nor read `n > buf.len()` (slice panic).
- **Tests = the spec's 17 unit + 4 integration[#serial, stub DCS shell] + 6 mutation-completion mock
  unit** (partial_write_requeues[NON-ORIGIN multi-write, assert recorded-concat==full input],
  classify_write both-sides, pump_decodes_applies_renders[a COALESCED read: init+preexec+`hi`+precmd in
  ONE chunk → Finished block output_text contains "hi" — this kills the ordering regression],
  pump_child_exit[exact Some(3)], resize_maps_error, dcs_scanner_split + the ordered-event interleave).
- **KILL-MAP FIXTURE TRAPS (the critics):** exact encodings (h/p/q + an unknown→None); the
  Undecodable-vs-Unknown split (`b"\xff"`→Undecodable vs `b"nope"`→Unknown); UPPERCASE hex (kills the
  `A..=F` arm) + nonzero high nibble (kills `<<→>>`); `\x` NOT at byte-index 2 (else `i+2`==`i*2`
  equivalent); 3-block lists asserting EXACT ids `[0,1,2]` + indices `[0,1,2]` (kills `+=→*=` on the
  allocators); embedded-ESC + ESC-abort scanner fixtures; **exact `ExitCode(Some(3))`** (004 trap); the
  Bootstrapped 3-state white-box read (fresh→None, true→Some(true), false→Some(false)); R11 Preexec/Precmd
  on empty registry → MissingSession + `blocks()` UNCHANGED; `unwrap_or_default()` None-branch (Preexec
  with nothing staged); `session_id()` line-cov call (its Default mutant is unviable — SessionId no
  Default). All `Default::default()` return mutants on the enums/Block are UNVIABLE (no Default) EXCEPT
  `blocks() -> Box::leak(Default BlockList)` which IS viable (BlockList: Default) — kill by reading
  blocks() after a mutation.

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate (2026-06-29)

**Tests (69):** in-crate `#[cfg(test)]` — dcs (23), block (6), apply (12), session (23, via a
`MockPtyChannel` injected through `with_channel`) + `tests/integration.rs` (5 real-PTY `#[serial]` with
a stub `/bin/sh` DCS shell). All the inspect kill-map fixtures used verbatim (uppercase/nonzero-nibble
hex, `\x` off-index-2, 3-block exact ids/indices `[0,1,2]`, embedded-ESC scanner, exact `ExitCode(Some(3))`,
the Bootstrapped 3-state white-box, R11 blocks-unchanged, the `unwrap_or_default` None-branch,
`Dims::total_lines` exercised). **The coalesced-read pump test** (init+preexec+`hi`+precmd in ONE read →
Finished block, output_text contains "hi") is the ordering-regression guard; the real-shell DCS
round-trip proved reliable (not flaky).

**Results:** `cargo nextest -p marley_terminal` = **69 passed**; `cargo llvm-cov` = **100%** on
dcs/block/apply/session (pty_os.rs excluded); `cargo mutants` = **139 mutants → 0 missed** (112 caught +
8 timeout[the budget/loop infinite-loop kills] + 19 unviable[the `Default::default()` returns — no Default
on the enums/Block]). The `|→+` former-equivalents (dcs.rs:174/202) are now CAUGHT; the viable
`blocks() -> Box::leak(Default BlockList)` mutants killed by read-after-mutation. fmt + clippy clean. No
production bug found (the design + the 3 inspect fixes held).

**FULL gate (010):** `GATE GREEN [diff]` (23:44:03, re-run) — **15 passed, 0 failed**. Coverage **100%**
(dcs/block/apply/session; pty_os.rs excluded); mutation **120 caught / 0 missed → MSI 100.0%**; miri +
deny + machete + visual(N/A) all green. Commit receipt written (41 b, MATCH). The FIRST run was GATE RED
on gate:13 ONLY — a Cargo.toml doc comment said "`unsafe`-free" (the crate IS genuinely unsafe-free),
tripping the literal-"unsafe" SAST grep (the harness's known trap); reworded to drop the word (no `.rs`/
coverage/mutation change) + re-ran green.

**Phase 4 status:** PASS.
