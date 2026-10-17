# Marley's Mac app and its install — Notes

- **Local ticket doc:** [TICKET-787](../../tickets/open/TICKET-787-marleys-mac-app.md)
- **Pipeline spec:** 787-marleys-mac-app.spec.md

## Phase 1 — Plan
- **Request:** the move of Chad's desk to the Mac (2026-10-16, "lets make it happen"). #786 made
  Marley compile there; this ticket puts it on the Mac as an app.
- **Classification / tier:** chore, macOS. `script/install-marley` only, a Marley-owned file
  (no touchpoint row). No Rust.
- **Recall (§18.3):**
  - AD-445: Marley stays on `dev` and stages its own remote server, matched by commit,
    `remote_servers/dev/linux-x86_64/<commit>.gz`. The Mac reads the same layout under its own
    data folder.
  - F-762: `Exec` quoting broke xdg-open's generic mode. The Mac's equivalent is the
    Info.plist's `CFBundleURLTypes`, with no quoting involved.
  - L-517: never edit `install-marley` while a run of it is going. That holds on both machines.
  - The old self-test rules in `prevention-rules.md` (the pre-fork Marley on macOS):
    `screencapture` needs an awake, unlocked display (`caffeinate -u` first), and `activate()`
    does not make a gpui window key.
  - The brain (consultation `2fd9cc4e875f471eb734f3201df0ae8c`): nothing on a Mac bundle of the
    fork.
- **Discovery:**
  - `script/install-marley:66-75`: arguments, then `realpath -m` (GNU) at :70. The Darwin
    branch has to come before it.
  - The Linux-only pieces:
    - `stage_remote_server` (:89-121: `musl-gcc`, `llvm-objcopy`, `find -printf`, `xargs -r`);
    - the launcher's `stat -c` (:162);
    - `install -D` (:173);
    - the desktop entry (:181-197).
  - `crates/zed/resources/info/Permissions.plist` and `SupportedPlatforms.plist` are dict entries
    without the wrapping `<dict>` that cargo-bundle concatenates. The strings say "Zed".
  - `crates/zed/Cargo.toml:5` gives the version, 1.25.0.
  - Profiles: `[profile.release]` has `debug = "limited"`, `lto = "thin"` and
    `codegen-units = 1`, with `zed` at 16 (`Cargo.toml:1140-1146`).
  - The Mac's clone: `/Volumes/Offload/Projects/marley_ide`, at `f062e29978`, fetched from the
    box. Its environment is `/Volumes/Offload/Builds/marley-env.sh`: `CARGO_HOME` and
    `RUSTUP_HOME` shared with Rustal on the SSD, and the target at
    `/Volumes/Offload/Builds/target/marley_ide`.
- **Decisions:** D1 to D5 in the spec.

### Design
- **Approach.** In `script/install-marley`:
  - Parse the new options with the others: `--app-dir DIR` and `--remote-server-from HOST`.
  - Right after parsing, `if [[ $(uname -s) == Darwin ]]; then install_mac; exit; fi`, before
    `realpath -m`.
  - `install_mac`:
    1. Refuse `--regress`, since the scenarios need sway.
    2. `commit=$(git rev-parse HEAD)`. With `--remote-server-from`, fetch the server first
       (`ssh HOST 'cat "${XDG_DATA_HOME:-$HOME/.local/share}/marley/remote_servers/dev/linux-x86_64/<commit>.gz"'`
       into a temporary file beside the target, `gzip -t`, rename into place, keep the newest
       three by `ls -t`). When the host has none, stop with the command to run there.
    3. Stop here under `--remote-server-only`.
    4. Wait for cargo to be idle. Read the memory with `sysctl -n hw.memsize`. Under 32 GB, export
       `CARGO_PROFILE_RELEASE_LTO=false` and `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16` unless set,
       and pass `-j` a third of the gigabytes unless `CARGO_BUILD_JOBS` is set, saying so. Then
       `nice -n 10 cargo build --release -p zed --bin marley`.
    5. Assemble `<app dir>/.Marley.app.new.$$`:
       - `Contents/MacOS/marley`;
       - `Contents/Resources/Marley.icns` from `app-icon-dev@2x.png` through an `.iconset`
         (`sips -z` at 16 to 1024, then `iconutil -c icns`);
       - `Contents/Info.plist` with CFBundleIdentifier, CFBundleName and CFBundleDisplayName,
         CFBundleExecutable, CFBundlePackageType APPL, CFBundleShortVersionString (the crate
         version), CFBundleVersion (the commit), CFBundleIconFile, LSMinimumSystemVersion
         10.15.7, LSApplicationCategoryType developer tools, NSHighResolutionCapable, and
         CFBundleURLTypes with `marley`. Zed's two plist fragments are appended with "Zed" read as
         "Marley", and the result is checked with `plutil -lint`.
    6. `codesign --force --deep --sign -` on the new bundle.
    7. Move the old `Marley.app` aside, rename the new one in, remove the old.
    8. Write the launcher to `<prefix>/bin/marley` by rename. It sends stderr to
       `~/Library/Logs/Marley/stderr.log` when stderr is not a terminal, rolling at 10 MiB with
       `stat -f %z`.
    9. Report the bundle, the commit, a running Marley (`pgrep -x marley`), and whether the
       prefix's `bin` is on the PATH.
  - The Linux path is untouched. The header comment gains a paragraph for macOS.
- **File manifest:** `script/install-marley` (Marley-owned). Nothing else in code. The docs come
  at Complete.

### Visual check plan
| REQ | What runs | What shows it |
|---|---|---|
| 007, setup | Box: `script/install-marley --remote-server-only` at the tree under test | its line naming `linux-x86_64` at the commit |
| 006 | Mac: `script/install-marley --remote-server-only --remote-server-from dev` against a commit the box never staged | the error and its exit before any build. The Mac clone checks out a scratch branch for this one step |
| 003, 005, 001 | Mac: `just install --remote-server-from dev` (the env file sourced), logged | the log's sizing line (005); `ls` and `gzip -t` of the server (003); `PlistBuddy -c Print` and `codesign -dv` (001) |
| 002 | Mac: `caffeinate -u`, `open ~/Applications/Marley.app` | `787-mac-app.png` from `screencapture` run by a process with the grant (iTerm through AppleScript); System Events' process name and window titles |
| 004 | Mac: `~/Applications/Marley.app/Contents/MacOS/marley ssh://dev/srv/stacks/marley_ide` started, or Open Remote Folder | box: `ls ~/.zed_server/` and `pgrep -af zed-remote-server-dev`; `787-mac-remote.png` |

What no scenario reaches: none of this runs under `script/e2e.sh`, which needs Hyprland or sway.
The Mac's screen is reachable only through a process with the Screen Recording grant. If none
has it, System Events and the box's record stand in, and the notes say so.

### Risks
- **The first release build on 16 GB.** It is long, and memory is read during it. D3 sizes it,
  and it runs detached under `nohup` so a dropped link does not kill cargo.
- **An install that builds on the Mac changes what Chad sees.** The app lands in
  `~/Applications` and opens on the Mac's screen during the check. Chad is told before the first
  build.
- **`marley ssh://…` on the command line.** On macOS a second launch while one runs exits
  "already running" (Zed's TCP check). The remote open uses the first launch, or the UI.
- **The display may be asleep or locked.** If so, `screencapture` returns black or the lock
  screen, and the shot is recorded as unavailable rather than faked.

## Phase 2 — Code
- **What was built**, all in `script/install-marley`:
  - The header gains a macOS paragraph.
  - Two options, `--app-dir` and `--remote-server-from`. Linux refuses both with "is for macOS",
    and its path is otherwise unchanged.
  - `refuse_unquotable`, the old prefix check as a function, used for the prefix and the app
    folder on macOS.
  - `fetch_remote_server`: `ssh -o BatchMode=yes -- HOST sh -s -- <commit>` reads a script from a
    quoted heredoc, so the host expands its own data folder and the commit goes as `$1`. Then
    `gzip -t`, a rename into place, and the newest three kept through `stat -f '%m %N'`.
  - `install_mac`, as designed: sizing, a niced build, the bundle (icon from `sips` and
    `iconutil`, Info.plist from a heredoc plus Zed's two fragments with "Zed" read as "Marley",
    `plutil -lint`, `codesign --sign -`), the swap by rename, `lsregister -f`, the launcher, and
    the report. `cleanup_mac` on EXIT removes the staged bundle, the old one and the icon folder.
  - The Darwin dispatch makes a relative prefix or app folder absolute before `install_mac`
    moves to the checkout, since macOS has no `realpath -m`.
- **Deviations:**
  - The bundle's `CFBundleVersion` is the crate version, not the commit. LaunchServices compares
    it as a version, so the commit goes in a key of Marley's own, `MarleyCommit`.
  - The LTO override is `off`, not `false`. `false` still runs thin-local LTO; `off` runs none.
- **Review:**
  - The script keeps to bash 3.2: no `mapfile`, no `${x,,}`, no `local -n`, no array expanded
    under `set -u`. `/bin/bash -n` on the Mac (3.2.57) passes.
  - The tools are BSD's: `stat -f`, `paste -sd ' ' -`, `date +%Y-%m-%dT%H:%M:%S%z`.
  - A failed `lsregister` was fatal after the app was already in place. It is a note now.
  - Linux runs of `--app-dir` and `--remote-server-from` exit 2 with the message.
- **Gate:** a script-only change, `just gate-fast`: **GATE GREEN [fast]**, 16 of 16, gate:11
  shellcheck `-S info` included.

## Phase 3 — Test
- **REQ-006**, on the Mac under `/bin/bash` 3.2.57:
  `script/install-marley --remote-server-only --remote-server-from dev` at `f062e29978`, before the
  box had staged it. The host's `cat` said no such file; the script printed "dev has no remote
  server staged for f062e2997816…" and "Run this there, in its checkout at that commit:
  script/install-marley --remote-server-only", and exited 1 within seconds. The cache folder was
  left empty and no cargo started.
- **REQ-007**, on the box: `script/install-marley --remote-server-only` built and staged
  `linux-x86_64/f062e2997816….gz` in 2 m 30 s, "with changes not committed". The Linux path is
  unchanged.
- **REQ-003, REQ-005**, on the Mac: `just install --remote-server-from dev`, detached, logged to
  `install-0.log`.
  - It fetched the server into `~/Library/Application Support/Marley/remote_servers/dev/
    linux-x86_64/f062e2997816….gz` first.
  - It printed "16 GB here: LTO off, 16 codegen units, 5 jobs".
  - The first release build took **11 m 15 s**. Memory stayed at 62% free or more and swap never
    grew past the 1.6 GB it started at.
  - The binary is 603 MB. The linker's note on `__eh_frame` over 16 MB is informational.
- **REQ-001:** PlistBuddy on the installed bundle reads:
  - `com.ignibyte.Marley`, Marley as both names, executable `marley`, version 1.25.0,
    `MarleyCommit` the full commit;
  - URL types holding only `marley`, and no `CFBundleDocumentTypes`;
  - the microphone string "An application in Marley wants to use your microphone".

  `codesign -dv`: `Identifier=com.ignibyte.Marley`, `Signature=adhoc`, arm64. `codesign --verify
  --deep` passes. `Resources/Marley.icns` is there.
- **REQ-002:**
  - `open ~/Applications/Marley.app`: `pgrep` shows `marley`. System Events reports the process
    `marley`, `com.ignibyte.Marley`, frontmost. `windows.js` finds one window, 1536×1050, owned
    by "Marley".
  - `787-mac-app.png`, Marley's window shot by window id through iTerm, which holds Screen
    Recording: Marley on macOS in the Marley layout, with traffic lights, the rail (Projects,
    Threads, Filter ⌘F), Default with Home and Onboarding, and Zed's first-run Onboarding page.
    Claude Agent is found (✓). On Linux too, the onboarding title reads "Welcome to Zed":
    pre-existing, not in scope.
  - `787-menubar.png`, the menu bar's left 700 px shot with Marley in front: the app menu reads
    **Marley**, then File, Edit, Selection, View, Go, Run, Window, Help.
  - iTerm was not running before the shots. AppleScript's `quit` was refused with -128
    (iTerm's confirm), so `pkill -x iTerm2` closed it. The two windows it held were the shots'
    own.
- **REQ-004, first run: red.** `open "marley://ssh/dev/srv/stacks/marley_ide"` reached Marley,
  which handled the scheme and tried the remote. The log said: "Failed to open project: failed
  to connect: ssh: connect to host … port 22: No route to host". `ssh dev` from a shell on the
  Mac works. At that second the unified log shows `UserEventAgent … LocalNetwork: found bundle id
  com.ignibyte.Marley by PID`: macOS's Local Network privacy. A new app, and the ssh it starts,
  may not reach a LAN address until the user allows it once. Zed's `Permissions.plist` has no
  `NSLocalNetworkUsageDescription`, which is the reason the prompt shows.
  - **Fix:** the Info.plist gains `NSLocalNetworkUsageDescription` ("Marley connects to machines
    on your local network over SSH, for remote projects and terminals."), and the header says
    why. shellcheck is clean. The reinstall took seconds, since cargo had nothing to build;
    PlistBuddy reads the key; the running test Marley was named in the report ("quit it and open
    Marley again", D5), quit, and reopened. A retry still fails until Chad allows Marley, at the
    Mac's prompt or under System Settings, Privacy & Security, Local Network. Asked.
- **Seen, not in scope:** `marley_workbench::ports` logs a WARN every 3 s on macOS ("reading
  /proc/net/tcp6: File not found"). It is the Ports list that #786 named as Linux-only; it fills
  the log. This is the next ticket.

- **The import, Rusty through the tunnel.** With the move's default (Rusty's store stays on the
  box), the Mac's `settings.json` is Chad's with `marley.rusty.connection: "service"` and
  `agent_folder: "~/"`, and a launchd agent keeps an `ssh -L` from the Mac's `127.0.0.1:4174` to
  the box's. The reopened Mac Marley logged "Using HTTP transport for http://127.0.0.1:4174/mcp".
  - `787-mac-rusty.png`: signed in (Chad's avatar, from the copied `development_credentials`),
    the larger UI font from his settings, the Default group, and the Onboarding tab.
  - The relaunch restored only the Onboarding tab, and Home was not on the rail. The log had
    `home_tab.rs:193 entity released`, an async load that finished after its page was dropped.
    `home_page::ensure` puts Home first beside a restored tab, and something removed it again.
    Seen with a restored first-run Onboarding tab. Pre-existing; to be checked on Linux before
    a ticket.
- **REQ-004: not seen yet, waiting on Chad.** The Local Network rule macOS keeps for
  `com.ignibyte.marley` (`/Library/Preferences/com.apple.networkextension.plist`, read with
  `sudo plutil -p`) has `DenyMulticast => true` and `MulticastPreferenceSet => false`: no decision
  yet. Only the user can grant it. The Mac is not on the tailnet, so no non-LAN route to the box
  exists. Routing Marley around the check, through a loopback forward, would sidestep the user's
  consent, so it was not done.
  - Everything up to the connect is seen: the staged server in place, `marley://ssh/...` taken by
    the running app, and the connect attempted.
  - A watch on that rule runs. When it turns, the remote open is run and its result appended here
    and to the ticket.
- **Visual check verdict:** REQ-001, 002, 003, 005, 006 and 007 seen. REQ-004 pending the user's
  permission, recorded with its reason (§7: never skipped silently). The gate, run again after the
  plist fix: **GATE GREEN [fast]**, 16 of 16.

## Phase 4 — Complete
- **Docs:**
  - `CHANGELOG.md`, Added: "Marley installs on a Mac".
  - `docs/marley/guide.md`: "On a Mac (#786, #787)" under Install and run, and a line in the
    opening paragraph.
  - `docs/marley/three-prong-plan.md`: the macOS bullet names #787 shipped and #788 to #791.
  - No crate touched, so no per-crate note or touchpoint row.
- **Ledger:**
  - F-claude-787-a-new-mac-app-could-not-reach-the-local-network-001
  - PR-claude-787-a-mac-bundle-that-reaches-lan-hosts-names-its-local-network-reason-001
  - L-claude-787-seeing-a-mac-apps-window-from-another-machine-001
  - L-claude-787-reading-a-mac-apps-local-network-decision-001
  - AD-claude-787-marleys-mac-app-is-written-by-its-install-script-001
- **Brain:** consultation `2fd9cc4e875f471eb734f3201df0ae8c` closed by
  `decisions/marleys-mac-app-is-written-by-its-install-script`, follow-up by 2026-10-30.
- **Open after close:** REQ-004's remote open, once Chad allows Local Network. Its result goes to
  this entry and to the ticket.

## After close — REQ-004 seen (2026-10-16, 21:36)
- Chad allowed Marley under Local Network. The allow first went to `com.ignibyte.marley`, an
  earlier signing; the app signs as `com.ignibyte.Marley` (`codesign -dv`), and that rule turned
  to `DenyMulticast false`, `MulticastPreferenceSet true` once he switched the second entry on
  (read as XML, by `SigningIdentifier`; see the corrected L-claude-787 lesson).
- The Mac's `Marley.app` at `5fd021ee44`, its cache holding the server for that commit
  (`--remote-server-from dev`); the box had no `~/.zed_server`.
- `open "marley://ssh/dev/srv/stacks/marley_ide"` on the Mac: within a minute the box had
  `~/.zed_server/zed-remote-server-dev-5fd021ee447fa7910cd7e60e97eed162086f24ee` (114 MB, uploaded
  from the Mac's cache), running as `proxy --identifier dev-workspace-3` and `run --log-file …`.
  The Mac's log: the remote server set its environment from `/usr/bin/bash`, loaded the HTML
  language server, and opened the git repository at `/srv/stacks/marley_ide/.git`.
- `787-mac-remote.png` (scratchpad, not kept): a new Marley window, the title bar's remote icon,
  `dev`, `marley_ide`, `main / marley/workbench-shell`, and Zed's Restricted Mode for a folder
  not yet trusted; the rail with Default's Home and the `marley_ide` group.
- **REQ-004 seen.** Every criterion of #787 is now met.

