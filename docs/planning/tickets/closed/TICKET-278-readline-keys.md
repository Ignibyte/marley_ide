# TICKET-278 — Readline keys at the cooked prompt

- **Forge:** #278 `672bb1f9-025c-43e7-8132-b9789b721dce` (sprint #30, M17)
- **Type:** feature
- **Status:** closed
- **Pipeline:** `docs/planning/pipeline/active/278-readline-keys.spec.md` (9d995524-0a91-4969-884e-565f13fdd07f)

## Summary
⌃A/⌃E/⌃K/⌃U/⌃W/⌃Y over Marley's own cooked prompt buffer: pure
`ReadlineOp` + `op_for_ctrl_key` + `apply_readline` (one kill slot on
`TerminalPane`, D2 empty-kill rule, the #257 word class for ⌃W). The
arm intercepts exactly those six chords BEFORE `input_route` (which
Raw-routes EVERY ctrl today — the ticket's premise corrected at plan),
gated on cooked conditions; the route and ⌃C/⌃D/⌃Z behavior stay
byte-identical. Editor ⌃-chords remain excluded by the #267 gate.

## Acceptance
Spec REQ-001..004: the exact chord map, the kill/yank arithmetic with
multibyte + empty-kill fenceposts, the headless cooked flows + editor
exclusion, and the running-command raw-stream negative.
