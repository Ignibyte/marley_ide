# Rusty's Tasks and brain pages as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-939-rusty-tasks-knowledge-brain-as-the-board.md
- **Pipeline spec:** 939-rusty-tasks-knowledge-brain-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11, Rusty's boards).
- **Classification / tier:** feature, M. Marley crate only (`marley_workbench/src/rusty/`). The
  ticket's list was L; the Knowledge panel's items are split off (D5).
- **Recall (§18.3):**
  - F-claude-867 (opening a project from a page read the page): a click that opens something from
    inside a view's update defers it; `decisions_tab.rs` opens the form with `window.defer`, and
    Tasks does the same.
  - F-claude-865 (a hook called from a render never reads its own view): the checkbox callback
    runs inside the `Markdown` entity's update; it captures the page view's `WeakEntity` and
    defers, as `on_url_click` does (`page.rs:1726-1733`, D9 of #875).
  - F-claude-903: the stand-ins go on the terminals' PATH; this run starts no terminal agent.
  - #903's and #875's scenarios: Rusty's stand-in with `MARLEY_RUSTY_MCP` and
    `RUSTY_STAND_IN_STATE`, a scratch vault, `tasks.json`; reuse their setup.
  - L-claude-839 (a uniform list cannot take a heading row): the tasks list is not a
    `uniform_list` today (cards of one height keep it so either way); check before Phase 2.
- **Discovery:** `rusty/tasks_tab.rs` (`render_side` at `:1079-1172`, `render_header`,
  `render_task`, the lists column); `rusty/follow_up.rs` (`open`, `FollowUpModal::new`, the
  after-Record step at `:208`); `rusty/decisions_tab.rs` (`:395-415`, the caller); `rusty/page.rs`
  (`markdown_element`, `render_outline`, `render_outline_button`, the page's load to read links);
  `rusty/properties.rs` (`value`, which needs the key); `rusty/brain.rs` (`:1335-1356`).
- **Decisions:** D1–D5 in the spec.

### Design
- `follow_up::open(workspace, summary, candidates, on_recorded: OnRecorded, …)`, where
  `OnRecorded` is `Rc<dyn Fn(&mut App)>`; the Decisions tab passes a closure that updates its
  `WeakEntity` and reads again; the Tasks tab passes one that re-reads its column (`read_side`).
- Tasks' due row: `Button::new(…, "Follow Up").style(ButtonStyle::Filled)` coloured as the accent
  primary (`prompts::render_button` with `Role::Primary`, the coat), then
  `ButtonStyle::Outlined` "Open the page". Follow Up's click defers `follow_up::open` on the
  workspace the tab keeps (`self.workspace`), candidates from `side.due` minus this one.
- Cards: wrap the row's content in a `div` with `border_1`, `rounded_md`, `p_2`,
  `border_color(colors.border_variant)`, `bg(colors.element_background)` or the selected colour;
  keep the `ListItem` inside for its slots, or draw an `h_flex` with the checkbox and title if the
  `ListItem`'s own hover fights the card's. The drag and drop stay on the outer `div`.
- Header: `Headline::new(name).size(HeadlineSize::Large)`; "LISTS" as an XSmall muted label;
  the swatch a `div().size(px(10.)).rounded(px(3.)).bg(color)` in place of `Indicator::dot`.
- Checkbox: `.on_checkbox_toggle(move |range, checked, window, cx| …)` in `markdown_element`
  while `Mode::Read` and the page is editable. The range is in the markdown's source; the page
  feeds `Markdown` its body text, so the offset must be mapped to the page's full text (the
  frontmatter's length added, if the body is fed without it). The view replaces the bracket at
  that range with `[x]` or `[ ]` and calls the save it uses for an edit.
- "Linked here": when a page is shown, a task calls `BRAIN_GET_LINKS` through the page's Rusty
  client (the Knowledge panel's reader in `marley_rusty::knowledge`), keeps `Vec<Backlink>` on the
  view, and `render_outline` draws them after the headings as `ListItem`s that open the page
  (`super::brain::open_page`). `render_outline` takes the backlinks as an argument, so the column
  shows also when the page has no headings but has backlinks; the Outline button's "No headings"
  rule follows the same "anything to show" test.
- Pills: `properties::value(key, value, cx)`; a `pill(text, foreground, background)` helper; the
  date test compares `follow_up_by` with today's local date (`chrono::Local::now().date_naive()`);
  see Risks on Rusty's own overdue flag.
- ★: `start_slot(Label::new("★").color(Color::Accent))`.
- Outline: `Button::new("rusty-page-outline", "Outline").toggle_state(…)` with the same
  `disabled`, tooltip and click.
- File manifest (all Marley crate): `crates/marley_workbench/src/rusty/tasks_tab.rs`,
  `follow_up.rs`, `decisions_tab.rs`, `page.rs`, `properties.rs`, `brain.rs`. No Zed path; the
  markdown hook already exists, so no `docs/marley/zed-touchpoints.md` row.

### Visual check plan
- REQ-001, REQ-003, REQ-004: open Tasks (`rusty: open tasks`); choose the
  "Work" list; `939-01-tasks`. `939-05-day` after switching to Marley Day.
- REQ-002: click Follow Up (place from the first run's shot); `939-02-form`; Escape.
- REQ-006 to REQ-009: open the decision page from Open the page; open the Brain tab's tree with
  the favourite; `939-03-page`.
- REQ-005: click the body's checkbox; `939-04-ticked`; the steps `grep -n '\[x\] Rusty cards'`
  the vault file into the run's log, read with the shot.
- Not reachable: a page changed under a toggle (Rusty's refusal); the save path's own handling is
  #875's and is checked in the review of the diff.

### Risks
- The checkbox source range may not line up with the page's text if `Markdown` gets a body
  without frontmatter or with Marley's rewrites (wikilinks turned into `rusty:` links). Map the
  offset against the exact string `Markdown` was given, and refuse the toggle if the bracket is
  not at that place.
- Cards are taller than rows; a long list scrolls more. The tab already scrolls.
- `marley_rusty::decisions` computes no date ("Rusty flags an overdue follow-up itself"). A
  page's properties carry no such flag, so the pill compares `follow_up_by` with today's local
  date. If Phase 2 finds `brain_render` or the page's frontmatter answer carries Rusty's flag, use
  the flag instead.
- The form's change of signature touches the Decisions tab; its scenario (#874) is not re-run
  (§7), so the review checks that tab's call by reading.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
