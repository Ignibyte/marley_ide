//! The PTY shell session: spawn, write, read-back, resize, and the stateful hook application.
//!
//! The fallible OS calls live behind the small `PtyChannel` trait, so the testable logic — the
//! [`TerminalSession::write_bytes`] re-queue loop (R13), the [`TerminalSession::pump`]
//! read/decode/apply/render orchestration (R16/R18), and [`TerminalSession::resize`] (R17) — runs
//! over any channel and is unit-tested with a mock. The default channel is the raw-PTY shim in
//! `pty_os`; only that shim is ACCEPTED-UNTESTABLE.

use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::SessionId;
use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::vte::ansi::Processor;

use crate::apply::SessionModel;
use crate::block::{BlockList, ExitCode, PromptInfo};
use marley_dcs::{DcsEvent, DcsScanner};

use crate::dcs::{DcsHook, decode_frame};
use crate::styled::{StyledLine, coalesce_row, trim_trailing_blank_rows};

pub use crate::apply::ApplyHookError;

// The shim is a child of `session`, the one module that calls it; `#[path]` keeps the file where
// the coverage gate's exclusion and the docs name it.
#[cfg(unix)]
#[path = "pty_os.rs"]
mod pty_os;

/// The read buffer size for one `pump` read.
const READ_BUF: usize = 4096;
/// How many consecutive `WouldBlock`/`Interrupted` reads `pump` tolerates (each with a ~1 ms
/// yield) before returning so a freshly-forked child has a window to write without `pump` blocking
/// the host event loop.
const PUMP_RETRY_BUDGET: u32 = 8;
/// `EIO` raw OS error — a disconnected PTY leader on both Linux and macOS.
const EIO: i32 = 5;

/// How long `pump` waits (measured ACROSS pump calls — never slept inside one; the frame-tick
/// rule) for the child-exit event after the master read returns `EIO`. On Linux the slave fds die
/// WITH the child, so the read-side `EIO` routinely arrives BEFORE the SIGCHLD byte lands in the
/// edge-triggered self-pipe (#423; alacritty's event loop handles the same race by looping back
/// for the inevitable `Exited` — `event_loop.rs`, Apache-2.0, adopted). Within the grace the pump
/// reports quietly-no-events; a master death with NO child exit still surfaces
/// [`SessionError::Disconnected`] once the grace expires.
const EIO_EXIT_GRACE: Duration = Duration::from_secs(1);

/// #423: has the EIO exit-grace been exhausted? PURE — the boundary (`elapsed == GRACE` counts
/// as EXPIRED) is pinned by a unit at exactly the deadline, which a live `Instant` comparison
/// could never hold still for (the `>=`-vs-`>` mutant dies here, deterministically).
fn eio_grace_expired(elapsed: Duration) -> bool {
    elapsed >= EIO_EXIT_GRACE
}

/// The stage of the bounded child-reap escalation (TICKET-348): the shim sends `SIGHUP` and enters
/// [`ReapStage::Hup`], then `SIGKILL` and [`ReapStage::Kill`] if the child outlives the first deadline.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ReapStage {
    Hup,
    Kill,
}

/// What the reap executor should do next, decided purely by [`reap_step`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ReapAction {
    /// Keep polling — the child has not exited and the current stage's deadline has not passed.
    Wait,
    /// The `SIGHUP` deadline passed with the child still alive — escalate to `SIGKILL`.
    SendKill,
    /// The `SIGKILL` deadline passed with the child STILL alive (pathological) — stop waiting, leak
    /// the child deliberately (D5 zombie-over-hang: a zombie is recoverable, a hang is not).
    GiveUp,
    /// The child has exited (reaped by the non-blocking poll) — teardown is complete.
    Done,
}

/// How long (ms) the reap waits at EACH stage before escalating: `SIGHUP`→`SIGKILL`→give-up.
pub(crate) const REAP_DEADLINE_MS: u64 = 2000;

/// The pure escalation decision for the bounded PTY teardown (TICKET-348). The executor shim has
/// ALREADY sent `SIGHUP` and entered [`ReapStage::Hup`] before the first call; `waited_ms` is the
/// elapsed time in the CURRENT stage (the clock is injected — never read here — so this stays a pure,
/// exhaustively-testable step function); `exited` is a fresh non-blocking child-exit poll.
///
/// This replaces the unbounded `waitpid` that `alacritty_terminal`'s `Pty::Drop` blocks on: by the
/// time that `Drop` runs, the child is already dead + reaped (so its `child.wait()` returns at once),
/// or the give-up path has leaked the `Pty` so that `Drop` never runs.
pub(crate) const fn reap_step(stage: ReapStage, waited_ms: u64, exited: bool) -> ReapAction {
    if exited {
        return ReapAction::Done;
    }
    match stage {
        ReapStage::Hup => {
            if waited_ms >= REAP_DEADLINE_MS {
                ReapAction::SendKill
            } else {
                ReapAction::Wait
            }
        }
        ReapStage::Kill => {
            if waited_ms >= REAP_DEADLINE_MS {
                ReapAction::GiveUp
            } else {
                ReapAction::Wait
            }
        }
    }
}

/// The OS seam `TerminalSession` drives. Implemented for real by the raw-PTY shim and, in tests,
/// by a scripted mock injected through the private channel field. `Send` is a supertrait so a
/// whole [`TerminalSession`] can move to a reaper thread. Dropping a session reaps the child, and
/// that reap is now BOUNDED (TICKET-348: the shim's `Drop` escalates `SIGHUP`→`SIGKILL`→give-up on
/// [`reap_step`] deadlines instead of blocking on an unbounded `waitpid`), so the app can eat a
/// tab-close on its main thread without risking a hang (this softens the TICKET-023 concern).
pub(crate) trait PtyChannel: Send {
    /// Read up to `buf.len()` bytes from the PTY leader (non-blocking).
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    /// Write up to `buf.len()` bytes to the PTY leader, returning the count actually written.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
    /// Set the PTY window size to `cols`×`rows`.
    fn set_winsize(&mut self, cols: u16, rows: u16) -> io::Result<()>;
    /// Poll for a child-exit event, returning its [`ExitCode`] once the child has exited.
    fn poll_child_exit(&mut self) -> Option<ExitCode>;
}

/// What to do about a failed PTY write (R15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WriteOutcome {
    /// Transient — re-queue the remaining bytes after a short yield.
    Retry,
    /// The child has gone away — surface [`SessionError::Disconnected`].
    Disconnected,
    /// A non-recoverable write failure — surface [`SessionError::Write`].
    Fatal,
}

/// Classify a PTY write error (R15): `WouldBlock`/`Interrupted` retry; `BrokenPipe` and raw `EIO`
/// mean the child disconnected; everything else is fatal.
pub(crate) fn classify_write(error: &io::Error) -> WriteOutcome {
    match error.kind() {
        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted => WriteOutcome::Retry,
        io::ErrorKind::BrokenPipe => WriteOutcome::Disconnected,
        _ => {
            if error.raw_os_error() == Some(EIO) {
                WriteOutcome::Disconnected
            } else {
                WriteOutcome::Fatal
            }
        }
    }
}

/// How to spawn a shell session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionOptions {
    /// The shell program to run.
    pub shell: PathBuf,
    /// Arguments for the program (empty for a plain login shell; e.g. the `ssh` target for a remote pane).
    pub args: Vec<String>,
    /// The working directory to start in.
    pub cwd: PathBuf,
    /// Extra environment variables for the child.
    pub env: Vec<(String, String)>,
    /// Initial terminal column count.
    pub cols: u16,
    /// Initial terminal row count.
    pub rows: u16,
}

/// A failure from a [`TerminalSession`] operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionError {
    /// The PTY/shell could not be spawned (R12).
    Spawn,
    /// A non-recoverable write to the PTY leader failed (R13).
    Write,
    /// The child disconnected during a read or write (R15).
    Disconnected,
    /// The resize ioctl failed (R17).
    Resize,
}

/// Something observed during a [`TerminalSession::pump`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEvent {
    /// New output was read and applied — the front-end should refresh (R16).
    Wakeup,
    /// The child process exited with this code (R18).
    ChildExited(ExitCode),
}

/// A PTY-backed shell session producing a per-command [`BlockList`].
pub struct TerminalSession {
    channel: Box<dyn PtyChannel>,
    model: SessionModel,
    scanner: DcsScanner,
    term: Term<VoidListener>,
    processor: Processor,
    cols: u16,
    rows: u16,
    /// Latched TRUE when [`pump`](Self::pump) surfaces [`SessionEvent::ChildExited`]. Writes
    /// consult it FIRST (#423): Linux masters ACCEPT post-close writes (the bytes queue into the
    /// flip buffer with no reader), so errno alone cannot carry the post-exit disconnect
    /// contract — state does, uniformly on every platform.
    child_exited: bool,
    /// When the master read first returned `EIO` with no child-exit event yet observed — the
    /// [`EIO_EXIT_GRACE`] clock (#423). Cleared by a successful read or the exit event.
    eio_since: Option<Instant>,
}

// The PTY channel, the parser and the render grid have no useful `Debug`; the session's id is
// what tells two sessions apart.
impl std::fmt::Debug for TerminalSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TerminalSession")
            .field("session_id", &self.model.session_id())
            .finish_non_exhaustive()
    }
}

impl TerminalSession {
    /// Spawn a shell on a fresh PTY (R12): allocate the session's [`SessionId`], start the shell
    /// from `options`, and return the handle. The raw spawn (and its failure mapping) lives in the
    /// `pty_os` shim, which exists only for Unix; elsewhere a spawn fails cleanly while the block
    /// model and the mock-driven session logic stay available.
    ///
    /// One function with the platform split inside it, not two `cfg`-gated functions: a mutation
    /// tool mutates both copies, and the one compiled out on this platform could never be killed.
    ///
    /// # Errors
    ///
    /// [`SessionError::Spawn`] when the PTY cannot be opened or the shell cannot be started, and
    /// always on a platform without the `pty_os` shim.
    pub fn spawn(options: &SessionOptions) -> Result<Self, SessionError> {
        #[cfg(unix)]
        {
            let cols = options.cols;
            let rows = options.rows;
            let channel = pty_os::spawn(options)?;
            Ok(Self::with_channel(
                Box::new(channel),
                SessionId::next(),
                cols,
                rows,
            ))
        }
        #[cfg(not(unix))]
        {
            let _ = options;
            Err(SessionError::Spawn)
        }
    }

    /// Assemble a session around an already-built channel and id. The shared constructor for
    /// [`spawn`](Self::spawn) and the mock-driven seam tests.
    pub(crate) fn with_channel(
        channel: Box<dyn PtyChannel>,
        session_id: SessionId,
        cols: u16,
        rows: u16,
    ) -> Self {
        Self {
            channel,
            model: SessionModel::new(session_id),
            scanner: DcsScanner::default(),
            term: build_term(cols, rows),
            processor: Processor::new(),
            cols,
            rows,
            child_exited: false,
            eio_since: None,
        }
    }

    /// Rewind the [`EIO_EXIT_GRACE`] clock past its deadline (#423 tests): lets a unit prove the
    /// grace EXPIRES (→ `Disconnected`) without sleeping through the real ceiling.
    #[cfg(test)]
    pub(crate) fn expire_eio_grace_for_test(&mut self) {
        self.eio_since = Some(
            Instant::now()
                .checked_sub(EIO_EXIT_GRACE + Duration::from_millis(1))
                .expect("the monotonic clock has run longer than the EIO grace"),
        );
    }

    /// The session's process-unique [`SessionId`].
    #[must_use]
    pub const fn session_id(&self) -> SessionId {
        self.model.session_id()
    }

    /// Whether a full-screen program has switched to the alternate screen (R26) — the app renders
    /// the live grid + streams raw keystrokes while this holds.
    #[must_use]
    pub fn is_alt_screen(&self) -> bool {
        self.term.mode().contains(TermMode::ALT_SCREEN)
    }

    /// Whether a foreground command is running (R26) — a `Running` block exists, opened by the shell's
    /// Preexec and finished by the next Precmd. While it holds, the running child owns the terminal,
    /// so the app streams every keystroke to the PTY instead of feeding its local line editor.
    #[must_use]
    pub fn is_command_running(&self) -> bool {
        self.blocks().current().is_some()
    }

    /// Whether a program has enabled bracketed paste (DECSET 2004) (R26) — the app wraps a clipboard
    /// paste in the `ESC[200~`…`ESC[201~` markers while this holds, so a multi-line paste is literal
    /// data. Mirrors [`is_alt_screen`](Self::is_alt_screen).
    #[must_use]
    pub fn is_bracketed_paste(&self) -> bool {
        self.term.mode().contains(TermMode::BRACKETED_PASTE)
    }

    /// Whether a program has enabled application cursor-key mode (DECSET 1 / DECCKM) (R26 / #286) —
    /// the app encodes arrows and the alt-scroll wheel fallback as SS3 (`ESC O A`) instead of the
    /// legacy CSI (`ESC [ A`) while this holds. Mirrors [`is_alt_screen`](Self::is_alt_screen).
    #[must_use]
    pub fn is_app_cursor(&self) -> bool {
        self.term.mode().contains(TermMode::APP_CURSOR)
    }

    /// The mouse-tracking mode snapshot (M17 #280) — the DECSET flags alacritty parses,
    /// packaged gpui-free for the app's grid handlers + the pure [`crate::mouse_report`]
    /// encoder. One mode read; mirrors the [`is_alt_screen`](Self::is_alt_screen) idiom.
    #[must_use]
    pub fn mouse_modes(&self) -> crate::mouse::MouseModes {
        let mode = self.term.mode();
        crate::mouse::MouseModes {
            click: mode.contains(TermMode::MOUSE_REPORT_CLICK),
            drag: mode.contains(TermMode::MOUSE_DRAG),
            motion: mode.contains(TermMode::MOUSE_MOTION),
            sgr: mode.contains(TermMode::SGR_MOUSE),
            alt_scroll: mode.contains(TermMode::ALTERNATE_SCROLL),
            alt_screen: mode.contains(TermMode::ALT_SCREEN),
            app_cursor: mode.contains(TermMode::APP_CURSOR),
        }
    }

    /// The live grid as styled rows (R26) — what the app paints in alt-screen mode (vim/top/less),
    /// reusing the same coalesced-run model as the Block output.
    #[must_use]
    pub fn grid_styled_rows(&self) -> Vec<StyledLine> {
        term_to_styled_rows(&self.term)
    }

    /// Write every byte of `bytes` to the PTY leader, re-queueing the unwritten remainder of a
    /// partial write until the whole buffer lands (R13). A successful partial write refreshes the
    /// retry budget, so only a channel making no progress can exhaust it (unreachable on the real
    /// PTY, but it keeps a degenerate channel from looping forever).
    ///
    /// # Errors
    ///
    /// A disconnect — a `BrokenPipe`/`EIO` error or a zero-length write (EOF symmetry with
    /// `pump`) — yields [`SessionError::Disconnected`] (R15); any other failure, or a stall that
    /// exhausts the `PUMP_RETRY_BUDGET` consecutive retries, yields [`SessionError::Write`].
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), SessionError> {
        // #423: once the child's exit has been OBSERVED (pump surfaced ChildExited), writes fail
        // by STATE — Linux masters accept post-close writes (flip-buffer, no reader), so the
        // errno classification below can never carry this half of the contract portably.
        if self.child_exited {
            return Err(SessionError::Disconnected);
        }
        let mut remaining = bytes;
        let mut budget = PUMP_RETRY_BUDGET;
        while !remaining.is_empty() {
            match self.channel.write(remaining) {
                Ok(0) => return Err(SessionError::Disconnected),
                Ok(written) => {
                    remaining = &remaining[written..];
                    budget = PUMP_RETRY_BUDGET;
                }
                Err(error) => match classify_write(&error) {
                    WriteOutcome::Retry => {
                        if budget == 0 {
                            return Err(SessionError::Write);
                        }
                        budget -= 1;
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    WriteOutcome::Disconnected => return Err(SessionError::Disconnected),
                    WriteOutcome::Fatal => return Err(SessionError::Write),
                },
            }
        }
        Ok(())
    }

    /// Write `command` followed by carriage-return then line-feed (`\r\n`) to the PTY leader (R14).
    ///
    /// # Errors
    ///
    /// As [`write_bytes`](Self::write_bytes).
    pub fn write_command(&mut self, command: &str) -> Result<(), SessionError> {
        let mut bytes = command.as_bytes().to_vec();
        bytes.push(b'\r');
        bytes.push(b'\n');
        self.write_bytes(&bytes)
    }

    /// Resize the PTY window and the render grid to `cols`×`rows` (R17).
    ///
    /// # Errors
    ///
    /// A failed winsize ioctl yields [`SessionError::Resize`].
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), SessionError> {
        self.channel
            .set_winsize(cols, rows)
            .map_err(|_| SessionError::Resize)?;
        self.cols = cols;
        self.rows = rows;
        self.term.resize(Dims {
            lines: rows as usize,
            cols: cols as usize,
        });
        Ok(())
    }

    /// Read available output, decode + apply any DCS hooks, render the passthrough into the current
    /// block, and report what happened (R16/R18).
    ///
    /// Returns a [`SessionEvent::Wakeup`] when bytes were read, and a
    /// [`SessionEvent::ChildExited`] once the child has exited (finishing the running block first).
    ///
    /// # Errors
    ///
    /// A fatal read with no accompanying child-exit yields [`SessionError::Disconnected`] —
    /// immediately for a non-`EIO` failure; for `EIO` only after the `EIO_EXIT_GRACE` deadline
    /// with no exit event (#423: on Linux the master's `EIO` routinely beats the SIGCHLD byte,
    /// so within the grace the pump reports quietly and the NEXT call picks the exit up).
    pub fn pump(&mut self) -> Result<Vec<SessionEvent>, SessionError> {
        let mut events = Vec::new();
        let mut got_bytes = false;
        let mut read_failed = false;
        let mut read_eio = false;
        let mut budget = PUMP_RETRY_BUDGET;
        let mut buf = [0u8; READ_BUF];
        loop {
            match self.channel.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    got_bytes = true;
                    self.ingest(&buf[..n]);
                }
                Err(error) => match error.kind() {
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted => {
                        // Idle fast-path: nothing read THIS call → return now instead of
                        // sleeping through the retry budget. The budget exists so a MID-BURST
                        // child gets ~1 ms windows to keep streaming; on an idle PTY the sleeps
                        // cost ~12 ms per pane per pump — the multi-pane frame killer
                        // (TICKET-023 inspect, measured at 98 ms/tick for 8 idle panes).
                        if !got_bytes || budget == 0 {
                            break;
                        }
                        budget -= 1;
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    _ => {
                        // #423: EIO is the Linux master's "slave side died" — routinely ahead of
                        // the SIGCHLD self-pipe byte, so it gets the exit-grace below instead of
                        // an instant Disconnected (alacritty event_loop precedent, adopted).
                        read_eio = error.raw_os_error() == Some(EIO);
                        read_failed = true;
                        break;
                    }
                },
            }
        }
        if got_bytes {
            events.push(SessionEvent::Wakeup);
            self.eio_since = None; // a live read ends any pending EIO-exit grace
        }
        match self.channel.poll_child_exit() {
            Some(code) => {
                // #52: full-capture before an ABNORMAL finish (the shell process exited mid-command) too,
                // so a block finished this way keeps its scrollback — consistent with the Precmd path.
                self.model
                    .set_current_output(trim_trailing_blank_rows(full_term_to_styled_rows(
                        &self.term,
                    )));
                self.model.finish_current_if_running();
                self.child_exited = true; // #423: the write-path latch — state, not errno
                self.eio_since = None;
                events.push(SessionEvent::ChildExited(code));
            }
            None => {
                if read_failed {
                    // #423: within the grace an EIO-failed read is "the child just died and the
                    // SIGCHLD byte hasn't landed" — report quietly and let the NEXT pump surface
                    // ChildExited (order-independence, never slept in-pump). Grace expiry — or a
                    // non-EIO read failure — is the honest no-exit disconnect, as before. Once
                    // the exit HAS been observed (`child_exited`) there is nothing to wait for:
                    // a later EIO is an instant Disconnected, the pre-#423 dead-pane cadence.
                    if read_eio && !self.child_exited {
                        let since = *self.eio_since.get_or_insert_with(Instant::now);
                        if !eio_grace_expired(since.elapsed()) {
                            return Ok(events);
                        }
                    }
                    return Err(SessionError::Disconnected);
                }
            }
        }
        Ok(events)
    }

    /// Apply one decoded [`DcsHook`] directly to the model (R3–R7, R11, R21, R22).
    ///
    /// # Errors
    ///
    /// [`ApplyHookError::MissingSession`] when a `Preexec` or `Precmd` arrives before any
    /// `InitShell` registered the session; the blocks are left unchanged.
    pub fn apply_hook(&mut self, hook: DcsHook) -> Result<(), ApplyHookError> {
        self.model.apply_hook(hook)
    }

    /// #433: the monotone block-lifecycle epoch — bumps once per block BORN
    /// (Preexec) and once per block FINISHED (Precmd or child exit). The
    /// problems producer's change gate: compare, never count.
    #[must_use]
    pub const fn block_epoch(&self) -> u64 {
        self.model.block_epoch()
    }

    /// #435: stage the run-block identity that the `Preexec` reporting
    /// exactly `command` binds to the block it births — the `staged_prompt`
    /// correlation shape, hardened to MATCH-BIND (a tag can only ever label
    /// the command it was minted for). The app calls this in the same sync
    /// region as a SUCCESSFUL `write_command`; the tag is OPAQUE to this
    /// crate (the app owns the encoding). A mismatching or later command
    /// binds nothing; a second stage before consumption is refused
    /// (fail-safe: lost identity beats wrong identity).
    pub fn stage_run_tag(&mut self, tag: String, command: String) {
        self.model.stage_run_tag(tag, command);
    }

    /// The session's per-command blocks.
    #[must_use]
    pub const fn blocks(&self) -> &BlockList {
        self.model.blocks()
    }

    /// #433 (cross-crate TEST SCAFFOLDING — hidden): see
    /// [`SessionModel::seed_finished_block_for_test`].
    ///
    /// # Errors
    ///
    /// Whatever the model's seed refuses (see there).
    #[doc(hidden)]
    pub fn seed_finished_block_for_test(
        &mut self,
        command: &str,
        pwd: Option<&str>,
        exit: Option<i32>,
        output: &str,
    ) -> Result<(), ApplyHookError> {
        self.model
            .seed_finished_block_for_test(command, pwd, exit, output)
    }

    /// The prompt context staged for the LIVE prompt (the cwd/git the next command will run in), or
    /// `None` when no precmd has reported it yet. Renders the input-row context segments (R43).
    #[must_use]
    pub const fn current_prompt(&self) -> Option<&PromptInfo> {
        self.model.current_prompt()
    }

    /// Shut the session down, dropping the PTY and discarding the staged-prompt buffer (R22). The
    /// PTY's `Drop` runs the BOUNDED reap (TICKET-348 — `SIGHUP`→`SIGKILL`→give-up on the
    /// `reap_step` deadlines), so this returns within those bounds even against a child that
    /// ignores `SIGHUP`.
    pub fn shutdown(self) {
        drop(self);
    }

    /// Feed one read chunk through the scanner and process its [`DcsEvent`] stream IN ORDER: render
    /// each passthrough run into the current block, and decode + apply each completed DCS hook where
    /// it falls in the byte stream (resetting the render grid on `Preexec` before the following
    /// output is rendered, dropping `MissingSession`).
    ///
    /// Processing in order is what keeps a coalesced `[Preexec]output[Precmd]` read correct: the
    /// `Preexec` opens the block before the command's output is rendered into it, instead of the
    /// output landing while no block is open (R4) and the block finishing blank (R16).
    fn ingest(&mut self, chunk: &[u8]) {
        for ev in self.scanner.feed(chunk) {
            match ev {
                DcsEvent::Passthrough(passthrough) => {
                    self.processor.advance(&mut self.term, &passthrough);
                    // Trim the grid's trailing blank rows so a block is only as tall as its real
                    // output — command blocks then STACK as scrollback (R19). The alt-screen
                    // `grid_styled_rows` keeps the full untrimmed grid.
                    self.model
                        .set_current_output(trim_trailing_blank_rows(term_to_styled_rows(
                            &self.term,
                        )));
                }
                DcsEvent::Hook(frame) => {
                    let Ok(hook) = decode_frame(&frame) else {
                        continue;
                    };
                    if matches!(hook, DcsHook::Preexec(_)) {
                        self.reset_term();
                    }
                    if matches!(hook, DcsHook::Precmd(_)) {
                        // #52: on command FINISH, capture the FULL output (the grid's scrollback history +
                        // the visible screen) into the running block BEFORE `apply_hook` closes it — so a
                        // command whose output exceeded the screen keeps ALL of it, not just the last
                        // screen. One bounded full read at finish (the per-Passthrough snapshot above stays
                        // the live, screen-bounded view while the command runs).
                        self.model.set_current_output(trim_trailing_blank_rows(
                            full_term_to_styled_rows(&self.term),
                        ));
                    }
                    if let Err(error) = self.model.apply_hook(hook) {
                        // R11: a hook before the shell's `InitShell` leaves the blocks unchanged.
                        log::debug!("marley_terminal: dropped a shell hook: {error:?}");
                    }
                }
            }
        }
    }

    /// Reset the render grid for a new command's output (on `Preexec`).
    fn reset_term(&mut self) {
        self.term = build_term(self.cols, self.rows);
    }
}

/// Minimal [`Dimensions`] for `Term::new`/`Term::resize`: `WindowSize` does not implement
/// `Dimensions`, so an external crate must supply its own. Only `screen_lines`/`columns` are
/// consulted at runtime (scrollback comes from `Config`).
struct Dims {
    lines: usize,
    cols: usize,
}

impl Dimensions for Dims {
    // EQUIVALENT mutant (spike-confirmed): `Term` never calls `total_lines()` (scrollback comes
    // from `Config`), so its return is indistinguishable at runtime — unkillable. `screen_lines`
    // and `columns` ARE consulted and killed by the grid-dimension assertions.
    fn total_lines(&self) -> usize {
        self.lines
    }
    fn screen_lines(&self) -> usize {
        self.lines
    }
    fn columns(&self) -> usize {
        self.cols
    }
}

/// Build a fresh headless `Term` of `cols`×`rows`.
fn build_term(cols: u16, rows: u16) -> Term<VoidListener> {
    let dims = Dims {
        lines: rows as usize,
        cols: cols as usize,
    };
    Term::new(Config::default(), &dims, VoidListener)
}

/// Convert the visible grid to one trimmed `String` per screen line (R19's rendered view).
fn term_to_styled_rows(term: &Term<VoidListener>) -> Vec<StyledLine> {
    let grid = term.grid();
    let cols = grid.columns();
    let mut rows = Vec::with_capacity(grid.screen_lines());
    for line in 0..i32::try_from(grid.screen_lines()).unwrap_or(i32::MAX) {
        let row = &grid[Line(line)];
        let cells = (0..cols).map(|col| {
            let cell = &row[Column(col)];
            (
                cell.c,
                cell.fg,
                cell.bg,
                cell.flags,
                cell.hyperlink().map(|h| h.uri().to_string()),
            )
        });
        rows.push(coalesce_row(cells));
    }
    rows
}

/// The FULL command output (#52): the grid's HISTORY region (lines that scrolled off the top of the
/// visible screen) PLUS the visible screen. alacritty keeps the scrolled-off lines (Config
/// `scrolling_history: 10000`); we read them via the grid's negative `Line` indices. `history =
/// total_lines - screen_lines`. Used at command-finish so a long command's block keeps everything, not
/// just the last screen; the live per-chunk snapshot uses [`term_to_styled_rows`] (screen-bounded).
fn full_term_to_styled_rows(term: &Term<VoidListener>) -> Vec<StyledLine> {
    let grid = term.grid();
    let cols = grid.columns();
    let screen = i32::try_from(grid.screen_lines()).unwrap_or(i32::MAX);
    // Scrolled-off lines (saturating; underflow-proof).
    let history = i32::try_from(grid.history_size()).unwrap_or(i32::MAX);
    let mut rows = Vec::with_capacity(grid.history_size() + grid.screen_lines());
    for line in -history..screen {
        let row = &grid[Line(line)];
        let cells = (0..cols).map(|col| {
            let cell = &row[Column(col)];
            (
                cell.c,
                cell.fg,
                cell.bg,
                cell.flags,
                cell.hyperlink().map(|h| h.uri().to_string()),
            )
        });
        rows.push(coalesce_row(cells));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{Block, BlockState, ShellSessionId};
    use crate::dcs::PreexecValue;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    /// A scripted [`PtyChannel`] that replays read/write/winsize/child-exit outcomes from queues,
    /// RECORDS every written slice (concatenated), and COUNTS every `read` call. It NEVER scripts a
    /// `write` `Ok(0)` (that path is the EOF-disconnect, asserted separately) and never copies more
    /// than `buf.len()` bytes into a read buffer.
    struct MockPtyChannel {
        reads: VecDeque<io::Result<Vec<u8>>>,
        writes: VecDeque<io::Result<usize>>,
        winsize: VecDeque<io::Result<()>>,
        child_exit: VecDeque<Option<ExitCode>>,
        recorded_writes: Arc<Mutex<Vec<u8>>>,
        read_count: Arc<AtomicUsize>,
        /// When the read script is exhausted, return `WouldBlock` forever instead of EOF.
        read_block_forever: bool,
        /// When the write script is exhausted, return `WouldBlock` forever instead of writing all.
        write_block_forever: bool,
    }

    impl MockPtyChannel {
        fn new() -> Self {
            Self {
                reads: VecDeque::new(),
                writes: VecDeque::new(),
                winsize: VecDeque::new(),
                child_exit: VecDeque::new(),
                recorded_writes: Arc::new(Mutex::new(Vec::new())),
                read_count: Arc::new(AtomicUsize::new(0)),
                read_block_forever: false,
                write_block_forever: false,
            }
        }
        fn push_read(&mut self, bytes: &[u8]) {
            self.reads.push_back(Ok(bytes.to_vec()));
        }
        fn push_read_err(&mut self, kind: io::ErrorKind) {
            self.reads.push_back(Err(io::Error::from(kind)));
        }
        /// #423: a raw-errno read error — `io::ErrorKind` alone can't carry `EIO`
        /// (`from(kind)` leaves `raw_os_error()` `None`, so the pump's EIO arm never fires).
        fn push_read_raw_err(&mut self, code: i32) {
            self.reads
                .push_back(Err(io::Error::from_raw_os_error(code)));
        }
    }

    impl PtyChannel for MockPtyChannel {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let _previous = self.read_count.fetch_add(1, Ordering::Relaxed);
            match self.reads.pop_front() {
                Some(Ok(bytes)) => {
                    assert!(bytes.len() <= buf.len(), "mock read must fit the buffer");
                    buf[..bytes.len()].copy_from_slice(&bytes);
                    Ok(bytes.len())
                }
                Some(Err(e)) => Err(e),
                None if self.read_block_forever => Err(io::Error::from(io::ErrorKind::WouldBlock)),
                None => Ok(0),
            }
        }
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            match self.writes.pop_front() {
                Some(Ok(n)) => {
                    // A scripted `Ok(0)` is the deliberate EOF-disconnect probe; only the explicit
                    // queue produces it (the exhaustion default below writes the whole slice), so
                    // the mock never *accidentally* reports a zero-length write.
                    assert!(n <= buf.len(), "mock write count must not exceed the slice");
                    self.recorded_writes
                        .lock()
                        .unwrap()
                        .extend_from_slice(&buf[..n]);
                    Ok(n)
                }
                Some(Err(e)) => Err(e),
                None if self.write_block_forever => Err(io::Error::from(io::ErrorKind::WouldBlock)),
                None => {
                    self.recorded_writes.lock().unwrap().extend_from_slice(buf);
                    Ok(buf.len())
                }
            }
        }
        fn set_winsize(&mut self, _cols: u16, _rows: u16) -> io::Result<()> {
            self.winsize.pop_front().unwrap_or(Ok(()))
        }
        fn poll_child_exit(&mut self) -> Option<ExitCode> {
            self.child_exit.pop_front().flatten()
        }
    }

    /// Build a `TerminalSession` (80×24, `SessionId::from(1)`) around `mock`.
    fn session(mock: MockPtyChannel) -> TerminalSession {
        TerminalSession::with_channel(Box::new(mock), SessionId::from(1), 80, 24)
    }

    #[test]
    fn debug_names_the_session_by_its_id() {
        let s = session(MockPtyChannel::new());
        let id = s.session_id();
        assert_eq!(
            format!("{s:?}"),
            format!("TerminalSession {{ session_id: {id:?}, .. }}")
        );
    }

    /// The block at execution position `n` (the `BlockIndex` field is private to `block`).
    fn block_at(s: &TerminalSession, n: usize) -> &Block {
        s.blocks().iter().nth(n).expect("block at position exists")
    }

    /// A Plain (`p`) DCS frame: `ESC P p <payload> ESC \`.
    fn dcs_plain(payload: &str) -> Vec<u8> {
        let mut v = vec![0x1b, b'P', b'p'];
        v.extend_from_slice(payload.as_bytes());
        v.extend_from_slice(&[0x1b, b'\\']);
        v
    }

    // ── classify_write (R15) — both sides of every arm ──────────────────────
    #[test]
    fn classify_write_table() {
        assert_eq!(
            classify_write(&io::Error::from(io::ErrorKind::WouldBlock)),
            WriteOutcome::Retry
        );
        assert_eq!(
            classify_write(&io::Error::from(io::ErrorKind::Interrupted)),
            WriteOutcome::Retry
        );
        assert_eq!(
            classify_write(&io::Error::from(io::ErrorKind::BrokenPipe)),
            WriteOutcome::Disconnected
        );
        // Raw EIO (5) → Disconnected; a non-EIO, non-pipe error → Fatal. Together these kill the
        // `raw_os_error() == Some(EIO)`→`!=` mutant (which would flip both verdicts).
        assert_eq!(
            classify_write(&io::Error::from_raw_os_error(EIO)),
            WriteOutcome::Disconnected
        );
        assert_eq!(
            classify_write(&io::Error::from(io::ErrorKind::PermissionDenied)),
            WriteOutcome::Fatal
        );
    }

    // ── write_bytes (R13) — NON-ORIGIN partial-write re-queue ───────────────
    #[test]
    fn write_bytes_requeues_partial_writes() {
        // Three partial writes [3,2,5] over a 10-byte buffer; the RECORDED concatenation of the
        // slices actually handed to the channel must equal the whole input (kills the re-queue
        // slicing, the `while !remaining.is_empty()` guard, and write_bytes -> Ok(())).
        let mut mock = MockPtyChannel::new();
        mock.writes.extend([Ok(3), Ok(2), Ok(5)]);
        let recorded = Arc::clone(&mock.recorded_writes);
        let mut s = session(mock);
        assert_eq!(s.write_bytes(b"ABCDEFGHIJ"), Ok(()));
        assert_eq!(&*recorded.lock().unwrap(), b"ABCDEFGHIJ");
    }

    #[test]
    fn write_bytes_retry_then_success() {
        // A WouldBlock then a full write: the budget is NOT exhausted, so the byte lands and
        // write_bytes returns Ok. Kills the `budget == 0`→`!=` mutant (which would bail Err on the
        // very first retry).
        let mut mock = MockPtyChannel::new();
        mock.writes
            .extend([Err(io::Error::from(io::ErrorKind::WouldBlock)), Ok(2)]);
        let recorded = Arc::clone(&mock.recorded_writes);
        let mut s = session(mock);
        assert_eq!(s.write_bytes(b"AB"), Ok(()));
        assert_eq!(&*recorded.lock().unwrap(), b"AB");
    }

    #[test]
    fn write_bytes_budget_exhausts_to_write_error() {
        // A channel that makes NO progress (always WouldBlock) drains the retry budget and fails
        // with Write. The real `budget -= 1` terminates; a `+=`/`/=` mutant never reaches zero and
        // loops forever → cargo-mutants timeout.
        let mut mock = MockPtyChannel::new();
        mock.write_block_forever = true;
        let mut s = session(mock);
        assert_eq!(s.write_bytes(b"X"), Err(SessionError::Write));
    }

    #[test]
    fn write_bytes_zero_write_disconnects() {
        // A zero-length write is the EOF/disconnect symmetry with pump → Disconnected (line 175).
        let mut mock = MockPtyChannel::new();
        mock.writes.push_back(Ok(0));
        let mut s = session(mock);
        assert_eq!(s.write_bytes(b"X"), Err(SessionError::Disconnected));
    }

    #[test]
    fn write_bytes_broken_pipe_disconnects() {
        // A BrokenPipe write error classifies as Disconnected (R15).
        let mut mock = MockPtyChannel::new();
        mock.writes
            .push_back(Err(io::Error::from(io::ErrorKind::BrokenPipe)));
        let mut s = session(mock);
        assert_eq!(s.write_bytes(b"X"), Err(SessionError::Disconnected));
    }

    #[test]
    fn write_bytes_fatal_error_is_write() {
        // A non-recoverable, non-disconnect write error → Write.
        let mut mock = MockPtyChannel::new();
        mock.writes
            .push_back(Err(io::Error::from(io::ErrorKind::PermissionDenied)));
        let mut s = session(mock);
        assert_eq!(s.write_bytes(b"X"), Err(SessionError::Write));
    }

    #[test]
    fn write_command_appends_crlf() {
        // write_command("ls") writes exactly b"ls\r\n" (R14).
        let mock = MockPtyChannel::new();
        let recorded = Arc::clone(&mock.recorded_writes);
        let mut s = session(mock);
        assert_eq!(s.write_command("ls"), Ok(()));
        assert_eq!(&*recorded.lock().unwrap(), b"ls\r\n");
    }

    // ── #433 — the session-level seed delegate mints a real block (kills the
    // delegate-body mutant crate-locally; the app drives cover it cross-crate) ──
    #[test]
    fn seed_delegate_mints_block_and_epoch_on_the_session() {
        let mock = MockPtyChannel::new();
        let mut s = session(mock);
        s.seed_finished_block_for_test("make", Some("/w"), Some(2), "src/a.rs:2: err")
            .unwrap();
        assert_eq!(s.block_epoch(), 2, "born + finished through the delegate");
        let b = s.blocks().iter().next().expect("the seeded block");
        assert_eq!(b.command, "make");
        assert_eq!(b.prompt.pwd.as_deref(), Some("/w"));
        assert_eq!(b.output_text(), "src/a.rs:2: err");
    }

    // ── #433 — the epoch through the PUMP byte path (the D4 both-paths proof) ──
    // #435 (the L-433 lesson — per-crate mutation scope): the SESSION-level
    // stage delegate must be pinned crate-locally or `stage_run_tag -> ()`
    // survives. Rides the real apply machine end-to-end.
    #[test]
    fn stage_run_tag_delegate_binds_through_the_session() {
        let mut s = session(MockPtyChannel::new());
        s.stage_run_tag("main".to_string(), "cargo run".to_string());
        s.seed_finished_block_for_test("cargo run", None, Some(1), "err")
            .unwrap();
        assert_eq!(
            s.blocks().iter().next().and_then(|b| b.run_tag.as_deref()),
            Some("main")
        );
    }

    #[test]
    fn pump_dcs_bytes_bump_the_block_epoch() {
        let chunk = [
            dcs_plain("init;id=9"),
            dcs_plain("preexec;command=make"),
            b"boom".to_vec(),
            dcs_plain("precmd;exit=2"),
        ]
        .concat();
        let mut mock = MockPtyChannel::new();
        mock.push_read(&chunk);
        mock.push_read_err(io::ErrorKind::WouldBlock);
        let mut s = session(mock);
        assert_eq!(s.block_epoch(), 0);
        assert_eq!(s.pump(), Ok(vec![SessionEvent::Wakeup]));
        assert_eq!(
            s.block_epoch(),
            2,
            "born + finished through the raw DCS byte path — the same bumps apply_hook takes"
        );
    }

    // ── pump (R16/R18) — the COALESCED-read ordering guard ──────────────────
    #[test]
    fn pump_coalesced_read_orders_hooks_and_output() {
        // ONE read chunk carrying init + preexec + "hi" + precmd, then WouldBlock. The ordered
        // DcsEvent stream means "hi" renders into the block the preexec opened → a Finished block
        // whose output contains "hi" with exit code Some(0). This is the ordering-regression guard.
        let chunk = [
            dcs_plain("init;id=1"),
            dcs_plain("preexec;command=run"),
            b"hi".to_vec(),
            dcs_plain("precmd;exit=0"),
        ]
        .concat();
        let mut mock = MockPtyChannel::new();
        mock.push_read(&chunk);
        mock.push_read_err(io::ErrorKind::WouldBlock);
        let mut s = session(mock);
        assert_eq!(s.pump(), Ok(vec![SessionEvent::Wakeup]));
        let b = block_at(&s, 0);
        assert_eq!(b.state, BlockState::Finished);
        assert!(b.output_text().contains("hi"), "got {:?}", b.output_text());
        assert_eq!(b.exit_code, ExitCode(Some(0)));
    }

    #[test]
    fn pump_two_commands_reset_render_grid_between_blocks() {
        // Two commands in one coalesced read: the second block's output must NOT contain the
        // first's. Kills reset_term -> () (without the per-Preexec grid reset, "second" would
        // render on top of the stale "first").
        let chunk = [
            dcs_plain("init;id=1"),
            dcs_plain("preexec;command=one"),
            b"first".to_vec(),
            dcs_plain("precmd;exit=0"),
            dcs_plain("preexec;command=two"),
            b"second".to_vec(),
            dcs_plain("precmd;exit=0"),
        ]
        .concat();
        let mut mock = MockPtyChannel::new();
        mock.push_read(&chunk);
        let mut s = session(mock);
        let _events = s.pump().unwrap();
        assert_eq!(s.blocks().len(), 2);
        assert_eq!(block_at(&s, 0).output_text(), "first");
        assert_eq!(block_at(&s, 1).output_text(), "second");
    }

    #[test]
    fn pump_renders_multi_row_output() {
        // A two-row render exercises term_to_rows + the grid dimensions: with a collapsed grid the
        // join would lose a row. Kills term_to_rows -> vec![]/vec![""]/vec!["xyzzy"] and the
        // build_term / Dims screen_lines·columns mutants via the rendered text.
        let chunk = [
            dcs_plain("init;id=1"),
            dcs_plain("preexec;command=run"),
            b"abc\r\ndef".to_vec(),
        ]
        .concat();
        let mut mock = MockPtyChannel::new();
        mock.push_read(&chunk);
        let mut s = session(mock);
        let _events = s.pump().unwrap();
        assert_eq!(s.blocks().current().unwrap().output_text(), "abc\ndef");
    }

    #[test]
    fn ingest_keeps_output_beyond_the_grid() {
        // #52: a command whose output EXCEEDS the 24-row screen keeps ALL rows in its FINISHED block —
        // the lines that scrolled off the visible grid into alacritty's history, not just the last screen.
        // Kills the `full_term_to_styled_rows` `-history..screen` range mutants (a screen-only read at
        // Precmd would drop L01..L16); a plain `term_to_styled_rows` snapshot could not keep L01.
        let output: String = (1..=40)
            .map(|i| format!("L{i:02}"))
            .collect::<Vec<_>>()
            .join("\r\n");
        let chunk = [
            dcs_plain("init;id=1"),
            dcs_plain("preexec;command=seq"),
            output.into_bytes(),
            dcs_plain("precmd;exit=0"),
        ]
        .concat();
        let mut mock = MockPtyChannel::new();
        mock.push_read(&chunk);
        let mut s = session(mock);
        let _events = s.pump().unwrap();
        let text = block_at(&s, 0).output_text();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(
            lines.len(),
            40,
            "all 40 rows kept (incl. the ~16 scrolled off)"
        );
        assert_eq!(lines.first(), Some(&"L01")); // the earliest row (scrolled off the screen) survives
        assert_eq!(lines.last(), Some(&"L40"));
    }

    #[test]
    fn ingest_fitting_output_unchanged() {
        // #52 no-regression: output that FITS the screen (history == 0 → `-0..screen` == `0..screen`)
        // keeps exactly those rows — the full capture equals the visible snapshot.
        let chunk = [
            dcs_plain("init;id=1"),
            dcs_plain("preexec;command=run"),
            b"one\r\ntwo\r\nthree".to_vec(),
            dcs_plain("precmd;exit=0"),
        ]
        .concat();
        let mut mock = MockPtyChannel::new();
        mock.push_read(&chunk);
        let mut s = session(mock);
        let _events = s.pump().unwrap();
        assert_eq!(block_at(&s, 0).output_text(), "one\ntwo\nthree");
    }

    #[test]
    fn pump_block_output_trims_the_grids_trailing_blank_rows() {
        // The render grid is 24 rows tall, but a 2-line command's block output must be trimmed to its
        // real 2 rows — else the block is a full-screen block and history can't stack (#50). Kills an
        // identity/no-trim regression that output_text (which trims independently) cannot catch.
        let chunk = [
            dcs_plain("init;id=1"),
            dcs_plain("preexec;command=run"),
            b"abc\r\ndef".to_vec(),
        ]
        .concat();
        let mut mock = MockPtyChannel::new();
        mock.push_read(&chunk);
        let mut s = session(mock);
        let _events = s.pump().unwrap();
        let out = s.blocks().current().unwrap().output_styled();
        assert_eq!(
            out.len(),
            2,
            "output must be the 2 real rows, not the 24-row grid"
        );
        assert!(
            out.last()
                .unwrap()
                .iter()
                .any(|r| !r.text.trim().is_empty()),
            "the last row must be non-blank (no trailing blanks)"
        );
    }

    #[test]
    fn pump_leading_wouldblock_is_idle_fast_path_not_fatal() {
        // A LEADING WouldBlock (nothing read yet this call) returns Ok([]) IMMEDIATELY — the
        // idle fast-path (TICKET-023: sleeping here cost ~12 ms per idle pane per frame). It is
        // NOT a disconnect (kills `delete WouldBlock|Interrupted arm` → fatal Err): the queued
        // bytes arrive on the NEXT pump, exactly how the app's frame timer re-polls.
        let mut mock = MockPtyChannel::new();
        mock.push_read_err(io::ErrorKind::WouldBlock);
        mock.push_read(b"hi");
        let mut s = session(mock);
        assert_eq!(s.pump(), Ok(vec![]));
        assert_eq!(s.pump(), Ok(vec![SessionEvent::Wakeup]));
    }

    #[test]
    fn pump_midburst_wouldblock_retries_within_budget() {
        // Once bytes HAVE been read this call, a WouldBlock is a transient mid-burst pause: pump
        // sleeps and retries within the budget and picks up the rest — ONE pump, one Wakeup,
        // both chunks ingested. Kills `!got_bytes`→`got_bytes` (would fast-path out mid-burst
        // and drop the tail) and `budget == 0`→`!=` (would break on the first retry).
        let mut mock = MockPtyChannel::new();
        mock.push_read(&dcs_plain("init;id=1"));
        mock.push_read_err(io::ErrorKind::WouldBlock);
        mock.push_read(&dcs_plain("preexec;command=x"));
        let mut s = session(mock);
        assert_eq!(s.pump(), Ok(vec![SessionEvent::Wakeup]));
        // The TAIL chunk's preexec opened the block IN THIS SAME pump — the retry read it.
        assert_eq!(s.blocks().len(), 1);
    }

    #[test]
    fn pump_drops_unknown_selector_and_undecodable_hooks() {
        // Two DCS frames that ingest must SILENTLY drop: one with an unknown selector (`Z` → no
        // encoding) and one with a valid selector but an unknown-name payload (`nope`). These are
        // the two `continue` arms in ingest; pump still reports the read as a Wakeup and opens no
        // block (nothing registered the session).
        let mut bad_selector = vec![0x1b, b'P', b'Z'];
        bad_selector.extend_from_slice(b"init;id=1");
        bad_selector.extend_from_slice(&[0x1b, b'\\']);
        let chunk = [bad_selector, dcs_plain("nope")].concat();
        let mut mock = MockPtyChannel::new();
        mock.push_read(&chunk);
        let mut s = session(mock);
        assert_eq!(s.pump(), Ok(vec![SessionEvent::Wakeup]));
        assert_eq!(s.blocks().len(), 0);
    }

    #[test]
    fn pump_drops_a_hook_that_arrives_before_init_shell() {
        // R11 through the byte path: a `Preexec` with no `InitShell` before it opens no block.
        let mut mock = MockPtyChannel::new();
        mock.push_read(&dcs_plain("preexec;command=make"));
        let mut s = session(mock);
        assert_eq!(s.pump(), Ok(vec![SessionEvent::Wakeup]));
        assert_eq!(s.blocks().len(), 0);
        assert_eq!(s.block_epoch(), 0);
    }

    #[test]
    fn pump_budget_exhausts_cleanly() {
        // An endless WouldBlock source with NOTHING read takes the idle fast-path: Ok([])
        // immediately, no retry sleeps (TICKET-023 — the multi-pane frame killer).
        let mut mock = MockPtyChannel::new();
        mock.read_block_forever = true;
        let mut s = session(mock);
        assert_eq!(s.pump(), Ok(vec![]));
    }

    #[test]
    fn pump_burst_then_endless_wouldblock_exhausts_budget_cleanly() {
        // Bytes WERE read this call, then the source blocks forever: the MID-BURST retry path
        // drains the budget (8 × ~1 ms) and returns the Wakeup. Kills `!got_bytes || budget == 0`
        // → `&&` (would never break → a debug u32 underflow panic on `budget -= 1`) and
        // `-=` → `+=` (would loop forever → the cargo-mutants timeout).
        let mut mock = MockPtyChannel::new();
        mock.push_read(b"burst");
        mock.read_block_forever = true;
        let mut s = session(mock);
        assert_eq!(s.pump(), Ok(vec![SessionEvent::Wakeup]));
    }

    #[test]
    fn pump_child_exit_finishes_current_with_exact_code() {
        // poll_child_exit → Some(ExitCode(Some(3))): the running block finishes and a
        // ChildExited(Some(3)) event is emitted (R18, the 004 exact-code trap).
        let mut mock = MockPtyChannel::new();
        mock.child_exit.push_back(Some(ExitCode(Some(3))));
        let mut s = session(mock);
        s.apply_hook(DcsHook::InitShell {
            shell_session_id: ShellSessionId(1),
        })
        .unwrap();
        s.apply_hook(DcsHook::Preexec(PreexecValue {
            command: "x".into(),
        }))
        .unwrap();
        assert_eq!(
            s.pump(),
            Ok(vec![SessionEvent::ChildExited(ExitCode(Some(3)))])
        );
        assert!(s.blocks().current().is_none()); // finished
        assert_eq!(block_at(&s, 0).state, BlockState::Finished);
    }

    #[test]
    fn pump_fatal_read_without_child_exit_disconnects() {
        // A fatal read (BrokenPipe) with no child-exit → Disconnected (R15).
        let mut mock = MockPtyChannel::new();
        mock.push_read_err(io::ErrorKind::BrokenPipe);
        let mut s = session(mock);
        assert_eq!(s.pump(), Err(SessionError::Disconnected));
    }

    #[test]
    fn pump_fatal_read_with_child_exit_reports_exit() {
        // A fatal read but a child-exit available: the exit takes precedence over Disconnected.
        let mut mock = MockPtyChannel::new();
        mock.push_read_err(io::ErrorKind::BrokenPipe);
        mock.child_exit.push_back(Some(ExitCode(Some(0))));
        let mut s = session(mock);
        assert_eq!(
            s.pump(),
            Ok(vec![SessionEvent::ChildExited(ExitCode(Some(0)))])
        );
    }

    // ── #423 — the EIO exit-grace + the child-exited write latch ────────────
    // The Linux order-race: the slave fds die WITH the child, so the master's EIO routinely
    // arrives BEFORE the SIGCHLD self-pipe byte. The pump must ride it out (bounded), and the
    // post-exit write contract must live in STATE (Linux masters accept post-close writes).

    #[test]
    fn pump_eio_within_grace_reports_quietly_then_exit_lands() {
        let mut mock = MockPtyChannel::new();
        mock.push_read_raw_err(EIO);
        mock.child_exit.push_back(None); // the SIGCHLD byte hasn't landed yet...
        mock.child_exit.push_back(Some(ExitCode(Some(0)))); // ...it lands for the NEXT pump
        let mut s = session(mock);
        assert_eq!(s.pump(), Ok(vec![])); // within the grace: quiet, NOT Disconnected
        assert_eq!(
            s.pump(),
            Ok(vec![SessionEvent::ChildExited(ExitCode(Some(0)))])
        );
    }

    #[test]
    fn eio_grace_expiry_boundary() {
        // The pure boundary a live Instant can never hold still for: elapsed == GRACE counts as
        // EXPIRED (kills the `>=`-vs-`>` mutant deterministically; the neighbors kill the rest).
        assert!(!eio_grace_expired(
            EIO_EXIT_GRACE.saturating_sub(Duration::from_nanos(1))
        ));
        assert!(eio_grace_expired(EIO_EXIT_GRACE));
        assert!(eio_grace_expired(EIO_EXIT_GRACE + Duration::from_nanos(1)));
    }

    #[test]
    fn pump_eio_grace_expiry_disconnects() {
        let mut mock = MockPtyChannel::new();
        mock.push_read_raw_err(EIO);
        mock.push_read_raw_err(EIO);
        let mut s = session(mock);
        assert_eq!(s.pump(), Ok(vec![])); // grace armed
        s.expire_eio_grace_for_test();
        // Still EIO, still no exit event past the deadline → the honest no-exit disconnect.
        assert_eq!(s.pump(), Err(SessionError::Disconnected));
    }

    #[test]
    fn pump_bytes_then_eio_restarts_the_grace() {
        // Burst-then-die: a successful read CLEARS an armed grace and the same pump's trailing
        // EIO re-arms it FRESH — a stale timestamp must never instantly expire the new grace.
        let mut mock = MockPtyChannel::new();
        mock.push_read_raw_err(EIO);
        mock.push_read(b"x");
        mock.push_read_raw_err(EIO);
        let mut s = session(mock);
        assert_eq!(s.pump(), Ok(vec![])); // grace armed at T0
        s.expire_eio_grace_for_test(); // T0 now reads as expired...
        // ...but this pump's bytes clear it, and its trailing EIO starts a FRESH grace → quiet.
        assert_eq!(s.pump(), Ok(vec![SessionEvent::Wakeup]));
    }

    #[test]
    fn pump_eio_after_observed_exit_disconnects_instantly() {
        // Once ChildExited has SURFACED there is nothing to wait for: a later EIO is an instant
        // Disconnected (the pre-#423 dead-pane cadence — no 1 s quiet window per dead pane).
        let mut mock = MockPtyChannel::new();
        mock.push_read_err(io::ErrorKind::WouldBlock); // pump 1: idle read, exit event surfaces
        mock.push_read_raw_err(EIO); // pump 2: EIO on the already-dead session
        mock.child_exit.push_back(Some(ExitCode(Some(0))));
        let mut s = session(mock);
        assert_eq!(
            s.pump(),
            Ok(vec![SessionEvent::ChildExited(ExitCode(Some(0)))])
        );
        assert_eq!(s.pump(), Err(SessionError::Disconnected));
    }

    #[test]
    fn write_after_observed_exit_disconnects_by_state() {
        // The latch beats errno: the mock WOULD accept the write (its exhausted write queue
        // records + succeeds — the Linux-master shape), so only the state check can refuse.
        let mut mock = MockPtyChannel::new();
        mock.child_exit.push_back(Some(ExitCode(Some(0))));
        let recorded = Arc::clone(&mock.recorded_writes);
        let mut s = session(mock);
        assert_eq!(
            s.pump(),
            Ok(vec![SessionEvent::ChildExited(ExitCode(Some(0)))])
        );
        assert_eq!(s.write_bytes(b"late"), Err(SessionError::Disconnected));
        assert!(
            recorded.lock().unwrap().is_empty(),
            "the latch must refuse BEFORE touching the channel"
        );
    }

    // ── resize (R17) ────────────────────────────────────────────────────────
    #[test]
    fn resize_success() {
        let mut mock = MockPtyChannel::new();
        mock.winsize.push_back(Ok(()));
        let mut s = session(mock);
        assert_eq!(s.resize(100, 40), Ok(()));
    }

    #[test]
    fn resize_maps_winsize_error() {
        let mut mock = MockPtyChannel::new();
        mock.winsize
            .push_back(Err(io::Error::from(io::ErrorKind::Other)));
        let mut s = session(mock);
        assert_eq!(s.resize(100, 40), Err(SessionError::Resize));
    }

    // ── session_id / apply_hook delegate / blocks ───────────────────────────
    #[test]
    fn session_id_returns_constructed_id() {
        let s = session(MockPtyChannel::new());
        assert_eq!(s.session_id(), SessionId::from(1));
        s.shutdown(); // exercise the (empty) shutdown consumer
    }

    #[test]
    fn apply_hook_delegates_to_model() {
        let mut s = session(MockPtyChannel::new());
        // Before InitShell, Preexec is rejected (delegation reaches the model's guard).
        assert_eq!(
            s.apply_hook(DcsHook::Preexec(PreexecValue {
                command: "x".into(),
            })),
            Err(ApplyHookError::MissingSession)
        );
        s.apply_hook(DcsHook::InitShell {
            shell_session_id: ShellSessionId(1),
        })
        .unwrap();
        s.apply_hook(DcsHook::Preexec(PreexecValue {
            command: "x".into(),
        }))
        .unwrap();
        // blocks() reads the real list after the mutation (kills blocks -> leak(Default)).
        assert_eq!(s.blocks().len(), 1);
    }

    #[test]
    fn output_text_does_not_touch_the_pty() {
        // Render a block, then read output_text repeatedly: no further PTY reads occur (R19 — the
        // output is an owned view, not a live grid).
        let chunk = [
            dcs_plain("init;id=1"),
            dcs_plain("preexec;command=run"),
            b"hi".to_vec(),
            dcs_plain("precmd;exit=0"),
        ]
        .concat();
        let mut mock = MockPtyChannel::new();
        mock.push_read(&chunk);
        let read_count = Arc::clone(&mock.read_count);
        let mut s = session(mock);
        let _events = s.pump().unwrap();
        let after_pump = read_count.load(Ordering::Relaxed);
        for _ in 0..5 {
            let _text = block_at(&s, 0).output_text();
        }
        assert_eq!(read_count.load(Ordering::Relaxed), after_pump);
    }

    // ── the gate:4 coverage trap + the Dims dimension mutants ───────────────
    #[test]
    fn dims_methods_are_exercised() {
        // `Term` never calls total_lines at runtime, so it is only exercised here;
        // screen_lines/columns are asserted directly.
        let dims = Dims {
            lines: 24,
            cols: 80,
        };
        assert_eq!(dims.total_lines(), 24);
        assert_eq!(dims.screen_lines(), 24);
        assert_eq!(dims.columns(), 80);
    }

    // ── R26 — is_alt_screen tracks DECSET 1049 (entering the alternate screen) ──────
    #[test]
    fn is_alt_screen_tracks_decset() {
        let mut mock = MockPtyChannel::new();
        // DECSET 1049 (enter the alternate screen) — CSI (ESC [), not DCS (ESC P), so the scanner
        // passes it to the ANSI parser → the term mode flips. (A pump consumes all queued reads,
        // so a single enter sequence is the clean observation.)
        mock.push_read(b"\x1b[?1049h");
        let mut s = session(mock);
        assert!(!s.is_alt_screen()); // starts on the primary screen (false)
        let _events = s.pump().unwrap();
        assert!(s.is_alt_screen()); // entered the alternate screen (true) — kills contains/flag mutants
    }

    // ── M17 #280 — mouse_modes tracks the DECSET mouse flags (kills the Default-body
    // mutant IN-CRATE: cargo-mutants scopes to the mutated package, so the app-side
    // headless flow cannot carry this one).
    #[test]
    fn mouse_modes_track_decset_flags() {
        let mut mock = MockPtyChannel::new();
        mock.push_read(b"\x1b[?1000h\x1b[?1006h\x1b[?1007h");
        let mut s = session(mock);
        assert!(
            !s.mouse_modes().tracking(),
            "no tracking before the DECSETs"
        );
        let _events = s.pump().unwrap();
        let m = s.mouse_modes();
        assert!(
            m.click && m.sgr && m.alt_scroll,
            "1000+1006+1007 all flipped"
        );
        assert!(
            !m.drag && !m.motion && !m.alt_screen,
            "unset flags stay false"
        );
        assert!(m.tracking());
    }

    // ── M17 #286 — DECCKM (DECSET 1) flips application-cursor mode; both the accessor and the
    // snapshot mirror it. before/after the pump covers the flag BOTH ways (kills the contains /
    // bool-fn / Default-body mutants in-crate).
    #[test]
    fn app_cursor_tracks_decckm() {
        let mut mock = MockPtyChannel::new();
        mock.push_read(b"\x1b[?1h"); // DECSET 1 — application cursor keys
        let mut s = session(mock);
        assert!(!s.is_app_cursor(), "cursor keys are normal by default");
        assert!(!s.mouse_modes().app_cursor);
        let _events = s.pump().unwrap();
        assert!(s.is_app_cursor(), "DECSET 1 flips application-cursor on");
        assert!(
            s.mouse_modes().app_cursor,
            "the snapshot mirrors the accessor"
        );
    }

    // ── R28 — is_bracketed_paste tracks DECSET 2004 (drives #42's paste wrapping) ──
    #[test]
    fn is_bracketed_paste_tracks_decset() {
        let mut mock = MockPtyChannel::new();
        // DECSET 2004 (enable bracketed paste) — a private-mode CSI the term parser tracks.
        mock.push_read(b"\x1b[?2004h");
        let mut s = session(mock);
        assert!(!s.is_bracketed_paste()); // off by default
        let _events = s.pump().unwrap();
        assert!(s.is_bracketed_paste()); // on after DECSET 2004 — kills contains/flag mutants
    }

    // ── R26 — is_command_running tracks the foreground Running block (drives #40 input routing) ──
    #[test]
    fn is_command_running_tracks_the_foreground_block() {
        // Build a session, pump one coalesced chunk, and report whether a command is running.
        fn ran(chunk: &[u8]) -> bool {
            let mut mock = MockPtyChannel::new();
            mock.push_read(chunk);
            mock.push_read_err(io::ErrorKind::WouldBlock);
            let mut s = session(mock);
            let _events = s.pump().unwrap();
            s.is_command_running()
        }
        // A fresh session — no command has ever run → not running (kills the `-> true` mutant).
        assert!(!session(MockPtyChannel::new()).is_command_running());
        // init THEN preexec opens a Running block → a command is running (kills `-> false`). init is
        // required first — a Preexec on an unregistered session is rejected (the #37 precondition).
        assert!(ran(&[
            dcs_plain("init;id=1"),
            dcs_plain("preexec;command=x")
        ]
        .concat()));
        // ...and the next precmd finishes that block → back at the prompt, not running.
        assert!(!ran(&[
            dcs_plain("init;id=1"),
            dcs_plain("preexec;command=x"),
            dcs_plain("precmd;exit=0"),
        ]
        .concat()));
    }

    // ── R27 — current_prompt delegates the staged context (init BEFORE precmd is REQUIRED) ──
    #[test]
    fn current_prompt_delegates_the_staged_context() {
        // A fresh session (no precmd) → None.
        assert!(session(MockPtyChannel::new()).current_prompt().is_none());
        // init THEN precmd — a Precmd on an unregistered session stages nothing, so the ordering is
        // load-bearing; once the pump applies both hooks the delegator returns Some.
        let chunk = [dcs_plain("init;id=1"), dcs_plain("precmd;exit=0")].concat();
        let mut mock = MockPtyChannel::new();
        mock.push_read(&chunk);
        mock.push_read_err(io::ErrorKind::WouldBlock);
        let mut s = session(mock);
        let _events = s.pump().unwrap();
        assert!(s.current_prompt().is_some());
    }

    // ── R26 — grid_styled_rows reflects the live grid ──────────────────────────────
    #[test]
    fn grid_styled_rows_reflects_grid() {
        let mut mock = MockPtyChannel::new();
        mock.push_read(b"hi");
        let mut s = session(mock);
        let _events = s.pump().unwrap();
        let rows = s.grid_styled_rows();
        // The first row's concatenated run text is "hi" (kills grid_styled_rows -> vec![]).
        let first: String = rows[0].iter().map(|r| r.text.as_str()).collect();
        assert_eq!(first, "hi");
    }

    // ── #214 — an OSC 8 hyperlink escape populates StyledRun.hyperlink (end-to-end extraction) ──
    #[test]
    fn grid_styled_rows_carries_osc8_hyperlink() {
        // OSC 8 = ESC ] 8 ; <params> ; <uri> ST  <text>  ESC ] 8 ; ; ST   (ST = ESC \). alacritty
        // parses this into the cell's hyperlink (not config-gated); term_to_styled_rows carries
        // cell.hyperlink() into the run so the render can prefer it over the text-scan heuristic.
        let mut osc8 = Vec::new();
        osc8.extend_from_slice(b"\x1b]8;;https://example.com\x1b\\");
        osc8.extend_from_slice(b"click");
        osc8.extend_from_slice(b"\x1b]8;;\x1b\\");
        let mut mock = MockPtyChannel::new();
        mock.push_read(&osc8);
        let mut s = session(mock);
        let _events = s.pump().unwrap();
        // The run rendering "click" carries the explicit hyperlink URI; a plain char would carry None.
        let link_run = s
            .grid_styled_rows()
            .into_iter()
            .flatten()
            .find(|r| r.text.contains("click"))
            .expect("a run rendering the linked text");
        assert_eq!(link_run.hyperlink.as_deref(), Some("https://example.com"));
    }

    // ── #214 — a FINISHED block's output (the #52 full-history capture path) also carries the OSC 8
    // hyperlink: full_term_to_styled_rows extracts cell.hyperlink() at command finish, not just the
    // live grid snapshot. ──
    #[test]
    fn finished_block_output_carries_osc8_hyperlink() {
        let mut osc8 = Vec::new();
        osc8.extend_from_slice(b"\x1b]8;;https://example.com\x1b\\click\x1b]8;;\x1b\\");
        let chunk = [
            dcs_plain("init;id=1"),
            dcs_plain("preexec;command=echo"),
            osc8,
            dcs_plain("precmd;exit=0"),
        ]
        .concat();
        let mut mock = MockPtyChannel::new();
        mock.push_read(&chunk);
        let mut s = session(mock);
        let _events = s.pump().unwrap();
        let link_run = block_at(&s, 0)
            .output_styled()
            .iter()
            .flatten()
            .find(|r| r.text.contains("click"))
            .expect("a run rendering the linked text in the finished block");
        assert_eq!(link_run.hyperlink.as_deref(), Some("https://example.com"));
    }

    // ── R25 — the Ctrl-C encode→write path delivers the interrupt byte ─────────────
    #[test]
    fn ctrl_c_writes_the_interrupt_byte() {
        let mock = MockPtyChannel::new();
        let recorded = Arc::clone(&mock.recorded_writes);
        let mut s = session(mock);
        let bytes = crate::keys::encode_key(
            crate::keys::KeyInput {
                code: crate::keys::KeyCode::Char('c'),
                ctrl: true,
                alt: false,
                shift: false,
            },
            false,
        );
        assert_eq!(s.write_bytes(&bytes), Ok(()));
        assert_eq!(&*recorded.lock().unwrap(), &[0x03]); // SIGINT byte reached the PTY leader
    }

    // ── TICKET-348 — reap_step escalation truth table (REQ-004) ──────────────
    #[test]
    fn reap_step_done_when_child_exited_at_any_stage() {
        // `exited` short-circuits to Done regardless of stage or elapsed (kills the `if exited` delete).
        assert_eq!(reap_step(ReapStage::Hup, 0, true), ReapAction::Done);
        assert_eq!(reap_step(ReapStage::Hup, 5000, true), ReapAction::Done);
        assert_eq!(reap_step(ReapStage::Kill, 0, true), ReapAction::Done);
        assert_eq!(reap_step(ReapStage::Kill, 5000, true), ReapAction::Done);
    }

    #[test]
    fn reap_step_hup_waits_then_escalates_to_kill_at_the_deadline() {
        assert_eq!(REAP_DEADLINE_MS, 2000);
        // Before the deadline → Wait; at/after it → SendKill. 1999/2000/2001 pins the `>=` boundary.
        assert_eq!(reap_step(ReapStage::Hup, 1999, false), ReapAction::Wait);
        assert_eq!(reap_step(ReapStage::Hup, 2000, false), ReapAction::SendKill);
        assert_eq!(reap_step(ReapStage::Hup, 2001, false), ReapAction::SendKill);
        assert_eq!(reap_step(ReapStage::Hup, 0, false), ReapAction::Wait);
    }

    #[test]
    fn reap_step_kill_waits_then_gives_up_at_the_deadline() {
        // The Kill stage's past-deadline action is GiveUp (NOT SendKill — kills a stage-arm swap).
        assert_eq!(reap_step(ReapStage::Kill, 1999, false), ReapAction::Wait);
        assert_eq!(reap_step(ReapStage::Kill, 2000, false), ReapAction::GiveUp);
        assert_eq!(reap_step(ReapStage::Kill, 2001, false), ReapAction::GiveUp);
        assert_eq!(reap_step(ReapStage::Kill, 0, false), ReapAction::Wait);
    }
}
