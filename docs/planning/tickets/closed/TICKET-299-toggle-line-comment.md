# TICKET-299 — ⌘/ toggle line comment (language-aware, indent-aligned, multi-cursor aware)

- **Forge ticket:** #299 `3e5054ab-439f-43b3-b73f-4e43cba2ac2d` (feature, M19)
- **Owner:** autonomous /goal run (sprint #32 — M19 Editor power tools)
- **AAR:** `5fa3e15f-fc17-45aa-837a-d2282669414c`
- **Pipeline doc:** ../../pipeline/active/299-toggle-line-comment.spec.md
- **Depends on:** #296 (`135b439`) · #297 (`edb62d4`) · #298 (`bc27c70`) — all SHIPPED + PUSHED
- **Status:** closed

## Summary
Marley has no comment toggle — arguably the most-pressed editing chord after save. ⌘/ comments (or uncomments)
every line touched by every cursor, in ONE undo unit, with the markers **aligned at one column** (the minimum
indent of the touched lines) rather than staircased down each line's own indent. Blank lines are skipped.

Most of the machinery already exists: `indent.rs` ships `LineEdit`, `line_span` and `rebase_selections` for
exactly this shape — Tab/⇧Tab are the same kind of line-prefix edit. The genuine deltas are the **decision**
(direction · min-indent · blank skip · uncomment strip), the **deduped union of rows across N cursors**, and a
`pub` accessor on the comment-token table that **already exists** in `code_syntax.rs`.

## Acceptance
A ragged-indent block → ⌘/ → every marker at the SAME column → ⌘/ again → byte-identical → ⌘Z reverts in one
step. Then ⌘⌥↓ ×2 → ⌘/ → every cursor's row toggles at once. **Proven on live pixels.** Full EARS in the spec.

## Two things surfaced at plan time
- **The ticket's own premise is wrong.** It says the token comes from tree-sitter; tree-sitter is Rust-only and
  language-blind. The token table **already exists** (privately) in `code_syntax.rs`, alongside the editor's one
  real language seam. Building it as written would ship a second source of truth for "what is a line comment in
  Rust". Honest consequence: ⌘/ is a **no-op** on JSON / Markdown / Plain (no token in the table) — promoted to
  an acceptance criterion rather than buried.
- **The round trip is not what it looks like.** `/// foo` starts with `//`, so a block of *only* doc comments
  uncomments to `/ foo` — mangled, and not reversible. That is what VS Code and Zed both do (§20), and the
  ordinary flow is safe (a mixed block comments-all, so doc comments survive intact) — but it means
  "toggle twice is byte-identical" is **false as stated**. The guarantee is scoped to an uncommented starting
  state, and the `///` outcome gets its own pinning test so it is a decision, not an accident.
