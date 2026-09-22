# Cockpit onto the registry — the residency half (the #388 slice-8) — Notes

- **Forge ticket:** #400 `95954769-f57f-4167-99ff-e8b01bfd5756` (sprint #39 `6558258f-a0d2-45fb-b601-cd5329a1cd77`)
- **AAR:** adb07e4f-5271-4895-a691-765631dbcf2c (opened 2026-08-05 at promotion)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-400-cockpit-residency.md
- **Pipeline spec:** 400-cockpit-residency.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** the #388 train slice-8, cockpit half only — Details/Agents/Forge become
  `Content::Cockpit` registry residents (one-instance-many-views); Browser half stays gated on
  #389/Phase E. Largely structural; M; milestone M28.
- **Classification / tier:** feature, M. Hard-ordered after #396 (its `Content` enum declares
  `Cockpit`; also ships `RootView.content: ContentRegistry<Content>` + the migration pattern — #396 is
  OPEN in the current sprint, so /work must not promote #400 first). #398 (slice-5) SOFT — exposure
  half only; residency needs only #396 and may land before #398 (#398 also needs #397).
- **Forge recall (§18.3):** knowledge-search run on "right dock cockpit section state" + "content
  registry lifecycle" (hits returned as ids; names recovered from the repo's own AAR captures —
  docs-search was cross-project-polluted this session, noted for hygiene). Applicable by name:
  `PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-001` (the registry's load-bearing
  catch — this ticket adds the INVERSE guard: pinned residents that must never drop),
  `PR-claude-registry-release-returning-a-resource-must-be-must-use-001` (every new cockpit
  `release_view` site), `PR-claude-closed-enum-gate-exhaustive-match-not-predicate-chain-001` (the new
  `Content::Cockpit`/section match arms), `BF-claude-skip-detach-pump-fleet-live-001` +
  `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators` (app.rs masked-shim edits),
  `AD-claude-pane-content-id-registry-001` + `AD-claude-one-instance-many-views-001` (the two ADs this
  slice extends to the cockpit kind).
- **Discovery (the edit surface for Design):** right_dock.rs:11-81 (vocabulary — unchanged);
  tabs.rs:24/28/:127/:153/:195/:199/:411-424 (`TabContent::Cockpit` → id-bearing; `open_or_switch`
  takes the resolved id); grid_layout.rs:217/:355/:411 (`C=<key>` codec — byte-identical, ids never
  serialize); app.rs:433/:2387 (`right_section` selection — stays), :2038-2044 (restore →
  resolve-or-register), :3689-3700 (`open_cockpit_section`), :4993-4994 (serialize), :5396-5550
  (`cockpit_body` — resolve through registry, render byte-identical; #391 `try_workspace` totality
  preserved), :19071 (top-bar strip — unchanged), close paths per the #396 audit;
  content_registry.rs:56/:66/:78 (the #394 lifecycle — generic code untouched; pinning is an app-side
  anchor-view policy); settings.rs:130/:571 (#95 key — untouched). Surface data stays app-side:
  agents :190, remotes :382, forge_sprint :389, forge_pending :392 (pure-projection modules
  pane_details.rs/agent_view.rs/forge_view.rs unchanged). Cross-project wrinkle: two projects' tabs of
  one section become the app's FIRST >1-view content — the acquire/release machinery gets its first
  real exercise here.
- **Decisions:** D1-D5 locked (cash the #396 declaration; app-wide singletons; selection semantics
  unchanged; byte-identical dock render; Browser OUT). D-OPENs: SINGLETON-LIFECYCLE (recommend PINNED
  via a standing dock-anchor view — no generic-registry special case; register-at-boot preferred),
  STATE-MIGRATION (recommend identity-only; all service state stays app-side),
  KIND-GATING (recommend gated-behind-follow-up unless #398 shipped registry-generic AND zero new
  cockpit render code — evidence: no add-to-pane machinery in the tree today, and `PaneContent`
  (workspace.rs:317) has no Cockpit cell arm). React-first arm: **N/A expected (pure residency),
  conditional on Phase 2's re-check** — the spec carries the flip protocol + the ContentId↔PaneItem
  vocabulary tie. Reference (§20): N/A Marley-specific, with the Zed map's "no uniform panel
  citizenship" GAP row (zed 07) as the mapped analog this slice closes and VS Code's published
  movable-panels convention for the later exposure; Warp 04 checked, no owner; permissive deps
  checked, no owner — the in-house #394/#396 seams own this.

## Phase 2 — Design

### Shipped-seam re-verification (2026-08-05 — the spec predates #396–#399 shipping; every assumption re-checked)
- **content.rs (#396/#397):** `Content::Cockpit` is DECLARED **payloadless** (`Cockpit`, :42) — the spec's
  candidate `Cockpit(RightSection)` is the cash-in. `ContentKind::Cockpit.addable() == false` (:70-78) is
  #398's shipped verb gate — the add-to-pane menu can never offer cockpit while it holds. The module already
  homes the editor lifecycle seams (`resolve_open`/`release_editor_views`) — the cockpit seams land beside
  them (the #397 precedent).
- **workspace.rs:341:** `PaneContent { Terminal(ContentId), FileTree, CodeView(EditorSurface), Git }` — NO
  cockpit cell arm. The exposure is NOT free at the render layer → **D-OPEN-KIND-GATING = gated behind a
  follow-up; `addable(Cockpit)` stays `false`** (doc comment updated to cite #400: registry-resident, but no
  pane-cell arm yet). **React-first arm = N/A — no UI delta (pure ownership migration).**
- **registry (#394):** `insert` starts the entry at ONE view; `acquire_view`/`release_view` both
  `#[must_use]`; ids never serialize; `ContentId::test` is the cfg(test) mint for tab-suite plumbing.
- **tabs.rs:** `TabContent::Cockpit(RightSection)` :29; `Tab::cockpit` :161; accessors :192-267 (all
  match-exhaustive); `open_or_switch_cockpit` :422 (find-by-section → switch | append);
  `SectionAction::OpenCockpit` :97 (the section-＋ menu verb). `active_tab()` :343.
- **app.rs doors (complete):** OPEN ×2 — `open_cockpit_section` :3874 (sets `right_section` +
  `persist_right_section` + notify) and `SectionAction::OpenCockpit` :7195 (menu verb — `persist_grid`, no
  selection change; the asymmetry is shipped behavior, preserved). RESTORE :2213 (`TabLayout::Cockpit` →
  `Tab::cockpit`). SERIALIZE :5198 — reads the `cockpit_section()` TAG → `TabLayout::Cockpit(section)` —
  **untouched** (ids never serialize; codec byte-identical with ZERO grid_layout.rs edits). RENDER :18298 —
  `active_tab().cockpit_section()` → `cockpit_body(section, …)` :5748 (body internals untouched). FOCUS
  :20426 (`FocusTab::Cockpit` pattern arm). CLOSE ×2 — `close_tab_at` :8348 + `close_project_at` :8487 (both
  collect-ids-then-release; cockpit collects nothing today). A cockpit lives ONLY at tab level (no cells) →
  the close-path universe is exactly these two + process exit (quit drops everything; no release needed —
  the terminal precedent).
- **right_dock.rs:** `RightSection` derives Copy/Eq but NOT Hash → the singleton index is an
  **array-slot** (`[Option<ContentId>; 3]` with an exhaustive-match slot fn), not a HashMap.

### D-OPEN resolutions
- **D-OPEN-SINGLETON-LIFECYCLE = PINNED via the insert-anchor.** `insert` acquires the first view — that
  view IS the standing dock anchor, held by the `CockpitIndex` forever (no code path releases it). Tab
  views stack on top via `acquire_view`; every user-reachable release therefore returns `None` by
  construction (`debug_assert`ed at the release seam — REQ-005's "asserted, not assumed"). The generic
  `ContentRegistry` is untouched — pinning is pure app-side policy.
  **Sub-fork: LAZY resolve-or-register — DIVERGES from Phase 1's register-at-boot lean, with reason:**
  boot-registering 3 entries would shift the registry len/iter baselines of every existing #396/#397/#398
  suite (+3 in every headless drive) for zero behavioral gain; lazy keeps all non-cockpit tests
  byte-stable. The race register-at-boot was buying protection from does not exist: restore and the
  interactive opens all pass through the ONE `resolve_or_register_cockpit` door, and the door self-heals
  the (structurally impossible) dead-id desync by re-registering — unit-provable.
- **D-OPEN-STATE-MIGRATION = identity only.** Payload is `RightSection`; `right_section` selection (#95),
  `agents`/`remotes`/`forge_sprint`/`forge_pending`, and the three pure projection modules stay app-side,
  untouched.
- **D-OPEN-KIND-GATING = gated behind the follow-up** (evidence above). The visible delta of this slice is
  ZERO — even the add-to-pane menu is unchanged because `addable` stays false.

### Architecture
- **content.rs — the pure cockpit lifecycle seams** (beside the #397 editor seams; NEW code stays
  generic-free per `PR-claude-cov-100-pure-modules-stay-generic-free-001` — slice params, no closures):
  - `Content::Cockpit(RightSection)` + `as_cockpit(&self) -> Option<RightSection>`; `kind()` arm adjusts.
  - `CockpitIndex { slots: [Option<ContentId>; 3] }` — `get(section)`, private exhaustive-match slot fn.
  - `resolve_or_register_cockpit(reg, idx, section) -> ContentId` — the ONE door: live slot hit → the same
    id; miss (or dead-id heal) → `insert(Content::Cockpit(section))` (the first view = the pinned anchor)
    + slot write.
  - `release_cockpit_views(reg, ids: &[ContentId])` — per id: `let reaped = release_view(id);
    debug_assert!(reaped.is_none(), …); drop(reaped)` (binding + drop consumes the `#[must_use]` without a
    release-build unused-var warning; `Option<Content>` is a needs-drop type so clippy's `drop_non_drop`
    stays quiet).
- **tabs.rs — id-bearing tab citizenship:** `TabContent::Cockpit(ContentId, RightSection)` (the #388 Q1
  `(id, tag)` shape — `cockpit_section`/`is_cockpit`/`rail_section`/`key_context` keep reading the TAG,
  zero registry reads); `Tab::cockpit(title, id, section)`; NEW `cockpit_content_id() -> Option<ContentId>`
  (the close-path collector); `open_or_switch_cockpit(section, id) -> bool` — switch arm ignores the id
  and returns false; append arm builds `Tab::cockpit` with it and returns true (the caller acquires — the
  inverse of #397's speculative-release idiom, exact because the anchor decouples insert from tab views).
- **app.rs — the masked wiring:** field `cockpit_index: crate::content::CockpitIndex`; ONE helper
  `open_cockpit_tab(&mut self, section)` = resolve → `open_or_switch_cockpit(section, id)` → on `true`,
  `let _ = self.content.acquire_view(id)` (structurally infallible — just resolved; no never-true branch
  to leave uncovered); both open doors route through it (:3874 keeps selection+persist_right_section+notify,
  :7195 keeps persist_grid). RESTORE arm: resolve + acquire + `Tab::cockpit(label, id, *section)` inline
  (a fresh boot's index is empty → first restored tab of a section registers, later ones share). CLOSE:
  both paths collect `removed … cockpit_content_id()` → `release_cockpit_views`. RENDER :18298: match the
  active tab's `(id, tag)` → `self.content.get(id).and_then(Content::as_cockpit).unwrap_or(tag)` →
  `cockpit_body(section, …)` — total (the tag fallback can never disagree: both are set from one value at
  birth and neither ever mutates — unit-pinned), body internals untouched → byte-identical pixels.
- **Untouched by design:** grid_layout.rs (codec), right_dock.rs, settings.rs (#95), status_bar.rs, the
  three projection modules, marley-web (N/A arm).

### File manifest
| file | change |
|---|---|
| `crates/marley_app/src/content.rs` | `Cockpit(RightSection)` payload + `as_cockpit`; `CockpitIndex`; `resolve_or_register_cockpit`; `release_cockpit_views`; `addable` doc cite; tests |
| `crates/marley_app/src/tabs.rs` | id-bearing `TabContent::Cockpit`; `Tab::cockpit(title, id, section)`; `cockpit_content_id()`; `open_or_switch_cockpit(section, id) -> bool`; pattern arms; suite updates + new pins |
| `crates/marley_app/src/app.rs` | `cockpit_index` field; `open_cockpit_tab` helper; 2 open doors routed; restore arm; 2 close paths collect+release; render dispatch resolve; `FocusTab` arm pattern |
| `crates/marley_app/src/headless_drive.rs` | residency drives: restore-resolves-shared-id (double-boot), close/reopen anchor-held, cross-project twin + leak pairing |

### Regression test plan
| REQ | tests |
|---|---|
| REQ-001 | content.rs: `as_cockpit` both-flavors + `kind()` arm units; tabs.rs: accessors read the TAG on a `ContentId::test` tab (no registry in scope — purity by construction); render resolve exercised by the headless drives (cockpit tab renders through the id) |
| REQ-002 | content.rs: resolve miss-registers (len +1, view 1) / hit-returns-same-id (len stable, no view acquired) / dead-id self-heal / three sections → 3 distinct ids, re-resolving all → still 3 (≤3 pin); serialize site untouched + grid_layout suites green unchanged (byte-identity by NO-EDIT); headless double-boot: persisted `C=` tabs resolve through the door, same id per section across projects |
| REQ-003 | `cockpit_body` internals + strip untouched (byte-identity by construction); tag==payload equality unit; live before/after capture pair via the #399 sandbox-HOME harness (before = pre-implement HEAD build), pixel-compared; fallback = the documented env-blocked protocol |
| REQ-004 | tabs.rs `open_or_switch_cockpit_cases` (updated signature: asserts the appended bool + same-id switch), `cockpit_section_cases`; right_dock.rs suite, #95 settings round-trip, status_bar `FocusTab` — all green unchanged |
| REQ-005 | content.rs: release on anchor+tab views → entry SURVIVES (`view_count` ≥ 1, `get` Some); release every tab view across two projects → survives; headless: close cockpit tab → reopen → SAME id (anchor held) |
| REQ-006 | content.rs: stale/unknown id release is a safe no-op (debug_assert passes — `None` either way); headless leak-pairing: view_count exact across open/second-project/close-tab/close-project sequences; every release site consumes the `#[must_use]` (binding+drop / `let _`), §18.1 site audit |
| REQ-007 | Phase 3 notes record "React-first: N/A — no UI delta (pure ownership migration)" + the ContentId↔PaneItem vocabulary tie |

### Risks / decisions
- `open_or_switch_cockpit` signature ripple is contained to tabs.rs suites + 2 app doors (both routed
  through the one helper).
- app.rs edits sit near masked shims (`refresh_forge` et al. carry `mutants::skip`) — Validate re-runs
  `cargo mutants --list -f` on the touched files and re-verifies neighboring skip bindings (the 5th-strike
  `BF-claude-skip-detach-pump-fleet-live-001` protocol).
- The render tag-fallback arm lives in masked app.rs (coverage-excluded) — the pure equality (tag ==
  payload at birth, both immutable) is what the unit pins; no uncovered pure branch anywhere (the
  `let _ = acquire_view` idiom deliberately avoids a never-true branch in covered code).
- LAZY registration means a section never opened this session has no registry entry — `≤3`, not `== 3`;
  REQ-002's pin asserts ≤ and exact ids, not a fixed census.

## Phase 3 — Implement
- **React-first: N/A — no UI delta (pure ownership migration).** The vocabulary tie holds:
  Marley's `ContentId` stays aligned with the POC's `PaneItem` stand-in (MARLEY-PARITY.md § Shared
  vocabulary) so the slice-5/-6 exposure ports cleanly; `addable(Cockpit)` stays `false`, so even
  the add-to-pane menu is unchanged. No marley-web edits.
- **Built exactly to the manifest:**
  - `content.rs` — `Content::Cockpit(RightSection)` cashed (+`as_cockpit`); `CockpitIndex`
    (array-slotted, lazy); `resolve_or_register_cockpit` (the ONE door, self-healing);
    `release_cockpit_views` (debug_assert `None` — the pinned invariant, asserted); `addable`
    doc updated (resident but no cell arm → stays false). New units: classify/resolve,
    singleton door (miss/hit/trio census), pinned-release survival, release safety + self-heal.
  - `tabs.rs` — `TabContent::Cockpit(ContentId, RightSection)`; `Tab::cockpit(title, id,
    section)`; `cockpit_view()`/`cockpit_content_id()` (new pure accessors);
    `open_or_switch_cockpit(section, id) -> bool` (append=true → caller acquires); all pattern
    arms widened `(..)`; suites updated (id-bearing helper, appended-bool + view-id pins).
  - `app.rs` — `cockpit_index` field; restore arm resolves + acquires per restored `C=` tab;
    `open_cockpit_tab` helper (resolve → open-or-switch → acquire-if-appended); close_tab_at +
    close_project_at collect `cockpit_content_id()` → `release_cockpit_views`; render dispatch
    resolves id→`Content::as_cockpit` with the tag as total fallback; `FocusTab` arm widened.
  - grid_layout.rs / right_dock.rs / settings.rs / status_bar.rs / marley-web: ZERO edits (codec
    and selection semantics byte-identical by no-edit).
- **Deviation from design (found by the compiler):** the open-door census was THREE, not two —
  the `"toggle-right-dock"` verb (⌘⇧B → Details cockpit tab, the #153 retirement shim) also
  called `open_or_switch_cockpit` directly. Routed through the same `open_cockpit_tab` helper.
  Root cause of the miss: the design's caller-grep targeted `open_cockpit_section`/`SectionAction`
  spellings, not the raw `open_or_switch_cockpit` method name across app.rs. Inspect re-walks the
  full caller census.
- **REQ-003 evidence prep:** pre-implement "before" captures taken from the #399 sandbox-HOME
  harness (fixed geometry, all three sections seeded via `C=` restore):
  `scratchpad/400-sandbox-home/captures/before/{forge,agents,details}.png`.
- `cargo check --workspace --all-targets` green.

## Phase 3.5 — Inspect
Four independent critics over the working-tree diff (lenses: reap-safety/leak-pairing ·
correctness-vs-EARS · data/state integrity · reuse/provenance/hygiene). Headline verdicts: the
acquire/release pairing is **airtight** (no reap, no leak, no double-release constructible — the
removal universe is closed at exactly two paths, both balanced; adversarial sequences incl.
double-close, duplicate same-section `C=` blobs, and cross-project interleavings all stay
balanced), **zero functional defects** against REQ-001/002/004/005/006, codec byte-identity and
the #95 selection semantics verified untouched, all 24 diff mutants in content.rs/tabs.rs
dispositioned to killing asserts, and provenance/generic-free discipline clean.

| # | sev | finding (critic) | verdict | fix |
|---|-----|------------------|---------|-----|
| 1 | med | gate:1 rustfmt red on three new test lines (c2, c4) | REAL | `cargo fmt --all` run; `--check` clean |
| 2 | med | `open_cockpit_tab` is the one unskipped new app.rs mutant; nothing in-tree kills it (c2, c4) | REAL — deliberate drive-killed class | HANDED TO PHASE 4 as a named obligation: the residency drive must open a cockpit tab **through a real door that reaches `open_cockpit_tab`** (`dispatch_for_test("toggle-right-dock")`) and assert tab + view_count — a seed-only drive would not kill it. No skip attr added (matches the app.rs drive-killed class) |
| 3 | low | `release_cockpit_views` accepts any id; a misrouted terminal id would inline-drop a PTY in release (c1) | REAL hardening | pre-release `debug_assert` (cockpit-ids-only, catches ANY live non-cockpit id regardless of view count) + the CONTRACT stated in the doc |
| 4 | low | `open_cockpit_tab` funneled three doors onto panicking `active_project_mut()` (c1) | REAL — the #392 total idiom applies to the new seam | total `project_mut(active)` + early return |
| 5 | low | stale doc: `ContentKind::Cockpit` "payload lands at #400" (c1) | REAL | reworded ("identity payload landed at #400; the KIND stays payload-free") |
| 6 | low | acquire sites used `let _ =`, diverging from the shipped None-handling idiom; infallibility verified but unasserted (c2) | REAL | both sites now `let acquired = …; debug_assert!(acquired.is_some(), …)` (binding read in all profiles; the side-effect-in-debug_assert trap avoided) |
| 7 | low | Details/Agents/Forge label match duplicated (pre-existing #153/#163 copies; right_dock owns the pairs) (c4) | REAL, in-scope-adjacent | `right_dock::section_label` extracted (ONE mint); `section_tabs` + both touched copies consume it; exact-words unit added (kills the `""` mutants). context_menu's menu-row copies left (different surface, untouched files) |
| 8 | nit | `drop(reaped)` ceremony (binding alone consumes must_use) (c4) | ACCEPTED as-is | kept — explicit discarded-on-purpose signal; critic confirms the bind-assert shape is load-bearing (folding the call into the assert would compile the release out) |
| 9 | info | test ids reused across sections in `open_or_switch_cockpit_cases` (c2) | REAL nit | distinct ids per appended section |
| 10 | info/low | headless drives absent from the diff (c1, c3) | EXPECTED — Phase 4 owns test authorship | drives blocked-on at Validate with three recorded constraints: (a) finding-2's real-door requirement; (b) `reap_sessions` doesn't reset `cockpit_index` — drives boot FRESH windows and lean on the self-heal knowingly; (c) the existing seeded-boot `registry_len == terminals` assert constrains seeds — new drives use their own seeds |
| 11 | info | zero-project dispatch panic reachability through the doors (c3) | PRE-EXISTING class; the cockpit seam now total via finding 4 | no further change (launcher short-circuits before the affordances render) |
| 12 | info | switch arm ignores the passed id (c3) | BY DESIGN — sole-id-source invariant | contract stated in the doc comment; pinned by the ignored-id test |
| 13 | info | release-build posture of a violated pinned invariant (c3) | BY DESIGN | silent trivial drop + index self-heal + total tag fallback — "asserted, not assumed" confirmed |

Post-fix verification: `cargo fmt --check` clean; `cargo clippy -p marley --all-targets` zero
warnings; `content:: + tabs:: + right_dock::` suites **83/83 green**.

## Phase 4 — Validate
- **Units (Phase 3.5-verified + this phase):** content.rs 4 new (classify/resolve; singleton door
  miss/hit/trio-census; pinned-release survival; release safety + forced-desync self-heal),
  right_dock.rs 1 new (`section_label` exact words), tabs.rs suites updated to the id-bearing
  contract (appended-bool, ignored-id-on-switch, view-id pins). 83/83 across the three modules.
- **Headless drives (3 new, the inspect finding-10 constraints honored — own seeds, fresh boots):**
  - `cockpit_restore_shares_the_singleton_across_projects_headless` — REQ-001/002: two projects'
    `C=forge` tabs resolve ONE id (view_count 3 = anchor + 2 tabs; the app's first legitimately
    >1-view content), agents distinct (vc 2), census exactly the 2 resolved sections, active
    cockpit tab renders; then a REAL second boot from the same dir re-derives it all on fresh ids.
  - `cockpit_door_pins_close_and_reunites_headless` — REQ-005/006 + the finding-2 mutant kill:
    `dispatch_for_test("toggle-right-dock")` (the REAL door → `open_cockpit_tab`) appends at vc 2;
    re-dispatch SWITCHES (no tab, no view); `close_tab_at` releases to vc 1 with the resident
    ALIVE; reopen reunites with the SAME id.
  - `cockpit_close_project_releases_but_never_reaps_headless` — REQ-005/006 cross-project:
    project-close releases its twin's view (3→2), tab-close leaves the bare anchor (→1, alive),
    the door reunites with the pinned id.
- **Full suite:** `cargo nextest run --workspace` **2033/2033 passed** (5 skipped, the standing
  env-gated set) + doctests green.
- **REQ-003 live proof (the sandbox-HOME harness, fixed geometry):** before-captures from the
  pre-implement HEAD build vs after-captures from the #400 build, identical seed (T= + C=forge +
  C=agents + C=details, forge active): all three sections **`cmp` BYTE-IDENTICAL** —
  `400-sandbox-home/captures/{before,after}/{forge,agents,details}.png`. Live codec round-trip:
  the ⌘Q re-persisted wire is `T=<blob>·C=forge·C=agents·C=details` with no ids anywhere and
  `right_section = "forge"` untouched by rail tab clicks (the shipped selection semantics).
- **Parity:** React-first N/A (pure residency) — no parity pair owed; `enforce-react-parity.sh`
  satisfied by the Phase 3 N/A record + vocabulary tie.
- **Mutants:** `cargo mutants --list -f` re-run on the touched files; the one unskipped new app.rs
  mutant (`open_cockpit_tab with ()`) is killed by the door drive — proven by gate:5 below. No
  skip-detach (Phase 3.5 audit; attributes re-verified attached).
- **Gate:** run 1 RED at gate:14 only — rustdoc intra-doc link from the public `Content::Cockpit`
  variant to the crate-private `resolve_or_register_cockpit` (the content module is private with
  selective re-exports; links from re-exported items must not target unexported seams). Fixed at
  source (plain code spans). Run 2: **GATE GREEN [diff] 15/15** — coverage 100% lines, MSI 100,
  miri green, visual/AX green; receipt at `.git/ignibyte-gate-receipt`.
- Pre-existing exclusions: none touched; the 5 nextest skips are the standing env-gated set.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG.md — full TICKET-400 [Unreleased]/Changed entry above #399's;
  `docs/marley_architecture/pane-composition-model.md` — Q5 slice-8 → "✅ SHIPPED (M28 #400)"
  (cockpit half; Browser stays gated on #389) with the as-shipped shape (singleton door, pinned
  anchor, id+tag, addable stays false). Parity sync: `marley-web/docs/MARLEY-PARITY.md` § Shared
  vocabulary gains the #400 paragraph — pure ownership migration, NOTHING ports; the tie is that
  a cockpit surface's identity is now a ContentId, so the future slice-5 exposure rides the same
  AddToPane/PaneItem vocabulary. No marley-web code (the N/A arm held through validate).
- **Forge capture:** failure `BF-claude-caller-census-grepped-wrapper-names-not-the-callee-001`
  (medium — the door census counted wrapper spellings, missed the raw ⌘⇧B caller; compiler
  caught it; the #399 census-family sibling) + prevention rule
  `PR-claude-a-caller-census-greps-the-callee-name-001` (grep the CALLEE's own name
  workspace-wide; where feasible let an arity change make the compiler emit the census).
  Lessons for the AAR: (1) the insert-anchor pattern pins a resident with ZERO changes to the
  generic registry — lifecycle policy staying app-side kept #394 pure; (2) LAZY registration
  beat register-at-boot on evidence Phase 1 couldn't have (existing suites assert registry
  census — boot-registering would have shifted every baseline); (3) the byte-identity claim was
  provable for free by seeding identical sandbox state and `cmp`-ing PNGs across builds —
  cheaper and stronger than pixel-sampling; (4) rustdoc intra-doc links from RE-EXPORTED items
  must not target unexported seams (gate:14's first red on this class).
- **AAR** adb07e4f submitted (completed). **Ticket:** local doc → tickets/closed/ (status:
  closed); forge #400 → done. **Pipeline pair** → completed/.
