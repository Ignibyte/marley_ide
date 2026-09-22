---
pipeline_id: 13dada8b-00df-4bc5-a92f-a5bdfdd64298
ticket: forge#182 (0dd1b3d4-f985-4085-b4e7-0adf59e190ba) · local docs/planning/tickets/open/TICKET-182-agent-followups.md
aar_id: 083fab37-5311-4eaa-a577-16b49a4e1938
status: Phase 5 — Complete PASS
title: M12 — #174 follow-ups: project-scoped agent diff + dead-agent send flash
type: bug
milestone: M12 — The Agent Cockpit
references:
  - crates/marley_app/src/tabs.rs (PURE: Workspace::pane_project_root over locate_pane)
  - crates/marley_app/src/app.rs (SHIM: git_working_diff_in + the ⌘-click root + the send else-flash)
---

## Title
Close two documented #174 gaps: the ⌘-click "agent diff" should show the AGENT's project's diff (not the
active project's), and ⌘⇧S to a resolved-but-DEAD agent should say so instead of failing silently.

## Scope
### In
- PURE `tabs.rs`: `Workspace::pane_project_root(pane) -> Option<&Path>` = `locate_pane(pane) → projects[p].root`.
- SHIM `app.rs`: `git_working_diff_in(root)` (extracted from git_working_diff, which becomes
  `git_working_diff_in(&self.project_root)`); the Agents-cockpit ⌘-click resolves the agent's root via
  `pane_project_root` (fallback: the active root) and diffs THERE + flashes the project name; the
  send-to-agent arm gains an `else` flashing "{label}'s agent has exited" when the write fails (the compose
  line stays, #72).

### Out
- The git-panel file-row diff (correctly active-project) + ⌘⇧D (active-project); the Fleet overlay row (no
  ⌘-click diff branch).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `pane_project_root` shall return the owning project's root for a live pane and `None` for an unknown one. | unit |
| REQ-002 (visual) | ⌘-clicking a cross-project agent's row shall show THAT project's working diff, not the active project's. | driven capture |
| REQ-003 (visual) | ⌘⇧S to a dead agent shall flash "…exited" and KEEP the composed line. | driven capture |
| REQ-004 | gate GREEN; the pure fn cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 the pure fn + the 3 shim edits. P3.5 self-review (the root fallback; the label lookup; no
regression to the git-panel/⌘⇧D diffs). P4 unit + driven + gate. P5 docs.
