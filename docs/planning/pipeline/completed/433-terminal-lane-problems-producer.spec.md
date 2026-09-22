---
pipeline_id: f4b50044-6e6e-4720-8c14-4a283e509d72
ticket: docs/planning/tickets/open/TICKET-433-terminal-lane-problems-producer.md
status: Phase 5 — Complete PASS
title: The terminal-lane problems producer — failed Blocks feed problem_rows (M33 tail step 3)
type: feature
milestone: M33
references:
  - docs/planning/design-notes/m33-tail-and-wedge-shelf.md
---

## Title
The #310 two-producer doctrine's standing empty seam finally fills:
`marley_lsp::problem_rows(lsp, terminal, cap)`'s `terminal` parameter —
`&[]` at both live call sites since #327, deferred again at #430
(D-TERMINAL) — receives real rows. Failed command Blocks' file:line refs
(the #289 fold's scanners, lifted from one-open-file to any-file), scanned
WORKSPACE-wide across every project's every terminal grid (the PR-1345
cross-tab iterator rule — NOT the active-or-first grid), enter the
aggregation as `ProblemSource::Terminal` rows. ⌘⇧M and the #430
diagnostics multibuffer then show terminal build failures with or without
an LSP running — the first fusion thread of the Phase-C wedge (Zed has no
block model; Warp has no problems panel; Marley now has both feeding one
surface).

## Scope
### In
- The workspace terminal producer: a pure any-file ref extractor over the
  shipped #212/#291 scanners (`scan_links` + `parse_trace_frames` yield
  `(path, line)` pairs for ANY referenced file, not just the open one),
  a pure per-pane supersession fold (latest COMPLETED block per
  command/cwd — D5), and the app shim iterating ALL terminal grids of ALL
  projects (D2), feeding `problem_rows`' `terminal` parameter at BOTH live
  call sites (`refresh_problems` app.rs:16923, `refresh_problems_mb`
  app.rs:13121).
- Path resolution per ref against the OWNING project's canonical root
  (D3), 1-based scanner lines converted to the seam's 0-based `u32`,
  `(path, line)` deduped within the producer.
- The refresh gates learn terminal changes: the panel's
  `workspace_diag_total` gate and the mb's `workspace_diag_epoch` gate
  today see only LSP publishes — a failed block while either surface is
  open would never re-derive. Both must incorporate the terminal lane
  (D4 — direction locked here, seam at design).
- The ⌘⇧M row gains a terminal-source distinction (glyph/source column in
  the EXISTING row shape — zone B, POC first); terminal rows jump via the
  existing `jump_to_problem` (char 0 → line start; host-encoding fallback
  already total) and flow into the #430 multibuffer through the existing
  `build_problems` builder unchanged.
- A driven multi-terminal-tab, multi-project proof (the PR-1345 corollary:
  the enumeration bug class is invisible to pure units).

### Out (explicitly deferred)
- Severity inference beyond `Error` — no parsing "warning:" out of block
  output; every terminal row is the seam's fixed `Severity::Error`.
- Terminal rows in the diagnostics MULTIBUFFER's bands beyond what
  `problem_rows` already feeds — no new band/slot forms; terminal rows
  ride the shipped NoteMeta path exactly as LSP rows do.
- Log-follow live streaming — no live tail of a RUNNING block; rows derive
  from COMPLETED blocks at refresh points only.
- Any change to the #289 gutter lane (`open_file_diagnostic_rows` keeps
  its last-block-only, active-project policy — the two lanes may diverge
  on freshness; recorded, deliberate).
- Per-block rerun and block-scoped jump-to-failure (#435), runnables
  (#434) — the wedge's own tickets.
- Persistence/session-restore of terminal rows (stateless v1 — D1) and
  stat-per-ref existence filtering (the jump already fails closed; no IO
  in the producer).

## Reference (§20)
**Warp** (the terminal reference). The behavior matched is Warp's Block
model: terminal output segmented per command, each Block carrying its
command text, exit code, and cwd via the shell's DCS hooks (Precmd/
Preexec) — a failed command is a first-class, identifiable record with
its failure status and working directory attached, surfaced in situ by a
block-header indicator. Behavior map:
`docs/warp_architecture/subsystems/03-terminal-session-core.md` §6
(Block/BlockState/ExitCode/PromptInfo grammar; hooks carry pwd + exit
code) — the map's own Marley note says it exactly: "any … panel can
consume the same block stream + DCS metadata to know, per command, the
cwd / exit status / session — no extra instrumentation needed."
**Stated N/A:** Warp has NO workspace problems panel and no diagnostics
aggregation of any kind — failure surfacing stays per-block, in the
scrollback. The AGGREGATION grammar (a workspace-wide jumpable problems
list) is the Zed Project-Diagnostics behavior Marley already shipped at
#327/#430; no new Zed mining is needed (the consumer side is done). The
FUSION — blocks feeding the diagnostics surface — matches neither
reference; it is the M33 wedge premise itself. Clean-room: maps +
published material only; the implementation is Marley's own shipped
`terminal_blocks`/`links`/`problem_rows` substrate.

### Prior art
1. **Behavior maps** — Warp `03-terminal-session-core.md` §6: the block
   grammar above, incl. that exit codes arrive via Precmd (state-first
   classification matters — a Finished block with NO captured code is a
   failure-class anomaly, exactly Marley's `exit_status_kind` R42). Zed
   `03-editor-multibuffer.md`: already mined at #427/#430 for the
   consumer; nothing new to take. Verdict: the producer concept is
   map-confirmed (blocks carry everything needed); the aggregation half
   has no Warp analog to imitate.
2. **Published** — the compiler/runtime output formats are the ones the
   #196/#212 link recipe + #291 trace fold ALREADY parse: rustc/cargo's
   `path:line:col` human rendering, Python's `File "<path>", line N`
   traceback frames, rust-panic/node `path:line` backtrace forms
   (`links.rs` `scan_links` :~191, `parse_trace_frames` :216). This
   ticket adds ZERO new format parsing — it re-aims shipped scanners from
   "rows in the open file" to "(path, line) anywhere". LSP severity
   mapping: N/A (all rows Error by doctrine).
3. **Permissive deps / owned substrate** — `alacritty_terminal 0.26`
   (Apache-2.0, `terminal_blocks`' grid/PTY substrate) owns terminal
   mechanics, not block-ref aggregation; sweep found no crate owning
   "failed-block file-ref aggregation" (unsurprising — the command-block
   model itself is Marley's own `terminal_blocks`). The substrate is our
   shipped modules: `links.rs` scanners, `marley_project::
   resolve_under_root`, `block_status::exit_status_kind`, `problem_rows`
   + `ProblemSource::Terminal` (seam pre-built at #327). Verdict: add a
   producer, not machinery.

## React-first (parity)
UI-AFFECTING, minimally: the only visual delta is the ⌘⇧M panel row
gaining a terminal-source distinction — a glyph/source column in the
EXISTING row shape (the row today renders `severity_glyph + message +
rel_path:line`, and `ProblemSource` is unconsumed by any render). Judged
**zone B**: POC file `overlays/ProblemsPanel.tsx` (the Problem mock type
grows `source: 'lsp' | 'terminal'` + a terminal-sourced glyph in the
16px glyph slot or a trailing source hint; design picks the exact form
from the beautifului grammar). Build & verify in marley-web first,
screenshot + READ the PNG, then port 1:1; Validate captures the parity
pair. The multibuffer needs NO POC change (terminal rows ride the
shipped band path; the #430 capture already shows the grammar).

## Locked-In Decisions
- D1 — **Stateless re-derive, no latch**: terminal rows are derived from
  the CURRENT grids at every refresh (#331/#327/#430 lessons — a latch
  wedges, an epoch-gated stateless derive never lies). Closing the
  block's tab/project removes its rows at the next derive; a
  re-appearing failure re-enters; nothing persists.
- D2 — **ALL grids, ALL projects** (PR-claude-shim-aggregate-all-
  terminal-grids-001, prevention-rules ~1345; BF-claude-diagnostics-
  single-terminal-scope-001): the shim iterates `shell.projects()` ×
  `Project::terminal_grids()` (tabs.rs:568) × `PaneGrid::states()` —
  NEVER `terminal_grid_index()` (tabs.rs:541, the active-or-first trap).
  The proof MUST include a driven multi-terminal-tab case in a non-active
  project (the PR's corollary: pure units can't see the enumeration).
- D3 — **Resolve against the OWNING project's root** (the #289/#319
  resolution-root class: same root family both sides): each grid's refs
  resolve against the canonical root of the project that OWNS that grid
  — never the active project's, never `self.project_root`. The pure fn
  takes `root` as a parameter; the shim passes each project's own.
- D4 — **The refresh gates incorporate terminal changes — HOW is a named
  Phase-2 deliverable**: both `workspace_diag_total` (panel, app.rs:5963)
  and `workspace_diag_epoch` (mb, app.rs:5971) are LSP-only today. The
  locked DIRECTION per the #430 live lesson (row-sum fingerprints collide;
  the fix was a monotone publish epoch): a MONOTONE terminal-lane
  lifecycle signal (block opened/finished — the `DiagnosticStore::
  publish_epoch` shape), folded into BOTH gates so a failed block
  registers while either surface is open and the scan itself stays
  epoch-gated (no per-tick rescan of every grid when nothing moved).
  Design decides where the counter lives (session vs pane state) and the
  exact fold; it must also state the panel-vs-mb gate split (total vs
  epoch) explicitly.
- D5 — **Freshness = latest COMPLETED block per (command, cwd), per
  pane**: within each pane's BlockList (BlockIndex = execution order),
  group completed blocks by `(command, prompt.pwd)`; only the LATEST
  completed block of a group contributes, and only when it classifies
  Failure via `block_status::exit_status_kind` (state FIRST; Finished +
  no code = Failure — the BF-claude-details-status-line lesson: never
  read raw `exit_code` alone). A passing rerun therefore supersedes its
  predecessor's rows; a still-RUNNING rerun does NOT clear them yet (the
  problem stands until disproven — a deliberate, recorded delta from the
  #289 gutter's last-block-only policy, which is out of scope and keeps
  its own semantics).
- D6 — **The seam's contract holds; identity stays collision-free**:
  rows enter through the EXISTING `terminal: &[(PathBuf, u32)]`
  parameter — severity Error, character 0, `ProblemSource::Terminal`,
  scanner's 1-based line → 0-based u32, producer-deduped `(path, line)`.
  The synthesized message ("failed here (terminal)") and any tuple
  enrichment to carry the failing command are design-owned (D-MSG).
  `keep_selection_by_identity`'s 4-tuple `(path, line, character,
  severity)` can now collide across producers (an LSP Error at char 0 on
  the same line) — design decides the key growth (likely `source` joins
  the tuple; the #327 inspect-C2 rule generalized to two producers).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a command block finishes as a Failure in ANY terminal grid of ANY project — including a NON-FIRST terminal tab of a NON-ACTIVE project — and its output carries file:line refs, the system shall list those refs in ⌘⇧M as `ProblemSource::Terminal` rows (severity Error, path resolved against the OWNING project's canonical root, 0-based line), visually distinguished from LSP rows. | headless drive (multi-project, multi-terminal-tab — the PR-1345 corollary) + pure units on the extractor |
| REQ-002 | WHEN Enter is pressed on a terminal-sourced row, the system shall open the row's file (closed files included) with the caret at that line via the existing verify-the-landing jump + NavStack push; a failed open shall flash and move nothing. | headless drive |
| REQ-003 | WHEN the same command reruns in the same pane and cwd and finishes SUCCESS, the next refresh shall drop the superseded failure's rows; a rerun failing AGAIN shall show only the latest block's refs. | pure unit (supersession fold) + headless drive |
| REQ-004 | WHEN no LSP host is running (or a workspace holds zero LSP diagnostics), ⌘⇧M shall still populate from terminal rows alone, and ⌘⏎ shall materialize the #430 problems multibuffer over those rows through the existing builder (excerpts + bands, jump intact). | headless drive |
| REQ-005 | WHILE ⌘⇧M or the problems multibuffer is open, a newly-finished failed block shall register through the refresh gate and appear without reopening the surface, and the panel refresh shall keep the selection by IDENTITY (no teleport), the key collision-free across both producers. | headless drive + pure unit |
| REQ-006 | The producer shall re-derive statelessly from current grids at each refresh: closing the failed block's terminal tab (or its project) shall remove its rows at the next derive, with no latch or cache surviving the source. | headless drive + review |
| REQ-007 | WHILE the gate runs, the new pure logic (any-file ref extractor, supersession fold, gate fns) shall hold cov 100 / MSI 100 with no suppressions (new files `git add -N`-staged before the diff gate — PR:747). | gate exit |

## Phase Plan
- **P2 Design** — the pure-fn split (links-level any-file extractor vs
  editor_problems-level supersession fold vs the app shim's iteration);
  DECIDE D4's exact epoch seam + both gate folds; D6's message/tuple call
  (D-MSG) + the identity-key growth; the panel's terminal glyph (POC
  grammar); the cost posture (epoch-gated scan bounds); file manifest +
  regression test plan.
- **P3 Implement** — POC first (`overlays/ProblemsPanel.tsx` source
  distinction, visually verified), then Rust per manifest; every new
  pure fn unit-tested at birth.
- **P3.5 Inspect** — independent critics vs the diff; the enumeration
  (D2), root discipline (D3), and gate folds (D4) are the named hunts.
- **P4 Validate** — write + RUN the planned tests incl. the driven
  multi-project multi-tab proof and a LIVE no-LSP drive; parity pair;
  gate green.
- **P5 Complete** — docs (§21), parity sync, ledger capture (§19), close
  the ticket, archive the pair.
