---
pipeline_id: f89bd2fd-e6ce-4b0a-b08a-f31eba277fd7
ticket: forge#235 (ca7bdb07-de31-484f-8c70-e97881c0a061) · local docs/planning/tickets/open/TICKET-235-scope-driven-topbar.md
aar_id: 5390a623-f206-495b-a62c-16ab0776a173
status: Phase 5 — Complete PASS
title: Scope-driven top bar — a focused-workspace indicator + the workspace-scoped actions grouped left
type: feature
milestone: M13
references: [forge#233, forge#236, forge#126, forge#142]
---

## Title
chad feedback #6 + #7. Today the top-bar actions split left (files / new-terminal / new-agent) + right
(the cockpit tabs Details/Agents/Forge). #235 regroups them by SCOPE: it adds a **focused-workspace
indicator** (the active project's name · branch) to the top bar so it's clear the buttons act on the
FOCUSED workspace (#7 — "indicate which workspace you're focused on"), and moves the cockpit tabs from
the top-right into the LEFT cluster next to the other workspace-scoped actions + the indicator (#6 —
"combine the tabs on the left"). The dispatch already targets the active project (= the focused
workspace, #233), so switching focus (⌘⇧] / the rail #236) re-targets the buttons + updates the
indicator with no dispatch change.

## Scope
### In
- **Pure (cov/MSI 100):** `top_bar::focused_workspace_indicator(name, branch) -> String` — the indicator
  label (the workspace `name`, then ` · {branch}` when a branch is present), truncated to a max width
  with an ellipsis (mirrors `titlebar_label`'s composition + the rail's "name · branch").
- **Shim (app.rs top bar):** render the indicator (absolute, after the 3 left icons, max-width + the
  active project's name + branch); RELOCATE the cockpit-tabs cluster from `.right(px(16))` to a LEFT x
  after the indicator (so files/+/sparkle · indicator · Details/Agents/Forge form one left group). The
  active-project name + `branch_from_git_head(project_root/.git/HEAD)` feed it (both already computed
  for the footer/titlebar).

### Out (deferred)
- A top-bar workspace SWITCHER (a dropdown/click-to-switch on the indicator) — the rail (#236) is the
  switcher for now; the indicator is read-only this ticket.
- Converting the whole top bar to a flex layout — the absolute positioning stays; the indicator is
  fixed-width (truncated) so it doesn't reflow the cockpit tabs after it.
- The rail highlight (#236) — same focused-workspace source of truth, its own ticket.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — the focused workspace = the active project** (#233); the indicator reads `active_project().name`
  + the branch, and the buttons already dispatch to it. No new focus source.
- **D2 — absolute layout kept; the indicator is truncated to a fixed max width** so the relocated
  cockpit tabs sit at a fixed x after it (no flex refactor, no variable-width reflow).
- **D3 — read-only indicator** (a label/chip, not a switcher) — switching is ⌘⇧] / the rail (#236).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The pure `focused_workspace_indicator(name, branch)` shall compose `name` (+ ` · {branch}` when present), truncated to the max width with an ellipsis. | `top_bar` unit tests (with/without branch, truncation), cov/MSI 100 |
| REQ-002 | The top bar shall render the indicator showing the FOCUSED workspace's name · branch, next to the workspace-scoped action icons. | shim review + driven capture |
| REQ-003 | The cockpit tabs (Details/Agents/Forge) shall render in the LEFT cluster (with the actions + indicator), not the top-right. | shim review + driven capture |
| REQ-004 | WHEN the focused workspace changes (⌘⇧]), the indicator shall update to the new workspace's name. | driven capture (2 workspaces, switch focus → indicator changes) |

## Phase Plan
- **P2 Design** — the pure `focused_workspace_indicator` + truncation + tests; the indicator render
  (position, max-width, style) + the cockpit-tabs relocation x; run `cargo mutants --list`.
- **P3 Implement** — pure (top_bar.rs) then the app.rs top-bar shim.
- **P3.5 Inspect** — critics: the layout (no overlap with search/icons), the indicator correctness, the
  relocation, clean-room.
- **P4 Validate** — RUN the pure tests; gate green; DRIVEN capture (2 workspaces, switch focus → the
  indicator + the left-grouped layout).
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; close #235.
