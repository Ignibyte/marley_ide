# The harness's verb payloads and capabilities in the contract crate — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-597-harness-verb-payloads-and-capabilities.md
- **Pipeline spec:** 597-harness-verb-payloads-and-capabilities.spec.md

## Phase 1 — Plan
- **Request:** the second of the two unheld tickets Chad asked to finish on 2026-09-29; the top
  of the Queue once #596 closed. The harness filed MREQ-003 and MREQ-004 on 2026-09-26, after
  #533's plan, whose risk rule gave them their own ticket.
- **Classification / tier:** chore, S. `marley_fleet` and `marley_mcp` only; no Zed crate.
- **Pre-flight:** no active pipeline; README marker present; cargo idle; `/mnt/fast` at 90%
  (95G free).
- **Recall (§18.3):**
  - #533 (completed): the contract's conventions: optional fields `Option` with
    `skip_serializing_if`, so every request in flight keeps its bytes (D3); receipt values are
    Marley types named `<Verb>Receipt` with a doc on each field (D4); `marley_fleet` depends on
    serde alone; Marley never edits the harness's repository (D6); the harness reads the crate
    from a pinned export, so it sees a change when it moves its pin.
  - AD-claude-533-the-harness-embedded-and-standalone-001: Marley answers the harness's requests
    in its own tickets and documents.
  - The ledger has nothing else on `SurfaceAck`, capabilities or receipts' values.
  - Brain (consultation 615da2a748b84e9b9c94c30806071ca4): nothing on this seam.
- **Discovery** (an Explore pass during #596, then read):
  - `crates/marley_fleet/src/verbs.rs`: the requests, `SendReceipt`, `OpenReceipt`,
    `Receipt<T>`; the round-trip test's `SendRequest` literal (153). `session.rs`: `Session`
    (52 to 77; `labels` is a `BTreeMap<String, String>` defaulted and skipped when empty, its doc
    naming `capabilities.mode` as a label an adapter writes), test literals at 121 and 163.
    `reducer.rs:154`: the placeholder seat. `marley_fleet.rs:38`: the re-exports.
  - `crates/marley_mcp/src/tools.rs:79`: `SurfaceAck`; `surface_receipt` (92), `surface_result`
    (108); `marley_mcp.rs:79` re-exports it, and nothing outside `marley_mcp` names it.
    `dispatch.rs:210`: the verb. `registry.rs:459`: `fleet_snapshot_schema`, "kept in step with
    `Session`".
  - The app builds no surface index (`ServerData::surface_index` is filled only by tests), so
    `session_surface_to_human` refuses every id in the running app.
  - rustal-harness (`/srv/stacks/rustal-harness`, not a git checkout here):
    `docs/planning/MARLEY_REQUESTS.md` 50 to 85; `crates/harness-runtime/src/mcp.rs`: the read's
    value (1168: `id`, `start`, `end`, `total`, `lines`, `gaps`, all counts `usize`, `gaps` the
    number of gaps in the archive per `docs/MCP.md`), the answer's (572: `id`, `choice`), the
    surface's (917: `surfaced`, `views`), `requires` parsed on a send (676);
    `capabilities.rs`: `pub type Capabilities = BTreeMap<String, String>` with its bounds.
- **Decisions:** D1 to D5 in the spec.

### Design
- **`crates/marley_fleet/src/verbs.rs`** (Marley).
  - `SurfaceAck` moved here verbatim, with its doc naming the verb and saying a substrate's
    extra fields (the harness's `views`) are ignored on reading.
  - `ReadReceipt { id: String, start: u64, end: u64, total: u64, lines: Vec<String>, gaps: u64 }`
    and `AnswerReceipt { id: String, choice: String }`, deriving as `SendReceipt` does, a doc on
    each field.
  - `SendRequest.requires: Capabilities`, `#[serde(default, skip_serializing_if =
    "BTreeMap::is_empty")]`; the test literal gains `requires: Capabilities::new()`.
- **`crates/marley_fleet/src/session.rs`** (Marley). `pub type Capabilities = BTreeMap<String,
  String>` with its doc (the names the design uses, the substrate's names and values);
  `Session.capabilities` after `labels`, defaulted and skipped when empty; the `labels` doc drops
  `capabilities.mode` as a label example; the test literals gain the field.
- **`crates/marley_fleet/src/reducer.rs`** (Marley): the placeholder seat's
  `capabilities: BTreeMap::new()`.
- **`crates/marley_fleet/src/marley_fleet.rs`** (Marley): re-export `Capabilities`,
  `SurfaceAck`, `ReadReceipt`, `AnswerReceipt`; the module list's `verbs` line mentions the
  values.
- **`crates/marley_mcp/src/tools.rs`** (Marley): the struct goes; `use marley_fleet::SurfaceAck`.
  **`marley_mcp.rs`**: `SurfaceAck` leaves the `tools` re-export. **`registry.rs`**: the seat's
  `capabilities` in `fleet_snapshot_schema`.
- **`script/e2e/browser-fixture.sh`**: the stand-in client's `schema <tool>`, a tool's output
  schema as `tools/list` gives it.
- **Ledger rows.** No touchpoint row. At Complete: an AD for capabilities as a name-to-value map,
  empty meaning unset (D3, D4).

### Visual check plan
`script/e2e/597-harness-verb-payloads-and-capabilities.sh`, `compositor sway`.

| REQ | Scenario part | Shot or log |
|---|---|---|
| — | setup: a scratch repository; the scenario's HOME; the stand-in client `mcp` (#533's); a stand-in `claude` that runs the plugin's hook once with a `UserPromptSubmit` and writes its `terminalSequence` to the terminal, then exits | — |
| REQ-004 | click the terminal; type `claude`; settle, so the hook's event reaches Marley's fleet | the run's `mcp_agent tool fleet_snapshot`: one seat, no `capabilities` key |
| REQ-006 | type `clear; mcp schema fleet_snapshot` | `597-01-contract`: the seat's `capabilities` property, an object of strings |
| REQ-001 | type `mcp tool fleet_snapshot; mcp tool session_surface_to_human '{"id": "none"}'` | `597-01-contract`: the seat without `capabilities`; the surface verb's refusal by its name |

Not reached by a scenario: an accepted surface (the app builds no surface index, so the moved
type's accepted JSON is read in the diff: the struct moved unchanged); `ReadReceipt`,
`AnswerReceipt` and `SendRequest.requires` (Marley serves no read, answer or send yet; their
reader is the harness's conformance suite, once it pins the fork).

### Risks
- The harness's conformance parses values with Marley's types and requires them to serialize
  back byte-equal; a harness surface value with `views` will not, by D1's choice. The harness
  owns that check; the requests file records the difference.
- Adding a field to `Session` and `SendRequest` breaks struct literals anywhere they are built;
  the tree has four (the reducer's placeholder, three tests), and gate:2 builds every target.

## Phase 2 — Code
- **Built:**
  - `marley_fleet/src/verbs.rs`: `SurfaceAck` moved in verbatim; `ReadReceipt { id, start, end,
    total, lines, gaps }`; `AnswerReceipt { id, choice }`; `SendRequest.requires: Capabilities`,
    defaulted and skipped when empty; the round-trip test's literal gains `requires`.
  - `session.rs`: `pub type Capabilities = BTreeMap<String, String>`; `Session.capabilities`,
    defaulted and skipped when empty; the `labels` doc no longer names `capabilities.mode` as a
    label; the tests' literals gain the field. `reducer.rs`: the placeholder seat's empty map.
  - `marley_fleet.rs`: re-exports `Capabilities`, `SurfaceAck`, `ReadReceipt`, `AnswerReceipt`.
  - `marley_mcp`: `tools.rs` takes `SurfaceAck` from `marley_fleet` (its serde import went with
    the struct); `marley_mcp.rs` no longer re-exports it; `registry.rs`'s `fleet_snapshot_schema`
    lists `capabilities` as an object of strings.
  - `script/e2e/browser-fixture.sh`: the stand-in client's `schema <tool>`; the scenario
    `script/e2e/597-harness-verb-payloads-and-capabilities.sh`, written now so one gate run covers
    it.
- **Deviations:** none from the design. clippy asked for `Capabilities`' first doc paragraph to be
  shorter; it was split.
- **Review of the diff:**
  - REQ-001: the struct moved with its name, field and derives, so a receipt's JSON is the bytes it
    was; nothing outside `marley_mcp` named the old path.
  - REQ-002 and REQ-003: the fields and their JSON names match `rh mcp`'s values (`mcp.rs:1168`,
    `mcp.rs:572`); the harness's `usize` counts read as `u64`.
  - REQ-004 and REQ-005: `default` with `skip_serializing_if = "BTreeMap::is_empty"`, as `labels`,
    so an envelope or a send without them keeps its bytes.
  - REQ-006: the schema's seat lists `capabilities`, beside `labels`.
  - No UI path, no entity, no Zed path; provenance is Marley's own contract.
- **Gate:** run 1: GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/597-harness-verb-payloads-and-capabilities.sh` under `compositor
  sway`: a stand-in `claude` runs the plugin's real hook (`hooks/event.py`) once with a
  `UserPromptSubmit` and writes its `terminalSequence`, so the fleet holds a seat; the stand-in
  MCP client (#533's `mcp`, through the plugin's bridge) prints the snapshot and a surface call.
- **Run 1 red (the plan):** the stand-in client's new `schema fleet_snapshot` printed `null`:
  `tools/list` lists only the served families (`Family::is_served`, `registry.rs:44`: terminal,
  browser, ports), so `fleet_snapshot`, which `tools/call` still answers, is not listed and its
  output schema reaches no client. The surface call was refused before its handler, by the
  `session.write` grant (deny-by-default), not by the empty surface index. Neither is a defect of
  this change. The `schema` subcommand came out of the shared fixture again (nothing else would
  use it); REQ-006 is verified by review; the surface check asks for a refusal by the verb's name
  and no unknown-tool error.
- **Run 2:** exit 0, every check passes. The shot read:
  - `597-01-contract` (REQ-004, REQ-001): the terminal shows `fleet_snapshot: {"seats": [{"id":
    …, "title": "Claude Code", "state": "done", "labels": {…}, "last_event_ms": …, "transport":
    "local"}]}`, with no `capabilities` key on the seat, which declares none; then
    `session_surface_to_human refused: {"result":"refused","reason":"write tool requires an
    explicit grant for class 'session.write' (deny-by-default)"}`, the verb answered by its name.
    The rail shows the project's terminal and nothing else changed on screen.
- **Focus report:** the run was in its own headless sway; Hyprland had 0 Marley windows before
  and after.
- **Not reached:** an accepted surface (no grant, no surface index in the app), so the moved
  `SurfaceAck`'s accepted JSON is read in the diff, where the struct moved unchanged;
  `fleet_snapshot`'s output schema (the fleet family is not listed); `ReadReceipt`,
  `AnswerReceipt` and `SendRequest.requires` (Marley serves no read, answer or send; their reader
  is the harness's conformance suite once it pins the fork).
- **Gate after the scenario's fix:** GATE GREEN [diff] (run 2).

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Changed); `docs/marley/three-prong-plan.md` (MREQ-003 and
  MREQ-004 answered); `docs/marley_architecture/marley_fleet.md` (the harness's requests, #597).
  No Zed path changed.
- **Knowledge:** `AD-claude-597-capabilities-are-a-name-to-value-map-empty-means-unset-001`,
  `L-claude-597-fleet-tools-answer-but-are-not-listed-001`. No `F-…` block: the Test phase's red
  was the plan's assumption about what the app exposes, not the code. Brain: the decision on
  consultation 615da2a7 (`decisions/marley-fleet-contract-capabilities-are-a-name-to-value-map-empty-means-unset`),
  follow-up by 2026-10-29.
- **Closed** TICKET-597, archived the pair.
