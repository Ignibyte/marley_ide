# herdr and Hermes for Marley, 2026-10-02

Chad asked: "lets deep research (checking out the repos and researching documentation) for
herdr and also hermes. I want to see what value they may bring to Marley." With it came the idea
behind the question: "an all in one system for Marley which brings in obsidian like knowledge
graphs and also voice talking so that you can manage these projects + have a sort of developer
centric knowledge system for yourself. However, i would like these to be enabled rather than by
default because some may not want."

What was read: herdr's source at v0.9.3 (HEAD `5d78d05`) and its docs; the herdr plugin
marketplace index (1,480 plugins in 1,436 repos on 2026-10-02) with about a dozen plugins read
closely, among them the GPUI client `penso/herdr-gpui`; Hermes Agent's source at `0a374d1`
(v0.21.5) and its docs, against the June comparison in the brain
(`projects/brain-roadmap-hermes-obsidian-research`); and Marley's and Rusty's own trees for what
already exists. herdr is Apache-2.0 and Hermes is MIT, so like Orca both may be read under §20,
and copied code keeps its notice (Apache-2.0 code stays out of the `MIT OR Apache-2.0` crates
unless it sits in a module that carries the Apache notice). Nothing from either repo was built or
run. Research pages: `research/agent-workspaces/herdr`, `research/agent-workspaces/hermes-agent`,
and one page per plugin read.

The short answer: Marley should run neither. herdr is a terminal multiplexer whose server owns the
PTYs and re-renders them, which would strip Blocks and the Claude hook frames from any terminal
Marley showed through it; its overlap is with rustal-harness, not Marley. It is the best reference
found for multi-agent resume, agent-reported state and remote federation, and its marketplace is
direct evidence of what agent users install. Hermes is the most complete voice pipeline found, with
a memory-provider interface worth copying, and its knowledge side is still behind Rusty's brain:
two capped files, and issue #31720 (a linked wiki memory) open with no work. For the knowledge and
voice layer, Rusty already holds the graph Chad describes (740 pages, 2,353 links, a `brain_graph`
tool with local depth) and Marley already has local dictation (#480). What is missing is Marley
drawing the brain, joining it to the rail's projects, and turning dictation into commands, all off
until switched on.

## Part 1: herdr

**What it is:** One Rust binary. `herdr server` owns every pane's PTY and emulates it with
Ghostty's VT core (libghostty-vt, built with Zig 0.16); the client is a ratatui TUI. The server
serves two 0600 sockets: a newline-delimited JSON API of about 100 methods and 30 events
(`herdr.sock`), and a bincode protocol of server-rendered cell grids (`herdr-client.sock`). Agent
state comes from 22 TOML rule files matched against the bottom of the screen every 300 ms; six
agents report their own state instead. Plugins are child processes with no sandbox, and "the
entire Herdr CLI is the plugin API". 94 releases in six months, still 0.x, one main author.

### 1. Resume agents other than Claude Code

**Where it came from:** `src/agent_resume.rs:198-300` holds resume argv for 18 agents:
`claude --resume <id>`, `codex resume <id>`, `copilot --resume=<id>` and the rest. Self-reported
resume argv is checked at `:58`: a plain command name first, at most 64 elements and 8 KiB, no
apostrophe or control character.

**Marley today:** #540 resumes Claude Code only (`crates/marley_workbench/src/resume.rs:1-10`),
keyed by `MARLEY_TERMINAL_ID` from the SessionStart event.

**What Marley would do:** A resume table in `marley_agent` keyed by the agent kind Marley already
recognizes, each entry naming where the session id comes from (a hook event, a reported value, or
nothing), plus herdr's argv checks on any reported command.

**Size and ticket:** S for the table and checks; each agent's session-id source is its own small
job. No ticket.

### 2. A report-state contract for agents without Marley's plugin

**Where it came from:** `pane.report_agent` carries a source, an increasing `seq` and an optional
`resume_argv`; `pane.release_agent` clears it. An agent that reports is the authority for its pane
and screen rules switch off (`src/detect/mod.rs:323-333`). Agents find the API through
`HERDR_ENV`, `HERDR_PANE_ID` and `HERDR_BIN_PATH`.

**Marley today:** Typed state comes only from Claude Code's hook frames over OSC 777
(`crates/marley_agent/src/claude_events.rs`). Every other agent gets `agent_status`, a quiet timer
plus the bell (`crates/marley_agent/src/marley_agent.rs:593-599`).

**What Marley would do:** `marley report-agent` (and the same verb on `marley_mcp`) taking state,
seq and resume argv for the terminal named by `MARLEY_TERMINAL_ID`, so any agent or a small
wrapper can put typed rows on the rail.

**Size and ticket:** M. No ticket.

### 3. Screen rules as a labelled fallback, with explain

**Where it came from:** `src/detect/manifests/*.toml`: region-scoped matchers
(`bottom_non_empty_lines(12)`, `after_last_horizontal_rule`, `osc_title`) with any/all/not gates and
priorities, debounced (three confirmations within 700 ms for working to idle). `herdr agent explain
<pane> --json` names the rule and the evidence. `blocked` never comes from a fallback.

**Marley today:** The quiet timer above. The plan's rule is "never screen scraping" for state
Marley acts on (`three-prong-plan.md:20-22`), and herdr's tracker backs it: detection misreads are
its largest bug theme, many of them Claude (#4819, #4668, #4573).

**What Marley would do:** Rules only for agents with no hooks and no reports, shown as a weaker
status on the rail row, never driving the approvals inbox, with an explain view. Rules ship with
Marley; herdr's auto-fetched rule catalog is the part to leave behind.

**Size and ticket:** M. No ticket.

### 4. Send a prompt with a wait, and refuse a blocked agent

**Where it came from:** `agent.prompt` submits text and can wait atomically until the agent is
working; it refuses with `agent_blocked` while the agent shows a question.

**Marley today:** Plan D9 reserves `session_send` and `session_answer`; #508's rule already refuses
picks into an agent waiting on a permission prompt.

**What Marley would do:** Give `session_send` the same wait-until-working option and the refusal,
so an orchestrating agent never answers a permission prompt by accident.

**Size and ticket:** S, inside the D9 work. No ticket.

### 5. A read-only herdr fleet provider

**Where it came from:** `session.snapshot`, then `events.subscribe`, treating events as signals to
re-read and starting over on `events_lost`. Remote hosts answer the same API through
`herdr remote-api-bridge` over SSH.

**Marley today:** The Fleet panel draws `marley.work/v1` and `marley.host/v1` from the pseudo
provider, the host collector (#610) and the MCP and HTTP clients (#611).

**What Marley would do:** A `marley_herdr` provider, pure core plus thin transport like plan D8's
adapters, mapping herdr's workspaces, panes and agent states into the fleet snapshot, read only.
Someone who already runs herdr sees those agents in Marley's Fleet panel and opens Marley for the
editor, the browser and Blocks in Marley's own terminals. Prompting through it comes later, under
a write grant.

**Hard parts:** herdr's JSON API is at protocol 22 and still moving; only "endpoint generation 1"
and the JSON API are promised stable. A herdr pane cannot become a Marley terminal, because the
stream is herdr's re-render and Marley's DCS block hooks and OSC 777 frames never reach it.

**Size and ticket:** M. No ticket.

### 6. Remote hosts and the session layer

**Where it came from:** A herdr server on the remote owns its PTYs, reached over an SSH stdio
bridge. Only the selected machine streams screens; others send state and notifications. A health
ping keeps a dead link from reading Online; reconnect backs off to two minutes; cached state shows
dimmed with input off (`health.rs`, `supervisor.rs`, `registry.rs` and `shell_runtime.rs` in herdr's remote client; `connecting-machines.mdx:68, 90`). It also takes a systemd-logind
delay lock to save before shutdown, and keeps 48 rolling layout copies.

**Marley today:** #543 opens an ssh into a tmux session of Marley's own per remote terminal
(`crates/marley_workbench/src/remote.rs:1-9`).

**What Marley would do:** Use the streaming, ping and dimmed-cache rules wherever Marley shows a
remote host, and bring herdr's server model (PTYs owned on the host, observer and controller with
takeover, live handoff of PTY descriptors) to open decision 3, which program is Marley's session
layer. That is rustal-harness's brief, and herdr is the working example.

**Size and ticket:** Reference for decision 3; the logind lock is S on its own. No ticket.

### 7. What herdr's users install

The marketplace is an unreviewed index of GitHub repos tagged `herdr-plugin`. Counted by theme
over names and descriptions, with stars summed:

| Theme | Plugins | Stars | Marley |
|---|---|---|---|
| Review, diff, annotate, send to agent | 144 | 5,404 | #509, #522, #498 shipped |
| Phone, mobile, remote, Telegram | 108 | 5,224 | nothing |
| Browser in a pane | 32 | 5,123 | prong 3 shipped |
| Orchestration | 87 | 2,569 | fleet and harness |
| Notifications | 85 | 2,552 | #478 shipped |
| Tasks, boards, issues | 131 | 1,257 | `marley.work/v1`, read only |
| Usage and quota | 111 | 1,081 | tokens only |
| Memory, notes, knowledge | 45 | 677 | nothing in Marley |
| Voice | 17 | 41 | dictation (#480) |

Three things Marley lacks stand out. A phone surface is the largest category (collie 1,166 stars,
Heeler 461, herdr-remote 399, herdr-mobile-relay 271): approve and watch agents away from the
desk. Usage and quota probes come next; herdr-gpui alone carries about 40. And 558 of the 677
stars in the knowledge row belong to one plugin, `eliasstravik/herdr-projects`, which keeps memory
inside the work loop rather than beside it (Part 3, K4). Graph and Obsidian plugins sit between 1
and 20 stars. Voice drew at least 15 experiments in three months and none passed 9 stars.

**What not to take from herdr:** the TUI and multiplexer model; libghostty-vt (Marley committed to
vendored alacritty, D1); screen rules as the main source of state; rules and product announcements
fetched from herdr.dev; plugins as unsandboxed child processes; a herdr-compatible API so herdr
plugins run in Marley (they call herdr's whole CLI against its pane model, and setting `HERDR_ENV`
also triggers herdr's agent skill, which teaches agents to drive herdr).

## Part 2: Hermes Agent

**What it is:** Nous Research's Python agent runtime: memory, agent-written skills, a messaging
gateway, cron, delegation, a Kanban toolset, an Electron desktop app, and an ACP adapter. Python 3.14
only, 332 locked packages, about 870k lines of Python, 14,496 open issues. Since June it gained the
wake word, barge-in, desktop voice and GPT-Live, staged memory writes, a skills ledger and curator,
and the ACP adapter.

### 1. The voice pipeline, for when Marley goes past dictation

**Where it came from:** `tools/voice_mode.py`, `hermes_cli/cli_voice_mixin.py`,
`hermes_cli/cli_chat_turn_mixin.py`, `tools/tts_streaming.py`.

- Modes, each its own switch: push-to-talk (Ctrl+B), continuous, and an on-device wake word
  (openWakeWord with a bundled "Hey Hermes" model, sherpa-onnx, or Porcupine), the wake word off by
  default.
- Endpointing on loudness: speech counts after 0.3 s above the floor with 0.3 s dips allowed; 3 s
  of silence ends it, 15 s with no speech gives up (`voice_mode.py:621-708`). Transcription runs on
  the whole clip; nothing streams.
- A voice turn reaches the model with `[Voice input — respond concisely and conversationally, 2-3
  sentences max. No code blocks or markdown.]`, kept out of history
  (`cli_chat_turn_mixin.py:302-305`). A stop phrase counts only as the whole utterance.
- Replies go to speech sentence by sentence (20-character minimum, reasoning blocks stripped), the
  next sentence synthesized while one plays.
- Barge-in calibrates on the quiet room for 450 ms before any speech plays, trips at three times
  that floor (at least 1500 RMS during playback), interrupts the model while it generates and cuts
  audio while it speaks. With no echo cancellation in the CLI, a capture whose text matches what
  was just said is dropped. The next turn starts with a note that the user interrupted, valid for
  120 s (`voice_mode.py:1297-1446`, `tts_streaming.py:44-62`).
- The STT model loads when the mic opens, not when transcription starts (`tools/stt_lease.py`).

**Catches to avoid:** The default speech output is Edge, which sends the text to Microsoft's
online service, and with no explicit `stt.provider` a missing local engine falls through to any
cloud key it finds (`tools/transcription_tools.py:247-275`). Both break Chad's local-first rule.

**Marley today:** #480 drives Voxtype (Whisper `base.en`, local) and follows its status; D2 says
Marley does not touch audio. Zed's `crates/audio` already has cpal capture and a WebRTC echo
canceller (`audio_pipeline/echo_canceller.rs`), which beats Hermes's heuristic if Marley ever
captures audio itself. Rusty v2's whisper-rs and cpal capture loop is in `/srv/stacks/omarchy-ops`
history (5443f1e, deleted in 92e5beb).

**Size and ticket:** Reference for Part 3's V3 and V4. No ticket.

### 2. A memory-provider shape for the knowledge layer

**Where it came from:** `agent/memory_provider.py:84-206`. Required: name, availability,
initialize, tool schemas. Optional hooks: prefetch in the background and use it next turn,
`sync_turn`, `on_pre_compress`, `on_session_end`, and `on_memory_write` carrying the previous
content. Recall goes into the user message inside `<memory-context>` marked authoritative, so the
cached system prompt stays byte-identical (`agent/turn_context.py:107-128`).

**What Marley would do:** The same hook points on Marley's side of a knowledge provider, with Rusty
over `rusty-mcp` as the first provider. The fencing rule matters for any brief Marley composes for
an agent (K4).

**Size and ticket:** Folded into K4. No ticket.

### 3. Show and gate what agents write

**Where it came from:** `memory.write_approval` stages writes behind pending, approve and reject,
pinned to the entry they replace. Skill writes go to a ledger with rollback, the background review
may only touch skills it created, deletes become archives, and `/journey` lists, edits and deletes
what was learned. The journey edit path skips its own gate and ledger
(`agent/learning_mutations.py:193-199`), which is the mistake to avoid.

**What Marley would do:** When an agent writes to the brain through the knowledge layer, the write
shows in the Knowledge panel with who wrote it, and a setting can hold agent writes for approval.
Rusty's git history already gives the rollback.

**Size and ticket:** S on top of K2. No ticket.

### 4. `hermes acp` in the Agent Panel

**Where it came from:** `acp_adapter/`, ACP v1 over stdio, with a Zed snippet in its docs:
`"agent_servers": { "hermes-agent": { "type": "custom", "command": "hermes", "args": ["acp"] } }`.
Sessions, modes, models, streamed tool calls, diffs in permission prompts, and MCP servers passed
in by the client all work.

**Catches:** It never calls the client's `fs/*` or `terminal/*` methods, so its edits skip Zed's
buffers and review. Its plan update tests for a tool named `todo` while the tool registers as
`todo_list` (`acp_adapter/events.py:294`, `tools/todo_tool.py:217`), so Zed's plan view likely stays
empty. An ACP session still writes `~/.hermes` memory and skills unless both are switched off.
146 open `comp/acp` issues, including a missing model picker in Zed 1.18 and later.

**What Marley would do:** Nothing in code. Anyone who wants Hermes in the Agent Panel can add the
entry today.

**What not to take from Hermes:** the gateway and channels; GPT-Live (OpenAI only, $0.05 a
minute); Edge as a default; the LLM approval guardian; Kanban and cron (the harness covers them);
any runtime dependency on Hermes or its Python stack.

## Part 3: An opt-in knowledge and voice layer

### Where things stand

- **Knowledge:** Rusty's brain is the Obsidian-style vault: 740 pages, 2,353 links, typed decision
  edges, full-text plus vector search, git history. rusty-mcp serves `brain_search`,
  `brain_render` (Obsidian-flavoured HTML), `brain_graph` (nodes and edges, with `around` and
  `depth` for a local graph), `brain_capture`, the ask, decide and follow-up loop, and a
  `resources/list_changed` notification after every change. Rusty's app has a graph view
  (`crates/rusty-app/qml/GraphView.qml`). Marley reads none of it; `marley_workbench/src/rusty.rs`
  only offers rusty-mcp to Zed's agents.
- **Plan:** D11 says "Marley does not rebuild the knowledge workspace"; D20 says providers send
  data and Marley draws it. Marley drawing Rusty's brain fits D20 and needs D11 amended to say so.
  §20's brain boundary keeps the store a separate program over MCP.
- **Voice:** #480's dictation is shipped, local, and Marley touches no audio. Rusty's roadmap rules
  voice out (`rusty-v3/ROADMAP.md:362`), so voice is Marley's own.
- **Projects:** Nothing joins a rail project to a brain project page or a Rusty task group.
- **Switches:** `rusty_tools` defaults to true and the dictation mic shows wherever Voxtype is
  installed, so neither is opt-in today.

### Settings

Two blocks shaped like `marley.system_one` (`crates/settings_content/src/marley.rs:252-293`), each
with `enabled: false` as the master switch and its own uses:

```json
"marley": {
  "knowledge": {
    "enabled": false,
    "provider": "rusty",
    "panel": true,
    "project_join": true,
    "agent_briefs": "off",
    "agent_writes": "hold"
  },
  "voice": {
    "enabled": false,
    "dictation": true,
    "commands": "off",
    "speak": "off",
    "speech_output": "espeak-ng"
  }
}
```

Off means off the way `system_one` is: no process started, no MCP connection, no file watched.
Panels read the switch through `workspace::Panel::enabled` so their dock buttons disappear, the
pattern the Agent Panel uses (`crates/agent_ui/src/agent_panel.rs:5129`). The settings page gains
Knowledge and Voice sections. Every Zed-side change extends an existing row in
`zed-touchpoints.md` (`crates/settings_content/src/marley.rs`, `assets/settings/default.json`,
`crates/settings_ui/src/marley_page.rs` and `settings_ui.rs` for any new dropdown); none is new.

### Knowledge slices

The code splits the house way: a pure `marley_knowledge` crate (contract types for a
`marley.knowledge/v1` beside `marley_sdk`'s, fixtures, a pseudo provider, graph layout, the join
rule) and a thin adapter in `marley_workbench::knowledge` with an MCP client built on
`context_server`, as `harness.rs` and `fleet_providers.rs` already connect. Writes go only through
Rusty's tools, so its index, embeddings and git stay consistent.

- **K1, the project join.** Each rail project resolves to a brain project page (by `path:` in its
  frontmatter, then by name) and a Rusty task group. This is what "manage these projects" needs
  first, and everything below hangs off it. S.
- **K2, the Knowledge panel.** A right-dock panel: search, today's note, the current project's
  page, follow-ups due (`brain_due`), the project's tasks, and recent agent writes. Refresh on
  `resources/list_changed`. M.
- **K3, Page and Graph tabs.** A page opens as a center tab drawn from `brain_render`, so Marley
  does no wikilink parsing of its own. The graph tab draws `brain_graph` around the current page or
  project, with depth, filters and colour by link type; Rusty's `GraphView.qml` is the reference.
  M for the page, M for the graph.
- **K4, memory inside the work loop.** The pattern `herdr-projects` proved: when an agent starts on
  a project, its brief carries the project's memory (capped; herdr-projects uses 32,000
  characters), fenced as Hermes fences recall; lessons the agent marks under `## Remember` come
  back to the brain through `brain_capture` for the user to keep or drop. herdr-projects' README
  names the risk: whatever reaches memory repeats in every later brief, so a thread that writes a
  bad lesson poisons the next one. A lesson enters a brief only after the user keeps it. Off by
  default even with knowledge on (`agent_briefs: "off"`). M.
- **K5, capture from anywhere.** `marley: capture to brain` from the editor, a block or a browser
  pick, and voice notes (V2) into the daily page or inbox. S.

### Voice slices

- **V1, behind the switch.** The #480 mic moves under `marley.voice.enabled`. Changes a shipped
  default, so it waits for Chad's answer. S.
- **V2, spoken commands and voice notes.** `voxtype record start --file=<path>` hands Marley the
  transcript without any audio code, so D2 holds. A local grammar maps it to Marley actions and
  `marley_mcp` verbs ("open the fleet", "what's waiting", "send to the agent", "note:"). System One
  can be an optional second stage under its own `uses` entry for phrases the grammar leaves open.
  Anything irreversible is read back and needs a confirmation that expires (the
  `eliasstravik/herdr-call` rule), and text bound for an agent is inserted, never submitted. M.
- **V3, spoken output.** A one-sentence line when an agent finishes or blocks, rate limited, through
  `espeak-ng` now and Piper when installed, behind a provider setting. The sentence chunking and
  stop rules from Hermes apply. S to M.
- **V4, a spoken conversation.** Push-to-talk or continuous talk with the Agent Panel or an agent
  terminal, with barge-in. This means Marley captures audio, which overturns #480's D2; Zed's
  `crates/audio` gives capture and echo cancellation, and Hermes gives the pipeline. A provider
  is used only when named, never by fallback. L, and last.

### What the evidence says about order

K1, K2 and K4 first: herdr's users adopt memory inside the work loop, and Marley has nothing that
joins a project to what is known about it. V2 and V3 next, because they cost little once Voxtype's
file mode is used. The graph tab (K3) is what Chad pictured, and it is cheap because `brain_graph`
exists, but no community has shown a standalone graph pane gets daily use. V4 is the expensive one
and the one with the least demand behind it.

## Open questions for Chad

1. Amend D11 to read: Marley draws Rusty's brain (panel, pages, graph, project join) and Rusty stays
   the store and index?
2. Voice scope: commands, notes and spoken notifications through Voxtype (D2 holds), or also a full
   spoken conversation where Marley captures audio (D2 overturned)?
3. Move the dictation mic and `rusty_tools`, both on by default today, behind the new switches?
4. The brain's embeddings go to OpenAI today because Ollama is not installed. Is that acceptable for
   Marley's knowledge search, or should semantic search stay off until a local model runs?
5. herdr: build the read-only fleet provider (Part 1, 5)? And is a phone surface for approvals
   Marley's job or Rusty's?

## Chad's answers, 2026-10-02

1. "maybe rusty becomes Marley. We would take our rusty custom QML app and build it inside of
   marley. The mcp then lives inside of the marley ide but enabled/disabled." Not decided. Read
   against §20 and D19: Rusty's UI (30 QML screens, about 7.9k lines in `crates/rusty-app/qml`)
   would be rebuilt as Marley panels and tabs, and Marley would start `rusty-mcp` as a process of
   its own behind a switch, the way D19 runs `rh`, while it keeps running standalone for the Claude
   Code hooks, skills and CLI on the box that use it with no Marley open. Rusty is MIT. Linking
   `rusty-core` into Marley's process instead would need §20's brain boundary amended. Many of the
   screens map to things Zed and Marley already draw: agent page, transcript and terminal, command
   palette, quick switcher, file explorer, settings. The ones new to Marley: graph, notes with
   links and backlinks, brain search, bookmarks, memory, skills, tasks, secrets, and Rusty's
   decisions beside Marley's System One Decisions tab.
2. and 3. Asked for explanations; answered in the session, no answer yet.
4. Embeddings: "configurable". A provider setting under `marley.knowledge` (off, a local model,
   a hosted one), off by default, never falling back from local to hosted on its own.
5. herdr: "for herdr we need to reach over to the Rustal Harness agent and we want anything related
   to that built there. The harness will be absorbed into Marley or stand alone." Part 1's items 1
   to 6, the herdr adapter (item 5 here becomes the harness serving Marley's contract from herdr's
   API), the herdr-projects loop (K4's brief composition for agents the harness starts), progress
   reports, usage and quota, and the phone question went to the rustal-harness session on
   2026-10-02 to triage and ticket in that repo. Contract changes come back as entries in its
   `docs/planning/MARLEY_REQUESTS.md`. Marley keeps drawing: a weaker rail style for screen-read
   state, stale hosts dimmed with input off, #540 for its own terminals, and Part 3.

### The harness's triage, 2026-10-02

The rustal-harness session filed TICKET-091 to TICKET-095 under its `docs/planning/tickets/open/`
(decision D163, triage TICKET-090 closed, the reasons in its `docs/research/HERDR.md`):

- TICKET-091, M13's plan widened from Claude Code to any agent in its own client: reported state,
  seq, progress and resume argv with herdr's checks, a resume table past Codex and Claude Code,
  and quota sources (items 1, 2 and 9).
- TICKET-092, an optional bounded `wait_ms` on `session_send`; the refusal already exists (D120).
- TICKET-093, the systemd-logind delay lock.
- TICKET-094, a health ping and reconnects backing off to two minutes on M8's remote view.
- TICKET-095, the read-only herdr adapter, detected states labelled as detected.

Set aside there: screen rules as a source of state (M9 serves declared state only); PTY handoff
(tmux and the native backend already outlive the runtime); rolling layout copies (a client
concern, so Marley's); several hosts in one fleet (the harness's open choice 4, Chad's call); the
herdr-projects loop (M10's manager and M11's Brain already hold it; the memory-repeat risk goes to
rustal-brain); the phone surface (Chad's question).

Three requests for Marley's contract, in its `docs/planning/MARLEY_REQUESTS.md`, each an optional
field on `marley.work/v1`'s agent: MREQ-005 `state_source` (protocol, reported or detected, with
detected drawn weaker), MREQ-006 `progress` (percent and an activity line), MREQ-007 `quota` in
usage (windows with percent used and reset time, keyed by an account id that is not an email, no
money). Until Marley takes them they ride as session labels.

Open alignment question from the harness: it still serves the pinned `marley_fleet` session
envelope (0ba2872), while `marley.work/v1` says the central workflow store serves it. Which
contract the harness serves once Marley embeds it (D19) needs settling before TICKET-091's plan.

### Chad's answers on the harness's questions, 2026-10-02

- Several hosts in one fleet (the harness's open choice 4): "local or remote via ssh". Relayed to
  the harness.
- A phone surface for approvals: "paused until the end. will work via an app." Relayed.
- Which contract the harness serves once embedded, and the memory-repeat risk for rustal-brain:
  asked for plain explanations; no answer yet.

### The harness's D164, 2026-10-02

Chad answered the harness directly and it recorded his answers as D164. Approving agents from a
phone is Marley's work, paused until the end and through an app. On the contract: "We would want
to do what we have borrow ideas", so the harness keeps serving the pinned `marley_fleet` session
envelope and borrows from `marley.work/v1` rather than switching. On other machines: "marley will
handle this via ssh into a remote box", so Marley makes the SSH connection to each box and the
harness does not join hosts into one fleet.

What that leaves Marley:

- MREQ-005 to MREQ-007 now ask Marley to read session labels: `state.source` (`protocol`,
  `reported`, `detected`, with `detected` drawn weaker), `progress.percent` and
  `progress.activity`, and `quota.KIND.percent_used` with `quota.KIND.resets_at_ms`. The same
  values could become `marley.work/v1` fields later.
- TICKET-094 is withdrawn in the harness, so herdr's link habits (a health ping, reconnects backing
  off to two minutes, a stale host drawn dimmed with input off) belong to Marley's SSH connections:
  the remote terminals (#543) and the host collector (#610).

### Chad's answers, third round, 2026-10-02

- The contract split (the harness keeps its envelope, rustal-brain serves the work picture,
  Marley joins them): "ok that makes sense to me." He added: "also we need to brain storm
  integration into claude and codex using their tools instead of fighting them as well."
- The memory-repeat warning for rustal-brain: "this makes sense and i think that we are handing
  this another way so lets shelve this." Shelved.
- Voice (Part 3, V1 to V4; option A, commands and notes through Voxtype, or B, a spoken
  conversation): "shelve this for later but keep it open". Shelved, open.
- The dictation mic and `marley.rusty_tools`, both on wherever their program is installed: "Both
  off by default". Each gets a setting, off unless switched on.
- Rusty inside Marley: "Plan it now". A plan follows: Rusty's screens as Marley panels and tabs,
  Rusty's engine started by Marley behind a switch and still running on its own, the knowledge
  layer (Part 3, K1 to K5) as its first part.
- Correction to open question 4's premise: Rusty's embeddings never fall back to a hosted model.
  `embedding_provider` is `auto` (local Ollama only), `ollama`, `openai` or `off`, and OpenAI
  runs only when named (`rusty-core/src/brain/semantic.rs:1-9`). This box names `openai`.
  "Configurable" is already true; Marley exposes the setting (`docs/marley/rusty-in-marley.md`,
  R-D0).
- The plan for Rusty inside Marley: `docs/marley/rusty-in-marley.md` (slices R0 to R9).

### The manager and the human, 2026-10-02

Chad: "We have the rustal brain but then the Marley personal brain (for the human locally). In
between we are going to use the UCSOS pattern: Foreman -> Directs the agents just to make sure
they are runnin | Manager | Human (via Marley). We should probably have a way to communicate
between manager and human. One thing is i wonder if we should build a remote rust application
meant to be on the cloud (probably via a vpn) that will allow communication between Marley,
Manager, and the humans phone."

The harness already has the foreman (M10's supervision) and one manager per project with an
owner inbox (`rustal-harness/docs/MANAGER.md`). The manager's own question reaches the person in
the fleet, answered with `session_answer`; a person messages the manager with `session_send`.
Missing: a durable conversation between the manager and the person, the manager's reports, and
reach when the person is away. Asked when to build the conversation and a cloud relay, he chose
"Thread now, relay later": the manager and person thread is designed now in the harness and shown
in Marley; a relay (a mailbox on the VPN: per-device keys, end-to-end encrypted, tailnet only)
comes with the phone app at the end, carrying the same threads. The two brains stay apart: the
Rustal Brain is the agents' work memory, the personal brain (Rusty's, moving into Marley) stays
local, and only messages cross.
- The harness filed the thread as its TICKET-106 (roadmap M15, decision D166): ordered, durable
  records in its journal (author manager or person; kind message, report or confirmation;
  reply-to), manager-only tools to post, `rh mcp` grants and `rh thread` for the person, one
  delivery with a receipt, confirmations that read the action back and take one answer within
  60 seconds, read-only clients that can't post. MREQ-008 asks Marley to draw the thread beside
  the owner inbox (`owner://thread` with a cursor, a send verb with a delivery id, a confirm verb
  by id); names settle in TICKET-106.
