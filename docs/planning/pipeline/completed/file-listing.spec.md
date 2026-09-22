---
pipeline_id: 926f7778-6eb1-4a1e-acd7-ccbda98b026f
ticket: forge#55 (11e051df-031d-4b9e-a3be-ce34c34e9d88) · local docs/planning/tickets/open/TICKET-055-file-listing.md
aar_id: f30f381f-069f-482f-8bf4-785624f43db7
status: Phase 5 — Complete PASS
title: project file listing — walk + skip
type: feature
milestone: M2.A
references:
  - crates/marley_project/src/lib.rs (ADD should_skip + list_files_in + FileListing)
---

## Title
Enumerate a project's files: a curated `should_skip` + a bounded, testable `list_files_in` recursive
walk. Feeds the Files-dock tree (#56) + fuzzy file-open (#57). Joins the `marley_project` crate.

## Scope
### In (all in `crates/marley_project/src/lib.rs`)
- `pub fn should_skip(component: &str) -> bool` — true for the curated skip-set `.git`, `node_modules`,
  `target`, `.DS_Store` (first cut).
- `pub struct FileListing { pub files: Vec<PathBuf>, pub truncated: bool }` (Debug/Clone/Eq).
- `pub const MAX_FILES: usize = 10_000;`
- `pub fn list_files_in(root: &Path) -> FileListing` = `list_files_capped(root, MAX_FILES)`.
- private `fn list_files_capped(root: &Path, cap: usize) -> FileListing` — a stack walk: for each dir,
  `read_dir` (an unreadable dir is SKIPPED, not a panic), sort entries by file-name; for each entry,
  `should_skip(name)` → skip; use `entry.file_type()` (does NOT follow symlinks) — a real DIR is pushed
  to the stack, a real FILE is collected as a path RELATIVE to `root` (`strip_prefix`); symlinks are
  ignored (loop-safe). Stop collecting at `files.len() == cap` (→ `truncated = true`). Finally
  `files.sort()`.

### Out
- A `.gitignore` parser / hidden-file policy beyond the curated set — later. Symlink following —
  intentionally not (loop safety). Following the walk into the app/docks — #56/#57.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — Use `DirEntry::file_type()` (from `read_dir`, does NOT stat-follow symlinks) for the dir/file
  decision — a symlink is neither `is_dir` nor `is_file` here, so it's ignored and cannot cause a
  walk loop (a symlink to an ancestor would otherwise recurse forever). Load-bearing safety.
- D2 — BOUNDED with a `truncated` flag (not a silent cap, §7); `MAX_FILES` const; the private
  `list_files_capped(root, cap)` makes the bound testable with a SMALL cap (no 10k-file test).
- D3 — A skipped DIR prunes its whole subtree (we never push it). An unreadable dir / entry with an
  errored `file_type` is skipped, never a panic (no `unwrap` on IO).
- D4 — Paths are RELATIVE to `root` (`strip_prefix`); the final list is `sort`ed for determinism.
- D5 — Same crate as #53 (`marley_project`); `root` is any `&Path` (typically `Project::root`).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `should_skip(name)` is called, it shall be `true` for exactly `.git`, `node_modules`, `target`, `.DS_Store` and `false` otherwise. | unit |
| REQ-002 | WHEN `list_files_in(root)` walks a tree, it shall return every non-skipped FILE as a path relative to `root`, sorted, and shall NOT descend into a skipped dir. | unit (tmp tree with `.git`/`target`/`node_modules`/`.DS_Store` + real files) |
| REQ-003 | WHEN the walk reaches `cap` files, `list_files_capped` shall stop and set `truncated = true`. | unit (3-file tree, cap 2) |
| REQ-004 | WHEN a directory cannot be read, `list_files_in` shall skip it without panicking. | unit (walk succeeds over a tree; no panic) + a non-existent root → empty, not panic |
| REQ-005 | `scripts/gates.sh` GREEN, cov/MSI 100 on the new code. | gate |

## Phase Plan
- **P2** — the `should_skip`/`FileListing`/`list_files_capped` shapes, the symlink-safe file_type walk,
  the mutation targets, the tmp-tree test plan.
- **P3** — implement in marley_project/src/lib.rs.
- **P3.5** — critic: the prune correctness, symlink-loop safety, the cap `== / >=` boundary, relative
  path strip, the truncated flag, unreadable-dir no-panic, mutants.
- **P4** — the unit tests (tmp trees + small-cap truncation) + gate GREEN. No UI.
- **P5** — docs (arch marley_project.md update + CHANGELOG), AAR, archive, close #55.
