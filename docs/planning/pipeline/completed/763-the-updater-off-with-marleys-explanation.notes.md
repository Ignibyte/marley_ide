# The updater off, with Marley's own explanation — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-763-the-updater-off-with-marleys-explanation.md
- **Pipeline spec:** 763-the-updater-off-with-marleys-explanation.spec.md

## Phase 1 — Plan (2026-10-11)
- **Request:** #445's split (Chad, 2026-10-11, "7.) yes"); its spec's Out item 4. Taken from the
  Queue under the goal "continue until all tickets are finished".
- **Classification:** chore; one Zed hunk (`auto_update`), a scenario. Size XS.
- **Recall (§18.3):** AD-claude-437 (the channel keys the updater; `dev` never polls);
  AD-claude-445 (the remote server's lookup on `dev` no longer reaches zed.dev); #502 (the install
  is how Marley updates). Brain: consultation f87160eae2f94f3d80277fe5a11b67ee, nothing on this seam.
- **Discovery:** `check` (`auto_update.rs:314-344`): Zed's explanation prompt when
  `ZED_UPDATE_EXPLANATION` is set, else a silent return when the channel does not poll, else
  `poll`. `init` starts polling only when the channel polls (`:280-305`). The app menu's item
  (`app_menus.rs:75`). `release_notes_url` on `dev` opens Zed's GitHub commits (Out).

### Design
- **`crates/auto_update/src/auto_update.rs` (Zed crate):** at the top of `check`, on `dev`, an
  Info prompt with Marley's title and steps, then return. The ledger row (#445's) extended.
- Docs: the guide's update paragraph; CHANGELOG.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | `auto update: check` from the palette | 763-01 |
| 002 | — | The review |

### Risks
- None of note: the prompt replaces a silent return.

## Phase 2 — Code (2026-10-11)
- **Built:** the ledger row (#445's, extended); in `auto_update::check`, a `dev` arm ahead of Zed's
  explanation: an Info prompt, "Marley updates through its install script", with the steps and OK.
  The guide's update paragraph; CHANGELOG.
- **Deviations from the design:** the steps quote "just install" instead of backticks, which the
  prompt drops (seen in the first Test run).
- **The review:** on `dev` `init` never starts polling (`poll_for_updates` is false), `check` now
  returns before `poll`, and #445's arms keep the remote server's lookup off zed.dev.
- **Clippy:** clean.
- **The gate:** `script/gates.sh --diff` green on its first run, 17 of 17.

## Phase 3 — Test (2026-10-11)
- **The scenario:** `script/e2e/763-the-updater-off-with-marleys-explanation.sh`, under
  `compositor sway`. Exit 0. Shot read:
  - `763-01-check`: `auto update: check` shows "Marley updates through its install script":
    "Pull Marley's checkout and run "just install" there, which runs script/install-marley. A
    Marley that is running keeps its build until you quit it and start it again." and OK
    (REQ-001).
- **Fix on the way:** the first run's text lost its backticks ("run just install there"); quoted.
- **What no scenario shows:** that no request reaches zed.dev (REQ-002), read in the diff.
- **Brain:** consultation f87160eae2f94f3d80277fe5a11b67ee.

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added, #763); the guide's update paragraph; the ledger row.
- **Knowledge appended:**
  AD-claude-763-marley-has-no-update-source-and-check-for-updates-says-so-001.
- **Brain:** the consultation closed with a decision.
- **Closed:** the ticket moved to `closed/`, the pipeline archived, committed. #445's five slices
  are all shipped.
