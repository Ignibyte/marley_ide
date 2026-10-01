---
pipeline_id: ceefa7b8-6a63-4074-80bf-44914c328a1d
ticket: docs/planning/tickets/open/TICKET-612-settings-edit-after-marley-writes.md
status: Phase 4 — Complete PASS
title: "A settings.json edit reloads after Marley has written the file"
type: bug
slice: workbench shell; found in #607's Test phase
references: [docs/planning/pipeline/completed/607-fleet-panel-with-pseudo-agents.spec.md]
---

## Title
On Linux, once Marley or Zed has written `settings.json` itself (the layout switch, the theme
picker, every writer that goes through `update_settings_file`), an edit to the file outside
Marley stops reloading until a restart. The writers save by writing a new file and renaming it
over the old (`Fs::atomic_write`); the settings watcher holds an inotify watch on the old file
alone, which goes with it. The watcher now also watches the file's folder, as Zed's own
`watch_config_dir` does, and reloads on an event for the file.

## Scope
### In
- `settings::watch_config_file` adds its file's parent folder to the watch it opens on Linux and
  FreeBSD, where a file's watch follows its inode, and a batch of events reloads the file only
  when one names the file, or asks for a rescan.
- Every watched config file gains it: `settings.json`, the global settings, `keymap.json`, the
  user's `AGENTS.md`, a project's settings and `.editorconfig`, the remote servers' settings.

### Out (explicitly deferred)
- Changing how Zed writes its settings (`atomic_write` stays the writer).
- `fs_watcher` itself: a plain file's watch keeps following its inode, as upstream's does.

## Reference (§20)
Upstream Zed: `settings::watch_config_dir` (`crates/settings/src/settings_file.rs:202`) watches a
folder and reloads the config files named in its events, Removed, Created, Changed and Rescan;
`fs_watcher::watch` (`crates/fs/src/fs_watcher.rs:37-82`) adds a symlinked file's target and the
target's parent folder for the same reason. `watch_config_file` follows that pattern for a plain
file.

### Prior art
- **Behavior maps:** #607's Test phase found it (TICKET-612): a `ui_font_size` edit after a layout
  round trip did not apply, with notify's "unable to remove watch descriptor" in the log.
- **Published material:** inotify(7): a watch is on an inode; a file renamed over the watched one
  ends the watch (`IN_DELETE_SELF`, then `IN_IGNORED`), and a folder's watch reports its
  entries' `IN_MOVED_TO`, `IN_CREATE` and `IN_MODIFY`.
- **Code we already ship:** `Fs::atomic_write` (`fs.rs:972`, a `NamedTempFile` in the same folder,
  persisted over the path); `Watcher::add` on the watcher `Fs::watch` returns, which
  `watch_config_file` discarded; `watch_config_dir`'s event filter. The fork's copy of these is
  upstream as of the 2026-09-18 base; no newer upstream copy was checked.

## UI proof
The scenario `script/e2e/612-settings-edit-after-marley-writes.sh` (sway) opens `repo`, runs
`marley: use zed layout` then `marley: use marley layout` (Marley replaces settings.json, a new
inode), then edits `ui_font_size` outside Marley:
- `edited.png`: 22, written in place: the rail's and the title bar's text larger than in
  `before.png`;
- `edited-again.png`: 12, in place: the text smaller;
- `replaced.png`: 18, a new file renamed over the old: the text at 18.
Run on the build before the fix (Plan, 2026-09-30), `edited.png` matched `before.png`: the bug.

## Locked-In Decisions
- D1 — The fix is in the watcher, not the writers: every writer, Zed's and Marley's and the
  user's editor, can replace the file.
- D2 — The parent folder's events are filtered to the file, as `watch_config_dir` filters, so a
  sibling's change (`keymap.json` beside `settings.json`) does not reload the file.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN settings.json is edited in place after Marley has written it, Marley shall apply the edit without a restart. | Shots `edited.png`, `edited-again.png` |
| REQ-002 | WHEN a new settings.json is renamed over the old one, Marley shall apply it without a restart. | Shot `replaced.png` |
| REQ-003 | WHEN another file in the config folder changes, the watcher shall not reload settings.json. | Review of the filter |

## Phase Plan
- **P1 Plan** — this spec, the reproduction, and the design in the notes.
- **P2 Code** — the watcher in `settings_file.rs`, its ledger row; a review; the gate green.
- **P3 Test** — the scenario on the fixed build, every shot read.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close, archive,
  commit.
