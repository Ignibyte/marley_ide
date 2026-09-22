# Size the PTY to the carved content rect — Notes

- **Forge ticket:** #287 `defe74a1-3684-4e49-83ca-7df32bc5d332`
- **AAR:** `4497d877-6c70-4a36-b822-ce994a5d5c3c`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-287-pty-size-title-bar.md
- **Pipeline spec:** 287-pty-size-title-bar.spec.md

## Phase 1 — Plan
- **Request:** size a terminal pane's PTY to its CONTENT rect (pane height minus
  the 24px title bar), not the full pane. Bug from #280 inspect F3, pre-existing
  since M5 #108.
- **Classification / tier:** work pipeline, one slice, `bug`. Crates: `marley_app`
  (`workspace.rs` — a pure `inset_top`; `app.rs` — one resize-loop call). Touches
  the terminal render/mouse behavior → Validate drives htop if reachable.
- **Root cause (grounded this phase):**
  - `app.rs:7258` — the resize loop: `plan_resize(*rect, cell, term.pty_size)`
    feeds the FULL pane rect.
  - `app.rs:7285-7287` — the render carves the content div:
    `.top(px(r.y + PANE_TITLE_H)).h(px((r.h - PANE_TITLE_H).max(0.0)))`
    (`PANE_TITLE_H = 24.0`, app.rs:463).
  - `workspace.rs:120` — `plan_resize(rect, cell, current)` → rows =
    `grid_axis(rect.h, cell.h)` = `floor(rect.h / cell.h)`. Fed the full `rect.h`,
    it yields ~1 row too many vs the carved `(r.h - 24)`.
  - `app.rs:1740` — `pane_mouse_cell`/`bottom_anchored_row` trust `pty_size` (#280),
    so the extra row skews top-of-pane clicks. Both consequences derive from the
    same over-tall `pty_size`.
  - `workspace.rs:39` — `inset_right(rect, inset)` is the mirror to follow (carve
    an edge, clamp ≥0, `..rect`). `app.rs:119` already imports `inset_right` +
    `plan_resize` from `crate::workspace`.
- **Decisions:** D1–D3 in the spec. Crux: fix ONLY `pty_size` (size to the carved
  rect) — the render + mouse already trust it, so one fix heals the clip AND the
  skew. `inset_top` mirrors `inset_right`; the const `PANE_TITLE_H` is passed as
  the inset (keeps `workspace.rs` gpui-free).
- **Risk:** minimal. Additive pure helper + a one-line call change; `plan_resize`
  only fires when the grid actually changes (its own guard), so the first frame
  after this ships resizes each pane to the correct (one-shorter) grid. No other
  resize call sites feed a pane rect to `plan_resize` (grep-confirmed: the sole
  site is 7258).

## Phase 2 — Design

### Approach
A pure `inset_top` carves the pane title bar off the rect the resize loop hands
`plan_resize`, so the PTY grid matches the render's content div. The render
(`.top(r.y + PANE_TITLE_H).h((r.h - PANE_TITLE_H).max(0))`) and `pane_mouse_cell`
(trusts `pty_size`, #280) are UNCHANGED — one corrected `pty_size` heals the
top-row clip AND the mouse-row skew. §20 confirmed: Warp (terminal) — a TUI fills
the content area below the pane chrome; matched by sizing the grid to the carved
content rect; clean-room, the rect arithmetic is Marley's own.

```rust
// workspace.rs — mirrors inset_right (the right gutter).
pub fn inset_top(rect: Rect, inset: f32) -> Rect {
    Rect { y: rect.y + inset, h: (rect.h - inset).max(0.0), ..rect }
}
// app.rs:7258 — size the PTY to the CARVED content rect.
plan_resize(inset_top(*rect, PANE_TITLE_H), cell, term.pty_size)
```

### File manifest
| File | Change |
|------|--------|
| `crates/marley_app/src/workspace.rs` | ADD pure `inset_top(rect, inset) -> Rect` (carve the TOP edge; mirror `inset_right`). |
| `crates/marley_app/src/app.rs` | Import `inset_top` (the `use crate::workspace::{…}` at 119-121); wrap the resize-loop rect at 7258 with `inset_top(*rect, PANE_TITLE_H)`. |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | `workspace.rs` `inset_top` unit: normal (`y += inset`, `h -= inset`, `x`/`w` kept); `inset > h` → `h` clamps to 0; `inset == 0` → identity. | REQ-002 |
| T2 | `workspace.rs` `plan_resize` golden: a rect whose full height gives N rows but `inset_top(rect, 24)` gives N-1 — the carved height changes the row count by ~1. | REQ-001 |
| T3 | Driven `htop` capture: the top status row VISIBLE (not clipped) + a top-row click reports the top row. If env-blocked (locked screen), state so + carry via the mechanism (pty_size now equals the content grid the render + `pane_mouse_cell` already consume, both unit-proven). | REQ-003 |
| — | cov/MSI 100 on `inset_top` via `--diff`. | REQ-004 |

Uncoverable: the app.rs one-line call is inside the live gpui render/resize loop (no unit harness); covered behaviorally by T3 (or the mechanism) — the wiring is a trivial pure-fn application whose correctness is T1+T2.

### Risks / decisions
- **R1:** `inset_top` must match the render carving EXACTLY (`y += inset`, `h = (h-inset).max(0)`), else the grid and the drawn content disagree. Verified against app.rs 7285-7287 + 8119-8121. T2 pins the row count.
- **D-scope:** the sole `plan_resize` pane call site is 7258 (grep-confirmed).

## Phase 4 — Validate
- **Tests added (2, `workspace.rs`):** `inset_top_shaves_height_dropping_origin_preserving_width` (T1 — `y+=inset`, `h=(h-inset).max0`, `x`/`w` kept + the `inset>h` clamp; one assert kills every viable mutant like the `inset_right` sibling); `plan_resize_uses_the_content_height_not_the_full_pane` (T2/REQ-001 — a 250-tall pane gives 15 rows full vs 14 rows carved, cols unchanged at 50).
- **`cargo nextest run -p marley` (the 2 new tests): PASS.** Full suite unaffected.
- **`scripts/gates.sh --diff` → `GATE GREEN [diff]`** — 15/15 incl. coverage 100% + MSI 100% on `inset_top`, clippy -D, brand-scrub, miri, visual/AX.
- **Unrelated environmental unblock:** gate:8 cargo-deny went RED mid-session on a YANKED transitive dep (`spin 0.9.8`, pulled from crates.io by its maintainer — gate:8 was GREEN for #285/#286/#288 earlier today). Fixed with `cargo update -p spin` → `0.9.9` (a clean patch bump, 1 package, 63 unchanged) — the standard fix for a yanked lockfile entry, NOT a #287 change; ridden into this commit to unblock the gate. `advisories ok`.
- **Live-drive: documented env-block (not silently skipped).** The fix is visible ONLY when an alt-screen TUI fills a tall pane (a normal bottom-anchored shell leaves the extra row empty at top). Reaching that needs synthetic input to launch a TUI (`top`/vim — `htop` is absent) on the bundled `.app`, which requires Accessibility perm; that has been env-blocked ALL session (every driven capture — #204/#205/#284 — was blocked; no Marley window is running). The fix is carried by the AIRTIGHT mechanism: T2 proves `pty_size` now equals the carved content grid, and the render (app.rs 7285-7287) + `pane_mouse_cell` (#280) ALREADY consume `pty_size` correctly (pre-existing, unit-proven) — so correcting `pty_size` heals the top-row clip AND the mouse-row skew by construction. Re-verify with a driven `top`/vim capture when the machine is unlocked (30s, no ticket).
- No pre-existing test failures.

## Phase 5 — Complete
- **Docs:** CHANGELOG.md ### Fixed (#287); docs/marley_architecture/app_shell.md `workspace.rs sizing` section gained the M17 #287 note (the shim feeds `inset_top(rect, PANE_TITLE_H)` — the content rect — so the grid isn't a row too tall; correcting `pty_size` heals the render clip + the #280 mouse skew).
- **Knowledge (forge):** AAR `4497d877` submitted — completed, effectiveness 5. No failure-record (clean first-try). No new prevention rule (a faithful mirror of `inset_right` + one call-site). Lessons: the "single source of truth" pattern — correct the ONE value both consumers trust (`pty_size`) rather than patch the render + mouse separately ("one fix heals both"); and a yanked transitive dep (`spin`) can turn gate:8 RED mid-session — `cargo update -p <crate>` is the standard unblock.
- **Ticket** TICKET-287 → closed/ (status closed) + forge ticket-close. **Pipeline** spec+notes → completed/.
- **Result:** the PTY grid sizes to the carved content rect; the alt-screen top row is visible and top-of-pane clicks map correctly (mechanism-proven; a driven `top`/vim capture is the env-blocked re-verify). cov/MSI 100, GATE GREEN [diff]. LOCAL commit only (push un-OK'd).

## Phase 3 — Implement
- **Built to the manifest, no deviations.** `workspace.rs`: pure `inset_top(rect, inset)` (carve the TOP edge, clamp `h` ≥0, `..rect`) mirroring `inset_right`. `app.rs`: added `inset_top` to the `use crate::workspace::{…}` import; the resize loop now calls `plan_resize(inset_top(*rect, PANE_TITLE_H), cell, term.pty_size)`. The render (7285-7287) + `pane_mouse_cell` (#280) are UNTOUCHED.
- `cargo check -p marley --all-targets` clean; `cargo fmt` clean; `cargo clippy -p marley` clean (the `block v0.1.6` note is a pre-existing dependency future-incompat warning, not this change).

## Inspect (Phase 3.5)
Inline adversarial trace (a pure fn mirroring `inset_right` + one call site — no subtle invariant). **No defects.**

| Angle | Verdict | Evidence |
|---|---|---|
| **Matches the render carving EXACTLY** | **SAFE** | `inset_top`: `y = rect.y + inset`, `h = (rect.h - inset).max(0.0)`. Render (app.rs 7285-7287): `.top(px(r.y + PANE_TITLE_H)).h(px((r.h - PANE_TITLE_H).max(0.0)))`. With `inset = PANE_TITLE_H` → byte-for-byte identical (also matches the 8119-8121 content rect). |
| **plan_resize rows vs cols** | **SAFE** | `plan_resize` → `grid_axis(rect.h, cell.h)` (rows, from the CARVED h) + `grid_axis(rect.w, cell.w)` (cols, from w — unchanged by `inset_top`). A title bar reduces rows only, correct. |
| **Sub-title-bar pane clamp** | **SAFE** | `(rect.h - inset).max(0.0)` → a pane shorter than 24px yields `h = 0`; `grid_axis(0, cell.h)` clamps ≥1 → 1 row (no panic, no zero grid). |
| **The `y += inset` is harmless to plan_resize** | **SAFE** | `plan_resize` uses only `w`/`h`; the `y` shift makes `inset_top` a faithful content-rect helper (matching the render) but doesn't affect the grid. |
| **Sole resize site** | **SAFE** | Grep-confirmed: the only `plan_resize` pane call is app.rs:7258; no other path feeds a pane rect. |
| **MSI** | **Phase-4-gated** | Viable mutants = `rect.y + inset` (`+`→`-`/`*`) and `rect.h - inset` (`-`→`+`/`/`); `.max(0.0)` is an unmutated method call. T1's exact-value asserts (+ the `inset > h` clamp case) kill them. |

No `failure-record`, no new prevention rule (a faithful mirror of an existing helper). Fully additive; corrects `pty_size` so the render + `pane_mouse_cell` (already trusting it) both heal.
