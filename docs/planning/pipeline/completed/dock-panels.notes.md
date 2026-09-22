# dock panels + titlebar — Notes

- **Forge ticket:** #38 `d6bf9b17-cc33-4cd1-8792-599806b52e22`
- **AAR:** `47b891b7-03fa-4399-b892-82f46c5217ab`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-038-dock-panels.md
- **Pipeline spec:** dock-panels.spec.md

## Phase 1 — Plan
- **Request:** forge #38 (M1.E "The Warp Look" seq-5, auto-approved) — the dock panel treatment +
  titlebar cohesion. The most shim-heavy ticket.
- **Classification / tier:** work pipeline, `feature`, mostly-SHIM (the dock/titlebar render) with a
  thin PURE surface (`dock_title`). marley_app only.
- **Discovery (§18):**
  - The dock render (app.rs:612-634): two near-identical `if regions.{left,right} > 0.0` blocks, each
    a bare `div().absolute()…bg(colors.surface).child("Files"/"Details")` — no header/divider/padding.
    The label is an inline hardcode → extract to `dock_title`.
  - `DockSide { Left, Right }` (layout.rs:54) — 2 variants → exhaustive `dock_title` match. `layout.rs`
    already owns the pure `region_widths` (#24, tested) + a test mod (305).
  - Titlebar: `run()` (app.rs:889) sets `TitlebarOptions { title: "Marley" }` — the native macOS
    titlebar; custom wordmark chrome is limited. The cohesion lever is the root bg = `background`.
  - Deps #34 (font) + #35 (palette) DONE — the panel uses surface/border/foreground.
- **Decisions:** D1–D4 in the spec (dock_title pure; divider on the inner edge; muted caption via
  size; native titlebar + themed root bg).
- **Open questions for Design:** whether to factor a private shim `dock_panel(...)` helper vs inline
  both branches (only 2 — leaning inline with the shared style); the caption "muted" tone (smaller
  text_sm + `border`-ish color, no new role); confirm the current root bg (is it already
  `background`?).
- **AAR id:** `47b891b7-03fa-4399-b892-82f46c5217ab`.

## Phase 2 — Design

### PURE — `crates/marley_app/src/layout.rs` (beside `DockSide`/`region_widths`)
```rust
/// The panel title for a dock side (R44) — extracted from the render's inline hardcode.
pub fn dock_title(side: DockSide) -> &'static str {
    match side {
        DockSide::Left => "Files",
        DockSide::Right => "Details",
    }
}
```

### SHIM — `crates/marley_app/src/app.rs` (a free helper, mutants::skip; + 2 call sites)
```rust
#[cfg_attr(test, mutants::skip)]
fn dock_panel(side: DockSide, x: f32, w: f32, h: f32, colors: &ThemeColors) -> gpui::Div {
    let panel = div().absolute().left(px(x)).top(px(0.0)).w(px(w)).h(px(h))
        .bg(colors.surface).flex().flex_col();
    // the divider on the INNER edge (toward the center pane)
    let panel = match side { DockSide::Left => panel.border_r_1(), DockSide::Right => panel.border_l_1() };
    panel.border_color(colors.border)
        .child(div().w_full().px_3().py_2().border_b_1().border_color(colors.border)
            .text_sm().text_color(colors.foreground).child(dock_title(side)))   // header
        .child(div().flex_1().p_3())                                            // content pad (M2 fills)
}
```
Call sites replace the two bare `div()…child("Files"/"Details")`:
```rust
if regions.left > 0.0 { root = root.child(dock_panel(DockSide::Left, 0.0, regions.left, bounds.h, &colors)); }
if regions.right > 0.0 { root = root.child(dock_panel(DockSide::Right, regions.left + regions.center, regions.right, bounds.h, &colors)); }
```
+ `use marley_ui_components::ThemeColors;` (for the helper signature).

### Decisions
- D-2.1 REQ-003 is ALREADY met — `render` sets `root.bg(colors.background)` (app.rs:510); #38 changes
  only the dock panels, no root-bg edit (a regression-confirm, not a change).
- D-2.2 `dock_panel` is a free shim helper (both sides share it) — the divider side is the only
  per-side branch (`border_r` Left / `border_l` Right). D-2.3 Header caption = `dock_title` in
  `text_sm` foreground with a `border_b` under it; icons deferred; "muted" is via size.
- D-2.4 The pure surface is ONLY `dock_title` (the layout math is #24's tested `region_widths`).

### File manifest
- M `crates/marley_app/src/layout.rs` — `dock_title` + test.
- M `crates/marley_app/src/app.rs` — the `dock_panel` helper + the 2 call sites + the `ThemeColors`
  import.
- M `docs/specs/SPEC-app-shell.spec.md` — R44 + AC row 44 + Mutation-Targets. CHANGELOG; arch doc.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `dock_title_by_side` — `dock_title(Left)=="Files"`, `dock_title(Right)=="Details"` (both arms; kills the whole-fn `""`/`"xyzzy"` replace + an arm swap) | unit |
| REQ-002 | the dock-panel render (header + inner-edge divider + padding) | shim + masked 3-region visual — chad-verified |
| REQ-003 | root bg = `background` (already met) | shim + masked visual (regression) |
| REQ-004 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs `dock_panel` div layout (shim exclude; needs a live window).

### Risks / decisions
- The change is render-heavy — `dock_title` is the only new mutation surface (2 arms, both asserted →
  MSI 100). The panels are masked-shim, chad-verified; no cov/MSI risk beyond `dock_title`.
- `region_widths` (#24) is unchanged — the panels sit inside the same region rects, so no layout
  regression; the divider is a border INSIDE the dock's existing rect (no width change).
- gpui method availability (`border_r_1`/`border_l_1`/`border_b_1`/`px_3`/`text_sm`/`flex_1`) verified
  at implement via `cargo check`.

## Phase 3 — Implement
- **Built (per manifest):** `layout.rs` — `dock_title(side)` (Left→"Files", Right→"Details");
  `app.rs` — a free `dock_panel(side, x, w, h, &colors) -> gpui::Div` shim helper (surface bg +
  `flex_col` + the inner-edge divider [`border_r` Left / `border_l` Right] + a header row [dock_title
  in `text_sm` foreground, `px_3`/`py_2`, a `border_b` divider] + a `flex_1` padded content area),
  replacing the two bare `div()…child("Files"/"Details")` call sites; the `dock_title` + `ThemeColors`
  imports. SPEC-app-shell R44 + row 44 + Mutation-Targets; CHANGELOG.
- **Deviations from design:** none. (REQ-003 needed no change — root bg was already `background`.)
- **Verification at this phase:** `cargo check -p marley` 0 errors (all gpui methods
  `border_r_1`/`border_l_1`/`border_b_1`/`px_3`/`py_2`/`p_3`/`text_sm`/`flex_1`/`w_full` exist);
  fmt; clippy `-D warnings` 0; docs gate 0. The `dock_title` unit test is Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (scoped cargo-mutants on `dock_title` + divider-side/geometry/re-hardcode reads).
  Verdict: **PASS** — no code defect; the diff's one logic-bug risk (divider side) is correct.
- **Findings table:**
  | # | Sev | Finding | Verdict | Action |
  |---|---|---|---|---|
  | F1 | MED (Phase-4) | `dock_title` MSI 0/2 — both whole-fn mutants (`→""`, `→"xyzzy"`) survive because NO test exists yet (expected at inspect; layout.rs is NOT shim-excluded). | REAL (Phase-4) | P4 adds `dock_title(Left)=="Files"` + `dock_title(Right)=="Details"` — the critic confirmed this pair kills BOTH survivors. cargo-mutants generated NO arm-swap mutant (only whole-fn replaces). |
  | F2 | LOW | The header caption uses `foreground` (not a muted tone) — `ThemeColors` has no muted role, so a muted caption needs a NEW token (out of this shim's scope). | ACCEPTED (noted) | A `muted_foreground`/caption token is a #39 (typography pass) consideration; `foreground`@`text_sm` is fine now. |
- **Verified CLEAN by the critic (the 3 real risks):**
  - **Divider side CORRECT** (app.rs:95-97): `Left→border_r_1` (inner=right ✓), `Right→border_l_1`
    (inner=left ✓) — both center-facing; a swap would've compiled + passed the shim gate but isn't present.
  - **No re-hardcode:** full-file grep of app.rs for `"Files"`/`"Details"` = none; labels flow ONLY
    from `dock_title(side)`.
  - **No #24 regression:** call-site geometry matches the old bare divs exactly; `region_widths`
    untouched; the dividers are `border_*_1` drawn INSIDE the fixed `.w(px(w))` → no width shift.
  - No panic; both match arms return `gpui::Div` (type-checks); `mutants::skip` correctly exempts the shim.
- **No code fix** — the diff is sound. F1 is the standard Phase-4 test.

## Phase 4 — Validate
- **Test added (layout.rs):** `dock_title_by_side` — `dock_title(Left)=="Files"`,
  `dock_title(Right)=="Details"` (the critic-confirmed pair that kills both whole-fn mutants).
- **Runs (actual):** `cargo nextest run -p marley` → 99 passed (the new test PASS);
  `cargo nextest run --workspace` → 504 passed, 5 skipped.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **2 caught /
  0 missed → MSI 100.0%** (the 2 `dock_title` whole-fn mutants). Receipt written. No PTY hang.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `app_shell.md` — the dock-panel bullet.
  SPEC-app-shell R44 at implement.
- **Knowledge captured:** no new prevention rule (a clean shim ticket — the critic verified the one
  logic risk [divider side] was correct). aar-submit `completed` (score 5). Win: the shared
  `dock_panel` helper de-duplicated the two near-identical dock branches AND extracted the label into
  a tested `dock_title`, so a render-heavy visual ticket still lands a real (if thin) cov/MSI-100 pure
  surface. Noted for #39: a `muted_foreground` caption token.
- **Ticket:** forge #38 → done; local doc → closed/; pipeline pair archived. 5 of 6 in M1.E.
