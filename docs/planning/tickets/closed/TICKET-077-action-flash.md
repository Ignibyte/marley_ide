# TICKET-077 — action confirmation indicator (flash)

- **Forge ticket:** #77 `b0663d2a-b137-4f38-b01b-0484db040b5e` (feature, M2.D seq-6 — the finale)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `69a8654a-9d4b-43bd-958a-6125f9dcc678`
- **Pipeline doc:** ../../pipeline/active/action-flash.spec.md
- **Source ticket:** forge sprint #12 `c93f9693-5569-4208-bb6a-20d38afec99b` (M2.D — The Controlling Cockpit)
- **Status:** closed

## Summary
A transient confirmation flash (copied/sent/claimed/comment) so cockpit actions give feedback. PURE (new
flash.rs): `Flash{message, remaining}` + `new`/`tick` + FLASH_TICKS (~2s, tick-counted — Date::now banned).
SHIM: status_flash field, the pump decrements it (dirty), a bottom strip renders it, the 5 actions
(#70/#72/#73/#75/#76) set it. cov/MSI 100 on flash.rs; the wiring masked + self-test-verified. Deps the
pump (#67) + the action sites.

## Acceptance
Flash::new/tick at cov/MSI 100; an action → the flash appears then fades (self-test); FULL gate GREEN.
Full EARS in the spec.
