---
pipeline_id: da0f2056-7b66-4306-9531-3d152c1a8e49
ticket: docs/planning/tickets/closed/TICKET-567-find-tools.md
status: Phase 4 — Complete PASS
title: "browser_find and terminal_find for agents"
type: feature
slice: prong 2 (the MCP server's tools) with prong 3 (the Browser tab); the Jev note's use 2, on #565's layer
references: [docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/pipeline/completed/565-system-one-layer.spec.md, docs/planning/pipeline/completed/566-stop-kind.spec.md, docs/planning/pipeline/completed/492-browser-tools-for-agents.spec.md, docs/planning/pipeline/completed/491-marley-mcp-in-the-app.spec.md, docs/planning/pipeline/completed/516-secret-redaction-for-agents.spec.md]
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
  verb `find`, read tier) and `terminal_find` (`Family::Terminal`, verb `find`, read tier), with
  `find` arms of their own in `browser_schemas` and `terminal_schemas` ahead of the arms that catch every other verb.
  Both are in `CONDITIONAL_TOOLS`, listed only while enabled: `RequestCtx` gains the enabled set
  from the server's `ServerData.enabled`, `tools_list_for(principal, enabled)` leaves an off tool
  out, and `dispatch::tools_call` refuses one by name with the setting to turn on as the reason.
  `marley_workbench::mcp` pushes the set whenever #565's layer applies new settings (each tool on
  while `use_mode` reads its use as not `off`), through a channel to a background task that takes
  the server's lock (`transport::set_enabled`), as the fleet snapshot goes. Neither tool joins
  the outside clients' lists (#524).
- **`browser_find`** (`crates/marley_workbench/src/browser_tools.rs`): arguments `query`
  (required, at most 200 characters), `tab` and `full` as `browser_snapshot` takes them. It
  takes the snapshot through the walk `browser_snapshot` uses, moved into a helper both call, and
  keeps the refs in the hub (`set_refs`), so a ref it answers is clickable. The tab's project is
  `hub.project_of(target)`. The answer: `tab`, `query`, `source` (`rules`, `model`, `none`),
  `sure`, `ref` (in `act`, the top candidate when sure), `candidates` (at most three: `ref`,
  `role`, `name`, `probability`), `present` (the noul, when asked), `verify` (in `suggest`),
  `next` (`browser_snapshot` when not sure), `note` (why the model was not asked, when it was
  not), and the same as text.
- **`terminal_find`** (`crates/marley_workbench/src/mcp.rs`, a spawned route like `ports_list`):
  arguments `terminal` (the caller's own by default, through `terminal_of`), `block` as
  `terminal_read` takes them, and `query`. It reads the block's output as `terminal_read` does
  (masked whole, the tail of 2,000 lines and 256 KiB kept) and answers `terminal`, `block`,
  `query`, `source`, `sure`, `line` (in `act`; the line's number in the block's output),
  `candidates` (at most three: `line`, `text`, `probability`), `present`, `verify`, `next`
  (`terminal_read`), `cut` (whether the model saw less than the block: the requests stop at
  24,000 tokens, the newest lines kept), `note`, and the text. The project is the terminal view's
  workspace's.
- **The local match** (`crates/marley_mcp/src/find.rs`, pure): the query's words (lowercased,
  punctuation and stop words such as `the`, `a`, `where` and `did` dropped); a ref matches when
  its role and name hold every word as a whole word; a line matches when it holds every word. One match: `sure: true`, `source: rules`, no call. Several: the
  candidates, and the layer is asked to rank among them only. None: the layer is asked over
  everything. With the layer off, refusing or unavailable: the local matches with `sure: false`,
  or none with `next`.
- **The question sets** (`marley_system_one`'s root, pinned to `jev-1.13.0`): `find_1/1` to
  `find_254/1`, one per item count, since #565's questions are compiled in and a caller never
  supplies its own. Each asks a choice `which` ("Which item in the state matches its `query`
  line?") over `none` ("no item matches") and the options `1` to `N`, and a noul `present`
  ("Does any item match the `query` line?", true when at least one item is the thing asked for or
  states it, false when none does). The options are one table, `none` first, so each set's are a
  prefix of it, held in a `static LazyLock`. The state is the `query` line and the items as
  `1: …` to `N: …`, each masked and cut like any text. A page's refs or a block's lines go in
  windows of at most 254 items, a request each, asked together.
- **Reading**: `present` reads found above 0.65, absent below 0.35 and unsure between (the
  layer's noul band, as every use reads it), found when any window's reads yes and absent when
  every window's reads no; a `which` needs confidence at or above 0.5 and an option other than
  `none`; candidates are the top three by probability across windows, from the answers #565's
  `Asked` now carries. In `act` the top
  candidate is `ref` or `line` when found; in `suggest` the candidates carry `verify: true` and no
  top; in `shadow` the answer is the local one and the model's reading goes to the log only.
- **Policy**: the tab's or the terminal's project must be on the allow list, else the local
  match answers with `note: "project not listed for System One"` and no call is made or logged; a
  metadata-only project is local only too, since the items are text and the whole state. A
  repeated find over an unchanged page, which #565's gate refuses, answers locally with a note.
  Deadline 3 seconds; the agent's own call waits on it, inside the bridge's 40 seconds and the
  app's 30.
- Two uses, `browser_find` and `terminal_find`: their modes in `marley.system_one.uses`
  (`"off"` in `default.json`), and Browser Find and Terminal Find dropdowns after Stop Kind on the
  Marley page's System One section. #565's `ask` and `record` take a `UseSpec` by value, since
  the find tools make theirs at call time. The stand-in agent (`script/e2e/browser-fixture.sh`)
  gains `find`, `tfind` and `click-ref`, a click on a ref as given.
- `script/e2e/567-find-tools.sh` on the `replay` provider.

### Out (explicitly deferred)
- A locator or a text answer beyond the ref (`get`, `is`, Orca's `find` with an action, report
  03 §2.11): the agent acts on the ref with the tools it has.
- `notifications/tools/list_changed` when a mode flips: a running client sees the tools at its
  next `tools/list`, which for Claude Code means reconnecting the MCP server.
- The find tools for outside clients (#524's lists stay as they are).
- Context for elements that share a name (two "Submit" buttons look alike to the model).
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
- **The code we already ship** (the lines as of promotion, `e47cbc7b9d`). The registry
  (`crates/marley_mcp/src/registry.rs`: `Family` 13, `is_served` 42, `ToolSpec` 49, `REGISTRY` 82,
  `browser_read` 281, `lookup` 311, `tools_list_for` 331, `tool_schemas` 352, `browser_schemas`
  401 with its catch-all at 415, `browser_arguments` 455, `snapshot_schemas` 527,
  `terminal_schemas` 1087 with its catch-all at 1091, `terminal_read_schemas` 1237); the router
  (`dispatch.rs`: `handle_message`'s `tools/list` 43, `tools_call` 151, `permits` 172, the
  deferred arm 188; `RequestCtx` at `marley_mcp.rs:94`; `clients.rs`'s `CLIENT_READ_TOOLS` 12 and
  `permits` 79); the transport (`transport.rs`: `ServerData` 35, the context built in `serve_post`
  at 350, `signal_change` 495); the browser tools (`browser_tools.rs`: `answer` 89, `run` 246 with
  its page match at 280, `page_of` 360, `take_snapshot` 941-1005 ending in `hub.set_refs`,
  `ref_target` 1328, `target_point` 1340); the snapshot (`crates/marley_browser/src/snapshot.rs`:
  `MAX_CHARS` 15 is 30,000, `RefTarget` 114, `describe` 130, `Snapshot` 143, `render` 154, the
  `[ref=eN]` text 333); the hub (`browser.rs`: `project_of` 799, `BrowserProject` 487, `set_refs`
  2419, `ref_target` 2427); the terminal tools (`mcp.rs`: `answer` 407, `ports_list` 433 as the
  spawned route, `terminal_of` 528, `terminal_read` 659-693, `tail` 697, `MAX_READ_LINES` 49 and
  `MAX_READ_BYTES` 52, `publish` 145) and the terminal's blocks
  (`crates/terminal/src/terminal.rs`: `blocks` 1915, `block_output` 1929); the redactor path
  (`mcp.rs`: `agent_redactor` 377, `model_redactor` 388, `for_agents` 396); the stand-in agent
  (`script/e2e/browser-fixture.sh`: `write_mcp_agent` 571, `find_ref` 646, the `tools` command
  667); #565's and #566's `ask`, `record`, `use_mode`, `detail`, `Asking` and `Asked`
  (`system_one.rs` 511-762) and the stop kind's static sets as the pattern. Does a crate we build own
  the seam? The registry owns the tools, the snapshot the refs, the terminal the blocks and #565
  the ask; the local match and the window split are the new pieces, pure in `marley_mcp`.

## UI proof
UI-AFFECTING: what agents see, the Browser tab after a click on an answered ref, and the
tools' listing. `script/e2e/567-find-tools.sh` (`compositor sway`, for the Browser tab).
Fixtures: the browser fixture's offline Chromium and a local page with a "Sign in" button that
shows "Signed in" when clicked, two "Submit" buttons in two forms and a "Docs" link; a scratch
repository whose terminal runs a script printing 300 numbered lines with one `connection refused`
line and a token assembled at run time; the layer on `replay` with the repository listed and both
uses in `act` unless a step says otherwise; the replay file holds a `find_2/1` row matched on the
`submit` query (`which`: `2` at 0.8, `present` 0.9), a row over the page's refs for `coupon code`
(`present` 0.05), and a `find_254/1` row for the terminal's second query (`present` 0.92, `212`
at 0.85). The stand-in agent runs each call and prints the answer. Shots and checks:
- `567-01-clicked`: `find "sign in"` answers the button's ref with `source: rules` and `sure`,
  no call in the day's file; `click-ref` on it: the page shows "Signed in".
- The log: `find "submit"` answers the replay's ref with `source: model`, probability 0.8.
- The log: `find "coupon code"` answers `sure: false`, `present` absent, `next:
  browser_snapshot`.
- `567-03-settings`: the Marley settings page's System One section with Browser Find and Terminal
  Find after Stop Kind.
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
- D4: The state is the query and the items as labeled lines (`1: button “Sign in”`), masked; a
  choice over a window of at most 254 items plus `none`, and one `present` noul, per request, the
  windows asked together (promotion: #565's sets are compiled in, so there is a set per item
  count). The block's requests stop at 24,000 tokens, newest lines kept, and the answer says
  `cut`.
- D5: The layer's noul band, not the cookbook's cutoffs (promotion): `present` is found above
  0.65 and absent below 0.35, as for every use; a `which` needs confidence at or above 0.5 and
  not `none`; three candidates at most.
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
| REQ-003 | WHEN the query matches no ref and the layer reads `present` absent, the system shall answer `sure: false` with `browser_snapshot` as the next step. | The run log: `find "coupon code"`, whose replay row reads `present` 0.05 |
| REQ-004 | WHEN `browser_click` is called with a ref `browser_find` answered, the system shall click that element. | Shot `567-01-clicked` |
| REQ-005 | WHEN `terminal_find` is called with a query one line of the block holds, the system shall answer that line with `source: rules`; WHEN no line holds it, the system shall ask the layer over the tagged lines and answer its line. | The run log: both `tfind` calls |
| REQ-006 | WHERE a use is in `shadow`, its tool shall answer from the local match alone and the system shall log the model's reading. | The run log; the day's file |
| REQ-007 | WHERE a use is `off`, `tools/list` shall not list its tool and a call by name shall be refused with the setting named. | Shot `567-02-off-not-listed`; the run log |
| REQ-008 | WHERE the project is not on the allow list, the tools shall answer from the local match with a note and make no request. | The run log; the day's file |
| REQ-009 | WHEN a block's output holds a secret, the state the layer sees and logs shall hold it masked. | The day's file for a `tfind` over a block printing a token assembled at run time |
| REQ-010 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. At promotion: #565 and #566
  shipped; `brain_ask`; the seams re-read through an Explore agent (the notes' Promotion entry).
- **P2 Code:** the ledger rows 55, 58 and 61 first; the two dropdowns, `default.json` and the
  docstring; `marley_mcp`'s `find.rs`, the two rows, their schemas, `CONDITIONAL_TOOLS`, the
  enabled set through `RequestCtx` and `ServerData`, and the refusal; the find sets; `UseSpec` by
  value and `Asked.answers`; the two tools and the shared reading; the enabled set's observer and
  channel; fmt and clippy clean; a review of the diff against each REQ and D6.
- **P3 Test:** write and run the scenario and read every shot; rerun 491, 492, 524 and 584 (their
  tool lists gain nothing while the uses are off), 565 and 566 (the section grows, and `ask`'s
  signature moved); the golden set; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_mcp.md`, `marley_workbench.md`
  and `marley_system_one.md`; the plan's C0 and B rows; the ledger capture; close the ticket,
  archive, commit.
