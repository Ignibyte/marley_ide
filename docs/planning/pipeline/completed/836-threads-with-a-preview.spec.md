---
pipeline_id: e193f6b8-d0ed-4fc9-8076-da27a1cfd3b1
ticket: docs/planning/tickets/open/TICKET-836-threads-with-a-preview.md
status: Phase 4 — Complete PASS
title: The Threads page with filters, day groups and a preview
type: feature
slice: The Marley look, W2.3 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The Threads page reads like a mail client. Filters on the left (who, which folder), the
conversations as cards grouped by day (Today, Yesterday, This week, Earlier), and on the right a
preview of the selected card: its title, agent, folder, when it last changed, what the search
found in it, and Open, Open in a Split and Archive. A click selects a card and a double-click
opens it. Today the page is one long list under agent chips, and every click opens the
conversation at once.

## Scope
### In
- A left column (200 px): WHO (All and each agent, the existing chips, stacked) and FOLDER (All
  and each folder the listed conversations ran in).
- The middle: the search box on top (as today), then the rows (Marley and Rusty's and the others
  together) grouped under day headings from `updated_at`: Today, Yesterday, This week, Earlier. The
  selected card is drawn selected. The archived toggle stays at the end.
- The right column (320 px): the selected row's preview, or "Pick a conversation" with none:
  its agent's icon and name, the title, the folder, the last change, the search's snippet when
  there is one, and Open, Open in a Split (a split to the right of the active pane, then open) and
  Archive (a Zed conversation only; a CLI session has none).
- A click selects; a double-click opens (as Open).

### Out (explicitly deferred)
- The conversation's own messages in the preview (the store holds titles, not messages; loading a
  thread to preview it would start its agent).

## Reference (§20)
N/A — Marley-specific. The Threads page is Marley's own (`threads_page.rs`, #745, #754); Zed's
agent history is a picker. The layout is the design canvas's Threads board.

### Prior art
- The code we ship: `ThreadsPage` (`sections`, `render_chips`, `render_row`, `open_row`,
  `open_thread`), `Row { target, title, folder, updated_at, snippet, when }`, `Target { Thread,
  Cli }`, `ThreadMetadataStore::archive` (the rail's `archive_thread`), `Workspace::split_pane`,
  `ListItem::toggle_state`, gpui's `ClickEvent::click_count` (the rail's double-click rename).
- Ledger: #754's scenario (CLI sessions from fixture files, stand-in `claude`), PR-claude-701 (a
  page's handlers close over a weak workspace). The brain had nothing on this seam.

## UI proof
Scenario `script/e2e/836-threads-with-a-preview.sh` (sway): #754's fixtures, two Claude Code CLI
sessions, one in the repository changed now and one in `sub` changed four days ago; a stand-in
`claude` on PATH that logs its launch.
- `836-01-night`: `marley: open threads`: the filters at the left (WHO: All, Claude Code CLI;
  FOLDER: All, the two folders), the cards under Today and This week (or Earlier), and the preview's
  "Pick a conversation" (REQ-001, REQ-002).
- `836-02-selected`: the first card clicked: selected, the preview filled with Open and Open in a
  Split (REQ-003).
- `836-03-folder`: the `sub` folder picked: only its card (REQ-001).
- `836-04-opened`: All folders again, the first card's Open: the stand-in launched to resume it
  (REQ-004).
- `836-05-day`: Marley Day, the page with a card selected (REQ-001 to REQ-003).

## Locked-In Decisions
- D1 — The day groups replace the MARLEY AND RUSTY / ALL CONVERSATIONS split; the assistants'
  rows carry their own icons.
- D2 — A click selects; Enter, a double-click, or Open opens.
- D3 — Archive shows only for a Zed conversation; it goes through `ThreadMetadataStore::archive`,
  as the rail's row does.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Threads page shall offer WHO and FOLDER filters at its left, and list only the matching conversations, in Marley Night and Marley Day. | `836-01-night`, `836-03-folder`, `836-05-day` |
| REQ-002 | The page shall list the conversations as cards under day headings by their last change. | `836-01-night` |
| REQ-003 | WHEN the user clicks a card, the page shall select it and show its preview with Open and Open in a Split (and Archive for a Zed conversation). | `836-02-selected`, `836-05-day` |
| REQ-004 | WHEN the user clicks Open (or double-clicks a card), Marley shall open the conversation as the page's rows did. | `836-04-opened`; the launch log |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the three columns, the folder filter, the day groups, selection and the preview;
  a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check above; read every shot.
- **P4 Complete** — CHANGELOG, the guide and the architecture note (§21), ledger capture (§19),
  close the ticket, archive, commit.
