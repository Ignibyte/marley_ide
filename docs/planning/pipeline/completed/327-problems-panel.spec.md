---
pipeline_id: d99f73f1-dfe9-4ee0-9499-00d6d6cd92b1
ticket: forge#327 (898479e5-952e-4e0a-b532-307a7f11398f) · local docs/planning/tickets/open/TICKET-327-problems-panel.md
aar_id: d1fda6ae-20d0-415b-861b-e41a18f6a222
status: Phase 5 — Complete PASS
title: The problems panel (⌘⇧M) — every diagnostic in the workspace, one jumpable list
type: feature
milestone: M21
references: [forge#327, forge#310, forge#290, forge#312, forge#317, forge#325, forge#326]
---

## Title
⌘⇧M opens a list of EVERY error and warning rust-analyzer knows about, across every file — including files
you don't have open — grouped, severity-sorted, and jumpable. F8 (#290/#310) walks the FOCUSED file's
diagnostics + the footer counts them per-file; this is the WORKSPACE view. Phase 1 is a READ-ONLY list (a
diagnostics-multibuffer inherits the later gate, exactly as #326's search deferred its editable phase 2).

## Scope
### In
- **Store enumeration (the missing primitive, PURE)** — `DiagnosticStore` (marley_lsp diagnostics.rs:74) is a
  private `HashMap<PathBuf, Vec<Diag>>` exposing only `for_path`; add `iter() -> impl Iterator<Item=(&Path,
  &[Diag])>` (pure, unit-tested), + an `LspHost::diagnostics_iter()` passthrough (the store is private on the
  host at lsp_host.rs:100).
- **Workspace aggregation + the composed rows (PURE)** — `problem_rows(hosts_diags, terminal_rows) ->
  Vec<ProblemRow{path, line, character, severity, message, source}>`: aggregate every Ready host's store
  (`App.lsp_hosts` is keyed per workspace root — a two-project workspace merges both), MERGED with the M18
  terminal failed-block lane as `merged_rows` does per-file (one lane, two producers — the #310 doctrine,
  lifted workspace-wide). Sort by **severity** (the `Severity` `Ord`: Error<Warning<Info<Hint) then **path**
  then **line**. Capped (~500) with the honest "+N more" tail (#317). gpui-free + tool-shaped (a future
  `workspace.problems` MCP read-tier).
- **UI (⌘⇧M, the finder recipe)** — a modal picker (`FinderState`; owns the keyboard: ↑/↓, Enter, Esc). Rows
  = a severity glyph (reusing the gutter's danger/warning coloring) + `path:line` + the message through
  `truncate_cols`. Enter = the #312 `open_and_place_caret` + NavStack — and the target MAY be a CLOSED file
  (the open path handles it; that is the panel's point). A failed open flashes + moves nothing. Registered at
  every overlay choke point. **⌘⇧M is a NEW Editor-scoped binding — it SHADOWS NOTHING** (the chord is free,
  unlike #325/#326's shadows).
- **Live-updating** — the picker re-derives its rows on the pump when a `publishDiagnostics` lands (the store
  REPLACE/CLEAR semantics make this free); the selected row is kept by **IDENTITY (path+line)**, not index, so
  a refresh doesn't teleport the selection (the live-identity family applied to a LIST).
- **Footer segment** — `cockpit_status` (status_bar.rs:56, ordered-`Vec` pushes, order-tested) gains a
  workspace tier when the workspace count differs from the focused file's ("2 errors, 1 warning · 7
  workspace"). Formatter pure, the order tests extended.
- **Position honesty** — Diags keep RAW line/col (they survive edits by design, diagnostics.rs header); for an
  OPEN dirty buffer a row may have drifted since publish. v1: jump to the stored line CLAMPED
  (`open_and_place_caret` already clamps); the drift window closes at the next publish. Stated, not hidden.
- **Cap == nav** — the display cap EQUALS the ↑/↓/Enter clamp (the #325/#326 rule).

### Out (explicitly deferred)
- **A diagnostics-multibuffer** (the editable all-diagnostics surface) — inherits the phase-2 gate, like #326.
- **Filter chips** — no severity/source filter UI (an errors-only toggle is the ONE exception IF cheap at
  design; else deferred). No `relatedInformation` expansion.

## Reference (§20)
**Behavior reference — Zed's ProjectDiagnosticsEditor (OBSERVED only)**: a workspace-wide diagnostics list,
severity-grouped, each row jumping to the file:line (incl. closed files). Cited from Marley's OWN
deconstruction analysis `docs/zed_architecture/subsystems/05-lsp-language-intelligence.md §5` — a spec we
authored, NOT Zed source. Marley matches the BEHAVIOR via its own composition — the `finder.rs` picker, the
#310 `DiagnosticStore` + `merged_rows` two-producer doctrine, the #312 navigation — over the published **LSP
3.17 `publishDiagnostics` wire** (already shipped in #310). Zed's ProjectDiagnosticsEditor is a multibuffer
(GPL, `[Zed-derived]`) — Marley ships the read-only list, and the multibuffer stays deferred. Clean-room §20:
no GPL source read or translated.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — pure `DiagnosticStore::iter()`** (marley_lsp) + an `LspHost::diagnostics_iter()` passthrough; cov/MSI
  100 on the pure iter.
- **D2 — pure `problem_rows` composer** (workspace aggregation + the terminal-lane merge + severity/path/line
  sort + caps), gpui-free + tool-shaped. cov/MSI 100.
- **D3 — ⌘⇧M is a NEW Editor-scoped binding** (shadows nothing; the #322/#323/#324 "free chord" posture, not
  the #325/#326 shadow).
- **D4 — the finder-recipe MODAL** (owns the keyboard) + the #312 `open_and_place_caret` + NavStack WHOLE
  (guard inheritance — a failed open flashes + moves nothing; opens a CLOSED file) + the #325 cap==nav rule +
  the #317 tail. Opening it DISMISSES any lower live overlay (the #325 modal-steals-keys lesson).
- **D5 — live re-derive on the pump; selection kept by IDENTITY (path+line), not index** (a refresh must not
  teleport the selection — the live-identity family).
- **D6 — `cockpit_status` gains a workspace tier** (pure formatter; the order tests extended).
- **D7 — position honesty**: RAW stored line, jump CLAMPED; drift closes at the next publish.
- **D-TERMINAL (DESIGN-OWNED)**: HOW the M18 terminal failed-block lane folds into the WORKSPACE panel — the
  existing `open_file_diagnostic_rows` merge is per-FOCUSED-file. Design decides: include the terminal rows
  for the file(s) the terminal references (tagged `source=terminal`) vs LSP-only v1 with the terminal merge a
  named follow-up. Either way `problem_rows` takes both producers so the seam is ready.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN ⌘⇧M is pressed in the editor, the system shall open a modal problems picker that owns the keyboard (↑/↓, Enter, Esc). | headless + LIVE (or units+mechanism if env-blocked) |
| REQ-002 | `DiagnosticStore::iter()` shall yield `(path, &[Diag])` for every path that has diagnostics, and an empty iterator for an empty store. | pure unit cov/MSI 100 |
| REQ-003 | `problem_rows` shall aggregate every host's diagnostics with the terminal-lane rows into `ProblemRow`s sorted by severity then path then line, capped with a "+N more" tail. | pure unit cov/MSI 100 |
| REQ-004 | The picker shall list rows across files INCLUDING files not open, each a severity glyph + `path:line` + the message. | pure unit + headless |
| REQ-005 | WHEN a row is accepted (Enter), the system shall open the row's file (even if closed) and place the caret at the stored line (clamped), pushing the NavStack; a failed open shall flash and move nothing. | headless + review |
| REQ-006 | WHEN a `publishDiagnostics` REPLACE/CLEAR lands, the system shall re-derive the picker's rows and keep the selection by identity (path+line), not index. | headless |
| REQ-007 | WHEN multiple Ready hosts exist, the system shall merge every host's diagnostics into the list. | headless (2-host) + review |
| REQ-008 | The `cockpit_status` footer shall show a workspace-diagnostics tier when the workspace count differs from the focused file's count. | pure unit (formatter + order) |
| REQ-009 | The picker's display cap shall EQUAL its ↑/↓/Enter navigation clamp. | pure unit + headless |

## Phase Plan
- **P2 Design** — DECIDE D-TERMINAL (the terminal-lane fold). The pure layer (`DiagnosticStore::iter`,
  `problem_rows`/`ProblemRow`, the cap+tail, the identity-keep decision fn, the footer formatter's workspace
  tier) + the host passthrough + the app shim (⌘⇧M, the picker state reusing `FinderState`, the pump
  re-derive, the row render, Enter→#312, the choke arms). The mutation surface. Confirm §20.
- **P3 Implement** — to the manifest; every new pure fn gets a direct unit.
- **P3.5 Inspect** — critics vs the diff; the workspace aggregation + the two-producer merge + the
  selection-by-identity + the closed-file open + guard inheritance get the hardest look.
- **P4 Validate** — tests + gate green; the LIVE drive (two errors across two probe files, one CLOSED →
  ⌘⇧M lists both → Enter opens the closed file on the squiggle; fix one + save → its row drops live), falling
  back to units+mechanism if the screen is locked.
- **P5 Complete** — CHANGELOG + editor.md + crate-map.md; AAR; archive; close #327.
