# The Marley walkthrough

A hands-on tour of all of Marley, one stop at a time, in an order where each stop needs only
what came before it. Every stop says what to do and what you should see, and names the ticket
that shipped it. The tour follows the tree at #597 (2026-09-29) on Linux, as built and tested on
Omarchy with Hyprland.

[guide.md](guide.md) is the reference behind this tour, with the rules and edge cases a stop
leaves out. [tutorial-outline.md](tutorial-outline.md) is the plan for a longer tutorial, and
stops at #516. This file covers everything shipped through #597.

- [How to use this tour](#how-to-use-this-tour)
- [Part 0. Before you start](#part-0-before-you-start)
- [Part 1. Start Marley and learn the window](#part-1-start-marley-and-learn-the-window)
- [Part 2. The rail](#part-2-the-rail)
- [Part 3. The editor](#part-3-the-editor)
- [Part 4. The block terminal](#part-4-the-block-terminal)
- [Part 5. Agent CLIs in terminals](#part-5-agent-clis-in-terminals)
- [Part 6. Zed's Agent Panel](#part-6-zeds-agent-panel)
- [Part 7. The Browser tab](#part-7-the-browser-tab)
- [Part 8. Launch configs](#part-8-launch-configs)
- [Part 9. Worktree agents](#part-9-worktree-agents)
- [Part 10. System One on local rules](#part-10-system-one-on-local-rules)
- [Part 11. Living with Marley](#part-11-living-with-marley)
- [Appendix A. Keys](#appendix-a-keys)
- [Appendix B. Commands](#appendix-b-commands)
- [Appendix C. Not in yet](#appendix-c-not-in-yet)

## How to use this tour

Part 0 runs in any terminal. Once Marley is up (stop 1.2), open this file in it from one of its
terminals with `marley /srv/stacks/marley_ide/docs/marley/walkthrough.md`, then press Ctrl+K V
to put the Markdown preview beside it. In the preview every `sh` block has an **Insert in
Terminal** button (#530), which clears the prompt of the terminal you used last, puts the command
there without running it, and brings that terminal forward. Press Enter to run it. That button is
itself stop 3.5.

Each stop has a number such as `4.7`. Every line under a stop that starts with a box is
something you should see. When one does not match, write down the stop number and what you saw
instead. The ticket number after a stop's title leads to its entry in `CHANGELOG.md` and to the
e2e scenario that proved it, `script/e2e/<ticket>-*.sh`, which `just e2e <scenario>` runs against
the debug build.

A stop marked **optional** needs something the tour does not set up: a second machine, an ntfy
server, or a Vite project. Skip it freely.

Commands with a relative path assume a `marley-tour` terminal at the project's root. When a shell
has wandered, `cd ~/marley-tour` first.

Keys are written for Linux with Zed's default keymap. Your own `~/.config/marley/keymap.json`
wins over Marley's keys, so a key you rebound behaves your way.

## Part 0. Before you start

### 0.1 Install the build you are testing (#502)

Quit any running Marley first (`zed: quit` from the command palette), then:

```sh
cd /srv/stacks/marley_ide && just install
```

- [ ] It waits while any other cargo command runs on the machine, then builds the release
  profile. The first build takes about ten minutes on the dev box; an unchanged tree takes
  seconds.
- [ ] It ends by printing `~/.local/bin/marley at <commit>` and the menu entry's path, and warns
  when a Marley is running (that Marley keeps its old build until you restart it).

### 0.2 Check the programs the tour uses

```sh
for p in claude codex gemini opencode voxtype chromium gh tmux npm python3; do printf '%-9s %s\n' "$p" "$(command -v $p || echo missing)"; done
```

- [ ] `claude`, `chromium`, `npm` and `python3` are found. Parts 5 to 10 need Claude Code signed
  in, and Part 7 needs Chromium.
- [ ] The rest are optional: `codex`, `gemini` and `opencode` for stop 5.17, `voxtype` for 5.8,
  `gh` for 2.9's pull request chip, and `tmux` 3.3 or later on a remote host for 4.16.

### 0.3 Make the practice projects

Run this once, in any terminal. It makes `~/marley-tour`, a git repository with a small web page
in `site/`, an npm `dev` script that serves that page on `$PORT` (8000 when unset), a launch
config, two tasks, a `.env` that git ignores and `.worktreeinclude` names, a runbook and an icon.
It also makes `~/marley-tour-b`, a second repository with one file. It refuses to run when
`~/marley-tour` exists.

```sh
bash -e <<'TOUR'
tour=${TOUR:-$HOME/marley-tour}
if [ -e "$tour" ]; then echo "$tour already exists; move it away or set TOUR"; exit 1; fi
mkdir -p "$tour"/{site,public,notes,.zed} "$tour-b"
cd "$tour"

cat >README.md <<'EOF'
# Marley tour

A practice project for docs/marley/walkthrough.md.
EOF
printf '# Notes\n\n- [ ] try every stop\n' >notes/todo.md
printf '.env\nnode_modules/\n' >.gitignore
printf 'DEMO_TOKEN=not-a-real-one\n' >.env
printf '.env\n' >.worktreeinclude

cat >package.json <<'EOF'
{
  "name": "marley-tour",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "dev": "python3 -m http.server ${PORT:-8000} --directory site"
  }
}
EOF
cat >package-lock.json <<'EOF'
{
  "name": "marley-tour",
  "version": "1.0.0",
  "lockfileVersion": 3,
  "requires": true,
  "packages": {
    "": {
      "name": "marley-tour",
      "version": "1.0.0"
    }
  }
}
EOF

cat >public/favicon.svg <<'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" rx="6" fill="#e0a526"/><text x="16" y="23" font-size="20" text-anchor="middle" font-family="sans-serif" fill="#1d1d1d">M</text></svg>
EOF
cp public/favicon.svg site/favicon.svg

cat >site/index.html <<'EOF'
<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <title>Marley tour</title>
  <link rel="icon" href="/favicon.svg">
  <style>body { font: 16px sans-serif; margin: 2rem; } section { margin: 1rem 0; }</style>
</head>
<body>
  <h1>Marley tour</h1>
  <section>
    <button id="hello" data-testid="say-hello">Say hello</button>
    <input id="name" placeholder="Your name">
  </section>
  <section>
    <select id="food">
      <optgroup label="Fruit"><option>apple</option><option>pear</option></optgroup>
      <optgroup label="Vegetables"><option>carrot</option><option disabled>kale</option></optgroup>
    </select>
    <a href="page2.html" target="_blank">Open page two in a new tab</a>
  </section>
  <section>
    <button id="alert">Show an alert</button>
    <button id="fetch">Fetch a missing file</button>
    <button id="retitle">Change the title in two seconds</button>
  </section>
  <section>
    <button id="order">Place order</button>
    <button id="delete">Delete account</button>
  </section>
  <script src="app.js"></script>
</body>
</html>
EOF

cat >site/app.js <<'EOF'
function greet(name) {
  return `hello, ${name || "stranger"}`;
}

document.getElementById("hello").addEventListener("click", () => {
  console.log(greet(document.getElementById("name").value));
});

document.getElementById("alert").addEventListener("click", () => {
  alert("Hello from the tour page");
});

document.getElementById("fetch").addEventListener("click", async () => {
  const response = await fetch("/missing.json");
  if (!response.ok) console.error(`fetch failed: ${response.status} for /missing.json`);
});

document.getElementById("retitle").addEventListener("click", () => {
  setTimeout(() => { document.title = "Title set later"; }, 2000);
});

document.getElementById("order").addEventListener("click", () => {
  console.log("order placed (not really)");
});

document.getElementById("delete").addEventListener("click", () => {
  console.log("account deleted (not really)");
});
EOF

cat >site/page2.html <<'EOF'
<!doctype html>
<html><head><meta charset="utf-8"><title>Page two</title></head>
<body><h1>Page two</h1><p><a href="index.html">Back to the tour page</a></p></body></html>
EOF

cat >.zed/marley.json <<'EOF'
{
  "launch": {
    "Dev": {
      "items": [
        { "terminal": "npm run dev", "title": "dev server" },
        { "agent": "claude", "split": "right", "focus": true },
        { "browser": "http://localhost:8000", "split": "down" }
      ]
    }
  }
}
EOF

cat >.zed/tasks.json <<'EOF'
[
  {
    "label": "say hello",
    "command": "echo hello from a task"
  },
  {
    "label": "tear down the worktree",
    "command": "echo tearing down $MARLEY_WORKTREE_PATH; sleep 2",
    "hooks": ["remove_worktree"]
  }
]
EOF

fence=$(printf '\140\140\140')
printf '# Runbook\n\nOpen the preview with Ctrl+Shift+V. Each block has Insert in Terminal.\n\n%ssh\ngit log --oneline\n%s\n\n%ssh\necho one\necho two\n%s\n' \
  "$fence" "$fence" "$fence" "$fence" >runbook.md

git init -q -b main
git add -A
git commit -qm "Start the Marley tour"

cd "$tour-b"
printf '# Second project\n' >README.md
git init -q -b main
git add -A
git commit -qm "Start the second project"

echo "Made $tour and $tour-b"
TOUR
```

- [ ] It prints `Made /home/<you>/marley-tour and /home/<you>/marley-tour-b`.

Stop 11.5 removes everything the tour made.

## Part 1. Start Marley and learn the window

### 1.1 First launch (#460, #438)

Start Marley from the app menu (Super+Space, then type `Marley`), or run `marley`.

- [ ] The rail runs down the left: a PROJECTS header, Add Project (a folder with a plus) and a
  filter field under it.
- [ ] Ctrl+? brings up the Agent Panel, docked on the right.
- [ ] The status bar has no Terminal Panel button.
- [ ] The title bar has a `?` before Sign In; a click opens the Marley guide in a Browser tab
  once a project is open, or in your system browser before (#599).

### 1.2 Open the practice project (#455, #476, #564)

In the rail, click Add Project, then Open Local Folders, and choose `~/marley-tour`. If Zed asks
whether to trust the folder ("Unrecognized Project", Zed's Restricted Mode), trust it.

- [ ] The project appears in the rail with a yellow `M` icon before its name, read from
  `public/favicon.svg` (#564).
- [ ] One terminal sits under it, open at the folder's root with the focus (#455).
- [ ] The prompt is on the terminal's bottom row, not the top (#476).

### 1.3 Two layouts, one switch (#438, #451, #456)

1. Open the command palette with Ctrl+Shift+P and run `marley: use zed layout`.
2. Press Ctrl+\`.
3. Run `marley: use marley layout`.
4. From the title bar's Panel Layout menu, choose Classic.

- [ ] In Zed's layout, Zed's Threads Sidebar replaces the rail, and Ctrl+\` opens the bottom
  Terminal Panel.
- [ ] Back in the Marley layout, the rail returns and each dock shows the panel it had.
- [ ] Classic changes nothing and shows a toast saying the presets belong to Zed's layout, with a
  Use Zed's Layout button.

Keep the switch in mind: when panels, docks or terminal placement misbehave, the Zed layout tells
you whether the problem is Marley's or Zed's.

### 1.4 The Marley settings page (#515)

Run `marley: open settings`.

- [ ] The Settings window opens on its Marley page, first in the list.
- [ ] The page has six sections: Layout, Agents, Terminal, Push, System One and Privacy.
- [ ] Privacy shows both telemetry toggles off (#514), and Agents shows Redact Secrets for Agents
  on (#516).
- [ ] The Layout dropdown switches every window, as the two commands do.

### 1.5 Quit and come back (#575, #577, #486)

1. In the terminal, run `cd notes`.
2. Press Ctrl+Q in the terminal.
3. Quit with `zed: quit` from the command palette, then start Marley again.

- [ ] Ctrl+Q went to the shell, not to Zed: a focused terminal passes it on.
- [ ] After the restart the project and its terminal come back, as a shell in `notes`, without
  its scrollback (#577).
- [ ] The terminal opens at the size it had last time, so a wide two-line prompt draws right from
  the first prompt (#486).

### 1.6 One Marley at a time (#513, #545)

With Marley running, run this in one of its terminals:

```sh
marley ~/marley-tour/README.md
```

Then start Marley again from the app menu.

- [ ] The command prints that Marley is already running and was handed one of this launch's
  paths, and `README.md` opens in the running window.
- [ ] The menu launch opens no second window; it brings the running one forward (#545).

## Part 2. The rail

### 2.1 Terminals as rows (#438, #452, #468, #551)

1. Click the project's `+`, then New Terminal. Do it twice.
2. In one new terminal, run `cd site`, then `false`.
3. Double-click the other new terminal's row and rename it `server`.
4. Check the rows against the list below, then right-click the `site` terminal's row and choose
   Close. Point at another row to find its close button, and leave that one open.

- [ ] Each row has a `>_` icon, its title, and a second line with its folder relative to the
  project (empty at the root, `site` after the `cd`).
- [ ] A plain terminal's row also shows its last command and state: `false · exit 1 · 0 s` in
  red, `· done` and the time after a success, `· running` while one runs (#551).
- [ ] The name `server` shows on the row and the tab, and survives a restart.

### 2.2 A second project (#453, #507)

1. Add Project again and open `~/marley-tour-b`.
2. Fold `marley-tour` with its chevron and unfold it.
3. Right-click `marley-tour-b`'s header.

- [ ] A line separates the two projects, and clicking a row in either shows that project.
- [ ] Only one row is highlighted at a time.
- [ ] The header's menu holds Move Project Up, Move Project Down, Clear Browser Data… and Remove
  Project. Leave both projects in place for now.
- [ ] Later, after any restart (#606): the project that was not shown at the quit is still
  listed, dimmed and without rows; pointing at it reads "Not open. Click to open it.", and a click
  opens it with its terminals back in their folders.

### 2.3 Dots that ask for you (#438)

1. In a `marley-tour` terminal, run the line below, then at once click a terminal of
   `marley-tour-b`.
2. Fold `marley-tour` and do it again.

```sh
sleep 3; printf '\a'
```

- [ ] The first time, a dot lights the terminal's row.
- [ ] Folded, the dot sits on the project's header instead.
- [ ] Clicking the row clears it.

### 2.4 A long command's end (#551)

1. Run the line below in a `marley-tour` terminal, then click into another pane and wait.

```sh
sleep 31
```

2. Optional: run `sudo -k; sudo true` and click away before you type the password.

- [ ] About 31 seconds later a desktop notification titled `sleep 31` reads `done in 31 s`, and the
  row gets the unread dot.
- [ ] The `sudo` line posts `waiting for a password` once.

Long Command Seconds on the Marley page (Terminal section) sets the 30-second threshold; 0 turns
it off.

### 2.5 The rail from the keyboard (#453, #442)

1. Press Ctrl+Alt+; to put the focus in the rail.
2. Try Up, Down, Home and End. On a terminal row press Left, then Left again, then Right. Press
   Enter on a row.
3. Press Ctrl+Alt+J to close the rail, restart Marley, and press Ctrl+Alt+J again.

- [ ] Left climbs from a row to its project, then folds it; Right unfolds; Enter opens the row as
  a click does.
- [ ] The rail stays closed across the restart and comes back at its old width.

### 2.6 The filter (#457)

1. With the focus in the rail, press Ctrl+F and type `serv`.
2. Type something that matches nothing. Then press Escape twice.

- [ ] The `server` row stays, with the matching letters highlighted; Enter opens it.
- [ ] `No matches` shows for text that matches nothing. A terminal also matches by its last
  command, so `sleep` finds the terminal from 2.4.
- [ ] The first Escape clears the filter, the second returns to the rows.

### 2.7 The switcher and next thread (#454, #459)

1. With the focus in the rail, hold Ctrl, press Tab twice, and let go.
2. Run `multi workspace: next thread` from the palette a few times.

- [ ] The switcher lists terminals and threads, the one used last first; letting go of Ctrl opens
  the selection.
- [ ] Next thread walks the rows in order and wraps at the end.

### 2.8 Port rows (#521)

In the `server` terminal, run:

```sh
npm run dev
```

- [ ] Within about three seconds a row appears under `marley-tour`: a server icon, `:8000` with
  the process's name, and a URL on `127.0.0.1`, since the server listens on `0.0.0.0`.
- [ ] Pointing at it shows the command line, folder and pid, with Open, Copy and Stop.
- [ ] One click on the row marks it and opens nothing; Enter then opens the URL in a Browser tab,
  and so does a double-click (#604).
- [ ] Optional, a server run by a systemd service (#603): in a terminal of `marley-tour`, run
  `systemd-run --user --unit=marley-tour-web -p Restart=always --working-directory=$PWD python3
  -m http.server 8010`. Its row shows `marley-tour-web.service` under the URL; Stop's tooltip
  says "Stop the User Service"; after Stop the row goes and stays gone, and
  `systemctl --user is-active marley-tour-web` prints `inactive`. Then run
  `systemctl --user reset-failed marley-tour-web`.

Leave the server running: Parts 5 and 7 use it.

### 2.9 Lines changed and the pull request (#531)

1. Ctrl+P, type `README`, open `marley-tour`'s `README.md`, add a line, and save with Ctrl+S.
2. Point at the counts that appear at the right of the project's header.
3. Put the file back: `git restore README.md` in a `marley-tour` terminal.

- [ ] Within a second or two of the save the header shows the lines changed, `+1` added and none
  removed.
- [ ] The tooltip names the base the counts are against, `main` here.
- [ ] The counts go after the restore.
- [ ] Optional: a repository whose branch has a pull request on GitHub, with `gh` signed in, shows
  the PR's icon and number beside the counts, colored by its state.

### 2.10 Groups with no folder (#600)

1. Right-click the rail's empty space under the last row. Choose New Group…, type `Scratch` and
   press Enter.
2. Click Scratch's `+`, choose New Terminal, and run `pwd`.
3. Right-click the empty space again and choose New Terminal.
4. Right-click Scratch's header: Rename Group… to `Tools`, then Remove Group.

- [ ] The menu offers New Group…, New Terminal, New Browser Tab and the agent CLIs.
- [ ] Scratch lists after your projects with a group icon, a chevron and a `+`; its `+` has no
  New Agent Thread, worktree or Launch entry.
- [ ] Its terminal's `pwd` prints your home folder.
- [ ] Step 3 makes a Home group with a terminal in `~`.
- [ ] Rename shows Tools; Remove takes the group and its rows away.
- [ ] Before removing it, quit (`zed: quit`) and start Marley from the app menu: the group comes
  back with its name, its terminals in their folders and its Browser tabs (#601).

### 2.11 Drag to reorder (#602)

1. With a group and `marley-tour` listed, drag the group's header up onto `marley-tour`'s header
   and hold before letting go.
2. Let go. Then drag the last terminal row of `marley-tour` onto its first terminal row.
3. Drag a terminal row onto another project's header and let go.
4. Right-click `marley-tour`'s header and choose Move Project Down.
5. Quit (`zed: quit`) and start Marley from the app menu.

- [ ] While dragging, a card with the name follows the pointer, a line runs above
  `marley-tour`'s block, and nothing else in the rail moves.
- [ ] The group lists above `marley-tour`, and the terminal you dragged is its first row.
- [ ] Step 3 changes nothing: the row goes back.
- [ ] Move Project Down moves `marley-tour` one place down, past a group as well as a project.
- [ ] After the restart, the headers and rows are in the order you left them.

## Part 3. The editor

Marley's editor is Zed's. This part checks the everyday tools. The first time you open
`site/app.js`, Zed downloads its JavaScript language server, so allow a minute before 3.2's
language features answer.

### 3.1 Find things

| Key | What it opens |
|---|---|
| Ctrl+P | Go to a file by name |
| Ctrl+Shift+P, F1 | The command palette |
| Ctrl+Shift+F | Search the whole project |
| Ctrl+F | Search this file |
| Ctrl+G | Go to a line (in an editor; in an agent's terminal Ctrl+G is rich input) |
| Ctrl+Shift+O | This file's symbols |
| Ctrl+T | The project's symbols (in a Browser tab, Ctrl+T is a new tab) |
| Ctrl+Alt+O, Ctrl+R | Recent projects (in a Browser tab, Ctrl+R reloads) |
| Ctrl+Shift+E | The project panel, Zed's file tree |

- [ ] Ctrl+P, `app`, Enter opens `site/app.js`.
- [ ] Ctrl+Shift+F for `greet` lists the definition and the call in `site/app.js`.
- [ ] Ctrl+Shift+O in `app.js` lists `greet`.

### 3.2 Edit and navigate code

In `site/app.js`:

1. Put the cursor on `greet` in `console.log(greet(...))` and press F12.
2. Press F2 on `greet`, type `greetUser` and press Enter. Then undo with Ctrl+Z.
3. Put the cursor on `document` and press Ctrl+D three times.
4. Press Alt+Up on a line, then Alt+Down.
5. Press Ctrl+/ on a line, then again.

- [ ] F12 jumps to `function greet`; Alt+Shift+F12 lists every reference.
- [ ] F2 renames both uses at once.
- [ ] Ctrl+D adds a cursor at each next `document`.
- [ ] Alt+Up and Alt+Down move the line; Ctrl+/ comments it and back.
- [ ] Ctrl+. offers code actions, Ctrl+K Ctrl+I shows hover docs, F8 moves to the next diagnostic
  and Ctrl+Shift+M opens the diagnostics list.

### 3.3 Panes, docks and zoom

- [ ] Ctrl+\\ in an editor splits it to the right; Ctrl+K then Ctrl+Left (or another arrow) moves
  between panes.
- [ ] Shift+Escape zooms the active pane and back.
- [ ] Ctrl+Alt+B toggles the right dock with the Agent Panel.
- [ ] In a terminal, Ctrl+Shift+5 splits it to the right and Ctrl+Shift+W closes it.

### 3.4 Git

1. Edit `notes/todo.md` and save.
2. Press Ctrl+Shift+G for the git panel. Stage the file with Space.
3. Run `git: diff` from the palette. Then put the file back with `git restore --staged
   notes/todo.md && git restore notes/todo.md` in a terminal.

- [ ] The git panel lists the changed file, stages it, and would commit with Ctrl+Enter.
- [ ] `git: diff` opens the project's diff, one excerpt per changed hunk.
- [ ] In the editor, the gutter marks changed lines; Ctrl+' expands a hunk, and Alt+G B shows
  blame.

### 3.5 Markdown preview and Insert in Terminal (#530)

1. Click in a `marley-tour` terminal first, so it is the terminal you used last.
2. Open `runbook.md` and press Ctrl+Shift+V.
3. Click Insert in Terminal on the `git log --oneline` block, then on the two-line block.
4. Start `python3` in that terminal and click Insert in Terminal again. Exit Python afterwards.

- [ ] The preview draws the file, and each `sh` block has Insert in Terminal beside Copy.
- [ ] The command lands at the prompt without running, and the terminal comes forward; Enter
  runs it.
- [ ] The two lines go in as one paste.
- [ ] While Python runs, nothing is typed and a toast says why.

### 3.6 Settings, themes, extensions

- [ ] Ctrl+, opens the Settings window; Ctrl+Alt+, opens `~/.config/marley/settings.json`, which
  holds `"marley": { "layout": "marley" }`.
- [ ] Ctrl+K Ctrl+T picks a theme; Ctrl+Shift+X opens extensions; Ctrl+K Ctrl+S opens the keymap
  editor.
- [ ] Marley keeps its own settings, apart from a stock Zed's `~/.config/zed`.

### 3.7 Tasks run in the center (#441)

1. Press Alt+Shift+T and choose `say hello`.
2. Run `task: rerun` from the palette.

- [ ] The task runs in a terminal in the main area, not in a bottom panel.
- [ ] Rerun reuses that terminal.

## Part 4. The block terminal

### 4.1 Is the shell integrated? (#463, #465, #520)

In a `marley-tour` terminal:

```sh
echo "$SHELL integration=$MARLEY_SHELL_INTEGRATION id=$MARLEY_TERMINAL_ID project=$MARLEY_PROJECT"
```

- [ ] It prints your bash or zsh, `integration=1`, an id, and the project's folder.
- [ ] With fish, or a `terminal.shell` with arguments of its own, the integration is empty and
  that terminal makes no blocks (fish waits on TICKET-466).

### 4.2 Read the blocks (#470)

Run each in turn: `echo hi`, `false`, `sleep 5`, then `less README.md` and quit it with `q`.

- [ ] `echo hi`: a green bar in the left margin and a check pill at the right of its first row.
- [ ] `false`: a red bar, the pill `1`, and a faint red tint.
- [ ] `sleep 5`: a blue bar, `running`, and a faint blue tint until it turns green.
- [ ] Nothing is drawn while `less` holds the screen.

### 4.3 Move between blocks and select them (#473, #554)

1. Run `seq 1 200`, then `echo a` and `echo b`.
2. Press Ctrl+Up. Press Up twice, then Down.
3. Press Ctrl+Down until the outline goes. Then Ctrl+Up again and Escape.

- [ ] Ctrl+Up outlines the newest block and scrolls to it; Up and Down move the outline block to
  block.
- [ ] Ctrl+Down past the last block ends the selection and returns to the live screen; Escape or
  typing ends it too.
- [ ] The first time you use a key Marley takes from terminal programs (Ctrl+Up, Ctrl+Down,
  Ctrl+G, Ctrl+I, Ctrl+Alt+N from a terminal), a toast names the key and offers Open Keymap
  (#563). Each note shows once.

### 4.4 Buttons on a hovered block (#474, #528, #558, #559)

Point at the `echo hi` block.

- [ ] Buttons appear beside its pill: Copy Output, Rerun Command, and the Filter, Bookmark, Find and
  Save as Workflow (a book) buttons.
- [ ] Copy Output puts `hi` on the clipboard.
- [ ] Rerun Command clears what you had typed and runs `echo hi` again. It shows only while the
  shell waits at its prompt, and only for a command the shell itself reported.

### 4.5 The block menu, reinput and Markdown (#554)

1. Right-click the `false` block.
2. Choose Copy as Markdown and paste it into an editor.
3. Select the `echo a` block (Ctrl+Up, then Up until it is outlined) and press Ctrl+Shift+I.

- [ ] The menu has a Block section: Copy Command, Copy Output, Copy Both, Copy as Markdown, Reinput
  and Reinput with sudo. It also offers Send to Agent, Find in Block and a bookmark.
- [ ] The Markdown holds the command and output in a code block, then the exit code, how long it
  ran, and its folder and branch.
- [ ] Ctrl+Shift+I types `echo a` at the prompt without running it.

### 4.6 The sticky command header (#529)

Scroll up into the middle of the `seq 1 200` block.

- [ ] Its command, `seq 1 200`, is pinned over the terminal's top row with its check.
- [ ] A click on it scrolls to the block's start.
- [ ] It never shows on the live screen. Sticky Command Header (Terminal section of the Marley
  page) turns it off.

### 4.7 Filter a block's output (#528)

1. Select the `seq 1 200` block with Ctrl+Up (Up until it is outlined), then press Alt+Shift+F.
2. Type `7`. Turn on the regular expression toggle and type `^1[0-9]$`. Type `(`.
3. Try the invert toggle and the context lines. Press Escape, then Alt+Shift+F again.

- [ ] A panel over the terminal lists only the matching lines, highlighted.
- [ ] `^1[0-9]$` keeps 10 to 19; `(` says the expression is invalid and keeps the last list.
- [ ] Invert keeps the lines that do not match; context lines add neighbors with `--` between
  groups.
- [ ] Escape shows the block untouched; reopening keeps the query.

### 4.8 Bookmarks and find within a block (#559)

1. Select the `seq 1 200` block and press Ctrl+Shift+B. Run a few more commands.
2. Press Alt+Up, then Alt+Down.
3. Select the `seq` block again and press Ctrl+Shift+F, search `15`, and press Enter a few times.

- [ ] The bookmarked block shows a bookmark before its exit mark and a tick at the terminal's
  right edge.
- [ ] Alt+Up and Alt+Down scroll to the bookmark before or after the view.
- [ ] The search walks only that block's matches, with the block outlined; Escape ends it.
  Ctrl+Shift+F with no block selected searches the whole terminal.

### 4.9 Save a command as a workflow (#558)

1. Run the line below (the dev server from 2.8 is still up).

```sh
curl -s http://localhost:8000/page2.html | head -3
```

2. Point at its block and click Save as Workflow. Name it `fetch page`, describe its parameters
   if you like, and press Enter.
3. Press Alt+Shift+T and pick `fetch page`. Change the URL parameter and press Enter.
4. Commit the change so the tree stays clean for Part 9:

```sh
git add .zed/tasks.json && git commit -qm "Add the fetch page workflow" && git log --oneline -1
```

- [ ] The editor shows the command with the URL already a `{{...}}` parameter, its typed value as
  the default.
- [ ] Save adds a task to `.zed/tasks.json` and keeps the tasks that were there.
- [ ] Running it asks for each parameter, prefilled, then runs the filled command as a task.

### 4.10 Autosuggestions (#484)

1. Run `git status`.
2. Type `git s` and press →.
3. Type text no command starts with and press →.

- [ ] The rest of `git status` shows dimmed after the cursor, and → types it.
- [ ] With nothing suggested, → moves the cursor. Commands from your history file are suggested
  too.

### 4.11 A desktop notification from a terminal (#478)

1. Run the first line and click another pane within five seconds.
2. Run the second line and stay in that terminal.

```sh
sleep 5; printf '\e]777;notify;Build;finished\a'
```

```sh
sleep 5; printf '\e]9;Tests passed\a'
```

- [ ] A notification titled `Build` reads `finished`; clicking it brings the terminal forward, and
  its row carries the dot.
- [ ] The second shows nothing, since you were looking at that terminal.

### 4.12 Terminals in the center (#441, #449)

1. From an editor, press Ctrl+\`, then Ctrl+\` again.
2. Press Ctrl+~.
3. In the project panel (Ctrl+Shift+E), right-click `site` and choose Open in Terminal.

- [ ] Ctrl+\` goes to the terminal you used last, and back to the editor.
- [ ] Ctrl+~ opens a new terminal in the main area.
- [ ] Open in Terminal opens one in `site`. Nothing opens the bottom panel.

### 4.13 Blocks survive a resize (#544)

In a terminal with several blocks, press Ctrl+Shift+5 to split it, or drag the pane edge to
make it narrow, then wide.

- [ ] The bars and pills stay on their own rows as long lines rewrap.

### 4.14 The error a running command prints (#572)

This needs System One's Running Error use, which Part 10 turns on. Come back to it from 10.7.

### 4.15 Blocks over ssh (#526), optional

With a host you reach by `ssh <host>` whose shell is bash or zsh:

1. Run `ssh <host>`, then `echo remote`, `false`, and `exit`.

- [ ] Each command on the host is a block with its exit code and Copy Output, and nothing is
  installed there.
- [ ] Rerun is offered for the host's blocks only while the host's shell waits at its prompt.

### 4.16 A remote terminal that survives a dropped link (#543), optional

Needs a host in your settings' `ssh_connections` with tmux 3.3 or later.

1. Run `marley: open remote terminal` and pick the host. Run `sleep 600`.
2. Drop the connection (sleep the laptop, or cut the network), then run `terminal: rerun task`.

- [ ] The tab says the terminal ended when the link dropped.
- [ ] Rerun attaches the same tmux session, with what it printed meanwhile and `sleep` still
  running.

## Part 5. Agent CLIs in terminals

### 5.1 Start Claude Code from the rail (#440, #477)

Click `marley-tour`'s `+`, then Claude Code under Agent CLIs. Ask it: "What files are in this
project?"

- [ ] A new terminal opens at the project root with `claude` typed into it.
- [ ] Its row shows Claude Code's icon, the title Claude Code sets, and `Claude Code · working`,
  then `waiting` once it has been quiet for two seconds.
- [ ] The agent bar under the terminal shows the icon and name, `+` (Attach File), a pencil (Rich
  Input), a microphone when Voxtype is installed, the Connect Claude Code to Marley chip, and at
  the right the folder and the branch `main`.

### 5.2 Connect Claude Code to Marley (#482, #500, #547)

1. Click Connect Claude Code to Marley.
2. In the session, run `/reload-plugins`.

- [ ] A toast says Marley's plugin is installed, and the chip goes away.
- [ ] The plugin sits in `~/.local/share/marley/claude-code/` as version 1.6.0.
- [ ] An older plugin shows Update Marley's plugin instead; restart Claude Code after it.

### 5.3 What Claude Code is doing, on its row (#519)

Ask Claude Code: "Read site/app.js and tell me what each button does."

- [ ] The row shows your prompt, then the tool in flight (`Read`), then its last message once it
  stops.

### 5.4 Notifications that say what happened (#538)

Ask Claude Code: "Create hello.txt containing hi." At once click another pane.

- [ ] While it waits for your permission, a desktop notification reads `marley-tour: Claude needs
  input` over the tool it wants to use.
- [ ] Once you allow it and it stops, one reads `marley-tour: Claude finished` over its last
  message.
- [ ] The row keeps a dot until you look at the terminal.

### 5.5 Needs you (#508)

Ask for another file ("Create bye.txt containing bye"), and do not answer yet.

- [ ] **Needs you** and a count appear between the filter and the projects, with an entry naming
  Claude Code, `marley-tour`, what it asks, and how long it has waited.
- [ ] A click on the entry shows the terminal, where you answer. The entry leaves once you do.

### 5.6 Rich input (#481)

1. Press Ctrl+G. Type a line, Shift+Enter, another line, then Enter.
2. Press Ctrl+G, type a draft, press Escape, then Ctrl+G again.

- [ ] An editor opens above the bar; Enter sends both lines as one prompt.
- [ ] The draft survives Escape.

### 5.7 Attach files, images and copy (#479, #536)

1. Click the bar's `+` and choose `site/index.html` and `site/app.js`. Ask: "Compare these two."
2. Drop a PNG screenshot on the terminal.
3. Select a few lines of Claude Code's reply, copy them, and paste into an editor.

- [ ] Both full paths are typed into the prompt, quoted where needed.
- [ ] The image goes in as its raw path in a paste of its own, the form in which Claude Code
  attaches an image.
- [ ] The pasted reply has no leading indent shared by every line.

### 5.8 The microphone (#480), optional

With Voxtype installed, click the microphone, speak, and click it again.

- [ ] It turns red while recording and yellow while transcribing, and your words land in the
  prompt.

### 5.9 The agent reads your terminals (#491, #520, #516, #562)

1. In another `marley-tour` terminal, run `ls /nonexistent`.
2. Ask Claude Code: "Which command failed in my other terminal, and what did it print?"
3. In that terminal run `export DEMO_TOKEN=not-a-real-one` and `cat .env`. Ask: "What were the
   last two commands in that terminal, and what did they print?"

- [ ] Claude Code asks to use Marley's tools (`terminal_list`, `terminal_blocks`,
  `terminal_read`), then names the command, its exit code and the error.
- [ ] `terminal_list` marks Claude Code's own terminal `self`.
- [ ] It reports `export DEMO_TOKEN=[redacted: secret]` and the `.env` value hidden too. Your
  terminal still shows the value.

### 5.10 The agent runs commands at your prompt (#556, #553)

1. Ask Claude Code: "Use terminal_run to run `ls site` in my other terminal."
2. Ask: "Use terminal_run to run `rm -f bye.txt` there."

- [ ] `ls site` runs at once, as a block with an agent mark before its pill, and Claude Code gets
  its output.
- [ ] `rm` waits on a card under the terminal with Run and Refuse; Enter runs, Escape refuses.
- [ ] Agent Commands in History on the Marley page (Agents) keeps such commands out of your shell
  history when turned off, in terminals opened after the change.

### 5.11 The agent types into a running program (#525, #594, #595)

1. In a terminal, start `python3`.
2. Ask Claude Code: "Type 6*7 into the python in my other terminal and press Enter."
3. When it has typed, press Ctrl+I in that terminal, then click Hand Back. Then exit Python.

- [ ] A card under the terminal names the agent, the program and the text, with Allow and Deny.
- [ ] After Allow, `42` appears: the Enter ran the line instead of leaving it at `...` (#594).
- [ ] A bar shows what it typed, with Take Over; Ctrl+I takes over until Hand Back.
- [ ] The bar goes once Python exits (#595).

### 5.12 Send the editor's selection (#549)

Open `site/app.js`, select the three lines of `greet`, and press Ctrl+> (Ctrl+Shift+.).

- [ ] `@site/app.js#L1-3` lands at Claude Code's prompt, unsent, and its terminal takes the
  focus.
- [ ] With several agents running, a picker asks which; with none, the key adds the selection to
  Zed's Agent Panel as in Zed.

### 5.13 Send a block to the agent (#555)

1. In the other terminal, run `ls /nonexistent` again.
2. Click Ask the agent on that failed block.
3. Select an earlier block with Ctrl+Up and press Ctrl+Shift+Enter.

- [ ] Ask the agent shows on the newest failed block while an agent runs in another terminal.
- [ ] The block lands at Claude Code's prompt, unsent: short output as Markdown, long output as a
  reference Claude Code reads with `terminal_read`.
- [ ] Nothing is typed while Claude Code waits on a permission or a question.

### 5.14 English at the prompt (#557)

1. In a plain `marley-tour` terminal, type `what is using port 8000` and do not press Enter.
2. Press Ctrl+Shift+Enter.
3. Run `show me the biggest files here`.

- [ ] A dimmed hint follows the line, and Ctrl+Shift+Enter hands it to the Claude Code running in
  the window.
- [ ] The third line fails with `command not found` (exit 127), and its block offers Ask the
  agent.
- [ ] English at the Prompt on the Marley page turns the hint off.

### 5.15 Each turn as its own diff (#509)

Under Claude Code's row, open **Turns (N)**.

- [ ] Each turn that changed files is listed, newest first: its prompt and `· 1 file`.
- [ ] A click opens Zed's commit view with only that turn's changes.

### 5.16 Review notes to the agent (#522)

1. Run `git: diff` from the palette.
2. Point at a changed line of `hello.txt`, choose Add Review in the gutter, type "Say hello to
   the world instead" and press Enter.
3. Click **Send Review to Agent (1)** in the diff's toolbar.

- [ ] A picker lists Claude Code's terminal as `ready` while it idles at its prompt.
- [ ] Picking it pastes the note as one prompt, presses Enter and shows the terminal.
- [ ] The note stays in the diff with Sent beside it.

### 5.17 The New Agent picker and other CLIs (#450, #552), optional

1. Press Ctrl+Alt+N and type `cod`, then Enter.
2. Under Codex's agent bar, click Turn on Codex notifications. Under OpenCode, click Connect
   OpenCode to Marley.

- [ ] The picker lists Zed's agents marked Thread and the installed CLIs marked Terminal.
- [ ] Codex starts in its own terminal with its own row and agent bar.
- [ ] Turn on Codex notifications writes three keys under `[tui]` in Codex's `config.toml` and
  keeps the rest of the file. After it, Codex posts a notification when a turn ends in a
  terminal you are not looking at.

### 5.18 Permission modes (#532)

1. On the Marley page (Agents), set Claude Code Permissions to bypass.
2. Start Claude Code from the `+`, then set the setting back to ask.

- [ ] Marley types `claude --dangerously-skip-permissions`.
- [ ] The row carries a `bypass` chip in the warning color, whose tooltip says where Marley read
  it.
- [ ] The first bypass start asks you to accept Claude Code's warning in the terminal.

`agent_permissions_by_project` sets the same per folder; a repository's own
`.zed/settings.json` cannot.

### 5.19 Close protection and undo (#550)

1. Ask Claude Code something that takes a while ("Explain every file in this project in
   detail").
2. While it works, close its terminal tab (Ctrl+Shift+W in the terminal) and choose Close.
3. Within a minute, press Ctrl+Shift+T.

- [ ] Marley asks first and names the agent: `marley-tour · Claude Code · working`, with Close,
  Show and Cancel.
- [ ] A toast offers Undo; Ctrl+Shift+T brings the terminal back with its scrollback, the agent
  still running.
- [ ] Quitting Marley while an agent works asks the same way.

### 5.20 No password prompts for agents (#537, #596)

In the shell of a terminal Marley opened for an agent (exit Claude Code in one), run:

```sh
echo "git_prompt=$GIT_TERMINAL_PROMPT gcm=$GCM_INTERACTIVE askpass_require=$SSH_ASKPASS_REQUIRE"
```

- [ ] It prints `git_prompt=0 gcm=never askpass_require=force`. A plain terminal prints them
  empty.
- [ ] Optional: with an ssh key whose passphrase is not in your agent, ask Claude Code to run
  `ssh -T git@github.com`. Marley brings the terminal forward and shows a password dialog headed
  `ssh for Claude Code in marley-tour`. The passphrase goes to ssh alone; Escape fails the
  command at once.

### 5.21 The rail orders by attention (#542)

1. Start Claude Code in `marley-tour-b` too, and give each agent a task.
2. Leave one waiting on a permission. Fold `marley-tour-b`.

- [ ] Projects and rows sort by attention: waiting or failed first, then finished and unseen, then
  working, then quiet, then idle.
- [ ] A folded project's header counts its agents, as in `1 waiting, 1 working`.
- [ ] While the pointer is over the rail nothing moves. `marley.rail_order: "window"` (Marley page,
  Layout) keeps the window's order.
- [ ] A working row with no event for 30 minutes reads `no update in N m` (#547); the minutes are
  a Marley setting.

### 5.22 Pushes to your phone (#535), optional

Needs an ntfy server on this machine. In `settings.json`:

```json
"marley": { "push": { "url": "http://127.0.0.1:<port>", "topic": "marley" } }
```

- [ ] When Claude Code needs input, finishes or fails in a terminal you are not looking at, one
  line such as `marley-tour: Claude needs input` arrives, with nothing the agent wrote.

## Part 6. Zed's Agent Panel

Part 6 needs an agent set up in the Agent Panel: a model for the Zed Agent, or an external
agent such as Claude Agent. Ctrl+Alt+C in the panel opens its settings.

### 6.1 Threads in the rail (#439, #468)

1. Click `marley-tour`'s `+`, then New Agent Thread, and pick an agent.
2. Send a prompt, and click a terminal while it works.

- [ ] A thread row appears under the project's terminals, reading `<agent> · working`, then idle.
- [ ] Its dot lights when the run ends while the thread is off screen.
- [ ] A click opens the thread, focused, in the Agent Panel on the right.
- [ ] Pointing at the row shows Archive at its end, "Archive Thread" on hover; a click archives the
  thread and its row goes, without opening it (#605). A right-click on another thread's row offers
  Archive Thread too. Archived threads stay out after a restart.

### 6.2 Zed's agents use Marley's tools (#501)

1. In the thread, ask: "List my Marley terminals and tell me which command failed."
2. Add `"context_servers": { "marley": { "enabled": false } }` to your settings, start a new
   thread and ask again. Then take the setting out.

- [ ] The agent calls the `marley` server's terminal tools and answers.
- [ ] With the setting, it has no Marley tools.

### 6.3 Answer from the inbox (#508)

Ask the thread's agent to do something that needs your confirmation, such as running a command.

- [ ] **Needs you** lists it with Deny and Allow, which answer the call as the thread's own
  buttons do.

## Part 7. The Browser tab

The dev server from 2.8 should still serve `site/` on port 8000. If it stopped, run `npm run
dev` in the `server` terminal.

### 7.1 Open a Browser tab (#488, #490, #500, #504)

1. Click `marley-tour`'s `+`, then New Browser Tab. Type `localhost:8000` and press Enter.
2. Press Ctrl+L, type `zed editor`, and press Enter. Press Alt+Left.

- [ ] The first time, the tab reads `Starting Chromium…`, then shows the page at the tab's size.
- [ ] The tab carries the page's title and a globe icon, with the URL as its tooltip.
- [ ] A row for the tab appears under `marley-tour` in the rail: the page's icon (the yellow `M`),
  its title, and `localhost:8000` (#504).
- [ ] `localhost:8000` loads over http; plain words go to a DuckDuckGo search.

### 7.2 Pages from the terminal (#503, #579, #561, #586)

1. In the `server` terminal, look at the footer.
2. In another terminal run `echo http://localhost:8000/page2.html`. Click the URL; then
   Ctrl+click it; then Shift+Ctrl+click it.
3. Run each line below.

```sh
python3 -m webbrowser http://localhost:8000/page2.html
```

```sh
python3 -m webbrowser "file://$HOME/marley-tour/site/page2.html"
```

- [ ] The server terminal's footer offers `localhost:8000`; a click opens it, and its menu opens
  it in your browser or copies it (#503).
- [ ] A plain click on the URL opens a menu: Open in Browser Tab, Open in System Browser, Copy Link
  (#579).
- [ ] Ctrl+click opens it in a Browser tab of the project, or brings forward the tab already on
  it; Shift+Ctrl+click opens it in your system browser.
- [ ] Both `webbrowser` lines open in a Browser tab of `marley-tour`, with the focus (#561,
  #586).

`marley.terminal_links` (Marley page, Terminal) sends every URL to a Browser tab, or every URL to
your browser.

### 7.3 Browse as in a browser (#489, #490, #493, #495)

On the tour page:

1. Click Say hello. Type your name in the field.
2. Click the "Open page two in a new tab" link. Press Ctrl+T.
3. Click the select list. Then Tab to it and press Alt+Down.
4. Click Show an alert and press Enter.
5. Select the heading's text, press Ctrl+C, and paste in an editor.

- [ ] The link opens page two in a new tab beside its opener, with the focus.
- [ ] Ctrl+T opens a blank tab with the focus in its address bar.
- [ ] The select opens as Marley's list: the Fruit and Vegetables groups, `kale` greyed out.
- [ ] The alert shows as a card, `localhost:8000 says`, and Enter answers it.
- [ ] The copied text is on the system clipboard. Ctrl+S and Ctrl+W stay Zed's (save, close the
  tab).

### 7.4 The title follows the page (#582)

Click "Change the title in two seconds".

- [ ] Two seconds later the tab and its rail row both read `Title set later`.

### 7.5 Tabs that come back (#494, #507)

1. Scroll the page and type into its field.
2. Quit Marley and start it again.
3. Run the line below.

```sh
systemctl --user list-units 'marley-browser-*'
```

- [ ] Each tab is back on its own page, with your text and your scroll.
- [ ] One `marley-browser-…` unit runs for each project that has a Browser tab. Chromium keeps
  running after Marley quits, so the tabs find their pages.

The quit ended the dev server with its terminal. Run `npm run dev` in the `server` terminal again
before 7.7.

### 7.6 A browser per project, and clearing one (#507, #581)

1. Open a Browser tab in `marley-tour-b` (its `+`, New Browser Tab, `localhost:8000`).
2. Right-click `marley-tour-b`'s header and choose Clear Browser Data….

- [ ] A second `marley-browser-…` unit starts: `marley-tour-b` has its own Chromium and profile, so
  a site you sign in to in one project stays signed out in the other.
- [ ] After you confirm, `marley-tour-b`'s tabs close and a toast says its data is cleared;
  `marley-tour`'s tabs stay.

### 7.7 Let the agent drive (#492, #574)

Keep the focus in Claude Code's terminal and ask: "Open localhost:8000 in the browser, type Ada in
the name field, click Say hello, and tell me what the console says."

- [ ] Claude Code asks to use the browser tools, and the page acts without taking your focus.
- [ ] The toolbar's Agent chip names each action while it runs; typed text shows as its length.
- [ ] The answer quotes `hello, Ada`.
- [ ] It acts in `marley-tour`'s own Browser tab, and the tab's rail row is marked when the agent
  acted while no tab showed it.

### 7.8 Pick an element for the agent (#496, #518)

1. Click in Claude Code's terminal first, so it is the terminal you used last.
2. In the Browser tab, press Ctrl+Shift+C and move over the page. Click Say hello.
3. Type "Make this button green" in the caption and press Enter. In the terminal, press Enter.

- [ ] Chromium's highlight follows the pointer, and the click does not reach the page.
- [ ] `Pick 1` waits in a tray under the toolbar, and the tab's rail row counts it.
- [ ] Enter types a line such as `[browser pick 1: button “Say hello” on localhost:8000;
  browser_pick id 1] Make this button green` into the terminal and takes you there.
- [ ] Claude Code reads the pick with `browser_pick`: locators (the `say-hello` test id first),
  HTML, styles and nearby texts, and changes the page's files.

### 7.9 Check the fix (#505)

After Claude Code's change, reload the tab with Ctrl+R and click Check on Pick 1.

- [ ] Marley finds the button again and opens a card with the crop at the pick beside the crop
  now, and what changed (its styles here).
- [ ] The pick's row keeps the verdict, such as `1 change`, which opens the card again.

### 7.10 From a pick to its source (#497)

- [ ] On the tour page, a pick of Say hello shows its click listener as `app.js` and a line,
  muted: the script has no source map, so it opens nothing.
- [ ] Optional: on a Vite TypeScript project (`npm create vite@latest demo -- --template
  vanilla-ts`, then `npm install` and `npm run dev` inside the project), a picked element's
  listener reads `click src/<file>.ts:<line>`, and a click opens that line in the editor.

### 7.11 Annotate the page (#498)

1. Click the pencil. Drag a box around the heading, type "too small", and press Enter.
2. Ask Claude Code: "Draw a box around the Place order button with the note 'this one', then list
   the annotations."
3. Click your note and press Delete. Load page two.

- [ ] Your box uses the theme's warning color; the agent's uses its accent color, with a sparkle.
- [ ] Both stay on what they mark as you scroll, and the tab's rail row counts them.
- [ ] Delete removes yours, and page two starts clean.

### 7.12 Record this, and a test from it (#499, #506)

1. On the tour page, click Say hello, type in the field, then click Fetch a missing file.
2. Click the red dot in the toolbar.
3. Tell Claude Code: "It broke just now. Read the latest recording and tell me what I did."
4. Then: "Draft a Playwright test from that recording."

- [ ] A toast names the recording; its files are in
  `~/.local/share/marley/browser/recordings/<id>/`.
- [ ] Claude Code lists your clicks, keys, the number of characters typed (never the text), the
  console error and the 404 for `/missing.json`.
- [ ] `browser_draft_test` answers a Playwright test using `getByTestId('say-hello')` and a path
  under `tests/`. It writes nothing itself.

### 7.13 Console, network and hidden values (#492)

1. Load `localhost:8000/?token=abc` and click Fetch a missing file.
2. Ask Claude Code what failed on the page.

- [ ] It reports the 404 from `browser_network`, which holds no headers or bodies, and the
  console line.
- [ ] The page's own address shows with the `token` value hidden.

### 7.14 Playwright scripts (#523)

1. Click the Playwright Scripts button in the toolbar (or run `marley: playwright scripts`).
2. Type the name `hello` and press Enter.
3. Click Run on the script.

- [ ] A new script from Marley's template opens in the editor, kept in Marley's config folder and
  never in the repository.
- [ ] Run runs it on the tab's page in a terminal beside the tab, and the tab stays in front.
- [ ] The first run installs `playwright-core` 1.63.0 into Marley's data folder, inside that run's
  block, which needs the network.
- [ ] A script that fails saves the tab's last minute as a recording, named in a toast.

### 7.15 Browser Clients (#524, #584), optional

1. Run `marley: browser clients`, type a name such as `playwright-mcp`, choose read or act, and
   press Enter.

- [ ] The client gets its own token in an endpoint file only you can read.
- [ ] A tab it acts in shows `Driven by playwright-mcp` with Cut Off, which refuses its token at
  once.
- [ ] The list offers an SSH line with Copy, for a client on another machine; the line holds no
  token.

## Part 8. Launch configs

### 8.1 Run a project's launch config (#527, #592)

1. Stop the dev server from 2.8 first: point at the `:8000` row and click Stop, or press Ctrl+C in
   the `server` terminal.
2. Click `marley-tour`'s `+`. Under Launch, choose Dev.
3. Read the question and click Run.

- [ ] The question shows each of the three items on its own line, exactly as `.zed/marley.json`
  has it.
- [ ] A `dev server` terminal runs `npm run dev`, Claude Code opens in a split to the right with
  the focus, and a Browser tab opens below, waiting up to 30 seconds for port 8000.
- [ ] Change `"dev server"` to `"web"` in `.zed/marley.json` and choose Dev again: it asks again
  and says the config changed. Click Cancel, then put the file back with `git restore
  .zed/marley.json`. An unchanged config runs without asking.

## Part 9. Worktree agents

Several agents can work on one repository at once, each in a git worktree and branch of its own.
Start with a clean tree:

```sh
git status --short
```

Commit or restore anything it lists. `hello.txt` and the page changes from Part 5 and 7 are
Claude Code's; commit them with `git add -A && git commit -qm "Tour changes"`.

### 9.1 Start an agent in a worktree (#510, #585, #587)

1. Click `marley-tour`'s `+`, then New Agent in Worktree, then Claude Code.
2. Tick the `Setup: npm install (package-lock.json found)` box.
3. Type: "Add a footer to site/index.html that says Built in a worktree, and commit it." Press
   Enter.

- [ ] The prompt names what it will make: `agent/<name> from main`.
- [ ] The worktree appears as a row under `marley-tour`, with its name and branch, and its
  terminal under it. You stay where you were.
- [ ] Its terminal runs `npm install && claude …`, so Claude Code starts once the install passes.
- [ ] If Claude Code has not trusted this repository, a notification names the worktree, with
  Trust Folder and Show Terminal.
- [ ] The worktree is in `~/worktrees/marley-tour/<name>/marley-tour`.

### 9.2 The worktree's environment (#585, #590)

Click the worktree's row, press Ctrl+~ for a terminal in it, and run:

```sh
ls -a; echo "PORT=$PORT offset=$MARLEY_PORT_OFFSET"; npm run dev
```

- [ ] `.env` is there, copied because `.worktreeinclude` names it and git ignores it.
- [ ] `PORT=3010 offset=10`: the first worktree gets 3010, the next 3020.
- [ ] The dev server prints `Serving HTTP on 0.0.0.0 port 3010`, clear of the main checkout's
  8000.

Stop that server with Ctrl+C before 9.6.

### 9.3 Drift from main (#560)

1. Wait until the agent has committed its footer: the worktree row's second line reads
   `agent/<name> · 1 ahead of main`.
2. In a terminal of the main checkout (`marley-tour`, not the worktree), run:

```sh
echo "Main moved on." >>README.md && git commit -qam "Move main" && git log --oneline -1
```

- [ ] About a second later the worktree's row shows `1 behind`, muted. Its tooltip reads `1 commit
  behind main` and names main's commit.
- [ ] Had both branches changed the same lines, it would show `1 conflict` in the warning color,
  with the file named in the tooltip.

### 9.4 Review the branch (#511, #522)

Right-click the worktree's row and choose Review.

- [ ] Zed's branch diff opens in the worktree's workspace: "Changes since main", with the footer.
- [ ] Add Review and Send Review to Agent work there as in 5.16.

### 9.5 Merge it (#511)

Right-click the row and choose Merge 1 commit into main…

- [ ] Marley checks the main checkout is on `main` with no changes, that the worktree has none,
  and that there is something to merge, and names any check that fails.
- [ ] After you confirm, a toast names the merge commit, `Merge branch 'agent/<name>' into main`.
  Nothing is pushed.

### 9.6 Remove it, with its teardown task (#589, #591)

Right-click the row and choose Remove….

- [ ] Marley asks first, and names uncommitted changes when git counts any (the button then reads
  Remove Anyway).
- [ ] The `tear down the worktree` task runs in a terminal of its own before the worktree goes,
  printing `tearing down` and the worktree's path.
- [ ] The worktree's workspace closes, its row goes, and the branch is deleted because it is
  merged. An unmerged branch would stay, and the toast would say why.

## Part 10. System One on local rules

System One asks a model typed questions about what Marley knows. It is off until you turn it on,
and every use can run on Marley's own rules alone, with no key and no request. This part uses the
`rules` provider, so nothing leaves the machine.

Open `settings.json` (Ctrl+Alt+,) and merge these keys into its `marley` block, beside
`"layout"`:

```jsonc
"marley": {
  "browser_click_pause_agents": "all_agents",
  "system_one": {
    "enabled": true,
    "provider": "rules",
    "projects": ["~/marley-tour"],
    "uses": {
      "check": "act",
      "stop_kind": "act",
      "stall_kind": "act",
      "browser_find": "act",
      "terminal_find": "act",
      "click_consequence": "act",
      "inbox": "act",
      "question_route": "act",
      "running_error": "act"
    }
  }
}
```

`browser_click_pause_agents: "all_agents"` makes Claude Code's clicks pause too; by default only
agents without permission prompts of their own wait.

### 10.1 Decisions and the check (#565)

1. Run `false` in a terminal, then `marley: system one check`.
2. Run `marley: open decisions`.

- [ ] A toast gives the reading, with `rules` as the provider.
- [ ] Decisions lists the day's calls, newest first, and a click opens one to what was sent and
  what came back.

### 10.2 What a stop needs (#566)

1. Ask Claude Code: "Ask me which color I want the button to be, then stop."
2. Ask it something long and press Escape while it works.

- [ ] The idle row reads `asks you` instead of `idle`.
- [ ] After Escape it reads `interrupted`.
- [ ] Your next prompt clears it.

### 10.3 A looping agent (#569)

Ask Claude Code: "Run `sleep 1` with the Bash tool three separate times in a row."

- [ ] The row gets a warning mark and `looping?`, whose tooltip names the repeated line. Marley
  never stops the agent; the flag only marks the row.

### 10.4 Risk and route in the inbox (#568, #570)

Ask Claude Code: "Run `rm -rf build`." Do not allow it.

- [ ] The Needs you entry carries a `destroys` chip and reads `for you`; the tooltip on the mark
  says `Marley's rule`.
- [ ] Deny it in the terminal. Nothing in the inbox answers anything by itself.

### 10.5 A consequential click waits for you (#571)

Ask Claude Code: "Click the Place order button on the tour page."

- [ ] A card under the toolbar reads that Claude Code wants to click button "Place order", which
  pays, on localhost:8000, with Refuse and Allow, and a toast with Show points to it.
- [ ] Refuse, or 25 seconds with no answer, tells the agent you did not allow it. Allow clicks,
  and the console logs `order placed (not really)`.
- [ ] Asked to click Say hello instead, it clicks at once: a plain button never waits.

### 10.6 Find tools for agents (#567)

1. In Claude Code, run `/mcp` and reconnect `marley`, so it lists the tools anew.
2. Ask: "Use browser_find to find the fetch button on the tour page, then click it."

- [ ] `browser_find` answers the button's ref from the query's words alone.
- [ ] `terminal_find` does the same for a line of a block's output.

### 10.7 A running command's printed error (#572)

Run the line below and click another pane at once.

```sh
echo "error: build failed"; sleep 8; echo "Compiled successfully"; sleep 20
```

- [ ] Five seconds after the error, the row shows a red × with the line, and a notification reads
  `marley-tour: <the command> printed an error`.
- [ ] After `Compiled successfully`, a second notification says it recovered.

### 10.8 Turn it off

Set `"enabled": false` under `system_one`, or remove the block, and put
`browser_click_pause_agents` back.

- [ ] Rows lose their stop kinds and flags, and Decisions records no new calls.

With a TypeSafe key, `"provider": "typesafe"` sends the questions the rules leave open to Jev; see
the guide's System One section before you do.

## Part 11. Living with Marley

### 11.1 Where your data lives

```sh
ls ~/.config/marley ~/.local/share/marley; ls -l ~/.local/share/marley/mcp-endpoint.json
```

- [ ] Settings and keymap in `~/.config/marley`; the database, logs, the Claude Code plugin, shell
  integration and browser profiles under `~/.local/share/marley`.
- [ ] `mcp-endpoint.json` exists while Marley runs, readable by you alone (`-rw-------`).

The guide's "Where data lives" table lists every path.

### 11.2 When something goes wrong

- [ ] `~/.local/share/marley/logs/Marley.log` is the log.
- [ ] `~/.local/share/marley/logs/stderr.log` holds a menu-started Marley's stderr, the only
  record of a panic on the dev channel.
- [ ] `ZED_LOG=marley_browser=debug marley` logs each input-to-frame time in the browser.
- [ ] `journalctl --user -u marley-browser-<id>` says why a Chromium stopped.
- [ ] `marley: use zed layout` tells a Marley problem from a Zed one.

### 11.3 A second Marley, safely (#513)

```sh
marley --user-data-dir ~/marley-scratch ~/marley-tour
```

- [ ] A second window behaves as a fresh install, with its own settings, logs and Chromium.
- [ ] Quit it and remove `~/marley-scratch` when done.

### 11.4 Update

```sh
cd /srv/stacks/marley_ide && git pull && just install
```

- [ ] The running Marley keeps its old build until you quit and start it again.

### 11.5 Clean up after the tour

1. Stop the dev servers (Stop on their port rows).
2. Right-click each tour project's header: Clear Browser Data…, then Remove Project.
3. Remove the folders:

```sh
rm -rf ~/marley-tour ~/marley-tour-b ~/worktrees/marley-tour ~/marley-scratch
```

- [ ] The tour's projects, browsers and folders are gone. Undo 10's settings if you have not.

## Part 12. The Fleet panel (#607)

This part needs only Marley and one settings line; do it before or after 11.5's clean-up.

1. Ctrl+Alt+, opens `settings.json`. Inside the `"marley"` block, add
   `"fleet": { "providers": [ { "kind": "pseudo" } ] },` and save.
2. Run `marley: toggle fleet` from the palette.

- [ ] The panel opens in the right dock with a FLEET header and "Pseudo provider" under it.
- [ ] Two hosts, build-1 and vps-2, each with a server icon. Under build-1: build-1 (`working`,
  `RB-142 · code 2/4`) and review-1 (`waiting`, a yellow warning mark, `RB-139 · test 3/4`).
  Under vps-2: docs-1 (`error`, a red mark, `RB-151 · code 2/4`).
- [ ] Within about 20 seconds, docs-1's chip reads `stale`.
- [ ] A minute on, build-1 reads `test 3/4`; a minute after that, `complete 4/4`.
- [ ] Click build-1 (#608): its row is highlighted and the panel splits, with its snapshot
  below: `working` and "for …", RB-142's title, a strip of four phases with one green and one
  blue, CPU and memory bars, and tokens today.
- [ ] Click review-1: its snapshot ends with the question "Delete the old dispatch module?" and
  Yes and No. Click docs-1: its strip has a red segment and "failed at code (2/4)".
- [ ] With the panel focused, Up and Down move the highlight and the snapshot follows.
- [ ] Drag the line above the snapshot up or down: the two parts resize.
- [ ] `marley: toggle fleet` again closes the panel, and once more opens it where it was.
- [ ] Change the line to `"fleet": { "providers": [] },` and save: the panel says "The fleet is
  not set up." and names the setting.

A setting saved by hand after a layout switch in the same session may not take until a restart
(TICKET-612); restart Marley if the panel does not change.

## Appendix A. Keys

Marley's keys load after Zed's defaults and before your keymap.

| Key | Where | What it does |
|---|---|---|
| Ctrl+Shift+P | Anywhere | Command palette |
| Ctrl+Alt+N | Anywhere | New Agent picker |
| Ctrl+Alt+; | Anywhere | Focus into the rail and back |
| Ctrl+Alt+J | Anywhere | Open or close the rail |
| Ctrl+? | Anywhere | Focus the Agent Panel |
| Ctrl+\` | Marley layout | Between code and the project's terminals |
| Ctrl+~ | Marley layout | New terminal in the main area |
| Ctrl+Shift+T | After closing a working agent | Undo the close |
| Ctrl+F, Ctrl+Tab | Rail | Filter; switcher |
| Ctrl+Up, Ctrl+Down | Terminal | Select the previous block; the next, or end the selection |
| Up, Down, Escape | Selected block | Move the selection; end it |
| Ctrl+Shift+I | Selected block | Reinput its command |
| Ctrl+Shift+Enter | Selected block | Send it to an agent |
| Ctrl+Shift+Enter | Terminal prompt | Hand the typed line to an agent |
| Ctrl+Shift+F | Selected block | Find within it |
| Alt+Shift+F | Terminal | Filter a block's output |
| Ctrl+Shift+B | Terminal | Bookmark a block |
| Alt+Up, Alt+Down | Terminal | Previous or next bookmark |
| → | Terminal prompt | Take the autosuggestion |
| Ctrl+G | Terminal with an agent CLI | Rich input |
| Ctrl+I | Terminal an agent typed into | Take over, and hand back |
| Enter, Escape | An agent's waiting command | Run; refuse |
| Ctrl+> | Editor | Send the selection to a terminal agent |
| Ctrl+Q | Terminal | Goes to the shell; quit with `zed: quit` |
| Ctrl+L, Ctrl+T | Browser tab | Address bar; new tab |
| Alt+Left, Alt+Right | Browser tab | Back, forward |
| Ctrl+R, F5 | Browser tab | Reload |
| Ctrl+Shift+C | Browser tab | Pick mode |
| Alt+Down, F4, Space | Focused select list | Open Marley's list |
| Enter, Escape | Page dialog, paused click | OK or Allow; Cancel or Refuse |

## Appendix B. Commands

Every Marley command runs from the palette. The ones with no key of their own:

| Command | What it does |
|---|---|
| `marley: open guide` | The Marley guide (also the `?` in the title bar) |
| `marley: use marley layout`, `marley: use zed layout` | Switch the layout in every window |
| `marley: open settings` | The Settings window on the Marley page |
| `marley: attach file` | Attach File for the focused terminal |
| `marley: toggle dictation` | Start or stop a Voxtype dictation |
| `marley: save as workflow` | Save the selected or newest block as a task |
| `marley: send selection to agent` | Ctrl+>'s action, in either layout |
| `marley: open remote terminal` | A terminal on a saved SSH host, in tmux |
| `marley: open browser`, `marley: new browser tab` | Show or open Browser tabs |
| `marley: pick element`, `marley: annotate`, `marley: record this` | The Browser tab's toolbar buttons |
| `marley: playwright scripts` | The Browser tab's scripts tray |
| `marley: browser clients` | Programs outside Marley allowed into its browser |
| `marley: clear project browser data` | Sign every site out for this project |
| `marley: system one check`, `marley: open decisions` | System One's check and its log |
| `marley: toggle fleet` | Show or hide the Fleet panel (#607) |
| `multi workspace: next project`, `previous project`, `next thread`, `previous thread` | Walk the rail |

## Appendix C. Not in yet

- fish shell integration (TICKET-466), which waits for a machine with fish.
- Marley's own release identity, keyring label, updater and `zed://` URL scheme (TICKET-445);
  `zed://` links open a stock Zed where one is installed.
- From `three-prong-plan.md`: block-scoped path links and a jump to the first failure (T2), a
  prompt editor with completions (T3, T6), tasks as blocks (T4), native block headers (T5), and
  harness seats and Rusty's sessions in the rail (C1 to C5).
