# TICKET-479 — Attach a file to an agent's prompt

- **Ticket:** LOCAL #479 (feature, prong 1: T7c)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/479-attach-a-file.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T7)
- **Status:** closed

## Summary
Warp's agent bar has a + that attaches a file to the prompt. For a CLI agent that comes down to
the file's location: Chad, "basically just provides the location of a file after choosing it in
the claude session". The agent bar gets an Attach File button that opens a file chooser and
types the chosen files' paths into the terminal, the way Zed already types a dropped file's
path.

## Acceptance
Choosing files sends their absolute paths, quoted as a drop quotes them; cancelling sends
nothing. The EARS criteria are in the spec.
