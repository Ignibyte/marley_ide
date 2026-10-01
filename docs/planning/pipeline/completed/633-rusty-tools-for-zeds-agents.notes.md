# Rusty's tools for Zed's agents — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-633-rusty-tools-for-zeds-agents.md
- **Pipeline spec:** 633-rusty-tools-for-zeds-agents.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-01)
- **Request:** Chad, 2026-09-30 (wave 4); plan C2 and D11.
- **Recall (§18.3):**
  - An Explore read of Rusty (2026-10-01): rusty-mcp has no authentication; stdio is
    `rusty-mcp` with no arguments (Rusty's `.mcp.json`), HTTP is `127.0.0.1:4174/mcp`; the brain
    loop is `brain_ask`, `brain_decide`, `brain_no_decision`, `brain_follow_up`, `brain_due`;
    rusty-mcp exposes no agent-session tools; Rusty's constitution: "The back end is MCP only."
  - #501 adds `marley` to Zed's defaults with `update_default_settings`; a user's own entry wins.
  - Privacy: nothing of the user's Rusty (its data paths, its tokens, its pages) goes into Marley's
    public docs or fixtures; the scenario's server is a stand-in.
- **Risk to settle at promotion:** how the scenario puts a stand-in `rusty-mcp` on Marley's own
  search path (the harness may pass Marley's environment, or Marley reads the login shell's PATH).

## Phase 1 — Plan (promoted 2026-10-01)
- **Recall (§18.3):** the queued entry above; #501 (`offer_to_zeds_agents` in `mcp.rs`: a stdio
  server added with `update_default_settings`, a user's own entry winning). Brain: nothing on this
  seam.
- **Re-verified:** `agents::launcher(cx).search_path` is Marley's own `PATH` at start, so a
  scenario's exported `PATH` reaches it; the e2e harness copies `~/.config/marley/settings.json`
  into each run's profile (`script/e2e.sh`), so a user who has Rusty would see every scenario start
  the real `rusty-mcp` once this ships: the harness sets the switch off in the copy. Zed's Settings
  window's MCP Servers page (`settings_ui/src/pages/mcp_servers_page.rs`) lists each server with
  its `ContextServerStatus`.

### Design
- **Setting:** `marley.rusty_tools: Option<bool>`, on by default (`settings_content`,
  `default.json`), `MarleySettings::rusty_tools` as `RustyTools::{Offered, Off}`, and a Rusty
  Tools for Agents toggle in the Marley page's Agents section.
- **`marley_workbench::rusty`** (new): a global `RustyOffer(Option<PathBuf>)`, the command last
  offered. `init` and a `SettingsStore` observer call `offer`: while the switch is on, look for
  `rusty-mcp` on the search path off the main thread (`which::which_in`); then, when that differs
  from what was offered, insert `context_servers.rusty` as `Stdio { enabled, remote: false, command:
  rusty-mcp with no arguments }` into Zed's defaults, or remove Marley's entry. The defaults' own
  update notifies the store again, which finds nothing changed.
- **`script/e2e.sh`:** after copying the user's settings, `marley.rusty_tools: false` in the copy.

### File manifest
- Marley: `crates/marley_workbench/src/rusty.rs` (new), `marley_workbench.rs`; `script/e2e.sh`;
  `script/e2e/633-rusty-tools-for-zeds-agents.sh`.
- Zed: `crates/settings_content/src/marley.rs`, `assets/settings/default.json`,
  `crates/settings_ui/src/marley_page.rs` (rows widened first).

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | the stand-in on `PATH`, the switch on; Settings, MCP Servers | `633-01-listed` |
| REQ-002 | the switch off | `633-02-off` |
| REQ-003 | `context_servers.rusty` of the user's own | `633-03-own` |

## Phase 2 — Code (2026-10-01)
- **Built:** `marley.rusty_tools` (`settings_content`, `default.json`, `MarleySettings` as
  `RustyTools`, the Agents section's Rusty Tools for Agents toggle); `rusty.rs` (`RustyOffer`,
  `offer` looking for `rusty-mcp` off the main thread, `settle` inserting or removing
  `context_servers.rusty` in Zed's defaults only on a change); `script/e2e.sh` writing
  `marley.rusty_tools: false` into each run's copy of the user's settings.
- **Review:** the defaults' update notifies the settings store, whose observer runs `offer` again
  and stops at the unchanged offer; a user's own `rusty` entry is in the user's layer and wins over
  the default; turning the switch off removes Marley's default only.
- **Clippy found:** the module doc's first paragraph, a value passed for a borrow, a variant
  without a doc; rustc: the context traits' imports.
- **Gate:** GREEN, 17 PASS.

## Phase 3 — Test (2026-10-01)
- **Scenario:** `script/e2e/633-rusty-tools-for-zeds-agents.sh` (sway): a stand-in `rusty-mcp`
  (python, stdio, answering `initialize` and listing the four brain-loop tools) first on the PATH
  exported in `setup`, which Marley inherits; `marley.rusty_tools` set back on in the run's copy.
- **First run:** the Settings window's search found the new toggle (its description names
  `rusty-mcp`) but not the MCP Servers page, which the search does not list. The scenario now
  writes a keymap into the run's profile binding Ctrl+Alt+Shift+M to `zed::OpenSettingsAt
  { path: "context_servers" }`. No code changed.
- **Shots (second run):**
  - `633-01-listed` (REQ-001): User / AI / General / MCP Servers: `marley` and `rusty`, each with a
    green dot, running.
  - `633-02-off` (REQ-002): the switch off: `marley` alone.
  - `633-03-own` (REQ-003): the switch on again and `context_servers.rusty` set to `/bin/false`:
    `rusty` listed with a grey dot, not running: the user's entry, not Marley's working default
    (zoomed against the first shot's green).
- **Not shot:** a real call through Rusty's tools (the stand-in only lists them; a call to the
  user's Rusty would write into their brain).

## Phase 4 — Complete (2026-10-01)
- **Documented:** `CHANGELOG.md`; the guide (Zed's agents and Marley's server); `marley_workbench.md`
  (Rusty's tools); the plan's C2 row; the ledger rows of `settings_content`, `default.json` and
  `marley_page.rs`.
- **Knowledge:** L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001,
  L-claude-633-the-mcp-servers-page-opens-by-a-keymap-in-the-runs-profile-001,
  AD-claude-633-rustys-server-is-offered-where-installed-as-a-default-001. No bug reached Test.
- **Brain:** the consultation closed with `brain decide`.
