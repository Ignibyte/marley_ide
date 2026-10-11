# Marley's own remote server - the install stages it, the client uploads it, both ends match by commit — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-445-marley-release-identity.md
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
