# The embedded harness — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-632-the-embedded-harness.md
- **Pipeline spec:** 632-the-embedded-harness.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-01)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything
  except the cloud flare ones" (wave 4); plan D19 (Chad, 2026-09-25: "The harness is embedded in
  Marley and also runs standalone").
- **Recall (§18.3):**
  - AD-claude-534 and L-claude-534-zeds-mcp-client-sees-no-server-exit-001: the follow loop and its
    limits.
  - An Explore read of the harness (2026-10-01): `serve` runs in the foreground and prints
    `{"ready":true,"runtime":…,"instance_id":…}`; it handles no signal, and killing it leaves tmux
    running; a second `serve` on a root fails ("another runtime owns this state root"); the root is
    absolute, a real 0700 directory owned by the user, its socket path under 108 bytes; `rh mcp`
    reads `fleet_snapshot`, `fleet_events` and `session_read` from the journal while `serve` is
    down; `shutdown` refuses while Codex or Claude sessions live; no install target, no LICENSE,
    `bin/rh` runs `target/debug/rh`; tmux is the default backend (`/usr/bin/tmux`, 3.7c here).
  - gate:22: a Marley crate spawns only from a listed file (`.config/spawn-sites.txt`,
    `SPAWN_SITES_PIN`); `process.rs` is listed.
- **Discovery:** the Explore reads above; promotion re-verifies.
