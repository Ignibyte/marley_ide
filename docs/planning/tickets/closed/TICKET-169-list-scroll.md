# TICKET-169 — M10: scroll the rail + Files panel

- **Forge ticket:** #169 `ab767d07-a74a-4c27-8bb2-b38456d4ed52` (feature, M10; sprint #21)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `ec887cf5-7d59-40cf-8a74-920b28cabc36`
- **Pipeline doc:** ../../pipeline/active/list-scroll.spec.md
- **Status:** closed

## Summary
Wheel-scroll the overflowing left lists: row-skip offsets for the rail + the Files tree, clamped by the
reused scroll_code (cov/MSI 100 stands), remainder-precise; the files offset resets on a re-walk. Deps #152,
#154, #165.

## Acceptance
Driven — 12+ ⌘T tabs wheel into view + pin back; the Files tree reveals crates/ past the fold; gate GREEN.
