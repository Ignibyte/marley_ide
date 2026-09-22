---
pipeline_id: a206dcb4-2df7-4597-8097-061939c3b5db
ticket: forge#396 (cd706b38-4b96-4390-a7c0-86339d96f086) · local docs/planning/tickets/open/TICKET-396-terminals-onto-registry.md
aar_id: a9c14f62-2a23-458a-85a6-d88d43b7727c
status: Phase 5 — Complete PASS (auto run, 2026-08-04 — CHANGELOG + pane-composition-model Q5 slice-2 SHIPPED, AAR a9c14f62 submitted with 2 failures + 2 prevention rules, forge #396 done, archived)
title: Terminals onto the ContentId registry — the load-bearing ownership migration (the #394 slice-2 capstone)
type: feature
milestone: M28 (carried from M27; the #388 train slice-2)
references:
  - docs/marley_architecture/pane-composition-model.md
  - docs/planning/pipeline/completed/394-content-registry.spec.md
  - docs/planning/pipeline/completed/394-content-registry.notes.md
  - crates/marley_app/src/content_registry.rs
  - crates/marley_app/src/workspace.rs
  - crates/marley_app/src/tabs.rs
  - crates/marley_app/src/grid_layout.rs
  - crates/marley_app/src/app.rs
  - crates/terminal_blocks/src/pty_os.rs
  - crates/terminal_blocks/src/session.rs
---

## Title
The #388 train slice-2 — the D5 split's L half (394-content-registry.notes.md §(0): #394 shipped the
pure generic `ContentRegistry<C>` lifecycle, deliberately WITHOUT residents; THIS ticket is the
terminal-OWNERSHIP migration onto it, its own reviewed ticket so a half-migrated reaper/persistence
never ships). Five pieces, per the ticket + the #388 Q3 ripple table:

1. **The concrete `Content` enum** — all 6 kinds declared per the #388 doc (`Terminal` WIRED with the
   owned payload; Editor[=CodeView]/Git live-but-later; Cockpit/Browser net-new declared — the
   #385-comment discipline: a new kind picks its variant HERE); `RootView` gains
   `content: ContentRegistry<Content>` constructed at boot.
2. **Terminal ownership migrates:** `PaneContent<S>::Terminal(Box<TerminalPane<TerminalSession>>)`
   (workspace.rs:317/322 at ticketing) → the pane/tab cell holds a **`ContentId` view**; the ~46
   app.rs accessor sites (`focused_terminal`/`_mut` ×~40 + `states`/`_mut` ×6; defs
   workspace.rs:627/633/705/710 at ticketing) resolve through the registry; `kind()`/`rail_section()`
   stay PURE (the id rides a per-kind variant or an id+`ContentKind` tag — the #388 open-Q1
   recommendation; exact shape is D-OPEN-VIEW-SHAPE).
3. **The close→reaper contract moves into the registry:** the 7 spawn-drop sites (5 user: pump
   dead-pane ×2, close-tab, close-project, close-focused ×2→1?, close-pane; + 2 boot-cleanup — the
   ticket's line refs are 2026-07-23 and STALE; P2 re-inventories) route through
   `release_view(id)` → the registry returns the owned content ONLY on the last-view drop → the
   CALLER keeps the ~600ms off-thread `thread::spawn(drop)` reap (`OsPtyChannel` Drop / the
   TICKET-348 bounded reap in terminal_blocks/pty_os.rs + session.rs `reap_step` — UNCHANGED; only
   WHO holds the session until the last view changes). The `#[must_use]` returns (the #394 inspect
   F1) are consumed at EVERY site. Byte-identical to today for the 1-view case (every view is 1-view
   until add-to-pane/#398).
4. **Persistence rebuild:** restore constructs sessions from the restored grid shapes exactly as
   today, then `insert`s them into the registry; ids NEVER serialize; the #163/#205/#177 round-trips
   stay byte-identical (the codec stays coordinate/shape-based — the #394 hard invariant).
5. **The app-side maps re-key** from `PaneId` → `ContentId` where they track the INSTANCE
   (`agents`/`remotes`/`notify_ticks` — agents follow content, not the cell); their consumers'
   behavior is unchanged (behavior-neutral re-key; today's scrub/leak landscape preserved, #226's
   known gaps neither fixed nor widened here).

Zero user-visible change — the full suite is the proof. All line refs in this spec are the
TICKETING-TIME tree (2026-07-23; app.rs has since moved ~+250 lines via #352): **Phase 2's first act
is the fresh inventory at HEAD** (the #352 lesson: sweep equality/comparison sites too —
PR-claude-domain-sweep-includes-equality-gates-001).

## Scope
### In
- `Content` enum (marley_app; home = D-OPEN-ENUM-HOME) + `RootView.content: ContentRegistry<Content>`
  at boot; the `pub use` gate-safety idiom carried if any new pub item needs it (#371/#394).
- `PaneContent`/`PaneState`/`PaneGrid` (workspace.rs) + `TabContent` (tabs.rs) hold `ContentId` views
  for terminal content; the accessor spine re-resolves through the registry; pure fns stay pure.
- The S-genericity simplification IF it falls out (the #388 open-Q2: with content in the registry the
  model may stop being generic over the session handle — D-OPEN-S-GENERICITY; P2 sizes it and may
  keep it minimal to bound the diff).
- Every terminal close/replace/error path routes `release_view`; the off-thread reap at each caller;
  leak-path symmetry (every acquire paired with exactly one release).
- Restore/boot rebuild through `insert`; the two boot-cleanup drop sites re-routed.
- The `agents`/`remotes`/`notify_ticks` re-key + their close-path scrubs moved 1:1.
- Regression pins: the FULL suite byte-identical (~1990 tests incl. #163/#205/#177 round-trips, #390
  rail, #382 focus, #348 reap, #394 registry matrix); new registry-integration units per the EARS.

### Out (explicitly deferred — later train slices)
- Editors onto the registry (#397, slice-3 — the #259 two-Buffers collapse; the `Editor` variant is
  DECLARED here, its payload lands there).
- Add-to-pane / one-instance-many-views (#398, slice-5 — view_count > 1 becomes REACHABLE there;
  here every view stays 1-view).
- Nameable arrangements (#399, slice-6); cockpit/browser residency (#400, slice-8; browser gated on
  #389).
- Any codec change (leaves stay `t=<cwd>`/`c=<path>`/…); any UI/behavior change; the #226
  notify-scrub gaps (preserved as-is); multi-workspace (slice-7).

## Reference (§20)
**N/A — Marley-specific composition model.** The #388 doc's central fork (Marley's OWN id-keyed
registry, gpui `Entity<T>` evaluated-and-REJECTED for the model layer —
`AD-claude-pane-content-id-registry-001`) was decided with adoption-side gpui reading at #388/#394;
no reference app is being matched (ownership plumbing renders nothing). Zed's mapped
one-instance-many-views behavior (07-workspace-panes-palette.md — items as shared entities behind
views) is the RESEARCH background already recorded in pane-composition-model.md; behavior-level
only, no copyleft source consulted (§20 wall intact).

### Prior art
1. **The in-house seams ARE the prior art (the highest-yield leg, all shipped):** the #394
   `ContentRegistry<C>` lifecycle (insert-acquires-1-view; `release_view` returns the owned content
   exactly on last drop; `#[must_use]` on both handles — its unit matrix at cov/MSI 100 is the
   contract this ticket wires); the #348 bounded off-thread PTY reap (pty_os.rs/session.rs — the
   caller-side contract that must survive verbatim); the #163/#205/#177 shape-based persistence
   (restore constructs-then-registers; ids never serialize); the PaneId side-table pattern
   (`agents`/`remotes`/`notify_ticks`) this migration re-keys.
2. **Behavior maps:** pane-composition-model.md (the decision-complete #388 spike) is the design
   source; zed 07 as recorded research. Nothing further consulted; Warp has no analog (its panes are
   not registry-backed in any mapped sense).
3. **Permissive deps:** gpui `Entity<T>` — the #388 NO re-confirmed at #394 and #397's sweeps
   (render-layer-only future option; the model stays gpui-free for cov/MSI 100). No other dep owns
   an ownership-registry seam. Checked; no external owner.

## React-first (parity)
**N/A — no UI delta: behavior-neutral ownership migration; the shell renders identically before and
after (the full suite + the byte-identity pins are the no-change proof).** Vocabulary tie carried
per the ticket: **ContentId stays aligned with the React POC's `PaneItem` stand-in**
(marley-web/docs/MARLEY-PARITY.md § Shared vocabulary) so the exposure slices (#398 add-to-pane,
#399 arrangements, #400 residents) port cleanly; MARLEY-PARITY.md needs no edit unless P2 renames
the shared vocabulary (it must not).

## Locked-In Decisions
- **D1 — the enum is closed and declared ONCE, here.** `Content` carries all 6 kinds
  (Terminal/Editor/FileTree/Git/Cockpit/Browser — the #388 inspect's full set); ONLY `Terminal`
  gains a live payload this slice (`Box<TerminalPane<TerminalSession>>` or the bare
  `TerminalSession` — D-OPEN-PAYLOAD picks with the code); the other five are DECLARED (the
  #385-comment discipline) with their payloads landing in their own slices (#397 editors, #400
  cockpit, #389-train browser). Unconstructed variants ship gate-green via the crate-public API
  (the #371/#394 `pub use` rule).
- **D2 — the registry instance lives on `RootView` (`content: ContentRegistry<Content>`),
  constructed at boot.** The #394-shipped generic is instantiated, NOT modified — its unit matrix
  stays the lifecycle contract; any new helper needed lands beside it with its own 100/100.
- **D3 — views are `ContentId`-bearing per-kind variants; `kind()`/`rail_section()`/`focus_label`
  stay PURE** (no registry read in gpui-free fns) — the #388 open-Q1 resolved as recommended; the
  exact carrier (per-kind variant `Terminal(ContentId)` vs a `(ContentId, ContentKind)` pair) is
  D-OPEN-VIEW-SHAPE, settled by P2 against the real accessor spine.
- **D4 — the close contract:** every terminal close/replace/error path calls `release_view`,
  consumes the `#[must_use]` return, and on `Some(content)` performs TODAY'S off-thread reap
  (`thread::spawn(drop)`) at the call site — the registry NEVER spawns threads (stays pure; the
  #394 §14 note), the reaper crates are untouched, and the 1-view case is byte-identical in timing
  and thread placement (the #348 bounded contract).
- **D5 — ids never serialize; restore REBUILDS.** The codec is byte-untouched; boot/restore
  construct sessions exactly as today then `insert` (view_count 1); a registry id is session-local
  (the #394 doc line: never reused, never persisted).
- **D6 — the maps re-key 1:1, behavior-neutral.** `agents`/`remotes`/`notify_ticks` key by
  `ContentId` where they track the instance; every existing insert/read/scrub site moves with its
  semantics intact; the #226-documented scrub gaps are PRESERVED (not fixed, not widened) — this
  ticket changes WHO the key names, nothing else.
- **D7 — behavior-neutral, NO driven capture** (the #394 precedent, stated explicitly at validate);
  the React-first N/A + byte-identity pins carry parity.
- **D8 — the strangler discipline:** every intermediate implement step compiles green
  (enum+field first, then the view swap, then accessors, then closes, then maps — exact order in
  P2's manifest); a half-migrated state never ships (the D5-split's whole point).

**D-OPEN (Phase 2 decides, with evidence):**
- **D-OPEN-VIEW-SHAPE** — `PaneContent::Terminal(ContentId)` (kind = the variant, recommended) vs a
  kind-tagged pair; must keep `kind()` pure AND the FileTree/CodeView/Git variants (which do NOT
  migrate this slice) compiling unchanged.
- **D-OPEN-PAYLOAD** — `Content::Terminal(Box<TerminalPane<TerminalSession>>)` (the whole pane
  state moves, accessors stay thin — recommended by the ticket's own phrasing) vs the bare
  `TerminalSession` (splits TerminalPane's non-session state back onto the view — likely wrong for
  1:1 neutrality). P2 reads `TerminalPane`'s fields and picks the byte-identity-preserving one.
- **D-OPEN-S-GENERICITY** — does `PaneContent<S>`/`PaneState<S>`/`PaneGrid<…>`'s session-handle
  genericity dissolve (the #388 open-Q2 simplification) or stay? Dissolving simplifies the pure
  tests but widens the diff; keeping it may need a phantom. P2 sizes BOTH and picks the bounded one
  (the L budget is the migration, not a genericity crusade).
- **D-OPEN-ID-ALLOC-AT-RESTORE** — restore's construct-then-insert ordering per grid leaf (who
  holds the id between construction and the cell's mount) — settled with the restore code in hand.
- **D-OPEN-ENUM-HOME** — `content_registry.rs` (beside the lifecycle) vs a new `content.rs` vs
  workspace.rs; constraint: `Content` holds `TerminalPane<TerminalSession>` (app-side concrete), so
  it CANNOT live in a gpui-free pure-only module if that drags impurity — P2 places it where
  cov/MSI stays clean (the registry generic stays pure regardless).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the app runs, every live terminal session shall be owned exactly ONCE in `RootView.content` (registry len == live terminal count; every pane/tab cell holds a `ContentId` view at view_count 1), and `kind()`/`rail_section()`/`focus_label` shall remain pure fns with unchanged outputs. | registry-integration units over open/split/close sequences; the pure-fn suites (#382 focus_label exhaustive, #390 rail) green unchanged |
| REQ-002 | WHEN any terminal close path runs (every user path + the boot-cleanup paths — the P2-verified inventory), the system shall route it through `release_view`, consume the `#[must_use]` return, and on the last-view drop perform the off-thread `thread::spawn(drop)` reap at the call site — never an inline main-thread drop, never a leaked entry. | per-path units (registry len returns to expected; a drop-tracking double proves off-thread handoff where testable); §18.1 inspect: the leak-path critic walks EVERY acquire/release pairing; #348 reap suites green |
| REQ-003 | WHEN the app persists and restores, the settings bytes shall be byte-identical to today's (`t=`/`c=`/`T=`/`V=` codec untouched; no id ever serialized), and restore shall rebuild one registry entry per restored terminal (constructed exactly as today, then inserted at view_count 1). | #163/#205/#177 round-trip suites byte-identical; new restore unit (entry count + view counts post-restore); §18.1 inspect: no `ContentId` reaches any codec/persist path |
| REQ-004 | WHILE terminals are tracked by the `agents`/`remotes`/`notify_ticks` side-tables, those tables shall key by `ContentId` with every producer/consumer/scrub site moved 1:1 — their observable behavior (badges, agent cards, remote indicators, close-time scrubs AND the #226-documented scrub gaps) unchanged. | the existing agent_view/status_bar/notify suites green unchanged; §18.1 inspect: a site-by-site key-migration walk |
| REQ-005 | WHEN the full test suite runs, it shall pass byte-identical (zero behavior change — the D5-split's contract); any test that constructed `PaneContent`/`TabContent` directly shall be mechanically re-keyed with its ASSERTIONS unchanged. | `cargo nextest run --workspace` full green; the diff of test files shows re-keying only (inspect confirms no assertion weakened) |
| REQ-006 | WHEN the gate runs, the pure seams (the registry + any new pure helpers + the re-keyed pure fns in workspace.rs/tabs.rs) shall hold cov/MSI 100 with the app.rs wiring masked per the shipped pattern, and `scripts/gates.sh --diff` shall print GATE GREEN. | gate receipt; `cargo mutants --list -f` on the actual touched files (the skip-detach re-check) |
| REQ-007 | WHEN validate completes, the notes shall record "driven capture: N/A — behavior-neutral structural migration" AND "React-first: N/A — no UI delta" explicitly (never silently), with the byte-identity pins named as the no-change proof. | the notes entry (the enforce-react-parity contract is satisfied by the spec's N/A section) |

## Floors (constitution)
Pure seams at **cov/MSI 100**: `content_registry.rs` (shipped 100 — stays; extended only if a
helper lands there), the re-keyed pure fns in workspace.rs/tabs.rs (kind/rail/label/grid algebra —
their existing suites carry, re-pointed), any new pure resolution helper. MASKED: the app.rs wiring
(accessor shims, close-path plumbing, boot/restore, the pump) — app.rs is coverage-excluded; the
`Content` enum's placement must not drag a live-PTY payload into a cov-measured pure module
(D-OPEN-ENUM-HOME). Typed inputs; no `unwrap`/`expect` on any close/restore path (registry misses
are `Option`-total — a stale id is a `None`, never a panic). The app.rs edits land in the
close-path/pump/boot shim neighborhoods — the skip-detach trap's home turf: re-run
`cargo mutants --list -f` on the ACTUAL touched files after placement and re-verify neighboring
`#[mutants::skip]` bindings (`BF-claude-skip-detach-pump-fleet-live-001`). Every `release_view`/
`acquire_view` call site consumes the `#[must_use]` return
(`PR-claude-registry-release-returning-a-resource-must-be-must-use-001`).

## Phase Plan
- **P2 Design** — **the fresh inventory IS the first act** (the ticket's own line refs are stale;
  the #352 lesson binds: grep the accessor spine, the close paths, the births, the maps — including
  EQUALITY/comparison sites, not just call sites; fan an Explore over app.rs/workspace.rs/tabs.rs);
  settle the five D-OPENs with the code in hand; the exact `Content` enum + view-shape signatures;
  the strangler-step manifest (each step compile-green); the per-REQ test plan.
- **P3 Implement** — React-first N/A (recorded); the strangler steps in the P2 order; `cargo check`
  green between steps; no test expansion beyond compile-necessity.
- **P3.5 Inspect** — the ticket's own critic list: leak-path (every acquire ↔ exactly one release on
  EVERY close/replace/error path), reaper-timing equivalence (still off-thread, still bounded, at
  every site), double-release safety, restore-rebuild correctness, the 1-view byte-identity; plus
  the #352-class sweep (equality sites) and the skip-detach re-check.
- **P4 Validate** — the per-REQ units + the FULL suite byte-identical; `scripts/gates.sh --diff`
  green (receipt); driven capture N/A stated; React-first N/A stated.
- **P5 Complete** — CHANGELOG; pane-composition-model.md Q5 slice-2 marked SHIPPED; crate-map/editor
  docs touched where the ownership story is described; AAR (lessons: the strangler order, whatever
  the inventory drift teaches); close #396; archive; /commit.
