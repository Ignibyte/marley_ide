---
pipeline_id: 298e473d-e910-4ff4-8069-58fb20dd2d5f
ticket: forge#248 (d369915f-4728-4836-9f18-730bfc47dfb8) · local docs/planning/tickets/open/TICKET-248-force-warp-reference.md
aar_id: e2221e1e-b048-40cc-a84c-7540722ef1c4
status: Phase 5 — Complete PASS
title: Force Warp-reference into the pipeline (§20 discipline)
type: chore
milestone: M15
references: [CONSTITUTION §20, forge#237]
---

## Title
Make "how does Warp do this?" a FORCED, enforced per-ticket pipeline step — a required `## Warp Reference (§20)`
spec section, plan-filled + design-confirmed, backed by a commit hook — so Warp-matching is disciplined, not
ad-hoc. The first ticket of the M15 editor train; the rest inherit it.

## Scope
### In
- **The spec template** (`docs/planning/pipeline/_templates/pipeline.spec.md`) gains a `## Warp Reference (§20)`
  section with inline 3-option guidance.
- **The plan skill** (`.claude/commands/pipeline/plan.md`) requires filling it; **the design skill**
  (`.claude/commands/pipeline/design.md`) requires confirming it + stating how the design matches (the §20 wall).
- **A new commit hook** `.claude/hooks/enforce-warp-reference.sh` blocks a `git commit` that stages a pipeline
  spec whose `## Warp Reference` section is missing/empty — registered in `.claude/settings.json`.
- **CONSTITUTION §20** gains a bullet codifying the forced section.
- **A durable `docs/warp_architecture/observed/`** dir (with a README) for Warp observation captures (not the
  ephemeral scratchpad).

### Out (explicitly deferred)
- Actually capturing the Warp editor/input observations — that happens per-ticket in M15 (#249+), not here.
- Any `.rs` / product-code change. This is pipeline/process infrastructure only.
- Retro-filling `## Warp Reference` into already-completed specs (the hook gates specs staged going forward).

## Reference (§20)
**N/A — Marley-specific.** This is a pipeline-process/governance change (how *Marley's* clean-room workflow is
enforced); Warp has no analog to observe. This spec is also the FIRST to carry this section — it dogfoods the
very artifact it introduces.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1** — the section is `## Warp Reference (§20)`, placed after `## Scope` and before `## Locked-In Decisions`
  (it informs the decisions). One of three contents: (a) the Warp behavior matched (cite `docs/warp_architecture/
  …`), (b) an observed capture in `docs/warp_architecture/observed/`, (c) `N/A — Marley-specific + why`.
- **D2** — enforce at COMMIT (mirror `enforce-changelog.sh`): a PreToolUse:Bash hook intercepts `git commit`,
  and if the staged set includes any `docs/planning/pipeline/**/*.spec.md` whose `## Warp Reference` section is
  absent or empty (only whitespace / a bare `…`/`TODO` placeholder), it exits 2 (block). No staged spec → exempt.
- **D3** — the hook is ADVISORY-STRICT: it gates the section's PRESENCE + non-emptiness, NOT its correctness
  (a human judges whether the Warp match is right — the §18.1 inspect provenance check + design review do that).
- **D4** — dogfood: this spec carries the section (D-Warp-Reference above), so the hook passes on its own commit.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The spec template shall contain a `## Warp Reference (§20)` section with the 3-option guidance. | file check |
| REQ-002 | `enforce-warp-reference.sh` shall exit non-zero (block) when a `git commit` stages a pipeline spec whose `## Warp Reference` section is missing or empty, and exit 0 when it is present + non-empty (or no spec is staged). | negative + positive smoke |
| REQ-003 | The hook shall be registered in `.claude/settings.json` (PreToolUse:Bash, alongside enforce-changelog). | grep |
| REQ-004 | The plan skill shall require filling `## Warp Reference`; the design skill shall require confirming it + stating the match. | doc check |
| REQ-005 | CONSTITUTION §20 shall codify the forced `## Warp Reference` section. | doc check |
| REQ-006 | `docs/warp_architecture/observed/` shall exist (with a README naming it the durable Warp-observation store). | file check |

## Phase Plan
- **P2 Design** — confirm D1-D4 + the exact hook algorithm (staged-spec detection + the section-emptiness check,
  mirroring enforce-changelog's staged-file pattern); the exact template/skill/CONSTITUTION wording; the smoke
  matrix. (Design also confirms the §20 wall wording for the design skill.)
- **P3 Implement** — the template + skills + CONSTITUTION edits; the hook; the settings registration; the
  observed/ dir + README.
- **P3.5 Inspect** — critic: the hook's staged-detection matches enforce-changelog (no false-allow / false-block);
  the emptiness check (whitespace / placeholder = empty); shellcheck-clean; the section placement; clean-room.
- **P4 Validate** — gate-is-test: RUN the hook's negative smoke (a spec missing the section piped as a staged
  commit → exit 2) + positive smoke (present → exit 0) + no-spec-staged → exit 0; `scripts/gates.sh --fast`
  green (shellcheck gate:11 on the new hook). No `.rs`, no unit tests.
- **P5 Complete** — CHANGELOG + `docs/marley_architecture/` (the pipeline/process doc); AAR; close #248; archive.
