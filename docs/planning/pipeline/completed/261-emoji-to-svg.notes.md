# 261-emoji-to-svg — Notes

- **Forge ticket:** #261 c8ab7eee-86ba-4213-90db-05d8f9e38f45
- **AAR:** f54937d4-737f-443b-837c-9c10d01562bf
- **Local ticket doc:** docs/planning/tickets/open/TICKET-261-emoji-to-svg.md
- **Pipeline spec:** 261-emoji-to-svg.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan
- **Request:** /goal batch, ticket 4 of 10. "NO mo emojis" — the 11
  remaining colorful emoji → the #137 tintable SVG system.
- **Classification / tier:** chore, one slice (the audit already split
  A/B/C precisely and pre-mapped every replacement; the ticket allows a
  per-group split if SVG vendoring inflates — not needed, the assets are
  10 small self-authored files).
- **Forge recall (§18.3):** AAR opened. The #232 conversion (status
  indicator) is the shipped pattern template; #137 the asset system.
- **Discovery:** icon-audit.md IS the discovery (11 emoji, 3 groups, exact
  sites + proposed replacements + reuse calls). icons.rs read: 9 variants
  today, icon_path + distinctness test to extend. Assets dir: 9 SVGs +
  NOTES.md. Audit warns app.rs line numbers drifted — design re-greps the
  glyphs.
- **Decisions:** D1–D5 in the spec (reuse sparkle/forge/files; pure fns
  return Icon; header icons shim-inline; self-authored 24×24 stroke SVGs;
  grep proof).
- **Autonomy note:** /goal run — no human pause.

## Phase 2 — Design

### Architecture / approach
The #137/#232 system extends: 10 new `Icon` variants + 10 self-authored SVG
assets; the two pure seams change return type to `Icon`; 15 shim string-sites
(7 emoji) swap `format!("<emoji> {text}")` for an icon+text flex row. §20
confirmed N/A (self-authored geometric primitives; house style = 24×24
viewBox, FILLED single path — gpui rasterizes an alpha mask and tints via
`text_color`; no stroke).

**Live site map (re-grepped — the audit's lines had drifted):**
- 🔍 `\u{1f50d}`: app.rs 4875/4877 (session filter), 6379 (palette),
  6895/6897/6903 (find bar ×3 branches), 7189/7191 (top search) → Search
- 🧠 6479 (launcher) → NewAgent (reuse sparkle) — ALSO reword the 4 stale 🧠
  comments (3736/3738/3746/7064) to "the agent-launch icon" (they'd be
  factually wrong once the icon is a sparkle)
- 📄 6526 (finder) → File; 🕐 6567 (history) → Clock; 🔨 6602 (forge) →
  Forge (reuse); 🛰 6632/6634 (fleet) → Fleet; 🔀 6856 (agent-diff) → Diff
- file_icon (file_tree_view.rs:8-17): rs→CodeFile, toml→Gear,
  **json→CodeFile** (was the text "{}"; structured machine text — brackets
  fit; distinctness loss documented), md|markdown→Document, _→File
- pane_icon (workspace.rs:252-258, ALL 4 arms): Terminal→Terminal,
  FileTree→Files (reuse), CodeView→File, Git→GitBranch
- Render sites: Files-panel row (app.rs ~2478 region) + pane title (~5258
  region) swap the text child for the svg idiom.

**SVG authoring specs** (all filled paths; donuts/cutouts via
`fill-rule="evenodd"` — resvg supports it; R1 fallback = opposite-winding
subpaths): search (ring + handle), file (page + folded-corner cutout),
clock (ring + two hand bars), grid/fleet (4 rounded squares), diff (⇄ two
opposing arrows), code-file (‹ / › three polygons), gear (8-tooth ring +
center hole), document (page + 3 line cutouts), terminal (frame ring +
prompt chevron + cursor bar), git-branch (2 node dots + trunk band + branch
band). NOTES.md gains one provenance row each ("self-authored geometric
primitive, #261").

### File manifest
| file | change |
|---|---|
| crates/marley_app/assets/icons/{search,file,clock,grid,diff,code-file,gear,document,terminal,git-branch}.svg | NEW ×10 |
| crates/marley_app/assets/icons/NOTES.md | +10 provenance rows |
| crates/marley_app/src/icons.rs | +10 variants, +10 icon_path arms, tests extended (19 distinct) |
| crates/marley_app/src/file_tree_view.rs | file_icon → Icon; tests rewritten |
| crates/marley_app/src/workspace.rs | pane_icon → Icon; tests rewritten |
| crates/marley_app/src/app.rs | 15 header sites → icon+text rows; 2 seam render sites → svg idiom; 4 comment rewords |

### Regression test plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | file_icon_by_extension rewritten: rs/RS→CodeFile, toml→Gear, json→CodeFile, md+markdown→Document, txt/noext→File | unit |
| REQ-002 | pane_icon exact 4-arm asserts + distinctness (Terminal/Files/File/GitBranch) | unit |
| REQ-003 | icon_path_maps_each_variant extended to 19 exact arms + dedup==19; missing asset = `include_bytes!` compile error (Assets embeds the dir) | unit + compile |
| REQ-004/005 | driven captures: palette ⌘⇧P, finder ⌘P, history ⌘R, fleet ⌘⇧E headers + Files panel rows + a pane title — tinted monochrome icons, zero colorful emoji | visual |
| REQ-006 | `grep -n '\\u{1f50d}\|…\|\\u{1f4c1}\|🔍…' crates` → zero (strings AND comments after the reword) | grep |
- Uncoverable: none new (render sites live in the documented app.rs shim
  exclude; the pure seams are cov/MSI-100).

### Risks / decisions
- R1 evenodd support in gpui's rasterizer — verify via the driven capture;
  fallback = reverse-winding subpaths (same geometry).
- R2 Header layout shift (text child → flex row) — captures confirm
  alignment; gap_1 + items_center matches the #137 top-bar idiom.
- R3 json loses its distinct "{}" marker (→ CodeFile) — documented,
  low-value distinction.
- R4 pane_icon/file_icon type change ripples ONLY their tests + 2 render
  sites (grep-verified: no other callers).

## Phase 3 — Implement
- **Built to manifest:** 10 SVGs authored (filled 24×24 house style, evenodd
  rings/cutouts); NOTES.md provenance table appended; icons.rs +10 variants
  +10 arms + tests extended to 19; file_icon → Icon (rs|json→CodeFile,
  toml→Gear, md|markdown→Document, _→File) + tests rewritten; pane_icon →
  Icon (all 4 arms) + test rewritten (pairwise-distinct — Icon isn't Hash);
  app.rs: `icon_label` helper (mutants::skip, render-shape) + 11
  Edit-swapped header sites (session filter, palette, launcher, finder,
  history, forge, fleet ×2, agent-diff, find-bar label+child, top search) +
  Files-panel glyph (entry_color hoisted; dirs keep ▸/▾ text per bucket C) +
  pane-title svg swap; 4 stale brain-icon comments + 5 folder-emoji comments
  reworded (comments would've been factually wrong / kept the sweep dirty).
- **Deviations:** none functional. The find-bar label branches lost their
  emoji INSIDE the string builder and the icon wraps at the single child
  site (cleaner than three icon_labels).
- **Verification:** full 22-pattern emoji sweep (11 escapes + 10 literals)
  → ZERO hits crate-wide (exit 1); cargo fmt + check green.

## Phase 3.5 — Inspect
- **Critics run:** 2 parallel — A (general-purpose: SVG validity via xmllint +
  a written path tokenizer, geometry bounds check, asset wiring, tests,
  clippy, layout); B (Explore: broad emoji sweep, right_dock, provenance
  judgment, DRY, doc drift).
- **Findings ledger:**

| # | sev | finding | verdict | action |
|---|---|---|---|---|
| A1 | CRITICAL | the 10 new SVGs were NOT registered in `Assets::load` (a static 9-arm include_bytes! match, NOT a dir embed) — every new icon = a silent 12px blank at runtime; compile/tests/clippy all green; the notes' "compile proof" claim was FALSE | REAL — the ticket's whole visible purpose defeated | FIXED: 10 match arms + the permanent `assets_serve_every_icon_variant` guard (every Icon → icon_path → load → Some(non-empty)); notes claim corrected; BF + PR recorded |
| A2/B1 | HIGH | the mutants::skip DETACH TRAP (3rd recurrence): icon_label inserted between caption_header's doc/attr and fn — caption_header lost its skip+doc (a body mutant appeared in --list; unviable only by luck, gpui::Div has no Default) | REAL | FIXED: icon_label block re-seated above; --list re-run = 0 mutants on both helpers; BF recorded |
| A3 | LOW-MED | Files-panel file row lacked items_center — the 12px svg rides ~2px high beside the text | REAL | FIXED: .items_center() added |
| A4 | LOW | spec/notes cited a vacuous "include_bytes! dir-embed compile proof" | REAL (the belief behind A1) | corrected here; the guard test is the real proof |
| A5 | INFO | spec's "stroke=currentColor fill-none house style" line was wrong (existing assets are FILL-based; the shipped set correctly matched the assets) | noted | none (docs corrected by this ledger) |
| B2 | LOW | icon-audit.md still claims "11 emoji STILL RENDER" + follow-up rows A/B/C open | REAL doc drift | Phase 5 (planned status flip) |
| B3 | LOW | right_dock.rs:74 stale "one recognizable emoji per section" wording (renders SVGs since #137) | REAL | FIXED: comment reworded |
| B4 | LOW | optional icon_el DRY across 3 svg-builder sites | REJECTED — the two raw sites need per-row color / non-label layout (critic's own judgment: justified) | none |

- **Verified clean:** SVG 10/10 well-formed + single-path + viewBox 24 +
  every coordinate within [-2,26] (a written tokenizer, no y=113-class
  typos); provenance judged defensible (filled coarse geometry, structurally
  unlike stroke-based packs; NOTES.md claims self-authored, names no pack as
  source); broad 4-byte F0-9F sweep + escape sweep = zero missed pictographs
  (the `\u{1f600}` in input.rs is test data for width handling; `\u{001f}`
  in grid_layout is the unit-separator control char); icon_label ×12 all
  muted-tinted; find-bar label branches compose clean; allocations
  equivalent to the old format! strings.
- **Post-fix verify:** nextest 369/369 (the new guard included); fmt clean;
  `cargo mutants --list -f app.rs` shows 0 mutants on
  caption_header/icon_label.

## Phase 4 — Validate
- **Tests (written at implement/inspect, RUN here):** icons.rs 19-arm exact +
  distinctness; file_icon 8-case Icon mapping; pane_icon 4-arm +
  pairwise-distinct; `assets_serve_every_icon_variant` (the A1 guard — every
  Icon → icon_path → Assets::load → Some(non-empty)).
- **RUN:** `cargo nextest run --workspace` → **895/895 passed, 5 skipped**;
  doctests 0 failed.
- **Driven live captures (PNGs read + asserted; scratchpad/261-*.png):**
  1. Files panel: gear on audit.toml, lined-document on every .md, generic
     page on .sh, code-brackets on .rs + settings.json — all muted-tinted
     monochrome, dirs keep ▸/▾; NO colorful emoji (REQ-005, and proof the
     A1 fix works — these are the NEW assets rasterizing, including the
     evenodd gear ring).
  2. Palette ⌘⇧P: the magnifier leads the query row (evenodd donut renders
     correctly); keycap chips normal (REQ-004).
  3. Finder ⌘P captured (file icon header); 4. Fleet ⌘⇧E: the 2×2 grid icon
     leads "no agents running" (REQ-004).
- **REQ-006 (RUN at implement):** 22-pattern sweep (11 escapes + literals) →
  zero crate-wide.
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] — 15/15**
  (coverage ≥100%, MSI ≥100% — the icons/file_icon/pane_icon mutant sets
  killed by the rewritten pins). Receipt written.
- **Pre-existing:** upstream `block v0.1.6` note only.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG [Unreleased]/Changed entry; icon-audit.md status
  flipped (groups A/B/C DONE by #261; D/E remain); app_shell.md gains a #261
  M-log bullet.
- **Knowledge (§19):** AAR submitted (completed). Captured at inspect:
  BF-claude-svg-asset-not-registered-renders-blank (CRITICAL class),
  PR-claude-new-asset-needs-loader-registration-and-guard-001,
  BF-claude-mutants-skip-detach-third-recurrence.
- **Ticket:** TICKET-261 → tickets/closed/, forge #261 → done.
- **Archive:** spec+notes → docs/planning/pipeline/completed/.

## Phase 5 — Complete
- (pending)
