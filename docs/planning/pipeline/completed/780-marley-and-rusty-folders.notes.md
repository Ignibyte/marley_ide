# Marley and Rusty folders — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-780-marley-and-rusty-folders.md
- **Pipeline spec:** 780-marley-and-rusty-folders.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-12, in two answers: Marley and Rusty start in "the location that
  makes sense", then "this should be a settings as well and we dont ask for the folder for marley
  and rusty … The rusty agent probably should open in the place in which rusty has chosen." Run
  autonomously with the batch.
- **Classification / tier:** feature; Marley crate plus the Marley files in two Zed crates
  (`settings_content/src/marley.rs`, `settings_ui/src/marley_page.rs`), rows extended.
- **Checklist:** pick ✓ (BACKLOG row removed), pre-flight ✓, recall ✓, mint ✓, prior art ✓,
  spec ✓, design ✓.
- **Recall (§18.3):**
  - Memory "Assistant is one switch": Rusty off leaves no trace outside its own switch, so Rusty's
    folder sits in Rusty's section.
  - #738's `folder_in(data_dir, entry)` made the hidden folders; #735's picker keeps a `fixed`
    folder for Open Agent Here.
  - The brain (consultation 11445764…): nothing on this seam.
- **Discovery:** `assistant.rs` `talk_to` (the one place a folder is chosen for Talk to and New
  Conversation), `talk_by_name` (an explicit folder); `agents.rs` `NewAgentDelegate::confirm`
  (`fixed`, else Where); `rusty.rs` `vault_folder`; `system_one::folder_path`.

### Design
- `settings_content/src/marley.rs` (Zed crate, row extended): `MarleyAssistantSettingsContent.folder`
  and `MarleyRustySettingsContent.agent_folder`, both `Option<String>`.
- `settings_ui/src/marley_page.rs` (Zed crate, row extended): "Marley's Folder" in the Marley Agent
  section and "Rusty Agent's Folder" in the Rusty section, text fields as `service_url`; the
  Agent item's description names #779's full access.
- `assistant.rs` (Marley): `folder_of(entry, cx)` (the setting through `folder_path`, else
  `paths::config_dir()` or `rusty::vault_folder`), `made(folder)` (`create_dir_all`), and
  `pub(crate) conversation_folder(agent, cx)` for the picker; `talk_to` uses them, a failure a
  toast as before; `folder_in` goes.
- `agents.rs` (Marley): `confirm` takes `fixed`, else the entry's folder, before asking Where; a
  folder that cannot be made is a toast through the workspace.

### Visual check plan
| REQ | Set-up and action | Shot |
|---|---|---|
| 001, 003 | New Agent, "Marley", Enter | `780-01-marley` |
| 002, 003 | New Agent, "Rusty", Enter | `780-02-rusty` |
| 004 | `marley.assistant.folder` set in the run's settings; `marley: new marley conversation` | `780-03-moved` |
| 005 | `zed: open settings`, the Marley page, scrolled to the two | `780-04-settings` |

### Risks
- A folder in the user's settings that cannot be made (a read-only path) leaves Talk to with a
  toast and no conversation, as #738's hidden folder did.
- Rusty's vault joins the project as a hidden worktree while its conversation is open, so Zed
  scans it; it is the folder Rusty's own Brain tab reads.

## Phase 2 — Code
- **Built:**
  - `settings_content/src/marley.rs` (Zed crate, row extended): `MarleyAssistantSettingsContent.folder`,
    `MarleyRustySettingsContent.agent_folder`.
  - `settings_ui/src/marley_page.rs` (Zed crate, row extended): "Marley's Folder" (the Marley Agent
    section, now four items) and "Rusty Agent's Folder" (the Rusty section, seven); the Agent
    item says Codex runs read-only unless Codex Permissions is Full Access (#779).
  - `assistant.rs`: `folder_of(entry, cx)`, `made(folder)`, `conversation_folder(agent, cx)`;
    `talk_to` uses them; `folder_in` and its hidden data folders are gone.
  - `agents.rs`: the picker's `confirm` starts a Marley or Rusty thread in its folder at once,
    unless the picker was opened on a folder; a folder that cannot be made is a toast.
- **Deviations:** none.
- **Review:** `folder_of` reads the merged user settings as `setting` does; `~/` goes through
  `system_one::folder_path`; `rusty::vault_folder` falls back to `~/.rusty/brain` before Rusty's
  settings arrive. The picker's toast goes through the workspace's update, not inside it.
  `talk_by_name` with a folder (#749) still starts there.
- **Gate:** `script/gates.sh --diff` GATE GREEN [diff], 17 of 17 (`gate-780.log`).

## Phase 3 — Test
- **Scenario:** `script/e2e/780-marley-and-rusty-folders.sh` under `compositor sway`, #746's set-up
  (the Marley entry on a scripted ACP agent that logs each message, Rusty on through its stand-in
  with a scratch vault), `list_cli_sessions` off; `started_in` reads the newest `session/new`'s
  `cwd` from the agent's log.
- **The runs found:** the first run's settings shot showed "Rusty Agent's Folder" with the Settings
  window's red NO DEFAULT marker: a field whose `pick` finds nothing in the defaults is drawn as an
  error in a debug build. Fixed at the source: both picks fall back to an empty string
  (`NO_FOLDER`), as Zed's own pages do with `page_data`'s `DEFAULT_EMPTY_STRING`; the gate ran
  again. The settings shots also needed clicks on the navigation's Marley Agent and Rusty.
- **The final run**, every check passed (`e2e exit 0`, 4 checks):
  - **780-00-picker / 780-01-marley** (REQ-001, REQ-003): "Marley" in New Agent lists the Marley
    Thread; Enter opens "Message Marley" in a repo tab at once, no Where; `cwd` the run's
    `$E2E_PROFILE/config`, Marley's settings folder under `--user-data-dir`.
  - **780-02-rusty** (REQ-002, REQ-003): "Message Rusty" in a tab at once; `cwd` the stand-in's
    `rusty/vault`, the vault Rusty stores.
  - **780-03-moved** (REQ-004): with `marley.assistant.folder` set to a missing scratch folder,
    `marley: new marley conversation` opened a third tab; `cwd` that folder, which now exists.
  - **780-04-settings / 780-05-settings-rusty** (REQ-005): the Settings window, searched for
    "Folder": Marley Agent shows Marley's Folder with the path the run set and its reset mark;
    Rusty shows Rusty Agent's Folder, an empty field, with its description.
- **Not reached:** Open Agent Here keeping its folder for Marley (the picker's `fixed` path is
  unchanged and read in the review); a folder that cannot be made (its toast is the same as #738's).
- **Gate after the fix and the scenario:** `script/gates.sh --diff` again on the final tree.

## Phase 4 — Complete
- Docs: `CHANGELOG.md` (Changed, #780); `docs/marley/guide.md` (the Marley agent's and Rusty's
  "In a tab, anywhere"); `docs/marley_architecture/marley_workbench.md` (`talk_to`'s folder);
  `docs/marley/zed-touchpoints.md` (the two settings rows, `NO_FOLDER` included).
- Ledger appends: `F-claude-780-a-folder-field-with-no-default-showed-no-default-001`,
  `PR-claude-780-a-settings-page-field-needs-a-default-its-pick-can-find-001`,
  `AD-claude-780-marley-and-rusty-start-in-folders-their-settings-give-never-asked-001`.
- Brain: consultation 11445764… closed with `brain decide`.
- Ticket closed; pipeline archived; committed on `marley/workbench-shell`.
