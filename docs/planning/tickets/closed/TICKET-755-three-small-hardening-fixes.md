# TICKET-755 — Three small hardening fixes

- **Ticket:** LOCAL #755 (chore, Marley's MCP server and the agent bar; from the T3 Code survey)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [755-three-small-hardening-fixes.spec.md](../../pipeline/completed/755-three-small-hardening-fixes.spec.md)
- **Source ticket:** [the T3 Code survey](../../../t3code_architecture/README.md), "Facts about Marley
  the survey turned up" and "Smaller things"; report 05 item 6, report 04 item 3. Chad,
  2026-10-11: "8.) yes to all".
- **Status:** closed

## Summary
- **Safe config writes.** The agent bar's setup rewrites Codex's `config.toml` with a plain write
  (`agent_notify.rs:292-302`): a crash halfway leaves a torn file, a dotfiles symlink stays a
  link only by luck, and nothing is kept to go back to. Marley writes it through a small helper:
  re-read and refuse when the file changed since it was read, keep one backup, write a temp file
  beside the real one with its mode, and rename it into place. OpenCode's plugin file takes the
  same path, without the backup.
- **A token per terminal.** Any client holding the MCP bearer can name any terminal in the
  `Marley-Terminal` header (`transport.rs:585`), and `terminal_list` hands every agent every id.
  Since #704 the editors area keys its approvals on that id, so an agent can borrow another
  terminal's Allow for This Session. Each local terminal gets `MARLEY_TERMINAL_TOKEN` beside its
  id; the bridge and `marley-edit` send it, and the server drops a terminal whose token does not
  match.
- **`OOMPolicy=continue` on Chromium's unit.** systemd's default stops a whole unit when the
  kernel kills one of its processes for memory, so one renderer killed under pressure closes
  every Browser tab of the project. One more `--property` on the transient unit.

## Acceptance
Codex's setup through a symlinked `config.toml` keeps the link and the file's mode and leaves a
backup; a terminal that claims another terminal's id is not taken for it; a project's Chromium
unit carries `OOMPolicy=continue`, and a renderer killed for memory leaves the project's other
tabs drawing.
