//! REAL-PTY integration (R12/R13/R15/R16/R17/R18) — the only exercise of the `pty_os` shim
//! (the raw `tty::new` spawn, the leader-fd read/write, `tcsetwinsize`, and the `next_child_event`
//! reap). Each test spawns a genuine `/bin/sh` on a fresh PTY in a throwaway `tempfile` cwd. The
//! deterministic lifecycle tests (spawn, child-exit, write-after-disconnect, resize) carry the
//! end-to-end coverage; one best-effort DCS round-trip drives a scripted hook stream through a real
//! shell. All hold `PTY_LOCK` — they share process/SIGCHLD state.
#![cfg(unix)]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use marley_terminal::{SessionError, SessionEvent, SessionOptions, TerminalSession};
/// The real-PTY tests share the process-wide SIGCHLD self-pipe, so they never run concurrently.
static PTY_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serialized() -> std::sync::MutexGuard<'static, ()> {
    PTY_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Spawn `/bin/sh` on a fresh 80×24 PTY rooted in a throwaway temp dir. Returns the session and the
/// `TempDir` guard (kept alive so the cwd survives the test).
fn spawn_sh() -> (TerminalSession, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let options = SessionOptions {
        shell: PathBuf::from("/bin/sh"),
        args: Vec::new(),
        cwd: dir.path().to_path_buf(),
        env: vec![],
        cols: 80,
        rows: 24,
    };
    let session = TerminalSession::spawn(options).expect("spawn /bin/sh on a PTY");
    (session, dir)
}

/// Spawn `program` with `args` on a fresh `cols`×`rows` PTY rooted in a throwaway temp dir.
fn spawn_program(
    program: &str,
    args: &[&str],
    cols: u16,
    rows: u16,
) -> (TerminalSession, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let options = SessionOptions {
        shell: PathBuf::from(program),
        args: args.iter().map(|arg| arg.to_string()).collect(),
        cwd: dir.path().to_path_buf(),
        env: vec![],
        cols,
        rows,
    };
    let session = TerminalSession::spawn(options).expect("spawn a program on a PTY");
    (session, dir)
}

/// The live grid's text, one line per row.
fn screen_text(session: &TerminalSession) -> String {
    session
        .grid_styled_rows()
        .iter()
        .map(|row| row.iter().map(|run| run.text.as_str()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Pump until the screen shows `needle` or the deadline passes.
fn pump_until_screen_shows(
    session: &mut TerminalSession,
    needle: &str,
    deadline: Duration,
) -> bool {
    pump_until(session, deadline, |session, _events| {
        screen_text(session).contains(needle).then_some(())
    })
    .is_some()
}

/// Pump until `pred` holds (returning its captured value) or the deadline passes.
fn pump_until<T>(
    session: &mut TerminalSession,
    deadline: Duration,
    mut pred: impl FnMut(&mut TerminalSession, &[SessionEvent]) -> Option<T>,
) -> Option<T> {
    let start = Instant::now();
    while start.elapsed() < deadline {
        match session.pump() {
            Ok(events) => {
                if let Some(v) = pred(session, &events) {
                    return Some(v);
                }
            }
            Err(_) => return None,
        }
    }
    None
}

// ── R12 — spawn yields a live, process-unique session ───────────────────────
#[test]
fn spawn_real_pty_succeeds() {
    let _serialized = serialized();
    let (session, _dir) = spawn_sh();
    // A real spawn returns a usable handle with a process-unique id.
    let _ = session.session_id();
    session.shutdown();
}

// ── R18 — a real child exit surfaces its exact code ─────────────────────────
#[test]
fn child_exit_reports_exact_code() {
    let _serialized = serialized();
    let (mut session, _dir) = spawn_sh();
    session.write_command("exit 3").expect("write exit");
    let code = pump_until(&mut session, Duration::from_secs(5), |_s, events| {
        events.iter().find_map(|e| match e {
            SessionEvent::ChildExited(code) => Some(*code),
            _ => None,
        })
    });
    assert_eq!(
        code,
        Some(marley_terminal::ExitCode(Some(3))),
        "the shell should exit with code 3"
    );
}

// ── R15 — writing after the child has gone surfaces Disconnected ────────────
#[test]
fn write_after_disconnect_errors() {
    let _serialized = serialized();
    let (mut session, _dir) = spawn_sh();
    session.write_command("exit 0").expect("write exit");
    // Drain until the child has exited — and ASSERT it surfaced: the exit event is this test's
    // precondition, and its non-arrival is the #423 order-race, not a skippable hiccup.
    let observed = pump_until(&mut session, Duration::from_secs(15), |_s, events| {
        events
            .iter()
            .any(|e| matches!(e, SessionEvent::ChildExited(_)))
            .then_some(())
    });
    assert!(
        observed.is_some(),
        "ChildExited must surface within the deadline (the #423 EIO/SIGCHLD order race)"
    );
    // #423: once the exit is OBSERVED, the very FIRST write reports Disconnected — by STATE,
    // uniformly on macOS AND Linux. (The old errno-retry loop spun green on Linux, whose master
    // happily accepts post-close writes into the flip buffer.)
    assert_eq!(
        session.write_command("echo late"),
        Err(SessionError::Disconnected)
    );
}

// ── R17 — resize drives the real winsize ioctl without aborting ─────────────
#[test]
fn resize_real_pty_succeeds() {
    let _serialized = serialized();
    let (mut session, _dir) = spawn_sh();
    assert_eq!(session.resize(120, 40), Ok(()));
    session.shutdown();
}

// ── TICKET-348 — teardown is BOUNDED even with a LIVE (non-exited) child ─────
#[test]
fn teardown_is_bounded_with_a_live_child() {
    let _serialized = serialized();
    let (session, _dir) = spawn_sh(); // /bin/sh is alive and never told to exit
    let start = Instant::now();
    session.shutdown(); // SIGHUP → the bounded reap; the shell dies on HUP
    assert!(
        start.elapsed() < Duration::from_secs(5),
        "teardown of a live child must finish within the HUP+KILL bound (~4s), not hang"
    );
}

// ── TICKET-348 (F1) — a child reaped during pumping tears down INSTANTLY ──────
// Regression for the edge-triggered-poll leak: the pump reaps a normally-exited shell and consumes
// its one SIGCHLD self-pipe byte, so a later teardown poll can no longer observe the exit. The
// `child_reaped` latch is what lets `shutdown` skip the 4s HUP+KILL loop here.
#[test]
fn teardown_is_instant_when_child_already_reaped() {
    let _serialized = serialized();
    let (mut session, _dir) = spawn_sh();
    session.write_command("exit 0").expect("write exit");
    // 15 s ceiling (#423 D4): generous under lane load/CPU quota; early-exit keeps green runs
    // fast. The #423 pump EIO-grace makes the exit's arrival order-independent (Linux EIO can
    // beat the SIGCHLD byte; the pump now waits it out instead of erroring).
    let reaped = pump_until(&mut session, Duration::from_secs(15), |_s, events| {
        events
            .iter()
            .any(|e| matches!(e, SessionEvent::ChildExited(_)))
            .then_some(())
    });
    assert!(
        reaped.is_some(),
        "the shell should exit and be reaped by the pump (setting the latch)"
    );
    let start = Instant::now();
    session.shutdown();
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "an already-reaped child must tear down instantly via the child_reaped latch \
         (without the fix this runs the full ~4s HUP+KILL loop)"
    );
}

// ── R12/R13/R16 — a scripted DCS hook stream round-trips through a real shell ─
//
// Best-effort: a real shell's echo/prompt timing can vary, so this drives the hooks and asserts the
// resulting Block; if the round-trip does not complete in the budget the deterministic lifecycle
// tests above still carry spawn/read/write/child-exit coverage.
#[test]
fn dcs_hook_stream_produces_a_block() {
    let _serialized = serialized();
    let (mut session, _dir) = spawn_sh();
    // Let the shell print its initial prompt.
    pump_until(
        &mut session,
        Duration::from_millis(500),
        |_s, _e| None::<()>,
    );
    // One line emitting: init DCS, preexec DCS, the command output "hi", then precmd DCS (exit 0).
    // The wire format is `ESC P p <name;key=value;…> ESC \` (Plain selector `p`); printf's `\033`
    // is ESC and `\\` is the terminating backslash.
    let script = "printf '\\033Ppinit;id=1\\033\\\\'; \
         printf '\\033Pppreexec;command=echo hi\\033\\\\'; \
         echo hi; \
         printf '\\033Ppprecmd;exit=0\\033\\\\'";
    session.write_command(script).expect("write script");

    let found = pump_until(&mut session, Duration::from_secs(5), |s, _events| {
        s.blocks()
            .iter()
            .find(|b| {
                b.state == marley_terminal::BlockState::Finished && b.output_text().contains("hi")
            })
            .map(|b| b.exit_code)
    });

    assert_eq!(
        found,
        Some(marley_terminal::ExitCode(Some(0))),
        "expected a Finished block containing \"hi\" with exit 0"
    );
}

// ── R12 — the spawn runs the given program with its arguments, not a default shell ──
#[test]
fn spawn_runs_the_given_program_and_arguments() {
    let _serialized = serialized();
    let (mut session, _dir) = spawn_program("/bin/sh", &["-c", "exit 7"], 80, 24);
    let code = pump_until(&mut session, Duration::from_secs(5), |_s, events| {
        events.iter().find_map(|e| match e {
            SessionEvent::ChildExited(code) => Some(*code),
            _ => None,
        })
    });
    assert_eq!(
        code,
        Some(marley_terminal::ExitCode(Some(7))),
        "`sh -c 'exit 7'` must run as given and exit 7 on its own"
    );
}

// ── R12 — the child starts in the requested working directory ────────────────
#[test]
fn spawn_starts_in_the_given_directory() {
    let _serialized = serialized();
    // Wide enough that a long temp path never wraps across rows.
    let (mut session, dir) = spawn_program("/bin/sh", &[], 400, 24);
    let expected = dir
        .path()
        .canonicalize()
        .expect("the temp dir resolves")
        .display()
        .to_string();
    session.write_command("pwd -P").expect("write pwd");
    assert!(
        pump_until_screen_shows(&mut session, &expected, Duration::from_secs(5)),
        "the shell must start in {expected}; the screen showed:\n{}",
        screen_text(&session)
    );
    session.shutdown();
}

// ── R17 — a resize reaches the program on the PTY ───────────────────────────
#[test]
fn resize_reaches_the_program() {
    let _serialized = serialized();
    let (mut session, _dir) = spawn_program("/bin/sh", &[], 120, 24);
    assert_eq!(session.resize(100, 30), Ok(()));
    session.write_command("stty size").expect("write stty");
    assert!(
        pump_until_screen_shows(&mut session, "30 100", Duration::from_secs(5)),
        "`stty size` must report the resized 30 rows by 100 columns; the screen showed:\n{}",
        screen_text(&session)
    );
    session.shutdown();
}

// ── TICKET-348 — teardown escalates to SIGKILL for a child that ignores SIGHUP ──
// Without the bounded reap, alacritty's own `Pty::Drop` hangs up and then waits for the child.
// The child ends on its own after 30 s: it calls `setsid`, so a test process that dies first
// (a failed run, a killed mutant) leaves it outside nextest's reach, and it must not live on.
// The ignored HUP survives the `exec`.
#[test]
fn teardown_kills_a_child_that_ignores_hangup() {
    let _serialized = serialized();
    let (mut session, _dir) = spawn_program(
        "/bin/sh",
        &["-c", "trap '' HUP; echo trap-set; exec sleep 30"],
        80,
        24,
    );
    // A hang-up that beats the trap kills the shell outright and the SIGKILL path never runs.
    assert!(
        pump_until_screen_shows(&mut session, "trap-set", Duration::from_secs(5)),
        "the child must install its HUP trap; the screen showed:\n{}",
        screen_text(&session)
    );
    let (done, finished) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        session.shutdown();
        done.send(()).expect("the test is still waiting");
    });
    assert!(
        finished.recv_timeout(Duration::from_secs(8)).is_ok(),
        "teardown must SIGKILL a child that ignores SIGHUP within the two reap deadlines"
    );
}

// ── TICKET-348 — a normal teardown closes the PTY's file descriptors ────────
#[test]
fn teardown_closes_the_pty_file_descriptors() {
    fn open_descriptors() -> usize {
        // `/dev/fd` lists the calling process's descriptors on Linux and macOS alike.
        std::fs::read_dir("/dev/fd")
            .expect("list this process's descriptors")
            .count()
    }
    let _serialized = serialized();
    // A precaution: a first spawn may leave one-time process state behind, so the count starts
    // after one and measures only the second teardown.
    spawn_sh().0.shutdown();
    let before = open_descriptors();
    spawn_sh().0.shutdown();
    assert_eq!(
        open_descriptors(),
        before,
        "a reaped PTY must drop its leader and signal descriptors, not leak them"
    );
}

// ── R12 — the child gets the extra environment the options carry ────────────
#[test]
fn spawn_passes_the_given_environment() {
    let _serialized = serialized();
    let dir = tempfile::tempdir().expect("tempdir");
    let options = SessionOptions {
        shell: PathBuf::from("/bin/sh"),
        args: Vec::new(),
        cwd: dir.path().to_path_buf(),
        env: vec![(
            "MARLEY_PTY_PROBE".to_string(),
            "marley-env-probe-value".to_string(),
        )],
        cols: 120,
        rows: 24,
    };
    let mut session = TerminalSession::spawn(options).expect("spawn /bin/sh on a PTY");
    // Print the value through a second variable so the echoed command line never contains it.
    session
        .write_command("probe=$MARLEY_PTY_PROBE; echo \"seen:$probe\"")
        .expect("write echo");
    assert!(
        pump_until_screen_shows(
            &mut session,
            "seen:marley-env-probe-value",
            Duration::from_secs(5)
        ),
        "the child must see MARLEY_PTY_PROBE; the screen showed:\n{}",
        screen_text(&session)
    );
    session.shutdown();
}
