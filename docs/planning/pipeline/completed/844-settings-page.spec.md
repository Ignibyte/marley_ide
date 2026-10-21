---
pipeline_id: aa64ce82-8b70-495c-89d5-1957ab870a96
ticket: docs/planning/tickets/open/TICKET-844-settings-window.md
status: Phase 4 — Complete PASS
title: The Settings window's Marley page as the boards draw it
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The Settings window's Marley page reads as the Settings and SettingsAll boards draw it: each
section a card, each row its title, what it does and its key in small mono under it, a
"changed · reset" marker on a row set away from its default, switches for on and off, and
segmented choices in place of a dropdown for a choice of up to three; six plain keys missing from
the page get their rows.

## Scope
### In
- Rows whose key starts with `marley.`: the key line, "changed · reset" in place of Zed's Undo
  icon, segmented choices for enums of up to three variants.
- The Marley page's sections as cards (a bordered, rounded block from the section's header to its
  last row).
- New rows: `marley.thread_details`, `marley.embedded_harness`, `marley.harness_writes`,
  `marley.review_notes_template`, `marley.system_one.cloudflare_api`,
  `marley.system_one.price_cents_per_million_tokens`.
- Other pages keep Zed's look.

### Out (explicitly deferred)
- The lists and maps (`fleet.providers`, `tmux_hosts`, `harnesses`, `agent_session_meta`,
  …, 17 keys): no generic list renderer exists; they stay in settings.json (the page's "Edit in
  settings.json").
- The keymap editor: it already draws in the theme's colors; the board asks nothing more of it.
- The board's "You / This project" scope tabs (Zed's own file picker stays).

## Reference (§20)
Upstream Zed: `settings_ui` (`render_settings_item_layout`, `render_dropdown`, the page's
virtualized list in `SettingsWindow::render_page`; `reset_to_default_fn`). Marley's page is
`settings_ui/src/marley_page.rs` (#515 onward).

### Prior art
- The code we ship: Zed's reset closure and modified-in label, `EnumVariantDropdown`, `Switch`,
  the list's `is_last_in_section`; `ToggleButtonGroup` in `ui` for segmented choices.
- The brain (consultation cc42cade): nothing on this seam.

## UI proof
Scenario `script/e2e/844-settings-page.sh` (sway): the profile sets `marley.rail_order` to
`attention` (away from its default).
- `844-01-layout`: the Marley page's Layout card: Layout and Rail order as segmented choices,
  Rail order "changed · reset", the keys under each row, switches (REQ-001, REQ-002).
- `844-02-reset`: "changed · reset" clicked: Rail order back to Window, the marker gone
  (REQ-002).
- `844-03-new`: scrolled to the new rows (Thread details, Harness writes) (REQ-003).
- `844-04-day`: Marley Day, the Layout card (REQ-001).

## Locked-In Decisions
- D1 — A row is Marley's when its `json_path` starts with `marley.`; that is what the row-level
  changes key on, so a Marley key shown on another page looks the same.
- D2 — Segmented choices for an enum of two or three variants; more keep the dropdown.
- D3 — Cards are drawn on the Marley page only (its title), by the list's per-item wrapper.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Marley settings page shall draw each section as a card and each row with its title, description and key, enums of up to three choices as segmented buttons, in Marley Night and Marley Day. | `844-01-layout`, `844-04-day` |
| REQ-002 | WHEN a Marley row's value differs from its default, it shall show "changed · reset", and clicking it shall restore the default. | `844-01-layout`, `844-02-reset` |
| REQ-003 | The Marley page shall have rows for the six plain keys it lacked. | `844-03-new` |

## Phase Plan
- **P1 Plan**, **P2 Code** (`settings_ui.rs` hunks, `marley_page.rs`), **P3 Test**, **P4 Complete**.
