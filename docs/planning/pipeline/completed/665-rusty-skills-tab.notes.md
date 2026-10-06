# The Skills tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-665-rusty-skills-tab.md
- **Pipeline spec:** 665-rusty-skills-tab.spec.md

## Phase 1 — Plan (2026-10-06)
- **Request:** the Queue's top after #664, from Chad's 2026-10-06 "adding the last things missing".
- **Classification / tier:** feature, R8's second part; Marley crates only.
- **Pre-flight:** green; no active pipeline; cargo idle; 75 GB free on the build disk.
- **Recall (§18.3):**
  - AD-664 and #664's tab: the Decisions tab's `Link` and `ReadDue`, a modal form, Delete through
    `window.prompt`, a toggle group in a box of its own (L-664).
  - AD-662, AD-663: read again after Marley's own writes; the service connection announces nothing.
  - L-663: a stand-in tool that waits goes on its own thread; not needed here.
  - The brain (consultation `b420bfe4330741c094c83e158f0e8460`): nothing on this seam.
- **Discovery:**
  - Rusty at `eb1ab51`: `skill_list { include_pending }` answers `[Skill]` (`name`,
    `display_name`, `description`, `origin` user or auto, `status` active or pending, `path`, `body`)
    by name; `skill_view { name }` one, or `no skill named <name>`; `skill_create { name,
    description, body, pending, force }` answers the skill, refusing `invalid skill name "<n>": use
    lowercase letters, digits, and hyphens` and `skill "<n>" already exists (use --force to
    overwrite)`; `skill_update { name, description?, body? }` answers `{ skill, findings }`;
    `skill_delete` `deleted`; `skill_scan` the findings (`[String]`); `skill_approve { name, force }`
    `approved`, or `blocked by safety scan (<n> issue(s)): <findings> — re-run with --force to
    override`; `skill_reject` `rejected`; `script_list { include_pending }` answers `[Script]`
    (`name`, `skill`, `path`, `status`, `executable`) by name then skill; `script_view { name }`
    `{ script, text }` with `name` as `skill/name`; `script_update { name, body }` the script. Every
    write commits the store (TICKET-051) and emits `DataChanged`. Rusty's own `script_run` runs
    `bash <path>`.
  - The scan (`scan_skill_md`): over 200 lines, `allowed-tools` in the frontmatter, a bang-backtick
    marker, or a secret-looking marker.
  - Rusty's app: staged first then by name; Approve anyway only after findings; Run opens a terminal
    on the script's path.
  - Marley's terminal openers (an Explore map): `crate::agents::start_in_terminal` types a line into
    a new shell; Zed's `TaskTemplate` through `Workspace::schedule_resolved_task` runs one command in
    a center terminal titled by its `label`, as `remote.rs:170-213` opens ssh, routed to the center
    by `routing.rs`. Neither spawns in this crate, so gate:22's spawn sites are untouched.

### Design
- **`marley_rusty/src/skills.rs`** (Marley): the eleven tool names; `Skill` (`Deserialize`:
  `name`, `display_name`, `description`, `origin`, `status`, `path`, `body`) with `is_pending`,
  `is_auto`; `Script` (`name`, `skill`, `path`, `status`, `executable`) with `qualified()`
  (`skill/name`) and `is_pending`; `skills_from_answer`, `scripts_from_answer`,
  `findings_from_answer`, `ScriptText` and `script_text_from_answer`, `UpdateAnswer { skill,
  findings }`; `ordered(skills)` (staged first, then by name); `blocked_by_scan(error)` (whether a
  refusal is the scan's); `SkillWrite` (`Create`, `Update`, `Delete`, `Approve { force }`,
  `Reject`, `UpdateScript`) with `tool` and `arguments`.
- **`rusty/skills_tab.rs`** (new, Marley): `OpenSkills`; `open`, `open_later`; `BrainSkillsView`
  with the shared link and reads (`skill_list` and `script_list` with pending, both read together),
  `Chosen { Skill(name) | Script(path) }`, the left column (New Skill, Skills, Scripts) and the right
  side: for a skill the name, chips (pending, auto), the description editor, the body editor
  (`Editor::multi_line` in a box that fills the side), the findings, a refusal line and the buttons;
  for a script `$ rusty <name>`, its skill, the text editor and Save, Run. The editors are filled
  when the choice changes and after a write's read; Save sends what they hold. `run_script` builds
  a `TaskTemplate` (`bash`, the quoted path, label `rusty <name>`, center) and schedules it, for a
  project on this machine only. `NewSkillForm` (`ModalView`): name, description, body, a checkbox
  for staging, Create; the refusal stays.
- **`brain.rs`**: a Skills button (`IconName::ToolHammer` or the nearest) after Memory.
- **The stand-in**: a store in `skills/` of its state folder (`.claude/skills/<name>/SKILL.md`
  active, `staging/<name>/SKILL.md` pending, `*.sh` beside them), the eleven tools with Rusty's
  order, scan, refusals and answers; writes announced and the store watched.
- **`rusty.rs`**: `mod skills_tab;` and its `init`. **The guide page**: a Skills article.
- **File manifest:** `crates/marley_rusty/src/{skills.rs, marley_rusty.rs}`, the stand-in,
  `crates/marley_workbench/src/rusty.rs`, `crates/marley_workbench/src/rusty/{skills_tab.rs,
  brain.rs}`, the guide page (all Marley); `script/e2e/665-rusty-skills-tab.sh` (Test). No Zed
  crate.

### Visual check plan
| Criterion | What the scenario does | Shot |
|---|---|---|
| REQ-001, 002 | Palette: `rusty: open skills` | `665-01-skills` |
| REQ-003 | Click an active skill | `665-02-skill` |
| REQ-004 | Edit its description, Save | `665-03-saved` |
| REQ-005 | Click the marked staged skill, Scan | `665-04-scan` |
| REQ-006 | Approve | `665-05-approve-refused` |
| REQ-007 | Click the clean staged skill, Approve; the marked one, Reject | `665-06-approved`, `665-07-rejected` |
| REQ-008 | Click the script | `665-08-script` |
| REQ-009 | Run | `665-09-run` |
| REQ-010 | New Skill, a bad name, Create; a good name, staged, Create | `665-10-new`, `665-11-created` |
| REQ-011 | Delete, Delete in the prompt | `665-12-deleted` |
| REQ-012 | Not shot | Review: the Memory tab's state line |

### Risks
- **A long body** scrolls inside its editor; the right side's editor fills the space left.
- **Run in a remote project** would run on the project's host, where the path is not; Run says so
  and runs nothing.
- **A script without the execute bit** still runs: `bash <path>`, as Rusty runs it.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall, the brain; discovery.
- [x] Mint the pair; the BACKLOG row removed; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request ("continue on tickets until finished").

## Phase 2 — Code (2026-10-06)
### Built
- **`marley_rusty/src/skills.rs`**: the tool names; `Skill` (`is_pending`, `is_auto`); `Script`
  (`is_pending`, `qualified` as `skill/name`); `ordered` (staged first, then by name); the answers'
  parsers (`skills_from_answer`, `scripts_from_answer`, `findings_from_answer`,
  `update_from_answer` with `UpdateAnswer`, `script_text_from_answer` with `ScriptText`);
  `blocked_by_scan`; `SkillWrite` (`Create`, `Update`, `Delete`, `Approve { force }`, `Reject`,
  `UpdateScript`) with `tool` and `arguments`.
- **`rusty/skills_tab.rs`**: `OpenSkills`; `open` and `open_later`; `BrainSkillsView` with the
  shared link and reads (`skill_list` and `script_list` read together), `Chosen` (a skill by name,
  a script by path), the description and body editors filled when the choice changes,
  `show_script` reading `script_view` on a task of its own, `send` and `answered` for each write
  (findings after Save, Approve Anyway after a scan's refusal, the choice cleared after Delete and
  Reject), `scan`, Delete through `window.prompt`, and `run_script`: a `TaskTemplate` (`bash`, the
  quoted path, the script's folder as its directory, label `rusty <name>`, the center) scheduled
  for a project on this machine. The left column (New Skill, Skills, Scripts) and the right side
  (a skill's buttons, description, body, findings and notice; a script's `$ rusty <name>`, Run,
  Save and text). `NewSkillForm`: name, description, body (prefilled as Rusty's app does), "Stage
  for approval", Create; the skill Rusty answers is listed and chosen at once.
- **`brain.rs`**: a Skills button (`ToolHammer`) after Memory.
- **The stand-in**: a store in `skills/` of its state folder; the eleven tools with Rusty's order,
  name rule, scan, refusals in Rusty's words and answers; each write announced. A smoke run over
  stdio passed each, the blocked approval included.
- **The guide page**: a Skills article.

### Deviations
- A change made elsewhere to the chosen skill shows when it is chosen again: the editors keep what
  they hold across reads, so a read never overwrites an edit in progress.
- No `skill_view` call: `skill_list` carries each skill's body.

### Review
- Clippy: `Button::start_icon` (no `icon`), a missing `;` in a closure, `needless_pass_by_value`
  (`scan` takes `&str`), `unused_self` (`delete` an associated function), `use_self`, an unused
  binding.
- Spawns: none in this crate; Run goes through Zed's task system (gate:22 untouched).
- Re-entrancy: the form and the run open through `window.defer`; the form hands the created skill
  to the tab from its own task.

### Gate
`just gate-diff`: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-06)
Scenario `script/e2e/665-rusty-skills-tab.sh` under `compositor sway`: the stand-in's store with
`release-notes` (active, its script `notes-draft.sh` printing a line) and `tidy-commits` (active),
`fetch-docs` and `quick-shell` (staged, an agent's; the second runs ``!`date` ``). Four runs; the
last two passed every check.

### What each shot shows (last run, read in detail in the one before; the last run's montage
matches)
- `665-01-skills` (REQ-001, 002): Skills (4): fetch-docs and quick-shell (pending, auto), then
  release-notes and tidy-commits, each with its description; Scripts (1): `$ notes-draft`
  release-notes; New Skill; "Choose a skill or a script on the left…".
- `665-02-skill` (REQ-003): tidy-commits: Scan, Save, Delete; its description in the field and its
  body in the editor.
- `665-03-saved` (REQ-004): "Keep commits small, plain and signed" in the field and the list,
  "Saved." under the editor.
- `665-04-scan` (REQ-003, 005): quick-shell: Approve, Reject, Scan, Save, Delete; Scan findings:
  "contains a dynamic command-injection marker (bang-backtick)".
- `665-05-approve-refused` (REQ-006): Rusty's "blocked by safety scan (1 issue(s)): …" in red, and
  Approve Anyway between Approve and Reject.
- `665-06-approved` (REQ-007): fetch-docs now among the active (auto, no pending), "Approved:
  Claude Code can load it now."
- `665-07-rejected` (REQ-007): quick-shell gone, Skills (3), the right side cleared.
- `665-08-script` (REQ-008): `$ rusty notes-draft` release-notes, Run and Save, the script's text.
- `665-09-run` (REQ-009): a center tab "rusty notes-draft" with a finished mark, the terminal
  showing "Drafting release notes for the week" in a finished block; the rail lists it done.
- `665-10-new` (REQ-010): New Skill with "Tour Skill": Rusty's "invalid skill name "Tour Skill": use
  lowercase letters, digits, and hyphens" in the form.
- `665-11-created` (REQ-010): tour-skill listed first, pending and auto, chosen, its description
  and Rusty's starting body.
- `665-12-deleted` (REQ-011): after the prompt, tour-skill gone, Skills (3).
- REQ-012 by review: the state line is the Memory tab's, shot in #664.

### Fixes, each rebuilt and run again
- **The run's summary named the wrong command** (second run, `665-09`): Zed's task summary read
  "Command: /usr/bin/bash -i -c 'bash'", without the script's path, though the script ran. The task
  now hides its command, as `remote.rs`'s does; the tab's title names the script.
- **The scenario**: the description field's middle at y 231; the buttons at y 166, Approve and
  Reject from their widths; the script's row follows the number of skills; the form's Create moves
  down once a refusal shows.

The gate runs on the final tree at Complete, after the fix and the docs.

## Phase 4 — Complete (2026-10-06)
- **Docs:** `CHANGELOG.md` (Added); `docs/marley/rusty-in-marley.md` (the screens row, R8);
  `docs/marley/guide.md` (The Skills tab); `docs/marley/walkthrough.md` (stop 2.15g,
  `rusty: open skills` in Appendix B); `docs/marley_architecture/marley_rusty.md` (`skills`, the
  stand-in's store); `docs/marley_architecture/marley_workbench.md` (The Skills tab). No Zed crate
  touched, so no zed-touchpoints row.
- **Knowledge:** L-claude-665-a-task-runs-one-command-in-a-center-terminal-001,
  L-claude-665-check-a-new-scenario-before-the-final-gate-001,
  AD-claude-665-the-skills-tab-follows-rustys-app-and-runs-in-a-terminal-001. No F block: the one
  fault found (the summary's command line) was Zed's wording, met by a setting.
- **Brain:** `brain decide` on consultation `b420bfe4330741c094c83e158f0e8460`, follow up by
  2026-11-06: `decisions/marleys-skills-tab-follows-rustys-app-and-runs-scripts-in-a-terminal`.
- **Closed:** the ticket in `tickets/closed/`; this pair in `completed/`.
- **Final gate:** the first run went red on gate:11, shellcheck's SC2016 on the fixture's literal
  backtick in single quotes; the fixture is now a quoted heredoc, and the gate ran again. A new
  scenario now gets `shellcheck` and `typos` before the final gate (L-665).
