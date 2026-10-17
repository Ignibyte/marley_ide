---
pipeline_id: f631ff1b-c74c-4099-8458-6b4ec9511b08
ticket: docs/planning/tickets/open/TICKET-793-a-default-mode-hides-marleys-agent-entry.md
status: Phase 4 — Complete PASS
title: A default mode keeps Marley's own agent entry
type: bug
slice: Cross-cutting (the Marley and Rusty agents)
references:
  - docs/planning/pipeline/completed/779-permission-modes-for-every-agent.spec.md
  - docs/planning/pipeline/completed/791-rustys-vault-on-another-machine.notes.md
---

## Title
Marley puts its Marley and Rusty entries in the settings' defaults as custom agent servers
(`assistant.rs`, `put_in`): the adapter's command, its environment and the session's `_meta`.
When the user picks a mode in such a thread, or makes a config option a default, Zed's
`CustomAgentServer` (`agent_servers/src/custom.rs`: `set_default_mode`,
`set_default_config_option`, `toggle_favorite_config_option_value`) writes `agent_servers.<name>` into
the user's settings file. The user's file had no entry, so Zed makes one with
`default_settings_for_agent`, a `registry` entry, and the settings merge (`MergeFrom` derived on
an enum) puts the user's entry in place of the default one. The entry then names a registry agent
called Marley or Rusty, which the registry does not have, and it leaves the New Agent picker and
the title bar. The box's settings hold `agent_servers.Rusty` with `"mode": "bypassPermissions"`,
so the installed Marley has no Rusty entry now.

## Scope
### In
- `CustomAgentServerSettings`'s merge: a user's `registry` entry, which carries only preferences
  (`env`, `default_mode`, `default_config_options`, `favorite_config_option_values`), over a
  default `custom` entry keeps the default's command and arguments and takes the user's
  preferences, key by key for the maps.

### Out (explicitly deferred)
- Every other pair merges as before: a user's `custom` entry replaces a default one whole, and a
  `registry` default takes the user's entry whole.
- Zed's writers in `agent_servers/src/custom.rs`: unchanged; they keep writing `registry` entries.
- Any edit of the user's settings file by Marley.
- Which mode Marley's permission settings give (#779): unchanged, still first.

## Reference (§20)
Upstream Zed, the `settings_content` and `agent_servers` crates. Zed's own built-in agents are
`registry` entries in its defaults, so the entry Zed's writers make (`default_settings_for_agent`)
matches the default and the merge loses nothing. Marley's entries are the only `custom` entries in
the defaults; the change gives the merge the rule Zed's writers already assume, that a user entry
carrying only preferences sits on top of the agent it names. Behavior kept: an entry the user
writes with a command replaces the default, as before.

### Prior art
- **Zed's crates.** `settings_macros::MergeFrom` derived on an enum does `*self = other.clone()`;
  on a struct it merges field by field, and `MergeFrom` for `HashMap` merges key by key
  (`settings_content/src/merge_from.rs`). Hand-written `MergeFrom` impls already exist where a
  derive's rule is wrong for a type (`MarleyActionNames`, #711, a replacing merge for a list).
- **`agent_servers/src/custom.rs`.** `set_default_mode` (:128), `set_default_config_option` (:159)
  and `toggle_favorite_config_option_value` (:81) all `entry(name).or_insert_with(default_settings_for_agent)`, then
  set a field on either variant. `project::agent_server_store` turns the merged entry into the
  agent: `Custom` with a command is a custom agent, `Registry` is looked up in the registry.
- **`agent_servers/src/acp.rs`.** A session's defaults come from the merged entry
  (`refresh_from_settings`): `marley_default_mode` first (#779), then the entry's `default_mode`;
  `apply_default_config_options` sends `session/set_config_option` for each default the agent
  offers.
- **The ledger.** L-claude-779 named the trap ("a value Marley puts in Zed's `agent_servers`
  defaults loses to any user entry for that agent"), and #779 kept its modes out of the entry.
- **Rejected:** a `custom` entry written by Zed's writers when the merged entry is custom (it would
  copy Marley's command, a path on this machine, into the user's file, frozen there); Marley
  moving the user's entry out of the settings file (Marley does not edit the user's settings);
  keeping Marley's entries out of the picker's way some other way (the picker reads Zed's store).

## UI proof
`script/e2e/793-a-default-mode-keeps-marleys-agent-entry.sh`, `compositor sway`, keys only. The
run's profile has `agent_servers.Rusty` as the box's settings have it (`registry`,
`default_config_options.mode` `bypassPermissions`) and `agent_servers.Marley` as Zed's mode
selector writes it (`registry`, `default_mode` `plan`); the Marley and Rusty entries on a scripted
ACP agent that offers session modes and a `mode` config option, and logs every message; Rusty on
through its stand-in.
- `793-01-rusty-in-picker`: New Agent, "Rusty": the Rusty row in the picker (REQ-001).
- `793-02-rusty-thread`: Enter: a Rusty conversation; the agent logged `session/new` and a
  `session/set_config_option` for `mode` = `bypassPermissions` (REQ-001, REQ-002).
- `793-03-marley-thread`: New Agent, "Marley", Enter: a Marley conversation; the agent logged a
  `session/set_mode` to `plan` (REQ-003).

## Locked-In Decisions
- D1 — The fix is the merge, in `settings_content`: a hand-written `MergeFrom` for
  `CustomAgentServerSettings` in place of the derive. `Custom` (self) with `Registry` (other)
  keeps `path` and `args`, merges `env`, `default_config_options` and
  `favorite_config_option_values` key by key with the user's winning, and takes `default_mode`
  when the user's is set. Every other pair is `*self = other.clone()`, the derive's rule.
- D2 — Nothing writes to the user's settings to repair them: the merge reads the entries the box
  has now correctly.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the user's settings hold a `registry` entry named Rusty with only preferences, the New Agent picker shall list Rusty, and choosing it shall start Marley's adapter for it | `793-01-rusty-in-picker`, `793-02-rusty-thread`; the agent's log |
| REQ-002 | WHEN such a Rusty thread starts, its session shall get the user's default config option | the agent's log: `session/set_config_option` `mode` = `bypassPermissions` |
| REQ-003 | WHILE the user's settings hold a `registry` entry named Marley with a `default_mode`, the Marley entry shall start its agent and the session shall get that mode | `793-03-marley-thread`; the agent's log: `session/set_mode` `plan` |
| REQ-004 | WHERE a user's entry has a command (`custom`), or the default is a `registry` entry, the merge shall give the user's entry, as before | the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the touchpoint row, then `settings_content/src/agent.rs`; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
