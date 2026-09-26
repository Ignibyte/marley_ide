# Orca survey 07: how Orca is engineered, and the real feature list

Surveyed 2026-09-25 against `stablyai/orca` at `1c2cf120e3` (checkout `/srv/stacks/orca-refs/orca`)
and the GitHub releases up to v1.4.212. Paths are relative to the Orca repo root unless they
start with `crates/`, `script/`, `docs/planning/` or `CONSTITUTION.md`, which are Marley's.

## 1. Summary

Orca (first commit 2026-03-16; 11,752 commits, 2,533 in the last month from five main accounts;
1.77M lines of TypeScript plus 2.16M of tests) is written at a rate only coding agents reach, and
its engineering is the scaffolding that keeps that rate from breaking users: a detached terminal
daemon that keeps PTYs alive through quits, crashes and updates; fsync-before-rename writes with a
backup ring; architecture rules as tests with shrink-only allowlists; executable before/after
audits; golden e2e suites gating every release; about one stable release a day. Most worth taking:
(1) a golden set and path-to-scenario routing, so Marley's scenarios become a regression suite;
(2) for #502, an instance guard, an owned durable endpoint file and local crash dumps; (3) ratchet
checks with self-tests in `script/gates.sh`; (4) the daemon's lessons for harness-backed terminals;
(5) captured agent-screen transcripts and one status store for the approvals inbox (#508).

## 2. Features

Part A covers how Orca is engineered (Half 1). Part B is the feature list from the release notes
(Half 2). Each Part A section ends with **Marley today**.

### Part A: how Orca is engineered

#### A1. Process architecture

**What it is.** Three processes on the desktop (main, the renderer with its preload script, the
terminal daemon), plus a headless runtime, a relay on remote hosts, and clients:

| Process or layer | Owns | Key source |
|---|---|---|
| Electron main | the runtime (`OrcaRuntimeService`), git, worktrees, persistence, the agent hook server, browser guests, the updater | `src/main/index.ts` → `src/main/startup/*`; `src/main/runtime/` (1,016 entries, 155k lines of non-test source) |
| preload (runs in the renderer) | one `window.api` object of about 90 domain bridges, each owning its IPC channel names | `src/preload/index.ts`, `src/preload/api/*-bridge.ts` |
| renderer | React + Zustand UI, sandboxed with `contextIsolation` | `src/renderer/src/` |
| terminal daemon | every local PTY, detached from main | `src/main/daemon/` (432 entries, 30k lines + 53k test lines) |
| `orcad` / `orca serve` | the same runtime on plain Node, headless | `src/main/orcad/`, `docs/reference/orcad-operations.md` |
| SSH relay | git, files, PTYs and hooks on a remote host, uploaded by Orca | `src/relay/` (29k lines) |
| clients | the `orca` CLI over a Unix socket; mobile and web over WebSocket | `src/cli/`, `src/main/runtime/runtime-rpc/` |

**How it works.**

- **The daemon.** It speaks NDJSON over `<data-root>/daemon/daemon-v<N>.sock` (a named pipe on
  Windows). Each session keeps an `@xterm/headless` emulator with the serialize addon
  (`src/main/daemon/headless-emulator.ts`). A client that reattaches gets a snapshot plus the
  sequences that re-arm the terminal modes. Dirty sessions are checkpointed to per-session history
  directories about every 5 s, and only while something changed
  (`daemon-pty-checkpoint-scheduler.ts`). When the daemon itself died, a cold restore replays those
  checkpoints. The protocol is at version 36. Versions 1–35 are still attachable through legacy
  adapters, because a daemon outlives the app that started it (`daemon-protocol-version.ts`).
- **Owning the socket path.** `src/main/daemon/AGENTS.md` records two invariants. Only a daemon
  publishing itself may replace the entry, and only one it has just proven dead. No actor removes
  a name it did not create. The protocol (`daemon-endpoint-ownership.ts`):
  1. bind a private `.p<hex>` name;
  2. try an exclusive `link`;
  3. on `EEXIST`, prove the incumbent dead by connecting;
  4. re-check the entry and probe once more;
  5. `rename` onto the path.

  Only "refused" or "missing" proves death; a timeout proves nothing. The doc says seven review
  rounds of the older "launcher reclaims a dead name" design produced 23 defects, all the same
  race.
- **`orcad` on Linux.** It starts the daemon with `systemd-run --user --scope`, so the daemon and
  its PTYs sit in their own `orca-daemon-<nonce>.scope` and a service restart leaves them running.
  The doc spells out why `KillMode=mixed`, `nohup`, `setsid` and `tmux` do not escape a cgroup. It
  also takes `<data-root>/orcad.lock` and refuses to start when another live `orcad` owns the data
  root.
- **The runtime stays off Electron.** A ratchet keeps any `electron` import out of the runtime's
  import graph (`config/runtime-electron-baseline.txt`, empty and required to stay so). Host
  services go through ports in `src/main/host/`.
- **Runtime RPC** (`runtime-rpc-lifecycle.ts`). A Unix socket or named pipe serves the CLI. A
  WebSocket serves mobile and web, with per-device tokens and tweetnacl end-to-end encryption
  instead of TLS, because React Native cannot pin a self-signed certificate. A metadata file makes
  the runtime discoverable; a runtime that cannot write it closes its transports rather than run
  undiscoverable.
- **The renderer and its browser tabs.** The renderer cannot import Node builtins; a test walks
  the import graph from every renderer entry and fails on any `node:` specifier
  (`src/renderer/src/renderer-node-builtin-boundary.test.ts`). Browser tabs are `<webview>` guests
  kept in a fixed-position root outside React's tree
  (`src/renderer/src/components/browser-pane/browser-client-page-retained-elements.ts`), so pane
  remounts do not reload pages.

**Well and badly.** Every long-lived thing has one owner that is not the UI, and the daemon
survives the app. The cost is extreme complexity:

- 36 wire versions;
- 844 files (tests included) in `src/main/ipc/`;
- a runtime split into about 90 mixin files to satisfy a 300-line `max-lines` rule, 171 of which
  carry `@ts-nocheck` because the mixin chain defeats the type checker (`config/ts-nocheck-baseline.txt`).

**Size.** The daemon and orcad are a subsystem of about 34k lines. The whole process model is
most of the codebase.

**Marley today: has part of it.**

- Marley is one Zed process. Terminals are in-process (Zed's `terminal` crate on alacritty) and
  die with Marley.
- `marley_mcp` runs in-process and writes `mcp-endpoint.json` into the data directory
  (`crates/marley_workbench/src/mcp.rs`).
- Chromium already runs as a transient systemd user unit that outlives Marley and ends at logout
  (`crates/marley_browser/src/service.rs`, plan D16).
- The out-of-process owner of terminal sessions is planned as the rustal-harness runtime (prong 2,
  C1/C3 in `docs/marley/three-prong-plan.md`). Orca's daemon is the closest working reference for
  it.

#### A2. What survives a renderer reload, a quit, an update, a reboot

| Event | PTYs and CLI agents in them | Native (structured) chats | Browser tabs | Remote PTYs |
|---|---|---|---|---|
| Renderer crash | Kept. Main auto-reloads the renderer (at most 3 reloads in 60 s: `src/main/crash-reporting/renderer-recovery-circuit-breaker.ts`; a reload that never lands is caught by `src/main/window/renderer-recovery-reload-watchdog.ts`), then panes reattach through `pty:replay` | Main keeps them | Reloaded | Kept |
| Quit, app crash, auto-update | Kept by the daemon. On Windows the daemon runs from a copy of the runtime under `%LOCALAPPDATA%` so an update cannot replace its files (`docs/reference/windows-daemon-host-relocation.md`) | Journaled in SQLite. On relaunch Orca offers to reconnect chats that were working (v1.4.206, #21096) | URL and history restored (`src/shared/workspace-session-browser-schema.ts`); pages reload | Leased on the host by the relay; "Keep terminals alive until reset" is on by default (`docs/site/content/docs/ssh.mdx`) |
| Reboot | Processes gone. Layout and last checkpointed scrollback restored (`docs/site/content/docs/model/session-restore.mdx`) | Journal replayed | URL and history restored | Survive if the remote host did |
| Idle agent | Optional hibernation: a done, untouched agent is stopped after 30 min and relaunched with its resume flags when the worktree is reopened (`docs/site/content/docs/agents/hibernation.mdx`) | n/a | n/a | n/a |

**Well and badly.** The native-chat journal is careful about delivery:

- a database with a newer schema opens read-only, so a downgrade never writes it
  (`src/main/native-chat/agent-session-journal/journal-database.ts`);
- after a crash every in-flight submission becomes `unknown`;
- restart reconciliation narrows that to `accepted` or `rejected` only from the provider's own
  history, and never re-sends on the user's behalf (`journal-restart-reconciliation.ts`).

The quit path is bounded too:

- teardown races a deadline (`src/main/quit-teardown-deadline.ts`, after a wedged socket forced
  users to Force Quit in #9447);
- an armed update install force-exits after 20 s (`src/main/update-install-exit-watchdog.ts`).

**Marley today: has part of it.**

- Zed restores the workspace layout and the terminals' working directories. Processes and
  scrollback are lost on quit.
- Marley restores browser tabs (#494). Chromium outlives Marley.
- The e2e runner can quit and relaunch on one profile (`quit_marley`, `launch_marley`).

#### A3. IPC and wire compatibility

**How it works.**

- **The preload bridge is typed.** The `api` object must `satisfies PreloadApi`. A dropped bridge
  key became a compile error in v1.4.197 (#18025).
- **Contracts are generated and checked.** RPC parameters live in a generated catalog that CI
  regenerates and compares (`verify:rpc-params-catalog`).
- **`docs/reference/remote-wire-compatibility.md`** holds the rules, because a client and a remote
  host update independently:
  1. a new optional JSON field is safe, for only as long as every reader treats it as optional;
  2. a new stream opcode must be capability-negotiated, because old decoders drop unknown opcodes
     silently and the feature looks hung;
  3. changing what the host *publishes* is a wire change even when no codec moves;
  4. opcode numbers are never reused.
- **Skew is tested in CI.** The `cross-version-wire` job in `.github/workflows/pr.yml`
  materializes the newest release tag and runs old/new client and server journeys against the
  current code (`tests/e2e/cross-version-wire/*.unit.test.ts`).

**Size.** The rules document is 345 lines. The harness is about 20 files.

**Marley today: lacks it.** Marley's MCP tools are called by agents, and the Claude Code plugin is
installed separately from Marley. Nothing records which tool names or fields are permanent, and
the bridge carries no version check.

#### A4. Durable state

**How it works.**

- **`src/main/durable-file-write.ts`** (213 lines) holds the write helpers:
  - write the temp file;
  - fsync it;
  - consult an optional veto (`writeFileDurableIfCurrent`), so a superseded writer never publishes
    a stale snapshot;
  - rename;
  - fsync the directory where the platform allows it.

  It also has a synchronous variant for quit and crash paths, a durable copy, and a sweep for temp
  files orphaned by a death between write and rename. The header names the bug it fixes: rename
  is atomic but not durable, and a power loss can leave an empty file (issue #1158).
- **`src/main/durable-file-write-syscall-proof.test.ts`** mocks `node:fs` and records the order of
  fsync and rename calls, so the order is proven rather than read off the code.
- **`orca-data.json`** (`src/main/persistence/loading-store/`, 15.5k lines):
  - saves are debounced 1 s with a 5 s maximum wait;
  - a write-generation counter stops an in-flight async write from overwriting a newer sync
    write;
  - once the quit flush starts, no new save is scheduled, so the quit flush is the last write by
    construction (`write-scheduling.ts`);
  - a five-slot `.bak.N` ring is rotated at most hourly, and restore tries each slot and validates
    it as JSON before using it (`backup-recovery-rotation.ts`);
  - large, frequently rewritten data lives in sidecar files with a per-file queue, so the main file
    is not rewritten per scan (`src/main/sidecar-snapshot-file.ts`).
- **`src/main/file-transaction-lock.ts`** (31 lines, `proper-lockfile`) serializes whole-file
  transactions across Orca processes on one host, in a 0700 directory.
- **`src/main/rolling-file-backup.ts`** refuses a symlinked backup path and always writes a fresh
  inode, so a hard link cannot redirect the write.

**Well and badly.** This is small, correct and reusable. The trade-off is sound: the backup ring
costs at most an hour of state, and fsync stops the empty-file case from happening at all.

**Size.** About 500 lines for the helpers. The store around them is a subsystem.

**Marley today: lacks the durable half.**

- Zed's `fs.atomic_write` writes a temp file in the same directory and renames it, with no fsync
  of the file or the directory (`crates/fs/src/fs.rs:972`).
- Marley's `mcp-endpoint.json` is opened with truncate and written in place
  (`crates/marley_mcp/src/discovery.rs`), so a reader can see a half-written file. Marley deletes
  it on quit whether or not it is still Marley's own (`crates/marley_workbench/src/mcp.rs:102`).
- Flight recordings are written with `fs::write` (`crates/marley_browser/src/recorder.rs:373`).

#### A5. Hang watchdog, crash capture, diagnostics, resource manager

**The hang watchdog** (`src/main/hang-watchdog/`, 318 lines):

- it runs on packaged macOS builds only, in a worker thread, because the worker survives an AppKit
  deadlock without another process;
- main sends a heartbeat every 2 s, the worker checks every 5 s, and 45 s of silence is a hang;
- a tick gap over three check intervals means the machine slept, and the wait restarts;
- it only observes and never kills, because "a false positive must never kill a live main thread
  mid-write";
- it writes `main-thread-hang.json` and rewrites it with `selfRecovered: true` if heartbeats come
  back, which separates a deadlock from a long stall;
- the next launch consumes the marker and reports it.

**Crash capture** (`src/main/crash-reporting/`, 4.5k lines):

- Crashpad runs with `uploadToServer: false`;
- each minidump is paired with the `render-process-gone` / `child-process-gone` event that
  reported it, polling up to 8 s because the two race;
- dumps stay on disk under a budget of 128 MiB and 64 files;
- a text crash signature is lifted out of the dump (`minidump-crash-signature.ts`), so a CHECK
  failure is nameable without shipping memory anywhere;
- the report store keeps 5 reports, and handling one also settles the related ones (same reason,
  exit code, version and platform within 5 s);
- durable breadcrumbs flush immediately;
- a GPU-crash fallback disables acceleration behind a restart prompt;
- process deaths are classified, and Orca's own tree kills are recorded so they do not read as
  crashes.

**Diagnostics:**

- a local NDJSON trace sink (`src/main/observability/local-file-sink.ts`) and a redactor;
- a *user-initiated* bundle upload: a short-lived token, a 4 MiB cap, and an endpoint that official
  builds pin so an environment variable cannot redirect it (`diagnostic-upload-endpoint.ts`);
- `ORCA_STARTUP_DIAGNOSTICS=1` for startup milestones;
- `ORCA_MAIN_THREAD_DIAGNOSTICS=1`: a 25 ms timer probe that buckets blocking spawns as
  `git status`, `gh api` and so on (`src/main/diagnostics/main-thread-churn-probe.ts`).

**The resource manager.** A status-bar popover lists every daemon session with its CPU and memory
(PTY process subtrees plus Orca's own processes, per worktree, with a sparkline). It can kill one
session, kill all, or restart the daemon (`src/main/memory/collector.ts`,
`src/renderer/src/components/status-bar/ResourceUsageStatusSegment.tsx`; the preload channels
`pty:management:listSessions|killOne|killAll|restart`). The docs site mentions it only as a status
bar toggle in `settings.mdx`.

**Well and badly.** The discipline is good: evidence stays local, uploads are opt-in, and the
watchdog cannot hurt the user. The downside is that each path is Electron-specific.

**Size.** A subsystem, about 8.5k lines with observability.

**Marley today: has part of it, switched off.**

- Zed's gpui hang detector and frame-budget reports run, writing `hang_traces/` and
  `telemetry.log` under `~/.local/share/marley/` (`crates/zed/src/reliability/hang_detection.rs`).
- Zed's crash handler (a minidumper server process writing into the logs directory) is installed
  only when `ZED_GENERATE_MINIDUMPS=1`, or on a non-`dev` channel with `ZED_MINIDUMP_ENDPOINT` set
  (`crates/client/src/telemetry.rs:92-103`). Marley stays on `dev`, so today it captures no
  crashes.
- There is no per-project resource view.

#### A6. Test strategy

**How it works.**

- **Unit tests.** Vitest (`config/vitest.config.ts`), 10,699 test files across the repo. They
  include node-pty "live-shell" tests against real zsh and fish.
- **E2E.** Playwright (`@stablyai/playwright-test`, Stably's own wrapper) launches the built
  Electron app with `_electron.launch()`. There are 410 specs.
  - Runs are headless by default: the window never reaches the screen and focus is never taken
    (`src/main/window/foreground-activation-policy.ts`, `tests/AGENTS.md`).
  - Each test gets its own userData directory and HOME (`tests/e2e/helpers/electron-home-isolation.ts`).
  - Retries are 0 and traces are kept on failure (`tests/playwright.config.ts`).
  - The store is exposed as `window.__store` only in `--mode e2e` builds, and the rule is
    "use the store to reach a state; use the DOM to prove it" (`tests/e2e/AGENTS.md`, after a
    store-only test let a crashing composer ship in #1186).
- **Golden suites.** 15 `tests/e2e/golden-*.spec.ts`. Among the flows they cover:
  - fresh startup;
  - quit and relaunch;
  - worktree create and switch;
  - file open, edit and save;
  - source-control commit and diff;
  - agent TUI launch;
  - shell after agent exit;
  - terminal file link.

  `release-cut.yml` runs most of them as *blocking release gates*: terminal rendering, workspace
  session, source control and agent TUI on Linux and macOS, and fresh startup on Windows. New
  golden suites are trialled in `golden-e2e-experiment.yml` first.
  `golden-quit-relaunch-session.spec.ts` echoes a marker in an extra terminal, opens a file, quits
  and relaunches. It then asserts the same worktree, the same tab count, the same file, the *same
  PTY id*, the marker still in scrollback, no Reconnect button, and a new command that runs.
- **Routing.** PR CI runs only the e2e specs routed from the changed paths. The routes are an
  executable table of path regexes mapped to specs (`config/scripts/pr-e2e-source-routing.mjs`,
  326 lines). The full suite runs twice a day (`e2e.yml` cron `0 17,22 * * *`) and on each release
  tag after publishing.
- **Special lanes.**
  - Transport and platform: Docker SSH with fault injection, real WSL, a real input method (ibus
    Hangul).
  - Compatibility: a Git matrix built against a baseline Git 2.25 binary, real zsh and fish shell
    contracts, Node 18 managed hooks.
  - Performance: daily performance contracts, and a nightly terminal-performance run judged
    against budgets set from recorded runs. Typing median is 25 ms, the worst key 300 ms, restore
    1 s. The rule: "do not set limits just above a new failure or mirror relaxed test timeouts"
    (`docs/reference/terminal-perf-report-budgets.md`).
  - Packaged builds: packaged CLI, packaged hang-watchdog worker under xvfb, and headless
    `serve` signal shutdown.
- **Benchmarks.** About 25 `bench:*` scripts measure startup, idle CPU, main-thread jank,
  multi-workspace typing and more.

**Well and badly.** The breadth is real, and the budgets are honest. The cost is 2.16M lines of
tests leaning on `vi.mock`, which an anti-slop lint rule now bans outside tests. The E2E suite
needed per-worker isolation and still flakes: the gate registry (A8) lists 77 of 126 gates with an
unknown flake history.

**Size.** The largest part of the repo.

**Marley today: has a different model.**

- Proof is an e2e visualization scenario per ticket (`script/e2e/<ticket>-<slug>.sh`, 25 so far,
  CONSTITUTION §7), run once at that ticket's Test phase, with the shots read by the agent.
- No scenario is re-run later. The scenarios carry no machine-checked assertion. There is no
  golden set, no routing and no timing budget.

#### A7. Architecture rules as tests: boundary, ratchet, proof and audit tests

There are 58 `*-boundary*.test.ts`, 6 `*ratchet*.test.ts`, 27 `*proof*.test.ts` and 9
`*audit*.test.ts` files. They come in six shapes.

1. **Chokepoint scans with a shrink-only allowlist.**
   `src/shared/child-process/child-process-import-boundary.test.ts` requires every child process
   to go through `runProcess`/`spawnProcess`. It uses five devices:
   - the allowlist is a data file (`__fixtures__/child-process-import-allowlist.txt`);
   - a listed file that no longer offends fails the test ("delete the line"), because a stale entry
     hides the next regression in the same path;
   - a *literal* pin (152) must equal the real count in both directions, because bounding by the
     list's own length lets a swap through;
   - an anti-vacuity check requires the scan to see more than 500 files, so a broken root cannot
     pass silently;
   - comment lines are ignored, so prose about the old idiom is not an offender.

   `src/main/ripgrep/bare-ripgrep-spawn-boundary.test.ts` has an empty allowlist and says in a
   comment what it cannot see. `src/main/global-fetch-call-site-audit.test.ts` pins the expected
   number of `fetch(` lines per audited file, so any change forces a re-audit (undici can crash the
   process on an unread body).
2. **Import-graph walks.** `renderer-node-builtin-boundary.test.ts` (A1).
   `src/main/claude/claude-agent-sdk-import-boundary.test.ts` keeps the Claude Agent SDK behind a
   deferred import, because loading it rewrites a Windows environment variable for every later
   subprocess. The runtime-electron ratchet (A1) is the third.
3. **Reaper ratchets.** `src/main/runtime/pty-exit-per-pty-map-reaper-ratchet.test.ts` builds a
   real `OrcaRuntimeService`, lists every field named `*ByPtyId` that holds a Map or Set, and
   requires each to be deleted in the PTY-exit reaper or placed in a verified bucket: cleared by a
   named helper, self-clearing in flight, or intentionally retained with a reason. It exists
   because a map added next to 25 siblings leaked one entry per PTY for the life of the process.
4. **Planted-violation self-tests.** `src/shared/agent-status-legacy-ingress-ratchet.test.ts`
   plants each forbidden mutation form in source text and asserts that the detector catches every
   one, so the guard is itself tested.
5. **File-level ratchets in lint.**
   - `max-lines` allows 300 lines per `.ts`, 400 per `.tsx` and 800 per test file, and a baseline
     of 7 grandfathered files may only shrink (`config/max-lines-baseline.txt`,
     `config/scripts/check-max-lines-ratchet.mjs`).
   - `ts-nocheck` is ratcheted at 171 files.
   - `AGENTS.md` forbids adding a `max-lines` disable.
6. **Proof tests.** Tests that demonstrate a mechanism at a module boundary rather than assert a
   result. `src/main/durable-file-write-syscall-proof.test.ts` is one; the Claude exit and
   transcript-rewind proofs under `src/main/claude/` are others.

**Well and badly.**

- Shapes 1, 3 and 4 are the best ideas in the repo for a codebase written by agents. They turn a
  review comment into a rule that cannot rot quietly.
- Shape 5 did harm: the 300-line limit produced the runtime's mixin sprawl and its `@ts-nocheck`
  files.

**Size.** Each test is 100–200 lines.

**Marley today: has the shape, not the devices.**

- gate:12 (suppressions) and gate:13 (transmute, `unsafe` without `// SAFETY:`) are grep
  meta-gates. gate:20 runs semgrep and gate:21 runs Zed's dylint lints. None has a shrink-only
  allowlist, a stale-entry check, a literal pin, an anti-vacuity check or a planted self-test.
- §7's "negative smokes" are the manual version of the self-test, run once when a gate changes.
- §14's "keep spawns in adapter modules" is not checked by anything.

#### A8. Evidence documents: audits, review evidence, bug reproductions, the gate registry

- **`docs/audits/<name>/`** (46 folders). Each has:
  - a README stating the claim, how to run the proof, and a *Scope and limits* section on what it
    does not establish;
  - a `fix.patch`;
  - a `reproduce.mjs` that reverse-applies the patch *in memory* through a Vite transform plugin,
    so the checkout never changes, and runs the same tests against before and after;
  - a `results.json` with the sha256 of every source involved and the pass/fail counts.

  Examples: `acknowledged-tab-retirement` (28 failing assertions before, 31 passing after) and
  `pty-detector-retention` (retained heap per owner from 2.1 MB to 41 KB). All 41 commits that
  touch `docs/audits/` landed on 2026-09-17..19 from one account, `OrcaWin` (created June 2026, no
  profile), and nearly every audit is a memory-retention fix.
- **`docs/review-evidence/pr-19217/README.md`** is the only one of its kind. It records:
  - what was observed live in a hidden background dev instance, with CDP screenshots reviewed and
    not retained;
  - test counts;
  - an *ablation*: removing the fixed call made the call-site test fail with two rows where one was
    expected;
  - a "remain unverified" list naming the platforms and scenarios nobody ran.
- **`docs/bug-reproductions/`** (2) holds live captures against shipped third-party binaries: the
  OpenCode 2 SSE event stream, and hook POST logs before and after the fix. Each has its repro
  commands and the traps met on the way.
- **`config/reliability-gates.jsonc`** is 20,280 lines and holds 126 gates, validated in PR CI by
  `config/scripts/check-reliability-gates.mjs`. A gate declares:
  - an invariant and an oracle;
  - commands (title selectors such as `-t` or `--grep` are refused);
  - test files, which must exist and be referenced by a command;
  - assertion references;
  - evidence runs, whose command must be one of the gate's commands;
  - covered platforms, which must include every platform with a passed run;
  - red/green evidence, flake history, runtime and performance budgets;
  - known gaps and promotion and demotion rules.

  The policy says "blocking" needs 100 soak runs over 14 days with no unexplained flake. In fact
  all 126 gates are still `experimental`, 115 are `partial`, and 77 flake histories are `unknown`.

**Well and badly.** The audits are strong. They are executable before/after proofs, and their
"Scope and limits" paragraphs refuse to overclaim. The registry is bookkeeping without teeth so
far. CI checks that the manifest is well formed, but no gate has reached `blocking`, so no entry
decides whether a merge lands.

**Size.** Each audit is a README plus a reproduce script of 57–150 lines (6.2k lines across all
46). The registry checker is 431 lines.

**Marley today: has part of it.**

- The knowledge ledger (`docs/planning/knowledge/`, 9,177 lines of failures, prevention rules,
  lessons and architecture decisions) is append-only and recalled by grep.
- Pipeline notes record what each shot shows.
- Nothing is executable before/after, and nothing records what a proof does not establish.

#### A9. How Orca steers coding agents

**The root `AGENTS.md`.** `CLAUDE.md` is one line, `@AGENTS.md`. The file covers:

- **The design system.** Follow `docs/STYLEGUIDE.md`, enforced on changed lines.
- **UI validation.**
  - Always launch with `ORCA_BACKGROUND_LAUNCH=1`, never steal focus, and use CDP screenshots of
    hidden renderers.
  - Keep native-focus tests off the user's desktop.
  - Do not use computer-use to check Orca's own UI.
- **Code habits.**
  - Reuse before reimplementing.
  - Write one-line "why" comments.
  - Never disable `max-lines`.
  - No files named `helpers`, `utils` or `common`.
  - Type assertions need a `SAFETY:` comment.
- **PRs.** How to write the PR.
- **Traps, each with a pointer.** Each trap names the reference doc to read *before touching*
  that code:
  - Windows spawns, ripgrep, WSL argv, the Linux glibc floor;
  - the SSH execution boundary ("loss of contact is never evidence of process death", with the
    verdicts `live` / `unverifiable` / `exited`);
  - the single agent-status store;
  - agent terminal screens, written against captured transcripts;
  - remote wire compatibility, Git 2.25 compatibility, and a ban on ref × tree fan-out scans.

**Directory-level files.**

- `src/main/daemon/AGENTS.md` holds the endpoint invariants and "Traps that already cost us".
- `tests/AGENTS.md` holds the foreground policy.
- `tests/e2e/AGENTS.md` holds the `--mode e2e` build trap, "prefer a store-slice unit test when
  the logic is pure", and "assert on the DOM".

**`docs/STYLEGUIDE.md`** (28 KB):

- tokens and primitives;
- "UI copy must not overclaim";
- a screen UX review rubric: top 3 fixes, friction notes, suggested changes, a keyboard and speed
  check, follow-up links;
- feedback matched to duration: 0–100 ms none, then disabled, then a spinner, then stages; the
  visible loading state is deferred 200 ms so SSH users see it and local users do not;
- "Cancel is not destructive".

**Lint aimed at agent habits.** `config/oxlint-anti-slop.json` loads a plugin whose rules ban:

- module mocking outside tests;
- object parameters;
- `Reflect.get` and `Reflect.apply`;
- copying the accumulator inside `reduce`;
- the word "shape" in symbol names;
- `unknown` type aliases;
- widen-then-assert.

Other gates of the same kind:

- changed-lines gates for code quality and React Doctor (`check:code-quality:changed`);
- a root-directory guard that rejects new top-level files;
- a PR comment counting test against non-test lines (`.github/workflows/pr-test-loc.yml`).

**The PR template** (`.github/pull_request_template.md`) asks for:

- an ELI5;
- the before and after as the user experiences it;
- the mechanism;
- why this approach over the alternatives;
- Visual Proof: before/after, video preferred, attached and never committed;
- the platforms actually tested;
- AI disclosure from outside contributors.

**Well and badly.** The "before you touch X, read Y" pointers put knowledge where the edit
happens. The directory `AGENTS.md` files keep traps next to the code they guard. The overall
volume (52 reference docs, a 119 KB checklist in `docs/`) is the price.

**Marley today: has most of it, differently.**

- `CLAUDE.md`, `CONSTITUTION.md` and hooks enforce the phases, the Zed ledger (gate:16), the
  commit receipt and the check that e2e actually ran.
- Knowledge is recalled by grep (§18.3), not pushed at the moment a path is edited.

#### A10. Release engineering

**Channels.** `src/shared/release-channel.ts` defines five:

| Channel | Where it publishes | How it is built |
|---|---|---|
| `stable` | `stablyai/orca` | cut by hand |
| `rc` | `stablyai/orca` | cut by hand |
| `hourly` | `stablyai/orca-hourly` | every hour main moved |
| `daily` | `stablyai/orca-daily` | once a day |
| `adhoc` | `stablyai/orca-adhoc` | on demand, from any branch or tag |

- **Why separate repos.** The main repo's releases feed exposes only its 10 newest entries, so 24
  hourly tags a day would evict every stable entry and break updates.
- **How dev builds are versioned.** The timestamp is in the version (`1.4.213-hourly.202609260126`;
  adhoc is stamped to the second so two people cutting at once cannot collide).
- **Hourly.** Top of every hour, skipped when main has not moved. Signed and notarized, so macOS
  permission grants survive updates. No tests. The last 72 builds are kept, "so a regression can be
  bisected across a weekend".
- **Daily.** 18:15 UTC. It dispatches the full e2e suite after publishing without waiting for it.
- **Adhoc.** Any branch or tag of the repo; PR refs are refused because the build job holds the
  signing secrets. Titles look like `1.4.213 • sqlite-bun-reviewed • Sep 25, 3:15PM • 5be4c22`.
- **Platforms.** Dev channels build for macOS and Windows only; Linux falls back to stable. Windows
  dev builds are unsigned, because the SignPath approval wait does not fit an hourly cadence. The
  way into a dev channel on Windows is therefore one manual install, and the way out works in-app.
  In practice RC tags stopped after v1.4.184-rc.0 (2026-08-16), and hourly and daily builds took
  over that role.

**The updater** (`src/main/updater*.ts`, `src/main/updater/`, about 4.4k lines, electron-updater
loaded lazily by `src/main/electron-updater-loader.ts` so dev and e2e launches do not fail on
version validation):

- **Modifier clicks on Check for Updates.** Shift picks the latest RC, Cmd/Ctrl the latest
  perf-tagged prerelease, and Option a validated local macOS build served by a local feed server.
- **Pinned builds.** A developer can pin to any exact tag on any channel, older ones included
  (`updater/updater-build-selection.ts`).
- **The build list.** It is read from the GitHub API with rate-limit handling.
- **Remote content.** Nudge campaigns come from `onorca.dev/whats-new/nudge.json`, and an in-app
  "what's new" card from `onorca.dev/changelog`.

**Cutting a release** (`.github/workflows/release-cut.yml`, 2,432 lines, manual dispatch only; the
schedule-handling code left in it is dead):

1. **Inputs.** `kind` (rc, patch, minor or major), `ref`, `dry_run`, `version_suffix` (the `.perf`,
   `.ghes` and `.issue7936` tags came from this) and an exact `version`.
2. **Version rules.** A stable no newer than the latest stable is refused.
3. **Draft.** The release is created as a draft.
4. **Blocking gates.** The golden e2e suites run on three OSes, and the skill-sharing suites run on
   three OSes plus a glibc 2.31 container.
5. **Builds.**
   - Linux: AppImage, deb and rpm for x64 and arm64.
   - Windows: NSIS, signed by SignPath after a Slack ping asks a human to approve.
   - macOS: an isolated signed build.
   - Every release also carries a main-process source-map zip.
6. **Asset check.** Every updater manifest and installer must be present before the release leaves
   draft.
7. **Prerelease flag.** It is set from the tag's shape, because electron-builder once flipped an RC
   to "latest".
8. **After publishing.** Post-release e2e on the tag, a docs deploy for stable tags, and Homebrew
   casks (`orca`, `orca@rc`). `release-policy.yml` reverts any release not published by the bot
   with a valid tag.

**Cadence.** 100 stable releases and 208 prereleases since 2026-06-26. The prereleases are RCs,
`.perf` and `.ghes` builds and mobile APKs. Since July 1 there were 89 stable releases on 65 days,
up to four in one day. Recent notes open by saying a landed PR takes 48–72 hours to ship.

**Linux:**

- **AppImage.** It updates itself.
- **deb and rpm.** Orca downloads the package and offers **Copy Install Command** with absolute
  paths (`/usr/bin/sudo /usr/bin/apt install -- '<path>'`). It never escalates privileges itself.
- **Repackaged installs.** AUR (`stably-orca-bin`, `stably-orca-git`, both maintained outside the
  repo) and Nix builds are detected through the `package-type` marker they inherit and left alone
  (`src/main/linux-update-package-type.ts`).
- **The CLI name.** The CLI is `orca-ide`, because GNOME's screen reader owns `/usr/bin/orca`.
- **The glibc floor.** Packaging fails if a bundled native module needs a glibc newer than 2.31.

**Well and badly.** It is well built for thousands of users on three OSes. Most of it exists to
serve other people's machines.

**Size.** About 4.4k lines of updater and 4.8k lines of release workflow YAML.

**Marley today: lacks it.**

- No release build is installed; #502 is queued (`just install` of a release `marley` into
  `~/.local/bin` with a desktop entry, sharing the `dev` profile).
- TICKET-445 is deliberate: on any channel but `dev`, Zed's updater would install stock Zed over
  Marley.

#### A11. Telemetry and privacy

**How it works.**

- **The policy.** `docs/site/content/docs/telemetry.mdx`:
  - PostHog Cloud (US);
  - a random local install id;
  - only fixed enums, versions and that id;
  - opt out in Settings, with `DO_NOT_TRACK=1` or with `ORCA_TELEMETRY_DISABLED=1`;
  - CI is detected and disables it.
- **The schema is the validator.** zod schemas with `.strict()` and `.max()` in
  `src/shared/telemetry-events.ts` fail closed on an unknown event, an extra key or a long string,
  with warnings rate-limited to one per event per minute (`src/main/telemetry/validator.ts`).
- **Burst caps** (`burst-cap.ts`): 30 events a minute per event name (20 for `agent_error`), 1,000
  per session, 5 consent changes per session.
- **Consent** (`consent.ts`) is a resolver that returns a reason, not a boolean.
- **Error tracking is a separate lane.** It never shares code with product telemetry, by an import
  rule (`src/main/observability/index.ts`). `ORCA_DIAGNOSTICS_DISABLED=1` also stops the local
  logs.

**Size.** About 1k lines in main plus the shared schemas.

**Marley today: inherits Zed's, probably on.**

- `telemetry.metrics` and `telemetry.diagnostics` default to `true` in Zed's
  `assets/settings/default.json:1665`, and the dev box's Marley settings do not override them.
- Marley's `logs/telemetry.log` holds Zed metric events (frame-budget reports).
- Zed's flush posts them to `build_zed_api_url("/telemetry/events")`
  (`crates/client/src/telemetry.rs:637-657`), with an empty checksum when no seed is compiled in.
- I did not confirm whether a request leaves the machine. See Open questions.

#### A12. Orca's practices against Marley's workflow

| Concern | Orca | Marley |
|---|---|---|
| Work flow | PRs, the PR template, CI gates, human review; no phase model | Plan → Code → Test → Complete, enforced by hooks |
| Proof of a change | Unit tests plus e2e specs; before/after visuals in the PR | One e2e visualization scenario per ticket, shots read by the agent |
| Regression | Routed e2e per PR, full suite twice a day, golden suites gate releases | None: a scenario runs only at its own ticket's Test phase |
| Static gates | oxlint (several configs), type-aware lint, anti-slop plugin, changed-lines gates, ratchets with baselines | 16 gates in `script/gates.sh`: fmt, clippy pedantic and nursery, dylint, cargo-shear, typos, semgrep, docs and more; "no baselines, no suppressions" |
| Architecture rules | Boundary, ratchet and proof tests with allowlists, pins and self-tests | grep meta-gates, semgrep, dylint; no allowlist ratchets |
| Evidence | Audit folders with an executable before/after; one review-evidence file; the gate registry | Pipeline notes, the knowledge ledger |
| Knowledge | 52 reference docs, pointed to from `AGENTS.md` and directory `AGENTS.md` files | An append-only ledger, recalled by grep |
| Upstream | Patch files regenerated from a pinned upstream, checked in CI (`docs/reference/xterm-patch-regeneration.md`) | The Zed-touchpoints ledger (gate:16) and `// Marley:` comments |
| Release | Five channels, a signed multi-OS cut, post-release e2e | None yet (#502) |
| Crash and hang | Local minidumps, text signatures, a watchdog marker, opt-in bundle | Zed's hang detector on; the crash handler off on `dev` |

Marley is ahead in two places:

- The receipt-bound gate at commit (§15) is harder to fake than Orca's CI checks.
- "No baselines" keeps the Marley crates clean. That is affordable because the Marley surface is
  small, while Orca needs baselines and changed-lines gates to add rules to 1.77M lines.

Orca is ahead in regression, in machine-checked architecture rules, and in keeping evidence.

### Part B: the real feature list (the changelog)

#### B0. What was read and how

**The range.** I read every stable release body from v1.4.113 (2026-07-01) to v1.4.212
(2026-09-25): 89 releases, about three months and more than twice the 40 asked for. I also
skimmed the mobile-android pre-releases v0.0.25–v0.0.50, which are auto-generated lists of every
PR between mobile tags (desktop PRs included), so only their mobile items are kept.

**What counts as a feature.** An entry is:

- every `feat` line;
- every unprefixed line that adds or changes a capability;
- a fix only when it changes what the user can do or see.

Performance, test and plain bug-fix lines are left out. A feature is listed once, at the release
where it first shipped, with the releases that extended it.

**The docs check.** Each feature was grepped for in `docs/site/content/docs/` (57 MDX pages) and
the hits read. `NOT IN DOCS` means no page describes it. `partial` means the page covers the area
but not this capability.

**How the notes are written.**

- From about v1.4.194 the notes are curated: a "short version" paragraph, then product areas with
  a one-line italic summary each, and performance and reliability collapsed into `<details>`.
- Before that they are mostly GitHub's generated "What's Changed" lists. Some are hand-written:
  v1.4.149 is hand-grouped, v1.4.158 is a "stable patch from v1.4.157-rc.1", and v1.4.170 opens
  with a summary paragraph.
- From about v1.4.188 every body opens by saying a landed PR takes 48–72 hours to ship.

**The volume.**

- The feat-prefixed lines alone number 111 in July (45 releases), 93 distinct feat commits in the
  August window, and 139 in September (17 releases).
- Many capabilities ship without the prefix: Orca Relay (#8536), per-workspace environments
  (#6320), the Draw markup tool (#6335) and the menu bar item (#9042) are examples.
- Counting those, a release carries anywhere from one change (the patch releases) to several dozen
  (v1.4.198, v1.4.206).

#### B1. The feature list, by area

Tags are the first stable release that shipped the feature. PR numbers are `stablyai/orca` PRs.

##### Agents

*September (v1.4.194–v1.4.212)*

- **Structured native chat runtime for Claude and Codex.** Claude moved onto the Claude Agent SDK,
  is enabled on macOS and Linux, and renders through the same UI as Codex. The setting is
  "Use updated structured native chat" under Settings → Experimental → Chat UI; it is local-only,
  and WSL/SSH stay on terminal chat — v1.4.198, extended v1.4.199 (native Windows Codex #18519)
  (#18560, #18743) — NOT IN DOCS (`agents/native-chat.mdx` describes only the older terminal-backed
  Chat UI)
- Agent file edits shown as inline diff cards in chat — v1.4.198 (#18765) — NOT IN DOCS
- Split and move-to-pane actions in Chat UI mode; the terminal/chat switcher kept for bridge chat
  only — v1.4.198 (#18714, #18532) — partial: agents/native-chat.mdx
- Codex tool rows labelled by what the command did; Codex MCP and web-search calls shown as items
  — v1.4.198 (#18760, #18763) — NOT IN DOCS
- Background work in chat: Claude background task status, stop monitored tasks one at a time,
  Codex background tasks in the chat strip — v1.4.198 (#18757, #18807), extended v1.4.200
  (#19346) — NOT IN DOCS
- Chat says when a structured launch fell back to a terminal — v1.4.198 (#18762) — NOT IN DOCS
- Clickable document paths and links in chat, with the link-action popover — v1.4.198 (#18712),
  extended v1.4.199 (#19130) — partial: terminal.mdx (popover documented for terminal links only)
- Resume an Agent Session History row into a new structured chat — v1.4.198 (#18933), extended
  v1.4.199 (#19176) — partial: agents/session-history.mdx (Resume opens a terminal only)
- **Structured transcript.** It adds:
  - tool calls grouped into batches;
  - per-row execution details;
  - Codex subagent activity instead of opcode rows;
  - compaction notices, plan documents and images rendered;
  - hover timestamps;
  - provider activity in turn tails;
  - a turn-scoped activity indicator.

  — v1.4.199 (#19372, #19226, #18773, #19228, #19218, #19055, #19044) — NOT IN DOCS
- Structured `/clear` and `/compact` — v1.4.199 (#19164) — NOT IN DOCS
- Structured chat tabs can be renamed; model and effort picks are remembered; the slash commands
  and skills a Claude session loaded are listed; the workspace is auto-renamed on the first turn —
  v1.4.199 (#19153, #19147, #19127, #19138) — partial: agents/native-chat.mdx
- Structured session rewind (backend only in this window) — v1.4.199 (#19235) — NOT IN DOCS
- Claude subagent activity shown in chat — v1.4.200 (#18806) — partial: agents/claude-code.mdx
  (child rows for terminal sessions only)
- **Per-turn summary of changed files**, plus resolved prompt receipts — v1.4.200 (#19229) — NOT
  IN DOCS
- Task checklists with update diffs and a composer progress panel — v1.4.200 (#19230) — NOT IN
  DOCS
- Picker-selected skills shown as pills; effort pill moved right of the model pill — v1.4.200
  (#19616, #19617) — partial: agents/native-chat.mdx
- Images in chat: paste in structured Codex chat, previews of pending attachments, previews while
  pasted images save — v1.4.195 (#17498, #17517), extended v1.4.197 (#18118, #18266) — partial:
  agents/native-chat.mdx
- Live tool progress in chat — v1.4.196 (#17597), extended v1.4.198 (mobile, #18761) — NOT IN DOCS
- Agent-send startup delivery diagnostics and success announcements — v1.4.196 (#17814) — NOT IN
  DOCS
- File drag-and-drop into chat, with the whole pane as the drop target — v1.4.204 (#20494, #20561)
  — partial: agents/native-chat.mdx
- Provider-aware Fast mode in chat — v1.4.204 (#20506) — NOT IN DOCS
- **Chat restart recovery.** It covers:
  - a send stranded by a restart is checked against provider history;
  - structured chats resume cleanly;
  - Orca offers to reconnect chats that were working, and a setting resumes them automatically;
  - recovery stays visible in the status bar.

  — v1.4.203 (#20139), extended v1.4.204 (#20509), v1.4.206 (#21096), v1.4.210 (#21397) — NOT IN
  DOCS
- **Message rail** for jumping between your own prompts — v1.4.205 (#20719) — NOT IN DOCS
- Pending-prompt queue: precise cancel, queued messages delivered while the pane is hidden —
  v1.4.205 (#20601), extended v1.4.206 (#20659) — NOT IN DOCS
- OMP child conversations resume from session history under their saved names; nested OMP history
  expanded — v1.4.205 (#20629, #20636, #20663) — partial: agents/session-history.mdx
- **Session search.** It covers:
  - indexed search in the history panel, behind opt-in consent;
  - local index settings and controls;
  - search across every paired computer;
  - indexing turned on for paired servers from a client;
  - an `orca search` CLI;
  - results sorted newest first.

  — v1.4.206 (#20580, #20582, #20885, #20670, #20886, #20887, #20514, #20677), groundwork v1.4.203
  and v1.4.204 (#20029, #20277, #20516), extended v1.4.209 (#21863) — partial:
  agents/session-history.mdx (documents only the list filter box)
- OpenCode 2 beta support — v1.4.206 (#21418), extended v1.4.209, v1.4.210 (v2 plugins, #22078),
  v1.4.211 (session_v2 usage, #22391) — partial: cli/orchestration.mdx names the agent id;
  agents/supported.mdx does not list it
- A proposed plan renders as a plan, not a generic approval; the question tool shows as an
  awaiting-input row — v1.4.206 (#21090, #20724) — partial: agents/native-chat.mdx
- OMP model discovery and switching in chat, desktop and mobile — v1.4.206 (#20612) — partial:
  agents/native-chat.mdx
- **One agent-launch executor, exposed as `agent.launch`.** A launch delivers the launch prompt to
  terminal agents, carries inputs the host cannot derive, reports the pane it created, and can
  reserve that pane — v1.4.206 (#19849), extended v1.4.209 (#21891), v1.4.210 (#22037, #22108),
  v1.4.211 (#22291) — NOT IN DOCS
- A finished structured turn collapses to its answer; a failed turn ends instead of hanging; unread
  indicators light when a chat finishes — v1.4.210 (#22029, #22047, #21924) — NOT IN DOCS
- Notification on every settled structured turn — v1.4.211 (#22105) — NOT IN DOCS
- Codex goal shown above the composer and settable from goal mode — v1.4.211 (#22377) — NOT IN
  DOCS
- A structured session with live child work shows as working — v1.4.211 (#22295) — partial:
  model/agents-sessions.mdx
- Muse Code as a first-class harness, with a local usage provider and model/effort for supervised
  workers — v1.4.211 (#22216, #22379, #22383) — partial: agents/supported.mdx lists it;
  usage-tracking.mdx omits Muse usage
- Antigravity as a supervised worker — v1.4.210 (#21705) — docs: cli/orchestration.mdx
- Usage pricing for GPT-6 Astra/Sol/Luna, Opus 5.5 and Fable 5.1, corrected GPT-5.6; Codex long
  context priced per request; the Codex cost total says when a model is omitted — v1.4.209
  (#22073, #22350), extended v1.4.211 (#22360) — partial: agents/usage-tracking.mdx
- Usage sources: OpenCode Go from the console API, OpenCode cache read/write totals, Claude polling
  kept for Fable accounts, Codex usage fresh after a reset — v1.4.209 (#21462, #21886, #21748),
  extended v1.4.210 (#22071) — partial: agents/usage-tracking.mdx
- **Accounts and providers.** MiniMax China routing with credential-expiry and region sync; Codex
  personal and enterprise accounts on one email told apart; kimi-code recognized as Kimi; Grok
  usage no longer 0% — v1.4.199 (#14929, #19250, #19279, #18634, #17936) — partial:
  agents/usage-tracking.mdx, agents/codex-hot-swap.mdx
- Agent CLIs installed outside a version manager are detected — v1.4.198 (#18336) — NOT IN DOCS
- A timed-out hook is reported as unverifiable and its process tree killed — v1.4.204 (#20559),
  extended v1.4.205 (#20576) — NOT IN DOCS
- Orchestration: multi-agent workflows and worker lineage survive an app restart; structured chats
  born in the app can be orchestrated; structured worker placement — v1.4.199 (#16904, #19121,
  #18827), extended v1.4.200 (#19431) — partial: cli/orchestration.mdx
- In v1.4.201's undocumented diff (B4): one `/` picker for every agent anywhere in the prompt
  (#19832); slash commands described from the provider's own report (#19928, #19929); a
  background-tasks strip naming each row by kind (#19311, #19705); native-chat subagents as sidebar
  child rows (#19807); a single live-turn indicator where "Thinking" means reasoning (#19977) — NOT
  IN DOCS

*August (v1.4.164–v1.4.193)*

- Chat UI renders OMP transcripts — v1.4.177 (#11523) — docs: agents/native-chat.mdx
- Chat UI model and reasoning-effort pickers for Grok; effort kept for models outside the seed
  catalog; Extra high effort per model — v1.4.178, extended v1.4.180, v1.4.186 (#12780, #13365,
  #14577) — docs: agents/native-chat.mdx
- Chat UI picks the Codex model straight from the pill — v1.4.174 (#12657) — docs:
  agents/native-chat.mdx
- Claude model list read from the installed Claude CLI on each host; Chat UI shows the model the
  terminal is running — v1.4.174, extended v1.4.176 (#12369, #12860) — docs: agents/native-chat.mdx
- Codex slash commands typed from the Chat UI composer; pickers open upward — v1.4.182 (#13685,
  #13754) — partial: agents/native-chat.mdx
- Codex Chat UI restructured as a structured transcript — v1.4.193 (#16729) — partial:
  agents/native-chat.mdx
- AskUserQuestion card in desktop Chat UI for agents on a paired headless server; the question
  keeps waiting while parallel tools finish — v1.4.169, extended v1.4.176, v1.4.180 (#12223,
  #12782, #13714) — docs: agents/native-chat.mdx
- Settings names the agents Chat UI supports — v1.4.192 (#16830) — partial: agents/native-chat.mdx
- Prime Agent supported as a TUI agent, with session history, status hooks and WSL status —
  v1.4.182 (#12935, #13384, #13430) — docs: agents/supported.mdx, agents/session-history.mdx
- **Agent status hooks switch on and off live**, with no restart, the Windows WSL hook relay
  included — v1.4.178 (#13361) — docs: agents/hooks-memory.mdx, settings.mdx
- **Hook events that fire while Orca restarts are delivered afterwards** — v1.4.192 (#16685) —
  partial: agents/hooks-memory.mdx
- Session history rows show the first user prompt; the agent filter gets Select All / Clear —
  v1.4.165, extended v1.4.168 (#12006, #12128, #12085) — docs: agents/session-history.mdx
- Delete a provider session from session history; live sessions cannot be deleted; WSL deletion is
  contained — v1.4.178, extended v1.4.182 (#13106, #13108, #13279) — NOT IN DOCS
- Session history scans sessions in SSH worktrees — v1.4.169 (#11004) — NOT IN DOCS (the page says
  remote workspaces browse local history)
- Session history finds OpenCode SQLite and Cline sessions; OMP task-subagent transcripts grouped
  under their parent — v1.4.176, extended v1.4.183, v1.4.192 (#12803, #13853, #16814) — partial:
  agents/session-history.mdx
- Terminal tab titles show the session's conversation name — v1.4.176 (#12778) — docs: terminal.mdx
- Copilot and Kimi Code sessions resume after an app restart — v1.4.190 (#15879, #15883) — partial:
  agents/session-history.mdx
- Sleeping a workspace keeps finished and interrupted sessions resumable — v1.4.169 (#12214) —
  docs: agents/hibernation.mdx
- Auto-hibernation also parks idle agents in the active worktree; only the tab on screen and tabs
  inside the idle window are exempt — v1.4.191 (#16591, #16430) — **the docs contradict it**:
  agents/hibernation.mdx still says the active worktree is exempt
- Agents do not sleep while an orchestration dispatch is unsettled — v1.4.164 (#11808) — docs:
  agents/hibernation.mdx
- Agent Dashboard toggle shortcut — v1.4.190 (#15353) — docs: model/agents-sessions.mdx
- Agent Dashboard badges for SSH and remote hosts — v1.4.184 (#14177) — docs:
  model/agents-sessions.mdx
- **Agent Dashboard highlights agents waiting for input**; the question state gets its own color
  and glyph — v1.4.182, extended v1.4.185 (#13523, #14248) — docs: model/agents-sessions.mdx
- Experimental agent map view (orchestration edges, lineage, flares for unread finishes and new
  questions), removed again in v1.4.190 — v1.4.176, extended v1.4.177, v1.4.182, v1.4.185 (#12168,
  #13087, #13503, #14197, #14338, #14349, #15853) — NOT IN DOCS
- **Status detection.** It adds:
  - Claude background monitoring as its own state;
  - Claude turn-complete announced while background work runs;
  - the model each Codex subagent runs;
  - what an OpenCode permission request waits on;
  - the OMP ask tool shown as blocked;
  - Claude Code's quarter-circle spinner counted as working.

  — v1.4.183, extended v1.4.184, v1.4.187, v1.4.191 (#13925, #14114, #14580, #14614, #14627,
  #16201) — partial: model/agents-sessions.mdx, agents/codex.mdx, agents/claude-code.mdx
- Agent state glyphs carry labels; monitoring shows as a heartbeat — v1.4.193 (#16981) — NOT IN
  DOCS
- Smart sort ranks done agents by completion time — v1.4.183 (#13899) — NOT IN DOCS
- Claude Code Agent Teams run through an Orca tmux shim, which must name an absolute Orca CLI path —
  v1.4.185 (#14438) — partial: agents/claude-code.mdx (the shim is not described)
- Share private skills or bundles between hosts through an unlisted, revocable link — v1.4.187
  (#14934, #15000) — docs: cli/skills.mdx
- Skills discovered from the Hermes home — v1.4.190 (#15862) — NOT IN DOCS
- Skill install target picker becomes a searchable combobox — v1.4.191 (#16380) — partial:
  cli/skills.mdx
- Orchestration: the coordinator releases settled worker terminals — v1.4.169 (#12355) — docs:
  cli/orchestration.mdx
- Orchestration: per-worker model and effort overrides — v1.4.176, extended v1.4.185 (#12851,
  #14281) — docs: cli/orchestration.mdx
- **Orchestration.** A worker blocked on a human prompt is reported to the coordinator; worker-exit
  escalations reach lightweight coordinators; the reason a terminal's process is gone is recorded;
  unsupervised dispatch lanes are exposed — v1.4.187 (#15261, #15235, #15244, #15105) — NOT IN DOCS
- Orchestration: nested worker depth enforced, shown and carried across hosts — v1.4.192 (#16668,
  #16669) — NOT IN DOCS
- Usage cost: Claude 1-hour cache writes billed at 2x base input — v1.4.192 (#16878) — partial:
  agents/usage-tracking.mdx

*July (v1.4.113–v1.4.163)*

- Claude Agent Teams: the tmux shim for `orca claude-teams` supports `respawn-pane` — v1.4.113
  (#6771) — partial: agents/supported.mdx
- Chat UI (then "native chat"): a per-pane terminal/chat toggle for Claude and Codex, plus composer
  handling for paste, drafts, scrollbar and focus — v1.4.113 (#6781, #6783) — docs:
  agents/native-chat.mdx
- File links inside chat messages open in Orca — v1.4.115 (#6998) — NOT IN DOCS
- Launch prompts show in the transcript — v1.4.124 (#7444) — partial: agents/native-chat.mdx
- Chat UI on remote runtimes: sessions go through the remote runtime transport; attachments upload
  over SSH — v1.4.124, extended v1.4.129 (#7478, #7832) — docs: agents/native-chat.mdx
- Chat UI on desktop, mobile and web, with per-model option pickers and a verified Claude model
  switch — v1.4.144, extended v1.4.145 (#5824, #9085, #9084) — docs: agents/native-chat.mdx,
  mobile.mdx
- Chat UI keeps chats and drafts through reconnects — v1.4.147 (#9242) — docs:
  agents/native-chat.mdx
- Chat UI skill picker that discovers skills per host — v1.4.147 (#9480) — docs:
  agents/native-chat.mdx
- "Native chat" renamed "Chat UI"; the composer grows to 8 lines; draft launch context shown —
  v1.4.155, extended v1.4.160, v1.4.162 (#10036, #10848, #9802) — docs: agents/native-chat.mdx
- Hibernation: the agent's screen comes back on wake; panes wake when revealed; only real input
  counts as activity; typed drafts protected — v1.4.113, extended v1.4.121, v1.4.122 (#6833, #7145,
  #7283) — docs: agents/hibernation.mdx
- Claude Fable weekly usage meter for OAuth accounts, with reset countdown — v1.4.118, extended
  v1.4.120–v1.4.122 (#7079, #7167, #7190, #7212, #7294) — docs: agents/usage-tracking.mdx
- MiniMax usage and rate-limit tracking — v1.4.124 (#7387, #7411) — docs:
  agents/usage-tracking.mdx, settings.mdx
- Grok usage: weekly credits, then unified-billing monthly usage — v1.4.132, extended v1.4.142
  (#7869, #8769) — NOT IN DOCS
- Antigravity usage in the status bar — v1.4.135 (#7996) — NOT IN DOCS
- Status-bar usage: % used or % remaining, tooltips, a live session-reset countdown — v1.4.136,
  extended v1.4.147, v1.4.150 (#7574, #8319, #9374, #9693) — docs: agents/usage-tracking.mdx
- Usage roster: all agent usage in one status-bar popover — v1.4.149 (#8761) — docs:
  agents/usage-tracking.mdx
- OpenCode Go usage through a session cookie jar — v1.4.155 (#8047) — NOT IN DOCS
- Token pricing for the Claude 5 family and GPT-5.6 in Stats — v1.4.160 (#10822) — docs:
  agents/usage-tracking.mdx
- SSH session history knows its host — v1.4.123 (#7367) — partial: agents/session-history.mdx
- OMP and Antigravity sessions in session history — v1.4.128, extended v1.4.144 (#7618, #8971) —
  docs: agents/session-history.mdx
- Configurable Codex session-history home for host and WSL — v1.4.128 (#7629) — NOT IN DOCS
- Recoverable zero-turn sessions detected; empty sessions shown by default — v1.4.129 (#7889,
  #7853) — partial: agents/session-history.mdx
- Subagents displayed in session history — v1.4.133 (#7423) — NOT IN DOCS
- **Live-tail a session log** — v1.4.136, extended v1.4.138 (#8205, #8432) — partial:
  agents/session-history.mdx ("Open log" only)
- Session history view options persist — v1.4.144 (#8961) — partial: agents/session-history.mdx
- **Claude subagents as child rows under the lead**, with "done" held back until children finish;
  finished subagents and teammates reaped — v1.4.136, extended v1.4.139, v1.4.143, v1.4.152 (#8211,
  #8522, #8825, #9850) — docs: agents/claude-code.mdx
- Codex nested and v2 subagents as child rows — v1.4.150, extended v1.4.162 (#9637, #11059) —
  docs: agents/codex.mdx
- **"Needs you" detection.** Claude AskUserQuestion becomes a waiting state, Pi ask_user_question
  shows as blocked, Codex request_user_input maps to Needs You, and one question glyph is used
  everywhere — v1.4.129, extended v1.4.147, v1.4.152, v1.4.155 (#7852, #9457, #9861, #9996) — docs:
  model/agents-sessions.mdx
- Agent status in terminal tabs — v1.4.139 (#8142) — docs: terminal.mdx
- Agent rows labelled with conversation names — v1.4.156 (#9989) — docs: model/agents-sessions.mdx
- **Agent Dashboard.** It started as one pop-out window with attention, working and idle columns.
  It later gained finished time, a terminal dialog sized to its grid, an in-window or pop-out
  choice, state-tinted cards with project icon and chat name, and a status search board — v1.4.149,
  extended v1.4.150, v1.4.156, v1.4.160, v1.4.162 (#9604, #9686, #9997, #10243, #11012, #11042) —
  docs: model/agents-sessions.mdx
- Agent status over WSL through a hook relay in the guest — v1.4.132, extended v1.4.158 (#7903,
  #10328) — docs: agents/hooks-memory.mdx
- Droid and Copilot managed hooks install over SSH — v1.4.128 (#7744) — partial: ssh.mdx
- Settings changed inside Codex written back to `~/.codex` — v1.4.131 (#7960) — partial:
  agents/codex-hot-swap.mdx
- The user's global AGENTS.md shared into the managed Codex home — v1.4.138 (#7927) — NOT IN DOCS
- Codex multi-account homes, one per account; resume stays on the creating account — v1.4.149
  (#9501, #9613, #9696) — docs: agents/codex.mdx, agents/codex-hot-swap.mdx
- Codex "Continue in New Session" with a handoff prompt from the prior transcript — v1.4.152
  (#9170) — docs: agents/codex.mdx
- A stalled Codex config sync is reported — v1.4.156 (#10449) — docs: agents/codex-hot-swap.mdx
- Pi session resume — v1.4.145 (#8876) — docs: agents/session-history.mdx
- Trae CLI as a TUI agent — v1.4.162 (#10763) — docs: agents/supported.mdx
- Orchestration: dispatch task preview; worker tab titles refresh — v1.4.132 (#8024) — partial:
  cli/orchestration.mdx
- Orchestration group routing (@grok, @cursor) — v1.4.138 (#8058, #8436) — docs:
  cli/orchestration.mdx
- Orchestration: tasks complete on worker_done; `ask` works as a long-poll — v1.4.138, extended
  v1.4.156 (#8030, #9351) — docs: cli/orchestration.mdx
- Orchestration primitives rebuilt, with workers on connected servers — v1.4.160 (#9925) — docs:
  cli/orchestration.mdx

##### Worktrees, workspaces and layout

*September*

- One sidebar Create button holding New workspace and Add project — v1.4.199 (#19375) — NOT IN
  DOCS (first-session.mdx still says "Click Add Repo")
- New Workspace composer: choose a base ref; the compact branch picker is back — v1.4.206 (#17250,
  #21741) — partial: model/worktrees.mdx
- **Checkouts prepared while the composer is open**; the create's own git runs before background
  preparation — v1.4.195 (#17290), extended v1.4.198 (#18951, #18967), v1.4.206 (#20722) —
  partial: model/worktrees.mdx
- WSL projects get their worktrees inside the distro — v1.4.195 (#17387) — NOT IN DOCS
- **Worktree removal blocked when the archive hook fails** — v1.4.205 (#20153) — NOT IN DOCS
- Workspace names you set survive branch changes — v1.4.196 (#17448) — NOT IN DOCS
- Copy the workspace name from the sidebar context menu — v1.4.211 (#22338) — NOT IN DOCS
- Sidebar nesting by drag, with animated reordering — v1.4.203 (#20412) — partial:
  model/worktrees.mdx
- Sleeping workspaces distinguishable in the new card style — v1.4.206 (#21540) — partial:
  settings.mdx
- Agents-sidebar search visibility persisted per pairing; pinned rows labelled with their host —
  v1.4.199 (#19313, #19351) — NOT IN DOCS
- Tab-strip scrollbar; minimum tab width 72 px — v1.4.198 (#18526, #18871) — NOT IN DOCS
- Floating workspace: Markdown files opened from the OS land there; it stays above a working chat
  pane — v1.4.197 (#17906), extended v1.4.198 (#18692) — partial: settings.mdx
- **Cmd-J.** It adds:
  - palette layout with keyboard-clickable hints;
  - show-more for recent tabs on an empty query;
  - a filter seeded from the sidebar scope;
  - better ranking.

  — v1.4.194 (#17273, #17272), extended v1.4.195, v1.4.199 (#17693, #19036, #19005) — docs:
  model/quick-open.mdx
- **The Agents activity view left Experimental.** It gained filters (unread, grouping), groups
  ranked by attention, and working agents under "unread only" — v1.4.197 (#18222, #18255), extended
  v1.4.199 (#19329, #19535, #19547) — partial: activity.mdx (settings.mdx still lists it as
  experimental)
- A deliberately slept workspace stays cold until woken (v1.4.201 diff, #20075) — NOT IN DOCS

*August*

- Pick a parent workspace in Create Workspace — v1.4.192 (#15420) — docs: model/worktrees.mdx
- Set the project location on a setup-needed host from the host picker — v1.4.187 (#14965) — docs:
  model/worktrees.mdx
- Non-Orca worktree sources: per-source visibility, global defaults per host, one hidden-worktrees
  card — v1.4.184, extended v1.4.185, v1.4.186 (#13652, #14189, #14341, #14345, #14276) — docs:
  model/worktrees.mdx
- **Workspace cleanup reworked in the Resource Manager.** The changes:
  - nothing pre-selected;
  - each filter named and removable;
  - blockers shown as labels;
  - git status and PR/MR review pills on every row;
  - removal refused when the owning host is uncertain.

  — v1.4.185, extended v1.4.191, v1.4.192 (#13413, #14731, #15152, #15300, #16282, #16690, #16726)
  — partial: model/worktrees.mdx
- Review the branches git kept after a bulk delete — v1.4.182 (#13693; only in the mobile notes) —
  docs: model/worktrees.mdx
- Delete the hovered workspace from the keyboard (Cmd/Ctrl+Shift+Backspace) — v1.4.191 (#16271) —
  docs: model/worktrees.mdx
- **Workspace delete names the live PTYs** and offers force for a wedged sweep — v1.4.168 (#12153,
  #12394) — NOT IN DOCS
- Sleep with Descendants — v1.4.164 (#11810) — docs: model/worktrees.mdx
- Emoji in workspace names, with a shortcode picker — v1.4.164, extended v1.4.169, v1.4.182
  (#11845, #11888, #11837, #11843, #12058, #13429) — docs: model/worktrees.mdx
- **Cmd+J Jump Palette.** It adds:
  - a host and project filter;
  - Recent Chats & Terminals on Cmd/Ctrl+1–6;
  - open tabs interleaved with worktrees;
  - live attention badges;
  - recency ranking;
  - a stable result order while typing.

  — v1.4.174, extended v1.4.177–v1.4.191 (#12638, #13076, #13120, #12954, #13299, #15170, #15133,
  #15551, #16281, #16533) — docs: model/quick-open.mdx
- New-tab omnibox searches open tabs; filenames rank first; tab results show agent icons —
  v1.4.176, extended to v1.4.187 (#12670, #12679, #13100, #13454, #14677, #14824, #15134) —
  partial: model/quick-open.mdx
- Jump Palette search button in the sidebar nav — v1.4.190 (#15854) — docs: model/worktrees.mdx
- Sidebar filter menu with host and project submenus — v1.4.169, extended v1.4.174 (#12257, #12460,
  #12759) — docs: model/worktrees.mdx
- Hide workspaces other paired clients created on a shared server — v1.4.182, extended v1.4.186
  (#13718, #14579, #14595) — docs: model/worktrees.mdx, remote-servers.mdx
- Inline SSH reconnect on workspace cards — v1.4.169 (#12396) — docs: model/worktrees.mdx, ssh.mdx
- Link a Linear issue from Edit Worktree Details — v1.4.169 (#12380) — docs: model/worktrees.mdx
- Sidebar jump-to-top button — v1.4.183 (#13864) — NOT IN DOCS
- Pinned section headers collapse; pinning a parent shows its descendants — v1.4.168, extended
  v1.4.187 (#12147, #15035) — NOT IN DOCS
- Folder workspaces under every Group by mode — v1.4.187 (#15404) — NOT IN DOCS
- Worktree labels in a right-hand badge rail — v1.4.183 (#14313) — NOT IN DOCS
- The Workspace Board shortcut toggles the board — v1.4.184 (#14240) — docs: settings.mdx
- Drag-reorder tabs in the Floating Workspace — v1.4.183, extended v1.4.192 (#13995, #16828) — NOT
  IN DOCS
- A folder project becomes a git repo after `git init` outside Orca — v1.4.186 (#11480) — NOT IN
  DOCS
- `ORCA_WORKTREE_ADD_TIMEOUT_MS` raises the 180 s worktree-add timeout to up to 30 min — v1.4.186
  (#12823) — NOT IN DOCS
- Windows long paths for worktree creation — v1.4.190 (#15866) — NOT IN DOCS
- Windows setup scripts run in the configured Windows shell — v1.4.168 (#6967, #12406) — NOT IN DOCS

*July*

- **Localhost labels per worktree.** A loopback proxy serves `<worktree>.orca.localhost:<proxy-port>`
  in front of each dev server, so the same app in different worktrees can be told apart in any
  browser (`src/main/localhost-worktree-label-proxy.ts`) — v1.4.113 (#6424) — NOT IN DOCS
- Per-workspace environments (disposable runtimes created on demand, later "Cloud VM") and remote
  host setup in Add Project — v1.4.113, extended v1.4.124, v1.4.130, v1.4.163 (#6320, #6926, #7485,
  #7908, #11527) — docs: ways-to-run.mdx
- Shortcut for the active tab group's commands menu — v1.4.113 (#6325) — NOT IN DOCS
- Worktrees made with plain `git worktree add` appear in discovery — v1.4.118 (#7078) — docs:
  model/worktrees.mdx
- Manual sorting of Project Groups and project headers — v1.4.120, extended v1.4.122 (#7191, #7230)
  — partial: model/worktrees.mdx
- Custom worktree branch names — v1.4.121 (#6454) — docs: model/worktrees.mdx
- Editable sidebar hover title — v1.4.122 (#7307) — docs: model/worktrees.mdx
- Workspace names prefixed with the work-item id, reverted in v1.4.152 — v1.4.136 (#8238, #9821) —
  NOT IN DOCS
- APFS shared paths out of Experimental — v1.4.136 (#8318) — docs: model/worktrees.mdx
- Agents sidebar cards redesigned for scanning across worktrees — v1.4.138 (#8420) — partial:
  model/agents-sessions.mdx
- "Hide sleeping" never hides a workspace with an open agent session — v1.4.138, extended v1.4.145
  (#7511, #9101) — partial: model/worktrees.mdx
- Search worktrees by cached PR/MR title or number — v1.4.139 (#8505) — docs: model/quick-open.mdx
- **Cmd+Shift+T reopens closed tabs, terminals included** — v1.4.144 (#7445), extended v1.4.168
  (the right tab at its old position, #12236) — partial: browser/overview.mdx (browser tabs only)
- Keep pinned worktrees in their original lists — v1.4.145 (#6216) — partial: model/worktrees.mdx
- Common tab-switch chords for new users — v1.4.145 (#9240) — docs: model/tabs-panes-splits.mdx
- The Create worktree picker always offers "Add a new project" — v1.4.146 (#9251) — docs:
  model/worktrees.mdx
- **Workspace Board** (a kanban of workspaces by status). Children move with their parent, Move to
  Status syncs Linear, and it has search, virtualized lanes and more than 12 columns — v1.4.147,
  extended v1.4.156, v1.4.163 (#9472, #10176, #11244, #11269, #11605) — partial: settings.mdx
  (shortcut only)
- Coding-agent scratch worktrees never appear in the sidebar — v1.4.149 (#9535) — NOT IN DOCS
- Hosts still needing setup appear in the Run on picker — v1.4.152 (#7835) — docs:
  model/worktrees.mdx
- Sidebar filters hide automation-created, CLI-created and detached-HEAD workspaces — v1.4.155,
  extended v1.4.159 (#10000, #10712, #10786) — docs: model/worktrees.mdx
- **`.worktreeinclude`** copies listed gitignored paths into new worktrees (bounded from v1.4.158);
  **`worktree.sharedDirectories`** in orca.yaml shares gitignored directories — v1.4.156, extended
  v1.4.158, v1.4.162 (#9791, #10540, #10459) — docs: model/worktrees.mdx
- Desktop tab context menus with bulk close (Others, Left, Right) — v1.4.159 (#9323) — partial:
  mobile.mdx (mobile side only)
- Type-ahead Project and Run on pickers — v1.4.155, extended v1.4.162 (#10120, #11062) — docs:
  model/worktrees.mdx
- Emoji-only workspace names — v1.4.162 (#11057, #11063) — docs: model/worktrees.mdx
- The agent picked in the create dialog can become the default — v1.4.163 (#11443) — docs:
  model/worktrees.mdx
- Workspace cleanup list reworked and made safer — v1.4.132 (#7053, #8010) — docs:
  model/worktrees.mdx

##### Review and source control

*September*

- **Opt-in collapsed unchanged regions in file diffs** — v1.4.206 (#11955) — NOT IN DOCS
- **Multi-line range selection for diff comments**, by drag and keyboard — v1.4.206 (#20959) — NOT
  IN DOCS (annotate-ai-diff.mdx describes single-line comments)
- Show Whitespace toggle in the diff viewer — v1.4.197 (#15120) — NOT IN DOCS
- Large diffs wait until you choose to load them; the combined-diff file tree is windowed —
  v1.4.195 (#17521), extended v1.4.197 (#18236) — NOT IN DOCS
- Shift+wheel horizontal scroll in combined diffs — v1.4.199 (#11756) — NOT IN DOCS
- Stage, unstage and discard failures shown with Retry — v1.4.205 (#20423) — partial:
  review/commit-push.mdx
- The Fix broken checks prompt makes the agent confirm the cause before changing code — v1.4.199
  (#19435) — partial: review/github.mdx
- Source Control AI can generate with OMP — v1.4.206 (#20624, #21149) — partial:
  review/commit-push.mdx
- Bind a project to a specific `gh` account; Enterprise Managed User logins accepted — v1.4.206
  (#13664, #20450) — NOT IN DOCS

*August*

- **Stacked GitHub PRs.** Create a PR above the base's open PR, see a Stack #N map in the PR
  sidebar, and merge stack-aware, failing closed — v1.4.180, extended v1.4.183 (#13730, #13750,
  #13866) — docs: review/github.mdx
- Reactions on PR comments and review threads, CodeRabbit included — v1.4.182 (#13470, #13456) —
  docs: review/github.mdx
- Grouped PR comments newest-first; resolving a thread counts as the acknowledgement — v1.4.183
  (#13893, #13894) — partial: review/github.mdx
- Draft reviews get their own glyph and Ready / Close actions — v1.4.183, extended v1.4.193
  (#13919, #16889) — partial: review/github.mdx
- Copy buttons on check-run details — v1.4.192 (#16884) — NOT IN DOCS
- GitLab Checks load pipeline job traces, bridge and child jobs included — v1.4.169, extended
  v1.4.177 (#12266, #12863) — docs: review/github.mdx
- HTML preview in combined diffs — v1.4.177 (#12965) — docs: review/diff-viewer.mdx
- **Branch header chip with total lines changed, split into source, tests and generated** —
  v1.4.174, extended v1.4.175, v1.4.177 (#12771, #12842, #13057) — docs: review/commit-push.mdx
- AI PR details: linked-issue guidance and ELI5 sections; validated output; an empty description
  accepted — v1.4.174, extended v1.4.192 (#12613, #12752, #16873) — **the docs contradict it**:
  review/commit-push.mdx says empty descriptions are rejected
- Sending diff comments to an agent reworked — v1.4.169 (#12150) — partial:
  review/annotate-ai-diff.mdx
- Review notes persist for folder workspaces — v1.4.184 (#14112, #14234) — partial:
  review/annotate-ai-diff.mdx
- Upstream divergence counts clarified for rebased branches — v1.4.191 (#16358) — partial:
  review/commit-push.mdx
- Commit-message generation with Kimi — v1.4.178 (#11674) — partial: review/commit-push.mdx
- Removed: the terminal git/gh PATH wrapper used for attribution — v1.4.184 (#14141, #14255) — NOT
  IN DOCS

*July*

- Auto-merge UI clarified; auto-merge disabled for unstable PRs — v1.4.113, extended v1.4.124
  (#6777, #7415) — docs: review/github.mdx
- The Checks panel follows the active terminal's worktree — v1.4.113, extended v1.4.138 (#6799,
  #8349) — NOT IN DOCS
- **Git status refreshes when a shell command completes** — v1.4.118 (#7086) — NOT IN DOCS
- AI action recipes: per-repository overrides with a warning and management UI — v1.4.123 (#7386)
  — docs: review/commit-push.mdx
- Create PR shows detailed commit-failure summaries — v1.4.124 (#7401) — docs:
  review/commit-push.mdx
- External review link in the branch panel — v1.4.124 (#7461) — docs: review/github.mdx
- **AI recovery for failed pushes** — v1.4.129 (#7826) — partial: review/commit-push.mdx (commit
  failures only)
- Push and force push available with no upstream — v1.4.129 (#7864) — partial:
  review/commit-push.mdx
- Linked reviews must resolve an explicit push target — v1.4.138 (#6159, #8351) — NOT IN DOCS
- Mark a PR comment author as a bot for the Humans/Bots filter — v1.4.138 (#7598) — NOT IN DOCS
- Diff Previous/Next change buttons, F7 / Shift+F7 — v1.4.138 (#6668, #8240) — docs:
  review/diff-viewer.mdx
- Reply to any comment in a review thread — v1.4.139 (#8562, #8621) — docs: review/github.mdx
- Shortcut to add a markdown review note — v1.4.145, extended v1.4.146, v1.4.147 (#8250, #9257,
  #9412) — docs: editing/markdown.mdx
- Create PR fast-forwards branches that are only behind; remote errors inline — v1.4.146, extended
  v1.4.147 (#8534, #9481) — NOT IN DOCS
- The PR panel composer blocks on classified errors — v1.4.147 (#9428) — partial:
  review/commit-push.mdx
- Per-repo Source Control AI settings saved one way everywhere — v1.4.147 (#9427) — docs:
  review/commit-push.mdx
- Merge confirmation from the sidebar dropdown, reverted in v1.4.149 — v1.4.147 (#8911, #9636) —
  NOT IN DOCS
- Assignable "Send Review Notes to Agent" shortcut — v1.4.156 (#10070) — docs:
  review/annotate-ai-diff.mdx
- Current branch in the Source Control header (reverted, relanded) — v1.4.152, v1.4.156 (#9787,
  #10032, #10215) — docs: review/commit-push.mdx
- `{linkedIssue}` template variable for commit and PR generation — v1.4.158 (#10640) — docs:
  review/commit-push.mdx
- Copy Relative Path; Cmd+Enter commits — v1.4.159 (#9018, #9773) — docs: review/commit-push.mdx
- The combined-diff file tree is resizable and does not remount diffs — v1.4.162 (#11088) — docs:
  review/diff-viewer.mdx

##### Browser

*September*

- **Page annotations editable inline in the tray** (comment and intent); delivered annotations
  cleared — v1.4.195 (#17511), extended v1.4.209 (#22060) — NOT IN DOCS (design-mode.mdx covers
  only the picker)
- Previews and browser tabs convert into each other in place — v1.4.194 (#16998) — NOT IN DOCS
- The app-wide HTTP proxy applies to browser sessions — v1.4.194 (#15536) — NOT IN DOCS
- Linked reviews open in the Orca browser — v1.4.194 (#17360) — partial: review/github.mdx
- **Orca's browser offered for SSH terminal links** — v1.4.194 (#16503) — partial: terminal.mdx and
  browser/overview.mdx still say SSH-owned links open in the system browser only
- Cookie import: search in the picker, Chromium SameSite values decoded (135 cookies per profile had
  been dropped), scoping off the stale public-suffix list — v1.4.196 (#17627), extended v1.4.203
  (#20076, #20421) — docs: browser/profiles.mdx (import only)
- Favicons on browser entries — v1.4.197 (#18099) — NOT IN DOCS
- Browser identity: Electron's own UA so Cloudflare Turnstile clears, then one process-wide
  Cleaned/Native choice — v1.4.198 (#18749), extended v1.4.206 (#20767) — docs:
  browser/profiles.mdx
- Viewport presets: oversized ones scroll; not scaled by UI zoom — v1.4.196 (#17569), extended
  v1.4.206 (#20962) — partial: browser/overview.mdx
- Screenshot markup works on client-hosted pages — v1.4.206 (#19577) — NOT IN DOCS
- **Inactive pages deferred across worktree switches**; new background tabs load when opened —
  v1.4.199 (#19326), extended v1.4.200 (#19633) — partial: browser/overview.mdx
- Browser shortcuts stay with the split or floating panel that received them — v1.4.211 (#22340,
  #22361) — NOT IN DOCS
- First address-bar click selects the full URL; immediate reload feedback; themed loading surfaces
  — v1.4.195, v1.4.198, v1.4.199 (#17635, #18738, #19118) — NOT IN DOCS

*August*

- Remote HTML documents render locally over an `orca-preview` scheme with explicit folder grants —
  v1.4.192 (#16679, #16920, #16921, #16975) — docs: editing/viewers.mdx
- **Client-hosted remote browser**: pages of a paired-server workspace render on this desktop while
  their traffic goes through the host — v1.4.191 (#15448) — docs: browser/overview.mdx,
  settings.mdx
- `target=_blank` links and unnamed popups open as new Orca tabs — v1.4.192 (#16720) — docs:
  browser/overview.mdx
- Closing a browser tab returns to the previously active tab — v1.4.191 (#16306) — NOT IN DOCS
- WebAuthn account picker for security-key passkeys — v1.4.187 (#14687) — docs:
  browser/profiles.mdx
- Hard Reload in the reload menu — v1.4.174 (#12483) — docs: browser/overview.mdx
- Browser identity: native-UA profiles; a Firefox UA on Google auth hosts — v1.4.174, extended
  v1.4.177, v1.4.182 (#12608, #12884, #12849) — docs: browser/profiles.mdx
- **Google sign-in only directly in Orca.** FedCM is disabled, Google cookies are excluded from
  imports, a post-import notice names the host to sign in on, and there is one identity across
  hosts — v1.4.182, extended to v1.4.186 (#14023, #13666, #13667, #13670, #14085, #15221, #15481) —
  docs: browser/profiles.mdx
- Cookie import replaces cookies only for the imported domains, reports undecryptable cookies,
  keeps `__Host-` host-only — v1.4.168, extended v1.4.187, v1.4.188 (#12166, #14683, #15030,
  #15375) — partial: browser/profiles.mdx
- Permission denial notices name the frame and the permission — v1.4.188 (#15542) — NOT IN DOCS
- Non-URL address-bar text searches Google — v1.4.182 (#13863) — docs: browser/overview.mdx

*July*

- Helium as a cookie-import source — v1.4.113 (#6768) — NOT IN DOCS
- Copy in the page context menu — v1.4.121 (#7159) — NOT IN DOCS
- CDP Page.printToPDF (`orca pdf`) — v1.4.121 (#7044) — docs: cli/reference.mdx
- **Design annotations go to an existing agent session or start a new one** — v1.4.129, extended
  v1.4.136 (#7347, #8309) — NOT IN DOCS
- **External CDP clients can attach to Orca's browser targets** — v1.4.132 (#7891) — NOT IN DOCS
- Popups: pointer lock granted, trusted embedded popups allowed, others in an Orca window with an
  origin bar — v1.4.135, extended v1.4.138, v1.4.144 (#7697, #7392, #8343, #8782) — partial:
  browser/overview.mdx
- **"Draw": screenshot markup over a frozen image of the page** (pen, highlighter, arrow,
  rectangle, ellipse, text, undo/redo) — v1.4.142 (#6335) — NOT IN DOCS
- Local HTTPS: "Try HTTPS" and proceeding past certificate errors — v1.4.145, extended v1.4.147
  (#9104, #9070) — NOT IN DOCS
- Holding Shift inverts link routing — v1.4.163 (#10991) — docs: browser/overview.mdx

##### Terminal

*September*

- Choose the default terminal shell — v1.4.206 (#21085) — partial: terminal.mdx (Windows only)
- Interactive Unix shell arguments — v1.4.209 (#21904) — NOT IN DOCS
- **Find shows a match count**; Cmd+F focus parity — v1.4.206 (#9035) — partial: terminal.mdx
- URL click and middle-click configurable; Shift+middle-click paste and paste suppression in
  mouse-tracking TUIs — v1.4.206 (#21438), extended v1.4.209 (#21858, #21834) — partial:
  terminal.mdx
- **Inline images** via `@xterm/addon-image` — v1.4.206 (#19512) — NOT IN DOCS
- User-configurable contrast floor — v1.4.199 (#18126) — NOT IN DOCS
- Copy Session ID on terminal tab menus — v1.4.197 (#18039, #18070) — NOT IN DOCS
- **Confirm before stopping running terminals** (setting) — v1.4.206 (#21569) — NOT IN DOCS
- **Warning about remote work when closing the window or quitting** — v1.4.198 (#18593) — NOT IN
  DOCS
- The terminal theme overrides Ghostty colors — v1.4.210 (#22069) — partial: terminal.mdx
- **Linux: the terminal daemon in its own systemd scope**, so a service restart no longer kills
  PTYs; descendants reaped at shutdown — v1.4.210 (#19430, #22232, #22247) — partial:
  model/session-restore.mdx
- macOS: help when the terminal service cannot read a folder — v1.4.210 (#21923) — NOT IN DOCS
- Parked remote-pane scrollback kept across a hard restart; input preserved during replay —
  v1.4.199 (#19075), extended v1.4.209 (#21367) — partial: ssh.mdx, model/session-restore.mdx
- IME: composition kept across async updates, a drawn caret, preedit masking the placeholder, CJK
  preedit spacing — v1.4.194 (#17169, #17170, #17377), extended v1.4.206 (#19367) — NOT IN DOCS
- The agent gutter is left out of copied selections — v1.4.204 (#20545) — NOT IN DOCS

*August*

- **`orca terminal read --screen` returns the rendered frame** — v1.4.187 (#15380) — docs:
  cli/reference.mdx
- `orca terminal list` reports each terminal's execution host — v1.4.187 (#14973) — docs:
  cli/reference.mdx
- A click on a terminal link opens an anchored action popover with Copy link — v1.4.182, extended
  v1.4.183 (#13414, #13857) — docs: terminal.mdx
- Terminal file links reuse an already-open sibling-workspace tab — v1.4.169, extended v1.4.191
  (#11369, #16544) — docs: terminal.mdx
- OSC 8 links survive cold parking — v1.4.182 (#13382) — partial: terminal.mdx
- Kitty keyboard: IME commits as CSI-u under the all-keys flag; select-all and copy in kitty TUIs —
  v1.4.182 (#13310, #13940, #13388) — partial: terminal.mdx
- Korean, CJK and Windows IME rework — v1.4.166, extended to v1.4.192 (#12259, #12278, #12280,
  #15198, #15480) — NOT IN DOCS
- Bold font weight as its own setting — v1.4.185 (#14368) — NOT IN DOCS
- The running-process close confirmation covers every tab-close path — v1.4.169 (#12272) — NOT IN
  DOCS
- **Pastes into agent panes are bracketed**, so a pasted newline cannot submit the draft — v1.4.186
  (#14456) — NOT IN DOCS
- macOS: held keys repeat — v1.4.192 (#15589) — NOT IN DOCS
- **Live panes recover after a renderer restart**; daemon scrollback restored from durable history
  — v1.4.176, extended v1.4.184 (#12776, #13061, #14193) — partial: model/session-restore.mdx
- Quick Commands: local and remote collections by host, copy per command — v1.4.178, extended
  v1.4.182 (#13094, #13727, #13768) — docs: terminal.mdx, settings.mdx

*July*

- Default browser action for terminal links, reverted two releases later — v1.4.121 (#6129, #7378)
  — NOT IN DOCS
- Ghostty config import resolves theme references and paddings — v1.4.123 (#7126, #7125) —
  partial: terminal.mdx
- Terminals can start outside the worktree — v1.4.128 (#7750) — NOT IN DOCS
- Arabic/RTL shaped on the cell grid — v1.4.129 (#7665) — NOT IN DOCS
- The floating terminal shows an amber dot for unacknowledged bells and completions — v1.4.129
  (#7888) — partial: terminal.mdx
- Right-click paste on every platform — v1.4.137 (#8322) — NOT IN DOCS
- Plain-text `file://` links clickable — v1.4.147 (#9467) — partial: terminal.mdx
- Copied image files paste on Windows — v1.4.150 (#9640) — NOT IN DOCS
- OSC 52 copy on by default for Zellij and other TUIs — v1.4.158 (#10588) — docs: terminal.mdx
- A manual "parking" developer action — v1.4.160, extended v1.4.162 (#11016, #11091) — NOT IN DOCS

##### Remote and mobile

*September*

- **Mobile "OTA page": the desktop serves a mobile web bundle over the air** from a private origin.
  - What moved onto it: tasks, the file explorer and preview, session history, the session page,
    source control and review, the browser screencast, the terminal, Mermaid, HTML preview, the rich
    Markdown editor, dictation and haptics.
  - What it added: a build-time native/OTA switch, gzipped bundle ranges negotiated by capability,
    the device Back key, and failed updates shown in Troubleshoot.

  — v1.4.206 (#21417, #21694, #21710, #21596), extended v1.4.209–v1.4.211 (#21758, #21809, #21871,
  #21862, #21905, #21957, #21950, #21977, #22099, #22141, #22193, #22308, #22321, #22381) — NOT IN
  DOCS
- Mobile says why a host is unreachable; clearer notification opt-in; the auth-failed banner offers
  Re-pair — v1.4.206 (#21566, #20930), extended v1.4.211 (#22363) — partial: mobile.mdx
- Structured native chat on mobile, Codex then Claude — v1.4.198 (#18074, #18761), extended
  v1.4.199 (#18741) — partial: mobile.mdx
- Native mobile push notifications from paired desktops, shipped after being pulled once — v1.4.203
  (#20068) — docs: mobile.mdx
- Relay regions: near-region selection, rehoming, probe logging, far-cell alerts; an Asia cell —
  v1.4.199 (#19233, #19241, #19307, #19253), extended v1.4.211 (#22375) — NOT IN DOCS
- The relay tells the phone when its desktop is signed out — v1.4.198 (#18698) — partial:
  mobile.mdx
- `orca host list` shows each SSH host's platform — v1.4.198 (#18896) — docs: cli/reference.mdx
- File uploads to remote hosts stream — v1.4.205 (#16106) — partial: editing/file-explorer.mdx
- SSH: a rebuildable node-pty failure repaired once; every MFA stage answered; node-pty compiled
  from the host's Node headers — v1.4.197 (#17907, #17946), extended v1.4.198 (#18774) — partial:
  ssh.mdx
- Remote control outages shown on host surfaces; SSH transport problems told apart from runtime
  availability — v1.4.196 (#17531, #17710) — NOT IN DOCS
- Structured Codex chat on native Windows; WSL1 recognized — v1.4.199 (#18519, #19061) — NOT IN
  DOCS
- Mobile pre-releases: causal network diagnostics (#16837) and pairing recovery (#16659) in v0.0.47;
  structured Codex chat (#18074) in v0.0.48; structured Claude chat (#18741), native push (#19951,
  #20068), iOS text selection in chat (#19769) in v0.0.50 — partial: mobile.mdx (still links APK
  0.0.48)

*August*

- SSH host key verification against `known_hosts`, fingerprint on first contact,
  `StrictHostKeyChecking` honored — v1.4.187 (#14844) — docs: ssh.mdx
- FIDO2 security-key identities through system OpenSSH — v1.4.164 (#11913, #12029) — docs: ssh.mdx
- SSH host form in a modal, with an OpenSSH config host picker — v1.4.168, extended v1.4.169
  (#12172, #12334) — docs: ssh.mdx
- SSH worktrees listed at once from persisted metadata while the host connects — v1.4.176 (#12646,
  #12799) — docs: ssh.mdx
- A dropped SSH relay recovers on its own; a timed-out remote pane reconnects — v1.4.164, extended
  v1.4.168, v1.4.171 (#11999, #12213, #12432, #12216) — partial: ssh.mdx
- Orca Relay picks the closest region — v1.4.185 (#14366) — NOT IN DOCS
- **orcad: the Orca runtime as a 4.43 MB plain-Node bundle with no Electron.** It boots, pairs,
  registers repos, creates worktrees and runs PTYs; headless browser providers came later —
  v1.4.190, extended v1.4.191, v1.4.192 (#15968, #16193, #16368, #16369, #16398) — NOT IN DOCS
  (remote-servers.mdx covers only `orca serve`; `docs/reference/orcad-operations.md` is outside the
  site)
- `orca serve` signal forwarding and clean exits; a duplicate serve no longer crash-loops —
  v1.4.169, extended v1.4.185 (#12212, #14071, #14334) — partial: remote-servers.mdx
- Provisioned SSH roots for Cloud VM recipes — v1.4.185 (#14352, #14353, #14359) — partial:
  ways-to-run.mdx
- **The runtime listener stays on loopback until you pair over the network** — v1.4.168 (#11956,
  #12405) — NOT IN DOCS
- Pairing tolerates clock skew and never advertises virtual bridge addresses — v1.4.168, extended
  v1.4.177 (#12340, #12209, #12962, #13107) — NOT IN DOCS
- Task agents start on remote hosts — v1.4.180 (#13412) — NOT IN DOCS
- Quick Open searches paths on the remote host — v1.4.187 (#15158) — partial: model/quick-open.mdx
- Mobile Relay UX overhaul and escalation of persistent outages — v1.4.174, extended v1.4.175,
  v1.4.187 (#12609, #12796, #14986, #15071, #15237) — partial: mobile.mdx
- Mobile Chat UI model and session-option picker — v1.4.174 (#12366) — docs: mobile.mdx
- Mobile: tappable file links and path citations in chat — v1.4.174, extended v1.4.184 (#12364,
  #14166) — NOT IN DOCS
- Mobile: Mermaid, the TUI footer above the iOS keyboard — v1.4.169, extended v1.4.183 (#11185,
  #9178, #13856) — partial: mobile.mdx
- Mobile Source Control diff tabs and the linked PR — v1.4.174 (#12770, #12659) — partial:
  mobile.mdx
- Mobile host-card actions menu and a host picker for New Workspace — v1.4.176 (#11648, #11647) —
  docs: mobile.mdx
- Mobile "Close tabs to the right" removed — v1.4.190 (#15894) — **the docs contradict it**:
  mobile.mdx still lists it
- Taking the host back from mobile releases the phone-fit lock — v1.4.188 (#15473) — NOT IN DOCS
- The mobile emulator works in folder workspaces — v1.4.184 (#14009) — partial: cli/overview.mdx
- WSL: one runner for every `wsl.exe` call; transcript I/O time-boxed and quarantined — v1.4.184,
  extended to v1.4.190 (#14090, #14203, #15381, #15903) — NOT IN DOCS

*July*

- Tailscale recommended when a remote runtime is unreachable — v1.4.113 (#6637) — partial:
  remote-servers.mdx
- **Remote SSH terminals persist by default** — v1.4.113 (#6955) — docs: ssh.mdx
- SSH reconnect overlay in the terminal, later non-blocking — v1.4.115, extended v1.4.123, v1.4.152
  (#7009, #7344, #9928) — partial: ssh.mdx
- ControlMaster multiplexing for the system SSH transport — v1.4.117 (#6922) — docs: ssh.mdx
- Headless `orca serve`: bundled CLI installed, automations scheduler runs, safe relaunch after
  updates — v1.4.121, extended v1.4.123, v1.4.152 (#7267, #7296, #9634, #9785) — partial:
  remote-servers.mdx
- Manual network address entry accepts any hostname:port — v1.4.121 (#7223) — partial: mobile.mdx
- Binary files on a remote server render in the editor — v1.4.122 (#6606) — partial:
  editing/viewers.mdx
- Node.js prerequisite guidance for SSH — v1.4.124 (#6881) — docs: troubleshooting.mdx
- Desktop clients attached to a Remote Orca Server get their own SSH state — v1.4.129 (#7831) — NOT
  IN DOCS
- **The full Orca CLI bridged over the SSH relay** — v1.4.129 (#7771) — partial: cli/reference.mdx
- Provider account screens follow the active Remote Orca Server — v1.4.132 (#7999) — docs:
  remote-servers.mdx
- WSL workspaces open in VS Code remote — v1.4.132 (#7982) — NOT IN DOCS
- SSH workspaces open in VS Code Remote-SSH — v1.4.155 (#10005) — docs: ssh.mdx
- File transfers over system ssh; folder download from the remote explorer — v1.4.138, extended
  v1.4.147 (#7804, #7793) — docs: ssh.mdx
- Kerberos/GSSAPI hosts through system OpenSSH — v1.4.142 (#7507) — docs: ssh.mdx
- **Orca Relay**, an end-to-end-encrypted relay for desktop and mobile pairing (desktop sign-in
  required; LAN/Tailscale preferred) — v1.4.142, extended v1.4.156, v1.4.159 (#8536, #10147,
  #10709) — docs: mobile.mdx
- Paired Orca servers can be updated from the active client — v1.4.155 (#9839) — NOT IN DOCS
- Linux hosts that cannot compile node-pty still work for files, git and editor — v1.4.159 (#10776)
  — docs: ssh.mdx
- The remote filesystem picker shows Windows drives — v1.4.162 (#7439) — NOT IN DOCS
- SSH reconnect fan-out and relay PTY output bounded end to end — v1.4.162, v1.4.163 (#11003,
  #11005) — NOT IN DOCS
- **Mobile.**
  - Source Control hub — v1.4.113, extended v1.4.129, v1.4.132 (#6659, #7879, #7923) — docs:
    mobile.mdx
  - agent icons on terminal tabs — v1.4.113 (#6792) — NOT IN DOCS
  - terminal file links — v1.4.120 (#7134) — NOT IN DOCS
  - worktrees from every host in one view — v1.4.124 (#7500) — docs: mobile.mdx
  - the full file tree — v1.4.126 (#7289) — docs: mobile.mdx
  - a keyboard dismiss control — v1.4.129 (#5917) — partial: mobile.mdx
  - opening a worktree wakes its sleeping agents — v1.4.131 (#7906) — NOT IN DOCS
  - wedged Tailscale recovery and a per-host connection log — v1.4.132 (#7980, #7984) — NOT IN DOCS
  - session history — v1.4.135 (#6786) — NOT IN DOCS
  - no screen lock during dictation — v1.4.136 (#7746) — NOT IN DOCS
  - usage reset countdown — v1.4.139 (#7954) — docs: mobile.mdx
  - Windows Firewall repair for pairing — v1.4.138, extended v1.4.142 (#8439, #8846) — NOT IN DOCS
  - notification onboarding — v1.4.142, extended v1.4.149 (#8780, #9478) — partial: mobile.mdx
  - saved endpoints editable — v1.4.144 (#8294) — docs: mobile.mdx
  - PNG rendering — v1.4.144 (#9087) — NOT IN DOCS
  - Quick Commands — v1.4.147 (#9298) — docs: mobile.mdx
  - the sidebar button hideable once paired — v1.4.147 (#5785) — NOT IN DOCS
  - the Floating Workspace — v1.4.150 (#9523) — NOT IN DOCS
  - a protocol-compatibility block screen — v1.4.152 (#9780, #9782) — docs: mobile.mdx
  - Codex reset credits spent safely — v1.4.156 (#9394) — docs: mobile.mdx
  - attached images in chat — v1.4.156 (#10135) — docs: mobile.mdx
  - IPv6-only pairing — v1.4.156 (#9131) — NOT IN DOCS
  - bulk tab close — v1.4.159 (#9323) — docs: mobile.mdx
  - pairing address handling — v1.4.162, extended v1.4.163 (#9912, #10498, #11741) — partial:
    mobile.mdx

##### Editor

*September*

- Setting to turn off preview tabs — v1.4.211 (#22398) — NOT IN DOCS
- The file explorer finds files by name in large local workspaces — v1.4.211 (#22369) — partial:
  editing/file-explorer.mdx
- Copy button on code blocks — v1.4.205 (#20357) — NOT IN DOCS
- PDF zoom persists across tabs and restarts; CJK in PDFs — v1.4.206 (#21625), extended v1.4.209
  (#21879) — partial: editing/viewers.mdx (says PDF positions are session-only)
- Plain `<details>` blocks open in rich Markdown; the preview kept when following wiki links —
  v1.4.209 (#19784, #19790) — partial: editing/markdown.mdx
- File rename in the editor header becomes a breadcrumb morph — v1.4.210 (#21265) — NOT IN DOCS
- Rich Markdown size limit raised to 600 KB — v1.4.194 (#17288) — **the docs contradict it**:
  editing/markdown.mdx still says 300 KB
- New Markdown files get focus; domain-like paths treated as URLs; Salesforce Apex mapped —
  v1.4.197, v1.4.211 (#18071, #18340, #14287) — NOT IN DOCS

*August*

- Shift+Tab unindents list items and code lines in rich Markdown — v1.4.192 (#16677) — docs:
  editing/markdown.mdx
- Nested toggles editable in place — v1.4.192 (#16861) — docs: editing/markdown.mdx
- "Open anyway" puts an oversized Markdown file in the rich editor — v1.4.192 (#16971) — docs:
  editing/markdown.mdx
- Markdown table row and column controls — v1.4.174 (#11985) — docs: editing/markdown.mdx
- PDF tabs restore their scroll position — v1.4.168 (#12163) — docs: editing/viewers.mdx
- The code editor keeps selections across tab switches — v1.4.191 (#16132) — NOT IN DOCS
- Close All / Close Others no longer freeze on large tab sets — v1.4.168 (#12399, #12404) — NOT IN
  DOCS
- Natural sort of numbered names in the explorer — v1.4.176 (#11576) — docs:
  editing/file-explorer.mdx
- `.liquid` and `.jsp`/`.jspf` detection — v1.4.182, extended v1.4.190 (#12402, #15213) — NOT IN DOCS

*July*

- File explorer "Open in terminal" — v1.4.120 (#7029) — NOT IN DOCS
- Spellcheck setting for rich Markdown — v1.4.120 (#7103) — NOT IN DOCS
- The Markdown TOC keeps its state — v1.4.122 (#6476) — partial: editing/markdown.mdx
- Hard-wrapped prose reflowed and edited — v1.4.124 (#7407) — NOT IN DOCS
- "View file" in the file context menu — v1.4.125 (#4620) — NOT IN DOCS
- **Editor tabs pick up external file changes** instead of dropping them — v1.4.129 (#7591) —
  partial: editing/file-explorer.mdx
- HTML superscript links in Markdown — v1.4.136 (#8307) — NOT IN DOCS
- One confirmation for deleting several files — v1.4.137 (#7459) — NOT IN DOCS
- Word-wrap preference (Alt+Z) — v1.4.138, extended v1.4.159 (#8423, #10086) — docs:
  editing/monaco.mdx
- Front matter shown by default in the rich editor — v1.4.139 (#8623) — docs: editing/markdown.mdx
- Toggle Heading 2–5 — v1.4.142, extended v1.4.147 (#8822, #9448) — docs: editing/markdown.mdx
- The rich editor keeps the file's original Markdown style — v1.4.143 (#8862) — partial:
  editing/markdown.mdx
- Optional editor font family — v1.4.150 (#9658) — docs: editing/monaco.mdx
- Find starts from the selected text — v1.4.152 (#9982) — docs: editing/monaco.mdx
- Quick Open in projects over 10k files — v1.4.163 (#11440) — partial: model/quick-open.mdx
- Tab, Enter and Backspace as table keys in rich Markdown — v1.4.163 (#11724) — docs:
  editing/markdown.mdx

##### Integrations

*September*

- GitHub Projects Roadmap views as a timeline — v1.4.198 (#17795) — partial: review/github.mdx
- The Tasks page folded into one tree grouped by provider — v1.4.197 (#18008) — partial:
  review/github.mdx
- The CLI describes Linear write support — v1.4.211 (#21830) — docs: cli/reference.mdx
- Computer use on Windows as one persistent helper — v1.4.198 (#17858) — partial:
  cli/computer-use.mdx
- Skill guides rewritten; the orchestration skill description cut under the 1,024-character Agent
  Skills limit — v1.4.198, v1.4.199 (#18683, #19128) — docs: cli/skills.mdx

*August*

- **Bitbucket Cloud**: connect and create pull requests — v1.4.183 (#5832, #13887) — docs:
  review/github.mdx, settings.mdx
- Azure DevOps Server (on-prem) api-version retry — v1.4.176 (#12832) — partial: review/github.mdx
- GitHub Enterprise avatar fallback — v1.4.190 (#13981) — NOT IN DOCS
- Open GitHub items in the workspace composer, with issue automation — v1.4.174 (#12653) — docs:
  review/github.mdx
- GitHub Tasks: honest pagination and restored position; GitLab Tasks past 50 — v1.4.165, extended
  to v1.4.192 (#11584, #9068, #13096, #13538) — partial: review/github.mdx
- Linear list view and filters persist per device — v1.4.178 (#12710, #13391) — docs:
  review/linear.mdx
- Linear "Has Workspace" mode — v1.4.174 (#12632) — docs: review/linear.mdx
- Linear: pasted issue URLs recognized; "Start workspace" and "Open on Linear" actions — v1.4.184,
  extended to v1.4.191 (#14190, #14492, #15824, #16382) — partial: review/linear.mdx
- New Linear and Jira issue dialogs keep drafts — v1.4.168 (#12167) — docs: review/linear.mdx,
  review/jira.mdx
- Computer use: macOS middle click, Windows horizontal scroll, HID-tap coordinate clicks, no
  repeated screen-recording prompts — v1.4.176, extended to v1.4.187 (#12839, #12981, #13025,
  #13427, #14721, #14727) — NOT IN DOCS

*July*

- A repo's Custom GitHub Issue Command runs when a workspace starts from an issue — v1.4.120
  (#6827) — partial: review/github.mdx
- Agents launched from a Linear issue get its inline screenshots and media — v1.4.124 (#7484) —
  docs: review/linear.mdx
- GitHub issue drafts survive dismissal — v1.4.129 (#7778, #7887) — NOT IN DOCS
- Linear filters run server-side — v1.4.132 (#8021) — docs: review/linear.mdx
- Linear agent-skill setup with guided setup for task providers — v1.4.132, extended v1.4.145,
  v1.4.150, v1.4.163 (#7990, #9218, #9764, #11533) — partial: review/linear.mdx
- Jira issues grouped by status and sortable — v1.4.135, extended v1.4.136 (#7958, #7959) — NOT IN
  DOCS
- Skill content: launch the agent first when creating a worktree — v1.4.137 (#7957) — NOT IN DOCS
- GitHub issue types through issue templates — v1.4.138 (#8346) — NOT IN DOCS
- GitHub Enterprise Server: PR creation, diffs, avatars — v1.4.139, extended v1.4.145, v1.4.149
  (#8603, #9107, #8932) — partial: github-errors.mdx
- The CLI serves skill guides matching the installed version — v1.4.141 (#8624) — docs:
  cli/skills.mdx
- Skill freshness detection with an update rail; background updates — v1.4.144, extended v1.4.160
  (#8637, #10843) — docs: cli/skills.mdx, settings.mdx
- Worktrees use Linear's own branch names — v1.4.144 (#8617) — docs: model/worktrees.mdx,
  review/linear.mdx
- Self-hosted Jira Server/Data Center — v1.4.144 (#8976) — docs: review/jira.mdx
- Skills ship as thin "hybrid discovery" stubs — v1.4.149, extended v1.4.152 (#9238, #9846) — docs:
  cli/skills.mdx
- Linear CLI: activity history, MCP-compatible listing and save, relations — v1.4.150 (#9667, #9672,
  #9670, #9674) — docs: review/linear.mdx, cli/reference.mdx
- iOS emulator accessibility-tree (`ax`) command — v1.4.155 (#10007, #10029) — NOT IN DOCS
- Jira images in a lightbox — v1.4.160 (#8938) — NOT IN DOCS
- **Orca plugin system (experimental)**: kernel, content packs, panels, workers, a v0 marketplace;
  four trust-boundary holes closed one release later — v1.4.160, extended v1.4.162 (#8549, #11232)
  — partial: settings.mdx
- OMP as a skill discovery source — v1.4.163 (#6422) — docs: cli/skills.mdx
- Installed skills read from the connected remote runtime — v1.4.163 (#6887) — partial:
  cli/skills.mdx
- Jira issues linked from the create dialog — v1.4.163 (#11296) — docs: review/jira.mdx

##### Other: settings, CLI, automations, updates, diagnostics

*September*

- French UI locale; Korean and Japanese additions — v1.4.198, extended v1.4.199 (#16455, #15849,
  #18787, #18589) — partial: settings.mdx (omits French)
- HTTP/1.1 compatibility toggle; multiline proxy bypass rules kept — v1.4.198, extended v1.4.206
  (#18621, #20957) — NOT IN DOCS
- **Automations runs dashboard** with pagination and filtering — v1.4.194, extended to v1.4.198
  (#17284, #17626, #18226, #18885) — partial: cli/automations.mdx (the dashboard is not described)
- Improved notification view — v1.4.198 (#18699) — partial: notifications.mdx
- Artifact sharing limit 5 MiB, then 10 MiB — v1.4.196 (#17708, #17910) — NOT IN DOCS
- Updater: background check errors open from the status bar; the release picker's build list uses
  the gh token and is cached; dev builds sort by timestamp — v1.4.203, extended v1.4.206, v1.4.210
  (#20270, #21720, #21902) — partial: install.mdx
- CLI: `terminal close` as the canonical teardown; registration failures explained; a denied
  connection reported as a denial, not a dead Orca — v1.4.197, extended v1.4.211 (#18073, #18125,
  #22341) — partial: cli/reference.mdx
- Linux CLI gets one entrypoint by extracting the AppImage once — v1.4.197 (#15081) — partial:
  install.mdx
- Telemetry: macOS stale-daemon adoption and cwd denials measured, with the daemon's code identity
  — v1.4.197, extended v1.4.211 (#18043, #22171) — NOT IN DOCS (telemetry.mdx lists no daemon
  events)
- Failure reporting: pane load failures told apart from empty states; drop and microphone errors
  reported — v1.4.205 (#20735, #20795, #20704, #20801) — NOT IN DOCS

*August*

- **Artifacts**: account-backed sharing of HTML and Markdown as public links, off by default, with
  a management page — v1.4.178, extended to v1.4.192 (#13012, #13356, #13368, #13369, #13796,
  #15233, #17037) — docs: settings.mdx, browser/overview.mdx, editing/markdown.mdx,
  cli/reference.mdx
- Keep-awake control on the status bar (On / Agent / Off) — v1.4.182, extended v1.4.191 (#13480,
  #13751, #14775) — docs: settings.mdx
- Dictation: microphone choice and a sound-reactive visualizer — v1.4.164, extended v1.4.168,
  v1.4.190 (#11842, #12119, #16017) — partial: settings.mdx
- **Automations across all local and remote hosts.** It adds a host column, filter and picker;
  filters by state, outcome and agent; search; delete without an SSH connection; the full prompt in
  the detail view — v1.4.169, extended to v1.4.192 (#12561, #13261, #13975, #14158, #15224, #15805,
  #15884, #16067, #16532, #16552, #16665, #16667, #16823) — docs: cli/automations.mdx
- Korean localization of Automations, onboarding and the Agent Dashboard — v1.4.187, extended
  v1.4.191, v1.4.193 (#15004, #16212, #17108, #17121) — partial: settings.mdx
- A warning when macOS Mission Control captures digit chords — v1.4.186 (#14734) — NOT IN DOCS
- The Help menu always shows Onboarding and Restart Orca — v1.4.169 (#12379) — NOT IN DOCS
- Errors and feedback carry the Orca version and OS — v1.4.183 (#13851) — partial:
  troubleshooting.mdx
- Linux: Orca says when secrets are only obfuscated; secret writes fail closed when encryption is
  unavailable — v1.4.177, extended v1.4.190 (#12983, #16033, #16044) — NOT IN DOCS
- **Crash reporting.** Crashpad minidumps are captured with the failing CHECK named and pruned at
  startup; user crash notes are kept; concurrent deaths correlated; GPU identity captured — v1.4.177,
  extended to v1.4.193 (#12938, #14823, #14968, #15251, #15252, #16449, #16973) — NOT IN DOCS
- Diagnostics: renderer OOM breadcrumbs attribute memory; PTY output processors in the memory
  census; the code driving a React commit cascade named — v1.4.168, extended v1.4.176, v1.4.192
  (#12198, #12795, #16730) — NOT IN DOCS
- Artifacts and Skills pages join back/forward navigation — v1.4.190 (#15969) — NOT IN DOCS
- Compact native density for menus — v1.4.190 (#15924) — NOT IN DOCS
- The Tasks button always enabled, with an empty state — v1.4.192 (#16838) — NOT IN DOCS
- Updater: an adhoc channel for branch builds; hourly numbers restart per version; the reason an
  install failed shown — v1.4.165, extended v1.4.169, v1.4.174 (#12051, #12224, #12587) — partial:
  install.mdx (no adhoc or hourly)

*July*

- UI language applied at startup, findable by native-language terms; gettext PO as the canonical
  source — v1.4.121, extended to v1.4.163 (#7193, #9967, #10594, #11478) — docs: settings.mdx
- Check for Updates Cmd/Ctrl-click picks the latest perf prerelease (`vX.Y.Z-rc.N.perf`) — v1.4.124
  (#7278) — docs: install.mdx, settings.mdx
- macOS notification-permission onboarding step — v1.4.129 (#7684) — NOT IN DOCS
- **Optional Orca account sign-in** (OAuth with PKCE in the system browser; later required for Orca
  Relay, artifacts and skill sharing) — v1.4.130 (#7515) — partial: settings.mdx; telemetry.mdx
  still says "Orca has no account system"
- Voice dictation: resumable, verified model downloads; new Korean, SenseVoice and Parakeet models;
  a Stop button — v1.4.133, extended to v1.4.162 (#3922, #8775, #9893, #7436, #8207, #10735,
  #11152) — docs: settings.mdx
- **The CLI corrects agents.** It accepts conventional-verb aliases (`worktree remove` → `rm`),
  suggests "did you mean", lists valid flags, and prints the full schema with
  `orca agent-context --json` — v1.4.136 (#6303) — NOT IN DOCS
- **Resource Manager** lists browsers and never destroys a session it cannot prove idle — v1.4.136,
  extended v1.4.150, v1.4.160 (#8267, #8845, #10821, #10893) — partial: settings.mdx
- Native macOS menu bar status item with an amber activity dot — v1.4.144 (#9042) — NOT IN DOCS
- Imported pets (animated companions) — v1.4.145 (#8730, #9140) — NOT IN DOCS
- Update errors as a plain-language card with the raw error behind "Show details" — v1.4.146
  (#9248) — NOT IN DOCS
- Dev-instance tray icons carry a DEV marker — v1.4.146 (#9253) — NOT IN DOCS
- Check for Updates Option-click picks a validated local macOS build — v1.4.160 (#10889) — docs:
  install.mdx
- The updater reports releases still being published — v1.4.162 (#8914) — NOT IN DOCS
- **Diagnostics.** Renderer OOM reports name what grew; OS suspend/resume is recorded so sleep gaps
  are not read as freezes; a typing-latency self-diagnostic; main-thread hangs recorded — v1.4.156,
  extended to v1.4.162 (#9984, #10530, #10784, #10256) — NOT IN DOCS
- **Hourly macOS dev channel and a hidden release-channel picker** (Option-click the Updates header)
  to move between stable, RC and hourly or pin any build — v1.4.163 (#11250, #11708, #11812, #11817)
  — NOT IN DOCS
- `orca skills install|update` and `orca account add|list` for headless hosts — v1.4.163 (#9201,
  #9177) — docs: cli/skills.mdx, remote-servers.mdx
- The feedback dialog accepts images — v1.4.163 (#10465) — docs: troubleshooting.mdx
- The About panel shows whether GPU acceleration is on — v1.4.163 (#11722) — NOT IN DOCS

#### B2. What the docs site does not describe

The site is well kept for the core model (worktrees, SSH, the agent dashboard, GitHub, Linear,
automations). Its gaps cluster where the product moved fastest.

**Large capabilities with no page:**

- the structured native chat runtime (Claude Agent SDK and Codex), its restart recovery, and its
  message rail;
- the mobile OTA page;
- orcad;
- the hourly, daily and adhoc channels and exact-build pinning;
- crash reporting and diagnostics;
- the browser's Draw markup and editable annotations;
- per-worktree localhost labels;
- `agent.launch`;
- external CDP attach;
- the terminal's inline images;
- collapsed unchanged diff regions and multi-line diff comments;
- session search. The session-history page covers only the filter box.

**Pages that now contradict what shipped:**

- `agents/hibernation.mdx`: the active worktree is no longer exempt (#16591);
- `review/commit-push.mdx`: empty generated PR descriptions are now accepted (#16873);
- `mobile.mdx`: "Close Tabs to the Right" was removed (#15894), and the page links APK 0.0.48 while
  0.0.50 exists;
- `editing/markdown.mdx`: says 300 KB, but the limit is 600 KB (#17288);
- `editing/viewers.mdx`: says PDF positions are session-only, but zoom now persists (#21879);
- `telemetry.mdx`: says "Orca has no account system", which sign-in changed (#7515), and names only
  the `stable` and `rc` channels;
- `first-session.mdx`: says "Click Add Repo", but there is now one Create button (#19375);
- `terminal.mdx` and `browser/overview.mdx`: say SSH links open in the system browser only (#16503);
- `settings.mdx`: lists the Agents activity view as experimental after it graduated.

#### B3. Release-engineering facts the notes reveal

- **How stables are cut.** A stable is an RC cut from a main commit plus cherry-picks on a release
  branch. v1.4.158 was "promoted from v1.4.157-rc.1" on `release/1.4.157-rc.1-cherry`. A stable is
  not always a descendant of the previous stable: v1.4.180's base is main at #13286, and it does
  not contain v1.4.179.
- **Numbers never published**, and why:
  - 127, 140, 151, 153, 154 and 157 were cancelled or never promoted;
  - v1.4.172 was tagged, but its build failed on the relay watcher harness, and v1.4.173 carries
    the fix (#12754);
  - v1.4.181 exists only as -rc.0;
  - there is no v1.4.189 tag.
- **The notes can be wrong.** v1.4.201's notes are byte-identical to v1.4.200's, although the build
  is 85 commits ahead, so its own changes are undocumented. Seven items of v1.4.209 reappear in
  v1.4.210. Cherry-picks are routinely listed twice.
- **Prereleases moved out of the main repo.** RC tags stopped after v1.4.184-rc.0 (2026-08-16).
  The dev builds in their own repos replaced RCs as the pre-stable lane:
  - hourly, since v1.4.163 (#11250);
  - adhoc, since v1.4.165 (#12051);
  - a nightly cut, since v1.4.182 (#13410), which became the daily. Since #14870 the full e2e suite
    runs against it.
- **A daemon gate born from an incident.** v1.4.129-rc.1 shipped a terminal daemon that could not
  start (#7844). Since #7849, builds, packaging and CI fail when the daemon cannot start, and PR CI
  boots `orcad` and round-trips a terminal (`smoke:orcad-terminal`).
- **Agents in the release loop.**
  - A scheduled "auto e2e tests autofix" job (#18227) and a golden-suite autofix (#15379) open
    fixes.
  - A "pr-bug-scan validated finding" bot opens fix PRs (#7180, #6839, #8804).
  - `[P0]`/`[P1]`/`[P2]` tags appear in PR titles from v1.4.163.
- **Gates named in the notes, and when they landed:**
  - the reliability gate manifest (#7001, July);
  - the max-lines ratchet (#7608);
  - only changed e2e specs on PRs (#11834, v1.4.164);
  - the root-directory guard (#11903);
  - the cross-version harness (#12682) with stable baselines (#14156);
  - PR test-LoC counts (#14738);
  - a gate on Electron imports reachable from the runtime (#15919);
  - a real input-method PR gate (#17365);
  - a child-process import ratchet "made able to fail" (#18026);
  - the anti-slop oxlint plugin (#20726, #20780–#20786);
  - the RPC params-catalog parity gate (#20281).
- **Windows signing.** SignPath inner-binary signing was bypassed and restored more than once
  (#7137, #7866, #10719). The NSIS uninstaller is signed (#17868).
- **Linux.** An AppImage no longer leaks its environment into terminal shells (#7076). Ubuntu 20.04
  launch was restored by pinning node-pty glibc symbols, with a packaging gate (#10019). `.deb` and
  `.rpm` installs recover from a failed privilege escalation (#12183, #12395). No AUR change appears
  in any note.
- **Source maps.** Main-process source maps are published with each release (#17630).
- **Third-party requests.** Agent icons are bundled instead of fetched from Google's favicon service
  (#8474).

## 3. Bring to Marley

Ranked by value for the effort. Size: S = a day, M = a few days, L = a week or more.

| # | Take | Why it is worth it | Where it lands in Marley | Size | What makes it hard |
|---|---|---|---|---|---|
| 1 | **A golden scenario set, plus routing from changed paths to scenarios** | Marley's 25 scenarios each run once, at their own ticket's Test phase, so a change to `marley_browser` can break #496's picker and no step would notice. Orca runs most of its 15 golden specs as blocking release gates, and routes each PR to the specs that cover its paths (`config/scripts/pr-e2e-source-routing.mjs`). | A `script/e2e/routes` table (path glob → scenarios) that `/pipeline:test` runs next to the ticket's own scenario; a `script/e2e/golden` list and `just e2e-golden`, run at the sprint's last Complete and before `just install` swaps the binary (#502). | S for routing, M with machine checks | The cost is agent time spent reading shots. Give golden scenarios a machine check through Marley's own MCP server (curl the endpoint in the run's profile and call `terminal_read`, `fleet_snapshot` or the browser `snapshot` tool), so a passing run needs no reading and only failures are read. That is Orca's rule "reach the state through the store, prove it on the DOM", transposed: reach it by keys, prove it by the shot, triage by MCP. It amends §7, so it is Chad's call. |
| 2 | **#502 done safely (queued)** | On the `dev` channel Zed skips its single-instance check (`crates/zed/src/main.rs:360-363`). With #502's plan (the installed build shares the `dev` profile), the installed `marley` and a debug `marley` can run on one profile at once. Both then write the same workspace database, and whichever quits first deletes the `mcp-endpoint.json` the other still serves (`crates/marley_workbench/src/mcp.rs:102`). Orca's `orcad` refuses to start while another live runtime holds `orcad.lock`; its daemon never removes a name it did not create. | (a) An instance guard: enable Zed's Linux `listen_for_cli_connections` for Marley on `dev` (a Zed touchpoint), or a lock file in the data directory. (b) An owned endpoint: write pid and start time with the bearer, and remove it only if it is still ours. (c) `Exec=env ZED_GENERATE_MINIDUMPS=1 …` in the desktop entry, which turns on Zed's crash server writing minidumps into Marley's logs directory; with no `ZED_MINIDUMP_ENDPOINT`, nothing is uploaded (`crates/zed/src/reliability.rs:259-263`). (d) Keep the previous binary as `marley.prev` and put the git SHA in `--version`, so a bad build rolls back in one command (Orca keeps 72 hourly builds for bisecting). | S | The guard changes what a second launch does: it hands its paths to the running Marley. e2e runs use `--user-data-dir`, so their socket path differs and they are unaffected. |
| 3 | **Ratchet checks with the five devices** | Orca's best idea for code written by agents: a chokepoint rule held as data. The five devices are a shrink-only allowlist file, a failure on a stale entry, a literal pin equal to the real count, an anti-vacuity count, and a planted-violation self-test (A7). Marley's gate:12/13 greps have the shape but none of the devices. §14's "spawns live in adapter modules" is enforced by nothing. | New semgrep rules (gate:20) or a gate step in `script/gates.sh`, each with an allowlist under `.config/` and a planted fixture the gate must catch. Candidate rules: (1) spawn only in `marley_terminal`, `marley_mcp::transport`, `marley_browser::service` and the harness client; (2) no plain `fs::write` for a file another process reads, only the durable writer (row 4); (3) no gpui in the core of `marley_mcp` and `marley_browser`, which keeps a headless `marley serve` possible (Orca's runtime-electron ratchet); (4) every per-terminal map removed on close (Orca's reaper ratchet). | S per rule | grep over Rust is brittle, so semgrep or a dylint lint is the better engine. The reaper rule needs a lint that sees struct fields. |
| 4 | **A durable, owned file writer** | Zed's `fs.atomic_write` renames without fsync (`crates/fs/src/fs.rs:972`), which is the empty-file-after-power-loss case Orca fixed (#1158). Marley's endpoint file is truncated and rewritten in place, so a reader can see half of it. Recordings use `fs::write` (`crates/marley_browser/src/recorder.rs:373`). | A `durable` module in a Marley crate: a temp file in the same directory, fsync, rename, fsync of the directory, plus remove-if-still-ours. Use it for the endpoint file, the bridge script, recordings, picks, annotations and the browser restore state. Prove the call order in an e2e-reachable way, or leave it to review since the unit-test gate retired. | S | None. It goes with #502. |
| 5 | **Terminal sessions that outlive Marley** | This is Orca's defining capability: quit, crash or update and the agents keep running; relaunch and the same PTY is back with its scrollback (`golden-quit-relaunch-session.spec.ts` asserts the same PTY id). Marley's plan already puts it in the rustal-harness runtime (prong 2, C1/C3). | The harness client, with Zed terminals in display-only mode fed by the harness (plan D10). Take from Orca's daemon: (1) version the protocol from day one and keep old versions attachable, since a session owner outlives the client; (2) a per-session emulator snapshot for reattach plus dirty-gated checkpoints for cold restore; (3) the endpoint ownership protocol (`src/main/daemon/AGENTS.md`); (4) `systemd-run --user --scope` for the session owner, as Marley already does for Chromium, because `nohup`, `setsid` and tmux do not leave a cgroup; (5) the `live` / `unverifiable` / `exited` vocabulary; (6) a golden quit-and-relaunch scenario. | L | Harness protocol v1 is paused upstream (`docs/marley/three-prong-plan.md`). Blocks must be rebuilt from a snapshot. |
| 6 | **Captured transcripts for any rule that reads an agent's screen** | #508 (approvals) and #509 (turn boundaries) must read Claude Code's screen where hooks do not reach. Orca's rule: such a rule is written against a raw transcript captured at a pinned size, with scripted keys, a meta sidecar (CLI version, size), same-length redaction, and a secret scan over every committed fixture (`docs/reference/agent-pty-transcript-capture.md`). It had five failed attempts at Antigravity readiness without one. | A `script/capture-transcript` recorder, fixtures under `script/e2e/fixtures/`, and a gate step that scans them; an e2e scenario replays a transcript with `cat` in a Marley terminal and shots the result. | S–M | Claude Code redraws constantly, so pin the fixture to a CLI version and re-capture when it changes. It feeds #508 and #509. |
| 7 | **One agent-status store, with provenance (queued, #508)** | Orca found three copies of the same row in main and four readers applying their own precedence, so a pane read differently on the desktop, the phone and in the CLI (`docs/reference/agent-status-store.md`). Marley's rail icons, `fleet_snapshot` and the coming inbox all read status. | `marley_fleet` holds the one store. The producers (the plugin's OSC 9/777 notifications and hooks, Zed Agent Panel thread state, later the harness) write into it. Precedence is decided at write time and recorded on the row. Readers keep presentation only (decay, unread, dismissals). Rows restored after a restart are marked unconfirmed until a live producer confirms them. | M | The Agent Panel's state and terminal hook state arrive through different paths and at different times. |
| 8 | **Wire rules for the MCP tools and the plugin** | Agents and the Claude Code plugin outlive a Marley upgrade, and after #502 the bridge written by one build will be read by another. Orca's rules: new optional fields are safe; renamed or repurposed ones are not; numbers and names are permanent; changing what the host publishes is a wire change (`docs/reference/remote-wire-compatibility.md`). | A short `docs/marley/mcp-compatibility.md`, a `version` field in the endpoint JSON and in `fleet_snapshot`, and a version check in `marley-mcp-bridge`. | S | None. |
| 9 | **Evidence discipline** | Orca's audits prove a bug fix by reversing the patch and showing the same test fail (A8). Every audit says what it does not establish. The style guide bans UI copy that overclaims, and has a rubric for reviewing a screen. | CONSTITUTION §7 and §19 text plus the pipeline templates. (a) A bug ticket's scenario is run once on the base commit, or on the installed build after #502, to show the bug, then on the fix. (b) Notes and ledger failure entries gain a "Scope and limits" line. (c) Borrow the rubric (top three fixes, friction, keyboard check) for reading shots. | S | Running the base build doubles the scenario time for bugs. |
| 10 | **A per-project resource view** | Orca's Resource Manager shows each worktree's CPU and memory across its PTY process trees, and kills one session or all (A5). Marley runs several agents per project with no view of what they cost. | A rail popover or footer backed by `marley_fleet`. `sysinfo` is already in the tree (`crates/zed/src/reliability.rs`). | M | Attributing process trees to projects, and deciding what "kill" means for an agent inside a terminal. |
| 11 | **Knowledge pushed at the moment a path is edited** | Orca's `AGENTS.md` says "before touching X, read Y" and keeps directory `AGENTS.md` files of "traps that already cost us". Marley's 9,177-line ledger is recalled only by grep (§18.3). | A PreToolUse hook with a trigger table (path glob → ledger codes) that prints the matching prevention rules on the first edit of a session. The hooks directory already exists. | S | Keeping the table current as the ledger grows. |
| 12 | **Typing and restore budgets for the block terminal** | Orca sets its terminal budgets from recorded runs (typing median 25 ms, worst key 300 ms, restore 1 s) and refuses to loosen a limit just above a new failure. | A measuring scenario, using gpui's frame telemetry or timestamps through `marley_mcp`, with budgets set from about ten recorded runs. | M | Timing on a shared dev box is noisy. |

### What the changelog teaches the queued tickets

| Ticket | What Orca shipped that the ticket should include |
|---|---|
| #502 installed build | Row 2 above. Orca's hourly build is skipped when main has not moved, and a fixed number of builds is kept. |
| #503 terminal URLs open a Browser tab | Orca watches PTY output for advertised URLs. Its carry is bounded per PTY: 4,096 characters for a bound PTY, 16,384 for each of at most 32 unbound ones (`docs/audits/pty-detector-retention/`). It also has: Orca's browser offered for SSH terminal links (#16503); Shift inverting the routing (#10991); URL-click and middle-click behavior as settings (#21438); a link popover with Copy (#13414). |
| #504 Browser tabs in the rail | Inactive pages are deferred across switches, and background tabs load when opened (#19326, #19633). Closing a tab returns to the previously active tab (#16306). |
| #505 pick, fix, check | Annotations are editable in the tray, and delivered ones are cleared (#17511, #22060). "Draw" marks up a frozen image of the page (#6335). Annotations can go to an existing agent session or start a new one (#7347). |
| #506 recording to a Playwright test | Nothing in three months of notes. Orca's own e2e runs on its maker's Playwright wrapper (`@stablyai/playwright-test`). |
| #507 browser context per project | Cookie import replaces only the imported domains, reports cookies it could not decrypt, and keeps `__Host-` cookies host-only (#12166, #14683, #15030, #15375). Chromium SameSite values must be decoded; before #20076, 135 cookies per profile were silently dropped. Google sign-in works only directly in the app: FedCM is off and Google cookies are excluded from imports (#14023 and follow-ups). |
| #508 approvals inbox | Map each agent's ask tool to one "needs you" state with one glyph (#7852, #9457, #9861, #9996). A proposed plan renders as a plan, not a generic approval (#21090). Agents waiting for input are highlighted (#13523). A worker blocked on a human prompt is reported upward (#15261). Rows 6 and 7 above. |
| #509 per-turn diffs | Structured chat shows a per-turn summary of changed files (#19229) and file edits as inline diff cards (#18765). The unit is the turn, not the session. |
| #510 worktree agents | Orca's related features: (1) checkouts prepared while the composer is open (#17290); (2) `.worktreeinclude` for gitignored files a worktree needs, and `worktree.sharedDirectories` for shared ones (#9791, #10459); (3) per-worktree **localhost labels**: a loopback URL with a port that a worktree's dev server prints is served through a proxy as `<worktree>.orca.localhost:<proxy-port>` (`src/main/localhost-worktree-label-proxy.ts`, #6424), so tabs and cookies tell worktrees apart; each server still needs its own port, so this complements the port offset; (4) workspace names surviving branch changes (#17448); (5) scratch worktrees kept out of the sidebar (#9535); (6) a worktree comment agents set from the CLI as a status line (`docs/site/content/docs/cli/worktree-checkpoints.mdx`). |
| #511 review, merge, remove the worktree | Removal is blocked when the archive hook fails (#20153). Delete names the live PTYs and offers force (#12153). A home-directory guard asks the execution host whose home a path is (`src/main/worktree-removal-home-guard.ts`, #18275). After a bulk delete you can review the branches git kept (#13693). Diffs can collapse unchanged regions (#11955), and comments can span line ranges (#20959). |

## 4. Skip

- **Five release channels in separate repos, SignPath and notarization.** They serve thousands of
  users on three OSes; one installed build plus a rollback copy covers one user on one box.
- **The reliability-gate registry.** It is 20,280 lines of JSONC, and all 126 gates are still
  `experimental`, so none has ever decided a merge. The ledger and `gates.sh` already cover the
  need.
- **A 300-line `max-lines` budget.** It split Orca's runtime into about 90 mixin files and put
  `@ts-nocheck` on 171 of them.
- **Changed-lines gates and baselines.** Orca needs them to add rules to 1.77M lines. Marley's "no
  baselines" is affordable on its small owned surface, and is stricter.
- **The product telemetry pipeline** (PostHog, schemas, burst caps). There is no user base to
  measure. The opposite question applies to Marley (Open questions).
- **Nudge campaigns and the in-app "what's new" card.** Marketing to a user base.
- **The cross-version wire harness at Orca's scale.** Marley has one client and one host. The
  written rules (row 8) are enough until a remote host exists.
- **The unit-test volume and the `vi.mock`-heavy suites.** Marley chose e2e-only proof on
  2026-09-23.
- **Windows, WSL, EDR, localization and the mobile OTA page.** Not Marley's platforms or scope.
- **A hosted pairing relay** (`cloud/`, 26 `cloud-*` workflows). A service to run, not an editor
  feature.

## 5. Open questions for Chad

1. **Telemetry.** Marley inherits Zed's `telemetry.metrics` and `telemetry.diagnostics`, both on by
   default, and the dev box's settings do not turn them off. Marley's `logs/telemetry.log`
   shows metric events being produced, and Zed's flush posts them to `api.zed.dev` with an empty
   checksum (`crates/client/src/telemetry.rs:637-657`). I did not confirm whether a request leaves
   the machine. Should Marley's defaults set both to `false` (a small Zed touchpoint)?
2. **The #502 profile.** Should the installed build share the `dev` profile behind an instance guard
   (row 2), or get a profile of its own, so a debug `marley` can run next to it?
3. **Machine checks.** May golden scenarios pass on a machine check through `marley_mcp`, with the
   shots read only on failure? That changes §7's "reading the shots is the test" for regression
   runs only.
4. **Terminal survival before the harness.** Should terminals survive a Marley restart before the
   harness lands, through a small Marley-owned session process, or wait for rustal-harness C1/C3?
