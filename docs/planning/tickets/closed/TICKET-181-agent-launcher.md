---
id: forge#181 (67543670-020d-4a13-87e3-c95d97a9cd72)
title: M12 — agent launch picker: ⌘⇧A opens a chooser (kind + initial prompt)
status: closed
milestone: M12 — The Agent Cockpit
pipeline: 784149f0-9731-441d-9d60-257ee8f9570d
---

⌘⇧A (and the 🧠 icon) opens a small picker: choose the agent kind (↑/↓ wrapping) + type an optional initial
prompt; Enter splits a pane, launches the kind, and delivers the prompt; Esc cancels. Pure AgentLauncherState
(mirrors PaletteState); shim overlay + key handling + a generalized launch_agent(kind, prompt).
