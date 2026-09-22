# Nameable arrangements — the [[panes]] settings table — Notes

- **Forge ticket:** #399 d7b3b391-534f-4f39-93c0-033c408f55ab (feature, sprint #39 "M28 — The Registry Payoff")
- **AAR:** pending-promotion
- **Local ticket doc:** docs/planning/tickets/open/TICKET-399-nameable-arrangements.md
- **Pipeline spec:** 399-nameable-arrangements.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request / provenance:** the #388 spike's train slice-6, verbatim from pane-composition-model.md
  Q4 ("a saved Pane = a named grid layout + the ContentId graph its cells reference … persist via a
  new settings table (the #204 `[[workflows]]` round-trip idiom), NOT the shell/grid codec") + Q5
  ("6. Nameable arrangements — the `[[panes]]` settings table + 'make a Pane' + restore with
  dangling-id drop. (M)"). Queued in the sprint #39 /spec batch (2026-08-04). **ORDERING:
  hard-after #398 (slice-5)** — the naming verb and the named rows sit on slice-5's add-to-pane +
  `ContentId`-derived Pane-row labels; #396/#397 transitively (slice-5's own deps). tabs.rs:845's
  own comment marks the seam ("no stored identity; #394 adds a ContentId to these rows later").
- **Classification / tier:** feature, ONE slice, M. Composition of shipped idioms — the #204
  settings-table + inline-draft templates, the #177 sanitize/rename discipline, the #390 rail
  branch, the #394/#396 registry — plus one genuinely new pure seam (the arrangement codec +
  dangling-drop resolution). React-first APPLICABLE (Zone A rail + a new naming card).
- **Forge recall (§18.3):** knowledge-search run 2026-08-04 ("settings codec round-trip back-compat
  tier"; "inline draft naming rename tab") — the applicable named rules, bound into the spec:
  `PR-claude-persist-verify-trigger-not-just-codec-001` (REQ-001 exercises the persist helper —
  the settings.rs:934 "kills its Ok(()) mutant" house pattern);
  `PR-claude-new-overlay-register-at-every-choke-point-001` (REQ-007's roster walk);
  `PR-claude-second-consumer-must-inherit-the-first-consumers-guards-001` (D3 — the draft's second
  consumer inherits #204's trim/empty/flash guards; the settings table inherits the tolerance
  arms); `PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-001` (restore acquires
  views through the registry, never side-builds);
  `PR-claude-boot-decisions-key-the-restored-active-root-001` (D-OPEN-SCOPE keys the restored
  root); `PR-claude-trace-the-real-cargo-mutants-list` (Floors). Standing ADs:
  `AD-claude-pane-content-id-registry-001`, `AD-claude-one-instance-many-views-001` (ids are
  session-local — the D1 spine). docs-search leaned on pane-composition-model.md Q4 directly.
- **Discovery (the sweep's load-bearing evidence — file:line, all read 2026-08-04):**
  - **The rail seam:** tabs.rs:843-877 — the derived Panes branch; :857 `label: format!("PANE
    {pane_n}")` (the string this ticket replaces for named rows); :846-853 the 1-based
    per-project counter over split tabs (renumbers on close — no identity). Render:
    app.rs:16515-16559 — the Arrangement row, explicitly "Read-only: no rename/close" (#390),
    hidden under an active session_filter (:16522) — the named label inherits that filter arm
    (P2: does a NAME now match the filter? candidate yes — names are user-given titles, unlike
    the generic "PANE n" the #390 inspect excluded).
  - **The naming idiom (the #204 template, whole-cloth):** app.rs:171 `naming_workflow:
    Option<(String, String)>`; :3261-3284 `begin_naming_workflow` (blank target → `status_flash`,
    not a dead card); :15044-15104 the key-ladder branch (Enter commit trims + refuses empty, Esc,
    backspace/space/printables, `stop_propagation` :15101); :18213-18239 the centered framed card
    (`{name}\u{258f}` caret + muted context line); :9479-9499 `text_input_blocked` (:9488; the
    :9477 comment — "a NEW overlay only needs a line here"); :7078-7080 the #378 F3/F4
    two-text-owners exclusion; :6734-6739 `close_transient_overlays` clears the draft; :7424/:7522
    the close-tab/index-shift clearing discipline (renaming_tab's rule — ours follows); :16485-
    16496 the #177 double-click rename seed (`live_tab_title` seed idiom — the candidate second
    entry point).
  - **The settings-table template:** settings.rs:75-79 (`Workflows: Vec<Workflow>` = the
    `[[workflows]]` array-of-tables); :914-965 the full tolerance test (round-trip identity
    THROUGH `persist_workflows`; absent → empty; malformed scalar → empty, no panic; missing
    optional key → serde default, "NOT a wipe-all" — the F3 lesson at workflows.rs:17-21);
    `AppliedSettings` field + `applied_from` mapping. The framework: marley_settings manager.rs
    (typed registry over a retained toml::Table; `set`/`get`/`reload`; R5 absent-file → empty).
  - **The codec discipline this rides beside (NOT inside):** grid_layout.rs — `serialize_shell`
    :329-372 (`T=`/`C=`/`V=` per-tab entries; the #177 `title\x1f blob` rider :344-352 — the
    precedent for a persisted per-tab name AND the candidate re-bind vehicle);
    `breaks_grid_framing` :309-311 (reserved `\t \n \r , : = \x1f`); `sanitize_title` :316-322
    (write-time, cap 60 — the name hygiene D4 adopts); the `t=<cwd>`/`c=<path>` leaf keys :79-87
    (the stable-key vocabulary D-OPEN-REF-SHAPE mirrors). Restore guards: app.rs:2201
    `path.is_file()` (#205 — dead V= path drops), :1344 `is_dir` cwd guard (dead cwd → root
    fallback TODAY; D2 flags the arrangement-cell divergence for a P2 pin). Persist trigger sites:
    :4882-4943 (`persist_grid` → `serialize_shell` → `persist_shell`).
  - **The registry (resolution target):** content_registry.rs:23 `ContentId(u64)` (session-local
    — why D1 forbids serializing it), :56 `insert`, :66 `acquire_view`, :78 `release_view`
    (last-view drop returns ownership). #396 (cd706b38…) declares `Content` + migrates terminal
    ownership; #397 editors; #398 (7aa54235…) the add-to-pane gesture + `ContentId`-derived
    Pane-row labels this slice's named rows replace/decorate.
  - **Behavior maps (sweep leg a):** warp.md:48 — `launch_configs` a public-module NAME only;
    nothing behavioral mapped (honest thinness, stated in Reference). Zed
    07-workspace-panes-palette.md:191-193 — `SerializableItemRegistry` rebuild-at-boot (session
    restore, not naming). Published: Warp Launch Configurations (named files, save-from-session,
    reopen-by-name, fresh spawn) + tmux named sessions — behavior level only.
  - **The POC half:** marley-web artifacts/marley-ide/src/components/LeftRail.tsx:253-307 — the
    Panes section exists (`PANE ${pane.id}` :281, post-#390/#395 comment :276-280); NO
    arrangement-naming flow in the POC today (the F2 `renameDraft` is editor-surface) → the POC
    gains the flow here as the prototype. Parity: MARLEY-PARITY.md port map (LeftRail.tsx ↔
    app.rs rail/tabs.rs; SplitTerminalView.tsx ↔ grid_layout.rs/content_registry.rs) +
    § Shared vocabulary (ContentId ↔ PaneItem) + the POC-never-ports-data rule.
- **The central design fork (P2, named in the spec):** the restore AUTHORITY — the #163 shell blob
  already rebuilds every live tab (split grids included); a named arrangement's live tab must come
  back exactly once, named. Candidates: (a) shell rebuild + name RE-BIND (a rider on the T= entry
  — the #177 `title\x1f` slot precedents a persisted per-tab string; `[[panes]]` stays the durable
  object that outlives the tab) vs (b) `[[panes]]`-driven rebuild with shell dedup. The spec locks
  only the observables (REQ-003: back once, named, no duplicate); P2 picks with the codec in hand.
- **EARS drafted (8):** REQ-001 name→persisted entry (trigger-exercised round-trip); REQ-002 named
  rail row; REQ-003 restore rebuilds, once, named; REQ-004 dangling-drop / all-drop-skip / never
  fail; REQ-005 ids never serialize; REQ-006 unnamed "PANE n" byte-identical + cancel paths inert;
  REQ-007 draft at every choke point; REQ-008 suites green + parity pair.
- **Decisions:** locked D1 (shapes/coords/names serialize, ContentIds never — refs resolve at
  restore), D2 (dangling refs DROP silently, all-dropped skips the arrangement, restore never
  fails/invents), D3 (the #204/#177 inline-draft idiom, no new modal machinery, guards inherited),
  D4 ([[panes]] on the existing settings file + the #163/#205-tier codec discipline + #177
  write-time name hygiene).
- **Open design questions (Phase 2):** D-OPEN-REF-SHAPE (per-kind stable key: cwd/path + ordinal —
  recommend mirroring the #205/#258 leaf-key vocabulary), D-OPEN-SCOPE (recommend per-project
  first, keyed by restored root), D-OPEN-APPLY-SEMANTICS (recommend new tab; owns the
  restore-authority sub-fork), D-OPEN-UNNAMED-DEFAULT (recommend yes — "PANE n" byte-identical).
- **Forge ids:** ticket #399 `d7b3b391-534f-4f39-93c0-033c408f55ab`; sprint #39
  `6558258f-a0d2-45fb-b601-cd5329a1cd77` "M28 — The Registry Payoff"; pipeline
  `29dc884e-ea8f-4b3a-992a-8159469ba47c`; AAR pending-promotion (mint at /work). Depends-on:
  #398 `7aa54235-760c-4191-a5e6-12a5b65dfa7e` HARD (#396 `cd706b38-…`/#397 `19d1d1cd-…`
  transitive) — verify shipped before promotion.

## Phase 2 — Design

### Architecture — the D-OPEN settlements (all four, with evidence)

- **The restore-authority fork → candidate (a): shell rebuild + name RE-BIND via a second `T=` rider.**
  The #163 shell blob stays the ONE rebuild authority; `[[panes]]` never drives boot. Evidence that
  makes this cheap: #177 titles live ON the `Tab` struct (`custom_title`, tabs.rs:139) — not an
  index-keyed map — captured at `build_shell_layout` (app.rs:5143) and re-bound at the boot restore
  arm (app.rs:2151). The arrangement name becomes the sibling field `Tab::pane_name`, rides the
  `T=` entry as a second `\x1f` slot, and re-binds in the same two places. REQ-003's observables
  (back once, named, no duplicate) hold STRUCTURALLY — single authority, no dedup, no zombie
  resurrection of deliberately-closed named tabs.
  **Wire:** `T=blob` (bare) | `T=title\x1fblob` (#177, byte-identical when unnamed — REQ-006/008 pin)
  | `T=title\x1fname\x1fblob` (named; the title slot may be EMPTY — old serialize never emits an
  empty title slot (grid_layout.rs:348 skips), so part-count disambiguates: 1=blob, 2=title+blob,
  3=title+name+blob). Old files parse unchanged; downgrade degrades to default-grid exactly as the
  #177 rider did (accepted precedent). `sanitize_title` guards the name at BOTH write boundaries
  (verb commit + `serialize_shell`) — writer guards, reader trusts (#163 D2).
- **D-OPEN-REF-SHAPE → typed TOML fields, word-kind vocabulary, per-cell ordinal.** Hand-editability
  wins over the leaf-token alphabet (TOML strings need no framing rules). Schema:
  `PaneArrangement { name: String (required — a missing name is the house malformed→empty arm, like
  a `[[workflows]]` entry missing `name`), scope: String #[serde(default)], axis: String
  #[serde(default)] ("H"/"V", unknown→H — the restore_grid tolerant posture), cells: Vec<PaneCell>
  #[serde(default)] }`; `PaneCell { kind: String ("terminal"/"editor"/"files"/"git" — kind is a
  STRING, classified at RESOLVE, so a future kind ("browser", #400) loads today, drops its cell
  (D2), and round-trips byte-preserved — no data loss, the kind-agnostic seam promised in Out),
  key: Option<String> #[serde(default)] (terminal cwd | editor path; terminal None → scope root,
  the bare-`t` precedent; editor None → drop), ordinal: u32 #[serde(default)] }.
  **The ordinal encodes instance identity without serializing ids:** at snapshot, cells grouped by
  (kind, key) get ordinal = the first-seen index of their `ContentId` within the group — #398 twins
  (same cid) share an ordinal; coincidental same-cwd terminals get 0,1,… At apply, equal
  (kind, key, ordinal) cells REUNITE as twins (first spawns + `insert`, rest `acquire_view`) —
  the registry-acquire stance (`PR-…-shared-registry-needs-refcount-…`). Editors: `resolve_open`
  already unifies by path, so editor ordinals are structurally 0 — kept uniform anyway.
- **D-OPEN-SCOPE → per-project.** `scope` = the project root string (keyed-by-root, #234/#245);
  the palette lists only the ACTIVE project's entries; apply opens into the active project.
- **D-OPEN-APPLY-SEMANTICS → new tab; the reopen verb IS in scope.** "Pane: {name}" palette rows —
  the Reference behavior (Warp reopen-by-name, fresh spawns) and the consumer that makes
  `[[panes]]` live rather than write-only. `PANE_ARRANGEMENT_BASE = 5000` (the new TOP id block),
  rebuilt at every palette open (the #398 D5 rebuild-not-append stance) with a BY-VALUE snapshot
  `pane_targets: Vec<PaneArrangement>` (a settings reload between open and dispatch must not
  misdispatch). **Collateral hardening:** `rebuild_add_to_pane_commands`' retain (app.rs:7214,
  `< ADD_TO_PANE_BASE`) assumes it owns the top block — it would strip the 5000 range; it becomes
  band-scoped ([4000, 5000) stripped) and its append gets the 1000-row cap it lost the right to
  skip (the #398 inspect's own "top block or bounded" rule). Boot restore does NOT apply entries —
  all-dropped skip (pure-fn `None`) surfaces as a `status_flash` on the VERB (a verb needs
  feedback — the #204 blank-guard stance); boot keeps shipped shell semantics untouched
  (dead cwd → root-fallback :1344 stays SHELL-side; the arrangement RESOLUTION drops instead —
  the D2 pin, divergence deliberate and unit-pinned).
- **D-OPEN-UNNAMED-DEFAULT → yes, byte-identical.** `rail_rows` needs NO signature change — the
  Panes branch reads `tab.pane_name` off the workspace it already walks (tabs.rs:1016-1034):
  named → the name, unnamed → `format!("PANE {pane_n}")` untouched; #390 suites green unchanged.
  Named rows additionally PARTICIPATE in the "Search tabs" filter by name (the render arm's
  blanket hide at app.rs:17504 gains a named-match arm — names are user-given titles, unlike the
  generic labels the #390 inspect excluded); unnamed rows keep the hide.

### The naming verb (D3 — the #204/#177 idiom, second consumer inherits the guards)
- Draft `naming_pane: Option<(usize, usize, String)>` (project, tab, draft) — the `renaming_tab`
  shape (per-tab + index-shift discipline) with the `naming_workflow` CARD render (app.rs:19243-
  19266 mirrored: caption + `{name}▏` caret + muted context line; POC settles the exact copy).
- Entry points: palette verb **"Name Pane…"** (static id next to `SAVE_WORKFLOW_ID`; guard: active
  tab must own a MULTI-CELL grid else `status_flash` — the #204 blank-guard) and **double-click the
  Arrangement row** (the #177 rename idiom; seeds the draft with the current name; the #390
  "read-only" comment updates). `begin_naming_pane` calls `close_transient_overlays` FIRST (no
  two-text-drafts overlap by construction), then sets the draft.
- Choke points (REQ-007 roster): key-ladder branch beside naming_workflow's (:15914 shape — Esc /
  Enter / backspace / space / printables + `stop_propagation` + notify); `text_input_blocked`
  (:10112 region); the #378 two-owners exclusion (:7642 region); `close_transient_overlays`
  (:7079); close-tab / close-project index-shift clearing (the renaming_tab :7992/:8123 lines,
  mirrored adjacently).
- Enter commit: `sanitize_title(draft)`; empty → close inert (the #204 refuse stance, REQ-006).
  Non-empty: bind `tab.pane_name`; snapshot cells (per-cell kind/cwd/path from the
  `build_shell_layout` captures :5111-5140 + per-cell `ContentId` for grouping) → `snapshot_cells`;
  axis from the grid group (the serialize_grid :109-115 match); `upsert` by (scope, name) into
  `self.pane_arrangements`; `persist_panes`; `persist_grid()` (rider lands). Re-naming a named tab
  seeds + saves under the new name; the old entry REMAINS (recipes are durable; no delete verb —
  slice-out, settings hand-edit deletes).
- Apply: `resolve_arrangement(entry, key_ok)` (terminal: `is_dir`, editor: `is_file`) → `None` →
  flash; `Some` → build a NEW terminal tab in the active project via the #398 primitives
  (first cell mounts the tab; each next cell: group-first → spawn/`resolve_open` + `insert`, else
  `acquire_view`; per-kind mount mirrors `add_content_to_split`'s arms / `restore_panes`' f/g
  cells), bind `pane_name`, switch, persist. Applying twice = two tabs, both named (bindings are
  per-tab; ENTRY uniqueness is (scope, name) via upsert).

### File manifest
**marley-web FIRST (the React-first half — approved look before any Rust):**
1. `artifacts/marley-ide/src/utils/arrangements.ts` — NEW: snapshot/resolve/upsert mirrors (mocked
   store; vocabulary parity with the Rust module).
2. `components/LeftRail.tsx` — named Arrangement rows (name over `PANE n`), double-click → naming
   draft; filter participation for named rows.
3. `overlays/` naming card (new component or Workspace-inline per POC idiom) — the #204-card shape.
4. `overlays/CommandPalette.tsx` — "Name Pane…" verb + dynamic "Pane: {name}" rows.
5. `pages/Workspace.tsx` — paneNames state + [[panes]] mock store + dispatch arms + key handling.
6. `docs/MARLEY-PARITY.md` — § Shared vocabulary + port-map rows.

**Rust (ported 1:1 after the POC is confirmed):**
1. `crates/marley_app/src/arrangements.rs` — NEW pure module (cov/MSI 100): serde model
   (`PaneArrangement`/`PaneCell`), `kind_word`/`kind_of`, `snapshot_cells` (ordinal grouping),
   `resolve_arrangement` (+`ResolvedArrangement`/`ResolvedCell`), `upsert`.
2. `crates/marley_app/src/main.rs` — `mod arrangements;`.
3. `crates/marley_app/src/grid_layout.rs` — `TabLayout::Terminal` gains `pane_name`; the 3-part
   rider serialize/parse; tests (old-shape compat + unnamed byte-pin + named round-trip).
4. `crates/marley_app/src/tabs.rs` — `Tab::pane_name` field (ctors default `None`); the Panes
   branch named arm; tests.
5. `crates/marley_app/src/settings.rs` — `define_setting!(PaneArrangements … "panes")`,
   `AppliedSettings.panes` + `applied_from`, `persist_panes` (mirrors `persist_workflows` :486),
   the #204-template tolerance test THROUGH the helper.
6. `crates/marley_app/src/app.rs` — masked shims: field `naming_pane` + `pane_arrangements` +
   `pane_targets`; begin/commit verbs + guards + card render + key ladder + choke points; palette:
   static "Name Pane…" + `PANE_ARRANGEMENT_BASE` rebuild (top block) + dispatch arm + add-to-pane
   band retain + cap; `apply_arrangement`; `build_shell_layout` + boot-restore rider bind; the
   Arrangement row render (named filter arm + double-click).

### Regression Test Plan
| REQ | Test (file) |
|---|---|
| REQ-001 | settings.rs: `[[panes]]` round-trip THROUGH `persist_panes` (temp-dir manager; kills the `Ok(())` mutant); tolerance arms: absent key → empty, malformed scalar → empty no panic, entry missing `ordinal`/`key`/`axis`/`scope` → serde defaults (NOT wipe-all), entry missing `name` → whole-setting default (the workflows-parity arm); `upsert` replaces (scope,name) / pushes new (arrangements.rs unit) |
| REQ-002 | tabs.rs: named binding → row label = name; hostile name sanitized at write (arrangements/grid_layout units); #390 suites unchanged |
| REQ-003 | grid_layout.rs: 3-part rider round-trip; named-untitled (empty title slot); titled+named; 1/2-part legacy parse pins; headless drive: name → persist → rebuild-from-blob → binding present ONCE, rail named |
| REQ-004 | arrangements.rs `resolve_arrangement` permutations: dead-cwd terminal drops (rest intact — the D2 pin vs the :1344 fallback), dead-path editor drops, editor-no-key drops, unknown-kind drops (future-kind), terminal-no-key → scope root, all-dangling → None, empty cells → None, ordinal grouping (same (kind,key,ordinal) → one group; differing ordinal → two; twins reunite) |
| REQ-005 | arrangements.rs: serialized TOML of a snapshot contains name/scope/axis/kind/key/ordinal only — no id field EXISTS to leak (struct-shape proof pinned by asserting the exact serialized key set) |
| REQ-006 | tabs.rs unnamed byte-pin (`PANE {n}`); headless: Esc cancels draft inert; empty/whitespace Enter commits nothing (no entry, no binding, draft closed) |
| REQ-007 | headless: draft open → `text_input_blocked`; `close_transient_overlays` clears; close-tab/-project shifts/clears the (p,t) draft (renaming_tab parity); begin clears other drafts first |
| REQ-008 | full suite + gate `--diff`; React↔Marley parity pair (named rows + open naming card) at Validate |
- trybuild: N/A — no new public type contract (serde tolerance is unit-proven); noted per §7.
- Genuinely uncoverable: live PTY spawn inside `apply_arrangement` (masked shim — headless drives
  cover the resolution + registry seams; the live half is the Validate driven check).

### Risks / decisions
- The `T=` wire change is the highest-blast-radius edit → byte-pins for every legacy shape + the
  unnamed-tab serialize path asserted byte-identical.
- `rebuild_add_to_pane_commands` band retain + cap is a behavior change to #398 code — its #398
  suites must stay green; new unit for the band boundary.
- app.rs edits sit in skip-detach territory → `cargo mutants --list -f` re-run on touched files at
  Validate (`PR-claude-trace-the-real-cargo-mutants-list`).
- Apply-path construction reuses #398 split/mount primitives — exact per-kind calls settled at
  implement against `restore_panes` + `add_content_to_split` (no side-build; registry acquire only).

## Phase 3 — Implement

**React-first (marley-web, built + confirmed at localhost:5173 BEFORE any Rust):**
`utils/arrangements.ts` (NEW — snapshotPane/upsertPane/resolvePane/applyPaneUpdate),
`overlays/NamePaneCard.tsx` (NEW — the #204-card shape), `App.tsx` (SavedPane/SavedPaneCell types +
paneName/paneNaming/savedPanes state; draft never persists open), `Workspace.tsx` (the naming key
branch ahead of every owner + card render), `LeftRail.tsx` (named row + double-click via a new
SubItem onDoubleClick; **back-port fix**: filteredPanes matched `pane n`/cell names under filter —
Marley hides generic rows, named rows participate by name), `CommandPalette.tsx` ("Name Pane…" +
dynamic "Pane: {name}" rows + both dispatches). Typecheck green. Captures (scratchpad
399-captures/): 399-react-1-naming-card (card open: caption/caret/context), -2-named-rail-restored
(named row AFTER reload — persistence), -4-palette-filtered (Close Pane / Name Pane… /
Pane: dev cockpit), -5-applied (reopen-by-name rebuilt + named). The persisted mock entry verified:
name/scope/axis-V/3 cells with the twins sharing (terminal, Marley, ordinal 0) — no id anywhere.
(Playwright screenshots wedge after keyboard interactions this session — worked around by
close→navigate per capture point; state persists in localStorage.)

**Rust (ported 1:1):** all manifest files —
- `arrangements.rs` NEW (model + kind/axis words + snapshot_cells ordinals + resolve_arrangement
  groups/drops + upsert); `lib.rs` mod line.
- `grid_layout.rs`: `TabLayout::Terminal` + `pane_name`; the 3-part `T=` rider (empty-title-slot
  disambiguation; both riders sanitized at write); tolerant `splitn(3)` parse.
- `tabs.rs`: `Tab::pane_name` (ctors None); the Panes branch named arm (unnamed byte-identical);
  `cell_label`/`cell_content_id` → pub(crate) (reuse in the card/snapshot — small manifest add).
- `settings.rs`: `PaneArrangements` define_setting + `AppliedSettings.panes` + `applied_from` +
  `persist_panes` (persist_workflows mirror).
- `app.rs`: field trio (naming_pane, pane_arrangements seeded from applied, pane_targets);
  `PANE_ARRANGEMENT_BASE=5000` + `NAME_PANE_ID` (CommandId(30) after inspect #1 — the drafted 20
  collided with a shipped static); static verb + both dispatch arms;
  palette-open `rebuild_pane_commands` (top block, scope-filtered, name-sorted, take(1000),
  by-value snapshot); **rebuild_add_to_pane_commands band retain [4000,5000) + 1000-cap** (it lost
  top-block status — the #398 inspect's own rule); `begin_naming_pane` (multi-cell guard + flash +
  close_transient first + seed) / `commit_naming_pane` (sanitize→snapshot via the
  build_shell_layout captures + cell_content_id→upsert→persist_panes→bind→persist_grid) /
  `apply_arrangement` (resolve with is_absolute+is_dir / resolve_under_root+is_file predicates —
  D2 drop, no root-fallback; all-dropped → flash; terminal-seed + per-kind mounts mirroring
  restore_panes/add_content_to_split; twins reunite via group→acquire_view; editors via
  resolve_open; fresh tab + name bind + persist); the key ladder branch; text_input_blocked;
  close_transient_overlays; the fleet two-owners refusal; naming_pane cleared at ALL FIVE
  renaming_tab dismissal sites (file-ref menu, terminal right-click menu, close_tab_at,
  close_project_at, close_transient); boot re-bind + build_shell_layout capture; the Arrangement
  row render (named-filter participation + double-click begin via click_count>=2 — the first
  click's switch makes (p,t) active so the begin targets the row).

**Deviations from design (with reason):**
- `shell_codec_titles`' hand-mangled-payload pin updated: `T=a\x1fb\x1fc` (2 separators) IS the
  named wire now — the pre-#399 writer could never emit it (titles sanitize `\x1f`, blob alphabet
  excludes it), so the byte pattern was free to claim; a 3rd separator still falls to the blob →
  default-grid tolerance. Documented in the test.
- Apply seeds pane 0 from the FIRST TERMINAL cell (grids are terminal tabs — pane 0 is a terminal
  by construction; every real snapshot has one; a hand-authored all-editor entry seeds a root
  terminal — the boot posture for odd blobs, content-complete over order-perfect).
- `begin_naming_pane(cx)` (notify) — mirrors `begin_naming_workflow`.
Compile green (`cargo check --workspace --tests`); the touched suites (grid_layout/tabs/settings,
120 tests) green; `cargo fmt` clean.

## Phase 3.5 — Inspect

Four parallel critics (correctness / data-state integrity / security-provenance /
simplification-reuse) over the working-tree diff, each instructed to verify concretely. 22
findings; every fix re-verified (check --tests 0 errors, 120 touched-suite tests green, POC
typecheck green).

| # | Sev | Finding | Verdict → action |
|---|-----|---------|------------------|
| 1 | HIGH | `NAME_PANE_ID = CommandId(20)` collides with "Toggle Find: Regex Mode" (cockpit_commands mints 0-29; `action_for_command` maps 20 → the find toggle, and that dispatch arm runs FIRST) — the palette verb was dead code that silently flipped the regex mode | REAL (both correctness + data critics traced it end-to-end) → `CommandId(30)` + comment records the trap; P4 adds the reverse-guard unit (statics resolve to NO verb) + a no-duplicate-ids pin |
| 2 | MED | Snapshot walked `pane_ids()` (numeric-id order), not the layout tree — "visual order" skews after any mid-grid split/close | REAL → `grid.group().panes()` (DFS) at BOTH the commit snapshot and the card summary |
| 3 | MED | Apply rotated cells around the terminal-first seed (pre-seed cells landed after it; an all-editor entry silently gained a terminal) | REAL → two-direction mount plan: post-seed cells chain `After`, pre-seed cells re-target the seed with `Before` — persisted order survives; the no-terminal extra-seed case stays (documented, content-complete over count-perfect) |
| 4 | MED | No un-name path — an empty commit refused, leaving a bound name unclearable forever (diverging from the #177 idiom it cites) | REAL → empty commit on a NAMED tab clears the binding + persists (`pane_name.take()`); the `[[panes]]` entry stays (recipes outlive bindings); unnamed stays untouched (REQ-006 intact) |
| 5 | MED | One malformed `[[panes]]` entry empties the whole table and the next save persists the loss | REAL but REJECTED as a #399 change: it IS the #204 `Vec<T>` house arm (workflows behave identically, pinned); forking one table's tolerance model splits the template. P4 pins the arm (REQ-001); **follow-up recorded**: per-entry-tolerant decode as a uniform upgrade across all Vec settings |
| 6 | MED | `valid_dir_or` ("the ONE spawn-cwd authority") cited but re-implemented inline; both apply fallbacks hand-rolled key-or-root without the validity re-check | REAL → extracted `valid_spawn_dir(cand)` predicate (valid_dir_or + the resolve `key_ok` share it); both fallbacks now `valid_dir_or(...)` (a cwd dying between resolve and spawn falls back to root instead of losing the cell) |
| 7 | MED | The #275 stat-before-read closure existed 3× (two boot restores + apply) — the race-direction rationale had three homes | REAL → extracted `open_editor_instance` (one home), consumed by all three sites |
| 8 | MED | Size cap ran AFTER `std::fs::read` — a hand-authored 8 GB regular file fully allocates before the 2 MB cap (precedent-matching: the shipped boot blocks share the gap) | REAL → the extraction (#7) gates on the STAT length before the read (the post-read checks stay authoritative for a racing append) — the class fix lands at all three sites at once |
| 9 | MED | The CodeView-cell mount expression existed 5× | REAL → `editor_cell(id, path, tab_width)` constructor; 4 sites collapsed (the 5th reuses a pre-loaded CodeViewState — left) |
| 10 | MED | Palette row showed the RAW hand-edited name while the rail binds the sanitized one; `sanitize_title` allocated per entry per open just to test emptiness | REAL → single-pass filter_map: one sanitize, reused as display title AND sort key |
| 11 | MED | Third copy of the #204 card chrome + fourth copy of the draft key-ladder | REAL but DEFERRED — the extraction touches three SHIPPED overlays; the critic itself scopes it as a follow-up ticket, not a #399 rider. **Follow-up recorded** (`draft_card` + `edit_draft_key`) |
| 12 | LOW | `begin_naming_pane` could open over a live fleet dispatch draft (the doc's "by construction" claim didn't cover it; close_transient deliberately never clears a typed brief) | REAL → symmetric refusal (`fleet_dispatch_draft.is_some() → return`), doc corrected. The pre-existing reverse gap (fleet card starves the #177 rename arm) recorded, not fixed |
| 13 | LOW | The rail Tab-row double-click rename cleared neither naming draft — keys meant for the tab title would land in the arrangement name (pre-existing hole for naming_workflow) | REAL → both naming drafts cleared at the rename begin (fixes the inherited #204 hole too) |
| 14 | LOW | Commit didn't re-check the multi-cell guard — a pane close mid-draft could persist a phantom 1-cell entry with an invisible rail row | REAL → `pane_ids().len() >= 2` re-guard in the read phase |
| 15 | LOW | Hand-edited duplicate (scope, name) pairs rendered as identical adjacent palette rows | REAL → sorted-stable `dedup_by` in the rebuild, FIRST wins (matches `upsert`'s first-match update) |
| 16 | LOW | `axis_of` doc cited `restore_grid` for behavior it doesn't have (restore_grid rejects the whole blob on a bad axis) | REAL → doc rewritten: the #204 tolerance lesson, divergence stated |
| 17 | LOW | Three copies of find-or-push in the pure module (needless MSI surface) | REAL → `index_or_push` helper; 2 plain sites swapped (the keyed-payload site compares on key only — kept, commented) |
| 18 | LOW | `kind_word` pub with no cross-module consumer | REAL → `pub(crate)` (matches `kind_of`'s posture; in-module P4 tests don't need pub) |
| 19 | LOW | Two spellings of the 1000-cap in adjacent rebuilds | REAL → the add loop uses `.take(1000)` (one spelling) |
| 20 | LOW | rustfmt glued pre-existing block comments onto the new clear lines at close_tab_at / close_project_at | REAL → blank lines restored |
| 21 | LOW | The naming card rebuilds `content_labels()` per frame while open | ACCEPTED-AS-IS with a comment (modal, short-lived, decorative line); the critic's struct-promotion alternative recorded as an option |
| 22 | LOW | `sanitize_title`-empty-means-none spelled four ways | PARTIAL → the two new app.rs binds use the #177 `then_some` spelling; a `clean_name` helper recorded as follow-up |

**Rejected as designed** (critic raised, no change): `ResolvedArrangement.dropped` unused outside
tests-to-come (doc'd; REQ-004 units assert it); `axis: String` not an enum (an enum would nuke the
table on one typo — the tolerance is the point); `pane_targets` by-value snapshot (safer than ids
under a settings reload); the two-lookup pane_name (label in rail_rows, filter in the render arm)
— a `named` RailRow field would grow a #390-pinned struct for a HashMap lookup.

**Clean areas verified by the critics** (evidence in their reports): the `T=` wire round-trips
all 6 shapes incl. legacy + hand-mangled (the part-count discriminator is sound — the old writer
could emit at most one separator); ordinal/group math exhaustive (twins never split across a
drop); registry refcounts EXACTLY balanced through apply incl. mid-build failures; spawn safety —
the cwd key reaches `libc::chdir` via alacritty pre-exec, never a shell string (traced to the
syscall); §20 clean-room holds (flat ordinal-keyed TOML is structurally unlike Zed's recursive
SQLite tree and Warp's YAML launch configs; no source consulted); no new persistence exposure
(same file/mode as the shipped shell blob's cwds); pre-#399 files round-trip byte-identically;
dispatch band arithmetic sound at every boundary; totality over the #395 empty workspace at every
new accessor.

**Pre-existing, recorded (not fixed here):** the shipped boot restores shared finding #8's
read-then-cap order (now fixed via the shared extraction); the fleet-vs-renaming_tab arm-order
starvation (#12's mirror); uncapped cell counts on hand-edited entries (same trust as the shipped
grid blob); the raw-name workflow palette row precedent (:2047).

## Phase 4 — Validate

**Units written + RUN (all green):**
- `arrangements.rs` — 9 tests: the byte-exact serialized-TOML pin (REQ-005 — no id field exists to
  leak), round-trip identity, serde tolerance split (per-field defaults; `name` required),
  future-kind preservation + resolve-drop, kind/axis word round-trips, ordinal identity (twins
  share / distinct count / interleave / no-id / cross-kind), the REQ-004 drop table (9 cell
  permutations + the asked-predicate trace proving git/files/unknown never consult it), all-drop /
  empty skips + empty-key normalization, group reunion (twins reunite, ordinals split, drops
  never split a twin pair), upsert by (scope, name).
- `grid_layout.rs` — `shell_codec_pane_name_rider`: all wire shapes byte-exact (titled+named /
  named-untitled empty slot / hostile-name sanitize / 60-cap / whitespace-name → #177 bytes /
  hand-mangled double-empty), + the reassigned 2-separator pin updated in `shell_codec_titles`.
- `tabs.rs` — `rail_rows_named_arrangement_shows_the_name` (name label + the unnamed neighbour's
  byte-identical "PANE 2" — the counter counts every multi-cell tab).
- `settings.rs` — `panes_setting_round_trips_and_tolerates` THROUGH `persist_panes` (the trigger,
  killing its `Ok(())` mutant) + absent/malformed/lean/no-name arms (the inspect-#5 house pin).
- `app.rs` — `special_dispatch_statics_resolve_to_no_verb_and_collide_with_nothing` (the inspect-
  HIGH reverse guard + no-dup-ids) + `open_editor_instance_gates_and_resolves` (the extraction
  unmasked previously-masked logic → direct kills: open/share/stat-gate/binary/missing).
**Headless drives (4 new; lane total 203, all green):**
- `naming_commit_rider_restart_and_unname_headless` — commit → entry + bind + rider; a REAL second
  boot from the same config dir → back ONCE, named, 2 cells (REQ-003 driven); empty re-commit
  un-names, the recipe survives.
- `apply_arrangement_reunites_twins_and_drops_dangling_headless` — twins → ONE instance, 2 views;
  dangling cwd drops with the rest intact; additive named tab; all-dangling → skip + flash.
- `naming_draft_choke_points_headless` — text_input_blocked / close_transient / close-tab
  dismissal / fleet-brief refusal (silent) / blank-target flash (REQ-007).
- `palette_pane_rows_rebuild_scoped_and_sanitized_headless` — scope filter, sanitized display,
  duplicate first-wins, contiguous ids, by-value snapshot, rebuild-to-zero.
**Full suites:** `cargo nextest run -p marley` green (887 total incl. the #390/#394/#396/#397/#398
suites unchanged — REQ-008); doctests via the gate.
**Live drive (the sandbox-HOME harness):** the freshly-signed bundle tripped a macOS
removable-volume TCC prompt on Chad's real config (the app + project root live on
/Volumes/Offload) — NOT clicked (a security grant is the operator's); worked around by launching
with `open --env HOME=<scratch sandbox>` + a seeded `T=H:t,t` session on the internal disk: no
removable read, no dialog, and the operator's real state untouched (an improvement over #398's
real-config drives — recorded for the harness docs). Captures (scratchpad 399-captures/):
marley-1-boot-split (PANE 1 + cells), marley-2-naming-card ("Name this arrangement" /
"dev cockpit▏" / "proj · proj"), marley-3-named-rail ("dev cockpit" replaces PANE 1),
**marley-4-restart-named (⌘Q + relaunch → the rail comes back NAMED — REQ-003 on live pixels)**,
marley-5-palette-pane-row ("Name Pane…" + the #241-disambiguated add rows + "Pane: dev cockpit").
The sandbox settings.toml holds the real artifact: `[[panes]]` with name/scope/axis +
`[[panes.cells]]` kind/key/**ordinal 0 and 1** (the two seeded same-cwd cells are DISTINCT
instances — the ordinal doing its job in production data), and the shell blob carries
`T=\x1Fdev cockpit\x1F<blob>` (the empty-title-slot rider). No id anywhere in the bytes.
**Parity pair (React ↔ Marley):** React refs 399-react-1-naming-card / -2-named-rail-restored /
-4-palette-filtered vs Marley marley-2 / -3+-4 / -5 — same 3-line card (caption / draft▏ / cells
joined " · "), same named-row presentation, same palette row text. **Pixel-sampled: the card
background is rgb(26,27,31) in BOTH** (PIL sample; the muted caption grays match family) — the
card reuses the #204 chrome and the row the #390 style, so no new color existed to diverge.
Verdict: 1:1 per MARLEY-PARITY.md.
**Mutants recheck (`PR-claude-trace-the-real-cargo-mutants-list`):** `--list -f` on every touched
file — arrangements.rs 31 real mutants (no skips, per-mutant kill traced to an assert);
app.rs 73 listed, the four masked #399 shims correctly ABSENT (skips bound), the new free fns
present and now unit-covered.
**Gate story (red→green, each red fixed at source):** run 1 RED ×3 — gate:1 rustfmt (the
heredoc-appended drives; `cargo fmt --all`), gate:2 clippy (`type_complexity` on the snapshot
group table → a named `SnapshotGroups` alias), gate:4 coverage (three missed lines + a missed
"function" = ONE never-executed test closure — the deliberately-unreachable `panic!` predicate;
restructured to the recorder pattern with a legit key so it executes while still proving empty
keys never reach it). Run 2/3 RED ×1 — gate:4 at 44651/44650 lines: ONE phantom missed line in
arrangements.rs that NO per-line view could show (lcov DA all-nonzero, annotated text clean,
region cross-reference clean — llvm-cov's line-summary merge artifact from never-run generic
instantiations inlined into non-calling test binaries' rlib covmaps; `--show-missing-lines`
even flips the exit code, masking it). Fixed at the ROOT: the module is now **generic-free** —
`index_or_push<T>` un-extracted back to inline find-or-push, `resolve_arrangement` takes
`&mut dyn FnMut` (one compiled body; documented in the module NOTE — the syntax/parse.rs
phantom class the gate already documents). Post-fix: arrangements.rs 100/100/100 clean.
**Run 4: GATE GREEN [diff] 15/15 — coverage 100% lines, MSI 100.0 (23/23), receipt written.**
**Pre-existing, not in scope:** none new surfaced by the runs.

## Phase 5 — Complete

**Documentation (§21):** CHANGELOG.md [Unreleased]/Changed TICKET-399 entry (above #398's);
docs/marley_architecture/pane-composition-model.md Q5 train slice-6 → "✅ SHIPPED (M28 #399)"
with the as-shipped shape (ordinal semantics, single-rebuild-authority rider, dangling-drop
reopen). **Parity sync:** marley-web/docs/MARLEY-PARITY.md — § Shared vocabulary gains the #399
naming-vocabulary paragraph (SavedPane ↔ PaneArrangement, applyPaneUpdate ↔ apply_arrangement,
the filter back-port note) + a port-map row (utils/arrangements.ts + overlays/NamePaneCard.tsx ↔
arrangements.rs + the app.rs wiring); the one port-time deviation (the empty-commit UN-name from
inspect #4) was back-ported to Workspace.tsx, so the 1:1 holds at close.
**Forge capture:** AAR b6ede482 submitted (completed). Failures recorded:
`BF-claude-static-command-id-collision-swallowed-the-verb-001` (HIGH — the dead palette verb),
`BF-claude-persisted-visual-order-came-from-id-sort-not-the-tree-001`,
`BF-claude-llvm-cov-phantom-line-from-generic-instantiations-001` (infra — 3 gate cycles).
Prevention rules: `PR-claude-a-next-free-id-claim-needs-the-full-allocator-map-001`,
`PR-claude-persisted-order-names-its-walk-001`,
`PR-claude-cov-100-pure-modules-stay-generic-free-001`.
**Lessons (what worked):** the sandbox-HOME live-drive harness (`open --env HOME=<scratch>`)
dodged the removable-volume TCC prompt AND kept the operator's real state untouched — strictly
better than driving the real config; candidate for the selftest README. The four-critic inspect
caught a ship-stopping id collision no test could see and reshaped three fix classes before
Validate. The double-boot headless drive proved REQ-003 without pixels; the live captures then
confirmed the same flow on real glass with a pixel-identical parity sample.
**Ticket:** local doc → docs/planning/tickets/closed/ (status closed); forge #399 → done.
**Archive:** the spec/notes pair → docs/planning/pipeline/completed/.
