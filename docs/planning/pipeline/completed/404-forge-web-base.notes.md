# forge_web_base — the .mcp.json → Forge web-origin derivation seam — Notes

- **Forge ticket:** #404 e84add90-0452-4bfa-9772-4ca04a54d5ae
- **AAR:** 62602a55-22db-42ed-9036-2ef6400646d1 (opened 2026-08-07 at promotion; no pre-flagged codes)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-404-forge-web-base.md
- **Pipeline spec:** 404-forge-web-base.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch). -->

## Phase 1 — Plan (2026-08-06, /spec batch — M29 sprint #40)
- **Request:** embedded-browser-model.md **Q3** names the endpoint→web-base step "a slice-3 input,
  not yet pinned". Pin it as its own pure ticket ahead of the pane: derive the Forge web-UI base
  (origin) from the one #381 `.mcp.json` resolution; #405 wires the consumer.
- **Classification / tier:** feature, **S**, PURE (one fn + units; no UI, no masked surface
  expected, no consumer wiring). React-first N/A — no UI delta.
- **Forge recall (§18.3):** `bulletin-list` → **empty** (no active bulletins).
  `knowledge-search "mcp_config active root .mcp.json resolution restored project"` → ranked ids
  only (top: architecture_decisions 8777d021-…, 723c3714-…; prevention_rules 22e91bec-…,
  452ae5ee-…, d2f85b7b-…) — the search surface returns no bodies; the governing lineage is already
  named in the ticket brief and verified in-tree
  (`PR-claude-boot-decisions-key-the-restored-active-root-001` ↔ mcp_config.rs's active-root-wins
  contract). `knowledge-search "bearer never logged URL loopback wall"` → top structural hit
  prevention_rule 004a7a9f-… (+ ADs cf751352-…, 22585523-…) — consistent with the #370/#375
  bearer-never-logged lineage the tree enforces structurally (ForgeEndpoint Debug-redaction).
  `docs-search "Forge web UI origin browser pane URL derivation"` → returned ANOTHER project's
  corpus chunks (hub-nav / backpack-tabs tickets) — **no Marley yield; recorded as cross-project
  index noise**, the design doc itself (embedded-browser-model.md) is the authority and was read
  directly.
- **Discovery (verified against the tree, 2026-08-06 — today's lines; drift from the M26 design doc
  recorded):**
  - **`mcp_config.rs` (137 lines total):** `mcp_json_path(active_root, cwd, exists) → Option<PathBuf>`
    (:28-40) — pure over PATHS, charter-explicit "never reads, parses, logs, or holds file contents"
    (:11-12), so the DERIVATION cannot live there. Active root WINS; cwd is a missing-file-only
    fallback; a present-but-unparseable active-root file is still returned so it "surfaces AS
    misconfigured downstream" (:20-24). Seven units incl. `both_present_active_root_wins` (:81-92)
    — the composed REQ-002 unit mirrors it.
  - **The two shipped consumers (app.rs — the doc's Q3 cites :2207/:2220, M26 lines; TODAY):** the
    single resolution at :2400-2406 (post-shell-restore `active_root`/`cwd`, `mcp_json_path` :2403,
    `read_to_string` → `mcp_json: Option<String>`); **consumer 1** the forge MCP client :2407-2410
    (`forge_endpoint_from` → `ForgeClient::new`); **consumer 2** the fleet brain decision :2426-2428
    (`endpoint_for_brain(url, mcp_json.as_deref())` — url from settings `subscription_target`
    :2420-2425, the `.mcp.json` string feeds the bearer-iff-url-match).
  - **The parse + wall (marley_forge_client/src/lib.rs):** `forge_endpoint_from(&str) →
    Option<ForgeEndpoint>` :124-141 — serde_json parse, `mcpServers.forge.url` :126-127, loopback
    wall :131-134, and it REQUIRES `headers.Authorization` present :135-139 (a url-only config
    yields None today — feeds D-OPEN-INPUT-SEAM's recorded consequence). `endpoint_for_brain`
    :149-163 (same wall :150-153). `is_loopback_authority` :168-178 (hand-parse: IPv6 brackets,
    `rsplit_once(':')`, `localhost`/`IpAddr::is_loopback`) — incidentally refuses userinfo forms
    (`user:pass@host` parses as a non-loopback authority; D5 pins this deliberately). `split_url`
    :182-188 — **`http://`-ONLY** (https refused). `ForgeEndpoint` fields PRIVATE, bearer
    Debug-redacted `"***"` :38-49; pub `url()` accessor :52-57 (the ride-the-endpoint input).
  - **The loopback wall truth (the brief's open question — CHECKED, not genuinely open):** the wall
    lives in marley_forge_client, applied inside BOTH endpoint constructors; its stated rationale is
    bearer-egress ("the bearer only ever goes to the LOCAL forge", :128-130) — i.e.
    MCP-endpoint-construction-specific, NOT a general egress wall (marley_mcp's `is_loopback`,
    auth.rs:7, is the SERVER bind-side twin). Q3 D3 pins "the wall stands" for the browser URL;
    riding the constructed endpoint makes that inheritance structural (non-loopback → no client AND
    no web base). No standalone D-OPEN; folded into D-OPEN-INPUT-SEAM.
  - **The url-crate truth (§20 leg 3 — the highest-yield finding):** **`url` 2.5.8 IS in Cargo.lock**
    (:6396-6398), transitively via `gpui_http_client`, `zed-reqwest`, `ashpd`, `zvariant`, `git2`;
    **direct dep of NO workspace crate** (no Cargo.toml hit; marley_forge_client deps =
    serde/serde_json/marley_fleet). So adopting it adds zero new supply-chain weight (already
    license-vetted in the graph), and `Url::origin().ascii_serialization()` is the WHATWG anchor
    implemented (incl. default-port elision). In-house promotion precedent: `unicode-width`
    (#277, marley_app/Cargo.toml — "already in the lock transitively"). Locking a hand-parse here
    would risk the #339-regex pattern → D-OPEN-URL-CRATE, recommend ADOPT.
  - **The #384 sibling (fleet_live.rs):** `classify_fleet_setup` :98-127 (first-failing-gate names
    ONE reason), `FleetSetup` :28-37, `Misconfig` with PRIVATE clamped reason :41-48 ("Never bearer
    material" :46), arms :69-81. THIS seam is its sibling but presentation-FREE this slice:
    `Option`, no reason string, no render — the empty rail home is #385's section + #403's
    placeholder (sibling M29 ticket).
  - **Train neighbors (this /spec batch):** #403 = slice 2 (`TabContent::Browser` + bare-marker
    codec + placeholder — the doc's Q4 `B=` tag, re-derive-on-restore); #405 = slice 3 proper (the
    wry pane + consumer wiring, loads THIS ticket's origin). Browser rail home today:
    `RailSection::Browser` tabs.rs:51/:60, cockpit-resident mapping :131 (the doc's ":82" is M26
    drift).
- **Decisions:** D1 one-resolution-three-consumers (keyed on restored active root; today-lines
  cited); D2 origin-only per WHATWG (path dropped); D3 bearer never in URL/page JS (sentinel
  negative unit); D4 unresolved → None → no Browser content (presentation-free #384 sibling); D5
  hostile-input totality (§14). D-OPENs for Phase 2: URL-CRATE (recommend adopt `url` — in-lock
  transitive, WHATWG serialization for free), INPUT-SEAM (recommend ride `forge_endpoint_from` →
  `ForgeEndpoint::url()` — one parse one wall; recorded consequence: bearer-less config → None),
  DEFAULT-PORT (recommend WHATWG elision; #405's nav pin must compare origins not strings). P2
  validates against the REAL Forge instance: the web UI's actual origin (escape-hatch trigger) +
  whether it authenticates itself (Q3 D3 assumption).

### Promotion to active (2026-08-07, /work 404)
- Queued pair → `docs/planning/pipeline/active/` (git mv); no other active pipeline (checked).
- Forge ticket #404 claimed (owner `c484e8af-a328-4d8e-9534-b0e2deacca12`, previous owner null);
  AAR opened `62602a55-22db-42ed-9036-2ef6400646d1` (no pre-flagged codes).
- **Prior-art sweep re-verified against today's tree:** `url` 2.5.8 in Cargo.lock (line :7326 —
  the spec's :6396 cite is lock-churn drift; claim holds), direct dep of NO workspace crate
  (grep over all Cargo.tomls), marley_forge_client deps = serde/serde_json/marley_fleet (+ dev:
  mutants, tempfile). `mcp_config.rs`, `forge_endpoint_from`/`endpoint_for_brain`, and the
  app.rs:2400-2428 consumer sites re-read and match the Discovery lines above.
- Environment pre-flight (from /work): cargo 1.96.0, gates.sh OK, cargo-mutants 27.1.0,
  cargo-llvm-cov 0.8.7, hooks wired, forge wired, marley-web OK. Bulletins: none.

## Phase 2 — Design (2026-08-07)

### D-OPENs settled (evidence cited)
- **D-OPEN-URL-CRATE → ADOPT `url = "2"`** (direct dep of marley_forge_client; resolves to the
  in-lock 2.5.8 — zero new packages). Evidence: read `url-2.5.8/src/origin.rs` (Apache/MIT —
  adoption leg, outside the wall): `url_origin` returns `Origin::Tuple(scheme, host, port)` for
  http (:27-31); `ascii_serialization` (:78-89) implements the WHATWG rule verbatim — default
  port ELIDED (`default_port(scheme) == Some(port)` → `scheme://host`, :82-83), else
  `scheme://host:port`; opaque origins serialize to the STRING `"null"` (:80) — so the derivation
  MUST guard `is_tuple()` or a non-tuple arm would yield a poison `Some("null")`. Host
  canonicalization comes free (IPv6 via std `Ipv6Addr` Display → `[0:0:0:0:0:0:0:1]` → `[::1]`).
  Hand-parse equivalence would need bracket-aware port split + default-port elision + IPv6
  canonicalization re-rolled against an in-tree owner — the #339 pattern; refused.
- **D-OPEN-INPUT-SEAM → ride `forge_endpoint_from` → `ForgeEndpoint::url()`.** One JSON parse,
  one wall (loopback + http-only, applied at construction), bearer stays behind the private
  field. Consequence CONFIRMED as one-config-coherence: a url-only bearer-less config yields
  `None` (it builds no MCP client either) — pinned by a named unit.
- **D-OPEN-DEFAULT-PORT → WHATWG ASCII elision** (free under ADOPT; origin.rs:82-86). Recorded
  for #405: the pinned-origin nav check must compare ORIGINS (canonical serializations), never
  raw strings.

### Real-instance validation (2026-08-07, the operator probe)
- This machine's `.mcp.json`: endpoint `http://127.0.0.1:8080/mcp/forge` → derived origin
  `http://127.0.0.1:8080`.
- **Unauthed** `GET /` → **401** JSON `{"error":"missing or malformed Authorization header"}` —
  the Q3 D3 "web UI authenticates itself" assumption does NOT hold at this origin…
- **Authed** (bearer via header file, never argv/logged) `GET /`, `/ui`, `/ui/`, `/web`, `/app`,
  `/index.html`, `/dashboard` → **all 404**. …because **no web UI exists at the MCP origin
  today at all** — the sidecar is API-only behind bearer middleware.
- Consequences: (a) #404 is UNBLOCKED — the seam pins the contract for when the UI ships;
  (b) the **escape hatch stays OUT** — no path/port mismatch evidence exists (nothing serves
  anywhere); (c) the pane (#405) will honestly render the 401 JSON on mount today — #406's
  error-state work owns making that graceful; (d) the D3 fallback stance stands recorded: if the
  UI later expects the bearer, the pane renders unauthenticated — the gap closes later, never
  via a URL bearer. **Carry (a)–(c) into #405/#406 design.**

### Architecture / approach (§14, §20)
- **Signature + home** (settled): in `crates/marley_forge_client/src/lib.rs` beside
  `forge_endpoint_from` —
  ```rust
  /// The Forge web-UI base for a resolved `.mcp.json` string: the wall-admitted MCP
  /// endpoint's WHATWG origin (ASCII serialization — scheme+host+port, path dropped,
  /// scheme-default port elided), or `None` on every unresolved/refused arm.
  pub fn forge_web_base(mcp_json: &str) -> Option<String>
  ```
  Body: `web_origin_of(forge_endpoint_from(mcp_json)?.url())`. Private helper:
  `fn web_origin_of(url: &str) -> Option<String>` = `Url::parse(url).ok()?` → `.origin()` →
  `origin.is_tuple().then(|| origin.ascii_serialization())` (the poison-`"null"` guard; §14
  total, no unwrap/expect on any config-derived path — in-crate unit tests reach the private
  fn directly, including the opaque arm the wall makes unreachable in production).
- **Return type**: `Option<String>` (the WHATWG ASCII serialization is canonical, so string
  equality IS origin equality). No newtype this slice — #405 introduces one iff its nav
  callback wants typed compare. Reversible; recorded.
- **§20 confirm**: still N/A — Marley-specific seam; the published WHATWG anchor + a
  permissive-dep adoption (`url` is Apache/MIT). No Warp/Zed behavior observed or needed; the
  copyleft wall untouched.
- **§14**: typed `Option` totality; no panic arms; no IO in the seam (caller reads the file —
  the mcp_config charter split holds); no shared-type changes; no process spawn.

### File manifest
| # | File | Change |
|---|---|---|
| 1 | `crates/marley_forge_client/Cargo.toml` | + `url = "2"` under `[dependencies]` (promotes the in-lock transitive 2.5.8 to direct — the #277 `unicode-width` precedent; gate:8 already vets it) |
| 2 | `crates/marley_forge_client/src/lib.rs` | + `forge_web_base` (pub) + `web_origin_of` (private) + module-doc line; `#[cfg(test)]` units T1–T6, T9–T15 |
| 3 | `crates/marley_app/src/mcp_config.rs` | **tests only** (module charter "pure over PATHS" untouched): composed units T7–T8 importing `marley_forge_client::forge_web_base` (marley_app already depends on the crate) |

No app.rs touch; no consumer wiring (#405's); no UI (React-first N/A holds).

### Regression Test Plan (per REQ)
| # | Test (home) | Proves |
|---|---|---|
| T1 | `web_base_drops_mcp_path` — `…:8080/mcp` fixture → `Some("http://127.0.0.1:8080")` (fc) | REQ-001 |
| T2 | `web_base_drops_nested_path` — `…/mcp/forge` → same origin (fc) | REQ-001 |
| T3 | `web_base_pathless_endpoint` — `http://localhost:9000` → itself (fc) | REQ-001 |
| T4 | `web_base_default_port_elided` — `:80/mcp` → `Some("http://127.0.0.1")`; portless input unchanged (fc) | REQ-001 + D-DEFAULT-PORT |
| T5 | `web_base_bracketed_ipv6` — `[::1]:8080/mcp` → `Some("http://[::1]:8080")` (fc) | REQ-001 |
| T6 | `web_base_canonicalizes_ipv6` — `[0:0:0:0:0:0:0:1]:8080` (wall admits via `IpAddr`) → `Some("http://[::1]:8080")` (fc) | REQ-001/005 |
| T7 | `active_root_config_feeds_web_base` (mcp_config tests) — both roots hold configs with different ports; the resolved path picks the ACTIVE root's content → its origin (mirrors `both_present_active_root_wins`) | REQ-002 |
| T8 | `no_candidate_file_yields_no_web_base` + unreadable-file arm (mcp_config tests) — the app.rs Option-chain mirrored end-to-end → `None` | REQ-003 arms 1–2 |
| T9 | `unparseable_json_is_none` (fc) | REQ-003 arm 3 |
| T10 | `missing_forge_entry_or_url_is_none` — `{}`, no `mcpServers`, no `forge`, no `url` (fc) | REQ-003 arm 4 |
| T11 | `wall_refused_url_is_none` — non-loopback host; `https://`; `HTTP://` mixed-case; userinfo `user:pass@127.0.0.1` (fc) | REQ-003 arm 5 + D5 userinfo pin |
| T12 | `bearerless_config_is_none` (fc) | D-INPUT-SEAM consequence |
| T13 | `derived_origin_carries_no_bearer_bytes` — sentinel `Authorization` → `Some(origin)` with `!origin.contains(sentinel)` (fc) | REQ-004 / D3 |
| T14 | `hostile_inputs_are_total` table — garbage/empty/multi-KB JSON; empty url; `http://`; malformed `[::1`; `:99999` (wall admits, `Url::parse` refuses → `None` — divergence PINNED); `:0` (outcome pinned at implement); `localhost.` trailing dot → `None`; `127.1` abbreviated → `None` (fc) | REQ-005 |
| T15 | `web_origin_of_opaque_scheme_is_none` — direct private-fn unit, `data:` scheme → `None` (kills the `is_tuple` guard mutant) (fc) | REQ-005 / totality |
| T16 | `scripts/gates.sh --diff` green; `cargo mutants --list -f` over the touched file(s) → zero missed | REQ-006 |

Uncoverable-by-unit: **none** — pure seam end to end; no GUI, no bound port, no live forge in
any test. (fc = `marley_forge_client/src/lib.rs` `#[cfg(test)]`.)

### Risks / decisions (reversible-but-load-bearing)
- **Derivation strictly narrower than the wall** (T14 pin): the wall's host-only loopback check
  admits `:99999`, `Url::parse` refuses it → web base `None` while `ForgeClient` constructs.
  Deliberate — no page could load from an unparseable-port origin; consumers stay coherent on
  every REAL origin.
- **Lock churn**: `url = "2"` must reuse 2.5.8 — verify the Cargo.lock diff adds only the
  dependent edge on `marley_forge_client` (no version bumps). If the resolver moves anything,
  pin `url = "=2.5.8"` instead and record why.
- **Return-type reversibility**: `Option<String>` now; a `WebOrigin` newtype is #405's call.
- **Real-instance finding** (above) is the design's biggest carry-forward — recorded for
  #405/#406.

## Phase 3 — Implement (2026-08-07)
- **React-first: N/A** — no UI delta (spec §React-first; recorded, not skipped).
- Built exactly the Phase 2 manifest, no deviations:
  1. `marley_forge_client/Cargo.toml` — `url = "2"` promoted transitive→direct (comment cites
     #404 + the #277 precedent). **Lock churn: ONE line** (`+ "url",` in marley_forge_client's
     dep list; url stays 2.5.8, zero version bumps — the risk row's clean outcome).
  2. `marley_forge_client/src/lib.rs` — `pub fn forge_web_base(&str) -> Option<String>` riding
     `forge_endpoint_from` + private `fn web_origin_of(&str) -> Option<String>` with the
     `is_tuple()` poison-`"null"` guard; module-doc PURE-layer line gains "web-base derivation".
     Placed between `endpoint_for_brain` and `is_loopback_authority` (the pure-parse
     neighborhood). No unwrap/expect; total on any input.
- `cargo check -p marley_forge_client` + `--workspace` green (pre-existing `block v0.1.6`
  future-incompat note only); `cargo fmt` clean. mcp_config.rs untouched (its rows are
  Phase 4 test-only). No consumer wiring (per scope).

## Phase 3.5 — Inspect (2026-08-07)
- **Lenses (2 critics + the inspector's own pass):** A = hostile-parser correctness (scratch-crate
  empirical, ~40 inputs); B = D3 bearer walk + §20 provenance + reuse/clippy. The inspector
  independently formed + empirically confirmed finding 1 before the critic reports landed.
- **Findings ledger:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | 1 | **HIGH** | **Wall-admit vs WHATWG-parsed-host divergence → non-loopback browser base.** `is_loopback_authority` reads the last-`:` tail as a port (no userinfo concept); WHATWG reads a last-`@` as userinfo — so `http://127.0.0.1:pass@evil.com/` (also `:8080@`, `:@`, `[::1]:pass@`, embedded-tab `\t@`, double-`@`) is wall-ADMITTED with hand-parsed host `127.0.0.1` yet `Url::parse` takes host `evil.com` → `forge_web_base` returned `Some("http://evil.com")` — #405 would navigate the webview to an attacker origin on a tampered `.mcp.json` (the wall's own stated threat model). Bearer does NOT ride the string, and the MCP-client arms are inert (raw-socket connect to such an authority fails) — the leak was the browser origin only. | **REAL** — empirically confirmed TWICE independently (inspector scratch `wallcheck` + critic A scratch, both on in-lock url 2.5.8). Critic B's contrary checked-clean row ("WHATWG never widens the wall") was reasoning-only, missed the `@`-in-port-tail shapes → **rejected**. | **FIXED** — `web_origin_of` re-asserts loopback on the PARSED host (`url::Host` match: `Domain == "localhost"` \| `Ipv4/Ipv6 is_loopback`) before serializing — the check sits on the value the origin is BUILT from. Post-fix: all 6 hostile shapes → `None`; legit origins byte-identical (incl. IPv6 canonicalization + port-80 elision). Captured: `BF-claude-wall-admit-vs-parsed-host-divergence-001` (high/security) + `PR-claude-recheck-the-parsed-value-the-consumer-uses-001`. Critic A's deeper fix — unify the WALL itself onto `url::Url` — recorded as a **follow-up hardening candidate** (it changes shipped admit behavior for the MCP client/brain arms, e.g. `:99999` configs would stop constructing → out of #404's scope). |
  | 2 | low | Wall-admitted endpoint but `None` web base (`:99999` out-of-range port, `:8080@/` empty host): an active MCP endpoint with no derivable browser base. | REAL but **BY-DESIGN** — the Phase 2 risk row pins "derivation strictly narrower than the wall" deliberately; `None` = the empty rail home, never a broken page. | No change. Carried to #405: the pane must tolerate `None` beside a live MCP client. |
  | 3 | low | D3 no-bearer property has zero in-diff tests; module-doc "unit-tested at 100%" momentarily false. | REAL but **PHASE-CORRECT** — validate owns the tests (T12/T13 + arms). | Validate lands T12/T13/T15 + the new divergence rows before `/commit`; the 100% claim is re-true then. |
  | 4 | nit | Two URL engines over one string (hand `split_url` wall + WHATWG derivation). | Acknowledged — finding 1's fix IS the mitigation for the two-engine reality (re-check on the consumed side, per the new PR). | Unify-the-wall recorded as the same follow-up candidate as #1. |
  | 5 | nit | `is_tuple` guard unreachable via the wall. | Correct-as-designed (doc'd; total-on-any-url contract). | Direct unit (T15) kills the mutant at validate. |
  | — | info | Pre-existing wall UX quirk (critic A): genuinely-loopback `http://127.1:8080` / `LOCALHOST` silently refused everywhere. Pre-dates this diff. | Out of scope; noted. | none |
- **Checked-clean (verified, not assumed):** totality zero-panic over ~40 pathological inputs incl.
  NUL/100KB-host/punycode/`[`-truncation (critic A, empirical); full D3 bearer walk — bearer field
  never read on the path, userinfo/path/query/fragment all dropped by `Origin::Tuple`
  serialization, `ParseError` variants carry no input bytes, no fixture secrets, `.mcp.json`
  gitignored (critic B); §20 provenance — url 2.5.8 LICENSE-APACHE+MIT present, 2-line compose
  over public API, no Warp/Zed material (critic B); no duplicate origin helper in-tree; clippy
  clean; lock churn = the single `+ "url",` line (critic B).
- **Validate-phase carries:** add the divergence shapes (`:pass@`, `:8080@`, `:@`, `[::1]:pass@`,
  `\t@`, double-`@`) as named unit rows proving `None` (the finding-1 regression tests); T14's
  `:99999` row stays; T15 hits the opaque arm directly.

## Phase 4 — Validate (2026-08-07)
- **Tests written (design T1–T15 + the inspect carries), all in-crate `#[cfg(test)]`:**
  - marley_forge_client (13 new units): `web_base_drops_mcp_path`, `web_base_drops_nested_path`
    (rides the shipped `MCP_JSON` fixture), `web_base_pathless_endpoint`,
    `web_base_default_port_elided` (`:80` ≡ portless — the D-DEFAULT-PORT pin),
    `web_base_bracketed_ipv6`, `web_base_canonicalizes_ipv6` (`[0:0:…:1]` → `[::1]`),
    `unparseable_json_is_none`, `missing_forge_entry_or_url_is_none`, `wall_refused_url_is_none`
    (non-loopback / https / `HTTP://` / userinfo-ahead — the D5 pin), `bearerless_config_is_none`
    (D-INPUT-SEAM), `derived_origin_carries_no_bearer_bytes` (T13 sentinel),
    `hostile_inputs_are_total` (T14 — 64KB garbage + 7 URL rows incl. the pinned `:99999`
    wall-admits/parse-refuses divergence and the `:0` port kept verbatim),
    `parsed_host_divergence_never_widens_loopback` (**the finding-1 regression** — all 6 shapes →
    `None`), `web_origin_of_arms_exact` (T15 — opaque `foo://localhost` reaches the `is_tuple`
    guard; `data:`/`mailto:` exit at `host()?`; Domain/Ipv4/Ipv6 arms in both polarities).
  - marley (mcp_config, 2 new units): `active_root_config_feeds_web_base` (T7 — both-present →
    active wins AND active-absent → cwd fallback, through the app.rs `.and_then` chain shape),
    `unresolved_or_unreadable_yields_no_web_base` (T8 — read-fails-on-resolved-path; the
    no-candidate arm is `neither_present_is_none` + Option laws, noted in-test).
- **Mutation-tightness refactor of the fix** (validate-phase, recorded): `web_origin_of` uses
  `parsed.host()?` instead of a `None => false` match arm — the arm's `false→true` mutant was
  behaviorally unkillable (no host-less URL has a tuple origin); the `?` form has no such mutant.
- **Runs (real output in transcript):** targeted `cargo nextest run -p marley_forge_client -p
  marley` → 997 passed / 2 skipped; workspace `cargo nextest run --workspace` → **2078 passed,
  5 skipped**; doctests 0 (none in scope).
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN, 15/15** (first run had 2 reds, both fixed at
  source: clippy `unnecessary_lazy_evaluations` on a test chain → `.and(None)`; coverage 100%-line
  floor caught two dead fixture arms in the composed test → restructured to the no-dead-arm slice
  lookup + the redundant arm folded into the existing `neither_present_is_none` pin).
- **Mutants (REQ-006 receipt):** gate:5 (diff) MSI 100, zero missed. `cargo mutants --list` over
  the touched files: lib.rs → 8 mutants on the new fns (2×3 body replacements, `==`→`!=` on the
  Domain arm, `delete !` on the loopback guard) — each killed by a named row
  (exact-value asserts / the localhost-vs-example.com pair / the divergence table);
  mcp_config.rs → 2 (pre-existing `mcp_json_path` bodies, still killed).
- **Live-app drive:** N/A — pure library seam, no UI/render/input path (React-first N/A);
  no consumer wiring ships this ticket.
- **Pre-existing exclusions:** none touched; the only workspace warning is the pre-existing
  `block v0.1.6` future-incompat note (mac windowing stack).

## Phase 5 — Complete (2026-08-07)
- **§21 docs:** CHANGELOG `[Unreleased] Added` entry (the seam, the ADOPT, the inspect catch, the
  no-web-UI-yet probe); embedded-browser-model.md **Q3 ticked to PINNED** — names
  `forge_web_base`, the WHATWG-origin + default-port-elision rule, the parsed-host re-check
  (+ its PR code), the hatch-stays-out probe result, and the compare-origins carry for #405.
  Parity sync N/A (non-React-first).
- **Knowledge:** AAR `62602a55…` submitted (`completed`, effectiveness 5; materialized
  BF-claude-wall-admit-vs-parsed-host-divergence-001 +
  PR-claude-recheck-the-parsed-value-the-consumer-uses-001; 2 novel findings → distillation).
  Lessons in short: the prior-art sweep's ADOPT leg paid again (in-lock `url` → WHATWG semantics
  free); the design-time REAL-INSTANCE probe prevented building #405 against an imagined web UI;
  independent empirical critics + the inspector's own scratch test caught a wall bypass a
  reasoning-only pass had waved through.
- **Ticket:** forge #404 → `done`; local doc → `tickets/closed/` (status closed).
- **Follow-up candidates recorded (not ticketed here):** unify the loopback WALL itself onto
  `url::Url` (closes the two-engine split for the MCP/brain arms too — changes admit behavior, own
  ticket); the wall's `127.1`/`LOCALhost` UX quirk (pre-existing, info).
- **Archive:** pair moved to `docs/planning/pipeline/completed/`.
- **Delivery-gate sweep catch (post-archive, recorded here for §15 truth):** the FULL gate's
  whole-workspace mutation surfaced ONE pre-existing missed mutant OUTSIDE this ticket's diff —
  `refresh_efind_matches` app.rs:15786 `ms < resume` → `<=` (the M19/#339 F5 Replace-One resume
  boundary: a match starting EXACTLY at the resume point was untested). Fixed at source per §0:
  `efind_replace_resume_boundary_keeps_adjacent_match_headless` (headless_drive) drives two
  Replace-Ones over `aaa` → asserts `bba` (the `<=` mutant yields `bab`). Scoped re-run:
  app.rs 76 mutants — 72 caught, 4 unviable, 0 missed.
