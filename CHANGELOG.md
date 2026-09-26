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

- **Browser tools act in the agent's own project** (#574, 2026-09-26). A browser call that names
  no tab now acts on the Browser tab the user focused last in the agent's project, the project of
  the terminal it runs in or, for Zed's own agents, of the folder it runs in, rather than on the
  tab the user focused last anywhere. When its project has no tab, `browser_navigate` opens one
  there, beside the agent's terminal, and the other tools refuse and say so, so an agent in one
  project no longer drives another project's page. `browser_tabs` names each tab's project and
  marks `default` the tab a call from this agent would act on. An agent outside every project
  keeps the old behavior.
- **Marley's tools know which terminal is calling** (#520, 2026-09-26). Each local terminal now
  starts with `MARLEY_TERMINAL_ID`, an id of its own (a split gets another), and
  `MARLEY_PROJECT`, its project's folder; a task, a remote terminal and a Marley started inside
  a Marley terminal pass on neither. The Claude Code plugin's bridge sends them with each call,
  so `terminal_list` marks the agent's own terminal `self`, and `terminal_blocks` and
  `terminal_read` read that terminal when no `terminal` is named: an agent no longer guesses
  which terminal it runs in. Browser tools that act in the caller's project are #574, and the id
  surviving a restart is #575.
- **No more agents lost to a stray close** (#550, 2026-09-26). Closing a terminal, a window or
  Marley while an agent in it is working now asks first and names each agent, as in "Quit
  Marley? 1 agent is working: marley_ide · Claude Code · working", with Quit (or Close), Show
  and Cancel. An idle agent closes as before. A working agent's terminal closed from its tab
  keeps running for a minute: the toast's Undo, or Ctrl-Shift-T, puts it back with its
  scrollback. `marley.ask_before_ending_a_working_agent` turns the question off and
  `marley.undo_close_seconds` sets the minute (0 ends the terminal at once), both on the Marley
  settings page.
- **Claude Code's events on your phone** (#535, 2026-09-26). With `marley.push` set to an ntfy
  server on this machine and a topic (the Marley settings page's new Push section), Marley
  pushes one line when Claude Code in a terminal you are not looking at needs input, finishes or
  fails: `marley_ide: Claude needs input`. The line carries nothing the agent wrote. The ntfy app
  on your phone shows it; published to the phone over Tailscale, nothing goes through anyone
  else's server but, for iOS, ntfy.sh's relay of a message id. A token, if your server wants one,
  comes from a file only you can read. A burst from one project makes one push, a server off
  this machine is refused, and a server that stops answering shows one toast until it answers.
- **Claude Code's sessions for agents, the plugin update, and quiet rows** (#547, 2026-09-26).
  An older install of Marley's plugin for Claude Code now gets "Update Marley's plugin" in the
  agent bar, which runs Claude Code's own `plugin marketplace update` and `plugin update`
  commands; restart a running Claude Code to pick it up. Marley's MCP tool `fleet_snapshot` (and
  the `fleet://snapshot` resource) lists every Claude Code session in Marley's terminals, with
  what each is doing, so an agent or the harness can see the others; a session reads `done` once
  its Claude Code leaves the terminal, and goes with the terminal. A working row that has had no
  event for 30 minutes (Escape fires no hook) reads `no update in N m` instead of `working`; the
  minutes are `marley.no_update_after_minutes`, on the Marley settings page, and 0 turns it off.
  `MARLEY_CLAUDE` names the `claude` Marley runs for the plugin, else the PATH's.
- **What Claude Code is doing, in the rail** (#519, 2026-09-26). Marley's plugin for Claude Code
  (now 1.2.0) reports each of its hook events to the terminal it runs in, and the agent's row in
  the rail follows them: `working` with your prompt and the tool it runs (`Bash: ls -la`),
  `waiting` with the permission it asks for, until that tool finishes, `idle` with its last
  message, or `failed` with the error, and a count of the subagents running. A prompt Claude
  Code or a harness sends on your behalf (a task notification, a system reminder, the
  continuation after a compaction) keeps your own prompt on the row. The events never ring the
  terminal's bell or post a desktop notification, and a terminal whose Claude Code sends none
  keeps the old reading from its output. An installed plugin picks the events up once it is
  updated to 1.2.0 (`claude plugin update marley@marley`); the agent bar offering that update is
  #547.
- **Marley's own regression suite** (#517, 2026-09-26). `just regress` runs a golden set of
  Marley's e2e scenarios (the MCP server and terminal tools, blocks and suggestions, rich input,
  the Browser tab driven by an agent and from the rail, the settings page, secret redaction, one
  Marley per data directory), each in a hidden sway of its own and each checking itself through
  Marley's own tools, and prints a PASS or FAIL line for each. `just install` now runs the set
  against the build it is about to install and installs nothing when a scenario fails, so the
  Marley in your menu keeps working; `--skip-regress` installs without it.
- **Secrets hidden from what agents read** (#516, 2026-09-25). What Marley's tools hand an agent
  from a terminal (each command and its output) and from a page's console now has its secrets
  replaced by `[redacted: <kind>]`: private keys, values set on names such as `API_TOKEN` or
  `DATABASE_PASSWORD`, bearer tokens, passwords and tokens in URLs, JWTs, and the key shapes of
  AWS, GitHub, Slack, Stripe, Google, OpenAI and Anthropic. Each answer says how many it hid. Add
  your own regular expressions as `marley.redaction_patterns`; one that is not a regular
  expression is named in a notification. It is on by default, and the Marley settings page's new
  Agents section turns it off. Your terminal still shows everything as printed.
- **A Marley page in the Settings window** (#515, 2026-09-25). `marley: open settings` opens
  Zed's Settings window on a new Marley page, first in its list. It holds Marley's own settings:
  the layout (Marley's rail or Zed's own, as a dropdown) and, under Privacy, the telemetry
  toggles, both off by default. New Marley settings will appear there as they arrive.
- **Marley in the app menu** (#502, 2026-09-25). `just install` builds Marley in the release
  profile and installs it under `~/.local`, with a desktop entry, so Marley starts from the menu
  like any other app instead of from a debug build in the shared target directory, which a
  `cargo clean` or a broken build used to take away. Run it again after a pull; a Marley that is
  running keeps its old build until you restart it. When Marley started from the menu writes to
  stderr, which is where a panic goes on this build's channel, the launcher keeps it in
  `~/.local/share/marley/logs/stderr.log`. The installed and the debug builds share your
  settings and sessions, so run one of them at a time.
- **Zed's own agents drive the browser too** (#501, 2026-09-25). Marley now offers its MCP
  server to the agents of the Agent Panel: the Zed Agent (in its default Write profile) gets
  Marley's tools, the terminals and the Browser tabs among them, and every external agent you
  start there, Claude Agent, Codex and the rest, is handed Marley's server with its session. So
  any of them can open a page in a Browser tab and click, type and scroll while you watch. It is
  the context server `marley`, running Marley's small bridge; no password or token goes into your
  settings, and `"context_servers": {"marley": {"enabled": false}}` turns it off.
- **A Browser tab from the rail** (#500, 2026-09-25). A project's + in the rail now lists New
  Browser Tab after New Terminal: it brings the project to the front and opens a blank page in
  a new Browser tab there, with the cursor in the address bar, starting Marley's Chromium when it
  must. The chip under a Claude Code terminal that installs Marley's plugin now reads "Connect
  Claude Code to Marley", and its tooltip says what it brings: notifications, and Marley's tools
  for its terminals and Browser tabs, so the agent can open a page and drive it while you watch.
- **The Browser tab's flight recorder** (#499, 2026-09-25). While a Browser tab shows a page,
  Marley keeps that page's last minute: your clicks and where they landed, keys by name, typing
  as a count of characters (never the characters), the agent's actions, console messages,
  requests with secret-looking URL values hidden, navigations, the page's accessibility
  snapshot after each load, and two frames a second at most. The red dot in the toolbar, or
  `marley: record this`, saves that minute as a recording in Marley's data folder, never in a
  project, and a toast names it. Agents list recordings with `browser_recordings` and read one,
  with any of its frames as an image, with `browser_recording`, so "it broke just now" comes
  with what happened.
- **Annotations over the page** (#498, 2026-09-25). The Browser tab's pencil button, or
  `marley: annotate`, turns on annotate mode: drag a box around anything on the page, type a
  note, and Enter keeps it (Escape drops it). Boxes are drawn by Marley over the page, never
  put into it, and stay on what they mark as the page scrolls. Click a note and press Delete
  to remove its box; leaving the page for another clears them. Agents draw boxes too:
  `browser_annotate` puts one of the agent's, blue with a sparkle, around an element or over an
  area, and `browser_annotations` lists every box with its place on the page, its note and who
  drew it. A full `browser_snapshot` now gives headings and other named elements refs, so an
  agent can point at them.
- **A pick's code, one click away** (#497, 2026-09-25). A pick in the Browser tab now shows
  where the element's listener was written: Marley reads the script's source map, from the page
  or from the script itself, and finds that source in your project, so the tray reads
  `click src/app.ts:2` and a click on it opens `src/app.ts` at line 2 in the editor. A listener
  whose script has no map, or whose source is not in the project, shows its script's name and
  line instead, and opens nothing. The tooltip lists every listener. `browser_pick` gives agents
  each listener's original source, file and line too, and every line and column the pick tools
  give now counts from 1, as an editor does.
- **Pick an element for the agent** (#496, 2026-09-25). The Browser tab's crosshair button, or
  Ctrl+Shift+C, turns on pick mode: Chromium's own inspect highlight and its tooltip follow the
  pointer, and a click picks the control under it (a click on a button's label picks the
  button) without reaching the page; Escape leaves pick mode. Each pick waits in a tray under
  the toolbar with a caption field; Enter or Send types a line such as `[browser pick 1: button
  “Save changes” on localhost:3000/card; browser_pick id 1] Make this green` into the terminal
  you used last and takes you there. Agents read picks through Marley's MCP server:
  `browser_picks` lists them, and `browser_pick` gives one's locators (test id, id, text, CSS
  path, each marked when it finds the element alone), its role and name, the listeners on it and
  its ancestors with their script, line and column, what would block a click on it (something
  on top of it, `pointer-events`, visibility, `disabled`), its box and a crop of the page around
  it. A pick never records what was typed into a field. Escape now reaches a page in a Browser
  tab; before, Zed took it.
- **Select lists in the Browser tab** (#495, 2026-09-25). A page's drop-down list, which
  headless Chromium opens where no one can see it, now opens as a Marley list right under it,
  in the page or in a frame from another site: the current option checked, each group's name
  over its options, a disabled option greyed. Click an option, or use the arrows and Enter;
  Escape or a click elsewhere closes the list and keeps the value. Alt+Down, F4 or Space on a
  focused list opens it too, and the arrows on a closed one change its value as in Chromium. A
  choice reaches the page's own scripts as an input and a change. A list an agent clicks stays
  shut, so an agent never takes your focus that way.
- **Browser tabs come back after a relaunch** (#494, 2026-09-25). Browser tabs are saved with
  their workspace. When Marley starts again while its Chromium still runs, each tab returns on
  its own page, with what you typed into the page and where you scrolled; when the Chromium has
  stopped since, each tab opens its saved address again. A returning tab shows its old title
  and address until its page is back. Marley's Chromium now opens no page of its own at start,
  so a start adds no stray tab. A page opened on request, such as a new tab or an agent's, now
  keeps drawing after its tab is resized when it holds a frame from another site; before, it
  could stay on its old frame.
- **A Browser tab for each page** (#493, 2026-09-25). Every page of Marley's Chromium has a
  Browser tab of its own. A link that opens a new window, or a page's `window.open`, opens a tab
  beside the page that opened it, with the focus, as in a browser. Ctrl+T in a Browser tab, or
  `marley: new browser tab`, opens a blank page in a new tab with the focus in its address bar.
  Closing a tab closes its page, and a page that closes, from a script or another DevTools
  client, closes its tab. A page an agent opens gets a tab that leaves your focus where it is:
  behind the tab you are in, or, when no Browser tab is open and you are working in another
  pane such as the agent's terminal, in a new pane beside it; either way it is laid out at the
  size of the tabs around it. The agent tools take the tab to act on, by the id the new
  `browser_tabs` lists, and otherwise act on the tab you used last; `browser_navigate` with
  `new_tab` opens its page in a new tab. `marley: open browser` shows the tab you used last.
- **Agents see and drive the Browser tab** (#492, 2026-09-25). Marley's MCP server gains ten
  browser tools. An agent can look at the page the user sees (its URL, title, scroll, focused
  element and selection, and the frame as an image), read its accessibility tree as a list of
  the elements it can act on, each with a ref (fields in cross-site iframes included), and read
  the page's recent console messages and requests, with secret-looking values in URLs hidden and
  no headers or bodies. It can go to an http or https page, go back, click, type, press keys and
  scroll, with the same events the user's mouse and keys send. An agent's first action opens the
  Browser tab if none is open, without taking the focus, and an Agent chip in the toolbar says
  what the agent did. No tool runs script the agent supplies.
- **Marley's tools for agents, over MCP** (#491, 2026-09-25). While Marley runs, it serves MCP on
  127.0.0.1, behind a bearer that changes at every start and sits in a file only the user can
  read. The first tools read Marley's terminals: `terminal_list` names each terminal with its
  project, working directory and running command; `terminal_blocks` lists the commands run in
  one, with their exit codes, working directories, start times and durations; `terminal_read`
  gives one command's output. Marley's plugin for Claude Code, now 1.1.0, carries a bridge to
  the server, so Claude Code in any terminal on the machine finds the tools while Marley runs,
  and an empty server rather than an error when it does not.
- **Browsing in the Browser tab** (#490, 2026-09-25). A toolbar runs across the top of the
  Browser tab: back and forward, reload (a stop button while a page loads, with a thin bar under
  the toolbar), and an address bar that takes a URL, a host such as `localhost:3000` (loopback
  hosts load over http, others over https), or anything else as a DuckDuckGo search. Ctrl+L puts
  the focus in the address bar with its text selected; Escape gives the page its URL and the
  focus back; Alt+Left and Alt+Right go back and forward; Ctrl+R and F5 reload. A page's alert,
  confirm or prompt opens as a card over the page, naming the site that asks, since headless
  Chromium draws none: Enter answers OK and Escape Cancel. The address bar follows every
  navigation, a link's or an agent's.
- **Typing and clicking in the Browser tab** (#489, 2026-09-24). With the Browser tab focused,
  the page takes the mouse, the wheel and the keyboard as in Chromium: clicks, double clicks and
  right clicks where you make them, drags that keep going when the pointer leaves the tab, the
  wheel at 100 pixels a detent, typing and editing keys, Ctrl chords such as select all and
  undo, a compose sequence's character, and an input method's text. Ctrl+V pastes the system
  clipboard into the page and Ctrl+C and Ctrl+X copy the page's selection to it, since
  headless Chromium keeps a clipboard of its own. Super chords stay Marley's. A key reaches the
  screen in about 20 ms in a debug build, whose JPEG decoder is now built optimized.
- **A Browser tab** (#488, 2026-09-24). `marley: open browser` opens a tab in the main area
  that shows a web page rendered by Marley's own Chromium. The first time, Marley starts
  Chromium in the background, headless, as a user service with its own profile, and it keeps
  running when the tab or Marley closes. The tab is the page at the tab's size, laid out again
  when the tab resizes, with the page's title on the tab. Any agent or tool that speaks the
  Chrome DevTools Protocol can attach to the same Chromium, through the endpoint it writes into
  its profile, and what it does to the page shows in the tab. Typing and clicking in the page
  come next (#489). Without Chromium, the tab says so and names where it looked.
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

- **Telemetry is off by default** (#514, 2026-09-25). Marley no longer sends Zed's usage
  metrics or crash and hang reports unless you turn them on: `telemetry.metrics` and
  `telemetry.diagnostics` now default to false. Both are still settings, in your settings file or
  the Settings window (search "telemetry"), and turning one on works as it did.
- **E2E scenarios can click, drag and scroll** (#487, 2026-09-24). A scenario that names
  `compositor sway` runs the debug Marley in a headless sway of its own, whose seat is a
  virtual pointer (`script/e2e/seat-pointer.c`, built on first use) and a virtual keyboard
  (`wtype`) that nothing else sees. It gains `click`, `pointer_to`, `pointer_down`,
  `pointer_up` and `scroll`, types any text, and shoots the headless output. The user's
  desktop and Hyprland are never touched, and a scenario may define `teardown` for what it
  started outside Marley. Scenarios on Hyprland run as before, keys only.
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

- **Agents read blocks while vim or less is open** (#546, 2026-09-26). While a full-screen
  program held the terminal, an agent asking for an earlier command's output was told it had left
  the scrollback, or got the full-screen program's rows. Marley now reads blocks from the
  terminal's main screen, where they live, whatever the terminal shows.
- **Blocks survive a resize** (#544, 2026-09-26). Making a terminal narrower or wider rewraps
  its long lines, and every block before the rewrap lost its place: its bar and pill vanished or
  sat on the wrong rows, and an agent reading a block got rows from its neighbours. Blocks now
  follow their lines through the rewrap, while a full-screen program shows too, so the bars, the
  pills, Copy Output and what agents read stay right at any width.
- **One Marley at a time** (#513, 2026-09-26). Starting Marley while it already runs, from the
  menu or with `marley <folder>` in a terminal, used to start a second app on the same settings
  and data, and it hung. Now the second launch hands its folders and files to the Marley that
  runs, which opens them, and exits; with nothing to open it asks the running Marley to come
  forward. It prints what it did. A Marley with a data directory of its own
  (`--user-data-dir`) still runs beside the first.
- **Marley no longer dies at start on a seat without a keyboard** (#512, 2026-09-25). When the
  compositor had no keyboard to describe (a keyboard that had just gone, or a keymap that would not
  compile), the first keyboard event made Marley exit with nothing in its log; started from the
  menu, it just vanished. Marley now waits for a usable keymap and takes keys as soon as one
  arrives. The same fault is in upstream Zed.
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
