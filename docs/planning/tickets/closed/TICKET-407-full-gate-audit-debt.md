# TICKET-407 — FULL-gate audit debt: a surviving mutant in refresh_efind_matches (M17 #272) + the timeout backlog

- **Ticket:** LOCAL #407 (bug, M29)
- **Tags:** M29, gate, mutation, tech-debt, full-audit, 272-followup
- **Created:** 2026-08-06
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id 7f3d1cfe-b36d-4d9e-a595-5800ea8787cd)
- **Status:** closed (2026-08-14 — shipped; GATE GREEN [full], 5,834/5,834, MSI 100.0%)

## Closed — 2026-08-14

Every deliverable landed, most beyond the ticket's ask: REQ-001 the complete
enumeration (5,830 → re-measured 5,834 at close); REQ-002 verify-only (#404's
killer confirmed by enumeration AND a live sabotage smoke); REQ-003 the 16
survivors killed (12b78c8) + scoped 0-missed; REQ-004 all 54 timeouts
classified (39 hang / 15 caught-by-assert / **0 false kills** — the standing
convention holds with evidence); REQ-005 a MEASURED `GATE GREEN [full]` via
the new gate:5 import lane (see
`AD-claude-407-full-mutation-is-an-imported-verdict-behind-artifact-belts-001`)
over the three-generation merged record, 4,832 caught / 0 missed.

**Runner rider list for Chad (accumulated; the runner is owned outside this
repo):** (1) drop the `-E` exclusion filter + `NEXTEST_RETRIES=1` (dead
weight since #423 — verified); (2) write `MEASURED_SHA` (+ optionally
`MEASURED_ENV`: retries/filter) into `mutants.out/` so the gate's sidecar
belt becomes authoritative instead of pull-recipe-supplied; (3) CLOEXEC on
the lock fd + a post-run orphan reap (the zsh-orphan/70-min-stale-RUNNING
discovery); (4) OOMPolicy=continue + the restore-recipe correction are
already in (08-13).

## Description

Surfaced by the FULL (whole-workspace) gate run at #402's /commit on 2026-08-06 — PRE-EXISTING, unrelated to #402 (whose diff never touches app.rs).

THE MISSED MUTANT: `crates/marley_app/src/app.rs:15712:67: replace < with <= in RootView::refresh_efind_matches` — SURVIVES, so gate:5 MSI < 100 and the FULL audit is RED. Introduced by commit 1150385 (M17 #272, the ⌘F find & replace bar); it has ridden every subsequent commit because the per-commit loop runs `--diff` (mutation only on touched lines) and nothing has touched that line since.

The code: `Some(resume) => matches.partition_point(|&(ms, _)| ms < resume)` — the F5 Replace-One resume seam, whose comment states the contract as "the current match is the first AT/after the resume point … so a needle-containing replacement advances". `<` gives first-with-ms >= resume (the match AT resume); the `<=` mutant gives first-with-ms > resume (skipping it). The two differ EXACTLY when a match starts at the resume offset — which is precisely the needle-containing-replacement case the comment says the idiom exists to serve. So this is a genuine untested behavior, not an equivalent mutant.

THE FIX is a test, not a code change (the shipped `<` matches the documented contract): drive Replace One with a replacement text that CONTAINS the needle (e.g. find "ab" → replace with "xaby"), then assert efind_index lands on the match starting AT the resume offset rather than the one after it. Check the existing #272/#339 find suites for the closest harness; a headless drive may be the natural home since refresh_efind_matches is a RootView method (app.rs is coverage-excluded but NOT mutation-excluded — every fn there is either mutants::skip-justified or must be killable; this one is neither).

ALSO IN SCOPE — the timeout backlog: the same partial run logged 15 timeouts before it was stopped, at least 6 of them clustered in `crates/marley_app/src/code_syntax.rs` (is_ident_start / is_ident_continue / highlight_line). Timeouts are not kills; confirm whether the gate's MSI arithmetic counts them, and either raise the per-mutant timeout for that file or make the hot loops fail fast so the audit reports honestly rather than ambiguously.

NOTE ON SCOPE: the run was stopped early (a single missed mutant already determines RED, and the whole-workspace sweep takes hours), so this list is NOT exhaustive — 474 caught / 1 missed / 15 timeout / 83 unviable at stop time. FIRST STEP of this ticket is a complete FULL run to enumerate the true backlog before fixing anything. That also answers the standing question of how much audit debt the `--diff` per-commit loop has accumulated since the last green FULL — §0 calls FULL "the periodic audit" precisely because --diff's soundness argument ("the unchanged code was mutation-verified in its own commit") only holds if the audit actually runs periodically.
