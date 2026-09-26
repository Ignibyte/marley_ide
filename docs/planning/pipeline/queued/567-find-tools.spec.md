---
pipeline_id: da0f2056-7b66-4306-9531-3d152c1a8e49
ticket: docs/planning/tickets/open/TICKET-567-find-tools.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "browser_find and terminal_find for agents"
type: feature
slice: prong 2 (the MCP server's tools) with prong 3 (the Browser tab); the Jev note's use 2, on #565's layer
references: [docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/pipeline/queued/565-system-one-layer.spec.md, docs/planning/pipeline/completed/492-browser-tools-for-agents.spec.md, docs/planning/pipeline/completed/491-marley-mcp-in-the-app.spec.md, docs/planning/pipeline/completed/516-secret-redaction-for-agents.spec.md]
---

## Title
Two read tools answer a query in words: `browser_find` gives the ref of the element a page holds
for "the sign in button", and `terminal_find` gives the line of a block's output for "where the
server refused the connection", so an agent reads three candidates instead of a 30,000-character
snapshot or a 2,000-line block. Code matches first and answers alone when it can; #565's layer
ranks what the words leave open, over tagged lines as the options of a choice with `none`, and a
`present` noul says whether there is anything to find. Off by default: while the use is off the
tools are not listed.

## Scope
### In
- **The registry** (`crates/marley_mcp/src/registry.rs`): `browser_find` (`Family::Browser`,
  verb `find`, read tier) and `terminal_find` (`Family::Terminal`, verb `find`, read tier), each
  with a `feature: Option<&'static str>` on its `ToolSpec` (`Some("find")`); `tools_list` takes
  the set of enabled features from the `RequestCtx`, and `dispatch::tools_call` refuses a
  feature tool by name while its feature is off, with the setting to turn on as the reason.
  `marley_workbench::mcp` fills the set from `marley.system_one.uses` (`browser_find`,
  `terminal_find`, each on while its mode is not `off`) and refreshes it when settings change.
- **`browser_find`** (`crates/marley_workbench/src/browser_tools.rs`): arguments `query`
  (required, at most 200 characters), `tab` and `full` as `browser_snapshot` takes them. It
  takes the snapshot the way `take_snapshot` does and keeps the refs in the hub (`set_refs`), so
  a ref it answers is clickable. The answer: `tab`, `query`, `source` (`rules`, `model`, `none`),
  `sure`, `ref` (in `act`, the top candidate when sure), `candidates` (at most three: `ref`,
  `role`, `name`, `probability`), `present` (the noul, when asked), `verify` (in `suggest`),
  `next` (`browser_snapshot` when not sure), `note` (why the model was not asked, when it was
  not), and the same as text.
- **`terminal_find`** (`crates/marley_workbench/src/mcp.rs`): arguments `terminal`, `block` as
  `terminal_read` takes them, and `query`. It reads the block's output as `terminal_read` does
  (masked whole, the tail of 2,000 lines and 256 KiB kept), tags each line `L0001|` on, and
  answers `terminal`, `block`, `query`, `source`, `sure`, `line` (in `act`), `candidates` (at
  most three: `line`, `text`, `probability`), `present`, `verify`, `next` (`terminal_read`),
  `cut` (whether the model saw less than the block: the request is cut to fit 24,000 tokens, the
  newest lines kept), `note`, and the text.
- **The local match** (`crates/marley_mcp/src/find.rs`, pure): the query's words (lowercased,
  punctuation dropped); a ref matches when its role or name holds every word; a line matches
  when it holds every word. One match: `sure: true`, `source: rules`, no call. Several: the
  candidates, and the layer is asked to rank among them only. None: the layer is asked over
  everything. With the layer off, refusing or unavailable: the local matches with `sure: false`,
  or none with `next`.
- **The question sets** (`marley_system_one::question`, pinned to `jev-1.13.0`): `find/1`, a
  template choice `which` ("Which item matches: '<query>'?") whose options are the tagged ids
  plus `none` ("no item matches"), instantiated per window of at most 254 items (a page's refs
  or a block's lines, one question per window in one request), and a noul `present` ("Does any
  item match: '<query>'?", true when at least one item is the thing asked for or states it,
  false when none does). The state is the tagged lines, each masked.
- **Reading**: `present` at or above 0.7 is found, at or below 0.35 absent, between is unsure
  (the cookbook's cutoffs); a `which` needs confidence at or above 0.5 and an option other than
  `none`; candidates are the top three by probability across windows. In `act` the top
  candidate is `ref` or `line` when found; in `suggest` the candidates carry `verify: true` and no
  top; in `shadow` the answer is the local one and the model's reading goes to the log only.
- **Policy**: the tab's or the terminal's project must be on the allow list, else the local
  match answers with `note: "project not listed for System One"`; a metadata-only project is
  local only too, since tagged text is the whole state. Deadline 3 seconds; the agent's own call
  waits on it, inside the bridge's 40 seconds and the app's 30.
- The use registered with #565 twice (`browser_find`, `terminal_find`), their modes in
  `marley.system_one.uses` and their dropdowns on the Marley page's System One section; the
  stand-in agent (`script/e2e/browser-fixture.sh`) gains `find` and `tfind` commands.
- `script/e2e/567-find-tools.sh` on the `replay` provider.

### Out (explicitly deferred)
- A locator or a text answer beyond the ref (`get`, `is`, Orca's `find` with an action, report
  03 §2.11): the agent acts on the ref with the tools it has.
- `notifications/tools/list_changed` when a mode flips: a running client sees the tools at its
  next `tools/list`.
- Searching every tab or every terminal in one call; a query over the flight recorder or a
  recording.
- Filtering a block by meaning in the terminal Chad reads (#528's business, the note's later
  list).
- A ref for a node the snapshot cut (`cut: true` at 30,000 characters): the agent asks with
  `full` or scrolls, as with `browser_snapshot`.
- Fitted thresholds; the cookbook's cutoffs stand until a use has labels.

## Reference (§20)
- **Orca:** its browser CLI's `find` takes a semantic locator plus an action (report 03 §2.11,
  `browser-advanced.ts`); the skill it teaches is snapshot, act on an `@eN` ref, snapshot again
  (§2.11). Marley keeps the ref as the answer and leaves the action to the agent's next call.
  Read as a report; nothing of Orca's browser engine is copied.
- **Upstream Zed:** the Agent Panel's tools search files by path and text (`grep`, `find_path`)
  and never a page; N/A for the page. For the terminal, Zed's `terminal` crate keeps the blocks
  (`blocks`, `block_output`) that `terminal_read` serves, kept as the source.
- **Warp:** N/A. Warp's agent reads its own blocks inside its model prompts; nothing of Warp's
  was read.

### Prior art
- **Behavior maps.** Report 03 §2.11 (agent-browser's `find`, `get`, `is`; the `@eN` refs), §3
  item 4 (the current-worktree tab); `docs/warp_architecture/subsystems/04-agent-ai-mcp.md`
  (Warp's MCP layer, no find tool).
- **Published material.** TypeSafe's semantic-find cookbook (docs.typesafe.ai/cookbooks/semantic_find,
  read 2026-09-26): lines tagged `L052| …`, a choice whose options are the line ids ("A Choice
  question returns a probability for every option. Use the line IDs as the options"), a noul
  for whether any line answers at all ("the Noul probability doesn't depend on the other
  options, so it can fall near zero when the document has no answer"), 218 lines in one request
  with 255 options as the choice's cap, and the cutoffs FOUND 0.7 and ABSENT 0.35. The API
  reference's limits (docs.typesafe.ai/api: 255 options; the note's 64k request with 32k for
  the state and the longest question). The MCP spec's `tools/list` and `listChanged`.
- **The code we already ship.** The registry (`crates/marley_mcp/src/registry.rs`: `Family` 12,
  `is_served` 38, `ToolSpec` 46, `REGISTRY` 78, `browser_read` 221, `tools_list` 264,
  `tool_schemas` 285, `browser_arguments` 357, `snapshot_schemas` 421, `terminal_schemas` 811,
  `terminal_read_schemas` 916); the router (`dispatch.rs`: `tools_call` 133, the permission check
  153, the deferred arm 165; `RequestCtx` in `marley_mcp.rs`); the browser tools
  (`crates/marley_workbench/src/browser_tools.rs`: `answer` 53, `run` 63 with its verb match,
  `page_of` 115, `take_snapshot` 318-383 ending in `hub.set_refs`, `ref_target` 678,
  `target_point` 690); the snapshot (`crates/marley_browser/src/snapshot.rs`: `MAX_CHARS` 15
  is 30,000, `RefTarget { id, role, name, .. }` 115, `describe` 133, `Snapshot { text, refs, cut }`
  144, `render` 155, the `[ref=eN]` line 334); the hub's refs (`browser.rs`: `set_refs` 1593,
  `ref_target` 1601); the terminal tools (`mcp.rs`: `answer` 317, `terminal_with_id` 364,
  `terminal_argument` 373, `terminal_read` 491-526, `tail` 530, `MAX_READ_LINES` 48 and
  `MAX_READ_BYTES` 51) and the terminal's blocks (`crates/terminal/src/terminal.rs`: `blocks`
  1845, `block_output` 1859); the redactor path (`mcp.rs`: `agent_redactor` 297, `for_agents`
  306); the stand-in agent's `find_ref` over snapshot lines (`script/e2e/browser-fixture.sh`,
  `write_mcp_agent` 205); #565's `ask`, `StateBuilder` and `Mode`. Does a crate we build own
  the seam? The registry owns the tools, the snapshot the refs, the terminal the blocks and #565
  the ask; the local match and the window split are the new pieces, pure in `marley_mcp`.

## UI proof
UI-AFFECTING: what agents see, the Browser tab after a click on an answered ref, and the
tools' listing. `script/e2e/567-find-tools.sh` (`compositor sway`, for the Browser tab).
Fixtures: the browser fixture's offline Chromium and a local page with a "Sign in" button, two
"Submit" buttons in two forms and a "Docs" link; a scratch repository whose terminal runs a
script printing 300 numbered lines with one `connection refused` line; the layer on `replay`
with the repository listed and both uses in `act` unless a step says otherwise; the replay file
holds a `find/1` row matched on `submit` (`which`: the second submit at 0.8, `present` 0.9), one
matched on `refused` for the terminal (`present` 0.92, the line at 0.85), and none for the
"coupon" query. The stand-in agent runs each call and prints the answer. Shots and checks:
- `567-01-clicked`: `find "sign in"` answers the button's ref with `source: rules` and `sure`,
  no call in the day's file; `browser_click` on it: the page shows the signed-in state.
- The log: `find "submit"` answers the replay's ref with `source: model`, probability 0.8.
- The log: `find "coupon code"` answers `sure: false`, `present` absent, `next:
  browser_snapshot`.
- The log: `tfind <block> "connection refused"` answers the line by the local match;
  `tfind <block> "where did the server fail to connect"` answers the replay's line.
- The log: in `shadow`, `find "submit"` answers the two local matches with `sure: false` and the
  day's file holds the model's row.
- `567-02-off-not-listed`: with `browser_find` set `off`, `tools/list` lacks it (the stand-in
  prints the list) and a call by name is refused with the setting named; the shot shows the
  Browser tab unchanged.
- The log: with the repository not listed, `find "submit"` answers the local matches with the
  note, and no request is logged.

## Locked-In Decisions
- D1: Code matches first ("local first and then jev second"): every word of the query in a
  ref's role or name, or in a line, is a match; one match answers alone; several are ranked by
  the model among themselves; none sends everything. The words are the deterministic path that
  works with the layer off.
- D2: The tools are listed only while their use is on: a feature flag on the registry's rows
  and the enabled set on the request context, so a client that never turns the use on never sees
  the tools (Chad: "turned off / on where the system will use or wont use it").
- D3: The refs are a snapshot the tool takes itself and keeps in the hub, so an answered ref is
  the same currency as `browser_snapshot`'s and `browser_click` takes it without a second call.
- D4: The state is tagged lines (`e12| button "Sign in"`, `L0042| …`), masked; a choice per
  window of 254 items plus `none`, and one `present` noul, all in one request (questions are
  judged in parallel and in isolation). The block's request is cut to 24,000 tokens, newest
  lines kept, and the answer says `cut`.
- D5: The cookbook's cutoffs to start: found at or above 0.7, absent at or below 0.35 on
  `present`; a `which` needs confidence at or above 0.5 and not `none`; three candidates at most.
- D6: An unlisted or metadata-only project gets the local match only, and the answer says why:
  the tagged text is the whole state, so there is no facts-only form of this use.
- D7: The modes: `shadow` answers locally and logs the model; `suggest` answers the model's
  candidates with `verify: true` and no top `ref` or `line`; `act` puts the top candidate in
  `ref` or `line`. The agent decides what to do with it either way; the tools change nothing.
- D8: Deadline 3 seconds inside the agent's own call; a few hundred calls and about 10 cents a
  day at five agents (the note's budget row), each logged with its cost.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `browser_find` is called with a query that exactly one ref's role or name holds, the system shall answer that ref with `source: rules` and `sure: true` and make no call. | Shot `567-01-clicked`; the day's file |
| REQ-002 | WHEN the query matches several refs, the system shall ask the layer to rank among them and answer the top candidate in `ref` with `source: model` and its probability. | The run log: `find "submit"` |
| REQ-003 | WHEN the query matches no ref and the layer reads `present` absent, the system shall answer `sure: false` with `browser_snapshot` as the next step. | The run log: `find "coupon code"` |
| REQ-004 | WHEN `browser_click` is called with a ref `browser_find` answered, the system shall click that element. | Shot `567-01-clicked` |
| REQ-005 | WHEN `terminal_find` is called with a query one line of the block holds, the system shall answer that line with `source: rules`; WHEN no line holds it, the system shall ask the layer over the tagged lines and answer its line. | The run log: both `tfind` calls |
| REQ-006 | WHERE a use is in `shadow`, its tool shall answer from the local match alone and the system shall log the model's reading. | The run log; the day's file |
| REQ-007 | WHERE a use is `off`, `tools/list` shall not list its tool and a call by name shall be refused with the setting named. | Shot `567-02-off-not-listed`; the run log |
| REQ-008 | WHERE the project is not on the allow list, the tools shall answer from the local match with a note and make no request. | The run log; the day's file |
| REQ-009 | WHEN a block's output holds a secret, the state the layer sees and logs shall hold it masked. | The day's file for a `tfind` over a block printing a token assembled at run time |
| REQ-010 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. At promotion: #565
  shipped; `brain_ask`; re-read `registry.rs`, `dispatch.rs` and `browser_tools.rs` (#520 and
  #521 may have landed rows and a caller in between).
- **P2 Code:** the settings page's two dropdown rows (the ledger row for `marley_page.rs`,
  first); the registry's feature flag, the two rows and their schemas; `find.rs`; the set; the
  two tools; the stand-in's commands; fmt and clippy clean; a review of the diff against each
  REQ and D6.
- **P3 Test:** write and run the scenario and read every shot; rerun 491 and 492 (their tool
  lists gain nothing while the uses are off); `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_mcp.md`, `marley_workbench.md`
  and `marley_system_one.md`; the plan's C0 and B rows; the ledger capture; close the ticket,
  archive, commit.
