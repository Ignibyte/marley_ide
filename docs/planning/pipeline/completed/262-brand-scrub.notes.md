# 262-brand-scrub — Notes

- **Forge ticket:** #262 ae50af5a-4d7a-4d26-a2fb-4d92375b999a
- **AAR:** 5d1aeb91-91ae-426e-b7ec-4e9c5eecb483
- **Local ticket doc:** docs/planning/tickets/open/TICKET-262-brand-scrub.md
- **Pipeline spec:** 262-brand-scrub.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** /goal autonomous run `/work 262 265 260 261 263 264 266 267 268 269`
  — #262 first (chad's leading order). Execute the audited brand-scrub +
  keep-clean lint (ticket #262, sprint #29 "M16 — Cleanup + Editor Frontier").
- **Classification / tier:** chore, single shippable slice (comment-prose
  rewords + one gate function edit). No split needed — the audit already did
  the discovery.
- **Forge recall (§18.3):** bulletins: none. `knowledge-context` (Plan) logged
  13 surfacings (prevention rules + distilled lessons + ADs + 2 recent
  failures). Load-bearing priors from the pipeline memory: comment-only
  changes cannot move coverage/mutation, so the `--diff` gate is the right
  mode; gates.sh edits bind the §15 receipt (gate-defining file) — expected;
  enforce-changelog requires CHANGELOG staged in a separate bash before the
  compound commit bash.
- **Discovery:** pre-built by the 5-agent audit in
  `docs/planning/design-notes/brand-scrub/` — exact file:line → reword catalogs
  for all 56 mentions across 11 files (36+7+3+2+2+1+1+1+1+1+1); sweep confirms
  no extras, zero whole-word "Zed", zero non-comment hits. Lint design
  (matcher, -w rationale, docs_g fold-in point, sequencing) pre-written in
  `sweep-and-policy.md`.
- **Decisions:** D1–D6 in the spec (catalog-verbatim rewords; attestation
  reworded-not-deleted per chad's ticket text; gate:14 fold-in; `-w` BSD-safe
  matcher; same-commit sequencing; chad-quote paraphrase).
- **Autonomy note:** /goal Stop-hook run — plan review is pre-authorized
  (chad: "or any order you like"); proceeding to Phase 2 without a human
  pause, consistent with the sprint-#25 precedent.

## Phase 2 — Design

### Architecture / approach
Comment-prose rewrite + one shell-gate extension; no module/type surface
changes anywhere (no §14 concerns — no code paths touched).

1. **Rewords** — apply the three catalogs row-by-row with exact-string `Edit`
   ops (never line numbers — drift-proof; freshness re-verified: the detector
   returns exactly 56 lines / 11 files, byte-matching the audit table).
   `replace_all` only for the three literally-repeated app.rs strings:
   `#217 … hidden at rest, revealed on block-hover.` (×3),
   `#221 … rounded, framed card.` (×5), `#221 … rounded card (already framed).`
   (×4). Everything else is a per-row unique Edit.
2. **Gate fold-in** — extend `docs_g()` (scripts/gates.sh:156) with a THIRD
   check after rustdoc + doc-todos, per the file's own idiom
   (`local brand` on its own line, then assignment — matches `local hits`,
   shellcheck SC2155-safe):
   `brand=$(grep -rniwE 'warp|zed' crates --include='*.rs' 2>/dev/null || true)`
   → non-empty ⇒ echo header + hits, `return 1`. Update the gate:14 header
   comment (lines 147-155) to document the third half + the `-w` substring
   rationale, and the `run_gate` label (line 317) →
   `"gate:14 docs (rustdoc -D warnings + doc-todos + brand-scrub)"`.
   No new gate number (gate:16 stays retired). The grep scans `*.rs` under
   `crates/` only, so gates.sh's own header prose can name the brands safely
   and `docs/*_architecture/` stays exempt by scoping.

### Reference (§20) — confirmed
Still `N/A — Marley-specific source hygiene`. No reference-app behavior is
matched; the change removes brand names from Marley's own comments. Clean-room
wall untouched (no reference source read).

### File manifest
| file | change |
|---|---|
| crates/marley_app/src/app.rs | 36 comment lines reworded (39 tokens; 3 replace_all groups + singles) |
| crates/marley_app/src/typography.rs | 7 comment lines (1 doc, 6 in tests) |
| crates/ui_components/src/lib.rs | 3 comment lines (1 + 2 in tests) |
| crates/marley_app/src/workspace.rs | 2 `///` item-doc lines (#107, #120 blocks) |
| crates/marley_app/src/grid_layout.rs | 2 `//!` module-doc lines |
| crates/ui_components/src/render/keyboard_shortcut.rs | 1 `///` line (#222) |
| crates/marley_app/src/workflows.rs | 1 `//!` line (#204 feature-name) |
| crates/marley_app/src/right_dock.rs | 1 `///` line (chad-quote paraphrase) |
| crates/marley_app/src/nav.rs | 1 `//!` line (M12 #184) |
| crates/marley_app/src/file_tree_view.rs | 1 `//!` line (M5 #113) |
| crates/marley_app/src/color.rs | 1 `///` attestation reword (keeps clean-room + AGPL claim) |
| scripts/gates.sh | docs_g third check + header comment + run_gate label |

### Regression test plan (gate-is-test — §7; no runtime code changes ⇒ no new unit tests)
| REQ | Proof | How it runs |
|---|---|---|
| REQ-001 | detector empty | `grep -rniwE 'warp\|zed' crates --include='*.rs'` exits 1/empty in Validate; new gate:14 check green |
| REQ-002 | comment-only diff | inspect reviews full `git diff` (only `//`/`///`/`//!` text changes); `cargo nextest` green unchanged; `--diff` mutation sees no mutable code |
| REQ-003 | values survive | post-apply spot-grep: `#195`, `#230`, `#231`, `#222`, `#107`, `#120`, `#113`, `#204`, `#184`, `#134`, `13.0`, `Nav`, `L=0.11`→`0.11`, `REQ-003` all still present in their files |
| REQ-004 | lint catches | negative smoke: append `// Warp` to a crates .rs → `scripts/gates.sh --fast` RED on gate:14 (fmt/clippy/tests stay green — pure comment) → revert → `--fast` GREEN. Gate exit code is the verdict (§15) |
| REQ-005 | no substring FPs | the green runs happen on a tree still holding 80+ `…zed` substrings (`grep -c` StandardizedPath > 0 recorded) |
| REQ-006 | rustdoc clean | docs_g's rustdoc half inside the same gate runs |
| REQ-007 | attestation intact | `grep -n 'clean-room' crates/marley_app/src/color.rs` shows the AGPL originality claim |
- Uncoverable paths: none (no runtime code). Coverage/MSI floors unaffected by
  comment edits; gates.sh is held by gate:11 shellcheck.

### Risks / decisions
- R1 rustdoc breakage from `//!`/`///` rewords → REQ-006 gate catches; rewords
  keep link syntax untouched.
- R2 replace_all over-matching → post-apply detector must be exactly 0 AND
  `git diff` touched-line count must be 56 (+ gates.sh); any delta is
  investigated before inspect.
- R3 shellcheck (gate:11) on the new check → follow the existing
  local/assign idiom.
- R4 receipt binding: gates.sh is a gate-defining file (§15) — the same-commit
  scrub+lint plan means one `--diff` green covers both.
- R5 same-commit sequencing (D5) avoids the lint-born-red trap flagged in
  sweep-and-policy.md.

## Phase 3 — Implement
- **Built:** all 56 rewords applied via exact-string Edits (27 ops on app.rs —
  24 singles + 3 replace_all groups; 6 typography; 3 lib.rs; 1 each color /
  keyboard_shortcut / nav / right_dock / file_tree_view / workflows; 2 each
  workspace / grid_layout). gates.sh: `docs_g` gained the brand third-check
  (`local hits brand`; grep -rniwE 'warp|zed' crates --include='*.rs'), the
  gate:14 header comment documents the brand-scrub half, run_gate label now
  "… + brand-scrub".
- **Deviations from the catalogs (3, all reads-broken fixes — D1 allows):**
  1. typography.rs:43 — catalog's "(dense-density parity)" read redundant →
     `#195: calibrated to a tight type scale (…)` (keeps #195 + all values).
  2. typography.rs:123 — catalog's "a future density tune" would double the
     article (line 122 already ends "so a future") → "density tune can't …".
  3. app.rs:2905-2906 — applied the catalog's OPTIONAL period fix (Nav = 12. /
     The "Files" …) as a two-line edit so the sentence stays grammatical.
- **Verification:** detector grep → 0 hits (exit 1); `git diff --stat` = 11 .rs
  files + gates.sh, app.rs 37 pairs (36 + the 2905 period line), counts match
  the audit; `shellcheck -S info -e SC1091` (gate:11's exact invocation) CLEAN;
  `cargo check --workspace` green (pre-existing upstream `block v0.1.6`
  future-incompat note only).

## Phase 3.5 — Inspect
- **Critics run:** 2 (parallel, general-purpose): A = scrub-correctness
  (comment-only, values, coherence, completeness, rustdoc, attestation);
  B = gate-mechanics + provenance (BSD grep/-w semantics, Bash 3.2, set -e/
  pipefail, self-trip, shellcheck, §15 receipt interplay, §20).
- **Mechanical verifications (critic A):** diff -U0 = 57-/57+ lines; code
  prefixes before the first `//` byte-identical across the pair-stream ⇒
  provably comment-only. Ordered token streams of every #NNN/M-/R-/numeric
  value identical removed→added ⇒ zero lost refs/values. All-substring `warp`
  grep empty ⇒ no remnant possible. Paren/quote/backtick parity per pair.
- **Findings ledger:**

| # | sev | finding | verdict | action |
|---|---|---|---|---|
| A1 | LOW | app.rs:416 "header band matching the section headers" reads circular after the brand (the matched authority) was dropped | REAL (catalog-verbatim wording, reads-broken) | fixed: "a compact ~22px band for the section headers" |
| A2 | LOW | app.rs:5763 "The signature block actions" lost its possessor | REAL (minor) | fixed: "The hallmark block actions (R49)" |
| A3 | LOW | right_dock.rs:74 paraphrase presented inside verbatim-quote marks misattributes wording to chad | REAL | fixed: `(chad's ask: icons, not text tabs)` — no quote marks |
| B1 | MED | quality-bar.spec.md:24 gate-14 row stale — gates.sh names it "the single source of truth" yet the row lacked the new brand-scrub clause | REAL (doc drift) | fixed now (not deferred): row documents the brand check + the docs/*_architecture exemption |
| B2 | LOW | `-w` misses CamelCase/underscore identifiers (WarpBlock/warp_mode) | REJECTED — spec Out-scope: documented accepted limitation (sweep-and-policy.md); current exposure proven ZERO (case-sensitive + snake-boundary probes empty); boundary-variant is the recorded escape hatch | none (recorded residual) |
| B3 | LOW | `|| true` conflates grep exit 2 with no-match | REJECTED — measured fail-closed-or-stricter under POSIXLY_CORRECT; identical pre-existing idiom in gates 12/13/14 (parity, not a new hazard) | none |
| B4 | INFO | no --exclude-dir=vendor | REJECTED — correct for a brand lint (vendored brand-bearing code under crates/ SHOULD trip it) | none |

- **Platform verifications (critic B):** /usr/bin/grep = BSD 2.6.0-FreeBSD;
  `-w` applies to both alternatives; --include after path parses + filters;
  Bash 3.2.57 `bash -n` clean + both branch paths proven in a 3.2 harness;
  gates.sh runs `set -uo pipefail` (no -e) and docs_g executes in run_gate's
  errexit-suppressed `if "$@"` context; shellcheck (gate invocation) exit 0;
  §15: change strictly additive, `gate_state_hash` fingerprints scripts/*.sh so
  this edit invalidates any stale receipt (mechanism confirmed working);
  §20: zero added lines contain a brand substring; attestation semantically ≥
  the old claim (and the old one HAD to be reworded — it contained the brand
  and would have tripped the new lint).
- **Post-fix re-verify:** detector exit 1 (empty); 416/417 continuation reads
  clean.

## Phase 4 — Validate
- **Gate-is-test change (§7):** no runtime code touched ⇒ no new unit tests;
  proof = gate exit codes + negative smoke, per the Phase 2 plan.
- **Driven UI capture: N/A** — comment-only diff; comments have no render/input
  surface, nothing to drive. (Stated explicitly per the validate charter.)
- **REQ-004 negative smoke (RUN):** injected `// Warp` at color.rs top →
  `scripts/gates.sh --fast` = **GATE RED**, gate:14 FAIL printing exactly
  `crates/marley_app/src/color.rs:9:// Warp`; all 10 other static gates PASS
  (fmt/clippy/tests/audit/deny/machete/gitleaks/shellcheck/no-suppress/SAST) —
  the injection changed nothing else. Reverted; detector clean again (exit 1).
- **REQ-001 (RUN):** `grep -rniwE 'warp|zed' crates --include='*.rs'` → empty,
  exit 1.
- **REQ-005 (RUN):** 23 `StandardizedPath` lines remain in-tree and unflagged.
- **REQ-007 (RUN):** color.rs:9 attestation present (clean-room + AGPL claim).
- **REQ-003 (RUN):** spot-refs alive post-scrub: #195 ×5 + #230 ×2
  (typography), #231 ×4 (lib.rs), #107 ×2 (workspace), #204 ×7 (workflows),
  #184 ×17 (nav), #113 ×3 (file_tree_view), #134 ×2 (right_dock), #222 ×1
  (keyboard_shortcut).
- **REQ-002 (RUN):** `cargo nextest run --workspace` → **873/873 passed**,
  5 skipped (the #[ignore] headed lane) — zero test changes, behavior
  unchanged.
- **Gate (RUN):** `scripts/gates.sh --diff` → **GATE GREEN [diff]** — 15
  passed, 0 failed (incl. the extended gate:14 "docs (rustdoc -D warnings +
  doc-todos + brand-scrub)", gate:4 coverage ≥100% lines, gate:5 MSI ≥100%,
  gate:6 miri, gate:15 visual/AX). Receipt written (.git/ignibyte-gate-receipt)
  binding the scrubbed tree + the edited gates.sh.
- **Pre-existing notes:** none in scope (the `block v0.1.6` future-incompat
  advisory is upstream, pre-existing, and not gated).

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG.md — new entry at the top of [Unreleased] ###
  Changed; docs/marley_architecture/clean-build-plan.md — clean-room
  discipline gained the "source itself is brand-clean" bullet;
  docs/planning/design-notes/brand-scrub/README.md — status flipped to
  EXECUTED (historical record); docs/specs/standards/quality-bar.spec.md
  gate-14 row — updated at inspect (B1 fix).
- **Knowledge (§19):** AAR 5d1aeb91 submitted (completed). Captured:
  BF-claude-gate-spec-row-drift (inspect B1),
  PR-claude-gate-change-updates-quality-bar-row-001,
  AD-claude-brand-lint-gate14-001 (fold-in over new gate number; -w matcher;
  same-commit sequencing).
- **Lessons:** (1) a pre-audited catalog still carries its own defects — the
  typography.rs:123 proposed reword would have doubled an article; treat
  catalogs as input, re-derive context. (2) Extending a gate = updating its
  canonical spec row, same change. (3) The attestation HAD to be reworded —
  the old text itself contained the brand and would have tripped the new lint
  (self-referential lint hazard worth remembering for future "ban the word X"
  gates).
- **Ticket:** TICKET-262 → tickets/closed/, forge #262 → done.
- **Archive:** spec+notes → docs/planning/pipeline/completed/.
