# Marley's own remote server - the install stages it, the client uploads it, both ends match by commit — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-445-marley-release-identity.md
- **Pipeline spec:** 445-marley-release-identity.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11, the finish-line list (`docs/planning/design-notes/finish-line-2026-10-11.md`,
  row 7), answering "Marley's own identity: its own app id, keyring name, `marley://` links and
  updater … it also unblocks building Marley's remote server, so SSH projects work": "7.) yes". The
  hold of 2026-10-01 ("wait on 445 that way if there is any bugs we wont have to cut multiple
  releases") is lifted; #445 is built last of the build-now items, as the release step. The first
  slice is the research note's first recommendation (M1,
  `docs/planning/design-notes/harness-and-zed-remote-2026-10-11.md`).
- **Classification:** feature, packaging. Three small Zed hunks (`remote` twice, `auto_update`), one
  Marley-owned script. Size M.
- **The split.** #445 as written is five slices; this spec is the first. The other four are
  described in the spec's Out, in build order: the app id and the channel-keyed data, the keyring
  label, the `marley://` scheme and the desktop entry, the updater. The parent session mints their
  tickets.
- **Recall (§18.3):**
  - AD-claude-437-marley-identity-is-app-name-and-the-dev-channel-001 and
    PR-claude-a-zed-fork-identity-is-more-than-app-name-001: the identity beyond `APP_NAME` is keyed
    by the channel, which stays `dev` behind the `paths` guard test (`crates/paths/src/paths.rs:657-660`).
    This slice keeps `dev`.
  - #437's inspect, finding 6: "`~/.zed_server` on SSH hosts is shared with any upstream Zed dev
    build", accepted then; D1 settles it.
  - AD-claude-502-the-installed-marley-is-a-copy-and-a-launcher-001 and
    L-claude-502-the-release-build-on-the-dev-box-001: the install is a release build (about ten
    minutes on a warm target); a second target adds its own.
  - L-claude-584-a-scenario-runs-an-sshd-of-its-own-001: an unprivileged sshd on loopback with its
    own keys, and a client that reads nothing of `~/.ssh`. A remote command there runs in the
    account's login shell, in the real home, which is why this scenario forces a fake home.
  - The research note's traps: the debug build compiles the server at connect unless
    `ZED_BUILD_REMOTE_SERVER=never`; `build-remote-server-binary` is no fix in release; nothing
    compares the two ends today.
  - The memory note on `/mnt/fast` filling up: the musl target adds a second release tree to the
    shared target directory.
  - Brain: not consulted (`/spec` drafts; the promotion runs the loop).
- **Discovery:**
  - `crates/remote/src/transport/ssh.rs`: `ensure_server_binary` 1014-1145 (name 1021-1036; probe
    1038-1047, its output dropped with `.is_ok()`; the debug build from source 1049-1071; the `dev`
    bail 1077-1086; download and upload 1088-1145); `upload_local_server_binary` 1247-1287;
    `extract_server_binary_posix` 1305-1338; `upload_file` 1426-1479; `run_command` returns stdout
    (1568-1580).
  - `crates/remote/src/transport/docker.rs`: `ensure_server_binary` 206-330 (it receives the
    commit; Nightly's name 219-223; the `dev` bail 278-287).
  - `crates/auto_update/src/auto_update.rs`: `download_remote_server_release` 596-654 (cache
    layout 626-629, cleanup 643-651), `get_remote_server_release_url` 656-675; both start by reading
    the `GlobalAutoUpdate`, so the `dev` branch goes first.
  - `crates/remote_connection/src/remote_connection.rs`: both delegates call the two (488-542,
    655-700).
  - `crates/remote_server/src/server.rs`: `version` 121-138; `cleanup_old_binaries` 1428-1446.
  - `crates/release_channel/src/lib.rs`: `AppCommitSha` 56-87.
  - `crates/paths/src/paths.rs`: `remote_server_dir_relative` 69-74 (`.zed_server`),
    `remote_servers_dir` 476-479 (`data_dir()/remote_servers`).
  - `script/install-marley`: the editor build 75; the dirty-tree warning 135-140.
    `script/bundle-linux` 99-103, 131-137, 193, 254.
  - `script/e2e.sh`: `--user-data-dir "$E2E_PROFILE"` (301, 304); the profile copies settings, `db`
    and `threads` only (623-705), so the scenario stages the server into it.
  - `crates/marley_workbench/src/single_instance.rs:23`: the hand-off accepts `ssh` URLs, so a
    second launch can open a host in the running Marley.

### Design (draft; confirmed at promotion)
- **`ssh.rs` (Zed crate).** In `ensure_server_binary`, on `dev` with a commit: the name uses the
  commit; the probe keeps the output and the server counts as present only when it ends with the
  commit; the `dev` arm of `wanted_version` becomes `Ok(None)` like Nightly's, so the path goes on to
  `get_download_url` (which returns `None` on `dev`) and `download_server_binary_locally`, then
  Zed's upload and unpacking. The output may carry a `\r` from the PTY; it is trimmed. Each hunk
  carries a `// Marley:` line.
- **`docker.rs` (Zed crate).** The same three changes with the commit it already receives.
- **`auto_update.rs` (Zed crate).** At the top of `get_remote_server_release_url`: `Ok(None)` on
  `dev`. At the top of `download_remote_server_release`: on `dev`, the staged
  `remote_servers_dir()/dev/<os>-<arch>/<commit>.gz` when it exists, else an error naming the
  platform, the short commit and `script/install-marley --remote-server-only`.
- **`script/install-marley` (Marley-owned).** After the editor's build: `rustup target add` only if
  the target is missing; `CC_x86_64_unknown_linux_musl=musl-gcc cargo build --release --target
  x86_64-unknown-linux-musl -p remote_server`; `llvm-objcopy --strip-debug --discard-all` into a
  temporary file; `gzip --best` to `<commit>.gz.new`, renamed into place; staged files beyond the
  newest three removed; the log line with the path and the dirty-tree note. `--remote-server-only`
  skips the editor's build, the golden set and the install. The header comment says what is staged
  and why.
- **Docs.** The guide's install section (56-90), its option table (2808) and command table (2908);
  a developer note on `ZED_BUILD_REMOTE_SERVER=never`.
- **File manifest:** `crates/remote/src/transport/ssh.rs`, `crates/remote/src/transport/docker.rs`,
  `crates/auto_update/src/auto_update.rs` (Zed crates); `script/install-marley`,
  `script/e2e/445-marley-release-identity.sh`, `docs/marley/guide.md`,
  `docs/marley/zed-touchpoints.md` (Marley-owned).
- **Ledger rows:** three new rows, written before the edits: `crates/remote/src/transport/ssh.rs`,
  `crates/remote/src/transport/docker.rs`, `crates/auto_update/src/auto_update.rs`. Each says what
  to keep at a merge: the commit name, the commit check, the `dev` arm and the staged lookup.

### Visual check plan
| REQ | The scenario does | Evidence |
|---|---|---|
| 004 | Opens host one with nothing staged in the profile | `445-01-not-staged`: the error naming the platform, the commit and the flag |
| 001, 002 | Copies the staged server into the profile; opens host one again | `445-02-opened`: the remote project with its two files; host one's `~/.zed_server` holds `-dev-<commit>`, whose `version` prints the commit, and no `-dev-build` |
| 003 | Opens host two, whose stand-in answers another commit | `445-03-replaced`: the project open; the file at that name now prints Marley's commit |
| 005, 006 | The Test phase's staging run, `XDG_DATA_HOME` in the scratchpad | The staged path, `file` on the unpacked server, its `version`; four fake older files cut to three; the installed editor's mtimes unchanged |
| 007 | — | Review of the diff |

- **Checks every run:** the user's `~/.ssh`, `~/.zed_server` and `~/.local/share/marley/server_state`
  keep their mtimes (where they exist); the sshd logs show only the scenario's key; the teardown
  kills the servers started under the fake homes (a daemon idles ten minutes otherwise) and both
  sshds.
- **Out of a scenario's reach:** a dev container (no container runtime in a run), a real second
  machine, and a non-Linux host. The container path is the same lookup, reviewed in the diff.

### Risks
- **The first install after this lands** compiles the server's dependency graph for a second
  target: several more minutes and several more GB on `/mnt/fast`. Later installs rebuild only
  what changed.
- **A Marley crate in the server's graph** (`project` depends on `marley_terminal`, which takes
  `marley_dcs` and `rustix`) must build for musl. The first staging run shows it; a failure there
  is fixed in that crate, not worked around in the script.
- **A dirty tree** gives the editor and the server the same commit name over possibly different
  code. The install already says the tree is dirty; the staging line says it again.
- **The probe's output** passes through a PTY (`run_command(…, true)`), so it may end in `\r\n`.
- **The scenario's sshd** must route sftp's server through the `ForceCommand` too, or Zed's upload
  lands in the real home; Zed falls back to scp, which the wrapper also catches.
- **Two Marleys of different commits on one host** each keep their own server; the host's cleanup
  deletes the one not running at the next start, and the other re-uploads at its next connect.
- **#758** says a remote project's window is out of a scenario's reach; once this slice lands it
  is not, if #758 is built after it.

### Promotion (2026-10-11)
- **Promoted** from `queued/` to `active/` under the goal "continue until all tickets are
  finished"; the BACKLOG row removed; the ticket `in-progress`, its link on `active/`. The open
  questions for Chad (D5, D6, slice 2's channel) take the written defaults, as the session runs
  the queue on defaults.
- **Brain:** consultation ae6b7024268d48188085752a6a0bf544. Nothing on this seam came back, only
  the due follow-ups. A second consultation asked by mistake (27d15e43…) is closed with no
  decision.
- **Seams checked again** (an Explore pass at `6594e7f7fa`). Every cited range holds within a line
  or two. What the design takes from it:
  - **ssh.rs has no commit.** `SshRemoteConnection::new` reads the channel and version
    (1004-1005) and `ensure_server_binary` (1014-1020) takes no commit; it reads
    `AppCommitSha::try_global(cx)` itself, as docker.rs does at 85-91. `AppCommitSha` is set only
    when the build had `ZED_COMMIT_SHA` (`main.rs:526-528`); `crates/zed/build.rs:49-71` sets it
    from `git rev-parse HEAD`. A `dev` build with no commit keeps `build` and Zed's bail.
  - **The probe** (1038-1047) keeps `run_command`'s `Result<String>` (stdout, 1568-1584) and,
    on `dev` with a commit, counts the server only when the trimmed output ends with the full
    commit. docker.rs's probe is `run_docker_exec(…)`, also a `Result<String>`.
  - **The release build compiles the build-from-source block out**
    (`cfg(any(debug_assertions, feature = "build-remote-server-binary"))`, 1049-1071), so on
    `dev` it reaches the bail at 1077-1086. In a debug build `ZED_BUILD_REMOTE_SERVER=never`
    skips it (`transport.rs:262-295`).
  - **The delegates** (`remote_connection.rs:488-537`, `639-704`) pass the channel, the version,
    `os` and `arch` as strings (`RemoteOs::as_str` "linux", `RemoteArch::as_str` "x86_64"); the
    staged lookup in `auto_update.rs` reads the commit through its own `cx`.
  - **`download_remote_server_release`** starts with the `GlobalAutoUpdate` read (599-613); the
    `dev` branch goes before it. Zed's cache keeps five per platform
    (`REMOTE_SERVER_CACHE_LIMIT`, 49); the install keeps three of its own.
  - **The host's cleanup** (`server.rs:1428-1447`, at each server start) deletes every
    `zed-remote-server-dev-*` not running, since no commit parses as a newer semver: servers
    named by commit clean themselves up, as D1 says.
  - **`remote_server`'s commit** comes from `git rev-parse HEAD` alone
    (`crates/remote_server/build.rs:20-32`), so a dirty tree names the clean commit at both ends.
  - **The scenario's sshds:** 584's `start_sshd` (55-95), 596's and 641's set-ups; none uses
    `ForceCommand` or opens a Zed remote project. `MARLEY_SSH` reaches only the workbench, so
    Zed's transport needs `ssh`, `scp` and `sftp` wrappers first on Marley's PATH.
  - `single_instance.rs:24` accepts `ssh://`; the e2e profile copies nothing under
    `remote_servers`, so each run starts with nothing staged.
- **Pre-flight:** clean; `musl-gcc`, `llvm-objcopy` and `sshd` present; the
  `x86_64-unknown-linux-musl` target installed for 1.98.1; 302 GB free on `/mnt/fast`.

## Phase 2 — Code (2026-10-11)
- **Built:** the three ledger rows first (`ssh.rs`, `docker.rs`, `auto_update.rs`). `ssh.rs`:
  `AppCommitSha` imported; in `ensure_server_binary` the commit read on `dev`, the name
  `zed-remote-server-dev-<commit>`, the probe's `is_ok_and` that compares the trimmed `version`
  output with the commit, and `ReleaseChannel::Dev if commit.is_some() => Ok(None)` before Zed's
  bail. `docker.rs`: the same three with the commit it receives (`dev_commit`).
  `auto_update.rs`: `Ok(None)` at the top of `get_remote_server_release_url` on `dev`, and on
  `dev` `download_remote_server_release` returns `staged_dev_remote_server`, a new private
  function: the staged `<commit>.gz` or the error naming the platform, the short commit, the
  install step and the path it looked for. `script/install-marley`: `--remote-server-only`,
  `stage_remote_server` (tools checked, the target added through rustup when missing, the musl
  build with `bundle-config.toml` and `musl-gcc`, `llvm-objcopy`, `gzip --best`, a rename into
  place, the newest three kept, the log line with the dirty-tree note), run after the editor's
  build. The guide: what you need, the install paragraph, a new "Remote projects over SSH (#445)"
  section, the debug build's `ZED_BUILD_REMOTE_SERVER=never`, and the variables table.
- **Deviations from the design:** none in the code. The staging run moved before the gate, so a
  crate that failed to build for musl would be fixed before the gate rather than after; none did.
- **The review:** a `dev` build with no commit keeps Zed's name and bail; the probe's output may
  carry the PTY's `\r\n` and a login banner, so it is trimmed and compared with `ends_with`; the
  staged lookup runs before the `GlobalAutoUpdate` read, so it works with auto-update off; the
  error path reaches the connection modal (445-01). Each hunk carries a `// Marley:` line.
- **Clippy:** `./script/clippy -p remote -p auto_update` stops in `project`
  (`trusted_worktrees.rs:161`, `RemoteConnectionOptions::Mock` not covered): the narrow scope turns
  on `remote`'s test-support without `project`'s. Pre-existing, not in scope; the gate's scope
  (every Marley crate, `--all-features`) unifies the features and is clean.
- **The gate:** `script/gates.sh --diff` green on its first run, 17 of 17, after the Test phase's
  scenario fixes.

## Phase 3 — Test (2026-10-11)
- **The staging run** (REQ-005, REQ-006): `XDG_DATA_HOME=<scratchpad> script/install-marley
  --remote-server-only`, with four older fake `.gz` files seeded. The musl build took 9 min 2 s.
  The staged `6594e7f7fa02f723b1cdf2fb58d8e9ee73cbc375.gz` (39 MB, 114 MB unpacked) is "ELF 64-bit
  LSB pie executable, x86-64, static-pie linked"; its `version` prints that commit. Three files
  were left: the new server and the two newest fakes. The installed `lib/marley/marley`,
  `bin/marley`, desktop entry and icon kept their mtimes.
- **The scenario:** `script/e2e/445-marley-release-identity.sh`, under `compositor sway`, on the
  debug build at the same commit, with `E2E_REMOTE_SERVER` naming the staged file. Exit 0, all six
  checks passing. Shots read (the last run):
  - `445-01-not-staged`: "Failed to connect over SSH": "no Marley remote server is staged for
    linux-x86_64 at commit 6594e7f: run script/install-marley --remote-server-only in Marley's tree
    at that commit (looked for …/remote_servers/dev/linux-x86_64/6594e7f7….gz)", Retry and Cancel;
    host one received no server (REQ-004).
  - `445-02-opened`: a window on `127.0.0.1`, `project`, its Project Panel listing `host-one.md`
    and `host_one.rs`; host one's `.zed_server` holds only `zed-remote-server-dev-6594e7f7…`, whose
    `version` prints the commit (REQ-001, REQ-002).
  - `445-03-replaced`: host two's window with `host-two.md` and `host_two.rs`; the stand-in that
    answered `…0445` now answers Marley's commit (REQ-003).
  - The user's `~/.ssh`, `~/.zed_server` and `~/.local/share/marley/server_state` listings equal
    before and after.
- **Fixes on the way:** `user_listing` exits non-zero for a missing folder and the runner runs
  with errexit (`|| true`); the fake homes moved under the runtime directory after the server's
  socket paths passed 108 bytes (F-claude-445-…, L-claude-445-…); a teardown guard for an unset
  hosts folder; distinct file names per host and the Project Panel opened, so each shot shows
  which host it is.
- **What no scenario shows:** a dev container (REQ-007, no container runtime in a run), read in the
  diff: the same lookup through the same delegate.

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added, #445); the guide (what you need, the install paragraph,
  "Remote projects over SSH (#445)", the debug build's note, `ZED_BUILD_REMOTE_SERVER`); the three
  ledger rows describe what shipped; M1's row in the research note says shipped. The in-app
  guide's Install article (`crates/marley_workbench/guide/index.html`) gets the staged server and
  `--remote-server-only` with the next change to that file, since the file is in the gate's
  receipt.
- **Knowledge appended:**
  AD-claude-445-marleys-remote-server-is-staged-by-the-install-and-matched-by-commit-001;
  F-claude-445-a-fake-hosts-home-under-the-scratchpad-made-the-servers-sockets-too-long-001;
  L-claude-445-a-remote-servers-own-error-is-on-the-host-001.
- **Brain:** consultation ae6b7024268d48188085752a6a0bf544 closed with a decision.
- **Closed:** the ticket moved to `closed/` (slice 1; slices 2 to 5 are #760 to #763), the pipeline
  archived, committed.
