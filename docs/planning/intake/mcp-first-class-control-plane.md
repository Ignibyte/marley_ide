---
status: intake
created: 2026-07-15
ticket: unassigned
pipeline_spec: unassigned
---

# MCP-enabled everything — the first-class control plane (a Marley CORE pillar, future)

## What
**MCP (Model Context Protocol) as a first-class, bidirectional citizen of Marley — an agent should be
able to control nearly everything a human can, down to typing in the IDE.**

chad (2026-07-15): *"MCP enabled everything. MCP is going to be a huge part of this. It should be able
to control nearly everything … We need first class support … all the way up to even typing in the
IDE."* One of the **two concepts that are going to be the core of Marley** (the other:
[tmux-grade detached sessions](tmux-grade-detached-agent-sessions.md)).

## The two directions

**1. Marley EXPOSES an MCP server** — the IDE as a toolbox. The tool surface, roughly by organ:
- **Workspace/shell**: open/close workspaces, list projects/tabs/panes, focus, split, launch/close.
- **Terminal**: run a command in a pane, read blocks (the `terminal_blocks` command+output units — an
  MCP-perfect shape: structured, bounded, addressable), read a block's exit/cwd/duration, scrollback.
- **Editor**: open a file, read/edit/TYPE (through the ONE insert mechanism, `ime::replace_text`),
  move the caret, save, query selections — the full "typing in the IDE" bar.
- **Intelligence**: query diagnostics (the per-path store), hover/definition/references/completions
  via the shipped LSP host — the agent asks the IDE what the compiler knows.
- **Command layer**: dispatch any palette command by id (the same enumerable verb set mission-control
  rides), workflows, theme, settings.

**2. Marley CONSUMES MCP** — the agents Marley hosts (agent panes, the brain, the manager) get MCP
client capability, with servers configured in settings — a `[[mcp.servers]]` table mirroring the
shipped `[[lsp.servers]]` round-trip pattern (#308; the #87/#204 settings idiom). Marley's own forge
sidecar is already an MCP server Marley-adjacent agents use daily; this makes the pattern symmetric.

## Why the architecture is already 90% shaped for this
- **Every feature ships as a pure seam + a thin shim** (the constitution's testing doctrine). A pure
  seam is *mechanically* exposable as an MCP tool: typed inputs → typed outputs, no gpui in sight.
  The MCP server is a THIRD consumer of the same seams (after the UI and the tests).
- **`headless_drive.rs` proves the drive surface exists** — the whole app boots headless
  (`RootView::new_in`) and is driven by injected keystrokes today, in tests. MCP is that lane made a
  stable, permissioned, external protocol instead of a `#[cfg(test)]` one.
- **The palette's command ids** are already the enumerable verb set ("every cockpit cmd resolves to a
  verb" — the #204 invariant). `tools/list` is close to a projection of it.
- **Mission control** (see [mission-control-hypermedia-surface](mission-control-hypermedia-surface.md))
  settled "desktop for hands, web for intents" — MCP is the *agent* half of the same doctrine: agents
  get intents AND hands (the tool surface), humans get the organs.

## The line that must be drawn (day-one stance)
- **A permission model, not an open port.** Tool calls that mutate (type, run, edit, save) are gated
  by scope grants the human sets — the Claude-Code-style approve/allowlist model, per server, per
  tool class. Read tools (blocks, diagnostics, state) are the loose tier; the editor/terminal WRITE
  tier is explicit. TICKET-043's lesson stands: *"the operator may not want the system seeing
  everything they do"* — observation scopes are opt-in per pane/session, mirroring the
  manager-created-only rule in the tmux pillar.
- **Local transport first** (stdio / unix socket). Remote MCP rides the mission-control auth story
  when that lands, not its own.

## Ties
- **`docs/marley_architecture/orchestration-shell.md` (2026-07-20) — the implementation deep dive that
  makes this pillar concrete**: `marley_mcp` (the expose-side server + permission tiers), the consume-side
  subscription (push = MCP resource subscription, decided), the receipted session/editor verb families, the
  CDP→MCP **browser** tool family (Phase E), and the **hosted agent** — Marley launching its own Claude/Codex
  seat pointed at its own loopback MCP server ("MCP-enabled everything", turned on itself).
- [tmux-grade-detached-agent-sessions](tmux-grade-detached-agent-sessions.md) — the manager agent that
  spins up/observes worker sessions WANTS these tools (`session.create/read/send/attach`); MCP is the
  protocol those verbs are served over. The two pillars compose into the VPS agent-fleet cockpit.
- [brain-agent-session-supervision](brain-agent-session-supervision.md) — `session.read` delta is one
  of these tools.
- [mission-control-hypermedia-surface](mission-control-hypermedia-surface.md) — same verb set, human
  transport vs agent transport.
- `docs/marley_architecture/detached-sessions.md` — ignibyte-bridge already speaks "structured
  protocol + MCP"; Marley meeting it as MCP client closes that loop.

## Notes
- Not scheduled; **the IDE milestones (M20 remainder, M21) continue first** (chad, same message:
  "we are currently still focused on the IDE part right now").
- v1 slice when promoted: the read-only server (blocks/diagnostics/state) + `terminal.run` behind
  approval — smallest thing that lets an external agent pair with Marley.
- Provenance: MCP is an open spec (Anthropic, MIT-licensed SDKs); implementing a server/client from
  the published spec is the same §20 posture as LSP 3.17 was for M20.

## Promotion
This is NOT an active pipeline doc — it is a candidate pillar. Promote via `/work` when chad calls
it; expect it to decompose into a ticket BATCH (server core + tool tiers + permissions + settings),
not one ticket.
