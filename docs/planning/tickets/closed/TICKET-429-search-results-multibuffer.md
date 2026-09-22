# TICKET-429 — ⌘⇧F phase 2: editable results + Replace All (B-c step 5)

- **Ticket:** LOCAL #429 (feature, M32)
- **Tags:** editor, search, multibuffer, replace, b-c
- **Created:** 2026-08-14
- **Provenance:** the #326 phase-2 unlock the roadmap names for the multibuffer
  ("editable results + replace-all"), under Chad's "/work 425 to 430" directive; shelf:
  `../../design-notes/display-map-shelf.md`
- **Status:** closed (2026-08-14 — shipped through the full pipeline; see docs/planning/pipeline/completed/429-search-results-multibuffer.{spec,notes}.md)

## Summary

Project search completes. The #427/#428 surface becomes ⌘⇧F's real results form: hits
open as the editable multibuffer (the #427 action graduates from "extra action" to the
canonical results view — the overlay list stays as the picker; the exact split is a
design decision from the observed reference). On top of it, **Replace All**: the search
bar grows a replace field (reusing #339/#347's regex + capture-group semantics — `$1`,
`${name}`, `$$` — and the in-buffer replace's corruption fixes), applying across every hit
as ONE undoable story through the #322-style multi-file engine, save-orchestrated like
#314/#354 (never blocking, origin-bound). Per-hit exclusion (dismiss a hit before
replacing) is in scope if the reference behavior carries it — a design-phase call.

## Acceptance

Headline: a project-wide query + replacement with a capture group applies to every hit
across files in one action, is reverted by one undo, and persists via the standing save
orchestration; editing directly inside a result excerpt keeps working (#428's contract).
Full EARS at promotion.

## React-first (parity)

UI-affecting — `overlays/ProjectSearch.tsx` gains the replace field + the results-as-
multibuffer flow in the POC first; screenshot + read the PNG; MARLEY-PARITY.md's project-
search row re-baselines.
