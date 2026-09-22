# Marley × Forge — The Fleet Control Plane (End-Goal Design Note)

> **Status (2026-07-19) — DESIGN NOTE, owner + fleet-manager authored. Not built, not scheduled.**
> The concrete end-goal design that **converges three ratified core pillars** —
> [tmux-grade detached sessions](../planning/intake/tmux-grade-detached-agent-sessions.md),
> [MCP-first-class control plane](../planning/intake/mcp-first-class-control-plane.md), and
> [mission-control hypermedia](../planning/intake/mission-control-hypermedia-surface.md) — and grounds
> them in a real night of fleet operation. It **extends [detached-sessions.md](./detached-sessions.md)**
> with the sharper thesis *"tmux is the substrate, never the API,"* the completion-signal hierarchy,
> and the `seat_events` event contract.
>
> **What to build first is NOT Marley code.** The IDE milestones continue first (the pillars' standing
> stance); the **pre-Marley deliverable is the event contract (§6)**, dogfooded in ucsosv2, which
> Marley later *consumes unchanged*. On the roadmap: [roadmap.md → The Fleet Control Plane](./roadmap.md#the-fleet-control-plane--marley--forge-the-orchestration-end-goal).
>
> **Implementation deep dive (2026-07-20): [orchestration-shell.md](./orchestration-shell.md)** — the owner
> conversation that digested this note. It fixes the cast (Forge = brain/state, manager seat = policy loop,
> Marley = mechanism shell), makes the design **project-agnostic** (the generic `Session` envelope; UCSOS via
> the Forge adapter only), decides push (MCP-native subscription, not raw PG `LISTEN`), and adds the
> **browser + hosted-agent extension** (the project Forge URL in the embedded Chromium — *retired by the
> 2026-08-09 scrap-forge pivot, #410/#411; see that doc's amendment banner — and #411 has now LANDED:
> the fleet-brain forge transport, the sprint cockpit, and `marley_forge_client` itself are ripped out;
> the fleet rail survives forge-agnostic (#411): a bare "Fleet" header, the demo feed, dispatch/answer
> machines terminating in "no fleet transport" — and the Forge-facing claims below carry (retired #411)
> markers where they read as live design*; Marley hosting its own Claude/Codex seat). Read that doc for
> *how this gets built*.

**Written:** 2026-07-19 (manager seat, evening session)
**Audience:** the Marley project — so the IDE/TUI is built against the real end goal, not a guess.
**Origin:** a live design conversation between the owner and the fleet manager during an evening
that dispatched 4 agent seats, merged 5 PRs, and hit every classic remote-agent failure mode in
one night. Every requirement in here traces to something that actually happened.

---

## 1. What Marley is (as stated by the owner)

An IDE/TUI that integrates **Forge as first-class** — with tmux support and everything. Not an
editor with a Forge plugin: an IDE whose *project model is the work-state machine* (tickets,
sprints, pipeline runs, gates, PRs, agent seats), with the filesystem and terminals attached.

The fleet today: a manager agent (Claude, on the owner's desktop) orchestrating worker agent
seats (Claude Code sessions in tmux on dev-1/dev-2/dev-3 + a forge box seat) via SSH, tmux
`capture-pane`/`send-keys`, and the Forge MCP (tickets, mailbox, knowledge, pipeline events).
Marley is the permanent home this grows into.

---

## 2. The evidence night — why this design (2026-07-19)

Everything below happened in one evening of real fleet operation. These are requirements
wearing incident costumes:

| Incident | Root cause | Design lesson |
| --- | --- | --- |
| 3 seats sat idle holding **unsent briefs** — the dispatched text was in the composer, Enter never took | `tmux send-keys` is fire-and-forget; no submit receipt | Dispatch needs **delivery states** (deposited → claimed → started) |
| All 3 dev seats **dead-stalled at first tool approval** | Sessions recreated post-reboot with bare `claude` (no permissions flag); nothing surfaced the mode | Seat **capabilities/mode must be declared state**, checked before dispatch |
| Detector reported live seats **UNREACHABLE** | It screenshotted tmux window `:1`; rebooted sessions start at window `0` | Screen addressing is not identity; seats need **stable identity + self-reported liveness** |
| Fleet status = grep screenshots for the literal string `esc to interrupt` | Terminal text is the only observable | State must be **declared by the agent**, not inferred from a picture |
| A seat died on **API 529/500 three times**; sat "idle" 15 min before anyone noticed | Dead-and-idle are indistinguishable in a screenshot | **Error is a first-class state**, distinct from idle; absence-of-heartbeat is a signal |
| A menu had to be answered by **pressing the digit "4" at a screenshot** | Questions surface only as rendered text | Interrupts must carry **question + options as data** |
| One base-red got independently diagnosed + filed by **three seats in 90 minutes** | No shared, queryable "known branch state" | Branch health belongs in a **registry**, cited by gates |
| Shift-Tab meant to cycle modes instead **poked an open dialog** | The sender can't know a dialog is up | Input without state knowledge is unsafe; typed commands beat keystrokes |

**And the one thing that never failed:** the Forge **mailbox** (deposit → claim → response with
`responded_at`). Zero failures across every dispatch, all night. That half of the system is
already control-plane-shaped. The lesson of the night in one line: *everything that flowed as
data worked; everything that flowed as keystrokes-and-screenshots broke.*

---

## 3. The core thesis

> **tmux is the substrate, never the API.**

- **Keep tmux (or keep multiplexing pluggable)** for what it is unbeatable at: process
  persistence across disconnects/reboots, human attach/detach, a hardened PTY layer. Rebuilding
  this = years of terminal edge cases (PTY quirks, resize, scrollback, unicode, mouse) for zero
  differentiated value. Consider **zellij** as an alternative substrate (Rust-native, WASM
  plugin API — a Marley plugin could live inside the multiplexer); tmux wins on ubiquity.
  Either way: substrate.
- **Stop using keystrokes + screen-scraping as an RPC protocol.** That is the disease behind
  every incident above. The cure is a **control plane beside the terminal stream**: agents
  declare state and accept typed, receipted commands; the terminal remains the human window.

Metaphor that stuck: the terminal is the **cockpit window**; the control plane is the **radio
and the flight recorder**. Marley renders both, side by side.

---

## 4. Completion signals — the hierarchy

"How do you know something finished?" has three answers, weakest to strongest:

1. **PTY silence** — the bridge's current mechanism: poll `bridge-session-status`, watch
   `idle_seconds`/`output_bytes`; long silence ≈ "finished responding," then read the screen to
   see what state it landed in. Program-agnostic (works for a dev server or a REPL), which is
   why the bridge uses it — but silence is ambiguous: *done*, *stuck at a menu*, and *waiting
   on a question* all look identical. Keep as the universal fallback + wedge detector.
2. **Harness events** — the truth lives *inside* the agent. Claude Code fires hooks at turn
   end and at every pipeline phase boundary; those are deterministic events, not inferences.
   A ~20-line hook that POSTs `{turn-complete | phase-pass | halted-with-question | error}` to
   Forge is **the single missing brick** in the whole architecture.
3. **Domain events** — the strongest: work-level completion as data. Phase rows and gate
   results (already recorded by seats today), ticket status flips, PR-opened, mailbox
   `responded_at`. The forge seat's "PR is open" reply attached to its mailbox row was a
   completion signal delivered perfectly, with no terminal involved.

**Push vs poll, honestly:**
- Forge is Postgres-backed → **`LISTEN/NOTIFY` gives Marley true push.** A long-running
  program subscribes and repaints the moment `phase: validate → PASS` lands. No polling
  anywhere in the end state.
- A turn-based agent (today's manager) can't be interrupted mid-nothing — but its harness has
  Monitor/notification primitives that fire on stream events, collapsing "poll every 15 min"
  into "wake on write." The 15-minute tick demotes to a heartbeat fallback.
- The "you must poll" limitation is a property of *turn-based agents*, not of the design —
  a quiet argument that Marley (a real process) is the right permanent home for orchestration,
  consulting the manager agent for judgment rather than mechanics.

---

## 5. What already exists (build on, don't rebuild)

| Piece | Status | Notes |
| --- | --- | --- |
| **Forge mailbox** (`agent-prompt-run-record` delivery=mailbox → `agent-prompt-claim`, response attach) | **Proven in production** | The dispatch half of the control plane. Zero failures. Templates + bundle hashes + receipts already recorded |
| **`bridge-session-*` daemon tools** (start/send/keys/read/screen/status/stop) | Shipped | PTY sessions owned by the forge daemon. `bridge-session-send` already delivers Enter **as a separate write** — the swallowed-Enter bug is pre-solved there |
| **`bridge-remote-start`** (WORK-1109) | Shipped | Starts a Claude seat **on a fleet box over SSH**, daemon-owned, box allowlist (`FORGE_BRIDGE_REMOTE_HOSTS`). The seat-adoption path exists as one MCP call |
| **Pipeline phase events + gate results in Forge** | Recorded today | Seats write them as they work; nothing consumes them live yet *(the planned Marley consumer retired #411)* |
| **Claude Code hook system** | In production (repo hooks) | The natural emitter site: Stop hooks, phase gates already fire deterministically |
| **tick-state.sh detector** | In production | The screen-scraping fleet reader — becomes the *fallback* sensor, and its state vocabulary (WORKING/IDLE/AT_MENU/RATE_LIMITED/UNREACHABLE) seeds the event schema |
| **Manager-side wake primitives** (Monitor, notifications, cron) | In harness | The "wake on write" mechanism for turn-based agents |

Important nuance: **the manager never runs *under* the bridge.** It is the bridge's *caller*
(MCP client), wherever it lives. "Uses the bridge" ≠ "lives inside the bridge." Seats are what
may become bridge-owned — and only when Marley's terminal passthrough makes that *nicer* to
watch than tmux, because today a daemon-owned seat leaves the owner's iTerm attach flow.

---

## 6. The event contract — the pre-Marley deliverable

Define this FIRST. The ucsosv2 dogfood hook emits it now; Marley consumes it later unchanged.
The contract is the deliverable; the IDE is a consumer.

```jsonc
// seat_events (Forge PG table + LISTEN/NOTIFY channel)
{
  "session_id":  "dev-1/agent1",         // stable seat identity (not a tmux window index)
  "box":         "dev-1",
  "ticket":      1696,                    // nullable
  "phase":       "implement",             // nullable; pipeline phase slug
  "state":       "working",               // working | idle | at-menu | error | done | starting
  "event_type":  "phase-pass",            // see enum below
  "payload":     { },                     // event-specific, versioned
  "capabilities": { "mode": "bypass", "model": "opus-4-8", "effort": "max" },
  "ts":          "2026-07-20T02:10:00Z",
  "schema":      1
}
```

**Event enum (v1):** `session-start` · `heartbeat` · `turn-complete` · `phase-start` ·
`phase-pass` · `phase-fail` · `gate-result` · `halted-with-question` (payload: question,
options[], context refs) · `api-error` (payload: status, attempt) · `pr-opened` (payload: repo,
number, held: bool) · `dispatch-claimed` · `car-complete` · `session-end`.

**Transport:** seat hook POSTs to a Forge endpoint (or MCP tool) → row + NOTIFY. Consumers:
Marley (subscribe — *retired #411*), manager agent (Monitor/one-query tick), dashboards, audit.

**Derived rules the consumers get for free:**
- *stuck* = no `turn-complete`/`heartbeat` AND bridge `idle_seconds` climbing → unambiguous,
  fixes the dead-vs-idle blindness.
- *unsafe-to-dispatch* = capabilities.mode ≠ bypass → refuse with a reason (fixes the
  manual-mode dead-stall class at the protocol level).

---

## 7. Marley: the first-class feature set

Core data model, not plugins:

1. **The event contract as the spine.** Every pane is a subscriber to `seat_events` +
   Forge work-state *(retired #411 — the shipped subscription transport is gone)*. LISTEN/NOTIFY in;
   no screen-scraping anywhere in the primary path.
2. **The fleet rail — seats as first-class objects.** Live state, current ticket, phase,
   elapsed, held PR, last event, capabilities. This *is* the tick detector promoted to
   always-on truth. Includes the **branch health / base-red registry** line (known base-reds →
   owning ticket), cited by gates so the triple-filing class dies.
3. **Terminal passthrough per seat.** The real PTY stream rendered natively — watchable,
   typeable, with a who's-driving indicator and clean human-takeover/hand-back. The single
   feature whose absence blocks migrating seats off tmux. (Substrate: tmux pane, bridge PTY,
   or local — abstracted behind one session model.)
4. **Structured interrupts — questions as forms.** `halted-with-question` renders the actual
   question + options as buttons with context attached; the answer flows back as a receipted
   typed reply. No digits pressed at screenshots.
5. **Dispatch composer with delivery states.** Pick ticket → assignment template renders as a
   form (the a2a flow) → merge-scope as a field (`cheap-self-merge` / `hold-at-PR`) → deposit →
   watch `deposited → claimed → started` on the seat rail. Auto-refuses seats whose declared
   capabilities are wrong.
6. **Forge work objects as native panes** *(retired #411)*. Ticket ↔ pipeline run ↔ phase/gate evidence ↔ held
   PR ↔ AAR as one linked view. The IDE's project model *is* the work-state machine with the
   filesystem attached — this is what "Forge first-class" cashes out to.
7. **Verify-and-merge as a guided flow.** Held PRs land in a review pane: diff, the seat's
   evidence, buttons that run the manager-side verification and show results, then merge.
   (Ran five times by hand tonight; highest-stakes repetitive ritual in the system.)
8. **Owner-class inbox, separate from the firehose.** Holds awaiting a human call, deploy
   rituals pending, policy questions — routed distinctly from manager-class noise. The
   activity feed is for the orchestrator; the inbox is for the owner.

**The role split this produces:** Marley = hands and eyes (mechanics, watching, routing);
the manager agent = judgment (merge calls, curation verdicts, halt decisions), *invoked by*
Marley. The same brain/hands split the fleet already runs (spec-first manager → worker seats),
applied one level up.

---

## 8. Adoption path (no big-bang, dogfood first)

**Layer 1 — now, in ucsosv2, zero topology change:**
- The seat state-emitter hook (harness events → Forge, schema §6). ~20 lines riding the
  existing hook system. Sessions stay in tmux, launched exactly as today.
- Tick detector reads Forge state first, screen-scrapes only as fallback.
- `dispatch.sh` hardening: verified submission (text → Enter → confirm state change → retry
  once) + capability preflight. Every failure mode it encodes is a requirement Marley inherits.
- Manager Monitor sits on the event feed → near-real-time reaction without architecture change.

**Layer 2 — per-seat opt-in, when passthrough exists:**
- One experimental bridge-owned seat (`bridge-remote-start dev-2`) to feel the trade: receipts
  and daemon ownership vs losing the native tmux window until Marley renders passthrough.

**End state — Marley:**
- Subscribes to everything; renders §7; seats bridge-owned where it helps; tmux remains for
  humans and as the degraded-mode fallback (Marley down ≠ fleet down: the mailbox + tmux path
  keeps working — that resilience is a feature, keep it).

---

## 9. Open questions (deliberately unresolved)

- **Substrate:** tmux vs zellij vs pluggable — decide when passthrough is built; nothing
  upstream depends on it.
- **Event schema versioning + retention** (schema field is there; pruning policy TBD).
- **Multi-project:** seats serve one repo today; Marley's session model should assume
  N projects × M seats from day one (Forge is already multi-project).
- **Auth/trust:** bridge is admin/capability-gated with box allowlists today — Marley needs the
  same posture per pane (who may dispatch, who may merge, who may take over a terminal).
- **The attach UX bar:** bridge-owned seats must be *nicer* to watch than iTerm-attached tmux
  before any migration is proposed. This is the make-or-break of the whole transport story.

---

## 10. One-line summary

**Dispatch as receipted messages, status as declared events, terminals for humans —**
tmux stays as the substrate, Forge becomes the nervous system, Marley is the body that
finally has both eyes and hands.
