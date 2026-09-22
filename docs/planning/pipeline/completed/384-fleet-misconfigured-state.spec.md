---
pipeline_id: 026c1b6c-f1b1-44c1-a13c-acb85788d231
ticket: forge#384 (3215851c-050a-46ff-86b4-ac06bfa8bafa) · local docs/planning/tickets/open/TICKET-384-fleet-misconfigured-state.md
aar_id: a7321a75-efff-45b1-b09d-18848f6a72b0
status: Phase 5 — Complete PASS
title: Fleet header "misconfigured" diagnosability state — the silent-degrade arms get a visible reason (#376 F8)
type: feature
milestone: M25
references:
  - docs/planning/pipeline/completed/376-fleet-rail-livewire.spec.md
  - docs/planning/pipeline/completed/376-fleet-rail-livewire.notes.md
  - crates/marley_app/src/fleet_live.rs
  - crates/marley_app/src/fleet_rail.rs
  - crates/marley_app/src/app.rs
  - crates/marley_forge_client/src/adapter.rs
  - crates/marley_forge_client/src/lib.rs
---

## Title
#376's inspect F8, cashed in: per the locked D5/D6 (376-fleet-rail-livewire.spec.md), a configured-
but-broken fleet setup is indistinguishable from an unconfigured one. The precise arms, verified
against the shipped boot path (app.rs:2192-2228):
- **(a) Unconfigured (LEGIT — stays quiet):** `subscription_target` → `None` (no
  `[[projects.orchestration]]` entry for the restored active root, or `brain_endpoint`
  absent/empty — fleet_live.rs:14-18). No subscription; header is the byte-pinned bare "Fleet"
  (fleet_live.rs:23-29); demo verb permitted (app.rs:8641-8646). Correct today, unchanged here.
- **(b) Endpoint rejected (SILENT DEGRADE):** target `Some(url)` but `endpoint_for_brain` → `None`
  — non-loopback host or a URL `split_url` won't parse (only plain `http://` is accepted —
  lib.rs:149-163/:182-188). The `?` at app.rs:2208 silently aborts the start; the operator who
  configured an entry sees exactly the unconfigured bare "Fleet". The wall itself is the ratified
  D6 posture — only its *silence* is the defect.
- **(c) No forge client (CAUSE-MUTE):** target `Some(url)` and the endpoint constructs, but no
  `ForgeClient` was built — `.mcp.json` missing/unreadable at the resolved location, or present but
  unparseable/forge-entry-less (app.rs:1428-1434). The subscription still starts with an EMPTY
  bearer (lib.rs:154-158), so the header shows "Fleet · reconnecting" (a bearer-requiring brain
  401s forever — a config fault presented as a transient wire state) or even "Fleet · live" with
  dead writes; every answer/dispatch fails in-card with the clamped
  "no forge client (.mcp.json)" (app.rs:6883-6888/:6938-6944) while the header never names the
  root cause. This is the QA-2026-07-21 find that produced #381.

The work: a pure app-side **setup determination** in `fleet_live` — computed once at the #376
post-shell-restore boot site, keyed to the restored active root — plus a widened pure header
presentation: misconfigured roots render a distinct "Fleet · misconfigured" header and a compact,
clamped reason naming which arm fired. `ConnectionState` (adapter.rs:177-187) does NOT grow a
variant. **Hard-ordered AFTER #381:** today `mcp_json` is read from the LAUNCH CWD
(app.rs:1428-1430), so pre-#381 a Finder launch (cwd `/` or `$HOME`) makes arm (c) fire spuriously
for EVERY configured workspace — shipping this banner first would make it lie on every Finder
boot. #381's root-anchored `.mcp.json` resolution decides which arm genuinely fires there; the
classifier consumes the same post-#381-resolved value the boot site already holds.

## Scope
### In
- **Pure setup classifier + widened presentation** (fleet_live.rs, cov/MSI 100): a typed
  determination from the boot site's already-resolved inputs (orchestration configs + restored
  active root + endpoint-construction outcome + forge-client presence) → Unconfigured (arm a) /
  Misconfigured{arm, reason} (arms b/c) / Configured; `fleet_header_title` widened (shape =
  D-OPEN-STATE-SHAPE) so misconfigured roots title "Fleet · misconfigured" while the shipped
  `None`→"Fleet" / Live / Reconnecting arms stay byte-identical.
- **Reason hygiene:** the compact reason names the ARM (not a sub-diagnosis), passes the shipped
  clamp discipline (`clamp_card_text` — control-strip + ≤200, fleet_rail.rs:140-145 — or an
  equivalent pure clamp), and can never carry bearer material.
- **No-silent-abort contract on target-Some roots:** every outcome between `subscription_target ==
  Some` and a healthy start maps to a named reason — the two `?` aborts on the start path
  (endpoint app.rs:2208; config-dir app.rs:2209, the never-named micro-arm) and the
  forge-client-absent case all classify; only target-`None` stays quiet.
- **Boot-site population (masked, app.rs):** compute + store the determination ONCE at the #376
  start-gate site (post-shell-restore, app.rs:2192-2228 — the F2 lesson site), a field beside
  `fleet_conn_last` (:404); the header render (:16306) and the reason render read the stored
  field (the F4 one-coherent-pair discipline).
- **Reason render surface:** one compact clamped line per D-OPEN-REASON-PLACEMENT (candidate: a
  muted caption atop `fleet_rail_body` when misconfigured).
- **Live capture** of the misconfigured header on an arm-(b) root (REQ-007).
- **Regression pins:** #369 rail + demo suites, #376 `fleet_live` units + `fleet_livewire`
  integration, #377/#378 in-card failure reasons — all green unchanged; arm (a) byte-identity.

### Out (explicitly deferred)
- **Retarget/re-classify on project switch** — boot-resolved-only stands (#376 D1 +
  D-OPEN-RETARGET's resolution); a workspace switch does not re-run the determination.
- **Runtime re-validation** — no settings/`.mcp.json` watcher; no runtime settings-reload path
  exists (settings load once at boot; the #376 spec pinned this), and inventing one is not this
  ticket.
- **Any `marley_forge_client` change** — `ConnectionState` untouched (the central fork's
  resolution), `endpoint_for_brain`'s `Option` signature untouched: the reason names the arm, so
  no typed rejection cause is needed from the crate.
- **Wire-level auth diagnosability** — a PRESENT `.mcp.json` whose forge url ≠ brain url is a
  SUPPORTED empty-bearer config (#376 D-OPEN-BEARER's resolution, lib.rs:143-148), not
  misconfiguration; the running-subscription 401→eternal-"reconnecting" case (F8's third
  sub-symptom sensu stricto) needs crate-side classification and stays deferred.
- **Fixing `.mcp.json` resolution itself** — that is #381; this ticket consumes its result.
- **Any new settings schema/fields; any write-path behavior change** (the in-card reasons stay
  exactly as shipped).

## Reference (§20)
**N/A — Marley-specific; no reference-app analog** (carries #376's stance: the fleet control plane
is Marley's own design, docs/marley_architecture/orchestration-shell.md). The checked behavior map
docs/warp_architecture/subsystems/04-agent-ai-mcp.md shows Warp's nearest surface distinguishes
only "online/offline via `NetworkStatus`" (04-agent-ai-mcp.md:126) — a binary wire state on a
cloud client, no configured-but-broken arm to observe. At the published-behavior level (no source),
the IDE convention Marley matches is the status indicator that DISTINGUISHES off / connecting /
error states and names the cause on inspection — e.g. VS Code's remote indicator (documented,
published behavior: a status-bar item that labels the remote context and surfaces a distinct
error/disconnected state with an explanation) — behavior-level only. Clean-room §20 untouched: no
Warp (AGPL) / Zed (GPL) source consulted.

### Prior art
1. **Behavior maps — checked, no owner.** docs/warp_architecture/subsystems/04-agent-ai-mcp.md:126
   (online/offline `NetworkStatus` — binary, no misconfigured arm mapped; :191 name-drops a
   `reconnecting_peer.rs` among MCP-manager UI files, name-level only);
   docs/zed_architecture/crates/diagnostics.md:26/:76 map Zed's status-bar `DiagnosticIndicator`
   status-item — supports the "chrome indicator that names the problem class" convention, no
   fleet/connection analog. Research, not source.
2. **Published material.** VS Code remote-indicator / status-bar error conventions (states named;
   click → explanation), per its published docs — adopted at the behavior level as the distinct-
   state-plus-reason shape. The MCP spec adds nothing here (no protocol surface in this ticket).
3. **Our permissive deps — checked, no owner.** gpui (Apache-2.0) provides render primitives only;
   ropey/regex/alacritty_terminal/tree-sitter own nothing near a status-presentation seam. The
   real adoption leg is IN-HOUSE shipped seams: `fleet_header_title` already owns the header
   presentation (fleet_live.rs:23-29 — this ticket widens it rather than adding a second title
   path), and `clamp_card_text` already owns hostile/config-derived render-text hygiene
   (fleet_rail.rs:140-145 — the reason goes through it or an equivalent, never a new ad-hoc
   sanitizer).

## React-first (parity)
**UI-AFFECTING — Zone A adjacent (the fleet dock header + rail-body caption; port-map row: fleet
strip ↔ `fleet_rail.rs`/`fleet_live.rs` — `marley-web/docs/MARLEY-PARITY.md`).** Implement builds
the presentation in `marley-web` FIRST: the "Fleet · misconfigured" header variant + the compact
muted reason caption atop the fleet rail body, one mocked state per arm (b: rejected endpoint;
c: no forge client), iterated at localhost:5173 (`pnpm --filter @workspace/marley-ide run dev`)
until the look is confirmed — THEN ported 1:1 into the `fleet_live` pure seam + the app.rs render
shim. The approved React state is the reference for the NEW misconfigured presentation; the
shipped bare-"Fleet"/Live/Reconnecting renders stay Marley-authoritative and byte-identical
(D3/D6). Validate captures the React↔Marley parity pair (REQ-007's capture is the Marley half).
POC discipline: the React misconfigured state is driven by a mocked setup flag — never a real
`.mcp.json` probe (the POC never ports its data).

## Locked-In Decisions
- **D1 — misconfiguration is an app-side BOOT determination; `ConnectionState` does NOT grow a
  variant.** The crate enum is the PUMP's published wire state ("published by the pump thread",
  adapter.rs:177-187); its only writers are the loop's seed/publish sites (:202, :317, :473). For
  arm (b) no pump ever EXISTS — no publish site could legitimately write `Misconfigured`, so a
  crate variant would be a dead arm breaking the #376 D3 ownership split (the loop owns wire
  truth; ABSENCE/config knowledge is the app's). The determination lives app-side in `fleet_live`;
  its representation shape is D-OPEN-STATE-SHAPE.
- **D2 — computed ONCE at the #376 boot site, keyed to the RESTORED active root.** The same
  post-shell-restore block that runs the start-gate (app.rs:2192-2228) computes and stores the
  determination (never the launch cwd — the #376 F2 lesson,
  `PR-claude-boot-decisions-key-the-restored-active-root-001`); the render reads the stored field,
  never re-derives per frame (the F4 one-coherent-pair discipline). Boot-resolved-only, mirroring
  #376 D1.
- **D3 — arm (a) stays QUIET.** `subscription_target` → `None` keeps the byte-pinned bare "Fleet"
  (`fleet_header_title(None) == "Fleet"`, fleet_live.rs:23-29 + its t376_req002 pin), the demo
  verb, and the #369 rail behaviors verbatim — a root that never opted into fleet must not be
  nagged.
- **D4 — the reason is CLAMPED and bearer-free.** Every rendered reason passes the shipped
  control-strip + ≤200 discipline (`clamp_card_text`, fleet_rail.rs:143-145, or an equivalent pure
  clamp in `fleet_live`) — it may embed the operator's own configured value (e.g. the rejected
  `brain_endpoint` url, settings-sourced) but NOTHING sourced from `.mcp.json`'s Authorization
  ever reaches presentation (the #64/#375/#376-D6 never-logged-bearer contract extended to the new
  surface).
- **D5 — exactly ONE compact reason renders, deterministically.** When multiple arms hold at once
  (e.g. endpoint rejected AND no forge client), a fixed precedence picks one (candidate: b > c —
  the endpoint is unusable regardless of the client); no stacking, no oscillation.
- **D6 — Live/Reconnecting presentation is untouched.** On a well-configured root the widened seam
  yields byte-identical "Fleet · live"/"Fleet · reconnecting"; a transient reconnect NEVER
  presents as misconfigured (wire trouble ≠ config trouble — the same line #376 D3 drew against
  age-inference).
- **D7 — hard-ordered AFTER #381.** The classifier consumes the post-#381 root-anchored
  `.mcp.json` resolution (the same `mcp_json`/forge-client values the boot site holds). Pre-#381,
  the launch-cwd read (app.rs:1428-1430) would make arm (c) fire on every Finder launch of a
  correctly-configured workspace — a lying banner. /work must not promote #384 while #381 is
  unshipped.

**D-OPEN (Phase 2 decides, with evidence):**
- **D-OPEN-STATE-SHAPE** — the app-side representation: a typed setup state (e.g.
  `FleetSetup { Unconfigured, Misconfigured(Misconfig{arm, reason}), Configured }` — or
  `Option<Misconfig>` beside `fleet_conn_last`) with `fleet_header_title` widened to
  (setup, wire-state), vs the minimal widening `fleet_header_title(state, misconfig:
  Option<&str>)`. **Recommendation: the typed setup state** — the arm stays machine-checkable
  (exhaustive-match units, the MSI-friendly house shape for a cov/MSI-100 module) instead of
  living only inside a string; the title fn's two callers (app.rs:16306 + the t376_req002 unit)
  make the signature change cheap. Phase 2 confirms against the render-site ergonomics.
- **D-OPEN-C-PRECEDENCE** — arm (c) coexists with a RUNNING wire (the empty-bearer subscription
  may reach Live). Header when misconfigured AND wire=Live: candidate — misconfigured wins the
  header (writes are dead regardless and the operator must fix `.mcp.json`; the rail BODY keeps
  rendering delivered snapshots — #376 D7 data-retention untouched); alternative — wire state
  wins, reason renders only in the rail caption. Decide with the QA scenario replayed.
- **D-OPEN-REASON-PLACEMENT** — where the compact reason renders: candidate a one-line muted
  caption atop `fleet_rail_body` when misconfigured (the rail already composes an empty state;
  the header stays a short title); alternatives: title suffix (overlong for the dock header),
  hover tooltip (undiscoverable). Also naming: reason strings are fixed short templates
  ("brain_endpoint rejected (loopback http:// only): <url>", "no forge client (.mcp.json)" —
  reusing the exact in-card wording at app.rs:6888 for arm (c) is the candidate, so header and
  card tell one story).
- **D-OPEN-DEMO-MISCONFIG** — is `fleet-demo-feed` still permitted on a misconfigured root?
  Today arm (b) leaves it permitted (no subscription → `demo_feed_permitted(false)=true`,
  app.rs:8641-8646). Candidate: unchanged — the misconfigured header already disambiguates demo
  seats from live truth; gating would change #369 behavior for no diagnostic gain.
- **D-OPEN-DIR-ARM** — the config-dir-absent micro-arm (`fleet_config_dir.as_deref()?`,
  app.rs:2209; origin :1388-1394): classify under a named reason (candidate, per the
  no-silent-abort contract) vs accept-documented as unreachable-in-practice. Phase 2 checks what
  can actually make `config_dir` `None`.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the restored active root has an orchestration entry whose `brain_endpoint` fails endpoint construction (non-loopback host, non-`http://` scheme, or unparseable), the header shall read "Fleet · misconfigured" and a compact clamped reason naming the invalid-endpoint arm shall be presented; no subscription starts (unchanged). | pure classifier units per rejection shape (mirroring the lib.rs endpoint_for_brain suite's negative arms); widened-title unit; §18.1 inspect: the start path still aborts, now classified |
| REQ-002 | WHEN the restored active root has an orchestration entry and no forge client was built from the post-#381-resolved `.mcp.json` (missing, unreadable, or unparseable/forge-entry-less), the header shall read "Fleet · misconfigured" with a distinct clamped reason naming the missing-forge-client arm — the header now names the same cause the in-card write failures already report. | pure classifier unit (target `Some` + client-absent → arm c + its reason); widened-title unit; regression: the in-card "no forge client (.mcp.json)" reasons (app.rs:6888/:6944) unchanged |
| REQ-003 | WHEN the restored active root has NO orchestration entry (or `brain_endpoint` absent/empty), the header shall read exactly "Fleet" (byte-identical to #369/#376) and the unconfigured behaviors — demo verb, rail empty state — shall be unchanged. | classifier negative units mirroring t376_req003's four arms; the "Fleet" byte-pin unit carried; #369 rail + demo suites green unchanged |
| REQ-004 | WHEN a misconfigured reason is presented, the rendered text shall have passed the control-strip + ≤200-char clamp and shall contain no bearer material. | unit feeding a hostile configured value (control chars + >200 chars) → clamped output; §18.1 inspect: no bearer-bearing source flows into the reason builder (the only bearer sources are `.mcp.json` parse results, which the classifier consumes as a presence boolean) |
| REQ-005 | WHILE a subscription on a well-configured root is running, the header shall render "Fleet · live"/"Fleet · reconnecting" exactly as #376 shipped, and a transient reconnect shall never present as misconfigured. | exhaustive units over the widened input (every setup × wire combination); existing t376_req002 + the 4 `fleet_livewire` integration scenarios green unchanged |
| REQ-006 | WHEN the app boots, the misconfiguration determination shall be computed exactly ONCE, at the post-shell-restore start-gate site, keyed to the restored active root (never the launch cwd), and stored; the render shall read the stored determination only. | §18.1 inspect: one call site inside the app.rs:2192-2228 block, after the shell restore; classifier is a pure fn of resolved inputs (no ambient reads); render reads the field (the F4 discipline) |
| REQ-007 | WHEN a root is configured with an invalid `brain_endpoint` (arm b), a live capture of the running app shall show the "Fleet · misconfigured" header (and the reason at its resolved placement). | driven/headless render-executing capture receipt at Validate; if the machine state blocks driving, the recorded env-blocked protocol applies (documented explicitly, units + mechanism carry — the #204/#205 precedent) |

## Floors (constitution)
Pure seams at **cov/MSI 100**: the setup classifier, the widened title fn (exhaustive match over
the closed setup × wire space — no defensive catch-all), the reason builder + clamp. MASKED: the
boot-site population, the stored-field plumbing, the render call sites (app.rs is
coverage-excluded). Typed inputs, no `unwrap` on config-derived paths. The app.rs edits land near
the masked pump/boot shims — the skip-detach trap's home turf (5th strike was `pump_fleet_live`
itself, `BF-claude-skip-detach-pump-fleet-live-001`): re-run `cargo mutants --list -f` on the
ACTUAL touched files after placement, and re-verify neighboring `#[mutants::skip]` bindings.

## Phase Plan
- **P2 Design** — settle D-OPEN-STATE-SHAPE (typed setup state vs minimal widening — confirm the
  recommendation), C-PRECEDENCE, REASON-PLACEMENT (+ exact reason templates), DEMO-MISCONFIG,
  DIR-ARM; exact signatures + the classifier's input struct; the stored-field + render deltas;
  per-REQ test plan; verify #381's shipped resolution shape and bind the classifier to it.
- **P3 Implement** — **React-first: prototype the misconfigured header + reason caption in
  `marley-web` and confirm the look at localhost:5173 (per `## React-first (parity)`)**; then pure
  seams (classifier, widened title, reason builder/clamp), then the masked boot-site population +
  render delta.
- **P3.5 Inspect** — adversarial: can any target-Some outcome still end silent (walk every `?`/
  gate on app.rs:2202-2223)? does any reason path touch bearer material? does misconfigured ever
  mask a wire truth it shouldn't (C-PRECEDENCE holds)? arm (a) byte-identity? skip-detach
  re-check; provenance (§20).
- **P4 Validate** — write + RUN the units per REQ; `cargo mutants --list -f` on the actual touched
  files; gate green (`--diff`), cov/MSI 100 on the pure seams; #369/#376/#377/#378 regressions
  green; the REQ-007 capture (or the documented env-blocked fallback).
- **P5 Complete** — CHANGELOG; close the #376-F8 shelf note; AAR capture; archive; close #384.
