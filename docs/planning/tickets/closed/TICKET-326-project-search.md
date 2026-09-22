# TICKET-326 — Project-wide content search (⌘⇧F): grep file contents across the workspace

- **Forge ticket:** #326 992a1da5-7236-4e66-8bc2-e1557a672e78 (feature, M21)
- **Owner:** claude (this session)
- **AAR:** 206fb9a2-6683-47d2-bcc1-c7d687c1b0ac
- **Pipeline doc:** ../../pipeline/active/326-project-search.spec.md
- **Source ticket:** M21 "The workspace IDE" batch (#322-331); roadmap B7 (phase 1)
- **Status:** closed

## Summary
⌘⇧F searches file CONTENTS across the whole workspace — a needle, every matching line grouped by file,
Enter jumps there. The flagship gap by the roadmap's own sequencing (the step after LSP): nothing greps
across files today (`find.rs` is terminal scrollback, editor find is one buffer, ⌘P is file names). Roadmap
B7 **phase 1 = a read-only results surface** ("80% of the value with none of the multibuffer cost"); the
editable multibuffer / replace-all is phase 2, explicitly deferred. A PURE literal-substring engine (case +
whole-word toggles) over an off-thread walker (adopt the `ignore` crate vs extend `list_files_in` — the one
design-owned decision), dirty buffers searched via live text not stale disk, honest caps + "+N more" tail,
generation-cancel on re-query, and a finder-recipe picker (Enter → #312 open_and_place_caret + NavStack).
Works in every language day one — no LSP needed.

## Acceptance
⌘⇧F opens a query picker; a needle planted in two probe files yields both `path:line` rows grouped by file,
Enter lands centered (⌃- returns); editing one match away WITHOUT saving and re-searching finds N-1 (the
dirty-buffer proof). Pure `search_lines` at cov/MSI 100 (case/word toggles, multi-match rows, non-ASCII col
correctness, the skip rules, cap+tail, the dirty-vs-disk decision table); a stale-generation walk's results
dropped. Full EARS in the pipeline spec.
