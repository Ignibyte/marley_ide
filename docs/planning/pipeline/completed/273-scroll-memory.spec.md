---
pipeline_id: e99e2c62-272c-40f8-81f3-3f12803515b4
ticket: forge#273 (0d3556f7-8948-4f21-904b-8f9a5dbb83bf) · local docs/planning/tickets/open/TICKET-273-scroll-memory.md
aar_id: 1ca979c4-1d8b-4994-9e56-33d2a8e332cb
status: Phase 5 — Complete PASS
title: Per-file editor scroll memory + the scroll_editor_to_row mechanism
type: bug
milestone: M17
references:
  - docs/marley_architecture/editor.md
---

## Title
Heal the disclosed #266 regression (inspect F4): the editor moved to ONE
shared `RootView.editor_scroll: UniformListScrollHandle`, so switching
file tabs CARRIES the scroll offset instead of restoring each file's own.
Remember a per-file scroll position on `OpenFile`, captured/restored at
the file-switch choke points, and ship the ONE shared
`scroll_editor_to_row(row)` helper (non-strict Center) that #270
caret-follow, #272 find-next, and the #212/#213 open-at-line fusion
tickets will consume.

## Scope
### In
- API ground truth (verified in vendored gpui 0.2.2 uniform_list.rs +
  div.rs): `logical_scroll_top_index()` is **test-support-only** — the
  SHIPPING capture path is the pub `base_handle.offset()` (pixel Point)
  and restore is `base_handle.set_offset(...)` (pixel-exact, preserves
  partial-row position); `scroll_to_item(ix, ScrollStrategy)` is the
  deferred row mechanism (non-strict = no-op when already visible).
- `OpenFile.scroll_px: f32` (+ surface accessors) — captured from
  `offset().y` BEFORE activate/open/close mutate the active file,
  restored after (a RootView `capture_editor_scroll()` /
  `restore_editor_scroll()` pair wrapping the switch sites: the file-tab
  strip click [app.rs:5964 region], `open_file_in_viewer`
  [app.rs:2448], the tab-strip × close).
- `scroll_editor_to_row(row)` on RootView →
  `editor_scroll.scroll_to_item(row, ScrollStrategy::Center)` — the
  shared mechanism; NOT wired to any new behavior this ticket (its
  consumers are #270/#272).
- A pure clamp for restore-after-external-shrink (a reloaded/smaller
  buffer must not restore past EOF — the offset clamps into the content
  height; the #268 nonce marks the generation).
- Headless + driven verification of the A-deep/B-top/back-to-A flow.

### Out (explicitly deferred)
- Caret-follow itself (#270 — consumes the helper next).
- Scroll persistence ACROSS RESTART (the #243 open-set persists paths,
  not offsets; a codec change is its own ticket if ever wanted).
- The #246 read-only pane (keeps its own `cv.scroll` model).
- Find-next/open-at-line wiring (#272/#212/#213).

## Reference (§20)
N/A — restoring Marley's OWN pre-#266 behavior (the per-file `cv.scroll`
memory the uniform_list swap traded away); the mechanism is gpui's public
Apache-2.0 scroll-handle API consumed as shipped.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — Store the PIXEL offset (`scroll_px: f32`), not a row index:
  `set_offset` restores exactly (mid-row positions survive), needs no
  cell-height math at the capture site, and the paint clamps overshoot.
  Row-based `scroll_to_item` is reserved for the SHARED helper where
  row semantics are wanted.
- D2 — Capture ONLY at the switch choke points (no per-frame mirror):
  the handle is the live truth while a file is active; the field is the
  parked value for inactive files.
- D3 — `scroll_editor_to_row` uses NON-STRICT `ScrollStrategy::Center`
  (a visible target row is a no-op — exactly the caret-follow/find-next
  semantic its consumers want).
- D4 — `OpenFile.scroll_px` does NOT persist across restart (session
  memory only; restart lands at top as today).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Switching file tabs shall restore each file's own scroll offset: scroll A deep, switch to B (its own position, top for fresh), switch back → A shows the SAME offset (pixel-exact). | headless (test-support `logical_scroll_top_index`/offset read) + driven capture |
| REQ-002 | Opening a NEW file (open_file_in_viewer / finder ⌘↵ / tree click) shall park the outgoing file's offset and start the new file at ITS remembered offset (top when fresh). | headless |
| REQ-003 | Closing a file tab shall not disturb the surviving files' remembered offsets, and the newly-active file shall restore its own. | headless |
| REQ-004 | After the active buffer SHRINKS below a remembered offset (external reload/undo), the restore shall clamp into range (never a blank over-scrolled viewport). | pure clamp units + a headless shrink case |
| REQ-005 | `scroll_editor_to_row(row)` shall bring `row` into view centered when off-screen and be a NO-OP when `row` is already visible (non-strict Center). | headless (off-screen row → top index moves; visible row → unchanged) |
| REQ-006 | The #246 read-only pane and the terminal scroll paths shall be byte-identical (untouched). | existing tests + driven capture |

## Phase Plan
- **P2 Design** — the exact capture/restore wrapper shape + call-site
  list, the clamp arithmetic, the accessor manifest, test plan.
- **P3 Implement** — OpenFile field + accessors; the RootView pair +
  helper; site wiring.
- **P3.5 Inspect** — critic (choke-point completeness — any switch path
  missed?; clamp fenceposts; handle-borrow safety in render).
- **P4 Validate** — units + headless flows + driven capture; gate --diff.
- **P5 Complete** — CHANGELOG, editor.md coda, AAR, archive, close.
