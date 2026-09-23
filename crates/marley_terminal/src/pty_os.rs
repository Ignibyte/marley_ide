//! SHIM (ACCEPTED-UNTESTABLE) — the four raw OS calls behind [`crate::session::PtyChannel`].
//!
//! These have no deterministic unit harness; the real-PTY integration tests exercise them end to
//! end. The spawn-failure mapping to [`SessionError::Spawn`] lives here too, so the uncoverable
//! error path stays inside this file rather than leaking into the testable `session` logic.

use std::fs::File;
use std::io::{Read, Write};
use std::time::{Duration, Instant};

use alacritty_terminal::event::WindowSize;
use alacritty_terminal::tty::{self, ChildEvent, EventedPty, Options, Shell};

use crate::block::ExitCode;
use crate::session::{PtyChannel, ReapAction, ReapStage, SessionError, SessionOptions, reap_step};

/// The real PTY channel: the live `Pty` (held so its leader fd stays open) plus a `try_clone`d
/// leader-fd `File` for the `O_NONBLOCK` read/write path. The `Pty` is an `Option` so our [`Drop`]
/// can CHOOSE whether to run `alacritty_terminal`'s `Pty::Drop` (which reaps via an unbounded
/// `waitpid`): after a successful bounded reap the child is already dead, so we leave it in place and
/// let it drop normally (its `wait()` returns at once and it closes the fds); on the pathological
/// give-up path we `mem::forget` it to skip that blocking `Drop` (leaking to avoid the hang —
/// TICKET-348 D5). It is `Some` for the channel's whole life except after a give-up leak.
pub(super) struct OsPtyChannel {
    pty: Option<tty::Pty>,
    io: File,
    /// Set once [`poll_child_exit`](OsPtyChannel::poll_child_exit) has seen the child exit. The
    /// child-exit poll is EDGE-triggered (`next_child_event` consumes the one `SIGCHLD` self-pipe
    /// byte, then returns `None`), so a child reaped during normal pumping is invisible to a later
    /// poll. `Drop` reads this flag to skip the reap loop (and its signals to a possibly-recycled
    /// pid) when the child is already gone — alacritty's `Pty::Drop` then reaps from the cached
    /// status at once.
    child_reaped: bool,
}

/// Spawn `options.shell` on a fresh `cols`×`rows` PTY (R12), holding the `Pty` so its output is not
/// discarded before the first read. Any failure maps to [`SessionError::Spawn`].
pub(super) fn spawn(options: &SessionOptions) -> Result<OsPtyChannel, SessionError> {
    let pty_options = Options {
        shell: Some(Shell::new(
            options.shell.to_string_lossy().into_owned(),
            options.args.clone(),
        )),
        working_directory: Some(options.cwd.clone()),
        env: options.env.iter().cloned().collect(),
        ..Default::default()
    };
    let window_size = WindowSize {
        num_lines: options.rows,
        num_cols: options.cols,
        cell_width: 8,
        cell_height: 16,
    };
    let pty = tty::new(&pty_options, window_size, 0).map_err(|_| SessionError::Spawn)?;
    let io = pty.file().try_clone().map_err(|_| SessionError::Spawn)?;
    Ok(OsPtyChannel {
        pty: Some(pty),
        io,
        child_reaped: false,
    })
}

impl PtyChannel for OsPtyChannel {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        (&self.io).read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        (&self.io).write(buf)
    }

    fn set_winsize(&mut self, cols: u16, rows: u16) -> std::io::Result<()> {
        rustix::termios::tcsetwinsize(
            &self.io,
            rustix::termios::Winsize {
                ws_row: rows,
                ws_col: cols,
                ws_xpixel: 0,
                ws_ypixel: 0,
            },
        )?;
        Ok(())
    }

    fn poll_child_exit(&mut self) -> Option<ExitCode> {
        let event = self.pty.as_mut().and_then(EventedPty::next_child_event);
        if event.is_some() {
            // Remember the exit: the poll is edge-triggered, so this is the ONLY time we will see it.
            self.child_reaped = true;
        }
        event.map(|ChildEvent::Exited(status)| ExitCode(status.and_then(|s| s.code())))
    }
}

impl Drop for OsPtyChannel {
    /// BOUNDED child reap (TICKET-348): `SIGHUP` → poll for exit under a deadline → `SIGKILL` → poll
    /// under a second deadline → give up and leak. Replaces the implicit UNBOUNDED `waitpid` that
    /// `alacritty_terminal`'s `Pty::Drop` blocks on (the 11-hour-hang source). ACCEPTED-UNTESTABLE
    /// (real signals + wall-clock): the escalation decisions live in the pure [`reap_step`]; this only
    /// executes them and runs the syscalls.
    fn drop(&mut self) {
        // The child's pid — still readable when already reaped (the `Child` keeps its id); used by
        // the reap signals and the give-up log. `pty` is `Some` here (only a give-up leak sets it
        // `None`, and that is the last thing this `Drop` does).
        let raw = match self.pty.as_ref() {
            Some(pty) => pty.child().id().cast_signed(),
            None => return,
        };
        // If the child was already reaped during normal pumping, its exit is invisible to the
        // edge-triggered poll (the `SIGCHLD` byte is gone) and its pid may have been recycled — so
        // do NOT signal or loop; alacritty's `Pty::Drop` reaps from the cached status at once.
        let reaped = if self.child_reaped {
            true
        } else {
            let pid = rustix::process::Pid::from_raw(raw);
            let signal = |sig| {
                // ESRCH: the child exited between the reap check and this signal, and the poll
                // below sees that exit. Any other failure is worth a line in the log.
                if let Some(p) = pid
                    && let Err(error) = rustix::process::kill_process(p, sig)
                    && error != rustix::io::Errno::SRCH
                {
                    log::warn!("marley_terminal: signalling the shell failed: {error}");
                }
            };
            // Stage 1 — hang up the controlling terminal, then poll for exit under the first deadline.
            signal(rustix::process::Signal::HUP);
            let mut stage = ReapStage::Hup;
            let mut started = Instant::now();
            loop {
                let exited = self.poll_child_exit().is_some();
                let elapsed = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
                match reap_step(stage, elapsed, exited) {
                    ReapAction::Done => break true,
                    ReapAction::Wait => std::thread::sleep(Duration::from_millis(20)),
                    ReapAction::SendKill => {
                        signal(rustix::process::Signal::KILL);
                        stage = ReapStage::Kill;
                        started = Instant::now();
                    }
                    ReapAction::GiveUp => break false,
                }
            }
        };
        if !reaped {
            // D5 zombie-over-hang: the child outlived `SIGKILL` (a pathological D-state). `mem::forget`
            // the `Pty` so alacritty's `Pty::Drop` — whose `child.wait()` would block forever — never
            // runs. Leak the child + its fds deliberately: a zombie is recoverable, a hang is not.
            std::mem::forget(self.pty.take());
            log::warn!(
                "terminal_blocks: PTY child {raw} survived SIGKILL past the reap deadline — \
                 leaking to avoid a teardown hang (TICKET-348 D5)"
            );
        }
        // On the reaped path `self.pty` stays `Some` and drops normally when the struct's fields drop:
        // alacritty's `Pty::Drop` runs, and with the child already reaped its `child.wait()` returns at
        // once (and closes the leader/signal fds). `self.io` (the try_clone'd leader fd) drops either way.
    }
}
