---
pipeline_id: 47dd805d-2725-4a40-88b2-c3a480734bc5
ticket: forge#205 (85a7388c-4d30-427c-a1b6-163ffb97edfb) · local docs/planning/tickets/open/TICKET-205-warp-session-cwd.md
aar_id: 592d2ba3-0e6b-461f-b088-c4769c8f3726
status: Phase 5 — Complete PASS
title: session persistence — restore each terminal pane's cwd on relaunch
type: feature
milestone: M12.2
references: []
---

## Title
Restore each terminal pane's working directory on relaunch (forge #205). #163 already persists the whole
shell — every project, tab, the PaneGroup split tree, pane kinds, and actives (relaunch-proven). The one
gap: `serialize_grid` records a terminal pane as just the kind char `"t"`, with NO cwd, so a restored
terminal respawns in the project ROOT, not its previous directory. Extend the pure grid-blob codec to
carry an optional per-terminal cwd and spawn the restored terminal in it (falling back to the root when
the cwd is gone).

## Scope
### In
- **Pure codec** (`grid_layout.rs` `serialize_grid`/`restore_grid` + `flatten` + the `GridLayout`
  carrier, cov/MSI 100): extend the terminal leaf encoding to carry an OPTIONAL cwd — `"t"` →
  `"t=<cwd>"` when a framing-safe cwd is known, else plain `"t"`. Back-compatible: an old-format `"t"`
  (or a malformed leaf) restores with no cwd → root. A cwd that would break the blob framing (contains a
  blob delimiter) is OMITTED (encode plain `t`) — the #163 D2 "unrepresentable → drop" stance. Round-trip
  identity + framing-drop + back-compat, never panics.
- **Shim** (app.rs, masked): (a) at serialize time supply each terminal pane's live cwd
  (`session.current_prompt().pwd`, the #201 source); (b) at restore (`restore_panes`) spawn each restored
  terminal in its persisted cwd via `spawn_session_in(cwd, …)` IF the cwd exists on disk, else fall back
  to the project root (the not-exists→redirect rule — an IO `Path::exists` check).

### Out (already done / deferred)
- The whole-shell structure / kinds / actives / project-root persistence — **#163, DONE**.
- A **debounce** on capture — capture already piggybacks the existing `persist_grid`/switch-site writes;
  a debounce is an optimization → deferred.
- Persisting a code/git pane's scroll position or a cockpit sub-state; a foreground command's process
  (the ticket says restore the SHELL in the cwd, not the running process — already the #163 behavior;
  we only change WHERE the fresh shell spawns).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — Per-terminal cwd rides the grid-blob leaf encoding** (`"t"` → `"t=<cwd>"`), an OPTIONAL
  extension (the #177 `T=title\x1fblob` precedent), so old blobs restore unchanged (no cwd → root).
- **D2 — Framing-safe cwd or drop it** (reuse/extend `breaks_framing`): a cwd containing a blob delimiter
  (`,` `:` `=` `\t` `\n` `\r` — design pins the exact set; `=` is the new `t=cwd` sub-delimiter) is
  omitted → plain `t` → restores to root. Never misparse; never panic.
- **D3 — The live cwd is `session.current_prompt().pwd`** (the #201 source that tracks `cd`), captured
  in the shim at serialize time (per terminal pane).
- **D4 — Restore honors the cwd only if it still exists** (`Path::exists`); a deleted/moved cwd → the
  project root fallback (the shim; an IO check, not pure).
- **D5 — Clean-room (§20).** Original codec extension; no Warp source consulted.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a terminal pane with a known framing-safe cwd is serialized then restored, the grid layout shall round-trip that cwd (encoded `t=<cwd>`, restored with it). | unit: `restore_grid(serialize_grid(…with cwd…))` yields the cwd; incl. a split with mixed per-terminal cwds. |
| REQ-002 | WHEN a terminal's cwd contains a blob delimiter, the serializer shall omit the cwd (encode plain `t`). | unit: a cwd with `,`/`:`/`=`/`\t` → the blob has no `t=…` for it → restores with no cwd. |
| REQ-003 | WHEN an old-format blob (plain `t`, no cwd) or a malformed leaf is restored, the terminal shall restore with no cwd (→ root) and never panic. | unit: `restore_grid("t")` / `restore_grid("H:t,f")` → terminals with no cwd; a junk leaf → default. |
| REQ-004 | WHEN a restored terminal has a persisted cwd, the shim shall spawn it in that cwd if the directory exists, else fall back to the project root. | review: `restore_panes` reads the cwd + `spawn_session_in(cwd if exists else root, …)`; driven if unlocked. |
| REQ-005 | WHEN the layout is persisted, each terminal's cwd shall be its live shell-reported pwd. | review: the serialize shim reads `current_prompt().pwd` per terminal pane. |

## Phase Plan
- **P2 Design** — pin the leaf encoding (`t=<cwd>`) + the `flatten`/`serialize_grid` signature change
  (a `cwds: HashMap<PaneId, String>` param) + the `GridLayout` cwd carrier + `restore_grid` parse + the
  `breaks_framing` delimiter set; the shim's serialize-time cwd capture + the `restore_panes` spawn-in-cwd
  (exists-check). `cargo mutants --list -f grid_layout.rs` for the real set. Confirm no split needed.
- **P3 Implement** — the codec + the shim wire.
- **P3.5 Inspect** — critics vs the diff (the framing/back-compat/round-trip, the exists-check, no
  regression to #163's existing round-trip); fix real findings.
- **P4 Validate** — the round-trip matrix + RUN; driven relaunch capture (if unlocked, else units +
  mechanism per the locked-screen rule); gate green.
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; archive; close.
