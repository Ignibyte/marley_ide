---
pipeline_id: 1c94725d-f929-44ea-8ef1-af12c13b46d6
ticket: forge#326 (992a1da5-7236-4e66-8bc2-e1557a672e78) · local docs/planning/tickets/open/TICKET-326-project-search.md
aar_id: 206fb9a2-6683-47d2-bcc1-c7d687c1b0ac
status: Phase 5 — Complete PASS
title: Project-wide content search (⌘⇧F) — grep file contents across the workspace (roadmap B7, phase 1)
type: feature
milestone: M21
references: [forge#326, forge#312, forge#317, forge#325, forge#304]
---

## Title
⌘⇧F searches file CONTENTS across the whole workspace — type a needle, see every matching line grouped by
file, Enter jumps there. Today NOTHING greps across files: `find.rs` is terminal scrollback, editor find is
one buffer, ⌘P is file NAMES. This is **roadmap B7, phase 1** (docs/zed_architecture/subsystems/
06-project-fs-search.md §4): a READ-ONLY results surface — "80% of the value with none of the multibuffer
cost." Phase 2 (editable multibuffer results / replace-all) stays gated exactly as the deconstruction gates
it; NOT this ticket. Works in every language day one — no LSP needed.

## Scope
### In
- **Pure engine (`search_lines`)** — `search_lines(text, needle, opts) -> Vec<LineMatch{row, col, preview}>`:
  literal substring v1, a **case-sensitive** toggle + a **whole-word** toggle. Per-line scan, every match on a
  line reported (multi-match rows), `col` correct on non-ASCII lines (char offset, the #309 bridge's unit).
  Empty needle → no matches. gpui-free + editor-free (tool-shaped — a future `workspace.search` MCP read-tier).
- **The walker (seam stays in `marley_project`)** — enumerate workspace files, skipping the curated skip set
  (`should_skip`) + anything the editor treats as non-text (**`is_probably_binary`** / larger than
  **`VIEWER_MAX_BYTES`**, the SAME rules the editor loads by — one truth about what Marley considers text).
  The Route-A (`ignore` crate) vs Route-B (extend `list_files_in`) choice is the ONE **design-owned decision**
  (see Locked-In D-WALKER) — both routes keep the seam here.
- **Dirty-buffer precedence (the correctness heart)** — a file open with UNSAVED edits is searched via its
  LIVE `Buffer` text (`open_docs`), disk otherwise. A rename you just typed must be findable before ⌘S.
- **Honest caps** — `MAX_FILES` / `MAX_RANGES` bounds (sized at design; the deconstruction pins 5_000 /
  10_000) with the #317 "+N more" tail; truncation STATED in the footer, never silent.
- **Off-thread execution** — the search runs on a worker (the syntax-worker `channel` + pump-drain precedent,
  app.rs:367/8957/1092), STREAMING results into the picker as files complete; input never blocks. A new query
  CANCELS the old walk via a **generation counter** (the stale-guard family — only the live generation's
  results apply).
- **UI (⌘⇧F, the finder recipe)** — a query picker (`FinderState`, the #221 card) that OWNS the keyboard
  (↑/↓ walk matches, Enter, Esc). Rows = `path:line: preview` with the matched span highlighted, grouped by
  file with per-file counts; a footer "N matches in M files (+K more)". Enter = the #312
  `open_and_place_caret` + NavStack push (⌃- returns; a failed open flashes and moves nothing). Registered at
  every overlay choke point. ⌘⇧F is Editor-scoped.
- **Cap == nav** — the results display cap EQUALS the ↑/↓/Enter navigation clamp (the #325 rule,
  `PR-claude-display-cap-must-equal-navigation-cap-001`): navigation can never address an unshown row.

### Out (explicitly deferred)
- **Phase 2 — editable multibuffer results / replace-all** — needs the `[Zed-derived]` multibuffer; gated by
  the deconstruction, a named follow-up.
- **Regex** — literal substring v1; regex (fancy-regex, the deconstruction's pick) is a named follow-up.
- **Include/exclude globs**, **search history**, **symbol-kind/file-type filters** — v1 shows every text file.

## Reference (§20)
**Behavior reference — Zed / VS Code project search (⌘⇧F), OBSERVED only**: a query field, results as
`path:line` rows grouped by file with per-file counts, Enter jumps to the match. Marley matches that BEHAVIOR
via its OWN composition — the `finder.rs` picker, the #312 navigation, an off-thread worker (the syntax-worker
precedent), and a PURE literal-substring engine — plus the permissive **`ignore`** crate (MIT OR Unlicense —
passes deny.toml's MIT allow) IF Route A is chosen. The two-phase plan + the cap/algorithm picks come from
Marley's OWN deconstruction analysis (`docs/zed_architecture/subsystems/06-project-fs-search.md §4`), a spec we
authored — NOT Zed source. Per that doc's provenance flag, the phase-1 read-only surface is a GENERIC
capability (grep-across-files); the `[Zed-derived]` GPL-bound design is the phase-2 multibuffer, explicitly
deferred. Clean-room §20: no Zed source read or translated; gpui + permissive crates used freely.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — pure `search_lines` engine** (literal substring, case + whole-word toggles), gpui-free + editor-free,
  tool-shaped so it doubles as the future `workspace.search` MCP read-tier. cov/MSI 100.
- **D2 — the walker seam lives in `marley_project`** (beside `list_files_in`/`should_skip`), whichever route.
- **D3 — one truth about "text"**: skip via the editor's OWN `is_probably_binary` + `VIEWER_MAX_BYTES`, not a
  second heuristic.
- **D4 — dirty buffers win**: `open_docs` live text over disk for any open+edited file (the correctness heart).
- **D5 — off-thread + generation-cancel**: worker + channel + pump-drain (the syntax-worker precedent); a new
  query bumps a generation and only the live generation's streamed results apply (the stale-guard family).
- **D6 — the finder-recipe MODAL** (owns the keyboard) + the #312 `open_and_place_caret` + NavStack WHOLE
  (guard inheritance — `PR-claude-second-consumer-must-inherit-the-first-consumers-guards-001`) + the #325
  cap==nav rule + the #317 "+N more" honest tail.
- **D7 — v1 cuts named**: no phase-2 multibuffer/replace, no regex, no globs, no history.
- **D-WALKER (DESIGN-OWNED — the one open decision)**: Route A adopt `ignore` (gitignore-aware parallel walk,
  correctness for free; COST = a transitive dep tree — globset/regex-automata/memchr/… — each needing the
  deny.toml license-allow + `[bans]` review, and the gate's cargo-deny stays green) vs Route B extend
  `list_files_in` (no new deps; COST = hand-rolled gitignore semantics or the current hardcoded skip set, more
  code to test to MSI 100). Design PICKS one, records the rationale, and — if Route A — confirms the full
  dependency tree passes `scripts/gates.sh` (deny) BEFORE implement.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN ⌘⇧F is pressed in the editor, the system shall open a query picker that owns the keyboard (↑/↓ walk matches, Enter accepts, Esc closes). | headless + LIVE (or units+mechanism if env-blocked) |
| REQ-002 | The pure `search_lines(text, needle, opts)` shall return every literal-substring match as `(row, col, preview)`, reporting multiple matches per line, with `col` a correct CHAR offset on non-ASCII lines, and shall return empty for an empty needle. | pure unit cov/MSI 100 |
| REQ-003 | The engine shall honor a case-sensitive toggle and a whole-word toggle (a whole-word match rejects a needle flanked by an alphanumeric/`_`). | pure unit cov/MSI 100 |
| REQ-004 | WHEN a searched file is open with unsaved edits, the system shall search its LIVE buffer text; a file not open shall be read from disk. | pure decision-table unit + headless |
| REQ-005 | The walk shall skip a file in the curated skip set, a binary file (`is_probably_binary`), and a file larger than `VIEWER_MAX_BYTES`, using the editor's own rules. | pure unit + headless |
| REQ-006 | WHEN the result set exceeds `MAX_FILES` or `MAX_RANGES`, the system shall truncate and STATE the truncation in the footer ("+N more"), never silently drop. | pure unit (cap+tail) + headless |
| REQ-007 | WHEN a new query supersedes an in-flight walk, the system shall cancel the old walk (generation counter) and apply only the latest generation's results. | headless (stale-generation dropped) |
| REQ-008 | WHEN a result row is accepted (Enter), the system shall open the file and place the caret at the match (encoding-aware), pushing the prior location to the NavStack; a failed open shall flash and move nothing. | headless + review |
| REQ-009 | The search shall execute OFF the UI thread (worker + channel), streaming results into the picker as files complete without blocking input. | review + headless drain |
| REQ-010 | The results picker's display cap shall EQUAL its ↑/↓/Enter navigation clamp — navigation shall never select a row the render does not show. | pure unit + headless |

## Phase Plan
- **P2 Design** — DECIDE D-WALKER (Route A/B + rationale + deny check if A). The pure layer (`search.rs`:
  `search_lines`, `LineMatch`, `SearchOpts`, the cap+tail helper, the dirty-vs-disk decision fn) + the walker
  seam in `marley_project` + the off-thread worker (req/result types, the generation, the pump-drain arm) +
  the app shim (⌘⇧F, the picker state reusing `FinderState`, the streaming merge, the row render grouped by
  file, Enter→#312, the footer). The mutation surface. Confirm §20.
- **P3 Implement** — to the manifest; every new pure fn gets a direct unit.
- **P3.5 Inspect** — critics vs the diff; the dirty-buffer precedence, the generation-cancel race, the
  cap==nav clamp, the guard inheritance, and (if Route A) the new-dependency provenance get the hardest look.
- **P4 Validate** — tests + gate green; the LIVE drive (⌘⇧F a needle in two probe files → both rows, Enter
  lands centered; edit one match away without saving → re-search finds N-1, the dirty-buffer proof), falling
  back to units+mechanism if the screen is locked.
- **P5 Complete** — CHANGELOG + editor.md + crate-map.md; AAR; archive; close #326.
