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
- [System One](#system-one)
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
| Agents in terminals | Claude Code, Codex, Gemini CLI and OpenCode are recognized in any terminal. An agent bar shows the folder and branch, with rich input, Attach File and, once Voice is on, dictation. A Claude Code plugin adds desktop notifications and Marley's tools. |
| Marley's MCP server | Tools that let an agent list your terminals, read each command's exit code and output, and see and drive the Browser tab. |
| The Browser tab | A page from Marley's own Chromium, in a tab, with an address bar and one tab per page. An element picker, annotations and a flight recorder hand what you see to the agent. |
| System One | Off until you turn it on: typed questions to a model (TypeSafe's Jev first) about what Marley knows, sent only for the projects you list and masked, with every call in System One calls. |

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
  dictation once Voice is turned on.

### Install with `just install`

From the checkout, run `just install`. It waits until no other cargo runs on the machine (the
target directory is shared by every project on the dev box), builds `marley` in the release
profile and installs it under `~/.local`. `just install --regress` first runs Marley's golden set
of e2e scenarios against that build (#517, about an hour; see [For developers](#for-developers))
and installs only when every scenario passed. `just install --prefix DIR` installs somewhere
else.

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
filter field sits under the header. While Rusty is on and connected, PROJECTS gives way to two
buttons, Projects and Brain (see [The Brain view](#the-brain-view)), and the end button follows the
view shown.

Each project has a header row: its name as a muted label, a chevron that folds it, an attention
dot when a row it hides (folded, or filtered out) needs you, and a `+`. Under the header come the
project's terminals, its Browser tabs, its Zed agent threads, newest first, and the ports its
servers listen on. A line separates one project from the next.

| Row | What it shows | A click |
|---|---|---|
| Terminal | A `>_` icon, the title, and the working directory relative to the project root (`~` for your home outside it, empty at the root) | Shows its project, focuses the terminal, clears its bell |
| Agent CLI | The agent's icon, the title the CLI sets (or the agent's name), and "Claude Code · working" or "Claude Code · waiting" | As a terminal row |
| Zed agent thread | The agent's icon, the thread's title, and "Zed Agent · working" (or idle, waiting, failed) | Shows the project and opens the thread, focused, in the Agent Panel on the right |
| Port | A server icon, `:<port>` and the process's name, and the URL | Marks the row; a double-click or Enter opens the URL in a Browser tab of the project (#604) |

One row is highlighted at a time: the Agent Panel's thread while the panel has the focus, else the
active terminal, else the displayed project's header.

Dots mark what needs you:

- A bell in a terminal lights its row, and the header of its project while the project is folded.
  A desktop notification from a terminal marks it the same way.
- A thread whose run ends while it is off screen lights its row until you show it. A thread
  waiting for your confirmation also lights the dot on the sidebar toggle and on a folded
  project's header.
- Zed's own desktop notifications for threads still fire.

To close a thread you are done with, point at its row and click Archive at its end, or right-click
the row and choose Archive Thread (#605). The thread is archived as Zed's thread history archives
it, and its row goes. To bring it back, right-click the project's header and choose it under
Archived Threads (#616), or use Zed's archive view in the Agent Panel. To delete a thread for good,
right-click its row and choose Delete Thread…: Marley asks first, then the thread goes from every
Agent Panel, from the rail and from the agent's own sessions where the agent keeps them.

While an agent waits on you, **Needs you** and a count sit between the filter and the projects
(#508), with an entry for each wait, the one that has waited longest first. An entry names the
agent and its project, what it asks, and how long it has waited (`now`, `3 m`, `1 h 5 m`); a click
on it shows where it waits.

- A tool call in a Zed agent thread that waits for your confirmation has Deny and Allow under its
  entry, which answer that one call as the thread's own buttons do. A prompt that offers more than
  allowing or denying once, or a sandbox escalation, has no buttons: its click opens the thread.
- Claude Code in a Marley terminal that waits on a permission or a question is listed with what it
  asks. Its click shows the terminal, where you answer it.
- A Codex on its own App Server (#650) lists each request it asks you to approve (#651): a command
  and its folder, the files a change edits, the permissions it asks for, an MCP server's question.
  Under each are the decisions the request offers: Allow, Allow session, Deny and Stop turn (deny,
  and Codex's turn stops) for a command or a file change; Allow, Allow session and Deny for
  permissions; Deny and Dismiss for an MCP server's question. A decision answers that request
  alone, and only while it still waits; Codex's own prompt in the terminal then closes, and an
  answer you give in the terminal takes the entry away. An allow shows only where Marley can read
  what it allows. A question of Codex's own is listed as "Waits on an answer" and opens the
  terminal.
- An agent's click a Browser tab holds (see "The click consequence") has Refuse and Allow, as the
  tab's card does. Its click brings the tab forward with the focus on the card.

An entry leaves once its wait ends, wherever you answered it, and the section leaves with the
last one. The filter and folded projects never hide it. With the inbox's risk use on, each entry
also says what its action would do, and the entries that matter most come first (see "The inbox
risk" under System One).

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
  addresses of one port gets one row. A local URL reads without `http://` (#618); a line too long
  for the rail loses a URL's middle or a unit's start, and the row's tooltip gives the whole URL.
- One click on a row marks it (#604); a double-click, or Enter once it is marked, opens its URL in
  a Browser tab of the project, as Open does.
- The pointer on a row shows the process's command line, working directory and pid, and three
  buttons: Open (the URL in a Browser tab of the project, or the tab already on it), Copy (the URL
  on the clipboard) and Stop. Stop sends the process SIGTERM once a fresh look finds it still
  listening on that port; otherwise a message says Marley left it alone.
- A server that a systemd service runs (#603) shows its unit on a line under the URL, and the
  pointer says whether it is a user or a system service. Stop then stops the service, since its
  restart policy would start a signalled process again: `systemctl --user stop <unit>` for a user
  service, `systemctl stop <unit>` for a system one, which asks for your password through the
  desktop's dialog. If that fails, a message says why and offers the command to run yourself
  (`sudo systemctl stop <unit>` for a system service) with a Copy Command button. Stop's tooltip
  says which it will do.
- A port a Docker or Podman container publishes (#614) is listed too. Where `docker ps` (or
  `podman ps`) answers, the row is named by its container ("container web") and sits under the
  project its Compose folder is in; otherwise it sits under a CONTAINERS label after the
  projects, with the container's address ("→ 172.18.0.5:80"). Stop stops the container with
  `docker stop`; if the engine refuses (you are not in the `docker` group, say), a message says
  why and offers a command to run yourself. Marley never stops Docker's own service from a row.
  The CONTAINERS header says how many there are and starts folded: click it, or its chevron, to
  show the rows or fold them again, and each window remembers which (#670).
  To leave the CONTAINERS list out, turn off Containers in the Rail on the Settings window's
  Marley page (`"marley": { "rail_containers": false }`, #669); a container whose Compose
  folder is in a project still shows under it.
- Right-click a port row for its menu (#615): Open in a Browser Tab, Copy URL, and for a server a
  systemd service runs, Restart Service, Stop Service and Show Logs. Restart keeps the row while
  the service comes back; its unit line says `starting`, then the row is the new process's. The
  unit's state shows on its line whenever it is not simply running: `failed` in red, `restarting`
  while systemd restarts it on its own, `stopping`, `stopped`. Show Logs opens `journalctl --user
  -u <unit> -f` in a new terminal of the project.
- Marley's own listeners and other users' processes get no row, and neither does a server whose
  working directory is in no project, such as one that moved to `/` when it went to the
  background.

A project whose folder is a git repository's main checkout shows two things at the right of its
header (#531):

- **The lines changed** on its branch, `+12 ‒3`: added and removed since the branch left its base,
  your uncommitted edits to tracked files included (new files git does not track yet are not
  counted). The base is the pull request's base when there is one, otherwise the repository's
  default branch, its `origin` copy when there is one, so on `main` the counts are what you have
  not pushed. The pointer on them names the base. They follow each save within a second or two.
- **The pull request**, when the branch has one on GitHub: its icon and number, green while open,
  gray as a draft, in the accent color once merged, red when closed; the pointer gives its state,
  title and link. Marley asks the GitHub CLI, `gh`, when the branch or its latest commit changes
  and every two minutes while the window is active. Without `gh` on the PATH Marley started with,
  logged out of it, or with a remote that is not GitHub, the row shows the counts alone.

Marley reads neither for a repository Zed does not trust yet.

A project header's right-click menu has Move Project Up and Move Project Down, which move it one
place in the rail, past a group as well as a project.
A thread row's right-click menu also has Delete Thread…, which asks first and deletes the thread
for good (#616), and a project's has Archived Threads: its archived threads, with when each was
last updated; choose one to open it, which brings it back. Ctrl+Alt+J closes
and opens the rail. A rail you close stays closed after a restart, and its width is saved with
the window; one width holds in both layouts.

After a restart Zed reopens only the project the window showed. The window's other projects stay
in the rail, dimmed (#606): point at one to read "Not open. Click to open it.", and click it, or
press Enter on it, to open it with its terminals and tabs. Its right-click menu still moves or
removes it. Next and Previous Project pass over the dimmed ones. Under a dimmed header are the
project's agent threads and the ports listening in its folders (#617), and its chevron folds
them. Click a thread to open the project and the thread; a port row's Open and Show Logs open the
project first, while Copy, Stop and Restart work with the project closed. Next and Previous
Thread pass over a closed project's threads.

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

### Groups with no folder (#600)

A right-click on the rail's empty space (under the last row, or beside PROJECTS, or the Projects and
Brain buttons) opens a menu:
New Group… asks for a name and makes a group with no folder, listed after the window's projects
with a group icon, a chevron and a `+`. A group's terminals and agent CLIs start in the home
folder, and its Browser tabs use a Chromium of the group's own. New Terminal, New Browser Tab and
the agent CLIs in the same menu open in a group named Home, made the first time. A group's `+`
has no New Agent Thread, New Agent in Worktree or Launch, which need a folder. Right-click a
group's header for Rename Group… and Remove Group. Groups come back after a restart with their
names, order, terminals and Browser tabs (#601).

### Drag to reorder (#602)

Drag a project's or a group's header up or down to move it, with its rows, among the headers; a
group can sit between projects. Drag a terminal, Browser tab or thread row up or down to move it
among the rows of its kind in its own group (a worktree's terminals among that worktree's). A
card with the name follows the pointer, and a line shows where it lands: above the target when
you drag up, below it when you drag down. A terminal row dropped on another open project's or
group's header moves to that project, with its shell still running (#613); its right-click
menu's Move to Project does the same. Any other row dropped on another group, a header or
outside the rail goes back. The rail holds still while you drag, and a press that barely moves is still a
click. The order is saved with the window and comes back after a restart; under the attention
order, a row moves within its class and the order you set breaks ties. Something added later goes
after what you placed, and moving a row never moves its tab in the panes.

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

### Shell integration for bash, zsh and fish

Blocks come from the shell, which reports each prompt and each command to the terminal. When
Marley starts an interactive bash, zsh or fish in a local terminal, it loads its integration:

- **bash** starts with `--rcfile ~/.local/share/marley/shell_integration/marley.bash`. The script
  sources your `~/.bashrc` first, then adds the reports (#463).
- **zsh** starts with `ZDOTDIR` pointing at `~/.local/share/marley/shell_integration/zsh`, whose
  `.zshenv` puts your own `ZDOTDIR` back (or leaves it unset if you had none) and runs your
  `.zshenv`, so zsh reads your startup files as it always did (#465). If you have no zsh startup
  files at all, zsh's new-user menu no longer opens.
- **fish** starts with `~/.local/share/marley/shell_integration` first on `XDG_DATA_DIRS`, where
  fish finds `fish/vendor_conf.d/marley.fish`. fish runs that snippet before your `config.fish`;
  it puts your own `XDG_DATA_DIRS` back (or unsets it if you had none) and adds the reports as
  event handlers (#466). fish's history file feeds the ghost text and the prompt editor's
  completions.

A shell with the integration finds `MARLEY_SHELL_INTEGRATION=1` in its environment, and its tab
title leaves out the arguments Marley added. Tasks, remote terminals and a shell you configure
with arguments of its own start without the integration, and so show no blocks. Marley's `ssh`
function, which carries the integration to a host, is bash's and zsh's only: in fish, `ssh`
stays fish's own.

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

With Block Headers on, as it is unless you turn it off (the Marley settings page's Terminal
section, or `"marley": { "block_headers": false }`), each command's prompt rows show Marley's
header instead of your prompt: the command in the terminal's font, with its pill at the right
(#628), and the folder it ran in with the git branch it was on when it started, muted (#630). The
header takes the rows your prompt took, so nothing moves: over a two-line prompt the folder and
branch sit above the command, over a one-line prompt they follow it. The folder reads `~` for
your home, and a long one shows its last two folders after `…`.
The prompt waiting for your next command is drawn as your shell draws it. A click on a header
selects nothing. The block keys bring a block's header to the top row, the command pinned while you
scroll back reads as the header does, and a search skips the text a header hides (#629). A command whose report did not carry the terminal's value (see "Shell
integration" above) keeps its prompt.

A path a command printed, such as `src/main.rs:2:5`, opens with Ctrl+click against the folder that
command ran in, wherever the shell has gone since (#619); hold Ctrl over it to see the whole path.

A failed block whose output names a failing place (rustc and cargo, gcc, clang and go, tsc, or a
Python traceback) carries a Jump to Failure chip, and its right-click menu Jump to First Failure
(#620): the block scrolls to its first error, under its pinned command, and the file opens at that
line and column. `marley: jump to first failure` does it for the newest failed block.

A task you run (`task: spawn`, a runnable's ▶) is a block too (#621): its output framed with the
bar, the wash and the pill (its exit code when it fails), with the block's menu, its copies and
Jump to First Failure, and its state on the terminal's rail row. Point at a finished task's block,
or right-click it, for Rerun Task, which runs the task again in its tab as the tab's own Rerun does
(#622). In the Marley layout the block's pill takes the place of Zed's `Task … finished` line.

A failed block's errors and warnings in the project's files are also the project's diagnostics
(#623): the status bar counts them, the Diagnostics view lists them (marked `marley`), and the
editor underlines them. The next run of the same command in the same folder replaces them, and a
run that succeeds clears them.

### Moving between blocks, copying, rerunning

- Ctrl+Up scrolls to the start of the block above the top of the view, and Ctrl+Down to the next
  one, or back to the live screen from the last. Each press moves one block, however fast you
  press. A binding of your own on those keys wins (#473).
- Point at a block and two buttons show beside its pill (#474). Copy Output puts the block's output
  on the clipboard. Rerun Command, shown while the shell waits at its prompt and only for a
  verified command, clears what you had typed on the line and runs the command again. A click on
  either button starts no selection and never reaches a program that reads the mouse.

### Blocks over ssh

`ssh host` typed in Marley's bash or zsh keeps blocks working on the host (#526): Marley starts the
host's bash or zsh with its integration, carried in the ssh command, so each command you type
there is a block with its exit code and its Copy Output. Nothing is installed on the host; the
temporary folder the integration is written to is removed as soon as the shell has read it. The
`ssh` block itself ends when the host's shell starts.

- Rerun is offered for a block only while the shell that ran it waits at its prompt: at the host's
  prompt for the host's commands, back at yours after `exit` for your own.
- Autosuggestions at the host's prompt come from the commands you ran on the host.
- `ssh` runs as plain ssh with a remote command (`ssh host uptime`), with `-N`, `-T`, `-W`, `-f`
  and the like, when its input or output is not a terminal, for a host whose ssh config sets
  `RemoteCommand` or `Tag marley-plain`, and as `command ssh`. A host whose shell is neither bash nor
  zsh gets its login shell as usual.

### Remote terminals (#543, #641)

`marley: open remote terminal` lists the SSH hosts saved in your settings (`ssh_connections`) and
opens a terminal on the one you pick, in a tmux session of Marley's own on the host, so a shell or
an agent there keeps running when the link drops. Every ssh Marley starts, this one included,
checks its link every 5 seconds, so a link that went silent is known dead within 20 seconds rather
than when the network finally gives up.

- When the link dies, the terminal keeps its last screen, dimmed, under a line that names the host,
  says what ssh said, and counts down to Marley's next check of the host. Keys, pastes and Marley's
  own sends are not sent while it is dimmed; the line counts them, and nothing is typed later.
- Marley checks the host 1 second after the drop, then twice as long after each failed check, up
  to every two minutes. A link that stays up a minute starts the next drop's checks from 1 second
  again.
- When the host answers, Marley runs the terminal again in the same tab, attached to the same tmux
  session, without moving the focus or changing the tab you are on.
- `terminal: rerun task` tries at once. If the host refuses the login (a changed key, a host key
  that no longer matches), Marley stops checking and the line says why; Rerun logs in by hand.
- `exit` in the session, or anything else that ends ssh normally, ends the tab as before.

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
  microphone where Voice is on and Voxtype is installed, and, for Claude Code, the "Connect Claude Code to Marley"
  chip until Marley's plugin is installed, and a version chip while something is off (below).
- At the right: the folder the agent works in (`~` for your home) and its git branch, from the
  innermost repository of the project that holds the folder.

The rich input's editor opens above the bar.

**The agent's version** (#648). Marley runs your own `claude` and `codex`: the one
`MARLEY_CLAUDE` or `MARLEY_CODEX` names, else the first on Marley's PATH. It reads each one's
version with `--version` when Marley starts, and again when an agent bar for it shows and the
program's file changed (an update moves Claude Code's link). One part of Marley rests on something
Claude Code does not document: the tags that tell your prompts from the ones Claude Code adds
itself, task notifications and system reminders. Marley checked them on Claude Code 2.1.283 and
later 2.1 releases. On another version, or one it cannot read, it keeps them off and reads every
prompt as yours, so the rail may show a task notification as your prompt, and the agent bar shows
a chip: "Untested Claude Code 2.2.0", "Claude Code version unknown" or "Claude Code not on
Marley's PATH". Its tooltip says what is off, the program and version found, and the versions
checked; a click opens the Settings window at Prompt Tags on Untested Claude Code, which turns
them on anyway. A remote terminal's Claude Code is the host's, so its prompts are read with the
tags and no chip shows.

### Rich input

While an agent CLI runs, Ctrl+G or the pencil opens a Zed editor above the bar for the agent's
prompt, one to eight lines high (#481). Select with the mouse, undo, move by word, and press
Shift+Enter for a new line. Enter sends the text to the agent as one paste (bracketed when the
agent asked for bracketed paste) followed by a carriage return, and closes the editor. Escape
closes it and keeps the draft for the next Ctrl+G.

**Agent prompts in a tab** (#649), Agent Prompts in a Tab in the Marley settings, off by default.
On, the terminals Marley opens for agents (the New Agent picker, a project's `+`, a launch
configuration's agent) give the agent Marley's editor, `marley-edit`, as `VISUAL` and `EDITOR`,
over what your `.bashrc` exports. There Ctrl+G and the pencil send the agent its own editor key
(Ctrl+G for Claude Code, Codex and Gemini CLI, Ctrl+X then E for OpenCode): the agent writes its
draft to a file and Marley opens the file in a tab, with the focus. Edit it as any file; saving
does not hand it back, closing the tab does, and the agent's terminal comes back to the front with
the draft in its prompt, nothing sent until you press Enter. Other programs the agent runs there
get the same editor (a `git commit` with no message opens a tab too). A New Terminal, an agent run
there by hand, a remote project, a terminal a restart brings back, and every terminal while the
setting is off keep the Rich Input editor above. The setting applies to agent terminals opened
after it changes.

At a shell's prompt with no agent running, Ctrl+G opens the same editor for the shell (#624),
holding what you had typed at the prompt. Enter clears the shell's line and runs the editor's
command as a block; Escape closes the editor and leaves the shell's line as it was. Anywhere else
(a full-screen program, a command running), Ctrl+G reaches the program as it always did.

In the shell's editor, Tab opens completions (#625): the word before the cursor as a path in the
prompt's folder (folders end in `/`), and the command so far as the start of one from the shell's
history or the project's tasks. Enter or Tab takes an entry without running the line.

The shell's editor needs no Ctrl+G (#627): in the terminal you are typing in, it docks at every
prompt and takes the keys, so you write each command in a Zed editor. When a command starts or a
full-screen program such as vim shows, the editor closes and the program gets every key raw; at
the next prompt it comes back, empty. Escape gives the keys to the shell until its next prompt, and
Ctrl+C empties the editor, as it drops a line at the shell's prompt. To have the editor only on
Ctrl+G, turn off Prompt Editor in the Settings window's Layout section
(`"marley": { "prompt_editor": false }`).

A line that reads as a request in words, such as `what is using port 3000` or `kill the dev
server`, shows `· ctrl-shift-enter asks the agent` after it, in the editor or at the shell's own
prompt (#557). Ctrl+Shift+Enter hands it to the window's agent, a picker when there are several,
or a new Claude Code when there is none; on a command the key goes to the shell. Marley's own
rules decide, and nothing leaves the machine unless the System One layer's typed line is on
([The typed line](#the-typed-line)). English at the Prompt on the settings page turns the hint
off.

What you type at a shell prompt is drawn in your theme's syntax colours as you type, as Zed colours
a shell script: commands, strings, variables and operators apart (#626). The shell's editor reads
the same way. Once the command runs, its block keeps the shell's own drawing.

### Attach File

The bar's `+`, or `marley: attach file` for the focused terminal, opens a file chooser (#479).
The files you pick, several at once if you like, go into the terminal as their full paths, quoted
where the shell needs it, the way dropping them on the terminal types them. Claude Code and the
other agents read a file from its path. In a remote project the chooser lists the machine the
terminals run on. The command works in any focused terminal, agent or not.

### The microphone

Dictation is off until you turn it on: Voice in the Voice section of the Marley settings page, or
`"marley": { "voice": { "enabled": true } }` (#642). Off, Marley shows no microphone and starts no
`voxtype`, and `marley: toggle dictation` says dictation is off and where to turn it on. Turning it
on or off takes effect at once.

With Voice on, where Voxtype, the dictation daemon Omarchy ships, is on the PATH, the agent bar has
a microphone (#480). Click it, or run `marley: toggle dictation`, to start or stop a dictation. The microphone
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

### Permission modes

Marley starts each agent with its own permission prompts unless you ask for otherwise (#532):

| Setting | Values | What Marley types |
|---|---|---|
| `marley.claude_code_permissions` | `"ask"` (the default), `"bypass"` | `claude`, or `claude --dangerously-skip-permissions` |
| `marley.codex_permissions` | `"ask"` (the default), `"full_access"` | `codex`, or `codex --sandbox danger-full-access --ask-for-approval never` |

Both sit in the Marley page's Agents section. `marley.agent_permissions_by_project` sets either
for a project, by its folder:

```json
"agent_permissions_by_project": {
  "~/scratch": { "claude_code": "bypass", "codex": "full_access" }
}
```

An entry applies to a local project whose main folder is its folder or inside it, and the
longest folder wins; a remote project takes the defaults. Only your own settings can set these:
a repository's `.zed/settings.json` that asks for bypass changes nothing.

An agent row that runs without its prompts carries a chip at its end, `bypass` or `full access`,
in the warning color, however the agent was started: from the `+`, a task, or typed by hand. Its
tooltip says where Marley read it. For Claude Code with the plugin connected, the permission mode
its events report decides, so a bypass Claude Code's own settings chose shows, and one you left
with Shift+Tab goes at its next event. Otherwise the arguments it was started with decide:
`--dangerously-skip-permissions` or `--permission-mode bypassPermissions` for Claude Code, and
for Codex `--sandbox danger-full-access` (or `-s`), a `--config` of `sandbox_mode` to
`danger-full-access`, or `--dangerously-bypass-approvals-and-sandbox`. A full access Codex's own
`config.toml` sets shows no chip: Codex tells Marley nothing but its arguments, unless it runs
against its own App Server, below.

**Codex's own App Server** (#650), Codex App Server in the Marley settings, off by default. On,
each Codex Marley launches in a local project (the `+`, New Agent, a worktree agent, a launch
config) runs against an App Server of its own: Marley starts `codex app-server` for the terminal,
types `codex --remote unix://… --cd <folder>` once it is up, and joins it as a second client once
Codex has. Codex's row then reads the thread's own state, `working`, `waiting on approval`,
`waiting on input`, `idle`, or `failed` with the turn's error on the next line, and its token use,
`working · 12k tokens`, however long the terminal stays quiet. A waiting Codex enters the inbox as
Codex's, and a click shows its terminal, where you answer it. The close guard asks before a
working or waiting Codex closes. The `full access` chip follows the sandbox the thread reports,
whatever the arguments said, and its tooltip says so. The server's commands run with the
environment the terminal's programs get, and the server stops with its terminal or with Marley.
It runs on Codex 0.155.1 to 0.158.0; on another version Codex starts as before, and the agent
bar's chip says which version Marley found (App Server on Untested Codex, in the Agent Versions
section, runs it anyway). Marley answers nothing Codex asks. While it is joined, the user agent
Codex sends with its requests names Marley. A Codex you type by hand runs as you typed it.

The first time Claude Code starts in bypass it asks you to accept its warning, in the terminal.

### Launch configs

A project can name sets of things to open together in `.zed/marley.json` at its root (#527). Its
`+` then lists them under **Launch**:

```json
{
  "launch": {
    "Dev": {
      "items": [
        { "terminal": "npm run dev", "title": "dev server", "cwd": "web" },
        { "agent": "claude", "split": "right", "focus": true },
        { "browser": "http://localhost:5173", "split": "down" }
      ]
    }
  }
}
```

- An item is one of `terminal` (the command typed once the shell is ready; `""` is a plain
  shell), `agent` (`claude`, `codex`, `gemini` or `opencode`, started as the Agent CLIs entries
  start it) and `browser` (an `http` or `https` URL). A URL on this machine waits up to 30 seconds
  for its port, so the dev server an earlier item starts has time to come up.
- `title` names a terminal's or an agent's tab, `cwd` is a folder inside the project, `split`
  (`right` or `down`) puts the item in a new pane off the previous item's, and `focus` gives it the
  focus at the end (otherwise the first item has it).
- The first time you choose a config, Marley shows what it will run, one line an item, with Run
  and Cancel. It remembers the exact text you approved, so the same config opens at once next
  time; when the file changes it, Marley asks again and says it changed.
- A file Marley cannot read shows one grayed entry with the reason, and the log has the whole of
  it. An edit shows the next time you open the menu.

### Worktree agents

Several agents can work on one repository at once, each in a git worktree and on a branch of its
own (#510). In a project's `+`, New Agent in Worktree lists the installed agent CLIs; it shows for
a local project whose folder is a git repository.

1. Choose an agent. A prompt opens with a line under it that names what it will make:
   `agent/<name> from main`, the name one Zed makes, the base the main checkout's branch (its
   commit when it is detached).
2. Type the agent's first prompt, or nothing (Shift-Enter adds a line, Escape closes it), and
   press Enter.
3. Marley makes the worktree through Zed's worktree service, where Zed puts its own
   (`git.worktree_directory`, `<parent>/worktrees/<project>/<name>/<project>` by default), on the
   new branch, which tracks nothing. Zed carries the folder's trust over and runs the repository's
   `create_worktree` tasks, as for its own worktrees. Marley writes the base as
   `branch.agent/<name>.base` in the repository's config, for review and merge later, and starts
   the agent in a terminal of the worktree's workspace with the prompt on its command line and
   the project's permission mode. The workspace opens beside the project's, and you stay where
   you are.

The rail lists each linked worktree of a project's repository as a row under the project, after
the main checkout's terminals: its name, its branch, and while its workspace is open, its
terminals under it. A click shows an open worktree, or opens one that is not. Left on a worktree's
terminal selects the worktree's row. Worktrees under the main checkout's `.claude/worktrees/`,
which Claude Code makes for itself, get no row; their terminals list under the project.

A create that fails, such as for a folder Marley cannot write, says so in Zed's toast and leaves
no row and no agent.

A new worktree is a clean checkout: none of the main checkout's gitignored files (`.env`, local
config) and no dependencies (#585).

- **`.worktreeinclude`** at the main checkout's root names gitignored files to copy into every new
  worktree before its agent starts, in `.gitignore` syntax, as Claude Code reads the same file for
  its own worktrees. Only files git ignores are copied, so a tracked file never is. A pattern that
  starts with `**/`, or has no slash, reaches into a folder git ignores as a whole only when the
  folder matches or the first name after `**/` is one of the folder's names; to copy out of such a
  folder, name it: `vendor/**/config.json`. Nothing in the worktree is overwritten. The copy stays
  within 100 MB and 10,000 files; an entry that would pass that is left out whole, and a toast
  names it.
- **The setup command.** When the repository's root has a `package.json` and the lockfile of one
  package manager (`pnpm-lock.yaml`, `bun.lock` or `bun.lockb`, `yarn.lock`, `package-lock.json`)
  and no `create_worktree` task, the prompt offers the install command, as in
  `Setup: pnpm install (pnpm-lock.yaml found)`, its box clear. Checked, the agent's terminal runs
  `pnpm install && claude …`, so the agent starts once the install succeeds. Marley keeps your
  choice in `git config marley.worktreeSetup` (the command, or `none`), and the next prompt starts
  checked when it holds the command offered. Zed's own `create_worktree` tasks in
  `.zed/tasks.json` start with the worktree, alongside the copy; a step that needs a copied file is
  safer as the setup command.
- **Paths in tasks.** Tasks in a linked worktree, `create_worktree` tasks among them, get
  `MARLEY_ROOT_PATH` (the main checkout) and `MARLEY_WORKTREE_PATH` (the worktree) beside Zed's
  `ZED_MAIN_GIT_WORKTREE` and `ZED_WORKTREE_ROOT`, so a setup script written for Orca's
  `ORCA_ROOT_PATH` works after one rename.
- **A port for each worktree** (#590). Each worktree New Agent in Worktree makes gets a slot, the
  lowest from 1 that no other worktree of the repository holds, and its terminals and tasks start
  with `MARLEY_PORT_OFFSET` (the slot times ten) and `PORT` (3000 plus that): the first worktree's
  dev server gets 3010, the second's 3020, each with ten ports to itself. A server that reads
  `PORT`, or a script that adds `MARLEY_PORT_OFFSET` to its own ports, stays out of the others'
  way; one that picks its own port does as before, and Marley's localhost links still find it.
  The main checkout gets neither. A `PORT` in your `terminal.env` setting or in a task's `env`
  wins. The slot is kept in `git config branch.<branch>.marleySlot`; a removed worktree's slot
  goes to the next one made.

Claude Code keys its folder trust on the repository's main checkout, so a worktree of a
repository you have trusted in Claude Code starts without asking. For a repository you have not,
Claude Code asks whether to trust the folder before it runs the first prompt (#587). The
worktree's workspace opens behind the one you are on, so Marley brings the question to you: a
notification in every workspace names the worktree and the folder, shows the question's
warnings, such as a folder that pre-approves tool permissions, and offers two buttons.

- **Trust Folder** answers yes: Marley moves Claude Code's focus to "Yes, I trust this folder",
  wherever it was, and confirms it. Since Claude Code 2.1.263 the focus starts on "No, exit".
- **Show Terminal** shows the worktree's workspace with the agent's terminal, to read the
  question whole and answer it there.

The notification goes once the question does, however you answered it. If Marley's answer did
not take, a second notification says to answer it in the terminal, with Show Terminal. To have
Marley answer by itself when Zed trusts the worktree's folder, set
`"marley": { "claude_code_worktree_trust": "follow_zed" }` (Worktree Trust Question on the Marley
settings page); Marley then shows a note that it did, which stays until you close it. It answers
only while Zed trusts the folder, and never writes Claude Code's own record of trusted folders.

A worktree's row shows its drift from its base (#560), so two agents about to collide show it
before anyone merges:

- `2 behind`, muted: the base has two commits the branch does not, and a merge would go through.
- `1 conflict` (`3 conflicts`), in the warning color: a merge would stop on that many files.
- No chip: the branch has everything its base has, or has no commits of its own, such as a
  branch already merged.

The pointer on the chip shows the rest: `2 commits behind main (main at 3f2a1c9)`, then the files
a merge would stop on, twenty at most. The base is the branch the worktree started from, which
Marley records as `branch.<branch>.base`; a worktree Marley did not make takes the repository's
default branch, when it is a local branch, and gets no chip without one. Marley reads the drift
from git in the main checkout a second after the branch or its base moves, against the local base
(it never fetches), and only while the rail shows and the repository is trusted. The reads change
no branch, index or file. A git older than 2.38 cannot say whether a merge would stop; its chip
shows the commits behind alone.

While the branch has commits its base lacks, the row's second line counts them:
`agent/ok · 2 ahead of main`. A right-click on the row opens its menu (#511):

- **Review** opens Zed's branch diff of the worktree against its base, "Changes since main", in
  the worktree's own workspace. A worktree that is not open opens with the diff alone, without
  the terminal a newly opened folder gets. A worktree Marley did not make compares with the
  repository's default branch. Review comments work there as in any branch diff.
- **Merge 2 commits into main…** merges the branch into its base in the main checkout with a
  merge commit, and pushes nothing. Marley first checks, in order, that the main checkout is on
  the base, that it has no changes not committed (on disk, or unsaved in Marley), that the
  worktree has none, and that there is something to merge; a failed check is named and nothing
  changes. Then it asks, naming the count, the base and the main checkout's folder. The merge
  commit reads "Merge branch 'agent/ok' into main", and the toast names it. A merge that stops on
  a conflict is aborted, so the main checkout is as it was, and the files are named. The
  worktree and its branch stay.
- **Remove…** (#589) takes the worktree away. Marley asks first, and when git counts changes not
  committed, untracked files included, the question names how many and the button reads Remove
  Anyway: those changes are deleted with the folder. Then the worktree's workspace closes, its
  terminals with it (Marley asks about unsaved files, and Cancel there stops Remove), and Zed's
  own code removes the worktree, which it does only for a worktree Zed made that a project has
  open. The branch goes when git agrees it is merged, or when Marley finds its commits in the
  recorded base, `origin/HEAD` or the main checkout's branch, a squash merge included; otherwise
  it stays and the toast says why, so committed work is never lost. In a repository the Rustal
  workflow merges, Remove waits until the branch is merged.
- **A teardown task** (#591). A task in `.zed/tasks.json` whose `hooks` hold `remove_worktree`
  runs when you confirm Remove, before the worktree goes, in a terminal of its own with the
  worktree's paths in its environment, for example `"command": "docker compose down"`. Marley
  waits up to two minutes for each such task. If one fails or takes longer, Marley asks: Cancel
  keeps the worktree and the task's terminal, so you can read what it printed; Remove Anyway goes
  on.

Merge shows only for a branch whose base Marley recorded, in a repository Zed trusts. Otherwise
the menu says why: No base recorded, Nothing to merge into main, or, where the Rustal workflow
manages the repository, `2 commits ahead of main: the Rustal workflow merges here`. Marley takes a
repository as the workflow's when `git config marley.merge` is `workflow`, or when it is unset
and the main checkout's root holds a `workflow.toml` with a `[project]` table, which `rw init`
writes; `git config marley.merge marley` gives the merge back to Marley.

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

The plugin, `marley` 1.4.0, brings three things:

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
- **The rail's rows.** Its event hook writes a short summary of each of Claude Code's hook events
  to the terminal, and the rail's row shows the prompt, the tool in flight, what it waits on and
  the last message (#519). Since 1.3.0 a message over 300 characters keeps its start and its last
  whole sentences, where a question or a status sits (#566). Since 1.4.0 an agent's question
  carries its options, which the inbox's question route reads (#570). The agent bar offers the
  update to a plugin that is older than Marley's.

### Claude Code's IDE link (#653)

Claude Code links to an IDE through a small server the IDE runs and a lock file in Claude Code's
`ide` folder. With **Claude Code IDE Link** on (the Marley page's Agents section, off by default),
Marley is that IDE for each local project:

- It writes `<port>.lock` into `~/.claude/ide/`, or `$CLAUDE_CONFIG_DIR/ide/` when Marley was
  started with that variable, naming Marley and the project's folders, and removes it when the
  project closes, the switch turns off or Marley quits. It also removes a lock file an earlier
  Marley left behind, and touches no other IDE's.
- Each new terminal of the project gets `CLAUDE_CODE_SSE_PORT` and
  `CLAUDE_CODE_AUTO_CONNECT_IDE=true`, so a `claude` started there links to this project's Marley
  at its start. Terminals opened before the switch turned on, tasks and remote terminals get
  neither. From any other terminal, `/ide` lists the Marley projects whose folders hold the
  current folder, and `claude --ide` links when there is one.
- Claude Code then gets, with each prompt, the selection and the file of the last file editor you
  used (a terminal taking the focus leaves it as it was), the language servers' diagnostics
  through its `getDiagnostics` tool, and send selection (Ctrl+>) mentions the lines in its prompt
  instead of typing `@path#La-b`. Edits still show in Claude Code's terminal; diffs in Marley come
  later.

Claude Code's docs name the lock file, the token and `getDiagnostics`, but not the rest of the
link, so the connection, the selection and the mention are three rows of Marley's version table
(#648). No Claude Code release has been checked on them yet, so the agent bar shows Untested
Claude Code while the switch is on, and **IDE Link**, **IDE Selection** and **IDE Mentions on
Untested Claude Code** under Agent Versions turn each on anyway.

Any program of yours that can read the lock file can connect and read the selection, as with
Claude Code's own editor extensions. If Claude Code does not link: a `CLAUDE_CONFIG_DIR` your
shell profile sets for terminals but not for Marley sends it to another folder, and an
`http_proxy` without `127.0.0.1` in `no_proxy` can route the local connection through the proxy.

### An agent's own reports (#652)

Every local terminal Marley opens names a program in `MARLEY_BIN`. An agent, or a plugin or
wrapper around one, tells Marley its state through it, with the arguments rustal-harness's
`rh report` and `rh release` take:

```
"$MARLEY_BIN" report working --source mod:claude-code --seq 7 --activity "Running the tests"
"$MARLEY_BIN" release --source mod:claude-code
```

The states are `starting`, `working`, `idle`, `waiting` (with `--prompt` and one or more
`--option`), `error` and `done`, and a report can carry `--percent`, `--session-id`, token counts
and quota windows. `seq` must grow with each report. The process that sends the first report
holds the terminal: a report from any other process there, such as a `claude -p` the session
starts, is refused until the holder releases or ends. While a terminal holds a report, its row
shows the reported state, with the prompt and the tool in flight still read from the plugin's
hook events, and a restart resumes the session the report named, in the agent's folder.

The program exits 0 when Marley took the report. Otherwise it exits 1 and prints why, such as
`agent_report_stale: seq 6 is not past 7`, or `marley_not_running` when no Marley answers.
Tasks, remote terminals and terminals of remote projects get `MARLEY_BIN` empty. Nothing reports
by itself yet: the shared Claude Code plugin that will is the next step.

### Per-turn diffs

With the plugin connected, Marley keeps each turn of a terminal's Claude Code that changed the
repository's tree (#509). A turn runs from a prompt to its stop: Claude Code's `Stop`, an API
error, a manual `/compact`, an interrupt, the next prompt, a new session, or Claude Code leaving
the terminal. At the prompt and at the turn's end Marley takes a checkpoint of the innermost
repository that holds Claude Code's directory, as Zed's agent does, through a temporary index. A
turn whose tree changed becomes a commit of its end on its start, pinned under
`refs/marley/turns/<session>/<n>` with Marley as its author; your index, your branches and `HEAD`
are never touched, and a turn that changed nothing leaves nothing.

Under the terminal's rail row, **Turns (N)** and its chevron open the turns of the terminal's
Claude Code, newest first:

| A turn row | What it says |
|---|---|
| The title | The prompt on one line; a slash command's name, such as `/review`; what a harness injected, such as `task notification` |
| `· 2 files` | How many files the turn changed |
| `· failed` | The turn ended in an API error |
| `· injected` | A harness's prompt started it, not yours |

A click opens the turn in Zed's commit view in the terminal's project: the files the turn changed
against its start, shell commands' edits as much as the agent's, and nothing of the turns before
or after it. The turns stay listed after Claude Code exits, until the terminal closes; the refs
keep the commits after that. A session's first turn deletes the repository's turn refs whose
commits are older than 30 days.

Worth knowing:

- The checkpoint leaves out what Zed's checkpoints do: ignored files, untracked files of 2 MB or
  more, and untracked binaries, archives and media.
- The turn is the tree's change, whoever made it: your own edits while a turn runs, or another
  agent's in the same checkout, land in it.
- A remote project's terminals get no turns.
- Marley runs the `git` on its PATH for the turn's commit and its refs.

### Review notes to the agent

In a project diff or a branch diff (`git: diff`, or Review on a worktree's row), hover a changed
line and choose Add Review in the gutter, type a note and press Enter; notes on several lines and
files add up. **Send Review to Agent (N)** in the diff's toolbar, or `editor: send review to
agent` from the palette, opens a picker of the agent terminals whose folder holds every noted
file (#522):

| The row says | What happens when you pick it |
|---|---|
| `ready` | Claude Code idles at its prompt: Marley pastes the notes as one prompt, presses Enter and shows the terminal |
| `working` | Nothing is sent: the agent is in a turn |
| `asking for permission` | Nothing is sent: a paste would answer its question |
| `no idle signal` | Nothing is sent: Marley cannot tell (another agent, or Claude Code without Marley's plugin) |
| Copy notes | The same prompt goes on the clipboard, for any agent |

The prompt gives each note as `File:` (relative to the agent's folder), `Line:` or `Lines:`, and
`User comment: "…"`. Sent notes stay in the diff with Sent beside them, and the button counts only
the ones not sent; Copy notes marks nothing sent.

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

With Rusty on and Rusty Tools for Agents on (both in the Marley settings page's Rusty section,
or `"marley": { "rusty": { "enabled": true, "agent_tools": true } }`; both off until you turn them
on, #642, #643) and `rusty-mcp` found (as the Rusty section below finds it), Marley also offers
Rusty's MCP server to the same agents as the context server `rusty` (#633): a stdio `rusty-mcp`, as
Rusty's own `.mcp.json` names it, so an agent reaches Rusty's brain loop (`brain_ask`,
`brain_decide` and the rest) and its other tools. Zed asks before each call, as it does for every
server. Turning either setting off takes the server away while Marley runs, and a `rusty` entry of
your own in `context_servers` replaces Marley's. Without `rusty-mcp`, nothing is added. An older
`"rusty_tools": true` is carried into the block: Zed reads it as Rusty and its tools on, and offers
to update the file.

## Rusty

Marley can connect to Rusty, Ignibyte's local assistant store, through its MCP server, `rusty-mcp`
(#643). It is off until you turn it on: Rusty in the Marley settings page's Rusty section, or
`"marley": { "rusty": { "enabled": true } }`. Off, Marley starts no `rusty-mcp` and opens no
connection, and shows nothing of Rusty but the switch (#661): the command palette lists no
`rusty:` command and no `marley: toggle brain view`, and the settings page's Rusty section holds
the switch alone. A key bound to a Rusty command says Rusty is off.

- **Connection.** `embedded` (the default) starts `rusty-mcp` on stdio, from `MARLEY_RUSTY_MCP`
  when it is set, else your PATH, and ends it when Rusty turns off or Marley quits. `service`
  connects to Rusty's running service at the Service URL (`http://127.0.0.1:4174/mcp` by default)
  and starts nothing; Marley reaches a service on this machine only, since Rusty's server has no
  login.
- **Rusty's Server.** The section's Rusty's Server page shows whether Marley is connected, to which
  server and through what, or why not. Marley checks the connection every 5 seconds and connects
  again after 1, 2, 4 and so on up to 60 seconds when it is lost.
- **Rusty's settings.** The same page shows Rusty's own settings, read from Rusty. Marley keeps no
  copy, so `settings.json` never holds them. An embedded Rusty tells Marley when a setting changes
  elsewhere, and the page follows; a service is read again when Marley connects or writes.
  - **The embedding status** (#666): one line on how far Rusty's brain search has pages embedded,
    with which provider and model, and how many pages wait; or that brain search is full-text only.
  - **The ten settings Rusty's app lists** (#666): `brain_vault_path`, `notes_path`,
    `embedding_provider`, `embedding_model`, `ollama_url`, `pin_timeout_minutes`, `skills_enabled`,
    `skills_path`, `brain_auto_enrich` and `default_workflow`, each with Rusty's words and its
    default, which shows in an empty field. Embedding Provider is a menu (Auto, Ollama, OpenAI or
    Off); the rest are fields: type and press Enter to write a changed value.
  - **Other Stored Keys** (#666): any other key Rusty stores. A key, token, secret or password
    shows as an empty field, "hidden; type a new value to replace it": Enter with nothing typed
    writes nothing, and a new value replaces the old without either showing.
  - **Adding a key** (#666): a key and a value, then Set or Enter.
  - Rusty's refusal of a write shows under the settings.
- With Rusty on, `rusty-mcp` runs twice when Rusty Tools for Agents is on too: once for Marley and
  once for Zed's agents.

### The Brain view

While Rusty is connected, the rail's header shows Projects and Brain (#644). Brain, or Ctrl+Alt+V
from anywhere in the window, swaps the rail's rows for Rusty's vault; the same key goes back. With
Rusty off or not connected, the key says why in a toast. The rail starts on Projects, and goes back
to it when the connection is lost; Brain shows again once Rusty connects.

- **Today** (the notepad under the header) opens today's daily note, which Rusty makes when it is
  missing.
- **Search.** Type in the field and press Enter: Rusty's brain search lists its hits, title and
  slug, in the tree's place. Rusty's `tag:`, `path:`, `file:` and `type:` work as typed. The query
  is sent on Enter only, since with an embedding provider set Rusty embeds every query it gets.
  Up and Down reach the hits from the field, and Enter on the same query opens the selected one.
  Escape clears the search; a second Escape goes back to the tree.
- **The tree.** The vault's folders and pages as Rusty lists them, folders first, each folder
  with its page count. All folders start closed. A click on a folder opens or closes it; one
  click on a page opens it in a preview tab, two keep the tab, as Zed's project panel opens files.
  A page opens in a Page tab (below); a tab already showing it comes forward. With the focus in
  the tree, Up, Down, Home and End
  move, Right opens a folder or steps into it, Left closes it or climbs, and Enter does what a
  click does.
- **Changing the vault.** Right-click a folder for New Page, New Folder, Rename and Delete; a page
  for Open, Rename and Delete; the tree's empty space for New Page, New Folder and Refresh. A new
  name is typed in the tree: Enter makes it (an empty page name lets Rusty call it Untitled),
  Escape drops it, and clicking away keeps a typed name. The header's `+` makes a page in the
  selected row's folder. Drag a row onto a folder to move it there, onto a page to move it into
  that page's folder, or onto the empty space to move it to the top. Delete asks first; Rusty
  moves the page or folder into `archive/` and leaves the links to it. Everything goes through
  Rusty's tools, which rewrite links on a rename or a move; Marley writes no file of the vault. A
  refusal shows Rusty's message in a toast, such as "Could not rename delta: Already exists:
  notes/alpha".
- **Live.** On the embedded connection, a change Rusty makes or sees in the vault's folder shows
  with no click. On the service connection the tree is read again after Marley's own changes and
  on Refresh.
- While Brain shows, a dot on Projects says something in the Projects view needs you.
- **Favourites** (#662). Rusty's bookmarks, listed above the tree in Rusty's order whenever it
  holds one. A page opens as a tree row does; a folder opens in the tree and is selected; a search
  puts its query in the field and lists its hits; a heading opens its page with that heading at
  the top. Right-click one for Rename… (typed in place, Enter keeps it) and Remove. A page's star
  adds it here (below).

### A brain page in a tab

A page opens in a center tab (#645), from the Brain view or with the `rusty::OpenPage` action,
which a key of your own can bind with a slug (`["rusty::OpenPage", {"slug": "projects/marley"}]`;
`"preview": true` opens it in the preview tab).

- **What it shows.** The page's title, its properties (each frontmatter key and its value, a list
  as chips) and its body, drawn by Zed's own Markdown renderer: headings, lists, task boxes as
  written, code highlighted for its language, tables. The header carries Back, Forward, the
  Outline button (#656), the page's folder and name, the star (#662), and Edit.
- **The star** (#662). Filled when the page is one of Rusty's bookmarks. A click, or Ctrl+D with
  the focus in the tab, adds the page to the Brain view's Favourites or removes it; the change is
  Rusty's, in its vault, so Rusty's own app shows it too. A change made in Rusty's app fills or
  empties the star with no click, on the embedded connection.
- **Links.** A wikilink is a link. One to a page opens it in the same tab, at its heading when it
  names one (`[[page#Heading]]`); one to no page yet is drawn muted, and a click makes the page at
  that path, its folders too, and shows it (#654); Rusty's refusal, for a path under `archive/`
  say, shows in a toast. A web link opens in your browser.
- **Back and Forward.** The tab keeps its own history: the buttons, or Alt+Left and Alt+Right with
  the focus in the tab.
- **Preview tabs.** One click in the tree opens the page in the pane's preview tab (its title in
  italics), which the next one-click page replaces; a double-click, or an edit, keeps the tab.
  With `preview_tabs.enabled` off, every page opens kept.
- **Edit and Read.** Edit shows the page's file in an editor inside the same tab: the unsaved dot,
  Ctrl+S and the close prompt work as for any file. Read, Back and Forward save unsaved edits
  first, without formatting, then show the page as saved. Rusty's watcher sees the save and
  indexes it; Rusty commits it on its own since its TICKET-043, and with older Rusty binaries the
  commit comes with Rusty's next write.
- **Live.** While a tab shows a page, a change Rusty announces (one Rusty makes, or one it sees in
  the vault's folder) shows with no click, on the embedded connection.
- **Not connected.** With Rusty off or its connection lost, the tab keeps the page it shows and
  says so above it; nothing on it opens an editor.
- **The outline** (#656). In Read, a column beside the body lists the page's headings in order,
  indented by level, each as drawn (`[[people/sam|Sam]]` reads Sam). A click brings that heading to
  the top of the body, the second of two same-named headings included. The Outline button, or
  `rusty: toggle page outline`, hides and shows the column; a page with no headings has no column
  and the button is dimmed. In Edit the tab hands its editor to Zed, so Zed's outline panel lists
  the file's headings (and its frontmatter keys) as for any Markdown file.
- **Editing in place** (#656). In Read, with Rusty connected, a click on the title, the page's name
  in the header or a property's value opens a one-line editor in its place, all of its text
  selected. Enter keeps it, a click elsewhere keeps it too, and Escape drops it.
  - The title sets the page's `title` property; the file keeps its name.
  - The name renames the page in its folder (a `/` becomes `-`). Rusty rewrites the links to it and
    a toast says in how many pages. A name that is taken shows Rusty's refusal and the old name
    stays.
  - A value edits by its kind. Text takes any text; a number only a number, which Rusty stores as a
    number; a date only `YYYY-MM-DD`. A refused value keeps the editor open with the form it needs
    under it. A checkbox changes with a click. A list adds an item with + (the field stays open for
    the next) and drops one with its ×. A value that is an object, or a list holding anything but
    text, shows with no editor.
  - Each row's × removes the property. Add property, under the rows, asks for the kind (Text,
    List, Number, Checkbox or Date), then the key; a key the page has is refused. The new property
    starts empty: no text, an empty list, 0, unchecked, or today.
  - Each change goes to Rusty in order, one at a time (`brain_set_property`,
    `brain_remove_property`, `brain_rename`), and Rusty commits each. After the last, the tab reads
    the page again and shows what Rusty holds, a refused change included.
  - A rename or a move, from a tab or from the Brain view, moves every Page tab that shows the page,
    or has it in its history, along with it.

### Open a page by name (#654)

`rusty: open page`, or Ctrl+Alt+U from anywhere in the window (a terminal included, where Marley
takes the key from the program), opens a picker over every page of the brain.

- **The list.** Each row shows the page's title and its slug. With nothing typed, your favourite
  pages come first, each with a star, in the Brain view's order (#662); then a line, the pages you
  opened most recently, newest first; then a line and the rest in the order Rusty lists them, the
  most recently changed first. When a page's tab is in front, that page heads the list and the
  selection starts on the row after it; with no favourite that row is the page you had open
  before, so Enter goes back to it. Marley keeps the last 20 pages opened, from the picker, the
  Brain view, links, Back and Forward, in its own database, so the order holds after a restart.
- **Typing.** Part of a title or of a path, in any order of words (`quart rev`, `peop sam`); the
  matched letters light up. The list is read from Rusty once when the picker opens and matched in
  Marley, so nothing you type is sent to Rusty or to an embedding provider.
- **Opening.** Enter, or a click, opens the page in a kept tab with the focus, or brings forward
  the tab that shows it.
- **Making a page.** When what you typed, read as a path (`projects/marley/review`), names no
  page, the list ends with Create page. Enter on it makes the page at that path, its folders too,
  and opens it; a Rusty older than its TICKET-041 makes it from the folder and the name instead,
  and refuses a folder that does not exist. A refusal shows in a toast and the picker keeps what
  you typed.

With Rusty off, or not connected, the key says so in a toast and opens nothing.

### The Memory tab (#664)

What Rusty remembers across conversations: the facts, preferences and context its agents read at
the start of one, the high ones first. The book after Decisions in the Brain view, or `rusty: open
memory`, opens it in a center tab.

- **The list.** Rusty's order: high, then normal, then low, the newest first within each. Each
  memory shows its text (up to three lines), its category, its importance, who stored it and the
  day it last changed. The count above it follows the Category menu, which shows All or one
  category.
- **Adding.** Type in the top field, give a category (empty files it under `context`, as Rusty's
  app does), choose Low, Normal or High, and press Enter. The field empties and the memory shows
  in its place in the list.
- **Editing.** Click a memory: a form holds its text, category and importance. Save sends all
  three; Delete asks "Forget this memory?" and then removes it. If Rusty refuses, its words show in
  the form and nothing changes.
- **Live.** The tab reads Rusty when it opens, when Rusty connects, when Rusty announces a change
  (one made in its app, its CLI or by an agent), and after each change you make. With Rusty off it
  says so.

### The Skills tab (#665)

Rusty's skills store: a skill is a `SKILL.md` Claude Code loads once it is active, and a skill an
agent writes waits, staged, for your approval. A script is a `.sh` beside a skill, the command
`rusty <name>`. The hammer after Memory in the Brain view, or `rusty: open skills`, opens the tab.

- **The lists.** Skills, staged first (marked pending; `auto` marks one an agent wrote), then the
  active ones, each with its description; then Scripts, `$ name` with its skill.
- **A skill.** Its description and body in editors; Save sends both, and Rusty answers with its
  safety scan, whose findings show under the editor. Scan runs the scan alone. A staged skill has
  Approve and Reject; if the scan blocks Approve, Rusty says why and Approve Anyway appears.
  Delete asks first.
- **A script.** Its text in an editor and Save. Run opens a terminal in the center that runs it
  with `bash`, as Rusty runs it, from the script's folder; it works from a project on this
  machine, and a staged skill's script waits for the approval.
- **New Skill.** A name (lowercase letters, digits and hyphens: Rusty refuses others in its own
  words), a description, a body, and whether to stage it. The skill is listed and chosen at once.
- **Live.** Rusty commits each change to its store. The tab reads Rusty again after each change,
  on Rusty's announcement, and when you come back to it; an edit in progress stays in the editors
  until you choose another skill.

### The Secrets tab (#667)

Rusty's vault of keys and tokens for providers and services. The lock after Skills in the Brain
view, or `rusty: open secrets`, opens it. The names show to anyone; a value shows, changes or goes
only behind your PIN. Never type the PIN to an agent.

- **The PIN.** The first time, set one: six characters or more, typed twice. Change PIN, while
  unlocked, sets a new one the same way.
- **Unlock.** Type the PIN and press Enter or Unlock. For Rusty's `pin_timeout_minutes` (five by
  default) each name has Reveal (then Hide), Copy while shown, Replace (a new value, Enter saves)
  and Delete, asked first. Five wrong PINs lock Unlock for a minute, and Rusty's words say so.
- **A new secret.** Its key and value, then Set or Enter; once a PIN is set this needs the unlock.
- **Locking.** Lock, the unlock running out, Marley's window losing the focus, or closing the tab
  locks it again: the values hide and the unlock is dropped.
- **What stays private.** Every PIN and value field hides what you type and empties once sent. The
  unlock lives only in the tab. Marley writes no PIN, unlock or value to its log, and with Zed's
  trace logging on, Rusty's messages are logged by their size alone.

### Capture and import (#663)

While Rusty is on, these commands in the palette put things into its brain:

- **`rusty: capture to today`** and **`rusty: capture to inbox`** open a small form with one field.
  Type a line and press Enter: Rusty adds it to the timeline of today's daily note, or of the
  inbox page, making the page when it is missing. A toast names the page for a few seconds, with
  Open. Rusty refuses an empty line, and its words show in the form.
- **`rusty: capture url`** takes an `http` or `https` address. Rusty fetches it (a web page, a PDF,
  Markdown or text, within 20 seconds) and keeps it as a source page under `sources/`; the form
  says Capturing… meanwhile, and the page opens when Rusty answers. A fetch that failed still
  makes the page, marked failed with Rusty's reason, and a toast gives the reason. An address
  Rusty does not take shows its refusal in the form. A source page's text came from the web: Rusty
  marks it as data, never instructions.
- **`rusty: import vault`** asks for an Obsidian vault's folder on this machine (Rusty reads its own
  disk), then shows Rusty's plan: the pages, folders, attachments, tags and bookmarks that come in,
  the collisions (pages the brain has already, which are skipped and never overwritten), and the
  links that will not resolve, each list in full. Import brings it in; the form stays until Rusty
  answers, then gives the report, and Open Report opens Rusty's report page under `inbox/`. Rusty
  never writes the vault, and an import that fails part way leaves nothing behind.
- **`rusty: open today`** opens today's daily note, as Today in the Brain view does.

Escape closes a capture's form; a capture already sent still lands. While Rusty is off or not
connected, each command says so in a toast.

### The Knowledge panel

The Knowledge panel sits in the right dock while Rusty is on (#646): the Book button in the status
bar, or `rusty: toggle knowledge panel` from the command palette, which opens it with the search
field focused. With Rusty off there is no button, the command says where to turn Rusty on, and a
panel that was showing closes.

- **The page view.** While a Page tab is the active item, the panel shows that page: its title and
  slug, its tags (frontmatter and inline) each with how many pages carry it, its backlinks with
  the line each link sits on and the link lit, and its own links in order. A click on a backlink or
  a link opens that page in the preview tab, and the panel follows it. A link to no page yet shows
  as written, with Create: one click makes the page through Rusty and opens it.
- **The project view** (#655). With no Page tab in front, the panel shows the brain page of the
  window's project: the project page whose `path` property lists the project's folder, else the
  one project page named like the folder (by Rusty's slug rule: `My Demo` is `my-demo`). It shows
  the page's title and slug, how it matched, Open Page, its `summary` property (else the first
  paragraph of its body), the follow-ups due today or overdue among the decisions linked to or from
  it (a click opens the decision; an overdue day in the warning colour), and the open tasks of its
  task group: the groups its `task_group` property names, else the group named like the page.
  A path can be absolute or under `~`; text such as `old laptop ~/code/demo` names another machine
  and matches nothing. A window with no folder shows the line to open a brain page.
- **Linking.** When no page matches, a callout says what it looked for and offers Link a Page
  (`rusty: link project page`), a picker over the project pages: the pick adds the folder to that
  page's `path`, after what was there, through Rusty, which commits it. When several pages share the
  folder's name, none is taken, and each has Link. With no task group, Link a Task Group
  (`rusty: link task group`) writes the group you pick to the page's `task_group`. Nothing is
  written except on your pick. A remote project matches by name only and offers no link.
- **A tag.** A click on a tag searches for it: `tag:<name>` in the field, sent.
- **Brain search.** The field takes Rusty's query as you would type it in Rusty's search pane:
  words, `tag:`, `path:`, `file:` and `type:`, quotes to keep spaces, a leading `-` to exclude.
  It asks Rusty when you press Enter, never while you type, since Rusty embeds each query it is
  sent when an embedding provider is set. Match Case and Regular Expression sit beside the field
  and ask again. The results show each page's title, slug and the matching words lit; Up and
  Down move through them, Enter opens the selected one (the first when none is), Escape clears
  the query and brings the page view back.
- **Live.** A change Rusty announces reads the page and the query again, with no click.
- **Not connected.** With Rusty's connection down, the panel shows why in place of the page view.

### The Graph tab

The Graph tab draws Rusty's vault as a graph in a center tab (#647): `rusty: open graph` from the
command palette, or the graph icon after Today in the rail's Brain view. With a brain page in front
it opens on that page's neighbourhood; with none, on the neighbourhood of the window's project page
(#655), its header marking it "(project)". `rusty: open local graph` turns it to the page in
front, else the project's page, or says to open a page first. There is one Graph tab per window.

- **What it shows.** Pages are dots, larger the more links they have, coloured by page type in
  Zed's accent colours; the legend names each type with its count. Links are lines. A decision's
  typed edges are dashed: consulted, supersedes and follows up, a colour each.
- **Local and Vault.** Local is the neighbourhood of the brain page last in front, one to four
  links deep (the Depth buttons), that page ringed in the middle; it follows the pages you open.
  Vault is every page.
- **The mouse.** A click on a page opens it. Rest the pointer on a dot to light it and its
  neighbours and read its title. Drag the background to move the view, turn the wheel to zoom about
  the pointer (a pinch too), and drag a dot to hold it while the rest settle around it. Titles show
  once you zoom in.
- **The panel** at the top right: Local or Vault, the depth, and a filter where every term must
  match: words in a title or slug, `tag:name` (with tags nested under it), `path:folder/`,
  `type:decision`. A click on a type in the legend hides it. The switches: Tags (each tag as a dot
  joined to its pages; a click on one filters to it), Unresolved links (a hollow dot for each link
  to no page yet), Decision edges and Orphans. Restart layout lays it out afresh, Fit shows
  everything, and the close button folds the panel to a button.
- **Big vaults.** The layout runs away from the window's thread and settles in a moment. Past 2,000
  dots the tab lays out the 2,000 most linked and says how many it shows; a filter or a local graph
  reaches the rest.
- **Live.** While the tab shows, a change Rusty announces reads the graph again, the dots staying
  where they were; a change while it is hidden is read when it shows. With the service connection,
  each showing reads.
- **Off.** With Rusty off the tab is empty, says so, and asks Rusty nothing.
- **Groups** (#657). Under the switches, Groups, Display and Forces fold open with a click on
  their names. New group adds a row: type a query in the filter's words and the pages it matches
  take the group's colour (red, then green, yellow, magenta, cyan and blue, from your theme's
  terminal colours), the row counting them. A page two groups match takes the first one's colour,
  so a group whose pages an earlier one already has counts 0. The dot gives the group the next
  colour, and × removes it. With no group, pages keep their type's colour.
- **Display.** Arrows draws a head at the end of each link and decision edge where it points;
  a tag's lines have none. Text fade threshold, 0 to 1 (0.5 to start), sets how far you zoom in
  before titles show: at 1 they show from far out, at 0 only close in. Node size (0.3 to 3) and
  Link thickness (0.2 to 4) scale every dot and every line.
- **Forces.** Center force (0 to 1, 0.5) pulls everything to the middle, Repel force (0 to 20,
  10) pushes dots apart, Link force (0 to 1, 1) pulls linked dots together, and Link distance (30
  to 500, 250) is the length links settle at. A change lays the graph out again from where it is.
  The names and ranges are Rusty's, but they move Marley's own layout, so a value does not draw
  the picture Rusty's app draws with it.
- **Sliders.** Drag the thumb, click the track, or, with the focus on the slider, use the arrow
  keys, Page Up and Page Down for ten steps, and Home and End for the ends. The value shows by
  the slider's name.
- **Kept.** The groups, the four switches, the depth and every slider are one set for all Graph
  tabs in every window, and Marley keeps them across restarts. A Graph tab open when Marley quits
  comes back in its place with its scope, page, filter, hidden types, panel and open sections, and
  reads the graph once Rusty is connected. Its dots are laid out afresh. With Rusty off when Marley
  starts, no Graph tab comes back.

### The Tasks tab

The Tasks tab shows Rusty's to-do lists in a center tab (#658): the list icon after Graph in the
rail's Brain view, `rusty: open tasks` from the command palette, or the list icon on the Tasks
heading of the Knowledge panel's project view, which opens the project's task group. There is one
Tasks tab per window.

- **What it shows.** The lists in a column on the left, the chosen one lit; its tasks on the
  right, in Rusty's order. A done task stays in its place, struck through. Show archived lists the
  archived tasks too, in their places, faint and marked Archived.
- **Tasks.** Type in the field above the tasks and press Enter: the task goes last, and the field
  stays ready for the next. A click on a task's box, or Space, checks it done or open again.
  Double-click a title, or press F2 or Enter, to rename it in its row: Enter keeps it, Escape
  drops it. Delete or Backspace archives the selected task, or brings an archived one back.
  Shift+Delete, or Delete for Good… in its right-click menu, asks first, then deletes it; nothing
  brings it back. A task's menu also has Rename and Archive or Restore.
- **Order.** Drag a task onto another to put it in that one's place; a line shows where it lands.
  Alt+Up and Alt+Down move the selected task one place.
- **Lists.** The + by Lists names a new list, which is chosen with the field ready. A list's
  right-click menu renames it, or deletes it after asking, with all its tasks.
- **Keys.** Up and Down move the selection, Down from the field goes to the first task, and Escape
  goes from the tasks to the field, then empties it.
- **When it reads.** Each change goes to Rusty one at a time, in order, and the tab reads Rusty
  again after it. It also reads when Rusty announces a change (while it is hidden, when it next
  shows), when you click back into it, when Marley's window comes back to the front, and on
  Refresh. A task an agent adds through its own Rusty is announced to no one else, so it shows
  at one of those moments. A change Rusty refuses shows Rusty's words in a toast, and the tab then
  shows what Rusty holds.
- **Off.** With Rusty off the tab drops its lists, says so, and asks Rusty nothing; while Marley is
  not connected it keeps what it last read and takes no edit.

### The Decisions tab

The Decisions tab lists the decisions recorded in Rusty in a center tab (#659): the double check
after Tasks in the rail's Brain view, or `rusty: open decisions` from the command palette. There is
one Decisions tab per window. System One's log is another tab, System One calls.

- **What it shows.** Under the title, Rusty's loop in one line: `brain_ask` before a choice,
  `brain_decide` when it is made, `brain_follow_up` when its day comes. Then Due, the decisions
  whose follow-up day has come, in Rusty's order; an overdue one shows its day in the warning
  colour with the word overdue. Then every decision, the newest decided first, under their count.
  A row gives the title, the status (decided, kept, revised or superseded), the day it was decided,
  the day of its last follow-up, and the next follow-up day when one is set; a replaced decision
  names the one that replaced it. A long title is cut; hover the row for the whole title and the
  page's slug.
- **Opening one.** A click on a row opens the decision's page in a Page tab, and a click on the
  title after "replaced by" opens the decision that replaced it. A row's right-click menu has Open
  Page too.
- **When it reads.** It asks Rusty (`brain_due`) when it opens and when Rusty announces a change;
  while it is hidden it waits until it shows again. What is due and what is overdue are Rusty's
  answers: Marley works out no date.
- **A failed read** shows Rusty's words and Read again above the list it last drew.
- **Off.** With Rusty off the tab says so and asks Rusty nothing, and `rusty: open decisions` shows
  a toast naming the switch; while Marley is not connected the tab keeps what it last read.
- **Following up** (#660). Follow Up at the end of a row under Due, or Follow Up… in the
  right-click menu of any decision that is not replaced, opens a form named for the decision:
  - Choose Kept, Revised or Superseded; none is chosen at first.
  - Write how it went. Shift+Enter starts a new line.
  - Revised asks for the next follow-up day, `YYYY-MM-DD`; leave it empty for none. Kept and
    Superseded clear the day, so the decision leaves Due.
  - Superseded asks for the decision that replaced it, from a list of the others you can filter
    by typing; Change… picks again.
  - Enter or Record sends the follow-up to Rusty (`brain_follow_up`). Rusty adds it to the page,
    sets the status and the days, and the tab reads again. If Rusty refuses, a day that is not a
    date for one, its words show in the form and nothing is written. Escape, or a click outside,
    closes the form without recording.

## The Marley agent

An agent that explains Marley and sets it up with you (#683). It reaches Marley's docs and
settings through Marley's MCP server: `docs_search`, `docs_read`, `settings_schema`,
`settings_read`, `actions_list`, `settings_change` and `keymap_change`. Every change it proposes
waits for your Apply. **Agent** (`marley.assistant.agent`, #687) picks what it runs on:

- **Claude Code** (`claude_code`, the default): Zed's `claude-acp` adapter on your own login,
  as a **Marley** entry in the Agent Panel. It is told what it is for and kept from editing files
  or running commands: Bash, Edit, Write, NotebookEdit and MultiEdit are off, and so is bypass
  mode.
- **Codex** (`codex`): the registry's `codex-acp` adapter on your own login, as the same
  **Marley** entry. It starts in Codex's read-only mode, which asks before any edit or network use.
  The instructions go in as Codex's `developer_instructions`.
- **Zed's agent** (`zed`): a **Marley** profile for Zed's own agent, with no built-in tools and,
  of the context servers' tools, only the seven above. Zed's agent has no prompt per profile, so
  the tools' own descriptions guide it.

- **Turning it on.** Settings → Marley → **Marley Agent** (`marley.assistant.enabled`), off by
  default, and **Agent** below it. When you have not chosen, Marley offers it once a few seconds
  after start. It offers whichever it finds first: Claude Code signed in (`claude auth status`),
  then Codex signed in (`codex login status`), then a Zed default model whose provider is signed
  in. The offer names that agent. Turn On writes the switch with that agent, and Not Now writes it
  off.
- **Using it.** On Claude Code or Codex: New Agent Thread → **Marley**, from the Agent Panel or a
  project's + in the rail. On Zed's agent: a Zed Agent thread, then the profile picker (Ctrl+I) →
  **Marley**. Ask how a feature works, what a setting does or is set to, or which key runs a
  command. You can also ask it to change a setting or bind a key. Picking the profile makes it
  your default profile, as any pick does. When the profile goes away, a default of Marley goes
  back to Write.
- **In a terminal** (#684). `marley: open marley agent in terminal` starts the same agent with
  its own interface in a new terminal of the active project.
  - On Claude Code it runs `claude --append-system-prompt-file <data dir>/assistant/instructions.md
    --disallowedTools Bash Edit Write NotebookEdit MultiEdit`, and reaches Marley's tools through
    Claude Code's Marley plugin.
  - On Codex it runs `codex --sandbox read-only -c developer_instructions="…"` (#687).
  - The palette lists the command only while the switch is on and the agent is not Zed's.
- **How it is made.** While the switch is on, Marley adds to the settings' defaults, not to your
  file:
  - **Claude Code:** a `Marley` agent server with the command Zed resolves for `claude-acp`, run
    with the `claude` Marley launches and no API key. Also `marley.agent_session_meta.Marley`, the
    `_meta` each of its sessions carries: the instructions appended to Claude Code's system
    prompt, and the tools it may not use. Any agent server can have an entry in
    `marley.agent_session_meta`.
  - **Codex:** the same entry with `codex-acp`'s command, `INITIAL_AGENT_MODE=read-only`, and
    `CODEX_CONFIG` holding the instructions and `sandbox_mode: read-only`. It runs the adapter's own
    Codex, which reads your `~/.codex` login.
  - **Zed's agent:** `agent.profiles.marley`.
  - `MARLEY_ASSISTANT_ADAPTER` names an ACP program to run in the adapter's place.

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
| `terminal_read` | One block's command and output as text, a page at a time (#680): the newest whole lines that fit in 12,000 bytes (2,000 lines at most), numbered from the block's first line (`first_line`, `last_line`, `total_lines`). `previous`, passed back as `before`, reads the page before; a line longer than a page keeps its end (`line_cut`). `redacted` counts the secrets hidden in the whole output |
| `terminal_find` | The line of a block's output a `query` in words names, such as "where the server refused the connection": its number and up to three candidates, by the query's words first and the System One model for the rest (#567); listed only while its use is on |
| `terminal_screen` | What a terminal's screen shows now: its rows (secrets redacted), the cursor, whether a full-screen program has the alternate screen, the program in the foreground, and `generation`, `taken_over` and whether you approved writes to it (#525) |
| `terminal_type` | Types into the program running in a terminal's foreground: `text` as a paste, `keys` by name (`escape`, `ctrl-c`, `up`), and Enter with `submit`, at most 4,096 bytes, given the `generation` `terminal_screen` gave. Never at the shell's prompt or into another agent CLI (#525) |

When an agent first types into a program, such as psql or a debugger, Marley asks: a card under
the terminal names the agent, the program and what it would type, with Allow and Deny, and a toast
with Show points to it. Allow holds for that program until it exits; the Agents section's **Agent
Terminal Writes** makes Marley ask for every write, or never. An unanswered question refuses the
write after 25 seconds. Once an agent has typed, a bar under the terminal shows what it typed:
**Take Over**, or Ctrl-I in the terminal, stops its writes until you choose **Hand Back**. Without
an agent's writes, Ctrl-I reaches the program as it always did.

The ports family (#521):

| Tool | What it gives |
|---|---|
| `ports_list` | The ports the rail's port rows show: each listener's project, the project folder its working directory is in, its address, port, URL, pid, process name and working directory; never its command line, which can carry a token |

The docs, settings and actions families (#681), for questions about Marley itself:

| Tool | What it gives |
|---|---|
| `docs_search` | Zed's docs and this guide, as this Marley ships them, searched for a query in words: up to 10 sections, best first (those holding the most of its words, then the most hits, a hit in a heading counting more), each with its page, heading and a snippet |
| `docs_read` | A page (`zed/<path>` or `marley/guide.md`), whole or one section by `heading`, from the top in pages of up to 12,000 bytes; `next`, passed back as `after`, reads on. Zed's key placeholders show the keys bound in this Marley, and its action placeholders their palette names |
| `settings_schema` | What a setting is, by its key path (`terminal.font_size`): its type, description, allowed values and default, and for an object its keys; an empty key lists the top-level keys |
| `settings_read` | What a setting is set to: its value in each settings file that sets it, highest precedence first (each open project's `.zed/settings.json`, your settings, the defaults), which file wins, the value in effect, and `global`, the value where no project file sets it. A value under a name such as `token`, `secret`, `password`, `api_key` or `auth` comes back as `[redacted: setting]` |
| `actions_list` | The actions whose name or documentation holds every word of a query, up to 50, shortest name first: each one's name, palette name, first line of documentation, and the keys bound to it with the context each applies in |

Their refusals carry `no_doc`, `no_section`, `no_setting`, `too_large` or `bad_argument`.

One write tool sits with them (#682):

| Tool | What it does |
|---|---|
| `keymap_change` | Proposes a key binding: `keystrokes` as Zed's keymap writes them (`ctrl-alt-m`), an `action` (actions_list finds its name), an optional `context` (`Workspace`) and optional `arguments`. The same notification names the keys, the action's palette name and the context; Apply adds the binding to your `keymap.json` through Zed's keymap updater, comments and other bindings kept, and it works at once. An action Marley lacks is refused with `no_action` and close names, keystrokes or a context that do not parse with `bad_argument` (#686) |
| `settings_change` | Proposes a value for one key of your settings. A notification in every window names the agent, the key, the value now and the value proposed, and the file, with Apply and Decline; Apply writes it, keeping the file's comments and other keys, and it takes effect at once. A key the settings schema lacks (`no_setting`) or a value Zed would not parse (`invalid_value`, with Zed's error) is refused before you are asked; Decline answers `declined`, no answer within 25 seconds `no_answer`, and a file that changed while you were asked `changed` |

The browser family, reading:

| Tool | What it gives |
|---|---|
| `browser_tabs` | Each page: its id (the `tab` the other tools take), title, URL, whether it loads, and which tab a call without `tab` acts on, the one you focused last |
| `browser_look` | The page as you see it: URL, title, viewport, scroll, loading, the focused element and the selection, and the frame on your screen as an image |
| `browser_snapshot` | The accessibility tree as text: the interactive elements with refs such as `e3` (every node with `full`), cross-site iframes included, no field values |
| `browser_find` | The element a `query` in words names, such as "the sign in button": its ref, which `browser_click` takes, and up to three candidates, by the query's words first and the System One model for the rest (#567); listed only while its use is on |
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

`browser_find` and `terminal_find` are listed and answered only while their use is on in the
System One settings ([The find tools](#the-find-tools)). A client lists its tools when it
connects, so a Claude Code already running sees a tool you turned on after you reconnect
Marley's server (`/mcp`), or in its next session.

The server also holds `fleet_snapshot` and `session_surface_to_human`, which it does not list. The
fleet stays empty until prong 2 feeds it, and a session write is refused, since Marley grants no
`session.write`.

### Grants and what keeps an agent in check

- Read tools need no grant. A write tool needs its class granted, and Marley grants
  `browser.write`, and nothing else, when it starts the server. No setting changes that today.
- The checks on the browser's write tools are your agent client's approval of each call (Claude
  Code asks by default) and the Browser tab, where every action happens in front of you and the
  Agent chip names it. With the click consequence on, a consequential click also waits for you
  (see "The click consequence").
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

### What agents are told, and how a call is refused

- The `initialize` answer carries `instructions` (#680), under 2 KB, which Claude Code adds to
  its model's system prompt: read a block's output before asking for a paste, page back with
  `previous`, use `terminal_run` for what you should see run, answer a waiting program through
  `terminal_screen` and `terminal_type`, and use the Browser tab rather than another browser.
- A refused call answers `{result: "refused", code, reason, next_steps}` (#680). The terminal
  tools' codes are `no_terminal` (with the terminals there are), `no_block` (with the blocks'
  range), `output_gone` and `bad_argument`; the server's are `not_granted`, `tool_off`,
  `not_permitted`, `timed_out` and `unavailable`. Any other refusal says `refused`.
- A page of `terminal_read` or `terminal_run` holds at most 12,000 bytes of output, so a whole
  answer stays near 25 KB, about 6,000 tokens, under the 10,000 at which Claude Code warns about
  a tool's result. When a page leaves part of the output out, its text starts with a line such as
  `[lines 6002 to 8001 of 8001; read earlier lines with terminal_read before=6002]`, for a client
  that reads only the text.

### Limits

- The server holds 32 sessions at most, and an idle one lasts 30 minutes, so a client that never
  closes its session holds a slot that long. The bridge closes its own.
- A call the app does not answer within 30 seconds is refused with `timed_out`.
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
terminal to send to: click in one, then Send." While Claude Code in that terminal waits for your
answer, Send types nothing, since the line would land in its prompt: the tray says so, and the pick
keeps its caption for a Send once you have answered (#508). Picks last for the session. Pick mode
works in the page's own frames, not inside a cross-site iframe.

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

## System One

Marley can ask a System One model typed questions about what it knows (#565): a yes or no, a
choice among fixed options, or a score. Jev, through TypeSafe's API, is the first model. What
comes back is a reading a feature may show, rank or route on; it never approves, sends or stops
anything. Nothing in Marley needs it, and it stays off until you turn it on. Off, Marley makes no
request, reads no key and writes no file.

### Turning it on

1. Give Marley the key: set `MARLEY_SYSTEM_ONE_KEY` in the environment Marley starts from, or open
   System One calls (`marley: open system one calls`), choose Set Key, paste the key and press
   Enter, which writes it to the system keyring at the provider's URL. The variable comes before
   the keyring. System One calls shows where the key came from and never shows the key.
2. List the folders whose projects may send their state, in `settings.json`:

   ```jsonc
   "marley": {
     "system_one": {
       "enabled": true,
       "projects": ["~/code"],
       // Projects that send the facts Marley computes and none of their text.
       "metadata_only_projects": ["~/code/client-work"]
     }
   }
   ```

3. Turn it on: the System One switch on the Marley settings page, or `enabled` as above.

### What leaves the machine

A state is labeled lines. Some are facts Marley computed, such as a project's name, an exit code
or a program. Others are text, such as a command or a terminal's title. Every value passes through
#516's redaction rules and your `redaction_patterns`, whatever `redact_secrets_for_agents` says,
and the key itself is hidden as well. A text value is masked first, then cut to 300 characters. A
project on `metadata_only_projects` sends the facts alone. A project on neither list, or a remote
one, sends nothing: the call is refused before any request.

### The check

`marley: system one check` asks whether the last command of the terminal you used last failed.
It exists so you can see one request and one answer. A toast gives the reading, the provider, the
tokens and the time, as in `System One: command failed: yes (0.92) · typesafe · 1,200 tokens ·
310 ms`. Its mode, Check on the settings page, is Act; Off turns it off.

### The stop kind

With its mode on, an idle Claude Code's rail row says what the stop needs (#566): `done · checked`
(the work is done and a command ran after the last edit), `done · claimed` (done, with no check
behind it), `asks you`, `blocked`, `still going` or `interrupted`. When the prompt has parts, such
as "Add a README. Add a license.", the row's last line names a part the message leaves out: `not
covered: "Add a license."`.

- Marley's own rules come first and send nothing: an interrupt, a permission never answered, or a
  last sentence that asks you something (a question mark, or an opening such as "Should I" or
  "Let me know") settle the kind with no call. Only the rest is asked, and only for a listed
  project.
- A state sent for a stop holds the facts (the tools and how many times each ran, the failures,
  the permissions pending, whether a command ran after the last edit, how long the turn took),
  the prompt, the last message as the plugin cut it, and the prompt's parts, masked as above. A
  metadata-only project sends the facts alone and is not asked about parts.
- The model can take a check away but never add one: `done · checked` needs the command after the
  edit that Marley saw.
- The mode is Stop Kind on the settings page, or `uses.stop_kind`: Off (the default) makes no
  call; Shadow asks and logs, and the row keeps saying `idle`; Suggest adds the kind after `idle`
  with a question mark (`idle · still going?`); Act shows it in place of `idle`. The next prompt you
  type clears it. Agents reading `fleet_snapshot` see the kind as the seat's `stop_kind` labels.
- Each stop is a row in System One calls, the ones the rules settled under the provider `rules`,
  and your next prompt adds an outcome to the day's file: how long it came after the stop and how
  long it was.

### The find tools

`browser_find` and `terminal_find` (#567) are agents' tools: an agent asks for "the sign in
button" or "where the server refused the connection" and gets a ref or a line instead of reading
a whole page or block.

- The query's words come first and send nothing: an element whose role and name, or a line,
  hold every word of the query (short words such as "the", "where" and "did" aside) is the answer.
- When several do, the model ranks them; when none does, it looks among every element of the
  page, or every line of the block's newest part (about 24,000 tokens), up to 254 a request. It
  also says whether anything matches at all.
- The state sent holds the query and the items as labeled lines, masked as above: a block's
  output is masked whole before any cut. A project on neither list, or a metadata-only one, gets
  the words' answer and a note, with no call.
- The modes are Browser Find and Terminal Find on the settings page, or `uses.browser_find` and
  `uses.terminal_find`: Off (the default) takes the tool out of the agents' list; Shadow answers by
  the words and logs the model in System One calls; Suggest gives the model's candidates marked to
  check; Act gives the element or the line to act on when the model found one and is sure.
- An answer that is not sure names the tool to read with (`browser_snapshot`, `terminal_read`).

### The stall kind

With its mode on, a working Claude Code's rail row says when it may be stuck (#569), with a
warning mark whose tooltip says why. Marley never stops, interrupts or types into the agent on a
flag: the flag only marks the row.

- `looping?`: the turn ended the same tool line three times in a row, such as `Bash: cargo test`,
  or failed the same line twice. Marley's own rule decides, with no call. A run of edits or reads
  of one file is not a loop, since the line names only the file.
- `stalled?`: the turn has been quiet past a check with no tool of its own using the CPU, and the
  model read it as waiting for input, stuck on something that is not coming, or frozen. A tool
  whose processes use the CPU, such as a build or a test run, is a long task, and nothing is
  asked however long it runs. The CPU counted is the turn's tools': Claude Code's own and its
  servers' are left out.
- The checks come after Stall Check After Seconds on the settings page
  (`stall_check_after_seconds`, 60 by default) and at twice, four and eight times it, so a quiet
  turn is asked about at most four times; 0 turns the quiet checks off and leaves the loop rule on.
- A state sent for a quiet turn holds the facts (the project, how long it has been quiet, the tool
  in flight by name, whether the tools use the CPU, the subagents, the permission mode) and the
  prompt, the tool line and the terminal's last five lines, masked as above. A metadata-only
  project sends the facts alone.
- The mode is Stall Kind on the settings page, or `uses.stall_kind`: Off (the default) makes no
  call and shows nothing; Shadow asks and logs, and the row is unchanged; Suggest shows the flag;
  Act adds one desktop notification for the quiet spell, `repo: Claude Code may be stuck`, when the
  terminal is not in front. The agent's next event takes the flag off, and a loop's leaves when the
  loop ends. Agents reading `fleet_snapshot` see the flag as the seat's `flag` labels.
- Each reading is a row in System One calls, a loop's under the provider `rules`, and what came
  next is its outcome in the day's file: how long until the agent's next event, and which it was.

### The click consequence

With its mode on, an agent's click in a Browser tab that pays, deletes, sends in your name or
changes an account waits for you (#571). A card under the tab's toolbar says who wants to click
what and why, as in "Claude Code wants to click button “Place order”, which pays (its name), on
shop.example", with Refuse and Allow, and a toast with Show points you to it.

- Allow clicks, but only the same element on the same page: if the page changed under the pause,
  nothing is clicked. Refuse, or 25 seconds with no answer, tells the agent you did not allow it,
  so its next move is to ask you. While a click waits, the tab's other writes wait too.
- The card never takes the focus by itself, so a key you meant for the page never answers it.
  Click Refuse or Allow, or click the card (or the toast's Show), then press Enter to allow or
  Escape to refuse.
- Marley's own rules decide first and send nothing: the element's name ("Place order", "Delete
  account", "Send", "Change password" and the like), a form that posts to a checkout or a delete,
  a link to one, and the page's words around the element. A plain click, such as "Next" or
  "Cancel", goes at once.
- What the rules leave open, a "Continue" among text about a charge, may be read by the model,
  for a listed project only, with the element's name, its form's target, the page's path and
  title and the text around it, masked as above. A reading can add a pause, never remove one.
- By default only agents with no permission prompt of their own wait: Claude Code in a Marley
  terminal running `bypassPermissions` or `dontAsk`, Zed's agent while its tool permissions let
  `browser_click` run without asking, and any caller Marley cannot name. Browser Click Pause
  Agents on the settings page (`browser_click_pause_agents`: `agents_without_prompts`, or
  `all_agents`) makes every agent wait.
- The mode is Click Consequence on the settings page, or `uses.click_consequence`: Off (the
  default) pauses nothing; Shadow pauses on the rules and logs the model's reading; Suggest adds a
  notice after a click the model reads as consequential; Act pauses on that reading too.
- Each pause is a row in System One calls, a rule's under the provider `rules`, with how it ended,
  and the flight recorder keeps the pause and its end.

### The inbox risk

With its mode on, each entry of the rail's "Needs you" section carries chips that say what the
waiting action would do (#568), and the entries order by them: what could destroy data, touch
credentials or rewrite history first, then what sends data out, installs software or changes an
account, then what reaches outside the project, and among equals the one that has waited
longest.

- Marley's own rules read the tool and what it acts on, with no call: `destroys` for `rm -rf
  build` or `git reset --hard`, `credentials` for `~/.ssh/config`, a `.env` or a token in the
  command, `rewrites history` for a rebase or a forced push, `sends out` for a `curl` that posts
  or a `git push`, `installs` for `npm install` or `curl … | sh`, `outside project` for a write
  or a command that reaches outside the project's folders, and `claims approval` for text that
  says the action was already approved, which puts the entry higher, never lower. A click a
  Browser tab holds carries its own chip: `pays`, `destroys`, `sends out` or `changes account`.
- What the rules find nothing on may be read by the model, for a listed project only, with the
  tool's name, the agent and the project as facts and the waiting line, masked, as text. A
  reading can add a chip and move an entry up, never take a chip away or move an entry down.
- Nothing here answers anything: Allow and Deny stay your clicks, and Claude Code's own
  permission checks keep their say.
- The mode is Inbox Risk on the settings page, or `uses.inbox`: Off (the default) keeps the inbox
  oldest first with no chips; Shadow shows the rules' chips and order and logs the model's
  reading; Suggest shows the model's chips with a question mark; Act shows them with a dashed
  border and lets them order the inbox too.
- Each entry is a row in System One calls, a rule's under the provider `rules`, and how it was
  cleared, from the inbox or elsewhere, is its outcome.

### The question route

With its mode on, each entry of the rail's "Needs you" section also says who should answer it
(#570): `for you`, `for the manager`, `could proceed` (the agent could go on by itself) or
`unclear`. Nothing here answers anything; the mark tells you what to look at first.

- Marley's own rules decide first, with no call: what the inbox risk marks as destroying,
  touching credentials, rewriting history, sending out or reaching outside the project, a held
  click, and anything that names money or a message to people is yours; a read inside the
  project, or a command that only reads (`ls`, `cat`, `git status` and the like), could proceed.
- The rest, such as `npm test` or an agent's question with its options, may be read by the model
  for a listed project, with your prompt and the question's options as masked text. A reading
  that cannot tell, falls under the floor or gets no answer is `unclear`, which sorts with yours.
- The pointer on a mark says where it came from: "Marley's rule: rewrites history", or "System
  One: for the manager (0.88)".
- No manager agent is connected yet, so a `for the manager` entry stays yours to answer; the mark
  is what the manager will take once rustal-harness's manager runs.
- The mode is Question Route on the settings page, or `uses.question_route`: Off (the default)
  shows no marks; Shadow shows the rules' marks and logs the model's; Suggest shows the model's
  with a question mark; Act shows them and orders the entries within each level: yours and the
  unclear first, then the manager's, then what could proceed.
- Each entry is a row in System One calls, and its outcome says who answered it: `owner` when you
  answered it from the inbox, or had its terminal or thread in front while it waited, and `agent`
  when it went on without you.

### The typed line

With its mode on, a line typed at a shell's prompt that Marley's rules cannot settle is read by the
model (#573): a command followed by plain words with an English word among them, such as `find
all the large files in this repo` or `rm the old build folder`. Lines with flags, pipes, paths or
variables, lines whose first word is no command, and plain commands such as `git status` are
never asked about.

- The reading shows after the line, where English at the prompt shows its hint: `· a request?
  ctrl-shift-enter asks the agent`, `· a comment?`, or `· English after rm? ctrl-shift-enter
  asks the agent`. Suggest adds the question mark; Act shows it plainly, and for a command
  followed by English draws the words the command would take as arguments in the warning colour
  in the prompt editor. A reading of `command` hides the hint in Act.
- Marley asks only after the line has stayed the same for 250 ms, once per line, with the line as
  masked text and facts such as where its first word comes from, the word count and the last
  exit code. A line holding anything the redaction rules find is never sent. Enter never waits:
  a reading that comes after the line changed is dropped.
- Ctrl+Shift+Enter also asks the agent when the reading offers it. Enter always runs the line.
- The mode is Typed Line on the settings page, or `uses.typed_line`: Off (the default) asks
  nothing; Shadow logs the reading in System One calls and shows only the rules' hint.
- Each call's outcome follows it in the day's file: `entered` and the command's `exit N`,
  `asked the agent`, `edited`, `cleared`, or `dropped`.

### System One calls

`marley: open system one calls` opens a tab with the day's calls, newest first: the time, the use,
the project, the provider and model, the reading (refused and unavailable calls in red), the time
taken, the tokens and the cost. A click opens a row to the state as it was sent and the answers as
they came. The header gives the day's calls and spend against the budget, the provider, where the
key came from, and whether the breaker is open. Run Check runs the check, and Set Key and Forget
Key write and remove the keyring's key. The tab was called Decisions before #659, when Rusty's
Decisions tab took the name; a key bound to `marley::OpenDecisions` still opens it.

### Providers, budget and failures

| `provider` | Who answers |
|---|---|
| `typesafe` (the default) | TypeSafe's API, `https://api.typesafe.ai/v1/systemone` |
| `compatible` | Another server that speaks the same request, at `endpoint`: `https`, or `http` on this machine |
| `rules` | Each feature's own rules, with no request |
| `replay` | Answers recorded in `system_one/replay.jsonl` under Marley's data directory |
| `cloudflare` | Jev through Cloudflare Workers AI, which states it keeps no data, in the account `cloudflare_account_id` (#548) |

- **Cloudflare** (#548). Set `cloudflare_account_id` (also on the settings page) and give Marley
  an API token that may run Workers AI: `MARLEY_CLOUDFLARE_API_TOKEN`, or Set Cloudflare Token in
  System One calls, which writes it to the keyring at the account's URL. Marley reads the token
  only while the layer is on and some project uses Cloudflare, masks it in every state as it does
  the key, and never shows it. Cloudflare's model is `typesafe/jev`, at the same price as
  TypeSafe's. A 401 or 403 from Cloudflare marks the token refused until it changes; Cloudflare's
  own words show in the call. `cloudflare_api` (`https://api.cloudflare.com/client/v4`) moves the
  base: `https`, or `http` on this machine.
- **A provider per project** (#548). `provider_by_project` names one for a project, by folder:

  ```jsonc
  "provider": "typesafe",
  // A client's code goes through Cloudflare, which keeps nothing; the rest go direct.
  "provider_by_project": { "~/work/client": "cloudflare" }
  ```

  An entry applies to a local project with a folder in it or under it, the longest folder
  winning; every other project takes `provider`. A project still sends only when `projects`
  lists it. Only your own settings set this, never a project's `.zed/settings.json`. The budget
  and the breaker are the whole layer's: five failures through one provider hold the others too.
- `model` is pinned (`jev-1.13.0`).
- `daily_budget_cents` (50) caps the day's spend, counted from the input tokens at the model's
  price: 0.042 USD per million for Jev, or `price_cents_per_million_tokens` for a compatible
  server. Once it is spent, calls wait for the next day.
- A call with no answer within its deadline (2 s for the check), a 5xx or an unreachable provider
  reads as unavailable, with the reason. A 429, 503 or 529 is tried once more while the deadline
  leaves time. Five failures in a row hold calls for two minutes.
- A state the same as the last one answered for the same terminal makes no new call.
- `uses` sets each feature's mode: `off`, `shadow` (ask and log, shown only in System One calls),
  `suggest` or `act`. The check, the stop kind and the two find tools are the four so far.

Every call, refused and failed ones included, is a line in `system_one/calls-<day>.jsonl` under
Marley's data directory, readable by you alone: the masked state as sent, the answers, the reading,
the time, the tokens and the cost.

## The harness's sessions

Marley follows the sessions of a rustal-harness (#534) when your settings name its MCP server, as
a context server's command is named:

```jsonc
"marley": {
  "harness": { "command": "/path/to/rustal-harness/bin/rh", "args": ["--state", "/path/to/root", "mcp"] }
}
```

Or let Marley run the harness itself (#632): set `"marley": { "embedded_harness": true }` where
`rh` is installed. Marley finds `rh` through `MARLEY_RH`, else your PATH, starts `rh serve` on a
root in its own data folder (`~/.local/share/marley/harness`), and follows it. The section's
header then also says when the runtime is starting or stopped and why, or that no `rh` was found
and where Marley looked; hover it for the whole reason. A stopped runtime is started again after a
growing wait, and the harness's sessions, kept by its tmux server, outlive it: turning the setting
off or quitting Marley ends the runtime and leaves them running. A `harness` command, when set,
wins over this.

The rail then shows a Harness section after the projects and the containers. Its header says
whether Marley is connected, connecting, or why the harness is not running; a click on it folds
the rows. Each session is a row: its title, a dot for its state, and under it the question it
waits on, `no update in N m` for a working session quiet longer than `no_update_after_minutes`,
or its state in a word. Where the harness says more (#640), the row says it too: how far the agent
is (`40 % · Running the tests`) and its account's most-used quota window (`five_hour 62 % ·
resets in 1 h 35 m`), each on a line of its own, and the pointer on the row shows where the state
came from, every quota window and the account. A state the harness read off the agent's screen
(`detected`), or one from a source Marley does not know, is drawn weaker: a grey dot and the
source's word at the end of the line. Marley shows it and never acts on it, so its question is not
in the inbox. A click opens a tab with the session's last 500 lines, which follows its
output while the session runs. A session waiting on a question is also in the rail's "Needs you"
inbox, with its options; opening it opens the session's tab. Marley only reads: answering, sending
and stopping stay the harness's own commands for now. When the harness stops answering, the
section keeps its rows marked stale and Marley starts the command again after 1 s, then 2, 4 and
so on up to a minute. The section stays out of the rail's keys and its filter, as the containers
do. Without the setting Marley starts nothing.

## The Fleet panel

The Fleet panel (#607) lists the agents a workflow store reports, grouped under the hosts they
run on. It is the first piece of Marley's fleet view (`docs/marley/fleet-contract.md`): the
store says what each agent works on and where its run is, and Marley draws it. Until the Rustal
services serve that contract, the one provider is a pseudo one that shows example data.

Turn the pseudo provider on in your settings:

```jsonc
"marley": {
  "fleet": { "providers": [ { "kind": "pseudo" } ] }
}
```

Then run `marley: toggle fleet`. The panel opens in the right dock, beside the Agent Panel; the
server icon in the status bar toggles it too. Each agent's row shows:

- its runtime's mark (Claude Code, Codex and the others) and its name;
- its work item's key and its phase as `name n/m`, such as `RB-142 · code 2/4`;
- a yellow warning mark when it waits on a question, a red mark when its run failed;
- a chip with its state: `working`, `waiting`, `idle`, `starting`, `error` or `done`, or `stale`
  when it has not reported for three of the store's polls.

The pseudo provider serves three agents on two hosts and moves on its own: the working agent's
run takes a phase a minute, and docs-1 stops reporting after a few seconds, so it reads `stale`.
With no provider set, the panel says "The fleet is not set up." The panel reads the providers
only while it shows.

Click an agent to select it (#608). The panel splits, with the list above and the agent's
snapshot below:

- its name and state chip, what runs it (runtime and model) and for how long it has been in that
  state, and its host and folder;
- its work item's key, title and status;
- its run as a strip of phases: green for passed, blue for active, red for failed, grey for
  still to come, with the active phase (or the one that failed) named under it;
- its host's CPU and memory as bars with their numbers;
- its tokens today, in and out;
- its question and the options, when it asks one (answering comes later).

Up and Down move the selection while the panel has the focus. Drag the line between the list and
the snapshot to give either more room.

Double-click an agent, press Enter on the selected one, or click Open in its snapshot, to open
its Agent tab in the center (#609). Opening it again brings the same tab forward. The tab shows:

- its run's phases on a timeline, each bar from its start to its end, with its gates under it
  (a failed gate's message in red);
- its events, newest first;
- its host's CPU, memory and network as lines over the last 30 minutes Marley watched;
- its tokens for this run and for the day, with cache reads;
- the other agents on its host: click one to open its tab.

The fleet keeps reading while the panel shows or an Agent tab is in front. The graphs start
empty after a restart.

### Hosts over SSH (#610)

Marley can read your hosts itself. List them in the settings:

```jsonc
"marley": {
  "fleet": {
    "hosts": [
      { "ssh": "me@build-1", "name": "build-1", "id": "host-build-1" },
      { "local": true }
    ],
    "agent_processes": ["aider"]
  }
}
```

Every 5 seconds while the panel shows (or an Agent tab is in front), Marley runs a small script
on each host over SSH, the script sent in the command, so nothing is installed there. It needs a
key or an agent: SSH never prompts for a password here. `{ "local": true }` runs the same script
on this machine.

- Each host's header shows a line with its CPU, memory, disk and network.
- `claude` and `codex` processes on a host (and names in `agent_processes`) show as agents. One
  that belongs to a store's agent joins it: set `MARLEY_FLEET_SESSION` to the agent's id when the
  agent starts, or give the host the store's `id`. Its snapshot then shows the host's numbers
  and the process. The others are listed under **Hosts** as `running`, with their folder, pid,
  CPU and memory.
- A host that does not answer reads `unreachable`; point at the chip for SSH's error. The
  store's agents on it read `offline`.
- A destination that starts with `-` is refused, and nothing runs.

### Real stores (#611)

A workflow store that answers Marley's contract (`docs/marley/fleet-contract.md`) is a provider
beside or instead of the pseudo one:

```jsonc
"marley": {
  "fleet": {
    "providers": [
      { "kind": "mcp", "name": "brain", "command": "rustal-brain", "args": ["mcp"] },
      { "kind": "mcp", "name": "brain over HTTP", "url": "http://127.0.0.1:3000/mcp", "bearer_env": "BRAIN_TOKEN" },
      { "kind": "http", "name": "ci", "url": "https://ci.example.com", "bearer_env": "CI_TOKEN" }
    ]
  }
}
```

`bearer_env` names an environment variable that holds the token; Marley sends it as a bearer
and never writes it to a log. Each store's header says how it stands:

- `ready`: it answered;
- `connecting`: its first answer has not come;
- `unreachable`: it failed, with the cause (point at it for the whole message); Marley tries
  again after 1, 2, 4, 8, 16, then every 30 seconds, and its agents read `offline`;
- `stale`: no answer for three polls; its last list stays;
- `incompatible`: it speaks another version of the contract, named.

One store's trouble leaves the others as they are.

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
| Ctrl+Alt+V | Anywhere, while Rusty is connected | The rail's Projects and Brain views |
| Ctrl+Alt+U | Anywhere, while Rusty is connected | `rusty: open page`, the page picker |
| Alt+Left, Alt+Right | A brain page's tab | Back, Forward |
| Ctrl+? | Anywhere | Focuses the Agent Panel (Zed's) |
| Ctrl+\` | Marley layout | Switches between the code and the project's terminals |
| Ctrl+J | Marley layout | The same, while the bottom dock is closed |
| Ctrl+~ | Marley layout | A new center terminal |
| Up, Down, Home, End | Rail | Moves the highlight |
| Enter | Rail | Opens the highlighted row |
| Left, Right | Rail | Folds a project or climbs to it; unfolds |
| Ctrl+F | Rail | The filter |
| Escape | Rail filter | Clears the filter; a second press goes back to the rows |
| Up, Down, Home, End, Left, Right, Enter | Brain view | Move in the tree, open and close folders, open a page |
| Enter, Escape | Brain view's search field | Searches, or opens the selected hit; clears, then goes back to the tree |
| Ctrl+Tab, Ctrl+Shift+Tab | Rail, Agent Panel | The switcher; hold Ctrl, press Tab or Shift+Tab to move, let go to open |
| Ctrl+Up, Ctrl+Down | Terminal | The previous or next block |
| → | Terminal, at a prompt | Takes the autosuggestion |
| Ctrl+G | Terminal running an agent CLI | Rich input |
| Enter, Shift+Enter, Escape | Rich input | Sends; adds a line; closes and keeps the draft |
| Ctrl+C | The shell's prompt editor | Empties it |
| Ctrl+Shift+Enter | The shell's prompt editor, or a terminal at a shell prompt | Asks the agent with a line in words; a command's key goes to the shell |
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
| Enter, Escape | A paused click's card, once clicked | Allow; Refuse |
| Alt+Down, F4, Space | A focused select list | Opens Marley's list |
| Up, Down, Enter, Escape | An open select list | Moves; chooses; closes |

Commands with no key of their own, from the command palette:

| Command | What it does |
|---|---|
| `marley: use marley layout`, `marley: use zed layout` | Switch the layout in every window |
| `marley: open guide` | The Marley guide, a page shipped with Marley, in a Browser tab or the system browser (#599; also the `?` in the title bar) |
| `marley: open settings` | The Settings window on its Marley page |
| `marley: new agent` | The New Agent picker (also Ctrl+Alt+N) |
| `marley: attach file` | Attach File for the focused terminal |
| `marley: toggle dictation` | Starts or stops a Voxtype dictation, while Voice is on |
| `rusty: toggle knowledge panel` | Shows or hides the Knowledge panel, while Rusty is on (#646) |
| `rusty: open graph`, `rusty: open local graph` | The Graph tab, on the vault or on the page in front (#647) |
| `rusty: open tasks` | The Tasks tab, Rusty's to-do lists (#658) |
| `rusty: open decisions` | The Decisions tab, Rusty's decisions and the follow-ups due, each followed up from its row (#659, #660) |
| `rusty: toggle page outline` | Hides or shows the outline column of the brain page in front (#656) |
| `marley: open browser` | Shows or opens a Browser tab |
| `marley: new browser tab` | A new Browser tab (Ctrl+T inside one) |
| `marley: pick element`, `marley: annotate`, `marley: record this` | The Browser tab's three buttons, while a Browser tab has the focus |
| `multi workspace: next project`, `previous project`, `next thread`, `previous thread` | Walk the rail |

## Settings

Marley keeps its settings in `~/.config/marley/settings.json`, in Zed's format (Ctrl+Alt+, opens
the file; Ctrl+, opens the Settings window). `marley: open settings` opens the Settings window on its Marley page,
first in the list (#515). The page has a Layout section, with the layout as a dropdown and the
Prompt Editor toggle (#627), an Agents
section with Redact Secrets for Agents (#516), Codex App Server (#650) and Agent Prompts in a Tab (#649), an Agent Versions section with Prompt Tags on
Untested Claude Code (#648) and App Server on Untested Codex (#650), a Voice section with the Voice toggle (#642), a Rusty
section with Rusty's switch, its connection and Rusty Tools for Agents, and the Rusty's Server page
(#643), and a Privacy section with the two telemetry toggles; later Marley settings add their
sections there.

Marley's own keys in the file:

```jsonc
{
  "marley": {
    // "marley" (the default) or "zed". marley: use marley layout and use zed layout write it.
    "layout": "marley",
    // The shell's prompt editor at every prompt (true, the default), or only on Ctrl+G (#627).
    "prompt_editor": true,
    // Hide secrets in what Marley's tools give agents (true by default), and more to hide: each
    // entry is a regular expression, and one that does not compile is named in a notification.
    "redact_secrets_for_agents": true,
    "redaction_patterns": ["INTERNAL-[0-9]{6}"],
    // What runs on an agent version Marley has not tested it on (#648): claude_prompt_tags, the
    // tags that tell your prompts from the ones Claude Code adds, and codex_app_server (#650).
    // Off by default.
    "allow_untested_versions": { "claude_prompt_tags": false, "codex_app_server": false },
    // Agent prompts in a tab (#649): the agent's own editor key opens its prompt in a Marley tab.
    "agent_editor_in_tab": false,
    // Codex's own App Server (#650): the rail reads each launched Codex's thread. Off by default.
    "codex_app_server": false,
    // Dictation through Voxtype: the agent bar's microphone and marley: toggle dictation. Off
    // until you turn it on (#642).
    "voice": { "enabled": false },
    // Rusty (#643), off until you turn it on: "embedded" starts rusty-mcp, "service" connects to
    // Rusty's running service; agent_tools also offers Rusty's tools to Zed's agents (#633).
    "rusty": {
      "enabled": false,
      "connection": "embedded",
      "service_url": "http://127.0.0.1:4174/mcp",
      "agent_tools": false
    },
    // Where the Fleet panel's agents come from (#607): [] (the default) shows "not set up";
    // { "kind": "pseudo" } shows Marley's example data. "hosts" lists machines Marley reads
    // over SSH or locally (#610), "agent_processes" more process names to list as agents.
    "fleet": { "providers": [], "hosts": [], "agent_processes": [] }
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
| `MARLEY_SYSTEM_ONE_KEY` | Marley | The System One layer's key; it comes before the keyring |
| `MARLEY_CLOUDFLARE_API_TOKEN` | Marley | The `cloudflare` provider's API token (#548); it comes before the keyring |
| `MARLEY_RH` | Marley | The harness's `rh` to run when `embedded_harness` is on; when set, the PATH is not searched |
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
| `~/.local/share/marley/system_one/` | The System One layer's calls, a file a day (mode 0600), and `replay.jsonl` |
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
- **A terminal shows no blocks.** Blocks need Marley's integration: a bash, zsh or fish that
  Marley started in a local interactive terminal. `echo $MARLEY_SHELL_INTEGRATION` prints `1` in
  one. Tasks, remote terminals and a shell configured with its own arguments run without it, and
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
| `just install [--prefix DIR] [--regress]` | The release `marley`, installed with its menu entry; `--regress` checks it with the golden set first |
| `just gate-diff` | Every gate on the change, and the receipt a commit needs |
| `just gate-fast` | The same gates without a receipt, for a change with no Rust |
| `just clippy <crates>` | Clippy on the named crates, every target, warnings as errors |
| `just fmt <crates>` | Formats the named crates |
| `just e2e <scenario>` | A ticket's e2e scenario (`script/e2e/`) against the debug build, on a hidden workspace, or in a headless sway of its own when it clicks |
| `just regress [scenario...]` | Marley's regression suite: the golden set in `script/e2e/golden` (or the scenarios named), each in a headless sway, each checking itself; a PASS or FAIL line per scenario and a verdict. `E2E_BINARY=<path>` runs another build. The runs go under `~/.local/state/marley/regress/` |
| `just shot <name> [seed]` | One shot of the debug Marley on a copy of your profile; `OPEN=<path>` opens a path |

Work moves through four phases, Plan, Code, Test and Complete (`/pipeline:plan`,
`/pipeline:code`, `/pipeline:test`, `/pipeline:complete`), under `CONSTITUTION.md`. A change is
proven by the static gate, a review of its diff, and a visual check: an e2e scenario that starts
the real Marley and drives what the ticket changed, every shot read. Since 2026-09-29 no ticket
writes unit tests or runs the golden set's regression; those return in a testing phase at the
end. Every change outside `crates/marley_*` and Marley's other owned paths gets a row in
`docs/marley/zed-touchpoints.md`.

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

Waiting on purpose: Marley's own release identity, with its keyring label, updater, app id and
URL scheme (TICKET-445).

`docs/marley/three-prong-plan.md` names the longer road: block-scoped path links and a jump to the
first failure (T2), a prompt editor (T3), tasks and runnables as blocks (T4), native block headers
with the shell's prompt hidden (T5), completions in the prompt (T6), and the control plane that
brings harness seats and Rusty's sessions into the rail (C1 to C5).
