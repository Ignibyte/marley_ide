---
pipeline_id: 5ee56c15-e36a-4070-bc15-40766471b3b3
ticket: docs/planning/tickets/open/TICKET-665-rusty-skills-tab.md
status: Phase 4 — Complete PASS
title: "The Skills tab"
type: feature
slice: Rusty in Marley R8 (docs/marley/rusty-in-marley.md)
references: [docs/marley/rusty-in-marley.md, docs/planning/pipeline/completed/664-rusty-memory-tab.spec.md]
---

## Title
The Skills tab. Rusty keeps a store of skills (a `SKILL.md` each, active or staged for approval,
in a git repository it commits) and the scripts beside them, served by `skill_list`, `skill_view`,
`skill_create`, `skill_update`, `skill_delete`, `skill_scan`, `skill_approve`, `skill_reject`,
`script_list`, `script_view` and `script_update`. Since Rusty's TICKET-051 every write over MCP
commits. Marley shows none of it. This ticket adds a center tab to review staged skills, edit a
skill or a script, make a skill, delete one, and run a script in a Marley terminal.

## Scope
### In
- **The tab.** `rusty: open skills`, and a Skills button after Memory in the Brain view's header,
  open a center tab (or bring it forward). On the left, Skills (staged ones first, marked pending,
  then the active ones, each by name with its description and `auto` when an agent wrote it) and
  Scripts (`$ name`, its skill, pending when its skill is); New Skill above them. On the right, the
  one chosen.
- **A skill.** Its name, its description in a field and its body in an editor; Save sends both;
  Scan shows Rusty's findings or says the scan is clean; Delete asks first. A staged skill also has
  Approve, Reject, and, once Approve was refused for the scan's findings, Approve Anyway.
- **A script.** `$ rusty <name>` and its skill; its text in an editor; Save; Run, for a script of an
  active skill, opens a terminal in the center running it.
- **New Skill.** A form: name (lowercase letters, digits and hyphens, Rusty's rule), description,
  body, and "Stage for approval"; Create makes it and selects it. Rusty's refusal shows in the form.
- **Live.** Reads when it opens, when Rusty connects, on Rusty's announcement while it shows (else
  when it next shows), after each write, and on Read again; off or not connected says so. A Save
  answered with findings shows them.
- **`marley_rusty::skills`** and the stand-in's eleven tools over a store in its state folder, with
  Rusty's order, scan, refusals and paths.

### Out (explicitly deferred)
- **Running a script inside Rusty** (`script_run`, cut at 60 s with its output); Run uses a terminal,
  as Rusty's app does.
- **Making or deleting a script**, and renaming a skill; Rusty's tools have none over MCP.
- **The skill's frontmatter keys other than the description**; Rusty keeps them as they are.

## Reference (§20)
N/A — Marley-specific: Rusty's own app is the reference for the behaviour
(`crates/rusty-app/qml/SkillsPage.qml` and `Main.qml:970` in Rusty's repository at `eb1ab51`:
staged skills first, then by name; a skill's description and body with Save, Scan, Approve,
Approve anyway after findings, Reject and Delete; scripts as `$ rusty <name>` with Run in a
terminal on the script's path; New with name, description, body and "stage for approval"). Zed's
own list items, chips, editors, buttons, modal, prompt and terminal draw it.

### Prior art
- **Behaviour maps:** `docs/t3code_architecture/` and `docs/orca_architecture/` hold no skill
  store's review; `docs/zed_architecture/` none (Zed's rules library is a different store).
- **Published material:** Rusty's tools (`crates/rusty-mcp/src/main.rs:1120-1175`, `:1250-1268`,
  `:1619-1630`, `:1794-1858`, `:2004-2016`; `CreateSkillParams` `:320-333`, `ApproveSkillParams`
  `:337-343`, the script parameters `:369-399`, `SkillUpdateResult` `:591-596`) and its store
  (`crates/rusty-core/src/skills/mod.rs`: `Skill`, `SkillOrigin`, `SkillStatus`, `list` by name,
  `is_valid_skill_name`, `create_skill_at`'s refusals, `approve` refusing on findings without
  `force`, `scan_skill_md`; `skills/scripts.rs`: `Script`, `scripts` by name then skill,
  `resolve_script` by `skill/name`).
- **The code we ship:** the Memory tab (`rusty/memory_tab.rs`) and the Decisions tab's link and
  read bookkeeping; Zed's `Editor` (multi-line for a body), `Window::prompt`, `ListItem`, `Chip`;
  Marley's own terminal opener for Run (see the design). No crate we build owns skills.

## UI proof
`script/e2e/665-rusty-skills-tab.sh` (`compositor sway`: clicks on rows and buttons). Setup: the
stand-in over a scratch store in its state folder with two active skills (one with a script) and
two staged ones (one clean, one whose body has a bang-backtick marker), never the user's store or
Rusty (R-D8). Shots:
- `665-01-skills`: `rusty: open skills`: staged skills first, then active, then the scripts.
- `665-02-skill`: an active skill chosen: its description and body, Save, Scan, Delete.
- `665-03-saved`: its description edited and Save: the list's description follows.
- `665-04-scan`: Scan on the marked staged skill: the findings.
- `665-05-approve-refused`: Approve on it: Rusty's refusal, and Approve Anyway.
- `665-06-approved`: Approve on the clean staged skill: it moves to the active ones.
- `665-07-rejected`: Reject on the marked one: it is gone.
- `665-08-script`: the script chosen: `$ rusty <name>`, its text, Run.
- `665-09-run`: Run: a terminal in the center running it, its output shown.
- `665-10-new`: New Skill: the form, a bad name refused in Rusty's words.
- `665-11-created`: a good name, staged: it is listed pending and chosen.
- `665-12-deleted`: Delete on it, answered: it is gone.

## Locked-In Decisions
- D1 — **A center tab like Memory** (`rusty/skills_tab.rs`): one per workspace, the Decisions tab's
  link and reads, a Brain view button and a palette command.
- D2 — **Rusty's order, with staged first**: as Rusty's app sorts them; scripts as Rusty lists them.
- D3 — **One selection, a skill or a script**, kept across reads by name (a skill) or by path (a
  script); one that went away clears the right side.
- D4 — **Save sends what the fields hold**: a skill's description and body together, a script's
  whole text; a Save answered with findings shows them under the editor.
- D5 — **Approve Anyway appears only after Approve was refused for findings**, as in Rusty's app.
- D6 — **Run opens a terminal on the script's path in the center** with Marley's own opener, titled
  `rusty <name>`, only for a script of an active skill; nothing runs through `script_run`.
- D7 — **New Skill is a modal form**; Rusty checks the name and its refusal stays in the form.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `rusty: open skills` runs or the Brain view's Skills button is clicked, the system shall open the Skills tab or bring it forward. | Shot `665-01-skills` |
| REQ-002 | WHILE the tab shows, it shall list staged skills first, then active ones, each with its description, then the scripts. | Shot `665-01-skills` |
| REQ-003 | WHEN a skill is chosen, the tab shall show its description and body with Save, Scan and Delete, and for a staged one Approve and Reject. | Shots `665-02-skill`, `665-04-scan` |
| REQ-004 | WHEN Save is clicked on a skill, the system shall send its description and body and show it as Rusty holds it. | Shot `665-03-saved` |
| REQ-005 | WHEN Scan is clicked, the tab shall show Rusty's findings, or that the scan is clean. | Shot `665-04-scan` |
| REQ-006 | IF Rusty refuses an Approve for the scan's findings, THEN the tab shall show Rusty's words and offer Approve Anyway. | Shot `665-05-approve-refused` |
| REQ-007 | WHEN Approve or Reject succeeds, the skill shall move to the active ones or leave the list. | Shots `665-06-approved`, `665-07-rejected` |
| REQ-008 | WHEN a script is chosen, the tab shall show its text with Save, and Run when its skill is active. | Shot `665-08-script` |
| REQ-009 | WHEN Run is clicked, the system shall open a terminal in the center running the script. | Shot `665-09-run` |
| REQ-010 | WHEN New Skill's Create is clicked, the system shall make the skill as given, or show Rusty's refusal in the form. | Shots `665-10-new`, `665-11-created` |
| REQ-011 | WHEN Delete is clicked and confirmed, the system shall delete the skill. | Shot `665-12-deleted` |
| REQ-012 | WHILE Rusty is off or not connected, the tab shall say so and list nothing. | Review (the Memory tab's state line, shot in #664) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_rusty::skills`; `rusty/skills_tab.rs` (the lists, the right side, the
  writes, New Skill, Run); the Brain view's button; the stand-in's tools; the guide page; a review;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, the plan's R8, the architecture notes, the guide and the
  walkthrough, ledger capture, the brain decision, close, archive, commit.
