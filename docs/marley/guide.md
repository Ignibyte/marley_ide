# Marley: the guide

Marley is Ignibyte's fork of the Zed editor. It keeps everything Zed gives you (the editor,
language support, projects, settings, themes, the Agent Panel) and adds a layout built around
terminals, a block terminal, agents recognized in any terminal, an MCP server that lets agents
read your terminals and your browser, and a Browser tab that you and your agents drive together.

This guide describes Marley as of 2026-10-12 (through #778) on Linux, where it is built and
tested (Omarchy on Hyprland). Since #786 and #787 it also builds and installs on an Apple
Silicon Mac; see [On a Mac](#on-a-mac-786-787) for what it does and lacks there. A number such
as #496 is the ticket that shipped a feature; its entry in `CHANGELOG.md` says more. Marley is
pre-1.0.

- [What Marley adds](#what-marley-adds)
- [Install and run](#install-and-run)
- [The Marley layout and the rail](#the-marley-layout-and-the-rail)
- [The block terminal](#the-block-terminal)
- [Agent CLIs in terminals](#agent-clis-in-terminals)
- [Zed's Agent Panel with Marley's tools](#zeds-agent-panel-with-marleys-tools)
- [Rusty](#rusty)
- [MCP Servers](#mcp-servers)
- [The rustal-ste skill for Marley's agents](#the-rustal-ste-skill-for-marleys-agents)
- [The Marley agent](#the-marley-agent)
- [Marley's MCP server](#marleys-mcp-server)
- [The Browser tab](#the-browser-tab)
- [System One](#system-one)
- The harness's sessions
- [The Fleet panel](#the-fleet-panel)
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
- For the remote server the install builds: `musl-gcc` and `llvm-objcopy` (Arch's `musl` and
  `llvm` packages). The `x86_64-unknown-linux-musl` target is added through rustup when missing.

### Install with `just install`

From the checkout, run `just install`. It waits until no other cargo runs on the machine (the
target directory is shared by every project on the dev box), builds `marley` in the release
profile and installs it under `~/.local`. `just install --regress` first runs Marley's golden set
of e2e scenarios against that build (#517, about an hour; see [For developers](#for-developers))
and installs only when every scenario passed. `just install --prefix DIR` installs somewhere
else. Each install also builds Marley's remote server and stages it (see
[Remote projects over SSH](#remote-projects-over-ssh-445)); `script/install-marley
--remote-server-only` stages the server and builds or installs nothing else.

| Path under the prefix | What it is |
|---|---|
| `bin/marley` | The launcher: the command you run, and what the menu entry starts |
| `lib/marley/marley` | The binary |
| `share/applications/marley.desktop` | The menu entry, "Marley" |
| `share/icons/hicolor/512x512/apps/marley.png` | The icon, Zed's dev-channel icon for now |

The first release build takes about ten minutes on the dev box; a rebuild with nothing changed
takes under a second. The binary is about 2.1 GB because it keeps its debug information, which
gives a backtrace its file and line numbers. When it finishes, the installer prints the commit it
installed (and says so when the tree had uncommitted changes), the staged remote server's path,
the menu entry's path, a warning when a Marley is running, and a note when the prefix's `bin` is
not on your PATH. The entry claims no file types and no scheme but `marley://` (#762), so Marley
never becomes the default app for text files, folders or stock Zed's `zed://` links.

Marley's windows have the class `com.ignibyte.Marley` (#760; stock Zed's is `dev.zed.Zed`, and a
Marley installed before #760 had `dev.zed.Zed-Dev`), which is what a Hyprland window rule matches.
Its About window and a window title's `${appName}` say Marley. Marley stays on Zed's `dev`
channel, so its database (`db/0-dev`), its settings and its credentials stayed where they were when
it took its own name. What it keeps in the system keyring is labelled Marley (#761; see
[Turning it on](#turning-it-on) under System One).

### Remote projects over SSH (#445)

Open Remote Folder (in Add Project's popover), `ssh_connections` and `marley ssh://user@host/path`
open a folder on another machine as Zed does: Marley runs its remote server there and edits,
searches and runs terminals through it. Marley ships on Zed's `dev` channel, where Zed downloads
no server, so the install builds one:

- **What is staged.** Every `just install` builds `remote_server` from the same tree for Linux
  x86_64 (static, musl), strips and gzips it, and stages it as
  `~/.local/share/marley/remote_servers/dev/linux-x86_64/<commit>.gz`. The newest three are kept.
  The build adds several minutes to an install, and more the first time.
- **On the host.** Marley's server lives at `~/.zed_server/zed-remote-server-dev-<commit>`, the
  full commit Marley was built from. Marley uses a server there only when its `version` answers
  that commit, and otherwise uploads the staged one over SSH and unpacks it in its place. Each
  Marley commit has its own server, and the server removes the others it finds idle when it
  starts.
- **When nothing is staged** for the host's platform and Marley's commit, the connection fails
  with a message naming the platform, the commit and `script/install-marley
  --remote-server-only`. Run that in the checkout at the commit Marley was built from.
- **Hosts:** Linux x86_64 only for now. A macOS or aarch64 host gets the same message.
- **Dev containers** take the same staged server through the same lookup. No run has opened one
  yet, and #445's scenario reached its hosts through an sshd on the same machine.

### Start it and update it

Start Marley from your desktop's app menu, or run `marley`. When stderr is not a terminal, as
when the menu starts it, the launcher appends stderr to `~/.local/share/marley/logs/stderr.log`,
with a line for each start, and keeps one older log as `stderr.log.old` once the file passes
10 MiB. On Marley's `dev` release channel a panic is printed to stderr and nowhere else, so this
file is the only record of one; `Marley.log` says nothing about it.

To update, pull and run `just install` again. Each file is replaced by a rename, so a Marley that
is running keeps its old build until you quit it and start it again. Marley never checks zed.dev
for an update. Check for Updates (`auto update: check`) says so (#763): "Marley updates through
its install script", with what to run.

### On a Mac (#786, #787)

Marley builds for Apple Silicon Macs since #786, and `just install` on a Mac installs it as an
app (#787). It needs full Xcode (the build compiles Metal shaders), `cmake`, `just` and rustup,
which fetches the pinned toolchain. The Mac installs from its own checkout, and Linux keeps the
visual checks: none of the e2e scenarios runs on a Mac.

- **What it installs.** `~/Applications/Marley.app`, which `--app-dir DIR` moves:
  - the identifier `com.ignibyte.Marley`, the `marley://` scheme and no claimed file types;
  - Zed's privacy strings with Marley's name;
  - a Local Network reason;
  - Zed's dev icon;
  - an ad hoc signature.

  The launcher is `~/.local/bin/marley`, and `--prefix DIR` moves it. When stderr is not a
  terminal, it appends stderr to `~/Library/Logs/Marley/stderr.log`, as Linux's does to its log.
- **The build.** On a Mac with under 32 GB, the release build turns off the profile's thin LTO,
  uses 16 codegen units and runs a third as many jobs as there are gigabytes, at low priority.
  It says so, and `CARGO_PROFILE_RELEASE_*` or `CARGO_BUILD_JOBS` in the environment win. The
  first build took 11 minutes on a 16 GB M4, and the binary is about 600 MB.
- **Remote projects on a Linux host.** A Mac builds no Linux server. Run the install with
  `--remote-server-from HOST`, a host whose own install staged the server for the same commit,
  and the install copies it into `~/Library/Application Support/Marley/remote_servers` before it
  builds. When HOST has none, the install stops at once and says what to run there:
  `script/install-marley --remote-server-only`, in its checkout at that commit.
  `--remote-server-only --remote-server-from HOST` does only the copy.
- **Local Network.** macOS lets a new app, and the ssh it starts, reach a machine on your local
  network only once you allow it. Until then a remote folder on a LAN host fails with "No route
  to host". Allow Marley at the prompt, or under System Settings, Privacy & Security, Local
  Network. The choice follows the app's identifier, so a reinstall keeps it.
- **Where things are.**
  - Settings: `~/.config/marley`.
  - Data: `~/Library/Application Support/Marley`.
  - Logs: `~/Library/Logs/Marley`.
- **What a Mac lacks so far:**
  - the Ports list (#788);
  - the agent editor, whose helper cannot sit under `Application Support`'s space (#790);
  - Cmd for Marley's own keys (#789);
  - agent reports' process chain and stall checks;
  - the Browser tab's Chromium;
  - `marley <command>`;
  - Codex's app server.

  All of these need `/proc` or systemd today.

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
- The hand-off goes through a socket in the data directory, `marley-dev.sock` since #760
  (`zed-dev.sock` before). A Marley started from an older build listens on the old name, which a
  new build does not look for, so quit the running Marley before you start the first build past
  #760.

### `marley://` links (#762)

`marley://` is Marley's own scheme: a `marley://` link means what the `zed://` link of the same path
means (`marley://settings` opens the Settings window, `marley://open` brings Marley forward). The
install's desktop entry claims `x-scheme-handler/marley`, and only that, so `xdg-open marley://…`
starts Marley, which hands the link to the Marley that runs.

A `zed://` or `marley://` link clicked inside Marley opens in Marley: Ctrl-click or Open URL in a
buffer, a link in the markdown preview, an agent's message, a hover or a notification, and a
Ctrl-click in a terminal. Outside Marley, the desktop still sends `zed://` to stock Zed.

### `marley <command>` (#749)

`marley` followed by a command opens something in the running Marley, working in the folder you
run it from. `marley help` lists them:

| Command | Opens |
|---|---|
| `marley agent-thread [--provider=NAME] [--folder=PATH]` | an agent thread in a tab, working in the folder |
| `marley agent-cli [--provider=NAME]` | an agent CLI in a new terminal in the folder |
| `marley terminal [PATH]` | a terminal in PATH, else in the folder |
| `marley talk marley` or `marley talk rusty` | a conversation with the Marley or the Rusty agent |
| `marley threads` | the Threads page |
| `marley home` | the Home page |
| `marley rusty [SCREEN]` | the Home page, or `brain`, `today`, `graph`, `tasks`, `decisions`, `memory`, `skills` or `secrets` |
| `marley panel NAME` | `project`, `git`, `outline`, `collab`, `fleet`, `knowledge` or `containers` as a tab, or `debug` in the bottom dock |
| `marley browser URL` | a Browser tab on the URL, or on a file in the folder |

- **Where it opens.** In the group whose project holds the folder (the deepest one when projects
  nest), else in the window's Default group. The window comes forward and shows that group.
- **`--provider`.** For a thread: `claude`, `codex`, `zed`, `marley`, `rusty`, or an agent
  server's id or name as `agent_servers` gives it. For a CLI: `claude`, `codex`, `gemini` or
  `opencode`. Without it, the agent you started last that way, from New Agent, Home, the rail or a
  command. An agent CLI starts with the permission mode its settings give, as New Agent starts it.
- **Errors.** A wrong argument prints the error and exits with code 2. What only Marley can judge,
  such as an agent no one has, Rusty off or a panel turned off, shows as a message in the window.
- **No Marley running.** Marley starts, restores your session, then runs the command. With
  Marley running, the command prints one line, "Marley is already running on …; it was asked to
  run `marley <command>`." (#775).
- **A Marley on its own data directory.** Its terminals carry `MARLEY_USER_DATA_DIR`, so a
  command typed there reaches that Marley, not the one on the default directory.
- A folder named like a command opens as a folder with `./`: `marley ./home`.

A key for a new thread, in Hyprland's `bindings.conf` (Hyprland runs it from your home folder, so
the thread lands in Default unless `--folder` names a project):

```
bind = SUPER SHIFT, A, exec, marley agent-thread
```

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

A debug build compiles the remote server with cargo when it connects to an SSH host; start it
with `ZED_BUILD_REMOTE_SERVER=never` to use the staged server instead, which then has to be staged
for the debug build's commit.

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
| Agent threads | In the Agent Panel | In tabs, never in the Agent Panel (#772) |
| Side panels (project, git, outline, collab, Fleet, Knowledge, Containers) | In their docks | Tabs in the pane you are in (#747, #748); Debug stays in the bottom dock (#774) |
| A folder you open for the first time | Zed's behavior | Starts with a terminal at its root |
| Ctrl+\`, Ctrl+J, Ctrl+~ | Zed's Terminal Panel | The center terminals |

The block terminal, the agent bar, the MCP server and the Browser tab work in both layouts. The
layout sets three defaults, `terminal.button: false`, `agent.dock: "right"` and `debugger.button:
false` (#774), under your own settings, so a value you set wins in either layout. A switch to the
Zed layout and back gives each dock the panel it showed before (#456).

Zed's Panel Layout presets, Classic and Agentic, would rewrite the docks the Marley layout sets.
In the Marley layout, choosing one shows a toast saying the presets belong to Zed's layout, with a
Use Zed's Layout button. The title bar's Panel Layout menu still lists them, with Custom checked.

With AI turned off (`disable_ai`), Zed draws no sidebar at all, the rail included.

### The rail

The header reads PROJECTS and THREADS (#745), the view showing lit, and carries Add Project (a
folder with a plus) while the projects show. Add Project opens Zed's recent-projects popover: your
recent projects, Open Local Folders and Open Remote Folder. The filter field sits under the header.
THREADS lists your conversations in the projects' place (see [Threads in the
rail](#threads-in-the-rail)). Rusty's vault no longer takes over the rail: it opens in a Brain tab
(see [The Brain tab](#the-brain-tab)).

Each project has a header row: its name as a muted label, a chevron that folds it, an attention
dot when a row it hides (folded, or filtered out) needs you, and a `+`. Under the header come the
Home page's row, first, where the group holds it (#782), then the project's terminals, its Browser
tabs, its other tabs (a panel tab, a Rusty screen, a search or a diff), its open files under a **Files** row (#674), its agent threads, newest first,
and the ports its servers listen on. A line separates one project from the next. A thread in a
tab has one row, under the group that holds the tab (#734), lit while the tab is in front (#702).

| Row | What it shows | A click |
|---|---|---|
| Terminal | A `>_` icon, the title, and the working directory relative to the project root (`~` for your home outside it, empty at the root) | Shows its project, focuses the terminal, clears its bell |
| Agent CLI | The agent's icon, the title the CLI sets (or the agent's name), and "Claude Code · working" or "Claude Code · waiting" | As a terminal row |
| Zed agent thread | The agent's icon, the thread's title, and "Zed Agent · working" (or idle, waiting, failed) | Shows the project and the thread in a tab of it, its tab forward when it has one (#772) |
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
it: its row goes, and its tab closes if it has one, in every window, and stays closed after a
restart (#777). To bring it back, right-click the project's header and choose it under Archived
Threads (#616), or open it from the Threads page's Show archived: it opens in a tab, its row comes
back, and the Threads view lists it again (#777). To delete a thread for good,
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
  logged out of it, or with a remote that is not GitHub, the row shows the counts alone. The same
  lookup reads the pull request's checks (#718), marked beside the number: a red cross when a
  check failed, a yellow dot while one is still running and none failed, a green tick when all
  passed, and no mark when it has none. The pointer adds a line such as
  `Checks: 2 failed (test, ci/external), 1 passed`.

Marley reads neither for a repository Zed does not trust yet.

**Send Failed Checks to Agent…** (#718) is in the project's right-click menu, which a right-click
on the chip opens, while a check failed; `marley: send failed checks to agent` does the same for
the project you are in. Marley asks `gh` for the pull request again, then lists the agent CLIs
working in the project's folder, each with whether it is ready, and Copy Prompt. Pick one and
Marley reads the failed steps' log of each failed GitHub Actions job (the end of each, where the
error is), writes them to one file of its own,
`~/.local/share/marley/checks/<owner>-<repo>-pr<n>-<commit>.log`, readable by you alone and
redacted while Redact Secrets for Agents is on, and pastes a one-line prompt at the agent's prompt
without pressing Enter: the pull request, the commit, the failed checks, the file, and each check
whose log it could not read (another CI's, or an expired log) with its link. Add your ask and press
Enter. An agent that waits on a question gets nothing, as with a block you send.

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

### Threads in the rail

The rail's header reads PROJECTS and THREADS (#745); the one showing is lit. THREADS, or Ctrl+G with
the focus in the rail, swaps the project rows for every saved conversation, newest first, each with
its agent's icon, its title, and its agent and folder; archived conversations and drafts are left
out. The search field ("Search conversations…") takes the filter's place and matches titles
and folders. A click opens the conversation in a tab of the group you are looking at, with its
earlier turns, or brings its tab forward where it is open. Your Claude Code and Codex CLI sessions
are listed too, and a click resumes one (#754). PROJECTS, or Ctrl+G again, brings the projects back;
while the conversations show, PROJECTS carries a dot when a project needs you. The rail starts on
the projects. For agent filters and archived conversations, open the Threads page: the Threads
button on the Home page's NEW AGENT card, `marley: open threads` or `marley threads` (see [The
Threads page](#the-threads-page)).

### The + menu

A project's `+` ("New in this project") starts things in that project.

| Entry | What it does |
|---|---|
| New Terminal | A shell in the project's directory, in the main area |
| New Browser Tab | Shows the project and opens a blank page in a new Browser tab, the cursor in the address bar, starting Chromium if it must |
| New Agent… | The New Agent picker, below, opened on this project: the one way to start an agent, a thread or a CLI. It replaced New Agent Thread and the list of agent CLIs (#744) |
| Reopen Thread… | A picker over your saved conversations ("Reopen a conversation…"), newest first, each with its agent, folder and time; Enter opens one in a tab here, with its earlier turns, or brings its tab forward (#772) |
| New Agent in Worktree | For a local git repository, a submenu of the installed agent CLIs (see [Worktree agents](#worktree-agents)) |
| Launch | The project's launch configs, when it has any (see [Launch configs](#launch-configs)) |

A group with no folder has the first four. Ctrl+Alt+N opens the New Agent picker ("Start an
agent…") in either layout. So does every pane's `+` (New Agent… at its head, then Reopen Thread…),
and in the Marley layout the status bar's sparkle, Ctrl+? and Zed's `agent: new thread` too, in
place of Zed's Agent Panel, which starts nothing in a group with no folder (#744). Zed's agents are marked Thread and the CLIs Terminal, and typing filters the list.
After the agent, it asks where (#735):

- **The project's root** comes first in a group with a project, marked "this project" (#744), so
  Enter twice starts the agent in the project.
- **The guess** follows when it is another folder: the active terminal's folder, else the active
  file's project. In a group with no folder it comes first, else your home folder.
- Then the window's open projects, your recent projects, and **Browse…**, Zed's own folder prompt
  (a file picked there means its folder).
- A thread opens in a tab of the shown group (#734), a CLI in a new terminal there, each working in
  the folder you picked.
- **On box-2…** (#741), for Claude Code and Codex, starts the agent on a harness Marley follows
  (see "Several harnesses"), one entry per harness that takes a seat, **On the harness…** for the
  first. Type the folder on that machine, a full path such as `/srv/project`, and Enter: Marley
  adds and starts a seat there named for the agent and the folder (`claude-project`, then
  `claude-project-2` …), then opens a terminal of the shown group attached to it, through `ssh -t`
  for a harness on another host. The agent runs under the harness's supervision and keeps running
  when the terminal or the link closes; its row is under the harness's section in the rail. A
  refusal, such as a folder the host does not have, comes up as a notification with the harness's
  reason, and no terminal opens.
- For **Claude Agent**, a thread, **On box-2…** makes the same Claude Code seat and opens it as a
  thread tab instead (#742): what you type goes to the seat as a message, and its replies come back
  as the thread, through the harness's window onto the seat (`rh acp --seat NAME`, run through
  `ssh` for another host). The seat keeps running when you close the tab. The thread is filed under
  the seat's folder, and its agent is named `claude-web · box-2`.

**Open Agent Here…** asks only which agent: on a terminal's row in the rail, it starts in the
terminal's folder; in the project panel's right-click menu, in that folder, or a file's folder.
`marley::NewAgent` takes the folder too: `["marley::NewAgent", {"folder": "/srv/stacks/x"}]`. With
AI turned off the key opens nothing.

A folder you open for the first time in the Marley layout starts with a terminal at its root,
focused (#455). A project you opened before comes back as you left it, with its saved terminals
or none.

### Groups with no folder (#600)

A right-click on the rail's empty space (under the last row, or beside PROJECTS and THREADS while
the projects show) opens a menu:
New Group… asks for a name and makes a group with no folder, listed after the window's projects
with a group icon, a chevron and a `+`. A group's terminals and agent CLIs start in the home
folder, and its Browser tabs use a Chromium of the group's own. New Terminal, New Browser Tab and
the agent CLIs in the same menu open in the window's Default group. A group's `+` has no New
Agent in Worktree or Launch, which need a folder; its New Agent… starts a thread in a tab of the
group, or a CLI in a terminal, in your home folder (#734, #744). Right-click a group's header for
Rename Group… and Remove Group. Groups come back after a restart with their
names, order, terminals and Browser tabs (#601).

Every window has its Default group from the start (#700; called Home until #743, which kept that
name for the Home page). It holds what belongs to no project. The rail lists Default, then your
projects, then the groups you named; a header you dragged keeps its place. A window started with no
folder and nothing to restore shows Default. Default is always there, so its header menu has
nothing to rename or remove: it reads "What belongs to no project opens here". There is no Rusty
group since #743: Rusty's screens open where you are (see [Rusty on Home](#rusty-on-home)), and a Rusty group an earlier build made closes when it comes back after
a restart; a window that showed it shows Default.

Default's first tab is the Home page (#701), never Zed's Welcome page: when Default is shown without
it, or its last tab closes, the page comes back. The status bar's Home button opens the same page,
a tab titled Home, in the group you are looking at, or brings it forward where that group has it
(#739); `marley home` opens it in the group of the folder you run it from (#749). Its buttons work
in the group that shows it.

- **Start:** New Terminal (a terminal in the page's group, in your home folder in Default), Open
  Folder…, Clone Repository… and Command Palette.
- **New Agent:** New Agent…, the picker that asks which agent and where (#735); Threads, which
  opens the Threads page (#745); then a button for each agent CLI on your PATH (Claude Code, Codex,
  Gemini CLI, OpenCode), started in a terminal in the page's group.
- **Recent Projects:** the projects you opened lately, each with its folder; a click opens one.
- **Agents at Work:** every agent thread and agent CLI in the window that is working, waiting or
  failed, with its project; a click opens it where it lives. It follows the rail, also after a
  switch to Zed's layout and back (#713).
- **Rusty**, while Rusty is on (#782): Rusty's screens, Open Page… and the captures, then Recent
  Pages, Follow-ups Due and Tasks (see [Rusty on Home](#rusty-on-home)). While Rusty is on but not
  connected, one Rusty card says why; while it is off, Home shows nothing of Rusty.
- **Agent Activity:** Stop or Resume for Marley's tools, the five newest calls agents made to them,
  and Open Agent Activity (#703; see [Agent activity and the kill
  switch](#agent-activity-and-the-kill-switch)).
- **Configure**, last since #782: Settings, Marley Settings, Keymap, Extensions, MCP Servers (#720)
  and the Marley guide.

In the rail, Home's row is always the first under Default, above its terminals (#782).

### Drag to reorder (#602)

Drag a project's or a group's header up or down to move it, with its rows, among the headers; a
group can sit between projects. Drag a terminal, Browser tab or thread row up or down to move it
among the rows of its kind in its own group (a worktree's terminals among that worktree's). A
card with the name follows the pointer, and a line shows where it lands: above the target when
you drag up, below it when you drag down. A terminal row dropped on another open project's or
group's header moves to that project, with its shell still running (#613); its right-click
menu's Move to Project does the same. Any other row dropped on another group, a header or
outside the rail goes back. The rail holds still while you drag, and a press that barely moves is
still a click. The order is saved with the window and comes back after a restart; under the attention
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

### Splits (#785)

In the Marley layout a split never starts a terminal you did not ask for. Splitting a terminal or
an agent's terminal, with the pane's split button, a split key or `pane: split right` and its
siblings, opens Home in the new pane, where its cards start a terminal or an agent if you
want one. Splitting Home opens a second Home, and a file still opens in both panes, as in Zed.

Right-click any tab for Split Right, Split Left, Split Up and Split Down: the tab moves into a new
pane on that side. When it is the pane's only tab, it stays where it is and Home opens on the
other side, so the tab still ends up on the side you picked. The Zed layout keeps Zed's splits.

### Side panels as tabs (#747)

In the Marley layout the project panel, the git panel, the outline panel, the collab panel, Fleet,
Knowledge and Containers open as tabs in the pane you are in, not in the right dock (#747, #748).
Only the Agent Panel stays in the right dock, and Debug in the bottom one. Their buttons at the
right of the status bar, their keys (Ctrl+Shift+E, Ctrl+Shift+G, Ctrl+Shift+B), `marley panel NAME`
(#749) and Zed's own ways to them, such as Reveal in Project Panel, bring the panel's tab forward
where it is, or open one. A tab is titled as its panel's button: Project Panel, Git Panel, Outline
Panel, Collab Panel, Fleet, Knowledge, Containers. Each sits in the rail like any tab.

- **One of each per group.** Asking again brings the same tab forward. Drag a panel tab to a pane's
  edge, or use the split keys, to put it in a pane of its own: it moves, it is not copied.
- **The outline** keeps listing the symbols of the editor you were in last, while its own tab, the
  Project Panel tab or a terminal has the focus (#773).
- **The docks.** A dock that opens on a moved panel by itself, such as the project panel Zed
  opens on a new project, closes again and makes no tab. With the Agent Panel open in the right dock, a
  panel's button opens its tab and leaves the Agent Panel where it is.
- **Fleet and Knowledge** keep working while their own tab is in front: Fleet keeps reading its
  providers, Knowledge keeps the page it follows. A panel turned off (Knowledge while Rusty is off,
  Containers while hidden) closes its tab.
- **Debug** stays in the bottom dock (#774). Its button is the first one at the left end of the
  status bar: a click opens the bottom dock on the Debug panel, and a second click closes it.
  Right-click it for Hide Button, which sets `debugger.button` to false. Setting
  `debugger.button` to true puts Zed's own button back among the dock buttons at the right.
- **Zed's layout** keeps its docks: switching to it closes the panel tabs, and the Debug button
  goes back among the bottom dock's buttons.
- **After a restart** each panel tab is back in its pane, beside the editors that come back there
  (#773), unless its panel is turned off by then or the window starts in Zed's layout.

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

### Earlier output after a restart (#751)

A local terminal that Marley restores after a restart shows the output it held before, dimmed,
above the new shell's first prompt. Each local terminal keeps its newest 5,000 lines (at most
8 MiB) in `<data dir>/terminal_history/<terminal id>.log`, a file only you can read (mode 0600, in
a folder of mode 0700), written within a second of the output and at the quit.

- **Text and colour only.** The file keeps what draws: text, colours and styles, cursor motion,
  erasing and scrolling. Every terminal query, mode change, title, notification and clipboard
  escape is left out, so nothing in the old output answers into the new shell.
- **Blocks come back too (#764).** Where each command started and ended is kept as a mark of
  Marley's own beside the text, with the command, its exit code, its folder and its times. The
  old commands come back as blocks over their output: the bar, the exit pill, and a header that
  says `restored` where the branch goes. A restored block is history. It offers no Rerun or
  Reinput, no Ask the agent and no English hint, publishes no diagnostics, and is not the
  terminal's last command in the rail or for System One; `terminal_blocks` marks it `restored`.
  A command still running at the quit comes back without an exit pill. The first command you run
  makes a live block as usual.
- **Full-screen programs.** What a program draws on the alternate screen (`less`, `vim`, `htop`)
  is not kept, so a program open at the quit leaves the lines before it and none of its screen.
- **Going away.** Closing a terminal deletes its file. Closing a window or quitting keeps the
  files of the terminals Zed brings back. Restore Scrollback on the Marley page's Terminal
  section (`marley.restore_scrollback`, on by default) turns it off and deletes every file.
- **What it holds.** The file keeps what was printed, a token you echoed included, until it
  scrolls past the 5,000 lines or the terminal closes. Zed's own Clear (`terminal: clear`) does
  not reach the file, so a restore shows the output it cleared, with its blocks. Tasks, remote
  terminals and the Agent Panel's terminals keep nothing.

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

**Codex and OpenCode** (#552). While Codex runs, its agent bar offers Turn on Codex notifications,
which sets three keys under `[tui]` in Codex's `config.toml`; while OpenCode runs, Connect OpenCode
to Marley writes Marley's plugin into OpenCode's plugin folder. Marley writes these files the way
another program's files should be written (#755): a `config.toml` that is a symlink stays one and
its target is written, the file keeps its mode, the old text stays in `config.toml.marley-backup`
beside it, and when the file changed while Marley wrote it, nothing is written and Marley says so.

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

When the version found is older than a part you use needs (#720), say Claude Code 2.1.280 while
the shared plugin wants 2.1.287, a notification names the program, its version, each part with
the version it needs, and the update command (`claude update`, `codex update`), with Open MCP
Servers. It shows once per program and version, so staying on an old version does not bring it
back at every start. A version newer than the tested range gets no notification: only Marley can
change that, so the chip stays the only sign.

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

Up on the editor's first line shows the shell's older commands, newest first, and Down brings back
newer ones and then what you were typing (#783). The shell's line keys work on the editor's line
(#784): Ctrl+W deletes a word, Ctrl+E goes to the end, Ctrl+B back a character, Alt+B and Alt+F
back and forward a word. Ctrl+R, Ctrl+O and Ctrl+Q do nothing in the editor; Ctrl+Q no longer
quits Marley from a terminal. Ctrl+Shift+V pastes into the editor, and Ctrl+Shift+C copies its
selection, or the terminal's when the editor has none. The same keys behave the same way in an
agent's rich input, where an image pasted with Ctrl+Shift+V goes to the agent.

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

Choose New Agent… in a project's `+` menu, or press Ctrl+Alt+N, and pick an entry marked Terminal,
then where it starts: the project's root first, then the active terminal's folder, the open and
recent projects, and Browse… (#735, #744; see "The + menu"). **Open Agent Here…** on a terminal's
rail row, or Open Agent Here on a folder in the project panel, skips the question and starts it in
that folder. Home's button for each CLI, the CLIs in the rail's empty-space menu and
`marley agent-cli` start one too. Marley opens a center terminal in the folder and, once the shell
is ready (or after five seconds), types the program's name, with any arguments the settings below
add, and Enter. The shell outlives the CLI.

After a restart, an agent's terminal comes back as a shell in its old directory, with its earlier
output dimmed above the new prompt (#751). A Claude Code that was running at the quit comes back
too, with `claude --resume` and its session, when Marley's plugin is connected; a Codex that ran
on Marley's App Server comes back with `codex resume` and its thread, joining a new App Server
while `marley.codex_app_server` is on (#750). One you ended yourself does not, and Resume Agent
Sessions (`marley.resume_agents`) turns resuming off.

When the quit cut an agent's turn off (it was working or waiting), its rail row shows
**Interrupted · Continue** under its state. Continue types "Continue where you left off." and
Enter once the agent waits at its prompt; while it is doing anything else, a toast names what it is
doing and nothing is typed. Marley never continues a turn by itself, and the chip goes once the
agent starts a turn or ends.

A session you ran earlier, in any folder, opens again from the Threads page, the rail's Threads
view or Reopen Thread…: Marley brings forward the terminal that runs it, or runs
`claude --resume <id>` or `codex resume <id>` in a new terminal in its folder (#754; see "The
Threads page").

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

The same two settings start agent threads too (#779), in a tab or in Zed's Agent Panel: a new
Claude Agent thread starts in Bypass Permissions, a new Codex thread in Full access, and so do the
Marley and Rusty agents, in the mode of the agent they run on. The thread's mode switch sits at
the bottom right of its message box, beside the send button: click it to pick a mode, or press
Shift+Tab in the box to move to the next one. The Marley and Rusty agents offer Bypass Permissions
there too, whatever the setting says; they still never edit files or run commands. A mode you
pick applies to that thread. Zed remembers it as the agent's default, but while the setting asks
for bypass or full access, the next thread starts there again; with the setting at ask, threads
start in the mode you last picked, as in Zed. The per-project entries do not reach threads: a
thread takes the defaults.

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

**Claude Code on your phone** (#757), Claude Code Remote Control in the Marley page's Agents
section (`marley.claude_code_remote_control`), off by default. On, every Claude Code Marley starts
gets `--remote-control`, so its session shows in the Claude app on your phone: from New Agent, the
English hint, a worktree, a launch config or a resume. A prompt Marley gives it follows a `--`,
since the flag takes an optional name. Its row shows a `remote` chip, read from the arguments it
runs with, so one you started by hand with the flag shows it too. Marley's own push skips the
session, since the Claude app notifies you for it; desktop notices stay. The session travels
through Anthropic's service, which is why the switch starts off. Checked on Claude Code 2.1.296.

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
It runs on Codex 0.155.1 to 0.162.0; on another version Codex starts as before, and the agent
bar's chip says which version Marley found (App Server on Untested Codex, in the Agent Versions
section, runs it anyway). Codex's approval requests (a command, a file change, extra
permissions, an MCP server's elicitation) also show in the rail's Needs you with the decisions
each offers, and Marley answers only the one you pick (#651). An MCP server that only asks to run
a tool offers Allow, Allow session or Always allow as it offers them, and Dismiss; Codex's own
question with options shows one button per option, and one with no options, a secret or more
questions shows its text and opens the terminal (#752). While it is joined, the user agent
Codex sends with its requests names Marley. A Codex you type by hand runs as you typed it.

**Usage limits** (#715). Codex's row carries its account's most-used rate-limit window on a line of
its own, `5 h 62 % · resets in 1 h 35 m` (`5 h` is the five-hour window, `7 d` the weekly one),
and the pointer on the row lists every window by name, most used first. The numbers are the ones
Codex sends with its answers, so a Codex shows them from its first turn after Marley joined;
Marley asks OpenAI for nothing. A Claude Code row shows the same line once the shared plugin
(**Shared Claude Code Plugin**, below) reports a turn's end. When a turn stops at the usage limit, the Codex
row reads `limited · resets in 1 h 35 m` in the warning color instead of `failed`, with Codex's
message on the next line, and Needs you lists it, `Limited · resets in 1 h 35 m (15:05)`, with
**Continue at Reset**. Clicked, the entry reads `Continues at 15:05` with Cancel, and at 15:05
Marley sends the thread "Continue where you left off." through the App Server, which Codex shows as
your message. Nothing is sent if a turn starts on the thread first, if Codex waits on a question,
or if the thread, the server or the terminal closes; a reset that passed while the machine slept
continues at the next check. Once the reset has passed the button reads Continue and sends at once.
A rate limit with no full window gives no reset, so the row reads `limited` and the entry has no
button. **Continue Limited Agents at Reset** (`marley.continue_at_reset`, Agents section, off by
default) arms every limited agent without the click; Cancel still holds.

A Claude Code its usage limit stops (#765) reads the same way: its turn ends with Claude Code's
`rate_limit` error, the row reads `limited · resets in 1 h 35 m · <prompt>`, and Needs you offers
Continue at Reset. The reset comes from the windows the shared plugin reports at the turn's end,
so without the plugin the row reads `limited` with no button. At the reset Marley types "Continue
where you left off." and Enter into the terminal, as Continue after a restart does, and only while
Claude Code still reads limited by the same failure; a prompt you send first, a new session or
Claude Code leaving the terminal cancels it. Whatever you left typed at Claude Code's prompt is
sent with it.

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
  shell), `agent` (`claude`, `codex`, `gemini` or `opencode`, started as New Agent starts a CLI)
  and `browser` (an `http` or `https` URL). A URL on this machine waits up to 30 seconds
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
   press Enter. To give the same task to more than one agent, tick the others under **Also
   start:** (#770): one box for each other installed agent CLI, all clear when the prompt opens.
   Each agent ticked gets its own worktree and branch from the same base, and the branches share a
   name with each agent's program after it, `agent/<name>-claude` and `agent/<name>-codex`, as the
   line under the prompt shows. The worktrees are made one after another, the chosen agent's
   first, each as step 3 says, and each gets its own row.
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

The plugin, `marley` 1.7.0, brings three things:

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

### Pushes to your phone (#535, #752)

With an ntfy server on this machine, Marley pushes one line to your phone when an agent you are not
looking at needs a permission or an answer, finishes a turn, or fails. Set the server in the Marley
page's Push section, or in `settings.json`:

```json
"marley": { "push": { "url": "http://127.0.0.1:8090", "topic": "marley" } }
```

- **Which agents.** Claude Code in a Marley terminal, through the events of Marley's plugin or
  the shared plugin; Codex on its own App Server (#650); and a thread in the Agent Panel or a tab
  when Zed itself would notify you: a tool's permission, a question, a finished turn, an error
  (#752). A subagent's finish or error pushes nothing, and neither does a Codex without the App
  Server, whose own notifications stay (#552).
- **The line** names the project and the agent and nothing the agent wrote:
  `marley-tour: Claude needs approval`, `needs input`, `finished` or `failed`; a thread's line
  names its agent as Zed shows it. Events from one project within five seconds make one push. The
  desktop notification for a Claude Code permission says `needs approval` too.
- **Four switches** in the same section, all on by default: Push Approvals, Push Questions, Push
  Finished Turns and Push Failures (`approval`, `input`, `finish` and `failure` in `marley.push`).
- **The server** must be on this machine (`localhost`, `127.0.0.1` or `[::1]`), so the line crosses
  no network before ntfy's. Push Token File (`token_file`) names a file holding an ntfy access
  token, readable by you alone. A server that stops answering shows one toast until it answers
  again.
- A Claude Code with Remote Control pushes nothing here, since the Claude app notifies you for it
  (#757).

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
Tasks, remote terminals and terminals of remote projects get `MARLEY_BIN` empty.

**The shared Claude Code plugin** (#709) is what reports for Claude Code. It's the plugin Marley
shares with rustal-harness, off until you turn on **Shared Claude Code Plugin** in Settings →
Marley → Agents (`marley.claude_code_shared_plugin`).
- On, each new local terminal loads it through `CLAUDE_CODE_PLUGIN_DIRS`, while your Claude Code
  is 2.1.287 or later (an earlier one needs Shared Plugin on Untested Claude Code).
- A `claude` started there reports working, waiting on you (with the tool and what it acts on) and
  idle, so the rail's row shows the state Claude Code itself gives rather than a reading of its
  hooks.
- Marley writes the plugin's files, read-only, to `claude-code/shared/<digest>/` under its data
  folder, and won't load a copy that was changed.
- A Claude Code under a managed `disableSideloadFlags` refuses to start with the plugin loaded;
  leave the setting off there.
- Since #724 the plugin carries rustal-harness's `rustal-ste` skill. While it is on, a terminal
  gets the skill from it, and Marley's own skill folder (#725) steps aside, so Claude Code lists the
  skill once.
- Since #776 the plugin is rustal-harness's 0.4.0 (its TICKET-119). It hands Claude Code's
  `ExitPlanMode` to Marley's Plan Review (below), and carries the `plan-questions` skill, which
  tells Claude Code to write each open question of a plan as a `:::question` block.

### Plan review (#726)

With **Plan Review** on in Settings → Marley → Agents (`marley.plan_review`, off by default), a
Claude Code in a new local terminal that finishes a plan and asks to leave plan mode hands the
plan to Marley instead of its own dialog. It needs the shared plugin (above), 0.4.0 or later
(#776); with the plugin off, Claude Code's dialog decides as before.

- The plan opens as Markdown in a **Plan · project** tab beside the terminal, taking the focus only
  when the terminal had it, and Needs you lists "Plan to review" until you answer; a click on the
  entry brings the tab forward.
- Select a passage and press `c`, or right-click and choose **Comment…**; type your comment and
  press Enter (Escape drops it). Each note shows its passage and your words, with × to remove it,
  and the passage stays highlighted. An **Overall** note sits below.
- **Approve** lets Claude Code leave plan mode and go on, and the tab says **Approved.** with its
  buttons off. **Send Feedback** (once there is a note or an overall note) sends the notes back as
  the reason: each note's passage and comment, then the overall note, asking Claude Code to
  address every note and plan again. Claude Code stays in plan mode, and its revised plan comes to
  the same tab with no notes.
- A revised plan shows **Version N** in the column's header, with the lines added and removed since
  the version before (#727). **Changes** shows that diff as Zed draws one, removed lines and added
  lines in their colors, read-only; **Plan** goes back to the plan, where notes are made. The tab
  keeps the versions while it is open.
- A plan's open questions are cards in their place (#728), each with "Question N of M" and its
  state: Open, Answered, Skipped, or Settled when the plan marked a choice already. Tick one choice
  (or several, for a question that takes several), press **Accept recommended** for the agent's
  pick, type your own answer in **Other…** (or the answer field of a free-text question), or press
  **Skip**. The column counts "K of M answered". While an answer or a skip waits to go, **Send
  Answers** takes Approve's place, since an approval carries no text; it sends the answers first,
  under "Answers to your questions", then any notes, and Claude Code revises the plan with them.
  An answer given by Accept recommended carries "(your recommendation)". A `c` typed in a card's
  field stays in the field.
- A question is a block in the plan, plannotator's syntax, which the shared plugin's
  `plan-questions` skill teaches Claude Code (#776): a line `:::question` (one choice),
  `:::question-multi` (several) or `:::question-text` (free text), then the question, any context,
  the choices as `- [ ] label — why`, an optional `Recommended: label`, and a closing line `:::`.
  `- [x]` marks a choice already settled. A block with no closing line, or one inside a code
  block, stays text.
- A plan left unanswered for nearly ten minutes, or one whose Claude Code stopped waiting, goes
  back to the terminal's own dialog; the tab says so and its buttons go off.
- Marley's terminals carry `MARLEY_APPROVE_TOOLS=ExitPlanMode` while the setting is on; the plugin
  reads it, and `"$MARLEY_BIN" approve` waits for your answer.

### Notes on a Markdown file or a reply (#729)

The plan review tab also opens on any Markdown file and on an agent's last reply, for notes that
go to an agent.

- **A Markdown file:** with its editor or its preview active, run `marley: review markdown`. A
  **Review · file** tab opens in the same pane with the file drawn as Markdown.
- **A reply:** run `marley: review last reply` with an agent thread or Claude Code's terminal
  active, or choose **Review Last Reply** in the rail's menu for the terminal or the thread. A
  thread's reply is its last turn's text, its thoughts left out. Claude Code's is read from the
  transcript its hook events name, which needs Marley's plugin and a local terminal; its thinking
  and tool output are left out too. A tab **Reply · …** opens with the reply drawn as Markdown.
- Select a passage and press `c` (or right-click, **Comment…**), write the note and press Enter;
  an **Overall** note sits below. **Send Notes** sends the notes not sent yet, and the count on the
  button says how many.
- A file's notes open the agent picker of the diff's review notes, listing the agents whose folder
  holds the file and Copy notes. Each note goes as `File:`, `Line:` or `Lines:`, `Quote:` (the
  passage) and `User comment:`, and the overall note as `File:` and `User comment:`, inside the
  review notes message (#732). Copy notes copies it with the file's full path and marks nothing
  sent.
- A reply's notes go back only to the agent that wrote it, into Claude Code's terminal with Enter
  while it is ready, or into the thread as your message. They come under "Notes on your last
  reply:", each as `Quote:` and `User comment:`, inside the same message. An agent at work, one
  waiting on a question, or a thread with a message of yours typed in it gets nothing, and a toast
  says why.
- Sent notes stay in the tab, marked **Sent**; an overall note that went is kept as a note.

### Per-turn diffs

With the plugin connected, Marley keeps each turn of a terminal's Claude Code that changed the
repository's tree (#509). A turn runs from a prompt to its stop: Claude Code's `Stop`, an API
error, a manual `/compact`, an interrupt, the next prompt, a new session, or Claude Code leaving
the terminal. At the prompt and at the turn's end Marley takes a checkpoint of the innermost
repository that holds Claude Code's directory, as Zed's agent does, through a temporary index. A
turn whose tree changed becomes a commit of its end on its start, pinned under
`refs/marley/turns/<session>/<n>` with Marley as its author; your index, your branches and `HEAD`
are never touched, and a turn that changed nothing leaves nothing.

Codex on Marley's App Server keeps its turns the same way (#753): a turn runs from the lead
thread's `turn/started` to that turn's `turn/completed` (completed, interrupted or failed), the
thread's close, its server's stop or Codex leaving the terminal; the checkpoints are taken in the
repository that holds the thread's folder, the title is the prompt Codex shows for the turn, and
the refs are `refs/marley/turns/<thread id>/<n>`, so a resumed thread counts on.

Under the terminal's rail row, **Turns (N)** and its chevron open the turns of the terminal's
agent, newest first:

| A turn row | What it says |
|---|---|
| The title | The prompt on one line; a slash command's name, such as `/review`; what a harness injected, such as `task notification` |
| `· 2 files` | How many files the turn changed |
| `· failed` | The turn ended in an API error, or the Codex turn failed |
| `· injected` | A harness's prompt started it, not yours |
| `· kept` | You chose Keep: the turn is reviewed |
| `· undone` | You undid the turn |

A click opens the turn in Zed's commit view in the terminal's project: the files the turn changed
against its start, shell commands' edits as much as the agent's, and nothing of the turns before
or after it. The turns stay listed after Claude Code exits, until the terminal closes; the refs
keep the commits after that. A session's first turn deletes the repository's turn refs whose
commits are older than 30 days.

Right-click a turn row for **Keep** and **Undo This Turn…** (#714). Keep marks the turn reviewed
and changes nothing on disk. Undo takes that one turn's changes out of your files: each file the
turn changed goes back to how the turn found it and each file it created goes away, while what
later turns and you yourself changed stays, in the same files and in others. Marley asks first,
naming each file it will restore or remove. Your index, `HEAD`, your branches and the turn refs
stay as they were, and the undone turn's row keeps its diff for a click; its menu then reads only
"This turn was undone". The agent is not told: its conversation still has the turn, so tell it
what you undid. Claude Code's and Codex's turns undo alike.

The undo refuses, says why and changes no file:

- while an agent works in the turn's checkout, a folder inside it or a folder around it, the
  turn's own agent included (wait for it, or stop it);
- when a later change, a turn's or yours, touched the same lines (the message names the files);
- when a file it would change is open in Marley with unsaved edits (save or revert it first);
- when a file it would change changes while the question is open (try again);
- when Marley's `git` is older than 2.40, which the merge needs.

When the files already match the turn's start, it says there is nothing to undo.

Worth knowing:

- The checkpoint leaves out what Zed's checkpoints do: ignored files, untracked files of 2 MB or
  more, and untracked binaries, archives and media.
- The turn is the tree's change, whoever made it: your own edits while a turn runs, or another
  agent's in the same checkout, land in it.
- A remote project's terminals get no turns.
- Marley runs the `git` on its PATH for the turn's commit and its refs.
- Undo takes out only what the turn's commit holds, so what the checkpoint leaves out stays.
- Undo writes your files and nothing else: a change you staged after the turn stays staged, and
  `git status` shows it beside the undo.

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

The prompt gives each note as `File:` (relative to the agent's folder; Copy notes gives the full
path), `Line:` or `Lines:`, and `User comment: "…"`. Sent notes stay in the diff with Sent beside
them, and the button counts only the ones not sent; Copy notes marks nothing sent.

The notes come inside a message (#732). Marley's own asks the agent to check each note against the
code before it changes anything, say whether it agrees and why, fix only what it agrees with, and
change nothing the notes do not name. To send your own, set `marley.review_notes_template` in
`settings.json`, with `{notes}` where the notes go and `\n` for a line break:

```json
"marley": { "review_notes_template": "Please look at these notes.\n\n{notes}\n\nReply before you edit." }
```

A template without `{notes}` gets the notes after it, past a blank line. Copy notes copies the same
message, and notes on a Markdown file or a reply (#729) come in it too. Unset or empty, Marley's own
applies. Marley drops a template's control characters other than line breaks and tabs, so it
cannot carry an escape sequence into an agent's terminal.

**Viewed** (#731). Each file's header in the project diff (`git: diff`) ends with a Viewed tick.
Tick it and the file folds; untick it and the file opens again. The tick is kept with a fingerprint
of the file's diff, so it is still there after a restart, and it goes, with the file unfolding, as
soon as the diff changes: your save, an agent's write, or a commit that moves the base. The branch
diff has no ticks yet.

**Guided review** (#733). An agent can split a change into sections and hand them to you with
`review_guide_open`, in 1 to 30 sections. A **Guide · title** tab opens in the project holding the
first file: the guide's title and summary, "K of N reviewed", then each section in order with its
summary, a chip per file (its path in the project and its lines, such as `alpha.txt:3`) and a
**Reviewed** box. A chip shows the file in the project diff when it has
changes, else opens it at its line. Ticking a section folds it to its title. A guide sent again with
the same title replaces the open one, its ticks cleared; ticks are not kept after the tab closes.
It asks first, as the editor tools do.

**Findings from tools** (#730). An agent or a tool connected to Marley's MCP server can put its
findings into the project diff with `review_notes_add`, up to 50 in one call: each note under its
lines on the changed side, with where it came from (`clippy`, `codex review`, or the caller's name) and **Keep** and
**Drop**. Marley asks first, as for the editor tools (Settings → Marley → Agent Control → Editors),
and opens the project's diff. A finding is a proposal until you keep it: it does not count on Send
Review to Agent and is not sent. Keep makes it an ordinary note that still names its source, and it
goes to the agent with a `Source:` line after its comment; Drop removes it. A finding on lines the
diff does not show (an unchanged part of a file, or a file with no changes) is left out, and the
tool's answer says which and why.

## Zed's Agent Panel with Marley's tools

In Zed's layout the Agent Panel works as it does in Zed. In the Marley layout its agents run in
thread tabs instead (#734): the status bar's sparkle, Ctrl+? and `agent: new thread` open the New
Agent picker (#744), every way Marley opens a thread shows it in a tab (#772), and the rail lists
the threads under each project. The panel itself stays in the right dock, out of sight unless you
open that dock. A Zed Agent thread with no model says so above its message box, with Configure
Providers, and Configure under Select a Model opens the Settings window's LLM Providers page,
not the panel (#781). Marley also offers its MCP server to the agents of the panel and of thread tabs
(#501):

- Marley adds a context server named `marley` to Zed's default settings: a stdio server that runs
  the bridge Marley writes to `~/.local/share/marley/mcp/marley-mcp-bridge`, with
  `MARLEY_MCP_ENDPOINT` naming the endpoint file. No password or token goes into your settings.
- The Zed Agent gets Marley's tools in its default Write profile, which turns on every context
  server.
- Each external agent you start in the panel or a tab, Claude Agent, Codex and the rest, is
  handed the `marley` server with its new session.
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

### Rusty on Home

Rusty's screens and pages open as tabs of the group you are looking at (#743), a project or a group
alike; before #743 they went to a Rusty group of their own (#675). Since #782 Rusty has no home page
of its own: while Rusty is on, the Home page holds its cards under Recent Projects and Agents at
Work. The Rusty card has a button for each screen (Brain, Today's Note, Graph, Tasks, Decisions,
Memory, Skills, Secrets) and a row with Open Page… and the three captures (to Today, to the Inbox, a
URL); under it come your recent pages, the follow-ups due and your open tasks. A recent page or a
decision opens in the Brain tab, a task in the Tasks tab on its list. While Rusty is on but not
connected, the Rusty card says why, and the lists come back once it connects. `rusty: open home`
and `marley rusty` (#749) open Home where you are.

### Buttons at the right of the status bar

Home sits at the right of the status bar, and the Rusty and Marley agents at the right of the
title bar, after the `?` and before the user menu (#739, #746, #778), in both
layouts. Each opens its page or conversation in a tab of the group you are looking at, or brings
that tab forward when the group has it open:

| Button | Where | Opens | Shown |
|---|---|---|---|
| Home | Status bar | Home's page (#701) | Always |
| Rusty | Title bar | `marley: talk to rusty`, Rusty's latest conversation or a new one (#778) | While Rusty's agent is there |
| Marley | Title bar | `marley: talk to marley` (#738) | While the Marley agent is there |

The rail's header no longer has a Rusty button, and the status bar's Rusty button went in #782:
Home holds Rusty's cards. Threads left the status bar for the rail's header
and the Home page (#745).

### The Brain tab

Rusty's vault opens in a Brain tab (#678) of the group you are looking at (#743), one per group:
Brain on Home's Rusty card, Ctrl+Alt+V from anywhere in the window (`marley: toggle brain view`),
or `marley rusty brain`. A second ask brings the tab forward. With Rusty off or not connected, the
key says why in a toast. The vault no longer takes over the rail, as the Brain view did from #644
to #678. The tab's left column holds the search field ("Search the brain…") with a `+` after it,
your favourites and the vault's tree; the page you pick shows on its right ("Pick a page on the
left." until then), with the same Back, Forward, Outline button, folder and name, star and Edit as
a Page tab (below).

- **Today's Note** on Home's Rusty card opens the Brain tab on today's daily note, which Rusty
  makes when it is missing, selected in the tree.
- **Search.** Type in the field and press Enter: Rusty's brain search lists its hits, title and
  slug, in the tree's place. Rusty's `tag:`, `path:`, `file:` and `type:` work as typed. The query
  is sent on Enter only, since with an embedding provider set Rusty embeds every query it gets.
  Up and Down reach the hits from the field, and Enter on the same query opens the selected one.
  Escape clears the search; a second Escape goes back to the tree.
- **The tree.** The vault's folders and pages as Rusty lists them, folders first, each folder
  with its page count. All folders start closed. A click on a folder opens or closes it; one
  click on a page shows it on the tab's right, and a second moves the keyboard into it. A page's
  **Open in New Tab** opens it in a Page tab of its own (below), or brings forward the tab already
  showing it. With the focus in the tree, Up, Down, Home and End move, Right opens a folder or
  steps into it, Left closes it or climbs, and Enter does what a click does.
- **Changing the vault.** Right-click a folder for New Page, New Folder, Rename and Delete; a page
  for Open, Open in New Tab, Rename and Delete; the tree's empty space for New Page, New Folder and
  Refresh. A new name is typed in the tree: Enter makes it (an empty page name lets Rusty call it
  Untitled), Escape drops it, and clicking away keeps a typed name. The `+` after the search field
  makes a page in the selected row's folder. Drag a row onto a folder to move it there, onto a
  page to move it into that page's folder, or onto the empty space to move it to the top. Delete
  asks first; Rusty moves the page or folder into `archive/` and leaves the links to it.
  Everything goes through Rusty's tools, which rewrite links on a rename or a move; Marley writes
  no file of the vault. A refusal shows Rusty's message in a toast, such as "Could not rename
  delta: Already exists: notes/alpha".
- **Live.** On the embedded connection, a change Rusty makes or sees in the vault's folder shows
  with no click. On the service connection the tree is read again after Marley's own changes and
  on Refresh.
- **Favourites** (#662). Rusty's bookmarks, listed above the tree in Rusty's order whenever it
  holds one. A page opens as a tree row does; a folder opens in the tree and is selected; a search
  puts its query in the field and lists its hits; a heading opens its page with that heading at
  the top. Right-click one for Rename… (typed in place, Enter keeps it) and Remove. A page's star
  adds it here (below).

### A brain page in a tab

A page opens in a center tab of its own (#645): Open in New Tab in the Brain tab's tree (#678), a
click in the Knowledge panel, Open a page by name (below), or the `rusty::OpenPage` action, which a
key of your own can bind with a slug (`["rusty::OpenPage", {"slug": "projects/marley"}]`;
`"preview": true` opens it in the preview tab). A page shown on the Brain tab's right works as this
tab does.

- **What it shows.** The page's title, its properties (each frontmatter key and its value, a list
  as chips) and its body, drawn by Zed's own Markdown renderer: headings, lists, task boxes as
  written, code highlighted for its language, tables. The header carries Back, Forward, the
  Outline button (#656), the page's folder and name, the star (#662), and Edit.
- **The star** (#662). Filled when the page is one of Rusty's bookmarks. A click, or Ctrl+D with
  the focus in the tab, adds the page to the Brain tab's Favourites or removes it; the change is
  Rusty's, in its vault, so Rusty's own app shows it too. A change made in Rusty's app fills or
  empties the star with no click, on the embedded connection.
- **Links.** A wikilink is a link. One to a page opens it in the same tab, at its heading when it
  names one (`[[page#Heading]]`); one to no page yet is drawn muted, and a click makes the page at
  that path, its folders too, and shows it (#654); Rusty's refusal, for a path under `archive/`
  say, shows in a toast. A web link opens in your browser.
- **Back and Forward.** The tab keeps its own history: the buttons, or Alt+Left and Alt+Right with
  the focus in the tab.
- **Preview tabs.** A one-click open, such as a link in the Knowledge panel, puts the page in the
  pane's preview tab (its title in italics), which the next one-click page replaces; a
  double-click, or an edit, keeps the tab. With `preview_tabs.enabled` off, every page opens kept.
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
  - A rename or a move, from a tab or from the Brain tab, moves every Page tab that shows the page,
    or has it in its history, along with it.

### Open a page by name (#654)

`rusty: open page`, or Ctrl+Alt+U from anywhere in the window (a terminal included, where Marley
takes the key from the program), opens a picker over every page of the brain.

- **The list.** Each row shows the page's title and its slug. With nothing typed, your favourite
  pages come first, each with a star, in the Brain tab's order (#662); then a line, the pages you
  opened most recently, newest first; then a line and the rest in the order Rusty lists them, the
  most recently changed first. When a page's tab is in front, or the Brain tab showing a page, that
  page heads the list and the selection starts on the row after it; with no favourite that row is
  the page you had open before, so Enter goes back to it. Marley keeps the last 20 pages opened,
  from the picker, the Brain tab, links, Back and Forward, in its own database, so the order holds
  after a restart.
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
the start of one, the high ones first. Memory on Home's Rusty card, `rusty: open memory` or
`marley rusty memory` opens it in a tab of the group you are looking at (#743).

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
`rusty <name>`. Skills on Home's Rusty card, `rusty: open skills` or `marley rusty skills` opens
the tab.

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

Rusty's vault of keys and tokens for providers and services. Secrets on Home's Rusty card,
`rusty: open secrets` or `marley rusty secrets` opens it. The names show to anyone; a value shows,
changes or goes only behind your PIN. Never type the PIN to an agent.

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
- **`rusty: open today`** opens today's daily note in a Page tab; Today's Note on Home's Rusty card
  opens it in the Brain tab.

Escape closes a capture's form; a capture already sent still lands. While Rusty is off or not
connected, each command says so in a toast.

### The Knowledge panel

The Knowledge panel is there while Rusty is on (#646): the Book button in the status bar,
`rusty: toggle knowledge panel` from the command palette, or `marley panel knowledge`, which opens
it with the search field focused. In the Marley layout it opens as a Knowledge tab in the pane you
are in (#748; see [Side panels as tabs](#side-panels-as-tabs-747)) and keeps the page it follows
while its own tab is in front; in Zed's layout it sits in the right dock. With Rusty off there is
no button, the command says where to turn Rusty on, and a panel or tab that was showing closes.

- **The page view.** While a Page tab is the active item, or the Brain tab showing a page, the
  panel shows that page: its title and
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
command palette, Graph on Home's Rusty card, or `marley rusty graph`. With a brain page in front
it opens on that page's neighbourhood; with none, on the neighbourhood of the window's project page
(#655), its header marking it "(project)". `rusty: open local graph` turns it to the page in
front, else the project's page, or says to open a page first. It opens in the group you are
looking at (#743), one Graph tab per group.

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

The Tasks tab shows Rusty's to-do lists in a center tab (#658): Tasks on Home's Rusty card,
`rusty: open tasks` from the command palette, `marley rusty tasks`, or the list icon on the Tasks
heading of the Knowledge panel's project view, which opens the project's task group. It opens in
the group you are looking at (#743), one Tasks tab per group.

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

The Decisions tab lists the decisions recorded in Rusty in a center tab (#659): Decisions on
Home's Rusty card, `rusty: open decisions` from the command palette, or `marley rusty decisions`.
It opens in the group you are looking at (#743), one Decisions tab per group. System One's log is
another tab, System One calls.

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

## MCP Servers

`marley: open mcp servers`, or MCP Servers on Home's Configure card, opens a tab listing every
MCP server your agents see, and what Marley needs of each agent CLI (#720). A second open brings
the same tab forward and reads the files again; so does Refresh.

- **Agent CLIs.** For Claude Code and Codex: the program Marley runs and its version, then each
  part of Marley that rests on it, with the versions it was tested on and whether it is on, not
  turned on, or off on this version. A part that needs a newer version than the one found shows
  in the warning colour, with the update command. A CLI Marley cannot find reads "not on Marley's
  PATH".
- **Too old.** When Marley finds a CLI older than a part you turned on needs, at start or when
  the program changes, a notification says so ("Claude Code 2.1.280 is too old for parts of Marley
  you use"), with each such part and the version it needs, the update command, and Open MCP
  Servers. It shows once per CLI and version. A CLI newer than Marley has tested gets no
  notification: the agent bar's chip says why a part is off.
- **Claude Code.** The user scope (`mcpServers` in `.claude.json`, which sits in
  `$CLAUDE_CONFIG_DIR` when that is set, else in your home folder), the local scope of each open
  project's folder (that folder's entry in the same file), and each open project's `.mcp.json`,
  marked approved, denied or not yet asked in that folder. Servers that Claude Code's plugins add
  are not listed; `claude mcp list` shows them.
- **Codex.** Each `[mcp_servers.<name>]` in `$CODEX_HOME/config.toml` (else
  `~/.codex/config.toml`), off where it says `enabled = false`.
- **Zed.** The context servers of the open projects, with each project's status (running,
  starting, stopped, off, or the error), Marley's own `marley` and `rusty` marked Marley's.
  Configure in Settings opens Zed's MCP Servers page, where Zed's servers are added, changed and
  turned off.

Each row shows the server's name, its scope, its state, and its command line or URL, then the
names of its env variables and headers. No env or header value is read into the tab. Keys and
tokens in a command line are masked, and secret-looking values in a URL's query are hidden, so
the tab can be shown on a shared screen.

**Editing a CLI's servers** (#768). Add Server, beside the Claude Code and Codex headings, opens a
form: a name, for Claude Code where it goes (your user scope, or an open folder's local scope or
`.mcp.json`), then a command with its arguments (quoted as in a shell) or a URL, and the env
variables or headers with their values, typed into fields that show dots. Edit on a row opens the
same form with the command, arguments or URL as stored; each env variable or header is listed by
name, its field reading "set; type to replace", and keeps its value unless you type a new one;
Clear removes it. An edit keeps the server's name and place, so to rename or move one, remove it
and add it again. Save writes and closes the form, and a toast says the CLI reads its servers at
its next start. Remove asks first, naming the server and its scope.

- A save changes that one entry and keeps the rest of the file: Codex's comments and other
  tables, `.claude.json`'s other keys and its layout. The file as it was stays beside it, with
  `.marley-backup` added to its name.
- If the entry changed since the tab read it, nothing is written and the form says so; Refresh,
  then try again. Claude Code rewrites `.claude.json` often while it runs, and that alone does not
  stop a save.
- A local scope exists only for a folder Claude Code has run in.
- Each CLI reads its servers when it starts, so restart a running one to pick up a change.
- Zed's servers are changed on Zed's own page: Configure in Settings.

## The rustal-ste skill for Marley's agents

Marley gives its agents the `rustal-ste` skill out of the box (#725): Simplified Technical English
(ASD-STE100) with Rustal's glossary, verbs, identifiers and message shapes, for what one agent
writes to another (a message, a report, a handoff, a tool description).
- **Zed's agent** has it as a built-in skill, which asks no permission.
- **Claude Code** in Marley's local terminals, and the Marley entry on Claude Code, load it from a
  plugin folder under Marley's data directory, named by its files' digest and written read-only,
  through `CLAUDE_CODE_PLUGIN_DIRS`. New terminals load it. While **Shared Claude Code Plugin**
  is on, a terminal loads that plugin alone, which carries its own `rustal-ste` (#724), so Claude
  Code lists the skill once; the Marley entry keeps Marley's folder.
- **Codex** finds a copy in `$CODEX_HOME/skills/rustal-ste` (`~/.codex/skills` by default), which
  Marley marks as its own with a `.marley-owned` file. A `rustal-ste` folder Marley didn't write is
  left alone.

**Rustal STE Skill** in the Agents section (`marley.rustal_ste_skill`, on by default) turns it all
off: no built-in, no plugin folder in new terminals, and Marley removes its own Codex copy. The
shared plugin's copy stays while that plugin is on. The skill's files are a copy of Rusty's skill
store; its `SOURCE.md` says where it came from.

## The Marley agent

An agent that explains Marley and sets it up with you (#683). It reaches Marley's docs and
settings through Marley's MCP server: `docs_search`, `docs_read`, `settings_schema`,
`settings_read`, `actions_list`, `settings_change`, `keymap_change`, `seat_add` (#692), and
`seat_stop` and `seat_remove` (#710). Its docs are Zed's and Marley's guide, walkthrough and
changelog, and it is told to look in the changelog for what changed and when (#723). Every change
it proposes waits for your Apply. It is on out of the box (#696). **Agent**
(`marley.assistant.agent`, #687) picks what it runs on:

- **Auto** (`auto`, the default, #696): the first found a second after start. That is Claude Code
  signed in (`claude auth status`), then Codex signed in (`codex login status`), then Zed's agent
  with a default model whose provider is signed in. Marley's log says which it chose. With none
  found, the Agent Panel has no Marley entry until you sign in to one and start Marley again, or
  pick an agent here.
- **Claude Code** (`claude_code`): Zed's `claude-acp` adapter on your own login,
  as a **Marley** entry in the Agent Panel. It is told what it is for and kept from editing files
  or running commands: Bash, Edit, Write, NotebookEdit and MultiEdit are off, and so is bypass
  mode.
- **Codex** (`codex`): the registry's `codex-acp` adapter on your own login, as the same
  **Marley** entry. It starts in Codex's read-only mode, which asks before any edit or network use.
  The instructions go in as Codex's `developer_instructions`.
- **Zed's agent** (`zed`): a **Marley** profile for Zed's own agent, with no built-in tools and,
  of the context servers' tools, only the ten above. Zed's agent has no prompt per profile, so
  the tools' own descriptions guide it.

- **Turning it off.** Settings → Marley → **Marley Agent** (`marley.assistant.enabled`), on by
  default, and **Agent** below it. Off, nothing of it is left.
- **Using it.** On Claude Code or Codex: New Agent… → **Marley**, from a project's + in the rail,
  a pane's +, the status bar's sparkle or Ctrl+Alt+N. On Zed's agent: a Zed Agent thread, then the profile picker (Ctrl+I) →
  **Marley**. Ask how a feature works, what a setting does or is set to, or which key runs a
  command. You can also ask it to change a setting or bind a key. Picking the profile makes it
  your default profile, as any pick does. When the profile goes away, a default of Marley goes
  back to Write.
- **In a tab, anywhere** (#738). `marley: talk to marley` brings its latest conversation forward
  where it is open, or opens it in a tab of the group you are in, or starts one there;
  `marley: new marley conversation` always starts a new one. It works in Marley's settings folder
  (`~/.config/marley`), where it can read the settings it changes, so it needs no project; Marley's
  Folder on the Marley page (`marley.assistant.folder`, `~/` for your home folder, made when
  missing) moves it (#780). New Agent… starts it there too, without asking where. On Zed's agent
  each talk starts a new conversation with the Marley profile, since Zed's threads don't say which
  profile they ran.
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

### Rusty in the Agent Panel

While Rusty is on (`marley.rusty.enabled`) and Marley has reached it, the Agent Panel also lists
**Rusty** (#696). It runs on the same agent as the Marley agent (**Agent** above), whether or not
the Marley agent is on. It is told it is your personal assistant, and it reaches your lists, notes,
memories, brain and skills through Rusty's own tools. It changes no file and runs no command.

- **Claude Code:** a `Rusty` agent server like the Marley entry. Its sessions' `_meta` names
  Rusty's server in `mcpServers.rusty`, which the adapter adds to the servers Zed passes. That is
  the `rusty-mcp` Marley found, or Rusty's service URL, so Rusty's tools reach this entry alone. It
  may not use Bash, Edit, Write, NotebookEdit, MultiEdit or any of Marley's tools (`mcp__marley`).
- **Codex:** the same entry, read-only, with Rusty's server in `CODEX_CONFIG`'s `mcp_servers`.
- **In a tab, anywhere** (#738): `marley: talk to rusty` and `marley: new rusty conversation`, as the
  Marley agent's, and the Rusty button in the title bar (#778). Rusty works in the vault folder
  Rusty itself stores (`brain_vault_path`, else `~/.rusty/brain`); Rusty Agent's Folder on the
  Marley page (`marley.rusty.agent_folder`) moves it, and New Agent… starts it there without
  asking where (#780).
- **Zed's agent:** a **Rusty** profile with no built-in tools and the context servers' tools. While
  that profile is there, Marley offers Rusty's server to Zed's agents as the context server `rusty`,
  as **Rusty's Tools for Agents** does, and Zed asks before each call.

### An agent thread in a center tab

An agent thread can sit in the main content, as a tab beside your terminals and files.

- **Start one in a tab** (#734, #744). A project's or a group's `+` → **New Agent…** → an agent
  marked Thread starts a thread in a tab of that project or group. Groups with no folder have it too: Default
  and the groups you named. The thread works in the project's root, or in your home folder
  in a group with no folder. Its row in the rail sits under the group that holds the tab, with its
  status, and its waits reach **Needs you**.
- **Any folder.** `marley::NewAgentThread` takes an agent and a folder, so a key of your own can
  start one anywhere: `["marley::NewAgentThread", {"agent": "claude-acp", "folder":
  "/srv/stacks/rusty-v3"}]`. The agent is its id as the Agent Panel runs it (`Zed Agent`,
  `claude-acp`, `codex-acp`, `Marley`); without one it is the panel's selected agent. Without a
  folder it is the project's root, else your home folder.
- **How the folder is reached.** Zed answers an agent's file reads and writes only inside the
  project's folders, so a folder the project does not hold joins it hidden: the project panel and
  the rail show nothing new, and the agent can read and write there. Your home folder never joins,
  nor a folder that holds it, since Zed would scan and watch all of it; a thread there reaches files
  only inside open projects, as Zed's own threads do, and Claude Code falls back on its shell.
  Zed's own agent works only in the project's visible folders, so in a group with no folder it
  talks, with Marley's and Rusty's tools, but reads no file.
- **Notifications.** A thread whose tab is in front does not notify when it finishes, as one in a
  visible panel does not, and clicking a notification brings its tab forward.

In the Marley layout every thread opens in a tab, never in Zed's Agent Panel (#772): a rail thread
row, Archived Threads, the Threads page, Reopen Thread…, the Needs you inbox's Manager entry and a
notification's click all bring the thread's tab forward, or move a thread the panel still holds
into a tab (it stays the same thread: it keeps running and its history), or open it from Zed's
store in a tab.

- **Reopen Thread…** (#772), in a project's `+`, every pane's `+` and the palette
  (`marley: reopen thread`), lists the saved conversations, and your Claude Code and Codex CLI
  sessions (#754), newest first, with their agent, folder and time; typing matches the title and
  the folder. Enter opens one in a tab of the group
  you are in, with its earlier turns, or brings its tab forward.
- **Zed's layout** keeps the panel: `marley: open thread in center` moves the panel's active
  thread into a tab (#697), and `marley: move thread to panel`, or **Move Thread to Panel** on the
  tab's right-click menu, moves it back. The Marley layout's menus and palette leave both out.
- **One place at a time.** A thread is never shown twice: its tab comes forward wherever it is.
- **After a restart** the tab comes back in the same group and pane, with the thread's earlier
  turns, as the panel loads a thread from its history (#736). A thread you deleted in the meantime
  leaves no tab, and a tab on a folder its project did not hold reaches that folder again.

### New Thread with Context (#717)

From an open thread, **New Thread with Context…** (the thread tab's right-click menu, or
`marley: new thread with context`) starts a new thread on an agent you pick, the old thread's own
agent first, in a split to the right of it. Its message box holds the old thread's history,
unsent: add your ask and send.

- **What it carries.** Your requests, the agent's answers without its thinking, and each tool call
  as its title, status and the paths it changed; tool output stays out. Within 16,000 bytes it
  keeps the latest request, the latest answer and the first request, then earlier items from the
  newest back, each whole.
- **How it reads.** It opens with `Continuing "<title>" from <agent>.` and says the rest is
  history, not a new request; each item has a label such as `[Earlier answer from Claude Agent]`.
  The new thread takes its title from that first line.
- **The whole conversation** goes to `~/.local/share/marley/handoffs/` (mode 0600), and the last
  line names the file and says how many items were left out. An agent that can read files outside
  its project can read it; Zed's own agent may not.
- **Secrets.** With Redact Secrets for Agents on, the history and the file are redacted, as
  everything Marley's tools hand agents is (#516).
- The old thread stays as it was. Harness seats and managers are not offered: each is a window onto
  an agent that already runs.

### The Threads page

`marley: open threads` opens a page of every agent conversation Zed keeps, from every project,
group and folder, in a tab of the group you are in (#737). Zed's own thread history lives in its
sidebar, which the Marley layout replaces with the rail; this page takes its place.

- **What it lists:** each conversation's title, its agent and folder (`~` for your home folder),
  and when it last changed, newest first. Marley's and Rusty's conversations come first, under
  their own heading, then all the others. Archived ones stay folded behind Show archived.
- **Finding one:** the search field matches titles and any part of a folder's path, and a chip per
  agent keeps only that agent's conversations.
- **Opening one:** a conversation already open comes forward in its tab; one an Agent Panel holds
  moves into a tab (#772). Any other opens in a tab of the page's group, with its earlier turns, its folder joining
  the project hidden when it needs to, as a new tab's does (#734).
- **Your CLI sessions** (#754): the Claude Code and Codex sessions you ran in terminals are listed
  among the conversations, from the CLIs' own files (Claude Code's under `~/.claude/projects`, or
  `CLAUDE_CONFIG_DIR`; Codex's under `~/.codex/sessions`, or `CODEX_HOME`), each titled by the
  CLI's own title or its first prompt, with a chip each, Claude Code CLI and Codex CLI. Only the
  sessions you started in the CLI's own terminal show; `claude -p`, the SDKs, subagents, reviews
  and Codex's desktop app stay out. The search also finds a session by the words of your prompts,
  and shows them under the row. Clicking one brings forward the Marley terminal running it, or
  runs `claude --resume <id>` or `codex resume <id>` in a new terminal in the session's folder,
  with your permission settings for the project; a folder that is gone says so instead. The rail's
  Threads view and Reopen Thread… list them too. Marley reads the newest 500 files per CLI, the
  start and end of a long one, and never writes to them; **List CLI Sessions** on the Marley page
  (Agents) turns the listing off.

## Marley's MCP server

### Agent activity and the kill switch

Every call an agent makes to one of Marley's tools that act is listed: typing into or running in
a terminal, the browser's clicks and typing, a settings or keymap change, a harness seat (#703),
and opening, editing or saving an editor, posting into or answering a thread, and running a
palette action (#704 to #707). Some reads are listed too, since they hand an agent a file's or a
conversation's text: `editor_read`, `thread_read` and the language server tools (#704, #706,
and #712).

- **Agent Activity** (`marley: open agent activity`, or Open Agent Activity on Home's AGENT
  ACTIVITY card) lists them newest first: the time, who called (Claude Code, Zed's agent, a
  client's name), the tool, what it acted on (a command, a URL, a key, with keys and tokens
  hidden), and whether it ran or was refused. Home's card shows the five newest.
- **The kill switch.** Stop, on the card or in the tab, or `marley: stop agent control`, makes every
  one of those tools refuse every agent, with `agent_control_stopped`, until you Resume
  (`marley: resume agent control`). The refusals are listed too. The tools that only read keep
  working. It is `marley.agent_control.stopped`, so it holds across a restart, and Settings →
  Marley → Agent Control shows it.
- The log is kept a day per file under Marley's data folder, in `agent_control/`, readable by you
  alone.

### Zed's editors for agents

An agent can see and open Zed's editors through Marley's tools (#704):
- `editor_list` names every open editor: its file, project, unsaved state, language, whether it's
  in front, and its cursor and selections.
- `editor_read` reads one as it is now, unsaved changes included, a page at a time, with keys and
  tokens masked. A file whose name matches a pattern in `marley.agent_control.secret_globs`
  (`.env*`, `*.pem`, `*.key`, `id_rsa*`, `id_ed25519*`, `*credentials*`, `.netrc` and `*.p12` by
  default; set in `settings.json`, with no item on the Marley page) is refused with
  `secret_file`, naming the file, and each read is listed in Agent Activity.
- `editor_open` opens a file inside one of your open projects, at a line, its tab in front.
- `editor_edit` replaces text in an open editor (#705): the text it names has to appear once, or
  every match is replaced when the agent says so; otherwise it is refused with `no_match` or
  `ambiguous` and nothing changes. The change is one undo step and stays unsaved, so Ctrl+Z takes
  it back. A secret file is refused here too.
- `editor_save` saves an open editor. A save asks every time, even in a session you allowed,
  unless Editors is set to Allow.
- `editor_diagnostics`, `editor_definition`, `editor_references` and `editor_hover` (#712) ask
  Zed's language servers: a file's errors and warnings (or every file of the agent's project that
  has some), where a symbol is defined, where it is used, and its hover text. The agent names a
  file inside an open project, a line, and the symbol's name on that line (or a 1-based column).
  The file needn't be open: Marley opens it without a tab for the call and waits up to 15 seconds
  for its language server to start. Answers carry 1-based lines and columns and each place's line,
  keys and tokens masked; a secret file is refused, and a place inside one keeps its path but not
  its text. Lists come a page at a time, and each call is listed in Agent Activity. They are
  reads, so Editors set to Off refuses them and nothing else asks. Their refusals carry
  `outside_projects`, `no_file`, `secret_file`, `no_symbol` (with the line's text),
  `no_language_server` or `lsp_timeout`; the last two say to call again.
- `thread_list` lists the threads in your Agent Panels (#706): each one's title, agent, project,
  whether it is running, and the permission it waits on. `thread_read` reads one, keys masked,
  and shows in Agent Activity.
- `thread_post` sends a message into a thread as you would; Settings → Marley → Agent Control →
  Threads sets whether it asks (Ask First by default). It is refused with `draft_in_progress`
  while you have a message typed there and not sent. A running thread queues the message.
- `thread_answer` allows a thread's waiting tool call once, or rejects it. It asks every time
  unless Threads is Allow, and a sandbox escalation is never answered by an agent
  (`sandbox_escalation`): only you answer that, in the Agent Panel.
- `draft_message` puts a message to another agent in front of you instead of sending it (#769).
  `to` names a terminal whose agent CLI runs (an id from `terminal_list`) or a thread (an id from
  `thread_list`). A Draft tab opens in that window, naming the agent that wrote it and the one it
  is for, with the text in an editor you can change. **Send** delivers what the editor holds, as
  you would: typed at the agent CLI's prompt with Enter, or put in the thread's message box and
  sent. **Drop** closes the tab and sends nothing. Nothing asks first, since the tab is the review.
  An agent CLI that waits on a question, or a thread where you have a message typed, keeps the
  draft open and says why.
- `review_notes_add` puts findings into the project diff as notes you keep or drop (#730), as
  "Findings from tools" under Review notes to the agent describes. It asks as `editor_open` does.
- `review_guide_open` opens a guided review in a tab (#733), as "Guided review" under Review notes
  to the agent describes. It asks as `editor_open` does.
- `action_list` lists the palette actions an agent may run (#707). These are Marley's safe ones
  (docks and panels, splits, pane navigation, search, go-to, Format, folds and the finders) and
  any you name in `marley.agent_control.actions_allowed`. Settings → Marley → Agent Control →
  Allowed Actions edits that list (#711): each name has a remove button, a name Zed has no action
  for is marked "not an action", and the field under the list adds one.
- `action_run` runs one by name, in the window of the project it names or the active one, as
  the command palette would. Settings → Marley → Agent Control → Actions sets whether it asks
  (Ask First by default). An action that could quit, run code or a task, type into a terminal,
  answer a consent, push or change git, delete, install, sign in or share, or change Marley's
  and Rusty's own controls is refused even when you name it (`action_refused`). An action that is
  neither Marley's nor named by you is refused with `action_not_allowed`, which says how to allow
  it, and one the focused element can't take with `not_available`.

**Settings → Marley → Agent Control → Editors** (`marley.agent_control.editors`) sets what agents
may do there:
- **Ask First**, the default: the first time an agent's session uses a tool that acts, such as
  `editor_open`, Marley asks Allow for This Session, Always for This Project, or Deny. A session is
  the agent's terminal (or Zed's agent) in a project, for as long as Marley runs; Always is
  remembered.
- **Ask Every** asks each time, **Allow** never asks, and **Off** refuses the editor tools, reads
  too (`agent_control_off`).

**Threads** and **Actions**, beside it, take the same four values for the thread and action
tools. Deny, or a question left unanswered, refuses the call with `agent_control_declined`. The
question is a notification naming the agent and the tool ("Claude Code wants to use
editor_open"), what it acts on, and the caller's folder.

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
- In a Marley terminal it names that terminal with `MARLEY_TERMINAL_ID` and proves it with
  `MARLEY_TERMINAL_TOKEN`, a secret Marley gives each terminal for the run (#755). A call that
  names a terminal without that terminal's token counts as from no terminal, so an agent in one
  terminal cannot pass for another's, nor use the approvals given there.
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
| `terminal_list` | Every terminal in every window, center and Terminal Panel: its id, its `terminal_id` (the `MARLEY_TERMINAL_ID` its programs see), tab title, project, working directory, running command, how many blocks it holds, and `self` for the terminal the call comes from |
| `terminal_blocks` | A terminal's newest blocks, oldest first (50 unless `last` says otherwise, 500 at most): each command, whether the shell's hook reported it (`verified`), exit code, working directory, start time, duration, whether it runs, and whether its output is still in the scrollback; `redacted` counts the secrets hidden in the commands |
| `terminal_read` | One block's command and output as text, a page at a time (#680): the newest whole lines that fit in 12,000 bytes (2,000 lines at most), numbered from the block's first line (`first_line`, `last_line`, `total_lines`). `previous`, passed back as `before`, reads the page before; a line longer than a page keeps its end (`line_cut`). `redacted` counts the secrets hidden in the whole output |
| `terminal_find` | The line of a block's output a `query` in words names, such as "where the server refused the connection": its number and up to three candidates, by the query's words first and the System One model for the rest (#567); listed only while its use is on |
| `terminal_screen` | What a terminal's screen shows now: its rows (secrets redacted), the cursor, whether a full-screen program has the alternate screen, the program in the foreground, and `generation`, `taken_over` and whether you approved writes to it (#525) |
| `terminal_type` | Types into the program running in a terminal's foreground: `text` as a paste, `keys` by name (`escape`, `ctrl-c`, `up`), and Enter with `submit`, at most 4,096 bytes, given the `generation` `terminal_screen` gave. Never at the shell's prompt or into another agent CLI (#525) |
| `terminal_run` | Runs one command line at a terminal's shell prompt, in your view, as a block with an agent mark before its pill, typed as Rerun types it, then answers the block's index, exit code, duration and output once it ends or `wait_seconds` (20 at most) pass, when it answers `running: true` and `terminal_read` reads the rest. At most 4,096 bytes. Refused while a program or an agent CLI runs there, while you have typed at the prompt, or while you have taken over (#556) |

A command `terminal_run` would type that `marley.agent_command_denylist` matches (Warp's
defaults: shells, `curl`, `wget`, `ssh`, `scp`, `rsync`, `rm` and more) waits on a card under the
terminal with Run and Refuse, which Enter and Escape answer. One `marley.agent_command_allowlist`
matches (`cat`, `echo`, `find`, `grep`, `ls`, `which`) runs at once; each list is regular
expressions matched against the whole command and every part of a pipeline or a list.
`marley.agent_commands_outside_lists` decides the rest: `run` (the default, since the agent's own
permission prompt already asked) or `ask`. **Agent Commands in History**
(`marley.agent_commands_in_history`, on by default) decides whether those commands enter your
shell's history and Marley's suggestions (#553).

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
| `docs_search` | Zed's docs and Marley's guide, walkthrough and changelog (#723), as this Marley ships them, searched for a query in words: up to 10 sections, best first (those holding the most of its words, then the most hits, a hit in a heading counting more), each with its page, heading and a snippet; a changelog entry is a section of its own, and a word not found also tries its form without -s, -es, -ies, -ing or -ed (#723) |
| `docs_read` | A page (`zed/<path>`, `marley/guide.md`, `marley/walkthrough.md` or `marley/CHANGELOG.md`), whole or one section by `heading`, from the top in pages of up to 12,000 bytes; `next`, passed back as `after`, reads on. Zed's key placeholders show the keys bound in this Marley, and its action placeholders their palette names |
| `settings_schema` | What a setting is, by its key path (`terminal.font_size`): its type, description, allowed values and default, and for an object its keys; an empty key lists the top-level keys |
| `settings_read` | What a setting is set to: its value in each settings file that sets it, highest precedence first (each open project's `.zed/settings.json`, your settings, the defaults), which file wins, the value in effect, and `global`, the value where no project file sets it. A value under a name such as `token`, `secret`, `password`, `api_key` or `auth` comes back as `[redacted: setting]` |
| `actions_list` | The actions whose name or documentation holds every word of a query, up to 50, shortest name first: each one's name, palette name, first line of documentation, and the keys bound to it with the context each applies in |

Their refusals carry `no_doc`, `no_section`, `no_setting`, `too_large` or `bad_argument`.

The tools that propose a settings or keymap change, set up the harness's seats, or start another
agent sit with them (#682, #692, #710, #716):

| Tool | What it does |
|---|---|
| `keymap_change` | Proposes a key binding: `keystrokes` as Zed's keymap writes them (`ctrl-alt-m`), an `action` (actions_list finds its name), an optional `context` (`Workspace`) and optional `arguments`. The same notification names the keys, the action's palette name and the context; Apply adds the binding to your `keymap.json` through Zed's keymap updater, comments and other bindings kept, and it works at once. An action Marley lacks is refused with `no_action` and close names, keystrokes or a context that do not parse with `bad_argument` (#686) |
| `seat_add` | Proposes a seat on the harness Marley follows: `name`, `agent` (`claude` or `codex`), `cwd` and an optional `role`. The same notification as a settings change shows the seat and the command it runs through. Apply runs the harness's `seat add` and answers `starting`, then starts the seat, which then shows in the rail. A start that fails comes up as a notification. Refused with the harness's own code (`seat_exists`, `seat_role_reserved`, `claude_signin_undeclared` …), or `tool_off` while `marley.harness_writes` is off (#692) |
| `seat_stop` | Proposes to stop a seat on the harness by its `name` (#710): the harness ends its sessions and their supervision, and the profile stays, so the seat can start again. Marley asks you, and runs the harness's `seat stop` only on Apply; the answer lists the stopped sessions. A refusal carries the harness's code (`seat_unknown`, `seat_runtime`, `seat_stop_failed`) |
| `seat_remove` | The same, then the harness deletes the seat's profile, so the name is free again (#710): `seat remove` on Apply, and the answer adds the removed profile's path; `seat_profile` when the file can't be deleted |
| `agent_start` | Starts another agent from the caller's Marley terminal as a seat on the first harness (#716): `agent` (`claude` or `codex`), a first `prompt`, `place` (`here` or `worktree`), `view` (`terminal`, or `thread` for Claude Code) and a `request_id` UUID the caller keeps. Marley asks every time unless `marley.agent_control.agents` is `allow`, answers `starting`, then adds the seat labelled with its caller and the delegation, opens the view beside the caller (or in the worktree's window) and sends the prompt once with the request id as its delivery, again while the seat is still starting; a start that fails comes up as a notification. Refused `caller_unknown` from outside a Marley terminal, `tool_off` while `marley.harness_writes` is off, `no_harness` when Marley follows no harness it can reach, `agent_control_off` while `marley.agent_control.agents` is `off`, `agent_control_declined`, or with the harness's own code |
| `agent_status` | A delegation's seat, its state, its delivery's state and outcome, its last 40 lines with secrets redacted, and whether its caller was told; reading an ended one marks it told (#716). `no_delegation` for an id no seat carries |
| `settings_change` | Proposes a value for one key of your settings. A notification in every window names the agent, the key, the value now and the value proposed, and the file, with Apply and Decline; Apply writes it, keeping the file's comments and other keys, and it takes effect at once. A key the settings schema lacks (`no_setting`) or a value Zed would not parse (`invalid_value`, with Zed's error) is refused before you are asked; Decline answers `declined`, no answer within 25 seconds `no_answer`, and a file that changed while you were asked `changed` |

The browser family, reading:

| Tool | What it gives |
|---|---|
| `browser_tabs` | Each page of every project's browser: its id (the `tab` the other tools take), title, URL, whether it loads, its project, and `default`, the tab a call without `tab` acts on (the one you focused last in the caller's project); with an open dialog, a file chooser waiting (#767), its downloads this session (#767), and the device size and scheme an agent set (#756) |
| `browser_look` | The page as you see it: URL, title, viewport, scroll, loading, the focused element and the selection, and the frame on your screen as an image; with an open dialog, a file chooser waiting, and the device size and scheme an agent set |
| `browser_snapshot` | The accessibility tree as text: the interactive elements with refs such as `e3` (every node with `full`), cross-site iframes included, no field values |
| `browser_find` | The element a `query` in words names, such as "the sign in button": its ref, which `browser_click` takes, and up to three candidates, by the query's words first and the System One model for the rest (#567); listed only while its use is on |
| `browser_console` | The latest console messages and uncaught errors, oldest first, 200 at most, with secrets hidden |
| `browser_network` | The latest requests, 200 at most: method, URL with secret-looking values hidden, type, status, duration, failure; no headers or bodies |
| `browser_picks` | The elements you picked this session: each pick's id, tab, URL, title, what it is, your caption, and whether you sent it |
| `browser_pick` | One pick: its locators (test id, id, text, CSS path, each marked when it finds the element alone), role and name, the listeners with their script, line and column and, through source maps, the project file and line, what would block a click, its box, and a crop of the page as an image |
| `browser_annotations` | The boxes over a page: each one's id, box in page coordinates, note, who drew it and when |
| `browser_recordings` | The recordings you saved: id, tab, URL, title, when, how many seconds, frames and entries |
| `browser_recording` | One recording's timeline; `frame` gives one frame, counted from 1, as an image |
| `browser_draft_test` | A Playwright test drafted from a recording, by its `id`, and where in the project it would go (#506) |
| `browser_wait` | Waits until the page shows `text`, stops showing `text_gone`, has an element of `role` (and `name`), or is at a URL holding `url`: every condition given, looked for every 250 ms for up to `timeout_ms` (10 s by default, 20 s at most). Refused `not_met`, with what did not hold, when time runs out (#756) |

The browser family, acting. These need the `browser.write` grant. Each but `browser_open_url`
brings its page's tab forward and shows on the Agent chip, and is refused in a tab you have taken
over (#767):

| Tool | What it does |
|---|---|
| `browser_navigate` | Loads an `http` or `https` URL, in a new tab with `new_tab`, and answers once the page has loaded |
| `browser_back` | Goes back in the tab's history and answers once the page has loaded |
| `browser_click` | Clicks an element by its ref, or a point of the viewport, as your mouse does (`button`, `count`) |
| `browser_type` | Types text as key presses into an element by its ref, or where the focus is; `submit` presses Enter after |
| `browser_press` | Presses a key or a chord: Enter, Tab, Escape, ArrowDown, Ctrl+A, Shift+Tab |
| `browser_scroll` | Scrolls by pixels (`dy` down, `dx` right), or an element into view |
| `browser_annotate` | Draws a box with a note around an element or over an area, marked as the agent's; `clear` removes the agent's own boxes |
| `browser_check_pick` | Finds a pick's element again after a change, by its most durable locator, and says what changed: its box, styles, text, role and name, or HTML, with a new crop (#505) |
| `browser_open_url` | Opens a URL a program in `directory` asked to open, in a Browser tab of that folder's project, as Marley's `BROWSER` opener does (#561) |
| `browser_hover` | Moves your mouse onto an element by its ref, or a point, and leaves it there (#756) |
| `browser_select` | Chooses options of a select list by `values` or `labels`; the page gets its input and change events, and Marley's list stays shut. An option not there is refused `no_option`, naming the options there |
| `browser_drag` | Drags from one element or point to another; a page's HTML drag and drop is finished with its drop events |
| `browser_upload` | Sets a file input's files by its `ref`, or with no `ref` answers the file chooser the page waits on: `paths` inside an open project, never a file a secret glob matches (`outside_projects`, `no_file`, `secret_file`, `no_chooser`) |
| `browser_dialog` | Answers the tab's open alert, confirm, prompt or leave-page dialog: `accept`, and `text` for a prompt (`no_dialog` when none is open) |
| `browser_resize` | Lays the page out as a device (`preset`: iPhone SE, iPhone 15 Pro, iPhone 15 Pro Max, Pixel 8, Galaxy S24, iPad Mini, iPad Air, iPad Pro 13, Surface Pro 7, Laptop, Desktop), at `width` and `height` (320 to 3840 by 240 to 2160), or back at the tab's size with `fill` |
| `browser_color_scheme` | Makes the page's `prefers-color-scheme` `dark` or `light`, or the system's again with `system` |

The tools that read or act in one page take `tab`, an id from `browser_tabs`, and without it act
on the tab you focused last. Every answer from a page names its tab.

Every tool Marley lists carries MCP's annotations (#766), so a client can tell a read from an
action: a title ("Browser: click"), `readOnlyHint` for the reads, `destructiveHint` for the
actions you cannot take back from Marley (typing into a terminal, saving a file, a click,
navigating, an upload), `idempotentHint` for the reads and for actions a second call leaves as
the first did (opening an editor, a device size), and `openWorldHint` for the browser tools that
reach a page. The tools of Claude Code's IDE link are marked read-only.

`browser_find` and `terminal_find` are listed and answered only while their use is on in the
System One settings ([The find tools](#the-find-tools)). A client lists its tools when it
connects, so a Claude Code already running sees a tool you turned on after you reconnect
Marley's server (`/mcp`), or in its next session.

The server also holds `fleet_snapshot` and `session_surface_to_human`, which it does not list. The
fleet stays empty until prong 2 feeds it, and a session write is refused, since Marley grants no
`session.write`. Nor does it list `prompt_open` and `prompt_wait`, which Marley's own
`marley-edit` calls to open an agent's prompt in a tab and wait for it to close (#649; named
`editor_open` and `editor_wait` until #704 gave those names to the agents' editor tools).

### Grants and what keeps an agent in check

- Read tools need no grant. A write tool needs its class granted. Marley grants ten when it
  starts the server: `browser.write`, `terminal.write`, `editor.write`, `settings.write`,
  `harness.write`, `thread.write`, `action.write`, `draft.write`, `review.write` and
  `agents.write`. The editor, thread and action tools also pass their area's mode under Agent
  Control, and every write is listed in Agent Activity (#703 to #707); the review tools pass the
  Editors area's (#730, #733).
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
  field has the focus, and a pick never records what was typed into a field. The flight recorder
  keeps the text of each fill of an ordinary field, up to 4,096 characters, for a drafted test
  (#506); a password, hidden, one-time-code or card field keeps only the fact that it was filled.
  `browser_recording` passes a recording through the secret redaction and gives an agent at most
  1,000 characters of a fill.
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

A project's first Browser tab starts a Chromium of its own in the background, as a transient
systemd user unit named `marley-browser-` and twelve hex digits taken from its profile's path
(#488, #507). Each project has its own, so a site you sign in to in one project stays signed out
in another.

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
- It keeps running when a tab, a window or Marley closes, so restored tabs find their pages, and
  ends when you log out or remove the project. Closing the last Browser tab leaves it running with
  no page.
- When the kernel kills one of its pages for memory, the unit keeps running
  (`OOMPolicy=continue`, #755): that page's tab closes, and the project's other tabs carry on.

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
  Navigating, going back or forward, or reloading answers an open dialog with Cancel first. An
  agent's click or key that opens a dialog ends there: the tool's answer names the dialog, and the
  agent answers it with `browser_dialog`, as you would with the card's buttons (#756).

### What agents can do in the tab

Through the browser tools an agent sees what you see (the frame, the accessibility tree, the
console and the network) and acts with the same events your mouse and keys send (#492). A write
call first brings its page's tab to the front of its pane, unless that pane has your focus. The
Agent chip in the toolbar names each action while it runs and for five seconds after; typed text
shows as its length, never itself.

Since #756 an agent can also hover, choose in a select list, drag, upload a file from an open
project, answer a dialog, and wait for the page. It can lay the page out as a phone or tablet, or
at a size, and force the page's dark or light scheme. While it does, a chip in the toolbar names
the device, the size and the scheme ("iPhone 15 Pro · 393×852 · light"), and the tab draws the page
at that size, scaled down to fit, from its top left. Reset, beside the chip, puts the page back at
the tab's size and the system's scheme. A page without `<meta name="viewport">` lays out 980 pixels
wide on a phone, as a real phone does. The user agent stays Chrome's at every size, and a device
size does not survive a restart: the tab comes back at its own size.

An upload takes files from your open projects only, never one a secret glob matches
(`outside_projects`, `no_file`, `secret_file`), and an option a select list lacks is refused with
`no_option`, naming the options there. `browser_tabs` and `browser_look` report a dialog that is
open, the device size and the forced scheme. A wait that times out is refused with `not_met`,
naming what did not hold.

### Files, downloads and taking over

- **File choosers (#767).** A page's Choose File opens Zed's path prompt when you clicked it; the
  files you pick fill the page's input, and Escape picks none. When an agent's click opened it,
  a bar under the toolbar says "The page asks for a file", with Choose… for you and Cancel, and
  the agent answers it with `browser_upload` with no `ref`. A newer chooser replaces one not yet
  answered, `browser_tabs` and `browser_look` report a chooser waiting, and with none waiting the
  upload is refused with `no_chooser`.
- **Downloads (#767).** A page's download is saved in your Downloads folder (`$XDG_DOWNLOAD_DIR`,
  else the desktop's download folder, else `~/Downloads`) under the name the page gives it, with
  " (1)" and on when that name is taken. A notification names it, with Open and Show in Folder,
  and `browser_tabs` lists each tab's downloads.
- **Take Over (#767).** Once an agent has acted in a page, a bar under the toolbar names who, with
  Take Over. Take Over, or Ctrl-I in the Browser tab, gives you the page: the bar reads "You have
  control", and every agent's browser action there is refused (`taken_over`) until you click Hand
  Back. Before
  any agent has acted, Ctrl-I reaches the page as before. It works as the terminal's Take Over
  does.

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
where they landed, keys by name, typing as a count of characters, the agent's actions, console
messages, requests with secret-looking URL values hidden, navigations, the page's accessibility
snapshot after each load, and up to two frames a second. Chromium sends a frame only when the page
changes, so a quiet page leaves few. Nothing is kept for a page behind another tab.

Since #506 the minute also holds each click, fill and key press with the element it landed on,
found by its locators as they were then, which `browser_draft_test` turns into a Playwright test.
A fill of an ordinary field (an e-mail, a search) keeps the field's text, up to 4,096 characters;
a password, hidden, one-time-code or card field keeps only the fact that it was filled.

The red dot, or `marley: record this`, saves that minute into
`~/.local/share/marley/browser/recordings/<id>/` (`timeline.json` and `frames/`), never into a
project, and a toast names it: "Saved this page's last minute as recording `<id>`; agents read it
with browser_recording." An agent you tell "it broke just now" can list the recordings with
`browser_recordings` and read what happened with `browser_recording`.

### Limits of the Browser tab

- Headless Chromium draws no browser UI. Marley draws dialogs and select lists, answers file
  choosers through Zed's path prompt and saves downloads (#767), but draws no context menus or
  the page's cursor shape.
- On a screen scaled above 1, pages look soft: Chromium's frames stay at 1×.
- Japanese and Chinese through a live input method are untested.
- A title a single-page app sets later, with no navigation, does not reach the tab.
- The network log starts when Marley starts watching a page. Console entries from before are
  replayed; requests are not.
- Downloads go to the Downloads folder only, with no list of them beyond the notification and
  `browser_tabs`, and no question before one starts (#767).
- Files dragged from the desktop do not reach a page (#767).

## System One

Marley can ask a System One model typed questions about what it knows (#565): a yes or no, a
choice among fixed options, or a score. Jev, through TypeSafe's API, is the first model. What
comes back is a reading a feature may show, rank or route on; it never approves, sends or stops
anything. Nothing in Marley needs it, and it stays off until you turn it on. Off, Marley makes no
request, reads no key and writes no file.

### Turning it on

1. Give Marley the key: set `MARLEY_SYSTEM_ONE_KEY` in the environment Marley starts from, or open
   System One calls (`marley: open system one calls`), choose Set Key, paste the key and press
   Enter, which writes it to the system keyring at the provider's URL, labelled Marley (#761). The
   variable comes before the keyring. System One calls shows where the key came from and never
   shows the key. A key written before #761 sits under stock Zed's label, `zed-github-account`:
   Marley reads it and copies it under its own label, leaving the old item, and Forget Key removes
   both.
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
inbox, with its options; opening it opens the session's tab. Without `harness_writes`, Marley
only reads (see below). When the harness stops answering, the
section keeps its rows marked stale, its header says why until a try connects again (#740), and
Marley starts the command again after 1 s, then 2, 4 and so on up to a minute. The section stays out of the rail's keys and its filter, as the containers
do. Without the setting Marley starts nothing.

**Writing to a session** (#689). Turn on `"marley": { "harness_writes": true }` and a session's
tab gains controls above its lines:
- **The question.** When the session waits on a question, its prompt shows with a button for each
  option. A click answers it (`session_answer`), naming the question by its full prompt, so an
  answer never lands on a later one.
- **Send.** A line of text, with Send or Enter, goes to the session as a new delivery
  (`session_send`). The line under it gives the delivery's state, or the harness's reason when it
  refuses, for example when the session is not waiting for a message.
- **Views.** Lists the commands that watch the session (`session_surface_to_human`), such as
  `rh view` and `rh attach`, each with Open and Copy. Open (#690) starts the command in a new
  terminal of the same workspace:
  - `rh attach` shows the session's tmux pane, and Ctrl-b d leaves it;
  - `rh view` shows the native view, where Ctrl-b c claims control before you type;
  - a Codex or Claude session offers its history and, while it runs, an observer.

  For a harness `marley.harnesses` names on another host (#740), each command runs there through
  `ssh -t` (#741), and Views shows and copies that line. A `marley.harness` command over SSH still
  runs them on this machine.

`marley: open harness session` in the palette opens a session from a profile declared in the
root's `profiles` folder (`session_open`) and opens its tab. When Marley runs the harness itself,
the picker lists the profiles, and you can always type a name.

**A seat in one step** (#691). `marley: new harness seat` sets up a seat from one form:
- **The form.** A name, the agent (Claude Code or Codex), the folder it works in (the project's
  at first) and a role. `manager` makes it the root's manager; any other role is a label.
- **Create.** Runs the harness's `seat add` and `seat start` with the command Marley follows the
  harness with, up to its `mcp`. A harness reached as `ssh HOST rh --state ROOT mcp …` is set up
  with `ssh HOST rh --state ROOT seat …`.
- **Waiting.** A Claude Code seat takes up to a minute, since the harness waits for its first
  report.
- **The result.** The seat's tab opens under Default. A refusal stays in the form as the harness
  said it, such as `seat_exists` or `claude_signin_undeclared`. For the latter, run
  `rh claude signin` on that root first.
- **From an agent** (#692). Agents, the Marley agent among them, propose the same with
  `seat_add`, and nothing happens until you press Apply.
- **Stop or remove a seat** (#710). Agents propose `seat_stop`, which ends the seat's sessions and
  keeps its profile so it can start again, and `seat_remove`, which also deletes the profile so
  the name is free. The question names the seat and what goes ("builder · stop its sessions and
  delete its profile"); on Apply Marley runs the harness's `seat stop` or `seat remove`, and a
  removed seat leaves the rail. A refusal comes back with the harness's code, such as
  `seat_unknown`.

**The manager in the Agent Panel** (#694). While the switch is on and the harness has a manager (a
session whose `role` label is `manager`, which the harness moves with the designation), the Agent
Panel's agents and New Agent… list **Manager**. Its thread is the manager thread, through the harness's
`rh acp`, run with the command Marley follows the harness with, ending in `acp`:
- **Sending.** What you type goes into the thread as your message, and the harness hands it to
  the manager once the manager is idle. The turn ends after that.
- **Reading.** The manager's messages and reports (`Report: …`) arrive as its replies, also
  between your turns.
- **Confirmations.** A confirmation the manager asks for comes up as a permission request,
  Allow or Reject, for 60 seconds.
- **No manager.** Without a manager the entry is not there.
- **Notices** (#688). Marley follows the thread. A record from the manager while the Manager thread
  is not in front (another tab, another window, or the panel closed) shows a desktop notice,
  "Manager: Report" with its first line. It also adds a "Needs you" entry, "Manager · Report: …",
  until you look. The entry opens the Manager thread in a tab of the group you are in (#758): its
  open tab, else its last thread, else a new tab that loads the manager's whole thread, so the
  record shows. Records from before Marley connected stay quiet.
- **Open Manager Thread** (#758). Right-click the manager's row in the Harness section for the
  same.

When Marley runs the harness itself, it follows it with `--grant write` while the switch is on. A
`harness` command keeps its own arguments: add `"--grant", "write"` there, or the harness refuses
each write and the tab shows why. Off, the tab only reads and the palette does not list the
command.

**Several harnesses** (#740). A harness on another machine gets a name in `marley.harnesses`, and
Marley follows it beside the one above:

```jsonc
"marley": {
  "harnesses": [
    { "name": "box-2", "ssh": "me@box-2", "state": "/srv/harness" },
    { "name": "scratch", "state": "/tmp/harness-scratch", "rh": "/opt/rh/bin/rh" }
  ]
}
```

- **Reaching it.** Marley runs `ssh -T -o BatchMode=yes HOST rh --state ROOT mcp` with its own
  keepalive, so the host must take your key without a password prompt. While `harness_writes` is
  on, Marley adds `--grant write` itself. Without `ssh`, the harness is on this machine. `rh`
  defaults to `rh` on the host's PATH.
- **The rail.** Each harness has its own section after the first, HARNESS · its name, with its
  own connection line, fold and rows. One that stops answering says why under its name, its rows
  stale, while the others go on.
- **Sessions.** A session's tab says its harness after its title, `builder · box-2`, and its
  reads, answers and messages go to that harness. Its question shows in the inbox as
  `Harness · box-2`.
- **Seats.** With `harness_writes` on, the New Harness Seat form gains a Harness row: Default
  (the harness above) and each name. The seat's commands run through the chosen one.
- **A manager on each** (#758). Each harness with a manager has its own entry, **Manager ·
  box-2**, beside the first harness's **Manager**, running `ssh … box-2 rh --state ROOT acp`. Its
  notices read "Manager · box-2: Report", its Needs you entries sit under `Harness · box-2`, and
  each opens that harness's tab, as above. A harness that loses its manager, or is no longer
  followed, takes its entry and its entries in Needs you with it. Restored after a restart, a
  `Manager · box-2` tab makes its entry again while box-2 is followed.
- **What stays with the first.** `marley: open harness session` and agents' seat tools
  (`seat_add`, `seat_stop`, `seat_remove`) work on the harness above only.
- **An agent there.** New Agent's **On box-2…** starts Claude Code or Codex as a seat on it, with a
  terminal attached here (#741), or, for Claude Agent, as a thread tab onto the seat (#742).
- **Open as Thread** (#742). Right-click a seat's row: a seat a profile opened has **Open as
  Thread**, which opens its thread tab in the group you are in, or brings it forward. A seat Marley
  did not start this session is filed under the harness's root on its host. The thread shows the
  seat's text replies only: its tool calls and approvals stay in the seat's terminal and tab.
  Restored after a restart, the tab finds its seat again by its name while the harness is
  followed.
- **From any window** (#758). A seat's or a manager's thread opens in the window you ask from, a
  remote project's included: Marley builds it on a project of its own with no folders, so it
  needs nothing of the window's project and its entry comes from this machine's settings.

**Scheduled runs** (#719). The harness runs a prompt on a schedule: `rh schedule add NAME
--profile PROFILE --cwd DIR --prompt TEXT` with `--every MINUTES`, `--daily HH:MM` or
`--weekdays HH:MM`, and `rh schedule pause|resume|remove NAME`. Each run opens a session from the
profile, sends the prompt, and ends with `finding`, `nothing` or `failed` and a first line; the
harness then stops the session and keeps the run (rustal-harness `docs/SCHEDULE.md`). Marley
builds no timer. It reads each followed harness's tasks every 15 seconds and lists them in the
Harness section after the sessions: the name, the schedule and the next run ("every 60 min · next
14:05", "daily 09:00 · paused"), and the last run's outcome and first line. A run's session is not
listed among the sessions. A click on a task opens its history in a tab: its runs newest first,
each with its time, outcome and first line; a click on a run opens its session. A run that ends
with `finding` raises one Needs you entry, "<task> found something: <first line>", and one
desktop notice; one that ended while Marley was closed is listed without a notice. Opening the
entry opens the run's session and sets the session's label `finding.seen` (harness writes on), so
the entry does not come back after a restart. `nothing` and `failed` raise nothing.

**Agents that start agents** (#716). An agent in a Marley terminal can start another agent with
`agent_start`. The new agent is a seat on the harness Marley follows first, so the harness hosts
it, supervises it and keeps its record: the seat carries the labels `started_by` (`marley:` and the
caller's terminal id) and `delegation` (the caller's request id). Marley asks you each time, with
the agent, the place and the prompt's first line (Agent Control's Agents setting,
`marley.agent_control.agents`: `off` refuses it, `allow` stops the asking). On Allow it starts the
seat, sends the prompt once, and opens the seat beside the caller, or in the new worktree's window.
The seat's row in the Harness section says which terminal started it.

When the seat's turn ends (its delivery reads `ended`), Marley types one line into the caller
once the caller is at its prompt, and presses Enter: `Agent <seat> ended: <outcome>. Read the
result: agent_status {"delegation": "<id>"}.` The line carries ids only, never the child's words.
Marley then sets the seat's `delegation.told` label, so the line comes once, after a restart too:
a caller restored with its terminal gets the line once it is ready. If no open terminal is the
caller, Needs you lists "<seat> ended; its caller is closed"; opening it shows the seat and
counts as told. The new agent runs with its own permission prompts, never a bypass. It needs
harness writes on and a harness that publishes the labels and the `ended` state
(rustal-harness MREQ-010).

**Try it on a real host** (#740 to #742, #758). No run has tried these four against a real second
machine: their scenarios reach the second harness through a stand-in `ssh` on one machine. This
list is that check, done by hand. `HOST`, `ROOT` and the `rh` path are yours:
1. On `HOST`: `rh` installed, `ROOT` owned by the SSH user with mode 0700, `rh --state ROOT serve`
   running, Claude Code's sign-in declared (`rh --state ROOT claude signin`), and a manager:
   `rh --state ROOT seat add boss --agent claude --role manager --cwd DIR`, then
   `rh --state ROOT seat start boss`.
2. In Marley's settings, `marley.harnesses` names `HOST` with `ssh`, `state` and `rh`, and
   `marley.harness_writes` is on. The rail shows `HARNESS · HOST` with `boss`.
3. New Agent lists `Manager · HOST`. Start it and type "hello": `rh --state ROOT thread show` on
   `HOST` holds it, and the manager's answer appears in the tab.
4. Show another tab and have the manager post a report: a notice reads `Manager · HOST: Report`
   and Needs you lists it. The entry brings the tab forward with the report and clears.
5. New Agent → Claude Code → **On HOST…**, then a folder that exists on `HOST`: a terminal opens
   attached to the new seat through `ssh -t`, and the seat's row is under `HARNESS · HOST`. In the
   seat's tab, Views → Open on a view runs it on `HOST` the same way.
6. New Agent → Claude Agent → **On HOST…**, then a folder: a thread tab whose message box names
   the seat and `HOST`. Type a line: the seat's reply comes back as the thread.
7. Quit Marley and start it again: the manager's tab and the seat's thread tab come back and still
   answer.
8. Open a project on `HOST` (Open Remote Folder). In that window, right-click a seat's row → Open as
   Thread, and the manager's row → Open Manager Thread: both tabs open in that window and answer.
9. Cut the network to `HOST` for a minute: the rail marks `HARNESS · HOST` stale, and once it is
   back the tab answers again after you reopen it.

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

Then run `marley: toggle fleet`, click the server icon at the right of the status bar, or run
`marley panel fleet` in a terminal (#749). In the Marley layout the panel opens as a tab in the pane
you are in, and asking again brings that tab forward (#748, see
[Side panels as tabs](#side-panels-as-tabs-747)); in Zed's layout it opens in the right dock,
beside the Agent Panel. Each agent's row shows:

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

The fleet keeps reading while the panel or its tab shows, or an Agent tab is in front. The graphs
start empty after a restart.

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
| Ctrl+Alt+V | Anywhere, while Rusty is connected | The Brain tab, in the group you are looking at (#678, #743) |
| Ctrl+Alt+U | Anywhere, while Rusty is connected | `rusty: open page`, the page picker |
| Alt+Left, Alt+Right | A brain page's tab | Back, Forward |
| Ctrl+? | Anywhere | The New Agent picker in the Marley layout (#744); Zed's Agent Panel in Zed's |
| Ctrl+Shift+E, Ctrl+Shift+G, Ctrl+Shift+B | Marley layout | The Project Panel, Git Panel or Outline Panel tab, brought forward or opened (#747) |
| Ctrl+\` | Marley layout | Switches between the code and the project's terminals |
| Ctrl+J | Marley layout | The same, while the bottom dock is closed |
| Ctrl+~ | Marley layout | A new center terminal |
| Up, Down, Home, End | Rail | Moves the highlight |
| Enter | Rail | Opens the highlighted row |
| Left, Right | Rail | Folds a project or climbs to it; unfolds |
| Ctrl+F | Rail | The filter |
| Escape | Rail filter | Clears the filter; a second press goes back to the rows |
| Ctrl+G | Rail | PROJECTS or THREADS (#745) |
| Up, Down, Home, End, Left, Right, Enter | Brain tab's tree | Move in the tree, open and close folders, show a page |
| Enter, Escape | Brain tab's search field | Searches, or opens the selected hit; clears, then goes back to the tree |
| Ctrl+Tab, Ctrl+Shift+Tab | Rail, Agent Panel | The switcher; hold Ctrl, press Tab or Shift+Tab to move, let go to open |
| Ctrl+Up, Ctrl+Down | Terminal | The previous or next block |
| → | Terminal, at a prompt | Takes the autosuggestion |
| Ctrl+G | Terminal running an agent CLI | Rich input |
| Enter, Shift+Enter, Escape | Rich input | Sends; adds a line; closes and keeps the draft |
| Ctrl+C | The shell's prompt editor | Empties it |
| Up, Down | The shell's prompt editor | The shell's older and newer commands, then your draft |
| Ctrl+W, Ctrl+E, Ctrl+B, Alt+B, Alt+F | Prompt editor, rich input | Delete a word; end of line; back a character; back and forward a word |
| Ctrl+R, Ctrl+O, Ctrl+Q | Prompt editor, rich input | Nothing |
| Ctrl+Shift+V, Ctrl+Shift+C | Prompt editor, rich input | Paste into it; copy its selection, or the terminal's |
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
| c | A Plan or review tab, a passage selected | Comments on the passage (#726, #729) |
| Enter, Escape | A Plan or review tab's comment field | Adds the note; drops it |

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
| `marley: review markdown` | The active Markdown file, or its preview's, in a review tab for notes (#729) |
| `marley: review last reply` | The active agent thread's or terminal Claude Code's last reply in a review tab (#729) |
| `marley: send failed checks to agent` | The project's failed pull request checks to an agent you pick (#718) |
| `marley: open threads` | The Threads page, every saved conversation, in a tab of the group you are looking at (#737) |
| `marley: reopen thread` | Reopen Thread…, a picker over the saved conversations (#772) |
| `marley: talk to marley`, `marley: new marley conversation` | The Marley agent's latest conversation, or a new one, in a tab (#738) |
| `marley: talk to rusty`, `marley: new rusty conversation` | The same for Rusty (#738) |
| `marley: open thread in center`, `marley: move thread to panel` | Zed's layout only: the Agent Panel's thread into a tab, and back (#697, #772) |
| `rusty: open home` | the Home page, with Rusty's cards, in the group you are looking at (#782) |
| `multi workspace: next project`, `previous project`, `next thread`, `previous thread` | Walk the rail |

## Settings

Marley keeps its settings in `~/.config/marley/settings.json`, in Zed's format (Ctrl+Alt+, opens
the file; Ctrl+, opens the Settings window). `marley: open settings` opens the Settings window on its Marley page,
first in the list (#515). The page has a Layout section, with the layout as a dropdown, the
Prompt Editor toggle (#627) and Follow Omarchy's Theme (#759), an Agents
section with Redact Secrets for Agents (#516), Codex App Server (#650), Shared Claude Code Plugin (#709) and Agent Prompts in a Tab (#649), an Agent Versions section with Prompt Tags on
Untested Claude Code (#648), App Server on Untested Codex (#650) and Shared Plugin on Untested
Claude Code (#709), an Agent Control section whose Agents setting says whether an agent may start
another (#716), a Voice section with the Voice toggle (#642), a Rusty
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
    // On Omarchy, draw Marley in the desktop's theme and follow each theme switch (#759). Off by
    // default.
    "omarchy_theme": false,
    // Hide secrets in what Marley's tools give agents (true by default), and more to hide: each
    // entry is a regular expression, and one that does not compile is named in a notification.
    "redact_secrets_for_agents": true,
    "redaction_patterns": ["INTERNAL-[0-9]{6}"],
    // What runs on an agent version Marley has not tested it on (#648): claude_prompt_tags, the
    // tags that tell your prompts from the ones Claude Code adds, codex_app_server (#650) and
    // claude_shared_plugin, the shared Claude Code plugin (#709). Off by default.
    "allow_untested_versions": {
      "claude_prompt_tags": false,
      "codex_app_server": false,
      "claude_shared_plugin": false
    },
    // Agent prompts in a tab (#649): the agent's own editor key opens its prompt in a Marley tab.
    "agent_editor_in_tab": false,
    // Codex's own App Server (#650): the rail reads each launched Codex's thread. Off by default.
    "codex_app_server": false,
    // A Codex stopped at its usage limit continues by itself at the reset (#715). Off by default:
    // Continue at Reset in Needs you does it on a click.
    "continue_at_reset": false,
    // New local terminals load the Claude Code plugin Marley shares with rustal-harness, so a
    // Claude Code there reports its own state to the rail (#709). Off by default.
    "claude_code_shared_plugin": false,
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
    // The rustal-harness whose sessions the rail follows, as a context server's command (#534).
    // Unset by default, which starts nothing.
    "harness": { "command": "/path/to/rh", "args": ["--state", "/path/to/root", "mcp"] },
    // Marley runs `rh serve` on a root of its own and follows it (#632); "harness" wins. Off by
    // default.
    "embedded_harness": false,
    // The harness's write verbs: a session's tab answers and sends, New Harness Seat sets up
    // seats, and agents' seat tools run (#689). Off by default.
    "harness_writes": false,
    // Harnesses on other hosts, each followed over SSH beside "harness" (#740). None by default.
    "harnesses": [{ "name": "box-2", "ssh": "me@box-2", "state": "/srv/harness", "rh": "rh" }],
    // One of Agent Control's settings (#703): whether an agent in a Marley terminal may start
    // another as a harness seat (#716). "off" refuses, "allow" never asks, and "ask_first" (the
    // default) or "ask_every" asks at every start.
    "agent_control": { "agents": "ask_first" },
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

The harness keys (`harness`, `embedded_harness`, `harness_writes` and `harnesses`) have no row on
the Marley page; set them in the file. The section on the harness's sessions, above, says what
each does.

Telemetry is off by default in Marley (#514): `telemetry.metrics` and `telemetry.diagnostics`
default to false, and turning either on works as in Zed.

On Omarchy, **Follow Omarchy's Theme** (`marley.omarchy_theme`, off by default) draws Marley in the
desktop's theme (#759). Turned on, Marley writes a theme template to
`~/.config/omarchy/themed/marley.json.tpl` and a hook to
`~/.config/omarchy/hooks/theme-set.d/marley-theme`. At each theme switch Omarchy fills the template
with the theme's colours and the hook copies the result to `~/.config/marley/themes/omarchy.json`,
which Marley reloads like any theme file there: the editor, the panels and the terminal's 16
colours change with the desktop, light or dark as the theme is, with no restart. When the current
theme has no render yet, turning it on runs `omarchy-theme-refresh` once, which re-applies the
theme, so Hyprland and the terminals reload too. Marley selects the "Omarchy" theme and remembers
the one it replaced. Turning it off puts that one back, unless you picked another theme meanwhile,
and removes the hook and the copy. The template stays Marley's while its first line is Marley's
comment; delete that line to keep your own edits, and Marley neither rewrites nor removes the file.
A render with a colour Omarchy could not fill keeps the last good copy. Without Omarchy the setting
does nothing.

Environment variables and flags:

| Name | Read by | What it does |
|---|---|---|
| `MARLEY_CHROMIUM` | Marley | The Chromium binary to run; when set, nothing else is tried |
| `MARLEY_SYSTEM_ONE_KEY` | Marley | The System One layer's key; it comes before the keyring |
| `MARLEY_CLOUDFLARE_API_TOKEN` | Marley | The `cloudflare` provider's API token (#548); it comes before the keyring |
| `MARLEY_RH` | Marley | The harness's `rh` to run when `embedded_harness` is on; when set, the PATH is not searched |
| `MARLEY_OMARCHY_HOME` | Marley | The home folder Omarchy's `.config/omarchy` and `.local/state/omarchy` are read under, for Follow Omarchy's Theme (default `$HOME`) |
| `MARLEY_OMARCHY_REFRESH` | Marley | The program that re-applies Omarchy's current theme (default `omarchy-theme-refresh` on the PATH) |
| `MARLEY_MCP_ENDPOINT` | The bridge | The endpoint file to read |
| `CLAUDE_CONFIG_DIR` | Marley | Claude Code's configuration directory, for whether the plugin is installed (default `~/.claude`) |
| `ZED_LOG`, `RUST_LOG` | Marley | The log filter. It matches the crate a line is logged from: `ZED_LOG=marley_browser=debug` logs each input-to-frame time, and `marley_workbench::browser` names the Browser tab's own lines |
| `XDG_DATA_HOME` | Marley, the bridge, the launcher | Where `marley/` data lives (default `~/.local/share`) |
| `--user-data-dir DIR` | Marley | A separate profile: config in `DIR/config`, data in `DIR` |
| `ZED_BUILD_REMOTE_SERVER` | A debug Marley | `never` uploads the staged remote server instead of compiling one when it connects to an SSH host (#445) |
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
| `~/.local/share/marley/claude-code/shared/<digest>/` | The plugin Marley shares with rustal-harness, read-only, named by its digest (#709) |
| `~/.local/share/marley/marley-dev.sock` | The socket a second launch hands its paths to, while Marley runs (#760; `zed-dev.sock` before) |
| `~/.local/share/marley/harness/` | The root of the harness Marley runs itself while `embedded_harness` is on (#632) |
| `~/.local/share/marley/remote_servers/dev/linux-x86_64/<commit>.gz` | The remote servers the install staged, the newest three (#445) |
| `~/.local/share/marley/shell_integration/` | `marley.bash` and `zsh/.zshenv` |
| `~/.local/share/marley/browser/projects/<key>/` | A project's browser: `profile/`, `project.json`, and while it runs the relay's `relay.sock` and `relay.json` (mode 0600) |
| `~/.local/share/marley/browser/recordings/<id>/` | A saved recording: `timeline.json` and `frames/` |
| `~/.local/share/marley/system_one/` | The System One layer's calls, a file a day (mode 0600), and `replay.jsonl` |
| `~/.local/share/marley/checks/` | The failed checks' logs sent to an agent, a file per pull request and commit (mode 0600, #718) |
| `~/.cache/marley/` | Zed's cache |
| `~/.local/bin/marley`, `~/.local/lib/marley/marley` | The installed launcher and binary |
| `~/.local/share/applications/marley.desktop` | The menu entry, which claims `marley://` (#762) |
| `~/.config/marley/themes/omarchy.json` | Omarchy's current theme drawn for Marley, while Follow Omarchy's Theme is on (#759) |
| `~/.config/omarchy/themed/marley.json.tpl`, `~/.config/omarchy/hooks/theme-set.d/marley-theme` | The template Omarchy fills and the hook that copies the result, while Follow Omarchy's Theme is on (#759) |
| `~/.zed_server/zed-remote-server-dev-<commit>`, on an SSH host | Marley's remote server, one per Marley commit (#445) |

Every `~/.local/share/marley` path follows `$XDG_DATA_HOME`, and a Marley started with
`--user-data-dir DIR` keeps them all under `DIR`. Marley's items in the system keyring, such as
System One's key, are labelled Marley (#761).

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
- **A `zed://` link opens stock Zed.** Since #762 a `zed://` or `marley://` link clicked inside
  Marley opens in Marley. A `zed://` link clicked anywhere else still goes to stock Zed, which owns
  that scheme on the desktop; write it as `marley://` with the same path to reach Marley.
- **Opening a folder over SSH fails with "no Marley remote server is staged for linux-x86_64 at
  commit …".** Nothing is staged for the commit Marley was built from. Run `script/install-marley
  --remote-server-only` in the checkout at that commit (#445). A debug build started with
  `ZED_BUILD_REMOTE_SERVER=never` needs a server staged for its own commit in the same way.
- **`just install` stops with "the remote server needs musl-gcc (the musl and llvm packages)"**,
  or names `llvm-objcopy`. Install Arch's `musl` and `llvm` packages and run it again; nothing was
  installed (#445).
- **A Hyprland window rule stopped matching Marley.** Since #760 Marley's class is
  `com.ignibyte.Marley`, no longer `dev.zed.Zed-Dev`. Match the new one.
- **After an install, a second Marley started beside the running one.** The running Marley came
  from a build before #760 and listens on `zed-dev.sock`, which the new build does not look for, and
  two Marleys on one data directory hang. Quit both (`pgrep -x marley` finds them) and start one.
- **The Harness section says "not running".** Point at the header for the whole reason. With
  `embedded_harness`, it may be that no `rh` was found: install it or set `MARLEY_RH`. For a
  harness in `marley.harnesses`, Marley runs `ssh` with `BatchMode=yes`, so a host that asks for a
  password or a passphrase never answers; `ssh -o BatchMode=yes HOST true` shows whether your key
  gets in.
- **A harness session's tab refuses every write.** Turn on `marley.harness_writes`. A `harness`
  command of your own also needs `"--grant", "write"` in its arguments; Marley adds it only to the
  harness it runs itself and to those in `marley.harnesses`.
- **New Harness Seat or New Agent's On box-2… refuses with `claude_signin_undeclared`.** The
  harness's root has no Claude Code sign-in declared: run `rh --state ROOT claude signin` there.

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

`docs/planning/tickets/BACKLOG.md` orders the queue of open tickets, each with its file under
`docs/planning/tickets/open/`. The walkthrough's Appendix C lists what shipped tickets left for
later.

Marley's own release identity shipped in five slices: its remote server, app id, keyring label,
`marley://` scheme and updater message (#445, #760 to #763).

Most of `docs/marley/three-prong-plan.md` has shipped: block-scoped path links and a jump to the
first failure (T2), the prompt editor (T3), tasks and runnables as blocks (T4), native block
headers with the shell's prompt hidden (T5), completions in the prompt (T6), and the harness's
sessions and seats in the rail. Two items remain: Rusty's sessions as rows in the rail, which wait
on `rusty-mcp` serving them (C2), and work objects (C5): rustal-brain's tickets, runs and gate
evidence as labels on seats and in a pane of their own, and the workflow's phase in the status bar.
