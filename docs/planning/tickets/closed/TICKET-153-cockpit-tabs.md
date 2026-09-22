# TICKET-153 — M9 seq-4: cockpit as full-screen tabs (retire the right dock)

- **Forge ticket:** #153 `334bd265-51fe-4f75-9a57-a7f9671e28d5` (feature, M9; sprint #20)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `296823c2-2f12-4234-95b5-d268434c7591`
- **Pipeline doc:** ../../pipeline/active/cockpit-tabs.spec.md
- **Status:** closed

## Summary
Details/Agents/Forge become full-screen Cockpit tabs (retire the right dock). Pure helpers (`cockpit_section`,
`terminal_grid_index` so `workspace()` never panics on a cockpit-active tab, `open_or_switch_cockpit`) at
cov/MSI 100; the shim extracts `cockpit_body`, branches the center render, and rewires the top-right icons to
open a cockpit tab. Deps #150/#151/#152. Mitigates the right-dock half of #158.

## Acceptance
The pure helpers at cov/MSI 100; a driven capture shows a cockpit tab filling the CENTER + returning to a
terminal without panic; FULL gate GREEN. Full EARS in the spec.
