---
pipeline_id: bc3a2368-f88c-4339-89e7-251feceadf44
ticket: forge#371 (ec7757ce-0d0c-4f4c-b7ac-978fa676a5df) · local docs/planning/tickets/open/TICKET-371-mcp-servers-settings.md
aar_id: 7c5631a2-b78b-428a-b5b7-b9fe5fb62ca4
status: Phase 5 — Complete PASS (shipped 2026-07-21; gate GREEN --diff first attempt, 15/15)
title: Settings — [[mcp.servers]] + per-project orchestration config (brain endpoint + web URL) round-trip
type: feature
milestone: M23
references: [docs/marley_architecture/orchestration-shell.md §2/:41-63 agnosticism · §7/:163 settings row · §8/:172-178 per-project URL · §10/:219-237 auth posture · §12/:267-273 Layer-1 slice ⑤, crates/marley_app/src/settings.rs:80-85 lsp.servers define_setting / :70-74 remote.hosts / :75-79 workflows / :86-97 root-keyed recents+collapsed / :200-201+:352+:392 AppliedSettings wiring / :458-464 test-only persist / :744-770+:803-854+:856-917 the three tolerance tests, crates/marley_lsp/src/config.rs:13-22 LanguageServerConfig (serde default :20-21), crates/marley_app/src/workflows.rs:10-22 Workflow (serde default :20-21), crates/marley_remote/src/lib.rs:134-141 RemoteHost, crates/marley_app/src/grid_layout.rs:228-241 ProjectLayout/ShellLayout + :248 breaks_framing, crates/marley_forge_client/src/lib.rs:107-126 forge_endpoint_from (.mcp.json today), crates/marley_settings/src/value.rs:14 from_file_value→.ok()]
---

## Title
**The fleet control plane's wiring layer (orchestration-shell §12 Layer-1 slice ⑤).** Two new
hand-editable settings tables, built exactly on the shipped `[[lsp.servers]]` idiom (#308): a
`[[mcp.servers]]` array-of-tables (name · stdio/http transport · enabled · the per-server permission
grants sibling #370's `marley_mcp` reads) and a per-project orchestration table (each project root →
its brain MCP endpoint, which sibling #368's subscription reads to know where to subscribe, + its web
URL, the future Phase-E Forge-pane target). Full load/persist/hand-edit-tolerant round-trip, pure
seam at cov/MSI 100. **Wiring only — no live connection behavior.** The schema is deliberately
project-agnostic: a non-Forge project points both fields anywhere; nothing UCSOS-specific anywhere.

## Scope

### In
- **`[[mcp.servers]]` setting** — `McpServers: Vec<McpServerConfig>` via `define_setting!` in
  `crates/marley_app/src/settings.rs`, exactly the #308 placement (settings.rs:80-85):
  - `McpServerConfig { name: String, command: Option<String>, args: Vec<String>, url: Option<String>,
    enabled: bool (defaults TRUE), allow: Vec<String>, allow_write: Vec<String> }` — every
    non-identity field `#[serde(default)]` (the #204 lesson, verbatim); `enabled` needs
    `#[serde(default = "…")]` returning `true` (plain `#[serde(default)]` on a bool gives `false`).
  - A PURE per-entry transport resolver: `command` present → Stdio(command, args); `url` present →
    Http(url); neither/both → a typed per-entry error (§14 — no panic), siblings unaffected.
  - `allow` (tool classes) / `allow_write` (write-verb allowlist) are CARRIED opaquely — stored,
    round-tripped, and exposed via a pure accessor; their semantics belong to #370.
- **Per-project orchestration setting** — `ProjectOrchestration { root: String,
  brain_endpoint: Option<String>, web_url: Option<String> }` (both option fields
  `#[serde(default)]`), keyed by project root; a pure resolver
  `orchestration_for(configs, root) -> Option<…>` (exact root match; unconfigured → `None`).
  The persisted landing (global keyed table vs project persistence) is D-OPEN-1.
- **Round-trip wiring** — new `AppliedSettings` fields + `applied_from`/`applied_defaults` lines
  (mirroring settings.rs:200-201/:352/:392); persist helpers `#[cfg(test)]` (the #308 posture,
  settings.rs:458-464 — Marley never writes these in this slice; the file is the interface).
- **Tests** — the #308 test shape verbatim (settings.rs:856-917): identity round-trip, absent key →
  empty default, malformed value → empty default (no panic), missing-optional-keys entry → declared
  defaults (not a wipe), `Default::default()` == the all-keys-absent decode, resolver units.

### Out (explicitly deferred)
- **Any live MCP connection** — no socket, no spawn, no subscription; #368/#370 consume this wiring.
- **Any UI.** Settings has NO editor surface today — `settings.toml` is file-based/hand-edited (the
  settings.rs:39-40 posture: "the settings file IS the interface"); both new tables are hand-edited
  in v1. No launcher/palette editor ships here.
- **Permission ENFORCEMENT** — grants are carried, never interpreted here (#370's turf).
- **The browser pane itself** (Phase E) — `web_url` is stored, not rendered.
- **Credential/auth fields** — deliberately absent from the v1 schema (D4).
- Deleting/editing entries from inside Marley; any non-TOML config source.

## Reference (§20)
**N/A — Marley-specific config plumbing; no reference-app behavior analog.** This is a settings-table
addition to Marley's own local-TOML framework (`marley_settings`, itself a clean-room
`[Marley-original]` rebuild — docs/marley_architecture/settings.md "Provenance"), wiring a
Marley-original architecture (the fleet control plane, docs/marley_architecture/orchestration-shell.md).
There is no observable Warp/Zed behavior to match: no Warp/Zed source read, no behavior capture needed.
The only external SHAPE consulted is the published MCP client-config convention (Prior art leg 2).

### Prior art
1. **Behavior maps (docs/warp_architecture, docs/zed_architecture): N/A — config plumbing.** There is
   no terminal/editor behavior here to observe or match; nothing in the maps covers a client's own
   MCP-server config file. Recorded as checked-and-not-applicable, not skipped.
2. **Published material — the MCP client-config ecosystem convention.** Marley's own repo-root
   `.mcp.json` (read for SHAPE ONLY — no token/credential value copied anywhere) is a live example of
   the convention Claude Code and other MCP clients share: a map of server name → either
   `{type: "http", url, headers?}` or `{command, args}`. The v1 TOML schema matches that vocabulary
   where sensible (`name`/`command`/`args`/`url`) so a user who knows `.mcp.json` can hand-write
   `[[mcp.servers]]` unaided — with one deliberate divergence: NO headers/credential field (D4). The
   MCP spec's transport vocabulary (stdio · Streamable HTTP) is the source of the two-shape
   requirement; the forge server itself is Streamable-HTTP (orchestration-shell.md §6/:139-141).
3. **Our own deps + shipped seams (the highest-yield leg).** The settings file's three shipped
   array-of-tables precedents ARE the spec: `[[lsp.servers]]` #308
   (settings.rs:80-85; struct `LanguageServerConfig` crates/marley_lsp/src/config.rs:13-22 with
   `#[serde(default)]` args :20-21; tolerance test settings.rs:856-917; test-only persist :458-464),
   `[[remote.hosts]]` #87 (settings.rs:70-74; `RemoteHost` crates/marley_remote/src/lib.rs:134-141;
   tolerance test :744-770; the drop-a-bad-entry-not-the-table posture lib.rs:152-154),
   `[[workflows]]` #204 (settings.rs:75-79; `Workflow` crates/marley_app/src/workflows.rs:10-22 with
   `#[serde(default)]` params :20-21; the missing-key-must-not-wipe test settings.rs:838-854).
   Root-keyed per-project state is also already shipped: `workspace.recents` #234 (settings.rs:86-91)
   and `rail.collapsed` #245 (settings.rs:92-97 — "keyed by root, NOT the shifting index").
   serde/toml already own the tolerance seam (`marley_settings`' blanket `SettingsValue` over
   `Serialize + DeserializeOwned`, from_file_value → `.ok()` at crates/marley_settings/src/value.rs:14)
   — no hand-rolled partial decoding is needed or wanted.

## Locked-In Decisions
- **D1 — The #308 idiom verbatim.** `define_setting!` `Vec<T>` tables in
  `crates/marley_app/src/settings.rs`, `#[serde(default)]` on every non-identity field, hand-edited
  only in this slice → persist helpers `#[cfg(test)]` (settings.rs:458-464), the same test shape
  (identity round-trip · absent → default · malformed → default, no panic · missing-key → no wipe).
- **D2 — Transport = flat permissive fields + a pure resolver, NOT a serde-tagged enum.** The
  framework decodes the WHOLE Vec or falls back to the default
  (crates/marley_settings/src/value.rs:14 `.ok()`), so a tagged enum would let one typoed
  `transport = "htpp"` wipe the runtime view of every server. Flat `command`/`url` Options keep the
  #204 tolerance at the serde layer, and the pure resolver rejects a semantically-invalid entry
  (neither/both set) PER-ENTRY with a typed error while siblings survive — the #87 drop-the-bad-entry
  posture (marley_remote/src/lib.rs:152-154).
- **D3 — Schema vocabulary matches the MCP ecosystem convention** (`name`/`command`/`args`/`url`,
  Prior art leg 2) so hand-writing an entry needs no Marley-specific knowledge. Shape only; no value
  from `.mcp.json` is copied anywhere.
- **D4 — NO credential/header fields in v1.** `settings.toml` is plaintext hand-edited config; a
  headers map is where an `Authorization` bearer sneaks in. The forge bearer stays where it lives
  today — the gitignored `.mcp.json` (`forge_endpoint_from`,
  crates/marley_forge_client/src/lib.rs:107-126; the never-logged-bearer lesson) — and
  orchestration-shell §10 already frames auth as a separate reconciled concern. Revisit only when a
  remote non-local brain demands it, as its own ticket.
- **D5 — Generic vocabulary only (the agnosticism face).** Field names are `brain_endpoint` /
  `web_url` / `allow` / `allow_write` — no "forge"/"ucsos" token in any key, field, or default
  (orchestration-shell §2: a Forge-specific string outside the adapter is a defect). Verified by
  review (REQ-006).
- **D6 — `enabled` defaults TRUE, and `Default` is kept in lockstep with serde.** `enabled` uses
  `#[serde(default = "…")]` (a bare bool default would flip the meaning of an omitted key). Each new
  struct gets a `Default` whose value EQUALS the all-keys-absent serde decode, pinned by a test —
  both for hand-edit semantics and because body-replacement mutants on fns returning these types are
  only viable with a `Default` in scope (the #203/#204 viability lesson).
- **D7 — Grants are carried, not interpreted.** `allow`/`allow_write` are opaque string lists with a
  pure read accessor; #370 owns their meaning. #371 makes no permission decision.

## Open Decisions (decide in Phase 2)
- **D-OPEN-1 — WHERE per-project orchestration config persists.** Evidence gathered: Marley has NO
  per-project persisted CONFIG today. The only per-project persisted state is (a) the
  `workspace.shell` LAYOUT blob (`ProjectLayout{root, active_tab, tabs}`,
  crates/marley_app/src/grid_layout.rs:228-241) — a custom line/tab-framed codec inside one global
  settings string, rewritten on every layout persist, whose `breaks_framing` (:248) silently DROPS
  unrepresentable entries — hostile to hand-editing config; and (b) root-keyed GLOBAL settings —
  `workspace.recents` #234 (settings.rs:86-91) + `rail.collapsed` #245 (settings.rs:92-97), both
  keyed by project root string precisely because indices shift. **Recommended landing:**
  `[[projects.orchestration]]` array-of-tables in global `settings.toml`, entries keyed by `root`,
  resolved by exact root match — the shipped array-of-tables idiom (D1) composed with the shipped
  root-keying precedent. Alternatives, honestly held open: extending the shell codec (rejected-leaning:
  couples config to layout state + framing fragility + not hand-editable) · a per-project in-repo
  file (e.g. `.marley.toml` — no precedent, a second persistence backbone, pollutes user repos).
- **D-OPEN-2 — struct ownership under concurrent siblings.** The single-owner rule (§14) + the #308
  precedent put a config struct in its DOMAIN crate (`LanguageServerConfig` lives in `marley_lsp`,
  config.rs:13-22) — so `McpServerConfig`'s natural owner is #370's NEW `marley_mcp` crate, which is
  being built in this same sprint; `marley_app` cannot own it (`marley_mcp` can't depend on the app
  crate — dependency direction). `ProjectOrchestration`'s candidates: `marley_project` (the pure
  project-domain crate — currently serde-free) vs app-local (the `Workflow` precedent,
  workflows.rs:10-22 — fine only while consumers stay app-side; #368's `marley_forge_client`
  consumption may force the lower home). Phase 2 decides against the siblings' actually-landed state.

### Planner sharpenings (Phase 1 review — the sibling #370 has since SHIPPED)
- **S1 — D-OPEN-2 is now resolvable by landed state: `McpServerConfig` → `marley_mcp`, closing the S1 loop.**
  #370 SHIPPED with `marley_mcp::GrantTable { write_classes: BTreeSet<String> }` + `GrantTable::from_classes`
  (permission.rs:13/21). Per the #308 domain-crate precedent (`LanguageServerConfig` lives in `marley_lsp`),
  `McpServerConfig`'s natural home is `marley_mcp` (a new `config` module; `marley_app` already deps
  `marley_mcp`), and its `allow_write: Vec<String>` maps to a permission grant via a PURE
  `grants_for(&McpServerConfig) -> GrantTable` (= `GrantTable::from_classes(allow_write)`) that marley_mcp
  owns — so #370 OWNS the permission type and #371's config PRODUCES it (the S1 contract from #370's plan,
  now concrete). `allow` (read classes) is carried opaquely (D7) — #370's read tier is loose, so `allow` is
  forward-looking, not yet consumed.
- **S2 — D-OPEN-1's recommendation is grounded in two SHIPPED idioms.** `workspace.recents` (#234) +
  `rail.collapsed` (#245) are shipped root-keyed global tables (settings.rs:86-97), so
  `[[projects.orchestration]]` as a global array-of-tables keyed by `root` composes the array-of-tables idiom
  (D1) with the shipped root-keying precedent — the shell-codec extension stays rejected (framing fragility +
  not hand-editable). `ProjectOrchestration` lives app-local (settings.rs — the `Workflow` precedent):
  consumers get PRIMITIVES (the app resolves `orchestration_for(root)` and hands `brain_endpoint`/`web_url`
  strings to #368 / the Phase-E pane), so no lower-crate home is forced.
- **S3 — consumption completeness (Phase-2 scope call).** #370's shipped `start_mcp_server` currently uses
  `GrantTable::default()` (empty → deny-all). #371 can COMPLETE the wiring by having the app read the
  configured grants (via `grants_for`) when starting a named server — a small masked-glue change, still "no
  live connection" (the server only runs on the opt-in `mcp-serve` verb). Include-vs-defer is Phase-2's call;
  the pure config + resolver + accessor is the core regardless.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `[[mcp.servers]]` entries are persisted and the manager is reloaded from the same dir, the system shall yield the identical `Vec<McpServerConfig>` (identity round-trip, including grants + args), and the same for the per-project orchestration table. | settings round-trip units (the #308 shape, settings.rs:856-917) |
| REQ-002 | WHEN a hand-edited entry omits optional keys (`args`, `enabled`, `allow`, `allow_write`, `brain_endpoint`, `web_url`), the system shall load that entry with its DECLARED defaults (empty lists, `enabled = true`, `None` endpoints) and shall NOT drop the table's other entries. | unit: hand-written TOML missing keys → decoded entry + surviving siblings (#204 lesson, settings.rs:838-854 shape) |
| REQ-003 | The schema shall represent BOTH transport shapes: a stdio entry (`command` + optional `args`) and an http entry (`url`), each resolving to its typed transport via the pure resolver. | pure resolver units (one per shape) |
| REQ-004 | WHEN an entry is semantically malformed (neither `command` nor `url`, or both), the resolver shall return a typed per-entry error — no panic (§14) — and sibling entries shall still resolve; WHEN the whole setting value is type-malformed (e.g. a string where an array belongs), the load shall yield the empty default, not a panic. | pure resolver units + the malformed-value unit (settings.rs:892-898 shape) |
| REQ-005 | WHEN the active project's root has an orchestration entry, `orchestration_for` shall resolve exactly that entry; WHEN it has none, it shall return `None` (an unconfigured project degrades to today's behavior). | pure resolver units (configured · unconfigured · multi-entry pick) |
| REQ-006 | The persisted schema shall contain NOTHING UCSOS/Forge-specific — every table, field name, and default is generic (a non-Forge project can point `brain_endpoint`/`web_url` anywhere). | review (Phase 3.5 inspect checklist item; D5) |
| REQ-007 | WHEN a consumer asks for a named server's grants, a pure accessor shall return its `allow`/`allow_write` lists (the call #370 makes), without interpreting them. | pure accessor unit |
| REQ-008 | The new settings seam (structs, resolvers, accessors, schema wiring) shall be pure — no IO beyond the existing `*_in(dir)` manager path — at cov 100 / MSI 100. | gate:4/gate:5 exit codes on the touched files |

## Phase Plan
- **P2 Design** — settle D-OPEN-1/D-OPEN-2 against the sibling pipelines' landed state (#368/#370);
  exact struct + resolver signatures; the `define_setting!` keys; the test manifest per REQ row;
  survey the REAL mutant set with `cargo mutants --list -f <touched files>` — never guessed operators
  (PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators; and the syntactic-form rule:
  the viable set depends on the actual code shape, including Default-derive viability per D6).
- **P3 Implement** — per design; schema + structs + resolvers + `AppliedSettings` wiring + test-only
  persist helpers. No live connection code.
- **P3.5 Inspect** — independent critics vs the diff; REQ-006 agnosticism review; the #204
  wipe-check on every new field; fix real findings.
- **P4 Validate** — write + RUN the planned tests; re-run `cargo mutants --list -f` on the ACTUAL
  shipped code and kill the real set; gate green (cov/MSI 100 on the pure seam).
- **P5 Complete** — archive; update docs/marley_architecture/settings.md (+ orchestration-shell §7
  row status); AAR capture; close #371.
