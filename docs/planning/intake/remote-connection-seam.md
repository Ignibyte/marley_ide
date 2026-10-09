---
status: superseded
created: 2026-06-28
ticket: unassigned
pipeline_spec: unassigned
note: audit (2026-10-09): the trait-and-stub design was never built; marley_remote (TICKET-83 to TICKET-87, an ssh-command builder) and TICKET-526, TICKET-543, TICKET-610, TICKET-641 made remote live
---

# Remote-connection seam — retained, non-functional (future)

## What
Marley keeps a **transport-agnostic remote-connection seam** in the architecture
from early on, **wired but inert** (a `NotImplemented`/stub transport). It is the
join point a later milestone fills in to "connect remotely to something" — remote
runners, an agent-on-web host, or device handoff (continue a session elsewhere).

Shape (clean-room, from the behavior reference, not Warp source):
- A `RemoteTransport`-style trait (à la Warp's `remote_server`): `connect`,
  `send`/`recv` of framed messages, `is_disconnected`. Concrete transports slot in
  behind it — SSH, `docker exec`, an in-process test transport, and (later) a
  cloud/relay transport.
- It binds to **`marley_util::HostId` + the local-vs-remote path sum type**
  (TICKET-002), which already make the data model remote-aware. The seam adds the
  *connection*, not new identity types.
- M0–M4: the only impl is a stub that returns `RemoteUnavailable` (or similar).
  No network, no auth, no cloud. Nothing in the product graph depends on it being
  live.

## Why
- Chad: "retain that ability to connect remotely to something but not functional —
  we will need that at some point." (M5 roadmap already names *agents-on-web /
  remote runners*.)
- A trait + a stub now is ~free; retrofitting a remote boundary into a codebase
  that assumed everything is local is expensive (Warp's own remote-vs-local path
  split exists for exactly this reason).

## Notes
- **Not** Warp's session-sharing: that is cloud-relayed through
  `sessions.app.warp.dev` + Firebase auth — SKIP bucket (clean-build-plan).
  Marley's eventual remote story is **INVENT** (our own relay/auth/transport,
  streaming our own Block/command model), reusing only the *pattern*:
  transport-agnostic trait + structured session state + authenticated subscription.
- Reference behavior: `docs/warp_architecture/crates/remote_server.md`
  (`RemoteTransport` trait, host/session-scoped RPC) and
  `subsystems/05-cloud-auth-networking.md` §9/§13.
- Likely home: a small `marley_remote` (or a `remote` module in the terminal/
  session crate) introduced around M3 (agent orchestration) or M5 (remote runners).

## Promotion
Not a pipeline doc yet — a candidate. Promote via `/work` when remote runners /
device handoff are scheduled (M3–M5). At that point it becomes a ticket + active
pipeline doc; the stub is replaced by real transports + a Marley auth/bearer seam.
Decision recorded as forge `AD-claude-retain-remote-seam-001`.
