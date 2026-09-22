# marley_mcp — wire [mcp.expose] grants into start_mcp_server (S3) — Notes

- **Forge ticket:** #374 1200f35c-e444-4c16-aa68-c1347269684b
- **AAR:** f4bc1341-c090-48ff-9dbd-f3426cf958f6 (opened at /work)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-374-mcp-expose-grants-wiring.md
- **Pipeline spec:** 374-mcp-expose-grants-wiring.spec.md

## Phase 1 — Plan
- **Request:** Ticket #374 (sprint #34 "M23.5 — Fleet Layer-1 Consolidation") — the S3 slice #371's
  design EXPLICITLY deferred (371-…notes.md Phase-2 "S3 → DEFERRED": *"WHICH `[[mcp.servers]]` entry
  feeds #370's OWN expose-side server's grants … is unsettled"*). Route operator grant config into
  the live loopback server so `session.surface_to_human` becomes grantable; today it is denied for
  every operator regardless of settings.
- **Classification / tier:** feature; SMALL — one new pure struct + accessor (marley_mcp), one
  settings row + wiring (the #371 idiom verbatim), and a one-line masked callsite swap. No protocol
  change, no new permission semantics (D3), no UI. Soft-ordered before #375 (hardening pins the
  final server-start surface; this changes what flows into `McpHost::start`).
- **Forge recall (§18.3):** drafted offline (Phase-1 doc pass — no live forge calls from this
  drafter); re-run live at `/work` promotion. Applied from the on-disk record: the #371 S3
  deferral + D2/D6/D7 decisions; the #204 missing-key-must-not-wipe lesson; the #219-class
  invisible-misconfiguration failure (a typo'd magic name silently dropping grants — D1 teeth);
  `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators` + the syntactic-form rule
  (Phase Plan); orchestration-shell §10 deny-by-default as the binding frame.
- **Discovery (the evidence the lock was verified against):**
  - **The gap, precisely:** `start_mcp_server` passes `marley_mcp::GrantTable::default()` —
    app.rs:6589 — with its own doc comment (:6577-6578) *"Grants default to empty (deny-by-default)
    until #371's settings land"*, and mcp_host.rs:25-26 saying the same. #374 is the wire both
    comments anticipate. The receiving side is fully pre-built: `McpHost::start(salt, grants)`
    (mcp_host.rs:28) → `ServerData.grants` (transport.rs:29) → `RequestCtx.grants` →
    `dispatch.rs:132` `decide(spec.tier, spec.grant_class, ctx.grants)` → Deny = typed `isError`.
    NO signature changes anywhere — only the VALUE passed at the callsite.
  - **transport()-poisoning check (the D1 lock's teeth) — HELD.** config.rs:110-120:
    `(None, None) → Err(McpConfigError::NoTransport)` (:117). A reserved-name or role-field expose
    row is legitimately transport-less (Marley IS the server) → it would be a VALID state living
    permanently in the #371 D2 resolver's typed-error space; every `[[mcp.servers]]` consumer
    resolving transports (#372's live-wire work next door) would special-case the magic row. Plus
    singleton-shape ("which of three expose rows wins?" must not be expressible) and the silent
    typo'd-name grant-drop. Lock (c) confirmed with code evidence.
  - **Verb gating (the no-`enabled` call) — CONFIRMED.** The host is `None` at boot (app.rs:191-192,
    :2106 "started on demand by the `mcp-serve` verb") and starts ONLY at app.rs:8147
    (`"mcp-serve" => self.start_mcp_server()`); re-invoke is a no-op (:6581-6583). An `enabled`
    field would duplicate the verb gate.
  - **from_classes unknown-class semantics — CONFIRMED inert.** permission.rs:21-25 collects ANY
    string into `write_classes` unvalidated; `decide` (:49-58) only ever tests membership of a
    registry `grant_class`, so an unknown/typo'd class grants nothing and errors nothing. D4
    matches this exactly; a startup lint = #375-candidate. NOTE the residual hazard is honest: a
    typo'd CLASS VALUE inside `[mcp.expose]` is still silently inert — the singleton table kills
    the typo'd-NAME failure (entry selection), not value typos; that residue is precisely #375's
    hardening turf.
  - **The exact grant-class string:** registry.rs:69-75 — `session.surface_to_human` requires
    `grant_class: "session.write"` (dispatch tests use it, dispatch.rs:298). The forge ticket's
    shorthand `allow_write=["session"]` would NOT match; the spec's AC uses the real string
    `"session.write"` (D3/D4: match shipped semantics, no new mapping layer).
  - **Absent-table pin arithmetic:** `from_classes([])` → `GrantTable { write_classes: {} }` ==
    `GrantTable::default()` (permission.rs:12-17 — one field, `Default` derived). Byte-equal holds
    structurally; REQ-002 pins it.
  - **Settings idiom (mirror #371 verbatim):** define_setting rows settings.rs:86-98; AppliedSettings
    :216-218; applied_defaults :370-371; applied_from :412-413; `#[cfg(test)]` persist helpers
    :487-504 (posture comment: hand-edited only, the file IS the interface); tolerance tests
    :965-1028 (`write_settings` fixture at :1007 for malformed/hand-written cases). Boot-applied
    copy precedent: app.rs:2108 (`language_servers: applied.language_servers.clone()`) — app code
    NEVER calls `manager.get` at runtime (grep: zero hits), so the wire is a RootView field, not a
    verb-time manager read.
  - **First struct-valued setting — mechanically fine.** All shipped settings are scalars/Strings/
    Vecs; `[mcp.expose]` is the first singleton-table value. The framework is shape-agnostic:
    `Registered.resolve`/`set` go through `to_file_value()` (manager.rs:76, :92-97) and the blanket
    `SettingsValue` (value.rs:7-19) covers any `Serialize + DeserializeOwned`; a struct serializes
    to `toml::Value::Table` at the `mcp.expose` node → `[mcp.expose]` in the file. Malformed value
    → `from_file_value` `.ok()` → default (value.rs:14-16); a PRESENT table missing a key →
    `#[serde(default)]` fills it (the #204 tolerance at the serde layer). Phase 2 should still
    eyeball the emitted TOML in the round-trip test (the one genuinely novel surface here).
  - **Testing boundary (REQ-007 honesty):** mcp_host.rs is masked + coverage-excluded (header, :5-7
    "no listener churns the test harness") and start_mcp_server is masked glue — there is NO
    headless lane that starts the listener. So the callsite swap is inspect-verified (one diff
    line), while the BEHAVIOR (config → table → allow/deny through `handle_message`) is fully
    unit-proven at the pure seam. Stated plainly in REQ-007 rather than over-claimed.
- **Decisions:** D1-D8 locked in the spec — singleton `[mcp.expose]` (c-lock, verified teeth) ·
  minimal struct, no enabled/port (verb gate + #370 loopback cites) · grant build mirrors
  `McpServerConfig::grants()` byte-identically · unknown classes inert per shipped `from_classes` ·
  absent-table byte-equal pin · `McpServerConfig::grants()` stays pub (client-entry accessor, not
  dead code) · boot-copy wire + one-line masked swap, no live re-grant · marley_mcp ownership
  (config.rs-vs-expose.rs = Phase-2 layout call).
- **Sibling context (sprint #34):** #372 live-wire readiness + #373 reconnect/backoff are
  client-side consolidation; #375 hardening lands AFTER this (final ctor surface). No build-order
  dependency on either — the expose seam is entirely #370/#371-shipped code.

## Phase 2 — Design

### Verified against the LANDED code (all spec cites confirmed)
`GrantTable::from_classes(iter)` (permission.rs:21-25) + `decide` (permission.rs:49-58, deny-by-default);
`McpServerConfig::grants()` = `GrantTable::from_classes(self.allow_write.iter().cloned())` (config.rs:124-126)
— the exact body `ExposeConfig::grants()` mirrors; `session.surface_to_human` needs grant class
`"session.write"` (registry.rs:73); the gate is `decide(spec.tier, spec.grant_class, ctx.grants)`
(dispatch.rs:132); `RequestCtx.grants: &GrantTable` (lib.rs:52); the callsite passes
`marley_mcp::GrantTable::default()` at app.rs:6589 (its doc: "until #371's settings land"), verb-gated
(host `None` until `mcp-serve`, app.rs:6581/:8147). `SettingsValue` is a BLANKET impl over any
Serialize+Deserialize (value.rs:19), so a struct value works — `[mcp.expose]` becomes a `[mcp]`-sub-table
via the dotted `"mcp.expose"` key, coexisting with the `[[mcp.servers]]` array under the same `[mcp]`.

### Approach — D8 resolved: a NEW `crates/marley_mcp/src/expose.rs`
A distinct singleton concern (vs config.rs's `[[mcp.servers]]` array entries) → its own file keeps
`cargo mutants -f expose.rs` crisp and config.rs focused. §20 N/A (self-expose grants = orchestration-shell
design, no Warp/Zed analog — confirmed).

**① `marley_mcp::expose::ExposeConfig`** (pure, cov/MSI 100):
```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExposeConfig {
    #[serde(default)] pub allow: Vec<String>,       // read classes carried, not consumed (D3)
    #[serde(default)] pub allow_write: Vec<String>, // → the GrantTable
}
impl ExposeConfig { pub fn grants(&self) -> GrantTable { GrantTable::from_classes(self.allow_write.iter().cloned()) } }
```
`derive(Default)` is trivially == the all-absent decode (no `enabled`-true trap like #371's McpServerConfig
— D2), still pinned by a test. Exported from lib.rs beside `McpServerConfig` (lib.rs:26).

**② `marley_app` settings.rs** (mirror the #371 rows exactly):
`define_setting!(McpExpose: marley_mcp::ExposeConfig = marley_mcp::ExposeConfig::default(), "mcp.expose")`;
`AppliedSettings.mcp_expose: marley_mcp::ExposeConfig`; `applied_defaults` (`McpExpose::default_value()`) +
`applied_from` (`manager.get::<McpExpose>()`); `#[cfg(test)] persist_mcp_expose`; the 3 test AppliedSettings
literals gain `mcp_expose: marley_mcp::ExposeConfig::default()`; the tolerance test.

**③ `marley_app` app.rs** (masked glue only): `RootView.mcp_expose: marley_mcp::ExposeConfig` (field near
:194) seeded at boot `mcp_expose: applied.mcp_expose.clone()` (the language_servers precedent :2108); the
one-line swap at :6589 → `self.mcp_expose.grants()` (from `GrantTable::default()`). `McpHost::start(salt,
grants)` + `ServerData.grants` UNCHANGED (signatures already carry grants; only the value changes).

### File manifest
| # | File | Change |
|---|---|---|
| 1 | `crates/marley_mcp/src/expose.rs` | NEW pure — `ExposeConfig` + `grants()` + unit tests (incl. a handle_message end-to-end grant proof) |
| 2 | `crates/marley_mcp/src/lib.rs` | `mod expose;` + `pub use expose::ExposeConfig;` |
| 3 | `crates/marley_app/src/settings.rs` | `define_setting!` McpExpose + AppliedSettings field + applied wiring + `persist_mcp_expose` + 3 test literals + tolerance test |
| 4 | `crates/marley_app/src/app.rs` | RootView field + boot seed + the :6589 callsite swap (masked glue) |
| 5 | `CHANGELOG.md` | Added entry |

### Regression Test Plan
| # | Test | Kind | Proves |
|---|---|---|---|
| U1 | `ExposeConfig{allow_write:["session.write"]}.grants()` → `session.surface_to_human` PASSES the permission check via `handle_message`; a table granting only an unrelated class → typed `isError` deny (deny-by-default preserved) | pure unit (expose.rs, the dispatch.rs:298 shape) | REQ-001 |
| U2 | `assert_eq!(ExposeConfig::default().grants(), GrantTable::default())` | pure unit | REQ-002 (D5 pin) |
| U3 | `grants()` maps every `allow_write` class + leaves `allow` untouched | pure unit | REQ-001/D3 |
| U4 | decode `{}` / empty table == `ExposeConfig::default()`; serde round-trip a full struct | pure unit | REQ-006 (D2 pin) |
| S1 | persist `[mcp.expose]` → reload → identical via `get` + `applied_from`; absent → default | settings unit | REQ-002/003 |
| S2 | hand-edited `[mcp.expose]` with only `allow_write` + a sibling `[[mcp.servers]]` in the same file → missing key defaults, NOTHING wiped | settings unit | REQ-004 |
| S3 | malformed `expose = "notatable"` under `[mcp]` → `ExposeConfig::default()`, no panic; other settings intact | settings unit | REQ-005 |
| R1 | inspect diff review of the app.rs:6589 swap (masked glue — no headless listener lane, mcp_host.rs:7) | inspect | REQ-007 |
| G1 | `cargo mutants --list -f expose.rs` + settings.rs touched fns real set killed; gate green | gate | REQ-008 |

### Risks / decisions
- **First struct-valued setting** — verified `SettingsValue` blanket impl covers it; the emitted TOML is a
  `[mcp.expose]` sub-table (Phase 3 will eyeball the round-tripped file to confirm the dotted-key shape).
- **`grants()` returns a Default-carrying `GrantTable`** → its body-replacement mutant is VIABLE (unlike
  #370's no-Default enums, the #204 case) → U2/U3 kill it (exact `write_classes` asserts).
- **REQ-007 masked** — the callsite swap is one line in the coverage-excluded app.rs; the pure resolver +
  U1/U2 carry the behavior; inspect reviews the diff (no automated listener test, honest per mcp_host.rs:7).
- **`McpServerConfig::grants()` untouched (D6)** — stays the client-entry accessor; named so inspect won't
  flag it dead.

## Phase 3 — Implement
- **Built to the manifest.** NEW `crates/marley_mcp/src/expose.rs` (`ExposeConfig{allow,allow_write}` +
  `grants()` + 4 unit tests incl. the handle_message end-to-end grant proof); `lib.rs` `mod expose` +
  `pub use expose::ExposeConfig`. `settings.rs`: `define_setting!(McpExpose … "mcp.expose")` +
  `AppliedSettings.mcp_expose` + applied_defaults/applied_from + `#[cfg(test)] persist_mcp_expose` + the 3
  test AppliedSettings literals + the `mcp_expose_setting_round_trips_and_tolerates` test. `app.rs` (masked
  glue): `RootView.mcp_expose` field + boot seed `applied.mcp_expose.clone()` + the one-line callsite swap
  at `start_mcp_server` (`GrantTable::default()` → `self.mcp_expose.grants()`).
- **Deviations:** none — D8 resolved to a new expose.rs as designed. Only the boot RootView site needed the
  new field (no other construction sites); the workspace compiles clean.
- **Compile + run:** `cargo check -p marley_mcp` + `-p marley` (tests) clean; expose 4/4, the settings
  module 31/31 (incl. the new tolerance test), marley_mcp lib 43/43 — all pass. fmt clean.
- **Floor posture:** expose.rs + settings.rs are PURE (in the coverage/mutation lane); the app.rs callsite
  is masked (`start_mcp_server` already `mutants::skip` + app.rs coverage-excluded). `McpServerConfig::
  grants()` UNTOUCHED (D6). The handle_message end-to-end test proves the grant actually gates the real
  dispatch path (not just `decide` in isolation).

## Phase 3.5 — Inspect
Two independent critics (correctness/security · settings-integrity/reuse/floors). **No material defects.**
Lenses covered + verdicts:
- **Deny-by-default preserved end-to-end** (CONFIRMED, full chain traced `ExposeConfig.allow_write` →
  `grants()` → `RootView.mcp_expose` → `McpHost::start` → `ServerData.grants` → `RequestCtx.grants` →
  `decide`): `from_classes` maps ONLY `allow_write`; `decide` is an exact `BTreeSet::contains` with no
  normalization; no path grants more than listed; the end-to-end `handle_message` test proves both the
  granted and unlisted-denied arms.
- **Absent-table pin byte-equal** (CONFIRMED: `ExposeConfig::default().grants() == GrantTable::default()`,
  empty BTreeSet) — an unconfigured Marley serves exactly as pre-#374.
- **Callsite swap correct + single path** (CONFIRMED: `mcp_expose` written once at boot, read once, no
  runtime reassignment; grants immutable after start — a positive tamper-resistance property).
- **No scope creep** (CONFIRMED: no enabled/port/bind in `ExposeConfig`; `config.rs`/`McpServerConfig::
  grants()` untouched — D6 held).
- **Unknown class names inert** (CONFIRMED: stored, never match `decide`, no panic/whole-config rejection).
- **First struct-valued setting emits a CLEAN `[mcp.expose]` sub-table** (critic 2 EMPIRICALLY verified the
  on-disk TOML is a proper header, not a mangled inline, coexisting with `[[mcp.servers]]` both orderings).
- **Floors** (CONFIRMED: expose.rs/settings.rs in the covered/mutated lane, app.rs excluded; the sole
  `grants -> Default::default()` mutant caught; `persist_mcp_expose` is `#[cfg(test)]` → not a prod mutant);
  **REQ-007 masked-glue honest** (no listener lane; carried by the pure resolver + the end-to-end test).

**One augmentation applied** (from critic 2's optional-hardening note): added a FIELD-level malformation
assertion to the tolerance test — `allow_write = "notanarray"` inside a valid `[mcp.expose]` header must
fail CLOSED (whole table → default = empty grants, deny-by-default), never a panic or a loosened grant. The
security-relevant "a typo in the grants field cannot accidentally grant." Passes.

Informational (accepted, no change): `ExposeConfig.allow` is carried-not-consumed (reads are loose — D3,
forward-looking, mirrors `McpServerConfig.allow`); `grants()` duplicates `McpServerConfig::grants()` (two
tiny domain types — extracting a shared helper would be over-abstraction); boot-snapshot staleness
(hand-edit-then-serve uses boot grants — documented, matches the `language_servers` posture).
- **Post-inspect:** expose 4/4 + settings 32/32 (with the added field-level case) + marley_mcp lib 43/43 green.

## Phase 4 — Validate
- **Tests RUN:** expose.rs 4 units (end-to-end grant gate via handle_message; grants-maps-allow_write;
  default==GrantTable::default; empty-decode==default + round-trip) + settings `mcp_expose_setting_round_
  trips_and_tolerates` (round-trip · absent→default · whole-table-malformed→default · FIELD-level-malformed→
  default fail-closed · partial-edit-no-wipe cross-table). marley_mcp lib 43/43, marley settings 32/32 —
  all pass.
- **Mutants (`cargo mutants --list -f expose.rs`):** 1 viable (`grants -> Default::default()`), CAUGHT by
  the grant-maps + end-to-end tests (a GrantTable Default → empty → both asserts fail). `persist_mcp_expose`
  is `#[cfg(test)]` → not a production mutant (matching `persist_mcp_servers`).
- **Full gate `scripts/gates.sh --diff`: GATE GREEN [diff] — 15/15** (coverage ≥100% incl. expose.rs +
  settings.rs, mutation MSI ≥100%, clippy -D warnings, machete/gitleaks/audit/deny/miri/visual green).
  Receipt written.
- **No live-app drive:** the change is a pure config type + settings wiring + a one-line masked callsite
  swap; no UI/render surface (§7 N/A). REQ-007's masked `McpHost::start(salt, self.mcp_expose.grants())`
  line is the honest masked-glue residual (no headless listener lane), carried by the pure resolver + the
  end-to-end grant test.
- **Pre-existing:** none touched.

## Phase 5 — Complete
- **CHANGELOG:** entry under Added (M23.5 ③, `[mcp.expose]` grants govern the live server).
- **Architecture docs:** `orchestration-shell.md` §10 permission-tiers bullet updated (grants wired live
  #370→#374, fail-closed on absent/malformed).
- **Knowledge:** inspect was clean (no defects) — no failure/prevention-rule to record; the one lesson
  (a malformed permission-config field must fail CLOSED) is a confirmed application of the existing
  `value.rs` `.ok()` decode-or-default mechanism, pinned by the added field-level test. AAR `f4bc1341`
  submitted.
- **Ticket #374 closed + pipeline archived to completed/.**
