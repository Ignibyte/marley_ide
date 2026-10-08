---
status: promoted
created: 2026-10-07
ticket: TICKET-680 to TICKET-684 (phase 1); phases 2 to 5 stay here
pipeline_spec: docs/planning/pipeline/active/680-tool-results-fit.spec.md
---

# The Marley agent, then the manager, then the foreman

Chad, 2026-10-07: "lets make a full plan from beginning to end and assign who we need to run this
(harness or you or whoever). I want to go Marley->Manager (i have one now)->foreman". And: "does
the user have to create Marley or can we detect claude or codex and create it automatically".

## Settled in the talk (2026-10-07)

- **Marley builds no manager.** The manager and the foreman are seats in the harness, on this box
  or another; Marley is how a person sees them and talks to them. Whatever has to keep running
  while Marley is closed does not live in Marley.
- **Across boxes the hub is the Brain.** Harness D165: the Brain directs the agents on every
  machine, and each machine's harness runs its own.
- **Opinionated, in three levels.** A harness and a Brain whose MCP follows Marley's SDK contracts:
  full integration. A harness and no Brain: seats, terminals, live state and the manager thread
  work, and Marley says the work records are missing. No harness: none of this shows.
- **The manager talks to the person in the Agent Panel.** The harness ships `rh acp`, a program
  that speaks ACP to Zed and reads and posts the manager's thread (harness TICKET-106). The
  manager stays a Claude Code session in the harness's terminal, on the user's own login.
- **The Marley agent is desk work, not the harness's.** It explains and configures Marley and
  never edits code. Both routes: Zed's own agent with a "Marley" profile (an API key, a local
  model or Zed's plan), and an ACP agent on the user's own plan. Chad uses Claude Code through
  ACP.

## Who does the work

| Who | Where | How work reaches them |
|---|---|---|
| The Marley session | `/srv/stacks/marley_ide` | Tickets through `/pipeline:plan` to `/pipeline:complete` |
| The harness session | `/srv/stacks/rustal-harness` | Its own tickets; Marley's asks go in its `docs/planning/MARLEY_REQUESTS.md` and by message. None is running on 2026-10-07; Chad opens one |
| The Brain session | `/srv/stacks/rustal-brain` | Its core requests (CR-005, CR-006). None is running |
| UCSOS | `/srv/stacks/ucsos_forge`, the forge box and the five agent boxes | Chad, with the harness session |
| Chad | | The decisions at the end, opening the other sessions, the visual check of each phase |

## Phase 1: the Marley agent

Owner: the Marley session. Nothing in it waits on another repository.

1. **Claude in the Agent Panel works again.** The registry adapter's install is incomplete:
   `node_modules/@anthropic-ai/claude-agent-sdk` is empty and the native binary is missing, and
   Marley's log shows "Cannot find package '…/claude-acp/node_modules/@agentclientprotocol/sdk/
   index.js'" three times on 2026-10-06. Deleting
   `~/.local/share/marley/external_agents/registry/npx/claude-acp` makes Marley install it again.
   Needs Chad's go; not a ticket.
2. **Tool results that fit Claude Code's limit** (T3 survey, item 4). `terminal_read` returns up
   to 256 KiB (`crates/marley_workbench/src/mcp.rs:54`), more than Claude Code's MCP output limit.
   A default near 20 KB with `offset` and `next_offset`, instructions in `initialize`, refusals
   with a code. S. **Done in #680 (2026-10-07).**
3. **Knowledge tools on Marley's MCP server.** `docs_search` and `docs_read` over Zed's docs
   (`docs/src`, `all-actions.md` among them) and Marley's guide, shipped in the build so they match
   the running version; `settings_schema` (a setting's type, default and description, from the
   schema Zed generates); `settings_read` (user and project, and which overrides which);
   `actions_list` (every command and its key). S to M. **Done in #681 (2026-10-07)**, with
   `settings_read` also answering `global`, the value where no project file sets the key.
4. **Changing settings.** `settings_change` and `keymap_change` propose a change, show the diff in
   Marley and apply it on accept, through Zed's settings writer so comments and formatting
   survive. M. **`settings_change` done in #682 (2026-10-07)**, asking through a notification in
   every window and waiting 25 seconds; `keymap_change` done in #686, through Zed's keymap
   updater.
5. **The Marley entry in the Agent Panel.** S to M. **Done.** Claude Code in #683 (2026-10-07),
   with the session meta in `marley.agent_session_meta` and one Zed hunk. Codex (`codex-acp`
   read-only, `CODEX_CONFIG`) and Zed's agent (a `marley` profile) in #687, chosen by
   `marley.assistant.agent`. The offer finds the agent once.
   - Detection that never touches a token: `claude auth status` (JSON with `loggedIn`),
     `codex login status`, and the language-model providers Zed has configured.
   - The choice: the setting if set, else Claude Code, else Codex, else Zed's agent.
   - Claude Code through the adapter with `_meta.claudeCode.options`: a `systemPrompt` and
     `disallowedTools` (Edit, Write, NotebookEdit, Bash), which the adapter accepts
     (`acp-agent.js:312-345`) and Zed does not send today. A small hunk in
     `crates/agent_servers`, listed in `docs/marley/zed-touchpoints.md`. Codex in its read-only
     sandbox. Zed's agent as a "Marley" profile with no file tools.
   - Off until turned on, as every new Marley layer ships. The first time the Agent Panel opens
     with an agent found, it offers the Marley agent once ("Marley can help set itself up, using
     Claude Code 2.1.293, signed in. Turn on"). A `marley.assistant` setting holds the switch and
     the agent.
6. **The same agent in a terminal.** **Done in #684 (2026-10-07).** `claude --append-system-prompt-file … --disallowedTools …
   --mcp-config …` in a Marley terminal tab. S.

Done when Chad asks the Marley agent in the Agent Panel to change a setting, sees the diff,
accepts it, and the agent cannot edit a file.

## Phase 2: the manager on the harness

Chad's UCSOS manager runs in plain tmux today.

The harness session:

1. **A seat in one step.** `rh seat add NAME --role ROLE --agent claude|codex --cwd DIR
   [--model]` writes the profile (today a hand-written 0600 JSON file, `ROOT/profiles/NAME.json`)
   and `rh seat start NAME` opens and supervises it. Roles other than `manager` are kept as labels.
   It runs on a remote box over SSH as the operator, because the harness lets an MCP client name a
   profile but never supply a program (`docs/MCP.md`).
2. **The manager and person thread**, TICKET-106 (open).
3. **`rh acp`** over the thread: the person's messages in, the manager's posts and reports out,
   `session/load` replaying the thread after a restart, confirmations as permission requests. For
   a remote root, `ssh HOST rh acp …`, so no port opens.

The Marley session:

4. **The write side of the harness's MCP.** **Done in #689 (2026-10-07)**, behind
   `marley.harness_writes`:
   - a session's tab answers its question, sends it text and lists its views with Copy;
   - `marley: open harness session` opens one from a profile.

   Harness TICKET-092 (closed 2026-10-07) adds `wait_ms` to `session_send`, which Marley does not
   use yet.
5. **Watching a seat.** **Done in #690 (2026-10-07):** Open on each view of a session's tab,
   local harness only. Marley runs the command `session_surface_to_human` returns (`rh view`,
   `rh attach`, or a Claude or Codex observer) in a terminal pane, typing through the harness's
   controller claim. This is `intake/harness-session-live-terminal.md`; its 2026-10-01 finding (no
   seat had a terminal) needs checking again now that M13 runs Claude Code's own interface in a
   harness terminal.
6. **The New Agent form.** The harness's draft shape (TICKET-109's plan, 2026-10-07):
   - **Commands.** `ssh HOST rh --state ROOT seat add NAME --agent claude|codex --cwd DIR
     [--role R] [--model M] [--binary P] [--attempts N --backoff-ms …] [--no-supervise]`, then
     `seat start NAME`.
   - **Output.** Each prints one JSON object and exits 0. `start` gives `{id, title, profile,
     request, state, opened, role?, supervise?, views}`.
   - **Refusals.** Exit 1, with `rh: CODE: reason` on stderr: `seat_exists`, `seat_name`,
     `seat_agent`, `seat_role`, `seat_role_reserved`, `harness_model_refused`,
     `claude_signin_undeclared`, or a cwd or binary refusal.

   **The form is done in #691 (2026-10-07):** `marley: new harness seat`, through the followed
   harness's own command. The Marley agent's tool for the same is done in #692: `seat_add`, asked
   like a settings change.

   The form itself: Host (this box or a fleet host), name, role, agent and folder; Marley
   runs `rh seat add` and `rh seat start` on that host over SSH. The Marley agent gets the same as
   a tool ("set up a manager on forge working in /srv/work/x").
7. **The manager in the Agent Panel.** When a manager seat exists, Marley adds its `rh acp` to
   `agent_servers`. MREQ-008 shrinks to whatever the panel cannot show.

UCSOS, Chad with the harness session:

8. **Adopt the existing manager.** A seat whose profile runs the same agent, prompt and Forge MCP
   in the same folder on forge.

Order inside the phase: 1 and 4 start together; 2, then 3, then 7; 5 after 4; 6 after 1; 8 after
1.

**The harness's tickets** (its reply, 2026-10-07, harness D178). It first closes TICKET-092, its
active pair, then: TICKET-109 (a seat in one step, JSON out, refusals by name over SSH; also gives
`agent/` seats the `rh attach`/`rh view` views, which item 5 needs), TICKET-110 (an interface seat
that can act as manager: today the harness's server refuses Claude Code in its own interface as
`agent_unknown`), TICKET-106 (the thread; a person's message to a busy manager now waits and is
delivered in order), TICKET-111 (`rh acp` against agent-client-protocol 2.1.0: one thread per
root, `session/cancel` interrupts the manager's turn, confirmations as permission requests with a
60 s expiry, the editor's MCP servers ignored), TICKET-112 (a seat's own MCP servers and prompt,
which adopting the UCSOS manager needs, since Forge's MCP is HTTP with a signing headers helper),
TICKET-113 (the foreman seat, plan only, after decision 1). Its corrections to this plan:

- `--agent codex` opens a Codex App Server session, watched with `rh codex connect` and `rh codex
  history`, not Codex's own TUI in a pane. Codex's TUI in a harness terminal would be a new ticket.
- Marley owes a proof before TICKET-111 is built: that Zed shows an agent's `session/update` sent
  between turns (`acp.rs:4805`), and whether it shows a `session/request_permission` outside a
  running turn. Proven in Marley's build on 2026-10-07 (#685) and sent to the harness: a report
  between turns shows as its own paragraph but raises nothing; a confirmation between turns shows
  in the panel and in Marley's Needs you, and the choice comes back; after it the agent sends a
  `tool_call_update` with a final status so the line reads as settled.
- The UCSOS manager's interface runs with `--strict-mcp-config` and no setting sources; whether
  it needs its login's hooks, skills or CLAUDE.md is a question for Chad when forge is looked at.

Done when Chad talks to the UCSOS manager in the Agent Panel, watches it in a pane, restarts
Marley, and the manager does not notice.

## Phase 3: the foreman

Waits on decision 1.

- **The harness's supervision** already restarts a failed session, stops and restarts a stalled
  one, and escalates to the owner inbox when its policy runs out (M10, `docs/SUPERVISION.md`). The
  seats declare policies (a profile's `supervise`).
- **If the foreman stays a seat** (UCSOS's foreman also judges, its stuck detector among other
  things): the harness accepts a `foreman` role that receives supervision's escalations before the
  manager, and Marley's New Agent form offers it.
- **Marley** shows the foreman's restarts, stops and escalations on each agent's row and in the
  inbox.

Done when an agent killed on purpose comes back, a stalled one is stopped and restarted, and one
past its policy reaches Chad through the manager.

## Phase 4: the fleet across the boxes

- **The Brain** serves `marley.work/v1`: agents, work items, phases, gates, questions and token
  use. For the UCSOS test, Forge's records through an adapter, or rustal-brain (decision 2).
- **The harness**, TICKET-097, the Brain bridge (M14): a manager on forge assigns work to agents on
  the other boxes.
- **Chad with the harness session:** a harness on each of the five agent boxes, seats from
  profiles.
- **Marley:** the Fleet panel and the Agent tab on real data; the bottom bar's snapshot opens an
  agent's tab; "No Brain connected" where the store is missing; each box's resources over SSH
  (built, #610).

Done when all six boxes show in Marley with each agent's ticket and phase, and an assignment the
manager makes on forge starts work on an agent box.

## Phase 5: the switch and the patterns

- **Marley:** one setting that turns the integration on, as `marley.rusty.enabled` does for Rusty,
  with the three levels stated where the integration shows.
- **Marley:** the patterns written for other systems: sessions (the fleet contract), conversation
  (ACP), questions (the inbox), memory (the brain SDK) and hands (Marley's MCP server).

## Later

The relay and the phone (the second half of harness M15). T3 survey items 1 to 3 for desk
terminals: Codex resume with the Interrupted chip, usage limits as a state, past sessions. T3 items
8 and 9 (handing work between agents, agents that start agents) belong to the harness.

## Decisions for Chad

One at a time. Each has the default the plan follows until Chad says otherwise.

1. **The foreman:** a seat, or the harness's supervision? *Default: supervision does the
   mechanics, and UCSOS's foreman stays a seat for judgment.*
2. **The Brain for the first fleet test:** UCSOS's Forge through an adapter, or rustal-brain?
   *Default: Forge, since UCSOS is the test apparatus.*
3. **The Marley agent:** offered once when an agent is found, or on for everyone who has one?
   *Default: offered once, since new Marley layers ship off.*

## Promotion
This is NOT an active pipeline doc — it is a candidate. Promote it via
`/pipeline:plan` when ready: it becomes a ticket (`docs/planning/tickets/open/`) + an active
pipeline doc pair (`docs/planning/pipeline/active/`). On promotion, set
`status: promoted` and fill `ticket:` + `pipeline_spec:`.
