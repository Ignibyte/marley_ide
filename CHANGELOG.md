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

- **Autosuggestions** (#484, 2026-09-23). As you type a command at a bash or zsh prompt in
  Marley's terminal, the rest of the newest matching command from your history shows dimmed after
  the cursor, and → types it in, as Warp and fish do. Commands you ran in the terminal come first,
  then the shell's history file. Without a suggestion, → moves the cursor as before.
- **Rich input** (#481, 2026-09-23). While Claude Code or another CLI agent runs in a terminal,
  Ctrl-G, or the pencil in the agent bar, opens an editor above the bar for the agent's prompt:
  select with the mouse, undo, move by word, Shift-Enter for a new line. Enter sends the text to
  the agent as one paste and closes the editor; Escape closes it and keeps the draft for the
  next Ctrl-G. Without an agent, Ctrl-G reaches the program as it always did.
- **Voice input through Voxtype** (#480, 2026-09-23). Where Voxtype, the dictation daemon
  Omarchy ships, is installed, the agent bar has a microphone. Click it, or run
  `marley: toggle dictation`, to start or stop a dictation, and Voxtype types what you said into
  the terminal. The microphone turns red while Voxtype records and yellow while it transcribes,
  however the dictation started, Omarchy's own keys included.
- **Attach File** (#479, 2026-09-23). The agent bar has a `+`, Attach File, and the command
  palette has `marley: attach file`. Both open a file chooser, and the files you pick go into the
  terminal as their full paths, quoted where the shell needs it, the way dropping them on the
  terminal types them. Claude Code and the other agents read a file from its path.
- **Enable Claude Code notifications** (#482, 2026-09-23). While Claude Code runs in a terminal,
  the agent bar offers "Enable Claude Code notifications". It installs a small Marley plugin
  into your Claude Code with its own `claude plugin` commands. The plugin's hooks ask Claude Code
  to send Marley a notification when it wants your permission, waits for you, or finishes, and
  they stay silent in other terminals. New Claude Code sessions use it; in a running one, run
  `/reload-plugins`.
- **Desktop notifications from the terminal** (#478, 2026-09-23). A program that asks the
  terminal for a desktop notification, with the escapes iTerm2, Ghostty and rxvt read (OSC 9 and
  OSC 777), now gets one from Marley while you are not looking at that terminal: another pane
  has the focus, or Marley's window is in the background. Clicking it brings the terminal to the
  front, and its tab and its row in the left pane carry the same mark a bell gives them.
- **The agent bar** (#477, 2026-09-23). While Claude Code, Codex, Gemini CLI or OpenCode runs
  in a terminal, a bar shows under it with the agent's name at the left and, at the right, the
  folder it works in and that folder's git branch, as Warp shows them. The terminal gives up a
  row to it while the agent runs, and gets the row back when the agent exits.
- **The prompt at the bottom of the terminal** (#476, 2026-09-23). While a terminal's screen
  has room to spare, as in a new terminal or one you have just cleared, Marley draws its content
  against the bottom of the pane instead of the top. The prompt sits on the last row and each
  command's output pushes the rest up, the way Warp pins its input to the bottom. Scrolling back
  shows the history above it, and full-screen programs such as vim are drawn as before.
- **`just shot` can open a path** (#476, 2026-09-23). `OPEN=<path> just shot <name>` opens that
  path in the copy of your profile, as `marley <path>` does; a folder Marley has not seen gets a
  first terminal to capture.
- **Copy and rerun a block from the terminal** (#474, 2026-09-23). Point at a block and two
  small buttons show beside its pill: Copy puts the block's output on the clipboard, and Rerun,
  shown while the shell waits at its prompt, clears what you had typed on the line and runs the
  block's command again. A click on either button stays with it: it starts no selection, and a
  program that reads the mouse does not receive it. Rerun is offered only for a command your
  shell reported itself: each terminal gives the shell a secret that Marley's bash and zsh
  integrations add to their reports, so text a program prints to imitate one cannot be run.
- **Keys to move between blocks** (#473, 2026-09-23). In a terminal, Ctrl-Up (Cmd-Up on macOS)
  scrolls to the start of the block above the top of the view, and Ctrl-Down to the next one,
  or back to the live screen from the last. Each press moves one block, however fast you press.
  Zed's terminal bound neither key; a binding of your own still wins.
- **A `justfile` for the workflow** (#471, 2026-09-23). `just` lists the commands every change
  runs: the gate in each mode, building the debug `marley`, a crate's tests, clippy and
  formatting over named crates, and `just shot`, which captures Marley on a copy of your profile
  on a hidden workspace without touching your screen. Each cargo recipe first waits for any
  other cargo run on the machine to end. `script/gates.sh` stays the gate.
- **Blocks drawn in the terminal** (#470, 2026-09-23). Every command Marley's shell integration
  reports is now visible as a block in the terminal: a thin bar in the left margin beside its
  rows, green when it succeeded, red when it failed and blue while it runs; a small pill at the
  right end of its first row with a check, its exit code, or `running`; and a faint red or blue
  tint over a failed or running block. The terminal's text and rows are unchanged. Nothing is
  drawn while a full-screen program such as vim uses the alternate screen.
- **Shell integration for zsh** (#465, 2026-09-23). An interactive zsh that Marley starts in a
  local terminal now loads Marley's integration, as bash has since #463, so every command you
  type in zsh becomes a block with its text, exit code and output. zsh starts with `ZDOTDIR`
  pointing at a `.zshenv` Marley writes to its data directory. That file puts your own
  `ZDOTDIR` back, or leaves it unset if you had none, and runs your `.zshenv`, so zsh reads your
  `.zprofile`, `.zshrc` and `.zlogin` as it always did. If you have no zsh startup files at all,
  zsh's new-user menu no longer opens in Marley's terminals. fish follows (#466).
- **Shell integration for bash** (#463, 2026-09-23). An interactive bash that Marley starts in a
  local terminal now loads Marley's integration: it starts with `--rcfile` pointing at a script
  Marley writes to its data directory. The script sources your own `~/.bashrc` first, then
  reports each prompt and command to the terminal. Every command you type now becomes a block
  with its text, exit code and output. Nothing draws the blocks yet (T1). Tasks, remote
  terminals and shells you start with arguments of your own start as before, and zsh and fish
  follow (#465, #466).
- **Blocks in Zed's terminal** (#464, 2026-09-23). A terminal keeps each command that Marley's
  shell hooks report as a block: the command, whether it is running or finished, its exit code,
  the prompt it was typed at, and where its lines sit in the scrollback. A block's output is read
  from the terminal itself while those lines are still held. Nothing draws the blocks yet (T1),
  and shells start sending the hooks with the integration scripts (#463).
- **Shell hooks found in the terminal's output** (#462, 2026-09-23). The terminal library
  Marley carries now takes Marley's shell-hook frames out of a terminal's output before its
  parser would drop them, and reports each one with the exact line it fell on, also when one
  read carries a whole command, and as old lines leave the scrollback. Every other byte reaches
  the parser as before, another program's control strings included. Nothing on screen changes
  yet: Zed's terminal starts keeping blocks from these reports in #464. The scanner is a crate
  of its own, `marley_dcs`, shared with `marley_terminal`.
- **Zed's dylint lints on the Marley crates** (#448, 2026-09-23). gate:21 runs Zed's own lint
  library, `tooling/lints`, over the seven Marley crates with `cargo dylint`, on the nightly the
  library pins. Its lints catch gpui mistakes clippy cannot see: an entity updated or notified
  while a view renders, blocking IO where a synchronous context runs, an async block with no
  await, and string and map misuses. Each Marley crate root makes them errors under the dylint
  driver, so a hit fails the gate, and Zed's own crates keep them as warnings. The twelve hits
  it raised, all `SharedString`s built from string literals, now use
  `SharedString::new_static`. The receipt's fingerprint covers `tooling/lints`.
- **Next and Previous Project and Thread in the Marley layout** (#459, 2026-09-23). The
  command palette's Next Project, Previous Project, Next Thread and Previous Thread did nothing
  in the Marley layout. Now Next and Previous Project show the project after or before the one
  the rail highlights, and Next and Previous Thread open the terminal or thread row after or
  before it, with focus, as a click on that row does. They go round at the ends, pass over what
  a fold or the filter hides, and work with the rail closed.
- **A first terminal for a new project** (#455, 2026-09-23). A folder you open for the first
  time in the Marley layout starts with a terminal at its root, with focus. A project you have
  opened before comes back as you left it, with its saved terminals or none, and never an extra
  one.
- **A switcher over recent terminals and threads** (#454, 2026-09-23). In the rail or the Agent
  Panel, `ctrl-tab` (on macOS too, as in Zed) opens a switcher over the window's terminals and
  threads, the ones you last worked in first, with the one before the current selected. Keep
  `ctrl` held: each `tab` moves down the list and `shift-tab` moves up. Let go of `ctrl` to
  open the selection; Enter or a click opens an entry too, and Escape closes it. Before this,
  `ctrl-tab` in the Agent Panel did nothing in the Marley layout. The center panes keep Zed's
  tab switcher.
- **A filter for the rail** (#457, 2026-09-23). Press `ctrl-f` (`cmd-f` on macOS) in the rail,
  or click the field under its header, and type. The rail keeps only the projects, terminals
  and threads whose name or title contains what you typed, ignoring the case of ASCII letters,
  and highlights the matching characters. A project whose name matches keeps everything under
  it, folded or not. As you type, the first match is highlighted: up and down move from it and
  Enter opens it. Escape clears the filter, and a second Escape takes you back to the rows. The
  filter stays until you clear it, and "No matches" says when nothing matched.
- **The rail from the keyboard, and project reorder** (#453, 2026-09-23). Focus the rail with
  `ctrl-alt-;` (`cmd-alt-;` on macOS) and walk it: up and down move the highlight, Home and End
  jump to the first and last row, Enter opens the highlighted row as a click does, left folds a
  project or climbs from a row to its project, and right unfolds it. These are Zed's own list
  keys, so your bindings for them apply in the rail too. The highlight goes back to the row the
  window shows when focus leaves the rail. A project header's right-click menu moves the
  project up or down.
- **Rename and close terminals from the rail** (#452, 2026-09-23). Right-click a terminal row
  for Rename and Close, or double-click it to rename; a close button shows on the row under
  the pointer. Rename brings the terminal up and edits its name in its tab, as the tab's own
  Rename does, and the name is kept across restarts. Close closes the tab as Zed does, asking
  first while a task runs in it. A terminal you renamed keeps your name on its row even while
  an agent CLI runs in it.
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

- **Every change is proven by an e2e visualization test** (#483, 2026-09-23). A ticket no
  longer writes unit tests or gpui driven tests. Its proof is a scenario in `script/e2e/`
  that `script/e2e.sh` (`just e2e <scenario>`) runs against the real debug Marley on a hidden
  workspace. The scenario builds its fixtures, presses and types keys in Marley's window only,
  and shoots each step, and the Test phase reads every shot. `just shot` is a one-shot scenario
  now. The tests already in the tree stay and keep building, but the gate no longer runs them.
- **The rail looks like Warp's tab list** (#468, 2026-09-23). Terminal and thread rows are
  taller, padded cards with their icon in a 28px circle: `>_` for a shell, the agent's own mark
  for an agent. The title sits over a muted second line. The selected row is a card with a
  border, and selecting another row moves nothing. A thread row's second line names its agent
  and what it is doing ("Zed Agent · working"), as an agent CLI's row does. Project names read
  as muted section labels, a line runs between projects, and the list and the filter have more
  room.
- **Marley carries its own copy of Zed's `alacritty_terminal`** (#461, 2026-09-23). The
  terminal library Zed builds on now comes from `vendor/alacritty_terminal`, an unchanged copy
  of the version Zed pins, so the block terminal can change its event loop in this repository
  (#462). Nothing behaves differently yet.
- **Marley starts in the Marley layout** (#460, 2026-09-23). A fresh install, and anyone who
  never chose a layout, now opens with the rail of projects and their terminals on the left
  and the terminals in the center. Before, the fork started in Zed's layout until you ran
  `marley: use marley layout`. `marley: use zed layout` still switches back, and a `zed`
  choice in your settings stays.
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

- **The test gates** (#483, 2026-09-23). gate:3 (the test suites), gate:4 (the 100% line
  coverage floor), gate:6 (miri) and gate:19 (empty suites) left `script/gates.sh`, and so did
  `--full`, which ran the first two over every Marley crate. `script/mutation.sh`, the
  end-of-sprint mutation run, went with them, as did `script/live-shot.sh`, which
  `script/e2e.sh` replaces.
- **Mutation testing from the per-change gate** (2026-09-22). gate:5 and its MSI floor left
  `script/gates.sh`, because mutation was too slow to run on every change. It now runs once at
  the end of a sprint through `script/mutation.sh`, which keeps #443's copy-mode isolation (a
  target directory per worker, `-p` for every crate, no masks). The mutation output and scratch
  directories are gone; gate:12 still bans `mutants::skip` masks.

### Fixed

- **Typing on a long prompt in a new terminal** (#485, 2026-09-23). With a two-line prompt whose
  second line is wider than 100 columns, such as starship's on a deep path, the first command
  typed in a new terminal lost every character after the first on screen, though the shell got
  them all. The shell started at Zed's small starting size and was resized after its first
  prompt, which readline does not recover from. A terminal now opens at the size the last
  terminal was drawn at. The first terminals of a launch still open small (TICKET-486).
- **The coverage gate reads only what its run built** (#469, 2026-09-23). gate:4's coverage
  tool read every test executable left in its target directory, and one that an earlier run
  built from older source reported 45 missed lines on doc comments in #465. The step now
  removes the test executables before it runs; libraries stay built, so it relinks only the
  tests it runs.
- **A shell's title no longer shows Marley's rcfile** (#467, 2026-09-23). Since #463 every
  bash tab and every terminal row in the rail read `bash --rcfile …/marley.bash`, because Zed
  titles a shell with its arguments and Marley adds that one to load its integration. The title
  now leaves out the arguments Marley added, so a shell reads `marley_ide — bash` again. A
  program you run in the terminal is titled with its arguments as before.
- **The rail follows a project's folders** (#458, 2026-09-23). A project whose last folder you
  removed kept its row in the rail until something else in the window changed. Now the row goes
  at once. Adding, removing or reordering a project's folders updates its row straight away,
  down to each terminal's path, which reads against the project's first folder.
- **The docks across a layout round trip** (#456, 2026-09-23). A switch to the Marley layout and
  back could leave the right dock closed, with the panel it showed lost: the Agent Panel took
  that dock over on the way in and closed it on the way out. Now the dock gets back the panel it
  showed, open or closed as it was. If you showed another panel there, or closed the dock,
  during the trip, your choice stands.
- **Zed's Panel Layout presets in the Marley layout** (#451, 2026-09-23). Choosing Classic or
  Agentic from the title bar's Panel Layout menu, or from the command palette, rewrote the
  docks the Marley layout sets, and Agentic moved the Agent Panel to the left for good. In the
  Marley layout both now change nothing and say that the presets belong to Zed's layout, with a
  button that switches to it. In the Zed layout they work as before.
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
