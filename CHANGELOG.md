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

- **A project's changed lines and pull request on its rail row** (#531, 2026-09-29). A project
  whose folder is a repository's main checkout shows, at the right of its row, the lines added and
  removed since its branch left its base (`+12 ‒3`, uncommitted edits to tracked files included),
  and when the branch has a pull request on GitHub, a chip with the pull request icon and its
  number, green while open, gray as a draft, in the accent color once merged and red when closed;
  its tooltip gives the state, the title and the link. The base is the pull request's base,
  otherwise the repository's default branch (its `origin` copy when there is one, so a project on
  its default branch counts what it has not pushed). The counts follow each save, the branch and
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
  since #517. `--skip-regress` is still accepted. The workflow went to three phases, Plan, Code
  and Complete: no ticket writes or runs tests, and the static gate runs at the end of Code
  (CONSTITUTION §0, §3 and §7).
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

- **wasmtime 48.0.3** (#511, 2026-09-29). The extension host's WebAssembly runtime moves from
  48.0.1 to its patch release 48.0.3, with cranelift 0.135.3, for RUSTSEC-2026-0314 (a guest
  could panic the host through a file's timestamp), RUSTSEC-2026-0315 and RUSTSEC-2026-0316
  (fuel accounting a guest could get around). Only the lockfile changed.
