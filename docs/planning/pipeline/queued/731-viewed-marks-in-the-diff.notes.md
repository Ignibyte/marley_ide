# Viewed marks in the project diff — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-731-viewed-marks-in-the-diff.md
- **Pipeline spec:** 731-viewed-marks-in-the-diff.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** plannotator finding 6 (`docs/planning/intake/plannotator-findings.md`): "A Viewed
  mark per file in the project diff that survives a restart and clears when an agent rewrites the
  file. An additive touch to Zed's `git_ui`." Chad, 2026-10-11, on the finish-line list
  (`docs/planning/design-notes/finish-line-2026-10-11.md`): "11.) yes". It waits under Deliberate
  with the rest of the review batch until after the finish.
- **Classification:** feature, small to medium. The touch lands in `crates/editor` (the header), not
  in `git_ui` as the ticket guessed: the header is the editor's, and `git_ui` reaches it through the
  same `Addon` seam Marley uses.
- **Recall (§18.3):**
  - AD-claude-522-review-notes-go-to-an-idle-terminal-agent-and-stay-marked-sent-001: the last
    touch to the diff's editor; it reached the right-hand editor through `act_as::<Editor>`.
  - The #522 visual check (`522-review-notes-to-the-agent.notes.md`, Phase 3): in the headless
    sway the first click on the diff's toolbar after typing in a note did nothing, a second worked.
    The Viewed box is in the header, not the toolbar; the scenario still waits a moment after the
    diff loads before it clicks.
  - Key-value store users in Marley: `launch.rs:335-360` (a sha256 per folder and name),
    `agent_control.rs:30-31` (a scope and a key).
  - No ledger entry on diff headers or folding.
- **Discovery:**
  - `crates/editor/src/editor.rs`: `pub trait Addon` (786-818), `render_buffer_header_controls`
    (789), `register_addon` (11175), `addons` (1180).
  - `crates/editor/src/element/header.rs`: `render_buffer_header` (627); the addon controls,
    `.take(1)` (806-814); the fold chevron (757-802); the context menu from every addon (1085-1096);
    the sticky header (184).
  - `crates/git_ui/src/git_panel.rs`: `GitPanelAddon::render_buffer_header_controls` (9463-9486),
    the stage checkbox (7993-8046).
  - `crates/git_ui/src/project_diff.rs`: `GitPanelAddon` registered on the right-hand editor (250);
    `editor` (335); `multibuffer` (340); `deploy_at_project_path` (188).
  - `crates/git_ui/src/diff_multibuffer.rs`: `refresh` (626-736), `register_buffer` (439-595), the
    per-file diff subscription (451-474), the fold of deleted and untracked files (567-573, 728).
  - `crates/editor/src/fold.rs`: `fold_buffer` (618), `unfold_buffer` (663), `is_buffer_folded`
    (678); `EditorEvent::BufferFoldToggled` (`editor.rs:12592`).
  - `crates/buffer_diff/src/buffer_diff.rs`: `base_text_string` (343), `DiffChanged` (1602-1613).
  - `crates/db/src/kvp.rs`: `KeyValueStore::global` (41), `scoped` (89-140).
- **Decisions:** D1 to D5 in the spec.

### Design
- **Zed touch, `crates/editor/src/editor.rs`:** `Addon::render_buffer_header_trailing_controls(&self,
  &ExcerptBoundaryInfo, &language::BufferSnapshot, &Window, &App) -> Option<AnyElement>`, default
  `None`, with `// Marley:`.
- **Zed touch, `crates/editor/src/element/header.rs`:** after the existing controls, every addon's
  trailing controls in a row, with `// Marley:`. No upstream addon implements the method, so
  upstream's headers are unchanged.
- **New `crates/marley_workbench/src/viewed_marks.rs`** (Marley crate):
  - `init`: `cx.observe_new` on `ProjectDiff`; each new diff's right-hand editor gets a
    `ViewedAddon` holding an `Entity<ViewedMarks>`.
  - `ViewedMarks`: the diff's repository folder, the marks by repo path with their fingerprints,
    and the store's scope `marley-viewed`. It loads the marks for the files the diff shows, and on
    each file's `DiffChanged` (through the multibuffer's diffs) recomputes the fingerprint of a
    marked file: a different one deletes the key and unfolds the file.
  - The fingerprint: sha256 of the base text, a NUL, and the buffer's text (`sha2`, already a
    dependency).
  - `ViewedAddon::render_buffer_header_trailing_controls`: a `Checkbox` labelled Viewed, ticked
    from `ViewedMarks`; a click toggles the mark, folds or unfolds the file through the editor, and
    writes or deletes the key on the background executor.
- **`crates/marley_workbench/src/marley_workbench.rs`:** the module and its `init`.
- **`docs/marley/guide.md`:** the Viewed box under the project diff.
- **File manifest:** `crates/editor/src/editor.rs`, `crates/editor/src/element/header.rs` (Zed
  crate); `viewed_marks.rs` (new), `marley_workbench.rs` (Marley crate); the guide; the scenario.
- **Ledger rows (`docs/marley/zed-touchpoints.md`, before the edits):**
  - `crates/editor/src/editor.rs`: `Addon::render_buffer_header_trailing_controls`, default none
    (#731). Why: a second addon's control in a buffer header, after the first's. On merge: keep the
    method; if upstream lets several addons draw header controls, move Marley's addon onto that.
  - `crates/editor/src/element/header.rs`: the trailing controls drawn after the existing ones
    (#731). Same why and merge note.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 002 | `git: diff` on the repository with two changed files; clicks Viewed on `alpha.txt` | 731-01-ticked: the box ticked, `alpha.txt` folded, `beta.txt` open |
| 003 | `quit_marley`, `launch_marley`, `git: diff` | 731-02-restart: `alpha.txt` still ticked and folded |
| 004 | Appends a line to `alpha.txt` from the scenario's shell, waits for the reload | 731-03-changed: the box unticked, `alpha.txt` unfolded with the new line |
| 005 | (review of the diff: untick, unfold, the key deleted) | — |

The box's position is found on a first run (`STOP_AT_DIFF=1`), as #739's scenario found its buttons.

### Risks
- `DiffChanged` also fires while the diff is recomputed with no change to its text. The mark clears
  only when the fingerprint differs, so a recompute alone keeps it.
- A file with a very large diff costs one sha256 per change; the work runs where the diff's own
  refresh already runs, and only for marked files.
- The sticky header draws through the same function, so it shows the box too; P3 looks for a
  doubled box when a header is stuck.
