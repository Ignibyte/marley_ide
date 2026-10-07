# Docs and settings tools on Marley's MCP server — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-681-docs-and-settings-tools.md
- **Pipeline spec:** 681-docs-and-settings-tools.spec.md

## Phase 1 — Plan
- **Request:** the queue's top, phase 1 item 3 of the Marley-agent plan; running on Chad's
  "just have it go" (2026-10-07).
- **Classification / tier:** feature, prong 2 C. Marley crates only; one new workspace dependency
  in `marley_workbench` (`rust-embed`, already in the workspace).
- **Checklist (no task tool offered):** pick ✓ · pre-flight ✓ (no active pipeline, README marker,
  cargo idle) · recall ✓ · mint ✓ · prior art ✓ (an Explore read) · spec ✓ · design ✓.
- **Recall (§18.3):**
  - #680's pager (`Page`, `page_ending`, `PAGE_BYTES`) and refusal codes are the shapes these tools
    answer in.
  - `F-claude-516-…`/PR-claude-redact-the-whole-text-before-cutting-it-001: redact before any cut;
    settings answers are redacted whole before the size check.
  - The dylint `blocking_io_on_foreground` holds in Marley crates: a dev build's `fs_embed!` reads
    files, so the bundle loads and searches off the main thread.
  - Nothing in the ledger on docs or settings tools; the brain: #680's consultation covered the
    plan, and this ticket decides only D1 to D6, recorded at Complete.
- **Discovery:** the Explore read's citations are in the spec's prior art. In `marley_mcp`, a family
  is a `Family` variant (`registry.rs:15-50`), `REGISTRY` rows (`:88-373`), a `tool_schemas` arm
  (`:462-482`), and the deferred arm in `dispatch.rs:230-245`; the registry test at `:1746` counts
  31 tools. In `marley_workbench`, `mcp.rs` `answer` branches by prefix.

### Design

**Approach.**

1. **`marley_mcp`.** Three families, `Docs` (`docs_search`, `docs_read`), `Settings`
   (`settings_schema`, `settings_read`) and `Actions` (`actions_list`), served, all `Tier::Read`
   with no grant class, deferred to the app like the terminal family. Their schemas in
   `docs_schemas`, `settings_schemas`, `actions_schemas`. `INSTRUCTIONS` gains one sentence:
   "Questions about Marley or Zed, how a feature works, what a setting does or is set to, or
   which key runs a command: docs_search and docs_read, settings_schema and settings_read,
   actions_list." The registry test's list and count follow.
2. **`docs_tools.rs`** (Marley crate, new module). `util::fs_embed!` over `../../docs` with
   `include = ["src/**/*.md", "marley/guide.md"]`. A `DocsIndex` built once, off the main thread,
   in a `OnceLock`: each page split into sections at its headings (`#` to `####`), each with its
   page path (`zed/<path under src>` or `marley/guide.md`), heading, slug, and line range. Search:
   the query's words (lower case, two letters or more, a short stop list out); per section the
   number of words it holds and its hits (a heading hit counting five); the top 10 by
   (words held, hits), each with a snippet of about 200 characters around the first hit. Read: the
   page or the section by heading (case-insensitive, the slug accepted), placeholders filled, then
   `page_from(text, after)`, #680's line pager run forward. Placeholders: `{#kb name}` becomes the
   highest-precedence binding's keystrokes (`ui::text_for_keybinding_keystrokes`) or "no key";
   `{#action name}` becomes Marley's palette name. The bindings are read on the main thread into
   a name-to-text map before the background work.
3. **`settings_tools.rs`** (Marley crate, new module).
   - `settings_schema`: `SettingsStore::json_schema` with the action names, documentation and
     deprecations from `cx`, the other lists empty; the key path resolved through `$ref`
     (`#/$defs/…`), `anyOf`/`oneOf`/`allOf` (the non-null branch) and `properties`. The answer:
     key, type, description, `enum` or the variants' `const` values, default read at the same path
     from `raw_default_settings` serialized, and for an object its property names with each
     description's first line. Unknown segment: `no_setting`, next steps listing the keys at the
     deepest part found (at most 40). Empty key: the top-level keys.
   - `settings_read`: for each `SettingsFile` in `get_all_files()` order, its content
     (`get_content_for_file`) serialized and read at the key path; each file named (`project
     <worktree root>/.zed/settings.json`, `user <path>`, `default`); `set_in` the first that has
     the key; `effective` from `merged_settings`. Values masked by D5, then the whole JSON through
     `agent_redactor`. Over 12,000 bytes: `too_large`, with the child keys to name instead.
   - `actions_list`: `all_action_names` with `action_documentation`; the query's words matched in
     the name, the palette name and the documentation; at most 50, by words held then name; each
     with its bindings from `key_bindings().borrow().bindings()` filtered by the action's name
     (keystrokes as text, the context predicate as text, highest precedence first).
4. **`mcp.rs`.** The answer branches: `docs_` to `docs_tools::answer` (spawned, like
   `ports_list`), `settings_` and `actions_` to `settings_tools::answer`. #680's `Page` gains
   `pub(crate) fn page_from(text, after)` beside `page`.
5. **The stand-in** needs nothing new: its `tool <name> <json>` prints the structured answer; the
   scenario adds `instructions` checks and a size check through a small Python wrapper of its own.

**File manifest.**

| File | Crate | Change |
|---|---|---|
| `crates/marley_mcp/src/registry.rs` | Marley | three families, five rows, schemas, the test's list |
| `crates/marley_mcp/src/dispatch.rs` | Marley | the deferred arm; the instructions' sentence |
| `crates/marley_workbench/Cargo.toml` | Marley | `rust-embed.workspace = true` |
| `crates/marley_workbench/src/docs_tools.rs` | Marley | new: the bundle, the index, search, read |
| `crates/marley_workbench/src/settings_tools.rs` | Marley | new: schema, read, actions |
| `crates/marley_workbench/src/marley_workbench.rs` | Marley | the two modules |
| `crates/marley_workbench/src/mcp.rs` | Marley | the answer branches; `page_from` |
| `script/e2e/681-docs-and-settings-tools.sh` | script | the scenario |

### Visual check plan

| REQ | Setup and action | Evidence |
|---|---|---|
| 001 | `tool docs_search {"query": "terminal font size"}` | ≤ 10 results, a `zed/` page in the first three |
| 002 | `tool docs_search {"query": "rail filter"}` | `marley/guide.md` in the first three |
| 003 | `tool docs_read {"page": "zed/configuring-zed.md", "heading": "Settings Editor"}` (the section with `{#kb zed::OpenSettings}`; the heading confirmed from the file at Code) | the section's text, the page fields, no `{#kb`/`{#action` |
| 004 | `tool docs_read {"page": "zed/nope.md"}` | `no_doc`, next step `docs_search` |
| 005 | `tool settings_schema {"key": "marley.rail_order"}` | description, `attention`, `window`, default `window` |
| 006 | `tool settings_schema {"key": "terminal.no_such_key"}` | `no_setting`, terminal's keys |
| 007 | the project's `.zed/settings.json` `{"tab_size": 3}`; `tool settings_read {"key": "tab_size"}` | project 3 first, default 4, `set_in` the project file |
| 008 | the run's settings add `context_servers.e2e-fake` with `env.API_TOKEN`; `tool settings_read {"key": "context_servers.e2e-fake"}` | `[redacted: setting]`, the fake token absent |
| 009 | `tool actions_list {"query": "save"}` | `workspace::Save`, "workspace: save", a `ctrl-s` binding |
| 010 | every call above | the largest answer under 40,000 bytes |
| 011 | `instructions` | the five names, ≤ 2,048 bytes |

Shot `681-01-after`: Marley drawing after the calls.

### Risks
- **The schema's shape.** schemars may wrap an `Option<Enum>` as `anyOf [ $ref, null ]` and an
  enum's variants as `oneOf` of `const`s with descriptions; the resolver handles both, and the
  scenario's `rail_order` shows which.
- **A dev build reads the docs from the checkout** (`fs_embed!`'s dev mode); the scenario runs the
  debug build in the checkout, so it reads them; a release build embeds them.
- **Search quality** is words, not meaning: a query in other words misses. The answer says how many
  sections matched, and `docs_read` of a page's top lists its headings.

## Phase 2 — Code
- **Built:**
  - `marley_mcp`: `Family::{Docs, Settings, Actions}`, served; five `Tier::Read` rows; schemas in
    `docs_schemas`, `settings_schemas`, `actions_list_schemas`; the deferred arm in `dispatch.rs`;
    one paragraph in `INSTRUCTIONS` naming the five; the registry test's list (it had also missed
    `editor_open` and `editor_wait`) and count, 38.
  - `marley_workbench/src/docs_tools.rs`: `util::fs_embed!` over `docs/src/**/*.md` and
    `docs/marley/guide.md`; `DocsIndex` in a `OnceLock`, pages split at headings `#` to `####`
    outside code fences; `search` (words held, then hits, a heading hit counting five), `snippet`,
    `read` (page or section, the slug accepted), `fill_placeholders`, all on the background
    executor (`futures::future::lazy`, the crate's way past `async_block_without_await`).
  - `marley_workbench/src/settings_tools.rs`: `query_words`, `palette_name` (Marley's own rule),
    `bound_keys` (the keymap, highest precedence first); `schema` (`SettingsStore::json_schema` on
    the background executor with the actions' names and documentation), `describe`, `resolve`
    (`$ref` into `$defs`, an optional's null branch dropped), `allowed_values`; `read` with
    `hidden` (D5) and `redact_strings`; `actions_list`; `file_name` for each `SettingsFile`.
  - `mcp.rs`: the `docs_` and `settings_`/`actions_` branches; `page_from`, `Page::next`,
    `fill_forward`, `text_block_forward` (#680's pager run forward).
  - `browser-fixture.sh`: the generic `tool` command prints the answer's size. The scenario,
    written in this phase so the gate's fingerprint covers it.
- **Deviations:**
  - No `rust-embed` line in `marley_workbench/Cargo.toml`: `fs_embed!` reaches rust-embed through
    `util`'s re-export.
  - `settings_read` answers `global` beside `effective`. The first scenario run answered
    `effective: 4` for a `tab_size` the project set to 3: `merged_settings` leaves project files
    out, since they apply only inside their project. `effective` is now the winning file's value
    (the merged one for an object), and `global` is the merged value. A real bug, found by reading
    the answers before the gate; an `F-` block at Complete.
  - `actions_list` ranks the shortest name first among those whose name holds every word: "save"
    had listed `editor::SaveLocation` before `workspace::Save`.
  - The scenario runs under `compositor sway`: on Hyprland the runner refuses to start beside an
    open Marley (Chad's).
- **Review of the diff:** placeholders' indices (`open`, `open + close`) and the unknown-kind
  passthrough; a secret read by its own key path (`env.API_TOKEN`) was not masked, since the top
  value had no name: `read` now passes the key's last name in (found in review, before the first
  run). `page_from` refuses an `after` at or past the end; a line longer than a page keeps its start.
  No entity is updated; the schema and the docs work run off the main thread; the keymap and the
  settings are read on it.
- **Checks before the gate:** `just clippy marley_mcp marley_workbench` clean after four lints
  (two first-paragraph doc lengths, two `map_or`/`map_or_else`); the scenario green, 15 of 15,
  twice (`scratchpad/681-e2e-3.log` after the fixes).
- **Gate:** `just gate-diff` green, 17 of 17, receipt written (`scratchpad/681-gate-1.log`).

## Phase 3 — Test
- **Scenario:** `script/e2e/681-docs-and-settings-tools.sh` (`compositor sway`), on the debug build
  with the fixes. Run 4, the Test phase's: 15 of 15 checks (`scratchpad/681-e2e-4.log`); runs 2
  and 3 in Code found the `effective` bug and the ranking, as the Code entry says. Run 1 never
  started: the Hyprland runner refuses beside an open Marley.
- **What the stand-in read:**
  - REQ-011: instructions 1,225 bytes, holding the five names.
  - REQ-001: "terminal font size" matched 387 sections; first `zed/visual-customization.md`
    "Fonts", whose snippet shows the terminal's `font_family` and `font_size` keys.
  - REQ-002: "rail filter" matched 77; first `marley/guide.md` "The rail".
  - REQ-003: `zed/configuring-zed.md` "Settings Editor": the text reads "The **Settings Editor**
    (Ctrl-,)" and "Or run zed: open settings from the command palette", no placeholder left,
    `first_line` 1, `next` null.
  - REQ-004: `zed/nope.md` is `no_doc`, next step `docs_search`.
  - REQ-005: `marley.rail_order` is "one of", described "The order the rail lists projects and rows
    in (#542).", values `attention` and `window` each with its line, default `window`.
  - REQ-006: `terminal.no_such_key` is `no_setting`, next step listing terminal's keys from `shell`
    to `font_size` and on.
  - REQ-007: `tab_size` effective 3, global 4, `set_in` the scratch project's
    `.zed/settings.json`, values project 3 then default 4.
  - `marley.rail_order` read: effective and global `attention`, set in the run's user settings,
    default `window`.
  - REQ-008: `agent_servers.e2e-fake` shows `API_TOKEN` as `[redacted: setting]`; read by its own
    path, the same; the fake token is in neither answer.
  - REQ-009: "save" matched 12; first `workspace::Save`, "workspace: save", "Saves the current file
    with the specified options.", keys `Ctrl-S` (Workspace) and `Save` (Workspace).
  - REQ-010: the largest answer 6,206 bytes.
- **Shot** `681-01-after` (Marley only): the scratch project `repo` with its `.zed` folder in the
  project panel and its terminal, drawing after every call. The tools add nothing to see.
- **Focus report:** "1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it"; the run's sway, Marley, pointer and keyboard stopped with it.

## Phase 4 — Complete
- **Docs (§21):** `CHANGELOG.md` (Added, #681). `docs/marley/guide.md`: a table for the docs,
  settings and actions families and their refusal codes. `crates/marley_workbench/guide/index.html`:
  the same table, shorter. `docs/marley_architecture/marley_mcp.md` (the families) and
  `marley_workbench.md` (`docs_tools.rs`, `settings_tools.rs`, the forward pager). The plan doc marks
  phase 1 item 3 done. No Zed crate changed, so no touchpoint row.
- **Knowledge (§19):** `F-claude-681-merged-settings-read-as-the-value-in-effect-skipped-the-project-001`
  (failures) with `PR-claude-681-a-settings-value-in-effect-counts-the-project-files-001`
  (prevention rules); `L-claude-681-a-new-scenario-runs-in-its-own-sway-while-chad-uses-marley-001`
  (lessons); `AD-claude-681-agents-read-marleys-docs-and-settings-as-the-build-ships-them-001`
  (decisions). Brain: consultation 57651f5a371f4e52807ce42414a5aea2 closed with `brain decide`
  (`decisions/agents-read-marleys-docs-and-settings-as-the-build-ships-them`, follow-up by
  2026-10-28).
- **Ticket:** TICKET-681 closed; it left `BACKLOG.md` at promotion.
- **Gate:** the in-app guide is in the receipt's fingerprint, so the gate ran again before the
  commit.
