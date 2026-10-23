# Small daily fixes from the design review — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-906-small-daily-fixes.md
- **Pipeline spec:** 906-small-daily-fixes.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, line 1 (what is wrong every day). This ticket takes four of its
  six items; the tab tiles and the doubled terminal icon are #905.
- **Classification / tier:** feature, S. Marley crates only (`marley_workbench`, `marley_rail`);
  no Zed hunk.
- **Recall (§18.3):**
  - AD-claude-773: a panel tab is saved as its panel's name and rebuilt through `PanelTab::new`,
    so the new name holds after a restart with no change to the saved row.
  - F-claude-747: a panel tab must not read its panel while the panel updates. The name is taken
    once in `new`, from a static table, so nothing new is read at draw time.
  - PR-claude-palette-accent-lightness-must-clear-wcag-vs-on-accent-001: contrast is worked out
    numerically. It settles D3 (Day keeps `text.accent`).
  - L-claude-828: to show Marley Day, write the theme into the profile copy's `settings.json`
    while Marley runs; no relaunch.
  - F-claude-834: a `flex_1` slot beside a group that takes the free width gets none. The Threads
    row avoids it by giving each column a basis, as the board does.
  - F-claude-903: stand-ins on the terminals' PATH through the scenario's `.bashrc`, never the
    box's real agent CLI.
  - #827's scenario: #542's stand-in Claude Code with Marley's hooks on a private bus puts an
    agent terminal in a waiting state; reused here.
- **Discovery:**
  - `crates/marley_workbench/src/threads_page.rs`: `ThreadsPage::render` (`:871-933`).
  - `crates/marley_workbench/src/panel_tabs.rs`: `PanelTab::new` (`:397-416`).
  - `crates/marley_workbench/src/panel_group.rs`: `BOARD`, `board_place` (`:91-110`).
  - `crates/marley_workbench/src/prompts.rs`: `render_button` (`:129-160`).
  - `crates/marley_workbench/src/home_page.rs`: `render_prompt`'s Start (`:384-414`).
  - `crates/marley_rail/src/marley_rail.rs`: `ProjectRow` (`:679-696`), `summary` (`:1342`),
    `rail_rows` (`:1809`).
  - `crates/marley_workbench/src/rail.rs`: `project_name` (`:10764`) and its caller (`:5824`).
- **Decisions:** D1 to D4 in the spec.

### Design
- Threads: in `render`, the columns' row gets `flex_wrap`; filters `flex_basis(px(230.))`,
  `flex_shrink`, `min_w(px(200.))`; list `flex_basis(px(480.))`, `flex_grow` with a large weight
  (the board's 999) or the rest of the row, `min_w_0`; preview `flex_basis(px(380.))`,
  `flex_shrink`, `min_w(px(300.))`. When the preview wraps, its left border turns into a top
  border so it still reads as a separate column (check by eye in the half-width shot).
- Panel tabs: `panel_group::board_label(persistent_name) -> Option<&'static str>` (from
  `board_place`), `pub(crate)`; `PanelTab::new` uses it before `icon_tooltip`.
- Primary: `prompts::primary_fill(cx: &App) -> Hsla`: `icon_accent` when
  `cx.theme().appearance()` is dark, `text_accent` when light. `render_button`'s `Role::Primary`
  and Home's Start use it; the ink stays `colors.background`.
- Summary: `pub struct GroupSummary { pub words: String, pub needs_you: bool }` in `marley_rail`;
  `summary` returns it with `needs_you` set when the waiting count is above zero or
  `project_needs_the_user` holds; `rail_rows` fills it for every group. `project_name` takes
  `Option<GroupSummary>` and the folded `attention`, and colours the words conflict when either
  holds.
- File manifest: `threads_page.rs`, `panel_tabs.rs`, `panel_group.rs`, `prompts.rs`,
  `home_page.rs`, `rail.rs` (Marley crate `marley_workbench`); `marley_rail.rs` (Marley crate
  `marley_rail`). No Zed path, so no row in `docs/marley/zed-touchpoints.md`.

### Visual check plan
- REQ-001: open the Threads page (`marley: open threads`) with #836's two CLI sessions, select
  the first card, run `pane: split right` and move the page into the right half
  (`906-01-threads-half`); close the other pane for the full-width shot
  (`906-02-threads-full`).
- REQ-002: click the Changes and Outline buttons in the status bar's panel group; the tab bar
  shows both titles (`906-03-panel-tabs`).
- REQ-003: type into the Rust file and close it with `pane: close active item`; the prompt card's
  Save fills with the coat (`906-04-primary`); open Home and shoot its Start
  (`906-05-home-start`).
- REQ-004: start the stand-in Claude Code in a terminal of the project and step it to waiting;
  the open header reads "1 waiting" in pink (`906-06-open-summary`).
- REQ-005 and Day: write `"theme": "Marley Day"` into the profile copy while Marley runs; reopen
  the close prompt and the Threads page in a half pane (`906-07-day`).

### Risks
- A wrapped row inside a column with `min_h_0` splits its height between lines; the list may get
  too little height when the preview wraps under it. If so, the preview takes its content height
  and the list the rest; the half-width shot decides.
- The tests in `marley_rail.rs` (`:2206` onward) build `ProjectRow { summary: None, … }`; they
  keep building with the new type, and their groups count no agents, so `None` stays right. Gate
  2 builds them (§7).
- `role` makes the first action primary even when it is not the default one; a prompt whose
  first button is a minor action would now be louder in Night. The scenario's prompt card is the
  common case; the diff review checks the other callers.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
