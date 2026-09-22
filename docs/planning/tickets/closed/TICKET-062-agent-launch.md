# TICKET-062 — launch + tag an agent pane

- **Forge ticket:** #62 `2ee9f90a-67c0-4a1e-889c-f99a78419a5f` (feature, M2.B seq-4)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `8a763e09-847a-471f-af02-bddf152e6e65`
- **Pipeline doc:** ../../pipeline/active/agent-launch.spec.md
- **Source ticket:** forge sprint #10 `4c988c99-56be-4434-8017-6909db864935` (M2.B — The Agent Cockpit)
- **Status:** closed

## Summary
cmd-shift-a launches `claude` in a new split pane, tagged as an AgentRun. PURE: `launch_command(kind)`
(marley_agent) + the keymap binding. SHIM (app.rs): dispatch new-agent → split_focused + spawn + run
the agent CLI + tag `RootView.agents`. cov/MSI 100 on launch_command + keymap; the launch is masked +
self-test-verified. Deps #61 + the workspace split.

## Acceptance
launch_command + keymap at cov/MSI 100 (Claude→"claude"/Codex→"codex"; cmd-shift-a→new-agent);
cmd-shift-a splits a pane + launches claude (self-test capture); FULL gate GREEN. Full EARS in the spec.
