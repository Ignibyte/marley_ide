---
pipeline_id: b93e4775-d108-461b-afbe-cbe2cef99f72
ticket: forge#314 (7104b4be-714d-46d4-b795-ba67a2a2959a) · local docs/planning/tickets/open/TICKET-314-lsp-formatting.md
aar_id: 1f05c50e-006e-44ae-93ca-8e848c6368ac
status: Phase 5 — Complete PASS
title: LSP formatting — ⌥⇧F format-document + opt-in format-on-save, the save never blocked
type: feature
milestone: M20
references: [apply_text_edits — its OWN doc names it "the #314-reusable single-document primitive" (workspace_edit.rs:215), apply_one_file's begin/end_undo_group reverse-offset apply (app.rs:8191), the capability-reader template inlay_hint_support (inlay.rs:118 + the gate at app.rs:3603), the version-keyed stale-guard family (CompletionKey/InlayKey), save_active is SYNCHRONOUS with did_save POST-write (app.rs:6384/:6445 — the async-format tension), FormattingOptions truth: tabSize=code_tab_width(4, a real setting), insertSpaces=true (Marley only ever inserts spaces — indent.rs:84), ⌥⇧F verified FREE, the #330/#331 settings-toggle recipe (settings.rs:405), REQUEST_TIMEOUT_TICKS=10s (lsp_host.rs:70)]
---

## Title
⌥⇧F formats the document through rust-analyzer (`textDocument/formatting` → rustfmt); an opt-in
`editor.format_on_save` (default OFF) runs it before every ⌘S. Two invariants carry the ticket: **the
apply is transactional** (one undo unit, ⌘Z restores the pre-format text byte-identically; an invalid
edit batch applies NOTHING), and **the save is never blocked** — a slow or absent formatter degrades to a
plain save with a quiet flash, on a short deadline.

**The good news the recon proved:** the hard half already shipped. `apply_text_edits` (pure: resolve →
sort last-to-first → reject Overlap/Inverted → splice) carries a doc comment naming it *"the
#314-reusable single-document primitive"*, and `apply_one_file` already wraps it in ONE
`begin_undo_group`/`end_undo_group` over open-buffer edits. This ticket is the request, the gates, the
save orchestration, and the caret story — not an edit engine.

## Scope
### In
- **The request:** `RequestPurpose::Formatting(FormattingKey { uri, version })` — **version-keyed** (the
  Completion/Inlay pattern): a response for a stale version DROPS (a whole-file rewrite addressed at text
  the user has since edited must never apply). Params = `{ textDocument, options: { tabSize:
  code_tab_width, insertSpaces: true } }` — **honest values**: `code_tab_width` is a real setting
  (default 4) and Marley only ever inserts spaces (indent.rs pads; verified). rust-analyzer largely
  ignores them (rustfmt-backed); send truth anyway.
- **The capability gate:** `formatting_support(server_caps)` reading `documentFormattingProvider`
  (object-or-`true`) — the `inlay_hint_support` template verbatim + an `LspHost` wrapper. No capability /
  no Ready host → ⌥⇧F flashes quietly; **format-on-save falls through to a plain save instantly**.
- **The apply + the caret:** route the `Vec<TextEdit>` through the SHIPPED single-doc path (resolve via
  the #309 encoding bridge → `begin_undo_group` → reverse-offset `edit(..., EditOrigin::Agent)` →
  `end_undo_group`). Granular edits preserve the caret for free (the buffer shifts selections through
  each edit). **The degenerate case — one whole-file TextEdit — collapses the caret**, so: capture the
  caret as an `Anchor` before apply and resolve after (anchor.rs, shipped; bias Left). Then
  `follow_editor_caret()` keeps its line in view. A format that teleports you to line 1 is the shipped
  bug in real editors this REQ pins against.
- **Format-on-save (the async orchestration — the recon's named tension):** `save_active` is SYNCHRONOUS
  and `did_save` fires post-write; formatting is an async round-trip. So the "save" dispatch arm becomes:
  flag ON + capability + Ready → send Formatting, park a `pending_save_after_format` latch (with the
  buffer's version), **do not write yet**; the response (version-guarded) applies the edits then calls
  the real `save_active` — ONE write, disk gets the formatted text, `didSave` fires after, buffer ends
  CLEAN. A **~2 s deadline** (a pump-tick counter on the latch — the #203 notify_ticks idiom; NOT the
  global 10 s request timeout) or an Err → plain `save_active` + a quiet flash: **a save is never lost
  and never blocked.** An EDIT while the latch is parked supersedes it (version mismatch) → plain save.
  ⌥⇧F alone leaves the buffer DIRTY (it is an edit); format-on-save ends clean (it saved).
- **Setting + palette + chord:** `define_setting!(FormatOnSave: bool = false, "editor.format_on_save")` +
  `persist_format_on_save` + the AppliedSettings plumbing (the #330 recipe); the round-trip test's
  NON-DEFAULT leg persists `true` (default-false — the standing trap inverted). Palette: "Format
  Document" + "Toggle Format on Save" (+ `action_for_command` arms; the resolves-to-a-verb test).
  ⌥⇧F `(F,F,T,T,"f")` verified FREE, Editor-scoped (roster 67→68, scoped 22→23, individual assert first).
### Out (explicitly)
- Range formatting (`textDocument/rangeFormatting`); on-type formatting; organize-imports (a code action
  — #323 shipped the engine); formatter CHOICE (rust-analyzer's is the formatter); non-LSP fallback
  formatting; changing the 10 s global timeout.

## Reference (§20)
LSP 3.17 published spec (`textDocument/formatting`, `FormattingOptions`, `documentFormattingProvider`;
the last-to-first apply is the canonical client algorithm). VS Code = OBSERVED (⌥⇧F; format-on-save
ordering: format → one write; save never blocked by a dead formatter).

### Prior art
1. **Behavior maps / observed** — the VS Code semantics above, incl. dirty-state truth (manual format =
   dirty; format-on-save = clean).
2. **Published material** — LSP 3.17: edits address the ORIGINAL document; overlapping edits are invalid
   (the reject-whole-batch rule comes from the spec, not caution).
3. **OUR OWN CODE — the sweep found the engine already built:** `apply_text_edits` is documented as
   #314-reusable (workspace_edit.rs:215) with the overlap/inverted rejection already tested;
   `apply_one_file` owns the one-undo-group open-buffer apply; the capability-reader and stale-guard
   patterns are shipped templates (#331/#313); `anchor.rs` ships the caret carry (zero consumers —
   shared first-consumer risk with #305, whichever lands first). The genuine delta: the two purposes,
   the gates, and the save orchestration.

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-REUSE-THE-322-APPLY** — no new edit engine; the single-doc primitive + undo-group path as shipped.
- **D-VERSION-KEYED-STALE-GUARD** — a format answer applies only to the version it was asked about.
- **D-SAVE-NEVER-BLOCKED** — the ~2 s latch deadline (own tick counter) → plain save + flash; an error →
  plain save; an intervening edit → plain save. Losing a save is the one unforgivable outcome.
- **D-ONE-WRITE** — format-on-save produces exactly one disk write, of the formatted text, then `didSave`.
- **D-CARET-ANCHOR-FALLBACK** — granular edits carry the caret via the buffer; the whole-file-edit
  degenerate resolves through an Anchor; follow keeps the line in view.
- **D-HONEST-OPTIONS** — tabSize/insertSpaces reflect what Marley actually does.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | apply a server edit batch last-to-first as ONE undo unit — ⌘Z restores the pre-format text byte-identically | pure (shipped tests extend) + headless |
| REQ-002 | reject an overlapping/inverted batch WHOLE (original text untouched) | pure — the LSP-spec pin |
| REQ-003 | keep the caret on its line through a format (granular edits; and the whole-file-edit degenerate via the Anchor) | headless |
| REQ-004 | drop a formatting response whose version is stale (an edit raced it) | headless — the guard row |
| REQ-005 | flash quietly and do nothing on ⌥⇧F with no capability/Ready host | headless |
| REQ-006 | with format_on_save ON: format → apply → ONE write of formatted text → didSave → buffer clean | headless (fake_ls lane) |
| REQ-007 | with the formatter slow/dead: save PLAINLY within the ~2 s deadline + flash — the save is never lost | headless — the deadline row |
| REQ-008 | leave the buffer dirty after ⌥⇧F alone (an edit, not a save) | headless |
| REQ-009 | round-trip editor.format_on_save with the NON-default (true) leg | settings unit |
| REQ-010 | resolve ⌥⇧F + both palette rows to verbs (roster/palette guards) | unit |

## Phase Plan
P2 confirm the fake_ls test lane can serve `textDocument/formatting` (extend the fixture server if not —
the #331 lane precedent) + the latch's exact tick budget + where the "save" arm forks; P3 the purposes/
gates/params first, then the apply + caret, then the save orchestration + setting + chord; P3.5 critics on
the latch races (edit-during-format, second ⌘S during format, quit-during-format), the one-write
invariant, the caret degenerate; P4 units + the fake_ls drives + gate; P5 docs (editor.md's LSP table +
crate-map). Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
