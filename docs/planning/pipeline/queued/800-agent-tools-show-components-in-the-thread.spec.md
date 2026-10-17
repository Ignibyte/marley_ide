---
pipeline_id: 0301b40a-2ad9-44ed-b6bf-21c8d80dab70
ticket: docs/planning/tickets/open/TICKET-800-agent-tools-show-components-in-the-thread.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Agents show components in the thread
type: feature
slice: Control plane (agent components)
references:
  - docs/planning/design-notes/agent-components-2026-10-17.md
  - docs/planning/pipeline/queued/799-marley-components-and-the-component-library.spec.md
---

## Title
Two Marley MCP tools, `component_list` and `component_show`, let any agent draw a catalog
component (#799). In an agent thread the tool call's row draws the component; from an agent in a
terminal the component opens in a center tab.

## Scope
### In
- **A `component` family in `marley_mcp/src/registry.rs`**, served to every agent Marley offers
  its tools to (the shared Claude Code plugin, Codex, Zed's agent through the `marley` context
  server):
  - `component_list`: answers with the catalog, each type's description, field schema and
    example (#799's catalog description); a template (#801) joins it when that ships.
  - `component_show`: takes `component` (a tree) and an optional `title`. Marley checks the tree
    and answers `shown: true` with where it showed (`thread` or `tab`), or `shown: false` with
    the parse error and its JSON path, so the agent can fix it and call again.
  - The input schema of `component_show` describes the tree well enough for an agent to write
    one without calling `component_list` first: the types, their main fields, the tones.
- **The thread hook** (`agent_ui/src/conversation_view/thread_view.rs`, small and additive): in
  `render_tool_call`, a tool call whose tool is Marley's `component_show` is drawn by a
  function a `Global` holds (`MarleyToolCallRenderer`, set by `marley_workbench`), given the
  call's raw input. The function returns the drawn component, or `None` to fall back to Zed's
  own rendering. The row keeps its header (the tool's title) and its disclosure, and the raw
  input stays one click away.
- **Which calls are Marley's:** the tool's name as each agent reports it: Claude Code's plugin
  name (`mcp__plugin_marley_marley__component_show`), its plain MCP name
  (`mcp__marley__component_show`), Codex's, and Zed's agent's context-server tool. The Plan
  phase records each from a real call.
- **A call from a terminal agent** (Claude Code or Codex in a Marley terminal, told apart by the
  terminal identity the plugin's bridge sends): the component opens in a center tab titled with
  the call's `title`, beside the terminal's tab, reused for the same title.
- The thread draws from the call's input, since Zed's agent drops structured MCP results
  (`agent/src/tools/context_server_registry.rs`); the same check as the tool's runs on it, so a
  bad tree in the input draws the error, not a broken component.

### Out (explicitly deferred)
- Updating a component already shown (A2UI's `updateDataModel`), and components answering the
  agent.
- Components inside an agent's message text (a fenced block): only tool calls.
- Zed's agent sending structured results through `ContextServerTool`.

## Reference (§20)
Upstream Zed (`agent_ui`, `acp_thread`): the thread view draws each tool call as a row with its
title and its content (markdown, diff, terminal, image), with `read_file`'s numbered output the
one case chosen by the tool's name. Marley adds one more case by name, drawn by Marley's code.
Published material: MCP tools (<https://modelcontextprotocol.io/specification>, tool input
schemas), and MCP Apps (SEP-1865) for the general idea of a tool whose call shows UI; Marley's is
native, not HTML.

### Prior art
- The code we ship: `render_numbered_read_file_output`'s name check is the precedent for drawing
  a tool by name; `MarleyThreadHost` (#734, `agent_ui.rs`) is the precedent for a `Global` of
  function pointers Marley sets for `agent_ui`; `review_guide_open` (#733) is the precedent for
  a tool that opens a tab, and `guide_tab::open` reuses a tab by title; `mcp.rs`'s `answer` routes
  a family to its handler on the app thread.
- Behavior maps: none.
- Published material: A2UI over MCP (`a2ui://` resources and an action tool) was considered and
  left for later in the design note.

## UI proof
Scenario `script/e2e/800-agent-tools-show-components-in-the-thread.sh`. Agents are stood in by
fakes named through environment variables (the #547 rule), each replaying a recorded ACP session
whose tool call is a `component_show`: one as Claude Code's adapter reports it, one as Codex's,
and Zed's agent driven through a fake language model that calls the `marley` tool.
- `800-01-claude`: the Claude Code thread's row draws a table with a badge column (REQ-001).
- `800-02-codex` and `800-03-zed`: the same for Codex and Zed's agent (REQ-001).
- `800-04-raw`: the row's disclosure opened: the raw input under the component (REQ-002).
- `800-05-error`: a call whose tree has an unknown type: the row shows the error with its path,
  and the agent's next message quotes the tool's error answer (REQ-003).
- `800-06-terminal`: a terminal stand-in calls `component_show` through the bridge: a center tab
  opens with the component and the call's title (REQ-004).
- `800-07-list`: the terminal stand-in prints `component_list`'s answer: every catalog type
  (REQ-005).

## Locked-In Decisions
- **D1 — Draw from the input.** Every agent keeps a call's input; Zed's agent keeps no structured
  result.
- **D2 — `component_show` is a read-tier tool:** it changes no file, setting or process, so it
  never asks the user.
- **D3 — One hook in `agent_ui`, the drawing in Marley's crates** (#799's D4).
- **D4 — A terminal agent's component opens in a tab**, since it has no thread row to draw into.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Claude Code, Codex or Zed's agent calls `component_show` with a valid tree in a thread, the thread shall draw the component in the call's row | shots `800-01-claude`, `800-02-codex`, `800-03-zed` |
| REQ-002 | WHEN the row's disclosure is opened, the thread shall show the call's raw input | shot `800-04-raw` |
| REQ-003 | IF the tree is not valid, THEN the tool shall answer `shown: false` with the error and its JSON path, and the row shall show that error | shot `800-05-error` |
| REQ-004 | WHEN an agent in a Marley terminal calls `component_show`, Marley shall open the component in a center tab titled with the call's `title` | shot `800-06-terminal` |
| REQ-005 | WHEN `component_list` is called, it shall answer with every catalog type's description, schema and example | shot `800-07-list` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, record each agent's tool name from a real call, and
  how the bridge names a terminal caller.
- **P2 Code:** the family, the handler, the hook and its touchpoints row, the tab; a review of
  the diff; `script/gates.sh --diff` green.
- **P3 Test:** the visual check above.
- **P4 Complete:** CHANGELOG, `docs/marley/guide.md` (the MCP tools section), ledger, close,
  archive, commit.
