---
pipeline_id: e6dd728b-ecd8-411a-8fe7-1733f6b3c8ee
ticket: docs/planning/tickets/closed/TICKET-822-agent-text-full-width.md
status: Phase 4 — Complete PASS
title: An agent's text runs the full width, with a toggle in its bottom row
type: feature
slice: the agent thread view (#798 the width button, #734 thread tabs)
references: []
---

## Title
A thread's messages and message box run the full width of their place by default, and a toggle in
the thread's bottom row narrows them to a readable column and back (Chad, 2026-10-20: "the agent
response i want to be flexible and default it should be 100%", "put that setting into the bottom
panel", "If the default zed setting conflicts we overwrite it").

## Scope
### In
- `assets/settings/default.json`: `agent.limit_content_width` defaults to `false`, with a
  `Marley:` comment; the doc string in `settings_content/src/agent.rs` says so.
- A toggle in the thread view's bottom row, after the width button (or after the follow toggle
  when the width button is hidden), shown in every place a thread shows, while
  `MarleyThreadHost` is set as the width button is. A click writes
  `agent.limit_content_width` to the user's settings through `update_settings_file`; its icon
  and tooltip say which width a click gives.
- The guide's #798 section names the toggle and the new default.

### Out (explicitly deferred)
- `agent.max_content_width`'s value (850 px) stays Zed's; the column's width is not stepped.
- The width button's hiding for a pane alone in its row (#798) stays as it is.
- Zed's Text Threads and the threads list, which read other widths.

## Reference (§20)
Upstream Zed (`agent_ui`, `agent_settings`): Zed's own `limit_content_width` and
`max_content_width` settings and the thread view's `max_w(..).mx_auto()` are kept as they are;
Marley changes only the default and adds a control that writes the same setting, as Zed's model
picker writes `agent.default_model` from the same bottom row (`agent_model_selector.rs`,
`update_settings_file`).

### Prior art
- Zed already owns the behavior: `AgentSettings::max_content_width` is `None` when
  `limit_content_width` is off, and every reader (`thread_view.rs` 3644, 4904, 4994, 6806,
  `agent_panel.rs` 6173) falls back to `w_full()`. Nothing new to draw, only a default and a
  writer.
- `settings::update_settings_file` is the path Zed's own bottom-row controls take to write the
  user's settings; settings observers redraw the thread.
- #798's `render_marley_panel_width` is the model for a Marley control in this row, gated on
  `MarleyThreadHost`.

## UI proof
Scenario `script/e2e/822-agent-text-full-width.sh`, `compositor sway`, a thread tab with a long
answer from the scenario's stand-in agent (as #798's scenario has one).
- `822-01-full` — a new Marley, no width setting: the answer and the message box run the tab's
  full width, and the toggle shows in the bottom row (REQ-001, REQ-002).
- `822-02-column` — after a click on the toggle: the text in the centered column; the run checks
  the settings file holds `"limit_content_width": false` → `true` (REQ-003).
- `822-03-full-again` — a second click: full width again, the file says `false` (REQ-003).

## Locked-In Decisions
- D1 — The toggle writes Zed's existing setting rather than holding a state of its own, so the
  settings file, the Settings UI and the button agree and the choice survives a restart.
- D2 — The toggle is global, not per thread: the setting is one value for every thread.
- D3 — The default changes in `default.json`, Zed's file, by one value: Marley overrides Zed's
  default there as its other `Marley:` defaults do.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the user's settings do not set `agent.limit_content_width`, a thread's messages and message box shall run the full width of their place. | `822-01-full` |
| REQ-002 | The thread's bottom row shall show the text-width toggle, whether or not the width button shows. | `822-01-full` |
| REQ-003 | WHEN the user clicks the toggle, Marley shall flip `agent.limit_content_width` in the user's settings and redraw the thread at the width it names. | `822-02-column`, `822-03-full-again` |
| REQ-004 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the default, the toggle, the doc string; a review of the diff; `script/gates.sh
  --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, guide, the ledger row for the Zed files (§19), close the ticket,
  archive, commit.
