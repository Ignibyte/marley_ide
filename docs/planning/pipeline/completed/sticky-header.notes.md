---
pipeline_id: d3b50a15-4bd3-4715-b458-cffd497b64bb
ticket: forge#185 (e0611622-933f-45bf-b53f-6d778b21f960)
aar_id: 13c90d9e-fced-4f82-9546-ca367094f0c1
---

# Notes — M12 #185 sticky command header

## Phase 1 — Plan / Phase 2 — Design (folded)
**Approach.** One pure fn on the #184 fold-aware rows + one shim overlay. `viewport.visible` already gives
`start` = the first visible content row (fold-aware). `sticky_block` indexes `fold_visible_rows` at `start`:
- `RowKind::Output(block, _)` at start → the header is above → `Some(block)` (sticky it).
- `RowKind::Header(_)` at start → the real header is already the top row → `None` (no sticky, no double).
- `None` (start past the block rows — the prompt / empty) → `None`.
This is the whole boundary logic; it's exactly "header-at-top → None, output-at-top → Some" the ticket names.

**File manifest.**
- `nav.rs` (PURE) — `sticky_block(counts, folds, viewport_top)` (+ tests in P4).
- `app.rs` (SHIM) — after `(start,end)`, `let sticky = sticky_block(&block_line_counts(state), &state.folds, start);`
  when Some(b), read block b's command + `exit_status_kind`/`status_indicator` glyph, render an `.absolute()`
  overlay row pinned at the pane content's TOP (`top(r.y + PANE_TITLE_H)`), `.occlude()` + surface bg so it sits
  over the scrolled output. Drawn AFTER the pane body so it's on top.

**Regression Test Plan.**
| Test | Proves |
|---|---|
| `sticky_block_pins_when_header_scrolled_above` | REQ-001/002 — output-row top → Some(block); header-row top → None (the boundary); prompt/past-end → None; empty → None |
| `sticky_block_respects_folds` | REQ-003 — a folded block above shifts which block a given viewport_top maps to |
| driven | REQ-004 — scroll a tall block → its header rides the top; scroll to another → switches |
| gate --diff (staged) | REQ-005 |

**Risk.** The boundary is the crux: viewport_top EXACTLY on a header → None (else a double header). fold_visible_rows
already encodes header-vs-output, so matching RowKind is the boundary-safe form (no arithmetic off-by-one). The
overlay must occlude (surface bg + .occlude()) so the scrolled output doesn't bleed through; draw it last. No
sticky at the prompt (start past the last block row → None).

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` clean (only the pre-existing `block v0.1.6` warning).
- `nav.rs` (PURE) — `sticky_block(counts, folds, viewport_top)`: `match fold_visible_rows(...).get(viewport_top)`
  → `Some(RowKind::Output(block, _)) => Some(*block)`, `_ => None`. The pattern match makes the header-vs-output
  boundary off-by-one-proof (a Header-at-top or a past-end None both fall to `_ => None`).
- `app.rs` (SHIM) — imported `sticky_block`; inside the cooked-block view after the prompt, when
  `sticky_block(&block_line_counts(state), &state.folds, start)` is Some, look up that block's command +
  `exit_status_kind`/`status_indicator` glyph and render an `.absolute().top(0).left(0).w_full().occlude()`
  surface-bg overlay row (glyph + command, a bottom border) pinned at the pane content's top edge. Drawn LAST
  in the cooked view so it paints over the scrolled output.

**Deviation:** none. The sticky is read-only (no chevron/actions — the real header keeps those, per scope). It
reuses the exact glyph + type-scale the real header uses, so it reads identically. `nth(sticky)` guards a
missing block (can't happen — sticky_block only returns an in-range index — but no unwrap on a response path).
## Inspect (Phase 3.5)
2 parallel general-purpose critics (correctness/mutation + render/gpui-layout). The first correctness critic
returned malformed output (0 tool-uses — a spurious failure); re-spawned it and got a full review. Both verified
concretely (traced fold_visible_rows values, ran cargo mutants --list, read gpui's own div.rs docs). Findings:

- **[HIGH] `.occlude()` creates a scroll dead-zone over the sticky band.** gpui's `.occlude()` = `BlockMouse`,
  which blocks SCROLL (not just clicks) on elements behind it — so hovering the full-width sticky band at the
  pane top would SWALLOW the scroll wheel, defeating a scroll-helper feature. gpui ships the purpose-built
  `.block_mouse_except_scroll()` (its own docs say it "should be preferred" for a non-modal overlay over a
  scroll area). REAL (render critic, verified against gpui-0.2.2 div.rs:588/1013). FIXED: swapped to
  `.block_mouse_except_scroll()` — still blocks click/selection fall-through (the read-only intent), lets scroll
  through. → prevention rule (occlude vs block_mouse_except_scroll for non-modal scroll overlays).
- **[MED] 4 mutants on sticky_block, all survive until Phase 4** (incl. a delete-match-arm mutant → constant
  None, which cargo-mutants DOES emit — correcting my prior assumption). Both critics gave the exact 2 kill-
  fixtures: one OUTPUT-row → Some(block) (kills the →None + delete-arm), one HEADER-row → None (kills the
  →Some(k)). DEFERRED to validate (correct); recorded verbatim + the full [2,0,3] boundary trace.
- **[LOW] `.iter().nth(sticky)` → `.get(sticky)` nit** — I tried it; `blocks().get` takes a `BlockIndex`
  NEWTYPE, not usize (that's WHY the original used `.iter().nth()`). REVERTED to `.iter().nth()` + a comment
  explaining the newtype. (Good catch that the "cleaner" idiom doesn't compile here — the newtype is deliberate.)
- Boundary correctness — VERIFIED CLEAN by both, traced exhaustively over [2,0,3]: header-row→None (no double
  header), output-row→Some(block), prompt/past-end→None (elegantly, the prompt row index == fold_visible_rows.
  len() so "at prompt" and "out of bounds" are the SAME None boundary), empty→None. The all-fits/following case
  yields no spurious sticky (start=0 → Header(0) → None).
- Double-header exclusion — VERIFIED CLEAN: the render paints only rows in [start,end), so a block's real header
  at a row < start is NOT painted; the sticky + real header are mutually exclusive, same `start`, one pass.
- Positioning (absolute top:0 pins to the pane top despite justify_end — proven by the sibling agent-badge),
  opaque surface bg (alpha 1.0 both themes), alt-screen exclusion, panics (nth always Some — sticky_block only
  returns in-range indices; the if-let is defensive) — all VERIFIED CLEAN.
- [LOW] the sticky overlays the topmost output row (by-design, like VS Code sticky-scroll); the agent badge
  paints over its top-right corner (cosmetic) — SIGN-OFF.

Lenses: boundary correctness, mutation-killability (+ delete-arm), fold-consistency, gpui occlusion/scroll
semantics, absolute positioning vs flex, paint-order/double-render, panics, alloc, fmt/secrets. Post-fix:
`cargo check --workspace` clean, scroll fix in.
## Phase 4 — Validate
**Tests** (nav.rs, the critics' exact kill-fixtures): sticky_block_pins_when_header_scrolled_above (the full
[2,0,3] boundary trace — header→None, output→Some(block), prompt/past-end→None, empty→None); sticky_block_
respects_folds (a folded block above shifts the row→block map). `cargo mutants` on the diff → **4 mutants
(incl. the delete-arm), 4 caught, 0 missed** (MSI 100). **cargo nextest run --workspace**: 760 passed.

**Driven live-app capture (REQ-004)** — minted a small `echo topmark185` block then a TALL `seq 1 80` block:
- Following the bottom (output 40–80 shown), **`✓ seq 1 80` pinned at the pane top** though the real header is
  ~40 rows off-screen (/tmp/mly185_tall2.png).
- Scrolled UP into the middle (output 32–73) → **`✓ seq 1 80` STAYS pinned at the top** (/tmp/mly185_mid.png).
  The scroll itself worked (32–73 vs 40–80) — proving the inspect `.block_mouse_except_scroll()` fix (scroll
  passes THROUGH the sticky band; `.occlude()` would've dead-zoned it).
- Scrolled to the top → the sticky reflects whichever block owns the top row, and the REAL headers (`▾ ✓ echo
  topmark185`, `▾ ✓ seq 1 80`, with #184 chevrons) appear separately in the body — no double-header
  (/tmp/mly185_top.png).

**Gate**: `scripts/gates.sh --diff` (staged) → **GATE GREEN [diff]**, 15/15. coverage 100%; mutation 4/4 → MSI
100.0%.

**Pre-existing exclusions**: none. (Restored ~/.marley/config/settings.toml from the pre-test backup.)
## Phase 5 — Complete
(pending)
