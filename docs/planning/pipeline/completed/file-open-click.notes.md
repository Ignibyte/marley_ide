# click a file-tree row to open it — Notes

- **Forge ticket:** #59 `f35cfb9c-c034-4510-8178-500f7991e975`
- **AAR:** `2bffe609-9c53-416a-afbe-d4b1de755aa1`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-059-file-open-click.md

## Phase 1 — Plan
- **Request:** forge #59 (M2.B seq-1, auto-approved) — completes #56's deferred file-row click. First
  M2.B ticket.
- **Classification:** work pipeline, `feature`, a small PURE `path_at` (marley_project) + an app.rs
  SHIM tweak. UI — validate self-test-captures.
- **Design:** `path_at_in` recursion mirrors `visit`/`toggle_at` (proven traversal), tracking a `prefix`
  path joined per node → the row's full relative path at the target ordinal.
- **Shim:** the Left-dock click currently attaches on_mouse_down only to DIR rows (toggle). #59 attaches
  it to ALL rows — dir→toggle, file→`path_at`→write to PTY (reuse `focused_state_mut().session.write_bytes`,
  the cmd-P write path).
- **Hollow-MSI note (#53):** `path_at`'s ancestor-join + cursor arithmetic give viable mutants; a NESTED
  path test (not just a root-level one) is load-bearing (catches a dropped `prefix.join`).
- **AAR id:** `2bffe609-9c53-416a-afbe-d4b1de755aa1`.

## Phase 2 — Design

### PURE — ADD to `crates/marley_project/src/lib.rs` (FileTree impl + a private fn)
```rust
    /// The full relative path of the node at visible-row `index` (each ancestor's name joined down
    /// from the root), or `None` when `index` is out of range. Walks the same pre-order as
    /// [`visible_rows`](FileTree::visible_rows), so a rendered row's ordinal maps straight to its path.
    pub fn path_at(&self, index: usize) -> Option<PathBuf> {
        let mut cursor = 0;
        path_at_in(&self.roots, Path::new(""), index, &mut cursor)
    }
// ...
fn path_at_in(nodes: &[Node], prefix: &Path, target: usize, cursor: &mut usize) -> Option<PathBuf> {
    for node in nodes {
        let node_path = prefix.join(&node.name);
        if *cursor == target {
            return Some(node_path);
        }
        *cursor += 1;
        if node.is_dir && !node.collapsed {
            if let Some(found) = path_at_in(&node.children, &node_path, target, cursor) {
                return Some(found);
            }
        }
    }
    None
}
```

### SHIM — `app.rs` Left-dock row loop (mutants::skip + cov-excluded)
The dir-row `if row.is_dir { on_mouse_down → toggle }` gains an `else` for FILE rows:
```rust
} else {
    entry = entry.on_mouse_down(MouseButton::Left, cx.listener(move |view, _event: &MouseDownEvent, _window, cx| {
        if let Some(path) = view.file_tree.path_at(index) {
            let bytes = path.to_string_lossy().into_owned().into_bytes();
            if let Some(state) = view.workspace.focused_state_mut() {
                let _ = state.session.write_bytes(&bytes);
            }
        }
        cx.notify();
    }));
}
```

### File manifest
- MODIFY `crates/marley_project/src/lib.rs` — `path_at` + `path_at_in` + tests.
- MODIFY `crates/marley_app/src/app.rs` — the file-row click else-branch.

### Mutation Targets
- `path_at_in` — `prefix.join(&node.name)` (a dropped join → just the leaf name → the NESTED-path test
  catches it); `*cursor == target`; `*cursor += 1`; the `is_dir && !collapsed` recurse guard (mirrors
  `toggle_at`, already MSI-proven in #56).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `path_at_returns_full_nested_path` — `from_files(["a/b/x","a/c"])` → `path_at(2)=="a/b/x"` (NESTED — proves the ancestor join) | unit |
| REQ-002 | `path_at_root_and_out_of_range` — `path_at(0)=="a"`, `path_at(3)=="a/c"`, `path_at(99)==None` | unit |
| REQ-003 | file-row click → path at prompt | self-test (drive.swift clickat a file row → capture) |
| REQ-004 | gate GREEN, cov/MSI 100 path_at; app shim excluded | gate |

Uncoverable: the app.rs file-row click — masked + cov-excluded, proven by REQ-003.

### Risks / decisions
- D-2.1 `path_at_in` MUST mirror `visit`/`toggle_at` traversal (the invariant proven for `toggle` in
  #56) — the same combined nested-collapse test class guards it; here a NESTED path is the key (a
  root-only test wouldn't catch a dropped `prefix.join`). D-2.2 the click writes the path relative to
  the project root (what `path_at` returns) — a first cut; an absolute-path or editor-open is later.

## Phase 3 — Implement
- **Built (PURE):** `FileTree::path_at` + the free `path_at_in` (marley_project) — verbatim from the
  design (mirrors `toggle_at`, tracking a `prefix` joined per node).
- **Built (SHIM, app.rs):** the file-tree row loop's `if row.is_dir {…toggle…}` gained an `else` for
  FILE rows → `path_at(index)` → `focused_state_mut().session.write_bytes(path)`.
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley_project -p marley` 0 err; clippy `-D warnings`
  OK. Tests + self-test are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (33-probe + isolated cargo-mutants + a hand cross-check). Verdict: **pure code CORRECT**
  — no defect; parity/join/bounds all verified.
- **Findings (no code fix — P4 test guidance):**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | F1 | MED | The `is_dir && !collapsed` recurse guard (lib.rs:271) `&&→||` mutant SURVIVES nested+bounds tests (MSI 99) — only observable when a COLLAPSED dir with children is walked (an all-expanded tree makes the guard true either way). Same load-bearing invariant as #56's toggle. | P4 adds a COLLAPSE test: `from_files(["a/b/x","a/c","d/y"]); toggle(0); path_at(1)==Some("d"); path_at(2)==Some("d/y"); path_at(0)==Some("a")` → MSI 100 (9/9). |
  | F2 | LOW | cargo-mutants makes NO mutant for `prefix.join(&node.name)` — a leaf-only regression is caught ONLY behaviorally; keep REQ-001 NESTED (`path_at(2)=="a/b/x"`), not root-only. | Already planned (nested). |
  | F3 | LOW | Inserted path isn't shell-escaped (`my file.txt` → unescaped space). | Accept — insert-not-run first cut; note for later. |
  | — | LOW/process | A shared `CARGO_TARGET_DIR` gives FALSE cargo-mutants "caught" (stale-binary reuse). The real gate (`mutation_g`, copy-mode, no override) is SAFE. | Noted — scoped runs must not force a shared target dir. |
- **Verified (probe):** full nested path (`a/b/x`), exact parity with visible_rows/toggle incl. the
  collapse case, `None` on out-of-range/empty, `Some(dir path)` for a collapsed-dir row, no panic; shim
  reuses the same `index` as toggle, writes the relative path with NO trailing `\n` (insert not run).
- **No code change** — F1 = the P4 collapse test (learned from #56).

## Phase 4 — Validate
- **Tests added** (`lib.rs`): `path_at_returns_full_nested_path` (REQ-001+002 — nested `a/b/x`, root,
  out-of-range, empty-tree) + `path_at_respects_collapse` (F1 — the collapse case that kills the
  recurse-guard mutant, learned from #56).
- **Runs (actual):** `cargo nextest -p marley_project -E 'test(path_at)'` → 2 passed.
- **SELF-TEST — caught + fixed a REAL bug (this is exactly its purpose):** the first cut wrote the path
  via `session.write_bytes` (like cmd-P #57). Driving a file-row click showed the click FIRED,
  `path_at` returned the correct path, and the write returned Ok — but NOTHING appeared at the prompt.
  A `\r\n`-terminated write RAN end-to-end (proving the PTY pipe works), which isolated the cause: in
  COOKED mode **Marley owns the line editor (`state.buffer`)** and only sends the whole line to the PTY
  on Enter, so a raw `write_bytes` reaches the shell's ZLE but not the rendered buffer. FIX (design
  deviation, correct approach): the file-click now inserts the path into `state.buffer` at the caret via
  `buffer.edit(caret..caret, path, Human)` + advances the caret — mirroring the cmd-V paste path.
  Re-drove the click → the prompt shows `❯ Marley .claude/commands/pipeline/complete.md` (`scratchpad/
  fileclick_fixed.png`). REQ-003 PASS.
- **Follow-up flagged:** #57's cmd-P finder Enter has the SAME latent bug (it `write_bytes` the path →
  won't show in the cooked buffer). Worth a small follow-up ticket to route it through `buffer.edit` too.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, MSI 100.0% (path_at
  mutants incl. the recurse-guard killed by the collapse test).
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; marley_project.md path_at line.
- **Knowledge:** PR `PR-claude-insert-at-prompt-goes-to-cooked-buffer-not-raw-pty-001` (HIGH — insert via buffer.edit, not raw write_bytes; the self-test caught it). Follow-up ticket #65 filed (cmd-P finder Enter has the same bug). aar-submit completed (5).
- **Ticket:** forge #59 → done; archived. **1/6 of M2.B.**
