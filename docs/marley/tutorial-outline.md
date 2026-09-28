# Marley from zero: the tutorial outline

The outline of a long tutorial that teaches Marley lesson by lesson, in the order a new user
would meet it. Each lesson has a goal, what you do, what you should see, and the feature it
teaches, with the ticket that shipped it so the writer can check the lesson against
`CHANGELOG.md`. The outline follows Marley as of 2026-09-26 (through #516, with #513) on Linux. The guide
(`guide.md`) is the reference each lesson points into.

The order follows what later lessons need. The layout and the rail come first because every
terminal opens there. Blocks come before agents because agents read blocks. Claude Code gets
connected in part 4 because the browser lessons in part 6 have it drive the page.

## What the reader needs

- Marley's checkout, with `cargo` and `just` (lesson 1 builds from it).
- Chromium and a systemd user session, for part 6.
- Claude Code (`claude` on the PATH, signed in), for parts 4 and 6. Codex, Gemini CLI or OpenCode
  are optional extras for lesson 22.
- A Zed agent set up in the Agent Panel (a model for the Zed Agent, or an external agent such as
  Claude Agent), for part 5.
- Voxtype, optional, for the microphone in lesson 20.

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

## Part 1. Install and first launch

### Lesson 1. Install Marley

- **Goal:** an installed Marley in your app menu.
- **Do:** in the checkout, run `just install`, and read the lines it prints at the end.
- **See:** it waits for any other cargo run, builds in the release profile (about ten minutes the
  first time on the dev box), then prints `~/.local/bin/marley at <commit>` and the menu entry's
  path. It adds a note when `~/.local/bin` is not on your PATH, and a warning when a Marley is
  already running.
- **Teaches:** `just install` and `--prefix`, the four installed files, the launcher (#502).

### Lesson 2. Meet the Marley layout

- **Goal:** know the parts of a Marley window.
- **Do:** start Marley from the app menu. Find the rail on the left, its PROJECTS header, the Add
  Project button and the filter field. Press Ctrl+? to bring up the Agent Panel.
- **See:** the rail on the left, the Agent Panel docked on the right, and no Terminal Panel button
  in the status bar.
- **Teaches:** the Marley layout as the default (#460) and the two defaults it sets,
  `terminal.button: false` and `agent.dock: "right"` (#438).

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
  paths to the first (#513; lesson 36 runs a second one on purpose).

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
- **See:** `1` after a bash or zsh path, and the scripts `marley.bash` and `zsh/.zshenv`. With fish,
  or a `terminal.shell` that has arguments of its own, the variable is empty, and those terminals
  make no blocks.
- **Teaches:** shell integration for bash (#463) and zsh (#465), your own startup files still
  read, and what runs without the integration (tasks, remote terminals).
- **Waits on:** fish users need TICKET-466 for this part.

### Lesson 12. Read the blocks

- **Goal:** read a command's outcome at a glance.
- **Do:** run `echo hi`, then `false`, then `sleep 10`, then `vim` and quit it with `:q`.
- **See:** a green bar and a check for `echo hi`; a red bar, the pill `1` and a faint red tint for
  `false`; a blue bar, `running` and a faint blue tint for `sleep 10` until it turns green; nothing
  drawn while vim holds the screen.
- **Teaches:** blocks drawn in the terminal (#470).

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
  shows the history.
- **Teaches:** the content drawn against the bottom edge (#476).
- **Waits on:** with a two-line prompt wider than 100 columns, the first terminal of a launch is
  misdrawn until its next prompt (TICKET-486).

### Lesson 15. Autosuggestions

- **Goal:** let history finish your commands.
- **Do:** run `git status`. Type `git s` and press →. Then type something no command starts with
  and press →.
- **See:** the rest of `git status` dimmed after the cursor, typed when you press →; with no
  suggestion, → moves the cursor. Commands from your shell's history file are suggested too.
- **Teaches:** autosuggestions from the terminal's own commands and `$HISTFILE` (#484).

### Lesson 16. Terminals in the center

- **Goal:** move between code and terminals without the bottom panel.
- **Do:** open a file in the editor. Press Ctrl+\`, then Ctrl+\` again. Press Ctrl+~. Right-click a
  folder in the project panel and choose Open in Terminal. Run a task from the palette
  (`task: spawn`).
- **See:** Ctrl+\` goes to the terminal you used last and back to the editor; Ctrl+~ opens a new
  center terminal; Open in Terminal opens one in that folder; the task runs in a center terminal.
  Nothing opens the bottom panel.
- **Teaches:** terminal routing (#441) and the terminal keys (#449).
- **Waits on:** a task's terminal makes no blocks yet; tasks as blocks are T4 in the plan.

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
- **Do:** the project's `+`, then Claude Code under Agent CLIs. Ask it a question.
- **See:** a new center terminal at the project root with `claude` typed into it; the row with
  Claude Code's icon, the title Claude Code sets, and "Claude Code · working", then "waiting" after
  two quiet seconds; the agent bar under the terminal with the folder and its branch.
- **Teaches:** agent CLIs recognized in any terminal (#440) and the agent bar (#477).

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
  and ask the agent to compare them. With Voxtype installed, click the microphone, speak, and click
  it again.
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
  press Enter.
- **See:** the picker lists Zed's agents marked Thread and the installed CLIs marked Terminal; the
  CLI starts in a new terminal with its own row and agent bar. The plugin chip shows only for
  Claude Code.
- **Teaches:** the New Agent picker (#450) and the four CLIs Marley knows.

## Part 5. Zed's agents

### Lesson 23. A Zed agent thread in the rail

- **Goal:** follow the Agent Panel's threads from the rail.
- **Do:** the project's `+`, then New Agent Thread, then the Zed Agent (or an external agent). Send
  a prompt, and click a terminal while the agent works.
- **See:** a thread row under the project's terminals reading "Zed Agent · working", later "idle";
  its dot lights when the run ends while the thread is off screen; a click opens the thread in the
  Agent Panel on the right.
- **Teaches:** thread rows and their status (#439, #468).

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
  a user unit.
- **Waits on:** clicking the server's URL in the terminal to open it in a Browser tab
  (TICKET-503); today you type it.

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
- **Teaches:** pick mode, the tray and its line (#496).
- **Waits on:** checking the agent's fix against the pick, before and after (TICKET-505).

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
  typing counts (never the text), console lines and requests.
- **Teaches:** the flight recorder and Record this (#499).
- **Waits on:** turning the recording into a Playwright test (TICKET-506).

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
- **Do:** `git pull`, `just install`, then quit Marley and start it again.
- **See:** the new commit in the installer's output, and the running Marley on its old build until
  the restart.
- **Teaches:** the update path (#502).

## Lessons that wait on planned features

Each lesson below joins the tutorial when its ticket ships. The first eleven rows come from
`docs/planning/tickets/BACKLOG.md` (its queue, then one deliberate row), the next from a ticket in
progress, and the last five from `docs/marley/three-prong-plan.md`.

| Future lesson | Where it goes | Waits on |
|---|---|---|
| Click a dev server's URL in the terminal to open it in a Browser tab | Replaces the typing in lesson 25 | TICKET-503 |
| Find a project's Browser tabs as rows in the rail | After lesson 26 | TICKET-504 |
| Check the agent's fix against the pick, before and after | After lesson 29 | TICKET-505 |
| Turn a recording into a Playwright test | After lesson 32 | TICKET-506 |
| Keep a separate login per project | After lesson 27 | TICKET-507 |
| Answer every agent's permission prompt from one list in the rail | After lesson 23 | TICKET-508 |
| Review Claude Code's work turn by turn | After lesson 21 | TICKET-509 (shipped) |
| Run agents side by side, each on its own worktree and branch | A new part after part 5 | TICKET-510 (shipped) |
| Review and merge a worktree agent's branch | In that new part | TICKET-511 |
| A long two-line prompt in the first terminal of a launch | Removes lesson 14's note | TICKET-486 |
| Blocks in fish | Lesson 11 for fish users | TICKET-466 (deliberate: waits for a machine with fish) |
| Print a secret in a terminal and see the agent get it redacted, with the toggle on the Marley page | After lesson 21 | TICKET-516 (in progress) |
| Open a failed block's `file:line` at the block's own directory, and jump to the first failure | Part 3 | Plan T2 |
| Edit the command in a prompt editor, with completions and coloring | Part 3 | Plan T3 and T6 |
| Tasks and runnables as blocks, failures in diagnostics | Lesson 16 | Plan T4 |
| Block headers with the shell's prompt hidden | Part 3 | Plan T5 |
| Harness seats and Rusty's sessions in the rail | A new part | Plan C1 to C5 |

An optional lesson could go in now, on attaching another CDP client (Playwright, say) to Marley's
Chromium through `DevToolsActivePort` and watching the tab show what it does. The code supports
it, but only a stand-in client in the e2e suite has exercised it, so the writer should run it
before it goes in.
