# TICKET-337 — Appearance settings: font size (+family) with ⌘= / ⌘− / ⌘0 live zoom

- **Forge ticket:** #337 169609bd-40e6-4e08-901d-2a86cfc15226 (feature, M22)
- **Owner:** claude (this session)
- **AAR:** d8d283b2-cfda-49e7-a41b-dd8fbfa8e340
- **Pipeline doc:** ../../pipeline/completed/337-appearance-font.spec.md
- **Source ticket:** M22 "The editing bar" batch (#336–340 + the 11 that follow) — the SECOND of the batch
  ([m22-editing-bar.md](../../design-notes/m22-editing-bar.md) · [roadmap.md](../../../marley_architecture/roadmap.md))
- **Status:** closed

## Summary
Nobody changes their font size by editing a Rust constant. `TERMINAL_FONT_SIZE: f32 = 13.0` is a hardcoded
`const` and — more importantly — it is the **one size behind BOTH surfaces**, so the editor and the terminal
share a single mono metric that no user can touch. This makes it `appearance.font_size` (default 13, clamped
[8, 32]) plus `appearance.font_family`, with ⌘= / ⌘− / ⌘0 live zoom that applies and persists immediately
(the #199 theme-picker's write-through posture). It is the first thing a developer reaches for in the first
ten minutes, and chad named it explicitly ("changing font size, text, themes").

The core is **D-ONE-METRICS-SEAM**: one cached `font_metrics(applied)` read replacing every consumer, so a
size change flows through a single place and the identifier `TERMINAL_FONT_SIZE` leaves the tree entirely (a
grep gate proves it). That is not tidiness — the codebase already has a stale-guard on record
(workspace.rs's non-positive default "was a stale 14"), which is exactly what a second source of truth buys
you. It is also **binding**: #336's `AD-claude-hscroll-shift-clip-split-and-two-probe-domains-001` records
that its x0 and code_w probes both feed on `cell_w`, and forbids #337 from introducing a third source.

A size change moves the cell size, which moves every terminal's cols/rows — so one zoom must re-grid every
live PTY through the EXISTING resize path, never a second resize mechanism.

## Acceptance
`appearance.font_size` resolves at boot into both surfaces and survives a NON-DEFAULT round-trip; ⌘=/⌘−/⌘0
apply live, clamp into [8,32] at every door, persist immediately, and re-grid every live PTY; an unresolvable
`font_family` falls back with a flash rather than a silent wrong font; and ZERO `TERMINAL_FONT_SIZE`
identifiers remain. Full EARS REQ-001..008 in the pipeline spec.
