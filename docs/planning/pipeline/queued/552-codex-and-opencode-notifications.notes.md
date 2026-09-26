# Notification setup for Codex and OpenCode from the agent bar — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-552-codex-and-opencode-notifications.md
- **Pipeline spec:** 552-codex-and-opencode-notifications.spec.md

## Phase 1 — Plan
- **Request:** the Warp second pass (2026-09-25), "Smaller": Warp's agent bar installs each
  CLI's notifications in one click (a Codex plugin, an OpenCode plugin); "Marley's agent bar
  offers only 'Connect Claude Code to Marley'. If Codex's own notifications arrive as OSC 9,
  Marley shows them already and a chip only has to set the config. S, after checking what Codex
  prints in a Marley terminal." Chad, 2026-09-26: every remaining finding gets built.
- **Classification / tier:** feature, S. Two modules in `marley_workbench`, two chips, one
  JavaScript file, one dependency. No Zed path.
- **Recall (§18.3):**
  - L-claude-482-claude-codes-print-mode-drops-a-hooks-terminal-sequence-001: prove a terminal
    sequence with an interactive session in a pty; the same method for Codex and OpenCode.
  - L-claude-477-a-quiet-foreground-process-is-seen-only-after-output-001: the stand-ins print
    a line so the bar sees them.
  - L-claude-478 (the notification service watched with `busctl --user monitor`, recorded in
    #519's scenario): the scenario reads the record for one `Notify` and then none.
  - The 482 spec's D1 and D3 (a plugin, silent outside Marley) and the 547 spec's D1 (semver
    compare, no chip for a newer or unparsable install).
  - Brain: no page on Codex or OpenCode notifications (searched 2026-09-26).
- **Discovery:**
  - `crates/marley_workbench/src/claude_plugin.rs`: `FILES` (35), `ClaudePlugin` (69),
    `needs_update` (94), `shipped_version` (109), `init` (120), `set_up` (139),
    `write_plugin_in` (170), `installed_in` (186), `installed_version_in` (195), `install`
    (229), `update` (312). `agent_bar.rs`: `contents` (93), `agent_in` (129), `render` (149),
    the chip gate on `AgentKind::Claude` (198 to 202), `claude_plugin_chip` (305 to 361, the
    busy labels, the two buttons, the debug selector). `marley_agent.rs`: `AgentKind` (32),
    `agent_kind_of` (76), `display_name` (60). `notifications.rs`: `init` (30), `notify` (59),
    `show_sender` (86). `marley_dcs/src/notification.rs`: the scanner (38), the OSC 9 and 777
    arms (96, 107). The plugin's `notify.sh` (the `TERM_PROGRAM` check at line 5, the OSC 777
    at 14) and `hooks.json`.
  - On the box: `codex` 0.155.1 (`~/.local/share/mise/installs/codex/latest/bin/codex`),
    `~/.codex/config.toml` with `[projects.*]` tables and `[tui.model_availability_nux]`, no
    notification key; `opencode` 1.18.31, `~/.config/opencode/opencode.json` with `$schema`
    and `autoupdate` only, no `plugin` array, no `plugins/` folder. Chad's own files are never
    written by a scenario (D5).
  - `Cargo.toml:888` (`toml_edit = { version = "0.22", default-features = false, … }`),
    `Cargo.lock:19049` (0.22.27). `util::paths::home_dir`.
  - Warp's docs, Codex's reference and OpenCode's plugin docs as the spec cites them; the
    binaries' strings as corroboration.
- **Decisions:** D1 to D6 in the spec.

### Design
- **`marley_workbench::codex_config`** (new): `config_dir()` = `CODEX_HOME` or
  `~/.codex`; `NotificationSetup { on: bool }`; `read_in(dir) -> NotificationSetup` (parses with
  `toml_edit::DocumentMut`; `on` when `tui.notifications` is `true` or a non-empty array, and
  `notification_condition == "always"`, and `notification_method == "osc9"`); `write_in(dir) ->
  io::Result<()>` (parses the existing text or starts empty, sets the three keys under `tui` as
  an implicit table when absent, writes the document back; `notifications` left alone when it
  is an array). Both run off the main thread (a lazy future).
- **`marley_workbench::opencode_plugin`** (new): `PLUGIN: &str = include_str!(
  "../agent_plugins/opencode/marley.js")`, `VERSION` parsed from its first line; `config_dir()`
  = `XDG_CONFIG_HOME` or `~/.config`, joined with `opencode`; `installed_version_in(dir) ->
  Option<Version>` from `plugins/marley.js`'s first line; `write_in(dir)` (creates `plugins/`,
  writes the file, mode 0644). The file:
  ```js
  // marley-opencode-plugin 1.0.0
  // Marley's plugin for OpenCode: a terminal notification on a turn's end, a permission
  // request and an error, shown by Marley while you are not looking at that terminal.
  // It says nothing in any other terminal.
  export const MarleyNotifications = async ({ directory }) => ({
    event: async ({ event }) => {
      try {
        if (process.env.TERM_PROGRAM !== "zed") return;
        const message = { "session.idle": "finished", "permission.asked": "needs your permission",
          "session.error": "hit an error" }[event.type];
        if (!message) return;
        const project = (directory || process.cwd()).split("/").pop().replace(/[\x00-\x1f;"]/g, "");
        process.stdout.write(`\x1b]777;notify;OpenCode;${project} ${message}\x07`);
      } catch (_) {}
    },
  });
  ```
- **The chips** (`agent_bar.rs`): a global `AgentSetups { codex: Option<bool>, opencode:
  Option<Option<Version>>, busy: Option<&'static str> }` refreshed off the main thread when a
  bar first renders for that agent and after each write; `codex_chip` and `opencode_chip` after
  `claude_plugin_chip`'s pattern (labels: "Turn on Codex notifications", "Connect OpenCode to
  Marley", "Update Marley's plugin for OpenCode"; tooltips naming what is written and where;
  toasts: "Codex will notify Marley: a turn's end, a permission, an error. Restart a running
  Codex to pick it up." and the OpenCode one), debug selectors `marley-codex-chip` and
  `marley-opencode-chip`.
- **File manifest.** Marley crates: `crates/marley_workbench/agent_plugins/opencode/marley.js`
  (new), `crates/marley_workbench/src/{codex_config.rs (new), opencode_plugin.rs (new),
  agent_bar.rs, marley_workbench.rs}`, `crates/marley_workbench/Cargo.toml`. Scripts:
  `script/e2e/552-codex-and-opencode-notifications.sh`.
- **Ledger rows.** None: no path outside the Marley-owned set. (`Cargo.lock` changes only if
  `toml_edit`'s feature set changes, and its row says "regenerate".)

### E2E plan
`setup` exports `CODEX_HOME=$E2E_WORK/codex` and `XDG_CONFIG_HOME=$E2E_WORK/xdg` for Marley's
process, seeds `codex/config.toml` (`# my codex\nmodel = "o4-mini"\n\n[tui]\ntheme = "dark"\n`),
writes stand-in `codex` and `opencode` (print "ready", read stdin; `opencode` with `--run-plugin
<event>` runs `node` on the plugin file with a fake event on stdin, as the scenario's own tester),
and the `busctl` monitor as #519's scenario starts it.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | type `codex`, Return | `552-01-codex-chip` |
| REQ-002 | click the chip; settle | `552-02-codex-written`; the log: `cat codex/config.toml` with the comment, `model`, `theme` and the three keys |
| REQ-003 | Ctrl-D; type `opencode`, Return | `552-03-opencode-chip` |
| REQ-004 | click the chip; settle | `552-04-opencode-written`; the log: `head -1 xdg/opencode/plugins/marley.js` |
| REQ-005 | the scenario rewrites the first line to `0.0.1`; Ctrl-D; `opencode` | `552-05-opencode-update` |
| REQ-006 | Ctrl-D; `opencode --run-plugin session.idle` with `TERM_PROGRAM=zed`, then with it unset; the terminal unfocused (a second terminal focused) | `552-06-notified`; the log: one `Notify` in the `busctl` record, the captured stdout with and without the OSC |
| REQ-007 | the same shot: the desktop notification's record names OpenCode | the log |

The Codex OSC 9 itself: Test's live pty run of the real `codex` with the keys set records what
it prints at start; a turn's end needs a model and is taken as far as the box allows (recorded,
never skipped silently).

### Risks
- OpenCode's TUI and server may not share a process in every launch; D4 names the check that
  settles it before code. If the plugin's stdout never reaches the terminal, the OpenCode half
  splits into its own ticket (a plugin that POSTs to Marley's MCP server instead, which #520's
  caller would need to place), and Codex's chip ships alone.
- Codex's `auto` method may already pick OSC 9 for `TERM_PROGRAM=zed`; forcing `osc9` is then
  redundant and harmless. Its OSC 9 text is Codex's, not Marley's, until #538.
- A user's `config.toml` with a `[tui]` table written as dotted keys or inline: `toml_edit` keeps
  the form it finds; the write uses `get_or_insert` on the table, whichever shape it has.
- The plugin folder's spelling: the docs say `plugins/`, the binary knows both; Marley writes
  `plugins/` and checks nothing in `plugin/`.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
