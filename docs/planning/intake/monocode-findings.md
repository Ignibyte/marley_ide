---
status: promoted
created: 2026-10-09
ticket: TICKET-714 to TICKET-721 (deliberate, for future use)
pipeline_spec: <unassigned>
---

# What Marley could take from MonoCode (usemono.dev)

Chad, 2026-10-09: "research monodev and see if any features there we may use here", then "sorry
usemono.dev". usemono.dev is **MonoCode** (github.com/hardbeat920/monocode, MIT):
- a Tauri desktop app that wraps the coding-agent CLIs a person already has (Claude Code, Codex,
  Cursor, OpenCode, Pi, Hermes and others), each tab a session;
- created 2026-08-20, about a release a day (v0.11.0 on 2026-10-09).

The comparison is from its README, `CHANGELOG.md`, `docs/remote-access.md` and `docs/jira.md`,
against Marley's guide and changelog. Nothing of its code is carried over: it is research.

## Already in Marley
- **Persistent agents with memory and skills:** Rusty (#696, #700, #664, #665).
- **Notes:** Rusty's brain, capture and import (#663).
- **Working agents:** Home's Agents at Work (#701), the rail's states and Needs you.
- **A worktree per session:** #510.
- **Diff comments to the agent:** #522.
- **Notices when a turn ends or an agent waits:** #478, #709.
- **Agents reading and messaging sessions:** the thread tools (#706) and `seat_add` (#692).
- **Remote agents over SSH:** #543, #641, #610.

## Partly in Marley
- **Turn checkpoints:** #509 records each turn's diff under `refs/marley/turns/`; MonoCode adds
  Keep and Undo per turn.
- **Usage meter:** #640 shows quota for harness seats; MonoCode shows Claude's and Codex's
  five-hour and weekly use, and pauses queued work at a limit, resuming at the reset.
- **Delegation:** the harness's manager and foreman; MonoCode's operator also starts sessions in a
  split or worktree, leaves a drafted message for review, and reports back when delegated work
  ends.
- **GitHub:** the rail's PR chip (#531); MonoCode has an inbox of issues and PRs.

## Not in Marley
- **Second opinion:** a finished turn sent to another agent in a split.
- **Handoff:** switching provider mid-session, carrying a recap.
- **A plan card:** review a plan, then build it with another model.
- **An inbox:** GitHub, GitLab, Linear and Jira items; "start work"; failed checks sent to an agent.
- **Scheduled runs ("habits"):** recurring prompts in a background session, reporting only when
  they find something.
- **A quick composer:** a global key opens a prompt that starts an agent in the background.
- **An MCP manager:** each agent CLI's MCP servers, edited in one place.

## Worth building, best first
1. **Undo a turn** (small to medium). Undo and Keep on each turn row of #509: a reverse apply of
   the turn's commit that refuses on conflict.
2. **Usage meter and resume at reset** (medium). #640's quota for local Claude Code and Codex
   terminals and Agent Panel threads; hold at a limit, resume at the reset.
3. **Delegation tools on Marley's MCP server** (medium):
   - start a thread or a terminal agent, in a worktree or a split beside the caller;
   - draft a message for the user to review;
   - report back to the caller when the work ends.
   They go through #704's agent-control modes and #703's log.
4. **Second opinion and handoff** (medium). "Ask another agent" on a turn, a turn's diff or a
   worktree branch, opened in a split with a recap; #696's Auto choice picks the provider.
5. **Failed checks to an agent** (medium), then **a GitHub inbox** (large). #531 already polls
   `gh`: show the checks and "Send failures to agent"; the inbox (issue, start work, worktree
   agent) after.
6. **Scheduled runs** (medium to large). Recurring prompts for Rusty or the Marley agent,
   reported in Needs you. Per Chad's 2026-10-02 call that agent hosting is the harness's, the
   scheduler may belong in rustal-harness, with Marley showing the runs.
7. **An MCP manager and agent CLI update checks** (small to medium). A Settings page for each
   agent CLI's MCP servers, and a warning when Claude Code is older than the shared plugin needs
   (#709).
8. **A quick prompt** (small). A `marley` verb Hyprland binds to a key, opening a prompt that
   starts an agent in a chosen project in the background.

Skipped: `/btw` and the follow-up queue (Claude Code has them), mascots and effects, transcript
search (Zed's panel covers it).
