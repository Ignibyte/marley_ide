---
pipeline_id: 02a2e38b-6381-4980-b75c-0877567e7f2b
ticket: docs/planning/tickets/open/TICKET-672-rusty-screens-in-the-rail-header.md
status: Phase 4 — Complete PASS
title: "Rusty's screens in the rail's header, and the row under it at the tab bar's height"
type: feature
slice: the rail; Rusty in Marley R-D9 (#644)
references: [docs/planning/pipeline/completed/644-brain-view-in-the-rail.spec.md, docs/marley/rusty-in-marley.md]
---

## Title
Rusty's seven screens move up into the rail's header beside Projects and Brain, and the row under
the header lines up with the pane's tab bar. Chad, 2026-10-06: "I want all the rusty brain icons
at the top in the same location as Projects and the Folder with the +. Instead of clicking over to
the other icon and then seeing the other tabs", and later "each sub tab just lives up in the top
next to the others and it shows its information when you click on it just as it does now except
not two levels deep". And: "on filter it should be the same height as the <- -> |Tab| of the main
menu. Even when the icons expand?"

## Scope
### In
- **The header, while Rusty is connected**: Projects, Brain, then Today, Graph, Tasks, Decisions,
  Memory, Skills and Secrets, then the space, then the folder `+` (Projects view) or New Page
  (Brain view), in both views. Each screen opens or focuses its tab as the Brain view's row did.
- **Overflow**: the screens that do not fit the rail's width go, in order, into a `…` menu at the
  end of the screens, so Projects, Brain and the `+` always show.
- **The Brain view** loses its icon row; its search row and tree move up.
- **The row under the header**: the filter row (Projects view) and the search row (Brain view)
  are `Tab::container_height` high, with their bottom border on the tab bar's, whatever they hold
  (the key hint, the clear button, text).

### Out (explicitly deferred)
- **The Rusty group** where these screens open (#675): they still open in the shown project.
- **A Rusty icon**: none yet; no placeholder in the header, since the group carries it (#675).

## Reference (§20)
Upstream Zed — the pane's tab bar (`ui::TabBar`, `Tab::container_height`) is the height the row
matches; the header keeps Zed's `IconButton` and the `PopoverMenu` + `ContextMenu` the folder
`+` already uses. The flat row of view buttons is R-D9's (Obsidian's ribbon, VS Code's activity
bar), one level instead of two.

### Prior art
- **Behaviour maps:** Warp's vertical tabs search row is a single fixed-height row with a 1px line
  under it (`docs/warp_architecture/observed/468-warp-vertical-tabs-notes.md`); Orca's sidebar
  header switches views with icon buttons (`docs/orca_architecture/01-agents-and-sessions.md`).
- **Published material:** none applies.
- **The code we ship:** `ui::TabBar` draws its bottom line inside `Tab::container_height`
  (`tab_bar.rs:100-130`); `ButtonSize::Compact` (18/16 rem); `ContextMenu` entries with icons
  for the overflow; `rusty::capture::open_today` and each tab's `open_later` open the screens
  without the Brain view.

## UI proof
`script/e2e/672-rusty-screens-in-the-rail-header.sh` (`compositor sway`), Rusty's stand-in
`rusty-mcp` over a scratch state folder, the copy at Chad's UI size. Shots:
`672-01-projects` (the header with Projects, Brain, the screens that fit, `…`, the folder `+`;
the filter row's line on the tab bar's); `672-02-filtering` (text in the filter and its clear
button: the same height); `672-03-brain` (the Brain view: no icon row, the search row's line on
the tab bar's); `672-04-graph` (the Graph button, clicked in the Projects view: the Graph tab in
front); `672-05-overflow` (the `…` menu open with the rest); `672-06-wide` (the rail dragged
wider: every screen in the header, no `…`).

## Locked-In Decisions
- D1 — **The count of screens shown is worked out from the rail's own width** and the window's rem
  size, not measured after layout: the rail knows its width (`Rail::width`), and every button is
  `ButtonSize::Compact`, square, so the sum is exact.
- D2 — **The header's buttons are all Compact**, Projects, Brain and the `+` too, so ten fit a
  rail of about 300px at a 24px UI font.
- D3 — **Today keeps revealing its page in the tree** when the Brain view exists; otherwise it is
  `open_today`.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Rusty is connected, the rail's header shall show Projects, Brain and Rusty's screens in one row, in both views. | `672-01-projects`, `672-03-brain` |
| REQ-002 | WHEN a screen's button is clicked, the system shall open or focus that screen's tab. | `672-04-graph` |
| REQ-003 | WHERE the screens do not all fit, the header shall show those that fit and a `…` menu holding the rest, with Projects, Brain and the `+` shown. | `672-01-projects`, `672-05-overflow`, `672-06-wide` |
| REQ-004 | The Brain view shall show no row of screen buttons. | `672-03-brain` |
| REQ-005 | The filter row and the Brain view's search row shall be the tab bar's height, their bottom line on the tab bar's, empty or holding text. | `672-01-projects`, `672-02-filtering`, `672-03-brain` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `rail.rs`, `rusty/brain.rs`, the guide; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the scenario and its shots.
- **P4 Complete** — CHANGELOG, the guide, the architecture notes, ledger capture, close, archive,
  commit.
