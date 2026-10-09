# Allowed actions in Settings — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-711-allowed-actions-in-settings.md
- **Pipeline spec:** 711-allowed-actions-in-settings.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-09: "lets do the small things".
- **Recall (§18.3):**
  - #704 added `MarleyAgentControlMode`'s dropdown through `add_basic_renderer`.
  - #707 shipped `actions_allowed` as JSON only.
  - Zed's string lists are `.unimplemented()`.
  - `marley_page.rs` is a child module of `settings_ui`, so it reaches its private helpers.
- **Design:**
  - `settings_content/src/marley.rs`: `MarleyActionNames(pub Vec<String>)`, transparent, with a
    `MergeFrom` that replaces; `actions_allowed: Option<MarleyActionNames>`.
  - `settings_ui/src/marley_page.rs`:
    - `render_action_names` takes the basic-renderer signature.
    - It reads the current list from the file (`get_value_from_file`).
    - Rows: `Label` plus a muted "not an action" when `cx.all_action_names()` lacks the name,
      and an `IconButton` (`IconName::Close`) that writes the list without it through
      `update_settings_file`.
    - Below the rows, a `SettingsInputField` (id `marley.agent_control.actions_allowed`,
      placeholder `editor::SelectAll`, confirm button) whose confirm appends.
    - The item goes in `agent_control_section` (array length 6).
  - `settings_ui/src/settings_ui.rs`: one line,
    `.add_basic_renderer::<settings::MarleyActionNames>(marley_page::render_action_names)`.
  - `marley_workbench/src/action_tools.rs`: `user_allowed` maps `.0`.
- **File manifest:**
  - Zed paths: `settings_content/src/marley.rs`, `settings_ui/src/marley_page.rs` and
    `settings_ui/src/settings_ui.rs`; their rows are extended first.
  - Marley crate: `action_tools.rs`.
  - Docs: the guide.
- **Visual check plan:** the spec's three shots, and settings.json read after each step.
- **Risk:** the renderer's place in the item. A basic renderer's control sits at the row's
  right, and a vertical list there may crowd it; 711-01 shows the layout.

## Phase 2 — Code
- **Built:**
  - `MarleyActionNames` (transparent, `MergeFrom` replaces via `clone_from`) as `actions_allowed`'s
    type. The rows were extended first.
  - `render_action_names` in `marley_page.rs`, registered with one line in `settings_ui.rs`.
  - The Allowed Actions item (the section's array is 6).
  - `action_tools::user_allowed` maps `.0`.
  - The guide.
- **Deviations:**
  - `cx.global::<SettingsStore>()` replaces `SettingsStore::global` (not in scope there).
  - The field clears after a confirm (`clear_on_confirm`), so the next name starts empty.
- **Review:**
  - Each remove and the add write the whole list through `update_settings_file`, as Zed's fields
    do, so the settings file stays the one source.
  - A blank or duplicate name is dropped before the write.
  - Ids: `("marley-allowed-action-remove", index)` per row, and the field's id is the JSON path.
- **Gate:** `711-gate-1.log` GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/711-allowed-actions-in-settings.sh`, under `compositor sway`. The
  profile names `editor::SelectAll` and `nothing::Here`; the Settings window's search finds the
  item.
- **Run a (`shots-711a`):** 711-01 already showed the editor working. The guessed clicks missed,
  since the shot is 1600 wide, so the field and the × were measured from it.
- **Run b:** the add passed. The remove missed: the list moves up about 14 px when it grows, so
  the × is measured from 711-02, the state at the time of the click.
- **Run c: a bug.** Both checks passed, but 711-03 showed a stale tooltip, "Remove
  nothing::Here", over `pane::SplitRight`'s ×. The buttons were keyed by row index, so after a
  removal the next row inherited the old button's hover and tooltip.
  - **Fixed at source:** each button is keyed by its name (`marley-allowed-action-remove-<name>`).
  - **Gate:** `711-gate-2.log` GATE GREEN [diff].
  - The scenario now moves the pointer off before the last shot.
- **Run d (`shots-711d`): both checks pass, and every shot shows its criterion.**
  - **711-01 (REQ-001):** `editor::SelectAll ×` and `nothing::Here` with "not an action" and its
    ×, then the empty field (placeholder `editor::SelectAll`).
  - **711-02 (REQ-002):** `pane::SplitRight` added as the third row, the field cleared; the
    settings file reads `["editor::SelectAll", "nothing::Here", "pane::SplitRight"]`.
  - **711-03 (REQ-003):** `nothing::Here` gone, two rows left, no stray tooltip; the settings
    file reads `["editor::SelectAll", "pane::SplitRight"]`.
- Chad's Hyprland untouched.

## Phase 4 — Complete
- **Documented:**
  - `CHANGELOG.md` (Added);
  - the guide (Phase 2);
  - the three touchpoint rows describe the shipped newtype, renderer and item.
  - No Marley crate's architecture note changed beyond `user_allowed`'s `.0`.
- **Knowledge appended:**
  - F-claude-711-a-removed-rows-button-left-its-tooltip-on-the-next-row-001;
  - PR-claude-711-key-a-list-rows-elements-by-what-the-row-is-001.
- **Brain:** no `rusty` MCP server in this repository's sessions.
- **Ticket:** closed; it never had a BACKLOG row.
- **Gate:** `711-gate-3.log`, GATE GREEN [diff], on the tree committed.
