---
pipeline_id: dac38f8c-1d39-4b8c-9013-a28e55ac14fe
ticket: docs/planning/tickets/open/TICKET-827-rail-rows-say-their-state.md
status: Phase 4 — Complete PASS
title: Rail rows say their state by color
type: feature
slice: The Marley look, W2 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
A rail row tells its kind and its state at a glance. Its icon sits on a rounded tile tinted for its
kind (agent thread, agent in a terminal, shell, Browser tab, tmux session) with a dot for its state
at the tile's corner, and its first line takes the state's color: working amber, needs you pink,
done the accent, idle muted. A folded project header's summary turns pink when something under it
waits on you. Today every row's icon sits on the same grey circle, and the state shows only in the
words and the scattered marks.

## Scope
### In
- `RowState { Working, NeedsYou, Done, Idle }` and its two colors from the theme (the dot and the
  line): `status().warning`, `status().conflict`, `status().success` (dot) / `text_accent` (line),
  `text_muted`. These are the "small set of tokens".
- `RowKind { Thread, AgentTerminal, Shell, Browser, Tmux }` and its tile tint from the theme:
  thread `status().warning_background`, agent terminal `status().info_background`, shell the
  raised surface (as today), Browser `text_accent` at 15 %, tmux `status().hint_background`.
- `row_card` draws the icon on a rounded-square tile of the kind's tint, with the state's dot at
  the bottom-right, when the caller gives a look; callers without one keep today's circle.
- The look for terminal rows (state from the agent's status, the bell, and a running command),
  thread rows (from the thread's status and attention), Browser tab rows (no state) and tmux rows
  (no state), and the first line's color from the state.
- A folded project header's summary in the needs-you color while its `attention` is set.

### Out (explicitly deferred)
- Browser tab states (loading, a dialog open), which the snapshot does not carry today.
- New glyph letters (`CC`, `CX`, `www`) in place of the existing icons: the icons stay (they name
  the agent better), only their ground changes.
- Ports, worktrees, Files and tab rows: they keep the circle.

## Reference (§20)
N/A — Marley-specific. The rail and its rows are Marley's own (`marley_workbench::rail`); Zed's
sidebar draws no kinds or states, and Warp's vertical tabs (`docs/warp_architecture/`) were the
rows' first shape (#507), not their states. The behavior is the design canvas's Rail board ("Rail
rows: kinds and states").

### Prior art
- The code we ship: `row_card` and `row_frame` (`rail.rs`), `RowLine { color }`, `terminal_marks`
  and `rail_terminal_icon`, `marley_rail::terminal_attention` / `thread_attention_class` (the same
  states, as `Attention`), `ProjectRow.attention` and `summary` (#542), and the theme's status
  colors and backgrounds (#828 set them for Marley Night and Day).
- Ledger: F-claude-603 and F-claude-750 (a row's trailing parts must not squeeze its text: the
  tile keeps `size_7`), L-claude-555 / L-claude-749 (rail points move; a scenario reads its
  points from a probe run), L-claude-831 (`tmux_hosts` for a stable rail). The brain had nothing
  on this seam.

## UI proof
Scenario `script/e2e/827-rail-rows-say-their-state.sh` (sway): projects `a` and `b` and `c`, each
with #542's stand-in Claude Code (working, waiting, finished unseen), `a` with a second terminal
running `sleep 60` and `c` with a shell that ran `false`; one tmux session; `b` folded.
- `827-01-night`: the rail in Marley Night: the agent tiles, the state dots and colored lines for
  working, needs you, done and idle; the shell's tile; the tmux tile (REQ-001, REQ-002).
- `827-02-folded`: `b` folded: its header's summary in pink (REQ-003).
- `827-03-day`: Marley Day, the same rail (REQ-001, REQ-002).

## Locked-In Decisions
- D1 — States come from what the rail already computes (`terminal_attention`,
  `thread_attention_class`, the command's `running`), mapped to four: NotReporting reads as
  Working.
- D2 — Tints and colors are the theme's status tokens, never literals, so Marley Night, Day and
  any Zed theme draw them.
- D3 — `row_card` keeps its signature for callers without a look (`impl Into<RowIcon>`).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Each terminal, thread, Browser tab and tmux row shall draw its icon on a tile tinted for its kind, in Marley Night and Marley Day. | `827-01-night`, `827-03-day` |
| REQ-002 | WHILE a terminal or thread row is working, waiting on the user, done unseen or idle, its tile shall carry that state's dot and its first line that state's color. | `827-01-night`, `827-03-day` |
| REQ-003 | WHILE a folded project holds a row that needs the user, its header's summary shall be drawn in the needs-you color. | `827-02-folded` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `RowState`, `RowKind`, `RowIcon`, `row_card`'s tile and dot, the looks for the
  four row kinds, the header summary; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check above; read every shot.
- **P4 Complete** — CHANGELOG, the guide and the architecture note (§21), ledger capture (§19),
  close the ticket, archive, commit.
