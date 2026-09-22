# TICKET-379 — marley_mcp: session TTL sweep — idle-session expiry (#375 hardening follow-up)

- **Forge ticket:** #379 (c5b91d70-f42e-4caf-9e47-9d9a7de7f80d) (chore, M24)
- **Owner:** unclaimed
- **AAR:** pending-promotion
- **Pipeline doc:** ../../pipeline/queued/379-mcp-session-ttl.spec.md
- **Source ticket:** sprint #35 "M24 — Fleet Layer 2"; hardens #375's bounded session registry
- **Status:** closed

## Summary
#375 shipped the bounded `Mcp-Session-Id` registry (cap 8, reject-new-never-evict) WITH stream-drop
reclamation (`reap_session` on SSE hang-up, transport.rs:221-228) — but a client that `initialize`s,
never opens a GET stream, and never DELETEs has NO reclamation path: its slot lingers until app
restart, and 8 such ghosts refuse every new manager (503). Add idle expiry, strictly additive to
#375: a **last-seen** stamp on assign refreshed on every successful validate (injected `now: u64` —
the `marley_fleet::attention` idiom, no clock in pure code), a **lazy sweep** of sessions idle ≥
`SESSION_TTL_MS` (proposed 30 min) run pre-dispatch inside `serve_connection`'s existing
session-gate lock (post-auth, NO background timers), and expired ids answered by the ALREADY-SHIPPED
`Reject(404)` arms (session.rs:106/:115) — expired is indistinguishable from unknown at the wire,
and the shipped client half already recovers (404 → `SubscriptionExit::SessionExpired`,
fleet.rs:141 → #373 re-initializes). The `session_decision` table gains no arm; #375 test
expectations are unchanged (mechanical `now`-threading only, inventoried in the pipeline notes).

## Acceptance
A session idle ≥ TTL is removed at the next pre-dispatch sweep and a request bearing its id gets
the shipped 404 (client re-initializes); any validated session-bearing request refreshes its
last-seen; a full-of-idle registry admits a new initialize after the sweep it itself triggers (the
wedge, closed); every #375 invariant holds unchanged (cap-8 reject-new for LIVE sessions, `ct_eq`,
redacted Debug, stream-drop reclamation, DELETE terminate, guard order with the sweep post-auth);
pure registry logic carries an injected `now` (no `SystemTime` outside the masked transport) at
cov/MSI 100. Full EARS criteria live in the pipeline spec
(../../pipeline/queued/379-mcp-session-ttl.spec.md).
