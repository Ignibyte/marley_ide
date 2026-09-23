# Changelog

All notable changes to **Marley** (the Zed fork) are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Marley is pre-1.0 and tracked by
the slices of `docs/marley/three-prong-plan.md`, so versions are placeholders until the first
release. A `CHANGELOG.md` entry is **mandatory** for every change that touches Rust source;
`enforce-changelog.sh` blocks a commit without one (CONSTITUTION §21).

The gpui-era app that preceded the fork kept its own changelog through 2026-09-18; it is
archived verbatim as [`docs/marley/history/CHANGELOG-gpui-era.md`](docs/marley/history/CHANGELOG-gpui-era.md).

## [Unreleased]

### Added

- **The rail remembers being closed, and its width** (#442, 2026-09-23). A rail you close stays
  closed when Marley restarts, where before every window rebuilt it open, and one left open
  stays open. The rail's width is saved with the window, and one width holds in both layouts:
  a width set on the rail carries to Zed's sidebar after a switch to the Zed layout, and back.
- **New Agent from the keyboard** (#450, 2026-09-23). `ctrl-alt-n` (`cmd-alt-n` on macOS)
  opens a New Agent picker in either layout: the Zed Agent and each configured external agent,
  then each agent CLI installed on the `PATH`, every entry marked "Thread" or "Terminal".
  Typing filters the list. A Zed agent starts a new thread, focused, in the project's Agent
  Panel; a CLI starts in a new center terminal once its shell is ready, as the rail's `+` menu
  starts one. The key comes from Marley's own keymap, which Zed's keymap loading binds after
  its defaults, so it holds across keymap reloads and a binding of your own on the same keys
  wins. With AI disabled the key opens nothing.
- **Terminal keys in the Marley layout** (#449, 2026-09-23). In the Marley layout
  `` ctrl-` `` switches between the code and the project's terminals: from an editor it
  focuses the terminal used last, or opens one, and from a terminal it goes back to the editor
  used last. `ctrl-j` does the same while it would otherwise show the Terminal Panel, and
  `ctrl-~` opens a new center terminal, so none of Zed's terminal keys opens the bottom panel
  any more. The command palette's Terminal Panel toggles and a user's own bindings for them
  route the same way. In the Zed layout every key keeps Zed's behavior.
- **Terminal routing in the Marley layout** (#441, 2026-09-22). In the Marley layout nothing
  opens the bottom Terminal Panel. Tasks run in center terminals, whether they start from the
  task modal, a runnable or a code lens. A rerun replaces the task's terminal as Zed's rules
  say, and a task that last ran in the panel before a switch reruns in the center. New Terminal
  and every Open in Terminal menu open a center terminal, the second in the folder it names;
  one that cannot open says why in a prompt. In the Zed layout each goes where upstream sends
  it, and a layout switch changes the routing on the next call with nothing to restart.
- **Agent CLIs in rail terminals** (#440, 2026-09-22). The rail recognizes Claude Code, Codex,
  Gemini CLI and OpenCode running in any terminal, however they were started. Such a row shows
  the agent's icon and the title the CLI sets, and its second line reads the agent and whether
  it is working or waiting: waiting once its output has been quiet for two seconds, or when it
  rings the bell. When the agent exits, the row is a plain terminal row again. A project's `+`
  menu lists the installed agent CLIs under an "Agent CLIs" header. One click opens a center
  terminal in the project and starts the CLI there once the shell is ready, writing nothing
  but the program's name.
- **Zed agent threads in the rail** (#439, 2026-09-22). In the Marley layout each project lists
  its Zed agent threads under its terminals, newest first. A thread row shows its title, its
  agent's icon and what it is doing: running, waiting for a confirmation, failed, or done. A
  click shows the project and opens the thread, focused, in the Agent Panel on the right. The
  project's `+` menu gains New Agent Thread, which lists the Zed Agent and every configured
  external agent by name and starts one in that project. A run that ends while its thread is
  not on screen lights the row's dot until the thread is shown, and a thread waiting for a
  confirmation also lights the sidebar toggle's dot and a folded project's header. The
  selected row follows focus: the Agent Panel's thread while the panel holds focus, otherwise
  the active terminal. Zed's OS notifications for threads still fire, since the rail does not
  yet list every kind of thread Zed would silence.
- **Rustal's quality gates on the Marley crates** (#447, 2026-09-22). Each of the seven
  `crates/marley_*` manifests carries rustal's lint table, and all 679 hits it raised are
  fixed. The table sets clippy's pedantic, nursery and cargo groups to warn and denies
  `missing_docs`, `missing_debug_implementations`, `unsafe_code`, `unwrap_used`,
  `expect_used` and the doc-section lints, plus `let_underscore_must_use`, which enforces Zed's
  rule against `let _ =` on a fallible call. Test code may still unwrap and expect
  (`clippy.toml`). Four new gates:
  - gate:17 runs cargo-sort and taplo on the Marley manifests;
  - gate:18 runs typos, as Zed's CI does;
  - gate:19 fails a Marley test suite that holds no tests;
  - gate:20 runs semgrep 1.156.0 with `.semgrep.yml`'s two rules: `std::process::exit` in a
    library, and a `Command::new` whose program is not a string literal.

  gate:13 accepts a `// SAFETY:` comment on the line above the `unsafe` and catches a bare
  `transmute(`. gate:14 fails on any `warning:` the doc build prints and on a TODO marker in
  Marley Rust source. The Marley crates' doctests run.
- **The Marley layout and its first rail** (#438, 2026-09-22). A `marley.layout` setting
  (`zed`, the default, or `marley`), written by the actions `marley: use marley layout` and
  `marley: use zed layout`. In the Marley layout each window's sidebar is the rail: every
  project, the terminals in its center panes under it, a `+` menu with New Terminal (started
  in the project's own directory), one selected row that follows what the window shows, a bell
  dot on a terminal's row and on a folded project's header, and an Add Project button. The
  layout also hides the bottom Terminal Panel's button and docks the Agent Panel on the right,
  as defaults a user value still overrides. Changing the setting swaps the sidebar in every
  open window without a restart, and switching back hands each window its own Zed sidebar,
  open or closed as it was, with its width and view. With AI off no sidebar is drawn, as in
  Zed. Two new crates: `marley_rail`, the row model (pure, gpui-free), and `marley_workbench`,
  the setting, the switch and the rail.
- **The workflow process, ported from the gpui-era repo** (2026-09-18). `CONSTITUTION.md`
  rewritten for the fork; the pipeline commands (`/work`, `/spec`, `/pipeline:*`, `/commit`)
  and the enforcement hooks under `.claude/`; `script/gates.sh` scoped to the Marley crates
  plus the crates a change touches, with the receipt fingerprint computed by one
  `git hash-object` pass; `.cargo/audit.toml` and `deny.toml` tuned to upstream Zed's
  dependency tree (the fork-point advisories and git sources listed with their reasons); the
  planning tree (`docs/planning/`: templates, tickets, the knowledge ledgers, the completed
  pipeline archive, intake and design notes) and the design record (`docs/marley_architecture/`,
  `docs/specs/`, `docs/warp_architecture/`, `docs/zed_architecture/`, `docs/decisions/`,
  `docs/tickets/`) carried over intact. Retired: the React parity hook and the macOS AX
  harness gate; the brand-scrub half of the docs gate.
- **The three-prong plan** (`docs/marley/three-prong-plan.md`, 2026-09-18): the block
  terminal on Zed's terminal, the control plane over rustal-brain, the rustal-harness runtime
  and Rusty, and the browser service.
- **The Zed touchpoint ledger, enforced** (#436, 2026-09-22). Every change outside the
  Marley-owned paths needs its row in `docs/marley/zed-touchpoints.md`, checked in three
  places. gate:16 in `script/gates.sh` fails on a changed path with no row, a row whose path no
  longer differs from the upstream fork point, a duplicate row or a row for an owned path, and
  an owned set that would claim an upstream file. `enforce-zed-ledger.sh` blocks a Write or
  Edit to a Zed path until its row exists, judging each file by the checkout it lives in.
  `enforce-commit-gate.sh` runs the same check at every commit, Rust or not. CONSTITUTION §0,
  §14 and §21 name the ledger; `upstream_base` moved into `lib-hook-helpers.sh` so the gate and
  the commit hook share it.
- **Five Marley crates ported as workspace members** (2026-09-18): `marley_terminal` (the
  block model, DCS hook codec and PTY session; `SessionId` folded in from the old
  `marley_core`), `marley_fleet`, `marley_mcp`, `marley_agent`, `marley_remote`. Adapted to
  the fork: rustix 1.x signal constants, a `cfg(unix)` gate around the PTY shim and the
  discovery file's modes, the cargo-mutants attributes stripped, the serial-test dependency
  replaced by a process-wide lock in the real-PTY tests. Not yet wired into the app.

### Changed

- **`marley_agent` is the fork's agent-CLI model** (#440). It knows four CLIs and judges an
  agent's status from a quiet spell measured on gpui's clock. The gpui-era tick counters,
  `AgentRun` and the idle and exited states are gone, since nothing used them.
- **`script/gates.sh` takes an explicit mode** (#447, 2026-09-22): `--full`, `--diff` or
  `--fast`. A run with no mode, or an unknown one, is a usage error (exit 2) and runs no gate.
  A `--full` or `--diff` run removes the earlier receipt when it starts, reports the heavy
  gates BLOCKED after a static red, and writes a receipt only when the gated files at the end
  are the ones it started on. The fingerprint also covers `rustfmt.toml`, `.config/typos.toml`,
  `.semgrep.yml` and every file under `crates/marley_*`.
- **The workflow is four phases** (2026-09-22, Chad's call): Plan → Code → Test → Complete,
  run as `/pipeline:plan`, `/pipeline:code`, `/pipeline:test` and `/pipeline:complete`. Plan
  takes in `/work`'s pre-flight and recall and the old design phase; Code ends with a review
  of its own diff in place of the inspect phase; Test is the old validate; Complete writes the
  docs, captures the knowledge, closes the ticket, archives the pipeline and commits, which
  `/commit` used to do. The phase hooks, the templates, CONSTITUTION §3, §7, §15, §18 and §21
  and the queued specs follow the new phases. The task hook no longer blocks a Stop in a
  harness that has no `TaskCreate`; it still blocks one that leaves a created task open.
- **The fork is Marley** (#437, 2026-09-22). `paths::APP_NAME` is `"Marley"` and the app
  binary is `marley`, so the fork keeps its settings, database, logs and cache in
  `~/.config/marley`, `~/.local/share/marley` and `~/.cache/marley` and never touches a stock
  Zed install's. Chad's Zed settings were copied into Marley's config once. A `paths` test
  keeps the release channel at `dev`: on any other channel the fork would share stock Zed's
  keyring items, updater and app id, which TICKET-445 will give Marley its own.

### Removed

- **Mutation testing from the per-change gate** (2026-09-22). gate:5 and its MSI floor left
  `script/gates.sh`, because mutation was too slow to run on every change. It now runs once at
  the end of a sprint through `script/mutation.sh`, which keeps #443's copy-mode isolation (a
  target directory per worker, `-p` for every crate, no masks). The mutation output and scratch
  directories are gone; gate:12 still bans `mutants::skip` masks.

### Fixed

- **Errors the ported Marley crates dropped** (#444, folded into #447). The MCP transport now
  logs a failed connection thread, a connection's IO error, and a focus effect the app can no
  longer take. The terminal logs a shell hook that arrives before `InitShell`, and any failure
  of the reap signal except ESRCH. The test seed returns its hook errors instead of dropping
  them.
- **`normalize_path` on macOS** (#436). Its worktree strip used `\+`, a GNU sed extension that
  BSD sed reads as a literal `+`, so the phase gate misread paths inside
  `.claude/worktrees/<name>/` on macOS. It uses the POSIX `\{1,\}`.
- **The quality gate over the ported tree** (#443, 2026-09-22). No `script/gates.sh --diff`
  run could have gone green in the fork. Its mutation step passed `--jobs` beside `--in-place`,
  which cargo-mutants rejects as a usage error. Without `-p`, it would also have mutated only
  Zed's `default-members` and passed without testing a Marley mutant. DIFF now names every
  touched package and fails closed when none resolves. FULL gives each mutation copy its own
  target directory: the two workers had been overwriting each other's test binaries. Both
  modes pass `--no-config` and clear the variables that could move or share the output, and
  gate:12 bans `mutants::skip` in any form. The receipt fingerprint covers `.cargo/config.toml`
  and `.cargo/mutants.toml`. Three hooks no longer race SIGPIPE under `pipefail`; the race had
  let the commit gate allow a commit without a receipt.
- **The MCP transport's event stream** (#443). It reads the snapshot version before writing
  the response head, so a change that lands in between is pushed instead of lost. The
  transport now has 19 loopback tests over a real socket, and the PTY shim has six real-PTY
  tests: the program and its arguments, the working directory, the environment, the window
  size, the SIGKILL escalation, and descriptor cleanup.
- **The gpui-era pipeline archive** (#443). 218 specs that predate the §20 reference rule and
  70 that predate the prior-art sweep now say so, which lets the reference hook accept them.
