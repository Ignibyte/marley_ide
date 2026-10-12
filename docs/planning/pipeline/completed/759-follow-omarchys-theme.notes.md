# Follow Omarchy's theme — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-759-follow-omarchys-theme.md
- **Pipeline spec:** 759-follow-omarchys-theme.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11, on the finish-line list
  (`docs/planning/design-notes/finish-line-2026-10-11.md`), item 9: "if this is easy to do and will
  be across mac, windows, and omarchy then lets do it. if not i dont want to maintain this. if its
  just here is an omarchy theme then maybe". Told it is Omarchy-only by nature (macOS and Windows
  already follow the system's light and dark), about a day, one template file and a setting off by
  default: "lets go with 9 as well". The survey's text: "a theme template in Omarchy's user
  templates folder, rendered into Marley's themes folder, which Zed's theme watcher reloads;
  opt-in, so a chosen Zed theme stays" (`docs/t3code_architecture/README.md`, item 7; report 02,
  §2.20, item 1, open question 1). Queued with the build-now items, before #445.
- **Classification:** feature; M. The code is one Marley module and three settings lines; the
  template's key mapping and the scenario's fake Omarchy are most of the work. One slice.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **Recall (§18.3):**
  - L-claude-725: a feature that writes outside Marley's data folder is off in the runner's copy,
    the runner points the outside place at the run's folder, and only the feature's own scenario
    turns it on. Here the outside place is the user's Omarchy, and a hook baked with an e2e themes
    path would silently stop Chad's real Marley from following his theme. D9.
  - #725 (`725-marleys-agents-get-rustal-ste.spec.md`): Marley owns a file in another tool's folder
    by a marker; writes it while missing or marked; off removes it only while marked. D5.
  - #709 (`shared_plugin.rs`): `init` + `reconcile` on `SettingsStore`, the switch from the merged
    settings, a re-check after background work. D7.
  - L-claude-669: a new on/off in `MarleySettings` must be a two-variant enum (clippy's bool and
    line limits). Reading the merged settings directly, as `SharedPlugin::of` does, avoids touching
    `MarleySettings`. D1.
  - L-claude-612: Zed's writers replace `settings.json` by rename; Zed watches the config folder, so
    a scenario's mid-run settings edit is seen. The scenario flips the switch that way.
  - L-claude-668: the runner copies only `settings.json` and `db`; the themes folder is not copied,
    so a run's themes folder starts empty under the run's profile.
  - L-claude-468: read a shot's pixels (`magick … %[pixel:p{x,y}]`) before trusting a colour. The
    Test phase samples the editor's background against tokyo-night's `#1a1b26` and latte's
    `#eff1f5`.
  - L-claude-650: a run ends without Marley's quit hooks; nothing here depends on quit.
  - Brain: no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **Discovery (2026-10-11; re-verify at promotion):**
  - **Zed's themes:** `crates/zed/src/main.rs` `load_user_themes_in_background` 1918–1966 (every file
    in the folder, any name), `watch_themes` 1970–2001 (100 ms batches; a changed file is loaded and
    `reload_theme` called); both started at 869–870. `crates/paths/src/paths.rs:373`
    `themes_dir()`. `crates/theme_settings/src/theme_settings.rs`: `init` 71 (its `SettingsStore`
    observer reloads when the theme name changes, 150–154), `configured_theme` 164 (falls back while
    the name is missing), `reload_theme` 204, `load_user_theme` 244, `deserialize_user_theme` 252
    (`serde_json_lenient`, comments allowed: `Deserializer::new` sets `allow_comments`),
    `refine_theme` 294 (light or dark base, each colour parsed alone, 339). `crates/theme/src/schema.rs:11`
    `AppearanceContent` is `light` or `dark`, as Omarchy's `{{ mode }}` gives.
    `crates/theme/src/registry.rs:174` `insert_themes` replaces by name.
    `crates/theme_settings/src/settings.rs:262` `set_theme`; `crates/theme_selector/src/theme_selector.rs:407`.
    `crates/settings_content/src/theme.rs:221` `theme: Option<ThemeSelection>`, 329 `ThemeSelection`
    (`Static` or `Dynamic { mode, light, dark }`).
  - **The watcher and links:** `crates/fs/src/fs_watcher.rs:36-82`: one watch on the folder, plus the
    target only when the watched path itself is a symlink. Native Linux watches are not recursive
    (30–33).
  - **Marley's side:** `crates/settings_content/src/marley.rs:9-12` `MarleySettingsContent`;
    `assets/settings/default.json:1701` the `marley` block; `crates/settings_ui/src/marley_page.rs:59`
    `layout_section` (six items; becomes seven); `crates/marley_workbench/src/shared_plugin.rs`
    103/118/130/146; `process.rs:48` `run_program`; `groups.rs:140` `KeyValueStore`;
    `marley_workbench.rs:885` where `shared_plugin::init` is called; toasts as in
    `claude_plugin.rs:238`; `include_str!` of files beside the sources, as `agent_notify.rs:28` and
    `fleet_hosts.rs:20` do. `.config/spawn-sites.txt` lists `process.rs` for "the workbench's
    programs"; gate:22's `SPAWN_SITES_PIN=7` counts spawn calls, which this does not add.
  - **The runner:** `script/e2e.sh` 57–58 (the user's config and data), 301–304 (`--user-data-dir`),
    562 `profile_setting`, 652–686 (the copy's scrub), 689–695 (the exports), 774–776 (`setup` runs
    after both, so a scenario's export wins).
  - **Omarchy on this box** (4.0.4, package `omarchy`, `$OMARCHY_PATH=/usr/share/omarchy`;
    `~/.local/share/omarchy` is a link to it): read only.
    - `bin/omarchy-theme-set`: stages `next-theme`, calls `omarchy-theme-set-templates`, then
      `rm -rf current/theme; mv next-theme current/theme`, writes `theme.name`, runs the app hooks in
      parallel (`omarchy-theme-set-t3code`, `-claude`, `-vscode`, …), then `omarchy-hook theme-set
      <slug>`. Headless runs skip both.
    - `bin/omarchy-theme-set-templates`: only when the theme has `colors.toml`; user templates
      (`~/.config/omarchy/themed/*.tpl`) first, then `$OMARCHY_PATH/default/themed/*.tpl`; a file the
      theme ships is not rendered over; tokens `{{ key }}`, `{{ key_strip }}`, `{{ key_rgb }}`,
      `{{ mix|mix_strip|mix_rgb a b N% }}` (both hex), gradient functions.
    - `bin/omarchy-theme-color --all` gives the slots: accent, background, bg, blue, bright_blue,
      bright_cyan, bright_fg, bright_foreground, bright_green, bright_magenta, bright_purple,
      bright_red, bright_yellow, brown, color0–color15, cursor, cyan, dark_background, dark_bg,
      dark_fg, dark_foreground, darker_background, darker_bg, fg, foreground, green, light_fg,
      light_foreground, lighter_background, lighter_bg, magenta, mode, muted, orange, purple, red,
      selection, selection_background, selection_foreground, theme_type, yellow. `accent` has no
      fallback; every stock theme defines it.
    - `bin/omarchy-hook`: runs `hooks/<name>` then each file in `hooks/<name>.d/` but `*.sample`,
      with `bash`, the slug as `$1`.
    - `bin/omarchy-theme-set-t3code` and `-claude`: the pattern the hook follows (skip a render with
      `{{`, `mktemp` beside the target, `cp`, `mv`).
    - `bin/omarchy-theme-refresh`: `OMARCHY_THEME_SKIP_BACKGROUND=1 omarchy-theme-set <current>`;
      `omarchy-restart-terminal` touches Alacritty's config and signals kitty and ghostty;
      `omarchy-restart-hyprctl` is `hyprctl reload`.
    - The current theme on this box ships `aether.zed.json` (Aether's Zed theme, 132 style keys,
      `players`, 48 syntax entries).

### Design (draft; the Plan phase confirms it at promotion)
- **New `crates/marley_workbench/src/omarchy_theme.rs`** (Marley crate):
  - `enabled(cx)`: `marley.omarchy_theme` from the merged settings.
  - `OmarchyPaths { home }` from `MARLEY_OMARCHY_HOME` or `$HOME`: `templates()`, `hooks()`,
    `current_theme()`; `present()` checks `current_theme()/colors.toml` and `.config/omarchy`.
  - `install_in(paths, themes_dir) -> io::Result<Installed>`: the template written while missing or
    marked, the hook written (paths shell-quoted, mode 755), and the render copied when it is newer
    than the copy; says whether a render was there.
  - `remove_in(paths, themes_dir)`: the hook, `omarchy.json`, and the template while marked.
  - `init(cx)`: a global with the last applied state; `reconcile` at start and on `SettingsStore`.
    Off→on: if present, `install_in` on the background; if no render, `process::run_program` on the
    refresh; then, if still wanted, load `omarchy.json` with `theme_settings::load_user_theme`, keep
    the user's current `theme` selection (JSON) in `KeyValueStore`, and write `"theme": "Omarchy"`
    through `settings::update_settings_file`. On→off: restore while `theme` is still "Omarchy",
    then `remove_in`. At start with it on: `install_in` only (no refresh, no selection write).
    Errors: a toast in each workspace, as `claude_plugin.rs:238` does, and `log_err`.
- **New `crates/marley_workbench/omarchy/marley.json.tpl`** (Marley crate): the family, D8's
  mapping. First line: `// Marley's Omarchy theme (marley.omarchy_theme). Marley rewrites this file
  while this line is here; delete the line to keep your edits.`
- **New `crates/marley_workbench/omarchy/theme-set-hook.sh`** (Marley crate): the hook, with two
  placeholders Marley fills (the render, the copy). A marker comment names Marley.
- **`marley_workbench.rs`:** `mod omarchy_theme;` and `omarchy_theme::init(cx)` beside
  `shared_plugin::init`.
- **`process.rs`** doc and **`.config/spawn-sites.txt`** comment: `omarchy-theme-refresh` among the
  workbench's programs (Marley-owned paths; the pin does not move).
- **`crates/settings_content/src/marley.rs`** (Zed crate path): `omarchy_theme: Option<bool>`, doc
  "Default: false". Ledger row 59 extended.
- **`assets/settings/default.json`** (Zed path): `"omarchy_theme": false` with its comment in the
  `marley` block. Ledger row 68 extended.
- **`crates/settings_ui/src/marley_page.rs`** (Zed crate path): Follow Omarchy's Theme in
  `layout_section` (`[SettingsPageItem; 7]`), description: "On Omarchy, follow the desktop's theme:
  Omarchy renders Marley's theme at each theme switch. Turning it on re-applies Omarchy's current
  theme once. Off puts back the theme chosen before. Nothing happens without Omarchy." Ledger row 63
  extended.
- **`script/e2e.sh`** (Marley path): `marley["omarchy_theme"] = False` in the scrub with its comment;
  `export MARLEY_OMARCHY_HOME=$E2E_WORK/omarchy-home` and
  `MARLEY_OMARCHY_REFRESH=$E2E_WORK/no-omarchy-refresh` beside `CODEX_HOME`.
- **`docs/marley/guide.md`:** under Settings, "Omarchy's theme (#759)" and the key in the jsonc
  block.
- **Ledger rows:** the three rows above (59, 63, 68) in `docs/marley/zed-touchpoints.md`, extended in
  the same change. No new Zed seam: `load_user_theme`, `update_settings_file` and `themes_dir` are
  public already.

### Visual check plan
| REQ | The scenario does | The shot or check |
|---|---|---|
| 001 | `marley: open settings`, searches "Omarchy" | 759-01-setting: the toggle, off |
| 002 | Fake home empty; `marley.omarchy_theme` true in the run's settings | 759-02-absent: Gruvbox Dark still; the fake home empty; the refresh log empty |
| 003 | Turns it off; builds the fake Omarchy on tokyo-night; turns it on | 759-03-on: tokyo-night's colours, the terminal's 16 colours tokyo-night's; `theme` "Omarchy"; one refresh logged |
| 004 | After 759-03 | `marley.json.tpl` in `themed/`, `marley-theme` in `theme-set.d/`, `omarchy.json` in the run's themes folder |
| 005 | Runs the fake switch to catppuccin-latte | 759-04-switch: Marley light, latte's colours, same process |
| 006 | Writes a render with `{{ accent }}` left into the fake theme folder and runs the hook | `cmp` of `omarchy.json` before and after: unchanged |
| 007 | Turns it off | 759-05-off: Gruvbox Dark; `theme` "Gruvbox Dark" |
| 008 | After 759-05 | the hook, `omarchy.json` and the template gone |
| 011 | Lists the user's own `~/.config/omarchy/themed` and `hooks/theme-set.d` (names and times) in `setup` and at the end | the two listings equal |

- REQ-009 (start with it on) and REQ-010 (toasts) are read in the diff: a relaunch would double the
  run for a path the reconcile already takes, and a failure is staged only by breaking the fake.
- The fake switch runs Omarchy's real `omarchy-theme-set-templates` with `HOME` at the fake home,
  so the shots also prove Omarchy's own renderer fills every slot the template uses. It needs
  Omarchy installed on the machine that runs the scenario; without `$OMARCHY_PATH` the scenario
  stops and says so. It never calls `omarchy-theme-set` or the apps' hooks.

### Risks
- **The refresh touches the desktop** (D3): `hyprctl reload` drops runtime `hyprctl keyword`
  changes, and the terminals and apps re-read the same colours. Once per turning on, and only when
  no render is there. Chad may prefer waiting for the next switch.
- **Omarchy moved its current-theme folder once** (report 02). The hook names the path as Omarchy's
  own app hooks do; if it moves again, the hook copies nothing and Marley keeps the last copy.
- **A theme's own `marley.json` wins**: Omarchy does not render over a file the theme ships. That
  is Omarchy's rule for every template and is kept.
- **Colours that are not six-digit hex** (rgba in a third-party `colors.toml`) leave `{{ mix … }}`
  unresolved, so the hook keeps the last good copy; an alpha suffix on an eight-digit colour drops
  that one key to Zed's base.
- **Several Marley data folders**: the hook names one themes folder; the last Marley to turn it on
  wins. Runs never write the real hook (D9).
- **The hook's temporary file** sits in the themes folder for a moment; Zed's watcher may see it.
  By the time a 100 ms batch is read it has been renamed, so its metadata fails and it is skipped.
  A temporary left by a killed hook logs one parse error at the next start.
- **A theme named "Omarchy" from an extension** would collide in the registry. None is installed
  here; the later load wins.

### Promotion (2026-10-11)
- Moved from `queued/` to `active/` at `28759687d8`; the BACKLOG row removed; the ticket in
  progress. Brain: consultation 0884411c91114017bf67a5b222bae860; Rusty's own theme follower
  (TICKET-029 there) reads `~/.local/state/omarchy/current/theme`, Omarchy 4's path, falling back
  to Omarchy 3's; the hook here names Omarchy 4's, as Omarchy's own app hooks do.
- **Seams checked again** (an Explore pass). What moved or differs:
  - `crates/zed/src/main.rs`: `load_user_themes_in_background` 1944-1994, `watch_themes`
    1996-2028, started at 882-883; both call `load_user_theme(&registry, &bytes)` and then
    `theme_settings::reload_theme`.
  - **Two corrections to the design:** `theme` and `theme_settings` are only dev-dependencies of
    `marley_workbench` (`Cargo.toml:114-115`), so they move to `[dependencies]`; and the settings
    observer reloads only when the theme's name or overrides change, so loading a changed family
    under the same name needs an explicit `theme_settings::reload_theme(cx)`, as `main.rs` does.
  - `theme_settings`: `load_user_theme(registry: &ThemeRegistry, bytes: &[u8])` 244, public;
    `reload_theme` 204; `ThemeRegistry::global(cx)` (`registry.rs:84`). The `theme` setting's
    path is `.theme.theme` (`ThemeSelection`, `settings_content/src/theme.rs:328-341`); the user's
    file through `raw_user_settings()`; Marley writes through `settings::update_settings_file`
    (as `links.rs:221`).
  - `claude_plugin.rs` shows its toast in one workspace; every workspace is
    `mcp::all_workspaces(cx)` from a task.
  - Lines: `shared_plugin::init` at `marley_workbench.rs:1003`; `default.json`'s
    `claude_code_shared_plugin` at 1897-1901; `layout_section` `[SettingsPageItem; 6]` at
    `marley_page.rs:59-168`; the runner's scrub at `script/e2e.sh:665-699`, exports at 703-708;
    ledger rows 59, 63, 68.
  - Omarchy here: `omarchy-hook` runs each `theme-set.d` file but `*.sample` with `bash`;
    `theme-set.d` exists and is empty; `tokyo-night` and `catppuccin-latte` ship `colors.toml`.
    The current theme (dazzle-dusk) ships its own `aether.zed.json`, no `marley.json`.

## Phase 2 — Code (2026-10-11)
- **Built:** `marley_workbench/src/omarchy_theme.rs` (`OmarchyPaths`, `refresh_program`,
  `template_is_marleys`, `install_in`, `copy_render`, `write_file`, `remove_in`, the
  `OmarchyTheme` global, `init`, `reconcile`, `turn_on`, `select`, `turn_off`, `user_theme`,
  `names_omarchy`, `show_warning`); `omarchy/marley.json.tpl` and `omarchy/theme-set-hook.sh`;
  `init` beside `shared_plugin::init`; `theme` and `theme_settings` moved to `[dependencies]`;
  `process.rs`'s doc and `.config/spawn-sites.txt` name `omarchy-theme-refresh`. Zed side: the
  setting (`settings_content/src/marley.rs`), its default and comment (`default.json`), the
  seventh Layout item (`settings_ui/src/marley_page.rs`), their three ledger rows extended. The
  runner's copy and exports; the guide's Settings section and two variables.
- **Deviations from the design:** the template's text keys left `muted` after the Test phase's
  first shot (F-claude-759-…): secondary text is a mix of `foreground` toward `background`, and
  comments, placeholders and hints take `dark_foreground`, against D8's "comment `muted`".
- **The review:** the refresh runs only when turned on after start, never at start; the switch
  is read again after the refresh; the replaced selection is kept only when it does not already
  name "Omarchy", so turning it on twice keeps the first; restore happens only while "Omarchy" is
  still selected (D4); the settings write happens outside any entity update; the file writes go
  through a temporary file and a rename; a path the shell cannot quote is an error, not a hook.
  Zed's watcher polls a themes folder that does not exist yet until it does
  (`fs_watcher.rs:238`), so the first copy is seen.
- **Clippy found and fixed:** `redundant_closure_for_method_calls`, `needless_pass_by_ref_mut`
  (three, `AsyncApp` by shared reference), and two compile errors (`TaskExt` in scope, a closure
  for `cx.update`). `cargo fmt`, then clippy again, clean.
- **The gate:** `script/gates.sh --diff` green on its first run, 17 of 17, on the tree with the
  template's fix.

## Phase 3 — Test (2026-10-11)
- **The scenario:** `script/e2e/759-follow-omarchys-theme.sh`, under `compositor sway`. A fake
  Omarchy home with tokyo-night's and catppuccin-latte's `colors.toml`; a `switch` that renders
  with Omarchy's own `omarchy-theme-set-templates` under the fake HOME and runs the `theme-set.d`
  hooks; a fake `omarchy-theme-refresh` that logs and switches to the current palette. Exit 0,
  all 13 checks passing. Shots read (second run, after the template fix):
  - `759-01-setting`: the Settings window's Marley page, Layout section; Follow Omarchy's Theme
    last, its description in full, the toggle off (REQ-001).
  - `759-02-absent`: on, with the fake home empty: Gruvbox Dark everywhere, the terminal's 16
    colours Gruvbox's; the home still empty, no refresh, `theme` "Gruvbox Dark" (REQ-002).
  - `759-03-on`: tokyo-night: the navy background, blue accent, Tokyo Night's syntax hues, the
    terminal's 16 colours tokyo-night's, secondary text legible; `theme` "Omarchy", one refresh,
    the template, the hook and `omarchy.json` in place, the copy dark (REQ-003, REQ-004).
  - `759-04-switch`: after the fake switch, the whole window light in latte's colours (red
    keywords, green strings, the latte terminal palette), the same process; the copy light, no
    second refresh (REQ-005).
  - `759-05-off`: Gruvbox Dark again; `theme` "Gruvbox Dark"; the template, the hook and the copy
    gone; the user's own `themed/` and `theme-set.d/` listings unchanged (REQ-007, REQ-008,
    REQ-011).
  - REQ-006: a render with `{{ accent }}` put back in, the hook run by hand: `omarchy.json`
    byte-for-byte the same.
- **Fix on the way:** the first run's 759-03 drew the rail's subtitles, the breadcrumb and the
  semicolons in `#414868`, close to invisible (F-claude-759-…, L-claude-759-…). The template
  changed; the build and the run again; 759-03 above is the second run's.
- **What no scenario shows:** a start with the setting on (REQ-009) and the toasts (REQ-010),
  read in the diff as the visual check plan says.

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added, #759); the guide (the Layout list, the settings block, a
  paragraph on Follow Omarchy's Theme, `MARLEY_OMARCHY_HOME` and `MARLEY_OMARCHY_REFRESH` in the
  variables table); `marley_workbench.md` (a new section). The three ledger rows describe what
  shipped.
- **Knowledge appended:**
  AD-claude-759-marley-follows-omarchys-theme-through-omarchys-own-templates-001;
  F-claude-759-the-theme-template-drew-secondary-text-in-omarchys-border-shade-001;
  L-claude-759-omarchys-muted-is-a-border-shade-not-a-text-colour-001.
- **Brain:** consultation 0884411c91114017bf67a5b222bae860 closed with a decision.
- **Closed:** the ticket moved to `closed/`, the pipeline archived, committed.
