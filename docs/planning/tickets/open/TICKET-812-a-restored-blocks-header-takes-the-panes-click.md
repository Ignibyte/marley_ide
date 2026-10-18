# TICKET-812 — A restored block's header takes the click that should focus its pane

- **Ticket:** LOCAL #812 (bug, the block terminal)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** (set at Plan)
- **Source ticket:** found by #806's triage of 575-terminal-id-across-restore, 2026-10-17.
- **Status:** open

## Summary
A click on a block's header (#628) stops at the header (`block_headers.rs`, its mouse-down stops
propagation so no selection starts and the program gets no click), which also stops the terminal's
own focus on press. In a split, clicking a block header in the other pane leaves the focus where it
was: 575's second launch clicked the right pane on a restored block's header, and its command ran
in the left pane. The header should focus its terminal before it stops the press.

## Acceptance
A click on a block header in another pane focuses that pane's terminal.
