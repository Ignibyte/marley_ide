---
pipeline_id: 5cec0c6a-d9c9-4f9c-96d9-57e5300d6ad6
ticket: docs/planning/tickets/open/TICKET-430-problems-multibuffer.md
status: Phase 5 — Complete PASS
title: Problems panel, editable form — the diagnostics multibuffer (M32 B-c step 6)
type: feature
milestone: M32
references:
  - docs/planning/design-notes/display-map-shelf.md
  - docs/planning/pipeline/completed/427-search-results-multibuffer.spec.md
  - docs/planning/pipeline/completed/428-multibuffer-editing.spec.md
  - docs/planning/pipeline/completed/429-search-results-multibuffer.spec.md
  - docs/planning/pipeline/completed/327-problems-panel.notes.md
---

## Title
The B-c chain's closing consumer: the #327 problems panel (⌘⇧M, today a
jump-list) gains its EDITABLE form — one action materializes the workspace's
current LSP diagnostics as a multibuffer (per-diagnostic excerpts with context
under file headers, severity + message visible), edits write through (#428),
and the surface tracks republishes: fix a diagnostic in place, save, and it
leaves the surface without closing it. The jump-list panel STAYS — this adds
the second form, not a replacement.

## Scope
### In
- A materialize action from the open ⌘⇧M problems panel (the #427 overlay's
  ⌘⏎ precedent) opening the diagnostics multibuffer as a real tab.
- The multibuffer model consuming the aggregated `problem_rows` set: per-file
  excerpt groups (context windows, merged runs — the shipped #427 machinery),
  each diagnostic's SEVERITY and MESSAGE visible at its excerpt, file header
  bands with per-file counts, the footer summary.
- Full #428 editability on the surface (caret, typing/IME, ⌫/⌦/Enter, ⌘Z/⌘⇧Z
  journal, ⌘S touched-consent, pinned/lazy targets, releases) — unchanged
  machinery, this ticket only feeds it a second builder.
- Jump-to-source per excerpt line (⌘⏎ — the #428 jump), the caret landing at
  the diagnostic position through the negotiated-encoding bridge (the #327
  panel's own jump shape).
- Refresh on LSP republish: a diagnostic fixed (or gone for any reason) leaves
  the surface without closing the tab; the refresh must never wedge on a
  stale set (the #331/#327 latch lessons). The exact policy (live rebuild vs
  marked-stale + explicit refresh; caret survival) is a Phase 2 decision
  bounded by REQ-004's observables.
- Empty-set and stale-surface guards fail CLOSED (PR-claude-429-a).

### Out (explicitly deferred)
- The terminal-lane problems producer (still wired-but-empty — the #310
  two-producer doctrine's D-TERMINAL follow-up).
- Code actions / quick-fix from the surface; severity filtering UI; a
  Replace-All analog (no query exists here).
- mb IME composition, paste, precise click-column mapping (the recorded #428
  v1 seams) and the DisplayMap excerpt unification (the chain's later prize).
- Any change to the jump-list panel's own behavior.

## Reference (§20)
**Zed** (the editor reference). The behavior matched is Zed's Project
Diagnostics view: every workspace diagnostic materialized as one editable
multibuffer — excerpts grouped under per-file headers, each with a
severity-marked message block above its context lines, edits landing in the
real buffers, and the view tracking diagnostic republishes so fixed items
disappear. Behavior map: `docs/zed_architecture/subsystems/03-editor-multibuffer.md`
(§BlockMap — "insert full-width block widgets (diagnostics, excerpt headers,
…) between lines"; §6 multibuffer headers-are-blocks), plus the roadmap's B7
naming ("a diagnostics-multibuffer inherits the later gate, like #326's
search"). Clean-room: behavior observed from the maps + published material
only; the implementation is the B-c chain's own shipped model (`multibuffer.rs`)
— NO Zed source read or translated. Marley deviation kept: our model renders
Header/Line slots with bands (no block-widget layer); the message band maps
into that slot shape, Phase 2 decides the exact slot/band form.

### Prior art
1. **Behavior maps** — `docs/zed_architecture/subsystems/03-editor-multibuffer.md`:
   diagnostics rows are BLOCKS in Zed's model (message bands above excerpts);
   excerpt headers are the same mechanism. Warp maps: N/A (pure editor
   surface). The #427 spec already mined this doc for the excerpt/header
   grammar — this ticket adds only the per-diagnostic band.
2. **Published material** — the LSP spec's `textDocument/publishDiagnostics`:
   per-file FULL-REPLACE sets (shipped at #308/#310) — so "a fixed diagnostic
   vanishes on the next publish" is protocol-guaranteed, no diffing needed;
   the store's canonical-PathBuf keying (the #308 F1 fix) already normalizes
   the wire uri.
3. **Our permissive deps / owned substrate (the highest-yield leg)** — no
   external crate owns diagnostics-excerpt grouping; the substrate is OUR OWN
   shipped modules, which is the whole point of the chain: `multibuffer.rs`
   `build` (context-window merge, cum/locate, wash rows), the #428 live
   index/journal/target lifecycle, the #429 guards + batch ledger,
   `marley_lsp::problem_rows` + `DiagnosticStore::iter()` (#327), the
   negotiated-encoding jump bridge (#310/#312). gpui/ropey/regex: checked, no
   new owner needed — this ticket should add a builder + a band, not
   machinery.

## React-first (parity)
UI-AFFECTING — zone B surface. marley-web files (per the MARLEY-PARITY port
map): `views/MultibufferView.tsx` grows the diagnostics form (per-diagnostic
severity glyph + message band at its excerpt, no query washes) and
`overlays/ProblemsPanel.tsx` grows the materialize affordance (the #427 hint
precedent); `pages/Workspace.tsx` wires the problems→multibuffer flow. Build
& visually verify in marley-web first (`pnpm --filter @workspace/marley-ide
run dev` → localhost:5173), screenshot + READ the PNG, then port 1:1.
Validate captures the React↔Marley parity pair (pixel-sampled).

## Locked-In Decisions
- D1 — **Reuse over machinery**: the surface IS the shipped B-c multibuffer
  (#427 model + #428 editing + #429 guards); this ticket adds a diagnostics
  BUILDER + the severity/message band + the refresh wiring. Any new editing
  machinery is out of bounds.
- D2 — **The jump-list stays**: ⌘⇧M's panel is untouched as a picker; the
  editable form is an additional surface reached FROM it.
- D3 — **Refresh never wedges, never closes**: whatever policy Phase 2 picks,
  a republish while the surface is open must (a) never leave a stale card
  latched (the #331/#327 lessons) and (b) never close/replace the tab out
  from under an editing session without the model's consent rules.
- D4 — **Fail closed** (PR-claude-429-a): an empty problems set does not
  materialize an empty surface; a materialize that did not open never
  proceeds to any side effect.
- D5 — **Encoding discipline**: diagnostic positions are RAW negotiated
  encoding at the store; any caret placement goes through the SAME bridge
  the #327 jump uses (no new conversion path).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the ⌘⇧M problems panel is open with N ≥ 1 aggregated diagnostics across M files and the user presses the materialize chord, the system shall open ONE multibuffer tab whose excerpt groups cover all N diagnostics under M file headers, each diagnostic's severity glyph and message visible at its excerpt. | headless drive |
| REQ-002 | WHEN the diagnostics multibuffer is open and the user places a caret and types, the system shall write the edit through to the file's ONE registry buffer (visible in the file's own tab), with ⌘Z routing through the shared journal — the #428 behavior unchanged on this surface. | headless drive |
| REQ-003 | WHEN the user presses the jump chord on a diagnostic's line, the system shall open that file with the caret at the diagnostic's position (negotiated-encoding bridge) and push the NavStack origin (⌃- returns). | headless drive |
| REQ-004 | WHEN the LSP republishes a file's diagnostics while the surface is open and a previously-shown diagnostic is no longer in the set, the surface shall stop showing that diagnostic's excerpt without the tab closing, and unaffected excerpts shall remain; a republish shall never wedge the surface on a stale set. | headless integration (fake publish through the real store) |
| REQ-005 | WHEN the aggregated problems set is EMPTY, the materialize action shall flash-and-bail with zero side effects (no tab, no state change). | unit + drive |
| REQ-006 | WHILE the gate runs, the new pure logic (builder, band mapping, refresh decision fns) shall hold cov 100 / MSI 100 with no suppressions (new files `git add -N`-staged before the diff gate — PR-claude-new-file-mutants). | gate exit |

## Phase Plan
- **P2 Design** — the builder's input mapping (ProblemRow → the model's match
  shape + band), the severity/message band's slot form, the refresh policy
  (D3-bounded), the entry-point chord, file manifest + regression test plan.
- **P3 Implement** — POC first (MultibufferView diagnostics form +
  ProblemsPanel affordance), then the Rust builder + wiring per manifest.
- **P3.5 Inspect** — independent critics vs the diff; fix the real findings.
- **P4 Validate** — write + RUN the planned tests; live drive + parity pair;
  gate green.
- **P5 Complete** — docs (§21), parity sync, ledger capture (§19), close the
  ticket (BACKLOG row already left at promotion), archive.
