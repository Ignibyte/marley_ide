---
pipeline_id: b92b8f6d-18fe-49a6-8db1-8ba6819f7c70
ticket: forge#397 (19d1d1cd-9326-4595-b757-e9912029cf65) · local docs/planning/tickets/open/TICKET-397-editors-onto-registry.md
aar_id: 97e858fb-c0b2-42fa-a181-6cae8b50d063
status: Phase 5 — Complete PASS (auto run, 2026-08-04; docs+parity synced, knowledge captured, ticket closed, archived)
title: Editors onto the ContentId registry — one Buffer, many views (the #388 slice-3 / #259 collapse)
type: feature
milestone: M28
references:
  - docs/marley_architecture/pane-composition-model.md
  - docs/planning/tickets/open/TICKET-396-terminals-onto-registry.md
  - crates/marley_app/src/content_registry.rs
  - crates/marley_app/src/editor_surface.rs
  - crates/marley_app/src/code_view.rs
  - crates/marley_app/src/workspace.rs
  - crates/marley_app/src/tabs.rs
  - crates/marley_app/src/grid_layout.rs
  - crates/marley_app/src/extchange.rs
  - crates/marley_app/src/lsp_host.rs
  - crates/marley_app/src/app.rs
  - crates/editor/src/buffer.rs
  - crates/editor/src/undo.rs
  - docs/zed_architecture/subsystems/02-text-buffer-anchors.md
  - docs/zed_architecture/subsystems/03-editor-multibuffer.md
  - docs/zed_architecture/subsystems/07-workspace-panes-palette.md
---

## Title
The #388 train slice-3 — the first VISIBLE registry payoff (HARD DEP #396, which ships the concrete
`Content` enum + the terminal migration this follows). Today "the same file in a tab and a split" is
**two divergent Buffers**: `TabContent::CodeView(EditorSurface)` (tabs.rs:31) and
`PaneContent::CodeView(EditorSurface)` (workspace.rs:329) each own an `OpenFile` whose instance-half
is `buffer` + `saved_version` (dirty) + `nonce` (buffer identity) + the #275 `disk`/`conflict`/
`armed_at`/`conflict_observed` snapshot state (editor_surface.rs:32-75). The four births: `Tab::code`
(tabs.rs:163-164), split-create `split_file_pane` (app.rs:5294-5311 — `EditorSurface::new` at :5307,
seeded from a fresh DISK read, so splitting a dirty tab shows STALE text), split-restore
(app.rs:1954-1978, `EditorSurface::new` :1972), and the `from_files` session-restore
(app.rs:2093-2095). #259 documented the consequence and its net: "saving either copy trips the
other's disk-stat conflict banner" (CHANGELOG #259 entry) — the #275 **self-conflict arm**, where the
editor external-conflicts against its own twin.

The work: `Content::Editor` owns the instance-half ONCE per open file in the shipped #394
`ContentRegistry` (content_registry.rs — insert/`acquire_view`/`release_view` drop-on-last-close,
cov/MSI 100, `PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-001`); editor tabs'
file rows and split cells become **views** holding `(ContentId, ContentKind)` + per-view state only
(caret/selection, v+h scroll, focus, IME composition — the #388 Q2 table,
`AD-claude-one-instance-many-views-001`). Undo is **per-instance** (the `UndoHistory` already lives
inside `Buffer` — buffer.rs:36-41 — so one history falls out structurally; either view's ⌘Z rewinds).
The #275 self-conflict dies by construction (one snapshot per file); external detection — the
`extchange` decision table — is untouched. LSP doc-sync derives from the registry so a file is ONE
`didOpen` no matter how many views show it. Ids never serialize; restore rebuilds entries from
restored paths, resolving a path's second mount to `acquire_view`. **Visible payoff: live shared
edits across tab + split, one dirty ●, one ⌘Z.** All line refs are the 2026-08-04 pre-#396 tree;
Phase 2 re-verifies against the post-#396 tree this actually lands on (D6).

## Scope
### In
- **`Content::Editor` instance move:** the `OpenFile` instance-half (`buffer`, `saved_version`,
  `nonce`, `marked`-invalidation discipline, `disk`/`conflict`/`armed_at`/`conflict_observed`, the
  file path) relocates into the #396-shipped `Content` enum's `Editor` arm, one registry entry per
  open FILE; the view-half (`CodeViewState` lines cache + `lines_version` memo, `scroll`,
  `scroll_px`/`scroll_x` parking, focus, composition span) stays with each view (per-view split =
  D2; the memo keys on `(nonce, version)` per
  `PR-claude-version-only-memo-unsound-across-buffer-identity-change-001`).
- **Views hold `(ContentId, ContentKind)`:** the editor tab's file rows and `PaneContent::CodeView`
  cells reference by id (the #396 tag pattern keeps `kind()`/`rail_section()` pure — D5); the ~75
  `active_editor()`/`active_editor_mut()` through-sites (app.rs:9515-9525/:14050) keep their
  signatures — only the resolution inside them moves onto the registry (the #259 accessor
  precedent).
- **Births become construct-or-resolve:** all four birth sites route through one pure
  `resolve_open(path) → existing-id (acquire_view) | fresh insert` seam, keyed by the #322
  `same_file` canonical-path equality (editor_surface.rs:22-27), scoped per D-OPEN-DEDUPE-SCOPE;
  split-create stops re-reading disk when the file is already open (the live buffer, dirty edits
  included, is the payoff).
- **LSP single registration:** the pump's open-doc collection (app.rs:1734-1754 — today per-TAB
  surfaces only, split panes LSP-dark) derives from the registry's editor entries: one
  `(path, buffer, version)` per `ContentId`, so `reconcile` (lsp_host.rs:177-236) sees each uri once
  — no double-`didOpen`, no divergent-twin `didChange` ping-pong, `didClose` exactly on last-view
  close (D-OPEN-LSP-SINGLE-REG decides the exact source shape; the drain-first invariant
  app.rs:1714-1723 / `BF-lsp-didopen-carries-empty-text-ready-race-001` is preserved untouched).
- **Close paths route through `release_view`:** every editor-content close (`close_editor_file`
  app.rs:7501, split-cell close, close-tab, close-project) releases its view; the LAST release drops
  the entry (a plain drop — no PTY-style off-thread reap needed for a buffer); the surviving view
  keeps buffer + undo + dirty intact (D-OPEN-SECOND-VIEW-ON-CLOSE pins the inventory).
- **Persistence rebuild:** the `V=<paths>` editor-tab codec (grid_layout.rs:410-415) and the
  `c=<path>` grid leaf (grid_layout.rs:84/:179-188) are byte-untouched; ids NEVER serialize (D4);
  restore resolves each mount through `resolve_open` so tab+split of one path rebuilds ONE entry
  with two views; #163/#205/#177 round-trips stay byte-identical
  (`PR-claude-new-persist-setting-needs-round-trip-test-001`,
  `PR-claude-persist-verify-trigger-not-just-codec-001`).
- **React-first demo + driven capture:** the marley-web one-buffer-two-views demo FIRST (per
  `## React-first`), then the Rust port; a headless driven capture of the shared-edit payoff
  (REQ-001/REQ-008).
- **Regression pins:** the #259 split-pane drive suite, #275/#284 `extchange` suites, #356
  `sync_lines_from`/resync tests, #163/#205/#177 codec round-trips, the #394 registry suite, and
  the #396 terminal-migration suites — all green unchanged.

### Out (explicitly deferred)
- **Add-to-pane + the rail `ContentId` cross-link** — the gesture dropping any open content into a
  cell, and the Pane row's id-derived label/cross-listing (slice-5; needs this slice + #396).
- **Nameable arrangements** — the `[[panes]]` settings table + make-a-Pane + dangling-key restore
  (slice-6).
- **Cockpit/browser onto the registry** (slice-8; browser waits on #389).
- **Any CRDT/collab buffer** — architecturally excluded: one process, one owner per instance; Zed's
  CRDT text engine is studied context, not the model (§20 wall).
- **Multi-language LSP (#315)** — doc-sync stays rust-scoped exactly as shipped; this slice changes
  WHO feeds the reconcile set, never what qualifies.
- **An unsaved-changes close prompt** — today closing a dirty editor silently discards; that posture
  is unchanged (recorded, not fixed, here).
- **Full editor-feature parity in split panes** (find bar, overlays, folding — the #259 follow-up)
  beyond what one shared instance yields for free.
- **marley_editor API surgery** beyond what D-OPEN-SELECTION-HOME strictly requires (the
  recommendation needs zero).

## Reference (§20)
**Zed — the editor reference, matched at the OBSERVED-BEHAVIOR level.** The behavior this slice
reproduces: *the same file open in two panes is one shared document — edits made in either pane
appear in both, the dirty state is one, and undo is a single history on the document (either pane's
⌘Z rewinds the shared text), while each pane keeps its own cursor and scroll.* The behavior map
records the mechanism-level facts from research (never source):
docs/zed_architecture/subsystems/03-editor-multibuffer.md:121 — a cloned buffer handle "lets two
editors share one buffer (splits, the multibuffer)";
docs/zed_architecture/subsystems/07-workspace-panes-palette.md:157 — `clone_on_split` "split
duplicates the item" (the VIEW is what a split copies, never the document);
docs/zed_architecture/subsystems/02-text-buffer-anchors.md:325-334 — undo `History`/`Transaction`
live on the buffer, so undo is a document-level operation shared by every view. Marley matches the
behavior with its OWN mechanism — the `ContentId` registry, not gpui `Entity<T>`
(`AD-claude-pane-content-id-registry-001`). Clean-room §20 intact: no Zed (GPL) / Warp (AGPL) source
read or translated.

### Prior art
1. **Behavior maps — checked.** The three Zed subsystem docs above (shared-buffer splits, view-clone
   on split, buffer-level undo) are the direct owners of this slice's observed behavior;
   docs/zed_architecture/crates/multi_buffer.md maps the excerpt machinery (out of scope — no
   multibuffer here). docs/warp_architecture/ — checked, **no owner**: Warp is a terminal; its
   subsystem maps carry no multi-view editor-buffer analog. Research, not source.
2. **Published material.** The LSP specification's text-synchronization model is one document per
   uri: `textDocument/didOpen` "must not be sent more than once without a corresponding close" —
   the published-protocol law behind REQ-005 and D-OPEN-LSP-SINGLE-REG (and the reason two divergent
   twin buffers can never both feed one reconcile: `lsp_host::reconcile`'s version arm
   (lsp_host.rs:207-211) would ping-pong full-text didChanges between them forever).
3. **Our permissive deps — checked, and the key question answered.** **ropey 1.6.1**
   (~/.cargo/registry/src/…/ropey-1.6.1/src/rope.rs:81-83): `#[derive(Clone)] pub struct Rope {
   root: Arc<Node> }` — a persistent structure whose clones share nodes. **Does clone-sharing change
   the one-Buffer design? NO:** a clone is a cheap SNAPSHOT with value semantics — it diverges on
   first edit, which is precisely the #259 defect, not a substitute for shared mutation. One mutable
   owner per file stands; ropey's cheap clones stay useful exactly where they are (snapshot/text
   materialization). **gpui (Apache-2.0) `Entity<T>`:** the #388 AD's NO **re-confirmed** — RootView
   is still the single monolithic entity (`cx.new` used once); #394 shipped the app-side registry
   precisely to keep tabs.rs/workspace.rs gpui-free at cov/MSI 100; `Entity` remains a
   render-layer-only future option. **In-house shipped seams (the highest-yield leg):** the #394
   `ContentRegistry` lifecycle IS the owner of share/close semantics — this ticket adopts it rather
   than inventing an editor-specific one; the #273/#336 nonce-keyed park idiom
   (editor_surface.rs:300-327) owns "per-view state over a shared substrate" and generalizes to the
   selection/scroll split (D-OPEN-SELECTION-HOME's recommendation).

## React-first (parity)
**APPLICABLE — the ticket's own call (policy 2026-08-04).** Zone A surfaces (port-map rows: "Editor
surface, tabs" ↔ `components/EditorView.tsx` ← `editor_surface.rs`/`code_view.rs`/`tabs.rs`;
"Splits" ↔ `components/views/SplitTerminalView.tsx` ← `grid_layout.rs`/`content_registry.rs` —
`marley-web/docs/MARLEY-PARITY.md`). The chrome is frozen; what this slice demos is the
INTERACTION: **build & visually verify the one-buffer-two-views demo in marley-web first
(`pnpm --filter @workspace/marley-ide run dev` → localhost:5173), then port 1:1** — the same
`PaneItem` (`App.tsx:20-23`, the ContentId stand-in per § Shared vocabulary) shown in the editor
view AND a split cell, typing in either propagates live to both, ONE dirty ● on both, a shared ⌘Z at
the behavior level. POC discipline: the demo document is scaffolding state (one shared per-path doc
entry in `AppState` — today's `openFiles`/`activeFile` spine, `App.tsx:44-46`); the POC never ports
its data. Validate captures the React↔Marley parity pair (the Marley half is REQ-001's driven
capture). ContentId↔PaneItem vocabulary stays aligned so slices 5/6 port cleanly.

## Locked-In Decisions
- **D1 — one instance per open FILE, in Marley's own registry.** `Content::Editor` owns the
  `OpenFile` instance-half (buffer, undo, `saved_version`, nonce, #275 snapshot state, path) keyed
  by `ContentId`; tab file rows and split cells are views. gpui `Entity<T>` stays out of the model
  layer (`AD-claude-pane-content-id-registry-001`, re-confirmed by the sweep); the one-instance-
  many-views split is `AD-claude-one-instance-many-views-001`, superseding #259's two-Buffers
  stance. HARD DEP #396: the enum, the tag, the accessor template, and the `release_view` routing
  ship there first — this slice extends, never forks, that pattern.
- **D2 — the Q2 state split is law.** Per-instance: buffer/undo/dirty/`saved_version`/nonce/LSP doc
  identity/#275 disk+conflict+arm state. Per-view: caret/selection, v+h scroll, focus flag, IME
  composition, the `CodeViewState` lines cache + `lines_version` memo (keyed `(nonce, version)` —
  the #268/#356 rule; a reload's nonce re-mint invalidates EVERY view's memo, not just one).
- **D3 — undo is per-instance.** One `UndoHistory` (already inside `Buffer`, buffer.rs:36-41);
  either view's ⌘Z/⇧⌘Z rewinds/replays the one shared history in edit order regardless of the
  originating view; the invoking view's cursors land per the recorded selection sets (undo.rs
  `sel_before`/`sel_after`). Per-view undo is a documented NON-GOAL (#388 open-Q3, resolved as
  recommended; the observed Zed behavior).
- **D4 — ids never serialize.** The `V=`/`c=`/`T=`/`C=` codecs and the #163 framing guard are
  byte-untouched; restore REBUILDS registry entries from restored paths (resolve-or-insert), and
  the #163/#205/#177 round-trips stay byte-identical. The #396 discipline verbatim
  (content_registry.rs:20-22 already pins "never serialized").
- **D5 — views carry `(ContentId, ContentKind)`.** The #388 open-Q1 recommendation, shipped by #396:
  the kind tag rides the view so `kind()`/`rail_section()`/`focus_label` stay pure (no registry
  read in gpui-free fns); editors reuse the exact tag pattern.
- **D6 — #275 external detection survives per-instance; the self-conflict arm dies structurally.**
  One disk snapshot + one conflict/arm state per file; save from either view updates the ONE
  snapshot (no twin to stale-out); the `extchange` decision table (extchange.rs:47-67), the #284
  acknowledgment logic, and every choke trigger (save / activate / focus-edge app.rs:14975) are
  UNCHANGED — only where the state lives moves. Line refs in this spec are pre-#396; Phase 2
  re-verifies every cited site against the post-#396 tree before design freeze, and /work must not
  promote #397 while #396 is unshipped.

**D-OPEN (Phase 2 decides, with evidence):**
- **D-OPEN-SELECTION-HOME** — the load-bearing wrinkle: since #297 the `Buffer` OWNS the
  `SelectionSet` (one owner — editor_surface.rs:34-38), but D2 makes selection per-VIEW.
  Candidates: (a) the buffer's set IS the focused view's cursors; each view PARKS its set on
  focus-out and restores on focus-in — the shipped #273/#336 park idiom generalized;
  (b) move `SelectionSet` out of `Buffer` into the view (marley_editor surgery; undo's
  `sel_before`/`sel_after` recording must be re-threaded). **Recommendation: (a)** — zero
  marley_editor change, undo's recorded sets restore into whichever view invokes it (the D3
  behavior), IME `marked` rides the same focused-view discipline (any cross-view edit clears it —
  the #267 rule extended). Phase 2 must ALSO re-key the #273/#336 scroll parking from `nonce` (now
  shared by twins) to a per-view identity, and decide the unfocused twin's caret render (candidate:
  none — today's read-only arm).
- **D-OPEN-LSP-SINGLE-REG** — the doc-sync set must contain each open file ONCE (the LSP one-doc-
  per-uri law; a twin-fed reconcile would ping-pong didChanges). Candidate: derive
  `open_docs` from the registry's editor entries — dedupe becomes structural, and split-ONLY files
  join the sync set (today they are LSP-dark, app.rs:1734-1740 collects per-tab only — a small
  deliberate behavior GAIN to confirm, closing part of the #259 gap). Phase 2 pins the source shape,
  confirms `did_save`/diagnostics keying, and preserves the drain-first ordering invariant
  (app.rs:1714-1723, `BF-lsp-didopen-carries-empty-text-ready-race-001`) untouched.
- **D-OPEN-SAVE-DIRTY-CONVERGENCE** — one `saved_version`/dirty per instance: both views' ●
  reflect it; ⌘S from either view writes once and updates the one snapshot; the #284 arm/observed
  state is per-instance (candidate: yes — the license is about the FILE, not the view, so a warning
  armed in one view is armed in both). Phase 2 pins the exact accessor moves
  (`active_mark_saved`/`active_disk`/`arm_active_save` resolve through the registry) and what the
  unfocused twin shows during a save-under-conflict.
- **D-OPEN-SECOND-VIEW-ON-CLOSE** — registry semantics give the frame: closing one view
  `release_view`s it; the surviving view keeps the live instance (buffer/undo/dirty intact); the
  LAST close drops the entry (plain drop — no reaper analog). The genuinely open half: the
  INVENTORY — every editor close path (`close_editor_file` app.rs:7501, split-cell close, close-tab,
  close-project, and any #396-reshaped path) must route exactly once, including the
  `EditorSurface::close` last-file → drop-the-tab arm (tabs drained file-by-file release one view
  EACH); and the last-close-of-DIRTY posture (candidate: silent discard — today's behavior,
  unchanged; the prompt is Out).
- **D-OPEN-DEDUPE-SCOPE** — what makes two opens "the same file": candidate = same
  workspace/project root + `same_file` canonical equality (the #322 symlink lesson,
  editor_surface.rs:22-27). Cross-PROJECT opens of one path stay TWO instances in this slice
  (surfaces and LSP hosts are root-scoped; the #275 external machinery still guards that edge
  exactly as today — two workspaces editing one file conflict via disk, as now). Phase 2 confirms
  and places the pure `resolve_open` seam.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the same file is open in an editor tab and a split cell (split-from-tab or open-from-split, either order), an edit typed in EITHER view shall appear in both views' rendered text (one `Buffer` — no divergence), and WHEN a DIRTY tab's file is split, the new cell shall show the unsaved edits (not a stale disk read). | headless driven capture: open → edit → split → type in each → assert both texts identical + the dirty edit visible in the fresh split (the #259 drive idiom extended); pure unit: `resolve_open` on an open path returns the existing id via `acquire_view` (registry len 1, view_count 2), never a second insert |
| REQ-002 | WHEN ⌘Z is pressed in either view after interleaved edits from both, the shared buffer shall rewind the ONE history in reverse edit order regardless of originating view, and ⇧⌘Z shall replay it; the invoking view's cursors land per the recorded selection sets. | unit on the shared instance: edit-via-A, edit-via-B, undo×2 → both rewound in order, redo×2 restores; drive: type in tab, type in split, ⌘Z in the tab rewinds the split's edit first |
| REQ-003 | WHILE a shared file's one instance is dirty, BOTH views shall present the dirty ●, and WHEN either view saves (⌘S), the flag shall clear everywhere — one `saved_version`, one disk snapshot. | unit: dirty read through both view resolutions flips together across edit→save; drive: edit in split → tab-strip ● present → ⌘S from the tab → no ● anywhere |
| REQ-004 | WHEN either view of a shared file saves, the OTHER view shall NOT flag a #275 conflict (the self-conflict twin arm is structurally gone), WHILE a genuinely EXTERNAL write (another process) shall still flag Changed/Deleted/CleanReload per the unchanged `extchange` table. | regression drive of the #259-documented scenario (save in one copy → banner in the other) → NO banner now; `extchange` decision-table suite green byte-unchanged; unit: exactly one snapshot per instance, updated by either view's save |
| REQ-005 | WHEN the same file is visible in N views, the workspace's LSP host shall hold exactly ONE open doc for its uri — one `didOpen`, `didChange` only on real version advances (no twin ping-pong), and `didClose` exactly when the LAST view closes. | unit on the registry-derived open-doc set (one `(path, buffer, version)` per id); the #321 `open_doc_count` hook (lsp_host.rs:499): tab+split → 1; close one view → still 1; close last → 0; §18.1 inspect: the reconcile input can never carry one path twice |
| REQ-006 | WHEN the app persists and restores with a file open in a tab AND a split, the settings blob shall be byte-identical to today's (`V=`/`c=` codecs untouched, no id ever serialized), and restore shall rebuild ONE registry entry holding two views. | #163/#205/#177 round-trip suites green byte-identical; new restore unit: both mounts resolve to one entry (len 1, view_count 2); §18.1 inspect: no `ContentId` reaches any codec/persist path (the #396 grep discipline) |
| REQ-007 | WHEN one of two views of a file closes, the surviving view shall keep the live instance (buffer, undo history, dirty flag intact), and WHEN the last view closes, the entry shall be released exactly once — no leak, no double-release, on every editor close path. | registry-integration units over the close inventory (close-file-tab / close-split-cell / close-tab / close-project); the #394 `release_view` suite carries the lifecycle floor; inspect: the leak-path critic walks every acquire/release pairing |
| REQ-008 | WHEN the marley-web demo runs, two editor views of the same `PaneItem` shall show live shared edits + one dirty ● at localhost:5173, visually confirmed BEFORE the Rust port begins; Validate shall capture the React↔Marley parity pair. | browser confirmation of the POC demo (the React half); REQ-001's driven capture (the Marley half); the pair recorded per the parity contract |

## Floors (constitution)
Pure seams at **cov/MSI 100**: `content_registry.rs` (shipped at 100 — stays there, extended only
if `Content::Editor` helpers land in-module); the instance/view split's pure logic (the reshaped
`editor_surface`/`code_view` decision fns, the `resolve_open` path→id seam, the per-view
park/restore + selection-home fns per D-OPEN-SELECTION-HOME); `extchange.rs` unchanged at 100.
Typed inputs; no `unwrap` on restore/config-derived paths. MASKED (app.rs is coverage-excluded):
the pump LSP-collect shim (:1734-1754), the `active_editor`/`_mut` resolution shims
(:9515/:14050), `split_file_pane` (:5294), the restore arms (:1954-1978/:2093-2095), the save shim
(:8282-8346), the close shims (`close_editor_file` :7501 + the tab/project/pane paths), and the
render call sites. These app.rs edits land squarely in the masked-shim neighborhoods — the
skip-detach trap's home turf (5th strike was `pump_fleet_live` itself,
`BF-claude-skip-detach-pump-fleet-live-001`): **re-run `cargo mutants --list -f` on the ACTUAL
touched files after placement, and re-verify neighboring `#[mutants::skip]` bindings.** The #396
migration floors carry: leak-path pairing on every acquire/release, double-release safety,
restore-rebuild correctness, and 1-view byte-identity for a file open in exactly one place.

## Phase Plan
- **P2 Design** — settle the five D-OPENs with evidence (SELECTION-HOME: read the undo
  `sel_before/after` recording + the park idiom's fit; LSP-SINGLE-REG: the registry-derived source
  + the split-only-files gain, drain-first preserved; SAVE-DIRTY-CONVERGENCE: the accessor moves;
  SECOND-VIEW-ON-CLOSE: the full close-path inventory on the POST-#396 tree; DEDUPE-SCOPE: confirm
  per-root + `same_file`); exact signatures (`Content::Editor` shape, the per-view state struct,
  `resolve_open`, park/restore re-key); re-verify every pre-#396 line ref; **file manifest BOTH
  halves — marley-web files FIRST (`EditorView.tsx`, `views/SplitTerminalView.tsx`, `App.tsx`
  `PaneItem`/`AppState`) per React-first, then the Rust manifest**; per-REQ test plan.
- **P3 Implement** — **React-first: build the one-buffer-two-views demo in marley-web and confirm
  the interaction at localhost:5173 (per `## React-first (parity)`) BEFORE any Rust**; then the
  pure seams (instance/view split, `resolve_open`, park/restore, selection home), then the masked
  app.rs migration (births → construct-or-resolve, accessor resolution, LSP collect, close routing,
  restore rebuild), each intermediate state compiling green (the strangler discipline).
- **P3.5 Inspect** — adversarial lenses: **leak-path** (every acquire paired with a release on
  every close/replace/error path — the #396 critic list carried); **double-didOpen / didChange
  ping-pong** (can the reconcile input ever carry one uri twice, or two divergent versions?);
  **undo divergence** (any path still able to mint a second history for an open file — reload
  epochs included); **restore rebuild** (two mounts → one entry; vanished path → dropped, never a
  phantom); **per-view state bleed** (focus switch parks/restores the RIGHT view's
  selection/scroll; nonce-shared twins can't collide); skip-detach re-check; provenance (§20).
- **P4 Validate** — write + RUN the units per REQ; the headless driven capture (shared typing, one
  undo, one ●, no self-conflict banner); #163/#205/#177 round-trips byte-identical; the React↔Marley
  parity pair; `cargo mutants --list -f` on the actual touched files; gate green (`--diff`),
  cov/MSI 100 on the pure seams; #259/#275/#284/#356/#394/#396 regression suites green unchanged.
- **P5 Complete** — CHANGELOG; pane-composition-model.md slice-3 marked SHIPPED (+ the open-Q3
  resolution recorded); MARLEY-PARITY.md vocabulary note if the demo changed the POC state shape;
  AAR capture; archive; close #397.
