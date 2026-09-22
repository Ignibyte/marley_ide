# Find references (⇧F12) — every usage, grouped by file — Notes

- **Forge ticket:** #317 51426ddb-f716-4de2-9865-42ce0b17c1d2
- **AAR:** fe3478a7-7ca2-4ee9-a922-3f9baa3cda28
- **Local ticket doc:** docs/planning/tickets/open/TICKET-317-find-references.md
- **Pipeline spec:** 317-find-references.spec.md
- **pipeline_id:** 81827455-a7cf-4ac4-b7f0-b6a79bfd416a

## Phase 1 — Plan

- **Request:** promote the pre-authored spec (Fable method). ⇧F12 find-references — a grouped picker (file headers + line-text rows, match span emphasized), Enter jumps + NavStack. NINTH of the goal; autonomous-through-commit.
- **Classification / tier:** work pipeline, ONE slice — the pure `group_references` + the request/guard/capability + a NEW grouped picker + the jump + the chord. Not larger (the JUMP path + stale-guard + helpers are all shipped).
- **Forge recall:** NO bulletins. The #312 definition patterns (the request path + `DefinitionKey` stale-guard + `jump_to_definition`/NavStack) are the direct reuse — this is their read-only sibling.

### Seam re-verification against `main` @ `9498150` (#312/#313/#314/#316 landed since)

| Seam | LIVE @ 9498150 | Verdict |
|---|---|---|
| `DefPicker` | `editor_nav.rs:70` — `struct DefPicker { items: Vec<(String, DefLocation)>, selected }` — flat, path:line labels, NO line text, NO grouping | **CONFIRMED — references NEED a NEW row model (D-NEW-ROW-MODEL); DefPicker cannot carry it.** |
| `DefLocation` | `marley_lsp/definition.rs:15` — `{ uri, line, character }`, NO text | **References REUSE it as the LSP `Location` (a reference = uri+line+char); the line TEXT is fetched app-side (D-LIVE-TEXT-FOR-OPEN-BUFFERS).** `parse_definition_result` already normalizes `Location[]` → likely reusable for `parse_references`. |
| `RequestPurpose` | `lsp_host.rs:38` — `Definition(DefinitionKey)` + others | add `References(ReferencesKey)`. |
| `DefinitionKey` (stale-guard) | `editor_nav.rs:113` — position-keyed, NO version (editor_complete.rs:19 confirms it carries no version, unlike Hover/completion) | **the stale-guard pattern to MIRROR verbatim (D-POSITION-KEYED-GUARD); ReferencesKey{uri,line,character}.** |
| `jump_to_definition` | `app.rs:9622` — `(loc: &DefLocation, root)` → open + center + push-NavStack | **the JUMP idiom to reuse (D-PUSH-NAVSTACK).** `lsp_position_for` (app.rs:8631), `go_to_definition` (app.rs:9521). |
| capability template | `inlay_hint_support` (inlay.rs:118) + `formatting_support` (formatting.rs:14) — object-or-bool wrappers | **`references_support(referencesProvider)` follows this template (a masked shim `host.references_support()` over the pure marley_lsp fn).** |
| `cap_with_tail` / `FinderState` / `fuzzy_score` | `editor_symbols.rs:57` / `finder.rs:10` / `marley_search_core/lib.rs:26` | **all shipped — the honest cap, the finder state, the LOCAL fuzzy (the list is in hand; unlike #325's server re-query).** |
| ⇧F12 | `keymap.rs:313` binds plain `f12` (F,F,F,F) = go-to-definition; ⇧F12 `(F,F,F,T,"f12")` UNBOUND | **⇧F12 FREE.** The TWO roster guards: `default_keymap_maps_named_chords` (count) + `all_chords_lists_every_binding` (scoped count + a `.contains`, keymap.rs:1172/1246) — BOTH update for the new chord. |
| **fake_ls** | grep: NO `textDocument/references` in `marley_lsp/src/bin/` | **DRIFT — the fixture does NOT serve references yet; P2 EXTENDS it** (the #313/#331 injection lane pattern), OR the headless drives use the `push_response_for_test` injection (process-less host) as #310-#314 did. |

**Load-bearing Phase-1 findings:**
1. **D-NEW-ROW-MODEL CONFIRMED** — a references-shaped picker (`OpenReferences { finder: FinderState, results: RefResults }` with file-HEADER rows + reference rows carrying line text + a match span). Do NOT stretch DefPicker.
2. **The line TEXT is the genuinely new work** — DefLocation/DefPicker carry none. `group_references` produces the structure; the app fetches text (open buffer = live rows; cold file = one disk read per file, batched by the grouping; unreadable = path-only). This is the REQ-004 headless surface.
3. **`parse_references` likely reuses the `Location[]` normalizer** the definition parser already has (a reference = a Location); confirm at design.
4. **fake_ls needs a references handler** (P2) OR the drives use `push_response_for_test` injection (the shipped headless lane — LIVE off-limits anyway).

### Prior art (§20 sweep — required)
1. **Behavior maps / observed** — VS Code / Zed ⇧F12: a grouped-by-file references list with line previews, declaration included, local-file-first reading order. The picker semantics above match that observed behavior.
2. **Published material** — LSP 3.17 `textDocument/references` + `ReferenceContext.includeDeclaration` + the `referencesProvider` capability. `includeDeclaration: true` is the honest client default (you asked "who uses this" — the definition anchors the list; D-INCLUDE-DECLARATION).
3. **OUR OWN CODE (highest-yield) — the sweep separated reuse from build:** the stale guard (Definition's verbatim), the capability reader (inlay/formatting template), the jump+NavStack (`jump_to_definition`), `cap_with_tail`, `FinderState`, `fuzzy_score`, and likely `parse_definition_result`'s Location normalizer are ALL shipped; the params builder's own doc names references as a future caller. What is GENUINELY NEW (proven by reading DefPicker/DefLocation): the grouped row model WITH line text — no shipped picker carries preview text. No new deps.

§20: LSP 3.17 published spec REIMPLEMENT (clean-room); VS Code/Zed = OBSERVED behavior only, source unread. CONFIRMED.

**Phase 1 PASS.** The recon CONFIRMS the spec's shaping (new row model; the reuse is the jump/guard/capability/helpers, not the picker) + one drift (fake_ls needs a references handler, or use the injection lane). No scope reshape — one slice. Next: `/pipeline:design`.

## Phase 2 — Design

### 1. Architecture

**(a) marley_lsp `references.rs` (NEW — pure, cov/MSI 100):**
- `pub struct RefLocation { uri: String, line: u32, start_col: u32, end_col: u32 }` — a reference's Location WITH the match span. **Why not reuse `DefLocation`{uri,line,character}:** references must EMPHASIZE the match span, and DefLocation drops `range.end`. `RefLocation` captures `range.start.character` (start_col) AND `range.end.character` (end_col); a multi-line range (rare for a reference) clamps end_col to a same-line marker (design: if `end.line != start.line`, set end_col = start_col so emphasis is empty-safe, never a panic).
- `pub fn parse_references(value: &Value) -> Vec<RefLocation>` — `textDocument/references` returns `Location[] | null`; per-element `filter_map` (a shared `one_ref(v)` mirroring definition's `one_location` range-parse, but keeping end_col; `u32::try_from` never `as`, the #312 lesson; malformed element → skipped, never lost line). Reuses definition.rs's uri/range extraction pattern.
- `pub fn references_request_params(uri, line, character) -> Value` — `text_document_position_params(...)` + `context: { includeDeclaration: true }` (D-INCLUDE-DECLARATION; the base builder's doc names this caller).
- `pub fn references_support(server_caps: &Value) -> bool` — `referencesProvider` object-or-`true` (the `formatting_support`/`inlay_hint_support` template verbatim).

**(b) marley_app `editor_references.rs` (NEW — PURE, cov/MSI 100 INCLUDED; NOT app.rs which is coverage-excluded):**
- `pub struct ReferencesKey { uri: String, line: u32, character: u32 }` — the position-keyed stale guard, `DefinitionKey` verbatim (D-POSITION-KEYED-GUARD; no version — an edit doesn't move which symbol was asked about; a newer ⇧F12 supersedes).
- `pub enum RefRow { Header { path: String, count: usize }, Reference { line: u32, start_col: u32, end_col: u32, text: String } }` — a FLAT rows-with-headers list (the picker renders + navigates linearly; ↑/↓ skip `Header`).
- `pub struct RefResults { rows: Vec<RefRow>, count_line: String, shown: usize, more: usize }`.
- `pub fn group_references(refs: Vec<RefLocation>, current_file: &Path) -> RefResults` — **PURE, no IO** (text is EMPTY here, filled app-side): DEDUPE exact `(uri,line,start,end)` dups; sort by `(path, line, start_col)`; **current-file rows FIRST** (partition, D-CURRENT-FILE-FIRST); cap the REFERENCE count via `cap_with_tail(total, ~200)` → `shown`+`more`; interleave `Header{path, per-file count}` before each file's runs; `count_line = "N references in M files"` (computed from the grouped structure, never re-counted — REQ-002). Zero refs → an empty `RefResults` (the caller flashes, never opens — REQ-007).
- `pub struct OpenReferences { finder: FinderState, results: RefResults }` — the picker: nav (↑/↓ over `Reference` rows, skipping `Header`), type-to-filter fuzzy over path+line-text (`fuzzy_score`, LOCAL — the list is in hand), the selected `Reference`'s (line,start_col) → the jump. Esc close.

**(c) app.rs (the shim — orchestration, `mutants::skip` on the render/thread bits):**
- `find_references()` (⇧F12): `lsp_position_for` → gate on `host.references_support()` (no cap → quiet flash, REQ-007) → send `RequestPurpose::References(ReferencesKey{...})` (the #311 request path) → set `OpenReferences` with a `searching…` state (REQ-008). Newer ⇧F12 mints a new key (supersedes).
- `apply_references_response`: the STALE GUARD (drop if `references_request != key` — a superseded query's answer, REQ-006) → `parse_references` → `group_references(refs, current_file)` → `fetch_reference_texts` → resolve the `OpenReferences` `searching…` into the rows.
- `fetch_reference_texts(&mut RefResults, root)`: fill each `Reference.text` — an OPEN buffer's rows read LIVE (`active_editor`/open-file buffers by path); a COLD file reads from disk ONCE PER FILE (batched — group the rows by path, one `read_to_string` per file, slice the needed lines; NEVER per row); an unreadable file → the rows keep EMPTY text (render path-only) (REQ-004; §14 no panic/stall). IO through a testable seam (a root/dir the drives control).
- the jump (Enter): build `DefLocation{ uri, line, character: start_col }` from the selected `Reference` → `jump_to_definition(&loc, root)` (open+center+push-NavStack, D-PUSH-NAVSTACK; REQ-005). Reuses the #312 idiom verbatim.
- the `RequestPurpose::References` drain arm; the picker render (header rows dim, reference rows `line: text` with the `start_col..end_col` span accent-emphasized via the styled-runs the finder rows use); `text_input_blocked` + the ⌘⇧A overlay-clear registration (the #312/#313 choke-points).

**(d) keymap.rs:** ⇧F12 `(F,F,F,T,"f12")` Editor-scoped (plain f12 = go-to-definition, untouched) → `find_references` verb; the TWO roster guards (`default_keymap_maps_named_chords` count + `all_chords_lists_every_binding` scoped count + a `.contains`). **palette.rs:** a "Find All References" CommandId.

**(e) fake_ls vs injection:** LIVE drives are OFF-LIMITS (chad at machine), so the drives use `push_response_for_test(RequestPurpose::References(key), Ok(json))` — the shipped process-less injection lane (#310-#314). fake_ls's references handler is NOT needed for v1 (a follow-up if a live drive is ever wanted).

**§14/§20:** typed, no panic on the response/IO paths (malformed refs skipped; unreadable file → path-only; try_from not `as`); clean-room from LSP 3.17 (`textDocument/references`, `ReferenceContext.includeDeclaration`, `referencesProvider`); VS Code/Zed OBSERVED only (⇧F12, grouped, declaration-included, local-first). CONFIRMED.

### 2. File manifest
- **crates/marley_lsp/src/references.rs (NEW)** — `RefLocation`, `parse_references`, `references_request_params`, `references_support` + tests.
- **crates/marley_lsp/src/lib.rs** — `mod references` + re-export.
- **crates/marley_app/src/editor_references.rs (NEW)** — `ReferencesKey`, `RefRow`, `RefResults`, `group_references` (PURE), `OpenReferences` + the nav/filter methods + tests. (**group_references lives HERE, not app.rs — coverage+mutation INCLUDED → cov/MSI 100.**)
- **crates/marley_app/src/lib.rs** — `mod editor_references`.
- **crates/marley_app/src/app.rs** — `find_references` / `apply_references_response` / `fetch_reference_texts` / the jump / the `RequestPurpose::References` drain arm / the picker render / `text_input_blocked` + overlay-clear registration / the 9 test hooks (`push_response_for_test` already generic; add `find_references_for_test`, `open_references_for_test() -> Option<&OpenReferences>` accessors).
- **crates/marley_app/src/lsp_host.rs** — `RequestPurpose::References(ReferencesKey)` + `host.references_support()` wrapper (`#[cfg_attr(test, mutants::skip)]` — masked shim over the tested pure `references_support`).
- **crates/marley_app/src/keymap.rs** — ⇧F12 + the 2 roster guards.
- **crates/marley_app/src/palette.rs** — "Find All References" CommandId.
- **crates/marley_app/src/headless_drive.rs** — the drives (REQ-004/005/006/007/008 via the injection lane).

### 3. Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `group_references` — dedupe exact dups, sort (path,line,col), current-file FIRST | pure table (editor_references) |
| REQ-002 | the `N references in M files` count line computed from the grouped structure | pure |
| REQ-003 | `cap_with_tail` — >200 refs → shown + "+K more" (reuse the shipped helper's boundaries) | pure |
| REQ-004 | `fetch_reference_texts` — open buffer LIVE text, cold file disk (ONE read/file), unreadable → path-only | headless (dir seam) |
| REQ-005 | Enter → `jump_to_definition` centered + NavStack pushed (⌃- returns), across files | headless (injection) |
| REQ-006 | a superseded `ReferencesKey`'s response is DROPPED (newer ⇧F12 raced) | headless — the guard |
| REQ-007 | zero refs / no `referencesProvider` → quiet flash, NO empty picker | headless |
| REQ-008 | the `searching…` state resolves when the injected response lands | headless (injection lane) |
| REQ-009 | ⇧F12 + the palette row resolve to `find_references`; plain f12 untouched; both roster guards | unit (keymap) |
| edges | `parse_references` (Location[]\|null, malformed skipped, multi-line range→empty-span-safe, 2^32→skip); the header-skip nav; a 500-ref symbol batches disk reads once/file | pure + headless |

LIVE drive: OFF-LIMITS (chad at machine) — the pixel deferred; the grouping/cap/count are PURE, the jump/guard/resolve/text-fetch headless via the injection lane. `references_support` is a masked shim over the tested pure fn.

### 4. Risks / decisions
- **RefLocation ≠ DefLocation** (the span): references need `end_col` for emphasis; the jump adapts `RefLocation → DefLocation{character:start_col}`. Recorded — do NOT force DefLocation.
- **group_references is PURE (no text)**; `fetch_reference_texts` is the app-side IO (batched one-read-per-file). The split keeps the grouping cov/MSI 100 + the IO headless-testable via a dir seam. A multi-line reference range → empty emphasis span (safe).
- **The flat RefRow-with-headers model** (vs nested files): linear render + skip-header nav; the picker's selected index walks `Reference` rows only.
- **fake_ls-vs-injection:** injection (shipped lane) for v1; fake_ls references handler deferred (LIVE off-limits).
- **The stale guard is position-keyed** (no version — the #312 pattern); newest ⇧F12 wins; a raced answer drops.

**Design PASS.** Ready for Phase 3 — Implement.

## Phase 3 — Implement (IN PROGRESS)

**DONE + compile-verified:**
- **crates/marley_lsp/src/references.rs (NEW)** — `RefLocation{uri,line,start_col,end_col}` + `parse_references` (Location[]|null, `one_ref`+`span_of` mirroring definition.rs, `u32::try_from`, multi-line range→end_col=start_col) + `references_request_params` (position base + context.includeDeclaration:true) + `references_support` (referencesProvider object-or-true, the formatting_support template). lib.rs `pub mod references` + re-export. **`cargo check -p marley_lsp` clean (49s job-capped).** `git add -N` done.
- **crates/marley_app/src/editor_references.rs (NEW, PURE)** — `ReferencesKey{uri,line,character}` (the #312 DefinitionKey verbatim); `RefRow::{Header{path,count}, Reference{uri,line,start_col,end_col,text}}` (**DEVIATION: Reference carries `uri` — self-contained jump + text-fetch; the design said no-uri, but the jump/fetch need the file**); `RefResults{rows,count_line,shown,more}` + accessors + `rows_mut` (app fills text); `group_references` (PURE, no IO — dedupe/current-first sort/cap_with_tail(200)/interleave Header per contiguous file run/count_line); `OpenReferences{finder:FinderState, results}` (filter-in-place: `visible_refs` = Reference rows fuzzy-matching in ROW ORDER, grouping preserved; move over visible refs; `chosen`). lib.rs `mod editor_references`. `git add -N` done.

**REMAINS (app plumbing — mirrors #312 verbatim):**
- **lsp_host.rs:** `RequestPurpose::References(ReferencesKey)` (mirror `Definition`) + `host.references_support()` wrapper (`#[cfg_attr(test, mutants::skip)]` over the pure `references_support`, like `formatting_support`).
- **app.rs:** fields `references_request: Option<ReferencesKey>` + `open_references: Option<OpenReferences>`; `find_references`/`send_references_request` (mirror go_to_definition/send_definition_request app.rs:9521/9534 — caret→lsp_position_for→ReferencesKey→references_request_params→gate on host.references_support()→request; set references_request + clear open_references [searching]); `apply_references_response` (mirror apply_definition_response app.rs:9568 — superseded-key drop [REQ-006] + live-uri guard + parse_references→empty→flash [REQ-007]→group_references→fetch_reference_texts→open_references=Some); `fetch_reference_texts` (batch by path: open-buffer line = live, cold = one std::fs::read_to_string per file cached, unreadable→empty; NEEDS an open-buffer-line-by-path helper); the Enter-jump (build DefLocation{uri,line,character:start_col} from the chosen Reference → jump_to_definition app.rs:9622); the `RequestPurpose::References` drain arm (mirror the Definition arm ~app.rs:9417); the picker render (Header dim + Reference `line: text` with start_col..end_col accent span); `text_input_blocked` + ⌘⇧A overlay-clear; the `*_for_test` hooks (find_references_for_test, open_references_for_test).
- **keymap.rs:** ⇧F12 (F,F,F,T,"f12") Editor-scoped → find_references + BOTH roster guards. **palette.rs:** "Find All References" CommandId.

**DONE (app plumbing — all compiled clean, no warnings):** lsp_host `RequestPurpose::References` + `references_support()` wrapper (mutants::skip); app.rs the 2 fields + `find_references`/`send_references_request` (capability-gated) + `apply_references_response` (stale-guard + live-uri guard + empty→flash + group + fetch + open) + `fetch_reference_texts` (open-buffers-live via the #326 overrides idiom + one disk read/cold file cached + unreadable→empty) + `jump_to_selected_reference` (adapts Reference→DefLocation→jump_to_definition) + the References drain arm + `handle_references_key` (↑/↓/Enter/Esc/type-filter) + the dispatch wiring + `references_overlay` render + the ⌘⇧A clear + `text_input_blocked` + 4 `*_for_test` hooks (`drive_references_for_test`, `open_references_shown_for_test`, `references_searching_for_test`, `selected_reference_for_test`); keymap ⇧F12 + BOTH roster guards (total 81→82, scoped 36→37); palette CommandId(27) "Find All References".

**DEVIATIONS:** (1) `RefRow::Reference` carries `uri` (jump/fetch need the file; the design said no-uri). (2) the "searching…" state is app-side `references_request.is_some() && open_references.is_none()` (no variant in OpenReferences — keeps the pure type simple). (3) dropped `RefResults::shown()`/`is_empty()` + the `shown` field (dead in the lib build — used only by cfg(test)); the count is `count Reference rows` + the count_line embeds it; `more()` (the cap tail) stays (render uses it). (4) sub-span (match-column) emphasis WITHIN a line is a v1 cut — the whole selected row is accent-washed; per-column highlight is a follow-up (not a testable AC).

**Compile/test state:** `cargo check -p marley` clean (no warnings, 44s job-capped); marley_lsp clean; roster + palette 12/12 pass (incl `every_cockpit_command_resolves` for CommandId(27) + the roster counts). Both NEW files `git add -N`'d. **⚠️ ALL builds job-capped (`CARGO_BUILD_JOBS=4`) — the late-session deadlock.**

Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.

## Phase 3.5 — Inspect

3 parallel general-purpose critics over the diff (pure grouping+parse · orchestration+state · simplification/reuse/hygiene), each verifying concretely by reading the code (one ran `cargo mutants --list` + `cargo doc`; none edited the tree). **2 HIGH + several LOW; all reviewed, 2 HIGH + 1 LOW FIXED, the rest rejected-with-reason or carried to Validate.**

| # | Sev | Finding | file:line | Verdict | Fix |
|---|-----|---------|-----------|---------|-----|
| F1 | HIGH | `find_references` MISSING `#[cfg_attr(test, mutants::skip)]` — its twin `go_to_definition` has it; a new shim fn with no test caller → its 2 mutants (body→`()`, delete `!`) SURVIVE → gate:5 MSI<100 FAILS (app.rs is coverage-excluded but NOT mutation-excluded; gate:5 mutates the in-diff app.rs lines, kept out only by per-fn skips) | app.rs:9769 | **REAL** — verified: doc comment sat directly on the fn, no attribute; all 7 other shims had it | **FIXED** — added the skip (mirrors `go_to_definition` verbatim) |
| F2 | HIGH | The "Finding references…" card is a naive app-side latch a timed-out/disconnected request NEVER clears — the pump drops a dead request's purpose with NO response (lsp_host.rs:322/345), so `apply_references_response` never runs, `references_request` stays `Some` forever, and the OCCLUDING card is stuck permanently (Esc can't reach it, a fresh ⇧F12 on a dead server fails at send without touching the latch). **The exact #331 `has_pending_inlay` lesson, reintroduced** — #317 made the latch VISIBLE (a card), which turns the shared latch bug into a real defect | app.rs:12037 + lsp_host.rs:322/345 | **REAL** — traced the timeout/disconnect paths; the card render gated only on `references_request.as_ref()` | **FIXED** — added `LspHost::has_pending_references()` (mirrors `has_pending_inlay`); the searching card now renders ONLY while the host reports a References purpose genuinely in flight; also added `references_request = None` to the ⌘⇧A/launcher overlay-clear (the escape hatch fully resets) |
| F3 | LOW | `span_of` can yield `end_col < start_col` for a malformed SAME-LINE end-before-start range (`{start:{5,10}, end:{5,3}}` → `(5,10,3)`), unclamped — currently harmless (nothing slices `text[start..end]`; the render washes the whole row) but the module doc PROMISES emphasis, so a future `start_col..end_col` slicer would panic on a malformed server range | references.rs:64 | **REAL (latent)** — both critic 1 & 3 flagged; §14 says no panic on a response-path input, and a server range IS a response-path input | **FIXED** — `end_char.max(start_col)` clamps to an empty span; documented `end_col >= start_col` as a `RefLocation` INVARIANT so any downstream slice is panic-safe |
| F4 | LOW | Dedupe keyed on the RAW uri, not the decoded path — two uri spellings that decode to the same path both survive (a visual dup under one Header) | editor_references.rs:97 | **REJECTED** — the contract IS "exact `(uri,line,start,end)`"; real servers (rust-analyzer) emit canonical uris; they still group correctly (contiguity intact). Theoretical, not a live bug |
| F5 | LOW | `handle_references_key` filter-push arm omits the `!platform && !control` modifier guard that the `naming_workflow` text draft uses — a Cmd/Ctrl chord could push `key_char` garbage into the filter | app.rs:2865 | **REJECTED** — the codebase's own comments establish `key_char` is absent under Ctrl and Cmd chords carry no IME text on macOS (so the arm pushes nothing, just swallows — same as the `def_picker`/`code_action` picker siblings, which ALSO don't guard). Matching the PICKER family (not the text-draft) is the consistent choice; practically unreachable |
| F6 | LOW | Spec Title/Scope prose still says "match span emphasized" but the impl washes the whole row | spec.md:14,38 | **ACCEPTED (not an AC)** — the EARS table has no `shall` for sub-span emphasis (REQ-004 = live/disk text); the v1 cut is honestly recorded in the `references_overlay` doc + deviation #4. No code change; the prose oversells but the plan is frozen — the deviation lives in the notes |

**Mutation surface (critic 3 ran `cargo mutants --list`, rc=0): 53 mutants → 46 VIABLE must-kill, 7 UNVIABLE (no `Default` derive — do NOT chase).** Carried to Validate:
- **Landmine A:** the 8 `span_of` tuple mutants (`Some((0|1,0|1,0|1))`) + the `:64 ==`→`!=` + **the new `.max()` clamp mutant** need an OFF-GRID fixture: `line∉{0,1}`, `start_col∉{0,1}`, `end_col∉{0,1}`, `end_col≠start_col`, on a MULTI-LINE range — AND a separate end-before-start same-line fixture to kill the clamp (the definition.rs off-grid-fixture lesson).
- **Landmine B:** `RefResults::rows_mut`→`Vec::leak(Vec::new())` — its only production caller is the skipped `fetch_reference_texts`, so a DIRECT pure test on `rows_mut()` is needed or it survives.
- **Landmine C:** `references_request_params`→`Default::default()` is VIABLE (serde_json `Value`'s Default = `Null`) — a test must assert the params are a non-null object with `context.includeDeclaration:true`.
- UNVIABLE (skip): whole-fn `Default::default()` body-replaces on fns returning `RefResults`/`RefRow`/`RefLocation`/`OpenReferences` (none derive `Default`).

**Sound (attacked, held):** the dedupe order (filter-before-map), the current-file-first sort + per-path contiguity (the `partition_point` interleave proven safe), the honest cap + post-truncate count consistency, empty-input no-panic, the `Location[]|null` lenient parse, `u32::try_from` no-truncation, the live-uri guard's byte-identical uri derivation (send vs apply), the double-in-flight drop, the disk cache holding both Some AND None, the borrow-safety of `fetch_reference_texts`, the NavStack push on jump, the picker owning keys before the editor, `references_support` a faithful copy of the capability template, the 4 deviations (uri-on-row / app-side-searching / dropped-shown / no-sub-span all sound), no Zed/Warp, no narrowing `as`, `cargo doc -p marley_lsp` clean.

**Verify:** `cargo check -p marley` clean (21s job-capped, only the pre-existing `block v0.1.6` transitive warning); roster + palette 12/12 still green after the fixes.

Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.

## Phase 4 — Validate

**Tests added (20 + 1 test hook), one+ per EARS REQ:**
- **marley_lsp `references.rs` (10 pure):** `parse_single_line_reference_keeps_the_span`, `parse_multi_line_range_collapses_end_col_to_start` (kills the `==` comparator), `parse_end_before_start_is_clamped_to_an_empty_span` (F3 — the `.max()` clamp), `parse_array_keeps_order`, `parse_null_and_empty_are_no_references`, `parse_malformed_elements_are_skipped`, `parse_mixed_array_keeps_the_good_elements`, `parse_position_out_of_u32_range_is_skipped`, `parse_negative_and_fractional_positions_are_skipped`, `request_params_carry_position_and_include_declaration` (landmine C — non-null object + includeDeclaration), `references_support_reads_object_or_true_only` (kills `||`/`==`). Every well-formed fixture is OFF-GRID (line 5, cols ∉{0,1}) to kill the `Some((0|1,..))` tuple mutants.
- **marley_app `editor_references.rs` (7 pure):** `group_dedupes_sorts_current_first_and_interleaves` (REQ-001 — the full shape table: dedupe drops the exact dup, current-file Header first, per-file contiguity + interleave, per-file counts), `group_sorts_same_line_refs_by_start_col` (REQ-001 — the sort's FINAL start_col tiebreak), `group_non_file_uri_keeps_the_raw_uri_as_path` (REQ-001 — the non-`file:` uri fallback), `count_line_singular_plural_and_empty` (REQ-002), `cap_truncates_with_an_honest_tail` (REQ-003 — 250→200 + more()==50), `rows_mut_exposes_rows_for_text_fill` (landmine B), `open_references_navigates_and_filters` (nav clamps + fuzzy filter resets selection + `chosen()` maps through the filter).
- **marley_app `headless_drive.rs` (3 injection-lane drives + the `reference_rows_for_test` hook):** `lsp_find_references_searching_then_fills_live_disk_and_missing_text_headless` (REQ-004 live-buffer-wins-over-overwritten-disk / cold-disk / missing→empty + REQ-008 searching→resolve), `lsp_find_references_jumps_and_navstack_across_files_headless` (REQ-005 — ↓ selects the B ref, Enter opens B + pushes NavStack, ⌃- returns; REAL keystrokes prove the picker's key arm), `lsp_find_references_superseded_dropped_and_empty_flashes_headless` (REQ-006 superseded drop + REQ-007 empty flash "No references found"). REQ-009 was already green (roster + palette 12/12).

**Runs (all job-capped `CARGO_BUILD_JOBS=4`):** marley_lsp references 72/72; editor_references 5/5; the 3 drives 3/3; roster+palette 12/12 — all green. The full workspace suite ran GREEN inside the gate (1667 tests via nextest + doctests).

**Coverage red fixed at SOURCE — TWO distinct traps (both the #315 family):**
- **Round 1 (fully-uncovered lines):** the first `--diff` gate RED on gate:4 — `editor_references.rs` 98.57% lines (lines 355 & 436). Both were **never-taken `panic!` match arms** in my OWN tests (`other => panic!(...)` / `RefRow::Header => panic!(...)`) — an explicit dead arm is an uncovered line. FIX: replaced each match with a branch-free whole-value `assert_eq!` on `Option<&RefRow>` (no dead arm; also exercises the derived `PartialEq`). `--show-missing-lines` (gate profdata) pinpointed these — they're FULLY-uncovered lines, so the detail lists them.
- **Round 2 (uncovered REGIONS = never-called closures):** the re-run went 99.32% (2 "missed lines") but `--show-missing-lines` listed NOTHING for the file — because these are lines with a MIXED hit/miss region (a never-called CLOSURE), which cargo-llvm-cov counts against LINE coverage yet renders 'covered' in HTML. Two closures no test reached: (a) `group_references`'s `.unwrap_or_else(\|\| r.uri.clone())` — the non-`file:` uri fallback (every test uri was `file://`); (b) the sort comparator's FINAL `.then_with(\|\| ra.start_col.cmp(...))` — reached only for two refs on the SAME file+SAME line. **Diagnosis method: the lcov `LF/LH` said 2 uncovered, the HTML showed 0 fully-uncovered lines, the missed-FUNCTIONS count (2) matched → the misses are never-called closures, found by reading the sort/decode code.** FIX: `group_non_file_uri_keeps_the_raw_uri_as_path` (a `untitled:` uri → the fallback) + `group_sorts_same_line_refs_by_start_col` (two refs same file+line, different start_col → the tiebreak). An isolated coverage check (`--fail-under-lines 100`) then confirmed `editor_references.rs` 321/321 lines = 100.00% BEFORE the full gate.

**Mutation (gate:5): MSI 100.0% — 50 caught / 0 missed** (the ~46 predicted viable + a few; the UNVIABLE no-Default body-replaces correctly never appeared). Landmines A/B/C all killed.

**Build-hang recurred (recovered):** a mid-session `nextest` build DEADLOCKED (rustc 0.0% CPU / state S / 12 min, empty output). Recovery per the standing rule: `pkill -9` ALL rustc/cargo/nextest → `cargo check -p marley_syntax` (0.2s, toolchain OK) → retried job-capped, built clean. `CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0` throughout.

**LIVE pixel:** deferred, not skipped — gpui 0.2.2's test platform has no `draw`, so the 3 headless injection drives + the pure tables carry #317 (chad at the machine → no synthetic-input drives anyway). Documented in the drive-section comment.

**GATE GREEN [diff] — 15/15** (fresh receipt written): fmt · clippy · tests (1667 + doctests) · coverage 100% (editor_references.rs 321/321, references.rs 164/164 lines) · mutation MSI 100% (50/50) · miri (skip-clean) · deny/audit/machete/gitleaks/shellcheck/no-suppressions/source-bans/docs · visual/AX. No pre-existing failures.

Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.

## Phase 5 — Complete
- Docs updated; AAR capture (lessons / failures / prevention rules / ADs); archive.
