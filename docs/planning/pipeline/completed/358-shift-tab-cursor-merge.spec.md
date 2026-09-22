---
pipeline_id: ead13037-dd2c-45cd-9232-743f5c68c7ad
ticket: forge#358 (ae7dbf9e-c8ef-4b48-87fb-5ad1ac5f0349) · local docs/planning/tickets/open/TICKET-358-shift-tab-cursor-merge.md
aar_id: d08e90f8-d95a-45e0-a8e8-a35d4d62fd67
status: Phase 5 — Complete PASS
title: ⇧Tab merges two cursors inside the same row's indentation (2→1) — the recon dissolved the fork to accept+document
type: bug
milestone: M22
references: [indent.rs:150 rebase_through / :158 the in-removed-span clamp / :175 rebase_selections, selection.rs overlaps (the `<=` caret law) + SelectionSet::from_selections, headless_drive.rs:8192 shift_tab_dedents_every_cursors_line + :6917 set_editor_carets + :403 editor_selections, #307 multi-row dedent, #297 inspect C1 silent-cursor-loss scar]
---

## Title
Two carets that both sit INSIDE the same row's leading indentation merge to one cursor on ⇧Tab. **The recon
dissolved the design fork: accept + document (Opt 1) is the only correct answer** — the merge is topologically
forced (the columns the carets held are deleted), Opt 3 is a proven no-op, and on an edit ⌘Z restores.

## Scope
### In
- A documenting headless drive: 2 carets inside one row's indent → ⇧Tab → exactly 1 cursor at the new line start;
  ⌘Z restores the 2-cursor set (the #297-scar distinction — this is an EDIT, recoverable).
- A regression guard: 3 carets on 3 separately-indented rows → ⇧Tab → all 3 survive (the common case stays safe).
- A doc note (`rebase_through`/`rebase_selections` in indent.rs and/or the `overlaps` law in selection.rs) that the
  in-removed-span collapse to one cursor is CORRECT — the merged columns no longer exist, and an edit's ⌘Z restores.

### Out (explicitly deferred)
- **Opt 3 (clamp differently) — REJECTED, it is a no-op** (see D2). No `rebase_through` behavior change.
- **Opt 2 (co-located cursors) — REJECTED**, it contradicts `SelectionSet::from_selections`' reason to exist.
- Any change to the merge / `overlaps` law, or to `rebase_through`'s clamp. This ticket ships NO behavior change —
  the behavior is already correct; the deliverable is the proof + the documentation.

## Reference (§20)
N/A — Marley-specific. ⇧Tab dedent + multi-cursor merge is Marley's own editor behavior (no Warp/Zed source read).
The "cursors in deleted whitespace collapse to one" outcome matches mainstream editors (observed, e.g. VS Code),
which is confirmation, not a source.

### Prior art
1. **OUR OWN CODE (the mechanism is ours + already correct):** `rebase_through` (indent.rs:150) with the
   in-removed-span clamp (:158 — `if p < at + remove { return at + delta }`, the #269 covering-collapse convention);
   `rebase_selections` (:175, the #297 inspect C11 fix that carries EVERY cursor through the edit rather than
   silently dropping the non-primary ones); `SelectionSet::from_selections` + `overlaps` (selection.rs, the `<=`
   caret arm) merges same-offset carets by law. The #307 multi-row dedent (headless_drive.rs:8192) is the reach that
   made the same-row case possible; the #297 inspect C1 scar (a merge that "correctly" destroyed a cursor on a
   MOTION, unrecoverable) is why this is filed rather than shrugged off.
2. **gpui / ropey:** checked — no owner; multi-cursor merge is Marley's `SelectionSet` invariant, not a crate seam.

## Locked-In Decisions
- **D1-ACCEPT (the merge is correct, topologically forced).** After ⇧Tab removes the N indent chars, the columns
  the two carets occupied ([line_start, line_start+N)) no longer exist. There is no distinct position for a second
  cursor to land — one cursor at the new line start is the only well-defined outcome. `from_selections` merging
  same-offset carets is the crate's law (leaving them separate would make that one logical cursor insert its text
  twice). It is an EDIT, so ⌘Z restores the full 2-cursor set — the recoverability that makes this correct-not-a-bug
  (unlike the #297 MOTION scar).
- **D2-OPT3-IS-A-NO-OP (traced on the real fn).** The ticket's Opt 3 `line_start + (col - stripped).max(0)`: for a
  caret INSIDE the removed span, `col = p - at` is BY DEFINITION `< remove` (that is what "inside [at, at+remove)"
  means), so `(col - remove).max(0)` is ALWAYS 0 → `at + 0` = `at` = the current clamp EXACTLY. For a caret OUTSIDE
  the span, `rebase_through` never reaches the clamp arm (it takes the `p + delta` path). So Opt 3 changes NOTHING
  for any caret — it is a pure no-op, not even a partial fix. No clamp formula can keep two carets in a deleted
  region distinct (the positions are gone). The fork dissolves to Opt 1.
- **D3-NO-BEHAVIOR-CHANGE.** This pipeline adds a headless drive + a regression guard + a doc note; it touches NO
  `rebase_through`/`overlaps`/merge logic. The deliverable is proof + documentation of already-correct behavior — a
  valid shippable slice (like a measured-defer).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-MERGE-CORRECT | WHEN two carets both sit inside one row's leading indentation and ⇧Tab dedents that row, the system shall leave exactly ONE cursor at the new line start. | headless drive: `"aa\n    bb\ncc\n"`, carets [4, 6] → `shift-tab` → text `"aa\nbb\ncc\n"`, `editor_selections().len() == 1` at the new line start. |
| REQ-RECOVERABLE | WHEN the merge above has happened, ONE ⌘Z shall restore the original text AND the 2-cursor set. | same drive: `cmd-z` → text restored AND `editor_selections().len() == 2` (the #297-scar distinction — an edit is recoverable). |
| REQ-COMMON-CASE-SAFE | WHEN N carets sit on N separately-indented rows and ⇧Tab dedents, the system shall keep all N cursors. | headless drive: `"    aa\n    bb\n    cc\n"`, carets [4, 11, 18] → `shift-tab` → `editor_selections().len() == 3`. |

## Phase Plan
- **P2 Design** — ratify D1/D2/D3 (confirm the Opt-3-is-a-no-op arithmetic on `rebase_through`'s exact body; confirm
  the merge is `from_selections`/`overlaps`); the exact headless drive plan (the merge + ⌘Z-restore + the 3-row
  guard, all SYNCHRONOUS — ⇧Tab is an immediate edit, NO pump tick, unlike #349); the doc-note site.
- **P3 Implement** — the doc note (indent.rs `rebase_selections`/`rebase_through` and/or selection.rs `overlaps`).
  No behavior code. (The tests are Phase 4.)
- **P3.5 Inspect** — ★ the #297 scar: does any OTHER cursor silently vanish? confirm the count assertions catch it;
  confirm the doc claims only what the code keeps; confirm Opt 3 is genuinely a no-op (no reviewer surprise).
- **P4 Validate** — the 3 headless drives + the `--diff` gate (no new pure-fn surface → cov/MSI unaffected; the app
  ⇧Tab arm is the mutants::skip shim the drives carry; `rebase_through`/`rebase_selections` tests stay green).
- **P5 Complete** — CHANGELOG (a documented-behavior note, not a fix) + editor.md (the multi-cursor dedent merge).
