# The Memory tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-664-rusty-memory-tab.md
- **Pipeline spec:** 664-rusty-memory-tab.spec.md

## Phase 1 — Plan (2026-10-06)
- **Request:** the Queue's top after #663, from Chad's 2026-10-06 "adding the last things missing".
- **Classification / tier:** feature, R8's first part; Marley crates only.
- **Pre-flight:** green; no active pipeline; cargo idle.
- **Recall (§18.3):**
  - Nothing in the ledgers on memories.
  - AD-659 and #659's tab: a center tab over one Rusty read, `Link` and `ReadDue` (L-658: only a
    change of the link counts), read again when shown after a hidden announcement (AD-609).
  - #660's form: a modal, `menu::Confirm` through a `… menu` key context, a refusal kept in it.
  - AD-662, AD-663: read again after Marley's own writes; the service connection announces nothing.
  - The brain (consultation `007d14b78dd348debbefc56149ffb863`): nothing on this seam.
- **Discovery:**
  - Rusty at `eb1ab51`: `store_memory { content, category?, importance? }` answers the new id
    (default category `fact`, importance `normal`); `update_memory { id, content?, category?,
    importance? }` answers the memory; `delete_memory { id }` answers `deleted` or refuses
    `Memory not found: <id>`; `list_memories { category? }` answers `[Memory]` (`id`, `category`,
    `importance`, `content`, `type`, `source`, `created_at`, `updated_at`, seconds), high first,
    then normal (and any other word), then low, newest first. `importance_word` takes `low`,
    `normal`, `high` and `medium` (as `normal`), refusing others with `importance is low, normal or
    high, not "<word>"`. Every write emits `DataChanged`, which Rusty announces.
  - Rusty's app (`MemoryPage.qml`): `context` as the add's default category; the categories sorted;
    the filter shows rows of one category; Delete asks "Forget this?".
  - Marley: `decisions_tab.rs` (650 lines) as the model; the Brain view's header buttons
    (`brain.rs` `render_header`, `open_decisions`); `rusty.rs`'s `DropdownMenu`; `chrono` in
    `marley_workbench` for the local day.
  - The stand-in: `TASK_WRITES` announce after each success; its watcher's stamp covers
    `SETTINGS` and `BOOKMARKS` and the vault.

### Design
- **`marley_rusty/src/memories.rs`** (Marley): `LIST_MEMORIES`, `STORE_MEMORY`, `UPDATE_MEMORY`,
  `DELETE_MEMORY`; `Memory` (`Deserialize`, defaults); `memories_from_answer`; `Importance { Low,
  Normal, High }` with `ALL`, `word`, `label` and `parse`; `categories(&[Memory])` (sorted, once
  each); `count_line(n)`; `DEFAULT_CATEGORY` (`context`); `MemoryWrite { Store { content, category,
  importance }, Update { id, content, category, importance }, Delete { id } }` with `tool` and
  `arguments` (an empty category stored as `context`).
- **`rusty/memory_tab.rs`** (new, Marley): `OpenMemory`; `open`, `open_later`; `BrainMemoryView`
  (the Decisions tab's `Link`, `ReadDue`, `read`, `take_read`, `showing`, `deactivated`), with the
  add row (content editor, category editor, importance toggle group), the Category dropdown
  (`DropdownMenu` over a `ContextMenu` built from `categories`), the rows (`ListItem`: the content
  clamped to three lines, then a `Chip` for the category, the importance, the source, the day from
  `chrono::Local`), and `write(&MemoryWrite)` (call, then read again; a refusal toasts). The form
  `MemoryForm` (`ModalView`, `RustyMemory menu`): an auto-height content editor, a category
  editor, the importance group, Delete, Save (disabled while the content is empty or a call runs);
  Delete asks with `window.prompt` and on yes deletes; Save updates; each success dismisses and has
  the tab read again; a refusal stays in the form.
- **`brain.rs`**: a Memory button (`IconName::Book`) after Decisions, `open_memory`.
- **The stand-in**: `memories.json` in the state folder; the four tools with Rusty's order,
  importance words, defaults and refusals; the writes in the announced set; the watcher's stamp
  covers the file.
- **`rusty.rs`**: `mod memory_tab;` and its `init`.
- **The guide page**: a Memory article.
- **File manifest:** `crates/marley_rusty/src/{memories.rs, marley_rusty.rs}`, the stand-in,
  `crates/marley_workbench/src/rusty.rs`, `crates/marley_workbench/src/rusty/{memory_tab.rs,
  brain.rs}`, the guide page (all Marley); `script/e2e/664-rusty-memory-tab.sh` (Test). No Zed
  crate.

### Visual check plan
| Criterion | What the scenario does | Shot |
|---|---|---|
| REQ-001, 002 | Palette: `rusty: open memory` | `664-01-memory` |
| REQ-003 | Type a line, a category, click High, Enter | `664-02-added` |
| REQ-004 | Category menu, `preference` | `664-03-filter` |
| REQ-005 | Category menu, All; click a row | `664-04-form` |
| REQ-006 | Change the content, click Low, Save | `664-05-saved` |
| REQ-007 | The stand-in's `fail` file for `update_memory`; open a row, Save | `664-06-refused` |
| REQ-008 | Delete, then Delete in the prompt | `664-07-delete-prompt`, `664-08-deleted` |
| REQ-009 | Rewrite `memories.json` from outside | `664-09-outside` |
| REQ-010 | Turn Rusty off in the run's settings | `664-10-off` |
| REQ-001 (button) | Not shot separately | Review: the Brain view's button calls the same `open_later` |

### Risks
- **A memory's content can be long**; rows clamp it to three lines and the form shows all of it.
- **A category typed with spaces**: sent trimmed; Rusty keeps what it gets.
- **`importance` words outside the three** (stored before TICKET-052): shown as written, and the
  form's toggle shows none chosen until one is picked.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall, the brain; discovery.
- [x] Mint the pair; the BACKLOG row removed; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request ("continue on tickets until finished").

## Phase 2 — Code (2026-10-06)
### Built
- **`marley_rusty/src/memories.rs`**: the four tool names, `DEFAULT_CATEGORY` (`context`),
  `Memory` (`importance()`), `memories_from_answer`, `Importance` (`ALL`, `word`, `label`, `parse`
  with `medium` as normal), `categories`, `count_line`, `category_or_default`, and `MemoryWrite`
  (`Store`, `Update` with the importance left out when none is picked, `Delete`) with `tool` and
  `arguments`.
- **`rusty/memory_tab.rs`**: `OpenMemory`; `open` and `open_later`; `BrainMemoryView` with the
  Decisions tab's `Link` and `ReadDue` (now `pub(super)` there), the add row (content and category
  editors, Low, Normal, High; `menu::Confirm` in `RustyMemoryAdd menu` stores and empties the
  content field), the count and the Category `DropdownMenu` (a `ContextMenu` keyed by the
  categories and the choice), the rows (`ListItem`: the content in up to three lines, a `Chip`
  for the category, then importance, source and the local day), and the off, not-connected,
  failure and empty lines. `MemoryForm`: content (auto-height), category, the importance group,
  Delete (asked through `window.prompt`) and Save (waits for content); a success has the tab read
  again and closes, a refusal stays.
- **`brain.rs`**: a Memory button (`IconName::Book`) after Decisions.
- **The stand-in**: `memories.json` in the state folder; `list_memories`, `store_memory`,
  `update_memory`, `delete_memory` with Rusty's order, defaults, importance words and refusals;
  each write announced, and the file watched. A smoke run over stdio passed each.
- **The guide page**: a Memory article.

### Deviations
- `MemoryWrite::Update` takes an `Option<Importance>`: a memory holding an older word keeps it
  until one of the three is picked, where Rusty's app would have sent `low`.
- A refused add shows in a toast; the add row has no line of its own for one.

### Review
- Clippy: `redundant_pub_crate` (`pub(super)` in the private module), `use_self`, `unused_self`
  (`render_row` an associated function), needless `&mut` on `add`, `open_form` and `delete`.
  `unused_results` would have caught `Map::insert`, so the update's arguments are built from a
  list.
- Re-entrancy: the form opens through `window.defer`, outside the tab's update; the menu's entries
  update the tab from the menu's own update; the form updates the tab from its spawned task.

### Gate
`just gate-diff`: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-06)
Scenario `script/e2e/664-rusty-memory-tab.sh` under `compositor sway`: the stand-in with five
made-up memories in `memories.json` (two high, two normal, one low; preference, fact, context).
Four runs; the last passed every check.

### What each shot shows (last run)
- `664-01-memory` (REQ-001, 002): the title and Rusty's line, the add row (an empty field,
  `context` as the category's hint, Normal chosen), "5 memories", Category: All, and the five in
  Rusty's order: Release notes and Prefers plain words (high), Working on the shell and The dev box
  (normal), The old app (low), each with its category chip, importance, source and day.
- `664-02-added` (REQ-003): "Keeps commit subjects under sixty characters", preference, high, mcp,
  today, at the top; the field empty, the category and High kept; "6 memories".
- `664-03-filter` (REQ-004): Category: preference, "2 memories", the two preference rows.
- `664-04-form` (REQ-005): the form over Working on the shell: its text, `context`, Normal.
- `664-05-saved` (REQ-006): the row reads "…and Rusty this month", low, today, now last.
- `664-06-refused` (REQ-007): with the stand-in refusing, "database is locked" in red in the form,
  its text and choices kept.
- `664-07-delete-prompt` (REQ-008): "Forget this memory?", the memory's line, Delete and Cancel.
- `664-08-deleted` (REQ-008): The old app gone, "5 memories", the form closed.
- `664-09-outside` (REQ-009): "Added from Rusty's CLI" (high, cli) second, after the file changed
  from outside.
- `664-10-off` (REQ-010): "Rusty is off. Turn it on in the Rusty section of the Marley settings.",
  nothing listed.
- REQ-001's button by review: the Brain view's Memory button calls the same `open_later`.

### Fixes, each rebuilt and run again
- **The add row's field was a sliver** (first run, `664-01`): `ToggleButtonGroup` fills its parent,
  so in the row it took the space the content field's `flex_1` should have had, and the typed line
  went nowhere. The group now sits in a fixed 16 rem box and the category in 10 rem; the form's
  group got the same treatment.
- **The scenario's steps**: the Category menu opens on the current choice, so going back to All
  needs Home first; the form's buttons sit at y 313; after the save the edited memory sorts last,
  so the delete aims at row 4.

The gate runs on the final tree at Complete, after the fix and the docs.

## Phase 4 — Complete (2026-10-06)
- **Docs:** `CHANGELOG.md` (Added); `docs/marley/rusty-in-marley.md` (the screens row, R8, the
  batch line); `docs/marley/guide.md` (The Memory tab); `docs/marley/walkthrough.md` (stop 2.15f,
  `rusty: open memory` in Appendix B); `docs/marley_architecture/marley_rusty.md` (`memories`, the
  stand-in's memory tools); `docs/marley_architecture/marley_workbench.md` (The Memory tab). No
  Zed crate touched, so no zed-touchpoints row.
- **Knowledge:** F-claude-664-a-toggle-group-squeezed-the-field-beside-it-001,
  L-claude-664-toggle-button-group-needs-a-box-of-its-own-001,
  AD-claude-664-the-memory-tab-follows-rustys-app-001.
- **Brain:** `brain decide` on consultation `007d14b78dd348debbefc56149ffb863`, follow up by
  2026-11-06: `decisions/marleys-memory-tab-follows-rustys-app`.
- **Closed:** the ticket in `tickets/closed/`; this pair in `completed/`.
- **Final gate:** the first run went red on gate:18, typos reading the scenario's last grep
  (`memor`) as a misspelling; the scenario only joins the gate's scope at Complete. It now names
  the four tools, and the gate ran again.
