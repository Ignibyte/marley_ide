# Test runs list none of the machine's containers — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-669-test-runs-list-none-of-the-machines-containers.md
- **Pipeline spec:** 669-test-runs-list-none-of-the-machines-containers.spec.md

## Phase 1 — Plan (2026-10-06)
- **Request:** the Queue's top, filed from #668's visual check.
- **Classification / tier:** chore; a Marley setting (Zed crates `settings_content`,
  `settings_ui`, `assets/settings`, each with its ledger row), the rail, the e2e harness.
- **Pre-flight:** green; no active pipeline; cargo idle.
- **Recall (§18.3):**
  - F-634: the scan's real IO (`/proc`, `docker`, `systemctl`) failed 103 tests; `Ports::proc_root`
    is the unit tests' override, not a run's.
  - PR-614: a port of an engine's own unit is never stopped through it; container Stops go through
    the engine's `stop`.
  - L-603: a fake program put first on the PATH in `setup` reaches Marley's own spawns.
  - The brain (consultation `66fbf01f33564dbc804291a755ef39ad`): nothing on this seam.
- **Discovery:**
  - `ports.rs`: `container_ports` sorts by port; `attribute_containers` keeps a port whose
    Compose folder no project holds in `apart`, served by `Ports::containers`.
  - `rail.rs:4322` `render_containers`, the only reader; its rows stay out of the keys and the
    filter. The rail refreshes on every `SettingsStore` change (`rail.rs:831`).
  - `MarleySettings` (`marley_workbench.rs:353`), mapped in `from_settings` (`:596-640`);
    `settings_content/src/marley.rs` `rail_order` at `:136`; `default.json` `rail_order` at
    `:1762`; the Settings page's Rail Order item at `marley_page.rs:79-98`.
  - #614's scenario: its stand-in proxy is Perl with `$0` set, binding nothing; `APART_Y=338`
    assumed two machine containers.

### Design
- **`settings_content/src/marley.rs`** (Zed crate, Marley's file): `rail_containers:
  Option<bool>` after `rail_order`, documented, default true.
- **`assets/settings/default.json`** (Zed path): `"rail_containers": true` after `rail_order`,
  with a comment.
- **`marley_workbench.rs`** (Marley crate): `MarleySettings::rail_containers: bool`, from the
  content, `unwrap_or(true)`.
- **`rail.rs`** (Marley crate): `render_containers` returns `None` while it is off.
- **`settings_ui/src/marley_page.rs`** (Zed crate, Marley's file): Containers in the Rail, after
  Rail Order.
- **`script/e2e.sh`**: `marley["rail_containers"] = False` in the copy, a line in the comment.
- **`script/e2e/614-…`**: `profile_setting marley.rail_containers true`; `PORT=614`; `APART_Y`
  measured from its first run.
- **`script/e2e/669-…`**: the new scenario.
- **Ledger rows** (`docs/marley/zed-touchpoints.md`): the three Zed paths' rows gain #669 before
  their edits.
- **Guide**: the rail's ports article names the setting.

### Visual check plan
| Criterion | What the scenario does | Proof |
|---|---|---|
| REQ-004 | `setup` reads the run's copy | Check: `marley.rail_containers` is `false` |
| REQ-001 | Starts with the copy's setting off, with its stand-in proxy and the machine's containers | `669-01-off`: no Containers |
| REQ-002 | `profile_setting marley.rail_containers true` mid-run | `669-02-on`: Containers, port 669 first |
| REQ-003 | Opens the Settings window and searches Containers | `669-03-setting` |
| REQ-005 | Runs 614 | Its checks; `apart` and `refused` on its own row |
| REQ-006 | Runs 640 | `640-01-rows`, `640-03-tooltip` on build's row |

### Risks
- **A machine with a container on a port below 614** would put its row above #614's; the
  dev box's lowest is 8081.
- **A user who reads the scenarios' shots for the Containers list** now sees it only in 614 and 669.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall, the brain; discovery.
- [x] Mint the pair; the BACKLOG row removed; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request ("continue on tickets until finished").

## Phase 2 — Code (2026-10-06)
### Built
- **`settings_content/src/marley.rs`**: `rail_containers: Option<bool>` after `rail_order`.
- **`assets/settings/default.json`**: `"rail_containers": true` with its comment.
- **`marley_workbench.rs`**: `RailContainers { Listed, Hidden }`, `from_content` (listed unless
  off), `MarleySettings::rail_containers`.
- **`rail.rs`**: `render_containers` returns `None` while it is `Hidden`.
- **`settings_ui/src/marley_page.rs`**: Containers in the Rail after Rail Order; the Layout
  section's array grows to six.
- **`script/e2e.sh`**: the copy sets `marley.rail_containers` to `false`, with a line in the
  comment.
- **`script/e2e/614-…`**: `PORT=614` (the stand-in binds nothing), `profile_setting
  marley.rail_containers true`, the header and `APART_Y`'s comment.
- **The guide** (`guide/index.html`, `docs/marley/guide.md`) and the walkthrough name the setting;
  the three ledger rows name #669.

### Deviations
- **A two-variant enum, not a `bool`**: clippy's `struct_excessive_bools` (a fourth bool in
  `MarleySettings`) and `too_many_lines` (`from_settings` at 101); the file's idiom
  (`PromptEditor`, `EmbeddedHarness`, `CodexAppServer::from_content`) answers both.
- **The Settings page's Layout array**: its fixed size went from 5 to 6.

### Review
- Each Zed hunk adds one item beside its neighbours, with its ledger row; nothing upstream's is
  rewritten.
- `render_containers` is the only reader of the list; its rows stay out of the keys and the filter,
  so hiding it leaves nothing selectable behind. The rail refreshes on every settings change.
- `APART_Y` keeps its value until the Test phase's first run measures the first row under
  Containers.

### Gate
`just gate-diff`: first RED at gate:2 (the two clippy findings above); after the enum, GATE
GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-06)
### The scenario
`script/e2e/669-test-runs-list-none-of-the-machines-containers.sh` (`compositor sway`), on the
debug build after `just build`: a fake `docker` first on the PATH that lists nothing and records
any `stop`, a stand-in `docker-proxy` for port 669, and the dev box's 13 running containers. One
check, three shots; shellcheck and typos clean.

| Criterion | Proof | What it shows |
|---|---|---|
| REQ-004 | check: the run's copy turns the Containers list off | pass |
| REQ-001 | `669-01-off` | the rail holds `repo` and its terminal and no CONTAINERS section, with the stand-in and 13 containers running |
| REQ-002 | `669-02-on` | after `profile_setting marley.rail_containers true`, with no restart: CONTAINERS, `:669 docker → 172.18.0.69:80` first, then the box's in port order |
| REQ-003 | `669-03-setting` | the Settings window's Marley page searched for Containers: Layout, Containers in the Rail with its description, the toggle on |
| — | the run's log | nothing was stopped |

### 614 and 640
- **614** (REQ-005): `APART_Y` measured from `669-02-on`, where the first row under Containers sits
  at y 222 with the same rows above it; set from 338. `named`: `:614 web-614` under `repo`;
  `hover`: its tooltip naming container web-614; Stop recorded `stop web-614`; `apart`: `:614
  docker → 172.18.0.5:80` first under Containers; `refused`: the click on that row, the toast with
  the fake engine's refusal, `docker stop $(docker ps -q --filter publish=614)` and Copy Command.
- **640** (REQ-006): the Harness section in the window on the run's default copy. Its first run
  put `640-03-tooltip` on asker's row: `BUILD_Y=460` was measured when the box's two containers
  sat above the section. Measured again from `640-01-rows` (build's row centred at y 310) and set;
  the second run's `640-03-tooltip` is on build's row ("State from the agent's protocol",
  five_hour 62 %, seven_day 31 %, Account: work), and `640-04-moved` shows build at 80 % and
  asker reported, in Needs you.

### Focus
Each run, in headless sway: 0 Marley windows on Hyprland before and after, no rule added, sway
stopped with the run's Marley.

## Phase 4 — Complete (2026-10-06)
- **Documented:** `CHANGELOG.md` (Added: Containers in the Rail); `docs/marley_architecture/
  marley_workbench.md` (the container ports' note); the in-app guide, `docs/marley/guide.md` and
  the walkthrough; the three ledger rows in `docs/marley/zed-touchpoints.md` describe what shipped.
- **Knowledge:** `F-claude-669-scenario-coordinates-measured-with-the-machines-containers-in-the-rail-001`,
  `PR-claude-669-a-runs-rail-shows-only-what-the-run-started-001`,
  `L-claude-669-a-fourth-bool-in-marley-settings-is-a-two-variant-enum-001`,
  `AD-claude-669-the-machines-containers-in-the-rail-are-a-setting-off-in-every-test-run-001`.
- **Brain:** consultation `66fbf01f33564dbc804291a755ef39ad` closed with
  `decisions/marleys-rail-lists-the-machines-containers-behind-a-setting-that-every-test-run-turns-off`.
- **Ticket:** closed; the pair archived.
