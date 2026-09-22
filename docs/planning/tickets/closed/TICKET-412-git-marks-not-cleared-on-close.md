# TICKET-412 — git_marks not cleared on file close (the #353 sibling)

- **Ticket:** LOCAL #412 (bug, M19)
- **Owner:** —
- **Pipeline doc:** docs/planning/pipeline/completed/412-git-marks-clear-on-close.spec.md
- **Source ticket:** found by the #353 inspect critics (2026-08-10)
- **Status:** closed (2026-08-11 — shipped; entry + key scrub census-guarded in the #353 arm, six drives, gate green)

## Summary
`git_marks: HashMap<PathBuf, Vec<(usize, GitMark)>>` (app.rs ~:320) has the exact
leak class #353 fixed for `editor_folds`: populated per viewed file whenever the
active file has diff marks, removed only when that same file is ACTIVE again with
an empty diff — no close path scrubs it, so a file closed while carrying marks
leaks its entry for the app's life. Memory-only: a reopen self-heals the visible
state via `refresh_git_marks` (the #275 activation choke bumps `git_marks_gen`),
so there is no wrong-render — a bounded leak, LOW like #353 was. The #353 choke
(`RootView::release_editor_views`, census-guarded) is the natural scrub point;
`git_marks_key` needs the same look when it names the dropped path. Deferred out
of #353 (one shippable slice; the scrub lines need their own drive fixtures —
git-diff marks in the headless lane — to clear the mutation bar).

## Acceptance
Closing the last view of a file's instance removes its `git_marks` entry (census
semantics identical to #353's `editor_folds` scrub, incl. the alias-root
survivor); `git_marks_key` never dangles on a dropped path; drives prove both.
