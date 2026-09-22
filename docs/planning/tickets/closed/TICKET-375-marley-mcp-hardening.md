# TICKET-375 — marley_mcp: bearer CSPRNG + discovery-file perms + per-session Mcp-Session-Id (#370 hardening)

- **Forge ticket:** #375 (b20f9b7b-b210-4310-810d-3399eabd7d45) (chore, M23.5)
- **Owner:** unclaimed
- **AAR:** pending-promotion
- **Pipeline doc:** ../../pipeline/queued/375-marley-mcp-hardening.spec.md
- **Source ticket:** sprint "M23.5 — Fleet Layer-1 Consolidation"; hardens #370's loopback MCP server
- **Status:** closed

## Summary
Defensive hardening of Marley's OWN loopback MCP server (#370) for the threat model L1 deliberately
deferred: more than one local client + a hostile local process / multi-user box. Three clusters, all
grounded in the Phase-1 audit of the shipped code: **(A) entropy** — today's bearer is
`derive_bearer(port, salt)` (`transport.rs:73`), a deterministic format over the OS-assigned port
(enumerable by any local process) and an `agents.len()`-derived salt (`app.rs:6586`) — guessable in a
handful of tries; replace with a 128-bit per-start bearer read from `/dev/urandom` via `std::fs`
(zero new deps), refuse-to-start on entropy failure, and a constant-time-ish compare (today
`bearer_ok` is `==`, `auth.rs:42`). **(B) discovery file** — `mcp-endpoint.json` (`{url, bearer}`) is
written with `std::fs::write` (`mcp_host.rs:62`) → 0644 world-readable under the default umask, and
NOTHING ever removes it (no `Drop`, no stop path exists); create/correct to 0600 and remove it on
clean shutdown. **(C) sessions** — the server assigns no `Mcp-Session-Id` at all (one implicit
session; `transport.rs` parses only Origin/Authorization/Content-Length); implement the MCP-spec
session lifecycle: fresh id on initialize, echo required (400 missing / 404 unknown), DELETE
terminates, a BOUNDED registry (cap 8, reject-new-when-full — never evict a live manager's session).
The #370 pre-auth invariants (loopback bind, Origin, bearer-pre-dispatch, the 1MiB body cap,
bearer-never-logged) are PINNED as regressions; session validation slots in AFTER auth, never before.
Soft-ordered after #374 (grant wiring touches `start_mcp_server`'s surface).

## Acceptance
The bearer + every session id come from the OS CSPRNG (pure hex formatter + injected-entropy units;
entropy failure → typed refuse-to-start, never a weaker fallback); the discovery file is provably
0600 on both the fresh-create and pre-existing paths (tempdir metadata asserts) and is removed on
clean shutdown; initialize mints `Mcp-Session-Id`, non-initialize requests without/with-unknown ids
get 400/404, DELETE terminates, the registry caps at 8 with a typed reject-new refusal — all proven
in the #370 pure-unit style at cov/MSI 100, with the existing pre-auth guard order (cap → Origin →
bearer → dispatch) regression-pinned around the new session gate. Full EARS criteria live in the
pipeline spec (../../pipeline/queued/375-marley-mcp-hardening.spec.md).
