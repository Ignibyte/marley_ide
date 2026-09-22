# TICKET-324 — LSP signature help: parameter hints while you type a call

- **Forge ticket:** #324 aa432c80-2bf4-40f9-acb5-e02f99416208 (feature, M21)
- **Owner:** claude (this session)
- **AAR:** ffda3cbe-8baa-49fc-8ea1-c5dbc5661fe4
- **Pipeline doc:** ../../pipeline/completed/324-lsp-signature-help.spec.md
- **Source ticket:** M21 "The workspace IDE" batch (#322-331)
- **Status:** closed

## Summary
Type `foo(` and a card appears above the caret with the call's signature and the ACTIVE parameter lit,
updating as you type commas and vanishing when the call closes — the third leg of the as-you-type triad
(completions #313 / signature help / inlay hints #331). Small by design: the #313 park-and-consume shape
with a card instead of a menu. Carries the #323 lesson forward — advertise `textDocument.signatureHelp`
(`labelOffsetSupport` + `activeParameterSupport`) or a real server withholds the offset labels + per-sig
active param.

## Acceptance
⌘⇧Space / a `(`|`,` trigger at a call sends `textDocument/signatureHelp`; the pure parser clamps both
indices (never panics), overrides top-level active-param per signature, and normalizes string-or-offset
parameter labels (OOB dropped); a card renders ABOVE the caret with the active param highlighted + an
"N of M" pip; the suppression gate stays quiet in strings/comments (positive control); the live-identity
poll dismisses; the handshake advertises the capability (live-proven the offset form arrives). Full EARS
criteria in the pipeline spec.
