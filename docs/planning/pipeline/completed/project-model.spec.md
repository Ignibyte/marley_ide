---
pipeline_id: 1fa8c532-9e07-4cef-8185-050e15f01629
ticket: forge#53 (c8e50f7b-316e-4724-8c98-0f2580aadfb7) · local docs/planning/tickets/open/TICKET-053-project-model.md
aar_id: 150002f3-f519-48de-bbe3-00911ea86b29
status: Phase 5 — Complete PASS
title: project model — discover the project root
type: feature
milestone: M2.A
references:
  - crates/marley_project/ (NEW crate — Project + discover_in)
  - Cargo.toml (workspace members = crates/* — auto-included)
---

## Title
The FOUNDATION of the M2 moat: Marley becomes aware of a PROJECT (a repo), not just a cwd. A pure
`Project` model + `discover_in` (git-root walk-up). #54 (search) / #55 (file listing) / #56 (file
tree) build on this; the docks + fuzzy-open get their context from it.

## Scope
### In
- `crates/marley_project/` (NEW crate, gpui-free PURE — cov/MSI 100):
  - `pub struct Project { pub root: PathBuf, pub name: String, pub is_git: bool }` (Debug/Clone/Eq).
  - `Project::discover_in(start: &Path) -> Project` — from `start`, then each `.parent()`, return the
    FIRST ancestor that CONTAINS a `.git` entry (a dir OR a file — git worktrees/submodules use a
    `.git` FILE) as `root` with `is_git = true`; if none up to the filesystem root, `root = start`,
    `is_git = false`. `name = name_for(&root)`.
  - `fn name_for(root: &Path) -> String` (pure, tested directly) — `root.file_name()` as a lossy
    `String`; when `file_name()` is `None` (e.g. `/`), fall back to `root`'s lossy display string. (A
    separate fn so the `/` fallback branch is directly testable — otherwise it's uncoverable.)
  - `Cargo.toml` (mirror the minimal-crate pattern; std-only runtime deps; `tempfile` dev-dep).
- Wire the crate into the workspace (auto via `crates/*`).

### Out (explicitly deferred)
- The app-side wiring (discover the Project at startup from the session cwd + hand it to the docks) —
  a later step once #55/#56 give it consumers; this ticket ships the pure model + crate. A full
  `.gitignore`/monorepo/nearest-vs-toplevel policy — the first cut is "nearest `.git` ancestor". Any
  git metadata beyond `is_git` (branch, remote) — later.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — NEW crate `marley_project` (not in marley_core) — the project/files domain; #55 file-listing +
  #56 file-tree join it. gpui-free, std-only runtime (dev-dep `tempfile` for the tmp-tree tests).
- D2 — `.git` is detected as a DIR **or** a FILE (`root.join(".git").exists()`) — worktrees + submodules
  point `.git` at a file; a dir-only check would misclassify them.
- D3 — NEAREST `.git` ancestor wins (stop at the FIRST, walking up) — the innermost repo, matching
  `git rev-parse --show-toplevel` for the common case.
- D4 — No `.git` anywhere → `root = start`, `is_git = false` (Marley still works in a plain dir).
- D5 — `name` via a separate pure `name_for` (basename, with a `/`-fallback) so both branches are
  cov/MSI-testable.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `discover_in(start)` is called and an ancestor of `start` (or `start` itself) contains a `.git`, it shall return the NEAREST such ancestor as `root` with `is_git = true`. | unit (tmp `a/.git` + `discover_in(a/b/c)` → root `a`) |
| REQ-002 | WHEN a `.git` is a FILE (worktree/submodule), `discover_in` shall still detect it (root there, `is_git = true`). | unit (tmp `w/.git` FILE) |
| REQ-003 | WHEN no `.git` exists up to the filesystem root, `discover_in` shall return `root = start`, `is_git = false`. | unit (tmp with no `.git`) |
| REQ-004 | WHEN `name_for(root)` is called, it shall return the root's basename; WHEN the root has no basename (`/`), it shall return the root's display string. | unit (`/tmp/a`→`a`; `/`→`/`) |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `marley_project`. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the crate skeleton, `discover_in` + `name_for` shapes, the mutation targets + the
  tmp-tree test approach.
- **P3 Implement** — the crate (`Cargo.toml` + `lib.rs`) + `Project`/`discover_in`/`name_for`.
- **P3.5 Inspect** — critic: nearest-vs-farthest walk, the `.git` dir-OR-file check, the no-git
  fallback, the name basename + `/` fallback, symlink/permission edge (no panic), equivalent mutants.
- **P4 Validate** — the discover_in + name_for unit tests (tmp trees) + gate GREEN. (Pure model, no
  UI → no self-test harness needed.)
- **P5 Complete** — docs, AAR, archive, close #53.
