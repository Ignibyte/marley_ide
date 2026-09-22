# TICKET-314 — LSP formatting (⌥⇧F format-document + opt-in format-on-save)

- **Forge ticket:** #314 7104b4be-714d-46d4-b795-ba67a2a2959a (feature, M20)
- **Owner:** session 99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c
- **AAR:** 1f05c50e-006e-44ae-93ca-8e848c6368ac
- **Pipeline doc:** ../../pipeline/active/314-lsp-formatting.spec.md
- **Source ticket:** M20 language-intelligence batch (the goal /work 300,302,303,304,305,314,315,316,317,259)
- **Status:** closed

## Summary
Format the active Rust document through rust-analyzer (`textDocument/formatting` → rustfmt): ⌥⇧F formats on
demand, and an opt-in `editor.format_on_save` (default OFF) formats before every ⌘S. Two invariants carry it —
the apply is **transactional** (one undo unit, ⌘Z restores byte-identically; an invalid edit batch applies
NOTHING), and the **save is never blocked** (a slow/absent/edit-raced formatter degrades to a plain save on a
~2 s deadline). The hard half — the single-document apply engine — already shipped with #322
(`apply_text_edits` + `apply_one_file`'s one-undo-group reverse-offset apply); this ticket is the request, the
capability + version gates, the never-block save orchestration, the caret carry, and the setting + chord.

## Acceptance
⌥⇧F formats via a version-keyed `textDocument/formatting`, applying the edits as one undo unit and keeping the
caret on its line; `editor.format_on_save` ON produces exactly ONE write of the formatted text then didSave with
the buffer clean, and NEVER loses or blocks a save. Full EARS criteria (REQ-001..010) in the pipeline spec.
