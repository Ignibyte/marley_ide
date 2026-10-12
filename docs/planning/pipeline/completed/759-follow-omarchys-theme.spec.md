---
pipeline_id: b5d2ed0d-3532-4f4a-a7f4-677fa9203da9
ticket: docs/planning/tickets/closed/TICKET-759-follow-omarchys-theme.md
status: Phase 4 — Complete PASS
title: Follow Omarchy's theme
type: feature
slice: the finish line (docs/planning/design-notes/finish-line-2026-10-11.md), item 9, follow Omarchy's theme
references:
  - docs/t3code_architecture/README.md
  - docs/t3code_architecture/02-threads-and-composer.md
  - docs/planning/pipeline/completed/725-marleys-agents-get-rustal-ste.spec.md
  - docs/planning/pipeline/completed/709-load-the-shared-claude-code-plugin.spec.md
  - docs/planning/pipeline/completed/612-settings-edit-after-marley-writes.spec.md
  - docs/planning/pipeline/completed/668-test-runs-act-on-nothing-of-the-users.spec.md
---

## Title
Marley follows the desktop's theme on Omarchy, behind a setting that is off by default. Chad,
2026-10-11: "if its just here is an omarchy theme then maybe", then "lets go with 9 as well".
Omarchy renders a Zed theme from Marley's template at each theme switch, a hook copies it into
Marley's themes folder, and Zed's theme watcher reloads it.

## Scope
### In
- **The setting:** `marley.omarchy_theme` (bool, default `false`), in `default.json` with its
  comment, and a Follow Omarchy's Theme toggle in the Marley page's Layout section.
- **Omarchy present:** the current theme folder (`<home>/.local/state/omarchy/current/theme`) holds
  `colors.toml` and `<home>/.config/omarchy` is a folder, where `<home>` is `MARLEY_OMARCHY_HOME`,
  default `$HOME`. Absent, the setting does nothing: no file written, no program run, no theme
  change, one log line.
- **Turned on with Omarchy present:**
  - `marley.json.tpl` written to `<home>/.config/omarchy/themed/`: a Zed theme family "Omarchy" with
    one theme "Omarchy", built from Omarchy's colour slots. Its first line is a `//` marker comment.
  - A hook, `marley-theme`, written to `<home>/.config/omarchy/hooks/theme-set.d/`. Omarchy runs it
    after each theme switch. It copies `<theme folder>/marley.json` to `<Marley's themes
    folder>/omarchy.json` through a temporary file and a rename, and skips a render that still holds
    `{{`, so the last good copy stays. The two paths are written into the hook, shell-quoted.
  - When the current theme folder has no `marley.json` yet, Marley runs `omarchy-theme-refresh` once
    (`MARLEY_OMARCHY_REFRESH` overrides the program), so Omarchy renders the template and runs the
    hook now rather than at the next switch.
  - Once `omarchy.json` is in the themes folder, Marley loads it into the theme registry and writes
    `"theme": "Omarchy"` to the user's settings. The selection it replaced goes into Marley's
    key-value store.
- **Omarchy switches theme while it is on:** the hook's copy lands in the themes folder; Zed's
  `watch_themes` loads it and redraws. Marley does nothing itself.
- **Turned off:** while the user's `theme` still names "Omarchy", the kept selection goes back (or
  the key is removed when there was none). The hook and `omarchy.json` are removed, and the template
  too while its first line is still Marley's marker. A theme the user picked while it was on stays.
- **At start with it on:** a missing template (or one still carrying the marker) or hook is written
  again, and a render newer than the copy is copied.
- **Errors reach the UI:** a failed refresh or a failed write shows a toast naming the failure.
- **The e2e runner:** every run's copied settings turn `marley.omarchy_theme` off, and the runner
  exports `MARLEY_OMARCHY_HOME` and `MARLEY_OMARCHY_REFRESH` pointing into the run's folder, so no
  scenario reaches the user's Omarchy. Only #759's scenario turns it on, with a fake Omarchy home.
- `docs/marley/guide.md`: the setting, what it writes where, and how to keep an edited template.

### Out (explicitly deferred)
- macOS and Windows: Zed already follows the system's light and dark there (`theme.mode: system`).
  Nothing to build.
- Fonts: Omarchy's `font-set` hook could set Marley's UI and buffer fonts. Not asked for.
- Picking or editing Omarchy themes from inside Marley.
- An icon theme from Omarchy.
- Rendering `colors.toml` in Marley without Omarchy's renderer (D3).

## Reference (§20)
Upstream Zed's user themes: a theme family JSON file in the themes folder
(`docs/src/themes.md:116`) is loaded at start and reloaded when the file changes (`watch_themes`),
and `theme` in settings picks it. Behavior kept: Zed's themes folder, its watcher, its theme schema
(v0.2.0) and its theme selector, unchanged. Omarchy's own app hooks (`omarchy-theme-set-t3code`,
`-claude`, `-vscode`, `-obsidian`) are the model for the Omarchy side: a template rendered at each
theme switch, copied atomically into the app's watched themes folder under a stable name.

### Prior art
- **The code we ship:**
  - `crates/zed/src/main.rs`: `load_user_themes_in_background` (1918) reads every file in
    `paths::themes_dir()` at start; `watch_themes` (1970) loads any file that changes there and calls
    `theme_settings::reload_theme`. A copied `omarchy.json` is therefore enough to retint a running
    Marley.
  - `crates/paths/src/paths.rs:373`: `themes_dir()` is `config_dir()/themes`, under the run's
    profile when `--user-data-dir` is given, as `script/e2e.sh` does.
  - `crates/fs/src/fs_watcher.rs:63-82`: the watcher follows a symlink only when the watched path
    itself is one. A link inside the themes folder pointing at Omarchy's render is never watched,
    and Omarchy replaces the whole current theme folder at each switch. A link would not retint;
    the hook copies (D2).
  - `crates/theme_settings/src/theme_settings.rs`: `load_user_theme` (244) parses with
    `serde_json_lenient`, which takes `//` comments (the marker line, D5); `configured_theme` (164)
    falls back to the default theme while the named one is missing, so Marley loads the file before
    it writes the setting (D4); `refine_theme` (294) starts from Zed's light or dark base, so keys the
    template leaves out keep sane values; each colour parses on its own (`try_parse_color(..).ok()`,
    339), so one bad value drops that key, not the theme.
  - `crates/theme/src/registry.rs:174`: `insert_themes` replaces a theme by name, so each reload of
    "Omarchy" replaces the last.
  - `crates/theme_settings/src/settings.rs:262` `set_theme` and
    `crates/theme_selector/src/theme_selector.rs:407`: how Zed writes `theme` through
    `update_settings_file`.
  - `crates/marley_workbench/src/shared_plugin.rs`: `init` (118) plus a `reconcile` (130) on every
    `SettingsStore` change, a switch read from the merged settings (`SharedPlugin::of`, 103), and a
    re-check after the background write (146). #725's Codex copy: Marley owns a file outside its folders by
    a marker, writes it only while the marker is there, and removes it on off only then.
  - `crates/marley_workbench/src/process.rs:48` `run_program`: runs the refresh with no new spawn
    call, so gate:22's pin holds.
  - `crates/marley_workbench/src/groups.rs:140`: `KeyValueStore::global(cx)`, where the replaced
    theme selection is kept.
  - `crates/settings_ui/src/marley_page.rs:59` `layout_section`: where the toggle goes.
- **The behavior maps:** `docs/t3code_architecture/README.md` item 7; report 02, §2.20 and §3
  item 1 (T3's watched themes folder, Omarchy's `t3code.json.tpl` and its hook), and open
  question 1, whose default ("an opt-in setting, off, so a chosen Zed theme stays") Chad took.
- **Published material:**
  - Omarchy 4.0.4 on this box (MIT, read, never edited): `omarchy-theme-set` (renders templates into
    `next-theme`, swaps it in with `rm -rf` and `mv`, runs the app hooks, then `omarchy-hook
    theme-set <slug>`), `omarchy-theme-set-templates` (user templates in `~/.config/omarchy/themed`
    before the built-in ones; `{{ key }}`, `{{ key_strip }}`, `{{ key_rgb }}`, `{{ mix a b N% }}`;
    a file the theme already ships is not rendered over), `omarchy-theme-color` (the slot cascade:
    every name a template may use), `omarchy-hook` (runs each file in `theme-set.d` but `*.sample`
    with `bash`), `omarchy-theme-refresh`, and the apps' own hooks. Its agent docs
    (`default/agents/skills/omarchy/theming.md`, `hooks.md`) name `themed/<config-name>.tpl` and
    `theme-set.d` as the user's places for both.
  - Aether (omacom/aether, MIT), installed here, writes `aether.zed.json`, a Zed theme of 132 style
    keys, into the themes it generates: a second reference for mapping an Omarchy palette onto Zed's
    keys. Marley's template is its own; nothing is copied from Aether's output.
  - Zed's theme docs (`docs/src/themes.md`) and schema v0.2.0.

## UI proof
`script/e2e/759-follow-omarchys-theme.sh`, under `compositor sway`, on a scratch repository, with
`theme` set to "Gruvbox Dark" in the run's copy. Omarchy is faked under `$E2E_WORK/omarchy-home`:
two stock palettes copied from `$OMARCHY_PATH/themes` (tokyo-night, dark; catppuccin-latte, light),
a `switch` script that does what a theme switch does there (stage the palette, render with Omarchy's
own `omarchy-theme-set-templates` under the fake `HOME`, swap the folder in, run the fake
`theme-set.d` hooks with `bash`), and a fake `omarchy-theme-refresh` that re-runs it for the current
palette and logs each call. The scenario flips the setting by writing the run's settings file, as
#612's does. A terminal in the project prints the 16 ANSI colours.

Shots:
- `759-01-setting`: the Settings window's Marley page with Follow Omarchy's Theme, off.
- `759-02-absent`: the setting on with the fake home empty: still Gruvbox Dark.
- `759-03-on`: the fake Omarchy on tokyo-night and the setting on again: Marley in tokyo-night's
  colours, the terminal's colours tokyo-night's.
- `759-04-switch`: the fake switch to catppuccin-latte: Marley light, in latte's colours, with no
  restart.
- `759-05-off`: the setting off: Gruvbox Dark again.

## Locked-In Decisions
- **D1 — One switch, Omarchy only, off by default.** `marley.omarchy_theme`, read from the merged
  settings like `SharedPlugin::of` (no new `MarleySettings` field, L-claude-669). Rejected: following
  whenever Omarchy is present (a theme picked in Zed would be overridden; Chad took report 02's
  opt-in default); a cross-platform "follow the system" feature (Zed's `theme.mode: system` is that
  already on macOS and Windows).
- **D2 — Omarchy renders, a hook copies.** Rejected: a symlink in the themes folder (the watcher
  never sees its target change, `fs_watcher.rs:63-82`, and Omarchy deletes and replaces the folder it
  points into at each switch); a Marley watcher on Omarchy's state folder (a second watcher for what
  Omarchy's hook mechanism already gives, and a hard-coded state layout read from inside Marley).
- **D3 — The first render is Omarchy's own `omarchy-theme-refresh`, run once on turning it on when
  no render is there.** It re-applies the current theme, so the terminals and Hyprland reload with
  the same colours. Rejected: waiting for the next theme switch (turning it on would show nothing
  until Chad changes theme); rendering `colors.toml` inside Marley (a second renderer to keep in step
  with Omarchy's slot cascade and its `mix`, the maintenance Chad said no to).
- **D4 — The setting names the theme.** Marley loads `omarchy.json` into the registry, then writes
  `"theme": "Omarchy"` (static), and keeps the replaced selection, static or by mode, in the
  key-value store; off restores it only while `theme` still names "Omarchy". Rejected: drawing the
  theme over the setting at runtime (Zed's settings observer and `watch_themes` call `reload_theme`
  and put the configured theme back on every change); restoring unconditionally (it would undo a
  theme Chad picked while the switch was on).
- **D5 — Marley owns what it writes, as in #725.** The template's first line is a marker comment:
  written while missing or marked, removed on off only while marked, so an edited template (marker
  deleted) survives. The hook and `omarchy.json` are always Marley's and go when it is off.
  Rejected: overwriting the template every time (loses the user's edits); leaving the hook behind
  when off (a copy would keep landing in Marley's themes folder).
- **D6 — Paths and program behind overrides.** `MARLEY_OMARCHY_HOME` (default `$HOME`; Omarchy's own
  scripts use `$HOME/.config/omarchy` and `$HOME/.local/state/omarchy`, not the XDG variables) and
  `MARLEY_OMARCHY_REFRESH` (default `omarchy-theme-refresh` on `PATH`), after `MARLEY_GIT` and
  `MARLEY_RH`. File IO goes through `*_in(home, themes_dir)` functions (§14).
- **D7 — Marley acts on transitions and at start, not on every settings change.** A global keeps the
  last state applied; a settings change that does not flip the switch costs a comparison. After the
  refresh, Marley checks the switch again before it selects the theme.
- **D8 — The template.** A family "Omarchy" with one theme "Omarchy", `appearance` from
  `{{ mode }}`; UI surfaces from `background`, its shades and `mix` steps toward `foreground`; text
  from `foreground`, `muted`, `dark_foreground`; borders and focus from `muted` and `accent`; status
  from `red`, `yellow`, `green`, `blue`; the terminal's 16 ANSI colours straight from `color0` to
  `color15`; syntax and players from the hues the way Omarchy's VS Code template assigns them
  (keyword `bright_magenta`, string `green`, function `blue`, type `yellow`, number `orange`,
  comment `muted`). Only `{{ key }}` and `{{ mix a b N% }}`, and two-digit alpha suffixes
  (`{{ accent }}40`), which every stock theme's six-digit colours take.
- **D9 — The e2e runner keeps every run away from the user's Omarchy** (L-claude-725): the copy's
  `marley.omarchy_theme` off, `MARLEY_OMARCHY_HOME=$E2E_WORK/omarchy-home` (empty, so Omarchy is
  absent) and `MARLEY_OMARCHY_REFRESH=$E2E_WORK/no-omarchy-refresh`. The fake refresh in #759's
  scenario never runs Omarchy's `omarchy-theme-set`, whose app hooks reload the user's real Hyprland
  and terminals.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
a check line of the scenario, the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Marley page's Layout section shall show a Follow Omarchy's Theme toggle, off by default. | Shot 759-01-setting |
| REQ-002 | WHEN the setting is turned on and Omarchy is absent, the system shall keep the current theme and write no file in the Omarchy home. | Shot 759-02-absent; checks: the fake home empty, the refresh log empty |
| REQ-003 | WHEN the setting is turned on and Omarchy is present, the system shall draw Marley, terminal colours included, in the current Omarchy theme's colours. | Shot 759-03-on; check: `theme` is "Omarchy" in the run's settings |
| REQ-004 | WHILE the setting is on and Omarchy is present, Omarchy's user templates folder shall hold `marley.json.tpl`, its `theme-set.d` shall hold `marley-theme`, and Marley's themes folder shall hold `omarchy.json`. | Checks after 759-03 |
| REQ-005 | WHEN Omarchy switches theme while the setting is on, the system shall redraw in the new theme's colours and appearance without a restart. | Shot 759-04-switch |
| REQ-006 | WHEN Omarchy renders `marley.json` with a `{{` placeholder left, the hook shall leave the previous `omarchy.json` unchanged. | Check: the scenario runs the hook on such a render and compares the copy; review of the hook |
| REQ-007 | WHEN the setting is turned off while `theme` names "Omarchy", the system shall restore the theme selected before it was turned on. | Shot 759-05-off; check: `theme` is "Gruvbox Dark" |
| REQ-008 | WHEN the setting is turned off, the system shall remove its hook and `omarchy.json`, and its template while the template's first line is Marley's marker. | Checks after 759-05; review |
| REQ-009 | WHEN Marley starts with the setting on and Omarchy present, the system shall write back a missing hook or marked template and copy a render newer than `omarchy.json`. | Review of the diff |
| REQ-010 | IF the refresh or a write fails, THEN the system shall show a toast naming the failure. | Review of the diff |
| REQ-011 | The e2e runner shall turn `marley.omarchy_theme` off and point `MARLEY_OMARCHY_HOME` and `MARLEY_OMARCHY_REFRESH` into the run's folder for every scenario. | Review of `script/e2e.sh`; check: the user's own `~/.config/omarchy/themed` and `hooks/theme-set.d` listings are the same before and after the run |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes, with every cited seam checked
  against the code again.
- **P2 Code** — a new `omarchy_theme.rs` in `marley_workbench` with `omarchy/marley.json.tpl` and
  `omarchy/theme-set-hook.sh` beside the crate's sources; its `init` from `marley_workbench.rs`;
  `process.rs`'s doc and `.config/spawn-sites.txt`'s comment naming `omarchy-theme-refresh`; the
  setting in `settings_content/src/marley.rs`, `default.json` and `settings_ui/src/marley_page.rs`
  with their three `zed-touchpoints.md` rows extended; `script/e2e.sh`'s copy and exports; the guide;
  a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
