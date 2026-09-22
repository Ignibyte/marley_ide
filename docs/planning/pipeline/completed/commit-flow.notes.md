# the commit flow (git WRITE) — Notes

- **Forge ticket:** #116 `bc1a1312-eb69-4e08-9b9c-8a6c505e6459` · **AAR:** `9c401f8b-623e-4f36-92e5-73ff0700051b`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-116-commit-flow.md

## Phase 1 — Plan
- **Request:** forge #116 (M5 10/12) — stage + commit from the git panel. **Marley's FIRST git write.**
- **Pre-flight:** #115 git panel + git_status (staged flags); #112 session-search = the input pattern;
  marley_command::blocking = the spawn seam.
- **Decisions:** D1 commit_enabled gate; D2 **SECURITY — add/restore/commit only, argv message, no push/
  force/rewrite**; D3 message input mirrors #112.
- **AAR id:** `9c401f8b-623e-4f36-92e5-73ff0700051b`.

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
- **git_diff.rs (PURE):** `pub fn commit_enabled(staged_count: usize, message: &str) -> bool` = `staged_count >= 1 && !message.trim().is_empty()`.
- **app.rs SHIM (git WRITE — confined):** 3 masked adapters via marley_command::blocking (cwd project_root, each arg a SINGLE argv value — NO shell): git_stage(path)=`git add -- <path>`; git_unstage(path)=`git restore --staged -- <path>`; git_commit(msg)=`git commit -m <msg>` (msg a single arg). Each returns Ok/Err → a status_flash. **NO push/-f/--force/rebase/reset/amend.** Fields commit_message:String + commit_focused:bool (empty/false in new()). handle_commit_message_key (esc→unfocus / backspace→pop / 1-char→push, mirrors #112) routed early in on_key_down when commit_focused. The #115 git panel row: split the click — the staged dot ● on_mouse_down → toggle (row.staged?git_unstage:git_stage)(row.path)+flash; the path on_mouse_down → the diff (#115). Below the list: a commit-message input box (click→commit_focused; shows commit_message/"Message…") + a "Commit" button colored by commit_enabled(staged_count,&commit_message) [staged_count = changes.iter().filter(|c|c.staged).count()]; click WHEN enabled → git_commit(&commit_message)+commit_message.clear()+flash.
- **Mutation targets:** commit_enabled the >=1, the !trim().is_empty(), the &&.
- **Test plan:** commit_enabled_cases ((0,"m")→false; (1,"m")→true; (1,"")→false; (1,"   ")→false; (2,"x")→true). cov/MSI 100. The write adapters + UI masked (env-blocked; code-reviewed for confinement).
- **SECURITY:** the FIRST git write — the adapters are the ONLY write path, confined to add/restore/commit; the message/path are argv args (marley_command, no shell) → no injection; no push/force/history-rewrite; a flash surfaces the result. VALIDATE must NOT auto-commit.

## Phase 3 — Implement
- **Built:** commit_enabled (git_diff.rs, cov/MSI 100); app.rs commit_message+commit_focused fields; the 3 confined write adapters git_stage(add --)/git_unstage(restore --staged --)/git_commit(commit -m) via marley_command (argv args, ok=status.success()); handle_commit_message_key + on_key_down routing; the git panel row split (● toggles staged / path opens diff) + a commit-message input + a Commit button gated on commit_enabled (re-checked at click against the fresh staged set).
- **Verification:** fmt; check 0 err; clippy OK; commit_enabled test.

## Phase 3.5 — Inspect (SECURITY-focused — the first git write)
- **Method:** rigorous self-review of the write path + a grep audit for forbidden subcommands.
- **SECURITY findings — NONE:** the ONLY git-write functions are git_stage/git_unstage/git_commit (grep-confirmed); their args are literal `add`/`restore`/`--staged`/`commit`/`-m`/`--` + the path/message as SINGLE argv values via marley_command::blocking (NO shell string) → a path/message like `; rm -rf /` or `--force` is an inert argument, not interpreted (the `--` before the path also stops option parsing). Grep audit: NO push/force/-f/rebase/reset/clean/checkout in any .arg() (confirmed). ok = status.success() (the real git result) → an accurate flash. The Commit button re-checks commit_enabled at click against a FRESH git_status (not the stale render count) before writing. No unwrap/panic.
- **Correctness findings — NONE:** commit_enabled = staged_count>=1 && !message.trim().is_empty() (tested 0/1/2 staged × empty/blank/text); the ● stage-toggle reads the fresh state on the next render (git_status re-run each render); the message input mirrors the proven #112 routing.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** commit_enabled_cases (0/1/2 staged × empty/blank/text → the gate). `cargo nextest` → pass; clippy OK.
- **Self-test:** the commit flow (stage-toggle click, message typing, Commit click) needs synthetic input — ENV-BLOCKED; NOT auto-committing during validation. commit_enabled engine-tested cov/MSI 100; the git-write adapter is masked + SECURITY-reviewed (add/restore/commit only, argv args, no push/force/rewrite — grep-audited).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (first try).

## Phase 5 — Complete
- CHANGELOG ### Added; forge #116 → done. **M5 10/12.** commit_enabled (cov/MSI 100) + the CONFINED git-write adapter (add/restore/commit only) + the commit UI. Marley's FIRST git write. AD recorded for the confinement.
