# TICKET-022 — shell-integration AnsiCQuoted emit: `;`/`=` in a command corrupts Block segmentation

- **Forge ticket:** #22 `02632a5b-cf30-48c7-a008-2cc1c0b9c5d7` (bug, M1.C — The Wired Cockpit, seq-1)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `c88f9a9c-44b8-4020-8c02-b6a49c013244`
- **Pipeline doc:** ../../pipeline/active/shell-integration-ansic.spec.md
- **Source ticket:** forge sprint #3 `b295976b-4f32-4edc-be3c-aecd80b02785` (M1.C — The Wired Cockpit)
- **Status:** closed

## Summary
The M1.A shell integration emits Plain-encoded DCS frames, so a command (or `$PWD`) containing
`;` is truncated at the first `;` and grows phantom fields (`ls; pwd` → Block command "ls"), and
a raw ESC can corrupt the DCS scan. The decode side already ships AnsiCQuoted
(SPEC-terminal-blocks R9/R20). Fix emit-side: `marley_zsh_init()` switches the dynamic frames to
the `q` selector with an in-rc zsh escaping chain (`\` `;` ESC `\n` `\t` `\r`), so every command
round-trips exactly into its Block. Emit-side only; no decode behavior change.

## Acceptance
A command containing `;`, `=`, quotes, and spaces run in the integrated shell produces a Finished
Block whose `command` (and staged prompt `pwd`) round-trip EXACTLY, proven by the extended
`#[serial]` real-zsh integration test; rc-text unit tests pin the selector + escaping chain;
cov 100 / MSI 100 on touched; FULL gate GREEN [--diff]. Full EARS in the pipeline spec.
