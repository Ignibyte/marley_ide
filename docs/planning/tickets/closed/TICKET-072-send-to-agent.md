# TICKET-072 — send a line to a running agent

- **Forge ticket:** #72 `a358706a-f0d8-4a21-b62f-e4a7dd3d5781` (feature, M2.D seq-1)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `62e4730a-7fda-41cd-ba1a-66ad8dc0c8e1`
- **Pipeline doc:** ../../pipeline/active/send-to-agent.spec.md
- **Source ticket:** forge sprint #12 `c93f9693-5569-4208-bb6a-20d38afec99b` (M2.D — The Controlling Cockpit)
- **Status:** closed

## Summary
cmd-shift-s sends the focused pane's composed prompt line to the last-launched agent (+ clears the
prompt). PURE: `send_payload` (line + \r) + the keymap. SHIM: `RootView.last_agent` + the send dispatch
(a RAW write_bytes — correct for a running agent, inverse of #59/#65). cov/MSI 100 on send_payload +
keymap; the send is masked + self-test-verified. Deps #62 + #40 + #61.

## Acceptance
send_payload + keymap at cov/MSI 100 (line+\r; cmd-shift-s→send-to-agent); compose → cmd-shift-s → the
agent receives the line + prompt clears (self-test); FULL gate GREEN. Full EARS in the spec.
