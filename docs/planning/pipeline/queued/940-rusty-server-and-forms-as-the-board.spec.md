---
pipeline_id: 7d6bd891-6325-48e3-86a4-8256c69994a4
ticket: docs/planning/tickets/open/TICKET-940-rusty-server-and-forms-as-the-board.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Rusty's Server, Secrets and Skills as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/E-rusty.md, docs/marley/look-plan.md]
---

## Title
Rusty's Server page, the Secrets tab and the Skills tab as the RustyServer and RustyTabs boards
draw them: the status card with its last check and the agents' tools, the retry countdown with
Try Now, the embedding bar, plain names on Rusty's settings; "locks in N m" and the locked card;
a staged skill's scan result and buttons on its row, Run on a script's row, and the New skill
form's words.

## Scope
### In
- **Header line** (Marley crate, `rusty.rs`, `Render for RustyServerView`). Under "Rusty's
  Server": "Settings › Marley › Rusty. Marley keeps no copy of Rusty's settings; it reads them
  from Rusty."
- **Status card** (`rusty.rs`, `RustyServerView`; the `Rusty` global). A bordered card of four
  label and value rows: a state dot and "Connected" (or the state's word) with "checked N s ago";
  Connection (the transport, "embedded · stdio" or the service URL); Program (the path and the
  server's version); Rusty's tools for agents ("on · as “rusty”" or "off"). The `Rusty` global
  keeps when the last connect or ping answered; the card ticks once a second while it shows.
- **When it drops** (`rusty.rs`, `keep`, `RustyServerView`). While the connection is down, the
  card shows the reason, then "Trying again in N s" counting down, and Try Now (#894's
  `try_now`). The `Rusty` global keeps when the next try starts. A muted line under it: "Marley
  tries again after 1, 2, 4… up to 60 s."
- **The dropped card on Rusty's tabs** (`rusty.rs`, `render_dropped`). The card's reason is
  followed by "Trying again in N s." from the same next-try time, as the States board draws it
  ("rusty-mcp exited. Trying again in 4 s."). #943 left this countdown to a follow-up; it rests
  on the state this ticket adds.
- **Brain search** (`rusty.rs`, `settings_section`; `marley_rusty/src/settings.rs`,
  `SemanticStatus`). "P of T pages embedded · provider · model", a bar of P over T, and "N wait
  their turn" while pages wait; T is the pages with vectors plus the pages waiting. Full-text only
  keeps its sentence.
- **Plain names on the ten settings** (Marley crates: `marley_rusty/src/settings.rs`,
  `KnownSetting` gains `name`; `rusty.rs`, `known_row`). Each row: the name (Brain vault, Notes,
  Embedding provider, Embedding model, Ollama URL, PIN timeout, Skills, Skills folder,
  Auto-enrich, Default workflow) over its about line, the key in mono under them, the field at the
  right; an unset value shows its default muted in the field. The page's "Rusty's Settings" and
  "Other Stored Keys" read "Rusty's settings · the ten its app lists" and "Other stored keys".
- **Secrets: locks in N m** (Marley crate, `rusty/secrets_tab.rs`, the unlocked block at
  `:654-668`). "Unlocked · locks in N m" (seconds under a minute), counting down from the unlock's
  `expires_in_seconds`.
- **Secrets: the locked card** (`secrets_tab.rs`, `:620-653`). A lock icon with "Names show to
  anyone. A value needs your PIN. Never type the PIN to an agent." beside it; the PIN field with a
  "PIN" label inside it, and Unlock. The lockout line stays when it applies.
- **A staged skill's row** (Marley crate, `rusty/skills_tab.rs`, `render_skill_row`, the list's
  read). A pending skill's row reads "<name> · staged by an agent" and shows its scan result:
  the first finding in the warning colour ("Scan: <finding>") or "Scan: clean". The row carries
  Reject, and Approve Anyway when the scan found something, else Approve. Marley scans each
  pending skill once when the list is read (`skill_scan`), as the Scan button does.
- **Run on a script's row** (`skills_tab.rs`, `render_script_row`). An active script's row ends
  with Run, which does what the detail pane's Run does.
- **New skill's words** (`skills_tab.rs`, `:1136-1166`). The checkbox reads "Stage it for
  approval" and the button **Make**.

### Out (explicitly deferred)
- **MoreTabs and MoreForms, a follow-up ticket.** From the review, all look unless named:
  - Capture says where the line lands ("Appends to daily/<date>", "Appends to inbox", "Rusty
    fetches it and keeps it as a source page") (`capture.rs:143-156, 311-344`);
  - Import's "Nothing is written until you choose Import." (the suffix for clashing names is
    Rusty's `brain_import`: Rusty-side, Out of Marley's tickets);
  - the Draft tab's agent tiles and arrow, its footnote under the text, Drop then Send at the
    right (`drafts.rs:327-366`);
  - two-line rows in the harness session picker (the profile's file), the new-thread agent picker
    and the send-selection picker (`harness.rs:2262-2273`, `handoff.rs:365-376`,
    `send_selection.rs:824-842`);
  - Memory's "Forget this memory?" with "Rusty's agents stop reading it." and **Forget**, and the
    edit form's "Delete…" (`memory_tab.rs:668-674, 800`);
  - the Link a project footnote ("Adds <folder> to the page's path") (`rusty/project.rs:583-610`);
  - the MCP server form's Cancel and its footnote "Claude Code reads its servers at its next
    start." (`mcp_server_form.rs:917-978`), deferred by #846 and handed on by #871; the list says
    to build them, in the follow-up.
- The bottom row for Rusty's tabs: #909 (the bottom row for other screens).

## Reference (§20)
N/A — Marley-specific: Marley's connection to Rusty and its Server page (#643, #666, #894), and
Rusty's Secrets and Skills tabs (#665, #667, #876). Rusty is a separate program across MCP (§20, the
brain boundary); Marley only shows what Rusty's tools answer.

### Prior art
- `crates/marley_workbench/src/rusty.rs:913-1000` `RustyServerView::render`: one
  `AiSettingItem` with the server's name and the transport under it; `:971-975` the header line
  "Marley's connection to rusty-mcp, Rusty's MCP server." Gap holds at HEAD.
- `rusty.rs:416-443` `keep`: the backoff of 1 s doubling to `BACKOFF_MAX_S` (60), its `wait`
  computed and slept, never stored; `:445-456` `try_now` wakes it; `:458-470` `render_dropped`
  (Rusty's tabs) has Try Now and no countdown. `:573-597` the ping every `PING_EVERY` (5 s);
  no time of the last answer is kept.
- `rusty.rs:130-155` `RustySettings { source, agent_tools }`: whether agents get Rusty's tools,
  and `:14-16` the context server's name `rusty`.
- `rusty.rs:1003-1035` `settings_section`; `crates/marley_rusty/src/settings.rs:167-219`
  `SemanticStatus { provider, stats: { model, pages, chunks }, stale }` and its one `line()`.
- `marley_rusty/src/settings.rs:85-150` `KnownSetting { key, about, fallback }` and `KNOWN`;
  `rusty.rs:1037-1056` `known_row`.
- `crates/marley_workbench/src/rusty/secrets_tab.rs:359` the unlock's lifetime from
  `expires_in_seconds` (`marley_rusty/src/secrets.rs:49`); `:620-668` the locked and unlocked
  blocks; `:40-43` the PIN rule in the about text.
- `crates/marley_workbench/src/rusty/skills_tab.rs:660-692` `render_skill_row` (name, pending
  chip, "auto"); `:695-716` `render_script_row`; `:454-470` `scan` calls `SKILL_SCAN` for one
  skill; `:720-826` the detail pane's Approve, Approve Anyway (`blocked_by_scan`) and Reject;
  `:855` Run; `:94-130` `run_script`.
- The stand-in `crates/marley_rusty/stand_in/rusty-mcp` answers `settings_list`,
  `brain_semantic_status`, `skill_scan`, `skill_approve`, `skill_reject`, `secret_pin_status`,
  `secret_unlock` (`:50-116`, `:2144`).
- `crates/ui`'s `AiSettingItem` (the row the page uses today) and Home's `state_tile` (#903) for
  the state dot.
- Specs: #643 (the connection), #666 (Rusty's settings on the page, its scenario's keymap to
  open the page), #894 (Try Now and its failing wrapper), #876 (the tabs and its scenario's
  skills, scripts and secrets).

## UI proof
Scenario `script/e2e/940-rusty-server-and-forms-as-the-board.sh` (sway). Setup: Rusty on through
#894's wrapper around `marley_rusty`'s stand-in `rusty-mcp` (it fails while
`$E2E_WORK/rusty-down` exists), #666's `settings.json` and `semantic.json` and its keymap entry
for the Server page, #876's skills (one staged with a script that runs `git push`, one active
with a script) and made-up secrets with a made-up PIN. No agent CLI; the stand-in only.
- `940-01-server`: the Server page connected: the header line, the status card, the brain search
  bar, the ten rows with plain names (REQ-001, REQ-002, REQ-004, REQ-005).
- `940-02-dropped`: the flag file made and the stand-in ended: the reason, "Trying again in N s"
  and Try Now (REQ-003).
- `940-03-back`: the flag gone, Try Now clicked: Connected, checked a few seconds ago (REQ-003).
- `940-04-locked`: `rusty: open secrets`: the locked card (REQ-007).
- `940-05-unlocked`: the PIN typed, Unlock: "Unlocked · locks in 5 m" (REQ-006).
- `940-06-skills`: `rusty: open skills`: the staged skill's row with "staged by an agent", its
  scan finding, Reject and Approve Anyway; the active script's row with Run (REQ-008, REQ-009).
- `940-07-new-skill`: New skill: "Stage it for approval" and Make (REQ-010).
- `940-09-dropped-tab`: while dropped, `rusty: open memory`: the card's reason, "Trying again in N
  s." and Try Now (REQ-011).
- `940-08-day`: the Server page and Skills in Marley Day (REQ-001, REQ-002, REQ-004, REQ-005,
  REQ-008).

## Locked-In Decisions
- D1 — The `Rusty` global keeps two instants: when Rusty last answered (a connect or a ping) and,
  while down, when the next try starts. The page draws from them and ticks once a second while it
  is shown; nothing else polls.
- D2 — "Rusty's tools for agents" reads Marley's own `marley.rusty.agent_tools` (as
  `RustySettings` resolves it), not Rusty.
- D3 — The embedding total is the pages with vectors plus the pages waiting; Marley asks Rusty for
  nothing new.
- D4 — Each pending skill is scanned once per read of the list; a scan that fails shows "Scan
  failed" and the row keeps Reject and Approve (Rusty's own check still runs on approve).
- D5 — The settings rows keep their fields and Enter-to-write (#666); only the names, layout and
  the default's look change.
- D6 — MoreTabs and MoreForms are split off as a follow-up (Out); this slice is the Server page,
  Secrets and Skills.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Rusty's Server page shall open with "Settings › Marley › Rusty. Marley keeps no copy of Rusty's settings; it reads them from Rusty." | `940-01-server`, `940-08-day` |
| REQ-002 | WHILE Rusty is connected, the page shall show a card with the state and "checked N s ago", the connection, the program with its version, and whether agents get Rusty's tools. | `940-01-server`, `940-08-day` |
| REQ-003 | WHILE Rusty's connection is down, the page shall show the reason, "Trying again in N s" counting down, and Try Now, which tries at once. | `940-02-dropped`, `940-03-back` |
| REQ-004 | WHILE an embedding provider is set, the page shall show "P of T pages embedded", the provider and model, a bar of P over T, and how many wait. | `940-01-server`, `940-08-day` |
| REQ-005 | The page shall show each of Rusty's ten settings with its plain name over its about line, its key in mono, and an unset value's default muted. | `940-01-server`, `940-08-day` |
| REQ-006 | WHILE the Secrets tab is unlocked, it shall show "Unlocked · locks in N m", counting down. | `940-05-unlocked` |
| REQ-007 | WHILE the Secrets tab is locked, it shall show a lock icon with the PIN rule beside it, a field labelled PIN, and Unlock. | `940-04-locked` |
| REQ-008 | WHERE a skill is staged, its row in the Skills list shall read "<name> · staged by an agent", show its scan result, and carry Reject and Approve Anyway (or Approve when the scan is clean). | `940-06-skills`, `940-08-day` |
| REQ-009 | WHERE a script is active, its row shall carry Run, which runs it as the detail pane's Run does. | `940-06-skills` |
| REQ-010 | The New skill form shall read "Stage it for approval" and **Make**. | `940-07-new-skill` |
| REQ-011 | WHILE Rusty's connection is down, the card on Rusty's tabs shall say "Trying again in N s." after its reason, counting down. | `940-09-dropped-tab` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `rusty.rs`, `marley_rusty/src/settings.rs`, `rusty/secrets_tab.rs`,
  `rusty/skills_tab.rs`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
