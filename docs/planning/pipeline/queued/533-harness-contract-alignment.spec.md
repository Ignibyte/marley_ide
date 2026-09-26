---
pipeline_id: 7e868a7d-2715-482e-a6ab-1d2b2c7faa36
ticket: docs/planning/tickets/open/TICKET-533-harness-contract-alignment.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "The harness's contract requests answered (tool names, retry ids), and the plan's harness text corrected"
type: chore
slice: prong 2, the fleet contract before C1 (#534)
references: [docs/planning/pipeline/completed/491-marley-mcp-in-the-app.spec.md, docs/marley/three-prong-plan.md]
---

## Title
Marley answers rustal-harness's two contract requests: its server's tool names are the ones a
Claude client can call, and its docs stop spelling them with dots (MREQ-001); `SendRequest` and
`OpenRequest` carry optional retry ids that their receipts return (MREQ-002). The plan's prong 2
stops calling the harness paused and records that the harness will be embedded in Marley and also
run standalone.

## Scope
### In
- **MREQ-001, tool names.** Marley's server keeps its `family_verb` names, which it has served
  since #491 (`registry::tool_name`); the answer is the docs: `crates/marley_fleet/src/verbs.rs`
  names each verb by its wire name (`session_send`, `session_read`, `session_open`,
  `session_surface_to_human`, `session_answer`), the plan's D9 (`fleet_snapshot`, the session
  verbs, `terminal_blocks`, `terminal_read`, `terminal_run`, `editor_open`, `editor_goto`,
  `editor_diff`) and C4 do too, and `docs/marley_architecture/orchestration-shell.md`'s amendment
  banner gains a line saying that the record's dotted verbs are its own spelling and the wire
  names are `family_verb` since #491. The grant classes (`session.write`, `browser.write`) keep
  their dots (D2).
- **MREQ-002, retry ids.** `SendRequest` gains `delivery: Option<String>` and `OpenRequest` gains
  `request: Option<String>`, each a client-chosen id (a UUID by convention) with
  `#[serde(default, skip_serializing_if = "Option::is_none")]`, so a request without one is the JSON
  it was. `marley_fleet` gains the two receipt values that return them, with the fields `rh mcp`
  returns: `SendReceipt { id, delivery, state: DeliveryState, detail }` and
  `OpenReceipt { id, title, profile, request }`, exported beside the other verbs. The tree's
  round-trip test literals gain the new fields as `None`.
- **The plan, prong 2** (`docs/marley/three-prong-plan.md`): the cast table's session-substrate row
  (Marley reaches the harness through `rh mcp`, the fleet contract over MCP, and its socket
  protocol for passthrough later); the paragraph that says the harness is paused (lines 161 to
  164) and the risk that repeats it (212 to 213), replaced with its state as the harness's
  `docs/STATUS.md` gives it when P2 writes (on the evening of 2026-09-25: resumed 2026-09-22, M8
  complete, M9's envelope and feed, answer, send, read and open served, surface in progress); D8's
  `marley_harness` line (its first transport is `rh mcp`); C1's and C4's rows (C1 is #534); a new
  design decision, D19 or the next free number, for the embedded and standalone harness (D5); and a
  short "The harness's requests" list with MREQ-001 and MREQ-002, their answers, the surface verb's
  two names (D7), and where the contract lives now.
- **The Orca survey's copy of the stale claim:** `docs/orca_architecture/README.md` names it among
  its corrections, and the two report lines that repeat it (report 06 §5 question 2, report 07's
  item 5 row) are corrected in place.

### Out (explicitly deferred)
- Listing the `fleet` and `session` families in `tools/list`, and handlers for the session verbs
  in Marley's own server (C1 and C4).
- Re-pinning the harness's conformance to the fork and dropping its field strip: the harness's own
  work, which Marley does not do (each project changes only its own files).
- Renaming Marley's surface verb to the harness's `session_surface`, or the reverse (D7).
- Packaging `rh` with Marley and Marley starting the harness's runtime: D19 records the decision;
  the work is its own ticket after #534.
- Renaming the grant classes.

## Reference (§20)
N/A — Marley-specific: this is the contract between Marley and rustal-harness, Ignibyte's own
programs, and no Warp or Zed behavior bears on it. The rules it follows are published: MREQ-001
quotes the pattern a Claude client's tool names must match, `^[a-zA-Z0-9_-]{1,64}$`, and Marley's
own registry already cites it ("Claude Code and the Anthropic API take tool names without dots",
#491 D4).

### Prior art
- **Behavior maps and reports.** The harness's own documents stand in for a map:
  `docs/planning/MARLEY_REQUESTS.md` (both requests, their evidence and the pinned commit),
  `docs/MCP.md` (the tools table: `fleet_snapshot`, `fleet_events`, `session_stop`,
  `session_answer`, `session_send` with an optional `delivery`, `delivery_status`, `session_read`,
  `session_open` with an optional `request`; "Tool names use underscores, which Claude's tool-name
  rules require"), `docs/FLEET.md` (the envelope, and "The contract is Marley's `marley_fleet` crate
  at commit `0ba2872d8f...`"), `docs/STATUS.md` (TICKET-048 to TICKET-056 on 2026-09-24 and
  2026-09-25; the pause of 2026-09-14 lifted on 2026-09-22, D95), `docs/ROADMAP.md` (M9 to M11,
  D107). The Orca survey's report 06 §2.6 names the verb types as defined and unhandled.
- **Published material.** The tool-name pattern above; the MCP specification's tools (a name is a
  string; the client decides what it accepts).
- **Code we already ship.** `crates/marley_mcp/src/registry.rs`: `tool_name` (70, `family_verb`),
  `is_served` (38, the `terminal` and `browser` families listed, `fleet` and `session` reachable by
  name), the `fleet` and `session` rows (79 to 92); `crates/marley_mcp/src/dispatch.rs` (the
  `Fleet` and `Session` arms, 159 to 165, and `surface_to_human`, 179). `crates/marley_fleet`:
  `verbs.rs` (the dotted verb names at 6, 16, 35, 44, 52 and 60; `SendRequest` at 9,
  `OpenRequest` at 47), `dispatch.rs` (`DeliveryState`), `marley_fleet.rs` (the exports, 38 to 44).
  On the harness's side: `crates/harness-conformance/src/main.rs:116-124` removes `delivery` before
  it checks a send against Marley's type; `crates/harness-runtime/src/mcp.rs` builds the send's
  value `{"id", "delivery", "state", "detail"}` (553) and the open's
  `{"id", "title", "profile", "request"}` (748); `dependencies/marley.json` pins
  `source_repository: /srv/stacks/marley`. At that commit the gpui-era `tool_name` joined with a dot
  (`git -C /srv/stacks/marley show 0ba2872d8f:crates/marley_mcp/src/registry.rs`, line 53); the
  fork's `verbs.rs` differs from it only in formatting. The sweep's win: MREQ-001 is already
  answered by what #491 ships.

## UI proof
N/A — no UI delta: the change is two fields, two types and documents. Its e2e run is
`script/e2e/533-harness-contract-alignment.sh` (the default Hyprland backend, keys only), which
starts Marley, runs #491's stand-in MCP client through the plugin's bridge in a terminal, prints
every listed tool name with whether it matches `^[a-zA-Z0-9_-]{1,64}$`, then calls
`fleet_snapshot` and `session_surface_to_human` by name; shot `533-01-names`.

## Locked-In Decisions
- D1 — MREQ-001 is answered by what the fork serves: `family_verb` names since #491
  (AD-claude-491). The harness saw dots because it pins the gpui-era repository, whose `tool_name`
  joined with `.`; what still spells dots in the fork is prose (the verb docs, the plan), and that
  is what changes. No alias for dotted names: no client can call one.
- D2 — Grant classes keep their dots: `session.write` and `browser.write` are Marley's permission
  vocabulary in settings, never a tool name a client sends.
- D3 — A retry id is optional and invisible when absent: `delivery` on `SendRequest` and `request`
  on `OpenRequest`, both `Option<String>` with `skip_serializing_if`, so every request in flight
  today keeps its bytes, and the harness can check both against Marley's types without removing a
  field. Each id is a string, a UUID by convention; `marley_fleet` depends on serde alone and stays
  that way.
- D4 — The receipts' values are Marley types: `SendReceipt { id, delivery, state: DeliveryState,
  detail }` and `OpenReceipt { id, title, profile, request }`, the value inside an accepted
  `Receipt`. `detail` is the substrate's own word for the delivery (the harness sends the runtime's
  state, such as `failed`), carried as opaque text that Marley shows and never matches, as it treats
  `labels`.
- D5 — The plan's decision (D19 or the next free number): the harness is embedded in Marley and
  also runs standalone (Chad, 2026-09-25). Embedded means Marley runs `rh` as its own process,
  never linked in (§20's program boundary), and speaks to it over `rh mcp`, the protocol a
  standalone harness on another host speaks (over SSH, the harness's documented path); the same
  client serves both. Standalone, the harness keeps its own lifecycle outside Marley.
- D6 — Marley answers the harness in its own documents: the plan's list of the harness's requests,
  and the commit that ships them. It does not edit the harness's repository, as the harness does
  not edit Marley's (its M9 dependency rule).
- D7 — The surface verb keeps Marley's name: Marley has served `session_surface_to_human` since
  #370 and the harness's TICKET-056 names its own `session_surface`. The plan's list records both
  names for the harness to settle; Marley renames nothing in this ticket.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an MCP client lists Marley's tools, every tool name shall match `^[a-zA-Z0-9_-]{1,64}$`. | Shot `533-01-names` |
| REQ-002 | WHEN a client calls `fleet_snapshot` or `session_surface_to_human` by those names, Marley's server shall answer with the fleet or a receipt, never an unknown-tool error. | Shot `533-01-names` |
| REQ-003 | The `marley_fleet` verb docs and Marley's plan shall name every verb by its wire name. | Negative smoke: the notes' `rg` for dotted verb names over `crates/marley_fleet` and the plan prints lines before P2 and nothing after |
| REQ-004 | WHEN a `SendRequest` or an `OpenRequest` carries no retry id, it shall serialize as it did before; WHEN it carries one, it shall carry it as `delivery` or `request`. | Review of the serde attributes; the tree's round-trip tests build under gate:2 |
| REQ-005 | `marley_fleet` shall export `SendReceipt` (`id`, `delivery`, `state` as a `DeliveryState`, `detail`) and `OpenReceipt` (`id`, `title`, `profile`, `request`), the fields of `rh mcp`'s send and open values. | Review against `rustal-harness/crates/harness-runtime/src/mcp.rs:553,748` |
| REQ-006 | The plan shall state the harness's status and the decision that it is embedded in Marley and also runs standalone, and shall not call it paused. | Review; `rg -n -i 'paus' docs/marley/three-prong-plan.md` names no harness pause |
| REQ-007 | The plan shall list MREQ-001 and MREQ-002 with their answers and name the fork's `marley_fleet` and `marley_mcp` as the contract's home. | Review |

## Phase Plan
- **P1 Plan** — this spec; the design and the checks in the notes.
- **P2 Code** — re-read the harness's `docs/STATUS.md` and `MARLEY_REQUESTS.md` first (both moved
  while this plan was written); `verbs.rs` (the fields, the types, the docs, the test literals) and
  the exports; the plan, the orchestration record's banner, the Orca survey's corrections; fmt and
  clippy clean; a review of the diff (§18.1).
- **P3 Test** — write and run `script/e2e/533-harness-contract-alignment.sh`, read the shot; the
  negative smokes of REQ-003 and REQ-006; `script/gates.sh --diff` green (Rust changed).
- **P4 Complete** — CHANGELOG (a Changed entry: the fleet contract's retry ids), the plan's slice
  table, ledger capture, close, archive, commit. The harness reads the answers there.
