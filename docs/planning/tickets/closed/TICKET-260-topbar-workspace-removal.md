# TICKET-260 — Remove the top-bar workspace indicator/switcher

- **Forge ticket:** #260 7d0d3756-9c85-4fdd-8fee-34e6b69d3566 (chore, M16)
- **Owner:** ede913c3-d048-4f39-ad2b-b21cef1efc8e
- **AAR:** 0dcc41ed-4c50-4d1c-b22a-44c86752d953
- **Pipeline doc:** ../../pipeline/active/260-topbar-workspace-removal.spec.md
- **Source ticket:** sprint #29 "M16 — Cleanup + Editor Frontier" (forge)
- **Status:** closed

## Summary
Remove the #235 focused-workspace indicator ("Marley · main") and the #244
click-to-switch popover from the top bar (chad: "I don't want the project
workspace at the top. we opt in for the left"). The left Workspace rail
(#233 click-switch + #236 highlight) is the canonical home; the removal loses
no capability. The now-dead pure fns (`focused_workspace_indicator`,
`workspace_switcher_rows` + `SwitcherRow`), state (`workspace_switcher_open`),
consts, and their tests are deleted in the same change; the cockpit tabs
re-anchor to the vacated slot. The #142 OS-titlebar label is separate and
untouched.

## Acceptance
Top bar shows no workspace indicator/popover; the rail still switches with
the highlight following; cockpit tabs sit at the vacated anchor and work; no
dead code remains; #142 unchanged. Full EARS in the pipeline spec.
