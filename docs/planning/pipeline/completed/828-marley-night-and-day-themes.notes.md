# Marley Night and Marley Day themes — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-828-marley-night-and-day-themes.md
- **Pipeline spec:** 828-marley-night-and-day-themes.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-20: "lets file all the tickets now and lets start on this". The first
  item of the look plan: Marley's theme, named for his cat, a solid orange cat with green eyes.
- **Classification / tier:** feature; data only (a theme JSON and a default setting), no Rust.
- **Recall (§18.3):**
  - PR-claude-palette-accent-lightness-must-clear-wcag-vs-on-accent-001: the light accent must
    carry its text at 4.5:1. The board's Day accent `#d9661a` gives 3.6:1 with white; `#b4500e`
    gives 5.1:1, so Day uses it.
  - PR-claude-selection-bg-distinct-from-container-001: `element.selected` and the active tab must
    differ from the panel under them (`#2a1d14` on `#15110e`; Day `#f6e3d0` on `#f3ebe2`).
  - F-claude-759: secondary text must not be a border shade; muted `#b3a69a` on `#15110e` is 7.9:1.
  - Completed pipelines 759 (Omarchy's theme) and 316 (syntax themes): the template route and the
    syntax capture names; 759's scenario sets `theme` in the profile copy, the pattern used here.
  - Brain: nothing on this seam (the consultation returned unrelated follow-ups).
- **Discovery:** `crates/assets/src/assets.rs` embeds `themes/**/*`;
  `theme_settings::load_bundled_themes` loads each `.json`; `assets/themes/one/one.json` has 141
  style keys plus players and syntax; `default.json` lines 9–13 hold the theme selection;
  `theme_settings.rs:178` falls back to `default_theme(appearance)`, the One constants, when a
  configured theme is missing. A user `theme` object needs `light` and `dark` both (the enum's
  fields are not defaulted), so the scenario removes the key rather than setting only a mode.
- **Decisions:** D1–D5 in the spec.

### Contrast (WCAG, computed)

| Pair | Ratio |
|---|---|
| Night accent `#ff8c42` / its text `#1a0e05` | 8.19 |
| Night text `#f3ece6` / ground `#0f0c0a` | 16.66 |
| Night muted `#b3a69a` / panel `#15110e` | 7.90 |
| Night placeholder `#968878` / panel | 5.44 |
| Night success, warning, needs-you, error on panel | 11.91, 11.57, 7.09, 6.71 |
| Day accent `#b4500e` / white | 5.13 |
| Day text `#2a1f18` / ground `#faf5ef` | 14.81 |
| Day muted `#6e5d4f` / ground | 5.80 |
| Day placeholder `#7a6655` / ground | 5.02 |
| Day success, warning, conflict, error on ground | 4.65, 5.31, 5.42, 5.19 |

### Design
- **Approach:** a small generator (in the scratchpad, not committed) writes `marley.json` from two
  token tables, mapping every key One Dark sets to a token, so no key falls through to Zed's
  default colors. Roles:
  - grounds: `background`, `status_bar`, `title_bar` = panel; `editor.background`,
    `terminal.background`, `tab.active_background`, `toolbar` = ground; `surface`,
    `elevated_surface`, `panel`, `tab_bar`, inactive tabs = panel/raised.
  - elements: `element.*` and `ghost_element.*` hover/active/selected = the coat tint `#2a1d14`
    (Day `#f6e3d0`), never equal to the panel.
  - borders: `border` `#2a221d`, `border.variant` `#1e1813`, `border.focused` the accent,
    `border.selected` `#7a4420`.
  - text and icons: text, muted, placeholder, disabled, accent (`#ffb27a` Night, `#b4500e` Day).
  - status: success/created = eye green; warning/modified = honey; conflict = pink; error/deleted
    = red; info/renamed/hint = light orange; hidden/ignored/unreachable = faint.
  - version control: added = eye green, modified = honey, deleted = red.
  - players: the first is the accent; the rest from green, teal, purple, pink, honey, red, blue.
  - terminal ANSI: red `#ff6b5a`, green `#9be15d`, yellow `#f6c453`, blue `#7aa7ff`, magenta
    `#c4b5fd`, cyan `#7dd3c0` (Day darker pairs), with bright and dim steps.
  - syntax: keywords purple, functions light orange, types teal, strings eye green, numbers and
    constants honey, comments faint italic, properties pink-orange, punctuation muted; Day uses
    the darker pairs.
- **File manifest:**
  - `assets/themes/marley/marley.json` (new; asset, Marley-owned).
  - `assets/settings/default.json` (Zed's asset): `theme.light`, `theme.dark`, with a Marley
    comment.
  - `docs/marley/zed-touchpoints.md`: the row for `default.json`'s theme lines.
  - `script/e2e/828-marley-night-and-day-themes.sh` (Test).
- **Ledger rows:** `assets/settings/default.json` (the theme selection).

### Visual check plan

| REQ | The scenario | Shot |
|---|---|---|
| REQ-001 | Profile `theme` removed; system dark; open a repo, run a passing and a failing command | `828-01-night-terminal` |
| REQ-003 | Open `src/main.rs` with a change | `828-02-night-editor` |
| REQ-004 | `workspace: open component preview` | `828-03-night-components` |
| REQ-002 | Relaunch with `theme` = "Marley Day"; same terminal; component preview | `828-04-day-terminal`, `828-05-day-components` |
| REQ-005 | `theme selector: toggle`, type "marley" | `828-06-selector` |
| REQ-006 | — | the diff and gate:16 |

What no scenario reaches: the light default under a light system appearance (the sway session's
portal reports the dev box's dark preference); REQ-002 shows Day by name instead, and the
`default.json` value is read in the diff.

### Risks
- A key One Dark does not set falls back to Zed's default for its appearance; the generator copies
  One Dark's full key set, and the component preview shot shows any stray color.
- Chad's own settings name a theme (One Dark or Omarchy); his installed Marley keeps it until he
  picks Marley Night. The CHANGELOG says how.

## Phase 2 — Code
- **Built:** `assets/themes/marley/marley.json`, generated from the spec's token tables by a
  scratch script that walks every key One Dark sets (141 style keys, 8 players, 46 syntax tokens
  per theme) and stops on any key without a Marley value, so none falls back to Zed's defaults.
  `assets/settings/default.json`: `theme.light` Marley Day, `theme.dark` Marley Night, with a
  `Marley:` comment. Ledger: the `default.json` row extended and a new row for the theme file
  (gate:16 asked for it: `assets/` is outside the Marley-owned paths).
- **Deviations:** comments are italic in both themes (One's are not), as the boards draw them; a
  first-party theme gets no entry in `assets/themes/LICENSES`, which lists the third-party ones.
- **Review:** the theme's key set equals One Dark's; selected and hover fills (`#2a1d14`,
  `#221c17`) differ from the panel (`#15110e`); the Day accent is `#b4500e`, white on it 5.13:1;
  focused borders take the accent. No Rust changed, so no entity or provenance concern.
- **For Test and after:** the debug build reads `assets/` at runtime (L-claude-815), so the
  scenario sees the theme without a rebuild; the installed build embeds it, so Complete greps the
  release binary for `themes/marley/marley.json` after the next install.
- **Gate:** `script/gates.sh --fast` GREEN, 16 of 16.

## Phase 3 — Test
- **Scenario:** `script/e2e/828-marley-night-and-day-themes.sh` (sway). The profile copy's `theme`
  key is taken out so `default.json` decides; Day is set by name in the copy while Marley runs
  (Zed reloads its settings file). Run once, all six shots written, every one read.
- `828-01-night-terminal` (REQ-001, REQ-003): with no theme set and the box dark, Marley draws in
  Marley Night: ground `#0f0c0a`, panel `#15110e`, PROJECTS lit in coat orange, the selected rail
  row a warm tint distinct from the panel, the passing block's bar and badge eye green, the failed
  block red with its `exit 1` pill, the bottom row in the panel color.
- `828-02-night-editor` (REQ-003): Rust in the board's syntax colors (keywords purple, types teal,
  strings green, properties pink-orange, comments faint italic), the added lines' gutter bar eye
  green, the active line number light orange, muted text readable.
- `828-03-night-components` (REQ-004): the component preview's list, search field (focused, an
  orange border) and the Sandbox Status Tooltip's previews, all in Marley Night; no blue or grey
  default color anywhere in the frame.
- `828-05-day-components` (REQ-002, REQ-004): the same preview in Marley Day: cream ground, white
  raised cards, the orange-tinted selection, dark brown text.
- `828-04-day-terminal` (REQ-002): the terminal in Marley Day: the passing block green, the failed
  block's pale red fill and red pill, the selected rail row the coat tint.
- `828-06-selector` (REQ-005): the theme selector filtered on "marley" lists Marley Day (checked)
  and Marley Night.
- **Not reached:** the light default under a light system appearance; the sway session's portal
  reports the dev box's dark preference. Day is shown by name, and `default.json`'s light value is
  read in the diff. The preview shows one component page per shot, not all of them; the rest of
  the canvas boards land ticket by ticket.
- **Focus report:** keys went to the run's own window only; the user's Hyprland was untouched (one
  Marley window before and after).

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added); the guide's settings section (Marley's themes, how a saved theme
  wins and how to switch); the look plan marks W0.1 done; both ledger rows (`default.json`, the
  theme file) describe what shipped.
- **Knowledge:** L-claude-828-a-theme-check-takes-the-theme-key-out-and-switches-it-live-001. No
  bug was found in Code or Test, so no F- block.
- **Brain:** `brain_decide` on consultation 076e284c: decisions/marleys-default-theme-is-its-own-marley-night-and-marley-day
  (follow up 2026-11-20).
- **After the next install:** `grep -ao 'themes/marley/marley.json' ~/.local/lib/marley/marley`
  shows the release build carries the theme (L-claude-815).
