# TICKET-041 — full interactive key coverage (Shift-Tab, F-keys, modified arrows, Insert)

- **Forge ticket:** #41 `acf70690-53e8-45ab-a9ad-3e86598d51a4` (feature, M1.F — Real Interactivity, seq-2)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `a5c302fd-1892-41ed-9006-4efced4cb324`
- **Pipeline doc:** ../../pipeline/active/interactive-keys.spec.md
- **Source ticket:** forge sprint #6 `73c9385b-91d4-44f9-be99-edf666e6510d` (M1.F — Real Interactivity)
- **Status:** closed

## Summary
Extend the pure gpui-free `encode_key` with the keys interactive menus/editors need beyond #33's
basics: `BackTab` (Shift-Tab → ESC[Z), `F(u8)` (F1-12), `Insert` (ESC[2~), and modified cursor keys
(the xterm `ESC[1;<param><final>` via `modifier_param` = 1+shift+2·alt+4·ctrl + `csi_cursor`).
`KeyInput` gains `shift`; the shim maps the new gpui key names. Golden byte-vector tests + the
modifier arithmetic — cov/MSI 100. Deps #40/#33.

## Acceptance
The new `encode_key` arms + `modifier_param` + `csi_cursor` at cov 100/MSI 100 (goldens per arm; the
arithmetic none→1/Shift→2/Ctrl→5/all→8; Shift-Up→ESC[1;2A, plain Up→ESC[A); FULL gate GREEN. Chad
verifies Shift-Tab in a real menu. Full EARS in the pipeline spec.
