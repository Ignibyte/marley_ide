# The spec batch of 2026-09-25

On 2026-09-25 Chad answered the Orca survey's questions and the Warp once-over's seven items, and
asked for everything decided to be specced ("Lets spec out everything from the orca and warp that
we decided on now"). Seven drafting agents wrote the queued pairs overnight from the survey
(`docs/orca_architecture/`), the once-over note, and the code; the lead added #513 and #517,
checked every pair for a real `pipeline_id`, a filled Reference and Prior art, a UI proof and EARS
criteria, and removed what the public repository must not carry. Nothing here is built yet: each
pair is promoted by `/pipeline:plan`, which re-verifies its seams (line numbers moved while #516
landed) and asks the brain.

## What was decided, and where it went

| Decision (Chad, 2026-09-25) | Tickets |
|---|---|
| Telemetry off by default, and a Marley settings page | shipped: #514, #515 |
| Warp item 2, secret redaction ("love it need it") | shipped: #516 |
| One Chromium per project; worktrees share their project's | #507 |
| Worktree agents under their project in the rail | #510 |
| Merging lives in the Rustal workflow where it runs; a merge commit elsewhere, never a push | #511 |
| Agents keep their own permission prompts; Claude's bypass or Codex's full access as a setting | #532 |
| Marley's own regression suite | #517 |
| A native phone app eventually, direct or through a small Rust relay | #535 (Deliberate), with the later slices in its notes |
| Saved Playwright scripts inside Marley; the browser exposed to trusted outside clients | #523, #524 |
| Warp items 1, 3 to 7 | #525, #526, #527, #528 and #529, #530, #531 |
| The list after the browser waves ("lets do 1 through 8"), redrafted with the survey's findings | #503 to #511 |
| The survey's other takes: hook events, terminal identity, a fuller pick, ports, resume, review notes, notifications, attention order, remote terminals, the harness contract, a spawn ratchet, agent-aware copy and paste, git prompts off, the user agent | #519, #520, #518, #521, #540, #522, #538, #542, #543, #533 and #534, #541, #536, #537, #539 |
| Found tonight | #513 (one Marley per data directory, from #502), #544 (blocks across a rewrap, from #516) |

## Order

The Queue in `BACKLOG.md` holds the order: the two fixes and the regression suite first, then
#519 and #520, which later tickets build on, then the browser work, the agent work, the worktree
pair, and the terminal items. #534, #535 and #540 are Deliberate, each with its reason.

## Open questions the pairs carry

- #506 D3: keep text typed into ordinary fields in a recording, which relaxes #499's rule that no
  typed character is kept (password, hidden, one-time-code and card fields keep counts only).
- #524: slice 1 leaves Chromium's DevTools port open without a credential, as today; slice 2
  moves Chromium to `--remote-debugging-pipe` behind a relay. The pair defaults to slice 1 first.
- #537: ssh's own passphrase and host-key prompts stay on by default; `GIT_SSH_COMMAND` with
  `BatchMode=yes` would stop them but override `core.sshCommand`.
- #535: which projects may push events to a phone, and when the phone path starts.
