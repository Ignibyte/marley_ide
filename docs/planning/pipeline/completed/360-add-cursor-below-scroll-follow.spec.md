---
pipeline_id: 49e897c8-e835-4d30-b976-f0b7cac73357
ticket: forge#360 (c243e4d4-59d2-4d19-8623-faa8c6103a37) · local docs/planning/tickets/open/TICKET-360-add-cursor-below-scroll-follow.md
aar_id: 98f090dd-d3a3-4e3f-b913-fc9095f0372c
status: Phase 5 — Complete PASS
title: add-cursor-below (⌘⌥↓) scroll-follow tracks the new bottom cursor — route the add-cursor arm through #342's added_member + scroll_editor_to
type: bug
milestone: M22
references: [app.rs:7358 the add-cursor arm (the fix site — `follow_editor_caret()` → the added_member+scroll_editor_to pattern) / :7388-7408 the add-next-occurrence arm (the EXACT template, #342) / :12885 scroll_editor_to(row, Option<CharOffset>) (column-aware) / :12901 scroll_editor_to_row (row-only, NOT this) / fn dispatch_action `#[cfg_attr(test, mutants::skip)]`, multi_cursor.rs:296 added_member(&SelectionSet,&SelectionSet)->Option<Selection> (+ unit :1076) / :67 add_cursor_vertical, buffer.rs:152 line_col(CharOffset)->(row,col), headless_drive.rs:763 editor_scroll_y (the vertical accessor) / :5760 cmd_d_horizontal_follow (#342 mirror) / :2237 t297 ⌘⌥↓ set test / :774 per_file_scroll_memory (editor_scroll_y moves after run_until_parked), #342 #336 #331]
---

## Title
The add-cursor arm follows the PRIMARY (`follow_editor_caret()`), so ⌘⌥↓'s new bottom cursor marches off the
bottom edge unseen. Fix: gate the follow on `added_member(&before, &after)` and route through the column-aware
`scroll_editor_to(row, Some(head))` — the exact seam #342 built for ⌘D — so BOTH ⌘⌥↑/⌘⌥↓ follow the actually-new
cursor regardless of how it sorts.

## Scope
### In
- Rewrite the `"add-cursor-above" | "add-cursor-below"` arm (app.rs:7358) to mirror the `add-next-occurrence`
  arm (:7388-7408): capture `before = s.active_selections().clone()` BEFORE the mutation; after
  `add_cursor_vertical(&before, buffer, dir)`, compute `follow = added_member(&before, &set).map(|sel| { let head
  = sel.head(); (s.active_buffer().line_col(head).0, head) })`; `set_active_selections(set)`; then `if let
  Some((row, head)) = follow { self.scroll_editor_to(row, Some(head)); }` — REPLACING the bare
  `self.follow_editor_caret()`. Gating on `added_member` covers BOTH directions (removes ⌘⌥↑'s luck-dependency).
- A headless drive proving the VERTICAL follow via `editor_scroll_y` (sustained ⌘⌥↓ scrolls the list down to the
  new bottom cursor; the OLD primary-follow would leave it at 0) + ⌘⌥↑ + a single-press-no-scroll guard.

### Out (explicitly deferred)
- Any change to `added_member`, `add_cursor_vertical`, `scroll_editor_to`, or `follow_editor_caret` — all UNCHANGED
  (correct + tested by #342/#297); the fix only rewires WHICH cursor the arm follows.
- A new pure seam — none needed (the fix is app-shim wiring reusing shipped, already-covered primitives).
- The horizontal-only proxy proof (#342's `scroll_x` trick) — ⌘⌥↓'s new cursor is at the SAME column as the
  primary (unless clamped), so the vertical `editor_scroll_y` axis is the honest distinguishing proof here.

## Reference (§20)
N/A — Marley's own multi-cursor + editor scroll-follow. No Warp/Zed source read.

### Prior art
- **Our own code (the fix IS a prior-art transplant):** the `add-next-occurrence` arm (app.rs:7388-7408, #342)
  already solves the identical "follow the ADDED cursor, not the sorted-member-0 primary" problem with
  `added_member` + `scroll_editor_to`. #360 mirrors it verbatim into the sibling add-cursor arm. `added_member`
  (multi_cursor.rs:296) + its unit (`added_member_names_the_new_cursor_not_the_primary` :1076) already prove the
  name-the-new-one contract; `scroll_editor_to` (app.rs:12885, column-aware) + `line_col` (buffer.rs:152) already
  exist and are covered. The `PR-claude-hook-the-shared-primitive` rule (#336) applies: hook the gesture's own
  follow, not a shared caret reader.
- **Our permissive deps:** gpui owns the vertical scroll (`UniformListScrollHandle::scroll_to_item`, which
  `scroll_editor_to` already calls) — no new adoption; `editor_scroll_y_for_test` reads its settled offset. Checked;
  no crate owns the "which cursor to follow" decision — that is ours.
- **Behavior maps / published:** N/A — no reference-app behavior.

## Locked-In Decisions
- **D1 — the fix is a verbatim mirror of the `add-next-occurrence` arm** (capture `before`, `added_member` gate,
  `scroll_editor_to(line_col(head).0, Some(head))`). The row-from-head derivation is `s.active_buffer().line_col(head).0`
  (confirmed in the template). Replaces `self.follow_editor_caret()`.
- **D2 — gate BOTH directions through `added_member` (not just ⌘⌥↓).** `added_member(&before, &after)` names the
  new cursor regardless of sort, so it is correct for ⌘⌥↑ too — routing both removes the "correct by luck"
  member-0 dependency the ticket flags. ⌘⌥↑ behavior is unchanged (it still scrolls to the new top cursor).
- **D3 — no new cov/MSI surface (gate-is-headless-drive).** The arm is inside `dispatch_action`
  (`#[cfg_attr(test, mutants::skip)]`) and app.rs is coverage-excluded; `added_member`/`scroll_editor_to`/`line_col`
  are already covered by #342/#297. The headless drive carries the wiring proof.
- **D4 — the vertical scroll IS headlessly observable via `editor_scroll_y`** (proven: `per_file_scroll_memory…`
  reads a moved offset after `scroll_editor_to_row` + `run_until_parked`). It settles on `run_until_parked` (the
  gpui layout), NOT the app pump — so NO mock-clock `tick_pump` is needed for this drive.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-DOWN-FOLLOWS-NEW | WHEN sustained ⌘⌥↓ adds cursors below past the bottom viewport edge, the system shall scroll the editor list so the newest (bottom) cursor stays visible. | headless: a tall fixture, caret near the top, N× `add-cursor-below` → `editor_scroll_y` moved DOWN (non-zero), i.e. the list followed the new bottom cursor (the old primary-follow left it at 0). |
| REQ-UP-FOLLOWS-NEW | WHEN sustained ⌘⌥↑ adds cursors above past the top edge, the system shall follow the newest (top) cursor. | headless: caret deep, N× `add-cursor-above` → `editor_scroll_y` moves toward the top; ⌘⌥↑ still correct via the same `added_member` path. |
| REQ-SINGLE-PRESS-NOOP | WHEN a single add places the new cursor already on screen, the system shall NOT move the view. | headless: one `add-cursor-below` near the top → `editor_scroll_y` unchanged (the non-strict Center no-op). |

## Phase Plan
- **P2 Design** — ratify D1/D2 (the exact arm rewrite + both-dirs-through-added_member) + the borrow shape (capture
  `before`, `follow` local, apply after the `if let Some(s)` block) + the headless test plan (tall fixture, sustained
  ⌘⌥↓/⌘⌥↑ via `dispatch_for_test` loop, `editor_scroll_y` assertions; the single-press no-op).
- **P3 Implement** — the arm rewrite.
- **P3.5 Inspect** — ★ `added_member` gates correctly for both dirs; the row-from-head is `line_col(head).0`;
  `scroll_editor_to` (column-aware) not `scroll_editor_to_row`; ⌘⌥↑ not regressed; the borrow compiles; no new pure
  seam missed.
- **P4 Validate** — the headless drives (`editor_scroll_y`) + the `--diff` gate. No mock-clock pump (D4). No live drive.
- **P5 Complete** — CHANGELOG + editor.md (the both-dirs-through-added_member note, sibling of #342).
