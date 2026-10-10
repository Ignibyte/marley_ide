---
status: promoted
created: 2026-10-09
ticket: TICKET-726 to TICKET-733 (deliberate, for future use)
pipeline_spec: <unassigned>
---

# What Marley could take from plannotator

Chad, 2026-10-09: "take a look at https://github.com/backnotprop/plannotator and see what may be to
add to the list".

**What it is.** Plannotator (`backnotprop/plannotator`, Apache-2.0 or MIT) is a local browser page
that reviews a coding agent's plan before the agent writes code. Its maturity on 2026-10-09:
- 9.3k stars;
- 175 releases since 2025-12-28, at v0.28.9 that day;
- one main maintainer, pre-1.0, changing daily.

**How it reaches the agents.**
- **Claude Code:** a `PermissionRequest` hook on `ExitPlanMode` blocks Claude while you decide.
  Claude Code 2.1.287 and later can instead end the turn and receive the decision as a message;
  that version is also the shared plugin's floor (#709).
- **Codex:** a `Stop` hook reads its transcript.
- **Elsewhere:** slash-command skills and an inbox MCP server.

Research only: no code is taken.

## Already in Marley
- **Notes on a diff to the agent:** #522; Marley also has per-turn diffs (#509), which plannotator
  lacks.
- **Annotations on a web page or a running app:** the Browser tab's picks and annotations
  (#496 to #498).
- **Answering a live prompt:** Needs you (#508, #651, #689).

## Worth building (TICKET-726 to TICKET-733, under Deliberate)
1. **Plan review in a center tab** (#726, large, in slices). The `ExitPlanMode` hook sits in the
   shared plugin, so rustal-harness takes a request for it.
2. **Plan versions and a diff between them** (#727, small after #726).
3. **Question cards in plans** (#728, medium). The syntax is taught to agents through the shared
   skill.
4. **Notes on any Markdown file or the agent's last reply** (#729, medium), extending #522.
5. **Agent findings as diff notes** over MCP (#730, medium), pairing with #717's second opinion.
6. **Viewed marks in the project diff** (#731, small to medium; a touch in Zed's `git_ui`).
7. **A feedback template for review notes** (#732, small): by default, verify each note, give a
   verdict, don't widen the review.
8. **Guided review** (#733, later).

## Skipped
- **An inbox that wakes the agent:** the harness's owner inbox and Rusty's Decisions cover it.
- **HTML and live-app annotation:** the Browser tab does more.
- **Hosted sharing and team workspaces:** Marley is single-user and local.
- **Note-app export:** Rusty's brain holds notes.
- **Other hosts (a VS Code extension, iOS, a herdr TUI)**, and other version control (jj,
  GitButler, Perforce, Bitbucket).
- **Semantic diff and call flow:** #712's LSP tools serve agents better.
- **CLI gates, and an Ask AI sidebar:** the Agent Panel sits beside the diff.

Sources: https://github.com/backnotprop/plannotator (README, releases v0.28.9, the docs under
`apps/marketing/src/content/docs`, `docs/custom-reviews.md`), https://plannotator.ai/blog/the-age-of-the-inbox/.
