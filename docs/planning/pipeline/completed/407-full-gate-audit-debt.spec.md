---
pipeline_id: 1ce83652-edde-4c43-a33d-32d3f217377f
ticket: docs/planning/tickets/open/TICKET-407-full-gate-audit-debt.md
status: Phase 5 — Complete PASS
title: FULL-gate audit debt — enumerate via a complete FULL run, kill the refresh_efind_matches mutant, resolve the timeout ambiguity
type: bug
milestone: M29
references:
  - docs/planning/pipeline/completed/272-editor-find.notes.md
  - docs/planning/pipeline/completed/339-find-regex.notes.md
  - scripts/gates.sh
---

## Title
Pay down the FULL-gate audit debt surfaced at #402's /commit (2026-08-06). The
per-commit loop runs `--diff` mutation, whose soundness argument ("unchanged code
was mutation-verified in its own commit") only holds if the FULL audit actually
runs periodically — and the last FULL is RED. Three deliverables: (1) a COMPLETE
FULL enumeration run (the stopped #402-era run — 474 caught / 1 missed / 15
timeout / 83 unviable — is explicitly not exhaustive); (2) kill the one known
missed mutant (`app.rs:15712:67` `<`→`<=` in `RootView::refresh_efind_matches`,
introduced by #272) with a behavior test — the shipped `<` matches the documented
Replace-One resume contract, so the fix is a test, not a code change; (3) resolve
the timeout backlog honestly (≥6 clustered in `code_syntax.rs` hot lexer fns) —
classify each timeout as genuine detection (hang / caught-by-assert) vs false
kill (slow-but-passing), remedy any false kills, and leave the FULL audit GREEN.

## Scope
### In
- A complete FULL whole-workspace mutation enumeration at current gate defaults
  (jobs=1, current timeout config) — the measurement of record for the debt.
- A killer test for the `refresh_efind_matches` `<`→`<=` mutant (Replace One with
  a needle-containing replacement; resume lands AT the resume offset).
- Resolution of every ADDITIONAL missed mutant the enumeration surfaces: killer
  test for genuine gaps; redundancy DELETION for equivalent mutants (PR-#298
  doctrine — never suppress, §0 zero-exclusion).
- Timeout honesty: per-timeout classification from the run logs; remedy false
  kills via cargo-mutants' native knobs (`timeout_multiplier` /
  `minimum_test_timeout` in `.cargo/mutants.toml`) or code-level fail-fast in the
  `code_syntax.rs` hot loops — design decides with the logs in hand.
- A final FULL `scripts/gates.sh` run that ends GREEN (MSI 100, receipt written).

### Out (explicitly deferred)
- Any pre-existing red unrelated to gate:5 (documented "pre-existing", not fixed).
- Raising clippy lint tiers or other §0 ratchet items not in this ticket.
- TICKET-365 headed-lane font-policy verification (its own queue row).
- If the enumeration surfaces a MOUNTAIN (>~10 additional missed mutants or a
  cross-crate structural cause), fix the tractable set here and split the
  remainder into follow-up tickets with the enumeration attached — the slice
  stays shippable.

## Reference (§20)
N/A — Marley-specific, no reference-app analog. This is quality-gate
infrastructure (the §0 mutation audit and its timeout semantics), not
user-facing behavior; Warp/Zed have no observable behavior to match here. The
one user-adjacent seam this touches — the F5 Replace-One resume contract — was
reference-matched when it shipped (#272, Warp find-bar behavior per
`docs/planning/pipeline/completed/272-editor-find.notes.md`); this ticket only
PINS that already-shipped contract with a test, changing no behavior.

### Prior art
- **Permissive deps/tools (the highest-yield leg):** cargo-mutants (MIT) OWNS
  the timeout seam — verified against the installed 27.1.0: `--timeout-multiplier`
  (relative to auto-measured base test time), `--minimum-test-timeout` (floor for
  the auto-set value), `-t/--timeout` (absolute), all mirrored as
  `.cargo/mutants.toml` keys. NOTABLE LIMIT: all are GLOBAL — there is no
  per-file timeout, so "raise the per-mutant timeout for that file" (the ticket's
  first remedy sketch) is not tool-native; the real choice space is global
  multiplier/floor vs code-level fail-fast vs accept-hang-as-detection.
  `-F/--re` scopes a run to matching mutants (e.g. `--re refresh_efind_matches`)
  — targeted kill-verification in minutes, adopted for REQ verification.
- **Published material:** the cargo-mutants book (mutants.rs) documents timeout
  semantics (auto timeout = multiplier × baseline; Timeout is a distinct outcome
  from Caught/Missed) and exit codes (0 clean / 2 missed / 3 timeout — already
  encoded in `mutation_g`). Stryker/Infection MSI convention (timeout counts as
  killed — "a hang IS detection") is the documented basis of gate:5's arithmetic,
  cited inline in `scripts/gates.sh` with the #345 mislabel audit on top.
- **Behavior maps:** checked `docs/warp_architecture/` — nothing on mutation
  tooling (as expected; gate infra is Marley-specific). The Replace-One resume
  behavior itself is already pinned in the #272/#339 completed-pipeline notes.

## React-first (parity)
N/A — no UI delta: this ticket adds tests and gate/timeout configuration only.
No shell chrome, overlay, pane surface, layout, type, color, or affordance
changes; the F5 Replace-One behavior being pinned is ALREADY shipped and stays
byte-identical. Nothing to build or verify in marley-web.

## Locked-In Decisions
- D1 — **Enumerate before fixing** (ticket-locked). The FIRST act is a complete
  FULL run at the gate's own defaults (jobs=1, current timeout config) so the
  measurement reflects the gate as it stands. Fix scope is finalized only after
  `mutants.out/outcomes.json` is complete.
- D2 — **The efind fix is a test, not a code change** (ticket-locked). The
  shipped `<` in `matches.partition_point(|&(ms, _)| ms < resume)` implements the
  documented contract ("the current match is the first AT/after the resume
  point"); the `<=` mutant breaks exactly the needle-containing-replacement case.
  Drive Replace One so the replacement contains the needle and assert efind lands
  on the match starting AT the resume offset.
- D3 — **Equivalent-mutant doctrine** (recall-locked, PR-#298): a surviving
  mutant no test can kill because behavior doesn't change is a DESIGN SMELL —
  delete the redundancy so the operator goes with it; re-run mutants to confirm
  the hole moved nowhere. Never `#[mutants::skip]` a decision-bearing fn, never
  carve an exclusion (§0).
- D4 — **Timeout = CAUGHT is the standing convention, not the bug** (gate-source
  fact). `mutation_g` counts Timeout as caught (Stryker convention) and the #345
  audit already flags caught-by-assert mislabels. The deliverable is HONESTY:
  classify every timeout in the enumeration (hang = genuine kill /
  caught-by-assert = #345 mislabel / slow-but-passing = FALSE kill), remedy only
  the false kills, and record the classification. The convention itself is
  unchanged.
- D5 — **No concurrent heavy cargo work while a FULL mutation run is live**
  (recall-locked: the 2026-08-07 dual-build-chain freezes). Design/doc phases may
  overlap the enumeration; implement waits for it (it needs the results anyway).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method (gate exit code,
negative smoke, or review).

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the FULL gate runs on the pre-fix workspace, the pipeline shall capture a COMPLETE mutation enumeration (run not stopped early; every viable mutant examined). | `mutants.out/outcomes.json` exists; cargo-mutants completed with exit ∈ {0,2,3}; caught/missed/timeout/unviable counts recorded in the notes Phase-1/2 entries. |
| REQ-002 | WHEN Replace One (F5) runs with a replacement text that CONTAINS the needle, the system shall place the next-match index on the match starting AT the resume offset (not the one after it). | A test that fails under the `<`→`<=` mutant: `cargo mutants -f crates/marley_app/src/app.rs --re refresh_efind_matches` reports 0 missed; sabotage smoke (hand-flip `<`→`<=` → test RED → revert → green). |
| REQ-003 | WHEN the enumeration surfaces additional missed mutants, each shall be resolved at source — a killer test for a genuine behavior gap, or deletion of the redundancy for an equivalent mutant (D3). | Per-item targeted re-run (`cargo mutants -f <file> [--re <fn>]`) shows 0 missed; each item + remedy class logged in the notes fix ledger. |
| REQ-004 | WHEN each timeout from the enumeration is classified, the audit record shall state its class (hang / caught-by-assert / slow-but-passing) with log evidence, and every slow-but-passing FALSE kill shall be remedied (timeout config or fail-fast) such that the final FULL run repeats none of them. | Classification table in the notes citing `mutants.out/log/*` lines; final run's timeout set ⊆ {hang, caught-by-assert}. |
| REQ-005 | WHEN the fixes land, a final FULL `scripts/gates.sh` run shall end GREEN — gate:5 MSI 100 with zero missed mutants — and write the commit receipt. | `scripts/gates.sh` (FULL) exit 0; `.git/ignibyte-gate-receipt` present; the gate:5 summary line recorded in the notes. |

## Phase Plan
- **P2 Design** — with the enumeration running in the background: design the
  Replace-One killer test (harness: the `headless_drive.rs` REQ-flow family from
  #272/#339 — `refresh_efind_matches` is a RootView method), the timeout
  classification procedure (log forensics per class), and the remedy decision
  tree (mutants.toml global knobs vs fail-fast vs none-needed, given the
  per-file limit found in prior art). When the enumeration completes, append the
  DESIGN ADDENDUM: the definitive fix ledger (every missed mutant → remedy
  class; every timeout → classification).
- **P3 Implement** — write the killer test(s), any D3 redundancy deletions, and
  the chosen timeout remedy. No production behavior changes expected (D2); any
  exception is a design deviation to justify in the notes.
- **P3.5 Inspect** — independent critics vs the diff; fix the real findings.
  Mandatory angles: sabotage-verify REQ-002 (flip the operator, expect red); the
  "MUST…otherwise" rule (run the bad thing, don't trust the comment); receipt
  fingerprint coverage if `.cargo/mutants.toml` is introduced (it becomes a
  gate-defining file — §15).
- **P4 Validate** — RUN the targeted mutant verifications (REQ-002/003) + the
  static gates; then the final FULL audit (REQ-005, hours) → green receipt.
- **P5 Complete** — archive, ledger capture (§19: the timeout classification and
  audit-debt lessons), CHANGELOG (.rs test code in the changeset), architecture
  docs if the timeout policy changed, close the ticket (row already removed at
  promotion).
