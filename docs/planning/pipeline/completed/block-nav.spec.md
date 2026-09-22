---
pipeline_id: 8aff44d1-0e4b-4e0f-a775-5cd8479a4d8e
ticket: forge#48 (be79e95e-979b-4850-8a9c-2a73ae89211d) · local docs/planning/tickets/open/TICKET-048-block-nav.md
aar_id: 054d392c-b1b1-48a4-b0ca-55b2f566e103
status: Phase 5 — Complete PASS
title: clear (cmd-K) + jump-to-block nav (⌘↑/⌘↓)
type: feature
milestone: M1.G
references:
  - crates/marley_app/src/nav.rs (NEW — block_boundary_rows + jump_target)
  - crates/marley_app/src/keymap.rs (cmd-up/down/k bindings)
  - crates/marley_app/src/app.rs (the jump + clear dispatch — shim)
  - docs/specs/SPEC-app-shell.spec.md (R52)
---

## Title
Everyday terminal nav the Block model makes clean (the M1.G FINALE): cmd-K clears the screen, and
⌘↑/⌘↓ jump the viewport between BLOCK boundaries (to the previous/next command, not line-by-line).

## Scope
### In
- `crates/marley_app/src/nav.rs` (NEW, gpui-free PURE — cov/MSI 100):
  - `block_boundary_rows(output_line_counts: &[usize]) -> Vec<usize>` — the content-row index where
    each block's header starts. Accumulate: `row = 0`; per block push `row`, then `row += 1 (header)
    + line_counts[i]`. So `[2, 0, 3]` → `[0, 3, 4]`.
  - `jump_target(boundaries: &[usize], current_top: usize, forward: bool) -> Option<usize>` — forward
    → the FIRST boundary `> current_top`; backward → the LAST boundary `< current_top`; `None` at the
    ends (NO wrap).
- `crates/marley_app/src/lib.rs` — `mod nav;`.
- `crates/marley_app/src/keymap.rs` — bindings `cmd-up → "jump-block-prev"`, `cmd-down →
  "jump-block-next"`, `cmd-k → "clear-screen"` (+ their `action_for` test assertions).
- `crates/marley_app/src/app.rs` (SHIM): `dispatch_action` arms — jump builds `boundaries` from the
  focused pane's per-block output-line-counts, then `jump_target(boundaries, viewport_top, forward)`
  → reuse #47's `scroll_focused_to_row(target)`; clear resets the viewport + sends the terminal clear
  (`write_bytes(b"\x0c")`, the Ctrl-L form-feed the shell redraws on) to the focused shell.
- SPEC-app-shell (R52, the clear + block-nav clause + Mutation-Targets). CHANGELOG + arch doc.

### Out (explicitly deferred)
- Smooth-scroll animation for the jump (it snaps). Jump-to-first/last block (⌘Home/⌘End). A clear that
  DROPS the block history (the first cut sends the shell's clear — the blocks stay in scrollback,
  matching a real terminal's Ctrl-L). Per-pane clear vs whole-window.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `block_boundary_rows` takes the per-block OUTPUT-LINE-COUNTS (`&[usize]`), not `&BlockList` —
  keeping the pure fn free of the Block type; the shim maps `blocks` → line-counts. The boundary is
  `header + output_lines` accumulated (matching the render's content-row walk).
- D2 — `jump_target` does NOT wrap (None at the ends) — ⌘↑ at the top / ⌘↓ at the bottom is a no-op,
  the expected block-nav feel (distinct from the find cursor's wrap in #47).
- D3 — cmd-K sends the shell's clear (`\x0c`) — a real Ctrl-L, so the shell redraws its prompt; the
  block history is untouched (first cut). PURE surface = the NAV (`block_boundary_rows` +
  `jump_target`); the clear + the jump-scroll are SHIM (app.rs, masked), reusing #32 + #47.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `block_boundary_rows(counts)` is called, it shall return each block's header content-row — `row` accumulating `1 + count` per block; `[2, 0, 3]` → `[0, 3, 4]`. | unit (multi-block; empty) |
| REQ-002 | WHEN `jump_target(boundaries, top, true)` is called, it shall return the first boundary `> top` (or `None` past the last); WHEN `forward = false`, the last boundary `< top` (or `None` before the first). | unit (fwd/back mid + both ends) |
| REQ-003 | WHEN the viewport is at the last (forward) or first (backward) boundary, `jump_target` shall return `None` (no wrap). | unit (both ends → None) |
| REQ-004 | WHEN ⌘↑/⌘↓ is pressed, the app shall scroll the viewport to the previous/next block boundary; WHEN cmd-K is pressed, it shall clear the focused shell. | shim + masked visual — chad-verified |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `block_boundary_rows` + `jump_target`. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the `nav.rs` shapes, the keymap bindings + dispatch arms + the clear, the SPEC clause
  + mutation targets.
- **P3 Implement** — nav.rs + mod + the keymap bindings + the app.rs jump/clear dispatch + spec + CHANGELOG.
- **P3.5 Inspect** — critics: the boundary accumulation (off-by-one header, push order), the jump
  direction (fwd `>` vs back `<`, the `.rev()`), the no-wrap ends, the shim's boundary-build alignment
  with the render rows, the clear byte, no keymap/viewport regression.
- **P4 Validate** — the block_boundary_rows/jump_target unit tests (+ the keymap binding assertions) +
  gate GREEN + the masked visual.
- **P5 Complete** — docs, AAR, archive, close #48 + **CLOSE M1.G sprint #7**.
