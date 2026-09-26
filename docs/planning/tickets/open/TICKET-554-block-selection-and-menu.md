# TICKET-554 — Block selection and the block menu

- **Ticket:** LOCAL #554 (feature, prong 1 T1: the block actions the bar names, item 3)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/queued/554-block-selection-and-menu.spec.md
- **Source ticket:** The Warp blocks note of 2026-09-25, recommendation 1 (`docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md`), specced because Chad decided on 2026-09-26 that every remaining Orca and Warp finding gets built. Open question 6 (whether the block keys select) had no answer; the spec takes Warp's binding.
- **Status:** open

## Summary
Marley has Copy Output and Rerun on a hovered block, keys that scroll from block to block, and
no way to choose a block with the keyboard or to act on one from the right-click menu, which is
Zed's and knows nothing of blocks. Warp selects a block with `Ctrl+↑`, moves the selection with
the arrows, clears it with Esc, and offers about fifteen actions on it. Marley keeps a selected
block per terminal view: `ctrl-up` selects the newest block (and moves up from there), `up`
and `down` move it, Escape or anything typed to the shell drops it, and an outline marks it.
A right-click on a block's rows adds a Block section to Zed's terminal menu with Copy Command,
Copy Output, Copy Both, Copy as Markdown (a fenced `$ command` and its output, then the cwd,
branch, exit code and duration on one line), Reinput (the command back at the prompt, unrun)
and Reinput with sudo. Reinput follows Rerun's rules: only a command the shell's hook reported
with the terminal's nonce, only while the shell waits at its prompt. Selection comes first
because every keyboard action Warp offers on a block needs it; #555 adds Send to Agent to the
same menu.

## Acceptance
`ctrl-up` outlines the newest block, `up` moves the outline, Escape and a typed character clear
it; a right-click on a block shows the Block section; each copy item puts the named text on the
clipboard, Copy as Markdown in its fenced form; Reinput leaves the command at the prompt with no
new block, Reinput with sudo the same with `sudo` in front; a block whose frame carried no nonce
offers neither.
