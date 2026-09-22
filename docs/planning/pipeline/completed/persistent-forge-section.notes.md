# persistent Forge section — Notes

- **Forge ticket:** #92 `2e809554-6495-4b7c-830b-c1c9ce2ea827` · **AAR:** `dd6db9cb-14bf-4aac-8a4c-1667175489f7`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-092-persistent-forge-section.md

## Phase 1 — Plan
- **Request:** forge #92 (M2.F 3/6) — the sprint view always-visible in the #90 Forge tab.
- **Pre-flight:** the ⌘⇧F overlay (app.rs ~2277) builds each sprint row with the trio on_mouse_down (copy
  #70/claim #75/comment #76); rows are `TicketRow`; render cx is `&mut Context<Self>`; forge_view exports
  sprint_rows/ticket_ref/status_glyph/forge_status_line; the #69 BG-thread keeps forge_sprint live.
- **Decisions:** D1 forge_ticket_row shared builder; D2 header+rows or forge_empty_hint; D3 trio unchanged.
- **AAR id:** `dd6db9cb-14bf-4aac-8a4c-1667175489f7`.

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
- **forge_view.rs (PURE):** `forge_empty_hint(loading: bool) -> &'static str` = `if loading { "loading sprint…" } else { "no active sprint" }`.
- **app.rs SHIM:** associated `#[cfg_attr(test, mutants::skip)] fn forge_ticket_row(row: &TicketRow, cx: &mut Context<Self>) -> gpui::Div` (NO self — the listener captures reference/ticket_id/row_number + calls view.*) built from the ⌘⇧F overlay row-block (trio unchanged); the overlay loop → `overlay.child(Self::forge_ticket_row(&row, cx))`; the #90 Forge-section body: `if let Some(view) = &self.forge_sprint { a flex_col: forge_status_line(Some(view)) header + each sprint_rows(view).take(30) via Self::forge_ticket_row } else { placeholder(forge_empty_hint(self.forge_pending.is_some())) }`.
- **Mutation targets:** forge_empty_hint loading-vs-empty arm.
- **Test plan:** forge_empty_hint(true)=="loading sprint…" + forge_empty_hint(false)=="no active sprint". cov/MSI 100. forge_ticket_row + the section render masked (static live + engine).
- **Risks:** the forge_ticket_row extraction MUST be output-identical (the overlay behavior unchanged); the trio (claim/comment WRITES #75/#76) is reused verbatim, not re-derived.

## Phase 3 — Implement
- **Built:** `forge_empty_hint(loading)` (forge_view); associated `RootView::forge_ticket_row(row, cx)` (mutants::skip, in impl RootView) — the shared trio-click row; the ⌘⇧F overlay loop + the #90 Forge-section body both call it; the Forge section shows forge_status_line header + rows when forge_sprint is Some, else forge_empty_hint(forge_pending.is_some()).
- **DEVIATION (dead-code cleanup):** #90's `section_label` became orphaned once #91/#92 replaced both dock placeholders (the tab strip uses section_tabs' own labels) → removed section_label + its test (YAGNI; the dock title already labels the pane). section_tabs_in_order stays.
- **Verification:** fmt; check --all-targets 0 err; clippy OK (TicketRow pub, import clean).

## Phase 3.5 — Inspect
- **Method:** self-review (a 2-arm pure hint + a masked shared-row extraction that reuses the proven M2.D trio verbatim).
- **Lenses — no findings:** forge_empty_hint = if loading "loading sprint…" else "no active sprint" (both arms tested); forge_ticket_row is the EXACT overlay row-build moved into a shared associated fn (trio unchanged: copy #70 / claim #75 write / comment #76 write + #77 flash; cx.stop_propagation preserved) → the overlay + section render identical rows; the Forge section reuses forge_status_line + sprint_rows (#64) + the #69 live forge_sprint; loading/empty via forge_empty_hint; the removed section_label was genuinely dead (both placeholders gone). No new write paths, no panics. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** forge_empty_hint_loading_vs_empty (both arms). `cargo nextest` → pass; gate cov/MSI 100.
- **Self-test:** the #90 static capture proved the tab strip; the Forge-section content reuses the proven ⌘⇧F overlay row-build (forge_ticket_row, output-identical) + forge_empty_hint (engine-tested); switching to the Forge tab needs a synthetic click (ENV-BLOCKED).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(5); forge #92 → done. **M2.F 3/6.** forge_empty_hint (cov/MSI 100) + shared forge_ticket_row (trio) + the Forge dock section. Removed the orphaned #90 section_label.
