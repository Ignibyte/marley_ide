# Detached agent sessions — the session is not owned by the app

> **Status (2026-07-15) — CONCEPT. Not built, not ratified.** Being trialed *outside* Marley first
> (tmux on the dev box + iTerm2 `-CC`, driven by the UCSOS manager) so the protocol is proven before
> any Marley code moves. This doc exists to capture the idea and name the seams it lands on.
> Relates to: [`marley_agent`](./marley_agent.md) (the run model), [`marley_remote`](./marley_remote.md)
> (the ssh-argv seam), [`terminal_blocks`](./terminal_blocks.md) (the renderer).
>
> **Sharpened 2026-07-19 → [fleet-control-plane.md](./fleet-control-plane.md).** A real night of fleet
> operation upgraded "renderer ⊥ transport" into the thesis **"tmux is the substrate, never the API"** and
> named the missing brick: a **`seat_events` event contract** (agents *declare* state; no screen-scraping on
> the primary path). That note is the concrete end-goal; this one remains the transport/ownership model it sits on.

## The idea, in one line

**The agent session lives outside the app. Marley is a viewport onto it, not its owner.**

## Where Marley is today

`marley_agent` launches agents as **CLI subprocesses in terminal panes** — `launch_command(Claude)`
→ `"claude"`, spawned as a child, with `AgentRun` tracking it and the pump projecting
`AgentStatus` from output activity.

That means **Marley owns the child process**, and therefore:

- Quit or crash Marley → every agent dies with it.
- Marley is 51k LOC under active development. It gets restarted dozens of times a day.
- So today, *the act of building Marley kills the agents Marley exists to run.*

That is the whole motivation. Nothing about the agent cockpit's design is wrong — the **ownership**
is.

## What changes

`AgentRun` stops being "a child I spawned" and becomes **a handle to a session that outlives me**.
Attach → the agent is there, mid-work, with its scrollback. Detach → it keeps working. Re-attach an
hour later, or from a different machine, or from iTerm instead of Marley → same session.

The mental model is Rusty's BetterDisplay vscreen: the vscreen exists on the box whether or not CRD
is connected; CRD is only a viewport. Kill the viewport, nothing is lost. tmux is that, for
terminals.

## Why the seams already fit

This is not a rewrite. Three pieces of Marley already point at it:

| Existing | Why it fits |
|---|---|
| **`marley_remote`** — *"spawn the user's OWN `ssh` client as a terminal pane; `ssh` owns ALL security"* | A detached remote session is just `ssh dev -t 'tmux attach -t agents'`. It rides the ssh-argv seam **as-is** — no new transport, no relay, no secrets, consistent with the no-cloud stance. |
| **`agent_status_from(exited, active, quiet_ticks)`** → `Waiting` | Already the correct state model. `Waiting` (quiet_ticks ≥ `WAITING_TICKS`) *is* "agent has gone quiet, needs input". The UCSOS manager computes the same thing (output idle > 180s → `NeedsAttention`). The signal exists on both sides already; what's missing is the verb — **send**. |
| **renderer ⊥ transport** (the ratified split) | Detached sessions are a **transport** concern. The block renderer doesn't care whether bytes come from an owned PTY, an ssh pane, or a tmux window. |

## Transports

Same shape, two backends, one seam. Marley should not have to pick.

| Context | Transport | Why |
|---|---|---|
| Chad, dev box, Marley-in-development | **tmux** | Free, persistent, attachable, battle-tested, works today. Manager drives it with `send-keys` **locally on the box the agent runs on** — no GUI, no network hop. |
| Ignibyte SaaS / sandboxed agents | **[ignibyte-bridge](https://github.com/chadmandoo/pty_agent)** | No tmux dependency to ship in a container product; structured protocol + MCP; service-owned durability. |

Note what ignibyte-bridge's own DESIGN.md says: *"The long-term goal is not to replace iTerm or
tmux. It is to provide the same core capability they expose to agents today."* And: *"Goal: make
`screen` work like tmux `capture-pane`."* The bridge is the productizable sibling, not the rival —
and it deliberately has **no `attach`** (it's listed under "Possible future commands"), because a
daemon-owned PTY is the opposite of a human sitting in it. That is exactly why tmux is the right
transport when a human wants to watch and intervene, and the bridge is right when nobody does.

## The adoption ladder

Deliberately incremental. Step 1 needs **zero Marley code**.

1. **Attach in a normal pane.** `ssh dev -t 'tmux attach -t agents'` in any terminal pane. You get
   tmux's own status bar and prefix keys. Ugly, free, works today, proves the model.
2. **Manager drives.** The manager `send-keys` into windows it created. Marley just watches. This is
   the step being trialed now, outside Marley.
3. **Control-mode client.** Implement tmux control mode (`-CC`) so each tmux window renders as a
   native Marley pane/tab — what iTerm2 does. This is where it stops looking like tmux and starts
   looking like Marley. Only worth doing once 1–2 prove the protocol.

## Rules that fall out

- **Manager-created only.** The manager creates, drives, and reaps **only** windows it named. A
  human's ad-hoc shell is invisible to it. This is not a nicety — it's the line drawn when
  TICKET-043 was scrapped (*"the operator may not want the system seeing everything they do"*).
  Manager-owned sessions honor that by construction.
- **Session reused, windows per-run.** One long-lived session (the container / the fleet view); one
  fresh window per ticket. Fresh-per-run keeps context clean, keeps memory bounded, and makes the
  window name *be* the ticket. The per-project **Primary** is the standing exception — a permanent
  window.
- **`remain-on-exit on`, and read dead panes with `-S -`.** Without it, a crashed agent's window
  vanishes and takes the evidence with it. With it the window persists — but tmux **overwrites the
  visible screen with a `Pane is dead (status N, <time>)` banner**, so a plain `capture-pane -p`
  returns the tombstone instead of the agent's last words. `capture-pane -p -S -` reads the
  scrollback, where the output actually is. `#{pane_dead_status}` carries the real exit code.
  *Verified empirically 2026-07-15 on tmux 3.4 (dev box) and 3.7b (Mac) — identical on both.*
- **Purity is unaffected.** Idle/status stays a pure projection over fed-in values
  (`agent_status_from`); nothing here needs a clock inside the domain crate (`Date::now` stays
  banned).

## Open questions

- Does `AgentRun` model "session I attached to" and "child I spawned" as one type with an ownership
  flag, or two? (Bridge and tmux both want the former.)
- Reconnect semantics: on attach, replay scrollback into blocks, or start from now?
- Who reaps a finished window — the manager after recording the outcome, or the human?
- Does the tmux control-mode client belong in `marley_remote`, `terminal_blocks`, or its own crate?

## Provenance

`[Marley-original]`. tmux is an external process invoked over argv — no source lineage, no license
entanglement, same posture as `marley_remote` shelling out to the user's own `ssh`.
