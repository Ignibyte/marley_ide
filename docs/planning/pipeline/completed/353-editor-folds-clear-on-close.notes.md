# editor_folds cleared on last-view close (#305 W-2 follow-up) — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-353-editor-folds-not-cleared.md
- **Pipeline spec:** 353-editor-folds-clear-on-close.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** TICKET-353 (top of the backlog Queue) — `editor_folds` is never
  cleared on file/tab close; #305 inspect W-2 parked this (bounded anchor leak +
  stale-fold-on-reopen edge), #352 re-parked it. Fix: clear on close / path-key
  change.
- **Classification / tier:** bug, work pipeline, one shippable slice (the scrub
  + tests). Systems: editor state in `marley_app` only.
- **Recall (§18.3):**
  - #305 notes W-2 (:320) — the origin; both critics flagged; parked as
    not-a-Slice-1-blocker; mitigation = the F1 re-derive (drops anchors off live
    region headers — incomplete by design).
  - #319 — path identity canonicalized; `editor_folds` keys the stored spelling
    (notes :160); inspect finding 6 ACCEPTED (LOW): alias-root workspaces share
    a fold entry, documented at the map decl. The fix must consciously handle
    that stance (spec D3).
  - #352 spec (:93) — explicitly out of scope there; nothing else touched it.
  - Ledger greps: no fold-close prevention rule or failure; clean seam.
- **Discovery (the edit surface for Design):**
  - Decl app.rs:284 (`HashMap<PathBuf, Vec<Anchor>>`), init :2587, fold verbs
    :13074–:13337 — verbs guard dup pushes; growth is only ever per-path.
  - **The choke:** every editor-instance drop funnels through
    `crate::content::release_editor_views` (content.rs:261) →
    `ContentRegistry::release_view -> Option<C>` (content_registry.rs:87) —
    `Some` = last view dropped, instance freed. Call sites: app.rs :5646, :5682
    (viewer speculative — instance survives, row holds it), :8491 (tab close),
    :8583 (file-strip close #237), :8674 (project close), :9910 + :20077 (pane
    close), headless_drive.rs:1111.
  - **The mirror:** `release_grid_terminals` (app.rs:5852, #398) scrubs the
    terminal content-keyed maps (`agents`/`remotes`/`notify_ticks`/`last_agent`)
    inside the `Some(content)` arm — the exact pattern to adopt for editors.
  - **Rename is a no-op today:** LSP resource ops (Create/Rename/DeleteFile)
    rejected outright (#322 D2, `marley_lsp/src/workspace_edit.rs:49`); no
    file-tree rename surface. Deferred with a P5 ledger note.
  - Twin views are REAL (#396/#397; cross-project twins via shared roots,
    #359) — clearing on *tab* close would kill a live twin's folds; hence D1
    (instance drop, never view release).
- **Prior-art sweep (§20, three legs):** Zed behavior map — FoldMap is
  view-scoped display-map state, dies with the view (03-editor-multibuffer.md);
  published — LSP foldingRange has no lifecycle semantics, viewState-restore
  declined at #305; deps — no external owner, but OUR substrate owns the hook
  (`release_view -> Option<C>` + the #398 scrub pattern). Adoption, not
  invention.
- **Decisions:** D1 clear-at-instance-drop; D2 no persistence (re-affirm #305);
  D3 alias-root fork → design (simple clear vs census guard); D4 wiring shape →
  design (App wrapper mirroring `release_grid_terminals` vs free-fn returning
  dropped paths). EARS REQ-001..003 pinned; REQ-004 iff D3 → census guard.

## Phase 2 — Design

### Architecture / approach
The scrub rides the one instance-drop choke, mirroring #398's terminal-side
`release_grid_terminals` (app.rs:5852) exactly:

1. **Pure seam (content.rs):** `release_editor_views(reg, ids)` changes from
   `()` to `-> Vec<PathBuf>` — the STORED paths (`EditorInstance::path()`,
   canonical post-#319) of instances whose LAST view released
   (`release_view -> Some`). Instances still drop inline (the #397 posture —
   a Buffer free is plain memory work; the `#[must_use]` stays answered here).
   Non-editor content contributes no path (defensive; callers only pass editor
   ids).
2. **Pure census helper (content.rs, next to `find_open`):**
   `any_open_editor_with_path(reg, path) -> bool` — one `reg.iter()` scan,
   exact `==` on stored spellings. NOT `same_file`: both sides are canonical
   post-#319 and `==` is the `editor_folds` key semantics itself (remove uses
   the same key); syscall-free.
3. **App wrapper (app.rs, sibling of `release_grid_terminals`):**
   `fn release_editor_views(&mut self, ids: impl IntoIterator<Item=ContentId>)`
   — calls the pure fn; for each returned path, `editor_folds.remove(&path)`
   UNLESS `any_open_editor_with_path` finds a surviving same-path instance
   (the D3 census guard). Census runs after the whole release batch, so two
   alias instances dropping in one gesture clear correctly.
4. **All call sites migrate to the method** (list in the spec D4). The decl
   comment at app.rs:284 updates: lifecycle (cleared on last same-path
   instance drop) + the #319 finding-6 note evolves (alias sharing now exactly
   handled — the survivor keeps the entry by census, the last drop clears it).
5. New test accessor `editor_folds_len_for_test()` (the map has no
   active-path-independent probe; REQ-001 asserts the ENTRY died, not just the
   active view's count).

§14: no new IO, no unwrap on fallible paths (remove/no-op semantics
throughout), no new shared types, no spawns. §20 CONFIRMED: matches the Zed
behavior map's view-scoped fold model (03-editor-multibuffer.md — FoldMap dies
with the view; reopen unfolded); mechanism adopted from OUR substrate
(registry `iter`/`release_view` + the #398 scrub pattern); no copyleft read.
React-first stays N/A — no UI delta (state hygiene; the only visible change is
the bug's stale re-fold disappearing).

### File manifest
- `crates/marley_app/src/content.rs` — `release_editor_views` returns
  `Vec<PathBuf>` of fully-dropped editor paths; add
  `any_open_editor_with_path`; extend the seam test
  (`release_editor_views_drops_on_exactly_last` asserts `[]` on non-last,
  `[path]` on last) + census truth-table test.
- `crates/marley_app/src/app.rs` — App method `release_editor_views` (census
  scrub); migrate sites :5646 :5682 :8491 :8583 :8674 :9910 :20077; update the
  :284 decl comment; add `editor_folds_len_for_test`.
- `crates/marley_app/src/headless_drive.rs` — migrate :1111; add the four
  drives below.

### Regression test plan (≥1 row per REQ)
| REQ | Test | Kills |
|---|---|---|
| REQ-001a | headless: fold `f.rs` → close its file ROW (surface close via the wrapper, the :1105 pattern) → `editor_folds_len_for_test() == 0` | the missing-scrub mutant (delete `remove`) |
| REQ-001b | headless: fold → `cmd-w` (real key ladder, tab close) → map len 0 | the tab-close path not funnelling |
| REQ-002 | headless: fold in the tab → split a twin VIEW of the same file (same instance, 2 views) → close one view → fold count AND map len unchanged | scrub-on-any-release (scrubbing in the `None` arm) |
| REQ-003 | headless: fold → close last view → reopen the same path → `fold_count_for_test() == 0` and visible == full | wrong-key removal; stale-anchor resurrection |
| REQ-004 | content.rs: census truth table (same path + different root survivor → `true`; none → `false`). headless: TWO alias-root projects (`/var/folders/…` vs `/private`-prefixed — macOS tmp is itself an alias), same file open in both (two instances), fold → close one instance → survivor's fold count unchanged, map len 1 | guard→always-clear (`if true`) |
| seam | content.rs: release returns `[]` while views remain, `[path]` exactly on last | boundary mutants in the return path |

Genuinely uncoverable: none — all rows live in the headless lane / pure units.
Mutation notes: guard→`false` (never clear) dies by REQ-001a/b; guard→`true`
(always clear) dies by REQ-004; `remove`-deletion dies by REQ-001; census `==`
→ `!=` dies by REQ-004 truth table.

### Risks / load-bearing notes
- **Census AFTER the batch** — the wrapper iterates the pure fn's returned
  paths only when the whole release loop finished; two same-path instances
  dropping in one gesture → double `remove`, second a no-op. Correct.
- **Same instance twice in one batch** (a tab close collecting a surface row +
  a grid pane view of one file): first release `None`, second `Some` — the
  funnel handles it; no special case.
- **The REQ-004 two-project seed** hand-rolls a two-line shell blob (the #163
  codec via `persist_shell`); if the codec fights it, fall back to driving the
  second project through the runtime open path — validate confirms.
- Candidate P5 lesson: a fix must not regress the exotic case the BUG
  accidentally got right (the alias-twin survivor) — the census guard exists
  for exactly that.

## Phase 3 — Implement
- **React-first: N/A** (per the spec — no UI delta; state hygiene only).
- **Built exactly to the manifest:**
  - `content.rs` — `release_editor_views` now returns `Vec<PathBuf>` (stored
    canonical paths of fully-dropped editor instances; owned content still
    frees inline at scope end); new pure `any_open_editor_with_path` census
    (one `reg.iter()` scan, exact `==` on the stored spelling) beside the open
    seams.
  - `app.rs` — `pub(crate) fn release_editor_views(&mut self, ids)` sibling of
    `release_grid_terminals`: pure release → census-guarded
    `editor_folds.remove`. All seven call sites migrated (:5646 :5682 :8491
    :8583 :8674 :9910 :20077 pre-edit numbering); the `editor_folds` decl
    comment now documents the #353 lifecycle + the census-guarded alias-root
    stance (supersedes the #319 finding-6 "accepted" note).
  - `headless_drive.rs` — the :1111 inline release migrated onto the wrapper.
- **Straggler grep (the PR-137 discipline):** `content::release_editor_views`
  has exactly ONE remaining production caller — the wrapper itself. Clean.
- **Deviation:** `editor_folds_len_for_test` (manifest item) is DEFERRED to
  Phase 4 — added at implement it is dead code (`-D warnings` red) until the
  drives that read it exist; validate adds accessor + tests atomically.
- `cargo check --workspace --all-targets` green; `cargo fmt` applied (the
  census helper folded to one line).

## Phase 3.5 — Inspect
Two independent critics (correctness/state-integrity; simplification/reuse +
provenance), each instructed to verify concretely. Lead re-probed the key
invariant independently before spawning (clean).

| # | Sev | Finding (critic) | Verdict | Fix |
|---|---|---|---|---|
| 1 | MED | `git_marks` one field down carries the IDENTICAL close-leak class, unscubbed (correctness) | REAL, pre-existing, out of slice | TICKET-412 minted + backlog row; F-claude-git-marks-close-leak-sibling-of-editor-folds-001; PR-claude-lifecycle-scrub-sweeps-sibling-per-path-maps-001 |
| 2 | MED | Zero tests added for the new lifecycle claims (correctness) | BY PHASE DESIGN | Phase 4 owns write+RUN; the Phase-2 plan already tables every claim the critic lists (REQ-001..004 + seam + batch). No mutants::skip added — the method must earn MSI via the drives. |
| 3 | LOW→fixed | TOCTOU birth can store a join-spelled instance; hit-arm rows minted from fresh spelling diverge → scrub misses the row-keyed entry (correctness) | REAL (narrow race) | Hit arms (viewer + split) now mint rows from the instance's STORED path — row == instance by construction (the `add_content_to_split` idiom). F-claude-hit-arm-row-mint-fresh-spelling-diverges-from-instance-001 |
| 4 | MED | Free fn's `Vec<PathBuf>` return silently discardable — a future direct call skips the scrub (both critics converged) | REAL | `#[must_use = "…route production releases through RootView::release_editor_views"]` on the free fn; seam test consumes/asserts the returns |
| 5 | LOW | Method doc "never the free fn directly" overbroad — the seam's own test calls it legitimately (simplify) | REAL (wording) | Doc now says "Every PRODUCTION … (the seam's own tests are the one legitimate direct caller)" |
| 6 | LOW | Open TICKET-353 file still describes the leak as current (simplify) | PROCESS-OWNED | No diff change; P5 closes/archives the ticket per §19 |

Clean probes (evidence in critic transcripts): key-spelling invariant at every
mount family (outside the fixed race); funnel completeness (no editor drop
bypasses the choke — terminal/cockpit/browser lanes kind-guarded); speculative
releases can never be last-view; batch semantics incl. same-id-twice and
same-path-twice; census `==` vs #319 birth coverage; sibling per-content maps
(LSP docs reconcile didClose; caches nonce-keyed; nav_stack bounded by design);
provenance §20 (structure = in-repo #398 pattern + registry seams; Zed matched
at behavior level only; Warp has no editor-folding surface).

Post-fix verification: `cargo fmt` + `cargo check --workspace --all-targets`
clean; the extended seam test RUN and green (1/1,
`release_editor_views_drops_on_exactly_last`).

## Phase 4 — Validate
- **Tests written (one per plan row) and RUN:**
  - `content.rs::census_sees_alias_root_survivor_until_the_last_drop` — REQ-004
    pure half: alias-root truth table + release reports each dropped path once
    (KEEP arm → CLEAR arm).
  - `headless_drive.rs::fold_entry_dies_when_its_file_row_closes_headless` —
    REQ-001 row arm (`CloseOutcome::Removed` → wrapper), second file survives.
  - `…fold_entry_dies_when_the_editor_tab_closes_headless` — REQ-001 tab arm
    (`close_tab_at` funnel).
  - `…fold_entry_survives_a_twin_view_close_headless` — REQ-002: split-pane twin
    (one instance, two views), tab close retains the entry AND the focused twin
    still resolves the fold.
  - `…reopen_after_close_presents_unfolded_headless` — REQ-003: visible 5→(close,
    reopen)→8, fold_count 0, no entry.
  - `…fold_entry_survives_alias_root_twin_instance_close_headless` — REQ-004
    drive: two projects over one dir's two spellings (macOS /var↔/private/var),
    two instances of one canonical path; B's drop keeps the entry (census), A's
    final drop clears. The `assert_ne!(root_a, root_b)` makes the TMPDIR-symlink
    assumption loud, never a silent pass.
  - Re-added `editor_folds_len_for_test` (the Phase-3 deferral) with its drives.
  - Seam test extended at inspect (`release_editor_views_drops_on_exactly_last`
    asserts `[]` non-last / `[path]` last / `[]` double-release).
- **Targeted run: 6/6 PASS. Full suite: `cargo nextest run --workspace` →
  2065 run, 2065 passed, 5 skipped. `cargo test --workspace --doc` → ok.**
- **Live-app drive: N/A** — no `visual_acceptance` clause and no look/affordance
  delta (React-first N/A): the change is close-path state hygiene; its one
  user-observable behavior (reopen presents unfolded) is exactly what the
  headless drives assert against the real RootView. Stated per the
  never-silently-skip rule.
- **Pre-existing:** none encountered (suite fully green).
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]— 15 passed, 0
  failed** (rustfmt, clippy `-D warnings`, tests, coverage ≥100% lines,
  mutation MSI ≥100% on the diff, miri, audit, deny, machete, gitleaks,
  shellcheck, no-suppressions, source-bans, docs, visual/AX). Receipt
  `.git/ignibyte-gate-receipt` written (the §15 commit hook's evidence).

## Phase 5 — Complete
- **§21 docs:** CHANGELOG.md Unreleased→Fixed entry (the lifecycle fix, census
  guard, TOCTOU row-mint, TICKET-412 pointer); `docs/marley_architecture/editor.md`
  gains the "#353 closes the lifecycle" bullet in the folding section;
  `docs/marley_architecture/pane-composition-model.md`'s #398 lifecycle-riders
  line notes the editor-side extension. Parity sync N/A (non-React ticket).
- **Ledger appends this pipeline (codes):**
  - inspect: F-claude-git-marks-close-leak-sibling-of-editor-folds-001,
    F-claude-hit-arm-row-mint-fresh-spelling-diverges-from-instance-001,
    PR-claude-lifecycle-scrub-sweeps-sibling-per-path-maps-001
  - complete: L-claude-fix-must-not-regress-what-the-bug-got-right-001,
    AD-claude-editor-fold-state-in-session-instance-scoped-001
- **Ticket:** TICKET-353 closed → `tickets/closed/`; TICKET-412 (git_marks
  sibling leak) minted at inspect stays queued in BACKLOG.
- **Archive:** this pair → `docs/planning/pipeline/completed/`.
