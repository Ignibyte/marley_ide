# One harness per host — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-740-one-harness-per-host.md
- **Pipeline spec:** 740-one-harness-per-host.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - #534 (sessions in the rail), #632 (the embedded harness), #689 (writes), #691 (the seat form), #694 (the Manager), #710 (stop and remove).
  - rustal-harness D164: Marley reaches each other box over its own SSH connection; the harness does not join hosts into one fleet.
  - `Harness::seat_command` guesses the base from the position of `mcp`; the new entries name host, root and program instead.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.

## Phase 1 — Plan (promoted 2026-10-10)
- **Pre-flight:** no active pipeline; README marker present; cargo idle.
- **Brain:** no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **Seams re-verified:**
  - `harness.rs`: `Harness` (137, one global: `source`, `connection`, `runtime`, `seats`,
    `server`, `run`, the manager's fields); `follow_setting` (499); `follow` (752), `connected`
    (790) and `seed` write the global's `server`, `connection` and `seats`; `follow_thread` and
    `sync_manager_entry` (the Manager, #688, #694); `open_in_home`, `open` (1202-1235);
    `HarnessView` (1240, `id`, its calls through `Harness::server`); `seat_command` (225).
  - `harness_seat.rs`: `Harness::seat_command` at 123, 211 (agents' tools) and 405 (the form);
    `open_in_home` after a seat starts (438).
  - `rail.rs`: `render_harness` (4491), `harness_row` (6716), `harness_entries` (6870),
    `InboxTarget::Harness(id)` (1658, 1963, 2288, 6886), `open_harness` (2005).
  - `marley_remote::{parse_ssh_target, ssh_command, keepalive_options}`: a destination that starts
    with `-` is refused, and `--` ends ssh's options.
  - `settings_content/src/marley.rs`: `harness`, `embedded_harness`, `harness_writes`; content
    structs derive `Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema,
    MergeFrom` under `#[with_fallible_options]`.

### Design
- **`settings_content/src/marley.rs`** (Zed crate, its row extended): `harnesses:
  Option<Vec<MarleyHarnessHostContent>>`, `MarleyHarnessHostContent { name, ssh, state, rh }`.
- **`marley_workbench.rs`:** `MarleySettings.harnesses`: the entries with a name and a root, the
  first of each name.
- **New `harness_hosts.rs`** (Marley crate): `HarnessHost { name, ssh: Option<SshTarget>, state, rh
  }` with `mcp_command(writes)` (`ssh -T -o BatchMode=yes <keepalive> [-p] -- HOST` and the quoted
  remote words, or `rh` here) and `seat_base()` (the same without `mcp`); the `Hosts` global, one
  `Followed { host, connection, seats, server, run, folded }` per entry, in the settings' order;
  `follow_hosts` on every settings change starts, keeps or drops each follow.
- **`harness.rs`:** `Slot { Primary, Host(name) }`. `follow`, `connected` and `seed` take a slot
  and write through `set_server`, `set_connection`, `set_seats`, `apply_events`; the Manager's
  thread and entry stay the primary's. `server_of(slot)`, `seats_of(slot)`,
  `seat_command_of(slot)`. `HarnessView` keeps its slot; `open` and `open_in_home` take one.
- **`harness_seat.rs`:** the form's Harness choice while hosts are followed (a button that cycles
  This machine and each host); the seat's commands and its tab use the slot.
- **`rail.rs`:** `render_harness` draws the primary section, then one section per host, "HARNESS ·
  name" with its own line and fold; `harness_row`, `harness_entries`, `InboxTarget::Harness` and
  `open_harness` carry the slot.
- **The guide:** "Several harnesses".
- **File manifest:** `settings_content/src/marley.rs` (Zed crate); `harness_hosts.rs` (new),
  `harness.rs`, `harness_seat.rs`, `rail.rs`, `marley_workbench.rs` (Marley crate); the ledger row;
  the guide; the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | `marley.harness` names root A's `rh … mcp`; `marley.harnesses` names `box-2` (`ssh: box-2`) on root B through a fake `ssh`; one session on each | 740-01-two |
| 002 | `marley: new harness seat`, Harness cycled to box-2, Create | 740-02-seat, the fake ssh log's `seat add` |
| 003 | Root B's runtime stopped | 740-03-one-down |
| 004 | — | The review of the diff |

### Risks
- **A fixture harness.** The scenario needs two harness roots of the built `rh`, as #710's does,
  and sessions in them; a stand-in `rh` answering `fleet_snapshot` may be simpler if a real
  root's session needs an agent. Settled in Test.

## Phase 2 — Code (2026-10-10)
- **Built:**
  - `settings_content/src/marley.rs`: `harnesses: Option<Vec<MarleyHarnessHostContent>>` and
    the struct (ledger row extended first).
  - `harness_hosts.rs` (new): `HarnessHost::from_content` (named, rooted, first of each name; an
    `ssh` that is no destination drops its entry with a log line), `base()` (`rh --state ROOT`, or
    `ssh -T -o BatchMode=yes <keepalive> [-p PORT] -- HOST rh --state ROOT` with the remote words
    quoted by `shell_word`), `mcp_command(writes)`; the `Hosts` global of `Followed { host,
    writes, connection, seats, server, folded, minute, _run }`; `follow_hosts` on every settings
    change keeps an unchanged entry's connection, restarts a changed one and drops one no longer
    named.
  - `harness.rs`: `Slot { Primary, Host(name) }`; `seats_of`, `server_of`, `seat_command_of`,
    `writes_on_slot`, `set_connection`, `change_seats`, `bump_minute`; `follow`, `connected` and
    `seed` take the slot; the Manager's thread is followed for the primary only; `HarnessView`
    keeps its slot (reads, answers, sends, controls, its observer) and its tab says `title ·
    name`; `open` and `open_in_home` take the slot; the palette lists New Harness Seat while any
    followed harness takes a seat.
  - `harness_seat.rs`: the form's Harness row (Default and each host) while more than one harness
    takes a seat; the seat's commands go through the chosen harness and its tab opens on it.
  - `rail.rs`: one section per harness (HARNESS, then HARNESS · name), each with its own line,
    fold and rows; rows, inbox entries and `open_harness` carry the slot; the rail also observes
    `Hosts`.
- **Deviations:**
  - No `MarleySettings.harnesses`: `follow_hosts` reads the merged settings as `harness_writes`
    does. Adding the field put `from_settings` over clippy's line limit, and nothing else reads it.
  - The seat form's choice is a row of buttons, not one that cycles: every harness is in sight.
- **Review found and fixed:**
  - `ask_lines` still read the primary's server: a host session's tab would have read the wrong
    harness. Now `server_of(slot)`.
  - Seat words cross the host's shell when it is over ssh: a folder with a space would split.
    `seat_command_of` says when, and `Seat::commands` quotes each word then.
  - Two harnesses may hold a session of the same id: host rows' element ids now carry the name.
  - The primary's inbox key kept its old shape (`harness:ID:ASK`), so dismissed entries stay
    dismissed.
  - Re-entrancy: nothing reads an entity under update; `Hosts` is a global changed only inside
    `cx.update` or a settings observer.
  - Provenance: nothing from Warp; no Zed function body carried over.
- **Checklist:** settings content ✓, harness_hosts.rs ✓, harness.rs ✓, harness_seat.rs ✓, rail.rs
  ✓, marley_workbench.rs ✓, ledger row ✓, scenario `script/e2e/740-one-harness-per-host.sh` ✓;
  the guide at Complete.
- **Gate:** `script/gates.sh --diff` GREEN, 17 passed (first run red on gate:14: the module doc
  linked to crate-private `Slot` and `Hosts`; now code spans).

## Phase 3 — Test (2026-10-10)
- **Scenario:** `script/e2e/740-one-harness-per-host.sh` (sway). Two runtimes of the built `rh`
  on scratch roots under `$XDG_RUNTIME_DIR`, `alpha` in A and `beta` in B; `marley.harness` follows
  A through #710's stand-in `rh`; `marley.harnesses` names `box-2` (`ssh: box-2`, root B); a fake
  `ssh` first on the PATH logs its arguments and runs the words after the destination in a shell;
  `marley.harness_writes` on.
- **Runs:**
  1. Red: the box-2 button's place was a guess, the seat went to A. The form shot showed the
     Harness row at y 205, box-2 at x 625; set as the defaults.
  2. Both checks pass; shot 03 read "connecting" over box-2's stale rows: after a failure the loop
     set `Connecting` for each retry, hiding the reason most of the time. Fixed (below).
  3. Both checks pass, every shot as below.
- **Shots (run 3):**
  - `740-01-two` (REQ-001): HARNESS connected, `alpha` working; HARNESS · box-2 connected, `beta`
    working. The ssh log holds `-T -o BatchMode=yes -o ServerAliveInterval=5 -o
    ServerAliveCountMax=3 -- box-2 …/bin/rh --state <B> mcp --grant write`.
  - `740-02a-form`: the New Harness Seat form with a Harness row, Default and box-2, box-2 chosen
    (accent), `builder` in Name.
  - `740-02-seat` (REQ-002): the `builder · box-2` tab in the Home group reads "the seat is up"
    from box-2's server; the rail lists `builder` under HARNESS · box-2, none under HARNESS. The
    ssh log holds `seat add builder` and `seat start builder` on root B through `-- box-2`.
  - `740-03-one-down` (REQ-003): box-2's link killed and its host refusing: HARNESS · box-2 reads
    "not running: sending into a…" in red, `beta` and `builder` muted and "stale"; HARNESS still
    connected, `alpha` live. The open tab says its own read failed ("cancelled").
- **REQ-004** (by review): the tab's read, answer, send and controls go through `server_of(slot)`;
  the seat form through `seat_command_of(slot)`; shot 02's tab text came from box-2's server.
- **Fix:** `follow` no longer sets `Connecting` after its wait; the header keeps "not running:
  <reason>" until a try connects. The primary's header behaves the same now. Gate re-run GREEN
  (17 passed).
- **Not reached:** a real remote host: the fake `ssh` stands in for sshd's join of the words. The
  quoting of seat words with spaces (`shell_word`) is by review only.
- **Focus:** the run's Hyprland check: 1 Marley window before and after, no rule added.

## Phase 4 — Complete (2026-10-10)
- **Documented:** `CHANGELOG.md` (Added: Several harnesses); `docs/marley/guide.md` ("Several
  harnesses" under the harness's sessions, and the header keeping its reason);
  `docs/marley_architecture/marley_workbench.md` (Harnesses on other hosts);
  `docs/marley/workbench-shell.md` (#740's line); the `settings_content/src/marley.rs` row in
  `docs/marley/zed-touchpoints.md` names `harnesses` and `MarleyHarnessHostContent` as shipped.
- **Knowledge:** F-claude-740-a-host-sessions-tab-read-the-first-harness-001,
  F-claude-740-seat-words-split-in-the-hosts-shell-001,
  F-claude-740-a-down-harness-read-as-connecting-001,
  PR-claude-740-quote-each-word-after-an-ssh-destination-001,
  AD-claude-740-several-harnesses-sit-beside-the-first-001,
  L-claude-740-a-fake-ssh-stands-in-for-a-host-001.
- **Brain:** no `rusty` MCP server in this repository's sessions, so no consultation to close.
