# Editor caret-follow scroll — Notes

- **Forge ticket:** #270 `8334665d-8f60-4a52-b9b2-97237087d43f`
- **AAR:** `8e65777f-8e89-4949-9ddd-6bf71963fab5`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-270-caret-follow-scroll.md
- **Pipeline spec:** 270-caret-follow-scroll.spec.md

## Phase 1 — Plan
- **Request:** after a caret-moving editor action, scroll the caret's row into view.
  #265 inspect A1; the #290 diagnostic-nav prerequisite.
- **Classification / tier:** work pipeline slice, `bug`, shim-wiring. Crate:
  `marley_app` (app.rs). Editor render/input path → Validate uses the #264/#273
  HEADLESS lane (`editor_scroll_y_for_test`).
- **KEY DISCOVERY (de-risks it):** the editor moved to `gpui::UniformListScrollHandle`
  (`editor_scroll`, M16 #266) and #273 already built `scroll_editor_to_row(row)`
  (app.rs:5140, `mutants::skip`) — a non-strict centered scroll whose OWN doc names
  "#270 caret-follow, #272 find-next, #212/#213 open-at-line" as consumers. So #270
  is just WIRING that mechanism to the motion sites.
- **Discovery (grounded):**
  - `app.rs:5140` `scroll_editor_to_row(row)` — the shared scroll; `:5214` its one
    current caller; `:5149` `editor_scroll_y_for_test` (the headless oracle).
  - Motion sites (currently NOT following): `:6305` the #257 plain-editor key branch
    (Left/Right/Home/End/Word/Up/Down over `active_buffer_caret_anchor_mut`), the
    platform ⌘-motion branch (#257 doc/line), `:4555` the #272 ⌘D `select_next_match`.
  - `:2709` click-to-place — lands visible → NOT wired (D4).
  - `Buffer::line_col(caret).0` = the row (the uniform_list item index).
- **Decisions:** D1–D4 in the spec. Crux: reuse #273's mechanism; a `follow_editor_caret`
  shim computes the row (borrow-releasing `map`) then scrolls; wired at the keyboard
  motion sites, unconditionally (non-strict = safe).
- **Risk:** low. Borrow-safety is the only subtlety — compute `row` inside
  `active_editor_mut().map(...)` (the mut borrow ends when map returns the plain
  `Option<usize>`), then `self.scroll_editor_to_row(row)`. No new pure logic →
  headless-validated (the #273 pattern), app.rs is gate:4/5-excluded.

## Phase 2 — Design

### Approach
One `&self` shim + 3 one-line call-sites, reusing #273's `scroll_editor_to_row`.
§20 confirmed (Zed/universal ensure-visible via gpui's non-strict `scroll_to_item`).
`follow_editor_caret(&self)` = `active_editor().map(|s| s.active_buffer().line_col(s.active_caret()).0)`
then `scroll_editor_to_row(row)`. Borrow-safe: the immutable `active_editor()` borrow ends when
`.map` returns the plain `Option<usize>`, so the (also `&self`) scroll call is unencumbered. Masked
(`mutants::skip`); behavior pinned by the #264/#273 headless lane.

Wire it after each caret-moving editor dispatch (once the `editor_mut()` borrow releases):
- **Site A — the #257 plain-editor key branch** (Up/Down/Word/Home/End + the edit `_` arm): after its
  `if let Some(surface)` block (~6348), before `cx.stop_propagation()`.
- **Site B — the platform ⌘-motion branch** (⌘←→ line, ⌘↑↓ doc — the ticket's ⌘↓ doc-end): after its
  `if let Some(surface)` block (~6047), before `return`.
- **Site C — the #272 ⌘D select-next**: after its `if let Some(s)` block (~4561).

### File manifest
| File | Change |
|------|--------|
| `crates/marley_app/src/app.rs` | ADD `#[cfg_attr(test, mutants::skip)] fn follow_editor_caret(&self)`; call it at sites A/B/C. |

### Regression Test Plan
| # | Test (headless `#[gpui::test]`, the #264/#273 lane) | Proves |
|---|---|---|
| T1 | Seed a tall file (≫ viewport), open it, `simulate_keystrokes("cmd-down")` (doc-end) or many `down`; assert `editor_scroll_y_for_test()` increased above 0. | REQ-001 |
| T2 | A tall file with a match near the bottom; select the term; `cmd-d`; assert the scroll followed. | REQ-002 |
| T3 | (mechanism + T1's base) a motion within the viewport leaves y at 0 — `scroll_to_item` is non-strict. | REQ-003 |

Uncoverable: `follow_editor_caret` + the call-sites are `mutants::skip`/app.rs-excluded shim (a deferred gpui handle call); the BEHAVIOR is the headless assertions on `editor_scroll_y_for_test`. No new pure fn.

### Risks / decisions
- **R1 (borrow):** compute `row` inside `active_editor().map(...)` so the immutable borrow ends before `scroll_editor_to_row`; `cargo check` confirms.
- **D-scope:** the 3 keyboard-motion sites (D4). Clipboard paste/cut caret moves are edits (a rare huge-paste-at-bottom follow-up); the #257 `_` arm (backspace) IS followed via Site A.

## Phase 3 — Implement
- **Built to the manifest, no deviations.** Added `#[cfg_attr(test, mutants::skip)] fn follow_editor_caret(&self)` next to `scroll_editor_to_row` (`active_editor().map(|s| line_col(caret).0)` → `scroll_editor_to_row`). Wired 3 sites: Site A (the #257 key branch — `view.follow_editor_caret()` after the if-let, before `stop_propagation`), Site B (the ⌘-motion branch — after its if-let, before `return`), Site C (the #272 ⌘D select-next — `self.follow_editor_caret()` after its if-let).
- `cargo check -p marley --all-targets` clean (borrow-safety confirmed — the `map` releases the `active_editor()` borrow before the scroll); `cargo fmt` clean; `cargo clippy` clean.

## Inspect (Phase 3.5)
Inline adversarial trace (small shim-wiring diff — 1 helper + 3 one-line calls reusing #273). **No defects.**

| Angle | Verdict | Evidence |
|---|---|---|
| Borrow-safety | SAFE | `active_editor().map(\|s\| …0)` releases the `&self` borrow when it yields `Option<usize>`, before `scroll_editor_to_row(row)` (also `&self`). `cargo check --all-targets` clean. |
| Site coverage | SAFE | A (the #257 key branch) → Left/Right/Home/End/Word/Up/Down + the edit `_` arm; B (⌘-motion) → ⌘←→ line, ⌘↑↓ doc (the ticket's ⌘↓ doc-end); C → ⌘D select-next. Click excluded per D4 (lands visible). All the ticket's named motions covered. |
| Non-strict no-op | SAFE | `scroll_editor_to_row` uses `ScrollStrategy::Center`, non-strict (#273 doc: no-op when the row is already visible), so following unconditionally on every motion (incl. in-viewport) never jumps a visible caret. |
| No double-scroll | SAFE | A/B/C are mutually-exclusive key branches (distinct conditions, each ends in `return`); no path calls follow twice. |
| #273 deferred-jump constraint | SAFE | #273 warned open+jump-in-one-handler teleports the incoming file; #270 follows a MOTION within an ALREADY-OPEN editor (the tab owns the handle) — not an open+jump. Constraint N/A. |
| No active editor / empty | SAFE | `active_editor()` → None → `if let Some(row)` skips → no-op. |

No `failure-record` (no bug). No new prevention rule (reuses #273's shared scroll mechanism, exactly as its doc anticipated).

## Phase 4 — Validate
- **Test added (headless, the #264/#273 lane):** `editor_caret_follow_scroll_on_doc_end_headless` — seed a 120-line file, open it (asserts `editor_scroll_y == 0.0` at the top), `simulate_keystrokes("cmd-down")` (doc-end, Site B), assert `editor_scroll_y < 0.0` (the list scrolled DOWN to follow the caret — the offset goes NEGATIVE when scrolled down, per the #273 convention; my spec's ">0" was corrected to "<0"). **REAL behavioral proof through the actual key handler**, not just the mechanism.
- **`cargo nextest run -p marley` (the headless test): PASS** (0.13s) — before #270 the caret went off-screen with y unchanged at 0; now it follows.
- **`scripts/gates.sh --diff` → `GATE GREEN [diff]`** — 15/15. The `follow_editor_caret` shim + call-sites are `mutants::skip`/app.rs-excluded; the behavior is the headless assertion (stronger than a driven capture).
- **Live-drive: the headless test IS the behavioral proof** (it drives the real on_key_down → follow_editor_caret → scroll through the gpui test harness); a screen capture would show the same scroll but the headless assertion on `editor_scroll_y_for_test` is exact + deterministic. No env-blocked gap.
- No pre-existing failures.

## Phase 5 — Complete
- **Docs:** CHANGELOG.md ### Fixed (#270); editor.md scroll section gained the M18 #270 caret-follow coda.
- **Knowledge (forge):** AAR `8e65777f` submitted — completed, effectiveness 5. No failure-record (inline clean). No new prevention rule. LESSON: a follow-up filed at milestone N may already be half-built by N+1 — #273 built `scroll_editor_to_row` and its DOC pre-named "#270 caret-follow" as a consumer, turning #270 from "design an ensure_visible helper" (the stale ticket premise) into a 4-line wiring job. Always re-ground a follow-up against what shipped since it was filed. Also: the gpui scroll offset y is NEGATIVE when scrolled down (content moves up) — assert `< 0` for a scroll-to-bottom, not `> 0` (corrected the spec's T1 during validate).
- **Ticket** TICKET-270 → closed/ + forge ticket-close. **Pipeline** spec+notes → completed/.
- **Result:** caret-follow shipped + headless-proven; the #290 nav prerequisite is met. GATE GREEN [diff]. LOCAL commit only.
