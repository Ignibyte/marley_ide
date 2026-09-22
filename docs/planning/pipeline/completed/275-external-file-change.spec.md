---
pipeline_id: 8d65d10b-fcd2-45ea-8b96-592807f61de0
ticket: forge#275 (f1b13464-6b27-4ce5-82fd-016a1b0c739c) · local docs/planning/tickets/open/TICKET-275-external-file-change.md
aar_id: 09e38da9-301a-46dd-9f47-c31d19ef9bc3
status: Phase 5 — Complete PASS
title: External file-change detection — reload clean, flag dirty conflicts
type: feature
milestone: M17
references:
  - docs/marley_architecture/editor.md
---

## Title
The agent-workflow gap: an agent rewrites a file in the terminal pane
NEXT to the editor and the open buffer never notices (#249 preserves
the buffer on re-open by design; #268 fixed the CACHE, not the
CONTENT). Per-OpenFile disk snapshots + poll-at-interaction checks:
clean buffers silently reload; dirty buffers get a non-modal conflict
banner; ⌘S never silently clobbers the agent's write.

## Scope
### In
- `OpenFile.disk: Option<(SystemTime, u64)>` snapshot (mtime, len) at
  load AND at save; `conflict: Option<Conflict>` per-file state
  (`Changed` / `Deleted`) + a `save_armed` flag for the double-⌘S.
- Pure decision table `external_action(snapshot, disk_now, dirty) ->
  ExternalAction { Noop | CleanReload | Conflict | Deleted }` and the
  reload clamps (caret `min(len)`, scroll via the #273
  `clamp_scroll_px`).
- Three poll choke points (NO fs-watcher thread in v1): the app
  window-focus REGAINED edge (an inactive→active transition detector
  in the render choke — gpui 0.2.2 pub `Window::is_window_active`),
  file-tab activate, and the head of ⌘S.
- Clean reload = fresh `Buffer::from_text`, caret/scroll clamped,
  nonce RE-MINTED (the #268 cache re-parses), `saved_version` reset,
  snapshot updated, status flash "reloaded from disk".
- The conflict banner (#221 card idiom, non-modal, per-file, above the
  editor content): "File changed on disk — Keep mine · Reload".
  Keep-mine = dismiss + RE-SNAPSHOT current disk (stops re-flagging
  until the next external change); Reload = the clean-reload path
  (discards buffer edits deliberately — an explicit click).
- ⌘S under conflict: first press ARMS (flash the warning, no write);
  the second press overwrites, clears the conflict, re-snapshots.
  Deleted-on-disk: banner "removed on disk", buffer kept, ⌘S recreates
  (the normal write path + snapshot).

### Out
- A real fs watcher (FSEvents/kqueue thread) — the recorded upgrade;
  poll-at-interaction covers the agent loop (the terminal pane sits
  NEXT to the editor — refocusing IS the interaction).
- Merge/diff UI (VS Code-style compare view) — v2 territory.
- Multi-window coordination (one window today).
- Watching non-active files' tabs beyond the activate check (a
  background tab flags on its NEXT activation).

## Reference (§20)
The universal external-change convention (the VS Code/JetBrains
behavior CLASS, from public product behavior, not source): clean
buffers auto-reload silently; dirty buffers warn without modal
interruption; saving over a newer disk copy requires explicit intent.
Marley-original implementation over Marley's own OpenFile/save seams.
No copyleft source consulted.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — Snapshot = (mtime, len): cheap, no hashing; mtime alone misses
  same-second rewrites — len breaks most ties; a same-mtime-same-len
  rewrite is accepted v1 exposure (the watcher upgrade closes it).
- D2 — Deleted beats dirty-ness: `Deleted` whenever the file is
  missing (clean OR dirty) — the buffer is the only surviving copy;
  never auto-close.
- D3 — Keep-mine re-snapshots (acknowledge THIS disk state) rather
  than suppressing forever — the next external change re-flags.
- D4 — The focus-regained check runs for the ACTIVE file only (the
  visible buffer); background tabs check on activate. Cheap + covers
  the agent loop.
- D5 — All fs access stays in the shim (a `disk_state(path)` stat
  wrapper + the existing read/write); the pure layer sees VALUES only
  (§14 — tests use tempdirs through the real fs).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `external_action` shall implement the decision table exactly: Missing→Deleted (dirty or clean); Present==snapshot→Noop; Present≠snapshot ∧ clean→CleanReload; Present≠snapshot ∧ dirty→Conflict; no-snapshot (never loaded)→Noop. | pure units (full table) |
| REQ-002 | A CLEAN buffer whose file changed on disk shall silently reload at the next choke point (focus-regain edge, tab activate, or pre-⌘S): buffer text = disk text, caret/scroll clamped into range, nonce re-minted, a "reloaded from disk" flash shown, and the file stays clean. | headless (std::fs rewrite + focus/activate) |
| REQ-003 | A DIRTY buffer whose file changed on disk shall show the non-modal per-file conflict banner with the dirty ● retained and the buffer text untouched; Keep-mine shall dismiss + re-snapshot (no re-flag until the next change); Reload shall replace the buffer from disk and clear the dirty state. | headless + banner state asserts |
| REQ-004 | ⌘S on a conflicted file shall NOT write on the first press (warning flash, armed) and shall overwrite + clear the conflict on the second press; ⌘S on a deleted file shall recreate it. | headless (fs asserts) |
| REQ-005 | The check sites shall be exactly the CHOKE SET — the window-focus-regained edge, every activation that reveals an editor buffer (file-tab activate, tab switch incl. ⌘1-9/⌘]/⌘[, workspace cycle, rail jump, close-reveal, open-switch), and pre-⌘S — with no watcher thread and no per-frame stat: a disk change with the window active and no interaction shall NOT be detected until the next choke event. (Amended at inspect: "exactly three" under-counted the reveal-activations D4 already implied.) | headless negative assert + code review |

## Phase Plan
- P2 exact types + choke wiring + banner render + test plan; P3
  implement; P3.5 critic (decision-table completeness, the focus-edge
  detector's frame semantics, save-arm state machine, banner routing
  per #267); P4 units + headless + gate (driven ENV-BLOCKED protocol
  if still locked); P5 docs.
