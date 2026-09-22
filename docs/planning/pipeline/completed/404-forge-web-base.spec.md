---
pipeline_id: 9cd72b10-31cc-42fd-a8bb-1aa4b2805255
ticket: forge#404 (e84add90-0452-4bfa-9772-4ca04a54d5ae) · local docs/planning/tickets/open/TICKET-404-forge-web-base.md
aar_id: 62602a55-22db-42ed-9036-2ef6400646d1
status: Phase 5 — Complete PASS
title: forge_web_base — the .mcp.json → Forge web-origin derivation seam (the slice-3 input, pinned)
type: feature
milestone: M29
references:
  - docs/marley_architecture/embedded-browser-model.md
  - crates/marley_app/src/mcp_config.rs
  - crates/marley_forge_client/src/lib.rs
  - crates/marley_app/src/fleet_live.rs
  - crates/marley_app/src/app.rs
---

## Title
The embedded-browser train's **slice-3 input, pinned ahead of the pane**: a PURE derivation of the
Forge **web-UI base** from the SAME `.mcp.json` the #381 active-root resolution already feeds — **one
config, THREE consumers** (embedded-browser-model.md Q3). `.mcp.json` yields the MCP *JSON-RPC*
endpoint (`mcpServers.forge.url` = `…/mcp`, parsed by `forge_endpoint_from`,
marley_forge_client/src/lib.rs:124-141); the web base is that endpoint's **ORIGIN** — scheme + host +
port, path dropped — per the WHATWG URL "origin" definition. The two shipped consumers already share
the one resolution (verified today; the design doc's M26 lines app.rs:2207/2220 have drifted): the
resolved string is built once at app.rs:2400-2406 (`mcp_config::mcp_json_path` at :2403 —
active-root-first, never launch cwd), then feeds **(1)** the forge MCP client (app.rs:2407-2410,
`forge_endpoint_from` → `ForgeClient::new`) and **(2)** the fleet brain-endpoint bearer decision
(app.rs:2426-2428, `endpoint_for_brain`). This ticket adds consumer **(3)** as a pure fn + total unit
surface — sketch shape (Phase 2 settles the exact signature + home):

```rust
/// The Forge web-UI base for a resolved `.mcp.json` string: the MCP endpoint's WHATWG
/// origin (scheme+host+port, path dropped), or `None` on every unresolved/refused arm.
pub fn forge_web_base(mcp_json: &str) -> Option<String /* or a WebOrigin newtype */>
```

Unresolved on ANY arm — no `.mcp.json`, unreadable, unparseable, no forge entry, wall-refused url —
→ `None` → **no Browser content** (the section's empty rail home, never a broken page): the sibling
of #384's misconfigured-state discipline (`fleet_live::classify_fleet_setup`, fleet_live.rs:98-127)
but **presentation-free this slice** — `Option`, no reason string, no render. Auth is the Q3 D3
inheritance, not an invention: the loopback wall stands; the MCP bearer NEVER enters the URL or page
JS (#370/#375 lineage; `ForgeEndpoint.bearer` is private + Debug-redacted `"***"`, lib.rs:38-49).
Consumer wiring (the pane loading the origin) is #405's; this slice is the input, at cov/MSI 100.

## Scope
### In
- **The pure derivation fn** — the sketch above; recommended home: `marley_forge_client` alongside
  `forge_endpoint_from` (the module that already owns the `.mcp.json` parse + the loopback wall;
  `mcp_config.rs` is charter-bound "pure over PATHS — never reads, parses, logs, or holds file
  contents", mcp_config.rs:11-12, so the derivation cannot live there). Phase 2 settles fn name /
  return type (String vs a `WebOrigin` newtype) / home per D-OPEN-INPUT-SEAM + D-OPEN-URL-CRATE.
- **The endpoint→origin step** per WHATWG origin: scheme + host + port kept, the endpoint's path
  (`/mcp`, `/mcp/forge`, …) dropped; default-port normalization per D-OPEN-DEFAULT-PORT.
- **The unit surface, total**: table-driven happy paths (ports, bracketed IPv6, `localhost`, nested
  paths, no path), one named unit per `None` arm, the no-bearer negative, hostile-input totality
  (§14 — no panic, no unwrap on config-derived input).
- **The composed keyed-on-active-root pin**: derivation consumes the SAME `mcp_json_path` resolution
  (#381) — a composed unit mirrors `both_present_active_root_wins` (mcp_config.rs:81-92); zero new
  resolution logic.

### Out (explicitly deferred)
- **Consumer wiring — #405** (sibling M29 ticket): the wry pane loading the derived origin, the
  boot/restore call site, pinned-origin nav enforcement. This slice ships NO caller.
- **Any UI / presentation.** The no-config empty state ships already: the #385 Browser rail section
  + #403's placeholder pane (sibling M29 ticket, train slice 2) render the empty rail home; this
  seam returns `Option` and draws nothing. No misconfigured-reason caption (the #384 sibling stays
  fleet-only until evidence demands a Browser one).
- **The explicit path/port mapping escape hatch** (web UI on a different path/port than the MCP
  origin) — deferred unless evidence demands it; Phase 2 carries the validate-against-the-REAL-Forge
  -instance step (the operator checks what origin the web UI actually serves on) and only a real
  mismatch revives the hatch.
- **Bearer-auth bridging.** Q3 D3: the web UI authenticates itself (session cookie / its own login);
  if it instead expects the MCP bearer, the pane renders unauthenticated — a gap closed LATER, never
  by putting the bearer in the URL. Assumption recorded; validated at design.

## Reference (§20)
**N/A — Marley-specific** (a config→origin derivation seam has no Warp/Zed behavior analog; Warp's
04-agent-ai-mcp map covers agent/MCP wiring, not a web-base derivation; no Zed analog — checked at
the behavior-map level only). The one **published anchor**: the **WHATWG URL Standard's "origin"**
definition + its ASCII serialization — the correct semantic frame for "scheme+host+port, path
dropped", including its default-port rule (a port equal to the scheme's default is elided in the
serialization). Adopted at the published-spec level; D-OPEN-DEFAULT-PORT carries the normalization
question into design.

### Prior art
1. **Behavior maps — checked, no owner.** docs/warp_architecture/ (04-agent-ai-mcp.md — MCP/agent
   wiring, no web-base seam) and docs/zed_architecture/ carry nothing shaped like a config-derived
   browser origin. The governing doc is Marley's own: embedded-browser-model.md **Q3** (this ticket
   executes it verbatim).
2. **Published material.** The WHATWG URL "origin" definition + ASCII serialization (above) — the
   anchor for D2 and the default-port question. No other protocol surface.
3. **Our permissive deps — the yield leg, checked honestly (the #339-regex pattern test).** The
   **`url` crate (2.5.8) IS in Cargo.lock** (:6396-6398) — transitively, via the gpui/zed stack
   (`gpui_http_client`, `zed-reqwest`, `ashpd`, `zvariant`) + `git2`; it is a **direct dependency of
   no workspace crate** (no Cargo.toml names it; `marley_forge_client`'s deps are serde/serde_json/
   marley_fleet only). Today's URL handling is **hand-rolled**: `split_url` (lib.rs:182-188 —
   `http://`-only prefix strip + first-`/` split) + `is_loopback_authority` (lib.rs:168-178 — IPv6
   brackets + `rsplit_once(':')` + `IpAddr::is_loopback`). `Url::origin().ascii_serialization()` IS
   the WHATWG anchor made executable. In-house precedent for promoting an already-locked transitive
   dep: `unicode-width` (#277, marley_app/Cargo.toml — "already in the lock transitively;
   MIT/Apache"). → D-OPEN-URL-CRATE records the adopt-vs-reuse fork with ADOPT recommended; locking
   a hand-parse against an in-tree owner would be #339 repeating. In-house seam owners reused, not
   reinvented: `mcp_config.rs` owns WHICH file (path resolution, #381), `forge_endpoint_from` owns
   the parse + wall, `fleet_live.rs` owns misconfigured PRESENTATION (out of scope here).

## React-first (parity)
**N/A — no UI delta:** a pure derivation fn + units; no pixel, chrome, or affordance changes. The
Browser section's rail presentation is untouched (#403 owns the placeholder pane, #405 owns the live
pane); the no-config arm renders whatever already ships. The marley-web POC has no config-derivation
layer to mirror — parity re-engages at #405 where the pane becomes visible.

## Locked-In Decisions
- **D1 — same config, three consumers; keyed on the restored ACTIVE root.** The derivation consumes
  the ONE `.mcp.json` resolution #381 shipped — `mcp_config::mcp_json_path` (mcp_config.rs:28-40;
  active root wins, launch cwd is the missing-file-only fallback, per
  `PR-claude-boot-decisions-key-the-restored-active-root-001`) — exactly as the two shipped
  consumers do today: the forge MCP client (app.rs:2400-2410) and the fleet brain-endpoint decision
  (app.rs:2426-2428) share the single post-shell-restore resolved string (app.rs:2402-2406). No
  second read, no second resolution path, never the launch cwd. (Doc drift recorded: Q3 cites
  app.rs:2207/2220 — M26 lines; today's are above.)
- **D2 — origin-only derivation, per WHATWG origin.** Web base = the MCP endpoint's scheme + host +
  port; the endpoint's path is DROPPED (`http://127.0.0.1:8080/mcp` → `http://127.0.0.1:8080`).
  No path/port mapping in v1 (the escape hatch stays out per Scope Out).
- **D3 — the bearer never enters the URL or page JS** (#370/#375 bearer-never-logged lineage;
  Q3 D3). Structural today: `ForgeEndpoint`'s fields are private with a Debug-redacted bearer
  (lib.rs:38-49); the derived value carries ONLY the origin — never the endpoint struct, never the
  bearer. Concrete negative test: derive over a `.mcp.json` whose `Authorization` holds a sentinel
  token → the derived string provably contains no sentinel bytes (REQ-004).
- **D4 — unresolved → `None` → no Browser content.** Every arm — no candidate file
  (`mcp_json_path` → None), unreadable file, unparseable JSON, absent `mcpServers.forge`/`url`,
  wall-refused url — yields `None`, and `None` means the Browser section shows its empty rail home,
  never a broken page (Q3). The #384 sibling stance: `classify_fleet_setup` names reasons for the
  FLEET header (fleet_live.rs:98-127, arms :69-81, clamped bearer-free reason :41-48); THIS seam
  stays presentation-free — `Option` only, no reason string, no render this slice.
- **D5 — hostile-input totality** (§14). Garbage strings, missing scheme, userinfo in the URL
  (`http://user:pass@127.0.0.1/`), bracketed/malformed IPv6, absurd/out-of-range ports, empty and
  multi-KB inputs: the fn is TOTAL — `Some(valid origin)` or `None`, never a panic, no
  `unwrap`/`expect` on any config-derived path. (Today's wall already refuses userinfo forms
  incidentally — `is_loopback_authority` sees `user:pass@host` as a non-loopback authority — the
  units pin that behavior deliberately rather than inheriting it by accident.)

**D-OPEN — ALL THREE SETTLED at Phase 2 (2026-08-07; evidence + the real-instance probe in the
notes):** URL-CRATE → **ADOPT** `url = "2"` (in-lock 2.5.8, origin.rs read — ascii_serialization
implements WHATWG elision; `is_tuple()` guard required against the opaque-`"null"` poison).
INPUT-SEAM → **ride `forge_endpoint_from`** (one parse, one wall; bearer-less → `None` confirmed
as one-config-coherence). DEFAULT-PORT → **WHATWG elision** (free under ADOPT; #405 must compare
origins, not strings). Signature settled: `pub fn forge_web_base(mcp_json: &str) -> Option<String>`
in marley_forge_client. **Real-instance finding:** no web UI exists at the MCP origin today
(unauthed `/` → 401 JSON; authed `/`+six candidates → 404) — seam unblocked, hatch stays out,
carry to #405/#406. Original fork records below.
- **D-OPEN-URL-CRATE — adopt `url` vs extend the hand-parse.** Leg-3 truth: `url` 2.5.8 is already
  in Cargo.lock (transitive; license-vetted by gate:8 since it ships in the graph), direct dep of
  nothing. **Recommend ADOPT** as a direct dep of the seam's home crate:
  `Url::parse(endpoint).origin().ascii_serialization()` is the WHATWG anchor implemented — origin
  extraction, default-port elision, host canonicalization, hostile-input totality — for zero new
  supply-chain weight (the #277 `unicode-width` promotion precedent; the #339 rule: a hand-parse
  must not be locked against an in-tree owner). The recorded alternative: reuse `split_url`
  post-wall (parser-coherence — the SAME parser the wall validated with; the surviving input space
  is already `http://` + loopback) — viable only if Phase 2 proves it equivalent INCLUDING
  default-port normalization; either way the wall keeps gating construction (the derivation never
  widens what the wall admits).
- **D-OPEN-INPUT-SEAM — derive from the CONSTRUCTED endpoint vs re-parse the raw JSON.**
  **Recommend: ride `forge_endpoint_from` → `ForgeEndpoint::url()`** (lib.rs:52-57): one parse, one
  wall, one truth — the loopback wall (:131-134) is inherited **by construction** (non-loopback
  config → no client AND no web base — the three consumers agree), and the bearer stays behind the
  private field. Recorded consequence: `forge_endpoint_from` also requires `headers.Authorization`
  present (:135-139), so a url-only bearer-less config yields `None` for the web base too — accepted
  as one-config-coherence (that config builds no MCP client either; it IS misconfigured), Phase 2
  confirms. The loopback-wall question itself was checked and is **not genuinely open**: Q3 pins
  "the wall stands"; the wall's implementation rationale is bearer-egress
  (lib.rs:128-130 — MCP-endpoint-construction-specific, not a general egress wall), and the
  ride-the-endpoint shape makes its application to the browser URL structural rather than re-imposed.
- **D-OPEN-DEFAULT-PORT — `http://host:80/mcp` → `http://host` or `http://host:80`?**
  **Recommend: WHATWG ASCII serialization (elide the scheme-default port)** — the published anchor's
  own rule, free under adopt-`url`; browsers treat the two as the same origin, and #405's
  pinned-origin nav check must compare ORIGINS, not strings (record for #405). Live surface is tiny
  today: the wall is `http://`-only (split_url :183) and the sidecar binds OS-assigned high ports
  (marley_mcp transport.rs:99) — but the units pin the rule either way.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the resolved `.mcp.json` parses and `mcpServers.forge.url` passes the wall (`http://` + loopback), the system shall derive `Some(origin)` = the endpoint's scheme+host+port with the path dropped — `…/mcp`, nested `…/mcp/forge`, and path-less endpoints all map to the same origin. | table-driven units: `/mcp`, `/mcp/forge`, no path, explicit port, `localhost`, `127.0.0.1`, bracketed `[::1]` |
| REQ-002 | WHEN the active project root and the launch cwd both hold a `.mcp.json`, the derivation shall consume the ACTIVE root's file (the #381 pin — same input seam, never launch cwd). | composed unit over `mcp_json_path` + derivation mirroring `both_present_active_root_wins` (mcp_config.rs:81-92); zero new resolution logic to review |
| REQ-003 | WHEN any unresolved arm holds — no candidate file, unreadable file, unparseable JSON, absent `mcpServers.forge` entry/url, wall-refused url — the system shall return `None` (no error surface, no content). | one NAMED unit per arm (5 arms); exhaustive over the `forge_endpoint_from` failure modes |
| REQ-004 | WHEN the `.mcp.json` carries `headers.Authorization` with a sentinel bearer, the derived value shall contain no bearer material — the sentinel's bytes provably absent from the `Some(origin)` string. | negative unit: derive over a sentinel-bearing fixture → `!origin.contains(sentinel)`; the return type carries only the origin (D3 review) |
| REQ-005 | WHEN handed hostile input — garbage, missing scheme, userinfo, malformed/bracketed IPv6, absurd ports, empty or multi-KB strings — the system shall return without panic (`Some` valid origin or `None`), with no `unwrap` on any config-derived path. | hostile-input table units (property-ish sweep); §14 review at inspect |
| REQ-006 | WHEN the slice lands, the touched seam shall hold line coverage 100 and MSI 100. | `scripts/gates.sh --diff` exit code; `cargo mutants --list -f` over the actually-touched file(s), zero missed |

## Floors (constitution)
The whole ticket is a **pure seam at cov/MSI 100** — the derivation fn + every arm + the hostile
table sit in unit-reachable code. **NO masked surface expected**: consumer wiring is #405's, so no
app.rs touch is planned; IF Phase 2's home-module decision forces one (e.g. a boot-site accessor),
name it masked EXPLICITLY and re-run `cargo mutants --list -f` on the actual touched files —
app.rs's pump/boot shims are the skip-detach trap's home turf (5th strike
`BF-claude-skip-detach-pump-fleet-live-001`). Typed inputs; no `unwrap`/`expect` on config-derived
paths (§14). Bearer discipline preserved structurally: `ForgeEndpoint`'s private fields +
Debug-redacted bearer (lib.rs:38-49) stay the only bearer holder; the new seam never returns or logs
one (#370/#375).

## Phase Plan
- **P2 Design** — settle the three D-OPENs with evidence (read `url`'s origin serialization — ADOPT
  leg; prove or drop split_url equivalence; confirm the ride-the-endpoint input seam + the
  bearer-less-config consequence); **validate against the REAL Forge instance**: what origin does
  the web UI actually serve on (the operator checks the browser — the escape-hatch trigger), and
  does it authenticate itself (the Q3 D3 assumption)? Fix the exact signature, return type
  (String vs `WebOrigin` newtype), fn name, and home module; per-REQ test plan incl. the hostile
  table's exact rows.
- **P3 Implement** — the pure seam + its units in the chosen home; the composed active-root unit;
  no consumer wiring.
- **P3.5 Inspect** — adversarial critics incl. a hostile-input adversary (parser edge cases beyond
  the table: percent-encoding, mixed-case schemes, trailing dots, port `0`); the D3 bearer walk
  (can ANY path let bearer bytes into the derived value or a Debug/log?); §20 provenance
  (adopt-`url` is adoption, outside the wall).
- **P4 Validate** — write + RUN the units per REQ (named per-arm units, the negative, the hostile
  table); `scripts/gates.sh --diff` green; `cargo mutants --list -f` on the touched file(s) →
  MSI 100, zero missed.
- **P5 Complete** — CHANGELOG entry; tick embedded-browser-model.md Q3's "endpoint→web-base step
  (a slice-3 input, **not yet pinned**)" to **pinned** (+ the settled origin/default-port rule);
  AAR capture; archive; close #404.
