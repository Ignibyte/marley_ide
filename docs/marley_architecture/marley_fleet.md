# `marley_fleet`

> Per-crate architecture note, written 2026-09-29 at #533. Provenance: **`[Marley-original]`**
> (`serde` only). The design record is [orchestration-shell.md](./orchestration-shell.md) and
> [fleet-control-plane.md](./fleet-control-plane.md).

The session envelope and the fleet snapshot the control plane stands on, as pure data: no gpui, no
transport, no product vocabulary. The same types are the rail's model, Marley's MCP tool schema
(`crates/marley_mcp`) and the contract rustal-harness's `rh mcp` checks itself against.

## Modules
- `session`: the `Session` envelope and its closed `State` and `Transport` vocabularies, with an
  optional standing `Question`.
- `reducer`: `SessionEvent` and the replay-safe fold `apply` / `reduce` into a `FleetSnapshot`.
- `attention`: read-time staleness (`is_stale`) and the attention order, with `now_ms` passed in.
- `dispatch`: `DeliveryState` (`deposited`, `claimed`, `started`, a monotone join) and
  `DeliveryAdvance`.
- `verbs`: the session verbs' requests and receipts, each named by its MCP tool name
  (`session_send`, `session_read`, `session_open`, `session_surface_to_human`, `session_answer`):
  `SendRequest`, `ReadRequest` with `ReadRange`, `OpenRequest`, `SurfaceRequest`, `AnswerRequest`,
  and `Receipt<T>`, accepted with the verb's value or refused with a reason.

## The harness's requests (#533)
- Tool names are `family_verb` (#491), the form a Claude client can call (MREQ-001).
- `SendRequest.delivery` and `OpenRequest.request` are optional retry ids, a UUID by convention,
  left out of the JSON when absent, so a request without one is the bytes it always was
  (MREQ-002). `SendReceipt { id, delivery, state, detail }` and `OpenReceipt { id, title, profile,
  request }` are the values an accepted send and open return, as `rh mcp` returns them; `detail` is
  the substrate's own word, shown and never matched, and fields a substrate adds beyond these are
  ignored on reading.
- Each verb's payload for the rest (MREQ-003) and a session's capabilities (MREQ-004) are #597.

Marley's own server serves no `session_send` or `session_open` yet (C4); these types have their
first reader on the harness's side.
