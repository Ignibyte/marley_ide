# project file listing — walk + skip — Notes

- **Forge ticket:** #55 `11e051df-031d-4b9e-a3be-ce34c34e9d88`
- **AAR:** `f30f381f-069f-482f-8bf4-785624f43db7`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-055-file-listing.md

## Phase 1 — Plan
- **Request:** forge #55 (M2.A seq-3, auto-approved) — project file enumeration.
- **Classification:** work pipeline, `feature`, PURE additions to the existing `marley_project` crate.
  No UI.
- **Key design (D1) — symlink-loop safety:** the walk decides dir-vs-file via `DirEntry::file_type()`
  (from `read_dir`), which does NOT follow symlinks, so a symlink to an ancestor is ignored rather than
  recursed → no infinite walk. The `cap` bounds file count but NOT a dir-only symlink cycle, so
  file_type-not-following is the actual loop guard.
- **Bound (D2):** `list_files_capped(root, cap)` private so truncation is testable with a small cap;
  `list_files_in` = `list_files_capped(root, MAX_FILES=10_000)`; `truncated` flag (no silent cap, §7).
- **No-panic (D3):** `read_dir`/`file_type` Errs → skip (match, no unwrap). A non-existent root →
  `read_dir` Err → empty listing, not a panic.
- **Hollow-MSI watch (#53 rule):** the walk is method-call-heavy (read_dir/file_type/strip_prefix), but
  `should_skip` (membership), the `cap` check, and the `truncated`/prune branches give viable mutants;
  behavioral tests + coverage guard the method-call parts.
- **AAR id:** `f30f381f-069f-482f-8bf4-785624f43db7`.

## Phase 2 — Design

### PURE — ADD to `crates/marley_project/src/lib.rs`
```rust
/// Directory/file names skipped when listing a project's files (first cut — a full .gitignore is later).
pub fn should_skip(component: &str) -> bool {
    matches!(component, ".git" | "node_modules" | "target" | ".DS_Store")
}

/// The result of [`list_files_in`]: the files (relative to the root) + whether the cap stopped the walk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileListing {
    pub files: Vec<PathBuf>,
    pub truncated: bool,
}

/// The most files [`list_files_in`] collects before it stops and marks the listing truncated.
pub const MAX_FILES: usize = 10_000;

/// List every non-skipped regular file under `root`, as a path relative to `root`, sorted. Skipped
/// directories ([`should_skip`]) are not descended; symlinks are ignored (the walk cannot loop);
/// unreadable directories are skipped; at most [`MAX_FILES`] are returned.
pub fn list_files_in(root: &Path) -> FileListing {
    list_files_capped(root, MAX_FILES)
}

fn list_files_capped(root: &Path, cap: usize) -> FileListing {
    let mut files = Vec::new();
    let mut truncated = false;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&dir) else { continue }; // unreadable dir → skip, no panic
        let mut entries: Vec<std::fs::DirEntry> = read.filter_map(Result::ok).collect();
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            if should_skip(&entry.file_name().to_string_lossy()) {
                continue;
            }
            match entry.file_type() {
                Ok(ft) if ft.is_dir() => stack.push(entry.path()),
                Ok(ft) if ft.is_file() => {
                    if files.len() >= cap {
                        truncated = true;
                        break;
                    }
                    if let Ok(rel) = entry.path().strip_prefix(root) {
                        files.push(rel.to_path_buf());
                    }
                }
                _ => {} // symlinks (loop-safe) + a rare errored file_type — ignored
            }
        }
        if truncated {
            break;
        }
    }
    files.sort();
    FileListing { files, truncated }
}
```

### File manifest
- MODIFY `crates/marley_project/src/lib.rs` — add `should_skip`, `FileListing`, `MAX_FILES`,
  `list_files_in`, `list_files_capped`, + tests. (No Cargo change — `tempfile` dev-dep already present.)

### Mutation Targets
- `should_skip` — the 4-name `matches!` set (dropping one → that dir leaks; a test asserts each). The
  walk: the `should_skip` prune (skipped dir's children excluded), `Ok(ft) if is_dir`/`is_file` arms +
  the `_` (symlink) arm, the `files.len() >= cap` boundary + `truncated=true`, the `strip_prefix`, the
  final `sort`. Method-call parts (read_dir/file_type) — hollow-MSI (#53), guarded by coverage +
  behavioral asserts.

### Regression Test Plan (pure unit, tmp trees; no UI)
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `should_skip_matches_curated_set` — true for `.git`/`node_modules`/`target`/`.DS_Store`, false for `src`/`main.rs`/`.gitignore` | unit |
| REQ-002 | `list_files_walks_and_prunes` — tmp tree `a.txt`, `sub/b.txt`, `.git/x`, `target/x`, `node_modules/x`, `.DS_Store` → files == `[a.txt, sub/b.txt]` (relative, sorted), `truncated=false` (skipped dirs pruned incl. their contents) | unit |
| REQ-002b | `list_files_ignores_symlinks` `#[cfg(unix)]` — a symlink among real files → not listed (covers the `_` arm + proves loop-safety) | unit |
| REQ-003 | `list_files_capped_truncates` — flat tree `a,b,c`, `list_files_capped(root, 2)` → 2 files + `truncated=true` | unit |
| REQ-004 | `list_files_missing_root_is_empty_not_panic` — `list_files_in(root.join("nope"))` → `{files:[], truncated:false}` (read_dir Err path, no panic) | unit |
| REQ-005 | gate GREEN, cov/MSI 100 | gate |

Uncoverable: the `file_type()` Err sub-case is not directly triggerable, but the `_` arm it shares is
COVERED by the symlink test — no gap. Truncation determinism relies on a FLAT tree in REQ-003 (avoids
stack-order ambiguity); the final `sort` makes non-truncated output order-deterministic.

### Risks / decisions
- D-2.1 `match entry.file_type()` with `_ => {}` (vs `if let`) — makes the symlink/err arm a single
  coverable branch (symlink test covers it) + keeps symlinks non-recursed (loop-safe). D-2.2 REQ-003
  uses a flat tree so the surviving-2 set is deterministic under the LIFO stack. D-2.3 non-existent
  root exercises the `read_dir` Err `continue` (REQ-004) without a fragile chmod-000.

## Phase 3 — Implement
- **Built:** added `should_skip`, `FileListing`, `MAX_FILES`, `list_files_in`, `list_files_capped` to
  `marley_project/src/lib.rs` (verbatim from the design — the `match entry.file_type()` with the `_`
  arm for symlink/err). All public items documented.
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley_project` 0 err; `cargo clippy -p marley_project`
  clean. Tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (13-test probe + a RUN symlink-loop test on a watchdog thread + real cargo-mutants +
  clippy). Verdict: **code CORRECT** — no code defect; the HIGH is the Phase-4 test suite (expected).
- **Findings:**
  | # | Sev | Finding | Verdict | Action |
  |---|---|---|---|---|
  | F1 | HIGH | No tests yet → cov/MSI 0 (7 viable mutants: should_skip→t/f, is_dir/is_file guards→t/f, `>=`→`<`). | EXPECTED (tests are Phase 4) | P4 writes the suite; critic PROVED 7/7 killable (probe MSI 100). Load-bearing kills: symlink-omission → `is_file guard→true`; cap matrix → `>=→<`; prune + per-name → `should_skip`/`is_dir`. |
  | F2 | LOW | `FileListing` has no `Default` → the 2 `→Default::default()` mutants are UNVIABLE (excluded). | Keep OFF | Matches #54 precedent — leaving Default off keeps those mutants excluded (no gain to add). #56 can derive it non-breaking if it needs an empty default. |
  | F3 | LOW | Truncated subset is traversal-order (LIFO) dependent, not lexicographically-first-N; the final sort can look like first-N-alpha but isn't. | Accept | Fine for "showing first N of many" — the `truncated` flag is the honest signal; which N is unspecified. Making it first-N-alpha would require walking everything (defeats the bound). |
  | F4 | LOW | Symlinks omitted entirely (both →file and →dir invisible), by design (D1 loop-safety). | Accept (documented) | Intended first cut; confirmed. |
  | F5 | LOW | `strip_prefix(root)` Err branch is dead (entry is always under root) — implicit else uncoverable. | Accept | gate:4 is LINE coverage (the `if let` + push lines run); no uncovered line, no surviving mutant (critic's mutant list has none for it). |
- **Verified (probe, RUN):** prune excludes skipped dirs' CONTENTS (`.git/x` absent), relative + sorted;
  **symlink loop TERMINATES (15 ms), symlinks absent**; cap matrix cap0..4 over 3 files correct (`>=`
  boundary right — cap==count → truncated=FALSE); non-existent/file root → `{[],false}` no panic; clippy
  `-D warnings` clean; §20 std-only. Probe MSI 7/7.
- **No code change** — F1 = Phase-4 tests; F2–F5 accepted.

## Phase 4 — Validate
- **Tests added** (`lib.rs` tests mod): `should_skip_matches_curated_set` (REQ-001),
  `list_files_walks_and_prunes` (REQ-002 — prune excludes skipped dirs' contents), `list_files_ignores_symlinks`
  `#[cfg(unix)]` (REQ-002b — loop-safety + the `_` arm), `list_files_capped_truncates` (REQ-003 — cap
  2→trunc, cap==count→no trunc), `list_files_missing_root_is_empty_not_panic` (REQ-004).
- **Runs (actual):** `cargo nextest run -p marley_project` → 9 passed (5 new + 4 discover/name).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **7 caught / 0
  missed → MSI 100.0%** (exactly the 7 viable mutants the critic predicted). Receipt written.
- **Environmental note (not a code issue):** the FIRST gate run hung on the real-PTY test
  `workspace_two_real_sessions_are_independent` (>720s) because a STRAY `cargo-mutants` proc from an
  earlier ticket was still loading the machine (the known #27 pattern). Killed the strays, confirmed
  the PTY test passes clean in 0.15s, re-ran the gate → GREEN. No source change.
- **UI:** N/A — library crate.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added`; `marley_project.md` file-listing section.
- **Knowledge:** aar-submit `completed` (5). Reused the #53 hollow-MSI + the #27 environmental-hang
  patterns (no new rule). Win: designed symlink-loop-safety in (file_type-not-following) BEFORE the
  critic, which then RAN a loop tree to confirm termination.
- **Ticket:** forge #55 → done; local doc → closed/; pair archived. **3/6 of M2.A** (halfway).
