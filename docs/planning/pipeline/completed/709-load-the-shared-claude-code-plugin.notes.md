# Load the shared Claude Code plugin — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-709-load-the-shared-claude-code-plugin.md
- **Pipeline spec:** 709-load-the-shared-claude-code-plugin.spec.md

## Phase 1 — Plan
- **Request:** Chad chose rustal-harness TICKET-108 first (2026-10-09); this is Marley's slice that
  loads its result. He asked the session to work through every open item.
- **Classification:** feature, Marley crates plus the `marley` settings paths (rows extended).
- **Recall (§18.3):**
  - #652 shipped `MARLEY_BIN` and the report contract, and planned this slice: D9 (the folder first
    in `CLAUDE_CODE_PLUGIN_DIRS`, other Marley copies removed) and D10 (by digest, 0400/0700).
  - #648's version rows gate integrations; #653's `claude_ide` is the model for a version-gated
    setting.
  - PR-687: scenarios never run a real Claude Code.
- **Discovery (an Explore pass, 2026-10-09):**
  - The digest of the six files on disk matches TICKET-108's.
  - `agent_environment` runs in `terminal.rs` for every terminal.
  - `write_program_in` writes `MARLEY_BIN`'s program.
  - The version table is `marley_agent::versions::INTEGRATIONS`.
  - Settings: `claude_code_ide` (marley.rs :196, default.json :1872, marley_page.rs :474) and
    `allow_untested_versions` (default.json :1855, the Agent Versions section).

### Design
- **Files:** `crates/marley_workbench/claude_shared_plugin/{.claude-plugin/plugin.json,
  hooks/hooks.json, hooks/register.js, LICENSE-MIT, LICENSE-APACHE, claude-code-versions.json}`,
  copied from rustal-harness.
- **`marley_workbench/src/shared_plugin.rs` (new):**
  - `FILES`, in `rh`'s order, and `digest()` (sha2: each path and text preceded by its length as a
    little-endian u64).
  - `install_in(shared) -> io::Result<PathBuf>`: the digest's folder. If it is there, `check_in`
    compares every file; if not, the files are staged in `.staged-<uuid>` (folders 0700, files
    0400) and renamed into place.
  - `SharedPlugin { Off, On }` from `marley.claude_code_shared_plugin`.
  - `reconcile`, run at `init`, on `SettingsStore` changes and on `agent_versions::observe`, asks
    whether the plugin is wanted (setting on and `agent_versions::is_on(&CLAUDE_SHARED_PLUGIN)`).
    Wanted, it installs off the main thread and then `set_shared_plugin(Some(dir))` if still
    wanted. Otherwise it calls `set_shared_plugin(None)`.
- **`marley_terminal/src/identity.rs`:** `PLUGIN_DIRS_VARIABLE`, a `SHARED_PLUGIN` static with
  `set_shared_plugin`/`shared_plugin`. For a named terminal, `agent_environment` writes the
  variable: the folder first, then Marley's own inherited entries with any under the same `shared`
  parent removed. It writes the variable only when that value differs from the inherited one.
- **`marley_agent/src/versions.rs`:** `CLAUDE_SHARED_PLUGIN` (from 2.1.287, no upper bound) in
  `INTEGRATIONS`. `agent_versions::wanted` chips it only while the setting is on.
- **Settings (rows extended first):**
  - `marley.rs`: `claude_code_shared_plugin`, and the `allow_untested_versions` doc list;
  - `default.json`: the setting `false` and `allow_untested_versions.claude_shared_plugin: false`;
  - `marley_page.rs`: the Agents toggle and the Agent Versions toggle.
- **`MarleySettings`:** `shared_plugin: SharedPlugin`.
- **File manifest:**
  - Marley crates: `marley_workbench` (`shared_plugin.rs`, `marley_workbench.rs`,
    `agent_versions.rs`, the six files), `marley_terminal/src/identity.rs` and
    `marley_agent/src/versions.rs`.
  - Zed paths: `settings_content/src/marley.rs`, `assets/settings/default.json` and
    `settings_ui/src/marley_page.rs`.
  - Docs: the touchpoints and the guide.
  - The scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 002 | `profile_setting marley.claude_code_shared_plugin true`, `MARLEY_CLAUDE` the stand-in (2.1.287); a new terminal runs `claude`, which prints the variable, reads the first folder's manifest, checks the files' modes, and runs `$MARLEY_BIN report waiting --activity "Bash: ls"` | 709-01-on: the printout, and the rail row waiting on Bash: ls |
| 003 | The setting off; a new terminal runs `claude` | 709-02-off: no Marley folder printed |
| 004 | — | The gate |

### Risks
- **Marley's environment.** A Marley started from a Marley terminal inherits the parent's
  variable. The filter removes any copy under this data dir's `shared` folder, and a copy from
  another data dir stays (D9).
- **Managed `disableSideloadFlags`:** Claude Code refuses to start with the variable set. The
  setting is off by default, and the guide names this.

## Phase 2 — Code
- **Built:**
  - The six files, copied from rustal-harness, under `crates/marley_workbench/claude_shared_plugin/`.
    Their digest, computed as `rh` does, is `83d0bb8f…0303`, TICKET-108's.
  - `shared_plugin.rs`:
    - `FILES` (`include_str!`) and `digest()`;
    - `install_in`, which stages in `.staged-<uuid>` (folders 0700, files 0400, `create_new`,
      `sync_all`) and renames into place, checking a folder another Marley put first;
    - `check_in`: an altered file is an error naming the folder;
    - `SharedPlugin::of(cx)`, which reads the merged settings;
    - `reconcile` at `init`, on `SettingsStore` and on `agent_versions::observe`. It installs off
      the main thread with `futures::future::lazy`, then sets the folder if it is still wanted.
  - `marley_terminal::identity`: `PLUGIN_DIRS_VARIABLE`, `set_shared_plugin` and `shared_plugin`,
    and `plugin_dirs(inherited, plugin)`. `agent_environment` writes the variable for a named
    terminal when it differs from the inherited one.
  - `marley_agent::versions::CLAUDE_SHARED_PLUGIN` (2.1.287 and later) in `INTEGRATIONS`;
    `agent_versions::wanted` chips it only while the setting is on.
  - Settings (rows extended first): `marley.rs` (the field, the `allow_untested_versions` doc),
    `default.json` (both entries with comments), and `marley_page.rs` (the Agents toggle and the
    Agent Versions toggle).
  - `guide.md`: "The shared Claude Code plugin", in the #652 section.
- **Deviation:** the setting is read from the merged settings in `shared_plugin.rs`, not added to
  `MarleySettings`, whose `from_settings` was at clippy's 100-line limit.
- **Review of the diff:**
  - `agent_environment` runs in the terminal's spawn task, and the static is read there under its
    lock, like `MARLEY_BIN`'s program.
  - Turning the setting off clears the folder at once, and only new terminals see the change.
  - Not covered: a Marley started from a Marley terminal while the plugin is off keeps an inherited
    copy from its parent, since without the folder there's no `shared` parent to filter by.
- **Gate:** `709-gate-1.log` RED (clippy `tuple_array_conversions` in the digest loop and
  `from_settings` at 101 lines; dylint, an `async` block with no `.await`). Fixed: the parts hashed
  one by one, the setting read in place, and `futures::future::lazy` as `agent_reports` does.
  `709-gate-2.log`: GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/709-load-the-shared-claude-code-plugin.sh`, under `compositor sway`. A
  stand-in `claude` (Python) is first on the terminal's PATH and named by `MARLEY_CLAUDE`, so
  Marley's version check reads 2.1.287. It logs what it finds, reports through `$MARLEY_BIN`, and
  waits for `q`. Real Claude Code never runs (PR-687), so the plugin's mod itself isn't exercised:
  the stand-in runs the report the mod runs while a dialog stands (`waiting --activity "Bash: ls"`,
  as TICKET-108 specifies). The mod's own behaviour is rustal-harness's, gated there (MH-001 to
  MH-008).
- **First run (`shots-709a`): every check passes.**
  - **Checks:** the plugin first is `rustal-harness 0.2.0`; its folder is the digest;
    `hooks/register.js=400 LICENSE-MIT=400 hooks=700`; the report's exit is 0; with the setting
    off the stand-in finds no plugin.
  - **709-01-on (REQ-001, REQ-002):** the new terminal prints
    `CLAUDE_CODE_PLUGIN_DIRS=<profile>/claude-code/shared/83d0bb8f…0303`, then
    `plugin: rustal-harness 0.2.0`, the folder, the modes and `report waiting: exit 0`. The rail's
    "Needs you 1" lists Claude Code · repo, "Waits for you", and the terminal's row reads waiting.
  - **709-02-off (REQ-003):** after `marley.claude_code_shared_plugin` was set false, a new
    terminal's stand-in prints `CLAUDE_CODE_PLUGIN_DIRS=` (empty) and `no plugin`.
  - Chad's Hyprland untouched.
- **Seen, not in scope:** in 709-02 the stand-in's row reads "Claude Code · waiting" with no report.
  The hook-path reading (#519) takes a `claude` in the foreground with no output as waiting on
  the user, as it did before.

## Phase 4 — Complete
- **Documented:**
  - `CHANGELOG.md` (Added: the shared Claude Code plugin);
  - `docs/marley_architecture/marley_workbench.md`, "The shared Claude Code plugin";
  - `docs/marley_architecture/marley_agent.md`, the version row;
  - the guide's paragraph (Phase 2);
  - the three settings rows in `docs/marley/zed-touchpoints.md`, checked against what shipped.
- **Knowledge appended:** AD-claude-709-the-shared-plugin-is-carried-by-digest-and-off-by-default-001.
  No bug was found.
- **Brain:** no `rusty` MCP server in this repository's sessions; the decision is in the ledger.
- **Ticket:** closed; the BACKLOG row left at promotion.
- **Gate:** `709-gate-3.log` RED on shellcheck SC2016 (the scenario wrote a literal `$PATH` in single quotes; it now passes `"\$PATH"` as an argument, and the scenario ran again, every check passing, `shots-709b`); `709-gate-4.log` GATE GREEN [diff], on the tree committed.
