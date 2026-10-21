---
pipeline_id: c12465b4-fe88-4b4c-8770-2cb822639ae8
ticket: docs/planning/tickets/open/TICKET-829-the-cat-mark.md
status: Phase 4 — Complete PASS
title: The cat mark: app icon, title bar and About
type: feature
slice: The Marley look, W0.2 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, assets/icons/marley_cat.svg, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Marley's mark is Chad's cat, Marley: a solid orange cat with green eyes. Chad's own drawing of her
head (`assets/icons/marley_cat.svg`, #807) becomes the app icon, the mark at the left of the title
bar and the image in About, which also gets the line about her. Today all three show Zed's grey
dev-channel "Z".

## Scope
### In
- `assets/images/marley_mark.svg`: Chad's drawing in color on a rounded coat-orange tile.
- `crates/zed/resources/app-icon-dev.png` (512) and `app-icon-dev@2x.png` (1024), rendered from
  it. Marley ships only the dev channel (#760), and these two files feed the window icon
  (`build.rs`), About, the macOS bundle and the desktop entry's icon.
- `script/install-marley`: also installs the SVG as `hicolor/scalable/apps/marley.svg`, and updates
  the icon cache when the folder has one.
- `crates/title_bar`: the mark, 16 px, first in the left-hand group.
- `crates/zed/src/zed.rs` About: the line "Named for Marley: a solid orange cat with green eyes."
  under the headline, dev channel only.
- The `zed-touchpoints.md` rows for the Zed paths.

### Out (explicitly deferred)
- Nightly, Preview, Stable and Windows icons: Zed's channels, which Marley never builds.
- The line on a first-run screen (Marley has none yet), and the mark on Home.
- The title bar's "Talk to Marley" button, which keeps its one-color `IconName::MarleyCat`.
- Project marks and the shrinking rail (TICKET-830, TICKET-831).

## Reference (§20)
Upstream Zed: `crates/zed` already draws its app icon in three places from one file per channel
(`resources/app-icon{channel}.png`): the X11 window icon (`build.rs` → `APP_ICON`), the About
window (`open_about_window`, `about_window_icon`) and the bundle. Marley keeps that path and
replaces the dev file. Zed's `title_bar` draws no app mark; the left-hand mark follows the design
canvas's Main board (a 20 px cat head before the project name), drawn here at the title bar's
16 px icon size.

### Prior art
- The code we ship: `gpui::img` renders an embedded SVG in full color (`ImageSource::Resource`,
  `ImageFormat::Svg`), while `IconName` icons are one-color masks. `assets` embeds `images/**/*`,
  so a new image needs no new match arm (unlike BF-claude-svg-asset-not-registered-renders-blank,
  which was the old per-file `Assets::load`). `rsvg-convert` is on the box for the PNGs.
- `crates/zed/build.rs` (`prepare_app_icon_x11`) copies the channel's PNG to `OUT_DIR`; About
  embeds it with `include_bytes!`, so a release build carries a new PNG without a watch.
- `script/install-marley` installs `app-icon-dev.png` as the desktop entry's `marley` icon and
  builds the macOS `.icns` from `app-icon-dev@2x.png`.
- Chad's drawing (#807) and the canvas Theme board's app-icon variant (coat tile, dark head, green
  eyes); the hand-off note says to use his drawing, not the canvas's head.
- Ledger: L-claude-815 (check the installed build after the install), L-claude-828 (switch the
  theme live in a scenario). The brain had nothing on this seam.

## UI proof
Scenario `script/e2e/829-the-cat-mark.sh` (sway, one window, a project open):
- `829-01-title-night`: the title bar in Marley Night, the mark left of the project name
  (REQ-002).
- `829-02-title-day`: the profile's `theme` switched to "Marley Day" while Marley runs; the same
  title bar (REQ-002).
- `829-03-about`: `zed: about` from the command palette; the About window with the mark and the
  line (REQ-003).
- The icon files themselves (REQ-001) are read as images in Test, and the installed icon after the
  next install (REQ-004).

## Locked-In Decisions
- D1 — The mark is Chad's drawing: its paths kept, colored, scaled into a tile. The head and
  whiskers are coat `#ff8c42` on a raised-dark `#1c1714` tile with rounded corners; the eyes are
  eye-green `#9be15d` ringed in dark, their radius grown from 0.65 to 0.85 so the green shows at
  16 px; the nose and mouth are dark. The tile carries its own ground, so one file reads on Marley
  Night, Marley Day and any desktop. (Code changed this from the plan's dark head on a coat tile,
  which read as a black cat.)
- D2 — One SVG, `assets/images/marley_mark.svg`, is the source of the PNGs, the title bar mark and
  the installed scalable icon. The PNGs are rendered with `rsvg-convert` and committed; the command
  goes in the notes and the touchpoint row.
- D3 — Only the dev channel's PNGs change.
- D4 — The title bar mark is always shown, has no click and no tooltip, and sits before the
  application menu.
- D5 — About's line is the ticket's default (open decision 2) until Chad changes it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The dev app icon files shall show Chad's cat, orange with green eyes, on a dark tile, at 512 and 1024 px. | The two PNGs read as images in Test |
| REQ-002 | WHEN a Marley window opens, its title bar shall show the cat mark at the left of the project name, in Marley Night and in Marley Day. | `829-01-title-night`, `829-02-title-day` |
| REQ-003 | WHEN the user runs `zed: about`, the About window shall show the cat mark and the line "Named for Marley: a solid orange cat with green eyes." | `829-03-about` |
| REQ-004 | WHEN `script/install-marley` runs, the desktop entry's `marley` icon shall be the cat at 512 px and as a scalable SVG. | The review of the diff; the installed files read after the next install |
| REQ-005 | Each Zed path changed shall have its row in `docs/marley/zed-touchpoints.md`. | The review of the diff; gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the SVG, the two PNGs, the title bar mark, About's line, the install lines, the
  touchpoint rows; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check above; read every shot and both PNGs.
- **P4 Complete** — CHANGELOG and the guide (§21), ledger capture (§19), close the ticket,
  archive, commit; after the install, read the installed icon (L-claude-815).
