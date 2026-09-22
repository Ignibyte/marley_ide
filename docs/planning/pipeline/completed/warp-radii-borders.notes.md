# Warp corner radii / borders — Notes

- **Forge ticket:** #221 (58a6eb14-fef1-46b2-b678-446c7d17cbdd) · closes #224 (aefd6a51-9d9e-42c0-9fd4-77b05db7dc7b)
- **AAR:** d6efcd44-6272-46fb-b167-6d02dcad6ae8
- **Local ticket doc:** docs/planning/tickets/open/TICKET-221-warp-radii-borders.md
- **Pipeline spec:** warp-radii-borders.spec.md

## Phase 1 — Plan
- **Request:** match Warp's rounding/borders/dividers. Auto-approved (/work 195-222). Closes #224.
- **Classification / tier:** work pipeline, small/bounded. Systems: app.rs overlay render (shim — 10 card
  boxes). Pure: the corner_radius/border tokens (already exact-value tested, ui_components lib.rs:184).
- **Forge recall (§18.3):** #194 set the tokens (border/corner_radius/surface); #219 re-consumed
  corner_radius for the rail rows (partial #224); #218 orphaned it (removed the prompt strip). This pass
  wires the token into the overlay cards — its intended use — CLOSING #224. The generic exact-value/f32
  rules apply (but no value change here). AAR opened.
- **Discovery (Explore-mapped, exact lines):**
  - 10 floating overlays, ALL SQUARE (none `.rounded`): Palette l.4899, Launcher l.4940, Finder l.4982,
    History l.5018, Forge l.5043, Fleet l.5072 (these 6 BORDERLESS — just `.bg(colors.surface)`), Diff
    l.5287, Find l.5357, Completion l.5707, Context-menu l.5799 (these 4 have `.border_1()`+`colors.border`).
  - Borders/separators uniform 1px + muted (`caption_header` l.336, block `border_t` l.4403, dock edges
    l.368/4008) — Warp-like, NO CHURN.
  - #130 divider (l.4857-4865): a 6px strip, `bg(border)` at rest → `accent` on hover. Functional +
    Warp-like; a grip handle is optional/subjective → DEFER.
  - corner_radius = px(6), tested at lib.rs:184; only the 3 #219 rail rows (l.3786/3885/3948) consume it.
- **Bounded deltas:** D-A round all 10 overlay boxes; the 6 borderless also get a border so the rounding
  reads. Closes #224.
- **Deferred (documented):** the divider grip; borders/separators (uniform+muted already); git side-panel
  + toast (not floating cards); any token VALUE change.
- **Decisions:** D1 reuse corner_radius/border tokens (no value change); D2 border the 6 borderless (a
  rounded corner needs an edge); D3 shim-only (capture-validated), the token test already guards the value;
  D4 auto-approved, document + close #224.
- **Open questions for Design:** (1) confirm `.rounded(colors.corner_radius)` + `.border_1().border_color(
  colors.border)` compose cleanly on each `.absolute().occlude().bg(surface)` overlay box (no clipping of
  the inner scroll list — rounding clips at the corners; the content is inset by padding so it shouldn't
  hit the corner; confirm per overlay). (2) do any of the 4 framed overlays inset their content such that
  a corner radius would clip a row? (3) any overlay whose box is NOT the outermost (e.g. a header + list in
  separate divs) where the rounding must go on the right element.

## Phase 2 — Design

### Discovery verified (all 10 boxes read)
- **Element / bg:** every overlay's `.bg(colors.surface)` is on the OUTER `div()` at the mapped line (the
  `.absolute()...occlude().bg(surface)` box) — so `.rounded`/`.border` go on that same div (they frame the
  whole card). ✓
- **No scroll conflict:** NONE of the 10 outer boxes has `overflow_scroll`/`on_scroll_wheel`/`overflow_hidden`
  → adding `.overflow_hidden()` is safe (no scroll-container conflict, no taffy size change on an auto-sized
  box). ✓
- **Selected-row bgs → need clipping:** the listing overlays (palette l.4930, launcher l.4954, finder
  l.5001, history l.5033) draw a selected `row.bg(colors.accent)` with NO padding on the outer box, so the
  content sits flush to the edges. A rounded box withOUT clipping would let a corner row's square bg poke
  past the rounded card corner. → add `.overflow_hidden()` so the rounding clips the content to the card
  shape (the standard rounded-card idiom). ✓
- **Double-border:** the 4 framed overlays (Diff l.5296, Find l.5366, Completion l.5714, Context-menu
  l.5805) ALREADY have `.border_1().border_color(colors.border)` → they get ONLY `.rounded` (+ overflow),
  NOT a second border. ✓

### Architecture / approach — SHIM-only render wiring
Each of the 10 overlay outer boxes gets `.rounded(colors.corner_radius).overflow_hidden()`. The 6 borderless
listing overlays ALSO get `.border_1().border_color(colors.border)`. `.rounded`/`.border`/`.overflow_hidden`
are paint/clip styling inside the `#[cfg_attr(test, mutants::skip)]` render — no pure logic, no new state,
`.occlude()`/position untouched. The `corner_radius`/`border` tokens are the pure values, already
exact-value tested (ui_components lib.rs:184). Capture-validated (like #217/#220). §14 clean.

### File manifest
- `crates/marley_app/src/app.rs` — 10 overlay-box edits:
  - **+`.rounded(colors.corner_radius).overflow_hidden()` + `.border_1().border_color(colors.border)`** on the
    6 borderless: Palette (~4909), Launcher (~4948), Finder (~4990), History (~5026), Forge (~5051), Fleet (~5080).
  - **+`.rounded(colors.corner_radius).overflow_hidden()`** only on the 4 framed (they already have the
    border): Diff (~5295), Find (~5364), Completion (~5713), Context-menu (~5804).
  - Each on the SAME `.bg(colors.surface)` outer div. A brief `#221 (Warp parity)` comment.

### Regression Test Plan
| REQ | test |
|---|---|
| REQ-001 | driven capture — open the command palette (⌘⇧P) + ≥1 other overlay (context menu via right-click, or the find bar); the card has ROUNDED corners. Review: 10 `.rounded(colors.corner_radius)` sites. |
| REQ-002 | driven capture — the palette (a borderless listing overlay) now shows a 1px muted BORDER defining the rounded edge. Review: `.border_1().border_color(colors.border)` on the 6 borderless. |
| REQ-003 | review + capture — the overlay CONTENTS (rows/text) + position are unchanged; the caption_header/dock/block separators + the #130 divider are untouched by the diff. |
| REQ-004 | review — the 10 card sites consume `colors.corner_radius` (resolving #224); close #224 at complete. |
- **Uncoverable by unit test:** the shim render (`mutants::skip`) — validated by driven captures (like
  #217/#220). No new pure logic; the `corner_radius` token exact-value test (lib.rs:184) already guards the
  value → cov/MSI unaffected.

### Risks / decisions
- **R1 — corner artifact.** Solved by `.overflow_hidden()` on all 10 (clips the selected-row bgs to the
  rounded card shape). Verified no scroll conflict on the outer boxes.
- **R2 — the 1px border insets content 1px on the 6 borderless.** Negligible + expected for a framed card
  (Warp's overlays are framed); the listing rows shift in 1px uniformly — no misalignment.
- **R3 — overflow_hidden on an auto-sized box** does NOT change its size (children define it) — it only
  clips paint to the (rounded) bounds. No layout/position change; `.occlude()` untouched.
- **R4 — the palette can grow tall (one child per match).** overflow_hidden doesn't change that (the box is
  as tall as its content); it only clips the corners — pre-existing height behavior unchanged.

## Phase 3 — Implement
- **app.rs — 10 overlay-box edits** (each on the outer `.bg(colors.surface)` div):
  - 6 borderless (Palette 🔍, Launcher 🧠, Finder 📄, History 🕐, Forge, Fleet 🛰) — added
    `.rounded(colors.corner_radius).overflow_hidden().border_1().border_color(colors.border)`.
  - 4 framed (Diff, Find, Completion, Context-menu) — added `.rounded(colors.corner_radius).overflow_hidden()`
    only (they already had `.border_1().border_color(colors.border)` — no double border).
  - Each edited by DISTINCTIVE content (the emoji child / `POPUP_W` / `MENU_W` / the diff's `bounds.w*0.7`
    width / the find's text_color-before-border) to survive line-drift; a `#221 (Warp parity)` comment each.
- **Guarded against the non-targets:** two OTHER `.bg(surface).border_1()` elements below l.5300 (the git
  changes side-panel ~5164, and the top-search results list ~5674) are NOT overlays (per the Explore map)
  and were NOT touched — verified: `grep -c .rounded(colors.corner_radius)` = **13** (3 #219 rail rows + 10
  overlays), `grep -c #221` = **10** (exactly the 10 edits). A first diff-anchor attempt matched 2 sites
  (the git panel shares the pattern) → re-anchored on the diff's unique `.w(px(bounds.w * 0.7))`.
- **Deviations from design:** none. (All 10 as designed; overflow_hidden on each for the clean rounded-card
  clip.)
- `cargo fmt` + `cargo check -p marley` clean (only the pre-existing transitive `block v0.1.6` note).
  (`.overflow_hidden()` total = 16 = my 10 + 6 pre-existing unrelated panels.)

## Inspect (Phase 3.5)
1 focused critic (edit-site correctness / double-border / overflow / clean-room) + my own diff-grep
self-review. **No findings.**

**Critic returned — fully corroborates the self-review: "No defects; verified edit-sites, double-border,
overflow, clean-room."** It confirmed all 10 `.rounded` on genuine overlay boxes (with line numbers: Palette
l.4911, Launcher l.4955, Finder l.5002, History l.5043, Forge l.5073, Fleet l.5107, Diff l.5329, Find
l.5402, Completion l.5753, Context-menu l.5847); the git side-panel (`.border_l_1()` only) + the top-search
hits list stayed SQUARE (the reported anchor false-match did NOT land); total 13 `.rounded` (3 rail + 10);
6 `.border_1()` added (borderless only, no double-border); overflow-safe (no scroll container, all
auto-sized). One NON-blocking, PRE-EXISTING note (NOT from this diff): the Palette + Diff can grow taller
than the viewport (neither caps height) — out of scope for #221.

**Self-review (evidence-backed):**
- **Edit-site correctness — VERIFIED.** `git diff` shows 10 `+.rounded(colors.corner_radius)` additions,
  each on a genuine floating-overlay outer box (each preceded by its distinctive `.bg(colors.surface)` +
  the overlay's identity — 🔍 palette / 🧠 launcher / 📄 finder / 🕐 history / forge / 🛰 fleet / the
  `bounds.w*0.7` diff / find / `POPUP_W` completion / `MENU_W` context-menu). Total
  `.rounded(colors.corner_radius)` = 13 (3 pre-existing #219 rail rows + 10 new). The two NON-target
  `.bg(surface).border_1()` elements — the git changes side-panel + the top-search hits list
  (`HitKind::Session` / `for (i, hit) in hits`) — are NOT in the diff (stayed square); a first diff-anchor
  attempt matched the git panel by accident (2-match error) → re-anchored on the diff's unique
  `.w(px(bounds.w * 0.7))`.
- **Double-border — VERIFIED.** The diff has exactly 6 `+.border_1()` (the 6 borderless overlays); the 4
  framed additions (comment "rounded card (already framed)") add ONLY `.rounded`+`.overflow_hidden`, no
  second border.
- **Overflow/clip safety — VERIFIED.** None of the 10 outer boxes has `overflow_scroll`/`on_scroll_wheel`
  (checked at design) NOR a fixed `.h(px)`/`.max_h`/`.h_full` (checked now) → they auto-size to content, so
  `.overflow_hidden()` clips ONLY the rounded corners, never real content (it's needed to clip the
  selected-row `bg(accent)` to the rounded card shape). The palette grows one child per match (auto-height)
  → overflow_hidden doesn't change what's visible.
- **Clean-room / scope — VERIFIED.** Tokens only (`corner_radius`/`border` — no new hsla/hex in the diff);
  the diff touches ONLY the 10 overlay boxes — no separator / dock-edge / #130-divider / #219-rail change.

**Verdict:** no findings requiring a fix. No forge `failure-record` (no bug). Lenses: edit-site correctness,
double-border, overflow/clip safety, clean-room/scope.

## Phase 3.5 — Inspect
- (superseded by "## Inspect (Phase 3.5)" above)

## Phase 4 — Validate
- **No unit test** (shim-only render; the `corner_radius` token exact-value test at ui_components lib.rs:184
  already guards the value) — like #217/#220. `cargo nextest run -p marley` = **301 passed, 2 skipped** (the
  10 render edits broke nothing).
- **Driven capture (live app; RE-BUNDLED during inspect):** `scratchpad/221-palette-after.png` +
  `221-palette-corner-tl.png` — opened the command PALETTE (⌘⇧P). It now renders as a **rounded, framed
  card**: the top-left corner clearly curves (6px `corner_radius`) with a visible 1px muted `border`, and the
  selected row's accent bg is clipped to the card edge (`overflow_hidden`), with contents/text fully intact.
  **REQ-001** (rounded) ✓, **REQ-002** (border on a borderless overlay) ✓, **REQ-003** (contents intact) ✓.
  This is the strongest visual — a BORDERLESS listing overlay transformed into a real Warp card (the primary
  #224 fix).
- **The 4 framed overlays** (Diff/Find/Completion/Context-menu) got the IDENTICAL `.rounded()` on their
  already-bordered boxes — code-verified by the inspect critic (all 10 sites). A second live capture was
  finicky: the palette is modal (`.occlude()`) so a right-click didn't reach the context menu, and ⌘F didn't
  surface the find pill cleanly in the driven run — but the palette (a real overlay) + the critic's code
  verification of all 10 sites cover the ACs. (Stated per §7: harness couldn't cleanly reach a 2nd distinct
  overlay in the driven run; not silently skipped.)
- **REQ-004 (#224):** 13 `corner_radius` consumers now (3 rail + the 10 cards) — the token frames the
  floating cards (its intended use). Close #224 at complete.
- **Gate:** `git add -A` + `scripts/gates.sh --diff` — GREEN (below). Shim-only → cov/MSI unaffected.
- **Pre-existing exclusions:** none (the `block v0.1.6` note is upstream; the palette/diff over-tall-list note
  is pre-existing + out of scope per the critic).

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG.md — #221 entry (above #220), "closes #224". app_shell.md — added the #221
  overlay-rounding note to the #219 rail entry (13 corner_radius consumers, #224 resolved) + updated the #218
  note's "reserved token" line (now closed by #221).
- **Knowledge (forge):** `aar-submit` d6efcd44 (completed, effectiveness 5). No `failure-record` (no bug — the
  critic's over-tall-list note was pre-existing/out-of-scope). No new prevention rule (overflow_hidden for a
  rounded card is standard; editing-by-distinctive-content-to-survive-drift is a technique, not a rule).
- **#224 CLOSED** (forge ticket-close done, with a comment noting #221 resolved it — the 10 overlay cards now
  consume corner_radius).
- **Lessons:** (1) editing 10 near-identical sites by DISTINCTIVE content (emoji child / POPUP_W / MENU_W /
  the diff's unique width) survives line-drift + avoids false-matches — a first bare-anchor attempt matched
  the out-of-scope git panel (2-match error) → re-anchor on a unique token. (2) a rounded card needs
  `.overflow_hidden()` (to clip content to the shape) + an edge (border) to READ as rounded — rounding alone
  on a borderless auto-sized box leaves selected-row bgs poking the corners. (3) a modal `.occlude()` overlay
  (the palette) blocks driving a SECOND overlay in the same capture — capture them separately or accept the
  strongest visual + the critic's code verification.
- **Close/archive:** TICKET-221 open→closed; forge ticket-close #221 done; pipeline pair → completed/.
