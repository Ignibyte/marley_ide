# TICKET-298 — ⌘D adds the next occurrence as a cursor + ⌘⇧L select-all-occurrences

- **Forge ticket:** #298 `fb7ba9ea-3757-4b7e-a928-57bc5b21a07e` (feature, M19)
- **Owner:** autonomous /goal run (sprint #32 — M19 Editor power tools)
- **AAR:** `3adb9d1d-4e70-4ca3-a0ca-c7d281f19e1e`
- **Pipeline doc:** ../../pipeline/active/298-cmd-d-adds-a-cursor.spec.md
- **Depends on:** #296 (`135b439`) + #297 (`edb62d4`) — both SHIPPED
- **Status:** closed

## Summary
#272 shipped ⌘D as a single-selection *advance* — a live selection jumps to the next occurrence, replacing
itself. The universal behavior is **additive**: ⌘D KEEPS the existing selections and ADDS the next occurrence
as a new cursor, so repeated ⌘D builds a multi-selection you type over in one stroke. It is the most-used
multi-cursor gesture in any editor. Plus **⌘⇧L** — every occurrence at once.

Two pure seams (`add_next_occurrence`, `select_all_occurrences`) over search machinery #272 already built,
plus two keymap rows. The N-cursor set (#296), the N-band highlight and the N-cursor edit in one undo unit
(#297) all exist — this ticket *verifies* them rather than rebuilding them.

## Acceptance
Open a file with a repeated word → **⌘D ⌘D** → two highlighted occurrences → type → **both replaced in one
undo unit** → **⌘⇧L** → every occurrence selected. Proven on **live pixels**. Full EARS in the spec.

## Two things surfaced at plan time
- **⌘⇧L is already `split-right`** (M12.2 #197). Resolved by Editor-scoping it — the shipped shadow precedent
  (⌘D/⌘F/⌘A/⌘⌥↑↓). Cost: while the editor is focused, ⌘⇧L no longer splits the pane. That matches the
  reference and is the same trade #297 made.
- **⌘D and ⌘⇧L currently disagree about what an "occurrence" is** — `next_occurrence` is case-SENSITIVE,
  `find_all` is case-INSENSITIVE. Shipped naively, the two gestures in this very ticket would behave
  differently on the same text, and typing over the wrong match rewrites text the user never targeted. Both
  must be case-sensitive; the find bar keeps its folding.
