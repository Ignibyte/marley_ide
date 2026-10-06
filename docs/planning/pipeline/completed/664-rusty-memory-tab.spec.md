---
pipeline_id: e55656b2-6be2-47ca-98dd-a1cba95cff46
ticket: docs/planning/tickets/open/TICKET-664-rusty-memory-tab.md
status: Phase 4 — Complete PASS
title: "The Memory tab"
type: feature
slice: Rusty in Marley R8 (docs/marley/rusty-in-marley.md)
references: [docs/marley/rusty-in-marley.md, docs/planning/pipeline/completed/659-rusty-decisions-tab.spec.md, docs/planning/pipeline/completed/660-rusty-decision-follow-up.spec.md]
---

## Title
The Memory tab. Rusty keeps long-term memories that its agents read at the start of a
conversation (`list_memories`, `store_memory`, `update_memory`, `delete_memory`; content,
category, importance `low`, `normal` or `high` since its TICKET-052, source and times). Marley
shows none. This ticket adds a center tab that lists them in Rusty's order, adds one, filters by
category, and edits or deletes one.

## Scope
### In
- **The tab.** `rusty: open memory`, and a Memory button after Decisions in the Brain view's
  header, open a center tab (or bring it forward). Under its title, the count; then the memories
  as Rusty lists them (high first, then normal, then low, newest first within each), each with its
  content (up to three lines), its category, its importance, who stored it and the day it last
  changed.
- **Adding.** A field at the top ("Remember something and press Enter"), a category field
  (empty means `context`, as Rusty's app files it), and Low, Normal, High (Normal chosen). Enter
  stores it; the field empties and the list shows it.
- **Filtering.** A Category menu: All, then each category the list holds; the count follows.
- **Editing.** A click on a row opens a form with the content, the category and the importance;
  Save sends the three, Delete asks first and then removes it. Rusty's refusal stays in the form.
- **Live.** The tab reads when it opens, when Rusty connects, on Rusty's announcement while it
  shows (else when it next shows), after each of its own writes, and on Read again after a failure.
  With Rusty off or not connected it says so.
- **`marley_rusty::memories`** and the stand-in's four tools over `memories.json`, with Rusty's
  order, importance words and refusals, its watcher announcing the file's changes.

### Out (explicitly deferred)
- **Searching memories' text** (Rusty's tools have no search; the filter is by category).
- **Bulk edits and import**; one memory at a time.
- **Showing which memories an agent's prompt receives** (Rusty's 8,000-character block); Rusty's.

## Reference (§20)
N/A — Marley-specific: Rusty's own app is the reference for the behaviour
(`crates/rusty-app/qml/MemoryPage.qml` in Rusty's repository at `eb1ab51`: an add field with a
category and an importance, `context` by default; the count and a category filter; rows with the
content in up to three lines, the category, importance, source and day; a click opens an editor
with Save, Delete asked first, and Close). Zed's own list items, chips, toggle buttons, dropdown,
modal and prompt draw it.

### Prior art
- **Behaviour maps:** `docs/t3code_architecture/` and `docs/orca_architecture/` hold no memory list;
  `docs/zed_architecture/` none (Zed's agent "rules" library is a different store).
- **Published material:** Rusty's tools (`crates/rusty-mcp/src/main.rs:1045-1056`, `:1566-1592`,
  `:1732-1738`; `StoreMemoryParams` `:182-190`, `UpdateMemoryParams` `:531-544`) and its store
  (`crates/rusty-core/src/engine/memory_manager.rs`: `IMPORTANCE`, `importance_word` refusing any
  other word, `medium` read as `normal`, `LIST_ORDER`, `Memory`, `update` answering the memory,
  `delete` refusing an unknown id, times in seconds).
- **The code we ship:** the Decisions tab (`rusty/decisions_tab.rs`: the item, `Link`, `ReadDue`,
  the reads) and #660's form (`rusty/follow_up.rs`); `ToggleButtonGroup`, `Chip`, `DropdownMenu`
  (as `rusty.rs`'s provider menu uses it), `Window::prompt`. No crate we build owns memories.

## UI proof
`script/e2e/664-rusty-memory-tab.sh` (`compositor sway`: clicks on rows, buttons and menus). Setup:
the stand-in with `memories.json` in its state folder holding five made-up memories in three
categories and all three importances; `MARLEY_RUSTY_MCP` names it, never the user's Rusty (R-D8).
Shots:
- `664-01-memory`: `rusty: open memory`: the count and the five in Rusty's order.
- `664-02-added`: a line typed, `preference`, High, Enter: it heads the list, the field empty.
- `664-03-filter`: Category, `preference`: only those, and the count follows.
- `664-04-form`: a row clicked: the form with its content, category and importance.
- `664-05-saved`: the content changed, Low, Save: the row shows both, now among the low ones.
- `664-06-refused`: Save with the stand-in refusing: Rusty's words in the form.
- `664-07-delete-prompt`: Delete: the question naming the memory.
- `664-08-deleted`: Delete confirmed: the row gone, the count down.
- `664-09-outside`: `memories.json` changed from outside, as Rusty's CLI would: the list follows.
- `664-10-off`: Rusty turned off: the tab says so.

## Locked-In Decisions
- D1 — **A center tab like Decisions** (`rusty/memory_tab.rs`): one per workspace, the same `Link`
  and `ReadDue` reads, the Brain view's header button and a palette command.
- D2 — **Rusty's order, untouched**: the list is drawn as `list_memories` answers it; the filter
  hides rows and asks nothing.
- D3 — **Importance is Rusty's three words**: Low, Normal, High in a toggle group; a word Rusty may
  have stored outside the three shows as written and sorts as Rusty sorts it.
- D4 — **An empty category stores as `context`**, as Rusty's app does (Rusty's MCP default is
  `fact`, its app's is `context`, and the tab follows the app the user knows).
- D5 — **The edit form is a modal in the tab's module**, Save sending content, category and
  importance together, as Rusty's app does; Save waits for non-empty content, since Rusty would
  store an empty one.
- D6 — **Delete asks first** through Zed's prompt, naming the memory's first line.
- D7 — **After each write the tab reads again**, the service connection announcing nothing.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `rusty: open memory` runs or the Brain view's Memory button is clicked, the system shall open the Memory tab or bring it forward. | Shot `664-01-memory` |
| REQ-002 | WHILE the tab shows, it shall list Rusty's memories in Rusty's order with their content, category, importance, source and day, under their count. | Shot `664-01-memory` |
| REQ-003 | WHEN Enter is pressed in the add field with text, the system shall store it with the category typed (or `context`) and the importance chosen, and list it. | Shot `664-02-added` |
| REQ-004 | WHEN a category is chosen in the Category menu, the tab shall show that category's memories only, and their count. | Shot `664-03-filter` |
| REQ-005 | WHEN a row is clicked, the system shall open a form holding its content, category and importance. | Shot `664-04-form` |
| REQ-006 | WHEN Save is clicked, the system shall send the three to Rusty and show the memory as Rusty holds it. | Shot `664-05-saved` |
| REQ-007 | IF Rusty refuses a write, THEN the form shall show Rusty's words and keep what was typed. | Shot `664-06-refused` |
| REQ-008 | WHEN Delete is clicked, the system shall ask first and, on yes, remove the memory. | Shots `664-07-delete-prompt`, `664-08-deleted` |
| REQ-009 | WHEN Rusty announces a change while the tab shows, the tab shall list the memories as they now are. | Shot `664-09-outside` |
| REQ-010 | WHILE Rusty is off or not connected, the tab shall say so and list nothing. | Shot `664-10-off` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_rusty::memories`; `rusty/memory_tab.rs` (the tab, its add row, filter, rows,
  the form); the Brain view's button; the stand-in's four tools and its watched file; the guide
  page; a review; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, the plan's R8, the architecture notes, the guide and the
  walkthrough, ledger capture, the brain decision, close, archive, commit.
