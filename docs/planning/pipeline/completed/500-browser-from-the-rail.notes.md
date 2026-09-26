# A Browser tab from the rail's +, and Claude Code told it can drive the browser — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-500-browser-from-the-rail.md
- **Pipeline spec:** 500-browser-from-the-rail.spec.md

## Phase 1 — Plan (2026-09-25)
- **Request:** Chad, on his first look at the browser: "on the left projects pane we should add
  a + and be able to select browser" and "if hasnt already done so an agent should be able to
  pull up the browser and drive".
- **Classification:** feature, small; `marley_workbench` only (`rail.rs`, `browser.rs`,
  `agent_bar.rs`, `claude_plugin.rs`). No Zed path.
- **Recall (§18.3):**
  - The brain (consultation `5b77f40dd520405d8f8640386ad75705`): nothing on this seam.
  - Each project row already has a + ("New in this project": New Terminal, New Agent Thread,
    the agent CLIs, #450); Chad saw none because no project was open.
  - Agents already pull up and drive the browser: `browser_navigate` starts Chromium and opens a
    tab when none is open (#492, #493 D5). Claude Code gets the tools only through Marley's
    plugin, not installed in Chad's Claude Code (`installed_plugins.json` lists no
    `marley@marley`), and the chip that installs it says "Enable Claude Code notifications",
    Warp's words from before the plugin carried Marley's MCP server (#491).
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, pre-flight ✓, recall ✓, prior
  art ✓, spec ✓, design ✓.

### Design
- **The menu:** `render_project_menu` gains `.entry("New Browser Tab", …)` after New Terminal,
  calling a new `Rail::new_browser_tab(workspace)`: `activate_workspace`, then
  `browser::new_tab` in that workspace (made `pub(crate)`).
- **The chip:** `claude_plugin_chip`'s button reads "Connect Claude Code to Marley", with
  `Tooltip::text` naming notifications and Marley's tools for its terminals and Browser tabs.
  `claude_plugin::install`'s toast names the same.
- **Manifest:** `crates/marley_workbench/src/rail.rs`, `browser.rs`, `agent_bar.rs`,
  `claude_plugin.rs`; `script/e2e/500-browser-from-the-rail.sh`. No Zed path.

### E2E plan
| REQ | Step (`compositor sway`, offline) | Shot |
|---|---|---|
| REQ-001 | the scratch repository in the Marley layout; the project's + | `500-01-menu` |
| REQ-002 | New Browser Tab; a loopback address typed at once, Enter | `500-02-opened` |
| REQ-003 | the terminal's stand-in `claude` (a script named `claude` first on its PATH); the pointer on the chip | `500-03-chip` |
| (regression) | #493's scenario (the Browser tabs) | its shots |

The chip is never clicked: the install writes into the Claude Code configuration Marley reads,
which in a scenario is the real one.

### Risks
- The rail's menu tests (`rail_tests.rs`) count entries by their text; they build but never run
  (§7).

## Phase 1 — closeout
Phase 1 PASS (autonomous).

## Phase 2 — Code (2026-09-25)
- **Built:** `rail.rs`: New Browser Tab after New Terminal in a project's + menu, and
  `Rail::new_browser_tab` (`activate_workspace`, then `browser::new_tab` in that workspace, now
  `pub(crate)`); `agent_bar.rs`: the chip reads "Connect Claude Code to Marley", with a tooltip
  naming notifications and Marley's tools for its terminals and Browser tabs;
  `claude_plugin.rs`: the install's toast names them too.
- **Deviations from the design:** none.
- **Review:** REQ-001 and REQ-002: the entry runs inside the rail's update, as New Terminal
  does; `new_tab` creates the hub on first use (`BrowserHub::global` starts it), so a first New
  Browser Tab starts Chromium. REQ-003: only the words and the tooltip change; the chip's id and
  its install stay.
- **Checks:** `cargo clippy -p marley_workbench --all-targets -- -D warnings` clean; `cargo fmt`
  clean.

## Phase 2 — closeout
Phase 2 PASS (autonomous).

## Phase 3 — Test (2026-09-25)
- **Scenario:** `script/e2e/500-browser-from-the-rail.sh` (`compositor sway`, offline Chromium):
  the scratch repository in the Marley layout, its terminal's shell renamed `claude` (a
  `.bashrc` of the scenario's own), a loopback page. Steps: a click on the project's +, one step
  down, a shot; Enter, then the page's address typed at once and Enter; a click on the
  terminal's tab and the pointer on the chip.
- **Shots (in the scratchpad, `e2e-500/`), each read:**
  - `500-01-menu` (REQ-001): the menu lists New Terminal, **New Browser Tab** (selected), New
    Agent Thread ›, then the Agent CLIs.
  - `500-02-opened` (REQ-002): a Browser tab "From the rail" beside the terminal's tab in the
    project's pane, showing the page typed into its address bar straight after the tab opened.
  - `500-03-chip` (REQ-003): "Connect Claude Code to Marley" in the agent bar under the stand-in
    Claude Code, its tooltip "Installs Marley's plugin for Claude Code: notifications, and
    Marley's tools for its terminals and Browser tabs, so the agent can open a page and drive it
    while you watch". The chip was not clicked.
- **What the first run found, in the scenario:** the menu opens with its first entry selected,
  so two steps down chose New Agent Thread; one step chooses New Browser Tab. The pointer was
  below the chip; it now sits on it.
- **Regressions:** `493-browser-tabs.sh`: its tabs, keys and log as before.
- **Focus report:** every run in the headless sway; "hyprland: 0 Marley windows before the run,
  0 after; the run added no rule and did not reload it".
- **Gate:** `just gate-diff` — 16 passed, 0 failed, `GATE GREEN [diff]`; the receipt matches the
  tree.
- **Checklist (no TaskCreate in this harness):** REQ-001 ✓, REQ-002 ✓, REQ-003 ✓, 493 ✓, gate ✓.

## Phase 3 — closeout
Phase 3 PASS (autonomous).

## Phase 4 — Complete (2026-09-25)
- **Documented:** `CHANGELOG.md` (#500 under Added); `docs/marley_architecture/marley_workbench.md`
  (the project's + menu, the agent bar's chip). No path outside the Marley-owned set changed.
- **Knowledge appended:** `L-claude-500-a-clicked-context-menu-starts-on-its-first-entry-001`.
- **Brain:** consultation `5b77f40dd520405d8f8640386ad75705` closed with
  `decisions/the-rails-opens-a-browser-tab-and-the-plugin-chip-names-the-browser`.
- **Ticket:** closed. Chad's third ask, Zed's own agents driving the browser, is #501.
