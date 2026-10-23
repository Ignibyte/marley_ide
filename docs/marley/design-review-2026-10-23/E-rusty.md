# Board review E: Rusty's screens

Boards: Tasks, Knowledge, BrainPage, RustyTabs, RustyServer, MoreTabs, MoreForms. Code paths are under `/srv/stacks/marley_ide/crates/`.

**About the shots.** Three of the named shots don't show what their names say. `t2-04-brain.png` is a copy of the Tasks tab, and `tour-15-tasks.png`, `tour-16-brain.png` and `tour-17-knowledge.png` all show the Component Library. So the Brain tab, a brain page, the Knowledge panel with a page open, Secrets, Rusty's Server, every form and every picker below were judged from the code only ("not seen in a run"). Only Tasks (`t2-03`), Graph (`t2-05`), Decisions (`t2-06`), Memory (`t2-07`, empty), Skills (`t2-08`, empty) and Knowledge (`t2-09`, no page open) were seen.

**One gap on several boards.** The Tasks, Knowledge and BrainPage boards each draw context chips in the bottom row for the Rusty tab in front. Marley's bottom-row focus item, `marley_workbench/src/focus_row.rs:1-6, 33-38`, knows only files, terminals and threads, so a Rusty tab shows nothing there. Each board's section lists it once.

## Tasks

- **Rusty column, Decisions due card buttons.** Board: a filled accent **Follow Up**, which opens the follow-up form, and an outlined **Open the page**. Marley: a plain **Decisions** button that opens the Decisions tab, and a plain Open the page (`t2-03-tasks.png`; `marley_workbench/src/rusty/tasks_tab.rs:1120-1139`). #845's spec said "Follow Up and Open the page". Kind: look, also wording and behavior. Size: S.
- **Decisions due card, rest.** Board: "1 overdue" sits at the right of the card's heading, and a footnote reads "Ask before a choice, record it when made, follow up when its day comes." Marley: "1 overdue" is a line under the heading and there is no footnote (`t2-03`; `tasks_tab.rs:1157-1172`). Kind: look. Size: S.
- **Today card** (TODAY, Open note, today's note text, a "Capture to today" field). Marley: none. Kind: deferred, #845 ("the daily note has no read tool in Marley yet"). Size: M.
- **Task rows: note line, due chip (red when late), and a per-row action button** (Ask an agent, Open thread, Open canvas). Marley: the title only (`t2-03`; `tasks_tab.rs:1220-1300`). Kind: deferred, #845 (`UserTask` has no note or due date; "Ask an agent" was left out). Size: M, and it needs Rusty fields.
- **Done date.** Board: a done row ends with "done Monday", and the header says "1 done this week". Marley: a "Done" heading over struck-through rows with no date, and the header says "1 done" with no time window. `UserTask` has no completion time (`marley_rusty/src/tasks.rs:49-63`). Kind: missing; it needs Rusty. Size: S on Marley's side.
- **Add field.** Board: a tall rounded field with a + icon and the placeholder "Add a task, or tell Rusty: remind me Friday to…", so typed words become a dated task. Marley: a plain box, "Add a task and press Enter" (`tasks_tab.rs:539`). Kind: missing for the "tell Rusty" parsing (it depends on due dates, see above); look for the field. Size: M.
- **Task rows as cards.** Board: each open task is a bordered, rounded card with padding. Marley: flat `ListItem` rows (`t2-03`; `tasks_tab.rs:1244`). Kind: look. Size: S.
- **Header and lists column.** Board: a 24 px list title, an uppercase "LISTS" heading, and a rounded-square colour swatch per list. Marley: a Large label (about 16 px), a "Lists" heading, and a round dot (`t2-03`; `tasks_tab.rs:1002, 1191, 970`). Kind: look. Size: S.
- **Memory card when empty.** Board: memories as lines. Marley: with no memories the card shows only its heading, with no empty line (`t2-03`; `tasks_tab.rs:1174`). Kind: look. Size: S.
- **Bottom row.** Board: chips "Tasks · Work" and "1 decision overdue" (red), and "3 open" at the right. Marley: nothing for the Tasks tab (`t2-03`; `focus_row.rs`). Kind: missing. Size: S.

## Knowledge

- **"Search the brain, or ask it."** Board: the panel's search also asks the brain a question. Marley: "Search the brain…" runs `brain_search` only; nothing found for `brain_ask` or a semantic ask in `rusty/` (`knowledge_panel.rs:158`). Kind: missing. Size: M.
- **Following card.** Board: a page-type tile ("Dc") before the title and path, and tags as `#tag`. Marley: a Large title over the slug with no type tile, and tags drawn as "#tag N" with page counts (`knowledge_panel.rs:730-758`). Not seen in a run. Kind: look. Size: S.
- **Layout.** Board: the Knowledge column at the left, the local graph in the middle, graph settings in a fixed right column. Marley: Knowledge is a dock panel (or a panel tab, `t2-09-knowledge.png`). Graph settings are a floating card over the canvas's top-right corner with a hide button (`t2-05-graph.png`; `graph_tab.rs:1543-1650`). Kind: look. Size: S–M.
- **Graph legend.** Board: a legend row under the graph (decision, note, project, and the "decided from" edge swatch). Marley: the legend sits at the bottom of the floating settings card (`t2-05`). Kind: look. Size: S.
- **Forces.** Board: Link distance and Repel sliders shown open. Marley: they sit in a collapsed "Forces" group with Groups and Display (`t2-05`). Kind: look; behavior is a superset. Size: S.
- **Bottom row.** Board: "Graph · local", "depth 2 · 9 pages" and "embedded 2,140 of 2,212". Marley: nothing for the Graph tab (`t2-05`; `focus_row.rs`). Kind: missing. Size: S.

## BrainPage

- **Linked here.** Board: the outline column ends with a "Linked here" list of backlinks. Marley: only "On this page" (`page.rs:1640-1688`). Not seen in a run. Kind: deferred, #875 ("the page view reads no backlinks; the Knowledge panel shows them"). Size: S.
- **Properties as coloured pills.** Board: status "decided" green, `follow_up_by` "· overdue" red, tags teal. Marley: plain text values and list items as uncoloured `Chip`s (`properties.rs:65-81`). Kind: look. Size: S–M.
- **Task checkboxes in the body.** Board: checkboxes you can tick ("Rusty cards on Home", "Follow up by …"). Marley: Zed's markdown draws them, but the page sets no `on_checkbox_toggle` (`page.rs:1715-1740`; hook at `markdown/src/markdown.rs:1881`), so a click does nothing in read mode. Kind: missing. Size: S.
- **Favourites glyph.** Board: a ★ before each favourite. Marley: an icon by bookmark kind (file, folder, search, heading) (`brain.rs:1339-1356`). Kind: look. Size: S.
- **Outline toggle.** Board: a text button "Outline" with a pressed state. Marley: an icon-only `ListTree` button (`page.rs:1354-1366`). Kind: look. Size: S.
- **Bottom row.** Board: "Brain", the page path, "★ favourite" and "Rusty connected". Marley: nothing for the Brain tab (`focus_row.rs`). Kind: missing. Size: S.

## RustyTabs

- **Memory.** Matches: the add row with category and Low/Normal/High, importance chips (#876), and "category · who · when" (`t2-07-memory.png`; `memory_tab.rs:393-428`). No gaps.
- **Skills: a staged skill's card in the list.** Board: the row reads "auto-commit-notes · staged by an agent" and shows the scan finding ("Scan: runs git push without asking") with Reject and Approve Anyway on the row. Marley: the list row has a name, a pending chip and "auto". The scan runs only from a Scan button, and its findings, Approve, Approve Anyway and Reject appear in the detail pane (`skills_tab.rs:660-692, 720-786, 811-826`). Not seen with data (`t2-08` is empty). Kind: look and flow. Size: M.
- **Skills: Run on the script row.** Board: "$ prune-target" with Run on the row. Marley: Run is in the script's detail pane only (`skills_tab.rs:695-716, 856-863`). Kind: look. Size: S.
- **Secrets: unlock countdown.** Board: "Unlocked · locks in 4 m". Marley: "Unlocked", then Lock and Change PIN, with no time left shown (`secrets_tab.rs:657-668`). Not seen in a run. Kind: missing. Size: S.
- **Secrets: locked card.** Board: a lock icon, the PIN rule in one line beside it, and a "PIN" label inside the field. Marley: the field and Unlock, with "Unlocking shows, edits, copies and deletes values…" under them; the "never type the PIN to an agent" rule is in the page's about text (`secrets_tab.rs:620-653, 40-43`). Kind: look. Size: S.
- **Capture: where it goes.** Board: each capture form says where the line lands ("Appends to daily/2026-10-20", "Appends to inbox", "Rusty fetches it and keeps it as a source page"). Marley: a title ("Capture to Today's Note", and so on), the field, and "Enter captures, Escape closes", with no line naming the target page or what a URL capture does (`capture.rs:143-156, 311-344`). Not seen in a run. Kind: look. Size: S.
- **Import: name clashes.** Board: "3 names clash and get a suffix. Nothing is written until you choose Import." Marley: Rusty's plan says collisions are "left as they are" (skipped) and "The vault is read and never written", with no "nothing is written until Import" line (`marley_rusty/src/capture.rs:171-183, 193-196`). Kind: look and wording; the suffix behavior is Rusty's `brain_import`. Size: S.

## RustyServer

- **Status card.** Board: four facts in a card: a Connected dot with "checked 3 s ago", Connection "embedded · stdio", Program "~/.local/bin/rusty-mcp 0.1.1", and Rusty's tools for agents "on · as 'rusty'". Marley: one `AiSettingItem` row with "rusty-mcp", the server's name and version, and the transport line ("embedded: <path>") under it. There is no "checked N s ago" and no line on whether agents get Rusty's tools (`marley_workbench/src/rusty.rs:913-965`). Not seen in a run. Kind: look; missing for the last check time and the agent-tools state. Size: M.
- **Brain search card.** Board: "2,140 of 2,212 pages embedded · ollama · nomic-embed-text", a progress bar, and "72 wait their turn". Marley: one muted sentence, "Embedding with X: N pages in M chunks have vectors; K waiting.", with no bar (`rusty.rs:1011-1018`; `marley_rusty/src/settings.rs:205-219`). Kind: look. Size: S.
- **"When it drops."** Board: the drop reason followed by "Trying again in 4 s", with the 1, 2, 4… 60 s schedule explained. Marley: the backoff exists (`rusty.rs:420-443`), but the page shows only the reason in red, with no countdown and no Try Now; Try Now is only on Rusty's tabs (`rusty.rs:458-470`). Kind: missing. Size: S.
- **Header line.** Board: "Settings › Marley › Rusty. Marley keeps no copy of Rusty's settings; it reads them from Rusty." Marley: "Marley's connection to rusty-mcp, Rusty's MCP server." (`rusty.rs:971-975`). Kind: look (wording). Size: S.
- **The ten settings rows.** Board: a plain-English name per row ("Brain vault", "PIN timeout"…), its about line, the key in mono, the value at the right, and default values muted. Marley: the key in mono beside an edit field, then "about. Default: X." under it, with no plain-English name. `KnownSetting` has no name field (`rusty.rs:1050-1066`; `marley_rusty/src/settings.rs:87-150`). Kind: look. Size: S.

## MoreTabs

- **Decisions with Follow Up.** Built (#874; `t2-06-decisions.png`). No gaps.
- **Harness session tab** (#878) and **scheduled task runs** (#886). Built. No gaps found in code.
- **Component tab from `show card.json`.** #849 left it out as already drawn by the same renderer. Not seen in a run; no gap found.
- **Draft tab header.** Board: lettered agent tiles, "M Marley → CC Fix the rail test". Marley: a Large text title, "Draft from {from} to {to}", with no tiles or arrow (`marley_workbench/src/drafts.rs:327-330`). Not seen in a run. Kind: look. Size: S.
- **Draft tab footnote and buttons.** Board: under the text, "Nothing goes until you send it. This tab is the review.", then Drop (quiet) and Send (primary) at the right. Marley: the note sits above the editor ("Nothing is sent until you press Send. Edit it first if you like."), and Send comes first, then Drop, at the left (`drafts.rs:331-366`). Kind: look. Size: S.

## MoreForms

- **Add or edit an MCP server.** Board:
  - Name and Where side by side, with Where a dropdown of scopes;
  - Command/URL as one framed toggle;
  - the Authorization header masked, "set; type to replace", with Clear;
  - the footnote "Claude Code reads its servers at its next start.";
  - Cancel and Save.

  Marley: the fields stacked, Where as a row of filled buttons, and Command and URL as two separate filled buttons. There is no Cancel button (Save only, Escape closes) and no footnote (`marley_workbench/src/mcp_server_form.rs:774-819, 917-978`). Masked headers with Clear do exist (`:845`). Not seen in a run. Kind: deferred, #846 ("the server form… keep their tabs and forms"); #871 handed it to "their screens' W5 tickets" and none has done it (look-plan W5 row: "the server form" not marked done). Size: S–M.
- **Agent asks to start another agent.** Built (#871; `agent_control.rs:342`, `delegation.rs:213`). No gaps.
- **Harness session picker rows.** Board: profile name over its file ("profiles/builder.toml"). Marley: the name only; placeholder "A profile to open a harness session from…" (`harness.rs:2203, 2262-2273`). Kind: look. Size: S.
- **"New thread with context: which agent" picker rows.** Board: the agent over a line ("the old thread's own agent", "Thread"). Marley: the name only; placeholder "The agent the new thread runs on…" (`handoff.rs:312, 365-376`). Kind: look. Size: S.
- **"Send the selection to" picker rows.** Board: two lines, the agent and thread, then the state and project ("ready · marley_ide", "working · nothing is sent"), with "Copy the reference" over the file:lines. Marley: one line, "Agent · project · status", with a Copy row (`send_selection.rs:83-90, 824-842`). Kind: look. Size: S.
- **Reopen a conversation picker.** Matches (title, detail, time; `threads_page.rs:1047`). No gaps.
- **Forget a memory.** Board: an alert "Forget this memory?" whose body is the quoted memory plus "Rusty's agents stop reading it.", with Cancel and **Forget**. Marley: the same title with the memory's first line, and the buttons **Delete** and Cancel (`memory_tab.rs:668-674`). Kind: look (wording). Size: S.
- **Edit a memory.** Board: the destructive button reads "Delete…". Marley: "Delete" (`memory_tab.rs:800`). Kind: look (wording). Size: S.
- **New skill.** Board: "Stage it for approval" and **Make**. Marley: "Stage for approval instead of making it active" and **Create**, plus a Body field (`skills_tab.rs:1136-1166`). Kind: look (wording only). Size: S.
- **Link a project to a brain page.** Board: a footnote under the match saying what Link does ("Adds /srv/stacks/marley_ide to the page's path"). Marley: the picker lists title and slug with no footnote (`rusty/project.rs:404, 583-610`). Kind: look. Size: S.
