# A changed assistant agent reaches new conversations without a restart — Notes

- **Local ticket doc:** ../../tickets/closed/TICKET-814-a-new-assistant-agent-reaches-new-conversations-without-a-restart.md
- **Pipeline spec:** 814-a-new-assistant-agent-reaches-new-conversations.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-19 (`/goal`): "lets do 812, 814, 809, 806". Filed by #813's visual
  check, which restarts Marley before its Codex step to work around it.
- **Classification / tier:** bug; one Zed crate (`agent_ui`, an additive method) and one Marley
  crate (`marley_workbench`).
- **Recall (§18.3):**
  - L-claude-813-zed-keeps-an-agents-process-by-its-id-001: the store caches one connection per
    id and drops it only when the id leaves `agent_servers`, a registry version changes, the
    connection fails, or the user restarts it.
  - #813's scenario (`813-rusty-runs-commands-and-edits-files.sh`): `quit_marley` and
    `launch_marley` around `profile_setting marley.assistant.agent '"codex"'`.
  - The brain (consultation 2eaa438a0051457c8397f7bececcc007): nothing on this seam.
- **Discovery:**
  - `crates/agent_ui/src/agent_connection_store.rs`: `entries` private; `request_connection`
    returns the cached entry; `restart_connection` removes, then connects at once;
    `handle_agent_servers_updated` retains ids still in `external_agents`.
  - `crates/marley_workbench/src/assistant.rs`: `sync` runs `take_out` then `put_in` for an entry
    whose agent changed, under the same id; `put_in` reaches `set_entry` (now, or after `resolve`
    finds the adapter's command), which inserts `CustomAgentServerSettings::Custom` into the
    defaults' `agent_servers`.
  - `AgentPanel::connection_store()` is public; `crate::mcp::all_workspaces(cx)` lists every
    workspace from outside a window's update (`set_entry` runs from a settings observer or a
    spawned task's `cx.update`).
- **Decisions:** D1 to D3 in the spec.

### Design
- Approach:
  - `AgentConnectionStore::forget_connection(&mut self, key: &Agent, cx: &mut Context<Self>)`
    (Zed crate, new method, `// Marley:` comment): `self.entries.remove(key)` and `cx.notify()`
    when it held one. The removed entry's spawned listeners already check
    `this.entries.get(&key) != entry` and stop acting on it.
  - `set_entry` reads the defaults' current value for `entry.name()`
    (`raw_default_settings().agent_servers`) before it inserts, and when it differs from the new
    one calls `forget_connection(entry)`: for each workspace in `all_workspaces(cx)`, its
    `AgentPanel`'s store forgets `Agent::Custom { id: AgentId::new(entry.name()) }`.
- File manifest: `crates/agent_ui/src/agent_connection_store.rs` (Zed crate, a row in
  `zed-touchpoints.md` first); `crates/marley_workbench/src/assistant.rs` (Marley crate);
  `script/e2e/814-a-new-assistant-agent-reaches-new-conversations.sh` (new, sources #813's).
  Phase 4: #813's scenario comment that says Zed keeps the process whatever the settings become.

### Visual check plan
- REQ-001: a Rusty conversation on Claude Code (`814-01-rusty-claude`, `sandbox: no
  CODEX_CONFIG`), then `profile_setting marley.assistant.agent '"codex"'` with Marley running,
  then a new Rusty conversation: a new `session/new` in the agent's log, and the answer holds
  `entry: Rusty`, `sandbox: none` (`814-02-rusty-codex`).
- REQ-002: a new Marley conversation: `INITIAL_AGENT_MODE: read-only`, `sandbox: read-only`
  (`814-03-marley-codex`).
- REQ-003: `pane: activate previous item` twice to the first conversation, a prompt: the answer
  holds `entry: Rusty`, `sandbox: no CODEX_CONFIG` (`814-04-still-claude`).
- Once on the installed build (no fix): REQ-001's `expect` fails.

### Risks
- Two tabs back may not land on the first conversation if the palette's new conversations open
  elsewhere; the shot shows the tab, and the answer's `entry` and `sandbox` say which process
  answered.

## Phase 2 — Code
- Built to the manifest. `agent_connection_store.rs`: `forget_connection(&mut self, key: &Agent,
  cx)` after `restart_connection`, with a `// Marley:` comment; it notifies only when it removed
  an entry. Its `zed-touchpoints.md` row went in first. `assistant.rs`: `set_entry` compares the
  defaults' current `agent_servers` value for the entry with the new one before it inserts, and
  after the insert calls the new `forget_connection(entry, cx)`, which asks every workspace's
  `AgentPanel` store (`crate::mcp::all_workspaces`) to forget `Agent::Custom { id:
  AgentId::new(entry.name()) }`.
- `script/e2e/814-a-new-assistant-agent-reaches-new-conversations.sh` sources #813's scenario and
  replaces its steps, as planned.
- Review: the comparison runs before the insert, so an unchanged entry (a repeated `resolve`)
  keeps its process; the store's spawned listeners for a removed entry already stop on
  `this.entries.get(&key) != entry`; `set_entry` runs from the settings observer or a task's
  `cx.update`, never inside a window's update, so `all_workspaces` sees every window
  (PR-claude-596); the forget happens after the defaults change, and the agent server store reads
  the new command when a conversation next connects. Zed hunk: one method, additive.
- The gate: `just gate-diff` green, 17 of 17.

## Phase 3 — Test
- `just build`, then `just e2e script/e2e/814-a-new-assistant-agent-reaches-new-conversations.sh`
  under sway, `SHOT_DIR` in the scratchpad. Focus report: "1 Marley windows before the run, 1
  after; the run added no rule". Every `expect` passed. Each shot read:
  - `814-01-rusty-claude`: a Rusty tab (the butler) answers `entry: Rusty`, `disallowed:
    mcp__marley`, `INITIAL_AGENT_MODE: unset`, `sandbox: no CODEX_CONFIG`: a Claude Code process.
  - `814-02-rusty-codex`: after `marley.assistant.agent` became `codex` with Marley running, a
    second Rusty tab answers `entry: Rusty`, `disallowed: none sent`, `sandbox: none`: a Codex
    process with Rusty's instructions (REQ-001).
  - `814-03-marley-codex`: a Marley tab (the cat) answers `entry: Marley`, `no-commands line: yes`,
    `INITIAL_AGENT_MODE: read-only`, `sandbox: read-only` (REQ-002).
  - `814-04-still-claude`: two `pane: activate previous item` later the first Rusty tab is in
    front, and its second prompt, `are you still there`, is answered `disallowed: mcp__marley`,
    `sandbox: no CODEX_CONFIG`: its Claude Code process still serves it (REQ-003).
- Without the fix (`E2E_BINARY`, the installed build): the new Rusty conversation after the change
  answered `entry: unknown`, `sandbox: no CODEX_CONFIG`, the old process with its session meta
  gone, as the ticket says, and REQ-001's `expect` failed.

## Phase 4 — Complete
- Docs: `CHANGELOG.md` (Fixed, #814); `docs/marley_architecture/marley_workbench.md`, the #687
  bullets of the Marley agent; the `zed-touchpoints.md` row for `agent_connection_store.rs` read
  against what shipped. #813's scenario comment no longer says Zed keeps the process whatever the
  settings become, so the gate ran again before the commit. No slice status moves.
- Ledger appends: `F-claude-814-a-changed-assistant-agent-kept-the-old-process-001`,
  `PR-claude-814-a-rewritten-agent-entry-forgets-its-connection-001`. L-claude-813 stays as
  history: it is still true of Zed, and the PR says what Marley does about it.
- Brain: consultation `2eaa438a0051457c8397f7bececcc007` closed with `brain decide`,
  `decisions/marley-tells-zeds-agent-connection-store-to-forget-an-entry-it-rewrote` (follow-up
  2026-11-19).
- Ticket closed; its `BACKLOG.md` row left at promotion.
