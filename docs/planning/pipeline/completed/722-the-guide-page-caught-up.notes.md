# The guide page caught up — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-722-the-guide-page-caught-up.md
- **Pipeline spec:** 722-the-guide-page-caught-up.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-09: "lets spin up some documentation bots and update the marley
  html".
- **Recall:**
  - The page was last touched at #687 (git log); `guide.md` is current through #711.
  - #599's scenario drives the page's contents filter.
- **Design:**
  - **The bots:** three run in parallel and write `fragments.html` (`REPLACE`/`INSERT-AFTER`
    markers) and `report.md` under `scratchpad/guide-bots/{rail,agents,mcp}`.
  - **The merge** is a script that applies the markers to `index.html`, then checks that the ids
    are unique and the tags balanced.
  - **My review:** each fragment is read against `guide.md` and the code before it is merged.
- **File manifest:**
  - `crates/marley_workbench/guide/index.html` (a Marley crate);
  - the scenario.
- **Visual check plan:** the spec's four shots.

## Phase 2 — Code
- **The bots' drafts (all three bots ran in parallel, drafts only):**
  - **rail:** 6 articles replaced, 3 new (`home-page`, `rusty-group`, `thread-in-center`);
  - **agents:**
    - `section#marley-agent` replaced by four articles;
    - a new `section#harness` (sessions, writes, a seat, the manager);
    - 3 agent articles replaced and 4 new (agent versions, the shared plugin, Claude Code's IDE
      link, Codex's App Server);
    - 3 fleet articles replaced;
  - **mcp:**
    - `mcp-tools` (48 served tools, by family), `mcp-safety` and `browser-clients` replaced;
    - `agent-activity` and `agent-control-modes` new.
- **Merge:** `merge.py` applied 26 markers; the check found 140 unique ids, every tag closed and
  0 problems. The nav got the 15 new entries, and every nav anchor resolves.
- **My review against the code:**
  - **Confirmed:** the secret globs (8, `editor_tools.rs`), Home's card at 5 rows
    (`ACTIVITY_SHOWN`), the palette names (`StopAgentControl`, `ResumeAgentControl`,
    `OpenAgentActivity`, `OpenThreadInCenter`, `MoveThreadToPanel`, `OpenMarleyAgentInTerminal`,
    `OpenHarnessSession`, `rusty::OpenHome`), the eight profile tools (`PROFILE_TOOLS`), the
    plugin's version 1.6.0, and the assistant's defaults.
  - **One bot claim rejected:** that guide.md's CONTAINERS list is stale. `rail_containers.rs`
    still draws it.
- **Out of the bots' areas, fixed by hand:**
  - the settings page's sections, nine → eleven;
  - the Commands table: thread in center and back, the Marley agent in a terminal, agent
    activity, stop and resume, a harness session, Rusty's home;
  - the Settings table: `embedded_harness`, `harness_writes`, `claude_code_shared_plugin`,
    `assistant` and `agent_control`; "edits all of them" corrected to "most".
- **guide.md fixes** (the text `docs_search` serves):
  - the Marley agent's tool list gains `seat_add`, and the Zed profile's count goes to eight;
  - the plugin's version, 1.4.0 → 1.6.0;
  - "Marley answers nothing Codex asks" replaced by #651's inbox answers;
  - the Grants bullet, which said only `browser.write`, now names the seven classes and the
    agent-control modes.
- **Gate:** `722-gate-1.log` GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/722-the-guide-page-caught-up.sh`, under `compositor sway`, run with
  `nice -n 19` while the harness's 093 gate ran (the load average stayed under 4.5). `marley: open
  guide`, then the contents filter and a click on its first match.
- **Run a:** three of the four shots opened an earlier article. The filter matches every word
  anywhere in an article (`words.every`), so "Home's page", "kill switch" and "on out of the box"
  first hit the groups, the Rusty group and Install. No source fault.
- **Run b:** the queries became phrases from the titles; 01–03 are right, but 04 still opened the
  settings page article. The queries were then checked against the page's own matching (a script
  emulating it) before the next run.
- **Run c (`shots-722c`): every shot shows its criterion.**
  - **722-01-home (REQ-001):** "Home's page: a terminal, an agent, a project #701 · #703" with
    its card table (START, NEW AGENT, RECENT PROJECTS, AGENTS AT WORK, CONFIGURE, AGENT ACTIVITY)
    and the note on Stop and the palette commands.
  - **722-02-agent-control (REQ-002):** "Agent Activity and the kill switch #703": the four
    steps, Stop and Resume, and the note on `marley.agent_control.stopped` and the day files.
  - **722-03-modes (REQ-002):** "How far agents go in editors, threads and actions": the modes
    table, the question's three buttons, saves and answers asking every time, Allowed Actions, and
    the refused actions.
  - **722-04-marley-agent (REQ-003):** "The Marley agent, on out of the box #683 · #687 · #696":
    the Auto/Claude Code/Codex/Zed table and "Marley no longer offers the agent at start".
  - Run b's 04 also showed the settings page article reading "eleven sections".
- **REQ-004:** `merge.py`'s check, 140 unique ids and 0 problems, and every nav anchor resolves
  (Phase 2).
- Chad's Hyprland untouched.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added). The page is the documentation; `guide.md` was fixed in
  Phase 2.
- **Knowledge appended:** L-claude-722-documentation-bots-draft-and-one-hand-merges-001. No
  product bug was found.
- **Brain:** no `rusty` MCP server in this repository's sessions.
- **Ticket:** closed; it never had a BACKLOG row.
- **Gate:** `722-gate-2.log`, GATE GREEN [diff], on the tree committed.
