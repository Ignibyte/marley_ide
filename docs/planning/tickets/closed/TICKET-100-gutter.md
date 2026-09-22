# TICKET-100 — line-number gutter

- **Forge ticket:** #100 `7af97ea6-a745-4f4c-9246-07c316beaf85` (feature, M4 seq-4; sprint #15)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `b95bfee9-1fd6-49ad-931a-7590205ce5bd`
- **Pipeline doc:** ../../pipeline/active/gutter.spec.md
- **Status:** closed

## Summary
A right-aligned line-number gutter: PURE `gutter_width(line_count)` + `gutter_label(n, width)` (cov/MSI
100); the viewer renders the muted gutter sized by the file's line count. Current-line highlight = #101.
Deps #97 + #39.

## Acceptance
gutter_width + gutter_label at cov/MSI 100; the viewer shows aligned numbers; FULL gate GREEN. Full EARS
in the spec.
