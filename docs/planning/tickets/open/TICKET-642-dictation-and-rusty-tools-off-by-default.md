# TICKET-642 — Dictation and Rusty's tools wait to be turned on

- **Ticket:** LOCAL #642 (feature, prong 1 T7d and prong 2 C2: their defaults; design note Part 3,
  V1)
- **Owner:** claude-opus-5-5, 2026-10-02 (/spec)
- **Pipeline doc:** ../../pipeline/queued/642-dictation-and-rusty-tools-off-by-default.spec.md
- **Source ticket:** `docs/planning/design-notes/herdr-and-hermes-2026-10-02.md`, Part 3
  (Switches, V1) and Chad's third round of answers: "Both off by default"
- **Status:** open

## Summary
Two Marley features switch on by themselves today: the agent bar's microphone (#480) shows
wherever `voxtype` is on the PATH, and `marley.rusty_tools` (#633) offers Rusty's MCP server to
Zed's agents wherever `rusty-mcp` is installed. Chad wants both off until the user turns them on
("i would like these to be enabled rather than by default because some may not want"). Dictation
gets a setting, `marley.voice.enabled`, the first key of the `marley.voice` block the design note
sketches, so later voice uses grow inside it without a migration. Off, the microphone is hidden,
Marley starts no `voxtype`, and `marley: toggle dictation` says dictation is off. `rusty_tools`
defaults to false. The Settings window's Marley page shows both, and a change applies while Marley
runs.

## Acceptance
With nothing set, the agent bar shows no microphone though Voxtype is installed, `marley: toggle
dictation` answers that dictation is off, Zed's agents get no `rusty` server though `rusty-mcp` is
installed, and the Marley page shows Voice and Rusty Tools for Agents off. Turning either on, or
off again, takes effect without a restart.
