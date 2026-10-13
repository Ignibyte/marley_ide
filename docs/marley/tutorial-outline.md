# Marley from zero: the tutorial outline

The outline of a long tutorial that teaches Marley lesson by lesson, in the order a new user
would meet it. Each lesson has a goal, what you do, what you should see, and the feature it
teaches, with the ticket that shipped it so the writer can check the lesson against
`CHANGELOG.md`. The outline follows Marley as of 2026-10-12 (through #776) on Linux. The guide
(`guide.md`) is the reference each lesson points into, and the walkthrough (`walkthrough.md`) has
the stop-by-stop checks behind most lessons.

The order follows what later lessons need. The layout and the rail come first because every
terminal opens there. Blocks come before agents because agents read blocks. Claude Code gets
connected in part 4 because the browser lessons in part 6 have it drive the page. Parts 8 to 16
go back over each area for what the first seven parts leave out, in the same order, and end with
the parts that need something the reader may not have: Rusty, rustal-harness, a second machine or
Omarchy.

## What the reader needs

- Marley's checkout, with `cargo` and `just` (lesson 1 builds from it), and `musl-gcc` and
  `llvm-objcopy` (Arch's `musl` and `llvm` packages) for the remote server every install builds.
- Chromium and a systemd user session, for parts 6 and 13.
- Claude Code (`claude` on the PATH, signed in), for parts 4, 6 and 10 onward. Codex, Gemini CLI
  or OpenCode are optional extras for lessons 22 and 54.
- A Zed agent set up in the Agent Panel (a model for the Zed Agent, or an external agent such as
  Claude Agent), for parts 5 and 11.
- Voxtype, optional, for the microphone in lesson 20, with Voice turned on in the Marley settings.
- Optional, each for its own lessons: `gh` signed in to GitHub (lesson 38's pull request chip), a
  host you reach by `ssh` with a key (lessons 47 and 81), an ntfy server (lesson 56), a TypeSafe key
  or a Cloudflare account (lesson 65), Rusty's `rusty-mcp` (part 14), rustal-harness's `rh`
  (lessons 76 to 80), a second machine running rustal-harness (lesson 80), and Omarchy (lesson 83).

## The practice folders

The tutorial ships two small folders, made once before part 1.

- `marley-tutorial/`: a git repository with a few files and one subfolder, so the rail has
  directories and a branch to show.
- `marley-tutorial/site/`: an `index.html` with a heading, a button whose click listener logs to
  the console, a text field, a `<select>` with option groups and a disabled option, a link with
  `target="_blank"`, a button that calls `alert()`, and a button that `fetch`es a file that does
  not exist and logs the error. Lesson 25 serves it with `python3 -m http.server 8000` from a
  Marley terminal.

Lesson 30 also needs a TypeScript page served by a dev server that serves source maps (Vite's dev
server does), with its source inside the project.

Parts 8 onward use more of the folder, and a second repository:

- In `marley-tutorial/`: a `package.json` whose `dev` script serves `site/` on `$PORT` (8000 when
  unset), with its `package-lock.json`; a `.env` that git ignores and `.worktreeinclude` names; a
  `runbook.md` with two `sh` blocks; a `.zed/tasks.json` with a `say hello` task and a teardown task
  whose `hooks` hold `remove_worktree`; and a `.zed/marley.json` with one launch config, Dev: the dev
  server in a terminal, Claude Code split to the right, and a Browser tab on `localhost:8000` split
  below. `site/index.html` also gains a Place order and a Delete account button, for lesson 64.
  The walkthrough's stop 0.3 makes the same files for its own tour folder.
- `marley-tutorial-b/`: a second git repository with one file, for lessons 7, 48 and 67.

## Part 1. Install and first launch

### Lesson 1. Install Marley

- **Goal:** an installed Marley in your app menu.
- **Do:** in the checkout, run `just install`, and read the lines it prints at the end.
- **See:** it waits for any other cargo run, builds in the release profile (about ten minutes the
  first time on the dev box), then builds Marley's remote server for Linux x86_64 hosts and prints
  where it staged it (`remote server for linux-x86_64 at <commit>: …/remote_servers/dev/linux-x86_64/<commit>.gz`).
  It ends with `~/.local/bin/marley at <commit>` and the menu entry's path. It adds a note when
  `~/.local/bin` is not on your PATH, and a warning when a Marley is already running. Without
  `musl-gcc` or `llvm-objcopy` it stops before installing anything and names the missing tool.
- **Teaches:** `just install` and `--prefix`, the four installed files, the launcher (#502), and
  the remote server each install stages for SSH projects (#445; lesson 81 uses it).

### Lesson 2. Meet the Marley layout

- **Goal:** know the parts of a Marley window.
- **Do:** start Marley from the app menu. Find the rail on the left, its PROJECTS header, the Add
  Project button and the filter field. Press Ctrl+?, then Escape.
- **See:** the rail on the left and no Terminal Panel button in the status bar. Ctrl+? opens the
  New Agent picker ("Start an agent…") in place of Zed's Agent Panel. The title bar's `?` opens
  the Marley guide.
- **Teaches:** the Marley layout as the default (#460) and the two defaults it sets,
  `terminal.button: false` and `agent.dock: "right"` (#438); New Agent… as the one way to start an
  agent (#744); the guide in the title bar (#599).

### Lesson 3. Open your first project

- **Goal:** a project in the rail with a terminal ready.
- **Do:** Add Project, then Open Local Folders, then choose `marley-tutorial`. If Zed asks whether
  to trust the folder, trust it.
- **See:** the project in the rail with one terminal under it, open at the folder's root with the
  focus, and the prompt on the terminal's bottom row.
- **Teaches:** the Add Project popover, the first terminal of a fresh folder (#455), and Zed's
  Restricted Mode, which asks about a folder it has not seen ("Unrecognized Project").

### Lesson 4. Two layouts, one switch

- **Goal:** switch to Zed's layout and back, and know what changes.
- **Do:** run `marley: use zed layout` from the command palette (Ctrl+Shift+P). Press Ctrl+\`.
  Run `marley: use marley layout`, then choose Classic from the title bar's Panel Layout menu.
  Open your settings file with Ctrl+Alt+, and find the `marley` block.
- **See:** Zed's Threads Sidebar replaces the rail; Ctrl+\` opens Zed's Terminal Panel at the
  bottom; the way back restores the rail and each dock's panel. Classic changes nothing and shows a
  toast saying the presets belong to Zed's layout, with a Use Zed's Layout button. The settings
  file holds `"marley": { "layout": "marley" }`.
- **Teaches:** `marley.layout` and its two commands (#438), the docks across a round trip (#456),
  the toast Zed's Panel Layout presets show in the Marley layout (#451), and the layout switch as
  a quick test of whether a problem is Marley's or Zed's.

### Lesson 5. Quit, come back, and keep to one Marley

- **Goal:** know what survives a restart and why only one Marley runs at a time.
- **Do:** quit with `zed: quit` from the palette. Start Marley again from the menu. List
  `~/.local/share/marley/logs/`.
- **See:** the project and its terminal come back, as a shell in its old directory without its
  scrollback. The logs folder holds `Marley.log` and, for a Marley the menu started,
  `stderr.log`.
- **Then:** with Marley running, start it from the menu again, and in a terminal run
  `marley ~/marley-tutorial/README.md`.
- **See, then:** no second Marley. The menu launch brings the running window forward; the
  terminal's line says Marley is already running and was handed one path, and the file opens in
  the running Marley.
- **Teaches:** session restore, Ctrl+Q going to the shell when a terminal has the focus, the
  launcher's stderr log (#502), and one Marley per data directory, a second launch handing its
  paths to the first through the data directory's `marley-dev.sock` (#513, #760; lesson 36 runs a
  second one on purpose).

## Part 2. The rail

### Lesson 6. Terminals as rows

- **Goal:** make, name and close terminals from the rail.
- **Do:** the project's `+`, then New Terminal, twice. In one, `cd` into the subfolder.
  Double-click a row and rename it `server`. Right-click another row and choose Close. Point at a
  row to find its close button.
- **See:** each terminal as a card with a `>_` icon, its title, and a second line with its
  directory relative to the project (empty at the root). The new name shows in the tab and the
  row, and survives a restart.
- **Teaches:** terminal rows (#438, #468), rename and close (#452), rows that follow a terminal's
  directory.

### Lesson 7. A second project

- **Goal:** work with several projects in one window.
- **Do:** Add Project with a second folder. Fold the first project with its chevron. Right-click
  the second project's header and choose Move Project Up. Click rows in both projects.
- **See:** a line between the projects, the window showing the project whose row you clicked, and
  one highlighted row at a time.
- **Teaches:** several projects per window, folding, reorder (#453), the single selection.

### Lesson 8. Dots that ask for you

- **Goal:** read the rail's attention dots.
- **Do:** in a terminal of the first project run `sleep 3; printf '\a'`, and click a terminal of
  the second project at once. Fold the first project and do it again.
- **See:** a dot on the first terminal's row; with its project folded, a dot on the project's
  header. Clicking the row clears it.
- **Teaches:** bell dots on rows and on folded headers (#438).

### Lesson 9. The rail from the keyboard

- **Goal:** drive the rail without the mouse, and hide it.
- **Do:** Ctrl+Alt+; to focus the rail. Up and Down, Home and End. Left on a terminal row, Left
  again, Right. Enter. Then Ctrl+Alt+J to close the rail, restart Marley, and Ctrl+Alt+J to open
  it.
- **See:** Left climbs from a row to its project and then folds it, Right unfolds, Enter opens the
  row as a click does. The rail stays closed across the restart and keeps its width.
- **Teaches:** the rail's list keys (#453), the rail's memory of being closed and its width (#442).

### Lesson 10. Find and switch

- **Goal:** get to any terminal or thread fast.
- **Do:** in the rail, Ctrl+F and type part of `server`; Down, then Enter. Press Escape twice.
  Then hold Ctrl, press Tab twice, and let go. Then run `multi workspace: next thread` from the
  palette a few times.
- **See:** matching characters highlighted and "No matches" for text that matches nothing; the
  switcher listing terminals and threads, the one you used last first; letting go of Ctrl opens
  the selection; next thread walks the rows and wraps at the end.
- **Teaches:** the filter (#457), the switcher (#454), Next and Previous Project and Thread (#459).

## Part 3. The block terminal

### Lesson 11. Is your shell integrated?

- **Goal:** confirm that your terminals will make blocks.
- **Do:** `echo $SHELL $MARLEY_SHELL_INTEGRATION`, then
  `ls -A ~/.local/share/marley/shell_integration ~/.local/share/marley/shell_integration/zsh`.
- **See:** `1` after a bash, zsh or fish path, and the scripts `marley.bash`, `zsh/.zshenv` and a
  `fish` folder. With a `terminal.shell` that has arguments of its own the variable is empty, and
  that terminal makes no blocks.
- **Teaches:** shell integration for bash (#463), zsh (#465) and fish (#466), your own startup
  files still read, and what runs without the integration (remote terminals, a shell with its own
  arguments).

### Lesson 12. Read the blocks

- **Goal:** read a command's outcome at a glance.
- **Do:** run `echo hi`, then `false`, then `sleep 10`, then `vim` and quit it with `:q`.
- **See:** a green bar and a check for `echo hi`; a red bar, the pill `1` and a faint red tint for
  `false`; a blue bar, `running` and a faint blue tint for `sleep 10` until it turns green; nothing
  drawn while vim holds the screen. Each command's prompt rows show Marley's header in place of your
  prompt: the command with its pill (lesson 45 says more).
- **Teaches:** blocks drawn in the terminal (#470), with native headers on by default (#628, #630).

### Lesson 13. Move between blocks, copy and rerun

- **Goal:** use blocks as units.
- **Do:** run `seq 1 200` and a few short commands. Press Ctrl+Up several times, then Ctrl+Down.
  Point at the `seq` block and click Copy Output, then paste into an editor. Point at `echo hi` and
  click Rerun Command.
- **See:** each Ctrl+Up lands at a block's start; Ctrl+Down from the last block returns to the live
  screen. Rerun shows only while the shell waits at its prompt, clears what you typed, and runs the
  command again.
- **Teaches:** block keys (#473), Copy and Rerun (#474), and why only commands the shell reported
  can be rerun.

### Lesson 14. The prompt at the bottom

- **Goal:** see where Marley puts the prompt.
- **Do:** run `clear`, then open a new terminal.
- **See:** the prompt on the last row; each command's output pushes the rest up; scrolling back
  shows the history. After a restart the first terminals open at the size they had, so a wide
  two-line prompt draws right from the start.
- **Teaches:** the content drawn against the bottom edge (#476), and a launch's first terminals at
  the last session's size (#486).

### Lesson 15. Autosuggestions

- **Goal:** let history finish your commands.
- **Do:** run `git status`. Type `git s` and press →. Then type something no command starts with
  and press →.
- **See:** the rest of `git status` dimmed after the cursor, typed when you press →; with no
  suggestion, → moves the cursor. Commands from your shell's history file are suggested too. The
  same ghost text shows in the prompt editor that docks at the prompt (lesson 44).
- **Teaches:** autosuggestions from the terminal's own commands and `$HISTFILE` (#484), and in the
  prompt editor (#637).

### Lesson 16. Terminals in the center

- **Goal:** move between code and terminals without the bottom panel.
- **Do:** open a file in the editor. Press Ctrl+\`, then Ctrl+\` again. Press Ctrl+~. Right-click a
  folder in the project panel and choose Open in Terminal. Run a task from the palette
  (`task: spawn`).
- **See:** Ctrl+\` goes to the terminal you used last and back to the editor; Ctrl+~ opens a new
  center terminal; Open in Terminal opens one in that folder; the task runs in a center terminal,
  its output a block with the bar and the pill. Nothing opens the bottom panel.
- **Teaches:** terminal routing (#441), the terminal keys (#449), and a task's run as a block
  (#621; lesson 45 says more).

### Lesson 17. A desktop notification from a terminal

- **Goal:** get told when a long command finishes.
- **Do:** run `sleep 5; printf '\e]777;notify;Build;finished\a'` and click another pane before five
  seconds pass. Then run `sleep 5; printf '\e]9;Tests passed\a'` and stay in that terminal.
- **See:** a desktop notification titled "Build" that reads "finished"; clicking it brings the
  terminal to the front, and its row and tab carry the bell's mark. The second command shows no
  notification, since you were looking at that terminal.
- **Teaches:** OSC 9 and OSC 777 notifications (#478).

## Part 4. Agents in terminals

### Lesson 18. Start Claude Code from the rail

- **Goal:** an agent in a terminal that the rail understands.
- **Do:** the project's `+`, then New Agent…, then Claude Code (marked Terminal). Press Enter on
  the first Where row, the project marked "this project". Ask it a question.
- **See:** a new center terminal at the project root with `claude` typed into it; the row with
  Claude Code's icon, the title Claude Code sets, and "Claude Code · working", then "waiting" after
  two quiet seconds; the agent bar under the terminal with the folder and its branch.
- **Teaches:** agent CLIs recognized in any terminal (#440), the agent bar (#477), and the New
  Agent picker's Where step (#735, #744).

### Lesson 19. Connect Claude Code to Marley

- **Goal:** notifications from Claude Code, and Marley's tools in it.
- **Do:** click "Connect Claude Code to Marley" in the agent bar. In the session, run
  `/reload-plugins`. Ask Claude Code to create a file, and click another pane while it waits for
  your permission.
- **See:** "Installing Marley's plugin for Claude Code…", then a toast saying it is installed; the
  chip goes away. A notification titled "Claude Code" reads "`<project>` needs your permission",
  and a later one "`<project>` finished". The plugin sits in `~/.local/share/marley/claude-code/`.
- **Teaches:** what the chip installs (#482, #500), the plugin's hooks, and its MCP bridge (#491).

### Lesson 20. Write to the agent

- **Goal:** use the agent bar's input tools.
- **Do:** press Ctrl+G, write two lines with Shift+Enter between them, and press Enter. Press
  Ctrl+G, type a draft, press Escape, and press Ctrl+G again. Click the bar's `+`, choose two files,
  and ask the agent to compare them. With Voxtype installed, turn on Voice in the Marley settings,
  then click the microphone, speak, and click it again.
- **See:** the editor opens above the bar and the prompt arrives as one paste; the draft is still
  there after Escape; the two files' full paths typed into the prompt, quoted where needed; the
  microphone red while recording, yellow while transcribing, and your words typed into the prompt.
- **Teaches:** rich input (#481), Attach File (#479), voice through Voxtype (#480).

### Lesson 21. The agent reads your terminals

- **Goal:** have the agent answer from your terminals instead of pasted output.
- **Do:** in another terminal run `ls /nonexistent`. Ask Claude Code: "Which command failed in my
  other terminal, and what did it print?"
- **See:** Claude Code asks to use Marley's terminal tools (`terminal_list`, `terminal_blocks`,
  `terminal_read`), then names the command, its exit code and its error message.
- **Then:** run `export DEMO_TOKEN=not-a-real-one` and ask Claude Code what the last command in
  that terminal was.
- **See, then:** it answers `export DEMO_TOKEN=[redacted: secret]`: the terminal still shows the
  value, and the agent never received it.
- **Teaches:** Marley's MCP server and its terminal family (#491), verified commands, and secrets
  hidden from what agents read (#516).

### Lesson 22. Other agents, and the New Agent picker

- **Goal:** start any agent from one key.
- **Do:** press Ctrl+Alt+N, type part of an installed CLI's name (for example `cod` for Codex), and
  press Enter, then Enter again on the project.
- **See:** the picker lists Zed's agents marked Thread and the installed CLIs marked Terminal, then
  asks where: the project first, the guess, open and recent projects, and Browse…. The CLI starts
  in a new terminal with its own row and agent bar. The plugin chip shows only for Claude Code.
- **Teaches:** the New Agent picker (#450, #735) and the four CLIs Marley knows.

## Part 5. Zed's agents

### Lesson 23. A Zed agent thread in the rail

- **Goal:** follow Zed's agent threads from the rail.
- **Do:** the project's `+`, then New Agent…, then an agent marked Thread (the Zed Agent, or an
  external agent such as Claude Agent), and Enter on the project. Send a prompt, and click a
  terminal while the agent works.
- **See:** the thread opens in a tab of the project. A thread row under the project's terminals
  reads "Zed Agent · working", later "idle"; its dot lights when the run ends while the thread is
  off screen; a click on the row brings the thread's tab forward.
- **Teaches:** thread rows and their status (#439, #468), and threads in center tabs (#734, #772).

### Lesson 24. Zed's agents use Marley's tools

- **Goal:** see that the Agent Panel's agents reach Marley's server too.
- **Do:** in the thread, ask: "List my Marley terminals and tell me which command failed." Then set
  `"context_servers": { "marley": { "enabled": false } }`, ask again, and take the setting out.
- **See:** the agent calls the `marley` server's terminal tools and answers; with the setting, the
  tools are not offered.
- **Teaches:** the context server `marley` and its bridge (#501), the Zed Agent's Write profile.

## Part 6. The Browser tab

### Lesson 25. Open a Browser tab

- **Goal:** a web page inside Marley.
- **Do:** in a terminal in `marley-tutorial/site`, run `python3 -m http.server 8000`. Then the
  project's `+`, New Browser Tab, type `localhost:8000` and press Enter. Press Ctrl+L, type
  `zed editor`, and press Enter.
- **See:** "Starting Chromium…" the first time, then the page at the tab's size, its title on the
  tab and its URL in the tooltip. `localhost:8000` loads over http; plain words go to a DuckDuckGo
  search.
- **Teaches:** the Browser tab (#488, #500), the address bar's rules (#490), Marley's Chromium as
  a user unit. Lesson 66 opens the same page from the server's terminal instead (#503).

### Lesson 26. Browse as in a browser

- **Goal:** everything a browser does, in the tab.
- **Do:** click the page's button; Alt+Left and Alt+Right; Ctrl+R; the `target="_blank"` link;
  Ctrl+T; the `<select>`, first with a click, then with Tab and Alt+Down; the alert button; select
  some text, Ctrl+C, and paste it into the editor.
- **See:** the link's page in a new tab beside its opener, with the focus; Marley's list under the
  select, with its groups and the disabled option greyed; the alert as a card naming
  `localhost:8000`, answered by Enter; the copied text on the system clipboard.
- **Teaches:** input (#489), navigation and dialogs (#490), a tab per page (#493), select lists
  (#495).

### Lesson 27. Tabs that come back

- **Goal:** know what a restart keeps.
- **Do:** scroll a page and type into its text field; quit Marley and start it again. Then quit
  Marley, stop Chromium (`systemctl --user list-units 'marley-browser-*'`, then
  `systemctl --user stop <unit>`), and start Marley again.
- **See:** the first time, each tab back on its page with your scroll and your text. The second
  time, each tab opens its saved address fresh.
- **Teaches:** saved Browser tabs (#494), and a Chromium that outlives Marley until logout.

### Lesson 28. Let the agent drive

- **Goal:** watch an agent use the page you see.
- **Do:** in Claude Code (connected in lesson 19), ask: "Open localhost:8000 in the browser, click
  the button, and tell me what the console says." Keep the focus in the terminal while it works.
- **See:** Claude Code asks to use the browser tools. The page opens without taking your focus,
  behind your current tab or in a split beside the terminal when no Browser tab is open. The Agent
  chip names each action, and the answer quotes the console line.
- **Teaches:** the browser tools (#492), agents' tabs and the `tab` argument (#493), the
  `browser.write` grant and your approval of each call.

### Lesson 29. Pick an element for the agent

- **Goal:** point the agent at the exact element.
- **Do:** click in Claude Code's terminal first, so it is the terminal you used last. In the
  Browser tab, press Ctrl+Shift+C, move the pointer over the page, and click the button. Type
  "Make this green" in the caption and press Enter. In the terminal, press Enter.
- **See:** Chromium's highlight and tooltip follow the pointer, and the click does not reach the
  page. "Pick 1" waits in the tray; Enter types
  `[browser pick 1: button “…” on localhost:8000; browser_pick id 1] Make this green` into the
  terminal you used last and takes you there. The agent reads the pick with `browser_pick`.
- **Teaches:** pick mode, the tray and its line (#496). Lesson 68 checks the agent's fix against
  the pick (#505).

### Lesson 30. From a pick to its source

- **Goal:** jump from an element to the line that handles it.
- **Do:** open the TypeScript page from the practice setup, and pick an element that has a click
  listener.
- **See:** the tray reads `click src/<file>.ts:<line>`; a click on it opens that file at that line
  in the editor; the tooltip lists every listener. On the plain `index.html` page the script's name
  and line show muted and open nothing.
- **Teaches:** listener sources through source maps (#497).

### Lesson 31. Annotate the page

- **Goal:** mark up the page for yourself and the agent.
- **Do:** click the pencil, drag a box around the heading, type "too small" and press Enter. Ask
  the agent to draw a box around the button with the note "this one", then to list the annotations.
  Click your note and press Delete. Load another page.
- **See:** your box in the theme's warning color, the agent's in its accent color with a sparkle;
  both stay on the content as you scroll; Delete removes yours; the other page starts clean.
- **Teaches:** annotations (#498), `browser_annotate` and `browser_annotations`.

### Lesson 32. Record this

- **Goal:** hand the agent the minute before a bug.
- **Do:** on the page, click around, type in the field, and press the fetch button. Click the red
  dot. Tell the agent: "It broke just now. Read the latest recording and tell me what I did."
- **See:** a toast naming the recording; its files in
  `~/.local/share/marley/browser/recordings/<id>/`; the agent's account of your clicks, key names,
  what you typed into the field (a password field keeps only that it was filled), console lines
  and requests.
- **Teaches:** the flight recorder and Record this (#499), and the text of ordinary fields kept
  for a test (#506). Lesson 68 drafts a Playwright test from the recording.

### Lesson 33. Debug with the console and the network

- **Goal:** let the agent read what the page logged and requested.
- **Do:** press the fetch button. Add `?token=abc` to the page's address, load it, and press the
  button again. Ask the agent what failed on the page.
- **See:** the agent reports the failed request with its status from the network list, which holds
  no headers or bodies, and the console line the page logged. The page's own request shows its
  `token` value hidden.
- **Teaches:** `browser_console` and `browser_network` (#492), redaction of secret-looking values.

## Part 7. Living with Marley

### Lesson 34. Your settings and your data

- **Goal:** know where Marley's settings and data live.
- **Do:** run `marley: open settings` and change the layout from its dropdown, then change it back.
  Open `settings.json` with Ctrl+Alt+, and find `marley.layout`. List `~/.config/marley` and
  `~/.local/share/marley`.
- **See:** the Settings window on its Marley page, first in the list, with a Layout section, an
  Agents section whose Redact Secrets for Agents is on, and a Privacy section whose telemetry
  toggles are off; the dropdown switches the windows as the commands do. The folders hold the paths in the guide's "Where data lives" table, with
  `mcp-endpoint.json` present, readable by you only, while Marley runs.
- **Teaches:** the Marley settings page (#515), the data layout, telemetry off by default (#514),
  redaction's toggle and `marley.redaction_patterns` (#516).

### Lesson 35. When something goes wrong

- **Goal:** find the evidence for each kind of failure.
- **Do:** read the end of `Marley.log`; read `stderr.log`; start the debug build from the checkout
  with stderr sent to a file; run Marley with `ZED_LOG=marley_browser=debug`; read Chromium's unit
  with `journalctl --user -u <unit>`.
- **See:** where panics go (stderr only, on the `dev` channel), where the browser's timing lines go,
  and where Chromium says why it stopped.
- **Teaches:** the guide's troubleshooting list.

### Lesson 36. A second Marley, safely

- **Goal:** try something without touching your main profile.
- **Do:** while your Marley runs, start `marley --user-data-dir ~/marley-scratch ~/marley-tutorial`.
  Quit it when done, and remove `~/marley-scratch`.
- **See:** a Marley that behaves as a fresh install, with its own settings, logs and Chromium,
  beside your own. Its tools reach Claude Code only with `MARLEY_MCP_ENDPOINT` pointing into
  `~/marley-scratch`.
- **Teaches:** one Marley per data directory: a plain second `marley` hands its paths to the first
  and exits (#513), so a second Marley needs a data directory of its own.

### Lesson 37. Update Marley

- **Goal:** keep the installed Marley current.
- **Do:** run `auto update: check` from the palette (Check for Updates). Then `git pull`,
  `just install`, quit Marley and start it again.
- **See:** Check for Updates says "Marley updates through its install script", with what to run,
  and asks zed.dev for nothing. The new commit shows in the installer's output, and the running
  Marley stays on its old build until the restart.
- **Teaches:** the update path (#502) and Check for Updates (#763).

## Part 8. More of the rail

### Lesson 38. Ports, changes and the pull request

- **Goal:** read a project's servers and its branch from the rail.
- **Do:** in a terminal of `marley-tutorial`, run `npm run dev`. Point at the `:8000` row, click it
  once, then double-click it. Open `README.md`, add a line, save, and point at the counts at the
  right of the project's header. Put the file back with `git restore README.md`.
- **See:** within about three seconds a row under the project: a server icon, `:8000` with the
  process's name, and a URL on `127.0.0.1`, since the server listens on `0.0.0.0`. Its tooltip
  gives the command line, folder and pid, with Open, Copy and Stop. One click marks the row and
  opens nothing; a double-click opens the URL in a Browser tab. After the save the header reads
  `+1`, its tooltip naming the base (`main`); the counts go after the restore. With `gh` signed in,
  a branch with a pull request on GitHub shows its icon, number and checks beside the counts.
- **Teaches:** port rows and their menu (#521, #604, #615, #618), services and container ports
  (#603, #614), the lines changed and the pull request (#531), its checks and Send Failed Checks
  to Agent… (#718).

### Lesson 39. Groups, Default and the Home page

- **Goal:** keep work that belongs to no project in the rail.
- **Do:** right-click the rail's empty space under the last row, choose New Group…, and name it
  `Scratch`. Its `+`, New Terminal, then `pwd`. Click Default's header, then the Home button at the
  right of the status bar. Right-click Scratch's header for Rename Group… and Remove Group.
- **See:** Scratch after your projects with a group icon, a chevron and a `+` that has no worktree
  or Launch entry; its terminal's `pwd` prints your home folder. Default heads the rail, and its
  first tab is the Home page, never Zed's Welcome page: Start, New Agent, Recent Projects, Agents
  at Work, Rusty's cards while Rusty is on, Agent Activity and Configure. Home's row is the first
  under Default, above its terminals. The Home button opens the same page as a tab of the group
  you are in. A group comes back after a restart with its name, terminals and Browser tabs.
- **Teaches:** groups with no folder (#600, #601), the Default group (#700, #743), the Home page
  (#701, #782) and the status bar's Home button (#739).

### Lesson 40. Arrange the rail, and what a restart keeps

- **Goal:** put the rail in the order you want, and find what a restart left closed.
- **Do:** drag a group's header onto a project's header and let go. Drag a terminal row onto
  another of the same project. Drag a terminal that runs `sleep 300` onto `marley-tutorial-b`'s
  header, then bring it back with its right-click menu's Move to Project. Quit with `zed: quit`
  and start Marley again.
- **See:** a card with the name follows the pointer and a line shows where it lands; nothing else
  moves. The moved terminal keeps its scrollback with `sleep` still running. After the restart the
  order is as you left it, and the project the window was not showing is listed dimmed: "Not open.
  Click to open it.", with its threads and ports under it.
- **Teaches:** drag to reorder (#602), moving a terminal to another project (#613), dimmed
  projects after a restart (#606) and what they still list (#617).

### Lesson 41. Panels as tabs, and `marley` commands

- **Goal:** open Zed's side panels where you work, and open things from a shell.
- **Do:** press Ctrl+Shift+E, then Ctrl+Shift+G, then Ctrl+Shift+E again. In a Marley terminal of
  the project, run `marley help`, `marley terminal ..`, `marley panel git` and
  `marley agent-cli --provider=nope; echo "exit $?"`.
- **See:** the project panel and the git panel open as tabs in the pane you are in, not in a dock;
  asking again brings the same tab forward. The Debug button sits at the left end of the status
  bar. `marley help` lists the commands; `terminal ..` opens a terminal in the parent folder;
  `panel git` opens the Git tab, or brings it forward; the wrong provider prints the error and
  `exit 2`, and nothing opens.
- **Teaches:** side panels as tabs (#747, #748), Debug in the bottom dock (#774), panel tabs after a
  restart (#773), and `marley <command>` (#749, #775).

## Part 9. More of the block terminal

### Lesson 42. Select a block and work with it

- **Goal:** treat a block as one thing to copy, reuse, read and search.
- **Do:** run `seq 1 200`, `echo a` and `false`. Press Ctrl+Up, then Up and Down. Right-click the
  `false` block and choose Copy as Markdown. Select `echo a` and press Ctrl+Shift+I. Scroll into
  the middle of `seq`'s output. Select the `seq` block and press Alt+Shift+F, type `7`; then
  Ctrl+Shift+B, a few more commands, Alt+Up; then Ctrl+Shift+F and `15`.
- **See:** an outline on the selected block; the menu's Copy Command, Copy Output, Copy Both,
  Copy as Markdown, Reinput and Reinput with sudo; Markdown with the command, output, exit code,
  time, folder and branch; `echo a` typed at the prompt, not run; `seq 1 200` pinned over the top
  row while you scroll its output; a panel over the terminal with only the matching lines; a
  bookmark mark and a tick at the right edge, Alt+Up scrolling back to it; a search held to that
  block. The first time you use a key Marley takes from terminal programs, a toast names it.
- **Teaches:** block selection and the block menu (#554), the sticky command header (#529), the
  output filter (#528), bookmarks and find in a block (#559), the note on taken keys (#563).

### Lesson 43. Workflows and runbooks

- **Goal:** keep a command you will run again, and run a runbook's commands without retyping.
- **Do:** with the dev server up, run `curl -s http://localhost:8000/ | head -3`. Point at its block
  and click Save as Workflow (the book), name it `fetch page`, and save. Press Alt+Shift+T and pick
  `fetch page`. Then click in a terminal, open `runbook.md`, press Ctrl+Shift+V, and click Insert in
  Terminal on each block.
- **See:** the URL already a `{{...}}` parameter, its value the default; the task added to
  `.zed/tasks.json` beside the ones there; running it asks for each parameter. Insert in Terminal
  puts the command at the prompt of the terminal you used last without running it, two lines as
  one paste.
- **Teaches:** workflows as Zed tasks (#558) and a runbook's Insert in Terminal (#530).

### Lesson 44. The prompt editor

- **Goal:** write commands in a Zed editor at the prompt.
- **Do:** in a new terminal, type `ls s` and press Tab. Type `git s` and press →. Type
  `what is using port 8000` and leave it. Press Ctrl+C. Run `vim`, quit it, and press Escape at the
  next prompt. Then turn off Prompt Editor (the Marley page's Layout section) and press Ctrl+G at a
  prompt.
- **See:** an editor docked at the prompt that takes your keys, with the command in your theme's
  colours; Tab lists paths in the prompt's folder and commands from history and tasks; ghost text
  that → takes; `· ctrl-shift-enter asks the agent` after the line in words; Ctrl+C empties the
  editor. The editor closes while vim runs and comes back empty at the next prompt; Escape gives the
  keys to the shell until then. With the setting off, the editor opens only on Ctrl+G.
- **Teaches:** the prompt editor on Ctrl+G (#624) and at every prompt (#627), completions (#625),
  colours (#626), ghost text in it (#637), and English at the prompt (#557).

### Lesson 45. Headers, paths and the first failure

- **Goal:** read where each command ran, and get from a failure to its line.
- **Do:** run `cd site`, `ls`, `cd ..`. Run `grep -rn button site`, then `cd /tmp`, and Ctrl+click
  the `site/index.html:<line>` the earlier block printed. Back in the project, run
  `printf 'print(1 / 0)\n' > fail.py; python3 fail.py` and click the block's Jump to Failure chip.
  Run the `say hello` task (Alt+Shift+T), point at its block, and click Rerun Task.
- **See:** each command's header with its pill, and its folder and branch muted; a two-line prompt
  puts them above the command. The Ctrl+click opens the file at that line, against the folder the
  `grep` ran in. The chip scrolls the block to its first error and opens `fail.py` at the line. The
  task's output is a block with its pill and the block's menu; Rerun Task runs it again in its tab.
- **Teaches:** native block headers (#628, #629, #630) and their density (#631), block-scoped path
  links (#619), Jump to First Failure (#620), tasks as blocks (#621, #622), and a failed block's
  compiler errors as the project's diagnostics (#623).

### Lesson 46. What a resize and a restart keep

- **Goal:** know that a terminal's blocks and output outlast a resize and a quit.
- **Do:** in a terminal with several blocks, press Ctrl+Shift+5, then drag the pane narrow and wide.
  Quit Marley and start it again.
- **See:** the bars and pills stay on their rows as long lines rewrap. After the restart the
  terminal shows its earlier output dimmed above the new prompt, the old commands as blocks whose
  headers say `restored`, with no Rerun. Restore Scrollback (the Marley page's Terminal section)
  turns this off and deletes the saved files.
- **Teaches:** blocks across a resize (#544), earlier output after a restart (#751) and its blocks
  (#764).

### Lesson 47. Blocks on other machines

- **Goal:** optional, with a host you reach by `ssh` with a key: keep blocks over ssh, and a remote
  terminal that survives a dropped link.
- **Do:** run `ssh <host>`, then `echo remote`, `false` and `exit`. Then, with the host in your
  settings' `ssh_connections` and tmux 3.3 or later on it, run `marley: open remote terminal`, pick
  the host, run `sleep 600`, and cut the network for half a minute.
- **See:** each command on the host is a block with its exit code, and nothing is installed there.
  The remote terminal dims within 20 seconds under a line naming the host, what ssh said and the
  next check's countdown; typing is counted, not sent. Once the host answers it reattaches to the
  same tmux session with `sleep` still running.
- **Teaches:** blocks over ssh (#526), remote terminals in tmux (#543) and a dead link noticed and
  reattached (#641).

## Part 10. More of the agents in terminals

### Lesson 48. What the row and the inbox tell you

- **Goal:** follow several agents without watching their terminals.
- **Do:** ask Claude Code to read `site/index.html` and say what each button does. Then ask it to
  create `hello.txt` and click another pane before it asks. Start a Claude Code in
  `marley-tutorial-b` too, give it a task, and fold that project. Set Rail Order to attention on
  the Marley page's Layout section, then back.
- **See:** the row shows your prompt, the tool in flight and the last message. A notification
  reads `marley-tutorial: Claude needs input` over the tool it wants, and later
  `marley-tutorial: Claude finished` over its last message. **Needs you** and a count sit above
  the projects, naming Claude Code, the project, what it asks and how long it has waited; a click
  shows the terminal. The folded header counts its agents (`1 working`). Under the attention order, waiting and failed rows come first,
  and nothing moves while the pointer is over the rail.
- **Teaches:** Claude Code's hook events on its row (#519), notifications that say what happened
  (#538), the inbox (#508), the folded counts and the attention order (#542, #671), and
  `no update in N m` (#547).

### Lesson 49. Hand the agent your work

- **Goal:** give the agent a selection, a block or a running program without copying anything.
- **Do:** open `site/index.html`, select a few lines, and press Ctrl+>. In another terminal run
  `ls /nonexistent` and click Ask the agent on its block. Ask Claude Code to use `terminal_run` to
  run `ls site` in that terminal, then `rm -f hello.txt`. Start `python3` there and ask Claude Code
  to type `6*7` into it and press Enter; then press Ctrl+I in that terminal and click Hand Back.
- **See:** `@site/index.html#L…` at Claude Code's prompt, unsent; the failed block at its prompt as
  Markdown or as a reference it reads with `terminal_read`; `ls site` run at once as a block with an
  agent mark; `rm` waiting on a card with Run and Refuse; a card naming the program and the text,
  with Allow and Deny, then `42`; a bar with Take Over that Ctrl+I takes until Hand Back.
- **Teaches:** send selection (#549), a block to the agent (#555), commands at your prompt and the
  card for risky ones (#556, #553), typing into a running program and taking over (#525, #594,
  #595).

### Lesson 50. Each turn as a diff, and notes back

- **Goal:** review what the agent changed, turn by turn, and answer it with notes.
- **Do:** under Claude Code's row open **Turns (N)** and click a turn. Right-click it for Keep and
  Undo This Turn…, and choose Cancel. Run `git: diff`, choose Add Review on a changed line, type a
  note, and click **Send Review to Agent (1)**. Tick Viewed on a file's header. Then run
  `marley: review last reply` with Claude Code's terminal active, comment a passage with `c`, and
  click Send Notes.
- **See:** each turn that changed files, newest first, with its prompt and file count; Zed's commit
  view with that turn alone; Undo names each file it would restore or remove. The picker lists
  Claude Code `ready`; the note goes as one prompt inside Marley's review message and stays marked
  Sent. A viewed file folds, and opens again when its diff changes. The reply opens in a Review tab
  and the notes go back to Claude Code under "Notes on your last reply:".
- **Teaches:** per-turn diffs (#509) with Keep and Undo (#714), review notes to the agent (#522)
  and their template (#732), Viewed (#731), notes on a file or a reply (#729).

### Lesson 51. Permission modes, close protection and passwords

- **Goal:** choose how much an agent may do alone, and keep from losing one by accident.
- **Do:** set Claude Code Permissions to bypass on the Marley page's Agents section, start Claude
  Code, then set it back to ask. Give an agent a long task and close its tab while it works; choose
  Close, then press Ctrl+Shift+T. In the shell of an agent's terminal, run
  `echo "git_prompt=$GIT_TERMINAL_PROMPT askpass_require=$SSH_ASKPASS_REQUIRE"`.
- **See:** `claude --dangerously-skip-permissions` typed, and a `bypass` chip in the warning colour
  whose tooltip says where Marley read it. The close asks first, naming the agent and its state;
  Ctrl+Shift+T brings the terminal back with the agent still running. The shell prints
  `git_prompt=0 askpass_require=force`; a plain terminal prints them empty.
- **Teaches:** permission modes (#532), close protection and undo (#550), no git credential prompts
  for agents (#537), and ssh's passphrase in a Marley dialog (#596).

### Lesson 52. The shared plugin and plan review

- **Goal:** let Claude Code report its own state, and review its plans in a tab.
- **Do:** on the Marley page's Agents section turn on Shared Claude Code Plugin and Plan Review.
  Open a new terminal, start Claude Code, and in plan mode ask it for a plan to add a footer to
  `site/index.html`. In the Plan tab, select a passage, press `c`, write a note, press Enter, and
  click Send Feedback. On the revised plan, answer any question card, then click Approve.
- **See:** the row reads working, waiting (with the tool) and idle as Claude Code reports them,
  and after a turn the account's usage line, such as `5 h 62 % · resets in 1 h 35 m`. The plan
  opens as Markdown in a **Plan** tab beside the terminal, and Needs you lists "Plan to review".
  The revised plan shows **Version 2** with Changes; an open question is a card with "Question 1
  of 1", its choices, Accept recommended and Skip. Approve lets Claude Code go on. The plugin needs
  Claude Code 2.1.287 or later.
- **Teaches:** an agent's own reports through `MARLEY_BIN` (#652), the shared plugin (#709) at
  rustal-harness's 0.4.0 (#776), Claude Code's usage line and limit (#715, #765), plan review
  (#726) with versions (#727) and question cards (#728).

### Lesson 53. Agents after a restart

- **Goal:** know what comes back when Marley quits while an agent works.
- **Do:** with Marley's plugin connected, give Claude Code a long task, quit Marley while it works,
  and start Marley again. Click Continue on its row.
- **See:** the quit asks first, "Quit Marley?", naming the working agent. After the restart the
  terminal comes back and runs `claude --resume` with the session. Its row shows
  **Interrupted · Continue**; Continue types "Continue where you left off." once Claude Code waits
  at its prompt. Resume Agent Sessions on the Marley page turns resuming off.
- **Teaches:** Claude Code sessions resumed after a quit (#540), and Codex threads on Marley's App
  Server resumed the same way (#750).

### Lesson 54. Codex, Gemini CLI and OpenCode

- **Goal:** optional, with Codex (and Gemini CLI or OpenCode) installed: the same help for the
  other CLIs.
- **Do:** start Codex from New Agent…. Click Turn on Codex notifications in its agent bar. Turn on
  Codex App Server (Marley page, Agents), start a new Codex, and ask it for a change that needs
  your approval. Start OpenCode and click Connect OpenCode to Marley.
- **See:** three keys under `[tui]` in Codex's `config.toml`, the old file kept beside it as
  `config.toml.marley-backup`. On the App Server, Codex's row reads `working · 12k tokens`, then
  `waiting on approval`; Needs you offers Allow, Allow session, Deny and Stop turn; after a prompt
  the row shows a usage line such as `5 h 12 % · resets in 3 h 40 m`. Codex's turns list under its
  row as Claude Code's do.
- **Teaches:** Codex and OpenCode notifications (#552) written with care (#755), Codex's App Server
  (#650), its approvals in Needs you (#651), usage limits and Continue at Reset (#715), and Codex's
  turns (#753).

### Lesson 55. Claude Code's IDE link

- **Goal:** optional, as a check by hand: link Claude Code to Marley as its IDE.
- **Do:** turn on Claude Code IDE Link (Marley page, Agents) and, under Agent Versions, IDE Link,
  IDE Selection and IDE Mentions on Untested Claude Code. Start `claude` in a new terminal of the
  project and type `/ide`. Select lines in a file, then press Ctrl+>.
- **See:** `/ide` names Marley as connected; Claude Code's prompt footer reads the selection; Ctrl+>
  puts a mention into its prompt instead of `@path#La-b`. No Claude Code release is checked on the
  link yet, so the agent bar shows Untested Claude Code until one is.
- **Teaches:** the IDE link's lock file, selection, diagnostics and mentions (#653), and the version
  table behind it (#648).

### Lesson 56. Off your desk: pushes and Remote Control

- **Goal:** optional, with an ntfy server or the Claude app: hear from agents away from Marley.
- **Do:** set `"marley": { "push": { "url": "http://127.0.0.1:<port>", "topic": "marley" } }` and
  leave Claude Code waiting on a permission in a terminal you are not looking at. Then turn on
  Claude Code Remote Control (Marley page, Agents) and start a new Claude Code.
- **See:** one line such as `marley-tutorial: Claude needs input` on the topic, a permission
  reading `needs approval`, with nothing the agent wrote. The new Claude Code starts with
  `--remote-control`, its row shows a `remote` chip, and its session shows in the Claude app.
- **Teaches:** pushes to your phone (#535, #752) and Claude Code Remote Control (#757).

## Part 11. Threads, the Marley agent and Agent Control

### Lesson 57. Find any conversation

- **Goal:** get back to any thread or CLI session, wherever it ran.
- **Do:** point at lesson 23's thread row and click Archive. Right-click the project's header and
  open it from Archived Threads. Run `marley: open threads`, type a word from an earlier prompt, and
  click the Claude Code CLI chip. Click THREADS in the rail's header, then PROJECTS. In the thread's
  tab, right-click and choose New Thread with Context….
- **See:** the archived row goes and comes back when you open it. The Threads page lists every
  conversation, newest first, with its agent, folder and time, your Claude Code and Codex CLI
  sessions among them; clicking a session no terminal runs opens a terminal running
  `claude --resume <id>`. THREADS swaps the rail's rows for the conversations. The new thread opens
  in a split with the old one's history in its message box, unsent.
- **Teaches:** archive and delete (#605, #616), the Threads page (#737), threads in the rail's
  header (#745), your CLI sessions (#754), Reopen Thread… and threads in tabs (#772), threads after
  a restart (#736), New Thread with Context (#717).

### Lesson 58. Ask the Marley agent

- **Goal:** have an agent explain Marley and change its settings with you.
- **Do:** run `marley: talk to marley`. Ask what Sticky Command Header does and what it is set to.
  Ask it to turn it off, and click Apply. Ask it to bind a key to open the Threads page, and click
  Apply. Then run `marley: open marley agent in terminal`.
- **See:** a Marley conversation in a tab, answered from Marley's docs and your settings. Each
  change comes as a notification in every window naming the agent, the key, the value now and the
  value proposed, and the file, with Apply and Decline; Apply writes it with your comments kept,
  and it takes effect at once. The terminal runs the same agent with its own interface. It runs on
  Claude Code, Codex or Zed's agent, whichever Settings → Marley → Marley Agent picks.
- **Teaches:** the Marley agent (#683) on each agent (#687, #696), in a tab (#738) and in a
  terminal (#684), and its docs, settings and keymap tools (#681, #682).

### Lesson 59. Agent Activity and the kill switch

- **Goal:** see what agents did through Marley's tools, and stop them all at once.
- **Do:** ask Claude Code to open `site/index.html` in the editor with `editor_open`, and choose
  Allow for This Session. Run `marley: open agent activity`. Click Stop, ask Claude Code to use
  `terminal_run` again, then click Resume.
- **See:** the question with Allow for This Session, Always for This Project and Deny, then the
  file in front. Agent Activity lists the call: the time, who called, the tool, what it acted on,
  and that it ran. While stopped, every tool that acts refuses and the read tools keep working;
  Settings → Marley → Agent Control shows Stopped, and it holds across a restart.
- **Then:** start a second Claude Code in the project and ask the first to use `draft_message` to
  ask it to list the files in `site`. Edit the draft and click Send.
- **See, then:** a Draft tab naming both agents, the text in an editor, Send and Drop; nothing
  reaches the second Claude Code until Send, which sends your edited text as its prompt.
- **Teaches:** Agent Activity and the kill switch (#703), Agent Control's areas for editors,
  threads and actions (#704, #705, #706, #707, #711), and draft messages an agent puts in front of
  you instead of sending (#769).

### Lesson 60. What your agents see

- **Goal:** know which MCP servers and which versions your agents run with.
- **Do:** run `marley: open mcp servers`. Click Add Server beside Claude Code, fill a name and a
  command, save, then Remove it. Look at Rustal STE Skill in the Marley page's Agents section.
- **See:** the agent CLIs Marley runs with their versions and the parts of Marley that rest on each;
  Claude Code's servers by scope, Codex's, and Zed's context servers with `marley` marked Marley's;
  keys and tokens masked. The save keeps the rest of the file and leaves a `.marley-backup` beside
  it. Rustal STE Skill is on by default; off, new terminals load no `rustal-ste` skill.
- **Teaches:** the MCP Servers tab (#720) and editing a CLI's servers (#768), the agent versions
  Marley checks (#648), and the `rustal-ste` skill (#725).

## Part 12. Launch configs, worktrees and System One

### Lesson 61. Run a launch config

- **Goal:** open a project's whole working set with one choice.
- **Do:** stop the dev server. Click the project's `+`, choose Dev under Launch, read the question
  and click Run. Then change the terminal's title in `.zed/marley.json`, choose Dev again, click
  Cancel, and put the file back.
- **See:** each item on its own line as the file has it; then the dev server's terminal, Claude
  Code split to the right with the focus, and a Browser tab below that waits for port 8000. A
  changed config asks again and says it changed; an unchanged one runs without asking.
- **Teaches:** launch configs (#527) and their approval (#592).

### Lesson 62. An agent in a worktree

- **Goal:** let an agent work on its own branch while you keep working.
- **Do:** with a clean tree, click the project's `+`, New Agent in Worktree, then Claude Code. Tick
  the `Setup: npm install` box and type a task that ends with a commit. Then click the worktree's
  row, press Ctrl+~, and run `ls -a; echo "PORT=$PORT"; npm run dev`.
- **See:** the prompt names `agent/<name> from main`; the worktree as a row under the project, with
  its branch and terminal, while you stay where you were; `npm install && claude …` in its terminal;
  a notification with Trust Folder if Claude Code has not trusted the repository. In the worktree,
  `.env` is there and `PORT=3010`, so its dev server stays clear of 8000. **Also start:** gives the
  same task to other agents, each on its own branch.
- **Teaches:** worktree agents (#510), `.worktreeinclude` and the setup command (#585), Claude
  Code's trust question (#587), a port per worktree (#590), one task to several agents (#770).

### Lesson 63. Drift, review, merge and remove

- **Goal:** bring a worktree agent's branch home and clean up after it.
- **Do:** once the agent has committed, commit something on `main` in the main checkout. Right-click
  the worktree's row: Review, then Merge 1 commit into main…, then Remove….
- **See:** the row reads `1 ahead of main`, then `1 behind`, muted, with main's commit in the
  tooltip (`1 conflict` in the warning colour had the same lines changed). Review opens "Changes
  since main", where Add Review works. Merge checks both checkouts first and names the merge commit;
  nothing is pushed. Remove asks first, runs the teardown task in a terminal of its own, closes the
  workspace and deletes the merged branch.
- **Teaches:** drift from the base (#560), review and merge (#511), remove (#589) and the teardown
  task (#591).

### Lesson 64. System One on local rules

- **Goal:** turn on System One with nothing leaving the machine.
- **Do:** merge the walkthrough's Part 10 block into `settings.json`: `system_one` on, the `rules`
  provider, the tutorial folder in `projects`, every use at `act`, and
  `browser_click_pause_agents: "all_agents"`. Run `false`, then `marley: system one check` and
  `marley: open system one calls`. Ask Claude Code to ask you a question and stop; to run `sleep 1`
  with its Bash tool three separate times; to run `rm -rf build` (deny it); to click Place order on
  the page. Run `echo "error: build failed"; sleep 8; echo "Compiled successfully"; sleep 20` and
  click another pane. Turn it all off afterwards.
- **See:** a toast with the check's reading and `rules` as the provider; System One calls listing
  each call. The idle row reads `asks you`; the inbox entry carries a `destroys` chip and `for you`;
  a looping agent's row reads `looping?`; a consequential click waits on a card with Refuse and
  Allow; a running command's error marks its row with a red × and a notification, and a second
  says it recovered.
- **Teaches:** the System One layer and its check (#565), the stop kind (#566), the find tools
  (#567), the inbox risk (#568), the stall kind (#569), the question route (#570), the click
  consequence (#571), and a running command's printed error (#572).

### Lesson 65. System One with a model

- **Goal:** optional, with a TypeSafe key or a Cloudflare account: send what the rules leave open to
  a model.
- **Do:** read the guide's "What leaves the machine" first. Set `"provider": "typesafe"` and give
  the key with Set Key in System One calls, or `"provider": "cloudflare"` with its token; use
  `provider_by_project` to send only some projects that way.
- **See:** System One calls shows where the key came from, never the key, and each call with what
  was sent (masked), the answer, the tokens and the cost.
- **Teaches:** providers, budget and failures (#565), Jev through Cloudflare Workers AI (#548), the
  typed line's reading at a shell prompt (#573), and keys kept in the keyring under Marley's own
  label (#761).

## Part 13. More of the Browser tab

### Lesson 66. Pages from the terminal, and Browser tabs in the rail

- **Goal:** open the page a server or a program names without typing it.
- **Do:** with the dev server up, look at its terminal's footer and click `localhost:8000`. In
  another terminal run `echo http://localhost:8000/`, then click the URL, Ctrl+click it and
  Shift+Ctrl+click it. Run `python3 -m webbrowser http://localhost:8000/`.
- **See:** the footer offers the server's address, and a click opens it in a Browser tab. A plain
  click on a URL opens a menu (Open in Browser Tab, Open in System Browser, Copy Link); Ctrl+click
  opens it in a Browser tab of the project, or brings forward the tab already on it;
  Shift+Ctrl+click opens your system browser. The `webbrowser` line opens a Browser tab too. Each
  Browser tab is a row under its project, with the page's icon and title.
- **Teaches:** the dev server's address in the footer (#503), Browser tabs as rail rows (#504),
  terminal links (#579), a program's browser request opened in Marley (#561, #586), and the tab's
  title following the page (#582).

### Lesson 67. A browser per project

- **Goal:** keep each project's sign-ins apart.
- **Do:** open a Browser tab in `marley-tutorial-b`. Run
  `systemctl --user list-units 'marley-browser-*'`. Right-click `marley-tutorial-b`'s header and
  choose Clear Browser Data….
- **See:** one `marley-browser-…` unit for each project with a Browser tab, each with its own
  profile; after you confirm, that project's tabs close and a toast says its data is cleared, while
  the other project's tabs stay.
- **Teaches:** a Chromium and a profile per project (#507), and clearing one (#581).

### Lesson 68. Check the fix, and draft a test

- **Goal:** confirm the agent's change on the element you picked, and keep the steps as a test.
- **Do:** after lesson 29's pick and Claude Code's change, press Ctrl+R in the Browser tab and click
  Check on Pick 1. Then, after lesson 32's recording, ask Claude Code: "Draft a Playwright test from
  that recording."
- **See:** a card with the crop at the pick beside the crop now, and what changed; the pick's row
  keeps the verdict, such as `1 change`. Claude Code reads the fuller pick (locators, HTML, styles
  and nearby texts) with `browser_pick`. `browser_draft_test` answers a Playwright test with a path
  under `tests/` and writes nothing itself.
- **Teaches:** the fuller pick (#518), Check (#505), and a test drafted from a recording (#506).

### Lesson 69. Playwright scripts and outside programs

- **Goal:** run your own scripts against the tab's page, and let another program in by name.
- **Do:** click the Playwright Scripts button in the toolbar, type `hello`, press Enter, then click
  Run on the script. Run `marley: browser clients`, type a name such as `playwright-mcp`, choose
  read or act, and press Enter.
- **See:** a script from Marley's template in the editor, kept in Marley's config folder and never
  in the repository; Run runs it on the tab's page in a terminal beside it, the first run installing
  `playwright-core` into Marley's data folder; a failed script saves the tab's last minute as a
  recording. The client gets a token of its own in a file only you can read; a tab it acts in shows
  `Driven by playwright-mcp` with Cut Off. The list also offers an SSH line for a client on another
  machine, with no token in it.
- **Teaches:** Playwright scripts (#523), Browser Clients (#524) and clients on other machines
  (#584), and the relay a DevTools Protocol tool reaches with the token in the project's
  `relay.json` (#583).

### Lesson 70. Take Over, device sizes, files and downloads

- **Goal:** take the page back from an agent, and see it as a phone would.
- **Do:** ask Claude Code to type into the page's field, then click Take Over (or press Ctrl+I in
  the Browser tab), try the page, and click Hand Back. Ask Claude Code to lay the page out as a
  phone, then click Reset. On a page with a Choose File button, click it.
- **See:** after the agent acts, a bar names it with Take Over; then "You have control", and the
  agent's browser actions are refused until Hand Back. A chip names the device, the size and the
  scheme, the page drawn at that size; Reset puts it back. Choose File opens Zed's path prompt. A
  page's download lands in your Downloads folder with a notification that offers Open and Show in
  Folder.
- **Teaches:** Take Over, file choosers and downloads in the Browser tab (#767), and the agent's
  newer browser tools: hover, select, drag, upload, dialogs, waits, devices and the colour scheme
  (#756).

## Part 14. Rusty and the brain

Every lesson in this part is optional: it needs Rusty's `rusty-mcp` installed. The lessons write to
your real vault, inside folders they make and delete; the walkthrough's stops 2.12 to 2.15i name
each step.

### Lesson 71. Turn Rusty on and browse the brain

- **Goal:** connect Marley to Rusty and find your pages from the rail.
- **Do:** with Rusty off, press Ctrl+Alt+V. Turn on Rusty in the Marley page's Rusty section and
  wait for Rusty's Server to say it is connected. Click Brain in the rail's header, make a folder
  `marley-tutorial` and a page `first` in it, open it, and click Today. Click the Rusty button in the
  title bar. Delete the folder from its right-click menu.
- **See:** with Rusty off, a toast says so and where to turn it on, and no `rusty:` command is in
  the palette. Connected, the rail's header shows Projects and Brain; Brain shows Today, a search
  field and your vault's folders with their counts. One click opens a page in a preview tab, two
  keep it. Home's Rusty card has a button per screen and the captures, with your recent pages,
  follow-ups due and open tasks under it; while Rusty is not connected it says why. Delete asks
  first and Rusty moves the folder into `archive/`.
- **Teaches:** Rusty's switch and connection (#643), off leaving no trace (#661), the Brain view
  (#644), Rusty's cards on Home (#679, #782) and its title bar button (#739).

### Lesson 72. Brain pages in tabs

- **Goal:** read, link and edit brain pages without leaving Marley.
- **Do:** make two pages, `links` and `other`, and with Edit give `links` a wikilink to `other`.
  Click Read, follow the link, then Alt+Left and Alt+Right. Press Ctrl+Alt+U and type part of a
  page's name. In `other`, click its title and rename it, add a property, and click the star.
- **See:** the page's title and properties above its body; Edit shows the file with its
  frontmatter, Read saves it and draws the links; Back and Forward move between pages. The picker
  lists every page and offers Create page for a name that has none. The title and the property
  change in place; the starred page shows under Favourites above the tree.
- **Teaches:** a brain page in a tab (#645), the page picker (#654), the outline and edits in place
  (#656), and Favourites (#662).

### Lesson 73. Knowledge, the graph, tasks and decisions

- **Goal:** see how pages connect, and work Rusty's lists and decisions.
- **Do:** run `rusty: toggle knowledge panel` with a page in front. Run `rusty: open graph`, then
  `rusty: open local graph`. Click the list icon after Graph in the Brain view, make a list and
  three tasks, and tick one. Click the double check after Tasks, then Follow Up on a decision
  that is due.
- **See:** the Knowledge panel with the page's tags, backlinks and links, and with no page in
  front, your project's page, its follow-ups and tasks. The Graph tab's pages as dots coloured by
  type, decision edges dashed, a local graph around one page. The Tasks tab in Rusty's order, a
  ticked task struck through. The Decisions tab with the follow-ups due first; Follow Up offers
  Kept, Revised and Superseded.
- **Teaches:** the Knowledge panel (#646) and its project view (#655), the Graph tab (#647) and its
  groups, display and forces (#657), the Tasks tab (#658), the Decisions tab (#659), and recording
  a follow-up (#660).

### Lesson 74. Capture, memory, skills, secrets and Rusty's settings

- **Goal:** put things into Rusty from Marley, and reach the rest of what it keeps.
- **Do:** run `rusty: capture to today` and type a line; `rusty: capture url` with
  `https://example.com`. Open the Memory, Skills and Secrets screens from Home's Rusty card. Open
  Rusty's Server in the Settings window. Run `marley: talk to rusty` and ask it what is on your
  list.
- **See:** the toast names today's note, and the URL becomes a source page with its text. Memory
  lists what Rusty remembers, high first; Skills lists staged skills first, then active ones and
  scripts; Secrets asks for your PIN and hides every value. Rusty's Server shows the embedding
  status and Rusty's ten settings, any secret key as an empty field marked hidden. Rusty answers
  from your lists through Rusty's own tools.
- **Teaches:** capture and import (#663), the Memory tab (#664), the Skills tab (#665), Rusty's
  settings (#666), the Secrets tab (#667), and the Rusty agent (#696) in a tab (#738).

## Part 15. The fleet, the harness and other hosts

Lesson 75 needs only Marley. Lessons 76 to 79 need rustal-harness's `rh` on this machine (on the
PATH, or named by `MARLEY_RH`), and lessons 80 and 81 a second machine you reach by `ssh` with a
key.

### Lesson 75. The Fleet panel

- **Goal:** see agents, their work and their hosts in one place, on example data first.
- **Do:** add `"fleet": { "providers": [ { "kind": "pseudo" } ] }` to the `marley` block and run
  `marley: toggle fleet`. Click build-1, then double-click it. Add `"hosts": [ { "local": true } ]`
  beside `"providers"`.
- **See:** a Fleet tab in the pane you are in, with two hosts and three agents, each with its work
  item, phase and state chip; docs-1 turns `stale` within about 20 seconds. Clicking an agent shows
  its snapshot below the list; a double-click opens its Agent tab with the phases on a timeline,
  events, the host's graphs and tokens. With `local`, a **Hosts** section lists this machine's CPU,
  memory, disk and network, and every Claude Code and Codex running here.
- **Teaches:** the Fleet panel and its pseudo provider (#607), the snapshot (#608), the Agent tab
  (#609), hosts read over SSH or locally (#610), real stores over MCP and HTTP (#611), and the panel
  as a tab (#748).

### Lesson 76. Follow a harness and make a seat

- **Goal:** watch a rustal-harness's sessions in the rail, and start a seat from Marley.
- **Do:** set `"marley": { "embedded_harness": true, "harness_writes": true }`. Run
  `marley: new harness seat`, name it `builder`, choose Claude Code, keep the project's folder, and
  click Create. Click `builder`'s row; type a line under Send; open Views and click Open on
  `rh attach`, then leave it with Ctrl-b d.
- **See:** a Harness section after the projects, its header saying whether Marley is connected or
  why the runtime is not running. A Claude Code seat takes up to a minute to start; then `builder`
  is a row with its title, a dot for its state and its state in a word, and its tab opens under
  Default with the session's last lines. The tab's question buttons, Send and Views come with
  `harness_writes`. A refusal stays in the form as the harness said it; for
  `claude_signin_undeclared`, run `rh claude signin` on that root first.
- **Teaches:** the harness's sessions in the rail (#534), the harness Marley runs itself (#632),
  where the state came from, progress and quota on a row (#640), writing to a session (#689), its
  views (#690), and a seat in one step (#691).

### Lesson 77. Seats from an agent, and the manager

- **Goal:** let an agent propose seats, and talk to the harness's manager from Marley.
- **Do:** ask the Marley agent to add a Codex seat `reviewer` in the project's folder, and click
  Apply. Ask it to stop `reviewer`, then to remove it, and Apply each. Then run
  `marley: new harness seat` with the role `manager`, open New Agent…, and pick **Manager**. Type
  "hello". Show another tab while the manager posts a report.
- **See:** each proposal waits on a notification with the seat and the command it runs through,
  Apply and Decline. `reviewer` shows in the rail once started; stopped, its row reads `done` and
  its profile stays; removed, it leaves the rail and the name is free again. **Manager** is listed
  while the harness has a manager; your message reaches it once it is idle, and its messages and
  reports arrive as replies. Out of sight, a report shows a
  desktop notice, "Manager: Report", and a Needs you entry that opens the Manager thread. Open
  Manager Thread on the manager's row does the same.
- **Teaches:** seats proposed by agents (#692), stopping and removing them (#710), the manager in
  the Agent Panel (#694), its notices (#688), and its thread from the inbox or the row (#758).

### Lesson 78. Scheduled runs

- **Goal:** have the harness run a prompt on a schedule, and hear only when a run finds something.
- **Do:** on the root Marley's harness uses, run `rh --state ~/.local/share/marley/harness schedule
  add checks --profile builder --cwd ~/marley-tutorial --every 1 --prompt "Read README.md. If it
  has fewer than five lines, report a finding that says so."` (an `rh` with `rh schedule`,
  rustal-harness TICKET-118). Click `checks` in the Harness section once it has run. Then
  `rh … schedule pause checks`, and `remove` it.
- **See:** the task after the sessions, with its schedule and next run (`every 1 min · next
  HH:MM · no run yet`, later `every 1 min · paused`) and its last run's outcome and first line. A
  click opens the task's history, newest first; a click on a run opens its session. A run that ends `finding`
  raises one Needs you entry, "checks found something: <first line>", and one desktop notice;
  `nothing` and `failed` raise nothing, and an entry you opened stays gone after a restart.
- **Teaches:** the harness's scheduled runs in the rail (#719). Marley builds no timer: the harness
  keeps the tasks and runs them.

### Lesson 79. An agent that starts another

- **Goal:** let an agent hand part of its work to a new agent, with you allowing each start.
- **Do:** with `harness_writes` on, ask Claude Code in a Marley terminal: "Use agent_start to start
  a Claude Code agent here with the prompt: list the files in site." Allow it. Wait until the new
  agent finishes, with the first one idle at its prompt.
- **See:** a question naming the agent, the folder and the prompt's first line, with Allow for This
  Session, Always for This Project and Deny. The new agent's terminal opens beside the caller, and
  its seat's row in the Harness section says which terminal started it. When its turn ends, Marley
  types one line into the caller: `Agent <seat> ended: <outcome>. Read the result:
  agent_status {"delegation": "<id>"}.`, once, even after a restart.
- **Teaches:** `agent_start` and `agent_status` (#716), Agent Control's Agents setting
  (`marley.agent_control.agents`). It needs a harness that publishes the seat labels and the
  `ended` state (rustal-harness TICKET-117).

### Lesson 80. A harness on another machine

- **Goal:** optional, with a second machine running rustal-harness: follow it, and start agents on
  it.
- **Do:** add `"harnesses": [ { "name": "box-2", "ssh": "me@box-2", "state": "/srv/harness" } ]`
  to the `marley` block, with your own host and root. Open New Agent…, pick Claude Code, choose
  **On box-2…** and type a folder there. Then pick Claude Agent and **On box-2…** again. Right-click
  a seat's row for Open as Thread, and start **Manager · box-2** if that harness has a manager.
- **See:** a second section, HARNESS · box-2, with its own connection line; one harness that stops
  answering marks its rows stale while the other goes on. Claude Code starts as a seat there, with
  a terminal attached here through `ssh -t`. Claude Agent opens a thread tab onto the seat. Each
  session's tab and inbox entry carry the harness's name.
- **Teaches:** one harness per host (#740), New Agent on a remote host (#741), a seat as a thread
  tab (#742), and a manager per harness, its thread from any window (#758). None of these four has
  been tried on a real second machine yet; the guide's "Try it on a real host" list is that check,
  done by hand.

### Lesson 81. Remote projects over SSH

- **Goal:** optional, with a Linux x86_64 host you reach by `ssh` with a key: open a folder on it.
- **Do:** Add Project, then Open Remote Folder, and pick a folder on the host. Then run
  `ls ~/.zed_server` on the host.
- **See:** the project opens and its terminals run on the host. The host holds
  `zed-remote-server-dev-<commit>`, the server lesson 1's install staged, uploaded on the first
  connection. With no server staged for Marley's commit, the connection fails with a message that
  names the platform, the commit and `script/install-marley --remote-server-only`.
- **Teaches:** Marley's own remote server, staged by the install and matched by commit (#445).

## Part 16. Marley's own name

### Lesson 82. Marley's name, links and keyring

- **Goal:** know how Marley names itself to the desktop.
- **Do:** run `hyprctl clients | grep -i 'class: com.ignibyte'` and
  `ls ~/.local/share/marley/*.sock`, then `zed: about` from the palette. Run
  `xdg-open marley://settings`. In a Markdown file, Ctrl-click a `zed://settings` link. Optional,
  with `secret-tool`: in System One calls click Set Key, type `not-a-real-key`, and run
  `secret-tool search --all application marley`; then click Forget Key.
- **See:** each window's class is `com.ignibyte.Marley`, the socket is `marley-dev.sock`, and About
  says Marley. `xdg-open` opens the Settings window in the running Marley, and the `zed://` link
  opens it in Marley too, not in a stock Zed. The keyring item is labelled Marley; Forget Key
  removes it.
- **Teaches:** Marley's app id, name and socket (#760), the `marley://` scheme and links clicked
  inside Marley (#762), and Marley's keyring label (#761).

### Lesson 83. Follow Omarchy's theme

- **Goal:** on Omarchy only: draw Marley in the desktop's theme.
- **Do:** turn on Follow Omarchy's Theme in the Marley page's Layout section. Switch the desktop's
  theme from Omarchy's menu (Style, then Theme). Turn the setting off.
- **See:** the editor, the panels and the terminal's 16 colours take the desktop's theme, and the
  theme selector shows Omarchy. A theme switch changes Marley with no restart, light or dark as the
  theme is. Off, the theme you had comes back, and Marley's hook and theme copy are gone.
- **Teaches:** Follow Omarchy's Theme (#759).

## Lessons that wait on planned features

Every ticket through #778 has shipped, so the lessons above cover the
whole of Marley. Two items of `docs/marley/three-prong-plan.md` have not shipped, and each gets a
lesson when it does:

| Future lesson | Where it goes | Waits on |
|---|---|---|
| Rusty's sessions as rows in the rail | Part 14 | Plan C2: `rusty-mcp` serving them |
| Work objects (brain tickets, runs and gate evidence) as labels on seats, and the workflow's phase in the status bar | Part 15 | Plan C5 |
