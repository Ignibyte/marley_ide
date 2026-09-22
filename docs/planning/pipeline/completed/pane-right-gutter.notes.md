---
pipeline_id: 4b23e9ab-70f4-48ac-9db4-2130ebf6573b
ticket: forge#189 (790fa62a-c887-4395-a99f-7cf8309dac39)
aar_id: bdf574a2-37f2-477c-b86b-b7d5cab7b804
---

# Notes — M12.1 #189 pane right gutter

## Phase 1 — Plan
Investigated: center_bounds (app.rs:3969) spans [regions.left+files_w, bounds.w] — flush right, no gutter.
pane_rects (workspace.rs:85) tiles by ratios within it. So the rightmost pane ends AT bounds.w. The 5-pane tab
is from repeated ⌘⇧A halving splits. Fix = a right gutter. PASS.

## Phase 2 — Design
**Ratio check (done, static):** `equal_ratios(n) = vec![1.0/n; n]` (layout.rs:138) sums to ~1.0, and every split
uses it (layout.rs:233) — NO ratio drift. So `pane_rects` fills its bounds exactly; the overflow is purely the
flush-right `center_bounds` (ends at bounds.w, no gutter). **Gutter-only fix confirmed** — no normalization needed.

**Approach.** Add a small pure `inset_right(rect, inset) -> Rect` (a Rect with `w` reduced by `inset`, clamped
≥ 0; x/y/h unchanged) next to `Rect`/`pane_rects` in workspace.rs. The terminal-tab tiling passes
`inset_right(center_bounds, PANE_GUTTER)` to `pane_rects`, giving the split grid an 8px right gutter. Cockpit/code
tabs keep the FULL `center_bounds` (they're a single full-screen panel — a right gap there would look like a
bug, not a gutter). So only the tiled multi-pane grid gains the margin — exactly chad's complaint.

**File manifest.**
- `crates/marley_app/src/workspace.rs` — add `pub fn inset_right(rect: Rect, inset: f32) -> Rect` + unit test.
- `crates/marley_app/src/app.rs` — `const PANE_GUTTER: f32 = 8.0;` (near PANE_TITLE_H); wrap the terminal-tab
  `pane_rects(self.workspace_mut().group(), center_bounds)` → `pane_rects(..., inset_right(center_bounds, PANE_GUTTER))`.

**Regression Test Plan.**
| Test | Proves |
|---|---|
| `inset_right_reduces_width` (workspace.rs) | REQ-001 — w reduced by inset; clamp at 0 when inset ≥ w; x/y/h unchanged (kills the `-`→`+`, the max(0) drop) |
| driven capture | REQ-002 — the multi-pane "Release Run" tab renders with a visible right gutter (rightmost pane inside the window) |
| gate --diff (staged) | REQ-003 |

**Risks/decisions.** Gutter applies ONLY to the tiled grid (cockpit/code full-width — deliberate). `max(0.0)`
guards a window narrower than the gutter (center collapses, no negative width). PANE_GUTTER=8 is a taste value
(matches the ~8px paddings elsewhere). No behavior change when there's a single pane (it just ends 8px earlier).

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` clean.
- `workspace.rs` — `pub fn inset_right(rect, inset) -> Rect` = `Rect { w: (rect.w - inset).max(0.0), ..rect }`.
- `app.rs` — `const PANE_GUTTER: f32 = 8.0;` (by PANE_TITLE_H); imported `inset_right`; the terminal-tab
  `pane_rects(group, center_bounds)` → `pane_rects(group, inset_right(center_bounds, PANE_GUTTER))`. Cockpit/code
  bodies untouched (full width).
**Deviation:** none.

## Inspect (Phase 3.5)
One correctness+mutation critic over the whole (tiny) diff — the pure `inset_right` + the one-line `PANE_GUTTER`
wrap. Lenses: correctness (traces + edges), mutation kill-fixtures, blast radius (cockpit/code untouched), PTY
resize/draw consistency, panics/borrow. I also ran the skip-detach self-check proactively (clean — `PANE_GUTTER`
is a const between consts; `inset_right` is a standalone fn, no adjacent `mutants::skip` shim → nothing detached).

**Findings**
- **[LOW / comment overclaim — REAL, fixed]** The wrap insets the tiling band for *every* terminal tab, not only
  a multi-pane split (`pane_rects` on a `Leaf` returns the full band). Behaviour is correct and arguably better —
  a consistent 8px right margin, and the rightmost pane's border/focus-edge is always visible — but the code
  comment + const doc said "multi-pane split", narrower than the code. **Fix:** reworded both the `const
  PANE_GUTTER` doc (app.rs:302) and the call-site comment (app.rs:4034) to say "every terminal tab (split or
  single)". No behaviour change. (No functional bug — the single-pane inset is intended per the Phase-2 risk note
  "No behavior change when there's a single pane (it just ends 8px earlier)".)
- **[mutation — Phase-4 deliverable]** `cargo mutants --list -f workspace.rs` → 4 candidates on `inset_right`
  (L39–40); mutant 1 (`-> Default::default()`) is **UNVIABLE** (Rect has no `Default` derive/impl → build fail,
  not counted). **3 live viable mutants:** field-delete `w` (→ w=100), `-`→`+` (→108), `-`→`/` (→12.5). The
  `.max(0.0)` clamp produces NO mutant (cargo-mutants doesn't mutate the `.max`/`0.0`). All 3 killed by ONE assert
  with `w=100, inset=8` (92/108/12.5/100 all distinct — avoids the w≈9.14 trap where `−`≡`/`). Expected MSI 3/3.

**Rejected / clean (verified concretely by the critic):**
- `inset_right` correctness — traces confirmed exactly: `{10,5,100,50}·8 → {10,5,92,50}`; narrow `{..w:5}·8 →
  {..w:0}` (clamped, not −3). x/y/h pass through via `..rect`.
- Cockpit / code_view bodies render raw `center_bounds` (app.rs:3982–4007); the `inset_right` wrap is *only* in
  the `active_is_terminal` arm → the inset can never reach a full-screen panel. One call site, one `PANE_GUTTER`
  use (rg-confirmed). No regression.
- PTY resize loop and draw loop iterate the *same* `rect_list` → grid width and drawn width agree (≤1 col
  narrower, intended). No panics/unwraps; no borrow conflict at the call site. `cargo check -p marley` clean.

Verdict: **Phase 3.5 PASS** — no functional bug; one comment fix applied; mutation kill-fixtures locked for Phase 4.

## Phase 4 — Validate
**Tests added.** `workspace.rs::inset_right_shaves_width_preserving_origin_and_height` (one `#[cfg(test)]` unit) —
`inset_right(r(10,5,100,50), 8) == r(10,5,92,50)` (x/y/h pass through, w−8; the 92/108/12.5/100 spread kills
field-delete, `-`→`+`, `-`→`/` in one assert) + the clamp assert `inset_right(r(1,2,5,3), 8) == r(1,2,0,3)`
(5−8 → 0, never negative). Covers REQ-001.

**Unit + mutation (actual).**
- `cargo nextest run -p marley inset_right` → **1 passed**.
- `cargo mutants -p marley -f workspace.rs -F inset_right` → **4 mutants: 3 caught, 1 unviable** (the
  `-> Default::default()` mutant is unviable — Rect has no `Default` — so it doesn't count). MSI on `inset_right`
  = **3/3 = 100%**.

**Driven live-app capture (REQ-002) — done, not deferred.** Stripped the persisted shell blob, `bundle-app.sh
debug`, `open`, frontmost. Split the active tab horizontally twice via the right-click context menu
(`rightclickat:` → `enter`, "Split Right" is the pre-selected item) → a 3-pane tiled tab (tree shows terminal 4 →
pane 1/2/3). Captured the window (`screencapture -l$WIN`) and read it: three panes tile left-to-right and the
**rightmost pane ends inside the window with a visible right gutter**. Quantified by a pixel run-length scan at
y=400: from the window right edge (x=3343) leftward there are **16 physical px of window-bg gutter**, then the
2px pane border (`rgb(50,54,62)`), then the pane body — i.e. the rightmost pane's border sits **exactly 8.0
logical px** (retina 2×) inside the window edge = `PANE_GUTTER`. Before the fix it was flush/overflowing (chad's
feedback #4). Cockpit/code tabs unaffected (single full-screen panel — untouched by the `active_is_terminal` arm).
Settings backup restored afterward.

**Full `--diff` gate (staged): `GATE GREEN [diff]` — 15/15 pass**, incl. gate:4 coverage ≥100% lines, gate:5
mutation MSI ≥100% (diff-scoped), gate:15 visual/AX. Receipt written for `/commit`.

No pre-existing failures in scope. **Phase 4 PASS.**

## Phase 5 — Complete
**Docs.** CHANGELOG `### Fixed` entry (TICKET-189). `app_shell.md` updated in two places: the `workspace.rs`
bullet now lists `inset_right`, and the `RootView` render note records the terminal-tab tiling band being
`inset_right`-shaved by `PANE_GUTTER` (cockpit/code keep un-inset `center_bounds`).

**Knowledge (forge wired).** `aar-submit` (aar `bdf574a2…`, completed, effectiveness 5). No `failure-record` —
the lone inspect finding was a comment overclaim, not a runtime defect. `prevention-rule-record`
`PR-claude-drive-tiling-split-via-context-menu-001` (`8dbd9a37…`): to validate pane tiling, drive the split via
the right-click context menu (`rightclickat:` → `enter` = "Split Right"); ⌘D is `new_terminal_pane()` (a new
tab), not `split_focused_pane()` — I burned one capture discovering that.

**Lessons.**
- Static root-cause first paid off: confirming `equal_ratios` sums to 1.0 (no ratio drift) meant the fix was
  purely the flush-right `center_bounds` — a one-const gutter inset, no normalization. The critic + gate agreed.
- Quantifying a UI fix beats eyeballing: a pixel run-length scan at one row proved the gutter is *exactly* 8.0
  logical px (16 physical, retina 2×) — window-bg 16px → 2px border → pane body — not "looks about right".
- Consistency sweep for the inspect finding: the "multi-pane split" overclaim lived in THREE doc strings
  (const, call-site, and the pure fn); fixed all three, not just the one the critic cited.

forge wired — captured in forge (above) AND locally here per §19.
