---
pipeline_id: 17ffe2ba-9c2d-427d-9fcb-fd838a978c5e
ticket: docs/planning/tickets/open/TICKET-500-browser-from-the-rail.md
status: Phase 4 — Complete PASS
title: "A Browser tab from the rail's +, and Claude Code told it can drive the browser"
type: feature
slice: prong 3 (after wave 2), from Chad's first look at the browser
references: [docs/planning/pipeline/completed/493-browser-tabs-and-restore.spec.md, docs/planning/pipeline/completed/482-claude-code-notifications-chip.spec.md, docs/planning/pipeline/completed/492-browser-tools-for-agents.spec.md]
---

## Title
The + of a project in the rail opens a Browser tab there, and the chip that installs Marley's
plugin for Claude Code says it brings Marley's tools, the browser among them, so the user knows
an agent can pull up the browser and drive it.

## Scope
### In
- **New Browser Tab** in a project's + menu in the rail, after New Terminal: the project comes to
  the front, a blank page opens in a new Browser tab of its active pane, and the focus goes to the
  tab's address bar, as Ctrl+T does in a Browser tab. Chromium starts first when it must.
- **The plugin chip** in the agent bar under a Claude Code terminal reads "Connect Claude Code to
  Marley" in place of "Enable Claude Code notifications", with a tooltip that names what the
  plugin brings: notifications, and Marley's tools for its terminals and Browser tabs. The toast
  after the install says the same.

### Out (explicitly deferred)
- Browser tabs as rows of their project in the rail.
- A key for opening the browser, and installing the plugin without the chip.

## Reference (§20)
Upstream Zed's new-item menus (the pane's + menu, `workspace::NewFile` and friends): a + that
lists what can be opened in place. Marley's rail menu (#450) already follows it with New
Terminal and the agents; the Browser tab joins the list. The chip's first words came from Warp's
"Enable Claude Code notifications" (#482, `docs/warp_architecture/`), but since #491 the plugin
also carries Marley's MCP server, so the chip names that; the behavior (one click installs the
plugin) is unchanged.

### Prior art
- **Code we already ship.** The rail's project menu (`rail.rs`, `render_project_menu`, with
  `new_terminal` and `activate_workspace`); `browser::new_tab`, which Ctrl+T runs; the agent
  bar's `claude_plugin_chip` and `claude_plugin::install`'s toast; the browser tools (#492,
  #493), whose `browser_navigate` opens a Browser tab when none is open and starts Chromium.
- **Published material.** None needed: no protocol changes.
- **Behavior maps.** Zed's pane + menu; nothing else owns this seam.

## UI proof
UI-AFFECTING. `script/e2e/500-browser-from-the-rail.sh` (`compositor sway`, offline): the
scratch repository open in the Marley layout, a stand-in `claude` first on the terminal's PATH.
Steps: the project's + and its menu (`500-01-menu`: New Browser Tab listed after New Terminal);
New Browser Tab, then an address typed straight away (`500-02-opened`: the page in a Browser tab
of the project); the terminal with the stand-in Claude Code and the pointer on the chip
(`500-03-chip`: "Connect Claude Code to Marley" and its tooltip). The chip is never clicked: it
would install the plugin into the real Claude Code configuration.

## Locked-In Decisions
- D1 — The entry opens a new tab (`browser::new_tab`), not `marley: open browser`'s "show the
  browser": the + means "new in this project".
- D2 — The chip keeps its one-click install; only its words and tooltip change.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user opens a project's + menu in the rail, the menu shall list New Browser Tab. | Shot `500-01-menu` |
| REQ-002 | WHEN the user chooses New Browser Tab, Marley shall open a blank page in a new Browser tab of that project with the focus in its address bar. | Shot `500-02-opened` |
| REQ-003 | WHILE Claude Code runs in a terminal and Marley's plugin is not installed, the agent bar shall offer "Connect Claude Code to Marley", naming the browser among the tools it brings. | Shot `500-03-chip` |

## Phase Plan
- **P1 Plan** — mint, recall, the design.
- **P2 Code** — the menu entry, the chip's words and the toast; fmt and clippy clean.
- **P3 Test** — the scenario, every shot; #482's chip scenario is replaced by this one's check;
  #493's scenario again; the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
