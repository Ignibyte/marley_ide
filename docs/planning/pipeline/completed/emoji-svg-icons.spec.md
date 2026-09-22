---
pipeline_id: 55023b11-bf8d-4348-a513-ebc636b8a6da
ticket: forge#232 (9e62e2a0-96e7-485c-b07e-0e2fcc9c36c9) · local docs/planning/tickets/open/TICKET-232-emoji-svg-icons.md
aar_id: 42d78ef3-f433-44fe-b773-da667a1b53f5
status: Phase 5 — Complete PASS
title: Replace remaining rendered emoji/glyphs with clean-room SVG icons + emoji→icon audit + brand-icon list
type: chore
milestone: M13
references: [forge#137, forge#230, forge#194]
---

## Title
Finish the emoji→icon migration chad asked for (feedback #1, "remove all emojis for icons").
chad feedback #1: "remove all emojis for icons" + "a list of icons an artist should make." A full
audit (this ticket) found that #137 only ever converted 6 glyphs (top bar + cockpit); **11 colorful
emoji still render across 3 subsystems #137 never touched** — overlay/search headers (🔍×5 🧠 📄 🕐
🔨 🛰 🔀), file-type icons (🦀 ⚙ 📝 📄 in `file_icon`), and pane-type icons (📁 📄 in `pane_icon`) —
plus ~40 bucket-B monochrome symbols (already tint) and bucket-C disclosure primitives (keep). That
"emoji" surface is far larger than one ticket, and chad said it "doesn't need to be now" while
explicitly asking for the artist list.

So THIS ticket delivers the two things chad actually asked for now — a comprehensive **emoji→icon
AUDIT** (every rendered glyph catalogued + bucketed) and a **brand-icon list** (standard vendorable
set vs. artist-needed brand set) — and converts ONE concrete slice, the always-visible per-block
**status indicator** (○ ✓ ✗), to clean-room SVG icons. That conversion doubly earns its place: it's a
real text-glyph→real-icon upgrade, AND it establishes the reusable **pure-seam pattern** (`*_indicator
/*_glyph/*_icon` fn returns an `Icon`, unit-pinned cov/MSI 100; the render swaps a text child for
`svg().text_color()`) that every follow-up conversion (`file_icon`, `pane_icon`, the overlay headers,
the agent/forge statuses) will copy. The audit's other buckets are scoped into named follow-up
tickets rather than jammed into this slice.

## Scope
### In
- **block_status.rs (pure):** `status_indicator(kind, colors)` returns `(Icon, Hsla)` instead of
  `(&'static str, Hsla)` — the DECISION (which icon + which color per status) stays pure + pinned.
- **icons.rs (pure):** extend the `Icon` enum + `icon_path` with 3 status variants
  (StatusRunning ○ / StatusOk ✓ / StatusFailed ✗); each maps to a distinct SVG asset.
- **3 new clean-room SVG assets** (`assets/icons/`): `dot.svg` (running), `check.svg`, `cross.svg` —
  original 24×24 monochrome single-fill silhouettes (NO evenodd holes, matching the #137 set's
  technique — even the robot's eyes are filled subpaths); `NOTES.md` updated.
- **app.rs (shim):** the two block-status render sites (the real header 4842 + the sticky header
  5076) render `svg().path(icon_path(icon)).size(px(STATUS_ICON_SIZE)).text_color(gcolor)` (matching
  the proven top-tabs svg-in-`flex_row gap_2` pattern), and each header row gains `.items_center()`
  so the icon box centers against the 13px Command text; `Assets::load` gains 3 `include_bytes!`
  arms; a `STATUS_ICON_SIZE` const is added by the top-bar `ICON_SIZE`.
- **Deliverable — emoji→icon AUDIT** (`docs/marley_architecture/icon-audit.md`): every rendered
  glyph in the app, where it renders, and its disposition (already-SVG #137 / converted #232 /
  keep-as-UI-primitive / follow-up).
- **Deliverable — BRAND-ICON list** (in the same doc): the standard action set (vendorable from a
  permissive pack or hand-authored — no artist) vs. the brand/identity set an artist should make
  (app icon, agent identity mark, workspace glyph).

### Out (explicitly deferred — the audit scopes these into named follow-up tickets)
The audit found the full emoji/glyph surface. All of the below is CATALOGUED in `icon-audit.md`
with a recommended follow-up grouping; none is converted here (each is its own coherent slice):
- **FOLLOW-UP A — overlay/search header emoji (bucket D):** 🔍 (5 search surfaces), 🧠 (agent
  launcher), 📄 (file finder), 🕐 (history), 🔨 (forge), 🛰 (fleet), 🔀 (agent-diff). Each is baked
  into a `format!("<emoji> {…}", …)` header string → needs the emoji pulled out + an svg + label
  per site. The highest-visibility colorful-emoji group.
- **FOLLOW-UP B — file-type icons (bucket D):** `file_icon` (file_tree_view.rs) 🦀 .rs · ⚙ .toml ·
  📝 .md · 📄 generic — a real file-type ICON set (clean-room GENERIC glyphs, not language logos);
  in chad's Files panel (his feedback #2 focus).
- **FOLLOW-UP C — pane-type icons:** `pane_icon` (workspace.rs) 📁 FileTree · 📄 CodeView (bucket D)
  + ▸ Terminal · ⎇ Git (bucket B) — convert the fn's 4 arms together.
- **FOLLOW-UP D — block-header hover-affordance SET (bucket B):** ⧉ cmd (4783), ⧉ out (4804), ↻ run
  (4829) — one shared pattern (glyph+label, muted→accent self-hover, opacity reveal); convert
  together (needs a copy icon + a rerun icon + the svg-hover-tint mechanism: gpui svg paints from its
  OWN `style.text.color`, so a hover color change needs per-svg handling).
- **FOLLOW-UP E — status glyphs (bucket B):** `agent_status_glyph` ●○◔✓✗ (agent_view.rs) + Forge
  `status_glyph` ✓◐○• (forge_view.rs, `format!`-baked at app.rs:2818). Same pure-seam pattern as
  this ticket's status indicator.
- **KEEP (bucket C — not emoji):** disclosure/nav primitives ▸ ▾ ❯ › (fold, file-tree dirs,
  jump-to-bottom, prompt markers) + typographic punctuation · … — × (separators, close buttons).
- No new theme roles, no top-bar/cockpit changes (already SVG via #137).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — clean-room authored SVGs (§20).** New icons are original geometric silhouettes I author
  in the existing 24×24 monochrome-fill style (per `assets/icons/NOTES.md`), NOT lifted from any
  third-party set. gpui rasterizes each as an alpha mask + fills with `text_color`, so the shape is
  all that matters — they inherit the status color exactly as the text glyph did.
- **D2 — focused slice.** Convert the two most-visible PRODUCT-surface glyphs (block status + run);
  catalogue the rest in the audit; file a follow-up for the internal-overlay/file-type glyphs.
  (Matches the train's per-ticket discipline — #223/#226/#227 follow-ups.)
- **D3 — the pure seam is `status_indicator` returning `Icon`.** The classification stays a pure,
  unit-pinned decision; only the render mechanism changes (text child → svg child). Reuses the
  proven `svg().path(icon_path(..)).size(px(..)).text_color(..)` idiom (top bar app.rs:5986;
  top-tabs 6074, which is the SAME `flex_row gap_2` structure as the block header).
- **D4 — a dedicated block-status icon size.** The inline glyph sits with 13px Command-role text;
  design picks `STATUS_ICON_SIZE = 14.0` (by the Command text size, not the 16px top-bar ICON_SIZE)
  so the icon reads at the same visual weight; tunable at the capture vibe-check.
- **D5 — `.items_center()` on the two header rows.** A text glyph aligned to the command text's
  baseline; an svg is a fixed box that would top-align in a stretch row. Centering the row's cross
  axis is the canonical icon+label alignment (chevron + icon + command + affordances all center) —
  strong mechanism-verification, a minor/likely-improving vertical shift of the existing text, and
  confirmed at the driven capture. This is the one part that a pixel capture (not just the mechanism)
  should confirm → capture the status indicator at Validate (env-permitting).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE a command block is Running/Success/Failure, the block-status indicator shall render that status's dedicated SVG icon (not a text glyph), tinted with its status color. | app.rs render diff (svg child) + driven capture (env-permitting) / mechanism |
| REQ-002 | The pure `status_indicator(kind, colors)` shall return `(Icon, Hsla)` mapping each `StatusKind` to a DISTINCT `Icon` + its ThemeColors role (Running→border, Success→success, Failure→danger). | `block_status` unit test (exact `(Icon, color)` per kind), cov/MSI 100 |
| REQ-003 | The pure `icon_path` shall map each new `Icon` variant (StatusRunning/StatusOk/StatusFailed/Refresh) to its own distinct, existing SVG asset path. | extended `icon_path` unit test (per-variant pin + all-distinct), cov/MSI 100 |
| REQ-004 | The three new SVG assets shall exist under `assets/icons/`, be clean-room original 24×24 monochrome single-fill silhouettes, and be embedded by the `Assets` source (no runtime load). | file presence + `NOTES.md` review + gate build (assets embed) |
| REQ-005 | An emoji→icon AUDIT doc shall catalogue every rendered glyph with its location + disposition, and a BRAND-ICON list shall split the standard (no-artist) set from the brand (artist-needed) set. | doc review |

## Phase Plan
- **P2 Design** — pick the exact block-status icon size (D4); author the 4 SVG path silhouettes;
  design the `Icon` additions + the `status_indicator` signature change + the 2 render-site edits +
  the run-affordance edit; run `cargo mutants --list` for the real viable set; write the test matrix;
  spawn the audit (Explore) to catalogue every glyph.
- **P3 Implement** — the SVGs + `icons.rs` + `block_status.rs` (pure) then the app.rs shim edits;
  write the audit doc.
- **P3.5 Inspect** — independent critics vs the diff (render correctness/layout, clean-room, the
  pure-seam mutation story, audit completeness); fix the real findings.
- **P4 Validate** — write + RUN the pure tests; gate green (cov/MSI 100 on the pure seams); driven
  capture env-permitting (else mechanism-verify the svg-in-flex-row + offer the vibe-check).
- **P5 Complete** — CHANGELOG + app_shell/ui docs; AAR capture; file the follow-up ticket; close #232.
