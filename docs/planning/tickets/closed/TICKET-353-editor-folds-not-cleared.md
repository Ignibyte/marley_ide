# TICKET-353 — editor_folds not cleared on file/tab close (#305 follow-up)

- **Ticket:** LOCAL #353 (bug, M19)
- **Tags:** M19, editor, folding, low-priority, follow-up, 305
- **Created:** 2026-07-18
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id 59bcba36-0f1a-436d-b498-dfe230d80611)
- **Status:** closed (2026-08-10 — pipeline 353-editor-folds-clear-on-close; the census-guarded
  scrub shipped, gate green MSI 100; the sibling `git_marks` leak found at inspect is TICKET-412)

## Description

#305 code folding stores active folds in app.rs `editor_folds: HashMap<PathBuf, Vec<Anchor>>` (~app.rs:249). Every mutation is an insert/remove from the fold verbs — nothing clears it when a tab or file closes. Two consequences, both LOW (parked as not-a-Slice-1-blocker at #305 inspect; flagged by BOTH inspect critics): (1) a bounded anchor leak — one Vec<Anchor> per distinct-file-ever-folded lingers forever; (2) a stale-fold-on-reopen edge — reopening a path resolves the OLD anchors (stamped with the prior buffer instance) against the fresh buffer; the #305 F1 re-derive (fold_projection keeps a fold only if fold_regions still has a region at the resolved header) DROPS anchors not on a live region header, so a re-fold only if an old anchor happens to resolve onto a valid header. Fix: clear editor_folds[path] on tab/file close and on the path-key change (e.g., in the close/rename paths). No unbounded growth otherwise (fold_at_caret/toggle/fold_all all guard dup pushes; the projection merges). M19, low priority.
