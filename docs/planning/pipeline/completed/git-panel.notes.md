# the git changes panel (source control) — Notes

- **Forge ticket:** #115 `a3897c5f-fecf-48fb-a199-8f1327434739` · **AAR:** `18d512ca-f7d2-4770-b47e-54d4af83e9b0`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-115-git-panel.md

## Phase 1 — Plan
- **Request:** forge #115 (M5 9/12) — a ⌘⇧C source-control panel (change list + empty state).
- **Pre-flight:** #102 git_working_diff = the masked git-spawn pattern; ⌘⇧C free but the copy handler (1627)
  ignores shift → my handler goes BEFORE it + checks shift + returns; #114 viewer_split + the right panel.
- **Decisions:** D1 XY-column parse; D2 ⌘⇧C-before-copy; D3 git panel takes the right region (else viewer).
- **AAR id:** `18d512ca-f7d2-4770-b47e-54d4af83e9b0`.

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
- **git_diff.rs (PURE):** ChangeStatus{Modified,Added,Deleted,Renamed,Untracked}(Copy); ChangeRow{path,status,staged}; `parse_status(porcelain)`: lines, skip len<4; x=bytes[0],y=bytes[1],path=line[3..]; if x==?&&y==? → Untracked,false; else staged=(x!=' '),code=(staged?x:y),match code A→Added/D→Deleted/R→Renamed/_→Modified; `change_summary(rows)` = empty→"no changes" / "N changed".
- **app.rs SHIM:** `git_panel_open: bool` field (false); `#[mutants::skip] fn git_status(&self)->Vec<ChangeRow>` (marley_command::blocking git status --porcelain, cwd project_root → parse_status, Err→[]); ⌘⇧C handler BEFORE the copy handler (1627): `if platform && shift && key=="c" { git_panel_open=!git_panel_open; notify; return; }`; the right region: `let right = self.git_panel_open || self.code_view.is_some(); let (term_w,viewer_w)= if right {viewer_split(regions.center)} else {(regions.center,0.0)};` render — if git_panel_open → the git panel (Source Control title + × + change_summary + a ChangeRow list [status letter M/A/D/R/? colored staged=success else danger + path, click→self.diff=git_working_diff] OR "No open changes"); else if code_view → the viewer (#114).
- **Mutation targets:** parse_status len<4/untracked/staged/code/status arms; change_summary empty+count.
- **Test plan:** parse_status_cases (" M f"→Mod-unstaged; "M  f"→Mod-staged; "A  f"→Added-staged; " D f"→Del-unstaged; "R  a"→Renamed-staged; "?? f"→Untracked; "x"→skip; ""→[]); change_summary_cases (empty→"no changes"; 2→"2 changed"). cov/MSI 100.
- **Risks:** ⌘⇧C must precede the shift-ignoring copy handler; the git panel takes right-region precedence over the viewer; the spawn is READ-ONLY (git status).

## Phase 3 — Implement
- **Built:** ChangeStatus/ChangeRow + parse_status + change_summary (git_diff.rs, cov/MSI 100 tested); app.rs git_panel_open field + git_status masked spawn (git status --porcelain via marley_command) + a ⌘⇧C toggle placed BEFORE the shift-ignoring copy handler + the right-region logic (right_open = git_panel_open||code_view; viewer_split width) + the git panel render (Source Control title + change_summary + × close + a ChangeRow list [status letter colored + a staged/unstaged dot + path, click→the working diff] OR "No open changes"); the viewer render is guarded by !git_panel_open (git panel takes precedence).
- **Verification:** fmt; check 0 err; clippy OK; parse_status + change_summary tests pass.

## Phase 3.5 — Inspect
- **Method:** self-review (a porcelain parser reusing the #102 spawn pattern + a masked panel; gate cargo-mutants the MSI authority).
- **Lenses — no findings:** parse_status skips len<4, reads x=col0/y=col1/path=[3..] (ASCII XY cols → char-boundary safe), ?? → Untracked, else staged=(x≠space) + code=(staged?x:y) + status by code (A/D/R/_→Modified) — tested unstaged/staged/added/deleted/renamed/untracked/short/empty; change_summary empty vs count; the git_status spawn is READ-ONLY (git status --porcelain via marley_command, Err→[]); ⌘⇧C is intercepted BEFORE the copy handler (which ignores shift) + returns; the git panel takes right-region precedence over the viewer (viewer guarded by !git_panel_open); no unwrap/panic (from_utf8_lossy). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** parse_status_cases (unstaged/staged/added/deleted/renamed/untracked/short-skip/empty) + change_summary_cases. `cargo nextest` → pass; clippy OK.
- **Self-test:** the git panel opens on ⌘⇧C (a synthetic chord — ENV-BLOCKED, like the ⌘⇧D diff overlay). parse_status/change_summary engine-tested cov/MSI 100; the panel render + ⌘⇧C routing masked + code-reviewed. On this clean repo it would show "No open changes".
- **Gate finding + fix:** first run RED on MSI — the untracked `x==? && y==?` left a `- **Gate:** (running).- **Gate:** (running).`→`||` mutant (my `??` test had BOTH ?, so it did not distinguish). Added parse_status_untracked_needs_both_question_marks ("?M" single-? → Modified not Untracked). Re-gate → GREEN 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG ### Added; forge #115 → done. **M5 9/12.** parse_status + change_summary (cov/MSI 100) + the ⌘⇧C git panel (masked git status --porcelain + the change list / "No open changes"). Gate caught the untracked &&→|| mutant → single-? test.
