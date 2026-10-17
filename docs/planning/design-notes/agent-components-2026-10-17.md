# Agent components — 2026-10-17

Chad, 2026-10-17: "Lets build a gpui component system where if you are talking to an agent in
the zed agent panel it can be provided tools that would build tables and stuff like that. In the
settings or from the home we could see the component library. Maybe even research how we could
make this pluggable to add outside components without recompile." On the plan: "So the options
can be inline editing for agents to show components or static html in a scratch file or existing
project for that sort of style. I'm not sure if the WASM plugins would be useful but maybe we
create a ticket for that later."

The batch, in order: #797 (the upstream merge first, since #798 and #800 edit the agent files
upstream is rewriting), #798 (the agent panel's width in percent), #799 (`marley_components` and
the Component Library), #800 (the agent tools and the thread hook), #801 (templates from files),
#802 (an agent's HTML page in a Browser tab). WASM components and MCP Apps wait in
`docs/planning/intake/agent-components-beyond-templates.md`.

## What the tree has

- **Zed's component registry.** `crates/component` (the `Component` trait, `component.rs:170`;
  `#[derive(RegisterComponent)]`, `ui_macros/src/derive_register_component.rs`) fills a static
  registry through `inventory` at `workspace::init`. `crates/component_preview` opens it as a
  workspace item (`workspace::OpenComponentPreview`), in release builds too. About 67 components
  register, none of them Marley's. The registry is static: a component is a Rust type, so a
  component loaded from a file at run time cannot join it. That is why Marley's library is its
  own tab (#799).
- **Zed's `ui` crate** has what a renderer needs: `data_table`, `progress`, `callout`, `chip`,
  `banner`, `divider`, `label`, `list`, `disclosure`.
- **The thread view draws tool content in one closed match.** `render_tool_call_content`
  (`agent_ui/src/conversation_view/thread_view.rs`) sends markdown, embedded resources, images,
  diffs and terminals to fixed renderers. The content's `meta` is carried and never read. The
  only special case is by tool name (`read_file`'s numbered output). Inline components need one
  hook there (#800).
- **Zed's own agent drops structured MCP results.** `ContextServerTool::run`
  (`agent/src/tools/context_server_registry.rs`) keeps text and images only and never reads
  `structured_content` or `_meta`. Every agent keeps the tool call's input (`raw_input`), so
  the renderer draws from the input.
- **Markdown in a thread already draws tables and Mermaid** (`markdown/src/parser.rs`, GFM
  tables on). What it cannot do is a styled component: a stat, a bar chart, a status table with
  tones.
- **Marley's MCP tools** are declared in `marley_mcp/src/registry.rs` and answered on the app
  thread by `marley_workbench/src/mcp.rs`'s `answer`. `review_guide_open` opening a `GuideTab`
  is the pattern for a tool that shows something. Zed's agent gets the same tools through the
  `marley` context server Marley adds to the default settings.
- **`browser_open_url` already opens a local `.html` file** in a Browser tab of the caller's
  project (#586). An agent can write a page and show it today; #802 adds the theme, the reload
  and a scratch place to write it.

## The published approaches (researched 2026-10-17)

| Spec | Backer | Status | Wire format | Native renderer |
|---|---|---|---|---|
| A2UI | Google | v0.9.1 production, v1.0 candidate, "early stage public preview" | A flat JSON component list with ids, data bound by JSON Pointer | Intended; none in Rust or GPUI |
| MCP Apps (SEP-1865) | MCP project | Final; stable spec 2026-01-26, in Claude, ChatGPT, VS Code, Goose | HTML (`text/html;profile=mcp-app`) in a sandboxed iframe | Not addressed |
| OpenAI Apps SDK | OpenAI | ChatGPT implements MCP Apps; `window.openai` for its own extras | HTML | No |
| ACP | Zed and others | No RFD on rich UI as of 2026-10-12; v2 content allows `_`-prefixed custom types | MCP content blocks | n/a |
| json-render | Vercel Labs | 0.21.1 | A JSON element map against a typed catalog | JavaScript only |
| AG-UI | CopilotKit | No version published | An event stream that carries the others | n/a |

Sources: A2UI (<https://developers.googleblog.com/introducing-a2ui-an-open-project-for-agent-driven-interfaces/>,
<https://github.com/google/A2UI>, <https://a2ui.org/specification/v0.9-a2ui/>,
<https://a2ui.org/reference/renderers/>, <https://a2ui.org/guides/a2ui_over_mcp>); MCP Apps
(<https://modelcontextprotocol.io/seps/1865-mcp-apps-interactive-user-interfaces-for-mcp>,
<https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/>,
<https://github.com/modelcontextprotocol/ext-apps>); OpenAI
(<https://developers.openai.com/apps-sdk/mcp-apps-in-chatgpt>); ACP
(<https://agentclientprotocol.com/rfds/updates>, <https://agentclientprotocol.com/protocol/v2/content.md>);
json-render (<https://github.com/vercel-labs/json-render>); AG-UI
(<https://github.com/ag-ui-protocol/ag-ui>).

## What Marley does

- **A nested JSON tree of a fixed catalog, drawn natively** (#799, #800). The agent sends data,
  never code, and only the catalog's types. The tool's input schema teaches the format, so no
  agent needs prior training on it. Type names follow A2UI's basic catalog where the two
  overlap (`text`, `row`, `column`, `card`, `divider`, `image`), so an A2UI translation can come
  later if A2UI reaches 1.0. A2UI itself is not adopted now: it is a preview, its flat id lists
  are harder for an agent to write in one call, and nobody has a GPUI renderer to share.
- **Colors by tone, never by value.** A component names `success`, `warning`, `error`, `muted`
  or `accent`, and the theme decides the color, so every component follows the user's theme.
- **Templates from files** (#801): a JSON file names its inputs and lays out built-in components
  with placeholders. No code, no sandbox, no recompile.
- **HTML for anything the catalog cannot draw** (#802): a page in a Browser tab, written to a
  scratch folder or the project, linked to a stylesheet of Marley's theme, reloaded as it
  changes.
- **Later, if templates and HTML fall short:** WASM components and MCP Apps (the intake doc).
- **Not a plugin path:** gpui's hot reloading (upstream `670a779a5e`, #65738, merged by #797)
  patches a running debug build from Dioxus's `dx serve --hot-patch` through Subsecond. It is
  for developing Marley's own components faster, not for adding components to an installed
  Marley: it needs the dev server and the exact build.
- **Rejected:** Rust dynamic libraries. Rust has no stable ABI, a library must match Marley's
  exact gpui build, and a crash in one takes Marley down.
