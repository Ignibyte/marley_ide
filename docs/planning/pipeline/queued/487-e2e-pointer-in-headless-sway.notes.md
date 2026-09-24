# Scenarios that click run Marley in a headless sway — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-487-e2e-pointer-in-headless-sway.md
- **Pipeline spec:** 487-e2e-pointer-in-headless-sway.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3 (Chad: "lets make this happen. plan, spec tickets, then
  execute"; "do your best for decisions"). The browser tab is mouse-driven; the harness could
  not click.
- **Classification:** chore, tooling. No Rust; the gate's shellcheck covers the scripts.
- **Recall (§18.3):**
  - L-claude-483-send-keys-to-one-hyprland-window-by-address-001: the Hyprland backend's key
    path; Hyprland has only `send_key_state`, `send_shortcut` and `pass`, none for a pointer.
  - L-claude-483-a-scenario-brings-its-own-shell-001 and L-claude-481-…: the stand-in agent
    this scenario reuses.
  - CONSTITUTION §7: "No mouse: a click would move the user's pointer" — the rule this
    ticket amends, for the sway backend only.
- **Discovery (this session):** the feasibility check in the scratchpad (headless sway,
  Marley at 1600×1000, a C helper over the virtual-pointer protocol, `wtype -s` holding the
  keyboard): click, wheel and chords all reached Marley. The helper prototype is 110 lines.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.
