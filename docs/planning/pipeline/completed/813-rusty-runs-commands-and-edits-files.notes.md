# Rusty's agent runs commands and edits files — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-813-rusty-runs-commands-and-edits-files.md
- **Pipeline spec:** 813-rusty-runs-commands-and-edits-files.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-18, after Rusty in the Agent Panel refused to check a system process:
  "it probably makes sense to allow the rusty agent certain permissions inside here. That way if
  someone wants to use it as a personal assistant then it can have access." Asked one question at a
  time: "Commands + file edits", and "Comes with Rusty" (no new setting). He approved the plain
  sentence ("correct").
- **Classification / tier:** feature, one Marley file, no Zed crate. Small.
- **Checklist:** pick ✓ (named, new ticket #813; no BACKLOG row to remove), pre-flight ✓ (cargo
  idle, gate and e2e present, hooks wired, no active pipeline, README marker present), recall ✓,
  mint ✓, prior art ✓, spec ✓, design ✓.
- **Recall (§18.3):**
  - AD-claude-683 / AD-claude-687: the entries' prompt and limits ride `marley.agent_session_meta`
    (Claude Code) and `CODEX_CONFIG` plus `INITIAL_AGENT_MODE` (Codex); Zed's agent gets a profile.
  - L-claude-696: a session `_meta` carries an MCP server for one entry alone; `disallowedTools`
    takes `mcp__<server>` to keep a whole server out.
  - AD-claude-779: Marley and Rusty entries send `allowDangerouslySkipPermissions: true`, and
    `marley.agent_default_modes` starts them in bypass or full access; "their disallowed tools
    stay". This ticket is where Rusty's go.
  - The brain (consultation d5f6a13f…): nothing on this seam.
- **Discovery:** all in `crates/marley_workbench/src/assistant.rs`.
  - `DISALLOWED_TOOLS` (line 93) goes into every entry's `session_meta` (line 595).
  - `codex_config` (line 629) sets `sandbox_mode: read-only` for both entries; `set_entry` (line
    833) sets `INITIAL_AGENT_MODE=read-only` for both.
  - `profile(entry)` (line 659) gives both profiles `tools: IndexMap::default()`; Rusty's has
    `enable_all_context_servers: true`. Its one caller is `put_in` (line 531), which has `cx`.
  - `RUSTY_INSTRUCTIONS` (line 153) ends with the no-commands paragraph.
  - codex-acp 2.2.2 (`npm pack` into the scratchpad): `INITIAL_AGENT_MODE` unset gives
    `DEFAULT_AGENT_MODE`, `agent` ("Auto review": workspace-write sandbox, on-request approvals,
    asks for actions judged unsafe).
  - `agent_settings::builtin_profiles::WRITE` is public; `SettingsStore::raw_default_settings()`
    holds `agent.profiles.write.tools` from `default.json`.
  - Docs that state the limit: `docs/marley/guide.md` line 2445 ("It changes no file and runs no
    command") and the Claude Code bullet below it.
- **Decisions:** D1–D7 in the spec.

### Design
- **Approach:** the limits become the Marley entry's alone; Rusty's entry keeps only what makes it
  Rusty (its prompt, its server, Marley's tools kept out).
  - `session_meta`: `DISALLOWED_TOOLS` and `marley_tools_kept_out()` only for `Entry::Marley`;
    `mcp__marley` for Rusty stays where it is (the `if let Some(server) = rusty` arm).
  - `codex_config`: `sandbox_mode: read-only` only for `Entry::Marley`.
  - `set_entry`: `INITIAL_AGENT_MODE=read-only` only for `Entry::Marley`.
  - `profile(entry, cx)`: Rusty's `tools` are the Write profile's from
    `cx.global::<SettingsStore>().raw_default_settings()` (`agent.profiles[WRITE].tools`, empty if
    absent, which it never is); Marley's stay empty. `put_in` passes `cx`.
  - `RUSTY_INSTRUCTIONS`: the last paragraph replaced by a bullet: run commands and edit files when
    the user asks for something on their machine; change Rusty's own data only through the rusty
    server's tools.
  - Doc comments: the module header (the Claude Code, Codex and Zed bullets and the Rusty
    paragraph), `DISALLOWED_TOOLS`, `session_meta`, `codex_config`, `profile`.
  - §14: no new IO, spawn or panic path; the profile read is an `Option` chain.
- **File manifest:**
  - `crates/marley_workbench/src/assistant.rs` — Marley crate. No Zed crate, so no ledger row.
  - `script/e2e/813-rusty-runs-commands-and-edits-files.sh` — new, at Test.
  - `docs/marley/guide.md`, `CHANGELOG.md`, the architecture doc — at Complete.

### Visual check plan
Set-up as #738's: Rusty on through the stand-in `rusty-mcp`; a fake `claude` signed in;
`MARLEY_ASSISTANT_ADAPTER` a scripted ACP agent that logs each message and its own environment, and
answers any prompt with what its session got: `disallowed: …`, `no-commands line: yes|no`,
`rusty-tools line: yes|no`, `INITIAL_AGENT_MODE: …|unset`, `sandbox: …|none`. Conversations start
from the palette, so keys alone; Hyprland's hidden workspace unless a step needs a click.

| REQ | What the scenario does | Shot |
|---|---|---|
| REQ-001, REQ-002 | `marley: new rusty conversation`, type `what were you given` | `813-01-rusty-claude`: the answer shows `disallowed: mcp__marley`, `no-commands line: no`, `rusty-tools line: yes` |
| REQ-003 | `marley: new marley conversation`, the same prompt | `813-02-marley-claude`: Bash, Edit, Write, NotebookEdit, MultiEdit among the disallowed, `no-commands line: yes` |
| REQ-004 | `profile_setting marley.assistant.agent codex` (a fake `codex` signed in if the entry needs one), a new Rusty conversation, then a new Marley one, each asked | `813-03-rusty-codex`: `INITIAL_AGENT_MODE: unset`, `sandbox: none`; `813-04-marley-codex`: `read-only` both |
| REQ-005 | `profile_setting marley.assistant.agent zed`, `agent: manage profiles`, Rusty, its built-in tools | `813-05-rusty-profile`: Terminal and Edit File on |

Not reachable by a scenario: the real Claude Code, Codex or a Zed model running a command for
Rusty, which needs a live login and a model; the scenario stops at what each session is given.
The real check is Chad's after the install: ask Rusty to check a process.

### Risks
- Rusty's working folder is Rusty's vault (#780), so a file edit there lands in the brain beside
  Rusty's own writes. Rusty's watcher records outside edits, but the instructions keep Rusty's data
  on its tools (D5).
- Codex's `agent` mode asks only for actions it judges unsafe, not before every command; that is
  Codex's own behavior in any thread (D3). Reversible by setting `INITIAL_AGENT_MODE` for Rusty.
- A user who removed `write` from their own profiles does not change `raw_default_settings`, so
  Rusty's tools come from Zed's defaults either way.

## Phase 2 — Code
- **Checklist:** `assistant.rs` ✓ (the one manifest file); README marker present; no Zed path, no
  ledger row.
- **Built:** in `crates/marley_workbench/src/assistant.rs`:
  - `session_meta`: `DISALLOWED_TOOLS` and `marley_tools_kept_out()` only for `Entry::Marley`;
    Rusty's `disallowedTools` is `mcp__marley` alone, from the existing Rusty-server arm.
  - `codex_config`: `sandbox_mode: read-only` only for Marley. `set_entry`: `INITIAL_AGENT_MODE`
    only for Marley, with a comment saying Rusty's start in codex-acp's default.
  - `profile(entry, cx)`: Rusty's `tools` are `raw_default_settings().agent.profiles["write"].tools`
    (`agent_settings::builtin_profiles::WRITE`), empty if absent; `put_in` passes `cx`, read
    before its `update_global`, so no re-entrant settings read.
  - `RUSTY_INSTRUCTIONS`: the no-commands paragraph gone; the data bullet says "through the rusty
    server's tools and never by editing its file"; a new bullet: on the user's machine, run the
    commands and edit the files a request needs and say what was run and changed.
  - Doc comments: the module header's three agent bullets and Rusty paragraph, `DISALLOWED_TOOLS`,
    `session_meta` (its stale "its own eight" now "its own"; the list has ten since #779),
    `codex_config`, `profile`.
- **Deviations:** none.
- **Review of the diff:**
  - REQ-001: Rusty's `rusty` is always `Some` (`Entries::of`), so `mcp__marley` is always there;
    nothing else. REQ-002: both the Claude Code append and Codex's `developer_instructions` read
    `entry.instructions()`. REQ-003: Marley's list is the same two lists in the same order.
    REQ-004: both Codex keys gated on `Entry::Marley`; #779's full-access mode is untouched.
    REQ-005: the Write profile comes from `default.json`, loaded before `marley_workbench::init`.
  - `open marley agent in terminal` still uses `DISALLOWED_TOOLS` and `--sandbox read-only`: the
    Marley agent's alone, unchanged.
  - Provenance: no Zed function body carried; only Zed's public `builtin_profiles::WRITE` and the
    settings store read.
  - Found: #696's own scenario still expects Bash and the rest disallowed for Rusty, and
    `allowDangerouslySkipPermissions: false`, stale since #779. It is not in the golden set and
    per-ticket scenarios stay as history (§7), so it is left; 813's scenario covers the new state.
- **Gate:** `just gate-diff` → `GATE GREEN [diff]`, every gate PASS (1, 2, 7–14, 16–18, 20–22).
  Log: the session scratchpad's `gate-813.log`.

## Phase 3 — Test
- **Scenario:** `script/e2e/813-rusty-runs-commands-and-edits-files.sh`, `compositor sway`, the
  debug `marley` from `just build`; Rusty on through the stand-in `rusty-mcp`; a scripted ACP agent
  as `MARLEY_ASSISTANT_ADAPTER` answering "what were you given" with its session's `_meta` and its
  own environment; conversations from the palette; a key bound to
  `agent::ManageProfiles {customize_tools: "rusty"}` for the tool picker.
- **First run:** Claude Code's two steps passed; the Codex step failed (`entry: unknown`,
  `disallowed: none sent`, `sandbox: no CODEX_CONFIG`). Cause: Zed's `AgentConnectionStore`
  (`agent_ui/src/agent_connection_store.rs`) caches a connection by agent id alone and drops it
  only when the id leaves `agent_servers` or a registry version changes, so after
  `marley.assistant.agent` moved to `codex` the new Rusty conversation ran on the Claude Code
  phase's process, whose meta had since been taken out. Not this ticket's change: pre-existing
  since #687, and the same for a user who switches the agent mid-session (new conversations stay on
  the old adapter until a restart). Filed as its own ticket at Complete. The scenario now quits
  Marley, sets `codex` and relaunches before the Codex step.
- **Second run:** every check passed. The shots:
  - `813-01-rusty-claude` (REQ-001, REQ-002): a "Rusty · idle" conversation; the answer reads
    entry Rusty, disallowed `mcp__marley`, no-commands line no, rusty-tools line yes. Met.
  - `813-02-marley-claude` (REQ-003): a "Marley · idle" conversation; disallowed Bash, Edit,
    Write, NotebookEdit, MultiEdit and 66 of Marley's tools by name; no-commands line yes;
    rusty-tools line no. Met.
  - `813-03-rusty-codex` (REQ-004): after the relaunch, a Rusty conversation; entry Rusty,
    `INITIAL_AGENT_MODE: unset`, `sandbox: none`, no-commands line no, rusty-tools line yes (the
    instructions arrive as `developer_instructions`). Met.
  - `813-04-marley-codex` (REQ-004): a Marley conversation; `INITIAL_AGENT_MODE: read-only`,
    `sandbox: read-only`, no-commands line yes. Met.
  - `813-05-rusty-profile` (REQ-005): "Rusty — Configure Built-in Tools"; every tool ticked but
    `ask_user` (off in Zed's Write profile too), `edit_file` among them. Met.
  - `813-06-rusty-terminal` (REQ-005): the picker filtered to `terminal`, ticked. Met.
  - `813-07-rusty-edit` (REQ-005): filtered to `edit`, `edit_file` ticked. Met.
  - `settings_read` checks: `agent.profiles.rusty` turns on 23 tools, `terminal`, `edit_file` and
    `write_file` among them; `agent.profiles.marley` turns on none and keeps Marley's ten.
- **Seen, not in scope:** after the relaunch the two earlier tabs read "Error Loading Rusty" and
  "Error Loading Marley": the scripted agent offers no `loadSession`, so a restored thread can't
  load. A stand-in limit, not Marley's.
- **Focus report:** "hyprland: 1 Marley windows before the run, 1 after; the run added no rule
  and did not reload it"; sway stopped with the run's Marley, pointer and keyboard.
- **Not reachable:** a real Claude Code, Codex or Zed model running a command for Rusty (a live
  login and a model). The scenario stops at what each session is given; Chad's check after the
  install is to ask Rusty to check a process.
- **No source fix**, so the Phase 2 gate stands for the Rust; the scenario file is new since that
  run, and the gate runs again before the commit.

## Phase 4 — Complete
- **Docs (§21):** `CHANGELOG.md` (Changed: "Rusty runs commands and edits files");
  `docs/marley/guide.md` "Rusty in the Agent Panel" (the intro, the Claude Code, Codex and Zed's
  agent bullets); `docs/marley/walkthrough.md` stop 2.16a "Rusty runs a command"; the architecture
  note `docs/marley_architecture/marley_workbench.md` ("Since #813" under the assistant). The plan
  (`three-prong-plan.md`) does not track the Agent Panel entries, so no slice row changes. No Zed
  path, so no touchpoint row.
- **Knowledge (§19):** `L-claude-813-zed-keeps-an-agents-process-by-its-id-001`,
  `AD-claude-813-rustys-entry-is-the-agent-it-runs-on-001`. No F- block: the Test phase's red was
  the pre-existing connection cache, not this change; it is TICKET-814 (queued second).
- **Brain:** consultation d5f6a13f… closed with `brain_decide` →
  `decisions/rustys-agent-in-marley-runs-commands-and-edits-files`, follow-up by 2026-11-01.
- **Ticket:** TICKET-813 closed; no BACKLOG row (it never had one). TICKET-814 filed with its row.
- **Try after the install:** walkthrough 2.16a: ask Rusty whether `rusty-mcp` is running.
