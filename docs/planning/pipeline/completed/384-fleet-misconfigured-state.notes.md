# Fleet header "misconfigured" diagnosability state — Notes

- **Forge ticket:** #384 3215851c-050a-46ff-86b4-ac06bfa8bafa (feature, sprint #36 "M25 — App-Grade QA Hardening")
- **AAR:** pending-promotion
- **Local ticket doc:** docs/planning/tickets/open/TICKET-384-fleet-misconfigured-state.md
- **Pipeline spec:** 384-fleet-misconfigured-state.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request / provenance:** #376's inspect F8, left on the shelf by design ("three silent-failure
  arms share one symptom … a 'Fleet · misconfigured' state = new UI surface, not v1" —
  376-fleet-rail-livewire.notes.md, F8 row) + the QA-2026-07-21 run: the Finder-launch find that
  became #381 (no forge client → fleet writes fail with clamped in-card reasons, the header never
  says why). Queued in the sprint #36 /spec batch (2026-07-21). **ORDERING: hard-after #381** —
  its `.mcp.json`-resolution fix decides which arm genuinely fires for the Finder-launch case
  (spec D7); promoting #384 first would ship a banner that lies on every Finder boot.
- **Classification / tier:** feature, ONE slice, small-medium. Almost entirely an extension of
  shipped pure seams (`fleet_live` presentation + `clamp_card_text` hygiene) + one masked
  boot-site population; zero `marley_forge_client` changes if the D1 recommendation holds.
- **Forge recall (§18.3):** live forge calls deferred to /work promotion (Phase-1 draft is
  docs-only); recall from the standing record: #376's locked D3 (state-ownership split: the loop
  owns wire truth, the app owns absence/config knowledge), D5 (demo verb-gated-while-live), D6
  (loopback wall + never-logged bearer carry to every new surface), F2/F4 lessons
  (`PR-claude-boot-decisions-key-the-restored-active-root-001`; render reads the pump-tracked
  field, one coherent pair), the skip-detach trap's 5th strike
  (`BF-claude-skip-detach-pump-fleet-live-001` — the edits land in exactly that neighborhood),
  `PR-claude-trace-the-real-cargo-mutants-list` (run `--list -f` on the ACTUAL files).
- **Discovery (the sweep's load-bearing evidence — file:line, all read 2026-07-21):**
  - **The pure seam this extends:** fleet_live.rs — `subscription_target` :14-18 (exact-root +
    non-empty filter; its `None` IS arm (a)), `fleet_header_title` :23-29 (exhaustive 3-arm match;
    `None`→"Fleet" byte-pin carried by t376_req002 :119-129), `demo_feed_permitted` :34-36,
    `fleet_cursor_dir` :46-55. Two `fleet_header_title` callers total (app.rs:16306 + the unit) —
    widening the signature is cheap.
  - **The crate enum that must NOT grow:** adapter.rs:177-187 — `ConnectionState { Reconnecting,
    Live }`, doc'd "published by the pump thread"; its only writers are the loop's publish sites
    (seed `Reconnecting` :202; re-publish on transient cycle end :317; `Live` :473 after the
    status-checked listen entry); accessor `connection_state()` :249-254 (poisoned → Reconnecting).
    For arm (b) no `FleetSubscription` ever exists — a `Misconfigured` variant would have NO
    legitimate publish site; it would be a dead arm in a closed wire enum (the D1 evidence).
  - **The boot start-gate site (population target):** app.rs:2192-2228 — post-shell-restore (the
    F2 comment :2192-2199), `project_count() > 0` gate :2202, `subscription_target` :2203-2206,
    the TWO silent `?` aborts: `endpoint_for_brain(&url, mcp_json.as_deref())?` :2208 (arm b) and
    `fleet_config_dir.as_deref()?` :2209 (the never-named micro-arm — D-OPEN-DIR-ARM; origin
    :1388-1394, `config_dir.clone()`); `FleetSubscription::start` :2211-2219;
    `fleet_conn_seed` :2226-2228. Fields: `fleet_subscription` :398, `fleet_conn_last` :404.
  - **Arm (c)'s exact shape (a PRECISION over the ticket text):** the ticket says all three arms
    render bare "Fleet" today — the code says arm (c) is subtler: `mcp_json` is read from the
    launch cwd (app.rs:1428-1430) and builds `forge_client` :1431-1434; when it is `None`,
    `endpoint_for_brain(url, None)` still constructs with an EMPTY bearer (lib.rs:154-158 —
    `unwrap_or_default`; pinned `Some` by the lib.rs:537 unit), so the subscription STARTS and the
    header shows "Fleet · reconnecting" forever (bearer-requiring brain → 401 loop presented as
    transient wire trouble) or even "Fleet · live" with dead writes. The shared defect across
    (b)/(c) is "the header never names the cause", not literally "bare Fleet" — the spec's Title
    states the arms precisely. Write-failure evidence: `let Some(client) = self.forge_client…else`
    → in-card clamped "no forge client (.mcp.json)" at app.rs:6883-6888 (answer) and :6938-6944
    (dispatch) — reusing that exact wording for arm (c)'s header reason is the
    D-OPEN-REASON-PLACEMENT candidate (header and card tell one story).
  - **Arm (b)'s rejection space:** lib.rs — `endpoint_for_brain` :149-163 behind the SAME private
    `is_loopback_authority` :168-178; `split_url` :182-188 accepts ONLY `http://` (an `https://`
    brain_endpoint is arm (b) too — worth a classifier unit); bearer-IFF-url-match :154-158 means
    a urls-mismatch empty-bearer setup is SUPPORTED config (#376 D-OPEN-BEARER resolution,
    doc :143-148) — explicitly Out, keeps the arm boundary crisp.
  - **The clamp discipline (locked D4):** fleet_rail.rs:140-145 — `clamp_card_text`
    (control-strip + `take(200)`; doc: "SERVER-controlled text … must not blow the layout or
    smuggle control sequences"), already shared by the #377/#378 machines. The reason is
    config-derived render text → same discipline (it may embed the operator's own settings url;
    never bearer material).
  - **Render + state-tracking sites:** header render app.rs:16295-16318 (`fleet_header_title
    (self.fleet_conn_last)` :16306 feeding `dock_panel`; the F4 comment: render reads the
    pump-tracked field, takes no mutex); pump `pump_fleet_live` :6975-6993 (drains the cell,
    tracks `fleet_conn_last`, returns changed→dirty). The setup determination is BOOT-STATIC, so
    it needs no pump tracking — a stored field read by render (D2), but any Phase-2 deviation must
    re-check the dirty rule.
  - **Demo verb (arm-a intactness + D-OPEN-DEMO-MISCONFIG):** app.rs:8641-8646 —
    `demo_feed_permitted(self.fleet_subscription.is_some())`; arm (b) leaves demo permitted today
    (no subscription); candidate = unchanged.
  - **Behavior maps (sweep leg a):** docs/warp_architecture/subsystems/04-agent-ai-mcp.md:126 —
    Warp's cloud AI client "Distinguishes online/offline via `NetworkStatus`" (binary; no
    configured-but-broken arm); :191 name-drops `reconnecting_peer.rs` (name-level).
    docs/zed_architecture/crates/diagnostics.md:26/:76 — status-bar `DiagnosticIndicator`
    status-item (the "chrome names the problem class" convention; no connection analog). Research
    only; Reference stays N/A — Marley-specific.
- **The central design fork (spec D1 + D-OPEN-STATE-SHAPE), reasoned:** where does Misconfigured
  live? **(0) Grow `ConnectionState` — REJECTED, locked (D1):** it is the pump's published wire
  state; for arm (b) no pump exists, so no publish site could ever write the variant — a dead arm
  in a closed enum, and it would smear the #376 D3 ownership split (only the loop knows wire
  truth; absence/config is app knowledge — the same split that put `None`→"off" app-side). It
  would also force a `marley_forge_client` change for a purely presentational app concern.
  **(1) App-side typed setup state (RECOMMENDED):** a small `fleet_live` type carrying
  {arm, clamped reason}, computed once at boot from already-resolved inputs; `fleet_header_title`
  widened to (setup, wire) with an exhaustive match — machine-checkable arms, MSI-friendly,
  matches the module's cov/MSI-100 house shape. **(2) Minimal widening** (`misconfig:
  Option<&str>` parameter only): smallest diff, but the arm identity then lives only inside a
  string — untypeable in units, invisible to exhaustiveness. Phase 2 confirms (1) against
  render-site ergonomics; either way the OBSERVABLES are locked (distinct header + clamped
  arm-naming reason; wire arms byte-unchanged).
- **EARS drafted (7):** REQ-001 arm (b) → "Fleet · misconfigured" + invalid-endpoint reason;
  REQ-002 arm (c) → same header, its own reason (matches the in-card cause); REQ-003 arm (a) →
  bare "Fleet" + demo/#369 verbatim (the negative case); REQ-004 reason clamped + bearer-free;
  REQ-005 Live/Reconnecting byte-unchanged, transient ≠ misconfigured; REQ-006 once-at-boot,
  restored-root-keyed, stored-field render; REQ-007 live capture of the misconfigured header
  (env-blocked fallback documented).
- **Decisions:** locked D1 (no crate-enum growth — app-side boot determination), D2 (once, at the
  #376 post-restore site, restored-root-keyed, stored-field render), D3 (arm (a) quiet — byte
  "Fleet" + demo intact), D4 (clamp + bearer-free), D5 (exactly one reason, deterministic
  precedence, candidate b > c), D6 (wire arms untouched), D7 (hard-ordered after #381).
- **Open design questions (Phase 2):** D-OPEN-STATE-SHAPE (typed setup state [recommended] vs
  minimal string widening), D-OPEN-C-PRECEDENCE (arm (c) + wire=Live: who owns the header —
  candidate misconfigured wins, rail body keeps rendering snapshots per #376 D7),
  D-OPEN-REASON-PLACEMENT (candidate: muted caption atop `fleet_rail_body`; reason templates —
  reuse the in-card wording for arm (c)), D-OPEN-DEMO-MISCONFIG (candidate: leave the demo gate
  keyed on subscription-exists only), D-OPEN-DIR-ARM (classify the `fleet_config_dir` `?` abort
  vs accept-documented — first check what can make `config_dir` `None`).
- **Forge ids:** ticket #384 `3215851c-050a-46ff-86b4-ac06bfa8bafa`; sprint #36 "M25 — App-Grade
  QA Hardening"; pipeline `026c1b6c-f1b1-44c1-a13c-acb85788d231`; AAR pending-promotion (mint at
  /work). Depends-on: #381 (same sprint) — verify shipped before promotion.

### Promotion re-verification (2026-08-05, /work 384)
- **Promoted** queued → active; ticket claimed (owner `57761d04-1446-4aab-a30d-884dafb593c6`);
  AAR opened `a7321a75-efff-45b1-b09d-18848f6a72b0`. Sprint note: forge now lists #384 in sprint
  #39 (M28 carry) — the frontmatter's #36 is where it was drafted.
- **D7 SATISFIED:** #381 shipped (197f857) — `crate::mcp_config::mcp_json_path(active_root, cwd,
  is_file)` (mcp_config.rs:28-40, active-root-first, missing-FILE-only fallback; its module doc
  explicitly anticipates this ticket: a present-but-unparseable active-root file "surfaces AS
  misconfigured downstream"). The boot site consumes it at the SAME post-shell-restore block.
- **Refreshed anchors (current main = 4890d55; the draft's 2026-07-21 numbers superseded):**
  fields `fleet_subscription` app.rs:424 / `fleet_conn_last` :430; boot block :2364-2419
  (`cwd`/`active_root` :2379-2380, `mcp_json` path+read :2381-2385, `forge_client` :2386-2389,
  start-gate closure :2393-2414 with the TWO `?` aborts — `endpoint_for_brain` :2399 (arm b),
  `fleet_config_dir.as_deref()` :2400 (micro-arm) — `fleet_conn_seed` :2417-2419); in-card
  "no forge client (.mcp.json)" :8068 (dispatch) / :8124 (answer); `pump_fleet_live` :8155
  (tracks `fleet_conn_last` :8170-8171); demo verb :9937; header render :18184-18208
  (`fleet_rail_body` :18185-18191, `fleet_header_title(self.fleet_conn_last)` :18196 →
  `dock_panel`). fleet_live.rs unchanged since #376 (target :14-18, title :23-29, demo :34-36).
  `clamp_card_text` fleet_rail.rs:143. `endpoint_for_brain` lib.rs:149-163 unchanged
  (split_url `http://`-only :182-188, loopback wall :168-178, bearer-iff-url-match
  `unwrap_or_default` :154-158).
- **D-OPEN-DIR-ARM evidence:** `config_dir` = `marley_core::marley_config_dir().ok()` (app.rs:1488
  → `new_in(config_dir: Option<PathBuf>)` :1500; `fleet_config_dir` clone :1513). `None` only when
  the config dir is unresolvable (no `$HOME`) or a test passes `None` — and settings themselves
  load from the SAME Option (:1515), so a `None` boot already runs on defaults. Phase 2 decides
  with this in hand.
- **Environment:** pre-flight green (cargo 1.96.0, gate OK, mutants 27.1.0, llvm-cov 0.8.7, hooks
  wired, marley-web OK); bulletins: none; tree clean at 4890d55.

## Phase 2 — Design

### D-OPEN resolutions (all five, with evidence)
- **D-OPEN-STATE-SHAPE → the typed setup state (the Phase-1 recommendation, confirmed).**
  `fleet_live` gains `FleetSetup { Unconfigured, Misconfigured(Misconfig), Configured }`,
  `Misconfig { arm: MisconfigArm, reason: String }` (reason PRE-CLAMPED at construction),
  `MisconfigArm { EndpointRejected, ConfigDirUnavailable, ForgeClientMissing }`. Arms stay
  machine-checkable (exhaustive-match units, the module's cov/MSI-100 house shape); the title fn's
  three touchpoints (render app.rs:18196, its own unit, lib.rs:114 re-export) make the widening
  cheap — the census GREPPED THE CALLEE workspace-wide per
  `PR-claude-a-caller-census-greps-the-callee-name-001`: no other caller exists (tests/
  fleet_livewire.rs does not call it).
- **D-OPEN-C-PRECEDENCE → misconfigured wins the header.** Arm (c)'s empty-bearer subscription
  still starts (behavior unchanged — write-path changes are Out) and may reach Live, but writes
  are dead regardless and the operator must fix `.mcp.json`; the header names the config fault
  while the rail BODY keeps rendering delivered snapshots (#376 D7 data retention untouched).
  The wire state stays pump-tracked and returns the moment the setup is fixed (next boot).
- **D-OPEN-REASON-PLACEMENT → one muted caption line atop the rail body.** `fleet_rail_body`
  (the masked shim, app.rs:1071) gains `misconfig_reason: Option<&str>` and prepends one
  compact muted line (caption-scale, muted color — the empty-hint idiom at 13px/muted is the
  neighbor; the React prototype settles the exact look). Templates are FIXED strings:
  - arm (b): `brain_endpoint rejected (loopback http:// only): <url>` — embeds the operator's
    own settings value, then the whole string passes `clamp_card_text`;
  - dir micro-arm: `config dir unavailable (fleet cursor store)`;
  - arm (c): `no forge client (.mcp.json)` — BYTE-EQUAL to the in-card wording (app.rs:8068/
    :8124) so header and card tell one story.
- **D-OPEN-DEMO-MISCONFIG → unchanged.** The demo gate stays keyed on subscription-exists only
  (app.rs:9937): arm (b)/dir → no subscription → demo permitted (the misconfigured header
  disambiguates fixture seats); arm (c) → subscription exists → demo gated. Exactly today's
  behavior; #369 suites untouched.
- **D-OPEN-DIR-ARM → classified (ConfigDirUnavailable), with a reachability note.** Evidence:
  `config_dir: None` ⇒ settings fall back to `applied_defaults` (app.rs:1515-1524) ⇒ zero
  orchestration entries ⇒ `subscription_target` → `None` ⇒ arm (a). So target-Some structurally
  implies config_dir Some — the `?` at app.rs:2400 is unreachable TODAY. Classified anyway: the
  classifier is total (no-silent-abort contract), the arm is unit-pinnable in the pure space, and
  a future settings source that breaks the invariant fires a NAMED reason instead of silence.

### Architecture
**Pure seam (fleet_live.rs).** One new decision fn, the classifier — pure over the boot site's
already-resolved values, mirroring the site's own abort order (endpoint → dir → client), which IS
the D5 precedence, deterministic by construction:
```rust
pub fn classify_fleet_setup(
    target: Option<&str>,        // subscription_target's result (the url, arm-a gate)
    endpoint_constructed: bool,  // endpoint_for_brain(url, mcp_json).is_some()
    config_dir_available: bool,  // fleet_config_dir.is_some()
    forge_client_built: bool,    // forge_client.is_some()
) -> FleetSetup
```
| target | endpoint | dir | client | → |
|---|---|---|---|---|
| None | — | — | — | Unconfigured |
| Some(url) | false | — | — | Misconfigured(EndpointRejected, template+url, clamped) |
| Some | true | false | — | Misconfigured(ConfigDirUnavailable) |
| Some | true | true | false | Misconfigured(ForgeClientMissing) |
| Some | true | true | true | Configured |

The reason passes `crate::fleet_rail::clamp_card_text` (the SHIPPED sanitizer — prior-art leg 3;
no new ad-hoc clamp; fleet_rail does not import fleet_live, so no cycle). Widened title:
```rust
pub fn fleet_header_title(setup: &FleetSetup, state: Option<ConnectionState>) -> String
```
exhaustive match over (setup, state) — Misconfigured(_) wins every wire state →
"Fleet · misconfigured"; Unconfigured|Configured arms reproduce the three shipped strings
byte-identically. No defensive catch-all; a 9-combination unit pins the full product space.
Accessor `FleetSetup::misconfig(&self) -> Option<&Misconfig>` feeds the caption render.

**Boot site (app.rs:2364-2419, minimal restructure).** Hoist the target + endpoint out of the
closure so classification and subscription consume the SAME evaluations (no double-evaluate):
```rust
let fleet_target = active_root.as_deref().and_then(|root|
    crate::fleet_live::subscription_target(&applied.project_orchestrations, &root.to_string_lossy()));
let fleet_endpoint = fleet_target.as_deref()
    .and_then(|url| endpoint_for_brain(url, mcp_json.as_deref()));
let fleet_setup = crate::fleet_live::classify_fleet_setup(
    fleet_target.as_deref(), fleet_endpoint.is_some(),
    fleet_config_dir.is_some(), forge_client.is_some());
let fleet_subscription = match (fleet_endpoint, fleet_target.as_deref(), fleet_config_dir.as_deref()) {
    (Some(endpoint), Some(url), Some(dir)) => /* today's start block verbatim (deposit closure) */,
    _ => None,
};
```
(`active_root` :2380 already holds the project_count-gated restored root — one source, REQ-006's
never-the-launch-cwd key; `ForgeEndpoint` is `Clone` but the match MOVES it — no clone needed.)
New field `fleet_setup: crate::fleet_live::FleetSetup` beside `fleet_conn_last` (:430), seeded in
the `Self {}` literal (:2564 vicinity). Boot-static — the pump/dirty rule is untouched (the field
never changes post-boot; D2).

**Render (app.rs:18184-18208).** `fleet_header_title(&self.fleet_setup, self.fleet_conn_last)`;
`fleet_rail_body(..., self.fleet_setup.misconfig().map(|m| m.reason.as_str()), ...)` prepends the
caption line when Some.

### File manifest
**React half FIRST (the POC has NO fleet-dock surface today — the parity row is "(titlebar
`fleetOpen`)" only, so the mock is a NEW small Zone-A panel matching Marley's dock recipe:
DOCK_WIDTH 220, surface bg, border-l, caption_header = 11px muted px-3 py-1 border-b):**
- `marley-web/artifacts/marley-ide/src/components/views/FleetDock.tsx` (NEW) — right-edge dock
  panel: caption header ("Fleet" / "Fleet · misconfigured") + the muted reason caption + the
  "no seats — the fleet is quiet" empty hint; driven by a mocked state prop, no real probes.
- `marley-web/artifacts/marley-ide/src/App.tsx` — `fleetDockMock: 'off'|'ok'|'endpoint'|'client'`
  state field (default 'off').
- `marley-web/artifacts/marley-ide/src/components/overlays/CommandPalette.tsx` — one mock row
  cycling the state (drive + screenshot handle).
- `marley-web/artifacts/marley-ide/src/pages/Workspace.tsx` — mount FleetDock when ≠ 'off'.
- `marley-web/docs/MARLEY-PARITY.md` — fleet-strip row gains `views/FleetDock.tsx`.

**Rust half (ported 1:1 from the approved React state):**
- `crates/marley_app/src/fleet_live.rs` — FleetSetup/Misconfig/MisconfigArm,
  classify_fleet_setup, widened fleet_header_title, misconfig accessor; units (below).
- `crates/marley_app/src/lib.rs` — extend the :114 re-export (plain code spans in any docs — the
  #400 gate:14 lesson).
- `crates/marley_app/src/app.rs` — field + boot restructure + render deltas + fleet_rail_body
  signature (+ its caption child).
- `crates/marley_app/src/headless_drive.rs` — the misconfigured-boot drive(s).

### Regression test plan
| REQ | Test |
|---|---|
| REQ-001 | units: classify(Some, false, …) → EndpointRejected for the three rejection shapes' BOOL (the url-shape negatives themselves are lib.rs:540's shipped suite); reason contains the url; title(Misconfigured, any) == "Fleet · misconfigured". Headless drive: seed settings.toml `[[projects.orchestration]]` for the seeded root with `brain_endpoint = "http://evil.example:9"` → boot → `fleet_setup` is Misconfigured(EndpointRejected) + subscription None + title reads misconfigured (REQ-006 wiring proof rides this). |
| REQ-002 | unit: classify(Some, true, true, false) → ForgeClientMissing with reason BYTE-EQUAL "no forge client (.mcp.json)"; in-card sites untouched (diff shows no :8068/:8124 change). |
| REQ-003 | units: classify(None, b, c, d) → Unconfigured over all 8 bool combos; title(Unconfigured, None) == "Fleet" byte-pin (t376_req002 carried, widened); #369/#376 suites green unchanged. |
| REQ-004 | unit: hostile url (control chars + >200 len) → reason control-free and ≤200 (clamp proof). |
| REQ-005 | the 9-combination exhaustive title unit — Unconfigured/Configured × None/Live/Reconnecting byte-identical to shipped; Misconfigured × all three → misconfigured. fleet_livewire integration suite green unchanged. |
| REQ-006 | the REQ-001 drive (boot-populated field, restored-root-keyed via active_root) + §18.1 inspect trace: ONE classify call site, render reads the stored field. |
| REQ-007 | live capture: sandbox-HOME settings with a rejected brain_endpoint → launch → ⌘⇧E (toggle-fleet-dock) → capture shows "Fleet · misconfigured" + caption; parity pair: the React mock at arm-b state, both PNGs read together. |

### Risks / notes
- `fleet_rail_body` widening: verify its `mutants::skip` status with `cargo mutants --list -f` on
  the ACTUAL touched files (the skip-detach trap's home turf — 5th strike was `pump_fleet_live`).
- The boot restructure moves the subscription-start into a `match` — keep the closure body
  byte-identical; the two `?`s become the match's `_ => None` (their classification lives in
  `fleet_setup`, computed from the same values — no behavior change on any arm).
- tests/fleet_livewire.rs boots real subscriptions against local listeners — no title calls; the
  new field defaults happen in new_in, so no test churn expected there.
- §20: clean-room untouched — no reference source consulted; the design extends shipped in-house
  seams only.

## Phase 3 — Implement

### React-first (parity) — DONE FIRST, look approved at localhost:5173
- Built the POC's missing fleet-dock surface as a mock: **`components/FleetDock.tsx`** (NEW —
  DELIBERATE DEVIATION from the manifest's `views/` placement: docks live beside `FileDock.tsx`
  in `components/`, views are main-content panes; the dock idiom is the sibling's), +
  `App.tsx` `fleetDockMock: 'off'|'ok'|'endpoint'|'client'` (default off, persisted-state-merge
  safe), + a palette row "Fleet Dock (mock): cycle state" (off → endpoint → client → ok), +
  the `Workspace.tsx` right-edge mount. Mock discipline held: state-driven only, no probes.
- Iterated visually (one adjustment: the reason caption's inset moved px-2/pt-2 → **px-3/pt-3**
  so its left edge aligns with the empty hint's 16px inset). Typecheck green.
- **Approved captures (all READ):** `scratchpad/384-captures/react/arm-b-endpoint.png`
  ("Fleet · misconfigured" + the wrapped brain_endpoint-rejected reason + empty hint),
  `arm-c-client.png` ("Fleet · misconfigured" + "no forge client (.mcp.json)"),
  `ok-quiet.png` (bare "Fleet", NO caption — the arm-(a)/healthy control). The approved look:
  caption 11px muted, empty-hint inset, wraps with the long URL; header title as-given.

### Rust port (1:1 from the approved React state)
- **fleet_live.rs** — `FleetSetup { Unconfigured, Misconfigured(Misconfig), Configured }`,
  `Misconfig { arm, reason }` (reason clamped AT CONSTRUCTION via the shipped
  `fleet_rail::clamp_card_text`), `MisconfigArm { EndpointRejected, ConfigDirUnavailable,
  ForgeClientMissing }` (declared in the start-gate's abort order = D5 precedence),
  `FleetSetup::misconfig()` accessor, `classify_fleet_setup(target, endpoint_constructed,
  config_dir_available, forge_client_built)` (first-failing-gate names the one reason; arm-c
  template BYTE-EQUAL to the in-card wording), widened
  `fleet_header_title(&FleetSetup, Option<ConnectionState>)` (Misconfigured wins every wire
  state; quiet arms byte-identical; exhaustive or-pattern match, no catch-all). Module doc
  updated; t376_req002 updated to the widened signature (compile fix — new units are Phase 4).
- **lib.rs** — re-export extended: `classify_fleet_setup, FleetSetup, Misconfig, MisconfigArm`.
- **app.rs** — field `fleet_setup` beside `fleet_conn_last` (+ struct-literal init); boot block
  restructured so classification + start consume ONE set of evaluations (`fleet_target` via the
  existing `active_root` — same restored-root value, one source; `fleet_endpoint` computed once,
  MOVED into the start via the 3-way match; the two silent `?`s became the match's `_ => None`
  with their classification in `fleet_setup`); render passes
  `self.fleet_setup.misconfig().map(|m| m.reason.as_str())` into the widened `fleet_rail_body`
  (caption child: px_3/pt_3/11px/muted — the React-approved geometry) and the header title takes
  `(&self.fleet_setup, self.fleet_conn_last)`.
- `cargo check --workspace --all-targets` GREEN — the callee-grep caller census (3 touchpoints)
  held; no #400-style hidden caller surfaced.
- Deviations from design: only the FleetDock.tsx placement (components/ not views/, above).

## Phase 3.5 — Inspect

Three critics, distinct lenses: (1) correctness-vs-boot-gates, (2) bearer/security/§20,
(3) state-integrity/simplification. All verified findings concretely (od -c byte compares,
full-crate greps, truth-table walks; critic 1 also ran the fleet_live 5/5 + fleet_livewire 4/4
suites green).

### Findings ledger
| # | Sev | Finding | Verdict | Fix |
|---|---|---|---|---|
| F1 | medium | `clamp_card_text` strips Cc only — bidi Cf chars (U+202A–202E overrides, U+2066–2069 isolates) survive → Trojan-source display reordering; the helper also clamps SERVER-controlled #377/#378 text, so the gap spans four consumers | REAL | filter widened to also strip the bidi-control set (fleet_rail.rs); hostile unit lands in the P4 suite |
| F2 | medium | `"no forge client (.mcp.json)"` duplicated ×3 in Rust (+1 React) with byte-equality enforced only by a comment | REAL | hoisted `fleet_rail::NO_FORGE_CLIENT_REASON`; all three Rust sites consume it (the React mock mirrors it as data, checked by the parity pair) |
| F3 | medium | React: corrupted persisted `fleetDockMock` (same STORAGE_VERSION, bogus value) destructures `undefined` → white-screen; no ErrorBoundary, so the palette's self-healing cycle is unreachable | REAL | unknown value renders as 'off' (`if (!mock) return null`) |
| F4 | low | `Misconfig.reason` clamp invariant was convention (pub field, publicly re-exported) — a future direct construction bypasses it silently | REAL | field made private; `Misconfig::new(arm, reason)` clamps at construction (type-enforced), `reason()` accessor |
| F5 | low | `misconfig()` deviates from the house `as_*` accessor shape (`Content::as_terminal`/`as_cockpit`) | REAL | renamed `as_misconfig()` |
| F6 | low | `FLEET_DOCK_WIDTH` exported-but-unused while the class hardcodes `w-[220px]` (FileDock precedent drives style width from its const) | REAL | `style={{ width: FLEET_DOCK_WIDTH }}` |
| F7 | low | ANSI residue: ESC (Cc) strips but the `[31m` remainder survives as literal text | ACCEPTED — inert in gpui (no terminal interpretation), cosmetic only; no fix |
| F8 | info | An operator's own `http://user:secret@host/` settings value renders verbatim in the arm-b reason | ACCEPTED per D4 (operator's OWN settings-sourced value; no `.mcp.json` cross-contamination possible — classifier takes presence BOOLEANS only) — recorded so the accepted risk is on paper |
| F9 | low | lib.rs re-export of the four new names has no consumer today | KEEP — the critic's own verdict: exact #376 block precedent, and pub-ness keeps the not-yet-read `arm` field lint-free pre-P4 |
| F10 | low | runtime project open/switch never recomputes `fleet_setup` | NO-ACTION — inherited #376 D1 boot-resolved-only posture; setup and wire go stale TOGETHER (subscription is equally boot-static), never contradictory; the spec's explicit Out |
| F11 | low | the "t384 suite" the code comment references does not exist yet; the carried t376 unit leaves the or-pattern halves unpinned; `classify_fleet_setup`/the Misconfigured arm have zero coverage in-diff | P4 HARD DELIVERABLE — the 9-combination product-space unit + classifier suite + hostile-clamp unit close all three (planned in the P2 test plan; the gate blocks /commit until they land) |

### Adversarial checks that PASSED (evidence in the critic reports)
- **Restructure equivalence:** `active_root` unmoved between definition and use; start-condition
  set identical (endpoint⇒target⇒project_count); one evaluation each of
  `subscription_target`/`endpoint_for_brain`; same url feeds `fleet_cursor_dir`; `_ => None`
  covers exactly the old `?` aborts. **No-silent-abort:** zero `?` remain in the block; every
  target-Some outcome classified. **Title byte-identity:** od -c on the three shipped strings —
  identical incl. U+00B7 spacing; match total with no catch-all (2-variant wire enum × 3 setups
  = 4 written arms cover all 9).
- **Bearer:** classifier signature takes booleans — `.mcp.json` CONTENT structurally cannot
  reach a reason; `mcp_json` flows only into the two pre-existing loopback-walled fns; no new
  logging anywhere in either diff; Debug output cannot hold bearer material.
- **State integrity:** `fleet_setup` written exactly once (ctor), moved into Self, never
  mutated — no-dirty-tracking is sound; header + caption read the ONE field in the same render
  pass (title says misconfigured ⟺ caption present, same variant test); caption survives the
  empty-hint early return and coexists with demo seats; init order verified (no stale/moved-from
  values at classification).
- **arm-c honesty:** `endpoint_for_brain` returns None only for non-loopback/non-http/unparseable
  — a MISSING `.mcp.json` degrades to empty-bearer (arm c), so the arm-b "loopback http:// only"
  template can never misfire for what is really arm c.
- **React:** old sessions resume 'off' (defaults-merge); palette cycle self-heals corrupted
  values (indexOf −1 → 'off'); Rust↔TSX byte-compare exact (arm-c string, arm-b prefix, title
  U+00B7, em-dash empty hint, 220px, 11px caption); gpui's char-level LineWrapper fallback
  matches `break-words`.
- **Mutation-gate status (skip-detach re-check):** `fleet_rail_body` IS `mutants::skip` (the
  caption if-let is masked shim — rides the REQ-007 capture); the LIVE targets are fleet_live's
  new pure fns + the widened clamp — the P4 t384 suite is what kills them (F11).
- **§20:** in-house seams only; no foreign idioms; no reference source consulted.

Post-fix: `cargo check --workspace --all-targets` green; POC typecheck green.

## Phase 4 — Validate

### Tests written + RUN (the t384 suite — closes inspect F11)
- **fleet_live.rs units (6):** t384_req001 (arm b: EndpointRejected + reason embeds the url),
  t384_req002 (arm c: reason IS `NO_FORGE_CLIENT_REASON`), t384_classify_dir_arm_and_precedence
  (micro-arm + the D5 first-failing-gate ladder + Configured-no-reason),
  t384_req003 (target-None → Unconfigured over all 8 bool combos),
  t384_req004 (hostile url: ESC stripped, U+202E stripped, 200-cap, template prefix survives —
  kills the Misconfig::new clamp + reason() mutants), t384_req005 (the FULL 3×3 title product
  space — closes the or-pattern halves t376_req002 leaves open).
- **fleet_rail.rs unit (1):** t384_clamp_strips_bidi_and_shared_reason_pinned (bidi
  embeds/overrides/isolates stripped; ESC stripped with printable residue per accepted F7; the
  shared const byte-pinned).
- **headless drives (3):** boot_misconfigured_endpoint_names_the_arm_headless (REQ-001/006 —
  seeded orchestration entry with an `https://` endpoint → arm b + reason + title over the REAL
  render pair + NO subscription + a dock-open render pass), boot_unconfigured_root_stays_quiet
  (REQ-003 control — bare "Fleet", no subscription), boot_no_forge_client_names_arm_c_over_the_wire
  (REQ-002 + C-PRECEDENCE — valid loopback endpoint, no `.mcp.json` at either candidate → arm c,
  reason == the in-card literal, subscription STARTS, wire seeds Reconnecting, header still
  misconfigured). New test accessors: fleet_setup_for_test / fleet_subscription_active_for_test /
  fleet_conn_last_for_test (the for_test block idiom).
- **Runs:** targeted 10/10 PASS; full `cargo nextest run --workspace` **2043/2043 PASS**
  (5 skipped — pre-existing); `cargo test --workspace --doc` green. No pre-existing failures.
- **`cargo mutants --list -f`** on fleet_live.rs+fleet_rail.rs: 72 mutants enumerated; every NEW
  one has a named killer (reason/as_misconfig/title → t384 exact-string asserts; the three
  classify gate-inversions → req001/dir-ladder/req002; clamp fn+filter → t384_clamp/t377/req004);
  the `Default::default()` replacements are UNVIABLE by design (no Default derives). app.rs's
  boot/render deltas enumerate ZERO live mutants (inside skip-masked shims — the skip-detach
  re-check from the Floors).

### Live drive (REQ-007) + parity pair
- Sandbox HOME (internal disk): `<session-scratchpad>/384-sandbox-home` — settings seed:
  one-project shell blob + `[[projects.orchestration]]` root=proj,
  `brain_endpoint = "https://brain.example:9000/mcp"` (arm b). Bundle: bundle-app.sh (debug).
- **Capture `scratchpad/384-captures/live-arm-b.png` (READ):** right dock open, header
  **"Fleet · misconfigured"**, caption `brain_endpoint rejected (loopback http:// only):
  https://brain.example:9000/mcp` (muted, wrapped, atop the body), `no seats — the fleet is
  quiet` beneath, rest of the app normal. Driven through the REAL palette verb ("Toggle Fleet
  Dock" — palette-only by design: ⌘⇧E is `toggle-fleet` the #68 agents overlay, NOT the dock).
- **Parity pair (React reference `scratchpad/384-captures/react/arm-b-endpoint.png`, both READ
  together):** header/caption/hint text + composition 1:1; dock bg sampled PIXEL-EXACT both
  sides (`srgb(26,27,31)`); caption color = the muted-foreground ROLE on both sides (Marley
  hsla 223.2°/8%/60% vs POC hsl 225/8%/59% — the standing palette calibration, pre-existing
  shell tolerance, not this ticket's delta). Also captured: react/arm-c-client.png +
  react/ok-quiet.png (the mock's other states, approved at implement).
- **⌘Q wire round-trip:** settings.toml before-vs-after quit `diff` **IDENTICAL** — the
  orchestration entry (hand-edited-only posture) and the shell blob are untouched; clean quit.
- **Harness lessons (recorded for forge capture at P5):** (a) `open` on an already-running app
  ACTIVATES the stale instance — `--env` silently ignored; kill first; (b) a sandbox HOME on a
  removable volume (/Volumes/Offload) fires a macOS TCC "removable volume" prompt that wedges
  boot pre-window — TCC dialogs are the operator's (never clicked); the workaround is an
  internal-disk sandbox HOME (session scratchpad); (c) macOS 26: `NSRunningApplication.activate`
  makes the app frontmost but the gpui window is NOT KEY until a real synthetic CLICK lands —
  keystrokes/chords silently drop until then (`focus clickat:…` THEN `type:`/`cmdshift:`);
  mouse events were the tell (they landed while keys didn't).

### Gate
- **Run 1 RED (two gates):** gate:1 rustfmt (5 mechanical diffs — long lines in the new code) and
  gate:4 coverage — fleet_live.rs 99.08% lines: the TWO misses were the dead-on-success `panic!`
  bodies of test `let-else` destructures (t384_req001/002). Fixed at source: `cargo fmt --all` +
  the destructures rewritten to the accessor shape (`as_misconfig().expect(…)`) which leaves no
  uncoverable arm. (Lesson: a let-else-with-panic in TEST code is an uncoverable line under the
  100%-lines floor — destructure through accessors instead.)
- **Run 2: `GATE GREEN [diff]` 15/15** — coverage 100% lines, MSI 100 (gate:5 passed on BOTH runs
  — the t384 kills held), receipt at `.git/ignibyte-gate-receipt`.
- Pre-existing exclusions: none (2043-suite fully green both runs).

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG entry (TICKET-384, above #400's);
  docs/marley_architecture/orchestration-shell.md precondition-1 block gains the #384 shipped
  shape (classifier, misconfigured header, reason discipline, bidi-hardened clamp);
  marley-web/docs/MARLEY-PARITY.md fleet-strip rows gain `components/FleetDock.tsx` (Zone A row +
  the port-map row with the byte-matching reason templates). Parity holds at close: the POC mock
  is the approved reference for the new surface; templates byte-equal the Rust.
- **Forge capture:** failure BF-claude-clamp-card-text-passed-bidi-format-chars-001 (the shipped
  #377-era Cf gap the inspect found); prevention rules
  PR-claude-display-sanitizers-strip-bidi-cf-not-just-cc-001 (prevents the BF),
  PR-claude-test-destructures-use-accessor-expect-under-a-lines-floor-001 (the gate-red lesson),
  PR-claude-live-drive-click-first-activation-does-not-make-the-window-key-001 (the macOS-26
  harness triple: click-before-key + open-reuses-running-instance + removable-volume TCC).
  AAR a7321a75 submitted: completed, effectiveness 5, 4 novel findings.
- **Ticket closed:** local doc → tickets/closed/ (Status: closed, owner + AAR recorded); forge
  #384 → done. Sprint #39 (M28 Registry Payoff) closed — its full roster has now shipped
  (#396–#399, #352, #400, #384).
- **Archive:** pipeline pair → docs/planning/pipeline/completed/.
