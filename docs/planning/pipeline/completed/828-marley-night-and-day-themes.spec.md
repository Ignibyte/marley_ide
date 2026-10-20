---
pipeline_id: eb4d8a27-4fc3-46ce-807f-370a7e4ef90d
ticket: docs/planning/tickets/open/TICKET-828-marley-night-and-day-themes.md
status: Phase 4 — Complete PASS
title: Marley Night and Marley Day themes
type: feature
slice: The Marley look, W0.1 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
Marley's own theme family, Marley Night and Marley Day, drawn from the design canvas's Theme board
(her orange coat as the accent, her green eyes for success), and the default theme for Marley.
Every other board of the plan is drawn in these colors, so this comes first.

## Scope
### In
- `assets/themes/marley/marley.json`: one theme family, "Marley", with Marley Night (dark) and
  Marley Day (light): every UI color key Zed's theme schema has (the keys One Dark sets), the
  status colors, the players, the terminal's ANSI colors and the syntax colors.
- `assets/settings/default.json`: `theme.dark` = "Marley Night", `theme.light` = "Marley Day".
- The ledger row for `default.json`.

### Out (explicitly deferred)
- The cat mark and app icon (TICKET-829).
- Any layout change on any screen (TICKET-830 on).
- Changing the code's fallback theme constants (`DEFAULT_DARK_THEME` and `DEFAULT_LIGHT_THEME`):
  they stay One Dark and One Light, the theme a user gets if a configured theme fails to load.
- Omarchy's template (#759): following Omarchy still renders its own theme and overrides.

## Reference (§20)
Upstream Zed (the `theme` and `theme_settings` crates): a bundled theme is a theme family JSON
under `assets/themes/`, listed by `load_bundled_themes` (`theme_settings.rs`) through the `assets`
crate's `themes/**/*` embed, and selected by name with `theme.light` and `theme.dark`. Marley adds a
family the same way One, Ayu and Gruvbox are added; no Rust changes.

### Prior art
- The code we ship: `crates/assets` embeds `themes/**/*`; `theme_settings::load_bundled_themes`
  parses every `.json` there into the registry; `assets/themes/one/one.json` is the shape to follow
  (141 style keys, players, syntax). A crate we already build owns the whole seam: a theme is data.
- `component_preview` (`workspace::OpenComponentPreview`) draws GPUI's components; the visual check
  uses it to see the theme on every component.
- Local ledger: PR-claude-palette-accent-lightness-must-clear-wcag-vs-on-accent-001 (the light
  accent must carry white text at 4.5:1), PR-claude-selection-bg-distinct-from-container-001,
  F-claude-759-the-theme-template-drew-secondary-text-in-omarchys-border-shade-001 (secondary text
  is not a border shade).
- The design: the canvas's Theme, GpuiFoundations and MainDay boards.

## UI proof
Scenario `script/e2e/828-marley-night-and-day-themes.sh` (sway, one window): the profile copy's
`theme` key removed so `default.json` decides; the system's appearance is dark, as on the dev box.
- `828-01-night-terminal`: Marley in Marley Night, a terminal with a failed and a passed block, the
  rail, the tab bar, the bottom row (REQ-001, REQ-003).
- `828-02-night-editor`: an editor with Rust code and a git change, in Marley Night (REQ-003).
- `828-03-night-components`: `workspace: open component preview` in Marley Night (REQ-004).
- `828-04-day-terminal`, `828-05-day-components`: the profile's `theme` set to "Marley Day";
  the same views in Marley Day (REQ-002, REQ-004).
- `828-06-selector`: `theme selector: toggle` lists Marley Night and Marley Day (REQ-005).

## Locked-In Decisions
- D1 — One family file, `assets/themes/marley/marley.json`, built from One Dark's key set so no key
  falls back to Zed's defaults.
- D2 — Night tokens from the Theme board: ground `#0f0c0a`, panel `#15110e`, raised `#1c1714`, line
  `#2a221d`, text `#f3ece6`, muted `#b3a69a`, placeholder `#968878`; accent `#ff8c42`, its text
  `#1a0e05`; success `#9be15d`, warning `#f6c453`, conflict and needs-you `#ff6f91`, error `#ff6b5a`.
- D3 — Day tokens: ground `#faf5ef`, panel `#f3ebe2`, raised `#ffffff`, line `#e6dacd`, text
  `#2a1f18`, muted `#6e5d4f`, placeholder `#7a6655`; accent `#b4500e` (white on it 5.1:1; the
  board's `#d9661a` gives 3.6:1 and fails), success `#3f7d22`, warning `#8a5d00`, conflict `#c2185b`,
  error `#c62828`.
- D4 — Marley Day is the default whenever the system is light (the plan's open decision 5, taken by
  default).
- D5 — The fallback constants stay One Dark and One Light.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN no user setting names a theme and the system is dark, Marley shall draw in Marley Night. | `828-01-night-terminal` |
| REQ-002 | WHEN the user selects Marley Day, Marley shall draw in Marley Day, its accent carrying white text at 4.5:1 or more. | `828-04-day-terminal`; the contrast table in the notes |
| REQ-003 | Marley Night shall draw the rail, tabs, terminal blocks and editor in the Theme board's tokens: warm darks, coat-orange accent, eye-green success and added lines, readable secondary text. | `828-01`, `828-02` |
| REQ-004 | Both themes shall draw every component in the component preview with no unthemed (default) colors. | `828-03`, `828-05` |
| REQ-005 | The theme selector shall list Marley Night and Marley Day beside Zed's themes. | `828-06-selector` |
| REQ-006 | `default.json` shall name Marley Night and Marley Day, with a ledger row. | The review of the diff; gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley.json` (generated from the token tables, every One Dark key), `default.json`,
  the ledger row; `script/gates.sh --fast` green (no Rust).
- **P3 Test** — the visual check above; read every shot.
- **P4 Complete** — CHANGELOG and the guide (§21), ledger capture (§19), close the ticket, archive,
  commit.
