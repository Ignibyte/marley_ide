# TICKET (doc-phase) — Strict documentation phase

- **Forge ticket:** #3 `368f2d21-74ec-4e21-bee1-47dcc09984e4` (chore)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `0c98d79e-be52-4aa9-a95f-743d5becb770`
- **Pipeline doc:** `../../pipeline/active/strict-doc-phase.spec.md`
- **Branch:** `ticket-strict-doc-phase`
- **Status:** closed — delivered (forge #3 → done); enforce-changelog live, §21 added, skip-audit done

## Summary

Force a CHANGELOG entry + architecture-doc update at the end of every pipeline,
unskippably: a root `CHANGELOG.md`, an `enforce-changelog.sh` PreToolUse hook that
blocks a `.rs` commit lacking a CHANGELOG change (no-`.rs` exempt, mirroring the
commit receipt), a STRICT `pipeline:complete` doc step, a CONSTITUTION §, and a
phase-skip-resistance audit. Done before the crate stack so 002→ each carries a
forced CHANGELOG + arch-doc update.

## Acceptance

REQ-001…009 in the pipeline spec. Headline: a code commit without a CHANGELOG
entry is blocked; `gates.sh --fast` green; complete-phase + CONSTITUTION made
strict; the 5 enforcement hooks audited (all bite).
