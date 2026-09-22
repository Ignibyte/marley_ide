# TICKET-294 — cwd↔editor link (open a terminal at the editor file's dir)

- **Forge ticket:** #294 `bb15ec7a-a875-45d8-8452-1c0523605b37` (feature, M18)
- **Owner:** autonomous /goal run (sprint #31 — M18 Terminal↔Editor Fusion) — THE LAST GOAL TICKET
- **AAR:** `3391a3fb-5bc6-4982-856a-977eb1b098b5`
- **Pipeline doc:** ../../pipeline/active/294-cwd-editor-link.spec.md
- **Status:** closed

## Summary
A small bidirectional cwd link between the editor and the terminal. **(a)** "Open
Terminal Here" spawns a new terminal tab rooted at the ACTIVE editor file's
directory (pure `dir_of` + the #281 `valid_dir_or`/`spawn_session_in`). **(b)** "cd
Terminal to Editor Dir" writes a shell-safe `cd <dir>` to the focused terminal
(pure `cd_command` + the shipped `write_command`). Both are cockpit palette commands.

## Acceptance
With a file open in the editor, invoking "Open Terminal Here" spawns a new terminal
whose pwd is the file's directory. Full EARS in the pipeline spec.
