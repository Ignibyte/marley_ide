---
pipeline_id: 9283ea9a-855c-49b8-a6ae-91024a2dfcba
ticket: forge#262 (ae50af5a-4d7a-4d26-a2fb-4d92375b999a) · local docs/planning/tickets/open/TICKET-262-brand-scrub.md
aar_id: 5d1aeb91-91ae-426e-b7ec-4e9c5eecb483
status: Phase 5 — Complete PASS
title: Brand-scrub — remove Warp/Zed mentions from the source + a keep-clean lint
type: chore
milestone: M16
references:
  - docs/planning/design-notes/brand-scrub/README.md
  - docs/planning/design-notes/brand-scrub/sweep-and-policy.md
  - docs/planning/design-notes/brand-scrub/app-rs.md
  - docs/planning/design-notes/brand-scrub/ui-cluster.md
  - docs/planning/design-notes/brand-scrub/workspace-misc.md
---

## Title
Execute the audited brand-scrub (roadmap Phase A): reword the 56 whole-word
"Warp" mentions (39 tokens in app.rs + 12 UI cluster + 8 workspace/misc — all
code comments, zero "Zed", zero strings/identifiers/config) across 11
`crates/**/*.rs` files in Marley's own terms, then wire the keep-clean brand
lint into gate:14 so a mention can never reappear. Motivation (chad): nothing in
the Marley codebase should mention the reference brands, so a future
proprietary/relicense pivot isn't shadowed by "derived-from" fear.

## Scope
### In
- Apply the exact per-line rewords from the three catalogs
  (`app-rs.md`, `ui-cluster.md`, `workspace-misc.md`) to the 11 files:
  app.rs (36 lines), typography.rs (7), ui_components/lib.rs (3),
  workspace.rs (2), grid_layout.rs (2 — `//!` header), keyboard_shortcut.rs (1),
  workflows.rs (1 — `//!` header), right_dock.rs (1), nav.rs (1 — `//!` header),
  file_tree_view.rs (1 — `//!` header), color.rs (1 — attestation).
- `color.rs:9` clean-room attestation: reword per the ticket ("not lifted from
  any AGPL-licensed source") — preserve the originality claim, do not delete.
- Keep-clean lint: the sweep doc's brand check folded into **gate:14 docs_g**
  (per the ticket), using the BSD-safe whole-word form
  `grep -rniwE 'warp|zed' crates --include='*.rs'` — landed in the SAME commit
  as the scrub (wiring it earlier goes red on the current 56).
- CHANGELOG + arch-doc touch at Phase 5 (§21).

### Out (explicitly deferred)
- `docs/warp_architecture/` + `docs/zed_architecture/` — the deliberate
  exception; they keep their mentions (that is their job).
- The stricter boundary-regex variant catching `warp_layout`-style identifiers —
  adopt only if an underscore-identifier mention ever appears (documented
  limitation of `-w`).
- Any code/string/identifier change — the audit found none carrying a brand;
  this pipeline is comment-prose only.
- The OS-titlebar / theme-name surfaces — already brand-clean per the audit.

## Reference (§20)
N/A — Marley-specific source hygiene. No reference-app behavior is being
matched; this change *removes* reference-brand names from Marley's own comments
(clean-room posture reinforcement). The reference transcriptions under
`docs/warp_architecture/` + `docs/zed_architecture/` keep their mentions and are
out of scope by the lint's `crates/`-only scoping.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — Apply the catalogs' proposed rewords verbatim (they preserve every
  ticket number + technical value); deviations only where a sentence reads
  broken in context, noted in the notes ledger.
- D2 — `color.rs:9`: reword to keep the clean-room claim ("independently
  authored, not lifted from any AGPL-licensed source") — chad's ticket text
  resolves the catalog's KEEP? flag as "reword, preserve the claim".
- D3 — Lint wiring = fold into gate:14 `docs_g` (ticket says gate:14; the sweep
  doc's Option A) as a third check after rustdoc + doc-todos; no new gate
  number (gate:16 stays retired).
- D4 — Matcher = `grep -rniwE 'warp|zed' crates --include='*.rs'`: `-w` is the
  substring guard (80+ `…zed` words like `StandardizedPath`); POSIX `-w`, not
  GNU `\b` (gates.sh is Bash-3.2/BSD-safe).
- D5 — Same-commit sequencing: scrub + lint land together so the lint is born
  green.
- D6 — `right_dock.rs:74` chad quote → paraphrase "use icons, not text tabs";
  `workflows.rs:1` → "#204 — reusable saved commands" (catalog wording).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method (gate exit code,
negative smoke, or review).

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `grep -rniwE 'warp|zed' crates --include='*.rs'` runs after the scrub, it shall return zero matches (exit 1 / empty). | Direct run in Validate + the new gate:14 check green. |
| REQ-002 | The scrub shall change comment text only — no code token, string literal, identifier, or test-data value shall differ. | Inspect review of the full diff; `cargo nextest` unchanged-green; `--diff` mutation finds no mutable code in the change. |
| REQ-003 | Every ticket number and technical value in an affected comment (e.g. #195/13pt, #230/Nav 12, #231/L=0.11, REQ-003 in typography.rs) shall survive the reword. | Inspect review against the catalogs; spot-grep the ticket refs post-apply. |
| REQ-004 | WHEN a whole-word `Warp` or `Zed` is introduced into any `crates/**/*.rs`, gate:14 shall exit non-zero naming the offending line. | Negative smoke in Validate: inject → gate red → revert → green. |
| REQ-005 | The lint shall NOT flag substring words (`StandardizedPath`, `initialized`, `normalized`, …) — the post-scrub tree (which still contains 80+ such lines) shall pass. | gate:14 green on the scrubbed tree (the substrings remain). |
| REQ-006 | The 4 reworded `//!` module-doc headers shall keep rustdoc warning-free. | gate:14 rustdoc `-D warnings` exit code. |
| REQ-007 | `color.rs` shall retain an explicit clean-room/originality attestation (mentioning AGPL, not the brand). | grep `clean-room` + `AGPL` in color.rs; inspect review. |

## Phase Plan
- **P2 Design** — confirm the exact edit list (catalog → file/line map is
  pre-built); design the gate:14 fold-in (where in `docs_g`, label text,
  shellcheck-clean); regression test plan = negative smokes + detector run.
- **P3 Implement** — apply the rewords (replace_all for the repeated #221
  variants); edit `scripts/gates.sh` docs_g; verify detector returns 0.
- **P3.5 Inspect** — independent critics vs the diff (comment-only? values
  preserved? attestation intact? lint BSD-safe?); fix the real findings.
- **P4 Validate** — RUN the gate (`--diff`); negative smokes (inject brand →
  red → revert); rustdoc green; record in notes.
- **P5 Complete** — CHANGELOG + arch-doc note (clean-build-plan/provenance),
  AAR capture, archive, close #262.
