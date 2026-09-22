# TICKET-6 — TICKET-004b — marley_command Windows parity (R6/R7) [BLOCKED on Windows CI runner]

- **Ticket:** LOCAL #6 (feature, M0)
- **Tags:** M0, marley_command, windows, blocked, needs-windows-runner
- **Created:** 2026-06-28
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id 229e12c2-e25a-49a1-b1e7-1d8f6abd1b56)
- **Status:** closed (2026-08-14 — won't-prioritize, owner decision)

## Description

Follow-up to TICKET-004 (which ships marley_command Unix-only). Implements the deferred Windows half of SPEC-process-command:
- R6: CREATE_NO_WINDOW creation flag on every spawned child (Windows).
- R7: kill_on_parent_process_close(bool) → JobObject that terminates the child when the parent handle closes (Windows).
- The #[cfg(windows)] windows module + win32job/windows deps.

## Closed — 2026-08-14 (won't-prioritize)

Owner decision (Chad, /goal 2026-08-14): Windows parity is not being prioritized
anytime soon and the blocker — a Windows CI runner — has no ETA. Closing rather
than letting the row sit in Deliberate indefinitely. Nothing is lost: the
deferred scope (R6 CREATE_NO_WINDOW, R7 JobObject kill-on-parent-close, the
`#[cfg(windows)]` module + win32job/windows deps) stays documented here and in
SPEC-process-command's ACCEPTED-UNTESTABLE clause. If a Windows runner ever
lands, mint a NEW ticket (numbering is never reused) and lift this scope into it.

BLOCKED: requires a Windows CI runner. On a macOS-only runner, cargo-mutants generates mutants for #[cfg(windows)] code that are reported MISSED (the code is cfg-compiled-out → mutation is a no-op → survives) → MSI < 100, and §0 bars exclusions. Spike-confirmed (cfgprobe): both inline cfg(windows) fns AND separate-file cfg(windows) modules generate surviving mutants on macOS. The spec's own ACCEPTED-UNTESTABLE clause already states the Windows kills are "required on the Windows runner job" — so this ticket lands when that runner exists. Until then, marley_command (Unix-only, TICKET-004) is the shipped surface; consumers spawning non-PTY children get the portable builder + WSL detection, just without the Windows console-flash suppression.
