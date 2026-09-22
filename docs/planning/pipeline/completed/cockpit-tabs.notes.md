# M9 seq-4 — cockpit as full-screen tabs — Notes

- **Forge ticket:** #153 `334bd265-51fe-4f75-9a57-a7f9671e28d5` · **AAR:** `296823c2-2f12-4234-95b5-d268434c7591`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-153-cockpit-tabs.md

## Phase 1 — Plan
- **Request:** forge #153 (M9 run 4/8) — cockpit → full-screen tabs; retire the right dock.
- **Pre-flight:** app.rs regions.right (~2330-2470) builds `body` (Details/Agents/Forge) in the right dock;
  top-right icons (~3620) set right_section + open docks[1]. seq-2 workspace() expects a terminal → a cockpit
  active tab would panic → the seq-4 guard.
- **Decisions:** D1 terminal_grid_index fallback (no panic, ≥1 terminal tab); D2 cockpit tab per-project
  (open_or_switch); D3 center render branches cockpit vs grid; retire the right dock cockpit.
- **AAR id:** `296823c2-2f12-4234-95b5-d268434c7591`.

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
### Approach
PURE helpers in tabs.rs + a masked render refactor in app.rs (move the cockpit body from the right dock into a
center full-screen branch; guard the grid accessor).
- **tabs.rs (PURE, cov/MSI 100):**
  - `Tab::cockpit_section(&self) -> Option<RightSection>` — Some for TabContent::Cockpit, None for Terminal.
  - `Project::terminal_grid_index(&self) -> Option<usize>` — `self.active` if that tab has a grid, else
    `self.tabs.iter().position(|t| t.grid().is_some())` (the first terminal tab).
  - `Project::tab_grid(&self, i) -> Option<&PaneGrid<S>>` + `tab_grid_mut(&mut self, i)`.
  - `Project::open_or_switch_cockpit(&mut self, section: RightSection)` — if a tab's cockpit_section == Some(section),
    switch_tab to it; else push Tab::cockpit(label, section) + activate. (label = a section name.)
- **app.rs (SHIM masked):**
  - `workspace()/workspace_mut()`: `let i = active_project.terminal_grid_index().expect(">=1 terminal tab");
    active_project.tab_grid(i)` (mut variant with tab_grid_mut) — never panics on a cockpit-active tab.
  - EXTRACT `fn cockpit_body(&self, section: RightSection, cx) -> gpui::Div` = the current right-dock body
    (Details: focused-pane detail_rows @~2355 + block_details; Agents: agent_rows Fleet; Forge: sprint list),
    self-contained.
  - CENTER render (~2463): `if let Some(section) = self.shell.active_project().active_tab().cockpit_section()
    { root=root.child(cockpit_body(section) sized to center_bounds) } else { <the existing grid render> }`.
  - RETIRE the right-dock cockpit block (~2330-2461: the `if regions.right>0.0 {...body...}` cockpit part) —
    remove it; the right dock no longer shows the cockpit.
  - Top-right icons (~3717): `view.shell.active_project_mut().open_or_switch_cockpit(section)` (drop
    right_section/docks[1]/persist_right_section for the cockpit open).

### File manifest
- `crates/marley_app/src/tabs.rs` — PURE: cockpit_section, terminal_grid_index, tab_grid/_mut, open_or_switch_cockpit + tests.
- `crates/marley_app/src/app.rs` — SHIM: workspace() guard; cockpit_body extraction; center render branch; retire right-dock cockpit; top-icon rewire.

### Regression Test Plan
| Test (tabs.rs, S=()) | AC |
|---|---|
| terminal_grid_index_cases — [T] active T→Some(0); [T,C] active C(1)→Some(0) first terminal; [C,T] active C(0)→Some(1) | REQ-001 |
| cockpit_section_cases — a cockpit tab→Some(section); a terminal tab→None | REQ-003 |
| open_or_switch_cockpit_cases — absent→append+active (tab_count+1, active=last, cockpit_section matches); existing→switch (no new tab, active=that index) | REQ-002 |
| DRIVEN: click a top-right cockpit icon → a cockpit tab fills the CENTER (rail shows it) | REQ-004 |
| DRIVEN: switch back to a terminal tab → the terminal renders, no panic | REQ-005 |

### Risks
- The big render MOVE (right dock → center) is masked — the DRIVEN capture is the proof; also confirm the grid
  still renders for a terminal tab (regression). Verify no OTHER code reads docks[1]/right_section for the
  cockpit after the rewire (search).
- terminal_grid_index.expect: assumes ≥1 terminal tab. Boot makes one; cockpit tabs are ADDED. Note; a future
  seq guards "all-cockpit" if tab-close lands.
- cockpit_body must stay self-contained (compute detail_rows inside) — the old code computed `details` above the
  dock block; move that in.

## Phase 3 — Implement
- **Built (tabs.rs PURE):** Tab::cockpit_section; Project::terminal_grid_index (active tab if terminal else first terminal), tab_grid/_mut, open_or_switch_cockpit (find cockpit tab of the section → switch, else append+activate).
- **(app.rs SHIM masked):** workspace()/workspace_mut() now go via terminal_grid_index+tab_grid[_mut] (never panic on a cockpit-active tab); NEW cockpit_body(section, colors, cx) method (the Details/Agents/Forge render, moved from the right dock, reads via workspace()); the CENTER render branches — active tab Cockpit(section) → cockpit_body full-screen + an EMPTY rect_list (no pane tiling), else the grid; the right-dock cockpit block DELETED; the top-right icons → open_or_switch_cockpit(section) (kept right_section+persist for the highlight; dropped docks[1]=Open).
- **Verification:** fmt; check --all-targets 0 err; clippy -D warnings OK.

## Inspect (Phase 3.5)
Method: 2 background critics (guard panic-safety; render move) + my review. Binary builds clean.

- **[panic] the workspace() guard — NO FINDINGS (critic A).** terminal_grid_index can't return None as wired:
  boot makes 1 terminal tab; +/⌘D add terminal tabs; open_or_switch_cockpit only ADDS cockpit tabs; ⌘W closes a
  PANE (grid refuses the last), not a tab; Project::close_tab has ZERO callers wired to input. Terminal-tab
  count is monotonic ≥1 → both expects unreachable. Borrows sound (usize idx is Copy; NLL ends the shared
  borrow before the mut). Build/clippy green.
  - **[FORWARD TRAP, noted]** When tab-close is later wired, closing the terminal in a [terminal, cockpit]
    project → [cockpit] → terminal_grid_index None → panic. close_tab's guard protects ≥1 tab of ANY kind, not
    ≥1 TERMINAL. Add a terminal-aware close refusal WHEN tab-close lands (seq-6 / a rail close button). Logged.
- **[fix] ⌘⇧B (toggle-right-dock) opened the now-empty right dock — FIXED.** The right-dock cockpit render is
  deleted, so opening docks[1] left an empty gap. Repurposed the action to open the Details cockpit TAB
  (open_or_switch_cockpit(Details)); the top-icon click already dropped docks[1]=Open. grep confirms no
  remaining docks[1]=Open opener; toggle_dock_state stays used by toggle-left-dock.
- **[render] cockpit_body move + center branch — my review clean (critic B fold-in at validate).** cockpit_body
  faithfully reproduces the dock body (workspace_mut→workspace for reads only); the deleted block's `focused`
  was scoped inside the `if` (the center uses the render-scope `let focused`, compile-confirmed); the center
  branch is mutually exclusive (cockpit → empty rect_list + full-screen body; terminal → grid); active_cockpit
  is Copy, computed before the else's workspace_mut.

Lenses: panic-safety, render fidelity, deletion safety, dock retirement, branch exclusivity. No blocking findings.

## Phase 4 — Validate
- **Tests:** terminal_grid_index_cases (REQ-001: active-terminal→active, active-cockpit→first terminal, all-cockpit→None; +tab_grid/_mut); cockpit_section_cases (REQ-003); open_or_switch_cockpit_cases (REQ-002: append/switch/append-different). Pass.
- **Self-test:** driven captures below (cockpit tab full-screen; return to terminal).
- **Gate:** first RED gate:4 (Forge label arm @tabs.rs:192 uncovered) → added a Forge open_or_switch case. Re-gate GREEN [diff] 15/15, cov/MSI 100. Captures: ck_details.png (Details cockpit tab fills the CENTER — "No command selected" — not a right panel; rail shows terminal 1[muted]/Details[active], REQ-004); ck_back.png (click terminal 1 → the terminal renders, no panic, REQ-005). Both critics NO FINDINGS; #159 filed (all-cockpit panic guard for when tab-close lands).

### Critic B (render move) — NO FINDINGS on all 4 checks
Body move faithful (keys off the `section` param not self.right_section — correct for the tab model; Forge's
`view`→`sprint` rename avoids shadowing); deletion safe (outer `let focused` @render top survives, used by the
pane loop + footer); center branch borrow-safe + exactly-one-draws (all rect_list consumers no-op when empty);
top-icon correct — confirmed `toggle_dock_state(Right)` is now fully unreachable (⌘⇧B rewired), so regions.right
is permanently 0, NO empty-gap. Non-blocking notes folded into the forward-trap ticket (#159): the latent
all-cockpit panic (both critics) + the vestigial right-dock code + the hidden-grid ⌘W/⌘D targeting (documented).

## Phase 5 — Complete
- CHANGELOG + app_shell seq-4 note; forge #153 → done. **M9 4/8.** Cockpit → full-screen tabs; right dock retired. Filed #159 (last-terminal-tab guard). LESSON: retiring a dock with ONE consumer → move the body to a method + branch the center (empty rect_list suppresses the grid); a per-variant match needs a test per arm (Forge label coverage).
