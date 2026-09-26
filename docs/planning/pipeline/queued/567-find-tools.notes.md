# browser_find and terminal_find for agents — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-567-find-tools.md
- **Pipeline spec:** 567-find-tools.spec.md

## Phase 1 — Plan
- **Request:** the Jev note's use 2 (2026-09-25): "a choice over a page's refs or a block's
  tagged lines, with `none`, and a `present` noul", so "the agent gets the top three, or 'not
  sure: read the snapshot', instead of a 7,500-token snapshot"; with Jev off or down "the tools
  are not listed". Chad approved the use on 2026-09-26 with the rules quoted in #565's D1 to
  D4; "local first" gives the tools a match by words before any call.
- **Classification / tier:** feature, prong 2 (the server's tools) with prong 3 (the Browser
  tab). Marley crates: `marley_mcp`, `marley_system_one`, `marley_workbench`. One Zed path with a
  row: `marley_page.rs`. Depends on #565. Size S each, as the note sizes them.
- **Recall (§18.3):**
  - AD-claude-491-marleys-mcp-server-runs-in-the-app-behind-a-stdio-bridge-001: wire names are
    `family_verb`; a tool whose answer is the app's comes back deferred and is answered on the
    main thread. Both tools follow their families' routes.
  - AD-claude-492-agents-drive-the-browser-tab-through-the-mcp-server-001: the refs are the
    snapshot's, kept in the hub per tab, and a stale ref says "take a new snapshot"; `browser_find`
    keeps the same currency (D3).
  - AD-claude-516-redact-at-the-tool-boundary-on-by-default-001: a block's output is masked
    before anything leaves; `terminal_find` masks before tagging.
  - L-claude-516-fake-secrets-are-put-together-at-run-time-001 (REQ-009's fixture).
  - The 491 completed notes: the stand-in agent's shape and the bridge's 40-second wait; the
    508 queued notes record `APP_CALL_TIMEOUT_SECONDS` at 30, which bounds the deadline (D8).
  - Brain: not consulted in this drafting session (the brief is read-only); promotion asks.
- **Discovery** (2026-09-26, at the working tree of `51bfe04034`):
  - `crates/marley_mcp/src/registry.rs`: `Family` (12-21), `is_served` (38: `Terminal` and
    `Browser`), `ToolSpec` (46-57: `family`, `verb`, `tier`, `grant_class`, `description`),
    `REGISTRY` (78-218), `browser_read` (221), `lookup` (251), `tools_list` (264-279: filters
    by `is_served`), `tool_schemas` (285-303, exhaustive over `Family`), `browser_schemas`
    (333-346, a `_` arm to the write schemas), `browser_arguments` (357), `snapshot_schemas`
    (421-440: `full`; the answer's `tab`, `snapshot`, `refs`, `cut`), `terminal_schemas`
    (811-817), `terminal_argument_schema` (820), `terminal_read_schemas` (916-948). The tests at
    963 pin the registry's names and count and will change with the rows.
  - `crates/marley_mcp/src/dispatch.rs`: `handle_message` (18: `tools/list` at 28 calls
    `registry::tools_list()` with no context), `tools_call` (133-174: `lookup`, the permission
    check, the deferred arm for the app's families). `crates/marley_mcp/src/marley_mcp.rs`:
    `RequestCtx { snapshot, grants, surface_index }`, `PendingCall`, `APP_CALL_TIMEOUT_SECONDS`.
  - `crates/marley_workbench/src/browser_tools.rs`: `answer` (53-61: spawns `run`), `run`
    (63-114: the verb match; `page_of` for the tab), `take_snapshot` (318-383: the main tree, the
    frames, the iframes, `snapshot::render(&trees, full)`, `hub.set_refs(tab, refs)`),
    `ref_target` (678), `target_point` (690: a `ref` argument resolves through the hub).
  - `crates/marley_browser/src/snapshot.rs`: `MAX_CHARS` (15), `RefTarget` (115-131), `describe`
    (133), `Snapshot` (144), `render` (155), the ref line (334: `[ref=eN]`).
  - `crates/marley_workbench/src/browser.rs`: `set_refs` (1593), `ref_target` (1601).
  - `crates/marley_workbench/src/mcp.rs`: `MAX_READ_LINES` (48), `MAX_READ_BYTES` (51), `answer`
    (317-329: a synchronous match; `terminal_find` needs a spawned path like the browser's),
    `terminal_with_id` (364), `terminal_argument` (373), `terminal_read` (491-526: the block,
    `block_output`, masking whole before `tail`), `tail` (530-545).
  - `crates/terminal/src/terminal.rs`: `blocks` (1845), `block_output` (1859),
    `block_output_kept` (1869).
  - `script/e2e/browser-fixture.sh`: `mcp_agent` (76), `write_mcp_agent` (205-290: the client,
    `tool`, `find_ref` by role and name over the snapshot's lines, the commands' `main`).
  - #565's spec: `UseSpec`, `StateBuilder`, `ask`, `Mode`, the replay rows with `match`.
  - TypeSafe's cookbook and API reference (the spec's prior art), read 2026-09-26.
- **Decisions:** D1 to D8 in the spec.

### Design
- **`marley_mcp::find`** (pure): `words(query) -> Vec<String>`; `Item { id, text }`;
  `matches(items, words) -> Vec<usize>` (every word in the item's text, case-insensitive);
  `windows(items, 254)`; `tag_lines(items) -> String` (`<id>| <text>`); `Local { Sure(id),
  Several(ids), None }`. A ref's text is `role "name"`; a line's is the line.
- **`find/1`** (`marley_system_one::question`): the template `which` filled with a window's ids
  and descriptions (the tagged text, so the option's description is the item), plus `none`; the
  `present` noul. `reading` joins the windows: each window's top option with its probability,
  the best three overall, `none` counted as absent for its window.
- **`browser_find`** (`browser_tools.rs`): after `take_snapshot`'s tree walk (shared: the walk
  moves into a helper both tools call) and `set_refs`, the items are the refs (`e1` on, `role
  "name"`); the local match; then, by mode and policy, `system_one::ask(&BROWSER_FIND, state,
  local, cx).await` with the tab as the dedupe subject; the answer as the spec's shape. The
  project for the allow list is the tab's workspace root (the `BrowserView` that holds the tab,
  or #520's caller when it has landed).
- **`terminal_find`** (`mcp.rs`): routed like the terminal tools but through `cx.spawn`, since
  the ask is async: the block and its output as `terminal_read` reads them, masked, `tail`,
  then the lines as items (`L0001` on, the tail's numbering kept so `line` names the block's
  line as `terminal_read`'s text shows it), cut to 24,000 tokens (bytes over four) from the
  newest line back; the local match; the ask with the terminal and block as the subject.
- **The listing**: `ToolSpec.feature`; `RequestCtx.features: &[&str]`; `tools_list(features)`;
  `tools_call` refuses `Some(feature)` not in the set with `"<tool> is off: set
  marley.system_one.uses.<tool> to shadow, suggest or act"`. `transport::ServerData` gains
  `features`, which `marley_workbench::mcp` sets from `MarleySettings` in its settings observer
  (the one that refreshes redaction).
- **The stand-in**: `find <query> [--tab id] [--full]` and `tfind <block> <query>` print the
  answer's JSON; `tools` prints `tools/list`'s names.
- **File manifest.** Marley: `crates/marley_mcp/src/registry.rs`, `dispatch.rs`,
  `marley_mcp.rs`, `transport.rs`, `find.rs` (new); `crates/marley_system_one/src/question.rs`;
  `crates/marley_workbench/src/browser_tools.rs`, `mcp.rs`, `system_one.rs`;
  `script/e2e/browser-fixture.sh`, `script/e2e/567-find-tools.sh`. Zed:
  `crates/settings_ui/src/marley_page.rs` (two dropdown rows).
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: the `marley_page.rs` row names the two
  mode items.

### E2E plan
Fixtures: the browser fixture's offline Chromium and `serve_site` with the page described in the
spec; a scratch repository with `print-lines.sh` (300 numbered lines, line 212 `connect: connection
refused`, and, for REQ-009, a `ghp_` token assembled at run time on line 250); the layer on
`replay` with the repository listed; the replay file's three rows; both uses `act` at the start.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-004 | New Browser Tab on the page; `mcp_agent find "sign in"`; `mcp_agent click <ref>` | `567-01-clicked`; the day's file has no `find/1` row yet |
| REQ-002 | `mcp_agent find "submit"` | the run log: `source: model`, the second submit's ref, 0.8 |
| REQ-003 | `mcp_agent find "coupon code"` | the run log: `sure: false`, `next: browser_snapshot`; the day's file: `NoSignal: no replay row` |
| REQ-005, REQ-009 | `./print-lines.sh` in the terminal; `mcp_agent tfind <block> "connection refused"`; then `"where did the server fail to connect"` | the run log: line 212 by rules, then by the replay; the day's file's state holds `[redacted: github token]` |
| REQ-006 | `system_one_setting uses {"browser_find": "shadow", "terminal_find": "act"}`; `mcp_agent find "submit"` | the run log: two candidates, `sure: false`; the day's file: the model's row |
| REQ-008 | `system_one_setting projects []`; `mcp_agent find "submit"` | the run log: the note; no new row |
| REQ-007 | `system_one_setting uses {"browser_find": "off", "terminal_find": "off"}`; `mcp_agent tools`; `mcp_agent find "submit"` | `567-02-off-not-listed`; the run log: the list without either tool, the refusal's text |
| REQ-010 | Test's gate and regression runs | the exit codes |

What no scenario reaches: a real model's ranking (the replay rows stand in) and a page with more
than 254 refs (the window split is reviewed and the replay file can carry a two-window answer if
Test wants it).

### Risks
- `tools/list` is pure and takes no context today; threading the enabled set through
  `RequestCtx` touches the transport's `ServerData`. Small, and the registry tests pin the
  names, so they change with the rows.
- A page with more refs than the snapshot's 30,000 characters keep: the items are the refs the
  snapshot returned, and `cut` says so; `full` gives every node a ref and a longer state.
- Token estimates by bytes over four: a block of wide characters underestimates; the cut keeps a
  margin under the 32k the note records for the state.
- The refs move when the page changes between the find and the click; the same rule as today
  ("take a new snapshot") applies.
- `terminal_find`'s async route in `mcp.rs`: `answer` is synchronous for the terminal tools; a
  spawned task with `call.answer` at its end follows `browser_tools::answer`'s shape.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
