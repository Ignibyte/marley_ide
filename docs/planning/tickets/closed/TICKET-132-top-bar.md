# TICKET-132 — the Warp top bar (search in-bar; fixes the close button) [M7 seq-1]

- **Forge ticket:** #132 `4bed8453-ad63-46fd-840f-1dbadb6e8c59` (bug, M7; sprint #18)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `3f8c5b3c-a044-44c1-b62d-23434ffbc10d`
- **Pipeline doc:** ../../pipeline/active/top-bar.spec.md
- **Status:** closed

## Summary
Build a real top bar: pure `content_band` reserves a top band (vertical analogue of `region_widths`); the
docks + pane grid shift below it so the #117 search sits IN the bar (not floating over the terminal, #2) and
the pane title bars are no longer occluded so the close-× works (#3). Foundation for #133/#134. Deps M6.

## Acceptance
content_band at cov/MSI 100; the search in a real top bar + the panes below it (live capture); FULL gate
GREEN. Full EARS in the spec.
