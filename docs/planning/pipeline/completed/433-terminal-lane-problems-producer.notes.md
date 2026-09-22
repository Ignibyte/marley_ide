# The terminal-lane problems producer — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-433-terminal-lane-problems-producer.md
- **Pipeline spec:** 433-terminal-lane-problems-producer.spec.md

## Phase 1 — Plan
- **Request:** TICKET-433 — M33 tail step 3 (shelf: m33-tail-and-wedge-
  shelf.md): fill the #310 two-producer doctrine's standing empty seam —
  `problem_rows`' `terminal: &[]` receives real rows from failed command
  Blocks, scanned workspace-wide, so ⌘⇧M + the #430 mb show terminal
  build failures with or without LSP. Deferred at #327 (D-TERMINAL,
  design-owned then punted to "named follow-up") and again at #430
  (Scope-Out line 1). The first fusion thread of the Phase-C wedge.
- **Classification / tier:** work pipeline, feature, one shippable slice
  (a producer + two gate folds + one row glyph over shipped machinery).
  React-first zone B (glyph/source column in the existing panel row).
- **Recall (§18.3, codes):**
  - PR-claude-shim-aggregate-all-terminal-grids-001 (prevention-rules
    ~1345): workspace-wide shim aggregation iterates ALL terminal grids
    across ALL tabs; `terminal_grid_index()` is the trap; corollary — a
    driven MULTI-terminal-tab case is required, pure units can't see it.
  - BF-claude-diagnostics-single-terminal-scope-001 (failures.md:238):
    the #289-era bug this class comes from — only the live multi-tab
    drive exposed it; #295 fixed it for the ACTIVE project only.
  - The #289 inspect-F1 / #319 resolution-root lesson (comment at
    app.rs:12027): resolve refs against the SAME root family that names
    the target — here, each grid's OWNING project's canonical root.
  - The #331/#327 latch lessons + #430 D-REFRESH-QUIESCENT: stateless
    re-derive, no latch to wedge; and the #430 LIVE lesson — row-SUM
    fingerprints collide; the fix was the monotone `publish_epoch`. The
    terminal gate fold must be epoch-shaped, not count-shaped.
  - BF-claude-details-status-line-keyed-off-exit-code-not-status:
    classify failure via `block_status::exit_status_kind` (state FIRST;
    Finished+None = Failure), never a raw `exit_code` read.
  - #327 inspect C2 / prevention-rules:797: selection-keep identity must
    be the FULL tuple; two producers can now collide at (path, line, 0,
    Error) — the key likely grows `source` (design call).
  - PR:747: new pure source files `git add -N`-staged before the `--diff`
    gate (REQ-007 bakes it in). PR-claude-429-a: fail-closed gestures
    unchanged (empty-set mb rule already shipped).
- **Discovery (exact seams, file:line):**
  - `marley_lsp::problem_rows` — crates/marley_lsp/src/diagnostics.rs:207
    — signature `(lsp: impl Iterator<Item=(&Path, &[Diag])>, terminal:
    &[(PathBuf, u32)], cap: usize) -> (Vec<ProblemRow>, usize)`; terminal
    rows minted at :223-230 (severity Error, character 0, message
    "failed here (terminal)", `ProblemSource::Terminal` — enum at
    :174-180); sort (severity, path, line) :232-237; cap + dropped
    :238-240. `ProblemRow.line` is 0-based u32.
  - **Every `problem_rows(` call site:** app.rs:13121 (`refresh_
    problems_mb`, terminal `&[]` at :13123), app.rs:16923 (`refresh_
    problems`, terminal `&[]` at :16925) — the two live sites this
    ticket feeds; marley_lsp/src/diagnostics.rs tests :661, :686 (the
    only non-empty-terminal exercise — `problem_rows_two_producer_
    merge`), :705, :711.
  - The #289/#295 scanner (the prior-art fold, single-lane): app.rs:12023
    `open_file_diagnostic_rows` — active-project-only grid iteration
    :12040-12044 (`shell.active_project().terminal_grids().flat_map(
    |g| g.states())`), LAST-block-only :12051, Failure via
    `exit_status_kind` :12052, refs via `links::diagnostics_for_file` +
    `links::trace_diagnostic_rows` :12060-12062, root discipline
    `canonical_root(&self.project_root)` :12035 + the F1 comment :12027.
  - Pure scanners (links.rs): `scan_links` (#212), `first_file_ref_on_
    line` :191, `first_failure_ref` :207, `parse_trace_frames` :216
    (Python `File "…", line N` at `python_frame` :231),
    `diagnostics_for_file` :258, `trace_diagnostic_rows` :285 — both
    filter to ONE open path via `resolve_under_root`; the producer needs
    the any-file variant returning `(path, line)` (1-based → convert).
  - Block model (terminal_blocks): src/block.rs:60 `Block` — `command`
    :69, `state` :71, `exit_code` :73, `prompt: PromptInfo` (`pwd` :48),
    `output_text()` :113 (pure join, no PTY read); `BlockIndex` strictly
    execution-ordered :17; `BlockList::iter` :175; session accessor
    src/session.rs:460 `blocks()`. NO epoch/counter exists on the
    session today (grep) — D4's seam is new at design.
  - Iteration seams: tabs.rs:568 `Project::terminal_grids()` (all
    terminal tabs, minted at #295), tabs.rs:734 `Shell::projects()`,
    tabs.rs:412 `Project` (`root` field), workspace.rs:838
    `PaneGrid::states()`. The trap: tabs.rs:541 `terminal_grid_index()`.
  - Classification: block_status.rs:26 `exit_status_kind` (state first;
    Finished + Some(0) = Success; non-zero OR None = Failure).
  - Refresh gates + pump: app.rs:5963 `workspace_diag_total` (read by
    the panel gate :16914), app.rs:5971 `workspace_diag_epoch` (read by
    the mb gate :13103; born from the #430 live sum-collision); pump
    tick calls :2282 `refresh_problems(false)` + :2288
    `refresh_problems_mb()`; panel open force-build :16902.
  - Consumers: panel row render app.rs ~18595-18630 (severity glyph +
    message + `rel_under_root(active root)` path:line; `ProblemSource`
    is UNCONSUMED by any render — grep hits only editor_problems.rs test
    fixtures); jump app.rs:17003 `jump_to_problem` (host-ranked
    encoding, Utf16 fallback → a terminal row's char 0 lands at line
    start; total with zero hosts); mb builder editor_problems.rs:73
    `group_problem_files` / :109 `build_problems` (terminal rows flow
    through as `(line, 0)` + NoteMeta from `severity_glyph`) / :164
    `mb_refresh_due`; identity keep editor_problems.rs:41
    `keep_selection_by_identity` (4-tuple, no source).
  - POC: marley-web `artifacts/marley-ide/src/components/overlays/
    ProblemsPanel.tsx` — `Problem = {severity, message, path, line}`
    (:29), no `source` field yet; glyph slot at :170-173.
- **Decisions:** D1–D6 locked in the spec — stateless re-derive (no
  latch); ALL grids ALL projects with a driven multi-tab proof; owning
  project's canonical root per ref; both refresh gates grow a MONOTONE
  terminal-lane epoch fold (seam + gate-split stated at design — the
  named Phase-2 deliverable); freshness = latest COMPLETED block per
  (command, prompt.pwd) per pane, Failure via `exit_status_kind`, a
  passing rerun supersedes, a running rerun does not clear (recorded
  delta from the #289 gutter's last-block policy, which stays); seam
  contract fixed (Error / char 0 / Terminal / 0-based dedup) with the
  message + identity-key growth design-owned.
- **Prior-art verdicts (§20, three legs, honest):**
  1. Behavior maps — Warp 03-terminal-session-core.md §6 confirms blocks
     carry command/exit/cwd via DCS hooks and the map's own Marley note
     anticipates exactly this consumption; Warp has NO problems panel
     (stated N/A — surfacing is per-block in situ). Zed's aggregation
     side was fully mined at #327/#430; nothing new needed.
  2. Published — zero new formats: rustc/cargo `path:line:col`, Python
     tracebacks, panic/node backtraces are already parsed by the shipped
     #196/#212 recipe + #291 fold in links.rs; this ticket only re-aims
     them from one-open-file filtering to any-file collection.
  3. Permissive deps — alacritty_terminal 0.26 (Apache-2.0) owns the
     grid substrate only; no crate owns "failed-block file-ref
     aggregation" (the block model is Marley's own terminal_blocks).
     Verdict: add a producer, not machinery — the substrate is links.rs
     + marley_project::resolve_under_root + block_status + the
     pre-built ProblemSource::Terminal seam.

## Phase 2 — Design

Designed at HEAD d13255e; every Phase-1 seam re-verified live (post-432 lines:
gates :5987/:5995, `open_file_diagnostic_rows` :12114, `refresh_problems_mb`
:13190 with `&[]` :13214, `refresh_problems` :17077 with `&[]` :17092, panel
glyph render :18707). `SessionEvent` is Wakeup/ChildExited ONLY — the app
cannot count block transitions from pump events, which settles D4's seam.

### The decisions (D4/D-MSG/D6 closed; the pure-fn split)

- **D4 — the epoch lives in `terminal_blocks` (the source owns its change
  signal, exactly `DiagnosticStore::publish_epoch`'s shape).** A `block_epoch:
  u64` on the session model, bumped ON EVERY BLOCK LIFECYCLE TRANSITION —
  block BORN (preexec) and block FINISHED (precmd) — through whichever ONE
  internal choke both the `pump` byte path and `apply_hook` funnel into
  (implement locates it; the unit test drives BOTH paths and asserts one bump
  each). Accessor `pub fn block_epoch(&self) -> u64`.
  **The gate folds are PAIRS, not sums** — summing the LSP and terminal lanes
  into one number can cancel (+1 LSP publish while a term with epoch 1
  closes = sum unchanged — the #430 collision class across lanes):
  - Panel: `OpenProblems.diag_fingerprint: (usize, u64)` = 
    `(workspace_diag_total(), terminal_block_epoch())` — tuple `!=` gates.
  - MB: `problems_mb_fingerprint: (u64, u64)` =
    `(workspace_diag_epoch(), terminal_block_epoch())` — tuple `!=` feeds
    `mb_refresh_due(moved, …)` unchanged.
  `terminal_block_epoch()` = the D2 walk summing `session.block_epoch()` over
  every live terminal session (a closed tab drops its term → the component
  moves → re-derive drops its rows — D1/REQ-006 falls out).
- **D2/D3 get ONE home:** `fn all_terminal_sessions(&self) ->
  Vec<(PathBuf, crate::ContentId)>` — for each `shell.projects()` compute the
  project's `canonical_root` ONCE, then `project.terminal_grids()` ×
  `grid.states()` → the terminal ContentIds. Both consumers (the producer,
  the epoch shim) walk this list — the PR-1345 iteration has exactly one
  spelling; `terminal_grid_index()` appears nowhere.
- **The producer split (winners-only extraction — scan cost bounded):**
  1. `editor_problems::latest_failed_indices(blocks: &[(String,
     Option<String>, bool)]) -> Vec<usize>` — PURE per-pane fold: slice order
     IS execution order (BlockIndex); group by `(command, pwd)`; the LAST
     entry of each group governs; keep the groups whose last is
     `failed == true`; answer their indices. (Callers pass COMPLETED blocks
     only, so a still-running rerun is absent and the previous failure
     still governs — D5's semantics verbatim; `failed` comes from
     `exit_status_kind(state, exit_code) == Failure`, state-first.)
  2. `links::file_refs(output: &str) -> Vec<(PathBuf, usize)>` — PURE
     any-file extractor: per line, union `scan_links` File-refs-with-line
     and `parse_trace_frames` (the Python shape), 1-BASED lines, sorted +
     deduped (the sibling fns' discipline).
  3. The shim `terminal_problem_refs(&self) -> Vec<(PathBuf, u32)>`: for
     each `(root, tid)` in `all_terminal_sessions()` — collect the pane's
     COMPLETED blocks as `(command, pwd.unwrap_or_default(), failed)`,
     fold via (1), extract refs via (2) ONLY for the winning blocks
     (superseded output is never scanned), resolve each ref
     `resolve_under_root(root, path)` (the OWNING project's canonical
     root — D3), convert `line.saturating_sub(1) as u32`; finally
     sort+dedupe `(path, line)` across all panes. Fed to BOTH
     `problem_rows` call sites in place of `&[]`.
- **D-MSG — closed: the seam is untouched.** The message stays the seam's
  own "failed here (terminal)"; the tuple stays `(PathBuf, u32)`. Carrying
  the failing command would grow the seam + its tests for a hint #435's
  block-scoped work will supersede; deliberate, recorded.
- **D6 — the identity key grows `source` (5-tuple).**
  `keep_selection_by_identity(rows, key: Option<(&Path, u32, u32, Severity,
  ProblemSource)>, prev)`; `OpenProblems.selected_key` grows the field;
  `ProblemSource` already derives `Copy/PartialEq` (diagnostics.rs:174). The
  collision an LSP Error at char 0 vs a Terminal row on the same line is now
  distinct (the #327 C2 rule at two producers).
- **The panel glyph (zone B, POC-first):**
  `editor_problems::problem_glyph(source, severity) -> &'static str` —
  `Terminal` → `"❯"` (the block/prompt identity, danger-tinted like Error by
  the existing row tinting), else `severity_glyph(severity)`. The app row
  render (:18707 zone) swaps to it; the row shape is otherwise untouched.
  POC: `ProblemsPanel.tsx`'s `Problem` grows `source?: 'lsp' | 'terminal'`
  and the glyph slot renders `❯` for terminal rows — built + captured FIRST.
- **§20 confirmed** — behavior per the Warp block map (command/exit/cwd via
  DCS hooks); the aggregation consumer is shipped #327/#430 machinery; no
  Warp/Zed source. The producer is Marley-original fusion (the wedge).

### File manifest

- **POC half (FIRST):** `marley-web/.../components/overlays/ProblemsPanel.tsx`
  — `Problem.source` + the `❯` terminal glyph in the 16px slot; one mock
  terminal row in the demo data; typecheck + screenshot + READ.
- `crates/terminal_blocks/src/session.rs` (+ the model if that is where
  BlockList mutates) — `block_epoch` counter + bumps at the one transition
  choke + `block_epoch()` accessor + unit tests (both hook paths).
- `crates/marley_app/src/links.rs` — `file_refs` + units.
- `crates/marley_app/src/editor_problems.rs` — `latest_failed_indices`,
  `problem_glyph`, `keep_selection_by_identity` 5-tuple + adjusted tests.
- `crates/marley_app/src/app.rs` — `all_terminal_sessions`,
  `terminal_problem_refs`, `terminal_block_epoch`; the two fingerprint
  fields' type growth + their init/compare sites; both `problem_rows` call
  sites fed; the panel glyph swap; `selected_key` 5-tuple threading.
- `crates/marley_lsp/src/diagnostics.rs` — UNTOUCHED (D-MSG).
- `crates/marley_app/src/headless_drive.rs` — validate's drives.

### Regression test plan

| REQ | Test |
|---|---|
| REQ-001 | units: `file_refs` (rustc `path:line:col`, Python `File "…", line N`, mixed, dedupe, 1-based) + `latest_failed_indices` groups; headless drive: TWO projects, the NON-ACTIVE one's SECOND terminal tab carries the failed block (real PTY `sh -c 'echo src/x.rs:3:1: error…; exit 1'` — the marley_terminal integration proves real PTYs run headless), rows appear as Terminal-sourced, path under the OWNING root |
| REQ-002 | drive: Enter on a terminal row → file opens at the line (closed file), NavStack pushed; missing file → flash, no move |
| REQ-003 | unit: fail→pass supersedes (empty), fail→fail latest-only, distinct cwd separate, running-absent keeps previous failure; drive: rerun-success drops the rows at the next derive |
| REQ-004 | drive: zero LSP hosts → ⌘⇧M populates from terminal rows alone; ⌘⏎ materializes the #430 mb over them (bands + jump) |
| REQ-005 | drive: panel OPEN, a new block fails → the pair-fingerprint moves → rows appear without reopening; selection kept by the 5-tuple across producers (unit: the LSP-vs-Terminal same-(path,line,0,Error) collision) |
| REQ-006 | drive: close the failed block's terminal tab → next derive drops its rows (the epoch component moved; no latch) |
| REQ-007 | `git add -N` any new files before `scripts/gates.sh --diff`; gate GREEN (cov/MSI 100 on the new pure fns incl. `block_epoch`) |
| parity | POC panel with a terminal row (❯ glyph) ↔ live panel at the same state; pixel-sample the glyph slot + row chrome |

Uncoverable: none new — the shims ride the standing `mutants::skip` glue
policy; every decision-bearing line is in the pure fns.

### Risks

- **R1** — the epoch bump must sit at the ONE choke both hook paths share; a
  missed path = a silent no-refresh. Unit drives BOTH (pump bytes +
  `apply_hook`).
- **R2** — the fingerprint TYPE changes ripple (OpenProblems init :17062,
  the mb field init) — compiler-led, but the `!=` compare semantics must
  stay tuple-wise (never re-sum).
- **R3** — the multi-project drive's block mechanics: real PTY commands in a
  tempdir project; if pump-to-Finished proves flaky headless, fall back to
  driving the DCS hook bytes directly through the session's input path
  (both are real paths; validate picks the stable one).
- **R4** — derive cost: epoch-gated derives + winners-only output scanning;
  no per-tick rescan when nothing moved (the gates hold it).
- **R5** — panel display of a NON-ACTIVE project's path renders via
  `rel_under_root(active_root, …)` → falls back to the fuller path; ACCEPTED
  v1 (the jump resolves against the owning root regardless); recorded.

## Phase 3 — Implement

**React-first (built + verified FIRST):** `ProblemsPanel.tsx` — `Problem.source?:
'lsp' | 'terminal'`, `problemGlyph` (terminal → `❯`, else the severity glyph), one
mock terminal row (`failed here (terminal)` / block_status.rs:27, severity error →
sorts first). Typecheck green; driven at 5173 (⌘⇧M) and captured —
`scratchpad/433-poc-panel.png` READ: the ❯ row leads the panel in the existing row
grammar, danger-tinted, loc column intact.

**Rust, to the manifest:**
- `terminal_blocks/src/apply.rs` — `SessionModel.block_epoch: u64` with bumps at
  the THREE transition points (Preexec open; Precmd finish — inside the
  `current_mut` Some-arm, so a bare prompt re-stage does NOT bump; child-exit
  `finish_current_if_running`, same guard) + accessor. `current_mut` filters to
  Running (block.rs:205-209), so every bump is transition-exact by construction.
  session.rs — the public `block_epoch()` delegate (both `pump`'s DCS path and
  `TerminalSession::apply_hook` funnel into `SessionModel::apply_hook`, so the
  bumps are exhaustive — the D4 requirement).
- `links.rs` — `file_refs` (any-file union of `scan_links` File-with-line refs +
  `parse_trace_frame` per line; 1-based; sorted+deduped like the siblings).
- `editor_problems.rs` — `problem_glyph`, `latest_failed_indices` (HashMap
  last-wins over `(command, pwd)`, failed-only, ascending indices),
  `keep_selection_by_identity` grows `source` (5-tuple); the five existing test
  keys adapted with `ProblemSource::Lsp` appended (assertion values verbatim).
- `app.rs` — `all_terminal_sessions` (the ONE D2/D3 walk: projects ×
  terminal_grids × states, canonical root once per project),
  `terminal_block_epoch` (sum of per-session epochs), `terminal_problem_refs`
  (completed blocks → summaries → the fold → winners-only `output_text` scanning →
  per-root resolve → 0-based → cross-pane dedupe); BOTH gates became tuple PAIRS
  (`problems_mb_fingerprint: (u64, u64)` = (LSP epoch, terminal epoch);
  `OpenProblems.diag_fingerprint: (usize, u64)` = (LSP total, terminal epoch)) —
  never cross-lane sums; both `problem_rows` call sites feed
  `terminal_problem_refs()` instead of `&[]`; the panel glyph render swaps to
  `problem_glyph`; `selected_key` carries the 5-tuple (both ↑/↓ writers + the
  refresh writer).
- `marley_lsp/diagnostics.rs` — untouched (D-MSG held).

Deviations: ONE, recorded at inspect — the design manifest's "+ units" phrasing
implied birth-adjacent tests; the units land at P4 per this pipeline's own phase
split (both critics flagged the notes' original "none" as inaccurate). (One
mechanical slip fixed in-flight: the
`block_epoch` insertion initially displaced `blocks()`'s doc comment — rustdoc
`-D missing_docs` caught it immediately; re-glued.)

Checks: `cargo check --workspace --tests` green; clippy `-D warnings` green
(one doc-lazy-continuation reword); `fmt --check` clean.

## Inspect (Phase 3.5)

Three critics (correctness — the D2/D3/D4 named hunts; provenance/security;
simplification) + the lead's review. **Verdict: no blockers; D2 enumeration, D3
root discipline, and D4 gate folds all verified clean with evidence.** Ledger:

1. **[MED · simplification] the epoch gate paid per-project `canonicalize`
   SYSCALLS on every 16ms pump tick** while a surface was open —
   `all_terminal_sessions` canonicalized eagerly and the epoch consumer threw
   the root away (the hoist-once rule's exact violation, lib.rs's own comment).
   FIXED: the walk now pairs VERBATIM roots (IO-free); the producer — the only
   root consumer, epoch-gated — canonicalizes once per DISTINCT root per derive
   (memoized). The one-walk spelling (PR-1345) survives.
2. **[minor · correctness] the terminal gate component is a cross-SESSION sum**
   — a tab close can cancel against concurrent bumps in one gate read (stale
   rows persist until the next lifecycle event). ACCEPTED, comments amended:
   the exact shape of the precedented LSP `workspace_diag_epoch` (same
   host-drop window), epoch-0 closes benign, self-healing; the apply.rs field
   doc now records the summing consumer's window honestly instead of
   overclaiming.
3. **[low · provenance] terminal text now reaches the mb AUTO-READ lane** (a
   hostile build's output can mint rows whose refresh reads+excerpts files
   with no gesture). ACCEPTED-with-record: pre-existing #212 exposure class
   (click-to-open has no root jail by design — LSP jumps need it); the
   increment is bounded by the reused viewer guard chain (canonicalize + stat
   + 2MB + binary sniff + TOCTOU recheck); display keeps the honest full-path
   head. VALIDATE adds the out-of-root absolute-ref drive.
4. **[low · provenance] marley-web hygiene**: the #432 POC half was never
   committed there — a #433 POC commit would sweep it in. DEFERRED to the
   /commit step (the commit-gate hook blocks all `git commit` while the Marley
   receipt is stale): two ordered commits, #432's first.
5. **FIXED lows/nits**: three stale field docs (pair/5-tuple reality);
   `mutants::skip` dropped from `terminal_block_epoch` (twin parity with the
   unskipped LSP epoch sum — the P4 drives kill the sum mutants); the
   completed-filter made Finished-EXACT (`matches!(Finished)` — a live
   `Pending` state would have entered the fold and superseded without
   failing); `latest_failed_indices` takes borrow-shaped
   `&[(&str, Option<&str>, bool)]` (drops per-derive clones; terser fixtures);
   `problem_key` collapses the 5-tuple's three writer spellings to one
   (PR-1691); `file_links_on_line` collapses the duplicated per-line
   File-with-line match shared by `file_refs`/`diagnostics_for_file`.
6. **Recorded follow-ups (not taken, with reasons)**: the panel's LSP half
   staying `workspace_diag_total` (D4 recorded the split; switching to the
   epoch would close the panel's own rare same-count blind spot — a
   design-owned follow-up, not an inspect fix); collapsing the #289 gutter
   caller onto a filtered `file_refs` (spec Out — the lanes' divergence is
   recorded, deliberate; the caller-level collapse is a clean future chore);
   NOT folding `open_file_diagnostic_rows` onto the new walk (different root
   FAMILY — the F1 sync-timing rule — and different scope semantics; sharing
   would confuse, both critics concurred).
7. **Correctness clean-sweep highlights** (evidence in the critics' reports):
   exactly 2 epoch bumps per normal command; the child-exit + buffered-Precmd
   double-finish yields ONE bump (`current_mut` filters Running); every
   fingerprint write site updates BOTH pair components; `completed[i]` proven
   in-bounds; junk/out-of-root refs fail closed at the jump (no panic);
   `pump`→`ingest`→`apply_hook` is the only mutation funnel. Provenance:
   clean-room clean (❯ is Marley's own #193 prompt grammar; the message
   pre-exists at #327), zero PTY writes, no ReDoS surface (no regex), rows
   capped at 500 with the honest tail.
8. **Validate additions from inspect**: the out-of-root absolute-ref drive;
   assert the exact landing LINE in the jump drive (makes the 1→0-based
   conversion mutation-visible end-to-end); the first-command `(cmd, None)`
   pwd-group case (a pre-prompt failure is never superseded by a post-prompt
   rerun — D5-conformant, worth pinning); the materialize-time snapshot skew
   note (#430-inherited one-tick window, no action).

**Ledger appends**: no `F-` (no shipped-behavior bug — the MED was a
performance-shape defect caught pre-ship); no new `PR-` (the MED is an instance
of the standing hoist-once rule — recorded here as its walk-payload corollary).

## Phase 4 — Validate

**Units (10 new, all green):**
- terminal_blocks/apply.rs: `epoch_bumps_once_per_born_and_once_per_finish`
  (InitShell/Bootstrapped never bump; a bare re-prompt never bumps),
  `epoch_missing_session_preexec_does_not_bump`,
  `epoch_child_exit_finish_bumps_once_and_buffered_precmd_does_not_double` (the
  double-finish guard), `seed_finished_block_mints_via_real_transitions` (the
  cross-crate seed rides the real machine: +2 per block, command/pwd/exit/output
  round-trip).
- terminal_blocks/session.rs: `pump_dcs_bytes_bump_the_block_epoch` — the D4
  BOTH-paths proof (raw DCS bytes through pump = the same bumps).
- links.rs: `file_refs_unions_scan_links_and_trace_frames_one_based` (rustc
  arrow + Python frame + plain, sorted 1-based), `…dedupes_the_trace_fallback_
  double_hit`, `…ignores_lineless_paths_and_urls`.
- editor_problems.rs: `latest_failed_indices_supersession_table` (fail→pass
  drops, fail→fail latest-only, distinct cwd, the `(cmd, None)` pre-prompt
  group case from inspect, empty), `problem_glyph_arms`,
  `keep_selection_distinguishes_producers_on_the_collision_tuple` (the exact
  4-tuple collision `source` was grown for).

**New scaffolding**: `seed_finished_block_for_test` (doc(hidden), terminal_blocks)
— mints COMPLETED blocks through the REAL hook machine (register → stage pwd →
Preexec → styled output → Precmd), so app drives are deterministic with zero PTY
timing; its own unit pins the honest epoch bumps. `problems_rows_for_test`
accessor (rows content, not just counts).

**Drives (3 new, all green first run):**
- `terminal_problems_workspace_wide_headless` (REQ-001/002/004 + the PR-1345
  corollary): TWO projects; the failed block in the NON-ACTIVE project's SECOND
  terminal tab; the row lists as ❯ Terminal / Error / the OWNING root's
  canonical path / 0-based line 2; Enter opens the closed file with the caret on
  the EXACT line (the inspect's landing-line assert — the 1→0-based conversion
  is mutation-visible end-to-end); zero LSP hosts → ⌘⏎ materializes the #430 mb
  over the terminal rows alone (NoteMeta "failed here (terminal)").
- `terminal_problems_gate_supersede_and_close_headless` (REQ-003/005/006): with
  the panel OPEN — a passing rerun supersedes THROUGH the pair gate
  (`refresh_problems(false)` returns true on the epoch move, rows drop, no
  reopen); a new failure appears; a row appearing ABOVE the selection keeps it
  by the 5-tuple; `close_tab_at` drops the closed tab's rows at the next
  stateless derive while the sibling grid's row survives.
- `terminal_problems_out_of_root_ref_fails_closed_headless` (the inspect threat
  drive): an absolute out-of-root ref keeps its honest absolute path in the
  row; Enter flashes and moves NOTHING.

**Suites:** full `cargo nextest run --workspace` **2265/2265 PASS** + doctests
clean; clippy `-D warnings` green (one drive-helper too-many-args collapse:
`(p, grid_ix)` pair + the shared `/w` pwd hoisted).

**LIVE drive (bundled app, real pixels — captures READ):**
- `433-live-block.png`/`433-live-panel.png` context; `433-live-panel2.png` —
  REAL failed commands (`echo …app.rs:100:1: error…; false` twice + an
  `a.rs:3` variant) produced × blocks with underlined refs; ⌘⇧M (editor
  focused) opened the panel: **"2 problems" — both `❯ failed here (terminal)`
  rows** (`a.rs:3`, `app.rs:100`), ❯ danger-tinted, the ⌘⏎ hint, the
  supersession fold collapsing the duplicate command to one row set — with the
  status bar reading `lsp: failed`: the REQ-004 no-LSP case live.
- Harness lesson recorded: chain EVERY verb in ONE drive.swift invocation —
  events sent across separate tool-call shells drop when the host terminal
  regains frontmost (the README's warning, observed twice); the typed queue
  drains slowly under the restored heavy session (short commands or seeded
  blocks preferred).

**Parity pair:** `433-poc-panel.png` (implement, READ) ↔ `433-live-panel2.png`
(READ): identical structure — teal selected first row with ❯ + fixed message +
right-aligned loc; unselected row's danger-red ❯. Pixels: card bg
(26,27,31)↔(25,26,28) — the standing Δ≤4 family; selected-row accents
(74,191,207)↔(60,185,199) — the two sides' standing primary-token pair (not a
#433 delta; this ticket's only visual delta is the glyph, identical both
sides). Verdict: **PASS**.

**Gate:** `scripts/gates.sh --diff` (`git add -N` first, PR:747) → first run RED
on gate:5 alone — ONE missed mutant: the session-level seed DELEGATE's body →
`()` (mutation runs per-crate tests; only the cross-crate app drives called the
delegate). Fixed at source with a crate-local unit
(`seed_delegate_mints_block_and_epoch_on_the_session`) — the delegate now dies
in its own crate. Re-run: **GATE GREEN [diff] — 15/15** (MSI 100), receipt
written 2026-08-15 12:24. Validator lesson: a doc(hidden) cross-crate test
scaffold needs a unit IN ITS OWN CRATE — cross-crate callers don't count for
per-crate mutation.

## Phase 5 — Complete

- **§21 (a) CHANGELOG**: entry under Unreleased → Added (the fusion story: the
  #310 empty seam filled, the epoch pair gates, the ❯ distinction, the proof
  chain incl. the live no-LSP panel).
- **§21 (b) Architecture**: `docs/marley_architecture/editor.md` problems
  section + `00-overview`-adjacent terminal notes — the two-producer doctrine
  is now LIVE both lanes; the block-epoch signal recorded as the terminal
  lane's publish_epoch twin (deferred list loses the producer).
- **§21 (c) Parity sync**: MARLEY-PARITY ProblemsPanel row — #433 LANDED (the
  ❯ source glyph both sides; the terminal mock row); the stray-#432 hygiene
  resolved by ordered commits at /commit.
- **Knowledge (§19)**: `AD-claude-433` (the producer architecture: source-owned
  epoch, pair gates, winners-only scanning, the verbatim-root walk),
  `L-claude-433` (per-crate mutation vs cross-crate scaffolding; the one-shot
  drive.swift chaining lesson).
- Ticket closed → `tickets/closed/`; pipeline pair archived.
