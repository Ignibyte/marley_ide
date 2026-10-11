# A quiet handed verb — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-775-a-quiet-handed-verb.md
- **Pipeline spec:** 775-a-quiet-handed-verb.spec.md

## Phase 1 — Plan
- **Request:** found in #749's visual check (`749-07-unknown-agent`): each handed verb printed
  Zed's start log line and "zed is already running" around #513's line.
- **Classification / tier:** chore, two Marley files, no Zed hunk.
- **Recall (§18.3):**
  - #513: the hand-off, its socket in the data directory, `socket_fits` (a path at `sun_path`
    starts without the check); PR-claude-check-a-unix-socket-path-against-sun-path-001.
  - #749: `args_if_invoked` runs before `Args::parse`, after `keep_activation_token`, so the token
    is already kept; `Invoked::Exit` prints in `main`.
  - `paths::set_custom_data_dir` panics after `data_dir` was read: the intercept must not read
    `paths::data_dir()` when the launch names a custom directory.
  - The brain: nothing on this seam.
- **Discovery:** `crates/marley_workbench/src/commands.rs` (`args_if_invoked`, `read_launch`),
  `crates/marley_workbench/src/single_instance.rs` (`socket_path`, `socket_fits`, `hand_off`,
  `send`), `crates/zed/src/main.rs` 312-321 (the logger), 393-416 (the check and its lines),
  `crates/paths/src/paths.rs` 104-150.
- **Decisions:** spec D1-D4.

### Design
- `single_instance.rs`: `socket_in(data_dir)`; `send(paths_or_urls, socket)` takes the socket;
  `hand_off` passes `socket_path()`; a new `pub(crate) fn hand_verb(url, data_dir) ->
  Result<Option<String>>`: `Ok(None)` when the socket is too long or nothing listens (`connect`
  gives `NotFound` or `ConnectionRefused`), `Ok(Some(line))` once sent, `Err` otherwise. The line
  is `hand_off`'s, by a shared `handed_line(data_dir, urls)`.
- `commands.rs`: `read_launch` stays pure; `args_if_invoked`, for `Launch::Run`, resolves the data
  directory (custom canonicalized, else `paths::data_dir()`), and on Linux and FreeBSD calls
  `hand_verb`: `Some(line)` → `Exit { 0, line }`, `None` → `Launch`, `Err` → `Exit { 1, … }`.
- **File manifest:** both in `crates/marley_workbench` (Marley crate). No Zed file.

### Visual check plan
| Criterion | The scenario | Shot |
|---|---|---|
| REQ-001 | #749's PATH setup; in the repo's terminal `marley home > out.txt 2>&1; cat out.txt`; back in the terminal; `expect` out.txt holds one line naming `marley home` | `775-01-quiet` |
| REQ-002 | quit; `launch_marley_with home` | `775-02-cold` |
| REQ-003 | not reachable without breaking a socket by hand; the review checks the arm | — |

### Risks
- A socket file left by a Marley that died: `connect` is refused, so the launch starts Marley,
  whose own check then finds the stale socket as #513's does.
- The intercept reads the default data directory before `main`; `main` reads the same `OnceLock`
  later and gets the same path, since nothing between changes the environment it reads.

## Phase 2 — Code
- **Built:** `single_instance.rs`: `socket_in(data_dir)`; `hand_off` ends in `handed_line`;
  `send` keeps its connect and passes the URLs to `with_token` and `deliver` (the length check
  and the sends), which `hand_verb(url, data_dir)` shares: `None` for a socket path at or over
  `sun_path`, or a `connect` that finds no socket or is refused; the line once sent; an error
  otherwise. `commands.rs`: `args_if_invoked`'s `Launch::Run` arm calls `hand_to_running` on Linux
  and FreeBSD (a custom directory canonicalized without being made, a missing one meaning no
  Marley; else `paths::data_dir()`), answering `Exit { 0, line }`, `Exit { 1, error }`, or
  falling through to `Launch`.
- **Deviation:** none.
- **Review of the diff:** REQ-003's arm (a send that fails on a listening socket) answers exit 1
  with the error; the activation token still goes first; the default directory is read only when
  the launch names no custom one, so `set_custom_data_dir` never meets an initialized directory.
- **Clippy:** `deliver` took its URLs by value; now a slice.
- **Gate:** see Phase 3 (one run over the code and the scenario).

## Phase 3 — Test
- **Scenario:** `script/e2e/775-a-quiet-handed-verb.sh` (`compositor sway`), one run.
- `775-01-quiet` (REQ-001): the Home tab opened in the repo group; the terminal shows `cat
  …/out.txt` and one line, "Marley is already running on /run/user/1000/marley-e2e/profile.…; it
  was asked to run \`marley home\`."; `check the handed verb printed one line: pass`.
- `775-02-cold` (REQ-002): after the quit, `launch_marley_with home` started Marley: the session
  back (the repo group with its terminal) and the Home tab in Default, since the launch's folder
  is the checkout, which no project holds (#749's D5).
- REQ-003 by the review: no scenario breaks a listening socket.
- **Gate:** the first run red at gate:14: the public module doc linked to the crate-private
  `hand_verb`; the link became plain code. The second run GREEN, 17 passed.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Changed: a handed command prints one line); `docs/marley/guide.md`
  (the `marley <command>` section's "No Marley running" bullet);
  `docs/marley_architecture/marley_workbench.md` (the #749 section); `docs/marley/workbench-shell.md`
  (the slice line). No Zed file.
- **Ledgers:** L-claude-775-read-the-data-directory-before-main-without-paths-001,
  L-claude-775-a-public-module-doc-links-only-public-items-001. No bug reached the app.
- **Brain:** consultation ae64a0fa…, `brain no-decision`: a follow-up inside #749's decision.
- Ticket closed, pipeline archived, committed.
