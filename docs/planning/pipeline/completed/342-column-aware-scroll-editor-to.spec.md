---
pipeline_id: 1db6ab7f-a75e-4c6e-9746-69ca8d614090
ticket: forge#342 (f6df65f0-8c45-4c4f-b973-e5a764c4bddd) · local docs/planning/tickets/open/TICKET-342-column-aware-scroll-editor-to.md
aar_id: 3af09837-6f88-4637-ae52-b0b56f3fe61f
status: Phase 5 — Complete PASS
title: A column-aware scroll_editor_to(row, target) — ⌘D's follow tracks the added cursor, not the primary
type: chore
milestone: M22
references: [app.rs:12778 scroll_editor_to_row (the #336 primitive), app.rs:12843 follow_editor_caret_x (the horizontal follow shim), app.rs:7277 dispatch_action "add-next-occurrence" (the ⌘D site), editor_surface.rs:233 active_caret (= primary().head()), multi_cursor.rs:296 added_member, h_scroll::follow_caret_x + caret_px (#336 pure, cov/MSI 100), code_view::col_of_offset (the display-col map), AD-claude-two-boundary-maps-for-phantom-text-001, PR-claude-hook-the-shared-primitive-not-one-of-its-callers-001, PR 71b36786 (a width in chars is not a width in cells)]
---

## Title
A column-aware `scroll_editor_to(row, target)` so ⌘D's horizontal caret-follow tracks the **added** cursor's
display column, not the primary's. A #336 wiring follow-up — the imperfection #336 recorded at its own site.

## Scope
### In
- Parametrize the horizontal follow on a target caret: `follow_editor_caret_x_to(target: CharOffset)` (the
  current `follow_editor_caret_x` body, reading `target` instead of `active_caret()`), and
  `follow_editor_caret_x()` delegates with `active_caret()`.
- `scroll_editor_to(row, target: Option<CharOffset>)` — scroll the vertical slot to `row`, then run the
  horizontal follow toward `target` (`None` → the primary). `scroll_editor_to_row(row)` becomes
  `scroll_editor_to(row, None)` — byte-identical for its existing callers.
- The ⌘D add-next-occurrence site (`dispatch_action`) passes the ADDED member's `head()`:
  `scroll_editor_to(added_row, Some(added_head))` — both axes track the same (added) cursor.

### Out
- go-to-def (`consume_pending_center`), find (`select_efind_current`), goto-line, symbol-jump — they land a
  SINGLE caret at the target BEFORE scrolling, so their primary IS the target and the follow is already
  correct. NO change (the ticket's "check + pass explicitly" resolves to "already right — leave them").
- `goto_preview` (caret stays at the origin) and `follow_editor_caret` (the caret IS the primary) — unchanged
  via the `None` delegation.
- Drag, ⌘⇧L select-all-occurrences (deliberately does not scroll), and any new h-scroll math.

## Reference (§20)
N/A — Marley-specific multi-cursor scroll behavior. There is no Warp (a terminal) or Zed (GPL, off-limits)
BEHAVIOR to match; keeping the viewport on the cursor you just created is a general editor expectation, and
the mechanism is entirely Marley's own #336 h-scroll.

### Prior art
1. **OUR OWN CODE (the whole seam):** `h_scroll::follow_caret_x` + `caret_px` (#336, pure, cov/MSI 100) do the
   horizontal follow math; `code_view::line_layout(...).col_of_offset` (tested) is the char-offset→display-col
   map (`AD-claude-two-boundary-maps-for-phantom-text-001` owns that domain); `marley_editor::added_member`
   (multi_cursor.rs:296, tested at :1076) names the newly added cursor. The change threads an existing offset
   through these — it invents no math. The #336 headless lane (`scroll_x_for_test`, `set_h_scroll_for_test`,
   `follow_caret_x_for_test`) is the test harness.
2. **gpui (Apache-2.0, adoption):** checked — gpui's `requested_autoscroll` is element-BOUNDS-based (it reveals
   a painted element's rect). It does not fit Marley's editor, which is a `uniform_list` (row-index vertical) +
   a hand-rolled horizontal margin (#336); there is no gpui seam for "scroll the custom h-axis to a specific
   cursor's column". No owner.
3. Checked ropey/regex/tree-sitter — not their seam (this is selection + viewport, not text or parse).

## Locked-In Decisions
- **D1-TARGET-IS-AN-OFFSET, NOT A RAW COL** — the seam takes `Option<CharOffset>`, not the ticket's literal
  `col: usize`. The horizontal follow needs a DISPLAY column (tab-/wide-char-aware), which only
  `col_of_offset` can produce; a raw `col` would force the caller to duplicate `line_layout` or pass a
  char-col in the wrong domain (`PR 71b36786` — a width in chars is not a width in cells; the #336/#331 trap).
  Passing the offset lets `follow_editor_caret_x_to` derive the display col via the SAME shipped path the
  primary uses. (Design confirms the exact type; `CharOffset` is what `added_member().head()` already yields.)
- **D2-DELEGATE-WITH-NONE** — `scroll_editor_to_row(row) = scroll_editor_to(row, None)`, where `None` runs the
  existing `follow_editor_caret_x()` (the primary). Every current caller (goto_preview, reveal_and_scroll_to_row,
  follow_editor_caret, the #336 lane) is byte-identical; only ⌘D passes `Some`.
- **D3-⌘D-ONLY** — #342 fixes ⌘D (the gesture the ticket names): every `scroll_editor_to_row` path other than
  ⌘D either keeps the caret at the primary or collapses to a single caret at the target first. **CORRECTION
  (Phase 3.5 inspect):** the recon's "the mismatch is UNIQUE to ⌘D" was an over-claim — `add-cursor-below`
  (⌘⌥↓) has the same class of gap (it scrolls via `follow_editor_caret` = the primary, but its new caret sorts
  to the bottom). That is PRE-EXISTING + out of this ⌘D-scoped ticket → filed as follow-up **#360**, which
  reuses the very `scroll_editor_to(row, Some(target))` seam this ticket adds. #342 stays ⌘D-scoped.
- **D4-NO-NEW-PURE-SEAM** — this is integration/wiring over shipped, tested primitives. The new fns are app.rs
  shims (`#[cfg_attr(test, mutants::skip)]`, coverage-excluded). There is NO new pure home for a cov/MSI 100
  obligation and one must NOT be invented; the proof is a HEADLESS behavioral test (REQ-ADDED-BOTH-AXES) plus
  the existing #336 suite staying green. The `--diff` mutation gate will see ~0 new mutants (all shims skip'd)
  — that is expected and correct for a wiring change, not a coverage gap to paper over.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-ADDED-BOTH-AXES | WHEN ⌘D adds an occurrence whose display column lies off-screen to the right, the system shall move `scroll_x` to follow the ADDED cursor's display column (not the primary's). | headless: seed two occurrences on rows far apart with the second's column past the viewport; drive add-next-occurrence; assert `scroll_x` follows the added member's col. A control that followed the primary (col 0-ish, on screen) would leave `scroll_x` at 0 → the test distinguishes. |
| REQ-DELEGATION-UNCHANGED | The system shall keep `scroll_editor_to_row(row)` behaviorally identical (it delegates as `scroll_editor_to(row, None)`, following the primary). | diff review + the #336 headless lane + goto/find/goto-line paths stay green (unedited). |
| REQ-IDENTITY-NOOP | WHEN the target column is already within the follow band, the system shall not move `scroll_x`. | headless: a target already on screen → `follow_caret_x_to` returns `false`/`scroll_x` unchanged (reuses `follow_caret_x`'s tested identity case). |

## Phase Plan
- **P2 Design** — settle the seam signature (`Option<CharOffset>`; confirm the `None`/`Some` split preserves
  the current callers byte-for-byte); the exact 2-3 shim edits; the headless test plan (how to seed the two
  occurrences + drive ⌘D + read `scroll_x` via the #336 hooks, incl. whether a new tiny test hook is needed to
  reach the added-cursor path); confirm the shims stay `mutants::skip` and app.rs stays coverage-excluded.
- **P3 Implement** — `follow_editor_caret_x_to` + `scroll_editor_to` + the ⌘D one-liner (`.0`→pass the head).
  `cargo check`.
- **P3.5 Inspect** — the delegation-preserves-behavior claim (is `scroll_editor_to_row` truly byte-identical?);
  the char-offset→display-col domain (no raw col leaks in); is it REALLY ⌘D-only (re-audit the callers).
- **P4 Validate** — the headless REQ-ADDED-BOTH-AXES test + REQ-IDENTITY-NOOP; the #336 suite green; the
  `--diff` gate → `GATE GREEN [diff]` (gate:5 ~0 diff mutants [shims skip'd — stated]; gate:4 app.rs excluded;
  the headless test is the behavioral guard). **Live pixel deferred (chad at the machine) — the AC is a
  headless scroll-target assert; stated, not masked.**
- **P5 Complete** — CHANGELOG + editor.md (the #336 follow note "a column-aware scroll_editor_to(row,col) is
  the follow-up" → SHIPPED); AAR; close + archive.
