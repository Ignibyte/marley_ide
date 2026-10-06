# Rusty off leaves no trace — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-661-rusty-off-leaves-no-trace.md
- **Pipeline spec:** 661-rusty-off-leaves-no-trace.spec.md

## Phase 1 — Plan (2026-10-06)
- **Request:** Chad, 2026-10-06: the personal assistant is optional by one switch in one build;
  off must leave no trace. The Queue's top.
- **Classification / tier:** chore; `marley_workbench` (Marley) plus `settings_ui` (Zed crate:
  `marley_page.rs` is Marley's file in it; `settings_ui.rs` and `page_data.rs` take a hunk each,
  their ledger rows widened first).
- **Pre-flight:** green; no active pipeline; the README marker present; cargo idle.
- **Recall (§18.3):**
  - The 2026-10-06 survey: every `rusty` action is registered on every workspace and toasts while
    off; `marley::ToggleBrainView` is in the `marley` namespace; the Knowledge panel's dock button
    and the Brain switch already hide; the settings page shows the Rusty section whatever the switch.
  - R-D0 (rusty-in-marley.md:91-95): a `rusty:` action run while off shows a toast naming the switch.
  - The ledgers hold nothing on the palette filter or the settings window's rebuild; the brain
    (consultation `9c2787c8f1a74fff8bf72f761d238cde`) nothing on this seam.
- **Discovery:** `agent_ui.rs:684-697`, `:790-860` (`update_command_palette_filter`);
  `command_palette_hooks` (`CommandPaletteFilter`); `settings_ui.rs`: `SettingsPageItem`
  (`:1111-1117`), `filter_matches_to_file`, the `SettingsStore` observer (`:1842-1858`),
  `rebuild_pages`, the `FeatureFlagStore` observer that rebuilds; `page_data.rs:65-86`
  (`settings_data(cx)`, `marley_page()` without `cx`); `marley_page.rs:1403-1522` (`rusty_section`, six
  items: header, Rusty, Connection, Service URL, Rusty Tools for Agents, Rusty's Server).

### Design
- **`marley_workbench/src/rusty.rs`** (Marley): `update_palette(cx)` keeps the last applied value in
  the `Rusty` global (or a small global) and, on a change, calls `CommandPaletteFilter::update_global`:
  off hides the `rusty` namespace and `TypeId::of::<ToggleBrainView>()`; on shows both. Called from
  `rusty::init` and from the observer that already applies `MarleySettings` changes.
- **`marley_workbench/Cargo.toml`**: `command_palette_hooks.workspace = true`.
- **`settings_ui/src/marley_page.rs`** (Marley file in a Zed crate): `marley_page(cx: &App)`;
  `rusty_section(cx)` returns the header and the switch, then the other four while
  `rusty_on(cx)`; `pub(crate) fn rusty_on(cx)` reads the user settings' `marley.rusty.enabled`
  (false when unset).
- **`settings_ui/src/page_data.rs`** (Zed crate): `marley_page(cx)`.
- **`settings_ui/src/settings_ui.rs`** (Zed crate): in the `SettingsStore` observer, a Marley hunk
  that remembers `marley_page::rusty_on` and calls `rebuild_pages` when it changes.
- **File manifest:** `crates/marley_workbench/src/rusty.rs`, `crates/marley_workbench/Cargo.toml`
  (Marley); `crates/settings_ui/src/marley_page.rs` (Marley file, row `:63`),
  `crates/settings_ui/src/page_data.rs` and `crates/settings_ui/src/settings_ui.rs` (Zed, rows);
  `script/e2e/661-rusty-off-leaves-no-trace.sh` (Test).

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, 002 | Off at start; the palette, "rusty", then "brain view" | `661-01`, `661-02` |
| REQ-003 | `marley: open settings`, search "Rusty" | `661-03-settings-off` |
| REQ-004 | `profile_setting marley.rusty.enabled true`, the window still open | `661-04-settings-on` |
| REQ-005 | Close the Settings window; the palette, "rusty" | `661-05-palette-on` |
| REQ-006 | `profile_setting marley.rusty.enabled false`; the palette, "rusty" | `661-06-off-again` |
| REQ-007 | Not shot | Review |

### Risks
- **`rebuild_pages` resets the window's place**: rebuilt while the Marley page shows, the window may
  return to its first page or lose its search; the shot shows what it does.
- **The user's settings only**: `marley` lives in the user's settings, so reading the user file is
  the switch as resolved; a default other than false in `default.json` would need the merge.
- **A shown action type overrides a hidden namespace** in the filter: nothing in Marley shows a
  `rusty` action type, so hiding the namespace holds.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall, the brain.
- [x] Mint the pair; the BACKLOG row removed; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request to add the missing pieces.

## Phase 2 — Code (2026-10-06)
- **Built:**
  - The ledger rows for `settings_ui/src/page_data.rs` and `settings_ui/src/settings_ui.rs`
    widened first.
  - `marley_workbench/src/rusty.rs`: `PaletteShown` and `filter_palette(on, cx)`, called at the end
    of `follow_setting` (so at start and on every settings change): on a change it shows or hides
    the `rusty` namespace and `TypeId::of::<ToggleBrainView>()` through
    `CommandPaletteFilter::update_global`. Before the palette's filter exists it records nothing,
    so a later call sets it; Zed sets the filter up (`main.rs:697`) before Marley starts (`:897`).
    `command_palette_hooks` added to `marley_workbench`'s dependencies.
  - `settings_ui/src/marley_page.rs`: `marley_page(cx)`; `rusty_on(cx)` reads
    `marley.rusty.enabled` from the merged settings (false when unset); the Rusty section's first
    two items (the header and the switch) alone while off.
  - `page_data.rs`: `marley_page(cx)`. `settings_ui.rs`: the `SettingsStore` observer remembers
    `rusty_on` and calls `rebuild_pages` when it changes, as the `FeatureFlagStore` observer does
    for the staff flag.
- **Deviations:** none.
- **Review:** the keys bound to `rusty` actions keep their toast (REQ-007, D4); `rebuild_pages`
  runs only on the switch's change, not on every settings write.
- **Gate:** `just gate-diff` GREEN, 17 of 17, on the first run.

## Phase 3 — Test (2026-10-06)
- **Scenario:** `script/e2e/661-rusty-off-leaves-no-trace.sh` under `compositor sway`, the debug
  `marley`, Rusty off at start, the stand-in named for when it turns on.
- **Bug found in Test:** the first run's `661-04` showed the switch on and still nothing else of
  Rusty's. `rebuild_pages` runs `build_ui`, which matches the open search ("Rusty") against the
  search index before `rebuild_pages` builds the new index, so the new pages kept the old index's
  two matches. The Marley hunk now calls `update_matches` again after the rebuild. Clippy clean;
  the full gate runs at Complete.
- **Shots**, each read (second run):
  - `661-01-palette-off` (REQ-001): "rusty" typed: one fuzzy hit, `workspace: use agentic layout`;
    no `rusty:` command.
  - `661-02-brain-off` (REQ-002): "brain view" typed: No matches.
  - `661-03-settings-off` (REQ-003): the Settings window searched for "Rusty": Marley > Rusty, the
    section's header and the Rusty switch off, nothing else.
  - `661-04-settings-on` (REQ-004): the switch on from the settings file, the window still open:
    Rusty, Connection (Embedded), Service URL, Rusty Tools for Agents and Rusty's Server with
    Configure.
  - `661-05-palette-on` (REQ-005): "rusty" typed: open page (Ctrl-Alt-U), open tasks, open graph,
    open decisions, link task group, open local graph, link project page, toggle knowledge panel.
  - `661-06-off-again` (REQ-006): off again: "rusty" lists the one fuzzy hit and nothing of Rusty's.
- **Not shot:** REQ-007 (a bound key's toast), unchanged and by review.
- **Focus:** the palette and the Settings window took the keys; nothing reached the user's Rusty.

## Phase 4 — Complete (2026-10-06)
- **Documented:** `CHANGELOG.md` (Changed); `docs/marley/guide.md` (Rusty's opening);
  `docs/marley/rusty-in-marley.md` (R-D0); `docs/marley_architecture/marley_workbench.md` (the Rusty
  section); `docs/marley/zed-touchpoints.md` (the two `settings_ui` rows, the second naming the
  search re-match).
- **Knowledge:** F-claude-661-a-rebuilt-settings-page-kept-the-old-search-matches-001,
  L-claude-661-hide-a-feature-from-the-palette-with-zeds-filter-001,
  AD-claude-661-the-assistant-off-shows-nothing-but-its-switch-001.
- **Brain:** `brain decide` on consultation `9c2787c8f1a74fff8bf72f761d238cde`, follow up by
  2026-11-06: `decisions/marley-with-rusty-off-shows-nothing-of-rusty-but-its-switch`.
- **Closed:** the ticket in `tickets/closed/`; this pair in `completed/`.
