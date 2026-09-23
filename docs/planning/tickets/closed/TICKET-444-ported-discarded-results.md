# TICKET-444 — Handle the results the ported Marley crates discard

- **Ticket:** LOCAL #444 (chore, port hygiene)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/447-rustal-quality-gates.spec.md (folded into #447)
- **Source ticket:** the #443 inspect ledger (`../../pipeline/completed/443-land-the-port.notes.md`)
- **Status:** closed

## Summary
Zed's `.rules` forbid `let _ =` on a fallible call, and the code ported from the gpui era has
eight of them. `marley_mcp/src/transport.rs` drops a failed connection-thread spawn (117), a
connection's IO error (120) and a failed effect send (202). `marley_terminal` drops
`apply_hook` errors for its synthetic hooks (`apply.rs:198`, `:205`, `:226`) and for decoded
hooks (`session.rs:562`), and the result of the reap signal (`pty_os.rs:118`, where ESRCH is
the expected failure). Most of these failures are harmless, and that is the reason to decide
each one in the open. The #443 inspect found them and left them alone because each fix adds a
branch that needs a test, and the ticket's job was to land the port as it stood. Line numbers
are as of the #443 commit.

## Acceptance
No `let _ =` on a fallible call remains in `crates/marley_*`. Each former discard propagates
its error, logs it with context, or matches the one expected failure (ESRCH for the reap
signal) and says why in a comment. Every new branch is covered and its mutants are caught
under `script/gates.sh --diff`.

## Resolution
Closed by TICKET-447 on 2026-09-22, which added `let_underscore_must_use = "deny"` to the Marley
lint table and so made all eight discards compile errors. The transport logs a failed connection
thread (warn), a connection's IO error and a focus effect the app can no longer take (debug). The
pump logs a hook dropped for arriving before `InitShell` (debug), and a new pump test covers it.
The test seed returns its hook errors with `?`. The reap signal logs any failure except ESRCH.
The mutation clause of the acceptance lapsed with the 2026-09-22 workflow change: mutation now
runs at the end of a sprint (`script/mutation.sh`), and every new branch is covered or sits in a
file gate:4 excludes.
