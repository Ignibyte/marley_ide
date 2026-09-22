# Settings — [[mcp.servers]] + per-project orchestration config round-trip — Notes

- **Forge ticket:** #371 ec7757ce-0d0c-4f4c-b7ac-978fa676a5df
- **AAR:** 7c5631a2-b78b-428a-b5b7-b9fe5fb62ca4
- **Local ticket doc:** docs/planning/tickets/open/TICKET-371-mcp-servers-settings.md
- **Pipeline spec:** 371-mcp-servers-settings.spec.md

## Phase 1 — Plan
- **Request:** Ticket #371 (M23, sprint "Fleet Control Plane: Layer 1" — orchestration-shell.md §12
  Layer-1 slice ⑤): the fleet control plane's WIRING layer as the shipped `[[lsp.servers]]` idiom —
  (a) `[[mcp.servers]]` (name · stdio/http transport · enabled · per-server grants for #370),
  (b) per-project orchestration config (brain MCP endpoint for #368 + web URL for the Phase-E pane),
  (c) full hand-edit-tolerant round-trip (`#[serde(default)]` everywhere — the #204 lesson),
  (d) pure seam cov/MSI 100. NO live connection behavior.
- **Classification / tier:** feature; pure config plumbing — schema + resolvers only, no gpui shim,
  no live IO beyond the existing `load_manager_in(dir)` path (tempdir-tested, §14 `*_in(dir)`).
  Consumers are the concurrent siblings #368 (endpoint) + #370 (grants) and Phase E (web URL).
- **Forge recall (§18.3):** drafted offline (Phase-1 doc pass — no live forge calls from this
  drafter); recall satisfied from the on-disk record and must be re-run live at `/work` promotion.
  Applied here: the #204 RemoteHosts/Workflows serde-default wipe lesson (verbatim, now a spec REQ);
  `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators` + the syntactic-form rule
  (baked into the Phase Plan); the #203/#204 Default-derive mutant-viability lesson (spec D6);
  the never-logged-bearer posture from `marley_forge_client` (spec D4);
  orchestration-shell.md §2/§7/§8/§10/§12 as the ratified design frame.
- **Discovery (the precedent anchors — the prior art IS the spec):**
  - `[[lsp.servers]]` #308: `define_setting!` at crates/marley_app/src/settings.rs:80-85;
    `LanguageServerConfig` at crates/marley_lsp/src/config.rs:13-22 (`#[serde(default)]` args
    :20-21 — struct lives in the DOMAIN crate); AppliedSettings field :200-201, applied_defaults
    :352, applied_from :392; test-only persist :458-464; the round-trip/tolerance test :856-917.
  - `[[remote.hosts]]` #87: settings.rs:70-74; `RemoteHost` crates/marley_remote/src/lib.rs:134-141;
    tolerance test settings.rs:744-770; drop-a-bad-entry-keep-the-rest posture lib.rs:152-154.
  - `[[workflows]]` #204: settings.rs:75-79; `Workflow` crates/marley_app/src/workflows.rs:10-22
    (`#[serde(default)]` params :20-21); the missing-key-must-not-wipe test settings.rs:838-854.
  - Framework tolerance seam: `marley_settings` blanket `SettingsValue`; `from_file_value` →
    `.try_into().ok()` at crates/marley_settings/src/value.rs:14 — a type-malformed VALUE decodes to
    the default (whole-Vec granularity), which is exactly why spec D2 rejects a serde-tagged
    transport enum in favor of flat Options + a pure per-entry resolver.
  - MCP config shape (published convention): repo-root `.mcp.json` read for SHAPE ONLY —
    `mcpServers.<name>` → `{type: "http", url, headers}` | `{command, args}`; no value copied.
    Today's forge wiring parses exactly that file: `forge_endpoint_from`,
    crates/marley_forge_client/src/lib.rs:107-126 (url + Authorization; bearer redacted in Debug).
  - **Per-project persistence finding (the one genuine design question):** NO per-project persisted
    CONFIG exists. Projects persist only as LAYOUT inside the global `workspace.shell` string blob —
    `ProjectLayout{root, active_tab, tabs}` at crates/marley_app/src/grid_layout.rs:228-241, a
    line/tab-framed codec whose `breaks_framing` (:248) silently drops unrepresentable entries;
    rewritten on layout persists — wrong home for hand-edited config. The root-keyed GLOBAL-settings
    precedent is shipped twice: `workspace.recents` #234 (settings.rs:86-91) and `rail.collapsed`
    #245 (settings.rs:92-97, "keyed by root, NOT the shifting index"). → spec D-OPEN-1 recommends
    `[[projects.orchestration]]` in global settings keyed by root; shell-codec + per-repo-file
    alternatives recorded with why they lean rejected.
  - Settings UI check: none exists — `settings.toml` is file-based/hand-edited (settings.rs:39-40:
    "the settings file IS the interface"); scope Out says so explicitly.
- **Decisions:** D1-D7 locked in the spec (idiom-verbatim · flat-fields transport + pure resolver ·
  ecosystem vocabulary · NO credential fields v1 · generic-only names · enabled-true serde default in
  lockstep with `Default` · grants carried-not-interpreted). D-OPEN-1 (per-project landing) +
  D-OPEN-2 (struct ownership vs the concurrent #368/#370 crates) deferred to Phase 2 with evidence.

### Phase 1 — Planner review (promotion to active, 2026-07-21)
Promoted the Fable draft (spec+notes) queued→active. Confirmed the prior art against SHIPPED code: the
`define_setting!` `Vec<T> = Vec::new(), "toml.key"` pattern (settings.rs:70-97) + the root-keyed precedents
(`workspace.recents` #234, `rail.collapsed` #245) + `marley_mcp::GrantTable{write_classes}`+`from_classes`
(permission.rs:13/21). Attacked with the now-known landed state (#370 shipped THIS session):
- **S1:** D-OPEN-2 resolves — `McpServerConfig` → `marley_mcp` (the #308 domain-crate rule; marley_app deps
  marley_mcp); `allow_write` → `GrantTable::from_classes` via a pure `grants_for`, closing #370's S1 loop.
- **S2:** D-OPEN-1's `[[projects.orchestration]]` global root-keyed table is grounded (recents/collapsed
  precedents); `ProjectOrchestration` app-local, consumers get primitives.
- **S3:** flagged the Phase-2 scope call — optionally wire the configured grants into #370's shipped
  `start_mcp_server` (currently `GrantTable::default()`), a small masked change, still no live connection.
- **Verdict:** the 7 locked decisions + 8 EARS AC survive; D-OPEN-1/D-OPEN-2 are now Phase-2 confirmations
  (not open explorations) given the landed state. Status → Phase 1 PASS.

## Phase 2 — Design

### §20 confirm
**N/A — Marley-specific config plumbing, still holds.** A settings-table addition to Marley's own
clean-room `marley_settings` framework; no Warp/Zed behavior to match, no copyleft source read. The only
external SHAPE consulted is the published MCP client-config convention (`.mcp.json`, read for shape only).

### Architecture / approach
Two hand-editable settings tables on the SHIPPED `define_setting!` idiom (macros.rs:8 — a setting's value
type only needs the blanket `SettingsValue` = serde `Serialize + DeserializeOwned`; a type-mismatched value
decodes to the default via `from_file_value`→`.ok()` at value.rs:14, which is why D2 uses flat Options +
a pure per-entry resolver, NOT a serde-tagged enum). PURE — no IO beyond the existing `load_manager_in(dir)`
path; §14 honored (typed per-entry error, no panic, single-owner types in their domain crate).

### D-OPEN forks — CONFIRMED against landed state
- **D-OPEN-2 → `McpServerConfig` lives in `marley_mcp` (new `config.rs`)**, mirroring `LanguageServerConfig`
  in `marley_lsp` (config.rs:13 — the domain-crate rule); `marley_app` already deps `marley_mcp`. Its
  `grants()` returns `marley_mcp::GrantTable` (`from_classes(allow_write)`), closing #370's S1 loop.
  `ProjectOrchestration` → a new app-local `marley_app/src/orchestration.rs` (the `Workflow` precedent),
  consumers get primitives.
- **D-OPEN-1 → `[[projects.orchestration]]` global array-of-tables keyed by `root`** (the `workspace.recents`
  #234 / `rail.collapsed` #245 root-keyed precedents, settings.rs:86-97), resolved by exact root match. The
  shell-codec extension stays rejected (framing fragility + not hand-editable).
- **S3 → DEFERRED (design call).** WHICH `[[mcp.servers]]` entry feeds #370's OWN expose-side server's grants
  (a self-named entry? a convention?) is unsettled, and `[[mcp.servers]]` also serves the client-connect side
  (#368). #371 stays PURE: it carries the table + the `grants()`/`transport()`/`orchestration_for` seams; a
  bounded follow-up wires a chosen entry's grants into #370's masked `start_mcp_server` (currently
  `GrantTable::default()`). Keeps #371 free of a masked-glue change to shipped code.

### Exact types (all PURE, cov/MSI 100)
- `marley_mcp::config::McpServerConfig { name: String, #[serde(default)] command: Option<String>,
  #[serde(default)] args: Vec<String>, #[serde(default)] url: Option<String>,
  #[serde(default = "default_enabled")] enabled: bool, #[serde(default)] allow: Vec<String>,
  #[serde(default)] allow_write: Vec<String> }` — **hand-impl `Default` (NOT derive)** so `enabled: true`
  matches the serde-absent decode (D6 — a derived `Default` gives `false`; the trap a test pins).
  `fn default_enabled() -> bool { true }`.
- `enum McpTransport { Stdio { command: String, args: Vec<String> }, Http { url: String } }`;
  `enum McpConfigError { NoTransport, BothTransports }` (typed, `Display`+`Error`, per-entry).
- `impl McpServerConfig { fn transport(&self) -> Result<McpTransport, McpConfigError>` (Some(command) & None(url)
  → Stdio; None & Some → Http; neither → `NoTransport`; both → `BothTransports` — D2/REQ-003/004);
  `fn grants(&self) -> GrantTable` (`GrantTable::from_classes(self.allow_write.iter().cloned())` — D7/REQ-007) }`.
- `marley_app::orchestration::ProjectOrchestration { root: String, #[serde(default)] brain_endpoint:
  Option<String>, #[serde(default)] web_url: Option<String> }` — `derive(Default)` is safe here (no bool trap);
  `fn orchestration_for<'a>(configs: &'a [ProjectOrchestration], root: &str) -> Option<&'a ProjectOrchestration>`
  (exact match; REQ-005).

### File manifest
| File | Change |
|---|---|
| `crates/marley_mcp/src/config.rs` | NEW PURE — `McpServerConfig` (+ hand `Default`, `default_enabled`), `McpTransport`, `McpConfigError`, `transport()`, `grants()` |
| `crates/marley_mcp/src/lib.rs` | `mod config;` + `pub use config::{McpServerConfig, McpTransport, McpConfigError}` |
| `crates/marley_app/src/orchestration.rs` | NEW PURE — `ProjectOrchestration` + `orchestration_for` |
| `crates/marley_app/src/lib.rs` | `mod orchestration;` |
| `crates/marley_app/src/settings.rs` | `define_setting!(McpServers: Vec<marley_mcp::McpServerConfig> = Vec::new(), "mcp.servers")` + `define_setting!(ProjectOrchestrations: Vec<crate::orchestration::ProjectOrchestration> = Vec::new(), "projects.orchestration")`; `AppliedSettings` fields `mcp_servers` / `project_orchestrations` + `applied_from` (`manager.get::<…>()`) + `applied_defaults` (`…::default_value()`) — mirror settings.rs:201/352/392; `#[cfg(test)] persist_mcp_servers` / `persist_project_orchestrations` (mirror `persist_language_servers` :459) |

### Regression Test Plan (per REQ)
| REQ | Test |
|---|---|
| REQ-001 identity round-trip | settings.rs unit: persist both tables (incl. grants+args) → reload → identical `Vec` (the #308 shape) |
| REQ-002 missing-key no-wipe + declared defaults | settings.rs unit: hand-written TOML entry missing `args`/`enabled`/`allow`/… → decoded with defaults (`enabled=true`), siblings survive (#204 shape) |
| REQ-003 both transport shapes | `config.rs` units: `transport()` on a `command` entry → `Stdio`; on a `url` entry → `Http` |
| REQ-004 malformed → typed err + type-mismatch → default | `config.rs` units: neither → `NoTransport`, both → `BothTransports`, sibling still resolves; settings.rs unit: a `mcp.servers = "x"` (string, not array) → empty default, no panic |
| REQ-005 orchestration_for | `orchestration.rs` units: configured root → Some, unconfigured → None, multi-entry exact pick |
| REQ-006 agnosticism | Phase-3.5 inspect review + a grep test (no `forge`/`ucsos` in any field/key/default) |
| REQ-007 grants accessor | `config.rs` unit: `grants()` → `GrantTable::from_classes(allow_write)`; `allow` carried untouched |
| REQ-008 pure seam cov/MSI 100 | gate:4/gate:5 on config.rs + orchestration.rs + the settings resolvers; **D6 test:** `McpServerConfig { name, ..Default::default() }` == decode of `name="x"` only (pins `enabled=true`) |

**Testing boundary:** everything is PURE (no shim, no masked glue — S3 deferred). `config.rs`/`orchestration.rs`
+ the settings resolvers are cov/MSI 100; the `define_setting!` macro-generated code + `AppliedSettings`
wiring are covered by the round-trip units (the #308 precedent — settings.rs is NOT coverage-excluded).
Mutant trace runs `cargo mutants --list -f config.rs -f orchestration.rs` on the ACTUAL code at Validate
(per the plan; the viable set depends on the hand-`Default` + the typed-error shape).

### Risks / decisions
- **The `enabled` bool Default trap (D6)** is the one real gotcha — hand-impl `Default`, pin with a test.
- **D2 flat-Options-not-tagged-enum** is load-bearing: a tagged `transport` enum would let one typo wipe the
  whole table via `from_file_value`→`.ok()`; the pure resolver rejects per-entry while siblings survive.
- **S3 deferred** — the expose-side-server-grants wiring is a follow-up (the entry-selection convention is
  unsettled); #371 ships the pure config + seams.

## Phase 3 — Implement
- **Built (all PURE, per the manifest):**
  - `crates/marley_mcp/src/config.rs` (NEW): `McpServerConfig` (7 fields, every non-identity `#[serde(default)]`,
    `enabled` via `#[serde(default = "default_enabled")]`→true), a HAND-IMPL `Default` (enabled=true, D6 — a
    derived Default would give false), `McpTransport{Stdio,Http}`, `McpConfigError{NoTransport,BothTransports}`
    (typed `Display`+`Error`), `transport()` (D2 — command→Stdio/url→Http/neither→NoTransport/both→BothTransports),
    `grants()`→`GrantTable::from_classes(allow_write)` (D7, closing #370's S1 loop). `lib.rs`: `mod config` +
    `pub use config::{McpConfigError, McpServerConfig, McpTransport}`.
  - `crates/marley_app/src/orchestration.rs` (NEW): `ProjectOrchestration` (derive Default — no bool trap) +
    `orchestration_for` (exact-root match). `lib.rs`: **`pub mod orchestration`** (the marley_lsp `pub mod
    config` precedent — makes `orchestration_for` crate-public API so it isn't dead-code before its
    #368/Phase-E consumers land).
  - `settings.rs`: `define_setting!(McpServers: Vec<marley_mcp::McpServerConfig>, "mcp.servers")` +
    `define_setting!(ProjectOrchestrations: Vec<…ProjectOrchestration>, "projects.orchestration")`;
    `AppliedSettings.mcp_servers`/`.project_orchestrations` + `applied_from`/`applied_defaults` + the 3 test
    fixtures; `#[cfg(test)] persist_mcp_servers`/`persist_project_orchestrations` (mirror
    `persist_language_servers`).
- **Checks:** `cargo check --workspace` GREEN; `cargo fmt --all -- --check` clean.
- **Deviations / notes for inspect:**
  - **`pub mod orchestration`** (vs the usual private `mod`) — the marley_lsp `pub mod config` resolver
    precedent, so `orchestration_for` (consumer-less in this wiring slice — #368 endpoint / Phase-E web URL
    land later) is crate public API, not dead code. `McpServerConfig::transport()/grants()` are already
    exempt (methods on the `pub use`-exported type).
  - **2 expected dead-code warnings** (`persist_mcp_servers`/`persist_project_orchestrations`) — `#[cfg(test)]`
    helpers whose CALLERS are the Phase-4 round-trip tests (identical posture to `persist_language_servers`).
    They clear when Validate writes the tests. `cargo check` (no `-D warnings`) is green now.
  - **S3 deferred** — no `start_mcp_server` grant-wiring (the entry-selection convention is unsettled); #371
    ships the pure config + `grants()` accessor for a follow-up to consume.
  - No tests here (Phase-4).

## Phase 3.5 — Inspect
**2 critics (correctness/tolerance/data-integrity · agnosticism/idiom/simplification).** Both confirm the
PRODUCTION LOGIC is clean on all 6 questions: the `enabled` hand-`Default` EQUALS the serde
all-optional-absent decode (enabled=true, D6); every non-identity field `#[serde(default)]` (no #204 wipe);
`transport()`'s 4 arms correct/typed/no-panic (§14); `grants()`→`from_classes(allow_write)` with `allow`
untouched (D7); `orchestration_for` exact-root match; the round-trip wiring threads both tables through
`AppliedSettings`/`applied_from`/`applied_defaults` + all 3 test fixtures. §20 clean-room + the typed-error
idiom (matches `ForgeError`) + domain-crate placement (McpServerConfig→marley_mcp, ProjectOrchestration→
marley_app) all confirmed.

### Findings ledger
| # | Lens | Sev | Finding | Verdict | Resolution |
|---|---|---|---|---|---|
| F1 | agnosticism (REQ-006) | — | grep `forge/ucsos` → only 2 DOC-comment hits; every field/key/variant/default generic | CLEAN | No action — REQ-006 satisfied |
| F2 | idiom | LOW | `pub mod orchestration` deviates from marley_app's house style (64 private `mod` + flat `pub use`, only this one `pub mod`); the marley_lsp precedent is a different crate with a permanent cross-crate consumer | REAL (idiom) | **Fixed** — `mod orchestration; pub use orchestration::{orchestration_for, ProjectOrchestration}` — kills the same dead-code, flat like every sibling domain type |
| F3 | correctness | INFO | the 2 `#[cfg(test)]` persist helpers are uncalled → `clippy -D warnings` warns NOW | EXPECTED (Phase-4) | **Hard Phase-4 requirement:** validate MUST add `mcp_servers_setting_round_trips` + `project_orchestrations_setting_round_trips` calling both helpers (mirror `language_servers_setting_round_trips`). `cargo check` (no `-D`) is green; the gate closes it when the tests land |
| F4 | correctness | INFO | config.rs doc says "a test pins the equality" but the D6 test lands in Phase 4 | forward-ref | Phase-4 adds `assert_eq!(McpServerConfig::default(), decode("name=\"x\""))` — making the claim true |

**Result:** 1 idiom fix (F2); **0 logic defects** (all 6 questions traced clean). F3/F4 are the Phase-4 test
layer (per the design's test plan), not inspect defects. `cargo check` green; `clippy` clean except the 2
expected persist warnings Phase-4 closes.

## Phase 4 — Validate
### Tests added (per REQ)
- **`config.rs` units (5):** `transport_resolves_all_four_cases` (D2/REQ-003/004 — Stdio/Http/NoTransport/
  BothTransports), `grants_maps_allow_write_and_leaves_allow_untouched` (D7/REQ-007), `config_error_display_
  distinguishes_the_two_arms`, **`default_equals_the_serde_minimal_decode_with_enabled_true`** (D6 — the
  hand-`Default` == a `serde_json` decode of `{"name":"x"}`, enabled=true), `serde_round_trips_a_full_entry`.
- **`orchestration.rs` units (2):** `orchestration_for_picks_the_exact_root_or_none` (REQ-005 — multi-pick /
  unconfigured / empty), `serde_defaults_a_root_only_entry_to_none_endpoints`.
- **`settings.rs` round-trip/tolerance (2, mirror the #308 shape):** `mcp_servers_setting_round_trips_and_
  tolerates` + `project_orchestrations_setting_round_trips_and_tolerates` — identity via the persist helpers
  (kills their `Ok(())` mutant AND closes the F3 dead-code), absent→empty default, malformed-value→default
  (no panic), missing-optional-key→declared defaults `enabled=true` (no #204 wipe), + `applied_from` flow.
  (Applied the #370 lessons: plain asserts, no formatted messages / unreachable arms.)
### Runs (actual)
- `cargo nextest run --workspace` → **1869 passed, 0 failed, 5 skipped** (+9).
- `llvm-cov -p marley_mcp` → `config.rs` **100% lines** (112/112).
- `cargo mutants -f config.rs -f orchestration.rs` → **7 mutants: 6 caught, 1 unviable (`transport→Ok(Default)`,
  McpTransport has no Default), 0 missed = MSI 100.**
- **Gate** `scripts/gates.sh --diff` → **GATE GREEN [diff] on the FIRST attempt** (15/15): coverage 100%
  lines (40575 lines, 0 missed; config.rs + orchestration.rs 100%), mutation 6 caught / 0 missed = MSI 100,
  docs ✓, miri + visual/AX ✓. Commit receipt written. (The #370 lessons — plain asserts, no unreachable
  arms, exhaustive matches — gave a clean first pass here.)

**Status: Phase 4 — Validate PASS.**
### Driven capture
**N/A — no UI.** #371 is pure config plumbing (settings tables + resolvers; `settings.toml` is the interface,
no editor/render/input surface). No pixels to capture.

## Phase 5 — Complete
