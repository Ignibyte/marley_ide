# Problems panel, editable form — the diagnostics multibuffer — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-430-problems-multibuffer.md
- **Pipeline spec:** 430-problems-multibuffer.spec.md

## Phase 1 — Plan
- **Request:** TICKET-430 — the B-c chain's step 6 (M32): the #327 problems
  panel's editable form — diagnostics materialized as an editable multibuffer,
  refresh-on-republish, jump preserved. Auto-approved sprint (Chad's "/work
  425 to 430"); predecessors 427/428/429 all completed pipelines.
- **Classification / tier:** work pipeline, one shippable slice (builder +
  band + refresh wiring over shipped machinery). React-first (zone B).
- **Recall (§18.3):**
  - #327's F: selection-keep identity must be the FULL tuple (path, line,
    character, severity) — two diagnostics can share a line; any 430 keep/
    re-find logic inherits the rule (PR at prevention-rules:797).
  - PR:747 — a NEW pure source file must be `git add -N`-staged before the
    `--diff` gate or cargo-mutants silently skips it (false MSI green).
    REQ-006 bakes this in.
  - The #331/#327 latch lessons: live-refresh surfaces must never wedge on a
    stale set; refresh identity + the two-map column rule (code spans vs
    caret) if bands ever carry spans.
  - #310/#312: diagnostic positions are RAW negotiated encoding; the ONLY
    caret path is the existing bridge (D5). The store is canonically
    PathBuf-keyed (#308 F1).
  - The B-c chain's own notes (427/428/429): the model's build shape
    (`SourceFile = (path, &[(row,col,len)], truncated)` + injected source),
    the #428 target lifecycle, the #429 fail-closed gesture rule
    (F-claude-429-a / PR-claude-429-a).
- **Discovery:** `marley_lsp::problem_rows` (path/line/character/severity/
  message, LSP lane live; cap + dropped) and `DiagnosticStore::iter()` feed
  the aggregation today at `refresh_problems` (app.rs ~16595). The panel is
  `open_problems` (modal, ⌘⇧M, `open_problems_finder` at app.rs ~16578).
  The mb builder seam is `multibuffer.rs build(query, files, dropped,
  source)` with CONTEXT_LINES windows + merge. POC: `overlays/
  ProblemsPanel.tsx` + `views/MultibufferView.tsx` exist; the parity map rows
  are in MARLEY-PARITY.md (Problems | 15; MultibufferView row).
- **Prior-art sweep:** recorded in the spec — Zed behavior map (BlockMap:
  diagnostics rows are blocks; headers are blocks), LSP publishDiagnostics
  full-replace semantics (protocol guarantees "fixed → gone on republish"),
  and the owned-substrate verdict: no external crate owns the seam; the
  builder rides our own shipped modules (the sweep's win is that the chain
  itself is the prior art).
- **Decisions:** D1–D5 locked in the spec (reuse over machinery; jump-list
  stays; refresh never wedges/never closes; fail closed; one encoding
  bridge). The refresh POLICY and the band's slot form are explicitly Phase 2
  calls bounded by D3/REQ-004.

## Phase 2 — Design

### Architecture (§20 confirmed: Zed Project Diagnostics BEHAVIOR via our own
shipped model — maps + published material only, no copyleft source read)

**D-BAND — notes ride the ExcerptLine, not new slots.** `locate` is pure
arithmetic over per-file cum sums (header=+0, line=+1+li); interleaving a new
Row variant would ripple through cum/locate/movers AND the #428 live index.
Instead: per-file, per-MATCH metadata — `FileExcerpts.match_meta:
Vec<(Severity, String)>` parallel (ascending (line,char) order) to the file's
matches — and each `ExcerptLine` gains `notes: Vec<usize>` (indices into
match_meta), attached at build AND at the #428 live rebuild to the line
carrying that match's span (derived from spans, so rebuild-safe; a line with
two diagnostics stacks two bands). Slot count/indices/movers/edit machinery
UNTOUCHED — a noted slot just renders taller (band(s) above the code line;
the render is a flex column, not height-uniform). Search/#429 pass empty meta
→ zero behavior change (build becomes a thin wrapper over build_with_meta).

**D-BUILDER — `build_problems(rows, dropped, source)`** (multibuffer.rs)
maps the aggregated `ProblemRow`s: group per canonical path → sort per file
by (line, character) → matches `(line, char→col, 1)` + meta `(severity,
message)`; delegates to build_with_meta. The pure row→per-file grouping fn
lives in editor_problems.rs (tested there). The model gains `kind: MbKind
{Search, Problems}` — the Problems surface is a SINGLETON: materialize with
one open focuses + force-rebuilds it instead of stacking tabs. Tab title
constant "Problems"; footer noun generalizes ("N problems in M files" — a
noun-aware footer variant, same accounting incl. the cap's "+K more" via
problem_rows' dropped).

**D-COL-ENCODING** — ProblemRow.character is RAW negotiated units; the wash
span col converts through the SAME raw→char helper the #310/#327 jump path
uses if it is pure-text-callable (implement greps and reuses; D5 forbids a
new conversion path). If the existing helper needs a live buffer, v1 clamps
character-as-usize for the WASH ONLY (whole-row wash makes the col nearly
invisible) and records it; the JUMP is `jump_to_problem` VERBATIM (always
bridge-correct).

**D-REFRESH-QUIESCENT (D3-bounded).** The diagnostic SET is a snapshot (like
search results); the pump tick — which already calls refresh_problems(false)
behind the cheap `workspace_diag_total` fingerprint — gains the mb arm: WHEN
the problems mb is open AND the fingerprint moved AND NO mb target is dirty
(quiescent — not mid-edit), the surface re-materializes IN PLACE (same tab/
ContentId: release pins, rebuild model via build_problems, re-pin/re-mint).
Typing (dirty) HOLDS refresh — no caret loss mid-edit; ⌘S (clean) → the
publish lands → next tick rebuilds → the fixed excerpt is gone (REQ-004's
scenario exactly). Stateless re-derivation each time — no latch to wedge
(#331/#327 lessons); a re-appearing diagnostic re-enters at the next rebuild
(never lies). The mb ⌘S outcome ALSO arms a pending-refresh flag consumed at
the next tick, covering the fingerprint total-collision (one fixed + one new
= unmoved sum). ALL resolved → the model goes empty and the surface renders
an empty-state line ("All problems resolved") — the tab NEVER closes itself.
Journal/batch entries for released targets drain via the #428 depth guards +
batches_invalidate (no new mechanism).

**D-ENTRY** — panel open + ⌘⏎ (the #427 overlay chord): a new
handle_problems_key arm BEFORE "enter"; panel hint gains "⌘⏎ edit as
multibuffer". Fail closed (REQ-005): 0 rows → flash "No problems to
materialize", panel stays; a materialize whose model sources zero rows →
flash + bail (the #429 rule).

### File manifest
marley-web FIRST (approved visually before Rust):
- `views/MultibufferView.tsx` — data grows per-line notes (glyph, message,
  severity tint); note bands render above flagged lines; empty-state line.
- `overlays/ProblemsPanel.tsx` — the "⌘⏎ edit as multibuffer" hint + key arm.
- `pages/Workspace.tsx` — wire problems→multibuffer with a mock payload.
Rust:
- `crates/marley_app/src/multibuffer.rs` — MbKind on the model;
  `match_meta`/`ExcerptLine.notes`; `build_with_meta` (+ build wrapper);
  `build_problems`; rebuild re-attaches notes; footer noun variant;
  empty-model state.
- `crates/marley_app/src/editor_problems.rs` — pure `group_problem_files`
  (rows → per-file sorted matches+meta) + the quiescent-refresh decision fn.
- `crates/marley_app/src/app.rs` — ⌘⏎ arm in handle_problems_key;
  `open_multibuffer_from_problems` (singleton focus/force-rebuild path);
  the pump-tick mb refresh arm + the ⌘S arming; note-band + empty-state
  render in multibuffer_body; test hooks.
- `crates/marley_app/src/tabs.rs` — `PROBLEMS_TAB_TITLE`.
- `crates/marley_app/src/headless_drive.rs` — the drives below.

### Regression Test Plan
| REQ | Test |
|---|---|
| REQ-001 | drive `problems_materialize_multibuffer_headless`: seed 2 files, feed diagnostics via `push_diagnostics_for_test` (+ one leg through `ingest_message_for_test`'s real publish), ⌘⇧M → ⌘⏎ → assert tab "Problems", 2 headers, per-diagnostic note bands (glyph+message), panel closed |
| REQ-002 | same drive: place caret (hook), type, assert write-through to the registry buffer + ⌘Z journals |
| REQ-003 | drive `problems_mb_jump_headless`: ⌘⏎ on a noted line → file open, caret at the bridge position, NavStack ⌃- returns |
| REQ-004 | drive `problems_mb_refresh_quiescent_headless`: republish REMOVING one diag while a target is DIRTY → no rebuild (caret preserved); ⌘S → clean → tick → rebuilt WITHOUT that excerpt, other intact, tab open; republish removing ALL → empty-state, tab open |
| REQ-005 | drive/unit: 0 problems → ⌘⏎ flashes + no tab, panel stays |
| REQ-006 | gate `--diff` green (any NEW file `git add -N` before it — PR:747) |
| units | `group_problem_files` (grouping, per-file (line,char) sort, meta parallel order, multi-diag one line), build_problems (cap/dropped pass-through, row-past-end drop, col clamp), notes re-attach across rebuild, MbKind singleton find, footer noun, quiescent decision table (dirty×fingerprint×armed) |
| parity | POC problems-mb capture ↔ live Marley same state; pixel-sample note band bg/glyph tints + header/wash families per MARLEY-PARITY.md |

### Risks / decisions
- In-place re-materialize touches the #427-b symmetric-release class — reuse
  the close path's release before re-pinning (one recipe, no new lifecycle).
- Fingerprint sum-collision bounded by the ⌘S arming; residual staleness
  (external fix + collision, no mb save) clears on the next fingerprint move
  — recorded, acceptable v1.
- Wash-col encoding approximation (if no pure helper exists) — wash-only,
  jump always exact; recorded above.
- rust-analyzer's per-keystroke republish storm is absorbed by the dirty
  hold + fingerprint gate (no rebuild while typing, cheap ticks otherwise).

## Phase 3 — Implement

**POC first (approved):** `MultibufferView.tsx` grew `kind` + per-line
`notes` (glyph/message/danger bands above flagged lines), the Problems chip,
the problems footer noun, and the all-resolved empty state; `ProblemsPanel
.tsx` grew the "⌘⏎ edit as multibuffer" hint + the ⌘⏎ arm building the
diagnostics payload (`problemsMultibuffer` — group per path, sort by line,
±2 context) fail-closed on empty; `Workspace.tsx` wires onMaterialize.
Typecheck green; driven ⌘⇧M → ⌘⏎ at localhost:5173; `430-poc-mb.png`
(in .playwright-mcp/) READ: 4 file groups, washed diagnostic lines, bands
with ▲/○/· glyphs + messages, "4 problems in 4 files" footer. The stale-
closure lesson applied up front (deps include rows/onMaterialize).

**Rust port:**
- `multibuffer.rs` — ExcerptLine.notes (match indices), NoteMeta
  (glyph/message/danger), MbKind on the model, `build_with_meta` (build is a
  thin Search wrapper; per-file meta filtered in step with the past-EOF match
  drop so meta stays parallel to surviving matches), rebuild_lines re-attaches
  notes by anchor order, footer noun branches on kind.
- **DEVIATION from D-BAND (recorded):** the design said "notes ride the
  ExcerptLine, not new slots" — but the render is `uniform_list`, which
  REQUIRES uniform row heights; an in-slot band would corrupt scroll math.
  The band is therefore a real slot: `Row::Note(fi, mi)`, and the model's
  private prefix-sum `cum` was REPLACED by a materialized `slots: Vec<Row>`
  (locate = index, slot_of_row = scan, refresh_cum builds it; Note slots mint
  ONLY where match_meta has content, so search layouts are byte-identical —
  proven by the 21 mb units + 44 mb-flavored drives all green unchanged).
  Pre-select now scans for the first Line slot (a leading band must not be
  selected); movers/line_at/mb_place_caret skip Note like Header.
- `editor_problems.rs` — `group_problem_files` (first-appearance file order =
  severity-first reading order; per-file (line,character) sort; meta parallel),
  `build_problems` (sources each file ONCE, converts raw (line,character) →
  char (row,col,1) via `marley_lsp::position_to_offset` with the file's
  negotiated encoding — D5's one conversion path; unsourceable files keep raw
  casts and are skipped by the builder), `mb_refresh_due` (the quiescent
  decision: (moved || armed) && !dirty).
- `app.rs` — pin loop EXTRACTED to `pin_mb_targets` (shared by search
  materialize, problems materialize, refresh); `build_problems_model` (live-
  buffer-first source + jump_to_problem's host-ranked encoding);
  `find_problems_mb_tab` (singleton per project); `replace_mb_model` (new
  pins acquired BEFORE old release — the F-claude-427-b dip guard);
  `open_multibuffer_from_problems` (⌘⏎ arm in handle_problems_key BEFORE
  plain Enter; empty set / zero-sourced flash-and-bail with the panel open);
  `refresh_problems_mb` on the pump tick (quiescent re-materialize in place;
  all-resolved leaves the EMPTY model + "All problems resolved" in the footer
  slot; tab never closes); mb_save_all ARMS the refresh on a Problems-kind
  save; the Note-slot band render (gutter-aligned glyph + message, danger
  tint via colors.danger); the panel title hint. Fields
  `problems_mb_fingerprint`/`problems_mb_armed` + init.
- `tabs.rs` — `PROBLEMS_TAB_TITLE`.

Workspace check + fmt clean; mb units 21/21 and mb drives 44/44 green.

## Phase 3.5 — Inspect

**Self-review (run inline while the critics worked):**
1. **MED (REAL, fixed) — cross-project live-text hole.** `build_problems_model`
   copied the search materialize's `editors_under(active root)` override map,
   but `problem_rows` aggregates EVERY host — a diagnostic in another
   project's file would source STALE DISK text while its live buffer differed.
   Fix: the overrides span ALL open editors (`content.iter()` filter-map),
   any root. (The search surface keeps `editors_under` — its walk is
   root-scoped by construction.)
2. **Verified clean — the sync pairing.** `sync_multibuffer_live` rebuilds
   `f.lines` then calls `refresh_cum()` (app.rs ~14105), so Note slots
   re-derive with every live rebuild; `rebuild_lines` numbers notes by the
   anchor order the meta is parallel to (anchors cannot reorder).
3. **Verified clean — the conversion clamps.** `position_to_offset` never
   panics: line past EOF → buffer end on BOTH calls → col 0; character past
   line end → line end (doc + body read).
4. **Recorded (by design) — refresh resets mb-side gesture state.** A
   quiescent re-materialize swaps the whole model: journal/batches/touched/
   caret reset (snapshot semantics; the per-buffer undo in each file's own
   tab is untouched). Dirty targets BLOCK refresh, so this only occurs at
   clean points.
5. **HARDENED — replace_mb_model's vanished-id arm.** If the content id raced
   away between find and swap, the NEW model's freshly-pinned views would
   have dropped un-released. Unreachable today (both callers are synchronous
   from find to swap), but the F-claude-427-b class demands the symmetric
   release — the None arm now releases the new pins (belt only, commented).
6. **Verified clean (each traced in code):** meta-parallel bookkeeping in
   build_with_meta (kept-indexed notes vs original-mi meta lookup; a short
   meta only ever truncates the SUFFIX and refresh_cum's `mi < match_meta
   .len()` guard skips those bands); the ⌘⏎ arm order in handle_problems_key
   (before plain Enter, nothing earlier consumes it); armed/fingerprint
   lifecycle (cleared at materialize + refresh; a closed tab leaves armed
   inert — refresh requires the tab); mb_live_versions scrubbed in the swap
   arm; group_problem_files determinism (input already sorted; sort_by_key is
   stable, same-position diagnostics keep severity order); gutter_label(0,5)
   spacer width == a real row's 5-char gutter; POC keydown deps carry
   rows/onMaterialize (the L-claude-429 stale-closure rule); notes render
   only on problems payloads POC-side; the band bg tokens are the pinned
   header-family pair (bg-card ↔ colors.surface).

**Critic round (honest record):** three independent critics (correctness /
state-integrity / reuse+parity) were spawned over the diff and ran deep
(340–560KB transcripts each, mid-verification with no findings reported yet)
— all three were TERMINATED EXTERNALLY by the account session limit before
composing final reports. The self-review above independently executed each
critic's hunt list in the main session (same lenses, findings 1–6). A
consolidated independent re-verify critic runs during Phase 4 (after the
limit reset) and its verdict is appended here before the gate.

Verify: workspace check clean after both fixes; the mb/problems regression
subset 46/46 green; fix tokens grep-audited (L-claude-428 discipline).

**Re-verify critic (ran during Phase 4, after the limit reset — closing the
§18.1 loop):** three findings, ALL CONFIRMED + fixed at source; three hunts
explicitly refuted-nothing (stale-slot caret feeds fail closed; the Note
render's missing-meta arm unreachable — one model read per frame + whole-model
swaps; acquire/release ordering holds both arms). Confirmed + fixed:
1. **HIGH — cross-project split-brain (the F-claude-430-a class's DEEPER
   layer).** pin_mb_targets/ensure_mb_target blanket-used the ACTIVE root, so
   a cross-project diagnostic's first caret click birthed a SECOND live
   instance from disk under the wrong root — mb edits never reached the
   file's real tab, last save clobbers. Fix: `root_of_mb_path` (longest
   project-root prefix, active fallback) threaded through both; search mbs
   land on the active root unchanged.
2. **HIGH — clippy type_complexity** on build_problems' `converted` binding
   (gate:2 red) — a named `ConvertedFile` alias.
3. **MED — refresh_cum's clamp could strand `selected` on a Header/band**
   after a live rebuild shifted slot blocks (highlight silently vanishes;
   self-healing on the next arrow). Fix: refresh_cum normalizes to the
   NEAREST Line (forward then backward); build's pre-select scan is subsumed
   by it. All 55 mb/problems tests green after the round; clippy
   --all-targets clean.

## Phase 4 — Validate

**Units (6 new, all green):** multibuffer — `build_with_meta_mints_note_slots
_and_keeps_meta_parallel` (past-EOF match drops WITH its meta; slots
H/N/L...; pre-select below the band; problems footer),
`movers_skip_note_slots_and_search_builds_stay_noteless` (adjacent bands;
search layout byte-identical, zero Note slots),
`rebuild_lines_reattaches_note_indices_in_anchor_order`; editor_problems —
`group_problem_files_first_appearance_order_and_line_sort` (severity-stable
same-position pair), `build_problems_converts_encoding_and_passes_dropped`
(emoji UTF-16→char col; unsourceable → skipped), `mb_refresh_due_quiescent
_table`.

**Headless drives (3 new, green):**
- `problems_materialize_edit_and_jump_headless` — REAL publishes through
  feed_lsp_publish_for_test; ⌘⇧M → ⌘⏎: "Problems" tab, severity-first
  groups, band above its line, panel closed; typed write-through + ⌘Z
  journal; Esc → Enter jump to the diagnostic position (char 17) + ⌃-
  returns to the origin. (Two drive-authoring lessons hit: the band sits
  after a CONTEXT line, and with an ARMED caret Enter is a NEWLINE — Esc
  first for the navigate-jump.)
- `problems_mb_refresh_quiescent_headless` — dirty HOLDS (stale excerpts +
  caret preserved); ⌘S → clean+armed → rebuilt WITHOUT the cleared file;
  clearing ALL → empty model, footer "0 problems in 0 files", tab open.
- `problems_materialize_empty_fails_closed_headless` — no tab, panel stays,
  the "No problems" flash.

**Full suite:** `cargo nextest run --workspace` **2241/2241 PASS** +
doctests green (real runs in transcript).

**LIVE drive (scratch HOME + a REAL cargo fixture + REAL rust-analyzer;
RUSTUP_HOME/CARGO_HOME pinned so the scratch HOME doesn't break the shim):**
- `430-live-panel.png` READ: ⌘⇧M panel "2 problems" + the "⌘⏎ edit as
  multibuffer" hint; the ● mismatched-types error selected.
- `430-live-mb.png`/`-mb2.png` READ: the Problems tab — bands ("· expected
  due to this", "● mismatched types / expected `i32`, found `&str`") above
  line 3, true line numbers, wash, header "src/main.rs 2", footer
  "2 problems in 1 files".
- Typed the FIX in place (13⌫ + "42;" — line-end caret): write-through ✓,
  dirty dot ●2 on the band ✓, refresh correctly HELD while dirty ✓.
- **LIVE FINDING (the drive earned its keep):** after ⌘S the refresh NEVER
  fired — rust-analyzer's republish replaced error+hint with
  warning+hint (the post-save flycheck's unused-variable pair): the row-SUM
  fingerprint collided (2→2) and the ⌘S arming had already consumed itself
  BEFORE the async republish landed. The headless drive missed it because
  its publishes are synchronous. **Fix at source:** the gate is now a
  monotone PUBLISH EPOCH — `DiagnosticStore.publish_epoch` (bumped on every
  replace, clears included) → `LspHost::diag_publish_epoch` →
  `workspace_diag_epoch` — so any republish registers, count collisions
  included. The ⌘S arming stays as belt.
- Re-bundled + re-drove: `430-live-refreshed2.png` READ — the refresh fired
  on the republish and the surface re-materialized to the NEW pair (the
  collision case itself, proven live); `430-live-allresolved.png` READ —
  fixing the warning too → **"All problems resolved" + "0 problems in
  0 files"**, the tab still open. The full REQ-004 arc on real pixels.

**Parity pair:** `.playwright-mcp/430-poc-mb.png` (POC, READ at implement)
↔ `430-live-mb2.png`. Pixel-sampled: body (11,12,15)↔(14,15,17), band +
header bg (26,27,31)↔(25,26,28), match wash (17,30,34)↔(18,28,31) — the
pinned Δ≤4 token families; band grammar (glyph + message above the line,
gutter-aligned) matches structurally. Live-only vs POC: the selected wash
(21,37,40) (recorded since #427). Verdict: PASS.

**Gate (honest record — six runs, every red fixed at source):**
1. Run D: MSI 91.3% — 4 missed (the Problems footer's `dropped > 0` arm; the
   epoch trio in marley_lsp — publish_epoch→0/→1, `+=`→`*=`: the L-claude-429-a
   own-crate class again). Killed: the footer "+7 more" both-ways row +
   `publish_epoch_bumps_on_every_replace_including_clears`.
2. Runs E–H: coverage 1–6 missed lines in multibuffer.rs that NO per-line view
   could locate (the #399 masked-reproduction signature). Chased the
   generic-phantom theory first (build/build_with_meta/build_problems/
   rebuild_lines all de-genericized to `&dyn Fn` — kept as belt), then plain-
   loop rewrites EXPLODED the miss from 1 to 6 lines and unmasked the truth:
   the refresh_cum normalize's BACKWARD scan was DEAD CODE by construction
   (a non-empty slot list always ENDS with a Line, so the forward scan always
   finds one). The closure form had hidden the dead arm as one uncovered
   line.
3. Fix: DELETED the backward arm + stated the invariant (#426's re-expression
   discipline); slot_of_row restored to its covered closure form.
4. Run I: **GATE GREEN [diff]** — coverage 100% (0 missed lines), mutation
   **51 caught / 0 missed → MSI 100%** (1 timeout verified caught-by-assert).
   fmt/clippy/docs/audit/deny/machete/gitleaks/shellcheck/no-suppressions/
   SAST/miri/visual all green.

## Phase 5 — Complete

Docs: CHANGELOG entry (430, top of Added); `editor.md` — the "#430 — the
diagnostics form" paragraph (slots/Note, root_of_mb_path, singleton, epoch
refresh, fail-closed), the #327 panel note corrected to shipped, deferred
list trimmed; `roadmap.md` — B-c 6/6 SHIPPED. Parity: MARLEY-PARITY's
ProblemsPanel row grew the #430 ✅ block (hint/arm/empty-set both sides; the
shared MultibufferView surface with pinned pixel families; Rust-only:
epoch refresh, per-file roots, +K tail, singleton). POC matches shipped —
no back-port beyond build-time work.

Knowledge: L-claude-430-a (a closure can hide a dead arm from coverage —
the plain-loop unmasking rule + delete-with-invariant), AD-claude-430
(publish-epoch refresh gate + Note-slot rationale + root discipline).
F-claude-430-a/-b + the inspect ledger were appended earlier.

Ticket closed → tickets/closed/; pipeline pair archived → completed/.
