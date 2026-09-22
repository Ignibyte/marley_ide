# Editors onto the ContentId registry — one Buffer, many views — Notes

- **Forge ticket:** #397 `19d1d1cd-9326-4595-b757-e9912029cf65` (feature, sprint "M28 — The Registry Payoff" `6558258f-a0d2-45fb-b601-cd5329a1cd77`)
- **AAR:** 97e858fb-c0b2-42fa-a181-6cae8b50d063 (opened at /work promotion, 2026-08-04)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-397-editors-onto-registry.md
- **Pipeline spec:** 397-editors-onto-registry.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request / provenance:** the #388 train slice-3 (pane-composition-model.md Q5 — "Editors onto the
  registry… kills the #259 two-Buffers dup… The first visible payoff"), queued into M28 as the
  registry's first user-visible cash-out after #394 (shipped, slice-1) and #396 (open, slice-2 —
  the HARD DEP). #259's CHANGELOG entry names the debt verbatim: "the same file open in the editor
  tab and an editable pane are two independent `Buffer`s (edits do not mirror live); the shipped
  #275 external-change machinery is the net… True shared-buffer multi-view is a named follow-up."
  This is that follow-up.
- **Classification / tier:** feature, **L** (the ticket's own sizing; the Q3 ripple table sizes the
  editor-seam rows M/L). One slice, REACT-FIRST applicable (policy 2026-08-04).
- **Forge recall (§18.3):** live `knowledge-search` run ("buffer ownership editor views undo";
  "persistence codec back-compat") — hits are id-opaque without an AAR to log against
  (pending-promotion), so names were ground-truthed from the standing record:
  `AD-claude-pane-content-id-registry-001` + `AD-claude-one-instance-many-views-001` (the #388
  ADs this slice executes); `PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-001`
  (content_registry.rs:6-7);
  `PR-claude-version-only-memo-unsound-across-buffer-identity-change-001` +
  `BF-claude-lines-memo-version-only-across-reload-001` (356 notes — every per-view memo keys
  `(nonce, version)`; reload re-mints the nonce, so ALL views' memos invalidate — D2);
  `PR-claude-transient-overlay-dismiss-poll-live-editor-identity-001` (overlays poll live editor
  identity — inspect must confirm identity polling survives the id indirection);
  `PR-claude-new-persist-setting-needs-round-trip-test-001` +
  `PR-claude-persist-verify-trigger-not-just-codec-001` (REQ-006's byte-identity + trigger
  verification); `BF-lsp-didopen-carries-empty-text-ready-race-001` (the drain-first invariant the
  LSP re-source must not disturb); `BF-claude-skip-detach-pump-fleet-live-001` (floors — the
  skip-detach re-check on app.rs shim turf). `docs-search "editor buffer"` returned cross-project
  noise (oathstar) — no additional owner.
- **Discovery (the sweep's load-bearing evidence — file:line, read 2026-08-04, PRE-#396 tree):**
  - **The shipped registry (#394):** content_registry.rs — `ContentId` never-serialized doc
    (:20-22), `insert` acquires view 1 (:56-61), `acquire_view` (:66-70), `release_view` returns
    owned content exactly on last (:78-86), cov/MSI 100 suite in-module; `pub use` at lib.rs:105
    (the #371 idiom). #396 (open) adds `Content` + terminal ownership — this slice lands on THAT
    tree; every app.rs line here shifts and Phase 2 re-verifies.
  - **The instance/view split surface:** editor_surface.rs `OpenFile` :32-75 — instance-half
    (`buffer`, `saved_version`, `nonce` :44-48, `disk`/`conflict`/`armed_at`/`conflict_observed`
    :60-74) vs view-half (`view: CodeViewState`, `scroll_px` :49-53, `scroll_x` :54-59, `marked`);
    `EditorSurface { files, active }` :122-125 (a MULTI-file tab surface — registry entries are
    per-FILE, so the tab becomes a list of views); dedupe-by-path `open` :166-176; `close` last-file
    → drop-the-tab :181-197; `reload_active` epoch :431-452 (fresh buffer + nonce re-mint + memo
    reset — the identity-change rule's home); `open_docs` :462-466 (#309); `same_file` canonical
    :22-27 (#322 — the dedupe key); nonce-keyed park :300-327 (#273/#336 — the park idiom
    D-OPEN-SELECTION-HOME generalizes; NOTE: twins share a nonce post-collapse → park must re-key
    per-view). code_view.rs `CodeViewState` :581-591 + `sync_lines_from` version memo :610 (#356).
  - **The #297 wrinkle (found, drives a D-OPEN):** `Buffer` OWNS the `SelectionSet`
    (buffer.rs:36-41; editor_surface.rs:34-38 "two sources of truth… now there is one owner") —
    but Q2 puts selection per-VIEW. Undo records `sel_before/sel_after` full sets (undo.rs #297).
    Candidates park-on-focus-switch (recommended, zero marley_editor surgery) vs move-out-of-Buffer
    — D-OPEN-SELECTION-HOME.
  - **The four births:** `Tab::code` tabs.rs:163-164 (→ `code_surface`); split-create
    `split_file_pane` app.rs:5294-5311 — `EditorSurface::new(cv, &text)` :5307 seeded from a fresh
    DISK read via `load_code_view_state` :5300, so splitting a DIRTY tab shows stale text (the
    visible half of #259 — becomes resolve-to-live-buffer); split-restore app.rs:1954-1978 (:1972);
    `from_files` session-restore app.rs:2093-2095. Restore of tab+split of one path today reads the
    file twice and mints two buffers — post-slice: resolve-or-insert, one entry, two views.
  - **The accessor spine:** `active_editor` app.rs:9515-9525 (focus-aware #259; ~75 through-sites
    kept then, kept again now — only the internal resolution moves); `active_editor_mut` :14050;
    `close_editor_file` :7501; tabs.rs `TabContent::{Terminal(PaneGrid), Cockpit, CodeView
    (EditorSurface)}` :24-31, `tab.editor()` :224-229; workspace.rs `PaneContent::CodeView
    (EditorSurface)` :317-329, `kind()` :361-366, `focused_editable_surface` :639-650.
  - **The #275 self-conflict mechanics (what dies):** per-copy snapshot state; save path
    app.rs:8282-8346 — arm/stale-ack/racing-write logic + `did_save` :8320 + snapshot update
    :8332-8344; chokes: activate :5283, focus-edge :14975, ⌘S; decision table extchange.rs:47-67
    (UNCHANGED — only state residence moves). #259 CHANGELOG: "saving either copy trips the other's
    disk-stat conflict banner" — the arm REQ-004 re-drives to prove gone.
  - **The LSP arm (what REQ-005 pins):** pump collect app.rs:1734-1754 gathers
    `tabs().iter().filter_map(|tab| tab.editor())` — per-TAB only: split panes are LSP-DARK today
    (the #259 follow-up gap), which is the ONLY reason the twin pair doesn't already double-feed
    reconcile; `lsp_host::reconcile` :177-236 keys docs by canonical path — a twin-fed input would
    ping-pong full-text didChanges (:207-211) forever; drain-first invariant app.rs:1714-1723
    (`BF-lsp-didopen-carries-empty-text-ready-race-001`) must survive the re-source; test hook
    `open_doc_count` lsp_host.rs:499-501 (REQ-005's probe); `ensure_lsp_host_for_open_docs` :5085.
    Registry-derived source ⇒ dedupe structural + split-only files JOIN the sync set (deliberate
    small gain — D-OPEN-LSP-SINGLE-REG confirms).
  - **Persistence (what stays byte-identical):** grid leaf `c=<path>` grid_layout.rs:84/:179-188;
    editor-tab `V=<paths>` :410-415 (`TabLayout::Code{paths, active}`); #163 framing guard :248;
    round-trip suites #163/#205/#177 + the #258 `c=` pins (:557-567). No id anywhere near a codec.
  - **Behavior maps (sweep leg 1):** zed 03-editor-multibuffer.md:121 (shared buffer behind splits),
    07-workspace-panes-palette.md:157 (`clone_on_split` — the view duplicates, not the document),
    02-text-buffer-anchors.md:325-334 (undo History on the buffer). warp_architecture: checked, no
    editor analog — PASS. **Published (leg 2):** LSP spec one-doc-per-uri (didOpen must not repeat
    without close) — REQ-005's law. **Permissive deps (leg 3):** ropey-1.6.1 rope.rs:81-83
    `Clone` + `Arc<Node>` — persistent clone-sharing = cheap DIVERGING snapshots, NOT shared
    mutation ⇒ does not change the one-Buffer design (answered the prompt's key question); gpui
    `Entity<T>` — the #388 NO re-confirmed (RootView still monolithic, `cx.new` once; #394 exists
    precisely to keep the model gpui-free).
  - **React POC surface:** `PaneItem { type: 'terminal'|'file', name }` App.tsx:20-23;
    `AppState.openFiles/activeFile` :44-46; EditorView.tsx + views/SplitTerminalView.tsx per the
    MARLEY-PARITY.md port map; § Shared vocabulary pins ContentId↔PaneItem; dev loop
    `pnpm --filter @workspace/marley-ide run dev` → localhost:5173.
- **Decisions (locked):** D1 one-instance-per-file in Marley's own registry (both ADs; HARD DEP
  #396's pattern); D2 the Q2 per-instance/per-view split (memos keyed `(nonce, version)`); D3 undo
  per-instance (open-Q3 resolved as recommended — one history, either view's ⌘Z; Zed behavior
  match); D4 ids never serialize, restore rebuilds, round-trips byte-identical; D5 views carry
  `(ContentId, ContentKind)` (open-Q1 as recommended, #396-shipped pattern); D6 #275 external
  detection survives per-instance, self-conflict dies structurally, `extchange` table untouched.
- **Open design questions (Phase 2):** D-OPEN-SELECTION-HOME (Buffer-owned `SelectionSet` vs
  per-view — recommended: focused-view-owns + park-on-switch, the #273/#336 idiom; also re-key
  scroll parking off the now-shared nonce, and the unfocused twin's caret render);
  D-OPEN-LSP-SINGLE-REG (registry-derived open-doc set — recommended; confirm split-only files
  joining sync, keep drain-first); D-OPEN-SAVE-DIRTY-CONVERGENCE (one saved_version/snapshot/arm —
  pin the accessor moves + unfocused-twin presentation); D-OPEN-SECOND-VIEW-ON-CLOSE (the full
  close-path inventory on the post-#396 tree; last-close-of-dirty stays silent-discard);
  D-OPEN-DEDUPE-SCOPE (per-workspace-root + `same_file` canonical equality; cross-project stays two
  instances guarded by #275 as today).
- **EARS drafted (8):** REQ-001 shared edits + dirty-split-shows-live (driven); REQ-002 one undo
  history across views; REQ-003 one dirty ●/save clears everywhere; REQ-004 self-conflict gone,
  external detection intact; REQ-005 LSP single registration (`open_doc_count` probe); REQ-006
  persistence byte-identity + one-entry-two-views rebuild; REQ-007 close semantics
  (survivor keeps instance; last release exactly once); REQ-008 the React-first parity pair.
- **Forge ids:** ticket #397 `19d1d1cd-9326-4595-b757-e9912029cf65`; sprint
  `6558258f-a0d2-45fb-b601-cd5329a1cd77` ("M28 — The Registry Payoff"); pipeline
  `b92b8f6d-18fe-49a6-8db1-8ba6819f7c70`; AAR pending-promotion (mint at /work). Depends-on: #396
  (HARD) — verify shipped before promotion; every app.rs line ref re-verified against the
  post-#396 tree at Phase 2 entry.

## Phase 2 — Design
_Entered 2026-08-04 (auto run). Framework drafted first; line refs + open-D resolutions filled from
the post-#396 tree sweep (three parallel Explore scouts over app.rs / the editor model layer /
containers+LSP+persistence+POC)._

### Architecture (the framework, pre-sweep)
- **`Content::Editor` gains its payload:** `Content::Editor(Box<EditorInstance>)` — the declared
  arm ships its migration exactly as content.rs:24 promised. `EditorInstance` (new struct, home
  `editor_surface.rs` — the editor model vocabulary's file) = the `OpenFile` instance-half: `path`,
  `root` (the workspace root at open — D-OPEN-DEDUPE-SCOPE's per-root key), `buffer` (which per
  #297 already owns `SelectionSet` + `UndoHistory` — D3 falls out), `saved_version`, `nonce`,
  the #275 `disk`/`conflict`/`armed_at`/`conflict_observed` block. content.rs adds
  `Content::editor(instance)` ctor + `as_editor()/as_editor_mut()` (the #396 accessor template
  verbatim). The two content.rs tests using `Content::Editor` as the payloadless specimen re-home
  onto another declared kind (`Content::Git`).
- **`ContentRegistry::iter()` returns:** deleted at #396 for zero-callers/cov; `resolve_open`'s
  by-path scan is the real read-only consumer that restores it (with its own in-lib test — the
  gate:4 integration-lane blind spot rule from #396's AAR says the test must be in-lib).
- **The `resolve_open` seam (one seam, lazy construct):**
  `resolve_open(reg, root, path, make: impl FnOnce() -> Option<EditorInstance>) -> Option<ResolvedOpen { id, existing }>`
  — scan `reg.iter()` for an Editor instance with `root` match + `same_file` path equality (#322);
  hit → `acquire_view` + id WITHOUT calling `make` (split-of-dirty-tab stops re-reading disk — the
  visible payoff, REQ-001); miss → `make()` (the disk read lives in the caller's closure, honoring
  today's read-error posture) → `insert`. Home: content.rs (operates on `ContentRegistry<Content>`;
  cov/MSI 100 with closure-driven units).
- **Release is plain-drop:** one helper `release_editor_view(reg, id)` (content.rs, unit-tested)
  documents the decision once — a Buffer drop is memory-free-cheap, so the last release drops
  INLINE (no PTY-style reaper analog; the `#[must_use]` is satisfied by the helper's explicit
  drop). Every editor close path routes through it exactly once per view.
- **Views:** `OpenFile` KEEPS its name and its surface home (`EditorSurface { files, active }`
  stays the view-side container in BOTH homes — tab `TabContent::CodeView` and split cell
  `PaneContent::CodeView`) but its fields become the view-half only: `id: ContentId`, `view:
  CodeViewState` (+ lines memo), `scroll_px`, `scroll_x`, IME/`marked`-adjacent per-view bits, and
  the parked selection (D-OPEN-SELECTION-HOME (a)). Accessor spine: resolution composes the split
  borrow (registry + surface are disjoint RootView fields) in ONE place; exact return shape pinned
  after the call-site census lands.
- **LSP:** the pump's open-doc collect re-sources from the registry's Editor entries filtered by
  the workspace's root (instances carry `root`) — one `(path, buffer, version)` per ContentId;
  split-only files JOIN the sync set (the deliberate gain); the drain-first block's position and
  ordering are untouched.
- **Persistence:** codecs byte-untouched; both restore arms route their mounts through
  `resolve_open`, so tab+split of one path rebuilds one entry/two views and reads disk once.

### Post-#396 tree sweep — app.rs (scout 1, 2026-08-04; file now 20,132 lines)
- **Births:** `split_file_pane` :5359-5376 (`load_code_view_state` :5282 → `EditorSurface::new`
  :5372, behind the #392 `try_workspace_mut()` guard); split-restore arm :2005-2029
  (`EditorSurface::new` :2023) inside the restore closure :1598-1601 which ALREADY takes
  `content: &mut ContentRegistry<Content>` (the terminal arm :1605-1614 is the pattern; the
  registry is built at :1659); `from_files` tab restore :2110-2153 (`EditorSurface::from_files`
  :2150 → `Tab::code_surface` tabs.rs:169; stat-before-read ordering :2116-2119 preserved).
- **Accessors:** `active_editor` :9667-9677 / `active_editor_mut` :14250-14264 — both return
  `&(mut) EditorSurface` (tab editor first, else grid `focused_editable_surface`), both
  `mutants::skip` shims. **Sites call SURFACE methods** (`active_file`, `active_buffer`,
  `active_conflict`, `arm_active_save`, …) — the accessor design must keep those call shapes.
- **Save shim** `save_active` :8427-8505 — choke first :8428; snapshot via `active_editor()`
  :8429-8436; the conflict ladder re-resolves `active_editor_mut()` THREE times (:8449/:8461/:8490);
  `did_save` :8478 keyed by `lsp_hosts.get_mut(&active_root)`; racing-write detect :8486.
  Registry design resolves the id ONCE up front.
- **Extchange chokes** (all preserved as-is): open/activate :5348, focus-edge :15215, file-tab ×
  :17207, neighbor-reveal :7669, rail jump :7179, reveal arms :8845/:8854/:8862/:8869, close-pane
  reveal :9154, save :8428. Machinery `check_active_file_external` :8028 / `disk_state` :8007 /
  `reload_active_from_disk` :8062.
- **LSP pump:** drain-first :1771-1776 (comment :1760-1770), collect :1781-1801 (per-TAB
  `tabs().iter().filter_map(|tab| tab.editor())` — splits LSP-dark), ensure-host :5150-5154
  (:1815-1818 call), reconcile :1819-1823, "no drain between" :1812-1814. **NEW finding: THREE
  open-doc collects** — the pump PLUS references overrides :11409-11416 and search overrides
  :12208-12216, all per-tab today.
- **Close inventory (D-OPEN-SECOND-VIEW-ON-CLOSE ground truth):**
  A `close_editor_file` :7658-7671 (sole caller :17182; drained → `close_tab_at`);
  B ⌘W close-pane arm :9001-9038 (`close_focused` :9013, terminal release :9019-9026 — **a
  CodeView pane's surface drops silently in the returned `PaneState` today**);
  C pane-header × :18310-18328 (same silent drop);
  D `close_tab_at` :7563-7600 (cid collect :7577-7582, `drop(removed)` :7592,
  `release_grid_terminals` :7593 — comment :7590-7591 "closed editor tab's buffers are
  memory-only" is the assumption #397 removes);
  E `close_project_at` :7677-7724 (collect :7702-7712, LSP host drop :7716-7718; callers :16551 +
  :5199);
  F pump dead-pane auto-reap :1625-1642 (terminal-only — editors have no exit-death; unchanged);
  G `set_content` workspace.rs:680 — DEAD (zero non-test callers), `#[must_use]` armed; doc
  contract extends to CodeView, no code path;
  H `open_or_switch_code` tabs.rs:437 — the reuse seam (`surface.open`) that becomes
  resolve-aware.
- **#396 release pattern:** `release_view` sites :1634/:5483/:9023/:18321; canonical
  `release_grid_terminals` :5480-5492 (collect → one off-thread reap). Helpers to extend sit
  :5462-5522 (`focused_terminal(_mut)`, `pane_of_content` — hard-codes `terminal_id`, stays
  terminal-only this slice (#398 generalizes it), `workspace_terminal(_mut)`). Grid seams:
  `PaneState::terminal_id` workspace.rs:386, `editable_surface(_mut)` :406/:415,
  `focused_editable_surface(_mut)` :636/:642, `PaneGrid::close` :561 / `close_focused` :582 /
  `set_content` :680 / `states` :702; `Shell::grids(_mut)` tabs.rs:569/:577.

### Post-#396 tree sweep — model layer (scout 2) + containers/LSP/persist/POC (scout 3)
- **`OpenFile`** editor_surface.rs:32-75 — instance-half: `buffer` :39, `saved_version` :40,
  `nonce` :48, `disk` :63, `conflict` :66, `armed_at` :70, `conflict_observed` :74; view-half:
  `view: CodeViewState` :33, `marked` :43, `scroll_px` :53, `scroll_x` :59. **The path lives in
  `f.view.path`** (CodeViewState.path — code_view.rs:583); there is NO OpenFile.path — identity
  reaches through the view struct (a split-ownership snag the design resolves: instance gains the
  canonical `path`; view keeps its copy as render metadata). `OpenFile { … }` literal exists ONLY
  at editor_surface.rs:92 — the struct is module-private; the only doors are `EditorSurface::new`
  / `from_files` / `open`.
- **`EditorSurface`** :122-125 `{ files: Vec<OpenFile>, active }`. `same_file` :22-27 (canonical,
  raw fallback); **`open` :166-176 dedupes by RAW `==`, not `same_file`** (pre-existing
  inconsistency the id-based dedupe folds away); `close` :181-197 (false = would-drain-last;
  out-of-range = true no-op); `reload_active` :431-452 (fresh buffer :437, caret preserved
  clamped :438, `marked=None` :440, nonce re-mint :441, `lines_version=None` :446, clean
  :447-451); park `:293-327` — `park_scroll(nonce, px)` searches rows by nonce (driven by
  `sync_editor_scroll` app.rs:14623-14676 with `scroll_owner: Option<u64>` = a NONCE, looping
  TAB surfaces only — split-owned scroll already misses today, pre-existing); `open_docs`
  :462-466 — **today yields the same path twice with two different `&Buffer`s for a tab+split
  twin** (the LSP-side symptom); `has_path`/`buffer_for_path_mut` :471-481 (same_file);
  dirty spine `is_dirty` :107-109 (`version != saved_version`), `mark_saved` :111-113,
  `active_*` accessors :344-362 (`file_dirty_flags` :360 feeds the tab-strip ●s).
- **`Buffer`** (crate `marley_editor`, crates/editor) buffer.rs:36-46 `{ rope, selection:
  SelectionSet, version, undo: UndoHistory, deltas }` — **owns BOTH selection and undo**; NO
  Rc/RefCell anywhere in the crate → `Send + Sync` (registry residency safe). Undo:
  `begin/end_undo_group(sel_before/after)` :731-739; `undo()/redo() -> Option<HistoryMove
  { selections }>` :745-789 — `set_selection` inside :765; `set_selection` :810-824 **clamps +
  re-canonicalizes** (restoring a stale parked set is safe by construction). `UndoGroup`
  undo.rs:30-50 (`sel_before/after: Option<SelectionSet>`). Nonce mint: `next_nonce()`
  :79-82 (AtomicU64); mint sites ONLY :97 (`OpenFile::new`) + :441 (reload).
- **Memo census (the twin-dedupe payoff):** app.rs `(nonce, version)` keys —
  `file_symbols_memo` :267, `fold_proj_cache` :279, `selection_ladder_at` :327,
  `sticky_headers` :331, `bracket_match_key` :341, inlay hints :345, stale-response gate
  :14965, highlight memos :3902/:4037/:4146/:4220/:4530/:4636/:11668/:11805/:12715/:14540/:14692.
  Nonce moving to the INSTANCE (shared by twins) makes every one dedupe for free — today a twin
  pair parses/highlights twice. **code_view.rs `sync_lines_from` :610-616 keys on version ONLY**
  (nonce handled out-of-band by reload's manual null — exactly the shape
  `PR-claude-version-only-memo-unsound-across-buffer-identity-change-001` warns about; the design
  re-keys it `(nonce, version)` and deletes the out-of-band null).
- **Containers:** `TabContent` tabs.rs:24-32 UNCHANGED by #396 (Terminal arm still holds the
  whole `PaneGrid` — the migration hit `PaneContent` one level down); `Tab::code` :163-165 →
  `code_surface` :169-175; `tab.editor()` :224-229; `open_or_switch_code` :437-448 (prod caller
  of `Tab::code` at :446); `Project::close_tab` :313-320 pure index surgery returning the Tab.
  `PaneContent` workspace.rs:341-355 — `Terminal(ContentId)` :345 vs **`CodeView(EditorSurface)`
  :352 still payload-in-tree**; `PaneState::terminal_id` :386; `editable_surface(_mut)`
  :406/:415; `PaneGrid::close` :561-580 (returns the PaneState; doc :553-560 pins the release
  contract); `close_focused` :582; `open_pane` :543-550 (how CodeView panes enter);
  `split_focused` :532-539 (ContentId, terminal-only); `set_content` :680 (dead, must-use armed).
- **Persistence:** `c=` encode :84 / decode :182-190; `V=` encode :269-287 + write :360-367 /
  decode :292-303 + read :412-415; framing guard :248-252; #258 pins :560-610. No id near any
  codec; the design keeps all of it byte-untouched.
- **LSP:** `reconcile(&[(PathBuf, String, BufferVersion)])` lsp_host.rs:177-232 (version arm
  :207-211 — a twin-fed input ping-pongs full-text didChanges), `needs_text` :242-250,
  `open_doc_count` :498-501 (#[cfg(test)]). lib.rs: all mods private, flat `pub use` (content
  :108, content_registry :105, tabs :119, workspace :122-125); grid_layout + lsp_host NOT
  re-exported.
- **POC (marley-web, artifacts/marley-ide):** `PaneItem { type: 'terminal'|'file', name }`
  App.tsx:20-23; `AppState.openFiles: string[]` + `activeFile: string` :45-47 (NAMES, no
  documents anywhere); single `useState` + prop drilling; debounced `saveState` :169-181.
  EditorView.tsx: read-only highlighted divs over module-frozen `FILE_CONTENTS` :32-76 (2
  files), file-tab bar :261-286, gutter, caret-on-click :208-223 — **no editing exists**.
  SplitTerminalView.tsx: split cells are hardcoded TerminalViews (pane2/3 empty :34-54);
  `state.panes`/`PaneItem` never read there. MARLEY-PARITY.md: shared-vocab paragraph :591-593
  (ContentId ↔ PaneItem); zone-A rows :28/:30; port-map rows :563/:567; rules :595-600.

### The five D-OPEN resolutions (Phase 2, evidence above)
- **D-OPEN-SELECTION-HOME → (a), as recommended.** Buffer keeps owning `SelectionSet`; the
  buffer's live set IS the focused view's cursors. Each `OpenFile` row gains
  `parked_selection: Option<SelectionSet>` + a per-row **`view_key: u64`** (minted from the same
  atomic; the park identity). A per-frame choke at render top (beside the focus-edge check
  :15211-15217 — the shipped precedent for render-time model sync) compares the resolved active
  view's `(view_key, id)` against `last_editor_view`; on a SAME-INSTANCE view switch it parks the
  old row's set (from the buffer) + clears the old row's `marked` (composition ends on focus-out,
  the #267 rule extended) and restores the new row's parked set (or caret-0) through
  `set_selection` (which clamps — stale-after-edits is safe). Unfocused twin renders NO caret
  (today's unfocused posture, unchanged). Undo lands its recorded sets into the buffer (=
  whichever view invokes it) — the D3 behavior, zero marley_editor surgery.
  **Scroll park re-key:** `park_scroll(_x)` re-keys nonce → `view_key`; `scroll_owner` becomes a
  view_key; the park loop extends to grid editable surfaces (today's tab-only loop already
  drops split-owned parks — a pre-existing miss this fixes; recorded as a deliberate tiny gain).
  Reload keeps standing row scroll exactly as today (reload never touched `scroll_px`; render
  clamps).
  **Lines memo re-key:** `CodeViewState.lines_version: Option<(u64, BufferVersion)>` — keyed
  `(instance nonce, version)` per the standing rule; `sync_lines_from(nonce, buffer, …)`;
  reload's out-of-band `lines_version = None` nulls are DELETED (the key does the work, and it
  now invalidates EVERY view of the instance, which the manual null never could).
- **D-OPEN-LSP-SINGLE-REG → registry-derived, all THREE collects.** The sweep found the pump
  collect is not alone: references overrides :11409-11416 and search overrides :12208-12216 are
  the same per-tab walk. All three re-source from the registry's Editor entries filtered by
  `instance.root == active_root`, **sorted by path** (HashMap order would be a nondeterministic
  ordering surface — the #396 AAR rule applied preemptively), one `(path, text-if-needed,
  version)` per ContentId. Split-only files JOIN all three sets (deliberate small gain, recorded:
  today they are LSP-dark and override-dark). Drain-first block position + "no drain between"
  ordering untouched. `did_save`/diagnostics keying unchanged (path-keyed).
- **D-OPEN-SAVE-DIRTY-CONVERGENCE → one snapshot per instance; id resolved once.** `save_active`
  resolves the active view's ContentId ONCE at the top, then every step (snapshot read, arm,
  disarm, mark-saved, disk update, conflict set) addresses the registry by that id — the three
  re-resolutions of `active_editor_mut()` (:8449/:8461/:8490) collapse to id-addressed access
  (no same-surface assumption left to violate). `is_dirty`/`file_dirty_flags` read the instance →
  both views' ● flip together. A warning armed in one view is armed in both (the license is the
  FILE's). Unfocused twin during save-under-conflict shows the same banner state (banner derives
  from instance conflict state — automatic).
- **D-OPEN-SECOND-VIEW-ON-CLOSE → the inventory is A-H (scout 1) and closes route one release
  per view row.** `EditorSurface::close` reshapes to `CloseOutcome { Removed(ContentId),
  WouldDrain, NoOp }` (pure, cov'd); app shims release on `Removed`; `WouldDrain` →
  `close_tab_at` which drains the surface (all row ids released). ⌘W arm (B) + pane-header ×
  (C) extend their returned-`PaneState` handling: an editable surface's row ids are collected
  and released (today they drop silently — correct pre-#397, a leak after it). `close_tab_at`
  (D) + `close_project_at` (E) collect editor ids from the tab's own surface AND grid CodeView
  panes alongside the existing terminal cids. Editor releases are **plain inline drops** via
  `release_editor_views` (a Buffer drop is memory-free-cheap — no PTY reaper analog; the
  `#[must_use]` satisfied explicitly in the helper). Dead-pane auto-reap (F) stays
  terminal-only (editors have no exit-death). `set_content` (G) stays dead; its must-use doc
  extends to CodeView. Last-close-of-dirty: silent discard (today's posture, unchanged, Out).
- **D-OPEN-DEDUPE-SCOPE → (root, same_file), confirmed.** `EditorInstance.root` = the workspace
  root at open; `resolve_open` scans for `root` match + `same_file` path equality. Cross-project
  opens stay two instances (#275 guards that edge via disk, as today). Surface-internal dedupe
  (`open`) compares resolved **ContentId equality** — which folds the raw-`==` vs `same_file`
  inconsistency away.

### Design — exact shapes
- **`EditorInstance`** (NEW, editor_surface.rs, pure, cov/MSI 100): `{ root: PathBuf, path:
  PathBuf, buffer: Buffer, saved_version: BufferVersion, nonce: u64, disk, conflict, armed_at,
  conflict_observed }` + `new(root, path, text, disk)` (mints nonce, saved = version),
  `is_dirty`, `mark_saved`, `reload(text, disk)` (fresh buffer, caret preserved-clamped, nonce
  re-mint, clean state — today's `reload_active` :431-452 semantics moved), the #275
  getters/setters. `Content::Editor(Box<EditorInstance>)` + `Content::editor(instance)` ctor +
  `as_editor()/as_editor_mut()` (content.rs, the #396 accessor template; the payloadless-specimen
  tests re-home onto `Content::Git`).
- **`OpenFile` (the view row):** `{ id: ContentId, view_key: u64, view: CodeViewState, marked,
  scroll_px, scroll_x, parked_selection: Option<SelectionSet> }`. Views are built LAZY (empty
  lines, `lines_version: None` — `sync_lines_from` does the one true fill from the shared
  buffer; kills the which-text-seeds-the-view question; `resync_active_view` already covers the
  post-restore first frame). `EditorSurface` keeps `{ files, active }` + ONLY view-side logic
  (rows, active index, `activate`, `CloseOutcome` close, park by view_key, `clear_marked`,
  per-row accessors); every instance-touching method moves to `EditorInstance` or the app-side
  join.
- **The app-side join (masked shims, the #396 pattern):** `active_editor()/active_editor_mut()`
  keep resolving the SURFACE (tab first, grid fallback — unchanged); NEW siblings
  `active_editor_instance(&self) -> Option<&EditorInstance>` /
  `active_editor_instance_mut(&mut self) -> Option<&mut EditorInstance>` resolve
  surface.active-row.id through `self.content` (split borrow: `self.shell` vs `self.content`
  are disjoint RootView fields — the #396-proven shape). The ~75 through-sites re-point
  mechanically: view-side calls stay on the surface; instance-side calls move to the instance
  accessor. `resolve_open` (content.rs): `resolve_open(reg, root, path, make: impl FnOnce() ->
  Option<EditorInstance>) -> Option<ResolvedOpen { id, existing }>` — hit → `acquire_view`
  WITHOUT calling `make` (split-of-dirty stops re-reading disk); miss → `make()?` → `insert`.
  `ContentRegistry::iter()` restored (the read-only scan consumer; in-lib test — the gate:4
  in-lib rule). `release_editor_views(&mut self, ids)` app-side inline-drop helper.
- **Births (all four → resolve):** `open_file_in_viewer`/`open_or_switch_code` (H) — app
  resolves path→id, pushes an id-bearing row into the editor tab's surface (tabs.rs stays
  registry-free; `Tab::code` DELETED — prod caller reshapes, test callers re-point to
  `code_surface` with `ContentId::test(n)` rows); `split_file_pane` — resolve (live buffer on
  hit — THE payoff) → `PaneContent::CodeView(EditorSurface::for_view(id, lazy-cv))`;
  split-restore arm + `from_files` restore — both route each mount through `resolve_open` with
  the disk read inside the closure (second mount of a path acquires, never re-reads; the
  readable-filter posture preserved: closure `None` → mount skipped; stat-before-read ordering
  :2116-2119 preserved inside the closure).
- **Persistence collects:** `persist_grid`'s `c=` path map and the `V=` tab paths read
  `row.view.path` (unchanged values) — codecs byte-identical; ids never serialize.

### File manifest
**marley-web FIRST (React-first — build + visually verify, then port):**
- `artifacts/marley-ide/src/App.tsx` — `AppState.docs: Record<string, { text: string; dirty:
  boolean }>` seeded from the (moved) file contents; defaults-merge covers missing-field.
- `artifacts/marley-ide/src/components/EditorView.tsx` — body becomes EDITABLE bound to
  `docs[activeFile]` (chrome frozen: tab bar/gutter/geometry untouched); dirty ● on the file tab
  from `docs[…].dirty`.
- `artifacts/marley-ide/src/components/views/SplitTerminalView.tsx` — a split cell renders a
  file view of the SAME doc when the pane item is `{ type: 'file' }` — typing in either
  propagates live; one ● in both places.
- (`Workspace.tsx` only if pane wiring needs a prop pass-through.)
**Rust (after the POC is visually approved):**
- `crates/marley_app/src/editor_surface.rs` — `EditorInstance`; `OpenFile` reshape;
  `EditorSurface` method split; `CloseOutcome`; view_key park; tests.
- `crates/marley_app/src/code_view.rs` — `lines_version: Option<(u64, BufferVersion)>`;
  `sync_lines_from(nonce, …)`; tests.
- `crates/marley_app/src/content.rs` — `Editor(Box<EditorInstance>)`; ctor + accessors;
  `resolve_open` + `ResolvedOpen`; specimen-test re-home; tests.
- `crates/marley_app/src/content_registry.rs` — restore `iter()`; test.
- `crates/marley_app/src/tabs.rs` — `Tab::code` deleted; `open_or_switch_code` reshaped
  id-based; tests re-pointed.
- `crates/marley_app/src/workspace.rs` — `editable_surface`/`set_content`/`close` doc-contract
  extensions (no enum change — `CodeView(EditorSurface)` stays: the tree owns VIEWS); tests.
- `crates/marley_app/src/app.rs` — the sweep: 4 births, accessor joins, save shim id-once, 3
  LSP/override collects, close inventory B/C/D/E, selection/scroll sync chokes, `scroll_owner`
  → view_key, `last_editor_view` field.
- `crates/marley_app/src/headless_drive.rs` — seeded-restore drive extended: editor tab+split
  rebuild (len 1 / views 2 / kind Editor), shared-edit assert, close-path registry-len asserts.
- `crates/marley_app/tests/integration.rs` — only if a live-PTY-adjacent editor lane is needed
  (expect no).

### Regression test plan (one row per REQ + floors)
| REQ | Tests |
|---|---|
| REQ-001 | units: `resolve_open` hit-no-make/miss-make/make-None (content.rs); surface open-by-id dedupe; headless drive: open → edit (dirty) → split same path → split shows the EDITED text, registry len 1 / views 2; type via each view's path → both render identical |
| REQ-002 | unit on one instance + two view rows: edit-A, edit-B, undo×2 rewinds in reverse order, redo×2 restores (Buffer-level history shared by construction); drive: ⌘Z from the tab rewinds the split's edit first |
| REQ-003 | unit: `is_dirty` through both view resolutions flips together across edit→save (`mark_saved` on the instance); drive: ● in both, ⌘S from one clears both |
| REQ-004 | unit: exactly one disk snapshot per instance updated by either view's save; extchange decision-table suite green byte-unchanged; drive of the #259 scenario (save in one → NO banner in the twin) |
| REQ-005 | unit: registry-derived collect is path-sorted + unique per path; `open_doc_count`: tab+split → 1, close one view → 1, close last → 0; drain-first ordering assert unchanged |
| REQ-006 | #163/#205/#177 + #258 round-trip suites green byte-identical; headless restore drive: tab+split of one path → one entry two views, disk read once; inspect: no ContentId near any codec (grep discipline) |
| REQ-007 | units: `CloseOutcome` all three arms; per-close-path registry-len asserts (close-file / ⌘W pane / close-tab / close-project) — survivor keeps buffer+undo+dirty; double-release safety rides the #394 suite |
| REQ-008 | POC: typecheck green + screenshot READ at localhost:5173 (shared edits + one ●) BEFORE Rust; Validate: React↔Marley parity pair captured |
| floors | cov/MSI 100: editor_surface, code_view, content, content_registry, tabs, workspace (all at 100 today — stay); `cargo mutants --list -f` re-run on actual touched files; neighboring `#[mutants::skip]` re-verified |

### Risks / decisions
- The app.rs sweep is #396-sized (~75 sites); strangler order: pure seams compile-green first,
  then app.rs region by region. Split-borrow joins proven by #396.
- Recorded deliberate deltas: (1) split-only files join LSP + references/search override sets;
  (2) collects become path-sorted (deterministic vs today's tab order); (3) twin memo dedupe
  (highlight/fold computed once); (4) split-owned scroll parks now land (tab-only loop miss
  fixed); (5) POC editor body is plain editable text for the demo (chrome frozen; the Rust side
  keeps the real highlighted editor — the POC demos the SHARING semantics, recorded per parity
  contract rules).
- reload-under-twins: nonce re-mint invalidates every view's lines memo via the new key — the
  one behavior to watch at inspect (no manual nulls left).

## Phase 3 — Implement
_Entered 2026-08-04 (auto run)._

### React-first (the demo BUILT + VISUALLY VERIFIED before any Rust)
- **Built:** `src/utils/docShare.ts` (NEW — the shared `Doc` store: `SEED_CONTENTS` moved from
  EditorView, `seedDocs()`, `docText()`, and the pure `applyKey` edit seam both views share);
  `App.tsx` (`AppState.docs: Record<string, Doc>` + seeded default — the ContentId↔PaneItem
  stand-in the parity doc asks for); `EditorView.tsx` (body binds `docText(state.docs, …)`,
  `tabIndex`+`onKeyDown` typing through `applyKey` patching the SHARED doc, per-DOCUMENT dirty ●
  on the file tab, `SyntaxHighlightedLines` exported); `SplitTerminalView.tsx` (NEW local
  `FilePaneView` — a split cell renders a file VIEW of the shared doc when the pane model holds a
  `{ type: 'file' }` item; its caret is the cell's OWN `useState` — per-view state; pane 2 and
  pane 3 both resolve the same item = the one-buffer-two-views demo, no default-state churn).
- **Deviation (better than planned):** the notes' "plain editable text" simplification was NOT
  needed — the editable body keeps the frozen chrome AND the syntax highlight (a focusable code
  div with a keydown handler over the pure `applyKey` seam), so the demo is closer to Marley than
  the design assumed.
- **Typecheck:** `pnpm --filter @workspace/marley-ide run typecheck` — green.
- **Visual verification (screenshots READ, 2026-08-04):** scratchpad `397-captures/`
  `397-demo-initial-split-grid.png` (split-grid: terminal left, TWO `code_syntax.rs` file cells
  right, clean), `397-demo-shared-edit-both-cells.png` (typed `// shared` at line 3 in the TOP
  cell → the BOTTOM cell shows it live at line 3; BOTH cell headers carry the dirty ●),
  `397-demo-editor-tab-same-doc.png` (the editor TAB shows the same line-3 edit + ● on the tab
  row — three views, one document, one flag). localhost:5173, seeded workspace state
  (`splitMode: 'split-grid'`, panes section).

### Rust (per the Phase 2 manifest)
- **editor_surface.rs** — rebuilt around the split: `EditorInstance` (root/path/buffer/
  saved_version/nonce/#275 block; `new(root, path, text, disk)`, `is_dirty`/`mark_saved`,
  `set_conflict` keeps the disarm coupling, `reload` = the epoch with the nonce re-mint and NO
  manual memo nulls); `OpenFile` row = `{ id, view_key, view, marked, scroll_px, scroll_x,
  parked_selection }` (module-private); `EditorSurface` keeps view-side logic only (`for_view`/
  `from_views`/`open_view` id-dedupe/`CloseOutcome` close/`activate`/parks/rows()/view_ids());
  parks re-keyed to `view_key`; `park_selection` also ends the row's composition (the #267
  focus-out rule); **the JOIN types `EditorRef`/`EditorMut`** mirror the whole pre-#397 surface
  API (view calls delegate, instance calls resolve `active_id` through the registry;
  reference-returning `EditorMut` methods consume `self` so one-expression chains stay legal) —
  this is what let the ~75 through-sites keep their call shapes.
- **code_view.rs** — `lines_version: Option<(u64, BufferVersion)>` + `sync_lines_from(nonce, …)`
  per the standing memo rule; tests extended with the nonce-epoch-invalidation case.
- **content.rs** — `Content::Editor(Box<EditorInstance>)` + `editor()` ctor + `as_editor(_mut)`;
  `ResolvedOpen` + `resolve_open(reg, root, path, make)` (hit → acquire WITHOUT calling make;
  miss → make()? → insert; `(root, same_file)` scope); `release_editor_views` (inline drop —
  answers the `#[must_use]` once); payloadless-specimen tests re-homed onto `Git`; new tests:
  editor accessor pair, miss-then-hit never-double-inserts, root scoping + make-None, release
  drops-on-exactly-last.
- **content_registry.rs** — read-only `iter()` restored WITH its consumer + in-lib test.
- **tabs.rs** — `Tab::code(title, id, state)`; `open_or_switch_code(id, state)` (dedupe by id;
  returns whether a row consumed the caller's resolved view — the speculative-release contract);
  tests re-pointed (`ContentId::test`).
- **workspace.rs** — no enum change (`CodeView(EditorSurface)` stays: the tree owns VIEWS);
  `close`/`set_content` doc contracts extended to editor views; tests re-pointed.
- **app.rs** — accessors return the joins (+ `sync_editor_selection_home` runs at the head of
  `active_editor_mut` AND at render top — D-OPEN-SELECTION-HOME (a): park outgoing set on its
  row by `view_key` across tab+grid surfaces, restore incoming parked set clamped, caret-0 first
  focus; raw resolvers `active_editor_view_identity`/`active_editor_surface_mut` avoid
  recursion); all 4 births route `resolve_open` (viewer with speculative-release; split pane —
  the #259 stale-split dies; both restore arms read-once with stat-before-read inside the make
  closure, views seeded LAZY); all THREE open-doc collects (pump + references + search
  overrides) derive from registry instances root-filtered, pump sorted by path; close inventory
  wired: `close_editor_file` (CloseOutcome), ⌘W arm + pane-header × release surface rows,
  `close_tab_at`/`close_project_at` collect editor ids from tab surfaces + grid CodeView panes
  and `release_editor_views` inline; rename applier → `buffer_for_open_path_mut` registry scan
  (`locate_open_file` deleted); `EditorRef::has_path` for the #314 origin check; scroll:
  `scroll_owner` is a view_key now, park loop extended to grid surfaces; accept-completion tail
  reshaped around the consuming `active_buffer_mut`.
- **headless_drive.rs** — drive helpers re-pointed through the joins; the scroll-park close
  drive releases the removed row like the shim.
- **Deviations from design:** (1) views are seeded from the read text at the interactive births
  (harmless — the memo overwrites from the live buffer; restore arms ARE lazy as designed);
  (2) `EditorRef`/`EditorMut` join API instead of per-site re-pointing (same semantics, ~90% less
  churn); (3) split-restore now stats a disk snapshot (was None pre-#397) — recorded delta:
  split-only files gain #275 external tracking, consistent with them joining the LSP set;
  (4, inspect) two projects sharing one ROOT now UNION into the shared host's open set (the old
  per-active-project walk churned didClose/didOpen on every same-root project switch — the union
  is stable and churn-free; instances are shared via resolve_open so no duplicate-uri feed);
  (5, inspect) the rename applier routes through registry instances — split-only files gain
  edit-through-buffer (instead of the closed-file disk write), and a file open under TWO roots
  (nested projects) picks the lexicographically-first root deterministically (was project-index
  order pre-#397; HashMap order was never acceptable — the ordering-surface rule).
- `cargo check --workspace --all-targets` — 0 errors; `cargo fmt` applied.

## Phase 3.5 — Inspect
_Run 2026-08-04 (auto). Five parallel adversarial critics over the diff, one lens each:
acquire/release pairing; LSP document-sync integrity; single-instance integrity (undo/dirty/
conflict/reload); persistence byte-identity + restore rebuild; per-view state bleed +
simplification. Every finding verified in-source by the critic (file:line quoted) and
re-verified here before fixing. All five cores came back SOUND — the ownership split, dedupe,
release accounting, memo re-key, and LSP feed hold; the findings cluster on the NEW
coordination machinery (selection home, scroll choke) and on seams where pre-#397 per-copy
reasoning stopped being valid once state became instance-owned._

### Ledger (19 findings)
| # | Sev | Finding (critic) | Verdict → action |
|---|---|---|---|
| 1 | HIGH | Selection tracker severed by ANY terminal/cockpit hop (`last_editor_view` overwritten with None) — twin edits with the sibling's cursors; return trip lands caret at 1:1; IME marked leaks across the hop (C4+C5, the same everyday-flow bug) | REAL → **redesigned**: tracker STICKY across non-editor frames; park runs on EVERY view change reading the outgoing set from the OUTGOING view's instance; rows BIRTH-SEEDED with a caret-0 park so the restore is total (no ownership bookkeeping); restore also clears the incoming row's stale composition (#267 focus-in symmetric) |
| 2 | HIGH | Split-FIRST open seeds the ONE instance `disk: None`; the later tab open's stat is discarded on the resolve hit → every #275 choke Noop → silent ⌘S clobber, no banner (C4/C5) | REAL — the pre-#397 "per-view None" parity argument died with per-view snapshots → **fixed**: `split_file_pane` stats BEFORE the read (the #275 safe direction) and seeds the instance |
| 3 | MED | Stale IME `marked` survives a hop-refocus → next IME replace misdirects into the shared buffer (C4) | REAL → covered by fix 1's focus-in clear (+ the park's existing focus-out clear) |
| 4 | MED | The scroll-park grid extension was UNREACHABLE dead code — `sync_editor_scroll`'s only call site was gated on the code-tab render arm, so pane-active frames never parked and twins bled viewport through the shared handle (C5) | REAL → **fixed**: the choke moved to render TOP (runs for every active editor view); the grid loop branch is now live; the code-tab arm call removed |
| 5 | MED | Unfocused twin panes rebuilt `cv.lines` O(file) PER KEYSTROKE — and the walk covered ALL projects × tabs incl. unpainted panes (a #397 regression: pre-#397 an unfocused pane's own buffer never advanced) (C5) | REAL → **fixed**: `resync_editable_pane_views` walks the ACTIVE TAB's grid only (unpainted panes resync on their first frame back via the memo); the visible twin's rebuild stays — the price of the live mirror, recorded |
| 6 | MED | `buffer_for_open_path_mut` picked among two-root instances in HashMap order (the ordering-surface rule violated; notes claimed determinism the code lacked) (C2+C5) | REAL → **fixed**: lexicographically-first root via `min_by`; notes now match |
| 7 | MED | PRE-EXISTING (not a #397 regression — verified identical at HEAD): a CleanReload of a never-edited file never reaches the LSP server (fresh buffer restarts at v0 → `synced == version` suppresses the didChange) (C2) | Pre-existing, recorded NOT fixed (§7 discipline); candidate follow-up ticket: key `DocEntry.synced` on `(nonce, version)` — the same identity-epoch rule the render memo adopted |
| 8 | LOW | "Cross-project twins are impossible" comment in `close_project_at` was FALSE (two projects can share a root — the #359 case 25 lines below) and invited a future bulk-drop (C1+C4) | REAL → comment corrected: twins are real and load-bearing for release-per-row |
| 9 | LOW | `close_editor_file`'s WouldDrain arm implicitly assumes the active tab is the code tab (a focused 1-row CodeView PANE would close the whole terminal tab); unreachable today (sole caller = the strip ×) (C1+C4) | REAL-but-unreachable → caller contract pinned in a comment incl. the routing rule for any future caller |
| 10 | LOW | Unrecorded delta: same-root projects now UNION into the shared host's open set (old walk churned didClose/didOpen per switch) (C2) | Improvement → recorded as deviation (4) |
| 11 | LOW | `resolve_open` roots compare RAW `==` while paths are canonical — aliased root spellings stay two instances; undocumented at the site (C4) | Deliberate (D-OPEN-DEDUPE-SCOPE's cross-root posture) → documented at `find_open` |
| 12 | LOW | The interactive hit path still read the whole file + built lines then discarded both (the recorded "stops re-reading disk" payoff only skipped `make`); splitting a dirty file whose disk copy vanished no-op'd (C5) | REAL → **fixed**: `find_open` fast-path in viewer + split — a hit does NO disk IO, mounts a lazy view; split-of-deleted-dirty now works (the live buffer is the truth) |
| 13 | LOW | REQ-006's end-to-end proof missing: no drive seeds a `V=` + `c=` same-path layout and asserts one entry/two views + byte-identical re-persist; the split-restore arm is test-dark end-to-end (C3) | REAL → Phase 4 work item (the validate plan already carried the restore drive; now pinned to this exact shape) |
| 14 | LOW | Dead code minted by the join rebuild: `rows()` (lying doc), `EditorRef::{surface, active_id, active_saved_version, active_save_armed}`, `EditorMut::{as_ref, active_file_mut, active_index, active_id, active_marked, active_selection, active_nonce, active_is_dirty, active_disk, active_conflict, active_save_armed, active_armed_at, active_conflict_observed, resync_active_view}` — two of them faithfully porting API that was ALREADY dead at HEAD (C5 + clippy) | REAL → all deleted (the #396 zero-callers discipline); `active_ime_mut` re-routed through `active_marked_mut` (its doc now true) |
| 15 | LOW | Park loops cloned the `SelectionSet` per PROBED surface; `accept_completion_item` built the identical caret set twice (C5) | REAL → `park_selection` takes `&SelectionSet` (clones on the hit only); `sel_after` built once + cloned |
| 16 | info | Restore hit-path skips the readability re-check (a file deleted between two mounts keeps its second row) (C3) | Deliberately better under the shared-instance model — recorded |
| 17 | info | `resolve_open` restore scan is O(instances × mounts) with 2 canonicalize syscalls per probe (C3) | Accepted — N is the open-file count; revisit only if large-session restore measures slow |
| 18 | info | A `ViewKey(u64)` newtype would make the nonce/view-key split structural (today a mixup is a guaranteed silent MISS via the shared mint — mitigated dynamically) (C5) | Rejected for now — recorded as #398-adjacent hygiene |
| 19 | info | The selection park, scroll park, and close-inventory walks share a ~30-line shape a `for_each_editor_surface_mut` could collapse (C5) | Rejected at this stage (structural churn at inspect; noted for #398, which reshapes these walks anyway) |

### Post-fix verification
- `cargo check --workspace --all-targets` 0 errors; `cargo clippy --workspace --all-targets`
  clean; `RUSTDOCFLAGS="-D warnings" cargo doc` clean; **full suite 1989/1989 green** after all
  fixes.
- Lenses with NO findings at all: none — every critic surfaced something; every core verdict
  was sound (leak pairing exact on all paths; LSP one-uri-per-file structurally unrepresentable;
  undo single-history proven; codecs byte-identical vs HEAD by extraction diff).

## Phase 4 — Validate
_Run 2026-08-04 (auto)._

### Tests added (per the Phase 2 plan + inspect's pins)
- **content.rs** — `find_open_scopes_and_never_acquires` (the read half: resolves by
  (root, same_file), NEVER acquires, root-scoped miss, unopened-path miss, non-editor content
  skipped); the resolve/release suite from implement carries REQ-001/REQ-007's registry floor.
- **editor_surface.rs** — `editor_ref_join_resolves_instance_reads` (every EditorRef accessor
  against a REAL registry: instance reads incl. the #275 block, per-row dirty flags mixing two
  instances, has_path both ways, view delegates); `editor_mut_join_converges_two_views_on_one_
  instance` (REQ-003 + REQ-004 through the joins: writes via the TAB's join read back via the
  PANE's — text, dirty, snapshot, conflict/arm incl. the disarm coupling; the IME span stays
  with the ROW; the consuming buffer accessor; reload-through-the-join re-mints the nonce for
  every view); the park-pair test extended for the birth seed + the #267 park-clear; the
  reshaped close/open/activate/from_views/nonce/park suites from implement.
- **code_view.rs** — the sync memo tests re-keyed, + the nonce-epoch-invalidation case (a new
  nonce alone rebuilds at an equal version — the identity-epoch rule the reload relies on).
- **headless_drive.rs** — `twin_views_share_one_instance_headless` (REQ-001/002/003/007 end to
  end: tab-edit visible in the fresh split — never stale disk; one instance/two views;
  pane-edit visible in the tab; ⌘Z from the tab rewinds the PANE's edit first, redo replays;
  the real ⌘W close releases exactly the pane's view — survivor keeps text+dirty; the last
  close empties the editor registry); `tab_plus_split_restores_one_instance_two_views_headless`
  (REQ-006: boot A builds tab+split via real gestures, boot B rebuilds ONE instance/TWO views —
  both rows the SAME id, buffer from one disk read — and the re-persist is BYTE-IDENTICAL to
  boot A's blob, via the new `shell_blob_for_test`/`persist_grid_for_test` accessors).

### Runs (actual)
- `cargo nextest run --workspace` — **1994 tests run: 1994 passed, 5 skipped** (was 1989
  pre-#397; +5 net new).
- `cargo test --workspace --doc` — 0 failed (no doctests in the workspace target set; ok).
- clippy `--all-targets` clean; `RUSTDOCFLAGS="-D warnings" cargo doc` clean.

### Live-app driven captures (REQ-001/REQ-003 on real pixels) + the REQ-008 parity pair
Bundled `target/Marley.app` (release), drove with `scripts/selftest/drive.swift`. One
environment hurdle: the freshly-bundled app blocked on a macOS TCC prompt ("access files on a
removable volume" — the repo lives on /Volumes/Offload), visible as a windowless process;
clicked Allow (System Events → UserNotificationCenter) and the window appeared — recorded for
the next selftest run on this volume. Captures (scratchpad `397-captures/`, all READ):
- `marley-3-tab-marker.png` — typed `SHARED397 ` into the restored editor tab's `inspect.md`
  (line 2) → the file tab shows the dirty ●.
- `marley-4-palette.png` — ⌘⇧P → "Split Right → File" (the #246 split-mode finder).
- `marley-5-split-shared.png` — **the payoff**: the fresh split pane renders
  `phase: 3.SHARED397 5` — the TAB's unsaved edit LIVE in the split (pre-#397 this pane read
  stale disk), pane header `inspect.md ●` (the one dirty flag through the second view).
- `marley-6-tab-sees-split-edit.png` — typed `TWIN ` in the SPLIT → the EDITOR TAB shows
  `TWIN ---` at line 1 — the reverse direction live; ● everywhere.
Quit without saving (dirty discards silently — disk untouched, worktree clean of app writes).
**Parity verdict:** the React captures (`397-demo-shared-edit-both-cells.png`,
`397-demo-editor-tab-same-doc.png`) and the Marley captures above demonstrate the SAME
interaction — an edit typed in either view of a file appears live in its sibling views with
ONE dirty ● — with zone-A chrome untouched on both sides. The POC's split-grid shows two file
cells side-by-side vs Marley's tab+split (layout differs by design — the pane MODELS differ
until #398/#399); the parity contract's subject here is the one-buffer-many-views interaction,
and it matches 1:1.

### Gate story (three runs — each red fixed at source)
1. **RED (13/15):** gate:4 — content.rs 2 missed lines: my hit-test's `panic!` guard closure
   is never executed BY DESIGN, and a never-run closure body is a forever-uncovered line →
   restructured to an observable POISON make (called once so its line runs; its non-effect
   asserted through registry len + surviving buffer text — a STRONGER test). gate:5 — MSI 97.1%
   (3 missed): `EditorRef::active_index -> 0`, `active_scroll_x -> 0.0`, `EditorMut::
   clear_marked -> ()` — my join test asserted DEFAULTS that equal the mutants' constants (the
   #200 exact-read-back lesson re-learned on delegates) → asserted at both indices / a parked
   non-zero x / set-clear-observe through the surface. Kills verified individually
   (`cargo mutants --re …`: 6 tested, 6 caught).
2. **RED (14/15):** gate:2 — clippy `dropping_copy_types` on the test's `drop(r)` scope
   punctuation (EditorRef is Copy) → deleted; NLL ends the borrows at last use.
3. **GREEN: `GATE GREEN [diff]` — 15 passed, 0 failed** (fmt, clippy, tests, cov 100%,
   **MSI 100.0% (103/103)**, miri, audit/deny/machete/gitleaks/shellcheck, no-suppressions,
   SAST, docs, visual/AX). Receipt written.

### Pre-existing (not in scope)
- The reload-didChange suppression (inspect ledger #7) — pre-#397, recorded; candidate
  follow-up ticket.

### Floors
- `cargo mutants --list` over the touched pure files: content.rs+content_registry.rs 52,
  editor_surface.rs 148, code_view.rs sync 2 — all in-lib and now join-covered; app.rs shims
  carry their `mutants::skip` markers (re-verified: the new fns `sync_editor_selection_home`,
  `active_editor_view_identity`, `active_editor_surface_mut`, `buffer_for_open_path_mut` are
  marked shims over tested seams, per the skip-detach discipline).

## Phase 5 — Complete
_Run 2026-08-04 (auto)._
- **CHANGELOG** — full `[Unreleased]/Changed` entry for TICKET-397 (the one-instance model, the
  visible payoff, the seam inventory, all recorded deltas, the unlocks).
- **Architecture** — `pane-composition-model.md` Q5 slice-3 marked **✅ SHIPPED (M28 #397)**
  with the shipped shapes (instance/view split, selection home, resolve/release seams, joins)
  and the open-Q3 resolution (undo per-instance) recorded.
- **Parity sync** — `marley-web/docs/MARLEY-PARITY.md` shared-vocabulary paragraph extended:
  `AppState.docs`/`docShare.ts` is the POC's instance half mirroring `Content::Editor`
  (ContentId ↔ PaneItem now has a doc-store ↔ registry pairing); the POC demo shipped at
  implement is the living reference. marley-web typecheck green at build time; no port-time
  deviation to back-port (the Rust side followed the approved demo).
- **Knowledge (forge)** — AAR submitted on `97e858fb`; failure-records: the split-first #275
  blind (a per-view default became an instance poison under the ownership move — the inspect
  HIGH), the selection-home hop sever (a None frame broke the park chain — the inspect HIGH);
  prevention rules: re-prove per-copy defaults across ALL birth orders when a field's owner
  changes view→instance; identity/focus trackers must be sticky across None frames (park at
  the choke on every real transition, direct or hopped). Gate lessons folded into the AAR:
  delegate methods die only to non-default asserts (the #200 lesson generalized to joins);
  a BY-DESIGN-never-run closure body is a forever-uncovered line — prove not-called
  OBSERVABLY instead of with a panic guard.
- **Ticket** — TICKET-397 closed (local doc → `tickets/closed/`, forge `ticket-close`).
- **Pipeline archived** — the spec/notes pair → `docs/planning/pipeline/completed/`.
