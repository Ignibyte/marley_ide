# The cat mark: app icon, title bar and About — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-829-the-cat-mark.md
- **Pipeline spec:** 829-the-cat-mark.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #828 (Chad's go on the look tickets, 2026-10-20); his
  review pattern: each look ticket is installed and he restarts to see it.
- **Classification / tier:** feature, S. Assets plus two small additive Zed hunks.
- **Recall (§18.3):**
  - BF-claude-svg-asset-not-registered-renders-blank: a missing asset draws as a silent blank. It no
    longer applies to `images/`, which `assets` embeds by glob, but the scenario's shots must show
    the mark, not an empty 16 px box.
  - L-claude-815: a debug check cannot see what a release build embeds; read the installed icon
    after the install. The PNGs are `include_bytes!` and `build.rs` copies, so cargo tracks them.
  - L-claude-828: switch the theme live by writing the profile's `settings.json`.
  - The brain (`rusty-cli brain ask`, consultation bc60f3d71aba4c88aaf56688a661d8e2): nothing on
    this seam.
- **Discovery:**
  - Dev icon users: `crates/zed/build.rs:237` (window icon via `OUT_DIR/app_icon.png`),
    `crates/zed/src/zed.rs:1534` (About), `crates/zed/Cargo.toml:295` (bundle metadata),
    `script/install-marley:191,439` (macOS `.icns`, desktop entry icon). Windows `.ico` is out.
  - Title bar left group: `crates/title_bar/src/title_bar.rs` ~line 320, `children.push(h_flex()
    ...)` whose first child is the application menu.
  - About: `open_about_window`, window 440×300, the headline is `"{channel} {version}"` ("Marley
    0.x" on dev).
  - Wayland has no per-window icon in gpui; Hyprland matches the desktop entry by
    `StartupWMClass=com.ignibyte.Marley`.
- **Decisions:** D1 to D5 in the spec. Chad's drawing over the canvas's head is from the hand-off
  note (2026-10-20).

### Design
- **The SVG** (`assets/images/marley_mark.svg`, new asset): `viewBox="0 0 16 16"`; a `rect` with
  rounded corners (rx ≈ 3.5) in `#ff8c42`; Chad's paths in a `<g transform>` scaled to about 0.78 and
  centered, so the whiskers clear the tile's edge. Head path: `fill="#1c1714"`, its stroke
  `#1c1714`; whiskers stroke `#1c1714`; eyes `fill="#9be15d"`; nose fill and stroke and mouth
  stroke `#ff8c42`. If the eyes do not read at 16 px in the shots, grow their radius (the one
  allowed change to his geometry) and note it.
- **The PNGs** (`crates/zed/resources/app-icon-dev.png`, `@2x`): `rsvg-convert -w 512 -h 512
  assets/images/marley_mark.svg -o crates/zed/resources/app-icon-dev.png`, and `-w 1024 -h 1024`
  for `@2x`. Zed crate resources (binary), touchpoint row.
- **Title bar** (`crates/title_bar/src/title_bar.rs`, Zed crate): a `// Marley:` child,
  `img("images/marley_mark.svg").size(IconSize::Medium.rems()).flex_none()` (or `size_4`), first in
  the left-hand `h_flex`, before the menu. No new dependency. Its touchpoint row gets the clause
  added.
- **About** (`crates/zed/src/zed.rs`, Zed crate): after the `Headline`, `.when(release_channel ==
  Dev, |this| this.child(Label::new("Named for Marley: a solid orange cat with green
  eyes.").color(Color::Muted).size(LabelSize::Small)))`, the channel kept in a field; the window
  height grows from 300 if the shot shows the buttons pushed. Touchpoint row for `zed.rs` gets the
  clause added.
- **Install** (`script/install-marley`, Marley's): `install -Dm644 assets/images/marley_mark.svg
  "$prefix/share/icons/hicolor/scalable/apps/marley.svg"`; when
  `$prefix/share/icons/hicolor/icon-theme.cache` exists and `gtk-update-icon-cache` is on the path,
  run it with `-f -t`, so a stale cache does not keep the old Z. Its header comment (line 11-12, "Zed's
  dev-channel icon until #445") is updated.
- **File manifest:**
  | File | Owner | Change |
  |---|---|---|
  | `assets/images/marley_mark.svg` | Marley asset | new |
  | `crates/zed/resources/app-icon-dev.png`, `app-icon-dev@2x.png` | Zed crate | replaced |
  | `crates/title_bar/src/title_bar.rs` | Zed crate | the mark |
  | `crates/zed/src/zed.rs` | Zed crate | About's line |
  | `script/install-marley` | Marley | scalable icon, cache |
  | `docs/marley/zed-touchpoints.md` | Marley docs | rows |

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001 | none; Test reads both PNGs with the image reader | the PNGs themselves |
| REQ-002 | sway, a repo opened, theme Marley Night (default) | `829-01-title-night` |
| REQ-002 | the copy's `theme` written to "Marley Day" while Marley runs | `829-02-title-day` |
| REQ-003 | command palette, `zed: about`, Enter; the About window opens beside | `829-03-about` |
| REQ-004 | not reachable in a scenario: the install writes `~/.local`; read the installed files after the next install | Complete |
| REQ-005 | review of the diff | gate:16 |

Not reachable: the X11 window icon (the box runs Wayland; it reads the same PNG), the launcher's
icon (a desktop launcher under Chad's session, not the headless sway), and macOS (the Mac is
shelved).

### Risks
- A 16 px mark of a 16 px-designed drawing scaled to 0.78 makes the eyes about 1 px: the shots
  decide. Reversible by the radius change in the design.
- Binary PNGs in a Zed path conflict on an upstream merge only when Zed changes its dev icon;
  take ours.
- The title bar is crowded on a narrow window; 16 px plus a gap costs about 20 px.

## Phase 2 — Code
- **Built:** `assets/images/marley_mark.svg`; `app-icon-dev.png` (512) and `@2x` (1024) rendered
  from it with `rsvg-convert`; the title bar mark (`gpui::img`, `IconSize::Medium`, `mr_1`, first
  in the left-hand group); About's `is_marley` field and the line under the headline;
  `script/install-marley` installs `hicolor/scalable/apps/marley.svg` and runs
  `gtk-update-icon-cache -f -t` when the folder has a cache (the box's `~/.local` has none today);
  three new ledger rows and clauses on the `title_bar.rs` and `zed.rs` rows.
- **Deviation:** D1's colors flipped. The plan's dark head on a coat tile, rendered at 512, read as a
  black cat; Marley is orange. The mark is now a coat head and whiskers on a dark `#1c1714` tile,
  nose and mouth dark. The eyes grew from r 0.65 to 0.85 with a dark ring (the change the plan
  allowed) so the green shows at 16 px. The spec's D1 and REQ-001 say so.
- **Review of the diff:** both Zed hunks additive with `// Marley:` comments; no entity is
  touched; `img` of an embedded path is a glob-embedded asset (`images/**/*`), so no
  BF-svg-blank class. The About window's 300 px height is left for Test to judge.
- **Gate:** `script/gates.sh --diff` 17/17 (log `/mnt/fast/tmp/marley-gate-829.log`). The first run
  failed only its receipt: the e2e scenario was written during the run. `just gate-diff` waited on
  elephc's cargo (another project, own target dir), so the gate ran without `just idle`.

## Phase 3 — Test
- **Scenario:** `script/e2e/829-the-cat-mark.sh` (sway, 1600×1000), on the debug build of the
  Code tree; `script/e2e.sh` exit 0, three shots.
- `829-01-title-night`: the title bar in Marley Night. The mark, an orange cat with green eyes,
  sits first in the left-hand group, before the menu, `repo` and the branch. The dark tile
  almost merges with the bar, so the cat reads as the mark itself (REQ-002 met).
- `829-02-title-day`: Marley Day after the theme switch. The same place, with the dark tile now
  visible around the cat on the light bar (REQ-002 met).
- `829-03-about`: `zed: about`. The window shows the mark at 64 px, "Marley 1.26.0 (debug)",
  then "Named for Marley: a solid orange cat with green eyes." in muted small text, the commit and
  the version; OK and Copy still fit in 300 px, so no height change (REQ-003 met).
- REQ-001: `crates/zed/resources/app-icon-dev.png` read as an image: the orange head and whiskers
  on the dark rounded tile, green ringed eyes, dark nose and mouth; the 1024 file is the same
  SVG at twice the size.
- Not reachable: REQ-004 (the install writes `~/.local`; checked after the next install), the X11
  window icon (the box is Wayland; it reads the same PNG), the launcher's icon in Chad's session.
- Seen in passing, pre-existing — not in scope: the scenario's rail lists the box's own tmux
  session ("tessera", under Default), since #808's tmux rows read the user's tmux server, which the
  scenario's HOME does not isolate.

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added, #829); `docs/marley/look-plan.md` W0.2 done; the guide's theme
  section names the mark; `zed-touchpoints.md` rows checked against what shipped (the SVG, both
  PNGs, the title bar and About clauses).
- **Knowledge:** L-claude-829-a-mark-is-drawn-in-its-subjects-colors-and-seen-at-16-px-before-it-is-locked-001.
  No bug reached Code or Test, so no F block.
- **Brain:** decisions/marleys-mark-is-chads-cat-drawing-orange-on-a-dark-tile (follow-up
  2026-11-03).
- **Ticket** closed; pipeline archived; committed on `marley/workbench-shell`.
