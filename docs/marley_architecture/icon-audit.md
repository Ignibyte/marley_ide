# Marley icon audit + brand-icon list (M13 #232)

**Purpose.** chad feedback #1: "remove all emojis for icons" + "a list of icons an artist should
make." This is the full inventory of every glyph Marley RENDERS (comment-only emoji excluded),
bucketed by what it is, plus the plan to migrate the real emoji to SVG icons and the split between
the standard set (we can vendor/author ourselves) and the brand set (worth a real artist).

**Context.** The `svg()` icon system (M8 #137, `src/icons.rs` + `assets/icons/*.svg`) replaced the
top-bar + cockpit emoji with clean-room SVG icons that respect `text_color`. But #137 only ever
converted **6 glyphs** — the audit below shows the emoji migration is far from finished.

> `app.rs` line numbers are accurate as of the #232 diff but drift with later edits — grep the
> glyph/escape to re-locate. Refs in other files are stable. *(2026-08-09, the scrap-forge rip —
> #410/#411: the **Forge** rows below (forge.svg, the ⌘⇧F overlay header, `forge_view.rs
> status_glyph`, the optional Forge brand mark) are retired history; those surfaces are deleted.)*

## Buckets

- **A — already an SVG icon (#137):** rendered via `svg().path(icon_path(..))`. 6 total.
- **B — monochrome symbol rendered as text:** a dingbat/arrow that DOES tint via `text_color`
  (○ ✓ ✗ ● ◔ ◐ • ↻ ⧉ ⋮ × ⇄ ⎇ → …). Not "colorful emoji", but not a real icon either.
- **C — disclosure / navigation primitive (KEEP):** ▸ ▾ ❯ › and typographic punctuation · … — ×.
  Standard UI/text, not emoji; leave as-is.
- **D — colorful pictographic EMOJI (the real target):** renders in its own colors, ignores
  `text_color` — the exact problem #137 named. **11 distinct emoji still rendered at audit time —
  ALL RETIRED by M16 #261** (groups D1/D2/D3 below → the SVG set; see the #261 pipeline docs).

## Bucket A — done (#137)

| icon | asset | where |
|---|---|---|
| folder | files.svg | Left "Files" dock toggle (top bar) |
| plus | plus.svg | New-terminal (top bar) |
| sparkle | sparkle.svg | New-agent (top bar) |
| list | details.svg | Details cockpit tab |
| robot | agents.svg | Agents cockpit tab |
| wrench | forge.svg | Forge cockpit tab |

## Bucket D — colorful emoji STILL RENDERED (the migration surface)

Grouped into the three subsystems #137 never touched:

**D1 — overlay / search header prefixes** (each baked into `format!("<emoji> {…}", …)`):

| emoji | U+ | surface | site |
|---|---|---|---|
| 🔍 | 1F50D | session filter, command palette, find bar, top global search (**5 surfaces**) | app.rs:4030/4032, 5368, 5884/5886/5892, 6171/6173 |
| 🧠 | 1F9E0 | agent-launcher header | app.rs:5468 |
| 📄 | 1F4C4 | file-finder (⌘P) header | app.rs:5515 |
| 🕐 | 1F550 | history-search (⌘R) header | app.rs:5556 |
| 🔨 | 1F528 | forge overlay header | app.rs:5591 |
| 🛰 | 1F6F0 | fleet overlay header/empty | app.rs:5621/5623 |
| 🔀 | 1F500 | agent-diff overlay header | app.rs:5845 |

**D2 — file-type icons** (`file_icon`, file_tree_view.rs:8; rendered app.rs:2478 — the Files panel):

| emoji | ext | site |
|---|---|---|
| 🦀 | .rs | file_tree_view.rs:11 |
| ⚙ | .toml | file_tree_view.rs:12 |
| 📝 | .md | file_tree_view.rs:14 |
| 📄 | generic | file_tree_view.rs:15 |

**D3 — pane-type icons** (`pane_icon`, workspace.rs:238-241; rendered app.rs:5258 — pane titles):

| emoji | pane | site |
|---|---|---|
| 📁 | FileTree | workspace.rs:239 |
| 📄 | CodeView | workspace.rs:240 |
| (▸ Terminal, ⎇ Git are bucket B in the same fn — convert the fn's 4 arms together) | | workspace.rs:238/241 |

## Bucket B — monochrome symbols (tint, but not real icons)

Convert opportunistically, grouped by their source fn (each a clean pure-seam like #232's status
indicator). NOT colorful emoji, so lower priority than bucket D.

| glyphs | source | render | note |
|---|---|---|---|
| ○ ✓ ✗ | `block_status.rs status_indicator` | app.rs:4854/5097 | **CONVERTED in #232** (the pattern template) |
| ⧉ ⧉ ↻ | inline (block header) | app.rs:4792/4813/4838 | the copy/rerun affordance SET → follow-up D |
| ● ○ ◔ ✓ (✗) | `agent_view.rs` `agent_status_glyph` (●○◔✓) + `agent_rows` (✗, non-zero exit) | app.rs:4203 + rows | agent status → follow-up E |
| ✓ ◐ ○ • | `forge_view.rs status_glyph` | app.rs:2825 (format-baked) | forge ticket status → follow-up E |
| × ⋮ ● ▏ ⇄ ⎇ → | inline (close btns, pane menu, badges, carets, SCM, remote) | various | low-value; keep or convert late |
| · … — − ⌘ ⇧ | separators / ellipses / kbd | various | typographic — KEEP (bucket C-adjacent) |

## Bucket C — KEEP (disclosure/nav primitives + punctuation)

▸ ▾ (fold toggle app.rs:4739; file-tree dirs 2455/2457) · ❯ (prompt marker 5017) · › (launcher prompt
5486) · ▾ (jump-to-bottom 5148) · and punctuation · … — ×. These are standard UI/text glyphs that
read correctly and tint fine; converting them to SVGs would be churn without benefit.

## #232 conversion (this ticket)

Converts **bucket-B status indicator ○ ✓ ✗** (`block_status.rs`) to 3 clean-room SVG icons
(`dot.svg` / `check.svg` / `cross.svg`), establishing the reusable **pure-seam pattern** every
follow-up copies: a `*_glyph/*_icon/*_indicator` fn returns an `Icon` (unit-pinned, cov/MSI 100); the
render swaps a text child for `svg().path(icon_path(icon)).size(px(..)).text_color(color)`. Chosen as
the template because it's always-visible (every block header) + low-risk + has the cleanest pure seam.

## Follow-up tickets (recommended — from the audit)

| id | scope | icons needed | status |
|---|---|---|---|
| A | D1 overlay/search headers (🔍🧠📄🕐🔨🛰🔀) | search, agent(→sparkle), file, clock, hammer(→forge.svg reuse), fleet/grid, diff/arrows | **DONE — M16 #261** (`icon_label` + search/file/clock/grid/diff.svg) |
| B | D2 file-type icons (`file_icon` 🦀⚙📝📄) | code-file, gear, document, file (GENERIC — not language logos, §20) | **DONE — M16 #261** (Icon-returning seam; .json joined the code class) |
| C | D3 pane-type icons (`pane_icon` 📁📄▸⎇) | folder(→files.svg reuse), file, terminal, git-branch | **DONE — M16 #261** (all 4 arms) |
| D | block-header affordance set (⧉ cmd/out, ↻ run) | copy (two-squares), rerun (circular-arrow / play) | open |
| E | status glyphs (`agent_status_glyph`, forge `status_glyph`) | working/idle/waiting/exited dots, ◐ partial, • | open |

## Brand-icon list (chad's "icons an artist should make")

The good news from the audit: **almost everything is a STANDARD UI glyph** — hand-authorable
clean-room (like the existing 6 + #232's 3) or vendorable from a permissive pack (Lucide/Feather,
MIT). **No artist needed** for: folder, file, plus, search/magnifier, gear, document, code-file,
clock, hammer, grid/fleet, diff-arrows, terminal, git-branch, copy, rerun, check, cross, dot, close,
chevrons.

**Artist-needed (brand / identity — a small, high-value list):**
1. **App icon** — the Marley dock/window/installer mark. The single most important brand asset; an
   original identity glyph, not a utility icon.
2. **Agent identity mark** — the "agent" concept (today 🧠 in the launcher / a generic sparkle for
   new-agent). A distinctive Marley agent glyph would carry identity where a generic sparkle doesn't.
3. **Workspace mark** — for the M13 workspace-centric direction (the launcher, the workspace rail /
   focused-workspace indicator). A recognizable Marley "workspace" glyph.
4. *(optional)* a **Forge** brand mark, if Forge is surfaced as a product identity (else the generic
   wrench is fine).

Everything else is utility iconography we own. Recommendation: commission the artist for #1-3 (the
identity set) and author/vendor the rest through the #137 system as the follow-up tickets land.

## See also
- `crates/marley_app/src/icons.rs` — the `Icon` enum + `icon_path` (the #137 + #232 system).
- `crates/marley_app/assets/icons/NOTES.md` — the clean-room SVG provenance.
- `app_shell.md` — the top-bar / cockpit / block-header render.
