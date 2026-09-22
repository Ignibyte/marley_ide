# 434-runnables-gutter-run-block — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-434-runnables-gutter-run-block.md
- **Pipeline spec:** 434-runnables-gutter-run-block.spec.md

## Phase 1 — Plan

- **Request:** TICKET-434 — Runnables: gutter ▶ spawns a command Block. Phase C's
  first named thread (shelf: `docs/planning/design-notes/m33-tail-and-wedge-shelf.md`;
  roadmap: "tree-sitter runnables → a gutter run-button → spawn as a first-class
  command Block"). Systems: `crates/syntax` (the 6th node API), `marley_app` gutter
  render + click + spawn wiring, `marley_terminal` (consumed as-is). Drafted at HEAD
  52a4c4d.
- **Classification / tier:** feature, M, ONE shippable slice — the pure API, the ▶,
  and the spawn are one behavior (a ▶ that spawns nothing, or a spawn with no
  affordance, is not shippable alone). IF Phase 2 must split, the line is: (a)
  `runnables_in` + gutter ▶ rendering (inert affordance behind the drive), then (b)
  click→spawn — never inside the spawn path itself.
- **Recall (knowledge codes actually read, with what each contributes):**
  - `AD-claude-caller-gates-language-not-the-pure-syntax-primitive-001` — the pure fn
    parses Rust unconditionally and CANNOT self-gate; the app call site gates on
    `language_of == Rust` (copy #340/#305's gate, not the ungated #329/#330). D1.
  - `AD-claude-syntax-node-range-api-foundation-001` — the crate answers node
    QUESTIONS; callers never hold a Node/Tree; a throwaway parse per recompute is the
    settled v1 cost model. `runnables_in` follows the free-fn parse-only shape
    (#340's), not the session-taking shape.
  - `PR-claude-language-specific-pure-primitive-needs-caller-side-gate-001` — the
    failure mode the D1 gate prevents (non-Rust parsed as Rust = silently wrong).
  - `PR-claude-verify-adopted-grammar-by-running-its-query-001` — grammar claims are
    verified by RUNNING against real fixtures, not by reading .scm — hence the plan
    spike (below) and REQ-001's fixture-verbatim unit table.
  - `PR-claude-verify-tree-answers-directly-at-pre-edit-caret-001` — spike the tree
    shape at plan before scoping on it; done (this plan's probe).
  - `PR-claude-429-a-gestures-verify-their-own-preconditions-not-ambient-state-001` —
    the click verifies its own target (idle pane exists) and bails closed. D4/REQ-005.
  - `PR-claude-verify-every-specified-trigger-is-actually-wired-001` — the ▶ click is
    the one trigger; inspect diffs affordance-vs-wiring anyway (P3.5).
  - `PR-claude-intent-add-new-files-before-diff-mutation-001` — new pure files enter
    the diff gate via `git add -N` (REQ-007 wording).
  - `AD-claude-git-gutter-state-instance-scoped-001` — the #328 lane's state scoping;
    the runnable row-set follows the same captured-per-frame map shape as
    `foldable_header_rows`/`git_marks_for_active`.
- **Completed-pipeline recall:** `completed/430-problems-multibuffer.spec.md` — the
  batch's quality bar + the fail-closed doctrine wording. Queued sibling
  `431-displaymap-excerpt-unification.{spec,notes}.md` — this batch's template shape.
  The #304/#305/#340 histories live in the code headers read below (their pipeline
  docs predate the current numbering style).
- **Discovery (what a designer must verify, file:line at HEAD 52a4c4d):**
  - **The five existing node APIs (the shape to match)** —
    `crates/syntax/src/lib.rs:14-16` re-exports; `symbols.rs:63 file_symbols` (#304 —
    runs the grammar's OWN `tags.scm` via `parse.rs:22-29 tags_query()`, compiled once
    per process); `fold.rs:42 fold_regions` (#305 — the ITERATIVE `TreeCursor` walk
    `collect_fold_regions` :54-80, stack-safe, the walk idiom `runnables_in` copies);
    `lib.rs:646 matching_delimiters_in` (#340 — parse-only free fn, total, the
    "spike-verified on tree-sitter-rust 0.24.2" documentation tradition);
    `lib.rs:488 enclosing_ranges` (#329) + `:738 all_headers` (#330). Dep:
    `crates/syntax/Cargo.toml:13` `tree-sitter-rust = "0.24"` (0.24.2 resolved).
  - **The tree-sitter query leg (HIGH-YIELD finding)** — tree-sitter-rust 0.24.2 ships
    exactly three queries (`bindings/rust/lib.rs:43-49`): HIGHLIGHTS / INJECTIONS /
    TAGS. **No runnables.scm exists to adopt**, and `queries/tags.scm` (read in full)
    captures `@definition.*`/`@name` only — NO attribute capture — so `#[test]`
    detection is necessarily OUR OWN query or walk. Zed ships its own per-language
    `runnables.scm` but those FILES are GPL (fusion doc §1's provenance table) — the
    mechanism is public, the text is walled.
  - **The plan spike (scratchpad `tsprobe`, tree-sitter-rust 0.24.2, real parses):**
    (1) attributes are preceding SIBLING `attribute_item`s of `function_item` —
    stacked attributes (`#[test]` + `#[ignore]`) are a contiguous sibling run, so
    detection scans the run, not one anchored neighbor; (2) the attribute path node is
    `identifier` text `test` for `#[test]`, `scoped_identifier` text `tokio::test`
    for `#[tokio::test]` (rule: == `test` or ends `::test`), and `#[cfg(test)]` is
    `identifier` `cfg` + arguments `(test)` — excluded by the path rule with no
    special case; `#[rstest]` does NOT match (recorded Out); (3) `fn main` parent
    kinds: `source_file` (runnable) vs `declaration_list` (impl method — not) vs
    `block` (nested fn — not); (4) the in-file mod chain falls out of `mod_item`
    ancestors (`["outer","tests"]` in the probe) — `runnables_in` needs no
    file-system knowledge; (5) `function_item.start_position().row` is the `fn` line
    itself (attributes sit on their own rows) — the ▶ row is the fn line, matching
    Zed's placement.
  - **The gutter (where ▶ renders)** — `crates/marley_app/src/app.rs`: the editor
    `uniform_list` :7202; per-row flex :7573-7617 — the #328 git lane (3px bar)
    :7588-7600, the number cell with the #289 danger tint :7601-7617, first-segment
    gating :7612 (#426); the fold capture `row_fold` :7224-7228 and the gutter-click
    arm :7680-7692 (`row_fold.is_some() && x < x0` → `toggle_fold_at_row`, early
    return before the caret logic). **Today's gutter renders NO per-line icon** — the
    #305 fold affordance is a click ZONE plus the "⋯ N lines" EOL inlay
    (`editor_phantoms` :15923-15943), so ▶ is the first rendered gutter icon; the POC
    decides its cell and the D7 hitbox split. The caller-gate + captured-map pattern
    to copy: `foldable_header_rows` :16289-16314 and `toggle_fold_at_row`
    :16246-16261 (`language_of(&path) == Language::Rust`).
  - **The spawn seam (Block = typed input)** — `on_submit` app.rs:3233-3246
    (`history.record` + `session.write_command(&line)` + buffer/caret/selection/
    viewport reset + pump); `write_command`
    `crates/terminal_blocks/src/session.rs:347-352` (appends `\r\n`; #423
    `child_exited` state guard; package name `marley_terminal`); Block framing comes
    from shell integration around the PTY — a written command IS a typed command, so
    the Block/status-pill model applies with zero new machinery. **The programmatic
    precedents:** `rerun_block` :8958-8972 (#175 — the #40 idle guard
    `!is_command_running()` then `write_command`; "never inject mid-command");
    `rerun-last-failed` :11060-11096 (#292 — reachable with the EDITOR focused,
    walks the workspace grid for IDLE panes, sorts by `PaneId.0`, picks lowest — the
    D4 resolution shape minus the failure filter); `open-terminal-here` :11097-11107
    (#294 `spawn_terminal_tab_in` — the auto-spawn fallback D4 deliberately does NOT
    take in v1). Terminal existence: `try_workspace` :6282-6286 returns `None` when
    the project has no terminal tab (#391/#392); `workspace_terminal(_mut)`
    :6384-6397 resolves pane→content. Status truth: `block_status.rs:26-34
    exit_status_kind` over `BlockState`/`ExitCode`. Bail feedback: `flash.rs:9,21`
    (`FLASH_TICKS`, `Flash::new`).
  - **The React side** — `marley-web/docs/MARLEY-PARITY.md:37` (editor row) + `:611`
    (`components/EditorView.tsx` ↔ `editor_surface.rs`/`code_view.rs`/…); zone B =
    free design space (`:75`), and the #418 inversion precedent covers new-affordance
    design source. `components/EditorView.tsx:295` is the gutter cell render;
    `components/TerminalView.tsx:21` already exposes `onExecuteCommand` — the POC
    spawn seam exists.
- **Decisions:** D1 pure/parse-only/caller-gated; D2 marley_syntax toolchain-free
  (command mint is a pure app seam); D3 spawn IS typed input (no new process spawn —
  §14 adapter boundary); D4 #292-shaped deterministic target, #40 idle guard,
  flash-and-bail fail-closed (no v1 auto-spawn); D5 ▶ reflects the file,
  reveal-on-spawn; D6 substring filter without `--exact` + plain `cargo run`, limits
  recorded (over-match, workspace-wide cwd, multi-bin, pane cwd as-typed); D7 the ▶
  owns its hitbox beside the fold zone. Full text in the spec.
- **Prior-art verdicts (§20, three legs):**
  1. **Zed behavior** (`docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md`,
     research only, no Zed source): the discovery pipeline is worth matching
     (query-marked `#[test]`/`fn main` → cached gutter play buttons → click runs the
     mapped command); the SINK is where Zed degrades (task = a terminal tab +
     `TaskState` icon + a plain summary line; no blocks; "current command" known only
     by polling the process table) — Marley's deviation (spawn as a Block via typed
     input) is the wedge, per the doc's own reimplement guidance. Template/variable
     resolution + reveal knobs are mapped there and deliberately deferred.
  2. **Published:** cargo/libtest filter semantics settle D6 — the filter is a
     SUBSTRING over the full test path; `-- --exact` needs the crate-internal path
     `src` cannot supply, so omitting it is correctness, not laziness (over-match
     recorded); `cargo run` needs `--bin` only in multi-bin packages (out).
     tree-sitter query docs: anchors/predicates available if Phase 2 prefers a query
     over the walk — both public mechanism.
  3. **Permissive deps:** tree-sitter-rust 0.24.2 ships NO runnables query and
     `tags.scm` captures no attributes → our own detection is REQUIRED (the leg's
     decisive finding); the engine's sibling/ancestor node API covers the spike-pinned
     walk. gpui: the gutter click zones + row elements already in-tree (#305/#328) —
     no new capability. In-tree art is the rest: `collect_fold_regions`'s iterative
     walk, `symbols.rs`'s capture adaptation, #175/#292's injection discipline.
     Verdict: import nothing; write one small pure API on shipped substrate.

## Phase 2 — Design

Designed at HEAD 52ccdd2 (post-431/432/433; the Phase-1 seams re-checked — line
drift only; `rerun-last-failed` walk now :11279, `foldable_header_rows` :16590).

- **D-WALK — the detection is an ITERATIVE `TreeCursor` walk, not a query** (the
  fold.rs `collect_fold_regions` idiom): the spike's decisive facts — a stacked
  attribute run is a CONTIGUOUS SIBLING scan and the mod chain is an ANCESTOR
  accumulation — are walk-natural and query-awkward (no clean "contiguous
  preceding siblings" predicate); the crate's own precedent walks. Shape:
  `crates/syntax/src/runnables.rs` —
  `pub struct Runnable { pub row: usize, pub kind: RunnableKind }`,
  `pub enum RunnableKind { Test { path: String }, Main }`,
  `pub fn runnables_in(src: &str) -> Vec<Runnable>` — fresh `rust_parser()`
  parse; walk maintaining a `mod`-chain stack (`mod_item` push/pop on
  enter/leave); at each `function_item`: (a) scan the contiguous run of
  preceding `attribute_item` siblings for a path == `test` or ending `::test`
  → `Test { path: mods.join("::") :: fn_name }`; (b) name == `main` AND parent
  kind == `source_file` → `Main`. Document order; total (§14); row =
  `function_item.start_position().row` (0-based fn line). Fixtures = the spike
  cases verbatim (REQ-001/002).
- **D-MINT — the command seam** in NEW pure `crates/marley_app/src/runnables.rs`:
  `pub fn run_command(kind: &RunnableKind) -> String` (`Test{path}` →
  `format!("cargo test {path}")`; `Main` → `"cargo run"`); plus the row-set
  helper `pub fn runnable_rows(rs: &[Runnable]) -> HashMap<usize, usize>` (row →
  index — the render's per-frame captured map, the `foldable_header_rows`
  shape). marley_syntax stays toolchain-free (D2).
- **D-MEMO** — `runnables_cache: RefCell<Option<(u64, u64, Vec<Runnable>)>>`
  keyed (nonce, version) — the `fold_proj_cache` idiom; recompute only when the
  buffer changes; the caller gate (`language_of == Rust`) sits in the accessor
  `active_runnables(&self) -> Vec<Runnable>` (empty off-Rust/no-editor; the
  #340/#305 gate copied verbatim).
- **D-TARGET — extract the ONE idle-pane walk** (PR-1691): the #292 arm's
  candidate collection generalizes into
  `fn idle_workspace_panes(&self) -> Vec<(u64, PaneId)>` (idle terminals of the
  active project's workspace grid, sorted by `.0`); the runnable click takes the
  LOWEST; `rerun-last-failed` re-expresses over it (filter its failure condition
  on top). No terminal / none idle → `None` → flash-and-bail (D4).
- **D-SPAWN** — `run_runnable(&mut self, kind)` : resolve target → mint command →
  the #175/#292 typed-input tail (`history.record` + `write_command` + pump) →
  reveal the terminal tab (`switch_tab` to the workspace tab index) — D3/D5.
- **D-HITBOX (D7)** — the ▶ is its own gutter element with its own
  `on_mouse_down` that acts and returns; the fold zone check (`x < x0`) excludes
  the ▶ cell's x-range... exact split PINNED AT POC: the ▶ replaces the number
  cell's leading pad on runnable rows (first segment only), the number keeps its
  right-aligned position; the fold-zone click arm keeps its `x < x0` guard and
  the ▶'s handler runs FIRST (gpui child handlers fire before the row's).
- **§20 confirmed** — behavior from the fusion doc only; the sink deviation IS
  the ticket; no Zed source/scm text; the walk is spike-derived on the public
  grammar.

### File manifest
- **POC half (FIRST):** `EditorView.tsx` (the gutter ▶ cell on runnable rows —
  design the visual: Chad's minimal chrome, muted idle → accent hover),
  `App.tsx` (runnable rows for the active mock file + click → spawn),
  `TerminalView.tsx` (receives via the existing `onExecuteCommand`).
- `crates/syntax/src/runnables.rs` (NEW) + `lib.rs` re-export — the API + the
  walk + fixture units.
- `crates/marley_app/src/runnables.rs` (NEW) — `run_command`, `runnable_rows` +
  units.
- `crates/marley_app/src/app.rs` — `runnables_cache` + `active_runnables`
  (gated memo), the gutter ▶ render + click, `idle_workspace_panes` extraction
  (+ #292 re-expression), `run_runnable` spawn + reveal.
- `crates/marley_app/src/headless_drive.rs` — validate's drives.

### Regression test plan
| REQ | Test |
|---|---|
| REQ-001 | syntax units: the spike fixtures verbatim — #[test], stacked, #[tokio::test], nested mods path, #[cfg(test)]-excluded, #[rstest]-excluded, fn-in-fn excluded, document order, 0-based fn row |
| REQ-002 | syntax units: top-level main vs impl-method vs nested; empty/garbage → empty |
| REQ-003 | app unit on `runnable_rows`; headless drive: Rust file → ▶ rows exact; non-Rust file → none (gate) |
| REQ-004 | headless drive: click ▶ (or `run_runnable` direct + the real click in live) → ONE new Block whose command == the mint; history recorded; terminal tab revealed |
| REQ-005 | unit on the target walk (busy-only → None) + drive: no terminal / all busy → flash, zero writes |
| REQ-006 | drive: fold-zone click on a ▶ row toggles the fold exactly as today; the ▶ click toggles nothing |
| REQ-007 | `git add -N` the two new files; gate green (cov/MSI 100) |
| parity | POC gutter ▶ + spawned block ↔ live pair, pixel-sampled |

### Risks
- R1: gpui handler ordering for the D7 hitbox (child-before-row assumed — the
  #428 mb click precedent; verify at implement, else geometry-guard the row arm).
- R2: the #292 re-expression must keep its failure-filter semantics byte-stable
  (its drive coverage pins it).
- R3: `write_command` needs the pump to frame the Block headless — the drives
  follow the on_submit tail exactly.
- R4: attribute-run scan must not cross non-attribute siblings (doc comments are
  siblings too — the spike's contiguity rule; a fixture pins a doc-commented
  test).

## Phase 3 — Implement

**React-first (built + captured FIRST):** `EditorView.tsx` — a regex STAND-IN
detector (like CELL_W; the Rust side runs the real walk) marks `fn main` /
`test`-attributed fns; the gutter cell became a right-justified flex with the ▶
LEFT of the number (muted/50 idle → primary hover, its own onClick with
stopPropagation — D7); `onRunCommand` prop; `Workspace.tsx` wires it to
`handleExecuteCommand(cmd)` + reveal (`activeSection: 'terminal'`);
`docShare.ts`'s mock `code_syntax.rs` grew `fn main` + a `#[test]`/`#[tokio::
test]` mod. Typecheck green. DRIVEN: `434-poc-gutter2.png` (▶ on the fn rows in
the minimal grammar) and `434-poc-spawned.png` (the click spawned `cargo run`
as a first-class framed BLOCK in the terminal and revealed it — the mock
simulator's "command not found: cargo" is its own vocabulary, the FLOW is the
deliverable). Both READ. POC gotcha recorded: `localStorage.clear()` drops the
demo workspace — re-enter via "Open Folder…".

**Rust, to the manifest:**
- `crates/syntax/src/runnables.rs` (NEW) — `Runnable`/`RunnableKind` + the
  ITERATIVE TreeCursor walk (`collect_fold_regions` shape) with a depth-keyed
  mod-chain stack; `classify_function` gates MODULE SCOPE by parent kind
  (`source_file`/`declaration_list` — a `block`-nested fn is never runnable,
  REQ-001/002's rule the draft first missed and the fixtures caught);
  `has_test_attribute` scans the CONTIGUOUS preceding `attribute_item` run
  (path == `test` or ends `::test`); 4 fixture units (spike cases verbatim:
  mod-chain paths, stacked/tokio, cfg/rstest/nested/doc-split exclusions,
  main's parent gate, totality, the fn-line row) — all green FIRST RUN.
- `crates/marley_app/src/runnables.rs` (NEW) — `run_command` (D6) +
  `runnable_rows` + units.
- `app.rs` — `runnables_cache` ((nonce via the registry instance, version)
  memo — the fold_proj_cache idiom), `active_runnables` (the D1 gate),
  `idle_workspace_panes` (the ONE #292-shaped walk, `.0`-sorted),
  `run_runnable` (flash-and-bail → mint → `history.record` + `write_command` →
  reveal via `terminal_grid_index`+`switch_tab`), the gutter ▶ mini-cell
  (RESERVED on every row — uniform width, no per-row jitter; populated
  first-segment-only; its own `on_mouse_down` with `app.stop_propagation()` —
  D7), and the #292 arm re-expressed over the shared walk (first idle pane
  WITH a failure == the old lowest-candidate pick).

Deviations: the gutter grew ONE reserved ▶ cell on every row (the POC squeezes
▶ into its fixed w-12; the Rust gutter is intrinsic-width, so a per-row ▶
would misalign rows — the uniform reservation is the clean equivalent;
recorded for the parity pair's judgment). Phase-gate note: the hook needed the
implement skill re-entered (the active-phase marker was stale from #433's
validate).

Checks: `cargo check --workspace --tests` green; clippy `-D warnings` green;
fmt clean.

## Inspect (Phase 3.5)

Three critics (correctness — the walk/spawn/hitbox hunts; provenance/security —
with an EMPIRICAL grammar probe harness; simplification) + the lead's own
probes (run DURING the critic wait — both landed real findings first). Ledger:

1. **[MED ×2 · correctness] the ▶ mini-cell broke BOTH row-geometry mirrors**:
   `code_area_left_px` (the h-scroll thumb's "aligned by construction" origin)
   still composed the pre-434 row — the thumb sat one cell+gap left of the code
   on every h-scrolled editor, with the pinned t341 test PINNING THE STALE
   value; and the sticky-header band (its own spacer-built row prefix) drifted
   the same amount. FIXED: the composition grew the ▶ term (3·GAP + cell) with
   t341 re-pinned (incl. the NaN floor case → 27), and the band gained the
   matching cell-width spacer. → PR appended (the row-prefix mirror class).
2. **[HIGH/MED ×2 · both critics] the insertion displaced `rerun_block`'s doc +
   `mutants::skip`** — `active_runnables` wore the wrong doc + a doubled skip;
   `rerun_block` (a deliberately masked shim) went BARE — a latent MSI miss on
   the next full sweep. FIXED (restored; duplicate dropped).
3. **[LOW · provenance, probe-proven] the `$VAR` metavariable hole**: grammar
   error-recovery can mint `fn $HOME()` at module scope from NON-COMPILING
   input; its raw `$…` text would reach the shell (expansion only — the probe
   proved the payload is always ONE shell word, `;|&$(` unreachable — no
   execution). FIXED AT SOURCE: the name node must be kind `identifier`
   (a metavariable never classifies) + a pinning unit. `r#ident`'s `#` verified
   benign (mid-word, comment-inert; rustc lists raw-ident tests verbatim).
4. **[LOW · lead's own probe, confirmed by both critics] impl-body `#[test]`
   over-match**: `declaration_list` is ALSO an impl/trait body — `impl S {
   #[test] fn t() {} }` was marked (a compile error libtest can never run).
   FIXED: the `declaration_list`'s parent must be the `mod_item` itself +
   pinning unit. RESIDUAL (accepted, recorded): a `mod` nested INSIDE a fn
   body still passes (it IS a mod_item), compiles, and libtest collects 0 —
   the ▶ green-runs nothing; degenerate shape, honest limit.
5. **[MED · simplification] the hand-rolled nonce** re-derived around the
   canonical `s.active_nonce()` with a dead-or-aliasing `unwrap_or(0)` arm.
   FIXED (one line, the fold-cache precedent's exact spelling).
6. **[MED · simplification] `run_runnable` dropped the R39 viewport re-anchor**
   — a ▶ into a scrolled-up pane would reveal a tab showing OLD scrollback
   while the doc claimed typed parity. FIXED (`viewport = Viewport::new()`);
   the reveal also collapsed onto `jump_to_pane` (#174 — the ONE pane-reveal
   derivation, and it FOCUSES the spawned pane so ⌃C works immediately).
7. **[LOW ×3 · simplification] all applied**: `idle_workspace_panes` drops the
   vestigial `(u64, PaneId)` tuple + reuses `workspace_terminal` (one
   resolution spelling); `runnable_rows` returns row→KIND directly (the
   design's row→index indirection had no surviving consumer — amendment
   recorded); the pump-absence judged acceptable (the ambient per-tick loop —
   `rerun_block`'s exact posture; the P4 drives must pump explicitly, noted).
8. **Kept with reasons**: `pub(crate) run_runnable` (the P4 drives call it);
   the mod-chain STACK over the critic's ancestor-chain alternative (equal
   correctness both probe-verified; the stack matches the crate's
   walk-in-one-pass shape — revisit only if a third walker forces the shared
   `preorder` driver the critic sketched); the sibling-mod pop logic VERIFIED
   correct by both the lead's and the critic's independent probes (a::x, b::y,
   z — no leakage; the pop fires exactly on returning to the mod's own node).
9. **Clean sweeps**: the D7 hitbox verified against gpui 0.2.2 SOURCE
   (bubble-phase, deepest-first, `stop_propagation` breaks the dispatch —
   load-bearing, since every runnable fn row is also a foldable header); the
   #40 guard sync-tight (no TOCTOU); the #292 re-expression EXACT; clean-room
   clean (no .scm text, node-kind names from the MIT grammar's public
   node-types.json, no Zed vocabulary); trigger-wiring sweep clean; no new
   spawn/unsafe/secrets; the POC stand-in honest.
10. **Process for P4**: `git add -N` the two new files before the gate
    (REQ-007); the drives pump explicitly after `run_runnable`.

**Ledger appends**: `PR-claude-row-prefix-mirrors-move-in-lockstep-001`
(prevention-rules — the finding-1 class); no `F-` (nothing shipped — every
find pre-gate).

## Phase 4 — Validate

**Units (13 across the phases, all green):** syntax — the 4 fixture suites +
the 3 inspect pins (metavariable gate, impl-body exclusion, sibling-mod
paths); app runnables — the D6 mint + the row→kind map (amended); h_scroll —
t341 re-pinned to the ▶-bearing composition (incl. the NaN floor → 27).

**Drives (2 new, green):**
- `runnables_gate_and_rows_headless` (REQ-003 model half): the Rust file
  yields exactly Main@0 + Test{t::a}@4 through the gated memo; the markdown
  file yields none (the parser is never invoked off-Rust).
- `runnables_spawn_and_fail_closed_headless` (REQ-004/005): `run_runnable`
  records the exact D6 command in the pane's HISTORY and reveals the terminal
  tab; with the pane made BUSY through the REAL hook machine (the #433 seed
  registers; a bare Preexec opens a running block) the second spawn flashes
  "No idle terminal" with byte-identical history before/after — zero side
  effects.

**Suites:** full `cargo nextest run --workspace` **2277/2277** + doctests
clean; clippy `-D warnings` green.

**LIVE drive (bundled app — captures READ):**
- `434-live-open.png` — a seeded demo file (`aaa_run_demo_434.rs`, removed
  after): the gutter renders ▶ on EXACTLY rows 1 (`fn main`), 8
  (`#[test] fn demo_passes`), 13 (`#[tokio::test] async fn async_demo`) —
  nothing on `#[cfg(test)]`, `mod tests`, or attribute rows; muted, left of
  the number, first-segment cells — REQ-003's pixel half.
- `434-live-spawned.png` — the ▶ click SPAWNED `cargo test tests::…` as a
  first-class framed BLOCK (fold arrow + status + `Finished test profile in
  1.74s` + `Running unittests…`), the terminal tab REVEALED and the pane
  FOCUSED (jump_to_pane), the tab retitled `cargo` — D3/D5 live.
- `434-live-done.png` — the block streaming `test result: ok … filtered out`
  across the workspace binaries: the D6 substring filter running exactly as
  recorded (the honest over-match posture, live). App quit clean; the demo
  file removed.
- Harness notes: ⌘P raced twice (typed-queue); the reliable route was ⌘⇧F →
  Enter (the walk reads disk live — the tree does NOT rescan for files
  created after launch, recorded).

**Parity pair:** `434-poc-gutter2.png` + `434-poc-spawned.png` (implement,
READ) ↔ `434-live-open.png` + `434-live-spawned.png` (READ): the ▶ grammar
identical (left of the number, muted idle, fn-row-only); the spawn grammar
identical (command as a framed block + reveal). Bodies sample the standing
(11,12,15)↔(14,15,17) family. Recorded deltas: the Rust gutter RESERVES one
▶ cell on every row (intrinsic-width gutter — the uniform-reservation
deviation from Phase 3, kept); the POC's simulateCommand vocabulary lacks
cargo (its error block is the mock's own). Verdict: **PASS**.

**Gate:** `scripts/gates.sh --diff` (both new files `git add -N`-staged,
REQ-007) → first run RED on gate:4 alone — ONE uncovered region: the
`if let Some(path) = attribute_path(…)` None arm, grammatically UNREACHABLE
from source text (tree-sitter always mints the inner `attribute` node). Fixed
at source by RE-EXPRESSION, not exclusion: `is_some_and` folds the unreachable
arm into the covered not-a-test path (§14 totality kept; the #426
state-the-invariant-delete-the-mutation-surface discipline) + a
malformed-attribute fixture (`#[]`, bare `#`) pinning the skip-not-fatal
behavior. Re-run: **GATE GREEN [diff] — 15/15**, receipt 2026-08-15 13:28.

## Phase 5 — Complete

- **§21 (a) CHANGELOG**: entry under Unreleased → Added (Phase C opens: the
  walk, the ▶, the Block sink — where Zed degrades to a tab, Marley spawns a
  Block).
- **§21 (b) Architecture**: editor.md gains the runnables section (the 6th
  node API + the D1–D7 shape + the fusion-wedge framing); the deferred list
  re-cut.
- **§21 (c) Parity sync**: MARLEY-PARITY EditorView row — #434 LANDED (the ▶
  grammar both sides; the uniform-reservation Rust delta recorded).
- **Knowledge (§19)**: `AD-claude-434` (the runnables architecture: walk
  shape, source-owned detection vs app-owned mint, the Block sink doctrine);
  `L-claude-434` (re-express unreachable arms instead of excluding them —
  the gate:4 lesson; the row-prefix-mirrors PR from inspect).
- Ticket closed → `tickets/closed/`; pair archived.
