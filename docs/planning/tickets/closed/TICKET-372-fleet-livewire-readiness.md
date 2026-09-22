# TICKET-372 — Fleet L0 live-wire READINESS: fixture forge server + real-socket pump proof + typed disconnect/404 + the going-live runbook

- **Forge ticket:** #372 23b95391-d4b3-4739-9581-bb6fb39b4f89 (chore, M23.5)
- **Owner:** unclaimed
- **AAR:** pending-promotion
- **Pipeline doc:** ../../pipeline/queued/372-fleet-livewire-readiness.spec.md
- **Source ticket:** forge #372 (sprint #34 "M23.5 — Fleet Layer-1 Consolidation"; the named #368 Phase-5 follow-up — pipeline/completed/368-forge-client-fleet-subscription.notes.md:378)
- **Status:** closed

## Summary
RESPEC'd from "live-wire verification against real `seat_events`" — Layer 0 (the ucsosv2 seat-emitter
+ the `fleet://events` resource on the real Forge server) is an external deliverable that does not
exist yet, so the literal verification run is un-runnable indefinitely. #372 becomes the LIVE-WIRE
READINESS slice, completable now: (1) a test-only loopback fixture forge server
(`std::net::TcpListener` + a thread, deterministic, no async runtime) speaking exactly the dialect
the masked pump speaks, replaying the frozen #368 fixture contract; (2) an end-to-end integration
test driving the REAL `FleetSubscription` pump over a REAL socket — lifecycle order, session-id
echo, catch-up-read-before-subscribe, SSE events split across ≥3 TCP segments, snapshot equality
with the pure-parser path, and a clean typed end on a mid-stream close — converting the #368
"accepted-untestable" pump into integration-tested behavior; (3) the missing typed half of session
handling — the `Mcp-Session-Id` echo already shipped in #368, so only the typed 404-session signal
(pure classifier + typed pump exit reason) lands here, with the recovery loop explicitly #373's;
(4) a named "going live" runbook subsection in docs/marley_architecture/orchestration-shell.md so
L0-day is config + checklist. The literal real-ucsosv2 run is deferred to a tiny ticket minted when
L0 lands.

## Acceptance
The real masked pump, pointed at the in-test fixture server over a real loopback socket, completes
initialize → initialized → cursor'd read → subscribe → SSE-notified re-reads and yields a
`FleetSnapshot` equal to the pure fixture path's; a mid-stream close and a 404-session response each
surface a TYPED reason with a clean thread end (no panic, no retry); the runbook subsection exists.
Full EARS table in the pipeline spec.
