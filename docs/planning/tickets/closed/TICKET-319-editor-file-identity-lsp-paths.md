# TICKET-319 — Editor file identity vs LSP canonical paths — a symlinked project root can open the same file twice

- **Ticket:** LOCAL #319 (bug, M20)
- **Tags:** M20, bug, editor, lsp, paths, from-validate, 312-followup
- **Created:** 2026-07-15
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id 6d660d5a-a7e6-4a6e-bccb-ecdc5409f84e)
- **Status:** closed (2026-08-10 — pipeline docs/planning/pipeline/completed/319-editor-file-identity.spec.md; shipped: one canonical stored identity via marley_project::canonical_under_root + consumer-family alignment; the #312 harness comment now points at the fix)

## Description

Surfaced by #312's Phase 4 headless drive (not a #312 regression — a pre-existing tension #312 is the first feature to expose).

THE TENSION: the LSP layer keys files by CANONICAL path on purpose — that is the #310 inspect F1 fix, recorded as `PR-claude-external-uri-key-normalize-both-sides-001` ("key by canonical PathBuf, the SAME normalization on both sides"). `uri_for` = `file_uri(absolute(path))`, and a server's returned uri decodes via `path_from_file_uri` to whatever the server spelled — rust-analyzer spells canonical.

But the EDITOR's file identity is root-relative: `resolve_under_root(project_root, path)` joins a tree/finder path onto the project root, and `CodeViewState` stores exactly that. `EditorSurface::open` switches to an existing tab by comparing `path` (editor_surface.rs — `self.files.iter().position(...)`).

SO: if the project root is reached through a symlink, the two spellings differ, and F12 to a file already open via the file tree opens a SECOND tab for the same file. Two tabs, two buffers, one file on disk — the #249 "re-opening no longer clobbers unsaved edits" guarantee is defeated because the second tab is a different buffer entirely.

HOW IT SURFACED: on macOS a `TempDir` lives at `/var/folders/...`, a symlink to `/private/var/folders/...`. #312's drive seeded a fixture at the `/var` spelling and asserted the jump landed there; it landed at `/private/var` instead — correctly, because the uri named the canonical path. The drive now canonicalizes its fixture (with a comment pointing here). A real project root under `~/Projects` is not symlinked, which is why this has never bitten in production.

REACHABILITY: needs a symlinked project root. Plausible ones: a project under `/tmp`, a symlinked `~/dev` → an external volume, or a home dir symlinked by an MDM/homedir-relocation setup.

SCOPE: pick ONE identity for an editor file and normalize on both sides — almost certainly canonicalize at `CodeViewState::new` / `resolve_under_root`, so the tree, the finder, terminal file-refs, and LSP targets all spell a file the same way. Then `EditorSurface::open`'s existing-tab match is sound for every producer.

WATCH OUT:
- `canonicalize` FAILS on a path that does not exist yet (a new/unsaved file) — needs a non-failing normalization, or the fallback must be well-defined rather than silently un-normalized.
- Do NOT "fix" this by making the LSP side non-canonical: that would resurrect the #310 F1 bug (an `@scope`/space/symlink path silently dropping every diagnostic).
- #243/#163 persist editor paths into the grid codec — changing the stored spelling touches restore; check the back-compat tier the way #205 did for cwds.
- The #312 drive's `seed_definition_files` canonicalizes and says why; if this is fixed, that comment should point at the fix instead.
