# Marley's own app identity — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-437-marley-app-identity.md
- **Pipeline spec:** 437-marley-app-identity.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** Chad answered "Rename to Marley now" (2026-09-22) after the plan found the fork
  sharing Zed's config and database with the stock Zed at `~/.local/zed.app` (updated
  2026-09-22 14:06; `~/.config/zed/settings.json` touched 14:07).
- **Classification / tier:** chore; two tiny Zed touchpoints, no Marley crate.
- **Recall (§18.3):** nothing on this seam in the ledgers (the gpui era had its own name).
  `PR-claude-rename-sweeps-intradoc-links-001` applies in spirit: sweep for other readers of
  the old name (only `paths.rs` and `main.rs` read `APP_NAME`).
- **Discovery:** `crates/paths/src/paths.rs:14-40` (the constants), `:120-270` (derived dirs);
  `crates/zed/Cargo.toml:9` (`default-run`), `:56-58` (`[[bin]]`); `crates/zed/src/main.rs:10-16`
  (the assert); Chad's current settings: `agent_servers.claude-acp`, `base_keymap: Zed`, font
  sizes, the One Light / One Dark theme pair.
- **Human confirmation:** Chad's goal authorizes autonomous execution through commit
  (2026-09-22). No `TaskCreate` in this harness; checklists live here.
- **Promoted to active (2026-09-22, after W0's commits `5c80fb7` and `7abdd3e`).** Seams
  re-verified against the committed tree: `APP_NAME` at `crates/paths/src/paths.rs:17` (the
  fork comment above it at :16), `APP_NAME_LOWERCASE` at :21; `default-run = "zed"` at
  `crates/zed/Cargo.toml:9` and the main `[[bin]]` at :56-58; the compile-time assert at
  `crates/zed/src/main.rs:8-16`. Only `paths.rs` and that assert read `APP_NAME`. Other
  `zed` names (`crates/install_cli`, `crates/util/src/util.rs:374-378`'s CLI lookup,
  `script/install.sh`, the flatpak manifest) are packaging, deferred by the spec. The stock
  config holds `settings.json` (548 bytes, 2026-09-22 14:07) and an empty `themes/`; there is
  no `keymap.json`. Neither `~/.config/marley` nor `~/.local/share/marley` exists yet, and no
  debug build of the app exists in `/mnt/fast/target/debug`.
- **Recall added.** W0 made the ledger mechanical, so the two rows go in before the edits (the
  write hook is live in this session: `L-claude-436-a-new-hook-is-live-in-the-same-session-001`).
  The first `.rs` edit can start the editor plugin's workspace `cargo check` on the shared
  target (`L-claude-443-the-editor-plugin-shares-the-target-001`), and `paths` is a dependency
  of almost every crate; no gate starts while that check runs. Brain consultation
  `0c880588239b4e3584d1af0298f6c136`: nothing on this seam.
- **Phase 1 checklist (promotion):** promote the pair ✓ · re-verify seams ✓ · recall ✓ · ticket
  in-progress ✓ · backlog row removed ✓ · status PASS (autonomous) ✓.

## Phase 2 — Design
Drafted while #443's first FULL run finished; confirmed after W0.

- **Approach.** Zed's documented fork switch, and nothing more: `APP_NAME` decides every
  config, data, state, log and database directory, and the compile-time assert in `main.rs`
  keeps the binary's name in step. Two Zed files change; no Marley crate does.
- **§20.** Upstream Zed's behavior is kept exactly (its own directory layout under the new
  name); the reference section stands.
- **Hunks.**
  - `crates/paths/src/paths.rs:17`: `APP_NAME` becomes `"Marley"`, under a
    `// Marley:` line saying why. A test module at the foot of the file (the crate has none
    today) checks that the last component of `config_dir()`, `data_dir()` and `state_dir()`
    is `marley` ignoring case, so it holds on Linux (`marley`), macOS (`Marley` under
    `Library/Application Support` and `.local/state`) and Windows alike, with its own
    `// Marley:` line.
  - `crates/zed/Cargo.toml`: `default-run` (:9) and the main `[[bin]]` name (:57) become
    `marley`, each under a `# Marley:` line; `main.rs:8-16` checks the pair at compile time.
  - Two ledger rows, written first (the write hook refuses the edits otherwise):
    `crates/paths/src/paths.rs` and `crates/zed/Cargo.toml`.
- **Left alone on purpose:** the crash handler's `binary: "zed"` label (`main.rs:401`), the
  clap command name (`main.rs:1693`), the bundle metadata, the release channel's app id,
  `crates/cli`'s `zed-editor` lookup, `crates/util`'s installed-CLI paths and
  `script/install.sh`. None moves state; each becomes its own touchpoint when Marley ships a
  package.
- **The settings copy** (the one environment action, after the build and before the live
  drive): `install -Dm600 ~/.config/zed/settings.json ~/.config/marley/settings.json` only
  when the target does not exist. There is no `keymap.json` and `themes/` is empty, so nothing
  else is copied. Nothing under `~/.config/zed` is written.
- **Manifest.** `crates/paths/src/paths.rs` (Zed crate: the constant and the test),
  `crates/zed/Cargo.toml` (Zed crate: the two names), `docs/marley/zed-touchpoints.md` (the
  rows). CHANGELOG at Phase 5.
- **Regression test plan.**

  | Test | How | REQ |
  |---|---|---|
  | The binary is `marley` | `cargo build -p zed`, then `target/debug/marley` exists and `target/debug/zed` is not rebuilt | 001 |
  | The directories end in `marley` | `paths` unit test `the_app_keeps_its_own_directories` (`cargo nextest run -p paths`) | 002 |
  | The stock config is untouched | live drive: the mtimes of `~/.config/zed/settings.json` and `~/.local/share/zed` before and after the run | 003 |
  | Chad's settings came across | `cmp` the two `settings.json` files right after the copy; the live drive shows his theme | 004 |
  | The rows and comments exist | gate:16 green with 8 rows; `rg "Marley:" crates/paths crates/zed/Cargo.toml` | 005 |
  | The workspace builds and the touched crates pass | `script/gates.sh --diff` (the scope gains `paths` and `zed`) | 006 |

  The live drive (`dev-box-desktop` skill): launch `target/debug/marley`, confirm the process
  and `~/.local/share/marley/db`, screenshot the window and read it.
- **Risks.**
  - The app has never had a debug build on this box, and the gate's clippy and nextest now
    cover the whole `zed` crate: the first runs are long. No gate starts while the editor
    plugin's workspace check (set off by the first `.rs` edit) holds the build lock.
  - First launch writes Marley's own directories; if it also writes into
    `~/.config/marley/settings.json` (a first-run migration), the live drive records it
    rather than hiding it, because D4 expects a fresh data directory to count as a new install.
- **Phase 2 checklist:** approach ✓ · §20 confirmed ✓ · manifest ✓ · test plan ✓ · risks ✓ ·
  present (autonomous) ✓.

## Phase 3 — Implement
- **Built, in order.**
  - `docs/marley/zed-touchpoints.md`: rows for `crates/paths/src/paths.rs` and
    `crates/zed/Cargo.toml`, first, as the write hook requires.
  - `crates/paths/src/paths.rs`: `APP_NAME = "Marley"` under a `// Marley:` line (between
    the doc comment and the item; the doc still attaches to the constant).
  - `crates/zed/Cargo.toml`: `default-run = "marley"` and the main `[[bin]]` `name = "marley"`,
    each under a `# Marley:` line.
- **Checks.** gate:16's check: "every touchpoint recorded (8 rows)". `cargo build -p zed`
  (17:49 to 17:53, 982 crates, 4m 39s, no warnings): `target/debug/marley` exists, 1.4 GB, so
  `main.rs`'s compile-time assert holds; no `target/debug/zed` was produced. The editor
  plugin started no workspace check after the `.rs` edit.
- **Deviations.** None. The `paths` unit test is Phase 4's, per the phase rule.
- **Phase 3 checklist:** rows ✓ · `APP_NAME` ✓ · binary name ✓ · build ✓.

## Inspect (Phase 3.5)
One critic, sized to a two-hunk change, on the lens that matters here: what the renamed fork
still shares with a stock Zed on the same machine. It read the code behind every candidate
(file:line in its report), ran read-only commands (`xdg-mime`, the desktop entries, the runtime
directory), changed nothing and ran no cargo. I re-checked the two medium findings in the code.

| # | Finding | Verdict | Fix |
|---|---|---|---|
| 1 | medium: the `dev` release channel is the only thing keeping Marley off stock Zed's Secret Service items (fixed label `zed-github-account`), its updater (which would rsync stock Zed over `~/.local/zed.app` and restart as stock) and its app id `dev.zed.Zed`; a changed `crates/zed/RELEASE_CHANNEL` or a bundle script would flip all three (`release_channel/src/lib.rs:13-34`, `auto_update.rs:281-311`) | real, latent: the channel is `dev` today, and `ZED_RELEASE_CHANNEL` is unset | a `paths` unit test (Phase 4) fails if `RELEASE_CHANNEL` leaves `dev`; the `paths.rs` ledger row and REQ-007 state the constraint; TICKET-445 (Deliberate) gives Marley its own keyring label, updater source, app id and URL scheme |
| 2 | medium: `zed://` links opened from Marley go through xdg-open to stock Zed, which owns the scheme on this box (`client.rs:1961` keeps only channel links in the app) | real, pre-existing; the rename makes the action land in the other app's state | out of this ticket's two hunks: TICKET-445 |
| 3 | low: on `dev`, credentials go to a plaintext `~/.config/marley/development_credentials` (0644 under a 0700 `~/.config`), so Marley starts signed out with no API keys | accepted: no collision, and Chad re-enters keys; `ZED_DEVELOPMENT_USE_KEYCHAIN=1` must stay unset (it would share stock's items) | noted for the report |
| 4 | low: the CLI's bundled `--uninstall` would delete stock Zed's files; unreachable today (no Marley CLI is built) | real, latent | TICKET-445 (build the CLI with `no-bundled-uninstall`) |
| 5 | low: nothing calls `paths::state_dir()`, so `~/.local/state/marley` is never created; the third directory the app makes is `~/.cache/marley` (`temp_dir()`) | real, spec accuracy | REQ-002 now expects config, data and cache live, with `state_dir` checked by the unit test |
| 6 | low: `~/.zed_server` on SSH hosts is shared with any upstream Zed dev build | accepted: dev names its server binary `-dev-build`, distinct from stock stable's | none |
| 7 | nit: `.claude/commands/pipeline/validate.md` said "the `zed` binary"; pick Marley's window by class `dev.zed.Zed-Dev` (stock's is `dev.zed.Zed`); check stock Zed is not running during the drive | real | `validate.md` says `marley`; the drive selects by class and pid; stock Zed was not running at the pre-drive snapshot |
| 8 | nit: the GPUI inspector's "open source" runs `zed` from PATH; `script/cargo-timing-info.js` writes under `$XDG_DATA_HOME/zed` | accepted: debug-only or unused, pre-existing | none |
| 9 | nit: telemetry is on by default and the copied settings leave it on, so Marley reports to zed.dev as "Zed Dev" with new ids | Chad's choice, not a defect: stock Zed runs with the same settings | raised in the report; the copy stays verbatim |

- **Checked and fine (critic):** the single-instance socket derives from `data_dir()` and Dev
  skips the check; the database, threads, node, languages, extensions, agents, logs
  (`Marley.log`) and the crash-handler socket all derive from `APP_NAME`; first run is a new
  install (fresh `system_id` and `installation_id`, onboarding shown, the layout backfill
  skipped: D4 holds); nothing uses `CARGO_BIN_EXE_*`; the settings copy keeps mode 0600.
- **Phase 3.5 checklist:** spawn critic ✓ · review findings ✓ · fix confirmed ✓ · verify fixes
  (the guard test lands in Phase 4) ✓ · write inspect ledger ✓.

## Phase 4 — Validate
- **Tests added** (`crates/paths/src/paths.rs`, a `// Marley:` test module):
  `the_app_keeps_its_own_directories` (REQ-002: the last component of `config_dir()`,
  `data_dir()` and `state_dir()` is `marley`, compared without case so macOS and Windows hold
  too) and `the_release_channel_stays_dev` (REQ-007, from inspect: `crates/zed/RELEASE_CHANNEL`
  is `dev`). `cargo nextest run -p paths`: 2 passed.
- **The settings copy.** `install -Dm600 ~/.config/zed/settings.json
  ~/.config/marley/settings.json` (the target did not exist); `cmp` identical, both 0600,
  548 bytes; the stock file's mtime unchanged (14:07:11).
- **Live drive** (the headless output, so Chad's screen was never touched; captures in
  `scratchpad/w1-drive/`).
  - A temporary window rule put class `dev.zed.Zed-Dev` on the headless workspace 3.
  - The first launch through Hyprland's `exec` (working directory `$HOME`) panicked right after
    loading the shell environment. Reproduced with output captured (`exec-run.log`): "dev asset
    loading requires running from within the checkout" (`util.rs:762`). A debug build reads its
    assets from the checkout, found by a `.git` above the executable or the working directory,
    and this box builds into `/mnt/fast/target`, outside the checkout. A fresh first run started
    from the checkout (`fresh-run.log`, its own `--user-data-dir`) works and writes no settings.
    Not a W1 regression; recorded on TICKET-445 as a need for a daily-driver build.
  - That panicked first launch had already created the installation id, so the next launch
    counted as an existing install and ran `agent_ui`'s one-time editor-layout backfill, writing
    a `project_panel`/`outline_panel`/`collaboration_panel`/`git_panel` left, `agent` right block
    into `~/.config/marley/settings.json` (823 bytes). The backfill flag was set by that run, so
    I restored the verbatim copy; after the next launch the file stayed byte-identical.
  - `marley-w1.png` (18:20) and `marley-w1-launcher.png` (18:23, started through Hyprland from
    the checkout, the way a launcher that runs it in place would): Marley's window on the
    scratch project, title "Restricted Mode · project", Zed's "Unrecognized Project" trust
    prompt, "Sign In" at the top right (a fresh install: no credentials carried over). In the
    first, the backfilled layout put the project panel on the left; in the second, with Chad's
    verbatim settings, it is gone from the left, as Zed's defaults have it.
  - Marley's own directories exist: `~/.config/marley` (`settings.json`, `themes/`),
    `~/.local/share/marley` (`db`, `extensions`, `languages`, `logs/Marley.log`, …) and
    `~/.cache/marley`. The running process's inotify watches cover `~/.config/marley/themes`,
    `~/.local/share/marley` and its extensions, and nothing under `~/.config/zed` or
    `~/.local/share/zed`.
  - REQ-003: every file under `~/.config/zed`, `~/.local/share/zed` and `~/.local/state/zed`
    (19,994 entries) has the same mtime after the drive as before it; stock Zed was not running.
  - Cleanup: all Marley processes ended with SIGTERM; `hyprctl reload` cleared the window rule;
    `rusty headless down` (HDMI-A-2 back at 0,0, no config errors); the two scratch data
    directories removed.
- **Gate** (`script/gates.sh --diff`, 18:24:20 to 18:29:09): GATE GREEN [diff], 15 of 15.
  Scope: the Marley crates plus `paths` and `zed`; 365 tests passed, 1 skipped (a
  pre-existing ignored test in the scope); gate:16 "8 rows"; mutation "no mutable lines in the
  diff" (a constant and a test module). The receipt matches the worktree.

| REQ | Proof |
|---|---|
| 001 | `target/debug/marley` built (Phase 3); `cargo run` and the drive start it; no `target/debug/zed` |
| 002 | the unit test; the live directories (config, data, cache) |
| 003 | the 19,994-entry mtime comparison, identical |
| 004 | `cmp` after the copy and again after the last launch |
| 005 | gate:16 green with 8 rows; the `// Marley:` and `# Marley:` lines in both files |
| 006 | the green `--diff` gate |
| 007 | `the_release_channel_stays_dev` |

- **Phase 4 checklist:** tests ✓ · settings copy ✓ · live drive ✓ · stock-state check ✓ ·
  gate ✓.

## Phase 5 — Complete
- **Documentation (§21).** `CHANGELOG.md` gains a `### Changed` entry (the fork is Marley).
  `docs/marley/workbench-shell.md`'s status line says W1 shipped and points at the debug-launch
  constraint. Both ledger rows describe what shipped: the `paths.rs` row names the tests and the
  `dev` channel constraint (updated at inspect), the `crates/zed/Cargo.toml` row the two names.
  No Marley crate changed. TICKET-445 carries the packaging identity and the debug-launch need.
- **Knowledge (§19).** Inspect appended `PR-claude-a-zed-fork-identity-is-more-than-app-name-001`
  (no real bug in W1's code, so no failure entry). This phase appended
  `L-claude-437-a-debug-marley-starts-inside-the-checkout-001`,
  `L-claude-437-the-headless-live-drive-recipe-001` and
  `AD-claude-437-marley-identity-is-app-name-and-the-dev-channel-001`; the project memory's
  traps note points at the drive recipe.
- **Brain.** Consultation `0c880588239b4e3584d1af0298f6c136` closed with `brain decide`:
  `decisions/marley-forks-own-identity-app-name-marley-the-marley-binary-the-dev-channel-kept`,
  follow-up due 2026-10-22.
- **For Chad** (the report): Marley starts signed out and without API keys (on `dev` it keeps
  credentials in `~/.config/marley/development_credentials`; do not set
  `ZED_DEVELOPMENT_USE_KEYCHAIN=1`, which would share stock Zed's keyring items); telemetry stays
  on as in his Zed settings; `zed://` links from Marley open stock Zed until TICKET-445.
- **Ticket.** TICKET-437 closed; its backlog row left at promotion. TICKET-445 was minted into
  Deliberate.
- **Phase 5 checklist:** CHANGELOG + architecture docs ✓ · capture knowledge ✓ · close ticket ✓
  · archive pipeline ✓.
