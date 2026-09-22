# TICKET-348 — Bounded PTY teardown + the nextest kill-switch: no test may ever hang a gate again

- **Forge ticket:** #348 `477c649a-a702-493b-94f2-9a2a516dbce8` (bug, M22)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `2a241d0b-0f77-4f1c-8076-d077336d88c0`
- **Pipeline doc:** ../../pipeline/active/348-pty-teardown-bound.spec.md
- **Source ticket:** the M22 integrity-five shelf — ../../design-notes/integrity-five-shelf.md
- **Status:** closed

## Summary

A gate started at 20:41 was still "running" 11 hours later, wedged on ONE test —
`marley_terminal::integration resize_real_pty_succeeds`, SLOW >39,240s at **0.0% CPU** under
`cargo llvm-cov nextest` (blocked, not slow), with nextest reporting SLOW forever without ever killing
it. A hung gate that reports nothing is the worst failure mode: no red, no green, just silence. Two
fixes, one systemic and one at the source:

- **(D1) The kill-switch:** a `.config/nextest.toml` terminate ceiling (`slow-timeout = { period = "60s",
  terminate-after = 3 }` → SLOW at 60s, SIGKILL at 180s, terminated test FAILS) covering every nextest
  lane (gate:3, gate:4, gate:15, and #345's `cargo mutants --test-tool=nextest`). No test can stall a
  gate again, whatever the cause.
- **(D2/D3/D5) The source fix:** the session teardown's only unbounded wait is the implicit reap
  delegated to `alacritty_terminal`'s `tty::Pty::Drop` (SIGHUP + a blocking `waitpid`). Replace it with an
  explicit bounded reap — a pure clock-injected `reap_step` step function (HUP → deadline → KILL →
  deadline → give-up-and-log; cov/MSI 100) executed by the `pty_os` shim with real syscalls
  (`mutants::skip`). A child surviving SIGKILL past the second deadline is leaked deliberately with a log
  line (D5 zombie-over-hang) — and alacritty's blocking `Drop` is neutralized on that path so the hang
  cannot return. `resize_real_pty_succeeds` spawns a live `/bin/sh`, resizes, then `session.shutdown()` —
  the exact drop path that hangs today.

## Acceptance

A test still running at the nextest ceiling is terminated and FAILS the run (both `cargo nextest run`
and `cargo llvm-cov nextest`); session teardown with a live child completes within the bounded deadlines;
the reap escalates HUP-first, KILL at the first deadline, give-up at the second (pure truth table, cov/MSI
100); `resize_real_pty_succeeds` stays byte-identical in intent and green; the read/write paths
(O_NONBLOCK + retry budget) are untouched. Full EARS criteria (REQ-001 … REQ-006) live in the pipeline
spec. The real-PTY tests stay under coverage — no exclusion (§0); the bound is the fix.
