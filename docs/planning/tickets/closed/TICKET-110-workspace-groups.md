# TICKET-110 — workspace groups in the sidebar [M5 seq-4]

- **Forge ticket:** #110 `ee9a310e-a2bf-437e-aac5-625c2d8f955c` (feature, M5 seq-4; sprint #16)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `87fd8c72-820d-492b-85b5-e9ceee19174d`
- **Pipeline doc:** ../../pipeline/active/workspace-groups.spec.md
- **Status:** closed

## Summary
`Session.workspace` + `SessionGroup` + `group_sessions(sessions, focused)` (first-seen order, reuses
session_rows; cov/MSI 100); the sidebar renders a header per group + its rows. Deps seq-3.

## Acceptance
group_sessions at cov/MSI 100 (grouping/first-seen/order/active); the sidebar shows named group headers
(live capture); FULL gate GREEN. Full EARS in the spec.
