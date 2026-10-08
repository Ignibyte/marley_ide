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

- **The manager in the Agent Panel** (#694, 2026-10-08). When the harness Marley follows has a
  manager (with `marley.harness_writes` on), the Agent Panel lists **Manager**. Its thread runs the
  harness's `rh acp` through the command Marley follows the harness with, so a root over SSH is
  reached over SSH.
  - Your messages go into the manager's thread.
  - Its posts and reports stream back, between your turns too.
  - Its confirmations come up as permission requests.
  - The entry follows the manager: it moves with the designation and goes when there is none.

- **The Marley agent sets up a harness seat** (#692, 2026-10-07). `seat_add` on Marley's MCP
  server lets an agent propose a seat ("set up a manager working in /srv/work/x"): a name, Claude
  Code or Codex, a folder and a role.
  - **Asking.** Marley asks you with the same card as a settings change.
  - **Apply.** Runs the harness's `seat add`, answers `starting`, then starts the seat, which
    then shows in the rail. A start that fails comes up as a notification.
  - **Refusals.** A refusal carries the harness's own code.
  - **Where it works.** It needs `marley.harness_writes`. The Marley agent's instructions name the
    tool, and its Zed profile turns it on.

- **A harness seat in one step** (#691, 2026-10-07). `marley: new harness seat` sets up a seat
  from one form:
  - **The form.** Name, agent (Claude Code or Codex), folder and role. A role of `manager` makes
    the seat the root's manager.
  - **The run.** Create runs the harness's `seat add` and `seat start` through the command Marley
    follows it with, so a harness over SSH is set up over SSH. The seat's tab then opens under
    Home.
  - **Refusals.** A refusal stays in the form with the harness's code and reason.

  A session opened with `marley: open harness session` now opens under Home too, as the rail's
  rows do. It needs `marley.harness_writes`.

- **Watching a harness session in a terminal** (#690, 2026-10-07). Each view a harness session's
  tab lists now has Open. Open starts the view's command in a new terminal of the tab's
  workspace:
  - `rh attach` shows the session's tmux pane;
  - `rh view` shows the native view, where Ctrl-b c claims control so you can type;
  - for a Codex or Claude session, its history or an observer.

- **Writing to harness sessions** (#689, 2026-10-07). With `marley.harness_writes` on, a harness
  session's tab can write to the session as well as show its output:
  - answer the question it waits on with a button per option;
  - send it a line of text, with the delivery's state shown;
  - list the commands that watch it, each with Copy.

  `marley: open harness session` opens a session from one of the harness's declared profiles.
  The harness Marley runs itself is followed with `--grant write`. A `marley.harness` command
  needs that grant in its own arguments. Off by default, and off the tab only reads, as before.

- **The Marley agent on Codex and on Zed's agent** (#687, 2026-10-07). Marley's own agent now
  runs on Codex as well as Claude Code. In the Agent Panel, the Marley entry runs the registry's
  Codex adapter in its read-only mode, with the Marley agent's instructions as Codex's developer
  instructions. On Zed's own agent it is a "Marley" profile with Marley's docs, settings, keymap
  and actions tools and no file tools.
  - Settings → Marley → Marley Agent gains an **Agent** dropdown, `marley.assistant.agent`:
    `claude_code` (the default), `codex` or `zed`.
  - The one-time offer looks for Claude Code signed in, then Codex signed in, then a Zed model
    set up, and names the one it found. Turn On writes that agent.
  - With Codex, `marley: open marley agent in terminal` runs `codex --sandbox read-only` with the
    instructions. With Zed's agent, the command is not listed.
  - When the Marley profile goes away and your default profile was Marley, the default goes back
    to Write, as Zed's own profile delete does.
  - The instructions now name `keymap_change`.

- **Key bindings you accept** (#686, 2026-10-07). An agent can propose a key binding with
  `keymap_change` (keystrokes, an action, a context): the same notification as a settings change
  names the keys, the action and where it applies, and Apply adds it to your `keymap.json`
  through Zed's own keymap updater, comments kept, working at once. An action Marley lacks or
  keystrokes that do not parse are refused before you are asked. The e2e runner's copy of the
  settings decides the Marley agent's offer off, so it never comes over a scenario.

- **The Marley agent in a terminal** (#684, 2026-10-07). `marley: open marley agent in terminal`
  starts Claude Code's own interface in a new terminal with the Marley agent's instructions
  (`--append-system-prompt-file`) and the same tools turned off, for those who prefer the TUI to the
  Agent Panel. The palette lists it only while the Marley agent is on.

- **The Marley agent** (#683, 2026-10-07). An agent in the Agent Panel, "Marley", that explains
  Marley and sets it up with you: Claude Code through Zed's `claude-acp` adapter on your own login,
  told what it is for, given Marley's docs and settings tools, and kept from editing files or
  running commands. Off by default (Settings → Marley → Marley Agent); when Claude Code is signed
  in, Marley offers it once. Any agent server can now carry a `_meta` for its sessions in
  `marley.agent_session_meta`, which Zed sends with each session (one hunk in `agent_servers`).

- **Settings changes you accept** (#682, 2026-10-07). An agent can propose a value for one of your
  settings with `settings_change`. Marley shows a notification in every window naming the agent,
  the setting, its value now and the value proposed, with Apply and Decline, and writes the file
  only on Apply, keeping its comments and other keys; the change takes effect at once. A key Zed's
  settings do not have, or a value Zed would not parse, is refused before you are asked; no answer
  within 25 seconds leaves the file as it was.

- **Docs, settings and actions tools for agents** (#681, 2026-10-07). Marley's MCP server gains five
  read tools, so an agent can explain and look up how this Marley works: `docs_search` and
  `docs_read` over Zed's docs and Marley's guide as the build ships them, the keys in the text
  being the ones bound here; `settings_schema`, a setting's type, description, values and default;
  `settings_read`, its value in each settings file, which file wins, the value in effect and the
  value outside projects, with secrets hidden; and `actions_list`, the actions a query names with
  their palette names and keys. `initialize`'s instructions name them. They are the knowledge half
  of the Marley agent to come.

- **A check of an ACP agent that speaks between turns** (#685, 2026-10-07). An e2e scenario,
  `script/e2e/685-acp-messages-between-turns.sh`, drives a scripted ACP agent through the Agent
  Panel: after its turn ends it sends a report and then asks permission. The panel shows the
  report as a paragraph of its own and the request with Allow and Deny, which Marley's rail also
  raises under Needs you, and the choice reaches the agent. It is the proof rustal-harness's
  `rh acp` builds on to show a manager's thread in the Agent Panel. No code changed.

- **Agent tool results that fit, and refusals that say what to do** (#680, 2026-10-07).
  `terminal_read` gives a block's output a page at a time, the newest whole lines that fit in
  12,000 bytes, numbered from the block's first line, with `previous` to pass back as `before` for
  the page before; a line longer than a page keeps its end. A long build log used to come back as
  up to 256 KiB, twice, more than Claude Code takes from a tool, so it moved the answer to a file.
  `terminal_run`'s output is the newest page the same way. Secrets are still hidden across the
  whole output before a page is cut. Marley's MCP server now tells agents at `initialize` which of
  its tools to reach for, and a refused call carries a code (`no_terminal`, `no_block`,
  `output_gone`, `bad_argument`, `not_granted`, `tool_off`, `not_permitted`, `timed_out`,
  `unavailable`, else `refused`) and next steps. The stand-in agent of the e2e fixture pages,
  prints refusals and prints the instructions.

- **The rail's Containers list folds** (#670, 2026-10-06). The CONTAINERS label is a header
  with a chevron and the number of ports; a click on it, or on the chevron, shows the rows or
  folds them away. It starts folded, and each window remembers it across a restart.

- **Containers in the Rail** (#669, 2026-10-06). A setting on the Settings window's Marley page,
  `marley.rail_containers`, on by default: off, the rail leaves out the CONTAINERS list after the
  projects, the ports of containers running on this machine that no project's folder holds. A
  container whose Compose folder is in a project still shows under that project. Each e2e run's
  copy of the settings turns it off, so no scenario lists the machine's containers or can stop one;
  #614's scenario turns it back on.

- **The Secrets tab** (#667, 2026-10-06). Rusty's vault in a center tab, from the lock after Skills
  in the Brain view or `rusty: open secrets`: the names show to anyone, and a value shows, changes
  or goes only behind your PIN. Set a PIN (six characters or more, typed twice), then Unlock for
  a few minutes: each name has Reveal (Hide), Copy, Replace and Delete, asked first, and a new
  secret is a key and a value with Set. The tab locks when the unlock runs out, on Lock, when
  Marley's window loses the focus, and when the tab closes. Every PIN and value field hides what
  you type and empties once sent; the unlock lives only in the tab. Rusty's refusals, a wrong PIN
  say, show in its words.

- **Rusty's own settings on the Rusty's Server page** (#666, 2026-10-06). The page shows how far
  Rusty's brain search has pages embedded, then the ten settings Rusty's app lists (the vault and
  notes folders, the embedding provider and model, Ollama's address, how long a Secrets unlock
  lasts, the skills store, enrichment, the default workflow), each with Rusty's words and its
  default, then any other key Rusty stores and a row to add one. Type in a field and press Enter
  to change it; Rusty keeps it and the page reads it back. A key, token or password shows hidden,
  and typing a new value replaces it without the old one ever showing.

- **The Skills tab** (#665, 2026-10-06). Rusty's skills store in a center tab, from the hammer
  after Memory in the Brain view or `rusty: open skills`: staged skills first, waiting for your
  approval, then the active ones Claude Code loads, then the scripts beside them. Choose a skill to
  edit its description and body and Save, run Rusty's safety scan, or Approve or Reject a staged
  one; when the scan blocks an approval, Rusty says why and Approve Anyway appears. Choose a script
  to edit it, or Run it in a terminal. New Skill makes one, active or staged, and Delete asks
  first. Rusty commits every change and its refusals show in the tab.

- **The Memory tab** (#664, 2026-10-06). What Rusty remembers across conversations, in a center
  tab from the book after Decisions in the Brain view or `rusty: open memory`: the memories in
  Rusty's order (high first, the newest first within each), each with its category, importance,
  who stored it and the day it last changed. Type a line at the top, give a category and Low,
  Normal or High, and press Enter to add one; the Category menu shows one category; a click opens
  a memory to edit its text, category or importance, or to delete it after asking. Rusty's
  refusal shows in the form, and a change made elsewhere shows at once.

- **Capture a line or a URL, and import a vault, into Rusty from the palette** (#663, 2026-10-06).
  `rusty: capture to today` and `rusty: capture to inbox` open a one-line form; Enter adds the line
  to that page's timeline and a toast names the page, with Open. `rusty: capture url` has Rusty
  fetch a web page or a file and keep it as a source page, which opens when Rusty answers; a fetch
  that failed still makes the page, which says why, and a toast gives the reason. `rusty: import
  vault` asks for an Obsidian vault's folder, shows Rusty's plan (what comes in, what is skipped as
  already in the brain, the links that will not resolve, the bookmarks), and Import brings it in
  and shows the report. `rusty: open today` opens today's daily note. Rusty's refusal shows in the
  form. A call to Rusty can now take longer than 5 s where Rusty needs it: 45 s for a URL, 60 s for
  a plan, 10 minutes for an import.

- **Favourites from Rusty's bookmarks** (#662, 2026-10-06). The Brain view lists Rusty's bookmarks
  under Favourites, above the tree, in Rusty's order: a page opens as a tree row does, a folder
  opens in the tree, a search runs in the search field, and a heading opens its page with the
  heading at the top. Right-click one to rename or remove it. A Page tab's star, or Ctrl+D in the
  tab, adds the page or removes it, and `rusty: open page` lists favourite pages, starred, after
  the page in front and before the recent ones. On the embedded connection a change made in Rusty's
  app shows at once. The bookmarks stay in Rusty's vault; Marley keeps no copy.

- **Jev through Cloudflare Workers AI, for every project or some** (#548, 2026-10-05). System
  One's new `cloudflare` provider asks the same model through Cloudflare, which states it keeps
  no data, in the account set as Cloudflare Account ID on the Marley settings page.
  `provider_by_project` sends chosen projects that way by folder, so a client's code can go
  through Cloudflare while the rest go direct. The token comes from `MARLEY_CLOUDFLARE_API_TOKEN`
  or Set Cloudflare Token in System One calls, which shows where it came from and never the
  token. Each call's row names its provider, and Cloudflare's own words show when it refuses.

- **Record a decision's follow-up from the Decisions tab** (#660, 2026-10-05). Follow Up on a row
  under Due, or Follow Up… in any decision's right-click menu, opens a form: choose Kept, Revised
  or Superseded, write how it went, and give the next follow-up day when revised or pick the
  decision that replaced it when superseded. Enter or Record sends it to Rusty, Shift+Enter starts
  a new line, and Escape closes the form without recording. If Rusty refuses, its words show in
  the form and nothing is written. The rows also show the day of each decision's last follow-up,
  and a replaced decision names the one that replaced it, which opens with a click.

- **The Decisions tab** (#659, 2026-10-04). Rusty's decisions in a center tab, from the double
  check after Tasks in the rail's Brain view or `rusty: open decisions`. Under Due come the
  follow-ups whose day has come, in Rusty's order, an overdue one's day in the warning colour with
  the word overdue; then every decision, the newest first, under their count, each with its
  status (decided, kept, revised or superseded), the day it was decided and its follow-up day.
  A click opens the decision's page. The tab reads again when Rusty announces a change, and a
  failed read shows Rusty's words with Read again. What is due and what is overdue are Rusty's
  answers; Marley works out no date.

- **The Tasks tab** (#658, 2026-10-04). Rusty's to-do lists in a center tab, from the list icon in
  the rail's Brain view or `rusty: open tasks`: the lists on the left, the chosen list's tasks on
  the right, in Rusty's order. Add a task from the field above them; check one done with its box
  or Space; rename it in its row with F2 or a double-click; archive it with Delete and see
  archived tasks with Show archived; delete one for good after a prompt; and move one by dragging
  it or with Alt+Up and Alt+Down. Lists are made with +, and renamed or deleted from their menu.
  Every change is one of Rusty's task tools, sent in order and read back. The tab reads again
  when Rusty announces a change, when you come back to it, and on Refresh. The Knowledge panel's
  project view opens it on the project's task group.

- **The Graph tab's groups, display and forces, kept across a restart** (#657, 2026-10-04). The
  Graph tab's panel gains Rusty's Groups, Display and Forces. A group is a query in the filter's
  words with a colour: the pages it matches take that colour, and its row counts them. The first
  matching group wins, and a click on the dot gives the next colour. Display has Arrows (a head at
  each link's and decision edge's target), Text fade threshold, Node size and Link thickness, and
  Forces has Center, Repel and Link force and Link distance. Each is on a slider that takes a
  drag, a click on its track, the arrow keys, Page Up and Page Down, and Home and End. The groups,
  the switches, the depth and the sliders are one set for every Graph tab, kept across restarts,
  and a Graph tab open at quit comes back in its place with its scope, page, filter, hidden types
  and panel, unless Rusty is off at the start.

- **A brain page's outline, and edits in place** (#656, 2026-10-04). In Read, a column beside a
  Page tab's body lists the page's headings, indented by level; a click brings one to the top of
  the page, and the Outline button beside Forward (`rusty: toggle page outline`) hides the column.
  In Edit, Zed's outline panel lists the file's headings. The title, the page's name in the tab's
  header and each property are edited where they are drawn: the title sets the `title` property,
  the name renames the page in its folder (Rusty rewrites the links to it, and a toast says in how
  many pages), and a property edits by its kind: text, a number, a `YYYY-MM-DD` date, a checkbox,
  or a list of chips with add and remove. Each property can be removed, and Add property asks for
  the kind, then the key. The changes go to Rusty one at a time, in order, and every Page tab
  follows a rename or a move, from the tab or from the Brain view.

- **The Knowledge panel's project view** (#655, 2026-10-04). With no brain page in front, the
  Knowledge panel shows your project's brain page: the project page whose `path` lists the
  window's folder, else the one named like it. It shows the page's summary, the follow-ups due
  among the decisions linked to it, and the open tasks of its Rusty task group (named by its
  `task_group`, else the group named like the page), with Open Page. When no page matches, Link a
  Page adds the folder to the page you pick, keeping its `path`; when several share the folder's
  name, each has Link; Link a Task Group writes the group you pick. Nothing is written except on
  your pick, and Rusty commits each write.

- **Open a brain page by name** (#654, 2026-10-04). `rusty: open page`, Ctrl+Alt+U from anywhere
  in the window, opens a picker over every page of Rusty's brain: its title and its slug on each
  row, the pages you opened most recently first (kept across restarts), and the matched letters
  lit as you type. Enter opens the page in a kept tab, or brings forward the tab that shows it.
  When what you type, read as a path, names no page, the list ends with Create page, which makes
  it there. The query is matched in Marley; nothing you type goes to Rusty.

- **Marley as Claude Code's IDE** (#653, 2026-10-04). With Claude Code IDE Link on in the Marley
  settings (off by default), Marley serves Claude Code's IDE connection for each local project: a
  lock file in Claude Code's `ide` folder, and the project's port in its new terminals, so a
  `claude` started there links to that project's Marley, and `/ide` reaches it from other
  terminals. Claude Code then gets the selection and the open file of your last file editor with
  each prompt, the language servers' diagnostics through its `getDiagnostics` tool, the project's
  folders and its open editors, and send selection mentions the lines in its prompt instead of
  typing `@path#La-b`. The parts Claude Code's docs leave unnamed are rows of the version table,
  which no release has been checked on yet: until one is, the agent bar says so, and the three IDE
  items under Agent Versions turn them on. Diffs opened in Marley come later.

- **Agents report their state to the terminal they run in** (#652, 2026-10-04). Every local
  interactive terminal now names `MARLEY_BIN`, a small program Marley writes into its data
  directory. `"$MARLEY_BIN" report STATE --source S --seq N ...` and `"$MARLEY_BIN" release
  --source S` take exactly the arguments rustal-harness's `rh report` and `rh release` take, so
  one Claude Code plugin can report to either host. Marley knows the caller by its processes: the
  report must come from inside one of its terminals, and the process that sent the first one holds
  the terminal until it releases or ends, so a `claude -p` run inside the session cannot move the
  row. While a terminal holds a report, its rail row shows the reported state, with the prompt and
  the tool in flight still read from the hook frames, and a restart resumes the reported session
  in the agent's folder. A refusal names the rule it broke, with the harness's names. Tasks and
  remote terminals get `MARLEY_BIN` empty. Until the shared plugin loads in Marley's terminals
  (the next slice), nothing reports by itself, and the hook frames go on as before.

- **Codex's approvals answered from the inbox** (#651, 2026-10-04). A Codex running on its own App
  Server (Codex App Server on) lists what it asks to approve in the rail's Needs you: a command
  with its folder, the files a change edits, the permissions it asks for, and an MCP server's
  question. Each has the decisions the request offers in place: Allow, Allow session, Deny and
  Stop turn for a command or a file change, Allow, Allow session and Deny for permissions, and
  Deny and Dismiss for an MCP server's question, each answering only the request you saw. Codex's
  own prompt in the terminal closes when you answer from the inbox, and the entry leaves when you
  answer in the terminal. A question of Codex's own still opens the terminal.

- **Codex's state from its own App Server** (#650, 2026-10-04). With Codex App Server on in the
  Marley settings (off by default), each Codex Marley launches in a local project runs against an
  App Server of its own, which Marley starts for the terminal and joins as a second client once
  Codex has. Codex's row then reads its thread's own state, `working`, `waiting on approval`,
  `waiting on input`, `idle` or `failed` with the turn's error, and its token use, such as
  `working · 12k tokens`, in place of the terminal's quiet; a waiting Codex enters the inbox as
  Codex's, and a click shows its terminal; the close guard asks before a working Codex closes;
  and the `full access` chip follows the sandbox the thread reports. The server lives as long as
  its terminal. It runs on Codex 0.155.1 to 0.158.0, the versions Marley tested; outside them
  Codex starts as before and the agent bar says why. Marley answers nothing Codex asks, and
  Codex's requests then name Marley in their user agent.

- **Agent prompts in a tab** (#649, 2026-10-04). With Agent Prompts in a Tab on in the Marley
  settings (off by default), the terminals Marley opens for agents give them Marley's editor,
  `marley-edit`, as `VISUAL` and `EDITOR`, over what the user's shell files export. Ctrl-G and the
  agent bar's pencil then send the agent its own editor key, and the agent's prompt opens in a
  Marley tab; closing the tab hands it back to the agent, which sends nothing until you do, and
  its terminal comes back to the front. A save alone does not end the edit, as with `zed --wait`.
  Everywhere else Ctrl-G opens the Rich Input overlay as before.

- **Marley checks the agent's version** (#648, 2026-10-04). Marley reads the version of the
  `claude` and `codex` it runs (`MARLEY_CLAUDE` and `MARLEY_CODEX`, else the PATH's) when it
  starts and again when an agent bar shows after the program changed. Claude Code's prompt tags,
  which tell your prompts from task notifications and system reminders and which Claude Code does
  not document, stay on for 2.1.283 and later 2.1 releases, the versions they were checked on; on
  another version every prompt reads as yours, and the agent bar's chip says which version Marley
  found and which it checked. A click opens the setting that turns the tags on anyway,
  `marley.allow_untested_versions`, in a new Agent Versions section of the Marley settings.

- **The Graph tab** (#647, 2026-10-04). `rusty: open graph`, or the graph icon after Today in the
  rail's Brain view, draws Rusty's vault in a center tab: pages as dots sized by their links and
  coloured by page type, links as lines, and a decision's consulted, supersedes and follows-up
  edges dashed, a colour each. Local shows the neighbourhood of the brain page last in front, one
  to four links deep, and follows it as you open pages; Vault shows every page. A click opens a
  page, the wheel zooms about the pointer, a drag pans or holds a dot while the rest settle. The
  panel filters by words, `tag:`, `path:` and `type:`, hides a page type from its legend, and turns
  tags, unresolved links, decision edges and orphans on or off. The layout runs off the window's
  thread; past 2,000 dots it keeps the most linked and says so. A change Rusty announces shows
  with no click, and with Rusty off the tab is empty and makes no call.

- **The Knowledge panel** (#646, 2026-10-04). While Rusty is on, the right dock holds a Knowledge
  panel (the Book button, or `rusty: toggle knowledge panel`). With a brain page's tab active it
  shows that page's tags with their page counts, the pages that link to it with the line each link
  sits on and the link lit, and its own links in order; a click opens one, and a link to no page
  yet offers Create, which makes the page through Rusty. A tag's click searches for it. Its field
  is brain search with Rusty's operators, sent on Enter, with Match Case and Regular Expression;
  the results show each page's title, slug and matching words, and Up, Down, Enter and Escape walk
  and open them. A change Rusty announces shows with no click. With Rusty off there is no button,
  the command says where to turn Rusty on, and a showing panel closes.

- **A brain page in a tab** (#645, 2026-10-04). A page opened from the rail's Brain view, or with
  the `rusty::OpenPage` action, shows in a center tab: its title, its properties (a list as chips)
  and its body, drawn by Zed's own Markdown renderer with headings, task boxes, highlighted code
  and tables. Wikilinks are links: one to a page opens it in the same tab, at its heading when the
  link names one, and one to no page yet is drawn muted and says so when clicked. Back and Forward
  (Alt+Left, Alt+Right) go through the pages the tab showed. Edit shows the page's file in an
  editor inside the tab, with Zed's unsaved dot and save; Read, Back and Forward save first,
  without formatting. One click in the tree previews a page, a double-click keeps it, and the tab
  already showing a page comes forward. A page that changes shows its new text with no click.

- **Rusty's vault in the rail** (#644, 2026-10-04). While Rusty is on and connected, the rail's
  header shows a Projects button and a Brain button in place of PROJECTS, and Ctrl+Alt+V flips
  between them from anywhere in the window. Brain shows Today (today's daily note, made when
  missing), a brain search field that asks Rusty when you press Enter and lists its hits, and the
  vault's folders and pages with their page counts. A click opens a page in a preview tab, a
  double-click keeps it; until the Page tab lands (#645), a page opens as its Markdown file. The
  list keys walk the tree. The right-click menus make pages and folders, rename and delete them,
  and a drag moves one into another folder, each through Rusty's tools: Marley writes no file of
  the vault, a refusal shows Rusty's message, and a delete asks first. Changes Rusty announces
  show with no click, a dot on Projects says something there needs you, and a lost connection
  puts the rail back on Projects. With Rusty off, the key says so in a toast.

- **Marley connects to Rusty when Rusty is turned on** (#643, 2026-10-04). `marley.rusty` is
  Rusty's switch in Marley, off by default, with a Rusty section on the Marley settings page.
  On, Marley starts `rusty-mcp` itself (`embedded`: `MARLEY_RUSTY_MCP`, else your PATH) or
  connects to Rusty's running service on this machine (`service`), checks the connection every 5
  seconds and connects again after a growing wait when it is lost. The section's Rusty's Server
  page says whether Marley is connected and why not, and shows Rusty's own Embedding Provider,
  read from Rusty and written back to it when you pick another; Marley keeps no copy. A new
  crate, `marley_rusty`, holds the pure half and a stand-in `rusty-mcp` the e2e scenarios use, so
  no run reaches your own Rusty.

- **SSH links that know they are dead** (#641, 2026-10-04). Every ssh Marley starts checks its
  link every 5 seconds, so a remote terminal or a Fleet host whose link went silent is known dead
  within 20 seconds instead of when the network gives up, which could take a quarter of an hour.
  A remote terminal whose link died keeps its last screen, dimmed, under a line naming the host,
  what ssh said and when Marley checks next; what you type is not sent, and the line counts it.
  Marley checks the host after 1 second, then twice as long each time up to two minutes, and when
  it answers runs the terminal again in the same tab, on the same tmux session, without moving
  the focus. Rerun tries at once; a refused login stops the checks and says why. The Fleet panel
  no longer freezes on a host that went silent behind an open connection.

- **A harness session's source, progress and quota on its row** (#640, 2026-10-04). When
  rustal-harness says where a session's state came from, how far its agent is and how much of its
  account's quota it has used, the rail's Harness section shows it: a line such as `40 % · Running
  the tests`, a line for the most-used quota window such as `five_hour 62 % · resets in 1 h 35 m`,
  and, with the pointer on the row, where the state came from, every window and the account. A
  state the harness read off the agent's screen, or one from a source Marley does not know, is
  drawn weaker, a grey dot with the source's word at the end of the line, and its question stays
  out of the "Needs you" inbox. A value that does not parse is left out, and an account name that
  looks like an email address is never shown.

- **Mutation runs on GitHub's runners** (#638, 2026-10-01). `script/mutants` runs the mutation
  pass (`just mutants`), or one shard of it, and merges any runs' outcomes into one table by crate
  with every missed mutant. `just mutants-cloud` sends the pass to GitHub's free runners, 16
  shards at once by default: it pushes HEAD to a `mutants/<stamp>` branch, follows the run,
  downloads the report and deletes the branch. The fork's Actions have to be turned on once in
  its Actions tab, then `script/mutants cloud-setup` turns off the workflows the fork carries
  from Zed; `docs/marley/mutation-runs.md` has the steps.

- **Claude Code sessions come back after a restart** (#540, 2026-10-01). Quit Marley while Claude
  Code runs in a terminal, and at the next launch that terminal runs `claude --resume` with the
  same session, in the folder the session started in, so the conversation picks up where it
  stopped. A session you ended yourself (`/exit`, logging out) comes back as a plain shell, and
  each session resumes in one terminal. Resume Claude Code Sessions on the Marley page
  (`marley.resume_agents`) turns it off.

- **Gaps between blocks, and a density setting** (#631, 2026-10-01). A terminal's blocks now stand
  half a row apart, the prompt you are at included, and a block whose prompt took one row gets a
  two-line header: the folder and branch over the command. The prompt stays on the last row, and
  a view scrolled back starts at the top edge. Clicks, double clicks, drags, right clicks and link
  hovers land on the rows as drawn. Block Density on the Marley page (`marley.block_density`)
  goes back to `compact`, #630's look with no gaps.

- **History suggestions in the prompt editor** (#637, 2026-10-01). The shell's prompt editor, on at
  every prompt since #627, shows the rest of a history command your text starts, dimmed after it,
  as the terminal's own prompt did (#484): this session's commands first, then the shell's history
  file. → at the end of the line takes it, and moves the cursor anywhere else; the suggestion shows
  only while the cursor is at the end. Where no suggestion applies, the English hint shows as
  before.

- **Rusty's tools for Zed's agents** (#633, 2026-10-01). Where `rusty-mcp` is on your PATH,
  Marley offers Rusty's MCP server to Zed's agents as the context server `rusty`, beside its own
  `marley` server, so an agent in Marley reaches Rusty's brain loop and its other tools; Zed asks
  before each call as for any server. Rusty Tools for Agents on the Marley page
  (`marley.rusty_tools`) turns it off, and a `rusty` entry of your own in `context_servers` wins.

- **The embedded harness** (#632, 2026-10-01). With `marley.embedded_harness` on, Marley runs
  rustal-harness's runtime itself: it finds `rh` (`MARLEY_RH`, else the PATH), serves a root in its
  own data folder, and shows that harness's sessions in the rail's Harness section. The header also
  says when the runtime stopped and why, or that no `rh` was found and where Marley looked (a
  tooltip holds the whole reason), and a stopped runtime is started again after a growing wait;
  the harness's sessions outlive it. `marley.harness`, when set, wins.

- **The harness's sessions in the rail** (#534, 2026-10-01). With `marley.harness` naming
  rustal-harness's MCP server (`{"command": "/path/to/rh", "args": ["--state", "<root>",
  "mcp"]}`), the rail gains a Harness section after the projects: the connection's state, and a
  row per harness session with its state, the question it waits on, and how long a working one
  has been quiet. A click opens a read-only tab with the session's last 500 lines, which follows
  its output. A waiting question joins the approvals inbox with its options. When the harness
  stops answering, the section says why, keeps its rows marked stale, and reconnects on its own.
  Read side only: Marley answers nothing.

- **Block headers with the folder and branch, on by default** (#630, 2026-10-01). A block's header
  shows the folder the command ran in (home as `~`, a long path cut to its last two folders) and
  the git branch it was on when it started, so a later checkout leaves older headers as they were.
  Where the prompt took two rows the folder and branch sit above the command; on one row they
  follow it. Block Headers is now on unless you turn it off. Gaps between blocks and a density
  setting wait in TICKET-631.

- **Block headers in navigation and search** (#629, 2026-10-01). With Block Headers on, the
  command pinned over a scrolled-back block reads as its header, with no `$ `, and a search no
  longer highlights a match inside a prompt the header hides. Moving between blocks still lands on
  a block's header, and bookmark ticks keep their place.

- **Block headers** (#628, 2026-10-01). With Block Headers on (`marley.block_headers`, off by
  default), a command's prompt rows are drawn as Marley's header: the command in the terminal's
  font with its pill, in place of the prompt the shell drew. The header takes exactly the rows the
  prompt took, so nothing scrolls or moves, and the prompt waiting for your next command stays the
  shell's. A click on a header starts no selection.

- **Shell integration for fish** (#466, 2026-10-01). A fish that Marley starts in a local
  terminal reports its prompts and commands as bash and zsh do, so its commands are blocks with
  their folder, exit status and pills, and the prompt editor docks at its prompt. Marley puts a
  vendor snippet's folder first on `XDG_DATA_DIRS`; the snippet puts your own `XDG_DATA_DIRS`
  back and takes Marley's values out of the environment before your `config.fish` runs. fish's
  history file feeds the ghost text and the prompt editor's completions.

- **English at the prompt, second stage** (#573, 2026-09-30). A line typed at a shell's prompt
  that Marley's own rules leave open, a command's name followed by plain words such as `kill the
  dev server`, can be read by the System One layer as a command, a request, a comment or a
  command followed by English, and the reading shows after the line: with a question mark in
  Suggest, plainly in Act, where `rm the old build folder` also has the words `rm` would take as
  files drawn in the warning colour. It asks only after 250 ms without typing, for a listed
  project, never for a line with a secret in it, and Enter never waits for it. The use is
  `typed_line` in `marley.system_one.uses`, off by default. #557's hint and Ctrl+Shift+Enter now
  work in the shell's prompt editor too, where they had gone missing since #627.

- **The prompt editor by default** (#627, 2026-09-30). In a terminal with the focus, the shell's
  prompt editor docks by itself at every prompt and takes the keys: no Ctrl+G first. When a
  command starts or a full-screen program shows, it closes and the program gets every key raw,
  and it comes back, empty, at the next prompt. Escape gives the keys to the shell until then,
  and Ctrl+C empties the editor. The setting `marley.prompt_editor` (Prompt Editor on the Settings
  page's Layout section) set to false brings back Ctrl+G only.

- **A command's colours at the prompt** (#626, 2026-09-30). What is typed at a shell prompt is
  drawn in the theme's syntax colours, as Zed colours a shell script, and the shell's prompt
  editor reads the same way. The colours are painted over the typed cells, which keep what the
  shell gave them, so readline's own redraws are never fought; a finished block keeps the shell's
  drawing.

- **Completions in the shell's prompt editor** (#625, 2026-09-30). In the editor Ctrl+G opens at
  a shell prompt, Tab lists completions in Zed's menu: the word before the cursor as a path in the
  prompt's folder, and the command so far as the start of one from the shell's history or the
  project's tasks. Enter or Tab takes an entry; the line runs only at the next Enter.

- **A prompt editor at the shell's prompt** (#624, 2026-09-30). At a shell prompt with no agent
  running, Ctrl+G opens the footer editor (#481's) for the shell, holding the line typed so far:
  Zed's editing for a command. Enter clears the shell's line and runs the command as a block;
  Escape leaves the shell's line as it was.

- **A failed block's errors as the project's diagnostics** (#623, 2026-09-30). The errors and
  warnings a failed block names in the project's files (rustc, the GNU shape, tsc, Python) are
  the project's diagnostics: counted in the status bar, listed in the Diagnostics view as
  `marley`'s, and underlined in the editor. The next run of the same command in the same folder
  replaces them; a run that succeeds clears them.

- **A task block's Rerun Task, and its pill in place of Zed's summary** (#622, 2026-09-30). A
  finished task's block offers Rerun Task on hover and in its menu, which runs the task again in
  its tab through Zed's task machinery, as the tab's own Rerun does. In the Marley layout the
  block's pill says how the task ended, so Zed's `Task … finished` line is left out; the Zed
  layout keeps it.

- **A task's run is a block** (#621, 2026-09-30). A task or runnable's run in its terminal is a
  block, as a typed command is: its output framed with the bar, the wash and the pill (its exit
  code when it failed), its block menu, and its state on the terminal's rail row. Tasks run
  without the shell's hooks, so the terminal opens the block when the task spawns and finishes
  it with the task's exit code, before Zed's summary lines.

- **Jump to a failed block's first failure** (#620, 2026-09-30). A failed block whose output names
  a failing place (rustc and cargo, gcc, clang and go, tsc, or a Python traceback) shows a Jump to
  Failure chip, and its right-click menu Jump to First Failure: the block scrolls to its first
  error and the file opens at that line and column, against the folder the command ran in.
  `marley: jump to first failure` does it for the focused terminal's newest failed block.

- **A block's path links open at the block's folder** (#619, 2026-09-30). A path a command
  printed, such as `src/main.rs:2:5`, opens with Ctrl+click against the folder that command ran
  in, and its tooltip names that file, even after the shell has moved on and Zed's own guess at
  the line's folder has given way (it does once the scrollback is full). Remote terminals and
  blocks on another host keep Zed's resolution.

- **Threads and ports under a closed project** (#617, 2026-09-30). A project the window holds no
  workspace of, as a restart leaves every one but the shown, lists its agent threads and the
  ports listening in its folders under its dimmed header, and its chevron folds them. A thread
  row opens the project and then the thread, loading the Agent Panel if the new workspace has
  none yet; a port row's Open and Show Logs open the project first, while Copy, Stop and Restart
  need nothing opened. Next and Previous Thread pass over a closed project's threads, and
  `ports_list` names a closed project's ports.

- **Delete a thread, and bring archived threads back, from the rail** (#616, 2026-09-30). A thread
  row's right-click menu adds Delete Thread…, which asks first, then lets every Agent Panel of the
  window drop the thread (a conversation it keeps would save it again), deletes its record,
  cleans up the worktrees its archive kept, and deletes the agent's own session where the agent
  can. A project's right-click menu adds Archived Threads, its archived threads with when each was
  last updated; choosing one opens it, which brings it back to the rail.

- **Restart a service from its port row, with its state and logs** (#615, 2026-09-30). A port row
  has a right-click menu: Open in a Browser Tab and Copy URL, then, for a server a systemd service
  runs, Restart Service, Stop Service and Show Logs (Stop Container or Stop Process for others).
  Restart keeps the row while the port is quiet, then the new process takes it; the unit's state
  shows on its line when it is not simply running (`starting`, `restarting`, `stopping`, `failed`
  in red, `stopped`), from one `systemctl show` a scan. Show Logs opens `journalctl --user -u
  <unit> -f` in a new terminal of the project. The row's tooltip no longer lies over its menu.

- **Container ports in the rail** (#614, 2026-09-30). A port a Docker container publishes belongs to
  root's `docker-proxy`, which the ports scan could not see; Marley now reads each proxy's command
  line (readable by anyone) and lists its port, and a rootless Podman helper's port as a
  container's rather than a process's. Where `docker ps` or `podman ps` answers, the port is named
  by its container ("container web") and listed under the project its Compose folder is in; the
  rest go under a CONTAINERS label after the projects, with the container's address. Stop runs
  `docker stop` or `podman stop`; where the engine refuses (you are not in the `docker` group),
  Stop says why and offers `docker stop $(docker ps -q --filter publish=<port>)` to copy. An
  engine's own systemd unit (`docker.service` and the like) is never what a port's Stop stops.

- **Move a terminal to another project in the rail** (#613, 2026-09-30). Drag a terminal's row onto
  another open project's (or group's) header, or choose Move to Project in its right-click menu,
  and the terminal joins that project's workspace with its shell still running, its scrollback,
  its id and what Marley keeps for it (agent events, turns) intact; the project shows. It is the
  same terminal view, moved with Zed's `move_item`, and Zed's `TerminalView::added_to_workspace`
  now re-points the view at the project it joins (a small Marley hunk), so its links, menu and
  Marley's tools act there. After a restart it comes back under its new project.

- **Real workflow stores in the Fleet panel, over MCP or HTTP** (#611, 2026-09-30).
  `marley.fleet.providers` takes `{ "kind": "mcp", "command": …, "args": […] }` (an MCP server
  Marley starts, through Zed's `context_server`), `{ "kind": "mcp", "url": … }` (an MCP server
  over HTTP) and `{ "kind": "http", "url": … }` (the contract's plain `GET /marley/v1/…`), each
  with an optional `name` and `bearer_env`, the environment variable that holds its token. Marley
  asks each for its handshake, then its agents every `poll_s` (or what changed, when it offers
  `changes`), and the detail of the agents a panel or tab shows. Each store's header says how it
  stands: `ready`, `connecting`, `unreachable` with the cause (tried again after 1, 2, 4, 8, 16,
  then 30 seconds, its agents `offline`), `stale` after three polls with no answer (its last list
  kept), or `incompatible` with the contract it speaks. Stores are polled together and one's
  trouble leaves the others as they are. The token is sent as a bearer and never logged or shown.

- **Hosts read over SSH, and their agents** (#610, 2026-09-30). The settings can list hosts under
  `"marley": { "fleet": { "hosts": [...] } }`: `{ "ssh": "user@host" }` or `{ "local": true }`,
  each with an optional `name` and `id`. Every 5 seconds while a Fleet surface shows, Marley runs
  its own POSIX `sh` script on each (`crates/marley_workbench/bin/marley-collect.sh`, carried in the
  SSH command, base64, with `BatchMode`, a 5 s connect timeout and one kept connection; nothing is
  installed) and reads CPU, memory, disk and network, and the `claude` and `codex` processes
  running there (plus `agent_processes`). Each host's header in the Fleet panel shows a line of
  its resources. A process whose `MARLEY_FLEET_SESSION` names a store's agent, or that matches
  one by host, runtime and folder, joins that agent: its snapshot and tab take the host's
  numbers and name the process. The rest are listed under a Hosts section as `running` agents
  with no work records. A host that does not answer reads `unreachable`, with SSH's error in the
  tooltip, and the store's agents on it read `offline`; a destination that starts with a dash is
  refused and nothing runs.

- **The Agent tab** (#609, 2026-09-30). A double-click on an agent in the Fleet panel, Enter on
  the selected one, or the snapshot's Open button opens the agent in a tab of its own in the
  center, or brings forward the one already open. The tab shows its run's phases on a timeline,
  each bar from its start to its end with its gates under it and a failed gate's detail in red;
  its events, newest first; its host's CPU, memory and network as lines over the samples Marley
  kept (one per read, the last 30 minutes, in memory only); its tokens for the run and the day,
  with cache reads; and the other agents on its host, each opening its own tab. The tab is
  read-only and not kept across a restart. The fleet now reads while a Fleet panel shows or an
  Agent tab is its pane's active item.

- **An agent's snapshot in the Fleet panel** (#608, 2026-09-30). One click on an agent's row
  selects it, and the panel splits: the list above, the agent's snapshot below. The snapshot
  shows its state and for how long, what runs it and where (host and folder), its work item with
  the store's status, its run as a strip of phases (passed green, active blue, failed red, with
  the active or failed phase named), its host's CPU and memory as bars, its tokens today, and its
  question with the options. Up and Down move the selection while the panel has the focus, and
  the line between list and snapshot drags to resize them. A section the provider does not offer
  is left out, and the panel reads the full detail only of agents a panel has selected.

- **The Fleet panel, on a pseudo provider** (#607, 2026-09-30). `marley: toggle fleet` opens a
  panel in the right dock that lists the agents a workflow store reports, grouped under their
  hosts: each with its runtime's mark, its name, its work item's key and its phase as `n/m`, a
  mark when it waits on a question or its run failed, and a state chip (`working`, `waiting`,
  `error` and the rest, or `stale` when it has not reported for three polls). Its data comes from
  the providers under `"marley": { "fleet": { "providers": [...] } }`; with none, it says the
  fleet is not set up and names the setting. The one provider so far is `{ "kind": "pseudo" }`:
  the examples of `docs/marley/fleet-contract.md`, three agents on two hosts, moved by the clock
  (a phase a minute, events, tokens, one agent going quiet), until the Rustal services serve the
  contract. The contract's types live in a new pure crate, `marley_sdk`
  (`marley.work/v1` and `marley.host/v1`, the stale rule, and the pseudo provider). The panel
  reads only while it shows.

- **A window's closed projects stay in the rail** (#606, 2026-09-30). After a restart Zed reopens
  only the project a window showed and keeps its others as project groups with no workspace, which
  the rail used to leave out, so they vanished from it. They are now listed, dimmed, as a header
  alone, with a tooltip that says they are not open; a click, or Enter, opens one as Zed's Threads
  Sidebar does (`MultiWorkspace::find_or_create_workspace`, through Zed's connection modal for a
  remote project), and Zed brings its terminals and tabs back. A closed header keeps Move Project
  Up and Down and Remove Project; Next and Previous Project pass over closed projects.

- **Archive an agent thread from the rail** (#605, 2026-09-30). The pointer on a thread row shows
  an Archive button at the row's end, in place of its status mark, with the tooltip "Archive
  Thread"; a right-click on the row opens a menu with Archive Thread. Either archives the thread
  as Zed's thread history does (`ThreadMetadataStore::archive`), without opening it, and the rail
  drops its row. It stays archived after a restart; Zed's archive view in the Agent Panel brings
  it back.

- **Port rows that know a service** (#603, 2026-09-30). A port row whose process runs in a systemd
  service (read from its `/proc/<pid>/cgroup`: a path ending in `<name>.service`) shows the unit
  on a line under the URL, and its tooltip says whether it is a user or a system service. Stop
  then stops the service instead of signalling the process, which a restart policy would start
  again: `systemctl --user stop <unit>` for a user service, `systemctl stop <unit>` for a system
  one, which asks the desktop's polkit agent. When `systemctl` fails, a toast names the unit and
  systemd's reason and offers the command that stops it by hand (`sudo systemctl stop <unit>` for
  a system service) to copy. Stop's tooltip says which of the three it will do. A process in the
  unit Marley itself runs in is still signalled, since stopping that unit would stop Marley.

- **Drag to reorder the rail** (#602, 2026-09-30). A project's or a group's header drags up or down
  among the headers, its rows with it, so a group can sit between projects; a terminal, Browser
  tab or thread row drags up or down among the rows of its kind in its own group and section. A
  card with the name follows the pointer, a line shows where it lands, and a row dropped anywhere
  else goes back. The rail holds still for the whole drag, and a press that barely moves is still
  a click. The order is saved with the window's sidebar state (headers by folders or group id,
  rows by terminal id, page or thread) and comes back after a restart; under the attention order
  it breaks ties within each class. Move Project Up and Down now move a project in the same order,
  past a group as well as a project, rather than in Zed's project list.

- **Projectless groups come back after a restart** (#601, 2026-09-30). #600's groups return when
  Marley starts again, as the app menu starts it: with their names, in their order, their
  terminals in the folders they were in and their Browser tabs on their pages, each tab on its
  group's own browser. Every group's record is kept in Zed's key-value store (`marley-groups`),
  read at startup so a group's tabs find their browser while they restore, and each window keeps
  its groups' workspace ids in its saved sidebar state and reopens them. A group whose workspace
  Zed no longer has is dropped, with a line in the log.

- **Groups with no folder, from the rail's right-click menu** (#600, 2026-09-30). A right-click on
  the rail's empty space (under the last row, or beside PROJECTS) opens a menu: New Group… makes a
  named group with no folder, listed after the window's projects with a group icon, its chevron
  and its `+`; its terminals and agent CLIs start in the home folder, and its Browser tabs use a
  Chromium and a profile of the group's own. New Terminal, New Browser Tab and the agent CLIs in
  that menu open in a group named Home, made the first time one is needed. A group's header menu
  has Rename Group… and Remove Group, which closes its items after Zed's prompts and #550's
  question and stops its browser. Opening a project while a group is shown keeps the group: Zed
  would otherwise replace a shown workspace with no folder. Groups last for the session; #601
  brings them back after a restart.

- **The Marley guide, one click away** (#599, 2026-09-30). A `?` in the title bar, before Sign
  In, and `marley: open guide` open Marley's user guide: one page shipped with Marley, with every
  feature as a short summary of what it is and then how to use it (the steps, the keys, the
  settings it reads and the tickets that shipped it), a contents column that filters as you type,
  and light and dark themes. It opens in a Browser tab of the project on screen, and a second
  click brings the same tab forward; with no project open, in the system browser. Marley writes
  the page to `guide/index.html` in its data folder when it opens it, so an updated Marley shows
  its own guide.

- **An agent's ssh asks for a key's passphrase in Marley** (#596, 2026-09-29). The terminals
  Marley opens for an agent CLI in a local project point `SSH_ASKPASS` at a helper of their own,
  with `SSH_ASKPASS_REQUIRE=force`. When ssh there needs a passphrase, Marley brings that
  terminal to the front and shows Zed's password dialog over it, headed `ssh for <agent> in
  <project>`, with the prompt ssh gave. The typed passphrase goes back to ssh alone, never to the
  terminal, a log or the agent. Escape gives ssh an empty answer, so the command fails at once
  instead of waiting. The helper, its socket and its folder go when the terminal closes. Git's own
  credential prompts still fail at once in those terminals (`GIT_ASKPASS` is set empty so git
  does not borrow ssh's helper), and every other terminal's ssh asks on the terminal as before.

- **A second launch brings Marley forward as its launcher's click** (#545, 2026-09-29). When Marley
  already runs, a second launch hands over the activation token its launcher gave it
  (`XDG_ACTIVATION_TOKEN`) with its paths, and the running Marley brings its window forward with
  it. Compositors that check tokens (sway, KWin, GNOME) now act on the hand-off instead of
  refusing a token Marley asked for itself.

- **gate:22, process spawns only in the listed adapters** (#541, 2026-09-29). The gate finds every
  call that starts a process in the Marley crates with a semgrep rule and allows them only in the
  files `.config/spawn-sites.txt` lists: the PTY, Chromium's unit and relay, and the workbench's
  `process.rs`. It fails on a spawn elsewhere, a listed file with none, a count other than its
  pin, a scan below its floor, and a planted file of every spawn form the rule does not find
  exactly.

- **Remote terminals that survive a dropped link** (#543, 2026-09-29). `marley: open remote
  terminal` lists the SSH hosts saved in your settings (`ssh_connections`) and opens a terminal on
  one, whose shell runs in a tmux session of Marley's own on the host. When the link drops, the
  shell and any agent in it keep running; the tab shows the terminal ended, and `terminal: rerun
  task` attaches the same session again, with what it printed meanwhile. Claude Code's events from
  inside the session reach the rail and the desktop as a local Claude Code's do, once Marley's
  plugin is updated (1.6.0) on the host. The host needs tmux 3.3 or later.

- **The rail lists what needs you first** (#542, 2026-09-29). Projects, and the rows under each,
  are ordered by attention: an agent waiting on you or a failed run you have not seen, then a
  finished run you have not seen, then working, then an agent that stopped reporting (`no update
  in N m`), then idle; ties keep the window's order. A collapsed project's header counts its
  agents by state (`1 waiting, 2 working`). While the pointer is over the rail the order holds
  still, so a row never moves under it. `marley.rail_order: "window"` (Settings, Marley, Layout)
  brings back the window's own order.

- **A project's own icon on its rail header** (#564, 2026-09-29). The rail draws the favicon or
  logo a repository holds, `favicon.png`, `public/favicon.svg`, `src-tauri/icons/icon.png` and
  the other names web and desktop projects use, or the icon its `index.html` declares, at 16 px
  before the project's name, and follows it when the file appears, changes or goes. Only the
  project's own files are read; nothing is fetched, and a project without an icon looks as before.

- **A note when Marley's key takes one from a terminal program** (#563, 2026-09-29). The first
  time Ctrl-G opens Rich Input, Ctrl-Up or Ctrl-Down moves between blocks, Ctrl-Alt-N opens New
  Agent from a terminal, or Ctrl-I takes a terminal over, a toast names the key as you have it
  bound, what it did, and Open Keymap, since your keymap wins over Marley's and gives the key
  back. Each note shows once per data directory; a key Marley lets through says nothing.

- **Codex and OpenCode notifications in a click** (#552, 2026-09-29). Under a terminal running
  Codex, the agent bar offers Turn on Codex notifications, which sets `notifications`,
  `notification_condition = "always"` and `notification_method = "osc9"` under `[tui]` in Codex's
  `config.toml` and keeps the rest of the file. Under OpenCode it offers Connect OpenCode to
  Marley, which writes Marley's plugin to OpenCode's plugin folder, and Update when an older one
  is there. Both agents then post a desktop notification when a turn ends, they need you or they
  fail, from a terminal you are not looking at.

- **English at the prompt** (#557, 2026-09-29). A line typed at a terminal's prompt that reads
  as a request, such as `what is using port 3000` or `find all the large files in this repo`,
  shows a dimmed hint after the cursor, and Ctrl+Shift+Enter hands it to the agent instead of the
  shell: the Claude Code already running in the window, one you pick when several run, or a new
  Claude Code with the line as its first prompt. A command the shell could not find (exit 127)
  on such a line offers Ask the agent on its block. Marley's own rules decide, from the programs
  on your search path and your shell's builtins; nothing leaves the machine until you ask. The
  setting English at the Prompt (`marley.english_hint`) turns the hint and the button off.

### Changed

- **The Rusty home page** (#679, 2026-10-07). The rail's row of Rusty's screens becomes one Rusty
  button (a placeholder icon until Rusty has its own) before PROJECTS. It opens Rusty's home page,
  always the first tab of the Rusty group: a card with every screen's button (Brain, Today, Graph,
  Tasks, Decisions, Memory, Skills, Secrets), then your recent pages, the decisions whose follow-up
  is due, and your open tasks as a table, each row opening what it names. `rusty: open home` opens
  it too.

- **The Brain tab** (#678, 2026-10-07). Rusty's vault no longer takes over the rail. The rail's
  header shows Brain as the first of Rusty's screens, and Brain (or Ctrl+Alt+V) opens a Brain tab
  in the Rusty group: the search, the favourites and the vault's tree on its left, the page you pick
  on its right, with the page's back, forward, Edit, outline and star. Today opens it on today's
  note. A page's menu adds Open in New Tab for a tab of its own, and New Page sits after the
  search field. The rail's Projects/Brain switch is gone. Also: a window's Rusty group is always
  called Rusty; a second window's was "Rusty 2".

- **The Rich Input button hides it too** (#677, 2026-10-07). While an agent's Rich Input editor is
  open, the agent bar's pencil shows pressed and reads Hide Rich Input; a click closes the editor
  and keeps the draft, as Escape does. Closed, it opens the editor as before.

- **Home takes what belongs to no project** (#676, 2026-10-07). A fleet agent's tab, System One
  calls and a harness session's tab now open in the window's **Home** group, the one a new
  terminal from the rail's empty space goes to, whatever project the window shows, and the window
  shows Home; the first one makes it. A project click goes back with the tab kept under Home.

- **Rusty's screens open in a Rusty group** (#675, 2026-10-07). Today, Graph, Tasks, Decisions,
  Memory, Skills, Secrets and every brain page now open in the window's **Rusty** group in the
  rail, whatever project the window shows, and the window shows the group; the first one makes
  it. Its tabs are its rows, and a project click goes back to your work with Rusty's tabs kept. The
  group has a placeholder icon, no Rename or Remove, and is left out while Rusty is off, when a
  window showing it goes back to its first project. In the Zed layout, screens open where they
  did.

- **Every tab in the rail** (#674, 2026-10-07). Each tab open in the middle now has a row under
  its project, so the rail and the tab bar list the same things: a project search, a diff, settings
  or one of Marley's tabs after the project's Browser tabs, and its open files under a **Files**
  row with their count, above its ports. A file's row shows its folder; a dot marks changes not
  saved; the × at a row's end closes the tab. Click Files to fold the files away; while folded, it
  is the row highlighted for a file in front.

- **Containers moved to a panel on the right** (#673, 2026-10-07). The ports of this machine's
  containers that no project holds are no longer listed at the foot of the rail. A box button in
  the status bar opens the Containers panel in the right dock: the count, and a row per port with
  Open in a Browser Tab (or a double-click), Copy URL and Stop the Container. A container whose
  Compose folder is in a project still shows under that project in the rail. The setting, now
  Containers Panel on the Settings window's Marley page (`marley.rail_containers`), hides the button
  and closes the panel.

- **Rusty's screens in the rail's header** (#672, 2026-10-07). While Rusty is connected, Today,
  Graph, Tasks, Decisions, Memory, Skills and Secrets sit in the rail's top row beside Projects and
  Brain, one click from either view; the Brain view no longer has a row of its own for them. When
  the rail is too narrow for all seven, the last ones wait under `…`. The filter row and the brain
  search row under the header are now the tab bar's height, and the header's line and theirs meet
  the title bar's and the tab bar's exactly.

- **The rail keeps the window's order** (#671, 2026-10-07). `marley.rail_order` now defaults to
  `"window"`: projects and the rows under them stay where the window and your drags put them, so
  typing into an agent no longer lifts its project to the top and drops it back when the output
  stops. Rail Order on the Marley page (Layout), or `"rail_order": "attention"`, brings back #542's
  sort by what needs you.

- **Rusty off leaves no trace** (#661, 2026-10-06). With Rusty's switch off, the command palette
  lists no `rusty:` command and no `marley: toggle brain view`, and the Marley settings page's
  Rusty section shows the switch alone. Turning it on brings the commands and the section's
  other settings back at once, the Settings window included. A key bound to a Rusty command
  still says Rusty is off.

- **System One's log is now System One calls** (#659, 2026-10-04). The tab that lists System
  One's calls, its command (`marley: open system one calls`), the Marley settings page's link
  (Open System One Calls) and every line that named it say System One calls, so Decisions is
  Rusty's alone. A key bound to `marley::OpenDecisions` still opens it.

- **The Graph tab starts on your project's page** (#655). With no brain page in front, `rusty:
  open graph` and `rusty: open local graph` show the neighbourhood of the window's project page,
  marked "(project)", where they showed the vault or asked to open a page first.

- **An unresolved link makes its page** (#654). In a brain page's tab, clicking a muted link to a
  page that does not exist yet makes the page at that path, its folders too, and shows it, where
  it used to say the page did not exist.

- **`marley.rusty_tools` is now `marley.rusty.agent_tools`** (#643, 2026-10-04). Rusty Tools for
  Agents moved from the Agents section to the Rusty section, and offers Rusty's tools to Zed's
  agents only while Rusty itself is on. An old `"rusty_tools": true` is carried at each load as
  Rusty and its tools on, which also starts Marley's own connection to Rusty; `false` carries as
  the tools off. Zed's banner offers to write the change into `settings.json`.

- **Dictation and Rusty's tools wait to be turned on** (#642, 2026-10-04). The agent bar's
  microphone no longer appears wherever Voxtype is installed, and Zed's agents no longer get
  Rusty's MCP server wherever `rusty-mcp` is: both are off until you turn them on. The Marley
  settings page has a new Voice section whose Voice toggle (`marley.voice.enabled`) brings the
  microphone and `marley: toggle dictation` back; Rusty Tools for Agents in the Agents section
  (`marley.rusty_tools`) offers the `rusty` server again. Off, Marley shows no microphone, starts
  no `voxtype`, and `marley: toggle dictation` says dictation is off and where to turn it on.
  Either switch takes effect without a restart. If you used either before, turn it on once.

- **A mutation run over the pure cores, reported** (#636, 2026-10-01). `cargo-mutants` over the
  nine Marley crates with no gpui: 2,658 mutants, 40.2% of the viable ones killed. The crates
  built with unit tests kill every mutant; the modules built since #483, proven by e2e scenarios
  alone, keep most of theirs. No survivor read as a bug. The findings are in
  `docs/planning/design-notes/mutation-run-2026-10.md`, and whether to write tests for the
  survivors is an intake for Chad.

- **One click on a port row marks it; a double-click opens it** (#604, 2026-09-30). A click on a
  port row's body used to open its URL in a Browser tab, so a look at a server's row started a
  browser. Now one click puts the rail's keyboard row on it, with the focus in the rail; a
  double-click, Enter or the row's Open button opens the URL. The row's tooltip says so. Other
  rows still open on one click.

- **New Agent in Worktree sits under New Agent Thread** (#598, 2026-09-30). In a project's `+` in
  the rail, New Agent in Worktree now follows New Agent Thread directly, above the Agent CLIs
  header, so the two submenus that start an agent sit together and the header holds only the
  CLIs. The entry, its submenu and when it shows are unchanged. The e2e scenarios that reach it by
  position (510, 585, 587, 589) step to it with three Downs instead of seven.

- **The fleet contract's verb values and capabilities** (#597, 2026-09-29). `marley_fleet` now
  holds `SurfaceAck`, which Marley's MCP server defined until now, and the values an accepted read
  and answer return, `ReadReceipt` and `AnswerReceipt`, as rustal-harness's `rh mcp` returns them.
  A `Session` carries its declared `capabilities` (`mode`, `model`, `effort`, name to value) and a
  `SendRequest` what the work `requires` in the same shape; both are left out of the JSON when
  empty, so no envelope or send in flight changes. `fleet_snapshot`'s schema lists the new field.

- **The workbench's programs start in one module** (#541, 2026-09-29). Every program
  `marley_workbench` runs (`claude plugin`, `voxtype`, `git` for turns, worktrees and the rail's
  drift and changed lines, `gh`) now starts in `src/process.rs`, through `output` for a finished
  program and `follow` for voxtype's status stream, with the same programs, arguments and
  messages as before. #560's scenario allows the read-only `rev-parse` and `diff` that #531's
  changed lines added.

- **The fleet contract's retry ids** (#533, 2026-09-29). `marley_fleet`'s `SendRequest` and
  `OpenRequest` take an optional retry id (`delivery`, `request`), left out of the JSON when
  absent, and the crate has the values an accepted send and open return, as rustal-harness's
  `rh mcp` returns them. The verbs are named by their tool names (`session_send`), and the plan's
  prong 2 says where the harness stands and that it is embedded in Marley and also runs
  standalone.

### Fixed

- **Every harness refusal reaches the agent by its code** (#693, 2026-10-07). `seat_add` passed
  the harness's codes through only from a list of twelve, so `seat_surface` and the runtime's own
  codes were lost. Marley now takes any `rh: CODE: reason`'s code as the harness gives it, and
  calls an exit 2 its own usage error.

- **Rusty's PIN, unlocks and secrets stay out of Zed's log** (#667, 2026-10-06). With trace
  logging on, Zed's MCP client wrote every message whole to its log, so a PIN, an unlock token or
  a revealed secret would have landed there. Rusty's connections are now logged by message size
  only; other MCP servers log as before.

- **A long line no longer widens a brain page's tab** (#663, 2026-10-06). A line that could not
  wrap, such as a path in code, made the Page tab's body wider than the tab, pushing the outline
  column and the properties' remove buttons out of sight. The body now keeps the tab's width.

- **A cleared terminal shows no old blocks, and an agent's terminal starts with its own block**
  (#639, 2026-10-01). After Clear (Ctrl-Shift-L), every old block's header stayed drawn over the
  cleared screen, and the next command's output read as an old block's. A terminal Marley opened
  for an agent CLI, or a resumed Claude Code session, showed Zed's startup check
  (`printf … __zed_init_command_ready_ …`) as its first block, with the agent's output under it.
  Blocks now follow the clear, and the startup check opens no block.

- **A drag in a terminal at its prompt selects again** (#631, 2026-10-01). Since the prompt editor
  docked at every prompt (#627), it took the focus as soon as you pressed in the terminal, and the
  terminal dropped the drag and the release that followed, so dragging over output selected
  nothing. A press made in the terminal now keeps its drag and its release.

- **Ctrl-Shift-W in the prompt editor closes the terminal, and a command leaves no dirty mark**
  (#635, 2026-10-01). Since the prompt editor opens at every prompt (#627), Ctrl-Shift-W there
  closed the whole window rather than the terminal, and every command sent from it rang the
  shell's bell, so the tab showed the dirty dot, closing the terminal asked to save all changes,
  and a quit waited on that dialog. The key now closes the terminal, as it does in the terminal,
  and sending a command rings no bell and clears the mark.

- **The golden regression set runs green** (#635, 2026-10-01). All 53 scenarios pass again:
  seven followed the UI their tickets changed, one turns the prompt editor off to check #484's
  suggestions where they still show (their place in the editor is #637), and the two bugs above
  were fixed. `script/e2e.sh` gains `profile_setting`.

- **A terminal counts as focused while its prompt editor has the keys** (#634, 2026-10-01). Since
  the prompt editor opens at every prompt (#627), a terminal at its prompt read as unfocused:
  ctrl-` from it refocused the terminal instead of going back to the code, the block keys found no
  terminal, the rail did not select its row, and a terminal's notification, phone push or note
  went out while you were typing in it. Each now asks whether the terminal or its editor has the
  keys.

- **The test suites run green** (#634, 2026-10-01). No test had run since 2026-09-23. The Marley
  crates' tests and Marley's tests in Zed's crates (724) and the suites of the 18 other Zed crates
  Marley changed (3,499) pass: tests updated to what tickets changed on purpose, each naming its
  ticket; the workbench's blocking reads moved onto gpui's executor; the tests kept off the
  machine's ports, ssh's passphrase socket and the context servers Marley offers Zed's agents;
  and the `zed` tests given a scratch data folder.

- **The tests no longer write your Marley data folder** (#475, 2026-10-01). The tests of
  `terminal`, `terminal_view` and `marley_workbench` start real shells, and starting one installs
  Marley's shell scripts in the data folder: until now `~/.local/share/marley`, so a test run
  could rewrite the scripts your own terminals load, and the workbench's tests opened a prompts
  database there too. Each of those test binaries now sets its data folder to `marley-test-data`
  in the build folder before its first test.

- **A Browser tab whose stream never drew is asked again** (#578, 2026-09-30). A restored
  Browser tab once drew nothing although its page had loaded (2 of 6 runs of #576's scenario on
  2026-09-26; none in 26 runs since). Chromium sends one frame when a screencast starts and none
  after on a still page, so a lost first frame (a start that failed, a frame that did not decode
  or came for no page shown) left the tab blank for good. The hub now starts a viewed page's
  stream again when it has drawn nothing two seconds after it started, up to three times, and
  logs the stream's starts, stops and frames at debug level.

- **A settings edit reloads after Marley has written the file** (#612, 2026-09-30). On Linux,
  once Marley or Zed had saved `settings.json` itself (the layout switch, the theme picker), an
  edit to the file outside Marley did not apply until a restart: the writers replace the file,
  and the watch was on the old one. The watcher of every config file (`settings.json`,
  `keymap.json`, a project's settings, `.editorconfig`) also watches its folder now, and reloads
  on that file's own events.

- **A port row's lines at the default rail width, and a closed header's tooltip** (#618,
  2026-09-30). A port row's Open, Copy and Stop no longer keep their room while hidden: they show
  over the row's end on hover, so the URL and the unit's name have the row's width. A local
  server's URL reads `127.0.0.1:<port>/` without `http://`, a URL still too long loses its
  middle, and a unit's name loses its start, so its end stays. The row's tooltip begins with the
  whole URL. A closed project's "Not open. Click to open it." no longer stays over the menu a
  right-click on its header opens.

- **A launch's first terminals at the last session's size** (#486, 2026-09-29). Marley keeps the
  size its terminals last had when it quits and opens the next launch's first terminals at it, so a
  multi-line prompt wider than 100 columns (starship's, for one) is drawn right from the first
  prompt instead of misdrawn until the next.

- **The e2e runner after its compositor exits** (#588, 2026-09-29). When a run's headless sway
  exits before the scenario ends, the runner says so with sway's exit status and fails the run,
  still copies Marley's log beside the shots, and stops the run's Marley, pointer helper, key
  holder and Browser units, removing the dead sway's sockets; sway logs verbosely. The runner now
  refuses a scenario that names no folder to open, and `just shot` opens a scratch repository
  unless `OPEN` names one: a run that opened nothing restored the profile copy's last session,
  the user's own projects and Agent Panel threads.

- **A rerun task's notifications** (#543, 2026-09-29). A task run again in its terminal gets a
  new terminal behind the same tab, and its OSC 9 and 777 notifications and Claude Code's events
  went unheard; Marley now follows the tab to its new terminal.

- **Inline Assist's prompt in a terminal shows** (#557). Ctrl+Enter's prompt went under the
  terminal's bottom edge, since short output is drawn down onto it; the terminal now keeps the
  rows under the cursor while the prompt is open. The prompt's run button (▶) runs the command.

- **An agent's commands can stay out of your shell history** (#553, 2026-09-29). The Marley
  setting Agent Commands in History (`marley.agent_commands_in_history`, on by default) decides
  whether the commands an agent runs at your prompt with `terminal_run` enter bash's and zsh's
  history and Marley's suggestions. Off, terminals opened after the change type them with a
  leading space that Marley's shell integration keeps out of the history list and the history
  file, while the block, its mark and its output stay as before.

- **Agents run commands at your prompt, as blocks** (#556, 2026-09-29). An agent connected to
  Marley's MCP server can run a command in a terminal of yours with `terminal_run`, which types it
  at the shell's prompt as Rerun does and answers with the block's exit code, duration and output.
  Each block an agent ran carries a mark before its pill. A command on the denylist (Warp's
  defaults: shells, `curl`, `ssh`, `rm` and more) waits on a card under the terminal with Run and
  Refuse, which Enter and Escape answer; one on the allowlist (`cat`, `ls`, `grep` and the like)
  runs at once; Agent Commands Outside Lists decides the rest, running them by default since the
  agent's own permission prompt already asked. Nothing is typed while a program runs, while you
  have typed at the prompt, or after Ctrl-I takes the terminal over.

- **A dev server's error while it keeps running** (#572, 2026-09-29). A command that prints a
  failure and is still running five seconds later, such as a dev server whose build broke, marks
  its terminal's rail row with a red × and the line, and posts one desktop notification,
  `repo: npm run dev printed an error`, when the terminal is not in front; a second says it
  recovered once it builds again. Failures and recoveries are known by how compilers, test
  runners, dev servers and runtimes print them; a line that only mentions an error's word can go
  to System One, whose reading only logs (Shadow), marks with a `?` and no notification
  (Suggest), or counts as a failure (Act). Agent CLIs' and SSH terminals are left out, and a
  command that ends inside the five seconds is told by its exit instead. The setting is System
  One's Running Error (`marley.system_one.uses.running_error`), off by default.

- **A long command's end, from a terminal you are not looking at** (#551, 2026-09-29). A command
  that runs 30 seconds or more and ends in a terminal not in front posts one desktop notification
  titled with the command, `done in 45 s` or `exit 1 after 4 m 12 s`, and marks the terminal's
  rail row with the unread dot. A command that stops to read a password (`sudo`, `ssh`, `read
  -s`) posts `waiting for a password` once. The rail's row for a plain terminal gains a line with
  its last command and its state: `running`, `waiting for a password`, `done · 7 s`, or `exit 1 ·
  3 s` in red; the rail's filter finds a terminal by that command too. Agent CLIs' terminals are
  left out, since their banners already say what the agent did. The threshold is the Marley
  setting Long Command Seconds (`marley.long_command_seconds`, 30; 0 turns it off).

- **Notifications that say what happened** (#538, 2026-09-29). Claude Code's banners now read
  `repo: Claude finished` over the turn's last message, `repo: Claude needs input` over
  `Using Bash: ls -la` or the question it asks, and `repo: Claude failed` over the failure's kind,
  cut to 180 characters. An event in a terminal you are not looking at marks its rail row with a
  dot until you look at it, even when no banner shows. A burst from one project within five
  seconds shows one banner, and a session that starts, resumes or clears shows none. The plugin
  no longer sends its fixed sentences beside the events (version 1.5.0), so each event shows
  once.

- **Git asks no agent for a password** (#537, 2026-09-29). The terminals Marley opens for an
  agent CLI, from the rail's +, the New Agent picker, a worktree agent or a launch config, start
  with `GIT_TERMINAL_PROMPT=0` and `GCM_INTERACTIVE=never`. A git command there that needs
  credentials no helper holds fails at once with git's own message, instead of waiting on a
  prompt the agent cannot answer; stored credentials keep working. Other terminals keep git's
  prompts.

- **Copy and paste that know an agent runs** (#536, 2026-09-29). While Claude Code, Codex or
  another agent CLI runs in a terminal, a copy of its reply drops the indent every line shares,
  so it pastes flush left elsewhere. A paste of several lines always goes in as one bracketed
  paste, so no line of it is sent early, even when the agent never asked for bracketed paste. A
  PNG, JPEG, GIF or WebP dropped on the terminal, or attached, goes in as its raw path inside a
  paste of its own, the form in which the agent attaches an image. In a plain shell all three
  behave as in Zed.

- **Shell commands in the Markdown preview go to the terminal** (#530, 2026-09-29). A code block
  in Zed's Markdown preview whose fence names `sh`, `shell`, `bash`, `zsh` or `fish`, or no
  language, shows Insert in Terminal beside Copy, as Warp's Markdown viewer does. A click clears
  the prompt's line in the terminal used last and puts the command there without running it,
  then brings that terminal forward so Enter runs it. Several lines go in as one bracketed paste.
  When a program runs there, the shell has bracketed paste off, or no terminal was used yet,
  nothing is typed and a toast says why.

- **A long block's command stays in view** (#529, 2026-09-29). Scrolled back into a block whose
  first row is above the view, the terminal pins the block's command over its top row, with its
  check, `exit N` or `running`, as Warp's sticky command header does. A click on it scrolls to the
  block's start. It never shows at the live screen, where it would cover new output, nor on a
  full-screen program's screen. Sticky Command Header, in the Marley page's Terminal section,
  turns it off (`marley.sticky_command_header`).

- **Bookmarks on blocks, and find within a block** (#559, 2026-09-29). `Ctrl+Shift+B` in a
  terminal bookmarks the selected block, or the newest one in view, for the session; a hovered
  block's Bookmark button and its right-click menu bookmark that block. A bookmarked block shows
  a bookmark before its exit mark and a tick at the terminal's right edge at its place in the
  scrollback. `Alt+Up` and `Alt+Down` scroll to the bookmark before or after the view, as Warp's
  keys do, and reach the program in a terminal with no bookmark. A hovered block's Find button,
  Find in Block in its menu, and `Ctrl+Shift+F` on a selected block open Zed's search bar held to
  that block: the count and Enter walk its matches alone, and the block is outlined. Escape ends
  it, and `Ctrl+Shift+F` with no block selected searches the whole terminal as before. Save as
  Workflow's button now shows a book.

- **Save a block's command as a workflow** (#558, 2026-09-29). A hovered block's Save as
  Workflow button, or `marley: save as workflow` on the selected or newest block, opens an editor
  with the command and a name. Numbers, URLs, the block's branch and paths that exist are already
  `{{name}}` parameters, each with the typed value as its default and a description to fill in.
  Save appends the workflow to the project's `.zed/tasks.json`, or the global `tasks.json`, as a
  Zed task. What the file held before, its comments included, stays as it was. The task shows in
  Zed's task picker at once. Running it asks for each parameter, prefilled with the last value
  used this session, then runs the filled command as a task; Escape runs nothing. A workflow with
  no parameters runs at once.

- **Filter a block's output** (#528, 2026-09-29). `Alt+Shift+F` in a terminal, or the Filter
  button on a hovered block, opens a panel over the terminal listing only the block's lines that
  hold some text, as Warp's block filter does. Toggles match the case, read the query as a regular
  expression, keep the lines that do not match instead, and keep a number of lines around each
  match, with `--` between groups. Matches are highlighted, and an invalid expression says so and
  keeps the last list. The terminal's own lines are never touched, so Escape shows the block as it
  was; the panel comes back with the same query, and a running block's list keeps up with its
  output. With a block selected (`Ctrl+Up`), `Alt+Shift+F` filters that one.

- **Send a block to an agent** (#555, 2026-09-29). Send to Agent, in a block's right-click menu,
  and `Ctrl+Shift+Enter` on a selected block, type the block at the prompt of the CLI agent
  running in another terminal, without pressing Enter. A block with long output goes as a
  reference the agent reads with Marley's `terminal_read` tool; a short one goes as its Markdown.
  Secrets are hidden in both, as in everything else agents read. With several agents a picker
  asks which; the block's own terminal is never offered, and an agent that waits on a permission
  or a question gets nothing. Under the newest block, when it failed and an agent runs in another
  terminal, an Ask the agent button does the same in one click; for a long failure it sits on the
  block's last line, just above the prompt.

- **Select a block, and a menu of what to do with it** (#554, 2026-09-29). `Ctrl+Up` in a
  terminal outlines its newest block; `Up` and `Down` move the outline from block to block,
  `Ctrl+Down` past the last ends it, and so do Escape and the first key typed to the shell. A
  right-click on a block selects it and adds a Block section to the terminal's menu: Copy
  Command, Copy Output, Copy Both, Copy as Markdown (the command and its output in a code block,
  then its exit code, how long it ran and its folder and branch), Reinput and Reinput with sudo,
  which put the command back at the prompt without running it. `Ctrl+Shift+I` reinputs the
  selected block. Reinput is offered as Rerun is: only for a command the shell itself reported,
  while that shell waits at its prompt.

- **Blocks over ssh** (#526, 2026-09-29). `ssh host` from Marley's bash or zsh now starts the host's
  bash or zsh with Marley's integration, so each command typed there is a block with its exit
  code, as on this machine. The integration travels in the ssh command, one line that any login
  shell reads alike, and the temporary folder it writes on the host is gone as soon as the shell
  has read it; nothing is installed there. The `ssh` block ends when the host's shell starts.
  Rerun is offered for a block only while the shell that ran it waits at its prompt, so a host's
  command never reruns here or the other way round, and autosuggestions draw on that shell's
  commands. `ssh` with a remote command, a flag that makes the session non-interactive, no
  terminal, a host whose config sets `RemoteCommand` or `Tag marley-plain`, and `command ssh`
  run as plain ssh, as does a host whose shell is neither bash nor zsh. `terminal_blocks` names
  each block's `host`, and the tab's title reads `ssh far`, without the command Marley added. The terminal's own nonce never leaves the machine: the connection gets a
  nonce of its own, announced in a frame the local shell signs, and every frame now carries its
  shell's nonce.

- **An agent reads and types into a running program** (#525, 2026-09-29). Two tools on Marley's
  MCP server: `terminal_screen` reads what a terminal's screen shows now (its rows through the
  secret redaction, the cursor, the alternate screen, the program in the foreground, and who
  controls the terminal), and `terminal_type` types text, named keys (`escape`, `ctrl-c`, `up`)
  and Enter into the program running there: psql, a debugger, a REPL. It never types at the
  shell's prompt or into another agent CLI, and it needs the `generation` `terminal_screen` gave,
  so a write meant for one program never lands in the next. Marley asks before an agent's first
  write to each program, with a card under the terminal (Allow, Deny) and a toast with Show;
  `marley.agent_terminal_writes` (the Marley page's Agents section) makes it ask every write or
  never. Once an agent has typed, a bar under the terminal shows what it typed, and Take Over,
  or Ctrl-I, stops its writes until Hand Back; Ctrl-I reaches the program as before otherwise.

- **A project's launch configs** (#527, 2026-09-29). A `.zed/marley.json` at a project's root
  names launch configs under `launch`, each a list of `items`: `"terminal": "<command>"` (a center
  terminal whose shell gets the command once it is ready; an empty string is a plain shell),
  `"agent": "claude"` (or `codex`, `gemini`, `opencode`, started as the rail's Agent CLIs entries
  start it), or `"browser": "<http or https URL>"` (a Browser tab; a URL on this machine waits up to
  30 seconds for its port, so a dev server an earlier item starts can come up), each with an
  optional `title`, `cwd` inside the project, `split` (`right` or `down`, off the previous item's
  pane) and `focus`. The rail's + lists the configs under Launch; one click shows the config's
  text, one line an item, with Run and Cancel, and Run opens the items in order. Marley keeps the
  SHA-256 of the approved text in Zed's key-value store, so the same config opens at once next
  time and a changed one asks again, saying it changed. A file that does not parse shows one
  disabled entry naming it and its first error; an edit shows in the next menu.

- **A project's changed lines and pull request on its rail row** (#531, 2026-09-29). A project
  whose folder is a repository's main checkout shows, at the right of its row, the lines added and
  removed since its branch left its base (`+12 ‒3`, uncommitted edits to tracked files included),
  and when the branch has a pull request on GitHub, a chip with the pull request icon and its
  number, green while open, gray as a draft, in the accent color once merged and red when closed;
  its tooltip gives the state, the title and the link. The base is the pull request's base,
  otherwise the repository's default branch (its `origin` copy when there is one, so a project on
  its default branch counts what it has not pushed; a branch with nothing changed shows none). The
  counts follow each save, the branch and
  its commits within a second or two; `gh pr list` is asked when the branch or its `HEAD` moves
  and every two minutes while the window is active. Without `gh`, logged out, or with a remote
  that is not GitHub, the row shows the counts alone and the log one line. Only repositories Zed
  trusts are read.

- **Review notes to a terminal agent** (#522, 2026-09-29). Zed's project and branch diffs now offer
  Add Review in the gutter of a changed line (its feature flag is on for everyone), and their
  Send Review to Agent (N) works: it opens a picker of the agent terminals whose folder holds
  every noted file, each marked ready, working, asking for permission, or no idle signal, with
  Copy notes at the end. Picking a ready Claude Code (idle at its prompt, as its plugin's events
  say) pastes the notes as one prompt, `File:`, `Line:` or `Lines:`, and `User comment: "…"` per
  note, presses Enter and shows that terminal. The notes stay in the diff marked Sent, and the
  button counts only the ones not sent. An agent that is working, waits on a permission or a
  question, or reports nothing is left alone with a toast; Copy notes puts the same text on the
  clipboard and marks nothing sent.

- **A worktree's teardown task before Remove** (#591, 2026-09-29). A task whose `hooks` hold
  `remove_worktree` (in the worktree's `.zed/tasks.json`, or your global tasks) now runs when you
  confirm Remove on the worktree's row, before the worktree goes, so the dev database, containers
  or server it started can be stopped. Each such task runs in its own terminal with the
  worktree's variables (`ZED_WORKTREE_ROOT`, `ZED_MAIN_GIT_WORKTREE`, `MARLEY_ROOT_PATH`,
  `MARLEY_WORKTREE_PATH`), one after another, and Marley waits up to two minutes for each. One
  that fails or runs longer stops Remove with a question naming it; Cancel keeps the worktree
  and the task's terminal open, Remove Anyway goes on. `remove_worktree` is a new value of Zed's
  task `hooks` field, beside `create_worktree`.

- **Remove on a worktree's row** (#589, 2026-09-29). Right-click a worktree in the rail and choose
  Remove… once its agent's work is done. Marley always asks first; when git counts changes not
  committed (untracked files included) the question names how many and the button reads Remove
  Anyway. Then the worktree's own workspace closes, which stops its terminals (Zed asks about
  unsaved files, and a refusal stops Remove), and Zed's own archive code removes the worktree: it
  checks that Zed made it and releases it from every open project. The branch goes when
  `git branch -d` agrees, or when its commits are shown to be in its recorded base, `origin/HEAD`
  or the main checkout's `HEAD` (an ancestor, a merge that changes nothing, as after a squash
  merge, or `git cherry` finding every change); otherwise it stays, and the toast says why. In a
  repository the Rustal workflow merges, Remove waits until the branch is merged.

- **A port of its own for each worktree agent's worktree** (#590, 2026-09-29). New Agent in
  Worktree gives each worktree it makes a slot, the lowest from 1 that no other worktree of the
  repository holds (kept as `git config branch.<branch>.marleySlot`; a removed worktree's slot is
  taken again), and every terminal and task of that worktree's project starts with
  `MARLEY_PORT_OFFSET`, the slot times ten, and `PORT`, 3000 plus the offset: 3010, 3020 and so on,
  ten ports each, as Conductor gives each workspace ten. A dev server that reads `PORT` in one
  worktree no longer takes the port another's wants. The main checkout's terminals get neither,
  and a `PORT` in your `terminal.env` setting or a task's own `env` wins. Terminals restored at a
  launch, Zed's terminal panel, the debugger's and the agents' terminals get them too. Zed's own
  `create_worktree` tasks start before the slot is written and get none.

- **A worktree agent's worktree gets its environment** (#585, 2026-09-29). New Agent in Worktree
  now copies into the new worktree, before the agent starts, the main checkout's gitignored files
  its `.worktreeinclude` names, read as Claude Code reads the file for its own worktrees:
  `.gitignore` syntax, only files git ignores (so a tracked file never is), and a `**/` pattern
  reaches into a wholly ignored folder only by the folder's own name. Nothing in the worktree is
  overwritten, and an entry that would pass 100 MB or 10,000 files in all is left out whole and
  named in a toast. When the repository's root has a `package.json` and the lockfile of one
  package manager (pnpm, bun, yarn or npm) and no `create_worktree` task, the prompt offers
  `Setup: pnpm install (pnpm-lock.yaml found)`; the box is clear unless you checked it last time
  (kept in `git config marley.worktreeSetup`), and checked, the install runs in the agent's
  terminal and the agent starts once it succeeds. Tasks in a linked worktree, Zed's
  `create_worktree` hooks among them, also get `MARLEY_ROOT_PATH` and `MARLEY_WORKTREE_PATH`.

- **Review and merge a worktree's branch from its row** (#511, 2026-09-29). Right-click a
  worktree's row in the rail. **Review** opens Zed's branch diff of the worktree against the base
  its branch started from, "Changes since main", in the worktree's own workspace, and opens the
  worktree first when it is not open; a worktree Marley did not make compares with the
  repository's default branch. **Merge N commits into main…** asks first, naming the count, the
  base and the main checkout's folder, then makes a merge commit of the branch on its base in the
  main checkout, and never pushes; the toast names the commit. It refuses and changes nothing
  while the main checkout is on another branch, has changes not committed or unsaved in Marley,
  or the worktree has changes not committed, and a merge that stops on a conflict is aborted and
  names the files. Merge is offered only for a branch whose base Marley recorded when it made the
  worktree. The row's second line counts the commits the branch has that its base lacks,
  `agent/ok · 2 ahead of main`, and its drift chip shows only while there are some, so a merged
  branch shows neither. Where the Rustal workflow manages the repository (`git config
  marley.merge workflow`, or a `workflow.toml` with a `[project]` table at its root), the menu
  shows the count and leaves the merge to the workflow. Removing a worktree comes next (#589).

- **A project's local HTML pages open in its Browser tab** (#586, 2026-09-29). A program in a
  project's terminal that opens a local page through `BROWSER`, `cargo doc --open`, a coverage
  report, Python's `webbrowser` with a `file://` URL, now gets it in a Browser tab of that
  project, with the focus, as a local dev server's URL already did (#561); the page's links to
  other local pages load in the tab. `cargo doc --open` hands `BROWSER` a plain path, and a path
  to an `.html` or `.htm` file opens too. A folder, a missing page and any other kind of file
  still go to your system's handler, and `marley.terminal_links` set to `system_browser` sends
  pages there as well. Agents' own navigation stays http and https: `browser_open_url` opens only
  an existing HTML page, never a folder or another file.

- **License files for the Marley crates** (#446, 2026-09-28). Each `crates/marley_*` now holds
  `LICENSE-MIT`, the MIT text under `Copyright (c) 2026 Ignibyte`, and `LICENSE-APACHE`, a link
  to the Apache-2.0 text at the repository's root, as its `license = "MIT OR Apache-2.0"` says.
  Zed's `script/check-licenses` passes over the tree again: it skips the third-party crates under
  `vendor/`, which keep their own license files. The README's Licensing section says who holds
  the Marley crates' copyright.

- **Claude Code's trust question in a new worktree, brought to you** (#587, 2026-09-28). Claude
  Code asks whether to trust a folder for a repository it has not trusted yet, and a worktree
  agent's workspace opens in the background, so the question would wait unseen. For a minute
  after New Agent in Worktree starts Claude Code, Marley reads the agent's screen; while the
  question shows, a notification in every workspace names the worktree and the folder, carries
  the question's warnings (such as a folder that pre-approves tool permissions), and offers
  Trust Folder, which moves Claude Code's focus to "Yes, I trust this folder" and confirms it,
  and Show Terminal. It goes once the question does, however it was answered. The setting
  `marley.claude_code_worktree_trust` (`"ask"`, the default, or `"follow_zed"`, on the Marley
  settings page) lets Marley answer by itself when Zed trusts the worktree's folder, with a note
  that stays until you close it. Marley never writes Claude Code's own record of trusted folders.

- **A worktree's drift on its row** (#560, 2026-09-28). Each worktree row in the rail shows how
  far its branch is behind its base, `2 behind` in a muted chip, and when a merge would stop, how
  many files it would stop on, `1 conflict` in the warning color with the conflict icon. The
  chip's tooltip names the base and its commit, and the files, twenty at most. The base is the
  one #510 records, `branch.<branch>.base`, else the repository's default branch when it is a
  local branch. Marley reads it with three read-only git calls against the local base, `merge-base`,
  `rev-list --count` and `merge-tree --write-tree`, a second after the branch or the base moves,
  off the main thread, while the rail shows, and only for a repository Zed trusts; it fetches
  nothing and changes no ref, index or checkout. A branch up to date with its base has no chip.
  With a git older than 2.38, which has no `merge-tree --write-tree`, the chip shows the commits
  behind alone and Marley logs one line for the repository.

- **Worktree agents** (#510, 2026-09-28). A project's `+` in the rail has New Agent in Worktree,
  with each installed agent CLI under it, for a local project that is a git repository. Its prompt
  names what it will make, `agent/<name> from main`; Enter makes a git worktree through Zed's own
  worktree service, on a new branch `agent/<name>` that starts at the main checkout's branch and
  tracks nothing, writes that base as `branch.agent/<name>.base` in the repository's config, and
  starts the agent in the worktree's workspace with the first prompt on its command line, as one
  argument whatever quotes it holds, and with the project's permission mode (#532). The rail lists
  every linked worktree of a project's repository as a row under it, with its branch and, while
  its workspace is open, its terminals; a click opens one that is not open. Claude Code's own
  `.claude/worktrees/` get no row. The worktree lands where Zed puts its own
  (`git.worktree_directory`); a create that fails says why in Zed's toast and leaves no row.

- **Agent permission modes** (#532, 2026-09-28). Two settings, both off by default, choose what
  Marley starts Claude Code and Codex with from the rail's `+` and the New Agent picker:
  `marley.claude_code_permissions`, `"ask"` or `"bypass"` (`--dangerously-skip-permissions`),
  and `marley.codex_permissions`, `"ask"` or `"full_access"` (`--sandbox danger-full-access
  --ask-for-approval never`). `marley.agent_permissions_by_project` sets either for a project by
  its folder, the longest folder winning; only your own settings can set these, never a
  repository's `.zed/settings.json`. Whenever an agent runs without its prompts, however it was
  started, its rail row carries a `bypass` or `full access` chip in the warning color whose
  tooltip says where Marley read it: for Claude Code with Marley's plugin, the permission mode
  its events report, so a bypass its own settings chose shows and one left with Shift+Tab goes;
  otherwise the arguments it was started with. The Marley page's Agents section has both
  defaults.

- **Per-turn diffs for Claude Code in a terminal** (#509, 2026-09-28). Each turn of a terminal's
  Claude Code that changes the tree, from your prompt to its stop, is kept as a commit of its
  own: Marley takes a checkpoint of the repository when the prompt comes and another when the
  turn ends, and a turn whose tree changed becomes a commit of the end on the start, pinned under
  `refs/marley/turns/<session>/<n>`. Under the terminal's rail row, "Turns (N)" opens to the
  session's turns, newest first, each with its title, the files it changed, and `failed` after an
  API error or `injected` for a turn a harness started (titled by what injected it, such as `task
  notification`); a slash command's turn is titled by the command. A click opens the turn in Zed's
  commit view, which shows that turn's changes and nothing else, shell commands' edits included.
  Your index, your branches and `HEAD` are never touched. A turn that changed nothing lists
  nothing, and a session's first turn prunes the turn refs older than 30 days.

- **The inbox says who should answer each waiting agent** (#570, 2026-09-28). With the question
  route's mode on, each entry of the rail's "Needs you" section carries a mark: `for you`, `for
  the manager`, `could proceed` or `unclear`. Marley's own rules decide first: what #568 marks as
  destroying, touching credentials, rewriting history, sending out or reaching outside the
  project, a held click, and anything that names money or a message to people is yours; a read
  inside the project, or a command that only reads, is one the agent could proceed on. The rest,
  such as `npm test` or an agent's question, the System One layer may read for a listed project,
  with the question's options and your prompt as masked text; a reading that cannot tell, or
  falls under the floor, is `unclear`, which sorts with yours. In Act the marks order the entries
  within each of #568's levels: yours and the unclear first, then the manager's, then what could
  proceed. A mark's tooltip names the rule or the reading and its confidence. Nothing here
  answers anything, and no manager is connected yet: a `for the manager` entry stays yours. When
  an entry leaves, its outcome records `owner` if you answered it from the inbox or had its
  terminal or thread in front while it waited, and `agent` otherwise. Off by default: Question
  Route on the Marley page, or `marley.system_one.uses.question_route`. Marley's Claude Code
  plugin goes to 1.4.0: an agent's question now carries its options, and the agent bar offers the
  update.

- **The inbox marks what each waiting action would do, and puts what matters most first** (#568,
  2026-09-28). With the inbox's risk use on, each entry of the rail's "Needs you" section carries
  chips from Marley's own rules, read from the tool and what it acts on: `destroys` (`rm -rf`,
  `git reset --hard`, `DROP TABLE`, a delete), `credentials` (`~/.ssh`, `.env`, a key file, a
  secret in the line), `rewrites history` (a rebase, an amend, a forced push), `sends out` (a
  `curl` that posts, `scp`, `git push`, a publish), `installs` (`npm install`, `pip install`,
  `curl … | sh`), `outside project` (a write or a command reaching outside the project's
  folders) and `claims approval` (text saying the action was already approved, which raises the
  entry rather than lowering it). A click a Browser tab holds carries its own: `pays`,
  `destroys`, `sends out` or `changes account`. Entries order by level, then age. What the rules
  find nothing on, such as `python3 scripts/cleanup.py`, the System One layer may read for a
  listed project, with the tool's name, the agent and the project as facts and the line masked;
  a reading may add chips and raise an entry, never remove a chip or lower one. Nothing here
  answers a prompt: Allow and Deny stay your clicks. Off by default: Inbox Risk on the Marley
  page, or `marley.system_one.uses.inbox`, with Shadow (the rules' chips and order, the model
  logged), Suggest (the model's chips with a question mark) and Act (dashed chips that order the
  inbox too). Each entry is a row in Decisions, and how it was cleared is its outcome.

- **One list at the top of the rail of every agent that waits on you** (#508, 2026-09-28). While
  an agent waits, "Needs you" and a count sit between the rail's filter and its projects, with an
  entry per wait, the longest waiting first: the agent and its project, what it asks, and how long
  it has waited (`now`, `3 m`, `1 h 5 m`). A Zed agent thread's tool call waiting for your
  confirmation has Deny and Allow under its entry, which answer that call once, as the thread's
  own buttons do; a prompt with other choices, or a sandbox escalation, opens its thread instead.
  Claude Code in a Marley terminal waiting on a permission or a question is listed with what it
  asks, and its entry shows the terminal. An agent's click a Browser tab holds (#571) has Refuse
  and Allow, as the tab's card does. An entry leaves once its wait ends, wherever it was answered,
  and the section leaves with the last one; the filter and a folded project never hide it. A pick
  from the Browser tab is no longer typed into a terminal whose Claude Code waits, where the line
  would land in its prompt: the tray says why and keeps the pick and its caption for Send.

- **An agent's click that pays, deletes, sends or changes an account waits for you** (#571,
  2026-09-28). With the click consequence's mode on, an agent's `browser_click` (or the click a
  `browser_type` with a ref makes first) on such an element waits in the Browser tab: a card
  under the toolbar names who wants to click what and why, with Refuse and Allow, and a toast's
  Show brings you to it. Allow clicks, but only the same element on the same page; Refuse, or 25
  seconds with no answer, tells the agent the user did not allow it, and while a click waits the
  tab's other writes wait too. Marley's own rules decide from the element's name, its form's
  target, its link and the page, with no call: "Place order", "Delete account" or a form that posts
  to `/checkout` wait. What they leave open, a "Continue" among text about a charge, the System One
  layer may read, for listed projects only, and a reading can add a pause, never remove one. The
  card never takes the focus by itself: Enter allows and Escape refuses only once you click it or
  press Show. By default only agents with no permission prompt of their own wait: Claude Code in
  a Marley terminal running `bypassPermissions` or `dontAsk`, Zed's agent while its tool
  permissions let `browser_click` run unasked, and any caller Marley cannot name; the Browser
  Click Pause Agents item (`marley.browser_click_pause_agents`) makes every agent wait. Off by
  default: Click Consequence on the Marley settings page, or
  `marley.system_one.uses.click_consequence`, with Shadow (the rules pause and the model is
  logged), Suggest (a notice after a click the model reads as consequential) and Act (the model's
  reading pauses too). The flight recorder keeps each pause and how it ended.

- **A working Claude Code that loops or has gone quiet is flagged on its rail row** (#569,
  2026-09-27). With the stall kind's mode on, the row reads `looping?` with a warning mark when the
  turn ends the same tool line three times in a row, such as `Bash: cargo test`, or fails the same
  line twice, from Marley's own rule with no call; a run of edits to one file is not a loop. A turn
  quiet past a check with no tool of its own using the CPU is asked of the System One layer, for
  listed projects only, and reads `stalled?` with the mark when the model says it waits for input,
  is stuck or is frozen; the mark's tooltip says what the flag rests on and how sure the reading
  is. A tool whose processes use the CPU, such as a build, is a long task and asks nothing. The
  first check comes after Stall Check After Seconds on the Marley settings page
  (`marley.stall_check_after_seconds`, 60; 0 turns the checks off), the next at twice, four and
  eight times it. Off by default: Stall Kind on the settings page, or
  `marley.system_one.uses.stall_kind`, with Shadow (logged in Decisions, the row unchanged),
  Suggest (the flag) and Act (the flag and one desktop notification while the terminal is out of
  sight). The agent's next event takes the flag off and logs how long it took, and agents see the
  flag as the seat's `flag` labels in `fleet_snapshot`. Marley never stops, interrupts or types
  into the agent on a flag.

- **Agents find an element or a line from words** (#567, 2026-09-27). Two tools of Marley's MCP
  server answer a query such as "the sign in button" or "where the server refused the
  connection". `browser_find` gives the ref of the page's element, which `browser_click` takes,
  and `terminal_find` the line of a block's output, each with up to three candidates. An element
  or a line that alone holds every word of the query answers at once, and no call is made. The
  System One layer ranks what the words leave open, for listed projects only, as a choice over
  the items and a question of whether anything matches, and an answer that is not sure says to
  read the page or the block. Both are off by default and, while off, agents do not see them: the
  Browser Find and Terminal Find items of the Marley settings page, or
  `marley.system_one.uses.browser_find` and `terminal_find`, with Shadow (the words answer and the
  model is logged in Decisions), Suggest (the model's candidates, to check) and Act. A running
  Claude Code sees a tool turned on once it lists its tools again. Decisions says "1 call" for one
  call.

- **What a stopped Claude Code turn needs, on its rail row** (#566, 2026-09-27). With the stop
  kind's mode on, an idle Claude Code's row says `done · checked`, `done · claimed`, `asks you`,
  `blocked`, `still going` or `interrupted` in place of `idle`, and names a part of the prompt the
  last message leaves out (`not covered: "Add a license."`). Marley's own rules come first and
  send nothing: an interrupt, a permission never answered, or a last sentence that asks settles
  the kind. The System One layer is asked the rest, for listed projects only, and `done · checked`
  needs a command Marley saw run after the last edit, so the model can take a check away but never
  add one. Off by default: Stop Kind on the Marley settings page, or
  `marley.system_one.uses.stop_kind`, with Shadow (logged in Decisions, the row unchanged),
  Suggest (`idle · still going?`) and Act. Agents see the kind as the seat's `stop_kind` labels in
  `fleet_snapshot`, and your next prompt clears it and logs how soon it came. Marley's Claude Code
  plugin goes to 1.3.0: a message over 300 characters now keeps its last whole sentences after
  its start, where a final question or status sits, and the agent bar offers the update.

- **A System One layer, off until you turn it on** (#565, 2026-09-27). Marley can ask a System One
  model, TypeSafe's Jev first, typed questions about what it knows. A feature gets back a reading
  it may show, rank or route on, never an approval. Off, which is the default, Marley makes no
  request, reads no key and writes no file. On, it sends only for the project folders you list: it
  masks every text value with #516's rules and your patterns, whatever agents' redaction is set to,
  and hides the key itself, and a metadata-only project sends facts alone. The key comes from
  `MARLEY_SYSTEM_ONE_KEY`, else the system keyring, and Decisions shows its source, never its
  value. `marley: system one check` asks whether the last command of the terminal you used last
  failed and shows the answer in a toast. `marley: open decisions` lists the day's calls with what
  each sent and got back, the spend against a daily budget (50 cents by default) and the key's
  source. The provider is TypeSafe's API, another server that speaks the same request, each use's
  own rules, or recorded answers. A slow or failing provider reads as unavailable with its reason,
  and five failures in a row hold calls for two minutes. The settings are the System One section
  of the Marley settings page and `marley.system_one` in settings.json.
- **A project's ports in the rail** (#521, 2026-09-27). Under each project the rail lists the TCP
  ports your processes listen on from inside its folders, such as a dev server started in its
  terminal: the port, the process's name and the URL, after the project's terminals, Browser tabs
  and threads. A row comes within seconds of a server's start and goes when it stops. The
  pointer on a row shows the command line, the working directory and the pid, with three buttons:
  Open (a Browser tab of the project on the URL), Copy (the URL) and Stop, which ends the server
  with SIGTERM once a fresh look finds it still listening there. A server on `0.0.0.0` or `::`
  opens on `127.0.0.1` or `[::1]`, and one on two addresses of a port has one row. Marley's own
  listeners, other users' and those outside every project get none. Agents get the same list,
  without the command lines, from the new `ports_list` tool.
- **Browser clients on other machines, over SSH** (#584, 2026-09-27). A program on another
  computer can now drive Marley's browser as an allowed client. Browser Clients shows, under the
  line for this machine, a line that runs Marley's bridge here over SSH, with Copy. The other
  machine's MCP client runs that line as its server. It holds no token: SSH checks who connects,
  and the bridge reads the client's token on this machine, so the line keeps working when Marley
  restarts. Marley listens on nothing new.
- **Programs outside Marley drive its browser, by name** (#524, 2026-09-27). Browser Clients
  (`marley: browser clients`) lets a program in by name, to read pages or also to act in them.
  Each gets a token of its own, new at each start of Marley, in an endpoint file only you can
  read, which the program points Marley's bridge at. It sees and calls only the browser tools its
  grant allows, never a terminal tool or what the agents' sessions hold. A tab it acts in names
  it, and shows "Driven by <name>" with Cut Off, which refuses its token at once. The server
  still listens on 127.0.0.1 alone.
- **Playwright scripts kept in Marley** (#523, 2026-09-27). A Browser tab's toolbar has a
  Scripts button, which opens a tray of Playwright scripts: the ones kept for the tab's project
  and the ones kept for every project, in Marley's config folder, so they never go into a
  repository. A name typed there makes a new script from Marley's template and opens it. Run
  runs a script on the tab's page, in a terminal beside the tab, while the tab stays in front: the
  script's default export gets the tab's page, its context and the browser, reached over the
  tab's own Chromium. The first run installs Marley's own Playwright (`playwright-core` 1.63.0)
  into its data folder, in that run's block. When a script fails, Marley saves the tab's last
  minute as a recording, with the run's start and end in its timeline, and names it in a toast.
  `marley: playwright scripts` opens the tray too.
- **Clear a project's browser data** (#581, 2026-09-27). The rail's project menu has Clear
  Browser Data…, and the command palette `marley: clear project browser data`. After asking,
  it closes the project's Browser tabs, closes its Chromium and deletes its profile, so every
  site in the project is signed out and its next Browser tab starts clean. Other projects keep
  their logins. A toast says when it is done, or what stopped it.
- **A Chromium and a profile per project** (#507, 2026-09-27). Each project now has a Chromium
  of its own on a profile of its own, so a site you sign in to in one project stays signed out
  in another, and each project's cookies, `localStorage` and IndexedDB survive restarts. A
  project is what the rail groups, its main folders, so a linked worktree shares its
  repository's browser and logins. A project's Chromium starts with its first Browser tab,
  keeps running when Marley quits so that restored tabs find their pages, and stops when you
  remove the project, which the rail's project menu now offers (Remove Project). The one profile
  of earlier builds moves, once, to the first project whose Chromium starts, logins and all; it
  is renamed, never deleted. Agents see every project's tabs in `browser_tabs`, a named tab acts
  in its own project's browser, and a new tab opens in the caller's project's browser. Before
  Marley stops a Chromium it asks Chromium to close: stopped by a signal alone, Chromium loses
  the cookies it set in its last 30 seconds, which a stop at logout still can.
- **A recording drafted as a Playwright test** (#506, 2026-09-27). A saved recording now holds
  each click, fill and key press you or an agent made in the page, with the target's locators
  as they were at that moment: a test id, the role and name, the label, the placeholder, the
  text, a CSS path, each marked when it found the element alone. Agents turn a recording into a
  test with `browser_draft_test`. The test replays each step with the first locator that
  found its target alone (`getByTestId`, `getByRole`, `getByLabel` and the rest), sets
  `baseURL` from where the recording started, and checks the URL after each navigation a step
  caused. A password or other secret field is filled from an environment variable, and the
  test fails naming the variable when it is unset. The tool writes nothing: it answers the
  test and a path in the project (the `testDir` of its `playwright.config`, else `tests/`),
  and the agent puts the file there, where it runs with `npx playwright test` in Marley or
  anywhere else.
- **Check a pick after a fix** (#505, 2026-09-26). Each pick in a Browser tab's tray has a
  Check button. Once you or an agent have changed the page, Check finds the picked element again
  by the most durable of its locators that still finds anything: its test id, its id, its role
  and name, its text, then its CSS path, and the one nearest where it was when several match.
  It scrolls the element into view when it is off screen, and opens a card over the page with
  the crop at the pick beside the crop now and what changed: the box, the computed styles, the
  text, the role and name, or only the HTML. The row keeps the verdict ("2 changes", "no
  change", "not found"), which opens the card again. Agents get the same through
  `browser_check_pick`, redacted as a pick is, and `browser_pick` now includes the pick's latest
  check. A pick's locators no longer rest on ids a framework generates, such as React's `useId`,
  Radix, Headless UI or a hash.
- **A fuller pick for agents** (#518, 2026-09-26). `browser_pick` now also gives an agent the
  picked element's HTML, sixteen of its computed styles, up to ten texts of the elements beside
  it, and the page's selection. On a React dev build it adds the components around the element
  and the file, line and column it was written at, found in the project: React 18 names the
  file, and on React 19 Marley finds it through the page's source maps. The HTML comes without
  scripts, field values or URL queries, with secret-looking attributes replaced, and is cut at
  4,096 characters. What a pick hands an agent, through `browser_pick` and `browser_picks`, now
  passes the secret redaction first (`marley.redact_secrets_for_agents`): the page's title, your
  caption, the element's text and name, the HTML, the nearby texts and the selection. Each
  field is redacted whole and only then cut, so a token that straddles a cut never leaks in
  part.
- **Browser tabs in the rail** (#504, 2026-09-26). Each Browser tab now has a row under its
  project in the rail, after the project's terminals. The row shows the page's icon (a globe when
  it has none, a spinner while it loads), the page's title, and its host and port. It counts the
  picks in the tab's tray and the page's annotations, and marks a page an agent acted in while no
  tab showed it, until you look at it. A click shows the tab with the focus. Up, Down and Enter,
  the filter and a close button on hover work as they do on terminal rows. The rail never starts
  the browser.
- **A menu on a clicked terminal link, and wrapped URLs opened whole** (#579, 2026-09-26). A
  plain click on a web URL in a terminal opens a small menu at the pointer: Open in Browser Tab,
  Open in System Browser and Copy Link. The right-click menu starts with the same entries when it
  lands on a link, and Copy Link there copies an OSC 8 link's hidden target. Until you choose,
  both menus also offer to make a Browser tab or the system browser the default for local links,
  which writes `marley.terminal_links`. A URL a program wrapped itself at the terminal's edge, or
  drew inside a box such as an agent's frame, now opens whole from any of its rows. A plain click
  on an OSC 8 link still opens it.
- **Programs that open a browser open a Browser tab** (#561, 2026-09-26). `gh pr view --web`,
  Vite's `--open`, Python's `webbrowser` and any other program that opens a URL through `BROWSER`
  now open a local one (`localhost`, a loopback address) in a Browser tab of the project their
  folder is in, with the focus, from any terminal Marley starts. Any other URL, a `file://` page,
  or a program outside every project still goes to your browser, as it did. Marley gives its
  terminals its own opener as `BROWSER`. A `BROWSER` that your shell's files or `terminal.env`
  set wins, and `marley.terminal_links` set to `system_browser` turns the opener off. Agents get
  `browser_open_url`, which opens such a URL in a Browser tab of the project that holds a folder.
- **Local URLs in a terminal open in a Browser tab** (#503, 2026-09-26). Ctrl+click on a
  `localhost` or loopback URL in a terminal opens it in a Browser tab of the terminal's project,
  or brings forward the tab already on it; any other URL still opens in your browser. A
  server's `0.0.0.0` address opens at `127.0.0.1`. Shift+Ctrl+click opens a URL in the other
  place. `marley.terminal_links`, in the Marley settings page's new Terminal section, can send
  every URL to a Browser tab or every URL to your browser. An OSC 8 link a program prints opens
  the same way on a plain click. While something listens on the port of a local URL a terminal
  printed, such as a dev server's, the terminal's footer offers it: click it to open the page,
  or use its menu to open it in your browser or copy the URL. A terminal running ssh sends every
  URL to your browser and offers nothing, since its `localhost` is another machine.
- **Send the editor's selection to an agent in a terminal** (#549, 2026-09-26). In the Marley
  layout, `ctrl->` in a file's editor types a reference to the selection at the prompt of the
  CLI agent running in a terminal of the window, without pressing Enter, and focuses that
  terminal: `@src/auth.rs#L12-40` for Claude Code, `src/auth.rs:12-40 ` for Codex, Gemini CLI
  and OpenCode, the path relative to the agent's folder when the file is inside it. With several
  agents a picker asks which; with none, the key does what it always did and adds the selection
  to Zed's Agent Panel. Rich input open on the terminal gets the reference instead. Nothing is
  typed while Claude Code waits on a permission or a question, which a paste would answer; a
  toast says so. `marley: send selection to agent` does the same from the palette, in either
  layout.
- **A terminal keeps its id across a restart** (#575, 2026-09-26). A terminal Marley restores
  at a launch now keeps the `MARLEY_TERMINAL_ID` it had, so an agent resumed there, or a script
  that kept the id, still names the same terminal, and Marley's tools mark it `self` as before.
  A split of a restored terminal and a new terminal still get an id of their own. The ids live
  in Marley's own table beside Zed's terminal rows.
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

- **`just install` no longer runs the golden set unless asked** (2026-09-29). It builds the
  release `marley` and installs it; `just install --regress` runs the golden set of e2e
  scenarios against the build first and installs nothing on a failure, as every install did
  since #517. `--skip-regress` is still accepted. Each ticket's Test phase is now a visual check
  of its own change, the real Marley driven and every shot read, with no unit tests and no
  regression run; the static gate runs at the end of Code (CONSTITUTION §0, §3 and §7).
- **Marley's browser listens on no network port** (#583, 2026-09-27). Each project's Chromium
  used to take DevTools connections on a port on 127.0.0.1 with no password, which any program
  on the machine could reach, other users' included, and then read the project's cookies or
  drive its pages. Chromium now talks only to Marley's relay, a small part of Marley that runs
  beside it. Marley reaches the relay through a file only you can open. Other tools, such as a
  Playwright script, connect to the relay with a token that is new each time the browser starts,
  kept in a file only you can read. A browser an earlier build started is closed and started
  again the new way the first time Marley opens it.
- **The flight recorder keeps what you type into ordinary fields** (#506, 2026-09-27). Until
  now a recording held typing only as a count of characters. It now also keeps the text of
  each fill of an ordinary field (an e-mail, a search), up to 4,096 characters, which a
  drafted test needs. A password, hidden, one-time-code or card field still keeps nothing but
  the fact that it was filled. `browser_recording` now passes the whole recording through the
  secret redaction (`marley.redact_secrets_for_agents`) and gives agents at most 1,000
  characters of a fill.
- **More secrets hidden from what agents read** (#562, 2026-09-26). An `Authorization` or
  `Proxy-Authorization` header now loses its credential under any common scheme, not only
  `Bearer`: `Basic`, `Token`, `Negotiate`, `NTLM`, `ApiKey`, and `Digest` with its whole parameter
  list. Agents read `Authorization: Basic [redacted: authorization]`, so the scheme still shows.
  A `Cookie` or `Set-Cookie` header loses its whole value (`[redacted: cookie]`), and values set on
  hyphenated names such as `x-api-key`, `private-key` and `access-key`, or on `BEARER` and
  `PRIVKEY`, are hidden like `API_KEY`'s. The rules come from comparing Orca's redactor with
  Marley's rule by rule. Orca's rule that hides every `NAME=value` line stays out, so `env` output
  still shows `PATH`.
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

- **A cleared session no longer reads as a finish** (#538, 2026-09-29). A `/clear`, a resume or a
  new session while Claude Code waited made Marley push `Claude finished` to the phone (#535). A
  session's start now pushes nothing and shows no banner.

- **The note that an agent typed into a program goes when the program does** (#595,
  2026-09-29). After an agent typed into a program in a terminal, the bar under it saying so, with
  Take Over, stayed under the shell once that program had exited. It now shows only while the
  program it names is still running.
- **An agent's Enter runs the line** (#594, 2026-09-29). When an agent typed into a running
  program with `terminal_type` and pressed Enter, Python's REPL took the Enter as part of the
  pasted text: the line sat there with `...` and never ran, while the tool said it was typed.
  The same could happen to the rich input's Enter and to review notes sent to an agent. Marley
  now presses Enter, and any keys, a moment after the paste, so the program reads them as keys.
- **Allow and Deny sit clear of the toast** (#593, 2026-09-29). When an agent asked to type into a
  running program, the toast that announced it covered Allow and Deny at the right end of the
  card under the terminal, so a click there hit the toast and the write ran out its 25 seconds.
  The card now puts Allow and Deny first, at the terminal's left edge.
- **A launch config's approval shows exactly what runs** (#592, 2026-09-29). The question a
  project's launch config asks before it first runs put every item on one line, showed `--` as
  a dash and curled the quotes, because Zed's prompt reads its text as Markdown. A command could
  also have hidden part of itself from the question, as an HTML comment for one. The approval
  now shows each item on its own line, every character as the file has it.
- **Websites see Marley's browser as Chrome** (#539, 2026-09-27). Pages in a Browser tab were
  told they ran in headless Chrome (`HeadlessChrome/152` in the user agent), which bot checks
  such as Cloudflare's refuse, so some sites would not load or let you sign in. Every page now
  sees `Chrome/152`, with Chromium's own client hints, which agree with it. That holds in a tab,
  in the cross-site frames a page holds, in an agent's new tab, and in a tab Marley opens from a
  link or at a restart. The first document of a popup a page opens, and service workers, still
  see the headless name.
- **A client that was cut off is told so** (#584, 2026-09-27). After Cut Off, a program using
  Marley's bridge was told that Marley was not running and to start it, while Marley ran. It is
  now told that Marley does not allow it, and that the user can allow it again in Browser
  Clients.
- **The secrets gate reads Marley's e2e scenarios before they are committed** (#584,
  2026-09-27). gate:10 scanned the scenarios, their runner, `script/regress` and the installer
  only once they were committed, so a new line reached the public repository before gitleaks saw
  it. They are now scanned in the working tree too. `.gitleaks.toml` also allows RFC 6455's
  sample WebSocket key, which #583's scenario sends.
- **A failed Playwright run's recording keeps the run's start** (#583, 2026-09-27). In the
  installed build, the recording a failed script saves could hold the run's end and not its
  start, when the run began in the moment its terminal took the tab's place before moving beside
  it. The start is now kept either way.
- **Marley's tool server bounds what it reads before it knows who asks** (#524, 2026-09-27). A
  program on the same machine could send Marley's MCP server one endless header line and make
  Marley take memory until it stopped, or keep connections open for good. A request line or
  header line now stops at 8 KiB, a request at 100 header lines, and a connection has 10
  seconds to send its request.
- **A Browser tab's title follows its page** (#582, 2026-09-27). A page that set its title
  after it loaded, such as an unread count or a title set once its data came in, kept its old
  title in its Browser tab, in its row in the rail and in what agents read with `browser_tabs`:
  Chromium tells Marley nothing about such a change. A watcher in each page now reports every
  new title, and all three follow it, for a tab behind another too.
- **A project's Browser tabs come back after another project's** (#576, 2026-09-26). A Browser
  tab saved in one project could replace another project's saved tab when Marley happened to give
  both the same internal item id in different launches, and that project's tab then did not come
  back. Each project's saved tabs are now kept apart; the saved tabs you already have are kept.
- **A restored terminal opens in the folder it was in** (#577, 2026-09-26). After a restart, a
  terminal that had moved into a subfolder could come back in the project's folder instead: the
  terminal panel, cleaning up its own saved terminals, deleted the other terminals' saved rows,
  sometimes before their restore read them. It now keeps the rows of the terminals the
  workspace restores.
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

### Security

- **Test runs leave the user's phone, harness, hosts and System One alone** (#668, 2026-10-06).
  `script/e2e.sh` copies the user's settings into each run's profile; the copy now drops
  `marley.push`, `marley.harness`, `marley.embedded_harness`, `marley.fleet` and
  `marley.system_one`, and the run unsets `MARLEY_SYSTEM_ONE_KEY` and
  `MARLEY_CLOUDFLARE_API_TOKEN`, so no scenario can push to the user's phone, reach their agent
  harness or hosts, or spend their System One budget. A scenario that tests one of them sets its
  own fake, as the six that do already did.

- **wasmtime 48.0.5** (#642, 2026-10-04). The extension host's WebAssembly runtime moves from
  48.0.3 to its patch release 48.0.5, with cranelift 0.135.5 and the wasm-tools crates it needs at
  0.254.2, for seven advisories published on 2026-10-02: RUSTSEC-2026-0325, -0326 and -0327 (GC
  heap corruption and a native stack overflow a guest could cause) and RUSTSEC-2026-0321 to -0324
  in its WASI layer (fuel accounting a guest could get around, host memory, uninitialized padding
  copied to a guest, a panic through a file's timestamp). Only the lockfile changed.

- **wasmtime 48.0.3** (#511, 2026-09-29). The extension host's WebAssembly runtime moves from
  48.0.1 to its patch release 48.0.3, with cranelift 0.135.3, for RUSTSEC-2026-0314 (a guest
  could panic the host through a file's timestamp), RUSTSEC-2026-0315 and RUSTSEC-2026-0316
  (fuel accounting a guest could get around). Only the lockfile changed.
