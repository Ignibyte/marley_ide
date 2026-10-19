# TICKET-816 — Keys typed right after Enter in the prompt editor reach the shell before it

- **Ticket:** LOCAL #816 (bug, the block terminal's prompt editor)
- **Owner:** 6bc788ef-5565-4849-9e73-e341a6348e9f
- **Pipeline doc:** ../../pipeline/completed/816-the-shells-enter-goes-with-its-paste.spec.md
- **Source ticket:** found by #806's triage of 561-browser-env-opener, 2026-10-19.
- **Status:** closed

## Summary
Return in a shell's prompt editor (#627) runs `rich_input::send`: it pastes the line into the
terminal, schedules the `\r` 200 ms later (`terminal_drive::paste_then`, `AFTER_PASTE`, from
#594's wait for agents' input), and at once gives the terminal the keys (`close`). A key typed
before the timer fires goes into the shell's line ahead of the Enter. In 561's run (regress
`20261019-101458`, shot `561-06`) `cd …/repo` and the next command ran as one line,
`cd …/repoBROWSER=/mnt`, exit 1, then the rest, exit 127; the run of 2026-10-17 met the same
with `cd …/elsewherepython3 -c …`. A person typing `cd x`, Enter, `ls` quickly can get `cd xls`.

## Acceptance
A command typed in the prompt editor and sent with Enter runs alone, whatever is typed right
after it; 561-browser-env-opener passes with its steps as they are.
