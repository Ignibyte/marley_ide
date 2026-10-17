---
status: intake
created: 2026-10-17
ticket: <unassigned>
pipeline_spec: <unassigned>
---

# Agent components beyond templates: WASM components and MCP Apps

## What
Two ways to add agent components that #799 to #802 leave out:

1. **WASM components.** A third party ships a component as a WebAssembly module, through Zed's
   extension mechanism. Marley calls the module with the agent's inputs, and the module returns a
   tree of built-in components, the same data #799 draws. The module never draws pixels.
2. **MCP Apps** (SEP-1865, stable since 2026-01-26, in Claude, ChatGPT, VS Code and Goose). An
   MCP server's tool names a `ui://` HTML resource; Marley would open it in a Browser tab, with
   the spec's sandbox, content security policy and the JSON-RPC bridge between the page and the
   host.

## Why
Chad, 2026-10-17: "I'm not sure if the WASM plugins would be useful but maybe we create a ticket
for that later. I'm curious what would be the benefit of that I suppose."

What WASM would add over templates (#801) and pages (#802):

- **Logic.** A template only places the inputs it is given. A WASM component can compute: group
  and total rows, sort, bucket timestamps into a histogram, lay out a Gantt chart or a flame graph
  from raw spans, format durations and sizes. Today the agent does that work before it calls,
  which spends its tokens and gets the arithmetic wrong at times.
- **Native and themed.** Unlike an HTML page it draws inside the thread row, in the theme, with
  Marley's fonts, because what it returns is the same tree a template makes.
- **Safe to install.** It runs in wasmtime, which Zed already embeds, with no file, network or
  process access unless Marley grants it. A Rust dynamic library would have the same power and
  none of the safety.
- **Shipped like extensions.** It installs and updates through the extension store's flow, with
  no Marley release.

What it costs: a new WIT interface beside `crates/extension_api/wit/since_v0.8.0` (which has
nothing for UI), its host side in `extension_host`, versioning that interface, and a way to
publish Marley-only extensions. Most of the value also comes from adding built-ins to the
catalog, which needs no plugin system. The case for WASM is components Marley would never write
itself, for someone else's tool or data.

MCP Apps matter for a different reason: the MCP servers Chad uses may already ship them, since
the major hosts support them. Marley showing them in a Browser tab would make those servers'
interactive views work as they do in Claude.

## Notes
- Research and sources: `docs/planning/design-notes/agent-components-2026-10-17.md`.
- WASM: `crates/extension_host` (wasmtime 48.0.5, the lockfile's pin), `crates/extension_api`.
  The return type would be #799's tree as JSON, so the host checks it with the same parser.
- MCP Apps: <https://modelcontextprotocol.io/seps/1865-mcp-apps-interactive-user-interfaces-for-mcp>,
  <https://github.com/modelcontextprotocol/ext-apps>. The bridge needs the page in a Browser tab
  to call back into Marley; `marley_browser`'s CDP session can bind a function into the page.
- Revisit WASM after templates have been in use, if agents keep doing arithmetic a component
  could do, or someone wants to ship a component for their own tool. Revisit MCP Apps when an MCP
  server Chad uses ships one.

## Promotion
This is NOT an active pipeline doc — it is a candidate. Promote it via
`/pipeline:plan` when ready: it becomes a ticket (`docs/planning/tickets/open/`) + an active
pipeline doc pair (`docs/planning/pipeline/active/`). On promotion, set
`status: promoted` and fill `ticket:` + `pipeline_spec:`.
