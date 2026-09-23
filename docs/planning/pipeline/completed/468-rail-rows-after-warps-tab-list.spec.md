---
pipeline_id: eacc98d8-fc13-4cae-b181-eded20b74a13
ticket: docs/planning/tickets/closed/TICKET-468-rail-rows-after-warps-tab-list.md
status: Phase 4 — Complete PASS
title: The rail's rows after Warp's tab list
type: feature
slice: workbench shell
references: [docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md, docs/marley_architecture/marley_workbench.md]
---

## Title
Chad asked for the rail to look like Warp's vertical tab list: more padding around the projects
and terminals, and bigger icons. The rail's terminal and thread rows become padded two-line
cards with a large round icon, the selected row a bordered card, each project a muted header,
and a line runs between projects.

## Scope
### In
- **One row for terminals and threads** (`marley_workbench::rail`), drawn by Marley instead of
  Zed's `ListItem` and `ThreadItem`: a fixed height, a round icon container, the title over a
  muted second line, and a trailing slot. The selected row is a filled card inside a border;
  every row keeps the border, clear when not selected, so the selection moves nothing.
  - A terminal row keeps its bell dot, swapped for its close button while the pointer is over
    it.
  - A thread row shows its status at the trailing end, as Zed's thread row does: a spinner while
    it runs, a warning while it waits for a confirmation, an error mark after a failure, the
    attention dot otherwise. Its second line names the agent and the status in a word, as an
    agent CLI's terminal row does ("Zed Agent · working").
- **The project header** (`rail`): taller, the name in the muted small type of a section
  label, the same card when selected; the chevron, the attention dot and the `+` menu stay.
- **Dividers** between consecutive project groups, none above the first.
- **Padding**: the list and the filter row get more room.
- **A word per thread status** (`marley_rail::ThreadStatus::label`) and the agent's name for a
  thread (`marley_workbench::agents`), the latter shared with the New Agent Thread menu.

### Out (explicitly deferred)
- Warp's `Ctrl <n>` hints: Marley has no binding for the n-th row.
- Warp's tab groups and their band: Marley's groups are its projects.
- The thread switcher keeps Zed's `ThreadItem`.
- New theme colors: every color comes from the theme; `raised` is its text color at 10%.

## Reference (§20)
- **Warp:** the vertical tab list, observed from a screenshot on 2026-09-23 and written up in
  `docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md` (layout only; no source).
  Marley matches the padding, the 28px round icon, the two lines of text, the selected card and
  the lines between sections, and gives a project's header the section label's muted type.
- **Upstream Zed:** the rows' behavior stays Zed's where it was: the filter's highlights, the
  hover-revealed close button of Zed's `ListItem::end_slot_on_hover`, and the thread statuses
  Zed's `ThreadItem` draws.

### Prior art
- **Behavior maps:** the Warp notes above; `docs/warp_architecture/` has nothing else on the
  tab list. The rail's earlier visual reference (beautifului, #418) asked for quiet rows and one
  lit selection, which this keeps.
- **Published material:** none needed.
- **Code we already ship.**
  - `ui::ListItem` has `Sparse` spacing, `inset`, `outlined` and `height`, but its outline is
    drawn only when set, so a border that follows the selection would move the row by a pixel,
    and its start slot takes no container. Not used for these rows.
  - `ui::ThreadItem` draws a fixed 16px icon slot and one line. Kept for the switcher.
  - `ui::Divider::horizontal()` draws the lines between groups.
  - gpui's `group`, `visible_on_hover` and `group_hover` give the hover swap the bell and the
    close button need, as `ListItem` builds it.
  - `ui::CommonAnimationExt::with_rotate_animation` spins the running thread's icon, as
    `ThreadItem` does.

## UI proof
UI-AFFECTING: everything the rail draws.
- **Driven tests** (`rail_tests.rs`): a terminal row with a second line, one without, and a
  thread row share one height; each has its round icon at the size set; moving the selection
  moves and resizes no row; a divider precedes every project header but the first; every
  existing rail and switcher test passes unchanged.
- **Live drive:** the debug `marley` on a copy of Chad's profile on hidden workspace 9, shot by
  its toplevel with no input sent (L-claude-467-capture-one-window-by-its-toplevel-001),
  compared with the capture taken before the change.

## Locked-In Decisions
- D1 — The rail draws its terminal and thread rows with one row element of its own. `ListItem`
  and `ThreadItem` stay Zed's, unchanged; the switcher keeps `ThreadItem`.
- D2 — Sizes come from gpui's rem scale, so they follow the UI font size: rows `h_11` (44px at
  the default size), the icon container `size_7` (28px), the project header `h_8`.
- D3 — A thread's second line is `<agent name> · <status word>`, the words `idle`, `working`,
  `waiting` and `failed`, in the voice of an agent CLI's row (`Claude Code · working`).
- D4 — Every debug selector, element id, click, double-click, menu, key and hover the rail has
  stays, so the driven tests read the new rows as they read the old.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The rail shall draw every terminal and thread row at one height, with its icon in a round container of one size, whether or not the row has a second line | driven test |
| REQ-002 | WHEN the selection moves, the rail shall draw the newly selected row as a bordered card without moving or resizing any row | driven test; live drive |
| REQ-003 | The rail shall draw a divider before every project header except the first | driven test |
| REQ-004 | A thread row's second line shall name the thread's agent and its status in one word | unit tests; live drive |
| REQ-005 | Every click, menu, key and hover the rail answers today shall still be answered | the crate's suite unchanged |
| REQ-006 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the row element, the header, the dividers and the padding in `rail.rs`; the
  status word in `marley_rail`; the agent's name in `agents.rs`; fmt and clippy clean; a review
  of the diff.
- **P3 Test** — the driven tests; the crate suites; a negative check; the live drive;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
