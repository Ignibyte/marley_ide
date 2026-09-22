# TICKET-193 — Hide the shell prompt input row while a foreground command runs

- **Forge ticket:** #193 (968fc19a-2615-4d41-b8c8-b506fc01416d) (feature, M12.1)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 4410fd5e-e0bd-46ef-ba7f-bb1f83097916
- **Pipeline doc:** ../../pipeline/active/hide-prompt-while-running.spec.md
- **Source ticket:** M12.1 "Cockpit polish & fixes" (sprint #24) — chad live-app feedback #5
- **Status:** closed

## Summary
When an agent (e.g. `claude`) runs in a pane, both the agent's own input area and
Marley's cooked ❯ prompt row are shown — two input areas. Hide Marley's prompt
input row whenever the pane's session has a foreground command running
(`is_command_running()`); keep the scrollback blocks visible; restore the prompt
when the command exits. Keep the viewport/content-row count consistent with the
render (hide the row AND drop it from the count) so scroll math cannot skew.

## Acceptance
Prompt row hidden while a foreground command runs and restored on exit; blocks
stay visible; the content-row count and the render gate derive from the same
`is_command_running` condition. Full EARS criteria (REQ-001..005) in the pipeline spec.
