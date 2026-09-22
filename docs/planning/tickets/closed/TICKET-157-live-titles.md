# TICKET-157 — M9 seq-8: live rail titles + real branch (FINAL M9)

- **Forge ticket:** #157 `13cbc967-15c4-4712-9d62-3f9a27ca8171` (feature, M9; sprint #20)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `d2091c32-0201-4af7-8ebc-913a24f9242c`
- **Pipeline doc:** ../../pipeline/active/live-titles.spec.md
- **Status:** closed

## Summary
The rail comes alive: a terminal tab row shows the running command (not "terminal N"); a project row shows its
real git branch (`name · branch`). Pure `rail_tab_title(command, fallback)` at cov/MSI 100; shim `live_tab_title`
+ reuse `branch_from_git_head`. Deps #150-153, #156. Closes M9 sprint #20.

## Acceptance
rail_tab_title at cov/MSI 100; a driven capture — a running command shows in the rail tab row; the project row
shows its branch; FULL gate GREEN. Full EARS in the spec.
