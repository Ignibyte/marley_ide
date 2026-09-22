---
pipeline_id: 84741d7a-84f3-42ba-92ac-9d264d4b8898
ticket: forge#3 (368f2d21-74ec-4e21-bee1-47dcc09984e4) · local docs/planning/tickets/open/TICKET-doc-phase.md
aar_id: 0c98d79e-be52-4aa9-a95f-743d5becb770
status: Phase 5 — Complete PASS
title: Strict documentation phase — CHANGELOG + architecture-doc enforcement
type: chore (infra; gate-is-test)
milestone: M0
references:
  - ../../../../CONSTITUTION.md
  - ../../../../.claude/commands/pipeline/complete.md
  - ../../../../.claude/hooks/enforce-commit-gate.sh
---

## Title

Add a **strict, unskippable documentation phase** to the Marley pipeline so
context is never lost: a `CHANGELOG.md`, a hook that **blocks a code commit
lacking a CHANGELOG entry**, a STRICT `pipeline:complete` doc step, a CONSTITUTION
clause, and a **phase-skip-resistance audit** of the enforcement layer. Gate-is-test
(config + hook + docs + the new `CHANGELOG.md`; **zero `crates/*/src`**), so it
commits via the no-`.rs` path (§15). Mirrors the commit-gate's
"block-on-a-changeset-condition" pattern (forge `AD-claude-receipt-scope-001`).

## Scope
### In
1. `CHANGELOG.md` at the repo root, **Keep a Changelog** format (`[Unreleased]` +
   a dated `0.0.0` section), backfilled with TICKET-000 (pipeline → 16-gate Marley
   bar) and TICKET-001 (`marley_text_offsets`).
2. `.claude/hooks/enforce-changelog.sh` — PreToolUse(Bash), structured like
   `enforce-commit-gate.sh`: detect a real `git commit`; if the changeset touches
   `crates/*/src/**/*.rs` but `CHANGELOG.md` is **not** in it → **block (exit 2)**;
   a no-`.rs` change is exempt. Wired into `.claude/settings.json` PreToolUse(Bash).
3. `pipeline:complete.md` step 1 → **STRICT/REQUIRED**: every ticket MUST add a
   CHANGELOG entry **and** update the relevant `docs/marley_architecture/` doc(s) —
   not "if behavior changed". A TaskCreate item enforces it in-phase.
4. `CONSTITUTION.md` — a new binding **§21 — Documentation Phase**, cited from §3
   (pipeline) and the §0/§15 enforcement summary.
5. **Phase-skip-resistance audit** — record (and fix any real hole in) whether
   `enforce-phase-gate` / `-phase-tasks` / `-pipeline-completion` / `-tests-ran` /
   `-commit-gate` genuinely bite.

### Out
- No `crates/*/src` changes. No new gate in `scripts/gates.sh` (the enforcement is
  a commit-time hook, like the receipt — keeping the gate's 16 numbered set stable).
- Defeating deliberate *fabrication* of a phase status (the §15-disclosed limit) —
  out of scope; the hooks catch omissions, the commit receipt + changelog hook are
  the hard, evidence-based checks.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — Commit-time enforcement, mirroring the receipt.** The CHANGELOG check is a
  PreToolUse(Bash) hook keyed on the same `.rs`-in-changeset trigger as
  `enforce-commit-gate`, so a code commit and its CHANGELOG entry are atomic and a
  no-`.rs` infra/docs change is exempt. (Not a `scripts/gates.sh` gate — that set
  stays the 16-gate quality bar.)
- **D2 — CHANGELOG is the HARD-forced half; arch-docs are the strict-process half.**
  "CHANGELOG.md touched on a code commit" is mechanically enforceable; "the right
  architecture doc was updated" is judgment, so it's a required `complete`-phase
  step verified at inspect/complete, not a hook predicate.
- **D3 — Backfill 000/001** so the CHANGELOG starts truthful, not empty.

## Acceptance Criteria (EARS)
Verify: **G** gate/hook exit code · **S** negative smoke · **R** review.

| # | EARS requirement | Verify |
|---|---|---|
| REQ-001 | The repo shall contain a root `CHANGELOG.md` in Keep-a-Changelog form with entries for TICKET-000 and TICKET-001. | R |
| REQ-002 | WHEN a `git commit` changeset includes `crates/*/src/**/*.rs` AND `CHANGELOG.md` is **not** in the changeset, `enforce-changelog.sh` shall block the commit (exit 2) with a remediation message. | S |
| REQ-003 | WHEN such a code changeset **does** include a `CHANGELOG.md` change, the hook shall exit 0 (allow). | S |
| REQ-004 | WHEN a changeset contains **no** `crates/*/src/*.rs` (config/docs/tooling), `enforce-changelog.sh` shall exit 0 (exempt — this ticket itself). | G |
| REQ-005 | `enforce-changelog.sh` shall be wired in `.claude/settings.json` PreToolUse(Bash) alongside `enforce-commit-gate.sh`, and shall be shellcheck-clean (gate:11). | R + G |
| REQ-006 | `pipeline:complete.md` shall REQUIRE (not "if") a CHANGELOG entry + an architecture-doc update each ticket, with a matching TaskCreate checklist item. | R |
| REQ-007 | `CONSTITUTION.md` shall carry a binding documentation-phase § (§21), cited from §3 and the enforcement summary. | R |
| REQ-008 | The notes shall record a phase-skip-resistance audit verdict for each of the 5 wired hooks; any real (non-§15-disclosed) hole shall be fixed at source. | R |
| REQ-009 | `scripts/gates.sh --fast` shall print `GATE GREEN [fast]` (the new hook is shellcheck-clean; no `.rs`). | G |

## Phase Plan
- **P2 Design** — exact `enforce-changelog.sh` algorithm (mirror commit-gate's git-detection + `.rs`-trigger), the CHANGELOG layout, the complete.md/CONSTITUTION edits, and the negative-smoke plan.
- **P3 Implement** — write the hook + CHANGELOG + complete.md/CONSTITUTION + settings wiring.
- **P3.5 Inspect** — critics: hook correctness/bypass (can a code commit dodge the CHANGELOG check? git-detection spellings?) + doc consistency; the skip-audit.
- **P4 Validate** — `gates.sh --fast` green + negative smokes (code-without-CHANGELOG blocked → add → allowed; no-`.rs` exempt).
- **P5 Complete** — CHANGELOG entry for THIS ticket (dogfood the new rule), arch-doc note, archive, AAR.
- **/commit** — no `.rs` → commit-gate + the new changelog hook both exit 0.
