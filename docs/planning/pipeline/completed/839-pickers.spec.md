---
pipeline_id: 7738bed6-def4-49df-bd9a-097c8d58156b
ticket: docs/planning/tickets/open/TICKET-839-pickers.md
status: Phase 4 — Complete PASS
title: Pickers: the palette's groups, the agent cards, the file finder's preview
type: feature
slice: The Marley look, W3.3 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The three pickers as the Palette board draws them, in the Marley layout: the command palette,
before anything is typed, lists its commands under headings (Recent, Marley, Editor and
workspace); the New Agent picker shows each agent as a card with a line about it and where it
runs; the file finder opens with its preview at the right.

## Scope
### In
- The command palette with an empty query: the recently used commands under RECENT, then Marley's
  commands under MARLEY, then the rest under EDITOR AND WORKSPACE, each heading over its first row.
  Typing keeps Zed's fuzzy order without headings.
- The New Agent picker's first step: each agent a two-line card, its icon on a tile, its name, a
  line about it (what it is, or that it is not on Marley's PATH), and Thread or Terminal at the end.
- The file finder: its preview at the right the first time it opens, until the user picks another
  layout (Zed keeps that choice).

### Out (explicitly deferred)
- A grid of agent cards (a picker's list is one column).
- Previews in the other pickers (outline, symbols, search).

## Reference (§20)
Upstream Zed: `command_palette` (its delegate orders an empty query by use), `picker` (the
preview's layout loads from the key-value store, `Hidden` when nothing is stored; the footer's
toggle), `file_finder` (`uniform_list_with_preview`). The New Agent picker is Marley's (#734).

### Prior art
- The code we ship: `Picker::new`'s `load_last_preview_layout(D::name())`, `preview::Layout`;
  the palette's `CommandPaletteDelegate` (`matches_updated`, `render_match`, its hit counts);
  `agents.rs`'s `NewAgentDelegate` and `Choice { name, icon, start }`; Marley's hooks into Zed as
  globals (#834, #838).
- Ledger: L-claude-836 (a full-width button centers its label), PR-claude-747.
- The brain (consultation 250006bb): nothing on this seam.

## UI proof
Scenario `script/e2e/839-pickers.sh` (sway): a repository with a few files; stand-in `claude`
and `codex` on the PATH.
- `839-01-palette`: the palette after one command was run: RECENT with it, MARLEY, EDITOR AND
  WORKSPACE (REQ-001).
- `839-02-typed`: "open" typed: no headings, Zed's order (REQ-001).
- `839-03-agents`: `marley: new agent`: the cards with their lines and places (REQ-002).
- `839-04-files`: the file finder, a file selected: the preview at the right (REQ-003).
- `839-05-day`: Marley Day, the palette (REQ-001).

## Locked-In Decisions
- D1 — Headings only for an empty query, in the Marley layout; Zed's layout keeps Zed's list.
- D2 — The preview's default comes from a Marley hook in `Picker::new` for pickers it names (the
  file finder), used only when nothing is stored, so the user's own choice wins.
- D3 — The agent cards stay a list (one column), each row two lines.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the command palette opens with nothing typed in the Marley layout, it shall list its commands under Recent, Marley, and Editor and workspace headings, and WHEN the user types, it shall drop them, in Marley Night and Marley Day. | `839-01-palette`, `839-02-typed`, `839-05-day` |
| REQ-002 | The New Agent picker shall show each agent as a card with its name, a line about it, and where it runs. | `839-03-agents` |
| REQ-003 | WHEN the file finder opens and no preview layout is stored, it shall show the selected file's preview at the right. | `839-04-files` |

## Phase Plan
- **P1 Plan** — this spec and the design in the notes.
- **P2 Code** — the two Zed hooks with ledger rows, the agent cards; review; gate.
- **P3 Test** — the visual check above.
- **P4 Complete** — docs, ledger, close, archive, commit.
