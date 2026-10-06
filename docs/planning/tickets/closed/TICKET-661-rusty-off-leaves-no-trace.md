# TICKET-661 — Rusty off leaves no trace

- **Ticket:** LOCAL #661 (chore, Rusty in Marley R-D0; Chad 2026-10-06: one build, one switch)
- **Owner:** claude-opus-5-5, 2026-10-06 (Chad: "we need to look into then adding the last things missing", after the R1 to R7b batch)
- **Pipeline doc:** ../../pipeline/completed/661-rusty-off-leaves-no-trace.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3, the slices table); the read-only survey of
  Rusty at `13249a8` and Marley on 2026-10-06
- **Status:** closed

## Summary
Chad chose one build with one switch for the personal assistant (2026-10-06). Today `marley.rusty.enabled` off starts no `rusty-mcp` and hides the Brain switch and the Knowledge panel's dock button, but every `rusty:` command (Knowledge panel, Tasks, Decisions, Graph, Open Page, Link a Page, Link a Task Group) and `marley::ToggleBrainView` stay in the command palette and answer with a toast, and the settings page's Rusty section shows its four items and the Rusty's Server link. With the switch off, Marley shall show no trace of the assistant but the switch itself: the palette filters the `rusty` namespace and the Brain toggle through Zed's `CommandPaletteFilter`, as Zed's `disable_ai` does (`agent_ui/src/agent_ui.rs:790-860`), and the settings page keeps only the switch. The plan's settled toast (rusty-in-marley.md R-D0) stays for a key bound in a user's keymap.

## Acceptance
With `marley.rusty.enabled` off, the command palette lists no `rusty:` command and no `marley: toggle brain view`, the settings page's Rusty section shows only its switch, and turning it on brings them all back without a restart.
