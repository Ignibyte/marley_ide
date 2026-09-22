---
pipeline_id: 298e473d-e910-4ff4-8069-58fb20dd2d5f
ticket: forge#248 (d369915f-4728-4836-9f18-730bfc47dfb8)
aar_id: e2221e1e-b048-40cc-a84c-7540722ef1c4
---

# Notes — Force Warp-reference into the pipeline (forge#248)

## Plan (Phase 1)

**Classification:** work-pipeline CHORE, gate-is-test (docs + a shell hook + settings + CONSTITUTION — NO `.rs`).
The FIRST ticket of the M15 "Editable Editor" train; the rest inherit the discipline. NOT auto-approved (chad
cleared the /goal) — present the manifest + WAIT for his OK before design.

**Intent (chad 2026-07-11):** "lets refer to how warp does everything... are you referring to the warp build for
guidance? We should probably force the pipeline on this project to do so." Make Warp-matching a forced, enforced
per-ticket step, not the ad-hoc "match Warp" vibe it's been.

**The gap (discovered):** CONSTITUTION §20 mandates clean-room-from-`docs/warp_architecture/` behavior specs +
observed behavior, BUT (a) there's no forced per-ticket "capture/observe how Warp does THIS and match it" step,
and (b) `docs/warp_architecture/` has crates/subsystems/depgraph.json only — the editor/input isn't specced
there, so the whole M15 editor train would have nothing concrete to reference without this.

**Discovery (the anchors, verified):**
- `enforce-changelog.sh` (the hook PATTERN) — a PreToolUse:Bash hook: intercepts `git commit` (a shared
  git-detection regex), reads `git diff --cached --name-only`, and exits 2 (block) if `.rs` is staged without
  `CHANGELOG.md`. Registered in `.claude/settings.json` under `hooks.PreToolUse` (matcher "Bash") beside
  `enforce-commit-gate.sh`. This is the exact shape `enforce-warp-reference.sh` mirrors (staged-file check → exit
  2), gating a staged pipeline spec instead of `.rs`-vs-CHANGELOG.
- The spec template (`_templates/pipeline.spec.md`) has `## Title / ## Scope / ## Locked-In Decisions / ##
  Acceptance Criteria (EARS) / ## Phase Plan` — the new `## Warp Reference (§20)` goes between Scope + Decisions.
- `docs/warp_architecture/` = crates/subsystems/depgraph.json; NO `observed/` dir yet.

**Decisions:** D1 the section shape + 3 options + placement · D2 enforce-at-commit (mirror enforce-changelog) ·
D3 the hook gates PRESENCE+non-emptiness only (a human judges correctness — the §18.1 provenance check) · D4
dogfood (this spec carries the section).

**Risks / load-bearing:**
- The hook's staged-spec detection must mirror enforce-changelog EXACTLY (git-commit detection + the staged-name
  grep) — a divergence risks a false-allow (a spec commits without the section) or false-block (`PR-claude-
  detection-tracks-runner`, cited in enforce-changelog itself). Design pins the exact regexes.
- The emptiness check: an empty section = the heading followed by nothing / only whitespace / a bare `…`/`TODO`
  placeholder. Design nails the grep so a stub doesn't pass.
- This is a PROCESS change — it changes how EVERY future ticket runs. chad reviews the concrete edits before
  implement (not auto-approved).

**Test plan (gate-is-test, §7):** REQ-002 the hook smokes — craft a temp staged spec missing the section →
pipe a fake `git commit` PreToolUse input → assert exit 2; with the section → exit 0; no spec staged → exit 0.
REQ-001/003/004/005/006 file/grep/doc checks. `scripts/gates.sh --fast` green (shellcheck gate:11).

**Warp Reference (§20) for THIS ticket:** N/A — Marley-specific pipeline-process change; no Warp analog. (This
spec dogfoods the section it introduces.)

**AAR:** e2221e1e-b048-40cc-a84c-7540722ef1c4 (opened).

**Phase 1 status: Plan PASS pending chad's OK — NOT auto-approved. Present the manifest; WAIT before design.**

## Design (Phase 2)

**chad greenlit** ("yep looks awesome") + a refinement: generalize `## Warp Reference` → **`## Reference (§20)`**
(reference-app-aware — Warp for terminal/cockpit, Zed for the editor later; same hook). Folded in below.

**Architecture.** Pure process/governance change — no `crates/*`, no `.rs`. Touches the pipeline docs (template
+ skills + CONSTITUTION), a new commit hook (mirroring `enforce-changelog.sh`), and its settings registration.
Verified via the hook's exit codes (gate-is-test, §7).

### The hook algorithm (`enforce-warp-reference.sh`) — the load-bearing design
Mirror `enforce-changelog.sh` EXACTLY for the trigger (PR-claude-detection-tracks-runner):
```
set -euo pipefail; INPUT=$(cat); command -v jq >/dev/null || exit 0
source "$(dirname "$0")/lib-hook-helpers.sh"          # PROJECT_ROOT = git root
CMD=$(echo "$INPUT" | jq -r '.tool_input.command // empty'); [ -n "$CMD" ] || exit 0
echo "$CMD" | grep -qE '(^|[^[:alnum:]_.-])git[[:space:]]+([^;&|]*[[:space:]])?commit([[:space:]]|$)' || exit 0
if echo "$CMD" | grep -qE -- '(--dry-run|--help|(^|[[:space:]])-h([[:space:]]|$))'; then
    echo "$CMD" | grep -qE '[;&|]' || exit 0
fi
cd "$PROJECT_ROOT" 2>/dev/null || exit 0
STAGED=$(git diff --cached --name-only 2>/dev/null || true); [ -n "$STAGED" ] || exit 0
```
THE NEW CHECK — a staged pipeline spec must carry a filled `## Reference (§20)`:
```
SPECS=$(printf '%s\n' "$STAGED" | grep -E '^docs/planning/pipeline/(active|completed)/.*\.spec\.md$' || true)
[ -n "$SPECS" ] || exit 0                               # no real spec staged → exempt
#  (the _templates/ path is NOT active|completed → auto-exempt; a template is not a real spec)
FAIL=""
printf '%s\n' "$SPECS" | while IFS= read -r f; do
    [ -n "$f" ] || continue
    BLOB=$(git show ":$f" 2>/dev/null || true)          # the STAGED (index) content, not the worktree
    printf '%s\n' "$BLOB" | grep -qE '^## Reference' || { echo "MISS $f"; continue; }
    SECTION=$(printf '%s\n' "$BLOB" | awk '/^## Reference/{c=1;next} c&&/^## /{c=0} c{print}')
    MEANINGFUL=$(printf '%s\n' "$SECTION" \
        | sed '/<!--/,/-->/d' \                          # strip the HTML-comment guidance block
        | grep -vE '^[[:space:]]*$' \                    # drop blank lines
        | grep -vE '^[[:space:]]*(…|<[^>]*>|TODO)[[:space:]]*$')   # drop bare placeholders
    [ -n "$MEANINGFUL" ] || echo "EMPTY $f"
done > /tmp/warpref.$$  ; ...
```
(DESIGN NOTE: `while | ... > file` because a bash-3.2 pipe subshell can't set a parent var; collect the
misses to a temp file, then read it after the loop. Implement finalizes the exact plumbing — a `for` over the
newline list avoids the subshell entirely; LEAN a `for f in $SPECS` with IFS=newline since spec paths have no
spaces.) If any MISS/EMPTY → print a §20 message naming the file(s) + `exit 2`; else `exit 0`.

**Why the HTML comment matters (D-guidance):** the template's `## Reference (§20)` body is an `<!-- … guidance
… -->` comment. The hook STRIPS comment blocks + whitespace + bare placeholders — so the bare template reads as
EMPTY (would block, but it's path-exempt), and a REAL spec MUST add non-comment prose (a cite / capture path / an
"N/A — …" line) to pass. This cleanly distinguishes "filled" from "untouched" without the hook judging content.

### Manifest
| File | Change |
|---|---|
| docs/planning/pipeline/_templates/pipeline.spec.md | ADD `## Reference (§20)` between `## Scope` and `## Locked-In Decisions`, body = an HTML-comment naming the 3 options (Warp / Zed / N/A) + the clean-room caveat (observe behavior, use gpui freely, NEVER read the copyleft app source). |
| .claude/commands/pipeline/plan.md | Step 4 (write the spec): require FILLING `## Reference (§20)`. Add a closeout bullet. |
| .claude/commands/pipeline/design.md | Step 1 (architecture): require CONFIRMING `## Reference (§20)` + a sentence on HOW the design matches that behavior (the §20 wall: observe/reimplement, never read source). |
| .claude/hooks/enforce-warp-reference.sh | NEW — the algorithm above. `chmod +x`. shellcheck-clean (bash 3.2 / BSD). |
| .claude/settings.json | ADD a 3rd entry to the PreToolUse "Bash" matcher hooks array: `{type:command, command:"bash .claude/hooks/enforce-warp-reference.sh", timeout:10}`. |
| CONSTITUTION.md §20 | ADD a bullet: every spec carries a `## Reference (§20)` section (plan-filled, design-confirmed); `enforce-warp-reference.sh` binds it. |
| docs/warp_architecture/observed/README.md | NEW dir + README — the durable Warp/Zed observation store (screenshots/notes), NOT ephemeral scratchpad. |
| docs/planning/pipeline/active/force-warp-reference.spec.md | DOGFOOD: rename this spec's own `## Warp Reference (§20)` heading → `## Reference (§20)`. |

### Regression Test Plan (gate-is-test, §7 — no `.rs`, no unit tests)
| # | Test | Proves |
|---|---|---|
| S1 (neg) | temp `git init` repo; stage a spec whose `## Reference (§20)` is comment-only/empty; pipe `{"tool_input":{"command":"git commit -m x"}}` to the hook (cwd=temp → PROJECT_ROOT=temp) → assert **exit 2** | REQ-002 blocks the empty case |
| S2 (pos) | same, section filled with `N/A — Marley-specific …` → assert **exit 0** | REQ-002 passes the filled case |
| S3 (exempt) | stage a non-spec file only (e.g. a .rs or README) → **exit 0** | no-spec exemption |
| S4 (exempt) | stage ONLY `docs/planning/pipeline/_templates/pipeline.spec.md` → **exit 0** | the template is path-exempt |
| S5 (missing) | stage a spec with NO `## Reference` heading → **exit 2** | REQ-002 blocks the missing case |
| F1-F5 | file/grep/doc checks: template has the section (REQ-001); settings registers the hook (REQ-003); plan+design require it (REQ-004); CONSTITUTION §20 codifies it (REQ-005); observed/README exists (REQ-006) | REQ-001/003/004/005/006 |
| — | `scripts/gates.sh --fast` green — shellcheck (gate:11) on the new hook; the existing gates unaffected (no `.rs`). | gate |

**Uncoverable:** none — this is fully shell-testable. The smokes run a REAL hook against a THROWAWAY temp repo
(no touch to the real index).

**Risks:** (1) the git-commit detection MUST stay in lockstep with enforce-changelog (copy verbatim). (2) the
section-extraction awk must be BSD/bash-3.2 safe (ASCII `^## Reference` prefix — don't try to match the `§`
byte). (3) the bash-3.2 pipe-subshell var trap (collect misses to a temp file OR use `for f in $SPECS`). (4) the
template path must be exempt (the active|completed regex already excludes _templates/ — confirm).

**`## Reference (§20)` for THIS ticket:** N/A — Marley-specific pipeline-process/governance change; no
reference-app analog. (This spec dogfoods the section + its new name.)

**Phase 2 status: Design PASS — manifest + hook algorithm + smoke matrix locked (chad greenlit). Ready for
Phase 3 — Implement.**

## Implement (Phase 3)

Built all 8 manifest items (docs + a shell hook — no `.rs`). Verified: `bash -n` OK, `shellcheck -S info -e
SC1091` (the EXACT gate:11 invocation) CLEAN on the new hook AND the whole hook+script set, settings.json valid
JSON, hook executable.

- **.claude/hooks/enforce-warp-reference.sh** (NEW, +x) — mirrors enforce-changelog's git-commit trigger
  verbatim; gates staged `docs/planning/pipeline/(active|completed)/*.spec.md` (the `_templates/` path is
  auto-exempt — not active|completed); for each, reads the STAGED blob (`git show :$f`), requires a `## Reference`
  heading + a body that survives stripping the HTML-comment guidance + whitespace + bare placeholders; else exit
  2 naming the file(s). Used a `for f in $SPECS` (IFS=newline) loop to dodge the bash-3.2 pipe-subshell var trap
  (accumulate `FAIL` in the parent). shellcheck's only finding = SC1091 on the `source` line — the SAME info
  enforce-changelog triggers, and gate:11 runs `-e SC1091` → excluded.
- **.claude/settings.json** — registered the hook as the 3rd PreToolUse:Bash entry (after enforce-changelog).
- **_templates/pipeline.spec.md** — `## Reference (§20)` section (HTML-comment guidance: Warp / Zed / N/A + the
  clean-room caveat) between Scope + Locked-In Decisions.
- **plan.md** — step 4 requires filling it + a closeout bullet. **design.md** — step 1 requires confirming it +
  stating the §20 match.
- **CONSTITUTION.md §20** — a "Forced reference (enforce-warp-reference.sh)" bullet after the spec_source line.
- **docs/warp_architecture/observed/README.md** (NEW dir) — the durable Warp/Zed observation store + the
  clean-room caveat + a naming convention.
- **This spec** — dogfooded: its `## Warp Reference (§20)` → `## Reference (§20)` (N/A body kept).

**Deviations from design:** none material. (The SC1091 info is gate-excluded, matching enforce-changelog — not
suppressed in-file, to stay identical to the sibling hook.)

**Phase 3 status: Implement PASS. Ready for Phase 3.5 — Inspect.**

## Inspect (Phase 3.5)

1 general-purpose critic that RAN the hook against ~14 crafted temp git repos + my own 6-case smoke. All 14
required cases PASS (block missing/empty, allow filled, exempt template/no-spec/non-commit/dry-run, staged-not-
worktree, multi-spec-names-the-bad-one, awk boundary no-bleed), detection regex BYTE-IDENTICAL to
enforce-changelog (PR-claude-detection-tracks-runner holds), shellcheck clean. The critic's ADVERSARIAL probing
then found 2 real bugs my self-review + smoke missed:

- **F1 — [HIGH, FIXED] false-BLOCK: a same-line `<!-- … -->` comment nukes following prose.** Root cause: `sed
  '/<!--/,/-->/d'` — in a POSIX sed RANGE the closing `/-->/` is only tested on lines AFTER the opener, so a
  comment opened+closed on ONE line never closes there → the range deletes that line + everything to the next
  `-->` (or EOF), swallowing legit prose. Bites when an editor/prettier collapses the guidance comment to one
  line, or an author inlines a comment in prose (`Warp <!-- n --> matches`). Inherent sed semantics (BSD == GNU).
  **Fix:** strip SAME-LINE comments first (`sed 's/<!--.*-->//g'`) THEN the multi-line block (`sed '/<!--/,/-->/d'`).
  Verified: single-line-comment+prose → ALLOW (0); inline-comment-in-prose → ALLOW (0); comment-only (single AND
  multi-line) still → BLOCK (2). `BF-claude-sed-range-cannot-close-same-line-html-comment`.
- **F2 — [LOW, FIXED] false-ALLOW: a `## References` (plural) heading satisfied the gate.** `grep '^## Reference'`
  + `awk /^## Reference/` are PREFIX matches → `## References` / `## Reference Material` with content passed when
  the real `## Reference (§20)` was ABSENT (bounded: only when the §20 heading is missing entirely). **Fix:**
  anchor both to the exact `^## Reference \(§20\)`. Verified: plural-References-no-real → BLOCK (2); no regression.
  `BF-claude-prefix-grep-heading-false-allows-sibling-heading`.

Both fixes re-verified via the smoke (7/7 correct) + shellcheck clean. My self-review + smoke covered the happy
paths + the multi-line-comment/bleed cases but MISSED the same-line-comment collapse (the sed-range gotcha) and
the prefix-heading sibling — the critic's job. All other lenses (detection parity, exemptions, staged-index read,
CONSTITUTION/skill/template wording consistency, clean-room) clean.

**Phase 3.5 status: Inspect PASS — F1 (HIGH) + F2 (LOW) fixed & re-verified; all 14 required cases pass. Ready
for Phase 4 — Validate.**

## Validate (Phase 4)

Gate-is-test (§7 — docs + a shell hook, no `.rs`, no unit tests). The "tests" are the hook's exit-code smokes,
now saved as a COMMITTED repeatable regression.

**Test added — `scripts/selftest/warp-reference-hook-smoke.sh`** (shellcheck-clean, bash-3.2/BSD-safe; builds
throwaway `mktemp -d` repos, pipes a fake `git commit` PreToolUse input, asserts exit codes). **RAN → 16/16
pass, exit 0:**
- BLOCK (2): no heading · comment-only (single-line) · comment-only (multi-line) · whitespace/…/TODO ·
  `completed/*.spec.md` empty · **plural `## References` sibling [F2]** · awk-boundary prose-in-next-section.
- ALLOW (0): filled N/A · filled Warp-cite · **single-line-comment-above-prose [F1]** · **inline-comment-in-prose
  [F1]**.
- EXEMPT (0): no spec staged · only the `_template` · non-commit command · standalone `--dry-run`.
- STAGED-not-worktree (2): staged empty + worktree filled → still blocks (reads the index).

**File/doc checks (REQ-001/003/004/005/006) — all pass:** the template has `## Reference (§20)`; settings.json
registers the hook (valid JSON); plan.md requires fill + design.md requires confirm; CONSTITUTION §20 has the
forced-reference bullet; `docs/warp_architecture/observed/README.md` exists. **SANITY:** this ticket's own spec
has a FILLED `## Reference (§20)` (the N/A dogfood) → the new hook won't block its own commit.

**No-regression:** `cargo nextest run --workspace` → **825 passed, 5 skipped** (the docs/hook change touches no
`.rs`).

**Gate — `git add -A && scripts/gates.sh --fast` → GATE GREEN [fast] 11/11** (rustfmt, clippy, tests, audit,
deny, machete, gitleaks, **shellcheck [the hook + the smoke script]**, no-suppressions, SAST, docs). Coverage/
mutation/miri/visual SKIP (`--fast`, no `.rs` — correct). This commit has no `crates/*/src/*.rs` → enforce-
commit-gate + enforce-changelog EXEMPT it (no FULL receipt needed). No pre-existing failures in scope.

**Phase 4 status: Validate PASS — the hook smoke is 16/16 green (committed regression), all REQ checks pass, no
regression, static gate green. Ready for Phase 5 — Complete.**

## Complete (Phase 5)

- **Docs (§21):** CHANGELOG.md `### Changed` (above #245) — "Every pipeline spec must now carry a `## Reference
  (§20)` section". docs/marley_architecture/clean-build-plan.md — a forced-per-spec bullet in the Clean-room
  discipline section. The architecture record for this IS CONSTITUTION §20 + the plan/design skills + the
  template (edited); no separate arch doc needed.
- **Knowledge:** `aar-submit e2221e1e` completed, effectiveness 5. What worked: the HTML-comment-strip trick
  cleanly separates a FILLED section from the bare template. What bit: a self-authored smoke tests only the
  paths the author thought of (multi-line comments) — the independent critic that EXECUTED the hook found 2 real
  bugs (the sed same-line-comment range false-BLOCK + the prefix-heading false-ALLOW); both fixed, the smoke
  hardened to 16 cases (incl those repros) + committed as a regression. Recorded at inspect:
  `BF-claude-sed-range-cannot-close-same-line-html-comment` (3279ae58, HIGH) +
  `BF-claude-prefix-grep-heading-false-allows-sibling-heading` (384d05b7, LOW) +
  `PR-claude-sed-range-prestrip-same-line-before-multiline-delete-001` (6f6397a3). Materialized that PR + the
  #245-era `PR-claude-detection-tracks-runner` (the git-detection was mirrored byte-identical to enforce-
  changelog — critic-confirmed).
- **Close:** forge #248 (`d369915f`) → done; TICKET-248 open→closed.
- **Archive:** spec status `Phase 5 — Complete PASS`; the spec+notes → `pipeline/completed/`.

**Phase 5 status: Complete PASS. Run `/commit` to deliver.**
