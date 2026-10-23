# Board review: Browser, BrowserTools, Tools, Fleet, Components, SystemOne, AgentAccess

Paths are relative to /srv/stacks/marley_ide/crates/marley_workbench/src unless they say
otherwise. A shot is named only when it shows the feature; "not seen in a run" means the
verdict comes from the code alone.

One finding covers four boards. Browser, Tools, Fleet and Components each draw a bottom row for
their screen (a focus chip, counts, a warning). `focus_row.rs:33-38` knows only None, File,
Terminal and Thread, and #833's deferrals don't mention these screens. That finding is repeated
under each board below.

Note on shots: `tour-18-fleet.png` shows the Component Library, not the Fleet panel, so no
Fleet item below was seen in a run.

## Browser

- **Bottom row for a Browser tab.** Board: a `www <page title>` chip, the address, a red
  "1 console error" chip, then "1 pick ready" and "recording the last minute". Marley: the row
  shows nothing for the browser (`t2-14-browser.png`; `focus_row.rs:33-38` has no browser
  case; grepped "pick ready" and "the last minute": not found). Kind: missing. Size: M.
- **Picked element marked on the page.** Board: once picked, the card on the page keeps an
  accent border and a `div.card · 2` badge. Marley: only Chromium's inspect highlight, and only
  while pick mode is on (`marley_browser/src/pick.rs:842-871`). Nothing is drawn over the frame
  for a pick (`browser.rs` `render_annotations` draws annotation boxes only). Not seen in a
  run. Kind: missing. Size: M.
- **Project chip in the address box.** Board: the project name is an accent-filled chip.
  Marley: a muted plain label, "Default" (`t2-14-browser.png`; `browser.rs:7058-7062`). Kind:
  look. Size: S.
- **Console drawer.** Board: the drawer always sits under the page, and the Console tab's count
  is the error count, in red. Marley: the drawer stays shut until you click the toolbar's
  Console button (`t2-14-browser.png`; `browser.rs:7133-7159`). Its tab counts every entry, not
  the errors (`browser.rs:7222`). Kind: look. Size: S.
- **Error rows in the console.** Board: an error row has a tinted background. Marley: only the
  level word turns red (`browser.rs:8517-8561`). Kind: look. Size: S.
- **Pick card head.** Board: a selector chip (`div.card · 2`), then the source location
  (`index.html:212`), then the element's text. Marley: "Pick 2" in accent plus a summary, with
  the listener's place on a line of its own (`browser.rs:8207-8226`). Not seen in a run. Kind:
  look. Size: S.
- **Note field wording.** Board: a 3-row textarea with the placeholder "What should change
  here?". Marley: a one-line editor, "What should the agent know? Enter sends"
  (`browser.rs:6171`). Kind: look. Size: S.
- **"To" control.** Board: a dropdown (`Claude Code · …` / `A new agent`). Marley: two stacked
  radio-style rows (`browser.rs:8108-8150`). Choosing among several agent terminals is
  deferred: #896 sends only to the terminal used last. Kind: look; the multi-terminal choice is
  deferred (#896). Size: S.

## BrowserTools

- **Record This: the last minute.** Board: an activity bar chart of the minute, a list of steps
  at relative times (−0:41 Open…, −0:22 Click…), then "Draft a test" and "Send to the agent".
  Marley: Record saves the minute and shows a toast (`browser.rs:6634-6650`). The drawer's
  Recordings tab lists when, title, URL, length and Send only (`browser.rs:7339-7388`). No
  timeline, no step list, and no "Draft a test" button; drafting a test exists only as the agent
  tool `browser_draft_test` (`browser_tools.rs:716`; grepped "Draft a test": not found in
  code). #892's summary names Draft a test, but its REQs left it out and nothing defers it. Not
  seen in a run. Kind: missing. Size: M (button S, timeline M).
- **Playwright scripts: last run and result.** Board: each script shows "this project · ran 2 h
  ago" and a coloured result (passed / 1 failed / not run / drafted from a recording). Marley:
  name, "for <scope>", Run and Edit only (`browser.rs:7866-7903`). `Script` keeps no run
  history (`playwright_scripts.rs:77-84`). Not seen in a run. Kind: missing. Size: M.
- **New script field.** Board: one field, "New script name, then Enter". Marley: the field
  "A new script's name" plus "For This Project" and "For All Projects" buttons
  (`browser.rs:6474`, `7905-7940`). Kind: look. Size: S.
- **Take-over bar.** Board: "Agents are driving this page" with a live dot and "Take Over ·
  Ctrl I". Marley: "<who> acted in this page" with a sparkle and a plain "Take Over" button;
  Ctrl-I works but isn't shown (`browser.rs:5862-5925`). Not seen in a run. Kind: look. Size: S.
- **The page's own dialog.** Board: the title is "127.0.0.1:4180 asks". Marley: "<origin> says"
  (`browser.rs:7616-7619`). Kind: look. Size: S.
- **Annotate note.** Board: the hint "Enter keeps it · Esc drops it" sits beside the field.
  Marley: the placeholder "A note for this box; Enter keeps it", and Esc isn't named
  (`browser.rs:6710`). Kind: look. Size: S.
- **Clear this project's browser data.** Board: "Clear marley_ide's browser data?" / "Its 2
  Browser tabs close, and every site signs out. Other projects keep theirs." Marley: "Clear the
  browser data of X?" / "Its Browser tabs close, and every site in it signs out: its cookies,
  local storage and IndexedDB are deleted." It has no tab count and no line saying other
  projects keep theirs (`browser.rs:10102-10110`). Kind: look. Size: S.
- The held click (Refuse · Esc / Allow · Enter), the download bar (saved to …, Show) and the
  file chooser match the board.

## Tools

- **Agent Activity beside MCP Servers.** Board: an "Agent activity" column to the right of the
  MCP Servers page, with "Stop all". Marley: Agent Activity is its own tab with a "Stop" button
  (`tour-19-agent-activity.png`; `agent_activity.rs:323-360`). Kind: deferred (#846: "Agent
  Activity … keep their tabs"). Size: M.
- **Activity row.** Board: a lettered agent tile (CC, M, R), the tool and the time, then what
  it acted on, then its result ("ran in repo — bash · exit 0", "you accepted", "no element at
  that point" in red). Marley: one table line of time, who, tool, summary and "done" or
  "refused: code" (`agent_activity.rs:363-407`). Kind: look. Size: S.
- **Tool results and failures in the activity list.** Board: a call that failed shows its
  error. Marley: `Outcome` is only Done or Refused (`agent_activity.rs:66-80`), so a tool error
  and a call's result aren't recorded. Kind: missing. Size: M.
- **Rusty's calls in the activity list.** Board: a Rusty row (`R brain_decide … page
  written`). Marley: the log covers only Marley's own write-tier MCP tools
  (`agent_activity.rs:1-7`; grepped rusty and brain: not found). Kind: missing. Size: M–L.
- **Page subtitle.** Board: "What each agent can reach, and what Marley needs of it" under the
  title. Marley: no subtitle (`tour-13-mcp-servers.png`). Kind: look. Size: S.
- **Agent card head.** Board: a lettered tile (CC / CX), the name over its path, and a state
  pill ("ready" in green, "missing" in red). Marley: name, version and path on one line, with no
  tile or state pill (`tour-13-mcp-servers.png`; `mcp_servers.rs:488-510`). Kind: look. Size: S.
- **Integrations.** Board: compact chips with a dot each; the "tested on" text sits in a
  tooltip. Marley: one row per integration, with the state word and the "tested on" text inline
  (`tour-13-mcp-servers.png`; `mcp_servers.rs:530-550`). Kind: look. Size: S.
- **Marley's own server card.** Board: an "M" tile, "marley · Zed's agents · running in repo",
  "90 tools: editor, terminal, browser, settings, threads. Every call that acts is listed in
  Agent Activity.", and a "running" pill. Marley: a "ZED" card with "marley Zed running in repo
  Marley's", the bridge path and the env var names; no tool count, no summary of what the tools
  cover, no pointer to Agent Activity (`tour-13-mcp-servers.png`). Kind: look. Size: S.
- **Bottom row on MCP Servers.** Board: "MCP Servers", a warning chip "Codex is not on the
  PATH", and "marley · 90 tools". Marley: nothing for this screen (`tour-13-mcp-servers.png`
  shows only the rust-analyzer error; `focus_row.rs:33-38`). Kind: missing. Size: S–M.

## Fleet

Not seen in a run: `tour-18-fleet.png` captured the Component Library.

- **Fleet as a centre tab with the agent beside it.** Board: a "Fleet" tab (and "Agent ·
  code-1") in the centre; the list on the left, the selected agent's snapshot as a right-hand
  column. Marley: Fleet is a dock panel (`fleet.rs:35`, `impl Panel`), with the snapshot under
  the list in a draggable split (`fleet.rs:1530-1545`). Kind: look. Size: M.
- **Snapshot: a CPU and memory chart over the last 30 minutes.** Board: an inline chart in the
  snapshot. Marley: the panel snapshot has bars only (`fleet.rs:1159-1240`); the chart lives in
  the full Agent tab (`agent_tab.rs:1-5`, `619`). Kind: deferred (#848: "the agent snapshot pane
  keeps #608's look"). Size: M.
- **Snapshot: events.** Board: an Events list ("14:08 gate clippy passed"…). Marley: the panel
  snapshot has none (`fleet.rs:965-1052`); events are in the Agent tab only (`agent_tab.rs:121`).
  Kind: deferred (#848). Size: S.
- **Snapshot: phase names.** Board: every phase's name under its segment (plan, code, test,
  complete), each coloured. Marley: one line naming the active phase only, "phase 2/4: code"
  (`fleet.rs:1115-1144`). Kind: deferred (#848). Size: S.
- **Snapshot: work item and tokens.** Board: the ticket key as a chip next to its title, and
  "Tokens today" as a third meter bar. Marley: "WORK ITEM key · title" as text, and "TOKENS
  TODAY in … · out …" as text (`fleet.rs:971-1020`). Kind: deferred (#848). Size: S.
- **Session quota on the rail's Harness rows.** Board: "five_hour 62 % · resets in 1 h 35 m"
  under a session. Kind: deferred (#886: "the rows carry no quota today"). Size: S–M.
- **Bottom row for Fleet.** Board: "Fleet", "3 agents · 2 hosts", a pink "review-1 waits"
  chip, "Harness connected". Marley: nothing (`focus_row.rs:33-38`). Kind: missing. Size: S–M.

## Components

- **Templates as a compact strip.** Board: "Your templates · components/ in Marley's config
  folder" with one chip per file and its state ("loaded" in green, "line 7: unknown type" in
  red). Marley: a "Templates" group with Open Folder and one full drawn card per template; a
  broken file is an error callout (`component_library.rs:344-420`). Kind: look. Size: S.
- **Try it header.** Board: "Try it · Paste JSON; it draws as you type", with the parse error
  on the header line ("line 3: "tone" is success, warning, danger or neutral"). Marley: "Try it"
  and the placeholder "Type or paste a component's JSON here to draw it"; the error is drawn
  under the field (`tour-14-components.png`; `component_library.rs:263-308`). Kind: look.
  Size: S.
- **Selected component in the left list.** Board: the chosen name (card) is highlighted.
  Marley: a click writes the name into Find and narrows the page; nothing marks it as chosen
  (`component_library.rs:208-221`). Kind: look. Size: S.
- **Bottom row for the Component Library.** Board: "Component Library", a red chip
  "quota.json · line 7", and "17 components · 3 templates". Marley: nothing
  (`tour-14-components.png`; `focus_row.rs:33-38`). Kind: missing. Size: S–M.

## SystemOne

Not seen in a run.

- **One System One page holding the uses and their modes.** Board: a "System One" page with
  the lede "Small typed questions to a model about what Marley already knows. Off, it asks
  nothing and reads no key.", the Provider and Today cards, then "Each use and its mode": ten
  rows of Off / Shadow / Suggest / Act. Marley: the modes are buttons in the Settings window
  (#847 REQ-001). The tab is titled "System One calls" and has no lede
  (`system_one_calls.rs:480`). Kind: look. Size: M.
- **"Where its answers show".** Board: a gallery of the rail row, the running-error block and
  the inbox risk mark. Kind: deferred (#847). Size: S.
- **Calls table header and columns.** Board: Time / Use / Mode / Answer / Cost / Took as
  columns, with the mode coloured. Marley: one line per call, "time · use · project · provider
  · model", the reading, then the numbers; no column header and no mode column; a shadow call
  reads "would show: …" (`system_one_calls.rs:361-425`). Kind: deferred (#847: "the rows keep
  #565's lines"). Size: S.
- **Failed call row.** Board: a tinted background. Marley: red text only
  (`system_one_calls.rs:411-417`). Kind: look. Size: S.
- **Calls subheading.** Board: "System One calls · newest first · what left the machine is
  shown whole". Marley: no such line; the row expands on click. Kind: look. Size: S.

## AgentAccess

Not seen in a run.

- **The editor consent question.** Board: an agent tile, "Claude Code wants to edit rail.rs",
  the task and project ("Fix the rail test · marley_ide"), the change as a −/+ diff, then the
  footnote "One undo step, left unsaved. Saving asks again." Marley: "<who> wants to use
  editor_edit", the line "change <file>", and a Project row. No diff, no task line, no footnote
  (`agent_control.rs:238`, `318-334`; `editor_tools.rs:322-331`). The three buttons match. Kind:
  missing (diff and footnote, M) and look (headline wording, S).
- **Agent Control, per area.** Board: a block listing Editors, Threads, Palette actions and
  Starting agents, each with a mode pill, and below it "Stop Agent Control · every tool that
  acts refuses". Marley: four-button rows in the Settings window, from the variant names (Off /
  Ask Every / Ask First / Allow) (`settings_content/src/marley.rs:599-609`;
  `settings_ui/src/marley_page.rs:1219`). The stop button is on the Agent Activity tab.
  "Ask Every" should read "Ask Every Time". Kind: look. Size: S.
- **Pairing question.** Board: "“laptop” asks to pair with Marley"; "From ~/work over SSH.
  Check the code matches what the command printed."; the code in six digit boxes; three radio
  grants (Read / Read and Write / Read, Write and Run), each with a line on what it allows;
  Refuse and Pair; then "No answer in two minutes pairs nothing." Marley: the code sits inside a
  sentence; the grants are four buttons with no descriptions; there is no Pair button; the
  footnote is only "Asked from <folder>", with no "over SSH" and no two-minute line
  (`clients.rs:676-712`). Kind: look. Size: M.
- **Outside Clients power levels.** Board: Read / Read and Write / Read, Write and Run.
  Marley: Read / Write / Run (`clients.rs:944-948`). Kind: look. Size: S.
- **"Kept from agents".** Board: the secret globs as chips, and the line "Keys and tokens in
  terminal output become [redacted: token] before an agent reads them. Your terminal still
  shows them." Marley: the globs exist only as `marley.agent_control.secret_globs` in
  settings.json (`assets/settings/default.json:1983-1993`); the redaction exists
  (`marley_mcp/src/redact.rs`). No screen shows either (grepped "Kept from agents" and "Your
  terminal still": not found). Kind: deferred for the glob list (#844: "lists and maps stay in
  settings.json"); missing for the section and its note. Size: S–M.
