# Editor file identity — one canonical spelling for an open file — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-319-editor-file-identity-lsp-paths.md
- **Pipeline spec:** 319-editor-file-identity.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** `/work` next-item → TICKET-319 (bug, M20): editor file identity
  vs LSP canonical paths — a symlinked project root can open the same file twice.
  Session goal `/work on the next item` is active (autonomous-through-commit);
  plan/design confirmations ride on that standing directive.
- **Classification / tier:** bug; single work pipeline, one shippable slice
  (identity normalization + compare-site adoption + back-compat), no split needed.
- **Recall (§18.3):**
  - #322 (`BF-rename-open-file-misroutes-to-disk-under-symlinked-root-001` +
    prevention-rules.md:292): never raw `PathBuf ==` across a canonical/
    non-canonical seam; compare canonical with raw fallback — shipped as
    `same_file` (editor_surface.rs:28). This ticket extends the discipline from
    compare-time to STORAGE-time.
  - #310 F1 (`PR-claude-external-uri-key-normalize-both-sides-001`): the LSP
    layer keys by canonical PathBuf deliberately; both sides of a store must use
    the SAME normalization. Locked as spec D1.
  - #289 F1 (prevention-rules.md:1203): compare a fresh-resolved path against a
    stored one only via the identical root/normalization that produced the store.
  - #322 containment rule (prevention-rules.md:194): canonicalize BOTH sides of
    a `starts_with` containment check; `resolve_under_root` is NOT a containment
    primitive. Drives spec D6 (the app.rs:5430 LSP host-spawn gate re-check).
  - #205 (cwd persistence): the back-compat tier pattern for a changed stored
    spelling — reused for session/arrangement restore (spec D4).
- **Discovery:** Explore agent (a9c14af…) mapped the full seam surface; digest:
  - **#397 reframed the ticket**: `EditorSurface::open_view` dedupes by
    `ContentId`; the path test lives in `content::find_open` (content.rs:214) =
    raw `i.root() == root` AND canonical `same_file(i.path(), path)`. The raw
    root half is DELIBERATE (D-OPEN-DEDUPE-SCOPE, content.rs:203-213) — out of
    scope (spec D3).
  - **The live bug is mixed stored spelling**: tree/finder/restore store
    `resolve_under_root(project_root, rel)` (verbatim-root join —
    app.rs:5553/5602/5671, marley_project/src/lib.rs:75), LSP jumps store the
    server's canonical spelling (jump_to_definition app.rs:12516 →
    open_and_place_caret app.rs:13881). Stored identity depends on which
    producer won the race.
  - **User-visible breakage under a symlinked root:**
    1. F12 landing check `ed.active_file().path == path` (app.rs:13889) — raw
       `==` canonical-vs-joined → false → "Can't open the definition's file" on
       a tab that just switched; caret never places. Reached by F12/⇧F12/⌘T/
       search-jump/problems-jump/nav_back (app.rs:12682, 12864, 13628, 13806, 13907).
    2. `links.rs:260/279` — `resolve_under_root(root, path) == *open_path` raw
       compare → terminal-diagnostic gutter rows drop when the open file is
       canonical-spelled.
    3. `same_file` canonicalize-failure edge (file deleted mid-session) falls
       back to raw compare of two different spellings → second buffer for one
       file (#249 defeat).
  - **Storage seams to adopt the helper:** load_code_view_state app.rs:5553;
    open_file_in_viewer probe :5602; split_file_pane :5671; open_editor_instance
    :1449/:1465; session restore :2283 (`TabLayout::Code`) + :2148
    (`PaneKind::CodeView`); apply_arrangement key_ok :7731 + mount :7810;
    open_link_target :5359; jump_to_block_failure :7987; git-diff header click
    hand-join :20689-20691 (bypasses resolve_under_root today).
  - **Consistent-by-construction consumers** (keyed by the stored spelling —
    fixed transitively once storage is canonical): git_marks (app.rs:316/4128),
    editor_folds (app.rs:279/13033…), save_active fs::write (app.rs:9327),
    did_save → LspHost re-canonicalizes (lsp_host.rs:256/268). Problems/
    references identity compares (editor_problems.rs:48, editor_references.rs:133)
    compare uri-derived vs uri-derived — Design confirms whether mixed-spelling
    is reachable there.
  - **Persistence wire:** `V=` codec grid_layout.rs:280/303, split `c=` slot
    :66, serialize from `cv.path.display()` app.rs:5326-5331/5314/5254;
    arrangement key app.rs:7667, groups keyed by string equality
    arrangements.rs:128.
  - **LSP side (untouched):** LspHost::new canonicalizes root (lsp_host.rs:132);
    `absolute` = join+canonicalize (:268); uri_for (:513); pure
    file_uri/path_from_file_uri (marley_lsp handshake.rs:41/:64).
  - **Harness pointer to update:** headless_drive.rs:2875-2877 (#319 comment in
    `seed_definition_files`), fixture helpers :3628/:3855.
- **Prior-art sweep (§20):** recorded in the spec. Highlights: Zed worktree
  `canonical_path`/ProjectPath + Fs::canonicalize at the boundary; Warp
  `CanonicalizedPath` newtype canonicalized at construction; LSP 3.17 leaves
  identity normalization to the client; no shipped crate owns symlink resolution
  (gpui_util is lexical-only — verified in registry source), `std::fs::canonicalize`
  is the primitive; `same-file` crate considered/not-adopted (need a storable key).
- **Decisions:** spec D1–D6. Open for Design: helper home (marley_project vs
  marley_app), landing-check form (`==` vs `same_file` per D5), arrangement
  legacy-key mechanics, whether problems/references compares need adoption.

## Phase 2 — Design

### Architecture / approach
Two sibling helpers land in `marley_project` (the single owner of root-path
discipline, §14 — `resolve_under_root` already lives there; tempfile dev-dep +
symlink-test precedent at lib.rs:491):

```rust
/// #319: THE stored identity of an open editor file — resolve_under_root, then
/// canonicalize when the target exists (collapses root-symlink aliases:
/// /tmp→/private/tmp, symlinked $HOME); the resolved join when it does not
/// (vanished/new file — matching same_file's raw fallback so storage and
/// compare can never disagree).
pub fn canonical_under_root(root: &Path, path: &Path) -> PathBuf {
    let joined = resolve_under_root(root, path);
    joined.canonicalize().unwrap_or(joined)
}

/// #319: the inverse — a stored (canonical) path back to root-relative for git
/// pathspecs / tree reveal / display labels. Strip the verbatim root, else the
/// CANONICAL root, else the path unchanged (outside the project). All arms
/// borrow from `path` — zero-alloc except the fallback's canonicalize temp.
pub fn rel_under_root<'a>(root: &Path, path: &'a Path) -> &'a Path
```

**One rule**: every editor-open RESOLVE goes through `canonical_under_root`, so
the stored spelling (instance path, view-row `cv.path`, persisted `V=`/`c=`
strings, arrangement keys) is canonical-by-construction; `same_file` stays the
compare primitive wherever one side may not exist (find_open, has_path,
buffer_for_open_path_mut — unchanged).

Decisions made this phase (extends spec D1–D6):
- **D7 — landing check**: `open_and_place_caret` normalizes its INCOMING path
  once at entry (`canonical_under_root`), keeping the raw `==` landing check
  (both sides helper-produced → D5-compliant) and making the fn
  spelling-agnostic for ALL five jump callers (incl. a relative
  `jump_to_match` path).
- **D8 — host-spawn gate (app.rs:5428)**: post-fix open docs are canonical, so
  `resolve_under_root(...).starts_with(verbatim_root)` would be ALWAYS-FALSE
  under a symlinked root → no LSP at all (found at design; the ticket missed
  it). Fix per the #322 containment rule: compare the stored (canonical) doc
  path against a LAZILY canonicalized root (`Option::get_or_insert_with` inside
  the `any` closure, extension check first) — tick cost unchanged when no .rs
  doc is open; extract the predicate as a small pure fn so it is unit-testable.
- **D9 — encoding/host lookups (app.rs:12873 `jump_to_symbol`, :13813
  `jump_to_problem`)**: `row.path.starts_with(map_key_root)` matches a
  canonical path against the VERBATIM map key → falls back to Utf16 under a
  symlinked root (wrong caret col on a Utf32/Utf8 host). Fix: filter/rank by
  the HOST's own root (`LspHost` stores it canonicalized since lsp_host.rs:132;
  expose an accessor if absent). Zero extra syscalls.
- **D10 — links.rs stays PURE** (documented no-IO, mutation-tested, runs per
  output line): the fns are untouched; their one caller
  (`open_file_diagnostic_rows`, app.rs:10891) passes a CANONICALIZED root so
  relative compiler refs join to the canonical spelling. ACCEPTED MISS
  (documented + pinned by test): an ABSOLUTE ref printed in the verbatim
  spelling under a symlinked root no longer matches (today it matches only when
  the file was tree-opened — already flaky); relative refs — the dominant
  rustc/cargo form — work consistently. Non-symlinked roots: zero delta
  (canonical root == verbatim root).
- **D11 — rel derivation**: `strip_prefix(&project_root)` consumers of stored
  paths would fail post-fix under a symlinked root → adopt `rel_under_root` at:
  git-gutter pathspec (app.rs:4072 — keeps `git diff` on a rel pathspec instead
  of betting on git's abs-path realpathing), RevealInTree (:7938 — also fixes
  canonical-spelled terminal refs today), search-overlay + problems-panel
  display rels (:15256, :15384 — display-only degradation otherwise).
- **D12 — no-change sites (verified, with reasons)**: producers (tree/finder/
  palette/git-click hand-join/terminal refs) stay untouched — the open seam
  normalizes internally; `open_editor_instance` stays a pass-through (its two
  restore callers hand it helper-resolved paths; doc line added);
  problems/references identity compares (editor_problems.rs:48,
  editor_references.rs:133) are uri-derived vs uri-derived — consistent;
  `find_open` root half + `tabs.rs:673` project check per D3;
  `git_marks`/`editor_folds` HashMaps key on the stored spelling — consistent
  transitively; save/did_save re-canonicalize inside LspHost; the rename
  applier's containment arm (app.rs:11813) already canonicalizes both sides.
- **§20 confirm**: matches Zed's canonicalize-at-the-boundary behavior (worktree
  `canonical_path`) and Warp's canonicalize-at-construction (`CanonicalizedPath`)
  — behavior only, mechanism Marley's own; no copyleft source read. React-first
  stays N/A (no visible surface change).

### File manifest
| file | change |
|---|---|
| `crates/marley_project/src/lib.rs` | ADD `canonical_under_root` + `rel_under_root` (+ doctests) after `resolve_under_root`; unit tests (symlink alias → canonical; nonexistent → join; absolute passthrough; rel strip verbatim/canonical/outside). |
| `crates/marley_app/src/app.rs` | Resolver swaps to `canonical_under_root`: 5553 (`load_code_view_state`), 5602 (`open_file_in_viewer` probe), 5671 (`split_file_pane` probe), 2148 + 2283 (restore arms), 7731 + 7810 (`apply_arrangement`). D7 normalize-at-entry in `open_and_place_caret` (13881). D8 gate restructure (5425-5437) + extracted pure predicate. D9 host-root filters (12873, 13813). D10 canonical root at 10891. D11 `rel_under_root` at 4072, 7938, 15256, 15384. Comment updates where rationale shifts (#289 F1 note at 10888). |
| `crates/marley_app/src/editor_surface.rs` | Doc updates only: `same_file` (storage now canonical per #319; fn remains the one-side-missing compare), `EditorInstance.path` (canonical identity note). |
| `crates/marley_app/src/content.rs` | One-line doc pointer on `find_open` (path side now stored canonical; root half deliberately raw, unchanged). |
| `crates/marley_app/src/headless_drive.rs` | Update the #319 comments (2875-2877, 3628, 3855 — point at the fix); NEW drive tests (see test plan). |
| `crates/marley_app/src/links.rs` | Doc note on the canonical-root caller contract + the accepted abs-verbatim miss; no code change. |

### Regression test plan
| REQ | Test (all RUN at Phase 4) |
|---|---|
| REQ-003 | `marley_project` units: `canonical_under_root_collapses_symlinked_root` (explicit `std::os::unix::fs::symlink` alias — link-root + rel → canonical spelling); `..._nonexistent_falls_back_to_join`; `..._absolute_passes_through` (existing → canonical; missing → unchanged); `rel_under_root_{verbatim,canonical_root,outside}`; doctests on both fns. Kills the unwrap_or / strip-arm mutants. |
| REQ-001 | `marley_app` unit (content/editor_surface level, extending the 1487 symlink test): resolve_open stores canonical under an aliased root; find_open hits from BOTH spellings → one instance. PLUS headless drive: aliased-root project, tree-style relative open + dirty edit, then canonical-spelling open → 1 instance, edits intact, stored path == canonical. |
| REQ-002 | Headless drive (#312-style scripted definition): aliased root (TempDir /var alias or explicit symlink), origin opened via RELATIVE path, definition response names the CANONICAL target → `open_and_place_caret` lands (caret at target row), no "Can't open the definition's file" flash. Inherently exercises D8 (the host must pass the gate for the .rs doc under the aliased root) — plus a focused unit on the extracted D8 predicate. |
| REQ-004 | Restore test (drive boot context): legacy `V=` + `c=` blob with verbatim-joined absolute spellings under an aliased root → each file one instance, twin views share it, stored paths canonical; arrangement `key_ok` liveness true for a legacy joined key. |
| REQ-005 | `links.rs` units: canonical root + RELATIVE ref → rows attach to the canonical open_path; PIN the accepted miss: absolute-verbatim ref + canonical open_path → no attach (documents D10). |
| REQ-006 | Existing #310/#312/#322/#397 suites green (gate:3); cross-root find_open behavior — verify an existing test covers two roots → two instances, add one at content level if missing. |
| REQ-007 | `scripts/gates.sh --diff` green (exit code; receipt written). |

Uncoverable paths: none new (all seams drive headless).

### Risks
- R1 — host-gate is per-pump-tick hot path → lazy croot preserves the
  cheapest-first order; syscall only when a .rs doc exists while hostless.
- R2 — D10's accepted abs-verbatim-ref miss under symlinked roots (cosmetic
  gutter; pinned by test; rel refs unaffected; zero delta on normal roots).
- R3 — canonical spellings become user-visible in rel-label fallbacks → D11
  covers the four known sites; inspect sweeps for stragglers (`.path` display).
- R4 — `marley_project` gains fs-touching fns → mutation-killable with the
  symlink/tempfile tests; crate keeps its 100% bars.
- R5 — persisted blobs now WRITE canonical spellings → old blobs restore via
  the same helper (collapse), new blobs are absolute paths any prior reader
  resolves (pass-through); no wire-format change.
- R6 — double-canonicalize on open gestures (probe + loader) — idempotent, one
  extra stat-class syscall per user gesture; negligible.

## Phase 3 — Implement
- **React-first:** N/A (per spec — no UI delta).
- **Built (to the Phase 2 manifest):**
  - `marley_project/src/lib.rs`: `canonical_under_root` + `rel_under_root` after
    `resolve_under_root`, with doctests (missing-file fallback; verbatim strip +
    outside-root pass-through).
  - `marley_app/src/app.rs`: resolver swaps at the loader (5553-area), both
    probes, both restore arms, arrangement `key_ok` + mount; D7
    normalize-at-entry in `open_and_place_caret`; D8 as an extracted free fn
    `wants_rust_host(root, paths)` (lazy canonical root inside the `any`
    closure; `ensure_lsp_host_for_open_docs` signature drops its `mut`); D9
    filters rank by `h.root()` in `jump_to_symbol` + `jump_to_problem`; D10
    canonical root in `open_file_diagnostic_rows`; D11 `rel_under_root` at the
    git-gutter pathspec, RevealInTree, search-overlay + problems-panel labels.
  - `marley_app/src/lsp_host.rs`: `pub(crate) fn root(&self)` accessor (+ doc:
    canonical since #309; prefix-test against THIS, not the map key).
  - Doc updates: `same_file` + `EditorInstance.path` (editor_surface.rs),
    `find_open` (content.rs), the links.rs caller contract + accepted
    abs-verbatim miss, three headless_drive fixture comments now point at the
    fix.
- **Deviations from design:** one, anticipated — `lsp_host.rs` was not in the
  manifest table but D9 said "expose an accessor if absent"; it was absent, so
  the accessor was added there (3 lines + doc).
- **Checks:** `cargo fmt --all` applied; `cargo check --workspace` green;
  `cargo clippy --workspace --all-targets` exit 0 (the `block v0.1.6`
  future-incompat note is pre-existing, transitive, untouched).

## Phase 3.5 — Inspect
Four critics (correctness, security/containment, data/state-integrity,
simplification/reuse) ran in parallel over the diff, fed the prior failure
classes (PR-292, PR-194, PR-557, PR-1203, BF-rename-misroutes, #289 F1).
Security returned ZERO findings (12 attack vectors traced to ground: gate is
fail-closed via the Cargo.toml co-gate; `git diff` keeps its `--` guard; argv
spawn is doc-path-free; reveal can't escape; provenance clean).

| # | Sev | Finding | Verdict | Action |
|---|---|---|---|---|
| 1 | MED | `LspHost::root()` was inserted between `encoding()`'s doc+`mutants::skip` and its body — attrs bind to the NEXT item, so `root()` stole the doc+skip and `encoding()` lost both (3 critics independently) | REAL | Restructured: `root()` above with its own doc (no skip — no viable mutant for a `&Path` accessor); `encoding()`'s doc+skip adjacency restored |
| 2 | MED | ⌘⇧F search: live-buffer `overrides` keyed by STORED (canonical) paths, but `run_search` walks the VERBATIM root — yielded paths inherit the walk-root spelling → every override misses under a symlinked root → dirty-buffer-wins silently searches stale disk; jump offsets computed against wrong text. #310-F1 recurrence the design sweep missed (walkers are PRODUCERS of probe keys) | REAL (verified at app.rs consume_search_query / run_search) | `SearchReq.root = canonical_root(&root)`; instance filter keeps the verbatim root; both overlay renders hoist `canonical_root` so per-row strips stay pure |
| 3 | MED | 3 in-diff hand-rolls of root-canonicalize + `LspHost::absolute` duplicating `canonical_under_root` wholesale | REAL (reuse) | `marley_project::canonical_root` added (fail-OPEN caveat documented); adopted at `wants_rust_host`, `open_file_diagnostic_rows`, search fix, `rel_under_root` fallback, `LspHost::new`; `LspHost::absolute` now delegates to `canonical_under_root` |
| 4 | MED | `ensure_lsp_host_for_open_docs` cost-doctrine comment ("borrow-and-compare") now false — the scan can canonicalize the root | REAL (doc) | Comment owns the at-most-one lazy syscall |
| 5 | MED (pre-existing) | `open_file_in_viewer` fresh-open statted the INCOMING `path` (possibly relative → CWD `/` → snapshot None → #275 chokes blind until first ⌘S); twin `split_file_pane` stats `full` | REAL, pre-#319 (verified byte-identical at HEAD) | One-token in-scope hardening: stat `&full`, comment explains |
| 6 | LOW | `editor_folds` keyed by path alone — two alias-root WORKSPACES of one dir now share a fold entry (D3 keeps them separate instances) | ACCEPTED (exotic, clamped anchors, nothing persists) | Documented at the map decl |
| 7 | LOW | `c=` grid-framing drop can newly trip when a symlink TARGET contains `,:=` etc. (canonical spelling persisted) | ACCEPTED (existing drop stance; alias spelling clean → exotic) | Recorded here; no code change |
| 8 | LOW | links.rs doc claimed rel refs "work consistently" — false when a ref's components cross an in-project symlink (lexical join) | REAL (doc) | Doc corrected: both miss classes named; REQ-005 pins them |
| 9 | LOW | `wants_rust_host` excludes an open `.rs` whose canonical target lies OUTSIDE the root (in-project symlink) — pre-#319 the verbatim spelling passed | ACCEPTED semantic (Zed `is_external` analog; host still syncs it once spawned) | Documented in the fn doc |
| 10 | LOW | `open_file_diagnostic_rows` gains a per-call canonicalize (per-frame while an editor renders) | ACCEPTED (fn already rescans terminal outputs per call — dominant cost; cross-tick caching goes stale on re-symlink) | Comment records the tradeoff |
| 11 | LOW | `same_file` issues 2 canonicalize syscalls to conclude equality in the now-dominant identical-spelling case | REAL (perf) | Raw-`==` fast path added |
| 12 | LOW | Stale comments: file-ref menu still cited `resolve_under_root`; loader comment likewise | REAL (doc) | Both updated |

Deferred to Phase 4: the links.rs "(pinned by test)" claim requires the REQ-005
pin to actually land (validator owes it); `root()` accessor gets a
mutation-killing assert in an existing lsp_host test if cargo-mutants emits a
viable mutant for it.
Post-fix verification: `cargo fmt --all` + `cargo clippy --workspace
--all-targets` exit 0.

## Phase 4 — Validate
- **Tests added (11 new, all RUN + PASS in `cargo nextest run --workspace`:
  2059 passed / 0 failed / 5 skipped; doctests: marley_project 2/2 ok):**
  - `marley_project::tests` — `canonical_under_root_collapses_a_symlinked_root`
    (REQ-003: relative/alias-absolute/canonical inputs converge),
    `canonical_under_root_missing_file_falls_back_to_the_join` (REQ-003 fallback,
    kills the unwrap_or arm mutants),
    `canonical_root_and_rel_under_root_cover_both_spellings` (REQ-003: root half +
    inverse, incl. the fail-open vanished-root arm).
  - `marley::content::tests::find_open_hits_across_alias_spellings_and_roots_stay_raw`
    (REQ-001: alias-spelling HIT with `make` proven never to run — the live buffer,
    unsaved edits included, is what mounts; REQ-006: the raw root half pinned).
  - `marley::app::tests::wants_rust_host_matches_canonical_docs_under_an_alias_root`
    (D8: canonical doc passes under the alias root; extension gate; outside-target
    exclusion; empty-set false).
  - `marley::app::tests::run_search_walks_the_canonical_root_so_live_overrides_hit`
    (inspect F2 regression pin: canonical walk hits the live override;
    verbatim alias walk pinned as the miss).
  - `marley::headless_drive::goto_definition_lands_across_alias_spellings_headless`
    (REQ-001/REQ-002: alias-opened target + canonical F12 uri → ONE instance,
    landing passes, caret parks — the ticket's headline repro, on an explicit
    symlink alias).
  - `marley::headless_drive::legacy_alias_spelled_layout_restores_to_one_canonical_instance`
    (REQ-004: a legacy blob listing BOTH spellings of one file restores to one
    instance with the canonical stored identity; real codec via seed_shell_layout).
  - `marley::links::tests` — `diagnostics_attach_rel_refs_under_a_canonical_root`
    (REQ-005 + the trace twin), `diagnostics_abs_verbatim_alias_ref_is_the_documented_miss`
    + `diagnostics_rel_ref_across_internal_symlink_is_the_documented_miss`
    (both D10 accepted misses pinned — the links.rs doc's "pinned by test" claim
    is now true).
- **REQ-006 rest**: full existing suite green (incl. #310/#312/#322/#397 lanes) in
  the same run.
- **Live-app smoke (input-path change → driven, not deferred):** bundled
  (`bundle-app.sh debug`); the `open`-launch hit the README's known TCC stall
  (process alive, zero AX windows) → relaunched by direct exec from the project
  dir per the README remedy. Captures (READ, both):
  `…/scratchpad/marley-319-boot.png` — the user's real persisted workspace
  restored THROUGH the new canonical restore arms: editor tab + inspect.md row,
  twin split panes, four terminal sessions, prompt bottom-anchored (a real-world
  legacy-blob restore smoke). `…/scratchpad/marley-319-open.png` — after
  activation + a driven sidebar click: inspect.md rendered in the focused editor
  split (line numbers, content, cyan focus border, status bar `focus: editor`),
  no error flash. The symlinked-root behavioral delta itself is headless-proven
  (the two drive tests boot the real RootView; README: headless-first for
  behavior, pixels for rendering). Noted: the user's own
  `~/Projects/ignibyte/Marley` is a SYMLINK to this repo — the aliased-root
  shape is real on this machine, not hypothetical.
- **Parity pair**: N/A (spec React-first is N/A — no UI delta).
- **Pre-existing failures**: none (suite fully green).
- **Gate**: `scripts/gates.sh --diff` → **GATE GREEN [diff]** (15/15 PASS; receipt
  written). First run was RED on two gates, both fixed at source: (1) gate:14
  brand-scrub — the `wants_rust_host` doc named the editor reference app by
  brand; reworded to point at the spec's Reference section. (2) gate:4 coverage
  100% — the content.rs test's make-must-not-run proof was a multiline `panic!`
  closure whose lines are unreachable BY DESIGN; replaced with an inline
  `|| None` make on the executed line (a broken hit would surface the None and
  fail the `expect`) — same guarantee, no dead lines. Mutation (gate:5, MSI 100
  on touched lines) was green on the first run.

## Phase 5 — Complete
- **Docs (§21)**: CHANGELOG.md gains the TICKET-319 `### Fixed` entry (full
  seam/consumer inventory + the inspect-found search fix + test/gate results).
  Architecture: `docs/marley_architecture/marley_project.md` records the three
  new helpers (surface block + per-fn bullets + the mutation-caveat note);
  `docs/marley_architecture/editor.md` gains the **File identity (#319)**
  paragraph beside the #322 rename-identity record. Parity sync: N/A (no UI
  delta).
- **Ledger appends (codes)**:
  - `L-claude-identity-unification-sweeps-prefix-and-strip-consumers-001`
    (lessons.md — appended at Design).
  - `F-claude-search-walk-verbatim-root-misses-canonical-override-keys-001`
    (failures.md — appended at Inspect).
  - `PR-claude-walker-yield-spelling-must-match-store-keys-001`
    (prevention-rules.md — appended at Inspect).
  - `AD-claude-editor-file-identity-canonical-at-storage-001`
    (architecture-decisions.md — appended here).
- **Ticket**: TICKET-319 closed → `tickets/closed/`; its BACKLOG row left at
  promotion (swept — no stale row).
- **Archive**: pipeline pair → `docs/planning/pipeline/completed/`.
