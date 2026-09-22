# open Files/Code/Git panes on demand (M6 openers) — Notes

- **Forge ticket:** #128 `3671a317-44bc-4677-b9c1-1a245cb9958c` · **AAR:** `2c2ea08f-1dce-4e1b-8824-71d3597cb0f2`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-128-openers.md

## Phase 1 — Plan
- **Request:** forge #128 (M6 run 7/10) — open_or_focus (pure) + a Files opener; code/git openers exist (#124/#125).
- **Pre-flight:** open_code_pane (#124)/open_git_pane (#125) use first_pane_of_kind; cockpit_commands (1680);
  command dispatch strings (~1472).
- **Decisions:** D1 Files/Git singletons focus-or-open; D2 "Files" command → FileTree pane.
- **AAR id:** `2c2ea08f-1dce-4e1b-8824-71d3597cb0f2`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **workspace.rs (PURE):** OpenAction{FocusExisting(PaneId),OpenNew}; open_or_focus(target)=first_pane_of_kind(target)→FocusExisting else OpenNew.
- **app.rs (SHIM):** open_files_pane (match open_or_focus(FileTree){FocusExisting(id)→focus; OpenNew→open_pane(FileTree)}; persist_grid); refactor open_git_pane to use open_or_focus (DRY); a "Files" cockpit command (new CommandId + dispatch string → open_files_pane).
- **Test plan:** open_or_focus_focuses_or_opens (Git pane→FocusExisting(id); none→OpenNew; FileTree same).
- **Risks:** the palette CommandId↔dispatch-string mapping — add "Files" consistently. Opener not live-drivable (palette=synthetic input); code-reviewed.

## Phase 3 — Implement
- **Built (workspace.rs PURE):** OpenAction{FocusExisting(PaneId),OpenNew} + open_or_focus(target). **(palette.rs PURE):** action_for_command CommandId(6)→"open-files" + its test assertion. **(app.rs SHIM):** open_kind_pane(kind,content) via open_or_focus (DRY); open_git_pane + open_files_pane delegate to it; a "Files" cockpit command (CommandId 6) + dispatch "open-files"→open_files_pane. OpenAction imported.
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (a tiny pure decision + a DRY of the openers + a palette command).
- **Lenses — no findings:** open_or_focus = first_pane_of_kind→FocusExisting else OpenNew (both arms); open_kind_pane matches both arms (focus vs open_pane) + persist_grid; open_git_pane/open_files_pane delegate. The "Files" command resolves via action_for_command(6)→"open-files"→dispatch→open_files_pane (the cockpit_commands_resolve test enforces the mapping). No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** open_or_focus_focuses_or_opens (OpenNew when none, FocusExisting(id) when a pane of the kind exists) + action_for_command CommandId(6) assertion. Pass.
- **Self-test:** the "Files" opener is palette-triggered (synthetic input, ENV-BLOCKED) → code-reviewed; the FileTree pane render is already live-proven (#121/#123 captures).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG; forge #128 → done. **M6 7/10.** open_or_focus (pure) + a Files palette command + DRY'd openers. Every pane kind user-openable. cov/MSI 100.
