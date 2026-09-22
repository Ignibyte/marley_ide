# The problems panel (⌘⇧M) — Notes

- **Forge ticket:** #327 898479e5-952e-4e0a-b532-307a7f11398f
- **AAR:** d1fda6ae-20d0-415b-861b-e41a18f6a222
- **Local ticket doc:** docs/planning/tickets/open/TICKET-327-problems-panel.md
- **Pipeline spec:** 327-problems-panel.spec.md

## Phase 1 — Plan
- **Request:** ⌘⇧M problems panel — every workspace diagnostic (incl. closed files) in one severity-sorted
  jumpable read-only list. Add DiagnosticStore::iter() + aggregate across hosts + a pure problem_rows merged
  with the terminal lane + a finder picker + live re-derive + a footer workspace tier.
- **Classification / tier:** FEATURE, single shippable slice (a read-only workspace diagnostics list). Builds
  directly on #310 (the DiagnosticStore + merged_rows + REPLACE/CLEAR) + reuses the #325/#326 finder-picker +
  #312 jump recipe — mostly a new enumeration primitive + a pure row-composer + a picker. No split.
- **Systems involved:** server/ui (the app shim + a new ⌘⇧M overlay + the footer segment), the pure
  marley_lsp diagnostics store (a new iter), a pure app row-composer. No fleet/agents/bridge/settings.
- **Forge recall (§18.3):** no bulletins. AAR d1fda6ae opened (0 pre-flagged). knowledge-search surfaced the
  applicable families (resolved by code): the finder recipe + open_and_place_caret guard inheritance
  (`PR-claude-second-consumer-must-inherit-the-first-consumers-guards`), the #325 cap==nav
  (`PR-claude-display-cap-must-equal-navigation-cap`), the modal-dismisses-lower-overlay
  (`PR-claude-modal-open-dismisses-lower-overlay-keys`), and the live-identity family (dismiss/keep by
  identity not index). Fed to Design.
- **Discovery (seams verified in the tree):**
  - `DiagnosticStore` (diagnostics.rs:74) — private `HashMap<PathBuf, Vec<Diag>>`, only `for_path` (:96).
    ADD `iter()`.
  - `Severity` (diagnostics.rs:25) — `#[derive(PartialOrd, Ord)]`, variants Error/Warning/Info/Hint IN THAT
    ORDER → the Ord already sorts Error-first. `severity_from_lsp` maps LSP (absent/unknown → Error).
  - `LspHost.diagnostics: DiagnosticStore` (lsp_host.rs:100, private) — needs a `diagnostics_iter()`
    passthrough. Replaced/cleared from publishDiagnostics (:442-444), pruned on close (:212), cleared on a
    dead server (:327).
  - The terminal lane: `App::open_file_diagnostic_rows` (app.rs:6026) builds `terminal_rows` (scanning the
    terminal via `links::trace_diagnostic_rows`, links.rs:276) + merges with the FOCUSED file's LSP diags via
    `merged_rows` (app.rs:6063). This is PER-FOCUSED-FILE — D-TERMINAL is how it lifts workspace-wide.
  - `cockpit_status` (status_bar.rs:56) + order tests (:126/:171/:188, incl. a diagnostics-segment-after-lsp
    test) — the workspace tier extends this pure formatter + its order tests.
  - **⌘⇧M is FREE** (no `chord(true,false,false,true,"m")` in keymap.rs) → a NEW Editor-scoped row, shadows
    nothing (the #322/#323/#324 posture).
- **Decisions:** D1–D7 locked (see spec). **D-TERMINAL is the one design-owned decision** — fold the terminal
  failed-block lane into the workspace panel (per-referenced-file, tagged source) vs LSP-only v1 with the
  terminal merge a named follow-up; either way `problem_rows` takes both producers.
- **§20:** behavior-only reference to Zed's ProjectDiagnosticsEditor (our own deconstruction 05-lsp §5); the
  panel composition + finder picker are Marley's own over the shipped-in-#310 publishDiagnostics wire. The
  `[Zed-derived]` multibuffer stays deferred. No GPL source read.

## Phase 2 — Design

### D-TERMINAL DECIDED → LSP-only v1, the terminal-lane merge a named follow-up (the seam stays ready)
The M18 terminal lane (`App::open_file_diagnostic_rows`, app.rs:6026) derives `Vec<usize>` row numbers for the
FOCUSED file only, via `links::trace_diagnostic_rows(output, open_path, root)` — a scan SCOPED to one open
path. Lifting it workspace-wide needs a NEW terminal scan returning every `(path, line)` the scrollback
references (regardless of the focused file) — non-trivial, and LOWER value than the LSP store, which is the
authoritative + complete "every diagnostic rust-analyzer knows". **Decision:** v1 sources the panel from the
LSP `DiagnosticStore` ONLY; `problem_rows` STILL takes a terminal producer (`&[(PathBuf, u32)]`) so the
two-producer doctrine is architecturally present + unit-tested, and the app passes an empty slice v1. The
workspace terminal-scan is a named follow-up (a pure-wiring change once the scan exists). Honest — the panel
title/footer count the LSP diagnostics; a terminal-referenced error not yet an LSP diagnostic is out of v1.

### Architecture / approach — the pure/masked split (pure seams cov/MSI 100; IO/gpui = masked)
1. **PURE `marley_lsp::diagnostics`** (gpui-free, tool-shaped): `DiagnosticStore::iter() -> impl
   Iterator<Item=(&Path, &[Diag])>` (`by_path.iter().map(...)`); `ProblemRow{path: PathBuf, line: u32,
   character: u32, severity: Severity, message: String, source: ProblemSource}` + `enum ProblemSource {Lsp,
   Terminal}`; `problem_rows(lsp: impl Iterator<Item=(&Path,&[Diag])>, terminal: &[(PathBuf,u32)], cap: usize)
   -> (Vec<ProblemRow>, usize)` — flatten each path's Diags into rows (source Lsp) + the terminal pairs
   (source Terminal, severity Error), SORT by `(severity, path, line)` (the `Severity` `Ord` already ranks
   Error<Warning<Info<Hint), then cap → `(kept, dropped)`. `DiagnosticStore::total_len()` (sum of vec lens) —
   the cheap re-derive fingerprint. All cov/MSI 100.
2. **App-pure `marley_app::editor_problems`** (cov/MSI 100): `MAX_PROBLEM_ROWS` (~500); `severity_glyph(sev)`
   (● Error / ▲ Warning / ○ Info / · Hint — the render colors it via the gutter danger/warning palette);
   `visible_problems(total)` = `cap_with_tail(total, MAX_PROBLEM_ROWS)` (cap == nav); `keep_selection_by_
   identity(rows: &[ProblemRow], key: Option<(&Path, u32)>) -> usize` — after a refresh, re-find the row
   whose (path, line) == key → its index (clamped to the new len); `None`/not-found → clamp the old index.
3. **`marley_app::status_bar`**: `workspace_diag_summary(workspace_total, focused_total) -> Option<String>` =
   `(workspace_total != focused_total).then(|| format!("{workspace_total} workspace"))` (pure); `cockpit_
   status` gains a `workspace: Option<String>` param pushed AFTER the focused `diagnostics` segment; the
   order tests extended.
4. **Masked app shim `app.rs`**: `open_problems: Option<OpenProblems>` where `OpenProblems{finder:
   FinderState, rows: Vec<ProblemRow>, dropped: usize, selected_key: Option<(PathBuf,u32)>, diag_fingerprint:
   usize}`. `open_problems_finder` (clears completion + other overlays — the #325 lesson; builds the initial
   rows). `refresh_problems` (rebuild rows via `problem_rows(self.lsp_hosts.values().flat_map(|h|
   h.diagnostics_iter()), &[], MAX)`, keep the selection via `keep_selection_by_identity`). The pump, WHILE
   open, re-derives ONLY when the cheap fingerprint `sum(host.diagnostics_total())` moved (a same-count
   content change is a rare miss fixed at the next publish — noted). `handle_problems_key` (↑/↓ clamp to
   `visible_problems`, Enter, Esc). `jump_to_problem` (reuse #312 `open_and_place_caret` + NavStack — the path
   may be CLOSED, `open_file_in_viewer` opens it; `Utf32`? NO — LSP diags are in the negotiated encoding, so
   resolve the owning host's encoding like #325's `jump_to_symbol`, NOT #326's Utf32). `problems_overlay`
   (`#[cfg_attr(test, mutants::skip)]` — the #326 lesson). Footer: compute `workspace_total` (cheap sum) +
   pass `workspace_diag_summary(...)` to `cockpit_status`. Drain/dispatch/router/text_input_blocked/choke
   arms; `open_problems_for_test`/`push_diag_for_test`/`problems_for_test` hooks.
5. **`lsp_host.rs`**: `diagnostics_iter()` + `diagnostics_total()` passthroughs (lsp_host.rs is gate-excluded).
6. **`keymap.rs`**: ⌘⇧M `chord(true,false,false,true,"m")` Editor-scoped → "problems-panel" (FREE — shadows
   nothing); roster 59→60, scoped 18→19.

### §20 confirm
Behavior-only reference to Zed's ProjectDiagnosticsEditor (our own deconstruction 05-lsp §5) — reimplemented
via the `finder.rs` picker + the #310 `DiagnosticStore` + the #312 jump over the shipped-in-#310
publishDiagnostics wire. The `[Zed-derived]` multibuffer stays deferred. No GPL source read. Holds.

### File manifest
- `crates/marley_lsp/src/diagnostics.rs` — ADD `DiagnosticStore::iter`, `total_len`, `ProblemRow`,
  `ProblemSource`, `problem_rows`. cov/MSI 100.
- `crates/marley_lsp/src/lib.rs` — re-export `ProblemRow`, `ProblemSource`, `problem_rows`.
- `crates/marley_app/src/editor_problems.rs` — NEW pure: `MAX_PROBLEM_ROWS`, `severity_glyph`,
  `visible_problems`, `keep_selection_by_identity`. cov/MSI 100.
- `crates/marley_app/src/lib.rs` — `mod editor_problems;`.
- `crates/marley_app/src/status_bar.rs` — `workspace_diag_summary` + `cockpit_status` workspace param + order
  tests.
- `crates/marley_app/src/lsp_host.rs` — `diagnostics_iter()` + `diagnostics_total()` passthroughs.
- `crates/marley_app/src/app.rs` — the `open_problems` shim (fields/struct/open/refresh/pump/keys/jump/overlay/
  footer/arms/test hooks).
- `crates/marley_app/src/keymap.rs` — ⌘⇧M Editor-scoped + roster guards.
- `crates/marley_app/src/headless_drive.rs` — the drives.
- Phase 5: `CHANGELOG.md`, `editor.md`, `crate-map.md`.

### Regression Test Plan (≥1 per REQ)
| REQ | Test(s) | Kind |
|---|---|---|
| 001 | `problems_open_and_esc_close` (drive) | headless + LIVE(fallback) |
| 002 | `diagnostic_store_iter` (empty + populated) | pure unit cov/MSI 100 |
| 003 | `problem_rows_aggregate_sort_cap`, `problem_rows_two_producer_merge` | pure unit cov/MSI 100 |
| 004 | `problem_rows_includes_closed_paths` (unit) + `problems_lists_across_files` (drive) | pure + headless |
| 005 | `problems_enter_opens_closed_file_and_navstack` (drive) | headless + review |
| 006 | `keep_selection_by_identity` (unit) + `problems_refresh_keeps_selection` (drive) | pure + headless |
| 007 | `problems_multi_host_merge` (drive) | headless (2-host) |
| 008 | `workspace_diag_summary`, `cockpit_status_workspace_tier_order` | pure unit |
| 009 | `visible_problems_cap`, `problems_cap_clamps_navigation` (drive) | pure unit + headless |

LIVE drive (env-fallback if locked): two probe files each with a planted error (one CLOSED) → ⌘⇧M lists
both → Enter on the closed row opens it at the squiggle → fix one + save → its row drops on the next publish.

### Risks / decisions (load-bearing)
- **D-TERMINAL = LSP-only v1** (the seam takes a terminal producer, app passes empty; workspace terminal-scan
  a follow-up) — the panel is LSP-complete; a terminal-only reference is out of v1 (stated).
- **Pump re-derive gated on a cheap `total_len` fingerprint** (not every frame) — a same-count content change
  is a rare miss the next publish fixes (noted; a diag-generation counter is the exact-gate follow-up).
- **jump_to_problem resolves the OWNING host's encoding** (LSP diags are negotiated-encoding, NOT char
  offsets — the #325 posture, NOT #326's Utf32). Load-bearing: a wrong encoding shifts the caret on a
  non-ASCII line.
- **Selection kept by identity (path, line)** — a refresh re-finds the row, never teleports by index.
- **ProblemRow owns path + message** — for the app hold + the tool-shaped MCP surface.

## Phase 3 — Implement
- **Built to the manifest.** `marley_lsp/diagnostics.rs`: `DiagnosticStore::iter` + `total_len`, `ProblemRow`
  + `ProblemSource`, `problem_rows` (flatten LSP → rows + terminal → rows, sort `(severity, path, line)`, cap
  → `(kept, dropped)`); lib.rs re-exports. `marley_app`: `editor_problems.rs` (`MAX_PROBLEM_ROWS`,
  `severity_glyph`, `visible_problems`, `keep_selection_by_identity`); `status_bar.rs` (`workspace_diag_
  summary` + `cockpit_status` `workspace` param + the 4 existing order-test call sites updated); `lsp_host.rs`
  (`diagnostics_iter`/`diagnostics_total` + a `push_diagnostics_for_test`); `app.rs` (the `open_problems`
  field + `OpenProblems` struct + `open_problems_finder`/`refresh_problems`/`handle_problems_key`/
  `jump_to_problem`/`problems_overlay` + `active_workspace_diag_summary` for the footer + the 6 wiring arms +
  3 test hooks); `lib.rs` `mod editor_problems`; `keymap.rs` ⌘⇧M Editor-scoped + roster guards (60/19).
- **Compile:** `cargo check --workspace --all-targets` clean — only the expected "3 test hooks + push_
  diagnostics_for_test never used" (validate wires them) + the pre-existing `block v0.1.6`. `cargo fmt` clean.
  No new dependency (all marley_lsp/marley_app additions are pure Rust).
- **Deviations from design (with reason):**
  1. **`OpenProblems` uses a plain `selected: usize`, not `FinderState`** — the panel has NO query filter v1,
     so FinderState's query would be dead; a plain index + inline ↑/↓ clamp is simpler and matches the render.
  2. **`refresh_problems(force: bool)`** — `force=true` on open (build even from the 0 fingerprint), the pump
     calls it `force=false` (rebuild only when the cheap `sum(diagnostics_total)` fingerprint moved).
  3. **`jump_to_problem` mirrors `jump_to_symbol`** (resolves the owning host's encoding via the deepest root
     prefixing the path) — LSP diags are in the negotiated encoding, NOT char offsets (the load-bearing #325
     posture, not #326's Utf32).
  4. **Severity color reuses the gutter mapping** (Error→`colors.danger`, else→`colors.muted`) — the theme
     has no distinct warning color; the glyph (● ▲ ○ ·) carries the finer distinction.
  5. **The terminal producer is wired-but-empty** (D-TERMINAL): `problem_rows` takes `&[(PathBuf,u32)]`
     (source Terminal, severity Error, message "failed here (terminal)"), the app passes `&[]` v1 — the
     merge logic is unit-tested, the workspace terminal-scan is the named follow-up.
- **Coverage/mutation reminder for Validate:** the pure `diagnostics` additions + `editor_problems` +
  `status_bar` additions need cov/MSI 100 via units; the app.rs shim + `problems_overlay` (mutants::skip) are
  gate-excluded, behavior-verified by headless drives. `problems_overlay` HAS its `#[cfg_attr(test,
  mutants::skip)]` (the #326 lesson applied up front).

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Inspect (Phase 3.5)
3 parallel critics (general-purpose) over the diff — lenses: pure-seam correctness, state/live-refresh
integrity, reuse/guards/provenance. **Verdict: SOUND — no HIGH/MED defects.** Critic 3 confirmed all 8
guard/reuse/provenance checks PASS (guard inheritance mirrors jump_to_symbol, owning-host encoding NOT Utf32,
closed-file open, cap==nav, all 5 modal wirings, keymap 60/19 [ran the test], §20 clean-room, problems_overlay
mutants::skip). Three LOW fixes applied at source; the rest are documented tradeoffs.

### CONFIRMED → FIXED at source
- **[LOW · C2 state] Non-unique `(path, line)` selection identity — the un-noted gap.** Two diagnostics can
  share a line (a clippy Error + a Warning, sorted far apart by severity); `keep_selection_by_identity`'s
  `.position()` returned the FIRST `(path,line)` match, so a live refresh teleported the selection (e.g. 30 →
  3) — bounded (lands on a real diag) but undercutting the "identity" guarantee. **Fix:** extended the
  identity key to `(path, line, character, severity)` (the field + `keep_selection_by_identity` sig + all 3
  set-sites). `Severity` is `Copy`, so the 4-tuple stores + compares cleanly.
- **[LOW · C2 simplification] The fingerprint expression was duplicated** at `refresh_problems` (the re-derive
  gate) and `active_workspace_diag_summary` (the footer). **Fix:** extracted `workspace_diag_total()`; both
  now read it, kept in lock-step.
- **[LOW · C1 hardening] The "+N more" sum is correct only because both caps == MAX_PROBLEM_ROWS.** `problem_
  rows` already caps at 500, so `visible_problems`' tail (`more_cap`) is always 0 and the honest tail is
  `open.dropped`. **Fix:** a comment in `problems_overlay` coupling the two caps (a larger `problem_rows` cap
  would double-count).

### CONFIRMED → documented tradeoffs / deferred (not fixing)
- **[LOW · C2/C3] Same-count fingerprint miss** — a REPLACE that keeps the count but moves a diagnostic's
  line (fix-one/create-one, or a cross-file +1/−1) leaves the rows stale until the next count-changing
  publish. Bounded + self-healing; the FOOTER count stays live (it's not fingerprint-gated); a stale jump
  clamps to a valid caret (no crash). Documented in the code + design as the cheap-gate tradeoff. A
  line+severity fingerprint hash is the exact-gate follow-up.
- **[LOW · C1/C2] `focused_total` assumes the focused file is under the ACTIVE project's host** (the footer
  tier). Pre-existing #310 `active_diagnostic_summary` pattern, unusual in the one-focus model. Not a #327
  regression.
- **[INFO · C1/C3] `visible_problems` is belt-and-suspenders** (problem_rows already caps) — harmless; kept
  for the cap==nav guarantee + if the caps ever diverge.
- **[LOW · C1] `character` is not a sort tiebreaker** — two diags at the same (severity, path, line) keep
  server order. Cosmetic; the documented sort key is `(severity, path, line)`.

### VALIDATE follow-through (critic 3 flagged, load-bearing)
The 4 pure seams (`problem_rows`, `visible_problems`, `severity_glyph`, `keep_selection_by_identity`) + the
`workspace_diag_summary`/`DiagnosticStore::iter` MUST reach cov/MSI 100, and the 4 test hooks (`open_problems_
for_test`, `problems_for_test`, `push_diag_for_test`, `lsp_host::push_diagnostics_for_test`) must be wired by
the drives — the `mutants::skip` justifications ("over the TESTED seams") are a promissory note Phase 4 pays.

**Post-fix:** `cargo check --workspace --all-targets` clean (only the expected test-hook-unused + pre-existing
`block v0.1.6`); `cargo fmt` clean.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate
### Tests added (16)
- **marley_lsp::diagnostics — 4 pure units** (cov/MSI 100): `store_iter_and_total_len`; `problem_rows`
  aggregate + sort (Error-first, `(severity,path,line)` deterministic over input order, field mapping from
  `Diag.span.start`); the two-producer merge (a terminal `(path,line)` → source Terminal, severity Error);
  the cap `(kept, dropped)` at/over the boundary.
- **marley_app::editor_problems — 4 pure units** (cov/MSI 100, NEW test mod): `severity_glyph` (all 4 arms);
  `visible_problems` (cap boundary); `keep_selection_by_identity` (re-find / gone→prev.min(len-1) / None /
  empty→0); AND **the inspect-C2 case** — two rows sharing `(path,line)` but differing severity → keeps the
  RIGHT one, proving the 4-tuple key.
- **marley_app::status_bar — 2 pure units**: `workspace_diag_summary` (differ→Some, equal→None); the
  `cockpit_status` workspace-tier order (after diagnostics, before focus).
- **marley_app — 6 headless drives** (real RootView): open + Esc; list across 2 NEVER-OPENED files
  (REQ-004/007); Enter → jump opens a CLOSED file at the line + NavStack (REQ-005, owning-host encoding);
  a live refresh KEEPS the selection by identity when a diag appears above (REQ-006 — selection tracks to the
  new index, not index-stuck); a CLEAR (empty replace) drops rows live (REQ-006); ↓ past the cap clamps
  (REQ-009). The drives wire all 4 test hooks → the dead-code warnings cleared.

### Gate — GREEN [diff] (first pass, no reds)
`scripts/gates.sh --diff` → **`GATE GREEN [diff]`, 15 passed / 0 failed** on the FIRST run. gate:4 coverage
**100% lines**, gate:5 mutation **MSI 100%** (the `problem_rows`/`editor_problems`/`status_bar` pure seams),
miri + visual green. Receipt written. The #325/#326 lessons applied UP FRONT avoided the #326 reds:
`problems_overlay` carried its `#[cfg_attr(test, mutants::skip)]` from implement, no "Zed"/"Warp" in source
comments (the gutter palette is described as "danger/warning", not a brand), and `cargo fmt` ran before the
gate. No new dependency.

### LIVE DRIVE — ENV-BLOCKED (locked screen), units + mechanism fallback
The screen was **LOCKED** (`CGSSessionScreenIsLocked=1`, `onconsole=1` — chad AFK, as in #324/#325/#326). A
locked mac fully blocks synthetic CGEvents + `screencapture`; no password attempted
(`PR-claude-selftest-locked-screen-blocks-capture`). **Coverage without the live drive:** the 6 headless
drives exercise the REAL RootView through open→push-diag→refresh→nav→Enter→jump (including the closed-file
open, the identity-keeping refresh, the live CLEAR, and cap==nav); the render reuses the #325/#326
`symbols_overlay`/`search_overlay` primitives (`menu_origin`/`popup_window`/`truncate_cols`) + cap==nav; the
jump reuses #312's **live-proven** `open_and_place_caret` + NavStack with the #325 owning-host encoding. **Re-run
the ⌘⇧M live drive when unlocked** (~2 min: two probe files each with a planted error, one CLOSED → ⌘⇧M lists
both → Enter opens the closed file at the squiggle → fix one + save → its row drops on the next publish) — a
confidence top-up, no ticket.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete
- **CHANGELOG** — #327 entry above #326 (Added): the ⌘⇧M panel, the pure `DiagnosticStore::iter`/`problem_
  rows` engine, the finder picker opening CLOSED files with owning-host encoding, the live re-derive +
  identity-keyed selection, the `cockpit_status` workspace tier.
- **Architecture docs** — `editor.md` gained a "Problems panel (#327) SHIPS" paragraph after the #326
  project-search block. `crate-map.md`: the `marley_lsp` row's #310 diagnostics clause now names the #327
  `iter`/`total_len`/`ProblemRow`/`problem_rows` additions (tool-shaped for a future `workspace.problems` MCP
  tier). (The crate-map doesn't enumerate app `editor_*` modules, so `editor_problems` needs no row.)
- **Knowledge captured (forge wired §19)** — AAR `d1fda6ae` CLOSED (`completed`, effectiveness 5, 2 novel
  findings; jobs distillation + confidence_drift + pattern_emergence). The failure + prevention rule were
  recorded at inspect: `BF-live-list-selection-identity-key-not-unique-001` +
  `PR-claude-live-refresh-selection-identity-key-must-be-unique-001` (a live-refreshed list's identity key
  must uniquely identify a row — `(path,line)` collided for two same-line diagnostics; fixed with the 4-tuple
  `(path,line,character,severity)`). **No new AD** — #327 is composition of established patterns (the #310
  store, the finder recipe, the #312 jump, the #325 encoding posture, the #325/#326 cap==nav + modal-clear);
  D-TERMINAL (LSP-only v1, the terminal merge deferred) is a scope cut, not a novel architecture decision.
- **Ticket closed** — forge #327 (`898479e5`) → `done`; local `TICKET-327` → closed, moved to `tickets/closed/`.
- **Pipeline archived** — this doc pair moved to `pipeline/completed/`.

status: Phase 5 — Complete PASS
