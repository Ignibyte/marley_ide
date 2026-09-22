# Workspace symbols (⌘T) — Notes

- **Forge ticket:** #325 e3000b55-3787-4d70-8470-156fa935d128
- **AAR:** d18ca6f9-f7e7-439b-8b7d-47ed4b8bd232
- **Local ticket doc:** docs/planning/tickets/open/TICKET-325-workspace-symbols.md
- **Pipeline spec:** 325-workspace-symbols.spec.md

## Phase 1 — Plan
- **Request:** `/work 325` — ⌘T jumps to any symbol across the workspace by name (the cross-file sibling of
  #304's in-file ⌘⇧O), riding `workspace/symbol` server-filtered.
- **Classification / tier:** work pipeline (integration-heavy — reuses the finder modal, the #304 row
  render, #312 navigation, the #313 park/stale). Systems: server/ui (the picker) + marley_lsp (a new pure
  parse seam + the handshake cap).
- **Forge recall (§18.3):** bulletins none. The load-bearing carry-forwards:
  - **The #323/#324 CAPABILITY posture** — `workspace/symbol` serves WITHOUT the client cap (like
    signatureHelp, NOT codeAction's dead-without-it); advertising `workspace.symbol.symbolKind` UPGRADES
    the reply (the kind set). Decide honestly at design; the live drive confirms symbols return.
  - `PR-claude-second-consumer-must-inherit-the-first-consumers-guards-001` — Enter reuses #312
    `open_and_place_caret` + NavStack + encoding WHOLE.
  - `PR-claude-new-overlay-register-at-every-choke-point-001` — the picker is a MODAL (owns the keyboard,
    like the finder/def_picker — NOT #324's passive card); register at every choke.
  - the #313 park/debounce + stale-key, here keyed on the QUERY STRING (a per-keystroke re-query).
  - the #317 cap+"+N more" tail rule.
- **Discovery (the edit surface for Design):**
  - REUSE: `finder.rs::FinderState` (the modal query+selected recipe), `editor_complete.rs::kind_glyph`
    (extend for SymbolKind), `code_view::truncate_cols` (the right-aligned path:line), the #304 in-file
    symbol row render (whatever module #304 landed), `app.rs::open_and_place_caret` + the NavStack (#312),
    `lsp_position_for`'s inverse (encoding), the ONE drain `consume_lsp_responses`, `RequestPurpose`, the
    handshake, `path_from_file_uri`.
  - NEW: `marley_lsp/src/workspace_symbol.rs` (pure parse + params + caps), the app picker state (a
    `SymbolQuery` stale key + reuse FinderState), the app.rs shim (⌘T, park/consume, multi-root fan-out +
    merge, drain arm, Enter→#312, row render, handshake cap), `keymap.rs` (⌘T), the re-exports.
  - **Design must locate #304's actual module/render** (the in-file-symbols sibling) via Explore — the row
    render + kind_glyph extension are shared; I should not duplicate.
- **Decisions:** D1 finder modal recipe; D2 server-filtered debounced, response REPLACES, stale-key=query;
  D3 both shapes + missing-range→line 0 + skip malformed; D4 Enter reuses #312 whole; D5 multi-root merge;
  D6 advertise workspace.symbol.symbolKind (upgrade); D7 v1 cuts. (Full text in the spec.)

## Phase 2 — Design

### Architecture / approach
⌘T (**Editor-scoped**, shadowing the global new-tab — the #265 pattern) opens a SERVER-FILTERED modal
picker: the `finder.rs` `FinderState` recipe drives the query + selection, but unlike ⌘P (client-filtered
files) each keystroke PARKS a `workspace/symbol` request (debounced one pump tick, the #313 idiom) that the
pump fans out to EVERY Ready host; each matching answer (its query key == the live query) is parsed
(`workspace_symbol.rs`, both shapes + missing-range) and APPENDED to the results (a query CHANGE clears
them first — so multi-root accumulates without per-host tracking, and a stale-query answer is dropped).
Enter reuses #312 `open_and_place_caret` + NavStack + the encoding-aware landing WHOLE. §14: the parse is
pure + typed (`filter_map`, no panic on a hostile payload), no process spawn (the wire is #308's). §20
reconfirmed — Zed/VS Code ⌘T observed only; Marley composes its OWN finder picker + a #304-style row render
over the published LSP 3.17 wire. No copyleft source read.

### File manifest
| File | Change |
|---|---|
| `crates/marley_lsp/src/workspace_symbol.rs` (NEW) | PURE: `WorkspaceSymbolResult{name, kind, container, path, line, character}`; `parse_workspace_symbols` (BOTH `SymbolInformation` and `WorkspaceSymbol`; a `WorkspaceSymbol` `location` without `range` → line 0; `path_from_file_uri`; per-element `filter_map`); `workspace_symbol_params(query)`; `workspace_symbol_support(caps)`. |
| `crates/marley_lsp/src/lib.rs` | re-export the new public items. |
| `crates/marley_lsp/src/handshake.rs` | advertise `workspace.symbol.symbolKind.valueSet` (the full kind set); extend `initialize_params_are_honest`. |
| `crates/marley_app/src/editor_symbols.rs` (NEW) | PURE: `SymbolQuery{ query }` (the stale key — the query string); `symbol_kind_glyph` (SymbolKind→glyph — SEPARATE from `kind_glyph`, which is CompletionItemKind); `cap_with_tail(results, max)` → the capped list + the "+N more" count (the #317 rule); a `SymbolRow` display projection (glyph + name + container + `path:line`). |
| `crates/marley_app/src/lib.rs` | module decl + re-exports. |
| `crates/marley_app/src/app.rs` | shim: `symbols_open` flag + a `FinderState` + `Vec<WorkspaceSymbolResult>` + `symbol_query: Option<SymbolQuery>` (the live key) + `pending_symbol_query` (the parked debounce); the ⌘T dispatch (opens the picker); the keystroke path (finder push/backspace → park a new query, clear results); `consume_symbol_query` (pump — fan out to every Ready host); the `WorkspaceSymbol` drain arm (append if the answer's query == live, capped); `handle_symbols_key` (printable→re-query, ↑/↓, Enter→#312 open+NavStack, Esc); `symbols_overlay` render; choke registration + `text_input_blocked`; test hooks. |
| `crates/marley_app/src/lsp_host.rs` | `RequestPurpose::WorkspaceSymbol(SymbolQuery)`; `workspace_symbol_support()` accessor (+ sibling `mutants::skip`). |
| `crates/marley_app/src/keymap.rs` | ⌘T **Editor-scoped** → `"workspace-symbols"` (shadows the global `new-tab`); roster bump. |
| `crates/marley_app/src/headless_drive.rs` | the drives. |

### Regression Test Plan (≥1 per AC)
| REQ | Test | Kind |
|---|---|---|
| REQ-002 | `parse_both_shapes` — `SymbolInformation` + `WorkspaceSymbol` → the result; a `WorkspaceSymbol` without a range → line 0; a `file:` uri decoded to a path | pure unit (`workspace_symbol.rs`) |
| REQ-002 | `parse_skips_malformed` — no name / bad uri / non-object → dropped, never a panic | pure unit |
| REQ-001 | `workspace_symbol_params_shape` — `{ query }` | pure unit |
| REQ-008 | `workspace_symbol_support` table + `initialize_params_are_honest` asserts `symbolKind.valueSet` | pure unit |
| REQ-004 | `symbol_kind_glyph_table` (SymbolKind numbers) + `cap_with_tail` (≤max kept, the "+N more" count) | pure unit (`editor_symbols.rs`) |
| REQ-003 | `symbol_query_key` — a differing query is a different key | pure unit |
| REQ-001/003 | `sym_query_replaces_and_stale_dropped_headless` — type → park → consume → results REPLACE; a stale-query answer dropped | headless drive |
| REQ-005 | `sym_enter_jumps_and_navstack_headless` — Enter → `open_and_place_caret` + NavStack (⌃- returns); a failed open flashes + moves nothing | headless drive |
| REQ-006 | `sym_no_host_flashes_headless` — no host / no provider → quiet flash, no picker | headless drive |
| REQ-007 | `sym_multi_root_merges_headless` — 2 Ready hosts → both queried, results merged | headless drive |
| REQ-001/005/008 | LIVE tee drive: ⌘T `main` in the fixture → a row for `main` in src/main.rs → Enter lands centered, ⌃- returns; the real `workspace/symbol` frame carries the typed query. | live |

Uncoverable-without-live: the multi-root fan-out's actual wire send (the `lsp_position_for`/tracked-file
gate, per #324) — headless proves the fan-out DECISION + the merge; the live drive proves the send + the
real symbol list.

### Risks / decisions
- **D-CHORD (⌘T Editor-scoped shadow)** — the ticket assumed ⌘T is free; it is Marley's `new-tab` (M10
  #170, a Warp-ism). Resolve via the #265 context-scope: ⌘T in the EDITOR → workspace symbols (VS-Code-
  like), in a TERMINAL → new-tab (Warp-like). Both surfaces get the right behavior; no chord is broken.
  (Flagged as a deviation from the ticket for the human-review checkpoint — an easy keymap change if chad
  wants a different chord.)
- **Multi-root merge** — a query CHANGE clears `results`; each Ready host's matching answer APPENDS. Avoids
  per-host result bookkeeping; a stale-query answer is dropped by the query-key check. Cap applies to the
  MERGED list.
- **`symbol_kind_glyph` is SEPARATE** from `kind_glyph` — `SymbolKind` and `CompletionItemKind` are
  different LSP enums (kind 2 = Module vs Method), so reusing `kind_glyph` would mis-glyph every row.
- **Capability (D6)** — advertise `workspace.symbol.symbolKind` (UPGRADE the kind set; rust-analyzer serves
  `workspace/symbol` without it — the signatureHelp posture). The live drive confirms symbols return.

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` + `cargo clippy --lib` clean; keymap roster 58/17 green.

- **`marley_lsp::workspace_symbol`** (pure): `parse_workspace_symbols` (BOTH `SymbolInformation` and
  `WorkspaceSymbol` via a per-element `filter_map` — name + kind + containerName + `location.uri` decoded
  by `path_from_file_uri`; a `location` without a `range` → line 0); `workspace_symbol_params(query)`;
  `workspace_symbol_support`. Re-exported.
- **`handshake.rs`**: advertises `workspace.symbol.symbolKind.valueSet` = 1..=26 (File..TypeParameter);
  the honest test asserts the value set.
- **`marley_app::editor_symbols`** (pure): `SymbolQuery` (the query stale key); `symbol_kind_glyph`
  (SymbolKind→glyph, SEPARATE from `kind_glyph`); `cap_with_tail(len, max)` → `(kept, dropped)` for the
  "+N more" tail.
- **`app.rs` shim**: `OpenSymbols{finder: FinderState, results}` (reusing the ⌘P finder recipe for the
  query+selection); ⌘T → `open_symbol_finder` (capability-gated, quiet flash if none); `park_symbol_query`
  + `consume_symbol_query` (pump — the debounced fan-out to EVERY Ready host); the `WorkspaceSymbol` drain
  arm → `apply_workspace_symbol_response` (query-key guard → parse + MERGE-append); `handle_symbols_key`
  (Esc/Enter-jump/↑↓/backspace/printable-re-query, a MODAL owning text input); `jump_to_symbol` (the #312
  `jump_to_definition` pattern — encoding-aware offset + verify-the-landing open + NavStack push on
  landing); `symbols_overlay` (a centered card, rows windowed by `popup_window`, `symbol_kind_glyph` +
  `truncate_cols`, the "+N more" tail) + its render hookup; `text_input_blocked` + the launcher-choke
  clear; test hooks.
- **`lsp_host.rs`**: `RequestPurpose::WorkspaceSymbol(SymbolQuery)`; `workspace_symbol_support()` accessor
  (+ sibling mutants::skip).
- **`keymap.rs`**: ⌘T **Editor-scoped** → `workspace-symbols` (SHADOWS the global new-tab); roster 57→58,
  scoped 16→17 (the ⌘D shadow pattern — both the global new-tab and the Editor symbols row present).

### Deviations from design (with reason)
- **The row render is self-contained** (`kind_glyph`-analog `symbol_kind_glyph` + `truncate_cols`) — #304's
  in-file symbols are NOT implemented (the ticket assumed the sibling existed), so there is no #304 module
  to share; the primitives (`truncate_cols`, `popup_window`, `menu_origin`) are reused directly.
- **⌘T Editor-scoped shadow** (the D-CHORD decision) — the ticket assumed ⌘T free; it is Marley's new-tab
  (M10 #170). Resolved via the #265 context-scope, exactly like ⌘D. Flagged for chad; a one-line change.
- **The merge stores ALL rows** (no truncate in apply); the RENDER caps at 64 via `cap_with_tail`, so the
  "+N more" count is honest (truncating in apply would lose the dropped count).

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect
Three parallel critics (correctness / reuse+guards+provenance / state+merge+modal) + my own pass. Strong
convergence — TWO HIGHs (one I'd missed), plus merge/cap MEDs. Post-fix: `cargo check` + `cargo clippy
--lib` clean; workspace 1341/1341, no regression.

### REAL — fixed in #325
| # | Sev | Finding | Fix |
|---|---|---|---|
| **F-COMPLETION-STEALS** | HIGH | ⌘T pressed mid-completion left `completion_menu` open — its key arm runs BEFORE the picker's, so ↑/↓/Enter drove the COMPLETION list and Enter INSERTED a candidate into the buffer behind the picker (the "no key leaks" invariant broken). | `open_symbol_finder` dismisses `completion_menu` + `completion` first, so the modal truly owns the keyboard. |
| **F-CAP-DIVERGENCE** | HIGH | `results` was uncapped but the overlay capped at 64, and `move_down`/Enter used `results.len()` — so ↓ past row 63 walked an INVISIBLE cursor and Enter jumped to an UNSEEN symbol (open parks `""` → the whole workspace comes back, so >64 is the common case). | A shared `editor_symbols::MAX_SYMBOL_ROWS`; `move_down` + Enter clamp to `cap_with_tail(len, MAX).0`; the overlay uses the same const — navigable set == shown set. |
| **F-DUP-ROWS** | MED | the stale key was the query STRING only → two concurrent in-flight fan-outs of the SAME string (type `par` → backspace → `par` while the first is slow) both passed the guard and merged → DUPLICATE rows. | `SymbolQuery` gains a monotonic `gen` (bumped per fan-out); an older answer's `gen` mismatches → dropped; `symbol_request` also cleared on Esc/Enter close. |
| **F-UNBOUNDED** | MED | the merged store grew uncapped (only the render capped) — a big workspace × many roots stored every host's full answer. | `apply` truncates the merged store to `MAX_SYMBOL_ROWS * 8` (a runaway guard; realistic server totals stay well under, so the "+N more" tail stays honest). |
| **F-LINE-OVERFLOW** | LOW | `r.line + 1` in the overlay could overflow on a hostile `u32::MAX` line (panic in debug / wrap in release) — breaking the seam's "never panic" contract at the render. | `r.line.saturating_add(1)`. |
| **F-CROSS-ROOT-ENC** | LOW | `jump_to_symbol` used the ACTIVE project's encoding, but the fan-out merges symbols from every root — a symbol from another root would be offset-mapped with the wrong encoding (dormant: all hosts negotiate utf-16). | Resolve the encoding from the DEEPEST root that prefixes the symbol's path (the owning host). |
| **F-CAP-COMMENT** | LOW | the handshake comment overclaimed — only 14 of the 26 advertised kinds get a distinct glyph (the rest fall back to the dot). | Reworded: "the kinds the client ACCEPTS; an unmapped kind → a neutral dot (the LSP-3.17 graceful default)". |

### REAL — accepted / deferred (with reason)
- **MED "the two pure seams have zero tests"** (both critics, self-caveated) — EXPECTED at inspect: the
  wired-but-unused hooks (`drive_symbols_for_test`/`symbols_for_test`) are the pre-validate handoff shape.
  Validate adds the cov/MSI-100 suites (both parse shapes + missing-range + malformed-drop, every
  `symbol_kind_glyph` arm, `cap_with_tail` boundaries, the `SymbolQuery` gen key, a query→merge→stale-drop
  driven test).

### VERIFIED CLEAN (independently, by the critics)
- **Guard inheritance** — `jump_to_symbol` inherits #312's guards WHOLE (verify-the-landing open,
  encoding-aware `position_to_offset`, NavStack push ONLY on landing); the "not a local file" guard moved
  EARLIER to parse-time (`filter_map` drops an undecodable uri before it reaches the picker).
- **Capability upgrade-not-enable** — `workspace_symbol_support` reads the SERVER's cap, independent of our
  advertised `valueSet`; the feature works if the server ignores it.
- **`mutants::skip`** — the new `workspace_symbol_support` accessor + all 7 shims carry it; no pure fn does.
- **Reuse** — `FinderState` reused for the query state, `truncate_cols`/`popup_window`/`menu_origin` for the
  render; `symbol_kind_glyph` correctly SEPARATE from `kind_glyph` (every arm verified vs the LSP
  `SymbolKind` enum — genuinely divergent from `CompletionItemKind`, a shared table would mis-glyph).
- **The ⌘T shadow** — Editor-scoped shadowing the global new-tab (the ⌘D #265 pattern); roster consistent
  (58 chords, 17 scoped, both ⌘T rows pinned).
- **Modal wiring** — every key `stop_propagation`'d; `open_symbols` in `text_input_blocked` (no double
  insert); the debounce coalesces (park overwrites, consume takes once); a late answer after close is
  dropped by the `open_symbols.as_mut()` guard.
- **Provenance** — clean-room LSP 3.17, no `lsp-types` structural fingerprint, no secrets.

### Prevention rules recorded
- **`PR-claude-display-cap-must-equal-navigation-cap-001`** (prevents `BF-symbol-picker-render-cap-
  diverges-from-nav-001`) — a picker that CAPS its displayed rows must apply the SAME cap to navigation +
  selection + accept, or ↓/Enter address rows the user can't see (a render-only cap is a silent
  wrong-target bug).
- **`PR-claude-modal-open-dismisses-lower-overlay-keys-001`** (prevents `BF-modal-open-over-live-
  completion-steals-keys-001`) — opening a MODAL that owns the keyboard must FIRST dismiss any overlay
  whose key arm runs before it (a live completion popup steals the modal's ↑/↓/Enter) — the sibling of
  #324's F-CHOKE.
- **`PR-claude-per-keystroke-refetch-needs-a-generation-001`** (prevents `BF-query-keyed-refetch-
  duplicates-same-string-001`) — a per-keystroke server re-query keyed only on the query string can't tell
  two concurrent same-string fan-outs apart → duplicate merges; carry a monotonic generation.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

### Tests written (13 new, all green)
- **`marley_lsp::workspace_symbol` — 4 pure units** (cov/MSI target 100): `parse_both_shapes`
  (SymbolInformation + WorkspaceSymbol + a deferred-range WorkspaceSymbol → line 0), `parse_skips_
  malformed` (no-name / undecodable-uri / no-location / non-object dropped; non-array → empty),
  `workspace_symbol_params_shape`, `workspace_symbol_support_table`.
- **`marley_app::editor_symbols` — 3 pure units**: `symbol_query_key` (a differing query OR a bumped `gen`
  is a different key — the F-DUP-ROWS supersede), `symbol_kind_glyph_arms` (every SymbolKind class + the
  neutral-dot fallback), `cap_with_tail_boundaries`.
- **`marley_app` headless drives — 6**: `sym_query_merges_and_stale_dropped` (REQ-001/003 — a response
  merges, a stale-query answer drops), `sym_multi_root_merges` (REQ-007 — two hosts' answers accumulate),
  `sym_enter_jumps_and_navstack` (REQ-005 — Enter opens the symbol's file at its line + pushes the
  NavStack), `sym_no_host_flashes` (REQ-006), `sym_cap_clamps_navigation` (**inspect F-CAP** — 100 rows,
  ↓×90 clamps `selected` to 63, never past the visible cap), `sym_open_dismisses_live_completion`
  (**inspect F-COMPLETION-STEALS** — ⌘T dismisses a live completion popup so the modal owns the keyboard).
- `cargo nextest -p marley_lsp workspace_symbol` 4/4; `-p marley editor_symbols` 3/3; the 6 drives 6/6;
  `cargo clippy --all-targets` clean (all test hooks used); `cargo fmt` clean.

### LIVE DRIVE — ENV-BLOCKED (locked screen), fell back to units + mechanism
Bundled the release `Marley.app` (#325) + seeded the `ca-fixture` project with `fn main`/`fn helper`
symbols for a ⌘T drive. On launch the screen had **LOCKED** (`CGSSessionScreenIsLocked=1`, confirmed twice
5 s apart — chad's screensaver engaged while AFK). A locked mac fully blocks synthetic CGEvents +
`screencapture` (`PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism`); I did NOT
attempt a password. App quit, chad's `~/.marley` restored clean.

**Coverage without the live drive** (the honest fallback, the #204/#205 precedent): the picker + jump are
NOT novel render/nav code — Enter reuses #312's **live-proven** `jump_to_definition` (open_and_place_caret
+ NavStack + encoding, driven live in #312), the modal reuses the **live-proven** ⌘P finder recipe +
`menu_origin`/`popup_window`/`truncate_cols` render, and the parse of BOTH shapes is unit-proven. The 6
headless drives exercise the REAL RootView through the actual query→fan-out→merge→Enter→jump path
(including the two inspect fixes). The one residual the live tee would add — the real `workspace/symbol`
wire frame carrying the typed query — is unit-proven at the `workspace_symbol_params` seam + headless-
proven at the fan-out decision. **Re-run the ⌘T live drive when the screen is unlocked** (~2 min via the
reusable harness; a `sym-02`-style capture of `main` → Enter-lands) — no ticket, a confidence top-up.

### Gate — GREEN [diff]
`scripts/gates.sh --diff` → **`GATE GREEN [diff]`, 15 passed / 0 failed** (attempt 1, no PTY-coverage
stall). gate:4 coverage **100% lines**, gate:5 mutation **MSI 100%** (the `workspace_symbol` +
`editor_symbols` pure seams), miri + visual green. Receipt written for `/commit`.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete
- **CHANGELOG** — added the #325 entry above #324 (Added): ⌘T server-filtered picker over `workspace/symbol`;
  the pure `workspace_symbol.rs` (both shapes, deferred-range → line 0); the app `editor_symbols` (`SymbolQuery`
  gen key, `symbol_kind_glyph`, `cap_with_tail`, `MAX_SYMBOL_ROWS`); the Editor-scoped ⌘T shadow; the
  `workspace.symbol.symbolKind` UPGRADE posture.
- **Architecture docs** — `editor.md` gained a "Workspace symbols (#325) SHIP" paragraph after the #324
  signature-help block (the modal-vs-passive-card contrast, the gen stale-key, the one-cap-bounds-both rule,
  the #312 Enter reuse, the two modal lessons, the ⌘T shadow, the cap posture). `crate-map.md`: the `marley_lsp`
  row gained the (#325) workspace-symbol clause + `workspace-symbol` in the SHIM seam list. (The crate-map does
  not enumerate app `editor_*` modules, so `editor_symbols` needs no row.)
- **Knowledge captured (forge wired §19)** — AAR `d18ca6f9` CLOSED (`outcome: completed`, effectiveness 5, 6
  novel findings, jobs: distillation + confidence_drift + pattern_emergence). The 3 failures + 3 prevention
  rules were recorded at inspect (re-record at complete returned conflict/duplicate — confirming they exist):
  `BF-symbol-picker-render-cap-diverges-from-nav-001`, `BF-modal-open-over-live-completion-steals-keys-001`,
  `BF-query-keyed-refetch-duplicates-same-string-001`; `PR-claude-display-cap-must-equal-navigation-cap-001`,
  `PR-claude-modal-open-dismisses-lower-overlay-keys-001`, `PR-claude-per-keystroke-refetch-needs-a-generation-001`.
  **No new AD** — #325 is composition of established decisions (finder.rs modal, #312 nav, #313 park+stale, the
  #265 editor-scoped-shadow, the #323/#324 capability posture); manufacturing an AD for reuse would be noise.
- **Ticket closed** — forge #325 (`e3000b55`) → `done`; local `TICKET-325` status → closed, moved to
  `tickets/closed/`.
- **Pipeline archived** — this doc pair moved to `pipeline/completed/`.

status: Phase 5 — Complete PASS
