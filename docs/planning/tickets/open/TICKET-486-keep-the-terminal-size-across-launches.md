# TICKET-486 — The first terminals of a launch open at the last session's size

- **Ticket:** LOCAL #486 (bug, prong 1: T0)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** (minted at promotion)
- **Source ticket:** TICKET-485's known limit
- **Status:** open

## Summary
Since #485 a terminal opens at the size the last terminal view laid out, so its shell lays its
first prompt out for the width it will have. The terminals a launch restores or seeds open before
any view has a size, still at Zed's 100 × 6 debug size, and a multi-line prompt wider than 100
columns is misdrawn until the next prompt. Keeping the last size across launches (Zed's
key-value store, read off the main thread before the first terminal opens, written when the size
changes or when Marley quits) would open them at it too.

## Acceptance
The first terminal of a launch, in a window the size of the last session's, echoes a typed line
whole on a long two-line prompt (`script/e2e/485-long-prompt.sh`'s first shot).
