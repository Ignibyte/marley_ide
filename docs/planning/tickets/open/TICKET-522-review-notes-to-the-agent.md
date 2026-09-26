# TICKET-522 — Review notes to the agent

- **Ticket:** LOCAL #522 (feature, prong 2, the review loop)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/522-review-notes-to-the-agent.spec.md
- **Source ticket:** Chad, 2026-09-25, on the Orca survey: "we will be taking what it does well and bring it in here" (`docs/orca_architecture/README.md`). Review notes back to the agent is the survey's item 11 and a new ticket it implies (report 02 §2.9 and §3 item 5); the Warp once-over kept the same idea, line comments sent to the CLI agent in one pass, for the review views (`docs/planning/design-notes/warp-once-over-2026-09-25.md`).
- **Status:** open

## Summary
Zed's diff views take review comments on lines and draw "Send Review to Agent (N)", but nothing in the tree handles the button's action (upstream deleted its handler in February), and the comments' Add Review button shows only under a feature flag that Marley's release build never turns on. Marley turns review comments on, and handles the action: it lists the agent terminals working in the diffed tree, each with whether it can take the notes now, and pastes the notes into the one Chad picks, in Orca's `File:` / `Line:` / `User comment:` format, then presses Enter, only while that agent is idle. The notes stay in the diff, each marked Sent, and the button counts only the unsent ones. When no agent can take them, Copy puts them on the clipboard.

## Acceptance
Review comments added in a project diff or a branch diff go, with Send Review to Agent, into an idle Claude Code terminal of that tree in Orca's format and are submitted; they stay in the diff marked Sent; a working agent, one asking for permission, or one Marley cannot read is refused with the reason.
