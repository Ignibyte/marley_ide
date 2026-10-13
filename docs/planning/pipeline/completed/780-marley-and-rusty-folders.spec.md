---
pipeline_id: 3a065a1b-c9ad-4674-bf4e-346712ce9421
ticket: docs/planning/tickets/open/TICKET-780-marley-and-rusty-folders.md
status: Phase 4 — Complete PASS
title: Marley and Rusty start in their own folders, set in settings
type: feature
slice: agents in the Marley layout (#735's New Agent picker, #738's Talk to)
references: [683, 696, 735, 738, 744, 749]
---

## Title
New Agent asks Where for the Marley and Rusty agents as for any agent, and Talk to starts them in
hidden empty folders under Marley's data folder. Chad, 2026-10-12: "this should be a settings as
well and we dont ask for the folder for marley and rusty. by default choose the ones you
recommended and if the user wants to somewhere else they can change it. The rusty agent probably
should open in the place in which rusty has chosen."

## Scope
### In
- `marley.assistant.folder`: where Marley's conversations start; by default Marley's settings
  folder (`~/.config/marley` on Linux).
- `marley.rusty.agent_folder`: where Rusty's conversations start; by default the vault folder Rusty
  stores (`brain_vault_path`), else `~/.rusty/brain`.
- Both on the Marley settings page, `~/` taken as the home folder, the folder made when missing.
- New Agent never asks Where for Marley or Rusty: they start in their folder at once. Talk to and
  New Conversation use the same folders.
- The Marley page's Agent description stops saying Codex always runs read-only (#779).

### Out (explicitly deferred)
- A folder given outright still wins: Open Agent Here's folder, and `marley agent-thread
  --provider=marley` with a folder (#749).
- Which agent Marley and Rusty run on stays `marley.assistant.agent`.
- Conversations started before this keep their folders.

## Reference (§20)
N/A — Marley-specific: the Marley and Rusty entries are Marley's own agents (#683, #696); the
folder rule is Marley's New Agent picker (#735) and Talk to (#738), not upstream behavior.

### Prior art
- Behavior maps: none apply.
- Published material: none needed.
- The code we ship: `paths::config_dir()` (Marley's settings folder), `rusty::vault_folder` (Rusty's
  stored `brain_vault_path`, #644), `system_one::folder_path` (`~/` expansion, #532's per-project
  map), the picker's `fixed` folder path (Open Agent Here, #735), `assistant::talk_to`.

## UI proof
`script/e2e/780-marley-and-rusty-folders.sh` (`compositor sway`): the Marley entry on a scripted
ACP agent that logs each `session/new` with its `cwd`; Rusty on through its stand-in, whose vault
is a scratch folder; the copy's thread list emptied.
- `780-01-marley`: New Agent → Marley: a Marley conversation in a tab with no Where step; its
  `session/new` `cwd` is the run's Marley settings folder (REQ-001, REQ-003).
- `780-02-rusty`: New Agent → Rusty: a Rusty conversation, `cwd` the stand-in's vault (REQ-002,
  REQ-003).
- `780-03-moved`: `marley.assistant.folder` set to a new scratch folder: a new Marley conversation
  starts there, the folder made (REQ-004).
- `780-04-settings`: the Marley page lists Marley's Folder and Rusty Agent's Folder (REQ-005).

## Locked-In Decisions
- D1 — Two settings, one per entry, in each entry's section: `marley.assistant.folder` beside the
  Marley Agent's switch, `marley.rusty.agent_folder` beside Rusty's, so Rusty off hides nothing new
  outside its own section.
- D2 — Unset, Marley's is `paths::config_dir()`: Marley's settings live there, so the agent can
  read what it changes. Rusty's is the vault Rusty itself stores, as the Brain tab reads it.
- D3 — The folder is made when missing, as the old hidden folders were; a failure is a toast.
- D4 — New Agent skips Where for an entry with a folder of its own; a picker opened on a folder
  (Open Agent Here) keeps that folder.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user picks Marley in New Agent, the system shall start its conversation in Marley's folder without asking where. | shot `780-01-marley`, the agent's log |
| REQ-002 | WHEN the user picks Rusty in New Agent, the system shall start its conversation in Rusty's folder without asking where. | shot `780-02-rusty`, the log |
| REQ-003 | WHILE neither folder setting is set, the system shall start Marley in Marley's settings folder and Rusty in the vault folder Rusty stores. | the log of `780-01`, `780-02` |
| REQ-004 | WHEN `marley.assistant.folder` names a folder, the system shall start new Marley conversations there, making it when missing. | shot `780-03-moved`, the log |
| REQ-005 | The Marley settings page shall show both folders. | shot `780-04-settings` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `settings_content/src/marley.rs` and `settings_ui/src/marley_page.rs` (Zed crates,
  rows extended first), `assistant.rs`, `agents.rs`; a review of the diff; `script/gates.sh
  --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
