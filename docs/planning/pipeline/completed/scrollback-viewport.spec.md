---
pipeline_id: 496388fd-4c5a-4a4d-9547-3b15c7b33a6a
ticket: forge#32 (ac48e97c-3937-4e91-9a8d-303013498b2b) · local docs/planning/tickets/open/TICKET-032-scrollback-viewport.md
aar_id: d27fac81-be82-4699-a3bf-750630018f42
status: Phase 5 — Complete PASS
title: scrollback viewport — scroll a pane's output; anchor-to-bottom
type: feature
milestone: M1.D
references:
  - crates/marley_app/src/workspace.rs (PaneState — gains the per-pane Viewport; likely a new viewport.rs)
  - crates/marley_app/src/app.rs (the shim: content/capacity + slice + scroll events)
  - docs/specs/SPEC-app-shell.spec.md (gains the viewport clause)
---

## Title
No scrollback — the pane renders all Block rows top-down under `overflow_hidden`, so once output
exceeds the pane height you can't see the top (the #23 inspect "prompt clips below the fold"). Add
a PURE per-pane `Viewport`: a following-vs-held scroll state + the visible-window math. Scroll up
(wheel / PageUp / shift-up) holds position; new output ANCHORS to the bottom unless the user has
scrolled up; the render slices the flattened rows to the visible window.

## Scope
### In
- NEW pure `Viewport` (a new `crates/marley_app/src/viewport.rs`):
  - `Viewport { top: usize, following: bool }` — `following` = pinned to the latest output;
    `top` = the first visible row when held. `new()` starts following.
  - `visible(&self, content: usize, capacity: usize) -> (usize, usize)` — the `[start, end)` row
    slice: WHEN following → the bottom `capacity` rows (`start = content − capacity`); WHEN held →
    `start = top` clamped to `max_scroll`; `end = (start + capacity).min(content)`. `capacity.max(1)`.
  - `scroll_up(&mut self, n, content, capacity)` — leave following (materialise `top` at the
    current bottom = `max_scroll` first), then `top = top.saturating_sub(n)`.
  - `scroll_down(&mut self, n, content, capacity)` — while held, `top += n`; at/over `max_scroll`
    re-anchor (`following = true`); a no-op while already following.
  - private `max_scroll(content, capacity) = content.saturating_sub(capacity)`.
- `crates/marley_app/src/workspace.rs` — `PaneState` gains `viewport: Viewport` (default `new()`);
  per-pane, travels with the pane.
- `crates/marley_app/src/app.rs` (shim) — flatten the pane's rows to a `content` count
  (Σ over blocks of `1` command line + `output_styled().len()`, plus the `1` prompt line); compute
  `capacity` from the pane rect height ÷ the gpui cell height (the #30 metric read); render ONLY
  the rows in `viewport.visible(content, capacity)`. Scroll: `on_scroll_wheel` → up/down by the
  wheel delta in rows; PageUp/PageDown → ±capacity; shift-up/down → ±1 (palette closed).
- SPEC-app-shell: the viewport clause + AC/Test-Plan/Mutation-Targets. CHANGELOG + arch doc.

### Out (explicitly deferred)
- A scrollbar affordance / thumb (a later visual polish); horizontal scroll (lines wrap, no
  h-scroll); scroll-to-a-specific-block / search-in-scrollback (M2 search); mouse-drag selection;
  a bounded scrollback cap (the Block list is already the store — capping is a later memory concern).
  Raw/alt-screen mode (seq-6) manages its own grid — the Viewport is the Block-view scroll.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — PURE `Viewport` (cov 100/MSI 100); the shim computes content/capacity + slices + wires
  events. Per-pane on `PaneState` (like `history` #29, `pty_size` #30).
- D2 — `following` (default) vs `top`-held is the anchor model: following auto-shows the bottom as
  content grows; held keeps `top` fixed so a running command's output doesn't yank the view — the
  ticket's "anchor-to-bottom UNLESS scrolled up".
- D3 — `scroll_up` MATERIALISES `top` at the current bottom (`max_scroll`) before moving, so the
  first up-scroll from the bottom lands one screen (n rows) above the latest, not at row 0.
- D4 — `scroll_down` re-anchors (`following = true`) at/over `max_scroll` — scrolling back to the
  bottom resumes following; scroll_down while following is a no-op.
- D5 — `capacity.max(1)` everywhere (a zero-height pane still shows ≥1 row; no div/underflow).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE following, `visible(content, capacity)` shall return the bottom `capacity` rows — `(content − capacity, content)` clamped ≥ 0 — so new content stays in view. | unit (content>capacity → the last capacity rows; content≤capacity → (0, content)) |
| REQ-002 | WHEN `scroll_up(n, …)` is called, the viewport shall leave following and show a window `n` rows above the current bottom (clamped at row 0); repeated scroll_up shall walk toward the top and clamp there. | unit (first up from bottom → top = max_scroll − n; walk; clamp at 0) |
| REQ-003 | WHEN `scroll_down(n, …)` returns the view to the bottom (top ≥ max_scroll), the viewport shall re-anchor (resume following); a partial scroll_down shall hold a higher `top`; scroll_down while following shall be a no-op. | unit (held → down past max → following again; partial hold; following+down → unchanged) |
| REQ-004 | WHILE held (not following), `visible` shall keep the same `top` window as `content` grows (until `top` exceeds the new `max_scroll`, then clamp). | unit (held at top=k; content grows → still (k, k+cap); content shrinks below k → clamp) |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `Viewport`. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the exact `Viewport` shape + `visible`/`scroll_up`/`scroll_down`/`max_scroll`
  signatures + the following/held transitions, the shim's content-count + capacity + slice + the
  scroll-event routing, the SPEC clause + mutation targets.
- **P3 Implement** — viewport.rs + PaneState field + app.rs slice/events + spec + CHANGELOG.
- **P3.5 Inspect** — critics: the clamp/anchor arithmetic, the materialise-on-first-up, the
  re-anchor threshold, the held-window-holds-as-content-grows, capacity edges.
- **P4 Validate** — the unit suite + gate GREEN [--diff].
- **P5 Complete** — docs, AAR, archive, close #32.
