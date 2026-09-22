# caret floats a space after the typed text — Notes

- **Forge ticket:** #88 `a406c036-43ea-4adb-b22b-93327514aba7` · **AAR:** `d67be772-0e94-4563-af71-2b9dec26b719`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-088-caret-space.md

## Phase 1 — Plan
- **Request:** forge #88 (Terminal Polish 1/15) — chad live-found "a weird space after the last character".
- **Classification:** work pipeline, `bug`, SHIM-only render fix (app.rs). No pure surface.
- **Root cause:** app.rs ~1884 input_row `.gap_2()` gaps every child incl. text↔caret.
- **Decisions:** D1 gapless inner flex for before+caret+after; D2 no new pure surface (masked render).
- **AAR id:** `d67be772-0e94-4563-af71-2b9dec26b719`.

## Phase 2 — Design
- **The restructure** (app.rs input_row render): the current tail
  ```rust
  input_row = input_row
      .child(div().child(before))
      .child(div().w(px(2.0)).h(px(TERMINAL_FONT_SIZE)).bg(colors.accent))
      .child(div().child(after));
  ```
  becomes ONE gapless inner flex child:
  ```rust
  input_row = input_row.child(
      div().flex().flex_row().items_center()
          .child(div().child(before))
          .child(div().w(px(2.0)).h(px(TERMINAL_FONT_SIZE)).bg(colors.accent))
          .child(div().child(after)),
  );
  ```
  The inner flex has NO `.gap_2()` → before/caret/after are flush. It's a single child of the outer
  `.gap_2()` row, so the ❯/cwd/git segments keep their spacing.
- **File manifest:** MODIFY `crates/marley_app/src/app.rs` (the input_row render tail only).
- **Regression Test Plan:** NO unit tests — the render is masked (cov-excluded + mutants::skip), no pure
  surface. REQ-001/002 = the self-test/capture; REQ-003 = the gate (masked change → green).
  Uncoverable: the render layout — gpui, masked; proven by the capture.
- **Risks:** D-2.1 the inner flex must be `flex_row().items_center()` (match the outer's vertical centering
  so the caret bar aligns with the text baseline). D-2.2 `before`/`after` stay separate `div`s around the
  caret (the caret split by CharOffset is unchanged — this is purely wrapping them in a gapless parent).

## Phase 3 — Implement
- **Built:** wrapped `before` + the caret bar + `after` in a gapless `div().flex().flex_row().items_center()`
  (one child of the outer `.gap_2()` row) in the app.rs input_row render. No pure-logic change.
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK.

## Phase 3.5 — Inspect
- **Method:** self-review (scaled to a 1-block masked render change with zero logic — no subagent).
- **Lenses covered:** correctness (the gapless inner flex makes before/caret/after flush; `items_center()`
  matches the outer row so the caret aligns; ❯/cwd/git stay direct children of the `.gap_2()` row →
  spacing unchanged; the `split_at_caret` before/after split is untouched — just wrapped; empty buffer →
  caret at the line start), simplification (minimal wrap, no dup), security/data (pure layout, no logic/
  input/secrets). **No findings.**
- **Fix applied:** none (clean).

## Phase 4 — Validate
- **Tests:** none added (masked render, no pure surface). `cargo nextest -p marley` → no regression.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15** (the render change is cov-excluded + mutants::skip).
- **Live capture (`caret88.png`):** the #88 build renders the prompt cleanly — `❯ chadpeppers |`, the caret
  one space after the cwd segment (empty-buffer case; the old double-gap is gone). The "caret FLUSH against
  typed text" is the structural consequence of removing the gap between `before` and the caret — demoing it
  WHILE typing is ENV-BLOCKED (synthetic input degraded this session, per #86/#87), but the diff + the
  clean empty-prompt render confirm the fix.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- CHANGELOG ### Fixed; aar-submit(5); forge #88 → done. **Terminal Polish 1/4.** The caret gap_2 render fix. Self-test typing env-blocked; verified via the diff + a clean empty-prompt capture + GATE GREEN.
