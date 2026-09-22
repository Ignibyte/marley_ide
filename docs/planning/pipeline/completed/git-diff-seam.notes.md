# a read-only git-diff seam — Notes

- **Forge ticket:** #102 `d0450d67-eb2a-49eb-b807-d47bd193fe3d` · **AAR:** `14643a8e-1d40-422e-bf3f-10a58728c152`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-102-git-diff-seam.md

## Phase 1 — Plan
- **Request:** forge #102 (M4 6/10) — parse `git diff` + a ⌘⇧D working-diff overlay.
- **Pre-flight:** marley_command re-exports std::process::{Output,Stdio}; the root = Project::discover_in(
  &cwd).root (a local in new() — store it); parse_diff pure in git_diff.rs (tested), the git spawn masked
  in cov-excluded app.rs. ⌘⇧D appears free (verify at implement).
- **Decisions:** D1 parse pure / spawn masked; D2 separate overlay v1; D3 +/-/context/header colors.
- **AAR id:** `14643a8e-1d40-422e-bf3f-10a58728c152`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **git_diff.rs (NEW PURE):** DiffKind{Context,Add,Remove}; DiffLine{kind,text}; Hunk{header,lines}; FileDiff{path,hunks}. `parse_diff(output)`: iterate output.lines(), maintain cur:Option<FileDiff> + cur_hunk:Option<Hunk>; on `line.strip_prefix("diff --git ")` → push cur_hunk into cur, push cur into files, cur=Some(FileDiff{path: rest.rsplit_once(" b/").map(|(_,p)|p).unwrap_or(rest).to_string(), hunks:[]}); on `line.starts_with("@@")` → push cur_hunk into cur, cur_hunk=Some(Hunk{header:line, lines:[]}); else if cur_hunk.as_mut() Some → classify: strip_prefix(+)→Add / (-)→Remove / (space)→Context (text = the stripped rest); other → skip; final flush cur_hunk→cur, cur→files.
- **lib.rs:** `mod git_diff;` (after finder, before flash).
- **app.rs SHIM (cov-excluded/masked):** `project_root: PathBuf` field (=project.root in new()); `diff: Option<Vec<FileDiff>>` field (None); `#[cfg_attr(test,mutants::skip)] fn git_working_diff(&self)->Vec<FileDiff>` = std::process::Command::new("git").arg("diff").current_dir(&self.project_root).output() → String::from_utf8_lossy(stdout) → parse_diff; a ⌘⇧D chord (key=="d" + platform + shift) toggles self.diff; a diff overlay renders files→hunks→lines colored (Add success/Remove danger/Context muted/header accent); key dispatch: if self.diff.is_some(), Esc closes (before the other overlays).
- **Mutation targets:** parse_diff the diff--git detect + path rsplit, the @@ detect, the +/-/space classify, the flush order (cur_hunk before cur).
- **Test plan:** parse_diff_one_file (a 1-hunk diff → FileDiff{path,[Hunk{header,[Context,Remove,Add]}]}); parse_diff_multi_file (2); parse_diff_add_only; parse_diff_remove_only; parse_diff_empty (""→[]); parse_diff_ignores_prehunk (index/--- before @@). cov/MSI 100. The spawn+overlay masked (+ a live git diff self-test).
- **Risks:** ⌘⇧D must be free (verify); the spawn is read-only (git diff, no writes); a file with no hunks (binary/mode-only) → FileDiff{path,[]} kept.

## Phase 3 — Implement
- **Built:** git_diff.rs (DiffKind/DiffLine/Hunk/FileDiff + parse_diff, cov/MSI 100); `mod git_diff`; app.rs `project_root`+`diff` fields (from the widened project_files block), `git_working_diff` (masked, routes `git diff` through marley_command::blocking, parse_diff), ⌘⇧D open + Esc/⌘⇧D modal-close + the diff overlay (files→hunks→colored +/- lines).
- **DEVIATIONS:** (1) std::process::Command is clippy-disallowed (§14) → routed via marley_command::blocking::Command (added marley_command as a marley_app dep). (2) the project_files block widened to `(project_files, project_root)` so the root is in scope at construction.
- **Verification:** fmt; check 0 err; clippy OK; parse_diff 4 tests pass; live `git diff` available (smoke).

## Phase 3.5 — Inspect
- **Method:** self-review of parse_diff; the Phase-4 gate cargo-mutants is the authoritative MSI check (it caught #99's lexer gaps — same net).
- **Lenses — no findings:** parse_diff — `diff --git ` starts a file (path = the ` b/` side via rsplit_once, else the raw rest), `@@` starts a hunk, body +/-/space classified (prefix stripped into text), other/pre-hunk lines ignored; flush order = hunk→file then file→files (tested one/multi/add-only/remove-only/empty/mode-only-no-hunk); the git_working_diff spawn is READ-ONLY (`git diff`, no writes) via the sanctioned marley_command seam, fail-soft (Err→[]); the overlay is modal (Esc/⌘⇧D) + colors by kind; ⌘⇧D is a free chord. No unwrap/panic (from_utf8_lossy, rsplit_once fallback). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** parse_diff_one_file/multi_file/add_and_remove_only/empty_and_stray. `cargo nextest` → 4 passed. Live `git diff` produces parseable output (the spawn works).
- **Self-test:** the ⌘⇧D diff overlay needs a synthetic chord (ENV-BLOCKED); parse_diff engine-tested cov/MSI 100; the spawn+overlay masked.
- **Gate finding + fix:** first run RED on COVERAGE (git_diff.rs 98.4%) — the no-` b/` path fallback + the non-+/-/space (`\ No newline`) skip were unhit; added parse_diff_edge_cases. Re-gate → GREEN 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG ### Added; forge #102 → done. **M4 6/10.** git_diff.rs parse_diff (cov/MSI 100) + masked git-diff spawn (via marley_command) + the ⌘⇧D diff overlay. Added marley_command dep. Gate caught 2 uncovered edges → edge-case test.
