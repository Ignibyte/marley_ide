# TICKET-037 — prompt + input row chrome

- **Forge ticket:** #37 `4121bade-c650-49c1-a0af-9e38d9102d5d` (feature, M1.E — The Warp Look, seq-4)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `f6d7851d-8f2f-4956-ab34-acfb52bb4895`
- **Pipeline doc:** ../../pipeline/active/prompt-chrome.spec.md
- **Source ticket:** forge sprint #5 `cf1ba6de-3af6-4158-813e-b300e596c831` (M1.E — The Warp Look)
- **Status:** closed

## Summary
The prompt is a bare `{before}▏{after}` row. Give it Warp's input treatment: a `❯` accent marker, a
styled caret, an input-row surface, and a cwd/git context prompt. Pure: expose
`TerminalSession::current_prompt()` (the live staged prompt) in gpui-free terminal_blocks + a
marley_app `prompt.rs` (`prompt_segments(info)` deciding pwd/git segments + `pwd_label` shortening).
The input-row layout is shim. First cut = pwd + git (virtual_env/node deferred). Deps #34/#35 done.

## Acceptance
`current_prompt` + `prompt_segments` + `pwd_label` at cov 100/MSI 100 (Some-after-precmd/None;
pwd/git present-absent + order; the label edge cases); the input-row render (masked prompt visual —
❯ + segments + styled caret, chad-verified); FULL gate GREEN. Full EARS in the pipeline spec.
