# A settings.json edit reloads after Marley has written the file — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-612-settings-edit-after-marley-writes.md
- **Pipeline spec:** 612-settings-edit-after-marley-writes.spec.md

## Phase 1 — Plan (2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything
  except the cloud flare ones"; #612 is wave 2's bug, found in #607's Test phase. No queued pair:
  minted here from the templates.
- **Classification:** bug, a Zed crate (`settings`), one function.
- **Recall (§18.3):**
  - TICKET-612: the edit before the layout round trip applied, the one after did not; notify
    logged "unable to remove watch descriptor" at Marley's write. #607's scenario edits its
    settings before the round trip for this reason.
  - L-claude-438's note that a process's inotify watches show in `/proc/<pid>/fdinfo`.
  - The brain (`rusty-cli brain ask`, consultation 537631a24d7c4916a87fe292e15c8a6b): nothing on
    this seam.
- **Discovery:** `SettingsStore::watch_settings_files` (`settings_store.rs:353`) watches
  `settings.json` and the global settings with `watch_config_file` (`settings_file.rs:171`), which
  calls `fs.watch(&path)` and throws the returned watcher away. On Linux `fs_watcher::watch` adds an
  inotify watch on the path itself (`OsWatcherKind::is_recursive` is false; only a symlink gets its
  target's parent watched too). `Fs::atomic_write` (`fs.rs:972`) saves through a temporary file in
  the same folder renamed over the path, so the inode changes and the watch ends.
- **Reproduced** on the build before the fix (#618's): the scenario's run printed settings.json's
  inode 2482609 before the round trip and 2482711 after it; `ui_font_size` set to 22, 12 and then
  18 by a rename changed nothing on screen (`edited.png` is `before.png`), and the log has notify's
  "unable to remove watch descriptor ... Invalid argument" once, at Marley's write.

### Design
- **`crates/settings/src/settings_file.rs`** (Zed crate), `watch_config_file`: keep the watcher
  `fs.watch` returns and `add` the path's parent to it, logging a failure; then reload only for a
  batch with an event whose path is the file, or a Rescan. A `// Marley:` comment on the hunk.
- **Ledger:** a row for `crates/settings/src/settings_file.rs` in `docs/marley/zed-touchpoints.md`.
- **Manifest:**

| File | Crate | Change |
|---|---|---|
| `crates/settings/src/settings_file.rs` | Zed | the parent watch and the filter |
| `docs/marley/zed-touchpoints.md` | Marley | its row |
| `script/e2e/612-settings-edit-after-marley-writes.sh` | e2e | the scenario, written in Plan for the reproduction |

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | the layout round trip, then `ui_font_size` 22 and 12 in place | `edited.png`, `edited-again.png` against `before.png` |
| REQ-002 | `ui_font_size` 18 written to a new file renamed over the old | `replaced.png` |
| REQ-003 | review only: a sibling's event does not name the file | — |

### Risks
- A path that is its own parent (none for a config file) or a parent that does not exist yet (a
  fresh install): `Watcher::add` polls a missing path until it appears, as it does for the file.
- macOS and Windows watch recursively already; the added parent watch covers the file there too,
  and the filter keeps siblings out.

## Phase 2 — Code (2026-09-30)
- **Built:** `watch_config_file` keeps the watcher `fs.watch` returns and adds the file's parent
  folder to it; its loop is now `while let Some(batch)`, and a batch with no event for the file
  and no rescan is passed over. Two `// Marley:` comments. The ledger row in
  `docs/marley/zed-touchpoints.md`.
- **Deviation, from the review:** the folder watch is added on Linux and FreeBSD only. A
  `.editorconfig` above a worktree is watched through this function (`editorconfig_store.rs:296`)
  and can sit in the home folder; macOS's and Windows's watches are recursive and by path, so
  there the added watch would have been the whole home folder's, for a bug they do not have.
- **Review:** the filter compares the canonical path the function already uses with the event's,
  which the watcher builds from the same canonical folder; a rescan still reloads, as before.
- **Clippy found:** `while_let_loop` (the loop rewritten as `while let`).
- **Gate:** `just gate-diff` GREEN, 17 PASS.

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/612-settings-edit-after-marley-writes.sh` (sway), the one Plan ran on
  the build before the fix. One run on the fixed build; settings.json's inode went from 2482715 to
  2482817 at Marley's writes, and to 2482818 at the scenario's rename.
- **Shots:**
  - `edited.png` (REQ-001): after `ui_font_size` 22 in place, the rail, the title bar, the tab and
    the project panel are drawn larger than in `before.png` (the Plan run's `edited.png` matched
    `before.png`).
  - `edited-again.png` (REQ-001): 12 in place: all of it smaller.
  - `replaced.png` (REQ-002): 18 by a new file renamed over the old: between the two.
- **REQ-003** (a sibling's change does not reload settings.json): reviewed in the filter; no shot
  can show a reload that does not happen.
- **No fix needed.**

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md` (Fixed); the ledger row in `docs/marley/zed-touchpoints.md`
  (written in Code). No Marley crate changed, so no per-crate note.
- **Knowledge:** L-claude-612-a-watched-config-file-on-linux-needs-its-folder-watched-001.
  No `F-` block: the bug is upstream's watcher's, found in #607's Test phase and recorded there
  as this ticket.
- **Brain:** consultation 537631a24d7c4916a87fe292e15c8a6b closed with `brain decide`.

