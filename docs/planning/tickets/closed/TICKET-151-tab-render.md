# TICKET-151 — M9 seq-2: full-screen active-tab render (retire default tiling)

- **Forge ticket:** #151 `3898b95e-fe1f-40d4-a302-a4b1674efc29` (feature, M9; sprint #20)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `09fd277c-de4f-457b-a524-fbb62fa98925`
- **Pipeline doc:** ../../pipeline/active/tab-render.spec.md
- **Status:** closed

## Summary
Wire the seq-1 `Workspace` model into the app view: a `shell` field replaces the direct grid; the content
area renders the active tab full-screen; `+` adds a terminal TAB (retire default tiling) while `⌘D` still
splits within the active tab. Migrate the 103 grid call-sites via `workspace()/workspace_mut()` accessors; a
temp next-tab key + pure `next_index` make switching drivable. Masked shim + one tiny pure fn. Deps #150.

## Acceptance
Driven captures: boot 1 terminal full-screen; `+` → a 2nd terminal TAB (not a split); next-tab key swaps the
view; `⌘D` splits within the tab. `next_index` at cov/MSI 100. FULL gate GREEN. Full EARS in the spec.
