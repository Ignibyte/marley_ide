# TICKET-042 — clipboard paste + bracketed-paste mode

- **Forge ticket:** #42 `d1b32f42-f609-4192-8214-12a09e3e8e03` (feature, M1.F — Real Interactivity, seq-3 FINALE)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `cba47182-58ae-4665-bdf7-de42bcd9ddf9`
- **Pipeline doc:** ../../pipeline/active/bracketed-paste.spec.md
- **Source ticket:** forge sprint #6 `73c9385b-91d4-44f9-be99-edf666e6510d` (M1.F — Real Interactivity)
- **Status:** closed

## Summary
No paste today. Add cmd-V paste with bracketed-paste safety: a pure `paste_bytes(text, bracketed)`
wraps `ESC[200~`…`ESC[201~` (stripping an embedded end-marker — the paste-injection guard) or passes
raw; `is_bracketed_paste()` tracks DECSET 2004 (mirrors is_alt_screen). The shim reads the clipboard +
routes via #40 (running program vs local buffer). Pure surfaces cov/MSI 100. Closes M1.F. Deps #40/#33.

## Acceptance
`paste_bytes` + `is_bracketed_paste` at cov 100/MSI 100 (wrap/raw goldens; the embedded-marker strip
→ exactly one closing marker; DECSET-2004 true/false); the cmd-V handler (masked visual — multi-line
paste into an editor arrives as one block, chad-verified); FULL gate GREEN. Full EARS in the spec.
