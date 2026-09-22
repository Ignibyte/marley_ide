---
pipeline_id: 365401bb-847d-4a00-81d3-bedd42efa8ba
ticket: docs/planning/tickets/open/TICKET-410-browser-forge-identity-rip.md
status: Phase 5 — Complete PASS
title: The browser loses its forge identity (product-rip slice 1)
type: chore
milestone: M30
references:
  - docs/planning/intake/scrap-forge-pivot.md
  - docs/planning/tickets/open/TICKET-411-forge-client-fleet-brain-rip.md
  - docs/marley_architecture/embedded-browser-model.md
---

## Title
Remove the embedded browser's forge identity — the `forge_web_base` origin chain,
the forge-flavored copy/keyword, and the dead `orchestration.web_url` fork — while
keeping the generic embedded-browser infrastructure (wry child, 27-state z-order
shim, #406 lifecycle machine) fully intact. Slice 1 of the scrap-forge product rip
(intake: `docs/planning/intake/scrap-forge-pivot.md`); MUST land before TICKET-411
because the browser chain holds independent edges into `marley_forge_client` that
would otherwise fall as collateral when the crate deletes.

## Scope
### In
- Delete the `forge_web_base` chain in `marley_app`: field (app.rs:539), boot
  derivation (:2489-2491), struct-literal init (:2735 — discovery addition), test
  hook (:2931-2933), `mount_plan` feed (:10913-10914), Retry re-derive arms
  (:11066-11096), `derive_forge_web_base_now` (:11111-11126), origin caption
  fallback (:19077-19081), and the two `mcp_json_path` call legs it owns.
- Re-home the pinned-origin nav predicate (`same_web_origin` + its `web_origin_of`
  parsed-host security core) out of `marley_forge_client`, or drop the pin —
  design fork F2.
- Retire the "Forge web view" placeholder copy (browser.rs:29-31) + repoint its
  exact-copy test; React POC twin (`BrowserPlaceholder.tsx:23`) rips FIRST.
- Drop the `"forge"` keyword from CommandId(31) "Browser: Reload" (app.rs:10439);
  the command itself survives.
- Decide + execute `orchestration.web_url`'s fate (dead — zero production
  readers; design fork F1: delete vs adopt as the post-forge origin source).
- Retire/repoint the 2 forge-identity headless drives; keep the 10 generic drives
  + 20 `browser_state` + 6 surviving `browser.rs` units green.
- Prose sweep of 410-scope forge comments in `marley_app` (browser.rs:21/:36/:193,
  grid_layout.rs:232, content.rs:367, browser_state.rs:344, lib.rs:72,
  webview_shim.rs prose).
- React POC parity: `BrowserPlaceholder.tsx` copy + `BrowserPane.tsx:33` ORIGIN
  stand-in doc updated first, visually verified, then the Rust port.

### Out (explicitly deferred)
- `mcp_config.rs` MODULE deletion — **re-scoped to TICKET-411** (discovery
  correction: its boot call site app.rs:2476 also feeds the sprint `forge_client`
  build :2480-2483 and the fleet-brain `endpoint_for_brain` bearer :2513, both
  411-scope; the ticket's "only two consumers" claim was wrong). 410 removes the
  `forge_web_base` legs only; the module + its 9 unit tests fall in 411 with
  their last consumers.
- The probe edge (`browser_state.rs:29` `ProbeOutcome`, app.rs:11049
  `probe_web_origin`, the probe livewire drives) — stays WIRED; 411 re-homes it
  into `browser_probe.rs`.
- The sprint/cockpit forge surface (`forge_view.rs`, ⌘⇧F, `RightSection::Forge`,
  status-bar sprint segment, fleet transport) — TICKET-411.
- Cockpit-Forge vocabulary inside the Browser section chrome (context_menu.rs
  row-0 "Forge", tabs.rs ＋-default `OpenCockpit(Forge)`) — discovery flagged it
  as owned by neither ticket; recorded into TICKET-411's doc (it dies with
  `RightSection::Forge`).
- `marley_forge_client` itself, `fleet_rail`/`fleet_live` wiring, CONSTITUTION
  preamble product words, `marley_forge_client.md`/crate-map arch docs — 411.

## Reference (§20)
N/A — Marley-specific. This pipeline removes Marley-specific product surface (the
forge-derived browser origin); no reference-app behavior is being matched. Warp
has no embedded-browser pane (no `docs/warp_architecture/` entry exists for one),
and the surviving generic browser infrastructure keeps the references its own
specs (#402/#405/#406, `embedded-browser-model.md`) already carry.

### Prior art
Swept 2026-08-09 (plan phase):
1. **Behavior maps** — no embedded-browser entry in `docs/warp_architecture/` or
   `docs/zed_architecture/` (grep: zero webview/browser-pane hits in warp docs);
   nothing to match for a removal.
2. **Published material** — the WHATWG URL spec's origin concept
   (https://url.spec.whatwg.org/#origin) is the semantic the pinned-origin
   predicate implements; no new adoption needed beyond leg 3.
3. **Permissive deps** — the `url` crate (2.5.8, MIT/Apache, already in-lock via
   `marley_forge_client`) OWNS the origin seam: `Url::origin()` → `Origin`
   (PartialEq tuple of scheme/host/port with `port_or_known_default`, WHATWG
   `ascii_serialization`, opaque origins for non-http schemes) — read at
   `~/.cargo/registry/src/…/url-2.5.8/src/origin.rs`. The existing
   `web_origin_of` already builds on `url::Url::parse` + `Host` + `is_tuple` +
   `ascii_serialization` (it is the #404 parsed-host fix); re-homing it is a MOVE
   of an adoption already made, not new invention. `marley_app` has no direct
   `url` dep today — F2 decides promotion vs pin-drop. No other in-tree crate
   (gpui, ropey, regex, alacritty_terminal) owns origin comparison.

## React-first (parity)
UI-AFFECTING — zone B (Browser pane surface; the frozen shell chrome is
untouched). The user-visible deltas: the B tab's placeholder detail copy loses
its forge sentence, and (per F1) the pane's origin source disappears so the
no-origin empty-rail-home state becomes the standing state. marley-web files
(port map: `marley-web/docs/MARLEY-PARITY.md`):
- `artifacts/marley-ide/src/components/views/BrowserPlaceholder.tsx` (:23 carries
  the exact literal the Rust copy test pins — the parity-critical edit)
- `artifacts/marley-ide/src/components/views/BrowserPane.tsx` (:33 ORIGIN
  stand-in doc'd as "Marley's #404-derived forge_web_base" — re-caption to the
  post-forge origin story F1 picks)
Plan line: build & visually verify in marley-web first (`pnpm --filter
@workspace/marley-ide run dev` → localhost:5173, screenshot + READ the PNG), then
port 1:1. Validate captures the React↔Marley parity pair. (`BrowserView.tsx`'s
`<ForgePane/>`-as-tab is documented design-ahead non-porting chrome — 411/cockpit
scope, untouched here.)

## Locked-In Decisions
- D1 — The generic embedded-browser infrastructure survives intact: wry child,
  27-state overlay z-order shim, #406 lifecycle machine, #404 empty-rail-home.
  The rip removes the ORIGIN SOURCE and forge vocabulary, not the machine.
- D2 — The probe edge (`ProbeOutcome` / `probe_web_origin`) stays wired through
  410; TICKET-411 re-homes it into `browser_probe.rs` (seam-map order preserved).
- D3 — `mcp_config.rs` module deletion re-scopes to TICKET-411 (discovery
  correction, recorded in both ticket docs). 410 deletes only the
  `forge_web_base` consumer legs.
- D4 — The B tab's no-origin state is the existing #404 empty-rail-home behavior;
  no new placeholder surface is invented.
- D5 — React-first ordering per the parity contract: the POC copy/caption edits
  land and are visually verified before the Rust port (M29 #403 lesson).
- D6 — Surviving weak-capture wiring in `webview_shim`/lifecycle is not
  re-plumbed (PR-claude-callback-stored-inside-owned-resource-captures-weak-001).

Design forks for Phase 2 (named, not locked):
- F1 — Post-forge origin source: none-until-a-future-URL-feature (delete
  `orchestration.web_url`) vs adopt `web_url` as the origin source now.
- F2 — `same_web_origin` re-home shape: move predicate + `web_origin_of` core
  into `marley_app` (promote `url = "2"` to a direct dep) vs drop the origin pin
  (a nav-security posture change — BF-claude-wall-admit-vs-parsed-host-
  divergence-001 binds any surviving parse).
- F3 — The replacement placeholder detail copy (with its POC twin).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method (gate exit code,
negative smoke, or review).

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `marley_app` boots (any `.mcp.json` present or absent), the system shall derive no browser origin from `.mcp.json` — the `forge_web_base` chain is absent | `grep -rn 'forge_web_base' crates/marley_app/src/` = 0 hits; `cargo build` green |
| REQ-002 | The `marley_app` crate shall hold no reference to `marley_forge_client::{forge_web_base, same_web_origin}` (probe edge exempt per D2) | grep over `crates/marley_app/src/` = 0 hits for those two symbols; compile green |
| REQ-003 | WHEN the Browser B tab is open with no origin source, the pane shall render the #404 empty-rail-home (headline "No page loaded" survives; detail copy carries no "Forge") | repointed `placeholder_copy_is_exact` + the repointed retry/unconfigure headless drive RUN green |
| REQ-004 | WHEN the palette is filtered by "forge", the system shall no longer surface CommandId(31); "Browser: Reload" shall keep its action under its generic keywords | palette unit(s) RUN green; grep app.rs:10439 keyword list has no "forge" |
| REQ-005 | The `ProjectOrchestration` settings schema shall reflect F1's locked outcome (field deleted or adopted — no dead `web_url` remains) | settings round-trip units RUN green; grep per F1 |
| REQ-006 | The surviving browser surface shall stay green: 10 generic headless drives, 20 `browser_state` units, 6 `browser.rs` units, probe livewire block | `cargo nextest run -p marley_app` targeted filters, RUN in Phase 4 |
| REQ-007 | The React POC browser placeholder/pane shall carry the same post-forge copy/caption as the Rust side (parity pair captured) | POC dev-server screenshot READ + Rust-side copy test; parity pair recorded in notes |
| REQ-008 | The full diff shall pass the delivery gate | `scripts/gates.sh --diff` exit 0 (receipt written) |

## Phase Plan
- **P2 Design** — resolve F1/F2/F3; file manifest; regression test plan; confirm
  Reference (§20) N/A + prior-art adoption stance.
- **P3 Implement** — React POC edits first (visual verify), then the Rust rip per
  manifest.
- **P3.5 Inspect** — independent critics vs the diff (correctness / security-
  posture of the pin decision / provenance / simplification); fix real findings.
- **P4 Validate** — write/repoint the planned tests; RUN them + gate `--diff`
  green.
- **P5 Complete** — CHANGELOG; arch docs (`embedded-browser-model.md` §Q3/Q4,
  `orchestration-shell.md` §8, `app_shell.md:954-958`, `pane-composition-
  model.md`, `roadmap.md` Phase E); ledger capture (§19); archive, close the
  ticket (its BACKLOG row already left at promotion).
