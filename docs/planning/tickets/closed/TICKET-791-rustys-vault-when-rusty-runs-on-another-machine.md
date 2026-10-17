# TICKET-791 — Rusty's vault when Rusty runs on another machine

- **Ticket:** LOCAL #791 (feature, macOS)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** [791-rustys-vault-on-another-machine.spec.md](../../pipeline/completed/791-rustys-vault-on-another-machine.spec.md)
- **Source ticket:** seen while setting up the Mac, 2026-10-16. The Mac's Marley reaches Rusty
  on the Linux box through `connection: "service"` and an SSH tunnel to `127.0.0.1:4174`.
- **Status:** closed

## Summary
Marley reads Rusty's vault tree through the `brain_tree` tool, so the Brain view works from any
machine the service reaches. Two things still take the vault as a local folder,
`rusty::vault_folder` (Rusty's stored `brain_vault_path`, else `~/.rusty/brain`):
- opening a page as a file (`rusty/page.rs`, `page_file_in`);
- the folder Rusty's agent starts in when `marley.rusty.agent_folder` is unset (`assistant.rs`).

When the service runs on another machine, that path is the other machine's (on the Mac,
`/home/cpeppers/.rusty/brain`, which does not exist there). Marley needs either a setting for
where this machine sees the vault (a mount, or none), or the page opened as a remote file
through an SSH project on the service's host. Until then, the Mac sets `agent_folder` and a
page cannot be opened as a file.

## Acceptance
With Rusty on another machine, opening a brain page from the Brain view opens that page's
Markdown in an editor tab, and Rusty's agent starts without a missing folder.
