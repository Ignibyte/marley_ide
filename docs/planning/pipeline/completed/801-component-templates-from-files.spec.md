---
pipeline_id: da1112c3-de32-4177-8aab-72df4f146b14
ticket: docs/planning/tickets/closed/TICKET-801-component-templates-from-files.md
status: Phase 4 — Complete PASS
title: Component templates from files
type: feature
slice: Control plane (agent components)
references:
  - docs/planning/design-notes/agent-components-2026-10-17.md
  - docs/planning/pipeline/queued/799-marley-components-and-the-component-library.spec.md
  - docs/planning/pipeline/queued/800-agent-tools-show-components-in-the-thread.spec.md
---

## Title
A component defined in a JSON file, laid out from the built-in catalog with placeholders for its
inputs, joins the Component Library and the agents' catalog without a restart or a recompile.

## Scope
### In
- **The folder:** `components/` in Marley's config directory (`~/.config/marley/components/` on
  Linux), one template per `*.json` file, read at start and again when a file in it is added,
  changed or removed (Zed's `Fs::watch`).
- **The file:** `name` (lowercase, digits and `_`), `description`, `inputs` (a JSON Schema
  object), `example` (inputs the library draws it with) and `body` (a #799 tree where a value
  may be `{"$input": "<name>"}`).
- **Use:** `{"type": "<template name>", "inputs": {...}}` anywhere a component goes, in
  `component_show` (#800) and in the library's scratch box (#799). The inputs are checked against
  the template's schema (`jsonschema`, a workspace dependency), substituted, and the result
  checked as any tree is.
- **The library:** a Templates group, each template drawn from its `example` with its file's
  path; a broken file shown with its error and its path; a button that opens the folder.
- **`component_list`** lists the templates after the built-ins, with their schemas and examples.
- Rules: a template cannot take a built-in's name; two files with one name are both refused,
  each naming the other; a template may use other templates, to a depth of 4, and a cycle is an
  error.

### Out (explicitly deferred)
- Repetition (a card per item of an input list) and conditions: placeholders replace whole
  values only. An array input placed in a table's rows covers lists.
- Templates kept in a project (`.marley/components/`).
- WASM components and MCP Apps: `docs/planning/intake/agent-components-beyond-templates.md`.

## Reference (§20)
N/A — Marley-specific: Zed's components are Rust types in a static registry and has no
user-defined ones. Published material: A2UI's custom catalogs (a client advertises catalogs
beyond the basic one, <https://a2ui.org/specification/v0.9-a2ui/>) and json-render's catalog,
both of which define new types by schema; here the body is laid out from built-ins, so nothing new
has to be drawn.

### Prior art
- The code we ship: `jsonschema` 0.51 validates inputs; Zed's `Fs::watch` and the settings
  file watcher (`settings::watch_config_file`) are the pattern for reloading a file in the config
  directory; `paths::config_dir()` names it.
- Behavior maps: none.
- Published material: A2UI custom catalogs, json-render, in the design note.

## UI proof
Scenario `script/e2e/801-component-templates-from-files.sh`, its HOME a scratch folder whose
config directory the setup fills:
- `801-01-library`: Marley started with one template, `deploy_status` (a card with a stat, a
  table and a badge): the Templates group draws it from its example, with its path (REQ-001).
- `801-02-added`: a second file copied in while Marley runs: it appears without a restart
  (REQ-002).
- `801-03-broken`: a file with a placeholder for an input its schema does not name: the library
  shows the error and the path (REQ-003).
- `801-04-thread`: a recorded Claude Code session calling `component_show` with `deploy_status`:
  the thread draws it (REQ-004).
- `801-05-bad-inputs`: the same call with an input of the wrong type: the row shows the schema
  error (REQ-005).
- `801-06-list`: `component_list` from a terminal stand-in: `deploy_status` listed, the broken
  file not (REQ-006).

## Locked-In Decisions
- **D1 — Layout, not code.** A template arranges built-ins; it cannot draw anything new, so no
  sandbox is needed and every template follows the theme.
- **D2 — Whole-value placeholders only** in this ticket.
- **D3 — The user's config directory only**, where Marley's own settings live.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts with a valid template in the folder, the library shall draw it from its example with its file's path | shot `801-01-library` |
| REQ-002 | WHEN a template file is added or changed while Marley runs, the library and `component_list` shall show it without a restart | shot `801-02-added` |
| REQ-003 | IF a template file is not valid, THEN the library shall show its error and path, and `component_list` shall leave it out | shots `801-03-broken`, `801-06-list` |
| REQ-004 | WHEN `component_show` names a template with valid inputs, the thread shall draw the expanded component | shot `801-04-thread` |
| REQ-005 | IF the inputs do not match the template's schema, THEN the tool shall answer the schema error and the row shall show it | shot `801-05-bad-inputs` |
| REQ-006 | `component_list` shall list every valid template with its schema and example | shot `801-06-list` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, the file's exact shape and the error type.
- **P2 Code:** loading, watching, expansion and the library's group in `marley_components` and
  `marley_workbench`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test:** the visual check above.
- **P4 Complete:** CHANGELOG, `docs/marley/guide.md` (writing a template, with the example),
  ledger, close, archive, commit.
