# 427-multibuffer-core — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-427-multibuffer-core.md
- **Pipeline spec:** 427-multibuffer-core.spec.md

## Phase 1 — Plan

- **Request:** TICKET-427 — multibuffer core, third of the M32 B-c sprint (auto-approved
  directive; shelf `display-map-shelf.md`). Systems: content registry + tabs + the
  search overlay (`marley_app`), the React POC.
- **Classification / tier:** feature, M–L, one shippable slice (read-only snapshot; the
  editable half is #428 by design — the Zed search doc's own phase split).
- **Recall (§18.3):** PR-1323 (shell-codec framing / bare-marker persistence — v1
  simply drops the tab); PR-1691 (`editors_under` is THE root-filtered iteration —
  reuse, never copy); PR-closed-enum-gate (exhaustive matches at every content gate —
  the #403 conversion exists BECAUSE a new kind once slid through silently); F-#553
  (overlay arms need stop_propagation); F-#623/#331 (no naive latches); F-#668
  (close_transient_overlays discipline at open); F-#868 (gen-scalar staleness — no
  dead stale-key fields); AD-305 (half-open runs); the shelf's standing "pure module,
  app.rs is the excluded shim" rule.
- **Discovery (Explore sweep, 2026-08-14, HEAD dd6d557 — the load-bearing facts):**
  - **The #403 checklist** (from the shipped Browser kind): 10 files — content.rs
    (variant + kind + addable + ALL FIVE accessors + a resolve/release door),
    tabs.rs (TabContent + rail_section + accessors + ctor + key_context + title
    const + open_or_switch + rail_rows arm), context_menu.rs (＋ row + SectionAction),
    app.rs (open verb, ＋ dispatch, persist writer arm, restore arm, close-tab +
    close-project releases, render else-if arm at ~:20192, status-bar focus arm),
    grid_layout.rs (TabLayout + tab_survives_shell_entry + writer + reader),
    status_bar.rs, lib.rs (a pure model module), headless_drive.rs (lifecycle +
    restore drives) + a `open_x_tab_for_test` hook. Plus (new for us): keymap/palette.
  - **Registry lifecycle fork** (AD-registry-lifecycle-fork): Editor=path-dedupe,
    Cockpit=pinned singleton, Browser=kind-singleton-dropped. The multibuffer has NO
    dedupe key → always-insert, dropped-on-last-close; mind the
    one-acquired-view-in-both-branches contract (content.rs:401-412).
  - **Search state**: `SearchFile.path` is canonical-absolute (keys the registry
    directly); `LineMatch{row,col,len,preview}` char-domain; previews are the WHOLE
    match line (≤500 chars) but NO context lines exist — excerpts re-source text.
  - **Two-source text rule** (the search worker's own override assembly, quoted in the
    sweep): `editors_under` live text wins; else disk via the `load_code_view_state`
    guard chain (canonical_under_root → stat → 2MB cap → binary sniff → flash).
    Canonical-root spelling is a documented past bug (#319 F2).
  - **Highlighting**: the hover code-fence path (`highlight_ranges` per loose line →
    StyledText, app.rs:16596-16644) is the excerpt-row precedent; `highlight_lines`
    (tree-sitter) parses WHOLE sources — per-result-file parses on the render thread
    are the recorded deferral.
  - **Headers**: the search overlay's own last-file change-detection group header
    (app.rs:16413-16444, `rel_under_root` + count + `+`) is the idiom to reuse.
  - **Render**: the editor's uniform_list is the repo's ONLY one; overlays window
    manually via `popup_window`; the viewer windows via `visible_range`. Either shape
    works if the row model stays pure (D5 constraint).
  - **⌘Enter arm**: a guarded `"enter" if keystroke.modifiers.platform` BEFORE the
    plain enter in `handle_search_key` (the efind ⌘↵ replace-all precedent shape);
    `visible` already in scope.
  - **Jump recipe**: `jump_to_match` → `open_and_place_caret` +
    `position_to_offset(…, Utf32)` + NavStack.
  - **POC**: no multi-file list exists; `FilePaneView`'s header+gutter+lines triple +
    `SyntaxHighlightedLines` are the reusable pieces; the BrowserView wiring
    (App state union + Workspace routing else-if + open helper + LeftRail section
    block) is the 6-step POC checklist.
- **Prior art:** recorded in the spec (the in-tree adoption dominates: group headers,
  the loose-lines highlight path, the guard chain, `editors_under`, the #403 template).
- **Decisions:** D1–D7 in the spec.
- **Auto-approval:** sprint directive; Phase-1 confirmation covered (recorded per §15).

## Phase 2 — Design

### Architecture (§20 confirmed — the Zed read-only multibuffer face, from the
deconstruction docs only; the in-tree idioms carry the implementation)

**The pure model — NEW `crates/marley_app/src/multibuffer.rs`** (cov/MSI-100):
- `CONTEXT_LINES: usize = 2` (D5).
- `ExcerptLine { row: usize /* 0-based file row */, text: String, spans: Vec<(usize, usize)> /* match (col,len) in chars */, window_start: bool }` — `window_start` marks a
  disjoint window's first line; the render draws a hairline there (no separate divider
  slot — slot math stays two-kind).
- `FileExcerpts { path: PathBuf, match_count: usize, truncated: bool, lines: Vec<ExcerptLine> }` — one group per FILE; windows built per match row `[r−2, r+3)` clamped,
  merged when overlapping OR adjacent (half-open, AD-305).
- `MultibufferModel { query: String, files: Vec<FileExcerpts>, skipped_files: usize }` +
  `total_rows()` (Σ 1 header + lines), `locate(slot) -> Row { Header(fi) | Line(fi, li) }`
  via prefix sums (the WrapIndex shape), `first_slot_of_file`, `line_at(fi, li)`
  accessors, `footer_summary()`.
- `build(query, files: &[(PathBuf, Vec<LineMatch-like>, bool)], source: impl Fn(&Path) -> Option<String>) -> MultibufferModel` — the text SOURCE IS INJECTED, so the
  builder is a pure unit-test surface; a `None` source result increments
  `skipped_files` and drops the group (REQ-004's degrade arm).
- `selected: usize` display-row selection lives IN the model (Content-owned state,
  like the finder's), with `move_selection(±1)` skipping Header rows.

**Content/tab wiring (the #403 checklist, concretized):**
- `Content::Multibuffer(Box<MultibufferModel>)` + `ContentKind::Multibuffer`;
  `addable() = false` (no ＋-menu row — see the D7 deviation below); ALL FIVE
  accessors gain their `None` arm (exhaustive); door pair: NO resolve (no dedupe key)
  — `insert` directly (always-insert), `release_multibuffer_views` mirrors the Browser
  release (dropped-on-last-close); the insert's acquired view is the tab's view.
- `TabContent::Multibuffer(ContentId)`; `rail_section() = Editor`; `Tab::multibuffer`
  ctor; `is_multibuffer`/`multibuffer_content_id` accessors; `key_context() = &[]`
  (the Browser posture; the multibuffer's OWN key arm handles ↑/↓/Enter);
  `MULTIBUFFER_TAB_TITLE = "Search results"` with the live query appended at ctor
  (`custom_title` unused). `rail_rows`' per-tab match gains the `content: None` arm
  (the Browser/Cockpit shape).
- **Persistence: DROPPED.** The exhaustive persist writer maps Multibuffer → no entry
  (the documented drop arm); `tab_survives_shell_entry` = false; no `TabLayout`
  variant, no reader arm; the wire-format doc records "a Multibuffer tab is transient".
  Restore's `surviving_tabs` reclamp already handles the vanished-active-tab case.
- `status_bar.rs`: `FocusTab::Multibuffer` + label `"search results"`.

**The body render (an `else if active_tab().is_multibuffer()` arm before the code_view
arm):** the repo's SECOND `uniform_list` — `multibuffer_scroll: UniformListScrollHandle`
(new field), sized `model.total_rows()`, the 'static closure re-reading the model via
`entity.read(app)` + `locate(slot)`:
- `Header(fi)` → the search overlay's group-band idiom (bg, `rel_under_root` label
  truncated at 76, right count with `+`).
- `Line(fi, li)` → a 3-part row: gutter number (`gutter_label(row+1, gw)` with gw from
  the file's max row), then `StyledText::new(text)` with `highlight_ranges(text,
  language_of(path))` colors + the match spans as background bands (char→byte via the
  line's own char_indices — the spans are char (col,len); the render maps once) +
  the accent wash when `slot == selected`; a top hairline when `window_start && li > 0`.
- Footer: `footer_summary` under the list (drops + counts).

**Keys:** a `handle_multibuffer_key` arm in the key dispatch BEFORE the editor router,
gated on the active tab being a multibuffer (↑/↓ move the selection skipping headers +
`scroll_to_item`; Enter jumps `line_at(selected)` via the `jump_to_match` recipe —
`open_and_place_caret` + Utf32 `position_to_offset` + NavStack — WITHOUT closing the
tab; Esc does nothing v1). `stop_propagation` on handled keys (F-#553). Click on a
line row = the same jump; click on a header = select it.

**Materialization (`open_multibuffer_from_search`, app.rs):** guarded on
`open_search` having ≥1 match; assembles the source closure — the live-override map
(`editors_under`, the search worker's own assembly shape) checked first, else
`load_file_text` (a small shim over canonical_under_root → metadata → 2MB cap →
binary sniff → flash+None, the viewer chain's primitives); `multibuffer::build`;
`Content::Multibuffer` insert; `Tab::multibuffer` push via `add_tab`; overlay closed;
`persist_grid()`. Entry: the ⌘Enter guarded arm in `handle_search_key` (before plain
enter) + a footer hint in the overlay ("⌘⏎ open as multibuffer").

**D7 deviation (recorded):** the palette command is DROPPED — `OpenSearch` lives only
while the overlay is open, and opening the palette closes transient overlays (F-#668),
so a palette entry could never see results: a dead affordance. ⌘Enter + the footer
hint are the entry. (The spec's D7 texted both; this is the design-phase correction.)

### File manifest

**POC first:** 1. `components/views/MultibufferView.tsx` (NEW — header band +
gutter+lines via `SyntaxHighlightedLines`, match bands, hairlines, read-only; jump =
sets activeFile+caret). 2. `overlays/ProjectSearch.tsx` (⌘Enter materializes from the
CORPUS with ±2 context + closes). 3. `App.tsx` (`multibuffer` state member + default).
4. `pages/Workspace.tsx` (editor-section routing arm + open helper). 5.
`components/LeftRail.tsx` (a "Search results" row under EDITOR while open, closable).

**Rust:** 6. `crates/marley_app/src/multibuffer.rs` — ADD (model + builder + row
model + selection + tests at P4). 7. `content.rs` — variant/kind/addable/accessors/
release. 8. `tabs.rs` — TabContent + section + accessors + ctor + title + rail arm +
key_context. 9. `grid_layout.rs` — survives=false + the writer drop arm note. 10.
`status_bar.rs` — focus arm. 11. `app.rs` — scroll-handle field; the ⌘Enter arm +
footer hint; `open_multibuffer_from_search` + `load_file_text`; the render arm + row
closure; the key arm; close-tab/close-project releases; persist writer arm; test
hooks (`open_multibuffer_for_test`, counts/selection oracles). 12. `lib.rs` — `mod
multibuffer;`.

### Regression Test Plan (≥1 row per REQ)

| REQ | Test | Where |
|---|---|---|
| REQ-001 | Drive: seed `open_search` results (test hook) → dispatch the ⌘Enter path → a Multibuffer tab exists in Editor section, overlay None | headless_drive |
| REQ-002 | Units: `locate` row model (headers at group starts, line numbering), `footer_summary`; render-side oracles (row-count hook + header labels); the LIVE capture | multibuffer.rs + drive + validate capture |
| REQ-003 | Drive: select a line → Enter → active editor at (path, row, col), NavStack +1, multibuffer tab still present | headless_drive |
| REQ-004 | Units: builder with an injected source — live-override text wins; `None` source → skipped_files+1, group dropped; drive with a DIRTY open buffer (edited line appears in the excerpt) | multibuffer.rs + drive |
| REQ-005 | Unit table: separate/overlapping/adjacent/out-of-order matches; window clamps at file edges; `window_start` marks; ≥3 out-of-order (PR-1370) | multibuffer.rs |
| REQ-006 | Drive: close the tab → registry view_count 0 for the id; persist+restore drive → no multibuffer tab | headless_drive |
| REQ-007 | Units: prefix-sum boundaries (slot at each group edge, first/last, past-end total); selection skips headers both directions | multibuffer.rs |
| REQ-008 | `scripts/gates.sh --diff` green | gate |
| parity | POC capture ↔ live capture at the same state (a wrapped… same QUERY, headers+gutter+bands), pixels sampled | validate |

Uncoverable: none new (the pure module carries the logic; app.rs is the standing shim).

### Risks / decisions

- R1 — Registry accounting: always-insert means the INSERT's acquired view belongs to
  the tab; close releases exactly once (`#[must_use]` on release enforces attention).
  The #403 acquired-view contract is the trap named in the sweep — the insert path
  here never resolves, so no both-branches subtlety, but the close paths (tab +
  project) must both release.
- R2 — The key arm's placement: BEFORE the editor router, AFTER overlays — a
  multibuffer tab has no editor so the editor router won't shadow it, but overlay keys
  must win (the arm gates on no-overlay-open implicitly via placement order — verify
  at implement against the router's actual ladder).
- R3 — Materialization re-reads every result file (bounded: ≤500 files, 2MB cap each,
  flash+skip on failure — the search worker already read the same set; accepted).
- R4 — The match spans are char (col,len); the render maps char→byte per line once —
  multibyte lines pinned by a unit fixture.
- R5 — `Box<MultibufferModel>` in Content keeps the enum small; the model is immutable
  after build except `selected`.
- R6 — uniform_list #2 shares no state with the editor's (its own scroll handle);
  the PR-137 sizer/closure rule applies (one model read per frame — the closure
  re-reads via entity, the sizer reads before; both hit the same registry entry).

## Phase 3 — Implement

**React-first (done FIRST):** `MultibufferView.tsx` (tab chip + per-file group bands +
gutter/lines via `SyntaxHighlightedLines` + match bands + hairlines + footer),
ProjectSearch ⌘⏎ materialization (corpus ±2 best-effort context) + footer hint, App
state (`multibuffer` + `multibufferShown`), Workspace routing + mount wiring, LeftRail
row ("Search: <query>", single-selection, closable; file rows clear the focus).
Typecheck green. Screenshot READ (`scratchpad/427-poc-multibuffer.png`): two file
groups with right-aligned counts, true line numbers, highlighted rows, banded matches,
footer "4 matches in 2 files", the rail row selected. The design is the port
reference.

**Rust, to the manifest:** `multibuffer.rs` (the pure model: CONTEXT_LINES=2, merged
half-open windows, `Row::{Header,Line}` prefix-sum locate, selection movers skipping
headers, `footer_summary`, `build` with the INJECTED text source); the #403 checklist
arms — `Content::Multibuffer(Box<model>)` + kind + addable(false) + all seven
accessors + `release_multibuffer_views`; `TabContent::Multibuffer` + Editor section +
ctor + `is_/content_id` accessors + key_context(&[]) + the rail_rows arm +
`MULTIBUFFER_TAB_TITLE_PREFIX`; the persist writer DROPS the tab (filter_map + an
explicit `return None` arm — the exhaustive match still forces the next kind to
decide) + the wire-format doc note; `FocusTab::Multibuffer` ("search results");
close-tab + close-project releases; the ⌘⏎ guarded arm + overlay footer hint;
`open_multibuffer_from_search` (editors_under overrides → `load_file_text`'s guard
chain, insert + acquire + add_tab + flash-on-skips); `handle_multibuffer_key`
(↑/↓/Enter with `scroll_to_item` + the jump recipe + NavStack); the key-router arm
(unmodified keys only, after every overlay, stop_propagation); `multibuffer_body`
(the repo's SECOND uniform_list — header bands, gutter numbers, hand-lexer colors +
match-span bands with a once-per-line char→byte map, selected wash, window hairlines,
click = select + jump) + its own `multibuffer_scroll` handle.

**Deviations from design:** (1) the D7 palette command dropped (recorded at design —
the transient-overlay lifecycle makes it dead); (2) test hooks deferred to validate
WITH their consumers (the #337-F2 discipline); (3) `viewer_size_ok(meta.len() as
usize, …)` — the helper's arg is usize (the viewer's own call site casts the same
way).

`cargo check --workspace --tests` clean, zero warnings; fmt applied.

## Inspect (Phase 3.5)

Three parallel critics over the uncommitted diff — lifecycle/registry+persistence,
model correctness (hand-traced every pure fn + the gpui run walk), and
simplification/provenance/parity across BOTH repos. Every finding verified against
the working tree before fixing; all fixes at source, same session.

**Confirmed → fixed:**
1. **HIGH (2 critics) — registry view leak.** `open_multibuffer_from_search` called
   `acquire_view` after `insert` — but `insert` ACQUIRES the first view, so both
   close paths (tab, project) release once and the boxed snapshot stranded at
   views=1 forever. The trap is documented at `open_browser_tab` and still got
   copied. Fix: the acquire + debug_assert deleted; the insert's view IS the tab's
   view (the `resolve_open` miss-arm shape). → F-claude-427-b, PR-claude-427-a.
2. **HIGH (2 critics) — `with_highlights` contract violation.** Match-band byte
   ranges were APPENDED after the syntax ranges — unordered + overlapping; gpui's
   run walk is strictly sequential, so bands painted at the wrong bytes on any
   token-bearing line (no panic — wrong pixels). Resolved by PARITY, not merging:
   the approved POC bands the whole LINE (`bg-primary/10`), not the span — the
   render now feeds syntax-only highlights (ascending+disjoint by construction)
   and washes the full row (alpha 0.10; selected 0.16 wins). The char→byte span
   map deleted; a mid-inspect `merge_highlights` helper was superseded by this and
   deleted (no dead code). Spans stay in the model (jump + #428).
   → F-claude-427-a.
3. **MAJOR (2 critics) — doc/attr splice stripped `cockpit_body`'s mutation mask.**
   The new fn was inserted between cockpit's doc + `mutants::skip` and its `fn`,
   re-attaching both to `multibuffer_body` (which had its own duplicate attr) and
   leaving the cockpit shim unmasked for the next dev-box sweep. Fix: the new fn
   sits above the doc block with ONE attr; cockpit's doc+attr re-attached.
   → F-claude-427-c, PR-claude-427-b.
4. **MEDIUM — gate:2 blocker: 3× `private_interfaces`.** `Content::Multibuffer` +
   the two accessors exposed the `pub(crate)` model types. Fix: multibuffer.rs
   types → `pub` (the module is private; nothing leaves the crate).
5. **MEDIUM — REQ-003's NavStack push was dead code.** `active_editor()` is None
   while the mb tab is active (the dispatch guard requires it), so `from` never
   populated. Fix: the model carries `origin` — the editor loc captured at
   MATERIALIZATION — and `multibuffer_jump_selected` pushes it, `take()`n on the
   first successful jump (after that the still-open tab is the way back; from a
   non-editor tab origin is None and nothing pushes, honestly).
6. **MEDIUM — persisted `active_tab` skew.** The writer drops mb tabs but
   `active_tab` counted the unfiltered list → restore focused a shifted neighbor.
   Fix: count surviving non-mb tabs before the live active index, clamped to the
   persisted list (mb-only project persists `tabs: []` + 0 — noted).
   → PR-claude-427-c.
7. **LOW (2 critics) — sourcing divergence.** `load_file_text` used STRICT
   `from_utf8` (the worker + viewer are lossy — a file the overlay shows was
   "unreadable" at ⌘⏎) and dropped the worker's post-read size re-check. Fix:
   lossy decode + the TOCTOU backstop, the worker's own shape.
8. **LOW (2 critics) — empty materialization.** Every-file-skipped still opened an
   empty tab. Fix: `total_rows() == 0` → flash "No excerpts could be sourced",
   overlay stays, nothing inserted.
9. **LOW — honest counts.** `match_count` counted past-EOF-dropped matches;
   "unreadable" mislabeled past-EOF skips. Fix: in-range count; footer suffix
   "· N file(s) skipped"; flash reworded ("could not be excerpted").
10. **MINOR — footer reuse + the missing cap tail.** `footer_summary` duplicated
    `editor_search::footer_summary` and never carried `OpenSearch.dropped` (the
    overlay's "+K more") — the surface under-reported. Fix: `dropped` rides the
    model; the footer reuses the overlay's base + truncated/skipped suffixes.
11. **MINOR parity — footer placement + hint copy.** POC's footer scrolls WITH the
    content → the Rust footer is now the trailing `uniform_list` slot (total+1),
    the pinned child removed. Hint → "⌘⏎ open as multibuffer" (POC is source).
12. **MINOR POC — ⌘⏎ empty guard.** `!files.length → return` added (the Rust
    twin's guard); tsc clean.
13. **MINOR — stale committed doc.** display_map.rs line 8 claimed #427 inserts
    into the facade — retargeted to the deferred unification (spec Out).
14. **Adopted nanos.** One model read per FRAME (was per slot — R6); redundant
    `li > 0` dropped (`window_start` ⇒ wi>0); the click handler calls
    `multibuffer_jump_selected` directly (the fabricated `Keystroke::parse`
    deleted); `jump_to_match` generalized to `(path, row, col) -> bool` and shared
    with the search overlay (its caller adapted — the ~15-line reimplementation
    deleted); clippy over the new code fixed at source (`OwnedSource` alias;
    `enumerate().take(e).skip(s)`).

**Rejected / recorded, no action:** palette command — a RECORDED design deviation
(F-#668 discipline), not a gap · shift not excluded by the dispatch guard —
harmless · fixed 5-char gutter — the approved POC uses a fixed gutter; POC is the
design source (deviation from the design note recorded here) · write-only
`model.query` — kept knowingly (#428 re-anchor + provenance) · the 3rd verbatim
overrides-map copy — accepted per PR-1691 (the derivation fn is the shared part);
a 4th copy forces `live_texts_under(root)` · POC static-mock gaps (line-click
jump, header click-select) — recorded micro-deviations · aggregate skip flash vs
REQ-004's per-file letter — aggregate is the better UX; the validate drive tests
the aggregate · zero tests in-tree — Phase 4's mandate by design · marley-web
commit hygiene — the POC tree mixes uncommitted #426 + #427 material; plan: TWO
marley-web commits at /commit closeout (the #426 wrap chunk first, then #427 —
`SyntaxHighlightedLines` was already exported at HEAD so the split is clean).

**Verified clean by hand-trace (model critic):** window merge math (clamps,
half-open abutment, out-of-order heal, no-empty-window invariant); locate prefix
sums + saturation; `move_selection`'s underflow clamp; the pre-selection
invariant (a group is always ≥2 slots); the jump encoding (Utf32 — identical to
`jump_to_match`); path plumbing end-to-end (canonical roots, `rel_under_root`
labels, override-map hits, `viewer_size_ok` agreement); ⌘⏎ arm order +
`add_tab` activation; every staleness path degrades without panic.

**Verify:** `cargo check --workspace --tests` clean · `cargo clippy -p marley
--tests` zero warnings · POC `tsc --noEmit` clean · fmt applied.

Ledger appends: F-claude-427-{a,b,c}, PR-claude-427-{a,b,c}.

## Phase 4 — Validate

**Tests added.**
- `multibuffer.rs` `mod tests` — 14 units: window merge (overlap/abut → one run;
  ≥3 out-of-order healed, PR-1370), edge clamps (row 0 / EOF), past-EOF drops +
  in-range `match_count` (+ the row == total BOUNDARY discriminant), None-source
  skip + all-fail EMPTY build, injected-source verbatim quoting (+ same-row span
  order, context rows span-free), multibyte char-domain spans (R4), locate over
  every boundary slot + past-end saturation + `line_at` payloads (REQ-007),
  selection movers (header skips both directions, end clamps, empty no-op, the
  out-of-range reclamp discriminant), footer base+suffix composition (dropped/
  truncated/skipped), carried metadata (query/dropped/origin), one-line-group
  pre-selection.
- `content.rs` — the multibuffer accessors' None arms (every other kind).
- `headless_drive.rs` — 4 drives on the REAL key router:
  `cmd_enter_materializes_a_multibuffer_tab_headless` (REQ-001/002 model side),
  `multibuffer_enter_jumps_and_pushes_the_origin_headless` (REQ-003 — caret 17
  on 'delta', origin = the pre-⌘⏎ editor loc, tab stays),
  `multibuffer_sources_live_buffer_and_skips_unreadable_headless` (REQ-004 —
  dirty text wins, aggregate flash),
  `multibuffer_close_releases_and_restore_drops_headless` (REQ-006 + the
  active_tab reclamp — view_count 1 → close → 0 + dropped content; restore has
  no mb tab and focuses a survivor).
- Hooks WITH consumers: `seed_search_results_for_test`, `nav_pop_for_test`.

**Runs.** `cargo nextest run --workspace`: **2212 passed** (first full run;
2219 after the gate-red additions), 7 skipped. Doctests all green. macOS
`/var → /private/var` canonicalization pinned in the jump drive's expectations.

**Live drive + parity pair (REQ-001/002 pixels).**
- Live: bundle → TCC re-consent (see L-claude-427-live-drive-three-traps-001) →
  `focus clickat:0.8,0.5` → ⌘⇧F → "sec" (worker streamed 5351 matches in 500
  files "+4851 more") → ⌘⏎ (via System Events key code 36; the drive.swift
  `cmd:enter` synth is the recorded harness gap). RESULT: the "Search: sec" rail
  row (selected, Editor section), per-file header band (CONSTITUTION.md · 16),
  TRUE line numbers, merged windows with hairlines at every disjoint boundary,
  full-row washes, `focus: search results` in the status bar, overlay closed.
  The empty-set guard verified live first ("secsec" → 0 matches → ⌘⏎ no-op,
  overlay stays, hint reads "⌘⏎ open as multibuffer").
  Capture: scratchpad/427-live-multibuffer.png.
- POC at the SAME state (query "sec"): scratchpad/427-parity-poc.png. TWO POC
  bugs found by the pixel pass and fixed: (1) the ⌘⏎ materializer tested
  0-based `no - 1` against the corpus's 1-BASED rows — NO line ever carried
  `bg-primary/10`, so the approved capture's wash was silently absent; (2) the
  missing empty-set guard (inspect #12). tsc clean after both.
- **Pixel verdict (sampled, not eyeballed):** match-row wash POC (17,30,34) ↔
  live (18,28,31); header band (26,27,31) ↔ (25,26,28); body bg (11,12,15) ↔
  (14,15,17); hairlines present both sides — every pair within Δ3/channel, the
  same 10%-teal-over-background the code declares on both sides. PARITY HOLDS
  on the new surface. (Recorded POC-internal inconsistency: its tab-strip
  stand-in chip says "Search results · sec" while its own rail row and Marley
  both say "Search: sec" — the rail form is canonical; POC chip alignment left
  to #429.) The live selected-row wash (0.16) has no POC twin — the static mock
  has no keyboard selection (recorded).

**Gate.** First `scripts/gates.sh --diff`: RED ×5 — fmt (unformatted
python-spliced edits), clippy `type_complexity` (the seed hook's tuple — fixed
by the shared `OwnedSourceFile` alias), docs (`<query>` in a tabs.rs doc read as
an unclosed HTML tag — backticked), coverage (2 lines: the new accessors' None
arms — test added), mutation (6 missed in multibuffer.rs: the two `row < total`
filters' `<=` boundary + `move_selection`'s `total - 1` arithmetic — killed by
the two discriminant tests; the pre-selection's `1.min(total_rows - 1)` clamp
was equivalent-by-invariant (a group is always ≥ 2 slots) and RE-EXPRESSED as
plain `1` with the proof inline, the #426 discipline). Re-run:
**GATE GREEN [diff] — 15/15** (receipt written; run 2 log:
scratchpad/gate-427-run2.log, final: gate-427-run3.log).

Pre-existing exclusions: none new (app.rs stays the standing shim exclude).

## Phase 5 — Complete

Docs (§21): CHANGELOG entry (the multibuffer, TICKET-427); `editor.md` gained
"The multibuffer (M32 #427 — the read-only face)"; `roadmap.md`'s Multibuffer
bullet now records phase 1 SHIPPED + the 428/429/430 remainder + the deferred
DisplayMap unification; `app_shell.md` records the first DROPPING persist arm +
the survivor-counted `active_tab` (PR-claude-427-c). Parity sync: the POC
already matches what shipped (the wash-marking + empty-guard fixes were
back-ported AT validate); `MARLEY-PARITY.md` gained the `MultibufferView.tsx ↔
multibuffer.rs/multibuffer_body` row with the sampled pixel pairs + recorded
gaps, and the ProjectSearch row notes the ⌘⏎ arm. marley-web commit plan
(recorded at inspect): TWO commits at /commit closeout — the #426 wrap chunk
first, then #427.

Knowledge appended: F-claude-427-{a,b,c} + PR-claude-427-{a,b,c} (at inspect),
L-claude-427-live-drive-three-traps-001 + AD-claude-427-multibuffer-own-row-
model-snapshot-origin-001 + L-claude-427-approve-poc-captures-by-pixels-not-
presence-001.

Ticket closed → tickets/closed/; pipeline pair archived → completed/.
