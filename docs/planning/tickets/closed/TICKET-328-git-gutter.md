# TICKET-328 — Git gutter: added/modified/deleted markers against HEAD in the editor

- **Forge ticket:** #328 37165c5a-02bb-43f6-856e-530301b163a8 (feature, M21)
- **Owner:** claude (this session)
- **AAR:** 34325337-3b6a-4eb3-a1f6-d1cf92625517
- **Pipeline doc:** ../../pipeline/active/328-git-gutter.spec.md
- **Source ticket:** M21 "The workspace IDE" batch (#322-331)
- **Status:** closed

## Summary
The editor learns what changed: a colored bar in the gutter per edited line — added (success), modified
(warning-tone), a deleted-run caret (danger) — against HEAD. Today git surfaces only as ⌘⇧D + the
changes/commit panel; the editor gutter knows nothing per-line. A PURE `gutter_marks_from_hunks` projects the
shipped `git_diff::parse_diff` output (route A — parse the `@@` header's new-side start, walk +/-/context,
classify added/modified-run/deleted-between-rows; ZERO new deps), a DECOUPLED per-path mark store (never on
`OpenFile`), a second gutter lane (a 3px bar coexisting with the diagnostic tint), refreshed on save +
external reload (marks reflect the SAVED working-tree-vs-HEAD state while dirty; route B live-diff deferred),
cached per `(path, buffer-version, head-oid)`.

## Acceptance
`gutter_marks_from_hunks` at cov/MSI 100 (pure-add → Added, replace-run → Modified, pure-delete → a
between-rows Deleted marker, the `@@`-header new-start parse, untracked → all-added, malformed → no marks).
Integration (git tempdir: init + commit + edit → marks; revert → clear). LIVE (seeded bundle): add + change +
delete a line in a tracked probe file → three distinct colored bars at the right rows (pixel-sampled); ⌘S
persists; revert clears. Full EARS in the pipeline spec.
