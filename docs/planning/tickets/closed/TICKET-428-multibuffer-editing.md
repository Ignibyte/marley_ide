# TICKET-428 — Editable multibuffer: excerpts write through (B-c step 4)

- **Ticket:** LOCAL #428 (feature, M32)
- **Tags:** editor, multibuffer, editing, undo, b-c
- **Created:** 2026-08-14
- **Provenance:** roadmap B-c chain (m22-editing-bar.md item 16, second slice) under
  Chad's "/work 425 to 430" directive; shelf: `../../design-notes/display-map-shelf.md`
- **Status:** closed

## Summary

The #427 stitched view becomes a real editor: typing, selection, and deletion inside an
excerpt route through the ONE insert mechanism to the correct underlying Buffer; excerpt
ranges rebase over the edits (anchor-composed positions — the multibuffer anchor is a
per-buffer anchor plus its excerpt, per the Zed §6 reference shape); undo/redo is one
cross-excerpt story; ⌘S saves the touched files through the shipped save orchestration.
The relationship to an already-open editor tab/pane of the same file is a design-phase
decision made from the LIVE instance model (the #397 one-buffer-many-views POC contract vs
#259's two-independent-Buffers-with-conflict-net) — whichever holds in the Rust tree rules,
and the #275 external-change conflict machinery stays the no-silent-clobber net either
way. Edits at excerpt BOUNDARIES are explicitly designed (grow, clamp, or reject — pinned
by test), the class of bug half-open runs exist to keep killable.

## Acceptance

Headline: typing inside an excerpt changes the real file's Buffer (visible in the file's
own tab), undo in the multibuffer reverts it, ⌘S persists it to disk through the existing
save path, and an external change to an excerpted file surfaces the standing conflict
behavior instead of silently clobbering. Full EARS at promotion.

## React-first (parity)

UI-affecting — the POC `MultibufferView` gains the editable interaction (caret in an
excerpt, live write-through to the shared doc store, dirty markers), inspected in the
browser before the Rust port.
