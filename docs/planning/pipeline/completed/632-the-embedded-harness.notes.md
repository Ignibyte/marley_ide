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

## Phase 1 — Plan (promoted 2026-10-01)
- **Recall (§18.3):** the queued entry above; AD-claude-534; L-claude-534 (the client sees no
  exit, so the runtime's own state must come from the child); gate:22's list. Brain (consultation
  1917a9141e6d465c986c9cfe1e31e426): nothing on this seam.
- **Re-verified:** `process::follow` pipes stdout, closes stderr and kills its child on drop;
  `voice.rs` finds Voxtype with `which` off the main thread and reads `follow`'s lines; e2e's
  `launch_marley` passes the scenario's exported environment to Marley, so `MARLEY_RH` set in
  `setup` reaches it; a scenario's data dir is its profile, so the root is `$E2E_PROFILE/harness`
  (its socket path about 60 bytes).

### Design
- **Setting:** `marley.embedded_harness: Option<bool>` (`settings_content`, `MarleySettings`,
  `default.json` false with its comment). No page toggle: the harness settings have no section
  yet.
- **`process.rs`:** `follow_with_errors(program, args) -> Child`, `follow` with stderr piped.
- **`harness.rs`:** the command's source, `Source::{Off, Command(cmd), Embedded}` from the
  settings, `marley.harness` first. `Embedded` runs `embed`: find `rh` (`MARLEY_RH`, else
  `which::which("rh")`), the root `paths::data_dir().join("harness")` (a socket path over 107
  bytes refused with that reason), then loop: start `serve`, read stdout lines for the ready line
  within 10 s while stderr's last line is kept, set `Runtime::Running` and, the first time, start
  #534's `follow` with `rh --state <root> mcp`; await the child's end, set `Runtime::Stopped`
  (the exit and stderr's last line), wait 1, 2 … 60 s and serve again. A `serve` whose error says
  another runtime owns the root counts as running. `Runtime` lives in the `Harness` global; the
  rows are stale while the connection is not up or the runtime is not running.
- **`rail.rs`:** the header reads the runtime's state before the connection's: `rh not found:
  …`, `starting the runtime`, `runtime stopped: …`.

### File manifest
- Marley: `crates/marley_workbench/src/harness.rs`, `process.rs`, `marley_workbench.rs`,
  `rail.rs`; `script/e2e/632-the-embedded-harness.sh`, `632-the-embedded-harness-missing.sh`.
- Zed: `crates/settings_content/src/marley.rs`, `assets/settings/default.json` (rows widened
  first).

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | `MARLEY_RH` at the harness's `bin/rh`, the setting on | `632-01-connected` |
| REQ-002 | `rh --state $E2E_PROFILE/harness actor new` | `632-02-session` |
| REQ-003 | the `serve` child killed; Marley's restart | `632-03-stopped`, `632-04-back` |
| REQ-004 | the second scenario, `MARLEY_RH` naming no file | `632-05-missing` |
| REQ-005, REQ-006 | review; the teardown finds the actor's workspace running | the diff, the log |

## Phase 2 — Code (2026-10-01)
- **Built:** `marley.embedded_harness` (`settings_content`, `default.json`, `MarleySettings` as
  `EmbeddedHarness`); `process::follow_with_errors` (stderr kept), `follow` and it sharing one
  spawning function; in `harness.rs`, `Source` (`marley.harness` first, then the embedded
  runtime), `Runtime` (starting, running, stopped, missing), `embed` (find `rh`, the root under the
  data dir with its socket path checked, `serve` in a loop with a growing wait), `serve` (the ready
  line within 10 s, then following `rh --state <root> mcp` once, then the exit and stderr's last
  line), and the lock-taken case followed as another runtime's; in `rail.rs`, the header reading
  the runtime first and the rows stale while it is not running.
- **Deviations:** no page toggle (the harness settings have no section yet); `rh compatibility` is
  not run: the ready line confirms the binary.
- **Review:** dropping the source's tasks ends `serve` (its child dies with its handle) and the MCP
  client; the harness's tmux and sessions are never stopped; the runtime's state is taken mutably
  only on a change; a `serve` that never gets ready is killed after 10 s, one that ends before it
  is not.
- **The gate found:** gate:21, an `async` block with no `.await` around the `rh` lookup (now
  `future::lazy`); gate:22, an eighth spawn call (the new function now shares `follow`'s, so the
  pin stays at 7).
- **Gate:** GREEN, 17 PASS.

## Phase 3 — Test (2026-10-01)
- **Scenarios:** `script/e2e/632-the-embedded-harness.sh` and
  `632-the-embedded-harness-missing.sh` (sway). `MARLEY_RH` and an unset `TMUX` are exported in
  `setup`, which Marley inherits; the root is the run's `$E2E_PROFILE/harness`.
- **First run:** every state showed, but the header cut both long reasons
  (`rh not found: MARLEY_RH names…`) before the part that says where Marley looked.
- **Fix:** shorter reasons (`no rh at MARLEY_RH: <path> is not there`, `no rh on the PATH, and
  MARLEY_RH is unset`) and a tooltip on the header with the whole of it. Gate GREEN, 17 PASS; both
  scenarios run again on that build.
- **Shots (final build):**
  - `632-01-connected` (REQ-001): `HARNESS connected`, no sessions: Marley served its own root
    and followed it.
  - `632-02-session` (REQ-002): `rh actor new worker` on that root: the `worker` row, `working`.
  - `632-03-stopped` (REQ-003): the root at 0755 and `serve` killed: `runtime stopped: exit 1: rh:
    state ro…` (the next `serve` refused the root), the worker's row kept, `· stale`, a grey dot.
  - `632-04-back` (REQ-003): the root at 0700: Marley's next `serve` came up, `connected`, the
    worker `working` again (its tmux outlived `serve`).
  - `632-05-missing` (REQ-004): `MARLEY_RH` naming no file: `no rh at MARLEY_RH: /mnt/fast/tm…`,
    the tooltip holding the whole path.
- **Review only:** REQ-005 (`marley.harness` set: `Source::Command`, no runtime started; #534's
  scenario runs that path); REQ-006 (the setting off or Marley quitting drops `serve`'s child; the
  teardown found and stopped the worker's workspace, so its session outlived the runtime's
  restarts).

## Phase 4 — Complete (2026-10-01)
- **Documented:** `CHANGELOG.md`; the guide (the harness's section, `MARLEY_RH` in the variables
  table); `marley_workbench.md` (the harness module); the plan's C1 row; the ledger rows of
  `settings_content/src/marley.rs` and `default.json`.
- **Knowledge:** L-claude-632-a-new-spawn-in-a-listed-site-shares-its-spawn-call-001,
  AD-claude-632-marley-runs-the-harness-as-a-child-and-says-its-state-001. The cut reasons of the
  first run were a fault in this ticket's own new header, fixed before the commit (Phase 3).
- **Brain:** the consultation closed with `brain decide`.
