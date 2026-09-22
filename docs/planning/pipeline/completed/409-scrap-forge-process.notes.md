# Scrap forge from the pipeline — Notes

- **Forge ticket:** LOCAL-ONLY (deliberate — this ticket retires the field)
- **AAR:** 6d71fd90-b213-4bf1-817d-8593b285ce8e — the FINAL forge AAR. Bookkeeping
  caveat, recorded honestly: `aar-open` requires a forge ticket id and 409 is
  deliberately LOCAL-ONLY, so it rides ticket #408's forge id; the intent text
  names its true subject.
- **Local ticket doc:** docs/planning/tickets/open/TICKET-409-scrap-forge-process.md
- **Pipeline spec:** 409-scrap-forge-process.spec.md

## Phase 1 — Plan

- **Request:** Chad's pivot (2026-08-09, interactive + the /goal amendment mid-run):
  scrap forge entirely for Marley; go directly ticket-based; ALSO turn the forge
  services off on this machine; bring every open forge ticket down as local files;
  end state = `/work` next item purely local. Scope answers (AskUserQuestion):
  everything-now (product rip = 410/411 from the seam map, in flight), Marley
  unplugs (amended mid-turn to: services OFF, DB kept), knowledge exported to a
  local ledger.
- **Classification:** docs/tooling chore, gate-is-test class (no `.rs`), local-only
  ticket — the FIRST one; runs under the OLD law end to end (final recall/capture
  against the live forge), then switches the law and turns the service off.
- **Recall (§18.3, old law):** the operative knowledge is this session's own:
  the forge-ops fragility record (TCC LaunchAgents, stale postmaster.pid,
  wrong-binary start-all.sh), the #405/#406 status drift found at /work pre-flight,
  and the FIFO probe of `ticket-next`. All recorded in the intake doc's Why.
- **Export inventory (REQ-002/003 baselines, read live at plan):**
  - prevention_rules 312 · failures 205 · distilled_lessons 201 ·
    architecture_decisions 43 · lessons 0 · golden_examples 0 (Marley project
    7c8f8800-87b5-4ad1-a7b9-f52e0c819d80).
  - Open tickets (16): 6, 223, 225, 226, 227, 271, 318, 319, 320, 332, 333, 353,
    354, 365, 366, 407.
- **Prior-art:** the shelf method (local spec files at production scale),
  TICKET-006 (de-automation by owner decision, reason recorded in the law), the
  gate-is-test discipline. Reference §20: N/A (process tooling, no product
  behavior). Details in the spec.
- **Decisions:** D1 history immutable · D2 DB outlives the service · D3 numbering
  continues · D4 amendment isolated commit · D5 exports before the off-switch.
- **Parallel work:** an Explore agent is mapping the product-layer forge seams
  (forge_client consumers, browser identity vs infra, fleet/brain wiring) for the
  410/411 authoring — running while this pipeline executes.

## Phase 2 — Design

**§20 confirmed:** N/A stands — process tooling only, no product behavior, wall untouched.

### Export mechanics (deterministic, count-asserted)
One python3 script per export (run at implement, key parts pasted into notes), reading
`psql --csv` with `ORDER BY code` / `ORDER BY number`, writing markdown; the script
asserts counts (312/205/201/43; 16 tickets) and exits nonzero on mismatch (D5).
Ledger block format, one entry per `## <code>` heading (greppable by code and text):
- `prevention-rules.md` — `## <code>` · severity (+ `prevents: <failure code>` when set) · rule body.
- `failures.md` — `## <code>` · category/severity · description (+ root_cause when set).
- `lessons.md` — distilled_lessons: `## <code>` · topic · guidance (+ confidence).
- `architecture-decisions.md` — `## <code>` · title · status · context / decision / rationale / consequences.
Each file opens with a provenance header (exported 2026-08-09 from the forge DB,
count, append discipline: inspect appends failures/prevention rules, complete appends
lessons/ADs — the ledger is the LIVE capture surface from now on).

### Ticket export + BACKLOG
`TICKET-<n>-<slug>.md` from the (de-forged) ticket shape: title, type, tags, created,
provenance line (forge-era uuid), description VERBATIM. Slugs hand-pinned kebab.
`BACKLOG.md` = two sections:
- **Queue** (`/work next` = the TOP row): 319 (bug: LSP file identity), 353 (bug:
  fold-state leak), 333 (bug: inlay lexer cosmetic), 320 (LSP doc-sync race-proofing),
  332 (LSP abandoned-request termination), 354 (format-on-save follow-up), 366
  (auto-close suppression), 318 (overlay-card recipe extraction), 223 (type_scale
  consolidation), 225 (keycap chip fill), 226 (OS notification follow-ups), 227
  (param-prompt modal), 365 (headed font-policy test).
- **Deliberate** (picked explicitly, never auto-next): 407 (the overnight FULL-audit —
  hours of mutation, idle-machine work), 271 (waits on a gpui `render_to_image`
  upgrade), 6 (BLOCKED: Windows CI runner).
Rationale: bugs → robustness chores → features/polish; anything gated on
machine-idle/upstream/hardware sits under Deliberate so "next" is always startable.

### Amendment (commit A — CONSTITUTION.md + enforcing hooks ONLY, per the rule)
- Preamble: the process sentence ("the knowledge sidecar this pipeline learns from
  (§19), via Marley's own per-project Forge bearer") → local-ledger wording. The
  PRODUCT words ("a **Forge MCP client**") stay until 410/411 — documented exception.
- §18.3 → "Local knowledge first": recall = grep `docs/planning/knowledge/` +
  `pipeline/completed/` notes before plan/implement; capture = ledger appends at
  inspect/complete.
- §19 → **Local Knowledge & Tickets (binding)**: local ticket docs + BACKLOG.md are
  THE tracker (top-of-Queue = next); numbering = 1 + max across open+closed; ledger
  append discipline; the loosening reason recorded in-section (owner pivot
  2026-08-09; the sidecar's operational fragility; recall/capture preserved locally).
- §3: "the `ticket:` frontmatter is the canonical link to the forge ticket (§19)" →
  the local ticket doc IS canonical.
- §0/§7's "a live Forge" uncoverable-path examples → "a live sidecar service".
- DELETE `.claude/hooks/enforce-mcp-config.sh`, `enforce-docs-before-code.sh`,
  `enforce-completion.sh` — verified UNWIRED in settings.json hooks (the `_comment`
  PARKED sentence is the only reference; commit A rewrites that comment too).
- `.claude/hooks/lib-hook-helpers.sh` — `is_pipeline_session` loses its third arming
  condition (the forge MCP tool-name grep, :180-196): arming = active phase command
  OR active pipeline doc. The intent-keying hole that arm closed dies with the tools
  themselves. English "forged" wording elsewhere in hooks is NOT forge-the-system
  (the REQ-001 smoke filters it).

### Manifest (commit B unless noted)
- NEW `docs/planning/knowledge/{prevention-rules,failures,lessons,architecture-decisions}.md`.
- NEW 16 × `docs/planning/tickets/open/TICKET-*.md` + `docs/planning/tickets/BACKLOG.md`.
- `.claude/commands/work.md` — pre-flight drops the forge/.mcp.json checks; Step 3 →
  "read BACKLOG.md; recall: grep the ledger + completed notes"; "next" = top Queue row.
- `.claude/commands/pipeline/plan.md` — LOCAL-ONLY always (drop forge minting +
  aar-open); numbering rule; recall = grep.
- `.claude/commands/pipeline/{design,implement}.md` — recall lines → grep the ledger.
- `.claude/commands/pipeline/inspect.md` — capture → append `failures.md` /
  `prevention-rules.md` blocks (same code conventions, `F-…`/`PR-…`).
- `.claude/commands/pipeline/complete.md` — §21 unchanged; capture → ledger appends;
  close = local mv + BACKLOG row removal; drop aar-submit/ticket-close.
- `.claude/commands/spec.md` — queue local tickets + BACKLOG rows; drop forge minting.
- `.claude/commands/commit.md` — drop the `forge.json` filename from the
  don't-stage example list (keep `.env`/secrets wording).
- Templates: `_templates/ticket.md` (Forge line → `Ticket: LOCAL` + provenance-free),
  `pipeline/_templates/pipeline.spec.md` (frontmatter `ticket:` local shape; drop
  `aar_id`), `pipeline/_templates/pipeline.notes.md` (drop Forge/AAR lines; recall
  wording local).
- `.mcp.json.example` — forge block removed (playwright stays).
- LOCAL-ONLY (untracked, no commit): `.mcp.json` forge entry removed;
  `.claude/settings.local.json` `enabledMcpjsonServers` loses "forge".
- CHANGELOG.md entry (commit B). Machine ops at P5 (no commit).

### Verification plan (gate-is-test, §7)
| S | REQ | Check |
|---|-----|-------|
| S1 | REQ-001 | `grep -riE 'forge' .claude docs/planning/_templates docs/planning/pipeline/_templates CONSTITUTION.md .mcp.json.example \| grep -viE 'forged\|forging'` → exactly the documented CONSTITUTION preamble product line (410/411's) and nothing else |
| S2 | REQ-002 | `grep -c '^## '` per ledger file == 312 / 205 / 201 / 43 (also asserted in-script at export) |
| S3 | REQ-003 | the 16 exported ticket files exist by number-set; BACKLOG rows == 16 |
| S4 | REQ-004 | static: work.md carries the top-of-Queue rule; behavioral: the first post-pivot `/work` (410 authoring) — recorded then |
| S5 | REQ-006 | `git status` shows zero modifications under `pipeline/completed/` + `tickets/closed/` |
| S6 | gates | `scripts/gates.sh --fast` green (shellcheck covers the lib-hook-helpers edit) |
| S7 | drift smoke | inject `forge` into a template → S1 red → revert → green (transcript-visible) |
| — | REQ-005 | at P5: `curl 127.0.0.1:8080` refused + `launchctl list` lacks both labels |
| — | REQ-007 | at commit: `git log --stat` of commits A + B pasted into notes |

### Risks
- **Hook-helper edit** breaks arming → mitigations: the two surviving arms carry this
  very session (active doc present); shellcheck at S6; the drift smoke S7 proves the
  Stop hooks still fire post-edit.
- **Session self-reference:** the forge curl shim stays usable until P5's off-switch
  (D5 ordering); the final aar-submit happens BEFORE the off-switch.
- **REQ-001 grep precision:** the forged/forging filter is pinned in S1; new English
  uses of "forge" in prose would false-positive later — acceptable, the smoke is a
  point-in-time gate, not a standing hook.

## Phase 3 — Implement

**React-first: N/A** (recorded — no `.rs`, no pixels).

Built to the manifest, in D5 order (exports while the DB lives → law → mechanics):
1. **Ledger export** — `export_ledger.py` (scratchpad; deterministic ORDER BY code,
   in-script count asserts): `docs/planning/knowledge/` 4 files, **312/205/201/43
   OK**. Fidelity spot-checked (incl. this morning's
   PR-claude-survivor-model-must-mirror-reader-drops-001).
2. **Ticket export** — `export_tickets.py`: 16 docs, number-set asserted; slugs
   hand-pinned; descriptions verbatim; provenance lines carry the forge-era uuids.
   `BACKLOG.md` written with the design's Queue (13) + Deliberate (3) order.
3. **Amendment set (the commit-A surface)** — CONSTITUTION: preamble process
   sentence → local (PRODUCT words remain, marked "retiring: TICKET-410/411");
   §0/§7 "a live Forge" examples → "a live sidecar service"; §3 diagram +
   frontmatter rule; §18.3 → Local knowledge first; §19 → Local Knowledge &
   Tickets with the in-section amendment record; the Amending section's stale
   §19-hook sentence. Hook trio `git rm`'d (verified unwired first);
   `lib-hook-helpers.sh` lost the forge arming arm + the now-orphaned
   `extract_mcp_tool_names`/`tool_was_called` helpers (shellcheck clean);
   `settings.json` `_comment` PARKED sentence dropped (0 forge refs).
4. **Skills** — all 8 de-forged with asserted python replacements (each printed
   `0 forge refs remain`): work (backlog-driven next + local recall), plan
   (local minting + numbering), design/implement (grep recall), inspect/complete
   (ledger appends; local close + BACKLOG upkeep), spec (local batch minting;
   sprint/aar leftovers cleaned), commit (example-filename scrub). validate.md had
   zero refs. No hook expects the dropped `aar_id`/forge frontmatter (grepped).
5. **Templates** — ticket/spec/notes de-forged (0 refs each).
6. **Configs** — `.mcp.json` (untracked), `.claude/settings.local.json`
   (untracked), `.mcp.json.example` (tracked): forge entries removed, playwright
   kept.

Deviation from design: none of substance; two orphaned generic helpers in
lib-hook-helpers were removed beyond the named arm (they had zero surviving
consumers — leaving dead shell in a gate-defining file would be its own smell).
Remaining forge tokens under `.claude/` are exactly the English word "forged" in
two hook comments (S1's filter covers them, verified).

## Phase 3.5 — Inspect

Three critics (export fidelity · law/process coherence · secrets/provenance).
The fidelity critic ran a FULL-corpus diff (all 761 ledger blocks + 16 ticket
descriptions byte-exact vs the DB). Ledger:

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| F1 | major | Tags lines were per-character soup in all 16 exported tickets — the DB double-encodes `tickets.tags` (jsonb STRING holding a serialized array); one `json.loads` + `join` iterated characters. Found independently by BOTH the fidelity and law critics. | REAL — fixed | Inner-decode with shape assert; all 16 Tags lines rewritten from the DB (verified readable). Captured to the ledger: F-claude-export-joined-double-encoded-jsonb-as-chars-001 + PR-claude-decode-jsonb-strings-twice-and-assert-shape-001 (the new law's first appends). |
| F2 | high | REQ-001/S1's "one documented exception" under-enumerated the legitimate survivors: the UNMARKED preamble thesis mention, the marker sentence's own tokens, and §19's mandated amendment record — Phase 4's smoke could never match its pin. | REAL — fixed | Preamble parenthetical now marks BOTH product mentions; REQ-001 re-pinned to the full exception set; S1 re-run post-fix → exactly 6 CONSTITUTION lines, all in-set (output in transcript). |
| F3 | med | §15 still claimed hooks catch "completing without the knowledge step" — that was the deleted (parked) enforce-completion.sh; nothing enforces ledger appends. | REAL — fixed | §15 omissions list rewritten to what IS enforced + an explicit "capture is discipline, not hook-enforced" sentence citing the amendment. |
| F4 | med | §19's code-prefix parenthetical (`PR-/F-/L-/AD-`) was false for the corpus (202 `BF-`, 201 `DL-`, 3 odd historical codes); normalizing would rewrite history AND break in-body cross-references. | REAL — fixed (law side) | §19 now pins: NEW appends use PR-/F-/L-/AD-; the corpus keeps its sidecar-era prefixes — history never renamed. Block format (code + italic meta + body) pinned in §19, making inspect.md's citation true. |
| F5 | med | REQ-007's commit-A letter ("only CONSTITUTION + enforcing-hook removals") contradicted the design's contents (settings.json comment, lib-hook-helpers EDIT). | REAL — fixed | REQ-007 re-pinned to the real amendment surface (deletions + the helpers edit + the settings comment citing the removed hooks). |
| F6 | low | Day-one self-violations of the new law: open TICKET-409 had no BACKLOG row; TICKET-410/411 were law-referenced but unreserved (next mint would steal #410). | REAL — fixed | BACKLOG semantics clarified law-wide (rows exist while open AND un-promoted; promotion removes the row — §3, BACKLOG prose, plan.md, complete.md sweep). 410/411 minted as real stub tickets from the seam map, at the TOP of the Queue (the post-pivot `/work next` picks 410 — REQ-004's behavioral proof). 409, being promoted/active, correctly has no row. |
| F7 | low | Ledger headers claimed inspect+complete append for ALL four files (wrong for lessons/ADs); §18.3 omitted design's lesson-append. spec.md still reported a "sprint id/name". Exported identity lines lacked the milestone. | REAL — fixed | Per-file appender lines; §18.3 gains the design append; spec.md report line fixed; milestone (from the M-tag) added to all 16 identity lines. |
| F8 | nit | settings.json `—` escaping + missing trailing newlines (settings.json, .mcp.json.example). | REAL — fixed | `ensure_ascii=False` + trailing newlines restored. |
| F9 | low/info | Secrets critic: `.claude/settings.local.json` ignored only via the user-global gitignore; no `.env` pattern in repo .gitignore. | REAL — hardened | Both added to `.gitignore`. Verified-negative kept on record: no real bearer EVER in git history (full-history sweep; only the 34-char placeholder). |

Clean sweeps (verified by the critics, kept as coverage): gitleaks over all new
content exit 0; zero forge-era MCP tool names or deleted-hook references in any
live surface; `bash -n` + the exact gate:11 shellcheck invocation GREEN;
`gate_state_hash` fingerprints the deletions cleanly (directory globs); gate:14
cannot red on the ledger (docs/planning excluded); the ledger is prose-only (zero
code fences) with zero third-party license markers; REQ-006 holds (nothing under
completed/ or closed/ modified); new-law path/section/numbering vocabulary agrees
across §19 ↔ work ↔ plan ↔ spec ↔ complete ↔ templates ↔ BACKLOG.

## Phase 4 — Validate

Gate-is-test class (§7) — no unit tests to add; verification = the design's smokes
+ the gate's own exit codes. All RUN in-transcript:
- **S1 (REQ-001):** the pinned grep → exactly the 6 documented CONSTITUTION
  exception lines (both marked preamble product mentions + marker + §19 amendment
  record). Nothing else across .claude/, templates, .mcp.json.example.
- **S2 (REQ-002):** ledger `^## ` counts 313/206/201/43 == the 312/205/201/43
  export baselines + the two inspect appends (F-…-jsonb-…-001, PR-…-decode-…-001)
  — every DB row carried, growth accounted.
- **S3 (REQ-003):** the 16 exported numbers all present in `open/`; 19 open docs
  total (16 + 409 active + 410/411 minted); BACKLOG rows = 18 (15 Queue incl.
  410/411 on top + 3 Deliberate).
- **S5 (REQ-006):** `git status` — zero paths under `pipeline/completed/` or
  `tickets/closed/`.
- **S7 (drift):** inject a forge line into ticket.md → S1 count 7 (RED) → revert
  → 6 (GREEN). First attempt used `git checkout` for the revert and CLOBBERED the
  template's own uncommitted de-forge edits (count stayed 7) — re-applied the
  edits, re-ran with a surgical `sed` revert: PASS. Lesson recorded in Phase 5.
- **Suites:** `cargo nextest run --workspace` → **2151 run, 2151 passed, 5
  skipped** (no `.rs` changed; proves the tree is undisturbed).
  `scripts/gates.sh --fast` → **GATE GREEN [fast], 11 passed 0 failed** —
  gate:10 gitleaks green over the 761-block import; gate:11 shellcheck green over
  the edited lib-hook-helpers.sh + surviving hooks; gate:14 green (ledger is
  outside doc-todos scope by design). Heavy gates skip per --fast; `/commit` runs
  `--diff` (no `.rs` in the changeset → receipt-exempt per §15).
- **REQ-004 static half:** work.md carries the top-of-Queue rule; the behavioral
  half lands when `/work next` promotes TICKET-410 (recorded there).
- **REQ-005/007:** deferred to P5/commit by design (service off-switch; two-commit
  shape).
**Pre-existing failures: none.**

## Phase 5 — Complete

- **CHANGELOG** — the pivot's `### Changed` entry (process half; the product half
  lands with 410/411).
- **§21(b) judgment, recorded:** no `docs/marley_architecture/` doc changes — the
  swept docs carry zero process-law forge mentions (grep shown in transcript);
  they describe the PRODUCT surface, which is 410/411's §21 duty. The process
  record IS the amended CONSTITUTION + the intake doc + this archive. Parity
  sync: N/A (non-UI).
- **Ledger appends (§19):** AD-claude-scrap-forge-local-tickets-001 ·
  PR-claude-drift-smoke-revert-surgically-001 ·
  L-claude-export-before-off-switch-full-diff-verification-001 (+ the two inspect
  appends). Ledger now 314 PR / 206 F / 202 L / 44 AD.
- **The final forge write:** aar-submit on 6d71fd90 (outcome completed, 5/5;
  materialized codes = the five above) at 2026-08-09T19:19:53Z — the sidecar's
  last act was recording its own retirement.
- **Ticket closed** → `tickets/closed/TICKET-409-scrap-forge-process.md`; BACKLOG
  sweep clean (the only 409 mention is the header's provenance prose, not a row).
- **Off-switch (REQ-005)** — executed post-archive, recorded here by commitment
  and in the transcript: forge-mcp/forge-sched processes stopped, both
  LaunchAgents booted out + plists renamed `.disabled` (reversible), 8080
  connection-refused verified. DB + postgres/redis untouched.
- **Memory** — the assistant's forge-ops memory rewritten to the post-pivot facts.
- Archived to `docs/planning/pipeline/completed/`.
