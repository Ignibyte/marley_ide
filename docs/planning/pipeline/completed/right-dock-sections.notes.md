# right-dock sections — Notes

- **Forge ticket:** #90 `d9ff256c-ea0c-4130-9ae9-626688d6c8ef` · **AAR:** `0423389e-798b-4e91-a1e9-98ca9f99a8b4`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-090-right-dock-sections.md

## Phase 1 — Plan
- **Request:** forge #90 (M2.F 1/6, FOUNDATION) — tabbed right dock (Details/Agents/Forge).
- **Pre-flight:** the right dock renders `details` (block_details #58) inside `dock_panel(Right,…)` @ app.rs
  ~1612; modules declared in lib.rs (alphabetical); RootView has forge_open/fleet_open (the overlays stay).
- **Decisions:** D1 RightSection+section_tabs/label; D2 accent-active tab; D3 Agents/Forge placeholders (#91/#92).
- **AAR id:** `0423389e-798b-4e91-a1e9-98ca9f99a8b4`.

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
- **right_dock.rs (NEW):** `enum RightSection {Details,Agents,Forge}` (Debug/Clone/Copy/PartialEq/Eq); `section_tabs() -> [(RightSection,&str);3]` fixed order; `section_label(RightSection)->&str` match. Used by app.rs: section_tabs for the strip, section_label for the placeholder headers (so BOTH are consumed — avoids dead_code on a pub-in-private-mod fn).
- **lib.rs:** `mod right_dock;` (after prompt).
- **app.rs:** `RootView.right_section: RightSection` (=Details in new()); the right-dock render builds a column: (1) a TAB STRIP = flex row over section_tabs(), each a clickable div (active [==self.right_section] bg colors.accent + on_accent text / else surface+muted), on_mouse_down(Left) → `view.right_section = section; cx.notify()`; (2) the BODY switched on self.right_section — Details → the existing block_details render (extracted to `details`); Agents → muted `format!("{} — ⌘⇧E", section_label(Agents))`; Forge → muted Forge placeholder. Wrap the column in dock_panel(Right,…).
- **Mutation targets:** section_tabs order/labels, section_label arms.
- **Test plan:** section_tabs_in_order (==the 3 tuples); section_label_per_section (3 arms). cov/MSI 100. The tab-strip render + click are masked (self-test/structure).
- **Risks:** the overlays (#68/#64 ⌘⇧E/⌘⇧F) STAY as quick toggles; #90 only adds the dock rail. Agents/Forge placeholders are #91/#92-filled.

## Phase 3 — Implement
- **Built:** right_dock.rs (RightSection enum + section_tabs()[3] + section_label()); `mod right_dock` in lib.rs; app.rs `right_section: RightSection` (Details in new()) + the right-dock render gains a tab strip (3 clickable tabs, active bg accent) over a body switched on right_section (Details → block_details #58, Agents/Forge → muted placeholders via section_label). Both section_tabs + section_label consumed (no dead_code).
- **Verification:** fmt; check --all-targets 0 err; clippy OK; both section tests pass.

## Phase 3.5 — Inspect
- **Method:** self-review (a small pure enum + 2 accessors + a masked tab-strip render mirroring the proven forge-row on_mouse_down #75).
- **Lenses — no findings:** section_tabs() fixed order Details/Agents/Forge (test asserts the whole array == the 3 tuples → kills an order/label mutant); section_label match (3 arms tested); the tab click sets view.right_section=section (section Copy, captured into the move listener) + cx.notify(); the body switches on right_section (Details unchanged = the exact #58 block_details Div); on_accent/accent for the active tab (WCAG-cleared palette #35). No panics. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** section_tabs_in_order + section_label_per_section (right_dock.rs) → 2 passed, cov/MSI 100 pending gate.
- **LIVE self-test (STATIC render PROVEN):** bundled + `open target/Marley.app` + screencapture (WIN 13796) → READ the PNG: the right dock shows the **3-tab strip — "Details" (active, cyan-accent bg + dark on_accent text), "Agents", "Forge" (muted)** — over the "No command selected" Details body. The #90 tab strip renders exactly as designed. **Click-switch ENV-BLOCKED:** the synthetic clickat did NOT switch the tab (the CGEvent synthetic-input degradation consistent all session, macOS TCC) — the click handler is engine-adjacent + mirrors the proven forge-row on_mouse_down (#75). Captures: scratchpad/dock90.png, dock90_agents.png.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(5); forge #90 → done. **M2.F 1/6 — the FOUNDATION.** right_dock.rs (RightSection + section_tabs/label, cov/MSI 100) + the tab strip. **LIVE static render PROVEN** (dock90.png: 3 tabs, Details active). Click-switch env-blocked (synthetic input).
