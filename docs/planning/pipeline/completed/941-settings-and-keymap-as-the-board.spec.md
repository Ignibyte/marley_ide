---
pipeline_id: 6cd08aa7-379c-483c-bba4-49b905e51478
ticket: docs/planning/tickets/open/TICKET-941-settings-and-keymap-as-the-board.md
status: Phase 4 — Complete PASS
title: Settings and the keymap as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/D-panels-editor.md, docs/marley/look-plan.md]
---

## Title
The Settings window and the keymap editor as the Settings and Keymap boards draw them: a tile
per section in the list, Marley first in the Layout choice, an "At the prompt" group, the "Agent
text full width" row and sentence-case titles on the Marley page; the keymap's column names,
Marley's own bindings marked Marley, a "Find by keys" text button and the record dialog's words.

## Scope
### In
- **Tiles in the section list** (Zed crate hunks: `crates/ui/src/components/tree_view_item.rs`,
  a `start_slot`; `crates/settings_ui/src/settings_ui.rs`, `render_nav`). Each root entry gets a
  small rounded tile with its letters before its name, as the board: M Marley, G General, Ap
  Appearance, Km Keymap, Ed Editor, Ln Languages & Tools, Sf Search & Files, Wl Window & Layout,
  Pn Panels, Tm Terminal, Vc Version Control, Db Debugger, AI AI, Co Collaboration, Nw Network,
  Dv Developer; Marley's tile in the accent colours. A section with no letters listed gets its
  first letter. The chevron stays.
- **Marley first** (Zed crate path, `crates/settings_content/src/marley.rs`, `MarleyLayout`). The
  variants are listed `Marley`, then `Zed`, so the buttons read Marley | Zed. `Marley` stays the
  default; stored values do not change.
- **"At the prompt"** (Zed crate path, `crates/settings_ui/src/marley_page.rs`). Prompt editor,
  English at the prompt and Follow Omarchy's theme leave Layout for a new section "At the prompt"
  right after it.
- **"Agent text full width"** (`marley_page.rs`; `settings_ui.rs`, `marley_row`). A switch in
  Layout: "An agent's answer runs the width of its tab. Off keeps Zed's readable column." It shows
  and writes `agent.limit_content_width`, inverted (on writes `false`). It draws with the Marley
  look (key line, "changed · reset") although its key is Zed's: a row on the Marley page draws the
  Marley look.
- **Sentence case** (`marley_page.rs`). Every row title on the Marley page in sentence case ("Rail
  order", "Prompt editor", "Redact secrets for agents"); names and acronyms keep their capitals
  (Claude Code, Codex, Omarchy, MCP, PIN, URL, System One, Rusty).
- **The keymap's columns** (Zed crate, `crates/keymap_editor/src/keymap_editor.rs`, the table's
  header at `:2149`). Action, Arguments, Keys, Where, From.
- **Marley's bindings marked Marley** (Zed crate, `crates/settings/src/keymap_file.rs`,
  `KeybindSource`; Marley crate, `marley_workbench.rs` `load_keymap_from`; `keymap_editor.rs`).
  A new source `Marley`, with the same precedence against the other sources as Marley's bindings
  have today (D3). Marley's keymap is bound with it. The From column reads Marley for it, Yours for
  the user's, Zed for Zed's defaults, and Base and Vim as now. Editing a Marley binding works as
  editing a Zed default does (it writes an override to the user's keymap). The source filter
  counts Marley's bindings with the defaults.
- **"Find by keys"** (`keymap_editor.rs`, `:2065-2095`). The keystroke-search icon toggle becomes a
  text button "Find by keys", pressed while keystroke search is on; same action and tooltip.
- **The record dialog's words** (`keymap_editor.rs`, `KeybindingEditorModal`, `:3098-3125`,
  `:2838-2850`, `:3206-3209`). The header reads "New keys for <action>" (the action's name as the
  table shows it), or "New keys" when creating. A conflict reads "Also <action>, in <where>" (with
  "and N more" when there are more). The save button reads "Save Anyway" while the keys conflict,
  else Save.

### Out (explicitly deferred)
- Zed's other settings pages as cards: deferred by #844 ("Other pages keep Zed's look").
- The board's "You | This project" scope tabs: deferred by #844 (Zed's file picker stays).
- Controls for the 17 list and map keys: deferred by #844 (they stay in settings.json).
- Dropping the Arguments column: the board shows none, but an action's arguments are what tells
  two bindings of one action apart; removing a column would rewrite Zed's table code (D2).
- Geist fonts (the board's type): the review's "Fonts" item waits for Chad's word.

## Reference (§20)
Upstream Zed: `settings_ui`'s Settings window (its navbar `TreeViewItem`s, `render_settings_item`,
#844's Marley look) and `keymap_editor` (the keymap table, `SearchMode::KeyStroke`,
`KeybindingEditorModal`'s conflict check), with `settings::KeybindSource` and gpui's keymap
precedence by meta. The behaviour kept: every binding, filter, search and edit works as in Zed;
only words, one source and the list's tiles change.

### Prior art
- `crates/settings_ui/src/settings_ui.rs:3305-3330`: the nav entries are `TreeViewItem`s with the
  section's title and a disclosure for roots; `crates/ui/src/components/tree_view_item.rs:28-113`
  has no start slot (Marley already touched it for the focus ring, #895, `:184`).
- `settings_ui.rs:5285-5330`: #844/#847's segmented buttons list `VariantArray` in its order;
  `crates/settings_content/src/marley.rs:1061-1067` `MarleyLayout { Zed, Marley }`, the only Marley
  enum with a Marley variant.
- `crates/settings_ui/src/marley_page.rs:59-238` `layout_section` holds Layout, Rail Order, Prompt
  Editor, English at the Prompt and Follow Omarchy's Theme (among nine items); 93 titles in Title
  Case (`grep -c 'title: "'`).
- `settings_ui.rs:1541-1544` `marley_row` is true only for `marley.` keys;
  `crates/settings_ui/src/page_data.rs:7069` Zed's own `agent.limit_content_width` row;
  `crates/settings_content/src/agent.rs:277` the setting.
- `crates/keymap_editor/src/keymap_editor.rs:2149` the header `["", "Action", "Arguments",
  "Keystrokes", "Context", "Source"]`; `:236-247` `SourceFilters::allows`; `:862-867` each
  binding's source from its meta; `:2345-2350` the override tooltip by source; `:2065-2095` the
  keystroke-search `IconButton`; `:2838-2850` the conflict sentence; `:3098-3125` the dialog's
  header; `:3206-3209` Save.
- `crates/settings/src/keymap_file.rs:1406-1450` `KeybindSource { User, Vim, Base, Default,
  Unknown }`, its discriminant used as the meta index; `crates/gpui/src/keymap.rs:222-236` a
  smaller meta is the stronger source when a `NoAction` binding hides others.
- `crates/marley_workbench/src/marley_workbench.rs:1266-1295` `load_keymap_from` tags every Marley
  binding `KeybindSource::Default`, bound after Zed's defaults so it wins at the same depth.
- Specs: #844 (the Marley look, its deferrals), #847 (four buttons), #895 (the tree item's focus
  ring).

## UI proof
Scenario `script/e2e/941-settings-and-keymap-as-the-board.sh` (sway). Setup: a scratch
repository and a fresh profile; no agent and no Rusty. The keymap's shots search for a Marley
action (for example "filter block") and for one action bound in the user's keymap, written into
the run's `keymap.json` by the setup.
- `941-01-settings`: `marley: open settings`: the section list with its tiles, Layout as Marley |
  Zed, Rail order, Agent text full width, the "At the prompt" card, sentence-case titles (REQ-001
  to REQ-005).
- `941-02-full-width`: Agent text full width switched on: the row shows "changed · reset", and the
  run's `settings.json` holds `"limit_content_width": false` (printed to the log) (REQ-004).
- `941-03-keymap`: `zed: open keymap`, searched for a Marley action and the user's: the column
  names, From reading Marley and Yours, the "Find by keys" button (REQ-006, REQ-007, REQ-008).
- `941-04-find-by-keys`: Find by keys clicked: pressed, the search by keys on (REQ-008).
- `941-05-record`: a Marley binding opened for new keys, keys typed that another binding holds:
  "New keys for …", "Also …, in …", Cancel and Save Anyway (REQ-009).
- `941-06-day`: the Settings window and the keymap in Marley Day (REQ-001 to REQ-003, REQ-006,
  REQ-007).

## Locked-In Decisions
- D1 — The section tiles are drawn for every section, Zed's too: the list is shared, and the
  board draws them all. The pages themselves keep Zed's look (#844).
- D2 — The keymap keeps Zed's Arguments column; only the names change (Out).
- D3 — `KeybindSource::Marley` sits between `Base` and `Default`, so a Marley binding beats a Zed
  default as it does today, loses to the user's and the base keymap's, and a Zed default's
  `NoAction` no longer hides a Marley binding. Phase 2 checks Marley's keymap and Zed's default
  keymap for `null` bindings on the same keys before relying on it.
- D4 — Reordering `MarleyLayout`'s variants changes only the buttons' order; serde names and the
  default are unchanged.
- D5 — "Agent text full width" writes Zed's own key, inverted; it adds no Marley setting.
- D6 — Sentence case applies to the Marley page's row titles only; Zed's pages keep their titles.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Settings window's section list shall show a lettered tile before each section's name, Marley's in the accent colours. | `941-01-settings`, `941-06-day` |
| REQ-002 | The Layout choice shall read Marley | Zed, and a stored `marley.layout` shall keep its meaning. | `941-01-settings`, `941-06-day`, the review of the diff |
| REQ-003 | The Marley page shall show Prompt editor, English at the prompt and Follow Omarchy's theme in an "At the prompt" section after Layout. | `941-01-settings`, `941-06-day` |
| REQ-004 | The Marley page's Layout section shall have an "Agent text full width" switch that shows the inverse of `agent.limit_content_width`, writes it when changed, and draws with the Marley look. | `941-01-settings`, `941-02-full-width` |
| REQ-005 | The Marley page's row titles shall be in sentence case, names and acronyms keeping their capitals. | `941-01-settings` |
| REQ-006 | The keymap editor's columns shall read Action, Arguments, Keys, Where and From. | `941-03-keymap`, `941-06-day` |
| REQ-007 | The keymap editor's From column shall read Marley for Marley's own bindings, Yours for the user's and Zed for Zed's defaults. | `941-03-keymap`, `941-06-day` |
| REQ-008 | The keymap editor shall offer "Find by keys" as a text button that toggles search by keys and shows pressed while it is on. | `941-03-keymap`, `941-04-find-by-keys` |
| REQ-009 | WHEN the user records keys another binding holds, the dialog shall read "New keys for <action>", "Also <action>, in <where>", and offer Cancel and Save Anyway. | `941-05-record` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `tree_view_item.rs`, `settings_ui.rs`, `marley_page.rs`, `settings_content`'s
  `marley.rs`, `keymap_file.rs`, `keymap_editor.rs` (Zed crate paths, each with its ledger row and
  a `// Marley:` comment on each code hunk) and `marley_workbench.rs`; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
