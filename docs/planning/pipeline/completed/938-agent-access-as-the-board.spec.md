---
pipeline_id: 85f61c07-82e9-4772-b920-dc5acc2c88c8
ticket: docs/planning/tickets/open/TICKET-938-agent-access-as-the-board.md
status: Phase 4 — Complete PASS
title: What agents may do, as the board draws it
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/F-tools.md, docs/marley/look-plan.md]
---

## Title
The editor consent question as the AgentAccess board draws it (the agent's tile, "wants to edit
<file>", the task, the change as −/+ lines, the footnote), a "Kept from agents" section on the
Agent Activity tab, and the board's words for the Agent Control modes and Outside Clients' levels.

## Scope
### In
- **The edit question's headline and tile** (Marley crate, `agent_control.rs`: `Asked`,
  `Question`, `admit_asking`, `show`; `asks.rs`: `Ask`). `Asked` gains an optional headline, so
  `editor_edit` asks "<who> wants to edit <file name>"; every other tool keeps "<who> wants to use
  <tool>". The card's title row leads with a lettered tile for the agent: CC for Claude Code, CX
  for Codex, Z for Zed's agent, else the first letter of the caller's words.
- **The task line** (`agent_control.rs`, `Asked`; `editor_tools.rs`). Under the headline, muted:
  the asking terminal's title, then " · " and the project, when the call comes from a Marley
  terminal; else the project alone. It replaces the "Project" row for this question.
- **The change as −/+ lines** (`asks.rs`, a new `Ask::change`; `editor_tools.rs`, the
  `editor_edit` branch of `handle`). The question shows `old_text`'s lines prefixed "−" on the
  deleted background and `new_text`'s lines prefixed "+" on the created background, in the
  buffer font, at most 12 lines with "… N more lines" after. It replaces the line "change
  <file>". With `replace_all`, a muted line "Every match in the file." follows.
- **The footnote** (`editor_tools.rs`). `editor_edit`'s question ends with "One undo step, left
  unsaved. Saving asks again."
- **"Kept from agents"** (Marley crate, `agent_activity.rs`, `Render for AgentActivityView`). A
  section under the Stop switch: the heading "Kept from agents", the globs of
  `marley.agent_control.secret_globs` as chips, and the note "Keys and tokens in terminal output
  become [redacted: token] before an agent reads them. Your terminal still shows them." While
  `marley.redact_secrets_for_agents` is off, the note reads "Redaction is off: agents read terminal
  output as it is." A muted last line: "The list is marley.agent_control.secret_globs in
  settings.json."
- **"Ask Every Time"** (Zed crate path, `crates/settings_content/src/marley.rs`,
  `MarleyAgentControlMode::AskEvery`). A `#[strum(serialize = "ask_every_time")]` on the variant,
  so the Settings window's buttons read "Ask Every Time". The settings value stays `ask_every`
  (serde's name is unchanged).
- **Outside Clients' levels** (Marley crate, `clients.rs`, the power buttons of
  `BrowserClientsModal`). Read / Read and Write / Read, Write and Run.

### Out (explicitly deferred)
- **The pairing card, a follow-up ticket.** The review lists, all missing or look: "“<name>” asks
  to pair with Marley" as the title; "From <folder> over SSH. Check the code matches what the
  command printed."; the code in six digit boxes; three grants as a radio list (Read / Read and
  Write / Read, Write and Run), each with its line ("The read tools only, as before pairing." /
  "Also editors, the Browser tab, threads, palette actions, settings proposals." / "Also typing
  and running in terminals, harness seats, starting agents."); Refuse and Pair; "No answer in two
  minutes pairs nothing." (`clients.rs:676-712`). "Over SSH" needs the bridge to say how it was
  reached (a new `Caller` field in `marley_mcp` and a header from `marley-mcp-bridge`), and the
  radio list needs a picked grant held in the notification. That is its own M.
- Agent Control as a block of areas with mode pills and "Stop Agent Control · every tool that
  acts refuses" under it: the modes stay rows in the Settings window, the Stop button on the Agent
  Activity tab (not on this ticket's list).
- Editing the secret globs on a screen: deferred by #844 ("lists and maps stay in settings.json").
- A diff for tools other than `editor_edit`: no other tool carries old and new text.

## Reference (§20)
Upstream Zed for the change's look: Zed draws removed and added text on
`status().deleted_background` and `status().created_background` (`crates/agent_ui/src/inline_assistant.rs`,
`crates/editor/src/element.rs`), with `crates/ui`'s `Color::Deleted` and `Color::Created`; the
question draws the same colours.
The rest is N/A — Marley-specific: Marley's Agent Control (#704, #705, #871), its redaction
(#516) and Outside Clients (#524, #811).

### Prior art
- `crates/marley_workbench/src/agent_control.rs:192-196` `Asked { what, rows, footnote }`;
  `:239-245` the headline is always "<who> wants to use <tool>"; `:318-334` `show` adds the
  "Project" row and the footnote. Gap holds at HEAD.
- `crates/marley_workbench/src/editor_tools.rs:54-55` `editor_edit` asks with `edit_words`
  (`:322-332`, "change <file>[, every match]"); `:338-355` the edit reads `old_text` and
  `new_text` from the arguments, so both are in hand before the question.
- `crates/marley_workbench/src/asks.rs:20-130` `Ask`: title, lines, label rows, footnote,
  buttons; no tile and no diff block.
- `crates/marley_workbench/src/click_pause.rs:56-106` `Who::of`: the caller's words (a client's
  name, "Claude Code", "Zed's agent", "An agent in a terminal", "An agent").
- `crates/marley_workbench/src/mcp.rs:958` `caller_terminal`: the asking terminal's view, whose
  title is the task line.
- `crates/marley_workbench/src/home_page.rs:925-950` `cli_glyph` and `glyph_tile` (#903): the
  lettered tile; private to Home today.
- `crates/marley_workbench/src/agent_activity.rs:323-360` `render_state` (the Stop switch);
  `:433-447` the tab's body. No "Kept from agents" (grepped "Kept from agents", "Your terminal
  still": not found).
- `crates/marley_workbench/src/editor_tools.rs:182-195` `secret_globs` reads
  `marley.agent_control.secret_globs`; `assets/settings/default.json:1983-1993` the eight globs;
  `crates/marley_workbench/src/marley_workbench.rs:477, 905-907` `redact_secrets`;
  `crates/marley_mcp/src/redact.rs` the `[redacted: …]` marks.
- `crates/settings_content/src/marley.rs:598-609` `MarleyAgentControlMode` derives
  `strum::VariantNames` with serde's `rename_all`; `crates/settings_ui/src/settings_ui.rs:5275-5310`
  draws a Marley enum's buttons from `VariantNames`, title-cased.
- `crates/marley_workbench/src/clients.rs:941-948` the power buttons read Read / Write / Run;
  `crates/marley_mcp/src/clients.rs:92-98` `Power::words` already says "read and write" and
  "read, write and run".
- Specs: #704 (the modes and the question), #705 (`editor_edit`, its scenario), #871 (`Asked`'s
  rows and footnote), #844 (lists stay in settings.json).

## UI proof
Scenario `script/e2e/938-agent-access-as-the-board.sh` (sway). Setup: a scratch repository with
`src/main.rs` open; #705's scripted MCP client run in Marley's terminal through the plugin's
bridge (L-claude-871), so the caller is that terminal; `marley.agent_control.editors` on
`ask_every`; one paired client written to the run's `clients.json` for Outside Clients. No real
agent CLI.
- `938-01-asked`: the client calls `editor_edit` replacing `let answer = 42;` with `let answer =
  43;`: the question with its tile, "wants to edit main.rs", the task line, "− let answer = 42;" and
  "+ let answer = 43;", the footnote and the three buttons (REQ-001, REQ-002, REQ-003, REQ-004).
- `938-02-day`: the question in Marley Day (REQ-001 to REQ-004).
- `938-03-kept`: `marley: open agent activity`: "Kept from agents" with eight chips and the note
  (REQ-005).
- `938-04-unredacted`: `marley.redact_secrets_for_agents` set false: the note says redaction is off
  (REQ-005).
- `938-05-modes`: the Settings window searched for "editors": the Agent Control row reads Off / Ask
  Every Time / Ask First / Allow (REQ-006).
- `938-06-clients`: `marley: outside clients`: the paired client's levels read Read / Read and
  Write / Read, Write and Run (REQ-007).

## Locked-In Decisions
- D1 — Only `editor_edit` gets the edit headline, the diff and the footnote; every other tool's
  question stays as #704 and #871 made it.
- D2 — The diff is the call's own `old_text` and `new_text`, line by line, not a computed diff of
  the buffer: it is exactly what the agent asked to change.
- D3 — "Kept from agents" sits on the Agent Activity tab, beside the Stop switch: Marley's screen
  for what agents may do. The list stays in settings.json (#844).
- D4 — "Ask Every Time" changes the label, never the stored value: existing settings keep working.
- D5 — The lettered tile moves out of `home_page.rs` into one shared function the question and
  Home both call; no second copy.
- D6 — The pairing card is split off (Out): it needs a new `Caller` field across `marley_mcp`
  and the bridge.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `editor_edit` and the editors' mode asks, the question shall lead with the agent's lettered tile and read "<who> wants to edit <file name>". | `938-01-asked`, `938-02-day` |
| REQ-002 | WHEN the edit question shows, it shall show the asking terminal's title and the project as its task line. | `938-01-asked`, `938-02-day` |
| REQ-003 | WHEN the edit question shows, it shall show `old_text`'s lines as "−" lines on the deleted background and `new_text`'s as "+" lines on the created background. | `938-01-asked`, `938-02-day` |
| REQ-004 | WHEN the edit question shows, it shall end with "One undo step, left unsaved. Saving asks again." above its buttons. | `938-01-asked`, `938-02-day` |
| REQ-005 | The Agent Activity tab shall show "Kept from agents" with each secret glob as a chip and the redaction note, which says redaction is off WHILE `marley.redact_secrets_for_agents` is false. | `938-03-kept`, `938-04-unredacted` |
| REQ-006 | The Settings window shall label the `ask_every` mode "Ask Every Time", and a setting of `ask_every` shall keep its meaning. | `938-05-modes`, the review of the diff |
| REQ-007 | Outside Clients shall label a paired client's levels Read, Read and Write, and Read, Write and Run. | `938-06-clients` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `agent_control.rs`, `asks.rs`, `editor_tools.rs`, `agent_activity.rs`,
  `clients.rs`, `home_page.rs` (the shared tile); the `settings_content` attribute and its ledger
  row; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
