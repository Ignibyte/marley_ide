# TICKET-588 — The e2e runner's cleanup after its headless compositor exits

- **Ticket:** LOCAL #588 (chore, the e2e harness)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet
- **Source ticket:** #560's golden run (`../../pipeline/completed/560-conflict-chip.notes.md`, Phase 3), 2026-09-28.
- **Status:** open

## Summary
In #560's golden run the headless sway of `508-approvals-inbox` exited between two steps, with no
core dump, no OOM kill and no GPU fault logged; the pointer helper printed `seat-pointer: the
compositor went away`, and the next `pointer` call failed. The runner's cleanup then stopped on
`SEAT_POINTER_PID: unbound variable` (`script/e2e.sh`, `sway_stop`): the pointer helper is a
`coproc`, bash unsets its `_PID` variable once it reaps it, and the runner runs under `set -u`. The
cleanup ended there, so the run's Marley log was never copied next to its shots. This slice makes
the cleanup finish when the helper is gone (`${SEAT_POINTER_PID:-}`), prints one line naming the
compositor's exit and its status when sway is gone before the scenario ends, and keeps sway's own
log at a level that says why it exited, so the next exit can be explained.

## Acceptance
A run whose headless sway is killed partway prints that the compositor exited, still copies
Marley's log beside the shots, and stops the run's Marley, pointer helper and key holder; `just
regress` reports the scenario as failed with that line.
