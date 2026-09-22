# TICKET-373 — Fleet subscription: auto-reconnect + backoff for the standing-SSE pump (#368 follow-up)

- **Forge ticket:** #373 09c7963a-b3e7-456d-a71c-f8e4da63d123 (feature, M23.5)
- **Owner:** unclaimed
- **AAR:** pending-promotion
- **Pipeline doc:** ../../pipeline/queued/373-fleet-reconnect-backoff.spec.md
- **Source ticket:** forge #373 (sprint #34 "M23.5 — Fleet Layer-1 Consolidation"; origin = a #368 inspect LOW). **Depends on #372** (queued ahead in this same batch): its loopback fixture-Forge harness hosts this ticket's kill-and-reconnect tests, and its typed disconnect surface + typed 404-session signal are what this ticket's reconnect loop consumes.
- **Status:** closed

## Summary
The #368 fleet pump dies silently on any disconnect: the standing-SSE read loop breaks on server-close
(`Ok(0)`) and on any hard mid-stream error (the #368 F6 break-don't-hot-spin fix), and an open failure
returns (`crates/marley_forge_client/src/adapter.rs:208-232`) — correct per-slice, but a transient
Forge bounce then permanently kills live fleet updates until the app re-invokes `start`. No data loss
(the durable cursor at `<config_dir>/fleet-cursor` persists per applied page, so a manual restart
already resumes from `?since=<cursor>`), but the death is silent — exactly the failure mode a
monitoring surface must not have ("Marley down ≠ fleet down", orchestration-shell.md §6). This ticket
adds (1) a PURE backoff policy seam (full-jitter exponential delay with injected jitter fraction +
injected clock — no `rand`, no ambient time; the `marley_fleet::attention` idiom) at cov/MSI 100, and
(2) the masked pump loop: on stream close / hard error / open failure / typed 404-session → wait the
jittered delay (stop-responsive at ≤1s granularity) → re-initialize → replay-from-cursor → subscribe →
listen, retrying FOREVER at the capped interval until the stop flag (until-stopped is LOCKED — a
monitoring surface that gives up silently recreates the bug; the #367/#369 staleness indicator
communicates degradation meanwhile). No gap, no dupe: the #367 reducer is idempotent under overlap
replay (`crates/marley_fleet/src/reducer.rs:138-140`).

## Acceptance
A transient Forge bounce no longer kills live fleet updates: the pump reconnects by itself (jittered
exponential backoff, capped, forever until stopped), and on the #372 harness a kill-after-event-3 run
converges to a final `FleetSnapshot` identical to an unbroken run — no gap, no duplicate. The stop flag
stays honored within ~1s even mid-backoff-wait; the pure backoff policy ships at cov/MSI 100 with the
pump loop staying masked. Full EARS table in the pipeline spec.
