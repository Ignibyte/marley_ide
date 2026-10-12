---
pipeline_id: 941d502e-22c9-4162-bd91-43eca28d63a5
ticket: docs/planning/tickets/closed/TICKET-763-the-updater-off-with-marleys-explanation.md
status: Phase 4 — Complete PASS
title: The updater off, with Marley's own explanation
type: chore
slice: packaging, the release identity (TICKET-445), slice 5 of 5
references:
  - docs/planning/pipeline/completed/445-marley-release-identity.spec.md
  - docs/planning/pipeline/completed/502-installed-release-build.spec.md
---

## Title
On Zed's `dev` channel the updater never polls, and Check for Updates (the app menu,
`auto update: check`) does nothing at all: `check` returns without a word
(`auto_update.rs:314-334`). Zed's packager route, `ZED_UPDATE_EXPLANATION`, would show a prompt
titled "Zed was installed via a package manager." This slice gives `dev` Marley's own answer:
Marley updates through its install script, with what to run. The last of #445's five slices
(#445's spec, Out item 4).

## Scope
### In
- **Check for Updates on `dev`** shows a prompt: "Marley updates through its install script",
  with "Pull Marley's checkout and run "just install" there, which runs script/install-marley. A
  Marley that is running keeps its build until you quit it and start it again." and OK.
- **Nothing reaches zed.dev for an update** on `dev`: polling stays off
  (`ReleaseChannel::poll_for_updates`), `check` returns before `poll`, and #445 took the remote
  server's download off zed.dev.
- Docs: the guide's update paragraph, CHANGELOG.

### Out (explicitly deferred)
- An update source of Marley's own: none exists; the install script is the way.
- Release notes on `dev` (`release_notes_url`, Zed's GitHub commits): not an update, and Marley's
  changes are in its CHANGELOG.
- The `ShowUpdateNotification` debug action on `dev`, which only draws Zed's notification.

## Reference (§20)
Upstream Zed, the `auto_update` crate: a packaged Zed sets `ZED_UPDATE_EXPLANATION`, and `check`
shows it in an Info prompt instead of polling (`auto_update.rs:314-324`). Marley keeps the shape
(an Info prompt, OK) with its own title and words on its own channel.

### Prior art
- **The code we ship:** `auto_update::check` (`auto_update.rs:314-344`), `init`'s polling
  guard (`:280-305`), `ReleaseChannel::poll_for_updates` (false on `dev`), the app menu's
  "Check for Updates" (`app_menus.rs:75`), #445's `dev` arms in the same file.
- **Behavior maps:** none.
- **Published material:** none needed.

## UI proof
`script/e2e/763-the-updater-off-with-marleys-explanation.sh`, under `compositor sway`.
- `763-01-check`: `auto update: check` from the palette shows the prompt "Marley updates
  through its install script" with the steps and OK.

## Locked-In Decisions
- **D1 — An arm for `dev` in `check`, ahead of Zed's.** Rejected: setting
  `ZED_UPDATE_EXPLANATION` at build time (its title says Zed and a package manager).
- **D2 — No update source.** Marley updates from its checkout; a source of its own is not planned.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Check for Updates runs on the `dev` channel, the system shall show a prompt saying Marley updates through its install script and what to run. | Shot 763-01 |
| REQ-002 | On the `dev` channel the system shall make no request to zed.dev for an update. | The review (`poll_for_updates`, `check`'s return, #445's arms) |
| REQ-003 | The change shall pass `script/gates.sh --diff`. | The gate |
