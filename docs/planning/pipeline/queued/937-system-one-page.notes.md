# A System One page — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-937-system-one-page.md
- **Pipeline spec:** 937-system-one-page.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11, SystemOne).
- **Classification / tier:** feature, M. Marley crate (`marley_workbench`) plus a two-string hunk
  in `settings_ui`'s Marley page.
- **Recall (§18.3):**
  - L-claude-565 (a scenario reaches the user's own keyring): the scenario never presses Set Key
    or Forget Key; the `rules` provider needs no key.
  - L-claude-565 (a poll in setup dies under `set -e`): the setup writes the call file directly;
    it waits on nothing.
  - F-claude-565 (the check read its workspace inside that workspace's update): the mode buttons
    write settings from a listener, not from inside a workspace update.
  - #847's scenario turns System One on with `profile_setting marley.system_one '{…}'`; reuse it.
  - No failure names `system_one_calls.rs` itself.
- **Discovery:** `crates/marley_workbench/src/system_one_calls.rs`: `Render for
  SystemOneCallsView` (title, opening line, order of blocks, subheading), `render_row` (the
  tint), a new `render_uses`, `Item::tab_content_text`. `crates/settings_ui/src/marley_page.rs`
  `:2182-2184` (the link's two strings).
- **Decisions:** D1–D5 in the spec.

### Design
- A list in `system_one_calls.rs`: `const USES: [SystemOneUse; 10]`, where `SystemOneUse` is a
  struct with named fields `key: &'static str` (the settings key: `check`, `stop_kind`,
  `browser_find`, `terminal_find`, `stall_kind`, `click_consequence`, `inbox`,
  `question_route`, `running_error`, `typed_line`) and `name: &'static str` (the board's name).
- `render_uses(&self, cx)`: reads the modes from the `SystemOne` global's settings when the layer
  exists, else from `SystemOneSettings::from_content` of the Marley settings (the page draws
  modes also while the layer is off, D4). Each row is an `h_flex` with the name and a track like
  `settings_ui.rs:5285-5330`'s: one `Button` per `SystemOneMode::VARIANTS`, ids
  `("system-one-mode", use index * 4 + mode index)`.
- A click: `settings::update_settings_file(<dyn Fs>::global(cx), cx, move |content, _| …)`
  inserting `key → mode` into `content.marley…system_one…uses`. The page already observes the
  `SystemOne` global, which follows the settings, so it redraws; a settings observer is added
  only if the global is absent while the layer is off.
- The subheading: an `h_flex` with "System One calls" (Small, default colour) and the muted note,
  placed before the rows and before "No calls today.".
- The tint: `.when(row.failed, |row| row.bg(cx.theme().status().error_background))` on the row's
  `v_flex`; the hover colour stays for the rest.
- File manifest:
  - `crates/marley_workbench/src/system_one_calls.rs` (Marley crate).
  - `crates/settings_ui/src/marley_page.rs` (Zed crate path, Marley's file in it): the link's
    title and button text. Its row in `docs/marley/zed-touchpoints.md` (the `marley_page.rs` row)
    gets "the System One link reads System One / Open System One (#937)".

### Visual check plan
- REQ-001: open the page from the palette; `937-01-page` shows "System One" as the title and the
  tab, the opening line under it.
- REQ-002: `937-01-page` shows the ten rows, Stop kind on Suggest, Stall kind on Shadow, Click
  consequence on Act, The check on Act, the rest Off.
- REQ-003: click Act on Stop kind (place from the first run's shot); `937-02-changed` shows it
  filled; open Settings, search "stop kind"; `937-03-settings` shows Act.
- REQ-004: `937-01-page` shows the subheading over the three rows.
- REQ-005: `937-01-page` shows the failed row on the error background; `937-04-day` the same in
  Marley Day.
- REQ-006: `937-03-settings` with the search cleared or "system one" searched shows the link's
  words.

### Risks
- #941 puts every Marley page title in sentence case; if it lands first, the link reads "System
  One" and "Open System One" in its case.
- The page grows by ten rows; the calls may fall below the window at 1080 px. The scenario's
  three calls should still be in view; if not, the shot is taken after a scroll and named.
- Writing settings rewrites `settings.json`; the run uses a profile copy, never the user's file.
- The call file's name and row shape must match `files::day_file` and `Row::Call`; the setup
  copies one line shape from `files.rs`'s own serde names (`"use"`, not `use_name`).

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
