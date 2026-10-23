# Boards G: Dialogs, OpenConnect, SystemPages, ForgeBoard, ForgeAgents

No current shot shows any of these five boards. `tour-01-start.png` opens straight into a terminal, and `tour-18-fleet.png` actually shows the Component Library. Every item below comes from the code and is marked "not seen in a run". Paths are relative to `/srv/stacks/marley_ide`. `mw/` stands for `crates/marley_workbench/src/`.

## Dialogs

- **First-run screen.** The board shows a "Welcome to Marley" card with the cat mark and "Four choices, each one changeable later in Settings". Each choice is a row of option cards: Layout (Marley/Zed), Theme (Night/Day/Follow Omarchy), Agents (Claude Code with its found version, Codex "Not on the PATH", Connect the plugin) and Rusty (On/Off). Below them sit "Open a Folder" (primary) and "Go to Home".
  - Marley still opens Zed's own onboarding view on first open: `crates/zed/src/main.rs:1606-1607` (`FIRST_OPEN` → `show_onboarding_view`). Open TICKET-794 line 7 shows it restoring "the first-run Onboarding tab". Grepping `Welcome to Marley`, `Four choices` and `Go to Home` in `crates/marley_*` finds nothing.
  - **deferred**: #870's spec Out says "The first-run screen … (W1's onboarding, its own ticket when planned)", and #872 says the same. look-plan W1 has no onboarding item, and no ticket is open or closed for it. **L**
- **Close dialog wording.** The board's title is "Claude Code is still working". The body names the turn ("'Fix the rail test' is in a turn") and says "Close keeps it running for 60 s so Undo can bring it back."
  - Marley's title is "Close Claude Code in <project>? It is working." and its body is "<project> · Claude Code · working" (`mw/close_guard.rs:306-314`). The undo hold and its seconds are real (`hold`, `marley.undo_close_seconds`), but the dialog never mentions them, and it never names the turn.
  - **look**. **S**
- **Close dialog "Wait" button.** The board has Cancel / Wait / Close.
  - Marley has Cancel / Show / Close (`mw/close_guard.rs:140`). Nothing waits for the turn to end and then closes: grepping `"Wait"` in `mw/` finds nothing.
  - **missing**. **M**
- **Quit dialog.** The board's title is "Quit with 2 agents working?". The body is one sentence naming each agent and its project, ending "Claude Code resumes at the next start". The buttons are Cancel and Quit only.
  - Marley's title is "Quit Marley? 2 agents are working:" with one "project · agent · status" line per agent (`mw/close_guard.rs:317`). Its buttons are Quit / Show / Cancel, so it has an extra Show. It never says which agents resume, although `mw/resume.rs` does resume them.
  - **look**. **S**
- **Second-launch notice.** The board shows a notice in the running Marley: "Opened repo from another launch". Its body is "Only one Marley runs on a data folder…" with a Dismiss button.
  - The running Marley shows nothing. `crates/zed/src/main.rs:406-414` only prints to the second launch's stdout, and #870's notes say so outright ("the second launch shows nothing in the running Marley"). Grepping "another launch" finds nothing.
  - **missing**. **S**
- **Rusty-not-connected dialog.** The board shows a dialog titled "Rusty is not connected" whose body is the reason. Its buttons are Open Settings (plain) and Dismiss.
  - There is no dialog. Rusty's pages show an empty-state card with "Try Now" only (`mw/rusty.rs:460-469`), and actions on those pages toast "Marley is not connected to Rusty: …" with no Open Settings action (`mw/rusty/tasks_tab.rs:1318`, `decisions_tab.rs:474`, `skills_tab.rs:596`).
  - **look**. **S**
- **Setting-change proposal.** The board shows a centred dialog titled "Marley's agent proposes a change". The body is "key: a → b. Nothing changes until you apply it.", with Refuse and Apply.
  - Marley shows an app notification card instead, titled "<agent> wants to change your settings". It has the change line and the file path as a footnote, and Decline / Apply buttons (`mw/settings_change.rs:134,306-317`). Only the wording and the placement differ.
  - **look**. **S**
- **Trust modal body.** The board's trust card has only the title, one line and Not now / Trust.
  - Marley draws those (`mw/trust_card.rs`), but keeps Zed's restricted-mode list and the "trust all projects in" checkbox below them.
  - **deferred**: #872 chose to keep these. **S**

## OpenConnect

- **Open a remote folder.** The board shows one address field ("ssh://user@host/path") over a list of hosts. Each host has a status dot and its path, and the last row is "Add a host…" pointing at `ssh_connections`. A banner at the foot shows upload progress: "Uploading Marley's server for this build · 18 of 31 MB".
  - Marley opens Zed's own Remote Projects modal and has not touched it: no `Marley:` hooks in `crates/recent_projects` beyond the recent-projects rows (#854). There is no address field taking an `ssh://` URL and no host status dots. Zed's connection status says "Uploading remote development server" with no byte count (`crates/remote/src/transport/*.rs`). #889's scope only checked the remote *terminal* menu (`mw/remote.rs`).
  - **look**. **M**
- **Open a page by name (Ctrl Alt U).** Built: `mw/rusty/page_picker.rs`, bound to `secondary-alt-u` in `crates/marley_workbench/keymap.json:21`.
  - The differences are small. The board puts a ★ at the left of a favourite; Marley puts a file icon at the left and the star at the right end (`page_picker.rs:471-479`). The board puts the slug under the title; Marley puts it beside the title. The board's create row is "+ Create page <slug>" under a divider; Marley's is "Create page: <slug>" with no divider.
  - **look**. **S**
- **New harness seat.** Built: `mw/harness_seat.rs:981-1050`.
  - The board has a standing footnote at the bottom left, "A Claude Code seat can take a minute". Marley shows that only as a status line once Create is pressed (`harness_seat.rs:804`). Marley also adds a headline, an explanation line and a Cancel button that the board does not have.
  - **look**. **S**
- **The two layouts.** The board shows two picture cards side by side, each with a diagram (rail + tabs for Marley; docks with a bottom panel for Zed), a name and a one-line description, the chosen one outlined in the accent colour.
  - Marley has a plain "Layout" setting row (`crates/settings_ui/src/marley_page.rs:59-80`) and the actions `UseMarleyLayout` / `UseZedLayout` (`mw/marley_workbench.rs:1234`). There are no picture cards.
  - **missing**. **M** (it overlaps with the first-run screen's Layout step)
- **About and updates.** The board shows a dialog with the cat mark, "Marley", and "dev · <sha> · built today 12:30". Below that is the line "Marley updates through its install script, never from zed.dev. Pull, then run just install.", and two buttons: "Copy the command" and "Open the log".
  - Marley's About is Zed's window with two small additions: the cat line (#829) and the "Marley" channel name (`crates/zed/src/zed.rs:1533-1700`). It shows Commit and Version but no build time and no update line. Its buttons are OK and Copy, and Copy copies the version details, not the install command. There is no Open the log button.
  - **look/missing**. **M**
- **The `marley` commands panel.** Nothing to do: the verbs exist (`mw/commands.rs:68-71`, `show` in `mw/terminal_show.rs`, `marley://` in #762). The board lists them as a reference, not as a screen.

## SystemPages

The System tab (#855, #885; `mw/system_pages.rs`) has all nine pages. It shows them as a list on the left with one page at a time, where the board shows a 3×3 grid; the tab is a reasonable form of that grid. The gaps below are per page.

- **Install and update: progress steps.** The board shows the last install as steps (toolchain found, build done with its time, remote server staged, desktop entry installed) and a warning row, "Running Marley · still on <sha> · restart".
  - Marley's rows don't change: Running (this commit), Its binary, Install (a sentence about `script/install-marley`) and After an install (`system_pages.rs:265-293`). It shows no step status, and it never notices that a newer build is installed while an old one runs.
  - **missing**. **M**
- **Install and update: Restart Marley button.** The board has "Restart Marley".
  - Marley has only "Open the logs" (`system_pages.rs:770-773`).
  - **deferred**: #885 Out says "Restart Marley … wants its own ticket and its confirmation". **S**
- **Where data lives: Clear terminal history.** The board has a "Clear terminal history" button.
  - Marley has Open the folder and Count again (`system_pages.rs:763-768`).
  - **deferred**: #855 and #885 both name it. **S**
- **This build.** The board's lead adds "A debug build with a data folder of its own says so in the title bar", and its Data folder row carries a "shared"/own value.
  - Marley's Data folder row has an empty value (`system_pages.rs:255-260`), and its lead stops at "on which data folder."
  - **look**. **S**
- **Troubleshooting: rusty-mcp version.** The board's rusty-mcp row shows "embedded · 0.1.1".
  - Marley's row says only connected or why not (`system_pages.rs:315-324`).
  - **deferred**: #885 Out says "Rusty's version: kept private in its state". **S**
- **Shell integration.** The board lists, for each shell, what it gets ("blocks · prompt editor · show"), names the agents recognised, and has an "Open the guide" button.
  - Marley's rows describe how each shell's integration is installed (rcfile, ZDOTDIR, vendor snippet), not which features the shell gets. The page has no buttons (`system_pages.rs:393-425`, `:778`).
  - **look**. **S**
- **Marley's MCP endpoint: rows.** The board's rows are Endpoint (loopback only), Bridge (path · stdio), Tools ("90 · 10 write classes granted"), Last refusal (tool · class · path · when) and Limits ("bodies ≤ 1 MiB · http and https only").
  - Marley shows Server (url · running), Agents' address file and Paired programs (`system_pages.rs:428-460`). It has no bridge, tool count, write classes, last refusal or limits.
  - **missing**. **M**
- **Marley's MCP endpoint: buttons.** The board has "Open Agent Activity" and "Copy the endpoint".
  - The page has no buttons (`system_pages.rs:778`).
  - **missing**. **S**
- **Marley's Chromium.** The board shows a Chromium row (headless shell and version, running), one row per profile with its tab count and how many sites it is signed in to, a "What it can't do" row (extensions · print · WebRTC camera), and a "Clear a project's data…" button.
  - Marley shows one row per project with its profile path and a pages count, or "no project has started its browser yet" (`system_pages.rs:463-503`). It has no version row, no signed-in count, no limits row and no button.
  - **missing**. **M**
- **Fleet providers.** The board's rows show each provider's address (URL or `ssh …`), and the page has "Add a provider" and "Open Settings".
  - Marley's rows show the provider kind ("an MCP server", "Forge") and not its address. The page has no buttons (`system_pages.rs:507-552`).
  - **look/missing**. **S**
- **For developers, and what's next.** The board's title is "For developers, and what's next". Its rows are the planned work, each with a status ("Next: … · designed"), plus Gate (`script/gates.sh --diff` · 17 gates) and Visual checks (`just e2e …`). Its buttons are "Open the plan" and "Open the guide".
  - Marley's title is "For developers". Its rows are fixed pointers to the repo, the docs map, CONSTITUTION, zed-touchpoints and the plan files (`system_pages.rs:554-582`). It has no planned items with status, no gate or e2e rows, and no buttons.
  - **look/missing**. **S**

## ForgeBoard

Built by #842 (`mw/forge_board.rs`) and #843 (`mw/forge_watch.rs`). Not seen in a run, since no Forge is configured in the shots.

- **Project picker.** The board has a project `<select>` in the header.
  - Marley has none; it uses the token's project, or the provider's first `projects` entry.
  - **deferred**: #842 Out. **S**
- **View tabs.** The board has a Board / By agent / List switch.
  - Marley has the board view only.
  - **deferred**: #842 Out. **M**
- **New Ticket.** The board has a primary "New Ticket" button.
  - Marley has none.
  - **deferred**: #842 Out. **M**
- **Header layout.** The board has an accent tile behind the forge (anvil) icon, sprint name over branch · days left on two lines, and a progress bar with done and in-flight segments beside "7 of 16 done".
  - Marley shows a plain `ListTodo` icon, the sprint and branch on one line, and "N of M done" as text with no bar (`forge_board.rs:459-505`).
  - **look**. **S**
- **Columns.** On the board each column is a framed panel (rounded fill and border, fixed 236 px, scrolling sideways) whose header has a coloured status dot, the name and the count.
  - Marley's columns are flex columns with no frame and no dot; the header is name and count only (`forge_board.rs:507-545`).
  - **look**. **S**
- **Cards: pills.** On the board the kind is a pill whose colour depends on the kind (feature orange, bug red, chore grey, design violet), and the phase is a boxed chip on the right.
  - On Marley's cards the kind is accent-coloured text whatever it is, and the phase is muted text beside it (`forge_board.rs:584-603`).
  - **look**. **S**
- **Cards: agent tile.** The board's card footer has an agent tile (CC/CX glyph, coloured per runtime, with a state dot) before the agent's name.
  - Marley's footer is the agent's name as text, with no tile and no state dot (`forge_board.rs:606-620`).
  - **look**. **S**
- **Detail header.** The board shows the kind and the status as coloured pills (qa-failure pink).
  - Marley shows them as accent text and muted text (`forge_board.rs:648-654`).
  - **look**. **S**
- **Phase progress.** The board's detail has a strip of four phase segments (plan · implement · test · complete), each coloured done, failed or to-do.
  - Marley has no such strip: grepping `phase` in `render_detail` finds nothing.
  - **missing**. **S**
- **Agent and Sprint tiles.** The board shows them as two tiles side by side.
  - Marley shows them as label/value rows (`forge_board.rs:627-672`).
  - **look**. **S**
- **QA-failed callout.** The board shows a red box: "QA failed · 18 m ago" over the failure's reason.
  - Marley has no callout: grepping `QA failed` in `forge_board.rs` finds nothing.
  - **missing**. **S**
- **Comment authors.** The board starts each comment with its author, coloured ("code-2 …", "qa-1 …").
  - Marley shows each comment's age and body with no author; the `Comment` struct reads only `body` and `created_at` (`forge_board.rs:100-104`).
  - **missing**. **S**
- **Open the agent.** The board's primary action is "Open the agent", with "Start here in a worktree" plain and "Move to…" quiet.
  - Marley has Claim, Move to… and Start here in a worktree, the last one in the accent colour. No action opens the ticket's agent (`forge_board.rs:705-745`).
  - **missing**. **S–M**
- **Bottom-row chip.** The board's chip reads "Forge · marley_ide", then "Sprint 14", then "#828 QA failed" as a pink chip with a dot, and at the right "4 online · 7 of 16 done".
  - Marley's `ForgeItem` reads "Forge · N online · #n QA failed" as text, with no project, no sprint, no done count and no pill (`forge_watch.rs:432-463`).
  - **look**. **S**

## ForgeAgents

Built by #841 (forge provider, `mw/fleet_providers.rs`) and #843 (`mw/forge_watch.rs`). Not seen in a run.

- **Online agents view.** The board shows a wide page headed "Online agents", with the note "From forge's agent-list · running and silent past 3 m reads 'needs attention'". The agents are grouped by host, and each host's header has a dot, the host name, its role ("forge-brain · forge-mcp · forge-sched", "factory worker") and an agent count.
  - Forge agents appear only in the right-dock Fleet panel. Its host cards show a server icon, the name and a resource line, with no role and no count (`mw/fleet.rs:707-798`). There is no page title or explanatory note.
  - **look**. **M**
- **Agent row columns.** On the board each agent row has six columns: tile, name over harness ("rustal-harness", "codex app server"), ticket title over folder, a state pill, "output 4 s ago", and an action button.
  - The Fleet row has a tile, the name over "work-item key · phase n/m", and a state chip (`mw/fleet.rs:801-907`). It has no harness, ticket title, folder or "output … ago" column; those appear only in the selected agent's snapshot.
  - **look**. **M**
- **Attach / Give a ticket.** The board has an action on each row: Attach, or Give a ticket for an idle agent.
  - Marley has neither.
  - **deferred**: #841 Out and #843 Out ("Forge has no tool for either"). **M**
- **Rail header.** The board's rail section header is the anvil icon tile, "Forge" and a green "4 online".
  - Marley's is the uppercase label "FORGE" in accent and a muted "N online", with no icon (`forge_watch.rs:286-310`).
  - **look**. **S**
- **Rail rows.** On the board each row's tile is coloured by runtime (CC yellow, CX violet) with a state dot at its corner, and the selected row is highlighted.
  - Marley's tile glyph is accent-coloured for every runtime, with no state dot and no selected state. Clicking a row opens the sprint board, not the agent (`forge_watch.rs:217-272`).
  - **look**. **S**
- **Home card chips.** On the board, "2 working", "1 needs you" and "1 QA failed" are filled pills, and "Board" is a link at the header's right.
  - Marley shows the three as coloured text labels and "Board" as a subtle button on the sprint line (`forge_watch.rs:393-412`).
  - **look**. **S**
- **Bulletins.** The board gives bulletins their own column. Each card shows its kind, *when* it was posted and Dismiss; a critical one has a red border.
  - Marley folds bulletins into Home's Forge card, showing severity, title, body and Dismiss. It has no time (`Bulletin` has no date field, `forge_watch.rs:31-40`) and no border for a critical one.
  - **look**. **S**
- **Ask forge's knowledge.** The board has a search field with result cards, each with a kind pill (lesson / golden / ADR), a title and a line on where it came from.
  - Marley has no knowledge search.
  - **deferred**: #843 Out ("a later ticket with its own design"). **M**
- **Connection card.** The board shows Endpoint, Token ("from the keyring"), Projects and Status ("connected · 120 ms").
  - Marley shows the Fleet panel's source header (name, state chip, failure reason; `mw/fleet.rs:707-736`) and the System tab's Fleet providers row (kind and state). Neither shows the endpoint, where the token came from, the projects or the latency.
  - **missing**. **S**
