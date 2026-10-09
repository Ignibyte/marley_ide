---
status: superseded
created: 2026-07-15
ticket: unassigned
pipeline_spec: unassigned
note: audit (2026-10-09): rustal-harness is the session layer (Prong 2, D19; Chad 2026-10-02); Marley's viewer half shipped as TICKET-534, TICKET-540, TICKET-543, TICKET-632, TICKET-689, TICKET-690
---

# tmux-grade detached agent sessions — sessions Marley can see into (a Marley CORE pillar, future)

## What
**First-class tmux support (pure tmux, or the agent bridge speaking the same shape): agent sessions
that OUTLIVE Marley, that a manager AI creates and drives, that a human can watch from iTerm OR
Marley, and that Marley has FULL insight into — list, read, watch live, attach — including over ssh
into VPS agent environments.**

chad (2026-07-15): *"tmux or tmux like support either via agent bridge or pure tmux. We have proved on
another project that a manager AI can spin up other agents to work. And then i can watch that entire
process take place inside of iTerm by reading the tmux session. The session lives and continues. The
manager even has the ability to load it up for me inside of iTerm. Its a beautiful thing. … we need to
fully have insight into tmux sessions ran from Marley so we can [view]. This enables us to be able to
reach out into VPS agent environments to work."* One of the **two concepts that are going to be the
core of Marley** (the other: [MCP-enabled everything](mcp-first-class-control-plane.md)).

## The architecture doc already exists — this intake RATIFIES the direction
`docs/marley_architecture/detached-sessions.md` (2026-07-15, status CONCEPT) captures the model:
**"The agent session lives outside the app. Marley is a viewport onto it, not its owner."**

> **End-goal design added 2026-07-19: `docs/marley_architecture/fleet-control-plane.md`** (owner +
> fleet-manager note, grounded in a real fleet night). It sharpens this pillar into
> **"tmux is the substrate, never the API"**, names the **pre-Marley deliverable** (a `seat_events`
> event contract on Forge PG + `LISTEN/NOTIFY`, dogfooded in ucsosv2), and specs Marley's 8-part
> first-class surface (fleet rail, passthrough, questions-as-forms, dispatch composer, Forge-objects-as-panes,
> verify-and-merge, owner inbox). **It converges this pillar with [MCP-first-class](mcp-first-class-control-plane.md)
> + [mission-control](mission-control-hypermedia-surface.md).** On the roadmap under "The Fleet Control Plane".
> **Implementation deep dive 2026-07-20: `docs/marley_architecture/orchestration-shell.md`** — the cast
> (Forge = brain, manager seat = policy loop, Marley = mechanism shell), the **project-agnostic `Session`
> envelope** (Marley is not UCSOS-baked), push decided as MCP-native subscription, and the browser +
> hosted-agent extension. The first slice (this file's "Promotion" section) is that doc's **Layer 1**. Today
`marley_agent` spawns agents as child processes, so *"the act of building Marley kills the agents
Marley exists to run."* The trial is running OUTSIDE Marley now (tmux on the dev box + iTerm2 `-CC`,
driven by the UCSOS manager) — that trial is exactly the "proved on another project" chad cites, and
it is the protocol proof this pillar promotes from. chad's message upgrades that doc's status: this
is not one option among several — **it is core**.

## The requirements, in chad's terms
1. **The session lives and continues.** Detach/attach; quit or rebuild Marley and nothing dies.
2. **Interop with real tmux, both directions.** iTerm can attach to what Marley/the manager created
   (`-CC` is the proven UX); Marley can see sessions created elsewhere. The manager can "load it up
   for me inside of iTerm" — surfacing a session to the human is a first-class verb, whatever the
   viewport.
3. **Full insight into Marley-spawned sessions.** Enumerate (`list-sessions`/`list-windows`), read
   scrollback (`capture-pane -p -S -` — the doc's empirically-verified dead-pane rule), watch live,
   attach as a native pane. Insight is for BOTH the human and the manager agent.
4. **VPS reach.** ssh + tmux on the remote box = the same insight into remote agent fleets. Rides the
   `marley_remote` ssh-argv seam AS-IS (`ssh dev -t 'tmux attach -t agents'`) — no new transport, no
   relay, no secrets, consistent with the no-cloud stance.
5. **Manager orchestration.** The manager spins up workers (one session per fleet, one window per
   ticket, per the doc's rules), drives them (`send-keys`), reaps them, and escalates to the human.

## The adoption ladder (from the architecture doc — unchanged, now with a destination)
1. Attach in a normal pane (`ssh … tmux attach`) — zero Marley code, works today.
2. Manager drives via `send-keys`; Marley just watches — **the step being trialed now, outside Marley**.
3. **tmux control mode (`-CC`) client** — each tmux window renders as a native Marley pane/tab (the
   iTerm2 model). This is where "tmux support" becomes invisible and it just looks like Marley.
   FIRST-CLASS means the ladder gets climbed to here, not parked at step 1.

Transport stays dual per the doc: **tmux** where a human may sit in it (dev box, VPS), the
**ignibyte-bridge** (pty_agent) where nobody does (sandboxed SaaS containers) — same seam, two
backends; the bridge deliberately has no `attach`, tmux deliberately does. Marley must not care.

## Ties
- [mcp-first-class-control-plane](mcp-first-class-control-plane.md) — the session verbs
  (create/list/read/send/attach/surface-to-human) are exactly an MCP tool family; the bridge already
  speaks MCP. **The two pillars compose: MCP is how agents reach the sessions; tmux is why the
  sessions survive.**
- [brain-agent-session-supervision](brain-agent-session-supervision.md) — `agent_status_from`'s
  `Waiting` is already the attention model; the manager computes the same signal (output idle →
  NeedsAttention). What's missing is the verb — send.
- [remote-connection-seam](remote-connection-seam.md) / `marley_remote` — the VPS door.
- `terminal_blocks` — renderer ⊥ transport is already ratified; a tmux window is just another byte
  source feeding blocks.

## Notes
- Not scheduled; **the IDE milestones continue first** (chad, same message). The out-of-Marley trial
  keeps running and de-risks the protocol for free in the meantime.
- Open questions stay owned by `docs/marley_architecture/detached-sessions.md` (AgentRun ownership
  flag; scrollback replay vs from-now; who reaps; which crate hosts the control-mode client).
- Provenance: `[Marley-original]`; tmux over argv, no license entanglement (same posture as
  marley_remote shelling to the user's own ssh).

## Promotion
This is NOT an active pipeline doc — it is a candidate pillar. Promote via `/work` when chad calls
it; the natural first slice is ladder step 2.5 — Marley ENUMERATES + READS (insight before control):
a sessions rail section listing tmux sessions (local + configured ssh hosts) with scrollback peek,
before any control-mode rendering.
