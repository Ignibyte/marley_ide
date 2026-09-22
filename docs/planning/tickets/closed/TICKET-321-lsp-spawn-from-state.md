# TICKET-321 — A RESTORED editor tab never spawns the LSP

- **Forge ticket:** #321 `afffcc4b-dc2f-4fc7-b166-f4057c26cbd5` (bug, M20)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `55b2817a-8e0b-4be3-8357-60f3cd2589bc`
- **Pipeline doc:** ../../pipeline/active/321-lsp-spawn-from-state.spec.md
- **Source ticket:** the M22 integrity-five shelf — ../../design-notes/integrity-five-shelf.md
- **Status:** closed

## Summary

Quit Marley with a `.rs` file open — the thing #205's persistence exists to preserve — relaunch, and
you silently get no diagnostics, no hover, no go-to-definition and no completions on the file you are
looking at. #308's only LSP spawn trigger is the open *gesture* (`open_file_in_viewer` →
`ensure_lsp_for_opened_file`), and a restored tab never gestures. Observed directly during #313's
Phase-4 drive: booted with `src/main.rs` open, no `lsp:` footer segment at all and no rust-analyzer
process; reseeding a terminal-only shell and clicking the same file in the tree spawned it
immediately. Every M20/M21 feature rides the host, so all of them are dead until the user happens to
open some *other* file.

The fix is smaller than the ticket sketched. The pump already derives doc-sync from state every tick
— it builds the open-doc set from the tabs' editors and reconciles the active root's host to it
(didOpen/didChange/didClose). Host **creation** is the one remaining event-derived piece, so it joins
that same derivation, consuming the same vec, and the gesture trigger is removed so there is one
trigger rather than two. Late creation is already wire-legal: `reconcile`'s newly-seen-doc arm sends
didOpen, which is exactly how a respawned server re-opens its documents.

## Acceptance

A restored editor tab spawns its workspace's rust-analyzer with no gesture, sends a `didOpen`
carrying the file's real text once the server is Ready, and shows the `lsp:` footer segment — while a
fresh gesture-open still spawns within one pump tick, and the spawn gate (`.rs` · under root · no
existing host · root has `Cargo.toml`) is unchanged, so a bare-tempdir headless boot still never
spawns one. Full EARS criteria (REQ-001 … REQ-006) live in the pipeline spec.
