# An installed release build of Marley — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-502-installed-release-build.md
- **Pipeline spec:** 502-installed-release-build.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25: "lets do 1 through 8", item 8 of the list after the browser
  waves: an installed release build, rebuilt by a script, so the debug binary in the shared
  target directory stops being his daily editor.
- **Classification / tier:** chore, packaging; scripts only, no Rust.
- **Recall (§18.3):**
  - L-claude-438: the `dev` channel skips the single-instance check, and a second `marley`
    on the same data directory hung. The installed and the debug builds share that directory,
    so the script warns when a Marley runs, and Chad quits the debug one before the first start
    from the menu.
  - L-claude-460: a second app on its own `--user-data-dir` does not hang (the e2e profile copy
    relies on it); a start through Hyprland's `hl.exec_cmd` exited silently after the shell
    environment line, where `setsid -f` from a shell worked. It happened again today at 20:02
    (Marley.log: the start line, the migration check, the environment line, then nothing; no
    coredump, nothing in the journal). None of those starts went through `uwsm-app`, which is
    how Omarchy starts every entry, so REQ-005 tests the menu's own path.
  - L-claude-448-running-zeds-dylint-library-on-the-fork-001 on guards: wait with `pgrep -x cargo`, never `pgrep -f`, which matches the
    guarding shell.
  - Brain: consultation 7cc8dbc9ec7b4a6f94e9d5e6f188a8bc returned no decision on installing
    Marley, only follow-ups due on other projects.
- **Checklist (no task tool in this session):** pick · pre-flight · recall · promote · prior
  art · spec · design — all done.
- **Discovery:** `script/install-linux`, `script/bundle-linux`, `script/install.sh`,
  `crates/zed/resources/zed.desktop.in`, `crates/zed/build.rs` (`ZED_COMMIT_SHA`),
  `crates/zed/src/main.rs` (no single-instance check on the `dev` channel),
  `script/e2e.sh` (`marley=` from the target directory at line 48, before the scenario is
  sourced), `.claude/hooks/lib-hook-helpers.sh` (`marley_owned_path`), `script/gates.sh`
  (`shellcheck_g`'s file list), Rusty's `omarchy/install.sh`.

- **Decisions:** D1 to D4 in the spec. The release build started in the background during
  Plan (nothing else needed cargo), so the script's own build at Test is the incremental one.

### Design
- **Approach.** One bash script, `script/install-marley [--prefix DIR]`, Marley-owned like
  `script/gates.sh`: `set -euo pipefail`; the repository root from the script's path; wait with
  `until ! pgrep -x cargo`; `cargo build --release -p zed --bin marley` from the root; the binary
  from `${CARGO_TARGET_DIR:-<root>/target}/release/marley` (the rule `script/e2e.sh` uses).
  Install: `install -Dm755` to `<bin>/.marley.new.<pid>` then `mv -f` over `<bin>/marley` (D2);
  `install -Dm644` the icon (`crates/zed/resources/app-icon-dev.png`, 512 px) to
  `share/icons/hicolor/512x512/apps/marley.png`; the desktop entry written by a here-document with
  the absolute binary path in `TryExec` and `Exec` (`%U`, which Zed's main binary takes as
  `file://` URLs), `Icon=marley`, `StartupWMClass=dev.zed.Zed-Dev` (the dev channel's app id),
  `Categories=Development;IDE;TextEditor;`, no `MimeType` (Marley must not become a handler for
  text files or folders) and no actions. `update-desktop-database <share>/applications` when the
  program exists, and `gtk-update-icon-cache` is not needed for a hicolor PNG lookup by name.
  Report: the commit (`git rev-parse --short HEAD`), "with changes not committed" when `git
  status --porcelain` prints anything, and a warning naming the pids of running Marleys
  (`pgrep -x marley`) that the new binary starts only in a new process and that two Marleys on
  one data directory hang.
- **`just install`** runs the script with its arguments.
- **`script/e2e.sh`** gains `binary <path>` for scenarios; `marley=` moves after the scenario is
  sourced and after `setup`, so `setup` may name the binary it installed. The existence check
  names the binary it looked for.
- **The owned set.** `script/install-marley` joins `marley_owned_path`, the ledger's prose list
  and `shellcheck_g`'s file list (and CONSTITUTION's gate:11 line).
- **File manifest.** `script/install-marley` (new, Marley-owned) · `justfile` · `script/e2e.sh`
  · `script/gates.sh` · `.claude/hooks/lib-hook-helpers.sh` · `docs/marley/zed-touchpoints.md`
  (the owned list) · `CONSTITUTION.md` (gate:11's list) · `script/e2e/502-installed-release-build.sh`
  (Test). No Rust, no Zed path.

### E2E plan
| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | setup: `script/install-marley --prefix $E2E_WORK/prefix` (the release build is current, so cargo only checks); its output names the commit | the run log |
| REQ-002 | setup: `desktop-file-validate` on the installed entry; `Exec`'s path is the installed binary, which `binary` names for the run | the run log |
| REQ-003 | the runner starts the installed binary on the profile copy with the scratch repository | `502-01-installed`; the startup line's sha equals `git rev-parse --short HEAD` |
| REQ-004 | steps: the script again while that Marley runs; its exit status; settle; the same window | `502-02-reinstalled` |
| REQ-005 | steps: `quit_marley`, then `uwsm-app -- gtk-launch marley.desktop <repo>` with the sway's `WAYLAND_DISPLAY`, `XDG_DATA_HOME=$E2E_WORK/prefix/share` (where the entry is) and `XDG_CONFIG_HOME` holding a copy of the profile's settings; the window maps and stays for ten seconds; teardown ends it by pid | `502-03-from-the-menu` |

Not reachable by a scenario: the menu itself (it is Chad's shell on his screen) and Chad's real
`~/.local` install, which Test runs once as `just install` and reports.

### Risks
- A release build of the whole workspace is long (thin LTO, one codegen unit per dependency).
  It ran in the background during Plan; later reinstalls rebuild only what changed, then link.
- The binary is large with `debug = "limited"`; measured at Test, and stripping stays out unless
  the size is a problem (backtraces keep their lines).
- The dev channel's app id is shared with Zed's own dev builds; Chad has none installed (his Zed
  is the stable `dev.zed.Zed`).

## Phase 2 — Code
- **Built.** `script/install-marley [--prefix DIR]`: the release build after the box's other
  cargo runs; the binary staged as `bin/.marley.new.<pid>` (removed by an EXIT trap if the run
  stops half way) and renamed over `bin/marley`; the icon; the desktop entry with the binary's
  path in `TryExec` and, quoted, in `Exec` with `%U`; `update-desktop-database`; the report (the
  commit, uncommitted changes, running Marleys, a `bin` not on `PATH`). A prefix is made absolute
  first, and one holding a character `Exec` would need escaped (`"`, a backtick, `$`, `\`, a
  newline) is refused. `just install *args`. `script/e2e.sh`: `binary <path>`; the debug
  binary's existence check runs only when no binary is named, and a named one is resolved and
  checked after `setup`. The owned set, the ledger's list, gate:11's file list, its comment and
  CONSTITUTION's gate:11 line name the script.
- **Deviations.** `Categories=Development;IDE;` rather than Zed's four: `desktop-file-validate`
  hints that `Utility` and `Development` are two main categories (the entry could show twice in
  a menu) and that `TextEditor` wants `Utility`; `Development;IDE;` draws neither hint (changed at
  Test, after the first run).
- **Review.** Against REQ-001 to REQ-005: the build and install steps stop the script on any
  failure (`set -euo pipefail`); a rename never writes into a running executable; `pgrep -x`
  guards, per L-claude-448; nothing in the script reads Warp, and nothing is Zed code. Checked
  by hand: shellcheck clean on the script and the runner, `--help`, an unknown argument (exit 2),
  a prefix with `$` (exit 2). No Rust, so no cargo check or clippy.
- **Addendum at Test: the launcher (D5).** The first run's menu start panicked (#512), and the
  panic went to stderr only: on the `dev` channel `crashes::force_backtrace` is the whole panic
  hook (`crates/zed/src/main.rs:419`), panics unwind (no coredump), and nothing reaches
  `Marley.log`. So the binary moved to `lib/marley/marley`, and `bin/marley` became a launcher
  written by the script: when stderr is not a terminal it appends a start line and then stderr to
  `${XDG_DATA_HOME:-~/.local/share}/marley/logs/stderr.log`, keeping one `.old` past 10 MiB, and
  `exec`s the binary, so the process is still `marley` to `pgrep -x`. The launcher is written to a
  temporary name and renamed like the binary. Checked by hand: shellcheck clean on the script and
  on the launcher it writes; `XDG_DATA_HOME=<scratch> <prefix>/bin/marley --no-such-flag` exits 2
  with clap's error in the log after the start line.

## Phase 3 — Test
- **Scenario:** `script/e2e/502-installed-release-build.sh` (`compositor sway`), four runs.
  - **Run 1, red.** Everything to 502-02 held; the menu start (`uwsm-app -- gtk-launch
    marley.desktop` with the sway's display) panicked at
    `crates/gpui_linux/src/linux/wayland/client.rs:1921` (`Option::unwrap()` on `None`, the
    `wl_keyboard.modifiers` arm): after `quit_marley` the run's last `wtype` had exited, so the
    seat had no keyboard and the new client got no usable keymap. Upstream Zed has the same code
    (checked against `main` through the GitHub API). Filed as #512, next in the queue; the
    scenario now gives the seat its held keyboard back before the menu start, as a desktop's
    seat has one.
  - **Run 2, green**, with the keyboard held: the menu's Marley ran in
    `app-graphical.slice/app-Hyprland-gtk\x2dlaunch-*.scope` and stayed up.
  - **Run 3, green**, after `Categories=Development;IDE;` (desktop-file-validate had hinted at
    two main categories).
  - **Run 4, green**, after the launcher (D5): the final record below.
- **Shots (run 4), read:**
  - `502-01-installed` — the installed release binary (`<prefix>/lib/marley/marley`, 2.1 GB) in
    the Marley layout on the scratch repository: the rail's `repo` group with its `repo — bash`
    row, the terminal, the project panel, branch `installed`. REQ-003; the startup line reads
    `sha 644f50b`, which is `HEAD`.
  - `502-02-reinstalled` — after `script/install-marley --prefix` ran again while that Marley ran
    (exit 0, "Marley is running (pid …)"), the process still alive and its palette open on
    Ctrl+Shift+P, Marley's own commands at the top. REQ-004.
  - `502-03-from-the-menu` — the Marley that `uwsm-app -- gtk-launch marley.desktop <repo>`
    started through the entry's launcher, on a fresh profile under scratch `XDG_DATA_HOME` and
    `XDG_CONFIG_HOME`, open on the repository from `%U`, alive after ten seconds. REQ-005.
- **Logs (run 4):** the install's output names `bin/marley at 644f50b667, with changes not
  committed` (REQ-001); `desktop-file-validate` printed nothing (REQ-002); the scratch
  `logs/stderr.log` holds the menu start's line and, after the launcher ran with
  `--no-such-flag` (exit 2), clap's `unexpected argument '--no-such-flag'` (REQ-006).
- **Focus:** sway, so Chad's session saw nothing: "hyprland: 0 Marley windows before the run, 0
  after; the run added no rule and did not reload it."
- **Chad's install:** `just install` into `~/.local`: the launcher `~/.local/bin/marley` (681
  bytes), the binary `~/.local/lib/marley/marley`, `marley.desktop` (validates clean) and the
  icon. No Marley was running: Chad had quit his debug one at 20:11:05 (corrected after the
  commit: `telemetry.log` ends with `App Closed` then; this entry first read the silent end of
  `Marley.log` as a death).
- **Gate:** `just gate-fast` (no Rust): `GATE GREEN [fast]`, log in the scratchpad
  (`gate-502.log`).
- **Pre-existing, not in scope:** the fresh profile logs `activation token received with no
  pending activation` (gpui, a gtk-launch start notification it did not ask for) and a llama.cpp
  provider error from Chad's settings.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Marley in the app menu); `docs/marley/README.md` gains "Running
  Marley" (`just build`, `just install`, the launcher's stderr log, one Marley at a time);
  CONSTITUTION's gate:11 line and the ledger's owned list name `script/install-marley`. No Zed
  path is touched, so no touchpoints row; no Marley crate changed, so no per-crate note.
- **Knowledge:** F-claude-502-a-seat-with-no-keymap-panics-gpuis-keyboard-handler-001 (to #512);
  L-claude-502-a-dev-channel-panic-reaches-stderr-only-001;
  L-claude-502-omarchy-starts-an-entry-through-uwsm-app-and-gtk-launch-001;
  L-claude-502-the-release-build-on-the-dev-box-001;
  AD-claude-502-the-installed-marley-is-a-copy-and-a-launcher-001.
- **Brain:** consultation 7cc8dbc9ec7b4a6f94e9d5e6f188a8bc closed with
  `decisions/marley-is-installed-as-a-release-copy-with-a-stderr-launcher-on-the-dev-channel`
  (follow-up 2026-10-09).
- **Ticket:** closed; the pipeline pair archived to `completed/`.
