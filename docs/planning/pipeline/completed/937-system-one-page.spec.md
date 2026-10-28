---
pipeline_id: 1f5ad2b6-a469-4f3a-81ec-a97c4ff41ad3
ticket: docs/planning/tickets/open/TICKET-937-system-one-page.md
status: Phase 4 — Complete PASS
title: A System One page
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/F-tools.md, docs/marley/look-plan.md]
---

## Title
The System One calls tab becomes the System One page the SystemOne board draws: its title and
opening line, the Provider and Today cards, each use with its Off / Shadow / Suggest / Act mode,
and the calls under their own subheading, a failed call's row tinted.

## Scope
### In
- **Title and opening line** (Marley crate, `system_one_calls.rs`, `Render for
  SystemOneCallsView` and `Item::tab_content_text`). The page's title and its tab read "System
  One". Under the title, muted: "Small typed questions to a model about what Marley already
  knows. Off, it asks nothing and reads no key."
- **Each use and its mode** (`system_one_calls.rs`, a new `render_uses`). A heading "Each use and
  its mode", then one row per use in the board's order: The check, Stop kind, Browser find,
  Terminal find, Stall kind, Click consequence, Inbox risk, Question route, Running error, Typed
  line. Each row: the use's name at the left, and at the right a track of four buttons, Off /
  Shadow / Suggest / Act, the set one filled. A click writes `marley.system_one.uses.<name>` to
  the user's settings. The block sits after the Provider and Today cards and before the buttons.
- **The calls subheading** (`Render for SystemOneCallsView`). Over the rows: "System One calls",
  then muted "newest first · what left the machine is shown whole".
- **A failed call tinted** (`SystemOneCallsView::render_row`). A row whose call failed has the
  theme's error background; its reading stays in the error colour.
- **The Settings link's words** (Zed crate hunk, `crates/settings_ui/src/marley_page.rs`, the
  System One section's link item). "System One Calls" / "Open System One Calls" read "System One"
  / "Open System One". The action it dispatches keeps its name.

### Out (explicitly deferred)
- "Where its answers show" (the gallery of the rail row, the running-error block and the inbox
  mark): deferred by #847 ("those marks show in their own places already").
- The calls as a table with Time / Use / Mode / Answer / Cost / Took columns and a coloured mode:
  deferred by #847 ("the rows keep #565's lines").
- The bottom row for this page: no board draws one for System One; #909 owns the bottom row for
  screens other than the thread and the terminal.
- Renaming `marley: open system one calls`: keymaps, Home's Configure card and the Settings link
  name it; the action keeps its name.
- Removing the use rows from the Settings window: both places write the same key (D2).

## Reference (§20)
N/A — Marley-specific: System One is Marley's own layer (#565 onward) and its calls tab (#565,
#659, #847). The four-button track copies the look of #844's Marley rows in Zed's `settings_ui`
(`settings_ui.rs:5285-5330`), drawn in the Marley crate.

### Prior art
- `crates/marley_workbench/src/system_one_calls.rs:480`: the title "System One calls" as a Large
  label, no opening line; `:551-553` the tab title. Gap holds at HEAD.
- `system_one_calls.rs:194-290` `render_header`, `render_today`: the Provider and Today cards
  (#847) and the "System One is off" lines; kept as they are.
- `system_one_calls.rs:361-436` `render_row`: one line per call; `:411-417` only the reading
  turns red on `row.failed`. No subheading over the rows (`:500-507`). Gaps hold.
- `crates/settings_ui/src/marley_page.rs:1658` `system_one_section`, its ten use rows from
  `:1863` (Check, `:1863-1890`; Stop Kind, `:1894-1920`) writing
  `marley.system_one.uses.<name>`; `:2182-2184` the "System One Calls" link.
- `crates/settings_ui/src/settings_ui.rs:5285-5330`: #844/#847's four-button track for a Marley
  enum: a bordered, rounded `h_flex`, the picked button `ButtonStyle::Filled` with
  `Color::Default`, the rest `Subtle` and `Muted`. The page draws the same look.
- `crates/settings_content/src/marley.rs:724` `uses: Option<BTreeMap<String, SystemOneMode>>`;
  `:775-790` `SystemOneMode` (Off, Shadow, Suggest, Act); `assets/settings/default.json` sets all
  ten uses (`check` is `act`, the rest `off`), so the merged map always holds ten.
- `crates/marley_workbench/src/system_one.rs:84-160` `SystemOneSettings::from_content` keeps
  `uses`; `:883-888` `use_mode` reads it (Off while the layer is off).
- The write path: `settings::update_settings_file(<dyn Fs>::global(cx), cx, …)`, as
  `agent_activity.rs:200` and `assistant.rs:752` write Marley settings from a Marley view.
- The tint: `cx.theme().status().error_background`, the colour Zed's own error rows use.
- Specs: #565 (the layer, the scenario's `rules` provider), #847 (the cards, the buttons, and the
  two deferrals kept Out here).

## UI proof
Scenario `script/e2e/937-system-one-page.sh` (sway). Setup: a scratch repository; System One on
with the `rules` provider (no key, nothing leaves the machine), `stop_kind` suggest, `stall_kind`
shadow, `click_consequence` act, as #847's run sets them; today's call file in the run's data
folder `system_one/` holding three calls, one of them failed.
- `937-01-page`: `marley: open system one calls`: the title, the opening line, the cards, the ten
  uses with their modes, the calls subheading and three rows, the failed one tinted (REQ-001,
  REQ-002, REQ-004, REQ-005).
- `937-02-changed`: Act clicked on Stop kind: Stop kind's Act filled (REQ-003).
- `937-03-settings`: the Settings window searched for "stop kind": its row shows Act too (REQ-003).
- `937-04-link`: the Settings window searched for "system one" and scrolled to the section's end:
  the link reads "System One" and "Open System One" (REQ-006).
- `937-05-day`: the page in Marley Day (REQ-001, REQ-002, REQ-004, REQ-005).

## Locked-In Decisions
- D1 — The calls tab becomes the page; no second tab. Its action keeps its name.
- D2 — The Settings window keeps the ten use rows. The page and the window write the same key,
  so each shows what the other set.
- D3 — A mode click writes the user's settings file, never a project's (`marley.system_one` is a
  user setting, `files: USER` on the Settings rows).
- D4 — The page shows the modes as set, also while System One is off; the existing "System One
  is off: turn it on in the Marley settings." line says why nothing asks.
- D5 — The use names are the board's sentence-case names, kept in one ordered list in
  `system_one_calls.rs` with each use's settings key.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The System One page shall be titled "System One", in the page and on its tab, with the opening line under the title. | `937-01-page`, `937-05-day` |
| REQ-002 | The System One page shall show, after the Provider and Today cards, the ten uses in the board's order, each with Off / Shadow / Suggest / Act and the set mode filled. | `937-01-page`, `937-05-day` |
| REQ-003 | WHEN the user clicks a mode on a use's row, the system shall write that mode to `marley.system_one.uses.<name>` in the user's settings, and the page and the Settings window shall show it. | `937-02-changed`, `937-03-settings` |
| REQ-004 | The System One page shall head the calls with "System One calls" and "newest first · what left the machine is shown whole". | `937-01-page`, `937-05-day` |
| REQ-005 | WHERE a call failed, its row shall have the error background. | `937-01-page`, `937-05-day` |
| REQ-006 | The Settings window's System One link shall read "System One" and "Open System One". | `937-04-link` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `system_one_calls.rs`; the `marley_page.rs` words and their ledger row; a review
  of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
