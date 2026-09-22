# project model — discover the project root — Notes

- **Forge ticket:** #53 `c8e50f7b-316e-4724-8c98-0f2580aadfb7`
- **AAR:** `150002f3-f519-48de-bbe3-00911ea86b29`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-053-project-model.md
- **Pipeline spec:** project-model.spec.md

## Phase 1 — Plan
- **Request:** forge #53 (M2.A "Project, Files & Search" seq-1, auto-approved) — the moat foundation:
  a `Project` model + git-root discovery.
- **Classification / tier:** work pipeline, `feature`, a PURE new crate `marley_project` (cov/MSI 100).
  No UI.
- **Discovery (§18):** workspace `members = ["crates/*"]` → a new `crates/marley_project` is
  auto-included. Mirror the minimal-crate `Cargo.toml` (marley_text_offsets): `[package]` with
  `edition.workspace`/`license.workspace`, deps, `[dev-dependencies]`. `marley_core` has paths but
  `discover_in` is pure std path-walking + `Path::exists` — no marley_core runtime dep needed. `tempfile`
  is already used in dev-deps elsewhere (shell_integration tests).
- **Decisions:** D1–D5 in the spec (new crate; `.git` dir-OR-file; nearest ancestor; no-git fallback;
  `name_for` extracted for the `/` fallback's coverage).
- **Key testability note:** `name_for(root)` is a SEPARATE pure fn so the `file_name() == None` (`/`)
  fallback branch is directly unit-testable — inside `discover_in` it'd be uncoverable (can't make
  `discover_in` return root `/` from a tmp tree).
- **Open for Design:** whether `.git` existence uses `try_exists()` (handles permission errors as
  Err → treat as "no .git") vs `exists()` (a permission error → false) — lean `exists()` (simplest;
  a `.git` we can't stat is effectively not-a-repo-root for us). The walk terminates at `parent() ==
  None` (filesystem root).
- **AAR id:** `150002f3-f519-48de-bbe3-00911ea86b29`.

## Phase 2 — Design

### PURE — `crates/marley_project/src/lib.rs` (NEW crate)
```rust
use std::path::{Path, PathBuf};

/// A discovered project — the git repo (or plain dir) the terminal works in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub root: PathBuf,   // the nearest `.git` ancestor, or `start` if none
    pub name: String,    // the root's basename (or its path for `/`)
    pub is_git: bool,
}

impl Project {
    /// The project containing `start`: the NEAREST ancestor (or `start` itself) holding a `.git`
    /// entry (dir OR file) is the root; if none up to the filesystem root, `start` is the root and
    /// `is_git` is false.
    pub fn discover_in(start: &Path) -> Project {
        let mut dir = start;
        loop {
            if dir.join(".git").exists() {
                return Project { root: dir.to_path_buf(), name: name_for(dir), is_git: true };
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
        Project { root: start.to_path_buf(), name: name_for(start), is_git: false }
    }
}

/// A project root's display name: its basename, or its whole path when it has none (`/`).
fn name_for(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.to_string_lossy().into_owned())
}
```
`Cargo.toml`: mirror the minimal-crate pattern (`edition.workspace`, `license.workspace`); NO runtime
deps (std only); `[dev-dependencies] tempfile = "3"`.

### File manifest
- A `crates/marley_project/Cargo.toml` — new crate.
- A `crates/marley_project/src/lib.rs` — `Project` + `discover_in` + `name_for` + tests.
- (workspace `members = ["crates/*"]` auto-includes it; the gate's `--workspace` cov/mutation covers
  it even with no consumer yet.)

### Mutation Targets
- `discover_in` — the `dir.join(".git").exists()` check (a `true`/`false` mutant → wrong root/fallback);
  the walk-up (return at the FIRST `.git`, not continue — a nested-nearest test kills "walk to
  outermost"); the `parent()` `Some`/`None` termination; the no-git fallback (`root=start`,
  `is_git=false`). `name_for` — `file_name()` basename + the `unwrap_or_else` `/` fallback.

### Regression Test Plan (all pure unit; no UI → no self-test harness)
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `discover_finds_nearest_git` — tmp `outer/.git` + `outer/inner/.git`, `discover_in(outer/inner/sub)` → root `outer/inner` (NEAREST, not `outer`), `is_git`, name `inner` | unit (tmp tree) |
| REQ-002 | `discover_detects_git_file` — tmp `wt/.git` as a FILE → `discover_in(wt)` → `is_git`, root `wt` (kills a dir-only check) | unit |
| REQ-003 | `discover_no_git_falls_back` — tmp `plain/deep` (no `.git` up the tree) → root `plain/deep` (start), `is_git=false` | unit |
| REQ-004 | `name_for_basename_and_root` — `name_for("/tmp/foo")=="foo"`; `name_for("/")=="/"` | unit |
| REQ-005 | `scripts/gates.sh --diff` GREEN, cov/MSI 100 on marley_project | gate |

Uncoverable: none (all pure + tmp-tree testable). Note: the no-git test relies on `tempfile::tempdir()`
(under `/var/folders/…` on macOS) having no `.git` ancestor — true on macOS/CI.

### Risks / decisions
- D-2.1 `exists()` (not `try_exists()`) for `.git` — a `.git` we can't stat is treated as absent
  (fine: it's not a usable repo root for us). D-2.2 The walk terminates at `parent() == None`
  (filesystem root) — no infinite loop. D-2.3 `name_for` extracted so the `/` fallback is coverable.
  D-2.4 Orphan crate (no consumer yet) is fine — `--workspace` builds/tests/covers it; machete flags
  unused DEPS, not unused crates. The app wiring is a later ticket.

## Phase 3 — Implement
- **Built:** `crates/marley_project/Cargo.toml` (minimal-crate pattern, no runtime deps, dev-dep
  `tempfile`) + `crates/marley_project/src/lib.rs` — `Project { root, name, is_git }`,
  `Project::discover_in` (walk-up, nearest `.git` dir-or-file, no-git fallback to start), `name_for`
  (basename + `/` fallback). Crate + module + all public items documented (`//!`/`///` for gate:14).
- **Deviations:** none — matches the design verbatim.
- **Verification:** `cargo fmt`; `cargo check -p marley_project` 0 err; `cargo metadata` confirms the
  crate is auto-included in the workspace (built/tested/covered by `--workspace`). Tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (verbatim probe of all edge cases + a real scoped `cargo-mutants` + a coverage
  measurement). Verdict: **PASS/SHIP** — cov 100 + MSI 100, no code defects.
- **Findings (both advisory, no code fix):**
  | # | Sev | Finding | Verdict | Action |
  |---|---|---|---|---|
  | F1 | MED (informational) | MSI 100 for `discover_in` is HOLLOW: cargo-mutants emits only 1 mutant for it (whole-body `Default::default()`), UNVIABLE (Project has no `Default`) → excluded; it does NOT mutate `.exists()`, the in-loop `return`, the `parent()` arms, or the field assignments. So the walk/`.git`-file/no-git logic yields ZERO viable mutants; MSI 100 rides on `name_for`'s 2 String mutants. Correctness is guarded by **coverage (100%: 123/123 regions) + the behavioral REQ-001..003 assertions**, not mutation. | REAL (nature of path-walking code; not the "0 viable → fail-closed" trap since `name_for` gives 2 viable) | P4 tests must be BEHAVIORALLY thorough (nearest≠farthest, .git-FILE, no-git→start). Critic PROVED sufficiency: `#[derive(Default)]` makes the mutant viable → CAUGHT by REQ-001/003 → no extra test needed. Note in the test file. |
  | F2 | LOW | The no-git test (REQ-003) trusts no `.git` exists up to `/` from a `tempfile::tempdir()`. Verified NONE on this machine (`/.git`, `/var/.git`, `/private/var/.git`, `/var/folders/*/*/T/.git` all absent). A `git init` in `$TMPDIR` would break it — but LOUDLY (is_git assert flips), never silently wrong. | REAL but acceptable | Add a one-line comment in the test recording the ambient-FS assumption. |
- **Verified CORRECT (probe):** nearest wins (`outer/.git`+`inner/.git`, `discover_in(outer/inner/sub)`
  → `inner`, NOT `outer`); `.exists()` catches a `.git` FILE (worktree — `.is_dir()` would MISS it,
  so REQ-002 is the guard); terminates at `/` → returns start (not `/`), no infinite loop;
  `name_for("/")` → the `unwrap_or_else` closure runs (covered+asserted); trailing slash stripped; no
  panic path; Cargo.toml matches siblings; orphan crate builds/covers under `--workspace`; fmt+clippy
  clean; §20 trivial. MSI 2/2 = 100.
- **No code fix** — F1/F2 are P4 test-thoroughness + a comment.

## Phase 4 — Validate
- **Tests added** (`lib.rs` tests mod): `discover_finds_nearest_git` (REQ-001 — nearest≠outer +
  start-has-git + climb-to-outer), `discover_detects_git_file` (REQ-002 — `.git` FILE), `discover_no_git_falls_back`
  (REQ-003 — no repo → start, with the F2 ambient-FS comment), `name_for_basename_and_root` (REQ-004 —
  basename + `/` + trailing-slash). F1 note recorded in the test mod.
- **Runs (actual):** `cargo nextest run -p marley_project` → 4 passed.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **2 caught / 0
  missed → MSI 100.0%**. Receipt written.
- **UI:** N/A — library crate, no UI surface (self-test harness not applicable).
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (new crate); new arch doc `marley_project.md`; crate-map
  mermaid node + table row.
- **Knowledge:** prevention rule `PR-claude-method-call-code-mutation-hollow-001` (medium) — method-call/
  struct-heavy pure code yields few/no viable mutants → MSI 100 is hollow; coverage + behavioral
  assertions are the guard (sibling of #50's vacuous-MSI rule). aar-submit `completed` (5).
- **Ticket:** forge #53 → done; local doc → closed/; pipeline pair archived. **1/6 of M2.A.**
