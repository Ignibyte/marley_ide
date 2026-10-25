---
pipeline_id: 1a89b855-4132-401c-beff-55f206b57050
ticket: docs/planning/tickets/open/TICKET-906-small-daily-fixes.md
status: Phase 4 — Complete PASS
title: Small daily fixes from the design review
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/README.md, docs/marley/design-review-2026-10-23/C-threads.md, docs/marley/design-review-2026-10-23/D-panels-editor.md, docs/marley/design-review-2026-10-23/B-rail-chrome.md]
---

## Title
Four fixes from the design review's first build-order line: the Threads page's columns flex and
wrap as the Threads board lays them out; the Git and Outline tabs say "Changes" and "Outline";
primary buttons fill with the coat under dark ink; an open project header in the rail shows its
summary, as a folded one does.

## Scope
### In
- **Threads columns** (Marley crate, `marley_workbench/src/threads_page.rs`, `ThreadsPage::render`):
  the row holding the filters, the list and the preview wraps. The filters take the board's
  `flex: 0 1 230px; min-width: 200px` in place of the fixed `w(px(200.))`; the list
  `flex: 999 1 480px; min-width: 0` in place of `flex_1`; the preview `flex: 0 1 380px;
  min-width: 300px` in place of the fixed `w(px(320.))`. In a narrow pane the preview drops under
  the list, and the list keeps its width.
- **Panel tab names** (Marley crate, `marley_workbench/src/panel_tabs.rs`, `PanelTab::new`): a
  panel tab takes its name from the panel group's board table (`panel_group.rs`, `BOARD`:
  `GitPanel` → "Changes", `Outline Panel` → "Outline"), falling back to the panel's tooltip as
  today. `board_place` (or a small `board_label`) becomes `pub(crate)`.
- **Primary buttons** (Marley crate, `marley_workbench/src/prompts.rs`, `render_button`,
  `Role::Primary`; `marley_workbench/src/home_page.rs`, `render_prompt`'s Start): the fill comes
  from one helper, `primary_fill(cx)`: in a dark theme the theme's `icon.accent` (the coat,
  #ff8c42 in Marley Night) under the dark ink `colors.background`; in a light theme
  `text.accent` as now (see D3). Every caller of `Role::Primary` gets it through
  `render_button`: the prompt card, the Threads page's New Agent, Sessions' New Session, Settings'
  Apply, the agent-control, client, launch and worktree-agent prompts.
- **Open header summary** (Marley crates: `marley_rail/src/marley_rail.rs`, `rail_rows` and
  `summary`; `marley_workbench/src/rail.rs`, `project_name`): `ProjectRow::summary` is filled for
  an open group too. It becomes `Option<GroupSummary>` with named fields `words` (the existing
  "1 waiting, 2 working") and `needs_you` (an agent waits, or a row under the group needs the
  user). The header draws the words after the name, in the conflict colour while `needs_you` or
  the folded group's `attention` holds, muted otherwise.

### Out (explicitly deferred)
- The folded header's summary as right-aligned pills with the "needs you" wording, and the
  Default header's home tile and "quiet": #910, which restyles the summary this ticket shows.
- An open header with nothing to count showing its row count ("5 rows", the Rail board): it sits
  in the same slot as "quiet", so #910 or a follow-up ticket takes it.
- The Inspector on by default: deferred by #837 (open decision 1, a choice beside #747's tabs,
  default off).
- The Threads page's header count, New Agent in the header, search placeholder, WHO tiles, card
  state and card shape, preview state line and facts: the review's build-order line 7, its own
  ticket.
- `ButtonStyle::Tinted(TintColor::Accent)` buttons (`send_block.rs`, `block_tables.rs`,
  `beside.rs`, `forge_board.rs`, `rail.rs`, `threads_page.rs:554`): tinted, not primary, so they
  stay.
- The Changes panel's layout and an Ask an agent button by Commit: build-order line 10.

## Reference (§20)
N/A — Marley-specific: the Threads page (#836), panel tabs (#747, #773), the prompt card's
button roles (#829's theme work) and the rail's group summary (#542, #827) are Marley's own
screens. Zed's panels keep their own tooltips ("Git Panel", "Outline Panel"); only Marley's tab
reads the board's name.

### Prior art
- `threads_page.rs:891-897`: the filters column, `w(px(200.))` and `flex_none`;
  `threads_page.rs:922-930`: the preview column, `w(px(320.))` and `flex_none`;
  `render_list` (`threads_page.rs:839-844`) is already `flex_1`, `min_w_0`, `overflow_y_scroll`.
- The Threads board (`Threads.dc.html`): the body is `display: flex; flex-wrap: wrap`; nav
  `flex: 0 1 230px; min-width: 200px`, main `flex: 999 1 480px; min-width: 0`, aside
  `flex: 0 1 380px; min-width: 300px`. #903 already lays Home's cards out as a wrapping row with
  `flex_basis` and growth in gpui (`home_page.rs`, `cards`), so gpui's taffy layout owns this.
- `panel_tabs.rs:397-416`, `PanelTab::new`: the name is `panel.icon_tooltip(window, cx)`,
  which Zed's panels answer "Git Panel" (`git_ui/src/git_panel.rs:9535`) and "Outline Panel"
  (`outline_panel/src/outline_panel.rs:5619`). `panel_group.rs:91-110`, `BOARD` and
  `board_place`, already name them "Changes" and "Outline" for the status-bar buttons (#862).
  A restored tab is built by the same `new` (AD-claude-773), so a restart gets the new name too.
- `prompts.rs:129-160`, `render_button`: `Role::Primary` fills with `colors.text_accent` under
  `colors.background`; `role` picks Primary for the first non-cancel, non-destructive action.
  `home_page.rs:384-414`: Start fills with `text_accent`.
- The theme (`assets/themes/marley/marley.json`): Night `text.accent` #ffb27a (Marmalade),
  `icon.accent`, `border.focused` and the cursor #ff8c42 (the coat); Day `text.accent` #a94a0c,
  `icon.accent` #b4500e (#828 kept Day darker than the board's #d9661a for contrast). No theme
  token holds the coat in both modes, and no Marley helper names it (grepped `coat`,
  `icon_accent` in `crates/marley_*`).
- Contrast, worked out (PR-claude-palette-accent-lightness-must-clear-wcag-vs-on-accent-001):
  #ff8c42 under Night's background #15110e is 8.1:1; #b4500e under Day's background #f3ebe2 is
  4.35:1 (under AA 4.5); #a94a0c under it is 4.85:1.
- `marley_rail.rs:1342-1381`, `summary`: the agents by state, most needing the user first;
  `marley_rail.rs:1809`: `(!project.expanded).then(|| summary(project)).flatten()`.
  `hidden_rows_need_the_user` (`:1962`) is false for an open group, so `attention` cannot colour
  an open header; `thread_needs_the_user` and `project_needs_the_user` (`:1975`, `:1981`) can.
  `rail.rs:10764-10790`, `project_name`: draws the summary after the name, conflict colour on
  `attention`. The RailStates board puts "1 needs you" in pink on an open header.
- The brain: nothing on these seams.

## UI proof
Scenario `script/e2e/906-small-daily-fixes.sh` (sway). Setup: a scratch repository with a
changed file and a Rust file; #836's two Claude Code CLI sessions in `CLAUDE_CONFIG_DIR` for the
Threads page; #542's stand-in Claude Code on the terminals' PATH through the scenario's
`.bashrc`, on a private notification bus with Marley's hooks (as #827), driven to wait on the
user.
- `906-01-threads-half`: the Threads page in the right half after `pane: split right`, a card
  selected: titles readable, the preview under the list or beside it (REQ-001).
- `906-02-threads-full`: the Threads page full width: filters, list and preview side by side
  (REQ-001).
- `906-03-panel-tabs`: the Changes and Outline status-bar buttons clicked: tabs titled "Changes"
  and "Outline" (REQ-002).
- `906-04-primary`: a changed buffer closed: the prompt card's Save in the coat with dark ink
  (REQ-003).
- `906-05-home-start`: Home's Start in the coat (REQ-003).
- `906-06-open-summary`: the rail with the project open while its stand-in waits: "1 waiting"
  in pink after the name (REQ-004).
- `906-07-day`: Marley Day: the Threads page, the prompt card's Save in `text.accent` with light
  ink, the open header's summary (REQ-001 to REQ-005).

## Locked-In Decisions
- D1 — The Threads columns take the board's flex values and wrap; no width breakpoint of our
  own. The list keeps at least the room the board's basis gives it before the preview wraps.
- D2 — One table names a panel in the status bar and on its tab: `panel_group::BOARD`. A panel
  the table does not name keeps its tooltip.
- D3 — Primary fills with the coat (`icon.accent`) in a dark theme and keeps `text.accent` in a
  light one: Day's `icon.accent` under its light ink is 4.35:1, under AA, and #828 already chose
  Day's darker accent for contrast. Links, labels and progress bars keep `text.accent`.
- D4 — An open header shows the same words as a folded one; this ticket changes no wording or
  shape. #910 restyles the summary into pills with "needs you".

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the Threads page sits in a half-width pane, its list shall keep room for readable titles, the filters and preview flexing and the preview wrapping under the list when the row runs out. | `906-01-threads-half`, `906-02-threads-full`, `906-07-day` |
| REQ-002 | WHEN the Git or Outline panel opens as a tab in the Marley layout, its tab shall read "Changes" or "Outline". | `906-03-panel-tabs` |
| REQ-003 | WHILE a dark theme is active, a primary button (the prompt card's first action, Home's Start) shall fill with the coat, `icon.accent`, under dark ink. | `906-04-primary`, `906-05-home-start` |
| REQ-004 | WHILE a project group is open and its agents have a state to count, its header shall show the summary after the name, in the conflict colour when one needs the user. | `906-06-open-summary`, `906-07-day` |
| REQ-005 | WHILE a light theme is active, a primary button shall keep `text.accent` under light ink. | `906-07-day`, the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the four fixes in the Marley crates; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
