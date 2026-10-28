---
pipeline_id: fa257a6c-c9c5-4172-a261-c0cc3cbcd298
ticket: docs/planning/tickets/open/TICKET-939-rusty-tasks-knowledge-brain-as-the-board.md
status: Phase 4 — Complete PASS
title: Rusty's Tasks and brain pages as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/E-rusty.md, docs/marley/look-plan.md]
---

## Title
Rusty's Tasks tab and brain page as the Tasks and BrainPage boards draw them: Follow Up on the
Decisions due card opens the form, task rows as cards under the board's header; on a page, body
checkboxes that toggle, "Linked here", properties as coloured pills, ★ favourites and an Outline
text button.

## Scope
### In
- **Decisions due card** (Marley crate, `rusty/tasks_tab.rs`, the Rusty column at `:1079-1172`).
  Each due decision gets a filled accent **Follow Up**, which opens #660's follow-up form for that
  decision, and an outlined **Open the page**. The plain Decisions button goes. "N overdue" moves
  to the right of the card's heading. Under the decisions, muted: "Ask before a choice, record it
  when made, follow up when its day comes."
- **The form from Tasks** (Marley crate, `rusty/follow_up.rs`, `open` and `FollowUpModal`). The
  form takes what to do after Record as a callback, not a `BrainDecisionsView`, so the Tasks tab
  can open it and read its column again after a Record. The Decisions tab passes its own re-read.
- **Task rows as cards** (`tasks_tab.rs`, `render_task`). Each task is a bordered, rounded card
  with padding, its checkbox at the left; the selected card has the selected background. Drag,
  right-click and double-click to rename keep working.
- **The header and lists column** (`tasks_tab.rs`, `render_header`, the lists column at
  `:960-1010`). The list's name in a headline size (the board's 24 px); the lists heading as
  uppercase "LISTS"; each list's dot as a small rounded square in the same colour.
- **Checkboxes in a page's body** (Marley crate, `rusty/page.rs`, `markdown_element`). In read
  mode, the page sets Zed's `MarkdownElement::on_checkbox_toggle`. A click flips that `[ ]` or
  `[x]` in the page's text and writes the page through the page's existing save path
  (`brain_write_page`), so Rusty keeps the file. While the page is not editable (Rusty
  disconnected, a read-only page), the click does nothing.
- **"Linked here"** (`rusty/page.rs`, `render_outline`). The outline column ends with a "Linked
  here" heading and the pages that link to this one, read with `brain_get_links` when the page
  opens; a click opens that page. With no headings and some backlinks, the column shows "Linked
  here" alone.
- **Properties as coloured pills** (Marley crate, `rusty/properties.rs`, `value`). Each value is a
  pill: `status` decided or done in the success colours; `follow_up_by` in the past in the error
  colours with "· overdue" after the date; `tags` in the teal player colour; every other value
  neutral.
- **★ favourites** (Marley crate, `rusty/brain.rs`, the favourites list at `:1335-1356`). Each
  favourite starts with ★; the kind's icon goes, its detail stays in the tooltip.
- **Outline as a text button** (`rusty/page.rs`, `render_outline_button`). A text button
  "Outline" that shows pressed while the outline column is open, same action, tooltip and
  disabled states.

### Out (explicitly deferred)
- **The Knowledge panel, a follow-up ticket.** From the review, all look or missing: "Search the
  brain, or ask it" (asking the brain with `brain_ask` beside `brain_search`; asking records a
  consultation, so it needs its own decision); the Following card's page-type tile and `#tag`
  without counts; graph settings in a fixed right column instead of a floating card, with the
  legend in a row under the graph; Link distance and Repel shown open.
- The Today card, each task's note line and due chip, "done Monday" and "1 done this week", and
  "tell Rusty: remind me Friday to…": deferred by #845; Rusty's `UserTask` has no note, due date
  or completion time.
- "Ask an agent" / "Open thread" / "Open canvas" on a task row: deferred by #845.
- The Memory card's empty line: not on this ticket's list.
- The bottom row for Tasks, Graph and Brain: #909 (the bottom row for other screens) builds it.

## Reference (§20)
Upstream Zed for the checkbox: Zed's markdown preview toggles a task checkbox through
`MarkdownElement::on_checkbox_toggle` and writes the flipped source back
(`crates/markdown_preview/src/markdown_preview_view.rs:1202`); the brain page does the same over
Rusty's page text. The rest is N/A — Marley-specific: Rusty's tabs (#845, #874, #875, #660).

### Prior art
- `crates/marley_workbench/src/rusty/tasks_tab.rs:1118-1139`: the due row's plain "Decisions"
  (dispatches `OpenDecisions`) and "Open the page"; `:1157-1172` "N overdue" as a line under the
  heading, no footnote. Gap holds at HEAD.
- `tasks_tab.rs:195-196` `Side { due: Vec<DecisionSummary> }`: the column already holds each
  due decision's summary, which the form needs.
- `crates/marley_workbench/src/rusty/follow_up.rs:35-52` `open(workspace, tab:
  WeakEntity<BrainDecisionsView>, summary, candidates, …)`; `:208` the form updates that tab
  after Record. `decisions_tab.rs:395-415` opens it with `window.defer`, outside the click's
  update.
- `crates/marley_workbench/src/rusty/home_tab.rs:370-410` (#903): Home's Follow Up opens the page,
  not the form; unchanged here.
- `tasks_tab.rs:1220-1300` `render_task`: a `ListItem` with the checkbox as `start_slot`, drag and
  drop on the wrapping `div`; `:1180-1215` the header's Large label; `:960-1010` the lists column
  with `Indicator::dot` and "Lists".
- `crates/marley_workbench/src/rusty/page.rs:1715-1735` `markdown_element` sets
  `on_url_click` only; `crates/markdown/src/markdown.rs:1881-1887`
  `on_checkbox_toggle(Range<usize>, bool, …)`; `page.rs:295` the page text from `brain_render`,
  saved with `brain_write_page`; `:589` and `:1938` the save path that checks the page did not
  change under it.
- `page.rs:1640-1688` `render_outline`: "On this page" and the headings only; #875 deferred
  "Linked here" ("the page view reads no backlinks"). `crates/marley_rusty/src/knowledge.rs:42,
  71-79` `BRAIN_GET_LINKS` and its answer's `backlinks`, which the Knowledge panel reads.
- `page.rs:1354-1370` `render_outline_button`: an `IconButton` with `IconName::ListTree`.
- `crates/marley_workbench/src/rusty/properties.rs:65-81` `value`: plain labels, lists as
  uncoloured `Chip`s.
- `crates/marley_workbench/src/rusty/brain.rs:1339-1356`: an icon per bookmark kind.
- `assets/themes/marley/marley.json:161-163`: the teal player colour (#7dd3c0) the board uses for
  tags.
- The stand-in `crates/marley_rusty/stand_in/rusty-mcp` answers `brain_render`,
  `brain_write_page`, `brain_get_links`, `bookmark_list`, `brain_follow_up` and the decision tools
  (`:60-100`, `:2092-2130`).

## UI proof
Scenario `script/e2e/939-rusty-tasks-knowledge-brain-as-the-board.sh` (sway). Setup: Rusty on
through `marley_rusty`'s stand-in `rusty-mcp` over a scratch vault (#903's and #875's pattern): a
decision due yesterday, a page with frontmatter (`status: decided`, `follow_up_by` in the past,
`tags`), two headings and a task list `- [ ] Rusty cards on Home`, a second page linking to it,
one favourite; made-up task lists. No agent CLI.
- `939-01-tasks`: the Tasks tab: the header, LISTS with square swatches, the tasks as cards, the
  Decisions due card with "1 overdue" at the right, Follow Up and Open the page, and the footnote
  (REQ-001, REQ-003, REQ-004).
- `939-02-form`: Follow Up clicked: the follow-up form for that decision (REQ-002).
- `939-03-page`: the decision page opened: the property pills, Outline pressed, the outline with
  "Linked here" and the linking page (REQ-006, REQ-007, REQ-009).
- `939-04-favourite`: the Brain tab's favourite with its ★ (REQ-008).
- `939-05-ticked`: the body's checkbox clicked: it shows ticked, and the vault file holds `- [x]`
  (an `expect` checks the file) (REQ-005).
- `939-06-day`, `939-07-day-page`: Tasks and the page in Marley Day (REQ-001, REQ-003, REQ-004,
  REQ-007).

## Locked-In Decisions
- D1 — Follow Up opens #660's form on the Tasks tab itself; the form's after-Record step becomes a
  callback so both tabs can open it. Its successor list is the decisions the Tasks tab holds.
- D2 — A checkbox toggles only in read mode and only while the page can be saved; it writes
  through the page's own save path, so a page changed under it is refused as an edit would be.
- D3 — "Linked here" reads `brain_get_links` once per opened page, as the Knowledge panel does; it
  does not follow the vault live.
- D4 — Pill colours come from the key and value: `status` (decided, done → success), a past
  `follow_up_by` (error, "· overdue"), `tags` (teal); nothing else is coloured.
- D5 — The Knowledge panel's items are a follow-up ticket (Out); this slice is Tasks and the page.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a decision is due, the Tasks tab's Decisions due card shall show it with a filled Follow Up and an outlined Open the page, "N overdue" at the right of the heading, and the footnote under the decisions. | `939-01-tasks`, `939-06-day` |
| REQ-002 | WHEN the user clicks Follow Up on the Tasks tab, the system shall open the follow-up form for that decision. | `939-02-form` |
| REQ-003 | The Tasks tab shall draw each task as a bordered, rounded card with its checkbox. | `939-01-tasks`, `939-06-day` |
| REQ-004 | The Tasks tab shall show the list's name in a headline size, the lists heading as "LISTS", and each list's colour as a rounded square. | `939-01-tasks`, `939-06-day` |
| REQ-005 | WHEN the user clicks a task checkbox in a brain page's body in read mode, the system shall flip it and write the page through Rusty. | `939-05-ticked` |
| REQ-006 | WHILE other pages link to the page shown, its outline column shall end with "Linked here" and those pages. | `939-03-page` |
| REQ-007 | A brain page shall draw its property values as pills, `status` decided in the success colours, a past `follow_up_by` in the error colours with "· overdue", and `tags` in teal. | `939-03-page`, `939-07-day-page` |
| REQ-008 | The Brain tab's favourites shall each start with ★. | `939-04-favourite` |
| REQ-009 | A brain page's outline toggle shall be a text button "Outline" that shows pressed while the outline is open. | `939-03-page` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `tasks_tab.rs`, `follow_up.rs`, `decisions_tab.rs`, `page.rs`, `properties.rs`,
  `brain.rs`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
