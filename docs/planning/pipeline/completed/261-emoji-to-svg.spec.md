---
pipeline_id: 58a0ca13-9a16-4ffb-9626-d409ab92b0a6
ticket: forge#261 (c8ab7eee-86ba-4213-90db-05d8f9e38f45) · local docs/planning/tickets/open/TICKET-261-emoji-to-svg.md
aar_id: f54937d4-737f-443b-837c-9c10d01562bf
status: Phase 5 — Complete PASS
title: Kill the remaining 11 colorful emoji → clean-room SVG icons (audit groups A–C)
type: chore
milestone: M16
references:
  - docs/marley_architecture/icon-audit.md
---

## Title
chad: "NO mo emojis." The #232 pass converted 6 glyphs and left 11 colorful
emoji rendering in their own colors (they ignore `text_color` — the exact
problem #137 named). Convert all three audit groups to the vendored tintable
SVG system: D1 overlay/search headers (🔍×5 surfaces, 🧠, 📄, 🕐, 🔨, 🛰, 🔀),
D2 `file_icon` (🦀 ⚙ 📝 📄), D3 `pane_icon` (📁 📄 + the ▸/⎇ bucket-B arms —
the fn's 4 arms convert together).

## Scope
### In
- **icons.rs**: +10 `Icon` variants — `Search, File, Clock, Fleet, Diff,
  CodeFile, Gear, Document, Terminal, GitBranch` — with `icon_path` arms +
  the distinctness/completeness tests extended.
- **10 new clean-room SVG assets** (self-authored, the existing minimal
  monochrome stroke style; §20-safe GENERIC glyphs — a code-file is angle
  brackets, NOT a language logo): search, file, clock, grid (fleet), diff,
  code-file, gear, document, terminal, git-branch. NOTES.md provenance rows.
- **file_tree_view::file_icon** → returns `Icon` (.rs→CodeFile, .toml→Gear,
  .md→Document, else File); tests rewritten to Icon asserts.
- **workspace::pane_icon** → returns `Icon` (FileTree→Files [reuse],
  CodeView→File, Terminal→Terminal, Git→GitBranch); tests rewritten.
- **app.rs shim**: the Files-panel row + pane-title renders swap the text
  glyph child for `svg().path(icon_path(..)).text_color(..)`; the 12 D1
  header sites drop the emoji from their `format!` strings and gain an svg
  icon child (launcher reuses NewAgent/sparkle; forge header reuses Forge;
  the rest use the new variants).
- Emoji-free proof: the 11 codepoints grep to zero in `crates/**/*.rs`.

### Out (explicitly deferred)
- Bucket B follow-ups D (block-header ⧉/↻ affordances) + E (agent/forge
  status glyph sets) — separate audit tickets.
- Bucket C disclosure/nav primitives (▸ ▾ ❯ › · … — ×) — KEEP per the audit
  (standard text glyphs that tint fine). NOTE: pane_icon's ▸/⎇ arms DO
  convert (in-scope as part of the fn).
- The ARTIST brand list (app icon, agent mark, workspace mark) — out of
  scope per the ticket.

## Reference (§20)
N/A — Marley's own icon set. The SVGs are clean-room self-authored generic
UI glyphs in Marley's existing #137 stroke style (no reference-app source or
asset is read/copied; "Lucide/Feather-style" names the visual genre only).
File-type icons are deliberately GENERIC (document/gear/code brackets), not
language logos, per the audit's §20 note.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — Reuse before vendoring: launcher 🧠→`NewAgent` (sparkle), forge
  🔨→`Forge` (wrench), FileTree 📁→`Files` (folder). Everything else gets a
  new variant + asset.
- D2 — `file_icon`/`pane_icon` change RETURN TYPE to `Icon` (the #232
  pure-seam idiom: the pure fn picks, the shim renders). Their tests pin the
  mapping.
- D3 — D1 header icon choices are shim-inline `Icon::` constants (static
  per-site, no logic → no pure seam needed; `icon_path` completeness tests +
  driven captures carry them).
- D4 — SVGs are self-authored minimal strokes (24×24 viewBox,
  `stroke="currentColor"`, fill-none — match the existing assets); each gets
  a NOTES.md provenance row.
- D5 — One emoji may remain in a COMMENT if any exists (comments don't
  render); the grep proof targets string literals — but prefer zero total.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `file_icon` shall map `.rs`→CodeFile, `.toml`→Gear, `.md`→Document, anything else→File (case-insensitive ext, as today). | unit tests (rewritten) |
| REQ-002 | `pane_icon` shall map FileTree→Files, CodeView→File, Terminal→Terminal, Git→GitBranch. | unit tests (rewritten) |
| REQ-003 | `icon_path` shall map every variant (19 total) to a distinct vendored asset path that `Assets::load` actually serves (non-empty bytes). | extended unit test + the `assets_serve_every_icon_variant` guard (inspect A1 — the loader is a static match, NOT a dir embed; a missed registration renders blank) |
| REQ-004 | WHEN the palette / file-finder / history / agent-launcher / forge / fleet / agent-diff / find bar / session filter / top search render, the header shall show a theme-tinted SVG icon and NO colorful emoji. | driven captures (≥4 overlays) + REQ-006 grep |
| REQ-005 (visual_acceptance) | The Files panel rows and pane title bars shall show tinted SVG type icons (no 🦀/⚙/📝/📄/📁). | driven captures |
| REQ-006 | The 11 emoji (🔍🧠📄🕐🔨🛰🔀🦀⚙📝📁) shall not appear in any `crates/**/*.rs` string literal. | grep (codepoint class) = zero |

## Phase Plan
- **P2 Design** — locate the 12 live header sites (lines drifted); author the
  10 SVG path specs; exact seam signatures; test plan.
- **P3 Implement** — assets + icons.rs + the two pure fns + shim swaps.
- **P3.5 Inspect** — critics (render regression, §20 asset provenance,
  completeness).
- **P4 Validate** — units + mutants trace; driven captures; gate --diff.
- **P5 Complete** — CHANGELOG, icon-audit.md status flip (groups A–C DONE),
  app_shell/ui docs touch, AAR, archive, close.
