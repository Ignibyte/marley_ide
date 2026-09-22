# TICKET-368 — marley_forge_client: MCP fleet subscription + the seat_events→envelope projection (Adapter #1)

- **Forge ticket:** #368 (9e447782-6ab1-4acb-be0e-318ff34d4455) (feature, M23)
- **Owner:** ca2d1534-e512-4d84-92e8-b2f73b4955c9 (M23 train session)
- **AAR:** d8235b74-e3d9-45ac-90f6-6e8a514cb6bd
- **Pipeline doc:** ../../pipeline/active/368-forge-client-fleet-subscription.spec.md
- **Source ticket:** M23 sprint "M23 — Fleet Control Plane: Layer 1" — item ② of the Layer-1 cut
  (docs/marley_architecture/orchestration-shell.md §12); sibling of #367 (`marley_fleet`)
- **Status:** closed

## Summary
Grow the EXISTING `crates/marley_forge_client` into **Adapter #1** — the only crate allowed to know
Forge's vocabulary (orchestration-shell.md §2/§7). Three parts, one shippable slice: **(a)** MCP
resource subscription to Forge's fleet resources (e.g. `fleet://seats`, `fleet://events`) with
`notifications/resources/updated` handled on a standing Streamable-HTTP connection — push = MCP
subscription is DECIDED; Marley never opens a Postgres connection (no raw `LISTEN`, no bespoke SSE
sidecar). **(b)** Cursor/replay catch-up: on (re)connect, `resources/read` from the last-seen cursor
→ replay into the `marley_fleet` reducer → re-subscribe (`seat_events` is durable; state replays
from cursor; PTY scrollback is a different store, out of scope). **(c)** The pure, table-driven
projection: `seat_events` rows + work-state (UCSOS vocabulary: `at-menu` → Waiting + question
payload, `api-error`/rate-limit → Error with detail in labels, ticket/phase/box → opaque labels) →
`marley_fleet`'s generic `SessionEvent`/`Session` (#367's contract). Cov/MSI 100 on the pure seams.

**Contract-first scope (honest):** Layer 0 — the seat-emitter hook + the `seat_events` table on the
Forge server — is a ucsosv2 (external) deliverable that does NOT exist yet. This ticket lands the
client machinery + the pure projection + a fake-feed integration harness (fixture frames driven
through subscribe/replay). Live-wire verification against real `seat_events` is an explicit OUT — a
named follow-up ticket when L0 ships. The AC is fully satisfiable without ucsosv2.

## Acceptance
Every contracted `seat_events` kind (fleet-control-plane.md §6 v1 enum) projects to the right
generic `SessionEvent`, with unknown/future kinds taking a defined non-crashing degrade; replay from
the persisted cursor is idempotent through the reducer (reconnect resumes, never duplicates); the
subscribe → `notifications/resources/updated` → re-read lifecycle works end-to-end against the
fake feed; the bearer never appears in any log/Debug/error path; Forge-unreachable degrades to a
typed `ForgeError` with no panic. Full EARS criteria live in the pipeline spec.
