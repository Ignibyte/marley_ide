---
pipeline_id: 82f2147e-3b0d-4933-855e-b283a7064ef6
ticket: docs/planning/tickets/open/TICKET-434-runnables-gutter-run-block.md
status: Phase 5 — Complete PASS
title: Runnables — gutter ▶ spawns a command Block (M33, Phase C opens)
type: feature
milestone: M33
references: [docs/planning/design-notes/m33-tail-and-wedge-shelf.md]
---

## Title

Phase C's first thread — the fusion wedge opens: a pure `marley_syntax` runnables API
(the 6th node API — Rust-only, caller-gated like #304 file_symbols / #340
bracket-match) marks the runnable lines of the active Rust file (`#[test]`-family fns
and the top-level `fn main`); the editor gutter renders a ▶ affordance on exactly those
lines; clicking SPAWNS the mapped command (`cargo test <in-file-path::name>` /
`cargo run`) as a first-class command Block in the workspace terminal — status pill,
framed output, exit code, the whole shipped block model. Zed's runnables pipeline is
excellent up to the point of spawn, where it degrades into a terminal TAB with a text
summary line (it has no block model at all); Marley routes the same discovery into a
Block and wins the sink. #435 (per-block rerun + block-scoped jump-to-failure) builds
on the run-block identity this ticket mints.

## Scope

### In
- `marley_syntax::runnables_in(src) -> Vec<Runnable>` — pure, parse-only (a fresh
  `rust_parser()` parse, the #340/#305 shape), total (§14: empty/garbage → empty Vec),
  document order. Marks: (a) a `function_item` whose contiguous run of preceding
  `attribute_item` siblings contains an attribute path that IS `test` or ENDS `::test`
  (`#[test]`, `#[tokio::test]`, stacked `#[ignore]` handled; `#[cfg(test)]` naturally
  excluded — its path is `cfg`) → `Test { path }` where `path` is the in-file
  `mod`-ancestor chain joined with the fn name (`tests::parses_ok`, nested mods
  included); (b) a top-level `fn main` (parent `source_file`) → `Main` (an impl-method
  `main` and a block-nested `fn` are NOT runnable — parent-kind gated). All node shapes
  spike-pinned on tree-sitter-rust 0.24.2 (notes).
- The command mint as a separate PURE app-side seam (marley_syntax stays
  toolchain-free): `Test{path}` → `cargo test <path>`; `Main` → `cargo run`.
- The app caller gate + per-(nonce, version) recompute of the runnable row set —
  copying the #340/#305 `language_of(path) == Language::Rust` gate
  (AD-claude-caller-gates-language-not-the-pure-syntax-primitive-001), empty off-Rust.
- The gutter ▶: rendered on runnable rows (first segment only, the #426 rule) as its
  OWN click target, coexisting with the #305 fold gutter-click and the #328 git lane /
  #289 diagnostic tint on the same row. Designed React-first (below) — today's gutter
  has NO per-line icon, so the POC decides the visual (cell, hover, color).
- The click: resolve a target pane by the #292 deterministic walk over the active
  project's terminal grid (IDLE panes only — the #40 never-inject-mid-command guard —
  lowest `PaneId`), then spawn through the EXISTING typed-input path (`history.record`
  + `session.write_command` + pump — the #175 `rerun_block` / #292 shape), and reveal
  the terminal tab. No idle target → flash-and-bail, zero side effects.
- Headless drives + unit suites per the EARS table; the React↔Marley parity pair.

### Out (explicitly deferred)
- #435's per-block rerun affordance and block-scoped jump-to-failure (this ticket only
  mints the run Block; identity metadata beyond the command line is #435's call).
- Bench (`#[bench]`), doc-test, and mod-level ("run all tests in mod") runnables;
  custom harnesses (`#[rstest]`, `#[test_case]` — spike-verified NOT matched; recorded).
- Multi-language runnables (Python/TS/etc — the #315 axis someday) and LSP-provided
  runnables (rust-analyzer's request; Zed merges them — later).
- A run-with-args prompt, task templates / `tasks.json` / variable substitution, and
  reveal/hide policy knobs (Zed's `TaskTemplate` model — the fusion doc maps it; a
  later Phase C thread).
- Package/binary inference: no `-p <crate>` / `--bin` derivation and no `--exact` (the
  file's crate-internal path prefix is not derivable from `src` alone) — the honest v1
  filter semantics + limits are locked in D6.
- Auto-spawning a terminal when the project has none (#294's verb exists but stays
  unwired here — the no-target case flashes; D4), and any cwd rewriting of the target
  pane (the command runs where typed input would — limit recorded).

## Reference (§20)

**Zed (the editor reference) — behavior only, via the fusion subsystem doc.** The
behavior adopted is Zed's runnables pipeline up to the spawn boundary:
`runnables.scm` per language (a `@run` capture + `#set! tag` — rust: `#[test]` fns,
`fn main`) → `runnable_ranges` → cached-per-buffer-version gutter play buttons
(`render_run_indicator`) → click runs the mapped task. Cited research:
`docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md` §1 (crate map +
provenance table), §2 (Zed has NO command blocks — the tab-sink degradation), §4 (the
query → gutter → spawn pipeline, "the good part"), TL;DR ("reimplement the discovery,
replace the sink with a Block spawn"). **The Marley deviation IS the ticket:** where
Zed spawns a terminal TAB carrying `Option<TaskState>` (status = a tab icon, result = a
plain text summary line), Marley spawns into the shipped block-terminal — the command
arrives as typed input, shell integration frames it, and the Block carries the status
pill/exit/output natively. No tasks panel, no task templates in v1. Clean-room
statement: no Zed source read or translated; Zed's `runnables.scm` FILES are GPL and
are NOT adopted — the detection query/walk is Marley's own, spike-pinned against
tree-sitter-rust's public grammar (the query MECHANISM is tree-sitter, MIT). Warp: N/A
— Warp has no editor-runnables surface (blocks are the Warp-side art, already shipped).

### Prior art

1. **Behavior maps** — `docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md`:
   §4's pipeline paraphrase (rust runnables = `#[test]` fn + `fn main` patterns with
   tags; `$ZED_SYMBOL` becomes the test filter), §2's two facts that define the wedge
   (no blocks; process-table polling instead of shell hooks), §9's reimplement guidance
   (port the model, re-derive the code; sink = Block). Research, never source.
2. **Published** — the cargo book / libtest CLI semantics: `cargo test <filter>` is a
   SUBSTRING match against the full test path (`mod::path::fn`); `-- --exact` requires
   the FULL path, which `src` alone cannot supply — so v1 omits it (D6). `cargo run`
   needs `--bin` only for multi-bin packages. tree-sitter's query docs (anchors,
   predicates) — the walk-vs-query choice is Phase 2's, both public mechanism.
3. **Our permissive deps (the high-yield leg)** — **tree-sitter-rust 0.24.2** (already
   shipped): the crate exports ONLY `HIGHLIGHTS_QUERY` / `INJECTIONS_QUERY` /
   `TAGS_QUERY` — there is NO runnables query to adopt, and `tags.scm` (read in full)
   captures definitions but NO attributes, so `#[test]` detection needs OUR OWN query
   or walk. Spike-pinned on the real grammar (scratchpad probe, this plan): attributes
   are preceding SIBLING `attribute_item`s of `function_item` (stacked = a contiguous
   run); the attribute path node is `identifier`("test") / `scoped_identifier`
   ("tokio::test") / `identifier`("cfg")+arguments("(test)"); `fn main` parent kinds
   separate runnable (`source_file`) from impl-method (`declaration_list`) and nested
   (`block`); the mod chain falls out of `mod_item` ancestors. In-tree art owns the
   rest: the iterative `TreeCursor` walk (`fold.rs collect_fold_regions`, stack-safe),
   the tags-query adoption pattern (`symbols.rs`), the #292 idle-pane walk, the #175
   `write_command` injection, gpui gutter click zones (#305). No new dependency.

## React-first (parity)

UI-AFFECTING — a NEW visible editor affordance with no Marley counterpart yet: zone B
design space, designed React-first (the #418 inversion precedent — the POC is the
design source until the port lands). marley-web files (per the MARLEY-PARITY port
map): `components/EditorView.tsx` grows the gutter ▶ (the per-line runnable cell —
placement vs the line number, hover/idle treatment, Chad's minimal-chrome grammar) and
`components/TerminalView.tsx` receives the spawned block through its existing
`onExecuteCommand` seam; `App.tsx` wires runnable-line data + the click→spawn flow.
Build & visually verify in marley-web first (`pnpm --filter @workspace/marley-ide run
dev` → localhost:5173), screenshot + READ the PNG, then port 1:1. Validate captures
the React↔Marley parity pair (pixel-sampled).

## Locked-In Decisions

- **D1 — Pure, parse-only, caller-gated (the settled node-API doctrine).**
  `runnables_in` parses Rust unconditionally and cannot self-gate; the APP gates on
  `language_of == Rust` at the call site (copy the #340/#305 gate, never the ungated
  #329/#330). Total, §14: never panics, empty on degenerate input.
- **D2 — marley_syntax stays toolchain-free.** The node API returns rows + semantic
  identity (`Test{path}` / `Main`); the `cargo …` command string is minted in a
  separate pure app-side seam (the block_status.rs altitude). Grammar knowledge and
  cargo knowledge never share a crate.
- **D3 — The spawn IS typed input.** The click routes through the existing submit
  tail: `history.record` + `session.write_command(&cmd)` + pump (the #175/#292
  precedent) — shell integration frames the Block exactly as if the user typed it. No
  new process spawn anywhere; PTY/child-process stays in the adapter crates (§14).
- **D4 — Deterministic target, fail closed.** Target pane = the #292 walk: idle
  terminal panes (`!is_command_running()` — the #40 guard; a busy PTY is NEVER written
  mid-command) in the active project's terminal grid, lowest `PaneId`. Phase 2 may add
  a cwd-aware preference (the tracked session cwd inside the project root) but must
  stay deterministic. No terminal tab (#391/#392 allows it) or no idle pane →
  flash-and-bail (PR-claude-429-a: the gesture verifies its own preconditions), zero
  side effects — v1 does NOT auto-spawn a terminal.
- **D5 — ▶ reflects the FILE; the spawn reveals the terminal.** The affordance renders
  whenever the Rust file has runnables, terminal or not (the click, not the render,
  checks D4). A successful spawn activates the workspace terminal tab so the live
  status pill is seen (Zed's reveal-always behavior, mapped to tabs, fusion doc §5).
- **D6 — Honest v1 command semantics, limits recorded.** `cargo test
  <in-file-mod-chain::name>` — substring filter, NO `--exact` (the crate-internal
  prefix is underivable from `src`; `--exact` would silently match nothing — worse
  than the over-match, which is recorded: same-suffix tests elsewhere also run, and a
  workspace-root cwd runs the filter across members). `cargo run` plain (no
  `-p`/`--bin`). The command runs in the target pane's CURRENT cwd — typed-input
  parity; a cd'd-away pane is the user's context, recorded limit.
- **D7 — The ▶ is its own click target.** A fn row is BOTH foldable (#305) and
  runnable; the ▶ element owns its hitbox (stops before the row handler), the fold
  gutter-zone click and the caret click keep their behavior on the same row. Exact
  zone split is the POC's call, pinned at Phase 2.

## Acceptance Criteria (EARS)

| # | EARS requirement (shall) | Verify |
|---|---|---|
| REQ-001 | WHEN `runnables_in` parses a file containing `#[test]` / stacked-attribute / `#[tokio::test]`-style fns under nested mods, the system shall return one `Test` runnable per test fn, in document order, on the fn's 0-based header row, with `path` = the full in-file mod chain + fn name (`outer::tests::case`); `#[cfg(test)]`-only fns, `#[rstest]`-style foreign harnesses, and fns nested inside another fn's body shall NOT be marked. | unit table (the spike fixtures verbatim) |
| REQ-002 | WHEN the file has a top-level `fn main`, the system shall return one `Main` runnable on its row; an impl-method `main` and any non-top-level `fn` shall not be `Main`; empty/unparsable input shall yield an empty Vec, never a panic. | unit |
| REQ-003 | WHEN a Rust editor file with runnables is open, the gutter shall render ▶ on exactly the runnable rows (first display segment only), coexisting with the git lane / diagnostic tint / fold click on the same row; WHEN the active file is non-Rust, no ▶ shall render and `runnables_in` shall not be invoked (the caller gate). | headless render drive + unit on the row-set fn |
| REQ-004 | WHEN the user clicks a ▶ and an idle workspace terminal pane exists, the system shall write exactly ONE command — the D6-mapped `cargo test <path>` / `cargo run` — through the typed-input path into the D4-resolved pane (recorded in history, framed as a new Block with a live status pill) and reveal the terminal tab. | headless drive (assert the new Block's command line + count) |
| REQ-005 | WHEN the user clicks a ▶ and no workspace terminal exists OR every pane is running a command, the system shall flash-and-bail with zero side effects — no bytes written to any PTY, no state change beyond the flash. | unit on target resolution + headless drive |
| REQ-006 | WHEN a ▶ row's fold gutter zone is clicked (not the ▶), the fold shall toggle exactly as today, and the ▶ click shall never toggle a fold nor move the caret. | headless drive |
| REQ-007 | WHILE the gate runs, the new pure seams (`runnables_in` + helpers, the command mint, target resolution, the gutter row-set fn) shall hold cov 100 / MSI 100 with no suppressions (new files `git add -N`-staged before the diff gate — PR-claude-intent-add-new-files-before-diff-mutation-001). | `scripts/gates.sh --diff` |

## Phase Plan

- **P2 Design** — walk vs query for the detection (the spike's node facts decide;
  either way pin fixtures name-by-name), the `Runnable` type + row semantics, the app
  memo shape ((nonce, version)-keyed like the syntax caches), the gutter cell design
  from the approved POC (+ the D7 hitbox split), the D4 resolution fn's exact
  signature, the reveal verb, file manifest + test plan + risks.
- **P3 Implement** — POC FIRST (EditorView gutter ▶ + the spawn flow into
  TerminalView, screenshot-verified), then Rust per manifest: syntax crate API → app
  seams → render/click wiring; `cargo check --workspace --tests` per cluster.
- **P3.5 Inspect** — independent critics vs the diff; provenance check (no Zed source,
  no GPL query text); the D4 busy-pane guard and D7 hitbox probed adversarially;
  PR-claude-verify-every-specified-trigger-is-actually-wired-001 sweep.
- **P4 Validate** — write + RUN the planned tests; live drive (real click → real Block
  in the running app); the React↔Marley parity pair; `scripts/gates.sh --diff` green.
- **P5 Complete** — docs (§21: editor.md gains the runnables section; the fusion wedge
  story starts), parity-map sync, ledger capture (§19), close the ticket, archive.
