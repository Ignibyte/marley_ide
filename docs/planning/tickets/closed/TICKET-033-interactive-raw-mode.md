# TICKET-033 — terminal: interactive/raw mode (vim, top, less, Ctrl-C) [FLAGSHIP]

- **Forge ticket:** #33 `09a81942-989f-4d0f-a191-1a4290131e3f` (feature, M1.D — The Daily Driver, seq-6)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `c58ef971-3d07-432f-b251-1b2e56504f1b`
- **Pipeline doc:** ../../pipeline/active/interactive-raw-mode.spec.md
- **Source ticket:** forge sprint #4 `a029f2bc-dabd-43f5-9f6d-13498f0916d5` (M1.D — The Daily Driver)
- **Status:** closed

## Summary
Input is line-buffered — no Ctrl-C, no full-screen programs. Add a pure keystroke→bytes encoder
(`encode_key` + `ctrl_byte` + `input_route`) in marley_terminal, plus alt-screen raw mode
(`is_alt_screen`/`grid_styled_rows` + the shim routing + a live-grid render reusing #31), so
vim/top/less/htop work and Ctrl-C/D/Z interrupt. Tab-completion at the bare prompt is DEFERRED (the
local-editing-vs-shell-ZLE model decision — intake `prompt-shell-line-editing-model.md`).

## Acceptance
`encode_key`/`ctrl_byte`/`input_route` at cov 100/MSI 100 (every key byte + the C0 formula + the
routing); `is_alt_screen` tested via a mock DECSET 1049; a real-zsh Ctrl-C-interrupts-`sleep`
integration test; the headed vim/top baseline masked (desktop-session deferral); FULL gate GREEN.
Honest partial: vim/top/Ctrl-C YES, prompt tab-completion DEFERRED. Full EARS in the pipeline spec.
