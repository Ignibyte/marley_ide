---
pipeline_id: ee632c52-1fc1-4c58-949f-39d0a576170f
ticket: docs/planning/tickets/open/TICKET-885-more-system-pages.md
status: Phase 4 — Complete PASS
title: The System tab's other pages
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The System tab gains the SystemPages board's seven other pages, each a card of rows with a dot
(ok, warning, wrong, or quiet): Install and update, Troubleshooting (checks with what each found),
Shell integration, Marley's MCP endpoint, Marley's Chromium, Fleet providers, and For developers.

## Scope
### In
- `system_pages.rs`: a row type with its mark; the seven pages; Troubleshooting's checks run off
  the main thread when the tab opens and on Run the checks again; Copy a report.
- `clients.rs`: `pub(crate)` readers for the server's address and the paired clients' names.
- `browser.rs`: a `pub(crate)` reader listing each project's browser with its state.

### Out (explicitly deferred)
- Restart Marley and Clear terminal history: an action that ends or deletes wants its own ticket
  and its confirmation.
- Rusty's version: kept private in its state; the page says connected or why not.

## Reference (§20)
N/A — Marley-specific: Marley's own facts; the SystemPages board.

### Prior art
- #855's `system_pages.rs`; `clients.rs` (`ClientsState`, `current_registry`), `browser.rs`
  (`BrowserHub`, `HubState`, `tabs`, `project_of`), `fleet.rs` (`Fleet`, `Source`,
  `SourceState`), `fleet_providers::name_of`, `rusty::{is_on, is_connected, unavailable}`,
  `agent_versions::found`, `claude_plugin::ClaudePlugin`, `agents::installed_clis`,
  `marley_terminal::shell_integration` (bash, zsh, fish).
- The brain (consultation c292fe89).

## UI proof
Scenario `script/e2e/885-more-system-pages.sh` (sway): a scratch repository; the run's Marley
with its MCP server, no Rusty, no fleet provider, stand-in `claude` and `codex`.
- `885-01-trouble`: Troubleshooting's checks with their marks and values (REQ-001, REQ-002).
- `885-02-mcp`: Marley's MCP endpoint: running, its address, its file, the paired clients (REQ-001).
- `885-03a-install` to `885-03e-developers`: Install and update, Shell integration, Marley's
  Chromium, Fleet providers and For developers, in turn (REQ-001).
- `885-04-day`: Marley Day, Troubleshooting (REQ-001).

## Locked-In Decisions
- D1 — A check is ok when found and working, a warning when it works with a gap (an update ready,
  a plugin not installed), wrong when missing, quiet when off by choice.
- D2 — Checks never write or start anything; Run the checks again reads again.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The System tab shall list Install and update, Troubleshooting, Shell integration, Marley's MCP endpoint, Marley's Chromium, Fleet providers and For developers, each drawn as the board's card of marked rows, in Marley Night and Marley Day. | `885-01-trouble`, `885-02-mcp`, `885-03a` to `885-03e`, `885-04-day` |
| REQ-002 | Troubleshooting shall check shell integration, Rusty, Claude Code, Codex, the Claude Code plugin, the other agent CLIs and git, and say what each found. | `885-01-trouble` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
