---
pipeline_id: 1ae37c98-3217-4bda-8b47-23f408f6db07
ticket: docs/planning/tickets/open/TICKET-791-rustys-vault-when-rusty-runs-on-another-machine.md
status: Phase 4 — Complete PASS
title: Rusty's vault when Rusty runs on another machine
type: feature
slice: Cross-cutting (macOS, Rusty in Marley)
references:
  - docs/planning/pipeline/completed/645-brain-page-tab.spec.md
  - docs/planning/pipeline/completed/780-marley-and-rusty-folders.spec.md
  - docs/planning/pipeline/completed/787-marleys-mac-app.notes.md
---

## Title
A brain page's Edit and Rusty's agent folder when Rusty's service runs on another machine. The
Mac's Marley reaches the Linux box's Rusty through an SSH tunnel (`connection: "service"`), and
Chad keeps Rusty's store on the box for now (2026-10-16: "port all of rusty data over keep it here
for now"). Marley takes Rusty's vault for a folder on its own machine in two places: Edit opens the
page's file (`page.rs`, `toggle_edit`), and Rusty's agent starts in the vault folder when
`marley.rusty.agent_folder` is unset (`assistant.rs`, `folder_of`). On the Mac the first opens
`/home/cpeppers/.rusty/brain/…`, which is not there, and the second would start in `~/.rusty/brain`,
which since the import is a copy of the box's vault that nothing keeps current.

## Scope
### In
- Marley learns where Rusty's vault is from Rusty: `brain_stats`'s `vault_root`, read with the
  settings at connect.
- Edit, when the vault is not on this machine or the window's project is remote: the page's whole
  file as `brain_render` gives it (`raw`) in an editor of its own, saved through Rusty's
  `brain_write_page`, refused when the page changed in Rusty since Edit began.
- Rusty's agent with no `agent_folder`: the vault folder when it is on this machine, else the home
  folder.
- The stand-in `rusty-mcp` answers `brain_stats` and `brain_write_page`, and with
  `RUSTY_STAND_IN_ELSEWHERE` names its vault and pages at a folder this machine does not have.

### Out (explicitly deferred)
- Edit when the vault is here and the project local: unchanged, the file in Zed's editor (#645).
- Merging an edit with a change made in Rusty meanwhile: the save is refused, nothing merges.
- Opening the vault's files through an SSH project, or a mount.
- Any change to Rusty (`Ignibyte/rusty`): both tools ship in its 0.1.1.
- Moving Rusty's store to the Mac, and the Rusty launchd service there.

## Reference (§20)
Upstream Zed's Rules Library: a rule is text whose store is not a file, edited in an ordinary
`Editor` over a buffer of its own and written to its store by the view, never through the
project. Marley's tree does not carry that crate (only its action name, in `zed_actions`); the
shape it does carry is `agent_ui`'s message editor, an `Editor` over a `Buffer::local` whose text
goes to a thread. Marley does the same with a brain page whose file is on another machine: Zed's
editor over a buffer made from the page's text, and the save is Rusty's tool. The file-backed
Edit (#645) is Zed's own `Project::open_local_buffer`, kept.

### Prior art
- **Zed's crates.** `language::Buffer::local` gives a buffer with no file;
  `Buffer::set_language` sets Markdown (from the project's `LanguageRegistry`,
  `language_for_name`); `Buffer::did_save(version, None, cx)` clears the unsaved state after a
  write that was not Zed's own, as `project::buffer_store` calls it after a disk save.
  `Buffer::is_dirty` follows `has_unsaved_edits` for a buffer with no file. `Editor::for_buffer`
  takes no project. `Editor::can_save` is false for a buffer with no project path, so the tab gives
  its own `can_save` and `save` (`workspace::Item`). `agent_ui`'s message editor and
  `git_ui`'s text diff view build editors over `Buffer::local` the same way.
- **Rusty 0.1.1 (the code we already reach).** `brain_render` answers `raw`, the whole file
  (Marley's `RenderedPage::raw` already reads it for Read); `brain_write_page {slug, content}`
  replaces the file "the way an editor saves", keeps the old text as a version, syncs the index
  and commits; `brain_stats` answers `vault_root`, the vault's folder as an absolute path. No new
  tool is needed.
- **The behavior maps.** `docs/zed_architecture/` and `docs/warp_architecture/` have nothing on a
  store reached over a tunnel; Warp has no notes vault.
- **Rejected:** opening `page.file` through an SSH project on Rusty's host (Marley cannot tell
  which host a tunnel reaches, and a stale copy at the same path on the wrong host would take the
  edit); rebuilding the file from `brain_read_page`'s parsed frontmatter (lossy).

## UI proof
`script/e2e/791-rustys-vault-on-another-machine.sh`, `compositor sway`: the stand-in with
`RUSTY_STAND_IN_ELSEWHERE` set to a folder under the run that is never made, the Marley entry on a
scripted ACP agent that logs each `session/new` (#780's), Rusty on.
- `791-01-edit`: Edit on a page shows its whole file, frontmatter first, in the tab (REQ-001).
- `791-02-typed`: a typed line, the tab's unsaved dot (REQ-002).
- `791-03-saved`: after Ctrl-S the dot is gone; the stand-in logged `brain_write_page` and its
  vault's file holds the typed line (REQ-002).
- `791-04-read`: another line, then Read: saved through Rusty, the page read with the line
  (REQ-004).
- `791-05-changed`: Edit again, a line, a change to the page from outside, then Ctrl-S: the tab's
  line says the page changed in Rusty, the dot stays, the file keeps the outside change (REQ-003).
- `791-06-rusty-agent`: New Agent, Rusty: its `session/new` `cwd` is the home folder, and the
  elsewhere folder was not made (REQ-005).

## Locked-In Decisions
- D1 — The vault is on this machine when the folder Rusty names as `vault_root` (`brain_stats`)
  is a folder here. With no `vault_root` (the read failed), the stored `brain_vault_path`, else
  `~/.rusty/brain`, as before.
- D2 — Edit with the vault here and a local project opens the file in Zed's editor, unchanged.
- D3 — Otherwise Edit opens the page's text (`brain_render`'s `raw`) in a Markdown buffer of the
  tab's own, and a save (Ctrl-S, Read, Back, Forward, the close prompt's Save) writes it with
  `brain_write_page`. This also ends #645's refusal of Edit in a remote project.
- D4 — Before it writes, the save renders the page again; a `raw` that differs from the text
  Edit began with writes nothing, says "changed in Rusty since Edit began" in the tab, and the
  edit stays unsaved.
- D5 — Rusty's agent with `agent_folder` unset starts in the vault folder when it is here (D1),
  else the home folder; neither is made.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Edit is chosen on a brain page while Rusty's vault is not on this machine, the page tab shall show the page's whole file, frontmatter included, in an editor | `791-01-edit` |
| REQ-002 | WHEN that edit is saved, Marley shall write it with `brain_write_page`, and the tab shall show no unsaved mark | `791-02-typed`, `791-03-saved`; the stand-in's call log and vault file |
| REQ-003 | IF the page changed in Rusty since Edit began, THEN the save shall write nothing, the tab shall say the page changed in Rusty, and the edit shall stay unsaved | `791-05-changed`; the vault file |
| REQ-004 | WHEN Read is chosen with unsaved edits in such an edit, Marley shall save them through Rusty and show the page read again | `791-04-read` |
| REQ-005 | WHILE `marley.rusty.agent_folder` is unset and the vault is not on this machine, a new Rusty conversation shall start in the home folder, and no folder for the vault shall be made | `791-06-rusty-agent`; the agent's log; `test ! -e` on the elsewhere folder |
| REQ-006 | WHILE the vault is on this machine and the project is local, Edit shall open the page's file as before | the review of the diff (the #645 path is untouched) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_rusty` (`brain_stats`'s answer, the write tool's name), `rusty.rs`
  (`vault_root`, `vault_here`), `rusty/page.rs` (the edit's two sources), `assistant.rs`
  (`folder_of`), the stand-in; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot; on the Mac, Edit on
  a real page through the tunnel, unsaved.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
