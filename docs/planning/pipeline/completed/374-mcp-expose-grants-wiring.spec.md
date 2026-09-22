---
pipeline_id: b819ed4a-8401-4355-b9f8-4b389b716f4f
ticket: forge#374 (1200f35c-e444-4c16-aa68-c1347269684b) · local docs/planning/tickets/open/TICKET-374-mcp-expose-grants-wiring.md
aar_id: f4bc1341-c090-48ff-9dbd-f3426cf958f6
status: Phase 5 — Complete PASS
title: marley_mcp — wire configured [mcp.expose] grants into start_mcp_server (S3 — #370/#371 follow-up)
type: feature
milestone: M23.5
references:
  - docs/marley_architecture/orchestration-shell.md §10 (permission tiers, deny-by-default)
  - docs/planning/pipeline/completed/371-mcp-servers-settings.notes.md (the S3 deferral + entry-selection fork)
  - crates/marley_mcp/src/config.rs:110-127 (transport() NoTransport at :117 · grants() :124-126)
  - crates/marley_mcp/src/permission.rs:12-26 (GrantTable · from_classes :21-25) + :49-58 (decide)
  - crates/marley_mcp/src/dispatch.rs:132 (the tools/call permission gate)
  - crates/marley_mcp/src/registry.rs:69-75 (session.surface_to_human → grant_class "session.write")
  - crates/marley_mcp/src/transport.rs:25-34 (ServerData.grants)
  - crates/marley_app/src/mcp_host.rs:23-45 (McpHost::start(salt, grants) :28)
  - crates/marley_app/src/app.rs:6577-6590 (start_mcp_server; GrantTable::default() :6589) + :8147 (the mcp-serve verb) + :191-192/:2106 (host None until the verb) + :2108 (boot-applied copy precedent)
  - crates/marley_app/src/settings.rs:86-98 (the #371 define_setting rows) + :216-218/:370-371/:412-413 (AppliedSettings wiring) + :487-504 (cfg(test) persist helpers) + :965-1028 (the tolerance-test shape)
  - crates/marley_settings/src/value.rs:14-16 (from_file_value → .ok(): malformed value → default)
---

## Title
**Close #371's S3 loop: route operator grant config into the LIVE expose server.** #371 shipped
`[[mcp.servers]]` with per-server `allow`/`allow_write` and the pure `grants() → GrantTable`
accessor; #370's loopback server runs and enforces `decide()` — but the `mcp-serve` verb still
starts it with the hardcoded `GrantTable::default()` (app.rs:6589, its own doc comment: *"Grants
default to empty (deny-by-default) until #371's settings land"*). Nothing routes settings into the
running permission check, so `session.surface_to_human` is undeniably denied for every operator.
This ticket adds the ONE missing hop — a singleton `[mcp.expose]` settings table (the
entry-selection convention #371 deferred), a minimal `ExposeConfig` in `marley_mcp`, and the
callsite swap — with an absent-table regression pin so an unconfigured Marley behaves byte-for-byte
as today. **Soft order: land BEFORE #375** (marley_mcp hardening pins the final server-start
surface; this ticket changes what flows into `McpHost::start`, so it lands first).

## Scope
### In
- **`ExposeConfig` in `marley_mcp`** (config.rs or a new expose.rs — Phase-2 file call; domain-crate
  rule either way): `{ #[serde(default)] allow: Vec<String>, #[serde(default)] allow_write:
  Vec<String> }`, `derive(Default)` (== the all-keys-absent decode — D2), plus a pure
  `grants(&self) -> GrantTable` that is EXACTLY the shipped semantics:
  `GrantTable::from_classes(allow_write)` with `allow` carried-not-consumed (D3). Exported from
  lib.rs beside `McpServerConfig` (lib.rs:26).
- **The `[mcp.expose]` setting** — `define_setting!(McpExpose: ExposeConfig =
  ExposeConfig::default(), "mcp.expose")` in `crates/marley_app/src/settings.rs` (the #371 rows,
  :86-98) + `AppliedSettings.mcp_expose` + `applied_from`/`applied_defaults` lines (:216-218 /
  :370-371 / :412-413) + a `#[cfg(test)] persist_mcp_expose` helper (the :487-504 posture:
  hand-edited only; the file IS the interface). First struct-valued (non-Vec, non-scalar) setting —
  mechanically supported: the manager is shape-agnostic over the blanket `SettingsValue`
  (manager.rs:76/:92-97 store any `to_file_value()`; value.rs:14-16 gives malformed → default).
- **The wire** — a boot-applied copy on `RootView` (the `language_servers: applied.
  language_servers.clone()` precedent, app.rs:2108); `start_mcp_server` (app.rs:6580) passes the
  ExposeConfig-built `GrantTable` instead of `GrantTable::default()` at :6589. `McpHost::start`
  (mcp_host.rs:28) and `ServerData.grants` (transport.rs:29) are UNCHANGED — the signatures already
  carry grants; only the value passed changes.
- **Tests** — the #371 settings tolerance shape (:965-1028) for `[mcp.expose]`; an end-to-end grant
  unit through `handle_message` (the dispatch.rs:298 test shape) proving allow-listed passes +
  unlisted stays denied; the absent-table byte-equality regression pin.

### Out (explicitly deferred)
- **`enabled` field** — serving is already verb-gated: the host is `None` until the explicit
  `mcp-serve` verb (app.rs:191-192, :2106, :8147 → start_mcp_server :6580); a flag would duplicate
  that gate and add a contradictory state (D2).
- **port/bind/listen fields** — loopback + OS-assigned port are fixed by #370 (transport.rs:82
  binds `127.0.0.1:0`); listen/discovery config is D-OPEN-LISTEN territory (#375-adjacent), not here.
- **Live re-grant / settings reload** — the server takes grants once at start (mcp_host.rs:30-32);
  `feed` updates snapshot + surface index only. A running server's grants change on restart (D7).
- **Client-side grant consumption** — `McpServerConfig::grants()` (config.rs:124-126) stays as the
  client-entry accessor for a future ticket (D6); this ticket touches only the expose side.
- **Validation/lint of unknown class names** — matches existing `from_classes` semantics: unknown =
  stored inert (D4); a startup warning for a typo'd class is a #375-candidate hardening, not here.
- **Any UI** — `settings.toml` stays the interface (the settings.rs:39-40 posture).

## Reference (§20)
**N/A — Marley-specific control-plane config; no reference-app behavior analog.** Marley exposing
ITSELF as a grant-gated MCP server is the orchestration-shell design (§4/§10), not an observable
Warp/Zed behavior: the Warp behavior map shows Warp's MCP subsystem is CLIENT-side only
(docs/warp_architecture/crates/mcp.md — `spawn_server`, per-server OAuth, connecting OUT to
servers; no self-expose surface, no operator grant table), and the Zed map has no MCP-server-expose
surface at all. The protocol contract is the open MIT-licensed MCP specification (the #370 posture);
no Warp (AGPL) or Zed (GPL) source read or relevant.

### Prior art
1. **Behavior maps — checked, N/A.** docs/warp_architecture/crates/mcp.md +
   subsystems/04-agent-ai-mcp.md map Warp as an MCP *client* (rmcp `spawn_server`, tool routing,
   per-server OAuth); neither map covers an app exposing its own grant-configured MCP server.
   docs/zed_architecture/: no MCP mention. Recorded as checked-and-not-applicable, not skipped.
2. **Published — the MCP spec (2025-06-18) has NO server-side grant-config convention.** The #370
   sweep's fetched record covers transports/tools/resources + the security posture
   (loopback/Origin/auth); authorization *granularity* — which caller may run which tool — is
   deployment-specific, with no spec-defined operator-config shape to adopt. The in-repo
   `.mcp.json.example` (repo root) is the CLIENT-side convention (`mcpServers.<name>` →
   `{url|command}`) that shaped #371's `[[mcp.servers]]`; there is no analogous published convention
   for a server's OWN grants — so our own shipped settings idiom governs (leg 3).
3. **OUR OWN source — the load-bearing leg; the seam is fully pre-built at both ends:**
   - `GrantTable` + `from_classes` (permission.rs:12-26): an UNVALIDATED collect into
     `write_classes: BTreeSet<String>` — an unknown class name is stored and simply never matches
     any registry `grant_class` in `decide` (permission.rs:49-58); silently inert, no error. D4
     matches this.
   - The enforcement path: `dispatch.rs:132` — `decide(spec.tier, spec.grant_class, ctx.grants)`,
     Deny → `isError` refusal; `registry.rs:73` — `session.surface_to_human` requires grant class
     **`"session.write"`** (the exact string the AC uses); `RequestCtx.grants` ← `ServerData.grants`
     (transport.rs:29) ← `McpHost::start(salt, grants)` (mcp_host.rs:28).
   - The gap itself: `start_mcp_server` passes `marley_mcp::GrantTable::default()` TODAY
     (app.rs:6589); serving starts ONLY via the `mcp-serve` verb (app.rs:8147; host `None` at boot,
     :191-192/:2106) — the evidence behind the no-`enabled` call (D2).
   - `transport()` (config.rs:110-120): `(None, None) → Err(McpConfigError::NoTransport)` at :117 —
     the evidence with teeth behind D1's rejection of putting "us" into `[[mcp.servers]]`.
   - The settings idiom to mirror verbatim: define_setting rows :86-98; AppliedSettings :216-218 /
     :370-371 / :412-413; `#[cfg(test)]` persist helpers :487-504; the tolerance-test shape
     :965-1028; the boot-applied copy precedent app.rs:2108; malformed-value → default via
     value.rs:14-16.

## Locked-In Decisions
- **D1 — Entry selection = a SEPARATE singleton `[mcp.expose]` TOML table** (the ticket's option c).
  Rejected **(a) reserved name** and **(b) per-entry role field**: both put a legitimately
  transport-less entry — Marley IS the server; there is no `command` and no `url` to write — into
  `[[mcp.servers]]`, where the #371 D2 resolver types no-command-and-no-url as the ERROR
  `McpConfigError::NoTransport` (config.rs:117, verified). A VALID config state landing in the
  resolver's error space poisons the per-entry contract: every transport consumer (#372's live-wire
  work included) would have to special-case the magic row before resolving. Additionally: the expose
  server is a SINGLETON — an array-of-tables invites "which of the three expose rows wins?", a
  question that shouldn't exist; and a typo'd magic name (`"marely"`) silently drops the operator's
  grants — the #219-class invisible-misconfiguration failure. A dedicated singleton table can
  express none of these failure modes.
- **D2 — `ExposeConfig` stays MINIMAL: `{ allow: Vec<String>, allow_write: Vec<String> }`**,
  `#[serde(default)]` on both, **`derive(Default)`** — with no non-derivable default in the struct
  (no #371 `enabled`-true bool trap), the derived Default trivially EQUALS the all-keys-absent
  decode; a test still pins it (the #371 D6 discipline). Rejected: an `enabled` field (serving is
  verb-gated — app.rs:8147/:6580, host `None` until the verb :2106; a flag duplicates the gate and
  invents an `enabled=false`-but-verb-invoked contradiction) and port/bind fields (loopback +
  OS-assigned port fixed by #370, transport.rs:82 — out of scope).
- **D3 — Grant resolution mirrors the SHIPPED semantics exactly.** The build is
  `GrantTable::from_classes(allow_write)` — the byte-identical body of the shipped
  `McpServerConfig::grants()` (config.rs:124-126) — and `allow` (read classes) is CARRIED, not
  consumed, exactly as today (config.rs:40-42; the read tier is loose, permission.rs:50-51). No new
  permission semantics are introduced anywhere in this ticket.
- **D4 — Unknown class names follow existing `from_classes` behavior: stored inert, no error.**
  `from_classes` collects any string unvalidated (permission.rs:21-25); a class that matches no
  registry `grant_class` simply never satisfies `decide` (permission.rs:52). `[mcp.expose]` entries
  are NOT validated against the registry — matching the shipped semantics is the contract; a
  startup lint for typo'd classes is deferred (#375-candidate hardening).
- **D5 — Absent-table regression pin.** No `[mcp.expose]` in settings ⇒ `ExposeConfig::default()`
  ⇒ `grants()` == `GrantTable::default()` BYTE-EQUAL (`from_classes([])` yields the same empty
  `write_classes` BTreeSet) — an unconfigured Marley serves exactly as today's shipped
  `start_mcp_server` (app.rs:6589). Pinned by an explicit `assert_eq!`.
- **D6 — `McpServerConfig::grants()` STAYS pub API.** It is the CLIENT-entry accessor (#371 D7 —
  grants carried opaquely per `[[mcp.servers]]` row) for future client-side grant consumption; this
  ticket adds the expose-side path WITHOUT touching it. Named here so no later inspector flags it
  as dead code.
- **D7 — The wire = a boot-applied copy + a one-line masked swap.** `AppliedSettings.mcp_expose` →
  a `RootView` field seeded at boot (the app.rs:2108 `language_servers` precedent — app code never
  calls `manager.get` at runtime) → `start_mcp_server` passes the built table into the UNCHANGED
  `McpHost::start(salt, grants)` (mcp_host.rs:28). Grants are read at serve time from the boot
  copy; live re-grant on hand-edit-while-running is out of scope (the server takes grants once at
  start, mcp_host.rs:30-32).
- **D8 — Ownership: `ExposeConfig` lives in `marley_mcp`** (the #308/#371 domain-crate rule —
  `marley_app` already deps `marley_mcp`; the grant type it produces is marley_mcp's). Whether it
  lands in config.rs beside `McpServerConfig` or a new expose.rs is a Phase-2 file-layout call —
  type, semantics, and tests are identical either way.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN settings carry `[mcp.expose]` with `allow_write = ["session.write"]`, a `tools/call` of `session.surface_to_human` through the server's request path shall PASS the permission check (proceed to execution), AND a write class NOT listed shall still be DENIED with the typed `isError` refusal (deny-by-default preserved). | unit through `handle_message` (the dispatch.rs:298 shape) with `RequestCtx.grants` = the `ExposeConfig`-built table: granted → receipt/effect path; plus a table granting only an unrelated class → `isError` refusal |
| REQ-002 | WHEN `[mcp.expose]` is ABSENT from settings, the built grant table shall equal `GrantTable::default()` exactly — today's shipped server behavior, byte-for-byte. | `assert_eq!(ExposeConfig::default().grants(), GrantTable::default())` (D5 pin) + settings unit: fresh dir → `get::<McpExpose>()` == `ExposeConfig::default()` |
| REQ-003 | WHEN `[mcp.expose]` is persisted (via the `#[cfg(test)]` helper) and the manager reloads from the same dir, the system shall yield the identical `ExposeConfig` through both `get` and `applied_from`. | settings round-trip unit (the #371 shape, settings.rs:969-999) |
| REQ-004 | WHEN a hand-edited `[mcp.expose]` omits a key (e.g. only `allow_write` present), the system shall load the missing field with its declared default (empty) and shall wipe NOTHING — not the present field, not the sibling `[[mcp.servers]]` table in the same file. | hand-written-TOML unit (the #204 shape, settings.rs:1013-1027) with both tables in one file |
| REQ-005 | WHEN the `mcp.expose` value is type-malformed (e.g. `expose = "notatable"` under `[mcp]`), the load shall yield `ExposeConfig::default()` — no panic, and other settings unaffected. | `write_settings` + `get` unit (the settings.rs:1005-1011 shape; the value.rs:14-16 `.ok()` mechanism) |
| REQ-006 | The derived `Default` shall equal the all-keys-absent serde decode of `ExposeConfig` (D2 pin — trivially true with no non-derivable defaults, still pinned). | unit: decode `{}` / an empty TOML table == `ExposeConfig::default()` (the #371 D6-test shape, config.rs:201-213) |
| REQ-007 | The `mcp-serve` callsite shall pass the settings-derived grant table (not the hardcoded default) into `McpHost::start`. | HONEST split: the pure resolver + REQ-001/002 units carry the behavior at cov/MSI 100; the one-line masked swap at app.rs:6589's successor is verified by Phase-3.5 inspect diff review (the masked-glue posture — no headless lane starts the listener, mcp_host.rs:7, so no automated callsite test is claimed) |
| REQ-008 | The new grant-resolution seam (`ExposeConfig`, `grants()`, the settings wiring) shall be PURE — no IO beyond the existing `*_in(dir)` manager path — at cov 100 / MSI 100; the mcp_host/app glue stays masked; any match over a `Family`/class shape stays EXHAUSTIVE (no defensive catch-all). | gate:4/gate:5 exit codes on the touched files + inspect checklist |

## Phase Plan
- **P2 Design** — settle config.rs-vs-expose.rs (D8); exact `define_setting!`/`AppliedSettings`/
  persist lines + the RootView field; the test manifest per REQ row; **run `cargo mutants --list -f`
  on the ACTUAL touched files** (marley_mcp config/expose + marley_app settings.rs) — the real set,
  never guessed operators (the syntactic-form rule; `grants()` returns a Default-carrying type, so
  its body mutant is viable — plan its killer).
- **P3 Implement** — `ExposeConfig` + `grants()` + lib.rs export (marley_mcp); the setting + wiring
  + `#[cfg(test)] persist_mcp_expose` (settings.rs); the boot copy + one-line callsite swap in
  `start_mcp_server` (app.rs). File manifest: `crates/marley_mcp/src/config.rs` (or expose.rs +
  lib.rs), `crates/marley_app/src/settings.rs`, `crates/marley_app/src/app.rs` (masked glue only).
- **P3.5 Inspect** — critics: deny-by-default preserved end-to-end? absent-table byte-equality
  real? no enabled/port creep? `McpServerConfig::grants()` untouched (D6)? the REQ-007 callsite
  diff line reviewed? unknown-class inertness matches D4?
- **P4 Validate** — write + RUN the planned tests; re-run `cargo mutants --list -f` on the ACTUAL
  shipped files and kill the real set; gate green (cov/MSI 100 on the pure seam).
- **P5 Complete** — archive; orchestration-shell §10 row + marley_mcp component doc updated
  (config now REACHES the live server); AAR capture; close #374.
