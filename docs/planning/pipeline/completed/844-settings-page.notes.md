# The Settings window's Marley page as the boards draw it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-844-settings-window.md
- **Pipeline spec:** 844-settings-page.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #843 (W5's first).
- **Classification / tier:** feature, M. Zed's `settings_ui.rs` (hunks), Marley's
  `marley_page.rs`.
- **Recall (§18.3):** #515 made the page; #837 added Layout's ninth row.
- **Discovery (Explore):** 11 sections, 83 items; Zed renders a flat virtualized list, enums as
  `EnumVariantDropdown`, reset as an Undo `IconButton` next to the title (`reset_to_default_fn`),
  "Modified in" as a muted label; 90 `marley` keys, 23 missing, 6 of them plain.
- **Decisions:** D1–D3 in the spec.

### Design
- **`settings_ui.rs` (Zed, additive):** `fn marley_row(json_path: Option<&str>) -> bool`;
  in `render_settings_item_layout`, for a Marley row: the reset as a `Button` "changed · reset"
  (XSmall, accent) and a mono XSmall muted key line under the description; in `render_dropdown`,
  for a Marley row with two or three variants, a row of `Button`s, the current one selected,
  writing as the dropdown does; in the page list's item wrapper, on the Marley page, a card
  frame (side borders and background for every item, the top rounded on a section header, the
  bottom rounded with a gap on the section's last row). Ledger row extended first.
- **`marley_page.rs`:** six rows in their sections (Agents: thread_details, review_notes_template;
  a Harness row group in Agents: embedded_harness, harness_writes; System One:
  cloudflare_api, price_cents_per_million_tokens), each section's array length raised.
- **File manifest:** `crates/settings_ui/src/settings_ui.rs`, `crates/settings_ui/src/marley_page.rs`;
  the scenario.
- **Visual check plan:** per the spec's UI proof; the Settings window opened with `marley: open
  marley settings`. Not reached: a key set in a project file ("Modified in").

## Phase 2 — Code
- **Built:** in `settings_ui.rs`, `marley_row(json_path)`; `render_settings_item_layout` moves a
  Marley row's reset into a "changed · reset" `Button` (XSmall, accent, tooltip Reset to Default)
  and adds the key under the description in the buffer font; `render_dropdown` draws a Marley
  row's enum of two or three variants as a framed row of buttons, the current one `Filled`,
  writing as the dropdown does; the page list wraps each item of the Marley page in a card frame
  (side borders and the panel background, the header's top rounded with a gap above, the
  section's last row's bottom rounded, Zed's extra bottom padding dropped inside the card);
  `use heck::ToTitleCase` for the buttons' labels. In `marley_page.rs`: Thread Details, Review
  Notes Message, Run the Harness, Harness Writes (Agents, 24 → 28), Cloudflare API, Price per
  Million Tokens (System One, 18 → 20).
- **Deviations:** the first segmented row drew plain text buttons, the picked one only coloured;
  the board's group has a frame and a filled pick, so it does now.
- **Gate:** green on the first run, 17 of 17.
- **Review:** the reset closure and the write path are Zed's own; a non-Marley row and every
  other page render as before.


## Phase 3 — Test
- **Scenario:** `script/e2e/844-settings-page.sh` (sway): `marley.rail_order` set to
  `attention`; a probe run placed Rail Order's marker.
- `844-01-layout`: the Settings window's Marley page: the Layout card; Layout (Zed | **Marley**)
  and Rail Order (**Attention** | Window) as framed choices; "changed · reset" on Layout, Rail
  Order and Containers Panel (the test profile sets those keys); each row's key in mono under
  its description; switches. REQ-001, REQ-002 met.
- `844-02-reset`: Rail Order's marker clicked: **Window** picked, the marker gone. REQ-002 met.
- `844-03-new`: the search "harness": the Agents card with Run the Harness
  (`marley.embedded_harness`) and Harness Writes (`marley.harness_writes`), and the Agent Control
  card. REQ-003 met (two of the six shown; the other four are the same kind of row).
- `844-04-day`: Marley Day, the same cards. REQ-001 met. (The first run pressed Escape after the
  search, which closed the window; the Day shot now keeps it.)
- Not reached: a key set in a project file ("Modified in").


## Phase 4 — Complete
- CHANGELOG (Changed); the page lives in `settings_ui`, so its ledger row and the look plan carry
  the architecture record.
- Knowledge: AD-claude-844-marley-rows-by-key-prefix-001. Brain: `brain decide` on consultation
  cc42cade.
- Ticket closed; pipeline archived.
