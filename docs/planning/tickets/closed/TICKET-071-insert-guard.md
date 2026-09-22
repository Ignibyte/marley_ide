# TICKET-071 — guard finder/file-click prompt-inserts on is_command_running

- **Forge ticket:** #71 `1743b2b3-91e7-4fb8-9652-42f0821a08f3` (bug, Terminal Polish; BACKLOG)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `fda397ee-2a23-4a1c-bbe6-645b34f68c7c`
- **Pipeline doc:** ../../pipeline/active/insert-guard.spec.md
- **Status:** closed

## Summary
The cmd-P finder Enter (#65) + file-tree click (#59) insert the path unconditionally into the cooked
buffer; while a command runs it should reach the program. FIX: mirror the #42 paste guard — running →
`write_bytes`, else `buffer.edit`. SHIM-only (app.rs, masked). Deps #59 + #65 + #42.

## Acceptance
Both sites guarded on is_command_running (running→PTY, prompt→cooked); FULL gate GREEN. Full EARS in the spec.
