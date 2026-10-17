# A default mode keeps Marley's own agent entry — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-793-a-default-mode-hides-marleys-agent-entry.md
- **Pipeline spec:** 793-a-default-mode-keeps-marleys-agent-entry.spec.md

## Phase 1 — Plan
- **Request:** found in #791's Test phase (2026-10-16): New Agent found no Rusty, in #791's run and
  in #780's own scenario run again, because the run's profile copies the box's settings with
  `agent_servers.Rusty` as a `registry` entry. Chad asked for whatever is left to be done.
- **Classification / tier:** bug; one Zed crate (`settings_content`), a hunk and a ledger row.
- **Recall (§18.3):**
  - L-claude-779-zeds-mode-selector-saves-each-pick-as-the-agents-default-001: the mode selector
    saves each pick as the agent's `default_mode`, and `CustomAgentServerSettings` merges whole,
    so "a value Marley puts in Zed's `agent_servers` defaults loses to any user entry for that
    agent". #779 kept its modes out of the entry (`marley.agent_default_modes`).
  - L-claude-791-a-scenarios-profile-copies-the-users-agent-servers-001: #791's scenario drops
    `agent_servers.Rusty` from its copy until this ticket.
  - AD-claude-683 and #696: the Marley and Rusty entries are custom agent servers in the defaults,
    with the adapter's command and `_meta`.
  - The brain (consultation cfd7922a): nothing on this seam.
- **Discovery:** `settings_content/src/agent.rs:840-902` (`CustomAgentServerSettings`, derive
  `MergeFrom`); `settings_macros.rs:61` (an enum's merge replaces); `merge_from.rs:89` (`HashMap`
  merges key by key); `agent_servers/src/custom.rs:98,135,173` (the three writers),
  `:325` (`default_settings_for_agent`, a `Registry`); `acp.rs:373-401` (`refresh_from_settings`)
  and `:1400` (`apply_default_config_options`); `assistant.rs:516` (`put_in`).
- **Decisions:** D1 and D2 in the spec.

### Design
- **`crates/settings_content/src/agent.rs`** (Zed crate): drop `MergeFrom` from the enum's derive
  and add `impl MergeFrom for CustomAgentServerSettings` below it, with a `// Marley:` comment:
  ```rust
  match (&mut *self, other) {
      (Self::Custom { env, default_mode, default_config_options, favorite_config_option_values, .. },
       Self::Registry { env: user_env, default_mode: user_mode, default_config_options: user_options,
                        favorite_config_option_values: user_favorites }) => {
          env.merge_from(user_env);           // key by key, the user's winning
          if user_mode.is_some() { default_mode.clone_from(user_mode); }
          ... the two maps the same way
      }
      (this, other) => *this = other.clone(),
  }
  ```
  The maps' value types need `MergeFrom` for `HashMap::merge_from`; if a value type lacks it, a
  plain insert of each of the user's keys does the same.
- **`docs/marley/zed-touchpoints.md`**: a row for `crates/settings_content/src/agent.rs` first.
- No Marley crate changes.

### Visual check plan
| Criterion | The scenario | The shot |
|---|---|---|
| REQ-001 | The profile's `agent_servers.Rusty` = `{"type": "registry", "default_config_options": {"mode": "bypassPermissions"}}`; New Agent, type Rusty | `793-01-rusty-in-picker` |
| REQ-001, REQ-002 | Enter; `expect` the agent's log has `session/new`, then `session/set_config_option` with `mode` and `bypassPermissions` | `793-02-rusty-thread` |
| REQ-003 | The profile's `agent_servers.Marley` = `{"type": "registry", "default_mode": "plan"}`; New Agent, Marley, Enter; `expect` `session/set_mode` `plan` | `793-03-marley-thread` |
| REQ-004 | Not run: the other pairs keep the derive's rule, read in the diff | review |

The scripted agent answers `session/new` with `modes` (`default`, `plan`, `bypassPermissions`,
current `default`) and a `mode` select config option of the same values, takes
`session/set_mode` and `session/set_config_option`, and logs every message (#779's and #780's
agents, joined). The scenario keeps `claude_code_permissions` at its default (`ask`), so
`marley.agent_default_modes` names no mode and the entry's own defaults are what reach the
session.

### Risks
- A user who writes a `registry` entry by hand under the name of a Marley entry now gets
  Marley's agent with the preferences, not a registry lookup that finds nothing. That is the
  intent.
- An upstream merge that changes the enum's fields: the hand-written impl stops compiling and
  says so; the row says to carry it.

## Phase 2 — Code
- **Built, to the manifest:**
  - `docs/marley/zed-touchpoints.md`: the row for `crates/settings_content/src/agent.rs`, first.
  - `settings_content/src/agent.rs`: `MergeFrom` out of `CustomAgentServerSettings`'s derive (a
    `// Marley:` line says where it went) and `impl crate::merge_from::MergeFrom` below the enum.
    `(Custom, Registry)`: `env`, `default_config_options` and `favorite_config_option_values`
    take each of the user's keys (`extend`), `default_mode` the user's when set; `path` and `args`
    stay. Any other pair: `*this = other.clone()`, the derive's body.
- **Deviations:** none. The maps take the user's values with `extend` rather than
  `HashMap::merge_from`, which would merge a favourites `Vec` by its own rule; the user's value
  for a key wins whole, as the replacing merge gave it before.
- **Review of the diff:**
  - REQ-001 to REQ-003 rest on the `(Custom, Registry)` arm; REQ-004 on the fallback arm, the
    derive's body unchanged.
  - Zed's writers find the user's `registry` entry on the next write (`entry(..).or_insert_with`)
    and change its fields; the merge keeps laying them over Marley's entry.
  - `put_in` replaces Marley's default entry when its command changes; the user's preferences
    stay in the user's file and keep applying.
  - The hunk is one derive line and one added impl; the row is written.
- `just clippy settings_content` clean.
- **Gate:** `just gate-diff` GREEN, 17 passed, 0 failed (`gate-793.log`).

## Phase 3 — Test
- **The scenario:** `script/e2e/793-a-default-mode-keeps-marleys-agent-entry.sh`, `compositor sway`,
  keys only. The profile's `agent_servers.Rusty` is the box's entry and `agent_servers.Marley` the
  mode selector's; the scripted agent behind both entries; Rusty on through its stand-in; the
  permission settings dropped from the copy.
- **Runs:**
  1. REQ-001 and REQ-002 passed; "the Marley session was set to plan: FAIL". The work folder is
     removed after a run, so the scenario now prints the agent's requests (`agent_log_summary`).
  2. The summary showed the Marley session got no `session/set_mode`. Zed's `config_state`
     (`acp.rs:7244`) drops a session's modes when it has config options, and the scripted agent
     offered both, so the Marley entry's `default_mode` had no mode to set. A scenario fault, not
     the change's: the agent now offers the config option to a session whose `_meta` names
     Rusty's server (`claudeCode.options.mcpServers.rusty`) and modes to any other.
  3. `e2e exit 0`. In order, the agent got: `initialize`, `session/new`,
     `session/set_config_option {configId: mode, value: bypassPermissions}` (Rusty);
     `initialize`, `session/new`, `session/set_mode {modeId: plan}` (Marley). Every check passed.
- **The shots, read:**
  - `793-01-rusty-in-picker` (REQ-001): the New Agent picker over the terminal, query "Rusty", one
    row, Rusty, with "Thread" on its right.
  - `793-02-rusty-thread` (REQ-001, REQ-002): a New Agent Thread tab, "Message Rusty — @ to include
    context", the rail's row "New Agent Thread · Rusty · idle".
  - `793-03-marley-thread` (REQ-003): a second New Agent Thread tab, "Message Marley — @ to include
    context", the rail's rows for Marley and Rusty, both idle.
- **The bug's own evidence, again:** #780's scenario, which failed on the build before this change
  with the box's settings in its copy ("Rusty started in its vault: FAIL", in #791's Test phase),
  passes on this build (`e2e exit 0`, every check). #791's scenario no longer drops
  `agent_servers.Rusty` from its copy, and passes (`e2e exit 0`).
- **Focus report:** the keys reached Marley's window in the run's own sway; the note #563 shows
  after Ctrl+Alt+N from a terminal sits at the bottom right in each shot, as L-claude-779 says.
- **REQ-004:** not run; the fallback arm is the derive's body, read in Phase 2.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Fixed, #793); `docs/marley/guide.md` ("Your defaults for it", under
  Rusty's agent); `docs/marley/zed-touchpoints.md` (the row for `settings_content/src/agent.rs`,
  written before the code, matches what shipped). No Marley crate changed, so no per-crate note;
  the slice is cross-cutting, with no plan line to move.
- **Ledger:** F-claude-793-a-default-mode-hid-marleys-own-agent-entries-001,
  PR-claude-793-a-default-marley-puts-in-zed-settings-survives-zeds-writers-001,
  L-claude-793-zed-drops-a-sessions-modes-when-it-has-config-options-001,
  AD-claude-793-a-user-registry-entry-adds-its-preferences-to-a-default-custom-agent-001.
- **Brain:** consultation cfd7922a closed with `brain decide`:
  `decisions/a-users-registry-agent-entry-adds-its-preferences-to-marleys-own-custom-agent-entry`
  (follow-up by 2026-11-16).
- **Also in this commit:** #791's scenario without its `agent_servers.Rusty` workaround, and
  #791's completed notes with the Mac and real-Rusty results.
- **Ticket closed, pipeline archived, committed** with the gate's green receipt (`gate-793b.log`).
