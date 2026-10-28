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
  - The brain (2026-10-28, consultation be21e90ead6f483894206904997f5523): nothing on this seam.
  - Promoted from `queued/` on 2026-10-28; every cited seam re-checked at HEAD c95665660f and holds.
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
- **Built** (`crates/marley_workbench/src/system_one_calls.rs`, Marley crate):
  - `USES`, ten `SystemOneUse { key, name }` in the board's order. `MODES` and `mode_label` hold
    the track's four buttons.
  - `render_uses` (an associated function) draws "Each use and its mode", then one row per use:
    the name, and a bordered track drawn like `settings_ui.rs`'s Marley rows (the set mode
    `Filled`/`Default`, the rest `Subtle`/`Muted`). The modes are read from the `SystemOne`
    global's merged `uses`, so they show while the layer is off (D4).
  - A click runs `set_use_mode`, which goes through `settings::update_settings_file` into
    `marley.system_one.uses`. The view already observes the global that follows the settings.
  - The title and tab are "System One", with the opening line under the title. The order is
    title, cards, uses, buttons, the calls heading ("System One calls" with the muted note), then
    the rows. A failed row gets `status().error_background`.
- **Zed crate:** `crates/settings_ui/src/marley_page.rs`, the link's `title` and `button_text`.
  Its `zed-touchpoints.md` row was extended first.
- **Deviations:**
  - The headings are `Headline` `XSmall`, not plain labels.
  - A click on the mode already set writes nothing.
- **Review:** no entity updates inside another's update (a button's `on_click` writes settings
  with the `App` only). No `bool` parameters, no tuples in signatures, named-field struct for a use.
- **Gate, first run:** gate:2 failed on `too_long_first_doc_paragraph` (the module doc, now split)
  and `needless_pass_by_ref_mut` (`set_use_mode` takes `&App`).
- **Gate, second run:** `just gate-diff` GREEN, receipt written.

## Phase 3 — Test
- **Scenario:** `script/e2e/937-system-one-page.sh` (sway).
  - Setup writes `$E2E_PROFILE/system_one/calls-<today>.jsonl` with three `"row":"call"` lines
    (stop_kind, stall_kind, and a failed click_consequence) and #847's `marley.system_one`
    profile setting.
  - The first run (`STOP_AT=1`) measured Stop kind's Act at (1298, 336).
  - The second run's link search, "open system one", matched other rows and left the link below
    the fold. The third run's "spend against the budget" matched nothing: the Settings search
    indexes an `ActionLink` by its title alone (`settings_ui.rs:2559`). The fourth run searches
    "system one" and scrolls to the end. The fourth run is the one read below.
- **`937-01-page` (REQ-001, REQ-002, REQ-004, REQ-005):**
  - The tab and title read "System One", with the opening line under the title in muted text.
  - The Provider card reads "rules · jev-1.13.0" and "Key: not needed". The Today card reads
    "0¢ of 50¢" and "3 calls".
  - "Each use and its mode" lists the ten uses in the board's order. The check is on Act, Stop
    kind on Suggest, Stall kind on Shadow, Click consequence on Act, and the rest on Off, each the
    filled button.
  - Then Run Check, Set Key and Forget Key, and "System One calls" with "newest first · what
    left the machine is shown whole".
  - Three rows follow. The click_consequence row has the dark red background and "the call
    failed" in red.
- **`937-02-changed` (REQ-003):** after a click on Act, Stop kind's Act is filled and Suggest is
  not. The check `stop_kind` "act" in the run's `settings.json` passed.
- **`937-03-settings` (REQ-003):** the Settings window searched for "stop kind". Stop Kind shows
  "changed · reset" with Act filled; Stall Kind shows Shadow.
- **`937-04-link` (REQ-006):** the System One section's last item is titled "System One", with
  the button "Open System One ↗".
- **`937-05-day` (REQ-001, REQ-002, REQ-004, REQ-005):** the page in Marley Day, with the modes
  filled (Stop kind now on Act) and the failed row on a light red background.
- **Seen, not in scope:** the page opens in the Home group (#676), so the left pane shows the
  project panel's empty state. That has been so since #676.

## Phase 4 — Complete
- **Documented (§21):**
  - `CHANGELOG.md`: an Added entry for #937.
  - `docs/marley/look-plan.md`: the System One row names #937.
  - `docs/marley_architecture/marley_workbench.md`: "The System One page (#937)".
  - `docs/marley/zed-touchpoints.md`: the `marley_page.rs` row names the link's words (written in
    Code).
- **Knowledge (§19):** L-claude-937-the-settings-search-reads-an-action-links-title-only-001. No
  `F-` block, no `AD-` block.
- **Brain:** consultation be21e90ead6f483894206904997f5523 closed with `brain no-decision`.
- **Ticket:** closed. No row was left in BACKLOG.md.
