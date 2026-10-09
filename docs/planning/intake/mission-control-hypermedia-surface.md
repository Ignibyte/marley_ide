---
status: superseded
created: 2026-07-14
ticket: unassigned
pipeline_spec: unassigned
note: audit (2026-10-09): its state source, Forge, left in TICKET-409 to TICKET-411; the phone path became TICKET-535 and, per Chad on 2026-10-02, a phone app with a relay at the end (design-notes/herdr-and-hermes-2026-10-02.md)
---

# Mission Control — the web/remote hypermedia surface (a Marley pillar, future)

## What
**One remote surface — "mission control" — that exposes Marley's STATE (forge tickets/sprints,
pipeline phase, gate results, agent status, workflows, notifications) to any browser or phone, with
intent-level ACTIONS back** (approve/deny a push, answer an agent's question, start/stop a
ticket/run, comment, run-a-command-in-a-pane). Served as **hypermedia** (server-rendered HTML
fragments + scalar signals over SSE, actions up) from the same Rust state Marley already holds — no
second frontend app, no client-side state replica, no API contract to drift.

chad (2026-07-14): "the real problem im trying to solve is the tension between web and desktop …
1) Remote control from app or phone 2) Forge, pipeline, agent status, workflow, etc etc and sharing
that to the web … that requires 2 different apps … or you limit the web to really read only. If you
try to port the web you get a clunky xterm silliness. do we just pick a lane?"
**Decision (chad, same day): "I think you are right and lets lock this in."**

## The doctrine — pick the lane for the ORGANS, not the state
> **Desktop for hands, web for intents.**

- **The organs** — terminal grid, editor, embedded Chromium — are keystroke-latency, cell-grid,
  glyph-level surfaces. **Desktop-only, forever.** The things we REFUSE to build: a web IDE, a
  phone terminal, any xterm.js port. That's where "clunky" lives (VS Code tunnels is the
  counterexample that proves the cost; Zed picked desktop-only and simply has no remote story).
- **The state** — forge / pipelines / agents / workflows — is one mission-control surface.
  chad's problems #1 (phone control) and #2 (web sharing) are the SAME product: observation +
  a small enumerable verb set. "Remote control from a phone" never meant typing shell characters
  on an iPhone; it means *see the fleet, issue intents*.
- "Read-only web" was a false choice — it assumed the web surface is a second app that would have
  to reimplement Marley's logic to be allowed to write. Actions-up into the EXISTING command layer
  makes it read-mostly-write-intents with no second app.
- Prior art convergence: Cursor background agents / Codex / Claude Code web+mobile all landed on
  exactly this split (desktop/CLI for hands-on; web/phone for fleet observation + approvals).
  Warp's block sharing is the terminal-sharing half.

## Architecture — hypermedia T1 (fragments/signals/actions)
Per the design doc `~/Projects/rusty/docs/hypermedia-desktop-rust.md` (the 2026-07-12→14 sessions;
brain `projects/dom-engine-for-consoles`): **the backend owns all truth; the client renders
presentation, never replicates data.** Three message types are the whole contract:
- **down — fragment patch**: HTML targeted at an element id, morphed in place (focus/scroll survive);
- **down — signal patch**: a bare scalar (`{phase: "validate"}`) written to bound nodes — the hot
  path, no re-parse (pipeline phase chips, gate counters, elapsed timers);
- **up — action**: `{action, payload}` → dispatched into the SAME command layer the palette uses.

Views are **askama templates** — on-disk HTML, compiled + type-checked against Rust structs at
build time, in THIS repo, rendering the same data the gpui cockpit renders. The "web app" is
templates + one SSE route; there is no frontend build, no React, no replica, no cache invalidation.
Datastar.js (~14 KB) is the entire client runtime.

## The terminal shares as BLOCKS, never grids
`terminal_blocks` already structures a session as command+output units (cmd, cwd, exit, styled
output). A **block renders as clean HTML** — selectable, searchable, beautiful (Warp-share-style).
A running command is a block with a streaming tail (fragment/signal patches into one element).
Running something remotely = an *action* ("run X in pane Y"); the block streams back. ~90% of
remote-terminal value, 0% PTY-in-the-browser. **CDP is to the browser what the Block model is to
the terminal — and mission control is the same three-capability seam (launch/control/observe) worn
remotely**: it consumes [[brain-agent-session-supervision]]'s Block/session stream and composes
with [[remote-connection-seam]] (M5).

## Flagship loop #1 — the AskUserQuestion push
Agent hits a fork → push notification → phone browser opens the question card → chad taps an
answer → the autonomous run unblocks. This is THE killer app for how this project actually runs
(long autonomous `/work` trains): the #202 A/B/C product fork sat **days** unanswered while chad
was AFK. Approvals ride the same loop ("validate PASSED on #297 — push? [yes/no]"). Web push via
PWA first; native wrapper later.

## Mobile
Web-first: responsive templates + PWA (installable, push notifications). chad (2026-07-14): "a
mobile app makes sense at some point with the web" — a thin native app LATER, wrapping the same
hypermedia surface (or speaking the same actions protocol natively). Never a second logic codebase.

## Topology — the one open fork (deferred; doesn't block design)
Protocol is identical either way; decide when promoted:
- **(a) Marley serves itself** — in-proc axum, reached over Tailscale. Simplest; dies when the
  desktop sleeps.
- **(b) rusty as the always-on hub** — rusty is ALREADY axum + owns the brain + drives Claude CLI
  subprocesses, and **forge is already a shared sidecar both can read**; Marley publishes an
  event stream (blocks, pipeline events, questions) into it. Favored by the always-on requirement
  (push must work while the desktop is locked/asleep — cf. the locked-machine capture lessons).
  Also keeps rusty as the hypermedia **T1 testbed** (its React replica + `.changed` sentinel is the
  disease this architecture deletes — prove the pattern there first).

## Convergence with the embedded-browser pillar (one HTML dialect, two transports)
The [[embedded-agent-browser-chromium-cdp]] pillar already decided: in-app document/data surfaces
(forge boards, wikis, dashboards) render as HTML inside the embedded Chromium over a `marley://`
custom scheme, with the terminal-stays-native guardrail and the
`ThemeColors → CSS variables` bridge. **Mission control reuses the SAME askama templates and theme
bridge** — one HTML dialect, two transports: custom scheme in-app, HTTPS/SSE remote. The pillar's
"cockpit dashboards ride the web surface" and this pillar's "share them to the web" are one render
path. (The shelved html_pane/subset-gpui-renderer idea stays shelved — see the hypermedia-UI
conclusion memory; it only returns if we ever want these panels WITHOUT Chromium in-process.)

## Security (design-in from day one; highest-priority item, like the browser pillar's)
This is Marley's first **outward-facing** surface:
- **Tailscale-first, no public exposure in v1**; authn required regardless (tailscale identity or
  per-device token — the `marley_forge_client` never-logged-bearer lesson applies).
- **The action verb allowlist IS the security boundary** — a closed enum of intents, each
  authorized + logged; no arbitrary eval. `run-command-in-pane` is the sharp one: gated
  (confirm-on-desktop or allowlisted commands) in v1.
- Fragments render server-side from typed structs — no user-content HTML injection path; escape at
  the template layer (askama auto-escapes).

## Testing boundary (honest, same split as the browser pillar)
Rust seams stay pure + gate-tested to cov/MSI 100: the event stream (block/pipeline/question
serializers), action dispatch (verb enum → command layer), template view-structs, block→HTML
composition. The HTML presentation itself gets web-style + driven-capture testing (a real browser
against a live instance), not MSI theater.

## Scale / shape
A **PILLAR** (its own milestone, M3/M5-adjacent — it consumes agent-session supervision and the
remote seam), not a near-term ticket. Rough seams when promoted: a `marley_mission` crate (view
structs + askama templates + the verb enum + block→HTML, all pure-testable) + the event-stream
publisher on the app side + the axum/SSE adapter (thin, shim-style) wherever topology lands.
Licensing clean: Datastar (MIT), askama (MIT/Apache), axum (MIT) — no §20 concern.

## Promotion
Candidate, not a pipeline doc yet. Promote via `/work` when scheduled. First slice on promotion:
**the AskUserQuestion push loop end-to-end** (event out → question card → answer action → run
unblocks) — smallest full round-trip through every layer (stream, template, morph, action, auth),
and the single highest-value loop for chad's actual workflow. Decide topology (a/b) at that point.
Recorded as forge AD `AD-claude-mission-control-hypermedia-surface-001`.
