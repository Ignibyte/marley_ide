---
pipeline_id: 12b9067b-bda2-44e8-8c08-8ed646a7ec9c
ticket: docs/planning/tickets/closed/TICKET-445-marley-release-identity.md
status: Phase 4 — Complete PASS
title: Marley's own remote server - the install stages it, the client uploads it, both ends match by commit
type: feature
slice: packaging, the release identity (TICKET-445), slice 1 of 5; M1 of docs/planning/design-notes/harness-and-zed-remote-2026-10-11.md
references:
  - docs/planning/design-notes/harness-and-zed-remote-2026-10-11.md
  - docs/planning/design-notes/finish-line-2026-10-11.md
  - docs/planning/pipeline/completed/437-marley-app-identity.spec.md
  - docs/planning/pipeline/completed/502-installed-release-build.spec.md
---

## Title
Marley opens SSH projects and dev containers. It ships on Zed's `dev` channel, where Zed's client
neither downloads a remote server nor checks one, and a release build stops with
"ZED_BUILD_REMOTE_SERVER is not set and no remote server exists at …"
(`crates/remote/src/transport/ssh.rs:1077-1086`, `crates/remote/src/transport/docker.rs:278-287`).
This slice gives Marley a server of its own: the install builds and stages a matching Linux server,
the client uploads it when the host lacks it, and the client uses a server on the host only when it
answers Marley's commit. Chad, 2026-10-11, answering "Marley's own identity: its own app id,
keyring name, `marley://` links and updater … it also unblocks building Marley's remote server, so
SSH projects work": "7.) yes". It is the first of #445's five slices (Out).

## Scope
### In
- **The name.** On the `dev` channel the host's server is
  `~/.zed_server/zed-remote-server-dev-<commit>`, the full commit Marley was built from, in place of
  `-dev-build` (`ssh.rs:1021-1036`, `docker.rs:219-231`). A build with no commit keeps `build`.
- **The check.** On `dev` the client reads what `<server> version` prints (the server's commit,
  `crates/remote_server/src/server.rs:121-138`) and uses the server only when that output ends
  with Marley's commit. Anything else counts as missing. Other channels keep Zed's check that the
  file runs.
- **The upload.** On `dev` a missing server is the one staged at
  `<data dir>/remote_servers/dev/<os>-<arch>/<commit>.gz`, the layout Zed's own downloads use
  (`crates/auto_update/src/auto_update.rs:626-629`), uploaded and unpacked by Zed's existing code
  (sftp, then scp; `gunzip`, `chmod`, `mv`). With nothing staged, the connection fails with a
  message that names the host's platform, Marley's commit and
  `script/install-marley --remote-server-only`.
- **Dev containers** get the same through the same lookup (`docker.rs`).
- **The install.** `script/install-marley` builds `remote_server` for
  `x86_64-unknown-linux-musl` in release, in its own cargo invocation after the editor's, strips
  its debug info, gzips it to the staged path under `${XDG_DATA_HOME:-$HOME/.local/share}/marley`,
  keeps the newest three staged servers, and prints the path with the dirty-tree warning it already
  prints for the editor. `--remote-server-only` stages the server and builds or installs nothing
  else.
- **Docs:** `docs/marley/guide.md` (the install section: the staged server and the new flag; remote
  projects work on Linux x86_64 hosts; for developers, `ZED_BUILD_REMOTE_SERVER=never` with a debug
  build), and three rows in `docs/marley/zed-touchpoints.md`.

### Out (explicitly deferred)
The rest of #445, each its own ticket, in this order:
1. **The app id and the channel-keyed data.** Marley's own Wayland app id and X11 class (today
   `dev.zed.Zed-Dev`, `crates/release_channel/src/lib.rs:228-235`, read by
   `crates/zed/src/zed.rs:366`, `412` and by the notification, sign-in and About windows), the
   name Marley shows for itself in place of "Zed Dev" (`lib.rs:206-213`), the desktop entry's `StartupWMClass` (`script/install-marley:127`) and
   `script/e2e.sh:59`'s class, and the single-instance socket name (`zed-dev.sock`,
   `crates/marley_workbench/src/single_instance.rs:58-60`, `crates/zed/src/zed/open_listener.rs:413`).
   Proposed default: stay on `dev` and change what Marley's `dev` build calls itself, so the
   database (`db/0-dev`), the `"dev"` settings block and the credentials file do not move and the
   `RELEASE_CHANNEL` guard test stays. A move off `dev` would migrate those four instead; the
   directories themselves are Marley's since #437.
2. **The keyring label.** Marley's own Secret Service label in place of the fixed
   `zed-github-account` (`crates/gpui_linux/src/linux/platform.rs:57`), System One's keys included,
   which today reach the Secret Service directly even on `dev`
   (`crates/marley_workbench/src/system_one.rs:501`, `546`, `560`).
3. **The `marley://` scheme and the desktop entry.** `x-scheme-handler/marley` in `marley.desktop`
   (today it registers no scheme, `script/install-marley:114`), `marley://` parsed where `zed://` is
   (`open_listener.rs:157-211`, `single_instance.rs:23`), and the `zed://` links Marley opens
   handled in Marley rather than sent through `xdg-open` to stock Zed
   (`crates/client/src/client.rs:1949-1988`, `crates/editor/src/navigation.rs:1158-1165`). The verbs
   of `marley <command>` (#749) ride the single-instance hand-off, which needs no desktop scheme; if
   #749 lands first and names its URLs `marley://`, this slice adds only the desktop's half.
4. **The updater.** Off for good with Marley's own explanation (`ZED_UPDATE_EXPLANATION`, read by
   `auto_update.rs:281-287` and `310-343`), or Marley's own update source later; a Zed CLI, if
   Marley ever ships one, built with `no-bundled-uninstall` (`crates/cli/Cargo.toml:21`).
- macOS hosts (a server built on a Mac from the same commit; this box has no macOS SDK), aarch64
  Linux hosts, and WSL (`wsl.rs:215-280` keeps `-dev-build`; Marley runs on no Windows).
- The research note's M2 and M3: #758 makes harness threads open from a remote project's window;
  New Agent's default to the host's harness and one host picker are not ticketed here.
- A protocol version on the wire. Zed has none; the commit match stands in for it.

## Reference (§20)
Upstream Zed, the `remote` and `auto_update` crates: on the stable and preview channels the client
probes `~/.zed_server/zed-remote-server-<channel>-<version> version` over its SSH master, and when the
server is missing it fetches the matching build and uploads it (sftp, then scp), then unpacks it in
place (`ssh.rs:1014-1145`, `1247-1338`, `1426-1479`). Behavior kept: the location `~/.zed_server`,
the uploader, the unpacking, the proxy and the daemon, and `upload_binary_over_ssh`'s local path.
Marley supplies the file on `dev` from its own install instead of zed.dev, through the function
Zed's comment names for packagers (`auto_update.rs:596-598`), and names the server by commit as
Zed's container transport already does for Nightly (`docker.rs:219-223`).

### Prior art
- **The code we ship:**
  - `SshRemoteConnection::ensure_server_binary` (`ssh.rs:1014-1145`): the name (1021-1036), the
    probe whose output is dropped (1038-1047), the debug-only build from source (1049-1071), the
    `dev` bail (1077-1086), the download on the host and the local download and upload
    (1088-1145); `upload_local_server_binary` and `extract_server_binary_posix` (1247-1338). Its
    container twin `docker.rs:206-330` already takes the commit (`commit: Option<AppCommitSha>`)
    and uses it in Nightly's name (219-223).
  - `AutoUpdater::download_remote_server_release` and `get_remote_server_release_url`
    (`auto_update.rs:596-675`), with Zed's comment at 596-598: "If you are packaging Zed and need
    to override the place it downloads SSH remotes from, you can override this function. You
    should also update get_remote_server_release_url to return Ok(None)." Both client delegates
    call them (`crates/remote_connection/src/remote_connection.rs:488-542`, `655-700`), so one
    lookup serves SSH and containers.
  - `remote_server version` prints the commit on `dev` (`server.rs:121-138`). Both build scripts
    bake `ZED_COMMIT_SHA` from `git rev-parse HEAD` and rerun when `.git/logs/HEAD` changes
    (`crates/zed/build.rs:43`, `49-71`; `crates/remote_server/build.rs:20-32`); the editor holds it
    as `AppCommitSha` (`crates/release_channel/src/lib.rs:56-87`, set at `crates/zed/src/main.rs:317`,
    `515`).
  - `cleanup_old_binaries` (`server.rs:1428-1446`) deletes every `dev` server on the host whose
    suffix is no version and which is not running, so servers named by commit clean themselves up.
  - `build_remote_server_from_source` (`crates/remote/src/transport.rs:261-296`), compiled into
    debug builds only: without `ZED_BUILD_REMOTE_SERVER=never` a debug Marley runs cargo when it
    connects.
  - `script/bundle-linux`: the musl triple (99-103), the server in its own invocation with
    `CC_x86_64_unknown_linux_musl=musl-gcc` (131-137), `llvm-objcopy --strip-debug --discard-all`
    (193), `gzip --best` (254). `rust-toolchain.toml` already lists `x86_64-unknown-linux-musl`
    for the remote server; `musl-gcc` and `llvm-objcopy` are on the box.
- **Behavior maps:** the research note's sections 1 and 3 a′
  (`docs/planning/design-notes/harness-and-zed-remote-2026-10-11.md`), which this slice builds as
  M1; `docs/zed_architecture/subsystems/06-project-fs-search.md` names `remote(_server)` only as a
  backend. Nothing in the Warp or Orca maps.
- **Published material:** Zed's remote development docs (`docs/src/remote-development.md:222-243`):
  the probe, the upload, and the musl, `crt-static`, stripped build for a server you maintain
  yourself.

## UI proof
`script/e2e/445-marley-release-identity.sh`, on the debug build, with two fake hosts and the server
the Test phase staged.

Set-up:
- Two user-level sshds on loopback ports (L-claude-584's recipe, as #596 runs one). Each one's
  `ForceCommand` runs every command, sftp's server included, in its own fake host home under
  `$E2E_WORK`, with `HOME` and the XDG directories pointing there, so nothing reaches the user's
  `~/.zed_server` or `~/.local/share/marley`.
- `ssh`, `sftp` and `scp` wrappers first on Marley's PATH add `-F /dev/null`, the scenario's key and
  known-hosts file, `IdentitiesOnly=yes` and `IdentityAgent=none`, so `~/.ssh` is neither read nor
  written.
- `ZED_BUILD_REMOTE_SERVER=never`, so the debug build compiles nothing when it connects.
- `ssh_connections` names both hosts. Each fake home holds a folder with two files. The second
  host also holds a stand-in at Marley's server name whose `version` prints another commit.
- `E2E_REMOTE_SERVER` names the `.gz` that `script/install-marley --remote-server-only` staged into
  the scratchpad from the same commit; the scenario stops with a message when it is missing.

Shots:
- `445-01-not-staged`: nothing staged in the run's profile; the first host opened through an
  `ssh://` URL handed to the running Marley: the connection's error names the platform, the commit
  and `script/install-marley --remote-server-only`.
- `445-02-opened`: the staged server copied into the profile's
  `remote_servers/dev/linux-x86_64/`, the first host opened again: the remote project in the window,
  its two files in the project panel. Check: the host's `~/.zed_server` holds
  `zed-remote-server-dev-<commit>`, whose `version` prints the commit, and no `-dev-build`.
- `445-03-replaced`: the second host opened: the project opens. Check: the file at Marley's name
  now prints Marley's commit, not the stand-in's.

## Locked-In Decisions
- **D1 — The `dev` server is named for Marley's commit** (`zed-remote-server-dev-<full commit>`), as
  the container transport names Nightly's. A stale server never matches by name, Marley stops
  sharing `-dev-build` with every other Zed dev build, and the host's cleanup removes old ones.
  Rejected: keep `-dev-build` and compare only `version` (two Marleys of different commits on one
  host would replace each other's server at every connect).
- **D2 — The two ends check their commit.** On `dev` the client compares `version`'s output with
  Marley's commit and treats a mismatch as missing; the probe already runs, so the check adds no
  round trip. Other channels keep Zed's check. Rejected: a protocol version in the wire (Zed has
  none; a change to `proto` and both ends for nothing a commit match misses).
- **D3 — The file comes through Zed's packager seam.** On `dev`, `get_remote_server_release_url`
  returns `Ok(None)` and `download_remote_server_release` returns the staged file or the error that
  names the install step; `ensure_server_binary`'s `dev` arm stops bailing in `ssh.rs` and
  `docker.rs`, so Zed's own upload path runs. One lookup serves both transports. Rejected: the
  upload written into each transport's `dev` arm (the same lookup twice, and more Zed lines);
  `build-remote-server-binary` turned on in release (it uploads on every connect and falls back to
  cargo; the research note's trap).
- **D4 — The staged layout is Zed's download cache**, `<data dir>/remote_servers/dev/<os>-<arch>/<commit>.gz`.
  `--user-data-dir` moves it with the rest of the data, so a scenario stages into its own profile.
- **D5 — Every install builds the server** for Linux x86_64 (a default; the research note had
  proposed "only when asked"): musl, in its own cargo
  invocation after the editor's (feature unification stays out, `bundle-linux:129-130`), stripped
  and gzipped as `bundle-linux` does, the newest three kept. `--remote-server-only` stages without
  touching the installed editor. Rejected: a build when Marley connects (a second cargo on a box
  that runs one at a time).
- **D6 — Linux x86_64 hosts only for now**; macOS hosts later (default). Any other platform gets the
  same error, naming its platform.

Open for Chad, with the defaults written above: does every install build the server (D5: yes), and
do macOS hosts wait (D6: yes)? For slice 2: does Marley stay on `dev` and change only what its build
calls itself (Out 1: yes)?

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley connects to an SSH host whose `~/.zed_server` lacks Marley's server, the system shall upload the server staged for that host's platform and Marley's commit, start it and open the project. | Shot `445-02-opened` and its host check |
| REQ-002 | The server Marley places on a host shall be named `zed-remote-server-dev-<commit>` for the commit Marley was built from. | The check after `445-02-opened` |
| REQ-003 | WHEN the server at Marley's name answers `version` with another commit, the system shall replace it with the staged server before starting it. | Shot `445-03-replaced` and its host check |
| REQ-004 | IF no server is staged for the host's platform and Marley's commit, THEN the system shall fail the connection with a message naming the platform, the commit and `script/install-marley --remote-server-only`. | Shot `445-01-not-staged` |
| REQ-005 | WHEN `script/install-marley` runs, it shall build `remote_server` for `x86_64-unknown-linux-musl` from the same tree and stage it stripped and gzipped at `<data dir>/remote_servers/dev/linux-x86_64/<commit>.gz`, keeping the newest three. | The Test phase's staging run into the scratchpad: the file, `file` on the unpacked server (static x86-64), its `version` (HEAD's commit) |
| REQ-006 | WHEN `script/install-marley --remote-server-only` runs, it shall stage the server and leave the installed editor, launcher and desktop entry unchanged. | The same run: the installed files' mtimes before and after |
| REQ-007 | WHEN Marley opens a dev container, the system shall supply and check the server as it does for an SSH host. | Review of the diff (no container runtime in a scenario) |
| REQ-008 | The change shall pass `script/gates.sh --diff`. | The gate's exit code |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes, every cited seam re-read.
- **P2 Code** — the ledger rows first (`ssh.rs`, `docker.rs`, `auto_update.rs`); the name and the
  check in both transports; the staged lookup in `auto_update.rs`; `script/install-marley`; the
  guide; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — stage the server once with `script/install-marley --remote-server-only` into the
  scratchpad (one cargo, nothing else running); then write and run the scenario and read every shot.
- **P4 Complete** — CHANGELOG and the guide (§21), ledger capture (§19), the ticket closed and the
  next slice's ticket asked of the parent session, archive, commit.
