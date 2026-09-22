# The integrity-five shelf — 5 pre-authored specs (2026-07-18)

> **✅ SHELF COMPLETE (2026-07-19).** All five shipped via `/work 307,321,350,345,348` (auto-approved,
> autonomous-through-commit): #307 `8dd068c`, #321 `df00e6b`, #350 `d90ff9c`, #345 `1e4c092`, #348 (final).
> Each GATE GREEN [diff] 15/15, LOCAL (push un-OK'd). Two spec corrections surfaced at promotion (the
> confident sentences): #345's "hard-ordered after #348" was FALSE (nextest needs no config file; it ran
> first), and #348 is NOT gate-is-test (it carries a real `.rs` `reap_step` seam). #348's inspect caught a
> real edge-triggered-poll leak; its gate caught four reds (best fixed by REMOVING the `unsafe`, not
> exempting it).

The next batch after the IDE-MVP shelf: **one B-a straggler + the four integrity/correctness bugs**
ranked above further features — two of them were actively lying to the gate. Authored in batch onto
`docs/planning/pipeline/queued/` (run 7 of the pre-authored-spec method; see
[m22-editing-bar.md](m22-editing-bar.md) for the method) against `main` @ `f044546`, with every cited
seam read from live code the same day. **Promotion still re-verifies — the confident sentences are
still the dangerous ones.** Rationale for going before the B-c display-map chain: fix the measurement
instruments first; B-c is a real architecture sprint and deserves a clean, trustworthy gate under it.

## The shelf, execution-ordered

| Spec | Ticket | One line |
|---|---|---|
| `348-pty-teardown-bound` | #348 | The 11-hour zombie gate: nextest terminate ceiling (`.config/nextest.toml`) + a bounded HUP→KILL PTY teardown (the unbounded wait lives in alacritty's `Pty::Drop`, not our code). **SHIPPED — the 11-hour test now passes in 33ms; a real `.rs` fix [`reap_step` seam + `Option`/`mem::forget` executor, no `unsafe`], NOT gate-is-test** |
| `345-mutation-timeout-truth` | #345 | gate:5 stops laundering: `cargo mutants --test-tool=nextest` (fail-fast) + a Timeout-log audit; **stands alone — #348 is additive synergy, not a dependency** (nextest needs no `.config/nextest.toml`; gate:3 proves it daily). SHIPPED `d90ff9c`→(345 commit) |
| `350-structural-language-gate` | #350 | ⌃W ladder + sticky headers gate on Rust like bracket-match (the #340 M1 class, back-filled); **#351 closed as its duplicate** |
| `321-lsp-spawn-from-state` | #321 | A restored editor tab finally spawns rust-analyzer: host creation derives from open-doc STATE on the pump (the #309 reconcile it joins already exists) |
| `307-multicursor-tab-indent` | #307 | Tab/⇧Tab learn the whole cursor set — BOTH branches (touched-rows union + per-caret pad), one undo unit |

## What the authoring recon changed (the method paying again)

- **#307 — the ticket's verify contradicts its work section.** Its drive (⌘⌥↓ ×2 → Tab) produces bare
  CARETS, which take the pad branch its "The work" items never touch; a rows-only fix would fail the
  ticket's own verify. The spec covers both branches — and found a second latent bug on the way:
  `has_selection` reads only the PRIMARY's range, so a mixed set misroutes today (D2's rider).
- **#348 — the hang is not in our code.** `terminal_blocks` has zero `Drop`/threads/`waitpid`;
  `shutdown(self) {}` is empty; reads are O_NONBLOCK, writes budget-bounded. The only unbounded wait
  is delegated to `alacritty_terminal::tty::Pty::Drop` ("its Drop reaps the child", pty_os.rs:18-19)
  — so the source fix is a bounded explicit reap BEFORE Drop runs, plus the syscall-agnostic nextest
  ceiling.
- **#345 — the doctest risk is measured zero.** nextest drops doctests, so the runner swap could lose
  kill signal — but `cargo test --doc` runs **0 tests** in both fence-bearing crates (the 4 grep hits
  are mid-prose backticks + one ```text fence). Nothing to lose; drift fails CLOSED (a sole-killer
  doctest would surface as a MISSED mutant).
- **#321 — smaller than the ticket sketched.** The pump already derives doc-sync from state every
  tick (app.rs:1386-1432); the ticket's design fork (pump-reconcile vs boot-trigger) dissolves —
  host CREATION just joins the derivation that exists, consuming the same `open_files` vec. And late
  creation is already legal wire-wise: `reconcile` re-didOpens after respawn (lsp_host.rs:343).
- **#350 — written pre-#315, re-verified post.** The two ungated sites survived #315 with its own
  "pre-existing behavior" comments marking them (app.rs:3405-3407/:3481-3483); the enumeration proves
  they are the ONLY ungated production Rust-session sites (the third, app.rs:13106, is the syntax
  worker's initial session — correctly rebuilt per `req.lang`). All three touched fns are
  `mutants::skip` shims → the pinning is the #340 two-arm drive shape, not cov/MSI.

## Cross-cutting facts

- **#348 → #345 is SYNERGY, not a hard order** (corrected at #345 Phase 1, verified on `d90ff9c`): the
  "hard order" claim was FALSE. `--test-tool=nextest` needs no `.config/nextest.toml` — it is absent
  today, yet gate:3 runs `cargo nextest run --workspace` on every gate and passes without it. So #345's
  runner swap + Timeout audit STAND ALONE (the goal ran 345 before 348 successfully). #348's terminate
  ceiling is ADDITIVE: once it lands, a mutant-induced per-test hang becomes an honest nextest kill →
  `CaughtMutant`, shrinking the Timeout residue the audit covers toward the one whole-run-overrun case.
- The batch touches **no display-map surface** — B-c (display-map foundation → soft wrap →
  multibuffer, [roadmap](../../marley_architecture/roadmap.md)) starts clean after it.
- #345 is a gate-is-test change (shell + config; §7's exit-code/negative-smoke verification); **#348 is
  NOT — it carries a real `.rs` fix (the pure `reap_step` cov/MSI-100 seam + the `Option`/`mem::forget`
  `pty_os` executor) PLUS the `.config/nextest.toml`**; #350 is drive-pinned shims; #307/#321 carry
  conventional pure-seam cov/MSI surfaces.

## Standing constraints

Push discipline and live-drive rules per session state at run time; the phase-gate hooks are inert on
`queued/` — promotion (`/pipeline:plan`) mints pipeline_id + AAR, creates the local ticket doc, and
re-verifies every cited seam (line numbers go stale; symbols are the anchors). **All heavy builds +
gates run `CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0`** (the deadlock). Batch lessons:
[m22-editing-bar.md](m22-editing-bar.md).
