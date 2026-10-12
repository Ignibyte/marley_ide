# TICKET-762 — The marley:// scheme and the desktop entry

- **Ticket:** LOCAL #762 (chore, the finish line)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [762-the-marley-scheme.spec.md](../../pipeline/completed/762-the-marley-scheme.spec.md)
- **Source ticket:** split from #445 (slice 4) on 2026-10-11, when Chad asked to "spec all" of the survey list. Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md).
- **Status:** closed

## Summary
`x-scheme-handler/marley` in `marley.desktop`, `marley://` parsed where `zed://` is, and the `zed://` links Marley opens handled in Marley rather than sent through `xdg-open` to stock Zed. #749's verbs ride the single-instance hand-off and need no desktop scheme; if #749 names its URLs `marley://`, this adds only the desktop's half.

## Acceptance
`xdg-open marley://open` brings Marley forward, and a `zed://` link clicked inside Marley opens in Marley.
