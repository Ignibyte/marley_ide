# TICKET-357 — Render the #275 external-change conflict banner on a focused editable split pane

- **Forge ticket:** #357 (02cbc091-eebf-4ca2-a73d-8b58fbaee5a6) (feature, M15 → ships M22)
- **Owner:** 466e35ad-09f6-4b81-89e7-b7fd16c1e45d
- **AAR:** 43fac3b4-aa2d-4df2-ade9-f753fcebb936
- **Pipeline doc:** ../../pipeline/active/357-conflict-banner-split-pane.spec.md
- **Source ticket:** M22 IDE wrap-up train (#306, #295, #356, #357, #355)
- **Status:** closed

## Summary
The #275 Keep-mine/Reload external-change banner renders only in the editor-tab block, so a focused editable
split pane with an armed `ExtConflict` shows the dirty ● but no banner. Fix: extract the inline banner into a
reusable `ext_conflict_banner` builder and render it on the focused split pane too (its buttons already act on
`active_editor()` = the focused surface). Full EARS in the pipeline spec.

## Acceptance
A focused editable split pane with an armed external-change conflict shows the #275 banner; the editor-tab
banner is unchanged.
