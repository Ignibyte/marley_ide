---
status: promoted
created: 2026-06-28
ticket: TICKET-491, TICKET-525, TICKET-534, TICKET-556, TICKET-689
pipeline_spec: unassigned
note: audit (2026-10-09): shipped in the fork's shape; marley_mcp carries the protocol, so no local_control crate was built
---

# Brain ↔ agent session supervision — launch, control, observe (future)

## What
The **brain** (the orchestrator / RLM, served over MCP) can **launch sessions it understands and
controls**, and — the load-bearing part — **see into the terminal of the agents working**. Three
capabilities over one session-supervision seam:

1. **Launch** — the brain owns the session lifecycle: spin up an agent's terminal session (in a
   project-scoped folder or git worktree, the agent-orchestration isolation modes), not just hand a
   task to a black box.
2. **Control** — drive that session: inject input/commands, activate/close, set cwd — programmatic
   control distinguished from human typing (the `EditOrigin::SystemEdit` vs `UserTyped` seam).
3. **Observe** — **read the agent's terminal output back** (the per-command Block stream: command +
   output + exit status + cwd). The brain watches what each delegated agent is doing, live, so it can
   supervise, intervene, summarize, or gate the next step. This is the observability that makes agent
   delegation trustworthy.

It is local-first (M3); the **remote** variant (observe/control an agent running on a remote runner)
composes this with the [[remote-connection-seam]] intake (M5).

Shape (clean-room, from the behavior reference, not Warp source):
- Built on **`local_control`** (REIMPLEMENT, M) — Warp's out-of-process control protocol (window/tab/
  pane/session create+activate+close, `input.insert`/`replace`, `TargetSelectors`). **The key Marley
  delta: add a `session.read` action — output read-back, which Warp's local_control has NONE of.**
  That single delta is what turns "drive a session" into "supervise a session."
- The observed unit is the **terminal Block model** (`terminal-blocks`, M1): each Block already carries
  command / output / status / cwd / SessionId (DCS-hook metadata) — exactly the agent-activity record
  the brain consumes.
- Runs over a **permissive transport** (rmcp / jsonrpc, REUSE) so the same surface **doubles as an
  agent tool**: the brain reaches it as an MCP tool (`launch_session`, `write`, `read`, `close`),
  which is also how a human or another agent drives it. One protocol, three callers.
- `SessionId` (`marley_core`, M0) + `HostId`/local-vs-remote path (`marley_util`, M0) already make the
  model session- and host-addressable; this seam adds the **control + read-back protocol**, not new
  identity types.

## Why
- Chad: "I do want the concept of the brain to be able to launch sessions it understands and controls
  … the brain should see into the terminal of agents working if possible." (2026-06-28.)
- **It is possible.** The architecture already supports it: `local_control` gives launch+control; the
  Block model gives a structured output stream; the one missing primitive (output read-back) is a
  known, small delta the triage already flags. No cloud/server dependency — fully local + self-hostable.
- Without explicit read-back the orchestrator is "fire-and-forget" — it can start agents but not watch
  them. Observability is the difference between delegation and supervision, and it's the natural seam
  for an agentic-workflow visualization panel (Hook 1).

## Notes
- **Not** Warp's session sharing / Agent Mode cloud client (`warp_multi_agent_client` → SKIP). This is
  the LOCAL control+observe seam; the agent's *reasoning* runs via INVENT direct-provider streaming.
- Reference behavior: `docs/warp_architecture/crates/local_control.md` (the action catalog +
  `TargetSelectors`) and `subsystems/03-terminal-session-core.md` §7 + `00-overview.md` §7.2 (Hook 2:
  "local_control has no byte-level read-back; Marley would add a `session.read` ActionKind").
- Composes with: **agent orchestration** (INVENT M3 — the brain delegating to project-scoped agents,
  worktree / folder-per-agent isolation), the **terminal-blocks** crate (M1, what's observed), and
  **[[remote-connection-seam]]** (M5 — supervise a remote agent the same way).
- Likely home: the `local_control` REIMPLEMENT crate (M3) carries the protocol; the brain side is INVENT
  (the orchestrator consuming it as an MCP tool).

## Promotion
Candidate, not a pipeline doc yet. Promote via `/work` when agent orchestration is scheduled (M3). At
that point `local_control` is built WITH the `session.read` read-back delta from day one (don't bolt it
on later), and the brain-orchestrator INVENT layer consumes it. Decision recorded as forge
`AD-claude-brain-agent-session-supervision-001`.
