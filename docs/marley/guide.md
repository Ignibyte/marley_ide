# Marley: the guide

Marley is Ignibyte's fork of the Zed editor. It keeps everything Zed gives you (the editor,
language support, projects, settings, themes, the Agent Panel) and adds a layout built around
terminals, a block terminal, agents recognized in any terminal, an MCP server that lets agents
read your terminals and your browser, and a Browser tab that you and your agents drive together.

This guide describes Marley as of 2026-09-25 (through #516) on Linux, where it is built and
tested (Omarchy on Hyprland). A number such as #496 is the ticket that shipped a feature; its
entry in `CHANGELOG.md` says more. Marley is pre-1.0.

- [What Marley adds](#what-marley-adds)
- [Install and run](#install-and-run)
- [The Marley layout and the rail](#the-marley-layout-and-the-rail)
- [The block terminal](#the-block-terminal)
- [Agent CLIs in terminals](#agent-clis-in-terminals)
- [Zed's Agent Panel with Marley's tools](#zeds-agent-panel-with-marleys-tools)
- [Marley's MCP server](#marleys-mcp-server)
- [The Browser tab](#the-browser-tab)
- [Key bindings](#key-bindings)
- [Settings](#settings)
- [Where data lives](#where-data-lives)
- [Troubleshooting](#troubleshooting)
- [For developers](#for-developers)
- [What is planned](#what-is-planned)

## What Marley adds

| Area | What you get |
|---|---|
| The Marley layout | A rail on the left with each project, its terminals and its agent threads. Terminals open in the main area, the Agent Panel docks on the right, and nothing opens the bottom Terminal Panel. |
| The block terminal | Every command you run becomes a block with its exit status. Keys move between blocks, a block's output copies and its command reruns with one click, the prompt sits on the bottom row, and history suggests the rest of what you type. |
| Agents in terminals | Claude Code, Codex, Gemini CLI and OpenCode are recognized in any terminal. An agent bar shows the folder and branch, with rich input, Attach File and dictation. A Claude Code plugin adds desktop notifications and Marley's tools. |
| Marley's MCP server | Tools that let an agent list your terminals, read each command's exit code and output, and see and drive the Browser tab. |
| The Browser tab | A page from Marley's own Chromium, in a tab, with an address bar and one tab per page. An element picker, annotations and a flight recorder hand what you see to the agent. |

Marley keeps its own settings, database, logs and cache in `~/.config/marley`,
`~/.local/share/marley` and `~/.cache/marley`, apart from a stock Zed install's. To start from
your Zed settings, copy `~/.config/zed/settings.json` into `~/.config/marley/`; extensions
download again on first use.

## Install and run

### What you need

- The checkout (`/srv/stacks/marley_ide` on the dev box), with `cargo` and `just`.
- For the Browser tab: Chromium (`/usr/lib/chromium/chromium`, or `chromium` or
  `chromium-browser` on the PATH) and a systemd user session, since Marley starts Chromium with
  `systemd-run --user`.
- For the Claude Code plugin: `claude` on the PATH, and Python 3 for its MCP bridge.
- Optional: the agent CLIs you use (`claude`, `codex`, `gemini`, `opencode`) and Voxtype, for
  dictation.

### Install with `just install`

From the checkout, run `just install`. It waits until no other cargo runs on the machine (the
target directory is shared by every project on the dev box), builds `marley` in the release
profile, runs Marley's golden set of e2e scenarios against that build (#517, about fifteen
minutes; see [For developers](#for-developers)), and installs it under `~/.local` only when
every scenario passed. `just install --prefix DIR` installs somewhere else, and
`--skip-regress` installs without the golden set.

| Path under the prefix | What it is |
|---|---|
| `bin/marley` | The launcher: the command you run, and what the menu entry starts |
| `lib/marley/marley` | The binary |
| `share/applications/marley.desktop` | The menu entry, "Marley" |
| `share/icons/hicolor/512x512/apps/marley.png` | The icon, Zed's dev-channel icon for now |

The first release build takes about ten minutes on the dev box; a rebuild with nothing changed
takes under a second. The binary is about 2.1 GB because it keeps its debug information, which
gives a backtrace its file and line numbers. When it finishes, the installer prints the commit it
installed (and says so when the tree had uncommitted changes), the menu entry's path, a warning
when a Marley is running, and a note when the prefix's `bin` is not on your PATH. The entry
claims no file types, so Marley never becomes the default app for text files or folders.

Marley's windows have the class `dev.zed.Zed-Dev` (stock Zed's is `dev.zed.Zed`), which is what a
Hyprland window rule matches.

### Start it and update it

Start Marley from your desktop's app menu, or run `marley`. When stderr is not a terminal, as
when the menu starts it, the launcher appends stderr to `~/.local/share/marley/logs/stderr.log`,
with a line for each start, and keeps one older log as `stderr.log.old` once the file passes
10 MiB. On Marley's `dev` release channel a panic is printed to stderr and nowhere else, so this
file is the only record of one; `Marley.log` says nothing about it.

To update, pull and run `just install` again. Each file is replaced by a rename, so a Marley that
is running keeps its old build until you quit it and start it again.

### One Marley at a time

The installed and the debug builds are both on the `dev` channel and share `~/.config/marley`
and `~/.local/share/marley`. Since #513 only one Marley runs on a data directory: start another
one there, from the menu or with `marley <folder>` in a terminal, and it hands its folders and
files to the Marley that runs, which opens them, and exits. Started with no path, it asks the
running Marley to come forward, and Omarchy's Hyprland brings its window to the front
(`focus_on_activate`). The second launch says what it did on stdout: "Marley is already running
on <data dir>; it was handed 1 of this launch's paths".

- To run the debug build while the installed Marley runs, give it a data directory of its own:
  `marley --user-data-dir ~/marley-scratch <folder>`. It gets its own settings, database, logs
  and Chromium, and behaves as a fresh install.
- A Marley that hangs keeps its data directory: a new launch hands off to it and exits. Kill
  the hung one (`pgrep -x marley` finds it), and the next launch starts.

### The debug build

`just build` builds the debug `marley` into the shared target directory
(`/mnt/fast/target/debug/marley` on the dev box). The e2e scenarios run this build. A debug
build reads its assets (the default settings and keymaps) from the checkout at run time, and finds
the checkout from its own path or the working directory. Its path is outside the checkout on the
dev box, so start it from the checkout:

```sh
cd /srv/stacks/marley_ide && /mnt/fast/target/debug/marley 2>>~/marley-debug.stderr
```

Started anywhere else it panics with "dev asset loading requires running from within the
checkout". That panic comes after the installation id is written, so the next launch counts as an
existing install and Zed may write its editor layout into your `settings.json`; look at the file
after such a failure. Send the debug build's stderr to a file, never to `/dev/null`, because a
panic goes there and nowhere else.

## The Marley layout and the rail

### Two layouts

Marley opens in the Marley layout (#460). The choice is the setting `marley.layout`, `"marley"`
by default or `"zed"`, and two commands write it for you: `marley: use marley layout` and
`marley: use zed layout`. A switch changes every open window at once, with nothing to restart.

| | Zed layout | Marley layout |
|---|---|---|
| Left column | Zed's Threads Sidebar | The rail |
| Terminals | The bottom Terminal Panel | The main area. Tasks, New Terminal and Open in Terminal open there, and the Terminal Panel's button is hidden. |
| Agent Panel | Where Zed puts it | Docked on the right |
| A folder you open for the first time | Zed's behavior | Starts with a terminal at its root |
| Ctrl+\`, Ctrl+J, Ctrl+~ | Zed's Terminal Panel | The center terminals |

The block terminal, the agent bar, the MCP server and the Browser tab work in both layouts. The
layout sets two defaults, `terminal.button: false` and `agent.dock: "right"`, under your own
settings, so a value you set wins in either layout. A switch to the Zed layout and back gives each
dock the panel it showed before (#456).

Zed's Panel Layout presets, Classic and Agentic, would rewrite the docks the Marley layout sets.
In the Marley layout, choosing one shows a toast saying the presets belong to Zed's layout, with a
Use Zed's Layout button. The title bar's Panel Layout menu still lists them, with Custom checked.

With AI turned off (`disable_ai`), Zed draws no sidebar at all, the rail included.

### The rail

The header reads PROJECTS and carries Add Project (a folder with a plus). Add Project opens Zed's
recent-projects popover: your recent projects, Open Local Folders and Open Remote Folder. The
filter field sits under the header.

Each project has a header row: its name as a muted label, a chevron that folds it, an attention
dot when a row it hides (folded, or filtered out) needs you, and a `+`. Under the header come the
project's terminals, its Browser tabs, its Zed agent threads, newest first, and the ports its
servers listen on. A line separates one project from the next.

| Row | What it shows | A click |
|---|---|---|
| Terminal | A `>_` icon, the title, and the working directory relative to the project root (`~` for your home outside it, empty at the root) | Shows its project, focuses the terminal, clears its bell |
| Agent CLI | The agent's icon, the title the CLI sets (or the agent's name), and "Claude Code · working" or "Claude Code · waiting" | As a terminal row |
| Zed agent thread | The agent's icon, the thread's title, and "Zed Agent · working" (or idle, waiting, failed) | Shows the project and opens the thread, focused, in the Agent Panel on the right |
| Port | A server icon, `:<port>` and the process's name, and the URL | Opens the URL in a Browser tab of the project |

One row is highlighted at a time: the Agent Panel's thread while the panel has the focus, else the
active terminal, else the displayed project's header.

Dots mark what needs you:

- A bell in a terminal lights its row, and the header of its project while the project is folded.
  A desktop notification from a terminal marks it the same way.
- A thread whose run ends while it is off screen lights its row until you show it. A thread
  waiting for your confirmation also lights the dot on the sidebar toggle and on a folded
  project's header.
- Zed's own desktop notifications for threads still fire.

Terminal rows rename and close from the rail (#452):

- Double-click a row, or right-click it and choose Rename, to edit the name in its tab as Zed's tab
  rename does. The name survives restarts, and stays on the row while an agent CLI runs.
- Right-click and choose Close, or use the close button that appears under the pointer. Marley
  closes the tab as Zed does, asking first while a task runs in it.

Port rows (#521) are the TCP ports your processes listen on from inside the project's folders:
a dev server you started in its terminal, or anywhere else in the project. Marley looks every
three seconds while the rail is open, so a row comes within seconds of a server's start and
goes when it stops.

- A server on `0.0.0.0` or `::` gets a URL on `127.0.0.1` or `[::1]`, and a server on two
  addresses of one port gets one row.
- The pointer on a row shows the process's command line, working directory and pid, and three
  buttons: Open (the URL in a Browser tab of the project, or the tab already on it), Copy (the URL
  on the clipboard) and Stop. Stop sends the process SIGTERM once a fresh look finds it still
  listening on that port; otherwise a message says Marley left it alone.
- Marley's own listeners and other users' processes get no row, and neither does a server whose
  working directory is in no project, such as one that moved to `/` when it went to the
  background.

A project header's right-click menu has Move Project Up and Move Project Down. Ctrl+Alt+J closes
and opens the rail. A rail you close stays closed after a restart, and its width is saved with
the window; one width holds in both layouts.

### The + menu

A project's `+` ("New in this project") starts things in that project.

| Entry | What it does |
|---|---|
| New Terminal | A shell in the project's directory, in the main area |
| New Browser Tab | Shows the project and opens a blank page in a new Browser tab, the cursor in the address bar, starting Chromium if it must |
| New Agent Thread | A submenu with the Zed Agent and each external agent configured in Zed, by name; starts a thread in this project's Agent Panel |
| Agent CLIs | Under their own header, each agent CLI found on the PATH; starts it in a new terminal at the project root |

Ctrl+Alt+N opens the same choices in the New Agent picker ("Start an agent in this project…"),
in either layout. Zed's agents are marked Thread and the CLIs Terminal, and typing filters the
list. With AI turned off the key opens nothing.

A folder you open for the first time in the Marley layout starts with a terminal at its root,
focused (#455). A project you opened before comes back as you left it, with its saved terminals
or none.

### The rail from the keyboard

Ctrl+Alt+; moves the focus into the rail and back out. In the rail, Up and Down move the
highlight, Home and End jump to the first and last row, Enter opens the row as a click does, Left
folds a project or climbs from a row to its project, and Right unfolds it. These are Zed's own list
keys, so your bindings for them apply. When the focus leaves, the highlight goes back to the row
the window shows.

### The filter

Press Ctrl+F in the rail, or click the field, and type. The rail keeps the projects, terminals and
threads whose name or title contains the text, ignoring the case of ASCII letters, and highlights
the matching characters. A project whose name matches keeps every row under it. The first match is
highlighted; Up and Down move from it and Enter opens it. Escape clears the filter, and a second
Escape takes you back to the rows. The filter stays until you clear it, and "No matches" says when
nothing matched. Vim's `/` does not reach it.

### The switcher

Ctrl+Tab in the rail or in the Agent Panel opens a switcher over the window's terminals and
threads, the ones you worked in last first, with the one before the current selected. While you
keep Ctrl held, each Tab moves down and Shift+Tab moves up. Let go of Ctrl to open the selection;
Enter or a click opens an entry too, and Escape closes the switcher. In a center pane Ctrl+Tab
stays Zed's tab switcher. The order lives in memory only.

The command palette's `multi workspace: next project`, `previous project`, `next thread` and
`previous thread` walk the rail from the highlighted row. They go round at the ends, pass over
what a fold or the filter hides, count terminal rows as threads, and work with the rail closed.

### Where terminals open

In the Marley layout nothing opens the bottom Terminal Panel.

- Tasks run in center terminals, from the task modal, a runnable or a code lens. A rerun reuses the
  task's terminal by Zed's rules, and a task that last ran in the panel reruns in the center.
- New Terminal and every Open in Terminal menu open a center terminal, the latter in the folder it
  names.
- Ctrl+\` switches between the code and the project's terminals: from an editor to the terminal
  you used last, or a new one, and from a terminal back to the editor you used last.
- Ctrl+J does the same while the bottom dock is closed and would show the Terminal Panel.
- Ctrl+~ opens a new center terminal.

Vim's `:!` and external agents' login terminals still open the Terminal Panel, since they call it
directly. A Terminal Panel moved to a side dock with `terminal.dock` still opens with that dock's
toggle.

## The block terminal

### Shell integration for bash and zsh

Blocks come from the shell, which reports each prompt and each command to the terminal. When
Marley starts an interactive bash or zsh in a local terminal, it loads its integration:

- **bash** starts with `--rcfile ~/.local/share/marley/shell_integration/marley.bash`. The script
  sources your `~/.bashrc` first, then adds the reports (#463).
- **zsh** starts with `ZDOTDIR` pointing at `~/.local/share/marley/shell_integration/zsh`, whose
  `.zshenv` puts your own `ZDOTDIR` back (or leaves it unset if you had none) and runs your
  `.zshenv`, so zsh reads your startup files as it always did (#465). If you have no zsh startup
  files at all, zsh's new-user menu no longer opens.

A shell with the integration finds `MARLEY_SHELL_INTEGRATION=1` in its environment, and its tab
title leaves out the arguments Marley added. Tasks, remote terminals and a shell you configure
with arguments of its own start without the integration, and so show no blocks. fish has no
integration yet (TICKET-466).

Each local terminal also gives its shell a random value, which the scripts take out of the
environment before your files run and add to each command's report. A block's command counts as
verified only when its report carried that value, so output that imitates a report cannot pass
as a command you ran.

### Blocks

Every command the shell reports is a block (#470):

- a thin bar in the left margin beside its rows: green when it succeeded, red when it failed, blue
  while it runs;
- a small pill at the right end of its first row: a check, its exit code, or `running`;
- a faint red or blue tint over a failed or running block.

The terminal's text and rows stay as they were. Nothing is drawn while a full-screen program such
as vim holds the alternate screen.

### Moving between blocks, copying, rerunning

- Ctrl+Up scrolls to the start of the block above the top of the view, and Ctrl+Down to the next
  one, or back to the live screen from the last. Each press moves one block, however fast you
  press. A binding of your own on those keys wins (#473).
- Point at a block and two buttons show beside its pill (#474). Copy Output puts the block's output
  on the clipboard. Rerun Command, shown while the shell waits at its prompt and only for a
  verified command, clears what you had typed on the line and runs the command again. A click on
  either button starts no selection and never reaches a program that reads the mouse.

### The prompt at the bottom

While the screen has room to spare, as in a new terminal or one you just cleared, Marley draws its
content against the bottom of the pane, so the prompt sits on the last row and each command's
output pushes the rest up (#476). Scrolling back shows the history above it, and full-screen
programs are drawn as before.

### Autosuggestions

As you type at a bash or zsh prompt, the rest of the newest command that starts with what you typed
shows dimmed after the cursor, and the Right arrow (→) types it (#484). Commands you ran in this
terminal come first, then your shell's history file (`$HISTFILE`). With no suggestion, → moves the
cursor as before. Suggestions show only on the live screen, at the prompt, with nothing after the
cursor.

### Desktop notifications

A program that asks the terminal for a desktop notification, with OSC 9 (iTerm2's escape) or
OSC 777 `notify` (rxvt's and Ghostty's), gets one from Marley while you are not looking at that
terminal, because another pane has the focus or Marley's window is in the background (#478). The
title is the one the OSC 777 escape gives, or the terminal's tab for OSC 9. Clicking the
notification brings the terminal to the front, and its tab and rail row carry the mark a bell
gives them. An OSC 9 whose text starts with a number and a `;` (ConEmu's commands, such as the
`9;4` progress) is not a notification.

To try it, run this and click another pane within five seconds:

```sh
sleep 5; printf '\e]777;notify;Build;finished\a'
```

### The footer and the agent bar

Below its grid a terminal has a footer, empty until an agent CLI runs in the terminal's
foreground. Then the terminal gives up a row to the agent bar (#477), and gets it back when the
agent exits.

- At the left: the agent's icon and name, Attach File (`+`), Rich Input (the pencil), the
  microphone where Voxtype is installed, and, for Claude Code, the "Connect Claude Code to Marley"
  chip until Marley's plugin is installed.
- At the right: the folder the agent works in (`~` for your home) and its git branch, from the
  innermost repository of the project that holds the folder.

The rich input's editor opens above the bar.

### Rich input

While an agent CLI runs, Ctrl+G or the pencil opens a Zed editor above the bar for the agent's
prompt, one to eight lines high (#481). Select with the mouse, undo, move by word, and press
Shift+Enter for a new line. Enter sends the text to the agent as one paste (bracketed when the
agent asked for bracketed paste) followed by a carriage return, and closes the editor. Escape
closes it and keeps the draft for the next Ctrl+G. In a terminal with no agent, Ctrl+G reaches the
program as it always did.

### Attach File

The bar's `+`, or `marley: attach file` for the focused terminal, opens a file chooser (#479).
The files you pick, several at once if you like, go into the terminal as their full paths, quoted
where the shell needs it, the way dropping them on the terminal types them. Claude Code and the
other agents read a file from its path. In a remote project the chooser lists the machine the
terminals run on. The command works in any focused terminal, agent or not.

### The microphone

Where Voxtype, the dictation daemon Omarchy ships, is on the PATH, the agent bar has a microphone
(#480). Click it, or run `marley: toggle dictation`, to start or stop a dictation. The microphone
first puts the focus on its terminal, and Voxtype types what you said where the focus is. It turns
red while Voxtype records and yellow while it transcribes, however the dictation started, Omarchy's
own keys included. Marley never touches the audio. Without Voxtype, `marley: toggle dictation`
shows an error saying Voxtype is not on the PATH.

## Agent CLIs in terminals

### How Marley recognizes an agent

Marley recognizes four CLIs in the foreground of any terminal, however they were started, from the
program's name in its arguments: Claude Code (`claude`), Codex (`codex`), Gemini CLI (`gemini`)
and OpenCode (`opencode`) (#440). Such a terminal's rail row shows the agent's icon, the title the
CLI sets, and the agent's status: waiting once its output has been quiet for two seconds or when it
rings the bell, working otherwise. The agent bar shows under the terminal. When the agent exits,
the row is a plain terminal row again.

Marley looks at the foreground program when the terminal writes output, so an agent that has
printed nothing yet shows up once it draws its screen. An agent your `.bashrc` starts as a child
of the shell never becomes the terminal's foreground program (bash has job control off while it
reads its startup files), so Marley does not recognize it; start the agent at the prompt instead.

### Starting one

Choose a CLI under Agent CLIs in a project's `+` menu, or a Terminal entry in the New Agent picker
(Ctrl+Alt+N). Marley opens a center terminal at the project root and, once the shell is ready (or
after five seconds), types the program's name and Enter, nothing more. The shell outlives the CLI.
After a restart, an agent's terminal comes back as a shell in its old directory, without the CLI.

### The Claude Code plugin

While Claude Code runs in a terminal and its plugin list has no Marley plugin, the agent bar shows
"Connect Claude Code to Marley". Clicking it:

1. writes the plugin into `~/.local/share/marley/claude-code/`, a local plugin marketplace named
   `marley`;
2. runs `claude plugin marketplace add ~/.local/share/marley/claude-code`, unless Claude Code
   already knows that marketplace;
3. runs `claude plugin install marley@marley`;
4. shows a toast when it is done, or the error.

New Claude Code sessions use the plugin; in a running one, run `/reload-plugins`. Marley decides
whether to show the chip from `plugins/installed_plugins.json` under Claude Code's configuration
directory (`$CLAUDE_CONFIG_DIR`, else `~/.claude`), read when Marley starts, and it runs the
`claude` it found on the PATH then. With no `claude` there, the install fails with "`claude` is not
on the PATH".

The plugin, `marley` 1.1.0, brings two things:

- **Notifications.** Its hooks run when Claude Code needs your permission, when it waits for you,
  and when it finishes, and ask Claude Code to write an OSC 777 notify to its terminal. You get a
  desktop notification titled "Claude Code" that reads "`<project>` needs your permission",
  "`<project>` is waiting for you" or "`<project>` finished", under the rules in
  [Desktop notifications](#desktop-notifications). The hooks answer only in Marley's terminals
  (`TERM_PROGRAM=zed`) and stay silent elsewhere. Claude Code's print mode (`claude -p`) drops the
  sequence, so only interactive sessions notify.
- **Marley's tools.** It declares an MCP server named `marley` that runs Marley's bridge, so
  Claude Code started in any terminal on the machine finds Marley's tools while Marley runs, and an
  empty server while it does not.

## Zed's Agent Panel with Marley's tools

Zed's Agent Panel works as it does in Zed. In the Marley layout it docks on the right (Ctrl+?
focuses it), and the rail lists its threads under each project. Marley also offers its MCP server
to the panel's agents (#501):

- Marley adds a context server named `marley` to Zed's default settings: a stdio server that runs
  the bridge Marley writes to `~/.local/share/marley/mcp/marley-mcp-bridge`, with
  `MARLEY_MCP_ENDPOINT` naming the endpoint file. No password or token goes into your settings.
- The Zed Agent gets Marley's tools in its default Write profile, which turns on every context
  server.
- Each external agent you start in the panel, Claude Agent, Codex and the rest, is handed the
  `marley` server with its new session.
- The server runs for local projects.

To turn it off, set `"context_servers": { "marley": { "enabled": false } }`. An entry of your own
named `marley` replaces Marley's. `agent: open settings` opens Zed's Settings window at its AI page.

## Marley's MCP server

### What it serves, and where

While Marley runs, it serves MCP over Streamable HTTP (protocol revision 2025-06-18) on 127.0.0.1,
at a port it picks, behind a bearer token that changes at every start (#491). It writes the URL and
the token to `~/.local/share/marley/mcp-endpoint.json` as an MCP client's server entry (`type`,
`url` and an `Authorization` header), with mode 0600, and removes the file when Marley quits. If
the server or the file fails, Marley logs it, shows it once as a toast, and runs on without it.

### How agents reach it

| Agent | Route |
|---|---|
| Claude Code, in any terminal on the machine | The plugin's MCP server `marley`, which runs the bridge |
| The Zed Agent and the external agents of the Agent Panel | The context server `marley`, which runs the same bridge |
| Any other MCP client | Run the bridge as a stdio server, or read the endpoint file (its URL and token change at every start) |

A client allowed in Browser Clients (`marley: browser clients`) can run on another machine. Its
MCP client runs, as its stdio server, the line Browser Clients shows under "From another
machine, run it over SSH", `ssh -T -o BatchMode=yes <you>@<this machine> 'env
MARLEY_MCP_ENDPOINT=… …/marley-mcp-bridge'`. The line holds no token and still works after
Marley restarts. SSH must log in without a prompt, with a key or an agent, so connect once by
hand first to accept the host key. If the other machine knows this one by another name (an SSH
alias, a tailnet name), change the host in the line.

### The bridge

`marley-mcp-bridge` is a Python 3 script that uses the standard library only.

- It reads the endpoint file named by `$MARLEY_MCP_ENDPOINT`, else
  `${XDG_DATA_HOME:-~/.local/share}/marley/mcp-endpoint.json`.
- It sends the token only to an `http` URL on a loopback address, and never prints it.
- It opens a session of its own with Marley, replays the client's `initialize` when Marley starts
  over, and closes its session when its input ends.
- With no Marley running, it answers `initialize` itself and lists no tools, and a tool call
  answers "Marley is not running, so its tools are not there. Start Marley and call again."
- Every two seconds it checks whether Marley answers, and sends `notifications/tools/list_changed`
  when that changes, so the client lists the tools again.

A Marley started with `--user-data-dir` writes its endpoint file in that directory. The plugin's
bridge does not look there, so set `MARLEY_MCP_ENDPOINT` for a Claude Code that should reach that
Marley.

### The tools

The terminal family:

| Tool | What it gives |
|---|---|
| `terminal_list` | Every terminal in every window, center and Terminal Panel: its id, tab title, project, working directory, running command, and how many blocks it holds |
| `terminal_blocks` | A terminal's newest blocks, oldest first (50 unless `last` says otherwise, 500 at most): each command, whether the shell's hook reported it (`verified`), exit code, working directory, start time, duration, whether it runs, and whether its output is still in the scrollback; `redacted` counts the secrets hidden in the commands |
| `terminal_read` | One block's command and output as text, at most 2,000 lines and 256 KiB with the end kept, whether the start was cut, and `redacted`, how many secrets were hidden |

The ports family (#521):

| Tool | What it gives |
|---|---|
| `ports_list` | The ports the rail's port rows show: each listener's project, the project folder its working directory is in, its address, port, URL, pid, process name and working directory; never its command line, which can carry a token |

The browser family, reading:

| Tool | What it gives |
|---|---|
| `browser_tabs` | Each page: its id (the `tab` the other tools take), title, URL, whether it loads, and which tab a call without `tab` acts on, the one you focused last |
| `browser_look` | The page as you see it: URL, title, viewport, scroll, loading, the focused element and the selection, and the frame on your screen as an image |
| `browser_snapshot` | The accessibility tree as text: the interactive elements with refs such as `e3` (every node with `full`), cross-site iframes included, no field values |
| `browser_console` | The latest console messages and uncaught errors, oldest first, 200 at most, with secrets hidden |
| `browser_network` | The latest requests, 200 at most: method, URL with secret-looking values hidden, type, status, duration, failure; no headers or bodies |
| `browser_picks` | The elements you picked this session: each pick's id, tab, URL, title, what it is, your caption, and whether you sent it |
| `browser_pick` | One pick: its locators (test id, id, text, CSS path, each marked when it finds the element alone), role and name, the listeners with their script, line and column and, through source maps, the project file and line, what would block a click, its box, and a crop of the page as an image |
| `browser_annotations` | The boxes over a page: each one's id, box in page coordinates, note, who drew it and when |
| `browser_recordings` | The recordings you saved: id, tab, URL, title, when, how many seconds, frames and entries |
| `browser_recording` | One recording's timeline; `frame` gives one frame, counted from 1, as an image |

The browser family, acting. These need the `browser.write` grant:

| Tool | What it does |
|---|---|
| `browser_navigate` | Loads an `http` or `https` URL, in a new tab with `new_tab`, and answers once the page has loaded |
| `browser_back` | Goes back in the tab's history and answers once the page has loaded |
| `browser_click` | Clicks an element by its ref, or a point of the viewport, as your mouse does (`button`, `count`) |
| `browser_type` | Types text as key presses into an element by its ref, or where the focus is; `submit` presses Enter after |
| `browser_press` | Presses a key or a chord: Enter, Tab, Escape, ArrowDown, Ctrl+A, Shift+Tab |
| `browser_scroll` | Scrolls by pixels (`dy` down, `dx` right), or an element into view |
| `browser_annotate` | Draws a box with a note around an element or over an area, marked as the agent's; `clear` removes the agent's own boxes |

The tools that read or act in one page take `tab`, an id from `browser_tabs`, and without it act
on the tab you focused last. Every answer from a page names its tab.

The server also holds `fleet_snapshot` and `session_surface_to_human`, which it does not list. The
fleet stays empty until prong 2 feeds it, and a session write is refused, since Marley grants no
`session.write`.

### Grants and what keeps an agent in check

- Read tools need no grant. A write tool needs its class granted, and Marley grants
  `browser.write`, and nothing else, when it starts the server. No setting changes that today.
- The checks on the browser's write tools are your agent client's approval of each call (Claude
  Code asks by default) and the Browser tab, where every action happens in front of you and the
  Agent chip names it.
- The server listens on 127.0.0.1 only, refuses a request whose `Origin` is not loopback, takes
  bodies up to 1 MiB, and never logs the token.
- No tool runs script an agent supplies, and an agent may navigate to `http` and `https` URLs only.
- In the URLs the browser tools and the recordings report, the values of parameters whose names
  contain `token`, `key`, `secret`, `password`, `auth`, `code`, `sig` or `session` are hidden, and
  a user name and password are dropped.
- The snapshot writes no field values, `browser_look` leaves out the selection while a password
  field has the focus, a pick never records what was typed into a field, and the recorder counts
  typed characters without keeping them.
- What the terminal tools and `browser_console` hand an agent has its secrets hidden (#516): each
  one becomes `[redacted: <kind>]`, and the terminal tools count them in `redacted`. The kinds are
  private key blocks (PEM and PGP), values assigned to names that hold `TOKEN`, `SECRET`,
  `PASSWORD`, `PASSPHRASE`, `API_KEY`, `PRIVATE_KEY`, `ACCESS_KEY` or `CREDENTIALS` (`NAME=value`,
  `NAME: value`, `export NAME=value`), `Authorization: Bearer` values, a password or a token in a
  URL, AWS access key ids, GitHub, Slack, Stripe and Google API keys, OpenAI and Anthropic keys
  (`sk-`), and JWTs. Your own regular expressions in `marley.redaction_patterns` are hidden as
  `[redacted: pattern]`. The terminal itself still shows what was printed: only what leaves for
  a model changes. A name such as `TOKEN_LIMIT=5` loses its value too, since the rules hide a
  value rather than risk one; turn redaction off on the Marley settings page when an agent needs
  what they hide.

### Limits

- The server holds 32 sessions at most, and an idle one lasts 30 minutes, so a client that never
  closes its session holds a slot that long. The bridge closes its own.
- A call the app does not answer within 30 seconds fails with a tool error.
- A block's times are stamped when Marley applies each shell report, so durations are right to
  tens of milliseconds.

## The Browser tab

### Opening one

- In the rail, a project's `+` and then New Browser Tab.
- `marley: open browser`: gives a tab to each page that has none, else shows the Browser tab you
  used last, else opens a blank page.
- Ctrl+T in a Browser tab, or `marley: new browser tab`: a blank page in a new tab, with the focus
  in its address bar.
- An agent's first browser action opens a tab when none is open, without taking your focus.

A tab shows its page's title, the URL as its tooltip, and a globe icon. While the browser starts,
the tab reads "Starting Chromium…", "Connecting to Chromium…" or "Opening a page…". If the browser
failed, the tab says why, and "Run “marley: open browser” to try again."

### Marley's Chromium

The first Browser tab starts Chromium in the background as a transient systemd user unit named
`marley-browser-` and twelve hex digits, one per Marley data directory (#488).

- It runs headless, with a profile of its own for each project under
  `~/.local/share/marley/browser/projects/`, apart from your own browser's. It talks only to
  Marley's relay, which runs beside it, and listens on no network port. It keeps cookies without
  the desktop keyring, so it never waits on an unlock prompt, and it opens no page of its own.
- Websites see it as Chrome: its user agent says `Chrome/<version>`, not `HeadlessChrome`, with
  Chromium's own client hints, in cross-site frames too, so bot checks that refuse a headless
  browser let it in (#539). The first document of a popup a page opens, and service workers,
  still see the headless name.
- The binary is `$MARLEY_CHROMIUM` when that is set, and no other is tried. Otherwise it is
  `/usr/lib/chromium/chromium`, the browser behind Arch's and Debian's `/usr/bin/chromium`
  launcher (which would add your `chromium-flags.conf`), else `chromium` or `chromium-browser` on
  the PATH.
- It keeps running when a tab, a window or Marley closes, and ends when you log out. Closing the
  last Browser tab leaves it running with no page.

Any tool that speaks the Chrome DevTools Protocol can attach to the same Chromium through the
relay: `relay.json` beside the project's profile holds its WebSocket address and a token, new at
each start, which the tool sends as `Authorization: Bearer <token>` (Playwright's
`connectOverCDP(url, { headers })`, or Playwright MCP's `--cdp-header`). What it does shows in the
tab.

### The toolbar and the address bar

From left to right: Back, Forward, Reload (a Stop button while a page loads, with a thin accent
bar along the toolbar's lower edge), the address bar, the crosshair (Pick an Element for the
Agent), the pencil (Annotate the Page), the red dot (Save the Last Minute of This Page), and, while
an agent acts, the Agent chip.

The address bar ("Search or enter an address") takes:

- a URL with a scheme Chromium opens (`http`, `https`, `file`, `about`, `data`, `chrome`,
  `view-source`), as you typed it;
- a host with an optional port and path, such as `localhost:3000` or `example.com/docs`, over
  `http` for a loopback host (`localhost`, a name under it, `127.0.0.1`, `[::1]`) and `https`
  otherwise;
- anything else, as a DuckDuckGo search.

It follows every navigation, a link's or an agent's. A page that fails to load shows Chromium's
error page, and the address bar keeps the address that failed. Ctrl+L puts the focus in the address
bar with its text selected, Enter goes, and Escape puts the page's URL back and gives the page the
focus. Alt+Left and Alt+Right go back and forward; Ctrl+R and F5 reload.

### Tabs

Every page of Marley's Chromium has a Browser tab of its own (#493).

- A link that opens a new window, or a page's `window.open`, opens a tab beside the page that
  opened it, with the focus.
- A page an agent opens leaves your focus where it is: behind the tab you are in, or, when no
  Browser tab is open and you work in another pane, in a new pane split to its right.
- Closing a tab closes its page, and a page that closes itself closes its tab.
- Tabs are saved with their workspace (#494). When Marley starts again while its Chromium still
  runs, each tab comes back on its own page, with what you typed into the page and where you
  scrolled. When Chromium has stopped since, each tab opens its saved address again. A returning
  tab shows its old title and address until its page is back.
- Closing a window, or quitting, closes no page. A page no restored tab claims gets a tab at the
  next `marley: open browser`, or when an agent acts in it.

### Typing, clicking and the clipboard

With the tab focused, the page takes the mouse, the wheel and the keyboard as in Chromium: clicks,
double clicks and right clicks, drags that keep going when the pointer leaves the tab, the wheel,
typing and editing keys, Ctrl chords such as select all and undo, a compose sequence's character,
and an input method's text (#489). Ctrl+V pastes the system clipboard into the page, and Ctrl+C
and Ctrl+X copy the page's selection to it, since headless Chromium keeps a clipboard of its own.
Super chords stay Marley's, and so do keys Zed binds above the tab: Ctrl+S saves and Ctrl+W closes
the tab. Escape reaches the page.

### Select lists and dialogs

- **Select lists (#495).** A page's drop-down list, which headless Chromium would open where no one
  can see it, opens as a Marley list under it, in the page or in a frame from another site. The
  current option is checked, each group's name sits over its options, and a disabled option is
  greyed. Click an option, or use the arrows and Enter; Escape or a click elsewhere closes the list
  and keeps the value. Alt+Down, F4 or Space on a focused list opens it, and the arrows on a closed
  one change its value. Your choice reaches the page's scripts as an input and a change event. A
  list an agent clicks stays shut, so an agent never takes your focus that way.
- **Dialogs (#490).** A page's `alert`, `confirm` or `prompt` opens as a card over the page that
  names the site asking ("`<host>` says"), with a field for a prompt's answer. Enter answers OK
  and Escape Cancel. A page that asks before you leave reads "Leave this page?" with a Leave button.
  Navigating, going back or forward, or reloading answers an open dialog with Cancel first.

### What agents can do in the tab

Through the browser tools an agent sees what you see (the frame, the accessibility tree, the
console and the network) and acts with the same events your mouse and keys send (#492). A write
call first brings its page's tab to the front of its pane, unless that pane has your focus. The
Agent chip in the toolbar names each action while it runs and for five seconds after; typed text
shows as its length, never itself.

### The element picker and the pick tray

The crosshair, or Ctrl+Shift+C, turns on pick mode (#496). Chromium's own inspect highlight and
its tooltip follow the pointer, and a click picks the control under it (a click on a button's label
picks the button) without reaching the page. Escape leaves pick mode.

Each pick waits in a tray under the toolbar, newest first: "Pick N", what the element is, a
caption field ("What should the agent know? Enter sends"), Send and Discard. Enter or Send types a
line such as

```
[browser pick 1: button “Save changes” on localhost:3000/card; browser_pick id 1] Make this green
```

into the terminal you used last and takes you there; press Enter to hand it to the agent, which
reads the pick with `browser_pick`. A sent pick shows what was sent, and its Discard takes it out
of the tray while the agent can still read it. With no terminal used yet, the tray reads "No
terminal to send to: click in one, then Send." Picks last for the session. Pick mode works in the
page's own frames, not inside a cross-site iframe.

### From a pick to its source

A pick's row shows where the element's listener was written (#497). Marley reads the script's
source map, from the page or from the script itself, and finds that source in your project, so the
tray reads `click src/app.ts:2`, and a click on it opens `src/app.ts` at line 2 in the editor. A
listener whose script has no map, or whose source is not in the project, shows its script's name
and line, muted, and opens nothing. The tooltip lists every listener. A page whose framework
delegates its events, such as React's root listener, shows the framework's listener rather than the
handler your app wrote. Every line and column the pick tools give counts from 1.

### Annotations

The pencil, or `marley: annotate`, turns on annotate mode (#498). Drag a box around anything on
the page and type a note: Enter keeps it and Escape drops it. The wheel still scrolls the page, and
Escape in the page ends the mode. Marley draws the boxes over the page, never into it, and they
stay on what they mark as the page scrolls. Your boxes use the theme's warning color; an agent's
use its accent color, with a sparkle on the note. Click a note and press Delete or Backspace to
remove its box. Loading another page clears the boxes, and they last for the session. Agents draw
with `browser_annotate` and list every box with `browser_annotations`.

### The flight recorder and "record this"

While a Browser tab shows a page, Marley keeps that page's last minute (#499): your clicks and
where they landed, keys by name, typing as a count of characters (never the characters), the
agent's actions, console messages, requests with secret-looking URL values hidden, navigations, the
page's accessibility snapshot after each load, and up to two frames a second. Chromium sends a frame
only when the page changes, so a quiet page leaves few. Nothing is kept for a page behind another
tab.

The red dot, or `marley: record this`, saves that minute into
`~/.local/share/marley/browser/recordings/<id>/` (`timeline.json` and `frames/`), never into a
project, and a toast names it: "Saved this page's last minute as recording `<id>`; agents read it
with browser_recording." An agent you tell "it broke just now" can list the recordings with
`browser_recordings` and read what happened with `browser_recording`.

### Limits of the Browser tab

- Headless Chromium draws no browser UI. Marley draws dialogs and select lists, but not file
  choosers, downloads, context menus or the page's cursor shape.
- On a screen scaled above 1, pages look soft: Chromium's frames stay at 1×.
- Japanese and Chinese through a live input method are untested.
- A title a single-page app sets later, with no navigation, does not reach the tab.
- The network log starts when Marley starts watching a page. Console entries from before are
  replayed; requests are not.
- Every tab shares one Chromium profile, so a login in one project is a login in all of them
  (TICKET-507 plans a browser context per project).

## Key bindings

Marley's own bindings load after Zed's defaults and before your keymap, so they win over a Zed
default in the same context and lose to your own bindings (`~/.config/marley/keymap.json`). The
table lists Marley's bindings and the Zed keys whose meaning Marley changes or relies on.

| Key | Where | What it does |
|---|---|---|
| Ctrl+Shift+P | Anywhere | Zed's command palette, for every `marley:` command |
| Ctrl+Alt+N | Anywhere | The New Agent picker |
| Ctrl+Alt+; | Anywhere | Moves the focus into the rail, or back out |
| Ctrl+Alt+J | Anywhere | Opens or closes the rail |
| Ctrl+? | Anywhere | Focuses the Agent Panel (Zed's) |
| Ctrl+\` | Marley layout | Switches between the code and the project's terminals |
| Ctrl+J | Marley layout | The same, while the bottom dock is closed |
| Ctrl+~ | Marley layout | A new center terminal |
| Up, Down, Home, End | Rail | Moves the highlight |
| Enter | Rail | Opens the highlighted row |
| Left, Right | Rail | Folds a project or climbs to it; unfolds |
| Ctrl+F | Rail | The filter |
| Escape | Rail filter | Clears the filter; a second press goes back to the rows |
| Ctrl+Tab, Ctrl+Shift+Tab | Rail, Agent Panel | The switcher; hold Ctrl, press Tab or Shift+Tab to move, let go to open |
| Ctrl+Up, Ctrl+Down | Terminal | The previous or next block |
| → | Terminal, at a prompt | Takes the autosuggestion |
| Ctrl+G | Terminal running an agent CLI | Rich input |
| Enter, Shift+Enter, Escape | Rich input | Sends; adds a line; closes and keeps the draft |
| Ctrl+Q | Terminal | Goes to the shell, not Zed (quit with `zed: quit`) |
| Ctrl+L | Browser tab | The address bar |
| Enter, Escape | Address bar | Goes; puts the page's URL back and focuses the page |
| Ctrl+T | Browser tab | A new Browser tab |
| Alt+Left, Alt+Right | Browser tab | Back, forward |
| Ctrl+R, F5 | Browser tab | Reload |
| Ctrl+Shift+C | Browser tab | Pick mode on or off |
| Escape | Browser tab | Goes to the page; ends pick or annotate mode |
| Ctrl+V, Ctrl+C, Ctrl+X | Browser tab | Paste the system clipboard; copy or cut the page's selection |
| Enter | A pick's caption | Sends the pick |
| Enter, Escape | An annotation's note | Keeps the box; drops it |
| Delete, Backspace | A selected annotation | Removes it |
| Enter, Escape | A page's dialog | OK; Cancel |
| Alt+Down, F4, Space | A focused select list | Opens Marley's list |
| Up, Down, Enter, Escape | An open select list | Moves; chooses; closes |

Commands with no key of their own, from the command palette:

| Command | What it does |
|---|---|
| `marley: use marley layout`, `marley: use zed layout` | Switch the layout in every window |
| `marley: open settings` | The Settings window on its Marley page |
| `marley: new agent` | The New Agent picker (also Ctrl+Alt+N) |
| `marley: attach file` | Attach File for the focused terminal |
| `marley: toggle dictation` | Starts or stops a Voxtype dictation |
| `marley: open browser` | Shows or opens a Browser tab |
| `marley: new browser tab` | A new Browser tab (Ctrl+T inside one) |
| `marley: pick element`, `marley: annotate`, `marley: record this` | The Browser tab's three buttons, while a Browser tab has the focus |
| `multi workspace: next project`, `previous project`, `next thread`, `previous thread` | Walk the rail |

## Settings

Marley keeps its settings in `~/.config/marley/settings.json`, in Zed's format (Ctrl+Alt+, opens
the file; Ctrl+, opens the Settings window). `marley: open settings` opens the Settings window on its Marley page,
first in the list (#515). The page has a Layout section, with the layout as a dropdown, an Agents
section with Redact Secrets for Agents (#516), and a Privacy section with the two telemetry
toggles; later Marley settings add their sections there.

Marley's own keys in the file:

```jsonc
{
  "marley": {
    // "marley" (the default) or "zed". marley: use marley layout and use zed layout write it.
    "layout": "marley",
    // Hide secrets in what Marley's tools give agents (true by default), and more to hide: each
    // entry is a regular expression, and one that does not compile is named in a notification.
    "redact_secrets_for_agents": true,
    "redaction_patterns": ["INTERNAL-[0-9]{6}"]
  },

  // Marley adds a context server named "marley" to Zed's defaults for the Agent Panel's agents.
  // This turns it off; an entry of your own named "marley" replaces Marley's.
  "context_servers": { "marley": { "enabled": false } }
}
```

The Settings window shows the two defaults the Marley layout sets (`terminal.button: false` and
`agent.dock: "right"`) as Zed's defaults, so in the Marley layout a stored `terminal.button: false`
looks like the default and has no reset control.

Telemetry is off by default in Marley (#514): `telemetry.metrics` and `telemetry.diagnostics`
default to false, and turning either on works as in Zed.

Environment variables and flags:

| Name | Read by | What it does |
|---|---|---|
| `MARLEY_CHROMIUM` | Marley | The Chromium binary to run; when set, nothing else is tried |
| `MARLEY_MCP_ENDPOINT` | The bridge | The endpoint file to read |
| `CLAUDE_CONFIG_DIR` | Marley | Claude Code's configuration directory, for whether the plugin is installed (default `~/.claude`) |
| `ZED_LOG`, `RUST_LOG` | Marley | The log filter. It matches the crate a line is logged from: `ZED_LOG=marley_browser=debug` logs each input-to-frame time, and `marley_workbench::browser` names the Browser tab's own lines |
| `XDG_DATA_HOME` | Marley, the bridge, the launcher | Where `marley/` data lives (default `~/.local/share`) |
| `--user-data-dir DIR` | Marley | A separate profile: config in `DIR/config`, data in `DIR` |
| `--prefix DIR` | `just install` | Where to install (default `~/.local`) |

## Where data lives

| Path | What it holds |
|---|---|
| `~/.config/marley/settings.json`, `keymap.json` | Your settings and key bindings |
| `~/.local/share/marley/db/` | Zed's database: workspaces, terminals, the saved Browser tabs |
| `~/.local/share/marley/logs/Marley.log` | The log, with `Marley.log.old` before it |
| `~/.local/share/marley/logs/stderr.log` | stderr of a Marley the menu started, with `stderr.log.old` |
| `~/.local/share/marley/logs/telemetry.log` | Telemetry events, written only while `telemetry.metrics` is on |
| `~/.local/share/marley/mcp-endpoint.json` | The MCP server's URL and token (mode 0600), while Marley runs |
| `~/.local/share/marley/mcp/marley-mcp-bridge` | The bridge Zed's agents run, written at each start |
| `~/.local/share/marley/claude-code/` | The Claude Code plugin, as a local marketplace |
| `~/.local/share/marley/shell_integration/` | `marley.bash` and `zsh/.zshenv` |
| `~/.local/share/marley/browser/projects/<key>/` | A project's browser: `profile/`, `project.json`, and while it runs the relay's `relay.sock` and `relay.json` (mode 0600) |
| `~/.local/share/marley/browser/recordings/<id>/` | A saved recording: `timeline.json` and `frames/` |
| `~/.cache/marley/` | Zed's cache |
| `~/.local/bin/marley`, `~/.local/lib/marley/marley` | The installed launcher and binary |
| `~/.local/share/applications/marley.desktop` | The menu entry |

Every `~/.local/share/marley` path follows `$XDG_DATA_HOME`, and a Marley started with
`--user-data-dir DIR` keeps them all under `DIR`.

## Troubleshooting

- **Marley closed with no message.** On the `dev` channel a panic goes to stderr and nowhere else,
  and `Marley.log` stays silent. For the installed Marley, read
  `~/.local/share/marley/logs/stderr.log`. For the debug build, send stderr to a file when you
  start it. With `telemetry.metrics` on, `telemetry.log` ends with an `App Closed` event after a
  normal quit, which tells a quit from a crash.
- **Starting Marley does nothing.** A Marley already runs on that data directory, and the
  launch handed its paths to it (the launcher's `stderr.log` or the terminal shows the line). If
  that Marley is hung, kill it (`pgrep -x marley`) and start again. To run two on purpose, give
  the second `--user-data-dir`.
- **The debug build panics with "dev asset loading requires running from within the checkout".**
  Start it with the checkout as the working directory. After such a failed first launch, check your
  `settings.json`: Zed may have written its editor layout into it.
- **Ctrl+Q does not quit.** With the focus in a terminal, Zed sends Ctrl+Q to the shell, as it does
  with the other keys the terminal passes on (Ctrl+R among them). Quit from the command palette,
  `zed: quit`.
- **A new folder opens behind "Unrecognized Project".** That is Zed's Restricted Mode, its trust
  check for a folder it has not seen. Trusting the folder lets Zed apply its project settings and
  start its language and MCP servers.
- **There is no rail.** You may be in the Zed layout (`marley: use marley layout`), the rail may be
  closed (Ctrl+Alt+J, and a closed rail stays closed after a restart), or AI may be turned off
  (`disable_ai`), in which case Zed draws no sidebar at all.
- **Is it Marley's bug or Zed's?** For panels, docks and where terminals open, `marley: use zed
  layout` gives you Zed's own layout and routing. A problem that stays there is Zed's.
- **An agent shows as a plain terminal.** Marley sees the foreground program when the terminal
  writes output, so an agent appears once it draws. An agent your `.bashrc` starts is never the
  terminal's foreground program; start it at the prompt. Only `claude`, `codex`, `gemini` and
  `opencode` are known.
- **No desktop notification from Claude Code.** Check that the plugin is installed (the chip is
  gone), that the session started after it or ran `/reload-plugins`, that you were not looking at
  that terminal, and that it is not print mode (`claude -p`), which drops the hook's escape. On
  Omarchy, notifications go to Quickshell;
  `busctl --user monitor org.freedesktop.Notifications` shows whether Marley sent one.
- **A terminal shows no blocks.** Blocks need Marley's integration: a bash or zsh that Marley
  started in a local interactive terminal. `echo $MARLEY_SHELL_INTEGRATION` prints `1` in one.
  Tasks, remote terminals, a shell configured with its own arguments, and fish run without it, and
  a full-screen program hides the blocks while it runs.
- **Typed characters vanish on a long prompt in the first terminal of a launch.** The first
  terminals of a launch open at Zed's small starting size, and a two-line prompt wider than 100
  columns (starship's on a deep path) is misdrawn until the next prompt. Terminals opened later are
  fine since #485; TICKET-486 is for the first ones.
- **Claude Code sees no Marley tools.** Marley must be running (the bridge answers "Marley is not
  running" otherwise, and tells Claude Code within about two seconds of Marley starting), and the
  plugin must be installed. A Marley started with `--user-data-dir` needs `MARLEY_MCP_ENDPOINT` set
  for the bridge.
- **An MCP client cannot open a session.** The server holds 32 sessions, and an idle one lasts 30
  minutes. A client that never closes its sessions fills the slots.
- **The Browser tab says "No Chromium found".** Install Chromium, or name its binary with
  `MARLEY_CHROMIUM`. Marley looks for `/usr/lib/chromium/chromium`, then `chromium` and
  `chromium-browser` on the PATH.
- **"Chromium stopped as it started" or "Chromium did not answer within 15 seconds".** The message
  names the unit; `journalctl --user -u <unit>` says why. Then run `marley: open browser` again.
- **Chromium runs after you closed every tab, or quit Marley.** The unit outlives Marley by design
  and ends when you log out. `systemctl --user list-units 'marley-browser-*'` finds it, and
  `systemctl --user stop <unit>` stops it.
- **Ctrl+S or Ctrl+W does a Zed thing in the Browser tab.** Zed binds them above the tab: Ctrl+S
  saves and Ctrl+W closes the tab.
- **`browser_network` misses a request the console mentions.** The network log starts when Marley
  starts watching the page, and only the console replays what came before.
- **A recording has few frames.** Chromium sends a frame only when the page changes.
- **An agent reads `[redacted: …]` where it needs a value.** Marley hid what looked like a
  secret. Turn off Redact Secrets for Agents on the Marley settings page (`marley: open
  settings`) while the agent needs it, or print the value under a name the rules do not take.
- **`zed://` links open stock Zed.** Stock Zed owns the scheme where it is installed; Marley's own
  URL scheme waits on TICKET-445.

## For developers

`just` lists the recipes. Every cargo recipe first waits until no other cargo runs on the machine,
and that includes rust-analyzer's `cargo check` in a Marley that has this repository open.

| Recipe | What it runs |
|---|---|
| `just build` | The debug `marley` |
| `just install [--prefix DIR] [--skip-regress]` | The release `marley`, checked by the golden set, then installed with its menu entry |
| `just gate-diff` | Every gate on the change, and the receipt a commit needs |
| `just gate-fast` | The same gates without a receipt, for a change with no Rust |
| `just clippy <crates>` | Clippy on the named crates, every target, warnings as errors |
| `just fmt <crates>` | Formats the named crates |
| `just e2e <scenario>` | A ticket's e2e scenario (`script/e2e/`) against the debug build, on a hidden workspace, or in a headless sway of its own when it clicks |
| `just regress [scenario...]` | Marley's regression suite: the golden set in `script/e2e/golden` (or the scenarios named), each in a headless sway, each checking itself; a PASS or FAIL line per scenario and a verdict. `E2E_BINARY=<path>` runs another build. The runs go under `~/.local/state/marley/regress/` |
| `just shot <name> [seed]` | One shot of the debug Marley on a copy of your profile; `OPEN=<path>` opens a path |

Work moves through four phases, Plan, Code, Test and Complete (`/pipeline:plan`, `/pipeline:code`,
`/pipeline:test`, `/pipeline:complete`), under `CONSTITUTION.md`. Every change is proven by an e2e
scenario on the real Marley, and every change outside `crates/marley_*` and Marley's other owned
paths gets a row in `docs/marley/zed-touchpoints.md`.

## What is planned

`docs/planning/tickets/BACKLOG.md` orders the queue, and most tickets in it have a spec waiting in
`docs/planning/pipeline/queued/` (`docs/planning/design-notes/specs-batch-2026-09-25.md` says where
each came from). The next ones:

| Ticket | What it brings |
|---|---|
| TICKET-517 | Marley's own regression suite: a golden set of scenarios that check themselves, run before every `just install` |
| TICKET-519 | Claude Code's hook events in the rail: the prompt, the tool in flight, what it waits on, its last message |
| TICKET-520 | Each terminal knows its id, and Marley's tools know which terminal called them |
| TICKET-503 | A local URL in a terminal opens in a Browser tab, and the terminal offers the URL a dev server printed |
| TICKET-504 | Browser tabs as rows of their project in the rail |
| TICKET-518, TICKET-505 | A fuller pick (HTML, styles, the React component), then the same element found again after the agent's fix |
| TICKET-507 | A Chromium and a profile per project |

After those come saved Playwright scripts and trusted outside clients for the browser, the
approvals inbox, per-turn diffs, permission modes, worktree agents with review and merge, and
the terminal items from Warp (an agent typing into a running program, blocks over ssh, filtering
a block, a sticky command header, runbook commands).

Waiting on purpose: fish shell integration (TICKET-466, for a machine with fish) and Marley's own
release identity, with its keyring label, updater, app id and URL scheme (TICKET-445).

`docs/marley/three-prong-plan.md` names the longer road: block-scoped path links and a jump to the
first failure (T2), a prompt editor (T3), tasks and runnables as blocks (T4), native block headers
with the shell's prompt hidden (T5), completions in the prompt (T6), and the control plane that
brings harness seats and Rusty's sessions into the rail (C1 to C5).
