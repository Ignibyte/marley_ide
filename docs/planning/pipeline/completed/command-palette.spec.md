---
pipeline_id: f142d14f-1d43-4409-9965-541d5f02896b
ticket: forge#19 (67473f5f-4aaf-40a4-9c9e-8bc7236f8ef1) · local docs/planning/tickets/open/TICKET-019-command-palette.md
aar_id: 05ba6945-9bcc-41a7-bcbf-b7c49674f49a
sprint: M1.B — The Cockpit (cbc92bf0) seq 3/5
status: Phase 5 — Complete PASS
title: command palette — fuzzy launcher overlay (filter_commands + the overlay)
type: feature
milestone: M1
references:
  - ../../../specs/SPEC-app-shell.spec.md
  - ../../../marley_architecture/app_shell.md
---

## Title

TICKET-019 — the [`SPEC-app-shell`](../../../specs/SPEC-app-shell.spec.md) command palette (R14–R18) in
`marley_app`: a PURE `filter_commands` (nucleo subsequence ranking over a static `&[Command]`) + the
`Command`/`CommandId`/`ScoredCommand` model, plus the centered overlay in `RootView` (open on the #18
keymap's `open-command-palette`, filter-as-you-type, activate→dispatch+dismiss, Escape→dismiss). M1.B
"The Cockpit" seq 3/5 — the fuzzy launcher, on #17's widgets + #18's keymap.

## Scope

### In
- **`palette.rs` (PURE)** — `Command { id: CommandId, title: String, keywords: Vec<String>, binding:
  Option<KeyBinding> }`, `CommandId(u32)`, `ScoredCommand<'a> { command: &'a Command, score: u32 }`,
  `filter_commands<'a>(&'a [Command], query) -> Vec<ScoredCommand<'a>>`:
  - **R15** — empty query → one `ScoredCommand` per command, in registration order.
  - **R16** — non-empty → rank `title` + each keyword via the nucleo subsequence matcher; include a
    command iff ≥1 field matches (query chars in order, case-insensitive); `score` = MAX nucleo score
    across matching fields; order DESCENDING by score; ties broken by ASCENDING registration order.
- **`app.rs` (SHIM)** — the palette overlay state + render on `RootView`: open on the keymap's
  `open-command-palette` action (R14 — centered, empty query); filter-as-you-type; activate → dispatch
  the command's bound action + dismiss (R17); Escape → dismiss with NO dispatch (R18).
- `nucleo` dep (v0.5.0). §21 — CHANGELOG + app_shell.md.

### Out / deferred
- `marley_search_core` / `SearchMixer` → M2 (the spec's isolation note — M1 is the LOCAL static-list
  wrapper; `ScoredCommand.score: u32` is M1-local, NOT a second permanent ranking engine).
- Rich widget composition of the overlay — a functional centered overlay for M1.B; the #17 widgets are
  available; polish is later.

## Acceptance Criteria (EARS — adopt SPEC-app-shell R14–R18)
- **AC-R15** — `filter_commands(cmds, "")` returns one `ScoredCommand` per command in registration order.
  Verify: `filter_empty_query_returns_all_in_reg_order`.
- **AC-R16** — non-empty query: nucleo-ranked, non-matches excluded, `score` = max nucleo field score,
  sorted descending, ties by ascending reg order. Verify:
  `filter_ranks_nucleo_matches_and_breaks_ties_by_reg_order` (assert membership + ORDERING, not exact
  internal scores).
- **AC-R14/R17/R18 (shim)** — the overlay opens centered on the action, activates→dispatch+dismiss,
  Escape→dismiss. Verify: the `#[ignore]` headed palette-open assertion (the window + a non-blank overlay
  region); the pure dispatch/dismiss decisions ride on `filter_commands` + the keymap.
- **AC-gate** — `palette.rs` (the model + `filter_commands`) cov 100 / MSI 100; the app.rs overlay shim
  ACCEPTED-UNTESTABLE (already `mutants::skip` + rust_cov-excluded); FULL `scripts/gates.sh` →
  `GATE GREEN [diff]`.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
1. **The pure/shim seam** — `filter_commands` + the model are PURE (nucleo is a deterministic dep; no
   gpui, no IO → cov 100 / MSI 100). The overlay STATE (open/closed, the query buffer, the selection) +
   the Render + the activate/Escape dispatch are SHIM in app.rs (already excluded).
2. **nucleo (v0.5.0)** — `filter_commands` creates a `nucleo::Matcher` internally + `fuzzy_match`es each
   field; the exact scores are internal (tests assert ORDERING + membership, never a literal nucleo
   score). Case-insensitivity per the nucleo `Config` (verify at implement — lowercase or the
   ignore-case config so R16's "case-insensitive" holds).
3. **The explicit reg-order tie-break** — keep `(idx, ScoredCommand)` through the sort +
   `.then(a.idx.cmp(&b.idx))` so the "drop the tie-break" mutant is killable (a stable sort alone would
   pass R16 but leave the tie-break mutant equivalent).
4. **The MIN-vs-MAX mutant** needs a fixture with a command having 2 MATCHING fields of different nucleo
   scores + a comparison command whose single-field score sits between them — construct EMPIRICALLY at
   validate (run `filter_commands`, read the scores, build the ordering assertion).

## Phase Plan
- **P2 Design** — the `filter_commands` body + the nucleo `field_score` helper + the model + the mutation
  map (empty-short-circuit, match-predicate, max-not-min, sort-flip, tie-break) + the overlay shim shape.
- **P3 Implement** — palette.rs + nucleo + the app.rs overlay (write-then-check nucleo's API).
- **P3.5 Inspect** — the mutant list (only palette.rs; app.rs excluded) + the max-vs-min fixture design.
- **P4 Validate** — the 2 pure tests (empirical nucleo fixtures) + the #[ignore] headed palette + gate.
- **P5 Complete** — §21; close #19; → #20 layout.
