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
- **Promoted 2026-10-25:** every cited seam re-read after #904; `threads_page.rs`,
  `panel_tabs.rs`, `panel_group.rs`, `prompts.rs`, `marley_rail.rs` and `rail.rs` match. Home's
  Start moved inside `render_prompt` with #904 and still fills with `text_accent`.

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
- **Built:** the four fixes as designed: `threads_page.rs` (flex columns that wrap),
  `panel_group::board_label` and `PanelTab::new`, `prompts::primary_fill` (used by
  `render_button` and Home's Start), `marley_rail::GroupSummary` and `rail.rs` `project_name`.
- **Deviations:** the columns' row also scrolls and the list takes `max_h_full` in place of
  `h_full`, after the first narrow shot lost the preview below the row
  (F-claude-906-a-wrapped-column-fell-below-a-row-that-did-not-scroll-001). The preview keeps its
  left border when wrapped; it reads as its own block in the narrow shots, so no top border.
- **Review:** no entity read in its own update; no Zed hunk; Google style: `GroupSummary` has
  named fields in place of a string and a separate flag; `board_label` returns `Option<&str>`.
  `role`'s other callers (Threads' New Agent, Sessions, Settings' Apply, the agent prompts) take
  the coat through `render_button` too.
- **Gate:** `just gate-diff` 17/17 green.

## Phase 3 — Test
- **Scenario:** `script/e2e/906-small-daily-fixes.sh` (sway): #836's two CLI sessions, #827's
  stand-in Claude Code on the terminals' PATH through `.bashrc`, on a private bus.
- `906-01-open-summary`: the open `repo` header reads "1 waiting" in pink (REQ-004).
- `906-02-threads-full`: filters, list and preview side by side, a card selected (REQ-001).
- `906-03-threads-half`: the output narrowed to 930 px, the page about as wide as half the
  center: filters on the first line, the list under them with whole titles, the preview under
  the list (REQ-001). `pane: split right` did not split the page (it cannot split), so the
  output was narrowed instead.
- `906-04-panel-tabs`: tabs "Changes" and "Outline", their rail rows named the same (REQ-002).
- `906-05-primary`: Save in the coat with dark ink (REQ-003).
- `906-06-home-start`: Home's Start in the coat (REQ-003).
- `906-07-day`: Marley Day, narrow: the columns stacked, "Pick a conversation" under the list,
  the open header's "1 waiting" in pink (REQ-001, REQ-004).
- `906-08-day-primary`: Marley Day: Save in `text.accent` (#a94a0c) with light ink (REQ-005).
- The first narrow run lost the preview below the row (fixed, above).

## Phase 4 — Complete
- CHANGELOG (Fixed), `docs/marley_architecture/marley_workbench.md`. No Zed path, so no ledger
  row.
- Knowledge: F-claude-906-a-wrapped-column-fell-below-a-row-that-did-not-scroll-001.
