---
pipeline_id: a61e9b26-1a9f-4d84-b407-0bf0ff5dac0d
ticket: docs/planning/tickets/closed/TICKET-775-a-quiet-handed-verb.md
status: Phase 4 — Complete PASS
title: A quiet handed verb
type: chore
slice: the Marley layout (docs/marley/workbench-shell.md), `marley <command>` (#749)
references:
  - docs/planning/pipeline/completed/749-marley-commands.spec.md
  - docs/planning/pipeline/completed/513-one-marley-per-data-dir.spec.md
---

## Title
A quiet handed verb. #749's visual check showed every `marley <verb>` typed in a terminal printing
Zed's log line "starting zed version …", #513's line, and Zed's "zed is already running": the
launch goes through Zed's `main`, which sends its log to the terminal when stdout is one and runs
the single-instance check only after that. The verb is handed to the running Marley in #749's
intercept itself instead, before Zed's `main` goes on, so a handed verb prints one line.

## Scope
### In
- `commands::args_if_invoked` hands the verb's URL to the Marley running on the data directory
  and exits with #513's line; only when no Marley answers does it answer `Launch`, as #749 does.
- `single_instance`: a hand-off to a given data directory that tells "no Marley runs there" from
  any other failure.

### Out (explicitly deferred)
- A plain `marley <path>` second launch keeps Zed's output: reading paths before Zed's parser
  would copy its flags (`--diff`, `--new`, `--wait` and the rest).
- macOS and Windows: Marley's hand-off is Linux and FreeBSD only (#513); elsewhere a verb
  launches as #749 does.

## Reference (§20)
N/A — Marley-specific: Zed's own second launch goes through its CLI binary, which Marley does not
ship (#513), and the verbs are Marley's (#749). Zed's behavior kept: a first launch's logging,
`set_custom_data_dir` and the single-instance check are untouched.

### Prior art
- **The code we ship:** `single_instance::hand_off` and `send` (#513), the datagram hand-off this
  reuses; `paths::set_custom_data_dir` (`paths.rs` 104), which panics once `data_dir` was read, so
  the intercept reads the directory the way that function makes it (created, canonicalized)
  without initializing `paths`; `paths::data_dir` for the default directory, safe to read early
  because `main` sets no custom directory then; `main`'s logger setup (`main.rs` 312-321), which
  sends the log to stdout when stdout is a terminal, before the check at 393.
- **Behavior maps:** none on this seam.
- **No dependency** owns it: `std::os::unix::net::UnixDatagram`'s `connect` errors
  (`NotFound`, `ConnectionRefused`) tell no listener from other failures.

## UI proof
Scenario `script/e2e/775-a-quiet-handed-verb.sh` (`compositor sway`): a scratch repo; `marley`
first on the terminals' PATH (#749's setup).

| Shot | Shows | Criteria |
|---|---|---|
| `775-01-quiet` | `marley home` in the repo's terminal: the Home tab opened, and the terminal, brought back, shows one line, "Marley is already running on …; it was asked to run `marley home`." | REQ-001 |
| `775-02-cold` | Marley quit; `marley home` launched: Marley starts with the session and the Home tab | REQ-002 |

## Locked-In Decisions
- **D1 — The intercept hands off.** `args_if_invoked` builds the URL as #749 does, then on Linux
  and FreeBSD sends it, the activation token first, to the data directory's socket. Sent: it
  answers `Exit { code: 0, message: <#513's line> }`, and Zed's `main` never runs past the
  intercept. No listener (no socket file, or refused): `Launch`, and the launch starts Marley.
  Any other failure: `Exit { code: 1, message }`.
- **D2 — The data directory without `paths`.** A custom directory (the launch's
  `--user-data-dir`, else `MARLEY_USER_DATA_DIR`) is canonicalized as `set_custom_data_dir` does it,
  without creating it: a directory that does not exist has no Marley. With none,
  `paths::data_dir()`.
- **D3 — The socket that does not fit** (`socket_fits`' rule, a path at or over `sun_path`): no
  check runs there (#513), so the launch answers `Launch` and the verb rides `main`'s hand-off as
  before.
- **D4 — One hand-off.** `hand_off` (the `main` path) and the intercept share `send`, which takes
  the socket path; the message names the verb as #749 made it.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a `marley <verb>` launch finds a Marley running on its data directory, the launch shall hand the verb over, print only "Marley is already running on <dir>; it was asked to run `marley <verb>`.", and exit 0. | shot `quiet`; the captured output holds one line |
| REQ-002 | WHEN no Marley runs on the data directory, the launch shall start Marley and run the verb, as #749's cold start does. | shot `cold` |
| REQ-003 | IF the hand-off fails other than for no Marley, THEN the launch shall print the error and exit 1. | review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `commands.rs`, `single_instance.rs`; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
