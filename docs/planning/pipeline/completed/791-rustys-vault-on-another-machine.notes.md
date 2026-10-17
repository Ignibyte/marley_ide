# Rusty's vault when Rusty runs on another machine — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-791-rustys-vault-when-rusty-runs-on-another-machine.md
- **Pipeline spec:** 791-rustys-vault-on-another-machine.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-16, on the Mac move: "go ahead and push and do whatever is left.
  port all of rusty data over keep it here for now". The store stays on the box, which is the
  branch where the ticket said the Mac needs this. The store was copied to the Mac the same hour
  (`rusty-cli export --include-secrets` on the box, `import` on the Mac: 1018 pages, 19 tasks, 5
  memories; the box stays live, the Mac's Marley reaches it through the tunnel).
- **Classification / tier:** feature, Marley crates only; Rusty's 0.1.1 tools as they are.
- **Recall (§18.3):**
  - #645's notes: Edit opens `page.file` (Rusty's TICKET-042) with `Project::open_local_buffer`
    and refuses a remote project ("Edit needs a local project: Rusty's vault is on this
    machine."); `brain_render`'s `raw` is the whole file. The save goes through Zed's editor with
    `format: false` (Prettier would reformat a page).
  - AD-claude-780: Rusty's agent with no `agent_folder` starts in the vault folder Rusty stores,
    else `~/.rusty/brain`, and a missing folder is made.
  - #787's notes: the Mac's Marley on the box's Rusty through `ssh -L 4174`; `agent_folder "~/"`
    set by hand in the Mac's settings because of this ticket.
  - The brain (`rusty-cli brain ask`, consultation d74783e0): nothing on this seam.
- **Discovery:** `rusty.rs:728` `vault_folder` (the stored `brain_vault_path`, else
  `~/.rusty/brain`), its two callers `assistant.rs:1117` (`folder_of`) and
  `rusty/page.rs:1053` (`toggle_edit`); `read_settings` (`rusty.rs:565`), run at connect and
  after each setting write; the `Item` impl (`page.rs:1694-1804`: `is_dirty`, `can_save`, `save`,
  `reload`, `for_each_project_item` through the editor); `leave_edit_then` (`page.rs:506`), the
  save before Read, Back and Forward. Rusty 0.1.1: `brain_stats` (`vault_root`),
  `brain_write_page` (`write_raw`: a version of the old text, the index synced, a commit
  "edit: <title>"). The stand-in answers neither.
- **Decisions:** D1 to D5 in the spec.

### Design
- **`marley_rusty`** (Marley crate):
  - `vault.rs`: `BRAIN_STATS`, `BRAIN_WRITE_PAGE`, and `vault_root_of(answer) -> Result<Option<PathBuf>, serde_json::Error>`
    reading `vault_root` from `brain_stats`'s answer (absent or empty is `None`).
- **`marley_workbench/src/rusty.rs`** (Marley crate):
  - `Rusty.vault_root: Option<PathBuf>`, read in `read_settings` beside the settings, cleared
    where `settings` is cleared.
  - `vault_folder(cx)`: `vault_root` first, then as now.
  - `vault_here(cx) -> Option<PathBuf>`: `vault_folder` when it is a folder on this machine.
- **`marley_workbench/src/rusty/page.rs`** (Marley crate):
  - `Mode::Edit` gains `source: Source`: `File` (Zed's project buffer, as now) or
    `Rusty { slug, began: String }` (the text Edit began from).
  - `toggle_edit`: `vault_here` and a local project → the file, as now (D2). Otherwise
    `Buffer::local(raw)` with the project's Markdown language, `Editor::for_buffer(buffer, None)`,
    the same event subscription (D3).
  - One `save_edit(window, cx) -> Task<anyhow::Result<()>>` for both sources: `File` is the
    editor's save with `format: false` as now; `Rusty` renders the page, compares `raw` with
    `began` (D4: an `anyhow` error "… changed in Rusty since Edit began …"), writes with
    `brain_write_page`, then `did_save(version, None)` on the buffer and `began` = the written
    text. `leave_edit_then` and `Item::save` both call it; `Item::can_save` is true for a `Rusty`
    edit; `Item::reload` does nothing for it; `has_conflict` stays the editor's.
- **`marley_workbench/src/assistant.rs`** (Marley crate): `folder_of`, Rusty unset → `vault_here`,
  else `paths::home_dir()` (D5).
- **`crates/marley_rusty/stand_in/rusty-mcp`** (Marley crate's test stand-in): `brain_stats`
  (counts and `vault_root`), `brain_write_page` (the whole file, as `write_raw`), and
  `RUSTY_STAND_IN_ELSEWHERE`: the folder its `vault_root`, `brain_vault_path` and each page's
  `file` name in place of its real vault.
- No Zed crate is touched, so no `zed-touchpoints.md` row.

### Visual check plan
| Criterion | The scenario | The shot |
|---|---|---|
| REQ-001 | Brain view, click a page (a preview tab, Read), Edit | `791-01-edit`: the frontmatter and body in an editor |
| REQ-002 | Type a line, Ctrl-S; `expect` the stand-in logged `brain_write_page` and its file has the line | `791-02-typed` (the dot), `791-03-saved` (no dot) |
| REQ-003 | Type again; the scenario rewrites the page's file (a change in Rusty); Ctrl-S; `expect` the file has the outside line and not the second typed one | `791-04-changed`: the tab's line, the dot |
| REQ-004 | Read (which saves): after reloading the page (Read, Edit again from the changed text), type, Read | `791-05-read`: the page read with the line |
| REQ-005 | New Agent, Rusty; `expect` `session/new`'s `cwd` is `$HOME` and the elsewhere folder does not exist | `791-06-rusty-agent` |
| REQ-006 | Not run: the vault-here path is #645's code, reached by the same branch as before | review of the diff |

The Mac, after the install: the Mac's Marley on the box's real Rusty through the tunnel, Edit on a
page shows its text; nothing typed or saved (Chad's data). Shot with `screencapture` through iTerm.

### Risks
- Two machines with the same vault path (the same user and layout) look like one: D1 then takes
  the local copy for the vault. Named in the guide; a Mac and the Linux box differ (`/Users`,
  `/home`).
- The window of D4's check: a change in Rusty between the render and the write is overwritten.
  Rusty keeps the overwritten text as a version, so nothing is lost.
- A Rusty older than `brain_write_page` refuses the save: the tab shows Rusty's error.

## Phase 2 — Code
- **Built, to the manifest:**
  - `marley_rusty`: `vault::BRAIN_STATS` and `vault_root_from_answer` (`vault_root`, empty as
    none); `page::BRAIN_WRITE_PAGE`.
  - `rusty.rs`: `Rusty.vault_root`, read by `read_settings` with `brain_stats` after the settings
    and the embedding status (a failed read logs a warning and leaves it unset), cleared where
    the settings are; `vault_folder` takes it first; `vault_here` is `vault_folder` when it is a
    folder here.
  - `page.rs`: `Mode::Edit` carries `source: EditSource` (`File`, or `Rusty { slug, began }`).
    `toggle_edit` takes the file only with a local project and `vault_here`; otherwise
    `edit_through_rusty` makes `Buffer::local(raw)`, sets Markdown when the registry loads it,
    and puts `Editor::for_buffer(buffer, None)` in the tab. `enter_edit` is the old inline
    subscription, focus and mode, shared by both. `save_edit` is the one save, which
    `leave_edit_then` (Read, Back, Forward) and `Item::save` both call: the file through Zed's
    editor as before; Rusty's through `write_through_rusty`, which renders the page, refuses when
    `raw` differs from `began`, writes with `brain_write_page`, `did_save`s the version it wrote
    and sets `began` to the text written, and puts any failure in the tab's line. `Item::can_save`
    is true for a Rusty edit (Zed's editor says false with no project path); `Item::reload` does
    nothing for one.
  - `assistant.rs`: `folder_of`, Rusty with no `agent_folder`: `vault_here`, else the home folder.
  - The stand-in: `brain_stats`, `brain_write_page` (`write_raw`), `RUSTY_STAND_IN_ELSEWHERE`
    (`shown` maps a vault file to the elsewhere folder for `file`; `vault_root` and the seeded
    `brain_vault_path` name it). Checked by hand over stdio in a scratch folder: `vault_root` and
    `file` under the elsewhere folder, the write landed in the scratch vault, the elsewhere folder
    never made.
- **Deviations:** none from the design. One addition from the review: Edit through Rusty refuses
  a page whose `raw` is empty ("Rusty gave no text for …"), since a save would then write the
  page over with only what was typed.
- **Review of the diff:**
  - REQ-001 to REQ-005 each have their path; REQ-006's path is #645's code, reached by the same
    `open_local_buffer` branch, its condition now `is_local() && vault_here`.
  - Re-entrancy: `write_through_rusty` reads the buffer before its task, calls `call_tool` inside
    `this.update` (it reads only the `Rusty` global), and updates the buffer and the tab apart,
    each after an await.
  - A save that races typing: `did_save` takes the version the text was read at, so later typing
    stays unsaved; `began` is the text written, which Rusty's `write_raw` stores as given.
  - The failed-save message names the way out (close the tab without saving), since Read saves
    first and would refuse again.
  - Provenance: Marley code over Zed's public APIs; nothing from Warp.
- **First gate run, red (2):**
  - gate:21 dylint, `async_block_without_await` on the background parse: now
    `futures::future::lazy`, as `load` does.
  - gate:7 cargo-audit: RUSTSEC-2026-0367, -0368, -0369, published 2026-10-09/10 against `zip`
    0.6.6, the build dependency of `webrtc-sys-build` in Zed's LiveKit fork pin. Not this change.
    No bump exists (`zip = "0.6"`, the fix is 9.0), and upstream main 322cd509dc (fetched today)
    has the same pin and lock. Listed in `.cargo/audit.toml` under its own upstream-tree policy,
    with the reason. `cargo audit` exits 0.
- **Second gate run: `just gate-diff` GREEN, 17 passed, 0 failed** (log in the session's
  scratchpad, `gate-791.log`). The warnings it prints are Zed crates' under the dylint pass, as
  every run has them.

## Phase 3 — Test
- **The scenario:** `script/e2e/791-rustys-vault-on-another-machine.sh`, `compositor sway`, keys
  only (the run's keymap binds `rusty::OpenPage` for Sam's page to Ctrl-Alt-Shift-Y and
  `rusty::TogglePageEdit` to Ctrl-Alt-Shift-U). The stand-in with `RUSTY_STAND_IN_ELSEWHERE` at
  `$E2E_WORK/elsewhere/vault`, never made; #780's scripted ACP agent, cut down, logging each
  `session/new`. `just build`, then `SHOT_DIR=<scratchpad> just e2e …`.
- **Runs:**
  1. Every check through REQ-003 passed. Then New Agent, Rusty: the picker said "No matches",
     and no `session/new` came. Two findings in the shots:
     - After the refused Ctrl-S, Zed's own "Failed to save" prompt shows the same error and
       adds a full stop to it ("…without saving.."). **Fix:** the error is one clause with no
       full stop, as Zed's errors are, and the tab's line adds one (`could_not_save`). The
       scenario presses OK on the prompt before going on.
     - The missing Rusty entry is not #791's: #780's own scenario, run on this build, fails the
       same way ("Rusty started in its vault: FAIL", the picker showing only Marley). The
       run's profile copies the user's settings, whose `agent_servers.Rusty` is
       `{"type": "registry", "default_config_options": {"mode": "bypassPermissions"}}`, written
       when a mode was made Rusty's default. It stands in place of the custom entry Marley puts in
       the defaults, and no registry agent is named Rusty, so the entry is gone. **Pre-existing —
       not in scope;** the same holds for the user's own Marley, and it gets a ticket of its own.
       The scenario drops that key from its copy.
  2. Rebuilt; `e2e exit 0`, every check passed:
     - "Marley asked Rusty where its vault is" (`brain_stats` in the stand-in's log, 4 times:
       at connect and after each announced change);
     - "the stand-in was asked to write the page once", "Sam's file holds the typed line";
     - "Read wrote the page through Rusty", "Sam's file holds the line Read saved";
     - "nothing more was written", "Sam's file keeps the outside change", "Sam's file lacks the
       refused line";
     - "Rusty started in the home folder" (`session/new cwd: /home/cpeppers`), "the elsewhere
       folder was not made".
- **The shots, read:**
  - `791-01-edit` (REQ-001): the tab's header says `people / sam` with Read in place of Edit; the
    editor holds the whole file from line 1, the frontmatter's `---`, `title: Sam`, `type:
    person`, `---`, then `# Sam` (Markdown's heading colour, so the language loaded) and the
    body; the caret at 1:1.
  - `791-02-typed` (REQ-002): line 9, the typed line; the unsaved dot on the tab and on the
    rail's Sam row.
  - `791-03-saved` (REQ-002): the same text, no dot on the tab or the row.
  - `791-04-read` (REQ-004): Read mode (the Edit button back), the properties `title` and
    `type`, the heading, the body, and the paragraph with both lines; the outline column lists
    Sam.
  - `791-05-changed` (REQ-003): line 11, the refused line; the dot on the tab and the row; the
    tab's red line "Could not save: people/sam changed in Rusty since Edit began, so nothing
    was written; to drop the edit, close the tab without saving."; Zed's "Failed to save"
    prompt with the same sentence and one full stop. The editor does not show the outside line,
    as an edit through Rusty keeps its own text.
  - `791-06-rusty-agent` (REQ-005): a New Agent Thread tab, "Message Rusty — @ to include
    context", the rail's row "New Agent Thread · Rusty · idle".
- **Focus report:** the keys reached Marley's window in the run's own sway; nothing else shows
  in any shot.
- **REQ-006** (the vault here, a local project): not run; #645's path, reached by the same
  branch, reviewed in Phase 2.
- **The Mac:** after the commit, the Mac's Marley (on the box's real Rusty through the tunnel)
  opens a page's Edit without typing or saving, and the result goes in the completed notes.
- **The fix's gate:** the first rerun failed gate:2 (`needless_borrow`, `could_not_save(&error)`
  where `error` was already a reference; L-claude-791-clippy-after-every-fix-before-the-gate-001).
  Fixed; `just clippy marley_workbench` clean; `just gate-diff` GREEN (`gate-791c.log`). The
  scenario's build predates that one-character change, which alters no behaviour.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Added, #791, with the audit listing); `docs/marley/guide.md` (Edit when
  the vault is elsewhere; Rusty's agent folder; the Mac section's "Rusty on the Linux box");
  `docs/marley/three-prong-plan.md` (the macOS bullet); `docs/marley_architecture/marley_workbench.md`
  (`vault_folder`, `vault_here`, Edit's two sources, the agent folder) and `marley_rusty.md`
  (`BRAIN_STATS`, `vault_root_from_answer`, `BRAIN_WRITE_PAGE`, the stand-in). No Zed path is
  touched, so no `zed-touchpoints.md` row.
- **Ledger:** F-claude-791-an-edit-through-rusty-with-no-text-would-write-the-page-over-001,
  F-claude-791-zeds-save-prompt-doubled-a-full-stop-001,
  PR-claude-791-an-error-zed-may-show-ends-without-a-full-stop-001,
  L-claude-791-a-scenarios-profile-copies-the-users-agent-servers-001,
  L-claude-791-an-advisory-with-no-fix-in-range-001,
  L-claude-791-clippy-after-every-fix-before-the-gate-001,
  AD-claude-791-a-page-whose-vault-is-elsewhere-is-edited-through-rustys-tools-001.
- **Brain:** consultation d74783e0 closed with `brain decide`:
  `decisions/marley-edits-a-brain-page-through-rustys-tools-when-the-vault-is-on-another-machine`
  (follow-up by 2026-11-16).
- **New ticket:** TICKET-793 (a default mode hides Marley's own agent entry), queued, from the
  Test phase's pre-existing finding.
- **Ticket closed, pipeline archived, committed** with the gate's green receipt (`gate-791c.log`).
