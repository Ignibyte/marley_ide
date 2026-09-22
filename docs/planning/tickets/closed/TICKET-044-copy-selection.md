# TICKET-044 — copy the selection (cmd-C)

- **Forge ticket:** #44 `e352a858-a281-4757-91a7-10a7f630019d` (feature, M1.G — Block Workflows & Selection, seq-2)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `5203985e-387c-4339-970b-ecbdc34b15ff`
- **Pipeline doc:** ../../pipeline/active/copy-selection.spec.md
- **Source ticket:** forge sprint #7 `eb842cb6-f4ab-4b27-89e0-78634476f304` (M1.G — Block Workflows & Selection)
- **Status:** closed

## Summary
cmd-C copies the #43 selection to the clipboard — completing copy/paste (#42 paste). Extends
text_selection.rs: `row_slice` (char-safe slice), `selected_text` (reuses #43 `row_selection` so copy
== highlight, `\n`-joined), `copy_payload` (None for absent/empty selection — no empty clipboard
write). The cmd-C handler + the clipboard write are shim. Pure surfaces cov/MSI 100. Dep #43.

## Acceptance
`row_slice` + `selected_text` + `copy_payload` at cov 100/MSI 100 (char boundaries/clamp; matches the
highlight; the empty-guard None/None/Some); the cmd-C round-trip (masked visual — select→cmd-C→paste,
chad-verified); FULL gate GREEN. Full EARS in the pipeline spec.
