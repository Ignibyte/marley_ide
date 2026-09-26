---
pipeline_id: c87c7161-ae9b-4b03-b657-210d0df95d76
ticket: docs/planning/tickets/closed/TICKET-502-installed-release-build.md
status: Phase 4 — Complete PASS
title: "An installed release build of Marley"
type: chore
slice: packaging, cross-cutting (item 8 of the list after the browser waves)
references: [docs/planning/tickets/open/TICKET-445-marley-release-identity.md]
---

## Title
`just install` builds `marley` in the release profile and installs a copy to
`~/.local/bin/marley` with a desktop entry, so Chad starts Marley from the launcher instead of
running the debug binary out of the shared target directory, and a `cargo clean` or a broken
build no longer takes his editor with it.

## Scope
### In
- `script/install-marley`: waits for the box's other cargo runs, builds
  `cargo build --release -p zed --bin marley`, and installs under a prefix (`~/.local` unless
  `--prefix <dir>` names another): the binary at `<prefix>/lib/marley/marley`, copied to a
  temporary name and renamed over the old one, so a running Marley keeps the file it started
  from; its launcher at `<prefix>/bin/marley`, which appends stderr to `logs/stderr.log` in
  Marley's data directory when stderr is not a terminal (added at Test, D5); the icon
  at `<prefix>/share/icons/hicolor/512x512/apps/marley.png`; the desktop entry
  `<prefix>/share/applications/marley.desktop` naming the installed binary. It refreshes the
  desktop database when `update-desktop-database` exists, prints the commit it installed (and
  says so when the tree had changes not committed), and warns when a Marley is running.
- `just install`, the recipe that runs it.
- `script/e2e.sh` gains `binary <path>`, so a scenario runs another build than the debug one.
- The script joins the Marley-owned set (`marley_owned_path`, the ledger's list) and the
  shell-lint gate's file list.

### Out (explicitly deferred)
- Marley's own app id, icon, URL scheme, keyring label and updater (#445). Until then the
  desktop entry uses Zed's dev-channel icon under the name `marley`.
- Packages for other machines (a tarball, the AUR, Flatpak) and Zed's `cli` companion binary.
- A Hyprland key binding: Chad's config, through the omarchy skill, if he wants one.

## Reference (§20)
Upstream Zed's own local install on Linux: `script/install-linux` runs `script/bundle-linux`
(a release build of `zed` and `cli` bundled as `zed<suffix>.app`) and `script/install.sh`
(the bundle under `~/.local`, a `zed` link in `~/.local/bin`, the desktop entry filled from
`crates/zed/resources/zed.desktop.in`). Marley keeps the shape (a release build, `~/.local`, a
desktop entry with the same fields) and drops the bundle, which exists for Zed's `cli` split and
its symbol upload. Warp: N/A, packaging.

### Prior art
- **Code we already ship.** Zed's three scripts above. Reusing `install-linux` as it stands
  is wrong here: it builds `zed` and `cli` rather than `marley`, names the app Zed Dev, and links
  `~/.local/bin/zed`, which on this box already points at Chad's installed Zed.
  `crates/zed/build.rs` stamps `ZED_COMMIT_SHA` from git, and the startup log line carries it,
  which is how a run proves which commit it is. The root `Cargo.toml`'s `[profile.release]`
  (thin LTO, one codegen unit, sixteen for the `zed` package). `main.rs` skips the single-instance
  socket on the `dev` channel, so two Marleys on one data directory would both run (and the
  second hangs, L-claude-438): the script warns rather than pretending to prevent it.
- **How this box launches an entry.** Omarchy's app library runs `uwsm-app -- gtk-launch
  <id>.desktop` (`/usr/share/omarchy/shell/services/AppLibrary.qml`), a scope under
  `app-graphical.slice`; `uwsm-app` evaluates the command the app daemon returns in the caller's
  own environment. Marley has twice exited silently when started through Hyprland's `exec`
  without `uwsm-app` (L-claude-460, and again on 2026-09-25), so the menu's own path is tested.
- **The house pattern on this box.** Rusty's `omarchy/install.sh`: idempotent, `install -D` for
  the desktop entry and icon, `update-desktop-database` when present, a `sed` of the entry's
  `Exec` to the installed path.
- **Published material.** The freedesktop Desktop Entry spec (`Exec` field codes, `Icon` by
  theme name, `StartupWMClass`); `desktop-file-validate` from desktop-file-utils, installed here.

## UI proof
UI-AFFECTING (the launcher gains Marley; the installed binary is what runs).
`script/e2e/502-installed-release-build.sh` (`compositor sway`): setup installs into a scratch
prefix under `$E2E_WORK`, validates the entry with `desktop-file-validate`, and names the entry's
`Exec` binary as the one the run starts (`binary`). Steps: the installed build open on the
scratch repository (`502-01-installed`); `script/install-marley --prefix` again while it runs,
then the same window, still drawing (`502-02-reinstalled`); the installed entry started the
way Omarchy's menu starts one, `uwsm-app -- gtk-launch marley.desktop`, with the scenario's
sway as its display and a scratch `XDG_DATA_HOME` and `XDG_CONFIG_HOME` (`502-03-from-the-menu`).
Last, the launcher with a flag Marley refuses, whose error must land in the scratch
`logs/stderr.log`. The run starts the binary rather than the launcher, whose log would otherwise
land in Chad's data directory. The run log carries the startup
line's commit, which must equal `HEAD`. Chad's own `~/.local` is installed by a separate run of
`just install` at Test, since the scenario never writes outside its scratch folder.

## Locked-In Decisions
- D1 — A copy, not a link into the target directory: the installed build survives
  `cargo clean`, a failed build and a checkout of another branch.
- D2 — Replace by rename, never by writing into the old file: a running Marley keeps its inode,
  and writing into a running executable fails anyway (`ETXTBSY`).
- D3 — The `dev` channel stays (#445 owns Marley's identity): the installed and the debug builds
  share `~/.config/marley` and `~/.local/share/marley`, so Chad's settings, sessions and database
  come along. The price is that nothing stops both from running at once; the script warns.
- D4 — The release profile as Zed defines it, not `release-fast`: `release-fast` keeps full debug
  info and no LTO, which makes a bigger and slower binary for a build that is meant to be used.
  The binary is installed as built, 2.1 GB with `debug = "limited"` (407 MB stripped): the debug
  sections are never loaded, and they keep file and line numbers in a panic's backtrace.
- D5 — A launcher keeps stderr (added at Test). The `dev` channel installs no crash handler, so
  a panic reaches stderr and nothing else, and a Marley the menu starts has no terminal. The
  launcher sends stderr to `logs/stderr.log` beside `Marley.log` when it is not a terminal, and
  keeps one earlier log past 10 MiB. Test found why this matters: the first menu start panicked
  (#512) and left nothing in `Marley.log`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `just install` runs, the system shall build `marley` in the release profile and install it at `<prefix>/bin/marley`, printing the commit it installed. | The scenario's install log; Chad's `just install` run |
| REQ-002 | WHEN the install completes, `<prefix>/share/applications/marley.desktop` shall name the installed binary and pass `desktop-file-validate`. | The scenario's setup log |
| REQ-003 | WHEN the installed binary starts, it shall open in the Marley layout on Marley's settings and data, at the commit installed. | Shot `502-01-installed`; the run log's startup line |
| REQ-004 | WHILE a Marley started from the installed binary runs, a second install shall replace the binary without an error, and the running Marley shall keep running. | Shot `502-02-reinstalled`; the install's exit status |
| REQ-005 | WHEN the desktop entry is started the way Omarchy's menu starts entries, Marley shall open a window and keep running. | Shot `502-03-from-the-menu` |
| REQ-006 | WHEN the installed Marley writes to stderr that is not a terminal, the launcher shall append it to `logs/stderr.log` in Marley's data directory. | The run log: the launcher's log after a refused flag |

## Phase Plan
- **P1 Plan** — this spec; the design and the test plan in the notes.
- **P2 Code** — the script, the recipe, `binary` in `script/e2e.sh`, the owned set and the lint
  list; shellcheck clean; a review of the diff.
- **P3 Test** — the scenario and every shot; Chad's real install; `script/gates.sh --fast` (no
  Rust changes).
- **P4 Complete** — CHANGELOG, the architecture docs, knowledge, close, archive, commit.
