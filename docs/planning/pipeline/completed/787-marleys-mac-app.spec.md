---
pipeline_id: dd7856ae-07de-4b1b-976b-752e5537470a
ticket: docs/planning/tickets/closed/TICKET-787-marleys-mac-app.md
status: Phase 4 — Complete PASS
title: "Marley's Mac app and its install"
type: chore
slice: macOS
references: [docs/planning/pipeline/completed/786-marley-compiles-on-macos.spec.md, docs/planning/pipeline/completed/502-installed-release-build.spec.md, docs/planning/pipeline/completed/445-marley-release-identity.spec.md]
---

## Title
Marley compiles for macOS since #786, but nothing installs it there. `script/install-marley` is
Linux-only: it writes a desktop entry, builds a musl remote server and uses GNU-only flags.
`script/bundle-mac` is Zed's release tool: it builds `zed`, signs with Zed's identity and takes
`dev.zed.Zed-Dev` from Zed's bundle metadata. Chad's desk moves to an Apple Silicon Mac with the
Linux box as the engine (2026-10-16, "lets make it happen"). `just install` on the Mac gives him
`Marley.app`, which opens the box's projects over SSH.

## Scope
### In
- `script/install-marley`, a macOS path chosen by `uname -s`:
  - the release build, sized for the machine;
  - a `Marley.app` the script assembles: Info.plist, binary, icon, ad hoc signature;
  - the app put in place, a `marley` launcher on the prefix's `bin`;
  - the Linux remote server for the same commit, fetched from the host that staged it;
  - `--remote-server-only` fetches the server and does nothing else, `--regress` is refused,
    and the Linux path is unchanged.
- `justfile`: nothing, since `just install` already calls the script.

### Out (explicitly deferred)
- Any change to Zed's `script/bundle-mac`, `crates/zed/Cargo.toml` bundle metadata or
  `crates/zed/resources/`. They are read, not edited.
- A Developer ID signature, notarization, a DMG, auto-update.
- A macOS keymap, `marley <verb>` on macOS, and the `/proc` and systemd features (#786's list).
- Building the musl remote server on the Mac.

## Reference (§20)
Upstream Zed's macOS app, from `script/bundle-mac` and `crates/zed/Cargo.toml`
(`package.metadata.bundle-dev`, `resources/info/*.plist`):
- a bundle whose Info.plist names the app, its identifier and its URL schemes;
- the privacy usage strings (`Permissions.plist`), which macOS needs before a program in its
  terminal may ask for the camera, the microphone or AppleEvents;
- `CFBundleSupportedPlatforms`;
- a signature, ad hoc (`--sign -`) when no certificate is present, as `bundle-mac` does locally.

Marley keeps all of that with its own name, `com.ignibyte.Marley` (#760) and only the `marley`
scheme (#762). It drops `DocumentTypes.plist`, since Marley claims no file types (#762), along
with Zed's signing identity, notarization and the DMG.

### Prior art
- **The code we ship:**
  - `script/bundle-mac` with Zed's fork of `cargo-bundle` owns bundling, but only from
    `crates/zed/Cargo.toml`'s metadata (`dev.zed.Zed-Dev`, "Zed Dev", the `zed` scheme) and for a
    binary named `zed`. Using it would mean editing Zed's manifest and installing that fork, and
    the script's signing and notarization arms are Zed's. Its resources are reused as files:
    `resources/info/Permissions.plist`, `SupportedPlatforms.plist` and `app-icon-dev@2x.png`
    (1024×1024).
  - `crates/paths` puts the remote server cache at `~/Library/Application Support/Marley/
    remote_servers` on macOS (`remote_servers_dir`), and `auto_update.rs:1029` looks there for
    `dev/<os>-<arch>/<commit>.gz` (#445).
  - Zed's `main` loads the login shell's environment for a launch that is not from a terminal
    (`util::load_login_shell_environment`), so a Dock launch finds `claude` and `codex`.
- **macOS's own tools:** `sips` and `iconutil` (the `.icns`), `/usr/libexec/PlistBuddy`,
  `plutil -lint`, `codesign`, and `lsregister` through LaunchServices on first open.
- **Behavior maps:** nothing applies.

## UI proof
The check runs on the Mac, which no `script/e2e.sh` scenario reaches: the scenarios need Hyprland
or sway. It is a run of `just install` there and of the installed app:
- `787-mac-app.png`: Marley's window on the Mac with "Marley" in the menu bar. It is taken with
  `screencapture` from a process that holds the Screen Recording grant. When none does, the proof
  falls back to System Events' process and window and is recorded as such.
- `787-mac-remote.png`: the box's `marley_ide` open as a remote project in the Mac's Marley, with
  the box's own record of its server.

The Linux path's run is `just shot`, since nothing new is seen on Linux.

## Locked-In Decisions
- D1 — The Mac path lives in `script/install-marley`, so `just install` stays the one way in.
  The script writes the bundle itself; `bundle-mac` and `cargo-bundle` are not used, and no Zed
  file changes.
- D2 — The app goes to `~/Applications/Marley.app` (`--app-dir DIR` moves it; no sudo). The
  launcher `<prefix>/bin/marley` runs the bundle's binary and sends stderr to
  `~/Library/Logs/Marley/stderr.log` when it is not a terminal, as Linux's does to its log.
- D3 — On a machine with under 32 GB, the build turns the release profile's thin LTO off, uses 16
  codegen units, caps the jobs at a third of the gigabytes (5 on 16 GB) unless
  `CARGO_BUILD_JOBS` is set, runs under `nice`, and says so. Thin LTO with one codegen unit per
  crate on 16 GB, beside Chad's sessions, is the freeze risk in the Mac's history. The
  environment's own `CARGO_PROFILE_RELEASE_*` values win.
- D4 — `--remote-server-from HOST` copies `HOST`'s staged server for this commit, checked with
  `gzip -t`. The script looks for it before the build, so a missing server fails in seconds, not
  after an hour, and the message says what to run on the host. Without the option, the install
  says remote projects need it.
- D5 — The new bundle replaces the old by rename, so a running Marley keeps running. The
  install says a quit and reopen picks the new one up.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `just install` runs on the Mac, it shall leave `~/Applications/Marley.app` whose Info.plist names `com.ignibyte.Marley`, the name Marley, the executable `marley` and only the `marley` URL scheme, with no document types, signed ad hoc | `PlistBuddy` and `codesign -dv` on the installed bundle |
| REQ-002 | WHEN `Marley.app` is opened on the Mac, Marley shall start and show a window, with Marley as the app's name | `787-mac-app.png`; System Events' process name and window count |
| REQ-003 | WHEN the install runs with `--remote-server-from <host>` and the host staged the server for the commit, it shall place that server at `~/Library/Application Support/Marley/remote_servers/dev/linux-x86_64/<commit>.gz` | `ls` and `gzip -t` on the Mac |
| REQ-004 | WHEN the Mac's Marley opens a folder on the Linux box over SSH, it shall open the folder through that server | `787-mac-remote.png`; on the box, `~/.zed_server/zed-remote-server-dev-<commit>` and its process |
| REQ-005 | WHILE the machine has under 32 GB, the install shall build without thin LTO and with capped jobs, and say so | the install's log |
| REQ-006 | WHEN the install runs with `--remote-server-from` and the host has no server for the commit, it shall stop before building and name the command to run there | a run against a commit the host never staged |
| REQ-007 | WHEN `script/install-marley` runs on Linux, it shall do what it did before | the review of the diff; gate:11 shellcheck; the box's `--remote-server-only` run that stages the server for REQ-003 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the macOS path; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — on the box, `--remote-server-only` at the commit; on the Mac, `just install
  --remote-server-from` the box; the app opened, a remote folder opened, each shot read.
- **P4 Complete** — CHANGELOG, the guide's install section for macOS, the plan's macOS bullet
  (§21), ledger capture (§19), close the ticket, archive, commit.
