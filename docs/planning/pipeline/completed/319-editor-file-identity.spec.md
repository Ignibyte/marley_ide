---
pipeline_id: c004a399-78d9-4da0-9ee1-a1965a9d1adf
ticket: docs/planning/tickets/open/TICKET-319-editor-file-identity-lsp-paths.md
status: Phase 5 — Complete PASS
title: Editor file identity — one canonical spelling for an open file (symlinked-root fix)
type: bug
milestone: M20
references:
  - docs/planning/pipeline/completed/312-lsp-goto-definition.notes.md
  - docs/planning/pipeline/completed/310-lsp-diagnostics.notes.md
  - docs/planning/knowledge/prevention-rules.md (PR-claude-external-uri-key-normalize-both-sides-001; the #322 canonical-compare rule at prevention-rules.md:292; the #289 identical-root rule at :1203)
  - docs/planning/knowledge/failures.md (BF-rename-open-file-misroutes-to-disk-under-symlinked-root-001)
---

## Title
An open editor file currently stores whichever path spelling its producer used —
the file tree / finder / session restore store `root.join(rel)` (verbatim root),
while LSP jump targets (F12, ⇧F12, ⌘T, search, problems) store the server's
canonical spelling. Under a symlinked project root (`/tmp`→`/private/tmp`,
symlinked `~/dev`, MDM-relocated home) the spellings differ, so every raw
`PathBuf ==` over the stored path misbehaves: the F12 landing check fails on an
already-open file ("Can't open the definition's file" on a tab that just
switched, caret never placed), terminal-diagnostic gutter rows drop, and a
canonicalize-failure edge still opens a second buffer for one file — defeating
the #249 "re-opening never clobbers unsaved edits" guarantee. Fix: pick ONE
identity — the canonical spelling — normalized at the storage seam by a single
shared helper, so every producer and every compare agrees.

## Scope
### In
- One shared, non-failing normalization helper (canonicalize with the resolved
  join as fallback — the same fallback `same_file` uses) producing the stored
  identity for every open-editor path.
- Adoption at the storage/convergence seams: `load_code_view_state`,
  `open_file_in_viewer` / `split_file_pane` probes, `open_editor_instance`,
  session-restore arms (`TabLayout::Code`, `PaneKind::CodeView`), arrangement
  `key_ok` + mount, terminal file-ref / block-failure jump joins, the git-diff
  header click's hand-join.
- The fresh-resolve compare sites that must match the stored identity:
  `open_and_place_caret`'s landing check, `links.rs` `diagnostics_for_file` /
  `trace_diagnostic_rows`.
- Session/arrangement back-compat: a layout persisted with the old root-joined
  spelling restores to the canonical identity with no duplicate tabs (#205-style
  tier check).
- Update the #312 harness comments (`seed_definition_files`) to point at the fix.

### Out (explicitly deferred)
- Workspace/root dedupe semantics — D-OPEN-DEDUPE-SCOPE stands: aliased ROOT
  spellings remain two workspaces; `find_open`'s raw root half and
  `tabs.rs` project-already-open stay as documented.
- Any LSP-layer change — the canonical keying there is the #310 F1 fix and is
  load-bearing.
- Windows canonicalization (`dunce::canonicalize` UNC handling) — macOS-only
  today; noted for the Windows-parity ticket (TICKET-6).
- The `HitKind::File` label-as-path producer cleanup beyond normalization.
- Zed-style internal-symlink UI affordances (`is_external` badge etc.).

## Reference (§20)
Zed (the editor reference — same-gpui stack). Behavior matched: an editor keeps
ONE identity per file on disk regardless of the spelling it was reached by. The
behavior map records Zed's worktree `Entry` carrying `canonical_path` for
symlinked entries (`docs/zed_architecture/crates/worktree.md:52`), identity as
the canonical (worktree, rel-path) pair (`docs/zed_architecture/crates/project.md:54`),
and `canonicalize` as a first-class `Fs` query (`docs/zed_architecture/crates/fs.md:41`)
— i.e. canonicalization happens at the fs/worktree boundary, not ad-hoc at
compare sites. Warp's map shows the same discipline at the type level:
`CanonicalizedPath`, a PathBuf newtype canonicalized at construction
(`docs/warp_architecture/crates/repo_metadata.md:28`, with the Borrow/Hash
uniformity warning at `:104`) and `StandardizedPath::from_local_canonicalized`
(`docs/warp_architecture/crates/warp_util.md:27`). Marley matches the BEHAVIOR
(one canonical identity at the storage boundary); the mechanism is Marley's own.

### Prior art
1. **Behavior maps** — Zed: `worktree.md:52` (`canonical_path` on symlinked
   entries), `project.md:54` (`ProjectPath` = canonical (worktree, rel) key),
   `fs.md:41` (`canonicalize` in the Fs trait). Warp: `repo_metadata.md:28,104`
   (`CanonicalizedPath` newtype, canonicalized at construction via
   `dunce::canonicalize`), `warp_util.md:27` (`StandardizedPath::from_local_canonicalized`).
   Both references canonicalize at a single boundary and let identity flow from it.
2. **Published** — LSP 3.17 leaves `DocumentUri` an opaque string and warns that
   client and server encodings/spellings may differ, so the CLIENT owns identity
   normalization; rust-analyzer returns canonicalized file paths in URIs (why F12
   targets arrive canonical). VS Code's long-standing symlink double-open is the
   documented anti-pattern this ticket avoids; Zed dedupes via canonical paths.
3. **Our permissive deps** — no shipped crate owns symlink-resolving identity;
   `std::fs::canonicalize` is the primitive. Checked `gpui_util` (Apache-2.0,
   in-tree) `paths.rs`: `SanitizedPath` is `dunce::simplified` (lexical Windows
   UNC trim only) and `normalize_lexically` is explicitly lexical — its own docs
   defer symlink resolution to `Path::canonicalize`. `dunce` is already in-tree
   via gpui_util if Windows ever needs UNC-safe canonicalize (TICKET-6). The
   `same-file` crate (in-tree via walkdir/ignore) does pairwise (dev,inode)
   equality — considered, NOT adopted: the editor needs a storable `Hash`/`Eq`
   KEY (`git_marks`, `editor_folds`, the #310 LSP docs map are `HashMap<PathBuf,…>`),
   and canonical `PathBuf` is that key. In-repo: #322's `same_file`
   (editor_surface.rs:28) and #310's `LspHost::absolute` (lsp_host.rs:268) —
   this fix extends their existing discipline from compare-time to storage-time.

## React-first (parity)
N/A — no UI delta: this is path-identity normalization inside the open/compare
seams. No chrome, overlay, layout, type, or color changes; the only user-visible
effect is bug-suppression (F12 lands with a caret instead of a false "Can't open
the definition's file" flash, and one file can no longer occupy two buffers).

## Locked-In Decisions
- D1 — The LSP layer stays canonical, untouched. Keying by canonical PathBuf is
  the #310 F1 fix (`PR-claude-external-uri-key-normalize-both-sides-001`);
  un-canonicalizing it would resurrect silently-dropped diagnostics.
- D2 — ONE stored identity for an open editor file: the CANONICAL spelling when
  the file exists, else the resolved join — produced by a single shared helper
  whose fallback matches `same_file`'s (`canonicalize().unwrap_or(join)`), so
  storage and compare can never disagree. No second identity field.
- D3 — Workspace identity is out of scope: D-OPEN-DEDUPE-SCOPE (content.rs:203)
  stands; `find_open`'s raw root half is untouched. This ticket fixes FILE
  identity within a workspace.
- D4 — Persisted layouts (session `V=` codec, split `c=`, arrangement keys) keep
  their wire format; restore re-resolves through the SAME helper. Old root-joined
  spellings must restore with no duplicate tabs (#205-style back-compat tier).
- D5 — `same_file` remains the compare primitive where one side may not exist;
  sites where both sides are stored identities may use `==` only if Design shows
  both sides route through the helper.
- D6 — Design confirms seam placement (helper in `marley_project` next to
  `resolve_under_root` vs `marley_app`) with the LSP host-spawn containment gate
  (`app.rs:5430` `starts_with(root)`) explicitly re-checked: canonicalizing the
  file side while the root stays verbatim must not break containment (the #322
  rule: canonicalize BOTH sides of a containment check).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method (gate exit code,
negative smoke, or review).

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a file is opened via the file tree under a symlinked project root and then reached again via an LSP jump (goto-definition), the system shall switch to the existing buffer — never a second buffer for the same file on disk — preserving unsaved edits (#249). | Unit/headless test: symlinked-root fixture, tree-open + dirty edit + F12 to the same file → one `EditorInstance`, edits intact. |
| REQ-002 | WHEN a goto-definition target is already open under a different alias spelling, the system shall place the caret at the target (landing verification passes) and shall not surface "Can't open the definition's file". | Headless drive: symlinked root, open via joined spelling, jump via canonical uri → caret placed, no error flash. |
| REQ-003 | WHEN an editor file path is stored (any producer: tree, finder, palette, terminal file-ref, git-diff header, block failure, session restore, arrangement mount), the system shall store the canonical spelling when the target exists, else the resolved join. | Unit test on the helper (symlink fixture → canonical; nonexistent path → join) + review of producer adoption list. |
| REQ-004 | WHEN a session or arrangement persisted with the old root-joined spelling is restored under a symlinked root, the system shall restore each editor to the canonical identity with no duplicate tabs and arrangement `key_ok` still matching. | Restore test with a legacy `V=`/`c=` payload + arrangement snapshot under a symlinked root. |
| REQ-005 | WHEN terminal diagnostics reference a file that is open (under any alias spelling), `diagnostics_for_file` / `trace_diagnostic_rows` shall attach rows to that open file. | Unit test: open canonical-spelled file, diagnostics keyed by joined spelling → rows returned. |
| REQ-006 | WHILE the fix is in place, the LSP-side canonical keying and the workspace-scope root semantics shall be unchanged. | Existing #310/#322/#397 suites green (gate:3); `find_open` root half untouched (review). |
| REQ-007 | The change shall pass the full quality bar on touched lines. | `scripts/gates.sh --diff` green (exit code). |

## Phase Plan
- **P2 Design** — pick the helper's home + exact contract; enumerate every
  adoption site from the Phase 1 discovery map (notes); decide landing-check
  form (`==` vs `same_file`) per D5; the containment-gate re-check (D6);
  arrangement/session back-compat mechanics; regression test plan incl.
  symlink fixtures, legacy-restore payload, and the #312 harness comment update.
- **P3 Implement** — code per design.
- **P3.5 Inspect** — independent critics vs the diff; fix the real findings.
- **P4 Validate** — write + RUN tests; gate green (`--diff`).
- **P5 Complete** — archive, ledger capture (§19), close the ticket (its BACKLOG
  row was removed at promotion).
