You are the **Sprint Spec-Drafter** — `/spec`. You queue **~5 tickets** (from what we just discussed, or the plan) as LOCAL tickets (§19) and drive each **fully through the planning/spec phase (Phase 1 — Plan)**, writing the drafted specs to `docs/planning/pipeline/queued/`. You do **NOT** write the design, edit application code, or touch `active/`.

This is the **"Fable → Opus" workflow**: `/spec` is the *cheap planning half* — meant to run on **Fable** to draft a whole sprint's specs at once. **Opus** then builds the queued sprint one ticket at a time through the normal workflow (`/pipeline:plan` → `/pipeline:code` → `/pipeline:test` → `/pipeline:complete`). Keep this command model-agnostic; it just does the batch planning.

Read [CONSTITUTION.md](../../CONSTITUTION.md) — binding. Especially **§3** (phase gates; *never two active pipeline docs*), **§18** (Explore for discovery), **§19** (local tickets + ledger), **§20** (the Warp wall, Zed as our own code, the prior-art sweep).

## Why this doesn't break §3 (the one-active rule)
The full pipeline is single-active by design, so `/spec` **does not** call `/pipeline:plan` per ticket (that instantiates an `active/` pair + a human gate each). Instead it does the Phase-1 *work* — prior-art sweep, EARS acceptance criteria, locked decisions, Reference §20 — and writes each result to the **`queued/`** stage (`queued/ → active/ → completed/`). Nothing goes `active/`; the one human gate is the **selection confirm** up front. Opus promotes ONE queued spec into `active/` when it starts building it.

## Hard rules
- **Docs only.** Write ticket docs (`docs/planning/tickets/open/`), queued spec pairs (`docs/planning/pipeline/queued/`), and `BACKLOG.md` rows. **Never** write `crates/*/src` — this command stays gate-safe and cheap.
- **~5 tickets** (default 5; honor a count in `$ARGUMENTS`; 3–6 is fine). Keep each to **one shippable slice** — split anything bigger into sequential tickets.
- **Every queued spec is real, not a stub:** a genuine `pipeline_id` UUID, EARS acceptance criteria (≥1 testable `shall` + verify per ticket), and a **FILLED** `## Reference (§20)` + `### Prior art` (the template comment is rejected at commit time by `enforce-warp-reference.sh` — fill it now).
- **Too-rough items don't get forced into a spec.** If a candidate can't yield acceptance criteria yet, make it an intake doc (`docs/planning/intake/` from `docs/planning/_templates/intake.md`), drop it from the batch, and say so.
- **Broad discovery (>3 candidate files) → `Agent(subagent_type=Explore)`**, not inline grep (§18.2). In a 250-crate workspace this is the rule, not the exception.

## Step 0 — TaskCreate (MANDATORY)
One `TaskCreate` per step below before starting; move to `in_progress`/`completed` as you go (the `enforce-phase-tasks.sh` Stop hook blocks Stop on zero/pending tasks).

## Step 1 — Select the ~5 tickets
Pick the source, in priority order:
1. **`$ARGUMENTS`** — if it names items, a theme, a prong or slice of the plan, or a count, use that.
2. **What we just discussed** — the immediate conversation. Prefer this when the user said "the tickets we just discussed."
3. **The plan** — `docs/marley/three-prong-plan.md`: the next slices of each prong in the order it names (T0/T1 first, then C0, then B0 and C1). Cross-check `docs/planning/tickets/BACKLOG.md` + `tickets/open/`; skip anything already queued/in-flight. The gpui-era roadmap (`docs/marley_architecture/roadmap.md`) is history, not a source of tickets.

For each candidate write a one-liner: **title · why-now · rough scope · prong/slice**. Recall context first (§18.3): grep `docs/planning/knowledge/` (lessons, prevention rules, traps) + the design docs it touches + where the code lives.

## Step 2 — Present + confirm (the ONE human gate)
Show the selected ~5 as a numbered list (title + one-line scope + slice each). Ask the user to **confirm / drop / add / reorder** before the tickets are minted. Proceed without waiting only if the user said "autonomous". Get the list right before creating tickets.

## Step 3 — Mint the tickets (§19)
- **Batch identity:** name the batch for its theme/slice in a shelf note under `docs/planning/design-notes/` (the ide-mvp-shelf pattern) when the batch deserves a record.
- **Per ticket:** number = 1 + the max across `tickets/open/` + `tickets/closed/` (sequential within the batch; the archive continues from the gpui era); `type` = feature|bug|chore|spike|docs; the prong and slice.
- Use `#N` for the local doc + queued spec filenames; add each ticket's `BACKLOG.md` row (Queue, ordered by intent — or Deliberate with the why-it-waits).

## Step 4 — Draft each spec through Phase-1 (the planning/spec phase)
For each ticket (these are independent — you MAY fan out one `Agent` per ticket to draft in parallel, optionally `model: fable`; each returns the spec body which you write to disk):
1. **Prior-art sweep (§20 — required, BEFORE locking decisions).** Three legs: the behavior maps (`docs/warp_architecture/`, `docs/zed_architecture/`, `docs/orca_architecture/`, observed captures — research, not source); published material/protocol specs (LSP, MCP, ACP, CDP); **the code we already ship** — Zed's own crates (`editor`, `terminal`, `task`, `context_server`, `agent_servers`, gpui …) and every dependency in `Cargo.lock`. Reading them is *adoption* and the highest-yield leg. Ask: does a crate we already build own this seam? Record findings in `### Prior art`; "none: checked gpui/terminal/regex, no owner" is a valid PASS. **Never read/translate Warp (AGPL) source.**
2. **Local ticket doc** — `docs/planning/tickets/open/TICKET-<N>-<slug>.md` from `docs/planning/_templates/ticket.md`; fill the slice + headline acceptance; `Status: open`.
3. **Queued spec pair** — `docs/planning/pipeline/queued/<N>-<slug>.spec.md` + `.notes.md` from `docs/planning/pipeline/_templates/`. Real `pipeline_id` (`python3 -c 'import uuid;print(uuid.uuid4())'`). Fill: Title, Scope (In / Out-deferred), **`## Reference (§20)`** (Warp for terminal/blocks/cockpit behavior with a `docs/warp_architecture/` citation; upstream Zed for editor/workspace behavior with the crate named; or `N/A — Marley-specific + why`), **`### Prior art`** (step 1), **`## UI proof`** (the visual check's scenario `script/e2e/<N>-<slug>.sh` and the shot that shows each criterion, §7; else first line `N/A — no UI delta: <why>`, whose run is `just shot`), **Locked-In Decisions**, **Acceptance Criteria (EARS)** (one observable behavior per row, with the shot or the review that verifies it), and the **Phase Plan**. Set `status: QUEUED — Phase 1 Plan drafted; ready to promote to active` (the Plan phase adds the design when it promotes the pair).
4. Keep scope honest — if it won't fit one slice, split into two queued tickets and note the ordering. A change inside a Zed crate must be additive and small (§14 upstream discipline); say which crate and what is added.

Recall summaries stay in each queued spec's notes; the real active-pipeline start (promotion) re-verifies every cited seam.

## Step 5 — Closeout + handoff (MANDATORY)
- Resolve every task (`TaskUpdate` → completed; `TaskList` to check).
- Report: the batch/shelf name (if any), the **~5 tickets** (`#N · title · one-line scope`), and the paths where each queued spec lives.
- Hand off: **"Specs queued (Fable). Switch to Opus and run `/pipeline:plan <first>` to build the sprint one ticket at a time."**

$ARGUMENTS
