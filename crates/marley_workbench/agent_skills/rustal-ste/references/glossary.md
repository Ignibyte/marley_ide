# rustal-ste glossary

The project glossary for `rustal-ste`, the ASD-STE100 rules for messages and instructions
between agents in the Rustal ecosystem: rustal-harness, Marley, Rusty and the Rustal Brain.
Each term has one meaning. Do not use the words in the "Don't use" column for that meaning.

Sources name the repository, then the file and section. `D186` and similar ids are entries in
rustal-harness `docs/DECISIONS.md`. The "Owner" column names the system whose docs set the
word: harness, Marley, Rusty, Brain, or workflow (the ticket and phase process that the
repositories share).

## Terms

### Harness

| Term | One meaning | Owner | Don't use | Source |
|---|---|---|---|---|
| root | The root is the private directory that holds the state, profiles and journal of one harness runtime. | harness | state dir, data dir, home | rustal-harness: docs/MCP.md#running-it; docs/MANAGER.md#designating-the-manager |
| runtime | The runtime is the `rh serve` process that owns one root and does all harness work. | harness | daemon, harness process | rustal-harness: docs/TERMINALS.md#start-and-attach; docs/ARCHITECTURE.md#component-model |
| workspace | A workspace is one tmux session that the runtime owns, with a UUID, a name, windows and panes. | harness | box, sandbox, tmux session | rustal-harness: docs/TERMINALS.md#start-and-attach |
| terminal | A terminal is one PTY and the program in it: a harness pane or a Marley terminal tab. | harness | console, tty | rustal-harness: docs/TERMINALS.md; marley_ide: docs/marley/guide.md#where-terminals-open |
| pane | A pane is one tmux pane in a workspace window, with an id such as `%0`. | harness | tile, frame | rustal-harness: docs/TERMINALS.md#shell-editor-and-server |
| session | A session is one agent entry in the fleet. A fleet id names it, and `session_*` tools act on it. | harness | fleet seat, job, agent run, instance | rustal-harness: docs/FLEET.md#the-envelope; docs/MCP.md#tools |
| agent seat | An agent seat is a session of kind `agent`: a harness terminal whose agent reports its own state. | harness | reporting terminal, mod seat | rustal-harness: docs/AGENT_SEATS.md#the-seat-in-the-fleet |
| seat | A seat is a named agent position in a root that one profile defines. `rh seat` commands act on it. | harness | slot, agent slot, seat label | rustal-harness: docs/AGENT_SEATS.md#a-seat-in-one-step; docs/planning/pipeline/active/seat-stop.spec.md |
| profile | A profile is the operator's file `ROOT/profiles/NAME.json` that tells the runtime how to open a seat's sessions. | harness | config, manifest, preset | rustal-harness: docs/MCP.md#opening-a-session; docs/AGENT_SEATS.md#a-seat-in-one-step |
| generation | A generation is one process lifetime of a session. A restart opens the next generation with the same fleet id. | harness | incarnation, attempt | rustal-harness: docs/FLEET.md#the-envelope; docs/SUPERVISION.md#restarts |
| state | A state is one session value: `starting`, `working`, `idle`, `waiting`, `error` or `done`. | harness | mode, condition, health | rustal-harness: docs/FLEET.md#the-envelope; docs/AGENT_SEATS.md#reporting |
| question | A question is a choice that a session asks for, with a prompt and options. | harness | query, poll | rustal-harness: docs/FLEET.md#the-envelope; docs/MCP.md#answering-a-question |
| approval | An approval is a question about one tool call, with the options `approve`, `deny` and `cancel`. | harness | permission request, permission prompt, consent | rustal-harness: docs/AGENT_SEATS.md#approvals; docs/CLAUDE_CODE.md#approvals |
| delivery | A delivery is one carried send or message, with a UUID and a state that only moves forward. | harness | packet, drop | rustal-harness: docs/MCP.md#sending; docs/AGENT_SEATS.md#sends-and-interrupts |
| send | A send is text that `session_send` delivers into the next turn of an idle session. | harness | nudge, poke, injection | rustal-harness: docs/MCP.md#sending |
| message | A message is text that the harness carries to one recipient's inbox, with a delivery id. | harness | mail, DM, ping, chat | rustal-harness: docs/AGENT_MESSAGES.md#the-tools; docs/MESSAGES.md |
| mailbox | The mailbox is the runtime's durable set of inboxes for the operator, actors and controllers. | harness | queue, postbox | rustal-harness: docs/MESSAGES.md#operator-commands |
| receipt | A receipt is the record of one result: the answer to a write, a received message or a passed gate. | harness | ack, acknowledgement, token | rustal-harness: docs/MCP.md#tools; docs/AGENT_MESSAGES.md#guarantees; marley_ide: CONSTITUTION.md §0 |
| operator | The operator is the local account that runs `rh` on a root through its private socket. | harness | admin, sysadmin | rustal-harness: docs/MESSAGES.md#bounds-storage-and-authority; docs/AGENT_MESSAGES.md#who-is-calling |
| controller | A controller is the one client connection that holds input control of a session or a pane. | harness | driver, pilot | rustal-harness: docs/ARCHITECTURE.md#input-ownership-and-resize; docs/MESSAGES.md#worker-and-harness-interfaces |
| actor | An actor is a scripted fixture agent that the runtime runs in a tmux terminal for offline tests. | harness | worker, fake agent, bot | rustal-harness: docs/ACTORS.md |
| manager | The manager is the one session that a root designates to assign work, answer questions and report to the user. | harness | lead, orchestrator, coordinator | rustal-harness: docs/MANAGER.md; docs/MANAGER.md#from-inside-its-session |
| foreman | The foreman is the one session that a root designates to take `failure` and `hold` owner items before the manager. | harness | supervisor, triager | rustal-harness: docs/MANAGER.md#the-foreman; D179, D180. UNSURE: see item 3. |
| owner item | An owner item is a `question`, `hold` or `failure` that the runtime raises in the owner inbox for judgment. | harness | alert, incident, escalation | rustal-harness: docs/MANAGER.md#the-owner-inbox |
| manager thread | The manager thread is the durable line of records between a root's manager and the user: `message`, `report` or `confirmation`. | harness | channel, chat log | rustal-harness: docs/MANAGER.md#the-thread; marley_ide: docs/marley/guide.md#the-harnesss-sessions |
| supervision | Supervision is the runtime policy that restarts a failed or stalled session and escalates when no restart is left. | harness | watchdog, auto-heal, babysitting | rustal-harness: docs/SUPERVISION.md |
| grant | A grant is the named class of tools that an MCP server serves to one client. | harness | scope, access level, privilege | rustal-harness: docs/MCP.md#tools; marley_ide: docs/marley/guide.md#grants-and-what-keeps-an-agent-in-check |
| report | A report is an agent's own statement of its state, phase or result, through a report tool. | harness | update, status update, check-in | rustal-harness: docs/AGENT_SEATS.md#reporting; docs/PHASES.md#reporting-a-phase; docs/MANAGER.md#the-thread |

### Workflow

| Term | One meaning | Owner | Don't use | Source |
|---|---|---|---|---|
| the user | The user is the person who owns the ecosystem and directs its agents. | workflow | the human, the person, the boss, the owner | rustal-harness: AGENTS.md#mission-and-scope; docs/WORKFLOW.md#start-or-resume |
| evidence | Evidence is the kept output, exit status, captures and paths that show that one criterion holds. | workflow | proof, logs | rustal-harness: docs/WORKFLOW.md#phases-and-records; marley_ide: CONSTITUTION.md §7; rustal-brain: docs/architecture/IDENTITY-ORCHESTRATION.md#orchestration-rules |
| gate | A gate is a project's check script, such as `script/gates.sh` or `bin/gate.sh`, with the checks it runs. | workflow | CI, the checks, lint run | marley_ide: CONSTITUTION.md §0; rustal-harness: docs/WORKFLOW.md#commands-and-evidence |
| ticket | A ticket is the local file that records one work item and its acceptance criteria. | workflow | issue, card, story | marley_ide: CONSTITUTION.md §19; rustal-harness: docs/WORKFLOW.md#phases-and-records |
| pipeline | The pipeline is the folder set through which a ticket's spec and notes files move, phase by phase. | workflow | flow, process, workflow run | marley_ide: CONSTITUTION.md §3; rustal-harness: docs/WORKFLOW.md#phases-and-records |
| phase | A phase is one named part of the pipeline. It must pass before the next phase starts. | workflow | stage, step, sprint | marley_ide: CONSTITUTION.md §3; rustal-harness: docs/WORKFLOW.md#phases-and-records. UNSURE: see item 1. |
| Plan | Plan is the phase that picks the ticket, recalls prior knowledge and writes the spec and notes files. | workflow | scoping, triage | marley_ide: CONSTITUTION.md §3; rustal-harness: docs/WORKFLOW.md#phases-and-records |
| Code | Code is the phase that makes the change, reviews its diff and runs focused checks. | workflow | build phase, dev phase | marley_ide: CONSTITUTION.md §3, §18; rustal-harness: docs/WORKFLOW.md#phases-and-records. UNSURE: see item 2. |
| Test | Test is the phase that runs the change end to end and keeps the evidence. | workflow | QA, verification phase | marley_ide: CONSTITUTION.md §3, §7; rustal-harness: docs/WORKFLOW.md#phases-and-records |
| Complete | Complete is the phase that audits each criterion, updates the docs, closes the ticket and archives its files. | workflow | close-out, finish, wrap-up | marley_ide: CONSTITUTION.md §3, §21; rustal-harness: docs/WORKFLOW.md#phases-and-records |
| CHANGELOG | CHANGELOG is the top-level `CHANGELOG.md` of a repository. It records what changed and why, one entry each change. | workflow | release notes, history, news | marley_ide: CONSTITUTION.md §21 |
| ledger | A ledger is an append-only knowledge file of coded entries: `F-`, `PR-`, `L-` and `AD-`. | workflow | wiki, KB, lessons file | marley_ide: CONSTITUTION.md §19. UNSURE: see item 9. |
| handoff | A handoff is the message or file that gives the next agent the work's state, evidence and next step. | workflow | baton pass, brain dump, summary | rustal-harness: .agents/skills/harness-workflow/SKILL.md (step 5); docs/CLI_HANDOFF.md; rustal-brain: docs/architecture/IDENTITY-ORCHESTRATION.md#orchestration-rules |

### Marley

| Term | One meaning | Owner | Don't use | Source |
|---|---|---|---|---|
| rail | The rail is Marley's left column that lists groups, projects and their terminals, tabs, threads and ports. | Marley | sidebar, tree, nav | marley_ide: docs/marley/guide.md#the-rail |
| project | A project is a folder that the user opens in Marley. The rail shows it with its terminals and threads. | Marley | repo, codebase | marley_ide: docs/marley/guide.md#the-rail |
| group | A group is a rail entry with no folder, such as Home, Rusty or a group that the user names. | Marley | space, bucket | marley_ide: docs/marley/guide.md#groups-with-no-folder-600 |
| thread | A thread is one exchange of prompts and replies with an agent in Zed's Agent Panel. | Marley | dialog, chat history | marley_ide: docs/marley/guide.md#the-rail; docs/marley/guide.md#an-agent-thread-in-a-center-tab |
| Marley agent | The Marley agent is the Agent Panel entry that explains Marley and proposes changes that the user applies. | Marley | the assistant, Marley bot, helper | marley_ide: docs/marley/guide.md#the-marley-agent |
| Rusty agent | The Rusty agent is the Agent Panel entry that does the user's personal work with Rusty's tools only. | Marley | Rusty bot, personal agent | marley_ide: docs/marley/guide.md#rusty-in-the-agent-panel |
| block | A block is one shell command in a Marley terminal, with its output and its exit status. | Marley | cell, entry, chunk | marley_ide: docs/marley/guide.md#blocks |
| Needs you | Needs you is the rail section that shows each wait on the user, the longest wait first. | Marley | inbox, attention list, notifications | marley_ide: docs/marley/guide.md#the-rail |
| Agent Activity | Agent Activity is Marley's log of each acting tool call that an agent makes, newest first. | Marley | audit log, activity feed | marley_ide: docs/marley/guide.md#agent-activity-and-the-kill-switch |
| agent control | Agent control sets how Marley's acting tools ask the user: Ask First, Ask Every, Allow or Off. | Marley | permission mode, trust level | marley_ide: docs/marley/guide.md#zeds-editors-for-agents |
| kill switch | The kill switch, the Stop button, makes each acting Marley tool refuse each agent until the user resumes. | Marley | panic button, lockout | marley_ide: docs/marley/guide.md#agent-activity-and-the-kill-switch |
| refusal | A refusal is a tool's answer that it did nothing, with a code and a reason. | Marley | rejection, denial, bounce | marley_ide: docs/marley/guide.md#what-agents-are-told-and-how-a-call-is-refused; rustal-harness: docs/MCP.md#tools |
| harness writes | Harness writes is the Marley switch that lets Marley answer, send to and open harness sessions. | Marley | write mode, control mode | marley_ide: docs/marley/guide.md#the-harnesss-sessions |

### Rusty

| Term | One meaning | Owner | Don't use | Source |
|---|---|---|---|---|
| Rusty | Rusty is the user's local store of tasks, notes, memories, skills, secrets and pages. `rusty-mcp` serves it. | Rusty | Rusty app, the assistant app | omarchy-rusty: AGENTS.md#product; docs/architecture.md#the-shape |
| Rusty's brain | Rusty's brain is the user's markdown vault of pages. Rusty indexes it and keeps it in git. | Rusty | the vault, Obsidian | omarchy-rusty: docs/architecture.md#the-store |
| page | A page is one markdown file in Rusty's brain. Its slug names it. | Rusty | doc, article | omarchy-rusty: docs/architecture.md#the-store; docs/tools.md#brain-pages-and-search |
| decision | A decision is a page under `decisions/` that `brain_decide` writes, with the choice and its rationale. | Rusty | ruling, verdict | omarchy-rusty: docs/architecture/brain-loop.md#the-shape; Rusty skills store: ask-decide-follow-up/SKILL.md#decide |
| follow-up | A follow-up is the dated outcome that `brain_follow_up` adds to a decision: `kept`, `revised` or `superseded`. | Rusty | retro, check-back | Rusty skills store: ask-decide-follow-up/SKILL.md#follow-up; omarchy-rusty: docs/architecture/brain-loop.md#the-shape |
| task | A task is one item on a Rusty to-do list. It is open, done or archived. | Rusty | todo, action item, chore | omarchy-rusty: docs/tools.md#to-do-lists; marley_ide: docs/marley/guide.md#the-tasks-tab |
| memory | A memory is one fact, preference or context line that Rusty keeps, with a category and an importance. | Rusty | recollection, saved fact | omarchy-rusty: docs/tools.md#memories; marley_ide: docs/marley/guide.md#the-memory-tab-664 |
| skill | A skill is a `SKILL.md` in Rusty's skills store that Claude Code loads once the skill is active. | Rusty | slash command, macro, recipe | marley_ide: docs/marley/guide.md#the-skills-tab-665; omarchy-rusty: docs/architecture.md#the-store |
| capture | A capture adds one line to the timeline of today's daily note or of the inbox page. | Rusty | jot, quick note | omarchy-rusty: docs/tools.md#brain_capture; marley_ide: docs/marley/guide.md#capture-and-import-663 |

### Brain

| Term | One meaning | Owner | Don't use | Source |
|---|---|---|---|---|
| the Brain | The Brain is the Rustal Brain server that assigns work, enforces the workflow and decides completion from evidence. | Brain | the tower, control plane | rustal-brain: docs/architecture/SYSTEM.md#system-boundary; rustal-harness: D165 |
| hive | The hive is every machine whose harness takes work from the Brain and reports back to it. | Brain | swarm, cluster, mesh | rustal-harness: D165; docs/planning/pipeline/completed/hive-direction.spec.md |
| hub | The hub is the Brain's role: all messages and assignments between machines pass through it. | Brain | router, switchboard | rustal-harness: D165; docs/planning/CORE_REQUESTS.md (CR-006) |
| workd | workd (`rustal-workd`) is the Brain's trusted local broker for identity, leases and writes on one host. | Brain | agentd, sandbox | rustal-brain: docs/architecture/IDENTITY-ORCHESTRATION.md#rustal-workd-trust-boundary |
| assignment | An assignment is one bounded unit of work that the Brain gives one agent, with paths, phases and an expiry. | Brain | work order, job | rustal-brain: docs/architecture/IDENTITY-ORCHESTRATION.md#root-runs-and-assignments |
| lease | A lease is the Brain's time-limited right for one assignment to write in one isolated worktree. | Brain | lock, reservation | rustal-brain: docs/architecture/IDENTITY-ORCHESTRATION.md#rustal-workd-trust-boundary; docs/architecture/SYSTEM.md#workspace-plane |
| identity | An identity is an agent's Brain principal. The Brain addresses the agent by it, never by its host. | Brain | hostname, username, seat label | rustal-brain: docs/architecture/IDENTITY-ORCHESTRATION.md#principals; rustal-harness: D165 |

## Words that mean different things in different systems

Use the glossary term in the "Write" column. Quote UI labels, commands and field names
verbatim, whatever word they use.

| Word | Meanings in the docs | Write |
|---|---|---|
| session | A fleet entry (harness). Claude Code's own session, `session_id` and `--resume`. Marley's Allow for This Session. `rusty session start` (Rusty's service). An MCP connection. | "session" for a fleet entry only. "conversation" for Claude Code's or Codex's transcript, as rustal-harness docs/CLAUDE_CODE.md#stop-and-restart does. |
| seat | A named position from a profile (harness `rh seat`). An entry in Marley's fleet `seats` array. `rw`'s seat label (Brain). | "seat" for the named position. "session" for a fleet entry. A seat label is metadata, not identity (rustal-brain docs/architecture/SYSTEM.md#agent-plane). |
| workspace | A tmux session (harness). A leased worktree (Brain: "isolated workspace"). Zed's window and its projects. | "workspace" for the harness meaning. "isolated worktree" for the Brain's. "window" for Zed's. |
| root | The harness state root. A project's top folder (Marley: "the project root"). The Brain's root run. | "root" for the harness meaning. "project folder". "root run". |
| thread | An Agent Panel thread (Marley). The manager thread (harness). A Codex or Claude `thread` label. Both Marley and the harness serve a tool named `thread_post`. | "thread" for the Agent Panel. "manager thread". Quote the label. Name the server with the tool: "the harness's `thread_post`". |
| brain | Rusty's vault. The Rustal Brain. | "Rusty's brain" or "the Brain". Never "the brain" alone. |
| inbox | The owner inbox. An agent's message inbox. Needs you ("the inbox" in Marley's guide). Rusty's inbox page. | Always qualify: "owner inbox", "message inbox", "Needs you", "Rusty's inbox page". |
| stop | Stop a seat or a session (harness). The kill switch's Stop button. Claude Code's Stop hook at a turn's end. A port row's Stop. | "stop" for a seat, session or workspace. "kill switch" for agent control. "The turn ends" for a turn. |
| decision | A Rusty decision page. A harness decision register entry. A Marley `AD-` ledger entry. An approval's `approve`, `deny` or `cancel`. | "decision" for the Rusty page. Cite `D186` or `AD-…` by id. An approval gets an answer. |
| profile | The harness profile file. Zed's agent profile (Write, the Marley profile). | "profile" for the harness file. "Zed agent profile" for Zed's. |
| pane | A tmux pane (harness). Zed's center pane. | "pane" for tmux. "the center" or "a tab" for Zed. |
| report | `agent_report`. `workflow_report`. A manager thread `report`. A gate's `result.json`. | "report" for an agent's statement. "gate result" for `result.json`. |
| PR | A pull request. A `PR-` prevention rule. | "pull request" in full. `PR-001` only for the rule. |
| park folder | rustal-harness parks to `docs/planning/pipeline/paused/`. Marley_ide has `docs/planning/pipeline/parked/`. | The verb is "park". Name the folder the repository uses. |
| the person | MANAGER.md says "the person" for the human in the manager thread. | "the user". Quote the field value `person` verbatim. |

## Verbs

One verb for each action. Tool names, commands and UI labels stay verbatim, even when their
word is in the "Don't use" column.

| Action | Approved verb | Don't use | Example sentence |
|---|---|---|---|
| End the live generation of a seat, session or workspace on purpose | stop | kill, shut down, terminate, take down | Stop the seat `scratch`. Its profile stays in the root. |
| Stop a seat and delete its profile | remove | destroy, tear down, purge, drop | Remove the seat `scratch` when the test ends. |
| Run a seat's profile and wait for its first report | start | launch, spin up, boot | Start the seat `lead` with `rh seat start lead`. |
| Make a new session from a profile | open | spawn, create, instantiate | Open a session from the profile `reviewer` with `session_open`. |
| Open the next generation of a failed session with the same fleet id | restart | respawn, relaunch, reboot, revive | Supervision restarts the seat from the resume argv that its agent reported. |
| End a running turn and keep the session | interrupt | abort, cancel, kill | Interrupt the turn with `rh interrupt agent/<workspace uuid>`. |
| Give text to a session or to another agent | send | ping, DM, nudge | Send the gate result to `manager` with `message_send`. |
| Carry a send or a message to its recipient | deliver | route, hand off | The runtime delivers the send when the seat is idle. |
| Take a message and record a receipt | receive | ack, acknowledge, read | Receive the message before you act on it. A read is not a receipt. |
| Send a message that answers an earlier message | reply | respond, write back | Reply with `reply_to` set to the delivery of the message that you answer. |
| Give the choice for a question or an approval | answer | resolve, handle, approve | Answer the approval with `deny`. Give the reason in the manager thread. |
| Tell the host your state, phase or result | report | announce, signal, notify | Report `waiting` with the prompt and the options. |
| End your own authority on your seat, as done | release | sign off, exit, quit | Release the seat when your session ends. |
| Close a view and keep its process | detach | leave, hide | Detach from the workspace with Ctrl+b, then d. The shell continues. |
| Name a session as the root's manager or foreman | designate | appoint, elect, crown | Designate `agent/<workspace uuid>` as the manager with `rh manager`. |
| Declare a restart policy for a session | supervise | watch, babysit, monitor | Supervise the seat with two attempts and backoffs of 0 and 5000 milliseconds. |
| Raise a failure for the manager when no restart is left | escalate | bubble up, punt, kick up | Supervision escalates the generation after two failed restarts. |
| Pass an owner item that you hold to the manager now | forward | pass on, hand up | Forward the failure to the manager with `owner_forward`. |
| Pass the user's words, verbatim, to another agent | relay | paraphrase, summarize, convey | Relay the user's words in quotes, with the date. |
| Give one bounded piece of work to one agent | assign | dispatch, delegate, hand out, allocate | Assign harness TICKET-114 to the seat `lead`. |
| Take input control of a session or a pane | claim | take over, grab, seize | Claim control with Ctrl-b c before you type in the view. |
| Consult Rusty's brain before a decision | ask | search, look up, query | Ask Rusty's brain with `brain_ask` before you choose the crate. |
| Record a choice with its rationale | decide | settle, conclude, rule | Decide with `brain_decide`. Set `follow_up_by` one week from today. |
| Record how a decision went | follow up | revisit, check back, review | Follow up the decision with the status `kept`. |
| Add one line to today's daily note or to the inbox page | capture | jot, note down | Capture the port number to today's daily note. |
| Keep a fact or a preference in Rusty | store | save, memorize, remember | Store the preference as a memory with the importance `high`. |
| Move a ticket's spec and notes out of the active pipeline as `Blocked`, ticket still open | park | shelve, defer, pause, freeze | Park harness TICKET-097 until the Brain publishes CR-006. |
| Continue a parked ticket or a conversation from its recorded point | resume | unpark, reopen, continue | Resume TICKET-097 at `Phase 2 — Code`. |
| Move a ticket from the backlog into the active pipeline at Plan | promote | pick up, pull, activate | Promote the top Queue row of `BACKLOG.md` with `/pipeline:plan`. |
| Move a finished ticket to `tickets/closed/` | close | finish, wrap up | Close #710 when each criterion has evidence. |
| Keep a thing, but move it out of the active list | archive | stash, retire | Archive the spec and notes files into `pipeline/completed/`. |
| Destroy a thing for good | delete | wipe, nuke, erase | Delete the thread only when the user tells you to. |
| Record staged changes in git | commit | check in, snapshot | Commit with the ticket id in the message body. |
| Send commits to a remote | push | ship, upload, sync | Push only when the user tells you to. Scan the range for private content first. |
| Execute a gate or a check | run | kick off, fire, trigger | Run `script/gates.sh --diff` at the end of Code. |
| Meet each check of a gate or a phase | pass | go green, clear, succeed | The phase passes when each criterion has evidence. |
| Not meet a check of a gate or a phase | fail | go red, break, flunk | The gate fails on `clippy`. Fix the cause. Do not add a suppression. |
| Decline a call and change nothing | refuse | reject, bounce, turn down | The runtime refuses the send because the seat is not idle. |
| Offer a change that waits for the user's Apply | propose | suggest, request | Propose the seat with `seat_add`. The user applies it. |
| Write a durable fact in a file or a store | record | log, jot, write down | Record the refusal code in the notes. |

## Identifiers

Write each id as the line shows. Put ids, codes, paths and commands in backticks.

- **Harness ticket:** `TICKET-114`, as rustal-harness writes it; cite it as "harness TICKET-114" outside that repository.
- **Marley ticket:** `#710` inside Marley; the file is `TICKET-710-<slug>.md`; cite it as "Marley #710" outside Marley. UNSURE: see item 7.
- **Rusty ticket:** "Rusty TICKET-053"; Rusty's record is private, so give the ticket id only.
- **Commit SHA:** 7 or more lowercase hex characters, `0b049fe`; all 40 when you pin a commit, `0ba2872d8f53e666d7a8596fa63afe353cb5800d`.
- **Branch:** the full name, `marley/workbench-shell`.
- **Fleet id (a session's id):** `agent/<workspace uuid>`, `claude/<session uuid>`, `codex/<session uuid>`, `actor/<actor uuid>` or `herdr/<server>/<pane>`; an agent seat's id is `agent/<workspace uuid>`.
- **Seat name:** the profile name, lowercase, as `rh seat add` takes it: `lead`.
- **Generation:** its UUID; a question's prompt shows its first 8 characters: `[generation 1a2b3c4d]`.
- **Delivery id:** a canonical UUID that you keep for a retry: `533f7d9e-7c26-4b9a-b815-e649ac41ff0f`.
- **Harness decision register entry:** `D186`.
- **Requests between repositories:** `CR-006` (harness to Rustal, the Brain or workd), `MREQ-009` (harness to Marley).
- **Requirements and criteria:** `R13-02` (a harness milestone requirement), `SS-001` (one ticket's criterion).
- **Ledger entries:** `F-…`, `PR-…`, `L-…`, `AD-…` in Marley; `PR-…`, `BF-…`, `AD-…` in Rusty. Write "pull request" in full, never "PR".
- **Rusty page:** its slug, `projects/orbit`; a decision, `decisions/<slug>`; a consultation, the id that `brain_ask` returns.
- **Refusal and error codes:** verbatim, never paraphrased: `seat_exists`, `agent_report_stale`, `not_granted`.
- **Tool names:** the underscore form, with the server when two servers serve the name: `session_send`, "the harness's `thread_post`".
- **Event names:** verbatim: `agent_seat_report`.
- **State, option and outcome values:** verbatim, lowercase: `waiting`, `approve`, `passed`.
- **Phase status line:** verbatim: `Phase 2 — Code PASS; ready for Phase 3 — Test`.
- **Setting keys:** the full dotted path: `marley.harness_writes`.
- **Commands:** exact, with the docs' capital placeholders: `rh --state ROOT seat stop NAME`.
- **File paths:** relative to the repository root, with the repository named when it is not the reader's: `rustal-harness: docs/AGENT_SEATS.md`; a line as `path:line`. Write `~` for a home folder, never an account name.
- **UI labels:** the exact text and case that the UI shows, without backticks: Apply, Stop, Allow for This Session.
- **The user's words:** verbatim, in double quotes, with the name and the date; keep the spelling: Chad, 2026-10-09: "lets do the small things".
- **Dates and durations:** dates as `YYYY-MM-DD`; durations as a number and a unit word: 590 seconds.

## Defaults until Chad decides

The docs disagreed or were silent on these. Each line is the rule agents follow now.

1. **Phases.** Name each repository's phases as its own docs do: Marley has Plan, Code, Test,
   Complete. rustal-harness has Plan, Code, Test, Document, Complete.
2. **The full gate.** Each repository keeps its own rule: Marley runs it at the end of Code,
   rustal-harness at the end of Test.
3. **The foreman.** Keep the term while the role exists.
4. **A repository's directing agent.** Name it by its peer name (for example `marley-ide-b2`), or
   by its fleet id when it runs in a seat. Do not call it a "session".
5. **"host".** A host is a machine. The program that a Claude Code mod reports to is its
   "report target".
6. **"receipt".** Say which: a write receipt (a write verb's answer), a message receipt (a
   received message), or a gate receipt (a passed gate).
7. **Ticket ids across repositories.** The owner's form with the repository's name: "Marley #710",
   "harness TICKET-114".
8. **A seat's name** is its profile name (`lead`). `agent/<workspace uuid>` is the agent seat's
   fleet id.
9. **Ledger codes.** Each repository keeps its own.
