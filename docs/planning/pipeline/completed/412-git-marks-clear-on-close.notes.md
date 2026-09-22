# git_marks + git_marks_key cleared on last-instance close (the #353 sibling) — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-412-git-marks-not-cleared-on-close.md
- **Pipeline spec:** 412-git-marks-clear-on-close.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** TICKET-412 (top of the backlog Queue) — `git_marks` is never
  cleared on file close; found by the #353 inspect correctness critic as the
  identical leak class one field down from `editor_folds`; deferred out of #353
  as its own shippable slice (the scrub lines need git-marks drive fixtures to
  clear the mutation bar).
- **Classification / tier:** bug, work pipeline, one shippable slice (two scrub
  lines + drives). Systems: `marley_app` editor/app state only.
- **Recall (§18.3):**
  - #353 completed pair — the direct template. As-built: pure
    `content::release_editor_views` returns the dropped paths; the App wrapper
    (app.rs:5907) census-guard-scrubs `editor_folds` via
    `any_open_editor_with_path` (content.rs:287, exact `==` on stored
    spellings, post-batch). Finding 1 of its inspect ledger IS this ticket.
  - #328 completed notes — the `git_marks` lifecycle: store + key + gen decls
    (app.rs:320–331), `refresh_git_marks` (:4089, `mutants::skip` shim,
    cache-gated), pump call (:2025), gen bumps on save/reload/commit ONLY.
    Test fixtures exist: `push_git_diff_for_test` (:4203),
    `git_marks_for_test` (:4208), `refresh_git_marks_for_test` (:4219), the
    seeded-real-git headless drive — the deferral reason is already paid for.
  - Ledger: F-claude-git-marks-close-leak-sibling-of-editor-folds-001 (open,
    points here — P5 flips its status);
    PR-claude-lifecycle-scrub-sweeps-sibling-per-path-maps-001 (the sweep this
    ticket must re-run at inspect).
- **Discovery (the edit surface for Design):**
  - The scrub arm: app.rs:5907–5913 — add `git_marks.remove(&path)` + the
    conditional `git_marks_key = None` inside the census arm.
  - **Load-bearing find: NO activation gen-bump exists.** The three production
    `git_marks_gen` bumps are in-app commit (:5053), external reload (:8987),
    save (:9261). Reopen self-heal rides the key's PATH half moving on a file
    switch — so close-active-A → reopen-A with no intervening bump matches the
    stale key at :4097 and early-returns. A map-only scrub would render a dirty
    file's gutter UNMARKED (a wrong-render the leak never had). The key-clear
    (ticket's "never dangles") is therefore REQUIRED-for-correctness, not
    hygiene — sharpened into the spec title block + D3 + REQ-004.
  - Sibling per-path-map sweep (per the prevention rule): `lsp_hosts` (:208,
    workspace-root-keyed, reconciled by its own LSP lifecycle — dispositioned
    at #353 inspect); `selected_key` (:933) is problems-panel-modal state,
    dies with `open_problems` → None; :12707+/13516+ are locals. No new
    per-file leak class since the 2026-08-10 sweep. Inspect re-runs on the
    final diff.
- **Prior-art sweep (§20, three legs):** behavior maps — Zed deconstruction 06
  §2 decoupled-git-store (adopted at #328) + the #353 FoldMap lifecycle stance;
  published — git's own diff/status semantics, no client lifecycle to adopt;
  deps/substrate — OUR substrate owns the whole seam (#353 arm + census, #328
  fixtures). Adoption, not invention; no external owner.
- **Decisions:** D1 scrub-in-the-#353-arm; D2 census identical to #353 (ticket
  pins); D3 key clears with the entry inside the census arm (load-bearing per
  discovery); D4 → design (key verification shape; accessor lands at validate
  atomically); D5 → design (push-seeded rows vs real-git end-to-end row mix).
  EARS REQ-001..004 pinned.

## Phase 2 — Design

### Architecture / approach
Two lines ride the existing #353 census arm (`RootView::release_editor_views`,
app.rs:5907) — no new choke, census, or call-site changes:

```rust
for path in crate::content::release_editor_views(&mut self.content, ids) {
    if !crate::content::any_open_editor_with_path(&self.content, &path) {
        self.editor_folds.remove(&path);
        self.git_marks.remove(&path);                                   // NEW
        if self.git_marks_key.as_ref().is_some_and(|(p, _)| p == &path) // NEW
        { self.git_marks_key = None; }                                  // NEW
    }
}
```

- **D3 shape confirmed:** the key compare is the PATH HALF ONLY (`p == &path`,
  ignoring the gen) — any key naming a fully-dropped path is stale by
  definition. Inside the census arm: a surviving same-path instance keeps
  entry + key as a consistent live pair.
- **D4 RESOLVED — the accessor.** `#[cfg(test)] git_marks_key_for_test(&self)
  -> Option<(PathBuf, u64)>` in the #328 hook region (placed after
  `refresh_git_marks_for_test`, honoring the attribute-placement BF). It is
  the only mutation-sound kill for the key-condition mutants: no pump-tick
  hook exists headlessly (checked — the only git refresh hook is the
  gen-BUMPING `refresh_git_marks_for_test`, which defeats a behavioral
  no-bump early-return repro), and `refresh_git_marks` is module-private to
  app.rs, unreachable from headless_drive.rs. Accessor + drives land at
  VALIDATE atomically (the #353 dead-code lesson).
- **D5 RESOLVED — push-seeded rows + one real-git row.**
  `push_git_diff_for_test` seeds entries without spawning git (per-path
  asserts via `git_marks_for_test` — seeding TWO files proves
  exactly-the-closed-path scrub with no new len accessor); the #328
  seeded-real-git pattern carries one end-to-end close→reopen row.
- **Key-state determinism (fixture invariant):** the headless lane never runs
  the frame pump's `refresh_git_marks` (evidence: the #328 push drives hold
  pushed state across `run_until_parked`) — key state moves only via
  `refresh_git_marks_for_test` or the scrub. Drives call refresh BEFORE push
  (a refresh with an empty/non-repo diff removes a pushed entry and sets the
  key — order is load-bearing).
- §14: no new IO, no unwrap on fallible paths (remove/None semantics), no new
  shared types, no new spawns (the real-git drive reuses the #328
  adapter-confined pattern). **§20 CONFIRMED:** matches the plan's Reference —
  Zed's buffer-scoped decoupled git state at behavior level; mechanism adopted
  wholly from our substrate (#353 arm + #328 fixtures); no copyleft read.
  React-first stays N/A (no UI delta beyond the bug's own edges disappearing).

### File manifest
- `crates/marley_app/src/app.rs` — (1) the two scrub lines in the census arm
  + the wrapper doc sentence (:5898 block gains the git-marks scrub); (2) decl
  comments: `git_marks` (:320) + `git_marks_key` (:325) gain the #412
  lifecycle sentences (entry dies with the last same-path instance,
  census-guarded; the key clears with the entry — never dangles); (3) AT
  VALIDATE: `#[cfg(test)] git_marks_key_for_test` after
  `refresh_git_marks_for_test` (:4222).
- `crates/marley_app/src/headless_drive.rs` — six drives (below), adapted from
  the #353 fold-lifecycle block (:8364) + the #328 git fixtures (:5251).
- No `content.rs` change — the pure release/census seams ship as-is (#353
  tests already pin them).

### Regression test plan (≥1 row per REQ)
| REQ | Test (headless_drive.rs) | Shape | Kills |
|---|---|---|---|
| REQ-001a | `git_marks_entry_dies_when_its_file_row_closes_headless` | f.rs+g.rs open; push marks for BOTH; close f's row (`CloseOutcome::Removed` → wrapper) → f empty, g intact | remove-deletion; wrapper-body Default; cross-path over-scrub |
| REQ-001b | `git_marks_entry_dies_when_the_editor_tab_closes_headless` | push marks; `close_tab_at` → entry empty | the tab-close funnel arm |
| REQ-002 | `git_marks_entry_survives_a_twin_view_close_headless` | push marks; `split_file_pane_for_test` twin; close editor tab → entry + rows unchanged | scrub-on-any-release (None arm) |
| REQ-003 | `git_marks_entry_survives_alias_root_twin_instance_close_headless` | `seed_two_projects` /var↔/private/var; SAME file, two instances; push marks under the CANONICAL spelling (stored-key semantics); close B → entry retained; close A → entry gone | census guard→always-clear; guard→never-clear (with 001) |
| REQ-004a | `git_marks_key_clears_with_its_entry_headless` | f+g open, g active; `refresh_git_marks_for_test` → key=(g,·); push f marks; close f's row → f entry gone, key STAYS (g,·) [negative]; then close g's row → key None [positive]; accessor asserts | is_some_and→true/→false; `p == &path`→`!=`; key-clear deletion |
| REQ-004b | `git_marks_close_then_reopen_recomputes_headless` | #328 seeded-real-git repo: commit, modify on disk, open, refresh → REAL marks; close tab → entry gone + key None; reopen + refresh → marks present | the end-to-end user journey (close→reopen shows fresh marks, not none/stale) |

Genuinely uncoverable: none — all rows live in the headless lane.
Mutation notes: the census guard's →true/→false already die by #353's fold
drives; REQ-003/001 re-kill them through the git_marks lens. The two new
lines + condition are fully covered by the matrix above.

### Risks / load-bearing notes
- **Refresh-before-push ordering** in drives (above) — inverted order silently
  empties the seeded entry and the test asserts vacuously.
- **The alias drive pushes under the canonical spelling** — production keys by
  the STORED (canonical, #319) path; pushing the /var spelling would mint a
  second entry the scrub never touches.
- **REQ-004a's close choreography** rides the #237 close clamp (closing f's
  row with g active keeps g active); activate explicitly if the clamp
  surprises.
- If the headless lane ever starts running the frame pump, the key-state
  determinism assumption breaks LOUDLY (asserts fail) — not silently.

## Phase 3 — Implement
- **React-first: N/A** (per the spec — no UI delta; close-path state hygiene).
- **Built exactly to the manifest (app.rs only, as designed):**
  - The census arm (`RootView::release_editor_views`) now scrubs
    `git_marks.remove(&path)` and clears a `git_marks_key` whose PATH half
    names the dropped path (`is_some_and(|(p, _)| p == &path)` → `None`),
    with the why-comment recording the no-activation-bump early-return hazard.
  - Wrapper doc comment extended (the #412 sentence; "survivor's folds" →
    "survivor's state").
  - Decl comments: `git_marks` gains the #412 lifecycle sentence
    (census-guarded death with the file); `git_marks_key` gains the
    cleared-with-the-entry sentence.
- **Deviations:** none. The `git_marks_key_for_test` accessor is
  validate-atomic per design D4 (dead code until its drives exist).
- `cargo fmt --all` applied; `cargo check --workspace --all-targets` green
  (4.9s; the `block v0.1.6` future-incompat note is pre-existing
  transitive-dep noise, untouched by this change).

## Phase 3.5 — Inspect
Two independent critics (correctness/state-integrity; simplification/reuse +
provenance + the sibling re-sweep), each instructed to verify concretely. Lead
cross-checked their claims against the Phase-1 discovery reads.

**Findings: none at any severity.** No fixes required. Ledger of clean probes
(each concretely verified in the critic transcripts):

- **Hazard analysis TRUE, fix closes it** — refresh traced line by line; gen
  writes exhaustively enumerated (init, test hook, commit :5058, reload :9001,
  save :9275 — none on activation/open). Bonus sharpening: the key-clear also
  fixes the closed-then-EXTERNALLY-modified-then-reopened case (no reload bump
  fires for a closed file) — recorded for the P5 CHANGELOG.
- **Funnel completeness** — `ContentRegistry` has no removal door besides
  `release_view`; every production editor release routes through the wrapper
  (sites re-enumerated post-diff); the two direct `release_view` callers are
  terminal-only lanes (PTY reap, `release_grid_terminals`).
- **Writer/reader census + invariant** — the only key mint is for the CURRENT
  active editor's path (a live instance by construction), so "a key never
  names an instance-less path" holds across every gesture sequence; pump and
  handlers share the main thread (no interleave).
- **Gen-bump interactions (#328-C2 recurrence probe)** — commit is synchronous
  blocking; a gen bump + the key-clear only ever converge on "next refresh
  recomputes".
- **Survivor-key retention** — census blocks the scrub; the retained key hits
  the early return over the RETAINED entry (marks are path-keyed saved-state,
  buffer-independent — identical for both alias twins). Correct render.
- **Batch semantics** — post-batch census (pure seam consumes the whole id
  batch); duplicate paths no-op; non-key paths leave the key alone.
- **Simplification** — no helper existed to reuse (first key-invalidation
  site); shape matches the in-repo `last_agent` clear precedent (:5889); the
  cheaper-looking `git_marks_gen`-bump alternative was checked and REJECTED
  (not semantics-preserving — it would invalidate the open active file's
  cache and respawn `git diff` on every unrelated close).
- **Doc truthfulness, gate-run** — `RUSTDOCFLAGS="-D warnings" cargo doc -p
  marley --no-deps` exit 0; the brand-scrub grep (`warp|zed` in crates/**.rs)
  zero hits; every factual claim in the added comments verified against code.
- **Provenance §20** — three lines shape-identical to the #353 arm + in-repo
  Option-clear precedent; nothing structurally foreign; no copyleft read.
- **Sibling re-sweep (the standing prevention rule)** — all `PathBuf`-keyed
  fields dispositioned: `lsp_hosts` (root-keyed, LSP-lifecycle reconciled),
  `editor_folds` (#353, same arm), `git_marks`/`git_marks_key` (this diff),
  `selected_key` (modal-scoped), `SearchReq.overrides` (NEW to the list —
  transient per-launch message, not a store), lsp_host.rs `docs` (reconciled
  by protocol `didClose` — correct layering; a bare map removal here would be
  WRONG). Nothing new leaks.
- **Tests absent from the diff** — BY PHASE DESIGN (the #353 inspect stance):
  Phase 4 owns write+RUN; the Phase-2 plan already tables every claim.

Post-inspect verification: no code changed after the critics ran; `cargo
check` remains green (critic 1 re-ran it clean).

## Phase 4 — Validate
- **Tests written (one per plan row) and RUN:**
  - `git_marks_entry_dies_when_its_file_row_closes_headless` — REQ-001 row arm
    (`CloseOutcome::Removed` → wrapper); the OTHER file's entry untouched
    (per-path asserts replace a len accessor).
  - `git_marks_entry_dies_when_the_editor_tab_closes_headless` — REQ-001 tab
    arm (`close_tab_at` funnel).
  - `git_marks_entry_survives_a_twin_view_close_headless` — REQ-002: split-pane
    twin (one instance, two views); tab close retains the entry unchanged.
  - `git_marks_entry_survives_alias_root_twin_instance_close_headless` —
    REQ-003: two projects over one dir's two spellings (/var↔/private/var);
    B's drop keeps the shared entry (census), A's final drop clears. The
    `assert_ne!` keeps the TMPDIR-symlink assumption loud.
  - `git_marks_key_clears_with_its_entry_headless` — REQ-004 both halves:
    dropping a NON-key path leaves the key alone (negative); dropping the
    key's own path clears it (positive). Kills the `is_some_and`→true/false
    and `==`→`!=` mutants.
  - `git_marks_close_then_reopen_recomputes_headless` — REQ-004 end to end,
    REAL git: dirty file's marks+key die together at close; the reopened
    still-dirty file recomputes `[(1, Modified)]` fresh.
  - `git_marks_key_for_test` accessor added (the D4 validate-atomic accessor).
  - All pushes/asserts use the canonical spelling (the design's stored-key
    risk note — `open_editor_file` returns the /var alias).
- **Targeted run: 9/9 PASS (6 new + the 3 #328 drives). Full suite:
  `cargo nextest run --workspace` → 2077 run, 2077 passed, 5 skipped.
  `cargo test --workspace --doc` → ok.**
- **Live-app drive: N/A** — no `visual_acceptance` clause and no
  look/affordance delta (React-first N/A): close-path state hygiene; the one
  user-observable behavior (reopen shows fresh marks) is exactly what the
  headless drives assert against the real RootView. Stated per the
  never-silently-skip rule.
- **Pre-existing:** none encountered (suite fully green; the `block v0.1.6`
  future-incompat note is pre-existing transitive-dep noise).
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] — 15 passed, 0
  failed** (rustfmt, clippy `-D warnings`, tests, coverage ≥100% lines,
  mutation MSI ≥100% on the diff, miri, audit, deny, machete, gitleaks,
  shellcheck, no-suppressions, source-bans, docs, visual/AX). Receipt
  `.git/ignibyte-gate-receipt` written (the §15 commit hook's evidence).

## Phase 5 — Complete
- **§21 docs:** CHANGELOG.md Unreleased→Fixed entry (the leak + the load-bearing
  key-clear story, drives, gate); `docs/marley_architecture/editor.md` — the
  git-gutter section gains the #412 lifecycle sentence and the #353 bullet's
  TICKET-412 pointer now records the closure; `pane-composition-model.md`'s
  lifecycle-riders line notes #412 riding the same arm. Parity sync N/A
  (non-React ticket).
- **Ledger appends this pipeline (codes):**
  - complete: PR-claude-per-path-scrub-sweeps-derived-key-twins-001,
    L-claude-plan-phase-producer-enumeration-sharpens-requirements-001,
    AD-claude-git-gutter-state-instance-scoped-001; the
    F-claude-git-marks-close-leak-sibling-of-editor-folds-001 status line
    flipped open → fixed (#412).
  - inspect: no new F-/PR- (both critics clean; the probes ledger lives in the
    Phase 3.5 entry above).
- **Ticket:** TICKET-412 closed → `tickets/closed/` (backlog row left at
  promotion; sweep found no stale row).
- **Archive:** this pair → `docs/planning/pipeline/completed/`.
