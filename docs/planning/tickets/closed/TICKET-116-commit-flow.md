# TICKET-116 — the commit flow (git WRITE) [M5 seq-10]

- **Forge ticket:** #116 `bc1a1312-eb69-4e08-9b9c-8a6c505e6459` (feature, M5 seq-10; sprint #16)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `9c401f8b-623e-4f36-92e5-73ff0700051b`
- **Pipeline doc:** ../../pipeline/active/commit-flow.spec.md
- **Status:** closed

## Summary
**Marley's FIRST git write.** `commit_enabled(staged_count, message)` (pure, cov/MSI 100) + a confined
git-write adapter (stage/unstage/commit only — no push/force/rewrite; argv message) + a commit-message
input + a Commit button + stage-toggle in the git panel. Deps #115 + #004.

## Acceptance
commit_enabled at cov/MSI 100; the adapter confined (add/restore/commit, argv message — code review);
FULL gate GREEN; nothing auto-committed during validation. Full EARS in the spec.
