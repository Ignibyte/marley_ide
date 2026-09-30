---
pipeline_id: 43f2e489-605f-4f9d-81c5-964939533de6
ticket: docs/planning/tickets/closed/TICKET-597-harness-verb-payloads-and-capabilities.md
status: Phase 4 — Complete PASS
title: "The harness's verb payloads and capabilities in the contract crate"
type: chore
slice: prong 2, the fleet contract before C1 and C4 (the harness's MREQ-003 and MREQ-004)
references: [docs/planning/pipeline/completed/533-harness-contract-alignment.spec.md, docs/marley_architecture/fleet-control-plane.md]
---

## Title
`marley_fleet`, the contract crate Marley and rustal-harness share, gains the values the
remaining verbs return and a session's capabilities. MREQ-003: `SurfaceAck` moves out of Marley's
server crate into `marley_fleet`, beside two new values, `ReadReceipt` (a read's lines and range)
and `AnswerReceipt` (an answer's choice), with the fields `rh mcp` returns, so a server and an
adapter check the same types. MREQ-004: `Session` gains a `capabilities` map and `SendRequest`
an optional `requires`, so an adapter reads and checks a seat's mode, model and effort without
the harness's `capability.NAME` labels.

## Scope
### In
- **`SurfaceAck { surfaced }`** moves verbatim from `marley_mcp::tools` to `marley_fleet::verbs`,
  re-exported at the crate root. `marley_mcp` builds its `session_surface_to_human` receipt from
  the moved type, and stops re-exporting a copy of its own (one owner, §14).
- **`ReadReceipt { id, start, end, total, lines, gaps }`**: the value of an accepted
  `session_read`, as `rh mcp` returns it (`harness-runtime/src/mcp.rs:1168`): the seat, the
  half-open `[start, end)` returned, the `total` of retained lines, the `lines`, and the count of
  `gaps` in the archive. Numbers are `u64`, as `ReadRange` counts lines.
- **`AnswerReceipt { id, choice }`**: the value of an accepted `session_answer`, as `rh mcp`
  returns it (`mcp.rs:572`).
- **`Capabilities`**, a name-to-value map (`BTreeMap<String, String>`, the harness's own shape,
  `harness-runtime/src/capabilities.rs:17`), with the names `fleet-control-plane.md` §6 designs
  (`mode`, `model`, `effort`) in its doc. `Session.capabilities` and `SendRequest.requires` are
  that map, defaulted when missing and left out of the JSON when empty, so every envelope and
  send in flight today keeps its bytes.
- **`fleet_snapshot`'s output schema** in `marley_mcp::registry` lists `capabilities` on a seat,
  as its comment requires (kept in step with `Session`).
- The docs that list MREQ-003 and MREQ-004 as open (`three-prong-plan.md`, `marley_fleet.md`)
  record them answered, at Complete.

### Out (explicitly deferred)
- The harness's `views` on a surface value (`mcp.rs:917`): the commands a person runs to watch a
  seat are the harness's own vocabulary; `SurfaceAck` names the seat, and a substrate's extra
  fields are ignored on reading, as `OpenReceipt` ignores `supervise` (#533). The harness's exact
  round-trip check is its call.
- Checking `requires` against `capabilities` (the harness's D129 refusal) and refusing dispatch to
  a seat whose mode is wrong (`fleet-control-plane.md` §6): Marley serves no `session_send` yet
  (C4), which is where that check lands.
- Capabilities on `SessionEvent::Upsert`: Marley's own seats come from Claude Code's hook events,
  which declare none; an event that carries them comes with the first adapter that sends them.
- Validating names and values (the harness's bounds, `capabilities.rs:21`): the substrate that
  declares them validates them; Marley reads.
- Re-pinning the harness's conformance to the fork: the harness's own work (#533's D6).

## Reference (§20)
N/A — Marley-specific: the contract between Marley and rustal-harness, Ignibyte's own two
programs. The reference is the other side of the seam, read as data: rustal-harness's
`docs/planning/MARLEY_REQUESTS.md` (MREQ-003 and MREQ-004), its `rh mcp` values
(`crates/harness-runtime/src/mcp.rs` 572, 917, 1168) and its capability map
(`crates/harness-runtime/src/capabilities.rs`), and Marley's own design,
`docs/marley_architecture/fleet-control-plane.md` §6 (a seat's `capabilities`: `mode`, `model`,
`effort`). No Warp or Zed behavior is involved.

### Prior art
- **Behavior maps.** `docs/marley_architecture/fleet-control-plane.md` §2 (the dead-stall row:
  "seat capabilities/mode must be declared state") and §6 (the event contract's `capabilities`).
  `docs/warp_architecture/`, `docs/zed_architecture/` and `docs/orca_architecture/` have no
  fleet contract.
- **Published material.** MCP's structured tool output (`outputSchema` beside
  `structuredContent`), which `fleet_snapshot`'s schema follows; the harness's `docs/MCP.md`
  tool table and "Reading output" section.
- **Code we already ship.** `marley_fleet` itself owns the seam: #533 added `SendReceipt` and
  `OpenReceipt` in `verbs.rs` with the same derives and the optional-and-skipped convention
  (`delivery`, `request`), and `Session.labels` is already a `BTreeMap<String, String>` defaulted
  and skipped when empty, the pattern `capabilities` takes. `marley_mcp::tools::SurfaceAck`
  (79) is the type that moves. No new dependency: `marley_fleet` stays serde-only.

## UI proof
`script/e2e/597-harness-verb-payloads-and-capabilities.sh` (`compositor sway`, for the click
into the terminal). A stand-in `claude` in the project's terminal runs the plugin's real hook
with one `UserPromptSubmit`, so Marley's fleet holds a seat; then a stand-in MCP client, through
the plugin's bridge as Claude Code runs it, prints the snapshot and a `session_surface_to_human`
call (`597-01-contract`). The checks read the same calls from the runner: the seat's JSON has no
`capabilities` key, and the surface verb answers by its name, its refusal without a
`session.write` grant. Not reachable in the running app: an accepted surface (no grant, and no
surface index), and `fleet_snapshot`'s output schema, since `tools/list` lists only the served
families (`registry.rs:44`).

## Locked-In Decisions
- D1: `SurfaceAck` moves verbatim (name, field, derives), so its JSON is the bytes it was; the
  harness already names its seat `surfaced` for it (its D125).
- D2: The receipt values are named as #533's are, `<Verb>Receipt` (`ReadReceipt`,
  `AnswerReceipt`), with `SurfaceAck` keeping its name, which MREQ-003 and the harness use.
- D3: `Capabilities` is a type alias for `BTreeMap<String, String>`, the harness's own shape: any
  names round-trip, and the three the design names are documented, not enumerated. The crate
  carries no product vocabulary.
- D4: Empty means unset: `capabilities` and `requires` default when missing and are skipped when
  empty, like `labels`, so no envelope or send changes bytes until an adapter declares something.
- D5: The placeholder seat the reducer makes carries no capabilities; no event sets them yet.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `marley_fleet` shall export `SurfaceAck { surfaced }`, and Marley's server shall answer `session_surface_to_human` with a receipt built from it, with no copy of the type left in `marley_mcp`. | Review of the diff (the struct moved unchanged); shot `597-01-contract` and the run's check (the verb answers by its name, refused without a grant) |
| REQ-002 | `marley_fleet` shall export `ReadReceipt` with `id`, `start`, `end`, `total`, `lines` and `gaps`, the fields and JSON names `rh mcp`'s `session_read` value has. | Review against `harness-runtime/src/mcp.rs:1168` |
| REQ-003 | `marley_fleet` shall export `AnswerReceipt` with `id` and `choice`, the fields `rh mcp`'s `session_answer` value has. | Review against `mcp.rs:572` |
| REQ-004 | `Session` shall carry a `capabilities` map of names to values that is filled from the JSON when present and left out of the JSON when empty. | Review of the serde attributes; the run's check (a seat's JSON holds no `capabilities` key) |
| REQ-005 | `SendRequest` shall carry an optional `requires` map of the same shape, left out of the JSON when empty. | Review of the serde attributes |
| REQ-006 | `fleet_snapshot`'s output schema shall list a seat's `capabilities` as an object of string values. | Review of `registry.rs`'s `fleet_snapshot_schema` (the fleet family is not listed by `tools/list`, so no client reads it yet) |

## Phase Plan
- **P1 Plan:** this spec, the design in the notes.
- **P2 Code:** `verbs.rs` (the moved `SurfaceAck`, `ReadReceipt`, `AnswerReceipt`, `requires`);
  `session.rs` (`Capabilities`, `Session.capabilities`); the crate root's re-exports; the
  reducer's placeholder and the tests' literals; `marley_mcp` (`tools.rs` imports the type, the
  root drops its re-export, `registry.rs`'s schema); a review; `script/gates.sh --diff` green.
- **P3 Test:** the scenario, every shot read.
- **P4 Complete:** CHANGELOG; `three-prong-plan.md` (MREQ-003 and MREQ-004 answered);
  `docs/marley_architecture/marley_fleet.md`; the ledger; close, archive, commit.
