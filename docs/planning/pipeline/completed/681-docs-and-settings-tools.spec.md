---
pipeline_id: e3ef8f6e-1a1c-43bd-8f58-b397255038f3
ticket: docs/planning/tickets/open/TICKET-681-docs-and-settings-tools.md
status: Phase 4 — Complete PASS
title: Docs and settings tools on Marley's MCP server
type: feature
slice: prong 2 C (Marley's MCP server); phase 1 item 3 of docs/planning/intake/marley-agent-manager-foreman.md
references:
  - docs/planning/intake/marley-agent-manager-foreman.md
  - docs/planning/pipeline/completed/680-tool-results-fit.spec.md
---

## Title
Five read tools that let an agent explain and look up how Marley works and how it is set:
`docs_search`, `docs_read`, `settings_schema`, `settings_read` and `actions_list`. They are the
knowledge half of the Marley agent (TICKET-683), and any agent on Marley's MCP server gets them.

## Scope
### In
- **The docs bundle:** Zed's docs (`docs/src`, 197 Markdown pages) and Marley's guide
  (`docs/marley/guide.md`), shipped in the build through `util::fs_embed!` (read from the checkout
  in a dev build, as Zed's settings assets are). Zed's `{#kb action}` and `{#action action}`
  placeholders are filled in when read: the key bound in this Marley, and the action's palette
  name.
- **`docs_search`:** a query in words; the best sections (a page's text between two headings),
  at most 10: page, heading, a snippet around the first hit. Ranked by how many of the query's
  words a section holds, then by hits, a hit in the heading counting more.
- **`docs_read`:** a page by its path, or one section of it by `heading`, in #680's pages of
  12,000 bytes, run from the top since a document reads forward: `next` is the `after` that reads
  the rest.
- **`settings_schema`:** a key path (`terminal.font_size`); the setting's type, description,
  allowed values, default (from `default.json`), and for an object its keys with one line each.
  An empty key lists the top-level keys.
- **`settings_read`:** a key path; its value in each settings file that sets it, highest
  precedence first (each open project's `.zed/settings.json`, the user's settings, the defaults),
  which one wins, the value in effect (the winner's; for an object, the merged value), and
  `global`, the value where no project file sets it. A value under a name that reads as a secret
  is hidden.
- **`actions_list`:** a query in words; the actions whose name or documentation holds them, at most
  50: the action's name, its palette name, the first line of its documentation, and each key bound
  to it with the context it applies in.
- Codes for these tools' refusals (`no_doc`, `no_section`, `no_setting`, `too_large`,
  `bad_argument`), and a line about them in `initialize`'s instructions.

### Out (explicitly deferred)
- Changing settings or keys (TICKET-682).
- The Marley agent's entry and instructions (TICKET-683).
- A search better than words: an index or embeddings (`marley.knowledge` is a separate, off-by-
  default layer).
- Settings set by a profile, a release channel or an OS override block; the tools read the files
  `SettingsStore::get_all_files` lists.

## Reference (§20)
Upstream Zed (the `settings`, `json_schema_store`, `settings_ui`, `gpui` and `docs_preprocessor`
crates): the schema is Zed's own (`SettingsStore::json_schema`), "which file set this value" is
the Settings UI's question (`get_all_files`, `get_content_for_file`), the bindings are gpui's
keymap, and the placeholders are the ones Zed's docs preprocessor fills when it builds the site.
The tools read those as Zed builds them; nothing in a Zed crate changes.

### Prior art
- **Behavior maps:** `docs/t3code_architecture/06-orchestration-mcp-and-automations.md` §2.4
  (results near 20 KB, typed refusals) and #680's pager. No map covers a docs or settings tool;
  T3 has none.
- **Published material:** MCP 2025-06-18 tools (read-only, structured output with a text
  fallback); JSON Schema 2019-09 `$defs`/`$ref`, as schemars 1.0 writes it.
- **The code we ship** (an Explore read, 2026-10-07):
  - `crates/settings/src/settings_store.rs:1297` `json_schema(&SettingsJsonSchemaParams)`, an
    associated fn; the params' name lists may be empty (`:1209-1295`).
  - `:502` `raw_default_settings`, `:497` `raw_user_settings`, `:657` `get_all_files`, `:683`
    `get_content_for_file`, `:438` `merged_settings`; `SettingsContent` serializes.
  - `crates/util/src/util.rs:770` `fs_embed!` over rust-embed 8.11, used by
    `crates/settings/src/settings.rs:120`.
  - `crates/gpui/src/app.rs:2406` `all_action_names`, `:2448` `action_documentation`, `:2356`
    `key_bindings`; `keymap.rs:90` `bindings`; `binding.rs:104-124` keystrokes and predicate;
    `crates/ui/src/components/keybinding.rs:668` `text_for_keybinding_keystrokes(&[…], &App)`.
  - `crates/docs_preprocessor/src/main.rs:260` and `:308` fill the placeholders at build time.
  - `crates/command_palette/src/command_palette.rs:867` `humanize_action_name`. Marley's crate
    does not depend on `command_palette`, and its body is Zed's GPL code, so Marley writes its own
    from the rule (the namespace, then the name's words in lower case).
  - No docs search exists in the tree.

## UI proof
`script/e2e/681-docs-and-settings-tools.sh`, on the default hidden workspace (no clicks). The
stand-in agent of `browser-fixture.sh` calls each tool through the plugin's bridge with its
generic `tool` command, and the scenario checks the answers; the run's settings carry a known
`marley.rail_order`, the scratch project a `.zed/settings.json` with its own `tab_size`, and a
fake context server with a token in its environment. Shot: `681-01-after`, Marley still drawing
after the calls. The tools add nothing to see; the answers are the evidence.

## Locked-In Decisions
- D1 — The bundle is the docs of the build that runs, embedded in a release build, so an answer
  never describes another version.
- D2 — Search by words over sections, ranked by coverage then hits, no index: 1.4 MB of
  Markdown searches in milliseconds off the main thread.
- D3 — `docs_read` pages from the top (`after`), using #680's line pages; a document reads
  forward, unlike a block's output.
- D4 — Defaults come from `default.json` (the schema carries none); "which file wins" walks
  `get_all_files()` in its order, each file's content serialized and read at the key path, not
  the Settings UI's typed pickers.
- D5 — A settings value under a key whose name holds `token`, `secret`, `password`, `key`,
  `credential` or `auth` is shown as `[redacted: setting]`, and the whole answer then passes the
  agents' redactor.
- D6 — Marley's own palette-name rule, not a copy of Zed's function.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `docs_search` with "terminal font size", the system shall answer at most 10 sections, each with its page, heading and a snippet, a Zed docs page among the first three. | The stand-in's answer |
| REQ-002 | WHEN the query names a Marley feature ("rail filter"), the system shall answer a section of `marley/guide.md` among the first three. | The stand-in's answer |
| REQ-003 | WHEN an agent calls `docs_read` with a page and a heading, the system shall answer that section's text as a page with `first_line`, `last_line`, `total_lines` and `next` (the `after` for the rest), and no `{#kb` or `{#action` left in it. | `docs_read` of `configuring-zed.md`, the section holding `{#kb zed::OpenSettings}`: a key or "no key", no placeholder |
| REQ-004 | WHEN `docs_read` names a page the bundle lacks, the system shall refuse with code `no_doc` and a next step naming `docs_search`. | The stand-in's refusal |
| REQ-005 | WHEN an agent calls `settings_schema` with `marley.rail_order`, the system shall answer its description, its allowed values `attention` and `window`, and its default `window`. | The stand-in's answer |
| REQ-006 | WHEN `settings_schema` names a key the schema lacks, the system shall refuse with code `no_setting` and the keys there are at the deepest part that exists. | `settings_schema terminal.no_such_key`: `no_setting`, next steps listing terminal's keys |
| REQ-007 | WHEN an agent calls `settings_read` with a key the project's `.zed/settings.json` sets, the system shall answer the value in each file that sets it, project first, name the project file as the one that wins, and answer the project's value as the value in effect. | `settings_read tab_size`: project 3, then user (if set), then default 4; `set_in` the project file |
| REQ-008 | WHEN a value read sits under a name that reads as a secret, the system shall answer it as `[redacted: setting]`. | `settings_read context_servers.e2e-fake`: the env's `API_TOKEN` hidden, the fake token nowhere in the answer |
| REQ-009 | WHEN an agent calls `actions_list` with "save", the system shall answer `workspace::Save` with its palette name, its documentation's first line and its key bindings with their contexts. | The stand-in's answer holds `workspace::Save`, "workspace: save" and a `ctrl-s` binding |
| REQ-010 | The system shall keep every answer of these tools under 40,000 bytes on the wire. | The sizes the stand-in prints |
| REQ-011 | WHEN a client initializes, the instructions shall name the five tools, and stay under 2,048 bytes. | `instructions`: their names, size |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_mcp` (three families, five tool rows and schemas, the dispatch arm, the
  instructions), `marley_workbench` (`docs_tools.rs`, `settings_tools.rs`, the answer branches,
  `rust-embed`); a review of the diff; `just gate-diff` green.
- **P3 Test** — the scenario; read the answers and the shot.
- **P4 Complete** — CHANGELOG, the guide's tool tables, the architecture notes (§21), ledger,
  close, archive, commit, push, install.
