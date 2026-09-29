# The Orchestration Shell — Marley × the fleet (implementation deep dive)

> **AMENDED 2026-08-09 (the scrap-forge pivot, #409/#410/#411):** Forge is scrapped for Marley —
> process AND product (`docs/planning/intake/scrap-forge-pivot.md`). The **Forge-facing halves of
> this design are RETIRED**: §8's Forge pane shipped as the #405/#406 embedded-browser train and was
> then de-Forged by **#410** (no production origin source remains; the generic wry pane + lifecycle
> machine survive awaiting a future URL feature), and the per-project `web_url` setting is DELETED
> (#410 — it was never read in production). The fleet/brain transport wiring retires under
> **TICKET-411** (the fleet RAIL stays, forge-agnostic, quiet-Unconfigured until a non-forge brain
> endpoint exists). The body below is the 2026-07-20 record, kept for the mechanism-not-policy
> reasoning that outlives Forge.

> **AMENDED 2026-09-29 (#533):** the dotted verb names below (`fleet.snapshot`, `session.send`, …)
> are this record's own spelling. The wire names are `family_verb` (`fleet_snapshot`, `session_send`)
> since #491, the form a Claude client can call; the grant classes keep their dots (`session.write`).

> **Status (2026-07-20) — DESIGN, ratified direction. Not scheduled; IDE milestones continue first.**
> The implementation synthesis of [fleet-control-plane.md](./fleet-control-plane.md) (the end-goal note),
> produced by the 2026-07-20 owner conversation that digested it. It settles **who does what** (the cast),
> **how Marley stays project-agnostic** (mechanism-not-policy / envelope-not-payload), **how push works**
> (MCP-native subscription — decided), and **the browser + hosted-agent extension** (the project Forge URL
> in the embedded Chromium; Marley hosting its own Claude/Codex seat).
> Companions: [detached-sessions.md](./detached-sessions.md) (transport/ownership) ·
> [../planning/intake/mcp-first-class-control-plane.md](../planning/intake/mcp-first-class-control-plane.md) ·
> [../planning/intake/embedded-agent-browser-chromium-cdp.md](../planning/intake/embedded-agent-browser-chromium-cdp.md) ·
> [../planning/intake/mission-control-hypermedia-surface.md](../planning/intake/mission-control-hypermedia-surface.md) ·
> [../planning/intake/brain-agent-session-supervision.md](../planning/intake/brain-agent-session-supervision.md).
> Roadmap home: [roadmap.md → The Fleet Control Plane](./roadmap.md#the-fleet-control-plane--marley--forge-the-orchestration-end-goal).

---

## 1. The cast — who does what

chad (2026-07-20): *"Marley is the shell with MCP capabilities. Our current work flow is forge is the
brain and there is a manager seat. That manager's job is to run a loop where it basically dispatches
work to 3+ agents. Its loop figures out when things are complete and does qa on them / gives them more
work. Right now that's pure tmux but the idea is making this more structured. … Marley is the
orchestrator of all of the above in a clean and reliable way. That's what we are looking to solve here."*

| Role | Holds | Is | Speaks |
|---|---|---|---|
| **Forge** | **State** (the brain): tickets, sprints, knowledge, mailbox, gate results, `seat_events` | A per-project Postgres-backed service + web UI | MCP server (+ its web URL) |
| **Manager seat** | **Policy** (the loop): dispatch → detect-complete → QA → reassign / escalate | An agent session (Claude/Codex — provider is config, §9) | MCP **client** of Forge *and* of Marley |
| **Marley** | **Mechanism**: reliable verbs, rendering, hosting | The shell — the IDE/TUI | MCP **server** (expose) + MCP **client** (consume) |
| **Worker seats** | The hands doing the work | Agent CLI sessions (tmux today; bridge/local later) | PTY + the event-emitting hook |
| **Human owner** | Judgment above the manager; the inbox | chad, watching through Marley (or iTerm — tmux interop stands) | Eyes + typed intents |

The one distinction that unlocks everything: **the brain (Forge) and the manager are not the same
thing.** Forge holds durable *state*; the manager runs the *loop* (judgment). Marley is neither — it is
the **reliable mechanism plane** both of them act through, and the surface the human watches from.
Today the manager pokes raw tmux (`send-keys`, screenshot-grep); *"making this more structured"* means
inserting Marley as the reliable conduit (§5) and the event contract as the truth channel (§6) — while
the manager keeps 100% of the judgment.

## 2. The principle — mechanism, not policy; envelope, not payload

chad (2026-07-20): *"we would potentially want this to be agnostic and not baked in. Meaning Marley
isn't designed to only work in UCSOS — we have other projects as well."*

> **Marley provides the reliable mechanism and interprets a generic *envelope*. The project-specific
> *policy* and *payload* live outside Marley — in the manager's prompts and the project's brain.**

Marley must never know what a "ticket", a "pipeline phase", or "branch health" *is*. Those are UCSOS
concepts, carried opaquely and rendered generically (chips, cards, timestamps). The fleet-control-plane
note's `seat_events` enum (`phase-pass`, `pr-opened`, `gate-result`, …) is a **UCSOS projection onto
the generic envelope**, performed by an adapter — never by Marley's core.

**This is the LSP pattern again, and M20 already proved it in this codebase**: a generic client built
from a contract, with a per-project server wired in settings. rust-analyzer is to language intelligence
what Forge-for-UCSOS is to orchestration. Marley implements the generic fleet/session *client*; each
project brings its own brain. Wiring rides the MCP pillar's planned `[[mcp.servers]]` settings table —
the same round-trip idiom as the shipped `[[lsp.servers]]` (#308).

**Anti-over-abstraction rule:** design the boundary generic, then ship **exactly one adapter — Forge —
now.** UCSOS is tenant #1; the generic line is what lets tenant #2 exist without a rewrite. The
mechanical check: **a Forge-specific string appearing in `marley_fleet` or the render path is a defect;
it belongs in the adapter** (`marley_forge_client`).

## 3. The generic `Session` envelope (v1 proposal)

The one schema agnosticism lives or dies on. Harden it at Layer-1 promotion; this is the conversation's
v1:

```jsonc
Session {
  id:         "dev-1/agent1",        // STABLE identity — never a tmux window index (the evidence-night lesson)
  title:      "…",                   // human label
  state:      Starting | Working | Idle | Waiting | Error | Done,
  question:   { prompt, options: [], context_refs: [] } | null,   // present iff Waiting on an answer
  labels:     { /* opaque string map: ticket, phase, box, capabilities.mode, … */ },
  last_event: "2026-07-20T…Z",       // absence-of-heartbeat is itself a signal
  transport:  tmux | bridge | local, // where the PTY actually lives (render hint only)
}
```

- **`state` is the closed generic vocabulary.** The UCSOS detector's `AT_MENU` maps to
  `Waiting` + a `question` payload; `RATE_LIMITED`/api-error maps to `Error` with detail in `labels`.
  Error ≠ Idle is first-class (the dead-vs-idle blindness dies here).
- **`labels` is the opaque bag** — the adapter puts `ticket: 1696`, `phase: implement`,
  `capabilities.mode: bypass` in; Marley renders chips and lets *policy* (the manager) interpret them.
  Derived rules like *unsafe-to-dispatch = capabilities.mode ≠ bypass* are *adapter/manager* logic —
  Marley just refuses when the receipted verb comes back refused.
- **`question` is structured** — the fleet note's `halted-with-question` becomes a form (buttons +
  context), answered as a receipted typed reply. No digits pressed at screenshots.

## 4. The two MCP directions, mapped

| Surface | Direction | What flows |
|---|---|---|
| Fleet state (rail, snapshot, subscribe) | **consume** (Forge → Marley) | `seat_events` + work-state → the envelope, via the adapter |
| Dispatch (composer, delivery states) | **consume** (Marley → Forge mailbox) | deposit → `deposited → claimed → started`, rendered live |
| Session verbs (list/read/send/open/surface) | **expose** (manager → Marley) | the reliable-conduit verbs of §5 |
| Editor verbs (open file, goto, diff — up to typing) | **expose** (manager → Marley) | the MCP pillar's "typing in the IDE" bar; QA/review use |
| Browser verbs (navigate/read/screenshot/act) | **expose** (agent → Marley, Phase E) | the CDP→MCP family of §8 |
| Structured interrupts (question-as-form) | **both** | event in (consume) → rendered form → answer verb (expose) |

One consequence worth naming: because the manager drives Marley **over MCP**, the manager is
**location-independent** — hosted inside Marley (§9), in iTerm, or on a fleet box. Marley doesn't care.
Same agnosticism, applied to the orchestrator itself.

## 5. The reliable conduit — dispatch *through* Marley

The evidence night's failures were all mechanism failures (fire-and-forget keystrokes, screen
addressing, screenshot inference). The cure: the manager stops touching raw tmux and acts through
Marley's **receipted verbs** — each one kills a named failure class:

| Manager intent (policy) | Marley verb (mechanism) | Kills |
|---|---|---|
| "give dev-2 this brief" | `session.send(id, text)` — receipted; Enter as a separate write (the bridge's pre-solved lesson); refuses when the declared state says a dialog is up | swallowed-Enter · Shift-Tab-pokes-dialog |
| "is it done?" | *no verb* — subscribe to declared state (§6) | dead-vs-idle screenshot ambiguity |
| "show chad this session" | `session.surface_to_human(id)` — open/focus a native pane (or hand off to iTerm attach) | chad's ask verbatim: *"opening new sessions so the user can view"* |
| "what did it output?" | `session.read(id, range)` — scrollback via `capture-pane -p -S -` under the hood (the dead-pane rule) | tombstone banners, lost evidence |
| "start a seat on dev-3" | `session.open(profile)` — capability-declared from birth | bare-`claude` dead-stalls at first approval |
| "review this diff" | editor verbs — open file/goto/diff in the real editor | digit-pressed-at-a-screenshot review |

Lineage: this verb family **is** [brain-agent-session-supervision](../planning/intake/brain-agent-session-supervision.md)'s
`local_control`-shaped protocol *with the `session.read` delta built in from day one* ("one protocol,
three callers" — brain, manager, human tooling). The fleet work doesn't invent a second protocol; it
promotes that intake's seam to the fleet scale.

**The two planes, kept distinct:**
- **Dispatch plane** (content): brief text flows as **mailbox data** (deposit → claim → respond — the
  half that never failed; it stays the ledger). The keystroke component shrinks to a receipted *nudge*
  where a seat needs waking, via `session.send` — verified submission, not fire-and-forget.
- **Status plane** (truth): seat hook → `seat_events` → Forge → subscribers (§6). **No screen-scraping
  on any primary path**; the tick detector's screenshot read demotes to the wedge-detector fallback.

## 6. Push — MCP-native subscription (DECIDED)

**Decided this conversation: Marley never opens a Postgres connection.** The agnosticism constraint
settles the fork — a raw `LISTEN` would bake Forge's schema + DB credentials into Marley.

- **Forge exposes the feed as subscribable MCP resources** (e.g. `fleet://seats`, `fleet://events`).
  The forge server is already Streamable-HTTP (`.mcp.json` → `http://127.0.0.1:8080/mcp/forge`), so
  server→client notifications are available on the standing connection. Postgres `LISTEN/NOTIFY` stays
  **Forge-internal** — the trigger that fires `notifications/resources/updated` outward.
- **One contract, N consumers**: Marley, the manager (Monitor/wake-on-write), dashboards, audit — all
  read the same resource. No consumer is special.
- **Catch-up is free because `seat_events` is a durable table**: on (re)connect, `resources/read` from
  the last seen cursor → replay → re-subscribe. This *answers* detached-sessions.md's open
  "replay or start-from-now?" question: **state replays from cursor; PTY scrollback replays from
  `capture-pane -S -`** — two different stores, two different answers, both already durable.
- **Marley down ≠ fleet down, honestly**: Marley is just one MCP client. Kill it — the manager (another
  client) keeps orchestrating, tmux keeps running, and Marley rejoins mid-stream from its cursor. The
  degraded mode *is* today's workflow; that resilience is a feature, kept by construction.

## 7. The crate map — new vs grown vs prerequisite

| Crate | Status | Carries |
|---|---|---|
| **`marley_fleet`** | **SHIPPED (#367)** — pure (cov/MSI 100, no gpui, no transport) | The `Session` envelope + `FleetSnapshot`; the reducer (`events → snapshot`); the delivery-state machine (`deposited→claimed→started`); staleness/attention (absence-of-heartbeat); the verb *types*. Simultaneously the UI model, the test surface, and the MCP tool schema — one seam, three consumers (the constitution's pure-seam doctrine cashing out). **v1 hardening (#367):** timestamps are `u64` epoch-millis (`ts_ms`/`last_event_ms`; RFC3339 parsing belongs to the adapter, §6); `question` is a flat `Option<Question>` with a reducer-enforced ONE-directional invariant (`Some ⇒ Waiting`); **`transport: Option<Transport>`** — a §3 divergence forced by honest auto-vivified placeholders (an unknown-id event never drops a live seat). **D-OPEN settled:** `session.read` range is line-oriented `Tail{lines}\|Lines{start,end}` (byte offsets rejected — `capture-pane` is line-oriented); NO default staleness const (the threshold stays caller-injected, owned by #369's rail). |
| **`marley_forge_client`** | **fleet-subscription SHIPPED (#368) + integration-tested + self-healing (#372/#373)** — live wire pending L0 | **Adapter #1.** The MCP subscription + cursor/replay; the projection `seat_events`/work-state → envelope. *The only crate allowed to know Forge's vocabulary.* **Landed (#368):** the pure `fleet` module — SSE multi-event framing, the `initialize`/`initialized`/`subscribe`/`read`/`listen` builders + response parsers, the `FleetSync` orchestration model, and the projection; the masked standing-SSE pump lives in `adapter.rs`. **The proposed v1 wire contract** (the fake-feed fixtures ARE the contract for Layer 0): a single cursor'd durable log `fleet://events?since=<cursor>` whose `resources/read` returns a `FleetPage { cursor, events: [SeatEventRow] }`; each `SeatEventRow` is a self-contained descriptor + trigger `event_type` with `ts_ms: u64` epoch-millis (R3, no date dep). **Projection realizations (R1/R2):** every descriptor-bearing kind → `Upsert` (state + opaque labels together — `StateChange` carries no labels); `session-end`+error → `Upsert(Error)` (a crashed seat never launders to `Done` — the #368 F1 fix); `halted-with-question` → `[Upsert(Waiting), QuestionRaised]`; an unknown kind/state → a `Heartbeat` degrade. Push = MCP subscription, never Postgres. **#372** put the masked pump under a real-socket fixture-forge integration harness (typed `SubscriptionExit` signals, the §12.1 go-live runbook); **#373** made it self-healing — an until-stopped jittered-backoff reconnect loop (pure `backoff` seam) that carries the snapshot forward across a bounce so a delta feed loses nothing. Live-wire verification vs real `seat_events` is a named L0-day follow-up. |
| **`marley_mcp`** | **L1 SHIPPED (#370)** — pure core cov/MSI 100 + masked `std::net` transport (unmasked in the fork: 19 loopback tests over a real socket kill every transport mutant, #443) | The **expose** side: one hand-rolled serde-only server (no rmcp), a `(family, verb)` tool registry (L1: `fleet` read · `session` write; `editor`/`browser` additive via an exhaustive `match Family`), deny-by-default permission tiers (read loose / write per-class grant; marley_mcp OWNS the `GrantTable` #371 fills), the `fleet.snapshot` tool + subscribable `fleet://snapshot` resource (#367 types ARE the schema), and the ONE gated write `session.surface_to_human` (loopback-only, `Origin`+bearer pre-dispatch, bearer never logged). Proven headlessly by an in-memory fake-transport suite + the surface-effect drive; the live socket was the masked shim until the fork's loopback tests (#443). **C0 (#491):** Marley starts it at startup (`marley_workbench::mcp`); wire names are `family_verb`; the family it lists is `terminal`, read tools the app answers through deferred calls; fleet and session stay unlisted until C1. |
| **Session transport trait** | NEW seam, small | One trait, three impls: **tmux** (argv: `list-sessions`/`capture-pane -S -`/`send-keys`; the `-CC` client much later), **bridge** (`bridge-session-*`), **local PTY**. Renderer ⊥ transport is already ratified; `marley_fleet` never sees which. |
| **`marley_agent` inversion** | PREREQUISITE refactor | `AgentRun`: "child I spawned" → **"handle to a session that outlives me"** ([detached-sessions.md](./detached-sessions.md)). Required for Marley-as-conduit; largely independent → can run in parallel with Layer 1. |
| **`marley_browser`** | Phase E | CDP client + screencast decode + input-forward (pure protocol seam) + the CDP→MCP family (§8). |
| Settings | grows | `[[mcp.servers]]` (the `[[lsp.servers]]` idiom) + **per-project orchestration config: the project's brain MCP endpoint** *(its `web_url` twin shipped dead at #371 and was deleted at #410)* — the project-based wiring of §8. |

## 8. The Forge pane — a project URL in the embedded browser *(RETIRED at #410 — see the 2026-08-09 amendment banner)*

chad (2026-07-20): *"the UI component that lives in Marley about forge will be project based and more
than likely a URL we can view. Basically we will have a snapshot of what forge is doing via a remote
URL — open tickets, knowledge etc. This should be the built-in browser interface so Marley can peer
into it."*

- **The Forge pane = the embedded Chromium at the active project's Forge web URL.** Per-project config
  (§7 settings): each project names its brain's MCP endpoint *and* its web URL — another face of
  agnosticism (a non-Forge project points the pane anywhere). "Snapshot" is a **live window**, not a
  static capture — the mission-control hypermedia surface (SSE fragments/signals) is exactly what
  renders there. This *is* the convergence the mission-control intake predicted: **one HTML dialect,
  two transports** — the remote HTTPS/SSE URL in this pane, the `marley://` custom scheme for
  locally-generated surfaces. Both are just URLs to the same embedded Chromium.
- **The engine fork is already settled at the pillar level** ([embedded-agent-browser-chromium-cdp](../planning/intake/embedded-agent-browser-chromium-cdp.md),
  chad 2026-07-09): **real Chromium driven over CDP** — WKWebView was *rejected* (no CDP on macOS; a
  DIY inject-JS imitation can't meet "understands what it's looking at"), and the ~150MB is accepted.
  The remaining sub-fork (headless-Chromium screencast → gpui texture, vs CEF child view) is decided by
  the pillar's planned **spike**, not here.
- **The browser becomes one more MCP tool family** — CDP wrapped as `browser.navigate` / `read_page`
  (AXTree/DOMSnapshot — the page as *data*) / `get_text` / `screenshot` / `click` / `type` /
  `read_network` / `read_console`. Via CDP the agent sees **more** than the human does (DOM + network +
  console, not just pixels). One CDP connection both renders the pane and gives the agent its eyes —
  the pillar's "one connection does BOTH" property, preserved.
- **The discipline: don't scrape what you can query.** Structured Forge data (tickets, knowledge, gate
  results) → the **forge MCP tools**, always. The browser is for the *human's rich view* and for the
  **any-web escape hatch** — no-API pages, third-party dashboards, "show me what the user sees" parity
  checks. An agent DOM-scraping Forge for a ticket number is a smell; the typed tool exists.

## 9. The hosted agent — Marley's own Claude/Codex seat

chad (2026-07-20): *"Marley would have its own claude or codex session itself that can use Marley for
interacting and first class access to see what's going on in the browser."* **Possible — and it's
assembly, not invention:**

1. `marley_agent` **already launches agent CLIs as sessions** — that's the shipped M2–M9 surface.
2. Point one such session's MCP client at **Marley's own `marley_mcp` server over loopback**.
3. Grant it tool families by permission tier: fleet-read, session verbs, editor verbs, the browser
   family.

That session is **the manager seat living inside Marley**: MCP hands on Marley, MCP data from Forge,
CDP eyes on the browser. It resolves the "where does the loop live?" fork cleanly — **Marley hosts the
agent (mechanism); the agent's prompt runs the loop (policy). Nothing baked**: the provider
(Claude/Codex/other) is a configured seat, and because it drives Marley over MCP it could equally run
outside Marley (§4's location-independence).

**Proof by existence**: this exact shape already runs daily *outside* Marley — a Claude session driving
a real Chrome over CDP tools while working a repo with editor/terminal tools. Marley internalizes that:
embedded Chromium instead of external Chrome, `marley_mcp` instead of the CLI harness. Same agent, same
protocol shapes, one process.

**License posture**: the hosted agent stays a **separate program speaking MCP across the seam** — the
same AGPL §13 boundary as Phase D's brain. Hosting a seat ≠ linking a brain.

## 10. Security & permissions (day-one, not bolted on)

The hosted agent + expose-side MCP is the most capable actor in the system; it gets the pillar's
permission model from the first slice:

- **Tiers**: read tools (fleet state, blocks, diagnostics, page-read) = loose. **Write tools**
  (`session.send`, editor typing, `browser.navigate`/`click`/`type`, take-over, merge) = explicit
  scope grants, Claude-Code-style approve/allowlist, per server and per tool class. **Wired live
  (#370→#374):** the deny-by-default `GrantTable` #370 enforces is now filled by the operator's
  `[mcp.expose]` settings table (`allow_write` tool classes → the grant), so a configured
  `session.surface_to_human` passes while everything unlisted stays denied; an absent/malformed table
  fails closed to the empty default.
- **Browser domain allowlist**: scoped to the project's Forge URL + explicitly allowed hosts. No
  arbitrary navigation (that's the exfiltration channel), no open `Runtime.evaluate`-to-anywhere —
  the browser pillar's highest-priority design item, inherited verbatim.
- **Auth is two credentials, reconciled**: `marley_forge_client`'s MCP bearer (never-logged lesson
  stands) *and* the embedded browser's own web session to the remote Forge URL. Mission-control's
  Tailscale-first + per-device identity posture is the frame for the latter. **Hardened (#375):** the
  expose server's bearer is a per-boot OS-CSPRNG value (refuse-to-start on entropy failure, no weak
  fallback), the discovery file that carries it is 0600 + removed on shutdown, and the MCP
  `Mcp-Session-Id` lifecycle (assign/echo/400/404/DELETE, bounded reject-new registry with
  stream-drop reclamation **+ a 30-minute idle TTL, #379** — the sweep runs before the decision, so
  an expired id is absent and answers with the same 404 as an unknown one; fresh sessions are never
  evicted) is enforced — pre-dispatch order cap → Origin → bearer → session(sweep→decide→touch) →
  dispatch.
- **Observation scopes**: the manager-created-only rule stands (TICKET-043 — *"the operator may not
  want the system seeing everything they do"*). A human's ad-hoc shell is invisible to agents by
  construction; per-pane observation is opt-in.
- **Per-pane trust posture** (fleet note §9): who may dispatch, who may merge, who may take over a
  terminal — mirrored from the bridge's admin/capability gating.

## 11. Forks — decided here vs still open

**Decided (2026-07-20 conversation):**
1. **Push = MCP resource subscription.** Never raw PG `LISTEN` in Marley; no bespoke SSE sidecar.
   Marley stays DB-credential-free (§6).
2. **The loop lives in the manager seat.** Marley = mechanism/shell; policy stays in the agent. (A
   later Phase-D migration of the loop into Marley-the-process would change *the caller*, not the
   seams — every verb is already MCP.)
3. **Agnostic by envelope.** The generic `Session` model + adapter rule (§2–3); UCSOS via
   `marley_forge_client` only; design generic, ship one adapter.
4. **Browser engine = Chromium/CDP** (re-affirmed; settled at the pillar 2026-07-09). First-class
   agent access is the bar; WKWebView can't meet it.
5. **`marley_agent` ownership inversion is the prerequisite refactor** — seats must outlive Marley
   before Marley can be a conduit onto them.

**Open (deliberately):**
- Multiplexer substrate: tmux vs zellij vs pluggable — decide when passthrough (Layer 3) is built.
- Screencast-vs-CEF sub-fork — decided by the Phase-E spike.
- Envelope v1 hardening (field-level) — at Layer-1 promotion; §3 is the proposal.
- Event retention/pruning; auth topology (mission-control's a/b); the attach-UX bar for bridge-owned
  seats (must beat iTerm-attached tmux before any migration is proposed).

## 12. Sequencing — layers, and where the first sprint cuts

Unchanged stance: **NOT scheduled; the IDE milestones continue first.** Ready when called:

- **Layer 0 — not Marley (ucsosv2, now):** the seat-emitter hook + `seat_events` on Forge — the
  fleet note's pre-Marley deliverable. **Gates everything below.**
- **Layer 1 — Marley read-only (the natural `/spec` sprint, ~5 tickets):**
  ① the `Session` envelope + `marley_fleet` reducer (pure — **SHIPPED #367**) → ② `marley_forge_client`
  subscription + adapter projection (**SHIPPED #368**) → ③ the **fleet rail** (read-only render in the
  revived right dock; state chips + render-only question-cards + staleness dimming — **SHIPPED #369**) →
  ④ a read-only `marley_mcp` slice (fleet snapshot/subscribe) + the ONE gated write,
  `session.surface_to_human` (**SHIPPED #370** — hand-rolled serde-only server, deny-by-default tiers,
  loopback-only) → ⑤ `[[mcp.servers]]` + per-project orchestration settings round-trip (**SHIPPED #371** —
  `McpServerConfig`+`grants()` in marley_mcp closing the S1 loop; `[[projects.orchestration]]` root-keyed;
  pure, cov/MSI 100). **Layer 1 COMPLETE (#367–#371).**
  All mechanism, one adapter, zero passthrough slog — and it forces the envelope decision while it's
  cheap. *Insight before control* (the tmux intake's promotion rule, honored).
- **Layer 1 consolidation — M23.5 (`/work 372–375`):** hardens the wire before it goes live —
  ① #372 live-wire READINESS (a real-socket integration harness driving the masked pump + typed
  `SubscriptionExit` signals + the §12.1 runbook) → ② #373 auto-reconnect + backoff → ③ #374 wiring the
  `[[mcp.expose]]` grants into the running server → ④ #375 security/protocol hardening (CSPRNG bearer,
  discovery-file perms, per-session `Mcp-Session-Id`).
- **Layer 2 — gated writes:** **`session.send` + the dispatch composer with delivery states — ✅
  SHIPPED (#378, M24 ③)**: every seat card offers an inline compose draft; a confirm dispatches ONE
  receipted `session.send` whose arguments are the shipped `SendRequest {id, text}` verbatim (the
  FROZEN v1 contract, fixture-recorded in livewire `t378_send_call_records_the_frozen_contract`);
  delivery renders as a chip advanced ONLY by the `DeliveryState` machine — receipt-Ok seeds
  `deposited`, and the PROPOSED echo kinds `dispatch-claimed` / `dispatch-started` (our symmetric
  addition beside fleet-control-plane §5's enum) advance it monotonically, projected adapter-side
  into a per-seat max-join queue on the subscription handle (echoes observed while the receipt is
  in flight BUFFER in the record — a terminal `dispatch-started` must never be dropped); a
  failed/refused send preserves the brief for a prefilled retry. Per-dispatch correlation ids stay
  a later contract rev (a same-seat echo from ANOTHER actor's dispatch advances the chip — the
  acknowledged v1 window) ·
  **question-forms answering — ✅ SHIPPED (#377, M24 ②)**: the Waiting seat's option chips
  dispatch exactly one receipted `session.answer` `tools/call` — the FROZEN proposed v1 contract is
  `arguments: {session_id, choice, prompt}` (choice = the picked option string verbatim; prompt =
  the answered question's identity so the brain REFUSES stale answers; fixture-recorded in
  `marley_forge_client/tests/livewire.rs` `t377_answer_call_records_the_frozen_contract` — the L0
  handoff literal). A local Pending/Answered/Failed overlay renders until the seat's own event
  stream flips it (the reducer stays the sole truth producer); a refusal/failure surfaces its
  clamped reason and re-arms. L0 follow-up recorded: a question NONCE in the envelope (+ echoed in
  `session.answer`) would close the two residual races a snapshot-level guard cannot (a
  byte-identical re-raised question; a sub-pump-tick Waiting→Working→Waiting round-trip) ·
  editor verbs · the permission tiers in anger.
- **Layer 3 — passthrough:** the native PTY render per seat (tmux `-CC` client / bridge PTY / local),
  who's-driving indicator, human takeover/hand-back. The one genuinely hard build; last, once 1–2
  prove the contract.
- **Phase E — the browser + the hosted agent:** the Chromium spike → the Forge-URL pane → the
  CDP→MCP family → Marley's own seat (§9). Depends on `marley_mcp` existing (Layer 1–2) — which is
  why the Layer-1 server must be shaped for tool-family growth from day one.

### 12.1 Going live (L0-day) runbook

When Layer 0 ships (the ucsosv2 seat-emitter hook + `seat_events` + the `fleet://events` cursor'd
resource on the real Forge server), the Marley **client/protocol layer is already live-wire ready** —
#372 landed the fixture-proven pump, the typed disconnect/`SessionExpired` signals, and this checklist.
Two Marley-side preconditions remain (app-side wiring + #373 reconnect, below); once those land, L0-day
itself is *config + verify*. Run it in order:

**Preconditions — BOTH SHIPPED; the runbook is now config + verify only:**
1. **App-side wiring — ✅ SHIPPED (#376, M24 ①).** The boot starts `FleetSubscription` for the
   restored ACTIVE project's `[[projects.orchestration]].brain_endpoint` (the pure
   `fleet_live::subscription_target` gate + `endpoint_for_brain` — the same loopback wall, bearer
   carried only when the `.mcp.json` forge url matches), snapshots drain through the pump into the
   rail, the right-dock header shows the TYPED connection state (`Fleet · live` /
   `Fleet · reconnecting`; plain `Fleet` when unconfigured — byte-identical to #369), the cursor
   persists under a per-endpoint home (`fleet-<sanitized-url>-<fnv32>/fleet-cursor`), and quit
   teardown rides the RootView drop. The `fleet-demo-feed` verb stays for unconfigured roots and is
   gated off while a subscription exists. **#384 (the #376-F8 cash-in):** the start-gate's outcomes
   are CLASSIFIED at the same boot site — `fleet_live::classify_fleet_setup` mirrors the gate's
   abort order into `FleetSetup` (Unconfigured / Misconfigured{arm, clamped reason} / Configured),
   and a configured-but-broken root titles **`Fleet · misconfigured`** (winning every wire state —
   arm (c)'s empty-bearer subscription still starts, but a config fault outranks wire truth) with
   ONE muted reason caption atop the rail body: arm (b) `brain_endpoint rejected (loopback http://
   only): <url>`, arm (c) the shared `fleet_rail::NO_FORGE_CLIENT_REASON` in-card literal, plus the
   structurally-unreachable config-dir micro-arm (named per the no-silent-abort contract). Reasons
   are clamped at construction (`Misconfig::new` → `clamp_card_text`, which since #384 also strips
   bidi controls); an unconfigured root's bare `Fleet` stays byte-identical.
2. **Reconnect (#373) — ✅ SHIPPED.** The until-stopped jittered-backoff loop carries the snapshot
   across bounces; a transient Forge bounce no longer ends live updates.

**Configure:**
3. Add the project's `[[projects.orchestration]]` row with `brain_endpoint = "http://127.0.0.1:<port>/mcp/forge"`
   (loopback only — the endpoint refuses to construct for a non-loopback host, `lib.rs` `is_loopback_authority`).

**Verify against the real ucsosv2 (the original #372 (a)–(e), now a tiny L0-day ticket):**
- **(a)** the MCP lifecycle (`initialize` → `notifications/initialized` → `resources/read` →
  `resources/subscribe`) is accepted by the real server (fixture-proven order = `tests/livewire.rs`
  `req001`);
- **(b)** the `Mcp-Session-Id` echo is honored, and a **404** on an expired session surfaces as
  `SubscriptionExit::SessionExpired` (the client re-initializes once #373 lands);
- **(c)** the real `seat_events` rows deserialize into `SeatEventRow` — reconcile the proposed v1 shape
  (`SeatEventRow{event_type,session_id,ts_ms,state,title,transport,labels,payload}`, `FleetPage{cursor,
  events}`, `fleet://events?since=<cursor>`) with L0's actual emitter output; adjust `project_row`'s table
  if the real event vocabulary differs;
- **(d)** the durable cursor (`<config_dir>/fleet-cursor`) persists and a reconnect replays from it with
  no gaps or duplicates (the reducer's idempotent max-join guarantees no double-count — fixture-proven =
  `req007`);
- **(e)** the masked pump (`run_once`/`refresh`/`open_listen_stream`) works over the real socket end to
  end (fixture-proven for a scripted peer; the real-peer run is this checklist).

## 13. Provenance & ties

`[Marley-original]` design over open protocols: MCP (open spec, MIT SDKs — the LSP-of-M20 posture),
tmux over argv (no lineage), CDP/Chromium/CEF (BSD). No Warp/Zed source involvement; §20 clean.
Sources: [fleet-control-plane.md](./fleet-control-plane.md) (the evidence night + end-goal) · the
2026-07-20 owner conversation (this doc's decisions §11) · the four pillar intakes (header). The
forge ADs of record: `AD-claude-tmux-grade-detached-sessions-001`,
`AD-claude-mission-control-hypermedia-surface-001`, `AD-claude-brain-agent-session-supervision-001`.

**One-line summary:** *Forge is the brain, the manager is the judgment, the seats are the hands —
and Marley is the reliable, project-agnostic shell they all act through: MCP in both directions,
a generic session envelope, receipted verbs, a live browser eye, and a home for the manager itself.*
