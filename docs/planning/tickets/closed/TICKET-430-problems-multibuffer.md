# TICKET-430 — Problems panel, editable form (B-c step 6)

- **Ticket:** LOCAL #430 (feature, M32)
- **Tags:** editor, diagnostics, multibuffer, problems, b-c
- **Created:** 2026-08-14
- **Provenance:** the roadmap's second named multibuffer unlock ("the #327 problems
  panel's editable form"), under Chad's "/work 425 to 430" directive; shelf:
  `../../design-notes/display-map-shelf.md`
- **Status:** closed (2026-08-15 — shipped through the full pipeline; see docs/planning/pipeline/completed/430-problems-multibuffer.{spec,notes}.md)

## Summary

The B-c chain's closing consumer. The #327 problems panel (today a jump-list over LSP
diagnostics) gains its editable form: an action materializes the current diagnostics as a
multibuffer — each diagnostic an excerpt with context, grouped under its file header,
severity-marked, the message visible with the code — so a fix session is "type into the
problems list" instead of round-tripping jump-by-jump. Edits write through (#428);
diagnostics refresh as the LSP republishes after edits/saves, and the surface's refresh
policy (live re-excerpt vs explicit refresh) is a design-phase decision taken from the
reference behavior + the #331/#327 latch lessons (a stale card must never wedge). The
panel's existing jump-list form stays — this adds the editable view, not a replacement
(the exact entry point — panel action vs palette — is a design call).

## Acceptance

Headline: from a workspace with N diagnostics across M files, one action opens the
editable diagnostics surface; fixing one in place (type + save) removes it on the next
publish without closing the surface; jump-to-source still works per excerpt. Full EARS at
promotion.

## React-first (parity)

UI-affecting — `overlays/ProblemsPanel.tsx` (or its multibuffer sibling) is designed in
the POC first: severity glyphs, message rows, context excerpts, the editable interaction;
screenshot + read the PNG; then the 1:1 port.
