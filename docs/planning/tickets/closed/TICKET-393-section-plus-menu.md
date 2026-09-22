# TICKET-393 — The section ＋ becomes a menu (section-scoped verbs, the extension point)

- **Forge:** #393 `dd98405c-ec53-4aba-8376-8f4e434520c5` (sprint #38 `fc972431`)
- **Type:** feature
- **Milestone:** M27
- **Status:** closed (done 2026-07-22)
- **Pipeline:** docs/planning/pipeline/queued/393-section-plus-menu.spec.md

## Summary
The #387 ＋ generalizes from one hardcoded verb to a small anchored section-scoped menu (the #166
machinery): Terminal → New Terminal / New Terminal Here; Editor → Open File… / Open in Split;
Browser → Forge / Agents / Details; Panes (if #390 landed) → Split Right / Split Down. Reuse-only;
the pure `section_menu` table carries routing at cov/MSI 100; future items (SSH, workflows, URL,
New File) are the documented extension point.

## Headline acceptance
Clicking a section ＋ opens its menu (first item = the old default); an item dispatches against the
row's project (activate-first + persist-the-switch) and dismisses; esc/click-away dismisses without
dispatch; the collapse toggle still never fires from the ＋.
