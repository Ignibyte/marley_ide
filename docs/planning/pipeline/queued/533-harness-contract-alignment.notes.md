# The harness's contract requests answered (tool names, retry ids), and the plan's harness text corrected — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-533-harness-contract-alignment.md
- **Pipeline spec:** 533-harness-contract-alignment.spec.md

## Phase 1 — Plan
- **Request:** the lead's brief for 2026-09-25: the harness's requests to Marley
  (`/srv/stacks/rustal-harness/docs/planning/MARLEY_REQUESTS.md`): MREQ-001, tool names Claude
  clients can call (check what `crates/marley_mcp` serves today and whether `crates/marley_fleet`'s
  docs still spell verbs with dots), and MREQ-002, an optional delivery id on `SendRequest`
  (`crates/marley_fleet/src/verbs.rs`), returned in the receipt; while this plan was written the
  harness finished `session_open` (TICKET-055) and widened MREQ-002 to "retry keys on
  `SendRequest` and `OpenRequest`", so this ticket takes both. And the plan text:
  `docs/marley/three-prong-plan.md` says the harness has been paused since 2026-09-14, which is
  wrong (the harness's `docs/STATUS.md`, `docs/ROADMAP.md` M9 to M11), and must say that the
  harness will be embedded in Marley, run by Marley as its own process over the same protocol as a
  standalone harness on other hosts, with a standalone version outside it. Chad's decision behind
  it, as the brief records it: rustal-harness will be embedded in Marley and also run standalone,
  and its `rh mcp` serves Marley's fleet contract.
- **Classification / tier:** chore, prong 2, S. Rust in `marley_fleet` only (two fields, two
  types, doc comments, the test literals); the rest is documents. No Zed path.
- **Recall (§18.3):**
  - AD-claude-491-marleys-mcp-server-runs-in-the-app-behind-a-stdio-bridge-001: "Wire names are
    `family_verb`". MREQ-001's premise predates it.
  - `docs/marley_architecture/marley_mcp.md` (the core): `tool_name` joins with `_` since #491, and
    `tools/list` lists the served families (`terminal`, `browser`) while `fleet` and `session` wait
    for C1, still reachable by name.
  - `docs/marley_architecture/orchestration-shell.md` (the gpui-era record, amended 2026-08-09): its
    Layer 2 notes (304 to 324) already name the missing per-dispatch correlation id ("Per-dispatch
    correlation ids stay a later contract rev"). MREQ-002 is that id, asked for by the substrate
    that now serves the contract.
  - Brain: no consultation run by this drafting agent; the Planner's `brain_ask` at promotion is
    owed.
- **Discovery (checked in the tree, the harness and the gpui-era repository):**
  - `crates/marley_mcp/src/registry.rs`: `Family` (12) and `is_served` (38 to 41); `ToolSpec::name`
    (62) and `tool_name` (70, `format!("{}_{}", ...)`, its doc naming Claude Code's and the API's
    rule); `REGISTRY` (78) with the `fleet` row (79 to 85) and the `session` row (86 to 92);
    `lookup` (247) by wire name; `tools_list` (260) over served families only.
  - `crates/marley_mcp/src/dispatch.rs`: the `Fleet` and `Session` arms (159 to 165),
    `surface_to_human` (179), which parses `marley_fleet::SurfaceRequest`; the tests call
    `fleet_snapshot` and `session_surface_to_human` by those names (285, 318).
  - `crates/marley_mcp/src/permission.rs:15` and `config.rs:185`: grant classes such as
    `session.write`, a separate vocabulary.
  - `crates/marley_fleet/src/verbs.rs`: `SendRequest` (9 to 14, `id` and `text`), `OpenRequest`
    (47 to 50, one opaque `profile`) and the dotted verb
    names in the docs at 6 (`session.send`), 16 and 35 (`session.read`), 44 (`session.open`), 52
    (`session.surface_to_human`) and 60 (`session.answer`); `Receipt<T>` (81); the round-trip
    test's `SendRequest` and `OpenRequest` literals (110, 122), the only constructions in the tree.
  - `crates/marley_fleet/src/marley_fleet.rs:38-44`, the exports; `dispatch.rs`, `DeliveryState`
    (`deposited`, `claimed`, `started`, lowercase on the wire).
  - `docs/marley/three-prong-plan.md`: the cast table's session-substrate row (149), the paragraph
    on the harness's native client and its pause (161 to 164), D8 (173 to 178), D9 (180 to 185, the
    dotted families), D10 (187 to 192), the C1 and C4 rows (204, 207), the risk (212 to 213), the
    open decisions (391 to 411, the last is 6).
  - `docs/orca_architecture/README.md` ("Two corrections to what the agents were told"),
    `docs/orca_architecture/06-cli-automations-skills.md:832` ("rustal-harness itself, paused since
    2026-09-14?"), `docs/orca_architecture/07-engineering-and-changelog.md:1849` ("Harness protocol
    v1 is paused upstream").
  - rustal-harness (read, not built): `docs/planning/MARLEY_REQUESTS.md` (both requests);
    `docs/MCP.md` (the tools, `session_send`'s optional `delivery`, the receipt, "Tool names use
    underscores"); `docs/FLEET.md` (the envelope and its pin); `docs/STATUS.md` (read twice this
    evening; the second time: TICKET-048 complete 2026-09-24, TICKET-049 to TICKET-055 complete
    2026-09-25, TICKET-056 `session_surface` at Plan; "Resumed;
    bootstrap focused checks pass — 2026-09-22", D95, after "Paused during bootstrap integration —
    2026-09-14"); `docs/ROADMAP.md` (M9, its exit and its dependency rule; M10; M11; D107 in
    `docs/DECISIONS.md`); `docs/MCP.md`'s "Running it" (`rh --state ROOT mcp`, stdio, one client,
    and remote clients running the same command over SSH under an OpenSSH `ForceCommand`);
    `dependencies/marley.json` (`source_repository: /srv/stacks/marley`, commit
    `0ba2872d8f53e666d7a8596fa63afe353cb5800d`); `crates/harness-conformance/src/main.rs` (116 to 124,
    the `delivery` strip; the exact round-trip check, 127 to 150); `crates/harness-runtime/src/mcp.rs`
    (the tools, 245 to 296; the send's value, 553; `Found.detail`, the runtime's own state; the
    open's value, 748); `docs/planning/pipeline/active/session-surface.notes.md` (TICKET-056, whose
    verb is `session_surface`, where Marley's is `session_surface_to_human`).
  - `/srv/stacks/marley` at `0ba2872d8f` (2026-08-15, TICKET-435): `crates/marley_mcp/src/registry.rs`
    line 53, `tool_name` joining with `.`, and its test naming `fleet.snapshot`. `diff` of that
    commit's `verbs.rs` (the harness's `.deps` export) against the fork's: formatting only.
- **Decisions:** D1 to D6 in the spec.

### Design
- **Approach.**
  - `crates/marley_fleet/src/verbs.rs`: `SendRequest.delivery: Option<String>` and
    `OpenRequest.request: Option<String>`, each with
    `#[serde(default, skip_serializing_if = "Option::is_none")]` and a doc line (a client-chosen id
    the substrate acts on once; a UUID by convention); `pub struct SendReceipt { pub id: String,
    pub delivery: String, pub state: DeliveryState, pub detail: String }` and
    `pub struct OpenReceipt { pub id: String, pub title: String, pub profile: String, pub request:
    String }`, both with `Debug, Clone, PartialEq, Eq, Serialize, Deserialize` and a doc for each
    field (`detail` opaque); the module and item docs name the verbs `session_send`,
    `session_read`, `session_open`, `session_surface_to_human`, `session_answer`; the round-trip
    test's literals gain `delivery: None` and `request: None` (the tree's tests keep building, §7).
  - `crates/marley_fleet/src/marley_fleet.rs`: `SendReceipt` and `OpenReceipt` in the `verbs`
    re-export, and the crate doc's list of modules unchanged in meaning.
  - `docs/marley/three-prong-plan.md`, prong 2: the cast row reads that Marley reaches the harness
    over `rh mcp` (the fleet contract, MCP on stdio: the envelope as a resource, its changes by
    cursor, the session verbs under a write grant) and over its socket protocol later for managed
    input (C3). The paused paragraph becomes the status, re-read from the harness's `STATUS.md` at
    P2: the owner lifted the pause on 2026-09-22, M8 is complete, and M9 began on 2026-09-24 (on the
    evening of 2026-09-25: the envelope and its durable feed, answer, send, read and open served
    over `rh mcp`, Marley's own `marley_fleet` checking them; surface in progress). The
    risk becomes: the harness moves fast, so Marley reads what it consumes from the harness's
    documents at a named state and answers its requests in its own tickets. D8's `marley_harness`
    line names `rh mcp` as the first transport. D9 uses the wire names. C1's row names #534 and
    `rh mcp`; C4's names `session_send` and `session_answer`. D19 (or the next free number) states
    the embedded and standalone harness (spec D5). A list, "The harness's requests": MREQ-001
    (answered: `family_verb` since #491, the prose fixed in #533; the pin at `/srv/stacks/marley` is
    the gpui era's), MREQ-002 (answered in #533: `SendRequest.delivery` with `SendReceipt`,
    `OpenRequest.request` with `OpenReceipt`), the surface verb's two names (Marley's
    `session_surface_to_human`, the harness's `session_surface`, TICKET-056), and that the
    contract's home is the fork's `crates/marley_fleet` and `crates/marley_mcp`.
  - `docs/marley_architecture/orchestration-shell.md`: one line in the amendment banner.
  - `docs/orca_architecture/README.md`: the corrections paragraph names the harness's status;
    reports 06 and 07 get the corrected wording at the two lines.
- **File manifest.** Marley crate: `crates/marley_fleet/src/verbs.rs`,
  `crates/marley_fleet/src/marley_fleet.rs`. Documents (Marley-owned): `docs/marley/three-prong-plan.md`,
  `docs/marley_architecture/orchestration-shell.md`, `docs/orca_architecture/README.md`,
  `docs/orca_architecture/06-cli-automations-skills.md`,
  `docs/orca_architecture/07-engineering-and-changelog.md`. Test phase:
  `script/e2e/533-harness-contract-alignment.sh`. No Zed path, so no touchpoint row.
- **Ledger rows:** none (no path outside the Marley-owned set).

### E2E plan and checks
| REQ | Scenario part or check | Shot or log |
|---|---|---|
| REQ-001 | the scenario: #491's stand-in client (`mcp-client.py`, through the plugin's `bin/marley-mcp-bridge`) as a shell function in the scenario's `.bashrc`; `mcp names` prints every listed tool with `ok` or `bad` against the pattern | `533-01-names` |
| REQ-002 | the same terminal: `mcp call fleet_snapshot` and `mcp call session_surface_to_human '{"id":"none"}'` print the answers (an empty fleet; a refused receipt), neither an unknown-tool error | `533-01-names` |
| REQ-003 | the negative smoke, run before and after P2: `rg -n 'session\.(send\|read\|open\|answer\|surface_to_human)\|fleet\.snapshot\|terminal\.(blocks\|read\|run)\|editor\.(open\|goto\|diff)' crates/marley_fleet docs/marley/three-prong-plan.md` (unescape the pipes) | the notes' Phase 2 and 3 entries |
| REQ-004 | review of the serde attributes on both fields; the round-trip test literals build under gate:2 | the Phase 2 entry |
| REQ-005 | review of both types against the harness's `mcp.rs:553` and `:748` and MCP.md's tools table | the Phase 2 entry |
| REQ-006 | review of the plan's diff; `rg -n -i 'paus' docs/marley/three-prong-plan.md` names no harness pause | the Phase 2 entry |
| REQ-007 | review of the plan's list of the harness's requests | the Phase 2 entry |

Not reachable by a scenario: Marley's server serves no `session_send` or `session_open` (C4), so
the new fields and types have no caller in the app yet; the harness's conformance suite is what
exercises them, on its side, once it pins the fork.

### Risks
- The harness pins a Marley commit and exports the crate from git objects, so it sees this change
  only when it moves its pin, and its pin names the gpui-era repository. The plan's list says where
  the contract lives now; changing the pin is the harness's call.
- Another ticket drafted the same night may take D19 in the plan; P2 takes the next free number.
- The harness's requests moved twice while this plan was written (MREQ-002 widened to
  `OpenRequest` after TICKET-055). P2 re-reads `MARLEY_REQUESTS.md`; a request filed after P1 that
  has the same shape (a field and a receipt value) joins this ticket with Chad's word, and any other
  gets its own.
- The surface verb's two names may become a third request. D7 keeps Marley's name, which its
  server has answered since #370, and records the difference; nothing in the app calls the verb
  yet, so either side can still move at no cost, and that choice waits for the harness.
