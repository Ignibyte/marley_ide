# Add-anything-to-a-pane + the ContentId cross-link — Notes

- **Forge ticket:** #398 7aa54235-760c-4191-a5e6-12a5b65dfa7e
- **AAR:** bad06f88-7368-44bf-bd67-58947689e178 (opened 2026-08-05 at promotion)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-398-add-to-pane-cross-link.md
- **Pipeline spec:** 398-add-to-pane-cross-link.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** the #388 train slice-5 (M28 "The Registry Payoff", sprint 6558258f-a0d2-45fb-b601-cd5329a1cd77):
  the gesture/command set to drop an OPEN ContentId into a split cell (palette verb + Section/Pane
  context-menu rows — one instance, many views, live for the first time) PLUS the cross-link half deferred
  from #390 (ContentId-derived Pane-row labels + Editor/Terminal home-section cross-listing). View-count
  rules per the #394 registry; terminals keep the off-thread reap. React-first (policy 2026-08-04) applicable.
- **Classification / tier:** feature, **M** (the #388 Q5 sizing for slice-5); **hard-ordered AFTER #396
  (slice-2, terminals) AND #397 (slice-3, editors) ship** — both HARD deps; pre-migration there is nothing
  to `acquire_view`. /work must not promote while either is unshipped.
- **Forge recall (§18.3):** knowledge-search "rail rows section context menu create verbs" + "palette
  command id append rebuild" surfaced prevention-rule/AD nodes (top palette hit
  `86107649-3fe6-40a5-865a-1f7d2c4d20e6`, rank-1 semantic; the MCP returns node ids without names/bodies,
  and knowledge-context needs an open AAR — ours is pending-promotion). Rule text recovered from the local
  corpus instead: **the #204/R1 append-vs-rebuild constraint** (warp-workflows.notes.md:148-149 + :193-194 —
  "append keeps the id = base+index consistent — v1 has no delete, so no id reuse/collision; a future
  delete needs a rebuild") → locked as **D5** (open content DOES close, so the add-to-pane palette range
  must rebuild, never append); **`PR-claude-new-palette-arm-extend-the-exhaustive-map-test`**
  (369-fleet-rail-readonly.notes.md:322) → REQ-006's verify. Carried idioms: #175 target-rides-the-kind
  (context_menu.rs:63-71), #174 activate-first + #387-F1 persist-the-switch (app.rs:6772-6820),
  `BF-claude-skip-detach-pump-fleet-live-001` (Floors). ADs: `AD-claude-pane-content-id-registry-001`,
  `AD-claude-one-instance-many-views-001` (pane-composition-model.md). docs-search returned no
  Marley-relevant hits for this query (the shared index surfaced another project's corpus) — noted, honest.
- **Discovery (the edit surface for Design):** `content_registry.rs` (acquire_view:66 / release_view:78 —
  `#[must_use]` contracts load-bearing); `context_menu.rs` (const row tables + `items_for`:182-194, the
  #393 extension-point comment :111-117; a new target-bearing MenuKind is the likely (a)-surface);
  `tabs.rs` (`rail_rows`:786 — the Panes derived branch :843-877 emits "PANE n":857 + "pane k":868 to be
  relabeled; cross-list rows join the per-section loop :879-895); `palette.rs` (`dynamic_command_index`
  :183-187; `CommandId` ranges); `app.rs` (bases :614/:617/:620; boot command build :1883-1931 —
  rebuild-not-append delta lands here; `open_section_menu`:6755; `dispatch_section_verb`:6772;
  `split_focused_pane`:6827 — the split mechanics the add reuses with acquire instead of spawn;
  `jump_to_pane`:7024; Arrangement render arm :16515, search-filter guard :16522). #396's 7 spawn-drop
  close paths (per TICKET-396) are the release sites the symmetry lens walks — Phase 2 re-verifies their
  shipped line numbers. marley-web: LeftRail.tsx (panes/items filter :96-100, PANE rows :281),
  ContextMenu.tsx (#393 tables mirrored :92-118), views/SplitTerminalView.tsx (the mocked split),
  `PaneItem { type, name }` = the ContentId stand-in (MARLEY-PARITY.md § Shared vocabulary).
- **Prior-art sweep (§20):** Warp maps — thin coverage, no move-pane/add-to-split behavior mapped
  (00-overview.md:45 pane-group MATCH only; observed/ has one unrelated capture); Zed
  07-workspace-panes-palette.md — behavior-level drag-item-between-panes + clone-on-split
  (:136/:157/:163), the one-instance end state, verb-first here; published material — none applicable;
  permissive deps — **gpui 0.2.2 ships typed on_drag/on_drag_move/on_drop (div.rs:62-499)**: recorded as
  the v2 drag substrate, NOT adopted (D1 verb-driven v1). No Warp/Zed source read.
- **Decisions:** D1 verb-driven through the ONE menu machinery; D2 acquire/release symmetry over the
  unchanged #396 close contract; D3 pure label derivation, rail_rows stays pure; D4 cross-listing
  presentation-only; D5 palette rows rebuild-never-append (#204/R1); D6 persistence untouched
  (session-local sharing; ids never serialize). Four D-OPENs to Phase 2 with recommendations:
  GESTURE-SURFACE (content-row menu + palette rows, NOT the ＋ create-menus), LABEL-SHAPE (cell rows get
  content labels; "PANE n" header stays until slice-6), CROSSLIST-MARKER (muted trailing glyph, confirmed
  in the POC), EMPTY-CELL-TARGET (create-the-split; no empty cells exist to fill).

## Phase 2 — Design

### Shipped-shape verification (the #384-D7 pattern — bind to reality, not predictions)
All spec line-number predictions re-verified against HEAD (425bf3a): `acquire_view`
content_registry.rs:75 / `release_view`:78→:87 (`#[must_use]` both); `resolve_open`/`find_open`/
`release_editor_views` content.rs:142-195; `section_items`:171 / `items_for`:182 + the #393
extension-point comment :111-117; `rail_rows` tabs.rs:791 (Panes branch :848-883, "PANE n" :862,
"pane k" :873); `dynamic_command_index` palette.rs:183; bases app.rs:636/639/642 (CONNECT 1000 /
THEME 2000 / WORKFLOW 3000); boot command build :1940-1991; palette dispatch :3290-3320;
`open_section_menu`:7034 / `dispatch_section_verb`:7051 / `split_focused_pane`:7106 /
`run_context_menu_action`:7136 / `jump_to_pane`:7302; `split_focused(axis, dir, cid)`
workspace.rs:532 / `open_pane(axis, dir, PaneContent)`:544; render arms Arrangement :17108 /
Pane :17154 (both skipped under `session_filter`); palette opens at :8714 (`palette_open = true`,
fresh `PaletteState`). Editor cells: `PaneContent::CodeView(EditorSurface)` (workspace.rs:352,
view rows carry id+path); terminal cells: `PaneContent::Terminal(ContentId)` (:345). Close paths
release per-view at app.rs:1642/5393/5426/5598/7743/7828/7903/9211/9220/18682/18690 — all route
the #396/#397 contracts; the add introduces NO new close path.

### Architecture
The add gesture composes five shipped mechanisms — nothing new below the shim layer:
right-click/palette → `MenuKind::AddToPane { id }` or the snapshot row → the ONE dispatch
`add_content_to_split(cid, axis)` → `acquire_view(cid)` → mount (`split_focused` for a terminal
cid; `open_pane(CodeView(EditorSurface::for_view(cid, lazy CodeViewState)))` for an editor — the
#397 hit-path idiom, zero IO) → `persist_grid`. Close symmetry is the UNCHANGED #396/#397 routing
(slice-5 only makes view_count>1 reachable). The rail gains a registry-derived label input +
cross-list rows, both derived fresh per render (#390 discipline).

### D-OPEN resolutions (all four settled)
- **D-OPEN-GESTURE-SURFACE = (a)+(b), not (c), no new chords.**
  (a) `MenuKind::AddToPane { id: ContentId }` (target rides the kind — #175), opened by
  right-click on rail rows that resolve to addable content: a single-cell terminal tab row (its
  sole cell's cid), a code tab row (its surface's ACTIVE file's cid, resolved at menu-open), and
  a CrossRef row (its cid). Rows `ADD_TO_PANE_MENU_ITEMS: [(AddToSplitRight, "Add to Split
  Right"), (AddToSplitDown, "Add to Split Down")]` — payload-less `MenuAction` arms.
  (b) Palette: dynamic "Add to Split Right: <label>" rows (split-right only — the #246
  "Split Right → File" naming precedent; the menu carries both axes), id range
  `ADD_TO_PANE_BASE = 4000` above WORKFLOW_BASE. The ＋ create-menus stay create-only (c
  rejected — the #393 table's documented meaning). ZERO new key chords → keymap.rs untouched,
  no collision surface.
- **D5 mechanism = rebuild-at-palette-open + dispatch belt.** At `palette_open = true` the add
  block is rebuilt: strip every `ADD_TO_PANE_BASE`-range row from `self.commands`, re-append one
  per CURRENT addable registry entry, snapshot `add_targets: Vec<ContentId>` in the same order.
  The palette is modal BUT the #173 pump can close dead panes while it's open, so dispatch
  resolves `add_targets[index]` and the belt is `acquire_view(cid) == None` (id died) → safe
  no-op. A ContentId target is immune to index-shift misdispatch by construction — worst case a
  dead id no-ops; never the wrong content.
- **D-OPEN-LABEL-SHAPE.** Terminal cell → the #177/#201 lineage via the EXISTING `display_title
  (custom, command, cwd, "terminal")`: the shim builds `ContentLabels { terminal:
  HashMap<ContentId, String> }` per render (custom titles walk tabs first-in-order-wins; command/
  cwd from the registry session — masked exactly like `live_tab_title`); `rail_rows` consumes
  the map purely. CodeView cell → active row's file BASENAME (pure from the ws model — the
  surface rows carry paths). FileTree → "files"; Git → "git diff". Cell rows get content labels;
  the "PANE n" arrangement header stays positional until #399 naming. Clamp = the rail's existing
  render truncation (no pre-clamp in the label text).
- **D-OPEN-CROSSLIST-MARKER = a muted trailing "⊞" (U+229E) text glyph.** Zero new SVG assets,
  theme-aware via text_color(muted), zero row-height cost; POC confirms the look first.
- **D-OPEN-EMPTY-CELL-TARGET = create-the-split (confirmed).** The add splits the FOCUSED pane
  of the active tab's grid along the verb's axis; the new cell holds the acquired view. No grid /
  empty workspace → enablement (REQ-007), not a target rule.

### Derived decisions (recorded here, not in the spec)
- **D-CROSSLIST-SEMANTICS — one home entry per content, subtraction per kind.**
  TERMINALS: a cid that is the sole cell of a SINGLE-cell terminal tab is "tab-listed" — its tab
  row IS the home entry, and it gains the ⊞ marker iff the cid also sits in a multi-cell grid
  cell (the post-add "original stays open" state). Every OTHER terminal cid mounted in a
  multi-cell grid cell (split-spawned twins, arrangement cells) gets a `RailLevel::CrossRef` row
  in the Terminal section — its FIRST home entry ever (the REQ-004 pane-only case). Invariant:
  every live terminal cid has exactly ONE Terminal-section entry (tab row or CrossRef).
  EDITORS: a CrossRef row per cid mounted in ANY CodeView pane cell (grids of any size — a
  1-cell grid holding a CodeView still pane-mounts it); per-file home entries, including the
  pane-only #246 split files. Code tab rows never mark (a 1:N surface can't carry a per-file
  marker honestly). CrossRef click = jump to the FIRST mounting cell's (project, tab, pane) in
  walk order (deterministic) via the Pane-row click path (switch project/tab + focus cell k +
  persist — the #174 idiom). CrossRef rows participate in the "Search tabs" filter BY LABEL
  (real content labels — unlike Arrangement/Pane rows, which stay filter-hidden; the relabeled
  Pane cell rows ALSO stay filter-hidden — structure rows, deliberate, the CrossRef rows are
  the searchable content entries).
- **D-TERM-VIEW-STATE — terminal twin views share display state in v1.** Viewport/selection/
  folds live in `TerminalPane` (= content), so twin cells scroll/select together — tmux-mirror
  semantics, deliberate and recorded (this answers the #396 content.rs comment that deferred
  "the per-view split of display fields" to this slice: the answer is shared-in-v1; a per-view
  split is its own #397-style migration if it ever grates). REQ-001 needs only shared session/
  scrollback + input-to-one-PTY — both hold. PTY sizing with twin cells inherits #396's
  focused-cell behavior (inspect lens: no panic).
- **Enablement (REQ-007)** — pure `ContentKind::addable(self) -> bool` (Terminal/Editor true;
  FileTree/Git/Cockpit/Browser false — exhaustive, no catch-all) composed with `has_grid`
  (`try_workspace().is_some()`): menu-open refuses, palette rows build empty, dispatch re-checks
  (belt). Empty workspace → absent, totality via the #392 try twins.

### File manifest — marley-web FIRST (the React-first halves)
1. `artifacts/marley-ide/src/App.tsx` — add-to-pane dispatch over `panes[].items` (+
   `PaneItem` reuse — `{type, name}` is the ContentId stand-in); palette add-rows source.
2. `artifacts/marley-ide/src/components/LeftRail.tsx` — pane child rows read `item.name`-derived
   labels (not "pane n"); CrossRef rows + ⊞ under Editor/Terminal; tab-row ⊞ marking;
   right-click on content rows → AddToPane menu.
3. `artifacts/marley-ide/src/components/ContextMenu.tsx` — `MenuKind` AddToPane + the 2-row
   table (label-for-label with the Rust consts).
4. `artifacts/marley-ide/src/components/overlays/CommandPalette.tsx` — dynamic "Add to Split
   Right: <label>" rows.
5. `artifacts/marley-ide/src/components/views/SplitTerminalView.tsx` — twin cells render shared
   content (terminal twins share `terminalHistory[name]` — already keyed by name; file twins
   already share `docs` per #397).

### File manifest — Rust (ported 1:1 after the POC confirms)
1. `crates/marley_app/src/content.rs` — `ContentKind::addable` (exhaustive method + unit).
2. `crates/marley_app/src/context_menu.rs` — 2 `MenuAction` arms, `MenuKind::AddToPane { id }`,
   `ADD_TO_PANE_MENU_ITEMS`, `items_for` arm; full-slice-equality + routing tests.
3. `crates/marley_app/src/tabs.rs` — `ContentLabels` (Default-able), `RailLevel::CrossRef`,
   `RailRow.pane_marked: bool`, `rail_rows` grows the `labels` param + cell-label derivation +
   CrossRef emission + tab-row marking (all pure); ~57 existing call sites mechanically gain
   `&ContentLabels::default()`; new pure tests.
4. `crates/marley_app/src/app.rs` — `ADD_TO_PANE_BASE` const; `add_targets` field; the
   palette-open rebuild; the dispatch arm; `add_content_to_split` (masked shim);
   `run_context_menu_action` arm; rail right-click wiring; the CrossRef render arm + tab-row
   marker glyph; the per-frame `ContentLabels` build.
5. `crates/marley_app/src/headless_drive.rs` — the two driven scenarios (below).

### Regression test plan (per REQ)
| REQ | Test |
|---|---|
| 001 | headless `terminal_add_to_split_shares_one_session_headless`: add via `add_content_to_split` → both cells same cid, `view_count == 2`, ONE registry entry, input through either focused cell lands in the one PTY (blocks identical), origin tab row still present + ⊞-marked |
| 002 | headless `editor_add_to_split_shares_one_buffer_headless`: tab edit visible in the added cell, per-view carets independent (the #397 park machinery), view_count 2; #246 split-to-file suite green unchanged |
| 003 | `rail_rows` units: pane cell labels per kind (terminal map / code basename / git / files); a changed labels map changes the label (derived-fresh); #390 numbering suite green |
| 004 | `rail_rows` units over the 4 terminal cases (tab-listed only / pane-only / both / none) + editor pane-only & tab+pane CrossRef; the one-home-entry invariant; CrossRef click path inspected §18.1 |
| 005 | headless: ⌘W one twin cell → survivor intact, no reap; last view → reap (the #396 timing suite green); close-tab-with-paned-view releases exactly one |
| 006 | unit over the rebuild: open→close→reopen → rows rebuild, dispatch resolves the CURRENT target; stale-cid dispatch → safe no-op; no new static palette arm (the exhaustive `action_for_command` map test needs NO extension — recorded per `PR-claude-new-palette-arm-extend-the-exhaustive-map-test`) |
| 007 | enablement units exhaustive over `ContentKind` × has_grid; #395 empty-workspace + #392 totality suites green |
| 008 | the React↔Marley parity pair captured at Validate |

### Reference (§20) — confirmed
Unchanged from Phase 1: Zed's behavior map (items pane-hostable without closing origin,
clone-on-split → one entity many views) is the behavior matched, verb-first; Warp thin-coverage
honest; gpui drag primitive recorded-not-adopted (D1). No copyleft source consulted.

### Risks / notes
- `rail_rows` signature churn (~57 test sites) — mechanical, sed-able.
- Per-frame `ContentLabels` build walks tabs+registry — same cost class as the existing per-row
  `live_tab_title`; N is small (open tabs).
- Terminal twin display-state shared (D-TERM-VIEW-STATE) — deliberate v1 semantics.
- Stale palette snapshot across the #173 pump — belted at dispatch (acquire None → no-op).

## Phase 3 — Implement

### React-first (built + confirmed FIRST, per the parity policy)
Built in marley-web, iterated at localhost:5173, typecheck green, four states captured + READ
(scratchpad `398-captures/`, also `.playwright-mcp/398-react-*.png`):
- `398-react-1-add-menu.png` — right-click on the "Marley" terminal row → the 2-row AddToPane
  menu at the pointer ("Add to Split Right" selected). Rail behind it already shows the full
  cross-link design: `code_syntax.rs ⊞` CrossRef under Editor, `Marley ⊞` marked tab row under
  Terminal, PANE 1 cells labeled "Marley"/"code_syntax.rs".
- `398-react-2-twin-terminals.png` — THE PAYOFF: after "Add to Split Right", two cells BOTH
  titled Marley rendering the IDENTICAL scrollback (ls / git log / cargo --version) — one
  session, two views; rail PANE 1 → Marley, Marley.
- `398-react-3-palette-rows.png` — ⌘⇧P "add to split" → the three dynamic rows (Marley,
  commit.md, code_syntax.rs) enumerating the CURRENT open set.
- `398-react-4-grid-file-added.png` — picking "Add to Split Right: code_syntax.rs" → the 3-cell
  grid (twin terminals + the file cell, syntax-highlighted); rail shows all three cell labels +
  both ⊞ markers.
Files: `utils/addToPane.ts` (NEW — the ONE dispatch `addToPaneUpdate` + `addableContent`),
`components/ContextMenu.tsx` (AddToPane kind + ADD_TO_PANE_MENU_ITEMS), `components/LeftRail.tsx`
(PaneMark ⊞, cell labels = item names, CrossRef rows, marked tab rows, onItemMenu right-click),
`components/views/SplitTerminalView.tsx` (cells DRIVEN by panes[0].items — twin cells resolve one
shared entry), `overlays/CommandPalette.tsx` (dynamic add rows, mount-fresh rebuild),
`pages/Workspace.tsx` (dispatch arms + wiring).

### Rust (ported 1:1 after the POC confirm)
- `content.rs` — `ContentKind::addable` (exhaustive; Terminal/Editor true, declared kinds false).
- `context_menu.rs` — `MenuAction::{AddToSplitRight, AddToSplitDown}`, `MenuKind::AddToPane
  { id: ContentId }` (target rides the kind), `ADD_TO_PANE_MENU_ITEMS`, the `items_for` arm.
- `tabs.rs` — `ContentLabels` (shim-built registry inputs; Default for the pure suites),
  `file_basename` (pub(crate) — shared with the palette row titles), `cell_label` (exhaustive
  over PaneContent: terminal map / code basename / "files" / "git diff"), `cell_content_id`,
  the `pane_mounts` pre-pass (terminal multi-cell mounts, editor any-cell mounts, tab_listed),
  `RailLevel::CrossRef`, `RailRow.{pane_marked, content}`, `rail_rows(…, labels)` — Pane cells
  read content labels; Tab rows carry content ids + the single-cell-terminal ⊞ mark; CrossRef
  rows emit after the section's tab rows (terminals: not-tab-listed only; editors: every paned
  file), walk-order deterministic. ~15 test call sites mechanically gained the labels arg.
- `app.rs` — `ADD_TO_PANE_BASE = 4000`; `add_targets: Vec<ContentId>` (the dispatch snapshot);
  `content_labels()` (masked — the #177/#201 lineage via `display_title`; custom titles walk
  tabs first-wins); `rebuild_add_to_pane_commands()` (masked — strip range + rebuild sorted by
  (label, id) — the registry HashMap order never becomes row order; empty when no grid);
  `add_content_to_split(cid, axis)` (masked — the ONE dispatch: acquire → `split_focused` for a
  terminal / `open_pane(CodeView(for_view(cid, lazy)))` for an editor → persist; acquire-None
  belt for dead cids); `open_add_to_pane_menu` (masked — addable + has-grid gates,
  close_transient_overlays, kind-borne target); palette-open rebuild call; the dispatch-chain
  arm via `dynamic_command_index`; `run_context_menu_action` arms + kind-borne `add_target`;
  render: Tab-row ⊞ + right-click, Pane-row right-click, the full CrossRef arm (label-filtered
  under search, Pane-style click-to-cell, ⊞, right-click); the wheel-clamp recount uses default
  labels (row COUNT is label-independent — labels change text, never emission).

### Deviations from design
None structural. One addition beyond the manifest: right-click add on Pane CELL rows (the POC
grew it naturally; the Rust mirrors it — same `row.content` plumbing, no new machinery).

`cargo check --workspace --all-targets` + clippy clean; fmt applied. Tests are Phase 4.

## Phase 3.5 — Inspect

Four independent critics over the diff (lenses: refcount/lifecycle symmetry · rail derivation
correctness/staleness · dispatch integrity/modal discipline · masking/persistence/twin edges).
Findings reviewed skeptically; every real one fixed at source. Ledger:

| # | Severity | Finding | Verdict | Fix |
|---|---|---|---|---|
| 1 | HIGH | One-home-entry invariant broken: `tab_listed` was single-cell-only, so every ordinary #155 split's own cells became Terminal CrossRefs — Tab row + 2 CrossRefs for a plain 2-cell split (rail regression) | REAL — the shipped cross-list semantics were wrong | Semantics reworked: a tab's row is the home entry for ALL its cells; terminal CrossRefs REMOVED; the ⊞ marks a tab any of whose terminal content is hosted by ≥2 DISTINCT tabs (`terminal_hosts` count — the post-add twin marks both the origin and the hosting arrangement). Editor CrossRefs stay (a CodeView cell files under a Terminal tab, so its Editor entry exists nowhere else — the spec's pane-only case IS the #246 file). Back-ported to the POC |
| 2 | HIGH | Content-keyed maps (`agents`/`remotes`/`last_agent`/`notify_ticks`) scrubbed per-VIEW at 5 close sites — closing one twin cell deleted a live agent's Fleet identity; a degraded remote twin lost its never-auto-close guard | REAL | Scrubs unified INTO `release_grid_terminals`, firing only on the LAST view's drop; ⌘W + pane × route through it; close-tab/close-project pre-scrub retain blocks deleted; the pump's dead-pane scrub moved inside its release-Some arm |
| 3 | HIGH | PTY resize thrash: per-CELL loop over a per-CONTENT `pty_size` shadow — twin cells of different size re-ioctl every frame (SIGWINCH storm vs a reflowing TUI) | REAL (reachable: split right, then Add-to-Split-Down the left terminal) | ONE resize authority per content: the FOCUSED cell wins (tmux-mirror per D-TERM-VIEW-STATE), else first in walk order; dedupe set per frame |
| 4 | HIGH | `add_content_to_split` gated on `try_workspace`, which falls back to ANY grid — palette/menu add while a cockpit/editor tab is active would mount + persist into an unseen background tab | REAL | Gate = the ACTIVE tab owns a grid (total over the #395 empty workspace via `.get`); same gate on menu-open (target project's active tab) and on the palette rebuild (rows absent — REQ-007's absent-or-inert) |
| 5 | HIGH | `pane_mounts` carried an unreachable `else { continue }` (pane_ids/state read the same map) — a forever-dead line, gate:4 unpassable | REAL | The R30 expect idiom (workspace.rs:590 precedent) here AND in the rail_rows Pane row (same dead Option) |
| 6 | HIGH (test) | `rail_rows_nested_panes` still asserted the deleted positional "pane 1"/"pane 2" labels — suite red as committed | REAL | Asserts updated to the content-label fallback ("terminal"); the populated-labels unit lands in Phase 4 |
| 7 | MED | Right-click add skipped the spec's LOCKED activate-first idiom — a cross-project right-click silently mounted into the ACTIVE project's grid | REAL (spec D-OPEN-GESTURE-SURFACE locked "activate-first #174 + #387-F1") | `MenuKind::AddToPane` gained `p` (rides the kind); new `dispatch_add_to_pane` mirrors `dispatch_section_verb` (switch → sync → add → persist-the-switch); the palette stays active-grid by design (invoked "here"). Render sites pass `row.project` |
| 8 | MED | `close_transient_overlays` missed `completion` + `renaming_tab` — the inline rail rename is an ordinary state for a rail right-click; the menu opened keyboard-dead under the rename's key grab (gap inherited by #393's section menu too) | REAL | Both added to `close_transient_overlays` (the one place — fixes #393's ＋ menu as a side effect). The deeper editor overlays (goto_line/renaming_symbol/…) are pre-existing #393-era posture, unchanged — recorded, not this ticket's regression |
| 9 | MED | Palette strip window `[4000,5000)` vs unbounded push — >1000 addable contents would leave ghost rows that survive every rebuild | REAL (theoretical volume, real mechanism) | Top-block retain (`id < ADD_TO_PANE_BASE` strips everything at/above) |
| 10 | MED | WORKFLOW_BASE is user-growable with the new block stacked above — workflow #1001 would mint id 4000, shadowing the add rows | REAL (the first user-growable range under a neighbor) | The range's 1000-id budget enforced at all three sites: boot `.take(1000)`, runtime append guard, dispatch `min(1000)` |
| 11 | MED | Pump double-pumped a twin (per-VIEW snapshot) — `notify_ticks` counted 2×, the #203 threshold fired at half time | REAL | Pump/tick deduped per content (first-walked view wins); `pane_focused` made content-aware (any cell viewing the cid focused ⇒ watched); per-pane dead detection unchanged (converges via the edge-triggered exit re-erroring next tick) |
| 12 | MED | `pane_of_content` doc claimed 1-view uniqueness — now false; jump consumers get an arbitrary twin | REAL (doc/rule gap) | Doc updated: FIRST mounting cell in walk order, deterministic; every twin renders the same session so the jump lands on an equivalent view |
| 13 | MED | Active-section force-expand orphaned: a focused CodeView CELL left Terminal active, so a collapsed Editor section could hide a pane-only file's ONLY home entry | REAL | `active_tab_section` now derives from the FOCUSED CELL's kind (CodeView cell ⇒ Editor active/force-expanded); the matching CrossRef row renders `active: true` |
| 14 | MED | Tab-row add-target was `terminal_id(focused)` — `None` on a mixed grid's CodeView-focused cell (right-click silently dead) | REAL | Resolves through `cell_content_id` of the focused cell — total over cell kinds (a focused file cell targets its file) |
| 15 | MED | Palette rows with identical labels indistinguishable (two idle "zsh" terminals) | REAL (cosmetic; dispatch was already exact via the cid snapshot) | `disambiguate_labels` (#241) over the sorted row labels |
| 16 | MED | Persisted terminal twin restores as two instances | REJECTED as a defect — the spec's D6/Out explicitly defers restore-time de-dup to #399 (codec byte-identical, session-local sharing); editor twins already re-share via `resolve_open` (stronger than promised) | Recorded here; no change |
| 17 | LOW | CrossRef click omitted `check_active_file_external` (#275) vs `jump_to_pane` | REAL | Added after the cell focus in the CrossRef click |
| 18 | LOW | `notify_ticks` never scrubbed by manual closes (pre-existing #396) | REAL (rider) | Folded into the unified last-release scrub (#2) |
| 19 | LOW | Rebuild welded to the one `dispatch_action` arm | ACCEPTED — one open site exists (verified); noted for a future `open_palette()` extraction | No change |
| 20 | LOW | Stray blank line in the struct literal | REAL | Removed |
| 21 | INFO | Five PRE-EXISTING orphaned doc+skip blocks (app.rs ~3827/4862/5547/9566/15001, identical at HEAD) — the skip-detach shape, harmless in masked app.rs | Pre-existing, not #398 | Follow-up candidate recorded, not fixed here |
| 22 | INFO | Scroll-clamp over-count while the search filter hides rows — pre-existing class, #398 adds one more filtered arm | Pre-existing posture | Recorded |

Clean checks confirmed by the critics (evidence in their reports): add-path acquire can never
strand (both mounts infallible, fallible path-read placed before acquire); no double-dispatch
(menu kind cleared before dispatch; single palette-open site verified); the dispatch chain cannot
swallow [4000,5000) below 1000 workflows (now hard-capped); masking intact (4 new skips attached,
0 detached — 315 attributes scanned); persist writes the PATH never the lazy text; twin leaves
codec-safe (two leaves, no double-free); dead-twin close converges in 2 ticks; the add-created
editor cell renders from the Buffer while focused (the resync skip is safe); zero keymap delta;
§20 provenance clean (no copyleft consulted; gpui drag primitive recorded-not-adopted).

Post-fix: `cargo check --workspace --all-targets` + clippy clean, fmt applied, the 58-test tabs
suite green, POC typecheck green (semantics back-ported: terminal CrossRefs removed, ⊞ markers +
file CrossRefs kept).

## Phase 4 — Validate

### Tests written + RUN (all green)
- **Pure units (in the cov/MSI lanes):** tabs.rs — `rail_rows_pane_cells_read_content_labels`
  (per-kind labels: mapped terminal / "terminal" fallback / basename / "git diff"; content ids;
  derived-fresh relabel), `rail_rows_row_count_is_label_independent` (the wheel-clamp guard),
  `rail_rows_marks_foreign_hosted_terminal_tabs` (both hosting tabs ⊞; plain split unmarked; NO
  terminal CrossRefs), `rail_rows_twin_in_own_grid_does_not_mark`, `rail_rows_editor_crossref_rows`
  (per-file dedupe, first-cell coord, cross-project scoping), 
  `rail_rows_focused_codeview_cell_activates_editor_section` (the #386 force-expand fix + active
  CrossRef + the terminal-focused other arm), `file_basename_component_and_fallback`; content.rs —
  `addable_kinds_are_terminal_and_editor` (exhaustive); context_menu.rs —
  `add_to_pane_menu_items_table_and_routing` (full-slice equality + routing + wrap).
- **Headless drives (the REAL RootView):** `terminal_add_to_split_shares_one_session_headless`
  (REQ-001+005: one entry/two views/both cells one cid; twin ⌘W keeps the instance AND the
  content-keyed tag — the inspect scrub fix — last close scrubs + drops),
  `editor_add_to_split_shares_one_buffer_headless` (REQ-002: the ADD verb mounts a live view —
  unsaved text visible, bidirectional edits), `palette_add_rows_rebuild_from_current_set_headless`
  (REQ-006+007: gate empties rows on a gridless active tab; 2 rows contiguous from 4000;
  close → re-based rebuild, never append; dead-cid dispatch inert), 
  `add_to_split_inert_without_workspace_headless` (REQ-007: zero-project totality through both
  dispatch doors).
- **Full suite:** `cargo nextest run --workspace` → **2007/2007 passed** (5 skipped, the standing
  live-lane skips); doctests green. The #390/#393/#396/#246/#395 regression suites are inside
  that count, green unchanged.

### Live-app drive (the pixels — captured + READ, scratchpad `398-captures/`)
Bundled (`scripts/selftest/bundle-app.sh`), launched, driven via drive.swift:
- `marley-1-boot.png` — the restored #397 session already renders the #398 rail: PANE 1 cells
  read **Marley / inspect.md** (content labels, not "pane n"), and `inspect.md ⊞` shows as the
  Editor cross-link row.
- `marley-2-add-menu.png` — right-click on the **Test** terminal row (after activating the Marley
  terminal tab): the 2-row AddToPane menu at the pointer — "Add to Split Right" / "Add to Split
  Down" — label-for-label with the POC.
- `marley-3-cross-tab-twin.png` — after "Add to Split Right": the Marley grid gains a third cell
  viewing TEST's session; the rail shows PANE 1 → Marley / inspect.md / **Test** (the twin cell,
  labeled by the #177 lineage) and **both hosting tabs marked** — `Marley ⊞`, `Test ⊞` (the
  corrected foreign-hosting semantics, live). The Test tab + PANE 2 stay intact — the original
  stays open (model A).
- `marley-4-typed-in-twin.png` / `marley-5-test-tab-sees-it.png` — **the one-PTY proof**: typed
  `echo twin-398` into the twin cell in MARLEY's grid; switched to the TEST tab; its own grid's
  cell shows THE SAME executed block (`✓ echo twin-398` / `twin-398`) — one session, two views,
  across tabs (REQ-001 at the pixel level).
Driven state tidied after capture (the added cell ⌘W-closed; app quit).

### Gate
Run 1 **RED** (three, all fixed at source): gate:1 rustfmt (the appended drive block — fmt run);
gate:4 one uncovered line (tabs.rs `cell_label`'s FileTree arm — a FileTree cell added to the
label test); gate:5 MSI 94.1% — `focused_cell_is_editor && coord == …` at the CrossRef active
flag was an IMPLIED guard (within the active project the coord match already forces the focused
cell to BE that CodeView) — an unkillable `&&→||` mutant; the expression simplified to the coord
match alone (the #200-class lesson: don't guard on an implied condition).
Run 2: **GATE GREEN [diff] 15/15** — coverage 100% lines, mutation 16/16 caught → **MSI 100.0%**,
miri skip-clean, visual/AX pass. Receipt written.

### Pre-existing (not in scope)
None encountered — the full workspace suite was green before and after.

### Parity pair (REQ-008)
React (`398-react-1-add-menu` / `-2-twin-terminals` / `-3-palette-rows` / `-4-grid-file-added`,
captured at localhost:5173 this session) ↔ Marley (`marley-2-add-menu` / `-3-cross-tab-twin` /
`-4/-5`). Verdict per MARLEY-PARITY (Zone A structure/vocabulary): **PASS** — identical menu rows
("Add to Split Right"/"Add to Split Down", 2 rows, row 0 selected), identical rail grammar
(content-labeled cells under the PANE group, muted trailing ⊞ on marked home rows, per-file
editor cross-link rows), the same add semantics (a new cell viewing the ONE shared content;
the origin stays). The POC's semantics were back-ported at inspect (terminal CrossRef rows
removed) BEFORE the Rust landed, so both sides shipped the corrected model. Deltas are the
POC's known mock simplifications (one fixed arrangement, max-3-cell layout, single project) —
recorded in MARLEY-PARITY as POC discipline, not surface drift.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG [Unreleased]/Changed TICKET-398 entry (above #397's);
  `docs/marley_architecture/pane-composition-model.md` Q5 slice-5 → "✅ SHIPPED (M28 #398)" with
  the as-shipped shapes (incl. the inspect-corrected cross-list semantics + the lifecycle
  riders); `marley-web/docs/MARLEY-PARITY.md` § Shared vocabulary gained the #398 ADD
  vocabulary paragraph (menu/palette/dispatch pairing + the ⊞ rules, both sides aligned).
- **Parity sync (c):** the POC matches what shipped — the inspect semantics correction
  (terminal CrossRefs removed, foreign-hosting ⊞) was back-ported to LeftRail.tsx BEFORE the
  Rust landed; POC typecheck green.
- **Forge capture (§19):** AAR `bad06f88-7368-44bf-bd67-58947689e178` submitted (completed,
  effectiveness 5, 4 novel findings; distillation/drift/pattern jobs enqueued). Failures:
  `BF-claude-per-cell-home-rows-tripled-the-section-001` (HIGH — arity-based cross-list rule),
  `BF-claude-per-view-walks-over-shared-content-001` (HIGH — the three per-view walks that
  silently assumed view==content). Prevention rules:
  `PR-claude-audit-every-per-view-walk-when-views-exceed-one-001` (grep every view-keyed walk
  over per-content state when a migration makes view_count>1 reachable),
  `PR-claude-nav-home-entry-is-the-container-not-the-cell-001` (home entry = the container's
  row; markers from cross-container hosting, never arity).
- **Ticket:** forge #398 → done; local doc → `tickets/closed/` (status: closed).
- **Archive:** the spec/notes pair → `docs/planning/pipeline/completed/`.
