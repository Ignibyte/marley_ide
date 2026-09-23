# Rail persistence and polish — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-442-rail-polish.md
- **Pipeline spec:** 442-rail-polish.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** the remainder of the plan's rail (workbench-shell W6).
- **Classification / tier:** feature, medium; `marley_workbench` only.
- **Recall (§18.3):** `PR-claude-live-refresh-selection-identity-key-must-be-unique-001`
  (filtered rows keep their item-id keys); `PR-claude-park-the-pointer-before-key-driving-hover-selecting-overlays-001`
  (the switcher under a hovering pointer); `PR-claude-gpui-keyboard-focus-needs-a-mouse-down-not-a-raise-001`
  (focus in live drives); the gpui-era rename and search tickets (#112, #177) in
  `docs/planning/pipeline/completed/`.
- **Discovery:** the sidebar sweep of 2026-09-22 (the switcher needs at least two entries, the
  filter editor, restore ordering at `workspace.rs:10336-10352`).
- **Human confirmation:** Chad's goal authorizes autonomous execution through commit
  (2026-09-22). No `TaskCreate` in this harness; checklists live here.

## Carried from #438's inspect (2026-09-22)
- The git branch on a project header: `project.active_repository(cx)` then the repository's
  `branch.name()` (as `sidebar.rs:2692-2698` reads it), refreshed from the git store's
  `RepositoryUpdated(_, HeadChanged | GitWorktreeListChanged, _)` events.
- Groups with no open workspace (Zed keeps a group after its last workspace closes): list them
  and reopen on click (`find_or_create_workspace`).
- Restore order: in the Marley layout the rail opens during window creation, which serializes
  the window's state before `apply_restored_multiworkspace_state` restores its project groups;
  when the rail was closed at the last quit, that partial state stays until a later serialize.
- `open_sidebar` records a "Sidebar Toggled" telemetry event for every window opened in the
  Marley layout and every swap; upstream's silent `restore_open_sidebar` is `pub(crate)`.
- Keyboard: while the rail is registered, `NextProject`, `NextThread` and the thread switcher
  reach the trait's no-op defaults.
- Swaps and subscriptions: Zed's `register_sidebar` has no unregister and adds two
  subscriptions per call, so every layout swap leaves a dead pair for a dropped rail and each
  round trip adds a live pair on the kept Zed sidebar (#438 inspect S6).
- A layout round trip with the agent panel open can close the right dock: the agent panel
  becomes the right dock's active panel, and moving it back closes the dock (`dock.rs:638-700`,
  `:895-918`). Record each dock's visibility across the swap and restore it (#438 inspect S9).
- The rail rebuilds its snapshot on each `Wakeup` and `UpdateTab` from every listed terminal,
  two per chunk of output; refresh on the events that change a row (#438 inspect G10).
- #438 shipped the swap with Zed's sidebar kept alive: while the rail stands in, it answers
  `serialized_state` with the kept sidebar's state, or with the blob restored into a window
  that opened in the Marley layout. The width design here (D1) must add the rail's fields to
  that blob rather than replace it, or a switch back to Zed loses Zed's saved state.
- The workbench-shell plan's restart check moved here from W2: restart with both center
  terminals and Terminal Panel terminals open, and check both kinds come back, since the
  workspace and the Terminal Panel each clean the shared `terminals` table with only their own
  item ids (`workspace.rs:7961-7982`, `terminal_panel.rs:359-378`).

## Carried from #441's promotion (2026-09-22)
Two items moved here from W5 when #441 was narrowed to routing:
- **The first-show terminal.** A project first shown in the Marley layout with no center
  terminal gets one at its root. The risk: a restored workspace adds its terminals after the
  rail's first read, which would double them. Seed only once the workspace's items are
  restored, or only for a project opened fresh.
- **Zed's Panel Layout presets** (carried from #438, see #441's notes). `UseClassicLayout` and
  `UseAgenticLayout` misread the Marley layout. The title bar hides them through
  `CommandPaletteFilter` when AI is off, and reapplies that on every settings change, so a
  Marley-side hide would be undone. The fix is either a title-bar touchpoint or catching the
  two actions in the capture phase in the Marley layout (as #441 does for the terminal
  actions), with a message instead of the write.
