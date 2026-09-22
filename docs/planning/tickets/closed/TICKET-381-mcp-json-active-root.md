# TICKET-381 — `.mcp.json` resolved against process cwd: a Finder/`open` launch silently has no forge client

- **Forge ticket:** #381 (81348947-e095-46fb-b060-b90c30549c1b) (bug, M25)
- **Owner:** unclaimed
- **AAR:** pending-promotion
- **Pipeline doc:** ../../pipeline/queued/381-mcp-json-active-root.spec.md
- **Source ticket:** sprint #36 "M25 — App-Grade QA Hardening" — found in the 2026-07-21 live QA run.
  Ordering: #384 (misconfigured/missing-`.mcp.json` diagnosis UX) is queued BEHIND this ticket and
  reads its outcome — #381 fixes WHICH file is read; #384 then decides how a bad/absent one surfaces.
- **Status:** closed

## Summary
Boot reads `.mcp.json` from `std::env::current_dir()` (app.rs:1428-1430) — but a Finder/`open
target/Marley.app` launch starts with cwd `/`, so the read misses, `forge_client` is `None`, and on
a machine whose repo HAS `.mcp.json` the fleet writes fail with "no forge client (.mcp.json)"
(dispatch app.rs:6883-6896, question-answer :6938-6950 — the 2026-07-21 live QA finding). The same
cwd-read string also feeds #376's `endpoint_for_brain` bearer decision (:2208), so the live-wire
subscription silently degrades to an empty bearer too. Pre-existing since #64 (⌘⇧F has always been
dead under Finder launches, :3674); M24 raised the stakes. This is the
`PR-claude-boot-decisions-key-the-restored-active-root-001` class (the #376 inspect-F2 sibling): fix
by resolving `.mcp.json` against the restored ACTIVE project's root at the same post-shell-restore
placement #376 moved the subscription start to (app.rs:2192-2223), with the launch-cwd read kept as
the fallback so a dev `cargo run` never regresses. The `marley_forge_client` loopback wall and
bearer-iff-url-match invariants are pinned, not edited.

## Acceptance
Headline: launched via `open target/Marley.app` (process cwd `/`) with a restored active project
whose root holds `.mcp.json`, the app builds the forge client from the ACTIVE project's root — fleet
question-pick and dispatch-send reach the wire instead of recording "no forge client (.mcp.json)"
(the QA repro, live-driven) — while a dev `cargo run` from a project root behaves byte-identically,
a missing file degrades exactly as today, and `forge_endpoint_from`/`endpoint_for_brain` (loopback
wall, bearer-iff-url-match) ship unedited. The full EARS criteria live in the pipeline spec.
