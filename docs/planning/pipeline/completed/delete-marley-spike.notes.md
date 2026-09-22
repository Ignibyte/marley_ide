# delete marley_spike — Notes

- **Forge ticket:** #27 `ef3f9837-2d8b-4009-b930-96839267d156`
- **AAR:** `69be8de9-21c6-4cb3-a9a3-7e2df3fa7d8d`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-027-delete-marley-spike.md
- **Pipeline spec:** delete-marley-spike.spec.md

## Phase 1 — Plan
- **Request:** forge #27 (M1.C seq-6, CHORE, auto-approved) — delete the M0 throwaway
  marley_spike + sweep every reference.
- **Classification / tier:** work pipeline, `chore` — a deletion. It REMOVES `crates/**/*.rs`, so
  despite being a chore it needs a FULL/`--diff` green receipt to commit (not the no-`.rs`
  exit-0 path). One shippable slice.
- **Forge recall (§18.3):** bulletins none. The spike is `PR-claude-write-first-not-research-...`'s
  exemplar (now superseded by marley_app — the lesson text is historical, left as-is). The
  gate-15 component-dir trap (`visual_g` reads `component:` then `[ -d crates/<comp> ]`) is the
  reason SPEC-foundation-spike must be deleted with the crate.
- **Discovery (footprint fully mapped in pre-flight — 15 files):**
  - `crates/marley_spike/` — the crate (src/{lib,app,term_io?,drive}.rs, bin/, tests/{integration,
    headed_one_block}.rs, Cargo.toml, the committed headed baseline). A LEAF — no crate depends
    on it (verify at inspect via the clean build).
  - Workspace: `members = ["crates/*"]` GLOB → deleting the dir auto-drops membership; NO root
    Cargo.toml edit (D2).
  - `scripts/gates.sh:196` rust_cov `--ignore-filename-regex` names
    `marley_spike/src/(app|term_io)\.rs|marley_spike/src/bin/` — REMOVE; `:177` comment — update.
  - `docs/specs/SPEC-foundation-spike.spec.md` — `component: marley_spike` + a non-N/A
    `visual_acceptance` (line 13) → a gate-15 binding; DELETE the spec (D1).
  - `docs/marley_architecture/marley_spike.md` — as-built doc, DELETE.
  - `docs/marley_architecture/crate-map.md:40` (mermaid `MSP` node) + `:121` (table row "to be
    deleted") — remove.
  - `crates/marley_app/tests/headed_shell.rs:4,44` + `crates/ui_components/tests/headed_widgets.rs:4`
    — COMMENT-only "marley_spike precedent" prose → reword (D3).
  - `terminal_blocks.md` / `app_shell.md` — lineage prose ("promoted from"/"the marley_spike gpui
    boot it promotes") → reword to not name a live crate.
  - `CHANGELOG.md` — historical spike entries stay; add a `### Removed` entry.
- **Decisions:** D1–D3 in the spec (delete the spec, glob = no manifest edit, reword precedent
  comments).
- **Open questions for Design:** the exact `visual_g` behavior on a component spec whose crate
  dir is missing (skip-clean vs fail — confirm the spec-deletion is REQUIRED not just tidy); the
  precise wording of the reworded comments/lineage; whether `term_io.rs` exists (the exclude names
  it — confirm the crate's actual file set at implement).
- **AAR id:** `69be8de9-21c6-4cb3-a9a3-7e2df3fa7d8d`.

## Phase 2 — Design

### Architecture / approach
A pure deletion + reference sweep — a "gate-is-test" change (no new `.rs` logic; verified by the
gate's exit codes + grep smokes, §7). It DOES remove `crates/**/*.rs`, so the commit gate requires
a FULL/`--diff` green receipt (not the no-`.rs` exit-0 path). `marley_spike` is a workspace LEAF
(no crate `[dependencies]` it) — the clean build post-delete is the proof nothing consumed it.

**`visual_g` behavior confirmed (corrects D1's framing):** `visual_g` iterates `SPEC-*.spec.md`,
and for each non-N/A `visual_acceptance` does `[ -n "$comp" ] && [ -d "crates/$comp" ] || continue`
— a component spec whose crate dir is MISSING is SILENTLY SKIPPED, not failed. So deleting the
crate alone would NOT break gate-15 — but it would leave `SPEC-foundation-spike.spec.md` as a
dangling spec-with-no-crate (precisely the silent-skip cruft the gate-15 component-dir trap
warns about). **Delete the spec too** — the reason is cruft-removal + the ticket's explicit ask,
not a gate failure. (D1 stands; the "gate would fail" wording is corrected here.)

**Crate file set (13 files, NO committed baseline):** `Cargo.toml`; `src/{lib,app,term_io,block,
config,drive,grid,read,shutdown}.rs`; `src/bin/marley_spike.rs`; `tests/{integration,
headed_one_block}.rs`. The headed test uses the AX + blank-detector (no committed PNG to remove).

### File manifest
- `git rm -r crates/marley_spike/` — the whole crate (the `members = ["crates/*"]` glob auto-drops
  membership; NO root Cargo.toml edit).
- `git rm docs/specs/SPEC-foundation-spike.spec.md` — the gate-15-bound spec.
- `git rm docs/marley_architecture/marley_spike.md` — the as-built doc.
- M `scripts/gates.sh` — drop `marley_spike/src/(app|term_io)\.rs|marley_spike/src/bin/` from the
  rust_cov `--ignore-filename-regex` (line ~196); update the exclude comment (line ~177) to no
  longer describe the spike.
- M `docs/marley_architecture/crate-map.md` — remove the `MSP["marley_spike · throwaway"]` mermaid
  node (line ~40) + its `| marley_spike | … |` table row (line ~121); drop any edge into `MSP`.
- M `docs/marley_architecture/terminal_blocks.md`, `docs/marley_architecture/app_shell.md` — reword
  the lineage prose ("promoted from marley_spike" / "the marley_spike gpui boot it promotes") to
  state the historical fact without naming a live crate.
- M `crates/marley_app/tests/headed_shell.rs` (lines 4, 44) + `crates/ui_components/tests/headed_widgets.rs`
  (line 4) — reword the "marley_spike precedent" COMMENTS (blank-detector-pattern lineage) to not
  name the deleted crate. (Comment-only; the `#[ignore]` headed tests still compile.)
- M `CHANGELOG.md` — a `### Removed` entry (§21); the historical spike entries stay (archival).

### Regression Test Plan
| REQ | Verification | Kind |
|---|---|---|
| REQ-001 | `grep -rn marley_spike crates/ scripts/ Cargo.toml docs/specs docs/marley_architecture` returns NOTHING (comments reworded, spec/doc/node removed) | grep smoke |
| REQ-002 | `cargo nextest run --workspace` green — the workspace builds + every surviving test passes (nothing depended on the leaf) | suite |
| REQ-003 | gate:15 PASS + `! test -f docs/specs/SPEC-foundation-spike.spec.md` (no dangling component binding) | gate + smoke |
| REQ-004 | `scripts/gates.sh --diff` GREEN [diff] + a written receipt (the deletion touches `crates/**/*.rs`) | gate |
| REQ-005 | review: crate-map has no `marley_spike` node/row; terminal_blocks/app_shell lineage reworded | review |

No unit tests to add (a deletion has no new logic surface — gate-is-test per §7). Uncoverable:
none new.

### Risks / decisions
- The `--diff` mutation over a pure deletion has nothing to mutate (deletions add no lines) — the
  receipt still writes from the full-workspace coverage pass; expect the FULL coverage denominator
  to SHRINK (the spike's pure layer leaves it). NOT a regression.
- Editing two `#[ignore]` headed test files (comment-only) keeps them compiling; they're not run
  in the per-commit gate, so no headed-lane dependency.
- No `deny.toml` reference to the spike (grep-confirmed) — nothing to sweep there.
- The forge `PR-claude-write-first-not-research-first-known-api-exemplar-001` lesson names the
  spike as its exemplar; it is HISTORICAL record (not edited) — marley_app is now the live
  exemplar, noted at complete.

## Phase 3 — Implement
- **Done (per manifest):** `git rm -r crates/marley_spike` (13 files) + `git rm`
  SPEC-foundation-spike.spec.md + marley_spike.md; gates.sh — dropped the
  `marley_spike/src/(app|term_io)\.rs|marley_spike/src/bin/` fragment from the rust_cov exclude +
  the exclude comment sentence; crate-map.md — removed the `MSP` mermaid node, its 2 `-.->` edges,
  and the table row (0 refs left); terminal_blocks.md + app_shell.md lineage reworded ("the M0
  viability spike whose patterns it promotes"); headed_shell.rs (×2) + headed_widgets.rs comments
  reworded to "the M0 spike precedent"; CHANGELOG — a `### Removed` entry appended under the
  existing [Unreleased] Removed section.
- **Deviations from design:** none. (Confirmed the crate had NO committed baseline PNG — nothing
  extra to remove; `term_io.rs` existed as the exclude named it.)
- **Verification at this phase:** `cargo fmt --check` clean; `cargo check --workspace` builds
  (nothing depended on the leaf); `grep -rn marley_spike crates/ scripts/ Cargo.toml docs/specs
  docs/marley_architecture` → **0** (REQ-001 smoke green). The nextest + gate --diff run is
  Phase 4.

## Phase 3.5 — Inspect
- **Critic run:** 1 completeness/sweep critic (a deletion's risk is a dangling ref or a hidden
  dep, not logic) over 8 lenses, all command-verified.
- **Findings:** NONE (0 HIGH/MED/LOW). Lenses covered + evidence:
  1. Exhaustive dangling-ref sweep (`git grep -ni marley_spike` over ALL tracked file types
     .rs/.toml/.md/.sh/.json/.lock/.png, excluding CHANGELOG + this pipeline's own docs) → **0
     hits**; no spike-named baseline PNG; no bin/target reference.
  2. No crate depended on the spike — `grep marley_spike crates/*/Cargo.toml` + root = 0;
     `cargo metadata --no-deps` → 10 packages, marley_spike ABSENT (it was a leaf).
  3. Cargo.lock = a PURE PRUNE (only the marley_spike `[[package]]` block removed; zero version
     pins changed on any other crate).
  4. gates.sh regex VALID — `bash -n` OK, shellcheck clean, 7 balanced alternatives, no dangling
     `|`/empty alternation, no leftover `marley_spike`; the removed fragment was strictly spike
     paths (no non-spike file lost coverage).
  5. gate-15 — `grep -l "component: marley_spike" docs/specs/` → 0 (the deleted spec was the only
     binding; 17 surviving component specs, none the spike).
  6. Both reworded headed tests COMPILE (`cargo check -p marley -p marley_ui_components --tests`
     → Finished; comment-only edits).
  7. crate-map mermaid WELL-FORMED (MSP node + 2 edges gone; both edge targets R_gpui/R_alac
     still referenced by other edges — no orphan/dangling).
  8. No other breakage surface — no `.github/`, CI yaml, justfile/Makefile, README, asset
     manifest, or `.gitignore` pattern references the spike.
- **Informational (not a finding):** stale gitignored `target/…marley_spike…` build artifacts —
  untracked, pruned on the next `cargo build`, zero gate/CI impact.
- **Post-verify:** the diff is a clean prune; the gate --diff run (Phase 4) is the receipt proof.

## Phase 4 — Validate
- **Gate-is-test (deletion): no unit tests to add** — verification = the smokes + the gate.
- **Smokes (REQ-001/003):** `grep -rn marley_spike crates/ scripts/ Cargo.toml docs/specs
  docs/marley_architecture` → **0**; `SPEC-foundation-spike.spec.md` gone.
- **Suite (REQ-002):** `cargo nextest run --workspace` → **462 passed, 5 skipped** (was 477 —
  exactly the spike's 15 tests gone, nothing else lost; the workspace builds → nothing depended
  on the leaf).
- **Gate (REQ-004):** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**; coverage 100% lines
  (10720 regions, `mutation: no mutable lines in the diff — pass` — exactly as a pure deletion
  predicts); receipt `3dbbaf6d5fe85f530ff14271491e7d28bac38cfd`.
- **Transient-flake note (NOT a defect, NOT in scope to "fix"):** the first two gate runs HUNG in
  the heavy gates on real-PTY tests (`workspace_two_real_sessions` under coverage, then
  `resize_real_pty_succeeds` under the cargo-mutants baseline). Diagnosed as environmental
  PTY-scheduling contention on a machine loaded by this long session (~8 gate runs), NOT a code
  issue: (a) each named test passes in ISOLATION in <1s; (b) the marley_terminal real-PTY suite
  passes in **0.54s** under the exact parallel one-process mode cargo-mutants uses; (c) no
  orphaned test shells existed (PPID-1 scan clean); (d) the SAME gate config passed at #22–#26;
  (e) the deletion removes a real-PTY test, it can't add a hang. After killing the accumulated
  runners + confirming the PTY tests fast, the 3rd run passed clean 15/15. Coverage/mutation are
  isolated-per-process under nextest (why they never flake there) but cargo-mutants' baseline
  shares one process — a known marginal interaction under load. Left as-is (a real-PTY-under-
  parallel-mutants-baseline robustness item, unrelated to this deletion).
- **Pre-existing failures:** none.

## Phase 5 — Complete
- **Docs (§21):** done at implement — CHANGELOG `### Removed` entry (under [Unreleased]);
  crate-map.md node/edges/row removed (0 refs); terminal_blocks.md + app_shell.md lineage
  reworded; `marley_spike.md` + `SPEC-foundation-spike.spec.md` deleted.
- **AAR capture:** `PR-claude-real-pty-flake-under-mutants-baseline-not-code-001` (the
  diagnose-the-gate-hang lesson — real-PTY tests flake under cargo-mutants' shared-process
  baseline on a loaded machine but not under nextest's per-process isolation; a pure-deletion
  diff proves it's infra via `mutation: no mutable lines in the diff — pass`); aar-submit
  `completed`. The forge `PR-claude-write-first-not-research-first-known-api-exemplar-001`
  lesson's exemplar is now marley_app (the spike text is historical, left as-is).
- **Ticket:** forge #27 → done; local doc → closed/; pipeline pair archived.
- **SPRINT COMPLETE:** this is the LAST ticket of M1.C "The Wired Cockpit" (sprint #3). All six
  (#22 AnsiCQuoted emit · #23 panes-for-real · #24 docks · #25 palette dispatch · #26 settings ·
  #27 delete-spike) shipped through the full pipeline at cov 100/MSI 100 + GREEN gate. Close the
  sprint after the commit.
